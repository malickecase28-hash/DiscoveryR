# Scientific ontology

This document defines the vocabulary used by the R→S→P machine. Contracts are
small and composable; a record names its identities, scope, timing, and evidence
state instead of relying on prose in a worker prompt.

## Core objects

`Instrument` identifies a traded instrument. `InstrumentScope` is an immutable
dataset interval and source manifest. `ProducerAuthority` binds producer commit,
payload and source schemas, detector/state versions, parameters, availability
rules, and required native scales. `AuthorityCompatibility` is either
`AUTHORITY_COMPATIBLE` or `AUTHORITY_DELTA_REQUIRED` with precise changed
surfaces.

`Detector` is an observed or derived information producer. A detector may have
multiple roles: `LIFECYCLE_OBJECT`, `STRUCTURAL_OBJECT`, `EVENT`, `STATE_REGIME`,
`DIRECTIONAL_CONTEXT`, `QUALITY_INSTRUMENTATION`, `NORMALIZATION_MEASURE`,
`TEMPORAL_CONTEXT`, `DERIVED_OBJECT`, and `COMPOSITE_CONTEXT`. `DetectorVersion`
and `DetectorLineage` make derivation and parameter identity explicit.

`AnchorDefinition` describes the information object around which a research
question is formed. `AnchorProgram` is a declared program for that anchor; an
anchor instance is not itself an experiment. `NativeScale` remains attached to
every observation. `OccurrenceRule` records when an event happened;
`AvailabilityRule` records when it became lawful input. Eligibility uses one
causal information clock: `availability_time <= research_time`.

`ExposureClass` describes development, detector-confirmation, strategy-holdout,
or portfolio-holdout exposure. It is separate from evidence maturity.
`ContextPermission` declares which context may be read. `NormalizationBasis`
identifies raw value, transform, basis, and timing. `OutcomeDefinition` is the
frozen measurement definition.

## Questions and evidence

`Question` is generated from a frozen phenotype, detector ontology, lineage,
permissions, and a compatible relationship operator. Operators are temporal,
directional, spatial, nesting, state transition, lifecycle, regime, lead/lag,
context, interaction, and incremental-information. Generation is bounded and
semantic; it does not enumerate meaningless Cartesian pairs or consume prior
outcome winners.

`Hypothesis`, `ExperimentContract`, `ControlDesign`, and `NullDesign` describe a
test before execution. `Finding` and `KnowledgeRecord` retain `KNOWN`, `NULL`,
`FAILED`, `ABSTAINED`, `INCONCLUSIVE`, and `CONTRADICTORY` states, including
metrics, uncertainty, failures, and partial evidence. A method challenge is
separate evidence: `SURVIVED_CHALLENGE` never means `CONFIRMED`.

## Strategy and portfolio

Program S consumes confirmed behavioral findings and declares a
`StrategyHypothesis`, entry/exit/sizing/management `DecisionRule`s,
`ExecutionAssumption`, `CostModel`, and `RiskRule`. `StrategyValidation` records
walk-forward partitions, coverage, abstentions, costs, risks, and evidence.
Program P consumes confirmed strategies as `PortfolioComponent`s and declares
`PortfolioConstraint`s for overlap, allocation, drawdown, capacity, turnover,
liquidity, concentration, diversification, and stress.

`ConfirmationContract` freezes claim, population, anchor, context, outcome,
metric, code identity, controls, null, and multiplicity family before an
externally authorized confirmation process can read its held-out scope.
