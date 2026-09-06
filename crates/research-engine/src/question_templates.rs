//! Outcome-blind question-template generation for Program R.
//!
//! Templates are generated from frozen semantic metadata only. Unlike the
//! legacy question generator, this API has no anchor timestamp or observed
//! context-availability values: causal availability is checked later for every
//! anchor instance by the execution layer. Anchor and context native scales are
//! explicit, so cross-scale E3 questions do not require a false scale
//! intersection.

use crate::{DetectorAtlas, DetectorDescriptor, DetectorDescriptorInput, RelationshipOperator};
use research_contracts::{
    ContextPermission, ContractError, DetectorRole, EvidenceState, NativeScale, ProgramConsumer,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};

const TEMPLATE_DOMAIN: &[u8] = b"trinityr-question-template-v1\0";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TemplateDirection {
    Positive,
    Negative,
    Bidirectional,
}

impl TemplateDirection {
    fn name(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Bidirectional => "bidirectional",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuestionTemplateRequest {
    pub experiment_id: String,
    pub instrument_ids: Vec<String>,
    pub phenotype_identity: String,
    pub anchor_detector_id: String,
    pub anchor_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub context_detector_ids: Vec<String>,
    pub anchor_scales: BTreeSet<NativeScale>,
    pub context_scales: BTreeSet<NativeScale>,
    pub directions: Vec<TemplateDirection>,
    pub max_questions: usize,
    pub detectors: Vec<DetectorDescriptorInput>,
    pub permission: ContextPermission,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TemplateRejectionReason {
    PermissionDenied,
    NoAllowedAnchorScale,
    NoAllowedContextScale,
    IncompatibleOperator,
    LineageDependent,
    BoundedOut,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RejectedQuestionTemplate {
    pub context_detector_id: String,
    pub operator: Option<String>,
    pub reason: TemplateRejectionReason,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuestionTemplate {
    pub question_id: String,
    pub experiment_id: String,
    pub instrument_ids: Vec<String>,
    pub phenotype_identity: String,
    pub anchor_detector_id: String,
    pub context_detector_id: String,
    pub anchor_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub anchor_roles: Vec<DetectorRole>,
    pub context_roles: Vec<DetectorRole>,
    pub operator: String,
    pub direction: TemplateDirection,
    pub anchor_scale: NativeScale,
    pub context_scale: NativeScale,
    pub same_native_scale: bool,
    pub evidence_state: EvidenceState,
    pub template_identity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuestionTemplateReport {
    pub report_version: u16,
    pub questions: Vec<QuestionTemplate>,
    pub rejected: Vec<RejectedQuestionTemplate>,
    pub bounded: bool,
    pub report_identity: String,
    pub status: EvidenceState,
}

impl QuestionTemplateReport {
    pub fn validate_identity(&self) -> Result<(), ContractError> {
        if self.report_version != 1 || self.report_identity != report_identity(self)? {
            return Err(ContractError::Invalid(
                "question-template report identity mismatch".into(),
            ));
        }
        Ok(())
    }
}

pub fn generate_question_templates(
    request: &QuestionTemplateRequest,
) -> Result<QuestionTemplateReport, ContractError> {
    validate_request(request)?;
    let atlas = DetectorAtlas::new(
        request
            .detectors
            .iter()
            .map(build_descriptor)
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    let anchor = atlas
        .detector(&request.anchor_detector_id)
        .ok_or_else(|| ContractError::Invalid("unknown anchor detector".into()))?;

    let anchor_scales = request
        .anchor_scales
        .intersection(&anchor.native_scales)
        .cloned()
        .collect::<BTreeSet<_>>();
    if anchor_scales.is_empty() {
        return Err(ContractError::Invalid(
            "question-template request has no valid anchor scale".into(),
        ));
    }

    let mut questions = Vec::new();
    let mut rejected = Vec::new();
    let mut bounded = false;

    'contexts: for context_id in &request.context_detector_ids {
        let context = atlas
            .detector(context_id)
            .ok_or_else(|| ContractError::Invalid(format!("unknown detector: {context_id}")))?;
        if !request.permission.allowed_detector_ids.contains(context_id) {
            rejected.push(RejectedQuestionTemplate {
                context_detector_id: context_id.clone(),
                operator: None,
                reason: TemplateRejectionReason::PermissionDenied,
            });
            continue;
        }
        let context_scales = request
            .context_scales
            .intersection(&context.native_scales)
            .filter(|scale| request.permission.allowed_scales.contains(*scale))
            .cloned()
            .collect::<BTreeSet<_>>();
        if context_scales.is_empty() {
            rejected.push(RejectedQuestionTemplate {
                context_detector_id: context_id.clone(),
                operator: None,
                reason: TemplateRejectionReason::NoAllowedContextScale,
            });
            continue;
        }
        if atlas.is_derived_from(context_id, &request.anchor_detector_id)
            || atlas.is_derived_from(&request.anchor_detector_id, context_id)
            || atlas.shares_ancestor(context_id, &request.anchor_detector_id)
        {
            rejected.push(RejectedQuestionTemplate {
                context_detector_id: context_id.clone(),
                operator: None,
                reason: TemplateRejectionReason::LineageDependent,
            });
            continue;
        }

        let mut any_operator = false;
        for operator in RelationshipOperator::ALL {
            if !operator_compatible(anchor, context, operator) {
                continue;
            }
            any_operator = true;
            for anchor_scale in &anchor_scales {
                for context_scale in &context_scales {
                    for direction in &request.directions {
                        if !operator.allows_direction(direction.name()) {
                            continue;
                        }
                        if questions.len() == request.max_questions {
                            bounded = true;
                            rejected.push(RejectedQuestionTemplate {
                                context_detector_id: context_id.clone(),
                                operator: Some(operator.name().into()),
                                reason: TemplateRejectionReason::BoundedOut,
                            });
                            break 'contexts;
                        }
                        let question_id = format!(
                            "q-{}-{}-{}-{}-{}-{}-{}",
                            request.experiment_id,
                            request.anchor_detector_id,
                            context_id,
                            operator.name(),
                            direction.name(),
                            scale_token(anchor_scale),
                            scale_token(context_scale)
                        );
                        let template_identity = template_identity(&(
                            &question_id,
                            &request.phenotype_identity,
                            &request.instrument_ids,
                            &request.anchor_ids,
                            &request.lifecycle_states,
                            &anchor.roles,
                            &context.roles,
                            operator.name(),
                            direction,
                            anchor_scale,
                            context_scale,
                        ))?;
                        questions.push(QuestionTemplate {
                            question_id,
                            experiment_id: request.experiment_id.clone(),
                            instrument_ids: request.instrument_ids.clone(),
                            phenotype_identity: request.phenotype_identity.clone(),
                            anchor_detector_id: request.anchor_detector_id.clone(),
                            context_detector_id: context_id.clone(),
                            anchor_ids: request.anchor_ids.clone(),
                            lifecycle_states: request.lifecycle_states.clone(),
                            anchor_roles: anchor.roles.clone(),
                            context_roles: context.roles.clone(),
                            operator: operator.name().into(),
                            direction: *direction,
                            anchor_scale: anchor_scale.clone(),
                            context_scale: context_scale.clone(),
                            same_native_scale: anchor_scale == context_scale,
                            evidence_state: EvidenceState::Unknown,
                            template_identity,
                        });
                    }
                }
            }
        }
        if !any_operator {
            rejected.push(RejectedQuestionTemplate {
                context_detector_id: context_id.clone(),
                operator: None,
                reason: TemplateRejectionReason::IncompatibleOperator,
            });
        }
    }

    let status = if questions.is_empty() {
        EvidenceState::Rejected
    } else if rejected.is_empty() {
        EvidenceState::Unknown
    } else {
        EvidenceState::Partial
    };
    let mut report = QuestionTemplateReport {
        report_version: 1,
        questions,
        rejected,
        bounded,
        report_identity: String::new(),
        status,
    };
    report.report_identity = report_identity(&report)?;
    Ok(report)
}

fn validate_request(request: &QuestionTemplateRequest) -> Result<(), ContractError> {
    request.permission.validate()?;
    if request.permission.consumer != ProgramConsumer::MarketResearch
        || request.experiment_id.is_empty()
        || request.phenotype_identity.is_empty()
        || request.anchor_detector_id.is_empty()
        || request.instrument_ids.is_empty()
        || request.anchor_ids.is_empty()
        || request.context_detector_ids.is_empty()
        || request.anchor_scales.is_empty()
        || request.context_scales.is_empty()
        || request.directions.is_empty()
        || request.max_questions == 0
        || request.detectors.is_empty()
    {
        return Err(ContractError::Invalid(
            "invalid question-template request".into(),
        ));
    }
    for values in [
        &request.instrument_ids,
        &request.anchor_ids,
        &request.lifecycle_states,
        &request.context_detector_ids,
    ] {
        let unique = values.iter().collect::<HashSet<_>>();
        if unique.len() != values.len() || values.iter().any(String::is_empty) {
            return Err(ContractError::Invalid(
                "question-template IDs must be unique and nonempty".into(),
            ));
        }
    }
    let unique_directions = request.directions.iter().collect::<HashSet<_>>();
    if unique_directions.len() != request.directions.len() {
        return Err(ContractError::Invalid(
            "question-template directions must be unique".into(),
        ));
    }
    Ok(())
}

fn build_descriptor(input: &DetectorDescriptorInput) -> Result<DetectorDescriptor, ContractError> {
    DetectorDescriptor::new(
        input.detector_id.clone(),
        input.roles.clone(),
        input.native_scales.clone(),
    )?
    .with_lineage(input.derives_from.clone())
}

fn operator_compatible(
    anchor: &DetectorDescriptor,
    context: &DetectorDescriptor,
    operator: RelationshipOperator,
) -> bool {
    anchor
        .roles
        .iter()
        .any(|role| operator.compatible_with(role.clone()))
        && context
            .roles
            .iter()
            .any(|role| operator.compatible_with(role.clone()))
}

fn scale_token(scale: &NativeScale) -> &'static str {
    match scale {
        NativeScale::Tick => "tick",
        NativeScale::Bar(research_contracts::BarScale::S15) => "15s",
        NativeScale::Bar(research_contracts::BarScale::S30) => "30s",
        NativeScale::Bar(research_contracts::BarScale::M1) => "1m",
        NativeScale::Bar(research_contracts::BarScale::M5) => "5m",
        NativeScale::Bar(research_contracts::BarScale::M15) => "15m",
        NativeScale::Bar(research_contracts::BarScale::H1) => "1h",
        NativeScale::Bar(research_contracts::BarScale::H4) => "4h",
    }
}

fn template_identity<T: Serialize + ?Sized>(value: &T) -> Result<String, ContractError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| ContractError::Invalid(error.to_string()))?;
    let mut hash = Sha256::new();
    hash.update(TEMPLATE_DOMAIN);
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

fn report_identity(report: &QuestionTemplateReport) -> Result<String, ContractError> {
    template_identity(&(
        report.report_version,
        &report.questions,
        &report.rejected,
        report.bounded,
        &report.status,
    ))
}
