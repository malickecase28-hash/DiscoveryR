[CmdletBinding()]
param(
 [string]$SourceRoot='F:\TrinityR-research\schemas',
 [string]$SpecPath='',
 [string]$OutputRoot='F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1',
 [string]$RegistryPath='',
 [switch]$UpdateRegistry,[switch]$RunSyntheticTests
)
$ErrorActionPreference='Stop'
$scriptRoot=Split-Path -Parent $MyInvocation.MyCommand.Path
if ([string]::IsNullOrWhiteSpace($SpecPath)) {$SpecPath=Join-Path $scriptRoot '..\authority_sources\XAUUSD.wave1.spec.json'}
if ([string]::IsNullOrWhiteSpace($RegistryPath)) {$RegistryPath=Join-Path $scriptRoot '..\authority_sources\XAUUSD.wave1.json'}
$PayloadContractSource='F:\TrinityR-research\analytical_lake\fusion_markets\xauusd\payload_manifest.json'
$PinnedPayloadSha256='78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed'
function Sha([byte[]]$Bytes) {([BitConverter]::ToString(([Security.Cryptography.SHA256]::Create().ComputeHash($Bytes)))-replace '-','').ToLowerInvariant()}
function FileSha([string]$Path) {Sha ([IO.File]::ReadAllBytes($Path))}
function Write-Json([object]$Value,[string]$Path) {[IO.File]::WriteAllText($Path,(($Value|ConvertTo-Json -Depth 40)+[Environment]::NewLine),[Text.UTF8Encoding]::new($false))}
function Select-Json([object]$Document,[string]$Pointer) {
 $current=$Document;if ($Pointer -eq '') {return $current}
 foreach ($part in ($Pointer.TrimStart('/') -split '/')) {$name=$part.Replace('~1','/').Replace('~0','~');if ($current -is [array]) {if ($name -notmatch '^\d+$' -or [int]$name -ge $current.Count) {throw "SELECTOR_MISSING: $Pointer"};$current=$current[[int]$name]} elseif ($null -eq $current) {throw "SELECTOR_MISSING: $Pointer"} else {$property=$current.PSObject.Properties[$name];if ($null -eq $property) {throw "SELECTOR_MISSING: $Pointer"};$current=$property.Value}}
 return $current
}
function DirectoryHash([string]$Root) {
 $rootItem=Get-Item -LiteralPath $Root -Force -ErrorAction Stop;if ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) {throw "REPARSE_POINT_REJECTED: $Root"};if (-not $rootItem.PSIsContainer) {throw "NON_REGULAR_FILE_REJECTED: $Root"}
 $prefix=([IO.Path]::GetFullPath($Root)).TrimEnd('\')+'\';$records=[Collections.Generic.List[string]]::new()
 foreach ($item in Get-ChildItem -LiteralPath $Root -Recurse -Force) {if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {throw "REPARSE_POINT_REJECTED: $($item.FullName)"};if ($item.PSIsContainer) {continue};if ($item -isnot [IO.FileInfo]) {throw "NON_REGULAR_FILE_REJECTED: $($item.FullName)"};$relative=$item.FullName.Substring($prefix.Length).Replace('\','/');$records.Add($relative+([char]9)+(FileSha $item.FullName)+([char]10))}
 $records.Sort([StringComparer]::Ordinal);Sha ([Text.UTF8Encoding]::new($false).GetBytes(($records -join '')))
}
function New-Authority([object]$Bundle,[string]$Target) {
 $selected=[ordered]@{};$provenance=[Collections.Generic.List[object]]::new();$sources=[Collections.Generic.List[object]]::new()
 if ($null -ne $Bundle.source_file) {
  $sourcePath=if ($Bundle.source_kind -eq 'payload_contract') {$PayloadContractSource} else {Join-Path $SourceRoot $Bundle.source_file};if ([string]::IsNullOrWhiteSpace($sourcePath) -or -not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {throw "SOURCE_REQUIRED: $($Bundle.source_file)"}
  $sourceName=[IO.Path]::GetFileName($sourcePath);$sourceHash=FileSha $sourcePath;if ([string]::IsNullOrWhiteSpace($Bundle.source_sha256)) {throw "SOURCE_SHA_REQUIRED: $sourceName"};if ($sourceHash -ne $Bundle.source_sha256.ToLowerInvariant()) {throw "SOURCE_HASH_MISMATCH: $sourceName"};if ($Bundle.source_kind -eq 'payload_contract' -and $sourceHash -ne $PinnedPayloadSha256) {throw "PINNED_SOURCE_HASH_MISMATCH: $sourceName"}
  $sources.Add([ordered]@{source_id=$sourceName;source_sha256=$sourceHash});$document=[Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($sourcePath))|ConvertFrom-Json
  foreach ($entry in $Bundle.selectors.PSObject.Properties) {$selected[$entry.Name]=Select-Json $document $entry.Value;$record=[ordered]@{source_id=$sourceName;source_sha256=$sourceHash;json_pointer=$entry.Value};if ($Bundle.source_kind -eq 'payload_contract') {$record.contract_schema_version=Select-Json $document '/declared_tick_payload_contract/schema_version';$record.contract_source=Select-Json $document '/declared_tick_payload_contract/source'};$provenance.Add($record)}
 }
 $authority=[ordered]@{schema_version='trinity.authority.v2';program_id=$Bundle.program_id;subject_id=$Bundle.subject_id;instrument='XAUUSD';selected=$selected;provenance=$provenance;status=@($Bundle.status);unresolved=@($Bundle.unresolved)};Write-Json $authority (Join-Path $Target 'authority.json');@{sources=$sources;content_identity=FileSha (Join-Path $Target 'authority.json')}
}
function New-Manifest([object]$Bundle,[object]$Result) {[ordered]@{schema_version='trinity.authority-bundle-manifest.v2';program_id=$Bundle.program_id;subject_id=$Bundle.subject_id;instrument='XAUUSD';source_files=$Result.sources;extraction_selectors=$Bundle.selectors;generated_files=@([ordered]@{relative_path='authority.json';sha256=$Result.content_identity});bundle_content_identity=$Result.content_identity}}
function Generate-Bundle([object]$Bundle,[string]$Root) {
 $target=Join-Path $Root $Bundle.program_id;$parent=Split-Path $target -Parent;New-Item -ItemType Directory -Force $parent|Out-Null;$temp=Join-Path $parent ('.tmp-'+[guid]::NewGuid().ToString('N'));New-Item -ItemType Directory $temp|Out-Null
 try {$result=New-Authority $Bundle $temp;Write-Json (New-Manifest $Bundle $result) (Join-Path $temp 'bundle_manifest.json');$transport=DirectoryHash $temp;if (Test-Path -LiteralPath $target) {if ((DirectoryHash $target) -eq $transport) {Remove-Item -LiteralPath $temp -Recurse -Force;return "ALREADY_GENERATED $($Bundle.program_id) $transport"};throw "IMMUTABLE_CONFLICT: $target"};try {[IO.Directory]::Move($temp,$target)} catch {if (Test-Path -LiteralPath $target) {throw "IMMUTABLE_CONFLICT: $target"};throw};"GENERATED $($Bundle.program_id) $transport"} catch {if (Test-Path -LiteralPath $temp) {Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue};throw}
}
function Scan([string]$Root) {foreach ($file in Get-ChildItem -LiteralPath $Root -Recurse -File) {$body=Get-Content -Raw $file.FullName;foreach ($term in 'emitted','feature_records','parquet_samples','market_samples','checkpoint_scan','performance','replay_write_wall_seconds','account_id','broker credentials','secrets','confirmation') {if ($body -match ('"'+[regex]::Escape($term)+'"\s*:')) {throw "FORBIDDEN_CONTENT: $term in $($file.FullName)"}}}}
function Update-Registry([string]$Path,[string]$Root) {$registry=Get-Content -Raw $Path|ConvertFrom-Json;foreach ($source in $registry.sources) {$source.hash=DirectoryHash (Join-Path $Root (Split-Path $source.relative_path -Leaf))};Write-Json $registry $Path}
function Verify-Registry([string]$Path,[string]$Root) {$registry=Get-Content -Raw $Path|ConvertFrom-Json;foreach ($source in $registry.sources) {if ([string]::IsNullOrWhiteSpace($source.hash) -or $source.hash -cnotmatch '^[0-9a-f]{64}$') {throw "REGISTRY_HASH_MISSING: $($source.source_id)"};$expected=DirectoryHash (Join-Path $Root (Split-Path $source.relative_path -Leaf));if ($source.hash -cne $expected) {throw "REGISTRY_HASH_MISMATCH: $($source.source_id)"}}}
function Expect-Throw([scriptblock]$Action,[string]$Message='') {try {& $Action;throw 'EXPECTED_FAILURE_NOT_RAISED'} catch {if ($_.Exception.Message -eq 'EXPECTED_FAILURE_NOT_RAISED' -or ($Message -and $_.Exception.Message -notmatch [regex]::Escape($Message))) {throw}}}
function Assert([bool]$Condition,[string]$Message) {if (-not $Condition) {throw "ASSERT_FAILED: $Message"}}
if ($RunSyntheticTests) {
 $fixture=Join-Path ([IO.Path]::GetTempPath()) ('wave1-'+[guid]::NewGuid());New-Item -ItemType Directory $fixture|Out-Null
 try {
  $payload=@{declared_tick_payload_contract=@{schema_version='v1';source='native_detector_serialization_contract';detectors=@{drift_burst=@{state=@('state');completed=@('completed')};feed_health=@{path='feed'};micro_volatility=@{path='micro'};quote_arrival=@{path='arrival'};quote_dynamics=@{path='dynamics'};quote_pressure=@{path='pressure'};spread_state=@{path='spread'}}};emitted=@{count=9};market_samples=@(100);performance=@{seconds=1}}|ConvertTo-Json -Depth 20;$PayloadContractSource=Join-Path $fixture 'payload_manifest.json';[IO.File]::WriteAllText($PayloadContractSource,$payload,[Text.UTF8Encoding]::new($false));$fixtureHash=FileSha $PayloadContractSource;$PinnedPayloadSha256=$fixtureHash;$specText=(Get-Content -Raw $SpecPath).Replace('78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed',$fixtureHash);$spec=$specText|ConvertFrom-Json
  $missing=$spec.bundles[0].PSObject.Copy();$missing.source_sha256='';Expect-Throw {New-Authority $missing $fixture} 'SOURCE_SHA_REQUIRED';$wrong=$spec.bundles[0].PSObject.Copy();$wrong.source_sha256=('0'*64);Expect-Throw {New-Authority $wrong $fixture} 'SOURCE_HASH_MISMATCH';$null=Select-Json (@{a=$null}|ConvertTo-Json|ConvertFrom-Json) '/a';Expect-Throw {Select-Json (@{a=$null}|ConvertTo-Json|ConvertFrom-Json) '/a/b'} 'SELECTOR_MISSING'
  $testRoot=Join-Path $fixture 'out';New-Item -ItemType Directory $testRoot|Out-Null;foreach ($bundle in $spec.bundles) {Generate-Bundle $bundle $testRoot|Out-Null};Scan $testRoot
  foreach ($program in 'AP-001','TC-001') {$authority=Get-Content -Raw (Join-Path $testRoot "$program\authority.json");Assert ($authority -notmatch 'emitted|market_samples|performance') "$program leakage"}
  $manifest=Get-Content -Raw (Join-Path $testRoot 'AP-001\bundle_manifest.json');Assert ($manifest -notmatch 'directory_transport_hash') 'manifest transport field';Assert ($manifest -match 'bundle_content_identity') 'manifest content identity'
  $ap2=Get-Content -Raw (Join-Path $testRoot 'AP-002\authority.json')|ConvertFrom-Json;Assert (@($ap2.selected.PSObject.Properties).Count -eq 0 -and @($ap2.unresolved) -contains 'UNRESOLVED_PENDING_SOURCE_APPROVAL') 'AP-002 unresolved';$tc=Get-Content -Raw (Join-Path $testRoot 'TC-001\authority.json');Assert ($tc -match 'MEASUREMENT_SEMANTICS_REQUIRE_PRODUCER_AUTHORITY' -and $tc -match 'AVAILABILITY_SEMANTICS_REQUIRE_PRODUCER_AUTHORITY') 'TC-001 unresolved'
  $registry=Join-Path $fixture 'registry.json';Copy-Item $RegistryPath $registry;$blankRegistry=Get-Content -Raw $registry|ConvertFrom-Json;foreach ($source in $blankRegistry.sources) {$source.hash=''};Write-Json $blankRegistry $registry;Expect-Throw {Verify-Registry $registry $testRoot} 'REGISTRY_HASH_MISSING';Update-Registry $registry $testRoot;Verify-Registry $registry $testRoot;$populated=Get-Content -Raw $registry;$populatedObject=$populated|ConvertFrom-Json;$populatedObject.sources[0].hash='0';Write-Json $populatedObject $registry;Expect-Throw {Verify-Registry $registry $testRoot} 'REGISTRY_HASH_MISSING';$populatedObject.sources[0].hash=('0'*64);Write-Json $populatedObject $registry;Expect-Throw {Verify-Registry $registry $testRoot} 'REGISTRY_HASH_MISMATCH';Update-Registry $registry $testRoot;Verify-Registry $registry $testRoot
  $first=DirectoryHash (Join-Path $testRoot 'AP-001');Generate-Bundle $spec.bundles[0] $testRoot|Out-Null;Assert ($first -eq (DirectoryHash (Join-Path $testRoot 'AP-001'))) 'deterministic regeneration';$extra=Join-Path $testRoot 'AP-001\unexpected.txt';[IO.File]::WriteAllText($extra,'x');Assert ((DirectoryHash (Join-Path $testRoot 'AP-001')) -ne $first) 'added file hash';Remove-Item -LiteralPath $extra;Assert ((DirectoryHash (Join-Path $testRoot 'AP-001')) -eq $first) 'deleted file hash';[IO.File]::WriteAllText((Join-Path $testRoot 'AP-001\bundle_manifest.json'),'tampered');Expect-Throw {Generate-Bundle $spec.bundles[0] $testRoot} 'IMMUTABLE_CONFLICT'
  $reparseTarget=Join-Path $fixture 'reparse-target';New-Item -ItemType Directory $reparseTarget|Out-Null;$reparse=Join-Path $testRoot 'AP-001\reparse';New-Item -ItemType Junction -Path $reparse -Target $reparseTarget|Out-Null;Expect-Throw {DirectoryHash (Join-Path $testRoot 'AP-001')} 'REPARSE_POINT_REJECTED';Remove-Item -LiteralPath $reparse -Force
  'Synthetic tests: PASS'
 } finally {Remove-Item -LiteralPath $fixture -Recurse -Force -ErrorAction SilentlyContinue};exit 0
}
$spec=Get-Content -Raw $SpecPath|ConvertFrom-Json;New-Item -ItemType Directory -Force $OutputRoot|Out-Null;foreach ($bundle in $spec.bundles) {Generate-Bundle $bundle $OutputRoot};Scan $OutputRoot;if ($UpdateRegistry) {Update-Registry $RegistryPath $OutputRoot};Verify-Registry $RegistryPath $OutputRoot;if ($UpdateRegistry) {'Registry update: PASS'};'Registry verification: PASS'
