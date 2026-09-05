use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{evidence::EvidenceState, identity::validate_identity, ContractError};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct UserParameter {
    pub parameter_id: String,
    pub value: Value,
}

impl UserParameter {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.parameter_id, "parameter_id")
    }
}

fn validate_parameters(parameters: &BTreeMap<String, Value>) -> Result<(), ContractError> {
    if parameters.is_empty() {
        return Err(ContractError::Invalid(
            "explicit user parameters are required".into(),
        ));
    }
    for key in parameters.keys() {
        validate_identity(key, "parameter_id")?;
    }
    Ok(())
}

fn validate_refs(refs: &[String], field: &str) -> Result<(), ContractError> {
    if refs.is_empty() {
        return Err(ContractError::Invalid(format!("{field} cannot be empty")));
    }
    for id in refs {
        validate_identity(id, field)?;
    }
    if refs.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContractError::Invalid(format!(
            "{field} must be canonical and unique"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct StrategyHypothesis {
    pub strategy_id: String,
    pub hypothesis_id: String,
    pub confirmed_finding_ids: Vec<String>,
    pub parameters: BTreeMap<String, Value>,
}

impl StrategyHypothesis {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.strategy_id, "strategy_id")?;
        validate_identity(&self.hypothesis_id, "hypothesis_id")?;
        validate_refs(&self.confirmed_finding_ids, "confirmed_finding_ids")?;
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct DecisionRule {
    pub rule_id: String,
    pub entry: String,
    pub exit: String,
    pub sizing: String,
    pub management: String,
    pub parameters: BTreeMap<String, Value>,
}

impl DecisionRule {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.rule_id, "rule_id")?;
        if [
            self.entry.as_str(),
            self.exit.as_str(),
            self.sizing.as_str(),
            self.management.as_str(),
        ]
        .iter()
        .any(|value| value.is_empty())
        {
            return Err(ContractError::Invalid(
                "decision rule clauses cannot be empty".into(),
            ));
        }
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExecutionAssumption {
    pub assumption_id: String,
    pub assumptions: BTreeMap<String, Value>,
}

impl ExecutionAssumption {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.assumption_id, "assumption_id")?;
        validate_parameters(&self.assumptions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct CostModel {
    pub cost_model_id: String,
    pub parameters: BTreeMap<String, Value>,
}

impl CostModel {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.cost_model_id, "cost_model_id")?;
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct RiskRule {
    pub risk_rule_id: String,
    pub parameters: BTreeMap<String, Value>,
}

impl RiskRule {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.risk_rule_id, "risk_rule_id")?;
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct StrategyValidation {
    pub validation_id: String,
    pub strategy_id: String,
    pub confirmed_input_ids: Vec<String>,
    pub code_identity: String,
    pub data_policy_identity: String,
    pub state: EvidenceState,
    pub parameters: BTreeMap<String, Value>,
}

impl StrategyValidation {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("validation_id", &self.validation_id),
            ("strategy_id", &self.strategy_id),
            ("code_identity", &self.code_identity),
            ("data_policy_identity", &self.data_policy_identity),
        ] {
            validate_identity(value, field)?;
        }
        validate_refs(&self.confirmed_input_ids, "confirmed_input_ids")?;
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct PortfolioComponent {
    pub component_id: String,
    pub confirmed_strategy_ids: Vec<String>,
    pub parameters: BTreeMap<String, Value>,
}

impl PortfolioComponent {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.component_id, "component_id")?;
        validate_refs(&self.confirmed_strategy_ids, "confirmed_strategy_ids")?;
        validate_parameters(&self.parameters)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortfolioConstraintKind {
    Correlation,
    Overlap,
    CapitalAllocation,
    Drawdown,
    Capacity,
    Turnover,
    Liquidity,
    Concentration,
    RiskBudget,
    Stress,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct PortfolioConstraint {
    pub constraint_id: String,
    pub kind: PortfolioConstraintKind,
    pub confirmed_strategy_ids: Vec<String>,
    pub parameters: BTreeMap<String, Value>,
}

impl PortfolioConstraint {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.constraint_id, "constraint_id")?;
        validate_refs(&self.confirmed_strategy_ids, "confirmed_strategy_ids")?;
        validate_parameters(&self.parameters)
    }
}
