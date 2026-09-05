use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

use crate::{
    identity::validate_identity, ContractError, DetectorRole, KnowledgeRecord, NativeScale,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceState {
    Known,
    Unknown,
    Null,
    Rejected,
    Inconclusive,
    Partial,
    Contradictory,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct OutcomeDefinition {
    pub outcome_id: String,
    pub definition: Value,
    pub anchor_id: String,
    pub availability_rule_id: String,
}

impl OutcomeDefinition {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("outcome_id", &self.outcome_id),
            ("anchor_id", &self.anchor_id),
            ("availability_rule_id", &self.availability_rule_id),
        ] {
            validate_identity(value, field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct Question {
    pub question_id: String,
    pub experiment_id: String,
    pub instrument_ids: Vec<String>,
    pub detector_ids: Vec<String>,
    pub roles: Vec<DetectorRole>,
    pub anchor_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub context_ids: Vec<String>,
    pub native_scales: BTreeSet<NativeScale>,
    pub evidence_state: EvidenceState,
}

impl Question {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.question_id, "question_id")?;
        validate_identity(&self.experiment_id, "experiment_id")?;
        for (field, values) in [
            ("instrument_ids", &self.instrument_ids),
            ("detector_ids", &self.detector_ids),
            ("anchor_ids", &self.anchor_ids),
            ("context_ids", &self.context_ids),
        ] {
            let ids: HashSet<&str> = values.iter().map(String::as_str).collect();
            if ids.len() != values.len() || values.iter().any(String::is_empty) {
                return Err(ContractError::Invalid(format!(
                    "{field} must contain unique nonempty IDs"
                )));
            }
        }
        if self.native_scales.is_empty() {
            return Err(ContractError::Invalid(
                "question scales cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct Hypothesis {
    pub hypothesis_id: String,
    pub question_id: String,
    pub experiment_id: String,
    pub claim: String,
    pub referenced_input_ids: Vec<String>,
}

impl Hypothesis {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("hypothesis_id", &self.hypothesis_id),
            ("question_id", &self.question_id),
            ("experiment_id", &self.experiment_id),
        ] {
            validate_identity(value, field)?;
        }
        if self.claim.is_empty() {
            return Err(ContractError::Invalid(
                "hypothesis claim cannot be empty".into(),
            ));
        }
        validate_ids(&self.referenced_input_ids, "referenced_input_ids")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ControlDesign {
    pub design_id: String,
    pub method: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct NullDesign {
    pub design_id: String,
    pub method: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttackKind {
    TimestampLeakage,
    SelectionBias,
    MultipleTesting,
    DataSnooping,
    Confounding,
    ReverseCausality,
    MeasurementError,
    Missingness,
    SurvivorshipBias,
    NonStationarity,
    SpecificationSensitivity,
    Placebo,
    Reproducibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeResult {
    Open,
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct MethodChallenge {
    pub challenge_id: String,
    pub target_identity: String,
    pub attack_kind: AttackKind,
    pub protocol_identity: String,
    pub result: ChallengeResult,
    pub evidence_ids: Vec<String>,
}

impl MethodChallenge {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("challenge_id", &self.challenge_id),
            ("target_identity", &self.target_identity),
            ("protocol_identity", &self.protocol_identity),
        ] {
            validate_identity(value, field)?;
        }
        validate_ids(&self.evidence_ids, "evidence_ids")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfirmationActor {
    Worker,
    Custodian,
    Runner,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfirmationState {
    Unknown,
    Null,
    Rejected,
    Inconclusive,
    Confirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ConfirmationContract {
    pub confirmation_id: String,
    pub claim_identity: String,
    pub population_identity: String,
    pub anchor_identity: String,
    pub context_identity: String,
    pub outcome_identity: String,
    pub metric_identity: String,
    pub code_identity: String,
    pub control_design_identity: String,
    pub null_design_identity: String,
    pub multiplicity_family_identity: String,
    pub data_policy_identity: String,
    pub holdout_policy_identity: String,
    pub custodian_policy_identity: String,
    pub actor: ConfirmationActor,
    pub state: ConfirmationState,
}

impl ConfirmationContract {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("confirmation_id", &self.confirmation_id),
            ("claim_identity", &self.claim_identity),
            ("population_identity", &self.population_identity),
            ("anchor_identity", &self.anchor_identity),
            ("context_identity", &self.context_identity),
            ("outcome_identity", &self.outcome_identity),
            ("metric_identity", &self.metric_identity),
            ("code_identity", &self.code_identity),
            ("control_design_identity", &self.control_design_identity),
            ("null_design_identity", &self.null_design_identity),
            (
                "multiplicity_family_identity",
                &self.multiplicity_family_identity,
            ),
            ("data_policy_identity", &self.data_policy_identity),
            ("holdout_policy_identity", &self.holdout_policy_identity),
            ("custodian_policy_identity", &self.custodian_policy_identity),
        ] {
            validate_identity(value, field)?;
        }
        if self.actor == ConfirmationActor::Worker {
            return Err(ContractError::Invalid(
                "worker cannot authorize confirmation unlock".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct KnowledgeFacets {
    pub instrument_ids: Vec<String>,
    pub detector_ids: Vec<String>,
    pub anchor_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub context_ids: Vec<String>,
    pub native_scales: BTreeSet<NativeScale>,
    pub evidence_ids: Vec<String>,
    pub strategy_ids: Vec<String>,
    pub portfolio_ids: Vec<String>,
}

impl KnowledgeFacets {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, values) in [
            ("instrument_ids", &self.instrument_ids),
            ("detector_ids", &self.detector_ids),
            ("anchor_ids", &self.anchor_ids),
            ("lifecycle_states", &self.lifecycle_states),
            ("context_ids", &self.context_ids),
            ("evidence_ids", &self.evidence_ids),
            ("strategy_ids", &self.strategy_ids),
            ("portfolio_ids", &self.portfolio_ids),
        ] {
            validate_ids(values, field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct KnowledgeEnvelope {
    pub envelope_version: u16,
    pub knowledge_id: String,
    pub record: KnowledgeRecord,
    pub facets: KnowledgeFacets,
}

impl KnowledgeEnvelope {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.envelope_version == 0 {
            return Err(ContractError::Invalid(
                "knowledge envelope version cannot be zero".into(),
            ));
        }
        validate_identity(&self.knowledge_id, "knowledge_id")?;
        self.facets.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct KnowledgeQuery {
    pub instrument_id: Option<String>,
    pub detector_id: Option<String>,
    pub anchor_id: Option<String>,
    pub context_id: Option<String>,
    pub native_scale: Option<NativeScale>,
    pub evidence_id: Option<String>,
    pub strategy_id: Option<String>,
    pub portfolio_id: Option<String>,
}

impl KnowledgeQuery {
    pub fn matches(&self, envelope: &KnowledgeEnvelope) -> bool {
        let f = &envelope.facets;
        self.instrument_id
            .as_ref()
            .is_none_or(|id| f.instrument_ids.contains(id))
            && self
                .detector_id
                .as_ref()
                .is_none_or(|id| f.detector_ids.contains(id))
            && self
                .anchor_id
                .as_ref()
                .is_none_or(|id| f.anchor_ids.contains(id))
            && self
                .context_id
                .as_ref()
                .is_none_or(|id| f.context_ids.contains(id))
            && self
                .native_scale
                .as_ref()
                .is_none_or(|scale| f.native_scales.contains(scale))
            && self
                .evidence_id
                .as_ref()
                .is_none_or(|id| f.evidence_ids.contains(id))
            && self
                .strategy_id
                .as_ref()
                .is_none_or(|id| f.strategy_ids.contains(id))
            && self
                .portfolio_id
                .as_ref()
                .is_none_or(|id| f.portfolio_ids.contains(id))
    }
}

fn validate_ids(values: &[String], field: &str) -> Result<(), ContractError> {
    let ids: HashSet<&str> = values.iter().map(String::as_str).collect();
    if ids.len() != values.len() || values.iter().any(String::is_empty) {
        return Err(ContractError::Invalid(format!(
            "{field} must contain unique nonempty IDs"
        )));
    }
    for id in values {
        validate_identity(id, field)?;
    }
    Ok(())
}
