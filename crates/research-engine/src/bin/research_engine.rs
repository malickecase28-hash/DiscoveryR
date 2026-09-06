use research_contracts::submission::reject_unsafe_ancestors;
use research_contracts::{
    CompleteReproducibilityIdentity, ConfirmationContract, InstrumentScope, MethodChallenge,
    Question, ReproducibilityIdentity,
};
use research_contracts::{ContextPermission, DetectorRole, EvidenceAccess, ProgramConsumer};
use research_engine::verify_identity;
use research_engine::{
    run_research, simulate_untrusted_strategy, CausalResearchRecord, ConfirmedBehavioralInput,
    ConfirmedInputManifest, DetectorAtlas, DetectorDescriptor, Direction,
    QuestionGenerationRequest, ResearchPlan, SimulationConfig, StrategyArchetype,
    StrategyObservation, StrategySpec,
};
use research_tape::{AnchorInstance, ContextObservation, NativeScale, SourceRef};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Deserialize)]
struct SyntheticStageInput {
    stage: String,
    instrument_id: String,
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
        "synthetic-stage" => synthetic_stage(value_as(value)?)?,
        _ => return Err(usage("unknown command")),
    };
    write_json(result, output.as_deref())
}

fn synthetic_stage(input: SyntheticStageInput) -> Result<Value, String> {
    if input.instrument_id.is_empty() || !matches!(input.stage.as_str(), "r" | "s" | "p") {
        return Err("synthetic stage requires stage r, s, or p and an instrument".into());
    }
    match input.stage.as_str() {
        "r" => {
            let scale = NativeScale::Tick;
            let atlas = DetectorAtlas::new(vec![
                DetectorDescriptor::new(
                    "anchor",
                    vec![DetectorRole::StructuralObject],
                    BTreeSet::from([scale.clone()]),
                )
                .map_err(|e| e.to_string())?,
                DetectorDescriptor::new(
                    "context",
                    vec![DetectorRole::Event],
                    BTreeSet::from([scale.clone()]),
                )
                .map_err(|e| e.to_string())?,
            ])
            .map_err(|e| e.to_string())?;
            let request = QuestionGenerationRequest {
                experiment_id: "synthetic-r".into(),
                instrument_ids: vec![input.instrument_id.clone()],
                anchor_detector_id: "anchor".into(),
                anchor_ids: vec!["anchor-1".into()],
                context_detector_ids: vec!["context".into()],
                lifecycle_states: vec!["formed".into()],
                context_ids: vec!["context".into()],
                native_scales: BTreeSet::from([scale.clone()]),
                anchor_time_ns: 100,
                context_available_time_ns: BTreeMap::from([("context".into(), 90)]),
                directions: vec![Direction::Positive],
                max_questions: 1,
            };
            let permission = ContextPermission {
                permission_id: "synthetic-r-permission".into(),
                consumer: ProgramConsumer::MarketResearch,
                allowed_detector_ids: vec!["context".into()],
                allowed_scales: BTreeSet::from([scale.clone()]),
                evidence_access: EvidenceAccess::UnconfirmedAllowed,
            };
            let identity = |name: &str| name.to_owned();
            let complete = research_contracts::CompleteReproducibilityIdentity {
                instrument_identity: identity(&input.instrument_id),
                producer_commit_identity: identity("producer"),
                producer_source_identity: identity("source"),
                producer_blob_identity: identity("blob"),
                authority_version_identity: identity("authority"),
                detector_version_identity: identity("detector"),
                parameter_identity: identity("parameters"),
                source_manifest_identity: identity("manifest"),
                payload_manifest_identity: identity("payload"),
                data_scope_identity: identity("scope"),
                experiment_contract_identity: identity("experiment"),
                code_identity: identity("code"),
                scanner_version_identity: identity("scanner"),
                control_design_identity: identity("controls"),
                null_design_identity: identity("null"),
                seed_identity: identity("seed"),
                output_identity: identity("output"),
                native_scale_identity: identity("tick"),
                availability_contract_identity: identity("availability"),
            };
            let records = vec![
                CausalResearchRecord::from_tape(
                    input.instrument_id.clone(),
                    "scope",
                    "manifest",
                    AnchorInstance {
                        anchor_id: "anchor-1".into(),
                        detector_id: "anchor".into(),
                        lifecycle_state: "formed".into(),
                        native_scale: scale.clone(),
                        anchor_time: 100,
                        value: Some(1.0),
                        occur_time: Some(80),
                        object_id: Some("a1".into()),
                        source: SourceRef {
                            part: "a".into(),
                            row_index: 0,
                        },
                    },
                    vec![ContextObservation {
                        detector_id: "context".into(),
                        native_scale: scale.clone(),
                        available_time: 90,
                        value: Some(2.0),
                        occur_time: Some(70),
                        object_id: Some("c1".into()),
                        source: SourceRef {
                            part: "c".into(),
                            row_index: 0,
                        },
                    }],
                )
                .map_err(|e| e.to_string())?,
                CausalResearchRecord::from_tape(
                    input.instrument_id.clone(),
                    "scope",
                    "manifest",
                    AnchorInstance {
                        anchor_id: "anchor-1".into(),
                        detector_id: "anchor".into(),
                        lifecycle_state: "formed".into(),
                        native_scale: scale.clone(),
                        anchor_time: 100,
                        value: Some(2.0),
                        occur_time: Some(80),
                        object_id: Some("a2".into()),
                        source: SourceRef {
                            part: "a".into(),
                            row_index: 1,
                        },
                    },
                    vec![ContextObservation {
                        detector_id: "context".into(),
                        native_scale: scale,
                        available_time: 90,
                        value: Some(4.0),
                        occur_time: Some(70),
                        object_id: Some("c2".into()),
                        source: SourceRef {
                            part: "c".into(),
                            row_index: 1,
                        },
                    }],
                )
                .map_err(|e| e.to_string())?,
            ];
            let plan = ResearchPlan {
                atlas,
                request,
                permission,
                reproducibility: research_contracts::ReproducibilityIdentity {
                    source_identity: "source".into(),
                    binary_identity: "binary".into(),
                    configuration_identity: "config".into(),
                    parameter_identity: "parameters".into(),
                    seed_identity: "seed".into(),
                    output_identity: "output".into(),
                    user_identities: vec![input.instrument_id],
                },
                complete_reproducibility: complete,
                max_records: 4,
            };
            let report = run_research(&plan, &records).map_err(|e| e.to_string())?;
            Ok(
                json!({"stage":"r","status":format!("{:?}", report.evidence_state),"identity":report.output_identity}),
            )
        }
        "s" => {
            let input_manifest = ConfirmedInputManifest::untrusted_for_description(
                "synthetic-strategy",
                "synthetic-holdout",
                vec![
                    ConfirmedBehavioralInput::untrusted_for_description("finding", 0)
                        .map_err(|e| e.to_string())?,
                ],
            )
            .map_err(|e| e.to_string())?;
            let spec = StrategySpec {
                archetype: StrategyArchetype::Continuation,
                hypothesis: research_contracts::StrategyHypothesis {
                    strategy_id: "synthetic-strategy".into(),
                    hypothesis_id: "synthetic-hypothesis".into(),
                    confirmed_finding_ids: vec!["finding".into()],
                    parameters: BTreeMap::from([("bound".into(), json!(true))]),
                },
                decision_rule: research_contracts::DecisionRule {
                    rule_id: "rule".into(),
                    entry: "signal_threshold".into(),
                    exit: "observation_end".into(),
                    sizing: "unit".into(),
                    management: "none".into(),
                    parameters: BTreeMap::from([("entry_threshold".into(), json!(0.5))]),
                },
                execution_assumption: research_contracts::ExecutionAssumption {
                    assumption_id: "execution".into(),
                    assumptions: BTreeMap::from([("fill_model".into(), json!("analytical_close"))]),
                },
                cost_model: research_contracts::CostModel {
                    cost_model_id: "cost".into(),
                    parameters: BTreeMap::from([("per_observation".into(), json!(0.0))]),
                },
                risk_rule: research_contracts::RiskRule {
                    risk_rule_id: "risk".into(),
                    parameters: BTreeMap::from([("max_abs_outcome".into(), json!(2.0))]),
                },
            };
            let observations = vec![StrategyObservation {
                timestamp_ns: 1,
                available_time_ns: 1,
                signal: Some(1.0),
                outcome: Some(1.0),
            }];
            let report = simulate_untrusted_strategy(
                &spec,
                &input_manifest,
                &observations,
                &SimulationConfig::default(),
            )
            .map_err(|e| e.to_string())?;
            Ok(
                json!({"stage":"s","status":"descriptive","identity":report.runtime_spec_identity,"authority_eligible":report.authority_eligible,"instrument_id":input.instrument_id}),
            )
        }
        "p" => {
            let streams = vec![
                research_engine::StrategyStream {
                    strategy_id: "s1".into(),
                    confirmation: research_contracts::ConfirmationState::Confirmed,
                    evidence_state: research_contracts::EvidenceState::Known,
                    returns: vec![Some(1.0), Some(-1.0)],
                    signals: vec![Some(1.0), Some(0.0)],
                    regimes: vec![Some("r".into()), Some("r".into())],
                    capital: Some(1.0),
                    capacity: Some(1.0),
                    liquidity: vec![Some(1.0), Some(1.0)],
                    risk_budget: Some(1.0),
                },
                research_engine::StrategyStream {
                    strategy_id: "s2".into(),
                    confirmation: research_contracts::ConfirmationState::Confirmed,
                    evidence_state: research_contracts::EvidenceState::Known,
                    returns: vec![Some(-1.0), Some(1.0)],
                    signals: vec![Some(1.0), Some(1.0)],
                    regimes: vec![Some("r".into()), Some("r".into())],
                    capital: Some(1.0),
                    capacity: Some(1.0),
                    liquidity: vec![Some(1.0), Some(1.0)],
                    risk_budget: Some(1.0),
                },
            ];
            let component = research_contracts::PortfolioComponent {
                component_id: "synthetic-portfolio".into(),
                confirmed_strategy_ids: vec!["s1".into(), "s2".into()],
                parameters: BTreeMap::from([("scope".into(), json!(input.instrument_id))]),
            };
            let report = research_engine::portfolio_report(
                &research_engine::PortfolioInput {
                    component,
                    strategies: streams,
                    constraints: vec![],
                },
                &[],
            )
            .map_err(|e| e.to_string())?;
            Ok(json!({"stage":"p","status":"descriptive","identity":report.identity}))
        }
        _ => unreachable!(),
    }
}

fn usage(message: &str) -> String {
    format!(
        "{message}; usage: research_engine <validate-instrument|materialize-scope|run-experiment|verify-result|challenge-result|confirm-frozen-claim|generate-report|synthetic-stage> <input.json> [output.json]"
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
