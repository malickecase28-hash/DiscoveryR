# AP-004 Risk Register

From `AP-004_ORDER_BLOCKS_PROGRAM_R_RESEARCH_PLAN_V3.md` §16, plus one row added by field reconciliation. None are checked yet since no work package has started.

| Risk | Control | Contingency | Status |
| --- | --- | --- | --- |
| unverified payload fields | WP1 enumerates actual fields before design commits further | drop assumed measurements that turn out not to exist | OPEN — WP1 not started |
| **MITIGATED assumed as a terminal state that does not exist** *(closed by field reconciliation, kept for record)* | market terminals corrected to `INVALIDATED` / `CANCELLED`; censoring kept as observation status | reject any table or contract that still writes `MITIGATED` as an Order-Block terminal state | CLOSED — corrected pre-execution |
| primary-scale mis-selection | frozen rule in Section 5 | reselect only through a change decision | OPEN |
| pooling formation and touch context | `FORMED` and `FIRST_TOUCH` measured and reported as separate anchors | reject any table that merges them | OPEN |
| future information leakage | use each lifecycle event's verified lawful availability coordinate; `bar_close_ts` only if WP1 proves it is the availability clock | reject affected evidence and repair only the offending join | OPEN |
| left-truncated open zones | `LEFT_TRUNCATED_FORMATION` flag, separate population | exclude from formation-anchored cohort only | OPEN |
| right censoring mistaken for lifecycle termination | separate `RIGHT_CENSORED_UNTOUCHED` / `RIGHT_CENSORED_TOUCHED` from `INVALIDATED` / `CANCELLED` market terminals | reject any table treating censoring as a market terminal event or forward anchor | OPEN |
| zone-family redundancy (`fvg`, `range`, `order_blocks` overlap) | Rule 11 dependency check, Control C | collapse dependent context into one information family | OPEN |
| search explosion across seven scales and multiple zone families | one primary scale first, others as nesting context only | register cross-family ideas for later work, not parallel studies | OPEN |
| cross-resolution search explosion | fixed bridge horizons only at the primary discovery scale | do not add arbitrary wall-clock grids or cross every tick field with every zone field | OPEN |
| post-anchor tick-state leakage | later tick transitions are outcomes until prospectively re-anchored | reject any claim that treats a post-anchor tick state as known at `FORMED` or `FIRST_TOUCH` | OPEN |
| low-support interactions | frozen 300/150 minimum, distinct-period requirement | mark `INSUFFICIENT_SUPPORT` | OPEN |
| AI over-interpretation | every claim links to deterministic evidence | skeptic downgrades unsupported wording | OPEN |
| confirmation contamination | locked custody | invalidate confirmation status if breached | OPEN |
| infrastructure creep | scope and defect rules | backlog unrelated engineering | OPEN |
