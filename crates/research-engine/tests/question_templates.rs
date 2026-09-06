use std::collections::BTreeSet;

use research_contracts::{
    BarScale, ContextPermission, DetectorRole, EvidenceAccess, NativeScale, ProgramConsumer,
};
use research_engine::{
    generate_question_templates, DetectorDescriptorInput, QuestionTemplateRequest,
    TemplateDirection, TemplateRejectionReason,
};

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
        directions: vec![TemplateDirection::Positive, TemplateDirection::Negative],
        max_questions: 100,
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
fn generator_preserves_cross_scale_pairs_without_observed_timestamps() {
    let report = generate_question_templates(&request()).unwrap();
    report.validate_identity().unwrap();
    assert!(!report.questions.is_empty());
    assert!(report.questions.iter().all(|question| {
        question.anchor_scale == NativeScale::Tick
            && question.context_scale == NativeScale::Bar(BarScale::M5)
            && !question.same_native_scale
            && question.phenotype_identity == "phenotype-frozen-v1"
    }));
}

#[test]
fn generator_rejects_lineage_dependent_context() {
    let mut value = request();
    value.detectors[1].derives_from = vec!["anchor".into()];
    let report = generate_question_templates(&value).unwrap();
    assert!(report.questions.is_empty());
    assert!(report
        .rejected
        .iter()
        .any(|item| item.reason == TemplateRejectionReason::LineageDependent));
}

#[test]
fn generator_is_bounded_deterministically() {
    let mut value = request();
    value.max_questions = 1;
    let first = generate_question_templates(&value).unwrap();
    let second = generate_question_templates(&value).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.questions.len(), 1);
    assert!(first.bounded);
    assert!(first
        .rejected
        .iter()
        .any(|item| item.reason == TemplateRejectionReason::BoundedOut));
}

#[test]
fn generator_has_no_outcome_or_prior_winner_input_surface() {
    let json = serde_json::to_value(request()).unwrap();
    let object = json.as_object().unwrap();
    assert!(!object.contains_key("outcomes"));
    assert!(!object.contains_key("findings"));
    assert!(!object.contains_key("prior_winner_ids"));
    assert!(!object.contains_key("anchor_time_ns"));
    assert!(!object.contains_key("context_available_time_ns"));
}
