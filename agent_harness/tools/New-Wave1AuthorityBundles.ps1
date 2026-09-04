[CmdletBinding()]
param(
    [string]$SourceRoot = 'F:\TrinityR-research\schemas',
    [string]$PayloadContractSource = 'F:\TrinityR-research\analytical_lake\fusion_markets\xauusd\payload_manifest.json',
    [string]$SpecPath,
    [string]$AuthorityRoot = 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V3',
    [string]$RegistryPath,
    [switch]$UpdateRegistry,
    [switch]$VerifyOnly
)

$ErrorActionPreference = 'Stop'
$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
if ([string]::IsNullOrWhiteSpace($SpecPath)) { $SpecPath = Join-Path $scriptRoot '..\authority_sources\XAUUSD.wave1.spec.json' }
if ([string]::IsNullOrWhiteSpace($RegistryPath)) { $RegistryPath = Join-Path $scriptRoot '..\authority_sources\XAUUSD.json' }

function Sha([byte[]]$Bytes) { (([Security.Cryptography.SHA256]::Create().ComputeHash($Bytes)) | ForEach-Object ToString x2) -join '' }
function FileSha([string]$Path) { Sha ([IO.File]::ReadAllBytes($Path)) }
function WriteJson($Value, [string]$Path) { [IO.File]::WriteAllText($Path, (($Value | ConvertTo-Json -Depth 40) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false)) }
function SelectJson($Document, [string]$Pointer) {
    $current = $Document
    foreach ($part in $Pointer.TrimStart('/') -split '/') {
        $name = $part.Replace('~1', '/').Replace('~0', '~')
        if ($current -is [array]) { if ($name -notmatch '^\d+$' -or [int]$name -ge $current.Count) { throw "SELECTOR_MISSING: $Pointer" }; $current = $current[[int]$name] }
        else { $property = $current.PSObject.Properties[$name]; if ($null -eq $property) { throw "SELECTOR_MISSING: $Pointer" }; $current = $property.Value }
    }
    $current
}
function DirectoryHash([string]$BundleRoot) {
    $rootItem = Get-Item -LiteralPath $BundleRoot -Force
    if (-not (Test-Path -LiteralPath $BundleRoot -PathType Container) -or ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "INVALID_BUNDLE_ROOT: $BundleRoot" }
    $prefix = [IO.Path]::GetFullPath($BundleRoot).TrimEnd('\') + '\'
    $records = @()
    foreach ($item in Get-ChildItem -LiteralPath $BundleRoot -Recurse -Force) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "REPARSE_POINT_REJECTED: $($item.FullName)" }
        if (-not $item.PSIsContainer -and $item.Name -ne 'bundle_manifest.json') {
            $relative = $item.FullName.Substring($prefix.Length).Replace('\', '/')
            $records += "$relative`t$(FileSha $item.FullName)`n"
        }
    }
    $records = @($records | Sort-Object)
    Sha ([Text.UTF8Encoding]::new($false).GetBytes(($records -join '')))
}
function NewAuthority($Bundle, [string]$Target) {
    $selected = [ordered]@{}
    $provenance = [Collections.Generic.List[object]]::new()
    $sources = [Collections.Generic.List[object]]::new()
    if ($null -ne $Bundle.source_file) {
        $sourcePath = if ($Bundle.source_kind -eq 'payload_contract') { $PayloadContractSource } else { Join-Path $SourceRoot $Bundle.source_file }
        if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) { throw "SOURCE_REQUIRED: $sourcePath" }
        $sourceHash = FileSha $sourcePath
        if ($Bundle.source_sha256 -and $sourceHash -ne $Bundle.source_sha256.ToLowerInvariant()) { throw "SOURCE_HASH_MISMATCH: $sourcePath" }
        if ($Bundle.source_kind -eq 'payload_contract' -and $sourceHash -ne '78d5fc20758831e06d487217d0367bbaab5f25cb7eefcac8bee9446f6eea3eed') { throw "PINNED_SOURCE_HASH_MISMATCH: $sourcePath" }
        $sourceName = [IO.Path]::GetFileName($sourcePath)
        $sources.Add([ordered]@{ source_id = $sourceName; source_sha256 = $sourceHash })
        $document = ([Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($sourcePath))) | ConvertFrom-Json
        foreach ($entry in $Bundle.selectors.PSObject.Properties) {
            $selected[$entry.Name] = SelectJson $document $entry.Value
            $record = [ordered]@{ source_id = $sourceName; source_sha256 = $sourceHash; json_pointer = $entry.Value }
            if ($Bundle.source_kind -eq 'payload_contract') {
                $record.contract_schema_version = SelectJson $document '/declared_tick_payload_contract/schema_version'
                $record.contract_source = SelectJson $document '/declared_tick_payload_contract/source'
            }
            $provenance.Add($record)
        }
    }
    $authority = [ordered]@{ schema_version = 'trinity.authority.v3'; program_id = $Bundle.program_id; subject_id = $Bundle.subject_id; instrument = 'XAUUSD'; selected = $selected; provenance = $provenance; status = @($Bundle.status); unresolved = @($Bundle.unresolved) }
    WriteJson $authority (Join-Path $Target 'authority.json')
    [pscustomobject]@{ content_identity = FileSha (Join-Path $Target 'authority.json'); sources = $sources }
}
function GenerateBundle($Bundle) {
    if (-not (Test-Path -LiteralPath $AuthorityRoot)) { New-Item -ItemType Directory -Force -Path $AuthorityRoot | Out-Null }
    $parent = (Resolve-Path $AuthorityRoot).Path
    $staging = Join-Path $parent ('.staging-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $staging | Out-Null
    try {
        $result = NewAuthority $Bundle $staging
        $transport = DirectoryHash $staging
        $manifest = [ordered]@{ schema_version = 'trinity.authority-bundle-manifest.v3'; program_id = $Bundle.program_id; subject_id = $Bundle.subject_id; instrument = 'XAUUSD'; source_files = $result.sources; extraction_selectors = $Bundle.selectors; generated_files = @([ordered]@{ relative_path = 'authority.json'; sha256 = $result.content_identity }); bundle_content_identity = $result.content_identity; directory_transport_hash = $transport }
        WriteJson $manifest (Join-Path $staging 'bundle_manifest.json')
        $name = "$($Bundle.program_id)-$($result.content_identity)-$transport"
        $target = Join-Path $parent $name
        if (Test-Path -LiteralPath $target) {
            if ((DirectoryHash $target) -ne $transport) { throw "IMMUTABLE_CONFLICT: $target" }
            $existing = Get-Content -Raw -LiteralPath (Join-Path $target 'bundle_manifest.json') | ConvertFrom-Json
            if ($existing.bundle_content_identity -ne $result.content_identity -or $existing.directory_transport_hash -ne $transport) { throw "IMMUTABLE_CONFLICT: $target" }
            return [pscustomobject]@{ path = $target; name = $name; content = $result.content_identity; transport = $transport }
        }
        [IO.Directory]::Move($staging, $target)
        [pscustomobject]@{ path = $target; name = $name; content = $result.content_identity; transport = $transport }
    } finally { if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force -ErrorAction SilentlyContinue } }
}

$spec = Get-Content -Raw -LiteralPath $SpecPath | ConvertFrom-Json
$registry = Get-Content -Raw -LiteralPath $RegistryPath | ConvertFrom-Json
$idByProgram = @{ 'AP-001' = 'xauusd.wave1.ap001'; 'AP-002' = 'xauusd.wave1.ap002'; 'TC-001' = 'xauusd.wave1.tc001'; 'BG-001' = 'xauusd.wave1.bg001' }
$results = @{}
foreach ($bundle in $spec.bundles) {
    $source = @($registry.sources | Where-Object source_id -eq $idByProgram[$bundle.program_id])[0]
    if ($null -eq $source) { throw "REGISTRY_SOURCE_MISSING: $($bundle.program_id)" }
    if ($VerifyOnly) {
        $path = Join-Path $AuthorityRoot (Split-Path $source.relative_path -Leaf)
        if (-not (Test-Path -LiteralPath $path)) { throw "BUNDLE_MISSING: $path" }
        $results[$bundle.program_id] = [pscustomobject]@{ path = $path; name = Split-Path $path -Leaf; content = $source.bundle_content_identity; transport = $source.directory_transport_hash }
    } else { $results[$bundle.program_id] = GenerateBundle $bundle }
}
if ($UpdateRegistry -and -not $VerifyOnly) {
    foreach ($source in $registry.sources) {
        $program = $idByProgram.Keys | Where-Object { $idByProgram[$_] -eq $source.source_id }
        $result = $results[$program]
        if ($null -eq $result) { throw "REGISTRY_SOURCE_MISSING: $($source.source_id)" }
        $source.root_env = 'TRINITYR_AUTHORITY_ROOT'
        $source.relative_path = "XAUUSD/WAVE1_AUTHORITY_V3/$($result.name)"
        $source.hash = $result.transport
        $source.bundle_content_identity = $result.content
        $source.directory_transport_hash = $result.transport
    }
    WriteJson $registry $RegistryPath
}
foreach ($program in $idByProgram.Keys) { "VERIFIED $program $($results[$program].content) $($results[$program].transport)" }
