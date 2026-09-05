[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string] $ProgramId,
    [Parameter(Mandatory = $true)] [string] $Role,
    [Parameter(Mandatory = $true)] [string] $Workspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Fail([string] $reason) {
    Write-Output "REPORT_GATE_FAIL $ProgramId/$Role $reason"
    exit 1
}

function Keys([object] $value) {
    if ($null -eq $value) { return @() }
    return @($value.PSObject.Properties.Name)
}

function Has-ExactKeys([object] $value, [string[]] $expected) {
    $actual = @(Keys $value | Sort-Object)
    $wanted = @($expected | Sort-Object)
    return $null -eq (Compare-Object -ReferenceObject $wanted -DifferenceObject $actual)
}

function Require-String([object] $value, [string] $where) {
    if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value)) {
        Fail "$where must be a non-empty string"
    }
}

function Find-BadKey([object] $value, [string] $where) {
    if ($null -eq $value) { return }
    if ($value -is [System.Collections.IEnumerable] -and $value -isnot [string]) {
        foreach ($item in $value) { Find-BadKey $item $where }
        return
    }
    $properties = $value.PSObject.Properties
    if ($null -eq $properties) { return }
    foreach ($property in $properties) {
        $key = $property.Name.ToLowerInvariant()
        if ($key -in @('provider', 'provider_id', 'model', 'model_name', 'model_provider', 'vendor', 'runtime_provider', 'runtime_identity')) {
            Fail "forbidden identity field '$($property.Name)' at $where"
        }
        Find-BadKey $property.Value "$where.$($property.Name)"
    }
}

$workspacePath = Resolve-Path -LiteralPath $Workspace -ErrorAction SilentlyContinue
if ($null -eq $workspacePath -or -not (Test-Path -LiteralPath $workspacePath.Path -PathType Container)) {
    Fail 'workspace does not exist'
}

$items = @(Get-ChildItem -LiteralPath $workspacePath.Path -Force)
$expectedFiles = @('authority_candidate.json', 'authority_review.md')
$actualFiles = @($items | Where-Object { -not $_.PSIsContainer } | Select-Object -ExpandProperty Name)
if ($items.Count -ne 2 -or $actualFiles.Count -ne 2 -or (($actualFiles | Sort-Object) -join '|') -cne (($expectedFiles | Sort-Object) -join '|')) {
    Fail 'workspace must contain exactly authority_candidate.json and authority_review.md'
}

$candidatePath = Join-Path $workspacePath.Path 'authority_candidate.json'
$reviewPath = Join-Path $workspacePath.Path 'authority_review.md'
try { $candidate = Get-Content -Raw -LiteralPath $candidatePath | ConvertFrom-Json }
catch { Fail "candidate JSON is invalid: $($_.Exception.Message)" }

if (-not (Has-ExactKeys $candidate @('artifact_type', 'program_id', 'role_id', 'subjects', 'claims', 'unresolved_questions', 'sources_consulted'))) {
    Fail 'candidate top-level keys are not canonical'
}
if ($candidate.artifact_type -cne 'AUTHORITY_CANDIDATE') { Fail 'artifact_type must equal AUTHORITY_CANDIDATE' }
if ($candidate.program_id -cne $ProgramId) { Fail "program_id does not match $ProgramId" }
if ($candidate.role_id -cne $Role) { Fail "role_id does not match $Role" }
Find-BadKey $candidate '$'

$claims = @($candidate.claims)
if ($claims.Count -eq 0) { Fail 'claims must be non-empty' }
$claimIds = @{}
$allowedStatuses = @('AUTHORITATIVE', 'PROVISIONAL', 'UNRESOLVED')
foreach ($claim in $claims) {
    if (-not (Has-ExactKeys $claim @('claim_id', 'subject', 'topic', 'claim', 'status', 'authority_evidence', 'counterevidence', 'future_information_risk', 'dependency_implications', 'notes'))) {
        Fail 'claim has non-canonical keys'
    }
    Require-String $claim.claim_id 'claim_id'
    Require-String $claim.subject 'claim subject'
    Require-String $claim.topic 'claim topic'
    Require-String $claim.claim 'claim text'
    if ($claimIds.ContainsKey($claim.claim_id)) { Fail "duplicate claim_id $($claim.claim_id)" }
    $claimIds[$claim.claim_id] = $true
    if ($claim.status -cnotin $allowedStatuses) { Fail "invalid claim status $($claim.status)" }
    $evidence = @($claim.authority_evidence)
    if ($claim.status -ceq 'AUTHORITATIVE' -and $evidence.Count -eq 0) { Fail "AUTHORITATIVE claim $($claim.claim_id) has empty authority_evidence" }
    foreach ($item in $evidence) {
        if (-not (Has-ExactKeys $item @('source_id', 'relative_path', 'locator', 'evidence_type', 'supports'))) {
            Fail "claim $($claim.claim_id) has non-canonical authority_evidence keys"
        }
        Require-String $item.source_id 'evidence source_id'
        Require-String $item.relative_path 'evidence relative_path'
        Require-String $item.locator 'evidence locator'
        Require-String $item.evidence_type 'evidence_type'
        Require-String $item.supports 'evidence supports'
    }
}

foreach ($source in @($candidate.sources_consulted)) {
    if (-not (Has-ExactKeys $source @('source_id', 'relative_path', 'locator'))) {
        Fail 'sources_consulted entry has non-canonical keys'
    }
    Require-String $source.source_id 'source_id'
    Require-String $source.relative_path 'source relative_path'
    Require-String $source.locator 'source locator'
}

$review = Get-Content -Raw -LiteralPath $reviewPath
if ([string]::IsNullOrWhiteSpace($review)) { Fail 'authority_review.md is empty' }
if ($review -match '(?i)\b(GLM|Codex|Gemini|Claude|ZCode|OpenAI)\b') { Fail 'authority_review.md contains a runtime identity name' }
if ($review -match '(?i)\b(peer|other researcher|peer report|peer-report)\b') { Fail 'authority_review.md contains a peer-report reference' }

Write-Output "REPORT_GATE_PASS $ProgramId/$Role claims=$($claims.Count)"
exit 0
