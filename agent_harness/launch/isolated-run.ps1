[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $Command,
    [string] $RunRoot = 'F:\TrinityR-runs',
    [string] $Distro = 'Ubuntu'
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$repoFull = [IO.Path]::GetFullPath($repo).TrimEnd('\') + '\'
function SafePath([string] $Path) {
    $full = [IO.Path]::GetFullPath($Path)
    if ($full.Equals($repo.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase)) { throw 'RunRoot must be outside the shared Git repository.' }
    return $full
}
function B64([string] $Value) { [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Value)) }
function WslPath([string] $Path) {
    if ($Path -notmatch '^([A-Za-z]):\\(.*)$') { throw "Only absolute Windows drive paths are supported: $Path" }
    "/mnt/$($Matches[1].ToLowerInvariant())/$($Matches[2] -replace '\\','/')"
}
function NoReparse([string] $Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while ($current -and $current -ne [IO.Path]::GetPathRoot($current)) {
        $item = Get-Item -LiteralPath $current -ErrorAction Stop
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse points are not allowed: $current" }
        $current = Split-Path -Parent $current
    }
}
$runRootFull = SafePath $RunRoot
New-Item -ItemType Directory -Force -Path $runRootFull | Out-Null
$assignmentPath = Join-Path $repoFull "agent_harness\assignments\$ProgramId\$Role.json"
if (-not (Test-Path -LiteralPath $assignmentPath -PathType Leaf)) { throw "Assignment not found: $ProgramId/$Role" }
$assignment = Get-Content -Raw -LiteralPath $assignmentPath | ConvertFrom-Json
if ($assignment.program_id -ne $ProgramId -or $assignment.role_id -ne $Role) { throw 'Assignment program_id/role_id mismatch.' }
if ([string]::IsNullOrWhiteSpace($assignment.access_profile)) { throw 'Assignment access_profile is required.' }
$profile = Get-Content -Raw -LiteralPath (Join-Path $repoFull 'agent_harness\access_profiles\default.json') | ConvertFrom-Json
if (-not $profile.profiles.($assignment.access_profile)) { throw "Undeclared access profile: $($assignment.access_profile)" }
if ($assignment.raw_lake_access -ne 'DENY') { throw 'Only raw_lake_access=DENY is launchable.' }
$lake = $null
if ($assignment.access_profile -eq 'CONFIRMATION') { throw 'CONFIRMATION access is locked until an explicit freeze action.' }
if ($assignment.access_profile -eq 'DEVELOPMENT') {
    $lake = 'F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development'
    if (-not (Test-Path -LiteralPath $lake -PathType Container)) { throw "Required instrument-scoped development view not found: $lake" }
    NoReparse $lake
}
$programRoot = Join-Path $runRootFull $ProgramId
$workspace = Join-Path $programRoot $Role
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
NoReparse $runRootFull; NoReparse $programRoot; NoReparse $workspace
$mounts = @()
$runtime = $null
if ($assignment.agent_runtime) {
    if ($assignment.agent_runtime -ne 'ZCODE') { throw "Undeclared agent runtime: $($assignment.agent_runtime)" }
    $runtime = [PSCustomObject]@{ source = '/home/malo/.zcode/server'; target = '/opt/zcode' }
}
foreach ($surface in @($assignment.authorized_repository_surfaces)) {
    if ([string]::IsNullOrWhiteSpace($surface) -or $surface.StartsWith('/') -or $surface.StartsWith('\') -or $surface -match '(^|[\\/])\.\.([\\/]|$)' -or $surface -match ':') { throw "Unsafe repository surface: $surface" }
    $source = [IO.Path]::GetFullPath((Join-Path $repoFull ($surface -replace '/','\')))
    if (-not $source.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase) -or -not (Test-Path -LiteralPath $source)) { throw "Unavailable or outside repository surface: $surface" }
    NoReparse $source
    $mounts += [PSCustomObject]@{ source = (WslPath $source); target = '/shared/research-program/' + ($surface -replace '\\','/') }
}
$authority = Get-Content -Raw -LiteralPath (Join-Path $repoFull 'agent_harness\authority_sources\XAUUSD.json') | ConvertFrom-Json
foreach ($sourceId in @($assignment.authority_source_ids)) {
    $source = @($authority.sources | Where-Object source_id -eq $sourceId)
    if ($source.Count -ne 1) { throw "Unknown authority source: $sourceId" }
    if ($source[0].root_path -ne 'F:\TrinityR-research') { throw "Unsafe authority root: $sourceId" }
    $sourcePath = Join-Path $source[0].root_path ($source[0].relative_path -replace '/','\')
    if (-not (Test-Path -LiteralPath $sourcePath)) { throw "Unavailable authority source: $sourceId" }
    NoReparse $sourcePath
    $members = Get-ChildItem -LiteralPath $sourcePath -Recurse -Force | ForEach-Object {
        if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Authority source contains reparse point: $($_.FullName)" }
        if ($_.PSIsContainer) { return }
        $relative = $_.FullName.Substring($sourcePath.Length + 1).Replace('\','/')
        $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$relative`t$hash`n"
    } | Sort-Object
    $digest = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($members -join ''))) | ForEach-Object ToString x2) -join ''
    if ($digest -ne $source[0].hash) { throw "Authority source hash mismatch: $sourceId" }
    $mounts += [PSCustomObject]@{ source = (WslPath $sourcePath); target = $source[0].mount_target }
}
if ($lake) { $mounts += [PSCustomObject]@{ source = (WslPath $lake); target = '/shared/data/XAUUSD_DATA_SCOPE_V1/development' } }
if ($runtime) { $mounts += $runtime }
$mountText = ($mounts | ForEach-Object { "$(B64 $_.source)|$(B64 $_.target)" }) -join "`n"
$launcher = Join-Path $PSScriptRoot 'isolated-run.sh'
$bootstrap = B64 (Get-Content -Raw -LiteralPath $launcher)
$commandLine = "printf %s $bootstrap | base64 -d > /tmp/trinityr-isolated-run.sh && chmod 700 /tmp/trinityr-isolated-run.sh && unshare --mount --pid --fork --mount-proc --propagation private -- /bin/bash /tmp/trinityr-isolated-run.sh $(B64 (WslPath $workspace)) $(B64 $assignment.access_profile) $(B64 $Command) $(B64 (B64 $mountText))"
& wsl.exe --distribution $Distro --user root -- /bin/bash -lc $commandLine
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
