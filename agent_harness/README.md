# Agent harness status

The WSL-based agent harness is retired on the direct-Windows branch. The
PowerShell launcher runs approved Windows commands from the declared folder
run root `.runs\wave1-native-v1\<PROGRAM>\<ROLE>`. The former shell launcher
is removed.

Do not reinstall WSL distributions or create `F:\TrinityR-runs`.

Operational status: `NATIVE_WINDOWS_WORKSPACE_ONLY`, `WSL_RUNTIME_RETIRED`,
`NO_NEW_SCRATCH_ROOTS`.

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

Run the policy, assignment, pair-parity, and folder-launch checks with:

    .\agent_harness\validation\validate-isolation.ps1

See `docs/operations/WORKSPACE_INDEX.md` and `protocol/RESEARCH_RULES.md` for
the active layout and lifecycle rules.

The folder run is workflow isolation only. It does not claim to provide a hard
Windows filesystem/process, credential, peer, or host-data boundary. Do not
copy credentials into workspaces; host validation and publication remain
separate steps.
