use crate::{ConfirmationKind, LockedConfirmation};
use research_contracts::{
    ContractError, CostModel, DecisionRule, ExecutionAssumption, RiskRule, StrategyHypothesis,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum StrategyArchetype {
    Continuation,
    ExhaustionReversal,
    StructuralReaction,
    Breakout,
    MeanReversion,
    MultiScaleStructural,
    RegimeFiltered,
    ExecutionAbstention,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfirmedBehavioralInput {
    input_id: String,
    available_time_ns: i64,
    confirmation_id: String,
    report_identity: String,
    authority_eligible: bool,
}

impl ConfirmedBehavioralInput {
    pub fn untrusted_for_description(
        input_id: impl Into<String>,
        available_time_ns: i64,
    ) -> Result<Self, StrategyError> {
        let input_id = input_id.into();
        if input_id.is_empty()
            || input_id.contains('/')
            || input_id.contains('\\')
            || available_time_ns < 0
        {
            return Err(StrategyError::Invalid("descriptive input identity"));
        }
        Ok(Self {
            input_id,
            available_time_ns,
            confirmation_id: "untrusted".into(),
            report_identity: "untrusted".into(),
            authority_eligible: false,
        })
    }

    pub fn from_confirmation(
        input_id: impl Into<String>,
        available_time_ns: i64,
        confirmation: &LockedConfirmation,
    ) -> Result<Self, StrategyError> {
        let input_id = input_id.into();
        if confirmation.contract().actor == research_contracts::ConfirmationActor::Worker
            || confirmation.contract().state != research_contracts::ConfirmationState::Confirmed
            || confirmation.contract().custodian_policy_identity.is_empty()
            || confirmation.contract().claim_identity != confirmation.target_identity()
            || confirmation.contract().confirmation_id.is_empty()
            || confirmation.kind() != ConfirmationKind::Detector
            || input_id.is_empty()
            || input_id.contains('/')
            || input_id.contains('\\')
            || available_time_ns < 0
        {
            return Err(StrategyError::Custody(
                "invalid detector confirmation input".into(),
            ));
        }
        Ok(Self {
            input_id,
            available_time_ns,
            confirmation_id: confirmation.confirmation_id().into(),
            report_identity: confirmation.report_identity().into(),
            authority_eligible: true,
        })
    }

    pub fn input_id(&self) -> &str {
        &self.input_id
    }
    pub fn available_time_ns(&self) -> i64 {
        self.available_time_ns
    }
    pub fn confirmation_id(&self) -> &str {
        &self.confirmation_id
    }
    pub fn report_identity(&self) -> &str {
        &self.report_identity
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfirmedInputManifest {
    strategy_id: String,
    holdout_policy_identity: String,
    inputs: Vec<ConfirmedBehavioralInput>,
}

impl ConfirmedInputManifest {
    pub fn untrusted_for_description(
        strategy_id: impl Into<String>,
        holdout_policy_identity: impl Into<String>,
        inputs: Vec<ConfirmedBehavioralInput>,
    ) -> Result<Self, StrategyError> {
        let manifest = Self {
            strategy_id: strategy_id.into(),
            holdout_policy_identity: holdout_policy_identity.into(),
            inputs,
        };
        manifest.validate_shape()?;
        Ok(manifest)
    }

    pub fn from_detector_confirmations(
        strategy_id: impl Into<String>,
        holdout_policy_identity: impl Into<String>,
        mut inputs: Vec<ConfirmedBehavioralInput>,
    ) -> Result<CustodiedInputManifest, StrategyError> {
        inputs.sort_by(|left, right| left.input_id.cmp(&right.input_id));
        if inputs.iter().any(|input| !input.authority_eligible) {
            return Err(StrategyError::UnconfirmedInput(
                "manifest contains descriptive input".into(),
            ));
        }
        let manifest = Self {
            strategy_id: strategy_id.into(),
            holdout_policy_identity: holdout_policy_identity.into(),
            inputs,
        };
        manifest.validate_shape()?;
        Ok(CustodiedInputManifest { manifest })
    }

    fn validate_shape(&self) -> Result<(), StrategyError> {
        if self.strategy_id.is_empty()
            || self.holdout_policy_identity.is_empty()
            || self.inputs.is_empty()
        {
            return Err(StrategyError::ConfirmedInputRequired);
        }
        let mut ids = BTreeSet::new();
        for input in &self.inputs {
            if input.input_id.is_empty()
                || !ids.insert(&input.input_id)
                || input.confirmation_id.is_empty()
                || input.report_identity.is_empty()
                || input.available_time_ns < 0
            {
                return Err(StrategyError::Invalid("confirmed input manifest"));
            }
        }
        Ok(())
    }

    pub fn strategy_id(&self) -> &str {
        &self.strategy_id
    }
    pub fn holdout_policy_identity(&self) -> &str {
        &self.holdout_policy_identity
    }
    pub fn inputs(&self) -> &[ConfirmedBehavioralInput] {
        &self.inputs
    }
    pub fn identity(&self) -> String {
        let mut hash = Sha256::new();
        hash.update(b"trinityr-confirmed-input-manifest-v2\0");
        hash.update(serde_json::to_vec(self).expect("manifest serializes"));
        format!("{:x}", hash.finalize())
    }

    fn validate_for(&self, hypothesis: &StrategyHypothesis) -> Result<(), StrategyError> {
        self.validate_shape()?;
        if self.strategy_id != hypothesis.strategy_id {
            return Err(StrategyError::IdentityMismatch("strategy_id"));
        }
        let ids = self
            .inputs
            .iter()
            .map(|input| input.input_id.as_str())
            .collect::<BTreeSet<_>>();
        if hypothesis
            .confirmed_finding_ids
            .iter()
            .any(|id| !ids.contains(id.as_str()))
        {
            return Err(StrategyError::ConfirmedInputRequired);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustodiedInputManifest {
    manifest: ConfirmedInputManifest,
}

impl CustodiedInputManifest {
    pub fn manifest(&self) -> &ConfirmedInputManifest {
        &self.manifest
    }
    pub fn identity(&self) -> String {
        self.manifest.identity()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StrategySpec {
    pub archetype: StrategyArchetype,
    pub hypothesis: StrategyHypothesis,
    pub decision_rule: DecisionRule,
    pub execution_assumption: ExecutionAssumption,
    pub cost_model: CostModel,
    pub risk_rule: RiskRule,
}

pub type StrategyDefinition = StrategySpec;

impl StrategySpec {
    /// Validate the reusable declarative strategy specification and its confirmed
    /// behavioral inputs. This intentionally does not hardcode a particular
    /// executable entry/exit/sizing/management grammar.
    pub fn validate(&self, manifest: &ConfirmedInputManifest) -> Result<(), StrategyError> {
        self.hypothesis.validate()?;
        self.decision_rule.validate()?;
        self.execution_assumption.validate()?;
        self.cost_model.validate()?;
        self.risk_rule.validate()?;
        if !self
            .execution_assumption
            .assumptions
            .contains_key("fill_model")
        {
            return Err(StrategyError::Invalid("execution fill model"));
        }
        manifest.validate_for(&self.hypothesis)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrategyError {
    Invalid(&'static str),
    Contract(String),
    IdentityMismatch(&'static str),
    ConfirmedInputRequired,
    UnconfirmedInput(String),
    FutureInput {
        timestamp_ns: i64,
        available_time_ns: i64,
    },
    EmptySample,
    NonFinite(&'static str),
    Holdout(String),
    Custody(String),
}

impl std::fmt::Display for StrategyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(field) => write!(f, "invalid {field}"),
            Self::Contract(message) | Self::Holdout(message) | Self::Custody(message) => {
                f.write_str(message)
            }
            Self::IdentityMismatch(field) => write!(f, "{field} identity mismatch"),
            Self::ConfirmedInputRequired => f.write_str("confirmed behavioral input required"),
            Self::UnconfirmedInput(id) => write!(f, "input is not confirmed: {id}"),
            Self::FutureInput {
                timestamp_ns,
                available_time_ns,
            } => write!(
                f,
                "input available at {available_time_ns} after timestamp {timestamp_ns}"
            ),
            Self::EmptySample => f.write_str("sample is empty"),
            Self::NonFinite(field) => write!(f, "non-finite {field}"),
        }
    }
}
impl std::error::Error for StrategyError {}
impl From<ContractError> for StrategyError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error.to_string())
    }
}
pub(crate) fn finite(value: f64, field: &'static str) -> Result<(), StrategyError> {
    value
        .is_finite()
        .then_some(())
        .ok_or(StrategyError::NonFinite(field))
}
