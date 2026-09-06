use research_contracts::submission::reject_unsafe_ancestors;
use research_contracts::{
    CompleteReproducibilityIdentity, ConfirmationContract, InstrumentScope, MethodChallenge,
    Question, ReproducibilityIdentity,
};
use research_engine::verify_identity;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

fn read_json(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {path}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {path}: {error}"))
}

fn write_json(value: Value, output: Option<&str>) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
    if let Some(path) = output {
        let path = output_path(path)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("create output root: {error}"))?;
            }
        }
        if fs::symlink_metadata(&path).is_ok() {
            return Err(format!(
                "refusing to replace existing output {}",
                path.display()
            ));
        }
        let parent = path.parent().ok_or("output has no parent")?;
        reject_unsafe_ancestors(parent).map_err(|error| error.to_string())?;
        let temp = parent.join(format!(
            ".research-output-{}-{}",
            std::process::id(),
            unique_suffix()
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| format!("create output: {error}"))?;
        use std::io::Write;
        file.write_all(&bytes)
            .map_err(|error| format!("write output: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("sync output: {error}"))?;
        fs::hard_link(&temp, &path)
            .map_err(|error| format!("publish {}: {error}", path.display()))?;
        // The hard-link claim is the durable commit; cleanup is recoverable.
        let _ = fs::remove_file(&temp);
    } else {
        println!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn output_path(path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(path);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("output path must be relative and stay under the workspace".into());
    }
    let root = env::var_os("TRINITYR_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().map_err(|error| error.to_string())?);
    reject_unsafe_ancestors(&root).map_err(|error| error.to_string())?;
    let target = root.join(relative);
    if let Some(parent) = target.parent() {
        reject_unsafe_ancestors(parent).map_err(|error| error.to_string())?;
    }
    Ok(target)
}

fn value_as<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
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

fn main() {
    if let Err(error) = run() {
        eprintln!("research-engine: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage("missing command"))?;
    let input = args
        .next()
        .ok_or_else(|| usage("missing input JSON path"))?;
    let output = args.next();
    let value = read_json(&input)?;
    let result = match command.as_str() {
        "validate-instrument" => {
            let scope: InstrumentScope = value_as(value)?;
            scope.validate().map_err(|error| error.to_string())?;
            json!({"status":"valid", "instrument_id": scope.instrument.instrument_id, "scope_id": scope.interval.scope_id})
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
            json!({"status":"complete", "identity": identity, "instrument_id": input.scope.instrument.instrument_id, "scope_id": input.scope.interval.scope_id})
        }
        "run-experiment" => {
            let question: Question = value_as(value)?;
            question.validate().map_err(|error| error.to_string())?;
            json!({"status":"not_run", "execution":"unavailable", "question_id": question.question_id})
        }
        "verify-result" => {
            let hash = if let Ok(identity) =
                serde_json::from_value::<CompleteReproducibilityIdentity>(value.clone())
            {
                identity
                    .identity_hash()
                    .map_err(|error| error.to_string())?
            } else {
                let identity: ReproducibilityIdentity = value_as(value)?;
                verify_identity(&identity, None).map_err(|error| error.to_string())?
            };
            json!({"status":"not_run", "verification":"artifact_bytes_required", "identity_hash":hash})
        }
        "challenge-result" => {
            let challenge: MethodChallenge = value_as(value)?;
            challenge.validate().map_err(|error| error.to_string())?;
            json!({"status":"not_run", "challenge_id": challenge.challenge_id, "result": challenge.result})
        }
        "confirm-frozen-claim" => {
            let contract: ConfirmationContract = value_as(value)?;
            contract.validate().map_err(|error| error.to_string())?;
            json!({"status":"not_run", "activation_locked":true, "confirmation_id":contract.confirmation_id, "custodian_required":true})
        }
        "generate-report" => {
            json!({"status":"not_run", "report_generation":"typed_runner_required"})
        }
        _ => return Err(usage("unknown command")),
    };
    write_json(result, output.as_deref())
}

fn usage(message: &str) -> String {
    format!(
        "{message}; usage: research_engine <validate-instrument|materialize-scope|run-experiment|verify-result|challenge-result|confirm-frozen-claim|generate-report> <input.json> [output.json]"
    )
}

#[cfg(test)]
mod tests {
    use super::output_path;

    #[test]
    fn output_path_rejects_escape_attempts() {
        assert!(output_path("..\\outside.json").is_err());
        assert!(output_path("C:\\outside.json").is_err());
        assert!(output_path("reports\\result.json").is_ok());
    }
}
