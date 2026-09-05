use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnchorTick {
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub received_time_ns: i64,
    pub event_time_ns: i64,
    pub source_sequence: i64,
    pub source_part: String,
    pub row_index: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Availability {
    pub available_time_ns: i64,
    pub source_sequence: i64,
    pub trigger_event_time_ns: i64,
    pub trigger_source_part: String,
    pub trigger_row_index: u64,
    pub anchor: Option<AnchorTick>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormationFields {
    pub detection_ts: i64,
    pub formation_atr: f64,
    pub formation_basis: String,
    pub fvg_quality: f64,
    pub gap: f64,
    pub gap_atr: f64,
    pub gap_bps: f64,
    pub impulse_body_atr: f64,
    pub impulse_body_fraction: f64,
    pub impulse_close_location: f64,
    pub impulse_ts: i64,
    pub lower: f64,
    pub origin_ts: i64,
    pub upper: f64,
    pub zone_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TouchFields {
    pub detection_ts: i64,
    pub direction: Direction,
    pub fvg_quality: f64,
    pub impulse_ts: i64,
    pub lower: f64,
    pub origin_ts: i64,
    pub touch_basis: String,
    pub touch_ts: i64,
    pub upper: f64,
    pub zone_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FillFields {
    pub detection_ts: i64,
    pub direction: Direction,
    pub fill_basis: String,
    pub fill_ts: i64,
    pub first_touch_observed: bool,
    pub fvg_quality: f64,
    pub impulse_ts: i64,
    pub lower: f64,
    pub origin_ts: i64,
    pub upper: f64,
    pub zone_id: u64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FvgPayload {
    #[serde(default, deserialize_with = "deserialize_events")]
    pub bullish_fvg: Vec<FormationFields>,
    #[serde(default, deserialize_with = "deserialize_events")]
    pub bearish_fvg: Vec<FormationFields>,
    #[serde(default, deserialize_with = "deserialize_events")]
    pub fvg_first_touch: Vec<TouchFields>,
    #[serde(default, deserialize_with = "deserialize_events")]
    pub fvg_filled: Vec<FillFields>,
}

fn deserialize_events<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        One(T),
        Many(Vec<T>),
    }
    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(value) => vec![value],
        OneOrMany::Many(values) => values,
    })
}

pub fn parse_payload(value: &str) -> Result<FvgPayload, serde_json::Error> {
    serde_json::from_str(value)
}

impl FvgPayload {
    pub fn formations(&self) -> Vec<(Direction, FormationFields)> {
        self.bullish_fvg
            .iter()
            .cloned()
            .map(|event| (Direction::Bullish, event))
            .chain(
                self.bearish_fvg
                    .iter()
                    .cloned()
                    .map(|event| (Direction::Bearish, event)),
            )
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Formation {
    pub direction: Direction,
    pub detection_ts: i64,
    pub formation_atr: f64,
    pub formation_basis: String,
    pub fvg_quality: f64,
    pub gap: f64,
    pub gap_atr: f64,
    pub gap_bps: f64,
    pub impulse_body_atr: f64,
    pub impulse_body_fraction: f64,
    pub impulse_close_location: f64,
    pub impulse_ts: i64,
    pub lower: f64,
    pub origin_ts: i64,
    pub upper: f64,
    pub zone_id: u64,
}

impl Formation {
    fn from_fields(direction: Direction, fields: FormationFields) -> Self {
        Self {
            direction,
            detection_ts: fields.detection_ts,
            formation_atr: fields.formation_atr,
            formation_basis: fields.formation_basis,
            fvg_quality: fields.fvg_quality,
            gap: fields.gap,
            gap_atr: fields.gap_atr,
            gap_bps: fields.gap_bps,
            impulse_body_atr: fields.impulse_body_atr,
            impulse_body_fraction: fields.impulse_body_fraction,
            impulse_close_location: fields.impulse_close_location,
            impulse_ts: fields.impulse_ts,
            lower: fields.lower,
            origin_ts: fields.origin_ts,
            upper: fields.upper,
            zone_id: fields.zone_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProspectiveOutcome {
    pub raw_price_delta: f64,
    pub raw_return_bps: f64,
    pub direction_adjusted_return_bps: f64,
    pub anchor_mid: f64,
    pub outcome_close: f64,
    pub outcome_bar_close_ts: i64,
    pub outcome_available_time_ns: i64,
    pub outcome_source_sequence: i64,
    pub normalization_basis: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZoneLifecycle {
    pub timeframe: String,
    pub identity: String,
    pub formation: Formation,
    pub formation_bar_close_ts: i64,
    pub formation_available_time_ns: i64,
    pub formation_source_sequence: i64,
    pub anchor_bid: f64,
    pub anchor_ask: f64,
    pub anchor_mid: f64,
    pub anchor_received_ts_ns: i64,
    pub anchor_source_sequence: i64,
    pub first_touch_ts: Option<i64>,
    pub first_touch_available_time_ns: Option<i64>,
    pub first_touch_source_sequence: Option<i64>,
    pub fill_ts: Option<i64>,
    pub fill_available_time_ns: Option<i64>,
    pub fill_source_sequence: Option<i64>,
    pub formation_to_touch_market_ms: Option<i64>,
    pub formation_to_touch_known_ms: Option<i64>,
    pub formation_to_fill_market_ms: Option<i64>,
    pub formation_to_fill_known_ms: Option<i64>,
    pub active_duration_ms: i64,
    pub active: bool,
    pub right_censored: bool,
    pub prospective_outcomes: [Option<ProspectiveOutcome>; 3],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnavailableCounters {
    pub formation_events: u64,
    pub touch_events: u64,
    pub fill_events: u64,
    pub outcome_events: u64,
}

#[derive(Debug, Default)]
pub struct ZoneBook {
    timeframe: String,
    zones: BTreeMap<u64, ZoneLifecycle>,
    response_schedule: BTreeMap<u64, Vec<(u64, usize)>>,
    lawful_row_index: u64,
    last_bar_close_ts: Option<i64>,
    last_lawful_bar_close_ts: Option<i64>,
    pub orphan_touch_count: u64,
    pub orphan_fill_count: u64,
    pub unavailable: UnavailableCounters,
}

#[derive(Debug)]
pub enum FvgError {
    Payload(serde_json::Error),
    Io(io::Error),
    Invalid(String),
}
impl std::fmt::Display for FvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Payload(error) => write!(f, "malformed FVG payload: {error}"),
            Self::Io(error) => write!(f, "sidecar I/O error: {error}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for FvgError {}
impl From<serde_json::Error> for FvgError {
    fn from(error: serde_json::Error) -> Self {
        Self::Payload(error)
    }
}
impl From<io::Error> for FvgError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SidecarLine {
    timeframe: String,
    bar_close_ts_ns: i64,
    available_time_ns: i64,
    trigger_event_time_ns: i64,
    trigger_source_sequence: i64,
    trigger_source_part: String,
    trigger_row_index: u64,
}

pub struct SidecarCursor<R> {
    reader: R,
    timeframe: String,
    pending: Option<SidecarLine>,
    last_bar_close_ts_ns: Option<i64>,
}
impl<R: BufRead> SidecarCursor<R> {
    pub fn new(reader: R, timeframe: impl Into<String>) -> Self {
        Self {
            reader,
            timeframe: timeframe.into(),
            pending: None,
            last_bar_close_ts_ns: None,
        }
    }
    fn read_next(&mut self) -> Result<Option<SidecarLine>, FvgError> {
        let mut line = String::new();
        loop {
            line.clear();
            if self.reader.read_line(&mut line)? == 0 {
                return Ok(None);
            }
            if line.trim().is_empty() {
                continue;
            }
            let record: SidecarLine = serde_json::from_str(&line)?;
            if record.timeframe == self.timeframe {
                if self
                    .last_bar_close_ts_ns
                    .is_some_and(|previous| record.bar_close_ts_ns <= previous)
                {
                    return Err(FvgError::Invalid(format!(
                        "sidecar {} records are not increasing",
                        self.timeframe
                    )));
                }
                self.last_bar_close_ts_ns = Some(record.bar_close_ts_ns);
                return Ok(Some(record));
            }
        }
    }
    pub fn lookup(&mut self, bar_close_ts_ns: i64) -> Result<Option<Availability>, FvgError> {
        loop {
            if self.pending.is_none() {
                self.pending = self.read_next()?;
            }
            let Some(record) = self.pending.as_ref() else {
                return Ok(None);
            };
            if record.bar_close_ts_ns < bar_close_ts_ns {
                self.pending = None;
                continue;
            }
            if record.bar_close_ts_ns > bar_close_ts_ns {
                return Ok(None);
            }
            let record = self.pending.take().expect("sidecar pending record exists");
            return Ok(Some(Availability {
                available_time_ns: record.available_time_ns,
                source_sequence: record.trigger_source_sequence,
                trigger_event_time_ns: record.trigger_event_time_ns,
                trigger_source_part: record.trigger_source_part,
                trigger_row_index: record.trigger_row_index,
                anchor: None,
            }));
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnchorIndex {
    ticks: BTreeMap<i64, AnchorTick>,
}
impl AnchorIndex {
    pub fn insert(&mut self, tick: AnchorTick) -> Result<(), FvgError> {
        if self.ticks.insert(tick.source_sequence, tick).is_some() {
            return Err(FvgError::Invalid("duplicate source sequence anchor".into()));
        }
        Ok(())
    }
    pub fn lookup(&self, availability: &Availability) -> Result<Availability, FvgError> {
        let Some(anchor) = self.ticks.get(&availability.source_sequence).cloned() else {
            return Err(FvgError::Invalid(format!(
                "missing trigger tick for source sequence {}",
                availability.source_sequence
            )));
        };
        if anchor.received_time_ns != availability.available_time_ns
            || anchor.event_time_ns != availability.trigger_event_time_ns
            || anchor.source_part != availability.trigger_source_part
            || anchor.row_index != availability.trigger_row_index
        {
            return Err(FvgError::Invalid(format!(
                "trigger source identity conflicts for source sequence {}",
                availability.source_sequence
            )));
        }
        Ok(Availability {
            anchor: Some(anchor),
            ..availability.clone()
        })
    }
    pub fn entries(&self) -> Vec<(i64, AnchorTick)> {
        self.ticks
            .iter()
            .map(|(sequence, tick)| (*sequence, tick.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReceiptRegression {
    pub previous_part: String,
    pub previous_row_index: u64,
    pub previous_received_ts_ns: i64,
    pub part: String,
    pub row_index: u64,
    pub received_ts_ns: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreflightResult {
    pub rows_scanned: u64,
    pub regressions_count: u64,
    pub first_regression: Option<ReceiptRegression>,
    pub last_regression: Option<ReceiptRegression>,
    pub source_view_manifest_sha256: String,
    pub scanner_commit: String,
    pub deterministic_preflight_hash: String,
}
#[derive(Debug)]
pub struct Preflight {
    rows_scanned: u64,
    regressions_count: u64,
    first_regression: Option<ReceiptRegression>,
    last_regression: Option<ReceiptRegression>,
    source_view_manifest_sha256: String,
    scanner_commit: String,
    previous: Option<(String, u64, i64)>,
    anchors: AnchorIndex,
    capture_sequences: Option<BTreeSet<i64>>,
}
impl Preflight {
    pub fn new(
        source_view_manifest_sha256: impl Into<String>,
        scanner_commit: impl Into<String>,
    ) -> Self {
        Self {
            rows_scanned: 0,
            regressions_count: 0,
            first_regression: None,
            last_regression: None,
            source_view_manifest_sha256: source_view_manifest_sha256.into(),
            scanner_commit: scanner_commit.into(),
            previous: None,
            anchors: AnchorIndex::default(),
            capture_sequences: None,
        }
    }
    pub fn with_capture_sequences(mut self, sequences: BTreeSet<i64>) -> Self {
        self.capture_sequences = Some(sequences);
        self
    }
    pub fn observe(
        &mut self,
        part: &str,
        row_index: u64,
        received_ts_ns: i64,
        _source_sequence: i64,
    ) {
        self.rows_scanned += 1;
        if let Some((previous_part, previous_row_index, previous_received_ts_ns)) = &self.previous {
            if received_ts_ns < *previous_received_ts_ns {
                let regression = ReceiptRegression {
                    previous_part: previous_part.clone(),
                    previous_row_index: *previous_row_index,
                    previous_received_ts_ns: *previous_received_ts_ns,
                    part: part.into(),
                    row_index,
                    received_ts_ns,
                };
                self.regressions_count += 1;
                if self.first_regression.is_none() {
                    self.first_regression = Some(regression.clone());
                }
                self.last_regression = Some(regression);
            }
        }
        self.previous = Some((part.into(), row_index, received_ts_ns));
    }
    pub fn observe_tick(
        &mut self,
        part: &str,
        row_index: u64,
        received_ts_ns: i64,
        event_time_ns: i64,
        source_sequence: i64,
        bid: f64,
        ask: f64,
    ) -> Result<(), FvgError> {
        if !bid.is_finite() || !ask.is_finite() || bid <= 0.0 || ask <= 0.0 || bid > ask {
            return Err(FvgError::Invalid(format!(
                "invalid tick quote at {part} row {row_index}"
            )));
        }
        self.observe(part, row_index, received_ts_ns, source_sequence);
        if self
            .capture_sequences
            .as_ref()
            .is_none_or(|sequences| sequences.contains(&source_sequence))
        {
            self.anchors.insert(AnchorTick {
                bid,
                ask,
                mid: (bid + ask) / 2.0,
                received_time_ns: received_ts_ns,
                event_time_ns,
                source_sequence,
                source_part: part.into(),
                row_index,
            })?;
        }
        Ok(())
    }
    pub fn finish(&self) -> PreflightResult {
        let mut result = PreflightResult {
            rows_scanned: self.rows_scanned,
            regressions_count: self.regressions_count,
            first_regression: self.first_regression.clone(),
            last_regression: self.last_regression.clone(),
            source_view_manifest_sha256: self.source_view_manifest_sha256.clone(),
            scanner_commit: self.scanner_commit.clone(),
            deterministic_preflight_hash: String::new(),
        };
        let bytes = serde_json::to_vec(&result).expect("preflight result serializes");
        result.deterministic_preflight_hash = format!("{:x}", Sha256::digest(bytes));
        result
    }
    pub fn anchors(&self) -> &AnchorIndex {
        &self.anchors
    }
}

impl ZoneBook {
    pub fn new(timeframe: impl Into<String>) -> Self {
        Self {
            timeframe: timeframe.into(),
            ..Self::default()
        }
    }
    pub fn ingest_bar(
        &mut self,
        bar_close_ts: i64,
        availability: Availability,
        payload: &str,
    ) -> Result<(), FvgError> {
        self.ingest_bar_with_close_optional(bar_close_ts, Some(availability), payload, None)
    }
    pub fn ingest_bar_with_close(
        &mut self,
        bar_close_ts: i64,
        availability: Availability,
        payload: &str,
        close: Option<f64>,
    ) -> Result<(), FvgError> {
        self.ingest_bar_with_close_optional(bar_close_ts, Some(availability), payload, close)
    }
    pub fn ingest_bar_with_close_optional(
        &mut self,
        bar_close_ts: i64,
        availability: Option<Availability>,
        payload: &str,
        close: Option<f64>,
    ) -> Result<(), FvgError> {
        if self
            .last_bar_close_ts
            .is_some_and(|previous| bar_close_ts <= previous)
        {
            return Err(FvgError::Invalid(
                "native bar close is not increasing".into(),
            ));
        }
        let payload = parse_payload(payload)?;
        let Some(availability) = availability else {
            self.unavailable.formation_events += payload.formations().len() as u64;
            self.unavailable.touch_events += payload.fvg_first_touch.len() as u64;
            self.unavailable.fill_events += payload.fvg_filled.len() as u64;
            self.unavailable.outcome_events += self
                .response_schedule
                .get(&self.lawful_row_index)
                .map_or(0, |events| events.len() as u64);
            self.last_bar_close_ts = Some(bar_close_ts);
            return Ok(());
        };
        let anchor = availability
            .anchor
            .ok_or_else(|| FvgError::Invalid("formation anchor tick is missing".into()))?;
        if !anchor.mid.is_finite() || anchor.mid <= 0.0 {
            return Err(FvgError::Invalid("formation anchor mid is invalid".into()));
        }
        if let Some(close) = close {
            if !close.is_finite() || close == 0.0 {
                return Err(FvgError::Invalid(
                    "bar close is not finite and nonzero".into(),
                ));
            }
        }
        for (zone_id, horizon) in self
            .response_schedule
            .remove(&self.lawful_row_index)
            .unwrap_or_default()
        {
            let zone = self
                .zones
                .get_mut(&zone_id)
                .ok_or_else(|| FvgError::Invalid("response schedule lost its zone".into()))?;
            let close = close.ok_or_else(|| {
                FvgError::Invalid("lawfully available outcome bar has no close".into())
            })?;
            if !after_receipt(
                zone.formation_available_time_ns,
                zone.formation_source_sequence,
                availability.available_time_ns,
                availability.source_sequence,
            ) {
                return Err(FvgError::Invalid(format!(
                    "outcome is not causally after formation for {}",
                    zone.identity
                )));
            }
            let raw_price_delta = close - zone.anchor_mid;
            let raw_return_bps = 10_000.0 * raw_price_delta / zone.anchor_mid;
            let direction_adjusted_return_bps = match zone.formation.direction {
                Direction::Bullish => raw_return_bps,
                Direction::Bearish => -raw_return_bps,
            };
            zone.prospective_outcomes[horizon] = Some(ProspectiveOutcome {
                raw_price_delta,
                raw_return_bps,
                direction_adjusted_return_bps,
                anchor_mid: zone.anchor_mid,
                outcome_close: close,
                outcome_bar_close_ts: bar_close_ts,
                outcome_available_time_ns: availability.available_time_ns,
                outcome_source_sequence: availability.source_sequence,
                normalization_basis: "anchor_mid_at_boundary_crossing_tick".into(),
            });
        }
        for (direction, fields) in payload.formations() {
            if fields.detection_ts != bar_close_ts {
                return Err(FvgError::Invalid(format!(
                    "formation detection timestamp {} does not match bar close {}",
                    fields.detection_ts, bar_close_ts
                )));
            }
            if self.zones.contains_key(&fields.zone_id) {
                return Err(FvgError::Invalid(format!(
                    "duplicate native zone id: {}",
                    fields.zone_id
                )));
            }
            let zone_id = fields.zone_id;
            self.zones.insert(
                zone_id,
                ZoneLifecycle {
                    timeframe: self.timeframe.clone(),
                    identity: logical_zone_key(&self.timeframe, zone_id),
                    formation: Formation::from_fields(direction, fields),
                    formation_bar_close_ts: bar_close_ts,
                    formation_available_time_ns: availability.available_time_ns,
                    formation_source_sequence: availability.source_sequence,
                    anchor_bid: anchor.bid,
                    anchor_ask: anchor.ask,
                    anchor_mid: anchor.mid,
                    anchor_received_ts_ns: anchor.received_time_ns,
                    anchor_source_sequence: anchor.source_sequence,
                    first_touch_ts: None,
                    first_touch_available_time_ns: None,
                    first_touch_source_sequence: None,
                    fill_ts: None,
                    fill_available_time_ns: None,
                    fill_source_sequence: None,
                    formation_to_touch_market_ms: None,
                    formation_to_touch_known_ms: None,
                    formation_to_fill_market_ms: None,
                    formation_to_fill_known_ms: None,
                    active_duration_ms: 0,
                    active: true,
                    right_censored: false,
                    prospective_outcomes: std::array::from_fn(|_| None),
                },
            );
            if close.is_some() {
                for (horizon, offset) in [1_u64, 3, 5].into_iter().enumerate() {
                    self.response_schedule
                        .entry(self.lawful_row_index.saturating_add(offset))
                        .or_default()
                        .push((zone_id, horizon));
                }
            }
        }
        for event in payload.fvg_first_touch {
            let Some(zone) = self.zones.get_mut(&event.zone_id) else {
                self.orphan_touch_count += 1;
                continue;
            };
            validate_touch(zone, &event, bar_close_ts)?;
            if zone.fill_ts.is_some() {
                return Err(FvgError::Invalid(format!(
                    "touch follows terminal fill for {}",
                    zone.identity
                )));
            }
            if zone.first_touch_ts.is_some() {
                return Err(FvgError::Invalid(format!(
                    "duplicate first-touch event for {}",
                    zone.identity
                )));
            }
            zone.first_touch_ts = Some(event.touch_ts);
            zone.first_touch_available_time_ns = Some(availability.available_time_ns);
            zone.first_touch_source_sequence = Some(availability.source_sequence);
        }
        for event in payload.fvg_filled {
            let Some(zone) = self.zones.get_mut(&event.zone_id) else {
                self.orphan_fill_count += 1;
                continue;
            };
            validate_fill(zone, &event, bar_close_ts)?;
            if zone.fill_ts.is_some() {
                return Err(FvgError::Invalid(format!(
                    "duplicate terminal fill for {}",
                    zone.identity
                )));
            }
            if zone
                .first_touch_ts
                .is_some_and(|touch| touch > event.fill_ts)
            {
                return Err(FvgError::Invalid(format!(
                    "fill precedes touch for {}",
                    zone.identity
                )));
            }
            zone.fill_ts = Some(event.fill_ts);
            zone.fill_available_time_ns = Some(availability.available_time_ns);
            zone.fill_source_sequence = Some(availability.source_sequence);
            zone.active = false;
        }
        self.last_lawful_bar_close_ts = Some(bar_close_ts);
        self.lawful_row_index = self.lawful_row_index.saturating_add(1);
        self.last_bar_close_ts = Some(bar_close_ts);
        Ok(())
    }
    pub fn finish(mut self, _last_observed_bar_close_ts: i64) -> Vec<ZoneLifecycle> {
        let end = self.last_lawful_bar_close_ts;
        self.zones
            .values_mut()
            .map(|zone| {
                let terminal_ts = zone.fill_ts.or(end).unwrap_or(zone.formation.detection_ts);
                zone.active_duration_ms = terminal_ts.saturating_sub(zone.formation.detection_ts);
                zone.formation_to_touch_market_ms = zone
                    .first_touch_ts
                    .map(|touch| touch.saturating_sub(zone.formation.detection_ts));
                zone.formation_to_fill_market_ms = zone
                    .fill_ts
                    .map(|fill| fill.saturating_sub(zone.formation.detection_ts));
                zone.formation_to_touch_known_ms =
                    zone.first_touch_available_time_ns.map(|known| {
                        known.saturating_sub(zone.formation_available_time_ns) / 1_000_000
                    });
                zone.formation_to_fill_known_ms = zone.fill_available_time_ns.map(|known| {
                    known.saturating_sub(zone.formation_available_time_ns) / 1_000_000
                });
                zone.right_censored = zone.fill_ts.is_none();
                zone.active = zone.fill_ts.is_none();
                zone.clone()
            })
            .collect()
    }
}

fn after_receipt(
    anchor_time: i64,
    anchor_sequence: i64,
    event_time: i64,
    event_sequence: i64,
) -> bool {
    (event_time, event_sequence) > (anchor_time, anchor_sequence)
}
fn validate_touch(
    zone: &ZoneLifecycle,
    event: &TouchFields,
    bar_close_ts: i64,
) -> Result<(), FvgError> {
    if event.touch_ts != bar_close_ts
        || event.touch_ts <= zone.formation.detection_ts
        || event.zone_id != zone.formation.zone_id
        || event.direction != zone.formation.direction
        || event.detection_ts != zone.formation.detection_ts
        || event.origin_ts != zone.formation.origin_ts
        || event.impulse_ts != zone.formation.impulse_ts
        || event.lower != zone.formation.lower
        || event.upper != zone.formation.upper
        || event.fvg_quality != zone.formation.fvg_quality
    {
        return Err(FvgError::Invalid(format!(
            "touch identity conflicts for {}",
            zone.identity
        )));
    }
    Ok(())
}
fn validate_fill(
    zone: &ZoneLifecycle,
    event: &FillFields,
    bar_close_ts: i64,
) -> Result<(), FvgError> {
    if event.fill_ts != bar_close_ts
        || event.fill_ts <= zone.formation.detection_ts
        || event.zone_id != zone.formation.zone_id
        || event.direction != zone.formation.direction
        || event.detection_ts != zone.formation.detection_ts
        || event.origin_ts != zone.formation.origin_ts
        || event.impulse_ts != zone.formation.impulse_ts
        || event.lower != zone.formation.lower
        || event.upper != zone.formation.upper
        || event.fvg_quality != zone.formation.fvg_quality
    {
        return Err(FvgError::Invalid(format!(
            "fill identity conflicts for {}",
            zone.identity
        )));
    }
    Ok(())
}
pub fn logical_zone_key(timeframe: &str, zone_id: u64) -> String {
    format!("{timeframe}:{zone_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    const FORMATION: &str = r#"{"bullish_fvg":{"detection_ts":1000,"formation_atr":2.0,"formation_basis":"three_bar_non_overlap","fvg_quality":0.8,"gap":1.0,"gap_atr":0.5,"gap_bps":1.0,"impulse_body_atr":0.7,"impulse_body_fraction":0.6,"impulse_close_location":0.9,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
    fn availability(time: i64, sequence: i64, bid: f64, ask: f64) -> Availability {
        Availability {
            available_time_ns: time,
            source_sequence: sequence,
            trigger_event_time_ns: time,
            trigger_source_part: "tick/0".into(),
            trigger_row_index: sequence as u64,
            anchor: Some(AnchorTick {
                bid,
                ask,
                mid: (bid + ask) / 2.0,
                received_time_ns: time,
                event_time_ns: time,
                source_sequence: sequence,
                source_part: "tick/0".into(),
                row_index: sequence as u64,
            }),
        }
    }
    fn book_with_zone() -> ZoneBook {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(
            1000,
            Some(availability(1100, 1, 10.0, 12.0)),
            FORMATION,
            Some(20.0),
        )
        .unwrap();
        book
    }
    #[test]
    fn decodes_single_and_array_payloads() {
        let single = parse_payload(FORMATION).unwrap();
        assert_eq!(single.formations().len(), 1);
        let array = FORMATION
            .replace("{\"bullish_fvg\":{", "{\"bullish_fvg\":[{")
            .replace("\"zone_id\":7}}", "\"zone_id\":7}]}");
        assert_eq!(parse_payload(&array).unwrap().formations().len(), 1);
    }
    #[test]
    fn malformed_payload_is_rejected() {
        assert!(parse_payload(r#"{"bullish_fvg":{"zone_id":"wrong"}}"#).is_err());
    }
    #[test]
    fn native_identity_keeps_same_zone_id_separate() {
        assert_ne!(logical_zone_key("15s", 7), logical_zone_key("30s", 7));
    }
    #[test]
    fn unavailable_tail_does_not_mutate_touch_fill_or_outcome() {
        let mut book = book_with_zone();
        let event = r#"{"fvg_first_touch":{"detection_ts":1000,"direction":"bullish","fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"touch_basis":"bar_range_overlap","touch_ts":2000,"upper":11.0,"zone_id":7},"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":2000,"first_touch_observed":true,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
        book.ingest_bar_with_close_optional(2000, None, event, Some(30.0))
            .unwrap();
        let zone = book.finish(2000).pop().unwrap();
        assert!(zone.first_touch_ts.is_none() && zone.fill_ts.is_none());
        assert!(zone.prospective_outcomes.iter().all(Option::is_none));
    }
    #[test]
    fn anchor_mid_excludes_formation_bar_close_and_outcomes_are_prospective() {
        let mut book = book_with_zone();
        book.ingest_bar_with_close_optional(
            2000,
            Some(availability(1200, 2, 11.0, 13.0)),
            "{}",
            Some(21.0),
        )
        .unwrap();
        let zone = book.finish(2000).pop().unwrap();
        assert_eq!(zone.anchor_mid, 11.0);
        assert_eq!(
            zone.prospective_outcomes[0].as_ref().unwrap().outcome_close,
            21.0
        );
        assert_eq!(
            zone.prospective_outcomes[0]
                .as_ref()
                .unwrap()
                .raw_price_delta,
            10.0
        );
    }
    #[test]
    fn lawful_right_censor_ignores_physical_unavailable_tail() {
        let mut book = book_with_zone();
        book.ingest_bar_with_close_optional(
            2000,
            Some(availability(1200, 2, 11.0, 13.0)),
            "{}",
            Some(21.0),
        )
        .unwrap();
        book.ingest_bar_with_close_optional(3000, None, "{}", Some(999.0))
            .unwrap();
        let zone = book.finish(3000).pop().unwrap();
        assert!(zone.right_censored);
        assert_eq!(zone.active_duration_ms, 1000);
    }
    #[test]
    fn touch_and_fill_preserve_event_and_receipt_times() {
        let mut book = book_with_zone();
        let payload = r#"{"fvg_first_touch":{"detection_ts":1000,"direction":"bullish","fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"touch_basis":"bar_range_overlap","touch_ts":2000,"upper":11.0,"zone_id":7},"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":2000,"first_touch_observed":true,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
        book.ingest_bar_with_close_optional(
            2000,
            Some(availability(2200, 2, 11.0, 13.0)),
            payload,
            Some(21.0),
        )
        .unwrap();
        let zone = book.finish(2000).pop().unwrap();
        assert_eq!(zone.first_touch_available_time_ns, Some(2200));
        assert_eq!(zone.fill_source_sequence, Some(2));
        assert_eq!(zone.formation_to_touch_market_ms, Some(1000));
        assert_eq!(zone.formation_to_touch_known_ms, Some(0));
    }
    #[test]
    fn equal_receipt_time_uses_source_sequence_ordering() {
        let mut book = book_with_zone();
        book.ingest_bar_with_close_optional(
            2000,
            Some(availability(1100, 2, 11.0, 13.0)),
            "{}",
            Some(21.0),
        )
        .unwrap();
        assert!(book.finish(2000).pop().unwrap().prospective_outcomes[0].is_some());
    }
    #[test]
    fn event_identity_mismatch_and_duplicate_fill_fail_closed() {
        let mut book = book_with_zone();
        let mismatch = r#"{"fvg_first_touch":{"detection_ts":1000,"direction":"bearish","fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"touch_basis":"bar_range_overlap","touch_ts":2000,"upper":11.0,"zone_id":7}}"#;
        assert!(book
            .ingest_bar_with_close_optional(
                2000,
                Some(availability(1200, 2, 11.0, 13.0)),
                mismatch,
                Some(21.0)
            )
            .is_err());
        let mut book = book_with_zone();
        let fill = r#"{"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":2000,"first_touch_observed":false,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
        book.ingest_bar_with_close_optional(
            2000,
            Some(availability(1200, 2, 11.0, 13.0)),
            fill,
            Some(21.0),
        )
        .unwrap();
        assert!(book
            .ingest_bar_with_close_optional(
                3000,
                Some(availability(1300, 3, 11.0, 13.0)),
                fill.replace("2000", "3000").as_str(),
                Some(22.0)
            )
            .is_err());
    }
    #[test]
    fn sidecar_lookup_preserves_trigger_identity() {
        let lines = concat!(
            r#"{"timeframe":"30s","bar_close_ts_ms":1000,"bar_close_ts_ns":1000000000,"available_time_ns":111,"trigger_event_time_ns":1001,"trigger_source_sequence":1,"trigger_source_part":"tick/0","trigger_row_index":0,"availability_kind":"BOUNDARY_CROSSING_TICK"}"#,
            "\n",
            r#"{"timeframe":"15s","bar_close_ts_ms":1000,"bar_close_ts_ns":1000000000,"available_time_ns":222,"trigger_event_time_ns":1001,"trigger_source_sequence":2,"trigger_source_part":"tick/0","trigger_row_index":1,"availability_kind":"BOUNDARY_CROSSING_TICK"}"#,
            "\n"
        );
        let mut cursor = SidecarCursor::new(Cursor::new(lines), "15s");
        let value = cursor.lookup(1_000_000_000).unwrap().unwrap();
        assert_eq!(value.available_time_ns, 222);
        assert_eq!(value.source_sequence, 2);
    }
    #[test]
    fn preflight_captures_anchor_tick_in_same_pass() {
        let mut preflight = Preflight::new("view-hash", "scanner-sha");
        preflight
            .observe_tick("tick/0", 0, 10, 9, 1, 10.0, 12.0)
            .unwrap();
        let availability = Availability {
            available_time_ns: 10,
            source_sequence: 1,
            trigger_event_time_ns: 9,
            trigger_source_part: "tick/0".into(),
            trigger_row_index: 0,
            anchor: None,
        };
        assert_eq!(
            preflight
                .anchors()
                .lookup(&availability)
                .unwrap()
                .anchor
                .unwrap()
                .mid,
            11.0
        );
    }
    #[test]
    fn preflight_records_receipt_regression_without_reordering() {
        let mut preflight = Preflight::new("view-hash", "scanner-sha");
        preflight.observe("tick/0", 0, 10, 1);
        preflight.observe("tick/0", 1, 9, 2);
        let result = preflight.finish();
        assert_eq!(result.rows_scanned, 2);
        assert_eq!(result.regressions_count, 1);
        assert_eq!(result.first_regression, result.last_regression);
        assert!(!result.deterministic_preflight_hash.is_empty());
    }
    #[test]
    fn deterministic_preflight_hash_replays_identically() {
        let build = || {
            let mut preflight = Preflight::new("view-hash", "scanner-sha");
            preflight.observe("tick/0", 0, 10, 1);
            preflight.observe("tick/0", 1, 11, 2);
            preflight.finish().deterministic_preflight_hash
        };
        assert_eq!(build(), build());
    }
}
