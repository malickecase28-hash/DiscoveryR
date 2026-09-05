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
    if ($null -eq $value -or $value -is [string] -or $value -is [System.ValueType]) { return $false }
    $actual = @(Keys $value | Sort-Object)
    $wanted = @($expected | Sort-Object)
    return $null -eq (Compare-Object -ReferenceObject $wanted -DifferenceObject $actual)
}

function Require-String([object] $value, [string] $where) {
    if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value)) {
        Fail "$where must be a non-empty string"
    }
}

function Require-StringValue([object] $value, [string] $where) {
    if ($value -isnot [string]) { Fail "$where must be a string" }
}

function Require-Array([object] $value, [string] $where) {
    if ($value -isnot [System.Array]) { Fail "$where must be a JSON array" }
}

function Require-StringArray([object] $value, [string] $where, [bool] $RequireNonEmpty = $false) {
    Require-Array $value $where
    if ($RequireNonEmpty -and $value.Count -eq 0) { Fail "$where must be non-empty" }
    foreach ($item in $value) { Require-String $item "$where item" }
}

function Require-SafeRelativePath([object] $value, [string] $where) {
    Require-String $value $where
    $path = [string] $value
    if ($path -eq '.' -or [IO.Path]::IsPathRooted($path) -or $path -match '^[A-Za-z]:' -or $path -match '(^|[\\/])\.\.([\\/]|$)') {
        Fail "$where must be a safe relative path"
    }
}

function Require-SourceId([object] $value, [string] $where) {
    Require-String $value $where
    if ([string] $value -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
        Fail "$where has invalid source_id syntax"
    }
}

function Find-BadKey([object] $value, [string] $where) {
    if ($null -eq $value) { return }
    if ($value -is [PSCustomObject]) {
        foreach ($property in $value.PSObject.Properties) {
            $key = $property.Name.ToLowerInvariant()
            if ($key -in @('provider', 'provider_id', 'provider_identity', 'model_id', 'model_name', 'model_provider', 'model_identity', 'vendor', 'runtime_provider', 'runtime_identity', 'runtime_slot', 'credential', 'credential_path', 'secret', 'token')) {
                Fail "forbidden identity field '$($property.Name)' at $where"
            }
            Find-BadKey $property.Value "$where.$($property.Name)"
        }
        return
    }
    if ($value -is [System.Collections.IEnumerable] -and $value -isnot [string]) {
        foreach ($item in $value) { Find-BadKey $item $where }
    }
}

function Validate-Evidence([object] $item, [string] $where) {
    if (-not (Has-ExactKeys $item @('source_id', 'relative_path', 'locator', 'evidence_type', 'supports'))) {
        Fail "$where has non-canonical authority_evidence keys"
    }
    Require-SourceId $item.source_id "$where source_id"
    Require-SafeRelativePath $item.relative_path "$where relative_path"
    Require-String $item.locator "$where locator"
    Require-String $item.evidence_type "$where evidence_type"
    Require-String $item.supports "$where supports"
}

function Validate-Source([object] $source, [string] $where) {
    if (-not (Has-ExactKeys $source @('source_id', 'relative_path', 'locator'))) {
        Fail "$where has non-canonical keys"
    }
    Require-SourceId $source.source_id "$where source_id"
    Require-SafeRelativePath $source.relative_path "$where relative_path"
    Require-String $source.locator "$where locator"
}

if ($ProgramId -cnotmatch '^[A-Z]{2,8}-[0-9]{3}$') { Fail 'ProgramId format is invalid' }
if ($Role -cnotmatch '^A-[0-9]{2}$') { Fail 'Role format is invalid' }

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
$candidateRaw = Get-Content -Raw -LiteralPath $candidatePath
try { $candidate = $candidateRaw | ConvertFrom-Json }
catch { Fail "candidate JSON is invalid: $($_.Exception.Message)" }

if (-not (Has-ExactKeys $candidate @('artifact_type', 'program_id', 'role_id', 'subjects', 'claims', 'unresolved_questions', 'sources_consulted'))) {
    Fail 'candidate top-level keys are not canonical'
}
if ($candidate.artifact_type -cne 'AUTHORITY_CANDIDATE') { Fail 'artifact_type must equal AUTHORITY_CANDIDATE' }
if ($candidate.program_id -cne $ProgramId) { Fail "program_id does not match $ProgramId" }
if ($candidate.role_id -cne $Role) { Fail "role_id does not match $Role" }
Find-BadKey $candidate '$'
if ($candidateRaw -match '(?i)\b(GLM|Codex|Gemini|Claude|ZCode|OpenAI)\b') { Fail 'candidate contains a runtime identity name' }
if ($candidateRaw -match '(?i)\b(peer report|peer-report|other researcher(?:''s)? report)\b') { Fail 'candidate contains a peer-report reference' }

Require-StringArray $candidate.subjects 'subjects' $true
Require-Array $candidate.claims 'claims'
Require-StringArray $candidate.unresolved_questions 'unresolved_questions'
Require-Array $candidate.sources_consulted 'sources_consulted'

$claims = @($candidate.claims)
if ($claims.Count -eq 0) { Fail 'claims must be non-empty' }
$claimIds = @{}
$allowedStatuses = @('AUTHORITATIVE', 'PROVISIONAL', 'UNRESOLVED')
$claimPrefix = (($ProgramId -replace '-', '') + '-' + ($Role -replace '-', '') + '-C')
$claimPattern = '^' + [regex]::Escape($claimPrefix) + '[0-9]{3,}$'

foreach ($claim in $claims) {
    if (-not (Has-ExactKeys $claim @('claim_id', 'subject', 'topic', 'claim', 'status', 'authority_evidence', 'counterevidence', 'future_information_risk', 'dependency_implications', 'notes'))) {
        Fail 'claim has non-canonical keys'
    }
    Require-String $claim.claim_id 'claim_id'
    if ($claim.claim_id -cnotmatch $claimPattern) { Fail "claim_id $($claim.claim_id) does not match $claimPrefix###" }
    Require-String $claim.subject 'claim subject'
    Require-String $claim.topic 'claim topic'
    Require-String $claim.claim 'claim text'
    if ($claimIds.ContainsKey($claim.claim_id)) { Fail "duplicate claim_id $($claim.claim_id)" }
    $claimIds[$claim.claim_id] = $true
    if ($claim.status -cnotin $allowedStatuses) { Fail "invalid claim status $($claim.status)" }

    Require-Array $claim.authority_evidence "authority_evidence for $($claim.claim_id)"
    $evidence = @($claim.authority_evidence)
    if ($claim.status -cin @('AUTHORITATIVE', 'PROVISIONAL') -and $evidence.Count -eq 0) {
        Fail "$($claim.status) claim $($claim.claim_id) has empty authority_evidence"
    }
    foreach ($item in $evidence) { Validate-Evidence $item "claim $($claim.claim_id)" }

    Require-Array $claim.counterevidence "counterevidence for $($claim.claim_id)"
    Require-String $claim.future_information_risk "future_information_risk for $($claim.claim_id)"
    Require-Array $claim.dependency_implications "dependency_implications for $($claim.claim_id)"
    Require-StringValue $claim.notes "notes for $($claim.claim_id)"
}

foreach ($source in @($candidate.sources_consulted)) {
    Validate-Source $source 'sources_consulted entry'
}

$review = Get-Content -Raw -LiteralPath $reviewPath
if ([string]::IsNullOrWhiteSpace($review)) { Fail 'authority_review.md is empty' }
if ($review -match '(?i)\b(GLM|Codex|Gemini|Claude|ZCode|OpenAI)\b') { Fail 'authority_review.md contains a runtime identity name' }
if ($review -match '(?i)\b(peer report|peer-report|other researcher(?:''s)? report)\b') { Fail 'authority_review.md contains a peer-report reference' }

Write-Output "REPORT_GATE_PASS $ProgramId/$Role claims=$($claims.Count)"
exit 0
