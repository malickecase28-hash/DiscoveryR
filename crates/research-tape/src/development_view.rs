//! Immutable physical development-scope firewall.  This module never parses payload columns.
use arrow_array::{Array, BooleanArray, Int64Array, RecordBatch};
use arrow_select::filter::filter_record_batch;
use parquet::{
    arrow::{
        arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
        ArrowWriter, ProjectionMask,
    },
    basic::Compression,
    file::properties::WriterProperties,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io,
    path::Path,
    process::Command,
    time::Instant,
};

pub const BOUNDARY_MS: i64 = 1_777_984_740_000;
pub const DEFAULT_BATCH_SIZE: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Bar { boundary_ms: i64 },
    Tick { boundary_ns: i64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    FullDevelopment,
    FullConfirmation,
    Mixed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateStats {
    Bar {
        rows: u64,
        bar_close_min: Option<i64>,
        bar_close_max: Option<i64>,
    },
    Tick {
        rows: u64,
        event_min: Option<i64>,
        event_max: Option<i64>,
        received_min: Option<i64>,
        received_max: Option<i64>,
    },
}
pub type RowStats = GateStats;

impl GateStats {
    fn new(gate: Gate) -> Self {
        match gate {
            Gate::Bar { .. } => Self::Bar {
                rows: 0,
                bar_close_min: None,
                bar_close_max: None,
            },
            Gate::Tick { .. } => Self::Tick {
                rows: 0,
                event_min: None,
                event_max: None,
                received_min: None,
                received_max: None,
            },
        }
    }
    fn rows(&self) -> u64 {
        match self {
            Self::Bar { rows, .. } | Self::Tick { rows, .. } => *rows,
        }
    }
    fn maxes_before(&self, boundary: i64) -> bool {
        match self {
            Self::Bar { bar_close_max, .. } => bar_close_max.is_none_or(|x| x < boundary),
            Self::Tick {
                event_max,
                received_max,
                ..
            } => {
                event_max.is_none_or(|x| x < boundary) && received_max.is_none_or(|x| x < boundary)
            }
        }
    }
    fn push(&mut self, gate: Gate, batch: &RecordBatch, row: usize) -> Result<(), String> {
        match self {
            Self::Bar {
                rows,
                bar_close_min,
                bar_close_max,
            } => {
                let a = batch
                    .column_by_name("bar_close_ts")
                    .and_then(|x| x.as_any().downcast_ref::<Int64Array>())
                    .ok_or("missing bar_close_ts")?;
                if a.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let v = a.value(row);
                *rows += 1;
                *bar_close_min = Some(bar_close_min.map_or(v, |x| x.min(v)));
                *bar_close_max = Some(bar_close_max.map_or(v, |x| x.max(v)));
            }
            Self::Tick {
                rows,
                event_min,
                event_max,
                received_min,
                received_max,
            } => {
                let e = batch
                    .column_by_name("event_ts_ns")
                    .and_then(|x| x.as_any().downcast_ref::<Int64Array>())
                    .ok_or("missing event_ts_ns")?;
                let r = batch
                    .column_by_name("received_ts_ns")
                    .and_then(|x| x.as_any().downcast_ref::<Int64Array>())
                    .ok_or("missing received_ts_ns")?;
                if e.is_null(row) || r.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let (ev, rv) = (e.value(row), r.value(row));
                *rows += 1;
                *event_min = Some(event_min.map_or(ev, |x| x.min(ev)));
                *event_max = Some(event_max.map_or(ev, |x| x.max(ev)));
                *received_min = Some(received_min.map_or(rv, |x| x.min(rv)));
                *received_max = Some(received_max.map_or(rv, |x| x.max(rv)));
            }
        }
        if matches!(gate, Gate::Bar { .. }) != matches!(self, Self::Bar { .. }) {
            return Err("gate/stat mismatch".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PartResult {
    pub classification: Classification,
    pub stats: RowStats,
    pub eligible: u64,
    pub scanned: u64,
}

fn gate_values(batch: &RecordBatch, gate: Gate) -> Result<Vec<Option<i64>>, String> {
    let names: &[&str] = match gate {
        Gate::Bar { .. } => &["bar_close_ts"],
        Gate::Tick { .. } => &["event_ts_ns", "received_ts_ns"],
    };
    let arrays: Result<Vec<&Int64Array>, _> = names
        .iter()
        .map(|name| {
            batch
                .column_by_name(name)
                .and_then(|a| a.as_any().downcast_ref())
                .ok_or_else(|| format!("missing/non-Int64 gate column {name}"))
        })
        .collect();
    let arrays = arrays?;
    (0..batch.num_rows())
        .map(|row| {
            let mut value = None;
            for array in &arrays {
                if array.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let v = array.value(row);
                if value.is_none() {
                    value = Some(v);
                }
            }
            Ok(value)
        })
        .collect()
}

fn eligible(
    values: &[Option<i64>],
    gate: Gate,
    row: usize,
    batch: &RecordBatch,
) -> Result<bool, String> {
    let boundary = match gate {
        Gate::Bar { boundary_ms } => boundary_ms,
        Gate::Tick { boundary_ns } => boundary_ns,
    };
    let value = values[row].ok_or_else(|| format!("null gating time at row {row}"))?;
    if matches!(gate, Gate::Bar { .. }) {
        Ok(value < boundary)
    } else {
        let received = batch
            .column_by_name("received_ts_ns")
            .and_then(|a| a.as_any().downcast_ref::<Int64Array>())
            .ok_or_else(|| "missing received_ts_ns".to_string())?;
        if received.is_null(row) {
            return Err(format!("null gating time at row {row}"));
        }
        Ok(value < boundary && received.value(row) < boundary)
    }
}

pub fn classify_batch(
    batch: &RecordBatch,
    gate: Gate,
) -> Result<(Classification, RowStats), String> {
    let values = gate_values(batch, gate)?;
    let mut stats = GateStats::new(gate);
    let mut included = 0;
    for row in 0..batch.num_rows() {
        stats.push(gate, batch, row)?;
        if eligible(&values, gate, row, batch)? {
            included += 1;
        }
    }
    let kind = if included == stats.rows() {
        Classification::FullDevelopment
    } else if included == 0 {
        Classification::FullConfirmation
    } else {
        Classification::Mixed
    };
    Ok((kind, stats))
}

pub fn filter_batch(batch: &RecordBatch, gate: Gate) -> Result<Option<RecordBatch>, String> {
    let values = gate_values(batch, gate)?;
    let mask = BooleanArray::from(
        (0..batch.num_rows())
            .map(|row| eligible(&values, gate, row, batch))
            .collect::<Result<Vec<_>, _>>()?,
    );
    if mask.true_count() == 0 {
        return Ok(None);
    }
    filter_record_batch(batch, &mask)
        .map(Some)
        .map_err(|e| e.to_string())
}

pub fn scan_part(
    path: &Path,
    gate: Gate,
    batch_size: usize,
) -> Result<PartResult, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let fields: Vec<_> = match gate {
        Gate::Bar { .. } => vec!["bar_close_ts"],
        Gate::Tick { .. } => vec!["event_ts_ns", "received_ts_ns"],
    };
    let indices = fields
        .iter()
        .map(|x| builder.schema().index_of(x))
        .collect::<Result<Vec<_>, _>>()?;
    let mask = ProjectionMask::roots(builder.parquet_schema(), indices);
    let reader = builder
        .with_projection(mask)
        .with_batch_size(batch_size)
        .build()?;
    let mut result = PartResult {
        classification: Classification::FullConfirmation,
        stats: GateStats::new(gate),
        eligible: 0,
        scanned: 0,
    };
    for batch in reader {
        let batch = batch?;
        result.scanned += batch.num_rows() as u64;
        let values = gate_values(&batch, gate)?;
        for row in 0..batch.num_rows() {
            result
                .stats
                .push(gate, &batch, row)
                .map_err(io::Error::other)?;
            if eligible(&values, gate, row, &batch)? {
                result.eligible += 1;
            }
        }
    }
    result.classification = if result.eligible == result.scanned {
        Classification::FullDevelopment
    } else if result.eligible == 0 {
        Classification::FullConfirmation
    } else {
        Classification::Mixed
    };
    Ok(result)
}

pub fn filter_part(
    source: &Path,
    target: &Path,
    gate: Gate,
    batch_size: usize,
) -> Result<RowStats, Box<dyn std::error::Error>> {
    let input = File::open(source)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(input)?;
    let schema = builder.schema().clone();
    let reader: ParquetRecordBatchReader = builder.with_batch_size(batch_size).build()?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let output = File::create(target)?;
    let mut writer = ArrowWriter::try_new(output, schema.clone(), Some(props))?;
    let mut stats = GateStats::new(gate);
    for batch in reader {
        let batch = batch?;
        let (kind, _) = classify_batch(&batch, gate).map_err(io::Error::other)?;
        if kind != Classification::FullConfirmation {
            if let Some(filtered) = filter_batch(&batch, gate).map_err(io::Error::other)? {
                for row in 0..filtered.num_rows() {
                    stats.push(gate, &filtered, row).map_err(io::Error::other)?;
                }
                writer.write(&filtered)?;
            }
        }
    }
    writer.close()?;
    let actual = ParquetRecordBatchReaderBuilder::try_new(File::open(target)?)?
        .schema()
        .clone();
    if actual.as_ref() != schema.as_ref() {
        return Err(io::Error::other("filtered output schema drift").into());
    }
    Ok(stats)
}

pub fn hard_link_or_copy(
    source: &Path,
    target: &Path,
    copy: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if copy {
        fs::copy(source, target)?;
        return Ok(());
    }
    fs::hard_link(source, target).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "hard link {} -> {} failed (use --copy-full-parts to opt in): {e}",
                source.display(),
                target.display()
            ),
        )
        .into()
    })
}

pub fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut h = Sha256::new();
    let mut f = File::open(path)?;
    io::copy(&mut f, &mut HashWriter(&mut h))?;
    Ok(format!("{:x}", h.finalize()))
}
struct HashWriter<'a>(&'a mut Sha256);
impl io::Write for HashWriter<'_> {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.update(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn hash_view_identity(
    scope: &str,
    source: &str,
    payload: &str,
    boundary: i64,
    paths: &[&str],
    stats: &[RowStats],
    hashes: &[&str],
    kinds: &[&str],
    code: &str,
    _runtime_millis: u128,
) -> String {
    let mut h = Sha256::new();
    for s in [scope, source, payload, code] {
        frame(&mut h, s.as_bytes());
    }
    h.update(boundary.to_le_bytes());
    for (((path, stat), hash), kind) in paths.iter().zip(stats).zip(hashes).zip(kinds) {
        frame(&mut h, path.as_bytes());
        frame(&mut h, kind.as_bytes());
        match stat {
            GateStats::Bar {
                rows,
                bar_close_min,
                bar_close_max,
            } => {
                h.update([0]);
                h.update(rows.to_le_bytes());
                h.update(bar_close_min.unwrap_or_default().to_le_bytes());
                h.update(bar_close_max.unwrap_or_default().to_le_bytes());
            }
            GateStats::Tick {
                rows,
                event_min,
                event_max,
                received_min,
                received_max,
            } => {
                h.update([1]);
                h.update(rows.to_le_bytes());
                h.update(event_min.unwrap_or_default().to_le_bytes());
                h.update(event_max.unwrap_or_default().to_le_bytes());
                h.update(received_min.unwrap_or_default().to_le_bytes());
                h.update(received_max.unwrap_or_default().to_le_bytes());
            }
        }
        frame(&mut h, hash.as_bytes());
    }
    format!("{:x}", h.finalize())
}
fn frame(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

pub fn reject_reparse(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(io::Error::other(format!("symlink exposed: {}", path.display())).into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(
                io::Error::other(format!("reparse point exposed: {}", path.display())).into(),
            );
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub scope_id: String,
    pub instrument: String,
    pub source_manifest_identity: String,
    pub payload_manifest_identity: String,
    pub boundary_rule: String,
    pub development_end_exclusive: String,
    pub builder_code_identity: String,
    pub source_groups: BTreeMap<String, Vec<ManifestFile>>,
    pub logical_view_identity: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestFile {
    pub logical_path: String,
    pub materialization_kind: String,
    pub development_row_count: u64,
    pub bar_close_min: Option<i64>,
    pub bar_close_max: Option<i64>,
    pub event_min: Option<i64>,
    pub event_max: Option<i64>,
    pub received_min: Option<i64>,
    pub received_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_part: Option<String>,
}

pub fn parse_scale(value: &serde_json::Value) -> Result<(String, Gate), String> {
    if value == "Tick" {
        return Ok((
            "tick".into(),
            Gate::Tick {
                boundary_ns: BOUNDARY_MS
                    .checked_mul(1_000_000)
                    .ok_or("boundary overflow")?,
            },
        ));
    }
    let scale = value
        .get("Bar")
        .and_then(|v| v.as_str())
        .ok_or("invalid scale")?;
    Ok((
        scale.into(),
        Gate::Bar {
            boundary_ms: BOUNDARY_MS,
        },
    ))
}

#[derive(Deserialize)]
struct Inventory {
    source_manifest_identity: String,
    payload_manifest_identity: String,
    sources: Vec<InventoryGroup>,
}
#[derive(Deserialize)]
struct InventoryGroup {
    scale: serde_json::Value,
    parts: Vec<InventoryPart>,
}
#[derive(Deserialize)]
struct InventoryPart {
    path: String,
    rows: u64,
}
fn required_identity(name: &str, value: &str) -> Result<String, Box<dyn std::error::Error>> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::other(format!("invalid {name}")).into());
    }
    Ok(value.into())
}
pub fn validate_code_identity(value: &str) -> Result<&str, Box<dyn std::error::Error>> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(
            io::Error::other("DISCOVERYR_CODE_IDENTITY must be a 40-character Git SHA").into(),
        );
    }
    let resolved = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--verify", &format!("{value}^{{commit}}")])
            .output()?
            .stdout,
    )?
    .trim()
    .to_owned();
    if resolved != value {
        return Err(io::Error::other("code identity is not the requested Git commit").into());
    }
    Ok(value)
}

fn revoked(root: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if root.join("REVOKED").exists() {
        return Ok(true);
    }
    let manifest: serde_json::Value =
        serde_json::from_reader(File::open(root.join("view_manifest.json"))?)?;
    Ok(
        manifest.get("revoked").and_then(serde_json::Value::as_bool) == Some(true)
            || manifest.get("status").and_then(serde_json::Value::as_str) == Some("REVOKED"),
    )
}

pub fn verify_view(
    root: &Path,
    manifest: &Manifest,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if revoked(root)? {
        return Err(io::Error::other("development view is revoked").into());
    }
    validate_code_identity(&manifest.builder_code_identity)?;
    let mut expected = BTreeSet::from(["view_manifest.json".to_string()]);
    let mut paths = Vec::new();
    let mut stats = Vec::new();
    let mut hashes = Vec::new();
    let mut kinds = Vec::new();
    for (group, files) in &manifest.source_groups {
        let gate = if group == "tick" {
            Gate::Tick {
                boundary_ns: BOUNDARY_MS
                    .checked_mul(1_000_000)
                    .ok_or("boundary overflow")?,
            }
        } else {
            Gate::Bar {
                boundary_ms: BOUNDARY_MS,
            }
        };
        for file in files {
            let relative = Path::new(&file.logical_path);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(io::Error::other("source path traversal").into());
            }
            let path = root.join(relative);
            reject_reparse(&path)?;
            expected.insert(file.logical_path.clone());
            let result = scan_part(&path, gate, batch_size)?;
            if result.classification != Classification::FullDevelopment
                || result.eligible != result.scanned
                || result.scanned != file.development_row_count
            {
                return Err(io::Error::other(format!(
                    "confirmation leak or row mismatch: {}",
                    file.logical_path
                ))
                .into());
            }
            let boundary = match gate {
                Gate::Bar { boundary_ms } => boundary_ms,
                Gate::Tick { boundary_ns } => boundary_ns,
            };
            if !result.stats.maxes_before(boundary) {
                return Err(
                    io::Error::other(format!("gate violation: {}", file.logical_path)).into(),
                );
            }
            let hash = file.sha256.as_deref().ok_or_else(|| {
                io::Error::other(format!("missing payload hash: {}", file.logical_path))
            })?;
            if hash_file(&path)? != hash {
                return Err(io::Error::other(format!(
                    "payload hash mismatch: {}",
                    file.logical_path
                ))
                .into());
            }
            let actual_stats = result.stats;
            let expected_stats = match (&actual_stats, group == "tick") {
                (GateStats::Tick { .. }, true) | (GateStats::Bar { .. }, false) => true,
                _ => false,
            };
            if !expected_stats || actual_stats != manifest_stats(file, group == "tick")? {
                return Err(io::Error::other(format!(
                    "manifest stats mismatch: {}",
                    file.logical_path
                ))
                .into());
            }
            paths.push(file.logical_path.as_str());
            stats.push(actual_stats);
            hashes.push(hash);
            kinds.push(file.materialization_kind.as_str());
        }
    }
    let mut actual = BTreeSet::new();
    for entry in walk_files(root)? {
        let path = entry.strip_prefix(root)?;
        reject_reparse(&entry)?;
        actual.insert(path.to_string_lossy().replace('\\', "/"));
    }
    if actual != expected {
        return Err(io::Error::other("unexpected files or missing manifest entry").into());
    }
    let identity = hash_view_identity(
        &manifest.scope_id,
        &manifest.source_manifest_identity,
        &manifest.payload_manifest_identity,
        BOUNDARY_MS,
        &paths,
        &stats,
        &hashes,
        &kinds,
        &manifest.builder_code_identity,
        0,
    );
    if identity != manifest.logical_view_identity {
        return Err(io::Error::other("logical view identity mismatch").into());
    }
    Ok(())
}

fn manifest_stats(file: &ManifestFile, tick: bool) -> Result<RowStats, Box<dyn std::error::Error>> {
    if tick {
        Ok(GateStats::Tick {
            rows: file.development_row_count,
            event_min: file.event_min,
            event_max: file.event_max,
            received_min: file.received_min,
            received_max: file.received_max,
        })
    } else {
        Ok(GateStats::Bar {
            rows: file.development_row_count,
            bar_close_min: file.bar_close_min,
            bar_close_max: file.bar_close_max,
        })
    }
}

fn walk_files(root: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            reject_reparse(&path)?;
            files.extend(walk_files(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

pub fn materialize(
    source_root: &Path,
    inventory_path: &Path,
    view_root: &Path,
    audit_root: &Path,
    code: &str,
    batch_size: usize,
    copy_full: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let started = Instant::now();
    validate_code_identity(code)?;
    let inventory: Inventory = serde_json::from_reader(File::open(inventory_path)?)?;
    let scope = "XAUUSD_DATA_SCOPE_V1";
    let source_id = required_identity(
        "source_manifest_identity",
        &inventory.source_manifest_identity,
    )?;
    let payload_id = required_identity(
        "payload_manifest_identity",
        &inventory.payload_manifest_identity,
    )?;
    if view_root.exists() {
        if revoked(view_root)? {
            return Err(io::Error::other("existing development view is revoked").into());
        }
        let existing: Manifest =
            serde_json::from_reader(File::open(view_root.join("view_manifest.json"))?)?;
        if existing.scope_id == scope
            && existing.source_manifest_identity == source_id
            && existing.payload_manifest_identity == payload_id
            && existing.builder_code_identity == code
            && existing.development_end_exclusive == "2026-05-05T12:39:00Z"
        {
            verify_view(view_root, &existing, batch_size)?;
            return Ok("ALREADY_MATERIALIZED".into());
        }
        return Err(io::Error::other("existing development view conflicts").into());
    }
    let parent = view_root.parent().ok_or("view has no parent")?;
    let temp = parent.join(format!(".development-building-{}", std::process::id()));
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;
    let mut groups = BTreeMap::new();
    let mut paths = Vec::new();
    let mut stats = Vec::new();
    let mut hashes = Vec::new();
    let mut kinds = Vec::new();
    let mut audit_parts = Vec::new();
    let mut bytes_linked = 0u64;
    let mut bytes_rewritten = 0u64;
    for group in &inventory.sources {
        let (name, gate) = parse_scale(&group.scale)?;
        let mut files = Vec::new();
        for part in &group.parts {
            let logical = part.path.as_str();
            let logical_path = Path::new(logical);
            if logical_path.is_absolute()
                || logical_path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(io::Error::other("source path traversal").into());
            }
            let source = source_root.join(logical_path);
            let result = scan_part(&source, gate, batch_size)?;
            if result.scanned != part.rows {
                return Err(
                    io::Error::other(format!("declared row count mismatch: {logical}")).into(),
                );
            }
            let classification = match result.classification {
                Classification::FullDevelopment => "FULL_DEVELOPMENT",
                Classification::FullConfirmation => "FULL_CONFIRMATION",
                Classification::Mixed => "MIXED",
            };
            audit_parts.push(serde_json::json!({"logical_path": logical, "classification": classification, "declared_source_rows": part.rows, "scanned_rows": result.scanned, "development_rows": result.eligible, "excluded_rows": result.scanned - result.eligible}));
            if result.classification == Classification::FullConfirmation {
                continue;
            }
            let target = temp.join(logical_path);
            fs::create_dir_all(target.parent().unwrap())?;
            let (kind, sha, development_stats) = match result.classification {
                Classification::FullDevelopment => {
                    hard_link_or_copy(&source, &target, copy_full)?;
                    bytes_linked += fs::metadata(&source)?.len();
                    (
                        if copy_full {
                            "COPIED_FULL_DEVELOPMENT"
                        } else {
                            "HARDLINK_FULL_DEVELOPMENT"
                        }
                        .to_string(),
                        Some(hash_file(&target)?),
                        result.stats.clone(),
                    )
                }
                Classification::Mixed => {
                    let filtered_stats = filter_part(&source, &target, gate, batch_size)?;
                    bytes_rewritten += fs::metadata(&target)?.len();
                    (
                        "FILTERED_MIXED_PART".to_string(),
                        Some(hash_file(&target)?),
                        filtered_stats,
                    )
                }
                Classification::FullConfirmation => unreachable!(),
            };
            paths.push(logical);
            stats.push(development_stats.clone());
            hashes.push(sha.clone().unwrap_or_default());
            kinds.push(kind.clone());
            files.push(ManifestFile {
                logical_path: logical.into(),
                materialization_kind: kind,
                development_row_count: result.eligible,
                bar_close_min: match development_stats {
                    GateStats::Bar { bar_close_min, .. } => bar_close_min,
                    _ => None,
                },
                bar_close_max: match development_stats {
                    GateStats::Bar { bar_close_max, .. } => bar_close_max,
                    _ => None,
                },
                event_min: match development_stats {
                    GateStats::Tick { event_min, .. } => event_min,
                    _ => None,
                },
                event_max: match development_stats {
                    GateStats::Tick { event_max, .. } => event_max,
                    _ => None,
                },
                received_min: match development_stats {
                    GateStats::Tick { received_min, .. } => received_min,
                    _ => None,
                },
                received_max: match development_stats {
                    GateStats::Tick { received_max, .. } => received_max,
                    _ => None,
                },
                sha256: sha,
                source_part: (result.classification == Classification::Mixed)
                    .then(|| logical.into()),
            });
        }
        groups.insert(name, files);
    }
    let path_refs: Vec<_> = paths.to_vec();
    let hash_refs: Vec<_> = hashes.iter().map(String::as_str).collect();
    let kind_refs: Vec<_> = kinds.iter().map(String::as_str).collect();
    let identity = hash_view_identity(
        scope,
        &source_id,
        &payload_id,
        BOUNDARY_MS,
        &path_refs,
        &stats,
        &hash_refs,
        &kind_refs,
        code,
        started.elapsed().as_millis(),
    );
    let manifest = Manifest { schema_version: 1, scope_id: scope.into(), instrument: "XAUUSD".into(), source_manifest_identity: source_id.into(), payload_manifest_identity: payload_id.into(), boundary_rule: "PHYSICAL DEVELOPMENT SCOPE FILTERS: end-exclusive; bars bar_close_ts < boundary; ticks event_ts_ns AND received_ts_ns < boundary_ns; null gating time is fatal".into(), development_end_exclusive: "2026-05-05T12:39:00Z".into(), builder_code_identity: code.into(), source_groups: groups, logical_view_identity: identity.clone() };
    let manifest_path = temp.join("view_manifest.json");
    serde_json::to_writer_pretty(File::create(manifest_path)?, &manifest)?;
    verify_view(&temp, &manifest, batch_size)?;
    fs::rename(&temp, view_root)?;
    fs::create_dir_all(audit_root)?;
    serde_json::to_writer_pretty(
        File::create(audit_root.join("build_audit.json"))?,
        &serde_json::json!({"scope_id":scope,"source_parts_examined":audit_parts,"hardlinked_parts":kinds.iter().filter(|k| k.as_str()=="HARDLINK_FULL_DEVELOPMENT").count(),"filtered_mixed_parts":kinds.iter().filter(|k| k.as_str()=="FILTERED_MIXED_PART").count(),"confirmation_only_parts":audit_parts.iter().filter(|p| p["classification"]=="FULL_CONFIRMATION").count(),"bytes_linked":bytes_linked,"bytes_rewritten":bytes_rewritten,"elapsed_millis":started.elapsed().as_millis(),"logical_view_identity":identity}),
    )?;
    Ok(identity)
}
