use research_contracts::BarScale;
use research_tape::{
    discover_columns, AnchorInstance, InstrumentConfig, LogicalOutputHasher, NativeScale,
    ProjectedParquetReader, ScanCounters, SourceInventory, SourcePart, SourceRef,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize)]
struct PilotSummary {
    pilot_id: String,
    instrument: String,
    anchor_detector: String,
    anchor_time_semantics: String,
    projected_columns: Vec<String>,
    rows_scanned: u64,
    anchors_emitted: u64,
    minimum_anchor_time: Option<i64>,
    maximum_anchor_time: Option<i64>,
    parts: u64,
    batches: u64,
    bytes: u64,
    elapsed_millis: u128,
    rows_per_second: f64,
    output_identity: String,
}

fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    hasher.update(fs::read(path)?);
    Ok(format!("{:x}", hasher.finalize()))
}
fn parts(value: &Value, key: &str) -> Result<Vec<SourcePart>, Box<dyn std::error::Error>> {
    Ok(value["bar_parts"][key]
        .as_array()
        .ok_or("missing bar parts")?
        .iter()
        .map(|part| {
            Ok(SourcePart {
                path: part["path"].as_str().ok_or("missing part path")?.into(),
                rows: part["rows"].as_u64().ok_or("missing part rows")?,
            })
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?)
}
fn utc_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("stage2 pilot failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config: InstrumentConfig = serde_json::from_str(&fs::read_to_string(
        "instruments/XAUUSD/instrument_config.json",
    )?)?;
    let lake_root = env::var(&config.lake_root_env)?;
    let lake_path = config.lake_path(&lake_root);
    let manifest_path = lake_path.join(&config.manifest);
    let payload_manifest_path = lake_path.join(&config.payload_manifest);
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest: Value = serde_json::from_str(&manifest_text)?;
    let payload_text = fs::read_to_string(&payload_manifest_path)?;
    let _payload: Value = serde_json::from_str(&payload_text)?;
    let native_scales = manifest["bar_parts"]
        .as_object()
        .ok_or("missing bar scales")?
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let pilot_parts = parts(&manifest, "15m")?;
    let first_path = lake_path.join(&pilot_parts[0].path);
    let columns = discover_columns(&first_path)?;
    let payload_columns = columns
        .iter()
        .filter(|column| column.starts_with("payload_"))
        .cloned()
        .collect();
    let inventory = SourceInventory {
        instrument: config.instrument.clone(),
        native_scales,
        bar_parts: pilot_parts.clone(),
        tick_parts: manifest["tick_parts"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|part| SourcePart {
                path: part["path"].as_str().unwrap_or_default().into(),
                rows: part["rows"].as_u64().unwrap_or_default(),
            })
            .collect(),
        bar_columns: columns.clone(),
        payload_columns,
        source_manifest_identity: hash_file(&manifest_path)?,
        payload_manifest_identity: hash_file(&payload_manifest_path)?,
    };
    let inventory_path = PathBuf::from("instruments/XAUUSD/source_inventory.json");
    fs::write(&inventory_path, serde_json::to_string_pretty(&inventory)?)?;
    let projected_columns = vec!["bar_close_ts".to_string()];
    let mut total = ScanCounters::default();
    let mut logical = LogicalOutputHasher::default();
    let mut count = 0;
    let mut minimum = None;
    let mut maximum = None;
    let mut previous = None;
    for part in &pilot_parts {
        let mut row_index = 0;
        let reader = ProjectedParquetReader::new(
            lake_path.join(&part.path),
            projected_columns.clone(),
            4096,
        )?;
        let mut scan = reader.scan()?;
        while let Some(batch) = scan.next() {
            let batch = batch?;
            let times = research_tape::batch_i64(&batch, "bar_close_ts")?;
            for row in 0..batch.num_rows() {
                let time = times.value(row);
                if previous.is_some_and(|p| time < p) {
                    return Err("bar input is not ordered".into());
                }
                previous = Some(time);
                let anchor = AnchorInstance {
                    anchor_id: format!("bar-close-{count}"),
                    detector_id: "canonical_bar_close".into(),
                    lifecycle_state: "completed_bar".into(),
                    native_scale: NativeScale::Bar(BarScale::M15),
                    anchor_time: time,
                    occur_time: None,
                    object_id: None,
                    source: SourceRef {
                        part: part.path.clone(),
                        row_index,
                    },
                };
                logical.update(&anchor);
                count += 1;
                row_index += 1;
                minimum = Some(minimum.map_or(time, |v: i64| v.min(time)));
                maximum = Some(maximum.map_or(time, |v: i64| v.max(time)));
            }
        }
        let metrics = scan.counters();
        total.rows += metrics.rows;
        total.batches += metrics.batches;
        total.bytes += metrics.bytes;
        total.parts += metrics.parts;
        total.elapsed_millis += metrics.elapsed_millis;
    }
    let rows_per_second = if total.elapsed_millis == 0 {
        0.0
    } else {
        total.rows as f64 / (total.elapsed_millis as f64 / 1000.0)
    };
    let summary = PilotSummary {
        pilot_id: "stage2-canonical-bar-close-15m".into(),
        instrument: config.instrument,
        anchor_detector: "canonical_bar_close".into(),
        anchor_time_semantics:
            "completed bar row is available at bar_close_ts; no detector lifecycle meaning asserted"
                .into(),
        projected_columns,
        rows_scanned: total.rows,
        anchors_emitted: count,
        minimum_anchor_time: minimum,
        maximum_anchor_time: maximum,
        parts: total.parts,
        batches: total.batches,
        bytes: total.bytes,
        elapsed_millis: total.elapsed_millis,
        rows_per_second,
        output_identity: logical.finish(),
    };
    if let Ok(path) = env::var("TRINITYR_RESEARCH_ARTIFACTS") {
        let output = PathBuf::from(path);
        fs::create_dir_all(&output)?;
        let manifest = research_tape::TapeManifest {
            tape_id: "stage2-canonical-bar-close-15m".into(),
            instrument: summary.instrument.clone(),
            pilot_id: summary.pilot_id.clone(),
            source_manifest_identity: inventory.source_manifest_identity,
            source_parts: pilot_parts.iter().map(|p| p.path.clone()).collect(),
            anchor_detector: summary.anchor_detector.clone(),
            anchor_time_semantics: summary.anchor_time_semantics.clone(),
            context_contract: "no detector context; causal primitives only".into(),
            row_count: summary.anchors_emitted,
            minimum_anchor_time: summary.minimum_anchor_time,
            maximum_anchor_time: summary.maximum_anchor_time,
            code_identity: env::var("DISCOVERYR_CODE_IDENTITY")
                .unwrap_or_else(|_| "research-tape-0.1.0".into()),
            generated_utc: utc_now(),
            output_identity: summary.output_identity.clone(),
        };
        fs::write(
            output.join("stage2_pilot_manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
