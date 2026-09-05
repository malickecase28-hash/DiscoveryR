Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$validator = Join-Path $PSScriptRoot 'validate-authority-report.ps1'
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ('trinityr-report-gate-' + [guid]::NewGuid().ToString('N'))
$negativeCount = 0
$validCount = 0

function Assert-Equal([object] $actual, [object] $expected, [string] $message) {
    if ($actual -ne $expected) { throw "$message (actual=$actual expected=$expected)" }
}

function Write-Report([string] $workspace, [object] $candidate, [string] $review = 'review') {
    New-Item -ItemType Directory -Path $workspace -Force | Out-Null
    $candidate | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $workspace 'authority_candidate.json') -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $workspace 'authority_review.md') -Value $review -Encoding UTF8
}

function Invoke-Gate([string] $workspace) {
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $validator `
        -ProgramId 'AP-001' -Role 'A-01' -Workspace $workspace 2>&1
    [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = ($output -join "`n") }
}

function Clone([object] $value) {
    return ($value | ConvertTo-Json -Depth 30 | ConvertFrom-Json)
}

function Expect-Fail([object] $candidate, [string] $name, [string] $review = 'review') {
    $script:negativeCount++
    $path = Join-Path $tempRoot ('neg-' + $negativeCount.ToString('D2') + '-' + ($name -replace '[^A-Za-z0-9._-]', '-'))
    Write-Report $path $candidate $review
    $result = Invoke-Gate $path
    Assert-Equal $result.ExitCode 1 "$name fixture must fail: $($result.Output)"
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
            claim = 'Episode identity remains unresolved.'
            status = 'UNRESOLVED'
            authority_evidence = @()
            counterevidence = @()
            future_information_risk = 'UNRESOLVED'
            dependency_implications = @()
            notes = ''
        })
        unresolved_questions = @()
        sources_consulted = @([ordered]@{
            source_id = 'repo.protocol'
            relative_path = 'RESEARCH_RULES.md'
            locator = 'causal availability rule'
        })
    }

    $validPath = Join-Path $tempRoot 'valid-unresolved'
    Write-Report $validPath $valid
    $result = Invoke-Gate $validPath
    Assert-Equal $result.ExitCode 0 "canonical unresolved fixture must pass: $($result.Output)"
    if ($result.Output -notmatch 'REPORT_GATE_PASS AP-001/A-01 claims=1') { throw "valid fixture output mismatch: $($result.Output)" }
    $validCount++

    $validAuthoritative = Clone $valid
    $validAuthoritative.claims[0].status = 'AUTHORITATIVE'
    $validAuthoritative.claims[0].claim = 'The protocol requires causal availability.'
    $validAuthoritative.claims[0].authority_evidence = @([pscustomobject]@{
        source_id = 'repo.protocol'
        relative_path = 'RESEARCH_RULES.md'
        locator = 'causal availability rule'
        evidence_type = 'PROTOCOL'
        supports = 'Only information available by anchor time is lawful.'
    })
    $validAuthoritativePath = Join-Path $tempRoot 'valid-authoritative'
    Write-Report $validAuthoritativePath $validAuthoritative
    $result = Invoke-Gate $validAuthoritativePath
    Assert-Equal $result.ExitCode 0 "canonical authoritative fixture must pass: $($result.Output)"
    $validCount++

    foreach ($evidence in @(
        [ordered]@{ source_id = 'x'; path = 'x'; fields = 'x' },
        [ordered]@{ source_id = 'x'; pointer = 'x'; evidence = 'x' },
        [ordered]@{ source = 'x'; location = 'x'; observed = 'x' },
        [ordered]@{ source_id = 'x'; relative_path = 'x'; locator = 'x'; evidence_type = 'x'; supports = 'x'; extra = 'x' }
    )) {
        $candidate = Clone $valid
        $candidate.claims[0].status = 'AUTHORITATIVE'
        $candidate.claims[0].authority_evidence = @($evidence)
        Expect-Fail $candidate 'noncanonical-evidence'
    }

    $candidate = Clone $valid
    $candidate.claims = @($candidate.claims[0], $candidate.claims[0])
    Expect-Fail $candidate 'duplicate-claim-id'

    $candidate = Clone $valid
    $candidate.claims[0].claim_id = 'WRONG-C001'
    Expect-Fail $candidate 'wrong-claim-id-prefix'

    $candidate = Clone $valid
    $candidate.claims[0].status = 'UNRESOLVED_RELATIONSHIP'
    Expect-Fail $candidate 'invalid-status'

    $candidate = Clone $valid
    $candidate.claims[0].status = 'AUTHORITATIVE'
    Expect-Fail $candidate 'empty-authoritative-evidence'

    $candidate = Clone $valid
    $candidate.claims[0].status = 'PROVISIONAL'
    Expect-Fail $candidate 'empty-provisional-evidence'

    $candidate = Clone $valid
    $candidate.program_id = 'AP-002'
    Expect-Fail $candidate 'wrong-program'

    $candidate = Clone $valid
    $candidate.role_id = 'A-02'
    Expect-Fail $candidate 'wrong-role'

    $candidate = Clone $valid
    $candidate.artifact_type = 'authority_candidate'
    Expect-Fail $candidate 'lowercase-artifact-type'

    $candidate = Clone $valid
    $candidate | Add-Member -NotePropertyName unexpected -NotePropertyValue 'x'
    Expect-Fail $candidate 'extra-top-level-key'

    $candidate = Clone $valid
    $candidate.subjects = 'drift_burst'
    Expect-Fail $candidate 'subjects-must-be-array'

    $candidate = Clone $valid
    $candidate.sources_consulted = $candidate.sources_consulted[0]
    Expect-Fail $candidate 'sources-consulted-must-be-array'

    $candidate = Clone $validAuthoritative
    $candidate.claims[0].authority_evidence = $candidate.claims[0].authority_evidence[0]
    Expect-Fail $candidate 'authority-evidence-must-be-array'

    $candidate = Clone $valid
    $candidate.claims[0].counterevidence = 'none'
    Expect-Fail $candidate 'counterevidence-must-be-array'

    $candidate = Clone $valid
    $candidate.claims[0].dependency_implications = 'none'
    Expect-Fail $candidate 'dependency-implications-must-be-array'

    $candidate = Clone $valid
    $candidate.claims[0].future_information_risk = ''
    Expect-Fail $candidate 'future-information-risk-empty'

    $candidate = Clone $valid
    $candidate.claims[0].notes = $null
    Expect-Fail $candidate 'notes-must-be-string'

    $candidate = Clone $validAuthoritative
    $candidate.claims[0].authority_evidence[0].relative_path = '../secret.txt'
    Expect-Fail $candidate 'unsafe-relative-path'

    $candidate = Clone $validAuthoritative
    $candidate.claims[0].authority_evidence[0].source_id = 'made.up.source'
    Expect-Fail $candidate 'unauthorized-source-id'

    $candidate = Clone $validAuthoritative
    $candidate.claims[0].authority_evidence[0].relative_path = 'DOES_NOT_EXIST.md'
    Expect-Fail $candidate 'missing-source-file'

    $candidate = Clone $valid
    $candidate.claims[0].notes = 'Codex produced this.'
    Expect-Fail $candidate 'runtime-identity-in-candidate'

    Expect-Fail (Clone $valid) 'runtime-identity-in-review' 'This mentions Codex.'
    Expect-Fail (Clone $valid) 'peer-report-reference' 'The peer report agrees.'

    Write-Output "REPORT_GATE_TEST_PASS valid=$validCount negatives=$negativeCount"
    exit 0
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        [System.IO.Directory]::Delete($tempRoot, $true)
    }
}
