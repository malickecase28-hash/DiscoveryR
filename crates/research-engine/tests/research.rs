use std::collections::{BTreeMap, BTreeSet};

use research_contracts::{
    CompleteReproducibilityIdentity, ContextPermission, DetectorRole, EvidenceAccess,
    EvidenceState, NativeScale, ProgramConsumer, ReproducibilityIdentity,
};
use research_engine::{
    run_research, CausalResearchRecord, DetectorAtlas, DetectorDescriptor, Direction, Operator,
    QuestionGenerationRequest, ResearchPlan,
};
use research_tape::{AnchorInstance, ContextObservation, SourceRef};

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
            NativeScale::Bar(research_contracts::BarScale::M5),
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
            NativeScale::Bar(research_contracts::BarScale::M5),
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
        native_scales: BTreeSet::from([NativeScale::Bar(research_contracts::BarScale::M5)]),
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

fn complete_identity() -> CompleteReproducibilityIdentity {
    CompleteReproducibilityIdentity {
        instrument_identity: "instrument-x".into(),
        producer_commit_identity: "producer-commit".into(),
        producer_source_identity: "producer-source".into(),
        producer_blob_identity: "producer-blob".into(),
        authority_version_identity: "authority-v1".into(),
        detector_version_identity: "detector-v1".into(),
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
        native_scale_identity: "tick".into(),
        availability_contract_identity: "availability-v1".into(),
    }
}

fn research_plan() -> ResearchPlan {
    ResearchPlan {
        atlas: DetectorAtlas::new(vec![
            detector("anchor", DetectorRole::StructuralObject, NativeScale::Tick),
            detector("event", DetectorRole::Event, NativeScale::Tick),
        ])
        .unwrap(),
        request: QuestionGenerationRequest {
            experiment_id: "exp-r".into(),
            instrument_ids: vec!["xauusd".into()],
            anchor_detector_id: "anchor".into(),
            anchor_ids: vec!["anchor-a".into()],
            context_detector_ids: vec!["event".into()],
            lifecycle_states: vec!["formed".into()],
            context_ids: vec!["event".into()],
            native_scales: BTreeSet::from([NativeScale::Tick]),
            anchor_time_ns: 100,
            context_available_time_ns: BTreeMap::from([("event".into(), 90)]),
            directions: vec![Direction::Positive],
            max_questions: 1,
        },
        permission: ContextPermission {
            permission_id: "r-market".into(),
            consumer: ProgramConsumer::MarketResearch,
            allowed_detector_ids: vec!["event".into()],
            allowed_scales: BTreeSet::from([NativeScale::Tick]),
            evidence_access: EvidenceAccess::UnconfirmedAllowed,
        },
        reproducibility: ReproducibilityIdentity {
            source_identity: "source-manifest".into(),
            binary_identity: "binary-v1".into(),
            configuration_identity: "configuration-v1".into(),
            parameter_identity: "parameters-v1".into(),
            seed_identity: "seed-v1".into(),
            output_identity: "declared-output".into(),
            user_identities: vec!["xauusd".into()],
        },
        complete_reproducibility: complete_identity(),
        max_records: 10,
    }
}

fn bound_record(row: u64, anchor_value: f64, context_value: f64) -> CausalResearchRecord {
    CausalResearchRecord::from_tape(
        "xauusd",
        "scope-development",
        "source-manifest",
        AnchorInstance {
            anchor_id: "anchor-a".into(),
            detector_id: "anchor".into(),
            lifecycle_state: "formed".into(),
            native_scale: NativeScale::Tick,
            anchor_time: 100,
            value: Some(anchor_value),
            occur_time: Some(80),
            object_id: Some(format!("anchor-object-{row}")),
            source: SourceRef {
                part: "anchors.parquet".into(),
                row_index: row,
            },
        },
        vec![ContextObservation {
            detector_id: "event".into(),
            native_scale: NativeScale::Tick,
            available_time: 90,
            value: Some(context_value),
            occur_time: Some(70),
            object_id: Some(format!("event-object-{row}")),
            source: SourceRef {
                part: "contexts.parquet".into(),
                row_index: row,
            },
        }],
    )
    .unwrap()
}

#[test]
fn program_r_binds_records_and_computes_paired_discovery() {
    let plan = research_plan();
    let records = vec![bound_record(1, 1.0, 2.0), bound_record(2, 2.0, 4.0)];
    let report = run_research(&plan, &records).unwrap();
    report.validate_identity().unwrap();
    assert_eq!(report.same_domain_discovery.len(), 1);
    assert_eq!(report.same_domain_discovery[0].correlation, Some(1.0));
    assert_eq!(report.same_domain_discovery[0].paired_observations, 2);

    let mut wrong_instrument = records.clone();
    wrong_instrument[0].instrument_id = "eurusd".into();
    assert!(run_research(&plan, &wrong_instrument).is_err());

    let mut missing_context = records;
    missing_context[0].contexts.clear();
    assert!(run_research(&plan, &missing_context).is_err());
}
