# Generic isolated agent harness

This harness keeps scientific definitions in this repository and keeps live
agent workspaces under F:\TrinityR-runs, outside the Git worktree.

The launcher requires WSL2. It reads the authority assignment from
`assignments/<ProgramId>/<Role>.json`, bind-mounts only its exact repository and
authority surfaces read-only, bind-mounts `F:\TrinityR-runs\<ProgramId>\<Role>`
read-write, and drops to the unprivileged nobody user inside private mount/PID
and proc namespaces.

AUTHORITY receives committed schema/inventory metadata and no raw lake.
DEVELOPMENT assignments declare `instrument_id` and `data_scope_id`; the
launcher resolves the committed scope and requires a matching external
`view_manifest.json` under `TRINITYR_VIEWS_ROOT` (default `F:\TrinityR-views`).
CONFIRMATION is always rejected until a separate explicit freeze action is
implemented.

The harness is an operational boundary, not a provider/model scheduler.
Provider identity remains in an orchestrator-owned runtime slot configuration
outside the scientific artifacts. Use `-RuntimeSlot slot-07` with
`TRINITYR_RUNTIME_CONFIG`; the private mapping is never mounted into the
research namespace.

Example:

    .\agent_harness\launch\isolated-run.ps1 -ProgramId AP-001 -Role A-01 -RunRoot F:\TrinityR-runs -Command 'id; test -r /shared/research-program/protocol/RESEARCH_RULES.md'

Run the adversarial checks with:

    .\agent_harness\validation\validate-isolation.ps1

The command fails closed when WSL2 is unavailable or the run root is inside
the repository.

## Submission lifecycle

Researcher completion is not a Git operation. The agent leaves its two required
artifacts in the private workspace and stops. A host operator then validates and
publishes them with `publish-submission.ps1`, which creates a dedicated review
branch and pushes only when `-Push` is supplied.

Examples:

    .\agent_harness\publication\publish-submission.ps1 `
      -ProgramId BG-001 -Role A-02 `
      -BaseRef <immutable-commit-sha> -ResearchBaseSha <same-commit-sha> `
      -SubmissionPath '\\wsl.localhost\<distro>\home\glm\workspace' `
      -Push

Codex run roots and WSL workspaces are both accepted as submission paths. The
publisher rejects extra files, identity mismatches, mutable/mismatched base
references, invalid claims/evidence, and explicit operational identity names
before creating the branch. Scientific phrases such as null model remain valid.
