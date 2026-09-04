# XAUUSD Wave 1 authority bundles

`XAUUSD.wave1.spec.json` is a positive extraction specification. The host-side
generator writes outcome-free bundles outside the repository under
`TRINITYR_AUTHORITY_ROOT` (the default is `F:\TrinityR-authority`).

The approved schema root contains no detector producer contracts for AP-001,
AP-002, or TC-001, and no detector dependency/lineage contract for BG-001.
Those claims remain `UNRESOLVED_PENDING_SOURCE_APPROVAL`. BG-001 includes only
the declared canonical-bar schema fields; it does not infer lineage.
