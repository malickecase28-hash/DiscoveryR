use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

#[derive(Debug, Clone, Copy, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Availability {
    pub available_time_ns: i64,
    pub source_sequence: i64,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ZoneLifecycle {
    pub timeframe: String,
    pub identity: String,
    pub formation: Formation,
    pub formation_bar_close_ts: i64,
    pub formation_available_time_ns: i64,
    pub formation_source_sequence: i64,
    pub first_touch_ts: Option<i64>,
    pub fill_ts: Option<i64>,
    pub formation_to_first_touch_ms: Option<i64>,
    pub formation_to_fill_ms: Option<i64>,
    pub active_duration_ms: i64,
    pub active: bool,
    pub right_censored: bool,
    pub prospective_response_bps: [Option<f64>; 3],
    #[serde(skip)]
    formation_reference_close: f64,
}

#[derive(Debug, Default)]
pub struct ZoneBook {
    timeframe: String,
    zones: BTreeMap<u64, ZoneLifecycle>,
    response_schedule: BTreeMap<u64, Vec<(u64, usize)>>,
    row_index: u64,
    last_bar_close_ts: Option<i64>,
    pub orphan_touch_count: u64,
    pub orphan_fill_count: u64,
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
    trigger_source_sequence: i64,
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
            }));
        }
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
        }
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

    pub fn finish(self) -> PreflightResult {
        let mut result = PreflightResult {
            rows_scanned: self.rows_scanned,
            regressions_count: self.regressions_count,
            first_regression: self.first_regression,
            last_regression: self.last_regression,
            source_view_manifest_sha256: self.source_view_manifest_sha256,
            scanner_commit: self.scanner_commit,
            deterministic_preflight_hash: String::new(),
        };
        let bytes = serde_json::to_vec(&result).expect("preflight result serializes");
        result.deterministic_preflight_hash = format!("{:x}", Sha256::digest(bytes));
        result
    }
}

impl ZoneBook {
    pub fn new(timeframe: impl Into<String>) -> Self {
        Self {
            timeframe: timeframe.into(),
            response_schedule: BTreeMap::new(),
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
        if let Some(close) = close {
            if !close.is_finite() || close == 0.0 {
                return Err(FvgError::Invalid(
                    "bar close is not finite and nonzero".into(),
                ));
            }
            for (zone_id, horizon) in self
                .response_schedule
                .remove(&self.row_index)
                .unwrap_or_default()
            {
                if let Some(zone) = self.zones.get_mut(&zone_id) {
                    zone.prospective_response_bps[horizon] =
                        Some(10_000.0 * (close / zone.formation_reference_close - 1.0));
                }
            }
        }
        let payload = parse_payload(payload)?;
        for (direction, fields) in availability
            .map(|_| payload.formations())
            .unwrap_or_default()
        {
            if fields.detection_ts != bar_close_ts {
                return Err(FvgError::Invalid(format!(
                    "formation detection timestamp {} does not match bar close {}",
                    fields.detection_ts, bar_close_ts
                )));
            }
            let zone_id = fields.zone_id;
            if self.zones.contains_key(&zone_id) {
                return Err(FvgError::Invalid(format!(
                    "duplicate native zone id: {zone_id}"
                )));
            }
            let formation = Formation::from_fields(direction, fields);
            self.zones.insert(
                zone_id,
                ZoneLifecycle {
                    timeframe: self.timeframe.clone(),
                    identity: logical_zone_key(&self.timeframe, zone_id),
                    formation,
                    formation_bar_close_ts: bar_close_ts,
                    formation_available_time_ns: availability
                        .expect("formation availability exists")
                        .available_time_ns,
                    formation_source_sequence: availability
                        .expect("formation availability exists")
                        .source_sequence,
                    first_touch_ts: None,
                    fill_ts: None,
                    formation_to_first_touch_ms: None,
                    formation_to_fill_ms: None,
                    active_duration_ms: 0,
                    active: true,
                    right_censored: false,
                    prospective_response_bps: [None; 3],
                    formation_reference_close: close.unwrap_or(0.0),
                },
            );
            if let Some(close) = close {
                for (horizon, offset) in [1_u64, 3, 5].into_iter().enumerate() {
                    self.response_schedule
                        .entry(self.row_index.saturating_add(offset))
                        .or_default()
                        .push((zone_id, horizon));
                }
                self.zones
                    .get_mut(&zone_id)
                    .expect("new zone exists")
                    .formation_reference_close = close;
            }
        }
        for event in payload.fvg_first_touch {
            if let Some(zone) = self.zones.get_mut(&event.zone_id) {
                if zone.first_touch_ts.is_none() {
                    zone.first_touch_ts = Some(event.touch_ts);
                }
            } else {
                self.orphan_touch_count += 1;
            }
        }
        for event in payload.fvg_filled {
            if let Some(zone) = self.zones.get_mut(&event.zone_id) {
                if zone.fill_ts.is_none() {
                    zone.fill_ts = Some(event.fill_ts);
                    zone.active = false;
                }
            } else {
                self.orphan_fill_count += 1;
            }
        }
        self.last_bar_close_ts = Some(bar_close_ts);
        self.row_index = self.row_index.saturating_add(1);
        Ok(())
    }

    pub fn finish(mut self, last_observed_bar_close_ts: i64) -> Vec<ZoneLifecycle> {
        self.zones
            .values_mut()
            .map(|zone| {
                let end = zone.fill_ts.unwrap_or(last_observed_bar_close_ts);
                zone.active_duration_ms = end.saturating_sub(zone.formation.detection_ts);
                zone.formation_to_first_touch_ms = zone
                    .first_touch_ts
                    .map(|touch| touch.saturating_sub(zone.formation.detection_ts));
                zone.formation_to_fill_ms = zone
                    .fill_ts
                    .map(|fill| fill.saturating_sub(zone.formation.detection_ts));
                zone.right_censored = zone.fill_ts.is_none();
                zone.clone()
            })
            .collect()
    }
}

pub fn logical_zone_key(timeframe: &str, zone_id: u64) -> String {
    format!("{timeframe}:{zone_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FORMATION: &str = r#"{"bullish_fvg":{"detection_ts":1000,"formation_atr":2.0,"formation_basis":"three_bar_non_overlap","fvg_quality":0.8,"gap":1.0,"gap_atr":0.5,"gap_bps":1.0,"impulse_body_atr":0.7,"impulse_body_fraction":0.6,"impulse_close_location":0.9,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;

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
    fn first_formation_keeps_direction_and_bounds() {
        let formation = parse_payload(FORMATION).unwrap().formations().pop().unwrap();
        assert_eq!(formation.0, Direction::Bullish);
        assert_eq!(formation.1.lower, 10.0);
        assert_eq!(formation.1.upper, 11.0);
    }

    #[test]
    fn later_touch_and_fill_do_not_change_formation_cohort_fields() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar(
            1000,
            Availability {
                available_time_ns: 1100,
                source_sequence: 1,
            },
            FORMATION,
        )
        .unwrap();
        book.ingest_bar(2000, Availability { available_time_ns: 2100, source_sequence: 2 }, r#"{"fvg_first_touch":{"detection_ts":1000,"direction":"bullish","fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"touch_basis":"bar_range_overlap","touch_ts":2000,"upper":11.0,"zone_id":7},"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":2000,"first_touch_observed":true,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#).unwrap();
        let zone = book.finish(2000).pop().unwrap();
        assert_eq!(zone.formation_available_time_ns, 1100);
        assert_eq!(zone.first_touch_ts, Some(2000));
        assert_eq!(zone.fill_ts, Some(2000));
        assert_eq!(zone.formation.zone_id, 7);
    }

    #[test]
    fn unfinished_zone_is_right_censored_and_state_survives_parts() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar(
            1000,
            Availability {
                available_time_ns: 1100,
                source_sequence: 1,
            },
            FORMATION,
        )
        .unwrap();
        book.ingest_bar(
            2000,
            Availability {
                available_time_ns: 2100,
                source_sequence: 2,
            },
            "{}",
        )
        .unwrap();
        let zone = book.finish(2000).pop().unwrap();
        assert!(zone.right_censored);
        assert!(zone.active);
    }

    #[test]
    fn sidecar_lookup_uses_receipt_time_and_ignores_other_strata() {
        let lines = concat!(
            r#"{"timeframe":"30s","bar_close_ts_ms":1000,"bar_close_ts_ns":1000000000,"available_time_ns":111,"trigger_event_time_ns":1001,"trigger_source_sequence":1,"trigger_source_part":"tick/0","trigger_row_index":0,"availability_kind":"BOUNDARY_CROSSING_TICK"}"#,
            "\n",
            r#"{"timeframe":"15s","bar_close_ts_ms":1000,"bar_close_ts_ns":1000000000,"available_time_ns":222,"trigger_event_time_ns":1001,"trigger_source_sequence":2,"trigger_source_part":"tick/0","trigger_row_index":1,"availability_kind":"BOUNDARY_CROSSING_TICK"}"#,
            "\n",
        );
        let mut cursor = SidecarCursor::new(Cursor::new(lines), "15s");
        assert_eq!(
            cursor
                .lookup(1_000_000_000)
                .unwrap()
                .unwrap()
                .available_time_ns,
            222
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
