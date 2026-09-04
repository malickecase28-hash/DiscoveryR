# Authority Review — subject `fvg` (XAUUSD)

Program AP-002 · Role A-02 · Phase AUTHORITY · Access profile AUTHORITY
Authority source: `xauusd.wave1.ap002`

This review accompanies `authority_candidate.json` (artifact_type
AUTHORITY_CANDIDATE). It cites claims only by their identifiers in that file.

## 1. Scope and method

Assignment mandate: "Reconstruct FVG lifecycle, availability, and episode identity
semantics independently."

All authorized repository surfaces were read in full:

- `repository/protocol/` — `AGENT_ACCESS.md`, `RESEARCH_RULES.md`
- `repository/contracts/` — detector registry entry, experiment contract,
  knowledge record, and run manifest schemas (v1)
- `repository/registry/` — `detectors_v1.json`
- `repository/instruments/XAUUSD/` — `instrument_config.json`, `source_inventory.json`
- `repository/agent_harness/assignments/AP-002/A-02.json`
- `authority/authority.json`, `authority/bundle_manifest.json`
- `manifest/research_input_manifest.json`

Raw lake access and peer visibility are DENIED for this assignment; no raw lake
data, peer workspaces, prior reports, legacy material, credentials, provider maps,
or host drives were inspected. No public-domain or ordinary-language definition of
the subject was applied: every statement below is grounded only in the mounted
surfaces. Detector architecture and semantics remain unexposed (not authorized for
this run).

## 2. Program unresolved state (preserved)

The authority bundle for this program/subject is **unresolved pending source
approval** (AP002-A02-C001): `authority.json` has empty `selected`, `provenance`,
and `status` collections, `unresolved = ["UNRESOLVED_PENDING_SOURCE_APPROVAL"]`,
and `authority.json` is the bundle's only generated file.

This state is program-level and persists until an explicit source-approval/freeze
action (confirmation is fail-closed per the access protocol). Accordingly, the
candidate carries no approved semantic selections. It reports (a) direct
structured facts supported by the inventory and registry, and (b) a structured
unresolved ledger of every requested topic.

## 3. Established facts (AUTHORITATIVE)

| Claim ID | Topic | Finding |
|----------|-------|---------|
| AP002-A02-C001 | authority_bundle_state | Bundle unresolved pending source approval; no approved selections, provenance, or statuses. |
| AP002-A02-C002 | registry_semantics | Registry entry for fvg: domain bar, known_surfaces [bar], explicit native scales 15s–4h, semantic_status UNRESOLVED, placeholder description, empty lifecycle_vocabulary / roles / derives_from. |
| AP002-A02-C003 | output_surface | Every bar scale (15s, 30s, 1m, 5m, 15m, 1h, 4h) carries a nullable LargeUtf8 column `payload_fvg` in all six schema-verified parts; the tick schema carries no fvg column. |
| AP002-A02-C004 | output_surface | No fvg-specific fields are physically materialized outside `payload_fvg`; bar schemas contain only instrument, time, OHLCV, and payload columns. |
| AP002-A02-C005 | native_scales | Registry native scales coincide exactly with the inventory bar scales carrying `payload_fvg`; no tick or other non-bar fvg surface exists. |

Inventory extent (context for availability and censoring; totals are sums of the
recorded per-part row counts, basis `parts[].rows` in `source_inventory.json`,
preserved per research rule 10):

| Scale | Parts | Total rows (sum of recorded parts) |
|-------|-------|------------------------------------|
| 15s   | 6     | 1,320,835 |
| 30s   | 6     | 660,859 |
| 1m    | 6     | 330,458 |
| 5m    | 6     | 66,147 |
| 15m   | 6     | 22,050 |
| 1h    | 6     | 5,515 |
| 4h    | 6     | 1,464 |

A tick source (six parts, 68,445,659 recorded rows total) also exists in the
inventory but has no fvg payload column (AP002-A02-C003).

## 4. Provisional finding

- **Detector-to-column binding (AP002-A02-C006).** The correspondence between
  detector `fvg` and column `payload_fvg` rests on name alignment plus the
  bar-domain match between the registry entry and the bar schemas. No authorized
  surface explicitly asserts the binding (recorded as counterevidence on the
  claim). Downstream use should carry it as an explicit assumption until an
  approved source confirms it.

## 5. Unresolved ledger (mandatory topics)

| Claim ID | Topic | Basis (locations checked) |
|----------|-------|---------------------------|
| AP002-A02-C007 | lifecycle | Registry lifecycle_vocabulary empty; no state or transition definitions on any surface. |
| AP002-A02-C008 | formation | No formation rule, direction, or threshold on any authorized surface. |
| AP002-A02-C009 | detection | No implementation or parameterization mounted; raw lake DENIED; registry records none. |
| AP002-A02-C010 | object_identity | No identity keys, equality, or merge/split rules; no identity fields in schema. |
| AP002-A02-C011 | geometry | No boundaries, extent, or price-level construction; no geometry fields in schema. |
| AP002-A02-C012 | touch | No touch or first-touch state or event definitions; empty lifecycle vocabulary. |
| AP002-A02-C013 | fill | No fill existence, completeness, or measurement definitions; empty lifecycle vocabulary. |
| AP002-A02-C014 | invalidation | No invalidation, termination, or expiry semantics on any surface. |
| AP002-A02-C015 | availability | Registry field is literally "UNRESOLVED; authoritative manifest surface only."; no anchor-time eligibility rule. |
| AP002-A02-C016 | censoring | No session calendar, gap, or data-quality metadata; time-coordinate unit and timezone unspecified; no boundary rules. |
| AP002-A02-C017 | mutation | No revision or recompute-stability semantics; single nullable column, no version fields. |
| AP002-A02-C018 | cross_scale_identity | Seven native bar scales registered; no scale-bridging identity rule; native scale stays attached (rule 3). |

Additional unresolved claims: AP002-A02-C019 (episode_identity),
AP002-A02-C020 (derivation), AP002-A02-C021 (payload_null_semantics),
AP002-A02-C022 (internal_serialization).

## 6. Requested concepts vs authoritative states

FORMATION, FIRST_TOUCH, and FILL are **requested concepts from the run
instructions, not authoritative lifecycle states** (AP002-A02-C007, with the
touch and fill specifics in AP002-A02-C012 and AP002-A02-C013). The
authoritative registry provides an empty lifecycle vocabulary for fvg.
Therefore:

- No state machine, state sequence, or transition rule is asserted for fvg.
- No temporal ordering among the requested concepts is asserted — in particular,
  nothing here claims that fill is definitionally later than touch or formation.
  Research rule 4 (lifecycle-first: future completion/termination cannot define an
  earlier-state cohort) is respected by construction.
- No property of any requested concept is inferred from its name, from physical
  field names, or from ordinary language.

## 7. Structural gaps visible to downstream phases

These are direct consequences of the established facts and will block or shape
later phases unless the approving source settles them:

1. **Contract grounding (AP002-A02-C019).** The experiment contract schema
   requires `anchor.lifecycle_state` (and `anchor_time_semantics`) for every
   anchor, but the registry provides no authoritative fvg vocabulary. fvg-anchored
   contracts cannot be written without importing non-authoritative labels.
2. **Anchor-time eligibility (AP002-A02-C015).** Research rule 2 requires a single
   causal information clock, but no availability rule exists for fvg, and each bar
   row carries two time coordinates (`bar_open_ts`, `bar_close_ts`) with no rule
   selecting between them.
3. **Time coordinate basis (AP002-A02-C016).** `bar_open_ts`/`bar_close_ts` are
   `Int64` with unit and timezone unspecified in the inventory.
4. **Censoring basis (AP002-A02-C016).** The inventory contains no session
   calendar, gap, holiday, or data-quality metadata; behavior at data edges and
   across the six-part partitions is undefined.
5. **Object population (AP002-A02-C003, AP002-A02-C022).** FVG outputs exist
   physically only as row-attached, nullable serialized strings; object counts,
   identity, and episode segmentation cannot be derived from the inventory alone.
6. **Independence (AP002-A02-C020).** `derives_from` is empty; any future claim of
   fvg participation in confluence must establish independence separately
   (rule 11).

## 8. Open questions for the source-approval gate

The unresolved_questions array in `authority_candidate.json` carries the program
unresolved state and fifteen open questions, each cross-referenced to its claim.
In summary, the approving source should settle:

1. The detector-to-column binding (AP002-A02-C006).
2. The formation rule (AP002-A02-C008) and detection semantics (AP002-A02-C009).
3. The authoritative lifecycle vocabulary and transitions, including whether
   touch and fill exist as states or events and any ordering (AP002-A02-C007,
   AP002-A02-C012, AP002-A02-C013).
4. Object identity and episode identity (AP002-A02-C010, AP002-A02-C019).
5. Geometry and payload serialization (AP002-A02-C011, AP002-A02-C022).
6. Availability, the governing time coordinate, and null semantics
   (AP002-A02-C015, AP002-A02-C021).
7. Invalidation and mutation (AP002-A02-C014, AP002-A02-C017).
8. Censoring (AP002-A02-C016) and cross-scale identity (AP002-A02-C018).
9. Derivation and independence (AP002-A02-C020).

## 9. Compliance

- Exactly two artifacts were written to the private workspace:
  `authority_candidate.json` and `authority_review.md`. No scripts, ZIP files, or
  other artifacts were created; no Git operations were performed.
- The candidate uses the required top-level shape (artifact_type AUTHORITY_CANDIDATE,
  program_id, role_id, subjects, claims, unresolved_questions, sources_consulted);
  claim identifiers follow the PROGRAMPREFIX-ROLEPREFIX-CNNN grammar and are unique.
- Every AUTHORITATIVE claim carries a non-empty authority_evidence array of
  structured objects (source_id, relative_path, locator, evidence_type, supports);
  the single PROVISIONAL claim states its assumption and counterevidence
  explicitly; all other topics are UNRESOLVED. No claim uses any status outside
  {AUTHORITATIVE, PROVISIONAL, UNRESOLVED}.
- No personal, operational, or implementation identity appears in either artifact.
- The bundle remains unresolved pending source approval; nothing here constitutes
  a freeze, publication, or approval action.

This review stops after validation of the two required files.
