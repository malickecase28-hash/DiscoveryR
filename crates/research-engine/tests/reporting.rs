use research_contracts::CompleteReproducibilityIdentity;
use research_engine::{generate_report_manifest, ReportManifestInput, ReportProgram};

fn identity() -> CompleteReproducibilityIdentity {
    CompleteReproducibilityIdentity {
        instrument_identity: "instrument-x".into(),
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
        output_identity: "artifact-v1".into(),
        native_scale_identity: "tick".into(),
        availability_contract_identity: "availability-v1".into(),
    }
}

#[test]
fn report_manifest_is_deterministic_and_bound_to_artifact_bytes() {
    let input = ReportManifestInput {
        program: ReportProgram::MarketResearch,
        artifact_kind: "PROGRAM_R_RESULT".into(),
        artifact_identity: "artifact-v1".into(),
        reproducibility: identity(),
    };
    let first = generate_report_manifest(&input, b"alpha").unwrap();
    let second = generate_report_manifest(&input, b"alpha").unwrap();
    let changed = generate_report_manifest(&input, b"beta").unwrap();
    first.validate_identity().unwrap();
    assert_eq!(first, second);
    assert_ne!(first.artifact_sha256, changed.artifact_sha256);
    assert_ne!(first.report_identity, changed.report_identity);
}
