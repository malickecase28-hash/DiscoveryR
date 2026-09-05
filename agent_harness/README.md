# Agent harness status

The WSL-based agent harness is retired on the direct-Windows branch. The
PowerShell launcher runs approved Windows commands from the declared folder
run root `.runs\wave1-native-v2\<PROGRAM>\<ROLE>`. The former shell launcher
is removed.

Do not reinstall WSL distributions or create `F:\TrinityR-runs`.

Every research role must start in a fresh directory. A non-empty role directory
is rejected as `NATIVE_WORKSPACE_NOT_FRESH`; prior runs are never overwritten.

Operational status: `NATIVE_WINDOWS_WORKSPACE_ONLY`, `WSL_RUNTIME_RETIRED`,
`NO_NEW_SCRATCH_ROOTS`.

## Active workflow

- Work directly in Windows sessions.
- Use the canonical repository at `F:\TrinityR-research\Research Program`.
- Create temporary work only under a declared phase/run directory or another
  explicitly registered project path.
- Keep each worker's files in its assigned private workspace.
- Researchers write only `authority_candidate.json` and `authority_review.md`.
- Researchers do not validate or self-certify their reports.
- The host runs the canonical report gate after researchers stop.
- Invalid reports are not synthesized or published.
- After host validation and signoff, retain only the required published
  artifacts, manifests/receipts, and reusable scripts; delete the temporary
  workspace.
- Do not create disposable top-level folders under `F:\`.

Run the policy, assignment, pair-parity, and folder-launch checks with:

    .\agent_harness\validation\validate-isolation.ps1

Run the report-gate regression suite with:

    .\agent_harness\validation\test-validate-authority-report.ps1

Validate all eight Wave-1 authority workspaces with one host command:

    .\agent_harness\validation\validate-wave1-authority-reports.ps1

A researcher's own statement that its output "passes" has no gate authority.
Only `validate-authority-report.ps1` / `validate-wave1-authority-reports.ps1`
determine report acceptance.

See `docs/operations/WORKSPACE_INDEX.md` and `protocol/RESEARCH_RULES.md` for
the active layout and lifecycle rules.

The folder run is workflow isolation only. It does not claim to provide a hard
Windows filesystem/process, credential, peer, or host-data boundary. Do not
copy credentials into workspaces; host validation and publication remain
separate steps.
