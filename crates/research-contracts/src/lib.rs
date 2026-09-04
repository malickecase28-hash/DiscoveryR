use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureClass {
    E1,
    E2,
    E3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BarScale {
    #[serde(rename = "15s")]
    S15,
    #[serde(rename = "30s")]
    S30,
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    H4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NativeScaleScope {
    Tick,
    Explicit { scales: Vec<BarScale> },
    AllRegistered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Anchor {
    pub detector_id: String,
    pub lifecycle_state: String,
    pub anchor_time_semantics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorTimeRule {
    pub source: String,
    pub semantics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EligibleContext {
    pub allowed_detectors: Vec<String>,
    pub requested_detectors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryStatus {
    Predeclared,
    Discovered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SemanticStatus {
    Authoritative,
    Provisional,
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DetectorRole {
    DirectionalEvidence,
    StateRegime,
    StructuralObject,
    TemporalContext,
    DataQuality,
    LifecycleState,
    NormalizationReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectorRegistryEntry {
    pub detector_id: String,
    pub name: String,
    pub description: String,
    pub lifecycle_vocabulary: Vec<String>,
    pub roles: Vec<DetectorRole>,
    pub derives_from: Vec<String>,
    pub native_scales: NativeScaleScope,
    pub known_surfaces: Vec<String>,
    pub semantic_status: SemanticStatus,
    pub availability_semantics: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordType {
    Question,
    Finding,
    Challenge,
    Knowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingStatus {
    Discovery,
    Candidate,
    Confirmed,
    Rejected,
    Null,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectionReason {
    MethodFailure,
    ConfirmationFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub experiment_id: Option<String>,
    pub run_id: Option<String>,
    pub evidence_ids: Vec<String>,
    pub code_identity: Option<String>,
    pub researcher_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeRecord {
    pub record_id: String,
    pub record_type: RecordType,
    pub content: Value,
    pub status: RecordStatus,
    pub provenance: Provenance,
    pub created_utc: String,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
    pub rejection_reason: Option<RejectionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordStatus {
    Open,
    Discovery,
    Candidate,
    Confirmed,
    Rejected,
    Null,
    Inconclusive,
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

pub fn validate_contract(contract: &ExperimentContract) -> Result<(), ContractError> {
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
    if record.record_type == RecordType::Finding
        && record.status == RecordStatus::Rejected
        && record.rejection_reason.is_none()
    {
        return Err(ContractError::Invalid(
            "rejected finding requires rejection_reason".into(),
        ));
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

#[cfg(test)]
mod tests {
    use super::*;
    fn contract(
        exposure_class: ExposureClass,
        native_scale_scope: NativeScaleScope,
    ) -> ExperimentContract {
        ExperimentContract {
            experiment_id: "AE-001".into(),
            anchor: Anchor {
                detector_id: "fvg".into(),
                lifecycle_state: "formed".into(),
                anchor_time_semantics: "known_at_anchor".into(),
            },
            instrument: "XAUUSD".into(),
            anchor_time_rule: AnchorTimeRule {
                source: "known_time".into(),
                semantics: "causal boundary".into(),
            },
            exposure_class,
            native_scale_scope,
            eligible_context: EligibleContext {
                allowed_detectors: vec!["range".into()],
                requested_detectors: vec!["range".into()],
            },
            population: Value::String("all anchors".into()),
            measurements_and_outcomes: Value::Array(vec![]),
            controls: Value::Null,
            development_data_scope: Value::String("development".into()),
            confirmation_data_scope: Value::String("confirmation".into()),
            discovery_status: DiscoveryStatus::Predeclared,
            normalization_basis: "native".into(),
            created_utc: "2026-09-03T00:00:00Z".into(),
        }
    }
    #[test]
    fn causal_boundaries() {
        assert!(ensure_causal(10, 9).is_ok());
        assert!(ensure_causal(10, 10).is_ok());
        assert!(ensure_causal(10, 11).is_err());
    }
    #[test]
    fn exposure_classes_do_not_require_history() {
        for e in [ExposureClass::E1, ExposureClass::E2, ExposureClass::E3] {
            assert!(validate_contract(&contract(e, NativeScaleScope::Tick)).is_ok());
        }
    }
    #[test]
    fn native_scales_parse() {
        for json in [
            r#"{"kind":"tick"}"#,
            r#"{"kind":"explicit","scales":["15m"]}"#,
            r#"{"kind":"explicit","scales":["15s","1h"]}"#,
            r#"{"kind":"all_registered"}"#,
        ] {
            assert!(serde_json::from_str::<NativeScaleScope>(json).is_ok());
        }
    }
    #[test]
    fn unresolved_detector_is_valid() {
        assert!(serde_json::from_str::<DetectorRegistryEntry>(r#"{"detector_id":"x","name":"x","description":"unresolved","lifecycle_vocabulary":[],"roles":[],"derives_from":[],"native_scales":{"kind":"tick"},"known_surfaces":[],"semantic_status":"UNRESOLVED","availability_semantics":"UNRESOLVED","domain":"unknown"}"#).is_ok());
    }
    #[test]
    fn execution_identity_stays_out_of_contract() {
        let c = serde_json::to_value(contract(ExposureClass::E1, NativeScaleScope::Tick)).unwrap();
        assert!(c.get("researcher_id").is_none());
        assert!(serde_json::to_value(RunManifest {
            run_id: "r".into(),
            experiment_id: "e".into(),
            researcher_id: "P-01".into(),
            lab: "lab".into(),
            code_identity: "sha".into(),
            contract_hash: "h".into(),
            input_identity: "i".into(),
            executed_utc: "t".into(),
            output_identity: "o".into()
        })
        .unwrap()
        .get("researcher_id")
        .is_some());
        assert!(c.get("provider").is_none());
    }
    fn finding(status: RecordStatus, reason: Option<RejectionReason>) -> KnowledgeRecord {
        KnowledgeRecord {
            record_id: "F-1".into(),
            record_type: RecordType::Finding,
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
            superseded_by: Some("F-2".into()),
            rejection_reason: reason,
        }
    }
    #[test]
    fn knowledge_statuses_and_rejections() {
        for status in ["NULL", "INCONCLUSIVE"] {
            let json = format!(
                r#"{{"record_id":"F-1","record_type":"FINDING","content":null,"status":"{status}","provenance":{{"evidence_ids":[]}},"created_utc":"t"}}"#
            );
            let record: KnowledgeRecord = serde_json::from_str(&json).unwrap();
            assert!(validate_knowledge_record(&record).is_ok());
        }
        for s in [RecordStatus::Null, RecordStatus::Inconclusive] {
            assert!(validate_knowledge_record(&finding(s, None)).is_ok());
        }
        assert!(validate_knowledge_record(&finding(
            RecordStatus::Rejected,
            Some(RejectionReason::MethodFailure)
        ))
        .is_ok());
        assert!(validate_knowledge_record(&finding(
            RecordStatus::Rejected,
            Some(RejectionReason::ConfirmationFailure)
        ))
        .is_ok());
        assert!(validate_knowledge_record(&finding(RecordStatus::Rejected, None)).is_err());
    }
    #[test]
    fn context_allowlist() {
        let mut c = contract(ExposureClass::E1, NativeScaleScope::Tick);
        assert!(validate_contract(&c).is_ok());
        c.eligible_context.requested_detectors = vec!["fvg".into()];
        assert!(validate_contract(&c).is_err());
    }
    #[test]
    fn superseded_records_remain_representable() {
        let r = finding(RecordStatus::Null, None);
        assert_eq!(r.superseded_by.as_deref(), Some("F-2"));
    }
}
