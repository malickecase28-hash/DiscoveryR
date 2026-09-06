//! AP-002 E1B final pre-interpretation contract audit: proves that the FILL
//! stratification flag on every frozen E1B fill anchor equals the producer's
//! `fvg_filled` payload field `first_touch_observed`, over the complete lawful
//! in-window fill population.  Fail-closed; Rust-only per PIPELINE_RULES.md.

use arrow_array::Array;
use research_tape::fvg_e1::parse_payload;
use research_tape::{batch_i64, batch_large_string, required_i64, ProjectedParquetReader};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Component, Path, PathBuf},
    process,
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const MAX_REPORTED_DISAGREEMENTS: usize = 20;

#[derive(Debug, Clone, Serialize)]
struct Disagreement {
    timeframe: String,
    zone_id: u64,
    fill_ts: i64,
    payload_first_touch_observed: bool,
    anchor_first_touch_observed: bool,
}

fn tf_index(timeframe: &str) -> Option<u8> {
    TIMEFRAMES
        .iter()
        .position(|tf| *tf == timeframe)
        .map(|i| i as u8)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();
    let mut view_root = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut sidecar = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v2_scope_repair.jsonl",
    );
    let mut anchors_path = PathBuf::new();
    let mut output = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--view-root" => view_root = args.next().ok_or("missing --view-root")?.into(),
            "--sidecar" => sidecar = args.next().ok_or("missing --sidecar")?.into(),
            "--anchors" => anchors_path = args.next().ok_or("missing --anchors")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if anchors_path.as_os_str().is_empty() || output.as_os_str().is_empty() {
        return Err("--anchors and --output are required".into());
    }

    // Availability set: (timeframe index, bar_close_ts_ns) pairs that have a
    // sidecar boundary-crossing record, i.e. lawful bars.
    let sidecar_set: HashSet<(u8, i64)> = {
        let reader = BufReader::new(File::open(&sidecar)?);
        let mut set = HashSet::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(&line)?;
            let timeframe = value["timeframe"]
                .as_str()
                .ok_or("sidecar line missing timeframe")?;
            let Some(index) = tf_index(timeframe) else {
                return Err(format!("sidecar references unknown timeframe {timeframe}").into());
            };
            set.insert((
                index,
                value["bar_close_ts_ns"]
                    .as_i64()
                    .ok_or("sidecar missing bar_close_ts_ns")?,
            ));
        }
        set
    };

    // Stream the corrected view: lawful in-window formations and their lawful
    // fill payloads, using the canonical parser.
    let manifest: serde_json::Value =
        serde_json::from_reader(File::open(view_root.join("view_manifest.json"))?)?;
    let mut formed: HashSet<(u8, u64)> = HashSet::new();
    let mut lawful_fills: BTreeMap<(u8, u64), (i64, bool)> = BTreeMap::new();
    let mut duplicate_fill_payloads = 0_u64;
    let mut payload_rows = 0_u64;
    for (index, timeframe) in TIMEFRAMES.iter().enumerate() {
        let index = index as u8;
        for part in manifest["source_groups"][*timeframe]
            .as_array()
            .ok_or_else(|| format!("missing source group: {timeframe}"))?
        {
            let logical = part["logical_path"]
                .as_str()
                .ok_or("part missing logical_path")?;
            let relative = Path::new(logical);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
            {
                return Err(format!("unsafe view path: {logical}").into());
            }
            let reader = ProjectedParquetReader::new(
                view_root.join(relative),
                ["bar_close_ts", "payload_fvg"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                8192,
            )?;
            let scan = reader.scan()?;
            for batch in scan {
                let batch = batch?;
                let bar_close = batch_i64(&batch, "bar_close_ts")?;
                let payload = batch_large_string(&batch, "payload_fvg")?;
                for row in 0..batch.num_rows() {
                    let bar_close_ts = required_i64(bar_close, row, "bar_close_ts")?;
                    if !sidecar_set.contains(&(
                        index,
                        bar_close_ts
                            .checked_mul(1_000_000)
                            .ok_or("bar close ns overflow")?,
                    )) {
                        continue;
                    }
                    let raw = if payload.is_null(row) {
                        continue;
                    } else {
                        payload.value(row)
                    };
                    if raw.is_empty() || raw == "{}" {
                        continue;
                    }
                    payload_rows += 1;
                    let parsed = parse_payload(raw)?;
                    for (direction, formation) in parsed.formations() {
                        if formation.detection_ts != bar_close_ts {
                            return Err(format!(
                                "formation detection {} does not match bar close {} ({direction:?} zone {})",
                                formation.detection_ts, bar_close_ts, formation.zone_id
                            )
                            .into());
                        }
                        formed.insert((index, formation.zone_id));
                    }
                    for event in parsed.fvg_filled {
                        if !formed.contains(&(index, event.zone_id)) {
                            continue; // orphan/pre-development fill: not in the lawful cohort
                        }
                        let key = (index, event.zone_id);
                        if lawful_fills
                            .insert(key, (event.fill_ts, event.first_touch_observed))
                            .is_some()
                        {
                            duplicate_fill_payloads += 1;
                        }
                    }
                }
            }
        }
    }

    // Frozen E1B FILL anchors.
    let anchor_reader = BufReader::new(File::open(&anchors_path)?);
    let mut anchor_flags: BTreeMap<(u8, u64), (i64, bool)> = BTreeMap::new();
    let mut fill_anchors_total = 0_u64;
    for line in anchor_reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)?;
        if value["stage"].as_str() != Some("FILL") {
            continue;
        }
        fill_anchors_total += 1;
        let timeframe = value["identity"]
            .as_str()
            .ok_or("anchor missing identity")?;
        let timeframe = timeframe
            .split(':')
            .next()
            .ok_or("anchor identity malformed")?;
        let Some(index) = tf_index(timeframe) else {
            return Err(format!("anchor references unknown timeframe {timeframe}").into());
        };
        let zone_id = value["zone_id"].as_u64().ok_or("anchor missing zone_id")?;
        let anchor_bar_close_ts = value["anchor_bar_close_ts"]
            .as_i64()
            .ok_or("anchor missing bar close")?;
        let first_touch_observed = value["first_touch_observed"]
            .as_bool()
            .ok_or("anchor missing first_touch_observed")?;
        if anchor_flags
            .insert(
                (index, zone_id),
                (anchor_bar_close_ts, first_touch_observed),
            )
            .is_some()
        {
            return Err(format!("duplicate FILL anchor for {timeframe} zone {zone_id}").into());
        }
    }

    // Reconcile payload field vs anchor field over the complete lawful cohort.
    let payload_true = lawful_fills.values().filter(|(_, flag)| *flag).count() as u64;
    let payload_false = lawful_fills.len() as u64 - payload_true;
    let anchor_true = anchor_flags.values().filter(|(_, flag)| *flag).count() as u64;
    let anchor_false = anchor_flags.len() as u64 - anchor_true;
    let mut agreement = 0_u64;
    let mut disagreements: Vec<Disagreement> = Vec::new();
    let mut fill_ts_mismatches = 0_u64;
    let mut missing_anchors = 0_u64;
    for ((index, zone_id), (fill_ts, payload_flag)) in &lawful_fills {
        match anchor_flags.get(&(*index, *zone_id)) {
            Some((anchor_bar_close_ts, anchor_flag)) => {
                if anchor_bar_close_ts != fill_ts {
                    fill_ts_mismatches += 1;
                }
                if anchor_flag == payload_flag {
                    agreement += 1;
                } else if disagreements.len() < MAX_REPORTED_DISAGREEMENTS {
                    disagreements.push(Disagreement {
                        timeframe: TIMEFRAMES[*index as usize].to_string(),
                        zone_id: *zone_id,
                        fill_ts: *fill_ts,
                        payload_first_touch_observed: *payload_flag,
                        anchor_first_touch_observed: *anchor_flag,
                    });
                }
            }
            None => {
                missing_anchors += 1;
                if disagreements.len() < MAX_REPORTED_DISAGREEMENTS {
                    disagreements.push(Disagreement {
                        timeframe: TIMEFRAMES[*index as usize].to_string(),
                        zone_id: *zone_id,
                        fill_ts: *fill_ts,
                        payload_first_touch_observed: *payload_flag,
                        anchor_first_touch_observed: false,
                    });
                }
            }
        }
    }
    let unmatched_anchors = (anchor_flags.len() as u64).saturating_sub(
        lawful_fills
            .keys()
            .filter(|key| anchor_flags.contains_key(key))
            .count() as u64,
    );
    let disagreement = (lawful_fills.len() as u64).saturating_sub(agreement);

    let lawful_fill_payload_count = lawful_fills.len() as u64;
    let pass = lawful_fill_payload_count == fill_anchors_total
        && agreement == lawful_fill_payload_count
        && disagreement == 0
        && missing_anchors == 0
        && unmatched_anchors == 0
        && fill_ts_mismatches == 0
        && duplicate_fill_payloads == 0;
    // Expected reference values from current evidence (not forced): FILL
    // anchors 379,132; gap-through/false 1,010.  Recorded, not asserted.
    let audit = json!({
        "artifact_type": "AP-002_E1B_FILL_TOUCH_CONSISTENCY_AUDIT",
        "generated_from": {
            "view_root": view_root.to_string_lossy().replace('\\', "/"),
            "sidecar": sidecar.to_string_lossy().replace('\\', "/"),
            "anchors": anchors_path.to_string_lossy().replace('\\', "/"),
        },
        "payload_rows_parsed": payload_rows,
        "lawful_fill_payload_count": lawful_fill_payload_count,
        "fill_anchors_total": fill_anchors_total,
        "payload_first_touch_observed_true": payload_true,
        "payload_first_touch_observed_false": payload_false,
        "anchor_first_touch_observed_true": anchor_true,
        "anchor_first_touch_observed_false": anchor_false,
        "agreement_n": agreement,
        "disagreement_n": disagreement,
        "missing_anchors": missing_anchors,
        "unmatched_anchors": unmatched_anchors,
        "fill_ts_mismatches": fill_ts_mismatches,
        "duplicate_fill_payloads": duplicate_fill_payloads,
        "expected_reference": {"fill_anchors": 379132, "false_or_gap_through": 1010},
        "representative_disagreements": disagreements,
        "pass": pass,
        "gate": if pass { "E1B_PRE_INTERPRETATION_GATE = PASS" } else { "E1B_PRE_INTERPRETATION_GATE = REWORK" },
        "elapsed_millis": started.elapsed().as_millis() as u64,
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_vec_pretty(&audit)?)?;
    println!(
        "lawful fills {lawful_fill_payload_count} | anchors {fill_anchors_total} | payload t/f {payload_true}/{payload_false} | anchor t/f {anchor_true}/{anchor_false} | agreement {agreement} | disagreement {disagreement} | missing {missing_anchors} | unmatched {unmatched_anchors} | fill_ts mismatches {fill_ts_mismatches} | duplicates {duplicate_fill_payloads} | PASS {pass} | elapsed_ms {}",
        started.elapsed().as_millis()
    );
    if !pass {
        process::exit(2);
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1B fill-touch consistency audit failed: {error}");
        process::exit(1);
    }
}
