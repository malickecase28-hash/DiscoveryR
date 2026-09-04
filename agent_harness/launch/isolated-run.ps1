[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Z]+-\d{2}$')]
    [string] $Role,
    [string] $RunRoot = 'F:\TrinityR-runs',
    [Parameter(Mandatory)]
    [string] $Command,
    [string] $Distro = 'Ubuntu'
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$repoFull = [IO.Path]::GetFullPath($repo).TrimEnd('\') + '\'
$runFull = [IO.Path]::GetFullPath((New-Item -ItemType Directory -Force -Path $RunRoot).FullName).TrimEnd('\') + '\'
if ($runFull.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'RunRoot must be outside the shared Git repository.'
}
$workspace = Join-Path $RunRoot (Join-Path 'AP-001' $Role)
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
$lake = Join-Path (Split-Path $repo -Parent) 'analytical_lake\fusion_markets\xauusd'
if (-not (Test-Path -LiteralPath $lake -PathType Container)) {
    throw "Frozen XAUUSD lake not found: $lake"
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
$commandLine = "printf %s $bootstrap | base64 -d > /tmp/trinityr-isolated-run.sh && chmod 700 /tmp/trinityr-isolated-run.sh && unshare --mount --fork --propagation private -- /bin/bash /tmp/trinityr-isolated-run.sh $(Base64 (WslPath $repo)) $(Base64 (WslPath $lake)) $(Base64 (WslPath $workspace)) $commandBase64"
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
