[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{40}$')][string]$ResearchBaseSha,
    [string]$RunRoot = 'F:\TrinityR-runs\runtime-smoke',
    [string]$ManifestRoot = 'F:\TrinityR-manifests\wave1-v4-smoke',
    [string]$AuthorityRoot = 'F:\TrinityR-authority',
    [string]$Distro = 'Ubuntu'
)
$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot '..\launch\isolated-run.ps1'
$env:TRINITYR_AUTHORITY_ROOT = $AuthorityRoot
$env:TRINITYR_MANIFEST_ROOT = $ManifestRoot
$env:TRINITYR_LAUNCH_WORKTREE_ROOT = Join-Path $RunRoot '.launch-worktrees'
$jobs = @('AP-001\A-01','AP-001\A-02','AP-002\A-01','AP-002\A-02','TC-001\A-01','TC-001\A-02','BG-001\A-01','BG-001\A-02')
foreach ($job in $jobs) {
    $parts = $job -split '\\'; $program = $parts[0]; $role = $parts[1]
    $suffix = @{ 'AP-001'='ap001'; 'AP-002'='ap002'; 'TC-001'='tc001'; 'BG-001'='bg001' }[$program]
    $roleManifestRoot = Join-Path (Join-Path $ManifestRoot $program) $role
    $before = @(Get-ChildItem -LiteralPath $roleManifestRoot -Filter 'runtime_mount_attestation.json' -File -Recurse -ErrorAction SilentlyContinue | ForEach-Object FullName)
    $command = "test -s /shared/research_input_manifest.json && test -s /shared/research-program/protocol/RESEARCH_RULES.md && test -s /shared/research-program/agent_harness/assignments/$program/$role.json && test -s /shared/authority/xauusd/$suffix/authority.json && test ! -e /shared/data && test ! -e /shared/peer_workspace && test ! -e /shared/confirmation && test ! -e /shared/legacy && test ! -e /shared/provider_mapping && test -r /shared/runtime_mount_attestation/runtime_mount_attestation.json && test -w /workspace && test ! -w /shared/runtime_mount_attestation/runtime_mount_attestation.json"
    & $launcher -ProgramId $program -Role $role -RunRoot $RunRoot -Distro $Distro -ResearchBaseSha $ResearchBaseSha -LaunchPurpose SMOKE -Command $command
    if ($LASTEXITCODE -ne 0) { throw "Runtime smoke failed: $job" }
    $after = @(Get-ChildItem -LiteralPath $roleManifestRoot -Filter 'runtime_mount_attestation.json' -File -Recurse | ForEach-Object FullName)
    $new = @($after | Where-Object { $_ -notin $before })
    if ($new.Count -ne 1) { throw "Runtime attestation selection is ambiguous: $job" }
    $attestationPath = $new[0]
    if (-not (Test-Path -LiteralPath $attestationPath -PathType Leaf)) { throw "Runtime attestation missing: $job" }
    $attestation = Get-Content -Raw -LiteralPath $attestationPath | ConvertFrom-Json
    if ($attestation.all_match -ne $true) { throw "Runtime attestation mismatch: $job" }
    "RUNTIME_ATTESTATION_PASS $job $((Get-FileHash $attestationPath -Algorithm SHA256).Hash.ToLowerInvariant())"
}
'ALL_EIGHT_RUNTIME_SMOKE_PASS'
