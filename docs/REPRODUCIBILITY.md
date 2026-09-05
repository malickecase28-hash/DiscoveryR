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

The command surface is intentionally small:

`validate instrument` checks source and authority contracts; `materialize scope`
creates a declared development view; `run experiment` executes a frozen
contract; `verify result` replays and compares identities; `challenge result`
attaches independent method evidence; `confirm frozen claim` is available only
to an externally authorized custodian; `generate report` emits a path-independent
machine-readable report.

Reproducibility proves the computation and its inputs. It does not prove a
scientific claim, profitability, or future performance.
