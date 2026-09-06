//! Canonical population-level Program R execution.
//!
//! The older `research` module is retained as a bounded synthetic compatibility
//! path. This module is the production-facing generic runner: an experiment is a
//! population of anchor instances across time, context is checked per anchor,
//! native anchor/context scales are preserved separately, and candidate
//! promotion requires an explicit pre-declared gate.

use crate::{DetectorAtlas, DetectorDescriptor, RelationshipOperator};
use research_contracts::{
    CompleteReproducibilityIdentity, ContextPermission, ContractError, DetectorRole,
    EvidenceState, NativeScale, ProgramConsumer, ReproducibilityIdentity,
};
use research_tape::{AnchorInstance, ContextObservation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const REPORT_DOMAIN: &[u8] = b"trinityr-population-r-report-v1\0";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DetectorDescriptorInput {
    pub detector_id: String,
    pub roles: Vec<DetectorRole>,
    pub native_scales: BTreeSet<NativeScale>,
    #[serde(default)]
    pub derives_from: Vec<String>,
}

impl DetectorDescriptorInput {
    fn build(&self) -> Result<DetectorDescriptor, ContractError> {
        DetectorDescriptor::new(
            self.detector_id.clone(),
            self.roles.clone(),
            self.native_scales.clone(),
        )?
        .with_lineage(self.derives_from.clone())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricKind {
    Pearson,
    DirectionalConcordance,
    MeanLagNs,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CandidateGate {
    pub min_support: usize,
    pub min_abs_metric: f64,
}

impl CandidateGate {
    fn validate(&self) -> Result<(), ContractError> {
        if self.min_support == 0
            || !self.min_abs_metric.is_finite()
            || self.min_abs_metric < 0.0
        {
            return Err(ContractError::Invalid(
                "invalid explicit candidate gate".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricRequest {
    pub question_id: String,
    pub context_detector_id: String,
    pub operator: String,
    pub anchor_scale: NativeScale,
    pub context_scale: NativeScale,
    pub metric: MetricKind,
    pub candidate_gate: Option<CandidateGate>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PopulationRecordInput {
    pub instrument_id: String,
    pub scope_identity: String,
    pub source_manifest_identity: String,
    pub anchor: AnchorInstance,
    pub contexts: Vec<ContextObservation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PopulationRunInput {
    pub detectors: Vec<DetectorDescriptorInput>,
    pub permission: ContextPermission,
    pub reproducibility: ReproducibilityIdentity,
    pub complete_reproducibility: CompleteReproducibilityIdentity,
    pub instrument_ids: Vec<String>,
    pub anchor_detector_id: String,
    pub anchor_ids: Vec<String>,
    pub lifecycle_states: Vec<String>,
    pub metric_requests: Vec<MetricRequest>,
    pub max_records: usize,
    pub records: Vec<PopulationRecordInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PopulationPhenotype {
    pub native_scale: NativeScale,
    pub lifecycle_state: String,
    pub samples: usize,
    pub mean: Option<f64>,
    pub minimum: Option<f64>,
    pub p10: Option<f64>,
    pub median: Option<f64>,
    pub p90: Option<f64>,
    pub maximum: Option<f64>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricDiscovery {
    pub question_id: String,
    pub context_detector_id: String,
    pub operator: String,
    pub metric: MetricKind,
    pub anchor_scale: NativeScale,
    pub context_scale: NativeScale,
    pub same_native_scale: bool,
    pub observations: usize,
    pub value: Option<f64>,
    pub mean_lag_ns: Option<f64>,
    pub identity: String,
    pub status: EvidenceState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CandidateEvaluation {
    pub question_id: String,
    pub discovery_identity: String,
    pub gate: Option<CandidateGate>,
    pub eligible: bool,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PopulationResearchReport {
    pub report_version: u16,
    pub records: usize,
    pub anchor_instances: usize,
    pub scope_start_ns: Option<i64>,
    pub scope_end_ns: Option<i64>,
    pub phenotypes: Vec<PopulationPhenotype>,
    pub discoveries: Vec<MetricDiscovery>,
    pub candidate_evaluations: Vec<CandidateEvaluation>,
    pub candidate_ids: Vec<String>,
    pub provenance: ReproducibilityIdentity,
    pub complete_provenance: CompleteReproducibilityIdentity,
    pub output_identity: String,
    pub status: EvidenceState,
}

impl PopulationResearchReport {
    pub fn validate_identity(&self) -> Result<(), ContractError> {
        if self.report_version != 1 || self.output_identity != report_identity(self)? {
            return Err(ContractError::Invalid(
                "population Program R report identity mismatch".into(),
            ));
        }
        Ok(())
    }
}

pub fn run_population_research(
    input: &PopulationRunInput,
) -> Result<PopulationResearchReport, ContractError> {
    input.permission.validate()?;
    input.reproducibility.validate()?;
    input.complete_reproducibility.validate()?;
    if input.permission.consumer != ProgramConsumer::MarketResearch {
        return Err(ContractError::Invalid(
            "population runner requires market-research permission".into(),
        ));
    }
    if input.max_records == 0
        || input.records.len() > input.max_records
        || input.detectors.is_empty()
        || input.instrument_ids.is_empty()
        || input.anchor_detector_id.is_empty()
        || input.anchor_ids.is_empty()
        || input.metric_requests.is_empty()
    {
        return Err(ContractError::Invalid(
            "population runner has an empty or exceeded bound".into(),
        ));
    }
    if has_duplicates(&input.instrument_ids)
        || has_duplicates(&input.anchor_ids)
        || has_duplicates(&input.lifecycle_states)
    {
        return Err(ContractError::Invalid(
            "population identities must be unique".into(),
        ));
    }

    let atlas = DetectorAtlas::new(
        input
            .detectors
            .iter()
            .map(DetectorDescriptorInput::build)
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    validate_metric_requests(input, &atlas)?;
    validate_records(input, &atlas)?;

    let phenotypes = phenotype_summary(&input.records)?;
    let discoveries = input
        .metric_requests
        .iter()
        .map(|request| discover(request, &input.records))
        .collect::<Result<Vec<_>, _>>()?;
    let candidate_evaluations = input
        .metric_requests
        .iter()
        .zip(&discoveries)
        .map(|(request, discovery)| evaluate_candidate(request, discovery))
        .collect::<Result<Vec<_>, _>>()?;
    let candidate_ids = candidate_evaluations
        .iter()
        .filter(|candidate| candidate.eligible)
        .map(|candidate| format!("candidate-{}", &candidate.discovery_identity[..16]))
        .collect::<Vec<_>>();

    let scope_start_ns = input.records.iter().map(|record| record.anchor.anchor_time).min();
    let scope_end_ns = input.records.iter().map(|record| record.anchor.anchor_time).max();
    let status = if input.records.is_empty() {
        EvidenceState::Unknown
    } else if discoveries
        .iter()
        .all(|discovery| discovery.status == EvidenceState::Inconclusive)
    {
        EvidenceState::Inconclusive
    } else {
        EvidenceState::Partial
    };

    let mut report = PopulationResearchReport {
        report_version: 1,
        records: input.records.len(),
        anchor_instances: input.records.len(),
        scope_start_ns,
        scope_end_ns,
        phenotypes,
        discoveries,
        candidate_evaluations,
        candidate_ids,
        provenance: input.reproducibility.clone(),
        complete_provenance: input.complete_reproducibility.clone(),
        output_identity: String::new(),
        status,
    };
    report.output_identity = report_identity(&report)?;
    Ok(report)
}

fn validate_metric_requests(
    input: &PopulationRunInput,
    atlas: &DetectorAtlas,
) -> Result<(), ContractError> {
    let anchor = atlas
        .detector(&input.anchor_detector_id)
        .ok_or_else(|| ContractError::Invalid("unknown anchor detector".into()))?;
    let mut question_ids = BTreeSet::new();
    for request in &input.metric_requests {
        if request.question_id.is_empty()
            || request.context_detector_id.is_empty()
            || !question_ids.insert(request.question_id.clone())
        {
            return Err(ContractError::Invalid(
                "metric request identities must be unique and nonempty".into(),
            ));
        }
        request
            .candidate_gate
            .as_ref()
            .map(CandidateGate::validate)
            .transpose()?;
        let operator = parse_operator(&request.operator)
            .ok_or_else(|| ContractError::Invalid("unknown relationship operator".into()))?;
        let context = atlas
            .detector(&request.context_detector_id)
            .ok_or_else(|| ContractError::Invalid("unknown context detector".into()))?;
        if !anchor.native_scales.contains(&request.anchor_scale)
            || !context.native_scales.contains(&request.context_scale)
            || !input.permission.allowed_scales.contains(&request.context_scale)
            || !input
                .permission
                .allowed_detector_ids
                .contains(&request.context_detector_id)
            || !anchor
                .roles
                .iter()
                .any(|role| operator.compatible_with(role.clone()))
            || !context
                .roles
                .iter()
                .any(|role| operator.compatible_with(role.clone()))
        {
            return Err(ContractError::Invalid(
                "metric request violates ontology or permission".into(),
            ));
        }
        if atlas.is_derived_from(&request.context_detector_id, &input.anchor_detector_id)
            || atlas.is_derived_from(&input.anchor_detector_id, &request.context_detector_id)
            || atlas.shares_ancestor(&request.context_detector_id, &input.anchor_detector_id)
        {
            return Err(ContractError::Invalid(
                "metric request treats dependent lineage as independent context".into(),
            ));
        }
    }
    Ok(())
}

fn validate_records(
    input: &PopulationRunInput,
    atlas: &DetectorAtlas,
) -> Result<(), ContractError> {
    let expected_contexts = input
        .metric_requests
        .iter()
        .map(|request| request.context_detector_id.clone())
        .collect::<BTreeSet<_>>();
    let anchor = atlas
        .detector(&input.anchor_detector_id)
        .ok_or_else(|| ContractError::Invalid("unknown anchor detector".into()))?;
    let mut previous: Option<(String, i64, String, u64)> = None;

    for record in &input.records {
        if record.instrument_id.is_empty()
            || record.scope_identity.is_empty()
            || record.source_manifest_identity.is_empty()
            || !input.instrument_ids.contains(&record.instrument_id)
            || record.scope_identity != input.complete_reproducibility.data_scope_identity
            || record.source_manifest_identity
                != input.complete_reproducibility.source_manifest_identity
            || record.anchor.detector_id != input.anchor_detector_id
            || !input.anchor_ids.contains(&record.anchor.anchor_id)
            || (!input.lifecycle_states.is_empty()
                && !input
                    .lifecycle_states
                    .contains(&record.anchor.lifecycle_state))
            || !anchor.native_scales.contains(&record.anchor.native_scale)
            || record.anchor.anchor_time < 0
            || record.anchor.source.part.is_empty()
            || record
                .anchor
                .value
                .is_some_and(|value| !value.is_finite())
        {
            return Err(ContractError::Invalid(
                "population record does not match frozen experiment".into(),
            ));
        }

        let key = (
            record.instrument_id.clone(),
            record.anchor.anchor_time,
            record.anchor.source.part.clone(),
            record.anchor.source.row_index,
        );
        if previous.as_ref().is_some_and(|prior| prior >= &key) {
            return Err(ContractError::Invalid(
                "population records must be canonical and unique".into(),
            ));
        }
        previous = Some(key);

        let actual_contexts = record
            .contexts
            .iter()
            .map(|context| context.detector_id.clone())
            .collect::<BTreeSet<_>>();
        if actual_contexts != expected_contexts || actual_contexts.len() != record.contexts.len() {
            return Err(ContractError::Invalid(
                "population record context set mismatch".into(),
            ));
        }

        for context in &record.contexts {
            let descriptor = atlas
                .detector(&context.detector_id)
                .ok_or_else(|| ContractError::Invalid("unknown context detector".into()))?;
            if context.source.part.is_empty()
                || context.available_time < 0
                || context.available_time > record.anchor.anchor_time
                || context.value.is_some_and(|value| !value.is_finite())
                || !descriptor.native_scales.contains(&context.native_scale)
                || !input.permission.allowed_scales.contains(&context.native_scale)
                || !input
                    .permission
                    .allowed_detector_ids
                    .contains(&context.detector_id)
            {
                return Err(ContractError::Invalid(
                    "population context violates causal or semantic contract".into(),
                ));
            }
        }
    }
    Ok(())
}

fn phenotype_summary(
    records: &[PopulationRecordInput],
) -> Result<Vec<PopulationPhenotype>, ContractError> {
    let mut groups: BTreeMap<(NativeScale, String), Vec<f64>> = BTreeMap::new();
    let mut counts: BTreeMap<(NativeScale, String), usize> = BTreeMap::new();
    for record in records {
        let key = (
            record.anchor.native_scale.clone(),
            record.anchor.lifecycle_state.clone(),
        );
        *counts.entry(key.clone()).or_default() += 1;
        if let Some(value) = record.anchor.value {
            groups.entry(key).or_default().push(value);
        }
    }

    counts
        .into_iter()
        .map(|((native_scale, lifecycle_state), samples)| {
            let mut values = groups
                .remove(&(native_scale.clone(), lifecycle_state.clone()))
                .unwrap_or_default();
            values.sort_by(|left, right| left.total_cmp(right));
            let mean = (!values.is_empty())
                .then(|| values.iter().sum::<f64>() / values.len() as f64);
            let minimum = values.first().copied();
            let maximum = values.last().copied();
            let p10 = quantile_sorted(&values, 0.10);
            let median = quantile_sorted(&values, 0.50);
            let p90 = quantile_sorted(&values, 0.90);
            let identity = hash_json(&(
                &native_scale,
                &lifecycle_state,
                samples,
                &values,
                mean,
                minimum,
                p10,
                median,
                p90,
                maximum,
            ))?;
            Ok(PopulationPhenotype {
                native_scale,
                lifecycle_state,
                samples,
                mean,
                minimum,
                p10,
                median,
                p90,
                maximum,
                identity,
                status: if samples == 0 {
                    EvidenceState::Unknown
                } else {
                    EvidenceState::Known
                },
            })
        })
        .collect()
}

fn discover(
    request: &MetricRequest,
    records: &[PopulationRecordInput],
) -> Result<MetricDiscovery, ContractError> {
    let mut pairs = Vec::new();
    for record in records {
        if record.anchor.native_scale != request.anchor_scale {
            continue;
        }
        let Some(context) = record.contexts.iter().find(|context| {
            context.detector_id == request.context_detector_id
                && context.native_scale == request.context_scale
        }) else {
            continue;
        };
        let lag = record.anchor.anchor_time - context.available_time;
        match request.metric {
            MetricKind::MeanLagNs => pairs.push((0.0, 0.0, lag)),
            MetricKind::Pearson | MetricKind::DirectionalConcordance => {
                if let (Some(anchor), Some(context_value)) = (record.anchor.value, context.value) {
                    pairs.push((anchor, context_value, lag));
                }
            }
        }
    }

    let observations = pairs.len();
    let mean_lag_ns = (!pairs.is_empty())
        .then(|| pairs.iter().map(|pair| pair.2 as f64).sum::<f64>() / observations as f64);
    let value = match request.metric {
        MetricKind::Pearson => pearson(&pairs),
        MetricKind::DirectionalConcordance => {
            let usable = pairs
                .iter()
                .filter(|(anchor, context, _)| *anchor != 0.0 && *context != 0.0)
                .collect::<Vec<_>>();
            (!usable.is_empty()).then(|| {
                usable
                    .iter()
                    .filter(|(anchor, context, _)| anchor.signum() == context.signum())
                    .count() as f64
                    / usable.len() as f64
            })
        }
        MetricKind::MeanLagNs => mean_lag_ns,
    };
    let status = if observations == 0 {
        EvidenceState::Unknown
    } else if value.is_none() {
        EvidenceState::Inconclusive
    } else {
        EvidenceState::Partial
    };
    let identity = hash_json(&(
        &request.question_id,
        &request.context_detector_id,
        &request.operator,
        request.metric,
        &request.anchor_scale,
        &request.context_scale,
        observations,
        value,
        mean_lag_ns,
    ))?;
    Ok(MetricDiscovery {
        question_id: request.question_id.clone(),
        context_detector_id: request.context_detector_id.clone(),
        operator: request.operator.clone(),
        metric: request.metric,
        anchor_scale: request.anchor_scale.clone(),
        context_scale: request.context_scale.clone(),
        same_native_scale: request.anchor_scale == request.context_scale,
        observations,
        value,
        mean_lag_ns,
        identity,
        status,
    })
}

fn evaluate_candidate(
    request: &MetricRequest,
    discovery: &MetricDiscovery,
) -> Result<CandidateEvaluation, ContractError> {
    let Some(gate) = &request.candidate_gate else {
        return Ok(CandidateEvaluation {
            question_id: request.question_id.clone(),
            discovery_identity: discovery.identity.clone(),
            gate: None,
            eligible: false,
            reason: "NO_PREDECLARED_GATE".into(),
        });
    };
    gate.validate()?;
    let eligible = discovery.observations >= gate.min_support
        && discovery
            .value
            .is_some_and(|value| value.abs() >= gate.min_abs_metric);
    Ok(CandidateEvaluation {
        question_id: request.question_id.clone(),
        discovery_identity: discovery.identity.clone(),
        gate: Some(gate.clone()),
        eligible,
        reason: if eligible {
            "PREDECLARED_GATE_PASSED"
        } else {
            "PREDECLARED_GATE_NOT_PASSED"
        }
        .into(),
    })
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

fn quantile_sorted(values: &[f64], probability: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let position = probability * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    Some(values[lower] + (values[upper] - values[lower]) * (position - lower as f64))
}

fn parse_operator(value: &str) -> Option<RelationshipOperator> {
    Some(match value {
        "temporal" => RelationshipOperator::Temporal,
        "directional" => RelationshipOperator::Directional,
        "spatial" => RelationshipOperator::Spatial,
        "nesting" => RelationshipOperator::Nesting,
        "state_transition" => RelationshipOperator::StateTransition,
        "lifecycle" => RelationshipOperator::Lifecycle,
        "regime" => RelationshipOperator::Regime,
        "lead_lag" => RelationshipOperator::LeadLag,
        "context" => RelationshipOperator::Context,
        "interaction" => RelationshipOperator::Interaction,
        "incremental_information" => RelationshipOperator::IncrementalInformation,
        _ => return None,
    })
}

fn has_duplicates(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().any(|value| value.is_empty() || !seen.insert(value))
}

fn report_identity(report: &PopulationResearchReport) -> Result<String, ContractError> {
    hash_json(&(
        report.report_version,
        report.records,
        report.anchor_instances,
        report.scope_start_ns,
        report.scope_end_ns,
        &report.phenotypes,
        &report.discoveries,
        &report.candidate_evaluations,
        &report.candidate_ids,
        &report.provenance,
        &report.complete_provenance,
        &report.status,
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
