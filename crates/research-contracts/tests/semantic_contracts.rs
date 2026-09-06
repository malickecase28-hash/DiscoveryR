use research_contracts::*;
use serde_json::json;
use std::collections::BTreeSet;

fn authority() -> ProducerAuthority {
    ProducerAuthority {
        producer_id: "producer".into(),
        producer_version: "v1".into(),
        producer_commit: "commit".into(),
        state_version: "state".into(),
        parameter_identity: "params".into(),
        payload_contract_identity: "payload".into(),
        source_schema_identity: "schema".into(),
        availability_contract_identity: "availability".into(),
        source_blob_identity: "blob".into(),
        detector_version: DetectorVersion {
            detector_id: "detector".into(),
            detector_version: "dv1".into(),
            state_version: "ds1".into(),
        },
        required_native_scales: BTreeSet::from([NativeScale::Tick, NativeScale::Bar(BarScale::M1)]),
    }
}

fn policy() -> HoldoutPolicy {
    HoldoutPolicy {
        policy_id: "policy".into(),
        frozen_at_ns: 10,
        dataset_identity: "dataset".into(),
        leaves: vec![
            HoldoutLeaf {
                leaf_id: "dev".into(),
                scope: ScopeInterval {
                    instrument_id: "XAUUSD".into(),
                    scope_id: "scope-dev".into(),
                    dataset_identity: "dataset".into(),
                    start_time_ns: 0,
                    end_time_ns: 100,
                },
                role: HoldoutRole::Development,
            },
            HoldoutLeaf {
                leaf_id: "detector".into(),
                scope: ScopeInterval {
                    instrument_id: "XAUUSD".into(),
                    scope_id: "scope-detector".into(),
                    dataset_identity: "dataset".into(),
                    start_time_ns: 100,
                    end_time_ns: 200,
                },
                role: HoldoutRole::DetectorConfirmation,
            },
            HoldoutLeaf {
                leaf_id: "strategy".into(),
                scope: ScopeInterval {
                    instrument_id: "XAUUSD".into(),
                    scope_id: "scope-strategy".into(),
                    dataset_identity: "dataset".into(),
                    start_time_ns: 200,
                    end_time_ns: 300,
                },
                role: HoldoutRole::StrategyHoldout,
            },
        ],
        future_data_policy: None,
    }
}

#[test]
fn semantic_roles_and_scale_sets_are_typed() {
    let roles = [
        DetectorRole::LifecycleObject,
        DetectorRole::StructuralObject,
        DetectorRole::Event,
        DetectorRole::StateRegime,
        DetectorRole::DirectionalContext,
        DetectorRole::QualityInstrumentation,
        DetectorRole::NormalizationMeasure,
        DetectorRole::TemporalContext,
        DetectorRole::DerivedObject,
        DetectorRole::CompositeContext,
    ];
    assert_eq!(roles.len(), 10);
    let scales = serde_json::to_string(&NativeScale::Bar(BarScale::M15)).unwrap();
    assert_eq!(scales, r#"{"Bar":"15m"}"#);
}

#[test]
fn authority_reports_targeted_deltas_and_rejects_empty_identity() {
    let left = authority();
    let mut right = left.clone();
    right.source_schema_identity = "schema-2".into();
    right.required_native_scales.remove(&NativeScale::Tick);
    let report = validate_compatibility(&left, &right).unwrap();
    let AuthorityCompatibility::DeltaRequired { changed_surfaces } = report else {
        panic!("expected authority delta");
    };
    assert!(changed_surfaces
        .iter()
        .any(|delta| delta.surface == "source_schema_identity"));
    assert!(changed_surfaces
        .iter()
        .any(|delta| delta.surface == "required_native_scales"));
    assert_eq!(
        serde_json::to_value(AuthorityCompatibility::DeltaRequired { changed_surfaces }).unwrap()
            ["status"],
        "AUTHORITY_DELTA_REQUIRED"
    );
    let mut invalid = right;
    invalid.producer_id.clear();
    assert!(validate_compatibility(&left, &invalid).is_err());
}

#[test]
fn holdout_is_frozen_disjoint_and_exposure_is_append_only() {
    let mut history = ExposureHistory::new(policy()).unwrap();
    record_exposure(
        &mut history,
        ExposureEvent {
            event_id: "e1".into(),
            policy_id: "policy".into(),
            actor_id: "worker".into(),
            scope: ScopeInterval {
                instrument_id: "XAUUSD".into(),
                scope_id: "scope-detector".into(),
                dataset_identity: "dataset".into(),
                start_time_ns: 100,
                end_time_ns: 200,
            },
            role: HoldoutRole::DetectorConfirmation,
            exposed_at_ns: 11,
        },
    )
    .unwrap();
    let contaminated = ExposureEvent {
        event_id: "e2".into(),
        policy_id: "policy".into(),
        actor_id: "worker".into(),
        scope: ScopeInterval {
            instrument_id: "XAUUSD".into(),
            scope_id: "scope-detector".into(),
            dataset_identity: "dataset".into(),
            start_time_ns: 100,
            end_time_ns: 200,
        },
        role: HoldoutRole::StrategyHoldout,
        exposed_at_ns: 12,
    };
    assert!(record_exposure(&mut history, contaminated).is_err());
    let renamed = ExposureEvent {
        event_id: "e3".into(),
        policy_id: "policy".into(),
        actor_id: "worker".into(),
        scope: ScopeInterval {
            instrument_id: "XAUUSD".into(),
            scope_id: "renamed".into(),
            dataset_identity: "dataset".into(),
            start_time_ns: 0,
            end_time_ns: 100,
        },
        role: HoldoutRole::Development,
        exposed_at_ns: 12,
    };
    assert!(record_exposure(&mut history, renamed).is_err());
}

#[test]
fn causal_normalization_lineage_and_confirmation_fail_closed() {
    let basis = NormalizationBasis {
        basis_identity: "basis".into(),
        available_time_ns: 11,
        raw_value: 1.0,
        normalized_value: 2.0,
        formula: "x".into(),
    };
    assert!(basis.validate(10).is_err());
    let cycle = vec![
        DetectorLineage {
            detector_id: "a".into(),
            parent_detector_ids: vec!["b".into()],
        },
        DetectorLineage {
            detector_id: "b".into(),
            parent_detector_ids: vec!["a".into()],
        },
    ];
    assert!(validate_detector_lineage(&cycle).is_err());
    let mut contract: ConfirmationContract = serde_json::from_value(json!({
        "confirmation_id":"c","claim_identity":"claim","population_identity":"population","anchor_identity":"anchor","context_identity":"context","outcome_identity":"outcome","metric_identity":"metric","code_identity":"code","control_design_identity":"control","null_design_identity":"null","multiplicity_family_identity":"family","data_policy_identity":"data","holdout_policy_identity":"holdout","custodian_policy_identity":"custodian","actor":"WORKER","state":"CONFIRMED"
    })).unwrap();
    assert!(contract.validate().is_err());
    contract.actor = evidence::ConfirmationActor::Custodian;
    assert!(contract.validate().is_ok());
}

#[test]
fn envelope_retains_partial_null_and_path_independent_identity() {
    let identity = ReproducibilityIdentity {
        source_identity: "source".into(),
        binary_identity: "binary".into(),
        configuration_identity: "config".into(),
        parameter_identity: "params".into(),
        seed_identity: "seed".into(),
        output_identity: "output".into(),
        user_identities: vec!["user".into()],
    };
    assert!(identity.identity_hash().is_ok());
    let mut bad = identity.clone();
    bad.source_identity = r#"C:\data\source"#.into();
    assert!(bad.validate().is_err());
    let envelope = KnowledgeEnvelope {
        envelope_version: 1,
        knowledge_id: "finding".into(),
        record: KnowledgeRecord::Finding(FindingRecord {
            record_id: "finding".into(),
            content: json!({"state":"partial"}),
            status: FindingStatus::Null,
            provenance: Provenance {
                experiment_id: None,
                run_id: None,
                evidence_ids: vec![],
                code_identity: None,
                researcher_id: None,
            },
            created_utc: "now".into(),
            supersedes: None,
            superseded_by: None,
            rejection_reason: None,
        }),
        facets: KnowledgeFacets {
            instrument_ids: vec!["XAUUSD".into()],
            detector_ids: vec!["detector".into()],
            anchor_ids: vec!["anchor".into()],
            lifecycle_states: vec!["state".into()],
            context_ids: vec!["context".into()],
            native_scales: BTreeSet::from([NativeScale::Bar(BarScale::M1)]),
            evidence_ids: vec![],
            strategy_ids: vec![],
            portfolio_ids: vec![],
        },
    };
    assert!(envelope.validate().is_ok());
    assert!(KnowledgeQuery {
        instrument_id: Some("XAUUSD".into()),
        ..Default::default()
    }
    .matches(&envelope));
}

#[test]
fn additive_schemas_are_deterministic_and_named_once() {
    let first = generated_additive_schemas();
    let second = generated_additive_schemas();
    assert_eq!(first, second);
    let names: std::collections::HashSet<_> = first.iter().map(|(name, _)| *name).collect();
    assert_eq!(names.len(), first.len());
    assert!(first.iter().all(|(_, schema)| schema.ends_with('\n')));
}
