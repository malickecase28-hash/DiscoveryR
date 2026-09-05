use research_contracts::submission::reject_unsafe_ancestors;
use research_contracts::{
    ConfirmationContract, InstrumentScope, MethodChallenge, Question, ReproducibilityIdentity,
};
use research_engine::verify_identity;
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
        fs::write(&path, bytes).map_err(|error| format!("write {}: {error}", path.display()))?;
    } else {
        println!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
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
            let scope: InstrumentScope = value_as(value)?;
            scope.validate().map_err(|error| error.to_string())?;
            json!({"status":"not_run", "materialization":"delegated_to_research_tape", "instrument_id": scope.instrument.instrument_id, "scope_id": scope.interval.scope_id})
        }
        "run-experiment" => {
            let question: Question = value_as(value)?;
            question.validate().map_err(|error| error.to_string())?;
            json!({"status":"validated", "execution":"dry_run_only", "question_id": question.question_id})
        }
        "verify-result" => {
            let identity: ReproducibilityIdentity = value_as(value)?;
            let hash = verify_identity(&identity, None).map_err(|error| error.to_string())?;
            json!({"status":"verified", "identity_hash":hash})
        }
        "challenge-result" => {
            let challenge: MethodChallenge = value_as(value)?;
            challenge.validate().map_err(|error| error.to_string())?;
            json!({"status":"recorded", "challenge_id": challenge.challenge_id, "result": challenge.result})
        }
        "confirm-frozen-claim" => {
            let contract: ConfirmationContract = value_as(value)?;
            contract.validate().map_err(|error| error.to_string())?;
            json!({"status":"locked", "activation_locked":true, "confirmation_id":contract.confirmation_id, "custodian_required":true})
        }
        "generate-report" => json!({"status":"validated", "report": value}),
        _ => return Err(usage("unknown command")),
    };
    write_json(result, output.as_deref())
}

fn usage(message: &str) -> String {
    format!(
        "{message}; usage: research_engine <validate-instrument|materialize-scope|run-experiment|verify-result|challenge-result|confirm-frozen-claim|generate-report> <input.json> [output.json]"
    )
}
