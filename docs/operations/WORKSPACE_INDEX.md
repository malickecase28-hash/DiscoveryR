# Workspace index

This is the active direct-Windows layout on `F:`. Do not create new top-level
TrinityR roots.

## Active roots

- `F:\TrinityR-research\Research Program` — canonical Git repository and
  implementation workspace.
- `F:\TrinityR-research\i01-development-view` — retained linked development
  view for its active branch.
- `F:\TrinityR-research\analytical_lake` — frozen data input.
- `F:\TrinityR-research\stage2-artifacts` — retained project artifacts.
- `F:\TrinityR` — existing data/history root; do not repurpose or sweep.

Supporting configuration and schema directories remain in place under
`F:\TrinityR-research` because they are inputs to the active repository.

## Temporary work

Create temporary material only under a declared phase/run directory inside the
canonical repository or an explicitly registered project path. On signoff,
delete that temporary directory. Preserve only the required published output,
manifests/receipts, and reusable scripts in their permanent location.

The approved native-Windows Wave-1 rerun is:

- `F:\TrinityR-research\Research Program\.runs\wave1-native-v3\<PROGRAM>\<ROLE>`

The prior `wave1-native-v2` directory is retained as earlier-run material and is
not reused for fresh researchers.

Each role directory is private by workflow convention only. Windows folder
separation does not provide hard peer, credential, network, or host-data
isolation. Researchers must write only `authority_candidate.json` and
`authority_review.md` there; host validation and publication happen afterward.

If signoff has not occurred, leave the directory intact and mark it pending.
Never create disposable roots directly under `F:\`.
