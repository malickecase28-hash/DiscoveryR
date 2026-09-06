use research_contracts::BarScale;
use research_tape::{
    batch_i64, explicit_time_coordinates, physical_schema, required_i64, validate_code_identity,
    validate_manifest_shape, verify_physical_schema, verify_row_count, AnchorInstance,
    ContractError, InstrumentConfig, LogicalOutputHasher, NativeScale, ProjectedParquetReader,
    ScaleInventory, ScanCounters, SourceInventory, SourcePart, SourceRef, TapeManifest,
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
    availability_semantics_status: String,
    availability_semantics_source: String,
    projected_columns: Vec<String>,
    rows_scanned: u64,
    anchors_emitted: u64,
    minimum_anchor_time: Option<i64>,
    maximum_anchor_time: Option<i64>,
    parts: u64,
    batches: u64,
    source_file_bytes: u64,
    elapsed_millis: u128,
    rows_per_second: f64,
    output_identity: String,
    tick_smoke_part: String,
    tick_smoke_projected_column: String,
    tick_smoke_rows: u64,
    tick_smoke_batches: u64,
    tick_smoke_source_file_bytes: u64,
    tick_smoke_elapsed_millis: u128,
    tick_smoke_rows_per_second: f64,
}

fn authority_error(message: impl Into<String>) -> ContractError {
    ContractError::Authority(message.into())
}
fn required_object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, ContractError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| authority_error(format!("required object missing or invalid: {field}")))
}
fn required_array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, ContractError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| authority_error(format!("required array missing or invalid: {field}")))
}
fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, ContractError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| authority_error(format!("required string missing or invalid: {field}")))
}
fn required_u64(value: &Value, field: &str) -> Result<u64, ContractError> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            authority_error(format!(
                "required positive row count missing or invalid: {field}"
            ))
        })
}
fn parse_part_values(parts: &[Value], label: &str) -> Result<Vec<SourcePart>, ContractError> {
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            Ok(SourcePart {
                path: required_string(part, "path")
                    .map_err(|_| authority_error(format!("{label}[{index}].path")))?
                    .into(),
                rows: required_u64(part, "rows")
                    .map_err(|_| authority_error(format!("{label}[{index}].rows")))?,
            })
        })
        .collect()
}
fn parse_parts(value: &Value, label: &str) -> Result<Vec<SourcePart>, ContractError> {
    parse_part_values(required_array(value, label)?, label)
}
fn bar_scale(name: &str) -> Result<BarScale, ContractError> {
    match name {
        "15s" => Ok(BarScale::S15),
        "30s" => Ok(BarScale::S30),
        "1m" => Ok(BarScale::M1),
        "5m" => Ok(BarScale::M5),
        "15m" => Ok(BarScale::M15),
        "1h" => Ok(BarScale::H1),
        "4h" => Ok(BarScale::H4),
        _ => Err(authority_error(format!("unsupported bar scale: {name}"))),
    }
}
fn source_group(
    root: &Path,
    scale: NativeScale,
    parts: Vec<SourcePart>,
) -> Result<ScaleInventory, Box<dyn std::error::Error>> {
    let verified_schema = physical_schema(root.join(&parts[0].path))?;
    for part in parts.iter().skip(1) {
        verify_physical_schema(
            &scale,
            &part.path,
            &verified_schema,
            &physical_schema(root.join(&part.path))?,
        )?;
    }
    let physical_time_coordinate_fields = explicit_time_coordinates(&scale, &verified_schema)?;
    let payload_columns = verified_schema
        .iter()
        .map(|field| field.name.clone())
        .filter(|column| column.starts_with("payload_") || column == "payload_json")
        .collect();
    Ok(ScaleInventory {
        scale,
        schema_verified_parts: parts.iter().map(|part| part.path.clone()).collect(),
        parts,
        physical_schema: verified_schema,
        physical_time_coordinate_fields,
        payload_columns,
    })
}
fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    hasher.update(fs::read(path)?);
    Ok(format!("{:x}", hasher.finalize()))
}
fn utc_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
fn scan_part(
    path: &Path,
    part: &SourcePart,
    column: &str,
    batch_size: usize,
    mut on_time: impl FnMut(i64, u64) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<ScanCounters, Box<dyn std::error::Error>> {
    let reader = ProjectedParquetReader::new(path, vec![column.into()], batch_size)?;
    let mut scan = reader.scan()?;
    let mut row = 0;
    for batch in scan.by_ref() {
        let batch = batch?;
        let times = batch_i64(&batch, column)?;
        for index in 0..batch.num_rows() {
            on_time(required_i64(times, index, column)?, row)?;
            row += 1;
        }
    }
    let metrics = scan.counters().clone();
    verify_row_count(&part.path, part.rows, row)?;
    Ok(metrics)
}

#[derive(Debug, Serialize)]
struct PilotOutput {
    summary: PilotSummary,
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
    let lake_root = env::var(&config.lake_root_env)
        .map_err(|_| authority_error(format!("missing {}", config.lake_root_env)))?;
    let lake_path = config.lake_path(&lake_root);
    let manifest_path = lake_path.join(&config.manifest);
    let payload_path = lake_path.join(&config.payload_manifest);
    let manifest: Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    validate_manifest_shape(&manifest)?;
    let payload: Value = serde_json::from_str(&fs::read_to_string(&payload_path)?)?;
    let bar_map = required_object(&manifest, "bar_parts")?;
    let tick_parts = parse_parts(&manifest, "tick_parts")?;
    if payload.as_object().is_none() {
        return Err(authority_error("payload manifest must be an object").into());
    }
    let mut sources = Vec::new();
    for (name, value) in bar_map {
        let parts = parse_part_values(
            value.as_array().ok_or_else(|| {
                authority_error(format!("required array missing or invalid: {name}"))
            })?,
            name,
        )?;
        sources.push(source_group(
            &lake_path,
            NativeScale::Bar(bar_scale(name)?),
            parts,
        )?);
    }
    sources.push(source_group(
        &lake_path,
        NativeScale::Tick,
        tick_parts.clone(),
    )?);
    let inventory = SourceInventory {
        instrument: config.instrument.clone(),
        sources,
        source_manifest_identity: hash_file(&manifest_path)?,
        payload_manifest_identity: hash_file(&payload_path)?,
    };
    research_tape::validate_source_inventory(&inventory)
        .map_err(|e| authority_error(format!("invalid source inventory: {e:?}")))?;
    fs::write(
        "instruments/XAUUSD/source_inventory.json",
        serde_json::to_string_pretty(&inventory)?,
    )?;
    let pilot_source = inventory
        .sources
        .iter()
        .find(|source| source.scale == NativeScale::Bar(BarScale::M15))
        .ok_or_else(|| authority_error("15m source group missing"))?;
    let projected_columns = vec!["bar_close_ts".into()];
    let mut total = ScanCounters::default();
    let mut logical = LogicalOutputHasher::default();
    let mut count = 0;
    let mut minimum = None;
    let mut maximum = None;
    let mut previous = None;
    for part in &pilot_source.parts {
        let metrics = scan_part(
            &lake_path.join(&part.path),
            part,
            "bar_close_ts",
            4096,
            |time, row| {
                if previous.is_some_and(|value| time < value) {
                    return Err(authority_error("bar input is not ordered").into());
                }
                previous = Some(time);
                let anchor = AnchorInstance {
                    anchor_id: format!("bar-close-{count}"),
                    detector_id: "canonical_bar_close".into(),
                    lifecycle_state: "completed_bar".into(),
                    native_scale: NativeScale::Bar(BarScale::M15),
                    anchor_time: time,
                    value: None,
                    occur_time: None,
                    object_id: None,
                    source: SourceRef {
                        part: part.path.clone(),
                        row_index: row,
                    },
                };
                logical.update(&anchor);
                count += 1;
                minimum = Some(minimum.map_or(time, |v: i64| v.min(time)));
                maximum = Some(maximum.map_or(time, |v: i64| v.max(time)));
                Ok(())
            },
        )?;
        total.rows += metrics.rows;
        total.batches += metrics.batches;
        total.source_file_bytes += metrics.source_file_bytes;
        total.parts += metrics.parts;
        total.elapsed_millis += metrics.elapsed_millis;
    }
    let tick_source = inventory
        .sources
        .iter()
        .find(|source| source.scale == NativeScale::Tick)
        .ok_or_else(|| authority_error("tick source group missing"))?;
    let tick_part = &tick_source.parts[0];
    let tick_column = tick_source
        .physical_time_coordinate_fields
        .first()
        .ok_or_else(|| authority_error("tick timestamp field missing"))?;
    let tick_metrics = scan_part(
        &lake_path.join(&tick_part.path),
        tick_part,
        tick_column,
        4096,
        |_time, _row| Ok(()),
    )?;
    let identity = logical.finish();
    let summary = PilotSummary {
        pilot_id: "stage2-canonical-bar-close-15m".into(),
        instrument: config.instrument.clone(),
        anchor_detector: "canonical_bar_close".into(),
        anchor_time_semantics:
            "physical ordering coordinate only; detector lifecycle semantics not established".into(),
        availability_semantics_status: "UNRESOLVED".into(),
        availability_semantics_source:
            "manifest/schema establishes the physical bar_close_ts field only".into(),
        projected_columns,
        rows_scanned: total.rows,
        anchors_emitted: count,
        minimum_anchor_time: minimum,
        maximum_anchor_time: maximum,
        parts: total.parts,
        batches: total.batches,
        source_file_bytes: total.source_file_bytes,
        elapsed_millis: total.elapsed_millis,
        rows_per_second: if total.elapsed_millis == 0 {
            0.0
        } else {
            total.rows as f64 / (total.elapsed_millis as f64 / 1000.0)
        },
        output_identity: identity.clone(),
        tick_smoke_part: tick_part.path.clone(),
        tick_smoke_projected_column: tick_column.clone(),
        tick_smoke_rows: tick_metrics.rows,
        tick_smoke_batches: tick_metrics.batches,
        tick_smoke_source_file_bytes: tick_metrics.source_file_bytes,
        tick_smoke_elapsed_millis: tick_metrics.elapsed_millis,
        tick_smoke_rows_per_second: if tick_metrics.elapsed_millis == 0 {
            0.0
        } else {
            tick_metrics.rows as f64 / (tick_metrics.elapsed_millis as f64 / 1000.0)
        },
    };
    if let Ok(path) = env::var("TRINITYR_RESEARCH_ARTIFACTS") {
        let code = env::var("DISCOVERYR_CODE_IDENTITY").map_err(|_| {
            authority_error("DISCOVERYR_CODE_IDENTITY is required for persisted tape manifests")
        })?;
        let code = validate_code_identity(Some(&code))?;
        let manifest = TapeManifest {
            tape_id: summary.pilot_id.clone(),
            instrument: summary.instrument.clone(),
            pilot_id: summary.pilot_id.clone(),
            source_manifest_identity: inventory.source_manifest_identity,
            source_parts: pilot_source
                .parts
                .iter()
                .map(|part| part.path.clone())
                .collect(),
            anchor_detector: summary.anchor_detector.clone(),
            anchor_time_semantics: summary.anchor_time_semantics.clone(),
            availability_semantics_status: summary.availability_semantics_status.clone(),
            availability_semantics_source: summary.availability_semantics_source.clone(),
            context_contract: "no detector context; causal primitives only".into(),
            row_count: summary.anchors_emitted,
            minimum_anchor_time: summary.minimum_anchor_time,
            maximum_anchor_time: summary.maximum_anchor_time,
            code_identity: code,
            generated_utc: utc_now(),
            output_identity: identity,
        };
        let output = PathBuf::from(path);
        fs::create_dir_all(&output)?;
        fs::write(
            output.join("stage2_pilot_manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&PilotOutput { summary })?
    );
    Ok(())
}
