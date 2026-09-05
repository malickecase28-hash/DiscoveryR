Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$validator = Join-Path $PSScriptRoot 'validate-authority-report.ps1'
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ('trinityr-report-gate-' + [guid]::NewGuid().ToString('N'))

function Assert-Equal([object] $actual, [object] $expected, [string] $message) {
    if ($actual -ne $expected) { throw "$message (actual=$actual expected=$expected)" }
}

function Write-Report([string] $workspace, [object] $candidate, [string] $review = 'review') {
    New-Item -ItemType Directory -Path $workspace -Force | Out-Null
    $candidate | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $workspace 'authority_candidate.json') -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $workspace 'authority_review.md') -Value $review -Encoding UTF8
}

function Invoke-Gate([string] $workspace) {
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $validator `
        -ProgramId 'AP-001' -Role 'A-01' -Workspace $workspace 2>&1
    [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = ($output -join "`n") }
}

try {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

    $valid = [ordered]@{
        artifact_type = 'AUTHORITY_CANDIDATE'
        program_id = 'AP-001'
        role_id = 'A-01'
        subjects = @('drift_burst')
        claims = @([ordered]@{
            claim_id = 'AP001-A01-C001'
            subject = 'drift_burst'
            topic = 'episode_identity'
            claim = 'Unresolved.'
            status = 'UNRESOLVED'
            authority_evidence = @()
            counterevidence = @()
            future_information_risk = 'UNRESOLVED'
            dependency_implications = @()
            notes = ''
        })
        unresolved_questions = @()
        sources_consulted = @([ordered]@{
            source_id = 'protocol'
            relative_path = 'RESEARCH_RULES.md'
            locator = 'section 1'
        })
    }

    $validPath = Join-Path $tempRoot 'valid'
    Write-Report $validPath $valid
    $result = Invoke-Gate $validPath
    Assert-Equal $result.ExitCode 0 "canonical valid fixture must pass: $($result.Output)"
    if ($result.Output -notmatch 'REPORT_GATE_PASS AP-001/A-01 claims=1') { throw "valid fixture output mismatch: $($result.Output)" }

    $invalidEvidence = @(
        [ordered]@{ source_id = 'x'; path = 'x'; fields = 'x' },
        [ordered]@{ source_id = 'x'; pointer = 'x'; evidence = 'x' },
        [ordered]@{ source = 'x'; location = 'x'; observed = 'x' },
        [ordered]@{ source_id = 'x'; relative_path = 'x'; locator = 'x'; evidence_type = 'x'; supports = 'x'; extra = 'x' }
    )
    foreach ($evidence in $invalidEvidence) {
        $candidate = $valid | ConvertTo-Json -Depth 20 | ConvertFrom-Json
        $candidate.claims[0].status = 'AUTHORITATIVE'
        $candidate.claims[0].authority_evidence = @($evidence)
        $path = Join-Path $tempRoot ([guid]::NewGuid().ToString('N'))
        Write-Report $path $candidate
        $result = Invoke-Gate $path
        Assert-Equal $result.ExitCode 1 'invalid evidence fixture must fail'
    }

    $cases = @(
        @{ Name = 'duplicate claim_id'; Mutate = { param($c) $c.claims = @($c.claims[0], $c.claims[0]) } },
        @{ Name = 'invalid status'; Mutate = { param($c) $c.claims[0].status = 'UNRESOLVED_RELATIONSHIP' } },
        @{ Name = 'empty authoritative evidence'; Mutate = { param($c) $c.claims[0].status = 'AUTHORITATIVE' } },
        @{ Name = 'wrong program'; Mutate = { param($c) $c.program_id = 'AP-002' } },
        @{ Name = 'wrong role'; Mutate = { param($c) $c.role_id = 'A-02' } },
        @{ Name = 'lowercase artifact type'; Mutate = { param($c) $c.artifact_type = 'authority_candidate' } }
    )
    foreach ($case in $cases) {
        $candidate = $valid | ConvertTo-Json -Depth 20 | ConvertFrom-Json
        & $case.Mutate $candidate
        $path = Join-Path $tempRoot ([guid]::NewGuid().ToString('N'))
        Write-Report $path $candidate
        $result = Invoke-Gate $path
        Assert-Equal $result.ExitCode 1 "$($case.Name) fixture must fail"
    }

    $extra = $valid | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $extra | Add-Member -NotePropertyName unexpected -NotePropertyValue 'x'
    $extraPath = Join-Path $tempRoot 'extra-top-level'
    Write-Report $extraPath $extra
    Assert-Equal (Invoke-Gate $extraPath).ExitCode 1 'extra top-level key fixture must fail'

    $reviewPath = Join-Path $tempRoot 'bad-review'
    Write-Report $reviewPath $valid 'This mentions Codex.'
    Assert-Equal (Invoke-Gate $reviewPath).ExitCode 1 'runtime identity in review must fail'

    Write-Output 'REPORT_GATE_TEST_PASS valid=1 negatives=11'
    exit 0
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        [System.IO.Directory]::Delete($tempRoot, $true)
    }
}
