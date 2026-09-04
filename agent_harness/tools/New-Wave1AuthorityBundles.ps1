[CmdletBinding()]
param(
    [string]$SourceRoot = 'F:\TrinityR-research\schemas',
    [string]$SpecPath = (Join-Path $PSScriptRoot '..\authority_sources\XAUUSD.wave1.spec.json'),
    [string]$OutputRoot = 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1',
    [string]$PayloadContractSource = $env:TRINITYR_PAYLOAD_CONTRACT_SOURCE,
    [string]$RegistryPath = (Join-Path $PSScriptRoot '..\authority_sources\XAUUSD.wave1.json'),
    [switch]$UpdateRegistry,
    [switch]$RunSyntheticTests
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($PayloadContractSource)) { $PayloadContractSource = Join-Path (Split-Path $SourceRoot -Parent) 'payload_manifest(20260904-012732).json' }
function Sha([byte[]]$Bytes) { ([BitConverter]::ToString(([Security.Cryptography.SHA256]::Create().ComputeHash($Bytes))) -replace '-', '').ToLowerInvariant() }
function FileSha([string]$Path) { Sha ([IO.File]::ReadAllBytes($Path)) }
function Write-Json([object]$Value, [string]$Path) { [IO.File]::WriteAllText($Path, (($Value | ConvertTo-Json -Depth 40) + "`n"), [Text.UTF8Encoding]::new($false)) }
function Select-Json([object]$Document, [string]$Pointer) {
    $current = $Document; if ($Pointer -eq '') { return $current }
    foreach ($part in ($Pointer.TrimStart('/') -split '/')) {
        $name = $part.Replace('~1', '/').Replace('~0', '~')
        if ($current -is [array]) { if ($name -notmatch '^\d+$' -or [int]$name -ge $current.Count) { throw "SELECTOR_MISSING: $Pointer" }; $current = $current[[int]$name] }
        else { $property = $current.PSObject.Properties[$name]; if ($null -eq $property) { throw "SELECTOR_MISSING: $Pointer" }; $current = $property.Value }
    }; return $current
}
# bundle_manifest.json is excluded to avoid a self-referential hash.
function DirectoryHash([string]$Root) {
    $rootItem=Get-Item -LiteralPath $Root -Force -ErrorAction Stop; if ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "REPARSE_POINT_REJECTED: $Root" }
    $records = [Collections.Generic.List[string]]::new()
    $prefix = ([IO.Path]::GetFullPath($Root)).TrimEnd('\') + '\'
    foreach ($item in Get-ChildItem -LiteralPath $Root -Recurse -Force) { if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "REPARSE_POINT_REJECTED: $($item.FullName)" }; if ($item.PSIsContainer -or $item.Name -eq 'bundle_manifest.json') { continue }; $relative=$item.FullName.Substring($prefix.Length).Replace('\','/'); $records.Add("$relative`t$(FileSha $item.FullName)`n") }
    $records.Sort([StringComparer]::Ordinal); Sha ([Text.UTF8Encoding]::new($false).GetBytes(($records -join '')))
}
function New-Authority([object]$Bundle, [string]$Target) {
    $selected=[ordered]@{}; $provenance=[Collections.Generic.List[object]]::new(); $sources=[Collections.Generic.List[object]]::new()
    if ($null -ne $Bundle.source_file) {
        $sourcePath=if ($Bundle.source_kind -eq 'payload_contract') {$PayloadContractSource} else {Join-Path $SourceRoot $Bundle.source_file}; if ([string]::IsNullOrWhiteSpace($sourcePath) -or -not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) { throw 'SOURCE_REQUIRED: payload_manifest(20260904-012732).json' }
        $sourceName=if ($Bundle.source_kind -eq 'payload_contract') {'payload_manifest(20260904-012732).json'} else {$Bundle.source_file}; $sourceHash=FileSha $sourcePath; $sources.Add([ordered]@{source_id=$sourceName;source_sha256=$sourceHash}); $document=[Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($sourcePath))|ConvertFrom-Json
        foreach ($entry in $Bundle.selectors.PSObject.Properties) { $selected[$entry.Name]=Select-Json $document $entry.Value; $record=[ordered]@{source_id=$sourceName;source_sha256=$sourceHash;json_pointer=$entry.Value}; if ($Bundle.source_kind -eq 'payload_contract') { $record.contract_schema_version=Select-Json $document '/declared_tick_payload_contract/schema_version'; $record.contract_source=Select-Json $document '/declared_tick_payload_contract/source' }; $provenance.Add($record) }
    }
    $authority=[ordered]@{schema_version='trinity.authority.v2';program_id=$Bundle.program_id;subject_id=$Bundle.subject_id;instrument='XAUUSD';selected=$selected;provenance=$provenance;status=@($Bundle.status);unresolved=@($Bundle.unresolved)}; Write-Json $authority (Join-Path $Target 'authority.json'); @{sources=$sources;content_identity=FileSha (Join-Path $Target 'authority.json')}
}
function Generate-Bundle([object]$Bundle, [string]$Root) {
    $target=Join-Path $Root $Bundle.program_id
    $parent=Split-Path $target -Parent; New-Item -ItemType Directory -Force $parent|Out-Null; $temp=Join-Path $parent ('.tmp-'+[guid]::NewGuid().ToString('N')); New-Item -ItemType Directory $temp|Out-Null
    try {
        $result=New-Authority $Bundle $temp; $manifest=[ordered]@{schema_version='trinity.authority-bundle-manifest.v2';program_id=$Bundle.program_id;subject_id=$Bundle.subject_id;instrument='XAUUSD';source_files=$result.sources;extraction_selectors=$Bundle.selectors;generated_files=@([ordered]@{relative_path='authority.json';sha256=$result.content_identity});bundle_content_identity=$result.content_identity;directory_transport_hash=''}; Write-Json $manifest (Join-Path $temp 'bundle_manifest.json'); $manifest.directory_transport_hash=DirectoryHash $temp; Write-Json $manifest (Join-Path $temp 'bundle_manifest.json')
        if (Test-Path -LiteralPath $target) { if ((DirectoryHash $target) -eq $manifest.directory_transport_hash) { Remove-Item -LiteralPath $temp -Recurse -Force; return "ALREADY_GENERATED $($Bundle.program_id) $($manifest.directory_transport_hash)" }; throw "IMMUTABLE_CONFLICT: $target" }
        try { [IO.Directory]::Move($temp,$target) } catch { if (Test-Path -LiteralPath $target) { throw "IMMUTABLE_CONFLICT: $target" }; throw }; "GENERATED $($Bundle.program_id) $($manifest.directory_transport_hash)"
    } catch { if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue }; throw }
}
function Scan([string]$Root) { foreach ($file in Get-ChildItem -LiteralPath $Root -Recurse -File) { $body=Get-Content -Raw $file.FullName; foreach ($term in 'emitted','feature_records','parquet_samples','market_samples','checkpoint_scan','performance','replay_write_wall_seconds','account_id','broker credentials','secrets','confirmation') { if ($body -match [regex]::Escape($term)) { throw "FORBIDDEN_CONTENT: $term in $($file.FullName)" } } } }
function Update-Registry([string]$Path, [string]$Root) { $registry=Get-Content -Raw $Path|ConvertFrom-Json; foreach ($source in $registry.sources) { $program=Split-Path $source.relative_path -Leaf; $manifest=Get-Content -Raw (Join-Path $Root "$program\bundle_manifest.json")|ConvertFrom-Json; $source.hash=$manifest.directory_transport_hash }; Write-Json $registry $Path }
function Verify-Registry([string]$Path, [string]$Root) { $registry=Get-Content -Raw $Path|ConvertFrom-Json; foreach ($source in $registry.sources) { $program=Split-Path $source.relative_path -Leaf; $expected=DirectoryHash (Join-Path $Root $program); if ($source.hash -ne $expected) { throw "REGISTRY_HASH_MISMATCH: $($source.source_id)" } } }
function Expect-Throw([scriptblock]$Action, [string]$Message = '') { try { & $Action; throw 'EXPECTED_FAILURE_NOT_RAISED' } catch { if ($_.Exception.Message -eq 'EXPECTED_FAILURE_NOT_RAISED') { throw }; if ($Message -and $_.Exception.Message -notmatch [regex]::Escape($Message)) { throw } } }
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw "ASSERT_FAILED: $Message" } }
if ($RunSyntheticTests) {
    $fixture=Join-Path ([IO.Path]::GetTempPath()) ('wave1-'+[guid]::NewGuid()); New-Item -ItemType Directory $fixture|Out-Null
    try {
        $payload=@{declared_tick_payload_contract=@{schema_version='v1';source='native_detector_serialization_contract';detectors=@{drift_burst=@{state=@('state');completed=@('completed')};feed_health=@{path='feed'};micro_volatility=@{path='micro'};quote_arrival=@{path='arrival'};quote_dynamics=@{path='dynamics'};quote_pressure=@{path='pressure'};spread_state=@{path='spread'}}};emitted=@{count=9};market_samples=@(100);performance=@{seconds=1}}|ConvertTo-Json -Depth 20
        $PayloadContractSource=Join-Path $fixture 'payload_manifest(20260904-012732).json'; [IO.File]::WriteAllText($PayloadContractSource,$payload,[Text.UTF8Encoding]::new($false)); $testRoot=Join-Path $fixture 'out'; $spec=Get-Content -Raw $SpecPath|ConvertFrom-Json
        $null=Select-Json (@{a=$null}|ConvertTo-Json|ConvertFrom-Json) '/a'; Expect-Throw { Select-Json (@{a=1}|ConvertTo-Json|ConvertFrom-Json) '/missing' } 'SELECTOR_MISSING'
        New-Item -ItemType Directory $testRoot|Out-Null; foreach ($bundle in $spec.bundles) { Generate-Bundle $bundle $testRoot|Out-Null }; Scan $testRoot
        $ap=Get-Content -Raw (Join-Path $testRoot 'AP-001\authority.json'); $tc=Get-Content -Raw (Join-Path $testRoot 'TC-001\authority.json'); Assert ($ap -match 'DECLARED_SERIALIZATION_CONTRACT_AVAILABLE') 'AP-001 status'; Assert ($ap -match 'drift_burst') 'AP-001 extraction'; Assert ($tc -match 'feed_health' -and $tc -match 'spread_state') 'TC-001 extraction'; Assert ($ap -notmatch 'emitted|market_samples|performance') 'AP-001 leak'; Assert ($tc -notmatch 'emitted|market_samples|performance') 'TC-001 leak'
        $first=DirectoryHash (Join-Path $testRoot 'AP-001'); Generate-Bundle $spec.bundles[0] $testRoot|Out-Null; Assert ($first -eq (DirectoryHash (Join-Path $testRoot 'AP-001'))) 'deterministic regeneration'
        [IO.File]::WriteAllText((Join-Path $testRoot 'AP-001\unexpected.txt'),'x'); Assert ((DirectoryHash (Join-Path $testRoot 'AP-001')) -ne $first) 'added file hash'; Expect-Throw { Generate-Bundle $spec.bundles[0] $testRoot } 'IMMUTABLE_CONFLICT'; [IO.File]::WriteAllText((Join-Path $testRoot 'AP-002\authority.json'),'changed'); Assert ((DirectoryHash (Join-Path $testRoot 'AP-002')) -ne (Get-Content -Raw (Join-Path $testRoot 'AP-002\bundle_manifest.json')|ConvertFrom-Json).directory_transport_hash) 'changed byte hash'
        $link=Join-Path $fixture 'link'; New-Item -ItemType SymbolicLink -Path $link -Target $PayloadContractSource -ErrorAction SilentlyContinue|Out-Null; if (Test-Path -LiteralPath $link) { Expect-Throw { DirectoryHash $fixture } 'REPARSE_POINT_REJECTED' }
        'Synthetic tests: PASS'
    } finally { Remove-Item -LiteralPath $fixture -Recurse -Force -ErrorAction SilentlyContinue }; exit 0
}
$spec=Get-Content -Raw $SpecPath|ConvertFrom-Json; New-Item -ItemType Directory -Force $OutputRoot|Out-Null; foreach ($bundle in $spec.bundles) { Generate-Bundle $bundle $OutputRoot }; Scan $OutputRoot; if ($UpdateRegistry) { Update-Registry $RegistryPath $OutputRoot; Verify-Registry $RegistryPath $OutputRoot; 'Registry update: PASS' }; 'Forbidden-content scan: PASS'
