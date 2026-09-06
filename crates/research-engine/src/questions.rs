//! Blind, bounded question generation for Program R.
//!
//! A request contains only frozen ontology and phenotype metadata. No result,
//! outcome, or winning finding is accepted by this API, which keeps discovery
//! questions independent of prior behavioral evidence.

use crate::role::{DetectorAtlas, RelationshipOperator};
use research_contracts::{
    ensure_causal, ContextPermission, ContractError, EvidenceState, NativeScale, Question,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Positive,
    Negative,
    Bidirectional,
}

impl Direction {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Bidirectional => "bidirectional",
        }
    }
}

/// Frozen metadata from which questions may be generated. It intentionally
/// has no field for findings or outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestionGenerationRequest {
    pub experiment_id: String,
    pub instrument_ids: Vec<String>,
    pub anchor_detector_id: String,
    pub anchor_ids: Vec<String>,
    pub context_detector_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub context_ids: Vec<String>,
    pub native_scales: BTreeSet<NativeScale>,
    pub anchor_time_ns: i64,
    pub context_available_time_ns: BTreeMap<String, i64>,
    pub directions: Vec<Direction>,
    pub max_questions: usize,
}

/// Alias retained for callers that name the frozen input a question spec.
pub type QuestionSpec = QuestionGenerationRequest;

impl QuestionGenerationRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.experiment_id.is_empty() || self.anchor_detector_id.is_empty() {
            return Err(ContractError::Invalid(
                "experiment and anchor detector IDs cannot be empty".into(),
            ));
        }
        if self.instrument_ids.is_empty()
            || self.anchor_ids.is_empty()
            || self.context_detector_ids.is_empty()
            || self.native_scales.is_empty()
            || self.directions.is_empty()
            || self.max_questions == 0
        {
            return Err(ContractError::Invalid(
                "question request has an empty required field".into(),
            ));
        }
        for (name, values) in [
            ("instrument_ids", &self.instrument_ids),
            ("anchor_ids", &self.anchor_ids),
            ("context_detector_ids", &self.context_detector_ids),
            ("context_ids", &self.context_ids),
            ("lifecycle_states", &self.lifecycle_states),
        ] {
            if values.iter().any(String::is_empty) {
                return Err(ContractError::Invalid(format!(
                    "{name} cannot contain empty IDs"
                )));
            }
            let unique: HashSet<&String> = values.iter().collect();
            if unique.len() != values.len() {
                return Err(ContractError::Invalid(format!("{name} must be unique")));
            }
        }
        if self.context_available_time_ns.keys().any(|id| {
            !self
                .context_detector_ids
                .iter()
                .any(|candidate| candidate == id)
        }) {
            return Err(ContractError::Invalid(
                "availability map contains an undeclared context".into(),
            ));
        }
        let mut directions = HashSet::new();
        if self
            .directions
            .iter()
            .any(|direction| !directions.insert(*direction))
        {
            return Err(ContractError::Invalid("directions must be unique".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    PermissionDenied,
    MissingAvailability,
    FutureAvailability {
        anchor_time_ns: i64,
        available_time_ns: i64,
    },
    IncompatibleOperator,
    DerivedLineage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedCandidate {
    pub detector_id: String,
    pub reason: RejectionReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedQuestion {
    pub question: Question,
    pub operator: RelationshipOperator,
    pub direction: Direction,
    pub context_available_time_ns: i64,
    /// Always empty: prior winners are deliberately outside generation input.
    pub prior_winner_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestionGenerationReport {
    pub questions: Vec<GeneratedQuestion>,
    pub rejected: Vec<RejectedCandidate>,
    pub status: EvidenceState,
    pub bounded: bool,
}

impl DetectorAtlas {
    /// Generate a deterministic, role-dispatched and bounded question set.
    pub fn generate_questions(
        &self,
        request: &QuestionGenerationRequest,
        permission: &ContextPermission,
    ) -> Result<QuestionGenerationReport, ContractError> {
        request.validate()?;
        permission.validate()?;
        if permission.consumer != research_contracts::ProgramConsumer::MarketResearch {
            return Err(ContractError::Invalid(
                "question generation requires market-research permission".into(),
            ));
        }
        let anchor = self
            .detector(&request.anchor_detector_id)
            .ok_or_else(|| ContractError::Invalid("unknown anchor detector".into()))?;

        let mut questions = Vec::new();
        let mut rejected = Vec::new();
        let mut accepted_contexts = Vec::new();
        for context_id in &request.context_detector_ids {
            let Some(context) = self.detector(context_id) else {
                return Err(ContractError::Invalid(format!(
                    "unknown detector: {context_id}"
                )));
            };
            if context
                .native_scales
                .is_disjoint(&permission.allowed_scales)
            {
                rejected.push(RejectedCandidate {
                    detector_id: context_id.clone(),
                    reason: RejectionReason::PermissionDenied,
                });
                continue;
            }
            if !permission.allowed_detector_ids.contains(context_id) {
                rejected.push(RejectedCandidate {
                    detector_id: context_id.clone(),
                    reason: RejectionReason::PermissionDenied,
                });
                continue;
            }
            let Some(&available_time_ns) = request.context_available_time_ns.get(context_id) else {
                rejected.push(RejectedCandidate {
                    detector_id: context_id.clone(),
                    reason: RejectionReason::MissingAvailability,
                });
                continue;
            };
            if ensure_causal(request.anchor_time_ns, available_time_ns).is_err() {
                rejected.push(RejectedCandidate {
                    detector_id: context_id.clone(),
                    reason: RejectionReason::FutureAvailability {
                        anchor_time_ns: request.anchor_time_ns,
                        available_time_ns,
                    },
                });
                continue;
            }
            if self.is_derived_from(context_id, &request.anchor_detector_id)
                || self.is_derived_from(&request.anchor_detector_id, context_id)
                || accepted_contexts
                    .iter()
                    .any(|prior: &String| self.shares_ancestor(context_id, prior))
            {
                rejected.push(RejectedCandidate {
                    detector_id: context_id.clone(),
                    reason: RejectionReason::DerivedLineage,
                });
                continue;
            }

            accepted_contexts.push(context_id.clone());

            for operator in RelationshipOperator::ALL {
                let anchor_compatible = anchor
                    .roles
                    .iter()
                    .any(|role| operator.compatible_with(role.clone()));
                let context_compatible = context
                    .roles
                    .iter()
                    .any(|role| operator.compatible_with(role.clone()));
                if !anchor_compatible || !context_compatible {
                    continue;
                }
                for direction in &request.directions {
                    if questions.len() == request.max_questions {
                        break;
                    }
                    let mut roles = anchor.roles.clone();
                    for role in &context.roles {
                        if !roles.contains(role) {
                            roles.push(role.clone());
                        }
                    }
                    let question_id = format!(
                        "q-{}-{}-{}-{}-{}",
                        request.experiment_id,
                        request.anchor_detector_id,
                        context_id,
                        operator.name(),
                        direction.name()
                    );
                    let question = Question {
                        question_id,
                        experiment_id: request.experiment_id.clone(),
                        instrument_ids: request.instrument_ids.clone(),
                        detector_ids: vec![request.anchor_detector_id.clone(), context_id.clone()],
                        roles,
                        anchor_ids: request.anchor_ids.clone(),
                        lifecycle_states: request.lifecycle_states.clone(),
                        context_ids: request.context_ids.clone(),
                        native_scales: request.native_scales.clone(),
                        evidence_state: EvidenceState::Unknown,
                    };
                    question.validate()?;
                    questions.push(GeneratedQuestion {
                        question,
                        operator,
                        direction: *direction,
                        context_available_time_ns: available_time_ns,
                        prior_winner_ids: Vec::new(),
                    });
                }
                if questions.len() == request.max_questions {
                    break;
                }
            }
        }
        let status = if questions.is_empty() {
            EvidenceState::Rejected
        } else if rejected.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Partial
        };
        Ok(QuestionGenerationReport {
            questions,
            rejected,
            status,
            bounded: true,
        })
    }
}

pub fn generate_blind_questions(
    atlas: &DetectorAtlas,
    request: &QuestionGenerationRequest,
    permission: &ContextPermission,
) -> Result<QuestionGenerationReport, ContractError> {
    atlas.generate_questions(request, permission)
}
