# AP-001 Work-Package Status

**PROJECT:** AP-001 Drift-Burst, XAUUSD, Program R
**PLAN:** `AP-001_PLAN_FROZEN.md` (sha256 `012fc0d676fe0092f10110b03a25c37fbe896198e136667bbfef7637c1676193`), APPROVED
**FROZEN INPUT:** `analytical_lake/fusion_markets/xauusd/` — manifest.json SHA256 `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`, payload_manifest.json SHA256 `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed` (verified, sizes match plan)
**CONFIRMATION STATUS:** LOCKED — confirmation leaf untouched
**PROJECT STATE:** STOPPED at user decision, 2026-09-06, after WP1 completion. Resume point: WP2.

## Current status record

```text
PROJECT: AP-001 Drift-Burst Program R
WORK_PACKAGE: WP1 Contract and benchmark
STATUS: COMPLETE (closed 2026-09-06)
OWNER: Rust toolsmith (agent) / research director
STARTED: 2026-09-06
FORECAST: 0.75 h most likely / 1.5 h pessimistic — actual ~3.5 h active (3 scan runs; two director decisions D-005/D-006); overrun noted, within project reforecast threshold
DELIVERABLE: Verified input identities, anchor sanity, rows/s, memory, full-run ETA
EVIDENCE_ID: AP-001-WP1-CONTRACT (R1_CONTRACT/AP-001_WP1_PREFLIGHT.json, content sha256 dbea294d38f67751f6e797d5c9a11501731589719f623df756fcfebde5d83248)
EXIT_CONDITION: Causal ordering and first-entry reconstruction pass — SATISFIED
CURRENT_RESULT: G1 PASS (identities exact); G2 PASS (43,007,489 dev-leaf rows, 0 ordering regressions, 0 received<event); G3 PASS (117,696 milestone comparisons, 0 mismatches under D-006); G4 PASS (118,673 rows/s, 362.4 s dev-leaf scan, RSS 0.11 GiB, full-tape ETA ~9.6 min)
BLOCKER: none (project stopped by user, not by defect)
NEXT_ACTION: none while stopped. Resume = dispatch WP2 per frozen contract.
SCOPE_CHANGE: D-005/D-006 anchor mechanics; forming-phase crossings registered as backlog idea (out of scope)
CONFIRMATION_STATUS: LOCKED
```

## Work-package ledger

| WP | Status | Evidence | Exit |
| --- | --- | --- | --- |
| WP1 Contract and benchmark | **COMPLETE** | R1_CONTRACT preflight + report, run 3 | Causal ordering and first-entry reconstruction PASS |
| WP2 Lifecycle and market discovery | STOPPED (contract frozen, not started) | R2 contract only | Every anchor has a result or INSUFFICIENT_SUPPORT |
| WP3 Same-domain tick context | STOPPED (contract frozen, not started) | R3 contract only | Tick family results or explicit null/insufficient/quality-excluded |
| WP4 Structural conditioning | STOPPED (contract frozen, not started) | R4 contract only | Structural report states what matters / recurs / lacks support |
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

## Standing notes

- AP-002 E2 remains STOPPED. The AP-002 E2 execution-design decision record is PROPOSED, not approved. No AP-002 work resumes in this project.
- Concurrency cap: 2 active workers (1 used throughout WP1).
- Confirmation partition frozen in WP1 contract; untouched until WP6 opens it once, after candidate freeze.
- AP-002 E1 completed evidence is not modified by this project.
