# SUPERSEDED: SCOPE BOUNDARY MISALIGNMENT

This evidence package (`logical_result_hash 735bd7ab06ae2caf72e37447d207dec62c17932ee1684c4c393d8ace07684c52`)
is **superseded** as of 2026-09-06.

Cause: it was generated from a development view materialized with an effective
start boundary of 2025-07-31T16:00:00Z, violating the frozen
`XAUUSD_DATA_SCOPE_V1` `development.start_utc_inclusive` of 2025-07-31T16:15:00Z
(29 pre-boundary formations in the cohort; pre-boundary preflight stream).

The corrected, active evidence lives in
[`docs/research/ap-002-e1a-scope-repair/`](../../ap-002-e1a-scope-repair/AP-002_E1A_SYNTHESIS.md)
(`logical_result_hash 1b701c016bd1effa300742ee74ce73d6e0e96313d62847d231391061dc94dca6`).

The reviews, reconciliation, method challenge and anomaly audit in this directory
remain valid historical records of the challenge process: every reviewed quantity
and classification is unchanged on the corrected cohort except formation counts
(−29) and orphan counts (+1/+2). See
`../ap-002-e1a-scope-repair/AP-002_E1A_SCOPE_REPAIR_IMPACT_ASSESSMENT.md`.

This package is preserved, not deleted.
