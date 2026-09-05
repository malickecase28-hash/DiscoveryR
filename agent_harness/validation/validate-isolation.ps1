[CmdletBinding()]
param([string] $AuthorityRegistryPath)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if ([string]::IsNullOrWhiteSpace($AuthorityRegistryPath)) { $AuthorityRegistryPath = Join-Path $repo 'agent_harness\authority_sources\XAUUSD.json' }
elseif (-not [IO.Path]::IsPathRooted($AuthorityRegistryPath)) { $AuthorityRegistryPath = Join-Path $repo $AuthorityRegistryPath }

function SafeId([string] $Value) { $Value -match '^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$' }
function RejectOperationalKeys($Node, [string] $Path = 'assignment') {
    if ($Node -is [PSCustomObject]) {
        foreach ($property in $Node.psobject.Properties) {
            if ($property.Name -match '^(provider|provider_id|provider_identity|model|model_id|model_identity|runtime|runtime_slot|runtime_config|credential|credential_path|secret|token)$') { throw "Operational selector present: $Path.$($property.Name)" }
            RejectOperationalKeys $property.Value "$Path.$($property.Name)"
        }
    } elseif ($Node -is [System.Collections.IEnumerable] -and $Node -isnot [string]) {
        foreach ($item in $Node) { RejectOperationalKeys $item $Path }
    }
}
function ValidateAssignment($Assignment, [string] $Name) {
    RejectOperationalKeys $Assignment $Name
    $common = @('program_id','role_id','phase','access_profile','assignment','authorized_repository_surfaces','authority_source_ids','raw_lake_access','peer_visibility')
    $phaseFields = @{
        AUTHORITY = @('private_workspace','detector_id','instrument_id','authority_registry_id')
        DEVELOPMENT = @('instrument_id','data_scope_id','authority_registry_id')
        CONFIRMATION = @('instrument_id','authority_registry_id')
    }
    $allowed = @($common + $phaseFields[$Assignment.phase])
    $unknown = @($Assignment.psobject.Properties.Name | Where-Object { $_ -notin $allowed })
    if ($unknown.Count) { throw "Unknown assignment fields: $Name/$($unknown -join ',')" }
    if (-not (SafeId $Assignment.program_id) -or -not (SafeId $Assignment.role_id)) { throw "Unsafe assignment identity: $Name" }
    if ($Assignment.raw_lake_access -ne 'DENY' -or $Assignment.peer_visibility -ne 'DENY') { throw "Unsafe assignment policy: $Name" }
    if ($Assignment.access_profile -eq 'DEVELOPMENT' -and (-not (SafeId $Assignment.instrument_id) -or -not (SafeId $Assignment.data_scope_id))) { throw "DEVELOPMENT identity missing: $Name" }
}
function MustFail([scriptblock] $Action, [string] $Name) {
    try { & $Action; throw "$Name unexpectedly succeeded" } catch { if ($_.Exception.Message -match 'unexpectedly succeeded') { throw } }
}
function SharedInputs($Assignment, $Authority) {
    [ordered]@{
        program_id = $Assignment.program_id
        detector_id = $Assignment.detector_id
        assignment = $Assignment.assignment
        authorized_repository_surfaces = @($Assignment.authorized_repository_surfaces | Where-Object { $_ -notmatch '^agent_harness/assignments/' } | Sort-Object)
        authority_source_ids = @($Assignment.authority_source_ids | Sort-Object)
        authority_bundle_identities = @($Assignment.authority_source_ids | ForEach-Object { (@($Authority.sources | Where-Object source_id -eq $_))[0].hash } | Sort-Object)
        raw_lake_access = $Assignment.raw_lake_access
        peer_visibility = $Assignment.peer_visibility
    }
}

$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
$launcherText = Get-Content -Raw $launcher
if ($launcherText -match '(?i)wsl\.exe|zcode-cli|TrinityR-runs|RuntimeSlot|broker_socket|TRINITYR_RUNTIME_CONFIG') { throw 'Retired runtime surface remains in the native launcher.' }
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot '..\launch\isolated-run.sh')) { throw 'Retired WSL launcher remains.' }
if ($launcherText -notmatch '\.runs\\wave1-native-v2' -or $launcherText -notmatch 'TRINITYR_WORKSPACE') { throw 'Native launcher is not bound to the approved folder workflow.' }
'NATIVE_WINDOWS_POLICY_PASS'

$authority = Get-Content -Raw $AuthorityRegistryPath | ConvertFrom-Json
if (@($authority.sources).Count -ne 1) { throw 'Canonical authority source registry is incomplete.' }
foreach ($source in @($authority.sources)) {
    if ($source.source_id -ne 'xauusd.schemas' -or $source.hash -notmatch '^[0-9a-fA-F]{64}$') { throw "Invalid authority source record: $($source.source_id)" }
}
'AUTHORITY_REGISTRY_SHAPE_PASS'

$profiles = Get-Content -Raw (Join-Path $repo 'agent_harness\access_profiles\default.json') | ConvertFrom-Json
foreach ($slot in @(
    @('AP-001','A-01'), @('AP-001','A-02'), @('AP-002','A-01'), @('AP-002','A-02'),
    @('TC-001','A-01'), @('TC-001','A-02'), @('BG-001','A-01'), @('BG-001','A-02')
)) {
    $program = $slot[0]; $role = $slot[1]
    $path = Join-Path $repo "agent_harness\assignments\$program\$role.json"
    $assignment = Get-Content -Raw $path | ConvertFrom-Json
    ValidateAssignment $assignment "$program/$role"
    if ($assignment.program_id -ne $program -or $assignment.role_id -ne $role) { throw "Wave-1 identity mismatch: $program/$role" }
    if (-not $profiles.profiles.($assignment.access_profile)) { throw "Wave-1 profile missing: $program/$role" }
    if ($assignment.access_profile -ne 'AUTHORITY' -or @($assignment.authority_source_ids).Count -ne 1) { throw "Wave-1 authority assignment invalid: $program/$role" }
    if ($assignment.authority_source_ids[0] -ne 'xauusd.schemas') { throw "Wave-1 authority source mismatch: $program/$role" }
    "WAVE1_PREFLIGHT_PASS $program/$role"
}

foreach ($program in @('AP-001','AP-002','TC-001','BG-001')) {
    $a = Get-Content -Raw (Join-Path $repo "agent_harness\assignments\$program\A-01.json") | ConvertFrom-Json
    $b = Get-Content -Raw (Join-Path $repo "agent_harness\assignments\$program\A-02.json") | ConvertFrom-Json
    if ((SharedInputs $a $authority | ConvertTo-Json -Compress -Depth 10) -cne (SharedInputs $b $authority | ConvertTo-Json -Compress -Depth 10)) { throw "Pair input mismatch: $program" }
    "PAIR_INPUT_PARITY_PASS $program"
}

$runRoot = (Resolve-Path (Join-Path $repo '.runs\wave1-native-v2')).Path
$result = & $launcher -ProgramId TEST-GLM-01 -Role TEST-GLM-01 -RunRoot $runRoot -Command 'if ($env:TRINITYR_NATIVE_WORKSPACE_ONLY -ne ''1'' -or -not (Test-Path -LiteralPath $env:TRINITYR_WORKSPACE -PathType Container)) { exit 1 }; Write-Output NATIVE_FOLDER_LAUNCH_PASS'
if ($LASTEXITCODE -ne 0 -or $result -notcontains 'NATIVE_FOLDER_LAUNCH_PASS') { throw 'Native folder launch failed.' }
'NATIVE_FOLDER_LAUNCH_PASS'
'BOUNDARY=WORKFLOW_ONLY_HARD_ISOLATION_NOT_CLAIMED'
