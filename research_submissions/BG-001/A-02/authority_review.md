# BG-001 / A-02 Authority Review — bar lineage

| field | value |
|---|---|
| program | BG-001 |
| role | A-02 (phase AUTHORITY, access profile AUTHORITY) |
| detector | bar_lineage |
| claim prefix | `BG001-A02-C###` |
| subjects | raw3, range, fvg, bos_choch, local_structure, order_blocks, micro_liquidity, micro_liquidity_context |
| artifacts | `authority_candidate.json`, `authority_review.md` (this file) |

---

## 1. Scope and method

Blind lineage-authority research only. No behavioral confluence research, profitability
research, frequency counts, co-occurrence analysis, raw-lake access, network access, peer
inspection, or shared-file edits were performed.

Surfaces actually read:

- `protocol/RESEARCH_RULES.md` — program research rules.
- `instruments/XAUUSD/instrument_config.json` — lake references and manifest names.
- `instruments/XAUUSD/source_inventory.json` — the declared authority source (`xauusd.schemas`).
- `assignment.json` — role authorization and denial profile.

Surfaces inspected and found **empty** (a load-bearing negative finding, recorded as
`ABSENCE_OF_ARTIFACT` evidence on every unresolved claim): `contracts/`, `registry/`,
`authority/`. There are no detector contracts, no registry entries, and no approved
authority artifacts on any authorized surface.

Raw-lake access and peer visibility are denied by the assignment; nothing outside the
authorized surfaces was read.

## 2. What the current authority directly establishes

### 2.1 Physical output surfaces (claims BG001-A02-C002 … C009, AUTHORITATIVE)

| family | physical output column | type | scales present | tick scale |
|---|---|---|---|---|
| raw3 | `payload_raw3` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| range | `payload_range` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| fvg | `payload_fvg` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| bos_choch | `payload_bos_choch` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| local_structure | `payload_local_structure` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| order_blocks | `payload_order_blocks` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| micro_liquidity | `payload_micro_liquidity` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |
| micro_liquidity_context | `payload_micro_liquidity_context` | LargeUtf8, nullable | 15s, 30s, 1m, 5m, 15m, 1h, 4h | absent |

Schema verification covers all 6 parts of each scale (`schema_verified_parts`). Each family
appears under exactly one identical column name at every Bar scale — this is
**cross-scale surface duplication** (claim BG001-A02-C026, AUTHORITATIVE), a physical fact
only; it implies nothing about content-level sharing or repackaging.

### 2.2 Time coordinates and availability (claims BG001-A02-C018 … C020, AUTHORITATIVE)

- Bar rows carry exactly two physical time coordinate fields: `bar_open_ts` and
  `bar_close_ts` (C018).
- No known-at, available-at, receipt-time, or other availability field exists anywhere in
  the Bar-row physical schema; the two time coordinates are the complete set (C019 — a
  directly evidenced schema absence, since the physical schema enumerates every column).
- The Tick source carries `event_ts_ns` and `received_ts_ns`; `received_ts_ns` is the only
  physical receipt-time coordinate on any authorized surface (C020). No authority maps
  Tick receipt times to Bar payload availability — that mapping is unresolved.

### 2.3 Schema-level co-location (claim BG001-A02-C024, AUTHORITATIVE)

On every Bar scale the payload columns are serialized on the same physical bar row as the
identity, time-coordinate, and OHLCV fields. Co-location of output surfaces establishes no
object-level lineage, no object-level co-occurrence, and no confluence — and no accepted
edge type expresses co-location.

### 2.4 Program rules that bind lineage work (claims BG001-A02-C021 … C023, AUTHORITATIVE)

- **Rule 2 — one causal information clock** (C021): at anchor time `t`, only information
  actually available by `t` is eligible; occurrence/origin timestamps remain provenance.
- **Rule 4 — lifecycle-first** (C022): future completion/termination cannot define an
  earlier-state cohort; completed may only be a later outcome or a later anchor.
- **Rule 11 — derived-object independence** (C023): if a detector derives from another
  object, the pairing is not automatically independent confluence. This general rule is
  authoritative; **specific pair warnings remain UNRESOLVED** unless directly evidenced.

## 3. The output-surface principle (P1)

A `payload_*` column is a physical **OUTPUT SURFACE**, not proof of producer raw input.
The source inventory establishes the physical output only; producer inputs remain
UNRESOLVED unless directly evidenced. Subject names are identifiers only; no semantic
reading of a name was treated as evidence.

## 4. Per-family authority dossier (11 points)

Codes: **A** = AUTHORITATIVE, **U** = UNRESOLVED, **A/U** = mixed (authoritative physical
fact + unresolved semantics). Surface claims: C002–C009; dossier claims: C010–C017.

| family | 1 surface | 2 producer inputs | 3 upstream detector inputs | 4 object ids | 5 retained upstream/source ids | 6 occurrence ts | 7 known/availability ts | 8 lifecycle | 9 duplication/repackaging | 10 information ancestry | 11 unresolved lineage |
|---|---|---|---|---|---|---|---|---|---|---|---|
| raw3 | `payload_raw3` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| range | `payload_range` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| fvg | `payload_fvg` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| bos_choch | `payload_bos_choch` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| local_structure | `payload_local_structure` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| order_blocks | `payload_order_blocks` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| micro_liquidity | `payload_micro_liquidity` (A) | U | U | U | U | A/U | U | U | A/U | U | U |
| micro_liquidity_context | `payload_micro_liquidity_context` (A) | U | U | U | U | A/U | U | U | A/U | U | U |

Legend for the mixed and constrained cells:

- **6 occurrence ts — A/U.** The physical row coordinates `bar_open_ts`/`bar_close_ts` are
  authoritative (C018); per-object occurrence semantics *inside* the payload content are
  unresolved. Occurrence time is provenance, not availability (C021).
- **7 known/availability ts — U.** No availability field exists on the Bar row surface
  (C019, authoritative absence); the known-time semantics of payload objects are unresolved.
- **8 lifecycle — U.** Formation/activation/invalidation/completion/termination semantics
  are unresolved per family; the lifecycle-first rule (C022) constrains any reconstruction.
- **9 duplication — A/U.** Surface duplication across the seven Bar scales is
  authoritative (C026); content-level sharing, independent recomputation, or repackaging
  within/across scales, and within-row object multiplicity, are unresolved.
- **10 ancestry — U.** The only directly evidenced ancestry is physical serialization into
  the family's own payload column on a bar row (C024); all informational ancestry is
  unresolved.
- **11 unresolved lineage — U.** All typed lineage relations are recorded as relationship
  questions (Section 6); candidate upstreams outside the declared subject set are not
  representable as graph nodes or pairs and remain unresolved inside the dossiers.

Episode and object identity semantics (identifier scheme, multiplicity, upstream identifier
retention, episode open/close identity) are unresolved program-wide (claim BG001-A02-C027):
only content-hash identities of the source and payload manifests are available
(`source_manifest_identity`, `payload_manifest_identity`); the manifest contents themselves
are not on authorized surfaces.

## 5. Graph decision

- `graph.nodes` — exactly the eight declared subjects; no undeclared node is referenced
  anywhere in the graph.
- `graph.edges` — **empty (accepted edges = 0)**. Decision claim BG001-A02-C025
  (AUTHORITATIVE): no typed relationship between any pair of declared subjects is directly
  established by authority on the authorized surfaces. Three facts force this outcome:
  the output-surface principle (P1), schema co-location without an accepted edge type
  (C024), and the absence of any producer-input authority artifact (C001).
- The `USES_SWING_FROM` edge type presupposes an upstream object type that is not a
  declared subject; no node, pair, or edge referencing such an object was created, and the
  corresponding lineage gap is carried as an unresolved aspect inside each dossier.
- **Edge time-semantics template** (binds any future accepted edge): every accepted edge
  must distinguish `upstream_occurrence_ts`, `upstream_known_ts`, and
  `downstream_known_ts`. Occurrence time is provenance, not availability; a downstream
  known time must be independently evidenced and must never default to an upstream
  occurrence time. With `edges = []` the template is currently vacuous.

## 6. Relationship questions (15)

All entries use `relation = UNRESOLVED_RELATIONSHIP`, `origin = ASSIGNMENT_SPECIAL_REVIEW`,
`evidence = []`. Pair order follows the declared subject order and asserts no direction.

| # | pair | why the pair is asked |
|---|---|---|
| 1 | raw3 + range | base-layer vs derived-range lineage |
| 2 | raw3 + fvg | base-layer vs derived-gap lineage |
| 3 | raw3 + bos_choch | base-layer vs break/structure lineage |
| 4 | raw3 + local_structure | base-layer vs local-structure lineage |
| 5 | raw3 + order_blocks | base-layer vs block lineage |
| 6 | raw3 + micro_liquidity | base-layer vs micro-liquidity lineage |
| 7 | raw3 + micro_liquidity_context | base-layer vs context lineage |
| 8 | range + fvg | range/boundary usage in gap definition |
| 9 | range + bos_choch | range/boundary usage in break definition |
| 10 | range + order_blocks | range/boundary usage in block definition |
| 11 | range + micro_liquidity | range/boundary usage in micro-liquidity definition |
| 12 | bos_choch + local_structure | nesting/derivation between structure objects |
| 13 | bos_choch + order_blocks | break-event usage in block definition |
| 14 | micro_liquidity + micro_liquidity_context | contextualization |
| 15 | fvg + order_blocks | gap/block overlap or derivation |

The set covers all eight subjects (each appears in at least one question) without
enumerating all 28 pairwise combinations; pairs were selected where the accepted edge
vocabulary anticipates a distinct relationship kind (derivation, boundary/range usage,
break usage, nesting, contextualization).

## 7. Fake-confluence warnings

- **BG001-A02-W001 (program, AUTHORITATIVE).** General rule from protocol rule 11: a
  derived object plus its upstream object must not be scored as two independent signals
  without an approved independence basis. This rule is authoritative program-wide.
- **BG001-A02-W002 … W009 (one per family, UNRESOLVED).** Pair-level confluence
  independence for each family against any candidate upstream (declared subject or
  non-subject input) is unresolved. No pair-specific fake-confluence warning is asserted,
  because no lineage edge is established; specific pair warnings remain unresolved unless
  directly evidenced.

## 8. Future-information matrix

Anchors (AUTHORITATIVE): bar row time coordinates `bar_open_ts`/`bar_close_ts` (C018);
no availability field on the Bar row surface (C019); causal clock rule (C021);
lifecycle-first rule (C022).

| subject | occurrence time | known time | bar-field → known-time mapping | lifecycle completion → earlier state | upstream id retention | cross-scale aggregation |
|---|---|---|---|---|---|---|
| raw3 | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| range | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| fvg | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| bos_choch | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| local_structure | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| order_blocks | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| micro_liquidity | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |
| micro_liquidity_context | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED | UNRESOLVED |

> Specific bar-field future mappings remain unresolved without direct lifecycle authority.

## 9. Blocker

**BG-001 E3 lineage-dependent research is BLOCKED pending approved lineage authority.**

E3 (cross-domain) work that depends on lineage cannot begin until an approved lineage
authority establishes, per family: producer raw inputs, upstream detector inputs, object
identity, retained upstream/source identifiers, occurrence and known-time semantics, and
lifecycle semantics. Under the current authorization those aspects are entirely
unresolved (Section 4).

## 10. Unresolved summary

- producer raw inputs — unresolved for all 8 families.
- upstream detector inputs — unresolved for all 8 families.
- object id scheme inside payload contents — unresolved for all 8 families.
- retained upstream/source ids — unresolved for all 8 families.
- occurrence-time semantics inside payload contents — unresolved for all 8 families.
- known/availability semantics — unresolved for all 8 families (no availability field
  exists on the Bar row surface).
- family lifecycle semantics — unresolved for all 8 families.
- content-level duplication or repackaging within and across scales — unresolved for all
  8 families.
- information ancestry beyond physical serialization — unresolved for all 8 families.
- all 15 candidate lineage pairs — recorded as UNRESOLVED_RELATIONSHIP questions.
- episode and object identity semantics — unresolved (manifest content hashes only).
- bar-field future mappings — unresolved without direct lifecycle authority.

## 11. Validation performed

| check | result |
|---|---|
| candidate JSON parses | pass |
| statuses restricted to AUTHORITATIVE / PROVISIONAL / UNRESOLVED | pass |
| claim ids: 29 unique, format `BG001-A02-C###`, declared prefix | pass |
| every claim carries well-formed `authority_evidence` objects (source_id, relative_path, locator, evidence_type, supports) | pass |
| `subjects` array exactly the eight declared families | pass |
| graph keys exactly nodes / edges / relationship_questions / fake_confluence_warnings | pass |
| graph nodes == subjects; question pairs within subjects; warning scopes declared | pass |
| accepted edges = 0 | pass |
| relationship question shape (pair, relation, origin, evidence) exact | pass |
| no dangling claim-id references | pass |
| blocker and matrix statements verbatim | pass |
| forbidden identity terms absent from both artifacts; only relative paths used | pass |
| workspace contains exactly the two required files | pass |
