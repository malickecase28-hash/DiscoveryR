[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Z]+-\d{2}$')]
    [string] $Role,
    [string] $RunRoot = 'F:\TrinityR-runs',
    [Parameter(Mandatory)]
    [string] $Command,
    [ValidateSet('AUTHORITY', 'DEVELOPMENT', 'CONFIRMATION')]
    [string] $AccessProfile = 'AUTHORITY',
    [string] $Distro = 'Ubuntu'
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$repoFull = [IO.Path]::GetFullPath($repo).TrimEnd('\') + '\'
$runItem = Get-Item -LiteralPath (New-Item -ItemType Directory -Force -Path $RunRoot).FullName
if ($runItem.Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'RunRoot must not be a junction or reparse point.'
}
$runFull = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $runItem.FullName).Path).TrimEnd('\') + '\'
if ($runFull.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'RunRoot must be outside the shared Git repository.'
}
$programRunRoot = Join-Path $runItem.FullName 'AP-001'
$workspace = Join-Path $programRunRoot $Role
$programRunItem = Get-Item -LiteralPath (New-Item -ItemType Directory -Force -Path $programRunRoot).FullName
if ($programRunItem.Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'AP-001 run root must not be a junction or reparse point.'
}
$workspaceItem = Get-Item -LiteralPath (New-Item -ItemType Directory -Force -Path $workspace).FullName
if ($workspaceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'Role workspace must not be a junction or reparse point.'
}
$lake = ''
if ($AccessProfile -eq 'CONFIRMATION') {
    throw 'CONFIRMATION access is locked until an explicit freeze action.'
}
if ($AccessProfile -eq 'DEVELOPMENT') {
    $lake = Join-Path $RunRoot 'AP-001\development-lake'
}
if ($AccessProfile -eq 'DEVELOPMENT' -and -not (Test-Path -LiteralPath $lake -PathType Container)) {
    throw ('Required lake view not found for {0}: {1}' -f $AccessProfile, $lake)
}
$launcher = Join-Path $PSScriptRoot 'isolated-run.sh'
function WslPath([string] $Path) {
    if ($Path -notmatch '^([A-Za-z]):\\(.*)$') {
        throw "Only absolute Windows drive paths are supported: $Path"
    }
    return "/mnt/$($Matches[1].ToLowerInvariant())/$($Matches[2] -replace '\\','/')"
}
function Base64([string] $Value) {
    return [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Value))
}
$bootstrap = Base64 (Get-Content -Raw -LiteralPath $launcher)
$commandBase64 = Base64 $Command
$lakeWsl = if ($lake) { WslPath $lake } else { '' }
$lakeBase64 = if ($lakeWsl) { Base64 $lakeWsl } else { '-' }
$commandLine = "printf %s $bootstrap | base64 -d > /tmp/trinityr-isolated-run.sh && chmod 700 /tmp/trinityr-isolated-run.sh && unshare --mount --fork --propagation private -- /bin/bash /tmp/trinityr-isolated-run.sh $(Base64 (WslPath $repo)) $lakeBase64 $(Base64 (WslPath $workspace)) $commandBase64"
$args = @(
    '--distribution', $Distro,
    '--user', 'root',
    '--',
    '/bin/bash',
    '-lc',
    $commandLine
)
& wsl.exe @args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
