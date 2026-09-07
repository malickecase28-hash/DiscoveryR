# AP-005 Risk Register

From `AP-005_QUOTE_PRESSURE_PROGRAM_R_RESEARCH_PLAN_V2.md` §16, with the discretization risk replaced by the co-occurrence-window risk per field reconciliation. None are checked yet since no work package has started.

| Risk | Control | Contingency | Status |
| --- | --- | --- | --- |
| **wrong detector shape assumed (state-regime instead of paired event)** *(closed by field reconciliation, kept for record)* | real schema enumerated before WP1 design finalized; regime framing replaced with paired-firing framing | none needed further; corrected before any code ran | CLOSED — corrected pre-execution |
| **undefined firing co-occurrence window** *(added by field reconciliation, replaces "undefined regime discretization")* | WP1 freezes the co-occurrence window and justifies it the same way the support floor was justified | if no clean window separates co-occurring from sequential firings, report as `AMBIGUOUS`, do not force a bucket | OPEN — WP1 not started |
| result is fully explained by Drift-Burst | explicit Control C redundancy test in WP3 | report as `NULL` incremental information; still valid Program R knowledge | OPEN |
| future information leakage | availability by `received_ts_ns`/`source_sequence` only | reject affected evidence, repair the join | OPEN |
| left-boundary antecedent context | flag firing anchors whose required antecedent window extends before DEVELOPMENT | exclude only analyses requiring unavailable pre-anchor history | OPEN |
| right-boundary horizon censoring | mark forward tick/bar outcomes missing when the lawful DEVELOPMENT window ends before the horizon | retain the firing anchor; do not invent a detector censoring state | OPEN |
| implicit amendment of AP-001 | cross-project rule from the execution skill | any AP-001 change requires its own decision record, never inherited from this plan | OPEN |
| lineage duplication with Drift-Burst | Rule 11 check, Control C | collapse into one information family if fully dependent | OPEN |
| search explosion across six other tick families and structural families | context families bounded to those already defined in AP-001 | register extra ideas for later work | OPEN |
| cross-resolution search explosion | fixed bar scales and bridge only; challenge only promoted relationships | do not add arbitrary bar horizons or cross every bar detector field with every pressure field | OPEN |
| future structural-event leakage | post-anchor bar events are outcomes until prospectively re-anchored | reject any claim that treats a later structural event as known at the pressure anchor | OPEN |
| low-support interactions | frozen support minimum, distinct-period requirement | mark `INSUFFICIENT_SUPPORT` | OPEN |
| AI over-interpretation | every claim links to deterministic evidence | skeptic downgrades unsupported wording | OPEN |
| confirmation contamination | locked custody | invalidate confirmation status if breached | OPEN |
| infrastructure creep | scope and defect rules | backlog unrelated engineering | OPEN |
