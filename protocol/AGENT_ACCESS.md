# Agent access

- Repository mounts are the assignment's exact read-only surfaces.
- The private workspace is the only writable location.
- Raw lake, future findings, peer workspaces, legacy material, credentials, and provider maps are unavailable.
- Authority sources are mounted only when their source IDs are authorized by the assignment.
- Confirmation is fail-closed until an explicit freeze action.

## Submission lifecycle

- The agent writes only `authority_candidate.json` and `authority_review.md` to its private workspace.
- The agent does not initialize a repository, create commits, access Git credentials, or push remotely.
- On completion, the agent stops and leaves the two files for host-side validation.
- The orchestrator validates identity, schema, forbidden operational identity, and exact file count before publication.
- Publication is a separate host-side action on a dedicated review branch; it is never implicit in agent completion.
