use crate::{batch_i64, required_i64, ProjectedParquetReader};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Component, Path, PathBuf},
};

pub const DEFAULT_BATCH_SIZE: usize = 65_536;
pub const DEFAULT_TIMEFRAMES: [(&str, u64); 7] = [
    ("15s", 15),
    ("30s", 30),
    ("1m", 60),
    ("5m", 300),
    ("15m", 900),
    ("1h", 3_600),
    ("4h", 14_400),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick {
    pub event_time_ns: i64,
    pub received_time_ns: i64,
    pub source_sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailabilityRecord {
    pub timeframe: String,
    pub bar_close_ts_ms: i64,
    pub bar_close_ts_ns: i64,
    pub available_time_ns: i64,
    pub trigger_event_time_ns: i64,
    pub trigger_source_sequence: i64,
    pub trigger_source_part: String,
    pub trigger_row_index: u64,
    pub availability_kind: &'static str,
}

#[derive(Debug)]
pub enum SidecarError {
    Invalid(String),
    Io(io::Error),
}

impl std::fmt::Display for SidecarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for SidecarError {}
impl From<io::Error> for SidecarError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone)]
pub struct TimeframeSpec {
    pub label: String,
    duration_ns: i64,
}

impl TimeframeSpec {
    pub fn new(label: impl Into<String>, seconds: u64) -> Result<Self, SidecarError> {
        let label = label.into();
        if label.is_empty() || seconds == 0 {
            return Err(SidecarError::Invalid("invalid timeframe".into()));
        }
        let duration_ns = i64::try_from(seconds)
            .ok()
            .and_then(|seconds| seconds.checked_mul(1_000_000_000))
            .ok_or_else(|| SidecarError::Invalid("timeframe duration overflow".into()))?;
        Ok(Self { label, duration_ns })
    }
}

#[derive(Debug)]
struct FormingBar {
    start_ns: i64,
}

#[derive(Debug)]
pub struct AvailabilitySidecar {
    timeframes: Vec<TimeframeSpec>,
    forming: Vec<Option<FormingBar>>,
    last_event_time_ns: Option<i64>,
    last_source_sequence: Option<i64>,
}

impl AvailabilitySidecar {
    pub fn new(timeframes: Vec<TimeframeSpec>) -> Result<Self, SidecarError> {
        if timeframes.is_empty() {
            return Err(SidecarError::Invalid("no timeframes supplied".into()));
        }
        let mut labels = std::collections::HashSet::new();
        for timeframe in &timeframes {
            if !labels.insert(&timeframe.label) {
                return Err(SidecarError::Invalid(format!(
                    "duplicate timeframe: {}",
                    timeframe.label
                )));
            }
        }
        Ok(Self {
            forming: (0..timeframes.len()).map(|_| None).collect(),
            timeframes,
            last_event_time_ns: None,
            last_source_sequence: None,
        })
    }

    pub fn ingest<F>(
        &mut self,
        tick: Tick,
        source_part: &str,
        source_row_index: u64,
        emit: &mut F,
    ) -> Result<(), SidecarError>
    where
        F: FnMut(AvailabilityRecord) -> io::Result<()>,
    {
        if tick.event_time_ns < 0
            || tick.received_time_ns < 0
            || tick.source_sequence < 0
            || self
                .last_event_time_ns
                .is_some_and(|previous| tick.event_time_ns < previous)
            || self
                .last_source_sequence
                .is_some_and(|previous| tick.source_sequence <= previous)
        {
            return Err(SidecarError::Invalid(
                "development tick ordering or timestamp invariant failed".into(),
            ));
        }

        for (index, timeframe) in self.timeframes.iter().enumerate() {
            let bucket_start = (tick.event_time_ns / timeframe.duration_ns) * timeframe.duration_ns;
            let Some(forming) = self.forming[index].as_mut() else {
                self.forming[index] = Some(FormingBar {
                    start_ns: bucket_start,
                });
                continue;
            };
            if bucket_start < forming.start_ns {
                return Err(SidecarError::Invalid(format!(
                    "bucket regression for {}",
                    timeframe.label
                )));
            }
            if bucket_start == forming.start_ns {
                continue;
            }

            let bar_close_ts_ns = forming
                .start_ns
                .checked_add(timeframe.duration_ns)
                .ok_or_else(|| SidecarError::Invalid("bar timestamp overflow".into()))?;
            emit(AvailabilityRecord {
                timeframe: timeframe.label.clone(),
                bar_close_ts_ms: bar_close_ts_ns / 1_000_000,
                bar_close_ts_ns,
                available_time_ns: tick.received_time_ns,
                trigger_event_time_ns: tick.event_time_ns,
                trigger_source_sequence: tick.source_sequence,
                trigger_source_part: source_part.into(),
                trigger_row_index: source_row_index,
                availability_kind: "BOUNDARY_CROSSING_TICK",
            })?;
            forming.start_ns = bucket_start;
        }

        self.last_event_time_ns = Some(tick.event_time_ns);
        self.last_source_sequence = Some(tick.source_sequence);
        Ok(())
    }

    pub fn timeframes(&self) -> impl Iterator<Item = &TimeframeSpec> {
        self.timeframes.iter()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SidecarSummary {
    pub artifact_type: &'static str,
    pub artifact_version: &'static str,
    pub availability_rule: &'static str,
    pub development_view_manifest_sha256: String,
    pub tick_parts: Vec<String>,
    pub projected_columns: [&'static str; 3],
    pub timeframes: Vec<String>,
    pub input_rows: u64,
    pub sidecar_rows: u64,
    pub sidecar_sha256: String,
    pub end_of_stream_policy: &'static str,
}

#[derive(Debug, Deserialize)]
struct ViewManifest {
    source_groups: BTreeMap<String, Vec<ViewFile>>,
}

#[derive(Debug, Deserialize)]
struct ViewFile {
    logical_path: String,
    development_row_count: u64,
}

struct HashingWriter<W> {
    inner: W,
    hasher: Sha256,
}

impl<W> HashingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
        }
    }
    fn finish(self) -> (W, String) {
        (self.inner, format!("{:x}", self.hasher.finalize()))
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(bytes)?;
        self.hasher.update(&bytes[..written]);
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn safe_relative_path(value: &str) -> Result<PathBuf, SidecarError> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        return Err(SidecarError::Invalid(format!(
            "unsafe development-view path: {value}"
        )));
    }
    Ok(path.to_owned())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn parse_timeframes(value: &str) -> Result<Vec<TimeframeSpec>, SidecarError> {
    value
        .split(',')
        .map(str::trim)
        .map(|label| {
            let seconds = DEFAULT_TIMEFRAMES
                .iter()
                .find(|(known, _)| *known == label)
                .map(|(_, seconds)| *seconds)
                .ok_or_else(|| SidecarError::Invalid(format!("unsupported timeframe: {label}")))?;
            TimeframeSpec::new(label, seconds)
        })
        .collect()
}

pub fn default_timeframes() -> Result<Vec<TimeframeSpec>, SidecarError> {
    DEFAULT_TIMEFRAMES
        .iter()
        .map(|(label, seconds)| TimeframeSpec::new(*label, *seconds))
        .collect()
}

pub fn materialize(
    view_root: &Path,
    output_path: &Path,
    timeframes: Vec<TimeframeSpec>,
) -> Result<SidecarSummary, Box<dyn std::error::Error>> {
    if output_path.exists() {
        return Err(format!("sidecar output already exists: {}", output_path.display()).into());
    }
    let manifest_bytes = fs::read(view_root.join("view_manifest.json"))?;
    let manifest: ViewManifest = serde_json::from_slice(&manifest_bytes)?;
    let tick_parts = manifest
        .source_groups
        .get("tick")
        .ok_or("development view has no tick group")?;
    if tick_parts.is_empty() {
        return Err("development view has no tick parts".into());
    }
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temp_path = output_path.with_extension(format!("partial-{}", std::process::id()));
    if temp_path.exists() {
        return Err(format!(
            "temporary sidecar output already exists: {}",
            temp_path.display()
        )
        .into());
    }

    let result = (|| -> Result<SidecarSummary, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        let mut writer = BufWriter::new(HashingWriter::new(file));
        let mut sidecar = AvailabilitySidecar::new(timeframes)?;
        let mut input_rows = 0u64;
        let mut sidecar_rows = 0u64;
        for part in tick_parts {
            let relative = safe_relative_path(&part.logical_path)?;
            let path = view_root.join(&relative);
            let reader = ProjectedParquetReader::new(
                path,
                ["event_ts_ns", "received_ts_ns", "source_sequence"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                DEFAULT_BATCH_SIZE,
            )?;
            let mut scan = reader.scan()?;
            let mut row = 0u64;
            while let Some(batch) = scan.next() {
                let batch = batch?;
                let event = batch_i64(&batch, "event_ts_ns")?;
                let received = batch_i64(&batch, "received_ts_ns")?;
                let sequence = batch_i64(&batch, "source_sequence")?;
                for index in 0..batch.num_rows() {
                    let tick = Tick {
                        event_time_ns: required_i64(event, index, "event_ts_ns")?,
                        received_time_ns: required_i64(received, index, "received_ts_ns")?,
                        source_sequence: required_i64(sequence, index, "source_sequence")?,
                    };
                    let mut write_record = |record: AvailabilityRecord| {
                        serde_json::to_writer(&mut writer, &record).map_err(io::Error::other)?;
                        writer.write_all(b"\n")?;
                        sidecar_rows += 1;
                        Ok(())
                    };
                    sidecar.ingest(tick, &part.logical_path, row, &mut write_record)?;
                    row += 1;
                    input_rows += 1;
                }
            }
            if row != part.development_row_count {
                return Err(format!(
                    "row count mismatch for {}: declared {}, actual {}",
                    part.logical_path, part.development_row_count, row
                )
                .into());
            }
        }
        writer.flush()?;
        let hashing_writer = writer.into_inner().map_err(|error| error.into_error())?;
        let (mut file, sidecar_sha256) = hashing_writer.finish();
        file.flush()?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, output_path)?;
        let timeframes = sidecar
            .timeframes()
            .map(|timeframe| timeframe.label.clone())
            .collect();
        Ok(SidecarSummary {
            artifact_type: "FVG_BAR_AVAILABILITY_SIDECAR",
            artifact_version: "V1",
            availability_rule: "For a normal completed bar, available_time_ns is received_time_ns of the first source tick whose event_time_ns enters a later bucket; source_sequence is the equal-receipt causal tie-break. bar_close_ts_ns is the temporal boundary only.",
            development_view_manifest_sha256: sha256_bytes(&manifest_bytes),
            tick_parts: tick_parts.iter().map(|part| part.logical_path.clone()).collect(),
            projected_columns: ["event_ts_ns", "received_ts_ns", "source_sequence"],
            timeframes,
            input_rows,
            sidecar_rows,
            sidecar_sha256,
            end_of_stream_policy: "No synthetic tail bar is emitted; a forming end-of-stream bar has no boundary-crossing receipt coordinate and remains unavailable in this sidecar.",
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(event_time_ns: i64, received_time_ns: i64, source_sequence: i64) -> Tick {
        Tick {
            event_time_ns,
            received_time_ns,
            source_sequence,
        }
    }

    fn collect(sidecar: &mut AvailabilitySidecar, ticks: &[Tick]) -> Vec<AvailabilityRecord> {
        let mut records = Vec::new();
        for (row, tick) in ticks.iter().copied().enumerate() {
            sidecar
                .ingest(tick, "tick/part-00000.parquet", row as u64, &mut |record| {
                    records.push(record);
                    Ok(())
                })
                .unwrap();
        }
        records
    }

    fn sidecar(labels: &[(&str, u64)]) -> AvailabilitySidecar {
        AvailabilitySidecar::new(
            labels
                .iter()
                .map(|(label, seconds)| TimeframeSpec::new(*label, *seconds).unwrap())
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn first_tick_and_end_of_stream_do_not_create_unlawful_rows() {
        let mut sidecar = sidecar(&[("15s", 15)]);
        let records = collect(
            &mut sidecar,
            &[tick(0, 100, 1), tick(15_000_000_000, 200, 2)],
        );
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bar_close_ts_ns, 15_000_000_000);
        assert_eq!(records[0].available_time_ns, 200);
        assert_eq!(records[0].trigger_source_sequence, 2);
        assert_eq!(records[0].trigger_row_index, 1);
    }

    #[test]
    fn exact_boundary_tick_closes_prior_bucket() {
        let mut sidecar = sidecar(&[("1m", 60)]);
        let records = collect(&mut sidecar, &[tick(1, 10, 1), tick(60_000_000_000, 20, 2)]);
        assert_eq!(records[0].bar_close_ts_ms, 60_000);
        assert_eq!(records[0].trigger_event_time_ns, 60_000_000_000);
    }

    #[test]
    fn gaps_do_not_synthesize_empty_bars_and_multiple_timeframes_are_independent() {
        let mut sidecar = sidecar(&[("15s", 15), ("30s", 30), ("1m", 60)]);
        let records = collect(&mut sidecar, &[tick(0, 10, 1), tick(60_000_000_000, 20, 2)]);
        assert_eq!(
            records
                .iter()
                .map(|record| (record.timeframe.as_str(), record.bar_close_ts_ns))
                .collect::<Vec<_>>(),
            vec![
                ("15s", 15_000_000_000),
                ("30s", 30_000_000_000),
                ("1m", 60_000_000_000)
            ]
        );
    }

    #[test]
    fn equal_receipt_time_keeps_source_sequence_tie_break() {
        let mut sidecar = sidecar(&[("15s", 15), ("30s", 30)]);
        let records = collect(
            &mut sidecar,
            &[
                tick(0, 10, 1),
                tick(15_000_000_000, 20, 2),
                tick(30_000_000_000, 20, 3),
            ],
        );
        assert_eq!(records[0].available_time_ns, 20);
        assert_eq!(records[0].trigger_source_sequence, 2);
        assert!(records[1..]
            .iter()
            .all(|record| record.available_time_ns == 20));
        assert!(records[1..]
            .iter()
            .all(|record| record.trigger_source_sequence == 3));
    }

    #[test]
    fn ordering_regressions_fail_closed() {
        let mut sidecar = sidecar(&[("15s", 15)]);
        sidecar
            .ingest(tick(1, 10, 2), "p", 0, &mut |_| Ok(()))
            .unwrap();
        assert!(sidecar
            .ingest(tick(0, 11, 3), "p", 1, &mut |_| Ok(()))
            .is_err());
        assert!(sidecar
            .ingest(tick(2, 12, 1), "p", 2, &mut |_| Ok(()))
            .is_err());
    }
}
