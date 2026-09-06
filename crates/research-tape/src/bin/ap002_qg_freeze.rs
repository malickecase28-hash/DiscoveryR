//! AP-002 QG V1.1 freeze tool.  Consumes the sealed blind QG V1 source, the
//! AP-002 authority, the active scope/data identities, the phenotype-derived
//! ledger and the director ledger, validates them, applies the director's
//! semantic and governance repairs, and emits the canonical V1.1 artifacts.
//!
//! The sealed blind V1 artifacts are read-only inputs; the blindness result
//! stands.  This tool is the only sanctioned producer of the V1.1 records
//! (PIPELINE_RULES.md).

use serde_json::{json, Value};
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
const SCOPE_DEVELOPMENT_START: &str = "2025-07-31T16:15:00Z";
const CORE_QUESTION_FIELDS: [&str; 20] = [
    "question_id",
    "plain_language_question",
    "formal_hypothesis",
    "anchor",
    "anchor_native_scale_domain",
    "context_detector_if_applicable",
    "context_native_scale_domain",
    "outcome",
    "prospective_horizon_domain",
    "exposure_class",
    "causal_eligibility_rule",
    "required_fields",
    "forbidden_fields",
    "appropriate_null_family",
    "required_controls",
    "minimum_support_concept",
    "multiplicity_family",
    "known_dependency_caveat",
    "reason_question_is_semantically_meaningful",
    "reason_it_might_be_scientifically_useful",
];
const FORBIDDEN_LIFECYCLE_TERMS: [&str; 6] = [
    "invalidation_event",
    "INVALIDATED_UNTOUCHED",
    "INVALIDATED_WITHOUT_TOUCH",
    "CENSORED_ACTIVE_AT_END_OF_STREAM",
    "FILL_ON_FIRST_TOUCH_BAR_INSTANT",
    "homogeneous-rate baseline",
];
const FORBIDDEN_OUTCOME_FIELD_TOKENS: [&str; 5] = [
    "raw_return_bps",
    "direction_adjusted_return_bps",
    "raw_price_delta",
    "outcome_close",
    "anchor_mid",
];
const NON_FVG_FAMILIES: usize = 22;
/// Director role map for cross-domain context families (proposed pending
/// authority confirmation; every registry family is currently UNRESOLVED).
const PROPOSED_ROLES: [(&str, &str, &str); 8] = [
    (
        "feed_health",
        "DATA_QUALITY",
        "validity/availability readouts only; never directional explanatory evidence",
    ),
    ("spread_state", "STATE_REGIME", "regime/state readouts"),
    (
        "drift_burst",
        "EVENT_LIFECYCLE",
        "event presence/state/lifecycle readouts",
    ),
    (
        "time_context",
        "TEMPORAL_STATE",
        "temporal state readouts; generic event-presence readouts are meaningless",
    ),
    (
        "vwma_atr",
        "NORMALIZATION_REFERENCE",
        "normalization/reference readouts",
    ),
    (
        "order_blocks",
        "PERSISTENT_STRUCTURAL_OBJECT",
        "active-object state/relationship readouts preferred over mere emission presence",
    ),
    (
        "structural_liquidity",
        "PERSISTENT_STRUCTURAL_OBJECT",
        "active-object state/relationship readouts",
    ),
    ("range", "STATE_REGIME", "regime/state readouts"),
];

struct Ctx {
    checks: Vec<(String, bool, String)>,
}

impl Ctx {
    fn check(&mut self, name: &str, ok: bool, evidence: String) -> Result<(), String> {
        self.checks.push((name.to_string(), ok, evidence.clone()));
        if ok {
            Ok(())
        } else {
            Err(format!("validation failed: {name}: {evidence}"))
        }
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

/// Collect every string value under `term`-bearing JSON paths, mapped to the
/// owning object's `question_id`/`family_id` where present.
fn scan_vocabulary(records: &[Value], terms: &[&str]) -> Vec<(String, String, String)> {
    let mut hits = Vec::new();
    for record in records {
        let id = record
            .get("question_id")
            .or_else(|| record.get("family_id"))
            .and_then(Value::as_str)
            .unwrap_or("<unnamed>")
            .to_string();
        let text = serde_json::to_string(record).unwrap_or_default();
        for term in terms {
            if text.contains(term) {
                hits.push((id.clone(), (*term).to_string(), term.to_string()));
            }
        }
    }
    hits
}

fn replace_terms_in_record(record: &mut Value, replacements: &[(&str, &str)]) -> usize {
    let mut replaced = 0;
    match record {
        Value::String(text) => {
            for (from, to) in replacements {
                if text.contains(from) {
                    replaced += text.matches(from).count();
                    *text = text.replace(from, to);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                replaced += replace_terms_in_record(item, replacements);
            }
        }
        Value::Object(map) => {
            for (_, value) in map.iter_mut() {
                replaced += replace_terms_in_record(value, replacements);
            }
        }
        _ => {}
    }
    replaced
}

fn core_cells(question_id: &str) -> u64 {
    if question_id == "QG1-CORE-12" {
        3
    } else {
        1
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 QG freeze failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut v1_universe = PathBuf::new();
    let mut v1_registry_path = PathBuf::new();
    let mut v1_artifact_registry = PathBuf::new();
    let mut authority_path = PathBuf::new();
    let mut scope_path = PathBuf::new();
    let mut detector_registry_path = PathBuf::new();
    let mut e1a_results = PathBuf::new();
    let mut e1b_results = PathBuf::new();
    let mut output_dir = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--v1-universe" => v1_universe = args.next().ok_or("missing --v1-universe")?.into(),
            "--v1-registry" => {
                v1_registry_path = args.next().ok_or("missing --v1-registry")?.into()
            }
            "--v1-artifact-registry" => {
                v1_artifact_registry = args.next().ok_or("missing --v1-artifact-registry")?.into()
            }
            "--authority" => authority_path = args.next().ok_or("missing --authority")?.into(),
            "--scope" => scope_path = args.next().ok_or("missing --scope")?.into(),
            "--detector-registry" => {
                detector_registry_path = args.next().ok_or("missing --detector-registry")?.into()
            }
            "--e1a-results" => e1a_results = args.next().ok_or("missing --e1a-results")?.into(),
            "--e1b-results" => e1b_results = args.next().ok_or("missing --e1b-results")?.into(),
            "--output-dir" => output_dir = args.next().ok_or("missing --output-dir")?.into(),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    for (name, path) in [
        ("--v1-universe", &v1_universe),
        ("--v1-registry", &v1_registry_path),
        ("--v1-artifact-registry", &v1_artifact_registry),
        ("--authority", &authority_path),
        ("--scope", &scope_path),
        ("--detector-registry", &detector_registry_path),
        ("--e1a-results", &e1a_results),
        ("--e1b-results", &e1b_results),
        ("--output-dir", &output_dir),
    ] {
        if path.as_os_str().is_empty() {
            return Err(format!("{name} is required").into());
        }
    }

    let mut ctx = Ctx { checks: Vec::new() };

    // ---- 1. Allowed source hashes ----
    let universe_bytes_hash = sha256_file(&v1_universe)?;
    let artifact_registry: Value = read_json(&v1_artifact_registry)?;
    let sealed_hash = artifact_registry["entries"]
        .as_array()
        .ok_or("artifact registry entries missing")?
        .iter()
        .find(|entry| entry["logical_name"] == "AP-002_BLIND_QG_UNIVERSE.json")
        .and_then(|entry| entry["sha256"].as_str())
        .ok_or("sealed universe entry missing from artifact registry")?
        .to_string();
    ctx.check(
        "sealed_blind_source_hash",
        sealed_hash == universe_bytes_hash,
        universe_bytes_hash.clone(),
    )?;

    // ---- 2. Active data identities ----
    let e1a: Value = read_json(&e1a_results)?;
    let e1b: Value = read_json(&e1b_results)?;
    let authority: Value = read_json(&authority_path)?;
    let scope: Value = read_json(&scope_path)?;
    let e1a_hash = e1a["logical_result_hash"].as_str().unwrap_or_default();
    let e1b_hash = e1b["logical_result_hash"].as_str().unwrap_or_default();
    let e1b_sidecar = e1b["tables"]["anchors"]["provenance"]["sidecar_sha256"]
        .as_str()
        .unwrap_or_default();
    ctx.check(
        "e1a_active_identity",
        e1a_hash == E1A_LOGICAL_HASH,
        e1a_hash.to_string(),
    )?;
    ctx.check(
        "e1b_active_identity",
        e1b_hash == E1B_LOGICAL_HASH,
        e1b_hash.to_string(),
    )?;
    ctx.check(
        "e1b_sidecar_identity",
        e1b_sidecar == E1B_SIDECAR_SHA256,
        e1b_sidecar.to_string(),
    )?;
    let confirmation = authority["confirmation"]
        .as_str()
        .or_else(|| {
            authority["confirmation"]
                .get("status")
                .and_then(Value::as_str)
        })
        .unwrap_or_default();
    ctx.check(
        "confirmation_locked",
        confirmation.contains("LOCKED"),
        confirmation.to_string(),
    )?;
    let scope_start = scope["development"]["start_utc_inclusive"]
        .as_str()
        .unwrap_or_default();
    ctx.check(
        "scope_start_corrected",
        scope_start == SCOPE_DEVELOPMENT_START,
        scope_start.to_string(),
    )?;
    let e1b_view_hash = e1b["provenance"]["development_view_manifest_sha256"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    // ---- 3. Question schema + exposure classification ----
    let universe: Value = read_json(&v1_universe)?;
    let core = universe["tiers"]["CORE_QUESTIONS"]
        .as_array()
        .ok_or("CORE_QUESTIONS missing")?
        .clone();
    ctx.check(
        "core_question_count",
        core.len() == 14,
        format!("{}", core.len()),
    )?;
    for question in &core {
        let id = question["question_id"].as_str().unwrap_or("<unnamed>");
        let missing: Vec<&str> = CORE_QUESTION_FIELDS
            .iter()
            .filter(|field| question.get(**field).is_none())
            .copied()
            .collect();
        ctx.check(
            "question_schema",
            missing.is_empty(),
            format!("{id} missing {missing:?}"),
        )?;
        let exposure = question["exposure_class"].as_str().unwrap_or_default();
        ctx.check(
            "exposure_classification",
            exposure.starts_with("E1")
                || exposure.starts_with("E2_SAME_DOMAIN")
                || exposure.starts_with("E3_CROSS_DOMAIN"),
            format!("{id}: {exposure}"),
        )?;
        let required = serde_json::to_string(
            question["required_fields"]
                .as_array()
                .unwrap_or(&Vec::new()),
        )?;
        let outcome_leak: Vec<&str> = FORBIDDEN_OUTCOME_FIELD_TOKENS
            .iter()
            .filter(|token| required.contains(**token))
            .copied()
            .collect();
        ctx.check(
            "forbidden_fields_absent",
            outcome_leak.is_empty(),
            format!("{id}: {outcome_leak:?}"),
        )?;
    }
    let mut all_records: Vec<Value> = core.clone();
    for family in universe["tiers"]["SYSTEMATIC_DISCOVERY_FAMILY"]
        .as_array()
        .ok_or("SYSTEMATIC_DISCOVERY_FAMILY missing")?
    {
        all_records.push(family.clone());
    }
    for family in universe["tiers"]["OPTIONAL_LOW_PRIORITY"]
        .as_array()
        .ok_or("OPTIONAL_LOW_PRIORITY missing")?
    {
        all_records.push(family.clone());
    }

    // ---- 4. Expansion / multiplicity arithmetic ----
    let accounting = &universe["search_space_accounting"];
    let sf_cells = accounting["systematic_expanded_test_cells"]
        .as_object()
        .ok_or("systematic cells missing")?;
    let sf_declared_total = sf_cells.get("total").and_then(Value::as_u64).unwrap_or(0);
    let sf_sum: u64 = universe["tiers"]["SYSTEMATIC_DISCOVERY_FAMILY"]
        .as_array()
        .ok_or("SF records missing")?
        .iter()
        .filter_map(|family| family["expanded_cell_count"].as_u64())
        .sum();
    ctx.check(
        "systematic_expansion_arithmetic",
        sf_sum == sf_declared_total,
        format!("records {sf_sum} vs accounting {sf_declared_total}"),
    )?;
    let opt_sum: u64 = universe["tiers"]["OPTIONAL_LOW_PRIORITY"]
        .as_array()
        .ok_or("optional records missing")?
        .iter()
        .filter_map(|family| family["expanded_cell_count"].as_u64())
        .sum();
    ctx.check(
        "optional_expansion_arithmetic",
        opt_sum
            == accounting["optional_expanded_test_cells"]
                .get("total")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        format!("records {opt_sum}"),
    )?;
    let core_primary: u64 = core
        .iter()
        .map(|q| core_cells(q["question_id"].as_str().unwrap_or("")))
        .sum();
    ctx.check(
        "core_expansion_arithmetic",
        core_primary == 16,
        core_primary.to_string(),
    )?;

    // ---- 5. Lifecycle vocabulary scan ----
    let lifecycle_hits = scan_vocabulary(&all_records, &FORBIDDEN_LIFECYCLE_TERMS);

    // ---- 6. Cross-scale identity language ----
    let cross_scale_hits = scan_vocabulary(
        &all_records,
        &[
            "same zone across scales",
            "same zone across strata",
            "cross-scale zone identity",
        ],
    );

    // ================= Repairs =================
    // Vocabulary replacement map applied to the V1.1 universe copy.
    let replacements: [(&str, &str); 6] = [
        (
            "invalidation_event",
            "far-edge fill/removal semantics (no separate producer invalidation event)",
        ),
        (
            "INVALIDATED_UNTOUCHED",
            "FORMED_UNTOUCHED (far-edge fill/removal per producer semantics)",
        ),
        (
            "INVALIDATED_WITHOUT_TOUCH",
            "FILLED via gap-through (producer first_touch_observed=false)",
        ),
        (
            "CENSORED_ACTIVE_AT_END_OF_STREAM",
            "RIGHT_CENSORED (observation status, not a market state)",
        ),
        (
            "FILL_ON_FIRST_TOUCH_BAR_INSTANT",
            "TOUCH_AND_FILL_SAME_COMPLETED_BAR",
        ),
        (
            "homogeneous-rate baseline",
            "session-preserving time-varying baseline",
        ),
    ];

    let mut universe_v11 = universe.clone();
    let mut repair_counts = Vec::new();
    {
        let tiers = universe_v11
            .get_mut("tiers")
            .and_then(Value::as_object_mut)
            .ok_or("tiers missing")?;
        for group in tiers.values_mut() {
            if let Some(records) = group.as_array_mut() {
                for record in records.iter_mut() {
                    let replaced = replace_terms_in_record(record, &replacements);
                    if replaced > 0 {
                        repair_counts.push((
                            record["question_id"]
                                .as_str()
                                .or_else(|| record["family_id"].as_str())
                                .unwrap_or("<unnamed>")
                                .to_string(),
                            replaced,
                        ));
                    }
                }
            }
        }
    }

    // CORE-09 null repair.
    if let Some(records) = universe_v11
        .get_mut("tiers")
        .and_then(|tiers| tiers.get_mut("CORE_QUESTIONS"))
        .and_then(Value::as_array_mut)
    {
        for record in records.iter_mut() {
            if record["question_id"] == "QG1-CORE-09" {
                record["appropriate_null_family"] = json!("NULL_SESSION_CONDITIONAL_TIME_VARYING: predeclared time-varying baseline preserving UTC day, session block and native-stratum activity structure while disrupting short-horizon local dependence (e.g. conditional within-day/session event-time randomization, or a predeclared inhomogeneous-rate null). Homogeneous-rate baselines removed: they misread ordinary intraday seasonality as clustering. Naive row shuffle prohibited; a circular shift of the series against itself is not sufficient as the operative null.");
                record["formal_hypothesis"] = json!("Formation-count series exhibit short-horizon autocorrelation at lags 1..16 anchor-stratum bars beyond the session-preserving time-varying null (two-sided: clustering or dispersion).");
                record["v1_1_null_repair"] = json!("CORE-09 null repaired per director ruling section E; question retained unchanged in substance.");
            }
            if matches!(
                record["question_id"].as_str(),
                Some("QG1-CORE-04") | Some("QG1-CORE-05")
            ) {
                record["warm_state_rule"] = json!("WARM_STATE_RULE: do not treat the start of DEVELOPMENT as empty historical state. Spatial/population context includes pre-boundary warm-state zones only where their required current state is lawfully represented; otherwise anchors are marked HISTORY_INCOMPLETE and excluded under a deterministic washout rule until pre-boundary state has cleared. Rolling prior-K history requires the declared K prior lawful completed zones before anchor eligibility. Excluded anchor counts are reported. No synthetic history.");
            }
            if record["question_id"] == "QG1-CORE-10" {
                record["competing_risk_rule"] = json!("COMPETING_RISK_RULE: for formation->first-touch analysis, FIRST_TOUCH is the event of interest; FILL_WITHOUT_RECORDED_TOUCH (gap-through) is a COMPETING TERMINAL EVENT, not non-informative right censoring; END_OF_DEVELOPMENT is right censoring. Use a competing-risk/multi-state estimator or an equivalent deterministic transition accounting. For touched->fill, end-of-development active touched zones are right censored.");
                record["state_model_note"] = json!("Same-bar touch+fill is TOUCH_AND_FILL_SAME_COMPLETED_BAR: one completed bar and one bar-level availability coordinate; no intrabar market-time inference.");
            }
        }
    }
    if let Some(families) = universe_v11
        .get_mut("tiers")
        .and_then(|tiers| tiers.get_mut("SYSTEMATIC_DISCOVERY_FAMILY"))
        .and_then(Value::as_array_mut)
    {
        for family in families.iter_mut() {
            if family["family_id"] == "QG1-SF-4" {
                family["v1_1_status"] = json!("HELD_PENDING_TYPED_CONTEXT_RESOLUTION");
                family["v1_1_accounting_defect"] = json!("DECLARED_UPPER_BOUND_INVALID: the declared 168 = 3 classes x 2 readouts x 2 windows x 2 outcomes x 7 strata omits the member-family dimension that the stated per-member-family execution requires (naive family-specific ceiling 22 x 2 x 2 x 2 x 7 = 1232). Neither 168 nor 1232 is execution accounting; cells are recomputed after typed context resolution and role-appropriate readout mapping.");
            }
            if family["family_id"] == "QG1-SF-5" {
                family["warm_state_rule"] = json!("WARM_STATE_RULE: spatial nesting context includes pre-boundary warm-state zones only where their required current state is lawfully represented; otherwise anchors are marked HISTORY_INCOMPLETE under a deterministic washout rule until pre-boundary state has cleared. Excluded anchor counts reported. No synthetic history.");
            }
            if family["family_id"] == "QG1-SF-3" {
                family["v1_1_status"] = json!("READY_AFTER_E2");
            }
        }
    }
    if let Some(families) = universe_v11
        .get_mut("tiers")
        .and_then(|tiers| tiers.get_mut("OPTIONAL_LOW_PRIORITY"))
        .and_then(Value::as_array_mut)
    {
        for family in families.iter_mut() {
            let id = family["family_id"].as_str().unwrap_or_default();
            match id {
                "QG1-OPT-01" => {
                    family["v1_1_status"] = json!("CONDITIONALLY_LAWFUL_DEFERRED_E3");
                    family["v1_1_conditions"] = json!("Time-point set-level context only: at lawful anchor time t, does the SET of already-active zones at another native scale overlap the anchor interval price-wise. NO cross-scale object identity, NO 'same zone across scales' statement, NO inherited identity. Context zones must be causally available at the anchor. Identity remains (timeframe, zone_id) per scale.");
                }
                "QG1-OPT-03" => {
                    family["v1_1_status"] = json!("PRE_E3_CAUSAL_ELIGIBILITY_AUDIT");
                }
                "QG1-OPT-02" | "QG1-OPT-04" => {
                    family["v1_1_status"] = json!("GRADUATED_E2_ROBUSTNESS_LANE");
                }
                _ => {}
            }
        }
    }
    if let Some(records) = universe_v11
        .get_mut("tiers")
        .and_then(|tiers| tiers.get_mut("CORE_QUESTIONS"))
        .and_then(Value::as_array_mut)
    {
        for record in records.iter_mut() {
            let id = record["question_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let status = match id.as_str() {
                "QG1-CORE-01" | "QG1-CORE-03" | "QG1-CORE-04" | "QG1-CORE-05" | "QG1-CORE-09"
                | "QG1-CORE-10" | "QG1-CORE-12" => "GRADUATED_E2_PRIMARY",
                "QG1-CORE-13" => "GRADUATED_E2_ROBUSTNESS_LANE",
                "QG1-CORE-14" => "PRE_E3_CAUSAL_ELIGIBILITY_AUDIT",
                "QG1-CORE-02" => "READY_AFTER_E2 (SF-3 calendar family)",
                "QG1-CORE-06" | "QG1-CORE-07" | "QG1-CORE-08" | "QG1-CORE-11" => {
                    "HELD_PENDING_TYPED_CONTEXT_RESOLUTION"
                }
                _ => "REVIEW",
            };
            record["v1_1_status"] = json!(status);
        }
    }
    universe_v11["v1_1_lifecycle_state_model"] = json!({
        "market_states": ["FORMED_UNTOUCHED", "TOUCHED_UNFILLED", "FILLED"],
        "lawful_terminal_paths": [
            "FORMED_UNTOUCHED -> TOUCHED_UNFILLED -> FILLED",
            "FORMED_UNTOUCHED -> FILLED (producer first_touch_observed=false; gap-through)",
            "TOUCHED_UNFILLED -> FILLED",
        ],
        "removed_states": ["invalidation_event", "INVALIDATED_UNTOUCHED", "INVALIDATED_WITHOUT_TOUCH", "CENSORED_ACTIVE_AT_END_OF_STREAM"],
        "observation_status": ["OBSERVED_TERMINAL", "RIGHT_CENSORED_UNTOUCHED", "RIGHT_CENSORED_TOUCHED"],
        "observation_status_note": "Research observation status is separate from market state; censoring is not a detector lifecycle state.",
        "same_bar_term": "TOUCH_AND_FILL_SAME_COMPLETED_BAR (one completed bar, one bar-level availability coordinate; no intrabar market-time inference)",
    });
    universe_v11["v1_1_competing_risk_rule"] = json!("For formation->first-touch: FIRST_TOUCH = event of interest; FILL_WITHOUT_RECORDED_TOUCH = competing terminal event; END_OF_DEVELOPMENT = right censor. Gap-through fills are never censored as non-informative. Touched->fill: end-of-development active touched zones are right censored. Competing-risk/multi-state estimator or equivalent deterministic transition accounting required.");
    universe_v11["v1_1_repair_summary"] = json!({
        "vocabulary_replacements": repair_counts,
        "lifecycle_hits_in_v1": lifecycle_hits,
        "cross_scale_language_hits_in_v1": cross_scale_hits,
    });

    // ---- V1.1 question registry ----
    let v1_registry: Value = read_json(&v1_registry_path)?;
    let mut registry_v11 = v1_registry.clone();
    registry_v11["artifact_type"] = json!("AP-002_QUESTION_REGISTRY_V1_1");
    registry_v11["v1_status"] = json!("SEALED_BLIND_SOURCE_SUPERSEDED_FOR_EXECUTION");
    registry_v11["v1_status_reason"] = json!("Authority/methodology review found execution-contract defects (lifecycle vocabulary, censoring semantics, CORE-09 null, SF-4 accounting, warm-state rules); the blindness result and the sealed V1 intellectual source remain valid.");
    if let Some(records) = registry_v11
        .get_mut("records")
        .and_then(Value::as_array_mut)
    {
        for record in records.iter_mut() {
            let id = record["registry_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let status = match id.as_str() {
                "DIR-01" | "PD-A" | "PD-B" | "PD-C" => {
                    "GRADUATED_E2_PRIMARY (stage-matched lifecycle panel instrument)"
                }
                "PD-D" => "GRADUATED_E2_PRIMARY (direction-asymmetry matched-null family)",
                "PD-E" | "PD-G" => {
                    "GRADUATED_E2_PRIMARY (merged into QG1-CORE-10 completion-mode instrument)"
                }
                "PD-F" => {
                    "GRADUATED_E2_PRIMARY (merged into QG1-CORE-01/SF-2 attribute instrument)"
                }
                "BQ-QG1-CORE-13" => "GRADUATED_E2_ROBUSTNESS_LANE",
                "BQ-QG1-CORE-06" | "BQ-QG1-CORE-07" | "BQ-QG1-CORE-08" | "BQ-QG1-CORE-11" => {
                    "HELD_PENDING_TYPED_CONTEXT_RESOLUTION"
                }
                "BQ-QG1-CORE-14" => "PRE_E3_CAUSAL_ELIGIBILITY_AUDIT",
                _ => "GRADUATED_E2_PRIMARY",
            };
            record["v1_1_execution_status"] = json!(status);
        }
    }
    registry_v11["v1_1_multiplicity"] = json!({
        "e2_primary_cells": {
            "blind_graduated_core": 9,
            "stage_matched_panel": {"cells": 42, "expansion": "2 matched stage contrasts (formation->first-touch, pre-fill->fill) x 7 strata x 3 horizons", "instruments": ["DIR-01", "PD-A", "PD-B", "PD-C"]},
            "direction_asymmetry_null_family": {"cells": 84, "expansion": "2 directions x 2 stage anchors x 3 horizons x 7 strata", "instruments": ["PD-D"]},
            "total": 135,
        },
        "e2_systematic_cells": {"SF-1": 98, "SF-2": 126, "SF-5": 63, "total": 287},
        "e2_robustness_cells": {"CORE-13": 1, "OPT-02": 21, "OPT-04": 28, "total": 50},
        "e2_total_cells": 472,
        "e3_ready_after_e2_cells": {"SF-3": 126},
        "e3_pre_eligibility_audit_cells": {"CORE-14": 1, "OPT-03": 28, "total": 29},
        "e3_deferred_cells": {"OPT-01": 84, "status": "CONDITIONALLY_LAWFUL_DEFERRED_E3"},
        "e3_held": {"SF-4": "v1 168 declared invalid; naive 1232 not adopted; recomputed after typed context resolution", "CORE-06/07/08/11": "held, cells not counted"},
        "v1_accounting_note": "V1's 346/412/758 accounting remains historical QG-V1 bookkeeping, not execution accounting.",
    });

    // ---- Repair ledger ----
    let repair_ledger = json!({
        "artifact_type": "AP-002_QG_REPAIR_LEDGER",
        "R1_rust_only_freeze": "This Rust tool (ap002_qg_freeze) is the canonical producer of the V1.1 artifacts; the V1 blind artifacts are sealed inputs. V1 status: SEALED_BLIND_SOURCE_SUPERSEDED_FOR_EXECUTION.",
        "R2_lifecycle_vocabulary": {"removed_states": ["invalidation_event", "INVALIDATED_UNTOUCHED", "INVALIDATED_WITHOUT_TOUCH"], "canonical_market_states": ["FORMED_UNTOUCHED", "TOUCHED_UNFILLED", "FILLED"], "canonical_paths": ["FORMED_UNTOUCHED->TOUCHED_UNFILLED->FILLED", "FORMED_UNTOUCHED->FILLED (first_touch_observed=false)", "TOUCHED_UNFILLED->FILLED"], "v1_hits": lifecycle_hits.iter().map(|(id, term, _)| format!("{id}: {term}")).collect::<Vec<_>>()},
        "R3_observation_status": {"separated_from_market_state": true, "statuses": ["OBSERVED_TERMINAL", "RIGHT_CENSORED_UNTOUCHED", "RIGHT_CENSORED_TOUCHED"]},
        "R4_competing_risk": "Gap-through fill is a competing terminal event for formation->first-touch, not non-informative censoring. Applied to CORE-10 and carried into the E2 contract as a global duration-model rule.",
        "R5_same_bar_rename": "FILL_ON_FIRST_TOUCH_BAR_INSTANT -> TOUCH_AND_FILL_SAME_COMPLETED_BAR; no intrabar timing inference.",
        "R6_core09_null": "Homogeneous-rate baseline removed; session-preserving time-varying null (UTC day + session block + stratum activity preserved; short-range dependence disrupted) is the operative null; naive shuffle prohibited; self-circular shift insufficient alone. Question retained.",
        "R7_sf4_accounting": "SF-4 v1 168-cell upper bound invalid (omits member-family dimension; naive per-family ceiling 1232). Classification: DECLARED_UPPER_BOUND_INVALID. Both numbers are historical QG-V1 accounting; recomputation after typed context resolution.",
        "R8_sf4_readout_model": "Generic class x readout model replaced by family -> frozen semantic role -> allowed typed readout form -> exact expansion. See AP-002_E3_CONTEXT_RESOLUTION_REQUIREMENTS.json; all 22 families currently NOT_EXECUTABLE (registry semantic_status UNRESOLVED).",
        "R9_warm_state": "CORE-04/CORE-05/SF-5 carry the warm-state rule: pre-boundary warm-state zones included only where lawfully represented; otherwise HISTORY_INCOMPLETE anchors excluded under a deterministic washout rule; rolling history requires K prior lawful completions; excluded counts reported; no synthetic history.",
        "R10_active_data_identity": {"development_start": SCOPE_DEVELOPMENT_START, "e1a_logical_result_hash": E1A_LOGICAL_HASH, "e1b_logical_result_hash": E1B_LOGICAL_HASH, "e1b_sidecar_sha256": E1B_SIDECAR_SHA256, "e1b_development_view_manifest_sha256": e1b_view_hash, "rule": "Authority V1 supplies semantics only; execution binds to the corrected 16:15 scope and the scope-repair sidecar used by active E1A/E1B."},
        "vocabulary_replacement_counts": repair_counts,
    });

    // ---- E2 contract ----
    let e2_contract = json!({
        "artifact_type": "AP-002_E2_CONTRACT_V1",
        "contract_version": 1,
        "program": "AP-002 FVG",
        "exposure_class": "E2_SAME_DOMAIN",
        "confirmation_status": "LOCKED",
        "data_identity": {
            "development_start_utc_inclusive": SCOPE_DEVELOPMENT_START,
            "development_end_utc_exclusive": scope["development"]["end_utc_exclusive"],
            "e1a_logical_result_hash": E1A_LOGICAL_HASH,
            "e1b_logical_result_hash": E1B_LOGICAL_HASH,
            "sidecar_sha256": E1B_SIDECAR_SHA256,
            "development_view_manifest_sha256": e1b_view_hash,
            "binding_rule": "Execution binds to the corrected 16:15 DEVELOPMENT scope and the scope-repair availability sidecar used by active E1A/E1B. Authority V1 supplies semantics only.",
        },
        "native_strata": ["15s", "30s", "1m", "5m", "15m", "1h", "4h"],
        "cross_scale_identity": "PROHIBITED",
        "lifecycle_state_model": universe_v11["v1_1_lifecycle_state_model"],
        "competing_risk_rule": universe_v11["v1_1_competing_risk_rule"],
        "terminology_rules": [
            "STAGE-CONDITIONED INFORMATIONAL RELATIONSHIP (or STAGE-ASSOCIATED CONDITIONAL CONTRAST); never 'causal stage effect'.",
            "Decision rule wording: SURVIVES_STAGE_CONDITIONING iff within-zone stage contrast survives AND matched temporal/drift-aware null survives AND selection diagnostics survive.",
            "TOUCH_AND_FILL_SAME_COMPLETED_BAR; no instant/zero-tick implication.",
            "PD-D temporal blocks used for null/control construction do not themselves become a temporal-context finding.",
        ],
        "lanes": {
            "behavioral_primary": "Graduated primary instruments below; multiplicity families frozen before execution.",
            "methods_robustness": "CORE-13, OPT-02, OPT-04: censoring-estimator and normalization robustness. Method robustness is reported separately from behavioral inference and never mixed into behavioral multiplicity.",
        },
        "graduated_instruments": [
            {"instrument_id": "E2-PANEL", "registry_ids": ["DIR-01", "PD-A", "PD-B", "PD-C"], "dedup": "One merged stage-matched lifecycle panel instrument", "cells": 42, "multiplicity_family": "MF_STAGE_MATCHED_PANEL = 2 matched stage contrasts (formation->first-touch, pre-fill->fill) x 7 strata x 3 horizons", "content": "Within-zone matched formation-vs-first-touch and pre-fill-vs-fill contrasts at identical calendar offsets; censor-aware selection model of touched vs never-touched and filled vs touched-unfilled; time-matched no-FVG drift-aware null; pre-registered decision rule SURVIVES_STAGE_CONDITIONING."},
            {"instrument_id": "E2-DIRNULL", "registry_ids": ["PD-D"], "cells": 84, "multiplicity_family": "MF_DIRECTION_ASYMMETRY_NULL = 2 directions x 2 stage anchors x 3 horizons x 7 strata", "content": "Direction asymmetry under direction-matched, time-matched and drift-aware temporal-block nulls."},
            {"instrument_id": "E2-CORE-01", "registry_ids": ["QG1-CORE-01", "PD-F", "SF-2-attribute-aspect"], "cells": 1, "content": "Formation-time magnitude attributes condition later-stage lifecycle outcomes; attribute x horizon sweep carried by SF-2."},
            {"instrument_id": "E2-CORE-03", "registry_ids": ["QG1-CORE-03"], "cells": 1, "content": "Within-zone stage-history conditioning (untouched vs already-touched-not-filled; stage age)."},
            {"instrument_id": "E2-CORE-04", "registry_ids": ["QG1-CORE-04"], "cells": 1, "content": "Same-stratum crowding / spatial configuration at formation.", "warm_state_rule": true},
            {"instrument_id": "E2-CORE-05", "registry_ids": ["QG1-CORE-05"], "cells": 1, "content": "Same-domain population memory (rolling prior-K completion outcomes).", "warm_state_rule": true},
            {"instrument_id": "E2-CORE-09", "registry_ids": ["QG1-CORE-09"], "cells": 1, "content": "Formation clustering beyond the repaired session-preserving time-varying null.", "null_repair": true},
            {"instrument_id": "E2-CORE-10", "registry_ids": ["QG1-CORE-10", "PD-E", "PD-G"], "dedup": "PD-E and PD-G merged here", "cells": 1, "content": "Completion-mode structure: gap-through vs touched-fill and same-completed-bar resolution as competing modes, with competing-risk accounting."},
            {"instrument_id": "E2-CORE-12", "registry_ids": ["QG1-CORE-12"], "cells": 3, "content": "Cross-stratum sign/gradient replication WITHOUT object identity (magnitude, stage-age, crowding meta-tests)."},
            {"instrument_id": "E2-SF-1", "registry_ids": ["SF-1"], "cells": 98, "content": "Stage-conditioned transition family (stage x horizon x stratum)."},
            {"instrument_id": "E2-SF-2", "registry_ids": ["SF-2"], "cells": 126, "content": "Formation-attribute x horizon family."},
            {"instrument_id": "E2-SF-5", "registry_ids": ["SF-5"], "cells": 63, "content": "Same-stratum spatial/nesting family.", "warm_state_rule": true},
        ],
        "robustness_instruments": [
            {"instrument_id": "E2-CORE-13", "registry_ids": ["QG1-CORE-13"], "cells": 1, "content": "Duration/transition conclusions robust to censoring treatment (censor-aware survival estimators vs deterministic transition accounting)."},
            {"instrument_id": "E2-OPT-02", "registry_ids": ["OPT-02"], "cells": 21, "content": "Normalization robustness."},
            {"instrument_id": "E2-OPT-04", "registry_ids": ["OPT-04"], "cells": 28, "content": "Censoring estimator robustness family."},
        ],
        "multiplicity": registry_v11["v1_1_multiplicity"],
        "execution_rules": [
            "One Rust research pass materializes the reusable same-domain features once and evaluates the entire frozen E2 universe from them.",
            "No outcome-based pruning; the graduated universe is executed as frozen.",
            "Behavioral and robustness lanes never share multiplicity accounting.",
            "Report all excluded HISTORY_INCOMPLETE anchor counts under the warm-state rule.",
            "No E3 execution is authorized by this contract.",
        ],
    });

    // ---- E3 context resolution requirements ----
    let detectors: Vec<Value> = serde_json::from_reader(File::open(&detector_registry_path)?)?;
    let mut context_requirements = Vec::new();
    for detector in &detectors {
        let id = detector["detector_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        if id == "fvg" {
            continue;
        }
        let proposed = PROPOSED_ROLES.iter().find(|(name, _, _)| name == &id);
        let role = proposed
            .map(|(_, role, _)| (*role).to_string())
            .unwrap_or_else(|| {
                if detector["domain"] == "tick" {
                    "PROPOSED_TICK_MICROSTRUCTURE_CONTEXT".to_string()
                } else {
                    "PROPOSED_BAR_CONTEXT".to_string()
                }
            });
        let readout_note = proposed
            .map(|(_, _, note)| (*note).to_string())
            .unwrap_or_default();
        let semantic_status = detector["semantic_status"].as_str().unwrap_or("UNRESOLVED");
        let allowed = if semantic_status == "UNRESOLVED" {
            json!([])
        } else {
            json!(["role-appropriate typed readouts per role map"])
        };
        context_requirements.push(json!({
            "detector_family": id,
            "proposed_role": role,
            "readout_note": readout_note,
            "semantic_status": semantic_status,
            "allowed_readout_forms": allowed,
            "forbidden_readout_forms": if id == "feed_health" { json!(["directional explanatory evidence", "any behavioral conditioning role"]) } else { json!(["generic event-presence readouts where the role map rejects them", "readouts implying cross-scale object identity"]) },
            "native_scale_domain": detector["domain"],
            "availability_rule": detector["availability_semantics"].as_str().unwrap_or("authoritative manifest surface; availability semantics unresolved"),
            "dependency_caveat": detector["derives_from"],
            "execution_eligibility": if semantic_status == "UNRESOLVED" { "NOT_EXECUTABLE" } else { "EXECUTABLE_AFTER_TYPED_RESOLUTION" },
        }));
    }
    let e3_requirements = json!({
        "artifact_type": "AP-002_E3_CONTEXT_RESOLUTION_REQUIREMENTS",
        "rule": "detector family -> frozen semantic role -> allowed typed readout form -> exact expansion. Unresolved family-role pairs remain NOT_EXECUTABLE.",
        "families": context_requirements,
        "executable_now": 0,
        "held_unresolved": NON_FVG_FAMILIES,
        "note": "Proposed roles are the director's role map plus domain defaults, pending authority confirmation. feed_health is DATA_QUALITY/validity-only.",
    });

    // ---- Manifest ----
    let manifest = json!({
        "artifact_type": "AP-002_QG_V1_1_MANIFEST",
        "v1_status": "SEALED_BLIND_SOURCE_SUPERSEDED_FOR_EXECUTION",
        "v1_status_reason": "Authority/methodology review found execution-contract defects; the blindness result and sealed V1 source remain valid.",
        "inputs": {
            "sealed_v1_universe_sha256": universe_bytes_hash,
            "authority": authority_path.to_string_lossy(),
            "scope": scope_path.to_string_lossy(),
            "e1a_results": e1a_results.to_string_lossy(),
            "e1b_results": e1b_results.to_string_lossy(),
            "detector_registry": detector_registry_path.to_string_lossy(),
        },
        "validation_checks": ctx.checks.iter().map(|(name, ok, evidence)| json!({"check": name, "pass": ok, "evidence": evidence})).collect::<Vec<_>>(),
        "generator": "ap002_qg_freeze (Rust)",
        "e2_universe": registry_v11["v1_1_multiplicity"],
        "e3_context_families": {"executable_now": 0, "held_unresolved": NON_FVG_FAMILIES},
        "opt01_status": "CONDITIONALLY_LAWFUL_DEFERRED_E3",
        "confirmation_status": "LOCKED",
        "e2_e3_executed": false,
    });

    let out = |name: &str| output_dir.join(name);
    write_pretty(&out("AP-002_BLIND_QG_UNIVERSE_V1_1.json"), &universe_v11)?;
    write_pretty(&out("AP-002_QUESTION_REGISTRY_V1_1.json"), &registry_v11)?;
    write_pretty(&out("AP-002_QG_REPAIR_LEDGER.json"), &repair_ledger)?;
    write_pretty(&out("AP-002_E2_CONTRACT_V1.json"), &e2_contract)?;
    write_pretty(
        &out("AP-002_E3_CONTEXT_RESOLUTION_REQUIREMENTS.json"),
        &e3_requirements,
    )?;
    write_pretty(&out("AP-002_QG_V1_1_MANIFEST.json"), &manifest)?;

    // Deterministic MD rendering of the E2 contract.
    let mut md = String::new();
    md.push_str("# AP-002 E2 Contract V1\n\nFrozen by the Rust `ap002_qg_freeze` tool. Machine record: `AP-002_E2_CONTRACT_V1.json`.\n\n");
    md.push_str("## Data identity\n\nExecution binds to the corrected 16:15 DEVELOPMENT scope and the scope-repair sidecar used by active E1A/E1B. Authority V1 supplies semantics only.\n\n");
    md.push_str("## Lifecycle state model\n\nMarket states: FORMED_UNTOUCHED, TOUCHED_UNFILLED, FILLED. Lawful terminal paths: FORMED_UNTOUCHED -> TOUCHED_UNFILLED -> FILLED; FORMED_UNTOUCHED -> FILLED (gap-through, producer first_touch_observed=false); TOUCHED_UNFILLED -> FILLED. Observation status is separate: OBSERVED_TERMINAL, RIGHT_CENSORED_UNTOUCHED, RIGHT_CENSORED_TOUCHED. Same-bar resolution is TOUCH_AND_FILL_SAME_COMPLETED_BAR (no intrabar inference). No separate producer invalidation event exists.\n\n");
    md.push_str("## Competing-risk rule\n\nFor formation->first-touch: FIRST_TOUCH is the event of interest; FILL_WITHOUT_RECORDED_TOUCH is a competing terminal event (never non-informative censoring); END_OF_DEVELOPMENT is right censoring. Touched->fill: end-of-development active touched zones are right censored.\n\n");
    md.push_str("## Graduated primary instruments\n\n| Instrument | Registry ids | Cells | Multiplicity family |\n| --- | --- | --- | --- |\n");
    if let Some(instruments) = e2_contract["graduated_instruments"].as_array() {
        for instrument in instruments {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                instrument["instrument_id"].as_str().unwrap_or(""),
                serde_json::to_string(instrument["registry_ids"].as_array().unwrap_or(&Vec::new()))
                    .unwrap_or_default(),
                instrument["cells"].as_u64().unwrap_or(0),
                instrument["multiplicity_family"]
                    .as_str()
                    .unwrap_or("declared per instrument record"),
            ));
        }
    }
    md.push_str("\n## Robustness lane\n\nCORE-13 (1 cell), OPT-02 (21), OPT-04 (28): method robustness reported separately from behavioral inference.\n\n");
    md.push_str("## Multiplicity\n\nE2 primary 135 (blind core 9 + stage-matched panel 42 + direction-asymmetry null family 84); systematic 287 (SF-1 98, SF-2 126, SF-5 63); robustness 50; total 472. V1's 346/412/758 accounting is historical QG-V1 bookkeeping only.\n\n");
    md.push_str("## Wording rules\n\nSTAGE-CONDITIONED INFORMATIONAL RELATIONSHIP, never 'causal stage effect'. Decision rule: SURVIVES_STAGE_CONDITIONING iff within-zone stage contrast survives AND matched temporal/drift-aware null survives AND selection diagnostics survive. PD-D temporal blocks used for null construction do not become a temporal-context finding.\n\n");
    md.push_str("## Execution rules\n\nOne Rust research pass materializes the reusable same-domain features once and evaluates the entire frozen E2 universe from them. No outcome-based pruning. Behavioral and robustness lanes never share multiplicity accounting. HISTORY_INCOMPLETE anchor counts reported under the warm-state rule. No E3 execution authorized by this contract. Confirmation LOCKED.\n");
    std::fs::write(out("AP-002_E2_CONTRACT_V1.md"), md)?;

    println!(
        "QG V1.1 freeze complete: {} checks passed | lifecycle hits repaired: {} | cross-scale hits: {} | E2 cells 472 (primary 135 / systematic 287 / robustness 50) | E3 executable now: 0, held: {NON_FVG_FAMILIES}",
        ctx.checks.len(),
        lifecycle_hits.len(),
        cross_scale_hits.len(),
    );
    Ok(())
}
