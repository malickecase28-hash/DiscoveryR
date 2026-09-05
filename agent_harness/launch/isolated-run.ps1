[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $Command,
    [string] $RunRoot
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$approvedRoot = Join-Path $repo '.runs\wave1-native-v3'
$requestedRoot = if ($RunRoot) { [IO.Path]::GetFullPath($RunRoot).TrimEnd('\') } else { $approvedRoot }
if (-not $requestedRoot.Equals($approvedRoot.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) { throw "RunRoot must be the approved phase root: $approvedRoot" }

$assignmentPath = Join-Path $repo "agent_harness\assignments\$ProgramId\$Role.json"
if (-not (Test-Path -LiteralPath $assignmentPath -PathType Leaf)) { throw "Assignment not found: $ProgramId/$Role" }
$assignment = Get-Content -Raw $assignmentPath | ConvertFrom-Json
if ($assignment.program_id -ne $ProgramId -or $assignment.role_id -ne $Role) { throw 'Assignment identity mismatch.' }
if ($assignment.raw_lake_access -ne 'DENY' -or $assignment.peer_visibility -ne 'DENY') { throw 'Only DENY lake and peer assignments are launchable.' }
if ($assignment.access_profile -eq 'CONFIRMATION') { throw 'CONFIRMATION access is locked.' }

$workspace = Join-Path (Join-Path $approvedRoot $ProgramId) $Role
if (Test-Path -LiteralPath $workspace) {
    $entries = @(Get-ChildItem -LiteralPath $workspace -Force)
    if ($entries.Count -gt 0) { throw "NATIVE_WORKSPACE_NOT_FRESH: $workspace" }
}
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
$old = @{}
foreach ($name in @('TRINITYR_NATIVE_WORKSPACE_ONLY','TRINITYR_PROGRAM_ID','TRINITYR_ROLE_ID','TRINITYR_WORKSPACE')) { $old[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
try {
    [Environment]::SetEnvironmentVariable('TRINITYR_NATIVE_WORKSPACE_ONLY', '1', 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_PROGRAM_ID', $ProgramId, 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_ROLE_ID', $Role, 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_WORKSPACE', $workspace, 'Process')
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $Command
    exit $LASTEXITCODE
} finally {
    foreach ($name in $old.Keys) { [Environment]::SetEnvironmentVariable($name, $old[$name], 'Process') }
}
