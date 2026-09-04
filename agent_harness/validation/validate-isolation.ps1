[CmdletBinding()]
param(
    [string] $RunRoot = 'F:\TrinityR-runs',
    [string] $Distro = 'Ubuntu'
)

$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
try {
    & $launcher -Role A-01 -RunRoot $RunRoot -AccessProfile CONFIRMATION -Command 'true' 2>$null
    throw 'Confirmation access was not rejected.'
} catch {
    if ($_.Exception.Message -notmatch 'CONFIRMATION access is locked') {
        throw
    }
}
try {
    & $launcher -Role A-01 -RunRoot $RunRoot -AccessProfile DEVELOPMENT -Command 'true' 2>$null
    throw 'Development access was not blocked without a derived view.'
} catch {
    if ($_.Exception.Message -notmatch 'Required lake view not found') {
        throw
    }
}
foreach ($role in @('A-01', 'A-02')) {
    $peer = if ($role -eq 'A-01') { 'A-02' } else { 'A-01' }
    $probe = @'
set -eu
test "$(id -u)" -eq 65534
test "$(id -g)" -eq 65534
! env | grep -q '^WSL_INTEROP='
test "$LOGNAME" = nobody
test "$PATH" = /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
! env | grep -Eiq 'provider|model|codex|gemini|claude'
test -r /shared/research-program/contracts/knowledge_record_v1.schema.json
test -r /shared/research-program/instruments/XAUUSD/data_scope_v1.json
test ! -e /shared/research-program/crates
test ! -e /shared/research-program/agent_harness/launch
test ! -r /shared/lake/manifest.json
! mountpoint -q /shared/lake
test ! -e /mnt/f/TrinityR-research
test ! -e /mnt/c/Users
test ! -e /mnt/f/TrinityR-runs/AP-001/__PEER__
test ! -e /mnt/f/TrinityR-runs/AP-001/P-01
test ! -e /mnt/f/TrinityR-runs/AP-001/orchestrator/provider-map.json
test ! -e /mnt/c/Users/malic/.codex
test ! -r /proc/1/root/mnt/f/TrinityR-research
test ! -r /proc/1/root/mnt/c/Users
test ! -e /workspace/../mnt/f/TrinityR-research
! command -v cmd.exe
! command -v powershell.exe
test ! -w /shared/research-program/README.md
test ! -w /shared/lake/manifest.json
printf 'isolated-probe\n' > /workspace/probe.txt
test -s /workspace/probe.txt
printf 'ISOLATION_PASS __ROLE__\n'
'@
    $probe = $probe.Replace('__PEER__', $peer).Replace('__ROLE__', $role)
    $result = & $launcher -Role $role -RunRoot $RunRoot -Distro $Distro -Command $probe
    if ($LASTEXITCODE -ne 0 -or ($result -notcontains "ISOLATION_PASS $role")) {
        throw "AP-001 isolation validation failed for $role."
    }
    Write-Output ($result -join [Environment]::NewLine)
}
