use ed25519_dalek::{Signer, SigningKey};
use research_contracts::{ConfirmationActor, ConfirmationContract, ConfirmationState};
use research_engine::{
    ConfirmationBinding, ConfirmationKind, ConfirmationRunner, LockedConfirmation,
    SignedCustodianArtifact,
};
use sha2::{Digest, Sha256};
use std::{fs, time::SystemTime};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn key(label: &str) -> SigningKey {
    let mut hash = Sha256::new();
    hash.update(label.as_bytes());
    hash.update(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes(),
    );
    SigningKey::from_bytes(&hash.finalize().into())
}

#[allow(dead_code)]
pub fn detector_lock(input_id: &str, holdout: &str) -> LockedConfirmation {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rsp-test-authority-{input_id}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let signing = key(input_id);
    let contract = ConfirmationContract {
        confirmation_id: format!("detector-confirmation-{input_id}"),
        claim_identity: input_id.into(),
        population_identity: "population".into(),
        anchor_identity: "anchor".into(),
        context_identity: "context".into(),
        outcome_identity: "outcome".into(),
        metric_identity: "metric".into(),
        code_identity: "code".into(),
        control_design_identity: "controls".into(),
        null_design_identity: "null".into(),
        multiplicity_family_identity: "family".into(),
        data_policy_identity: "data".into(),
        holdout_policy_identity: holdout.into(),
        custodian_policy_identity: "custodian-policy".into(),
        actor: ConfirmationActor::Custodian,
        state: ConfirmationState::Confirmed,
    };
    let binding = ConfirmationBinding::new(
        ConfirmationKind::Detector,
        contract,
        input_id,
        format!("report-{input_id}"),
        format!("challenge-{input_id}"),
        vec![format!("scope-{input_id}")],
        None,
        vec![],
        format!("repro-{input_id}"),
    )
    .unwrap();
    let trust_path = root.join("trust.json");
    fs::write(&trust_path, serde_json::to_vec(&serde_json::json!({"version":1,"policy_identity":"custodian-policy","public_key_hex":hex(&signing.verifying_key().to_bytes()),"replay_directory":"replay"})).unwrap()).unwrap();
    let mut artifact = SignedCustodianArtifact::new(
        "custodian-policy",
        hex(&signing.verifying_key().to_bytes()),
        binding.clone(),
        "test-custodian",
        format!("nonce-{input_id}"),
        i64::MAX,
        format!("replay-{input_id}"),
    )
    .unwrap();
    artifact.set_signature_hex(hex(&signing
        .sign(&artifact.signing_bytes().unwrap())
        .to_bytes()));
    let bytes = serde_json::to_vec(&artifact).unwrap();
    let lock = ConfirmationRunner::open(&trust_path)
        .unwrap()
        .confirm(&binding, &bytes, 1)
        .unwrap();
    let _ = fs::remove_dir_all(&root);
    lock
}

#[allow(dead_code)]
pub fn strategy_lock(
    strategy_id: &str,
    report_identity: &str,
    holdout: &str,
) -> LockedConfirmation {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rsp-test-strategy-{strategy_id}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let signing = key(strategy_id);
    let contract = ConfirmationContract {
        confirmation_id: strategy_id.into(),
        claim_identity: strategy_id.into(),
        population_identity: "population".into(),
        anchor_identity: "anchor".into(),
        context_identity: "context".into(),
        outcome_identity: "outcome".into(),
        metric_identity: "metric".into(),
        code_identity: "code".into(),
        control_design_identity: "controls".into(),
        null_design_identity: "null".into(),
        multiplicity_family_identity: "family".into(),
        data_policy_identity: "data".into(),
        holdout_policy_identity: holdout.into(),
        custodian_policy_identity: "custodian-policy".into(),
        actor: ConfirmationActor::Custodian,
        state: ConfirmationState::Confirmed,
    };
    let binding = ConfirmationBinding::new(
        ConfirmationKind::Strategy,
        contract,
        strategy_id,
        report_identity,
        format!("challenge-{strategy_id}"),
        vec![format!("scope-{strategy_id}")],
        Some(format!("manifest-{strategy_id}")),
        vec![format!("input-{strategy_id}")],
        format!("repro-{strategy_id}"),
    )
    .unwrap();
    let trust_path = root.join("trust.json");
    fs::write(&trust_path, serde_json::to_vec(&serde_json::json!({"version":1,"policy_identity":"custodian-policy","public_key_hex":hex(&signing.verifying_key().to_bytes()),"replay_directory":"replay"})).unwrap()).unwrap();
    let mut artifact = SignedCustodianArtifact::new(
        "custodian-policy",
        hex(&signing.verifying_key().to_bytes()),
        binding.clone(),
        "test-custodian",
        format!("nonce-{strategy_id}"),
        i64::MAX,
        format!("replay-{strategy_id}"),
    )
    .unwrap();
    artifact.set_signature_hex(hex(&signing
        .sign(&artifact.signing_bytes().unwrap())
        .to_bytes()));
    let lock = ConfirmationRunner::open(&trust_path)
        .unwrap()
        .confirm(&binding, &serde_json::to_vec(&artifact).unwrap(), 1)
        .unwrap();
    let _ = fs::remove_dir_all(root);
    lock
}
