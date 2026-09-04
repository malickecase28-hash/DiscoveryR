use arrow_array::{Array, Int64Array, RecordBatch, StringArray};
use parquet::arrow::{
    arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
    ProjectionMask,
};
use research_contracts::BarScale;
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
pub enum NativeScale {
    Tick,
    Bar(BarScale),
}

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
pub struct SourceInventory {
    pub instrument: String,
    pub native_scales: Vec<String>,
    pub bar_parts: Vec<SourcePart>,
    pub tick_parts: Vec<SourcePart>,
    pub bar_columns: Vec<String>,
    pub payload_columns: Vec<String>,
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
    pub bytes: u64,
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
                bytes,
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
        self.0.update(anchor.anchor_id.as_bytes());
        self.0.update(anchor.anchor_time.to_le_bytes());
        self.0.update(anchor.source.part.as_bytes());
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

pub fn discover_columns(path: impl AsRef<Path>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?;
    Ok(builder
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().clone())
        .collect())
}

pub fn validate_source_inventory(inventory: &SourceInventory) -> Result<(), ContractError> {
    if inventory.native_scales.is_empty()
        || inventory.bar_parts.is_empty()
        || inventory.bar_columns.is_empty()
    {
        return Err(ContractError::InvalidWindow);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../analytical_lake/fusion_markets/xauusd/15m/part-00000.parquet");
        let reader = ProjectedParquetReader::new(path, vec!["bar_close_ts".into()], 4096).unwrap();
        assert_eq!(reader.columns(), &["bar_close_ts".to_string()]);
        let mut scan = reader.scan().unwrap();
        let batch = scan.next().unwrap().unwrap();
        assert_eq!(batch.num_columns(), 1);
        assert_eq!(
            batch_i64(&batch, "bar_close_ts").unwrap().len(),
            batch.num_rows()
        );
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
}
