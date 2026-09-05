# XAUUSD Wave 1 authority bundles

`XAUUSD.wave1.spec.json` is a positive extraction specification. The host-side
generator writes outcome-free bundles outside the repository under
`TRINITYR_AUTHORITY_ROOT` (the default is `F:\TrinityR-authority`). The
approved payload source is
`F:\TrinityR-research\analytical_lake\fusion_markets\xauusd\payload_manifest.json`
with SHA-256
`78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed`; only
`/declared_tick_payload_contract` is selected.

AP-001 and TC-001 receive declared serialization contracts, but lifecycle and
producer semantics remain unresolved. AP-002 has no approved FVG contract.
BG-001 includes only the declared canonical-bar schema fields and does not
infer lineage. Bundle directories are atomically created and use the shared
recursive transport digest, including `bundle_manifest.json`.
