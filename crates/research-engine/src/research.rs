//! Bound, synthetic-capable Program R execution over causal tape records.

use crate::{
    DetectorAtlas, Direction, QuestionGenerationReport, QuestionGenerationRequest,
    RelationshipOperator,
};
use research_contracts::{
    CompleteReproducibilityIdentity, ContextPermission, ContractError, EvidenceState, NativeScale,
    ReproducibilityIdentity,
};
use research_tape::{AnchorInstance, ContextObservation};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const REPORT_DOMAIN: &[u8] = b"trinityr-program-r-report-v2\0";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CausalResearchRecord {
    pub instrument_id: String,
    pub scope_identity: String,
    pub source_manifest_identity: String,
    pub anchor: AnchorInstance,
    pub contexts: Vec<ContextObservation>,
}

impl CausalResearchRecord {
    pub fn from_tape(
        instrument_id: impl Into<String>,
        scope_identity: impl Into<String>,
        source_manifest_identity: impl Into<String>,
        anchor: AnchorInstance,
        contexts: Vec<ContextObservation>,
    ) -> Result<Self, ContractError> {
        let record = Self {
            instrument_id: instrument_id.into(),
            scope_identity: scope_identity.into(),
            source_manifest_identity: source_manifest_identity.into(),
            anchor,
            contexts,
        };
        record.validate_causal()?;
        Ok(record)
    }

    fn validate_causal(&self) -> Result<(), ContractError> {
        if self.instrument_id.is_empty()
            || self.scope_identity.is_empty()
            || self.source_manifest_identity.is_empty()
            || self.anchor.anchor_id.is_empty()
            || self.anchor.detector_id.is_empty()
            || self.anchor.source.part.is_empty()
            || self.anchor.anchor_time < 0
            || self.anchor.value.is_some_and(|value| !value.is_finite())
            || self.contexts.iter().any(|context| {
                context.detector_id.is_empty()
                    || context.source.part.is_empty()
                    || context.available_time < 0
                    || context.available_time > self.anchor.anchor_time
                    || context.value.is_some_and(|value| !value.is_finite())
            })
        {
            return Err(ContractError::Invalid(
                "invalid causal research record".into(),
            ));
        }
        Ok(())
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
    pub native_scale: NativeScale,
    pub lifecycle_state: String,
    pub samples: usize,
    pub mean: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiscoveryResult {
    pub question_id: String,
    pub anchor_detector_id: String,
    pub context_detector_id: String,
    pub operator: RelationshipOperator,
    pub direction: Direction,
    pub native_scale: NativeScale,
    pub paired_observations: usize,
    pub correlation: Option<f64>,
    pub mean_lag_ns: Option<f64>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CandidateFormation {
    pub candidate_ids: Vec<String>,
    pub source_discovery_ids: Vec<String>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Characterization {
    pub candidate_id: String,
    pub discovery_identity: String,
    pub sample_count: usize,
    pub concordant_count: usize,
    pub discordant_count: usize,
    pub minimum_lag_ns: Option<i64>,
    pub maximum_lag_ns: Option<i64>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InformationSystemReport {
    pub instrument_ids: Vec<String>,
    pub source_manifest_identity: String,
    pub data_scope_identity: String,
    pub source_identities: Vec<String>,
    pub native_scales: BTreeSet<NativeScale>,
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
    pub report_version: u16,
    pub question_report: QuestionGenerationReport,
    pub records: usize,
    pub evidence_state: EvidenceState,
    pub output_identity: String,
    pub provenance: ReproducibilityIdentity,
    pub complete_provenance: CompleteReproducibilityIdentity,
    pub native_phenotypes: Vec<NativePhenotype>,
    pub same_domain_discovery: Vec<DiscoveryResult>,
    pub cross_domain_discovery: Vec<DiscoveryResult>,
    pub candidates: CandidateFormation,
    pub characterization: Vec<Characterization>,
    pub information_system: InformationSystemReport,
    pub knowledge_records: Vec<ResearchKnowledgeRecord>,
}

impl ResearchReport {
    pub fn validate_identity(&self) -> Result<(), ContractError> {
        if self.report_version != 2 || report_identity(self)? != self.output_identity {
            return Err(ContractError::Invalid(
                "Program R report identity mismatch".into(),
            ));
        }
        Ok(())
    }
}

pub fn run_research(
    plan: &ResearchPlan,
    records: &[CausalResearchRecord],
) -> Result<ResearchReport, ContractError> {
    plan.reproducibility.validate()?;
    plan.complete_reproducibility.validate()?;
    plan.request.validate()?;
    plan.permission.validate()?;
    if plan.max_records == 0 || records.len() > plan.max_records {
        return Err(ContractError::Invalid("research record bound".into()));
    }
    validate_records(plan, records)?;

    let questions = plan
        .atlas
        .generate_questions(&plan.request, &plan.permission)?;
    let native_phenotypes = phenotypes(records);
    let discoveries = questions
        .questions
        .iter()
        .map(|question| discover(question, records))
        .collect::<Vec<_>>();
    let (same_domain_discovery, cross_domain_discovery): (Vec<_>, Vec<_>) =
        discoveries.into_iter().partition(|result| {
            records.iter().any(|record| {
                record.anchor.native_scale == result.native_scale
                    && record.contexts.iter().any(|context| {
                        context.detector_id == result.context_detector_id
                            && context.native_scale == record.anchor.native_scale
                    })
            })
        });
    let all_discoveries = same_domain_discovery
        .iter()
        .chain(cross_domain_discovery.iter())
        .collect::<Vec<_>>();
    let source_discovery_ids = all_discoveries
        .iter()
        .filter(|result| result.correlation.is_some())
        .map(|result| result.identity.clone())
        .collect::<Vec<_>>();
    let candidate_ids = source_discovery_ids
        .iter()
        .map(|identity| format!("candidate-{}", &identity[..16]))
        .collect::<Vec<_>>();
    let candidates = CandidateFormation {
        candidate_ids: candidate_ids.clone(),
        source_discovery_ids: source_discovery_ids.clone(),
        identity: hash_json(&(candidate_ids.clone(), source_discovery_ids))?,
        status: if candidate_ids.is_empty() {
            EvidenceState::Inconclusive
        } else {
            EvidenceState::Partial
        },
    };
    let characterization = candidate_ids
        .iter()
        .zip(
            all_discoveries
                .iter()
                .filter(|result| result.correlation.is_some()),
        )
        .map(|(candidate_id, result)| characterize(candidate_id, result, records))
        .collect::<Result<Vec<_>, _>>()?;
    let information_system = information_system(plan, records)?;
    let knowledge_records = all_discoveries
        .iter()
        .map(|result| ResearchKnowledgeRecord {
            record_id: format!("knowledge-{}", result.identity),
            source_identity: information_system.identity.clone(),
            stage_identity: result.identity.clone(),
            status: result.status.clone(),
        })
        .collect::<Vec<_>>();
    let evidence_state = if records.is_empty() {
        EvidenceState::Unknown
    } else if all_discoveries
        .iter()
        .any(|result| result.correlation.is_some())
    {
        EvidenceState::Partial
    } else {
        EvidenceState::Inconclusive
    };
    let mut report = ResearchReport {
        report_version: 2,
        question_report: questions,
        records: records.len(),
        evidence_state,
        output_identity: String::new(),
        provenance: plan.reproducibility.clone(),
        complete_provenance: plan.complete_reproducibility.clone(),
        native_phenotypes,
        same_domain_discovery,
        cross_domain_discovery,
        candidates,
        characterization,
        information_system,
        knowledge_records,
    };
    report.output_identity = report_identity(&report)?;
    Ok(report)
}

fn validate_records(
    plan: &ResearchPlan,
    records: &[CausalResearchRecord],
) -> Result<(), ContractError> {
    let expected_contexts = plan
        .request
        .context_detector_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let anchor = plan
        .atlas
        .detector(&plan.request.anchor_detector_id)
        .ok_or_else(|| ContractError::Invalid("unknown anchor detector".into()))?;
    let mut previous = None;
    for record in records {
        record.validate_causal()?;
        let key = (
            record.instrument_id.as_str(),
            record.anchor.anchor_time,
            record.anchor.source.part.as_str(),
            record.anchor.source.row_index,
        );
        if previous.is_some_and(|prior| prior >= key) {
            return Err(ContractError::Invalid(
                "research records must be canonical and unique".into(),
            ));
        }
        previous = Some(key);
        if !plan.request.instrument_ids.contains(&record.instrument_id)
            || record.scope_identity != plan.complete_reproducibility.data_scope_identity
            || record.source_manifest_identity
                != plan.complete_reproducibility.source_manifest_identity
            || record.anchor.detector_id != plan.request.anchor_detector_id
            || !plan.request.anchor_ids.contains(&record.anchor.anchor_id)
            || !plan
                .request
                .lifecycle_states
                .contains(&record.anchor.lifecycle_state)
            || record.anchor.anchor_time != plan.request.anchor_time_ns
            || !plan
                .request
                .native_scales
                .contains(&record.anchor.native_scale)
            || !anchor.native_scales.contains(&record.anchor.native_scale)
        {
            return Err(ContractError::Invalid(
                "research record does not match frozen plan".into(),
            ));
        }
        let actual_contexts = record
            .contexts
            .iter()
            .map(|context| context.detector_id.clone())
            .collect::<BTreeSet<_>>();
        if actual_contexts != expected_contexts || actual_contexts.len() != record.contexts.len() {
            return Err(ContractError::Invalid(
                "research record context set mismatch".into(),
            ));
        }
        for context in &record.contexts {
            let descriptor = plan
                .atlas
                .detector(&context.detector_id)
                .ok_or_else(|| ContractError::Invalid("unknown context detector".into()))?;
            if !plan.request.native_scales.contains(&context.native_scale)
                || !descriptor.native_scales.contains(&context.native_scale)
                || plan
                    .request
                    .context_available_time_ns
                    .get(&context.detector_id)
                    != Some(&context.available_time)
            {
                return Err(ContractError::Invalid(
                    "research context does not match frozen plan".into(),
                ));
            }
        }
    }
    Ok(())
}

fn phenotypes(records: &[CausalResearchRecord]) -> Vec<NativePhenotype> {
    let mut groups: BTreeMap<(NativeScale, String), Vec<f64>> = BTreeMap::new();
    for record in records {
        if let Some(value) = record.anchor.value {
            groups
                .entry((
                    record.anchor.native_scale.clone(),
                    record.anchor.lifecycle_state.clone(),
                ))
                .or_default()
                .push(value);
        }
    }
    groups
        .into_iter()
        .map(
            |((native_scale, lifecycle_state), values)| NativePhenotype {
                native_scale,
                lifecycle_state,
                samples: values.len(),
                mean: Some(values.iter().sum::<f64>() / values.len() as f64),
                minimum: values.iter().copied().reduce(f64::min),
                maximum: values.iter().copied().reduce(f64::max),
                identity: hash_json(&values).expect("finite values serialize"),
                status: EvidenceState::Known,
            },
        )
        .collect()
}

fn discover(
    question: &crate::GeneratedQuestion,
    records: &[CausalResearchRecord],
) -> DiscoveryResult {
    let context_id = &question.question.detector_ids[1];
    let pairs = records
        .iter()
        .filter_map(|record| {
            let context = record
                .contexts
                .iter()
                .find(|context| &context.detector_id == context_id)?;
            Some((
                record.anchor.value?,
                context.value?,
                record.anchor.anchor_time - context.available_time,
            ))
        })
        .collect::<Vec<_>>();
    let correlation = pearson(&pairs);
    let mean_lag_ns = (!pairs.is_empty())
        .then(|| pairs.iter().map(|pair| pair.2 as f64).sum::<f64>() / pairs.len() as f64);
    let native_scale = question
        .question
        .native_scales
        .iter()
        .next()
        .cloned()
        .expect("question scale");
    let identity = hash_json(&(
        &question.question.question_id,
        question.operator,
        question.direction,
        &native_scale,
        &pairs,
        correlation,
        mean_lag_ns,
    ))
    .expect("finite observations serialize");
    DiscoveryResult {
        question_id: question.question.question_id.clone(),
        anchor_detector_id: question.question.detector_ids[0].clone(),
        context_detector_id: context_id.clone(),
        operator: question.operator,
        direction: question.direction,
        native_scale,
        paired_observations: pairs.len(),
        correlation,
        mean_lag_ns,
        identity,
        status: if pairs.is_empty() {
            EvidenceState::Unknown
        } else if correlation.is_none() {
            EvidenceState::Inconclusive
        } else {
            EvidenceState::Partial
        },
    }
}

fn pearson(pairs: &[(f64, f64, i64)]) -> Option<f64> {
    if pairs.len() < 2 {
        return None;
    }
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|pair| pair.0).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|pair| pair.1).sum::<f64>() / n;
    let covariance = pairs
        .iter()
        .map(|pair| (pair.0 - mean_x) * (pair.1 - mean_y))
        .sum::<f64>();
    let variance_x = pairs
        .iter()
        .map(|pair| (pair.0 - mean_x).powi(2))
        .sum::<f64>();
    let variance_y = pairs
        .iter()
        .map(|pair| (pair.1 - mean_y).powi(2))
        .sum::<f64>();
    let denominator = (variance_x * variance_y).sqrt();
    (denominator > 0.0).then_some(covariance / denominator)
}

fn characterize(
    candidate_id: &str,
    discovery: &DiscoveryResult,
    records: &[CausalResearchRecord],
) -> Result<Characterization, ContractError> {
    let observations = records
        .iter()
        .filter_map(|record| {
            let context = record
                .contexts
                .iter()
                .find(|context| context.detector_id == discovery.context_detector_id)?;
            Some((
                record.anchor.value?,
                context.value?,
                record.anchor.anchor_time - context.available_time,
            ))
        })
        .collect::<Vec<_>>();
    let concordant_count = observations
        .iter()
        .filter(|(anchor, context, _)| match discovery.direction {
            Direction::Positive => anchor * context > 0.0,
            Direction::Negative => anchor * context < 0.0,
            Direction::Bidirectional => anchor * context != 0.0,
        })
        .count();
    let discordant_count = observations.len() - concordant_count;
    let minimum_lag_ns = observations.iter().map(|observation| observation.2).min();
    let maximum_lag_ns = observations.iter().map(|observation| observation.2).max();
    let identity = hash_json(&(
        candidate_id,
        &discovery.identity,
        concordant_count,
        discordant_count,
        minimum_lag_ns,
        maximum_lag_ns,
    ))?;
    Ok(Characterization {
        candidate_id: candidate_id.into(),
        discovery_identity: discovery.identity.clone(),
        sample_count: observations.len(),
        concordant_count,
        discordant_count,
        minimum_lag_ns,
        maximum_lag_ns,
        identity,
        status: EvidenceState::Partial,
    })
}

fn information_system(
    plan: &ResearchPlan,
    records: &[CausalResearchRecord],
) -> Result<InformationSystemReport, ContractError> {
    let instrument_ids = records
        .iter()
        .map(|record| record.instrument_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let source_identities = records
        .iter()
        .map(|record| {
            format!(
                "{}:{}",
                record.anchor.source.part, record.anchor.source.row_index
            )
        })
        .collect::<Vec<_>>();
    let native_scales = records
        .iter()
        .flat_map(|record| {
            std::iter::once(record.anchor.native_scale.clone()).chain(
                record
                    .contexts
                    .iter()
                    .map(|context| context.native_scale.clone()),
            )
        })
        .collect::<BTreeSet<_>>();
    let scope_start_ns = records.iter().map(|record| record.anchor.anchor_time).min();
    let scope_end_ns = records.iter().map(|record| record.anchor.anchor_time).max();
    let identity = hash_json(&(
        &instrument_ids,
        &plan.complete_reproducibility.source_manifest_identity,
        &plan.complete_reproducibility.data_scope_identity,
        &source_identities,
        &native_scales,
        scope_start_ns,
        scope_end_ns,
    ))?;
    Ok(InformationSystemReport {
        instrument_ids,
        source_manifest_identity: plan
            .complete_reproducibility
            .source_manifest_identity
            .clone(),
        data_scope_identity: plan.complete_reproducibility.data_scope_identity.clone(),
        source_identities,
        native_scales,
        scope_start_ns,
        scope_end_ns,
        identity,
        status: if records.is_empty() {
            EvidenceState::Unknown
        } else {
            EvidenceState::Known
        },
    })
}

fn report_identity(report: &ResearchReport) -> Result<String, ContractError> {
    hash_json(&(
        report.report_version,
        &report.question_report,
        report.records,
        &report.evidence_state,
        &report.provenance,
        &report.complete_provenance,
        &report.native_phenotypes,
        &report.same_domain_discovery,
        &report.cross_domain_discovery,
        &report.candidates,
        &report.characterization,
        &report.information_system,
        &report.knowledge_records,
    ))
}

fn hash_json<T: Serialize + ?Sized>(value: &T) -> Result<String, ContractError> {
    let mut hasher = Sha256::new();
    hasher.update(REPORT_DOMAIN);
    hasher.update(
        serde_json::to_vec(value).map_err(|error| ContractError::Invalid(error.to_string()))?,
    );
    Ok(format!("{:x}", hasher.finalize()))
}
