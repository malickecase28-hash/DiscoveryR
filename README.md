# TrinityR Research Program

DiscoveryR is the reusable Rust research workspace for TrinityR.

The analytical lake is external, frozen input and is not stored in Git. The repository now contains three bounded layers:

- `research-contracts` — semantic, authority, evidence, holdout, reproducibility, strategy and portfolio contracts.
- `research-tape` — causal/native-scale Parquet access, development-view materialization, physical-schema validation and availability sidecars.
- `research-engine` — reusable Program R/S/P orchestration, blind question templates, bounded statistics/nulls, challenge and confirmation boundaries, strategy simulation, portfolio research, knowledge publication, reporting manifests and CLI adapters.

The system preserves one causal information clock while retaining native scale and occurrence provenance. Program R (market research), Program S (strategy/alpha research), and Program P (portfolio research) are separate scientific programs; confirmation is externally custodied and cannot be unlocked by a research worker.

## Canonical infrastructure CLI

Use the `trinity_research` binary for new reusable workflows:

```text
trinity_research validate-instrument <input.json> [output.json]
trinity_research authority-compatibility <input.json> [output.json]
trinity_research materialize-scope <input.json> [output.json]
trinity_research generate-questions <input.json> [output.json]
trinity_research run-experiment <input.json> [output.json]
trinity_research verify-result <input.json> [output.json]
trinity_research challenge-result <input.json> [output.json]
trinity_research confirm-frozen-claim <input.json> [output.json]
trinity_research generate-report <input.json> [output.json]
```

`generate-questions` consumes frozen phenotype identity, ontology/lineage metadata, permissions and explicit anchor/context scale domains. It has no observed anchor timestamp, outcome or prior-winner input surface. Actual per-anchor availability is checked by execution.

`confirm-frozen-claim` is intentionally fail-closed in the worker CLI. Authority-bearing confirmation is performed only through the externally signed custodian path in `ConfirmationRunner::confirm_challenged`.

The older `research_engine` binary is retained as a synthetic/backward-compatibility harness. It is not the canonical production-facing CLI.

## Program R execution boundary

The canonical generic Program R runner is population-based: one experiment may contain many anchor instances at different causal times, and anchor/context native scales are preserved separately. Missing context remains in the anchor denominator as explicit missingness rather than silently removing the anchor. Candidate eligibility requires an explicit predeclared gate.

Generic numeric kernels are infrastructure only. They cannot be labeled as lifecycle, nesting, spatial, regime, interaction or incremental-information analyses unless a typed detector-specific adapter actually implements that semantic relationship. High-volume detector-specific E1/E2/E3 scanners remain streaming Rust adapters; the generic population engine consumes their bounded typed observations/results.

See `docs/SYSTEM_CHARTER.md`, `docs/SCIENTIFIC_ONTOLOGY.md`, `docs/RSP_IMPLEMENTATION_PLAN.md`, and the Program R/S/P documents for the governing boundaries.

No real behavioral experiment, holdout consumption, alpha claim, live execution, or portfolio allocation is performed by the reusable infrastructure itself.
