//! AP-002 E1B stage-anchor scanner (FIRST_TOUCH and FILL), implementing the
//! frozen E1B measurement contract on top of the E1 systems: same preflight,
//! sidecar, trigger-tick anchor index and zone reconstruction; adds stage
//! anchors at the touch/fill boundary ticks and lawful-close prospective
//! outcomes relative to those anchors.  DEVELOPMENT only.  No interpretation.

use arrow_array::Array;
use research_tape::fvg_e1::{
    AnchorIndex, AnchorTick, Availability, Direction, Preflight, PreflightResult, SidecarCursor,
    ZoneBook, ZoneLifecycle,
};
use research_tape::fvg_e1b::{
    max_identifiable_quantile, quantiles, stage_anchor, stage_outcomes, LawfulBar, Quantiles, Stage,
};
use research_tape::{batch_f64, batch_i64, batch_large_string, required_i64, ProjectedParquetReader};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const HORIZONS_BARS: [u64; 3] = [1, 3, 5];

#[derive(Debug, Deserialize)]
struct ViewManifest {
    scope_id: String,
    logical_view_identity: String,
    source_groups: BTreeMap<String, Vec<ViewPart>>,
}
#[derive(Debug, Deserialize)]
struct ViewPart {
    logical_path: String,
    development_row_count: u64,
}
#[derive(Debug, Deserialize)]
struct SidecarManifest {
    artifact_type: String,
    artifact_version: String,
    development_view_manifest_sha256: String,
    projected_columns: Vec<String>,
    timeframes: Vec<String>,
    input_rows: u64,
    sidecar_rows: u64,
    sidecar_sha256: String,
}
#[derive(Debug, Deserialize)]
struct SidecarSequenceLine {
    trigger_source_sequence: i64,
}

#[derive(Debug, Clone, Serialize)]
struct Provenance {
    scope_id: String,
    development_view_manifest_sha256: String,
    sidecar_manifest_sha256: String,
    sidecar_sha256: String,
    source_parts: Vec<String>,
    scanner_commit: String,
}

#[derive(Debug, Clone, Serialize)]
struct Rate {
    numerator: u64,
    denominator: u64,
    rate: Option<f64>,
}
fn rate(numerator: u64, denominator: u64) -> Rate {
    Rate {
        numerator,
        denominator,
        rate: (denominator != 0).then_some(numerator as f64 / denominator as f64),
    }
}

#[derive(Debug, Clone, Serialize)]
struct StageAnchorRow {
    stage: Stage,
    identity: String,
    zone_id: u64,
    direction: research_tape::fvg_e1::Direction,
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
    gap_atr: f64,
    fvg_quality: f64,
    outcomes: [Option<research_tape::fvg_e1b::StageOutcome>; 3],
}

#[derive(Debug, Clone, Serialize)]
struct DirectionSplit {
    direction: &'static str,
    n: u64,
    direction_adjusted_p50: Option<f64>,
}
#[derive(Debug, Clone, Serialize)]
struct MedianSplit {
    split: &'static str,
    n: u64,
    direction_adjusted_p50: Option<f64>,
    threshold: f64,
}
#[derive(Debug, Clone, Serialize)]
struct HorizonBlock {
    horizon_bars: u64,
    n: u64,
    end_of_window_missing: u64,
    raw_price_delta: Quantiles,
    raw_return_bps: Quantiles,
    direction_adjusted_return_bps: Quantiles,
    direction_split: Vec<DirectionSplit>,
    gap_atr_median_split: Vec<MedianSplit>,
    fvg_quality_median_split: Vec<MedianSplit>,
}
#[derive(Debug, Clone, Serialize)]
struct StageProspective {
    anchors_n: u64,
    horizons: Vec<HorizonBlock>,
}

#[derive(Debug, Clone, Serialize)]
struct StratumResult {
    timeframe: String,
    lawful_formation_n: u64,
    touch_anchors_n: u64,
    fill_anchors_n: u64,
    touched_fill_n: u64,
    gap_through_fill_n: u64,
    same_bar_touch_fill_n: u64,
    touch_incidence: Rate,
    formation_to_touch_market_ms: Quantiles,
    formation_to_touch_known_ms: Quantiles,
    touch_to_fill_market_ms: Quantiles,
    touch_to_fill_known_ms: Quantiles,
    formation_to_fill_market_ms: Quantiles,
    formation_to_fill_known_ms: Quantiles,
    fill_right_censor_fraction: Option<f64>,
    fill_max_identifiable_quantile: Option<f64>,
    touch_to_fill_right_censor_fraction: Option<f64>,
    touch_to_fill_max_identifiable_quantile: Option<f64>,
    orphan_touch_n: u64,
    orphan_fill_n: u64,
    n_bars: u64,
    first_touch: StageProspective,
    fill: StageProspective,
}

#[derive(Debug, Serialize)]
struct TableEnvelope<T> {
    n: u64,
    provenance: Provenance,
    rows: T,
}
#[derive(Debug, Serialize)]
struct AnchorTableRef {
    n: u64,
    path: String,
    absolute_path: String,
    sha256: String,
    logical_rows_sha256: String,
    provenance: Provenance,
}
#[derive(Debug, Serialize)]
struct Results {
    experiment_id: &'static str,
    scope_id: String,
    exposure_class: &'static str,
    confirmation_status: &'static str,
    scanner_commit: String,
    preflight_hash: String,
    sidecar_manifest_sha256: String,
    sidecar_rows: u64,
    logical_result_hash: String,
    tables: ResultsTables,
}
#[derive(Debug, Serialize)]
struct ResultsTables {
    strata: TableEnvelope<Vec<StratumResult>>,
    anchors: AnchorTableRef,
}

struct AnchorWriter {
    writer: BufWriter<File>,
    physical: Sha256,
    logical: Sha256,
    rows: u64,
}
impl AnchorWriter {
    fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            writer: BufWriter::new(File::create(path)?),
            physical: Sha256::new(),
            logical: Sha256::new(),
            rows: 0,
        })
    }
    fn write(&mut self, row: &StageAnchorRow) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec(row)?;
        self.writer.write_all(&bytes)?;
        self.writer.write_all(b"\n")?;
        self.physical.update(&bytes);
        self.physical.update(b"\n");
        self.logical.update(&bytes);
        self.logical.update(b"\n");
        self.rows += 1;
        Ok(())
    }
    fn finish(mut self) -> Result<(u64, String, String), Box<dyn std::error::Error>> {
        self.writer.flush()?;
        Ok((
            self.rows,
            format!("{:x}", self.physical.finalize()),
            format!("{:x}", self.logical.finalize()),
        ))
    }
}

fn safe_relative_path(value: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir))
    {
        return Err(format!("unsafe view path: {value}").into());
    }
    Ok(path.to_owned())
}
fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn source_parts<'a>(
    manifest: &'a ViewManifest,
    timeframe: &str,
) -> Result<&'a [ViewPart], Box<dyn std::error::Error>> {
    manifest
        .source_groups
        .get(timeframe)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing source group: {timeframe}").into())
}

fn sidecar_trigger_sequences(sidecar: &Path) -> Result<BTreeSet<i64>, Box<dyn std::error::Error>> {
    let mut sequences = BTreeSet::new();
    let reader = BufReader::new(File::open(sidecar)?);
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            sequences.insert(
                serde_json::from_str::<SidecarSequenceLine>(&line)?.trigger_source_sequence,
            );
        }
    }
    Ok(sequences)
}

fn verify_sidecar(
    sidecar: &Path,
    sidecar_manifest_path: &Path,
    view_manifest_hash: &str,
) -> Result<(SidecarManifest, String, String), Box<dyn std::error::Error>> {
    let manifest_bytes = fs::read(sidecar_manifest_path)?;
    let manifest: SidecarManifest = serde_json::from_slice(&manifest_bytes)?;
    if manifest.artifact_type != "FVG_BAR_AVAILABILITY_SIDECAR"
        || manifest.artifact_version != "V1"
        || manifest.development_view_manifest_sha256 != view_manifest_hash
        || manifest.projected_columns
            != ["event_ts_ns", "received_ts_ns", "source_sequence"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        || manifest.timeframes
            != TIMEFRAMES
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
    {
        return Err("frozen sidecar manifest does not match the declared authority".into());
    }
    let manifest_hash = format!("{:x}", Sha256::digest(&manifest_bytes));
    let sidecar_hash = sha256_file(sidecar)?;
    if sidecar_hash != manifest.sidecar_sha256 {
        return Err(format!(
            "sidecar hash mismatch: expected {}, got {}",
            manifest.sidecar_sha256, sidecar_hash
        )
        .into());
    }
    Ok((manifest, manifest_hash, sidecar_hash))
}

fn run_preflight(
    view_root: &Path,
    manifest: &ViewManifest,
    manifest_hash: &str,
    scanner_commit: &str,
    trigger_sequences: BTreeSet<i64>,
) -> Result<
    (
        research_tape::fvg_e1::PreflightResult,
        research_tape::fvg_e1::AnchorIndex,
    ),
    Box<dyn std::error::Error>,
> {
    let mut preflight =
        Preflight::new(manifest_hash, scanner_commit).with_capture_sequences(trigger_sequences);
    let mut last_source_sequence = None;
    for part in source_parts(manifest, "tick")? {
        let path = view_root.join(safe_relative_path(&part.logical_path)?);
        let reader = ProjectedParquetReader::new(
            path,
            [
                "event_ts_ns",
                "received_ts_ns",
                "source_sequence",
                "bid",
                "ask",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            65_536,
        )?;
        let mut scan = reader.scan()?;
        let mut row = 0_u64;
        while let Some(batch) = scan.next() {
            let batch = batch?;
            let event = batch_i64(&batch, "event_ts_ns")?;
            let received = batch_i64(&batch, "received_ts_ns")?;
            let sequence = batch_i64(&batch, "source_sequence")?;
            let bid = batch_f64(&batch, "bid")?;
            let ask = batch_f64(&batch, "ask")?;
            for index in 0..batch.num_rows() {
                let event = required_i64(event, index, "event_ts_ns")?;
                let received = required_i64(received, index, "received_ts_ns")?;
                let sequence = required_i64(sequence, index, "source_sequence")?;
                if last_source_sequence.is_some_and(|previous| sequence <= previous) {
                    return Err(format!(
                        "source_sequence is not strictly increasing at {} row {}",
                        part.logical_path, row
                    )
                    .into());
                }
                last_source_sequence = Some(sequence);
                preflight.observe_tick(
                    &part.logical_path,
                    row,
                    received,
                    event,
                    sequence,
                    bid.value(index),
                    ask.value(index),
                )?;
                row += 1;
            }
        }
        if row != part.development_row_count {
            return Err(format!(
                "preflight row count mismatch for {}: declared {}, actual {}",
                part.logical_path, part.development_row_count, row
            )
            .into());
        }
    }
    let result = preflight.finish();
    Ok((result, preflight.anchors().clone()))
}

fn median(values: &mut Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[(values.len() - 1) / 2])
}

fn stage_block(
    rows: &[&StageAnchorRow],
    attribute: fn(&StageAnchorRow) -> f64,
) -> StageProspective {
    let anchors_n = rows.len() as u64;
    let mut gap_thresholds = rows.iter().map(|row| attribute(row)).collect::<Vec<_>>();
    let gap_threshold = median(&mut gap_thresholds).unwrap_or(0.0);
    let mut quality_thresholds = rows.iter().map(|row| row.fvg_quality).collect::<Vec<_>>();
    let quality_threshold = median(&mut quality_thresholds).unwrap_or(0.0);
    let horizons = HORIZONS_BARS
        .iter()
        .enumerate()
        .map(|(horizon_index, horizon_bars)| {
            let applied: Vec<&StageAnchorRow> = rows
                .iter()
                .filter(|row| row.outcomes[horizon_index].is_some())
                .map(|row| *row)
                .collect();
            let raw_delta = applied
                .iter()
                .filter_map(|row| row.outcomes[horizon_index].as_ref())
                .map(|outcome| outcome.raw_price_delta)
                .collect::<Vec<_>>();
            let raw_bps = applied
                .iter()
                .filter_map(|row| row.outcomes[horizon_index].as_ref())
                .map(|outcome| outcome.raw_return_bps)
                .collect::<Vec<_>>();
            let adjusted = applied
                .iter()
                .filter_map(|row| row.outcomes[horizon_index].as_ref())
                .map(|outcome| outcome.direction_adjusted_return_bps)
                .collect::<Vec<_>>();
            let adjusted_p50 = |subset: &[&StageAnchorRow]| {
                let mut values = subset
                    .iter()
                    .filter_map(|row| row.outcomes[horizon_index].as_ref())
                    .map(|outcome| outcome.direction_adjusted_return_bps)
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    None
                } else {
                    values.sort_by(f64::total_cmp);
                    Some(values[(values.len() - 1) / 2])
                }
            };
            let direction_split = vec![
                DirectionSplit {
                    direction: "bullish",
                    n: applied
                        .iter()
                        .filter(|row| row.direction == Direction::Bullish)
                        .count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| row.direction == Direction::Bullish)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                },
                DirectionSplit {
                    direction: "bearish",
                    n: applied
                        .iter()
                        .filter(|row| row.direction == Direction::Bearish)
                        .count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| row.direction == Direction::Bearish)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                },
            ];
            let gap_split = vec![
                MedianSplit {
                    split: "low_at_or_below_median",
                    n: applied.iter().filter(|row| attribute(row) <= gap_threshold).count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| attribute(row) <= gap_threshold)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                    threshold: gap_threshold,
                },
                MedianSplit {
                    split: "high_above_median",
                    n: applied.iter().filter(|row| attribute(row) > gap_threshold).count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| attribute(row) > gap_threshold)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                    threshold: gap_threshold,
                },
            ];
            let quality_split = vec![
                MedianSplit {
                    split: "low_at_or_below_median",
                    n: applied.iter().filter(|row| row.fvg_quality <= quality_threshold).count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| row.fvg_quality <= quality_threshold)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                    threshold: quality_threshold,
                },
                MedianSplit {
                    split: "high_above_median",
                    n: applied.iter().filter(|row| row.fvg_quality > quality_threshold).count() as u64,
                    direction_adjusted_p50: adjusted_p50(
                        &applied
                            .iter()
                            .filter(|row| row.fvg_quality > quality_threshold)
                            .map(|row| *row)
                            .collect::<Vec<_>>(),
                    ),
                    threshold: quality_threshold,
                },
            ];
            HorizonBlock {
                horizon_bars: *horizon_bars,
                n: applied.len() as u64,
                end_of_window_missing: (rows.len() - applied.len()) as u64,
                raw_price_delta: quantiles(&raw_delta),
                raw_return_bps: quantiles(&raw_bps),
                direction_adjusted_return_bps: quantiles(&adjusted),
                direction_split,
                gap_atr_median_split: gap_split,
                fvg_quality_median_split: quality_split,
            }
        })
        .collect();
    StageProspective {
        anchors_n,
        horizons,
    }
}

fn scan_stratum(
    view_root: &Path,
    manifest: &ViewManifest,
    sidecar: &Path,
    timeframe: &str,
    anchors: &research_tape::fvg_e1::AnchorIndex,
    anchor_writer: &mut AnchorWriter,
) -> Result<StratumResult, Box<dyn std::error::Error>> {
    let parts = source_parts(manifest, timeframe)?;
    let mut sidecar_cursor = SidecarCursor::new(BufReader::new(File::open(sidecar)?), timeframe);
    let mut book = ZoneBook::new(timeframe);
    let mut lawful: Vec<LawfulBar> = Vec::new();
    let mut tick_by_sequence: BTreeMap<i64, AnchorTick> = BTreeMap::new();
    let mut n_bars = 0_u64;
    let mut last_bar_close_ts = None;
    for part in parts {
        let path = view_root.join(safe_relative_path(&part.logical_path)?);
        let reader = ProjectedParquetReader::new(
            path,
            ["bar_close_ts", "close", "payload_fvg"]
                .into_iter()
                .map(String::from)
                .collect(),
            65_536,
        )?;
        let mut scan = reader.scan()?;
        let mut row = 0_u64;
        while let Some(batch) = scan.next() {
            let batch = batch?;
            let bar_close = batch_i64(&batch, "bar_close_ts")?;
            let close = batch_f64(&batch, "close")?;
            let payload = batch_large_string(&batch, "payload_fvg")?;
            for index in 0..batch.num_rows() {
                let bar_close_ts = required_i64(bar_close, index, "bar_close_ts")?;
                let close = if close.is_null(index) {
                    return Err(format!("null close at {} row {}", part.logical_path, row).into());
                } else {
                    close.value(index)
                };
                let payload = if payload.is_null(index) {
                    "{}"
                } else {
                    payload.value(index)
                };
                let availability = sidecar_cursor
                    .lookup(bar_close_ts.checked_mul(1_000_000).ok_or("bar close ns overflow")?)?
                    .map(|value| anchors.lookup(&value))
                    .transpose()?;
                if let Some(availability) = &availability {
                    let anchor_tick = availability
                        .anchor
                        .clone()
                        .ok_or("lawful bar without trigger tick")?;
                    lawful.push(LawfulBar {
                        close_ts: bar_close_ts,
                        close,
                        available_time_ns: availability.available_time_ns,
                        source_sequence: availability.source_sequence,
                    });
                    tick_by_sequence.insert(availability.source_sequence, anchor_tick);
                }
                book.ingest_bar_with_close_optional(bar_close_ts, availability, payload, Some(close))?;
                last_bar_close_ts = Some(bar_close_ts);
                n_bars += 1;
                row += 1;
            }
        }
        if row != part.development_row_count {
            return Err(format!(
                "bar row count mismatch for {}: declared {}, actual {}",
                part.logical_path, part.development_row_count, row
            )
            .into());
        }
    }
    let last_bar_close_ts = last_bar_close_ts.ok_or("empty native stratum")?;
    let orphan_touch_n = book.orphan_touch_count;
    let orphan_fill_n = book.orphan_fill_count;
    let zones = book.finish(last_bar_close_ts);

    let mut touch_rows: Vec<StageAnchorRow> = Vec::new();
    let mut fill_rows: Vec<StageAnchorRow> = Vec::new();
    for zone in &zones {
        for (stage, rows) in [(Stage::FirstTouch, &mut touch_rows), (Stage::Fill, &mut fill_rows)] {
            let sequence = match stage {
                Stage::FirstTouch => zone.first_touch_source_sequence,
                Stage::Fill => zone.fill_source_sequence,
            };
            let Some(sequence) = sequence else { continue };
            let Some(tick) = tick_by_sequence.get(&sequence) else {
                return Err(format!(
                    "stage anchor tick missing for {} sequence {sequence}",
                    zone.identity
                )
                .into());
            };
            let anchor = stage_anchor(zone, stage, tick)?;
            let outcomes = stage_outcomes(
                &lawful,
                anchor.anchor_bar_close_ts,
                anchor.available_time_ns,
                anchor.source_sequence,
                anchor.anchor_mid,
                anchor.direction,
            )?;
            rows.push(StageAnchorRow {
                stage,
                identity: anchor.identity,
                zone_id: anchor.zone_id,
                direction: anchor.direction,
                anchor_bar_close_ts: anchor.anchor_bar_close_ts,
                available_time_ns: anchor.available_time_ns,
                source_sequence: anchor.source_sequence,
                anchor_bid: anchor.anchor_bid,
                anchor_ask: anchor.anchor_ask,
                anchor_mid: anchor.anchor_mid,
                first_touch_observed: anchor.first_touch_observed,
                formation_ts: anchor.formation_ts,
                formation_available_time_ns: anchor.formation_available_time_ns,
                formation_to_anchor_market_ms: anchor.formation_to_anchor_market_ms,
                formation_to_anchor_known_ms: anchor.formation_to_anchor_known_ms,
                gap_atr: anchor.gap_atr,
                fvg_quality: anchor.fvg_quality,
                outcomes,
            });
        }
    }
    for row in touch_rows.iter().chain(fill_rows.iter()) {
        anchor_writer.write(row)?;
    }

    // Aggregate the stratum.
    let formations = zones.len() as u64;
    let touched: Vec<&ZoneLifecycle> =
        zones.iter().filter(|zone| zone.first_touch_ts.is_some()).collect();
    let filled: Vec<&ZoneLifecycle> =
        zones.iter().filter(|zone| zone.fill_ts.is_some()).collect();
    let touched_fill: Vec<&ZoneLifecycle> = filled
        .iter()
        .copied()
        .filter(|zone| zone.first_touch_ts.is_some())
        .collect();
    let gap_through_fill_n = filled.len() as u64 - touched_fill.len() as u64;
    let same_bar_touch_fill_n = filled
        .iter()
        .filter(|zone| {
            zone.first_touch_ts.is_some_and(|touch| Some(touch) == zone.fill_ts)
        })
        .count() as u64;
    let collect = |values: Vec<Option<i64>>| {
        quantiles(&values.into_iter().flatten().map(|v| v as f64).collect::<Vec<_>>())
    };
    let formation_to_touch_market = collect(
        touched
            .iter()
            .map(|zone| zone.formation_to_touch_market_ms)
            .collect(),
    );
    let formation_to_touch_known = collect(
        touched
            .iter()
            .map(|zone| zone.formation_to_touch_known_ms)
            .collect(),
    );
    let touch_to_fill_market = collect(
        touched_fill
            .iter()
            .map(|zone| {
                zone.fill_ts
                    .zip(zone.first_touch_ts)
                    .map(|(fill, touch)| fill.saturating_sub(touch))
            })
            .collect(),
    );
    let touch_to_fill_known = collect(
        touched_fill
            .iter()
            .map(|zone| {
                zone.fill_available_time_ns
                    .zip(zone.first_touch_available_time_ns)
                    .map(|(fill, touch)| fill.saturating_sub(touch) / 1_000_000)
            })
            .collect(),
    );
    let formation_to_fill_market = collect(
        filled.iter()
            .map(|zone| zone.formation_to_fill_market_ms)
            .collect(),
    );
    let formation_to_fill_known = collect(
        filled.iter()
            .map(|zone| zone.formation_to_fill_known_ms)
            .collect(),
    );
    let fill_censor_fraction = (formations != 0)
        .then(|| (formations - filled.len() as u64) as f64 / formations as f64);
    let touch_censor_fraction = (!touched.is_empty())
        .then(|| (touched.len() - touched_fill.len()) as f64 / touched.len() as f64);

    let touch_refs: Vec<&StageAnchorRow> = touch_rows.iter().collect();
    let fill_refs: Vec<&StageAnchorRow> = fill_rows.iter().collect();
    Ok(StratumResult {
        timeframe: timeframe.to_string(),
        lawful_formation_n: formations,
        touch_anchors_n: touch_rows.len() as u64,
        fill_anchors_n: fill_rows.len() as u64,
        touched_fill_n: touched_fill.len() as u64,
        gap_through_fill_n,
        same_bar_touch_fill_n,
        touch_incidence: rate(touch_rows.len() as u64, formations),
        formation_to_touch_market_ms: formation_to_touch_market,
        formation_to_touch_known_ms: formation_to_touch_known,
        touch_to_fill_market_ms: touch_to_fill_market,
        touch_to_fill_known_ms: touch_to_fill_known,
        formation_to_fill_market_ms: formation_to_fill_market,
        formation_to_fill_known_ms: formation_to_fill_known,
        fill_right_censor_fraction: fill_censor_fraction,
        fill_max_identifiable_quantile: fill_censor_fraction.map(max_identifiable_quantile),
        touch_to_fill_right_censor_fraction: touch_censor_fraction,
        touch_to_fill_max_identifiable_quantile: touch_censor_fraction.map(max_identifiable_quantile),
        first_touch: stage_block(&touch_refs, |row| row.gap_atr),
        fill: stage_block(&fill_refs, |row| row.gap_atr),
        orphan_touch_n: orphan_touch_n,
        orphan_fill_n: orphan_fill_n,
        n_bars,
    })
}

fn utc_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "0".into(), |duration| duration.as_secs().to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1B scanner failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut view_root = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut sidecar = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v2_scope_repair.jsonl",
    );
    let mut sidecar_manifest = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v2_scope_repair.manifest.json",
    );
    let mut output_root = PathBuf::from(".runs/ap-002-e1-native-strata/e1b-run");
    let mut scanner_commit = String::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--view-root" => view_root = args.next().ok_or("missing --view-root")?.into(),
            "--sidecar" => sidecar = args.next().ok_or("missing --sidecar")?.into(),
            "--sidecar-manifest" => {
                sidecar_manifest = args.next().ok_or("missing --sidecar-manifest")?.into()
            }
            "--output-root" => output_root = args.next().ok_or("missing --output-root")?.into(),
            "--scanner-commit" => scanner_commit = args.next().ok_or("missing --scanner-commit")?,
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if scanner_commit.len() != 40 || !scanner_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--scanner-commit must be a 40-character Git SHA".into());
    }
    fs::create_dir_all(&output_root)?;
    let view_manifest_bytes = fs::read(view_root.join("view_manifest.json"))?;
    let view_manifest: ViewManifest = serde_json::from_slice(&view_manifest_bytes)?;
    let view_manifest_hash = format!("{:x}", Sha256::digest(&view_manifest_bytes));
    let (sidecar_manifest_value, sidecar_manifest_hash, sidecar_hash) =
        verify_sidecar(&sidecar, &sidecar_manifest, &view_manifest_hash)?;
    let trigger_sequences = sidecar_trigger_sequences(&sidecar)?;
    let (preflight, anchors) = run_preflight(
        &view_root,
        &view_manifest,
        &view_manifest_hash,
        &scanner_commit,
        trigger_sequences,
    )?;
    write_json(&output_root.join("PREFLIGHT.json"), &preflight)?;
    if preflight.regressions_count != 0 {
        return Err(format!(
            "receipt-time regressions detected: {}",
            preflight.regressions_count
        )
        .into());
    }
    if preflight.rows_scanned != sidecar_manifest_value.input_rows {
        return Err(format!(
            "preflight rows {} do not match sidecar input rows {}",
            preflight.rows_scanned, sidecar_manifest_value.input_rows
        )
        .into());
    }
    let provenance = Provenance {
        scope_id: view_manifest.scope_id.clone(),
        development_view_manifest_sha256: view_manifest_hash.clone(),
        sidecar_manifest_sha256: sidecar_manifest_hash.clone(),
        sidecar_sha256: sidecar_hash.clone(),
        source_parts: Vec::new(),
        scanner_commit: scanner_commit.clone(),
    };
    let anchors_path = output_root.join("E1B_ANCHORS.jsonl");
    let mut anchor_writer = AnchorWriter::new(&anchors_path)?;
    let mut strata = Vec::new();
    for timeframe in TIMEFRAMES {
        strata.push(scan_stratum(
            &view_root,
            &view_manifest,
            &sidecar,
            timeframe,
            &anchors,
            &mut anchor_writer,
        )?);
    }
    let (anchor_n, anchor_sha256, anchor_logical_hash) = anchor_writer.finish()?;
    let table_provenance = Provenance {
        source_parts: TIMEFRAMES
            .iter()
            .flat_map(|timeframe| source_parts(&view_manifest, timeframe).unwrap_or_default())
            .map(|part| part.logical_path.clone())
            .collect(),
        ..provenance.clone()
    };
    let strata_table = TableEnvelope {
        n: strata.len() as u64,
        provenance: table_provenance.clone(),
        rows: strata,
    };
    let logical_input = serde_json::to_vec(&(
        view_manifest.logical_view_identity,
        &strata_table,
        &anchor_logical_hash,
    ))?;
    let logical_result_hash = format!("{:x}", Sha256::digest(logical_input));
    let results = Results {
        experiment_id: "AP-002-FVG-E1B-STAGE-ANCHORS",
        scope_id: view_manifest.scope_id,
        exposure_class: "E1_PHENOTYPE",
        confirmation_status: "LOCKED",
        scanner_commit: scanner_commit.clone(),
        preflight_hash: preflight.deterministic_preflight_hash.clone(),
        sidecar_manifest_sha256: sidecar_manifest_hash,
        sidecar_rows: sidecar_manifest_value.sidecar_rows,
        logical_result_hash: logical_result_hash.clone(),
        tables: ResultsTables {
            strata: strata_table,
            anchors: AnchorTableRef {
                n: anchor_n,
                path: "E1B_ANCHORS.jsonl".into(),
                absolute_path: fs::canonicalize(&anchors_path)?.display().to_string(),
                sha256: anchor_sha256,
                logical_rows_sha256: anchor_logical_hash,
                provenance: table_provenance,
            },
        },
    };
    let experiment = json!({
        "experiment_id": "AP-002-FVG-E1B-STAGE-ANCHORS",
        "objective": "Measure FIRST_TOUCH and FILL stage-anchored lifecycle and prospective behavior of native FVG zones.",
        "instrument": "XAUUSD",
        "development_scope": "XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only",
        "exposure_class": "E1_PHENOTYPE",
        "confirmation_status": "LOCKED",
        "contract": "docs/research/ap-002-e1b/AP-002_E1B_EXPERIMENT_CONTRACT.json",
        "native_strata": TIMEFRAMES,
        "cross_scale_identity": "PROHIBITED",
        "stage_anchors": ["FIRST_TOUCH", "FILL"],
        "response_horizons_bars": HORIZONS_BARS,
    });
    write_json(&output_root.join("E1B_EXPERIMENT.json"), &experiment)?;
    write_json(&output_root.join("E1B_RESULTS.json"), &results)?;
    let contract_hash = sha256_file(Path::new(
        "docs/research/ap-002-e1b/AP-002_E1B_EXPERIMENT_CONTRACT.json",
    ))?;
    let run_manifest = json!({
        "run_id": output_root.file_name().and_then(|value| value.to_str()).unwrap_or("run"),
        "experiment_id": "AP-002-FVG-E1B-STAGE-ANCHORS",
        "researcher_id": "AP-002-E1B-STAGE-ANCHORS-TOOLSMITH",
        "lab": "TOOLSMITH",
        "code_identity": scanner_commit,
        "contract_hash": contract_hash,
        "input_identity": view_manifest_hash,
        "sidecar_identity": sidecar_hash,
        "executed_utc": utc_seconds(),
        "output_identity": logical_result_hash,
        "confirmation_status": "LOCKED",
    });
    write_json(&output_root.join("RUN_MANIFEST.json"), &run_manifest)?;
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
