[CmdletBinding()]
param(
    [string] $RunRoot = 'F:\TrinityR-runs',
    [string] $Distro = 'Ubuntu'
)

$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
foreach ($role in @('A-01', 'A-02')) {
    $peer = if ($role -eq 'A-01') { 'A-02' } else { 'A-01' }
    $probe = @'
set -eu
test "$(id -u)" -eq 65534
test -r /shared/research-program/README.md
test -r /shared/lake/manifest.json
test ! -e /mnt/f/TrinityR-research
test ! -e /mnt/c/Users
test ! -e /mnt/f/TrinityR-runs/AP-001/__PEER__
test ! -e /mnt/f/TrinityR-runs/AP-001/P-01
test ! -e /mnt/c/Users/malic/.codex
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
