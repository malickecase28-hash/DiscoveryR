# AP-003 Risk Register

From `AP-003_BOS_CHOCH_PROGRAM_R_RESEARCH_PLAN_V3.md` §16, plus one row added by field reconciliation. Status updated at work-package gates; none are checked yet since no work package has started.

| Risk | Control | Contingency | Status |
| --- | --- | --- | --- |
| unverified payload fields | WP1 enumerates actual fields before design commits further | if a field the plan assumed does not exist, the affected measurement is dropped, not invented | OPEN — WP1 not started |
| primary-scale mis-selection | frozen rule in Section 5, applied mechanically to measured counts | reselect scale only through a change decision, not silently | OPEN — WP1 not started |
| wall-clock/bar-count confusion | native-bar horizons remain primary; the bounded `+5/+15/+30/+60s` tick microreaction bridge is reported separately | reject any artifact that substitutes the tick bridge for the native-bar phenotype or pools the clocks | OPEN |
| future information leakage | use each event's verified lawful availability coordinate; `bar_close_ts` only if WP1 proves it is the availability clock | reject affected evidence and repair only the offending join | OPEN |
| left-truncated first cycle | `LEFT_TRUNCATED_STRUCTURE` flag, excluded from BOS/CHOCH split only | include in plain incidence count only | OPEN |
| right censoring | explicit `CENSORED_HORIZON` status | censor-aware analysis where required | OPEN |
| **retroactive break invalidation not modeled** *(added by field reconciliation)* | record invalidation as a post-event outcome; never use eventual invalidation as an anchor-time category | reject future-conditioned tables; prospectively re-anchor on invalidation if later useful | OPEN |
| lineage duplication with `swings`/`local_structure` | Rule 11 dependency check; pre-set `DEPENDENT_SAME_INFORMATION_FAMILY` for `source_swing_id`-linked attributes specifically | collapse dependent context into one information family | OPEN |
| search explosion across seven scales | one primary scale first, others as context only | register cross-scale ideas for later work, not parallel studies | OPEN |
| cross-resolution search explosion | fixed bridge horizons only at the primary discovery scale | do not add arbitrary wall-clock grids or cross every tick field with every bar field | OPEN |
| post-anchor tick-state leakage | later tick transitions are outcomes until prospectively re-anchored | reject any claim that treats a post-event tick state as known at the anchor | OPEN |
| low-support interactions | frozen 300/150 minimum, distinct-period requirement | mark `INSUFFICIENT_SUPPORT` | OPEN |
| AI over-interpretation | every claim links to deterministic evidence | skeptic downgrades unsupported wording | OPEN |
| confirmation contamination | locked custody | invalidate confirmation status if breached | OPEN |
| infrastructure creep | scope and defect rules | backlog unrelated engineering | OPEN |
