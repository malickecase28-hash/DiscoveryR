# R→S→P System Charter

This charter defines deterministic research infrastructure for three programs. R is Market Research, S is Strategy / Alpha Research, and P is Portfolio Research. Promotion is an evidence-lifecycle operation and is never Program P.

## Ownership and boundaries

| Surface | Owns | Must not own |
| --- | --- | --- |
| `research-contracts` | Composable semantic contracts, v1 compatibility, authority/submission metadata, exposure and provenance contracts, schemas | Parquet I/O, pure statistical algorithms, broker behavior |
| `research-tape` | Physical causal data, Parquet projection, as-of reads, raw and basis timing, availability sidecar, development-view physical materialization | Semantic lifecycle, sealed knowledge/workflow blinding, normalized measurement contracts |
| `research-engine` | Arrow-free deterministic R/S/P execution, role dispatch, bounded statistics/nulls, challenges, strategy/portfolio methods, reports, thin CLI adapter | Canonical knowledge storage, self-confirmation, live orders/fills |
| `agent_harness` and submission tooling | Scope isolation, workflow blinding, sealed publication and custody boundary | Scientific findings or detector semantics |

Existing v1 `ExperimentContract`, `RunManifest`, `Finding`, `KnowledgeRecord`, `ExposureClass`, registry, causal cursors, projected reads, view builder, sidecar, submission sealing, and workflow isolation remain readable. Additive schemas and re-exports preserve v1 APIs; the current baseline is not described as fully normalized until implemented.

## Semantic layer

The semantic layer uses small composable contracts grouped into 4–6 modules rather than a giant schema. It covers: `Instrument`, `InstrumentScope`, `ProducerAuthority` (compatibility, version, parameter IDs), `AvailabilityRule`, `OccurrenceRule`, `LifecycleVocabulary` and `LifecycleTransition`, `ObjectIdentity` and `DetectorLineage`, `AnchorDefinition` and `AnchorProgram`, `ExposureClass`, `ContextPermission`, `NormalizationBasis`, `OutcomeDefinition`, `Question`, `Hypothesis`, existing `ExperimentContract`/`RunManifest`/`Finding`/`KnowledgeRecord`, `ControlDesign`, `NullDesign`, `MethodChallenge`, `ConfirmationContract`, `StrategyHypothesis`, `DecisionRule`, `ExecutionAssumption`, `CostModel`, `RiskRule`, `StrategyValidation`, `PortfolioComponent`, `PortfolioConstraint`, and `ReproducibilityIdentity`.

The ten detector roles are exactly: `LIFECYCLE_OBJECT`, `STRUCTURAL_OBJECT`, `EVENT`, `STATE_REGIME`, `DIRECTIONAL_CONTEXT`, `QUALITY_INSTRUMENTATION`, `NORMALIZATION_MEASURE`, `TEMPORAL_CONTEXT`, `DERIVED_OBJECT`, and `COMPOSITE_CONTEXT`. The eleven relationship operator categories are `temporal`, `directional`, `spatial`, `nesting`, `state_transition`, `lifecycle`, `regime`, `lead_lag`, `context`, `interaction`, and `incremental_information`. A detector may carry multiple roles. The atlas dispatches compatible templates and does not brute-force the Cartesian product.

## Authority and causality

Every input and output records immutable source, binary, configuration, seed, and output identities; absolute paths are excluded from identity. Raw and basis timestamps remain distinct, and context is usable only when its declared availability satisfies the decision anchor. Invalid causal inputs fail closed. Contradictory, null, inconclusive, missing, and partial research findings remain first-class evidence with their metrics and uncertainty.

Workers remain denied raw/private lake, peer, holdout, and future context surfaces. Nested and future exposure is recorded in a ledger. Confirmation is a separate contract with an injectable external custodian policy and authentication boundary; a worker cannot self-unlock it, a shared boolean or secret is not a security boundary, and the real CLI remains locked until an approved external process.

## Program R — Market Research

R supports atlas dispatch, anchor programs, native phenotypes, question generation, same-domain and cross-domain discovery, candidate formation, method challenge, detector confirmation, characterization, instrument reports, and knowledge queries. Role-specific templates cover lifecycle formation/persistence/transition/termination/censoring/prospective behavior; state occupancy/transition/duration/persistence; event incidence/clustering/magnitude/recurrence/prospective consequence; quality failure/incidence/persistence/contamination; and normalization/context distribution/stability/regime interpretation/lawful causal usage. Questions are generated from frozen phenotype, ontology, lineage, semantics, permissions, and operators without prior outcome winners.

R statistics are reusable and bounded: streaming moments; quantiles with declared exact or approximate limits; effect sizes; exact-strata matched controls; block-aware bootstrap/permutation; circular/time-shift nulls; seeded Monte Carlo; full-family accounting with BH/BY FDR and max-statistic control; support and cluster concentration; and survival/hazard where applicable. Synthetic tests use small analytic or exhaustive oracles. No scientific thresholds, returns, outcomes, or real scanners are introduced here.

## Program S — Strategy / Alpha Research

S represents a `StrategyHypothesis`, decision entry/exit/sizing/management rules, execution assumptions, cost model, risk rules, and walk-forward validation. Strategy confirmation is a separate operation with explicit input confirmation and method challenge. Promotion may consume S evidence as a lifecycle operation, but it does not become Program P or grant live authority.

## Program P — Portfolio Research

P represents portfolio components and constraints: correlation and conditional correlation, overlap, capital allocation, drawdown, regime diversification, capacity, turnover, liquidity, concentration, risk budgets, optimization, stress, and portfolio confirmation. P consumes confirmed or explicitly unconfirmed strategy evidence according to `ContextPermission`; it reports unknowns and abstentions without manufacturing performance.

## Acceptance and deferrals

Acceptance requires a fresh-process synthetic two-instrument R→S→P path, causal timing, v1 compatibility, deterministic path-independent identities, explicit partial/null/contradictory evidence, future/nested holdout rejection, and locked confirmation. No real study, alpha claim, holdout exposure, AP change, OS sandbox rebuild, live broker/fills, distributed execution, graph database, or 21 adapter set is included. Add those only when an approved producer semantic need and acceptance test exist.
