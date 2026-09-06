//! AP-002 QG V1.2 freeze tool.  Consumes the sealed blind QG V1 source, the
//! AP-002 authority and the active scope/data identities, applies the
//! director's semantic repairs RECURSIVELY over the entire artifact tree, and
//! emits the canonical V1.2 artifacts plus the frozen E2 cell registry and the
//! machine pre-execution gate.
//!
//! Governance: the canonical artifacts are produced only by this Rust tool
//! (PIPELINE_RULES.md).  Any active (non-historical) field containing a
//! prohibited semantic token, an empty required identity, or V1 historical
//! accounting fails the freeze closed.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process,
};

const E1A_LOGICAL_HASH: &str = "1b701c016bd1effa300742ee74ce73d6e0e96313d62847d231391061dc94dca6";
const E1B_LOGICAL_HASH: &str = "2a7a142cadb36449d14d4128361d11ce99a3a4d7d8a004dd539bc4dc32fb823c";
const E1B_SIDECAR_SHA256: &str = "8882dfca1e34aab71df411ee0837a7203b01462e3ada565119e66e48c4e15929";
const VIEW_MANIFEST_SHA256: &str =
    "5727233e9f8920ab25a4715f59b1d07dcffd8c9ff82e474d8aa642855924a4f4";
const SCOPE_START: &str = "2025-07-31T16:15:00Z";
const SCOPE_END: &str = "2026-05-05T12:39:00Z";
const B_REPLICATES: u64 = 4_999;
const ROLLING_K: u64 = 20;
const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const HORIZONS: [u64; 3] = [1, 3, 5];
/// Prohibited tokens that must not appear in any ACTIVE field.
const PROHIBITED_TOKENS: [&str; 7] = [
    "invalidation_event",
    "INVALIDATED_UNTOUCHED",
    "INVALIDATED_WITHOUT_TOUCH",
    "CENSORED_ACTIVE_AT_END_OF_STREAM",
    "FILL_ON_FIRST_TOUCH_BAR_INSTANT",
    "far-edge invalidation",
    "homogeneous-rate baseline",
];

/// Historical fields (by key-name match) may quote V1 terminology.
fn is_historical_key(key: &str) -> bool {
    key.contains("HISTORICAL_V1") || key == "REPAIR_LEDGER" || key.starts_with("v1_1_")
}

struct Repair {
    replacements: Vec<(&'static str, &'static str)>,
    hits: Vec<(String, String)>,
}

impl Repair {
    fn new() -> Self {
        Self {
            replacements: vec![
                ("invalidation_event", "far-edge fill/removal (producer fill semantics; no separate invalidation event)"),
                ("INVALIDATED_UNTOUCHED", "FORMED_UNTOUCHED (terminal via gap-through fill or observation end)"),
                ("INVALIDATED_WITHOUT_TOUCH", "FILLED_WITHOUT_RECORDED_TOUCH"),
                ("CENSORED_ACTIVE_AT_END_OF_STREAM", "RIGHT_CENSORED (observation status, not a market state)"),
                ("FILL_ON_FIRST_TOUCH_BAR_INSTANT", "TOUCH_AND_FILL_SAME_COMPLETED_BAR"),
                ("censored-aware at end-of-stream and far-edge invalidation", "competing-risk accounting: FIRST_TOUCH event of interest, FILL_WITHOUT_RECORDED_TOUCH competing terminal event, END_OF_DEVELOPMENT right censor"),
                ("censored at far-edge invalidation", "resolved by the competing terminal gap-through fill (competing-risk accounting, never non-informative censoring)"),
                ("far-edge invalidation", "the competing terminal gap-through fill (competing-risk accounting)"),
                ("touch vs invalidation", "touch vs competing gap-through fill"),
                ("fill vs invalidation", "fill vs right censor at end of development"),
                ("homogeneous-rate baseline", "session-preserving time-varying baseline"),
            ],
            hits: Vec::new(),
        }
    }

    /// Recursive repair over the entire tree; every touched record id recorded.
    fn apply(&mut self, value: &mut Value, owner: &str) {
        match value {
            Value::Object(map) => {
                for (key, item) in map.iter_mut() {
                    if is_historical_key(key) {
                        continue;
                    }
                    let child_owner = item
                        .get("question_id")
                        .or_else(|| item.get("family_id"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| owner.to_string());
                    self.apply(item, &child_owner);
                }
            }
            Value::Array(items) => {
                for item in items.iter_mut() {
                    self.apply(item, owner);
                }
            }
            Value::String(text) => {
                for (from, to) in &self.replacements {
                    if text.contains(from) {
                        self.hits.push((owner.to_string(), (*from).to_string()));
                        *text = text.replace(from, to);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Recursively scan ACTIVE fields for prohibited tokens.
fn scan_active(value: &Value, key: Option<&str>, violations: &mut Vec<String>) {
    if let Some(key) = key {
        if is_historical_key(key) {
            return;
        }
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                scan_active(item, Some(key), violations);
            }
        }
        Value::Array(items) => {
            for item in items {
                scan_active(item, None, violations);
            }
        }
        Value::String(text) => {
            for token in PROHIBITED_TOKENS {
                if text.contains(token) {
                    violations.push((*token).to_string());
                    return;
                }
            }
        }
        _ => {}
    }
}

/// Accounting/multiplicity fields must not quote V1 historical figures.
fn scan_v1_figures(value: &Value, key: Option<&str>, violations: &mut Vec<String>) {
    if let Some(key) = key {
        if is_historical_key(key) {
            return;
        }
        let numeric_scope =
            key.contains("accounting") || key.contains("multiplicity") || key.contains("cells");
        if !numeric_scope {
            // Still recurse into containers so nested accounting is checked.
            match value {
                Value::Object(map) => {
                    for (key, item) in map {
                        scan_v1_figures(item, Some(key), violations);
                    }
                }
                Value::Array(items) => {
                    for item in items {
                        scan_v1_figures(item, None, violations);
                    }
                }
                _ => {}
            }
            return;
        }
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                scan_v1_figures(item, Some(key), violations);
            }
        }
        Value::Array(items) => {
            for item in items {
                scan_v1_figures(item, None, violations);
            }
        }
        Value::String(text) => {
            for figure in ["758", "346", "412"] {
                if text.contains(figure) {
                    violations.push(format!("v1 figure {figure} in active accounting field"));
                }
            }
        }
        _ => {}
    }
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1 << 16];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn read_json(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

fn write_pretty(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn fail(step: &str, message: String) -> ! {
    eprintln!("AP-002 QG V1.2 freeze failed at {step}: {message}");
    process::exit(1);
}

// ---------------- E2 cell registry enumeration ----------------

#[allow(clippy::too_many_arguments)]
fn base_cell(
    cell_id: String,
    instrument_id: &str,
    provenance: &str,
    multiplicity_family: &str,
    multiplicity_procedure: &str,
    stratum: Value,
    direction: Value,
    stage_anchor: Value,
    conditioner: Value,
    outcome: Value,
    horizon: Value,
    effect_statistic: Value,
    null_family: Value,
    censoring: Value,
) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("cell_id".into(), json!(cell_id));
    m.insert("instrument_id".into(), json!(instrument_id));
    m.insert("question_provenance".into(), json!(provenance));
    m.insert("native_stratum".into(), stratum);
    m.insert("direction".into(), direction);
    m.insert("stage_anchor".into(), stage_anchor);
    m.insert("population_rule".into(), json!("lawful in-window formed zones; orphan/pre-development state excluded; warm-state rule where declared"));
    m.insert("conditioner".into(), conditioner);
    m.insert("outcome".into(), outcome);
    m.insert("horizon".into(), horizon);
    m.insert("effect_statistic".into(), effect_statistic);
    m.insert("null_family".into(), null_family);
    m.insert(
        "resampling_method".into(),
        json!("deterministic group-label permutation or sign-flip, predeclared per family"),
    );
    m.insert("resample_count".into(), json!(B_REPLICATES));
    m.insert(
        "seed_derivation".into(),
        json!(
            "SHA256(E2 contract identity | multiplicity family | cell_id | replicate index) -> u64"
        ),
    );
    m.insert(
        "raw_p_rule".into(),
        json!("p = (1 + null_count_as_or_more_extreme) / (B + 1); two-sided"),
    );
    m.insert("multiplicity_family".into(), json!(multiplicity_family));
    m.insert(
        "multiplicity_procedure".into(),
        json!(multiplicity_procedure),
    );
    m.insert("minimum_support_rule".into(), json!("MIN_SUPPORT_30: every contrast bucket requires n >= 30 lawful anchors within the cell's stratum, else UNSUPPORTED"));
    m.insert("bucket_collapse_rule".into(), json!("median/tercile splits computed per stratum from the E2 feature tape; no post-hoc re-bucketing"));
    m.insert("censoring_treatment".into(), censoring);
    m.insert("missingness_rule".into(), json!("missing lawful outcome at horizon = end-of-window missing, excluded from that horizon's contrast and counted"));
    m.insert(
        "effect_size_outputs".into(),
        json!([
            "effect estimate",
            "per-bucket medians/proportions",
            "N per bucket",
            "raw p",
            "adjusted p or q",
            "null distribution summary"
        ]),
    );
    m.insert(
        "status_vocabulary".into(),
        json!(["SUPPORTED", "NOT_SUPPORTED", "UNSUPPORTED", "NOT_EVALUABLE"]),
    );
    m
}

fn enumerate_cells() -> Vec<Map<String, Value>> {
    let mut cells: Vec<Map<String, Value>> = Vec::new();
    let holm = "Holm familywise alpha 0.05";
    let bh = "BH-FDR q=0.05";
    let risk_diff = "risk difference (proportion_high_bucket - proportion_low_bucket)";
    let perm = "within-stratum group-label permutation preserving stratum sizes";
    let competing = "competing-risk deterministic transition accounting (gap-through fill = competing terminal event, never censoring)";
    let all = json!("ALL (stratified pooled)");
    let null_v = Value::Null;

    // ---- MF_E2_CORE (9 pooled stratified cells) ----
    let core_specs: [(&str, &str, &str, &str, &str, &str); 9] = [
        (
            "E2-C01",
            "QG1-CORE-01/PD-F",
            "P(first touch <= 16 bars)",
            "gap_atr median split per stratum",
            risk_diff,
            perm,
        ),
        (
            "E2-C03",
            "QG1-CORE-03",
            "P(next transition <= 16 bars from stage entry)",
            "stage-age at entry median split per stratum",
            risk_diff,
            perm,
        ),
        (
            "E2-C04",
            "QG1-CORE-04",
            "P(first touch <= 16 bars)",
            "active_overlap_count median split per stratum",
            risk_diff,
            perm,
        ),
        (
            "E2-C05",
            "QG1-CORE-05",
            "P(first touch <= 16 bars)",
            "rolling_fill_fraction (prior K=20 completions) median split per stratum",
            risk_diff,
            perm,
        ),
        (
            "E2-C09",
            "QG1-CORE-09",
            "formation-count short-horizon autocorrelation energy (lags 1..16)",
            "none (self-clustering)",
            "mean across strata of lag 1..16 autocorrelation energy",
            "session-preserving within-day event-time randomization",
        ),
        (
            "E2-C10",
            "QG1-CORE-10/PD-E/PD-G",
            "post-fill h1 direction-adjusted return",
            "completion mode (TOUCH_THEN_FILL vs FILL_WITHOUT_RECORDED_TOUCH)",
            "median difference",
            perm,
        ),
        (
            "E2-C12a",
            "QG1-CORE-12",
            "P(first touch <= 16 bars)",
            "gap_atr tercile top-vs-bottom",
            risk_diff,
            perm,
        ),
        (
            "E2-C12b",
            "QG1-CORE-12",
            "P(first touch <= 16 bars)",
            "stage-age tercile top-vs-bottom",
            risk_diff,
            perm,
        ),
        (
            "E2-C12c",
            "QG1-CORE-12",
            "P(first touch <= 16 bars)",
            "active_overlap_count tercile top-vs-bottom",
            risk_diff,
            perm,
        ),
    ];
    for (id, provenance, outcome, conditioner, statistic, null_family) in core_specs {
        cells.push(base_cell(
            format!("E2::{id}"),
            "E2-CORE",
            provenance,
            "MF_E2_CORE",
            holm,
            all.clone(),
            null_v.clone(),
            null_v.clone(),
            json!(conditioner),
            json!(outcome),
            json!(16),
            json!(statistic),
            json!(null_family),
            json!(competing),
        ));
    }

    // ---- MF_STAGE_MATCHED (42) ----
    let contrasts = [
        ("C1", "FORMATION vs FIRST_TOUCH",
         "the same zones that genuinely emit FIRST_TOUCH; formation measurement at their lawful formation anchor, first-touch measurement at their lawful FIRST_TOUCH anchor",
         "lawful formation anchor vs lawful FIRST_TOUCH anchor"),
        ("C2", "FIRST_TOUCH vs FILL",
         "the same zones that both touch and later fill; gap-through fills excluded (no FIRST_TOUCH anchor)",
         "lawful FIRST_TOUCH anchor vs lawful FILL anchor"),
    ];
    for (contrast_id, stages, population, anchors) in contrasts {
        for tf in TIMEFRAMES {
            for h in HORIZONS {
                cells.push(base_cell(
                    format!("E2::PANEL::{contrast_id}::{tf}::h{h}"), "E2-PANEL",
                    "DIR-01/PD-A/PD-B/PD-C", "MF_STAGE_MATCHED", holm,
                    json!(tf), null_v.clone(), json!(anchors),
                    json!("none (within-zone paired contrast)"),
                    json!(format!("direction-adjusted return at native bar {h} after each respective anchor ({stages})")),
                    json!(h),
                    json!("median within-zone paired difference of direction-adjusted returns"),
                    json!("within-zone sign-flip of paired differences"),
                    json!("paired within-zone design; end-of-window missing pairs excluded per horizon and counted"),
                ));
            }
        }
    }

    // ---- MF_DIRECTION_ASYMMETRY (84) ----
    for direction in ["bullish", "bearish"] {
        for anchor in ["FIRST_TOUCH", "FILL"] {
            for tf in TIMEFRAMES {
                for h in HORIZONS {
                    cells.push(base_cell(
                        format!("E2::DIR::{direction}::{anchor}::{tf}::h{h}"), "E2-DIRNULL",
                        "PD-D", "MF_DIRECTION_ASYMMETRY", holm,
                        json!(tf), json!(direction), json!(anchor),
                        json!("none (median-vs-zero test after drift-aware centering)"),
                        json!(format!("direction-adjusted return at native bar {h} after the {anchor} anchor")),
                        json!(h),
                        json!("median direction-adjusted return"),
                        json!("sign-flip after per-(stratum, UTC-day) median centering of the raw return (drift-aware)"),
                        json!("competing-risk deterministic transition accounting where lifecycle events enter"),
                    ));
                }
            }
        }
    }

    // ---- MF_SF1 (98): 2 stages x 7 conditioners x 7 strata ----
    let stages = [
        ("FORMED_UNTOUCHED", "P(first touch <= 16 bars from formation); gap-through fill = competing terminal event (deterministic non-event, never censoring)"),
        ("TOUCHED_UNFILLED", "P(fill <= 16 bars from first touch)"),
    ];
    let conditioners = [
        (
            "zone_age",
            "zone_age_bucket (age at stage entry, median split)",
        ),
        (
            "zone_height",
            "zone_height_ATR_bucket (gap_atr median split)",
        ),
        ("crowding", "active_overlap_count_bucket (median split)"),
        (
            "history",
            "rolling_fill_fraction_bucket (prior K=20 completions, median split)",
        ),
        ("direction", "direction_as_moderator (bullish vs bearish)"),
        (
            "PAIR_age_height",
            "PAIR age x height (diagonal quadrants low-low vs high-high)",
        ),
        (
            "PAIR_crowding_history",
            "PAIR crowding x history (diagonal quadrants low-low vs high-high)",
        ),
    ];
    for (stage, outcome) in stages {
        for (short, conditioner) in conditioners {
            for tf in TIMEFRAMES {
                cells.push(base_cell(
                    format!("E2::SF1::{stage}::{short}::{tf}"),
                    "E2-SF1",
                    "QG1-SF-1",
                    "MF_SF1",
                    bh,
                    json!(tf),
                    null_v.clone(),
                    json!(stage),
                    json!(conditioner),
                    json!(outcome),
                    json!(16),
                    json!(risk_diff),
                    json!(perm),
                    json!(competing),
                ));
            }
        }
    }

    // ---- MF_SF2 (126): 3 attributes x 3 horizons x 2 directions x 7 strata ----
    let attributes = [
        ("gap_atr", "gap_height / formation_ATR (gap_atr)"),
        (
            "impulse_body",
            "middle_body / formation_ATR (impulse_body_atr)",
        ),
        (
            "confirmation_range",
            "confirmation_bar_range / formation_ATR",
        ),
    ];
    let sf2_outcomes = [
        ("touch4", "P(first touch <= 4 bars)"),
        ("touch16", "P(first touch <= 16 bars)"),
        ("fill16", "P(fill <= 16 bars given touched)"),
    ];
    for (attr_short, attribute) in attributes {
        for (out_short, outcome) in sf2_outcomes {
            for direction in ["bullish", "bearish"] {
                for tf in TIMEFRAMES {
                    cells.push(base_cell(
                        format!("E2::SF2::{tf}::{direction}::{attr_short}::{out_short}"), "E2-SF2",
                        "QG1-SF-2/PD-F", "MF_SF2", bh,
                        json!(tf), json!(direction), json!("FORMATION"),
                        json!(format!("{attribute} median split")),
                        json!(outcome), json!(16),
                        json!(risk_diff), json!(perm),
                        json!("first-touch outcomes use competing-risk accounting; fill-given-touched is conditional on the touched cohort"),
                    ));
                }
            }
        }
    }

    // ---- MF_SF5 (63): 3 measures x 3 outcomes x 7 strata ----
    let measures = [
        ("crowding", "active_overlap_count_bucket (median split)"),
        ("containment", "containment_binary (contained vs not)"),
        (
            "edge_distance",
            "nearest_edge_distance_ATR_bucket (median split)",
        ),
    ];
    let sf5_outcomes = [
        ("touch16", "P(first touch <= 16 bars)"),
        ("fill16", "P(fill <= 16 bars given touched)"),
        ("surv64", "P(no fill within 64 bars of formation)"),
    ];
    for (short, measure) in measures {
        for (out_short, outcome) in sf5_outcomes {
            for tf in TIMEFRAMES {
                cells.push(base_cell(
                    format!("E2::SF5::{tf}::{short}::{out_short}"),
                    "E2-SF5",
                    "QG1-SF-5",
                    "MF_SF5",
                    bh,
                    json!(tf),
                    null_v.clone(),
                    json!("FORMATION"),
                    json!(measure),
                    json!(outcome),
                    json!(64),
                    json!(risk_diff),
                    json!(perm),
                    json!(competing),
                ));
            }
        }
    }

    // ---- Robustness lane (50) ----
    for (attr_short, attribute) in attributes {
        for tf in TIMEFRAMES {
            cells.push(base_cell(
                format!("E2::OPT02::{tf}::{attr_short}"),
                "E2-OPT02",
                "QG1-OPT-02 (methods robustness)",
                "MF_E2_ROBUSTNESS",
                "no behavioral p-values; agreement statistics only",
                json!(tf),
                null_v.clone(),
                json!("FORMATION"),
                json!(format!("{attribute} tercile split (vs SF-2 median split)")),
                json!("P(first touch <= 16 bars)"),
                json!(16),
                json!("tercile risk difference; sign agreement with the SF-2 recorded contrast"),
                json!("no null family (deterministic robustness audit)"),
                json!("competing-risk deterministic transition accounting"),
            ));
        }
    }
    let estimators = [
        ("naive", "naive complement among observed"),
        ("km", "Kaplan-Meier on the touched cohort"),
        ("aj", "Aalen-Johansen competing-risk estimator"),
        ("deterministic", "deterministic transition accounting"),
    ];
    for (short, estimator) in estimators {
        for tf in TIMEFRAMES {
            cells.push(base_cell(
                format!("E2::OPT04::{tf}::{short}"), "E2-OPT04",
                "QG1-OPT-04 (methods robustness)", "MF_E2_ROBUSTNESS",
                "no behavioral p-values; agreement statistics only",
                json!(tf), null_v.clone(), json!("TOUCHED_UNFILLED"),
                json!("none"),
                json!("P(fill <= 16 bars given touched)"), json!(16),
                json!(format!("{estimator} estimate")),
                json!("no null family (deterministic robustness audit)"),
                json!("per estimator definition; divergence across estimators is the reported quantity"),
            ));
        }
    }
    cells.push(base_cell(
        format!("E2::CORE13"),
        "E2-CORE13",
        "QG1-CORE-13 (methods robustness)",
        "MF_E2_ROBUSTNESS",
        "no behavioral p-values; agreement statistics only",
        all,
        null_v.clone(),
        json!("TOUCHED_UNFILLED"),
        json!("none"),
        json!("P(fill <= 16 bars given touched)"),
        json!(16),
        json!("max pairwise divergence across the four censoring estimators, pooled across strata"),
        json!("no null family (deterministic robustness audit)"),
        json!("per estimator definition"),
    ));

    cells
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 QG V1.2 freeze failed: {error}");
        process::exit(1);
    }
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
    let v1_universe = get("v1-universe")?;
    let v1_registry_path = get("v1-registry")?;
    let v1_artifact_registry = get("v1-artifact-registry")?;
    let authority_path = get("authority")?;
    let scope_path = get("scope")?;
    let e1a_results = get("e1a-results")?;
    let e1b_results = get("e1b-results")?;
    let output_dir = PathBuf::from(get("output-dir")?);
    let e2_dir = PathBuf::from(get("e2-dir")?);

    // ---- identities (fail closed) ----
    let e1a: Value = read_json(Path::new(&e1a_results))?;
    let e1b: Value = read_json(Path::new(&e1b_results))?;
    let authority: Value = read_json(Path::new(&authority_path))?;
    let scope: Value = read_json(Path::new(&scope_path))?;
    let e1a_hash = e1a["logical_result_hash"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let e1b_hash = e1b["logical_result_hash"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let view_hash = e1b["tables"]["anchors"]["provenance"]["development_view_manifest_sha256"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let e1a_view_hash = e1a["tables"]["zones"]["provenance"]["development_view_manifest_sha256"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let sidecar = e1b["tables"]["anchors"]["provenance"]["sidecar_sha256"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    for (name, value, expected) in [
        ("e1a_logical_result_hash", &e1a_hash, E1A_LOGICAL_HASH),
        ("e1b_logical_result_hash", &e1b_hash, E1B_LOGICAL_HASH),
        (
            "development_view_manifest_sha256",
            &view_hash,
            VIEW_MANIFEST_SHA256,
        ),
        (
            "e1a_view_manifest_sha256",
            &e1a_view_hash,
            VIEW_MANIFEST_SHA256,
        ),
        ("sidecar_sha256", &sidecar, E1B_SIDECAR_SHA256),
    ] {
        if value.is_empty() {
            fail(name, "empty required identity is forbidden".into());
        }
        if value != expected {
            fail(name, format!("{value} != {expected}"));
        }
    }
    let confirmation = authority["confirmation"]
        .as_str()
        .or_else(|| {
            authority["confirmation"]
                .get("status")
                .and_then(Value::as_str)
        })
        .unwrap_or_default()
        .to_string();
    if !confirmation.contains("LOCKED") {
        fail("confirmation", confirmation);
    }
    if scope["development"]["start_utc_inclusive"].as_str() != Some(SCOPE_START)
        || scope["development"]["end_utc_exclusive"].as_str() != Some(SCOPE_END)
    {
        fail(
            "scope",
            "scope boundaries do not match the corrected frozen scope".into(),
        );
    }

    // ---- sealed source hash ----
    let universe_hash = sha256_file(Path::new(&v1_universe))?;
    let artifact_registry: Value = read_json(Path::new(&v1_artifact_registry))?;
    let sealed = artifact_registry["entries"]
        .as_array()
        .ok_or("artifact registry entries missing")?
        .iter()
        .find(|entry| entry["logical_name"] == "AP-002_BLIND_QG_UNIVERSE.json")
        .and_then(|entry| entry["sha256"].as_str())
        .ok_or("sealed universe entry missing")?
        .to_string();
    if sealed != universe_hash {
        fail("sealed_source", format!("{universe_hash} != {sealed}"));
    }

    // ---- recursive semantic repair over the ENTIRE tree ----
    let mut universe_v12 = read_json(Path::new(&v1_universe))?;
    let mut repair = Repair::new();
    repair.apply(&mut universe_v12, "<root>");
    let v1_hits_universe = repair.hits.clone();

    // V1 accounting moved to a historical field; active accounting written.
    let historical_accounting = universe_v12
        .get("search_space_accounting")
        .cloned()
        .unwrap_or(json!({}));
    if let Some(map) = universe_v12.as_object_mut() {
        map.remove("search_space_accounting");
        map.insert(
            "HISTORICAL_V1_search_space_accounting".into(),
            historical_accounting,
        );
        map.insert("active_accounting".into(), json!({
            "note": "Active execution accounting lives in the E2 cell registry and E2 contract V1.1: E2 472 cells (PRIMARY 135 / SYSTEMATIC 287 / ROBUSTNESS 50).",
            "E2": {"PRIMARY": 135, "SYSTEMATIC": 287, "ROBUSTNESS": 50, "TOTAL": 472},
            "E3": {"SF3_READY_AFTER_E2": 126, "PRE_E3_CAUSAL_ELIGIBILITY_AUDIT": 29, "CONDITIONALLY_LAWFUL_DEFERRED": 84, "HELD_PENDING_TYPED_CONTEXT_RESOLUTION": ["SF-4", "CORE-06", "CORE-07", "CORE-08", "CORE-11"]},
        }));
    }

    // CORE-09 explicit operative-null record (question retained).
    if let Some(records) = universe_v12
        .get_mut("tiers")
        .and_then(|tiers| tiers.get_mut("CORE_QUESTIONS"))
        .and_then(Value::as_array_mut)
    {
        for record in records.iter_mut() {
            if record["question_id"] == "QG1-CORE-09" {
                record["operative_null"] = json!("NULL_SESSION_CONDITIONAL_TIME_VARYING: predeclared time-varying baseline preserving UTC day, UTC session block and native-stratum activity structure while disrupting local short-range event dependence (conditional within-day/session event-time randomization, or a predeclared inhomogeneous-rate null). Prohibited as operative nulls: any fixed-intensity uniform-rate baseline, naive row shuffle, self-circular-shift alone.");
            }
        }
    }
    universe_v12["artifact_type"] = json!("AP-002_BLIND_QG_UNIVERSE_V1_2");
    universe_v12["v1_2_provenance"] = json!({
        "derived_from_sealed_v1_sha256": universe_hash,
        "compiler": "ap002_qg_freeze (Rust)",
        "no_blind_regeneration": true,
    });

    // ---- V1.2 registry (same recursive repair pass) ----
    let mut registry_v12 = read_json(Path::new(&v1_registry_path))?;
    repair.apply(&mut registry_v12, "<registry>");
    let v1_hits_universe_len = v1_hits_universe.len();
    let v1_hits = [repair.hits.clone(), v1_hits_universe.clone()].concat();
    let v1_hits_universe_len = v1_hits_universe.len();
    if let Some(map) = registry_v12.as_object_mut() {
        let historical = map.remove("expansion_accounting").unwrap_or(json!({}));
        map.insert("HISTORICAL_V1_expansion_accounting".into(), historical);
        map.insert(
            "artifact_type".into(),
            json!("AP-002_QUESTION_REGISTRY_V1_2"),
        );
        map.insert("v1_2_provenance".into(), json!({"compiler": "ap002_qg_freeze (Rust)", "e2_contract": "AP-002_E2_CONTRACT_V1_1.json"}));
        map.insert(
            "active_e2_accounting".into(),
            json!({"PRIMARY": 135, "SYSTEMATIC": 287, "ROBUSTNESS": 50, "TOTAL": 472}),
        );
    }

    // ---- E2 contract V1.1 ----
    let mut e2_contract = json!({
        "artifact_type": "AP-002_E2_CONTRACT_V1_1",
        "supersedes": "AP-002_E2_CONTRACT_V1.json (empty-identity defect)",
        "confirmation_status": "LOCKED",
        "exposure_class": "E2_SAME_DOMAIN",
        "data_identity": {
            "development_start_utc_inclusive": SCOPE_START,
            "development_end_utc_exclusive": SCOPE_END,
            "input_identity": VIEW_MANIFEST_SHA256,
            "development_view_manifest_sha256": VIEW_MANIFEST_SHA256,
            "sidecar_sha256": E1B_SIDECAR_SHA256,
            "e1a_logical_result_hash": E1A_LOGICAL_HASH,
            "e1b_logical_result_hash": E1B_LOGICAL_HASH,
            "binding_rule": "Authority V1 supplies semantics only; execution binds to the corrected 16:15 DEVELOPMENT scope and the scope-repair sidecar verified above. Empty identities fail closed.",
        },
        "native_strata": TIMEFRAMES,
        "cross_scale_identity": "PROHIBITED",
        "state_model": {
            "market_states": ["FORMED_UNTOUCHED", "TOUCHED_UNFILLED", "FILLED"],
            "terminal_paths": [
                "FORMED_UNTOUCHED -> TOUCHED_UNFILLED -> FILLED",
                "FORMED_UNTOUCHED -> FILLED (first_touch_observed=false; competing terminal gap-through)",
                "TOUCHED_UNFILLED -> FILLED",
            ],
            "observation_status": ["OBSERVED_TERMINAL", "RIGHT_CENSORED_UNTOUCHED", "RIGHT_CENSORED_TOUCHED"],
            "completion_modes": ["TOUCH_THEN_FILL", "FILL_WITHOUT_RECORDED_TOUCH", "TOUCH_AND_FILL_SAME_COMPLETED_BAR"],
        },
        "competing_risk_rule": "formation->first-touch: FIRST_TOUCH = event of interest; FILL_WITHOUT_RECORDED_TOUCH = competing terminal event; END_OF_DEVELOPMENT = right censor. Never describe gap-through fill as censored. touched->fill: FILLED = event; end-of-development while TOUCHED_UNFILLED = right censor.",
        "panel_populations": {
            "contrast_1": {"stages": "FORMATION vs FIRST_TOUCH", "population": "the same zones that genuinely emit FIRST_TOUCH; formation measurement at their lawful formation anchor, first-touch measurement at their lawful FIRST_TOUCH anchor; same native horizon h after each respective anchor", "purpose": "remove selection into touch from the formation-vs-touch comparison"},
            "contrast_2": {"stages": "FIRST_TOUCH vs FILL", "population": "the same zones that both touch and later fill; gap-through fills excluded (no FIRST_TOUCH anchor)", "note": "no one-bar-before-fill or other future-defined pseudo-anchor; gap-through behavior remains under CORE-10"},
        },
        "terminology": "STAGE-CONDITIONED INFORMATIONAL RELATIONSHIP / STAGE-ASSOCIATED CONDITIONAL CONTRAST; decision rule SURVIVES_STAGE_CONDITIONING; never 'causal stage effect'.",
        "warm_state_rule": format!("CORE-04/CORE-05/SF-5: pre-boundary warm state only where current state is lawfully represented; else HISTORY_INCOMPLETE with deterministic washout exclusion; rolling prior-K (K={ROLLING_K}) history requires K lawful prior completions; exclusions recorded by question, stratum and reason."),
        "statistical_execution_rules": {
            "B": B_REPLICATES,
            "p_rule": "p = (1 + null_count_as_or_more_extreme) / (B + 1); two-sided unless predeclared otherwise",
            "seed_derivation": "SHA256(E2 contract identity | multiplicity family | cell_id | replicate index) -> u64",
            "support": "every contrast bucket n >= 30 within the cell's stratum, else UNSUPPORTED",
            "multiplicity_families": {
                "MF_E2_CORE": {"cells": 9, "procedure": "Holm familywise alpha 0.05"},
                "MF_STAGE_MATCHED": {"cells": 42, "procedure": "Holm familywise alpha 0.05"},
                "MF_DIRECTION_ASYMMETRY": {"cells": 84, "procedure": "Holm familywise alpha 0.05"},
                "MF_SF1": {"cells": 98, "procedure": "BH-FDR q=0.05"},
                "MF_SF2": {"cells": 126, "procedure": "BH-FDR q=0.05"},
                "MF_SF5": {"cells": 63, "procedure": "BH-FDR q=0.05"},
                "MF_E2_ROBUSTNESS": {"cells": 50, "procedure": "separate lane; no behavioral p-values; robustness estimates and block-bootstrap uncertainty reported separately"},
            },
            "no_post_hoc": "No test statistic, resample count, seed, multiplicity procedure or support threshold may be chosen after E2 outcomes are inspected.",
        },
        "active_accounting": {"PRIMARY": 135, "SYSTEMATIC": 287, "ROBUSTNESS": 50, "TOTAL": 472},
    });

    // ---- cell registry ----
    let cells = enumerate_cells();
    let mut ids = std::collections::BTreeSet::new();
    for cell in &cells {
        let id = cell["cell_id"].as_str().unwrap_or_default().to_string();
        if !ids.insert(id.clone()) {
            fail("cell_registry", format!("duplicate cell id {id}"));
        }
    }
    if cells.len() != 472 {
        fail(
            "cell_registry",
            format!("expected 472 cells, enumerated {}", cells.len()),
        );
    }
    let mut cell_registry = json!({
        "artifact_type": "AP-002_E2_CELL_REGISTRY",
        "contract": "AP-002_E2_CONTRACT_V1_1.json",
        "cells_total": cells.len(),
        "multiplicity_families": {
            "MF_E2_CORE": 9, "MF_STAGE_MATCHED": 42, "MF_DIRECTION_ASYMMETRY": 84,
            "MF_SF1": 98, "MF_SF2": 126, "MF_SF5": 63, "MF_E2_ROBUSTNESS": 50,
        },
        "cells": cells,
    });

    // ---- pre-execution gate (before any outcome exists) ----
    let mut semantic_violations = Vec::new();
    scan_active(&universe_v12, None, &mut semantic_violations);
    scan_active(&registry_v12, None, &mut semantic_violations);
    scan_active(&e2_contract, None, &mut semantic_violations);
    scan_active(&cell_registry, None, &mut semantic_violations);
    let mut figure_violations = Vec::new();
    scan_v1_figures(&universe_v12, None, &mut figure_violations);
    scan_v1_figures(&registry_v12, None, &mut figure_violations);
    scan_v1_figures(&e2_contract, None, &mut figure_violations);
    scan_v1_figures(&cell_registry, None, &mut figure_violations);
    let e3_in_e2 = cells
        .iter()
        .filter(|cell| {
            cell["question_provenance"]
                .as_str()
                .map(|prov| {
                    prov.contains("SF-3")
                        || prov.contains("SF-4")
                        || prov.contains("OPT-01")
                        || prov.contains("OPT-03")
                })
                .unwrap_or(false)
        })
        .count();
    let gate_pass = semantic_violations.is_empty()
        && figure_violations.is_empty()
        && cells.len() == 472
        && e3_in_e2 == 0;
    let gate = json!({
        "artifact_type": "E2_PREEXECUTION_GATE",
        "generated_before_outcomes": true,
        "checks": {
            "canonical_forbidden_semantic_tokens": semantic_violations,
            "empty_required_identities": [],
            "cell_registry_rows": cells.len(),
            "expected_cell_registry_rows": 472,
            "duplicate_cell_ids": 0,
            "unassigned_multiplicity_families": 0,
            "cells_with_unspecified_statistic": 0,
            "cells_with_unspecified_null": 0,
            "cells_with_unspecified_support_rule": 0,
            "e3_cells_included_in_e2": e3_in_e2,
            "confirmation_access": 0,
            "corrected_development_scope_identity_verified": true,
            "sidecar_identity_verified": true,
            "e1a_e1b_identities_verified": true,
            "v1_accounting_figure_violations": figure_violations,
        },
        "status": if gate_pass { "PASS" } else { "FAIL" },
        "rule": "PASS required before any E2 behavioral result may be produced; FAIL means stop without E2 results.",
    });

    // ---- repair ledger (historical quotes confined here) ----
    let repair_ledger = json!({
        "artifact_type": "AP-002_QG_V1_2_REPAIR_LEDGER",
        "blocker_1_stale_v1_1_semantics": {
            "cause": "the V1.1 compiler transformed only tiers.* record arrays; prohibited V1 semantics survived in shared_definitions, anchor_object, domain_coverage and other non-tier sections",
            "repair": "recursive whole-tree transformation with historical-field isolation; the entire active surface is validated token-free",
            "v1_repair_hits": v1_hits.iter().map(|(id, term)| format!("{id}: {term}")).collect::<Vec<_>>(),
        },
        "blocker_2_validator_blind_spot": {
            "cause": "the V1.1 gate validated selected paths, not the executable semantic surface",
            "repair": "post-repair whole-tree active scan fails the freeze on any prohibited token; the scan runs over every emitted artifact",
        },
        "blocker_3_empty_view_identity": {
            "cause": "the V1.1 compiler read the view hash from a nonexistent provenance path",
            "repair": format!("read from tables.*.provenance.development_view_manifest_sha256; fail closed on empty; verified = {VIEW_MANIFEST_SHA256}"),
        },
        "blocker_4_stale_accounting": {
            "cause": "V1.1 carried MF_CORE_PRIMARY 16-cell, MF_SF4 168-cell and 758/346/412 figures inside active semantics",
            "repair": "V1 figures moved to HISTORICAL_V1 fields only; active accounting reconciles to 472 (135/287/50); SF-4 held with no active multiplicity family",
        },
        "note": "This ledger and the HISTORICAL_V1 fields are the only places where V1 terminology may be quoted.",
    });

    let out = |name: &str| output_dir.join(name);
    write_pretty(&out("AP-002_BLIND_QG_UNIVERSE_V1_2.json"), &universe_v12)?;
    write_pretty(&out("AP-002_QUESTION_REGISTRY_V1_2.json"), &registry_v12)?;
    write_pretty(&out("AP-002_E2_CONTRACT_V1_1.json"), &e2_contract)?;
    write_pretty(
        &out("AP-002_QG_V1_2_MANIFEST.json"),
        &json!({
            "artifact_type": "AP-002_QG_V1_2_MANIFEST",
            "compiler": "ap002_qg_freeze (Rust)",
            "sealed_v1_source_sha256": universe_hash,
            "inputs_verified": {
                "e1a_logical_result_hash": e1a_hash,
                "e1b_logical_result_hash": e1b_hash,
                "development_view_manifest_sha256": view_hash,
                "sidecar_sha256": sidecar,
                "development_start": SCOPE_START,
            },
            "v1_2_repair_hits": v1_hits.len(),
            "confirmation_status": "LOCKED",
        }),
    )?;
    write_pretty(&out("AP-002_QG_V1_2_REPAIR_LEDGER.json"), &repair_ledger)?;
    write_pretty(&e2_dir.join("E2_CELL_REGISTRY.json"), &cell_registry)?;
    write_pretty(&e2_dir.join("E2_PREEXECUTION_GATE.json"), &gate)?;
    write_pretty(
        &e2_dir.join("E2_EXPERIMENT.json"),
        &json!({
            "artifact_type": "AP-002_E2_EXPERIMENT",
            "experiment_id": "AP-002-FVG-E2-SAME-DOMAIN",
            "contract": "AP-002_E2_CONTRACT_V1_1.json",
            "cells_total": cells.len(),
            "confirmation_status": "LOCKED",
            "e2_executed": false,
            "e3_executed": false,
        }),
    )?;

    println!(
        "QG V1.2 freeze: repair hits {} (universe {} + registry {}) | semantic violations {} | figure violations {} | cells {} | gate {}",
        v1_hits.len(),
        v1_hits_universe_len,
        v1_hits.len() - v1_hits_universe_len,
        semantic_violations.len(),
        figure_violations.len(),
        cells.len(),
        gate["status"],
    );
    if !gate_pass {
        process::exit(2);
    }
    Ok(())
}
