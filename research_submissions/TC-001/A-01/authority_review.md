# TC-001 · A-01 · Authority Review — Tick-Conditioning Families (corrected universal shape)

**Verdict: TC-001 behavioral conditioning research is BLOCKED pending approved authority.**

| Item | Value |
|---|---|
| Program | TC-001 |
| Role | A-01 (phase AUTHORITY, access profile AUTHORITY) |
| Detector | tick_conditioning |
| Subject | tick-conditioning families: feed_health, micro_volatility, quote_arrival, quote_dynamics, quote_pressure, spread_state |
| Date | 2026-09-04 |
| Claim count | 112 claims, TC001-A01-C001 through TC001-A01-C112, sequential without gaps |
| Unresolved questions | 90 structured entries in the JSON artifact |
| Machine-readable counterpart | authority_candidate.json |

## 1. Scope and method

This is blind authority research only. The following were not performed: drift interaction study (payload_drift_burst is out of scope), predictive testing, behavioral statistics, strategy research, raw-lake access (DENY honored — no lake file, manifest, or parquet data was accessed), network access, peer inspection (DENY honored), and shared-file edits. All writes are confined to the assigned private workspace.

**Correction pass.** The artifact was rebuilt into the required universal shape: a single flat `claims` array. All 111 prior claims (15 program-level, 96 family-level) were flattened and renumbered sequentially TC001-A01-C001 through TC001-A01-C111 with no gaps; the former block numbering (C101–C196) is superseded, which resolves the earlier mismatch between the reported claim count and the claim-ID range. The BLOCKED conclusion is preserved as claim TC001-A01-C112. Each claim records the former claim ID it was rebuilt from in its `notes`. No statement, status, or evidence object was changed; new per-claim fields (`counterevidence`, `future_information_risk`, `dependency_implications`, `notes`) were added, and the former output matrix's material claims are represented inside the claims (see section 6).

Status vocabulary: **AUTHORITATIVE** (directly supported by a cited declaration on an authorized surface), **PROVISIONAL** (candidate interpretation with physical support, explicitly unconfirmed), **UNRESOLVED** (not resolvable from any authorized surface; requires producer authority). Every claim carries structured `authority_evidence` objects (source_id, relative_path, locator, evidence_type, supports); `counterevidence` uses the same object shape and is empty where no counterevidence exists.

## 2. Authority surface survey and sources consulted

| Surface | State | Consequence |
|---|---|---|
| authority/ | present — 12 lake-storage and research-governance schemas | none declares tick-family record semantics; lake_manifest_v1, dataset_manifest_v1, feature_generic_v1 cited for storage-shape and dependency-closure findings |
| contracts/ | present — 4 governance contracts | define the *required shape* of detector authority and experiments; carry no per-family content |
| registry/ | **empty** | zero detector registry entries; every required registry field is unavailable for all six families |
| protocol/RESEARCH_RULES.md | present | 14 research rules; causal-clock, lifecycle-first, normalization, and dependency constraints cited per claim |
| instruments/XAUUSD/instrument_config.json | present | lake pointer configuration only; manifests not accessed |
| instruments/XAUUSD/source_inventory.json | present | physical schema inventory for 8 declared sources; the schema-bearing surface used under the designated xauusd.schemas source id |
| agent_harness/assignments/TC-001/A-01.json | absent | declared in authorized_repository_surfaces, not present on this host |

The assignment designates `xauusd.schemas` as the authority source. No artifact under that identity exists as a standalone producer authority; the only schema-bearing surface is the physical inventory, which states column names, types, nullability, part structure, and row counts — and nothing about record meaning. The JSON artifact's `sources_consulted` array lists all consulted sources with relative paths, roles, and authority scopes.

## 3. Physical layer (authoritative)

Tick source (`$.sources[7]`): six parts, schema verified on all six, 68,445,659 declared rows.

| Column | Type | Nullable | Classification |
|---|---|---|---|
| event_ts_ns | Int64 | no | PHYSICALLY_PRESENT_ON_TICK_ROW |
| received_ts_ns | Int64 | no | PHYSICALLY_PRESENT_ON_TICK_ROW |
| source_sequence | Int64 | no | PHYSICALLY_PRESENT_ON_TICK_ROW |
| bid | Float64 | no | PHYSICALLY_PRESENT_ON_TICK_ROW |
| ask | Float64 | no | PHYSICALLY_PRESENT_ON_TICK_ROW |
| payload_drift_burst | LargeUtf8 | yes | out of scope (drift family) |
| payload_spread_state | LargeUtf8 | yes | family carrier — spread_state |
| payload_quote_dynamics | LargeUtf8 | yes | family carrier — quote_dynamics |
| payload_quote_arrival | LargeUtf8 | yes | family carrier — quote_arrival |
| payload_micro_volatility | LargeUtf8 | yes | family carrier — micro_volatility |
| payload_feed_health | LargeUtf8 | yes | family carrier — feed_health |
| payload_quote_pressure | LargeUtf8 | yes | family carrier — quote_pressure |

Findings (C002–C005): the six family payload columns are declared only on the tick source (all seven bar scales carry a disjoint 16-column payload set), so the families' native scale is tick; the five physical columns are non-nullable row coordinates and quote fields; all payload columns are nullable text, so per-row absence is physically possible. The five physical columns are classified PHYSICALLY_PRESENT_ON_TICK_ROW only — no SAFE_AT_DETECTOR_STATE classification is asserted anywhere in this work.

## 4. Program-level findings (TC001-A01-C001 – TC001-A01-C015)

- **TC001-A01-C001 · authority_source_availability — AUTHORITATIVE.** The assignment designates xauusd.schemas as the authority source. No artifact under that identity exists on any authorized surface: the authority and contract directories contain lake-storage and research-governance schemas only, the registry directory is empty, the agent_harness assignment file is absent from this host, and no producer record authority for any tick-conditioning family exists anywhere on the authorized surfaces.
- **TC001-A01-C002 · tick_native_scale — AUTHORITATIVE.** The six tick-conditioning family payload columns (payload_feed_health, payload_micro_volatility, payload_quote_arrival, payload_quote_dynamics, payload_quote_pressure, payload_spread_state) are declared only on the Tick source. All seven declared bar scales (15m, 15s, 1h, 1m, 30s, 4h, 5m) carry a disjoint 16-column payload set containing none of them: the native scale of these families is tick, and native scale remains attached under the shared causal clock. Counterevidence on file: authority.dataset_manifest_v1.
- **TC001-A01-C003 · physical_row_fields — AUTHORITATIVE.** The tick row physically carries exactly five non-payload columns: event_ts_ns (Int64, non-nullable), received_ts_ns (Int64, non-nullable), source_sequence (Int64, non-nullable), bid (Float64, non-nullable), ask (Float64, non-nullable). event_ts_ns and received_ts_ns are the declared physical time coordinate fields of the source.
- **TC001-A01-C004 · payload_nullability — AUTHORITATIVE.** Each of the six family payload columns is nullable LargeUtf8, so a tick row can physically lack a family's payload value; the meaning of such absence is unresolved (see per-family dimension 16). Seven payload columns are declared on the tick source in total: the six families plus payload_drift_burst.
- **TC001-A01-C005 · detector_state_classification — AUTHORITATIVE.** The five physical tick columns are classified PHYSICALLY_PRESENT_ON_TICK_ROW only. SAFE_AT_DETECTOR_STATE classification is withheld for event_ts_ns, received_ts_ns, source_sequence, bid, and ask; lawful detector-state use remains unresolved for every family, and no detector-state-safe field is asserted anywhere in this artifact.
- **TC001-A01-C006 · causal_information_clock — AUTHORITATIVE.** General causal rule (authoritative): at anchor time t only information actually available by t is eligible; occurrence and origin timestamps remain provenance. Lifecycle-first rule (authoritative): future completion or termination cannot define an earlier-state cohort; completed may be a later outcome or later anchor. Both constrain all six families and all future-field questions.
- **TC001-A01-C007 · normalization_and_dependency_constraints — AUTHORITATIVE.** Normalization rule (authoritative): any normalization must preserve raw value, normalized value, and basis - unverified for every family because no normalization declaration exists. Dependency rule (authoritative): derives_from does not by itself establish independence - and derives_from declarations are entirely unavailable here (registry empty), so neither dependency nor independence is assertable for any family.
- **TC001-A01-C008 · family_payload_identification — AUTHORITATIVE.** Family-to-payload-column identification rests on exact name correspondence (payload_<family>) at the physical layer only; it confers no semantic authority over record content, which remains undeclared text. Counterevidence on file: authority.dataset_manifest_v1.
- **TC001-A01-C009 · row_keying_hypothesis — PROVISIONAL.** Row-keying hypothesis (candidate, unconfirmed): payload content on a tick row most plausibly keys to that row's event time, because the producer emits the payload onto the tick row itself. This is a candidate interpretation with physical support only; it is not producer authority, must not be treated as an availability rule, and requires explicit confirmation before any use. Counterevidence on file: repository.surfaces.
- **TC001-A01-C010 · raw_lake_access_scope — AUTHORITATIVE.** Raw-lake access is denied by the assignment and was not attempted; the instrument config declares the lake-root environment variable and the manifest filenames, but manifest contents were not accessed. Only the manifest identity hash field names declared in source_inventory are known to this role.
- **TC001-A01-C011 · drift_scope_exclusion — AUTHORITATIVE.** payload_drift_burst is declared on the tick source and belongs to the drift family, which is out of scope for this assignment (drift interaction study prohibited); no drift claims, semantics, or dependencies are asserted in this artifact.
- **TC001-A01-C012 · detector_registry_authority_gap — AUTHORITATIVE.** The detector registry entry contract requires every registered detector to declare exactly: availability_semantics, derives_from, description, detector_id, domain, known_surfaces, lifecycle_vocabulary, name, native_scales, roles, semantic_status (additionalProperties false). The registry surface contains zero entries, so every one of these authority fields is unavailable for all six families. The contract is authoritative about the shape of detector authority; no family satisfies it, and the families' presence as payload columns does not substitute for registry authority.
- **TC001-A01-C013 · experiment_contract_requirements — AUTHORITATIVE.** The experiment contract requires each experiment to declare an anchor (detector_id, lifecycle_state, anchor_time_semantics), an anchor_time_rule (semantics and source), an exposure_class, eligible_context (allowed and requested detectors), native_scale_scope, normalization_basis, population, controls, and data scopes. None of these can be lawfully populated for the six families from available authority, because lifecycle_state, anchor_time_semantics, and normalization_basis all depend on producer authority that does not exist on any authorized surface.
- **TC001-A01-C014 · dependency_closure_mechanism — AUTHORITATIVE.** Lake run manifests are the declared mechanism for run-level detector participation and dependency closure: the lake manifest schema declares tick_detectors and bar_detector_dependency_closure, and the dataset manifest schema declares tick_feature-kind datasets with detector and payload_encoding fields. Actual manifests for this instrument are under raw-lake denial, so detector participation and dependency closure for the XAUUSD tick source are unobtainable from authorized surfaces.
- **TC001-A01-C015 · generic_feature_storage_form — AUTHORITATIVE.** The lake's generic detector-feature storage form declares event_time_ns, detector, feature_type, payload_json (UTF-8, non-null), feature_hash, and config_hash per feature record. This is a lake storage convention, not authority over the tick-embedded payload columns of the six families; whether the six families' outputs also exist as separate tick_feature datasets is undeclared and unresolvable under raw-lake denial, and the internal encoding of the tick-embedded payload text is declared by no authorized surface.

## 5. Family dossiers

### 5.1 feed_health — payload_feed_health (claims TC001-A01-C016 – TC001-A01-C031)

**Output surface (AUTHORITATIVE, TC001-A01-C016).** The declared physical output surface of the feed_health family is the single nullable LargeUtf8 column payload_feed_health embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; whether the family also emits dataset-level surfaces is undeclared and unresolvable here because run and dataset manifests are under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- event_ts_ns and received_ts_ns are both declared non-nullable physical time coordinates, so an event-time versus received-time comparison is computable on every tick row from declared fields; this is capability, not producer fact.
- source_sequence is declared non-nullable Int64, so duplicate/gap/regression computation across consecutive rows is computable from declared fields; this is capability, not producer fact.

Dimensions:

- **TC001-A01-C016 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the feed_health family is the single nullable LargeUtf8 column payload_feed_health embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; whether the family also emits dataset-level surfaces is undeclared and unresolvable here because run and dataset manifests are under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_feed_health, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C017 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names feed-integrity semantics (clock inversion, event-time gap/regression, expected closure, latency spike, sequence duplicate/gap/regression, event-time versus received-time), but the designated authority source is absent. The registry contract defines a closed role vocabulary (directional_evidence, state_regime, structural_object, temporal_context, data_quality, lifecycle_state, normalization_reference), yet the registry contains zero entries, so no role is declared for this family.
- **TC001-A01-C018 · record_variants — UNRESOLVED.** Record variants within payload_feed_health are unresolved. The declared carrier is one nullable text value per tick row, so any variant structure must be encoded inside that value or across rows; the count and shape of variants (for example one record per integrity event versus a per-row status) are declared nowhere on authorized surfaces.
- **TC001-A01-C019 · event_vs_persistent_state — UNRESOLVED.** Whether payload_feed_health carries discrete events (for example a latency-spike or sequence-gap record) or persistent state (for example a standing health classification), or both, is unresolved; no producer authority exists to classify the family.
- **TC001-A01-C020 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records. The only row-physical candidates are event_ts_ns, received_ts_ns, and source_sequence, whose lawful detector-state use is itself unresolved.
- **TC001-A01-C021 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: whether records carry their own occurrence time inside the payload text or inherit the row's event_ts_ns is undeclared. Under the authoritative causal-clock rule the occurrence/origin timestamp remains provenance either way.
- **TC001-A01-C022 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved. availability_semantics is a required field of a detector registry entry, and the registry has no entry for this family; received_ts_ns is a declared non-nullable physical time coordinate and the natural known-at candidate, but no authority declares it as the availability timestamp, and it is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C023 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: whether a new record supersedes, accumulates on, or re-arms an open condition, and how an open condition closes (charter item 'expected closure'), are undeclared. lifecycle_vocabulary is a required registry field with no entry for this family; the lifecycle-first rule constrains any future resolution.
- **TC001-A01-C024 · duration_persistence — UNRESOLVED.** Duration/persistence of feed_health conditions (how long a detected gap, spike, or degraded condition remains chargeable after onset) are unresolved; no persistence rule is declared.
- **TC001-A01-C025 · transitions — UNRESOLVED.** Transition semantics (for example healthy/degraded/failed changes, if such states exist) are unresolved; no transition vocabulary is declared for this family anywhere on authorized surfaces.
- **TC001-A01-C026 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved and may be inapplicable to integrity phenomena; no authority resolves whether any direction-bearing field exists inside payload records.
- **TC001-A01-C027 · normalization_baseline_semantics — UNRESOLVED.** Normalization/baseline semantics (for example a latency value normalized against a baseline) are unresolved. The experiment contract requires an explicit normalization_basis per experiment, and rule 10 requires preserving raw value, normalized value, and basis; neither can be satisfied or verified from available authority.
- **TC001-A01-C028 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: gap or sequence checks plausibly require the prior row's clock and sequence values, but no declaration states which prior-row values the producer requires or retains.
- **TC001-A01-C029 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved. No payload field names are declared anywhere on authorized surfaces, and under the authoritative causal-clock and lifecycle-first rules no field may be treated as future-bearing from its name alone; names such as from, to, transition, completed, peak, fill, end, or duration would not establish future mapping by themselves.
- **TC001-A01-C030 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure (bar_detector_dependency_closure in lake manifests) is inaccessible under raw-lake denial. Absence of a derives_from declaration must not be read as independence (rule 11); the declared dual-clock and sequence columns are physical capability, not evidence of producer use.
- **TC001-A01-C031 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: how the producer represents missing or unsampleable inputs, and what a null payload value on a row means, are undeclared. Nullability of payload_feed_health is authoritative; its meaning is not.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

### 5.2 micro_volatility — payload_micro_volatility (claims TC001-A01-C032 – TC001-A01-C047)

**Output surface (AUTHORITATIVE, TC001-A01-C032).** The declared physical output surface of the micro_volatility family is the single nullable LargeUtf8 column payload_micro_volatility embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- event_ts_ns is declared non-nullable on every tick row, so windowed computation over tick history is physically supported by declared fields; this is capability, not producer fact.

Dimensions:

- **TC001-A01-C032 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the micro_volatility family is the single nullable LargeUtf8 column payload_micro_volatility embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_micro_volatility, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C033 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names burst, compression, transition, short RMS, baseline RMS, ratio, windows, from/to, and baseline-availability semantics, but the designated authority source is absent and the registry contains zero entries, so no purpose or registry role is declared for this family.
- **TC001-A01-C034 · record_variants — UNRESOLVED.** Record variants within payload_micro_volatility are unresolved: charter-named candidate records (burst, compression, transition, short RMS, baseline RMS, ratio) are unresolved as producer facts. The declared carrier is one nullable text value per tick row; variant count and shape are declared nowhere.
- **TC001-A01-C035 · event_vs_persistent_state — UNRESOLVED.** Whether micro_volatility emits discrete events (for example a burst onset) or persistent state (for example a compressed-regime classification), or both, is unresolved; no producer authority exists to classify the family.
- **TC001-A01-C036 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records, and lawful detector-state use of the row-physical fields is itself unresolved.
- **TC001-A01-C037 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: whether the occurrence time is a window onset, a burst tick, or the row's event_ts_ns is undeclared; under the causal-clock rule occurrence and origin timestamps remain provenance either way.
- **TC001-A01-C038 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved: availability_semantics is a required registry field with no entry for this family; no availability timestamp is declared for payload records, and received_ts_ns is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C039 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: whether a new burst supersedes or accumulates on an open one, and any re-arm behavior, are undeclared; lifecycle_vocabulary is a required registry field with no entry for this family.
- **TC001-A01-C040 · duration_persistence — UNRESOLVED.** Duration/persistence are unresolved: the charter item 'windows' is undeclared - neither the short-RMS window nor the baseline-RMS window geometry (length, step, alignment) is declared anywhere on authorized surfaces.
- **TC001-A01-C041 · transitions — UNRESOLVED.** Transition semantics are unresolved: the charter items 'transition' and 'from/to' correspond to no declared vocabulary, and from/to-style names alone do not fix transition semantics or future mapping under the causal-clock and lifecycle-first rules.
- **TC001-A01-C042 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved: whether bursts or compression states are direction-qualified (for example upward/downward or bid-side/ask-side) is undeclared.
- **TC001-A01-C043 · normalization_baseline_semantics — UNRESOLVED.** Normalization/baseline semantics are unresolved: the short-RMS to baseline-RMS ratio construction, the baseline estimation basis, and the charter item 'baseline availability' are undeclared. The experiment contract requires an explicit normalization_basis per experiment, and rule 10 requires preserving raw value, normalized value, and basis; neither can be satisfied from available authority.
- **TC001-A01-C044 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: RMS and baseline computations plausibly require trailing tick history, but the required depth, the retention rule, and any interaction with feed-integrity regions are undeclared.
- **TC001-A01-C045 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved: no payload field names are declared anywhere on authorized surfaces, and no field may be treated as future-bearing from its name alone under the authoritative causal rules.
- **TC001-A01-C046 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure in lake manifests is inaccessible under raw-lake denial. Whether the family consumes its own tick history, another family's outputs, or feed-health exclusions is undeclared; absence of derives_from must not be read as independence (rule 11).
- **TC001-A01-C047 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: warm-up behavior before a baseline exists (charter item 'baseline availability'), representation of insufficient history, and the meaning of a null payload value are all undeclared; nullability of the column is authoritative, its meaning is not.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

### 5.3 quote_arrival — payload_quote_arrival (claims TC001-A01-C048 – TC001-A01-C063)

**Output surface (AUTHORITATIVE, TC001-A01-C048).** The declared physical output surface of the quote_arrival family is the single nullable LargeUtf8 column payload_quote_arrival embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- event_ts_ns is declared non-nullable on every tick row, so interarrival deltas between consecutive rows are computable from declared fields; this is capability, not producer fact.

Dimensions:

- **TC001-A01-C048 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the quote_arrival family is the single nullable LargeUtf8 column payload_quote_arrival embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_arrival, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C049 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names arrival_burst, arrival_gap, interarrival, median baseline, ratio, and event-versus-maintained-state semantics, but the designated authority source is absent and the registry contains zero entries, so no purpose or registry role is declared for this family.
- **TC001-A01-C050 · record_variants — UNRESOLVED.** Record variants within payload_quote_arrival are unresolved: charter-named candidate records (arrival_burst, arrival_gap, interarrival) are unresolved as producer facts; the declared carrier is one nullable text value per tick row.
- **TC001-A01-C051 · event_vs_persistent_state — UNRESOLVED.** The charter explicitly poses event versus maintained state for quote_arrival; without producer authority the classification is unresolved - records could be per-event bursts or gaps or a maintained arrival-regime state, and no authorized surface decides between them.
- **TC001-A01-C052 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records, and lawful detector-state use of the row-physical fields is itself unresolved.
- **TC001-A01-C053 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: which declared clock grounds interarrival measurement (event_ts_ns versus received_ts_ns) is undeclared; under the causal-clock rule occurrence and origin timestamps remain provenance either way.
- **TC001-A01-C054 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved: availability_semantics is a required registry field with no entry for this family; no availability timestamp is declared for payload records, and received_ts_ns is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C055 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: whether an arrival_gap record closes a burst condition, whether a burst counter resets on a gap, and any re-arm behavior are undeclared; lifecycle_vocabulary is a required registry field with no entry for this family.
- **TC001-A01-C056 · duration_persistence — UNRESOLVED.** Duration/persistence are unresolved: gap-duration accounting (open, close, measured interval) and burst persistence are undeclared.
- **TC001-A01-C057 · transitions — UNRESOLVED.** Transition semantics are unresolved: no transition vocabulary is declared for this family, and from/to-style names alone do not fix transition semantics or future mapping under the authoritative causal rules.
- **TC001-A01-C058 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved and may be inapplicable to arrival timing; no authority resolves whether any direction-bearing field exists inside payload records.
- **TC001-A01-C059 · normalization_baseline_semantics — UNRESOLVED.** Normalization/baseline semantics are unresolved: the charter items 'median baseline' and 'ratio' are undeclared - the median construction window, warm-up, and what is ratioed against the median are declared nowhere. The experiment contract requires an explicit normalization_basis per experiment, and rule 10 requires preserving raw value, normalized value, and basis.
- **TC001-A01-C060 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: a median interarrival baseline plausibly requires trailing interarrival history, but the required depth and retention rule are undeclared.
- **TC001-A01-C061 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved: no payload field names are declared anywhere on authorized surfaces, and no field may be treated as future-bearing from its name alone under the authoritative causal rules.
- **TC001-A01-C062 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure is inaccessible under raw-lake denial. Consecutive-row interarrival computation is a declared physical capability of event_ts_ns, not a producer fact; any baseline sharing or feed-health interaction is undeclared; absence of derives_from must not be read as independence (rule 11).
- **TC001-A01-C063 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: first-tick warm-up, behavior across feed-integrity gaps, and the meaning of a null payload value are undeclared; nullability of the column is authoritative, its meaning is not.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

### 5.4 quote_dynamics — payload_quote_dynamics (claims TC001-A01-C064 – TC001-A01-C079)

**Output surface (AUTHORITATIVE, TC001-A01-C064).** The declared physical output surface of the quote_dynamics family is the single nullable LargeUtf8 column payload_quote_dynamics embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- bid and ask are declared non-nullable Float64 columns on every tick row, so bid/ask change detection between consecutive rows is computable from declared fields; this is capability, not producer fact.

Dimensions:

- **TC001-A01-C064 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the quote_dynamics family is the single nullable LargeUtf8 column payload_quote_dynamics embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_dynamics, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C065 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names repricing, asymmetric repricing, acceleration, spread impulses, motion class, bid/ask changes, and a price-mechanics-versus-liquidity distinction, but the designated authority source is absent and the registry contains zero entries, so no purpose or registry role is declared for this family.
- **TC001-A01-C066 · record_variants — UNRESOLVED.** Record variants within payload_quote_dynamics are unresolved: charter-named candidate records (repricing, asymmetric repricing, acceleration, spread impulse, motion class) are unresolved as producer facts; the declared carrier is one nullable text value per tick row.
- **TC001-A01-C067 · event_vs_persistent_state — UNRESOLVED.** Whether quote_dynamics emits discrete events (for example a repricing) or persistent state (for example a standing motion-class classification), or both, is unresolved; the motion-class charter item could be either and no producer authority decides.
- **TC001-A01-C068 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records, and lawful detector-state use of the row-physical fields is itself unresolved.
- **TC001-A01-C069 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: whether records carry their own occurrence time inside the payload text or inherit the row's event_ts_ns is undeclared; under the causal-clock rule occurrence and origin timestamps remain provenance either way.
- **TC001-A01-C070 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved: availability_semantics is a required registry field with no entry for this family; no availability timestamp is declared for payload records, and received_ts_ns is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C071 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: whether a new repricing or acceleration record supersedes an open motion state, and any re-arm behavior, are undeclared; lifecycle_vocabulary is a required registry field with no entry for this family.
- **TC001-A01-C072 · duration_persistence — UNRESOLVED.** Duration/persistence are unresolved: acceleration window geometry and how long a motion class persists across rows are undeclared.
- **TC001-A01-C073 · transitions — UNRESOLVED.** Transition semantics are unresolved: motion-class transitions (if the class exists) have no declared vocabulary, and from/to-style names alone do not fix transition semantics or future mapping under the authoritative causal rules.
- **TC001-A01-C074 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved: asymmetric repricing implies bid-side versus ask-side asymmetry and the bid/ask-change charter items imply per-side attribution, but the producer's direction convention and its price-mechanics-versus-liquidity split are declared nowhere.
- **TC001-A01-C075 · normalization_baseline_semantics — UNRESOLVED.** Normalization/baseline semantics are unresolved; the experiment contract requires an explicit normalization_basis per experiment and rule 10 requires preserving raw value, normalized value, and basis, neither satisfiable from available authority.
- **TC001-A01-C076 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: acceleration plausibly requires at least two consecutive price deltas and repricing requires the prior quote, but required depth and retention are undeclared.
- **TC001-A01-C077 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved: no payload field names are declared anywhere on authorized surfaces, and no field may be treated as future-bearing from its name alone under the authoritative causal rules.
- **TC001-A01-C078 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure is inaccessible under raw-lake denial. Non-nullable bid/ask columns are a declared physical basis for quote-change computation - capability, not producer fact - and any use of spread-state outputs for spread impulses is undeclared; absence of derives_from must not be read as independence (rule 11).
- **TC001-A01-C079 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: behavior at zero spread or crossed quotes, representation of insufficient history, and the meaning of a null payload value are undeclared.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

### 5.5 quote_pressure — payload_quote_pressure (claims TC001-A01-C080 – TC001-A01-C095)

**Output surface (AUTHORITATIVE, TC001-A01-C080).** The declared physical output surface of the quote_pressure family is the single nullable LargeUtf8 column payload_quote_pressure embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- No trade price, trade volume, or aggressor-side column is declared on the tick source; bid and ask are the only row-local price fields declared. Row-local trade-print attribution is therefore not supported by declared fields.

Dimensions:

- **TC001-A01-C080 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the quote_pressure family is the single nullable LargeUtf8 column payload_quote_pressure embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_pressure, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C081 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names buy/sell pressure, pressure variants, normalization/range, persistence, reset, and availability semantics, but the designated authority source is absent and the registry contains zero entries, so no purpose or registry role is declared for this family.
- **TC001-A01-C082 · record_variants — UNRESOLVED.** Record variants within payload_quote_pressure are unresolved: charter-named buy/sell pressure variants are unresolved as producer facts; the declared carrier is one nullable text value per tick row.
- **TC001-A01-C083 · event_vs_persistent_state — UNRESOLVED.** Whether quote_pressure carries discrete events or persistent state (for example a standing pressure level), or both, is unresolved; no producer authority exists to classify the family.
- **TC001-A01-C084 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records, and lawful detector-state use of the row-physical fields is itself unresolved.
- **TC001-A01-C085 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: whether records carry their own occurrence time inside the payload text or inherit the row's event_ts_ns is undeclared; under the causal-clock rule occurrence and origin timestamps remain provenance either way.
- **TC001-A01-C086 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved: availability_semantics is a required registry field with no entry for this family; no availability timestamp is declared for payload records, and received_ts_ns is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C087 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: the charter explicitly lists reset, and whether pressure resets on a schedule, on an event, or on state replacement is declared nowhere; lifecycle_vocabulary is a required registry field with no entry for this family.
- **TC001-A01-C088 · duration_persistence — UNRESOLVED.** Persistence is unresolved: whether pressure persists, decays, or is recomputed per row is undeclared.
- **TC001-A01-C089 · transitions — UNRESOLVED.** Transition semantics are unresolved: no transition vocabulary is declared for this family, and from/to-style names alone do not fix transition semantics or future mapping under the authoritative causal rules.
- **TC001-A01-C090 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved: the buy/sell distinction has no declared sign or attribution rule. The declared tick schema contains no trade price, trade volume, or aggressor-side column, so row-local declared fields provide no direct trade-print basis; the producer's attribution basis (bid/ask-derived, state-carried, or external) is undeclared.
- **TC001-A01-C091 · normalization_baseline_semantics — UNRESOLVED.** Normalization/range semantics are unresolved: the charter item 'normalization/range' suggests a bounded measure, but no range, bound, or basis is declared; the experiment contract requires an explicit normalization_basis per experiment and rule 10 requires preserving raw value, normalized value, and basis.
- **TC001-A01-C092 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: any accumulation or decay construction plausibly requires trailing state, but required depth and retention are undeclared.
- **TC001-A01-C093 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved: no payload field names are declared anywhere on authorized surfaces, and no field may be treated as future-bearing from its name alone under the authoritative causal rules.
- **TC001-A01-C094 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure is inaccessible under raw-lake denial; the attribution basis noted under direction semantics is itself an unresolved dependency on undeclared inputs. Absence of derives_from must not be read as independence (rule 11).
- **TC001-A01-C095 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: pressure behavior when inputs are absent or reset, and the meaning of a null payload value, are undeclared; nullability of the column is authoritative, its meaning is not.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

### 5.6 spread_state — payload_spread_state (claims TC001-A01-C096 – TC001-A01-C111)

**Output surface (AUTHORITATIVE, TC001-A01-C096).** The declared physical output surface of the spread_state family is the single nullable LargeUtf8 column payload_spread_state embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial.

Physical capability notes (authoritative schema facts; capability is not producer fact):

- bid and ask are declared non-nullable Float64 columns on every tick row, so an ask-minus-bid spread is computable from declared fields; the producer's actual spread definition is undeclared.

Dimensions:

- **TC001-A01-C096 · physical_output_surface — AUTHORITATIVE.** The declared physical output surface of the spread_state family is the single nullable LargeUtf8 column payload_spread_state embedded on every row of the XAUUSD Tick source (six parts, schema verified on all six, 68,445,659 declared rows total). source_inventory declares no other column or table for this family; dataset-level surfaces, if any, are unresolvable under raw-lake denial. Counterevidence on file: authority.dataset_manifest_v1.
  - Matrix material: safe_fields = SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_spread_state, event_ts_ns, received_ts_ns, source_sequence, bid, ask (recorded in this claim's notes).
- **TC001-A01-C097 · producer_purpose_and_scientific_role — UNRESOLVED.** Producer purpose and supported scientific role are unresolved. The charter names compression, widening, state transition, from/to, spread ratio, and median baseline semantics, but the designated authority source is absent and the registry contains zero entries, so no purpose or registry role is declared for this family.
- **TC001-A01-C098 · record_variants — UNRESOLVED.** Record variants within payload_spread_state are unresolved: charter-named candidate records (compression, widening, state transition) are unresolved as producer facts; the declared carrier is one nullable text value per tick row.
- **TC001-A01-C099 · event_vs_persistent_state — UNRESOLVED.** The charter explicitly poses event versus persistent state for spread_state, and the column name itself suggests state, but a column name is not producer authority; whether records are transition events, a per-row state declaration, or both is unresolved. Counterevidence on file: repository.surfaces.
- **TC001-A01-C100 · record_identity — UNRESOLVED.** Record identity is unresolved: no key, identifier, or idempotency rule is declared for payload records, and lawful detector-state use of the row-physical fields is itself unresolved.
- **TC001-A01-C101 · occurrence_timestamp — UNRESOLVED.** Occurrence-timestamp semantics are unresolved: whether records carry their own occurrence time inside the payload text or inherit the row's event_ts_ns is undeclared; under the causal-clock rule occurrence and origin timestamps remain provenance either way.
- **TC001-A01-C102 · availability_known_timestamp — UNRESOLVED.** Availability/known-timestamp semantics are unresolved: availability_semantics is a required registry field with no entry for this family; no availability timestamp is declared for payload records, and received_ts_ns is classified PHYSICALLY_PRESENT_ON_TICK_ROW only.
- **TC001-A01-C103 · reset_replacement_semantics — UNRESOLVED.** Reset/replacement semantics are unresolved: whether a new state record replaces the prior state, appends to it, or requires an explicit transition record is undeclared; lifecycle_vocabulary is a required registry field with no entry for this family.
- **TC001-A01-C104 · duration_persistence — UNRESOLVED.** Duration/persistence are unresolved: how long a compressed or widened state persists and the cadence at which state is re-declared on rows are undeclared.
- **TC001-A01-C105 · transitions — UNRESOLVED.** Transition semantics are unresolved: the charter items 'state transition' and 'from/to' have no declared vocabulary, and from/to-style names alone do not fix transition semantics or future mapping under the authoritative causal rules.
- **TC001-A01-C106 · direction_semantics — UNRESOLVED.** Direction semantics are unresolved: compression and widening are opposing directions of spread change, but the declared sign convention (which direction is positive) and the change measure are declared nowhere.
- **TC001-A01-C107 · normalization_baseline_semantics — UNRESOLVED.** Normalization/baseline semantics are unresolved: the charter items 'spread ratio' and 'median baseline' are undeclared - the spread basis (absolute ask-minus-bid versus a relative form), the median window, warm-up, and the ratio construction are declared nowhere. The experiment contract requires an explicit normalization_basis per experiment, and rule 10 requires preserving raw value, normalized value, and basis.
- **TC001-A01-C108 · history_required_fields — UNRESOLVED.** History-required fields are unresolved: a median spread baseline plausibly requires trailing spread history, but required depth and retention are undeclared.
- **TC001-A01-C109 · future_field_mapping — UNRESOLVED.** Future-field mapping is unresolved: no payload field names are declared anywhere on authorized surfaces, and no field may be treated as future-bearing from its name alone under the authoritative causal rules.
- **TC001-A01-C110 · dependencies — UNRESOLVED.** Dependencies are unresolved by two closed routes: the registry (which must declare derives_from per entry) is empty, and run-level dependency closure is inaccessible under raw-lake denial. Non-nullable bid/ask columns make a spread computation physically possible on every row - capability, not producer fact - and the actual spread definition plus any relation to quote_dynamics spread impulses are undeclared; absence of derives_from must not be read as independence (rule 11).
- **TC001-A01-C111 · unavailable_unresolved_semantics — UNRESOLVED.** Unavailable/unresolved semantics are unresolved: behavior at zero spread or crossed quotes and the meaning of a null payload value are undeclared; nullability of the column is authoritative, its meaning is not.

Unresolved summary: 15 of 16 dimensions unresolved (dimensions 2–16); only the physical output surface is authoritative. Unresolved questions for this family are recorded as structured entries in the JSON artifact.

## 6. Output matrix — material represented in the flattened claims

The universal shape has no separate matrix key, so the former output matrix is preserved as follows: every material cell is represented by a claim in the flattened `claims` array (mapping below), and the matrix appears here only as a non-authoritative presentation of those claims. All semantic cells are the exact token UNRESOLVED for every family — that is the finding, not a shortcut.

### 6.1 feed_health

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C018 |
| role | UNRESOLVED | TC001-A01-C017 |
| event_or_state | UNRESOLVED | TC001-A01-C019 |
| availability_rule | UNRESOLVED | TC001-A01-C022 |
| persistence_rule | UNRESOLVED | TC001-A01-C024 |
| direction_semantics | UNRESOLVED | TC001-A01-C026 |
| baseline_semantics | UNRESOLVED | TC001-A01-C027 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_feed_health, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C016 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C029 |
| dependencies | UNRESOLVED | TC001-A01-C030 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C017 – TC001-A01-C031 + unresolved_questions |

### 6.2 micro_volatility

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C034 |
| role | UNRESOLVED | TC001-A01-C033 |
| event_or_state | UNRESOLVED | TC001-A01-C035 |
| availability_rule | UNRESOLVED | TC001-A01-C038 |
| persistence_rule | UNRESOLVED | TC001-A01-C040 |
| direction_semantics | UNRESOLVED | TC001-A01-C042 |
| baseline_semantics | UNRESOLVED | TC001-A01-C043 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_micro_volatility, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C032 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C045 |
| dependencies | UNRESOLVED | TC001-A01-C046 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C033 – TC001-A01-C047 + unresolved_questions |

### 6.3 quote_arrival

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C050 |
| role | UNRESOLVED | TC001-A01-C049 |
| event_or_state | UNRESOLVED | TC001-A01-C051 |
| availability_rule | UNRESOLVED | TC001-A01-C054 |
| persistence_rule | UNRESOLVED | TC001-A01-C056 |
| direction_semantics | UNRESOLVED | TC001-A01-C058 |
| baseline_semantics | UNRESOLVED | TC001-A01-C059 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_arrival, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C048 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C061 |
| dependencies | UNRESOLVED | TC001-A01-C062 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C049 – TC001-A01-C063 + unresolved_questions |

### 6.4 quote_dynamics

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C066 |
| role | UNRESOLVED | TC001-A01-C065 |
| event_or_state | UNRESOLVED | TC001-A01-C067 |
| availability_rule | UNRESOLVED | TC001-A01-C070 |
| persistence_rule | UNRESOLVED | TC001-A01-C072 |
| direction_semantics | UNRESOLVED | TC001-A01-C074 |
| baseline_semantics | UNRESOLVED | TC001-A01-C075 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_dynamics, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C064 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C077 |
| dependencies | UNRESOLVED | TC001-A01-C078 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C065 – TC001-A01-C079 + unresolved_questions |

### 6.5 quote_pressure

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C082 |
| role | UNRESOLVED | TC001-A01-C081 |
| event_or_state | UNRESOLVED | TC001-A01-C083 |
| availability_rule | UNRESOLVED | TC001-A01-C086 |
| persistence_rule | UNRESOLVED | TC001-A01-C088 |
| direction_semantics | UNRESOLVED | TC001-A01-C090 |
| baseline_semantics | UNRESOLVED | TC001-A01-C091 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_quote_pressure, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C080 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C093 |
| dependencies | UNRESOLVED | TC001-A01-C094 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C081 – TC001-A01-C095 + unresolved_questions |

### 6.6 spread_state

| Matrix field | Value | Represented by |
|---|---|---|
| record_variant | UNRESOLVED | TC001-A01-C098 |
| role | UNRESOLVED | TC001-A01-C097 |
| event_or_state | UNRESOLVED | TC001-A01-C099 |
| availability_rule | UNRESOLVED | TC001-A01-C102 |
| persistence_rule | UNRESOLVED | TC001-A01-C104 |
| direction_semantics | UNRESOLVED | TC001-A01-C106 |
| baseline_semantics | UNRESOLVED | TC001-A01-C107 |
| safe_fields | SAFE_AT_DETECTOR_STATE: none · PHYSICALLY_PRESENT_ON_TICK_ROW: payload_spread_state, event_ts_ns, received_ts_ns, source_sequence, bid, ask | TC001-A01-C096 (notes) + TC001-A01-C005 |
| future_fields | UNRESOLVED | TC001-A01-C109 |
| dependencies | UNRESOLVED | TC001-A01-C110 |
| unresolved | 15 dimensions (2–16) | TC001-A01-C097 – TC001-A01-C111 + unresolved_questions |

## 7. Unresolved questions

90 structured unresolved questions are recorded in the JSON artifact (`unresolved_questions`, TC001-A01-UQ001 – TC001-A01-UQ090), one per unresolved dimension per family, each linked to its claim. Distribution: feed_health 15, micro_volatility 15, quote_arrival 15, quote_dynamics 15, quote_pressure 15, spread_state 15.

## 8. Constraints honored

- No field is classified SAFE_AT_DETECTOR_STATE. event_ts_ns, received_ts_ns, source_sequence, bid, and ask are classified PHYSICALLY_PRESENT_ON_TICK_ROW only (TC001-A01-C005); lawful detector-state use remains unresolved for every family.
- No name-based future inference. The names from, to, transition, completed, peak, fill, end, and duration were not treated as future-bearing; in fact no payload field names are declared on any authorized surface at all, so the guard is moot on this distribution, and the general causal rule (TC001-A01-C006) governs any future resolution.
- derives_from absence is not independence. The registry is empty and run-level dependency closure is under raw-lake denial (TC001-A01-C012, TC001-A01-C014); rule 11 constrains all confluence claims.
- The registry contract's required-field list is used only to state precisely what authority is missing; it is not treated as content authority for any family.
- payload_drift_burst is out of scope; no drift interaction study was performed (TC001-A01-C011).
- No identity terms and no identity inference appear in either artifact, per the research rules' closing constraint. All source paths in both artifacts are relative.

## 9. Conclusion

**TC-001 behavioral conditioning research is BLOCKED pending approved authority.** (Represented as claim TC001-A01-C112.)

Blocking reasons:

- The designated authority source xauusd.schemas does not exist as a producer record authority on any authorized surface; authority/ and contracts/ hold lake-storage and governance schemas only, and registry/ is empty.
- Every detector-meaning dimension (producer purpose and role, record variants, event versus state, identity, occurrence and availability timestamps, reset and replacement, duration, transitions, direction, normalization and baseline, history requirements, future-field mapping, dependencies, unavailable-value semantics) is UNRESOLVED for all six families; payload record internals are undeclared opaque text.
- The detector registry entry contract requires eleven authority fields per detector, and zero entries exist; the experiment contract requires anchor lifecycle and normalization-basis declarations that cannot be populated.
- The five physical tick columns are classified PHYSICALLY_PRESENT_ON_TICK_ROW only; lawful detector-state use is unresolved, so no detector-state-safe field exists for any family.
- derives_from declarations are unavailable and run-level dependency closure is under raw-lake denial, so neither dependency nor independence can be asserted for any family (rule 11).

Unblock conditions:

- Approved producer authority for the six families under the designated authority source id, declaring record schemas and variants, event/state classification, identity, occurrence and availability semantics, reset/replacement, duration and persistence, transitions, direction, normalization/baseline, history requirements, future-field mapping, dependencies, and unavailable-value semantics.
- Populated detector registry entries for the six families (including derives_from, roles, lifecycle_vocabulary, availability_semantics, native_scales, semantic_status).
- Explicit detector-state authorization for any physical-column use at detector state.
