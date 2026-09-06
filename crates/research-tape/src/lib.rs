pub mod fvg_availability;

use arrow_array::{Array, Int64Array, RecordBatch, StringArray};
use parquet::arrow::{
    arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
    ProjectionMask,
};
use research_contracts::BarScale;
pub use research_contracts::NativeScale;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    fs::File,
    iter::Peekable,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub part: String,
    pub row_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorInstance {
    pub anchor_id: String,
    pub detector_id: String,
    pub lifecycle_state: String,
    pub native_scale: NativeScale,
    pub anchor_time: i64,
    pub occur_time: Option<i64>,
    pub object_id: Option<String>,
    pub source: SourceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextObservation {
    pub detector_id: String,
    pub native_scale: NativeScale,
    pub available_time: i64,
    pub occur_time: Option<i64>,
    pub object_id: Option<String>,
    pub source: SourceRef,
}

impl ContextObservation {
    pub fn try_new(
        detector_id: String,
        native_scale: NativeScale,
        available_time: Option<i64>,
        occur_time: Option<i64>,
        object_id: Option<String>,
        source: SourceRef,
    ) -> Result<Self, ContractError> {
        Ok(Self {
            detector_id,
            native_scale,
            available_time: available_time.ok_or(ContractError::MissingAvailabilityTime)?,
            occur_time,
            object_id,
            source,
        })
    }
}

pub trait HasAvailableTime {
    fn available_time(&self) -> i64;
}
impl HasAvailableTime for ContextObservation {
    fn available_time(&self) -> i64 {
        self.available_time
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    MissingAvailabilityTime,
    MissingCausalTimestamp(String),
    Authority(String),
    NonMonotonicAnchor,
    UnorderedContext,
    FutureContext {
        anchor_time: i64,
        available_time: i64,
    },
    InvalidWindow,
}
impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ContractError {}

pub struct AsOfLatestCursor<I>
where
    I: Iterator,
    I::Item: HasAvailableTime,
{
    iter: Peekable<I>,
    current: Option<I::Item>,
    last_anchor: Option<i64>,
    last_context: Option<i64>,
}
impl<I> AsOfLatestCursor<I>
where
    I: Iterator,
    I::Item: HasAvailableTime,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            current: None,
            last_anchor: None,
            last_context: None,
        }
    }
    pub fn advance_to(&mut self, anchor_time: i64) -> Result<Option<&I::Item>, ContractError> {
        if self
            .last_anchor
            .is_some_and(|previous| anchor_time < previous)
        {
            return Err(ContractError::NonMonotonicAnchor);
        }
        while let Some(next) = self.iter.peek() {
            let available = next.available_time();
            if self
                .last_context
                .is_some_and(|previous| available < previous)
            {
                return Err(ContractError::UnorderedContext);
            }
            if available > anchor_time {
                break;
            }
            let next = self.iter.next().expect("peeked context exists");
            self.last_context = Some(available);
            self.current = Some(next);
        }
        self.last_anchor = Some(anchor_time);
        if self
            .current
            .as_ref()
            .is_some_and(|value| value.available_time() > anchor_time)
        {
            return Err(ContractError::FutureContext {
                anchor_time,
                available_time: self.current.as_ref().unwrap().available_time(),
            });
        }
        Ok(self.current.as_ref())
    }
}

pub struct CausalWindowCursor<I>
where
    I: Iterator,
    I::Item: HasAvailableTime,
{
    iter: Peekable<I>,
    events: VecDeque<I::Item>,
    lookback: i64,
    last_anchor: Option<i64>,
    last_context: Option<i64>,
}
impl<I> CausalWindowCursor<I>
where
    I: Iterator,
    I::Item: HasAvailableTime,
{
    pub fn new(iter: I, lookback: i64) -> Result<Self, ContractError> {
        if lookback < 0 {
            return Err(ContractError::InvalidWindow);
        }
        Ok(Self {
            iter: iter.peekable(),
            events: VecDeque::new(),
            lookback,
            last_anchor: None,
            last_context: None,
        })
    }
    pub fn advance_to(&mut self, anchor_time: i64) -> Result<(), ContractError> {
        if self
            .last_anchor
            .is_some_and(|previous| anchor_time < previous)
        {
            return Err(ContractError::NonMonotonicAnchor);
        }
        while let Some(next) = self.iter.peek() {
            let available = next.available_time();
            if self
                .last_context
                .is_some_and(|previous| available < previous)
            {
                return Err(ContractError::UnorderedContext);
            }
            if available > anchor_time {
                break;
            }
            self.last_context = Some(available);
            self.events
                .push_back(self.iter.next().expect("peeked context exists"));
        }
        let lower = anchor_time
            .checked_sub(self.lookback)
            .ok_or(ContractError::InvalidWindow)?;
        while self
            .events
            .front()
            .is_some_and(|event| event.available_time() < lower)
        {
            self.events.pop_front();
        }
        self.last_anchor = Some(anchor_time);
        Ok(())
    }
    pub fn events(&self) -> impl Iterator<Item = &I::Item> {
        self.events.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstrumentConfig {
    pub instrument: String,
    pub lake_root_env: String,
    pub relative_path: String,
    pub manifest: String,
    pub payload_manifest: String,
}
impl InstrumentConfig {
    pub fn lake_path(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(&self.relative_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourcePart {
    pub path: String,
    pub rows: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhysicalField {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScaleInventory {
    pub scale: NativeScale,
    pub parts: Vec<SourcePart>,
    pub physical_schema: Vec<PhysicalField>,
    pub schema_verified_parts: Vec<String>,
    pub physical_time_coordinate_fields: Vec<String>,
    pub payload_columns: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceInventory {
    pub instrument: String,
    pub sources: Vec<ScaleInventory>,
    pub source_manifest_identity: String,
    pub payload_manifest_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TapeManifest {
    pub tape_id: String,
    pub instrument: String,
    pub pilot_id: String,
    pub source_manifest_identity: String,
    pub source_parts: Vec<String>,
    pub anchor_detector: String,
    pub anchor_time_semantics: String,
    pub availability_semantics_status: String,
    pub availability_semantics_source: String,
    pub context_contract: String,
    pub row_count: u64,
    pub minimum_anchor_time: Option<i64>,
    pub maximum_anchor_time: Option<i64>,
    pub code_identity: String,
    pub generated_utc: String,
    pub output_identity: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanCounters {
    pub rows: u64,
    pub batches: u64,
    pub source_file_bytes: u64,
    pub parts: u64,
    pub elapsed_millis: u128,
}

pub struct ProjectedParquetReader {
    path: PathBuf,
    columns: Vec<String>,
    batch_size: usize,
}
impl ProjectedParquetReader {
    pub fn new(
        path: impl Into<PathBuf>,
        columns: Vec<String>,
        batch_size: usize,
    ) -> Result<Self, ContractError> {
        if columns.is_empty() || batch_size == 0 {
            return Err(ContractError::InvalidWindow);
        }
        Ok(Self {
            path: path.into(),
            columns,
            batch_size,
        })
    }
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
    pub fn scan(&self) -> Result<ParquetScan, Box<dyn std::error::Error>> {
        let file = File::open(&self.path)?;
        let bytes = file.metadata()?.len();
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let schema = builder.schema().clone();
        let indices = self
            .columns
            .iter()
            .map(|column| schema.index_of(column))
            .collect::<Result<Vec<_>, _>>()?;
        let mask = ProjectionMask::roots(builder.parquet_schema(), indices);
        let reader = builder
            .with_projection(mask)
            .with_batch_size(self.batch_size)
            .build()?;
        Ok(ParquetScan {
            reader,
            counters: ScanCounters {
                source_file_bytes: bytes,
                parts: 1,
                ..Default::default()
            },
            started: Instant::now(),
        })
    }
}
pub struct ParquetScan {
    reader: ParquetRecordBatchReader,
    counters: ScanCounters,
    started: Instant,
}
impl Iterator for ParquetScan {
    type Item = Result<RecordBatch, arrow_schema::ArrowError>;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.next();
        if let Some(Ok(ref batch)) = result {
            self.counters.rows += batch.num_rows() as u64;
            self.counters.batches += 1;
        }
        if result.is_none() {
            self.counters.elapsed_millis = self.started.elapsed().as_millis();
        }
        result
    }
}
impl ParquetScan {
    pub fn counters(&self) -> &ScanCounters {
        &self.counters
    }
}

pub fn batch_i64<'a>(
    batch: &'a RecordBatch,
    column: &str,
) -> Result<&'a Int64Array, ContractError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<Int64Array>())
        .ok_or(ContractError::InvalidWindow)
}
pub fn required_i64(array: &Int64Array, row: usize, field: &str) -> Result<i64, ContractError> {
    if array.is_null(row) {
        return Err(ContractError::MissingCausalTimestamp(field.into()));
    }
    Ok(array.value(row))
}
pub fn batch_string<'a>(
    batch: &'a RecordBatch,
    column: &str,
) -> Result<&'a StringArray, ContractError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or(ContractError::InvalidWindow)
}
pub struct LogicalOutputHasher(Sha256);
impl Default for LogicalOutputHasher {
    fn default() -> Self {
        Self(Sha256::new())
    }
}
impl LogicalOutputHasher {
    pub fn update(&mut self, anchor: &AnchorInstance) {
        hash_bytes(&mut self.0, anchor.anchor_id.as_bytes());
        hash_bytes(&mut self.0, anchor.detector_id.as_bytes());
        hash_bytes(&mut self.0, anchor.lifecycle_state.as_bytes());
        hash_scale(&mut self.0, &anchor.native_scale);
        self.0.update(anchor.anchor_time.to_le_bytes());
        match anchor.occur_time {
            Some(value) => {
                self.0.update([1]);
                self.0.update(value.to_le_bytes());
            }
            None => self.0.update([0]),
        }
        match anchor.object_id.as_deref() {
            Some(value) => {
                self.0.update([1]);
                hash_bytes(&mut self.0, value.as_bytes());
            }
            None => self.0.update([0]),
        }
        hash_bytes(&mut self.0, anchor.source.part.as_bytes());
        self.0.update(anchor.source.row_index.to_le_bytes());
    }
    pub fn finish(self) -> String {
        format!("{:x}", self.0.finalize())
    }
}
pub fn logical_output_hash(anchors: &[AnchorInstance]) -> String {
    let mut hasher = LogicalOutputHasher::default();
    for anchor in anchors {
        hasher.update(anchor);
    }
    hasher.finish()
}

pub fn physical_schema(
    path: impl AsRef<Path>,
) -> Result<Vec<PhysicalField>, Box<dyn std::error::Error>> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?;
    Ok(builder
        .schema()
        .fields()
        .iter()
        .map(|field| PhysicalField {
            name: field.name().clone(),
            data_type: format!("{:?}", field.data_type()),
            nullable: field.is_nullable(),
        })
        .collect())
}
pub fn verify_physical_schema(
    scale: &NativeScale,
    part: &str,
    expected: &[PhysicalField],
    actual: &[PhysicalField],
) -> Result<(), ContractError> {
    if expected.len() != actual.len() {
        return Err(ContractError::Authority(format!(
            "schema mismatch for {scale:?} {part}: field count expected {}, actual {}",
            expected.len(),
            actual.len()
        )));
    }
    for (expected, actual) in expected.iter().zip(actual) {
        if expected != actual {
            return Err(ContractError::Authority(format!(
                "schema mismatch for {scale:?} {part} field {}: expected {:?}, actual {:?}",
                expected.name, expected, actual
            )));
        }
    }
    Ok(())
}
pub fn explicit_time_coordinates(
    scale: &NativeScale,
    schema: &[PhysicalField],
) -> Result<Vec<String>, ContractError> {
    let expected = match scale {
        NativeScale::Tick => ["event_ts_ns", "received_ts_ns"].as_slice(),
        NativeScale::Bar(_) => ["bar_open_ts", "bar_close_ts"].as_slice(),
    };
    expected
        .iter()
        .map(|name| {
            schema
                .iter()
                .find(|field| field.name == *name)
                .map(|_| (*name).into())
                .ok_or_else(|| {
                    ContractError::Authority(format!(
                        "physical time coordinate missing for {scale:?}: {name}"
                    ))
                })
        })
        .collect()
}

fn hash_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
fn hash_scale(hasher: &mut Sha256, scale: &NativeScale) {
    match scale {
        NativeScale::Tick => hasher.update([0]),
        NativeScale::Bar(value) => {
            hasher.update([
                1,
                match value {
                    BarScale::S15 => 0,
                    BarScale::S30 => 1,
                    BarScale::M1 => 2,
                    BarScale::M5 => 3,
                    BarScale::M15 => 4,
                    BarScale::H1 => 5,
                    BarScale::H4 => 6,
                },
            ]);
        }
    }
}
pub fn validate_source_inventory(inventory: &SourceInventory) -> Result<(), ContractError> {
    let expected = [
        NativeScale::Bar(BarScale::S15),
        NativeScale::Bar(BarScale::S30),
        NativeScale::Bar(BarScale::M1),
        NativeScale::Bar(BarScale::M5),
        NativeScale::Bar(BarScale::M15),
        NativeScale::Bar(BarScale::H1),
        NativeScale::Bar(BarScale::H4),
        NativeScale::Tick,
    ];
    if inventory.sources.len() != expected.len()
        || inventory.source_manifest_identity.is_empty()
        || inventory.payload_manifest_identity.is_empty()
        || expected.iter().any(|scale| {
            !inventory
                .sources
                .iter()
                .any(|source| &source.scale == scale)
        })
    {
        return Err(ContractError::InvalidWindow);
    }
    let mut all_paths = std::collections::HashSet::new();
    for source in &inventory.sources {
        let schema_names = source
            .physical_schema
            .iter()
            .map(|field| field.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        if source.parts.is_empty()
            || source.physical_schema.is_empty()
            || source.schema_verified_parts.len() != source.parts.len()
            || source
                .parts
                .iter()
                .map(|part| &part.path)
                .ne(source.schema_verified_parts.iter())
            || source.physical_time_coordinate_fields.is_empty()
            || source
                .payload_columns
                .iter()
                .any(|column| !schema_names.contains(column.as_str()))
            || source
                .physical_time_coordinate_fields
                .iter()
                .any(|column| !schema_names.contains(column.as_str()))
            || source
                .payload_columns
                .iter()
                .any(|column| source.physical_time_coordinate_fields.contains(column))
        {
            return Err(ContractError::InvalidWindow);
        }
        for part in &source.parts {
            if part.rows == 0 || part.path.is_empty() || !all_paths.insert(&part.path) {
                return Err(ContractError::InvalidWindow);
            }
        }
    }
    Ok(())
}

pub fn verify_row_count(part: &str, declared: u64, actual: u64) -> Result<(), ContractError> {
    if declared == actual {
        Ok(())
    } else {
        Err(ContractError::Authority(format!(
            "row count mismatch for {part}: declared {declared}, actual {actual}"
        )))
    }
}
pub fn validate_manifest_shape(manifest: &serde_json::Value) -> Result<(), ContractError> {
    let bars = manifest
        .get("bar_parts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            ContractError::Authority("required object missing or invalid: bar_parts".into())
        })?;
    let ticks = manifest
        .get("tick_parts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            ContractError::Authority("required array missing or invalid: tick_parts".into())
        })?;
    for scale in ["15s", "30s", "1m", "5m", "15m", "1h", "4h"] {
        let parts = bars
            .get(scale)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                ContractError::Authority(format!(
                    "required source group missing or invalid: {scale}"
                ))
            })?;
        if parts.is_empty() {
            return Err(ContractError::Authority(format!(
                "source group has no parts: {scale}"
            )));
        }
    }
    if ticks.is_empty() {
        return Err(ContractError::Authority(
            "source group has no parts: tick".into(),
        ));
    }
    for part in bars
        .values()
        .flat_map(serde_json::Value::as_array)
        .flatten()
        .chain(ticks)
    {
        if part
            .get("path")
            .and_then(serde_json::Value::as_str)
            .filter(|path| !path.is_empty())
            .is_none()
            || part
                .get("rows")
                .and_then(serde_json::Value::as_u64)
                .filter(|rows| *rows > 0)
                .is_none()
        {
            return Err(ContractError::Authority(
                "part requires non-empty path and positive rows".into(),
            ));
        }
    }
    Ok(())
}
pub fn validate_code_identity(value: Option<&str>) -> Result<String, ContractError> {
    let value = value
        .filter(|value| value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| {
            ContractError::Authority(
                "DISCOVERYR_CODE_IDENTITY must be a 40-character Git SHA".into(),
            )
        })?;
    Ok(value.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{Int32Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use parquet::arrow::ArrowWriter;
    use std::sync::Arc;
    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "research_tape_{name}_{}.parquet",
            std::process::id()
        ))
    }
    fn write_fixture(path: &std::path::Path, values: &[Option<i64>]) {
        let schema = Arc::new(Schema::new(vec![
            Field::new("bar_close_ts", DataType::Int64, true),
            Field::new("unrelated", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(values.to_vec())),
                Arc::new(StringArray::from(vec![Some("ignored"); values.len()])),
            ],
        )
        .unwrap();
        let file = File::create(path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }
    fn write_schema_fixture(
        path: &std::path::Path,
        timestamp_type: DataType,
        include_unrelated: bool,
        nullable: bool,
    ) {
        let mut fields = vec![
            Field::new("bar_open_ts", timestamp_type.clone(), nullable),
            Field::new("bar_close_ts", timestamp_type.clone(), nullable),
        ];
        if include_unrelated {
            fields.push(Field::new("unrelated", DataType::Utf8, true));
        }
        let schema = Arc::new(Schema::new(fields));
        let timestamp_array = || match timestamp_type.clone() {
            DataType::Int64 => Arc::new(Int64Array::from(vec![Some(1)])) as _,
            DataType::Int32 => Arc::new(Int32Array::from(vec![Some(1)])) as _,
            _ => panic!("test fixture only supports integer timestamps"),
        };
        let mut arrays = vec![timestamp_array(), timestamp_array()];
        if include_unrelated {
            arrays.push(Arc::new(StringArray::from(vec![Some("x")])) as _);
        }
        let batch = RecordBatch::try_new(schema.clone(), arrays).unwrap();
        let file = File::create(path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }
    fn context(time: i64) -> ContextObservation {
        ContextObservation::try_new(
            "range".into(),
            NativeScale::Bar(BarScale::M1),
            Some(time),
            Some(time - 1),
            None,
            SourceRef {
                part: "p".into(),
                row_index: time as u64,
            },
        )
        .unwrap()
    }
    #[test]
    fn causal_boundaries() {
        assert!(research_contracts::ensure_causal(10, 9).is_ok());
        assert!(research_contracts::ensure_causal(10, 10).is_ok());
        assert!(research_contracts::ensure_causal(10, 11).is_err());
    }
    #[test]
    fn asof_is_latest_causal_monotonic_and_scale_preserving() {
        let mut cursor =
            AsOfLatestCursor::new(vec![context(1), context(5), context(10)].into_iter());
        assert_eq!(cursor.advance_to(4).unwrap().unwrap().available_time, 1);
        assert_eq!(cursor.advance_to(5).unwrap().unwrap().available_time, 5);
        assert_eq!(
            cursor.advance_to(10).unwrap().unwrap().native_scale,
            NativeScale::Bar(BarScale::M1)
        );
        assert!(cursor.advance_to(9).is_err());
    }
    #[test]
    fn asof_never_selects_future_or_unordered_context() {
        let mut cursor = AsOfLatestCursor::new(vec![context(5), context(3)].into_iter());
        assert!(cursor.advance_to(4).unwrap().is_none());
        assert!(matches!(
            cursor.advance_to(5),
            Err(ContractError::UnorderedContext)
        ));
    }
    #[test]
    fn causal_window_boundaries_and_eviction() {
        let mut cursor = CausalWindowCursor::new(
            vec![
                context(70),
                context(80),
                context(90),
                context(100),
                context(101),
            ]
            .into_iter(),
            20,
        )
        .unwrap();
        cursor.advance_to(100).unwrap();
        assert_eq!(cursor.events().count(), 3);
        cursor.advance_to(101).unwrap();
        assert_eq!(cursor.events().count(), 3);
        assert!(cursor.events().all(|event| event.available_time() <= 101));
    }
    #[test]
    fn missing_availability_never_falls_back_to_occurrence() {
        assert!(ContextObservation::try_new(
            "x".into(),
            NativeScale::Tick,
            None,
            Some(1),
            None,
            SourceRef {
                part: "p".into(),
                row_index: 0
            }
        )
        .is_err());
    }
    #[test]
    fn native_scale_is_retained_across_resolutions() {
        let a = context(1);
        let b = ContextObservation::try_new(
            "x".into(),
            NativeScale::Bar(BarScale::H1),
            Some(2),
            None,
            None,
            SourceRef {
                part: "p".into(),
                row_index: 1,
            },
        )
        .unwrap();
        assert_ne!(a.native_scale, b.native_scale);
    }
    #[test]
    fn part_boundary_does_not_reset_asof_state() {
        let first = vec![context(10)];
        let second = vec![context(20)];
        let mut cursor = AsOfLatestCursor::new(first.into_iter().chain(second));
        assert_eq!(cursor.advance_to(15).unwrap().unwrap().available_time, 10);
        assert_eq!(cursor.advance_to(20).unwrap().unwrap().available_time, 20);
    }
    #[test]
    fn projected_reader_reads_only_requested_column() {
        let path = fixture_path("projection");
        write_fixture(&path, &[Some(10), Some(20)]);
        let reader = ProjectedParquetReader::new(path, vec!["bar_close_ts".into()], 4096).unwrap();
        assert_eq!(reader.columns(), &["bar_close_ts".to_string()]);
        let mut scan = reader.scan().unwrap();
        let batch = scan.next().unwrap().unwrap();
        assert_eq!(batch.num_columns(), 1);
        assert_eq!(
            batch_i64(&batch, "bar_close_ts").unwrap().len(),
            batch.num_rows()
        );
        std::fs::remove_file(reader.path).unwrap();
    }
    #[test]
    fn physical_schema_uses_explicit_coordinates_and_verifies_parts() {
        let first = fixture_path("schema-first");
        let second = fixture_path("schema-second");
        write_schema_fixture(&first, DataType::Int64, true, false);
        write_schema_fixture(&second, DataType::Int64, true, false);
        let expected = physical_schema(&first).unwrap();
        assert_eq!(
            explicit_time_coordinates(&NativeScale::Bar(BarScale::M1), &expected).unwrap(),
            vec!["bar_open_ts", "bar_close_ts"]
        );
        let payload_schema = vec![
            PhysicalField {
                name: "payload_volume_by_time".into(),
                data_type: "Float64".into(),
                nullable: true,
            },
            PhysicalField {
                name: "payload_time_context".into(),
                data_type: "Utf8".into(),
                nullable: true,
            },
        ];
        assert!(
            explicit_time_coordinates(&NativeScale::Bar(BarScale::M1), &payload_schema).is_err()
        );
        assert!(verify_physical_schema(
            &NativeScale::Bar(BarScale::M1),
            "second",
            &expected,
            &physical_schema(&second).unwrap()
        )
        .is_ok());
        std::fs::remove_file(&first).unwrap();
        std::fs::remove_file(&second).unwrap();
    }
    #[test]
    fn physical_schema_mismatch_fails_closed_for_type_field_and_nullability() {
        let expected_path = fixture_path("schema-expected");
        let type_path = fixture_path("schema-type");
        let field_path = fixture_path("schema-field");
        let nullability_path = fixture_path("schema-nullability");
        write_schema_fixture(&expected_path, DataType::Int64, true, false);
        write_schema_fixture(&type_path, DataType::Int32, true, false);
        write_schema_fixture(&field_path, DataType::Int64, false, false);
        write_schema_fixture(&nullability_path, DataType::Int64, true, true);
        let expected = physical_schema(&expected_path).unwrap();
        for path in [type_path, field_path, nullability_path] {
            assert!(verify_physical_schema(
                &NativeScale::Bar(BarScale::M1),
                "synthetic",
                &expected,
                &physical_schema(&path).unwrap()
            )
            .is_err());
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_file(expected_path).unwrap();
    }
    #[test]
    fn synthetic_parts_preserve_continuity_and_reject_null_timestamps() {
        let first = fixture_path("part0");
        let second = fixture_path("part1");
        write_fixture(&first, &[Some(10), Some(20)]);
        write_fixture(&second, &[Some(30), Some(40)]);
        let mut values = Vec::new();
        for (path, part) in [(&first, "part-00000"), (&second, "part-00001")] {
            let reader =
                ProjectedParquetReader::new(path.clone(), vec!["bar_close_ts".into()], 1).unwrap();
            let scan = reader.scan().unwrap();
            for batch in scan {
                let batch = batch.unwrap();
                let times = batch_i64(&batch, "bar_close_ts").unwrap();
                for row in 0..batch.num_rows() {
                    values.push(
                        ContextObservation::try_new(
                            "x".into(),
                            NativeScale::Bar(BarScale::M1),
                            Some(required_i64(times, row, "bar_close_ts").unwrap()),
                            None,
                            None,
                            SourceRef {
                                part: part.into(),
                                row_index: values.len() as u64,
                            },
                        )
                        .unwrap(),
                    );
                }
            }
        }
        let mut cursor = AsOfLatestCursor::new(values.into_iter());
        assert_eq!(cursor.advance_to(30).unwrap().unwrap().available_time, 30);
        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
        let null_path = fixture_path("null");
        write_fixture(&null_path, &[Some(10), None]);
        let reader =
            ProjectedParquetReader::new(null_path.clone(), vec!["bar_close_ts".into()], 2).unwrap();
        let mut scan = reader.scan().unwrap();
        let batch = scan.next().unwrap().unwrap();
        let times = batch_i64(&batch, "bar_close_ts").unwrap();
        assert!(required_i64(times, 1, "bar_close_ts").is_err());
        std::fs::remove_file(null_path).unwrap();
    }
    #[test]
    fn duplicate_output_is_deterministic() {
        let values = vec![AnchorInstance {
            anchor_id: "a".into(),
            detector_id: "bar_close".into(),
            lifecycle_state: "completed_bar".into(),
            native_scale: NativeScale::Bar(BarScale::M1),
            anchor_time: 1,
            occur_time: None,
            object_id: None,
            source: SourceRef {
                part: "p".into(),
                row_index: 0,
            },
        }];
        assert_eq!(logical_output_hash(&values), logical_output_hash(&values));
    }
    #[test]
    fn logical_hash_changes_for_each_identity_field() {
        let base = AnchorInstance {
            anchor_id: "a".into(),
            detector_id: "d".into(),
            lifecycle_state: "s".into(),
            native_scale: NativeScale::Bar(BarScale::M1),
            anchor_time: 1,
            occur_time: None,
            object_id: None,
            source: SourceRef {
                part: "p".into(),
                row_index: 0,
            },
        };
        let mut changed = base.clone();
        changed.detector_id = "other".into();
        assert_ne!(
            logical_output_hash(std::slice::from_ref(&base)),
            logical_output_hash(&[changed])
        );
        let mut changed = base.clone();
        changed.lifecycle_state = "other".into();
        assert_ne!(
            logical_output_hash(std::slice::from_ref(&base)),
            logical_output_hash(&[changed])
        );
        let mut changed = base.clone();
        changed.native_scale = NativeScale::Tick;
        assert_ne!(
            logical_output_hash(std::slice::from_ref(&base)),
            logical_output_hash(&[changed])
        );
        let mut changed = base.clone();
        changed.object_id = Some("o".into());
        assert_ne!(
            logical_output_hash(std::slice::from_ref(&base)),
            logical_output_hash(&[changed])
        );
        let mut changed = base;
        changed.occur_time = Some(1);
        assert_ne!(
            logical_output_hash(&[changed.clone()]),
            logical_output_hash(&[AnchorInstance {
                occur_time: None,
                ..changed
            }])
        );
    }
    #[test]
    fn authority_and_identity_validation_fail_closed() {
        assert!(validate_manifest_shape(&serde_json::json!({})).is_err());
        assert!(verify_row_count("p", 2, 1).is_err());
        assert!(validate_code_identity(None).is_err());
        assert!(validate_code_identity(Some("0123456789012345678901234567890123456789")).is_ok());
    }
}
