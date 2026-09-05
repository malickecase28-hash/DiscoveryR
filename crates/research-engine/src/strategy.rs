use research_contracts::{
    ContractError, CostModel, DecisionRule, ExecutionAssumption, RiskRule, StrategyHypothesis,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmedBehavioralInput {
    pub input_id: String,
    pub confirmed: bool,
    pub available_time_ns: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmedInputManifest {
    pub strategy_id: String,
    pub holdout_policy_identity: String,
    pub inputs: Vec<ConfirmedBehavioralInput>,
}

impl ConfirmedInputManifest {
    pub fn validate_for(&self, hypothesis: &StrategyHypothesis) -> Result<(), StrategyError> {
        if self.strategy_id != hypothesis.strategy_id {
            return Err(StrategyError::IdentityMismatch("strategy_id"));
        }
        if self.holdout_policy_identity.is_empty() {
            return Err(StrategyError::Invalid("holdout policy identity"));
        }
        if self.inputs.is_empty() {
            return Err(StrategyError::ConfirmedInputRequired);
        }
        let mut seen = std::collections::BTreeSet::new();
        for input in &self.inputs {
            if input.input_id.is_empty()
                || input.input_id.contains('/')
                || input.input_id.contains('\\')
            {
                return Err(StrategyError::Invalid("input identity"));
            }
            if !seen.insert(&input.input_id) {
                return Err(StrategyError::Invalid("duplicate input identity"));
            }
            if !input.confirmed {
                return Err(StrategyError::UnconfirmedInput(input.input_id.clone()));
            }
            if input.available_time_ns < 0 {
                return Err(StrategyError::Invalid("input availability"));
            }
        }
        for id in &hypothesis.confirmed_finding_ids {
            if !seen.contains(id) {
                return Err(StrategyError::ConfirmedInputRequired);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategySpec {
    pub hypothesis: StrategyHypothesis,
    pub decision_rule: DecisionRule,
    pub execution_assumption: ExecutionAssumption,
    pub cost_model: CostModel,
    pub risk_rule: RiskRule,
}

pub type StrategyDefinition = StrategySpec;

impl StrategySpec {
    pub fn validate(&self, manifest: &ConfirmedInputManifest) -> Result<(), StrategyError> {
        self.hypothesis.validate()?;
        self.decision_rule.validate()?;
        self.execution_assumption.validate()?;
        self.cost_model.validate()?;
        self.risk_rule.validate()?;
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
            Self::Contract(message) => f.write_str(message),
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
            Self::Holdout(message) => f.write_str(message),
            Self::Custody(message) => f.write_str(message),
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
