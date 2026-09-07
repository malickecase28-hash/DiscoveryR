# AP-006 Work-Package Status

**PROJECT:** AP-006 Feed Health, XAUUSD, Program R
**ACTIVE PLAN AUTHORITY:** `AP-006_FEED_HEALTH_PROGRAM_R_RESEARCH_PLAN_V2.md` + `AP-006_FIELD_RECONCILIATION_AMENDMENT.md` (supersedes the degradation-episode framing in Sections 3, 5-9)
**EXECUTION SCHEDULE RULE:** `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 applies prospectively. Base-plan PERT hour tables are retained as historical planning text but are not executable forecasts for new work.
**HISTORICAL PLAN:** `AP-006_FEED_HEALTH_PROGRAM_R_RESEARCH_PLAN_V1.md`, preserved, superseded before any work executed against it
**FROZEN INPUT:** manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified, same identity as AP-001)
**CONFIRMATION STATUS:** LOCKED — no work has executed, confirmation leaf untouched
**PROJECT STATE:** NOT STARTED. No delta gate needed: no WP1 evidence exists under the original (incorrect) episode-based design.

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract and field enumeration | NOT STARTED | — | All eight anomaly-flag types enumerated with real field names; causal ordering passes |
| WP2 Firing discovery | NOT STARTED | — | Per-type incidence, inter-firing interval, and co-firing rate has a result or `INSUFFICIENT_SUPPORT` |
| WP3 Market-stress conditioning and session conditioning | NOT STARTED | — | Every firing type has a `DEGRADED_WITH_MARKET_STRESS` / `DEGRADED_WITHOUT_MARKET_STRESS` / `AMBIGUOUS_MARKET_CONTEXT` classification and a session-conditioned rate |
| WP4 Cross-resolution contamination tests | NOT STARTED | — | AP-005 (tick) and AP-003 (bar) contamination results are `MATERIAL` / `NULL` / `INCONCLUSIVE`, plus the cross-resolution comparison |
| WP5 Candidate challenge | NOT STARTED | — | Each candidate `SURVIVED_CHALLENGE` / `REJECTED` / `NULL` / `INCONCLUSIVE` |
| WP6 Confirmation and closure | LOCKED (opens once, after WP5) | — | Each candidate `CONFIRMED` / `FAILED_CONFIRMATION` / `INCONCLUSIVE_CONFIRMATION`; exclusion-rule recommendation produced |

Six work packages, per the base plan; this project never had a separate structural-conditioning package, and characterization is folded into WP6.

## WP1 dispatch requirements (from the field-reconciliation amendment)

Before WP1 closes, it must additionally freeze:

1. detector role corrected from a `HEALTHY`/`DEGRADED` state machine to eight independent instantaneous anomaly-flag types (`clock_inversion`, `event_time_gap`, `event_time_regression`, `expected_market_closure`, `latency_spike`, `sequence_duplicate`, `sequence_gap`, `sequence_regression`); episode onset/duration/recovery/flapping language is deleted, not deferred;
2. per-type firing counts in the development leaf, feeding the 300/150 support rule;
3. the co-occurrence window used to group nearby firings (same or different types) into one contamination incident, justified the same way, not just asserted;
4. whether any of the eight types fire too rarely to support their own table and must be reported only in an aggregate "any anomaly present" category.

## Explicit blocking dependency

WP4's contamination tests are explicitly scoped to reuse AP-005's and AP-003's frozen outputs, not re-derive them privately. Per `AP-005/AP-005_WORK_PACKAGE_STATUS.md` and `AP-003/AP-003_WORK_PACKAGE_STATUS.md`, neither has executed WP1 yet. **AP-006's WP4 cannot start before AP-003's WP1/WP2 and AP-005's WP1/WP2 have produced the populations it reuses.** This is a real scheduling dependency, not a formality; record it before dispatching any of the three.

## Schedule note

New work uses measured throughput or an explicit `NOT_YET_MEASURED` first-checkpoint estimate, named checkpoints, and the frozen deviation rule from `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5. Do not dispatch a new task using an unsupported PERT hour bucket as its operative forecast.

## Standing notes

- Concurrency cap: 2 active workers.
- The central deliverable is the exclusion-rule recommendation, not a phenotype table; per the execution skill's cross-project rule, a `MATERIAL` result here triggers a separate change decision for each affected AP project. It does not automatically rerun or invalidate them.
