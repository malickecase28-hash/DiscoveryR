# AP-002 E1A Synthesis — FROZEN

Frozen 2026-09-06 on the **corrected** evidence after the scope repair.
Machine-readable record: `AP-002_E1A_SYNTHESIS.json`.

## Accepted scanner evidence

```text
logical_result_hash = 1b701c016bd1effa300742ee74ce73d6e0e96313d62847d231391061dc94dca6
zones               = 381,498 lawful formations
preflight           = 51,969,691 rows, 0 regressions
integrity gate      = PASS (9/9, both runs)
reproducibility     = two fresh executions, identical logical / preflight / zone hashes
development window  = [2025-07-31T16:15:00Z inclusive, 2026-05-05T12:39:00Z exclusive)
```

Supersedes `735bd7ab…` (`SUPERSEDED_SCOPE_BOUNDARY_MISALIGNMENT`), which itself
superseded `4562b87b…`. Chain and deltas: `AP-002_E1A_SCOPE_REPAIR_IMPACT_ASSESSMENT.md`.

## Canonical observations

1. **381,498 lawful FVG formations** across the seven native XAUUSD strata,
   inside the frozen development window. Cross-scale object identity prohibited.
2. **Median formation → first touch = 2 native bars at every stratum.**
   Median formation → fill = 7/7/7/7/8/8/5 native bars (15s…4h).
3. **Rapid body + materially long-lived right tail** (p95 fill in hundreds of
   native bars at fine/mid strata), with tail estimates increasingly limited by
   administrative right censoring as native scale coarsens (0.45% → 15.57%).
4. `fill_rate + right_censored_rate = 1` by construction — an accounting
   complement, not two findings. Coarse-scale fill fractions are window-censored
   lower bounds, not lifetime probabilities.
5. Bullish formation share descriptively rises with native scale (51.0% → 60.5%):
   **OBSERVED, not generalized, not confirmed.**
6. Raw prospective medians positive in **21/21** cells; direction-adjusted
   medians **18 negative / 1 zero / 2 positive** (1h h3 +0.85 bps n=523;
   4h h5 +12.51 bps n=167). Descriptive only; no null test run.
7. Event-level availability delays are small in the body (touch p50 92–119 ms,
   fill p50 85–115 ms; p95 0.34–1.63 s) with large session-gap tails (~2.1 days max).

## Resolved anomalies

- **A1 `RESOLVED_EXPECTED_SEMANTIC_CASE`** — 1,010 formed zones filled without an
  emitted first touch: the linked producer tests far-edge fill independently of
  overlap (gap-through fill). The premise "a filling bar necessarily overlaps the
  zone" is false and removed from the canonical record.
- **A2 `LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE`** — 536 orphan events (178/358);
  every orphan zone predates its stratum's first lawful in-window formation.
  Not missing formations; excluded from formed cohorts. Note: two lifecycle
  events have detection timestamps exactly at the 16:15 boundary but originate
  from excluded pre-boundary formation-bar state (the corrected bar-view gate
  requires the formation bar itself to belong to the lawful cohort, not merely
  a detection timestamp equal to the boundary instant).
- **SCOPE `REPAIRED_AND_RE_RUN`** — start-boundary misalignment found, repaired,
  re-materialized, re-scanned twice, re-verified.

## Rejected interpretations

"a filling bar necessarily overlaps the zone" · "1h has strongest clock
dependence" · P-02's "4h coverage excess" as stated · P-01's "19/21 negative"
(corrected to 18/1/2) · the anomaly interpretation of A1 · orphans as missing
formations.

## E1B requirements (one stage-anchor extension, two anchors)

- **FIRST_TOUCH — REQUIRED, `BLOCKED_PENDING_MEASUREMENT_CONTRACT`.**
  Cohort = formation → actual emitted `fvg_first_touch`. Gap-through fills
  (`first_touch_observed=false`) are not imputed as touch and are excluded.
- **FILL — READY, second priority.** All lawful `fvg_filled` events, stratified
  by `first_touch_observed` true/false.
- Tail reporting additions: p99, max, censor-aware unconditional quantiles, and
  "max identifiable unconditional quantile = 1 − censor fraction" per stratum.
- Paired per-event information-delay distributions with the anchor clock declared.

## Question Generator candidates (PARTIAL eligibility — do not launch yet)

Direction-adjusted median pattern under a drift-aware null · bullish-share
gradient vs window drift · gap-through-fill frequency vs session-gap density ·
lifecycle tail beyond p95 · 4h incidence uptick (small-n) ·
`first_touch_observed` stratification at the FILL anchor.

## Claims that must NOT be made

No edge/alpha/profitability/strategy/predictive-advantage claims · fill fraction
is not a hazard or lifetime probability · the negative direction-adjusted medians
are not a tradable reversal effect · 1h h3 / 4h h5 positives are small-sample
medians, not sign reversals · no pooling, no cross-scale identity · HTF FVGs are
not established as structurally bullish · no clock-dependence claims from mixed
quantiles · orphans are not missing formations · nothing transfers to
E2/E3/confirmation/live.

## Post-challenge annotations (AP-002_E1_FINAL_METHOD_CHALLENGE)

- **Sign reversal**: SURVIVES_AS_DESCRIPTIVE_COHORT_PATTERN across nested
  formation/touch/fill cohorts — NOT a stage effect; the E2 stage-matched
  lifecycle panel is required before any stage-conditioned claim.
- **Censoring language**: formation→touch p95 is not identifiable unconditionally
  at 4h (ceiling = touch-incidence p94.01); p99/max at 5m–4h are observed-event
  quantiles only; formation→fill p95 is not identifiable at 1h/4h; all lifecycle
  maxima (~118.5 days) are window-boundary artifacts.

## Final state

```text
E1A_FORMATION_PHENOTYPE        = FROZEN
FIRST_TOUCH_E1B                = BLOCKED_PENDING_MEASUREMENT_CONTRACT
FILL_E1B                       = READY_SECOND_PRIORITY
QUESTION_GENERATOR_ELIGIBILITY = PARTIAL
CONFIRMATION                   = LOCKED
```
