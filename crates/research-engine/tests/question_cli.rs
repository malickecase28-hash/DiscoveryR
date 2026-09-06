use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use research_contracts::{
    BarScale, ContextPermission, DetectorRole, EvidenceAccess, NativeScale, ProgramConsumer,
};
use research_engine::{
    DetectorDescriptorInput, QuestionTemplateReport, QuestionTemplateRequest, TemplateDirection,
};

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("trinityr-question-cli-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn request() -> QuestionTemplateRequest {
    QuestionTemplateRequest {
        experiment_id: "qg-v1".into(),
        instrument_ids: vec!["xauusd".into()],
        phenotype_identity: "phenotype-frozen-v1".into(),
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["formation".into()],
        lifecycle_states: vec!["formed".into()],
        context_detector_ids: vec!["context".into()],
        anchor_scales: BTreeSet::from([NativeScale::Tick]),
        context_scales: BTreeSet::from([NativeScale::Bar(BarScale::M5)]),
        directions: vec![TemplateDirection::Positive],
        max_questions: 20,
        detectors: vec![
            DetectorDescriptorInput {
                detector_id: "anchor".into(),
                roles: vec![DetectorRole::Event],
                native_scales: BTreeSet::from([NativeScale::Tick]),
                derives_from: vec![],
            },
            DetectorDescriptorInput {
                detector_id: "context".into(),
                roles: vec![DetectorRole::StateRegime],
                native_scales: BTreeSet::from([NativeScale::Bar(BarScale::M5)]),
                derives_from: vec![],
            },
        ],
        permission: ContextPermission {
            permission_id: "r-question-generation".into(),
            consumer: ProgramConsumer::MarketResearch,
            allowed_detector_ids: vec!["context".into()],
            allowed_scales: BTreeSet::from([NativeScale::Bar(BarScale::M5)]),
            evidence_access: EvidenceAccess::UnconfirmedAllowed,
        },
    }
}

#[test]
fn canonical_cli_generates_cross_scale_templates_without_outcomes() {
    let root = temp_root();
    let input = root.join("question-request.json");
    fs::write(&input, serde_json::to_vec_pretty(&request()).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_trinity_research"))
        .env("TRINITYR_WORKSPACE", &root)
        .arg("generate-questions")
        .arg(&input)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: QuestionTemplateReport = serde_json::from_slice(&output.stdout).unwrap();
    report.validate_identity().unwrap();
    assert!(!report.questions.is_empty());
    assert!(report.questions.iter().all(|question| {
        question.anchor_scale == NativeScale::Tick
            && question.context_scale == NativeScale::Bar(BarScale::M5)
            && !question.same_native_scale
    }));
    let _ = fs::remove_dir_all(root);
}
