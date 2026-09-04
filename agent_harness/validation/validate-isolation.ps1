[CmdletBinding()]
param([string] $RunRoot = 'F:\TrinityR-runs', [string] $Distro = 'Ubuntu', [string] $AuthorityRegistryPath, [switch] $LocalSynthetic)
$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
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
if (-not (SafeId 'SAFE-INSTRUMENT') -or (SafeId 'unsafe/id')) { throw 'Safe ID validation failed.' }
'GENERIC_ID_VALIDATION_PASS'
if ($LocalSynthetic) {
    $root=Join-Path ([IO.Path]::GetTempPath()) ('i02-local-'+[guid]::NewGuid());New-Item -ItemType Directory -Force (Join-Path $root 'nested')|Out-Null
    try { 'a'|Set-Content -NoNewline (Join-Path $root 'authority.json');'manifest'|Set-Content -NoNewline (Join-Path $root 'bundle_manifest.json');'b'|Set-Content -NoNewline (Join-Path $root 'nested\child.txt');$records=Get-ChildItem $root -Recurse -File|ForEach-Object{$relative=$_.FullName.Substring($root.Length+1).Replace('\','/');"$relative`t$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())`n"}|Sort-Object;$hash=([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($records -join '')))|ForEach-Object ToString x2)-join '';if($hash.Length -ne 64 -or $records.Count -ne 3){throw 'LOCAL_SYNTHETIC_HASH_FAILED'};'LOCAL_SYNTHETIC_PASS' } finally { Remove-Item -Recurse -Force -LiteralPath $root -ErrorAction SilentlyContinue };exit 0
}
function MustFail([scriptblock] $Action, [string] $Name) {
    try { & $Action 2>$null; throw "$Name unexpectedly succeeded" } catch { if ($_.Exception.Message -match 'unexpectedly succeeded') { throw } }
}
function ValidateBundleIdentity($Record,$Manifest,[string]$AuthorityPath) { $authorityHash=(Get-FileHash (Join-Path $AuthorityPath 'authority.json') -Algorithm SHA256).Hash.ToLowerInvariant();if($Manifest.bundle_content_identity -notmatch '^[0-9a-fA-F]{64}$' -or $Manifest.bundle_content_identity -ne $authorityHash){throw 'Bundle identity mismatch: bundle_content_identity'} }
function ValidateInputManifest($Manifest) { foreach($field in 'schema_version','program_id','role_id','phase','research_base_sha','assignment_path','assignment_sha256','access_profile','raw_lake_access','repository_surfaces','authority_sources','shared_input_fingerprint'){$value=$Manifest.PSObject.Properties[$field].Value;if($null -eq $value -or ($value -is [string] -and [string]::IsNullOrWhiteSpace($value)) -or ($value -is [Collections.IEnumerable] -and $value -isnot [string] -and @($value).Count -eq 0)){throw "Input manifest field missing: $field"}} }
function ValidateParityPair($Pair) { if($Pair.a01_shared_input_fingerprint -notmatch '^[0-9a-fA-F]{64}$' -or $Pair.a02_shared_input_fingerprint -notmatch '^[0-9a-fA-F]{64}$' -or $Pair.a01_shared_input_fingerprint -ne $Pair.a02_shared_input_fingerprint -or $Pair.match -ne $true){throw 'parity mismatch'} }
function ValidateArtifactClaims($Claims) { $seen=@{};foreach($claim in @($Claims)){if($claim.claim_id -notmatch '^[A-Z]{2}\d{3}-A-?\d{2}-C\d{3,}$' -or $seen[$claim.claim_id]){throw 'duplicate or invalid claim id'};$seen[$claim.claim_id]=$true;if($claim.status -notin @('AUTHORITATIVE','PROVISIONAL','UNRESOLVED')){throw 'invalid status'};$e=$claim.evidence;$ae=$claim.authority_evidence;if($claim.status -eq 'AUTHORITATIVE' -and (($null -eq $e -or @($e).Count -eq 0) -and ($null -eq $ae -or @($ae).Count -eq 0))){throw 'malformed AUTHORITATIVE evidence'}}}
MustFail { & $launcher -ProgramId DOES-NOT-EXIST -Role A-01 -Command true } 'missing program'
MustFail { & $launcher -ProgramId AP-001 -Role DOES-NOT-EXIST -Command true } 'missing role'
MustFail { & $launcher -ProgramId AP-001 -Role A-01 -Command true -RunRoot $repo } 'repository run root'
$authority = Get-Content -Raw $AuthorityRegistryPath | ConvertFrom-Json
$env:TRINITYR_AUTHORITY_ROOT = if ($env:TRINITYR_AUTHORITY_ROOT) { $env:TRINITYR_AUTHORITY_ROOT } else { 'F:\TrinityR-authority' }
$env:TRINITYR_MANIFEST_ROOT = if ($env:TRINITYR_MANIFEST_ROOT) { $env:TRINITYR_MANIFEST_ROOT } else { 'F:\TrinityR-manifests\wave1-v2' }
$profiles = Get-Content -Raw (Join-Path $repo 'agent_harness\access_profiles\default.json') | ConvertFrom-Json
$source = @($authority.sources)
if ($source.Count -ne 4 -or @($source | Where-Object { $_.source_id -notmatch '^xauusd\.wave1\.(ap001|ap002|tc001|bg001)$' }).Count) { throw 'Wave-1 authority source registry is incomplete.' }
foreach ($authoritySource in $source) {
 $sourcePath = Join-Path $env:TRINITYR_AUTHORITY_ROOT ($authoritySource.relative_path -replace '/','\')
 if ([string]::IsNullOrWhiteSpace([string]$authoritySource.hash) -or $authoritySource.hash -notmatch '^[0-9a-fA-F]{64}$') { throw "Empty authority hash: $($authoritySource.source_id)" }
 $members = Get-ChildItem -LiteralPath $sourcePath -Recurse -Force | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Authority source contains reparse point: $($_.FullName)" }
    if (-not $_.PSIsContainer) {
        $relative = $_.FullName.Substring($sourcePath.Length + 1).Replace('\','/')
        $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$relative`t$hash`n"
    }
 } | Sort-Object
$digest = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($members -join ''))) | ForEach-Object ToString x2) -join ''
 if ($digest -ne $authoritySource.hash.ToLowerInvariant()) { throw "Recursive authority hash mismatch: $($authoritySource.source_id)" }
 $bundleManifest=Get-Content -Raw (Join-Path $sourcePath 'bundle_manifest.json')|ConvertFrom-Json; ValidateBundleIdentity $authoritySource $bundleManifest $sourcePath
}
'AUTHORITY_RECURSIVE_HASH_PASS'
$apAuthority=Get-Content -Raw 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1\AP-001\authority.json'|ConvertFrom-Json
$tcAuthority=Get-Content -Raw 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1\TC-001\authority.json'|ConvertFrom-Json
$ap2Authority=Get-Content -Raw 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1\AP-002\authority.json'|ConvertFrom-Json
if (@($apAuthority.selected.PSObject.Properties).Count -eq 0 -or @($apAuthority.provenance).Count -eq 0 -or @($tcAuthority.selected.PSObject.Properties).Count -eq 0 -or @($tcAuthority.provenance).Count -eq 0) { throw 'Declared serialization authority is empty.' }
if (@($ap2Authority.selected.PSObject.Properties).Count -ne 0 -or @($ap2Authority.unresolved) -notcontains 'UNRESOLVED_PENDING_SOURCE_APPROVAL') { throw 'AP-002 source-approval state changed.' }
if (($apAuthority|ConvertTo-Json -Depth 40) -match 'market_samples|performance' -or ($tcAuthority|ConvertTo-Json -Depth 40) -match 'market_samples|performance') { throw 'Behavioral payload content leaked into authority bundle.' }
'DECLARED_SERIALIZATION_VISIBILITY_PASS'
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
        $surfacePath=Join-Path $repo ($surface -replace '/','\'); if ((Get-Item -LiteralPath $surfacePath).PSIsContainer -and @(Get-ChildItem -LiteralPath $surfacePath -Force).Count -eq 0) { throw "Wave-1 surface empty: $program/$role $surface" }
    }
    if ($assignment.raw_lake_access -ne 'DENY' -or $assignment.access_profile -ne 'AUTHORITY') { throw "Wave-1 authority access invalid: $program/$role" }
    $expectedSource = @{ 'AP-001'='xauusd.wave1.ap001'; 'AP-002'='xauusd.wave1.ap002'; 'TC-001'='xauusd.wave1.tc001'; 'BG-001'='xauusd.wave1.bg001' }[$program]
    if (@($assignment.authority_source_ids).Count -ne 1 -or $assignment.authority_source_ids[0] -ne $expectedSource) { throw "Wave-1 authority source mismatch: $program/$role" }
    foreach ($sourceId in @($assignment.authority_source_ids)) {
        if (@($authority.sources | Where-Object source_id -eq $sourceId).Count -ne 1) { throw "Wave-1 authority source missing: $program/$role $sourceId" }
    }
    "WAVE1_PREFLIGHT_PASS $program/$role"
}
'PAIR_INPUT_PARITY_PASS'
$parityRoot = $env:TRINITYR_MANIFEST_ROOT; New-Item -ItemType Directory -Force $parityRoot | Out-Null
$parity = [ordered]@{ schema_version='trinity.pair-input-parity.v1'; pairs=@() }
foreach ($program in @('AP-001','AP-002','TC-001','BG-001')) {
    $a = Get-Content -Raw (Join-Path $repo "agent_harness\assignments\$program\A-01.json") | ConvertFrom-Json
    $b = Get-Content -Raw (Join-Path $repo "agent_harness\assignments\$program\A-02.json") | ConvertFrom-Json
    $shared = [ordered]@{ program_id=$a.program_id; detector_id=$a.detector_id; assignment=$a.assignment; authorized_repository_surfaces=@($a.authorized_repository_surfaces | Where-Object { $_ -notmatch '^agent_harness/assignments/' } | Sort-Object); authority_source_ids=@($a.authority_source_ids | Sort-Object); authority_bundle_identities=@($a.authority_source_ids | ForEach-Object { $s=(@($authority.sources | Where-Object source_id -eq $_))[0]; $s.hash } | Sort-Object); raw_lake_access=$a.raw_lake_access; peer_visibility=$a.peer_visibility }
    $other = [ordered]@{ program_id=$b.program_id; detector_id=$b.detector_id; assignment=$b.assignment; authorized_repository_surfaces=@($b.authorized_repository_surfaces | Where-Object { $_ -notmatch '^agent_harness/assignments/' } | Sort-Object); authority_source_ids=@($b.authority_source_ids | Sort-Object); authority_bundle_identities=@($b.authority_source_ids | ForEach-Object { $s=(@($authority.sources | Where-Object source_id -eq $_))[0]; $s.hash } | Sort-Object); raw_lake_access=$b.raw_lake_access; peer_visibility=$b.peer_visibility }
    if (($shared | ConvertTo-Json -Compress -Depth 10) -cne ($other | ConvertTo-Json -Compress -Depth 10)) { throw "Pair input mismatch: $program" }
    $aFingerprint = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($shared|ConvertTo-Json -Compress -Depth 10))) | ForEach-Object ToString x2) -join ''
    $bFingerprint = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($other|ConvertTo-Json -Compress -Depth 10))) | ForEach-Object ToString x2) -join ''
    $parity.pairs += [ordered]@{ program_id=$program; a01_shared_input_fingerprint=$aFingerprint; a02_shared_input_fingerprint=$bFingerprint; match=($aFingerprint -eq $bFingerprint) }
    ValidateParityPair $parity.pairs[-1]
}
$parityPath=Join-Path $parityRoot 'pair_input_parity.json'; $parityJson=($parity|ConvertTo-Json -Depth 20)+[Environment]::NewLine
if (Test-Path $parityPath) { if ((Get-Content -Raw $parityPath) -ne $parityJson) { throw 'Immutable pair parity conflict.' } } else { $parityJson | Set-Content -NoNewline -Encoding utf8 $parityPath }
$badManifest=$bundleManifest.PSObject.Copy();$badManifest.bundle_content_identity=('0'*64);MustFail { ValidateBundleIdentity $source[0] $badManifest $sourcePath } 'wrong bundle content identity'
$badParity=$parity.pairs[0].PSObject.Copy();$badParity.a02_shared_input_fingerprint=('0'*64);MustFail {ValidateParityPair $badParity} 'parity mismatch'
$manifestCheck=Get-Content -Raw (Join-Path $env:TRINITYR_MANIFEST_ROOT 'AP-001\A-01\research_input_manifest.json')|ConvertFrom-Json;ValidateInputManifest $manifestCheck;$badManifest=$manifestCheck.PSObject.Copy();$badManifest.shared_input_fingerprint=$null;MustFail {ValidateInputManifest $badManifest} 'missing manifest fields'
$claimFixture=@([pscustomobject]@{claim_id='AP001-A01-C001';status='PROVISIONAL';claim='null model'});ValidateArtifactClaims $claimFixture;$duplicate=@($claimFixture+$claimFixture);MustFail {ValidateArtifactClaims $duplicate} 'duplicate IDs';$invalid=$claimFixture[0].PSObject.Copy();$invalid.status='UNRESOLVED_RELATIONSHIP';MustFail {ValidateArtifactClaims @($invalid)} 'invalid status';$malformed=$claimFixture[0].PSObject.Copy();$malformed.status='AUTHORITATIVE';MustFail {ValidateArtifactClaims @($malformed)} 'malformed AUTHORITATIVE evidence';'NEGATIVE_CONTRACT_TESTS_PASS'
$glm = Get-Content -Raw (Join-Path $repo 'agent_harness\assignments\TEST-GLM-01\TEST-GLM-01.json') | ConvertFrom-Json
ValidateAssignment $glm 'TEST-GLM-01/TEST-GLM-01'
if ($glm.agent_runtime -or $glm.provider -or $glm.model) { throw 'TEST-GLM scientific assignment selects runtime/provider.' }
'{"program_id":"SAFE","role_id":"SAFE","phase":"DEVELOPMENT","access_profile":"DEVELOPMENT","assignment":"x","authorized_repository_surfaces":["protocol"],"authority_source_ids":[],"raw_lake_access":"DENY","peer_visibility":"DENY","instrument_id":"SAFE","data_scope_id":"SAFE","nested":{"ToKeN":"blocked"}}' | ConvertFrom-Json | ForEach-Object { MustFail { ValidateAssignment $_ 'nested-operational-key' } 'recursive operational identity' }
'{"program_id":"SAFE","role_id":"SAFE","phase":"DEVELOPMENT","access_profile":"DEVELOPMENT","assignment":"x","authorized_repository_surfaces":["protocol"],"authority_source_ids":[],"raw_lake_access":"DENY","peer_visibility":"DENY","instrument_id":"SAFE","data_scope_id":"SAFE","detector_id":"unexpected"}' | ConvertFrom-Json | ForEach-Object { MustFail { ValidateAssignment $_ 'phase-allowlist' } 'phase field allowlist' }
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
    $oldErrorAction = $ErrorActionPreference
    try { $ErrorActionPreference = 'Continue'; $liveSocket = (& wsl.exe --distribution $Distro --user root -- find /tmp -maxdepth 1 -type s -name 'znr-*.sock' -print -quit 2>$null) } finally { $ErrorActionPreference = $oldErrorAction }
    if ($liveSocket) {
        @{ schema_version = 1; slots = @(@{ slot_id = 'slot-07'; broker_socket = [string]$liveSocket }) } | ConvertTo-Json -Compress | Set-Content -NoNewline $runtimeConfig
        $result = & $launcher -ProgramId TEST-GLM-01 -Role TEST-GLM-01 -RuntimeSlot slot-07 -RunRoot 'F:\trinityr-runtime-run' -Distro $Distro -Command 'test -S /run/trinityr/runtime.sock'
        if ($LASTEXITCODE -ne 0) { throw 'Active neutral runtime socket launch failed.' }
        'ACTIVE_NEUTRAL_SOCKET_PASS'
    }
    $codex = Get-Command codex -ErrorAction SilentlyContinue
    $zcode = (& wsl.exe --distribution $Distro --user root -- sh -lc 'command -v zcode-cli' 2>$null)
    if (-not $codex) { 'CODEX_RUNTIME_BLOCKED executable=codex-not-discovered; smallest_fix=provide-approved-runtime-backed-Codex-entrypoint' } else { 'CODEX_RUNTIME_BLOCKED actual-researcher-execution-not-authorized-in-this-validation; smallest_fix=run-test-only-Codex-through-approved-neutral-boundary' }
    if (-not $zcode) { 'ZCODE_RUNTIME_BLOCKED executable=zcode-cli-not-discovered; smallest_fix=provide-approved-runtime-backed-ZCode-entrypoint' } else { 'ZCODE_RUNTIME_BLOCKED actual-researcher-execution-not-authorized-in-this-validation; smallest_fix=run-test-only-ZCode-through-approved-neutral-boundary' }
    if (Select-String -Path (Join-Path $repo 'agent_harness\launch\isolated-run.sh') -Pattern 'RUNTIME_ENDPOINT') { throw 'Raw runtime endpoint remains exposed.' }
    'RUNTIME_OPACITY_BLOCKED'
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
    'manifest' | Set-Content -NoNewline (Join-Path $authorityTestRoot 'bundle_manifest.json')
    $records = Get-ChildItem $authorityTestRoot -Recurse -File | ForEach-Object {
        $relative = $_.FullName.Substring($authorityTestRoot.Length + 1).Replace('\','/')
        "$relative`t$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())`n"
    } | Sort-Object
    $v2hash = ([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes(($records -join ''))) | ForEach-Object ToString x2) -join ''
    if ($v2hash.Length -ne 64 -or $records.Count -ne 3) { throw 'Synthetic v2 authority hash failed.' }
    'AUTHORITY_V2_HASH_PASS'
} finally { Remove-Item -Recurse -Force -LiteralPath $authorityTestRoot -ErrorAction SilentlyContinue }
$probe = @'
set -eu
test "$(id -u)" -eq 65534; test "$(id -g)" -eq 65534
test "$LOGNAME" = nobody; ! env | grep -Eiq 'provider|codex|gemini|claude|WSL_INTEROP'
test -r /shared/research-program/protocol/RESEARCH_RULES.md
test -r /shared/research-program/contracts/knowledge_record_v1.schema.json
test -r /shared/research-program/instruments/XAUUSD/instrument_config.json
test -r /shared/research-program/instruments/XAUUSD/source_inventory.json
test "$(find /shared/research-program/instruments/XAUUSD -mindepth 1 -maxdepth 1 -printf '%f\n' | sort | tr '\n' ' ')" = "instrument_config.json source_inventory.json "
! mountpoint -q /shared/research-program/instruments/XAUUSD
test ! -e /shared/research-program/instruments/XAUUSD/findings
test ! -e /shared/research-program/instruments/XAUUSD/future-finding.json
test -r /shared/authority/xauusd/ap001/authority.json
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
$serializationProbes = @(@('TC-001','A-01','test -s /shared/authority/xauusd/tc001/authority.json; grep -q feed_health /shared/authority/xauusd/tc001/authority.json'), @('AP-002','A-01','test -s /shared/authority/xauusd/ap002/authority.json; grep -q UNRESOLVED_PENDING_SOURCE_APPROVAL /shared/authority/xauusd/ap002/authority.json'))
foreach ($slot in $serializationProbes) { $result=& $launcher -ProgramId $slot[0] -Role $slot[1] -RunRoot $RunRoot -Distro $Distro -Command "set -eu; $($slot[2]); printf 'SERIALIZATION_MOUNT_PASS\n'"; if($LASTEXITCODE -ne 0 -or ($result -notcontains 'SERIALIZATION_MOUNT_PASS')){throw "Serialization mount validation failed for $($slot[0])/$($slot[1])"};$result }
