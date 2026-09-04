[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $ProgramId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string] $Role,
    [Parameter(Mandatory)][string] $SubmissionPath,
    [string] $BaseRef = 'origin/infra/i04-wave1-authority-bundles',
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
if ($candidate.program_id -ne $ProgramId -or $candidate.role_id -ne $Role) { throw 'Submission identity does not match ProgramId/Role.' }
if (-not $candidate.claims) { throw 'Submission has no claims.' }
$forbidden = '(?i)(provider|model|vendor|\bglm\b|\bcodex\b|\bgemini\b|\bclaude\b)'
foreach ($file in $files) { if ([regex]::IsMatch((Get-Content -Raw -LiteralPath $file.FullName), $forbidden)) { throw "Forbidden operational identity term in $($file.Name)." } }

$branch = "reports/$($ProgramId.ToLowerInvariant())-$($Role.ToLowerInvariant())-authority"
$worktree = Join-Path ([IO.Path]::GetTempPath()) ("trinityr-publish-" + [guid]::NewGuid().ToString('N'))
$added = $false
try {
    & git -C $repo worktree add -b $branch $worktree $BaseRef
    if ($LASTEXITCODE -ne 0) { throw 'Unable to create publication worktree.' }
    $added = $true
    $destination = Join-Path $worktree "research_submissions\$ProgramId\$Role"
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    Copy-Item -LiteralPath (Join-Path $source 'authority_candidate.json'),(Join-Path $source 'authority_review.md') -Destination $destination
    & git -C $worktree add -- research_submissions/$ProgramId/$Role/authority_candidate.json research_submissions/$ProgramId/$Role/authority_review.md
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
