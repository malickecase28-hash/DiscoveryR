//! Bounded synthetic-capable Program R runner.

use crate::{DetectorAtlas, QuestionGenerationReport, QuestionGenerationRequest};
use research_contracts::{
    ContextPermission, ContractError, EvidenceState, ReproducibilityIdentity,
};
use research_tape::{AnchorInstance, ContextObservation};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
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
            || contexts.iter().any(|context| {
                context.available_time < 0 || context.available_time > anchor.anchor_time
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
    pub max_records: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchReport {
    pub question_report: QuestionGenerationReport,
    pub records: usize,
    pub evidence_state: EvidenceState,
    pub output_identity: String,
    pub provenance: ReproducibilityIdentity,
}

pub fn run_research(
    plan: &ResearchPlan,
    records: &[CausalResearchRecord],
) -> Result<ResearchReport, ContractError> {
    plan.reproducibility.validate()?;
    if plan.max_records == 0 || records.len() > plan.max_records {
        return Err(ContractError::Invalid("research record bound".into()));
    }
    for record in records {
        CausalResearchRecord::from_tape(record.anchor.clone(), record.contexts.clone())?;
        if record.anchor.detector_id != plan.request.anchor_detector_id
            || record.anchor.anchor_time != plan.request.anchor_time_ns
            || record.contexts.iter().any(|context| {
                !plan
                    .request
                    .context_detector_ids
                    .contains(&context.detector_id)
                    || context.available_time > record.anchor.anchor_time
            })
        {
            return Err(ContractError::Invalid(
                "research record does not match frozen plan".into(),
            ));
        }
    }
    let questions = plan
        .atlas
        .generate_questions(&plan.request, &plan.permission)?;
    let mut hash = Sha256::new();
    hash.update(plan.reproducibility.identity_hash()?.as_bytes());
    hash.update((records.len() as u64).to_le_bytes());
    hash.update(serde_json::to_vec(&questions.questions.len()).unwrap_or_default());
    let output_identity = format!("{:x}", hash.finalize());
    let evidence_state = if records.is_empty() {
        EvidenceState::Unknown
    } else {
        questions.status.clone()
    };
    Ok(ResearchReport {
        question_report: questions,
        records: records.len(),
        evidence_state,
        output_identity,
        provenance: plan.reproducibility.clone(),
    })
}
