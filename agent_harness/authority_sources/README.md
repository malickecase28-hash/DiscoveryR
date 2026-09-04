# XAUUSD Wave 1 authority bundles

`XAUUSD.wave1.spec.json` is a positive extraction specification. The host-side
generator writes outcome-free bundles outside the repository under
`TRINITYR_AUTHORITY_ROOT` (the default is `F:\TrinityR-authority`). The
approved payload source is supplied through `TRINITYR_PAYLOAD_CONTRACT_SOURCE`
and only `/declared_tick_payload_contract` is selected.

AP-001 and TC-001 receive declared serialization contracts, but lifecycle and
producer semantics remain unresolved. AP-002 has no approved FVG contract.
BG-001 includes only the declared canonical-bar schema fields and does not
infer lineage. Bundle directories are atomically created and use the shared
recursive transport digest; `bundle_manifest.json` is excluded to avoid a
self-referential hash.
