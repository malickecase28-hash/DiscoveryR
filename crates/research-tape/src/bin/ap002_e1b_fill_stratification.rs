//! AP-002 E1B contract-completion aggregate + interpreter fact check.
//!
//! Completes the frozen E1B contract's FILL stratification
//! (first_touch_observed true/false) from the frozen row-level anchor table
//! without rerunning the scanner, and mechanically verifies the interpreter
//! sign-count claims against the frozen E1B results.  Descriptive only.

use research_tape::fvg_e1::Direction;
use research_tape::fvg_e1b::Stage;
use research_tape::fvg_e1b::StageOutcome;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process,
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const HORIZONS_BARS: [u64; 3] = [1, 3, 5];
const EXPECTED_ANCHOR_PHYSICAL_SHA256: &str =
    "911301ee6ac985f76a954af5ddac10d9d8957508f4c2d44e348c718d564285c2";

#[derive(Debug, Deserialize)]
struct AnchorRow {
    stage: Stage,
    identity: String,
    direction: Direction,
    first_touch_observed: Option<bool>,
    #[allow(dead_code)]
    gap_atr: f64,
    #[allow(dead_code)]
    fvg_quality: f64,
    outcomes: [Option<StageOutcome>; 3],
}

#[derive(Debug, Clone, Serialize)]
struct Quantiles {
    n: u64,
    p05: Option<f64>,
    p25: Option<f64>,
    p50: Option<f64>,
    p75: Option<f64>,
    p95: Option<f64>,
}

fn quantiles(values: &[f64]) -> Quantiles {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    let pick = |p: f64| {
        if values.is_empty() {
            None
        } else {
            Some(values[((values.len() - 1) as f64 * p).round() as usize])
        }
    };
    Quantiles {
        n: values.len() as u64,
        p05: pick(0.05),
        p25: pick(0.25),
        p50: pick(0.50),
        p75: pick(0.75),
        p95: pick(0.95),
    }
}

#[derive(Debug, Clone, Serialize)]
struct HorizonCell {
    horizon_bars: u64,
    anchor_n: u64,
    outcome_n: u64,
    end_of_window_missing_n: u64,
    raw_return_bps: Quantiles,
    direction_adjusted_return_bps: Quantiles,
    bullish_n: u64,
    bearish_n: u64,
}

#[derive(Debug, Clone, Serialize)]
struct FillTypeCell {
    timeframe: String,
    fill_type: &'static str,
    anchor_n: u64,
    eligibility: &'static str,
    horizons: Vec<HorizonCell>,
    bullish_n: u64,
    bearish_n: u64,
}

/// Predeclared descriptive support rule, fixed before inspecting per-cell
/// aggregates.  Labels aid sample-size reading only and imply no significance.
fn eligibility(anchor_n: u64) -> &'static str {
    match anchor_n {
        0 => "EMPTY",
        1..=29 => "TOO_SPARSE_FOR_COMPARISON",
        30..=199 => "DESCRIPTIVE_THIN",
        _ => "DESCRIPTIVE_ADEQUATE",
    }
}

fn sign_of(value: f64) -> &'static str {
    if value < 0.0 {
        "negative"
    } else if value == 0.0 {
        "zero"
    } else {
        "positive"
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

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1B fill stratification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();
    let mut anchors_path = PathBuf::new();
    let mut results_path = PathBuf::new();
    let mut stratification_output = PathBuf::new();
    let mut factcheck_output = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--anchors" => anchors_path = args.next().ok_or("missing --anchors")?.into(),
            "--results" => results_path = args.next().ok_or("missing --results")?.into(),
            "--stratification-output" => {
                stratification_output = args.next().ok_or("missing --stratification-output")?.into()
            }
            "--factcheck-output" => {
                factcheck_output = args.next().ok_or("missing --factcheck-output")?.into()
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    for (name, path) in [
        ("--anchors", &anchors_path),
        ("--results", &results_path),
        ("--stratification-output", &stratification_output),
        ("--factcheck-output", &factcheck_output),
    ] {
        if path.as_os_str().is_empty() {
            return Err(format!("{name} is required").into());
        }
    }

    // Stream the frozen anchor table, verifying its logical row hash while
    // reading, and collect FILL rows only.
    let expected_logical = if results_path.as_os_str().is_empty() {
        None
    } else {
        Some(
            serde_json::from_reader::<_, serde_json::Value>(File::open(&results_path)?)?["tables"]
                ["anchors"]["logical_rows_sha256"]
                .as_str()
                .ok_or("results missing anchors logical_rows_sha256")?
                .to_string(),
        )
    };
    let mut reader = BufReader::new(File::open(&anchors_path)?);
    let mut logical = Sha256::new();
    let mut rows_total = 0_u64;
    let mut fill_rows: Vec<(String, AnchorRow)> = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        // read_line retains the trailing newline, matching the frozen hash
        // computed by the scanner's AnchorWriter and the E1B integrity gate.
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        logical.update(line.as_bytes());
        rows_total += 1;
        let row: AnchorRow = serde_json::from_str(&line)?;
        if row.stage == Stage::Fill {
            let timeframe = row
                .identity
                .split(':')
                .next()
                .ok_or("anchor identity malformed")?
                .to_string();
            fill_rows.push((timeframe, row));
        }
    }
    let anchor_logical_hash = format!("{:x}", logical.finalize());
    if let Some(expected) = &expected_logical {
        if &anchor_logical_hash != expected {
            return Err(format!(
                "frozen anchor table logical hash mismatch: expected {expected}, got {anchor_logical_hash}"
            )
            .into());
        }
    }
    let anchor_physical_sha256 = sha256_file(&anchors_path)?;
    if anchor_physical_sha256 != EXPECTED_ANCHOR_PHYSICAL_SHA256 {
        return Err(format!(
            "frozen anchor table physical sha256 mismatch: expected {EXPECTED_ANCHOR_PHYSICAL_SHA256}, got {anchor_physical_sha256}"
        )
        .into());
    }

    // Partition FILL anchors by native timeframe x fill type.
    let mut cells: BTreeMap<(usize, bool), Vec<&AnchorRow>> = BTreeMap::new();
    for (timeframe, row) in &fill_rows {
        let index = TIMEFRAMES
            .iter()
            .position(|tf| tf == timeframe)
            .ok_or_else(|| format!("anchor references unknown timeframe {timeframe}"))?;
        cells
            .entry((index, row.first_touch_observed == Some(true)))
            .or_default()
            .push(row);
    }
    let mut stratification_cells = Vec::new();
    for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
        for (touched, fill_type) in [(true, "TOUCHED_FILL"), (false, "GAP_THROUGH_FILL")] {
            let rows = cells.get(&(index, touched)).cloned().unwrap_or_default();
            let anchor_n = rows.len() as u64;
            let horizons = HORIZONS_BARS
                .iter()
                .enumerate()
                .map(|(horizon_index, horizon_bars)| {
                    let applied: Vec<&AnchorRow> = rows
                        .iter()
                        .filter(|row| row.outcomes[horizon_index].is_some())
                        .copied()
                        .collect();
                    let raw = applied
                        .iter()
                        .filter_map(|row| row.outcomes[horizon_index].as_ref())
                        .map(|outcome| outcome.raw_return_bps)
                        .collect::<Vec<_>>();
                    let adjusted = applied
                        .iter()
                        .filter_map(|row| row.outcomes[horizon_index].as_ref())
                        .map(|outcome| outcome.direction_adjusted_return_bps)
                        .collect::<Vec<_>>();
                    HorizonCell {
                        horizon_bars: *horizon_bars,
                        anchor_n,
                        outcome_n: applied.len() as u64,
                        end_of_window_missing_n: anchor_n.saturating_sub(applied.len() as u64),
                        raw_return_bps: quantiles(&raw),
                        direction_adjusted_return_bps: quantiles(&adjusted),
                        bullish_n: applied
                            .iter()
                            .filter(|row| row.direction == Direction::Bullish)
                            .count() as u64,
                        bearish_n: applied
                            .iter()
                            .filter(|row| row.direction == Direction::Bearish)
                            .count() as u64,
                    }
                })
                .collect();
            stratification_cells.push(FillTypeCell {
                timeframe: (*timeframe).to_string(),
                fill_type,
                anchor_n,
                eligibility: eligibility(anchor_n),
                horizons,
                bullish_n: rows
                    .iter()
                    .filter(|row| row.direction == Direction::Bullish)
                    .count() as u64,
                bearish_n: rows
                    .iter()
                    .filter(|row| row.direction == Direction::Bearish)
                    .count() as u64,
            });
        }
    }
    let touched_total = stratification_cells
        .iter()
        .filter(|cell| cell.fill_type == "TOUCHED_FILL")
        .map(|cell| cell.anchor_n)
        .sum::<u64>();
    let gap_through_total = stratification_cells
        .iter()
        .filter(|cell| cell.fill_type == "GAP_THROUGH_FILL")
        .map(|cell| cell.anchor_n)
        .sum::<u64>();
    let stratification = json!({
        "artifact_type": "AP-002_E1B_FILL_STRATIFICATION",
        "anchors_logical_rows_sha256": anchor_logical_hash,
        "anchors_physical_sha256": anchor_physical_sha256,
        "anchors_rows_total": rows_total,
        "fill_anchors_total": fill_rows.len() as u64,
        "touched_fill_total": touched_total,
        "gap_through_fill_total": gap_through_total,
        "predeclared_support_rule": {
            "EMPTY": "anchor_n = 0",
            "TOO_SPARSE_FOR_COMPARISON": "1 <= anchor_n < 30",
            "DESCRIPTIVE_THIN": "30 <= anchor_n < 200",
            "DESCRIPTIVE_ADEQUATE": "anchor_n >= 200",
            "note": "rule fixed before inspecting per-cell aggregates; labels carry no significance meaning"
        },
        "cells": stratification_cells,
        "elapsed_millis": started.elapsed().as_millis() as u64,
    });

    // Mechanical fact check against the frozen E1B results.
    let results: serde_json::Value = serde_json::from_reader(File::open(&results_path)?)?;
    let strata = results["tables"]["strata"]["rows"]
        .as_array()
        .ok_or("results strata rows missing")?;
    let mut ft_adj_signs: BTreeMap<&'static str, u64> =
        BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]);
    let mut fill_adj_signs: BTreeMap<&'static str, u64> =
        BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]);
    let mut fill_gt_ft_count = 0_u64;
    let mut raw_signs: BTreeMap<&'static str, u64> =
        BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]);
    let mut bullish_signs_by_stage: BTreeMap<&'static str, BTreeMap<&'static str, u64>> =
        BTreeMap::from([
            (
                "first_touch",
                BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]),
            ),
            (
                "fill",
                BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]),
            ),
        ]);
    let mut bearish_signs_by_stage: BTreeMap<&'static str, BTreeMap<&'static str, u64>> =
        BTreeMap::from([
            (
                "first_touch",
                BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]),
            ),
            (
                "fill",
                BTreeMap::from([("negative", 0), ("zero", 0), ("positive", 0)]),
            ),
        ]);
    let mut cells_compared = 0_u64;
    let mut same_bar_rates: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for stratum in strata {
        let timeframe = stratum["timeframe"].as_str().ok_or("timeframe missing")?;
        for stage_key in ["first_touch", "fill"] {
            for horizon in stratum[stage_key]["horizons"]
                .as_array()
                .ok_or("horizons missing")?
            {
                let adjusted = horizon["direction_adjusted_return_bps"]["p50"]
                    .as_f64()
                    .ok_or("adjusted p50 missing")?;
                let raw = horizon["raw_return_bps"]["p50"]
                    .as_f64()
                    .ok_or("raw p50 missing")?;
                let sign = sign_of(adjusted);
                if stage_key == "first_touch" {
                    *ft_adj_signs.get_mut(sign).unwrap() += 1;
                } else {
                    *fill_adj_signs.get_mut(sign).unwrap() += 1;
                }
                *raw_signs.get_mut(sign_of(raw)).unwrap() += 1;
                cells_compared += 1;
                let split = |direction: &str| {
                    horizon["direction_split"]
                        .as_array()
                        .expect("direction split")
                        .iter()
                        .find(|entry| entry["direction"].as_str() == Some(direction))
                        .expect("direction entry")["direction_adjusted_p50"]
                        .as_f64()
                        .unwrap_or(0.0)
                };
                *bullish_signs_by_stage
                    .get_mut(stage_key)
                    .unwrap()
                    .get_mut(sign_of(split("bullish")))
                    .unwrap() += 1;
                *bearish_signs_by_stage
                    .get_mut(stage_key)
                    .unwrap()
                    .get_mut(sign_of(split("bearish")))
                    .unwrap() += 1;
            }
        }
        // FILL adjusted p50 > FIRST_TOUCH adjusted p50, paired per stratum/horizon.
        for horizon_index in 0..3 {
            let ft = stratum["first_touch"]["horizons"][horizon_index]
                ["direction_adjusted_return_bps"]["p50"]
                .as_f64()
                .ok_or("ft adjusted p50 missing")?;
            let fill = stratum["fill"]["horizons"][horizon_index]["direction_adjusted_return_bps"]
                ["p50"]
                .as_f64()
                .ok_or("fill adjusted p50 missing")?;
            if fill > ft {
                fill_gt_ft_count += 1;
            }
        }
        let touched_fill_n = stratum["touched_fill_n"]
            .as_u64()
            .ok_or("touched_fill_n missing")?;
        let same_bar_n = stratum["same_bar_touch_fill_n"]
            .as_u64()
            .ok_or("same_bar missing")?;
        same_bar_rates.insert(
            timeframe.to_string(),
            json!({
                "same_bar_touch_fill_n": same_bar_n,
                "touched_fill_n_denominator": touched_fill_n,
                "rate": (touched_fill_n != 0).then_some(same_bar_n as f64 / touched_fill_n as f64),
            }),
        );
    }
    let factcheck = json!({
        "artifact_type": "AP-002_E1B_INTERPRETER_FACT_CHECK",
        "cells_compared_per_stage": cells_compared / 2,
        "first_touch_adjusted_p50_signs": ft_adj_signs,
        "fill_adjusted_p50_signs": fill_adj_signs,
        "fill_adjusted_p50_greater_than_first_touch_count": {
            "count": fill_gt_ft_count,
            "denominator": 21,
        },
        "raw_p50_signs_both_stages": raw_signs,
        "bullish_direction_split_p50_signs_by_stage": bullish_signs_by_stage,
        "bearish_direction_split_p50_signs_by_stage": bearish_signs_by_stage,
        "same_bar_touch_fill_rate": same_bar_rates,
        "claims_verified": {
            "P03_first_touch_adjusted_positive_21_of_21": ft_adj_signs["positive"] == 21,
            "P03_P04_fill_adjusted_positive_21_of_21": fill_adj_signs["positive"] == 21,
            "P03_fill_greater_than_first_touch_19_of_21": fill_gt_ft_count == 19,
            "P04_raw_p50_positive_41_of_42": raw_signs["positive"] == 41,
            "P04_bullish_positive_42_of_42": bullish_signs_by_stage.values().all(|signs| signs["positive"] == 21),
            "P04_bearish_first_touch_5_pos_5_zero_11_neg": bearish_signs_by_stage["first_touch"] == BTreeMap::from([("negative", 11), ("zero", 5), ("positive", 5)]),
            "P04_bearish_fill_10_pos_2_zero_9_neg": bearish_signs_by_stage["fill"] == BTreeMap::from([("negative", 9), ("zero", 2), ("positive", 10)]),
        },
        "note": "mechanical verification only; no interpretation performed",
        "elapsed_millis": started.elapsed().as_millis() as u64,
    });

    for (path, value) in [
        (&stratification_output, &stratification),
        (&factcheck_output, &factcheck),
    ] {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    println!(
        "FILL anchors {} (touched {touched_total} / gap-through {gap_through_total}) | anchor hash verified {anchor_logical_hash} | ft adj signs {ft_adj_signs:?} | fill adj signs {fill_adj_signs:?} | fill>ft {fill_gt_ft_count}/21 | raw signs {raw_signs:?} | elapsed_ms {}",
        fill_rows.len() as u64,
        started.elapsed().as_millis()
    );
    Ok(())
}
