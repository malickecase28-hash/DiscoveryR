# R→S→P System Implementation Plan

> Status at this revision: semantic contracts and synthetic R/S/P plumbing are
> implemented; strict external confirmation, challenge coverage, materializer
> recovery, and fresh-process pipeline coverage remain partial or not run.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic infrastructure for R Market Research, S Strategy / Alpha Research, and P Portfolio Research while preserving v1 contracts, causal tape behavior, workflow blinding, and locked confirmation.

**Architecture:** Keep `research-contracts` as the semantic, authority, submission, and provenance boundary; keep `research-tape` as the physical causal Parquet boundary. Add one Arrow-free `research-engine` crate for reusable R statistics, S strategy validation, P portfolio methods, challenge, reproducibility, and a thin CLI wrapper around the existing materializer.

**Tech Stack:** Rust workspace, existing `serde`, `serde_json`, `sha2`, `NativeScale`, and current tape dependencies. Pure statistics do not depend on Arrow; do not add a dependency unless an existing one cannot express a tested operation.

**Spec:** `docs/SYSTEM_CHARTER.md`

## Global Constraints

- Preserve v1 `ExperimentContract`, `RunManifest`, `Finding`, `KnowledgeRecord`, `ExposureClass`, registry, and generated schemas; additions are composable and additive.
- Group new typed concepts into 4–6 focused contract modules; do not jam them into the existing `lib.rs` or one giant schema.
- Support 15+ instruments, 23 role-heterogeneous detectors, ten exact semantic roles, and eleven exact relationship operator categories without inventing unneeded adapters.
- Preserve distinct raw and basis timestamps and consume declared availability; reject invalid causal inputs while retaining contradictory, null, inconclusive, partial, and unknown findings. Do not claim full measurement normalization before it is implemented.
- Enforce declared access policies for raw/private lake, peer, holdout, and future context; the Windows folder workflow does not itself make host files inaccessible. Confirmation requires an externally authorized custodian policy and authentication boundary.
- Reproducibility identity includes immutable source/binary/config/seed/output identities and excludes absolute paths.
- Do not add scientific thresholds, returns, outcomes, real studies, scanners, AP changes, or scientific artifacts.

## Existing capability and gap map

| Capability | Existing owner/evidence | Planned gap |
| --- | --- | --- |
| Typed v1 experiment/run/evidence/exposure and registry | `crates/research-contracts/src/lib.rs`, `contracts/`, `registry/` | Add composable semantics and compatibility/version/parameter IDs |
| Physical causal data, raw+basis timing, projected Parquet, NativeScale | `crates/research-tape/src/lib.rs`, `development_view.rs` | Stable engine input adapter; reuse shared `NativeScale` via contracts/re-export |
| Availability sidecar | `crates/research-tape/src/fvg_availability.rs` | Consume declared availability; never infer from occurrence |
| Submission sealing and workflow blinding | `research-contracts/src/submission.rs`, `agent_harness/` | Typed exposure ledger, knowledge publication/query boundaries, custodian contract |
| Role-dispatched atlas/questions | None | R ontology, exact role/operator compatibility, bounded question generation |
| Reusable statistics/nulls | None | R statistics listed in Task 2 with declared exact/approximation ceilings |
| Strategy and portfolio methods | None | S decision/execution/cost/risk/walk-forward and P allocation/constraints/stress |
| Challenge, confirmation, provenance | Partial v1 authority/sealing | Separate challenge, external custody, and path-independent identity |

Boundary audit targets: development-view expected instrument/scope/source binding and root/ancestor reparse rejection; launcher current-directory correctness; interruption-safe immutable knowledge publication; declared-root output containment and expected identities; targeted report-gate source junction escape where its existing contract applies. The Windows folder workflow remains the accepted boundary; no OS sandbox rebuild.

## Decision ledger

| Decision | Ruling |
| --- | --- |
| Program meanings | R = Market Research; S = Strategy / Alpha Research; P = Portfolio Research; promotion remains an evidence-lifecycle operation |
| Input authority | S and P require confirmed strategy/behavioral inputs; synthetic confirmations are test-only labels and never real authority |
| Isolation claim | `agent_harness` records and checks access policy; the Windows folder workflow is not an OS security boundary and does not make host files inaccessible |
| Timing claim | Tape preserves physical timing fields and causal availability; complete normalization is a future contract/engine concern, not a baseline claim |
| Evidence handling | Invalid causal inputs fail closed; R preserves partial, contradictory, null, inconclusive, and unknown findings |
| Architecture | Retain `research-contracts` and `research-tape`; add one Arrow-free engine; reuse `NativeScale`; avoid detector adapters without producer need |

## Staged tasks

### Task 1: Semantic contracts and compatibility

**Files:**
- Create: 4–6 focused modules under `crates/research-contracts/src/` for identity/provenance, instrument/producer, timing/lifecycle, research evidence, and strategy/portfolio contracts.
- Modify: `crates/research-contracts/src/lib.rs` only for module exports and v1-compatible re-exports.
- Modify: `crates/research-contracts/src/submission.rs` for typed submission/exposure metadata and interruption-safe publication.
- Modify: `contracts/` via the existing schema generator; preserve all v1 schemas.
- Test: `crates/research-contracts/tests/contracts.rs`, `crates/research-contracts/tests/submission_sealing.rs`.

**Interfaces:**
- Add `Instrument`, `InstrumentScope`, `ProducerAuthority`, `AuthorityCompatibility`, `DetectorRole`, `DetectorVersion`, `AvailabilityRule`, `OccurrenceRule`, `LifecycleVocabulary`, `LifecycleTransition`, `ObjectIdentity`, `DetectorLineage`, `AnchorDefinition`, `AnchorProgram`, `ContextPermission`, `NormalizationBasis`, `OutcomeDefinition`, `Question`, `Hypothesis`, `ControlDesign`, `NullDesign`, `MethodChallenge`, `ConfirmationContract`, `StrategyHypothesis`, `DecisionRule`, `ExecutionAssumption`, `CostModel`, `RiskRule`, `StrategyValidation`, `PortfolioComponent`, `PortfolioConstraint`, and `ReproducibilityIdentity` as small serde contracts.
- Reuse existing `ExposureClass`, `ExperimentContract`, `RunManifest`, `Finding`, and `KnowledgeRecord`; expose `validate_compatibility(...)` and `record_exposure(...)` with explicit error types.

- [x] Add failing tests for v1 round trips, all ten roles, all eleven operators, illegal authority/operator combinations, nested/future exposure, path-independent identity, and partial/contradictory/null evidence retention.
- [x] Implement minimal modules and additive schemas; use `NativeScale` as the single shared scale type.
- [ ] Add expected view identity/scope/source and reparse checks, then run targeted tests and schema verification. (partial)

### Task 2: Program R engine and statistics

**Files:**
- Create: `crates/research-engine/Cargo.toml`, `src/lib.rs`, `src/role.rs`, `src/questions.rs`, `src/stats.rs`, `src/provenance.rs`.
- Modify: workspace `Cargo.toml`.
- Test: `crates/research-engine/tests/research.rs`.

**Interfaces:**
- Consume validated contract inputs and causal records through a tape adapter; pure stats depend on contracts only, never Arrow or tape internals.
- Produce `RoleAtlas`, `Question`, `ResearchPlan`, and `ResearchReport` with `Known`, `Abstained`, `Failed`, `Null`, `Inconclusive`, and `Contradictory` evidence states.
- Expose bounded operations for streaming moments; quantiles with declared exact/approximate limits; effect size; exact-strata matched controls; block-aware bootstrap/permutation; circular/time-shift nulls; seeded Monte Carlo; full-family accounting; BH/BY FDR; max-statistic control; support/cluster concentration; and survival/hazard.

- [ ] Test each implemented operation against a small analytic or exhaustive oracle, including seed repeatability, declared approximation bounds, and explicit abstention; do not add an estimator without its oracle.
- [ ] Test two synthetic instruments, multiple resolutions, positive/negative signs, role-compatible operators, and non-Cartesian question generation.
- [x] Implement R atlas, question generation, challenge/reproducibility identity, and immutable knowledge queries by reusing existing records. (synthetic; report stages remain bounded)

### Task 3: Program S strategy / alpha research

**Files:**
- Create: `crates/research-engine/src/strategy.rs`, `src/simulation.rs`, `src/walk_forward.rs`, `src/confirmation.rs`.
- Test: `crates/research-engine/tests/strategy.rs`.

**Interfaces:**
- Consume R evidence plus explicit `StrategyHypothesis`, entry/exit/sizing/management `DecisionRule`, `ExecutionAssumption`, `CostModel`, `RiskRule`, and a confirmed input manifest. Synthetic confirmation is test-only.
- Produce `StrategyValidation` with causal partitions, coverage, abstentions, costs, risks, and performance fields; promotion is a lifecycle operation over this evidence, never Program P.
- `confirm_strategy(request, external_custodian) -> Result<StrategyConfirmation, ConfirmationError>` requires injectable external policy/authentication; a worker bool, shared secret, or self-issued signature is invalid.

- [x] Test causal walk-forward boundaries, explicit cost/risk parameters, missing or future inputs, deterministic replay, and worker self-confirmation rejection.
- [x] Implement strategy simulation and method challenge without live orders, live fills, or hardcoded scientific outcomes. (external strategy confirmation remains not run)

### Task 4: Program P portfolio research and boundary-safe CLI

**Files:**
- Create: `crates/research-engine/src/portfolio.rs`, `src/bin/research_engine.rs`.
- Modify: `crates/research-tape/src/bin/materialize_development_view.rs` only for a stable library call; `agent_harness/launch/isolated-run.ps1` only for current-directory correctness.
- Test: `crates/research-engine/tests/portfolio.rs`, `tests/end_to_end.rs`; targeted existing harness tests.

**Interfaces:**
- Consume `PortfolioComponent`, `PortfolioConstraint`, confirmed strategy evidence, and declared `ContextPermission`. Synthetic confirmation is test-only.
- Produce reports for correlation/conditional correlation, overlap, capital allocation, drawdown, regime diversification, capacity, turnover, liquidity, concentration, risk budgets, optimization, stress, and portfolio confirmation.
- CLI is a thin adapter over the existing scope materializer and emits machine-readable reports, provenance, exposure ledger, and locked confirmation status under declared output roots.

- [ ] Test synthetic two-instrument R→S→P end-to-end with missing/future inputs, partial/contradictory evidence, deterministic path-independent identity, output containment, expected identity binding, and locked confirmation. (partial: stage artifacts and two fresh processes are not complete)
- [ ] Test portfolio methods against small analytic/exhaustive oracles; report unknowns and abstentions without fabricating performance.
- [ ] Add regressions for launcher CWD, root/ancestor reparse escapes, interruption-safe knowledge publication, and applicable report-gate junction escape.

### Task 5: Required documentation and final review

**Files:**
- Create/update exactly: `docs/SYSTEM_CHARTER.md`, `docs/SCIENTIFIC_ONTOLOGY.md`, `docs/PROGRAM_R.md`, `docs/PROGRAM_S.md`, `docs/PROGRAM_P.md`, `docs/REPRODUCIBILITY.md`, `docs/INSTRUMENT_ONBOARDING.md`, `docs/HOLDOUT_POLICY.md`, and `docs/AGENT_OPERATIONS.md`.
- Retain: `docs/RSP_IMPLEMENTATION_PLAN.md`.

- [ ] Document all interfaces, lifecycle evidence states, role/operator vocabulary, provenance, onboarding, holdout custody, and agent operations without scientific claims.
- [ ] Run targeted tests, `cargo fmt --all -- --check`, full workspace tests with corpus/lake variables unset, isolation validation, and fresh-process deterministic replay.
- [ ] Review changed surfaces for data leakage, invalid causal rejection versus finding preservation, and external confirmation custody; classify unavailable checks as `not_run`.
- [ ] Commit only after the independent review passes; leave live confirmation locked.

## Deliberate deferrals

No real studies, alpha claims, holdout exposure, AP changes, OS sandbox redesign, live broker/fill integration, distributed execution, graph database, or 21 detector adapters without a producer semantic need. Add any deferred surface only with an explicit contract and synthetic acceptance test.
