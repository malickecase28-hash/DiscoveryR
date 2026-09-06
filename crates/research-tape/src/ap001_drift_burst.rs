//! AP-001 work package WP1: contract-and-benchmark scanner for the drift-burst
//! detector tape (frozen contract `AP-001-WP1-CONTRACT`).
//!
//! Gates:
//! - G1 input identity: SHA256 + byte size of `manifest.json` and
//!   `payload_manifest.json` must match the frozen contract.
//! - G2 causal ordering: zero `(received_ts_ns, source_sequence)` regressions
//!   and zero `received_ts_ns < event_ts_ns` rows over the development leaf.
//! - G3 anchor sanity: live-snapshot first-entry times for WEAK / ONLINE /
//!   STRONG / DECAY_RISK / DYING must equal the emitted event ground truth for
//!   every episode completed inside the development leaf.
//! - G4 benchmark: streaming rows/s with the WP2 projection and peak RSS.
//!
//! Partition policy: rows with `received_ts_ns >= cutoff_ns` are the
//! confirmation leaf and are never evaluated. The scan stops at the first row
//! at/after the cutoff; only footer metadata is read for later parts.

use arrow_array::{Array, Float64Array, Int64Array, LargeStringArray, RecordBatch, StringArray};
use parquet::arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ProjectionMask};
use parquet::file::statistics::Statistics as ParquetStatistics;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub const CONTRACT_DEFAULT: &str =
    r"F:\TrinityR-research\Research Program\AP-001\R1_CONTRACT\AP-001_WP1_CONTRACT.json";
pub const PREFLIGHT_NAME: &str = "AP-001_WP1_PREFLIGHT.json";
pub const REPORT_NAME: &str = "AP-001_WP1_REPORT.md";
/// WP2 projection per contract gate G4: drift_burst payload + timestamps +
/// sequence + bid/ask.
pub const PROJECTION: [&str; 6] = [
    "event_ts_ns",
    "received_ts_ns",
    "source_sequence",
    "bid",
    "ask",
    "payload_drift_burst",
];
/// Payload columns outside the WP2 projection. Footer statistics are absent
/// for them (string columns), so their null counts are not measured by WP1.
pub const NON_PROJECTED_PAYLOADS: [&str; 6] = [
    "payload_spread_state",
    "payload_quote_dynamics",
    "payload_quote_arrival",
    "payload_micro_volatility",
    "payload_feed_health",
    "payload_quote_pressure",
];

// ---------------------------------------------------------------------------
// Detector snapshot model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DState {
    Idle,
    Developing,
    Burst,
    DecayRisk,
    Dying,
    Other,
}

impl DState {
    fn parse(raw: &str) -> Self {
        match raw {
            "IDLE" => Self::Idle,
            "DEVELOPING" => Self::Developing,
            "BURST" => Self::Burst,
            "DECAY_RISK" => Self::DecayRisk,
            "DYING" => Self::Dying,
            _ => Self::Other,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Developing => "DEVELOPING",
            Self::Burst => "BURST",
            Self::DecayRisk => "DECAY_RISK",
            Self::Dying => "DYING",
            Self::Other => "OTHER",
        }
    }
}

pub const ANCHOR_LABELS: [&str; 5] = ["WEAK", "ONLINE", "STRONG", "DECAY_RISK", "DYING"];
const A_WEAK: usize = 0;
const A_ONLINE: usize = 1;
const A_STRONG: usize = 2;
const A_DECAY: usize = 3;
const A_DYING: usize = 4;

/// Ground truth extracted from one `emitted_events` entry (epoch milliseconds).
#[derive(Debug, Clone, Default)]
pub struct EmittedTruth {
    pub event_id: i64,
    pub origin_ts: Option<i64>,
    pub weak_ts: Option<i64>,
    pub online_ts: Option<i64>,
    pub strong_ts: Option<i64>,
    pub decay_risk_ts: Option<i64>,
    pub first_dying_ts: Option<i64>,
    pub end_ts: Option<i64>,
}

/// One captured emitted entry: extracted truth plus a canonical hash of the
/// full raw entry used for mutation detection.
#[derive(Debug, Clone)]
pub struct EmittedEntry {
    pub truth: EmittedTruth,
    pub hash: [u8; 32],
}

/// Live detector snapshot for one tick.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub state: DState,
    /// `Some(id)` when `event_id >= 0` (episode armed at BURST); `None` while
    /// forming (`-1`, missing or null).
    pub eid_pos: Option<i64>,
    pub weak_ms: Option<i64>,
    pub online_ms: Option<i64>,
    pub emissions: Vec<EmittedEntry>,
}

#[derive(Deserialize)]
struct PayloadDoc<'a> {
    #[serde(rename = "drift_burst_state", borrow)]
    dbs: DriftStateIn<'a>,
}

#[derive(Deserialize)]
struct DriftStateIn<'a> {
    #[serde(borrow, default)]
    state: Option<&'a str>,
    #[serde(default)]
    event_id: Option<i64>,
    #[serde(default)]
    milestones: Option<MilestonesIn>,
    #[serde(default)]
    emitted_events: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct MilestonesIn {
    #[serde(default)]
    weak: Option<MilestoneIn>,
    #[serde(default)]
    online: Option<MilestoneIn>,
}

#[derive(Deserialize)]
struct MilestoneIn {
    #[serde(default)]
    ts_ms: Option<i64>,
}

/// Outcome of parsing one payload string.
pub enum ParsedPayload {
    Snapshot(Snapshot),
    /// `payload_drift_burst` present but no `drift_burst_state` object.
    NoStateObject,
}

fn truth_from_value(v: &Value, malformed: &mut u64) -> Option<EmittedTruth> {
    let event_id = v.get("event_id").and_then(Value::as_i64);
    let event_id = match event_id {
        Some(id) => id,
        None => {
            *malformed += 1;
            return None;
        }
    };
    let ts = |key: &str| v.get(key).and_then(Value::as_i64);
    Some(EmittedTruth {
        event_id,
        origin_ts: ts("origin_ts"),
        weak_ts: ts("weak_ts"),
        online_ts: ts("online_ts"),
        strong_ts: ts("strong_ts"),
        decay_risk_ts: ts("decay_risk_ts"),
        first_dying_ts: ts("first_dying_ts"),
        end_ts: ts("end_ts"),
    })
}

fn snapshot_from_value(v: &Value, malformed: &mut u64) -> Option<ParsedPayload> {
    let dbs = v.get("drift_burst_state")?;
    let state = dbs.get("state").and_then(Value::as_str);
    let eid = dbs.get("event_id").and_then(Value::as_i64);
    let milestones = dbs.get("milestones");
    let ms = |key: &str| {
        milestones
            .and_then(|m| m.get(key))
            .filter(|m| !m.is_null())
            .and_then(|m| m.get("ts_ms"))
            .and_then(Value::as_i64)
    };
    let emissions = dbs
        .get("emitted_events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut snapshot = Snapshot {
        state: state.map(DState::parse).unwrap_or(DState::Other),
        eid_pos: eid.filter(|id| *id >= 0),
        weak_ms: ms("weak"),
        online_ms: ms("online"),
        emissions: Vec::new(),
    };
    for entry in &emissions {
        if let Some(truth) = truth_from_value(entry, malformed) {
            snapshot.emissions.push(EmittedEntry {
                truth,
                hash: canonical_hash(entry),
            });
        }
    }
    Some(ParsedPayload::Snapshot(snapshot))
}

/// Parse a `payload_drift_burst` string with a fast borrowed-struct pass and a
/// `serde_json::Value` fallback for unexpected shapes.
pub fn parse_payload(payload: &str, fallbacks: &mut u64, malformed: &mut u64) -> ParsedPayload {
    match serde_json::from_str::<PayloadDoc>(payload) {
        Ok(doc) => {
            let emissions = doc.dbs.emitted_events.unwrap_or_default();
            let mut emissions_kept = Vec::with_capacity(emissions.len());
            for entry in &emissions {
                if let Some(truth) = truth_from_value(entry, malformed) {
                    emissions_kept.push(EmittedEntry {
                        truth,
                        hash: canonical_hash(entry),
                    });
                }
            }
            ParsedPayload::Snapshot(Snapshot {
                state: doc.dbs.state.map(DState::parse).unwrap_or(DState::Other),
                eid_pos: doc.dbs.event_id.filter(|id| *id >= 0),
                weak_ms: doc.dbs.milestones.as_ref().and_then(|m| m.weak.as_ref()).and_then(|w| w.ts_ms),
                online_ms: doc
                    .dbs
                    .milestones
                    .as_ref()
                    .and_then(|m| m.online.as_ref())
                    .and_then(|o| o.ts_ms),
                emissions: emissions_kept,
            })
        }
        Err(_) => {
            *fallbacks += 1;
            match serde_json::from_str::<Value>(payload) {
                Ok(value) => snapshot_from_value(&value, malformed).unwrap_or(ParsedPayload::NoStateObject),
                Err(_) => ParsedPayload::NoStateObject,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// G3 episode engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Anchor {
    /// First lawful entry observed live in-stream.
    Recorded { tick_ms: i64, snapshot_ms: Option<i64> },
    /// Already present on the first observed tick of a left-truncated episode;
    /// not a countable first entry.
    Absorbed,
}

/// D-006 milestone candidate: created when a milestone transitions null ->
/// value, replaced when the producer replaces the value. `record` is the
/// at-crossing snapshot (bounded, canonical).
#[derive(Clone)]
struct Candidate {
    ts_ms: i64,
    record: Value,
}

/// D-006 anchor value frozen at the episode's arm.
#[derive(Clone)]
struct AnchorValue {
    ts_ms: i64,
    record: Value,
}

#[derive(Clone)]
struct Track {
    id: Option<i64>,
    left_truncated: bool,
    closed: bool,
    last_tick_ms: i64,
    rows: u64,
    prev_state: Option<DState>,
    /// State anchors (STRONG / DECAY_RISK / DYING). WEAK/ONLINE live in
    /// `weak_anchor` / `online_anchor` under D-006.
    anchors: [Option<Anchor>; 5],
    weak_candidate: Option<Candidate>,
    online_candidate: Option<Candidate>,
    /// Frozen at the arm (first BURST tick of the episode).
    weak_anchor: Option<AnchorValue>,
    online_anchor: Option<AnchorValue>,
    /// Pre-amendment (run 1) rule diagnostic: first milestone sighting in any
    /// phase, forming or armed.
    first_sight_weak: Option<i64>,
    first_sight_online: Option<i64>,
}

impl Track {
    fn start(snap: &Snapshot, tick_ms: i64, left_truncated: bool, eid_pos: Option<i64>) -> Self {
        let mut track = Self {
            id: eid_pos,
            left_truncated,
            closed: false,
            last_tick_ms: tick_ms,
            rows: 0,
            prev_state: None,
            anchors: [None; 5],
            weak_candidate: None,
            online_candidate: None,
            weak_anchor: None,
            online_anchor: None,
            first_sight_weak: None,
            first_sight_online: None,
        };
        if left_truncated {
            // Baseline absorbs whatever is already present: none of it is a
            // countable first entry. Milestone carryover is governed by the
            // ground-truth window rule at cross-check time.
            if matches!(snap.state, DState::Burst | DState::DecayRisk | DState::Dying) {
                track.anchors[A_STRONG] = Some(Anchor::Absorbed);
            }
            if snap.state == DState::DecayRisk {
                track.anchors[A_DECAY] = Some(Anchor::Absorbed);
            }
            if snap.state == DState::Dying {
                track.anchors[A_DYING] = Some(Anchor::Absorbed);
            }
        }
        track
    }
}

struct EmittedRec {
    truth: EmittedTruth,
    hash: [u8; 32],
    appearance_tick_ms: i64,
}

#[derive(Default)]
pub struct EngineCounts {
    pub state_rows: u64,
    pub left_truncated_tracks: u64,
    pub tracks_started: u64,
    pub tracks_completed: u64,
    pub forming_fizzled: u64,
    pub right_censored_forming: u64,
    pub right_censored_armed: u64,
    pub duplicate_track_ids: u64,
    pub anchors_detected: [u64; 5],
    pub gt_present: [u64; 5],
    pub matched: [u64; 5],
    pub value_mismatch: [u64; 5],
    pub missing_live: [u64; 5],
    pub spurious_live: [u64; 5],
    pub tick_lag: [u64; 5],
    pub excluded_left_truncated: [u64; 5],
    pub boundary_ambiguous: [u64; 5],
    pub emissions_seen: u64,
    pub emitted_entries: u64,
    pub emitted_mutations: u64,
    pub duplicate_sightings: u64,
    pub malformed_entries: u64,
    pub emitted_at_leaf_start: u64,
    pub pre_leaf_history: u64,
    pub episodes_completed_in_leaf: u64,
    pub entries_without_track: u64,
    pub closed_tracks_without_emission: u64,
    pub end_ts_vs_appearance_mismatch: u64,
    pub end_ts_vs_appearance_checked: u64,
    pub unknown_states: u64,
    /// D-006: episodes whose WEAK/ONLINE anchors were frozen at arm.
    pub armed_detections: [u64; 2],
    /// D-006 null agreement (cross-checked completed episodes): milestones
    /// null at arm must match emitted nulls in both directions.
    pub anchor_null: [u64; 2],
    pub emitted_null: [u64; 2],
    /// At-crossing snapshot records that could not be extracted.
    pub anchor_record_extraction_failures: u64,
    /// Pre-amendment (run 1) first-sighting rule reproduced as a diagnostic.
    pub diag_first_sight_value_mismatch: [u64; 2],
    pub diag_first_sight_missing: [u64; 2],
    pub diag_first_sight_spurious: [u64; 2],
}

pub struct Engine {
    counts: EngineCounts,
    seen_state_row: bool,
    leaf_first_event_ns: Option<i64>,
    baseline_emitted: BTreeSet<i64>,
    active: Option<Track>,
    tracks: BTreeMap<i64, Track>,
    emitted: BTreeMap<i64, EmittedRec>,
    /// D-006 at-crossing anchor records per completed/armed episode:
    /// event_id -> (weak record, online record).
    anchor_records: BTreeMap<i64, (Option<Value>, Option<Value>)>,
    /// Consecutive state-bearing row transitions (episode boundaries included)
    /// for the anomaly report; sorted deterministically by (from, to).
    pub transitions: BTreeMap<(u8, u8), u64>,
    last_observed_state: Option<DState>,
}

pub fn canonical_hash(value: &Value) -> [u8; 32] {
    let canonical = serde_json::to_string(value).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// D-006 at-crossing snapshot record for a milestone candidate: the crossing
/// tick's availability plus the snapshot fields needed for the anchor record.
/// Extracted only on candidate creation/replacement (rare).
#[allow(clippy::too_many_arguments)]
fn crossing_record(
    payload: Option<&str>,
    ts_ms: i64,
    tick_ns: i64,
    received_ts_ns: Option<i64>,
    source_sequence: Option<i64>,
    state: DState,
    weak_present: bool,
    online_present: bool,
    extraction_failures: &mut u64,
) -> Value {
    let dbs = payload.and_then(|p| match serde_json::from_str::<Value>(p) {
        Ok(value) => value.get("drift_burst_state").cloned(),
        Err(_) => {
            *extraction_failures += 1;
            None
        }
    });
    let empty = Value::Null.clone();
    let dbs = dbs.as_ref().unwrap_or(&empty);
    json!({
        "milestone_ts_ms": ts_ms,
        "crossing": {
            "event_ts_ns": tick_ns,
            "received_ts_ns": received_ts_ns,
            "source_sequence": source_sequence,
        },
        "state": state.label(),
        "milestones_crossed": {"weak": weak_present, "online": online_present},
        "operational": dbs.get("operational").cloned().unwrap_or(Value::Null),
        "multiscale": dbs.get("multiscale").cloned().unwrap_or(Value::Null),
        "authority": dbs.get("authority").cloned().unwrap_or(Value::Null),
    })
}

impl Engine {
    pub fn new() -> Self {
        Self {
            counts: EngineCounts::default(),
            seen_state_row: false,
            leaf_first_event_ns: None,
            baseline_emitted: BTreeSet::new(),
            active: None,
            tracks: BTreeMap::new(),
            emitted: BTreeMap::new(),
            anchor_records: BTreeMap::new(),
            transitions: BTreeMap::new(),
            last_observed_state: None,
        }
    }

    pub fn counts(&self) -> &EngineCounts {
        &self.counts
    }

    /// Feed one development-leaf tick. `snap: None` means the payload was
    /// null or unusable (transparent row: no boundary, no anchor updates).
    /// `payload` is the raw `payload_drift_burst` string, re-parsed only on
    /// rare candidate events to extract the at-crossing anchor record.
    pub fn observe(
        &mut self,
        tick_ns: i64,
        received_ts_ns: Option<i64>,
        source_sequence: Option<i64>,
        payload: Option<&str>,
        snap: Option<&Snapshot>,
    ) {
        let tick_ms = tick_ns.div_euclid(1_000_000);
        if self.leaf_first_event_ns.is_none() {
            self.leaf_first_event_ns = Some(tick_ns);
        }
        let snap = match snap {
            Some(snap) => snap,
            None => return,
        };
        let first_state_row = !self.seen_state_row;
        self.seen_state_row = true;
        self.counts.state_rows += 1;
        if snap.state == DState::Other {
            self.counts.unknown_states += 1;
        }
        if let Some(previous) = self.last_observed_state {
            *self
                .transitions
                .entry((previous as u8, snap.state as u8))
                .or_insert(0) += 1;
        }
        self.last_observed_state = Some(snap.state);

        // Emitted ground truth: entries are captured wherever they appear
        // (each completed episode appends its entry on its end tick). Entries
        // already present on the first state-bearing row predate the
        // observable leaf window or end exactly at it; both are excluded from
        // the cross-check and counted separately.
        if first_state_row {
            for entry in &snap.emissions {
                if self.baseline_emitted.insert(entry.truth.event_id) {
                    let leaf_first = self.leaf_first_event_ns.unwrap_or(i64::MIN);
                    match entry.truth.end_ts {
                        Some(end_ms) if end_ms.saturating_mul(1_000_000) >= leaf_first => {
                            self.counts.episodes_completed_in_leaf += 1;
                            self.counts.emitted_at_leaf_start += 1;
                        }
                        _ => self.counts.pre_leaf_history += 1,
                    }
                }
            }
        }
        for entry in &snap.emissions {
            self.counts.emissions_seen += 1;
            let truth = &entry.truth;
            let hash = entry.hash;
            let id = truth.event_id;
            match self.emitted.get(&id) {
                Some(existing) => {
                    if existing.hash != hash {
                        self.counts.emitted_mutations += 1;
                        self.emitted.insert(
                            id,
                            EmittedRec {
                                truth: truth.clone(),
                                hash,
                                appearance_tick_ms: tick_ms,
                            },
                        );
                    } else {
                        self.counts.duplicate_sightings += 1;
                    }
                }
                None => {
                    self.counts.emitted_entries += 1;
                    self.emitted.insert(
                        id,
                        EmittedRec {
                            truth: truth.clone(),
                            hash,
                            appearance_tick_ms: tick_ms,
                        },
                    );
                }
            }
        }

        // Episode boundary detection.
        let eid_pos = snap.eid_pos;
        let state = snap.state;
        match self.active.take() {
            None => {
                if state != DState::Idle {
                    let left_truncated = first_state_row;
                    if left_truncated {
                        self.counts.left_truncated_tracks += 1;
                    }
                    let track = Track::start(snap, tick_ms, left_truncated, eid_pos);
                    self.counts.tracks_started += 1;
                    self.active = Some(track);
                    self.step(snap, tick_ns, received_ts_ns, source_sequence, payload);
                }
            }
            Some(mut track) => {
                let mut close = false;
                let mut superseded = false;
                if state == DState::Idle {
                    close = true;
                } else {
                    match (track.id, eid_pos) {
                        (Some(old), Some(new)) if old != new => {
                            close = true;
                            superseded = true;
                        }
                        (Some(_), Some(_)) => {}
                        (Some(_), None) => {
                            close = true;
                            superseded = state != DState::Idle;
                        }
                        (None, Some(new)) => track.id = Some(new),
                        (None, None) => {}
                    }
                }
                if close {
                    self.close_track(track, tick_ms);
                    if superseded {
                        let left_truncated = false;
                        let track = Track::start(snap, tick_ms, left_truncated, eid_pos);
                        self.counts.tracks_started += 1;
                        self.active = Some(track);
                        self.step(snap, tick_ns, received_ts_ns, source_sequence, payload);
                    }
                } else {
                    self.active = Some(track);
                    self.step(snap, tick_ns, received_ts_ns, source_sequence, payload);
                }
            }
        }
    }

    fn close_track(&mut self, mut track: Track, _tick_ms: i64) {
        track.closed = true;
        match track.id {
            Some(id) => {
                self.counts.tracks_completed += 1;
                self.anchor_records.entry(id).or_insert((
                    track.weak_anchor.as_ref().map(|a| a.record.clone()),
                    track.online_anchor.as_ref().map(|a| a.record.clone()),
                ));
                if self.tracks.insert(id, track).is_some() {
                    self.counts.duplicate_track_ids += 1;
                }
            }
            None => self.counts.forming_fizzled += 1,
        }
    }

    /// Anchor step for the active track on a state-bearing tick.
    fn step(
        &mut self,
        snap: &Snapshot,
        tick_ns: i64,
        received_ts_ns: Option<i64>,
        source_sequence: Option<i64>,
        payload: Option<&str>,
    ) {
        let tick_ms = tick_ns.div_euclid(1_000_000);
        let Some(track) = self.active.as_mut() else {
            return;
        };
        track.rows += 1;
        track.last_tick_ms = tick_ms;
        // D-006 candidate capture: a milestone transitioning null -> value
        // creates a candidate; the producer replacing the value (forming-window
        // restart) replaces it; the milestone vanishing drops it. The
        // at-crossing snapshot record is extracted only on these rare events.
        Self::update_candidate(
            &mut track.weak_candidate,
            snap.weak_ms,
            tick_ns,
            received_ts_ns,
            source_sequence,
            payload,
            snap,
            &mut self.counts.anchor_record_extraction_failures,
        );
        Self::update_candidate(
            &mut track.online_candidate,
            snap.online_ms,
            tick_ns,
            received_ts_ns,
            source_sequence,
            payload,
            snap,
            &mut self.counts.anchor_record_extraction_failures,
        );
        // Pre-amendment (run 1) rule diagnostic: first sighting in any phase.
        if track.first_sight_weak.is_none() {
            track.first_sight_weak = snap.weak_ms;
        }
        if track.first_sight_online.is_none() {
            track.first_sight_online = snap.online_ms;
        }
        // State anchors: first lawful entry only (re-entries are occupancy
        // material for WP2, not new anchors). The first BURST entry is the
        // episode's arm: the surviving candidates freeze into the WEAK/ONLINE
        // anchors, anchored at their crossing-tick availability.
        if snap.state == DState::Burst
            && track.prev_state != Some(DState::Burst)
            && track.anchors[A_STRONG].is_none()
        {
            track.anchors[A_STRONG] = Some(Anchor::Recorded {
                tick_ms,
                snapshot_ms: None,
            });
            self.counts.anchors_detected[A_STRONG] += 1;
            if track.weak_anchor.is_none() {
                track.weak_anchor = track.weak_candidate.as_ref().map(|c| AnchorValue {
                    ts_ms: c.ts_ms,
                    record: c.record.clone(),
                });
            }
            if track.online_anchor.is_none() {
                track.online_anchor = track.online_candidate.as_ref().map(|c| AnchorValue {
                    ts_ms: c.ts_ms,
                    record: c.record.clone(),
                });
            }
        }
        if snap.state == DState::DecayRisk && track.anchors[A_DECAY].is_none() {
            track.anchors[A_DECAY] = Some(Anchor::Recorded {
                tick_ms,
                snapshot_ms: None,
            });
            self.counts.anchors_detected[A_DECAY] += 1;
        }
        if snap.state == DState::Dying && track.anchors[A_DYING].is_none() {
            track.anchors[A_DYING] = Some(Anchor::Recorded {
                tick_ms,
                snapshot_ms: None,
            });
            self.counts.anchors_detected[A_DYING] += 1;
        }
        track.prev_state = Some(snap.state);
    }

    #[allow(clippy::too_many_arguments)]
    fn update_candidate(
        slot: &mut Option<Candidate>,
        value: Option<i64>,
        tick_ns: i64,
        received_ts_ns: Option<i64>,
        source_sequence: Option<i64>,
        payload: Option<&str>,
        snap: &Snapshot,
        extraction_failures: &mut u64,
    ) {
        let changed = match (slot.as_ref(), value) {
            (None, Some(_)) => true,
            (Some(current), Some(v)) => v != current.ts_ms,
            (Some(_), None) => {
                // The producer dropped the milestone entirely (never observed
                // mid-episode in this tape); mirror it.
                *slot = None;
                return;
            }
            (None, None) => return,
        };
        if !changed {
            return;
        }
        let ts_ms = value.unwrap_or_default();
        let record = crossing_record(
            payload,
            ts_ms,
            tick_ns,
            received_ts_ns,
            source_sequence,
            snap.state,
            snap.weak_ms.is_some(),
            snap.online_ms.is_some(),
            extraction_failures,
        );
        *slot = Some(Candidate { ts_ms, record });
    }

    /// End-of-leaf: settle the active episode and run the ground-truth
    /// cross-check. Idempotent per engine instance.
    pub fn finish(&mut self) {
        if let Some(mut track) = self.active.take() {
            match track.id {
                Some(id) => {
                    if self.emitted.contains_key(&id) {
                        self.counts.tracks_completed += 1;
                    } else {
                        self.counts.right_censored_armed += 1;
                    }
                    self.anchor_records.entry(id).or_insert((
                        track.weak_anchor.as_ref().map(|a| a.record.clone()),
                        track.online_anchor.as_ref().map(|a| a.record.clone()),
                    ));
                    if self.tracks.insert(id, track).is_some() {
                        self.counts.duplicate_track_ids += 1;
                    }
                }
                None => self.counts.right_censored_forming += 1,
            }
        }
        let leaf_first_ns = self.leaf_first_event_ns.unwrap_or(i64::MIN);
        // D-006 detections: episodes whose anchors were frozen at arm.
        for track in self.tracks.values() {
            if track.weak_anchor.is_some() {
                self.counts.armed_detections[0] += 1;
            }
            if track.online_anchor.is_some() {
                self.counts.armed_detections[1] += 1;
            }
        }
        self.counts.anchors_detected[A_WEAK] = self.counts.armed_detections[0];
        self.counts.anchors_detected[A_ONLINE] = self.counts.armed_detections[1];
        let truths: Vec<(i64, EmittedTruth, i64)> = self
            .emitted
            .iter()
            .map(|(id, rec)| (*id, rec.truth.clone(), rec.appearance_tick_ms))
            .collect();
        for (id, truth, appearance_tick_ms) in truths {
            if self.baseline_emitted.contains(&id) {
                // Classified at leaf start (pre-leaf history or a completion
                // landing exactly on the first observed tick).
                continue;
            }
            self.counts.episodes_completed_in_leaf += 1;
            if truth.end_ts.is_some() {
                self.counts.end_ts_vs_appearance_checked += 1;
                if truth.end_ts != Some(appearance_tick_ms) {
                    self.counts.end_ts_vs_appearance_mismatch += 1;
                }
            }
            let Some(track) = self.tracks.get(&id).cloned() else {
                self.counts.entries_without_track += 1;
                continue;
            };
            self.cross_check(&track, &truth, leaf_first_ns);
        }
        for (id, track) in &self.tracks {
            if track.closed && !self.emitted.contains_key(id) {
                self.counts.closed_tracks_without_emission += 1;
            }
        }
    }

    /// D-006: deterministic SHA256 over the at-crossing anchor records
    /// (canonical serde_json, ordered by event_id) per milestone, with counts.
    pub fn anchor_record_hashes(&self) -> ([String; 2], [u64; 2]) {
        let mut weak_records = Vec::with_capacity(self.anchor_records.len());
        let mut online_records = Vec::with_capacity(self.anchor_records.len());
        for (weak, online) in self.anchor_records.values() {
            weak_records.push(weak.clone().unwrap_or(Value::Null));
            online_records.push(online.clone().unwrap_or(Value::Null));
        }
        let hash = |records: &[Value]| {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(records).unwrap_or_default().as_bytes());
            format!("{:x}", hasher.finalize())
        };
        (
            [hash(&weak_records), hash(&online_records)],
            [weak_records.len() as u64, online_records.len() as u64],
        )
    }

    fn cross_check(&mut self, track: &Track, truth: &EmittedTruth, leaf_first_ns: i64) {
        // D-006 WEAK/ONLINE rule: the anchor is the candidate surviving at the
        // episode's arm, anchored at its crossing-tick availability. A null
        // milestone at arm means no anchor; it must agree with the emitted
        // null in both directions.
        for slot in [0usize, 1usize] {
            let gt_ms = match slot {
                0 => truth.weak_ts,
                _ => truth.online_ts,
            };
            let anchor_ts = match slot {
                0 => track.weak_anchor.as_ref().map(|a| a.ts_ms),
                _ => track.online_anchor.as_ref().map(|a| a.ts_ms),
            };
            let first_sight = match slot {
                0 => track.first_sight_weak,
                _ => track.first_sight_online,
            };
            match gt_ms {
                Some(g) => {
                    self.counts.gt_present[slot] += 1;
                    let g_ns = g.saturating_mul(1_000_000);
                    if track.left_truncated && g_ns < leaf_first_ns {
                        self.counts.excluded_left_truncated[slot] += 1;
                    } else {
                        match anchor_ts {
                            Some(value) => {
                                if value == g {
                                    self.counts.matched[slot] += 1;
                                } else {
                                    self.counts.value_mismatch[slot] += 1;
                                }
                            }
                            None => self.counts.missing_live[slot] += 1,
                        }
                        if let Some(first) = first_sight {
                            if first != g {
                                self.counts.diag_first_sight_value_mismatch[slot] += 1;
                            }
                        } else {
                            self.counts.diag_first_sight_missing[slot] += 1;
                        }
                    }
                }
                None => {
                    self.counts.emitted_null[slot] += 1;
                    match anchor_ts {
                        Some(_) => self.counts.spurious_live[slot] += 1,
                        None => self.counts.anchor_null[slot] += 1,
                    }
                    if first_sight.is_some() {
                        self.counts.diag_first_sight_spurious[slot] += 1;
                    }
                }
            }
        }
        let gt = [truth.strong_ts, truth.decay_risk_ts, truth.first_dying_ts];
        for (offset, gt_ms) in gt.iter().enumerate() {
            let idx = offset + 2;
            let anchor = &track.anchors[idx];
            match (*gt_ms, anchor) {
                (Some(g), Some(Anchor::Recorded { tick_ms, snapshot_ms })) => {
                    self.counts.gt_present[idx] += 1;
                    let g_ns = g.saturating_mul(1_000_000);
                    if track.left_truncated && g_ns < leaf_first_ns {
                        self.counts.excluded_left_truncated[idx] += 1;
                        continue;
                    }
                    let live_ms = match snapshot_ms {
                        Some(snapshot) => *snapshot,
                        None => *tick_ms,
                    };
                    if live_ms == g {
                        self.counts.matched[idx] += 1;
                    } else {
                        self.counts.value_mismatch[idx] += 1;
                    }
                    if *tick_ms != g {
                        self.counts.tick_lag[idx] += 1;
                    }
                }
                (Some(g), Some(Anchor::Absorbed)) => {
                    self.counts.gt_present[idx] += 1;
                    let g_ns = g.saturating_mul(1_000_000);
                    if g_ns > leaf_first_ns {
                        self.counts.value_mismatch[idx] += 1;
                    } else if g_ns == leaf_first_ns {
                        self.counts.boundary_ambiguous[idx] += 1;
                    } else {
                        self.counts.excluded_left_truncated[idx] += 1;
                    }
                }
                (Some(g), None) => {
                    self.counts.gt_present[idx] += 1;
                    let g_ns = g.saturating_mul(1_000_000);
                    if track.left_truncated && g_ns < leaf_first_ns {
                        self.counts.excluded_left_truncated[idx] += 1;
                    } else {
                        self.counts.missing_live[idx] += 1;
                    }
                }
                (None, Some(Anchor::Recorded { .. })) => {
                    self.counts.spurious_live[idx] += 1;
                }
                (None, Some(Anchor::Absorbed)) => {
                    self.counts.excluded_left_truncated[idx] += 1;
                }
                (None, None) => {}
            }
        }
    }

    pub fn mismatches_total(&self) -> u64 {
        let c = &self.counts;
        (0..5)
            .map(|i| c.value_mismatch[i] + c.missing_live[i] + c.spurious_live[i])
            .sum()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Contract loading and G1
// ---------------------------------------------------------------------------

pub struct Contract {
    pub contract_id: String,
    pub frozen_utc: String,
    pub path: PathBuf,
    pub lake_root: PathBuf,
    pub manifest_sha256: String,
    pub manifest_bytes: u64,
    pub payload_manifest_sha256: String,
    pub payload_manifest_bytes: u64,
    pub tick_parts_expected: u64,
    pub tick_rows_expected: u64,
    pub cutoff_ns: i64,
    pub cutoff_utc: String,
    pub contract_sha256: String,
    pub contract_bytes: u64,
}

fn sha256_file(path: &Path) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut size = 0u64;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        size += read as u64;
        hasher.update(&buf[..read]);
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

pub fn load_contract(path: &Path) -> Result<Contract, Box<dyn std::error::Error>> {
    let raw = std::fs::read(path)?;
    let value: Value = serde_json::from_slice(&raw)?;
    let identity = value
        .get("input_identity")
        .ok_or("contract missing input_identity")?;
    let manifest = identity
        .get("manifest_json")
        .ok_or("contract missing input_identity.manifest_json")?;
    let payload_manifest = identity
        .get("payload_manifest_json")
        .ok_or("contract missing input_identity.payload_manifest_json")?;
    let partition = value
        .get("partition_policy")
        .ok_or("contract missing partition_policy")?;
    let get_u64 = |v: &Value, key: &str| -> Result<u64, Box<dyn std::error::Error>> {
        v.get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("contract field missing or not u64: {key}").into())
    };
    let get_str = |v: &Value, key: &str| -> Result<String, Box<dyn std::error::Error>> {
        v.get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("contract field missing or not string: {key}").into())
    };
    Ok(Contract {
        contract_id: get_str(&value, "contract_id")?,
        frozen_utc: get_str(&value, "frozen_utc").unwrap_or_default(),
        contract_sha256: {
            let mut hasher = Sha256::new();
            hasher.update(&raw);
            format!("{:x}", hasher.finalize())
        },
        contract_bytes: raw.len() as u64,
        path: path.to_path_buf(),
        lake_root: PathBuf::from(get_str(identity, "lake_root")?),
        manifest_sha256: get_str(manifest, "sha256")?,
        manifest_bytes: get_u64(manifest, "bytes")?,
        payload_manifest_sha256: get_str(payload_manifest, "sha256")?,
        payload_manifest_bytes: get_u64(payload_manifest, "bytes")?,
        tick_parts_expected: get_u64(identity, "tick_parts_expected")?,
        tick_rows_expected: get_u64(identity, "tick_rows_expected")?,
        cutoff_ns: partition
            .get("cutoff_ns")
            .and_then(Value::as_i64)
            .ok_or("contract field missing or not i64: partition_policy.cutoff_ns")?,
        cutoff_utc: get_str(partition, "cutoff_utc")?,
    })
}

struct IdentityCheck {
    expected_sha: String,
    actual_sha: String,
    expected_bytes: u64,
    actual_bytes: u64,
    pass: bool,
}

fn check_identity(
    lake_root: &Path,
    relative: &str,
    expected_sha: &str,
    expected_bytes: u64,
) -> Result<IdentityCheck, Box<dyn std::error::Error>> {
    let (actual_sha, actual_bytes) = sha256_file(&lake_root.join(relative))?;
    let pass = actual_sha == expected_sha && actual_bytes == expected_bytes;
    Ok(IdentityCheck {
        expected_sha: expected_sha.to_owned(),
        actual_sha,
        expected_bytes,
        actual_bytes,
        pass,
    })
}

// ---------------------------------------------------------------------------
// Per-part inventory and scan
// ---------------------------------------------------------------------------

struct PartInventory {
    name: String,
    rows: u64,
    row_groups: usize,
    event_min_ns: Option<i64>,
    event_max_ns: Option<i64>,
    recv_min_ns: Option<i64>,
    recv_max_ns: Option<i64>,
    stats_present: bool,
    scanned_rows: u64,
    scan_policy: &'static str,
    manifest_rows: Option<u64>,
    manifest_rows_match: Option<bool>,
}

fn column_stats(
    metadata: &parquet::file::metadata::ParquetMetaData,
    column: &str,
) -> (Option<i64>, Option<i64>, bool) {
    let mut min = None;
    let mut max = None;
    let mut all_present = true;
    for group in metadata.row_groups() {
        for chunk in group.columns() {
            if chunk.column_path().string() != column {
                continue;
            }
            match chunk.statistics() {
                Some(ParquetStatistics::Int64(stats)) => {
                    if let Some(value) = stats.min_opt() {
                        min = Some(min.map_or(*value, |m: i64| m.min(*value)));
                    }
                    if let Some(value) = stats.max_opt() {
                        max = Some(max.map_or(*value, |m: i64| m.max(*value)));
                    }
                }
                _ => all_present = false,
            }
        }
    }
    (min, max, all_present)
}

fn str_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<StrCol<'a>, String> {
    let array = batch
        .column_by_name(name)
        .ok_or_else(|| format!("column missing: {name}"))?;
    if let Some(large) = array.as_any().downcast_ref::<LargeStringArray>() {
        return Ok(StrCol::Large(large));
    }
    if let Some(small) = array.as_any().downcast_ref::<StringArray>() {
        return Ok(StrCol::Small(small));
    }
    Err(format!("column {name} is not a string array"))
}

enum StrCol<'a> {
    Large(&'a LargeStringArray),
    Small(&'a StringArray),
}
impl<'a> StrCol<'a> {
    fn is_null(&self, row: usize) -> bool {
        match self {
            StrCol::Large(a) => a.is_null(row),
            StrCol::Small(a) => a.is_null(row),
        }
    }
    fn value(&self, row: usize) -> &'a str {
        match self {
            StrCol::Large(a) => a.value(row),
            StrCol::Small(a) => a.value(row),
        }
    }
}

#[derive(Default)]
struct G2Counts {
    rows_checked: u64,
    ordering_regressions: u64,
    receive_before_event: u64,
    received_nulls: u64,
    event_ts_nulls: u64,
    sequence_nulls: u64,
    bid_nulls: u64,
    ask_nulls: u64,
    drift_payload_nulls: u64,
    sub_ms_event_rows: u64,
    payload_parse_fallbacks: u64,
    malformed_emitted_entries: u64,
}

struct BoundaryInfo {
    part_index: usize,
    part_name: String,
    row_index_in_part: u64,
    row_group: usize,
    received_ts_ns: i64,
    event_ts_ns: i64,
}

struct ScanResult {
    dev_rows: u64,
    per_part: Vec<PartInventory>,
    g2: G2Counts,
    engine: Engine,
    boundary: Option<BoundaryInfo>,
    scan_wall: std::time::Duration,
    batch_size: usize,
}

fn verify_schema(schema: &arrow_schema::SchemaRef) -> Result<(), String> {
    let expected_types: [(&str, &str); 12] = [
        ("event_ts_ns", "Int64"),
        ("received_ts_ns", "Int64"),
        ("source_sequence", "Int64"),
        ("bid", "Float64"),
        ("ask", "Float64"),
        ("payload_drift_burst", "LargeUtf8"),
        ("payload_spread_state", "LargeUtf8"),
        ("payload_quote_dynamics", "LargeUtf8"),
        ("payload_quote_arrival", "LargeUtf8"),
        ("payload_micro_volatility", "LargeUtf8"),
        ("payload_feed_health", "LargeUtf8"),
        ("payload_quote_pressure", "LargeUtf8"),
    ];
    for (name, expected) in expected_types {
        let field = schema
            .field_with_name(name)
            .map_err(|_| format!("tick schema missing column {name}"))?;
        let actual = format!("{:?}", field.data_type());
        let ok = actual == expected
            || (name.starts_with("payload_") && actual == "Utf8");
        if !ok {
            return Err(format!(
                "tick schema mismatch for {name}: expected {expected}, actual {actual}"
            ));
        }
    }
    if schema.fields().len() != expected_types.len() {
        return Err(format!(
            "tick schema has {} columns, expected {}",
            schema.fields().len(),
            expected_types.len()
        ));
    }
    Ok(())
}

fn scan_leaf(
    contract: &Contract,
    parts: &[PathBuf],
    manifest_rows: &BTreeMap<String, u64>,
    batch_size: usize,
) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let cutoff = contract.cutoff_ns;
    let mut g2 = G2Counts::default();
    let mut engine = Engine::new();
    let mut per_part = Vec::new();
    let mut boundary: Option<BoundaryInfo> = None;
    let mut prev_pair: Option<(i64, i64)> = None;
    let mut dev_rows_total = 0u64;
    let started = Instant::now();

    'parts: for (part_index, path) in parts.iter().enumerate() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file = File::open(path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        if part_index == 0 {
            verify_schema(builder.schema()).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        }
        let metadata = builder.metadata().clone();
        let rows = metadata.file_metadata().num_rows() as u64;
        let row_groups = metadata.row_groups().len();
        let (event_min, event_max, event_stats) = column_stats(&metadata, "event_ts_ns");
        let (recv_min, recv_max, recv_stats) = column_stats(&metadata, "received_ts_ns");
        let stats_present = event_stats && recv_stats;

        // Partition policy from footer metadata: a part whose minimum
        // availability is at/after the cutoff is confirmation-only and its row
        // data is never opened.
        if let Some(min) = recv_min {
            if min >= cutoff {
                per_part.push(PartInventory {
                    name: name.clone(),
                    rows,
                    row_groups,
                    event_min_ns: event_min,
                    event_max_ns: event_max,
                    recv_min_ns: recv_min,
                    recv_max_ns: recv_max,
                    stats_present,
                    scanned_rows: 0,
                    scan_policy: "confirmation_leaf_footer_only",
                    manifest_rows: manifest_rows.get(&name).copied(),
                    manifest_rows_match: manifest_rows
                        .get(&name)
                        .map(|expected| *expected == rows),
                });
                eprintln!(
                    "[wp1] part {part_index} ({name}): footer only, {} rows, recv_min >= cutoff",
                    rows
                );
                continue;
            }
        }

        // Projection: WP2 benchmark column set only.
        let schema = builder.schema().clone();
        let indices = PROJECTION
            .iter()
            .map(|column| {
                schema
                    .index_of(column)
                    .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
            })
            .collect::<Result<Vec<usize>, _>>()?;
        let mask = ProjectionMask::roots(builder.parquet_schema(), indices);
        let reader = builder
            .with_projection(mask)
            .with_batch_size(batch_size)
            .build()?;

        // Row-group row offsets for reporting the boundary row group.
        let mut group_offsets = Vec::with_capacity(row_groups);
        let mut acc = 0u64;
        for group in metadata.row_groups() {
            group_offsets.push(acc);
            acc += group.num_rows() as u64;
        }

        let mut scanned_in_part = 0u64;
        let mut policy = "full_development_scan";
        'batches: for batch in reader {
            let batch = batch?;
            let num_rows = batch.num_rows();
            let event_col = batch
                .column_by_name("event_ts_ns")
                .and_then(|a| a.as_any().downcast_ref::<Int64Array>())
                .ok_or("event_ts_ns column missing or not Int64")?;
            let recv_col = batch
                .column_by_name("received_ts_ns")
                .and_then(|a| a.as_any().downcast_ref::<Int64Array>())
                .ok_or("received_ts_ns column missing or not Int64")?;
            let seq_col = batch
                .column_by_name("source_sequence")
                .and_then(|a| a.as_any().downcast_ref::<Int64Array>())
                .ok_or("source_sequence column missing or not Int64")?;
            let bid_col = batch
                .column_by_name("bid")
                .and_then(|a| a.as_any().downcast_ref::<Float64Array>())
                .ok_or("bid column missing or not Float64")?;
            let ask_col = batch
                .column_by_name("ask")
                .and_then(|a| a.as_any().downcast_ref::<Float64Array>())
                .ok_or("ask column missing or not Float64")?;
            let payload_col = str_column(&batch, "payload_drift_burst")?;

            for row in 0..num_rows {
                if recv_col.is_null(row) {
                    g2.received_nulls += 1;
                } else {
                    let recv = recv_col.value(row);
                    if recv >= cutoff {
                        // Confirmation leaf starts here: stop, never evaluate
                        // this or any later row. Rows before this one in the
                        // batch were scanned; this row and everything after it
                        // is confirmation leaf.
                        scanned_in_part += row as u64;
                        let row_index_in_part = scanned_in_part;
                        let row_group = group_offsets
                            .iter()
                            .rposition(|offset| *offset <= row_index_in_part)
                            .unwrap_or(0);
                        boundary = Some(BoundaryInfo {
                            part_index,
                            part_name: name.clone(),
                            row_index_in_part,
                            row_group,
                            received_ts_ns: recv,
                            event_ts_ns: if event_col.is_null(row) {
                                i64::MIN
                            } else {
                                event_col.value(row)
                            },
                        });
                        policy = "stopped_at_cutoff";
                        break 'batches;
                    }
                }
                // --- development leaf row ---
                g2.rows_checked += 1;
                dev_rows_total += 1;
                let recv = if recv_col.is_null(row) {
                    None
                } else {
                    Some(recv_col.value(row))
                };
                let seq = if seq_col.is_null(row) {
                    g2.sequence_nulls += 1;
                    None
                } else {
                    Some(seq_col.value(row))
                };
                match (recv, seq) {
                    (Some(r), Some(s)) => {
                        if let Some(previous) = prev_pair {
                            if (r, s) < previous {
                                g2.ordering_regressions += 1;
                            }
                        }
                        prev_pair = Some((r, s));
                    }
                    _ => {}
                }
                if payload_col.is_null(row) {
                    g2.drift_payload_nulls += 1;
                }
                if event_col.is_null(row) {
                    g2.event_ts_nulls += 1;
                } else {
                    let event = event_col.value(row);
                    if let Some(r) = recv {
                        if r < event {
                            g2.receive_before_event += 1;
                        }
                    }
                    if event.rem_euclid(1_000_000) != 0 {
                        g2.sub_ms_event_rows += 1;
                    }
                    if !payload_col.is_null(row) {
                        let payload = payload_col.value(row);
                        let parsed = parse_payload(
                            payload,
                            &mut g2.payload_parse_fallbacks,
                            &mut g2.malformed_emitted_entries,
                        );
                        if let ParsedPayload::Snapshot(snapshot) = parsed {
                            engine.observe(event, recv, seq, Some(payload), Some(&snapshot));
                        }
                    }
                }
                if bid_col.is_null(row) {
                    g2.bid_nulls += 1;
                }
                if ask_col.is_null(row) {
                    g2.ask_nulls += 1;
                }
            }
            scanned_in_part += num_rows as u64;
        }
        per_part.push(PartInventory {
            name: name.clone(),
            rows,
            row_groups,
            event_min_ns: event_min,
            event_max_ns: event_max,
            recv_min_ns: recv_min,
            recv_max_ns: recv_max,
            stats_present,
            scanned_rows: scanned_in_part,
            scan_policy: policy,
            manifest_rows: manifest_rows.get(&name).copied(),
            manifest_rows_match: manifest_rows.get(&name).map(|expected| *expected == rows),
        });
        eprintln!(
            "[wp1] part {part_index} ({name}): scanned={scanned_in_part} dev_total={dev_rows_total} elapsed={:.1}s",
            started.elapsed().as_secs_f64()
        );
        if boundary.is_some() {
            break 'parts;
        }
    }

    // Footer-only inventory for parts the scan never reached (after the
    // cutoff stop): parquet metadata only, row data never opened.
    for (part_index, path) in parts.iter().enumerate().skip(per_part.len()) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file = File::open(path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let metadata = builder.metadata().clone();
        let (event_min, event_max, event_stats) = column_stats(&metadata, "event_ts_ns");
        let (recv_min, recv_max, recv_stats) = column_stats(&metadata, "received_ts_ns");
        per_part.push(PartInventory {
            name: name.clone(),
            rows: metadata.file_metadata().num_rows() as u64,
            row_groups: metadata.row_groups().len(),
            event_min_ns: event_min,
            event_max_ns: event_max,
            recv_min_ns: recv_min,
            recv_max_ns: recv_max,
            stats_present: event_stats && recv_stats,
            scanned_rows: 0,
            scan_policy: "confirmation_leaf_footer_only",
            manifest_rows: manifest_rows.get(&name).copied(),
            manifest_rows_match: manifest_rows.get(&name).map(|expected| *expected == metadata.file_metadata().num_rows() as u64),
        });
        eprintln!(
            "[wp1] part {part_index} ({name}): footer only, {} rows, recv_min={:?}",
            metadata.file_metadata().num_rows(),
            recv_min
        );
    }

    engine.finish();
    Ok(ScanResult {
        dev_rows: dev_rows_total,
        per_part,
        g2,
        engine,
        boundary,
        scan_wall: started.elapsed(),
        batch_size,
    })
}

// ---------------------------------------------------------------------------
// Preflight + report output
// ---------------------------------------------------------------------------

struct ScannerFile {
    path: String,
    sha256: String,
    bytes: u64,
}

fn scanner_files() -> Result<Vec<ScannerFile>, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let relative = [
        "src/ap001_drift_burst.rs",
        "src/bin/ap001_wp1_contract.rs",
        "src/lib.rs",
        "Cargo.toml",
    ];
    let mut files = Vec::new();
    for rel in relative {
        let path = manifest_dir.join(rel);
        let (sha, bytes) = sha256_file(&path)?;
        files.push(ScannerFile {
            path: rel.to_owned(),
            sha256: sha,
            bytes,
        });
    }
    Ok(files)
}

fn rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn git_identity() -> (Option<String>, u64) {
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned());
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).lines().count() as u64)
        .unwrap_or(0);
    (head, dirty)
}

pub struct Wp1Config {
    pub contract_path: PathBuf,
    pub lake_root: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub batch_size: usize,
}

fn anchor_array(counts: &[u64; 5]) -> Value {
    json!({
        ANCHOR_LABELS[0]: counts[0],
        ANCHOR_LABELS[1]: counts[1],
        ANCHOR_LABELS[2]: counts[2],
        ANCHOR_LABELS[3]: counts[3],
        ANCHOR_LABELS[4]: counts[4],
    })
}

fn state_label(code: u8) -> &'static str {
    // Matches DState declaration order.
    match code {
        0 => "IDLE",
        1 => "DEVELOPING",
        2 => "BURST",
        3 => "DECAY_RISK",
        4 => "DYING",
        _ => "OTHER",
    }
}

/// Run WP1 end to end. Returns the process exit code (0 pass, 1 gate failure).
pub fn run(config: Wp1Config) -> Result<i32, Box<dyn std::error::Error>> {
    let run_started = Instant::now();
    let contract = load_contract(&config.contract_path)?;
    let lake_root = config.lake_root.clone().unwrap_or_else(|| contract.lake_root.clone());
    let out_dir = config
        .out_dir
        .clone()
        .unwrap_or_else(|| contract.path.parent().map(Path::to_path_buf).unwrap_or_default());
    let batch_size = config.batch_size;

    eprintln!("[wp1] contract {} frozen; cutoff {} ({} ns)", contract.contract_id, contract.cutoff_utc, contract.cutoff_ns);

    // ---- G1 ----
    let manifest_check = check_identity(
        &lake_root,
        "manifest.json",
        &contract.manifest_sha256,
        contract.manifest_bytes,
    )?;
    let payload_check = check_identity(
        &lake_root,
        "payload_manifest.json",
        &contract.payload_manifest_sha256,
        contract.payload_manifest_bytes,
    )?;
    let g1_pass = manifest_check.pass && payload_check.pass;
    eprintln!("[wp1] G1 manifest match={}", manifest_check.pass);
    eprintln!("[wp1] G1 payload_manifest match={}", payload_check.pass);
    if !g1_pass {
        eprintln!("[wp1] G1 FAILED: aborting before any scan");
    }

    // ---- inventory (footer metadata; row data only inside the leaf) ----
    let mut parts: Vec<PathBuf> = Vec::new();
    let tick_dir = lake_root.join("tick");
    for entry in std::fs::read_dir(&tick_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
            parts.push(path);
        }
    }
    parts.sort();
    let manifest_rows = parse_manifest_tick_rows(&lake_root)?;

    let mut scan: Option<ScanResult> = None;
    if g1_pass {
        let result = scan_leaf(&contract, &parts, &manifest_rows, batch_size)?;
        eprintln!(
            "[wp1] scan complete: dev_rows={} wall={:.1}s regressions={} recv_before_event={}",
            result.dev_rows,
            result.scan_wall.as_secs_f64(),
            result.g2.ordering_regressions,
            result.g2.receive_before_event
        );
        scan = Some(result);
    }

    let (head_sha, dirty_files) = git_identity();
    let files = scanner_files()?;
    let scanner_content_sha256 = {
        let mut hasher = Sha256::new();
        for file in &files {
            hasher.update(file.path.as_bytes());
            hasher.update([0]);
            hasher.update(file.sha256.as_bytes());
            hasher.update([0]);
        }
        format!("{:x}", hasher.finalize())
    };

    // ---- assemble preflight core (deterministic; no wall clock) ----
    let mut core = Map::new();
    core.insert("contract_id".into(), json!(contract.contract_id));
    core.insert("contract_path".into(), json!(contract.path.display().to_string()));
    core.insert("contract_sha256".into(), json!(contract.contract_sha256));
    core.insert(
        "contract_amendment".into(),
        json!({"decision": "D-005", "scope": "anchor_definitions WEAK/ONLINE; STRONG/DECAY_RISK/DYING unchanged; recorded in AP-001_DECISION_LOG.md"}),
    );
    core.insert("lake_root".into(), json!(lake_root.display().to_string()));

    let g1 = json!({
        "manifest_json": {
            "path": "manifest.json",
            "expected_sha256": manifest_check.expected_sha,
            "actual_sha256": manifest_check.actual_sha,
            "expected_bytes": manifest_check.expected_bytes,
            "actual_bytes": manifest_check.actual_bytes,
            "match": manifest_check.pass,
        },
        "payload_manifest_json": {
            "path": "payload_manifest.json",
            "expected_sha256": payload_check.expected_sha,
            "actual_sha256": payload_check.actual_sha,
            "expected_bytes": payload_check.expected_bytes,
            "actual_bytes": payload_check.actual_bytes,
            "match": payload_check.pass,
        },
        "pass": g1_pass,
    });
    core.insert("g1_input_identity".into(), g1);

    match &scan {
        Some(result) => {
            let total_rows: u64 = result.per_part.iter().map(|p| p.rows).sum();
            let parts_json: Vec<Value> = result
                .per_part
                .iter()
                .map(|p| {
                    json!({
                        "name": p.name,
                        "footer_rows": p.rows,
                        "row_groups": p.row_groups,
                        "event_ts_min_ns": p.event_min_ns,
                        "event_ts_max_ns": p.event_max_ns,
                        "received_ts_min_ns": p.recv_min_ns,
                        "received_ts_max_ns": p.recv_max_ns,
                        "footer_stats_present": p.stats_present,
                        "scanned_rows": p.scanned_rows,
                        "scan_policy": p.scan_policy,
                        "manifest_rows": p.manifest_rows,
                        "manifest_rows_match": p.manifest_rows_match,
                    })
                })
                .collect();
            core.insert("inventory".into(), json!({
                "parts": parts_json,
                "tick_parts_expected": contract.tick_parts_expected,
                "tick_parts_found": parts.len(),
                "parts_match": parts.len() as u64 == contract.tick_parts_expected,
                "total_footer_rows": total_rows,
                "tick_rows_expected": contract.tick_rows_expected,
                "rows_match": total_rows == contract.tick_rows_expected,
                "manifest_row_crosscheck_all_match": result.per_part.iter().all(|p| p.manifest_rows_match.unwrap_or(true)),
            }));

            let boundary = result.boundary.as_ref().map(|b| {
                json!({
                    "part_index": b.part_index,
                    "part_name": b.part_name,
                    "row_index_in_part": b.row_index_in_part,
                    "row_group": b.row_group,
                    "first_confirmation_row_received_ts_ns": b.received_ts_ns,
                    "first_confirmation_row_event_ts_ns": b.event_ts_ns,
                })
            });
            core.insert("partition".into(), json!({
                "cutoff_utc": contract.cutoff_utc,
                "cutoff_ns": contract.cutoff_ns,
                "development_leaf_rule": "received_ts_ns strictly before cutoff_ns, stream order; scan stops at the first row at/after the cutoff",
                "confirmation_leaf_policy": "no row data evaluated at or after cutoff; later parts recorded from parquet footer metadata only",
                "boundary": boundary,
            }));

            let g = &result.g2;
            core.insert("g2_causal_ordering".into(), json!({
                "rows_checked": g.rows_checked,
                "ordering_regressions": g.ordering_regressions,
                "receive_before_event": g.receive_before_event,
                "received_ts_nulls": g.received_nulls,
                "event_ts_nulls": g.event_ts_nulls,
                "source_sequence_nulls": g.sequence_nulls,
                "sub_ms_event_ts_rows": g.sub_ms_event_rows,
                "null_payloads": {
                    "payload_drift_burst": g.drift_payload_nulls,
                    "bid": g.bid_nulls,
                    "ask": g.ask_nulls,
                    NON_PROJECTED_PAYLOADS[0]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                    NON_PROJECTED_PAYLOADS[1]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                    NON_PROJECTED_PAYLOADS[2]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                    NON_PROJECTED_PAYLOADS[3]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                    NON_PROJECTED_PAYLOADS[4]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                    NON_PROJECTED_PAYLOADS[5]: "not_measured (outside WP2 projection; footer statistics absent for string columns)",
                },
                "payload_parse_fallbacks": g.payload_parse_fallbacks,
                "malformed_emitted_entries": g.malformed_emitted_entries,
                "pass": g.ordering_regressions == 0 && g.receive_before_event == 0,
            }));

            let counts = result.engine.counts();
            let mismatches_total = result.engine.mismatches_total();
            let (record_hashes, record_counts) = result.engine.anchor_record_hashes();
            let g3_pass = mismatches_total == 0
                && counts.entries_without_track == 0
                && counts.closed_tracks_without_emission == 0
                && counts.malformed_entries == 0;
            core.insert("g3_anchor_sanity".into(), json!({
                "episodes_completed_in_leaf": counts.episodes_completed_in_leaf,
                "tracks_started": counts.tracks_started,
                "tracks_completed": counts.tracks_completed,
                "left_truncated_tracks": counts.left_truncated_tracks,
                "forming_episodes_ended_without_burst": counts.forming_fizzled,
                "right_censored_forming": counts.right_censored_forming,
                "right_censored_armed": counts.right_censored_armed,
                "emitted_entries_seen": counts.emitted_entries,
                "emitted_entries_at_leaf_start": counts.emitted_at_leaf_start,
                "pre_leaf_history_entries": counts.pre_leaf_history,
                "emitted_entry_mutations": counts.emitted_mutations,
                "duplicate_emission_sightings": counts.duplicate_sightings,
                "malformed_entries": counts.malformed_entries,
                "entries_without_track": counts.entries_without_track,
                "closed_tracks_without_emission": counts.closed_tracks_without_emission,
                "duplicate_track_ids": counts.duplicate_track_ids,
                "unknown_state_rows": counts.unknown_states,
                "anchors_detected_armed_episodes": anchor_array(&counts.anchors_detected),
                "ground_truth_present": anchor_array(&counts.gt_present),
                "anchors_matched": anchor_array(&counts.matched),
                "mismatches": {
                    "WEAK": counts.value_mismatch[A_WEAK] + counts.missing_live[A_WEAK] + counts.spurious_live[A_WEAK],
                    "ONLINE": counts.value_mismatch[A_ONLINE] + counts.missing_live[A_ONLINE] + counts.spurious_live[A_ONLINE],
                    "STRONG": counts.value_mismatch[A_STRONG] + counts.missing_live[A_STRONG] + counts.spurious_live[A_STRONG],
                    "DECAY_RISK": counts.value_mismatch[A_DECAY] + counts.missing_live[A_DECAY] + counts.spurious_live[A_DECAY],
                    "DYING": counts.value_mismatch[A_DYING] + counts.missing_live[A_DYING] + counts.spurious_live[A_DYING],
                    "value_mismatch": anchor_array(&counts.value_mismatch),
                    "missing_live": anchor_array(&counts.missing_live),
                    "spurious_live": anchor_array(&counts.spurious_live),
                    "total": mismatches_total,
                },
                "tick_lag_informational": anchor_array(&counts.tick_lag),
                "excluded_left_truncated": anchor_array(&counts.excluded_left_truncated),
                "boundary_ambiguous": anchor_array(&counts.boundary_ambiguous),
                "end_ts_vs_appearance_tick": {
                    "checked": counts.end_ts_vs_appearance_checked,
                    "mismatch": counts.end_ts_vs_appearance_mismatch,
                },
                "state_transitions": result
                    .engine
                    .transitions
                    .iter()
                    .map(|((from, to), count)| {
                        json!({"from": state_label(*from), "to": state_label(*to), "count": count})
                    })
                    .collect::<Vec<_>>(),
                "amendment": {
                    "decision": "D-006 (mechanics; supersedes D-005 for WEAK/ONLINE, which superseded the run-1 frozen rule)",
                    "rule": "milestone candidate capture: milestones.weak/.online transitioning null->value creates a candidate (crossing tick received_ts/source_sequence + at-crossing snapshot record); producer value replacement replaces the candidate; at the episode's arm (first BURST tick) the surviving candidate becomes the anchor; null milestone at arm means no anchor; STRONG/DECAY_RISK/DYING unchanged",
                    "null_agreement": {
                        "note": "milestone null at arm vs emitted null must agree in both directions (equivalently missing_live == spurious_live == 0)",
                        "emitted_null": {"WEAK": counts.emitted_null[0], "ONLINE": counts.emitted_null[1]},
                        "anchor_null": {"WEAK": counts.anchor_null[0], "ONLINE": counts.anchor_null[1]},
                    },
                    "anchor_records": {
                        "note": "at-crossing snapshot records (milestone ts, crossing availability, state, milestones crossed, operational/multiscale/authority), canonical serde_json ordered by event_id",
                        "count": {"WEAK": record_counts[0], "ONLINE": record_counts[1]},
                        "sha256": {"WEAK": record_hashes[0], "ONLINE": record_hashes[1]},
                        "extraction_failures": counts.anchor_record_extraction_failures,
                    },
                    "forming_attempts_ended_without_arming": counts.forming_fizzled,
                },
                "pre_amendment_first_sighting_diagnostic": {
                    "note": "run-1 frozen rule (first sighting in any phase) reproduced as a diagnostic only, not the gate",
                    "value_mismatch": {"WEAK": counts.diag_first_sight_value_mismatch[0], "ONLINE": counts.diag_first_sight_value_mismatch[1]},
                    "missing_live": {"WEAK": counts.diag_first_sight_missing[0], "ONLINE": counts.diag_first_sight_missing[1]},
                    "spurious": {"WEAK": counts.diag_first_sight_spurious[0], "ONLINE": counts.diag_first_sight_spurious[1]},
                },
                "pass": g3_pass,
            }));

            let rows_per_second = if result.scan_wall.as_secs_f64() > 0.0 {
                result.dev_rows as f64 / result.scan_wall.as_secs_f64()
            } else {
                0.0
            };
            let full_tape_eta = if rows_per_second > 0.0 {
                contract.tick_rows_expected as f64 / rows_per_second
            } else {
                0.0
            };
            core.insert("g4_benchmark".into(), json!({
                "projection": PROJECTION,
                "batch_size": result.batch_size,
                "development_leaf_rows": result.dev_rows,
                "scan_wall_seconds": result.scan_wall.as_secs_f64(),
                "rows_per_second": rows_per_second,
                "eta_development_leaf_seconds": result.scan_wall.as_secs_f64(),
                "full_tape_rows": contract.tick_rows_expected,
                "full_tape_eta_seconds": full_tape_eta,
                "peak_rss_bytes": peak_rss_bytes(),
                "rss_method": if cfg!(windows) { "GetProcessMemoryInfo PeakWorkingSetSize (K32) at end of run" } else { "not available on this platform" },
                "run_wall_seconds_total": run_started.elapsed().as_secs_f64(),
            }));
        }
        None => {
            core.insert("inventory".into(), json!({"skipped": "G1 failed"}));
            core.insert("g2_causal_ordering".into(), json!({"skipped": "G1 failed", "pass": false}));
            core.insert("g3_anchor_sanity".into(), json!({"skipped": "G1 failed", "pass": false}));
            core.insert("g4_benchmark".into(), json!({"skipped": "G1 failed", "pass": false}));
        }
    }

    core.insert("scanner".into(), json!({
        "crate": "research-tape",
        "module": "src/ap001_drift_burst.rs",
        "binary": "src/bin/ap001_wp1_contract.rs",
        "files": files.iter().map(|f| json!({
            "path": f.path,
            "sha256": f.sha256,
            "bytes": f.bytes,
        })).collect::<Vec<_>>(),
        "scanner_content_sha256": scanner_content_sha256,
        "git_head": head_sha,
        "git_dirty_files": dirty_files,
        "commit_note": "WP1 directive forbids committing; code identity recorded as file hashes and git state",
    }));

    // Deterministic hash: compact serialization of the core without the hash
    // and without the wall-clock field.
    let core_value = Value::Object(core.clone());
    let canonical = serde_json::to_string(&core_value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let content_sha256 = format!("{:x}", hasher.finalize());
    core.insert("content_sha256".into(), json!(content_sha256));
    core.insert(
        "content_sha256_scope".into(),
        json!("sha256 of the compact serde_json serialization (BTreeMap key order) of this object excluding content_sha256 and run_wall_clock_utc"),
    );
    core.insert("run_wall_clock_utc".into(), json!(rfc3339_now()));

    let preflight_path = out_dir.join(PREFLIGHT_NAME);
    std::fs::write(&preflight_path, serde_json::to_vec_pretty(&Value::Object(core))?)?;
    eprintln!("[wp1] preflight written: {}", preflight_path.display());

    // ---- human report ----
    let report = build_report(&contract, &scan, &preflight_path, content_sha256, run_started.elapsed());
    let report_path = out_dir.join(REPORT_NAME);
    std::fs::write(&report_path, report)?;
    eprintln!("[wp1] report written: {}", report_path.display());

    let all_pass = g1_pass && {
        match &scan {
            Some(result) => {
                result.g2.ordering_regressions == 0
                    && result.g2.receive_before_event == 0
                    && result.engine.mismatches_total() == 0
                    && result.engine.counts().entries_without_track == 0
                    && result.engine.counts().closed_tracks_without_emission == 0
                    && result.engine.counts().malformed_entries == 0
            }
            None => false,
        }
    };
    Ok(if all_pass { 0 } else { 1 })
}

fn parse_manifest_tick_rows(lake_root: &Path) -> Result<BTreeMap<String, u64>, Box<dyn std::error::Error>> {
    let raw = std::fs::read(lake_root.join("manifest.json"))?;
    let value: Value = serde_json::from_slice(&raw)?;
    let mut map = BTreeMap::new();
    if let Some(parts) = value.get("tick_parts").and_then(Value::as_array) {
        for part in parts {
            if let (Some(path), Some(rows)) =
                (part.get("path").and_then(Value::as_str), part.get("rows").and_then(Value::as_u64))
            {
                let name = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_owned());
                map.insert(name, rows);
            }
        }
    }
    Ok(map)
}

fn build_report(
    contract: &Contract,
    scan: &Option<ScanResult>,
    preflight_path: &Path,
    content_sha256: String,
    total_wall: std::time::Duration,
) -> String {
    let mut out = String::new();
    out.push_str("# AP-001 WP1 Preflight Report\n\n");
    out.push_str(&format!(
        "- Contract: `{}` (frozen {})\n",
        contract.contract_id, contract.frozen_utc
    ));
    out.push_str(
        "- Amendment: **D-005** applied — WEAK/ONLINE anchors are now first milestone sightings made while the snapshot's event_id is bound to the armed episode (pre-origin sightings treated as carryover); STRONG/DECAY_RISK/DYING unchanged.\n",
    );
    out.push_str(&format!(
        "- Partition: development leaf = `received_ts_ns < {}` ({}); confirmation leaf untouched\n",
        contract.cutoff_ns, contract.cutoff_utc
    ));
    out.push_str(&format!(
        "- Preflight: `{}` (content sha256 `{content_sha256}`)\n\n",
        preflight_path.display()
    ));

    out.push_str("## Gates\n\n");
    out.push_str("| Gate | Result | Key numbers |\n|---|---|---|\n");
    match scan {
        None => {
            out.push_str("| G1 input identity | FAIL | manifest/payload SHA256 or size mismatch; scan aborted |\n");
            out.push_str("| G2 causal ordering | n/a | not run (G1 failed) |\n");
            out.push_str("| G3 anchor sanity | n/a | not run (G1 failed) |\n");
            out.push_str("| G4 benchmark | n/a | not run (G1 failed) |\n");
        }
        Some(result) => {
            let g1 = {
                let _ = result;
                true
            };
            out.push_str(&format!(
                "| G1 input identity | {} | manifest.json + payload_manifest.json SHA256 and byte size match contract |\n",
                if g1 { "PASS" } else { "FAIL" }
            ));
            let g = &result.g2;
            out.push_str(&format!(
                "| G2 causal ordering | {} | {} rows; regressions={}; received<event={} |\n",
                if g.ordering_regressions == 0 && g.receive_before_event == 0 { "PASS" } else { "FAIL" },
                g.rows_checked,
                g.ordering_regressions,
                g.receive_before_event
            ));
            let c = result.engine.counts();
            let mm = result.engine.mismatches_total();
            out.push_str(&format!(
                "| G3 anchor sanity | {} | {} completed episodes cross-checked under D-005; mismatch total={}; entries_without_track={}; closed_without_emission={} |\n",
                if mm == 0 && c.entries_without_track == 0 && c.closed_tracks_without_emission == 0 && c.malformed_entries == 0 { "PASS" } else { "FAIL" },
                c.episodes_completed_in_leaf,
                mm,
                c.entries_without_track,
                c.closed_tracks_without_emission
            ));
            let rps = if result.scan_wall.as_secs_f64() > 0.0 {
                result.dev_rows as f64 / result.scan_wall.as_secs_f64()
            } else {
                0.0
            };
            out.push_str(&format!(
                "| G4 benchmark | PASS | {rps:.0} rows/s over {} rows in {:.1}s; peak RSS {} |\n",
                result.dev_rows,
                result.scan_wall.as_secs_f64(),
                peak_rss_bytes()
                    .map(|b| format!("{:.2} GiB", b as f64 / (1 << 30) as f64))
                    .unwrap_or_else(|| "not measured".into()),
            ));

            out.push_str("
## G3 traceability: run 1 / run 2 / run 3

");
            out.push_str("| Metric | Run 1 (frozen first-sighting rule) | Run 2 (D-005 armed sighting + carryover filter) | Run 3 (D-006 candidate capture) |
|---|---|---|---|
");
            out.push_str(&format!(
                "| WEAK mismatches | 10123 (10100 value + 23 spurious) | 395 missing (pre-origin standing values) | {} |
",
                c.value_mismatch[A_WEAK] + c.missing_live[A_WEAK] + c.spurious_live[A_WEAK]
            ));
            out.push_str(&format!(
                "| ONLINE mismatches | 4140 (4114 value + 26 spurious) | 75 missing (pre-origin standing values) | {} |
",
                c.value_mismatch[A_ONLINE] + c.missing_live[A_ONLINE] + c.spurious_live[A_ONLINE]
            ));
            out.push_str(&format!(
                "| STRONG / DECAY_RISK / DYING mismatches | 0 / 0 / 0 | 0 / 0 / 0 | {} / {} / {} |
",
                c.value_mismatch[A_STRONG] + c.missing_live[A_STRONG] + c.spurious_live[A_STRONG],
                c.value_mismatch[A_DECAY] + c.missing_live[A_DECAY] + c.spurious_live[A_DECAY],
                c.value_mismatch[A_DYING] + c.missing_live[A_DYING] + c.spurious_live[A_DYING]
            ));
            out.push_str("| Run-1 at-arm diagnostic | 24220/24220 WEAK, 24196/24196 ONLINE matched, 0 mismatch | - | superseded: the surviving candidate at arm is the gate anchor |
");
            out.push_str(&format!(
                "| Null agreement (anchor-null vs emitted-null, cross-checked episodes) | not measured | not measured | WEAK {}/{}; ONLINE {}/{} |

",
                c.anchor_null[0], c.emitted_null[0],
                c.anchor_null[1], c.emitted_null[1]
            ));

            out.push_str("
## Anchors (anchors frozen at arm / ground-truth present / matched)

");
            out.push_str("\n## Anchors (armed-episode detections / ground-truth present / matched)\n\n");
            out.push_str("| Anchor | detected | gt present | matched | value mismatch | missing | spurious |\n|---|---|---|---|---|---|---|\n");
            for (i, label) in ANCHOR_LABELS.iter().enumerate() {
                out.push_str(&format!(
                    "| {label} | {} | {} | {} | {} | {} | {} |\n",
                    c.anchors_detected[i],
                    c.gt_present[i],
                    c.matched[i],
                    c.value_mismatch[i],
                    c.missing_live[i],
                    c.spurious_live[i],
                ));
            }
            out.push_str(&format!(
                "\nEpisodes: completed in leaf {}; left-truncated {}; right-censored (forming {}, armed {}); forming ended without burst {}.\n",
                c.episodes_completed_in_leaf,
                c.left_truncated_tracks,
                c.right_censored_forming,
                c.right_censored_armed,
                c.forming_fizzled
            ));
            out.push_str(&format!(
                "Re-entry/occupancy signal: BURST re-entries from DECAY_RISK {}; from DYING {}; end-of-episode supersessions (state-episode end merged into the next forming tick) appear in the transition table below; emitted entry mutations {}; duplicate sightings {}.\n\n",
                result.engine.transitions.get(&(3, 2)).copied().unwrap_or(0),
                result.engine.transitions.get(&(4, 2)).copied().unwrap_or(0),
                c.emitted_mutations, c.duplicate_sightings
            ));

            out.push_str("## Per-part inventory\n\n");
            out.push_str("| Part | Footer rows | Scanned | Event ts range (ns) | Policy |\n|---|---|---|---|---|\n");
            for p in &result.per_part {
                out.push_str(&format!(
                    "| {} | {} | {} | {} .. {} | {} |\n",
                    p.name,
                    p.rows,
                    p.scanned_rows,
                    p.event_min_ns.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                    p.event_max_ns.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                    p.scan_policy,
                ));
            }
            if let Some(b) = &result.boundary {
                out.push_str(&format!(
                    "\nCutoff boundary: first confirmation row at part {} row {} (row group {}), received_ts_ns {}.\n",
                    b.part_index, b.row_index_in_part, b.row_group, b.received_ts_ns
                ));
            }

            out.push_str("\n## Benchmark and ETA\n\n");
            out.push_str(&format!(
                "- Development leaf: {} rows in {:.1}s = {:.0} rows/s (WP2 projection: drift_burst + ts + sequence + bid/ask, batch {})\n",
                result.dev_rows,
                result.scan_wall.as_secs_f64(),
                rps,
                result.batch_size
            ));
            out.push_str(&format!(
                "- WP2-scale full development-leaf scan ETA: {:.1}s wall (measured directly)\n",
                result.scan_wall.as_secs_f64()
            ));
            let full_tape = if rps > 0.0 {
                contract.tick_rows_expected as f64 / rps
            } else {
                0.0
            };
            out.push_str(&format!(
                "- Full-tape extrapolation ({} rows): {:.1} min\n",
                contract.tick_rows_expected,
                full_tape / 60.0
            ));
            out.push_str(&format!(
                "- Total WP1 wall including G1 and report: {:.1}s\n",
                total_wall.as_secs_f64()
            ));

            out.push_str("\n## Anomalies and observations\n\n");
            let mut anomalies = Vec::new();
            let mm_weak = c.value_mismatch[A_WEAK] + c.missing_live[A_WEAK] + c.spurious_live[A_WEAK];
            let mm_online = c.value_mismatch[A_ONLINE] + c.missing_live[A_ONLINE] + c.spurious_live[A_ONLINE];
            if mm_weak == 0 && mm_online == 0 {
                anomalies.push(format!(
                    "D-006 candidate capture reproduces emitted weak_ts/online_ts with zero mismatches: anchor null at arm vs emitted null agrees in both directions (WEAK {}/{}; ONLINE {}/{}); at-crossing anchor records hashed into the preflight for WP2 reuse.",
                    c.anchor_null[0], c.emitted_null[0],
                    c.anchor_null[1], c.emitted_null[1]
                ));
            }
            if g.sub_ms_event_rows > 0 {
                anomalies.push(format!("{} rows have sub-millisecond event_ts_ns", g.sub_ms_event_rows));
            }
            if g.drift_payload_nulls > 0 {
                anomalies.push(format!("{} rows have null payload_drift_burst", g.drift_payload_nulls));
            }
            if g.payload_parse_fallbacks > 0 {
                anomalies.push(format!("{} payloads required the Value fallback parser", g.payload_parse_fallbacks));
            }
            if c.unknown_states > 0 {
                anomalies.push(format!("{} rows carried an unknown detector state", c.unknown_states));
            }
            if c.emitted_mutations > 0 {
                anomalies.push(format!("{} emitted entries changed after first sighting", c.emitted_mutations));
            }
            if c.malformed_entries > 0 {
                anomalies.push(format!("{} emitted entries lacked event_id", c.malformed_entries));
            }
            if c.end_ts_vs_appearance_mismatch > 0 {
                anomalies.push(format!(
                    "{} emitted entries have end_ts != tick that revealed them (appearance lag)",
                    c.end_ts_vs_appearance_mismatch
                ));
            }
            if c.left_truncated_tracks > 0 {
                anomalies.push(format!(
                    "{} episode(s) in progress at leaf start were left-truncated (anchors present at leaf start excluded from cross-check)",
                    c.left_truncated_tracks
                ));
            }
            if c.right_censored_forming + c.right_censored_armed > 0 {
                anomalies.push(format!(
                    "{} episode(s) still active at the cutoff are right-censored (forming {}, armed {})",
                    c.right_censored_forming + c.right_censored_armed,
                    c.right_censored_forming,
                    c.right_censored_armed
                ));
            }
            if anomalies.is_empty() {
                out.push_str("- None observed.\n");
            } else {
                for a in anomalies {
                    out.push_str(&format!("- {a}\n"));
                }
            }

            out.push_str("\n### State transitions (consecutive state-bearing rows, episode boundaries included)\n\n");
            out.push_str("| From | To | Count |\n|---|---|---|\n");
            for ((from, to), count) in &result.engine.transitions {
                out.push_str(&format!(
                    "| {} | {} | {} |\n",
                    state_label(*from),
                    state_label(*to),
                    count
                ));
            }
        }
    }
    out.push_str(&format!(
        "\nGenerated {} (wall clock only; not part of the preflight hash).\n",
        rfc3339_now()
    ));
    out
}

#[cfg(windows)]
pub fn peak_rss_bytes() -> Option<u64> {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }
    unsafe {
        let mut counters = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };
        if K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) != 0 {
            Some(counters.peak_working_set_size as u64)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn peak_rss_bytes() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn emission(id: i64, weak: i64, online: i64, strong: i64, decay: i64, dying: i64, end: i64) -> Value {
        json!({
            "event_id": id,
            "weak_ts": weak,
            "online_ts": online,
            "strong_ts": strong,
            "decay_risk_ts": decay,
            "first_dying_ts": dying,
            "end_ts": end,
        })
    }

    fn snap(state: &str, eid: i64, weak: Option<i64>, online: Option<i64>, emissions: Vec<Value>) -> Snapshot {
        let emissions = emissions
            .iter()
            .map(|value| {
                let mut malformed = 0u64;
                let truth = truth_from_value(value, &mut malformed).expect("test emission has event_id");
                EmittedEntry {
                    truth,
                    hash: canonical_hash(value),
                }
            })
            .collect();
        Snapshot {
            state: DState::parse(state),
            eid_pos: if eid >= 0 { Some(eid) } else { None },
            weak_ms: weak,
            online_ms: online,
            emissions,
        }
    }

    fn unpack(parsed: ParsedPayload) -> Snapshot {
        match parsed {
            ParsedPayload::Snapshot(s) => s,
            ParsedPayload::NoStateObject => panic!("expected snapshot"),
        }
    }

    #[test]
    fn full_lifecycle_anchors_match() {
        let mut engine = Engine::new();
        let e7 = emission(7, 1000, 2000, 4000, 5000, 6000, 7000);
        let rows = [
            (500_000_000, snap("IDLE", -1, None, None, vec![])),
            (1_000_000_000, snap("DEVELOPING", -1, None, None, vec![])),
            (2_000_000_000, snap("DEVELOPING", -1, Some(1000), None, vec![])),
            (3_000_000_000, snap("DEVELOPING", -1, Some(1000), Some(2000), vec![])),
            (4_000_000_000, snap("BURST", 7, Some(1000), Some(2000), vec![])),
            (5_000_000_000, snap("DECAY_RISK", 7, Some(1000), Some(2000), vec![])),
            (6_000_000_000, snap("DYING", 7, Some(1000), Some(2000), vec![])),
            (7_000_000_000, snap("DEVELOPING", -1, Some(7000), None, vec![e7.clone()])),
            (8_000_000_000, snap("DEVELOPING", -1, Some(7000), None, vec![])),
            (9_000_000_000, snap("IDLE", -1, None, None, vec![])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(engine.mismatches_total(), 0);
        assert_eq!(c.episodes_completed_in_leaf, 1);
        assert_eq!(c.tracks_completed, 1);
        assert_eq!(c.left_truncated_tracks, 0);
        assert_eq!(c.forming_fizzled, 1);
        for i in 0..5 {
            assert_eq!(c.matched[i], 1, "anchor {i}");
            assert_eq!(c.value_mismatch[i], 0);
            assert_eq!(c.missing_live[i], 0);
            assert_eq!(c.spurious_live[i], 0);
        }
        assert_eq!(c.end_ts_vs_appearance_mismatch, 0);
        // D-005: episode 7 armed sighting (weak=1000 at the arming tick).
        assert_eq!(c.matched[A_WEAK], 1);
        assert_eq!(c.matched[A_ONLINE], 1);
        assert_eq!(c.armed_detections[0], 1);
        assert_eq!(c.armed_detections[1], 1);
        // Episode 7 weak plus the successor forming episode's weak: only the
        // armed episode's sighting is a detection now.
        assert_eq!(c.anchors_detected[A_WEAK], 1);
        assert_eq!(c.anchors_detected[A_STRONG], 1);
    }

    #[test]
    fn left_truncated_baseline_absorbs_and_excludes() {
        let mut engine = Engine::new();
        // Leaf starts mid-episode at tick 200ms; weak/online/dying already
        // present. Ground truth for every anchor present at the baseline
        // predates the leaf.
        let e3 = emission(3, 100, 150, 120, 160, 150, 300);
        let rows = [
            (200_000_000, snap("DYING", 3, Some(100), Some(150), vec![])),
            (250_000_000, snap("BURST", 3, Some(100), Some(150), vec![])),
            (300_000_000, snap("DEVELOPING", -1, Some(300), None, vec![e3.clone()])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(engine.mismatches_total(), 0);
        assert_eq!(c.left_truncated_tracks, 1);
        assert_eq!(c.excluded_left_truncated[A_WEAK], 1);
        assert_eq!(c.excluded_left_truncated[A_ONLINE], 1);
        assert_eq!(c.excluded_left_truncated[A_STRONG], 1);
        assert_eq!(c.excluded_left_truncated[A_DECAY], 1);
        assert_eq!(c.excluded_left_truncated[A_DYING], 1);
        // The episode was already armed at leaf start; its in-leaf armed
        // sightings are captured but its ground truth predates the leaf.
        assert_eq!(c.armed_detections[0], 1);
        assert_eq!(c.anchors_detected[A_STRONG], 0);
        assert_eq!(c.episodes_completed_in_leaf, 1);
        assert_eq!(c.right_censored_forming, 1);
    }

    #[test]
    fn supersede_starts_new_episode_at_burst() {
        let mut engine = Engine::new();
        let e9 = emission(9, 100, 200, 300, 400, 500, 600);
        let rows = [
            (50_000_000, snap("IDLE", -1, None, None, vec![])),
            (100_000_000, snap("DEVELOPING", -1, Some(100), None, vec![])),
            (200_000_000, snap("DEVELOPING", -1, Some(100), Some(200), vec![])),
            (300_000_000, snap("BURST", 9, Some(100), Some(200), vec![])),
            (400_000_000, snap("DECAY_RISK", 9, Some(100), Some(200), vec![])),
            (500_000_000, snap("DYING", 9, Some(100), Some(200), vec![])),
            (600_000_000, snap("BURST", 10, Some(600), Some(600), vec![e9.clone()])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(engine.mismatches_total(), 0);
        assert_eq!(c.episodes_completed_in_leaf, 1);
        // Episode 9 closed at the supersede tick; episode 10 is active at the
        // end of the leaf and settles as right-censored (armed).
        assert_eq!(c.tracks_completed, 1);
        assert_eq!(c.right_censored_armed, 1);
        assert_eq!(c.matched[A_WEAK], 1);
        assert_eq!(c.matched[A_STRONG], 1);
        assert_eq!(c.anchors_detected[A_STRONG], 2);
        // Both tracks had observable armed phases (episode 9 armed mid-leaf,
        // episode 10 armed at the supersede tick).
        assert_eq!(c.armed_detections[0], 2);
    }

    #[test]
    fn carryover_sighting_is_skipped() {
        let mut engine = Engine::new();
        let e2 = json!({
            "event_id": 2,
            "origin_ts": 400,
            "weak_ts": 500,
            "online_ts": null,
            "strong_ts": 700,
            "decay_risk_ts": null,
            "first_dying_ts": null,
            "end_ts": 800,
        });
        let rows = [
            (100_000_000, snap("IDLE", -1, None, None, vec![])),
            // Forming-window provisional crossing: not an anchor under D-005.
            (300_000_000, snap("DEVELOPING", -1, Some(300), None, vec![])),
            // Arming tick carries the pre-origin value: carryover.
            (700_000_000, snap("BURST", 2, Some(300), None, vec![])),
            // First in-window crossing while armed.
            (750_000_000, snap("BURST", 2, Some(500), None, vec![])),
            (800_000_000, snap("DEVELOPING", -1, Some(800), None, vec![e2.clone()])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(engine.mismatches_total(), 0);
        assert_eq!(c.matched[A_WEAK], 1);
        assert_eq!(c.value_mismatch[A_WEAK], 0);
        assert_eq!(c.carryover_seen[0], 1);
        assert_eq!(c.carryover_replaced[0], 1);
        // Run-1 diagnostic: the first sighting (300) diverged from gt (500).
        assert_eq!(c.diag_first_sight_value_mismatch[0], 1);
        assert_eq!(c.matched[A_STRONG], 1);
    }

    #[test]
    fn standing_pre_origin_value_is_kept_when_no_replacement_appears() {
        let mut engine = Engine::new();
        let e3 = json!({
            "event_id": 3,
            "origin_ts": 400,
            "weak_ts": 300,
            "online_ts": null,
            "strong_ts": 700,
            "decay_risk_ts": null,
            "first_dying_ts": null,
            "end_ts": 800,
        });
        let rows = [
            (100_000_000, snap("IDLE", -1, None, None, vec![])),
            (700_000_000, snap("BURST", 3, Some(300), None, vec![])),
            (750_000_000, snap("BURST", 3, Some(300), None, vec![])),
            (800_000_000, snap("DEVELOPING", -1, None, None, vec![e3.clone()])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(engine.mismatches_total(), 0);
        // No later in-window crossing appeared: the standing pre-origin value
        // is the anchor and it matches the producer's emitted weak_ts.
        assert_eq!(c.matched[A_WEAK], 1);
        assert_eq!(c.value_mismatch[A_WEAK], 0);
        assert_eq!(c.carryover_seen[0], 1);
        assert_eq!(c.carryover_replaced[0], 0);
        assert_eq!(c.diag_first_sight_value_mismatch[0], 0);
    }

    #[test]
    fn forming_episode_that_fizzles_emits_nothing() {
        let mut engine = Engine::new();
        let rows = [
            (1_000_000, snap("DEVELOPING", -1, Some(500), None, vec![])),
            (2_000_000, snap("IDLE", -1, None, None, vec![])),
        ];
        for (ns, s) in rows {
            engine.observe(ns, Some(&s));
        }
        engine.finish();
        let c = engine.counts();
        assert_eq!(c.forming_fizzled, 1);
        assert_eq!(c.episodes_completed_in_leaf, 0);
        assert_eq!(engine.mismatches_total(), 0);
    }

    #[test]
    fn emitted_entry_change_is_a_mutation() {
        let mut engine = Engine::new();
        let first = emission(4, 100, 200, 300, 400, 500, 600);
        let second = emission(4, 100, 200, 300, 400, 500, 601);
        engine.observe(1_000_000, Some(&snap("DEVELOPING", -1, None, None, vec![first])));
        engine.observe(2_000_000, Some(&snap("DEVELOPING", -1, None, None, vec![second])));
        engine.finish();
        assert_eq!(engine.counts().emitted_mutations, 1);
    }

    #[test]
    fn payload_parser_handles_probe_shape() {
        let payload = r#"{"drift_burst_state":{"state":"BURST","event_id":0,"authority":{"weak_online_scale_s":30.0},"milestones":{"weak":{"index":232,"ts_ms":1752019281109,"mid":3304.795,"direction":-1},"online":null},"operational":{"idx":1},"multiscale":{},"emitted_events":[]}}"#;
        let mut fallbacks = 0;
        let mut malformed = 0;
        let parsed = parse_payload(payload, &mut fallbacks, &mut malformed);
        let snapshot = unpack(parsed);
        assert_eq!(fallbacks, 0);
        assert_eq!(malformed, 0);
        assert_eq!(snapshot.state, DState::Burst);
        assert_eq!(snapshot.eid_pos, Some(0));
        assert_eq!(snapshot.weak_ms, Some(1752019281109));
        assert_eq!(snapshot.online_ms, None);
    }

    #[test]
    fn payload_parser_falls_back_on_unexpected_shape() {
        let payload = r#"{"drift_burst_state":{"state":"BURST","event_id":"zero","milestones":{"weak":{"ts_ms":7}},"emitted_events":[]}}"#;
        let mut fallbacks = 0;
        let mut malformed = 0;
        let parsed = parse_payload(payload, &mut fallbacks, &mut malformed);
        let snapshot = unpack(parsed);
        assert_eq!(fallbacks, 1);
        assert_eq!(snapshot.eid_pos, None);
        assert_eq!(snapshot.weak_ms, Some(7));
    }
}
