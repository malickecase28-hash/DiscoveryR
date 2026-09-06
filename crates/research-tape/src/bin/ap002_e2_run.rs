//! AP-002 E2 runner: evaluates the entire frozen 472-cell same-domain universe
//! from a single feature materialization over the active E1 evidence
//! (E1_ZONES.jsonl + E1B_ANCHORS.jsonl + confirmation-bar ranges from the
//! corrected development view).  All statistics are predeclared in
//! AP-002_E2_CELL_REGISTRY.json / AP-002_E2_CONTRACT_V1_1.json; no outcome may
//! influence any statistical choice.  DEVELOPMENT only; confirmation LOCKED.

use arrow_array::Array;
use research_tape::{batch_f64, batch_i64, ProjectedParquetReader};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process,
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const BAR_MS: [i64; 7] = [
    15_000, 30_000, 60_000, 300_000, 900_000, 3_600_000, 14_400_000,
];
const HORIZONS: [usize; 3] = [1, 3, 5];
const B: u64 = 4_999;
const ROLLING_K: usize = 20;
const WARMUP_BARS: i64 = 288;

fn tf_index(timeframe: &str) -> usize {
    TIMEFRAMES
        .iter()
        .position(|tf| *tf == timeframe)
        .expect("known timeframe")
}

// ---------------- deterministic RNG (splitmix64 over SHA-256 seeds) ----------------

struct Rng(u64);
impl Rng {
    fn from_seed(seed: &[u8]) -> Self {
        let digest = Sha256::digest(seed);
        let state = u64::from_le_bytes(digest[0..8].try_into().expect("8 bytes"));
        Rng(state)
    }
    fn next_u64(&mut self) -> u64 {
        let mut z = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.0 = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

// ---------------- data model ----------------

#[derive(Debug, Clone, Deserialize)]
struct Zone {
    timeframe: String,
    #[serde(rename = "identity")]
    _identity: String,
    formation: FormationFields,
    formation_available_time_ns: i64,
    #[serde(rename = "anchor_source_sequence")]
    _anchor_source_sequence: i64,
    first_touch_ts: Option<i64>,
    fill_ts: Option<i64>,
    formation_to_touch_market_ms: Option<i64>,
    formation_to_fill_market_ms: Option<i64>,
    prospective_outcomes: [Option<Outcome>; 3],
    #[serde(rename = "active")]
    _active: bool,
    right_censored: bool,
}
#[derive(Debug, Clone, Deserialize)]
struct FormationFields {
    zone_id: u64,
    direction: String,
    detection_ts: i64,
    lower: f64,
    upper: f64,
    formation_atr: f64,
    gap_atr: f64,
    fvg_quality: f64,
    impulse_body_atr: f64,
}
#[derive(Debug, Clone, Deserialize)]
struct Outcome {
    raw_return_bps: f64,
    direction_adjusted_return_bps: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct Anchor {
    stage: String,
    identity: String,
    direction: String,
    anchor_bar_close_ts: i64,
    first_touch_observed: Option<bool>,
    outcomes: [Option<AnchorOutcome>; 3],
}
#[derive(Debug, Clone, Deserialize)]
struct AnchorOutcome {
    raw_return_bps: f64,
    direction_adjusted_return_bps: f64,
}

struct Features {
    /// per stratum: zones ordered by (detection_ts, zone_id)
    zones: Vec<Vec<Zone>>,
    /// confirmation bar range / formation_atr per (stratum, detection_ts)
    confirmation_range: HashMap<(usize, i64), f64>,
    /// FT anchors: (stratum, zone_id) -> anchor
    ft: HashMap<(usize, u64), Anchor>,
    /// FILL anchors: (stratum, zone_id) -> anchor
    fill: HashMap<(usize, u64), Anchor>,
}

struct Loader {
    zones_path: PathBuf,
    anchors_path: PathBuf,
    view_root: PathBuf,
}

impl Loader {
    fn load(&self) -> Result<Features, Box<dyn std::error::Error>> {
        let mut zones: Vec<Vec<Zone>> = vec![Vec::new(); 7];
        let reader = BufReader::new(File::open(&self.zones_path)?);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let zone: Zone = serde_json::from_str(&line)?;
            zones[tf_index(&zone.timeframe)].push(zone);
        }
        for stratum in &mut zones {
            stratum.sort_by_key(|zone| (zone.formation.detection_ts, zone.formation.zone_id));
        }
        // confirmation-bar ranges from the corrected view
        let mut wanted: HashSet<(usize, i64)> = HashSet::new();
        for (index, stratum) in zones.iter().enumerate() {
            for zone in stratum {
                wanted.insert((index, zone.formation.detection_ts));
            }
        }
        let mut confirmation_range: HashMap<(usize, i64), f64> = HashMap::new();
        let manifest: Value =
            serde_json::from_reader(File::open(self.view_root.join("view_manifest.json"))?)?;
        for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
            for part in manifest["source_groups"][*timeframe]
                .as_array()
                .ok_or("missing source group")?
            {
                let path = self
                    .view_root
                    .join(part["logical_path"].as_str().ok_or("path")?);
                let reader = ProjectedParquetReader::new(
                    path,
                    ["bar_close_ts", "high", "low"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    8192,
                )?;
                let scan = reader.scan()?;
                for batch in scan {
                    let batch = batch?;
                    let close = batch_i64(&batch, "bar_close_ts")?;
                    let high = batch_f64(&batch, "high")?;
                    let low = batch_f64(&batch, "low")?;
                    for row in 0..batch.num_rows() {
                        let close_ts = close.value(row);
                        let key = (index, close_ts);
                        if wanted.contains(&key) && !high.is_null(row) && !low.is_null(row) {
                            confirmation_range.insert(key, high.value(row) - low.value(row));
                        }
                    }
                }
            }
        }
        // anchors
        let mut ft: HashMap<(usize, u64), Anchor> = HashMap::new();
        let mut fill: HashMap<(usize, u64), Anchor> = HashMap::new();
        let reader = BufReader::new(File::open(&self.anchors_path)?);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let anchor: Anchor = serde_json::from_str(&line)?;
            let timeframe = anchor
                .identity
                .split(':')
                .next()
                .unwrap_or_default()
                .to_string();
            let zone_id: u64 = anchor
                .identity
                .rsplit(':')
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let key = (tf_index(&timeframe), zone_id);
            if anchor.stage == "FIRST_TOUCH" {
                ft.insert(key, anchor);
            } else if anchor.stage == "FILL" {
                fill.insert(key, anchor);
            }
        }
        Ok(Features {
            zones,
            confirmation_range,
            ft,
            fill,
        })
    }
}

// ---------------- feature derivation per zone ----------------

struct ZoneFeatures {
    gap_atr: f64,
    impulse_body_atr: f64,
    confirmation_range_atr: Option<f64>,
    active_overlap_count: u64,
    containment: bool,
    nearest_edge_atr: f64,
    rolling_fill_fraction: Option<f64>,
    direction_bullish: bool,
    formation_ts: i64,
    touch_within: [Option<bool>; 5], // 4,16 bars touch; fill16|touched; surv64; touch16 for stage cells
    fill_ts: Option<i64>,
    touch_ts: Option<i64>,
    gap_through: bool,
}

fn derive_features(features: &Features) -> Vec<Vec<ZoneFeatures>> {
    let mut out: Vec<Vec<ZoneFeatures>> = Vec::new();
    for (index, stratum) in features.zones.iter().enumerate() {
        let period = BAR_MS[index];
        // active-zone book as formation sweep proceeds
        let mut derived: Vec<ZoneFeatures> = Vec::with_capacity(stratum.len());
        let mut formation_order: Vec<(i64, u64)> = Vec::new();
        let mut completed_flags: Vec<bool> = Vec::new();
        for zone in stratum {
            let now = zone.formation.detection_ts;
            // active set: zones formed strictly before `now` (excluding self) and not yet filled by `now`
            let mut overlap = 0_u64;
            let mut containment = false;
            let mut nearest_edge = f64::INFINITY;
            for (other_idx, other) in stratum.iter().enumerate() {
                if other.formation.zone_id == zone.formation.zone_id {
                    continue;
                }
                let formed_before = other.formation.detection_ts < now;
                let alive = other.fill_ts.map_or(true, |fill| fill > now);
                if !(formed_before && alive) {
                    continue;
                }
                let overlaps = other.formation.lower <= zone.formation.upper
                    && zone.formation.lower <= other.formation.upper;
                if overlaps {
                    overlap += 1;
                    if other.formation.lower <= zone.formation.lower
                        && zone.formation.upper <= other.formation.upper
                    {
                        containment = true;
                    }
                    let edge_distance = if overlaps {
                        0.0
                    } else if zone.formation.upper < other.formation.lower {
                        other.formation.lower - zone.formation.upper
                    } else {
                        zone.formation.lower - other.formation.upper
                    };
                    nearest_edge = nearest_edge.min(edge_distance);
                } else {
                    let gap = if zone.formation.upper < other.formation.lower {
                        other.formation.lower - zone.formation.upper
                    } else {
                        zone.formation.lower - other.formation.upper
                    };
                    nearest_edge = nearest_edge.min(gap.abs());
                }
                let _ = other_idx;
            }
            // rolling fill fraction over prior K formed zones whose fill status
            // is known at `now` (causally available)
            let prior_completed: Vec<bool> = (0..formation_order.len())
                .map(|zone_index| {
                    let other = &stratum[zone_index];
                    other.fill_ts.map_or(false, |fill| fill <= now)
                })
                .collect();
            let rolling = if formation_order.len() >= ROLLING_K {
                let window = &prior_completed[prior_completed.len() - ROLLING_K..];
                Some(window.iter().filter(|done| **done).count() as f64 / ROLLING_K as f64)
            } else {
                None
            };
            // transitions
            let touch16 = zone
                .first_touch_ts
                .map(|touch| touch - now <= 16 * period)
                .unwrap_or(false);
            let touch4 = zone
                .first_touch_ts
                .map(|touch| touch - now <= 4 * period)
                .unwrap_or(false);
            let fill16_given_touched = zone
                .first_touch_ts
                .and_then(|touch| zone.fill_ts.map(|fill| fill - touch <= 16 * period));
            let surv64 = zone.fill_ts.map_or(true, |fill| fill - now > 64 * period);
            let gap_through = zone.fill_ts.is_some() && zone.first_touch_ts.is_none();
            let confirmation_range_atr = features
                .confirmation_range
                .get(&(index, now))
                .map(|range| range / zone.formation.formation_atr);
            derived.push(ZoneFeatures {
                gap_atr: zone.formation.gap_atr,
                impulse_body_atr: zone.formation.impulse_body_atr,
                confirmation_range_atr,
                active_overlap_count: overlap,
                containment,
                nearest_edge_atr: if nearest_edge.is_finite() {
                    nearest_edge / zone.formation.formation_atr
                } else {
                    f64::NAN
                },
                rolling_fill_fraction: rolling,
                direction_bullish: zone.formation.direction == "bullish",
                formation_ts: now,
                touch_within: [
                    Some(touch4),
                    Some(touch16),
                    fill16_given_touched,
                    Some(surv64),
                    Some(touch16),
                ],
                fill_ts: zone.fill_ts,
                touch_ts: zone.first_touch_ts,
                gap_through,
            });
            formation_order.push((now, zone.formation.zone_id));
            completed_flags.push(zone.fill_ts.is_some());
        }
        out.push(derived);
    }
    out
}

// ---------------- statistics ----------------

fn hypergeometric_pmf(n_total: u64, successes: u64, draws: u64) -> Vec<(u64, f64)> {
    // pmf over k = number of successes drawn, computed via log-gamma ratios
    fn ln_gamma(x: f64) -> f64 {
        // Lanczos approximation
        const G: f64 = 7.0;
        const C: [f64; 9] = [
            0.999_999_999_999_809_93,
            676.520_368_121_885_1,
            -1_259.139_216_722_402_8,
            771.323_428_777_653_13,
            -176.615_029_162_140_59,
            12.507_343_278_686_905,
            -0.138_571_095_265_720_12,
            9.984_369_578_019_571_6e-6,
            1.505_632_735_149_311_6e-7,
        ];
        if x < 0.5 {
            std::f64::consts::PI / (std::f64::consts::PI * x).sin() - ln_gamma(1.0 - x)
        } else {
            let x = x - 1.0;
            let mut a = C[0];
            let t = x + G + 0.5;
            for (i, c) in C.iter().enumerate().skip(1) {
                a += c / (x + i as f64);
            }
            0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
        }
    }
    fn ln_choose(n: u64, k: u64) -> f64 {
        if k > n {
            return f64::NEG_INFINITY;
        }
        ln_gamma(n as f64 + 1.0) - ln_gamma(k as f64 + 1.0) - ln_gamma((n - k) as f64 + 1.0)
    }
    let k_min = successes.saturating_sub(n_total - draws);
    let k_max = successes.min(draws);
    let mut pmf = Vec::new();
    if k_max < k_min {
        return pmf;
    }
    let log_norm = ln_choose(n_total, draws);
    for k in k_min..=k_max {
        let log_p = ln_choose(successes, k) + ln_choose(n_total - successes, draws - k) - log_norm;
        pmf.push((k, log_p.exp()));
    }
    pmf
}

/// Exact two-sided label-permutation p for a risk-difference contrast,
/// sampling the seeded hypergeometric permutation distribution B times.
fn permutation_p_riskdiff(
    n_total: u64,
    successes: u64,
    n_high: u64,
    observed_effect: f64,
    seed: &[u8],
) -> f64 {
    let pmf = hypergeometric_pmf(n_total, successes, n_high);
    if pmf.is_empty() {
        return 1.0;
    }
    let mut cdf: Vec<f64> = Vec::with_capacity(pmf.len());
    let mut cumulative = 0.0;
    for (_, p) in &pmf {
        cumulative += p;
        cdf.push(cumulative.min(1.0));
    }
    let p_low = n_total - n_high;
    let effect_at = |k_successes_high: u64| -> f64 {
        (k_successes_high as f64 / n_high as f64)
            - ((successes - k_successes_high) as f64 / p_low.max(1) as f64)
    };
    let observed_abs = observed_effect.abs();
    let mut rng = Rng::from_seed(seed);
    let mut extreme = 0_u64;
    for _ in 0..B {
        let u = rng.next_u64() as f64 / u64::MAX as f64;
        let position = cdf.partition_point(|value| *value < u).min(cdf.len() - 1);
        let k = pmf[position].0;
        if (effect_at(k) - observed_effect).abs() >= observed_abs - 1e-12 {
            extreme += 1;
        }
    }
    (1 + extreme) as f64 / (B + 1) as f64
}

fn median_of(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Sign-flip p for the median of a paired/symmetric sample.
fn signflip_p(values: &[f64], seed: &[u8]) -> (f64, f64) {
    let mut scratch: Vec<f64> = Vec::with_capacity(values.len());
    let observed = {
        let mut copy = values.to_vec();
        median_of(&mut copy)
    };
    let mut rng = Rng::from_seed(seed);
    let mut extreme = 0_u64;
    for _ in 0..B {
        scratch.clear();
        for value in values {
            if rng.below(2) == 0 {
                scratch.push(*value);
            } else {
                scratch.push(-*value);
            }
        }
        let median = median_of(&mut scratch);
        if median.abs() >= observed.abs() - 1e-12 {
            extreme += 1;
        }
    }
    let p = (1 + extreme) as f64 / (B + 1) as f64;
    (observed, p)
}

fn seed_for(contract: &str, family: &str, cell_id: &str, replicate: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(contract.as_bytes());
    hasher.update(b"|");
    hasher.update(family.as_bytes());
    hasher.update(b"|");
    hasher.update(cell_id.as_bytes());
    hasher.update(b"|");
    hasher.update(replicate.to_le_bytes());
    hasher.finalize().to_vec()
}

fn median_split(values: &mut [f64]) -> f64 {
    median_of(values)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E2 run failed: {error}");
        process::exit(1);
    }
}

struct CellResult {
    cell_id: String,
    effect: Option<f64>,
    n_high: u64,
    n_low: u64,
    excluded_history_incomplete: u64,
    excluded_end_of_window: u64,
    raw_p: Option<f64>,
    status: &'static str,
    detail: Value,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut named: Map<String, Value> = Map::new();
    while let Some(arg) = args.next() {
        if let Some(name) = arg.strip_prefix("--") {
            let value = args.next().ok_or(format!("missing value for {arg}"))?;
            named.insert(name.to_string(), Value::from(value));
        } else {
            return Err(format!("unexpected argument: {arg}").into());
        }
    }
    let get = |name: &str| -> Result<String, Box<dyn std::error::Error>> {
        named
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("--{name} is required").into())
    };
    let zones_path = get("zones")?;
    let anchors_path = get("anchors")?;
    let view_root = get("view-root")?;
    let registry_path = get("cell-registry")?;
    let contract_path = get("contract")?;
    let output_root = PathBuf::from(get("output-root")?);
    let run_label = get("run-id")?;

    let registry: Value = serde_json::from_reader(File::open(&registry_path)?)?;
    let contract: Value = serde_json::from_reader(File::open(&contract_path)?)?;
    let contract_identity = contract["contract_identity_sha256"]
        .as_str()
        .ok_or("contract identity missing")?
        .to_string();
    let expected_cells = registry["cells_total"]
        .as_u64()
        .ok_or("cells_total missing")?;

    let loader = Loader {
        zones_path: PathBuf::from(&zones_path),
        anchors_path: PathBuf::from(&anchors_path),
        view_root: PathBuf::from(&view_root),
    };
    let features = loader.load()?;
    let zone_features = derive_features(&features);

    std::fs::create_dir_all(&output_root)?;
    let mut results: Vec<CellResult> = Vec::new();

    // ---- MF_E2_CORE (9 pooled stratified cells) ----
    // bucket value per zone per stratum + outcome per zone
    fn pooled_riskdiff_cells(
        features: &Features,
        zone_features: &[Vec<ZoneFeatures>],
        contract_identity: &str,
        cell_prefix: &str,
        provenance: &str,
        outcome_of: impl Fn(&ZoneFeatures) -> Option<bool>,
        conditioner_of: impl Fn(&ZoneFeatures) -> Option<f64>,
        history_incomplete_of: impl Fn(&ZoneFeatures) -> bool,
        cells: &mut Vec<CellResult>,
    ) {
        // per stratum: build buckets by median split of the conditioner, then
        // pool risk differences with a stratified permutation null
        let mut per_stratum: Vec<(u64, u64, u64, u64)> = Vec::new(); // (n_low, n_high, s_low, s_high) resolved
        let mut total_excluded_history = 0_u64;
        let mut total_excluded_window = 0_u64;
        for stratum in zone_features.iter() {
            let mut eligible: Vec<(f64, bool)> = Vec::new();
            for zone in stratum {
                if history_incomplete_of(zone) {
                    total_excluded_history += 1;
                    continue;
                }
                let Some(conditioner) = conditioner_of(zone) else {
                    continue;
                };
                let Some(outcome) = outcome_of(zone) else {
                    total_excluded_window += 1;
                    continue;
                };
                eligible.push((conditioner, outcome));
            }
            if eligible.is_empty() {
                per_stratum.push((0, 0, 0, 0));
                continue;
            }
            let mut values: Vec<f64> = eligible.iter().map(|(c, _)| *c).collect();
            let threshold = median_split(&mut values);
            let mut n_low = 0_u64;
            let mut n_high = 0_u64;
            let mut s_low = 0_u64;
            let mut s_high = 0_u64;
            for (conditioner, outcome) in &eligible {
                if *conditioner <= threshold {
                    n_low += 1;
                    if *outcome {
                        s_low += 1;
                    }
                } else {
                    n_high += 1;
                    if *outcome {
                        s_high += 1;
                    }
                }
            }
            per_stratum.push((n_low, n_high, s_low, s_high));
        }
        // pooled effect = sum of per-stratum risk differences (equal weight)
        let mut observed = 0.0;
        let mut supported = true;
        for (n_low, n_high, s_low, s_high) in &per_stratum {
            if *n_low < 30 || *n_high < 30 {
                supported = false;
            }
            if *n_low > 0 && *n_high > 0 {
                observed += *s_high as f64 / *n_high as f64 - *s_low as f64 / *n_low as f64;
            }
        }
        if !supported {
            results_shift(
                cells,
                CellResult {
                    cell_id: cell_prefix.to_string(),
                    effect: None,
                    n_high: 0,
                    n_low: 0,
                    excluded_history_incomplete: total_excluded_history,
                    excluded_end_of_window: total_excluded_window,
                    raw_p: None,
                    status: "UNSUPPORTED",
                    detail: json!({"per_stratum": per_stratum}),
                },
            );
            return;
        }
        // stratified permutation null: permute labels within each stratum
        let mut seed_hasher = Sha256::new();
        seed_hasher.update(contract_identity.as_bytes());
        seed_hasher.update(cell_prefix.as_bytes());
        let seed = seed_hasher.finalize();
        let mut rng = Rng::from_seed(&seed);
        let mut extreme = 0_u64;
        for _ in 0..B {
            let mut null_effect = 0.0;
            for (n_low, n_high, s_low, s_high) in &per_stratum {
                if *n_low == 0 || *n_high == 0 {
                    continue;
                }
                // draw successes assigned to the high group under label permutation
                let total = n_low + n_high;
                let successes = s_low + s_high;
                let pmf = hypergeometric_pmf(total, successes, *n_high);
                if pmf.is_empty() {
                    continue;
                }
                let mut cumulative = 0.0;
                let u = rng.next_u64() as f64 / u64::MAX as f64;
                let mut k = pmf.last().unwrap().0;
                for (value, probability) in &pmf {
                    cumulative += probability;
                    if cumulative >= u {
                        k = *value;
                        break;
                    }
                }
                null_effect += k as f64 / *n_high as f64 - (successes - k) as f64 / *n_low as f64;
            }
            if (null_effect - observed).abs() >= observed.abs() - 1e-12 {
                extreme += 1;
            }
        }
        let p = (1 + extreme) as f64 / (B + 1) as f64;
        let _ = cell_prefix;
        results_shift(
            cells,
            CellResult {
                cell_id: cell_prefix.to_string(),
                effect: Some(observed),
                n_high: per_stratum.iter().map(|(n, _, _, _)| *n).sum(),
                n_low: per_stratum.iter().map(|(_, n, _, _)| *n).sum(),
                excluded_history_incomplete: total_excluded_history,
                excluded_end_of_window: total_excluded_window,
                raw_p: Some(p),
                status: "SUPPORTED",
                detail: json!({"per_stratum": per_stratum, "note": "effect = sum of per-stratum risk differences; stratified permutation null"}),
            },
        );
    }

    // helper to push while keeping the borrow checker happy
    fn results_shift(cells: &mut Vec<CellResult>, result: CellResult) {
        cells.push(result);
    }

    let outcomes_touch16 = |zone: &ZoneFeatures| zone.touch_within[1];
    let outcomes_fill16_touched = |zone: &ZoneFeatures| {
        zone.touch_ts
            .and_then(|touch| {
                zone.fill_ts
                    .map(|fill| fill - touch <= 16 * 0 + 0)
                    .map(|_| true)
            })
            .and(Some(true))
    };
    let _ = outcomes_fill16_touched;
    let _ = features;

    // E2-C01: gap_atr split, P(touch <= 16)
    {
        let zfs = &zone_features;
        let mut cells_tmp: Vec<CellResult> = Vec::new();
        pooled_riskdiff_cells(
            &features,
            zfs,
            &contract_identity,
            "E2::C01",
            "QG1-CORE-01/PD-F",
            |zone| zone.touch_within[1],
            |zone| Some(zone.gap_atr),
            |_| false,
            &mut cells_tmp,
        );
        results.append(&mut cells_tmp);
    }
    // E2-C03: stage-age median split, P(next transition <= 16)
    {
        let mut cells_tmp: Vec<CellResult> = Vec::new();
        pooled_riskdiff_cells(
            &features,
            &zone_features,
            &contract_identity,
            "E2::C03",
            "QG1-CORE-03",
            |zone| zone.touch_within[1],
            |zone| {
                zone.touch_ts
                    .map(|touch| (touch - zone.formation_ts) as f64)
            },
            |_| false,
            &mut cells_tmp,
        );
        results.append(&mut cells_tmp);
    }
    // E2-C04: crowding median split (warm-state washout applied)
    {
        let mut cells_tmp: Vec<CellResult> = Vec::new();
        pooled_riskdiff_cells(
            &features,
            &zone_features,
            &contract_identity,
            "E2::C04",
            "QG1-CORE-04",
            |zone| zone.touch_within[1],
            |zone| Some(zone.active_overlap_count as f64),
            |zone| {
                zone.formation_ts
                    < features.zones[0]
                        .first()
                        .map_or(0, |first| first.formation.detection_ts)
                        + WARMUP_BARS * 15_000
            },
            &mut cells_tmp,
        );
        results.append(&mut cells_tmp);
    }
    // E2-C05: rolling fill fraction (K prior completions required)
    {
        let mut cells_tmp: Vec<CellResult> = Vec::new();
        pooled_riskdiff_cells(
            &features,
            &zone_features,
            &contract_identity,
            "E2::C05",
            "QG1-CORE-05",
            |zone| zone.touch_within[1],
            |zone| zone.rolling_fill_fraction,
            |_| false,
            &mut cells_tmp,
        );
        results.append(&mut cells_tmp);
    }
    // E2-C09: session-preserving clustering null
    {
        let mut per_stratum_energy: Vec<(usize, f64)> = Vec::new();
        for (index, stratum) in features.zones.iter().enumerate() {
            let period = BAR_MS[index];
            let events: Vec<i64> = stratum
                .iter()
                .map(|zone| zone.formation.detection_ts)
                .collect();
            let first = events.first().copied().unwrap_or(0);
            let last = events.last().copied().unwrap_or(0);
            let n_bars = ((last - first) / period + 1).max(1) as usize;
            let mut counts = vec![0_u64; n_bars];
            for event in &events {
                counts[((event - first) / period) as usize] += 1;
            }
            let energy = |series: &[u64]| -> f64 {
                let n = series.len() as f64;
                let mean = series.iter().sum::<u64>() as f64 / n;
                let variance = series
                    .iter()
                    .map(|count| (*count as f64 - mean).powi(2))
                    .sum::<f64>()
                    / n;
                if variance <= 0.0 {
                    return 0.0;
                }
                (1..=16usize)
                    .map(|lag| {
                        let covariance: f64 = series
                            .iter()
                            .zip(series.iter().skip(lag))
                            .map(|(a, b)| (*a as f64 - mean) * (*b as f64 - mean))
                            .sum::<f64>()
                            / (n - lag as f64);
                        (covariance / variance).powi(2)
                    })
                    .sum()
            };
            let observed_energy = energy(&counts);
            per_stratum_energy.push((index, observed_energy));
        }
        let observed = per_stratum_energy
            .iter()
            .map(|(_, energy)| *energy)
            .sum::<f64>()
            / per_stratum_energy.len() as f64;
        // null: permute formation timestamps within (UTC day, 8h session) blocks
        let mut seed_hasher = Sha256::new();
        seed_hasher.update(contract_identity.as_bytes());
        seed_hasher.update(b"E2::C09");
        let seed = seed_hasher.finalize();
        let mut rng = Rng::from_seed(&seed);
        let mut extreme = 0_u64;
        const SESSION_NS: i64 = 8 * 3_600 * 1_000_000_000;
        const DAY_NS: i64 = 24 * 3_600 * 1_000_000_000;
        for _ in 0..B {
            let mut null_energy_sum = 0.0;
            for (index, stratum) in features.zones.iter().enumerate() {
                let period = BAR_MS[index];
                let mut events: Vec<i64> = stratum
                    .iter()
                    .map(|zone| zone.formation.detection_ts)
                    .collect();
                // group by (day, session) block and shuffle intra-block times
                let mut blocks: BTreeMap<(i64, i64), Vec<(usize, i64)>> = BTreeMap::new();
                for (position, event) in events.iter().enumerate() {
                    let day = event.div_euclid(DAY_NS);
                    let session = event.rem_euclid(DAY_NS) / SESSION_NS;
                    blocks
                        .entry((day, session))
                        .or_default()
                        .push((position, *event));
                }
                for (_, mut members) in blocks {
                    let n = members.len();
                    for draw in (1..n).rev() {
                        let pick = rng.below((draw + 1) as u64) as usize;
                        members.swap(draw, pick);
                    }
                    for (position, time) in members {
                        events[position] = time;
                    }
                }
                events.sort_unstable();
                let first = events.first().copied().unwrap_or(0);
                let last = events.last().copied().unwrap_or(0);
                let n_bars = ((last - first) / period + 1).max(1) as usize;
                let mut counts = vec![0_u64; n_bars];
                for event in &events {
                    counts[((event - first) / period) as usize] += 1;
                }
                let n = counts.len() as f64;
                let mean = counts.iter().sum::<u64>() as f64 / n;
                let variance = counts
                    .iter()
                    .map(|count| (*count as f64 - mean).powi(2))
                    .sum::<f64>()
                    / n;
                let energy = if variance <= 0.0 {
                    0.0
                } else {
                    (1..=16usize)
                        .map(|lag| {
                            let covariance: f64 = counts
                                .iter()
                                .zip(counts.iter().skip(lag))
                                .map(|(a, b)| (*a as f64 - mean) * (*b as f64 - mean))
                                .sum::<f64>()
                                / (n - lag as f64);
                            (covariance / variance).powi(2)
                        })
                        .sum()
                };
                null_energy_sum += energy;
            }
            let null_mean = null_energy_sum / per_stratum_energy.len() as f64;
            if (null_mean - observed).abs() >= observed.abs() - 1e-12 {
                extreme += 1;
            }
        }
        let p = (1 + extreme) as f64 / (B + 1) as f64;
        results.push(CellResult {
            cell_id: "E2::C09".into(),
            effect: Some(observed),
            n_high: 0,
            n_low: 0,
            excluded_history_incomplete: 0,
            excluded_end_of_window: 0,
            raw_p: Some(p),
            status: "SUPPORTED",
            detail: json!({"per_stratum_energy": per_stratum_energy}),
        });
    }
    // E2-C10: completion-mode contrast on post-fill h1 adjusted returns (per stratum, pooled)
    {
        let mut per_stratum: Vec<(Vec<f64>, Vec<f64>, u64)> = Vec::new();
        let mut total_excluded = 0_u64;
        for (index, stratum) in features.zones.iter().enumerate() {
            let mut touched_fill: Vec<f64> = Vec::new();
            let mut gap_through: Vec<f64> = Vec::new();
            for zone in stratum {
                let zone_id = zone.formation.zone_id;
                let Some(anchor) = features.fill.get(&(index, zone_id)) else {
                    continue;
                };
                let Some(outcome) = anchor.outcomes[0].as_ref() else {
                    total_excluded += 1;
                    continue;
                };
                if zone.first_touch_ts.is_some() {
                    touched_fill.push(outcome.direction_adjusted_return_bps);
                } else {
                    gap_through.push(outcome.direction_adjusted_return_bps);
                }
            }
            per_stratum.push((touched_fill, gap_through, 0));
        }
        let mut supported = true;
        for (touched, gap, _) in &per_stratum {
            if touched.len() < 30 || gap.len() < 30 {
                supported = false;
            }
        }
        if supported {
            let observed: f64 = per_stratum
                .iter()
                .filter(|(touched, gap, _)| !touched.is_empty() && !gap.is_empty())
                .map(|(touched, gap, _)| {
                    let mut t = touched.clone();
                    let mut g = gap.clone();
                    median_of(&mut t) - median_of(&mut g)
                })
                .sum();
            let mut seed_hasher = Sha256::new();
            seed_hasher.update(contract_identity.as_bytes());
            seed_hasher.update(b"E2::C10");
            let seed = seed_hasher.finalize();
            let mut rng = Rng::from_seed(&seed);
            let mut extreme = 0_u64;
            for _ in 0..B {
                let mut null_effect = 0.0;
                for (touched, gap, _) in &per_stratum {
                    if touched.is_empty() || gap.is_empty() {
                        continue;
                    }
                    // permute group labels within the pooled per-stratum sample
                    let mut pooled: Vec<f64> = touched.iter().chain(gap.iter()).copied().collect();
                    let n_high = gap.len();
                    let n = pooled.len();
                    for draw in (1..n).rev() {
                        let pick = rng.below((draw + 1) as u64) as usize;
                        pooled.swap(draw, pick);
                    }
                    let mut high: Vec<f64> = pooled[..n_high].to_vec();
                    let mut low: Vec<f64> = pooled[n_high..].to_vec();
                    null_effect += median_of(&mut high) - median_of(&mut low);
                }
                if (null_effect - observed).abs() >= observed.abs() - 1e-12 {
                    extreme += 1;
                }
            }
            let p = (1 + extreme) as f64 / (B + 1) as f64;
            results.push(CellResult {
                cell_id: "E2::C10".into(),
                effect: Some(observed),
                n_high: per_stratum.iter().map(|(_, gap, _)| gap.len() as u64).sum(),
                n_low: per_stratum.iter().map(|(touched, _, _)| touched.len() as u64).sum(),
                excluded_history_incomplete: 0,
                excluded_end_of_window: total_excluded,
                raw_p: Some(p),
                status: "SUPPORTED",
                detail: json!({"per_stratum_N": per_stratum.iter().map(|(t, g, _)| (t.len(), g.len())).collect::<Vec<_>>()}),
            });
        } else {
            results.push(CellResult {
                cell_id: "E2::C10".into(),
                effect: None,
                n_high: 0,
                n_low: 0,
                excluded_history_incomplete: 0,
                excluded_end_of_window: total_excluded,
                raw_p: None,
                status: "UNSUPPORTED",
                detail: json!({"reason": "bucket n < 30 in some stratum; reported per-stratum N only"}),
            });
        }
    }
    // E2-C12 a/b/c: tercile top-vs-bottom pooled contrasts
    for (cell_id, conditioner_kind) in [("E2::C12a", 0u8), ("E2::C12b", 1u8), ("E2::C12c", 2u8)] {
        let mut per_stratum: Vec<(u64, u64, u64, u64)> = Vec::new();
        for (index, _) in TIMEFRAMES.iter().enumerate() {
            let mut eligible: Vec<(f64, bool)> = Vec::new();
            for zone in &zone_features[index] {
                let conditioner = match conditioner_kind {
                    0 => Some(zone.gap_atr),
                    1 => zone
                        .touch_ts
                        .map(|touch| (touch - zone.formation_ts) as f64),
                    _ => Some(zone.active_overlap_count as f64),
                };
                let Some(conditioner) = conditioner else {
                    continue;
                };
                let Some(outcome) = zone.touch_within[1] else {
                    continue;
                };
                eligible.push((conditioner, outcome));
            }
            if eligible.len() < 90 {
                per_stratum.push((0, 0, 0, 0));
                continue;
            }
            let mut values: Vec<f64> = eligible.iter().map(|(c, _)| *c).collect();
            values.sort_by(f64::total_cmp);
            let n = values.len();
            let (t1, t2) = (values[n / 3], values[2 * n / 3]);
            let mut n_low = 0_u64;
            let mut n_high = 0_u64;
            let mut s_low = 0_u64;
            let mut s_high = 0_u64;
            for (conditioner, outcome) in &eligible {
                if *conditioner <= t1 {
                    n_low += 1;
                    if *outcome {
                        s_low += 1;
                    }
                } else if *conditioner > t2 {
                    n_high += 1;
                    if *outcome {
                        s_high += 1;
                    }
                }
            }
            per_stratum.push((n_low, n_high, s_low, s_high));
        }
        let mut observed = 0.0;
        let mut supported = true;
        for (n_low, n_high, s_low, s_high) in &per_stratum {
            if *n_low < 30 || *n_high < 30 {
                supported = false;
            }
            if *n_low > 0 && *n_high > 0 {
                observed += *s_high as f64 / *n_high as f64 - *s_low as f64 / *n_low as f64;
            }
        }
        if supported {
            let mut seed_hasher = Sha256::new();
            seed_hasher.update(contract_identity.as_bytes());
            seed_hasher.update(cell_id.as_bytes());
            let seed = seed_hasher.finalize();
            let mut rng = Rng::from_seed(&seed);
            let mut extreme = 0_u64;
            for _ in 0..B {
                let mut null_effect = 0.0;
                for (n_low, n_high, s_low, s_high) in &per_stratum {
                    if *n_low == 0 || *n_high == 0 {
                        continue;
                    }
                    let total = n_low + n_high;
                    let successes = s_low + s_high;
                    let pmf = hypergeometric_pmf(total, successes, *n_high);
                    if pmf.is_empty() {
                        continue;
                    }
                    let mut cumulative = 0.0;
                    let u = rng.next_u64() as f64 / u64::MAX as f64;
                    let mut k = pmf.last().unwrap().0;
                    for (value, probability) in &pmf {
                        cumulative += probability;
                        if cumulative >= u {
                            k = *value;
                            break;
                        }
                    }
                    null_effect +=
                        k as f64 / *n_high as f64 - (successes - k) as f64 / *n_low as f64;
                }
                if (null_effect - observed).abs() >= observed.abs() - 1e-12 {
                    extreme += 1;
                }
            }
            let p = (1 + extreme) as f64 / (B + 1) as f64;
            results.push(CellResult {
                cell_id: cell_id.to_string(),
                effect: Some(observed),
                n_high: per_stratum.iter().map(|(n, _, _, _)| *n).sum(),
                n_low: per_stratum.iter().map(|(_, n, _, _)| *n).sum(),
                excluded_history_incomplete: 0,
                excluded_end_of_window: 0,
                raw_p: Some(p),
                status: "SUPPORTED",
                detail: json!({"per_stratum": per_stratum}),
            });
        } else {
            results.push(CellResult {
                cell_id: cell_id.to_string(),
                effect: None,
                n_high: 0,
                n_low: 0,
                excluded_history_incomplete: 0,
                excluded_end_of_window: 0,
                raw_p: None,
                status: "UNSUPPORTED",
                detail: json!({}),
            });
        }
    }

    // ---- MF_STAGE_MATCHED (42) + MF_DIRECTION_ASYMMETRY (84) ----
    evaluate_anchor_cells(&features, &contract_identity, &mut results)?;

    // ---- MF_SF1 / MF_SF2 / MF_SF5 / robustness (systematic families) ----
    evaluate_systematic_cells(&features, &zone_features, &contract_identity, &mut results)?;

    // ---- reconcile with the frozen registry ----
    if results.len() as u64 != expected_cells {
        return Err(format!(
            "evaluated {} cells but the frozen registry expects {expected_cells}",
            results.len()
        )
        .into());
    }
    results.sort_by(|a, b| a.cell_id.cmp(&b.cell_id));
    let mut seen = HashSet::new();
    for result in &results {
        if !seen.insert(result.cell_id.clone()) {
            return Err(format!("duplicate evaluated cell {}", result.cell_id).into());
        }
    }

    // ---- multiplicity ----
    let adjusted = apply_multiplicity(&results);
    let mut nominal_discoveries = 0_u64;
    let mut surviving: BTreeMap<String, u64> = BTreeMap::new();
    for result in &results {
        if result.status == "SUPPORTED" {
            if let Some(raw) = result.raw_p {
                if raw < 0.05 {
                    nominal_discoveries += 1;
                }
            }
        }
    }
    for (family, count) in &adjusted.surviving_by_family {
        surviving.insert(family.clone(), *count);
    }

    // ---- outputs ----
    let logical_hash = {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(&adjusted.results_json)?);
        hasher.update(serde_json::to_vec(&json!({
            "contract_identity": contract_identity,
            "registry_identity": registry["contract_identity_sha256"],
            "cells_total": expected_cells,
        }))?);
        format!("{:x}", hasher.finalize())
    };
    let results_json = json!({
        "artifact_type": "E2_RESULTS",
        "run_id": run_label,
        "contract": "AP-002_E2_CONTRACT_V1_1.json",
        "contract_identity_sha256": contract_identity,
        "cell_registry": "E2_CELL_REGISTRY.json",
        "cells_total": results.len(),
        "logical_result_hash": logical_hash,
        "nominal_discoveries_raw_p_lt_0_05": nominal_discoveries,
        "surviving_multiplicity_by_family": surviving,
        "results": adjusted.results_json,
    });
    std::fs::write(
        output_root.join("E2_RESULTS.json"),
        serde_json::to_vec_pretty(&results_json)?,
    )?;
    let feature_tape_hash = {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(&json!({
            "zones": features.zones.iter().map(|s| s.len()).collect::<Vec<_>>(),
            "confirmation_ranges": features.confirmation_range.len(),
            "ft_anchors": features.ft.len(),
            "fill_anchors": features.fill.len(),
            "zone_feature_rows": zone_features.iter().map(|s| s.len()).collect::<Vec<_>>(),
        }))?);
        format!("{:x}", hasher.finalize())
    };
    let manifest = json!({
        "artifact_type": "E2_RUN_MANIFEST",
        "run_id": run_label,
        "logical_result_hash": logical_hash,
        "feature_tape_logical_hash": feature_tape_hash,
        "contract_identity_sha256": contract_identity,
        "cells_total": results.len(),
        "inputs": {"zones": zones_path, "anchors": anchors_path, "view_root": view_root},
        "confirmation_status": "LOCKED",
    });
    std::fs::write(
        output_root.join("E2_RUN_MANIFEST.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "E2 run {run_label}: cells {} | logical hash {logical_hash} | nominal {nominal_discoveries} | surviving {:?}",
        results.len(),
        surviving,
    );
    Ok(())
}

/// Panel (42) and direction-asymmetry (84) anchor-anchored cells.
fn evaluate_anchor_cells(
    features: &Features,
    contract_identity: &str,
    results: &mut Vec<CellResult>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Panel: paired contrasts. Population join by (stratum, zone_id).
    for (contrast_id, use_formation) in [("C1", true), ("C2", false)] {
        for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
            for (horizon_index, h) in HORIZONS.iter().enumerate() {
                let mut pairs: Vec<f64> = Vec::new();
                let mut excluded_window = 0_u64;
                for zone in &features.zones[index] {
                    let key = (index, zone.formation.zone_id);
                    let (first_value, second_value): (Option<f64>, Option<f64>) = if use_formation {
                        let formation_outcome = zone.prospective_outcomes[horizon_index].as_ref();
                        let touch_outcome = features
                            .ft
                            .get(&key)
                            .and_then(|anchor| anchor.outcomes[horizon_index].as_ref());
                        (
                            formation_outcome.map(|outcome| outcome.direction_adjusted_return_bps),
                            touch_outcome.map(|outcome| outcome.direction_adjusted_return_bps),
                        )
                    } else {
                        let Some(touch_anchor) = features.ft.get(&key) else {
                            continue;
                        };
                        let Some(fill_anchor) = features.fill.get(&key) else {
                            continue;
                        };
                        if fill_anchor.first_touch_observed == Some(false) {
                            continue; // gap-through fills have no FIRST_TOUCH anchor
                        }
                        (
                            touch_anchor.outcomes[horizon_index]
                                .as_ref()
                                .map(|outcome| outcome.direction_adjusted_return_bps),
                            fill_anchor.outcomes[horizon_index]
                                .as_ref()
                                .map(|outcome| outcome.direction_adjusted_return_bps),
                        )
                    };
                    let (Some(first), Some(second)) = (first_value, second_value) else {
                        excluded_window += 1;
                        continue;
                    };
                    pairs.push(second - first);
                }
                let _ = use_formation;
                let cell_id = format!("E2::PANEL::{contrast_id}::{timeframe}::h{h}");
                if pairs.len() < 60 {
                    results.push(CellResult {
                        cell_id,
                        effect: None,
                        n_high: pairs.len() as u64,
                        n_low: 0,
                        excluded_history_incomplete: 0,
                        excluded_end_of_window: excluded_window,
                        raw_p: None,
                        status: "UNSUPPORTED",
                        detail: json!({"reason": "paired n < 60"}),
                    });
                    continue;
                }
                let mut seed_hasher = Sha256::new();
                seed_hasher.update(contract_identity.as_bytes());
                seed_hasher.update(cell_id.as_bytes());
                let seed = seed_hasher.finalize();
                let (effect, p) = signflip_p(&pairs, &seed);
                results.push(CellResult {
                    cell_id,
                    effect: Some(effect),
                    n_high: pairs.len() as u64,
                    n_low: 0,
                    excluded_history_incomplete: 0,
                    excluded_end_of_window: excluded_window,
                    raw_p: Some(p),
                    status: "SUPPORTED",
                    detail: json!({"paired_n": pairs.len()}),
                });
            }
        }
    }
    // Direction-asymmetry: median adjusted return vs zero after drift-aware centering
    for anchor_stage in ["FIRST_TOUCH", "FILL"] {
        for direction in ["bullish", "bearish"] {
            for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
                for (horizon_index, h) in HORIZONS.iter().enumerate() {
                    // collect raw + adjusted per anchor row
                    let source = if anchor_stage == "FIRST_TOUCH" {
                        &features.ft
                    } else {
                        &features.fill
                    };
                    // group by (stratum, UTC day)
                    let mut per_day: BTreeMap<i64, Vec<(f64, f64)>> = BTreeMap::new();
                    for ((stratum, _), anchor) in source.iter() {
                        if *stratum != index || anchor.direction != direction {
                            continue;
                        }
                        let Some(outcome) = anchor.outcomes[horizon_index].as_ref() else {
                            continue;
                        };
                        let day = anchor.anchor_bar_close_ts * 1_000_000 / (86_400 * 1_000_000_000);
                        per_day.entry(day).or_default().push((
                            outcome.raw_return_bps,
                            outcome.direction_adjusted_return_bps,
                        ));
                    }
                    let mut values: Vec<f64> = Vec::new();
                    for (_, mut rows) in per_day {
                        let mut raws: Vec<f64> = rows.iter().map(|(raw, _)| *raw).collect();
                        let block_median = median_of(&mut raws);
                        for (_, adjusted) in rows.drain(..) {
                            values.push(adjusted - block_median);
                        }
                    }
                    let _ = timeframe;
                    let cell_id =
                        format!("E2::DIR::{direction}::{anchor_stage}::{timeframe}::h{h}");
                    if values.len() < 30 {
                        results.push(CellResult {
                            cell_id,
                            effect: None,
                            n_high: values.len() as u64,
                            n_low: 0,
                            excluded_history_incomplete: 0,
                            excluded_end_of_window: 0,
                            raw_p: None,
                            status: "UNSUPPORTED",
                            detail: json!({"reason": "n < 30"}),
                        });
                        continue;
                    }
                    let mut seed_hasher = Sha256::new();
                    seed_hasher.update(contract_identity.as_bytes());
                    seed_hasher.update(cell_id.as_bytes());
                    let seed = seed_hasher.finalize();
                    let (effect, p) = signflip_p(&values, &seed);
                    results.push(CellResult {
                        cell_id,
                        effect: Some(effect),
                        n_high: values.len() as u64,
                        n_low: 0,
                        excluded_history_incomplete: 0,
                        excluded_end_of_window: 0,
                        raw_p: Some(p),
                        status: "SUPPORTED",
                        detail: json!({"n": values.len(), "centering": "per-(stratum, UTC-day) raw median"}),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Systematic families SF-1/SF-2/SF-5 and the robustness lane.
fn evaluate_systematic_cells(
    features: &Features,
    zone_features: &[Vec<ZoneFeatures>],
    contract_identity: &str,
    results: &mut Vec<CellResult>,
) -> Result<(), Box<dyn std::error::Error>> {
    // The SF-1/SF-2/SF-5 sweep above is superseded by the unified evaluation below.
    sf_sweep(features, zone_features, contract_identity, results)
}

/// Unified systematic-family sweep (SF-1 98, SF-2 126, SF-5 63, robustness 50).
fn sf_sweep(
    features: &Features,
    zone_features: &[Vec<ZoneFeatures>],
    contract_identity: &str,
    results: &mut Vec<CellResult>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut excluded_report: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    fn run_riskdiff_cell(
        results: &mut Vec<CellResult>,
        contract_identity: &str,
        family: &'static str,
        procedure: &'static str,
        cell_id: String,
        stratum_label: &str,
        groups: (Vec<bool>, Vec<bool>),
        excluded: (u64, u64),
        cell_notes: Value,
    ) {
        let (low, high) = groups;
        let n_low = low.len() as u64;
        let n_high = high.len() as u64;
        let s_low = low.iter().filter(|outcome| **outcome).count() as u64;
        let s_high = high.iter().filter(|outcome| **outcome).count() as u64;
        if n_low < 30 || n_high < 30 {
            results.push(CellResult {
                cell_id,
                effect: None,
                n_high,
                n_low,
                excluded_history_incomplete: excluded.0,
                excluded_end_of_window: excluded.1,
                raw_p: None,
                status: "UNSUPPORTED",
                detail: json!({"n_low": n_low, "n_high": n_high, "notes": cell_notes}),
            });
            return;
        }
        let observed = s_high as f64 / n_high as f64 - s_low as f64 / n_low as f64;
        let mut seed_hasher = Sha256::new();
        seed_hasher.update(contract_identity.as_bytes());
        seed_hasher.update(cell_id.as_bytes());
        let seed = seed_hasher.finalize();
        let mut rng = Rng::from_seed(&seed);
        let mut extreme = 0_u64;
        for _ in 0..B {
            let total = n_low + n_high;
            let successes = s_low + s_high;
            let pmf = hypergeometric_pmf(total, successes, n_high);
            if pmf.is_empty() {
                continue;
            }
            let mut cumulative = 0.0;
            let u = rng.next_u64() as f64 / u64::MAX as f64;
            let mut k = pmf.last().unwrap().0;
            for (value, probability) in &pmf {
                cumulative += probability;
                if cumulative >= u {
                    k = *value;
                    break;
                }
            }
            let effect = k as f64 / n_high as f64 - (successes - k) as f64 / n_low.max(1) as f64;
            if (effect - observed).abs() >= observed.abs() - 1e-12 {
                extreme += 1;
            }
        }
        let p = (1 + extreme) as f64 / (B + 1) as f64;
        results.push(CellResult {
            cell_id,
            effect: Some(observed),
            n_high,
            n_low,
            excluded_history_incomplete: excluded.0,
            excluded_end_of_window: excluded.1,
            raw_p: Some(p),
            status: "SUPPORTED",
            detail: json!({"n_low": n_low, "n_high": n_high, "s_low": s_low, "s_high": s_high, "notes": cell_notes}),
        });
    }

    // conditioner/outcome extraction per zone index
    let stage_outcome = |zone: &ZoneFeatures, stage: &str, index: usize| -> Option<bool> {
        if stage == "FORMED_UNTOUCHED" {
            zone.touch_within[1]
        } else {
            zone.touch_ts
                .and_then(|touch| zone.fill_ts.map(|fill| fill - touch <= 16 * BAR_MS[index]))
        }
    };
    // SF-1: 2 stages x 7 conditioners x 7 strata
    for (stage, _) in [("FORMED_UNTOUCHED", 0u8), ("TOUCHED_UNFILLED", 1u8)] {
        for (short, conditioner_kind) in [
            ("zone_age", 0u8),
            ("zone_height", 1u8),
            ("crowding", 2u8),
            ("history", 3u8),
            ("direction", 4u8),
            ("PAIR_age_height", 5u8),
            ("PAIR_crowding_history", 6u8),
        ] {
            for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
                let mut low: Vec<bool> = Vec::new();
                let mut high: Vec<bool> = Vec::new();
                let mut excluded_history = 0_u64;
                let mut excluded_window = 0_u64;
                let mut conditioner_values: Vec<(f64, bool)> = Vec::new();
                let mut directional: Vec<(bool, bool)> = Vec::new();
                let mut pairs: Vec<((f64, f64), bool)> = Vec::new();
                for zone in &zone_features[index] {
                    if stage == "TOUCHED_UNFILLED" && zone.touch_ts.is_none() {
                        continue;
                    }
                    let Some(outcome) = stage_outcome(zone, stage, index) else {
                        excluded_window += 1;
                        continue;
                    };
                    match conditioner_kind {
                        0 => {
                            // stage age: formation age at stage entry
                            let age = if stage == "FORMED_UNTOUCHED" {
                                0.0
                            } else {
                                zone.touch_ts
                                    .map_or(0.0, |touch| (touch - zone.formation_ts) as f64)
                            };
                            conditioner_values.push((age, outcome));
                        }
                        1 => conditioner_values.push((zone.gap_atr, outcome)),
                        2 => {
                            if zone.formation_ts
                                < features.zones[index]
                                    .first()
                                    .map_or(0, |first| first.formation.detection_ts)
                                    + WARMUP_BARS * BAR_MS[index]
                            {
                                excluded_history += 1;
                            } else {
                                conditioner_values
                                    .push((zone.active_overlap_count as f64, outcome));
                            }
                        }
                        3 => match zone.rolling_fill_fraction {
                            Some(value) => conditioner_values.push((value, outcome)),
                            None => excluded_history += 1,
                        },
                        4 => directional.push((zone.direction_bullish, outcome)),
                        5 => {
                            if zone.formation_ts
                                < features.zones[index]
                                    .first()
                                    .map_or(0, |first| first.formation.detection_ts)
                                    + WARMUP_BARS * BAR_MS[index]
                            {
                                excluded_history += 1;
                            } else {
                                pairs.push((
                                    (zone.active_overlap_count as f64, zone.gap_atr),
                                    outcome,
                                ));
                            }
                        }
                        _ => {
                            if zone.rolling_fill_fraction.is_none()
                                || zone.formation_ts
                                    < features.zones[index]
                                        .first()
                                        .map_or(0, |first| first.formation.detection_ts)
                                        + WARMUP_BARS * BAR_MS[index]
                            {
                                excluded_history += 1;
                            } else {
                                pairs.push((
                                    (
                                        zone.active_overlap_count as f64,
                                        zone.rolling_fill_fraction.unwrap(),
                                    ),
                                    outcome,
                                ));
                            }
                        }
                    }
                }
                excluded_report.insert(
                    format!("SF1::{stage}::{short}::{timeframe}"),
                    (excluded_history, excluded_window),
                );
                match conditioner_kind {
                    4 => {
                        let bullish: Vec<bool> = directional
                            .iter()
                            .filter(|(bullish, _)| *bullish)
                            .map(|(_, outcome)| *outcome)
                            .collect();
                        let bearish: Vec<bool> = directional
                            .iter()
                            .filter(|(bullish, _)| !*bullish)
                            .map(|(_, outcome)| *outcome)
                            .collect();
                        run_riskdiff_cell(
                            results,
                            contract_identity,
                            "MF_SF1",
                            "BH-FDR q=0.05",
                            format!("E2::SF1::{stage}::{short}::{timeframe}"),
                            timeframe,
                            (bearish, bullish),
                            (excluded_history, excluded_window),
                            json!({"conditioner": "direction_as_moderator"}),
                        );
                    }
                    5 | 6 => {
                        // diagonal quadrants
                        let mut values: Vec<f64> = pairs
                            .iter()
                            .map(|((a, b), _)| if conditioner_kind == 5 { *b } else { *a })
                            .collect();
                        let threshold = median_split(&mut values);
                        let mut low_low: Vec<bool> = Vec::new();
                        let mut high_high: Vec<bool> = Vec::new();
                        for ((first, second), outcome) in pairs.iter() {
                            let (primary, secondary) = if conditioner_kind == 5 {
                                (*second, *first)
                            } else {
                                (*first, *second)
                            };
                            if primary <= threshold && secondary <= threshold {
                                low_low.push(*outcome);
                            } else if primary > threshold && secondary > threshold {
                                high_high.push(*outcome);
                            }
                        }
                        run_riskdiff_cell(
                            results,
                            contract_identity,
                            "MF_SF1",
                            "BH-FDR q=0.05",
                            format!("E2::SF1::{stage}::{short}::{timeframe}"),
                            timeframe,
                            (low_low, high_high),
                            (excluded_history, excluded_window),
                            json!({"conditioner": "diagonal quadrants"}),
                        );
                    }
                    _ => {
                        let mut values: Vec<f64> =
                            conditioner_values.iter().map(|(value, _)| *value).collect();
                        let threshold = median_split(&mut values);
                        low.clear();
                        high.clear();
                        for (value, outcome) in &conditioner_values {
                            if *value <= threshold {
                                low.push(*outcome);
                            } else {
                                high.push(*outcome);
                            }
                        }
                        run_riskdiff_cell(
                            results,
                            contract_identity,
                            "MF_SF1",
                            "BH-FDR q=0.05",
                            format!("E2::SF1::{stage}::{short}::{timeframe}"),
                            timeframe,
                            (low.clone(), high.clone()),
                            (excluded_history, excluded_window),
                            json!({"conditioner": short}),
                        );
                    }
                }
            }
        }
    }

    // SF-2: 3 attributes x 3 outcomes x 2 directions x 7 strata
    fn attribute_gap_atr(zone: &ZoneFeatures) -> Option<f64> {
        Some(zone.gap_atr)
    }
    fn attribute_impulse(zone: &ZoneFeatures) -> Option<f64> {
        Some(zone.impulse_body_atr)
    }
    fn attribute_confirmation(zone: &ZoneFeatures) -> Option<f64> {
        zone.confirmation_range_atr
    }
    for (attr_short, attribute_of) in [
        (
            "gap_atr",
            attribute_gap_atr as fn(&ZoneFeatures) -> Option<f64>,
        ),
        (
            "impulse_body",
            attribute_impulse as fn(&ZoneFeatures) -> Option<f64>,
        ),
        (
            "confirmation_range",
            attribute_confirmation as fn(&ZoneFeatures) -> Option<f64>,
        ),
    ] {
        for (out_short, outcome_kind) in [("touch4", 0u8), ("touch16", 1u8), ("fill16", 2u8)] {
            for direction in ["bullish", "bearish"] {
                for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
                    let mut low: Vec<bool> = Vec::new();
                    let mut high: Vec<bool> = Vec::new();
                    let mut excluded_window = 0_u64;
                    let mut conditioner_values: Vec<(f64, bool)> = Vec::new();
                    for zone in &zone_features[index] {
                        if (zone.direction_bullish) != (direction == "bullish") {
                            continue;
                        }
                        if out_short == "fill16" && zone.touch_ts.is_none() {
                            continue;
                        }
                        let outcome = match outcome_kind {
                            0 => zone.touch_within[0],
                            1 => zone.touch_within[1],
                            _ => zone.touch_ts.and_then(|touch| {
                                zone.fill_ts.map(|fill| fill - touch <= 16 * BAR_MS[index])
                            }),
                        };
                        let Some(outcome) = outcome else {
                            excluded_window += 1;
                            continue;
                        };
                        if let Some(value) = attribute_of(zone) {
                            conditioner_values.push((value, outcome));
                        }
                    }
                    let mut values: Vec<f64> =
                        conditioner_values.iter().map(|(value, _)| *value).collect();
                    let threshold = median_split(&mut values);
                    low.clear();
                    high.clear();
                    for (value, outcome) in &conditioner_values {
                        if *value <= threshold {
                            low.push(*outcome);
                        } else {
                            high.push(*outcome);
                        }
                    }
                    run_riskdiff_cell(
                        results,
                        contract_identity,
                        "MF_SF2",
                        "BH-FDR q=0.05",
                        format!("E2::SF2::{timeframe}::{direction}::{attr_short}::{out_short}"),
                        timeframe,
                        (low, high),
                        (0, excluded_window),
                        json!({"attribute": attr_short, "outcome": out_short}),
                    );
                }
            }
        }
    }

    // SF-5: 3 measures x 3 outcomes x 7 strata
    for (short, measure_kind) in [
        ("crowding", 0u8),
        ("containment", 1u8),
        ("edge_distance", 2u8),
    ] {
        for (out_short, outcome_kind) in [("touch16", 0u8), ("fill16", 1u8), ("surv64", 2u8)] {
            for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
                let mut low: Vec<bool> = Vec::new();
                let mut high: Vec<bool> = Vec::new();
                let mut excluded_history = 0_u64;
                let mut excluded_window = 0_u64;
                let mut conditioner_values: Vec<(f64, bool)> = Vec::new();
                let mut binary: Vec<(bool, bool)> = Vec::new();
                for zone in &zone_features[index] {
                    if zone.formation_ts
                        < features.zones[index]
                            .first()
                            .map_or(0, |first| first.formation.detection_ts)
                            + WARMUP_BARS * BAR_MS[index]
                    {
                        excluded_history += 1;
                        continue;
                    }
                    let outcome = match outcome_kind {
                        0 => zone.touch_within[1],
                        1 => zone.touch_ts.and_then(|touch| {
                            zone.fill_ts.map(|fill| fill - touch <= 16 * BAR_MS[index])
                        }),
                        _ => Some(
                            zone.fill_ts
                                .map_or(true, |fill| fill - zone.formation_ts > 64 * BAR_MS[index]),
                        ),
                    };
                    let Some(outcome) = outcome else {
                        excluded_window += 1;
                        continue;
                    };
                    match measure_kind {
                        0 => conditioner_values.push((zone.active_overlap_count as f64, outcome)),
                        1 => binary.push((zone.containment, outcome)),
                        _ => {
                            if zone.nearest_edge_atr.is_nan() {
                                continue;
                            }
                            conditioner_values.push((zone.nearest_edge_atr, outcome));
                        }
                    }
                }
                match measure_kind {
                    1 => {
                        let contained: Vec<bool> = binary
                            .iter()
                            .filter(|(contained, _)| *contained)
                            .map(|(_, outcome)| *outcome)
                            .collect();
                        let not_contained: Vec<bool> = binary
                            .iter()
                            .filter(|(contained, _)| !*contained)
                            .map(|(_, outcome)| *outcome)
                            .collect();
                        run_riskdiff_cell(
                            results,
                            contract_identity,
                            "MF_SF5",
                            "BH-FDR q=0.05",
                            format!("E2::SF5::{timeframe}::{short}::{out_short}"),
                            timeframe,
                            (not_contained, contained),
                            (excluded_history, excluded_window),
                            json!({"measure": short, "outcome": out_short}),
                        );
                    }
                    _ => {
                        let mut values: Vec<f64> =
                            conditioner_values.iter().map(|(value, _)| *value).collect();
                        let threshold = median_split(&mut values);
                        low.clear();
                        high.clear();
                        for (value, outcome) in &conditioner_values {
                            if *value <= threshold {
                                low.push(*outcome);
                            } else {
                                high.push(*outcome);
                            }
                        }
                        run_riskdiff_cell(
                            results,
                            contract_identity,
                            "MF_SF5",
                            "BH-FDR q=0.05",
                            format!("E2::SF5::{timeframe}::{short}::{out_short}"),
                            timeframe,
                            (low, high),
                            (excluded_history, excluded_window),
                            json!({"measure": short, "outcome": out_short}),
                        );
                    }
                }
            }
        }
    }

    // Robustness lane (50): OPT-02 21 tercile agreements, OPT-04 28 estimator cells, CORE-13 1
    for (attr_short, attribute_of) in [
        (
            "gap_atr",
            attribute_gap_atr as fn(&ZoneFeatures) -> Option<f64>,
        ),
        (
            "impulse_body",
            attribute_impulse as fn(&ZoneFeatures) -> Option<f64>,
        ),
        (
            "confirmation_range",
            attribute_confirmation as fn(&ZoneFeatures) -> Option<f64>,
        ),
    ] {
        for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
            let mut conditioner_values: Vec<(f64, bool)> = Vec::new();
            for zone in &zone_features[index] {
                let Some(outcome) = zone.touch_within[1] else {
                    continue;
                };
                if let Some(value) = attribute_of(zone) {
                    conditioner_values.push((value, outcome));
                }
            }
            let mut values: Vec<f64> = conditioner_values.iter().map(|(value, _)| *value).collect();
            values.sort_by(f64::total_cmp);
            let n = values.len();
            let (t1, t2) = (values[n / 3], values[2 * n / 3]);
            let mut low: Vec<bool> = Vec::new();
            let mut high: Vec<bool> = Vec::new();
            for (value, outcome) in &conditioner_values {
                if *value <= t1 {
                    low.push(*outcome);
                } else if *value > t2 {
                    high.push(*outcome);
                }
            }
            let n_low = low.len() as u64;
            let n_high = high.len() as u64;
            let s_low = low.iter().filter(|outcome| **outcome).count() as u64;
            let s_high = high.iter().filter(|outcome| **outcome).count() as u64;
            let effect = if n_low > 0 && n_high > 0 {
                Some(s_high as f64 / n_high as f64 - s_low as f64 / n_low as f64)
            } else {
                None
            };
            results.push(CellResult {
                cell_id: format!("E2::OPT02::{timeframe}::{attr_short}"),
                effect,
                n_high,
                n_low,
                excluded_history_incomplete: 0,
                excluded_end_of_window: 0,
                raw_p: None,
                status: "SUPPORTED",
                detail: json!({"lane": "robustness", "agreement_with": "SF2 median-split contrast"}),
            });
        }
    }
    let _estimator_names = ["naive", "full_cohort", "lower_bound", "upper_bound"];
    for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
        // touched cohort with 16-bar resolution; deterministic transition accounting
        let window_end = features.zones[index]
            .last()
            .map_or(0, |last| last.formation.detection_ts + BAR_MS[index]);
        let mut filled_within = 0_u64;
        let mut not_filled_known = 0_u64;
        let mut unknown = 0_u64;
        let mut all_touched = 0_u64;
        for zone in &zone_features[index] {
            let Some(touch) = zone.touch_ts else { continue };
            all_touched += 1;
            let horizon_end = touch + 16 * BAR_MS[index];
            if let Some(fill) = zone.fill_ts {
                if fill <= horizon_end {
                    filled_within += 1;
                } else {
                    not_filled_known += 1;
                }
            } else if horizon_end <= window_end {
                not_filled_known += 1;
            } else {
                unknown += 1;
            }
        }
        let known = filled_within + not_filled_known;
        let estimates: [(&str, f64); 4] = [
            ("naive", filled_within as f64 / known.max(1) as f64),
            (
                "full_cohort",
                filled_within as f64 / all_touched.max(1) as f64,
            ),
            (
                "lower_bound",
                filled_within as f64 / (known + unknown).max(1) as f64,
            ),
            (
                "upper_bound",
                (filled_within + unknown) as f64 / all_touched.max(1) as f64,
            ),
        ];
        for (short, effect) in estimates {
            results.push(CellResult {
                cell_id: format!("E2::OPT04::{timeframe}::{short}"),
                effect: Some(effect),
                n_high: filled_within,
                n_low: all_touched - filled_within,
                excluded_history_incomplete: 0,
                excluded_end_of_window: unknown,
                raw_p: None,
                status: "SUPPORTED",
                detail: json!({"lane": "robustness", "estimator": short, "all_touched": all_touched, "known": known, "unknown": unknown}),
            });
        }
    }
    {
        // compute from the just-pushed OPT04 cells
        let mut per_timeframe: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for result in results
            .iter()
            .filter(|result| result.cell_id.starts_with("E2::OPT04::"))
        {
            let timeframe = result.cell_id.split("::").nth(2).unwrap_or("").to_string();
            if let Some(effect) = result.effect {
                per_timeframe.entry(timeframe).or_default().push(effect);
            }
        }
        let max_pairwise = per_timeframe
            .values()
            .map(|values| {
                let mut max = 0.0_f64;
                for a in values {
                    for b in values {
                        max = max.max((a - b).abs());
                    }
                }
                max
            })
            .sum::<f64>()
            / per_timeframe.len().max(1) as f64;
        results.push(CellResult {
            cell_id: "E2::CORE13".into(),
            effect: Some(max_pairwise),
            n_high: 0,
            n_low: 0,
            excluded_history_incomplete: 0,
            excluded_end_of_window: 0,
            raw_p: None,
            status: "SUPPORTED",
            detail: json!({"lane": "robustness", "definition": "mean over strata of max pairwise estimator divergence"}),
        });
    }
    Ok(())
}

/// Holm (per family) for the three primary families; BH-FDR for the systematic
/// families; robustness lane untouched.
fn apply_multiplicity(results: &[CellResult]) -> MultiplicityOutput {
    let mut results_json: Vec<Value> = Vec::new();
    let mut surviving_by_family: BTreeMap<String, u64> = BTreeMap::new();
    for family in [
        ("MF_E2_CORE", "holm"),
        ("MF_STAGE_MATCHED", "holm"),
        ("MF_DIRECTION_ASYMMETRY", "holm"),
        ("MF_SF1", "bh"),
        ("MF_SF2", "bh"),
        ("MF_SF5", "bh"),
    ] {
        let indices: Vec<usize> = results
            .iter()
            .enumerate()
            .filter(|(_, result)| {
                cell_family(&result.cell_id) == family.0 && result.status == "SUPPORTED"
            })
            .map(|(index, _)| index)
            .collect();
        let mut p_values: Vec<(usize, f64)> = indices
            .iter()
            .map(|&index| (index, results[index].raw_p.unwrap_or(1.0)))
            .collect();
        if family.1 == "holm" {
            p_values.sort_by(|a, b| a.1.total_cmp(&b.1));
            let m = p_values.len() as f64;
            let mut running_max = 0.0_f64;
            for (rank, (index, p)) in p_values.iter().enumerate() {
                let adjusted = ((m - rank as f64) * p).min(1.0);
                running_max = running_max.max(adjusted);
                let adjusted = running_max;
                let result = &results[*index];
                results_json.push(cell_json(result, family.0, Some(adjusted), "Holm"));
                if adjusted < 0.05 {
                    *surviving_by_family.entry(family.0.to_string()).or_default() += 1;
                }
            }
            for (index, _) in p_values.iter() {
                if !results_json
                    .iter()
                    .any(|entry| entry["cell_id"] == json!(results[*index].cell_id))
                {
                    results_json.push(cell_json(&results[*index], family.0, None, "Holm"));
                }
            }
        } else {
            let mut ordered: Vec<(usize, f64)> = p_values.clone();
            ordered.sort_by(|a, b| a.1.total_cmp(&b.1));
            let m = ordered.len() as f64;
            let mut adjusted_values: Vec<(usize, f64)> = Vec::new();
            let mut running_max = 0.0_f64;
            for (rank, (index, p)) in ordered.iter().enumerate() {
                let adjusted = (p * m / (rank as f64 + 1.0)).min(1.0);
                running_max = running_max.max(adjusted);
                adjusted_values.push((*index, running_max));
            }
            for (index, adjusted) in &adjusted_values {
                let result = &results[*index];
                results_json.push(cell_json(result, family.0, Some(*adjusted), "BH-FDR"));
                if *adjusted < 0.05 {
                    *surviving_by_family.entry(family.0.to_string()).or_default() += 1;
                }
            }
            for (index, p) in &p_values {
                if !adjusted_values
                    .iter()
                    .any(|(adjusted_index, _)| adjusted_index == index)
                {
                    let _ = p;
                    results_json.push(cell_json(&results[*index], family.0, None, "BH-FDR"));
                }
            }
        }
    }
    // unsupported/robustness/other cells pass through without adjusted p
    for result in results {
        let family = cell_family(&result.cell_id);
        let is_primary_systematic = matches!(
            family.as_str(),
            "MF_E2_CORE"
                | "MF_STAGE_MATCHED"
                | "MF_DIRECTION_ASYMMETRY"
                | "MF_SF1"
                | "MF_SF2"
                | "MF_SF5"
        ) && result.status == "SUPPORTED";
        if is_primary_systematic {
            continue;
        }
        results_json.push(cell_json(result, &family, None, "none"));
    }
    MultiplicityOutput {
        results_json,
        surviving_by_family,
    }
}

struct MultiplicityOutput {
    results_json: Vec<Value>,
    surviving_by_family: BTreeMap<String, u64>,
}

fn cell_family(cell_id: &str) -> String {
    if cell_id.starts_with("E2::C0")
        || cell_id.starts_with("E2::C1")
        || cell_id.starts_with("E2::PANEL")
    {
        if cell_id.starts_with("E2::PANEL") {
            "MF_STAGE_MATCHED".into()
        } else {
            "MF_E2_CORE".into()
        }
    } else if cell_id.starts_with("E2::DIR::") {
        "MF_DIRECTION_ASYMMETRY".into()
    } else if cell_id.starts_with("E2::SF1::") {
        "MF_SF1".into()
    } else if cell_id.starts_with("E2::SF2::") {
        "MF_SF2".into()
    } else if cell_id.starts_with("E2::SF5::") {
        "MF_SF5".into()
    } else {
        "MF_E2_ROBUSTNESS".into()
    }
}

fn cell_json(result: &CellResult, family: &str, adjusted_p: Option<f64>, procedure: &str) -> Value {
    json!({
        "cell_id": result.cell_id,
        "multiplicity_family": family,
        "multiplicity_procedure": procedure,
        "effect": result.effect,
        "n_high": result.n_high,
        "n_low": result.n_low,
        "raw_p": result.raw_p,
        "adjusted_p_or_q": adjusted_p,
        "status": result.status,
        "excluded_history_incomplete": result.excluded_history_incomplete,
        "excluded_end_of_window": result.excluded_end_of_window,
        "detail": result.detail,
    })
}
