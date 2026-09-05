use std::collections::{BTreeMap, BTreeSet};

use research_contracts::{
    ContextPermission, DetectorRole, EvidenceAccess, EvidenceState, NativeScale, ProgramConsumer,
};
use research_engine::{
    DetectorAtlas, DetectorDescriptor, Direction, Operator, QuestionGenerationRequest,
};

fn detector(id: &str, role: DetectorRole, scale: NativeScale) -> DetectorDescriptor {
    DetectorDescriptor::new(id, vec![role], BTreeSet::from([scale])).unwrap()
}

#[test]
fn atlas_dispatches_only_role_compatible_operators() {
    assert_eq!(Operator::ALL.len(), 11);
    let atlas = DetectorAtlas::new(vec![
        detector(
            "anchor",
            DetectorRole::LifecycleObject,
            NativeScale::Bar(research_contracts::BarScale::M1),
        ),
        detector(
            "event",
            DetectorRole::Event,
            NativeScale::Bar(research_contracts::BarScale::M5),
        ),
    ])
    .unwrap();

    let lifecycle = atlas.operators_for("anchor").unwrap();
    assert!(lifecycle.contains(&Operator::Lifecycle));
    assert!(!lifecycle.contains(&Operator::IncrementalInformation));
    assert!(atlas
        .operators_for("event")
        .unwrap()
        .contains(&Operator::Temporal));
}

#[test]
fn question_generation_is_bounded_blind_and_causal() {
    let atlas = DetectorAtlas::new(vec![
        detector(
            "anchor",
            DetectorRole::LifecycleObject,
            NativeScale::Bar(research_contracts::BarScale::M1),
        ),
        detector(
            "event",
            DetectorRole::Event,
            NativeScale::Bar(research_contracts::BarScale::M5),
        ),
        detector(
            "state",
            DetectorRole::StateRegime,
            NativeScale::Bar(research_contracts::BarScale::M15),
        ),
    ])
    .unwrap();
    let permission = ContextPermission {
        permission_id: "r-market".into(),
        consumer: ProgramConsumer::MarketResearch,
        allowed_detector_ids: vec!["event".into(), "state".into()],
        allowed_scales: BTreeSet::from([
            NativeScale::Bar(research_contracts::BarScale::M5),
            NativeScale::Bar(research_contracts::BarScale::M15),
        ]),
        evidence_access: EvidenceAccess::UnconfirmedAllowed,
    };
    let mut availability = BTreeMap::new();
    availability.insert("event".into(), 90);
    availability.insert("state".into(), 110);
    let request = QuestionGenerationRequest {
        experiment_id: "exp-r".into(),
        instrument_ids: vec!["xauusd".into(), "eurusd".into()],
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["anchor-a".into()],
        context_detector_ids: vec!["event".into(), "state".into()],
        lifecycle_states: vec!["formed".into()],
        context_ids: vec!["session".into()],
        native_scales: BTreeSet::from([NativeScale::Bar(research_contracts::BarScale::M1)]),
        anchor_time_ns: 100,
        context_available_time_ns: availability,
        directions: vec![Direction::Positive, Direction::Negative],
        max_questions: 3,
    };
    let report = atlas.generate_questions(&request, &permission).unwrap();
    assert_eq!(report.questions.len(), 3);
    assert!(report
        .questions
        .iter()
        .all(|q| q.question.evidence_state == EvidenceState::Unknown));
    assert!(report
        .questions
        .iter()
        .all(|q| q.prior_winner_ids.is_empty()));
    assert!(report.rejected.iter().any(|r| r.detector_id == "state"));
    assert!(report
        .questions
        .iter()
        .all(|q| q.question.instrument_ids == request.instrument_ids));
    assert!(report
        .questions
        .iter()
        .all(|q| q.question.native_scales == request.native_scales));
}

#[test]
fn generation_preserves_negative_and_partial_evidence_without_reading_winners() {
    let atlas = DetectorAtlas::new(vec![
        detector("anchor", DetectorRole::StructuralObject, NativeScale::Tick),
        detector(
            "quality",
            DetectorRole::QualityInstrumentation,
            NativeScale::Tick,
        ),
        detector("state", DetectorRole::StateRegime, NativeScale::Tick),
    ])
    .unwrap();
    let permission = ContextPermission {
        permission_id: "r-market".into(),
        consumer: ProgramConsumer::MarketResearch,
        allowed_detector_ids: vec!["quality".into()],
        allowed_scales: BTreeSet::from([NativeScale::Tick]),
        evidence_access: EvidenceAccess::UnconfirmedAllowed,
    };
    let request = QuestionGenerationRequest {
        experiment_id: "exp-r".into(),
        instrument_ids: vec!["xauusd".into()],
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["anchor-a".into()],
        context_detector_ids: vec!["quality".into(), "state".into()],
        lifecycle_states: vec![],
        context_ids: vec![],
        native_scales: BTreeSet::from([NativeScale::Tick]),
        anchor_time_ns: 10,
        context_available_time_ns: BTreeMap::from([
            (String::from("quality"), 10),
            (String::from("state"), 10),
        ]),
        directions: vec![Direction::Positive, Direction::Negative],
        max_questions: 10,
    };
    let report = atlas.generate_questions(&request, &permission).unwrap();
    assert!(!report.questions.is_empty());
    assert_eq!(report.status, EvidenceState::Partial);
    assert_eq!(
        report.questions[0].question.evidence_state,
        EvidenceState::Unknown
    );
    assert!(report
        .questions
        .iter()
        .any(|question| question.direction == Direction::Negative));
}

#[test]
fn derived_detectors_are_not_treated_as_independent_context() {
    let anchor = detector("anchor", DetectorRole::StructuralObject, NativeScale::Tick);
    let derived = detector("derived", DetectorRole::DerivedObject, NativeScale::Tick)
        .with_lineage(vec!["anchor".into()])
        .unwrap();
    let atlas = DetectorAtlas::new(vec![anchor, derived]).unwrap();
    let permission = ContextPermission {
        permission_id: "r-market".into(),
        consumer: ProgramConsumer::MarketResearch,
        allowed_detector_ids: vec!["derived".into()],
        allowed_scales: BTreeSet::from([NativeScale::Tick]),
        evidence_access: EvidenceAccess::UnconfirmedAllowed,
    };
    let request = QuestionGenerationRequest {
        experiment_id: "exp-r".into(),
        instrument_ids: vec!["xauusd".into()],
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["anchor-a".into()],
        context_detector_ids: vec!["derived".into()],
        lifecycle_states: vec![],
        context_ids: vec![],
        native_scales: BTreeSet::from([NativeScale::Tick]),
        anchor_time_ns: 10,
        context_available_time_ns: BTreeMap::from([(String::from("derived"), 10)]),
        directions: vec![Direction::Bidirectional],
        max_questions: 10,
    };
    let report = atlas.generate_questions(&request, &permission).unwrap();
    assert!(report.questions.is_empty());
    assert_eq!(report.status, EvidenceState::Rejected);
}
