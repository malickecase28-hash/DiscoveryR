# Authority Review — Detector fvg

| Field | Value |
|---|---|
| Program | AP-002 |
| Role | A-02 |
| Phase | AUTHORITY |
| Access profile | AUTHORITY |
| Detector | fvg |
| Artifact | AUTHORITY_CANDIDATE (companion: `authority_candidate.json`) |
| Date | 2026-09-04 |
| Claim prefix | `AP002-A02-C###` |

## 1. Scope and method

Blind authority research only. This review reconstructs what the authorized repository
surfaces establish about detector fvg semantics and causal authority across native bar
scales. The following were **not** performed: behavioral research, fill-rate analysis,
prediction testing, strategy research, raw-lake access, peer inspection, or shared-file
edits. Meaning is never inferred from names: `payload_fvg` is treated as a column name
that establishes nothing beyond what its schema entry declares. Matters without direct
supporting authority are marked UNRESOLVED.

The general causal rule is treated as authoritative **only** because the protocol states
it (rule 2). Exact fvg availability and field mapping remain unresolved without direct
authority. An absent or empty `derives_from` declaration does not prove independence.

## 2. Sources consulted

| Source id | Path | Status | Notes |
|---|---|---|---|
| `xauusd.schemas` | `instruments/XAUUSD/source_inventory.json` | READ_FULL | Sole authority source id designated by the assignment (`$.authority_source_ids`). |
| `xauusd.instrument_config` | `instruments/XAUUSD/instrument_config.json` | READ_FULL | Lake root, relative path, manifest declarations. |
| `protocol.research_rules` | `protocol/RESEARCH_RULES.md` | READ_FULL | 14 program rules; rules 2, 3, 4, 5, 11 cited. |
| `assignment.ap002.a02` | `assignment.json` | READ_FULL | Tasking; confirms AP-002 / A-02 / fvg and access denials. |
| `repository.contracts` | `contracts` | EMPTY | 0 entries (verified 2026-09-04). |
| `repository.registry` | `registry` | EMPTY | 0 entries (verified 2026-09-04). |
| `repository.authority` | `authority` | EMPTY | 0 entries (verified 2026-09-04). |
| `agent_harness.assignment` | `agent_harness/assignments/AP-002/A-02.json` | NOT_FOUND | Listed as an authorized surface in `assignment.json ($.authorized_repository_surfaces)` but no such path exists in the repository root. |

Raw-lake access is DENIED, so the lake manifests (`manifest.json`, `payload_manifest.json`)
declared in `instruments/XAUUSD/instrument_config.json ($.manifest, $.payload_manifest)`
could not be inspected.

### Inventory locator map

The inventory's `$.sources` array has eight entries; locators below use these indices:

| Index | Scale | Index | Scale |
|---|---|---|---|
| `$.sources[0]` | Bar 15m | `$.sources[4]` | Bar 30s |
| `$.sources[1]` | Bar 15s | `$.sources[5]` | Bar 4h |
| `$.sources[2]` | Bar 1h | `$.sources[6]` | Bar 5m |
| `$.sources[3]` | Bar 1m | `$.sources[7]` | Tick |

## 3. Established physical facts (AUTHORITATIVE)

### 3.1 Physical payload column (Task Q1)

The detector fvg payload column is `payload_fvg`, declared `data_type: LargeUtf8`,
`nullable: true`, in the `physical_schema` of **every Bar scale** — 15m, 15s, 1h, 1m,
30s, 4h, and 5m (claim `AP002-A02-C001`). Exact locators, each resolving to
`{"name": "payload_fvg", "data_type": "LargeUtf8", "nullable": true}`:

- `instruments/XAUUSD/source_inventory.json → $.sources[0].physical_schema[11]` (15m)
- `instruments/XAUUSD/source_inventory.json → $.sources[1].physical_schema[11]` (15s)
- `instruments/XAUUSD/source_inventory.json → $.sources[2].physical_schema[11]` (1h)
- `instruments/XAUUSD/source_inventory.json → $.sources[3].physical_schema[11]` (1m)
- `instruments/XAUUSD/source_inventory.json → $.sources[4].physical_schema[11]` (30s)
- `instruments/XAUUSD/source_inventory.json → $.sources[5].physical_schema[11]` (4h)
- `instruments/XAUUSD/source_inventory.json → $.sources[6].physical_schema[11]` (5m)

`payload_fvg` is also registered in each Bar scale's `payload_columns` manifest list at
zero-based index 3 (`$.sources[0..6].payload_columns[3]`) — claim `AP002-A02-C002`.

### 3.2 Record variants (Task Q1)

- Detector fvg output exists **only** as a per-row nullable column on Bar records; the
  inventory declares no separate fvg output record type, table, or stream
  (`$.sources` — eight entries: seven Bar scales + Tick) — claim `AP002-A02-C003`.
- The Tick schema has no fvg payload column at all: its twelve fields are
  `event_ts_ns, received_ts_ns, source_sequence, bid, ask, payload_drift_burst,
  payload_spread_state, payload_quote_dynamics, payload_quote_arrival,
  payload_micro_volatility, payload_feed_health, payload_quote_pressure`
  (`$.sources[7].physical_schema`) — claim `AP002-A02-C003`.
- Each Bar scale declares all six of its parts in `schema_verified_parts`
  (`$.sources[0..6].schema_verified_parts`), so one declared schema is verified across
  every listed part; no variant record shapes are declared — claim `AP002-A02-C004`.
- The denotation of a null `payload_fvg` value is **not documented** — claim
  `AP002-A02-C042` (UNRESOLVED).

### 3.3 Physical time coordinates (Task Q27, partial)

Bar records carry `bar_open_ts` and `bar_close_ts` (both `Int64`, `nullable=false`) as
`physical_time_coordinate_fields`; Tick carries `event_ts_ns` and `received_ts_ns`
(`$.sources[0].physical_time_coordinate_fields`, `$.sources[7].physical_time_coordinate_fields`)
— claim `AP002-A02-C005`. The unit, epoch, and timezone of these integers are **not
documented**; no fvg-specific occurrence timestamp field is documented anywhere
(claim `AP002-A02-C034`, UNRESOLVED).

### 3.4 Corpus extent (Task Q19/Q20 context)

The inventory enumerates six parts per source with row counts (claim `AP002-A02-C041`):

| Scale | Rows | Scale | Rows |
|---|---|---|---|
| 15m | 22,050 | 30s | 660,859 |
| 15s | 1,320,835 | 4h | 1,464 |
| 1h | 5,515 | 5m | 66,147 |
| 1m | 330,458 | Tick | 68,445,659 |

No time coverage is documented for any source, so the corpus end time is not
determinable from approved sources (claim `AP002-A02-C026`).

### 3.5 Declared producer dependencies (Task Q30)

Directly declared in approved sources (claim `AP002-A02-C006`):

- `instruments/XAUUSD/instrument_config.json → $.lake_root_env` = `TRINITYR_ANALYTICAL_LAKE`
- `instruments/XAUUSD/instrument_config.json → $.relative_path` = `fusion_markets/xauusd`
- `instruments/XAUUSD/instrument_config.json → $.manifest` = `manifest.json`
- `instruments/XAUUSD/instrument_config.json → $.payload_manifest` = `payload_manifest.json`
- `instruments/XAUUSD/source_inventory.json → $.source_manifest_identity` =
  `1176d6518c83ff1b5ec27b03df416c11d146b9611fb307398193c5cb9b5053af`
- `instruments/XAUUSD/source_inventory.json → $.payload_manifest_identity` =
  `78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed`

**No producing implementation, library, tool, or derivation is named in any approved
source, and no `derives_from` declaration exists in any authorized surface**
(claim `AP002-A02-C007`, negative-existence within the authorized surfaces). Because
protocol rule 11 (`protocol/RESEARCH_RULES.md`, line 20) states that a detector deriving
from another does not create automatic independent confluence (claim `AP002-A02-C040`),
**independence of detector fvg is UNRESOLVED**; absence of a `derives_from` declaration
does not prove independence.

## 4. Registry status (cited separately, as required)

The registry surface `registry` contains **0 entries** (verified 2026-09-04);
no detector registry entry for fvg exists to consult — claim `AP002-A02-C008`. The
assignment designates exactly one authority source id, `xauusd.schemas`
(`assignment.json → $.authority_source_ids`). All registry-derived fvg semantics are
UNRESOLVED as a consequence. This section is deliberately separate from the
source-inventory claims in §3.

## 5. Program-rule authorities from the protocol

These are authoritative **as general program constraints**; none resolves an
fvg-specific question by itself:

| Rule | Locator | Content (summary) | Claim |
|---|---|---|---|
| Rule 2 | `protocol/RESEARCH_RULES.md`, lines 5–6 | One causal information clock: at anchor time `t`, only information actually available by `t` is eligible; occurrence/origin timestamps remain provenance. | `AP002-A02-C037` |
| Rule 3 | `protocol/RESEARCH_RULES.md`, lines 7–8 | Native scale preservation: a shared causal clock does not make 15s, 1m, 4h, and tick objects statistically identical; native scale remains attached. | `AP002-A02-C038` |
| Rule 4 | `protocol/RESEARCH_RULES.md`, lines 9–10 | Lifecycle-first: future completion/termination cannot define an earlier-state cohort; completed may be a later outcome or later anchor. | `AP002-A02-C039` |
| Rule 5 | `protocol/RESEARCH_RULES.md`, lines 11–12 | E1 PHENOTYPE, E2 SAME_DOMAIN, E3 CROSS_DOMAIN are information-permission classes, not progression gates. | cited in `AP002-A02-C036` |
| Rule 11 | `protocol/RESEARCH_RULES.md`, line 20 | If detector B derives from A, A+B is not automatically independent confluence. | `AP002-A02-C040` |

## 6. Native-scale and cross-scale status (Task Q23/Q24)

- `payload_fvg` is physically present with identical declared type (`LargeUtf8`,
  `nullable=true`) on all seven Bar scales — claim `AP002-A02-C030` (AUTHORITATIVE).
- Whether fvg objects share a cross-scale identity, or identity is native per scale, is
  **UNRESOLVED** — claim `AP002-A02-C031`. Protocol rule 3 constrains any cross-scale
  treatment (native scale stays attached; scales are not statistically identical) but
  neither creates nor denies cross-scale object identity. Each scale is a separate
  inventory source entry with its own parts and schema, and no cross-scale linkage field
  is documented.

## 7. Unresolved semantic matters

Every claim below is UNRESOLVED; each carries structured absence evidence in the
companion JSON (empty `contracts`, `registry`, and `authority` surfaces, plus
inventory/protocol locators where relevant).

| Task Q | Matter | Claim |
|---|---|---|
| 2 | Stable fvg object identity | `AP002-A02-C009` |
| 3 | Exact formation rule | `AP002-A02-C010` |
| 4 | Exact detection rule | `AP002-A02-C011` |
| 5 | Occurrence vs detection distinction (fvg-specific) | `AP002-A02-C012` |
| 6 | Lawful availability timestamp | `AP002-A02-C013` |
| 7 | Whether detection may refer to earlier bars | `AP002-A02-C014` |
| 8 | Direction semantics | `AP002-A02-C015` |
| 9 | Near edge, far edge, high/low, midpoint, gap width | `AP002-A02-C016` |
| 10 | Lifecycle vocabulary | `AP002-A02-C017` |
| 11 | Persistence after formation | `AP002-A02-C018` |
| 12 | First-touch definition | `AP002-A02-C019` |
| 13 | First touch as event, state, or retrospective annotation | `AP002-A02-C020` |
| 14 | Fill definition | `AP002-A02-C021` |
| 15 | Fill basis and modes | `AP002-A02-C022` |
| 16 | Fill timestamp availability | `AP002-A02-C023` |
| 17 | Invalidation/removal semantics | `AP002-A02-C024` |
| 18 | Terminal conditions | `AP002-A02-C025` |
| 19 | Objects unresolved at corpus end | `AP002-A02-C026` |
| 20 | Right censoring | `AP002-A02-C027` |
| 21 | Multiple touches / repeated interaction identity | `AP002-A02-C028` |
| 22 | Object mutation vs new records | `AP002-A02-C029` |
| 24 | Cross-scale identity | `AP002-A02-C031` |
| 25 | Causal fields by lifecycle stage | `AP002-A02-C032` |
| 26 | Retrospective fields | `AP002-A02-C033` |
| 27 | fvg provenance timestamps (beyond §3.3) | `AP002-A02-C034` |
| 28 | Future-information risk mapping (fvg-specific) | `AP002-A02-C035` |
| 29 | E1 object/episode reconstruction rule | `AP002-A02-C036` |
| — | Null denotation of `payload_fvg` | `AP002-A02-C042` |

FORMATION, FIRST_TOUCH, and FILL are **requested concepts, not authoritative lifecycle
states** (claim `AP002-A02-C017`): no authorized source names any fvg lifecycle state,
so no state vocabulary can be attributed to them.

## 8. FORMATION / FIRST_TOUCH / FILL future-information matrix

Basis: protocol rule 2 (eligibility at anchor `t` requires availability by `t`;
occurrence/origin timestamps remain provenance) and protocol rule 4 (future
completion/termination cannot define an earlier-state cohort) are authoritative as
general constraints. No authorized source maps any of these concepts to detector fvg
fields or availability times.

**General posture:** because no availability mapping exists for any requested concept,
every `payload_fvg`-derived field must be treated as **not eligible at any anchor** until
its availability is established by direct authority. This is a conservative application of
protocol rule 2, not a finding that the fields are retrospective.

| Concept | Classification | Definition authority | Availability authority | Retrospective risk | E1-usable now | Claims |
|---|---|---|---|---|---|---|
| FORMATION | Requested concept only (not an authoritative state) | UNRESOLVED | UNRESOLVED | UNASSESSABLE — presumed ineligible at all anchors until availability is established | No | `C010`, `C013`, `C017` |
| FIRST_TOUCH | Requested concept only (not an authoritative state) | UNRESOLVED | UNRESOLVED (event/state/retrospective classification also unresolved) | UNASSESSABLE — could be a retrospective annotation; presumed ineligible at anchors before availability is demonstrated | No | `C019`, `C020`, `C017` |
| FILL | Requested concept only (not an authoritative state) | UNRESOLVED (basis and modes unresolved) | UNRESOLVED | UNASSESSABLE — definitionally a later outcome relative to formation, so under rule 4 it cannot define an earlier-state cohort, and under rule 2 it is ineligible at anchors before its availability is established | No | `C021`, `C022`, `C023`, `C039` |

## 9. E1 reconstruction: **BLOCKED**

The exact object/episode reconstruction rule for exposure class E1 (PHENOTYPE) is
**UNRESOLVED and reconstruction is BLOCKED** (claim `AP002-A02-C036`). Blockers:

1. No content schema for `payload_fvg` exists in any authorized source; only the physical
   column (name, type, nullability) is documented
   (`instruments/XAUUSD/source_inventory.json → $.sources[0].physical_schema[11]` and
   per-scale equivalents).
2. No object identity rule exists (`AP002-A02-C009`), so objects cannot be segmented into
   episodes.
3. No lawful availability timestamp mapping exists (`AP002-A02-C013`), so no field can be
   asserted eligible at any anchor.
4. The `registry`, `contracts`, and `authority` surfaces are empty (verified 2026-09-04),
   and raw-lake access is denied, so no payload sample or lake manifest can be inspected.
5. The authorized surface `agent_harness/assignments/AP-002/A-02.json` listed in
   `assignment.json → $.authorized_repository_surfaces` is not present in the repository.

E1 PHENOTYPE is named as an information-permission class by protocol rule 5; naming the
class does not supply a reconstruction rule.

## 10. Claim index

| Claim | Status | Subject | Task Q |
|---|---|---|---|
| `AP002-A02-C001` | AUTHORITATIVE | payload_fvg physical field on all seven Bar scales | 1 |
| `AP002-A02-C002` | AUTHORITATIVE | payload_columns registration of payload_fvg | 1 |
| `AP002-A02-C003` | AUTHORITATIVE | per-row column only; no fvg record type; absent from Tick | 1 |
| `AP002-A02-C004` | AUTHORITATIVE | schema verified across all listed parts per scale | 1 |
| `AP002-A02-C005` | AUTHORITATIVE | physical time coordinate fields and types | 27 |
| `AP002-A02-C006` | AUTHORITATIVE | declared lake/manifest dependencies and digests | 30 |
| `AP002-A02-C007` | AUTHORITATIVE (negative) | no producer implementation or derives_from declaration in authorized surfaces | 30 |
| `AP002-A02-C008` | AUTHORITATIVE (negative) | registry surface empty; sole authority source id is xauusd.schemas | 30 |
| `AP002-A02-C009` | UNRESOLVED | stable object identity | 2 |
| `AP002-A02-C010` | UNRESOLVED | formation rule | 3 |
| `AP002-A02-C011` | UNRESOLVED | detection rule | 4 |
| `AP002-A02-C012` | UNRESOLVED | occurrence vs detection | 5 |
| `AP002-A02-C013` | UNRESOLVED | lawful availability timestamp | 6 |
| `AP002-A02-C014` | UNRESOLVED | detection referencing earlier bars | 7 |
| `AP002-A02-C015` | UNRESOLVED | direction semantics | 8 |
| `AP002-A02-C016` | UNRESOLVED | geometry (edges, extremes, midpoint, width) | 9 |
| `AP002-A02-C017` | UNRESOLVED | lifecycle vocabulary; requested concepts only | 10 |
| `AP002-A02-C018` | UNRESOLVED | persistence after formation | 11 |
| `AP002-A02-C019` | UNRESOLVED | first-touch definition | 12 |
| `AP002-A02-C020` | UNRESOLVED | first touch as event/state/retrospective | 13 |
| `AP002-A02-C021` | UNRESOLVED | fill definition | 14 |
| `AP002-A02-C022` | UNRESOLVED | fill basis and modes | 15 |
| `AP002-A02-C023` | UNRESOLVED | fill timestamp availability | 16 |
| `AP002-A02-C024` | UNRESOLVED | invalidation/removal | 17 |
| `AP002-A02-C025` | UNRESOLVED | terminal conditions | 18 |
| `AP002-A02-C026` | UNRESOLVED | objects unresolved at corpus end | 19 |
| `AP002-A02-C027` | UNRESOLVED | right censoring | 20 |
| `AP002-A02-C028` | UNRESOLVED | multiple touches / repeated interaction | 21 |
| `AP002-A02-C029` | UNRESOLVED | mutation vs new records | 22 |
| `AP002-A02-C030` | AUTHORITATIVE | column presence on all seven Bar scales | 23 |
| `AP002-A02-C031` | UNRESOLVED | cross-scale object identity | 23, 24 |
| `AP002-A02-C032` | UNRESOLVED | causal fields by lifecycle stage | 25 |
| `AP002-A02-C033` | UNRESOLVED | retrospective fields | 26 |
| `AP002-A02-C034` | UNRESOLVED | fvg provenance timestamps; integer unit/epoch undocumented | 27 |
| `AP002-A02-C035` | UNRESOLVED | field-level future-information exposure | 28 |
| `AP002-A02-C036` | UNRESOLVED | E1 reconstruction rule — BLOCKED | 29 |
| `AP002-A02-C037` | AUTHORITATIVE | protocol rule 2 (causal information clock) | general |
| `AP002-A02-C038` | AUTHORITATIVE | protocol rule 3 (native scale preservation) | general |
| `AP002-A02-C039` | AUTHORITATIVE | protocol rule 4 (lifecycle-first) | general |
| `AP002-A02-C040` | AUTHORITATIVE | protocol rule 11 (derivation ≠ independent confluence) | general |
| `AP002-A02-C041` | AUTHORITATIVE | inventory extent (8 sources, 6 parts each, row counts) | 19, 20 |
| `AP002-A02-C042` | UNRESOLVED | null denotation of payload_fvg | 1, 25 |

## 11. Validation record

- Companion JSON parses as valid JSON.
- Required top-level fields present: `program_id`, `role_id`, `artifact_type`
  (= `AUTHORITY_CANDIDATE`), `subjects`, `claims`, `unresolved_questions`,
  `sources_consulted`.
- All 42 claim ids use the prefix `AP002-A02-C###` and are sequential without gaps.
- Every claim carries ≥1 structured evidence object with exactly the fields
  `source_id`, `relative_path`, `locator`, `evidence_type`, `supports`; locators are
  exact file/object/field/section paths.
- Statuses used: AUTHORITATIVE and UNRESOLVED only.
- The physical payload claim cites the exact source-inventory field for `payload_fvg`,
  `LargeUtf8`, and `nullable=true` (§3.1); registry claims are cited separately (§4).
- Forbidden identity terms absent from both artifacts (verified by case-insensitive scan).
- Exactly two files exist in the workspace: `authority_candidate.json` and
  `authority_review.md`.
