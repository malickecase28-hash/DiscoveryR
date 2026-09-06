//! Immutable physical development-scope firewall.  This module never parses payload columns.
use arrow_array::{Array, BooleanArray, Int64Array, RecordBatch};
use arrow_select::filter::filter_record_batch;
use parquet::{
    arrow::{
        arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
        ArrowWriter, ProjectionMask,
    },
    basic::Compression,
    file::{properties::WriterProperties, statistics::Statistics},
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
/// Frozen XAUUSD_DATA_SCOPE_V1 development.start_utc_inclusive: 2025-07-31T16:15:00Z.
pub const DEVELOPMENT_START_MS: i64 = 1_753_978_500_000;
pub const DEVELOPMENT_START_NS: i64 = 1_753_978_500_000_000_000;
pub const DEVELOPMENT_START_INCLUSIVE: &str = "2025-07-31T16:15:00Z";
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
    #[allow(dead_code)]
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
    fn mins_at_or_after_start(&self) -> bool {
        match self {
            Self::Bar { bar_close_min, .. } => {
                bar_close_min.is_none_or(|x| x > DEVELOPMENT_START_MS)
            }
            Self::Tick {
                event_min,
                received_min,
                ..
            } => {
                event_min.is_none_or(|x| x >= DEVELOPMENT_START_NS)
                    && received_min.is_none_or(|x| x >= DEVELOPMENT_START_NS)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartResult {
    pub classification: Classification,
    pub stats: RowStats,
    pub eligible: u64,
    pub scanned: u64,
}

/// Gate columns resolved once per batch instead of once per row.
struct GateColumns<'a> {
    /// Bar: bar_close_ts. Tick: event_ts_ns.
    primary: &'a Int64Array,
    /// Bar: bar_open_ts. Tick: None.
    open: Option<&'a Int64Array>,
    /// Tick: received_ts_ns. Bar: None.
    received: Option<&'a Int64Array>,
    boundary: i64,
}

impl<'a> GateColumns<'a> {
    fn build(batch: &'a RecordBatch, gate: Gate) -> Result<Self, String> {
        let boundary = match gate {
            Gate::Bar { boundary_ms } => boundary_ms,
            Gate::Tick { boundary_ns } => boundary_ns,
        };
        let (primary_name, secondary_name) = match gate {
            Gate::Bar { .. } => ("bar_close_ts", "bar_open_ts"),
            Gate::Tick { .. } => ("event_ts_ns", "received_ts_ns"),
        };
        let primary = Self::column(batch, primary_name)?;
        let secondary = Self::column(batch, secondary_name)?;
        Ok(match gate {
            Gate::Bar { .. } => Self {
                primary,
                open: Some(secondary),
                received: None,
                boundary,
            },
            Gate::Tick { .. } => Self {
                primary,
                open: None,
                received: Some(secondary),
                boundary,
            },
        })
    }
    fn column(batch: &'a RecordBatch, name: &str) -> Result<&'a Int64Array, String> {
        batch
            .column_by_name(name)
            .and_then(|a| a.as_any().downcast_ref::<Int64Array>())
            .ok_or_else(|| format!("missing/non-Int64 gate column {name}"))
    }
    fn eligible(&self, row: usize) -> Result<bool, String> {
        if self.primary.is_null(row) {
            return Err(format!("null gating time at row {row}"));
        }
        let primary = self.primary.value(row);
        match (self.open, self.received) {
            (Some(open), _) => {
                if open.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                Ok(open.value(row) >= DEVELOPMENT_START_MS && primary < self.boundary)
            }
            (None, Some(received)) => {
                if received.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let received = received.value(row);
                Ok(primary >= DEVELOPMENT_START_NS
                    && primary < self.boundary
                    && received >= DEVELOPMENT_START_NS
                    && received < self.boundary)
            }
            (None, None) => Err("gate/stat mismatch".into()),
        }
    }
    fn push_stats(&self, stats: &mut GateStats, row: usize) -> Result<(), String> {
        if self.primary.is_null(row) {
            return Err(format!("null gating time at row {row}"));
        }
        match stats {
            GateStats::Bar { .. } => {
                let open = self.open.ok_or("gate/stat mismatch")?;
                if open.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let GateStats::Bar {
                    rows,
                    bar_close_min,
                    bar_close_max,
                } = stats
                else {
                    unreachable!()
                };
                let v = self.primary.value(row);
                *rows += 1;
                *bar_close_min = Some(bar_close_min.map_or(v, |x| x.min(v)));
                *bar_close_max = Some(bar_close_max.map_or(v, |x| x.max(v)));
            }
            GateStats::Tick { .. } => {
                let received = self.received.ok_or("gate/stat mismatch")?;
                if received.is_null(row) {
                    return Err(format!("null gating time at row {row}"));
                }
                let GateStats::Tick {
                    rows,
                    event_min,
                    event_max,
                    received_min,
                    received_max,
                } = stats
                else {
                    unreachable!()
                };
                let ev = self.primary.value(row);
                let rv = received.value(row);
                *rows += 1;
                *event_min = Some(event_min.map_or(ev, |x| x.min(ev)));
                *event_max = Some(event_max.map_or(ev, |x| x.max(ev)));
                *received_min = Some(received_min.map_or(rv, |x| x.min(rv)));
                *received_max = Some(received_max.map_or(rv, |x| x.max(rv)));
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn classify_batch(
    batch: &RecordBatch,
    gate: Gate,
) -> Result<(Classification, RowStats), String> {
    let columns = GateColumns::build(batch, gate)?;
    let mut stats = GateStats::new(gate);
    let mut included = 0;
    for row in 0..batch.num_rows() {
        columns.push_stats(&mut stats, row)?;
        if columns.eligible(row)? {
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

#[allow(dead_code)]
pub fn filter_batch(batch: &RecordBatch, gate: Gate) -> Result<Option<RecordBatch>, String> {
    let columns = GateColumns::build(batch, gate)?;
    let mask = BooleanArray::from(
        (0..batch.num_rows())
            .map(|row| columns.eligible(row))
            .collect::<Result<Vec<_>, _>>()?,
    );
    if mask.true_count() == 0 {
        return Ok(None);
    }
    filter_record_batch(batch, &mask)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Classify a part from parquet footer statistics alone, without reading rows.
/// Returns None whenever the footer cannot PROVE the classification (missing or
/// typed-incompatible statistics, any null gating times, or a straddling part);
/// the caller must then fall back to a full row scan. Sound: a provable
/// classification follows from per-column-chunk min/max/null-count alone.
fn classify_part_from_footer(
    path: &Path,
    gate: Gate,
) -> Result<Option<PartResult>, Box<dyn std::error::Error>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    let builder = match ParquetRecordBatchReaderBuilder::try_new(file) {
        Ok(builder) => builder,
        Err(_) => return Ok(None),
    };
    let names: &[&str] = match gate {
        Gate::Bar { .. } => &["bar_open_ts", "bar_close_ts"],
        Gate::Tick { .. } => &["event_ts_ns", "received_ts_ns"],
    };
    let mut indices = Vec::with_capacity(names.len());
    for name in names {
        match builder.schema().index_of(name) {
            Ok(index) => indices.push(index),
            Err(_) => return Ok(None),
        }
    }
    let metadata = builder.metadata().clone();
    let rows = metadata.file_metadata().num_rows();
    if rows <= 0 {
        return Ok(None);
    }
    let rows = rows as u64;
    let mut mins = vec![i64::MAX; names.len()];
    let mut maxs = vec![i64::MIN; names.len()];
    for row_group in metadata.row_groups() {
        for (offset, &schema_index) in indices.iter().enumerate() {
            let column = row_group.column(schema_index);
            if column.column_descr().name() != names[offset] {
                return Ok(None);
            }
            let Some(statistics) = column.statistics() else {
                return Ok(None);
            };
            if statistics.null_count_opt() != Some(0) {
                return Ok(None);
            }
            let Statistics::Int64(typed) = statistics else {
                return Ok(None);
            };
            let (Some(min), Some(max)) = (typed.min_opt(), typed.max_opt()) else {
                return Ok(None);
            };
            mins[offset] = mins[offset].min(*min);
            maxs[offset] = maxs[offset].max(*max);
        }
    }
    let stats = match gate {
        Gate::Bar { boundary_ms } => {
            let (min_open, max_open) = (mins[0], maxs[0]);
            let (min_close, max_close) = (mins[1], maxs[1]);
            let stats = GateStats::Bar {
                rows,
                bar_close_min: Some(min_close),
                bar_close_max: Some(max_close),
            };
            if min_open >= DEVELOPMENT_START_MS && max_close < boundary_ms {
                PartResult {
                    classification: Classification::FullDevelopment,
                    stats,
                    eligible: rows,
                    scanned: rows,
                }
            } else if max_open < DEVELOPMENT_START_MS || min_close >= boundary_ms {
                PartResult {
                    classification: Classification::FullConfirmation,
                    stats,
                    eligible: 0,
                    scanned: rows,
                }
            } else {
                return Ok(None);
            }
        }
        Gate::Tick { boundary_ns } => {
            let (min_event, max_event) = (mins[0], maxs[0]);
            let (min_received, max_received) = (mins[1], maxs[1]);
            let stats = GateStats::Tick {
                rows,
                event_min: Some(min_event),
                event_max: Some(max_event),
                received_min: Some(min_received),
                received_max: Some(max_received),
            };
            if min_event >= DEVELOPMENT_START_NS
                && min_received >= DEVELOPMENT_START_NS
                && max_event < boundary_ns
                && max_received < boundary_ns
            {
                PartResult {
                    classification: Classification::FullDevelopment,
                    stats,
                    eligible: rows,
                    scanned: rows,
                }
            } else if (max_event < DEVELOPMENT_START_NS && max_received < DEVELOPMENT_START_NS)
                || (min_event >= boundary_ns && min_received >= boundary_ns)
            {
                PartResult {
                    classification: Classification::FullConfirmation,
                    stats,
                    eligible: 0,
                    scanned: rows,
                }
            } else {
                return Ok(None);
            }
        }
    };
    Ok(Some(stats))
}

pub fn scan_part(
    path: &Path,
    gate: Gate,
    batch_size: usize,
) -> Result<PartResult, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let fields: Vec<_> = match gate {
        Gate::Bar { .. } => vec!["bar_open_ts", "bar_close_ts"],
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
        let columns = GateColumns::build(&batch, gate)?;
        for row in 0..batch.num_rows() {
            columns
                .push_stats(&mut result.stats, row)
                .map_err(io::Error::other)?;
            if columns.eligible(row)? {
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
        // Default row groups (1M rows) buffer multi-GB of encoded payload
        // columns before the first flush on wide mixed parts; smaller row
        // groups bound writer memory with no semantic effect.
        .set_max_row_group_size(32_768)
        .build();
    let output = File::create(target)?;
    let mut writer = ArrowWriter::try_new(output, schema.clone(), Some(props))?;
    let mut stats = GateStats::new(gate);
    for batch in reader {
        let batch = batch?;
        let columns = GateColumns::build(&batch, gate)?;
        let mut included = 0_u64;
        let mut mask_builder = Vec::with_capacity(batch.num_rows());
        for row in 0..batch.num_rows() {
            let keep = columns.eligible(row).map_err(io::Error::other)?;
            if keep {
                columns
                    .push_stats(&mut stats, row)
                    .map_err(io::Error::other)?;
                included += 1;
            }
            mask_builder.push(keep);
        }
        if included > 0 {
            let mask = BooleanArray::from(mask_builder);
            let filtered = filter_record_batch(&batch, &mask).map_err(io::Error::other)?;
            writer.write(&filtered)?;
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

#[allow(clippy::too_many_arguments)]
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
    h.update(DEVELOPMENT_START_MS.to_le_bytes());
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
    pub development_start_inclusive: String,
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

/// Full verification of one manifest file: row-scan gate re-check, payload hash,
/// and manifest stats comparison. Runs on a worker thread during verify_view.
fn verify_file(
    root: &Path,
    file: &ManifestFile,
    gate: Gate,
    tick_group: bool,
    batch_size: usize,
) -> Result<(PartResult, String), String> {
    let relative = Path::new(&file.logical_path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("source path traversal".into());
    }
    let path = root.join(relative);
    reject_reparse(&path).map_err(|e| e.to_string())?;
    let result = scan_part(&path, gate, batch_size).map_err(|e| e.to_string())?;
    if result.classification != Classification::FullDevelopment
        || result.eligible != result.scanned
        || result.scanned != file.development_row_count
    {
        return Err(format!(
            "confirmation leak or row mismatch: {}",
            file.logical_path
        ));
    }
    let boundary = match gate {
        Gate::Bar { boundary_ms } => boundary_ms,
        Gate::Tick { boundary_ns } => boundary_ns,
    };
    if !result.stats.maxes_before(boundary) {
        return Err(format!("gate violation: {}", file.logical_path));
    }
    if !result.stats.mins_at_or_after_start() {
        return Err(format!(
            "development start violation: {}",
            file.logical_path
        ));
    }
    let hash = file
        .sha256
        .as_deref()
        .ok_or_else(|| format!("missing payload hash: {}", file.logical_path))?;
    if hash_file(&path).map_err(|e| e.to_string())? != hash {
        return Err(format!("payload hash mismatch: {}", file.logical_path));
    }
    let expected_stats = matches!(&result.stats, GateStats::Tick { .. } if tick_group)
        || matches!(&result.stats, GateStats::Bar { .. } if !tick_group);
    if !expected_stats
        || result.stats != manifest_stats(file, tick_group).map_err(|e| e.to_string())?
    {
        return Err(format!("manifest stats mismatch: {}", file.logical_path));
    }
    Ok((result, hash.to_string()))
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
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, 8);
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
            expected.insert(file.logical_path.clone());
        }
        // Heavy per-file work (full row-scan verification + payload hashing),
        // parallel across files; results are assembled in manifest order.
        let checked: Vec<(PartResult, String)> = std::thread::scope(|scope| -> Result<_, String> {
            let chunk_size = files.len().div_ceil(workers).max(1);
            let handles: Vec<_> = files
                .chunks(chunk_size)
                .map(|chunk| {
                    scope.spawn(move || {
                        chunk
                            .iter()
                            .map(|file| verify_file(root, file, gate, group == "tick", batch_size))
                            .collect::<Result<Vec<_>, String>>()
                    })
                })
                .collect();
            let mut checked = Vec::new();
            for handle in handles {
                checked.extend(handle.join().expect("verify worker panicked")?);
            }
            Ok(checked)
        })
        .map_err(io::Error::other)?;
        for (file, checked) in files.iter().zip(checked) {
            let (result, hash) = checked;
            paths.push(file.logical_path.as_str());
            stats.push(result.stats);
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
    let hash_refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
    let identity = hash_view_identity(
        &manifest.scope_id,
        &manifest.source_manifest_identity,
        &manifest.payload_manifest_identity,
        BOUNDARY_MS,
        &paths,
        &stats,
        &hash_refs,
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
            && existing.development_start_inclusive == DEVELOPMENT_START_INCLUSIVE
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
    type Materialized = (String, Option<String>, RowStats, u64, u64);
    struct Job<'a> {
        group_index: usize,
        gate: Gate,
        part: &'a InventoryPart,
    }
    let mut group_order: Vec<String> = Vec::new();
    let mut jobs: Vec<Job> = Vec::new();
    for group in &inventory.sources {
        let (name, gate) = parse_scale(&group.scale)?;
        group_order.push(name);
        let group_index = group_order.len() - 1;
        for part in &group.parts {
            jobs.push(Job {
                group_index,
                gate,
                part,
            });
        }
    }
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, 8);
    // Phase A: classify every part. Footer statistics decide provably-in/out
    // parts with zero row reads; only straddling parts fall back to a row scan.
    let classify_one = |job: &Job| -> Result<PartResult, String> {
        let source = source_root.join(Path::new(&job.part.path));
        match classify_part_from_footer(&source, job.gate).map_err(|e| e.to_string())? {
            Some(result) => Ok(result),
            None => scan_part(&source, job.gate, batch_size).map_err(|e| e.to_string()),
        }
    };
    let classifications: Vec<PartResult> = std::thread::scope(|scope| -> Result<_, String> {
        let chunk_size = jobs.len().div_ceil(workers).max(1);
        let handles: Vec<_> = jobs
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(classify_one)
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut classifications = Vec::new();
        for handle in handles {
            classifications.extend(handle.join().expect("classify worker panicked")?);
        }
        Ok(classifications)
    })
    .map_err(io::Error::other)?;
    // Phase B: audit entries and declared-row checks (cheap, in order).
    let mut audit_parts = Vec::new();
    for (job, result) in jobs.iter().zip(&classifications) {
        if result.scanned != job.part.rows {
            return Err(io::Error::other(format!(
                "declared row count mismatch: {}",
                job.part.path
            ))
            .into());
        }
        let classification = match result.classification {
            Classification::FullDevelopment => "FULL_DEVELOPMENT",
            Classification::FullConfirmation => "FULL_CONFIRMATION",
            Classification::Mixed => "MIXED",
        };
        audit_parts.push(serde_json::json!({"logical_path": job.part.path, "classification": classification, "declared_source_rows": job.part.rows, "scanned_rows": result.scanned, "development_rows": result.eligible, "excluded_rows": result.scanned - result.eligible}));
    }
    // Phase C: materialize non-skipped parts in parallel (hardlink/copy or
    // row-filtered rewrite), hashing each target.
    let to_build: Vec<usize> = (0..jobs.len())
        .filter(|&index| classifications[index].classification != Classification::FullConfirmation)
        .collect();
    for &index in &to_build {
        let target = temp.join(Path::new(&jobs[index].part.path));
        fs::create_dir_all(target.parent().unwrap())?;
    }
    let materialize_one = |index: usize| -> Result<Materialized, String> {
        let job = &jobs[index];
        let result = &classifications[index];
        let logical_path = Path::new(&job.part.path);
        let target = temp.join(logical_path);
        match result.classification {
            Classification::FullDevelopment => {
                let source = source_root.join(logical_path);
                hard_link_or_copy(&source, &target, copy_full).map_err(|e| e.to_string())?;
                let bytes = fs::metadata(&source).map_err(|e| e.to_string())?.len();
                let kind = if copy_full {
                    "COPIED_FULL_DEVELOPMENT"
                } else {
                    "HARDLINK_FULL_DEVELOPMENT"
                };
                let sha = hash_file(&target).map_err(|e| e.to_string())?;
                Ok((kind.to_string(), Some(sha), result.stats, bytes, 0))
            }
            Classification::Mixed => {
                let source = source_root.join(logical_path);
                let stats = filter_part(&source, &target, job.gate, batch_size)
                    .map_err(|e| e.to_string())?;
                let bytes = fs::metadata(&target).map_err(|e| e.to_string())?.len();
                let sha = hash_file(&target).map_err(|e| e.to_string())?;
                Ok((
                    "FILTERED_MIXED_PART".to_string(),
                    Some(sha),
                    stats,
                    0,
                    bytes,
                ))
            }
            Classification::FullConfirmation => Err("unreachable".into()),
        }
    };
    let materialized: Vec<Option<Materialized>> =
        std::thread::scope(|scope| -> Result<_, String> {
            let mut results: Vec<Option<Materialized>> = (0..jobs.len()).map(|_| None).collect();
            let chunk_size = to_build.len().div_ceil(workers).max(1);
            let handles: Vec<_> = to_build
                .chunks(chunk_size)
                .map(|chunk| {
                    scope.spawn(move || {
                        chunk
                            .iter()
                            .map(|&index| materialize_one(index).map(|m| (index, m)))
                            .collect::<Result<Vec<_>, String>>()
                    })
                })
                .collect();
            for handle in handles {
                for (index, value) in handle.join().expect("materialize worker panicked")? {
                    results[index] = Some(value);
                }
            }
            Ok(results)
        })
        .map_err(io::Error::other)?;
    // Phase D: assemble manifest entries in inventory order.
    let mut groups: BTreeMap<String, Vec<ManifestFile>> = BTreeMap::new();
    for name in &group_order {
        groups.insert(name.clone(), Vec::new());
    }
    let mut paths = Vec::new();
    let mut stats = Vec::new();
    let mut hashes = Vec::new();
    let mut kinds = Vec::new();
    let mut bytes_linked = 0u64;
    let mut bytes_rewritten = 0u64;
    for (index, job) in jobs.iter().enumerate() {
        let result = &classifications[index];
        if result.classification == Classification::FullConfirmation {
            continue;
        }
        let (kind, sha, development_stats, linked, rewritten) = materialized[index]
            .as_ref()
            .ok_or_else(|| io::Error::other("missing materialization result"))?;
        bytes_linked += linked;
        bytes_rewritten += rewritten;
        let logical = job.part.path.as_str();
        paths.push(logical);
        stats.push(*development_stats);
        hashes.push(sha.clone().unwrap_or_default());
        kinds.push(kind.clone());
        groups
            .get_mut(&group_order[job.group_index])
            .unwrap()
            .push(ManifestFile {
                logical_path: logical.into(),
                materialization_kind: kind.clone(),
                development_row_count: result.eligible,
                bar_close_min: match development_stats {
                    GateStats::Bar { bar_close_min, .. } => *bar_close_min,
                    _ => None,
                },
                bar_close_max: match development_stats {
                    GateStats::Bar { bar_close_max, .. } => *bar_close_max,
                    _ => None,
                },
                event_min: match development_stats {
                    GateStats::Tick { event_min, .. } => *event_min,
                    _ => None,
                },
                event_max: match development_stats {
                    GateStats::Tick { event_max, .. } => *event_max,
                    _ => None,
                },
                received_min: match development_stats {
                    GateStats::Tick { received_min, .. } => *received_min,
                    _ => None,
                },
                received_max: match development_stats {
                    GateStats::Tick { received_max, .. } => *received_max,
                    _ => None,
                },
                sha256: sha.clone(),
                source_part: (result.classification == Classification::Mixed)
                    .then(|| logical.into()),
            });
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
    let manifest = Manifest { schema_version: 1, scope_id: scope.into(), instrument: "XAUUSD".into(), source_manifest_identity: source_id, payload_manifest_identity: payload_id, boundary_rule: "PHYSICAL DEVELOPMENT SCOPE FILTERS: development window [2025-07-31T16:15:00Z inclusive, 2026-05-05T12:39:00Z exclusive); bars bar_open_ts >= start and bar_close_ts < end; ticks event_ts_ns >= start and received_ts_ns >= start and event_ts_ns < end and received_ts_ns < end; null gating time is fatal".into(), development_start_inclusive: DEVELOPMENT_START_INCLUSIVE.into(), development_end_exclusive: "2026-05-05T12:39:00Z".into(), builder_code_identity: code.into(), source_groups: groups, logical_view_identity: identity.clone() };
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
