# AP-001 Work-Package Status

**PROJECT:** AP-001 Drift-Burst, XAUUSD, Program R
**ACTIVE PLAN AUTHORITY:** `AP-001_DRIFT_BURST_PROGRAM_R_RESEARCH_PLAN_V3.md` + `AP-001_V3_FINAL_HARDENING_AMENDMENT.md` (D-008)
**HISTORICAL PLAN:** `AP-001_PLAN_FROZEN.md` (sha256 `012fc0d676fe0092f10110b03a25c37fbe896198e136667bbfef7637c1676193`), preserved as the plan under which WP1 originally executed
**FROZEN INPUT:** `analytical_lake/fusion_markets/xauusd/` — manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified, sizes match plan)
**CONFIRMATION STATUS:** LOCKED — confirmation leaf untouched
**PROJECT STATE:** STOPPED at user decision after WP1 completion. Resume point is the D-008 V3 delta gate, not behavioral WP2 execution.

## Current status record

```text
PROJECT: AP-001 Drift-Burst Program R
WORK_PACKAGE: WP1 Contract and benchmark
STATUS: COMPLETE (closed 2026-09-06)
OWNER: Rust toolsmith (agent) / research director
STARTED: 2026-09-06
FORECAST: 0.75 h most likely / 1.5 h pessimistic — actual ~3.5 h active (3 scan runs; two director decisions D-005/D-006); overrun noted
DELIVERABLE: Verified input identities, anchor sanity, rows/s, memory, full-run ETA
EVIDENCE_ID: AP-001-WP1-CONTRACT (R1_CONTRACT/AP-001_WP1_PREFLIGHT.json, content sha256 dbea294d38f67751f6e797d5c9a11501731589719f623df756fcfebde5d83248)
EXIT_CONDITION: Causal ordering and first-entry reconstruction pass — SATISFIED
CURRENT_RESULT: G1 PASS (identities exact); G2 PASS (43,007,489 dev-leaf rows, 0 ordering regressions, 0 received<event); G3 PASS (117,696 milestone comparisons, 0 mismatches under D-006); G4 PASS (118,673 rows/s, 362.4 s dev-leaf scan, RSS 0.11 GiB, full-tape ETA ~9.6 min)
BLOCKER: project is intentionally stopped; pre-V3 R2-R4 contracts require D-008 reconciliation before use
NEXT_ACTION: on explicit resume, run the narrow V3 delta gate: bar/structural availability bindings, first-fully-post-anchor bar alignment, boundary/warm-state reconciliation, 300/150 support rule insertion, named same-domain lineage requirements; then re-freeze R2-R4 contracts
SCOPE_CHANGE: D-005/D-006 anchor mechanics; D-008 prospective V3 + final-hardening amendment; forming-phase crossings remain backlog only
CONFIRMATION_STATUS: LOCKED
```

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract and benchmark | **COMPLETE** | R1_CONTRACT preflight + report, run 3 | Causal ordering and first-entry reconstruction PASS |
| D-008 V3 delta gate | REQUIRED BEFORE WP2, NOT STARTED | V3 + final-hardening amendment | New prospective requirements bound to existing evidence; R2-R4 contracts reconciled and re-frozen |
| WP2 Lifecycle, tick-market, and bar-bridge discovery | STOPPED; pre-V3 contract **NOT EXECUTABLE** | R2 contract exists but requires D-008 re-freeze | Every anchor has native tick and bar-bridge result or INSUFFICIENT_SUPPORT |
| WP3 Same-domain conditioning and co-evolution | STOPPED; pre-V3 contract **NOT EXECUTABLE** | R3 contract exists but requires D-008 re-freeze | Tick family results, named lineage status, or explicit null/insufficient/quality-excluded |
| WP4 Structural conditioning and bridge interpretation | STOPPED; pre-V3 contract **NOT EXECUTABLE** | R4 contract exists but requires D-008 re-freeze | Structural and cross-resolution report states what matters / recurs / lacks support |
| WP5 Candidate challenge | NOT STARTED | — | Each candidate SURVIVED_CHALLENGE / REJECTED / NULL / INCONCLUSIVE |
| WP6 Confirmation | LOCKED (opens once, after WP5) | — | Each survivor CONFIRMED / FAILED_CONFIRMATION / INCONCLUSIVE_CONFIRMATION |
| WP7 Characterization | NOT STARTED | — | Definition of done satisfied |

## WP1 key facts for resumption

- Development leaf: 43,007,489 ticks (2025-07-09 → 2026-03-14 event time, part 3 stopped at cutoff row 7,659,664). Confirmation leaf (parts 4-5 + tail of part 3) untouched, footer-only.
- Producer lifecycle reproduced exactly: 24,253 completed episodes in the development leaf; first-entry anchors WEAK 24,220 / ONLINE 24,196 / STRONG 24,253 / DECAY_RISK 23,427 / DYING 22,600; 471,037 forming attempts ended without arming (not anchors — backlog idea FORMING_PHASE_CROSSINGS); 1 left-truncated episode; 0 right-censored.
- Anchor mechanics: D-006 milestone candidate capture (candidate recorded at milestone crossing tick, replaced on forming-window restart, becomes the anchor at arm). G3 cross-check vs emitted lifecycle records: 0 mismatches.
- Known artifact note: WP1 report/preflight amendment metadata is mislabeled "D-005" — the implemented and verified mechanics are D-006 (see decision log D-007). Numbers are correct; frozen artifact is annotated, not edited.
- Benchmark: 118,673 rows/s with WP2 projection (drift_burst + ts + seq + bid/ask), batch 16,384, RSS 0.11 GiB. WP3+ scans add six more payload columns — expect lower throughput; re-benchmark at WP3 start.
- Scanner: `crates/research-tape/src/ap001_drift_burst.rs` + `src/bin/ap001_wp1_contract.rs` (26 unit tests passing).

## D-008 requirements for resumption

The delta gate is a reconciliation task, not a new infrastructure phase and not a full WP1 rerun.

It must close only these prospective gaps before WP2:

1. bind lawful availability for bar/structural context used by the tick->bar bridge;
2. freeze the first-fully-post-anchor bar alignment rule;
3. show whether V3 boundary/warm-state requirements are already satisfied by WP1 evidence, repairing only a demonstrated gap;
4. insert the AP-001 support rule: 300 effective observations; 150-299 only as `LOW_SUPPORT_FALLBACK`; below 150 `INSUFFICIENT_SUPPORT`;
5. require named Rule 11 lineage adjudication for Drift-Burst vs `micro_volatility`, `quote_arrival`, `quote_dynamics`, `quote_pressure`, `spread_state`, and `feed_health`;
6. re-freeze R2, R3, and R4 contracts against V3 + amendment before dispatch.

## Standing notes

- AP-002 E2 remains STOPPED. The AP-002 E2 execution-design decision record is PROPOSED, not approved. No AP-002 work resumes in this project.
- Concurrency cap: 2 active workers.
- Confirmation partition frozen in WP1 contract; untouched until WP6 opens it once, after candidate freeze.
- AP-002 E1 completed evidence is not modified by this project.
- If AP-006 later confirms a material `feed_health` contamination effect, that triggers a separate AP-001 change decision. It does not automatically rerun or invalidate AP-001 evidence.
