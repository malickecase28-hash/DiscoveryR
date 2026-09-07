//! AP-001 WP2 lifecycle and native-market executor.
//!
//! This module deliberately keeps WP2 separate from the WP1 gate scanner.  It
//! reuses WP1's payload parser and frozen development cutoff, then performs a
//! second, bounded pass over prices for the six native tick horizons and the
//! separately reported seven-scale bar bridge.

use crate::ap001_drift_burst::{self, DState, EmittedTruth, ParsedPayload, Snapshot};
use arrow_array::{Array, Float64Array, Int64Array, LargeStringArray};
use parquet::arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ProjectionMask};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

pub const CONTRACT_DEFAULT: &str = r"F:\TrinityR-research\Research Program\AP-001\R2_LIFECYCLE_MARKET\AP-001_WP2_CONTRACT.json";
pub const OUTPUT_NAMES: [&str; 7] = [
    "AP-001_WP2_ANCHORS.json",
    "AP-001_WP2_LIFECYCLE_ATLAS.json",
    "AP-001_WP2_MARKET_PATHS.json",
    "AP-001_WP2_INTERNAL_SPLITS.json",
    "AP-001_WP2_SUPPORT_REFERENCE.json",
    "AP-001_WP2_E1_REPORT.md",
    "AP-001_WP2_PREFLIGHT.json",
];
const HORIZONS: [i64; 6] = [5, 15, 30, 60, 120, 300];
const BAR_SCALES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const BAR_HORIZONS: [usize; 4] = [1, 2, 3, 5];

#[derive(Debug, Clone)]
pub struct Wp2Config {
    pub contract_path: PathBuf,
    pub lake_root: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub bar_view_root: PathBuf,
    pub sidecar_path: PathBuf,
    pub sidecar_manifest_path: PathBuf,
    pub batch_size: usize,
    pub single_run: bool,
}

impl Default for Wp2Config {
    fn default() -> Self {
        Self {
            contract_path: PathBuf::from(CONTRACT_DEFAULT),
            lake_root: None,
            out_dir: None,
            bar_view_root: PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development-superseded-16x00"),
            sidecar_path: PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v1.jsonl"),
            sidecar_manifest_path: PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v1.manifest.json"),
            batch_size: 16_384,
            single_run: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    ts_ms: i64,
    event_ts_ns: i64,
    received_ts_ns: Option<i64>,
    source_sequence: Option<i64>,
    record: Value,
    direction: f64,
    bid: f64,
    ask: f64,
}

#[derive(Debug, Clone)]
struct Anchor {
    anchor_id: String,
    episode_id: i64,
    state: &'static str,
    event_ts_ns: i64,
    received_ts_ns: Option<i64>,
    source_sequence: Option<i64>,
    bid: f64,
    ask: f64,
    mid: f64,
    direction: f64,
    left_truncated: bool,
    record: Value,
}

#[derive(Debug, Clone)]
struct StatePeriod {
    state: &'static str,
    entry_event_ts_ns: i64,
    exit_event_ts_ns: Option<i64>,
    rows: u64,
    reentry: bool,
}

#[derive(Debug, Clone)]
struct Episode {
    id: i64,
    left_truncated: bool,
    start_event_ts_ns: i64,
    last_event_ts_ns: i64,
    last_state: DState,
    rows: u64,
    transitions: Vec<String>,
    states: Vec<StatePeriod>,
    anchors: Vec<String>,
    candidate_weak: Option<Candidate>,
    candidate_online: Option<Candidate>,
    weak_anchor: Option<Anchor>,
    online_anchor: Option<Anchor>,
    strong_anchor: Option<Anchor>,
    decay_anchor: Option<Anchor>,
    dying_anchor: Option<Anchor>,
    closed: bool,
    censored: bool,
}

impl Episode {
    fn new(id: i64, left_truncated: bool, tick_ns: i64, _state: DState) -> Self {
        Self {
            id,
            left_truncated,
            start_event_ts_ns: tick_ns,
            last_event_ts_ns: tick_ns,
            last_state: DState::Idle,
            rows: 0,
            transitions: Vec::new(),
            states: Vec::new(),
            anchors: Vec::new(),
            candidate_weak: None,
            candidate_online: None,
            weak_anchor: None,
            online_anchor: None,
            strong_anchor: None,
            decay_anchor: None,
            dying_anchor: None,
            closed: false,
            censored: false,
        }
    }
}

#[derive(Default)]
pub struct Capture {
    active: Option<Episode>,
    episodes: BTreeMap<i64, Episode>,
    emitted: BTreeMap<i64, EmittedTruth>,
    anchors: Vec<Anchor>,
    state_rows: u64,
    pseudo_rows: Vec<Anchor>,
    left_truncated: u64,
    forming_fizzled: u64,
    right_censored: u64,
    unknown_states: u64,
    transition_counts: BTreeMap<(String, String), u64>,
}

impl Capture {
    pub fn anchor_counts(&self) -> BTreeMap<&'static str, usize> {
        let mut counts = BTreeMap::new();
        for state in ["WEAK", "ONLINE", "STRONG", "DECAY_RISK", "DYING"] {
            counts.insert(state, self.anchors.iter().filter(|a| a.state == state).count());
        }
        counts
    }

    pub fn episode_count(&self) -> usize {
        self.episodes.len() + usize::from(self.active.is_some())
    }

    pub fn observe(
        &mut self,
        tick_ns: i64,
        received_ts_ns: Option<i64>,
        source_sequence: Option<i64>,
        bid: f64,
        ask: f64,
        payload: &str,
        snap: &Snapshot,
    ) {
        if snap.state == DState::Other {
            self.unknown_states += 1;
        }
        for entry in &snap.emissions {
            self.emitted.insert(entry.truth.event_id, entry.truth.clone());
        }
        if snap.state == DState::Idle {
            if let Some(mut active) = self.active.take() {
                active.closed = true;
                active.last_event_ts_ns = tick_ns;
                self.episodes.insert(active.id, active);
            }
            return;
        }
        self.state_rows += 1;
        let eid = snap.eid_pos;
        let mut active = match self.active.take() {
            Some(mut current) => {
                if current.id >= 0 && eid.is_none() {
                    current.closed = true;
                    self.episodes.insert(current.id, current);
                    Episode::new(-1, false, tick_ns, snap.state)
                } else if let Some(new) = eid {
                    if current.id >= 0 && current.id != new {
                        current.closed = true;
                        self.episodes.insert(current.id, current);
                        Episode::new(new, false, tick_ns, snap.state)
                    } else {
                        if current.id < 0 {
                            current.id = new;
                        }
                        current
                    }
                } else {
                    current
                }
            }
            None => {
                let id = eid.unwrap_or(-1);
                let left = self.state_rows == 1;
                if left {
                    self.left_truncated += 1;
                }
                Episode::new(id, left, tick_ns, snap.state)
            }
        };
        if active.id < 0 {
            if let Some(id) = eid {
                active.id = id;
            }
        }
        active.rows += 1;
        active.last_event_ts_ns = tick_ns;
        let state = snap.state;
        let previous = active.last_state;
        if previous != state {
            let from = previous.label().to_owned();
            let to = state.label().to_owned();
            *self.transition_counts.entry((from.clone(), to.clone())).or_default() += 1;
            active.transitions.push(to);
            if let Some(last) = active.states.last_mut() {
                last.exit_event_ts_ns = Some(tick_ns);
            }
            active.states.push(StatePeriod {
                state: state.label(),
                entry_event_ts_ns: tick_ns,
                exit_event_ts_ns: None,
                rows: 0,
                reentry: active.states.iter().any(|p| p.state == state.label()),
            });
        }
        if active.states.is_empty() {
            active.states.push(StatePeriod {
                state: state.label(),
                entry_event_ts_ns: tick_ns,
                exit_event_ts_ns: None,
                rows: 0,
                reentry: false,
            });
        }
        if let Some(last) = active.states.last_mut() {
            last.rows += 1;
        }
        let state_anchor = (state == DState::Burst && previous != DState::Burst && active.strong_anchor.is_none())
            || (state == DState::DecayRisk && active.decay_anchor.is_none())
            || (state == DState::Dying && active.dying_anchor.is_none());
        let record_needed = state_anchor
            || candidate_changed(active.candidate_weak.as_ref(), snap.weak_ms)
            || candidate_changed(active.candidate_online.as_ref(), snap.online_ms);
        let dbs = if record_needed {
            serde_json::from_str::<Value>(payload)
                .ok()
                .and_then(|raw| raw.get("drift_burst_state").cloned())
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        let anchor_record = lawful_snapshot(&dbs);
        let direction = if record_needed { direction_from(&dbs) } else { 1.0 };
        update_candidate(
            &mut active.candidate_weak,
            snap.weak_ms,
            tick_ns,
            received_ts_ns,
            source_sequence,
            &anchor_record,
            direction,
            bid,
            ask,
        );
        update_candidate(
            &mut active.candidate_online,
            snap.online_ms,
            tick_ns,
            received_ts_ns,
            source_sequence,
            &anchor_record,
            direction,
            bid,
            ask,
        );
        if state == DState::Burst && previous != DState::Burst && active.strong_anchor.is_none() {
            let strong = make_anchor(
                &active,
                "STRONG",
                tick_ns,
                received_ts_ns,
                source_sequence,
                bid,
                ask,
                direction,
                anchor_record.clone(),
            );
            active.strong_anchor = Some(strong.clone());
            active.anchors.push(strong.anchor_id.clone());
            self.anchors.push(strong);
            if let Some(candidate) = active.candidate_weak.take() {
                let anchor = candidate_anchor(&active, "WEAK", candidate);
                active.anchors.push(anchor.anchor_id.clone());
                active.weak_anchor = Some(anchor.clone());
                self.anchors.push(anchor);
            }
            if let Some(candidate) = active.candidate_online.take() {
                let anchor = candidate_anchor(&active, "ONLINE", candidate);
                active.anchors.push(anchor.anchor_id.clone());
                active.online_anchor = Some(anchor.clone());
                self.anchors.push(anchor);
            }
        }
        if state == DState::DecayRisk && active.decay_anchor.is_none() {
            let anchor = make_anchor(&active, "DECAY_RISK", tick_ns, received_ts_ns, source_sequence, bid, ask, direction, anchor_record.clone());
            active.anchors.push(anchor.anchor_id.clone());
            active.decay_anchor = Some(anchor.clone());
            self.anchors.push(anchor);
        }
        if state == DState::Dying && active.dying_anchor.is_none() {
            let anchor = make_anchor(&active, "DYING", tick_ns, received_ts_ns, source_sequence, bid, ask, direction, anchor_record.clone());
            active.anchors.push(anchor.anchor_id.clone());
            active.dying_anchor = Some(anchor.clone());
            self.anchors.push(anchor);
        }
        active.last_state = state;
        self.active = Some(active);
        if self.state_rows % 500 == 0 {
            let mid = (bid + ask) / 2.0;
            self.pseudo_rows.push(Anchor {
                anchor_id: format!("baseline-{}", self.state_rows),
                episode_id: -1,
                state: "BASELINE",
                event_ts_ns: tick_ns,
                received_ts_ns,
                source_sequence,
                bid,
                ask,
                mid,
                direction: 1.0,
                left_truncated: false,
                record: Value::Null,
            });
        }
    }

    pub fn finish(&mut self) {
        if let Some(mut active) = self.active.take() {
            active.censored = true;
            self.right_censored += 1;
            if active.id >= 0 {
                self.episodes.insert(active.id, active);
            } else {
                self.forming_fizzled += 1;
            }
        }
        self.anchors.sort_by(|a, b| {
            (a.received_ts_ns.unwrap_or(i64::MAX), a.source_sequence.unwrap_or(i64::MAX), &a.anchor_id)
                .cmp(&(b.received_ts_ns.unwrap_or(i64::MAX), b.source_sequence.unwrap_or(i64::MAX), &b.anchor_id))
        });
    }
}

fn candidate_changed(current: Option<&Candidate>, next: Option<i64>) -> bool {
    match (current, next) {
        (None, None) => false,
        (Some(_), None) => true,
        (None, Some(_)) => true,
        (Some(current), Some(next)) => current.ts_ms != next,
    }
}

fn update_candidate(
    slot: &mut Option<Candidate>,
    ts_ms: Option<i64>,
    event_ts_ns: i64,
    received_ts_ns: Option<i64>,
    source_sequence: Option<i64>,
    dbs: &Value,
    direction: f64,
    bid: f64,
    ask: f64,
) {
    match (slot.as_ref(), ts_ms) {
        (_, None) => *slot = None,
        (Some(current), Some(value)) if current.ts_ms == value => {}
        (_, Some(value)) => {
            *slot = Some(Candidate {
                ts_ms: value,
                event_ts_ns,
                received_ts_ns,
                source_sequence,
                record: dbs.clone(),
                direction,
                bid,
                ask,
            });
        }
    }
}

fn lawful_snapshot(dbs: &Value) -> Value {
    json!({
        "state": dbs.get("state"),
        "event_id": dbs.get("event_id"),
        "milestones": dbs.get("milestones"),
        "operational": dbs.get("operational"),
        "multiscale": dbs.get("multiscale"),
        "authority": dbs.get("authority"),
    })
}

fn candidate_anchor(active: &Episode, state: &'static str, c: Candidate) -> Anchor {
    let id = active.id;
    Anchor {
        anchor_id: format!("{id}:{state}"),
        episode_id: id,
        state,
        event_ts_ns: c.event_ts_ns,
        received_ts_ns: c.received_ts_ns,
        source_sequence: c.source_sequence,
        bid: c.bid,
        ask: c.ask,
        mid: (c.bid + c.ask) / 2.0,
        direction: c.direction,
        left_truncated: active.left_truncated,
        record: c.record,
    }
}

fn make_anchor(
    active: &Episode,
    state: &'static str,
    event_ts_ns: i64,
    received_ts_ns: Option<i64>,
    source_sequence: Option<i64>,
    bid: f64,
    ask: f64,
    direction: f64,
    record: Value,
) -> Anchor {
    Anchor {
        anchor_id: format!("{}:{state}", active.id),
        episode_id: active.id,
        state,
        event_ts_ns,
        received_ts_ns,
        source_sequence,
        bid,
        ask,
        mid: (bid + ask) / 2.0,
        direction,
        left_truncated: active.left_truncated,
        record,
    }
}

fn direction_from(dbs: &Value) -> f64 {
    let candidates = [
        dbs.pointer("/operational/direction"),
        dbs.pointer("/direction"),
        dbs.pointer("/multiscale/5s/direction"),
    ];
    for value in candidates.into_iter().flatten() {
        if let Some(number) = value.as_f64() {
            if number != 0.0 {
                return number.signum();
            }
        }
        if let Some(text) = value.as_str() {
            let upper = text.to_ascii_uppercase();
            if upper.contains("DOWN") || upper == "-1" {
                return -1.0;
            }
            if upper.contains("UP") || upper == "1" {
                return 1.0;
            }
        }
    }
    1.0
}

#[derive(Debug, Clone)]
struct Tick {
    avail_ns: i64,
    seq: i64,
    mid: f64,
}

#[derive(Debug, Clone, Default)]
struct Horizon {
    end_ns: i64,
    covered_ns: i64,
    complete: bool,
    last_mid: Option<f64>,
    max_fav: Option<(f64, i64)>,
    min_adv: Option<(f64, i64)>,
    first_favorable_ms: Option<i64>,
    first_adverse_ms: Option<i64>,
    tie: bool,
    abs_sum: f64,
    rv: f64,
    bucket: Option<i64>,
    bucket_first: Option<f64>,
    bucket_last: Option<f64>,
}

#[derive(Debug, Clone)]
struct Pending {
    anchor: Anchor,
    horizons: Vec<Horizon>,
    last_tick: Option<Tick>,
}

impl Pending {
    fn new(anchor: Anchor) -> Self {
        let start = anchor.received_ts_ns.unwrap_or(anchor.event_ts_ns);
        Self {
            anchor,
            horizons: HORIZONS.iter().map(|h| Horizon { end_ns: start.saturating_add(h * 1_000_000_000), ..Default::default() }).collect(),
            last_tick: None,
        }
    }
    fn update(&mut self, tick: &Tick) {
        let start = self.anchor.received_ts_ns.unwrap_or(self.anchor.event_ts_ns);
        let key_after = tick.avail_ns > start || (tick.avail_ns == start && tick.seq > self.anchor.source_sequence.unwrap_or(i64::MIN));
        if !key_after {
            return;
        }
        let previous = self.last_tick.as_ref().map(|t| t.mid).unwrap_or(self.anchor.mid);
        for h in &mut self.horizons {
            if h.complete || tick.avail_ns > h.end_ns {
                if !h.complete {
                    finalize_bucket(h);
                    h.covered_ns = h.last_mid.as_ref().map(|_| h.end_ns.saturating_sub(start)).unwrap_or(0);
                    h.complete = true;
                }
                continue;
            }
            let signed = self.anchor.direction * (tick.mid / self.anchor.mid).ln();
            let offset_ms = tick.avail_ns.saturating_sub(start) / 1_000_000;
            h.covered_ns = tick.avail_ns.saturating_sub(start).min(h.end_ns.saturating_sub(start));
            h.last_mid = Some(tick.mid);
            h.max_fav = Some(h.max_fav.map_or((signed, offset_ms), |x| if signed > x.0 { (signed, offset_ms) } else { x }));
            h.min_adv = Some(h.min_adv.map_or((signed, offset_ms), |x| if signed < x.0 { (signed, offset_ms) } else { x }));
            if signed > 0.0 && h.first_favorable_ms.is_none() { h.first_favorable_ms = Some(offset_ms); }
            if signed < 0.0 && h.first_adverse_ms.is_none() { h.first_adverse_ms = Some(offset_ms); }
            if signed == 0.0 { h.tie = true; }
            h.abs_sum += (tick.mid / previous).ln().abs();
            let bucket = tick.avail_ns.div_euclid(1_000_000_000);
            match h.bucket {
                Some(old) if old != bucket => { finalize_bucket(h); h.bucket = Some(bucket); h.bucket_first = Some(previous); h.bucket_last = Some(tick.mid); }
                Some(_) => h.bucket_last = Some(tick.mid),
                None => { h.bucket = Some(bucket); h.bucket_first = Some(previous); h.bucket_last = Some(tick.mid); }
            }
            if tick.avail_ns >= h.end_ns {
                finalize_bucket(h);
                h.complete = true;
            }
        }
        self.last_tick = Some(tick.clone());
    }
}

fn finalize_bucket(h: &mut Horizon) {
    if let (Some(first), Some(last)) = (h.bucket_first.take(), h.bucket_last.take()) {
        if first > 0.0 && last > 0.0 {
            h.rv += (last / first).ln().powi(2);
        }
    }
    h.bucket = None;
}

fn path_value(p: &Pending, h: &Horizon, horizon_s: i64) -> Value {
    let start = p.anchor.received_ts_ns.unwrap_or(p.anchor.event_ts_ns);
    let endpoint = h.last_mid.map(|mid| (mid / p.anchor.mid).ln());
    let favorable = h.max_fav.map(|(v, _)| v);
    let adverse = h.min_adv.map(|(v, _)| v);
    json!({
        "anchor_id": p.anchor.anchor_id,
        "anchor_state": p.anchor.state,
        "horizon_seconds": horizon_s,
        "endpoint_return_log": endpoint,
        "signed_return_log": endpoint.map(|v| p.anchor.direction * v),
        "direction_adjusted_return": endpoint.map(|v| p.anchor.direction * v),
        "mfe": favorable.map(|v| v.max(0.0)),
        "mae": adverse.map(|v| (-v).max(0.0)),
        "mfe_offset_ms": h.max_fav.map(|(_, o)| o),
        "mae_offset_ms": h.min_adv.map(|(_, o)| o),
        "favorable_before_adverse": h.first_favorable_ms.zip(h.first_adverse_ms).map(|(f,a)| f<a),
        "adverse_before_favorable": h.first_favorable_ms.zip(h.first_adverse_ms).map(|(f,a)| a<f),
        "tie": h.tie,
        "continuation_distance": favorable.map(|v| v.max(0.0)),
        "reversal_distance": adverse.map(|v| (-v).max(0.0)),
        "realized_variance": h.rv,
        "path_efficiency": endpoint.and_then(|v| (h.abs_sum > 0.0).then_some(v/h.abs_sum)),
        "covered_ns": h.covered_ns,
        "complete": h.complete,
        "censor_reason": if h.complete { Value::Null } else { json!("DEVELOPMENT_BOUNDARY") },
        "anchor_start_ns": start,
    })
}

struct SerializedPath {
    bytes: Vec<u8>,
}

fn serialize_path(
    p: &Pending,
    h: &Horizon,
    horizon_s: i64,
) -> Result<SerializedPath, serde_json::Error> {
    let value = path_value(p, h, horizon_s);
    Ok(SerializedPath { bytes: serde_json::to_vec(&value)? })
}

struct PathSink {
    writer: BufWriter<File>,
    first: bool,
    count: usize,
}

impl PathSink {
    fn create(path: &Path) -> std::io::Result<Self> {
        let mut writer = BufWriter::new(File::create(path)?);
        writer.write_all(b"[")?;
        Ok(Self { writer, first: true, count: 0 })
    }

    fn push(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if !self.first { self.writer.write_all(b",")?; }
        self.writer.write_all(bytes)?;
        self.first = false;
        self.count += 1;
        Ok(())
    }

    fn finish(mut self) -> std::io::Result<usize> {
        self.writer.write_all(b"]")?;
        self.writer.flush()?;
        Ok(self.count)
    }
}

#[derive(Default)]
struct BaselineSummary {
    values: [Vec<f64>; 6],
    count: usize,
}

struct MarketOutput {
    native_paths: PathBuf,
    native_path_count: usize,
    baseline_path_count: usize,
    baseline_values: [Vec<f64>; 6],
}

fn run_market_scan(parts: &[PathBuf], cutoff: i64, anchors: &[Anchor], pseudo: &[Anchor], batch_size: usize, native_path: &Path) -> Result<MarketOutput, Box<dyn std::error::Error>> {
    let mut all = anchors.to_vec();
    all.sort_by_key(|a| (a.received_ts_ns.unwrap_or(a.event_ts_ns), a.source_sequence.unwrap_or(i64::MIN), a.anchor_id.clone()));
    let mut base = pseudo.to_vec();
    base.sort_by_key(|a| (a.received_ts_ns.unwrap_or(a.event_ts_ns), a.source_sequence.unwrap_or(i64::MIN), a.anchor_id.clone()));
    let mut normal_i = 0usize;
    let mut base_i = 0usize;
    let mut pending = VecDeque::new();
    let mut baseline_pending = VecDeque::new();
    let mut sink = PathSink::create(native_path)?;
    let mut baseline = BaselineSummary::default();
    for path in parts {
        let file = File::open(path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let schema = builder.schema().clone();
        let indices = ["event_ts_ns", "received_ts_ns", "source_sequence", "bid", "ask"]
            .iter().map(|n| schema.index_of(n)).collect::<Result<Vec<_>, _>>()?;
        let mask = ProjectionMask::roots(builder.parquet_schema(), indices);
        let reader = builder.with_projection(mask).with_batch_size(batch_size).build()?;
        for batch in reader {
            let batch = batch?;
            let event = batch.column(0).as_any().downcast_ref::<Int64Array>().ok_or("event_ts_ns")?;
            let recv = batch.column(1).as_any().downcast_ref::<Int64Array>().ok_or("received_ts_ns")?;
            let seq = batch.column(2).as_any().downcast_ref::<Int64Array>().ok_or("source_sequence")?;
            let bid = batch.column(3).as_any().downcast_ref::<Float64Array>().ok_or("bid")?;
            let ask = batch.column(4).as_any().downcast_ref::<Float64Array>().ok_or("ask")?;
            for row in 0..batch.num_rows() {
                if recv.is_null(row) || recv.value(row) >= cutoff || event.is_null(row) || bid.is_null(row) || ask.is_null(row) { continue; }
                let tick = Tick { avail_ns: recv.value(row), seq: if seq.is_null(row) { i64::MIN } else { seq.value(row) }, mid: (bid.value(row)+ask.value(row))/2.0 };
                while normal_i < all.len() && key_le(&all[normal_i], &tick) { pending.push_back(Pending::new(all[normal_i].clone())); normal_i += 1; }
                while base_i < base.len() && key_le(&base[base_i], &tick) { baseline_pending.push_back(Pending::new(base[base_i].clone())); base_i += 1; }
                for p in &mut pending { p.update(&tick); }
                for p in &mut baseline_pending { p.update(&tick); }
                drain_pending(&mut pending, &mut sink)?;
                drain_baseline(&mut baseline_pending, &mut baseline);
            }
        }
    }
    while let Some(p) = pending.pop_front() { finish_pending(p, &mut sink)?; }
    while let Some(p) = baseline_pending.pop_front() { record_baseline(p, &mut baseline); }
    let native_path_count = sink.finish()?;
    Ok(MarketOutput { native_paths: native_path.to_owned(), native_path_count, baseline_path_count: baseline.count, baseline_values: baseline.values })
}

fn key_le(anchor: &Anchor, tick: &Tick) -> bool {
    let a = (anchor.received_ts_ns.unwrap_or(anchor.event_ts_ns), anchor.source_sequence.unwrap_or(i64::MIN));
    (a.0, a.1) <= (tick.avail_ns, tick.seq)
}

fn drain_pending(
    pending: &mut VecDeque<Pending>,
    sink: &mut PathSink,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let complete = pending.front().map(|p| p.horizons.iter().all(|h| h.complete)).unwrap_or(false);
        if !complete { break; }
        let p = pending.pop_front().expect("complete pending path");
        for (h, seconds) in p.horizons.iter().zip(HORIZONS) {
            let path = serialize_path(&p, h, seconds)?;
            sink.push(&path.bytes)?;
        }
    }
    Ok(())
}

fn drain_baseline(pending: &mut VecDeque<Pending>, baseline: &mut BaselineSummary) {
    loop {
        let complete = pending.front().map(|p| p.horizons.iter().all(|h| h.complete)).unwrap_or(false);
        if !complete { break; }
        let p = pending.pop_front().expect("complete baseline path");
        record_baseline(p, baseline);
    }
}

fn finish_pending(
    mut p: Pending,
    sink: &mut PathSink,
) -> Result<(), Box<dyn std::error::Error>> {
    for h in &mut p.horizons { finalize_bucket(h); }
    for (h, seconds) in p.horizons.iter().zip(HORIZONS) {
        let path = serialize_path(&p, h, seconds)?;
        sink.push(&path.bytes)?;
    }
    Ok(())
}

fn record_baseline(mut p: Pending, baseline: &mut BaselineSummary) {
    for h in &mut p.horizons { finalize_bucket(h); }
    for (index, (h, seconds)) in p.horizons.iter().zip(HORIZONS).enumerate() {
        if let Some(value) = path_value(&p, h, seconds).get("direction_adjusted_return").and_then(Value::as_f64) {
            baseline.values[index].push(value);
        }
        baseline.count += 1;
    }
}

fn sha256_file(path: &Path) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buf = [0u8; 65_536];
    loop { let n = file.read(&mut buf)?; if n == 0 { break; } bytes += n as u64; hasher.update(&buf[..n]); }
    Ok((format!("{:x}", hasher.finalize()), bytes))
}

fn canonical_hash(value: &Value) -> String {
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(value).unwrap_or_default());
    format!("{:x}", h.finalize())
}

fn write_json(path: &Path, value: &Value) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, &bytes)?;
    let mut h = Sha256::new(); h.update(&bytes);
    Ok((format!("{:x}", h.finalize()), bytes.len() as u64))
}

fn anchor_value(a: &Anchor) -> Value {
    json!({
        "anchor_id": a.anchor_id,
        "episode_id": a.episode_id,
        "anchor_state": a.state,
        "availability": {"received_ts_ns": a.received_ts_ns, "source_sequence": a.source_sequence},
        "provenance": {"event_ts_ns": a.event_ts_ns},
        "bid": a.bid, "ask": a.ask, "mid": a.mid,
        "direction": a.direction,
        "left_truncated": a.left_truncated,
        "internal_state": a.record,
    })
}

fn lifecycle_value(capture: &Capture) -> Value {
    let episodes = capture.episodes.values().filter(|e| e.id >= 0).map(|e| {
        json!({
            "episode_id": e.id,
            "left_truncated": e.left_truncated,
            "origin": {"event_ts_ns": e.start_event_ts_ns},
            "last_observed_event_ts_ns": e.last_event_ts_ns,
            "state_sequence": e.transitions,
            "state_periods": e.states.iter().map(|p| json!({"state":p.state,"entry_event_ts_ns":p.entry_event_ts_ns,"exit_event_ts_ns":p.exit_event_ts_ns,"rows":p.rows,"reentry":p.reentry})).collect::<Vec<_>>(),
            "anchor_ids": e.anchors,
            "termination": {"closed": e.closed, "censor_status": if e.censored {"RIGHT_CENSORED_EPISODE"} else {"COMPLETE"}},
            "rows": e.rows,
            "emitted_outcome": capture.emitted.get(&e.id).map(truth_value),
        })
    }).collect::<Vec<_>>();
    json!({
        "artifact_type": "AP-001_WP2_LIFECYCLE_ATLAS",
        "episodes": episodes,
        "population": {"episodes": capture.episodes.len(), "state_rows": capture.state_rows, "left_truncated": capture.left_truncated, "right_censored": capture.right_censored, "forming_fizzled": capture.forming_fizzled},
        "first_entry_transitions": capture.transition_counts.iter().map(|((from,to),n)| json!({"from":from,"to":to,"count":n})).collect::<Vec<_>>(),
        "status": "DISCOVERY_ONLY"
    })
}

fn truth_value(t: &EmittedTruth) -> Value {
    json!({"event_id":t.event_id,"origin_ts":t.origin_ts,"weak_ts":t.weak_ts,"online_ts":t.online_ts,"strong_ts":t.strong_ts,"decay_risk_ts":t.decay_risk_ts,"first_dying_ts":t.first_dying_ts,"end_ts":t.end_ts})
}

fn attribute_value(record: &Value, pointer: &str) -> Option<f64> { record.pointer(pointer).and_then(Value::as_f64) }

fn splits_value(anchors: &[Anchor]) -> Value {
    let attrs = ["operational/z","operational/score","operational/efficiency","operational/var_rate","operational/cabs","operational/update_count","operational/duration_s"];
    let mut edges = Map::new(); let mut cells = Vec::new();
    for attr in attrs {
        let pointer = format!("/{attr}");
        let mut vals = anchors.iter().filter_map(|a| attribute_value(&a.record,&pointer)).collect::<Vec<_>>(); vals.sort_by(f64::total_cmp);
        let e = [0,20,40,60,80,100].map(|q| if vals.is_empty() { Value::Null } else { json!(vals[((q as usize)*(vals.len().saturating_sub(1))/100).min(vals.len()-1)]) });
        edges.insert(attr.to_owned(), json!(e));
        for (band,lo,hi) in [(0,0,20),(1,20,40),(2,40,60),(3,60,80),(4,80,100)].into_iter() {
            let selected = anchors.iter().filter(|a| attribute_value(&a.record,&pointer).is_some_and(|v| { let rank=vals.partition_point(|x| *x < v)*100/vals.len().max(1); rank>=lo && rank<hi })).collect::<Vec<_>>();
            cells.push(json!({"attribute":attr,"band":band,"support":selected.len(),"anchor_ids":selected.iter().map(|a| a.anchor_id.as_str()).collect::<Vec<_>>() }));
        }
    }
    json!({"artifact_type":"AP-001_WP2_INTERNAL_SPLITS","quintile_edges":edges,"cells":cells,"categorical":{"anchor_state":anchors.iter().fold(BTreeMap::<String,u64>::new(),|mut m,a|{*m.entry(a.state.to_owned()).or_default()+=1;m}),"left_truncated":anchors.iter().filter(|a|a.left_truncated).count()},"status":"DISCOVERY_ONLY"})
}

fn support_reference(baseline: &mut [Vec<f64>; 6]) -> Value {
    let mut rows = Vec::new();
    for (h, vals) in HORIZONS.into_iter().zip(baseline.iter_mut()) {
        vals.sort_by(f64::total_cmp);
        let q = |p: usize| -> Option<f64> { vals.get(p*(vals.len().saturating_sub(1))/100).copied() };
        rows.push(json!({"horizon_seconds":h,"effective_observations":vals.len(),"median":q(50),"q25":q(25),"q75":q(75),"iqr":q(75).zip(q(25)).map(|(a,b)|a-b),"status":if vals.len()<150{"INSUFFICIENT_SUPPORT"}else if vals.len()<300{"LOW_SUPPORT_FALLBACK"}else{"SUPPORTED"}}));
    }
    json!({"artifact_type":"AP-001_WP2_SUPPORT_REFERENCE","baseline_paths":"pseudo-anchors every 500th development state-bearing row","rows":rows,"materiality_rule":"abs(median conditional effect - same-horizon baseline median) >= 0.10 * baseline_IQR(h)","support_rules":{"default_effective_observations":300,"low_support_range":"150-299","insufficient_support":"below 150","blocks_min":3,"block_definition":"UTC calendar month","dominance_max":0.5},"status":"FROZEN_BEFORE_WP3_WP4"})
}

fn anchors_json(anchors: &[Anchor]) -> Value {
    json!({"artifact_type":"AP-001_WP2_ANCHORS","anchor_count":anchors.len(),"anchors":anchors.iter().map(anchor_value).collect::<Vec<_>>(),"status":"DISCOVERY_ONLY"})
}

#[derive(Clone)]
struct BridgeBar { open_ns: i64, close_ns: i64, open: f64, high: f64, low: f64, close: f64, avail_ns: Option<i64>, seq: Option<i64> }

struct BridgeData {
    metadata: Value,
    bars: BTreeMap<String, Vec<BridgeBar>>,
}

fn load_bar_bridge(view_root: &Path, sidecar_path: &Path, cutoff: i64) -> Result<BridgeData, Box<dyn std::error::Error>> {
    let (sidecar_sha, sidecar_bytes) = sha256_file(sidecar_path)?;
    let mut records = 0u64; let mut before = 0u64; let mut by_scale = BTreeMap::new();
    let mut availability = BTreeMap::<(String, i64), (i64, i64)>::new();
    for line in BufReader::new(File::open(sidecar_path)?).lines() {
        let value: Value = serde_json::from_str(&line?)?; records += 1;
        let scale=value.get("timeframe").and_then(Value::as_str).unwrap_or("unknown").to_owned(); let close=value.get("bar_close_ts_ms").and_then(Value::as_i64).unwrap_or(-1); let avail=value.get("available_time_ns").and_then(Value::as_i64).unwrap_or(i64::MAX); let seq=value.get("trigger_source_sequence").and_then(Value::as_i64).unwrap_or(i64::MIN);
        if avail < cutoff { before += 1; *by_scale.entry(scale.clone()).or_insert(0u64) += 1; availability.insert((scale,close),(avail,seq)); }
    }
    let mut bars = BTreeMap::<String, Vec<BridgeBar>>::new(); let mut rows_by_scale=BTreeMap::new();
    for scale in BAR_SCALES { let mut rows=Vec::new(); let dir=view_root.join(scale); if dir.exists() { let mut files=std::fs::read_dir(dir)?.filter_map(Result::ok).map(|e|e.path()).filter(|p|p.extension().and_then(|e|e.to_str())==Some("parquet")).collect::<Vec<_>>(); files.sort(); for path in files { let builder=ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?; let schema=builder.schema().clone(); let names=["bar_open_ts","bar_close_ts","open","high","low","close"]; let indices=names.iter().map(|n|schema.index_of(n)).collect::<Result<Vec<_>,_>>()?; let mask=ProjectionMask::roots(builder.parquet_schema(),indices); for batch in builder.with_projection(mask).with_batch_size(16_384).build()? { let batch=batch?; let open=batch.column(0).as_any().downcast_ref::<Int64Array>().ok_or("bar_open_ts")?; let close=batch.column(1).as_any().downcast_ref::<Int64Array>().ok_or("bar_close_ts")?; let op=batch.column(2).as_any().downcast_ref::<Float64Array>().ok_or("open")?; let hi=batch.column(3).as_any().downcast_ref::<Float64Array>().ok_or("high")?; let lo=batch.column(4).as_any().downcast_ref::<Float64Array>().ok_or("low")?; let cl=batch.column(5).as_any().downcast_ref::<Float64Array>().ok_or("close")?; for i in 0..batch.num_rows() { let close_ms=close.value(i); let key=(scale.to_owned(),close_ms); let (avail,seq)=availability.get(&key).copied().map(|(a,s)|(Some(a),Some(s))).unwrap_or((None,None)); rows.push(BridgeBar{open_ns:open.value(i).saturating_mul(1_000_000),close_ns:close_ms.saturating_mul(1_000_000),open:op.value(i),high:hi.value(i),low:lo.value(i),close:cl.value(i),avail_ns:avail,seq}); } } } } rows.sort_by_key(|b|b.open_ns); rows_by_scale.insert(scale.to_owned(),rows.len()); bars.insert(scale.to_owned(),rows); }
    Ok(BridgeData { metadata: json!({"scales":BAR_SCALES,"bar_horizons":BAR_HORIZONS,"availability_coordinate":{"available_time_ns":"received_ts_ns","source_sequence":"trigger_source_sequence","bar_close_ts_ns":"temporal boundary only"},"first_fully_post_anchor_bar":"strictly post anchor event interval and strictly later verified availability key","sidecar":{"path":sidecar_path,"sha256":sidecar_sha,"bytes":sidecar_bytes,"records_total":records,"records_before_cutoff":before,"records_by_scale_before_cutoff":by_scale},"bars":{"view_root":view_root,"rows_by_scale":rows_by_scale},"status":"BRIDGE_OUTCOMES_COMPLETE_WITH_EXPLICIT_MISSINGNESS"}), bars })
}

fn bridge_outcome(anchor: &Anchor, bridge: &BridgeData, cutoff: i64) -> Value {
    let anchor_key = (anchor.received_ts_ns.unwrap_or(anchor.event_ts_ns), anchor.source_sequence.unwrap_or(i64::MIN));
    let mut scale_out = Map::new();
    for scale in BAR_SCALES {
        let empty = Vec::new();
        let rows = bridge.bars.get(scale).unwrap_or(&empty);
        let mut i = rows.partition_point(|bar| bar.open_ns <= anchor.event_ts_ns);
        while i < rows.len() {
            let bar = &rows[i];
            let key = (bar.avail_ns.unwrap_or(i64::MAX), bar.seq.unwrap_or(i64::MIN));
            if key > anchor_key && bar.avail_ns.is_some_and(|available| available < cutoff) { break; }
            i += 1;
        }
        let mut horizons = Vec::new();
        for count in BAR_HORIZONS {
            let end = i.saturating_add(count - 1);
            if i >= rows.len() || end >= rows.len() {
                horizons.push(json!({"horizon_bars":count,"complete":false,"censor_reason":"NO_FULLY_POST_ANCHOR_BAR"}));
                continue;
            }
            let slice = &rows[i..=end];
            if !slice.iter().all(|bar| bar.avail_ns.is_some_and(|available| available < cutoff)) {
                horizons.push(json!({"horizon_bars":count,"complete":false,"censor_reason":"UNRESOLVED_AVAILABILITY"}));
                continue;
            }
            let endpoint = (slice[count - 1].close / anchor.mid).ln();
            let mut mfe: f64 = 0.0; let mut mae: f64 = 0.0; let mut rv = 0.0; let mut previous = anchor.mid;
            for bar in slice { mfe = mfe.max(anchor.direction * (bar.high / anchor.mid).ln()); mae = mae.min(anchor.direction * (bar.low / anchor.mid).ln()); rv += (bar.close / previous).ln().powi(2); previous = bar.close; }
            horizons.push(json!({"horizon_bars":count,"bar_close_ts_ns":slice[count-1].close_ns,"bar_open_price":slice[0].open,"bar_close_return_log":endpoint,"direction_adjusted_return":anchor.direction*endpoint,"mfe":mfe.max(0.0),"mae":(-mae).max(0.0),"realized_volatility":rv,"complete":true}));
        }
        scale_out.insert(scale.to_owned(), Value::Array(horizons));
    }
    json!({"anchor_id":anchor.anchor_id,"scales":scale_out})
}

fn write_market_artifact(path: &Path, native_paths: &Path, bridge: &BridgeData, anchors: &[Anchor], cutoff: i64) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(br#"{"artifact_type":"AP-001_WP2_MARKET_PATHS","native_tick_horizons_seconds":[5,15,30,60,120,300],"paths":"#)?;
    let mut native = File::open(native_paths)?;
    std::io::copy(&mut native, &mut writer)?;
    writer.write_all(br#"],"bar_bridge":{"metadata":#)?;
    serde_json::to_writer(&mut writer, &bridge.metadata)?;
    writer.write_all(br#","outcomes":["#)?;
    for (index, anchor) in anchors.iter().enumerate() { if index > 0 { writer.write_all(b",")?; } serde_json::to_writer(&mut writer, &bridge_outcome(anchor, bridge, cutoff))?; }
    writer.write_all(br#"]},"native_clock_separation":"six native tick horizons remain separate from the bounded seven-scale bridge","status":"DISCOVERY_ONLY"}"#)?;
    writer.flush()?;
    let (sha, bytes) = sha256_file(path)?;
    Ok((sha, bytes))
}

fn scan_ticks(parts: &[PathBuf], cutoff: i64, batch_size: usize) -> Result<Capture, Box<dyn std::error::Error>> {
    let mut capture = Capture::default(); let started=Instant::now(); let mut rows=0u64;
    for (part_i,path) in parts.iter().enumerate() {
        eprintln!("[wp2] checkpoint=tick_part_start part={part_i} path={}", path.display());
        let builder=ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?;
        let schema=builder.schema().clone(); let indices=["event_ts_ns","received_ts_ns","source_sequence","bid","ask","payload_drift_burst"].iter().map(|n|schema.index_of(n)).collect::<Result<Vec<_>,_>>()?;
        let mask=ProjectionMask::roots(builder.parquet_schema(),indices); let reader=builder.with_projection(mask).with_batch_size(batch_size).build()?;
        'batches: for batch in reader {
            let batch=batch?; let event=batch.column(0).as_any().downcast_ref::<Int64Array>().ok_or("event")?; let recv=batch.column(1).as_any().downcast_ref::<Int64Array>().ok_or("received")?; let seq=batch.column(2).as_any().downcast_ref::<Int64Array>().ok_or("sequence")?; let bid=batch.column(3).as_any().downcast_ref::<Float64Array>().ok_or("bid")?; let ask=batch.column(4).as_any().downcast_ref::<Float64Array>().ok_or("ask")?; let payload=batch.column(5).as_any().downcast_ref::<LargeStringArray>().ok_or("payload")?;
            for row in 0..batch.num_rows() {
                if recv.is_null(row) || recv.value(row)>=cutoff { break 'batches; }
                rows+=1; if event.is_null(row)||bid.is_null(row)||ask.is_null(row)||payload.is_null(row) { continue; }
                let mut fallback=0; let mut malformed=0; if let ParsedPayload::Snapshot(snap)=ap001_drift_burst::parse_payload(payload.value(row),&mut fallback,&mut malformed) { capture.observe(event.value(row),Some(recv.value(row)),Some(if seq.is_null(row){i64::MIN}else{seq.value(row)}),bid.value(row),ask.value(row),payload.value(row),&snap); }
            }
        }
        eprintln!("[wp2] checkpoint=tick_part_done part={part_i} rows={rows} elapsed_s={:.1}",started.elapsed().as_secs_f64());
    }
    capture.finish(); eprintln!("[wp2] checkpoint=tick_scan_done rows={rows} anchors={} by_state={:?} state_rows={} elapsed_s={:.1}",capture.anchors.len(),capture.anchor_counts(),capture.state_rows,started.elapsed().as_secs_f64()); Ok(capture)
}

fn files_from_lake(lake_root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> { let mut parts=std::fs::read_dir(lake_root.join("tick"))?.filter_map(Result::ok).map(|e|e.path()).filter(|p|p.extension().and_then(|x|x.to_str())==Some("parquet")).collect::<Vec<_>>(); parts.sort(); Ok(parts) }

fn load_contract_value(path: &Path) -> Result<(Value, ap001_drift_burst::Contract), Box<dyn std::error::Error>> {
    let wp1_path = path
        .parent()
        .and_then(Path::parent)
        .ok_or("WP2 contract is not under AP-001/R2_LIFECYCLE_MARKET")?
        .join("R1_CONTRACT/AP-001_WP1_CONTRACT.json");
    Ok((serde_json::from_slice(&std::fs::read(path)?)?, ap001_drift_burst::load_contract(&wp1_path)?))
}

pub fn run(config: Wp2Config) -> Result<i32, Box<dyn std::error::Error>> {
    let started=Instant::now(); let (contract_value, contract)=load_contract_value(&config.contract_path)?; let lake_root=config.lake_root.clone().unwrap_or(contract.lake_root.clone()); let out_dir=config.out_dir.clone().unwrap_or(contract.path.parent().unwrap().to_path_buf()); std::fs::create_dir_all(&out_dir)?;
    eprintln!("[wp2] TASK WP2 lifecycle + native tick market + bounded tick-to-bar bridge"); eprintln!("[wp2] KNOWN_THROUGHPUT=118673 rows/s INPUT_SIZE={} PREDICTED_TICK_SCAN_SECONDS={:.1}",contract.tick_rows_expected,contract.tick_rows_expected as f64/118673.0); eprintln!("[wp2] CHECKPOINTS=preflight,tick parts,bar sidecar,serialization,deterministic replay; DEVIATION_RULE=actual > 3x predicted stops run");
    let parts=files_from_lake(&lake_root)?; let capture=scan_ticks(&parts,contract.cutoff_ns,config.batch_size)?; let native_path=out_dir.join("AP-001_WP2_MARKET_PATHS.native.tmp"); let mut market=run_market_scan(&parts,contract.cutoff_ns,&capture.anchors,&capture.pseudo_rows,config.batch_size,&native_path)?; eprintln!("[wp2] checkpoint=market_scan_done paths={} baseline_paths={}",market.native_path_count,market.baseline_path_count);
    let anchors=anchors_json(&capture.anchors); let atlas=lifecycle_value(&capture); let bridge=load_bar_bridge(&config.bar_view_root,&config.sidecar_path,contract.cutoff_ns)?; let splits=splits_value(&capture.anchors); let reference=support_reference(&mut market.baseline_values);
    let mut artifact_hashes=Map::new(); for (name,value) in [("AP-001_WP2_ANCHORS.json",anchors),("AP-001_WP2_LIFECYCLE_ATLAS.json",atlas),("AP-001_WP2_INTERNAL_SPLITS.json",splits),("AP-001_WP2_SUPPORT_REFERENCE.json",reference)] { let (sha,bytes)=write_json(&out_dir.join(name),&value)?; artifact_hashes.insert(name.to_owned(),json!({"sha256":sha,"bytes":bytes,"content_hash":canonical_hash(&value)})); }
    let (market_sha,market_bytes)=write_market_artifact(&out_dir.join("AP-001_WP2_MARKET_PATHS.json"),&market.native_paths,&bridge,&capture.anchors,contract.cutoff_ns)?; std::fs::remove_file(&market.native_paths)?; artifact_hashes.insert("AP-001_WP2_MARKET_PATHS.json".to_owned(),json!({"sha256":market_sha,"bytes":market_bytes}));
    let report=format!("# AP-001 WP2 E1 report\n\n- Status: **DISCOVERY_ONLY**.\n- Development cutoff: `{}` (`{}`).\n- Anchors: {} rows; lifecycle episodes: {}; native path rows: {}; baseline path rows: {}.\n- Native tick horizons: 5, 15, 30, 60, 120, 300 seconds. The seven-scale bar bridge is reported separately and does not replace native outcomes.\n- Missingness is retained per horizon. No confirmation rows, promotion, confluence claim, or activation decision is produced.\n\nThis report is frozen before WP3/WP4 interpretation.\n",contract.cutoff_ns,contract.cutoff_utc,capture.anchors.len(),capture.episodes.len(),market.native_path_count,market.baseline_path_count); std::fs::write(out_dir.join("AP-001_WP2_E1_REPORT.md"),report.as_bytes())?;
    let mut anchors_by_state=Map::new(); for state in ["WEAK","ONLINE","STRONG","DECAY_RISK","DYING"] { anchors_by_state.insert(state.to_owned(),json!(capture.anchors.iter().filter(|a|a.state==state).count())); }
    let (contract_sha,contract_bytes)=sha256_file(&config.contract_path)?; let (wp1_preflight_sha,wp1_preflight_bytes)=sha256_file(&contract.path.parent().unwrap().parent().unwrap().join("R1_CONTRACT/AP-001_WP1_PREFLIGHT.json"))?; let sidecar_manifest=sha256_file(&config.sidecar_manifest_path)?; let source_manifest=sha256_file(&config.bar_view_root.join("view_manifest.json"))?; let preflight_core=json!({"artifact_type":"AP-001_WP2_PREFLIGHT","contract":{"path":config.contract_path,"sha256":contract_sha,"bytes":contract_bytes,"contract_id":contract_value.get("contract_id").cloned().unwrap_or(Value::Null)},"wp1_preflight":{"sha256":wp1_preflight_sha,"bytes":wp1_preflight_bytes},"input":{"lake_root":lake_root,"tick_parts":parts.iter().map(|p|p.display().to_string()).collect::<Vec<_>>(),"tick_rows_expected":contract.tick_rows_expected,"development_rows_scanned":capture.state_rows,"cutoff_ns":contract.cutoff_ns},"availability_evidence":{"sidecar_path":config.sidecar_path,"sidecar_manifest":config.sidecar_manifest_path,"sidecar_manifest_sha256":sidecar_manifest.0,"sidecar_manifest_bytes":sidecar_manifest.1,"source_view_manifest":config.bar_view_root.join("view_manifest.json"),"source_view_manifest_sha256":source_manifest.0,"source_view_manifest_bytes":source_manifest.1},"counts":{"anchors":capture.anchors.len(),"anchors_by_state":anchors_by_state,"episodes":capture.episodes.len(),"state_rows":capture.state_rows,"left_truncated":capture.left_truncated,"right_censored":capture.right_censored},"artifact_hashes":artifact_hashes,"determinism":{"rerun_requested":!config.single_run,"canonical_output_hashes_match":"not_run","replay_mode":if config.single_run{"single_run"}else{"executor_first_pass"}},"runtime":{"wall_seconds":started.elapsed().as_secs_f64(),"peak_rss":"not measured by WP2 executor"},"status":"DISCOVERY_ONLY"}); let (core_sha,_)=write_json(&out_dir.join("AP-001_WP2_PREFLIGHT.json"),&preflight_core)?; eprintln!("[wp2] checkpoint=serialization_done preflight_sha={core_sha} wall_s={:.1}",started.elapsed().as_secs_f64()); Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_replacement_keeps_only_latest_value() {
        let mut slot=None; let dbs=json!({"operational":{"direction":"UP"}});
        update_candidate(&mut slot,Some(1),10,Some(10),Some(1),&dbs,1.0,100.0,100.0); update_candidate(&mut slot,Some(2),20,Some(20),Some(2),&dbs,1.0,101.0,101.0);
        assert_eq!(slot.unwrap().ts_ms,2);
    }
    #[test]
    fn pending_window_excludes_anchor_key_and_completes_at_horizon() {
        let a=Anchor{anchor_id:"a".into(),episode_id:1,state:"STRONG",event_ts_ns:0,received_ts_ns:Some(0),source_sequence:Some(1),bid:100.0,ask:100.0,mid:100.0,direction:1.0,left_truncated:false,record:Value::Null}; let mut p=Pending::new(a); p.update(&Tick{avail_ns:0,seq:1,mid:100.0}); assert!(p.horizons[0].last_mid.is_none()); p.update(&Tick{avail_ns:5_000_000_000,seq:2,mid:101.0}); assert!(p.horizons[0].complete); assert!(p.horizons[0].last_mid.is_some());
    }
    #[test]
    fn direction_parser_handles_down() { assert_eq!(direction_from(&json!({"operational":{"direction":"DOWN"}})),-1.0); }
    #[test]
    fn left_truncated_marks_only_first_state_episode() {
        let snapshot = |state, eid| Snapshot { state, eid_pos: eid, weak_ms: None, online_ms: None, emissions: Vec::new() };
        let mut capture = Capture::default();
        capture.observe(1, Some(1), Some(1), 100.0, 100.0, "", &snapshot(DState::Burst, Some(1)));
        capture.observe(2, Some(2), Some(2), 100.0, 100.0, "", &snapshot(DState::Idle, None));
        capture.observe(3, Some(3), Some(3), 100.0, 100.0, "", &snapshot(DState::Burst, Some(2)));
        assert_eq!(capture.left_truncated, 1);
    }
    #[test]
    fn serialized_path_keeps_sort_key_summary_and_json_bytes() {
        let a=Anchor{anchor_id:"a".into(),episode_id:1,state:"STRONG",event_ts_ns:0,received_ts_ns:Some(0),source_sequence:Some(1),bid:100.0,ask:100.0,mid:100.0,direction:1.0,left_truncated:false,record:Value::Null};
        let mut p=Pending::new(a);
        p.update(&Tick{avail_ns:0,seq:1,mid:100.0});
        p.update(&Tick{avail_ns:5_000_000_000,seq:2,mid:101.0});
        let path=serialize_path(&p,&p.horizons[0],HORIZONS[0]).expect("path serialization");
        let value: Value=serde_json::from_slice(&path.bytes).expect("serialized path JSON");
        assert_eq!(value.get("anchor_id").and_then(Value::as_str),Some("a"));
        assert_eq!(value.get("horizon_seconds").and_then(Value::as_i64),Some(5));
    }
    #[test]
    fn path_sink_writes_one_json_array_without_retaining_records() {
        let path=std::env::temp_dir().join(format!("ap001_wp2_path_sink_{}.json",std::process::id()));
        let mut sink=PathSink::create(&path).expect("create path sink");
        sink.push(br#"{"anchor_id":"a"}"#).expect("write path");
        sink.finish().expect("finish path sink");
        assert_eq!(std::fs::read_to_string(&path).expect("read path"),r#"[{"anchor_id":"a"}]"#);
        std::fs::remove_file(path).expect("remove path sink");
    }
}
