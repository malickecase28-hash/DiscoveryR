# R→S→P reusable infrastructure — implementation status

This document is the current implementation map for `architecture/full-rsp-system`. It replaces the earlier task checklist whose partial boxes no longer matched the code.

The governing rule remains: build reusable infrastructure only. Real detector experiments, holdout consumption, alpha claims, live execution, and real portfolio allocation are outside this branch.

## Architecture

```text
research-contracts
    semantic/authority/evidence/holdout/reproducibility contracts
        ↓
research-tape
    causal/native-scale physical access and development materialization
        ↓
research-engine
    Program R/S/P orchestration, statistics, challenge, confirmation boundary,
    strategy/portfolio research, knowledge, provenance, reporting
        ↓
agent_harness / submission tooling
    workflow access, sealed publication, custody policy
```

## Completed reusable surfaces

- [x] Small composable semantic contracts for instruments, authority, timing, lifecycle, lineage, anchors, context permissions, outcomes, evidence, strategy, portfolio, and reproducibility.
- [x] Ten detector roles and eleven relationship-operator families.
- [x] Detector Atlas role dispatch and lineage validation.
- [x] Authority Compatibility Gate returning `AUTHORITY_COMPATIBLE` or scoped `AUTHORITY_DELTA_REQUIRED` surfaces.
- [x] Native-scale preservation and causal availability enforcement.
- [x] Development-view materialization with expected scope/source identities and revocation/tamper checks.
- [x] Nested/disjoint holdout policy contracts.
- [x] Bounded Rust statistics: streaming moments, quantiles, effect sizes, exact-strata matching, block bootstrap/permutation, grouped circular shifts, BH/BY FDR, max-statistic control, support concentration, and survival/hazard utilities.
- [x] Blind bounded question-generation infrastructure without prior-winner inputs.
- [x] Program R synthetic/compatibility runner retained for existing tests.
- [x] Canonical population Program R runner allowing many anchor times, per-anchor causal context validation, separate anchor/context native scales, and explicit predeclared candidate gates.
- [x] Method-challenge protocol separated from confirmation.
- [x] Externally authenticated confirmation runner with signature verification, exact binding, expiry and replay protection; worker-side activation remains locked.
- [x] Program S strategy contracts, causal simulation, execution/cost/risk modeling and walk-forward infrastructure.
- [x] Program P confirmed-input portfolio descriptions: aligned causal streams, correlation/conditional correlation, overlap, drawdown, capacity, turnover, liquidity, concentration, risk budgets, constraints and stress reports.
- [x] Immutable/queryable knowledge publication with restart-safe supersession rules.
- [x] Complete path-independent reproducibility identities.
- [x] Deterministic report-manifest generation bound to artifact bytes and reproducibility identity.
- [x] Canonical `trinity_research` CLI for instrument validation, authority compatibility, scope materialization, bounded population experiment execution, result verification, non-authority challenge execution, locked confirmation preflight and report-manifest generation.
- [x] Required R/S/P, ontology, onboarding, holdout, agent and reproducibility documentation.
- [x] Portable GitHub Actions workflow for format/clippy/test gates on Windows and Linux. Repository/account runner availability is an external prerequisite, not scientific authority.

## Deliberately not universalized

The generic engine must not pretend every semantic relationship can be measured by one numeric kernel. Detector-specific lifecycle, structural, event, quality and normalization semantics remain typed adapter/scanner responsibilities. The generic population runner supplies only explicitly declared generic numeric metrics and never replaces detector-specific E1/E2/E3 implementations.

The older `research_engine` binary remains a synthetic/backward-compatibility harness. `trinity_research` is the canonical new workflow CLI.

## Intentionally deferred external/scientific work

These are not infrastructure defects and must not be auto-implemented from this branch:

- Real AP-001/AP-002 or other detector behavioral runs.
- The remaining producer-semantic detector adapters until each detector's role requires one.
- Real confirmation holdout access or deployment of the external custodian/signing service.
- Live broker/fill integration and real execution assumptions.
- Real strategy optimization or real portfolio capital allocation.
- Hostile OS sandboxing beyond the accepted workflow-access model.
- Distributed orchestration and graph-database deployment unless later scale proves they are needed.

## Merge acceptance gate

Before this branch is merged to `main`, run from a clean checkout:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
git diff --check main...HEAD
```

Also verify:

1. the canonical CLI can execute a synthetic population experiment with anchor instances at more than one timestamp;
2. future context is rejected per anchor;
3. cross-scale anchor/context identity is preserved rather than intersected away;
4. candidate creation is impossible without an explicit predeclared gate;
5. report verification is bound to actual artifact bytes;
6. worker confirmation remains locked;
7. V1/Wave-1 authority artifacts and AP-001/AP-002 scientific branches are untouched;
8. no real holdout or behavioral outcome has been consumed.

## Multi-instrument target flow

```text
instrument configuration
→ source validation
→ authority compatibility
→ investigate only changed authority surfaces if any
→ freeze data/holdout scope
→ materialize development view
→ run existing typed adapters/scanners and generic R/S/P infrastructure
→ verify/challenge/freeze artifacts
```

A compatible second instrument must not repeat global authority archaeology.
