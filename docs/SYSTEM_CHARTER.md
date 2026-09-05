# R→S→P System Charter

This charter defines the infrastructure boundary for deterministic research execution. It supports discovery, validation, and promotion as separate evidence states; it does not grant execution authority.

## Scope and ownership

The workspace keeps two existing crates and adds one engine crate:

| Surface | Owns | Must not own |
| --- | --- | --- |
| `research-contracts` | Small composable semantic contracts, compatibility checks, schemas, provenance, authority and holdout invariants | Parquet I/O, statistical algorithms, broker behavior |
| `research-tape` | Parquet projection, causal/as-of reads, normalized raw and basis timing, development views, availability sidecar, sealed knowledge/workflow blinding | Detector-specific science, activation decisions |
| `research-engine` | Deterministic R/S/P execution, role dispatch, bounded statistics/nulls, challenges, reproducibility identity, walk-forward/portfolio methods, CLI orchestration | Canonical knowledge storage, self-unlocking confirmation, live orders/fills |

Existing v1 contracts, registries, causal cursors, projected reads, view builder, sidecar, sealed knowledge, and workflow blinding remain valid and readable. New fields are additive or versioned; v1 serialization and generated schemas remain unchanged unless a compatibility proof is committed.

## Authority and causality

Every input and output carries immutable source, binary, configuration, seed, and output identities. Identity material excludes absolute paths and machine-local locations. Raw and basis timestamps are both retained; a context is usable only when its declared availability time is at or before the decision time. Missing, late, contradictory, or unavailable inputs produce explicit abstention or failure records.

Workers may read only their assigned scope and may never read raw/private lake material, peer workspaces, holdout data, or future context. A future exposure ledger records nested exposure and holdout violations as evidence. Confirmation requires a separately authorized custodian boundary and fails closed; a worker cannot unlock its own confirmation. The current real CLI remains locked.

## R/S/P meanings

R (research/discovery) characterizes role-specific relationships and methods across declared instruments, resolutions, lags, sessions, regimes, signs, and nulls. It preserves partial evidence and unknowns and does not hardcode scientific thresholds, returns, or outcomes.

S (simulation/validation) replays declared methods with causal availability, explicit costs and risk assumptions, walk-forward partitions, and input confirmation. It reports failure, abstention, and coverage separately from performance.

P (promotion) evaluates reproducible evidence against an explicit promotion policy and emits a bounded report. Promotion does not authorize live execution. Strategy, execution, cost, risk, portfolio, constraints, and stress methods are typed infrastructure; their parameters are explicit and their empirical results remain unknown until supplied data are run.

## Non-goals and deferrals

No real study, alpha, holdout exposure, AP change, OS sandbox rebuild, live broker integration, live fills, distributed execution, graph database, or invented 21-detector adapter set is part of this system. A role gets lifecycle support only when a producer gives it a declared semantic need. Knowledge queries reuse immutable records rather than creating a second canonical store.

## Acceptance boundary

The implementation is acceptable when a fresh process can run a synthetic two-instrument R→S→P path through the existing scope materializer, preserve v1 contracts, reject future/nested holdout exposure, produce deterministic identities across path changes, abstain on missing inputs, and leave confirmation locked. Real corpus and scientific scanners are outside acceptance and must not run in this phase.
