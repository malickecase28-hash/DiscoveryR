[CmdletBinding()]
param([string] $RunRoot = 'F:\TrinityR-runs', [string] $Distro = 'Ubuntu')
$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
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
$probe = $probe -replace "`r`n", "`n"
foreach ($role in @('A-01', 'A-02')) {
    $result = & $launcher -ProgramId AP-001 -Role $role -RunRoot $RunRoot -Distro $Distro -Command $probe.Replace('__ROLE__', $role)
    if ($LASTEXITCODE -ne 0 -or ($result -notcontains "ISOLATION_PASS $role")) { throw "Isolation validation failed for $role." }
    $result
}
