# AP-001 isolated agent harness

This harness keeps scientific definitions in this repository and keeps live
agent workspaces under F:\TrinityR-runs, outside the Git worktree.

The launcher requires WSL2. It starts a root WSL mount namespace, bind-mounts
the repository and XAUUSD lake read-only, bind-mounts only the selected role
workspace read-write, removes the host C: and F: mounts, then drops to the
unprivileged nobody user before running the command.

The harness is an operational boundary, not a provider/model scheduler.
Provider identity must remain in an orchestrator-owned system outside the
scientific artifacts.

Example:

    .\agent_harness\launch\isolated-run.ps1 -Role A-01 -RunRoot F:\TrinityR-runs -Command 'id; test -r /shared/research-program/protocol/RESEARCH_RULES.md'

Run the adversarial checks with:

    .\agent_harness\validation\validate-isolation.ps1

The command fails closed when WSL2 is unavailable or the run root is inside
the repository.
