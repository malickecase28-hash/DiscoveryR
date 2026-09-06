use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

use research_contracts::{
    AttackKind, ChallengeResult, ConfirmationContract, ConfirmationState, EvidenceState,
    MethodChallenge,
};

use crate::{ConfirmedInputManifest, StrategyError, StrategyObservation, WalkForwardPlan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationRequest {
    pub contract: ConfirmationContract,
    pub validation: research_contracts::StrategyValidation,
    pub manifest: ConfirmedInputManifest,
    pub plan: WalkForwardPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustodianReceipt {
    pub authorization_id: String,
    pub policy_identity: String,
    pub authenticated: bool,
}

pub trait ExternalCustodian {
    fn authorize(
        &self,
        request: &ConfirmationRequest,
    ) -> Result<CustodianReceipt, ConfirmationError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategyConfirmation {
    pub confirmation_id: String,
    pub holdout_policy_identity: String,
    pub externally_authorized: bool,
    pub activation_locked: bool,
}

/// Custodian receipt for the strict confirmation boundary. The receipt is
/// checked against the exact request and challenge identities before a locked
/// result is returned. `signature` is an externally supplied proof value; this
/// crate does not pretend to provide key custody.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundCustodianReceipt {
    pub authorization_id: String,
    pub custodian_policy_identity: String,
    pub request_identity: String,
    pub challenge_identity: String,
    pub actor_identity: String,
    pub nonce: String,
    pub expires_at_ns: i64,
    pub replay_identity: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedConfirmation {
    pub confirmation_id: String,
    pub request_identity: String,
    pub challenge_identity: String,
    pub holdout_policy_identity: String,
    pub activation_locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectorConfirmationRequest {
    pub contract: ConfirmationContract,
    pub target_identity: String,
    pub evidence_state: EvidenceState,
    pub holdout_policy_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectorConfirmation {
    pub confirmation_id: String,
    pub target_identity: String,
    pub challenge_identity: String,
    pub activation_locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortfolioConfirmationRequest {
    pub contract: ConfirmationContract,
    pub component_identity: String,
    pub strategy_confirmation_ids: Vec<String>,
    pub holdout_policy_identity: String,
    pub evidence_state: EvidenceState,
}

pub fn confirm_portfolio(
    request: &PortfolioConfirmationRequest,
    challenges: &[MethodChallenge],
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
    if request.component_identity.is_empty()
        || request.strategy_confirmation_ids.is_empty()
        || request
            .strategy_confirmation_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || request.holdout_policy_identity != request.contract.holdout_policy_identity
        || request.evidence_state != EvidenceState::Known
        || challenges.len() != 13
        || challenges.iter().any(|challenge| {
            challenge.result != ChallengeResult::Passed || challenge.evidence_ids.is_empty()
        })
    {
        return Err(ConfirmationError::Invalid("portfolio confirmation inputs"));
    }
    let challenge_identity =
        digest_json(challenges).map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let request_identity = digest_json(&(
        &request.contract,
        &request.component_identity,
        &request.strategy_confirmation_ids,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    if receipt.request_identity != request_identity
        || receipt.challenge_identity != challenge_identity
        || receipt.custodian_policy_identity != request.contract.custodian_policy_identity
        || receipt.expires_at_ns <= now_ns
        || receipt.signature != receipt_signature(receipt, &request_identity, &challenge_identity)
    {
        return Err(ConfirmationError::Custodian(
            "unbound portfolio receipt".into(),
        ));
    }
    Ok(LockedConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        request_identity,
        challenge_identity,
        holdout_policy_identity: request.holdout_policy_identity.clone(),
        activation_locked: true,
    })
}

pub fn confirm_detector(
    request: &DetectorConfirmationRequest,
    challenges: &[MethodChallenge],
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<DetectorConfirmation, ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
    if request.target_identity.is_empty()
        || request.holdout_policy_identity != request.contract.holdout_policy_identity
        || request.evidence_state != EvidenceState::Known
        || challenges.len() != 13
        || challenges.iter().any(|challenge| {
            challenge.target_identity != request.target_identity
                || challenge.result != ChallengeResult::Passed
                || challenge.evidence_ids.is_empty()
        })
    {
        return Err(ConfirmationError::Invalid("detector confirmation inputs"));
    }
    let challenge_identity =
        digest_json(challenges).map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let request_identity = digest_json(&(
        &request.contract,
        &request.target_identity,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    if receipt.request_identity != request_identity
        || receipt.challenge_identity != challenge_identity
        || receipt.custodian_policy_identity != request.contract.custodian_policy_identity
        || receipt.expires_at_ns <= now_ns
        || receipt.signature != receipt_signature(receipt, &request_identity, &challenge_identity)
    {
        return Err(ConfirmationError::Custodian(
            "unbound detector receipt".into(),
        ));
    }
    Ok(DetectorConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        target_identity: request.target_identity.clone(),
        challenge_identity,
        activation_locked: true,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmationError {
    Invalid(&'static str),
    Contract(String),
    Custodian(String),
    WorkerCannotAuthorize,
    UnconfirmedInput,
}

impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(field) => write!(f, "invalid {field}"),
            Self::Contract(message) | Self::Custodian(message) => f.write_str(message),
            Self::WorkerCannotAuthorize => f.write_str("worker cannot authorize confirmation"),
            Self::UnconfirmedInput => f.write_str("confirmation requires confirmed inputs"),
        }
    }
}

impl std::error::Error for ConfirmationError {}

pub fn confirm_strategy<C: ExternalCustodian>(
    request: ConfirmationRequest,
    custodian: &C,
) -> Result<StrategyConfirmation, ConfirmationError> {
    validate_request(&request)?;
    let receipt = custodian.authorize(&request)?;
    if !receipt.authenticated
        || receipt.authorization_id.is_empty()
        || receipt.policy_identity != request.contract.custodian_policy_identity
        || receipt.policy_identity.contains('/')
        || receipt.policy_identity.contains('\\')
    {
        return Err(ConfirmationError::Custodian(
            "external custodian receipt is invalid".into(),
        ));
    }
    Ok(StrategyConfirmation {
        confirmation_id: request.contract.confirmation_id,
        holdout_policy_identity: request.plan.policy_identity,
        externally_authorized: true,
        activation_locked: true,
    })
}

/// Strict confirmation entry point. A receipt must be bound to the complete
/// request, passed challenge set, policy, nonce, and expiry. Replay identities
/// are supplied by the custodian boundary and must be unique to the request.
pub fn confirm_strategy_bound(
    request: &ConfirmationRequest,
    challenges: &[MethodChallenge],
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    validate_request(request)?;
    if challenges.len() != 13
        || challenges.iter().any(|challenge| {
            challenge.result != ChallengeResult::Passed
                || challenge.target_identity != request.validation.validation_id
                || challenge.evidence_ids.is_empty()
        })
    {
        return Err(ConfirmationError::Invalid(
            "complete passed method challenge required",
        ));
    }
    let challenge_identity =
        digest_json(challenges).map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let request_identity = digest_json(&(
        &request.contract,
        &request.validation,
        &request.manifest,
        &request.plan.policy,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    if receipt.authorization_id.is_empty()
        || receipt.custodian_policy_identity != request.contract.custodian_policy_identity
        || receipt.request_identity != request_identity
        || receipt.challenge_identity != challenge_identity
        || receipt.actor_identity.is_empty()
        || receipt.nonce.is_empty()
        || receipt.replay_identity.is_empty()
        || receipt.expires_at_ns <= now_ns
        || receipt.signature != receipt_signature(receipt, &request_identity, &challenge_identity)
    {
        return Err(ConfirmationError::Custodian(
            "unbound or expired custodian receipt".into(),
        ));
    }
    Ok(LockedConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        request_identity,
        challenge_identity,
        holdout_policy_identity: request.plan.policy_identity.clone(),
        activation_locked: true,
    })
}

pub fn confirm_strategy_report(
    request: &ConfirmationRequest,
    report: &crate::WalkForwardReport,
    challenges: &[MethodChallenge],
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    if request.validation != report.validation {
        return Err(ConfirmationError::Invalid(
            "validation must come from completed run",
        ));
    }
    confirm_strategy_bound(request, challenges, receipt, now_ns)
}

pub fn seal_external_receipt(
    mut receipt: BoundCustodianReceipt,
    request: &ConfirmationRequest,
    challenges: &[MethodChallenge],
) -> Result<BoundCustodianReceipt, ConfirmationError> {
    receipt.request_identity = digest_json(&(
        &request.contract,
        &request.validation,
        &request.manifest,
        &request.plan.policy,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    receipt.challenge_identity =
        digest_json(challenges).map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    receipt.signature = receipt_signature(
        &receipt,
        &receipt.request_identity,
        &receipt.challenge_identity,
    );
    Ok(receipt)
}

fn digest_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let mut hash = Sha256::new();
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

fn receipt_signature(
    receipt: &BoundCustodianReceipt,
    request_identity: &str,
    challenge_identity: &str,
) -> String {
    let material = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        receipt.authorization_id,
        receipt.custodian_policy_identity,
        request_identity,
        challenge_identity,
        receipt.actor_identity,
        receipt.nonce,
        receipt.expires_at_ns
    );
    digest_json(&material).unwrap_or_default()
}

pub fn method_challenge(
    target_identity: &str,
    plan: &WalkForwardPlan,
    observations: &[StrategyObservation],
) -> Result<MethodChallenge, StrategyError> {
    plan.validate()?;
    if observations.is_empty() {
        return Err(StrategyError::EmptySample);
    }
    let mut passed = true;
    for observation in observations {
        if observation.timestamp_ns < 0
            || observation.available_time_ns < 0
            || observation.available_time_ns > observation.timestamp_ns
            || !in_plan(plan, observation.timestamp_ns)
        {
            passed = false;
            break;
        }
    }
    let challenge = MethodChallenge {
        challenge_id: format!("challenge-{target_identity}"),
        target_identity: target_identity.into(),
        attack_kind: AttackKind::TimestampLeakage,
        protocol_identity: "causal-partition-check-v1".into(),
        result: if passed {
            ChallengeResult::Passed
        } else {
            ChallengeResult::Failed
        },
        evidence_ids: vec![target_identity.into()],
    };
    challenge
        .validate()
        .map_err(|error| StrategyError::Custody(error.to_string()))?;
    Ok(challenge)
}

/// Executes the declared challenge surface for a synthetic, already frozen
/// target. Each attack remains separately addressable; a surviving challenge
/// is evidence only and never grants confirmation authority.
pub fn method_challenge_protocol(
    target_identity: &str,
    plan: &WalkForwardPlan,
    observations: &[StrategyObservation],
) -> Result<Vec<MethodChallenge>, StrategyError> {
    plan.validate()?;
    if observations.is_empty() {
        return Err(StrategyError::EmptySample);
    }
    let timestamp_ok = observations.iter().all(|observation| {
        observation.timestamp_ns >= 0
            && observation.available_time_ns >= 0
            && observation.available_time_ns <= observation.timestamp_ns
            && in_plan(plan, observation.timestamp_ns)
    });
    const ATTACKS: [AttackKind; 13] = [
        AttackKind::Lookahead,
        AttackKind::PopulationConditioning,
        AttackKind::DenominatorErrors,
        AttackKind::SelectionBias,
        AttackKind::LineageDuplication,
        AttackKind::FalseConfluence,
        AttackKind::BadControls,
        AttackKind::TemporalDependence,
        AttackKind::MultipleTesting,
        AttackKind::NormalizationLeakage,
        AttackKind::Fragility,
        AttackKind::SampleConcentration,
        AttackKind::AlternativeExplanations,
    ];
    ATTACKS
        .into_iter()
        .map(|attack_kind| {
            let challenge = MethodChallenge {
                challenge_id: format!(
                    "challenge-{target_identity}-{}",
                    attack_kind_name(attack_kind)
                ),
                target_identity: target_identity.into(),
                attack_kind,
                protocol_identity: "rsp-method-challenge-v1".into(),
                result: if attack_kind == AttackKind::Lookahead && timestamp_ok {
                    ChallengeResult::Passed
                } else if attack_kind == AttackKind::Lookahead {
                    ChallengeResult::Failed
                } else {
                    ChallengeResult::Inconclusive
                },
                evidence_ids: vec![target_identity.into()],
            };
            challenge
                .validate()
                .map_err(|error| StrategyError::Custody(error.to_string()))?;
            Ok(challenge)
        })
        .collect()
}

fn attack_kind_name(kind: AttackKind) -> &'static str {
    match kind {
        AttackKind::Lookahead => "lookahead",
        AttackKind::PopulationConditioning => "population_conditioning",
        AttackKind::DenominatorErrors => "denominator_errors",
        AttackKind::SelectionBias => "selection_bias",
        AttackKind::LineageDuplication => "lineage_duplication",
        AttackKind::FalseConfluence => "false_confluence",
        AttackKind::BadControls => "bad_controls",
        AttackKind::TemporalDependence => "temporal_dependence",
        AttackKind::MultipleTesting => "multiple_testing",
        AttackKind::NormalizationLeakage => "normalization_leakage",
        AttackKind::Fragility => "fragility",
        AttackKind::SampleConcentration => "sample_concentration",
        AttackKind::AlternativeExplanations => "alternative_explanations",
        AttackKind::TimestampLeakage => "timestamp_leakage",
        AttackKind::DataSnooping => "data_snooping",
        AttackKind::Confounding => "confounding",
        AttackKind::ReverseCausality => "reverse_causality",
        AttackKind::MeasurementError => "measurement_error",
        AttackKind::Missingness => "missingness",
        AttackKind::SurvivorshipBias => "survivorship_bias",
        AttackKind::NonStationarity => "non_stationarity",
        AttackKind::SpecificationSensitivity => "specification_sensitivity",
        AttackKind::Placebo => "placebo",
        AttackKind::Reproducibility => "reproducibility",
    }
}

fn validate_request(request: &ConfirmationRequest) -> Result<(), ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
    if request.contract.actor == research_contracts::ConfirmationActor::Worker {
        return Err(ConfirmationError::WorkerCannotAuthorize);
    }
    if request.contract.state != ConfirmationState::Confirmed {
        return Err(ConfirmationError::Invalid("confirmation state"));
    }
    request
        .validation
        .validate()
        .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
    request
        .plan
        .validate()
        .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
    if request.validation.state != EvidenceState::Known {
        return Err(ConfirmationError::Invalid("validation evidence state"));
    }
    if request.validation.strategy_id != request.manifest.strategy_id
        || request.contract.holdout_policy_identity != request.plan.policy_identity
        || request.manifest.holdout_policy_identity != request.plan.policy_identity
    {
        return Err(ConfirmationError::Invalid("confirmation identity"));
    }
    let manifest_ids: BTreeSet<&str> = request
        .manifest
        .inputs
        .iter()
        .map(|input| input.input_id.as_str())
        .collect();
    let validation_ids: BTreeSet<&str> = request
        .validation
        .confirmed_input_ids
        .iter()
        .map(String::as_str)
        .collect();
    if manifest_ids.is_empty()
        || manifest_ids != validation_ids
        || request.manifest.inputs.iter().any(|input| !input.confirmed)
    {
        return Err(ConfirmationError::UnconfirmedInput);
    }
    Ok(())
}

fn in_plan(plan: &WalkForwardPlan, timestamp_ns: i64) -> bool {
    contains(&plan.development_scope, timestamp_ns)
        || contains(&plan.strategy_holdout_scope, timestamp_ns)
        || plan
            .future_scope
            .as_ref()
            .is_some_and(|scope| contains(scope, timestamp_ns))
}

fn contains(scope: &research_contracts::ScopeInterval, timestamp_ns: i64) -> bool {
    scope.start_time_ns <= timestamp_ns && timestamp_ns < scope.end_time_ns
}
