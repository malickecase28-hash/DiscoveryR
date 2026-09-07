# AP-005 Work-Package Status

**PROJECT:** AP-005 Quote Pressure, XAUUSD, Program R
**ACTIVE PLAN AUTHORITY:** `AP-005_QUOTE_PRESSURE_PROGRAM_R_RESEARCH_PLAN_V2.md` + `AP-005_FIELD_RECONCILIATION_AMENDMENT.md` (supersedes the regime/state-machine framing in Sections 3, 5-8)
**EXECUTION SCHEDULE RULE:** `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 applies prospectively. Base-plan PERT hour tables are retained as historical planning text but are not executable forecasts for new work.
**HISTORICAL PLAN:** `AP-005_QUOTE_PRESSURE_PROGRAM_R_RESEARCH_PLAN_V1.md`, preserved, superseded before any work executed against it
**FROZEN INPUT:** manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified, same identity as AP-001)
**CONFIRMATION STATUS:** LOCKED — no work has executed, confirmation leaf untouched
**PROJECT STATE:** NOT STARTED. No delta gate needed: no WP1 evidence exists under the original (incorrect) regime-based design, so nothing needs reconciling against prior results.

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract, field enumeration, benchmark | NOT STARTED | — | Paired-firing definition frozen (not a discretized regime); causal ordering and first-firing reconstruction pass |
| WP2 Firing discovery and resolution bridge | NOT STARTED | — | Every firing class has a native tick-time result and bounded bar-bridge result, or `INSUFFICIENT_SUPPORT` |
| WP3 Cross-tick-detector conditioning, Drift-Burst redundancy test | NOT STARTED | — | Every tick family, including the Drift-Burst redundancy question, has a result or explicit null/insufficient-support/quality-excluded status |
| WP4 Structural conditioning, co-evolution, and bridge interpretation | NOT STARTED | — | Report states which contexts matter, which cross-resolution sequences recur, and which later structural events remain retrospective or are prospectively re-anchored |
| WP5 Candidate challenge | NOT STARTED | — | Each candidate `SURVIVED_CHALLENGE` / `REJECTED` / `NULL` / `INCONCLUSIVE` |
| WP6 Confirmation | LOCKED (opens once, after WP5) | — | Each survivor `CONFIRMED` / `FAILED_CONFIRMATION` / `INCONCLUSIVE_CONFIRMATION` |
| WP7 Characterization | NOT STARTED | — | Definition of done satisfied |

## WP1 dispatch requirements (from the field-reconciliation amendment)

Before WP1 closes, it must additionally freeze:

1. detector role corrected from `STATE_REGIME` to paired `EVENT` (`buy_quote_pressure` / `sell_quote_pressure`); the discretization step in the original design is deleted, not deferred;
2. whether the two signals ever fire on the exact same tick, and the co-occurrence window used to call two firings co-occurring versus sequential versus isolated, justified the same way the 300/150 support floor was, not just asserted;
3. per-signal firing counts in the development leaf, feeding the 300/150 support rule.

## Schedule note

New work uses measured throughput or an explicit `NOT_YET_MEASURED` first-checkpoint estimate, named checkpoints, and the frozen deviation rule from `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5. Do not dispatch a new task using an unsupported PERT hour bucket as its operative forecast.

## Standing notes

- Concurrency cap: 2 active workers.
- Section 8's explicit Drift-Burst redundancy test (Control C) is the actual point of this project; it is unaffected by the field-reconciliation amendment and remains required regardless of which detector shape the anchor turned out to be.
- No behavioral evidence exists yet; nothing here can conflict with AP-001, AP-002, AP-003, AP-004, or AP-006.
