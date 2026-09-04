[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ManifestRoot,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')][string]$ResearchBaseSha,
    [string]$OutputPath
)
$ErrorActionPreference='Stop'
if(-not $OutputPath){$OutputPath=Join-Path $ManifestRoot 'pair_input_parity.json'}
$pairs=[ordered]@{schema_version='trinity.pair-input-parity.v2';research_base_sha=$ResearchBaseSha.ToLowerInvariant();pairs=@()}
foreach($program in @('AP-001','AP-002','TC-001','BG-001')){
    $a=Get-Content -Raw (Join-Path $ManifestRoot "$program\A-01\research_input_manifest.json")|ConvertFrom-Json
    $b=Get-Content -Raw (Join-Path $ManifestRoot "$program\A-02\research_input_manifest.json")|ConvertFrom-Json
    if($a.research_base_sha-ne$pairs.research_base_sha-or$b.research_base_sha-ne$pairs.research_base_sha){throw "Manifest base mismatch: $program"}
    if($a.shared_input_fingerprint-ne$b.shared_input_fingerprint){throw "Pair fingerprint mismatch: $program"}
    $pairs.pairs += [ordered]@{program_id=$program;a01_shared_input_fingerprint=$a.shared_input_fingerprint;a02_shared_input_fingerprint=$b.shared_input_fingerprint;match=$true}
}
$json=($pairs|ConvertTo-Json -Depth 20)+[Environment]::NewLine
if(Test-Path -LiteralPath $OutputPath){if((Get-Content -Raw $OutputPath)-cne$json){throw 'Immutable pair parity conflict.'}}else{[IO.File]::WriteAllText($OutputPath,$json,[Text.UTF8Encoding]::new($false))}
$pairs|ConvertTo-Json -Depth 20
