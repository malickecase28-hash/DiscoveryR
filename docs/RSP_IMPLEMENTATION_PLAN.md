# R→S→P System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic, role-aware R→S→P infrastructure path while preserving existing v1 contracts, causal tape behavior, workflow blinding, and locked confirmation.

**Architecture:** Keep `research-contracts` as the semantic and authority boundary and `research-tape` as the causal/Parquet boundary. Add one `research-engine` crate for reusable role dispatch, statistics, simulation, promotion, provenance, and CLI composition. The implementation stages discovery evidence before operational gates and retain partial or unknown evidence.

**Tech Stack:** Rust workspace, existing `serde`, `serde_json`, `sha2`, and current Parquet/tape dependencies; no new dependency for pure statistics unless an existing dependency cannot express the required bounded operation.

**Spec:** `docs/SYSTEM_CHARTER.md`

## Global Constraints

- Preserve v1 serialization and generated schemas; additions are composable and compatibility-checked.
- Support 15+ instruments and 23 role-heterogeneous detectors through declared role semantics, without inventing adapters without a producer need.
- Preserve normalized raw and basis timestamps and fail closed on future, missing, contradictory, or unconfirmed inputs.
- Keep worker access denied for raw/private lake, peer, and holdout surfaces; never self-unlock confirmation.
- Record immutable source, binary, configuration, seed, and output identities without absolute paths.
- Do not hardcode scientific thresholds, returns, outcomes, or one universal lifecycle for every role.
- Do not run real scanners or alter science, holdout, or AP workspaces during implementation.

## Existing baseline and gap map

| Capability | Existing owner/evidence | Gap to fill |
| --- | --- | --- |
| Typed v1 experiment, run, evidence, exposure contracts and generated schemas | `crates/research-contracts/src/lib.rs`; `contracts/*_v1.schema.json` | Add composable role/operator/authority/holdout semantics without breaking v1 |
| Detector registry and instrument/source metadata | `registry/detectors_v1.json`, `instruments/` | Add semantic role and operator legality references; retain registry compatibility |
| Causal cursors, normalized timing, projected Parquet reads | `crates/research-tape/src/lib.rs` | Expose a stable engine input interface for raw+basis causal records |
| Development view materializer and verification | `crates/research-tape/src/development_view.rs`, `materialize_development_view` | Compose existing scope materializer into the CLI with explicit identity |
| Availability sidecar | `crates/research-tape/src/fvg_availability.rs`, `fvg_availability_sidecar` | Consume sidecar availability; never infer availability from occurrence |
| Sealed knowledge and workflow blinding | `crates/research-contracts/src/submission.rs`, `seal_submission`, `agent_harness/` | Add nested/future holdout exposure ledger and query immutable records |
| Role-dispatched atlas/question generation | None | Engine R role registry, atlas, and deterministic question generation |
| Reusable bounded statistics and nulls | None | Engine R stats/null interface with explicit parameters and abstentions |
| Method challenge and confirmation verification | Partial authority/sealing primitives | Add challenge/replay evidence and separate custodian confirmation boundary |
| Strategy/execution/cost/risk/walk-forward | None | Engine S/P typed method interfaces; current live confirmation remains locked |
| Portfolio/constraints/stress | None | Engine S/P composable methods and synthetic acceptance only |
| Typed provenance/run/report/knowledge querying | Partial v1 records | Compose immutable identities and query existing canonical records |

Boundary-audit seams to harden in the relevant stage: the development-view verifier must bind expected instrument, scope, and source and reject root/ancestor reparse escapes; the native launcher must set its declared current directory; immutable knowledge publication must be interruption-safe against partial artifacts; CLI output must satisfy declared-root containment and expected identity checks. The accepted Windows folder workflow remains the boundary; do not redesign it as an OS sandbox. A report-gate source junction escape is a targeted check if its existing contract permits it, not a harness rewrite.

## Staged tasks

### Task 1: Composable contracts, compatibility, and exposure invariants

**Files:**
- Modify: `crates/research-contracts/src/lib.rs`
- Modify: `crates/research-contracts/src/submission.rs` only where sealing metadata must carry the new typed evidence
- Modify: `contracts/` generated schemas through the existing `generate_schemas` path
- Test: `crates/research-contracts/tests/contracts.rs`, `crates/research-contracts/tests/submission_sealing.rs`

**Interfaces:**
- Produce `RoleId`, `Operator`, `AuthorityCompatibility`, `ExposureLedger`, and `ProvenanceIdentity` as serde types with explicit validation.
- Produce `validate_compatibility(v1, role, authority) -> Result<(), ContractError>` and `record_exposure(event) -> Result<(), ExposureError>`; nested future holdout exposure must fail closed.
- Reuse existing registry parsing, causal validation, sealing, and generated-schema ownership.

- [ ] Add failing tests for v1 round-trip compatibility, illegal role/operator combinations, nested holdout exposure, future availability, and path-independent identity.
- [ ] Implement the smallest composable types and validators; preserve v1 fields and error behavior.
- [ ] Regenerate schemas with `generate_schemas` and verify committed schemas match.
- [ ] Run targeted contract/sealing tests, then `cargo test --workspace` and `cargo fmt --all -- --check` with corpus/lake variables unset.

### Task 2: Engine R discovery core

**Files:**
- Create: `crates/research-engine/Cargo.toml`, `crates/research-engine/src/lib.rs`, `crates/research-engine/src/role.rs`, `crates/research-engine/src/stats.rs`, `crates/research-engine/src/provenance.rs`
- Modify: workspace `Cargo.toml`
- Test: `crates/research-engine/tests/research.rs`

**Interfaces:**
- Consume `research-contracts` validated inputs and `research-tape` causal records.
- Produce `RoleAtlas`, `Question { role, instrument, resolution, lag, sign, session, regime }`, `EvidenceRecord`, `NullResult`, and `run_research(input, plan) -> ResearchReport`.
- `RoleDefinition::operators()` declares legality; stats functions accept explicit method parameters and return `Known`, `Abstained`, or `Failed` evidence.

- [ ] Test a synthetic two-instrument, two-resolution role atlas, positive and negative signs, missing data abstention, bounded lag scan, and deterministic repeated-run identity.
- [ ] Implement reusable pure statistics with stdlib/installed dependencies, preserving raw metrics and unknowns.
- [ ] Add immutable provenance hashing that excludes absolute paths and includes source/binary/config/seed/output identities.
- [ ] Verify R tests and workspace baseline.

### Task 3: Engine S/P methods and confirmation boundary

**Files:**
- Create: `crates/research-engine/src/simulation.rs`, `src/walk_forward.rs`, `src/portfolio.rs`, `src/confirmation.rs`
- Test: `crates/research-engine/tests/simulation.rs`

**Interfaces:**
- Consume R evidence plus explicit method parameters and confirmed input manifests.
- Produce `SimulationReport`, `WalkForwardReport`, `PortfolioReport`, `StressReport`, and `PromotionEvidence` with separate coverage, abstention, cost, risk, and performance fields.
- `confirm(request, custodian_token) -> Result<Confirmation, ConfirmationError>` fails closed unless a separately authorized custodian signs; worker context has no unlock path.

- [ ] Test causal walk-forward partitions, explicit cost/risk/constraint parameters, missing-input abstention, deterministic portfolio aggregation, stress scenarios, and self-confirmation rejection.
- [ ] Implement pure replay/simulation and promotion evidence; keep live execution and live fills unavailable.
- [ ] Verify no method embeds scientific thresholds or outcomes.

### Task 4: CLI integration and synthetic end-to-end

**Files:**
- Create: `crates/research-engine/src/bin/research_engine.rs`
- Modify: `crates/research-tape/src/bin/materialize_development_view.rs` only if a stable library call is needed
- Test: `crates/research-engine/tests/end_to_end.rs`
- Check: `agent_harness/launch/isolated-run.ps1`, `agent_harness/validation/validate-isolation.ps1`

**Interfaces:**
- CLI consumes declared scope, synthetic inputs, plan, and output directory; it calls the existing development-view materializer and engine R/S/P APIs.
- CLI produces machine-readable report, provenance identity, exposure ledger, and explicit locked confirmation status.

- [ ] Run a fresh-process synthetic two-instrument R→S→P path through scope materialization, including one missing and one future input.
- [ ] Assert the report preserves partial evidence, abstentions, provenance, and locked confirmation; assert repeated runs match.
- [ ] Add synthetic regressions for expected instrument/scope/source binding, root and ancestor reparse rejection, launcher current-directory correctness, interruption-safe knowledge publish, declared-root output containment, and any report-gate source junction escape.
- [ ] Run isolation validation and full workspace tests with real corpus variables unset; do not invoke scanners.

### Task 5: Documentation, review, and release gate

**Files:**
- Create or update: `docs/SYSTEM_CHARTER.md`, `docs/RSP_IMPLEMENTATION_PLAN.md`, `docs/RSP_CONTRACTS.md`, `docs/RSP_ENGINE_R.md`, `docs/RSP_ENGINE_SP.md`, `docs/RSP_PROVENANCE.md`, `docs/RSP_HOLDOUT_BOUNDARY.md`, `docs/RSP_CONFIRMATION_BOUNDARY.md`, and `docs/RSP_ACCEPTANCE.md`
- Review: all changed contracts, engine, tape integration, and `agent_harness` boundaries

- [ ] Document interfaces, evidence states, provenance, holdout ledger, confirmation custody, and deferred capabilities without claiming empirical results.
- [ ] Run `cargo fmt --all -- --check`, targeted tests, full workspace tests, isolation validation, and fresh-process deterministic replay.
- [ ] Perform an independent defect-focused review; classify every failure or unavailable check as `not_run`, infrastructure failure, or analytical failure.
- [ ] Commit only after docs, code, and evidence checks are complete; preserve v1 and leave live confirmation locked.

## Deliberate deferrals

The plan does not add 21 detector adapters without producer semantics, real studies or alpha, holdout exposure, AP changes, an OS sandbox rebuild, live brokers/fills, distributed execution, or a graph database. Add those only when an approved producer contract and acceptance test require them.
