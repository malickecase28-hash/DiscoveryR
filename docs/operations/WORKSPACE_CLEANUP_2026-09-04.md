# Workspace cleanup — 2026-09-04

## Decision

WSL is retired for this project. Future work uses direct Windows folders on `F:`. No project worker or research launch may assume a WSL distribution exists.

## Preserved roots

These roots were deliberately not moved or deleted:

- `F:\TrinityR-research` — active research/data workspace
- `F:\TrinityR-runs\wave1-v3-custody` — canonical Git custody repository
- `F:\TrinityR-authority`
- `F:\TrinityR-manifests`
- `F:\TrinityR-runs`
- `F:\TrinityR-research-workspaces`
- `F:\TrinityR`
- `F:\TrinityR-legacy-run`
- `F:\TrinityR-views`

They contain the active research workspace, canonical custody repository, authority material, manifests, active runs, research workspaces, or data/history.

## Purged staging roots

The dated correction, rework, push-target, harness, dry-run, empty staging roots, superseded local report copies, smoke fixtures, ZIPs, and prior test manifests were removed after explicit signoff.

The temporary archive `F:\TrinityR-archive\20260904-staging` was also removed. Those disposable materials are not retained locally; authoritative committed/published copies remain where applicable.

## WSL retirement

The following stopped distributions were unregistered after process and state checks:

- `Ubuntu`
- `TrinityR-GLM-AP001-A01`
- `TrinityR-GLM-BG001-A02`
- `TrinityR-GLM-AP002-A02`
- `TrinityR-GLM-TC001-A01`

`wsl --list --verbose` now reports no installed distributions. Their virtual-disk contents were removed by the unregister operation and were not copied into the project archive.

## Safety record

- Active Windows processes were not terminated.
- Current research, authority, manifest, run, workspace, and data roots were not touched.
- Active program directories and current research descendants were not touched.
- The lifecycle rule was added to `protocol/RESEARCH_RULES.md`.
- V4 custody commit `9134975bc04cf7450268677f5f9cf32f756d83f1` remains the implementation base.

## Operating layout going forward

Use only the preserved canonical roots for active work. New temporary material must be created under `F:\TrinityR-runs` or a dated subdirectory of `F:\TrinityR-archive`; do not create new top-level correction/rework folders.
