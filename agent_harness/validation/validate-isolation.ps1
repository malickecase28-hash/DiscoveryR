[CmdletBinding()]
param([string] $RunRoot = 'F:\TrinityR-runs', [string] $Distro = 'Ubuntu')
$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
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
    $allowed = @('program_id','role_id','phase','access_profile','assignment','authorized_repository_surfaces','authority_source_ids','raw_lake_access','peer_visibility','private_workspace','detector_id','instrument_id','data_scope_id','authority_registry_id')
    $unknown = @($Assignment.psobject.Properties.Name | Where-Object { $_ -notin $allowed })
    if ($unknown.Count) { throw "Unknown assignment fields: $Name/$($unknown -join ',')" }
    if (-not (SafeId $Assignment.program_id) -or -not (SafeId $Assignment.role_id)) { throw "Unsafe assignment identity: $Name" }
    if ($Assignment.raw_lake_access -ne 'DENY' -or $Assignment.peer_visibility -ne 'DENY') { throw "Unsafe assignment policy: $Name" }
    if ($Assignment.access_profile -eq 'DEVELOPMENT' -and (-not (SafeId $Assignment.instrument_id) -or -not (SafeId $Assignment.data_scope_id))) { throw "DEVELOPMENT identity missing: $Name" }
}
if (-not (SafeId 'SAFE-INSTRUMENT') -or (SafeId 'unsafe/id')) { throw 'Safe ID validation failed.' }
'GENERIC_ID_VALIDATION_PASS'
function MustFail([scriptblock] $Action, [string] $Name) {
    try { & $Action 2>$null; throw "$Name unexpectedly succeeded" } catch { if ($_.Exception.Message -match 'unexpectedly succeeded') { throw } }
}
MustFail { & $launcher -ProgramId DOES-NOT-EXIST -Role A-01 -Command true } 'missing program'
MustFail { & $launcher -ProgramId AP-001 -Role DOES-NOT-EXIST -Command true } 'missing role'
MustFail { & $launcher -ProgramId AP-001 -Role A-01 -Command true -RunRoot $repo } 'repository run root'
$authority = Get-Content -Raw (Join-Path $repo 'agent_harness\authority_sources\XAUUSD.json') | ConvertFrom-Json
$profiles = Get-Content -Raw (Join-Path $repo 'agent_harness\access_profiles\default.json') | ConvertFrom-Json
$source = @($authority.sources | Where-Object source_id -eq 'xauusd.schemas')
$sourcePath = Join-Path $source[0].root_path ($source[0].relative_path -replace '/','\')
$members = Get-ChildItem -LiteralPath $sourcePath -Recurse -Force | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Authority source contains reparse point: $($_.FullName)" }
    if (-not $_.PSIsContainer) {
        $relative = $_.FullName.Substring($sourcePath.Length + 1).Replace('\','/')
        $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$relative`t$hash`n"
    }
} | Sort-Object
$digest = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($members -join ''))) | ForEach-Object ToString x2) -join ''
if ($digest -ne $source[0].hash) { throw "Recursive authority hash mismatch: $digest" }
'AUTHORITY_RECURSIVE_HASH_PASS'
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
    foreach ($surface in @($assignment.authorized_repository_surfaces)) {
        if (-not (Test-Path -LiteralPath (Join-Path $repo ($surface -replace '/','\')))) { throw "Wave-1 surface missing: $program/$role $surface" }
    }
    if ($assignment.raw_lake_access -ne 'DENY' -or $assignment.access_profile -ne 'AUTHORITY') { throw "Wave-1 authority access invalid: $program/$role" }
    foreach ($sourceId in @($assignment.authority_source_ids)) {
        if (@($authority.sources | Where-Object source_id -eq $sourceId).Count -ne 1) { throw "Wave-1 authority source missing: $program/$role $sourceId" }
    }
    "WAVE1_PREFLIGHT_PASS $program/$role"
}
$glm = Get-Content -Raw (Join-Path $repo 'agent_harness\assignments\TEST-GLM-01\TEST-GLM-01.json') | ConvertFrom-Json
ValidateAssignment $glm 'TEST-GLM-01/TEST-GLM-01'
if ($glm.agent_runtime -or $glm.provider -or $glm.model) { throw 'TEST-GLM scientific assignment selects runtime/provider.' }
'OPAQUE_ASSIGNMENT_PASS'
$dev = Join-Path $repo 'agent_harness\assignments\TEST-DEVELOPMENT-01\TEST-DEVELOPMENT-01.json'
ValidateAssignment (Get-Content -Raw $dev | ConvertFrom-Json) 'TEST-DEVELOPMENT-01'
$viewRoot = "F:\trinityr-view-$PID"
$view = Join-Path $viewRoot 'XAUUSD\XAUUSD_DATA_SCOPE_V1\development'
$devRun = "F:\trinityr-run-$PID"
$oldViews = $env:TRINITYR_VIEWS_ROOT
try {
    $env:TRINITYR_VIEWS_ROOT = $viewRoot
    MustFail { & $launcher -ProgramId TEST-DEVELOPMENT-01 -Role TEST-DEVELOPMENT-01 -RunRoot $devRun -Command true } 'missing development view'
    New-Item -ItemType Directory -Force $view | Out-Null
    MustFail { & $launcher -ProgramId TEST-DEVELOPMENT-01 -Role TEST-DEVELOPMENT-01 -RunRoot $devRun -Command true } 'missing development manifest'
    '{"schema_version":"1","instrument":"WRONG","scope_id":"XAUUSD_DATA_SCOPE_V1","logical_view_identity":"test","builder_code_identity":"test"}' | Set-Content -NoNewline (Join-Path $view 'view_manifest.json')
    MustFail { & $launcher -ProgramId TEST-DEVELOPMENT-01 -Role TEST-DEVELOPMENT-01 -RunRoot $devRun -Command true } 'mismatched development manifest'
    '{"schema_version":"1","instrument":"XAUUSD","scope_id":"XAUUSD_DATA_SCOPE_V1","logical_view_identity":"test","builder_code_identity":"test"}' | Set-Content -NoNewline (Join-Path $view 'view_manifest.json')
    $result = & $launcher -ProgramId TEST-DEVELOPMENT-01 -Role TEST-DEVELOPMENT-01 -RunRoot $devRun -Command 'test -r /shared/data/XAUUSD_DATA_SCOPE_V1/development/view_manifest.json; test ! -e /shared/data/OTHER/development'
    if ($LASTEXITCODE -ne 0) { throw 'Generic development launch failed.' }
'GENERIC_DEVELOPMENT_PASS'
} finally {
    $env:TRINITYR_VIEWS_ROOT = $oldViews
    Remove-Item -Recurse -Force -LiteralPath $viewRoot,$devRun -ErrorAction SilentlyContinue
}
$runtimeConfig = "F:\trinityr-runtime-$PID.json"
try {
    '{"schema_version":1,"slots":[{"slot_id":"slot-07","broker_socket":"http://user:secret@provider.example/run?token=x"}]}' | Set-Content -NoNewline $runtimeConfig
    $env:TRINITYR_RUNTIME_CONFIG = $runtimeConfig
    MustFail { & $launcher -ProgramId TEST-GLM-01 -Role TEST-GLM-01 -RuntimeSlot slot-07 -RunRoot 'F:\trinityr-runtime-run' -Command true } 'unsafe runtime config'
    '{"schema_version":1,"slots":[{"slot_id":"slot-07","broker_socket":"/run/trinityr/missing.sock"}]}' | Set-Content -NoNewline $runtimeConfig
    MustFail { & $launcher -ProgramId TEST-GLM-01 -Role TEST-GLM-01 -RuntimeSlot slot-07 -RunRoot 'F:\trinityr-runtime-run' -Command true } 'missing runtime broker'
    if (Select-String -Path (Join-Path $repo 'agent_harness\launch\isolated-run.sh') -Pattern 'RUNTIME_ENDPOINT') { throw 'Raw runtime endpoint remains exposed.' }
    'RUNTIME_OPACITY_PASS'
} finally {
    Remove-Item -Force -LiteralPath $runtimeConfig -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force -LiteralPath 'F:\trinityr-runtime-run' -ErrorAction SilentlyContinue
    Remove-Item Env:TRINITYR_RUNTIME_CONFIG -ErrorAction SilentlyContinue
}
$authorityTestRoot = "F:\trinityr-authority-$PID"
try {
    New-Item -ItemType Directory -Force (Join-Path $authorityTestRoot 'nested') | Out-Null
    'a' | Set-Content -NoNewline (Join-Path $authorityTestRoot 'root.txt')
    'b' | Set-Content -NoNewline (Join-Path $authorityTestRoot 'nested\child.txt')
    $records = Get-ChildItem $authorityTestRoot -Recurse -File | ForEach-Object {
        $relative = $_.FullName.Substring($authorityTestRoot.Length + 1).Replace('\','/')
        "$relative`t$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())`n"
    } | Sort-Object
    $v2hash = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($records -join ''))) | ForEach-Object ToString x2) -join ''
    if ($v2hash.Length -ne 64 -or $records.Count -ne 2) { throw 'Synthetic v2 authority hash failed.' }
    'AUTHORITY_V2_HASH_PASS'
} finally { Remove-Item -Recurse -Force -LiteralPath $authorityTestRoot -ErrorAction SilentlyContinue }
$probe = @'
set -eu
test "$(id -u)" -eq 65534; test "$(id -g)" -eq 65534
test "$LOGNAME" = nobody; ! env | grep -Eiq 'provider|model|codex|gemini|claude|WSL_INTEROP'
test -r /shared/research-program/protocol/RESEARCH_RULES.md
test -r /shared/research-program/contracts/knowledge_record_v1.schema.json
test -r /shared/research-program/instruments/XAUUSD/instrument_config.json
test -r /shared/research-program/instruments/XAUUSD/source_inventory.json
test "$(find /shared/research-program/instruments/XAUUSD -mindepth 1 -maxdepth 1 -printf '%f\n' | sort | tr '\n' ' ')" = "instrument_config.json source_inventory.json "
! mountpoint -q /shared/research-program/instruments/XAUUSD
test ! -e /shared/research-program/instruments/XAUUSD/findings
test ! -e /shared/research-program/instruments/XAUUSD/future-finding.json
test -r /shared/authority/xauusd/schemas/bars_canonical_v1.json
test ! -r /shared/lake/manifest.json; test ! -e /mnt/c; test ! -e /mnt/d; test ! -e /mnt/e; test ! -e /mnt/f
test ! -e /home/malo; test ! -e /root/.codex
test ! -r /proc/1/root/mnt/f/TrinityR-research
test "$(awk '/^NSpid:/{print NF}' /proc/self/status)" -ge 2
test "$(tr '\0' ' ' </proc/1/cmdline)" != "/sbin/init "
test ! -w /shared/research-program/contracts/knowledge_record_v1.schema.json
printf 'probe' > /workspace/probe.txt; test -s /workspace/probe.txt
printf 'ISOLATION_PASS __ROLE__\n'
'@
foreach ($role in @('A-01', 'A-02')) {
    $result = & $launcher -ProgramId AP-001 -Role $role -RunRoot $RunRoot -Distro $Distro -Command $probe.Replace('__ROLE__', $role)
    if ($LASTEXITCODE -ne 0 -or ($result -notcontains "ISOLATION_PASS $role")) { throw "Isolation validation failed for $role." }
    $result
}
