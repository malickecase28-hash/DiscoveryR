use ed25519_dalek::{Signer, SigningKey};
use research_contracts::{ConfirmationActor, ConfirmationContract, ConfirmationState};
use research_engine::{
    run_challenge_bundle, AttackEvidence, AttackMetrics, ChallengeRunInput, ConfirmationBinding,
    ConfirmationError, ConfirmationKind, ConfirmationRunner, SignedCustodianArtifact,
};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, time::SystemTime};

fn unique_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rsp-authority-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn ephemeral_key(label: &str) -> SigningKey {
    let mut hash = Sha256::new();
    hash.update(label.as_bytes());
    hash.update(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes(),
    );
    hash.update(std::process::id().to_le_bytes());
    SigningKey::from_bytes(&hash.finalize().into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn write_trust_root(root: &PathBuf, key: &SigningKey) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    let path = root.join("custodian-trust.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "policy_identity": "custodian-policy-v1",
            "public_key_hex": hex(&key.verifying_key().to_bytes()),
            "replay_directory": "replay"
        }))
        .unwrap(),
    )
    .unwrap();
    path
}

fn binding() -> ConfirmationBinding {
    ConfirmationBinding::new(
        ConfirmationKind::Strategy,
        ConfirmationContract {
            confirmation_id: "strategy-confirmation-1".into(),
            claim_identity: "strategy-1".into(),
            population_identity: "strategy-population".into(),
            anchor_identity: "strategy-anchor".into(),
            context_identity: "strategy-context".into(),
            outcome_identity: "strategy-outcome".into(),
            metric_identity: "strategy-metric".into(),
            code_identity: "strategy-code".into(),
            control_design_identity: "strategy-controls".into(),
            null_design_identity: "strategy-null".into(),
            multiplicity_family_identity: "strategy-family".into(),
            data_policy_identity: "strategy-data-policy".into(),
            holdout_policy_identity: "holdout-policy-sha256".into(),
            custodian_policy_identity: "custodian-policy-v1".into(),
            actor: ConfirmationActor::Custodian,
            state: ConfirmationState::Confirmed,
        },
        "strategy-1",
        "walk-forward-report-sha256",
        "challenge-bundle-sha256",
        vec![
            "development-scope-sha256".into(),
            "strategy-scope-sha256".into(),
        ],
        Some("confirmed-input-manifest-sha256".into()),
        vec!["detector-confirmation-1".into()],
        "complete-reproducibility-sha256",
    )
    .unwrap()
}

#[test]
fn binding_rejects_non_confirmed_contracts_and_wrong_claims() {
    let mut contract = binding().contract().clone();
    contract.state = ConfirmationState::Inconclusive;
    assert!(ConfirmationBinding::new(
        ConfirmationKind::Strategy,
        contract,
        "strategy-1",
        "report",
        "challenge",
        vec!["scope".into()],
        Some("manifest".into()),
        vec!["input-confirmation".into()],
        "reproducibility",
    )
    .is_err());

    let mut contract = binding().contract().clone();
    contract.claim_identity = "another-strategy".into();
    assert!(ConfirmationBinding::new(
        ConfirmationKind::Strategy,
        contract,
        "strategy-1",
        "report",
        "challenge",
        vec!["scope".into()],
        Some("manifest".into()),
        vec!["input-confirmation".into()],
        "reproducibility",
    )
    .is_err());
}

#[test]
fn caller_attested_challenge_bundle_is_ineligible_for_confirmation() {
    let metrics = AttackMetrics {
        evidence_id: "report:evidence".into(),
        sample_size: 2,
        violations: 0,
        denominator: 2,
        source_ids: vec!["source-a".into(), "source-b".into()],
        control_identity: Some("control".into()),
        block_count: 2,
        family_count: 1,
        fit_end_ns: Some(1),
        evaluation_start_ns: Some(2),
        perturbation_count: 2,
        max_sample_share: 0.25,
        alternative_explanations: vec!["alternative".into()],
        metric: 1.0,
    };
    let evidence = vec![
        AttackEvidence::Lookahead {
            metrics: metrics.clone(),
        },
        AttackEvidence::PopulationConditioning {
            metrics: metrics.clone(),
        },
        AttackEvidence::DenominatorErrors {
            metrics: metrics.clone(),
        },
        AttackEvidence::SelectionBias {
            metrics: metrics.clone(),
        },
        AttackEvidence::LineageDuplication {
            metrics: metrics.clone(),
        },
        AttackEvidence::FalseConfluence {
            metrics: metrics.clone(),
        },
        AttackEvidence::BadControls {
            metrics: metrics.clone(),
        },
        AttackEvidence::TemporalDependence {
            metrics: metrics.clone(),
        },
        AttackEvidence::MultipleTesting {
            metrics: metrics.clone(),
        },
        AttackEvidence::NormalizationLeakage {
            metrics: metrics.clone(),
        },
        AttackEvidence::Fragility {
            metrics: metrics.clone(),
        },
        AttackEvidence::SampleConcentration {
            metrics: metrics.clone(),
        },
        AttackEvidence::AlternativeExplanations { metrics },
    ];
    let bundle = run_challenge_bundle(&ChallengeRunInput {
        target_identity: "report".into(),
        evidence,
    })
    .unwrap();
    assert!(!bundle.authority_eligible());
}

#[test]
fn deployment_rejects_known_test_vector_as_trust_root() {
    let root = unique_root("placeholder");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("trust.json");
    fs::write(
        &path,
        serde_json::json!({
            "version": 1,
            "policy_identity": "custodian-policy-v1",
            "public_key_hex": "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
            "replay_directory": "replay"
        })
        .to_string(),
    )
    .unwrap();
    assert!(matches!(
        ConfirmationRunner::open(path),
        Err(ConfirmationError::TrustRoot(_))
    ));
    let _ = fs::remove_dir_all(root);
}

fn sign(key: &SigningKey, binding: ConfirmationBinding, replay_identity: &str) -> Vec<u8> {
    let mut artifact = SignedCustodianArtifact::new(
        "custodian-policy-v1",
        hex(&key.verifying_key().to_bytes()),
        binding,
        "external-custodian-process",
        format!("nonce-{replay_identity}"),
        10_000,
        replay_identity,
    )
    .unwrap();
    artifact.set_signature_hex(hex(&key
        .sign(&artifact.signing_bytes().unwrap())
        .to_bytes()));
    serde_json::to_vec(&artifact).unwrap()
}

#[test]
fn trust_root_binding_and_replay_are_enforced_across_runner_instances() {
    let root = unique_root("durable");
    let trusted = ephemeral_key("trusted");
    let untrusted = ephemeral_key("untrusted");
    let trust_path = write_trust_root(&root, &trusted);
    let expected = binding();
    let artifact = sign(&trusted, expected.clone(), "replay-1");

    let first = ConfirmationRunner::open(&trust_path).unwrap();
    let locked = first.confirm(&expected, &artifact, 1).unwrap();
    assert!(locked.activation_locked());
    assert_eq!(locked.report_identity(), "walk-forward-report-sha256");
    drop(first);

    let second = ConfirmationRunner::open(&trust_path).unwrap();
    assert!(matches!(
        second.confirm(&expected, &artifact, 1),
        Err(ConfirmationError::Replay)
    ));

    let wrong_key = sign(&untrusted, expected.clone(), "replay-2");
    assert!(matches!(
        second.confirm(&expected, &wrong_key, 1),
        Err(ConfirmationError::InvalidSignature)
    ));

    let mut altered = expected.clone();
    altered = altered.with_report_identity("different-report").unwrap();
    let altered_artifact = sign(&trusted, altered, "replay-3");
    assert!(matches!(
        second.confirm(&expected, &altered_artifact, 1),
        Err(ConfirmationError::BindingMismatch)
    ));

    assert!(matches!(
        ConfirmationRunner::open(root.join("missing.json")),
        Err(ConfirmationError::TrustRoot(_))
    ));
    fs::remove_dir_all(root).unwrap();
}
