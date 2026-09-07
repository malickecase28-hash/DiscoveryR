# AP-004 Work-Package Status

**PROJECT:** AP-004 Order Blocks, XAUUSD, Program R
**ACTIVE PLAN AUTHORITY:** `AP-004_ORDER_BLOCKS_PROGRAM_R_RESEARCH_PLAN_V3.md` + the Order Blocks sections of `AP-003_AP-004_FIELD_RECONCILIATION_NOTES.md`
**EXECUTION SCHEDULE RULE:** `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 applies prospectively. Base-plan PERT hour tables are retained as historical planning text but are not executable forecasts for new work.
**HISTORICAL PLAN:** `AP-004_ORDER_BLOCKS_PROGRAM_R_RESEARCH_PLAN_V1.md`, preserved, superseded by V3 before any work executed against it
**FROZEN INPUT:** manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified, same identity as AP-001)
**CONFIRMATION STATUS:** LOCKED — no work has executed, confirmation leaf untouched
**PROJECT STATE:** NOT STARTED. No delta gate needed: no WP1 evidence exists under any prior design.

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract, field enumeration, benchmark | NOT STARTED | — | Primary scale selected by the frozen rule; causal ordering and formation reconstruction pass |
| WP2 Lifecycle discovery | NOT STARTED | — | Every lawful anchor and observed market-terminal class has a result or `INSUFFICIENT_SUPPORT`; censored populations are reported separately |
| WP3 Tick conditioning and co-evolution | NOT STARTED | — | Every tick family has a result or an explicit null/insufficient-support/quality-excluded status |
| WP4 Structural conditioning and nesting | NOT STARTED | — | Report states which contexts and overlaps matter and which lack support |
| WP5 Candidate challenge | NOT STARTED | — | Each candidate `SURVIVED_CHALLENGE` / `REJECTED` / `NULL` / `INCONCLUSIVE` |
| WP6 Confirmation | LOCKED (opens once, after WP5) | — | Each survivor `CONFIRMED` / `FAILED_CONFIRMATION` / `INCONCLUSIVE_CONFIRMATION` |
| WP7 Characterization | NOT STARTED | — | Definition of done satisfied |

## WP1 dispatch requirements (from the field-reconciliation notes)

Before WP1 closes, it must additionally record:

1. market lifecycle corrected to `FORMED -> [FIRST_TOUCH if observed] -> {INVALIDATED | CANCELLED}`; there is no `MITIGATED` market state, and censoring remains a separate observation status (`RIGHT_CENSORED_UNTOUCHED` / `RIGHT_CENSORED_TOUCHED`);
2. `CANCELLED` (1,304 occurrences in the full corpus) tracked as its own terminal class, checked against the support floor before any table folds it into `INVALIDATED`;
3. whether invalidation/cancellation can occur without a prior recorded touch (formation count is roughly double the touch count in the full corpus; this needs confirming, not assuming, before the FORMED->FIRST_TOUCH->terminal chain is treated as guaranteed-sequential).

## Schedule note

New work uses measured throughput or an explicit `NOT_YET_MEASURED` first-checkpoint estimate, named checkpoints, and the frozen deviation rule from `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5. Do not dispatch a new task using an unsupported PERT hour bucket as its operative forecast.

## Standing notes

- Concurrency cap: 2 active workers.
- No behavioral evidence exists yet; nothing here can conflict with AP-001, AP-002, AP-003, AP-005, or AP-006.
