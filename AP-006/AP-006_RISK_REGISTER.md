# AP-006 Risk Register

From `AP-006_FEED_HEALTH_PROGRAM_R_RESEARCH_PLAN_V2.md` §16, with the episode-continuation risk replaced by a firing co-occurrence risk per field reconciliation. None are checked yet since no work package has started.

| Risk | Control | Contingency | Status |
| --- | --- | --- | --- |
| **wrong detector shape assumed (degradation episode instead of independent anomaly flags)** *(closed by field reconciliation, kept for record)* | real schema enumerated before WP1 design finalized; episode framing replaced with per-firing framing | none needed further; corrected before any code ran | CLOSED — corrected pre-execution |
| unverified payload fields | WP1 enumerates actual fields before design commits further | drop assumed measurements that turn out not to exist | OPEN — WP1 not started |
| **firing co-occurrence window undefined** *(added by field reconciliation, replaces the 5-second episode-continuation rule)* | WP1 freezes the window and justifies it the same way the support floor was justified | if types don't cleanly group, report per-type only, do not force a combined incident count | OPEN |
| inferring data truth from market volatility | Section 8 classifies market context, not whether the underlying price move is genuine/fake | do not make artifact-truth claims without independent ground truth | OPEN |
| **contamination tests depend on AP-005 and AP-003 WP1/WP2, neither of which has run** | explicit blocking dependency recorded in the work-package status | WP4 blocks and escalates until both prerequisites exist; no private re-derivation | OPEN — explicit blocker |
| future information leakage | availability by `received_ts_ns`/`source_sequence` only | reject affected evidence, repair the join | OPEN |
| left-truncated firing history | flag firings with no observed prior state | exclude only analyses requiring unavailable history | OPEN |
| right-boundary horizon censoring | anomaly firings are point events; only downstream measurement windows can be censored at the DEVELOPMENT boundary | mark affected outcomes missing/censored; do not invent a feed-health lifecycle state | OPEN |
| contamination exposure window misdefined | bind exposure to the detector subject's causal input interval, not anchor timestamp alone | downgrade to `INCONCLUSIVE` if the relevant source interval cannot be identified | OPEN |
| cross-resolution overgeneralization | separate tick-domain and bar-domain contamination decisions | never infer a program-wide exclusion from one domain alone | OPEN |
| recommendation treated as automatic amendment of other projects | explicit scope exclusion in Section 2 | each affected project needs its own change decision to act on the recommendation | OPEN |
| low-support interactions | frozen support minimum, distinct-period requirement | mark `INSUFFICIENT_SUPPORT` | OPEN |
| AI over-interpretation | every claim links to deterministic evidence | skeptic downgrades unsupported wording | OPEN |
| confirmation contamination | locked custody | invalidate confirmation status if breached | OPEN |
| infrastructure creep | scope and defect rules | backlog unrelated engineering | OPEN |
