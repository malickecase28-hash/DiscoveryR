# Authority Review — BG-001 / A-02 — bar_lineage

Run: Wave-1 Authority Run V2 · Phase: AUTHORITY · Access profile: AUTHORITY
Authority source: `xauusd.wave1.bg001` · Companion artifact: `authority_candidate.json` (artifact_type AUTHORITY_CANDIDATE, 27 claims BG001-A02-C001 to BG001-A02-C027)

## 1. Scope, evidence base, and provenance model

Read exactly the assigned surfaces: the authority bundle (`authority/authority.json`, `authority/bundle_manifest.json`), the input manifest, `repository/protocol/`, `repository/contracts/`, `repository/registry/`, `repository/instruments/XAUUSD/`, and the assignment `repository/agent_harness/assignments/BG-001/A-02.json`. Raw lake, prior reports, peer workspaces, legacy material, and credentials were not accessed.

Provenance model applied in this revision: every `authority_evidence` entry is anchored to one of the three authorized source IDs only — `xauusd.wave1.bg001` (the authority bundle), `protocol.research_rules`, and `assignment.bg-001_a-02`. No source_id was relabeled onto evidence it does not support. Where a claim's grounding surface (the physical source inventory, the detector registry, the repository contracts) is an authorized reading surface under the assignment but carries no authorized source ID for publication, the claim is held UNRESOLVED with its content preserved unchanged, and the limitation is recorded in the claim's notes. Scientific conclusions are unchanged; this revision changes provenance anchoring and status only.

Integrity check performed against mounted bytes (BG001-A02-C001): sha256 of `authority/authority.json` = `92dc60d4…797288`, equal to `bundle_manifest.json` `generated_files[0].sha256` and `bundle_content_identity`; the assignment authorizes source `xauusd.wave1.bg001` for this role. The additional cross-check of this hash against the input manifest that declares authority source identities is not asserted in the candidate, because that surface has no authorized source ID.

Discipline: fail-closed. No detector lineage is asserted beyond what mounted surfaces directly declare. The publication contract's top-level shape carries no graph object; semantically the graph is empty — no derivation edge is declared or established (BG001-A02-C019) — and every unresolved relationship is recorded in claim notes as `relation = UNRESOLVED_RELATIONSHIP` (a relation marker, never a claim status).

## 2. AUTHORITATIVE under authorized provenance (7 claims)

**Bundle identity (BG001-A02-C001).** The mounted authority bundle for `xauusd.wave1.bg001` is internally identity-consistent (computed sha256 equals the bundle manifest's declared file hash and bundle content identity), and the assignment authorizes the source.

**Canonical bar schema (BG001-A02-C002, BG001-A02-C003, BG001-A02-C004).** The bundle selects one schema, `trinity.research.bars.canonical.v1` ("Canonical replay-aggregated bars schema stored in Parquet lake"), with exactly 8 non-nullable columns (`instrument_id` UTF-8; `start_ts_ms`, `end_ts_ms` int64; `open`, `high`, `low`, `close`, `activity_volume` float64) and hive partitions `timeframe`, `year`, `month`. The bundle establishes canonical bar physical-schema facts only; it establishes no lineage. The description "replay-aggregated" is not treated as a derivation fact (BG001-A02-C019).

**Bundle lineage flag (BG001-A02-C005).** The bundle's status array is empty and its unresolved array contains exactly `lineage: UNRESOLVED_PENDING_SOURCE_APPROVAL`.

**Protocol rules (direct text).** Rule 11, the general fake-confluence rule: "If detector B derives from A, A+B is not automatically independent confluence" (BG001-A02-C024). Rules 1–4 on research unit, single causal information clock with occurrence timestamps as provenance, native scale preservation, and lifecycle-first cohorting (BG001-A02-C026).

## 3. Preserved findings held UNRESOLVED for provenance (content unchanged)

These findings were previously asserted from repository surfaces that the assignment authorizes for reading but that have no authorized source ID for publication. The findings are preserved verbatim as claim content; their status is UNRESOLVED, and each claim's notes name the grounding surface and the downgrade reason. No evidence was relabeled.

- **Physical lake layout (BG001-A02-C007).** 8 XAUUSD sources — bar scales 15s, 30s, 1m, 5m, 15m, 1h, 4h plus one tick source — each with 6 parquet parts (48 parts total), every part schema-verified, exact per-part row counts declared. Declared per-part row counts, summed arithmetically per source (sums are arithmetic aggregations of declared values, not lineage evidence):

| scale | parts | declared rows (sum) |
|---|---|---|
| 15s | 6 | 1,320,835 |
| 30s | 6 | 660,859 |
| 1m | 6 | 330,458 |
| 5m | 6 | 66,147 |
| 15m | 6 | 22,050 |
| 1h | 6 | 5,515 |
| 4h | 6 | 1,464 |
| tick | 6 | 68,445,659 |

- **Physical bar schema (BG001-A02-C008).** All 7 bar sources declare one non-payload physical schema: `instrument_id`, `bar_open_ts`, `bar_close_ts`, `open`, `high`, `low`, `close`, `volume` (all non-nullable), plus 16 nullable LargeUtf8 payload columns; physical time coordinate fields `bar_open_ts`, `bar_close_ts`; no receipt-time or sequence field.
- **Physical tick schema (BG001-A02-C009).** `event_ts_ns`, `received_ts_ns`, `source_sequence`, `bid`, `ask` (non-nullable) plus 7 nullable LargeUtf8 payload columns; time coordinate fields `event_ts_ns`, `received_ts_ns` — the tick surface physically distinguishes occurrence time from receipt time, the bar surface does not.
- **Registry (BG001-A02-C010).** Exactly 23 detectors: 16 bar-domain (swings, raw3, range, fvg, bos_choch, order_blocks, local_structure, volume_by_time, structural_liquidity, dealing_range, micro_liquidity, micro_liquidity_context, volume_trend, auction_context, time_context, vwma_atr) with surface `bar` and explicit native scales {15s, 30s, 1m, 5m, 15m, 1h, 4h}; 7 tick-domain (drift_burst, spread_state, quote_dynamics, quote_arrival, micro_volatility, feed_health, quote_pressure) with surface `tick` and native scale kind tick. The bar scale set coincides exactly with the contract `BarScale` enum and the inventory's bar source scales.
- **Payload column correspondences (BG001-A02-C011, BG001-A02-C012).** For each of the 16 bar detectors a same-named column `payload_<detector_id>` (LargeUtf8, nullable) exists in every schema-verified part of every bar scale, one-to-one with registry detector_ids; likewise for the 7 tick detectors in tick parts. Correspondences were verified at read time by exact set comparison.
- **Co-location and nullability (BG001-A02-C013).** Payload columns are co-located with base bar columns in the same parquet parts; the inventory declares no separate detector-output store; nullability means no payload value is guaranteed on any row.
- **Output-surface identification (BG001-A02-C014).** Identifying `payload_<detector_id>` as the physical output surface of detector `<detector_id>` rested on the naming correspondence plus surface agreement with `known_surfaces`, held PROVISIONAL in the prior revision; it is now UNRESOLVED because its grounding surfaces cannot be cited under authorized source IDs. No producer manifest is obtainable (raw lake access denied).
- **Registry self-declarations (BG001-A02-C015).** Every entry declares `availability_semantics` "UNRESOLVED; authoritative manifest surface only.", `semantic_status` UNRESOLVED, and empty `derives_from`, `lifecycle_vocabulary`, and `roles`.
- **Contract vocabularies (BG001-A02-C022).** The registry contract defines role and semantic-status vocabularies, but no entry instantiates any role or lifecycle term.

## 4. UNRESOLVED lineage (unchanged conclusions; relation = UNRESOLVED_RELATIONSHIP in claim notes)

- **Canonical ↔ physical mapping (BG001-A02-C006).** Canonical column names (`start_ts_ms`, `end_ts_ms`, `activity_volume`; hive partitions) differ from lake physical names (`bar_open_ts`, `bar_close_ts`, `volume`; part directories). No mounted surface declares a mapping or equivalence.
- **Producer raw inputs (BG001-A02-C017).** Declared by no mounted surface for any detector; raw lake access is denied and no producer manifest is among the authorized surfaces. Co-location is not read-evidence.
- **Upstream detector inputs (BG001-A02-C018).** `derives_from` is empty for all 23 entries — only that no registry edge is currently declared; it does not prove input independence. All upstream-input relationships are UNRESOLVED.
- **Derivation relationships (BG001-A02-C019).** None established between any detector pair or between bar and tick objects. The description "replay-aggregated" does not establish aggregation inputs. Any downstream graph built from this candidate must carry zero edges.
- **Object/source identity (BG001-A02-C020).** Unresolved beyond declared naming correspondences: canonical ↔ physical (BG001-A02-C006), detector ↔ payload column beyond the name match (BG001-A02-C014), and the lake manifests `manifest.json` / `payload_manifest.json` referenced by the instrument configuration are not mounted, so the inventory's declared manifest identities cannot be tied to any manifest content.
- **Availability and known time (BG001-A02-C016, BG001-A02-C021).** Which rows carry non-null payload values, which time coordinate governs a payload value's known time, replay-time versus storage-time computation, and the relation of tick receipt time to bar payload availability are all undeclared.
- **Episode identity (BG001-A02-C023).** Where anchor/lifecycle semantics must be declared is defined; no surface instantiates any episode or lifecycle-state vocabulary (the absence record itself is BG001-A02-C022).
- **Specific detector-pair independence (BG001-A02-C025).** For every pair of the detectors, independence and dependence are both unresolved. No pair-specific independence warning may be issued on this evidence base; discovery is not confirmation and a method challenge is not replication (protocol rules 6–7).

The open questions are enumerated as `unresolved_questions` UQ-001 to UQ-009 in the candidate, each tied to the claims that carry them; UQ-009 asks under which authorized source ID the currently uncitable repository declarations can be published.

## 5. Blocker

**BG-001 lineage-dependent E3 research is blocked pending approved lineage authority.** The approved authority bundle itself carries `lineage: UNRESOLVED_PENDING_SOURCE_APPROVAL` (BG001-A02-C005), and no producer-input, upstream-input, or derivation relationship is established on the mounted evidence (BG001-A02-C017, BG001-A02-C018, BG001-A02-C019). E3 CROSS_DOMAIN exposure work (protocol rule 5) that depends on derivation relationships — including any tick-to-bar cross-domain lineage — cannot be advanced until lineage authority is approved. This is a procedural block, not a negative finding. (BG001-A02-C027)

## 6. Structural and provenance compliance

The candidate carries exactly the required top-level shape: `artifact_type` ("AUTHORITY_CANDIDATE"), `program_id` ("BG-001"), `role_id` ("A-02"), `subjects`, `claims`, `unresolved_questions`, `sources_consulted`. Every claim carries exactly the ten required semantic fields; IDs use the `BG001-A02-C001` grammar and are unique; statuses are limited to AUTHORITATIVE (7), PROVISIONAL (0), UNRESOLVED (20). Every `authority_evidence` entry is a non-empty object with `source_id`, `relative_path`, `locator`, `evidence_type`, `supports`, and every `source_id` is one of the three authorized IDs (`xauusd.wave1.bg001`, `protocol.research_rules`, `assignment.bg-001_a-02`), which are also the only entries in `sources_consulted`. Ten claims whose grounding surfaces lack authorized source IDs were downgraded to UNRESOLVED with content preserved (BG001-A02-C007, BG001-A02-C008, BG001-A02-C009, BG001-A02-C010, BG001-A02-C011, BG001-A02-C012, BG001-A02-C013, BG001-A02-C014, BG001-A02-C015, BG001-A02-C022); no evidence was relabeled and no provenance was invented. `UNRESOLVED_RELATIONSHIP` appears only as a relation marker in claim notes, never as a status. No derivation edges are declared; no researcher, runtime, or implementation identity appears in either artifact; no commits or archives were created.
