# AP-002 Blind Question Generator — QG V1 Report

- Program: AP-002 (FVG native strata), TrinityR research program
- Generator: QG V1 (blind), generated 2026-09-06T15:20:19Z
- AP-002 E1 freeze commit: `a7a9637410a74aba4e23827fc1b1e7dc6ec34a6f` (E1 FROZEN COMPLETE; E2/E3 not started; confirmation LOCKED)
- Producer commit (authority lineage): `211891e705d5e8ea132ef1391a0da4a9e5ff4376`; detector `trinity.analytics.bar.fvg.v4.2`
- Companion artifacts: `AP-002_BLIND_QG_UNIVERSE.json` (machine-readable), `AP-002_BLIND_QG_MANIFEST.json` (provenance/counts)

---

## 1. Blindness statement

This generator consumed **only** the five authorized inputs:

1. `knowledge/wave1_authority_closure/AP-002_AUTHORITY_V1.json`
2. `knowledge/wave1_authority_closure/XAUUSD_AUTHORITY_WAVE1_V2.json`
3. `knowledge/wave1_authority_closure/PRODUCER_LINKAGE_V2.json`
4. `registry/detectors_v1.json`
5. `contracts/detector_registry_entry_v1.schema.json`

It did **not** open anything under `docs/research/` (E1A/E1B results, reviews, challenges, reconciliations, synthesis), `.runs/`, or scanner source, and it consumed no E1 results, phenotype interpreter reviews, Method Challenge findings, candidate questions/winners, effect sizes, return signs, observed lifecycle frequencies, or observed outcomes of any kind. No file containing behavioral outcomes was encountered; `blindness_incidents` is empty in the manifest.

Every question below is two-sided or of a predeclared structural form. No effect direction is specified anywhere, because the generator is blind and none is known.

## 2. Inputs and what they license

- **AP-002 authority (FROZEN_NATIVE_STRATA).** FVG identity is `(timeframe, zone_id)` per native stratum; **cross-scale object identity is UNRESOLVED_AND_PROHIBITED**. Lifecycle semantics are authoritative: three-completed-bar formation with gap >= 0.2 formation-ATR and middle body >= 0.2 ATR; first bar = origin edge, middle = impulse bar, third completed bar = confirmation/detection event; `fvg_first_touch` on the first later bar whose **range** overlaps the zone; **fill** when a later bar's extreme reaches the far edge; persistence until far-edge invalidation; state serialized/restored in the detector envelope. Formation payload is emitted before touch/fill processing (**leakage boundary**: touch/fill cannot define formation-time cohorts). Lawful availability: `available_time` = `received_ts_ns` of the first source tick whose `event_time_ns` enters a later bucket (`source_sequence` tie-break); `bar_close_ts` is a temporal boundary only; **no synthetic tail bar** (structural right-censoring). Native strata: 15s, 30s, 1m, 5m, 15m, 1h, 4h.
- **XAUUSD authority Wave1 V2 (CLOSED/LOCKED).** Confirms the AP-002 gate, the availability correction (boundary-crossing tick receipt, not `bar_close_ts`), TC-001 scoped-E2 eligibility (not started), and BG-001 lineage restriction: dependency-edge independence is **not established**; "DO_NOT_CLAIM_INDEPENDENT_CONFLUENCE".
- **PRODUCER_LINKAGE_V2.** All bar/tick detectors constructed with producer defaults (`DetectorParams::new()`, 0 external parameters). Tick-surface outputs are available at `received_ts_ns` of the emitting tick (same-step emission; `event_time_is_availability = false`).
- **Registry (23 families).** 16 bar-domain families + 7 tick-domain families. Every entry has `semantic_status=UNRESOLVED`, empty `roles`, empty `derives_from`, availability semantics UNRESOLVED. Consequence: the generator knows which families exist, on which surfaces/scales — nothing more. Detector-specific semantic dependence questions are therefore **not licensable**; only family-level, form-declared readouts behind a context-resolution gate are.
- **Ontology (registry entry schema).** Supplies the role vocabulary (`directional_evidence`, `state_regime`, `structural_object`, `temporal_context`, `data_quality`, `lifecycle_state`, `normalization_reference`), semantic-status and native-scale-scope vocabulary used to structure the context classes and gates. No family currently declares a role, so no role-based filtering was asserted.

## 3. Exposure classification rules (declared ex ante)

- **E1 (CLOSED).** Native phenotype using only the object's own lifecycle, unconditional (marginal distributions, transition frequencies, direction symmetry, plain durations). Frozen E1 is complete; the generator proposes **zero E1 reopens** (Section 8).
- **E2_SAME_DOMAIN.** Conditioning information from the FVG domain itself:
  - (a) the zone's own lifecycle stage history;
  - (b) sub-rule **E2B**: the zone's own formation-time semantic attributes (magnitude, direction, geometry) as conditioners of later-stage outcomes — *conditional* rather than marginal structure;
  - (c) sub-rule **E2C**: same-detector population properties in the same stratum (other zones' states, causally available completed outcomes, spatial configuration, population composition rules including censoring rules).
- **E3_CROSS_DOMAIN.** Conditioning on other registered families' outputs or instrument-level context (calendar features of lawful availability timestamps; frozen availability-sidecar metadata), subject to the context-resolution gate.

Rules (b) and (c) are declared here explicitly because the E1/E2 boundary for "own-attribute conditional structure" and "population composition rules" had to be fixed blind; they are fixed consistently across all records.

## 4. The question universe (summary)

Full records with all required fields are in `AP-002_BLIND_QG_UNIVERSE.json`.

### 4.1 CORE_QUESTIONS (14 questions, 16 primary test cells)

| ID | Domain focus | Class | One-line statement |
|---|---|---|---|
| CORE-01 | magnitude, duration | E2 | Does formation magnitude (gap in ATR, impulse body) shift time-to-first-touch / time-to-fill survival per stratum? |
| CORE-02 | direction, temporal context, interaction | E3 | Does session block interact with direction for FVG touch/fill outcomes (direction x session)? |
| CORE-03 | lifecycle transition, stage-conditioned | E2 | Does a zone's stage history (untouched vs touched; stage-entry age) condition its next transition? |
| CORE-04 | structural nesting, spatial | E2 | Does same-stratum crowding at formation (overlap count, containment, nearest-edge distance) condition outcomes? |
| CORE-05 | same-domain path dependence | E2 | Does the rolling fill fraction of recently completed same-stratum/same-direction zones condition the next zone's outcomes? |
| CORE-06 | cross-domain (tick) | E3 | Do tick-surface microstructure families (7 declared candidates), read at formation availability time, condition FVG outcomes? |
| CORE-07 | cross-domain (bar structural) | E3 | Do bar-surface structural families (10 declared candidates) with presence/recency readouts condition FVG outcomes? |
| CORE-08 | lead/lag | E3 | Lead/lag cross-correlation between context-family event counts and FVG formation intensity per stratum (availability-time indexed). |
| CORE-09 | lead/lag, self-excitation | E2 | Do FVG formations cluster in time beyond a homogeneous-rate baseline (formation-to-formation hazard, lags 1..16)? |
| CORE-10 | anomalous lifecycle | E2 | Do anomalous completion modes (gap-through-fill candidate, instant fill, invalidation-without-touch) follow the same conditional structure as ordinary completions? |
| CORE-11 | incremental information | E3 | Do cross-domain readouts improve prospective prediction of touch/fill beyond the fixed same-domain baseline? |
| CORE-12 | scale relationships | E2 | Are the signs of the magnitude, stage-age, and crowding gradients consistent across the 7 strata (stratified agreement; 3 meta-tests)? |
| CORE-13 | censoring robustness | E2 | Are duration/transition conclusions robust to censored-aware vs complete-case treatment of structural end-of-stream censoring? |
| CORE-14 | availability sensitivity | E3 | Does the lawful availability lag (vs `bar_close_ts`) materially change causal eligibility sets and measured horizons? (Audit; frozen sidecar metadata only.) |

### 4.2 SYSTEMATIC_DISCOVERY_FAMILY (5 families, 581 cells)

| Family | Class | Dimensions (cardinalities) | Expansion | Cells |
|---|---|---|---|---|
| SF-1 STAGE_CONDITIONED_TRANSITIONS | E2 | anchor stage (2) x conditioner incl. 2 interaction pairs (7) x stratum (7) | 2x7x7 | 98 |
| SF-2 FORMATION_ATTRIBUTE_HORIZONS | E2 | attribute (3) x outcome-horizon (3) x direction (2) x stratum (7) | 3x3x2x7 | 126 |
| SF-3 CALENDAR_CONTEXT | E3 | calendar context (3) x direction (2) x outcome (3) x stratum (7) | 3x2x3x7 | 126 |
| SF-4 CROSS_DOMAIN_CONTEXT | E3 | context class (3) x readout (2) x window (2) x outcome (2) x stratum (7); per resolvable member family | 3x2x2x2x7 (upper bound; gate-mediated drops counted DROPPED_AT_GATE) | 168 |
| SF-5 SPATIAL_NESTING | E2 | nesting measure (3) x outcome (3) x stratum (7) | 3x3x7 | 63 |

Context-class membership (registry `domain` field only, all UNRESOLVED): `bar_structural` = swings, raw3, range, bos_choch, order_blocks, local_structure, structural_liquidity, dealing_range, micro_liquidity, micro_liquidity_context (10); `bar_state` = volume_by_time, volume_trend, auction_context, time_context, vwma_atr (5); `tick_microstructure` = drift_burst, spread_state, quote_dynamics, quote_arrival, micro_volatility, feed_health, quote_pressure (7).

### 4.3 OPTIONAL_LOW_PRIORITY (4 items, 161 cells)

- **OPT-01 CROSS_STRATUM_COACTIVATION (E3, 84 cells, GATED):** time-point spatial overlap of a coarser stratum's active zones with the anchor interval — never object identity. Quarantined behind an explicit clearance gate against the cross-scale identity restriction; 21 anchor/coarser-strata pairs x direction 2 x outcome 2.
- **OPT-02 NORMALIZATION_ROBUSTNESS (E2, 21):** instrument-intrinsic alternative magnitude normalizations vs the SF-2 gradient rank structure.
- **OPT-03 AVAILABILITY_LAG_ELIGIBILITY_SWEEP (E3, 28):** per-stratum sweep of CORE-14.
- **OPT-04 CENSORING_ESTIMATOR_SENSITIVITY_PER_STRATUM (E2, 28):** per-stratum sweep of CORE-13.

## 5. Search-space accounting (exact)

| Tier | Questions | Expanded test cells |
|---|---|---|
| CORE | 14 | 16 (CORE-12 = 3 gradient meta-tests) |
| SYSTEMATIC | 5 families | 581 (98 + 126 + 126 + 168 + 63) |
| OPTIONAL | 4 | 161 (84 + 21 + 28 + 28) |
| **Total** | **23** | **758** |

By exposure class (cells): **E1 = 0**, **E2 = 346** (10 core + 287 systematic + 49 optional), **E3 = 412** (6 core + 294 systematic + 112 optional). By exposure class (question-level, families counted as one record): **E2 = 13**, **E3 = 10**, **E1 reopens = 0**.

SF-4 (168) and OPT-01 (84) are declared **upper bounds**: SF-4 cells execute only for (family, readout) pairs passing the context-resolution gate (drops counted as DROPPED_AT_GATE, not failures); OPT-01 executes only if the research worker clears it against the cross-scale identity restriction. Every family carries an explicit expansion formula, so multiplicity is exactly computable without re-derivation.

Multiplicity plan: cores = Holm step-down within `MF_CORE_PRIMARY` (16 cells, two-sided); each SF family = BH-FDR (q = 0.05) within its own family; OPTIONAL tier = exploratory only, no confirmatory claims.

## 6. Null design rationale

Six null families are declared; naive independent row shuffles are prohibited wherever the conditioned series is autocorrelated.

- **NULL_WITHIN_DIRECTION_MATCHED / NULL_MATCHED_CONTROL** (matched-control within stratum x direction x session block, plus magnitude tercile where declared) for formation-time and stage-entry conditioning: preserves within-block temporal dependence and session structure while breaking the conditioner-outcome link.
- **NULL_DAY_SESSION_MATCHED** for calendar contexts: permutes session labels within UTC day (and direction), preserving day-level heterogeneity that a global shuffle would destroy.
- **NULL_CIRCULAR_SHIFT** (session-bounded blocks) for every population-level series — rolling histories (CORE-05), formation-count series (CORE-08/09), context readout series (CORE-06/07, SF-4): these series are diurnal and autocorrelated, so row shuffles are invalid.
- **NULL_WITHIN_REGIME_MATCHED** as secondary null for cross-domain conditioning.
- **NULL_SESSION_BLOCK_BOOTSTRAP** for methods-robustness agreement deltas only (CORE-13, OPT-02, OPT-04).
- CORE-14/OPT-03 are deterministic causal-validity audits with placebo calibration rather than behavioral nulls — declared as such.

Direction is never pooled away: stratified by default, pooled only as declared secondaries with direction as control.

## 7. Domain coverage

Produced questions cover all nineteen listed domains (mapping in the universe JSON): lifecycle transition; duration/persistence; stage-conditioned behavior; same-domain conditioning; cross-domain conditioning; lead/lag; direction; magnitude; regime dependence; temporal context; structural nesting; spatial relationships; scale relationships; incremental information; interaction effects; anomalous lifecycle cases; normalization robustness; censoring-sensitive; missingness/availability-sensitive.

Semantically invalid combinations **rejected** (full reasons in universe JSON `rejected_semantic_combinations`):

- **R1 — unconditional own-lifecycle descriptive cuts** (direction asymmetry of pure kinetics, marginal transition frequencies, plain duration distributions, event censuses): semantically valid but exposure-invalid — E1 is closed; admitted only in conditional E2 form.
- **R2 — cross-scale object identity conditioning** (multi-timeframe zone alignment/linkage): prohibited by the authority's `UNRESOLVED_AND_PROHIBITED` cross-scale identity rule.
- **R3 — external calendar/news regime conditioning:** no external data surface is authorized.
- **R4 — post-formation information as formation-time conditioner:** violates the authority leakage boundary.
- **R5 — detector-specific semantic co-occurrence claims:** registry roles/`derives_from` are empty and all semantics UNRESOLVED; only family-level gated readouts are licensable (BG-001 forbids independent-confluence claims).
- **R6 — reverse anchors** (FVG as context for other families' outcomes): outside AP-002 scope.
- **R7 — counterfactual parameter sweeps over `min_gap_atr`:** parameters are code-frozen; within-cohort magnitude conditioning (SF-2) is the lawful residual.

## 8. E1 reopen proposals: zero, with justification

The generator is blind to the frozen E1 contract's contents by design, so it cannot establish that any own-object semantic cut is genuinely omitted; asserting a reopen would require consuming forbidden E1 materials. Accordingly: (i) every own-lifecycle question generated here is admitted only in conditional (E2) form under declared sub-rules E2B/E2C; (ii) the two methods-robustness questions that touch E1-domain quantities (CORE-13 censoring treatment, and the population-composition rationale for O4) are classified E2 because censoring rules are population composition rules — they propose no new phenotype; (iii) the CORE-10 anomaly census aspect (do anomalous modes exist at all) is embedded in a two-sided conditional question rather than filed as an E1 descriptive reopen. If the research worker finds, at merge time, that frozen E1 already covers the conditional same-domain tier, the E2 tier can be re-scoped without touching E3.

## 9. Key dependency caveats carried on every applicable record

- **CAVEAT_REGISTRY_UNRESOLVED** — all 23 families UNRESOLVED; context readouts are form-declared and gate-mediated.
- **CAVEAT_BG001_CONFLUENCE** — no independent-confluence claims; cross-domain results are conditional-information findings only.
- **CAVEAT_CROSS_SCALE_IDENTITY** — no cross-timeframe object linkage anywhere; availability-time gating and stratified replication only.
- **CAVEAT_END_OF_STREAM** — structural right-censoring from the no-synthetic-tail-bar rule.
- **CAVEAT_STATE_RESTORE** — age/history computations exclude windows crossing detector state-envelope restore boundaries.
- **CAVEAT_LEAKAGE_BOUNDARY** — formation-time conditioning uses formation-available information only.

## 10. Handoff

This generator stops here. It has run nothing, committed nothing to git, and written exactly the three authorized artifacts. The post-generation merge (exposure-class reconciliation, gate clearance for SF-4/OPT-01, multiplicity finalization against the frozen E1 contract) is the research worker's task.
