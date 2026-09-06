//! AP-002 E1A scope-repair audit.  Recomputes the A1 (fill without prior
//! recorded touch) and A2 (orphan lifecycle events) evidence against the
//! corrected development cohort, verifies every orphan zone was detected
//! before its stratum's first lawful in-window formation, and records the
//! aggregate delta against the superseded (scope-misaligned) E1 result.
//!
//! Uses the same canonical payload parser as the scanner; fail-closed if the
//! extracted orphan counts do not reconcile with the scanner's counters.

use arrow_array::Array;
use research_tape::fvg_e1::{parse_payload, ZoneLifecycle};
use research_tape::{batch_i64, batch_large_string, required_i64, ProjectedParquetReader};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process,
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];

#[derive(Debug, Clone, Serialize)]
struct OrphanEvent {
    kind: &'static str,
    zone_id: u64,
    detection_ts: i64,
    event_ts: i64,
    event_bar_close_ts: i64,
    first_touch_observed: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
struct StratumAudit {
    formed_n: u64,
    formed_fill_without_prior_touch_n: u64,
    orphan_touch_n: u64,
    orphan_fill_n: u64,
    first_formed_detection_ts: Option<i64>,
    orphan_detection_max: Option<i64>,
    all_orphans_left_truncated: bool,
    orphan_events: Vec<OrphanEvent>,
}

#[derive(Debug, Default, Serialize)]
struct ResultsDelta {
    timeframe: String,
    formations_old: u64,
    formations_new: u64,
    formed_fill_without_prior_touch_old: u64,
    formed_fill_without_prior_touch_new: u64,
    orphan_touch_old: u64,
    orphan_touch_new: u64,
    orphan_fill_old: u64,
    orphan_fill_new: u64,
    fill_rate_old: Option<f64>,
    fill_rate_new: Option<f64>,
    bullish_share_old: Option<f64>,
    bullish_share_new: Option<f64>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1A scope-repair audit failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();
    let mut view_root = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut zones = PathBuf::new();
    let mut results_new = PathBuf::new();
    let mut results_old = PathBuf::new();
    let mut output = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--view-root" => view_root = args.next().ok_or("missing --view-root")?.into(),
            "--zones" => zones = args.next().ok_or("missing --zones")?.into(),
            "--results-new" => results_new = args.next().ok_or("missing --results-new")?.into(),
            "--results-old" => results_old = args.next().ok_or("missing --results-old")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    for (name, path) in [
        ("--zones", &zones),
        ("--results-new", &results_new),
        ("--results-old", &results_old),
        ("--output", &output),
    ] {
        if path.as_os_str().is_empty() {
            return Err(format!("{name} is required").into());
        }
    }

    // Formed identity set + A1 counts from the corrected zone table.
    let mut formed: HashSet<(String, u64)> = HashSet::new();
    let mut first_formed_detection: BTreeMap<String, i64> = BTreeMap::new();
    let mut a1: BTreeMap<String, u64> = BTreeMap::new();
    let mut zone_rows = 0u64;
    let reader = BufReader::new(File::open(&zones)?);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let zone: ZoneLifecycle = serde_json::from_str(&line)?;
        zone_rows += 1;
        formed.insert((zone.timeframe.clone(), zone.formation.zone_id));
        let entry = first_formed_detection
            .entry(zone.timeframe.clone())
            .or_insert(zone.formation.detection_ts);
        *entry = (*entry).min(zone.formation.detection_ts);
        if zone.fill_ts.is_some() && zone.first_touch_ts.is_none() {
            *a1.entry(zone.timeframe.clone()).or_default() += 1;
        }
    }

    // Orphan events straight from the corrected view payload stream, using the
    // canonical parser (its deserialize_events already accepts the producer's
    // one-versus-many serialization).
    let manifest: serde_json::Value =
        serde_json::from_reader(File::open(view_root.join("view_manifest.json"))?)?;
    let mut audit: BTreeMap<String, StratumAudit> = BTreeMap::new();
    for timeframe in TIMEFRAMES {
        audit.entry(timeframe.to_string()).or_default();
    }
    let mut payload_rows = 0u64;
    let mut malformed_payloads = 0u64;
    for timeframe in TIMEFRAMES {
        let parts = manifest["source_groups"][timeframe]
            .as_array()
            .ok_or_else(|| format!("missing source group: {timeframe}"))?;
        for part in parts {
            let logical = part["logical_path"]
                .as_str()
                .ok_or("part missing logical_path")?;
            let path = view_root.join(logical);
            let reader = ProjectedParquetReader::new(
                path,
                ["bar_close_ts", "payload_fvg"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                8192,
            )?;
            let mut scan = reader.scan()?;
            while let Some(batch) = scan.next() {
                let batch = batch?;
                let bar_close = batch_i64(&batch, "bar_close_ts")?;
                let payload = batch_large_string(&batch, "payload_fvg")?;
                for index in 0..batch.num_rows() {
                    let bar_close_ts = required_i64(bar_close, index, "bar_close_ts")?;
                    let raw = if payload.is_null(index) {
                        continue;
                    } else {
                        payload.value(index)
                    };
                    if raw.is_empty() || raw == "{}" {
                        continue;
                    }
                    payload_rows += 1;
                    let parsed = match parse_payload(raw) {
                        Ok(parsed) => parsed,
                        Err(_) => {
                            malformed_payloads += 1;
                            continue;
                        }
                    };
                    let stratum = audit.get_mut(timeframe).ok_or("stratum vanished")?;
                    for event in &parsed.fvg_first_touch {
                        if formed.contains(&(timeframe.to_string(), event.zone_id)) {
                            continue;
                        }
                        stratum.orphan_events.push(OrphanEvent {
                            kind: "touch",
                            zone_id: event.zone_id,
                            detection_ts: event.detection_ts,
                            event_ts: event.touch_ts,
                            event_bar_close_ts: bar_close_ts,
                            first_touch_observed: None,
                        });
                        stratum.orphan_touch_n += 1;
                    }
                    for event in &parsed.fvg_filled {
                        if formed.contains(&(timeframe.to_string(), event.zone_id)) {
                            continue;
                        }
                        stratum.orphan_events.push(OrphanEvent {
                            kind: "fill",
                            zone_id: event.zone_id,
                            detection_ts: event.detection_ts,
                            event_ts: event.fill_ts,
                            event_bar_close_ts: bar_close_ts,
                            first_touch_observed: Some(event.first_touch_observed),
                        });
                        stratum.orphan_fill_n += 1;
                    }
                }
            }
        }
    }

    // Left-truncation check per stratum: every orphan zone must have been
    // detected strictly before that stratum's first lawful in-window formation.
    let mut total_touch = 0u64;
    let mut total_fill = 0u64;
    let mut all_left_truncated = true;
    for (timeframe, stratum) in audit.iter_mut() {
        stratum.formed_n = formed.iter().filter(|(tf, _)| tf == timeframe).count() as u64;
        stratum.formed_fill_without_prior_touch_n = a1.get(timeframe).copied().unwrap_or(0);
        stratum.first_formed_detection_ts = first_formed_detection.get(timeframe).copied();
        stratum.orphan_detection_max = stratum
            .orphan_events
            .iter()
            .map(|event| event.detection_ts)
            .max();
        stratum.all_orphans_left_truncated = stratum
            .orphan_events
            .iter()
            .all(|event| Some(event.detection_ts) < stratum.first_formed_detection_ts);
        all_left_truncated &= stratum.all_orphans_left_truncated;
        total_touch += stratum.orphan_touch_n;
        total_fill += stratum.orphan_fill_n;
    }

    // Reconcile with the scanner's own counters: fail closed on any mismatch.
    let results_value: serde_json::Value = serde_json::from_reader(File::open(&results_new)?)?;
    for row in results_value["tables"]["strata"]["rows"].as_array().expect("strata rows") {
        let timeframe = row["timeframe"].as_str().expect("timeframe");
        let stratum = audit.get(timeframe).ok_or("stratum vanished")?;
        let scanner_touch = row["orphan_touch_n"].as_u64().expect("orphan_touch_n");
        let scanner_fill = row["orphan_fill_n"].as_u64().expect("orphan_fill_n");
        let scanner_a1 = row["formed_fill_without_prior_touch_n"]
            .as_u64()
            .expect("formed_fill_without_prior_touch_n");
        if scanner_touch != stratum.orphan_touch_n
            || scanner_fill != stratum.orphan_fill_n
            || scanner_a1 != stratum.formed_fill_without_prior_touch_n
        {
            return Err(format!(
                "audit does not reconcile with scanner counters for {timeframe}: \
                 orphans {}/{} vs {}/{}, a1 {} vs {}",
                stratum.orphan_touch_n,
                stratum.orphan_fill_n,
                scanner_touch,
                scanner_fill,
                stratum.formed_fill_without_prior_touch_n,
                scanner_a1
            )
            .into());
        }
    }

    // Aggregate delta against the superseded (scope-misaligned) result.
    let old_value: serde_json::Value = serde_json::from_reader(File::open(&results_old)?)?;
    let mut deltas = Vec::new();
    for timeframe in TIMEFRAMES {
        let pick = |value: &serde_json::Value, timeframe: &str| {
            value["tables"]["strata"]["rows"]
                .as_array()
                .expect("strata rows")
                .iter()
                .find(|row| row["timeframe"].as_str() == Some(timeframe))
                .cloned()
                .expect("stratum row")
        };
        let old_row = pick(&old_value, timeframe);
        let new_row = pick(&results_value, timeframe);
        deltas.push(ResultsDelta {
            timeframe: timeframe.to_string(),
            formations_old: old_row["lawful_formation_n"].as_u64().expect("formations"),
            formations_new: new_row["lawful_formation_n"].as_u64().expect("formations"),
            formed_fill_without_prior_touch_old: old_row["formed_fill_without_prior_touch_n"]
                .as_u64()
                .expect("a1"),
            formed_fill_without_prior_touch_new: new_row["formed_fill_without_prior_touch_n"]
                .as_u64()
                .expect("a1"),
            orphan_touch_old: old_row["orphan_touch_n"].as_u64().expect("orphans"),
            orphan_touch_new: new_row["orphan_touch_n"].as_u64().expect("orphans"),
            orphan_fill_old: old_row["orphan_fill_n"].as_u64().expect("orphans"),
            orphan_fill_new: new_row["orphan_fill_n"].as_u64().expect("orphans"),
            fill_rate_old: old_row["fill_rate"]["rate"].as_f64(),
            fill_rate_new: new_row["fill_rate"]["rate"].as_f64(),
            bullish_share_old: old_row["bullish_n"].as_f64().zip(old_row["lawful_formation_n"].as_f64()).map(|(b, n)| b / n),
            bullish_share_new: new_row["bullish_n"].as_f64().zip(new_row["lawful_formation_n"].as_f64()).map(|(b, n)| b / n),
        });
    }
    let sign_counts = |value: &serde_json::Value| {
        let mut negative = 0u64;
        let mut zero = 0u64;
        let mut positive = 0u64;
        let mut raw_positive = 0u64;
        let mut cells = 0u64;
        for row in value["tables"]["strata"]["rows"].as_array().expect("rows") {
            for horizon in row["horizons"].as_array().expect("horizons") {
                let adjusted = horizon["direction_adjusted_return_bps"]["p50"].as_f64().unwrap_or(0.0);
                cells += 1;
                if adjusted < 0.0 {
                    negative += 1;
                } else if adjusted == 0.0 {
                    zero += 1;
                } else {
                    positive += 1;
                }
                if horizon["raw_return_bps"]["p50"].as_f64().unwrap_or(0.0) > 0.0 {
                    raw_positive += 1;
                }
            }
        }
        json!({
            "direction_adjusted_negative": negative,
            "direction_adjusted_zero": zero,
            "direction_adjusted_positive": positive,
            "raw_median_positive": raw_positive,
            "cells": cells,
        })
    };

    let report = json!({
        "artifact_type": "AP-002_E1A_SCOPE_REPAIR_AUDIT",
        "development_start_inclusive": "2025-07-31T16:15:00Z",
        "zone_rows_parsed": zone_rows,
        "payload_rows_parsed": payload_rows,
        "malformed_payloads": malformed_payloads,
        "formed_zones_total": formed.len(),
        "a1_total": a1.values().sum::<u64>(),
        "a2_orphan_touch_total": total_touch,
        "a2_orphan_fill_total": total_fill,
        "a2_orphan_total": total_touch + total_fill,
        "all_orphans_left_truncated": all_left_truncated,
        "reconciled_with_scanner_counters": true,
        "strata": audit,
        "results_delta_vs_superseded": deltas,
        "prospective_sign_counts": {
            "superseded_result": sign_counts(&old_value),
            "corrected_result": sign_counts(&results_value),
        },
        "scanner_logical_result_hash": results_value["logical_result_hash"],
        "elapsed_millis": started.elapsed().as_millis() as u64,
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "A1 total {} | A2 {} touch / {} fill | all left-truncated: {} | payload rows {} | malformed {} | elapsed_ms {}",
        a1.values().sum::<u64>(),
        total_touch,
        total_fill,
        all_left_truncated,
        payload_rows,
        malformed_payloads,
        started.elapsed().as_millis()
    );
    Ok(())
}
