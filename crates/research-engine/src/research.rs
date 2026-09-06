//! Functional, bounded Program R over causal synthetic tape records.

use crate::{DetectorAtlas, QuestionGenerationReport, QuestionGenerationRequest};
use research_contracts::{
    CompleteReproducibilityIdentity, ContextPermission, ContractError, EvidenceState,
    ReproducibilityIdentity,
};
use research_tape::{AnchorInstance, ContextObservation};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CausalResearchRecord {
    pub anchor: AnchorInstance,
    pub contexts: Vec<ContextObservation>,
}
impl CausalResearchRecord {
    pub fn from_tape(
        anchor: AnchorInstance,
        contexts: Vec<ContextObservation>,
    ) -> Result<Self, ContractError> {
        if anchor.anchor_time < 0
            || anchor.value.is_some_and(|value| !value.is_finite())
            || contexts.iter().any(|c| {
                c.available_time < 0
                    || c.available_time > anchor.anchor_time
                    || c.value.is_some_and(|value| !value.is_finite())
            })
        {
            return Err(ContractError::Invalid(
                "invalid causal research record".into(),
            ));
        }
        Ok(Self { anchor, contexts })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchPlan {
    pub atlas: DetectorAtlas,
    pub request: QuestionGenerationRequest,
    pub permission: ContextPermission,
    pub reproducibility: ReproducibilityIdentity,
    pub complete_reproducibility: CompleteReproducibilityIdentity,
    pub max_records: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NativePhenotype {
    pub samples: usize,
    pub values: Vec<f64>,
    pub mean: Option<f64>,
    pub identity: String,
    pub status: EvidenceState,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiscoveryResult {
    pub domain: String,
    pub observations: usize,
    pub estimate: Option<f64>,
    pub lag_ns: i64,
    pub identity: String,
    pub status: EvidenceState,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CandidateFormation {
    pub candidate_ids: Vec<String>,
    pub identity: String,
    pub status: EvidenceState,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Characterization {
    pub candidate_id: String,
    pub sample_count: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub identity: String,
    pub status: EvidenceState,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InformationSystemReport {
    pub instrument_id: String,
    pub source_identities: Vec<String>,
    pub scope_start_ns: Option<i64>,
    pub scope_end_ns: Option<i64>,
    pub identity: String,
    pub status: EvidenceState,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResearchKnowledgeRecord {
    pub record_id: String,
    pub source_identity: String,
    pub stage_identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResearchReport {
    pub question_report: QuestionGenerationReport,
    pub records: usize,
    pub evidence_state: EvidenceState,
    pub output_identity: String,
    pub provenance: ReproducibilityIdentity,
    pub complete_provenance: CompleteReproducibilityIdentity,
    pub native_phenotype: NativePhenotype,
    pub same_domain_discovery: DiscoveryResult,
    pub cross_domain_discovery: DiscoveryResult,
    pub candidates: CandidateFormation,
    pub characterization: Vec<Characterization>,
    pub information_system: InformationSystemReport,
    pub knowledge_records: Vec<ResearchKnowledgeRecord>,
}

pub fn run_research(
    plan: &ResearchPlan,
    records: &[CausalResearchRecord],
) -> Result<ResearchReport, ContractError> {
    plan.reproducibility.validate()?;
    plan.complete_reproducibility.validate()?;
    if plan.max_records == 0 || records.len() > plan.max_records {
        return Err(ContractError::Invalid("research record bound".into()));
    }
    for record in records {
        CausalResearchRecord::from_tape(record.anchor.clone(), record.contexts.clone())?;
        if record.anchor.detector_id != plan.request.anchor_detector_id
            || record
                .contexts
                .iter()
                .any(|c| !plan.request.context_detector_ids.contains(&c.detector_id))
        {
            return Err(ContractError::Invalid(
                "research record does not match frozen plan".into(),
            ));
        }
    }
    let questions = plan
        .atlas
        .generate_questions(&plan.request, &plan.permission)?;
    let values = records
        .iter()
        .filter_map(|r| r.anchor.value)
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    let mean = (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64);
    let raw_identity = hash(&(
        plan.complete_reproducibility.clone(),
        plan.request.clone(),
        plan.permission.clone(),
        records,
        &questions,
    ));
    let native = NativePhenotype {
        samples: values.len(),
        values: values.clone(),
        mean,
        identity: hash(&values),
        status: if values.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Known
        },
    };
    let context_values = records
        .iter()
        .flat_map(|r| r.contexts.iter().filter_map(|c| c.value))
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    let same = discovery("same-domain", &values, 0, &raw_identity);
    let cross = discovery(
        "cross-domain",
        &context_values,
        records.first().map_or(0, |r| r.anchor.anchor_time),
        &raw_identity,
    );
    let candidate_ids = if values.len() >= 2 {
        vec![format!("candidate-{}", &raw_identity[..12])]
    } else {
        Vec::new()
    };
    let candidates = CandidateFormation {
        candidate_ids: candidate_ids.clone(),
        identity: hash(&candidate_ids),
        status: if candidate_ids.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Partial
        },
    };
    let characterization = candidate_ids
        .iter()
        .map(|id| Characterization {
            candidate_id: id.clone(),
            sample_count: values.len(),
            positive_count: values.iter().filter(|v| **v > 0.0).count(),
            negative_count: values.iter().filter(|v| **v < 0.0).count(),
            identity: hash(&(id, &values)),
            status: EvidenceState::Partial,
        })
        .collect::<Vec<_>>();
    let source_identities = records
        .iter()
        .map(|r| format!("{}:{}", r.anchor.source.part, r.anchor.source.row_index))
        .collect::<Vec<_>>();
    let info = InformationSystemReport {
        instrument_id: plan
            .request
            .instrument_ids
            .first()
            .cloned()
            .unwrap_or_default(),
        source_identities: source_identities.clone(),
        scope_start_ns: records.iter().map(|r| r.anchor.anchor_time).min(),
        scope_end_ns: records.iter().map(|r| r.anchor.anchor_time).max(),
        identity: hash(&(source_identities, &plan.request.instrument_ids)),
        status: if records.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Known
        },
    };
    let knowledge_records = characterization
        .iter()
        .map(|c| ResearchKnowledgeRecord {
            record_id: format!("knowledge-{}", c.identity),
            source_identity: info.identity.clone(),
            stage_identity: c.identity.clone(),
            status: c.status.clone(),
        })
        .collect::<Vec<_>>();
    let evidence_state = if records.is_empty() {
        EvidenceState::Unknown
    } else {
        questions.status.clone()
    };
    Ok(ResearchReport {
        question_report: questions,
        records: records.len(),
        evidence_state,
        output_identity: hash(&(
            raw_identity,
            &native,
            &same,
            &cross,
            &candidates,
            &characterization,
            &info,
            &knowledge_records,
        )),
        provenance: plan.reproducibility.clone(),
        complete_provenance: plan.complete_reproducibility.clone(),
        native_phenotype: native,
        same_domain_discovery: same,
        cross_domain_discovery: cross,
        candidates,
        characterization,
        information_system: info,
        knowledge_records,
    })
}

fn discovery(domain: &str, values: &[f64], lag_ns: i64, parent: &str) -> DiscoveryResult {
    let estimate = (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64);
    DiscoveryResult {
        domain: domain.into(),
        observations: values.len(),
        estimate,
        lag_ns,
        identity: hash(&(domain, values, lag_ns, parent)),
        status: if values.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Partial
        },
    }
}
fn hash<T: Serialize>(value: &T) -> String {
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(value).unwrap_or_default());
    format!("{:x}", h.finalize())
}
