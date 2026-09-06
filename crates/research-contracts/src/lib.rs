use crate::identity::validate_identity;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, fmt};

pub mod authority;
pub mod evidence;
pub mod identity;
pub mod programs;
pub mod submission;
pub mod timing;

pub use authority::{
    validate_compatibility, validate_compatibility_for_scope, AuthorityCompatibility,
    AuthorityDelta, DetectorVersion, ProducerAuthority,
};
pub use evidence::{
    AttackKind, ChallengeResult, ConfirmationActor, ConfirmationContract, ConfirmationState,
    ControlDesign, EvidenceState, Hypothesis, KnowledgeEnvelope, KnowledgeFacets, KnowledgeQuery,
    MethodChallenge, NullDesign, OutcomeDefinition, Question,
};
pub use identity::{
    validate_detector_lineage, validate_object_lineage, CompleteReproducibilityIdentity,
    DetectorLineage, ObjectIdentity, ReproducibilityIdentity,
};
pub use programs::{
    CostModel, DecisionRule, ExecutionAssumption, PortfolioComponent, PortfolioConstraint,
    PortfolioConstraintKind, RiskRule, StrategyHypothesis, StrategyValidation, UserParameter,
};
pub use timing::{
    record_exposure, AnchorDefinition, AnchorProgram, AvailabilityRule, ContextPermission,
    EvidenceAccess, ExposureEvent, ExposureHistory, FutureDataPolicy, HoldoutLeaf, HoldoutPolicy,
    HoldoutRole, Instrument, InstrumentScope, LifecycleTransition, LifecycleVocabulary,
    NormalizationBasis, OccurrenceRule, ProgramConsumer, ScopeInterval,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureClass {
    E1,
    E2,
    E3,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum BarScale {
    #[serde(rename = "15s")]
    #[schemars(rename = "15s")]
    S15,
    #[serde(rename = "30s")]
    #[schemars(rename = "30s")]
    S30,
    #[serde(rename = "1m")]
    #[schemars(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    #[schemars(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    #[schemars(rename = "15m")]
    M15,
    #[serde(rename = "1h")]
    #[schemars(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    #[schemars(rename = "4h")]
    H4,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum NativeScale {
    Tick,
    Bar(BarScale),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeScaleScope {
    Tick,
    Explicit { scales: Vec<BarScale> },
    AllRegistered,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct Anchor {
    pub detector_id: String,
    pub lifecycle_state: String,
    pub anchor_time_semantics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AnchorTimeRule {
    pub source: String,
    pub semantics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct EligibleContext {
    pub allowed_detectors: Vec<String>,
    pub requested_detectors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExperimentContract {
    pub experiment_id: String,
    pub anchor: Anchor,
    pub instrument: String,
    pub anchor_time_rule: AnchorTimeRule,
    pub exposure_class: ExposureClass,
    pub native_scale_scope: NativeScaleScope,
    pub eligible_context: EligibleContext,
    pub population: Value,
    pub measurements_and_outcomes: Value,
    pub controls: Value,
    pub development_data_scope: Value,
    pub confirmation_data_scope: Value,
    pub discovery_status: DiscoveryStatus,
    pub normalization_basis: String,
    pub created_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryStatus {
    Predeclared,
    Discovered,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct RunManifest {
    pub run_id: String,
    pub experiment_id: String,
    pub researcher_id: String,
    pub lab: String,
    pub code_identity: String,
    pub contract_hash: String,
    pub input_identity: String,
    pub executed_utc: String,
    pub output_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SemanticStatus {
    Authoritative,
    Provisional,
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DetectorRole {
    LifecycleObject,
    StructuralObject,
    Event,
    StateRegime,
    DirectionalContext,
    QualityInstrumentation,
    NormalizationMeasure,
    TemporalContext,
    DerivedObject,
    CompositeContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "DetectorRole")]
pub enum LegacyDetectorRole {
    DirectionalEvidence,
    StateRegime,
    StructuralObject,
    TemporalContext,
    DataQuality,
    LifecycleState,
    NormalizationReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct DetectorRegistryEntry {
    pub detector_id: String,
    pub name: String,
    pub description: String,
    pub lifecycle_vocabulary: Vec<String>,
    pub roles: Vec<LegacyDetectorRole>,
    pub derives_from: Vec<String>,
    pub native_scales: NativeScaleScope,
    pub known_surfaces: Vec<String>,
    pub semantic_status: SemanticStatus,
    pub availability_semantics: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct Provenance {
    pub experiment_id: Option<String>,
    pub run_id: Option<String>,
    pub evidence_ids: Vec<String>,
    pub code_identity: Option<String>,
    pub researcher_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingStatus {
    Discovery,
    Candidate,
    Confirmed,
    Rejected,
    Null,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectionReason {
    MethodFailure,
    ConfirmationFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeDecision {
    Open,
    Survived,
    Failed,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionStatus {
    Open,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "record_type", rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(deny_unknown_fields)]
pub enum KnowledgeRecord {
    Question(QuestionRecord),
    Finding(FindingRecord),
    Challenge(ChallengeRecord),
    Knowledge(AcceptedKnowledgeRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct QuestionRecord {
    pub record_id: String,
    pub content: Value,
    pub status: QuestionStatus,
    pub provenance: Provenance,
    pub created_utc: String,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct FindingRecord {
    pub record_id: String,
    pub content: Value,
    pub status: FindingStatus,
    pub provenance: Provenance,
    pub created_utc: String,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
    pub rejection_reason: Option<RejectionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ChallengeRecord {
    pub record_id: String,
    pub content: Value,
    pub decision: ChallengeDecision,
    pub provenance: Provenance,
    pub created_utc: String,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AcceptedKnowledgeRecord {
    pub record_id: String,
    pub content: Value,
    pub provenance: Provenance,
    pub created_utc: String,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalViolation {
    pub anchor_time: i64,
    pub available_time: i64,
}
impl fmt::Display for CausalViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "context available at {} after anchor {}",
            self.available_time, self.anchor_time
        )
    }
}
impl std::error::Error for CausalViolation {}

pub fn ensure_causal(anchor_time: i64, available_time: i64) -> Result<(), CausalViolation> {
    if available_time <= anchor_time {
        Ok(())
    } else {
        Err(CausalViolation {
            anchor_time,
            available_time,
        })
    }
}
pub fn validate_context_available(
    anchor_time: i64,
    context_available_time: i64,
) -> Result<(), CausalViolation> {
    ensure_causal(anchor_time, context_available_time)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    Invalid(String),
}
impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => f.write_str(s),
        }
    }
}
impl std::error::Error for ContractError {}

fn validate_scale_scope(scope: &NativeScaleScope) -> Result<(), ContractError> {
    if let NativeScaleScope::Explicit { scales } = scope {
        if scales.is_empty() {
            return Err(ContractError::Invalid(
                "explicit native scales cannot be empty".into(),
            ));
        }
        if scales.iter().collect::<HashSet<_>>().len() != scales.len() {
            return Err(ContractError::Invalid(
                "explicit native scales cannot contain duplicates".into(),
            ));
        }
    }
    Ok(())
}

pub fn validate_contract(contract: &ExperimentContract) -> Result<(), ContractError> {
    validate_scale_scope(&contract.native_scale_scope)?;
    let allowed: HashSet<_> = contract.eligible_context.allowed_detectors.iter().collect();
    if let Some(detector) = contract
        .eligible_context
        .requested_detectors
        .iter()
        .find(|d| !allowed.contains(d))
    {
        return Err(ContractError::Invalid(format!(
            "context detector is not allowed: {detector}"
        )));
    }
    Ok(())
}

pub fn validate_knowledge_record(record: &KnowledgeRecord) -> Result<(), ContractError> {
    let (record_id, provenance) = match record {
        KnowledgeRecord::Question(record) => (&record.record_id, &record.provenance),
        KnowledgeRecord::Finding(record) => (&record.record_id, &record.provenance),
        KnowledgeRecord::Challenge(record) => (&record.record_id, &record.provenance),
        KnowledgeRecord::Knowledge(record) => (&record.record_id, &record.provenance),
    };
    validate_identity(record_id, "record_id")?;
    for id in &provenance.evidence_ids {
        validate_identity(id, "provenance evidence_id")?;
    }
    for id in provenance
        .experiment_id
        .iter()
        .chain(provenance.run_id.iter())
        .chain(provenance.code_identity.iter())
        .chain(provenance.researcher_id.iter())
    {
        validate_identity(id, "provenance identity")?;
    }
    if let KnowledgeRecord::Finding(finding) = record {
        match (&finding.status, &finding.rejection_reason) {
            (FindingStatus::Rejected, Some(_)) => Ok(()),
            (FindingStatus::Rejected, None) => Err(ContractError::Invalid(
                "rejected finding requires rejection_reason".into(),
            )),
            (_, Some(_)) => Err(ContractError::Invalid(
                "only rejected findings may have rejection_reason".into(),
            )),
            (_, None) => Ok(()),
        }
    } else {
        Ok(())
    }
}

pub fn validate_registry(entries: &[DetectorRegistryEntry]) -> Result<(), ContractError> {
    if entries.len() != 23 {
        return Err(ContractError::Invalid(format!(
            "expected 23 detector entries, got {}",
            entries.len()
        )));
    }
    let ids: HashSet<_> = entries.iter().map(|e| e.detector_id.as_str()).collect();
    if ids.len() != entries.len() {
        return Err(ContractError::Invalid("detector IDs must be unique".into()));
    }
    let bar_scales = vec![
        BarScale::S15,
        BarScale::S30,
        BarScale::M1,
        BarScale::M5,
        BarScale::M15,
        BarScale::H1,
        BarScale::H4,
    ];
    for entry in entries {
        if entry.known_surfaces.is_empty() {
            return Err(ContractError::Invalid(format!(
                "detector has no known surface: {}",
                entry.detector_id
            )));
        }
        validate_scale_scope(&entry.native_scales)?;
        let derives: HashSet<_> = entry.derives_from.iter().collect();
        if derives.len() != entry.derives_from.len() {
            return Err(ContractError::Invalid(format!(
                "duplicate derives_from entry: {}",
                entry.detector_id
            )));
        }
        if entry
            .derives_from
            .iter()
            .any(|id| !ids.contains(id.as_str()))
        {
            return Err(ContractError::Invalid(format!(
                "unknown derives_from entry: {}",
                entry.detector_id
            )));
        }
        match entry.domain.as_str() {
            "bar"
                if entry.native_scales
                    == NativeScaleScope::Explicit {
                        scales: bar_scales.clone(),
                    } => {}
            "tick" if entry.native_scales == NativeScaleScope::Tick => {}
            "bar" => {
                return Err(ContractError::Invalid(format!(
                    "bar detector has wrong native scales: {}",
                    entry.detector_id
                )))
            }
            "tick" => {
                return Err(ContractError::Invalid(format!(
                    "tick detector has wrong native scales: {}",
                    entry.detector_id
                )))
            }
            _ => {
                return Err(ContractError::Invalid(format!(
                    "unknown detector domain: {}",
                    entry.domain
                )))
            }
        }
    }
    Ok(())
}

pub fn parse_and_validate_contract(
    json: &str,
) -> Result<ExperimentContract, Box<dyn std::error::Error>> {
    let contract: ExperimentContract = serde_json::from_str(json)?;
    validate_contract(&contract)?;
    Ok(contract)
}
pub fn parse_and_validate_registry(
    json: &str,
) -> Result<Vec<DetectorRegistryEntry>, Box<dyn std::error::Error>> {
    let entries: Vec<DetectorRegistryEntry> = serde_json::from_str(json)?;
    validate_registry(&entries)?;
    Ok(entries)
}
pub fn parse_and_validate_knowledge_record(
    json: &str,
) -> Result<KnowledgeRecord, Box<dyn std::error::Error>> {
    let record: KnowledgeRecord = serde_json::from_str(json)?;
    validate_knowledge_record(&record)?;
    Ok(record)
}

pub fn schema_json<T: JsonSchema>() -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(&schema_for!(T)).expect("schema serialization is infallible")
    )
}
pub fn generated_schemas() -> [(&'static str, String); 4] {
    [
        (
            "experiment_contract_v1.schema.json",
            schema_json::<ExperimentContract>(),
        ),
        ("run_manifest_v1.schema.json", schema_json::<RunManifest>()),
        (
            "detector_registry_entry_v1.schema.json",
            schema_json::<DetectorRegistryEntry>(),
        ),
        (
            "knowledge_record_v1.schema.json",
            schema_json::<KnowledgeRecord>(),
        ),
    ]
}

pub fn generated_additive_schemas() -> Vec<(&'static str, String)> {
    vec![
        ("instrument_v1.schema.json", schema_json::<Instrument>()),
        (
            "instrument_scope_v1.schema.json",
            schema_json::<InstrumentScope>(),
        ),
        (
            "producer_authority_v1.schema.json",
            schema_json::<ProducerAuthority>(),
        ),
        (
            "authority_compatibility_v1.schema.json",
            schema_json::<AuthorityCompatibility>(),
        ),
        (
            "detector_role_v1.schema.json",
            schema_json::<DetectorRole>(),
        ),
        (
            "availability_rule_v1.schema.json",
            schema_json::<AvailabilityRule>(),
        ),
        (
            "occurrence_rule_v1.schema.json",
            schema_json::<OccurrenceRule>(),
        ),
        (
            "holdout_policy_v1.schema.json",
            schema_json::<HoldoutPolicy>(),
        ),
        (
            "exposure_event_v1.schema.json",
            schema_json::<ExposureEvent>(),
        ),
        (
            "normalization_basis_v1.schema.json",
            schema_json::<NormalizationBasis>(),
        ),
        (
            "knowledge_envelope_v1.schema.json",
            schema_json::<KnowledgeEnvelope>(),
        ),
        ("question_v1.schema.json", schema_json::<Question>()),
        ("hypothesis_v1.schema.json", schema_json::<Hypothesis>()),
        (
            "method_challenge_v1.schema.json",
            schema_json::<MethodChallenge>(),
        ),
        (
            "confirmation_contract_v1.schema.json",
            schema_json::<ConfirmationContract>(),
        ),
        (
            "strategy_hypothesis_v1.schema.json",
            schema_json::<StrategyHypothesis>(),
        ),
        (
            "decision_rule_v1.schema.json",
            schema_json::<DecisionRule>(),
        ),
        (
            "execution_assumption_v1.schema.json",
            schema_json::<ExecutionAssumption>(),
        ),
        ("cost_model_v1.schema.json", schema_json::<CostModel>()),
        ("risk_rule_v1.schema.json", schema_json::<RiskRule>()),
        (
            "strategy_validation_v1.schema.json",
            schema_json::<StrategyValidation>(),
        ),
        (
            "portfolio_component_v1.schema.json",
            schema_json::<PortfolioComponent>(),
        ),
        (
            "portfolio_constraint_v1.schema.json",
            schema_json::<PortfolioConstraint>(),
        ),
        (
            "reproducibility_identity_v1.schema.json",
            schema_json::<ReproducibilityIdentity>(),
        ),
        (
            "complete_reproducibility_identity_v1.schema.json",
            schema_json::<CompleteReproducibilityIdentity>(),
        ),
        (
            "object_identity_v1.schema.json",
            schema_json::<ObjectIdentity>(),
        ),
        (
            "detector_lineage_v1.schema.json",
            schema_json::<DetectorLineage>(),
        ),
        (
            "anchor_definition_v1.schema.json",
            schema_json::<AnchorDefinition>(),
        ),
        (
            "anchor_program_v1.schema.json",
            schema_json::<AnchorProgram>(),
        ),
        (
            "context_permission_v1.schema.json",
            schema_json::<ContextPermission>(),
        ),
        (
            "lifecycle_vocabulary_v1.schema.json",
            schema_json::<LifecycleVocabulary>(),
        ),
        (
            "lifecycle_transition_v1.schema.json",
            schema_json::<LifecycleTransition>(),
        ),
        (
            "scope_interval_v1.schema.json",
            schema_json::<ScopeInterval>(),
        ),
        (
            "future_data_policy_v1.schema.json",
            schema_json::<FutureDataPolicy>(),
        ),
        ("holdout_leaf_v1.schema.json", schema_json::<HoldoutLeaf>()),
        (
            "exposure_history_v1.schema.json",
            schema_json::<ExposureHistory>(),
        ),
        (
            "control_design_v1.schema.json",
            schema_json::<ControlDesign>(),
        ),
        ("null_design_v1.schema.json", schema_json::<NullDesign>()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn causal_boundaries() {
        assert!(ensure_causal(10, 9).is_ok());
        assert!(ensure_causal(10, 10).is_ok());
        assert!(ensure_causal(10, 11).is_err());
    }
    #[test]
    fn native_scale_validation() {
        for json in [
            r#"{"kind":"tick"}"#,
            r#"{"kind":"explicit","scales":["15m"]}"#,
            r#"{"kind":"explicit","scales":["15s","1h"]}"#,
            r#"{"kind":"all_registered"}"#,
        ] {
            assert!(serde_json::from_str::<NativeScaleScope>(json).is_ok());
        }
        assert!(validate_scale_scope(&NativeScaleScope::Explicit { scales: vec![] }).is_err());
        assert!(validate_scale_scope(&NativeScaleScope::Explicit {
            scales: vec![BarScale::M1, BarScale::M1]
        })
        .is_err());
    }
    #[test]
    fn nested_unknown_fields_rejected() {
        assert!(serde_json::from_str::<Anchor>(r#"{"detector_id":"x","lifecycle_state":"x","anchor_time_semantics":"x","provider":"x"}"#).is_err());
        assert!(serde_json::from_str::<EligibleContext>(
            r#"{"allowed_detectors":[],"requested_detectors":[],"provider":"x"}"#
        )
        .is_err());
    }
    #[test]
    fn rejected_reason_is_exact() {
        let finding = |status, reason| {
            KnowledgeRecord::Finding(FindingRecord {
                record_id: "F".into(),
                content: Value::Null,
                status,
                provenance: Provenance {
                    experiment_id: None,
                    run_id: None,
                    evidence_ids: vec![],
                    code_identity: None,
                    researcher_id: None,
                },
                created_utc: "t".into(),
                supersedes: None,
                superseded_by: None,
                rejection_reason: reason,
            })
        };
        assert!(validate_knowledge_record(&finding(
            FindingStatus::Rejected,
            Some(RejectionReason::MethodFailure)
        ))
        .is_ok());
        assert!(validate_knowledge_record(&finding(
            FindingStatus::Rejected,
            Some(RejectionReason::ConfirmationFailure)
        ))
        .is_ok());
        assert!(validate_knowledge_record(&finding(FindingStatus::Rejected, None)).is_err());
        assert!(validate_knowledge_record(&finding(
            FindingStatus::Null,
            Some(RejectionReason::MethodFailure)
        ))
        .is_err());
    }
    #[test]
    fn tagged_knowledge_variants() {
        for json in [
            r#"{"record_type":"QUESTION","record_id":"Q","content":null,"status":"OPEN","provenance":{"evidence_ids":[]},"created_utc":"t"}"#,
            r#"{"record_type":"FINDING","record_id":"F","content":null,"status":"NULL","provenance":{"evidence_ids":[]},"created_utc":"t"}"#,
            r#"{"record_type":"CHALLENGE","record_id":"C","content":null,"decision":"SURVIVED","provenance":{"evidence_ids":[]},"created_utc":"t"}"#,
            r#"{"record_type":"KNOWLEDGE","record_id":"K","content":null,"provenance":{"evidence_ids":[]},"created_utc":"t"}"#,
        ] {
            assert!(parse_and_validate_knowledge_record(json).is_ok());
        }
    }
}
