[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $Command,
    [string] $RunRoot = 'F:\TrinityR-runs',
    [string] $Distro = 'Ubuntu',
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $RuntimeSlot
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
function SafeId([string] $Value, [string] $Name) {
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$') { throw "Unsafe ${Name}: $Value" }
}
function SafeRelative([string] $Value, [string] $Name) {
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value.StartsWith('/') -or $Value.StartsWith('\') -or $Value -match '(^|[\\/])\.\.([\\/]|$)' -or $Value -match ':') { throw "Unsafe ${Name}: $Value" }
}
function RecursiveHash([string] $Path) {
    $members = Get-ChildItem -LiteralPath $Path -Recurse -Force | ForEach-Object {
        if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse points are not allowed: $($_.FullName)" }
        if (-not $_.PSIsContainer) {
            $relative = $_.FullName.Substring($Path.Length + 1).Replace('\','/')
            "$relative`t$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())`n"
        }
    } | Sort-Object
    ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($members -join ''))) | ForEach-Object ToString x2) -join ''
}
$runRootFull = SafePath $RunRoot
New-Item -ItemType Directory -Force -Path $runRootFull | Out-Null
$assignmentPath = Join-Path $repoFull "agent_harness\assignments\$ProgramId\$Role.json"
if (-not (Test-Path -LiteralPath $assignmentPath -PathType Leaf)) { throw "Assignment not found: $ProgramId/$Role" }
$assignment = Get-Content -Raw -LiteralPath $assignmentPath | ConvertFrom-Json
if ($assignment.program_id -ne $ProgramId -or $assignment.role_id -ne $Role) { throw 'Assignment program_id/role_id mismatch.' }
if ([string]::IsNullOrWhiteSpace($assignment.access_profile)) { throw 'Assignment access_profile is required.' }
$sensitive = 'provider|model|runtime|credential|secret|token'
foreach ($property in $assignment.psobject.Properties) {
    if ($property.Name -match $sensitive -or $property.Name -in @('environment','env','runtime_config')) { throw "Provider/runtime selection is not allowed in scientific assignments: $($property.Name)" }
}
SafeId $assignment.program_id 'program_id'; SafeId $assignment.role_id 'role_id'
if ($assignment.phase -notin @('AUTHORITY','DEVELOPMENT','CONFIRMATION')) { throw "Invalid phase: $($assignment.phase)" }
if ($assignment.peer_visibility -ne 'DENY') { throw 'peer_visibility must be DENY.' }
if (-not $assignment.authorized_repository_surfaces) { throw 'Repository surfaces are required.' }
$profile = Get-Content -Raw -LiteralPath (Join-Path $repoFull 'agent_harness\access_profiles\default.json') | ConvertFrom-Json
if (-not $profile.profiles.($assignment.access_profile)) { throw "Undeclared access profile: $($assignment.access_profile)" }
if ($assignment.raw_lake_access -ne 'DENY') { throw 'Only raw_lake_access=DENY is launchable.' }
$lake = $null
if ($assignment.access_profile -eq 'CONFIRMATION') { throw 'CONFIRMATION access is locked until an explicit freeze action.' }
if ($assignment.access_profile -eq 'DEVELOPMENT') {
    SafeId $assignment.instrument_id 'instrument_id'; SafeId $assignment.data_scope_id 'data_scope_id'
    $scopeManifest = Join-Path $repoFull "instruments\$($assignment.instrument_id)\data_scope_v1.json"
    if (-not (Test-Path -LiteralPath $scopeManifest -PathType Leaf)) { throw "Committed data scope not found: $($assignment.instrument_id)/$($assignment.data_scope_id)" }
    $scope = Get-Content -Raw -LiteralPath $scopeManifest | ConvertFrom-Json
    if ($scope.scope_id -ne $assignment.data_scope_id -or $scope.instrument -ne $assignment.instrument_id) { throw 'Committed data scope identity mismatch.' }
    $viewsRoot = if ($env:TRINITYR_VIEWS_ROOT) { $env:TRINITYR_VIEWS_ROOT } else { 'F:\TrinityR-views' }
    if ($viewsRoot -notmatch '^[A-Za-z]:\\' -or $viewsRoot -match '[*?]') { throw 'Unsafe TRINITYR_VIEWS_ROOT.' }
    $lake = Join-Path $viewsRoot "$($assignment.instrument_id)\$($assignment.data_scope_id)\development"
    if (-not (Test-Path -LiteralPath $lake -PathType Container)) { throw "Required instrument-scoped development view not found: $lake" }
    NoReparse $lake
    $manifestPath = Join-Path $lake 'view_manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { throw 'Development view_manifest.json is required.' }
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    if ($manifest.instrument -ne $assignment.instrument_id -or $manifest.scope_id -ne $assignment.data_scope_id) { throw 'Development view manifest identity mismatch.' }
}
$programRoot = Join-Path $runRootFull $ProgramId
$workspace = Join-Path $programRoot $Role
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
NoReparse $runRootFull; NoReparse $programRoot; NoReparse $workspace
$mounts = @()
$runtime = $null
$runtimeEndpoint = ''
if ($RuntimeSlot) {
    if (-not $env:TRINITYR_RUNTIME_CONFIG) { throw 'TRINITYR_RUNTIME_CONFIG is required for RuntimeSlot.' }
    if (-not (Test-Path -LiteralPath $env:TRINITYR_RUNTIME_CONFIG -PathType Leaf)) { throw 'Runtime config is unavailable.' }
    $runtimeConfig = Get-Content -Raw -LiteralPath $env:TRINITYR_RUNTIME_CONFIG | ConvertFrom-Json
    $slot = @($runtimeConfig.slots | Where-Object slot_id -eq $RuntimeSlot)
    if ($slot.Count -ne 1 -or [string]::IsNullOrWhiteSpace($slot[0].endpoint)) { throw "Runtime slot is not configured: $RuntimeSlot" }
    # The endpoint is opaque to the assignment and is not mounted or serialized.
    $runtimeEndpoint = $slot[0].endpoint
}
foreach ($surface in @($assignment.authorized_repository_surfaces)) {
    if ([string]::IsNullOrWhiteSpace($surface) -or $surface.StartsWith('/') -or $surface.StartsWith('\') -or $surface -match '(^|[\\/])\.\.([\\/]|$)' -or $surface -match ':') { throw "Unsafe repository surface: $surface" }
    $source = [IO.Path]::GetFullPath((Join-Path $repoFull ($surface -replace '/','\')))
    if (-not $source.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase) -or -not (Test-Path -LiteralPath $source)) { throw "Unavailable or outside repository surface: $surface" }
    NoReparse $source
    if ($surface -match '(^|[\\/])\.\.([\\/]|$)') { throw "Unsafe repository surface: $surface" }
    $mounts += [PSCustomObject]@{ source = (WslPath $source); target = '/shared/research-program/' + ($surface -replace '\\','/') }
}
$registryId = if ($assignment.authority_registry_id) { $assignment.authority_registry_id } else { 'XAUUSD' }
SafeId $registryId 'authority_registry_id'
$registryFile = Join-Path $repoFull "agent_harness\authority_sources\$registryId.json"
if (-not (Test-Path -LiteralPath $registryFile -PathType Leaf)) {
    if ($registryId -eq 'XAUUSD' -and $assignment.access_profile -eq 'AUTHORITY') { $registryFile = Join-Path $repoFull 'agent_harness\authority_sources\XAUUSD.json' } else { throw "Authority registry not found: $registryId" }
}
$authority = Get-Content -Raw -LiteralPath $registryFile | ConvertFrom-Json
foreach ($sourceId in @($assignment.authority_source_ids)) {
    $source = @($authority.sources | Where-Object source_id -eq $sourceId)
    if ($source.Count -ne 1) { throw "Unknown authority source: $sourceId" }
    SafeRelative $source[0].relative_path 'authority relative_path'
    if ($source[0].root_env) {
        if ($source[0].root_env -ne 'TRINITYR_AUTHORITY_ROOT') { throw "Unallowlisted authority root env: $sourceId" }
        $root = [Environment]::GetEnvironmentVariable($source[0].root_env)
        if ([string]::IsNullOrWhiteSpace($root)) { throw "Missing authority root env: $sourceId" }
        $sourcePath = Join-Path $root ($source[0].relative_path -replace '/','\')
    } elseif ($source[0].root_path) {
        if ($source[0].root_path -ne 'F:\TrinityR-research') { throw "Unsafe authority root: $sourceId" }
        $sourcePath = Join-Path $source[0].root_path ($source[0].relative_path -replace '/','\')
    } else { throw "Authority source root missing: $sourceId" }
    if (-not (Test-Path -LiteralPath $sourcePath)) { throw "Unavailable authority source: $sourceId" }
    NoReparse $sourcePath
    if ((RecursiveHash $sourcePath) -ne $source[0].hash) { throw "Authority source hash mismatch: $sourceId" }
    if ($source[0].mount_target -notmatch '^/shared/[A-Za-z0-9._/-]+$' -or $source[0].mount_target -match '\.\.') { throw "Unsafe authority mount target: $sourceId" }
    $mounts += [PSCustomObject]@{ source = (WslPath $sourcePath); target = $source[0].mount_target }
}
if ($lake) { $mounts += [PSCustomObject]@{ source = (WslPath $lake); target = "/shared/data/$($assignment.data_scope_id)/development" } }
if ($runtime) { $mounts += $runtime }
$mountText = ($mounts | ForEach-Object { "$(B64 $_.source)|$(B64 $_.target)" }) -join "`n"
$launcher = Join-Path $PSScriptRoot 'isolated-run.sh'
$bootstrap = B64 ((Get-Content -Raw -LiteralPath $launcher).Replace("`r", ''))
$safeCommand = $Command.Replace("`r", '')
$commandLine = "printf %s $bootstrap | base64 -d > /tmp/trinityr-isolated-run.sh && chmod 700 /tmp/trinityr-isolated-run.sh && unshare --mount --pid --fork --mount-proc --propagation private -- /bin/bash /tmp/trinityr-isolated-run.sh $(B64 (WslPath $workspace)) $(B64 $assignment.access_profile) $(B64 $safeCommand) $(B64 (B64 $mountText)) $(B64 $runtimeEndpoint)"
& wsl.exe --distribution $Distro --user root -- /bin/bash -lc $commandLine
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
