# AP-003 Work-Package Status

**PROJECT:** AP-003 BOS/CHOCH, XAUUSD, Program R
**ACTIVE PLAN AUTHORITY:** `AP-003_BOS_CHOCH_PROGRAM_R_RESEARCH_PLAN_V3.md` + the BOS/CHOCH sections of `AP-003_AP-004_FIELD_RECONCILIATION_NOTES.md`
**EXECUTION SCHEDULE RULE:** `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 applies prospectively. Base-plan PERT hour tables are retained as historical planning text but are not executable forecasts for new work.
**HISTORICAL PLAN:** `AP-003_BOS_CHOCH_PROGRAM_R_RESEARCH_PLAN_V1.md`, preserved, superseded by V3 before any work executed against it
**FROZEN INPUT:** manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified against uploaded copies, same identity as AP-001)
**CONFIRMATION STATUS:** LOCKED — no work has executed, confirmation leaf untouched
**PROJECT STATE:** NOT STARTED. No delta gate is needed the way AP-001 needed one: no WP1 evidence exists yet under any prior design, so the amendments above are simply the plan WP1 executes against from the start.

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract, field enumeration, benchmark | NOT STARTED | — | Primary scale selected by the frozen rule; causal ordering and first-occurrence reconstruction pass |
| WP2 Event discovery | NOT STARTED | — | Every primary-scale event class has a result or `INSUFFICIENT_SUPPORT` |
| WP3 Tick conditioning and co-evolution | NOT STARTED | — | Every tick family has a result or an explicit null/insufficient-support/quality-excluded status |
| WP4 Structural and cross-scale conditioning | NOT STARTED | — | Report states which contexts matter, which cross-scale patterns recur, which lack support |
| WP5 Candidate challenge | NOT STARTED | — | Each candidate `SURVIVED_CHALLENGE` / `REJECTED` / `NULL` / `INCONCLUSIVE` |
| WP6 Confirmation | LOCKED (opens once, after WP5) | — | Each survivor `CONFIRMED` / `FAILED_CONFIRMATION` / `INCONCLUSIVE_CONFIRMATION` |
| WP7 Characterization | NOT STARTED | — | Definition of done satisfied |

## WP1 dispatch requirements (from the field-reconciliation notes)

Before WP1 closes, it must additionally record:

1. per-scale event counts feeding the primary-scale selection rule;
2. the `structure_break_invalidated` rate (~3.3% in the full corpus, per-scale rate not yet measured) and how it's recorded per event;
3. the named lineage status of `bos_choch` vs `swings`/`local_structure`, pre-set to `DEPENDENT_SAME_INFORMATION_FAMILY` for any attribute reached through `source_swing_id`, `protected_high_swing_id`, `protected_low_swing_id`, or `protected_reference_swing_id` specifically, per Rule 11.

## Schedule note

New work uses measured throughput or an explicit `NOT_YET_MEASURED` first-checkpoint estimate, named checkpoints, and the frozen deviation rule from `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5. Do not dispatch a new task using an unsupported PERT hour bucket as its operative forecast.

## Standing notes

- Concurrency cap: 2 active workers.
- No behavioral evidence exists yet; nothing here can conflict with AP-001, AP-002, AP-004, AP-005, or AP-006.
