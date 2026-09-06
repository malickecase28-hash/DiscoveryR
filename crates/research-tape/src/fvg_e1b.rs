//! AP-002 E1B stage-anchor measurement (FIRST_TOUCH and FILL), implementing the
//! frozen E1B measurement contract on top of the E1 zone reconstruction.  Stage
//! anchors sit at the touch/fill event bars' boundary-crossing ticks; prospective
//! outcomes reference lawful native closes 1/3/5 bars after the anchor bar.

use crate::fvg_e1::{AnchorTick, Direction, ZoneLifecycle};
use serde::{Deserialize, Serialize};

pub const HORIZONS_BARS: [u64; 3] = [1, 3, 5];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage {
    #[serde(rename = "FIRST_TOUCH")]
    FirstTouch,
    #[serde(rename = "FILL")]
    Fill,
}

/// One stage anchor: the boundary-crossing tick of the event bar, with its
/// causal availability and the anchor price taken from that exact tick.
#[derive(Debug, Clone, Serialize)]
pub struct StageAnchor {
    pub stage: Stage,
    pub identity: String,
    pub zone_id: u64,
    pub direction: Direction,
    pub anchor_bar_close_ts: i64,
    pub available_time_ns: i64,
    pub source_sequence: i64,
    pub anchor_bid: f64,
    pub anchor_ask: f64,
    pub anchor_mid: f64,
    /// Fill stage only: whether the producer had recorded a touch before the fill.
    pub first_touch_observed: Option<bool>,
    pub formation_ts: i64,
    pub formation_available_time_ns: i64,
    pub formation_to_anchor_market_ms: i64,
    pub formation_to_anchor_known_ms: i64,
    pub gap_atr: f64,
    pub fvg_quality: f64,
}

pub fn stage_anchor(
    zone: &ZoneLifecycle,
    stage: Stage,
    anchor_tick: &AnchorTick,
) -> Result<StageAnchor, String> {
    let (anchor_bar_close_ts, available_time_ns, source_sequence, first_touch_observed) =
        match stage {
            Stage::FirstTouch => (
                zone.first_touch_ts
                    .ok_or_else(|| format!("zone {} has no emitted first touch", zone.identity))?,
                zone.first_touch_available_time_ns
                    .ok_or_else(|| format!("zone {} touch lacks availability", zone.identity))?,
                zone.first_touch_source_sequence
                    .ok_or_else(|| format!("zone {} touch lacks source identity", zone.identity))?,
                None,
            ),
            Stage::Fill => (
                zone.fill_ts
                    .ok_or_else(|| format!("zone {} has no terminal fill", zone.identity))?,
                zone.fill_available_time_ns
                    .ok_or_else(|| format!("zone {} fill lacks availability", zone.identity))?,
                zone.fill_source_sequence
                    .ok_or_else(|| format!("zone {} fill lacks source identity", zone.identity))?,
                Some(zone.first_touch_ts.is_some()),
            ),
        };
    // The anchor price must come from the exact boundary-crossing tick that
    // produced the stage availability.
    if anchor_tick.source_sequence != source_sequence
        || anchor_tick.received_time_ns != available_time_ns
    {
        return Err(format!(
            "stage anchor tick does not match the stage availability for {}",
            zone.identity
        ));
    }
    if !anchor_tick.mid.is_finite() || anchor_tick.mid <= 0.0 {
        return Err(format!("stage anchor mid is invalid for {}", zone.identity));
    }
    Ok(StageAnchor {
        stage,
        identity: zone.identity.clone(),
        zone_id: zone.formation.zone_id,
        direction: zone.formation.direction,
        anchor_bar_close_ts,
        available_time_ns,
        source_sequence,
        anchor_bid: anchor_tick.bid,
        anchor_ask: anchor_tick.ask,
        anchor_mid: anchor_tick.mid,
        first_touch_observed,
        formation_ts: zone.formation.detection_ts,
        formation_available_time_ns: zone.formation_available_time_ns,
        formation_to_anchor_market_ms: anchor_bar_close_ts
            .saturating_sub(zone.formation.detection_ts),
        formation_to_anchor_known_ms: available_time_ns
            .saturating_sub(zone.formation_available_time_ns)
            / 1_000_000,
        gap_atr: zone.formation.gap_atr,
        fvg_quality: zone.formation.fvg_quality,
    })
}

/// A lawful (availability-backed) native bar captured during the stream.
#[derive(Debug, Clone, Copy)]
pub struct LawfulBar {
    pub close_ts: i64,
    pub close: f64,
    pub available_time_ns: i64,
    pub source_sequence: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageOutcome {
    pub horizon_index: usize,
    pub horizon_bars: u64,
    pub outcome_close: f64,
    pub outcome_bar_close_ts: i64,
    pub outcome_available_time_ns: i64,
    pub outcome_source_sequence: i64,
    pub raw_price_delta: f64,
    pub raw_return_bps: f64,
    pub direction_adjusted_return_bps: f64,
}

/// Prospective outcomes at the lawful native closes 1/3/5 bars after the anchor
/// bar.  End-of-window shortfalls leave the trailing entries None (explicit,
/// never imputed).  Outcomes must be causally after the anchor receipt.
pub fn stage_outcomes(
    lawful: &[LawfulBar],
    anchor_bar_close_ts: i64,
    anchor_available_time_ns: i64,
    anchor_source_sequence: i64,
    anchor_mid: f64,
    direction: Direction,
) -> Result<[Option<StageOutcome>; 3], String> {
    let index = lawful.partition_point(|bar| bar.close_ts < anchor_bar_close_ts);
    match lawful.get(index) {
        Some(bar) if bar.close_ts == anchor_bar_close_ts => {}
        _ => {
            return Err(format!(
                "anchor bar {anchor_bar_close_ts} is not a lawful bar of this stratum"
            ))
        }
    }
    let mut outcomes: [Option<StageOutcome>; 3] = std::array::from_fn(|_| None);
    for (horizon_index, &offset) in HORIZONS_BARS.iter().enumerate() {
        let Some(bar) = lawful.get(index + offset as usize) else {
            continue;
        };
        if (bar.available_time_ns, bar.source_sequence)
            <= (anchor_available_time_ns, anchor_source_sequence)
        {
            return Err(format!(
                "stage outcome at {} is not causally after the anchor receipt",
                bar.close_ts
            ));
        }
        let raw_price_delta = bar.close - anchor_mid;
        let raw_return_bps = 10_000.0 * raw_price_delta / anchor_mid;
        let direction_adjusted_return_bps = match direction {
            Direction::Bullish => raw_return_bps,
            Direction::Bearish => -raw_return_bps,
        };
        outcomes[horizon_index] = Some(StageOutcome {
            horizon_index,
            horizon_bars: offset,
            outcome_close: bar.close,
            outcome_bar_close_ts: bar.close_ts,
            outcome_available_time_ns: bar.available_time_ns,
            outcome_source_sequence: bar.source_sequence,
            raw_price_delta,
            raw_return_bps,
            direction_adjusted_return_bps,
        });
    }
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize)]
pub struct Quantiles {
    pub n: u64,
    pub p05: Option<f64>,
    pub p25: Option<f64>,
    pub p50: Option<f64>,
    pub p75: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub max: Option<f64>,
}

pub fn quantiles(values: &[f64]) -> Quantiles {
    if values.is_empty() {
        return Quantiles {
            n: 0,
            p05: None,
            p25: None,
            p50: None,
            p75: None,
            p95: None,
            p99: None,
            max: None,
        };
    }
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    let pick = |p: f64| {
        Some(values[((values.len() - 1) as f64 * p).round() as usize])
    };
    Quantiles {
        n: values.len() as u64,
        p05: pick(0.05),
        p25: pick(0.25),
        p50: pick(0.50),
        p75: pick(0.75),
        p95: pick(0.95),
        p99: pick(0.99),
        max: values.last().copied(),
    }
}

/// Contract tail ceiling: with administrative right censoring fraction `c`,
/// the largest identifiable unconditional quantile is ~1 - c.
pub fn max_identifiable_quantile(censor_fraction: f64) -> f64 {
    (1.0 - censor_fraction).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fvg_e1::{Availability, ZoneBook};

    const FORMATION: &str = r#"{"bullish_fvg":{"detection_ts":1000,"formation_atr":1.0,"formation_basis":"three_bar_non_overlap","fvg_quality":0.8,"gap":0.5,"gap_atr":0.5,"gap_bps":10.0,"impulse_body_atr":1.2,"impulse_body_fraction":0.9,"impulse_close_location":0.9,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
    const TOUCH: &str = r#"{"fvg_first_touch":{"detection_ts":1000,"direction":"bullish","fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"touch_basis":"bar_range_overlap","touch_ts":3000,"upper":11.0,"zone_id":7}}"#;
    const FILLED: &str = r#"{"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":5000,"first_touch_observed":true,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;
    const GAP_THROUGH_FILL: &str = r#"{"fvg_filled":{"detection_ts":1000,"direction":"bullish","fill_basis":"wick_reaches_far_edge","fill_ts":5000,"first_touch_observed":false,"fvg_quality":0.8,"impulse_ts":900,"lower":10.0,"origin_ts":800,"upper":11.0,"zone_id":7}}"#;

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

    #[test]
    fn actual_emitted_touch_creates_exactly_one_stage_anchor() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), FORMATION, Some(20.0)).unwrap();
        book.ingest_bar_with_close_optional(3000, Some(availability(3100, 2, 10.5, 12.5)), TOUCH, Some(20.5)).unwrap();
        let zones = book.finish(3000);
        assert_eq!(zones.len(), 1);
        let anchor = stage_anchor(&zones[0], Stage::FirstTouch, &availability(3100, 2, 10.5, 12.5).anchor.unwrap()).unwrap();
        assert_eq!(anchor.anchor_bar_close_ts, 3000);
        assert_eq!(anchor.source_sequence, 2);
        assert!((anchor.anchor_mid - 11.5).abs() < 1e-9);
        assert!(stage_anchor(&zones[0], Stage::Fill, &availability(3100, 2, 10.5, 12.5).anchor.unwrap()).is_err());
    }

    #[test]
    fn duplicate_touch_is_rejected_by_the_zone_book() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), FORMATION, Some(20.0)).unwrap();
        book.ingest_bar_with_close_optional(3000, Some(availability(3100, 2, 10.5, 12.5)), TOUCH, Some(20.5)).unwrap();
        let duplicate = TOUCH.replace("\"zone_id\":7}}", "\"zone_id\":7},\"fvg_first_touch\":{\"detection_ts\":1000,\"direction\":\"bullish\",\"fvg_quality\":0.8,\"impulse_ts\":900,\"lower\":10.0,\"origin_ts\":800,\"touch_basis\":\"bar_range_overlap\",\"touch_ts\":3000,\"upper\":11.0,\"zone_id\":7}}");
        assert!(book.ingest_bar_with_close_optional(4000, Some(availability(4100, 3, 10.6, 12.6)), &duplicate, Some(20.6)).is_err());
    }

    #[test]
    fn gap_through_fill_creates_fill_anchor_without_touch_anchor() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), FORMATION, Some(20.0)).unwrap();
        book.ingest_bar_with_close_optional(5000, Some(availability(5100, 2, 10.4, 12.4)), GAP_THROUGH_FILL, Some(20.4)).unwrap();
        let zones = book.finish(5000);
        assert!(zones[0].first_touch_ts.is_none() && zones[0].fill_ts.is_some());
        let anchor = stage_anchor(&zones[0], Stage::Fill, &availability(5100, 2, 10.4, 12.4).anchor.unwrap()).unwrap();
        assert_eq!(anchor.first_touch_observed, Some(false));
        assert!(stage_anchor(&zones[0], Stage::FirstTouch, &availability(5100, 2, 10.4, 12.4).anchor.unwrap()).is_err());
    }

    #[test]
    fn touched_fill_stratifies_as_first_touch_observed_true() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), FORMATION, Some(20.0)).unwrap();
        book.ingest_bar_with_close_optional(3000, Some(availability(3100, 2, 10.5, 12.5)), TOUCH, Some(20.5)).unwrap();
        book.ingest_bar_with_close_optional(5000, Some(availability(5100, 3, 10.4, 12.4)), FILLED, Some(20.4)).unwrap();
        let zones = book.finish(5000);
        let anchor = stage_anchor(&zones[0], Stage::Fill, &availability(5100, 3, 10.4, 12.4).anchor.unwrap()).unwrap();
        assert_eq!(anchor.first_touch_observed, Some(true));
    }

    #[test]
    fn orphan_touch_never_creates_an_anchor() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), TOUCH, Some(20.0)).unwrap();
        let orphan_touch_count = book.orphan_touch_count;
        let zones = book.finish(1000);
        assert!(zones.is_empty());
        assert_eq!(orphan_touch_count, 1);
    }

    #[test]
    fn stage_anchor_price_is_tied_to_the_boundary_tick() {
        let mut book = ZoneBook::new("15s");
        book.ingest_bar_with_close_optional(1000, Some(availability(1100, 1, 10.0, 12.0)), FORMATION, Some(20.0)).unwrap();
        book.ingest_bar_with_close_optional(3000, Some(availability(3100, 2, 10.5, 12.5)), TOUCH, Some(20.5)).unwrap();
        let zones = book.finish(3000);
        let mismatched = availability(9999, 999, 10.5, 12.5);
        assert!(stage_anchor(&zones[0], Stage::FirstTouch, &mismatched.anchor.unwrap()).is_err());
    }

    fn lawful_bars() -> Vec<LawfulBar> {
        (0..8)
            .map(|i| LawfulBar {
                close_ts: 1000 + (i as i64 + 1) * 2000,
                close: 20.0 + i as f64,
                available_time_ns: 1100 + (i as i64 + 1) * 2000,
                source_sequence: 10 + i as i64,
            })
            .collect()
    }

    #[test]
    fn outcomes_reference_the_1_3_5_lawful_closes_after_the_anchor() {
        let lawful = lawful_bars();
        let outcomes = stage_outcomes(&lawful, 3000, 3100, 11, 20.5, Direction::Bullish).unwrap();
        assert_eq!(outcomes[0].as_ref().unwrap().outcome_close, 21.0);
        assert_eq!(outcomes[1].as_ref().unwrap().outcome_close, 23.0);
        assert_eq!(outcomes[2].as_ref().unwrap().outcome_close, 25.0);
        assert!(outcomes[0].as_ref().unwrap().outcome_bar_close_ts > 3000);
    }

    #[test]
    fn end_of_window_outcomes_are_explicitly_missing() {
        let lawful = lawful_bars();
        let outcomes = stage_outcomes(&lawful, lawful[lawful.len() - 1].close_ts, 9900, 99, 20.0, Direction::Bullish).unwrap();
        assert!(outcomes.iter().all(|o| o.is_none()));
    }

    #[test]
    fn pre_anchor_outcome_movement_fails_closed() {
        let lawful = lawful_bars();
        // Anchor receipt placed AFTER the candidate outcome bars' receipts.
        let result = stage_outcomes(&lawful, 3000, 9900, 99, 20.5, Direction::Bullish);
        assert!(result.is_err());
    }

    #[test]
    fn non_lawful_anchor_bar_is_rejected() {
        let lawful = lawful_bars();
        assert!(stage_outcomes(&lawful, 2999, 3100, 11, 20.5, Direction::Bullish).is_err());
    }

    #[test]
    fn tail_ceiling_is_one_minus_censor_fraction() {
        assert!((max_identifiable_quantile(0.1557) - 0.8443).abs() < 1e-12);
    }
}
