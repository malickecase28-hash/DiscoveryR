# AP-002 E1B Measurement Contract — FROZEN

Frozen 2026-09-06 under director ruling. Machine-readable record:
`AP-002_E1B_EXPERIMENT_CONTRACT.json`. Exposure: **E1_PHENOTYPE** (the FVG's own
lawful lifecycle — no E2 context, no E3 context, no strategy logic). Confirmation
**LOCKED**. Native strata 15s/30s/1m/5m/15m/1h/4h; cross-scale identity prohibited.

## Stage anchors

Two lifecycle stage anchors: **FIRST_TOUCH** and **FILL**.

### FIRST_TOUCH

```text
Eligible population:  lawful in-window FVG formations
Anchor:               first actual emitted fvg_first_touch event
Anchor cohort:        only zones with a genuine emitted first-touch event
Excluded:             gap-through fills (first_touch_observed = false),
                      orphan/pre-development zones, imputed touches,
                      future-derived touch status
```

Two denominators, kept separate:

```text
TOUCH_INCIDENCE          = lawful first-touch zones / lawful formation zones
FIRST_TOUCH FORWARD STUDY: N = lawful emitted first-touch anchors
```

### FILL

```text
Primary cohort: lawful fvg_filled events belonging to lawful in-window formations
Excluded:       orphan/pre-development fills
Stratified by:  first_touch_observed = true | false
                (gap-through fills are never imputed a first touch)
```

### Anchor availability and price (both anchors)

```text
available_time = received_ts_ns of the boundary-crossing source tick
                 that makes the event bar complete
tie-break      = source_sequence
anchor price   = bid, ask, mid=(bid+ask)/2 of that exact trigger tick
                 (event-bar close is NOT the anchor)
```

## Allowed E1 conditioning

Only properties of the same FVG object already known at the stage anchor:
direction, gap, gap_atr, fvg_quality, zone bounds, formation age / time since
formation, time since first touch where applicable, `first_touch_observed` at
FILL, native timeframe. No unrelated detector context.

## Lifecycle outputs

FIRST_TOUCH: formation→touch duration; touch→fill duration; remaining
active/unfilled fraction after touch; touch→fill path; right censoring after
touch. FILL: formation→fill duration; touch→fill duration if touched;
gap-through-fill indicator. Market/event duration and known/availability duration
are reported separately, never merged.

## Prospective price outcomes

At FIRST_TOUCH and FILL anchors: the lawful native closes **1, 3, 5 bars** after
the anchor bar, referenced to the stage-availability `anchor_mid`. Reported per
horizon: N, raw price delta, raw return bps, direction-adjusted return bps,
quantiles p05/p25/p50/p75/p95. Median-split descriptive conditioning on gap_atr
and fvg_quality plus direction split; declared splits only. **No significance
claims.**

## Tail reporting

p99, max observed, right-censor fraction, and the explicit ceiling
`max identifiable unconditional quantile ≈ 1 − censor_fraction` (e.g. 15.6%
censoring ⇒ only through roughly the 84.4th unconditional quantile is
identifiable without stronger survival assumptions). No heavy survival-analysis
framework.

## Required tests

- FIRST_TOUCH: one anchor per actual emitted event; duplicate touch rejected;
  gap-through fill creates no touch anchor; orphan touch excluded; stage
  availability and price tied to the boundary tick.
- FILL: normal touched fill; same-bar touch+fill; gap-through fill with
  `first_touch_observed=false`; orphan fill excluded; duplicate fill rejected.
- OUTCOMES: no pre-anchor movement; 1/3/5 lawful native closes; equal-receipt
  ordering by source_sequence; end-of-window missing outcomes explicit.
- TAIL: p99, max, censor ceiling. REPRODUCIBILITY: two fresh runs, identical
  logical result hash.

## Execution rules

Reuse the existing typed FVG parser, availability sidecar, trigger-tick anchor
index and zone reconstruction — do not duplicate those systems. **Rust only**
(`PIPELINE_RULES.md`). DEVELOPMENT only. Small committed review artifacts; large
row-level tables stay local with hashes. **No interpretation of outputs before a
separate interpretation stage.**
