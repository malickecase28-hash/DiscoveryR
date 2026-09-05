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

pub fn method_challenge(
    target_identity: &str,
    plan: &WalkForwardPlan,
    observations: &[StrategyObservation],
) -> Result<MethodChallenge, StrategyError> {
    plan.validate()?;
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
