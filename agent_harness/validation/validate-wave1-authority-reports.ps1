[CmdletBinding()]
param(
    [string] $RunRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$validator = Join-Path $PSScriptRoot 'validate-authority-report.ps1'
if ([string]::IsNullOrWhiteSpace($RunRoot)) {
    $RunRoot = Join-Path $repo '.runs\wave1-native-v2'
}
$runRootPath = [IO.Path]::GetFullPath($RunRoot).TrimEnd('\')
if (-not (Test-Path -LiteralPath $runRootPath -PathType Container)) {
    Write-Output "WAVE1_REPORT_GATE_FAIL run root missing: $runRootPath"
    exit 1
}

$jobs = @(
    @('AP-001','A-01'),
    @('AP-001','A-02'),
    @('AP-002','A-01'),
    @('AP-002','A-02'),
    @('TC-001','A-01'),
    @('TC-001','A-02'),
    @('BG-001','A-01'),
    @('BG-001','A-02')
)

$failures = @()
$passes = 0
foreach ($job in $jobs) {
    $program = $job[0]
    $role = $job[1]
    $workspace = Join-Path (Join-Path $runRootPath $program) $role
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $validator `
        -ProgramId $program -Role $role -Workspace $workspace 2>&1
    $exitCode = $LASTEXITCODE
    foreach ($line in @($output)) { Write-Output $line }
    if ($exitCode -eq 0) {
        $passes++
    } else {
        $failures += "$program/$role"
    }
}

if ($failures.Count -gt 0) {
    Write-Output "WAVE1_REPORT_GATE_FAIL passes=$passes failures=$($failures.Count) roles=$($failures -join ',')"
    exit 1
}

Write-Output "WAVE1_REPORT_GATE_PASS reports=$passes"
exit 0
