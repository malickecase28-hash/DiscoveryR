# Agent harness status

The WSL-based agent harness is retired on the direct-Windows branch. The
PowerShell launcher is retained only as a fail-closed guard: it reports
`CODEX_HARD_RUNTIME_BLOCKED` and starts no process. The former shell launcher
is removed.

Do not reinstall WSL distributions or create `F:\TrinityR-runs`.

## Active workflow

- Work directly in Windows sessions.
- Use the canonical repository at `F:\TrinityR-research\Research Program`.
- Create temporary work only under a declared phase/run directory or another
  explicitly registered project path.
- Keep each worker's files in its assigned private workspace.
- After host validation and signoff, retain only the required published
  artifacts, manifests/receipts, and reusable scripts; delete the temporary
  workspace.
- Do not create disposable top-level folders under `F:\`.

Run the read-only policy, assignment, and pair-parity checks with:

    .\agent_harness\validation\validate-isolation.ps1

See `docs/operations/WORKSPACE_INDEX.md` and `protocol/RESEARCH_RULES.md` for
the active layout and lifecycle rules.

This branch does not claim to provide a native Windows isolation boundary or
provider scheduler. A future native boundary must be introduced as a separate,
reviewed change with its own verification.
