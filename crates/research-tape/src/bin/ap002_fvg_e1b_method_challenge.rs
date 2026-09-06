//! AP-002 E1B scanner integrity gate.  Independent structural verification of
//! the stage-anchor table and aggregate reconciliation against the E1B results.
//! Scientific interpretation: false.  Confirmation: LOCKED.

use research_tape::fvg_e1::Direction;
use research_tape::fvg_e1b::{Stage, StageOutcome};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process,
};

#[derive(Debug, Clone, Deserialize)]
struct AnchorRow {
    stage: Stage,
    identity: String,
    direction: Direction,
    anchor_bar_close_ts: i64,
    available_time_ns: i64,
    source_sequence: i64,
    anchor_bid: f64,
    anchor_ask: f64,
    anchor_mid: f64,
    first_touch_observed: Option<bool>,
    formation_ts: i64,
    formation_available_time_ns: i64,
    formation_to_anchor_market_ms: i64,
    formation_to_anchor_known_ms: i64,
    outcomes: [Option<StageOutcome>; 3],
}

#[derive(Debug, Serialize)]
struct Check {
    id: &'static str,
    status: &'static str,
    evidence: String,
}

fn close_enough(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9 * left.abs().max(right.abs()).max(1.0)
}

fn validate_anchor(row: &AnchorRow) -> Result<(), String> {
    if !row.anchor_bid.is_finite()
        || !row.anchor_ask.is_finite()
        || row.anchor_bid <= 0.0
        || row.anchor_bid > row.anchor_ask
        || !close_enough(row.anchor_mid, (row.anchor_bid + row.anchor_ask) / 2.0)
    {
        return Err("anchor price is inconsistent with the boundary tick".into());
    }
    if row.anchor_bar_close_ts <= row.formation_ts
        || row.formation_to_anchor_market_ms
            != row.anchor_bar_close_ts.saturating_sub(row.formation_ts)
        || row.formation_to_anchor_known_ms
            != row
                .available_time_ns
                .saturating_sub(row.formation_available_time_ns)
                / 1_000_000
    {
        return Err("stage anchor is not after formation or durations are inconsistent".into());
    }
    match (row.stage, row.first_touch_observed) {
        (Stage::FirstTouch, None) => {}
        (Stage::Fill, Some(_)) => {}
        _ => return Err("stage and gap-through indicator disagree".into()),
    }
    for outcome in row.outcomes.iter().flatten() {
        if outcome.outcome_bar_close_ts <= row.anchor_bar_close_ts
            || (
                outcome.outcome_available_time_ns,
                outcome.outcome_source_sequence,
            ) <= (row.available_time_ns, row.source_sequence)
        {
            return Err("stage outcome is not causally after the anchor".into());
        }
        let raw = 10_000.0 * outcome.raw_price_delta / row.anchor_mid;
        let adjusted = match row.direction {
            Direction::Bullish => raw,
            Direction::Bearish => -raw,
        };
        if !close_enough(
            outcome.raw_price_delta,
            outcome.outcome_close - row.anchor_mid,
        ) || !close_enough(outcome.raw_return_bps, raw)
            || !close_enough(outcome.direction_adjusted_return_bps, adjusted)
        {
            return Err("stage outcome arithmetic is inconsistent".into());
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut results = PathBuf::new();
    let mut anchors = PathBuf::new();
    let mut preflight = PathBuf::new();
    let mut output = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--results" => results = args.next().ok_or("missing --results")?.into(),
            "--anchors" => anchors = args.next().ok_or("missing --anchors")?.into(),
            "--preflight" => preflight = args.next().ok_or("missing --preflight")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if results.as_os_str().is_empty()
        || anchors.as_os_str().is_empty()
        || preflight.as_os_str().is_empty()
        || output.as_os_str().is_empty()
    {
        return Err("--results, --anchors, --preflight and --output are required".into());
    }
    let results_value: serde_json::Value = serde_json::from_reader(File::open(&results)?)?;
    if results_value["exposure_class"] != "E1_PHENOTYPE"
        || results_value["confirmation_status"] != "LOCKED"
        || results_value.get("e2").is_some()
        || results_value.get("e3").is_some()
    {
        return Err("results are outside the locked E1 scope".into());
    }
    let preflight_value: research_tape::fvg_e1::PreflightResult =
        serde_json::from_reader(File::open(&preflight)?)?;
    let expected_n = results_value["tables"]["anchors"]["n"]
        .as_u64()
        .ok_or("results anchor count missing")?;
    let expected_hash = results_value["tables"]["anchors"]["logical_rows_sha256"]
        .as_str()
        .ok_or("results logical anchor hash missing")?;
    let mut reader = BufReader::new(File::open(&anchors)?);
    let mut line = String::new();
    let mut unique = BTreeMap::new();
    let mut count = 0_u64;
    let mut logical = Sha256::new();
    let mut first_touch_n = 0_u64;
    let mut fill_n = 0_u64;
    let mut gap_through_n = 0_u64;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)?;
        let row: AnchorRow = serde_json::from_value(value.clone())?;
        validate_anchor(&row).map_err(|error| format!("anchor {count}: {error}"))?;
        let key = (row.stage, row.identity.clone());
        if unique.insert(key, count).is_some() {
            return Err(format!("duplicate stage anchor: {}", row.identity).into());
        }
        logical.update(line.as_bytes());
        count += 1;
        match row.stage {
            Stage::FirstTouch => first_touch_n += 1,
            Stage::Fill => {
                fill_n += 1;
                if row.first_touch_observed == Some(false) {
                    gap_through_n += 1;
                }
            }
        }
    }
    let logical_hash = format!("{:x}", logical.finalize());
    if count != expected_n || logical_hash != expected_hash {
        return Err(format!(
            "anchor table mismatch: rows {count}/{expected_n}, hash {logical_hash}/{expected_hash}"
        )
        .into());
    }
    // Aggregate reconciliation against the results tables.
    let strata = results_value["tables"]["strata"]["rows"]
        .as_array()
        .ok_or("stratum rows missing")?;
    for stratum in strata {
        let timeframe = stratum["timeframe"].as_str().ok_or("timeframe")?;
        let prefix = format!("{timeframe}:");
        let stratum_touch = unique
            .keys()
            .filter(|(stage, identity)| {
                *stage == Stage::FirstTouch && identity.starts_with(&prefix)
            })
            .count() as u64;
        let stratum_fill = unique
            .keys()
            .filter(|(stage, identity)| *stage == Stage::Fill && identity.starts_with(&prefix))
            .count() as u64;
        if stratum_touch != stratum["touch_anchors_n"].as_u64().unwrap_or(u64::MAX)
            || stratum_fill != stratum["fill_anchors_n"].as_u64().unwrap_or(u64::MAX)
        {
            return Err(format!("aggregate anchor mismatch for {timeframe}").into());
        }
    }
    let mut checks = vec![
        Check {
            id: "native_stage_identity",
            status: "PASS",
            evidence: format!("{count} unique (stage, timeframe, zone_id) anchors"),
        },
        Check {
            id: "anchor_price_causality",
            status: "PASS",
            evidence:
                "anchor price equals the boundary-crossing tick mid; outcomes strictly post-anchor"
                    .into(),
        },
        Check {
            id: "stage_stratification",
            status: "PASS",
            evidence: format!(
                "FIRST_TOUCH {first_touch_n}, FILL {fill_n}, gap-through fills {gap_through_n}"
            ),
        },
        Check {
            id: "aggregate_reconciliation",
            status: "PASS",
            evidence: "per-stratum anchor counts match the results tables".into(),
        },
    ];
    let preflight_status = if preflight_value.regressions_count == 0 {
        "PASS"
    } else {
        "FAIL"
    };
    checks.push(Check {
        id: "receipt_regression_preflight",
        status: preflight_status,
        evidence: format!("{} regressions", preflight_value.regressions_count),
    });
    if preflight_status == "FAIL" {
        return Err("integrity gate rejects nonzero receipt regressions".into());
    }
    let package = json!({
        "gate_type": "E1B_SCANNER_INTEGRITY_GATE",
        "status": "PASS",
        "scientific_interpretation": false,
        "confirmation_status": "LOCKED",
        "anchors_n": count,
        "logical_rows_sha256": logical_hash,
        "first_touch_anchors_n": first_touch_n,
        "fill_anchors_n": fill_n,
        "gap_through_fill_n": gap_through_n,
        "checks": checks,
    });
    std::fs::write(&output, serde_json::to_vec_pretty(&package)?)?;
    println!("{}", serde_json::to_string_pretty(&package)?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1B scanner integrity gate failed: {error}");
        process::exit(1);
    }
}
