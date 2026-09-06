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

function Assert-NoReparseAncestor([string] $Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while ($null -ne $current -and $current.Length -gt 0) {
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "NATIVE_WORKSPACE_REPARSE_POINT: $current"
            }
        }
        $parent = [IO.Directory]::GetParent($current)
        if ($null -eq $parent) { break }
        $current = $parent.FullName
    }
}

Assert-NoReparseAncestor $approvedRoot

$assignmentPath = Join-Path $repo "agent_harness\assignments\$ProgramId\$Role.json"
if (-not (Test-Path -LiteralPath $assignmentPath -PathType Leaf)) { throw "Assignment not found: $ProgramId/$Role" }
$assignment = Get-Content -Raw $assignmentPath | ConvertFrom-Json
if ($assignment.program_id -ne $ProgramId -or $assignment.role_id -ne $Role) { throw 'Assignment identity mismatch.' }
if ($assignment.raw_lake_access -ne 'DENY' -or $assignment.peer_visibility -ne 'DENY') { throw 'Only DENY lake and peer assignments are launchable.' }
if ($assignment.access_profile -eq 'CONFIRMATION') { throw 'CONFIRMATION access is locked.' }

$workspace = Join-Path (Join-Path $approvedRoot $ProgramId) $Role
Assert-NoReparseAncestor $workspace
if (Test-Path -LiteralPath $workspace) {
    $entries = @(Get-ChildItem -LiteralPath $workspace -Force)
    if ($entries.Count -gt 0) { throw "NATIVE_WORKSPACE_NOT_FRESH: $workspace" }
}
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
Assert-NoReparseAncestor $workspace
$old = @{}
foreach ($name in @('TRINITYR_NATIVE_WORKSPACE_ONLY','TRINITYR_PROGRAM_ID','TRINITYR_ROLE_ID','TRINITYR_WORKSPACE')) { $old[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
try {
    [Environment]::SetEnvironmentVariable('TRINITYR_NATIVE_WORKSPACE_ONLY', '1', 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_PROGRAM_ID', $ProgramId, 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_ROLE_ID', $Role, 'Process')
    [Environment]::SetEnvironmentVariable('TRINITYR_WORKSPACE', $workspace, 'Process')
    $exitCode = 1
    Push-Location -LiteralPath $workspace
    try {
        & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command $Command
        $exitCode = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    exit $exitCode
} finally {
    foreach ($name in $old.Keys) { [Environment]::SetEnvironmentVariable($name, $old[$name], 'Process') }
}
