# AP-001 V3 final hardening amendment

**Status:** APPROVED normative amendment  
**Project state:** STOPPED at the WP1/WP2 boundary until explicit resume  
**Applies to:** `AP-001_DRIFT_BURST_PROGRAM_R_RESEARCH_PLAN_V3.md`  
**Does not rewrite:** historical `AP-001_PLAN_FROZEN.md` or completed WP1 evidence

This amendment closes three specification gaps found during cross-detector plan audit. It is part of the active AP-001 execution contract. Where this amendment conflicts with the earlier AP-001 frozen-plan copy or pre-V3 WP2-WP4 contracts, this amendment and V3 govern work that has not yet executed.

## 1. Native clock rule is explicit and transferable

The detector's native data granularity determines its primary phenotype clock.

- Tick-native detectors use justified wall-clock or event-time horizons.
- Bar-native detectors use native-bar counts on the detector's own scale.
- A cross-resolution bridge is a separate outcome layer and never replaces the native clock.

For Drift-Burst, `+5s/+15s/+30s/+60s/+120s/+300s` remain the native tick-time market horizons. The V3 tick->bar bridge remains separate and measures later bar-scale expression of the same tick anchor.

## 2. Frozen support floor for AP-001

Candidate promotion may no longer refer to an undefined "frozen minimum."

The AP-001 support rule is:

- **300 effective observations** is the default promotion minimum after the candidate's conditioning, matching, exclusions, and causal eligibility rules are applied.
- A scientifically valid population with **150-299 effective observations** may use the predeclared fallback only if support spans the required chronological breadth. It carries `LOW_SUPPORT_FALLBACK` through challenge and confirmation.
- **Below 150 effective observations** is `INSUFFICIENT_SUPPORT`.
- For matched or paired controls, support means matched or paired analytical units after exclusions, not raw input rows.
- The threshold is never relaxed because an observed effect looks attractive.

The 300-observation floor is an operational screening floor, not a claim of guaranteed tail precision. At 300 observations a 5 percent tail contains roughly 15 observations before further subdivision; the 150 fallback is explicitly weaker and must remain labeled as such.

## 3. Named same-domain lineage audit

Rule 11 is applied by named pair rather than through a generic lineage warning.

Before any same-domain relationship is promoted as independent or additive information, WP3 must adjudicate Drift-Burst against:

- `micro_volatility`;
- `quote_arrival`;
- `quote_dynamics`;
- `quote_pressure`;
- `spread_state`;
- `feed_health`.

Each pair receives one status:

- `INDEPENDENT_ENOUGH_FOR_CONTROL`;
- `PARTIALLY_OVERLAPPING`;
- `DEPENDENT_SAME_INFORMATION_FAMILY`;
- `QUALITY_OR_CONDITION_CONTROL_ONLY`;
- `UNRESOLVED_LINEAGE`.

A separate detector name is not evidence of independent information. `feed_health` is quality instrumentation and `spread_state` is a market/trading condition unless frozen producer evidence establishes otherwise; neither is counted as an independent confirmation vote. An `UNRESOLVED_LINEAGE` pair cannot be promoted as confluence.

## 4. Pre-WP2 delta gate

WP1 remains `COMPLETE` for the evidence it actually established: input identity, Drift-Burst causal ordering, anchor mechanics, population reconstruction, confirmation partition, and benchmark.

V3 introduced requirements after that WP1 close. Those requirements do not justify rerunning the entire completed package. Before WP2 executes, a narrow delta gate must verify and record only the newly required items:

1. lawful availability bindings for bar/structural context used by the tick->bar bridge;
2. the first-fully-post-anchor bar alignment rule;
3. V3 left/right-boundary and warm-state treatment against existing WP1 evidence;
4. the 300/150 support rule in WP2-WP5 contracts;
5. the named same-domain lineage statuses required before WP3 promotion.

If an item is already proven by frozen evidence, cite it. If a gap exists, repair only that gap.

The pre-V3 `R2_LIFECYCLE_MARKET`, `R3_TICK_CONTEXT`, and `R4_STRUCTURAL_CONTEXT` contracts are **not executable as currently frozen**. They must be reconciled and re-frozen against V3 plus this amendment before their work package starts. No discovery evidence has yet been produced, so this changes prospective contracts only.

## 5. AP-006 quality finding is a change trigger, not an automatic amendment

If AP-006 later confirms material contamination from `feed_health` degradation, AP-001 does not automatically rerun or discard evidence.

The confirmed quality finding triggers a separate AP-001 change decision stating:

- affected evidence and time intervals;
- whether the action is sensitivity-only, flagging, exclusion rerun, or no change;
- which completed artifacts are inside the blast radius;
- schedule impact;
- confirmation impact.

The same rule applies independently to AP-003, AP-004, and AP-005. One project's quality result never silently rewrites another frozen project.

## 6. Execution instruction

Until explicit research resume:

- WP1 stays closed `COMPLETE`;
- WP2-WP4 remain stopped;
- old WP2-WP4 contracts must not be dispatched;
- confirmation remains locked;
- the next authorized action on resume is the narrow V3 delta gate and contract reconciliation, not behavioral scanning.
