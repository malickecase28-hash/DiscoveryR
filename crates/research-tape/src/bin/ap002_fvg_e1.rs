use arrow_array::Array;
use research_tape::fvg_e1::{parse_payload, Preflight, SidecarCursor, ZoneBook};
use research_tape::{
    batch_f64, batch_i64, batch_large_string, required_i64, ProjectedParquetReader,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TIMEFRAMES: [&str; 7] = ["15s", "30s", "1m", "5m", "15m", "1h", "4h"];
const RESPONSE_HORIZONS_BARS: [u32; 3] = [1, 3, 5];

#[derive(Debug, Deserialize)]
struct ViewManifest {
    scope_id: String,
    logical_view_identity: String,
    source_groups: BTreeMap<String, Vec<ViewPart>>,
}
#[derive(Debug, Deserialize)]
struct ViewPart {
    logical_path: String,
    development_row_count: u64,
}
#[derive(Debug, Deserialize)]
struct SidecarManifest {
    artifact_type: String,
    artifact_version: String,
    development_view_manifest_sha256: String,
    projected_columns: Vec<String>,
    timeframes: Vec<String>,
    input_rows: u64,
    sidecar_rows: u64,
    sidecar_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
struct Provenance {
    scope_id: String,
    development_view_manifest_sha256: String,
    sidecar_manifest_sha256: String,
    sidecar_sha256: String,
    source_parts: Vec<String>,
    scanner_commit: String,
}

#[derive(Debug, Clone, Serialize)]
struct StratumResult {
    timeframe: String,
    n_bars: u64,
    n_payload_rows: u64,
    formation_payload_n: u64,
    lawful_formation_n: u64,
    unavailable_formation_n: u64,
    first_touch_events_n: u64,
    fill_events_n: u64,
    invalidations_n: u64,
    right_censored_n: u64,
    response_observations_n: [u64; 3],
    orphan_touch_n: u64,
    orphan_fill_n: u64,
}

#[derive(Debug, Serialize)]
struct TableEnvelope<T> {
    n: u64,
    provenance: Provenance,
    rows: T,
}

#[derive(Debug, Serialize)]
struct ZoneTableRef {
    n: u64,
    path: String,
    sha256: String,
    logical_rows_sha256: String,
    provenance: Provenance,
}

#[derive(Debug, Serialize)]
struct Results {
    experiment_id: String,
    scope_id: String,
    exposure_class: &'static str,
    confirmation_status: &'static str,
    scanner_commit: String,
    preflight_hash: String,
    sidecar_manifest_sha256: String,
    sidecar_rows: u64,
    logical_result_hash: String,
    tables: ResultsTables,
}

#[derive(Debug, Serialize)]
struct ResultsTables {
    strata: TableEnvelope<Vec<StratumResult>>,
    zones: ZoneTableRef,
}

#[derive(Debug, Serialize)]
struct Experiment {
    experiment_id: &'static str,
    objective: &'static str,
    instrument: &'static str,
    development_scope: &'static str,
    exposure_class: &'static str,
    confirmation_status: &'static str,
    native_strata: [&'static str; 7],
    lawful_identity: &'static str,
    formation_availability: &'static str,
    lifecycle_fields: [&'static str; 10],
    prospective_response: &'static str,
    response_horizons_bars: [u32; 3],
    exclusions: [&'static str; 5],
}

struct ZoneWriter {
    writer: BufWriter<File>,
    physical: Sha256,
    logical: Sha256,
    rows: u64,
}

impl ZoneWriter {
    fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            writer: BufWriter::new(File::create(path)?),
            physical: Sha256::new(),
            logical: Sha256::new(),
            rows: 0,
        })
    }

    fn write(
        &mut self,
        zone: &research_tape::fvg_e1::ZoneLifecycle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec(zone)?;
        self.logical.update((bytes.len() as u64).to_le_bytes());
        self.logical.update(&bytes);
        self.writer.write_all(&bytes)?;
        self.writer.write_all(b"\n")?;
        self.physical.update(&bytes);
        self.physical.update(b"\n");
        self.rows += 1;
        Ok(())
    }

    fn finish(mut self) -> Result<(u64, String, String), Box<dyn std::error::Error>> {
        self.writer.flush()?;
        Ok((
            self.rows,
            format!("{:x}", self.physical.finalize()),
            format!("{:x}", self.logical.finalize()),
        ))
    }
}

fn safe_relative_path(value: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        return Err(format!("unsafe view path: {value}").into());
    }
    Ok(path.to_owned())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn source_parts<'a>(
    manifest: &'a ViewManifest,
    timeframe: &str,
) -> Result<&'a [ViewPart], Box<dyn std::error::Error>> {
    manifest
        .source_groups
        .get(timeframe)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing source group: {timeframe}").into())
}

fn run_preflight(
    view_root: &Path,
    manifest: &ViewManifest,
    manifest_hash: &str,
    scanner_commit: &str,
) -> Result<research_tape::fvg_e1::PreflightResult, Box<dyn std::error::Error>> {
    let mut preflight = Preflight::new(manifest_hash, scanner_commit);
    let mut last_source_sequence = None;
    for part in source_parts(manifest, "tick")? {
        let path = view_root.join(safe_relative_path(&part.logical_path)?);
        let reader = ProjectedParquetReader::new(
            path,
            vec!["received_ts_ns".into(), "source_sequence".into()],
            65_536,
        )?;
        let mut scan = reader.scan()?;
        let mut row = 0_u64;
        while let Some(batch) = scan.next() {
            let batch = batch?;
            let received = batch_i64(&batch, "received_ts_ns")?;
            let sequence = batch_i64(&batch, "source_sequence")?;
            for index in 0..batch.num_rows() {
                let received = required_i64(received, index, "received_ts_ns")?;
                let sequence = required_i64(sequence, index, "source_sequence")?;
                if last_source_sequence.is_some_and(|previous| sequence <= previous) {
                    return Err(format!(
                        "source_sequence is not strictly increasing at {} row {}",
                        part.logical_path, row
                    )
                    .into());
                }
                last_source_sequence = Some(sequence);
                preflight.observe(&part.logical_path, row, received, sequence);
                row += 1;
            }
        }
        if row != part.development_row_count {
            return Err(format!(
                "preflight row count mismatch for {}: declared {}, actual {}",
                part.logical_path, part.development_row_count, row
            )
            .into());
        }
    }
    Ok(preflight.finish())
}

fn verify_sidecar(
    sidecar: &Path,
    sidecar_manifest_path: &Path,
    view_manifest_hash: &str,
) -> Result<(SidecarManifest, String, String), Box<dyn std::error::Error>> {
    let manifest_bytes = fs::read(sidecar_manifest_path)?;
    let manifest: SidecarManifest = serde_json::from_slice(&manifest_bytes)?;
    if manifest.artifact_type != "FVG_BAR_AVAILABILITY_SIDECAR"
        || manifest.artifact_version != "V1"
        || manifest.development_view_manifest_sha256 != view_manifest_hash
        || manifest.projected_columns
            != ["event_ts_ns", "received_ts_ns", "source_sequence"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        || manifest.timeframes
            != TIMEFRAMES
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
    {
        return Err("frozen sidecar manifest does not match the declared authority".into());
    }
    let manifest_hash = format!("{:x}", Sha256::digest(&manifest_bytes));
    let sidecar_hash = sha256_file(sidecar)?;
    if sidecar_hash != manifest.sidecar_sha256 {
        return Err(format!(
            "sidecar hash mismatch: expected {}, got {}",
            manifest.sidecar_sha256, sidecar_hash
        )
        .into());
    }
    Ok((manifest, manifest_hash, sidecar_hash))
}

fn scan_stratum(
    view_root: &Path,
    manifest: &ViewManifest,
    sidecar: &Path,
    timeframe: &str,
    zone_writer: &mut ZoneWriter,
) -> Result<StratumResult, Box<dyn std::error::Error>> {
    let parts = source_parts(manifest, timeframe)?;
    let mut sidecar_cursor = SidecarCursor::new(BufReader::new(File::open(sidecar)?), timeframe);
    let mut book = ZoneBook::new(timeframe);
    let mut n_bars = 0_u64;
    let mut n_payload_rows = 0_u64;
    let mut formation_payload_n = 0_u64;
    let mut unavailable_formation_n = 0_u64;
    let mut first_touch_events_n = 0_u64;
    let mut fill_events_n = 0_u64;
    let mut last_bar_close_ts = None;
    for part in parts {
        let path = view_root.join(safe_relative_path(&part.logical_path)?);
        let reader = ProjectedParquetReader::new(
            path,
            ["bar_close_ts", "close", "payload_fvg"]
                .into_iter()
                .map(String::from)
                .collect(),
            65_536,
        )?;
        let mut scan = reader.scan()?;
        let mut row = 0_u64;
        while let Some(batch) = scan.next() {
            let batch = batch?;
            let bar_close = batch_i64(&batch, "bar_close_ts")?;
            let close = batch_f64(&batch, "close")?;
            let payload = batch_large_string(&batch, "payload_fvg")?;
            for index in 0..batch.num_rows() {
                let bar_close_ts = required_i64(bar_close, index, "bar_close_ts")?;
                let close = if close.is_null(index) {
                    return Err(format!("null close at {} row {}", part.logical_path, row).into());
                } else {
                    close.value(index)
                };
                let payload = if payload.is_null(index) {
                    "{}"
                } else {
                    payload.value(index)
                };
                let parsed = parse_payload(payload).map_err(|error| {
                    format!(
                        "malformed payload at {} row {}: {error}",
                        part.logical_path, row
                    )
                })?;
                if payload != "{}" {
                    n_payload_rows += 1;
                }
                let formation_n = parsed.formations().len() as u64;
                formation_payload_n += formation_n;
                first_touch_events_n += parsed.fvg_first_touch.len() as u64;
                fill_events_n += parsed.fvg_filled.len() as u64;
                let bar_close_ts_ns = bar_close_ts
                    .checked_mul(1_000_000)
                    .ok_or("bar close nanosecond conversion overflow")?;
                let availability = sidecar_cursor.lookup(bar_close_ts_ns)?;
                if availability.is_none() {
                    unavailable_formation_n += formation_n;
                }
                book.ingest_bar_with_close_optional(
                    bar_close_ts,
                    availability,
                    payload,
                    Some(close),
                )?;
                last_bar_close_ts = Some(bar_close_ts);
                n_bars += 1;
                row += 1;
            }
        }
        if row != part.development_row_count {
            return Err(format!(
                "bar row count mismatch for {}: declared {}, actual {}",
                part.logical_path, part.development_row_count, row
            )
            .into());
        }
    }

    let last_bar_close_ts = last_bar_close_ts.ok_or("empty native stratum")?;
    let orphan_touch_n = book.orphan_touch_count;
    let orphan_fill_n = book.orphan_fill_count;
    let zones = book.finish(last_bar_close_ts);
    for zone in &zones {
        zone_writer.write(zone)?;
    }
    let response_observations_n = [0, 1, 2].map(|horizon| {
        zones
            .iter()
            .filter(|zone| zone.prospective_response_bps[horizon].is_some())
            .count() as u64
    });
    let lawful_formation_n = zones.len() as u64;
    let invalidations_n = zones.iter().filter(|zone| zone.fill_ts.is_some()).count() as u64;
    let right_censored_n = zones.iter().filter(|zone| zone.right_censored).count() as u64;
    Ok(StratumResult {
        timeframe: timeframe.into(),
        n_bars,
        n_payload_rows,
        formation_payload_n,
        lawful_formation_n,
        unavailable_formation_n,
        first_touch_events_n,
        fill_events_n,
        invalidations_n,
        right_censored_n,
        response_observations_n,
        orphan_touch_n,
        orphan_fill_n,
    })
}

fn utc_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "0".into(), |duration| duration.as_secs().to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1 scanner failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut view_root = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut sidecar = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v1.jsonl",
    );
    let mut sidecar_manifest = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v1.manifest.json",
    );
    let mut output_root = PathBuf::from(".runs/ap-002-e1-native-strata/run");
    let mut scanner_commit = String::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--view-root" => view_root = args.next().ok_or("missing --view-root")?.into(),
            "--sidecar" => sidecar = args.next().ok_or("missing --sidecar")?.into(),
            "--sidecar-manifest" => {
                sidecar_manifest = args.next().ok_or("missing --sidecar-manifest")?.into()
            }
            "--output-root" => output_root = args.next().ok_or("missing --output-root")?.into(),
            "--scanner-commit" => scanner_commit = args.next().ok_or("missing --scanner-commit")?,
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if scanner_commit.len() != 40 || !scanner_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--scanner-commit must be a 40-character Git SHA".into());
    }
    fs::create_dir_all(&output_root)?;
    let view_manifest_bytes = fs::read(view_root.join("view_manifest.json"))?;
    let view_manifest: ViewManifest = serde_json::from_slice(&view_manifest_bytes)?;
    let view_manifest_hash = format!("{:x}", Sha256::digest(&view_manifest_bytes));
    let (sidecar_manifest_value, sidecar_manifest_hash, sidecar_hash) =
        verify_sidecar(&sidecar, &sidecar_manifest, &view_manifest_hash)?;
    let preflight = run_preflight(
        &view_root,
        &view_manifest,
        &view_manifest_hash,
        &scanner_commit,
    )?;
    write_json(&output_root.join("PREFLIGHT.json"), &preflight)?;
    if preflight.regressions_count != 0 {
        return Err(format!(
            "receipt-time regressions detected: {}",
            preflight.regressions_count
        )
        .into());
    }
    if preflight.rows_scanned != sidecar_manifest_value.input_rows {
        return Err(format!(
            "preflight rows {} do not match sidecar input rows {}",
            preflight.rows_scanned, sidecar_manifest_value.input_rows
        )
        .into());
    }

    let provenance = Provenance {
        scope_id: view_manifest.scope_id.clone(),
        development_view_manifest_sha256: view_manifest_hash.clone(),
        sidecar_manifest_sha256: sidecar_manifest_hash.clone(),
        sidecar_sha256: sidecar_hash.clone(),
        source_parts: Vec::new(),
        scanner_commit: scanner_commit.clone(),
    };
    let zones_path = output_root.join("E1_ZONES.jsonl");
    let mut zone_writer = ZoneWriter::new(&zones_path)?;
    let mut strata = Vec::new();
    for timeframe in TIMEFRAMES {
        strata.push(scan_stratum(
            &view_root,
            &view_manifest,
            &sidecar,
            timeframe,
            &mut zone_writer,
        )?);
    }
    let (zone_n, zone_sha256, zone_logical_hash) = zone_writer.finish()?;
    let table_provenance = Provenance {
        source_parts: TIMEFRAMES
            .iter()
            .flat_map(|timeframe| source_parts(&view_manifest, timeframe).unwrap_or_default())
            .map(|part| part.logical_path.clone())
            .collect(),
        ..provenance.clone()
    };
    let strata_table = TableEnvelope {
        n: strata.len() as u64,
        provenance: table_provenance.clone(),
        rows: strata,
    };
    let logical_input = serde_json::to_vec(&(
        view_manifest.logical_view_identity,
        &strata_table,
        &zone_logical_hash,
    ))?;
    let logical_result_hash = format!("{:x}", Sha256::digest(logical_input));
    let results = Results {
        experiment_id: "AP-002-FVG-E1-NATIVE-STRATA".into(),
        scope_id: "XAUUSD_DATA_SCOPE_V1".into(),
        exposure_class: "E1_PHENOTYPE",
        confirmation_status: "LOCKED",
        scanner_commit: scanner_commit.clone(),
        preflight_hash: preflight.deterministic_preflight_hash.clone(),
        sidecar_manifest_sha256: sidecar_manifest_hash,
        sidecar_rows: sidecar_manifest_value.sidecar_rows,
        logical_result_hash: logical_result_hash.clone(),
        tables: ResultsTables {
            strata: strata_table,
            zones: ZoneTableRef {
                n: zone_n,
                path: "E1_ZONES.jsonl".into(),
                sha256: zone_sha256,
                logical_rows_sha256: zone_logical_hash,
                provenance: table_provenance,
            },
        },
    };
    let experiment = Experiment {
        experiment_id: "AP-002-FVG-E1-NATIVE-STRATA",
        objective: "Characterize FVG formation and native lifecycle behavior in XAUUSD development data.",
        instrument: "XAUUSD",
        development_scope: "XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only",
        exposure_class: "E1_PHENOTYPE",
        confirmation_status: "LOCKED",
        native_strata: TIMEFRAMES,
        lawful_identity: "(timeframe, zone_id); cross-scale identity prohibited",
        formation_availability: "Frozen sidecar available_time_ns from first later-bucket tick receipt; source_sequence breaks equal receipts.",
        lifecycle_fields: ["formation", "direction", "bounds", "active_persistence", "first_touch", "fill", "invalidation", "duration", "right_censoring", "prospective_response"],
        prospective_response: "Close returns at native bars 1, 3, and 5 strictly after the availability-associated formation bar; current formation-bar close is the known reference only.",
        response_horizons_bars: RESPONSE_HORIZONS_BARS,
        exclusions: ["E2", "E3", "confirmation", "strategy rules", "other detector outcomes"],
    };
    write_json(&output_root.join("E1_EXPERIMENT.json"), &experiment)?;
    write_json(&output_root.join("E1_RESULTS.json"), &results)?;
    fs::write(
        output_root.join("E1_FINDINGS.md"),
        format!(
            "# AP-002 E1 FVG native-strata findings\n\n- Scope: XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only.\n- Native identity: `(timeframe, zone_id)`; cross-scale identity was not used.\n- Preflight: {} rows, {} receipt-time regressions.\n- Logical result hash: `{}`.\n- Formation cohorts use frozen receipt-time availability; touch/fill fields are later lifecycle outcomes and do not define formation cohorts.\n- Unfilled zones are reported as right-censored at the last observed native bar.\n- This E1 result is phenotype evidence only; confirmation remains locked and no strategy rule is produced.\n",
            preflight.rows_scanned,
            preflight.regressions_count,
            logical_result_hash
        ),
    )?;
    let contract_hash = sha256_file(Path::new(
        "knowledge/wave1_authority_closure/AP-002_AUTHORITY_V1.json",
    ))?;
    let run_manifest = serde_json::json!({
        "run_id": output_root.file_name().and_then(|value| value.to_str()).unwrap_or("run"),
        "experiment_id": "AP-002-FVG-E1-NATIVE-STRATA",
        "researcher_id": "AP-002-E1-NATIVE-STRATA-TOOLSMITH",
        "lab": "TOOLSMITH",
        "code_identity": scanner_commit,
        "contract_hash": contract_hash,
        "input_identity": view_manifest_hash,
        "executed_utc": utc_seconds(),
        "output_identity": logical_result_hash,
    });
    write_json(&output_root.join("RUN_MANIFEST.json"), &run_manifest)?;
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
