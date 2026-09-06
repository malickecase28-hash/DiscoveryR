# AP-001 Risk Register

From plan §14. Status updated at work-package gates.

| Risk | Control | Contingency | Status |
| --- | --- | --- | --- |
| future information leakage | availability ordering by `received_ts_ns` and `source_sequence` | reject affected evidence and repair the offending join | OPEN — checked in WP1 |
| future conditioning | first-entry cohorts never require later completion | rebuild only the affected cohort | OPEN — checked in WP1 |
| left truncation | mark missing prior lifecycle history | exclude only analyses that require unavailable history | OPEN — partition boundary policy in WP1 |
| right censoring | explicit censor and missing-horizon status | censor-aware analysis where required | OPEN — WP1 declares cutoff policy |
| occupancy duplication | one first-entry anchor per state | keep occupancy for dwell analysis only | OPEN — WP1 anchor sanity |
| temporal dependence | chronological blocks and session-aware matching | block-aware challenge nulls | OPEN — WP5 |
| drift or session confounding | direction, day, session, and regime controls | downgrade, condition, or reject | OPEN — WP5 |
| lineage duplication | dependency map and provenance | collapse dependent context into one information family | OPEN — WP4 |
| low-support interactions | frozen support rule and distinct-period requirements | mark `INSUFFICIENT_SUPPORT` | OPEN — WP2 freezes reference |
| search explosion | semantic context families and controlled candidate promotion | register extra ideas for later work | OPEN — standing |
| scanner performance | WP1 benchmark before full run | optimize the local hot path only | OPEN — WP1 |
| AI over-interpretation | every claim links to deterministic evidence | skeptic downgrades unsupported wording | OPEN — WP5/WP7 |
| confirmation contamination | locked custody; chronological leaf frozen in WP1 | invalidate confirmation status if breached | OPEN — WP1 freezes leaf |
| infrastructure creep | scope and defect rules | backlog unrelated engineering | OPEN — standing |
