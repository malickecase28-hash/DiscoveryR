[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $Command,
    [string] $RunRoot = 'F:\TrinityR-runs',
    [string] $Distro = 'Ubuntu',
    [ValidatePattern('^[0-9a-fA-F]{40}$')][string] $ResearchBaseSha,
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $RuntimeSlot
)
$ErrorActionPreference = 'Stop'
$repoSource = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$baseSha = if ($ResearchBaseSha) { $ResearchBaseSha.ToLowerInvariant() } elseif ($env:TRINITYR_RESEARCH_BASE_SHA) { $env:TRINITYR_RESEARCH_BASE_SHA.ToLowerInvariant() } else { throw 'ResearchBaseSha is required; launch from the frozen research base.' }
if ($baseSha -notmatch '^[0-9a-f]{40}$') { throw 'Invalid immutable research base SHA.' }
$launchWorktreeRoot = if ($env:TRINITYR_LAUNCH_WORKTREE_ROOT) { $env:TRINITYR_LAUNCH_WORKTREE_ROOT } else { Join-Path $RunRoot '.launch-worktrees' }
if ($launchWorktreeRoot -notmatch '^[A-Za-z]:\\' -or $launchWorktreeRoot -match '[*?]') { throw 'Unsafe launch worktree root.' }
New-Item -ItemType Directory -Force -Path $launchWorktreeRoot | Out-Null
$baseTree = Join-Path $launchWorktreeRoot ('trinityr-launch-' + [guid]::NewGuid().ToString('N'))
$baseTreeAdded = $false
try {
& git -C $repoSource worktree add --detach $baseTree $baseSha | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Unable to materialize frozen research base.' }
$baseTreeAdded = $true
$repo = (Resolve-Path $baseTree).Path
$repoFull = [IO.Path]::GetFullPath($repo).TrimEnd('\') + '\'
function SafePath([string] $Path) {
    $full = [IO.Path]::GetFullPath($Path)
    if ($full.Equals($repo.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase)) { throw 'RunRoot must be outside the shared Git repository.' }
    return $full
}
function ManifestRoot([string] $Path) {
    if ($Path -notmatch '^[A-Za-z]:\\' -or $Path -match '[*?]') { throw 'Unsafe TRINITYR_MANIFEST_ROOT.' }
    $full=[IO.Path]::GetFullPath($Path).TrimEnd('\'); if ($full.Equals($repo.TrimEnd('\'),[StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($repoFull,[StringComparison]::OrdinalIgnoreCase)) { throw 'Manifest root must be outside the shared Git repository.' }; return $full
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
        if (-not $_.PSIsContainer -and $_.Name -ne 'bundle_manifest.json') {
            $relative = $_.FullName.Substring($Path.Length + 1).Replace('\','/')
            "$relative`t$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())`n"
        }
    } | Sort-Object
    ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($members -join ''))) | ForEach-Object ToString x2) -join ''
}
function ShaText([string] $Value) { ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes($Value)) | ForEach-Object ToString x2) -join '' }
function SharedInputFingerprint($Assignment, $Authority, $RepositorySourceHashes) {
    $shared = [ordered]@{
        program_id = $Assignment.program_id; detector_id = $Assignment.detector_id; assignment = $Assignment.assignment
        authorized_repository_surfaces = @($Assignment.authorized_repository_surfaces | Where-Object { $_ -notmatch '^agent_harness/assignments/' } | Sort-Object)
        authority_source_ids = @($Assignment.authority_source_ids | Sort-Object)
        authority_bundle_identities = @($Assignment.authority_source_ids | ForEach-Object { $s=(@($Authority.sources | Where-Object source_id -eq $_))[0]; "$($s.source_id):$($s.relative_path):$($s.bundle_content_identity):$($s.directory_transport_hash)" } | Sort-Object)
        repository_source_hashes = $RepositorySourceHashes
        raw_lake_access = $Assignment.raw_lake_access; peer_visibility = $Assignment.peer_visibility
    }
    ShaText (($shared | ConvertTo-Json -Compress -Depth 10))
}
function WriteInputManifest($Assignment, $Authority, [string] $ManifestRoot, [string] $BaseSha) {
    $surfaces=@($Assignment.authorized_repository_surfaces|Sort-Object|ForEach-Object{$p=Join-Path $repoFull ($_ -replace '/','\');$item=Get-Item $p;[ordered]@{relative_path=$_;identity=if($item.PSIsContainer){RecursiveHash $p}else{(Get-FileHash $p -Algorithm SHA256).Hash.ToLowerInvariant()};hash=if($item.PSIsContainer){RecursiveHash $p}else{(Get-FileHash $p -Algorithm SHA256).Hash.ToLowerInvariant()}}})
    $sourceMap=@{'protocol'='repo.protocol';'contracts'='repo.contracts';'registry'='repo.detector_registry';'instruments/XAUUSD/instrument_config.json'='repo.xauusd.instrument_config';'instruments/XAUUSD/source_inventory.json'='repo.xauusd.source_inventory'}
    $repositorySourceHashes=[ordered]@{}
    foreach($surface in $surfaces){$id=$sourceMap[$surface.relative_path];if($id){$repositorySourceHashes[$id]=$surface.hash}}
    $authoritySources=@($Assignment.authority_source_ids | ForEach-Object { $s=@($Authority.sources | Where-Object source_id -eq $_)[0]; [ordered]@{source_id=$s.source_id;relative_path=$s.relative_path;mount_target=$s.mount_target;hash=$s.hash;bundle_content_identity=$s.bundle_content_identity;directory_transport_hash=$s.directory_transport_hash} })
    $pair = SharedInputFingerprint $Assignment $Authority $repositorySourceHashes
    New-Item -ItemType Directory -Force -Path $ManifestRoot | Out-Null
    $manifest = [ordered]@{ schema_version='trinity.research-input-manifest.v3'; program_id=$Assignment.program_id; role_id=$Assignment.role_id; phase=$Assignment.phase; research_base_sha=$BaseSha; assignment_path=("agent_harness/assignments/$($Assignment.program_id)/$($Assignment.role_id).json"); assignment_sha256=(Get-FileHash $assignmentPath -Algorithm SHA256).Hash.ToLowerInvariant(); access_profile=$Assignment.access_profile; raw_lake_access=$Assignment.raw_lake_access; repository_surfaces=$surfaces; repository_source_hashes=$repositorySourceHashes; authority_sources=$authoritySources; shared_input_fingerprint=$pair }
    $path=Join-Path $ManifestRoot 'research_input_manifest.json'; if (Test-Path -LiteralPath $path) { $old=Get-Content -Raw $path; $new=($manifest|ConvertTo-Json -Depth 20)+[Environment]::NewLine; if ($old -ne $new) { throw 'Immutable research input manifest conflict.' } } else { [IO.File]::WriteAllText($path,(($manifest|ConvertTo-Json -Depth 20)+[Environment]::NewLine),[Text.UTF8Encoding]::new($false)) }
    return $path
}
function MountPath([string] $Path) { if ($Path.StartsWith('/')) { $Path } else { WslPath $Path } }
function RejectOperationalKeys($Node, [string] $Path = 'assignment') {
    if ($Node -is [PSCustomObject]) {
        foreach ($property in $Node.psobject.Properties) {
            if ($property.Name -match '^(provider|provider_id|provider_identity|model|model_id|model_identity|runtime|runtime_slot|runtime_config|credential|credential_path|secret|token)$') { throw "Operational identity is not allowed: $Path.$($property.Name)" }
            RejectOperationalKeys $property.Value "$Path.$($property.Name)"
        }
    } elseif ($Node -is [System.Collections.IEnumerable] -and $Node -isnot [string]) {
        foreach ($item in $Node) { RejectOperationalKeys $item $Path }
    }
}
function ValidateAssignmentShape($Assignment) {
    RejectOperationalKeys $Assignment
    $common = @('program_id','role_id','phase','access_profile','assignment','authorized_repository_surfaces','authority_source_ids','raw_lake_access','peer_visibility')
    $phaseFields = @{
        AUTHORITY = @('private_workspace','detector_id','instrument_id','authority_registry_id')
        DEVELOPMENT = @('instrument_id','data_scope_id','authority_registry_id')
        CONFIRMATION = @('instrument_id','authority_registry_id')
    }
    $allowed = @($common + $phaseFields[$Assignment.phase])
    $unknown = @($Assignment.psobject.Properties.Name | Where-Object { $_ -notin $allowed })
    if ($unknown.Count) { throw "Unknown assignment fields: $($unknown -join ', ')" }
    if ($Assignment.phase -eq 'DEVELOPMENT') {
        if (-not $Assignment.instrument_id -or -not $Assignment.data_scope_id) { throw 'DEVELOPMENT requires instrument_id and data_scope_id.' }
    } elseif ($Assignment.phase -eq 'AUTHORITY' -and $ProgramId -notin @('AP-001','AP-002','TC-001','BG-001')) {
        if (-not $Assignment.instrument_id -or -not $Assignment.authority_registry_id) { throw 'New AUTHORITY assignments require instrument_id and authority_registry_id.' }
    }
}
function RuntimeMount($Slot) {
    if (-not $env:TRINITYR_RUNTIME_CONFIG) { throw 'TRINITYR_RUNTIME_CONFIG is required for RuntimeSlot.' }
    if (-not (Test-Path -LiteralPath $env:TRINITYR_RUNTIME_CONFIG -PathType Leaf)) { throw 'Runtime config is unavailable.' }
    $runtimeConfig = Get-Content -Raw -LiteralPath $env:TRINITYR_RUNTIME_CONFIG | ConvertFrom-Json
    $unknownConfig = @($runtimeConfig.psobject.Properties.Name | Where-Object { $_ -notin @('schema_version','slots') })
    if ($runtimeConfig.schema_version -ne 1 -or $unknownConfig.Count) { throw 'Invalid runtime config schema.' }
    if (-not $runtimeConfig.slots -or $runtimeConfig.slots -is [string]) { throw 'Runtime config has no slots.' }
    $entry = @($runtimeConfig.slots | Where-Object slot_id -eq $Slot)
    if ($entry.Count -ne 1) { throw "Runtime slot is not configured: $Slot" }
    SafeId $entry[0].slot_id 'runtime slot_id'
    $fields = @('slot_id','broker_socket')
    $unknown = @($entry[0].psobject.Properties.Name | Where-Object { $_ -notin $fields })
    if ($unknown.Count) { throw 'Unknown runtime slot fields.' }
    $socket = [string]$entry[0].broker_socket
    if ($socket -notmatch '^/(run|tmp)/[A-Za-z0-9._/-]+$' -or $socket -match '\.\.' -or $socket -match '(?i)(provider|model|runtime|token|secret|credential|@|\?)') { throw 'Runtime broker socket is unsafe.' }
    $previousErrorAction = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & wsl.exe --distribution $Distro --user root -- test -S $socket 2>$null
        $socketExit = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previousErrorAction }
    if ($socketExit -ne 0) { throw 'Runtime broker socket is unavailable.' }
    [PSCustomObject]@{ source = $socket; target = '/run/trinityr/runtime.sock' }
}
$runRootFull = SafePath $RunRoot
New-Item -ItemType Directory -Force -Path $runRootFull | Out-Null
$assignmentPath = Join-Path $repoFull "agent_harness\assignments\$ProgramId\$Role.json"
if (-not (Test-Path -LiteralPath $assignmentPath -PathType Leaf)) { throw "Assignment not found: $ProgramId/$Role" }
$assignment = Get-Content -Raw -LiteralPath $assignmentPath | ConvertFrom-Json
if ($assignment.program_id -ne $ProgramId -or $assignment.role_id -ne $Role) { throw 'Assignment program_id/role_id mismatch.' }
if ([string]::IsNullOrWhiteSpace($assignment.access_profile)) { throw 'Assignment access_profile is required.' }
ValidateAssignmentShape $assignment
    SafeId $assignment.program_id 'program_id'; SafeId $assignment.role_id 'role_id'
    if ($assignment.phase -notin @('AUTHORITY','DEVELOPMENT','CONFIRMATION')) { throw "Invalid phase: $($assignment.phase)" }
    if ($assignment.phase -ne $assignment.access_profile) { throw 'phase and access_profile must match.' }
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
    foreach ($field in @('schema_version','instrument','scope_id','logical_view_identity','builder_code_identity')) {
        if ([string]::IsNullOrWhiteSpace([string]$manifest.$field)) { throw "Development manifest field missing: $field" }
    }
    if ($manifest.instrument -ne $assignment.instrument_id -or $manifest.scope_id -ne $assignment.data_scope_id) { throw 'Development view manifest identity mismatch.' }
}
$programRoot = Join-Path $runRootFull $ProgramId
$workspace = Join-Path $programRoot $Role
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
NoReparse $runRootFull; NoReparse $programRoot; NoReparse $workspace
$mounts = @()
$runtime = $null
if ($RuntimeSlot) {
    $runtime = RuntimeMount $RuntimeSlot
}
foreach ($surface in @($assignment.authorized_repository_surfaces)) {
    if ([string]::IsNullOrWhiteSpace($surface) -or $surface.StartsWith('/') -or $surface.StartsWith('\') -or $surface -match '(^|[\\/])\.\.([\\/]|$)' -or $surface -match ':') { throw "Unsafe repository surface: $surface" }
    $source = [IO.Path]::GetFullPath((Join-Path $repoFull ($surface -replace '/','\')))
    if (-not $source.StartsWith($repoFull, [StringComparison]::OrdinalIgnoreCase) -or -not (Test-Path -LiteralPath $source)) { throw "Unavailable or outside repository surface: $surface" }
    if ((Get-Item -LiteralPath $source).PSIsContainer -and @(Get-ChildItem -LiteralPath $source -Force).Count -eq 0) { throw "Empty repository surface: $surface" }
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
    SafeId $sourceId 'authority_source_id'
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
    if ([string]::IsNullOrWhiteSpace([string]$source[0].hash) -or $source[0].hash -notmatch '^[0-9a-fA-F]{64}$') { throw "Authority source hash is empty or invalid: $sourceId" }
    if ((RecursiveHash $sourcePath) -ne $source[0].hash.ToLowerInvariant()) { throw "Authority source hash mismatch: $sourceId" }
    $bundleManifestPath=Join-Path $sourcePath 'bundle_manifest.json'; if(!(Test-Path $bundleManifestPath -PathType Leaf)){throw "Authority bundle manifest missing: $sourceId"};$bundleManifest=Get-Content -Raw $bundleManifestPath|ConvertFrom-Json
    foreach($field in 'bundle_content_identity','directory_transport_hash'){if([string]::IsNullOrWhiteSpace([string]$source[0].$field)-or $source[0].$field -notmatch '^[0-9a-fA-F]{64}$'-or $bundleManifest.$field -ne $source[0].$field){throw "Authority bundle identity mismatch: $sourceId/$field"}}
    if ($source[0].mount_target -notmatch '^/shared/[A-Za-z0-9._/-]+$' -or $source[0].mount_target -match '\.\.') { throw "Unsafe authority mount target: $sourceId" }
    $mounts += [PSCustomObject]@{ source = (MountPath $sourcePath); target = $source[0].mount_target }
}
if ($lake) { $mounts += [PSCustomObject]@{ source = (MountPath $lake); target = "/shared/data/$($assignment.data_scope_id)/development" } }
if ($runtime) { $mounts += $runtime }
$actualHead = (& git -C $repo rev-parse HEAD).Trim().ToLowerInvariant()
if ($actualHead -ne $baseSha) { throw 'Repository HEAD does not match ResearchBaseSha.' }
if (@(& git -C $repo status --porcelain --untracked-files=all).Count -ne 0) { throw 'Research base worktree must be clean.' }
$manifestRootInput = if ($env:TRINITYR_MANIFEST_ROOT) { $env:TRINITYR_MANIFEST_ROOT } else { 'F:\TrinityR-manifests\wave1-v4' }
$manifestRoot = ManifestRoot $manifestRootInput
$manifestPath = WriteInputManifest $assignment $authority (Join-Path $manifestRoot "$ProgramId\$Role") $baseSha
$mounts += [PSCustomObject]@{ source = (MountPath $manifestPath); target = '/shared/research_input_manifest.json' }
$plannedManifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
$sourceMap = @{'protocol'='repo.protocol';'contracts'='repo.contracts';'registry'='repo.detector_registry';'instruments/XAUUSD/instrument_config.json'='repo.xauusd.instrument_config';'instruments/XAUUSD/source_inventory.json'='repo.xauusd.source_inventory'}
$runtimeAttestationDirectory = Join-Path (Split-Path $manifestPath -Parent) ('runtime-attestation-' + [guid]::NewGuid().ToString('N'))
$runtimeAttestationPath = Join-Path $runtimeAttestationDirectory 'runtime_mount_attestation.json'
New-Item -ItemType Directory -Path $runtimeAttestationDirectory | Out-Null
$repositorySpecs = @($plannedManifest.repository_surfaces | ForEach-Object { $id = $sourceMap[$_.relative_path]; if ($id) { "REPO`t$id`t/shared/research-program/$($_.relative_path -replace '\\','/')`t$($_.hash)" } })
$authoritySpecs = @($plannedManifest.authority_sources | ForEach-Object { "AUTH`t$($_.source_id)`t$($_.mount_target)`t$($_.bundle_content_identity)`t$($_.directory_transport_hash)" })
$attestationSpecs = ($repositorySpecs + $authoritySpecs) -join "`n"
$manifestHash = (Get-FileHash $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
$launchId = [guid]::NewGuid().ToString('N')
$mountText = ($mounts | ForEach-Object { "$(B64 $_.source)|$(B64 $_.target)" }) -join "`n"
$launcher = Join-Path $repo 'agent_harness\launch\isolated-run.sh'
$bootstrap = B64 ((Get-Content -Raw -LiteralPath $launcher).Replace("`r", ''))
$safeCommand = $Command.Replace("`r", '')
$commandLine = "mountpoint -q /mnt/f || mount -t drvfs F: /mnt/f; printf %s $bootstrap | base64 -d > /tmp/trinityr-isolated-run.sh && chmod 700 /tmp/trinityr-isolated-run.sh && unshare --mount --pid --fork --mount-proc --propagation private -- /bin/bash /tmp/trinityr-isolated-run.sh $(B64 (WslPath $workspace)) $(B64 $assignment.access_profile) $(B64 $safeCommand) $(B64 (B64 $mountText)) $(B64 (B64 $attestationSpecs)) $(B64 (WslPath $runtimeAttestationDirectory)) $(B64 $launchId) $(B64 $manifestHash) $(B64 $ProgramId) $(B64 $Role) $(B64 $baseSha)"
& wsl.exe --distribution $Distro --user root -- /bin/bash -lc $commandLine
$workerExit = $LASTEXITCODE
if (-not (Test-Path -LiteralPath $runtimeAttestationPath -PathType Leaf)) { throw 'Runtime attestation missing.' }
$runtimeAttestation = Get-Content -Raw -LiteralPath $runtimeAttestationPath | ConvertFrom-Json
if ($runtimeAttestation.schema_version -ne 'trinity.runtime-mount-attestation.v1' -or $runtimeAttestation.launch_id -ne $launchId -or $runtimeAttestation.program_id -ne $ProgramId -or $runtimeAttestation.role_id -ne $Role -or $runtimeAttestation.research_base_sha -ne $baseSha -or $runtimeAttestation.research_input_manifest_sha256 -ne $manifestHash -or $runtimeAttestation.all_match -ne $true) { throw 'RUNTIME_ATTESTATION_FAIL' }
Write-Output "RUNTIME_ATTESTATION_PASS launch_id=$launchId sha256=$((Get-FileHash $runtimeAttestationPath -Algorithm SHA256).Hash.ToLowerInvariant())"
if ($workerExit -ne 0) { exit $workerExit }
} finally {
    if ($baseTreeAdded) { & git -C $repoSource worktree remove --force $baseTree 2>$null }
    if (Test-Path -LiteralPath $baseTree) { [IO.Directory]::Delete($baseTree, $true) }
}
