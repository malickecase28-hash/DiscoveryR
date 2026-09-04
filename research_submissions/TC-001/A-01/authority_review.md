# Authority review — TC-001 / A-01 / tick_conditioning

Wave-1 Authority Run V2 · phase AUTHORITY · instrument XAUUSD · authority source `xauusd.wave1.tc001`

## Correction note (provenance-only; science unchanged)

This revision follows a provenance-only correction at the publication boundary. The
JSON structure is unchanged from the accepted shape; the only change is provenance
labeling. Evidence `source_id` values are now restricted to the three authorized
source IDs — `xauusd.wave1.tc001` (the authority bundle), `protocol.research_rules`,
and `assignment.tc-001_a-01` — and a source_id was substituted only where the cited
source actually supports the evidence. Claims whose support rested on registry,
instrument-inventory, governance-contract, or program-manifest surfaces — none of
which can be represented by an authorized source ID — were marked UNRESOLVED rather
than relabeled; their claim texts, conclusions, and prior findings are preserved
verbatim, with the downgrade recorded in each claim's `notes`. No evidence was
invented. The no-row-keying conclusion and all unresolved causal timing semantics are
unchanged. Status distribution is now AUTHORITATIVE 6, PROVISIONAL 0, UNRESOLVED 20.

## Assignment and scope

The assignment was to reconstruct tick-conditioning lineage and availability semantics
independently, using only the mounted V2 evidence. Raw lake access was denied, peer
visibility was denied, and no prior reports, legacy material, or host material outside
the authorized surfaces were consulted. The only authority source authorized for this
pass, `xauusd.wave1.tc001`, establishes **declared serialization structure** for six
detector families. Per the binding program constraint, that structure establishes
record-variant names and field-path names only; no semantic category (event, state,
transition, baseline, direction, reset, persistence, history-dependence, causal
availability) may be inferred from any name, and no row-keying hypothesis was formed:
physical co-location of payload columns on a tick row does not establish occurrence
time or availability time for payload records.

## Surfaces consulted

All 13 authorized-corpus surfaces were read in full: the authority bundle (2 files),
the program manifest, both protocol documents, all four contract schemas, the
registry, both XAUUSD instrument files, and the assignment. Under the provenance-only
correction, only surfaces mappable to the three authorized source IDs may anchor
evidence in the candidate: the authority bundle under `xauusd.wave1.tc001`, the
research rules under `protocol.research_rules`, and the assignment under
`assignment.tc-001_a-01`. The remaining surfaces (program manifest, agent access
protocol, governance schemas, registry, instrument files) were consulted and inform
the findings, but cannot anchor evidence in this artifact; claims that depended on
them were marked UNRESOLVED with the dependency recorded in their notes. The raw
lake, the payload manifests, and all parquet data were not accessed.

## Method and status discipline

- One flat `claims` array; one declarative sentence per claim; structured evidence
  objects (`source_id`, `relative_path`, `locator`, `evidence_type`, `supports`) on
  every claim that retains provenance.
- `AUTHORITATIVE` (6 claims) only where the claim is fully supported by the authority
  bundle (and, for source authorization, the assignment).
- `PROVISIONAL` (0 claims) — the two former corpus-absence claims were downgraded to
  UNRESOLVED because their scope spans surfaces not representable by authorized
  source IDs.
- `UNRESOLVED` (20 claims) for every semantic question not directly established and
  for every claim whose prior support is not provenance-representable. No
  `UNRESOLVED_RELATIONSHIP` status appears.
- All record-variant and field-path names were treated as opaque identifiers. Where a
  name comparison is reported, the claim is explicitly limited to string equality.

## What is established (AUTHORITATIVE, provenance-anchored to the authority source)

1. **The contract and its kind.** Source `xauusd.wave1.tc001` selects
   `trinity.research.tick-payload-contract.v1` with contract_source
   `native_detector_serialization_contract` (TC001-A01-C001; the assignment declares
   the source id authorized). The authority record's status is
   `DECLARED_SERIALIZATION_CONTRACT_AVAILABLE`, its unresolved list is empty, and its
   provenance is eight pointers into `payload_manifest.json` at sha256 `78d5fc…3eed`
   (TC001-A01-C002). The selected content is exactly family → variant → field-path
   name lists; no vocabularies, thresholds, windows, clocks, keying, or semantic
   categories are declared (TC001-A01-C003).
2. **Declared structure.** Six families, 26 record variants, with exact variant-name
   sets (TC001-A01-C005, TC001-A01-C006) and exact per-family field-path unions —
   feed_health 15, micro_volatility 5, quote_arrival 3, quote_dynamics 22,
   quote_pressure 6, spread_state 5 distinct paths (TC001-A01-C007).

## Downgraded to UNRESOLVED for provenance reasons (findings preserved)

These claims state findings as read; their prior evidencing surfaces cannot be
represented by an authorized source ID, so they are carried as UNRESOLVED with the
dependency recorded in notes rather than relabeled:

- **TC001-A01-C004** (was PROVISIONAL) — corpus-wide absence of occurrence time,
  availability time, semantic category, or row-keying assignments; only the
  authority-selection absence remains evidenced.
- **TC001-A01-C008** (was AUTHORITATIVE) — declared/physical name comparisons; only
  the declared-names half remains evidenced.
- **TC001-A01-C009** (was AUTHORITATIVE) — tick surface physical schema; no
  provenance-compatible evidence remains.
- **TC001-A01-C010** (was AUTHORITATIVE) — payload column / registry detector_id
  correspondence; only the contract-family-exclusion half remains evidenced.
- **TC001-A01-C011** (was AUTHORITATIVE) — registry posture; no provenance-compatible
  evidence remains.
- **TC001-A01-C012, TC001-A01-C013** (were AUTHORITATIVE) — provenance identity
  chains; only the authority-side values remain evidenced.
- **TC001-A01-C015** (was AUTHORITATIVE) — the registry's literal availability string;
  no provenance-compatible evidence remains.
- **TC001-A01-C022** (was AUTHORITATIVE) — absence of declared derivation edges; only
  the authority-selection absence remains evidenced.
- **TC001-A01-C025** (was PROVISIONAL) — absence of experiment/run/knowledge
  artifacts; only the assignment surface enumeration remains evidenced.

## What remains unresolved (all nine mandatory dimensions)

| Dimension | Claims | Blocker |
|---|---|---|
| Payload occurrence clock | TC001-A01-C014 | No surface maps any record variant to an occurrence time; `event_ts_ns`/`received_ts_ns` are row-level physical coordinates only. |
| Detector availability clock | TC001-A01-C015, TC001-A01-C016 | Availability semantics are undeclared; no visibility rule is declared for payload contents at any anchor time. |
| Reset | TC001-A01-C017 | No reset semantics declared; `market_state`/`authority_hash` and `from`/`to` are names only. |
| Persistence | TC001-A01-C018 | No emission or persistence rule declared; nullable payload columns are a storage fact, not an emission rule. |
| Direction | TC001-A01-C019 | Directional naming and `direction_pressure`/`bid_move`/`ask_move` are undeclared as semantics; no value vocabularies exist for `motion_class`, `spread_effect`, `semantics`, `direction_pressure`, `market_state`, `from`, `to`. |
| Baseline | TC001-A01-C020 | `baseline_*` and `median_*` field paths are names; reference windows, update rules, and causal status undeclared. |
| History | TC001-A01-C021 | `prior_*` and `from`/`to` field paths are names; history dependence is not established in either direction. |
| Dependencies | TC001-A01-C022, TC001-A01-C023, TC001-A01-C024 | Cross-family field-name sharing (e.g. `ratio` in five of six families) is not derivation; per protocol rule 11 absence of declared derivation is not evidence of independence. The `payload_drift_burst` column's relationship to the six families is undeclared. |
| Future mapping | TC001-A01-C025, TC001-A01-C026 | No anchor, eligible-context, or lifecycle mapping is declared; the registry's lifecycle vocabulary is empty for all tick detectors. |

These nine dimensions are mirrored as `unresolved_questions` UQ-001 through UQ-009 in
the candidate, each linked to its claims.

## Lineage reconstruction — result

The only lineage structure declared in the authorized corpus is the empty one: no
registered detector declares a `derives_from` edge, the authority source declares the
six families side by side without inter-family references, and no other surface
supplies lineage information. What can be stated is the declared co-surface structure
— seven tick-domain detectors, each with a payload column on the tick rows, six of
them with declared serialization structure of 26 record variants — plus name-sharing
facts. Whether any of these detectors conditions on another, what state (if any) they
maintain, and whether they are independent for confluence purposes are all
unresolved. The subject id `tick_conditioning` is the research subject of this pass;
no registered detector carries that id.

## Availability semantics — result

Availability semantics are undeclared across the corpus, and nothing in the authority
source or elsewhere contradicts or refines that. Both the occurrence clock and the
availability clock of payload records remain undeclared; the two physical time
coordinate fields on tick rows are schema facts that do not resolve either question.

## Validation performed (pre-submission)

- `artifact_type` present and exactly `AUTHORITY_CANDIDATE`; top-level keys exactly
  the seven required.
- `claims` a non-empty array (26 claims); all claim IDs match the `TC001-A01-C###`
  grammar and are unique; every claim has a non-empty `claim` field and the full
  ten-field set and nothing more.
- Every status is one of `AUTHORITATIVE`, `PROVISIONAL`, `UNRESOLVED`; every
  AUTHORITATIVE claim has a non-empty `authority_evidence` array whose objects each
  carry `source_id`, `relative_path`, `locator`, `evidence_type`, `supports`.
- Every `source_id` in the artifact — claims and `sources_consulted` alike — is one
  of the three authorized IDs; no path-like or unauthorized source_id remains.
- Claim counts, variant names, and field-path unions were re-verified
  programmatically against `authority/authority.json` for the claims that remain
  AUTHORITATIVE.
- This review cites only claim IDs that exist in the candidate.
- Exactly two files remain in the workspace: `authority_candidate.json` and
  `authority_review.md`. No researcher identity, runtime identity, or implementation
  identity appears in either artifact; no Git commits or archives were created.
