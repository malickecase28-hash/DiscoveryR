use research_contracts::submission::reject_unsafe_ancestors;
use research_contracts::{
    validate_compatibility, CompleteReproducibilityIdentity, ConfirmationActor,
    ConfirmationContract, InstrumentScope, ProducerAuthority,
};
use research_engine::{
    generate_question_templates, generate_report_manifest, hash_bytes, run_challenge_bundle,
    run_population_research, ChallengeRunInput, PopulationRunInput, QuestionTemplateRequest,
    ReportManifestInput,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityCompatibilityInput {
    expected: ProducerAuthority,
    actual: ProducerAuthority,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterializeScopeInput {
    scope: InstrumentScope,
    workspace_root: PathBuf,
    source_root: PathBuf,
    inventory_path: PathBuf,
    view_root: PathBuf,
    audit_root: PathBuf,
    source_manifest_identity: String,
    payload_manifest_identity: String,
    boundary_ms: i64,
    development_end_exclusive: String,
    code_identity: String,
    batch_size: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifyResultInput {
    identity: CompleteReproducibilityIdentity,
    artifact_path: String,
    expected_artifact_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateReportInput {
    manifest: ReportManifestInput,
    artifact_path: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("trinity-research: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage("missing command"))?;
    let input_path = args
        .next()
        .ok_or_else(|| usage("missing input JSON path"))?;
    let output = args.next();
    let value = read_json(&input_path)?;

    let result = match command.as_str() {
        "validate-instrument" => {
            let scope: InstrumentScope = value_as(value)?;
            scope.validate().map_err(|error| error.to_string())?;
            json!({
                "status":"VALID",
                "instrument_id":scope.instrument.instrument_id,
                "scope_id":scope.interval.scope_id
            })
        }
        "authority-compatibility" => {
            let input: AuthorityCompatibilityInput = value_as(value)?;
            serde_json::to_value(
                validate_compatibility(&input.expected, &input.actual)
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?
        }
        "materialize-scope" => {
            let input: MaterializeScopeInput = value_as(value)?;
            input.scope.validate().map_err(|error| error.to_string())?;
            let expected = research_tape::development_view::ExpectedDevelopmentScope {
                instrument: input.scope.instrument.instrument_id.clone(),
                scope_id: input.scope.interval.scope_id.clone(),
                source_manifest_identity: input.source_manifest_identity,
                payload_manifest_identity: input.payload_manifest_identity,
                boundary_ms: input.boundary_ms,
                development_end_exclusive: input.development_end_exclusive,
            };
            let identity = research_tape::development_view::materialize(
                &input.workspace_root,
                &input.source_root,
                &input.inventory_path,
                &input.view_root,
                &input.audit_root,
                &expected,
                &input.code_identity,
                input
                    .batch_size
                    .unwrap_or(research_tape::development_view::DEFAULT_BATCH_SIZE),
                true,
            )
            .map_err(|error| error.to_string())?;
            json!({
                "status":"COMPLETE",
                "identity":identity,
                "instrument_id":input.scope.instrument.instrument_id,
                "scope_id":input.scope.interval.scope_id
            })
        }
        "generate-questions" => {
            let input: QuestionTemplateRequest = value_as(value)?;
            let report = generate_question_templates(&input).map_err(|error| error.to_string())?;
            serde_json::to_value(report).map_err(|error| error.to_string())?
        }
        "run-experiment" => {
            let input: PopulationRunInput = value_as(value)?;
            let report = run_population_research(&input).map_err(|error| error.to_string())?;
            serde_json::to_value(report).map_err(|error| error.to_string())?
        }
        "verify-result" => {
            let input: VerifyResultInput = value_as(value)?;
            input.identity.validate().map_err(|error| error.to_string())?;
            let identity_hash = input
                .identity
                .identity_hash()
                .map_err(|error| error.to_string())?;
            let bytes = read_workspace_artifact(&input.artifact_path)?;
            let actual = hash_bytes(&bytes);
            if actual != input.expected_artifact_sha256 {
                return Err(format!(
                    "artifact SHA-256 mismatch: expected {}, got {actual}",
                    input.expected_artifact_sha256
                ));
            }
            json!({
                "status":"VERIFIED",
                "identity_hash":identity_hash,
                "artifact_sha256":actual,
                "artifact_bytes":bytes.len()
            })
        }
        "challenge-result" => {
            let input: ChallengeRunInput = value_as(value)?;
            let bundle = run_challenge_bundle(&input).map_err(|error| error.to_string())?;
            json!({
                "status":"COMPLETE",
                "target_identity":bundle.target_identity(),
                "challenge_identity":bundle.identity(),
                "evidence_identity":bundle.evidence_identity(),
                "report_identity":bundle.report_identity(),
                "authority_eligible":bundle.authority_eligible(),
                "challenges":bundle.challenges()
            })
        }
        "confirm-frozen-claim" => {
            // Research-worker CLI is deliberately not an unlock surface. The
            // authority-bearing path is ConfirmationRunner::confirm_challenged,
            // which requires an externally signed custodian artifact and an
            // in-process authority-eligible challenge bundle.
            let contract: ConfirmationContract = value_as(value)?;
            contract.validate().map_err(|error| error.to_string())?;
            if contract.actor == ConfirmationActor::Worker {
                return Err("worker cannot authorize confirmation".into());
            }
            json!({
                "status":"ACTIVATION_LOCKED",
                "activation_locked":true,
                "confirmation_id":contract.confirmation_id,
                "requires":"external signed custodian artifact plus authority-eligible challenged report"
            })
        }
        "generate-report" => {
            let input: GenerateReportInput = value_as(value)?;
            let bytes = read_workspace_artifact(&input.artifact_path)?;
            let manifest = generate_report_manifest(&input.manifest, &bytes)
                .map_err(|error| error.to_string())?;
            serde_json::to_value(manifest).map_err(|error| error.to_string())?
        }
        _ => return Err(usage("unknown command")),
    };

    write_json(&result, output.as_deref())
}

fn read_json(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {path}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {path}: {error}"))
}

fn value_as<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|error| error.to_string())
}

fn workspace_root() -> Result<PathBuf, String> {
    let root = env::var_os("TRINITYR_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().map_err(|error| error.to_string())?);
    reject_unsafe_ancestors(&root).map_err(|error| error.to_string())?;
    fs::canonicalize(&root).map_err(|error| format!("canonicalize workspace: {error}"))
}

fn workspace_path(value: &str) -> Result<PathBuf, String> {
    let relative = Path::new(value);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("artifact/output path must be workspace-relative".into());
    }
    let root = workspace_root()?;
    let target = root.join(relative);
    if let Some(parent) = target.parent() {
        reject_unsafe_ancestors(parent).map_err(|error| error.to_string())?;
    }
    Ok(target)
}

fn read_workspace_artifact(value: &str) -> Result<Vec<u8>, String> {
    let root = workspace_root()?;
    let path = workspace_path(value)?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("artifact metadata {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("artifact must be a regular non-symlink file".into());
    }
    let canonical = fs::canonicalize(&path)
        .map_err(|error| format!("canonicalize artifact {}: {error}", path.display()))?;
    if !canonical.starts_with(&root) {
        return Err("artifact escapes workspace".into());
    }
    fs::read(&canonical).map_err(|error| format!("read artifact {}: {error}", canonical.display()))
}

fn write_json(value: &Value, output: Option<&str>) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let Some(output) = output else {
        println!("{}", String::from_utf8_lossy(&bytes));
        return Ok(());
    };
    let path = workspace_path(output)?;
    if fs::symlink_metadata(&path).is_ok() {
        return Err(format!("refusing to replace existing output {}", path.display()));
    }
    let parent = path.parent().ok_or("output has no parent")?;
    fs::create_dir_all(parent).map_err(|error| format!("create output root: {error}"))?;
    reject_unsafe_ancestors(parent).map_err(|error| error.to_string())?;
    let temp = parent.join(format!(
        ".trinity-research-output-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| format!("create output: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("write output: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync output: {error}"))?;
    fs::hard_link(&temp, &path)
        .map_err(|error| format!("publish {}: {error}", path.display()))?;
    let _ = fs::remove_file(&temp);
    Ok(())
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn usage(message: &str) -> String {
    format!(
        "{message}; usage: trinity_research <validate-instrument|authority-compatibility|materialize-scope|generate-questions|run-experiment|verify-result|challenge-result|confirm-frozen-claim|generate-report> <input.json> [output.json]"
    )
}
