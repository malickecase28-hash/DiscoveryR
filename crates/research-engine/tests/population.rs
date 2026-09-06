use std::collections::BTreeSet;

use research_contracts::{
    BarScale, CompleteReproducibilityIdentity, ContextPermission, DetectorRole, EvidenceAccess,
    NativeScale, ProgramConsumer, ReproducibilityIdentity,
};
use research_engine::{
    run_population_research, CandidateGate, DetectorDescriptorInput, MetricKind, MetricRequest,
    PopulationRecordInput, PopulationRunInput,
};
use research_tape::{AnchorInstance, ContextObservation, SourceRef};

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

fn input() -> PopulationRunInput {
    let tick = NativeScale::Tick;
    let m5 = NativeScale::Bar(BarScale::M5);
    PopulationRunInput {
        detectors: vec![
            DetectorDescriptorInput {
                detector_id: "anchor".into(),
                roles: vec![DetectorRole::StructuralObject],
                native_scales: BTreeSet::from([tick]),
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
            question_id: "q-cross-scale-context".into(),
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

#[test]
fn population_runner_accepts_many_anchor_times_and_preserves_cross_scale_identity() {
    let report = run_population_research(&input()).unwrap();
    report.validate_identity().unwrap();
    assert_eq!(report.anchor_instances, 2);
    assert_eq!(report.scope_start_ns, Some(100));
    assert_eq!(report.scope_end_ns, Some(200));
    assert_eq!(report.discoveries.len(), 1);
    assert!(!report.discoveries[0].same_native_scale);
    assert_eq!(report.discoveries[0].anchor_population, 2);
    assert_eq!(report.discoveries[0].observations, 2);
    assert_eq!(report.discoveries[0].value, Some(1.0));
    assert_eq!(report.candidate_ids.len(), 1);
}

#[test]
fn future_context_is_rejected_per_anchor_not_by_one_global_timestamp() {
    let mut value = input();
    value.records[1].contexts[0].available_time = 201;
    assert!(run_population_research(&value).is_err());
}

#[test]
fn missing_context_does_not_remove_the_anchor_population() {
    let mut value = input();
    value.records[1].contexts.clear();
    let report = run_population_research(&value).unwrap();
    let discovery = &report.discoveries[0];
    assert_eq!(discovery.anchor_population, 2);
    assert_eq!(discovery.context_present, 1);
    assert_eq!(discovery.missing_context, 1);
    assert_eq!(discovery.observations, 1);
    assert!(discovery.value.is_none());
    assert!(report.candidate_ids.is_empty());
}

#[test]
fn candidate_promotion_requires_an_explicit_predeclared_gate() {
    let mut value = input();
    value.metric_requests[0].candidate_gate = None;
    let report = run_population_research(&value).unwrap();
    assert!(report.candidate_ids.is_empty());
    assert_eq!(
        report.candidate_evaluations[0].reason,
        "NO_PREDECLARED_GATE"
    );
}

#[test]
fn lineage_dependent_context_is_rejected_as_independent_context() {
    let mut value = input();
    value.detectors[1].derives_from = vec!["anchor".into()];
    assert!(run_population_research(&value).is_err());
}

#[test]
fn generic_metric_cannot_masquerade_as_an_unimplemented_semantic_operator() {
    let mut value = input();
    value.metric_requests[0].operator = "nesting".into();
    assert!(run_population_research(&value).is_err());
}

#[test]
fn phenotype_only_e1_requires_no_context_metric() {
    let mut value = input();
    value.metric_requests.clear();
    for record in &mut value.records {
        record.contexts.clear();
    }
    let report = run_population_research(&value).unwrap();
    assert_eq!(report.anchor_instances, 2);
    assert!(report.discoveries.is_empty());
    assert!(report.candidate_ids.is_empty());
    assert_eq!(report.phenotypes[0].anchor_instances, 2);
    assert_eq!(report.phenotypes[0].value_samples, 2);
}
