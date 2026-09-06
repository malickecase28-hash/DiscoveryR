use ed25519_dalek::{Signer, SigningKey};
use research_contracts::{
    BarScale, CompleteReproducibilityIdentity, ConfirmationActor, ConfirmationContract,
    ContextPermission, CostModel, DecisionRule, DetectorRole, EvidenceAccess, EvidenceState,
    ExecutionAssumption, HoldoutLeaf, HoldoutPolicy, HoldoutRole, NativeScale, PortfolioComponent,
    ProgramConsumer, RiskRule, ScopeInterval, StrategyHypothesis,
};
use research_engine::*;
use research_tape::{AnchorInstance, ContextObservation, SourceRef};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    process::Command,
};

fn metrics(id: &str) -> AttackMetrics {
    AttackMetrics {
        evidence_id: id.into(),
        sample_size: 4,
        violations: 0,
        denominator: 4,
        source_ids: vec!["source-a".into(), "source-b".into()],
        control_identity: Some("control".into()),
        block_count: 2,
        family_count: 1,
        fit_end_ns: Some(10),
        evaluation_start_ns: Some(11),
        perturbation_count: 2,
        max_sample_share: 0.25,
        alternative_explanations: vec!["alternative".into()],
        metric: 1.0,
    }
}
fn bundle(target: &str) -> ChallengeBundle {
    run_challenge_bundle(&ChallengeRunInput {
        target_identity: target.into(),
        evidence: evidence(),
    })
    .unwrap()
}
fn evidence() -> Vec<AttackEvidence> {
    vec![
        AttackEvidence::Lookahead {
            metrics: metrics("e-lookahead"),
        },
        AttackEvidence::PopulationConditioning {
            metrics: metrics("e-population"),
        },
        AttackEvidence::DenominatorErrors {
            metrics: metrics("e-denominator"),
        },
        AttackEvidence::SelectionBias {
            metrics: metrics("e-selection"),
        },
        AttackEvidence::LineageDuplication {
            metrics: metrics("e-lineage"),
        },
        AttackEvidence::FalseConfluence {
            metrics: metrics("e-confluence"),
        },
        AttackEvidence::BadControls {
            metrics: metrics("e-controls"),
        },
        AttackEvidence::TemporalDependence {
            metrics: metrics("e-temporal"),
        },
        AttackEvidence::MultipleTesting {
            metrics: metrics("e-multiple"),
        },
        AttackEvidence::NormalizationLeakage {
            metrics: metrics("e-normalization"),
        },
        AttackEvidence::Fragility {
            metrics: metrics("e-fragility"),
        },
        AttackEvidence::SampleConcentration {
            metrics: metrics("e-concentration"),
        },
        AttackEvidence::AlternativeExplanations {
            metrics: metrics("e-alternatives"),
        },
    ]
}
fn policy(instrument: &str) -> HoldoutPolicy {
    let scope = |id: &str, start: i64, end: i64| ScopeInterval {
        instrument_id: instrument.into(),
        scope_id: id.into(),
        dataset_identity: format!("dataset-{instrument}"),
        start_time_ns: start,
        end_time_ns: end,
    };
    HoldoutPolicy {
        policy_id: format!("holdout-{instrument}"),
        frozen_at_ns: 0,
        dataset_identity: format!("dataset-{instrument}"),
        leaves: vec![
            HoldoutLeaf {
                leaf_id: "development".into(),
                scope: scope("development-scope", 0, 10),
                role: HoldoutRole::Development,
            },
            HoldoutLeaf {
                leaf_id: "strategy".into(),
                scope: scope("strategy-scope", 10, 20),
                role: HoldoutRole::StrategyHoldout,
            },
        ],
        future_data_policy: None,
    }
}
fn contract(id: &str, holdout: &str) -> ConfirmationContract {
    let s = |prefix| format!("{prefix}-{id}");
    ConfirmationContract {
        confirmation_id: id.into(),
        claim_identity: s("claim"),
        population_identity: s("population"),
        anchor_identity: s("anchor"),
        context_identity: s("context"),
        outcome_identity: s("outcome"),
        metric_identity: s("metric"),
        code_identity: s("code"),
        control_design_identity: s("control"),
        null_design_identity: s("null"),
        multiplicity_family_identity: s("family"),
        data_policy_identity: s("data"),
        holdout_policy_identity: holdout.into(),
        custodian_policy_identity: "custodian-policy".into(),
        actor: ConfirmationActor::Custodian,
        state: research_contracts::ConfirmationState::Confirmed,
    }
}
fn artifact(request: &str, challenge: &str, nonce: &str) -> Vec<u8> {
    let mut a = SignedCustodianArtifact {
        authorization_id: format!("auth-{nonce}"),
        custodian_policy_identity: "custodian-policy".into(),
        request_identity: request.into(),
        challenge_identity: challenge.into(),
        actor_identity: "custodian-actor".into(),
        nonce: nonce.into(),
        expires_at_ns: 1_000_000,
        replay_identity: format!("replay-{nonce}"),
        signature_hex: String::new(),
        public_key_hex: custodian_public_key_hex(),
    };
    let key = SigningKey::from_bytes(&[
        0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c,
        0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae,
        0x7f, 0x60,
    ]);
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        a.authorization_id,
        a.custodian_policy_identity,
        a.request_identity,
        a.challenge_identity,
        a.actor_identity,
        a.nonce,
        a.expires_at_ns,
        a.replay_identity
    );
    a.signature_hex = a_hex(&key.sign(payload.as_bytes()).to_bytes());
    serde_json::to_vec(&a).unwrap()
}
fn a_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn complete(instrument: &str) -> CompleteReproducibilityIdentity {
    let s = |prefix| format!("{prefix}-{instrument}");
    CompleteReproducibilityIdentity {
        instrument_identity: instrument.into(),
        producer_commit_identity: s("commit"),
        producer_source_identity: s("source"),
        producer_blob_identity: s("blob"),
        authority_version_identity: s("authority"),
        detector_version_identity: s("detector"),
        parameter_identity: s("parameter"),
        source_manifest_identity: s("source-manifest"),
        payload_manifest_identity: s("payload-manifest"),
        data_scope_identity: s("scope"),
        experiment_contract_identity: s("experiment"),
        code_identity: s("code"),
        scanner_version_identity: s("scanner"),
        control_design_identity: s("control"),
        null_design_identity: s("null"),
        seed_identity: s("seed"),
        output_identity: s("output"),
        native_scale_identity: s("m1"),
        availability_contract_identity: s("availability"),
    }
}
fn run_one(instrument: &str) -> (String, String) {
    let scale = NativeScale::Bar(BarScale::M1);
    let atlas = DetectorAtlas::new(vec![
        DetectorDescriptor::new(
            "anchor",
            vec![DetectorRole::StructuralObject],
            BTreeSet::from([scale.clone()]),
        )
        .unwrap(),
        DetectorDescriptor::new(
            "context",
            vec![DetectorRole::DirectionalContext],
            BTreeSet::from([scale.clone()]),
        )
        .unwrap(),
    ])
    .unwrap();
    let request = QuestionGenerationRequest {
        experiment_id: format!("exp-{instrument}"),
        instrument_ids: vec![instrument.into()],
        anchor_detector_id: "anchor".into(),
        anchor_ids: vec!["anchor-1".into()],
        context_detector_ids: vec!["context".into()],
        lifecycle_states: vec!["formed".into()],
        context_ids: vec!["context-1".into()],
        native_scales: BTreeSet::from([scale.clone()]),
        anchor_time_ns: 11,
        context_available_time_ns: BTreeMap::from([(String::from("context"), 10)]),
        directions: vec![Direction::Positive],
        max_questions: 4,
    };
    let permission = ContextPermission {
        permission_id: format!("permission-{instrument}"),
        consumer: ProgramConsumer::MarketResearch,
        allowed_detector_ids: vec!["context".into()],
        allowed_scales: BTreeSet::from([scale]),
        evidence_access: EvidenceAccess::UnconfirmedAllowed,
    };
    let records = vec![CausalResearchRecord::from_tape(
        AnchorInstance {
            anchor_id: "anchor-1".into(),
            detector_id: "anchor".into(),
            lifecycle_state: "formed".into(),
            native_scale: NativeScale::Bar(BarScale::M1),
            anchor_time: 11,
            value: Some(1.0),
            occur_time: Some(11),
            object_id: Some("object-1".into()),
            source: SourceRef {
                part: format!("{instrument}-part"),
                row_index: 0,
            },
        },
        vec![ContextObservation {
            detector_id: "context".into(),
            native_scale: NativeScale::Bar(BarScale::M1),
            available_time: 10,
            value: Some(0.5),
            occur_time: Some(10),
            object_id: Some("context-object".into()),
            source: SourceRef {
                part: format!("{instrument}-context"),
                row_index: 0,
            },
        }],
    )
    .unwrap()];
    let compact = research_contracts::ReproducibilityIdentity {
        source_identity: format!("source-{instrument}"),
        binary_identity: format!("binary-{instrument}"),
        configuration_identity: format!("config-{instrument}"),
        parameter_identity: format!("parameter-{instrument}"),
        seed_identity: format!("seed-{instrument}"),
        output_identity: format!("output-{instrument}"),
        user_identities: vec![],
    };
    let research = run_research(
        &ResearchPlan {
            atlas,
            request,
            permission,
            reproducibility: compact,
            complete_reproducibility: complete(instrument),
            max_records: 4,
        },
        &records,
    )
    .unwrap();
    let policy = policy(instrument);
    let policy_id = policy.identity_hash().unwrap();
    let mut overlapping = policy.clone();
    overlapping.leaves[1].scope.start_time_ns = 9;
    assert!(WalkForwardPlan::from_policy(&overlapping, "development", "strategy", None).is_err());
    let plan = WalkForwardPlan::from_policy(&policy, "development", "strategy", None).unwrap();
    let input_id = format!("detector-confirmation-{instrument}");
    let detector_request = DetectorConfirmationRequest {
        contract: contract(&format!("detector-{instrument}"), &policy_id),
        target_identity: research.native_phenotype.identity.clone(),
        evidence_state: EvidenceState::Known,
        holdout_policy_identity: policy_id.clone(),
    };
    let detector_bundle = bundle(&detector_request.target_identity);
    let detector_req_id = sha_json(&(
        &detector_request.contract,
        &detector_request.target_identity,
        &detector_request.holdout_policy_identity,
    ));
    let detector = confirm_detector_from_artifact(
        &detector_request,
        &detector_bundle,
        &artifact(
            &detector_req_id,
            detector_bundle.identity(),
            &format!("detector-{instrument}"),
        ),
        1,
        &ReplayGuard::default(),
    )
    .unwrap();
    let manifest = ConfirmedInputManifest {
        strategy_id: format!("strategy-{instrument}"),
        holdout_policy_identity: policy_id.clone(),
        inputs: vec![ConfirmedBehavioralInput {
            input_id: input_id.clone(),
            confirmed: true,
            available_time_ns: 0,
        }],
    };
    let spec = StrategySpec {
        archetype: StrategyArchetype::Continuation,
        hypothesis: StrategyHypothesis {
            strategy_id: manifest.strategy_id.clone(),
            hypothesis_id: format!("hypothesis-{instrument}"),
            confirmed_finding_ids: vec![input_id.clone()],
            parameters: BTreeMap::from([(String::from("entry"), json!(0.5))]),
        },
        decision_rule: DecisionRule {
            rule_id: format!("rule-{instrument}"),
            entry: "signal threshold".into(),
            exit: "one observation".into(),
            sizing: "unit analytical exposure".into(),
            management: "abstain on missing input".into(),
            parameters: BTreeMap::from([(String::from("entry_threshold"), json!(0.5))]),
        },
        execution_assumption: ExecutionAssumption {
            assumption_id: format!("execution-{instrument}"),
            assumptions: BTreeMap::from([(String::from("mode"), json!("analytical"))]),
        },
        cost_model: CostModel {
            cost_model_id: format!("cost-{instrument}"),
            parameters: BTreeMap::from([(String::from("per_observation"), json!(0.1))]),
        },
        risk_rule: RiskRule {
            risk_rule_id: format!("risk-{instrument}"),
            parameters: BTreeMap::from([(String::from("max_abs_outcome"), json!(2.0))]),
        },
    };
    let observations = vec![
        StrategyObservation {
            timestamp_ns: 1,
            available_time_ns: 1,
            signal: Some(1.0),
            outcome: Some(1.0),
        },
        StrategyObservation {
            timestamp_ns: 11,
            available_time_ns: 11,
            signal: Some(1.0),
            outcome: Some(0.5),
        },
    ];
    let walk = run_walk_forward_fitted(
        &spec,
        &manifest,
        &observations,
        &plan,
        &SimulationConfig::default(),
    )
    .unwrap();
    let strategy_request = ConfirmationRequest {
        contract: contract(&manifest.strategy_id, &policy_id),
        validation: walk.validation.clone(),
        manifest: manifest.clone(),
        plan: plan.clone(),
    };
    let strategy_bundle = bundle(&walk.validation.validation_id);
    let strategy_req_id = sha_json(&(
        &strategy_request.contract,
        &strategy_request.validation,
        &strategy_request.manifest,
        &strategy_request.plan.policy,
        &strategy_request.plan.development_leaf_id,
        &strategy_request.plan.strategy_holdout_leaf_id,
        &strategy_request.plan.future_leaf_id,
    ));
    let guard = ReplayGuard::default();
    let lock = confirm_strategy_from_artifact(
        &strategy_request,
        &strategy_bundle,
        &artifact(
            &strategy_req_id,
            strategy_bundle.identity(),
            &format!("strategy-{instrument}"),
        ),
        1,
        &guard,
    )
    .unwrap();
    assert!(matches!(
        confirm_strategy_from_artifact(
            &strategy_request,
            &strategy_bundle,
            &artifact(
                &strategy_req_id,
                strategy_bundle.identity(),
                &format!("strategy-{instrument}")
            ),
            1,
            &guard,
        ),
        Err(ConfirmationError::Replay)
    ));
    let mut forged: serde_json::Value = serde_json::from_slice(&artifact(
        &strategy_req_id,
        strategy_bundle.identity(),
        &format!("forged-{instrument}"),
    ))
    .unwrap();
    forged["public_key_hex"] = json!("00");
    assert!(matches!(
        confirm_strategy_from_artifact(
            &strategy_request,
            &strategy_bundle,
            &serde_json::to_vec(&forged).unwrap(),
            1,
            &ReplayGuard::default(),
        ),
        Err(ConfirmationError::InvalidSignature)
    ));
    assert_eq!(detector.target_identity, research.native_phenotype.identity);
    let _custodied = CustodiedInputManifest::issue(manifest.clone(), lock.clone()).unwrap();
    let streams = vec![AlignedStrategyStream {
        stream: StrategyStream {
            strategy_id: manifest.strategy_id.clone(),
            confirmation: research_contracts::ConfirmationState::Confirmed,
            evidence_state: EvidenceState::Known,
            returns: vec![Some(0.1)],
            signals: vec![Some(1.0)],
            regimes: vec![Some("r".into())],
            capital: Some(1.0),
            capacity: Some(1.0),
            liquidity: vec![Some(1.0)],
            risk_budget: Some(1.0),
        },
        timestamps_ns: vec![11],
        availability_ns: vec![11],
        instrument_scope_identity: format!("scope-{instrument}"),
        source_identity: format!("stream-{instrument}"),
        time_grid_identity: canonical_time_grid_identity(&[11], &[11]),
        context_permission: ContextPermission {
            permission_id: format!("portfolio-{instrument}"),
            consumer: ProgramConsumer::PortfolioResearch,
            allowed_detector_ids: vec!["context".into()],
            allowed_scales: BTreeSet::from([NativeScale::Bar(BarScale::M1)]),
            evidence_access: EvidenceAccess::ConfirmedOnly,
        },
        confirmation: lock,
    }];
    let mut mismatched_grid = streams[0].clone();
    mismatched_grid.timestamps_ns[0] = 12;
    assert!(validate_aligned_streams(&[mismatched_grid]).is_err());
    let component = PortfolioComponent {
        component_id: format!("component-{instrument}"),
        confirmed_strategy_ids: vec![manifest.strategy_id.clone()],
        parameters: BTreeMap::from([(String::from("mode"), json!("synthetic"))]),
    };
    let portfolio = portfolio_report_aligned(component.clone(), &streams, vec![], &[]).unwrap();
    let portfolio_request = PortfolioConfirmationRequest {
        contract: contract(&component.component_id, &policy_id),
        component_identity: component.component_id.clone(),
        strategy_confirmation_ids: vec![manifest.strategy_id.clone()],
        holdout_policy_identity: policy_id,
        evidence_state: EvidenceState::Known,
    };
    let portfolio_bundle = bundle(&component.component_id);
    let portfolio_req_id = sha_json(&(
        &portfolio_request.contract,
        &portfolio_request.component_identity,
        &portfolio_request.strategy_confirmation_ids,
        &portfolio_request.holdout_policy_identity,
    ));
    let _portfolio_lock = confirm_portfolio_from_artifact(
        &portfolio_request,
        &portfolio_bundle,
        &artifact(
            &portfolio_req_id,
            portfolio_bundle.identity(),
            &format!("portfolio-{instrument}"),
        ),
        1,
        &ReplayGuard::default(),
    )
    .unwrap();
    assert!(run_challenge_bundle(&ChallengeRunInput {
        target_identity: "duplicate".into(),
        evidence: {
            let mut e = evidence();
            e[1] = e[0].clone();
            e
        }
    })
    .is_err());
    assert!(run_challenge_bundle(&ChallengeRunInput {
        target_identity: "missing".into(),
        evidence: evidence()[..12].to_vec()
    })
    .is_err());
    let optimized = optimize_synthetic(&PortfolioInput {
        component,
        strategies: streams.into_iter().map(|s| s.stream).collect(),
        constraints: vec![],
    })
    .unwrap();
    (
        research.output_identity,
        format!(
            "{}:{}:{}",
            walk.validation.validation_id,
            portfolio.correlations.len(),
            optimized.objective
        ),
    )
}
fn sha_json<T: serde::Serialize>(v: &T) -> String {
    let mut h = sha2::Sha256::new();
    use sha2::Digest;
    h.update(serde_json::to_vec(v).unwrap());
    format!("{:x}", h.finalize())
}

#[test]
fn synthetic_two_instrument_pipeline_is_bound_and_reproducible() {
    let a = run_one("XAUUSD");
    let b = run_one("EURUSD");
    assert_ne!(a, b);
    assert_eq!(a, run_one("XAUUSD"));
    let input = std::env::temp_dir().join(format!("rsp-e2e-{}.json", std::process::id()));
    std::fs::write(&input, serde_json::to_vec(&json!({"challenge_id":"c","target_identity":"t","attack_kind":"LOOKAHEAD","protocol_identity":"p","result":"PASSED","evidence_ids":["e"]})).unwrap()).unwrap();
    let exe = std::env::var("CARGO_BIN_EXE_research_engine")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_research-engine"))
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("research-engine.exe")
                .to_string_lossy()
                .into_owned()
        });
    let first = Command::new(&exe)
        .args(["challenge-result", input.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(&exe)
        .args(["challenge-result", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(first.status.success() && second.status.success());
    assert_eq!(first.stdout, second.stdout);
    let _ = std::fs::remove_file(input);
}
