# Authority Review — drift_burst (XAUUSD)

Program AP-001 · Role A-01 · Phase AUTHORITY · Run Wave-1 Authority Run V2
Subject: `drift_burst` · Instrument: XAUUSD · Authority source: `xauusd.wave1.ap001`
Companion artifact: `authority_candidate.json` (artifact_type AUTHORITY_CANDIDATE; 29 claims: 9 AUTHORITATIVE, 1 PROVISIONAL, 19 UNRESOLVED)

This review accompanies the authority candidate expressed in the host publication shape: a flat
`claims` array in which every claim carries exactly the required semantic fields (claim_id, subject,
topic, claim, status, authority_evidence, counterevidence, future_information_risk,
dependency_implications, notes), with claim IDs in the AP001-A01-CNNN grammar.

**Provenance-only correction (no science reopened).** The publisher restricts `source_id` values to
three authorized identifiers: `xauusd.wave1.ap001` (the authority bundle), `protocol.research_rules`
(the protocol research rules), and `assignment.ap-001_a-01` (the assignment). Evidence previously
cited from other repository surfaces (detector registry, source inventory, instrument configuration,
contract schemas, input manifest) was dropped rather than relabeled, and every claim whose support
depended on such a surface was marked UNRESOLVED. Claim texts are unchanged; no new research was
performed; no evidence was invented.

## 1. Scope and access

This review covers only the mounted, read-only V2 evidence surfaces authorized by the assignment:
the research input manifest, the authority bundle (`authority/authority.json`,
`authority/bundle_manifest.json`), `repository/protocol/`, `repository/contracts/`,
`repository/registry/`, `repository/instruments/XAUUSD/instrument_config.json`,
`repository/instruments/XAUUSD/source_inventory.json`, and the assignment file itself. Raw lake
data, peer workspaces, prior reports, legacy material, and credentials were not accessed (raw lake
access is DENY for this assignment).

Under the publication contract, only surfaces representable by the three authorized source IDs are
citable in the candidate. `sources_consulted` therefore lists exactly those: the authority bundle
(both files, under `xauusd.wave1.ap001`), the research rules (under `protocol.research_rules`), and
the assignment (under `assignment.ap-001_a-01`). The remaining surfaces were consulted during the
run and informed the analysis, but cannot be cited as evidence under the authorized source ID set;
claims that depended on them are marked UNRESOLVED (see §4).

## 2. Method

- Structure-first. Claims were built from declared record surfaces, declared field paths, declared
  status records, and normative rule text. Nothing was inferred from field or record names: names
  are recorded verbatim as declared strings, and every name-adjacent reading is either left
  unresolved or explicitly flagged as non-confirmatory.
- Status discipline. Only AUTHORITATIVE, PROVISIONAL, and UNRESOLVED are used. No claim uses a
  relationship status. Every AUTHORITATIVE claim carries a non-empty `authority_evidence` array of
  structured objects (source_id, relative_path, locator, evidence_type, supports), each citing an
  authorized source. PROVISIONAL is used only for one structural reading that the declared evidence
  is consistent with but does not confirm, carrying explicit counterevidence.
- Counting was verified programmatically with an in-memory check that persisted no artifacts:
  51 field paths on `drift_burst_completed`, 146 on `drift_burst_state`, and element-for-element
  equality between the `emitted_events[]` path list and the completed-record path list.
- Fail-closed, in both directions. Where scientific evidence is absent, the claim is UNRESOLVED;
  where scientific evidence exists but its source cannot be represented by an authorized source ID,
  the claim is likewise UNRESOLVED rather than relabeled.

## 3. Established structure (AUTHORITATIVE)

**Declared record surfaces (AP001-A01-C001).** Within the bundle's selected contract
(`trinity.research.tick-payload-contract.v1`, source `native_detector_serialization_contract`),
exactly two drift_burst record surfaces are declared: `drift_burst_completed` and
`drift_burst_state`. This is the complete declared set within this bundle.

**Declared field paths (AP001-A01-C002, AP001-A01-C003).** `drift_burst_completed` declares 51 flat
field paths: identity (`event_id`, `internal_sign`); origin (`origin_idx/ts/price/vr/cabs`); three
milestone quartets (`weak_*`, `online_*`, `strong_*`); `peak_*` (five paths); `decay_risk_*`
(four); `first_dying_*` and `last_dying_*` (three each); `dying_entries`, `decay_recoveries`,
`dying_recoveries`; `end_reason/idx/ts/price`; `burst_points`; `burst_duration_to_peak_s`; five
`available_after_*_points` paths (weak, online, strong, decay_risk, first_dying); and three
`*_remaining_pct` paths. `drift_burst_state` declares 146 field paths: top-level `state` and
`event_id`; `authority.weak_online_scale_s` and `authority.strong_scale_s`; seventeen-field groups
under `authority.score_30s`, `authority.score_60s`, and `operational`; four-field groups under
`milestones.weak` and `milestones.online`; eight-field groups under `multiscale.5s`, `.15s`, `.30s`,
`.60s`; and 51 element fields under `emitted_events[]`.

**Nested structure (AP001-A01-C004).** The state surface is declared with dotted nesting and an
array-element path family; the completed surface is declared flat.

**Structural equivalence (AP001-A01-C005).** The 51 `emitted_events[]` element paths are identical,
in order and content, to the 51 `drift_burst_completed` paths. This equality is a fact about the
two declared path lists only.

**Bundle status (AP001-A01-C011).** The bundle itself declares the two statuses
`DECLARED_SERIALIZATION_CONTRACT_AVAILABLE` and
`LIFECYCLE_SEMANTICS_STILL_REQUIRE_PRODUCER_AUTHORITY`, with an empty unresolved array. Per the
bundle's own declaration, serialization structure is available but lifecycle semantics still
require producer authority.

**Normative constraints in force (AP001-A01-C013, AP001-A01-C014, AP001-A01-C015).** Protocol
rules 2 (causal information clock: occurrence/origin timestamps are provenance, not availability),
4 (lifecycle-first: future completion/termination cannot define an earlier-state cohort; completed
may be a later outcome or later anchor), and 3 (native scale preservation) govern all use of the
established structure.

## 4. Claims reclassified UNRESOLVED for provenance-only reasons

Seven claims describe declarations that were verified against mounted surfaces during the run, but
whose declaring surfaces cannot be represented by an authorized source ID. Per the provenance
policy their citations were dropped (not relabeled), their claim texts are preserved verbatim, and
their statuses were set to UNRESOLVED. Resolution requires the declaring surface to become citable
under an authorized source ID; no scientific reinterpretation is involved.

- **AP001-A01-C006 (physical storage).** Source-inventory declaration of the tick physical schema,
  parts, and row counts.
- **AP001-A01-C007 (domain/surface consistency).** Registry domain declaration plus the inventory's
  payload-column distribution.
- **AP001-A01-C008 (registry semantic status).** The registry's UNRESOLVED semantic and
  availability declarations.
- **AP001-A01-C009 (registry declared absences).** The registry's empty lifecycle/derivation/role
  arrays.
- **AP001-A01-C010 (provenance linkage).** Partial: the authority-bundle half (the bundle's
  payload-manifest provenance citation) remains cited; the inventory-side digest half cannot be
  cited, so the cross-surface agreement is unsupported.
- **AP001-A01-C012 (instrument mount mapping).** The instrument configuration's lake-mapping
  declaration.
- **AP001-A01-C016 (multiscale label overlap).** Partial: the declared multiscale labels remain
  cited from the bundle; the bar-scale vocabulary half came from a contract schema and cannot be
  cited, so the overlap observation is unsupported (previously PROVISIONAL).

Additionally, within claims AP001-A01-C018, AP001-A01-C019, AP001-A01-C021, AP001-A01-C026, and
AP001-A01-C027, individual citations to uncitable surfaces were dropped rather than relabeled; those
claims were already UNRESOLVED and their statuses are unchanged.

## 5. Provisional reading (PROVISIONAL, non-confirmatory)

**AP001-A01-C017 — emitted_events[] reading.** That state-record elements mirror the
completed-record field set (AP001-A01-C005) is consistent with state records embedding
completed-event records. The declared structure does not establish when such records appear, how
many appear, whether the array accumulates, or any lifecycle meaning; the bundle's own
producer-authority status line is recorded as counterevidence against promotion. It is kept at
PROVISIONAL precisely so that it is available for later challenge without having been promoted.

## 6. Unresolved pending producer authority (UNRESOLVED, scientific)

The following topics are unresolved on the science and require producer authority. None is resolved
by name-based inference. Each entry names its claim in the candidate, where the evidence of absence
or guard and the resolution requirement are recorded.

| Topic (claim) | Why unresolved | What would resolve it |
|---|---|---|
| Lifecycle meaning (AP001-A01-C018) | Bundle declares lifecycle semantics still require producer authority | Producer-authority lifecycle declaration mounted as an authorized source |
| State transitions (AP001-A01-C019) | `state` path declared; no state value vocabulary, initial state, or transition rules on any mounted surface | Producer declaration of states and transitions |
| Episode identity persistence (AP001-A01-C020) | `event_id` declared on both surfaces and in `emitted_events[]`; no identity format, uniqueness scope, or persistence rule declared | Producer declaration of identity construction, scope, and cross-surface persistence |
| Availability timing (AP001-A01-C021) | Five `available_after_*_points` paths declared on completed-shape records only; no surface establishes anchor-time eligibility; rule 2 makes occurrence/origin timestamps provenance, not availability | Producer declaration of emission/availability timing per surface |
| Direction semantics (AP001-A01-C022) | Direction/sign paths declared on all surfaces; no value domain or sign convention declared | Producer declaration of value domains and conventions |
| Score semantics (AP001-A01-C023) | Score/z/efficiency/net_log/var_rate/cabs paths declared; no formula, units, ranges, or thresholds declared | Producer declaration of score construction |
| Recovery semantics (AP001-A01-C024) | Recovery/entry paths declared; no counting rule or window declared | Producer declaration of recovery counting semantics |
| Completion semantics (AP001-A01-C025) | `end_reason` and completion paths declared; no end_reason vocabulary or completion criterion declared; unknown whether every episode completes | Producer declaration of end_reason vocabulary and completion criteria |
| Future-field legality (AP001-A01-C026) | Whether completed-shape records are legal evidence at pre-completion anchors is not established; rules 2 and 4 forbid future-completion cohort definitions | Producer availability declaration plus explicit anchoring rules |
| Payload conformity (AP001-A01-C027) | Stored `payload_drift_burst` strings not inspectable (raw lake DENY); conformity to the declared contract unverified here | Authorized payload samples or a mounted conformity attestation |
| Surface relationship (AP001-A01-C028) | Whether state records are per-tick snapshots, whether completed records are per event or per episode, and whether `emitted_events[]` accumulates are unknown; declared structure does not select among readings | Producer declaration of record-production relationship |
| Scale parameter semantics (AP001-A01-C029) | `authority.weak_online_scale_s`, `authority.strong_scale_s`, and milestone groupings declared structurally; units, values, and effect undeclared | Producer declaration of scale parameters and milestone meaning |

The candidate's `unresolved_questions` carries all nineteen UNRESOLVED claims: the twelve
producer-authority topics above plus the seven provenance-pending items from §4, each keyed to its
claim ID. Consolidated producer-authority asks: lifecycle/state vocabulary and transitions; episode
identity construction, uniqueness, and persistence; per-surface availability timing; direction and
sign conventions; score construction; recovery and completion rules; scale parameter meaning; and
the state-to-completed record relationship.

## 7. Guardrails applied

- The bundle's structural declarations were treated as exactly that; none was upgraded to lifecycle
  or causal meaning. In particular, the five `available_after_*` paths, the milestone names, and
  the scale-parameter names were recorded verbatim without assigning them timing, transition, or
  unit semantics.
- Occurrence/origin-style paths (`origin_*`, `*_ts`, `*_idx`) are recorded as declared field paths
  only; rule 2 confines any such timestamps to provenance regardless of semantics later
  established.
- The established structure (§3) is compatible with several mutually exclusive lifecycle readings;
  no reading was selected (AP001-A01-C028).
- Null/absent evidence was retained rather than smoothed over: uncitable citations were dropped and
  disclosed (§4) rather than relabeled onto authorized source IDs, and non-inspectability of
  payload contents is recorded as such (AP001-A01-C027).

## 8. Structural validation performed

Independent checks were run against the rewritten artifacts before declaring completion:

- `artifact_type` is present and exactly `AUTHORITY_CANDIDATE`; the candidate has exactly the seven
  required top-level fields.
- `claims` is a non-empty array (29 claims); `subjects`, `unresolved_questions`, and
  `sources_consulted` are present and non-empty.
- Every claim contains exactly the ten required semantic fields and no others.
- All claim IDs match the required grammar (`AP001-A01-C001` … `AP001-A01-C029`) and are unique;
  no IDs outside that grammar remain.
- Every `claim` field is non-empty.
- Every status is within {AUTHORITATIVE, PROVISIONAL, UNRESOLVED} (9/1/19; no relationship-style
  status appears anywhere).
- Every AUTHORITATIVE claim has a non-empty `authority_evidence` array of objects, each containing
  non-empty source_id, relative_path, locator, evidence_type, and supports.
- Every `source_id` on every evidence object, counterevidence object, and consulted source is one of
  the three authorized values (`xauusd.wave1.ap001`, `protocol.research_rules`,
  `assignment.ap-001_a-01`); no repository path or surface name is used as a source_id anywhere, and
  no unsupported evidence was relabeled.
- Every claim ID cited in this Markdown exists in the candidate's claims array, and every
  UNRESOLVED claim is represented in `unresolved_questions`.
- Exactly two files remain in the workspace: `authority_candidate.json` and `authority_review.md`.
  No Git commits, archives, or extra artifacts were written, and no personal, runtime, or
  implementation identity appears in either artifact.

Per the submission lifecycle, this agent stops here and leaves the two files for host-side
validation; publication remains a separate host-side action.
