[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $Command
)

$ErrorActionPreference = 'Stop'

throw @"
CODEX_HARD_RUNTIME_BLOCKED
reason=OTHER_PRECISE_REASON:NATIVE_WINDOWS_HARD_BOUNDARY_UNAVAILABLE
program_id=$ProgramId
role_id=$Role
No process was launched. Native Windows Codex execution is host-managed and
cannot enforce the required filesystem, process, peer, lake, confirmation,
credential, or host-canary boundary. WSL and ZCode are retired; no credentials,
runtime socket, or replacement root may be created by this launcher.
"@
