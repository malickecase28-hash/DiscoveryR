# TrinityR Environment Map

Observation scope: `F:\TrinityR-research`, recorded 2026-09-03. The parent directory is not a Git repository. The legacy `Research Directive` repository and all other parent-level components were inspected read-only.

## Top-level map

| Path | Observed purpose | Status | Git status | Likely relationship | Cautions |
|---|---|---|---|---|---|
| `Research Program` | New bootstrap repository created by this task | active | New repository; initial commit/push performed after bootstrap | New research-program home | Architecture is intentionally unfrozen; external data stays outside |
| `analytical_lake` | Frozen analytical data tree; `fusion_markets/xauusd` contains native `tick`, `15s`, `30s`, `1m`, `5m`, `15m`, `1h`, and `4h` directories | data | No Git repository observed | Read-only scientific input/reference | Do not copy, hash recursively, or modify Parquet data |
| `Research Directive` | Legacy research protocol, reports, authority material, stream labs, and small Rust validation tools | historical | Git repository; `main` tracks `origin/main`; untracked `.serena/` and `reports/` were observed | Reference and selective future reuse only | Do not modify or automatically carry over its methodology, strategies, or stream isolation |
| `schemas` | JSON schema and contract material, including bars, manifests, experiments, hypotheses, metrics, studies, and strategy records | schema/contract | No Git repository observed | Reference; selective migration requires ownership decision | Do not change schemas during bootstrap |
| `duckdb` | DuckDB database plus SQL bootstrap/views and a Python initialization script | data/infrastructure | No Git repository observed | Reference; active-vs-legacy status is unresolved | Do not open, modify, or version the database bytes |
| `config` | State/manifest JSON plus a Google credentials template | configuration | No Git repository observed | Read-only reference; credentials must remain external | Credential-related material was not exposed |
| `google_service` | Python Google Drive/ledger/book/export service modules and cached bytecode | external-service integration | No Git repository observed | Reference or selective future adapter reuse | Authentication and service boundaries require review; no secrets were read |
| `.codex-openrouter` | Local agent/runtime state, sandbox directories, plugin cache, SQLite state, and session/log material | agent/harness tooling | No top-level Git repository observed | Exclude from research repository | Contains sandbox-secrets and runtime state; values were not read or copied |
| `All Read.md` | Launch/instruction text for an XAUUSD native-stream research process | unknown / historical instruction | Parent is not a Git repository | Archive/reference only until its authority is clarified | It directs work in the legacy protocol and must not silently define the new architecture |

## Analytical lake detail

**OBSERVED**

- Root layout includes `analytical_lake/fusion_markets/xauusd` and `analytical_lake/logs`.
- XAUUSD has eight native resolution directories: `tick`, `15s`, `30s`, `1m`, `5m`, `15m`, `1h`, and `4h`.
- Each observed resolution directory contains six Parquet part files. The `1m` filenames were `part-00000.parquet` through `part-00005.parquet`; other resolution directories were counted without enumerating all Parquet names.
- XAUUSD has `manifest.json` and `payload_manifest.json` at the instrument root.
- The manifest exposes instrument, timeframe, warmup, source-file, row, schema, payload-contract, authority, timing, qualification, and artifact-scope fields. It declares seven timeframes plus tick payload metadata.

**INFERRED**

- The XAUUSD directory is a plausible structural template for later instruments, subject to schema and authority checks.
- The manifests are the likely starting point for provenance and data-access mapping, but canonical ownership is not yet established.

**UNKNOWN**

- Whether all future instruments use the same partitioning, filenames, contracts, and manifest semantics.
- Whether `duckdb` is canonical active infrastructure or legacy support for the lake.

## Legacy directive architecture

**OBSERVED**

- The legacy repository has a lean six-gate state machine (`G0` through `G5`), stream-lab directories for the eight native resolutions, supervisor/orchestration policy, authority maps, reports, strategy categories, templates, and Rust binaries.
- The Rust package is `trinity-research-tools`, using `serde` and `serde_json`; source includes validators for repository state, manifests, gate submissions, authoritative GO decisions, and review challenges.
- Legacy documentation explicitly distinguishes causal availability from occurrence time, preserves provenance, uses Git checkpoint/freeze concepts, and protects validation data.

**INFERRED**

- Causal/provenance safeguards and validation tooling may be reusable concepts.
- The gate/state-machine and isolated-stream organization are coupled to the legacy protocol and require deliberate re-evaluation against the new causal-clock/lifecycle direction.

**UNKNOWN**

- Which legacy artifacts remain scientifically valid for the new program.
- Which Rust utilities are general enough to become shared infrastructure.

## Harness and sensitive surfaces

**OBSERVED**

- `.codex-openrouter` contains sandbox directories, a sandbox-secrets directory, local config/state databases, sessions, logs, skills, and plugin-related directories.
- `config` contains a Google credentials template; `google_service` contains authentication/service modules.
- No Docker, WSL, provider-specific project definition, or enforceable per-agent research workspace mechanism was established from the bounded filename observation.

**INFERRED**

- The local runtime may provide operational sandboxing, but its enforcement and suitability for blinded research agents are not proven by this observation.

**UNKNOWN**

- Whether the current runtime can enforce per-agent read-only shared inputs and isolated writable outputs.
- How GLM, OpenAI/Codex, Gemini, Claude, MCP, and OpenRouter identities would be hidden or routed in a future research harness.

No secrets, credential values, service-account material, tokens, or private keys were copied into this repository.
