# Agent harness status

The WSL-based agent harness is retired on the direct-Windows branch.

Do not launch `agent_harness/launch/isolated-run.ps1` or
`agent_harness/launch/isolated-run.sh`, reinstall WSL distributions, or create
`F:\TrinityR-runs`. Those files remain in Git history for auditability only;
they are not the active execution path.

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

See `docs/operations/WORKSPACE_INDEX.md` and `protocol/RESEARCH_RULES.md` for
the active layout and lifecycle rules.

This branch does not claim to provide a native Windows isolation boundary or
provider scheduler. A future native boundary must be introduced as a separate,
reviewed change with its own verification.
