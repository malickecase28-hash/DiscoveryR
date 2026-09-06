use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use research_contracts::{
    BarScale, CompleteReproducibilityIdentity, ContextPermission, DetectorRole, EvidenceAccess,
    NativeScale, ProgramConsumer, ReproducibilityIdentity,
};
use research_engine::{
    hash_bytes, CandidateGate, DetectorDescriptorInput, MetricKind, MetricRequest,
    PopulationRecordInput, PopulationResearchReport, PopulationRunInput,
};
use research_tape::{AnchorInstance, ContextObservation, SourceRef};
use serde_json::{json, Value};

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("trinityr-canonical-cli-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn complete_identity() -> CompleteReproducibilityIdentity {
    CompleteReproducibilityIdentity {
        instrument_identity: "xauusd".into(),
        producer_commit_identity: "producer-commit".into(),
        producer_source_identity: "producer-source".into(),
        producer_blob_identity: "producer-blob".into(),
        authority_version_identity: "authority-v2".into(),
        detector_version_identity: "detectors-v1".into(),
        parameter_identity: "parameters-v1".into(),
        source_manifest_identity: "source-manifest".into(),
        payload_manifest_identity: "payload-manifest".into(),
        data_scope_identity: "scope-development".into(),
        experiment_contract_identity: "experiment-contract".into(),
        code_identity: "code-commit".into(),
        scanner_version_identity: "scanner-v1".into(),
        control_design_identity: "controls-v1".into(),
        null_design_identity: "null-v1".into(),
        seed_identity: "seed-v1".into(),
        output_identity: "declared-output".into(),
        native_scale_identity: "mixed-native-scales".into(),
        availability_contract_identity: "availability-v1".into(),
    }
}

fn record(row: u64, anchor_time: i64, context_time: i64, a: f64, c: f64) -> PopulationRecordInput {
    PopulationRecordInput {
        instrument_id: "xauusd".into(),
        scope_identity: "scope-development".into(),
        source_manifest_identity: "source-manifest".into(),
        anchor: AnchorInstance {
            anchor_id: "anchor-stage".into(),
            detector_id: "anchor".into(),
            lifecycle_state: "formed".into(),
            native_scale: NativeScale::Tick,
            anchor_time,
            value: Some(a),
            occur_time: Some(anchor_time - 5),
            object_id: Some(format!("anchor-{row}")),
            source: SourceRef {
                part: "tick.parquet".into(),
                row_index: row,
            },
        },
        contexts: vec![ContextObservation {
            detector_id: "context".into(),
            native_scale: NativeScale::Bar(BarScale::M5),
            available_time: context_time,
            value: Some(c),
            occur_time: Some(context_time - 10),
            object_id: Some(format!("context-{row}")),
            source: SourceRef {
                part: "5m.parquet".into(),
                row_index: row,
            },
        }],
    }
}

fn experiment() -> PopulationRunInput {
    let m5 = NativeScale::Bar(BarScale::M5);
    PopulationRunInput {
        detectors: vec![
            DetectorDescriptorInput {
                detector_id: "anchor".into(),
                roles: vec![DetectorRole::StructuralObject],
                native_scales: BTreeSet::from([NativeScale::Tick]),
                derives_from: vec![],
            },
            DetectorDescriptorInput {
                detector_id: "context".into(),
                roles: vec![DetectorRole::StateRegime],
                native_scales: BTreeSet::from([m5.clone()]),
                derives_from: vec![],
            },
        ],
        permission: ContextPermission {
            permission_id: "market-r".into(),
            consumer: ProgramConsumer::MarketResearch,
            allowed_detector_ids: vec!["context".into()],
            allowed_scales: BTreeSet::from([m5.clone()]),
            evidence_access: EvidenceAccess::UnconfirmedAllowed,
        },
        reproducibility: ReproducibilityIdentity {
            source_identity: "source-manifest".into(),
            binary_identity: "binary-v1".into(),
            configuration_identity: "config-v1".into(),
            parameter_identity: "parameters-v1".into(),
            seed_identity: "seed-v1".into(),
            output_identity: "declared-output".into(),
            user_identities: vec!["xauusd".into()],
        },
        complete_reproducibility: complete_identity(),
        instrument_ids: vec!["xauusd".into()],
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["anchor-stage".into()],
        lifecycle_states: vec!["formed".into()],
        metric_requests: vec![MetricRequest {
            question_id: "q-context".into(),
            context_detector_id: "context".into(),
            operator: "context".into(),
            anchor_scale: NativeScale::Tick,
            context_scale: m5,
            metric: MetricKind::Pearson,
            candidate_gate: Some(CandidateGate {
                min_support: 2,
                min_abs_metric: 0.9,
            }),
        }],
        max_records: 10,
        records: vec![record(1, 100, 90, 1.0, 2.0), record(2, 200, 190, 2.0, 4.0)],
    }
}

fn run(
    root: &PathBuf,
    command: &str,
    input: &PathBuf,
    output: Option<&str>,
) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_trinity_research"));
    process
        .env("TRINITYR_WORKSPACE", root)
        .arg(command)
        .arg(input);
    if let Some(output) = output {
        process.arg(output);
    }
    process.output().unwrap()
}

#[test]
fn canonical_cli_runs_verifies_and_manifests_a_population_result() {
    let root = temp_root();
    let input_path = root.join("experiment.json");
    fs::write(
        &input_path,
        serde_json::to_vec_pretty(&experiment()).unwrap(),
    )
    .unwrap();

    let executed = run(
        &root,
        "run-experiment",
        &input_path,
        Some("results/program-r.json"),
    );
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let artifact = root.join("results/program-r.json");
    let bytes = fs::read(&artifact).unwrap();
    let report: PopulationResearchReport = serde_json::from_slice(&bytes).unwrap();
    report.validate_identity().unwrap();
    assert_eq!(report.scope_start_ns, Some(100));
    assert_eq!(report.scope_end_ns, Some(200));

    let verify_path = root.join("verify.json");
    fs::write(
        &verify_path,
        serde_json::to_vec_pretty(&json!({
            "identity": complete_identity(),
            "artifact_path": "results/program-r.json",
            "expected_artifact_sha256": hash_bytes(&bytes)
        }))
        .unwrap(),
    )
    .unwrap();
    let verified = run(&root, "verify-result", &verify_path, None);
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let value: Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(value["status"], "VERIFIED");

    let manifest_path = root.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&json!({
            "manifest": {
                "program": "MARKET_RESEARCH",
                "artifact_kind": "PROGRAM_R_RESULT",
                "artifact_identity": report.output_identity,
                "reproducibility": complete_identity()
            },
            "artifact_path": "results/program-r.json"
        }))
        .unwrap(),
    )
    .unwrap();
    let manifested = run(&root, "generate-report", &manifest_path, None);
    assert!(
        manifested.status.success(),
        "{}",
        String::from_utf8_lossy(&manifested.stderr)
    );
    let value: Value = serde_json::from_slice(&manifested.stdout).unwrap();
    assert_eq!(value["manifest_version"], 1);
    assert_eq!(value["artifact_sha256"], hash_bytes(&bytes));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn canonical_cli_refuses_output_escape() {
    let root = temp_root();
    let input_path = root.join("experiment.json");
    fs::write(
        &input_path,
        serde_json::to_vec_pretty(&experiment()).unwrap(),
    )
    .unwrap();
    let output = run(&root, "run-experiment", &input_path, Some("../escape.json"));
    assert!(!output.status.success());
    let _ = fs::remove_dir_all(root);
}
