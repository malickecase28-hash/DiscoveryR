# AP-006 field-reconciliation amendment (pre-V3)

**Status:** APPROVED, supersedes the degradation-episode framing in V1 and V2 before either executes
**Trigger:** `payload_manifest.json` (verified SHA256 `78d5fc20…3eed`, matching the frozen identity) shows the real `feed_health` schema does not match this plan's original design
**Applies to:** `AP-006_FEED_HEALTH_PROGRAM_R_RESEARCH_PLAN_V2.md`, before WP1 executes
**Rule invoked:** the same one AP-001 used for its own V3 delta gate: repair the gap, don't redo the whole document

## What was assumed versus what the schema actually shows

The V1/V2 plan assumed `feed_health` exposes a persistent `HEALTHY`/`DEGRADED`/`INVALID` state with onset, duration, recovery, and flapping behavior, the same general shape as a lifecycle or regime detector.

The real declared tick payload contract shows eight independent, instantaneous anomaly-flag event types, each firing at a specific tick with its own descriptive fields, with no state field and no dwell time anywhere in the contract:

```text
feed_health.clock_inversion:          event_time_ns, received_time_ns, signed_latency_ns
feed_health.event_time_gap:           gap_ns, threshold_ns
feed_health.event_time_regression:    (point event, per-tick)
feed_health.expected_market_closure:  (point event, per-tick)
feed_health.latency_spike:            (point event, per-tick)
feed_health.sequence_duplicate:       (point event, per-tick)
feed_health.sequence_gap:             (point event, per-tick)
feed_health.sequence_regression:      (point event, per-tick)
```

There is no episode to have a duration, no recovery to measure, and no flapping to detect, because there is no persistent state to flap between. Each of the eight types is its own independent alarm that fires or doesn't fire at a given tick. This is the same WP1 field-enumeration requirement doing its job a second time in the same conversation; it is not a new kind of mistake, it is the same discipline catching the same class of error before it reached code.

## What changes

**Detector role.** Confirmed `QUALITY_INSTRUMENTATION`, but the scientific unit is not an episode. It is a single anomaly-flag firing, of one of eight named types, at a lawful tick.

**Definition of done, restated.** For each anomaly type: incidence rate, whether it clusters with other anomaly types at the same or nearby ticks, the observable market-stress context around the firing, and whether ticks at or near a firing measurably contaminate another detector's evidence at that point. The exclusion-rule recommendation is the same deliverable as before; only the unit that recommendation is built from changes, from "exclude ticks inside a degraded episode" to "exclude or flag ticks at or within a defined short window of a named anomaly type's firing."

**Sections requiring rewrite before WP1 executes:**

- Section 3 (Detector role and scientific unit) — replace "episode, tracked from onset through terminal state" with "single firing of one of eight named anomaly types."
- Section 5 (Left/right boundary, episode-continuation rule) — the 5-second flap-continuation rule is deleted; there is no episode to continue. Replace with a co-occurrence window rule: how close together do two firings (same type or different types) have to be to call them one contamination incident versus two separate ones. This is a new arbitrary-number risk this amendment introduces and must be justified the same way, not just asserted.
- Section 6 (Episode-lifecycle measures) — "time to recovery," "flap count," "total degraded duration" are removed. Replaced with: per-type incidence rate, inter-firing interval distribution per type, and co-firing rate across types.
- Section 7 (What the study measures) — same replacement as Section 6; drop episode-onset framing.
- Section 8 (Market-stress conditioning) — unchanged in principle, applied per firing instead of per episode: characterize `micro_volatility`, `spread_state`, `quote_arrival`, and session/time context immediately before a firing versus baseline. Preserve V2's market-context labels `DEGRADED_WITH_MARKET_STRESS`, `DEGRADED_WITHOUT_MARKET_STRESS`, and `AMBIGUOUS_MARKET_CONTEXT`. Do not infer whether the underlying market move itself was genuine or fake.
- Section 9 (Cross-resolution contamination tests) — unchanged in structure (AP-005 tick-domain subject, AP-003 bar-domain subject, cross-resolution comparison); the comparison is now "AP-003/AP-005 evidence inside versus outside a window around a named firing type," not "inside versus outside a degraded episode."

**Sections unaffected:** Section 1's central deliverable (an exclusion-rule recommendation other AP-series projects can act on through their own change decisions), Section 2 scope, Section 4 causal clock and data identities, most of 10-19.

## Required WP1 additions

Before WP2 executes, WP1 must additionally freeze:

1. per-type firing counts in the development leaf, across all eight anomaly types, feeding the 300/150 support rule;
2. the co-occurrence window used to group nearby firings into one contamination incident (the new arbitrary-number risk noted above);
3. whether any of the eight types fire so rarely that they must be reported `INSUFFICIENT_SUPPORT` on their own and only usable in an aggregate "any anomaly present" category.

## Decision

`APPROVED` before AP-006 WP1 dispatches.
