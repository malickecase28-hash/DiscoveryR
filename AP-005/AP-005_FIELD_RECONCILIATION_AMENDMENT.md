# AP-005 field-reconciliation amendment (pre-V3)

**Status:** APPROVED, supersedes the regime-based framing in V1 and V2 before either executes
**Trigger:** `payload_manifest.json` (verified SHA256 `78d5fc20…3eed`, matching the frozen identity) shows the real `quote_pressure` schema does not match this plan's original design
**Applies to:** `AP-005_QUOTE_PRESSURE_PROGRAM_R_RESEARCH_PLAN_V2.md`, before WP1 executes
**Rule invoked:** the same one AP-001 used for its own V3 delta gate: repair the gap, don't redo the whole document

## What was assumed versus what the schema actually shows

The V1/V2 plan assumed `quote_pressure` exposes a single discrete or continuous regime field (`ALIGNED_STRONG`/`WEAK`/`OPPOSED`, or a continuous score requiring discretization), studied as a `STATE_REGIME` with a transition matrix and dwell-time distribution, the same shape as Drift-Burst's `state` field.

The real declared tick payload contract shows two independent event-style sub-detectors:

```text
quote_pressure.buy_quote_pressure:  semantics, direction_pressure, pressure, instantaneous_pressure, bid_move, ask_move
quote_pressure.sell_quote_pressure: semantics, direction_pressure, pressure, instantaneous_pressure, bid_move, ask_move
```

There is no `state` field, no dwell-time-bearing lifecycle, and no single regime dimension. `buy_quote_pressure` and `sell_quote_pressure` are parallel, independently-firing measurements, structurally closer to `quote_arrival`'s `arrival_burst`/`arrival_gap` pair or `micro_volatility`'s burst/compression/transition events than to Drift-Burst's state machine.

This is exactly the failure mode the WP1 field-enumeration requirement exists to catch, and it caught it before any code ran. Nothing here invalidates the requirement; it validates it. The plan is wrong precisely because the field gap was never assumed away; it was flagged, and now real evidence resolves it.

## What changes

**Detector role.** `quote_pressure` is not `STATE_REGIME`. It is closer to `EVENT`, paired: two independent signals (buy-side, sell-side) that each carry an instantaneous pressure/direction reading, not a persistent state with occupancy.

**Scientific unit.** One scientific unit is one lawful firing of `buy_quote_pressure` or `sell_quote_pressure` (kept as two related but separate populations, not merged), each carrying its own `pressure`, `instantaneous_pressure`, `direction_pressure`, `bid_move`, `ask_move` at the firing tick.

**Alignment/opposition is derived, not read.** "Aligned" or "opposed" pressure, which the original plan treated as a field to discretize, must instead be constructed from the relationship between the two signals: whether `buy_quote_pressure` and `sell_quote_pressure` fire together, in sequence, or in isolation, and how their magnitudes compare when both are present in a short window. This is a measurement design decision for WP1 to freeze, not an assumption to carry forward from V2.

**Sections requiring rewrite before WP1 executes:**

- Section 3 (Detector role and scientific unit) — replace `STATE_REGIME` framing with paired-event framing above.
- Section 5 (discretization requirement) — delete. There is no continuous score requiring a discretization rule; there are two discrete-firing signals requiring a co-occurrence design instead.
- Section 6 (measurement clocks) — the regime-entry anchor becomes a firing-event anchor for each of the two signals; forward horizons and the tick->bar bridge design carry over unchanged, since those were never regime-specific.
- Section 7 (what the study measures) — "transition matrix" and "dwell time" language is removed; replaced with firing-rate, co-occurrence rate (both signals within a short window versus one alone), and magnitude comparison when both are present.
- Section 8 (Drift-Burst redundancy test) — unchanged in principle. The Control C redundancy test against Drift-Burst's own state remains exactly as valuable, now applied to the paired-firing anchor instead of a regime anchor.

**Sections unaffected:** Sections 1 (objective, restated above), 2 (scope), 4 (causal clock, data identities), 9 (structural context), most of 10-19. The objective ("does quote_pressure add information beyond Drift-Burst's own state") does not change; only the object being anchored on changes.

## Required WP1 additions

Before WP2 executes, WP1 must additionally freeze:

1. whether `buy_quote_pressure` and `sell_quote_pressure` ever fire on the exact same tick, and if so how the co-occurrence is defined;
2. the window used to call two firings "co-occurring" versus "sequential" versus "isolated" (this is the new arbitrary-number risk this amendment introduces, and it must be justified the same way AP-001's support floor was: state the reason, not just the number);
3. per-signal firing counts in the development leaf, feeding the same 300/150 support rule the rest of the program uses.

## Decision

`APPROVED` before AP-005 WP1 dispatches.
