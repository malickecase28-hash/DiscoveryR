[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $SubmissionPath,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{40}$')][string] $BaseRef,
    [string] $ResearchBaseSha,
    [Parameter(Mandatory)][string] $InputManifestPath,
    [string] $PublicationBranch,
    [switch] $DryRun,
    [switch] $Push
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$source = (Resolve-Path -LiteralPath $SubmissionPath).Path
if ($source.StartsWith(([IO.Path]::GetFullPath($repo).TrimEnd('\') + '\'), [StringComparison]::OrdinalIgnoreCase)) { throw 'Submission must be outside the Git worktree.' }

$expected = @('authority_candidate.json', 'authority_review.md')
$files = @(Get-ChildItem -LiteralPath $source -Force)
if ($files.Count -ne $expected.Count -or @($files | Where-Object { $_.PSIsContainer -or $_.Name -notin $expected }).Count -or @($expected | Where-Object { -not (Test-Path -LiteralPath (Join-Path $source $_) -PathType Leaf) }).Count) { throw 'Submission must contain exactly authority_candidate.json and authority_review.md.' }
foreach ($file in $files) { if ($file.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point is not allowed: $($file.Name)" } }

$candidate = Get-Content -Raw -LiteralPath (Join-Path $source 'authority_candidate.json') | ConvertFrom-Json
if ($candidate.program_id -ne $ProgramId -or $candidate.role_id -ne $Role -or ($candidate.identity -and ($candidate.identity.program_id -ne $ProgramId -or $candidate.identity.role_id -ne $Role))) { throw 'Submission identity does not match ProgramId/Role.' }
if ($candidate.artifact_type -ne 'AUTHORITY_CANDIDATE' -or -not $candidate.claims -or @($candidate.claims).Count -eq 0) { throw 'Invalid authority artifact type or empty claims.' }
$assignmentPath=Join-Path $repo "agent_harness\assignments\$ProgramId\$Role.json"; $assignment=Get-Content -Raw $assignmentPath|ConvertFrom-Json
$registry=Get-Content -Raw (Join-Path $repo 'agent_harness\authority_sources\XAUUSD.json')|ConvertFrom-Json
$authorized=@($assignment.authority_source_ids) + "assignment.$($ProgramId.ToLowerInvariant())_$($Role.ToLowerInvariant())", 'protocol.research_rules'; $ids=@{}
foreach($claim in @($candidate.claims)) {
    if ($claim.claim_id -notmatch '^[A-Z]{2}\d{3}-A-?\d{2}-C\d{3,}$' -or $ids[$claim.claim_id]) { throw "Invalid or duplicate claim id: $($claim.claim_id)" }; $ids[$claim.claim_id]=$true
    if ($claim.status -notin @('AUTHORITATIVE','PROVISIONAL','UNRESOLVED') -or [string]::IsNullOrWhiteSpace([string]$claim.claim)) { throw "Invalid claim status or claim text: $($claim.claim_id)" }
    $evidence=$claim.authority_evidence; if ($claim.status -eq 'AUTHORITATIVE' -and ($null -eq $evidence -or @($evidence).Count -eq 0)) { throw "AUTHORITATIVE claim lacks authority_evidence: $($claim.claim_id)" }
    foreach($e in $evidence) { foreach($field in @('source_id','relative_path','locator','evidence_type','supports')) { if ([string]::IsNullOrWhiteSpace([string]$e.$field)) { throw "Malformed evidence: $($claim.claim_id)" } }; if ($e.source_id -notin $authorized -and $e.source_id -notin @($registry.sources.source_id)) { throw "Unauthorized evidence source: $($e.source_id)" } }
}
function RejectOperationalIdentity($Node,[string]$Path='artifact') { if($Node -is [PSCustomObject]) { foreach($p in $Node.psobject.Properties) { if($p.Name -match '^(provider|provider_id|provider_identity|model_id|model_provider|model_name|model_identity|runtime_provider|runtime_slot|runtime_config|vendor|credential|credential_path|secret|token)$'){throw "Operational identity field: $Path.$($p.Name)"}; RejectOperationalIdentity $p.Value "$Path.$($p.Name)" } } elseif($Node -is [Collections.IEnumerable] -and $Node -isnot [string]) { foreach($i in $Node){RejectOperationalIdentity $i $Path} } }
RejectOperationalIdentity $candidate
foreach ($file in $files) { if ([regex]::IsMatch((Get-Content -Raw -LiteralPath $file.FullName), '(?i)\b(glm|codex|gemini|claude)\b')) { throw "Explicit operational identity name in $($file.Name)." } }
$manifest=Get-Content -Raw -LiteralPath (Resolve-Path $InputManifestPath) | ConvertFrom-Json; foreach($field in 'schema_version','program_id','role_id','phase','research_base_sha','assignment_path','assignment_sha256','access_profile','raw_lake_access','repository_surfaces','authority_sources','shared_input_fingerprint'){$value=$manifest.PSObject.Properties[$field].Value;if($null -eq $value -or ($value -is [string] -and [string]::IsNullOrWhiteSpace($value)) -or ($value -is [Collections.IEnumerable] -and $value -isnot [string] -and @($value).Count -eq 0)){throw "Input manifest field missing: $field"}}; if($manifest.program_id -ne $ProgramId -or $manifest.role_id -ne $Role -or @($manifest.repository_surfaces|Where-Object{$null -eq $_.relative_path}).Count){throw 'Input manifest identity or surfaces invalid.'}; $assignmentHash=(Get-FileHash $assignmentPath -Algorithm SHA256).Hash.ToLowerInvariant(); if($manifest.assignment_sha256 -ne $assignmentHash){throw 'Input manifest assignment hash mismatch.'}; foreach($sourceId in @($assignment.authority_source_ids)){$s=@($registry.sources|Where-Object source_id -eq $sourceId)[0];foreach($field in 'bundle_content_identity','directory_transport_hash'){if($s.$field -notmatch '^[0-9a-fA-F]{64}$'){throw "Authority identity missing: $sourceId/$field"}}}; if ([string]::IsNullOrWhiteSpace($ResearchBaseSha)) { $ResearchBaseSha=$manifest.research_base_sha }; if ($ResearchBaseSha -notmatch '^[0-9a-fA-F]{40}$' -or $manifest.research_base_sha -ne $ResearchBaseSha.ToLowerInvariant()) { throw 'ResearchBaseSha or manifest-derived SHA is required and must match the manifest.' }
$baseCommit=(& git -C $repo rev-parse $BaseRef).Trim(); if ($baseCommit -ne $ResearchBaseSha.ToLowerInvariant()) { throw 'BaseRef does not resolve to immutable ResearchBaseSha.' }; $inputManifestHash=if($InputManifestPath){(Get-FileHash (Resolve-Path $InputManifestPath) -Algorithm SHA256).Hash.ToLowerInvariant()}
if ($DryRun) { [pscustomobject]@{Validated=$true;ProgramId=$ProgramId;Role=$Role;ResearchBaseSha=$ResearchBaseSha.ToLowerInvariant();InputManifestSha256=$inputManifestHash;ClaimCount=@($candidate.claims).Count} | Format-List; return }

$branch = if ([string]::IsNullOrWhiteSpace($PublicationBranch)) { "reports/$($ProgramId.ToLowerInvariant())-$($Role.ToLowerInvariant())-authority-v2" } else { if ($PublicationBranch -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]{0,127}$' -or $PublicationBranch -match '(^|/)\.\.($|/)' -or $PublicationBranch.EndsWith('/')) { throw 'Unsafe publication branch.' }; $PublicationBranch }
$worktree = Join-Path ([IO.Path]::GetTempPath()) ("trinityr-publish-" + [guid]::NewGuid().ToString('N'))
$added = $false
try {
    & git -C $repo worktree add -b $branch $worktree $BaseRef
    if ($LASTEXITCODE -ne 0) { throw 'Unable to create publication worktree.' }
    $added = $true
    $destination = Join-Path $worktree "research_submissions\$ProgramId\$Role"
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    Copy-Item -LiteralPath (Join-Path $source 'authority_candidate.json'),(Join-Path $source 'authority_review.md') -Destination $destination
    $authorityIdentities=@($assignment.authority_source_ids|ForEach-Object{$s=@($registry.sources|Where-Object source_id -eq $_)[0];[ordered]@{source_id=$s.source_id;bundle_content_identity=$s.bundle_content_identity;directory_transport_hash=$s.directory_transport_hash}})
    $submissionManifest=[ordered]@{schema_version='trinity.submission-manifest.v1';program_id=$ProgramId;role_id=$Role;research_base_sha=$ResearchBaseSha.ToLowerInvariant();research_input_manifest_sha256=$inputManifestHash;assignment_sha256=$assignmentHash;shared_input_fingerprint=$manifest.shared_input_fingerprint;authority_source_ids=@($assignment.authority_source_ids);authority_bundle_hashes=$authorityIdentities;authority_candidate_sha256=(Get-FileHash (Join-Path $source 'authority_candidate.json') -Algorithm SHA256).Hash.ToLowerInvariant();authority_review_sha256=(Get-FileHash (Join-Path $source 'authority_review.md') -Algorithm SHA256).Hash.ToLowerInvariant();published_utc=[DateTime]::UtcNow.ToString('o')}
    ($submissionManifest|ConvertTo-Json -Depth 20)+[Environment]::NewLine | Set-Content -NoNewline -Encoding utf8 (Join-Path $destination 'submission_manifest.json')
    & git -C $worktree add -- research_submissions/$ProgramId/$Role/authority_candidate.json research_submissions/$ProgramId/$Role/authority_review.md research_submissions/$ProgramId/$Role/submission_manifest.json
    & git -C $worktree diff --cached --check
    if ($LASTEXITCODE -ne 0) { throw 'Publication diff check failed.' }
    & git -C $worktree commit -m "Publish $ProgramId $Role authority submission"
    if ($LASTEXITCODE -ne 0) { throw 'Publication commit failed.' }
    if ($Push) {
        & git -C $worktree push -u origin $branch
        if ($LASTEXITCODE -ne 0) { throw 'Publication push failed.' }
    }
    [pscustomobject]@{ Branch = $branch; Commit = (& git -C $worktree rev-parse HEAD); Pushed = [bool]$Push; Path = "research_submissions/$ProgramId/$Role" } | Format-List
} finally {
    if ($added) { & git -C $repo worktree remove --force $worktree 2>$null }
    if (Test-Path -LiteralPath $worktree) { Remove-Item -LiteralPath $worktree -Recurse -Force -ErrorAction SilentlyContinue }
}
