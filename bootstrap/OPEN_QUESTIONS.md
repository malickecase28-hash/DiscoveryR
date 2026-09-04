# Open Questions

These questions were not answerable safely from the bounded read-only observation.

- Which directory is canonical for schemas and data manifests: the root-level `schemas`, the legacy `Research Directive` authority material, or the lake's own manifests?
- Is `duckdb/trinity_research.duckdb` active research infrastructure, a cache, or legacy support? What is its intended read-only access contract?
- Which Rust validators and Serde types are general reusable infrastructure versus protocol-specific legacy code?
- What mechanism can enforce one isolated writable workspace per research agent while exposing only read-only shared scientific inputs?
- Can the current Codex/OpenRouter runtime enforce filesystem boundaries, hidden provider identity, and blinded peer-agent outputs, or is a separate harness required?
- Which provider/model routing surfaces exist for GLM, OpenAI/Codex, Gemini, Claude, MCP, and OpenRouter, and which metadata must be hidden during blind phases?
- Which configurations may be exposed to future agents without exposing Google credentials, database connection details, tokens, or other sensitive material?
- Are all future instruments expected to follow the observed XAUUSD lake partitioning, resolution set, manifest fields, and payload contracts?
- What is the authoritative instrument set and how should new instruments register their provenance and access scope?
- Which legacy causal safeguards remain valid unchanged, and which state-machine, timeframe-isolation, strategy, and agent-policy assumptions must be retired or redesigned?
- What exact scope should Refactor Stage 1 cover after human review of this inventory?
