# AP-002 E1A Scope-Repair Impact Assessment

Recorded 2026-09-06 under the director ruling **"Fix view + rerun, then freeze"**.

## 1. Cause

The accepted AP-002 E1A evidence (logical result hash
`735bd7ab06ae2caf72e37447d207dec62c17932ee1684c4c393d8ace07684c52`) was generated
from a development view whose **effective start boundary was 2025-07-31T16:00:00Z
(exclusive)**, while the frozen `XAUUSD_DATA_SCOPE_V1` sets
`development.start_utc_inclusive = 2025-07-31T16:15:00Z`. The scope file was never
amended (Git history); the defect was in the view builder, which implemented only
the end-exclusive filter (`development_view.rs` carried `BOUNDARY_MS` = the
development end and no start gate). The lake's own bar data begins 16:00:00Z and
its tick segment extends to 2025-07-08, so the misaligned window was inherited
silently by the sidecar, preflight, and scanner.

Discovered during the freeze-order re-evaluation of anomaly A2 against the exact
frozen boundary; recorded in
`docs/research/ap-002-e1-causal-rework/challenge/AP-002_E1_ANOMALY_AUDIT.json#development_boundary_conformance_finding`.

## 2. Repair chain (all committed on `research/ap-002-e1-native-strata`)

| Commit | Change |
| --- | --- |
| `022f1a4` | Enforce frozen development start boundary in the scope view (content-preserving gate: bars `bar_open_ts >= 16:15:00Z && bar_close_ts < end`; ticks both clocks `>= start && < end`) |
| `2e98454` / `2fca3b1` | Materializer identity/source via CLI arguments (execution sandbox strips child-process environment; provenance semantics unchanged — identity still validated as a Git SHA and recorded) |
| `7fce3be` | Footer-statistics classification (provably in/out parts need zero row reads), batch-level gate-column resolution, parallel per-file verification/materialization; full row-scan verify pass retained |
| `763f453` | Bounded filtered-part writer memory (row-group size; default buffered multi-GB on wide payload parts) |
| `5d3f0a2` | Scope-repair audit tool (canonical payload parser; fail-closed reconciliation against scanner counters) |

## 3. Repaired inputs

- Development view re-materialized and internally verified; logical view identity
  `943c845d0fa8f2d8bcc1c0fffeddc84863ceb1bf1ec6e2cc66c16072794c092f`.
  First tick 2025-07-31T16:15:00Z; first bar closes 16:15:15 (15s) through
  2025-08-01T00:00 (4h); tick rows 51,969,691 (was 54,415,940).
- Superseded view preserved revoked at
  `F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development-superseded-16x00`.
- Sidecar rebuilt: `authority/fvg_availability_v2_scope_repair.jsonl`
  (1,922,771 rows, sha256 `8882dfca1e34aab71df411ee0837a7203b01462e3ada565119e66e48c4e15929`).

## 4. Corrected evidence

Two independent fresh scanner executions (`scope-repair-run-1/2`), measurement
code byte-identical to the accepted run (`b632a92`; only `development_view.rs`
differs):

- `logical_result_hash = 1b701c016bd1effa300742ee74ce73d6e0e96313d62847d231391061dc94dca6`
  — identical across both runs; preflight 51,969,691 rows, 0 regressions both runs.
- Zones: **381,498** lawful formations; integrity gate **PASS** on both runs
  (9/9 checks).
- See `REPRODUCIBILITY.json`, `ARTIFACT_SHA256.json`.

## 5. Impact vs the superseded result (old → new)

| Quantity | Old (superseded) | New (corrected) |
| --- | --- | --- |
| Formations | 381,527 | 381,498 (−29: 15s −16, 30s −8, 1m −5; the pre-boundary cohort, including one zone detected exactly at the 16:15:00.000 boundary instant on the boundary bar itself) |
| A1 fill-without-prior-touch | 1,010 (682/213/81/20/10/4/0) | **identical per stratum** — none of the removed zones were gap-through fills |
| Orphan lifecycle events | 177 touch / 356 fill | 178 / 358 (+1/+2: producer-state events for the removed boundary-bar zones, correctly left-truncated) |
| Fill / censoring rates | — | unchanged to 4 decimal places per stratum |
| Bullish formation share | — | unchanged to 4 decimal places (15s 0.5103→0.5104 rounding only) |
| Lifecycle medians | touch 2 native bars; fill 7/7/7/7/8/8/5 bars | **identical** |
| Prospective signs | raw 21/21 positive; adjusted 18 neg / 1 zero / 2 pos | **identical** |

Every synthesis conclusion survives unchanged. The 29 removed formations are
0.0076% of the cohort.

## 6. A1 / A2 against the corrected cohort (canonical)

- **A1** `RESOLVED_EXPECTED_SEMANTIC_CASE` — 1,010 in-window formed zones filled
  without an emitted first-touch event because the linked producer tests far-edge
  fill independently of overlap (gap-through/far-edge fill). Boundary-independent;
  unchanged by the repair.
- **A2** `LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE` — 536 orphan lifecycle events
  (178 touch / 358 fill). Every orphan zone was detected strictly before its
  stratum's first lawful in-window formation (orphan detection window
  2025-07-22 … 2025-07-31T16:15:00Z; two events sit exactly at the boundary
  instant). All remain excluded from in-window formed-zone cohorts and are not
  missing formations. Evidence: `AP-002_E1A_SCOPE_REPAIR_AUDIT.json`
  (fail-closed reconciliation with scanner counters).

## 7. Tooling decision

The audit layer was benchmarked per director challenge: Rust
`ap002_e1a_scope_repair_audit` **5.7 s** vs a functionally identical Python script
**113.1 s** (identical outputs, event-level equivalence verified). The audit layer
is Rust from now on; Python remains ad-hoc interactive forensics only and produces
no freeze-cited evidence.

## 8. Supersession

```text
4562b87b…  SUPERSEDED_CAUSAL_RESPONSE_AND_TAIL_AVAILABILITY
735bd7ab…  SUPERSEDED_SCOPE_BOUNDARY_MISALIGNMENT   (this repair)
1b701c01…  ACTIVE AP-002 E1A EVIDENCE
```

Note: the scanner-generated `RUN_MANIFEST.json` of the corrected runs carries the
prior generation's hardcoded `supersedes` field; the artifact is preserved
byte-faithful and the canonical supersession chain is this document and the
synthesis.
