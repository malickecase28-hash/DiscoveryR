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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowStats {
    pub rows: u64,
    pub gate_min: Option<i64>,
    pub gate_max: Option<i64>,
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
    let mut stats = RowStats::default();
    let mut included = 0;
    for row in 0..batch.num_rows() {
        let v = values[row].unwrap();
        stats.rows += 1;
        stats.gate_min = Some(stats.gate_min.map_or(v, |x| x.min(v)));
        stats.gate_max = Some(stats.gate_max.map_or(v, |x| x.max(v)));
        if eligible(&values, gate, row, batch)? {
            included += 1;
        }
    }
    let kind = if included == stats.rows {
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
        stats: RowStats::default(),
        eligible: 0,
        scanned: 0,
    };
    for batch in reader {
        let batch = batch?;
        result.scanned += batch.num_rows() as u64;
        let values = gate_values(&batch, gate)?;
        for row in 0..batch.num_rows() {
            let v = values[row].unwrap();
            result.stats.rows += 1;
            result.stats.gate_min = Some(result.stats.gate_min.map_or(v, |x| x.min(v)));
            result.stats.gate_max = Some(result.stats.gate_max.map_or(v, |x| x.max(v)));
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
    let mut writer = ArrowWriter::try_new(output, schema, Some(props))?;
    let mut stats = RowStats::default();
    for batch in reader {
        let batch = batch?;
        let (kind, part_stats) = classify_batch(&batch, gate).map_err(io::Error::other)?;
        stats.rows += part_stats.rows;
        stats.gate_min = match (stats.gate_min, part_stats.gate_min) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        stats.gate_max = match (stats.gate_max, part_stats.gate_max) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        if kind != Classification::FullConfirmation {
            if let Some(filtered) = filter_batch(&batch, gate).map_err(io::Error::other)? {
                writer.write(&filtered)?;
            }
        }
    }
    writer.close()?;
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
    code: &str,
    _runtime_millis: u128,
) -> String {
    let mut h = Sha256::new();
    for s in [scope, source, payload, code] {
        frame(&mut h, s.as_bytes());
    }
    h.update(boundary.to_le_bytes());
    for ((path, stat), hash) in paths.iter().zip(stats).zip(hashes) {
        frame(&mut h, path.as_bytes());
        h.update(stat.rows.to_le_bytes());
        h.update(stat.gate_min.unwrap_or_default().to_le_bytes());
        h.update(stat.gate_max.unwrap_or_default().to_le_bytes());
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
    pub gate_time_min: Option<i64>,
    pub gate_time_max: Option<i64>,
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

pub fn verify_view(
    root: &Path,
    manifest: &Manifest,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut expected = BTreeSet::from(["view_manifest.json".to_string()]);
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
            if result.classification == Classification::FullConfirmation
                || result.eligible != file.development_row_count
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
            if result.stats.gate_max.is_some_and(|max| max >= boundary) {
                return Err(
                    io::Error::other(format!("gate violation: {}", file.logical_path)).into(),
                );
            }
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
    Ok(())
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
    let inventory: serde_json::Value = serde_json::from_reader(File::open(inventory_path)?)?;
    let scope = "XAUUSD_DATA_SCOPE_V1";
    let source_id = inventory["source_manifest_identity"]
        .as_str()
        .unwrap_or_default();
    let payload_id = inventory["payload_manifest_identity"]
        .as_str()
        .unwrap_or_default();
    if view_root.exists() {
        let existing: Manifest =
            serde_json::from_reader(File::open(view_root.join("view_manifest.json"))?)?;
        if existing.scope_id == scope
            && existing.source_manifest_identity == source_id
            && existing.payload_manifest_identity == payload_id
            && existing.builder_code_identity == code
            && existing.development_end_exclusive == "2026-05-05T12:39:00Z"
        {
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
    for group in inventory["sources"].as_array().ok_or("sources missing")? {
        let (name, gate) = parse_scale(&group["scale"])?;
        let mut files = Vec::new();
        for part in group["parts"].as_array().ok_or("parts missing")? {
            let logical = part["path"].as_str().ok_or("path missing")?;
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
            if result.classification == Classification::FullConfirmation {
                continue;
            }
            let target = temp.join(logical_path);
            fs::create_dir_all(target.parent().unwrap())?;
            let (kind, sha) = match result.classification {
                Classification::FullDevelopment => {
                    hard_link_or_copy(&source, &target, copy_full)?;
                    (
                        if copy_full {
                            "COPIED_FULL_DEVELOPMENT"
                        } else {
                            "HARDLINK_FULL_DEVELOPMENT"
                        }
                        .into(),
                        None,
                    )
                }
                Classification::Mixed => {
                    filter_part(&source, &target, gate, batch_size)?;
                    ("FILTERED_MIXED_PART".into(), Some(hash_file(&target)?))
                }
                Classification::FullConfirmation => unreachable!(),
            };
            paths.push(logical);
            stats.push(RowStats {
                rows: result.eligible,
                ..result.stats
            });
            hashes.push(sha.clone().unwrap_or_default());
            files.push(ManifestFile {
                logical_path: logical.into(),
                materialization_kind: kind,
                development_row_count: result.eligible,
                gate_time_min: result.stats.gate_min,
                gate_time_max: result.stats.gate_max,
                sha256: sha,
                source_part: (result.classification == Classification::Mixed)
                    .then(|| logical.into()),
            });
        }
        groups.insert(name, files);
    }
    let path_refs: Vec<_> = paths.to_vec();
    let hash_refs: Vec<_> = hashes.iter().map(String::as_str).collect();
    let identity = hash_view_identity(
        scope,
        source_id,
        payload_id,
        BOUNDARY_MS,
        &path_refs,
        &stats,
        &hash_refs,
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
        &serde_json::json!({"scope_id":scope,"elapsed_millis":started.elapsed().as_millis(),"logical_view_identity":identity}),
    )?;
    Ok(identity)
}
