[CmdletBinding()]
param(
    [string]$SourceRoot = 'F:\TrinityR-research\schemas',
    [string]$SpecPath = (Join-Path $PSScriptRoot '..\authority_sources\XAUUSD.wave1.spec.json'),
    [string]$OutputRoot = 'F:\TrinityR-authority\XAUUSD\WAVE1_AUTHORITY_V1',
    [switch]$RunSyntheticLeakTest
)

$ErrorActionPreference = 'Stop'

function Get-Sha256([byte[]]$Bytes) {
    $hash = [Security.Cryptography.SHA256]::Create().ComputeHash($Bytes)
    return ([BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
}

function Get-FileSha256([string]$Path) { Get-Sha256 ([IO.File]::ReadAllBytes($Path)) }

function Write-Json([object]$Value, [string]$Path) {
    $json = $Value | ConvertTo-Json -Depth 30
    [IO.File]::WriteAllText($Path, $json + "`n", [Text.UTF8Encoding]::new($false))
}

function Get-SelectorValue([object]$Document, [string]$Pointer) {
    $current = $Document
    foreach ($part in ($Pointer.TrimStart('/') -split '/')) {
        $name = $part.Replace('~1', '/').Replace('~0', '~')
        if ($null -eq $current) { throw "Selector not found: $Pointer" }
        if ($current -is [array]) { $current = $current[[int]$name] }
        else { $current = $current.PSObject.Properties[$name].Value }
    }
    return $current
}

function New-Bundle([object]$Bundle, [string]$Root) {
    $dir = Join-Path $Root $Bundle.bundle_id
    New-Item -ItemType Directory -Force $dir | Out-Null
    $selected = [ordered]@{}
    $provenance = [Collections.Generic.List[object]]::new()
    $sourceRecords = [Collections.Generic.List[object]]::new()
    if ($null -ne $Bundle.source_file) {
        $sourcePath = Join-Path $SourceRoot $Bundle.source_file
        $sourceBytes = [IO.File]::ReadAllBytes($sourcePath)
        $sourceHash = Get-Sha256 $sourceBytes
        $sourceRecords.Add([ordered]@{ relative_path = $Bundle.source_file; sha256 = $sourceHash })
        $document = [Text.Encoding]::UTF8.GetString($sourceBytes) | ConvertFrom-Json
        foreach ($selector in $Bundle.selectors) {
            $key = $selector.TrimStart('/').Replace('/', '_')
            $selected[$key] = Get-SelectorValue $document $selector
            $provenance.Add([ordered]@{ source_relative_path = $Bundle.source_file; source_sha256 = $sourceHash; selector = $selector })
        }
    }
    $authority = [ordered]@{
        schema_version = 'trinity.authority.v1'
        bundle_id = $Bundle.bundle_id
        program_id = $Bundle.program_id
        instrument = 'XAUUSD'
        selected = $selected
        provenance = $provenance
        unresolved = @($Bundle.unresolved)
    }
    $authorityPath = Join-Path $dir 'authority.json'
    Write-Json $authority $authorityPath
    $authorityHash = Get-FileSha256 $authorityPath
    $manifest = [ordered]@{
        schema_version = 'trinity.authority-bundle-manifest.v1'
        bundle_id = $Bundle.bundle_id
        program_id = $Bundle.program_id
        instrument = 'XAUUSD'
        source_files = $sourceRecords
        extraction_selectors = @($Bundle.selectors)
        generated_files = @([ordered]@{ relative_path = 'authority.json'; sha256 = $authorityHash })
        bundle_identity = $authorityHash
    }
    Write-Json $manifest (Join-Path $dir 'bundle_manifest.json')
    return $authorityHash
}

function Assert-NoLeak([string]$Root) {
    $forbidden = @('emitted', 'parquet_samples', 'feature_records', 'performance', 'checkpoint_scan', 'replay_write_wall_seconds', 'broker credentials', 'account secrets')
    foreach ($file in Get-ChildItem $Root -Recurse -File) {
        $text = Get-Content -Raw $file.FullName
        foreach ($term in $forbidden) { if ($text -match [regex]::Escape($term)) { throw "Forbidden concept '$term' found in $($file.FullName)" } }
    }
}

if ($RunSyntheticLeakTest) {
    $fixture = Join-Path ([IO.Path]::GetTempPath()) ('wave1-authority-fixture-' + [guid]::NewGuid())
    New-Item -ItemType Directory -Force $fixture | Out-Null
    [IO.File]::WriteAllText((Join-Path $fixture 'fixture.json'), '{"declared_contract":{"field":"string"},"emitted":{"count":4},"market_samples":[100],"performance":{"seconds":1},"secret_like":{"account":"x"}}')
    $synthetic = [ordered]@{ schema_version='trinity.authority-extraction.v1'; instrument='XAUUSD'; bundles=@([ordered]@{bundle_id='SYNTHETIC'; program_id='synthetic'; source_file='fixture.json'; selectors=@('/declared_contract'); unresolved=@()}) }
    $syntheticPath = Join-Path $fixture 'spec.json'; Write-Json $synthetic $syntheticPath
    $oldSpec = $SpecPath; $oldSource = $SourceRoot; $oldOutput = $OutputRoot
    $SpecPath = $syntheticPath; $SourceRoot = $fixture; $OutputRoot = Join-Path $fixture 'out'
    $spec = Get-Content -Raw $SpecPath | ConvertFrom-Json; New-Item -ItemType Directory -Force $OutputRoot | Out-Null; New-Bundle $spec.bundles[0] $OutputRoot | Out-Null
    $body = Get-Content -Raw (Join-Path $OutputRoot 'SYNTHETIC\authority.json')
    if ($body -match 'emitted|market_samples|performance|secret_like') { throw 'Synthetic leak test failed' }
    Remove-Item -LiteralPath $fixture -Recurse -Force
    Write-Output 'Synthetic leak test: PASS'
    exit 0
}

$spec = Get-Content -Raw $SpecPath | ConvertFrom-Json
New-Item -ItemType Directory -Force $OutputRoot | Out-Null
foreach ($bundle in $spec.bundles) { New-Bundle $bundle $OutputRoot | Out-Null }
Assert-NoLeak $OutputRoot
Write-Output "Generated $($spec.bundles.Count) bundles at $OutputRoot"
