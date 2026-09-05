use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

use crate::{identity::validate_identity, ContractError, NativeScale};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct Instrument {
    pub instrument_id: String,
    pub venue: String,
    pub dataset_identity: String,
}

impl Instrument {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.instrument_id, "instrument_id")?;
        validate_identity(&self.venue, "venue")?;
        validate_identity(&self.dataset_identity, "dataset_identity")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ScopeInterval {
    pub instrument_id: String,
    pub scope_id: String,
    pub dataset_identity: String,
    pub start_time_ns: i64,
    pub end_time_ns: i64,
}

impl ScopeInterval {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.instrument_id, "instrument_id")?;
        validate_identity(&self.scope_id, "scope_id")?;
        validate_identity(&self.dataset_identity, "dataset_identity")?;
        if self.start_time_ns >= self.end_time_ns {
            return Err(ContractError::Invalid(format!(
                "scope interval is empty: {}",
                self.scope_id
            )));
        }
        Ok(())
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.instrument_id == other.instrument_id
            && self.dataset_identity == other.dataset_identity
            && self.start_time_ns < other.end_time_ns
            && other.start_time_ns < self.end_time_ns
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct InstrumentScope {
    pub instrument: Instrument,
    pub interval: ScopeInterval,
}

impl InstrumentScope {
    pub fn validate(&self) -> Result<(), ContractError> {
        self.instrument.validate()?;
        self.interval.validate()?;
        if self.interval.instrument_id != self.instrument.instrument_id
            || self.interval.dataset_identity != self.instrument.dataset_identity
        {
            return Err(ContractError::Invalid(
                "instrument scope identity mismatch".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AvailabilityRule {
    pub available_timestamp_field: String,
    pub sequence_field: String,
    pub source_blob_identity: String,
    pub rule_identity: String,
}

impl AvailabilityRule {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("available_timestamp_field", &self.available_timestamp_field),
            ("sequence_field", &self.sequence_field),
            ("source_blob_identity", &self.source_blob_identity),
            ("rule_identity", &self.rule_identity),
        ] {
            if value.is_empty() {
                return Err(ContractError::Invalid(format!("{field} cannot be empty")));
            }
        }
        if self.available_timestamp_field.contains("occur") {
            return Err(ContractError::Invalid(
                "availability cannot use occurrence time".into(),
            ));
        }
        validate_identity(&self.source_blob_identity, "source_blob_identity")?;
        validate_identity(&self.rule_identity, "rule_identity")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct OccurrenceRule {
    pub occurrence_timestamp_field: String,
    pub semantics: String,
    pub rule_identity: String,
}

impl OccurrenceRule {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            (
                "occurrence_timestamp_field",
                &self.occurrence_timestamp_field,
            ),
            ("semantics", &self.semantics),
        ] {
            if value.is_empty() {
                return Err(ContractError::Invalid(format!("{field} cannot be empty")));
            }
        }
        validate_identity(&self.rule_identity, "rule_identity")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct LifecycleTransition {
    pub from_state: String,
    pub to_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct LifecycleVocabulary {
    pub vocabulary_id: String,
    pub states: Vec<String>,
    pub transitions: Vec<LifecycleTransition>,
}

impl LifecycleVocabulary {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.vocabulary_id, "vocabulary_id")?;
        let states: HashSet<&str> = self.states.iter().map(String::as_str).collect();
        if states.len() != self.states.len()
            || states.is_empty()
            || self.states.iter().any(String::is_empty)
        {
            return Err(ContractError::Invalid(
                "lifecycle states must be unique and nonempty".into(),
            ));
        }
        for transition in &self.transitions {
            if !states.contains(transition.from_state.as_str())
                || !states.contains(transition.to_state.as_str())
            {
                return Err(ContractError::Invalid(
                    "lifecycle transition references unknown state".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AnchorDefinition {
    pub anchor_id: String,
    pub detector_id: String,
    pub lifecycle_state: String,
    pub availability_rule: AvailabilityRule,
    pub occurrence_rule: Option<OccurrenceRule>,
}

impl AnchorDefinition {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("anchor_id", &self.anchor_id),
            ("detector_id", &self.detector_id),
            ("lifecycle_state", &self.lifecycle_state),
        ] {
            validate_identity(value, field)?;
        }
        self.availability_rule.validate()?;
        if let Some(rule) = &self.occurrence_rule {
            rule.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProgramConsumer {
    MarketResearch,
    StrategyResearch,
    PortfolioResearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceAccess {
    UnconfirmedAllowed,
    ConfirmedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ContextPermission {
    pub permission_id: String,
    pub consumer: ProgramConsumer,
    pub allowed_detector_ids: Vec<String>,
    pub allowed_scales: BTreeSet<NativeScale>,
    pub evidence_access: EvidenceAccess,
}

impl ContextPermission {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.permission_id, "permission_id")?;
        let ids: HashSet<&str> = self
            .allowed_detector_ids
            .iter()
            .map(String::as_str)
            .collect();
        if ids.len() != self.allowed_detector_ids.len()
            || self.allowed_detector_ids.iter().any(String::is_empty)
        {
            return Err(ContractError::Invalid(
                "permission detector IDs must be unique and nonempty".into(),
            ));
        }
        if matches!(
            self.consumer,
            ProgramConsumer::StrategyResearch | ProgramConsumer::PortfolioResearch
        ) && self.evidence_access != EvidenceAccess::ConfirmedOnly
        {
            return Err(ContractError::Invalid(
                "strategy and portfolio context requires confirmed evidence".into(),
            ));
        }
        Ok(())
    }

    pub fn allows_confirmed_input(&self, confirmed: bool) -> bool {
        confirmed || self.evidence_access == EvidenceAccess::UnconfirmedAllowed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct NormalizationBasis {
    pub basis_identity: String,
    pub available_time_ns: i64,
    pub raw_value: f64,
    pub normalized_value: f64,
    pub formula: String,
}

impl NormalizationBasis {
    pub fn validate(&self, anchor_time_ns: i64) -> Result<(), ContractError> {
        validate_identity(&self.basis_identity, "basis_identity")?;
        if self.formula.is_empty()
            || !self.raw_value.is_finite()
            || !self.normalized_value.is_finite()
        {
            return Err(ContractError::Invalid(
                "normalization values and formula must be valid".into(),
            ));
        }
        if self.available_time_ns > anchor_time_ns {
            return Err(ContractError::Invalid(
                "normalization basis is available after anchor".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AnchorProgram {
    pub program_id: String,
    pub anchor_ids: Vec<String>,
    pub native_scales: BTreeSet<NativeScale>,
    pub context_permission_id: String,
}

impl AnchorProgram {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.program_id, "program_id")?;
        validate_identity(&self.context_permission_id, "context_permission_id")?;
        let ids: HashSet<&str> = self.anchor_ids.iter().map(String::as_str).collect();
        if ids.len() != self.anchor_ids.len()
            || self.anchor_ids.is_empty()
            || self.anchor_ids.iter().any(String::is_empty)
        {
            return Err(ContractError::Invalid(
                "anchor IDs must be unique and nonempty".into(),
            ));
        }
        if self.native_scales.is_empty() {
            return Err(ContractError::Invalid(
                "anchor scales cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldoutRole {
    Development,
    DetectorConfirmation,
    StrategyHoldout,
    PortfolioHoldout,
    FutureData,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct FutureDataPolicy {
    pub policy_id: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct HoldoutLeaf {
    pub leaf_id: String,
    pub scope: ScopeInterval,
    pub role: HoldoutRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct HoldoutPolicy {
    pub policy_id: String,
    pub frozen_at_ns: i64,
    pub dataset_identity: String,
    pub leaves: Vec<HoldoutLeaf>,
    pub future_data_policy: Option<FutureDataPolicy>,
}

impl HoldoutPolicy {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.policy_id, "policy_id")?;
        validate_identity(&self.dataset_identity, "dataset_identity")?;
        if self.leaves.is_empty() {
            return Err(ContractError::Invalid(
                "holdout policy needs explicit leaves".into(),
            ));
        }
        let mut leaf_ids = HashSet::new();
        for leaf in &self.leaves {
            validate_identity(&leaf.leaf_id, "leaf_id")?;
            if !leaf_ids.insert(&leaf.leaf_id) {
                return Err(ContractError::Invalid("duplicate holdout leaf".into()));
            }
            leaf.scope.validate()?;
            if leaf.scope.dataset_identity != self.dataset_identity {
                return Err(ContractError::Invalid(
                    "holdout leaf dataset mismatch".into(),
                ));
            }
            if matches!(leaf.role, HoldoutRole::FutureData)
                && self
                    .future_data_policy
                    .as_ref()
                    .is_none_or(|policy| !policy.allowed)
            {
                return Err(ContractError::Invalid(
                    "future-data leaf requires an allowed future-data policy".into(),
                ));
            }
        }
        for (index, left) in self.leaves.iter().enumerate() {
            if self.leaves[index + 1..]
                .iter()
                .any(|right| left.scope.overlaps(&right.scope))
            {
                return Err(ContractError::Invalid(
                    "holdout leaves must be disjoint".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExposureEvent {
    pub event_id: String,
    pub policy_id: String,
    pub actor_id: String,
    pub scope: ScopeInterval,
    pub role: HoldoutRole,
    pub exposed_at_ns: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExposureHistory {
    pub policy: HoldoutPolicy,
    pub events: Vec<ExposureEvent>,
}

impl ExposureHistory {
    pub fn new(policy: HoldoutPolicy) -> Result<Self, ContractError> {
        policy.validate()?;
        Ok(Self {
            policy,
            events: Vec::new(),
        })
    }
}

pub type ExposureLedger = ExposureHistory;

pub fn record_exposure(
    history: &mut ExposureHistory,
    event: ExposureEvent,
) -> Result<(), ContractError> {
    event.scope.validate()?;
    validate_identity(&event.event_id, "event_id")?;
    validate_identity(&event.actor_id, "actor_id")?;
    if event.policy_id != history.policy.policy_id
        || event.scope.dataset_identity != history.policy.dataset_identity
    {
        return Err(ContractError::Invalid(
            "exposure does not match frozen policy".into(),
        ));
    }
    if event.exposed_at_ns < history.policy.frozen_at_ns {
        return Err(ContractError::Invalid(
            "exposure predates frozen policy".into(),
        ));
    }
    if history
        .events
        .iter()
        .any(|prior| prior.event_id == event.event_id)
    {
        return Err(ContractError::Invalid(
            "exposure history is append-only".into(),
        ));
    }
    let leaf = history
        .policy
        .leaves
        .iter()
        .find(|leaf| leaf.scope.overlaps(&event.scope) && leaf.role == event.role)
        .ok_or_else(|| {
            ContractError::Invalid("exposure must name a declared holdout leaf".into())
        })?;
    if leaf.scope.scope_id != event.scope.scope_id {
        return Err(ContractError::Invalid(
            "renamed or overlapping scope cannot be treated as untouched".into(),
        ));
    }
    if history
        .events
        .iter()
        .any(|prior| prior.exposed_at_ns > event.exposed_at_ns)
    {
        return Err(ContractError::Invalid(
            "exposure history must be temporally ordered".into(),
        ));
    }
    let contaminated = history.events.iter().any(|prior| {
        prior.scope.overlaps(&event.scope)
            && matches!(prior.role, HoldoutRole::DetectorConfirmation)
            && matches!(
                event.role,
                HoldoutRole::StrategyHoldout | HoldoutRole::PortfolioHoldout
            )
    });
    if contaminated {
        return Err(ContractError::Invalid(
            "detector confirmation exposure contaminates strategy or portfolio holdout".into(),
        ));
    }
    history.events.push(event);
    Ok(())
}
