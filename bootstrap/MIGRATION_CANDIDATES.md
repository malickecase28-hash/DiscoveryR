# Migration Candidates

Nothing is migrated during bootstrap. These are observations for later review, not an architecture decision.

## KEEP / REUSE CANDIDATES

- **Causal availability and provenance safeguards** from the legacy start map and governing principles. They explicitly distinguish occurrence from known/available time and require reproducible evidence. Reuse only after checking fit with the new causal-clock/lifecycle direction.
- **Git identity and checkpoint concepts** from the legacy Git protocol. Exact commit identity and protected validation are useful reproducibility candidates, but the future workflow is not decided here.
- **Small Rust Serde/JSON validators** in `Research Directive/src`. They are concrete, typed tooling candidates for later inspection; no code is copied now.
- **Lake manifest and payload-manifest concepts** under `analytical_lake/fusion_markets/xauusd`. They expose provenance and artifact-scope fields that may support future data-access contracts.

## REFACTOR CANDIDATES

- **Legacy G0-G5 state machine.** It provides process vocabulary and review gates, but its lifecycle and stream assumptions may not match the new anchor-based research direction.
- **`stream_labs/<timeframe>` organization.** Native timeframe coverage is useful evidence, but treating each timeframe as an isolated universe is explicitly a legacy assumption requiring re-evaluation.
- **Authority, access, and manifest validators.** The validation boundaries may be reusable, while paths, schemas, instrument scope, and lifecycle semantics likely need generalization.
- **Supervisor/orchestration policy.** It contains useful review and context-boundary ideas, but provider identity hiding and enforceable filesystem isolation remain unproven.

## ARCHIVE-ONLY CANDIDATES

- **Legacy reports, journals, and historical conclusions.** Preserve as historical evidence; do not preload automatically into new agent context.
- **Strategy-category documents and matured strategy artifacts.** They belong to the legacy research history and must not seed current detector/lifecycle discovery.
- **Known-failure regression material.** Retain as adversarial historical reference; do not treat it as automatically authoritative for the new model.
- **`All Read.md`.** Preserve as a historical launch instruction until its authority and applicability are explicitly reviewed.

## EXCLUDE / DO-NOT-MIGRATE CANDIDATES

- **Parquet data and the analytical lake itself.** Keep external and frozen; reference or mount it read-only later.
- **`duckdb/trinity_research.duckdb` and generated database state.** Do not version or alter database bytes during bootstrap.
- **`.codex-openrouter` runtime state, sandbox state, secrets, sessions, logs, caches, and plugin internals.** These are environment-owned operational surfaces, not research source.
- **Credential/configuration values** from `config` or `google_service`. Keep secrets outside Git and expose only an explicitly reviewed interface later.
- **Legacy strategy execution or migration code** unless a later review identifies a narrowly scoped, evidence-backed reuse case.
