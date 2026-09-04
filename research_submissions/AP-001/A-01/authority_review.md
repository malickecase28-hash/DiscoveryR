# Authority Review — drift_burst lifecycle, availability, and episode identity semantics

- Candidate: `authority_candidate.json` (artifact_type `AUTHORITY_CANDIDATE`)
- Program / role: AP-001 / A-01, phase AUTHORITY, access profile AUTHORITY
- Instrument: XAUUSD · Detector: drift_burst
- Date: 2026-09-04
- Claim prefix: `AP001-A01-C###` (34 claims: 12 AUTHORITATIVE, 2 PROVISIONAL, 20 UNRESOLVED)
- E1 status: **E1_BLOCKED_PENDING_AUTHORITY=true**

## 1. Verdict

The physical layer of the drift_burst output is anchored and authoritative. Everything semantic about its lifecycle — episode identity, state vocabulary, transitions, weak/online/strong, peak, decay and recovery, completion, censoring, timestamp families, and per-stage availability — is **UNRESOLVED**, because no accessible source exposes a single detector-defined field, state term, or rule. The `contracts`, `registry`, and `authority` surfaces are empty, and the payload manifest that would ordinarily document payload internals is referenced but denied. Nothing was inferred from field names; every unresolved matter is recorded as unresolved with evidence of absence.

## 2. Exact source accounting

Files read (4):

| source_id | relative_path | bytes | sha256 (first 12) | role |
|---|---|---|---|---|
| assignment.ap001_a01 | assignment.json | 653 | `3ae786516b29` | Identity gate, access profile, authority-source designation |
| protocol.research_rules | protocol/RESEARCH_RULES.md | 1,908 | `9fad8a804de4` | Program-level causal, lifecycle, exposure-class, blind-work rules |
| xauusd.instrument_config | instruments/XAUUSD/instrument_config.json | 198 | `879378d5df95` | Lake reference; names manifest.json and payload_manifest.json |
| xauusd.schemas | instruments/XAUUSD/source_inventory.json | 33,963 | `62838cf49e4d` | **Designated authority source**: physical schemas, parts, rows, payload columns |

Surfaces observed empty (3): `contracts` (0 files), `registry` (0 files), `authority` (0 files).

Authorized but not available: `agent_harness/assignments/AP-001/A-01.json` (listed in the assignment record; not present among accessible surfaces; not consulted), `fusion_markets/xauusd/manifest.json` and `fusion_markets/xauusd/payload_manifest.json` (referenced by the instrument configuration; raw lake access DENIED).

Corpus accounting (from the inventory, all verified): 8 sources (7 bar scales + tick) × 6 parts each = 48 parts, all schema-verified; rows — 15m: 22,050 · 15s: 1,320,835 · 1h: 5,515 · 1m: 330,458 · 30s: 660,859 · 4h: 1,464 · 5m: 66,147 · tick: 68,445,659 · total 70,852,987. The inventory states no temporal coverage for any scale.

## 3. What is anchored (authoritative findings)

1. **Physical output family (C003).** drift_burst output persists as one per-row nullable serialized-text payload column, `payload_drift_burst` (LargeUtf8, nullable=true), on the **tick scale only**; present in all six schema-verified tick parts; absent from all seven bar scales. Nullability is a schema permission — whether nulls occur and what they denote is unresolved (UQ-21).
2. **Carrier coordinates (C018).** The tick stream exposes exactly two time coordinates, `event_ts_ns` and `received_ts_ns` (Int64, non-nullable), plus sequence key `source_sequence` and quote columns `bid`/`ask`. Names are recorded as physical designations only; no semantic role is assigned from them.
3. **Single-scale fact (C026).** Exactly one of eight corpus scales carries drift_burst; concurrent cross-scale drift_burst measurements do not exist in this corpus, and no bar stream can supply drift_burst phenotype data. Native scale stays attached (rule 3).
4. **Program causal law (C006, C016, C023).** One causal information clock — at anchor time t only information available by t is eligible; occurrence/origin timestamps remain provenance. Lifecycle-first — future completion/termination cannot define an earlier-state cohort; completed may be a later outcome or later anchor. Firewall — a field is FUTURE_RELATIVE_TO_STATE only on direct evidence that its value is unavailable at the state anchor; peak/end/remaining/duration/recovery/available-after naming is never sufficient.
5. **Surface availability (C030, C031).** The instrument configuration references `manifest.json` and `payload_manifest.json`, unreachable under the access profile; contracts/registry/authority are empty. Payload-internal authority is therefore unavailable by design, and this is not evidence that no authority exists elsewhere (C032, provisional, expects revision when surfaces are released).
6. **Compliance (C034).** Only the four files and three empty directories above were consulted; no shared files were modified.

## 4. Lifecycle-stage future-information matrix

Zero detector-defined fields are exposed anywhere (matrix field `exposed_detector_fields: 0`). Empty cells mean *nothing is classifiable*, not that fields were judged future.

| Stage (topics) | Documented | Fields exposed | SAFE_AT_STATE | FUTURE_RELATIVE_TO_STATE | Per-stage availability | Authoritative constraints |
|---|---|---|---|---|---|---|
| birth/origin (3) | No | 0 | — | — | UNRESOLVED | Origin timestamps are provenance (rule 2) |
| weak (6) | No | 0 | — | — | UNRESOLVED | Causal clock bounds availability (rule 2) |
| online (7) | No | 0 | — | — | UNRESOLVED | Causal clock bounds availability (rule 2) |
| strong (8) | No | 0 | — | — | UNRESOLVED | Causal clock bounds availability (rule 2) |
| peak (9) | No | 0 | — | — | UNRESOLVED | Not classifiable retrospective/future from its name (C023); rule 4 |
| decay-risk / dying (10) | No | 0 | — | — | UNRESOLVED | Name-based future classification prohibited (C023) |
| recovery (10) | No | 0 | — | — | UNRESOLVED | Name-based future classification prohibited (C023) |
| completion / termination (11) | No | 0 | — | — | UNRESOLVED | Completed is a later outcome/anchor; cannot define earlier cohorts (rule 4) |
| completed-record / post-terminal (11, 15) | No | 0 | — | — | UNRESOLVED | Retrospective character needs direct evidence (C023); rule 4 |

The only authoritative availability content is program-level: rules 2 and 4 and the firewall rule. All stage-specific timestamps await payload-internal authority (UQ-14, UQ-15).

## 5. Mandated-topic coverage

| # | Topic | Claims | Status |
|---|---|---|---|
| 1 | Physical output family; record variants; state stream vs completed | C003 / C004 | AUTHORITATIVE (physical) / UNRESOLVED (structure) |
| 2 | Episode identity, ID persistence, reuse, completed-record linkage | C005 | UNRESOLVED |
| 3 | Birth/origin and origin timestamp semantics | C006 / C007 | AUTHORITATIVE (origin = provenance) / UNRESOLVED (detector-specific) |
| 4 | Source-supported state vocabulary | C008 | UNRESOLVED |
| 5 | Legal/illegal transitions, reversion, repeat, re-entry, recovery, reset | C009 | UNRESOLVED |
| 6 | Weak semantics | C010 | UNRESOLVED |
| 7 | Online semantics | C011 | UNRESOLVED |
| 8 | Strong semantics | C012 | UNRESOLVED |
| 9 | Peak: contemporaneous vs retrospective | C013 | UNRESOLVED |
| 10 | Decay-risk, dying, recovery: event/state, persistence, re-entry, availability | C014 | UNRESOLVED |
| 11 | Completion, termination, end_reason, incomplete episodes | C015 | UNRESOLVED |
| 12 | Censoring and corpus-boundary representation | C017 | UNRESOLVED |
| 13 | Occurrence/evaluation/milestone/emission-known/market-event/received/completion timestamps | C018 / C019 | AUTHORITATIVE (carrier coordinates) / UNRESOLVED (family mapping) |
| 14 | Exact lawful availability timestamp per stage | C020 | UNRESOLVED |
| 15 | Completed-record retrospective summary fields | C021 | UNRESOLVED |
| 16 | Per-stage field classification (SAFE_AT_STATE / FUTURE_RELATIVE_TO_STATE / UNRESOLVED) | C022 + matrix | UNRESOLVED (all fields) |
| 17 | Direction/sign semantics | C024 | UNRESOLVED |
| 18 | Strength, score, z, efficiency semantics | C025 | UNRESOLVED |
| 19 | Multiscale: concurrent measurements, internal computation | C026 / C027 | AUTHORITATIVE (single-scale corpus fact) / UNRESOLVED (internal) |
| 20 | Exact episode segmentation rule for a future E1 scanner | C028 | UNRESOLVED |

Program-level claims: C001 (identity/access), C002 (authority source mapping), C023 (firewall), C029 (corpus accounting), C030 (manifest references and denial), C031 (empty authority surfaces), C032 (provisional release expectation), C033 (provisional two-clock structural constraint), C034 (blind-work compliance and E1 block).

## 6. Firewall application

The general causal rule is treated as authoritative and the exact detector-field mapping as unresolved, per the assignment mandate. No field — and specifically no peak, end, remaining, duration, recovery, or available-after concept — was classified FUTURE_RELATIVE_TO_STATE from its name, and none was classified SAFE_AT_STATE either: there are no exposed fields to classify. The prohibition is derived from rules 2 and 4 (a name cannot establish what information was available at an anchor time), not from convention.

## 7. Unresolved questions

23 questions are registered (UQ-01 … UQ-23) in the candidate, spanning record structure (UQ-01/02/21), episode identity (UQ-03), birth (UQ-04), vocabulary (UQ-05), transitions (UQ-06), weak/online/strong (UQ-07/08/09), peak (UQ-10), decay/recovery (UQ-11), completion (UQ-12), censoring and corpus coverage (UQ-13/22), timestamp families (UQ-14), per-stage availability (UQ-15), retrospective completed-record fields (UQ-16), direction/sign (UQ-17), strength/score/z/efficiency (UQ-18), internal multiscale (UQ-19), E1 segmentation (UQ-20), and derivation/confluence relationships among sibling payloads (UQ-23, under rule 11).

Resolution path: the referenced-but-denied `payload_manifest.json` at `fusion_markets/xauusd/` is the natural carrier of payload-internal schema; the empty contracts/registry/authority surfaces are the natural carriers of state vocabulary and transition rules. Release of either would trigger revision of this candidate.

## 8. E1 status

**E1 phenotype work on drift_burst is blocked pending authority** (E1_BLOCKED_PENDING_AUTHORITY=true). A future E1 scanner cannot be specified: the episode segmentation rule (topic 20) is undetermined, no state vocabulary exists to recognize, and no availability timestamps exist to enforce the causal clock. Per rule 5 this is an authority-availability block, not a class-progression gate; per rule 1 the lifecycle, not any timeframe stream, is the research unit when E1 opens.

## 9. Validation record

- Candidate parses as JSON; required keys present (`program_id`, `role_id`, `artifact_type=AUTHORITY_CANDIDATE`, `subjects`, `claims`, `unresolved_questions`, `sources_consulted`).
- 34 claims, IDs `AP001-A01-C001` … `AP001-A01-C034`, unique and prefix-conformant; statuses restricted to AUTHORITATIVE / PROVISIONAL / UNRESOLVED.
- Every claim carries ≥1 structured `authority_evidence` object with `source_id`, `relative_path`, `locator`, `evidence_type`, `supports`.
- All 20 mandated topics covered; difficult unresolved topics included explicitly.
- Forbidden identity terms scanned (case-insensitive) in both artifacts: none present.
- Exactly two files exist in the assigned workspace: `authority_candidate.json` and `authority_review.md`.
