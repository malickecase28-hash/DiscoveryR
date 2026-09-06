# Reproducibility

Every material artifact carries a `ReproducibilityIdentity`. It covers the
instrument, producer commit and source/blob identities, authority version,
detector and parameter identities, source and payload manifests, data scope,
experiment contract, code commit, scanner version, control and null designs,
random seed, and output logical hash. Absolute Windows paths are excluded.

Inputs are read-only and identified by content. Scope, availability, native
scale, timezone, warmup, exclusions, and transformations are recorded. A fresh
process can materialize the same scope and verify the same logical identity.
Missing checks are reported as `not_run`; a timeout is not a successful result.

The command surface is intentionally explicit:

`validate instrument` checks the typed scope. The tape materializer is the
operation that creates a declared development view and defaults to content
copies. The engine CLI returns `not_run` for execution, replay, challenge,
confirmation, and arbitrary report wrapping when their typed runner/custody
boundary is not supplied; it never reports validation as execution. The library
APIs `run_research`, `method_challenge_protocol`, and the bound confirmation
functions provide the synthetic infrastructure path. Output publication is
relative, no-replace, and rejects final symlink/reparse targets.

Reproducibility proves the computation and its inputs. It does not prove a
scientific claim, profitability, or future performance.
