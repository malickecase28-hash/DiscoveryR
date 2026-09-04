# Generic isolated agent harness

This harness keeps scientific definitions in this repository and keeps live
agent workspaces under F:\TrinityR-runs, outside the Git worktree.

The launcher requires WSL2. It reads the authority assignment from
`assignments/<ProgramId>/<Role>.json`, bind-mounts only its exact repository and
authority surfaces read-only, bind-mounts `F:\TrinityR-runs\<ProgramId>\<Role>`
read-write, and drops to the unprivileged nobody user inside private mount/PID
and proc namespaces.

AUTHORITY receives committed schema/inventory metadata and no raw lake.
DEVELOPMENT requires the external development view declared by
instruments/XAUUSD/data_scope_v1.json. CONFIRMATION is always rejected until a
separate explicit freeze action is implemented.

The harness is an operational boundary, not a provider/model scheduler.
Provider identity must remain in an orchestrator-owned system outside the
scientific artifacts.

Example:

    .\agent_harness\launch\isolated-run.ps1 -ProgramId AP-001 -Role A-01 -RunRoot F:\TrinityR-runs -Command 'id; test -r /shared/research-program/protocol/RESEARCH_RULES.md'

Run the adversarial checks with:

    .\agent_harness\validation\validate-isolation.ps1

The command fails closed when WSL2 is unavailable or the run root is inside
the repository.
