use crate::stats::MethodMetadata;
use crate::{finite, ConfirmedInputManifest, CustodiedInputManifest, StrategyError, StrategySpec};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StrategyObservation {
    pub timestamp_ns: i64,
    pub available_time_ns: i64,
    pub signal: Option<f64>,
    pub outcome: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CausalStrategyObservation {
    pub observation: StrategyObservation,
    pub input_ids: Vec<String>,
}

fn validate_observation_inputs(
    manifest: &ConfirmedInputManifest,
    observations: &[CausalStrategyObservation],
) -> Result<(), StrategyError> {
    let available = manifest
        .inputs()
        .iter()
        .map(|input| (input.input_id(), input.available_time_ns()))
        .collect::<std::collections::BTreeMap<_, _>>();
    for observation in observations {
        if observation.input_ids.is_empty() {
            return Err(StrategyError::Invalid("observation input IDs"));
        }
        for id in &observation.input_ids {
            let available_time = available
                .get(id.as_str())
                .ok_or(StrategyError::ConfirmedInputRequired)?;
            if *available_time > observation.observation.timestamp_ns {
                return Err(StrategyError::FutureInput {
                    timestamp_ns: observation.observation.timestamp_ns,
                    available_time_ns: *available_time,
                });
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SimulationConfig {
    pub entry_threshold: f64,
    pub per_observation_cost: f64,
    pub max_abs_outcome: f64,
}
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            entry_threshold: 0.5,
            per_observation_cost: 0.0,
            max_abs_outcome: f64::MAX,
        }
    }
}
impl SimulationConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        finite(self.entry_threshold, "entry threshold")?;
        finite(self.per_observation_cost, "per observation cost")?;
        finite(self.max_abs_outcome, "maximum outcome")?;
        if self.entry_threshold < 0.0
            || self.per_observation_cost < 0.0
            || self.max_abs_outcome < 0.0
        {
            return Err(StrategyError::Invalid("simulation parameter"));
        }
        Ok(())
    }
}

fn validate_reference_unit_score_runtime(spec: &StrategySpec) -> Result<(), StrategyError> {
    if spec.decision_rule.entry != "signal_threshold"
        || spec.decision_rule.exit != "observation_end"
        || spec.decision_rule.sizing != "unit"
        || spec.decision_rule.management != "none"
    {
        return Err(StrategyError::Invalid(
            "decision rule is not executable by the reference unit-score simulator",
        ));
    }
    Ok(())
}

fn declared_number(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
) -> Option<f64> {
    values.get(key).and_then(serde_json::Value::as_f64)
}
fn effective_config(
    spec: &StrategySpec,
    config: &SimulationConfig,
) -> Result<SimulationConfig, StrategyError> {
    let defaults = SimulationConfig::default();
    let mut effective = config.clone();
    for (value, declared, default, name) in [
        (
            &mut effective.entry_threshold,
            declared_number(&spec.decision_rule.parameters, "entry_threshold"),
            defaults.entry_threshold,
            "entry threshold",
        ),
        (
            &mut effective.per_observation_cost,
            declared_number(&spec.cost_model.parameters, "per_observation"),
            defaults.per_observation_cost,
            "cost model",
        ),
        (
            &mut effective.max_abs_outcome,
            declared_number(&spec.risk_rule.parameters, "max_abs_outcome"),
            defaults.max_abs_outcome,
            "risk rule",
        ),
    ] {
        if *value == default {
            if let Some(declared) = declared {
                *value = declared;
            }
        } else if declared.is_some_and(|declared| declared != *value) {
            return Err(StrategyError::IdentityMismatch(name));
        }
    }
    effective.validate()?;
    Ok(effective)
}
fn runtime_identity(
    spec: &StrategySpec,
    config: &SimulationConfig,
) -> Result<String, StrategyError> {
    let bytes = serde_json::to_vec(&(spec, config))
        .map_err(|_| StrategyError::Invalid("strategy runtime identity"))?;
    let mut hash = Sha256::new();
    hash.update(b"trinityr-strategy-runtime-v2\0");
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SimulationReport {
    pub observations: usize,
    pub acted: usize,
    pub abstentions: usize,
    pub risk_breaches: usize,
    pub gross_score: f64,
    pub costs: f64,
    pub net_score: f64,
    pub runtime_spec_identity: String,
    pub input_manifest_identity: String,
    pub observation_identity: String,
    pub authority_eligible: bool,
    pub metadata: MethodMetadata,
}
pub type SimulationResult = SimulationReport;

fn simulate(
    spec: &StrategySpec,
    manifest: &ConfirmedInputManifest,
    observations: &[CausalStrategyObservation],
    config: &SimulationConfig,
    authority_eligible: bool,
) -> Result<SimulationReport, StrategyError> {
    spec.validate(manifest)?;
    validate_reference_unit_score_runtime(spec)?;
    validate_observation_inputs(manifest, observations)?;
    let config = effective_config(spec, config)?;
    let mut report = SimulationReport {
        observations: observations.len(),
        acted: 0,
        abstentions: 0,
        risk_breaches: 0,
        gross_score: 0.0,
        costs: 0.0,
        net_score: 0.0,
        runtime_spec_identity: runtime_identity(spec, &config)?,
        input_manifest_identity: manifest.identity(),
        observation_identity: String::new(),
        authority_eligible,
        metadata: MethodMetadata {
            method: "declared-unit-score-strategy-simulation".into(),
            assumptions: vec![
                "confirmed inputs are required".into(),
                "referenced input availability is causal".into(),
                "unit exposure score is not P&L".into(),
                "this runtime implements signal_threshold/observation_end/unit/none only".into(),
            ],
            approximate: false,
            allocation_bound: observations.len(),
            seed: None,
        },
    };
    for observation in observations {
        if observation.observation.timestamp_ns < 0
            || observation.observation.available_time_ns < 0
            || observation.observation.available_time_ns > observation.observation.timestamp_ns
        {
            return Err(StrategyError::FutureInput {
                timestamp_ns: observation.observation.timestamp_ns,
                available_time_ns: observation.observation.available_time_ns,
            });
        }
        let (Some(signal), Some(outcome)) = (
            observation.observation.signal,
            observation.observation.outcome,
        ) else {
            report.abstentions += 1;
            continue;
        };
        finite(signal, "signal")?;
        finite(outcome, "outcome")?;
        if outcome.abs() > config.max_abs_outcome {
            report.risk_breaches += 1;
            report.abstentions += 1;
            continue;
        }
        let direction = if signal >= config.entry_threshold {
            1.0
        } else if signal <= -config.entry_threshold {
            -1.0
        } else {
            report.abstentions += 1;
            continue;
        };
        report.acted += 1;
        report.gross_score += direction * outcome;
        report.costs += config.per_observation_cost;
        report.observation_identity = hash_observations(
            &report.observation_identity,
            &observation.input_ids,
            &observation.observation,
        )?;
    }
    report.net_score = report.gross_score - report.costs;
    finite(report.net_score, "net score")?;
    if report.observation_identity.is_empty() {
        report.observation_identity = hash_observations(
            "",
            &[],
            &StrategyObservation {
                timestamp_ns: 0,
                available_time_ns: 0,
                signal: None,
                outcome: None,
            },
        )?;
    }
    Ok(report)
}
fn hash_observations(
    previous: &str,
    input_ids: &[String],
    observation: &StrategyObservation,
) -> Result<String, StrategyError> {
    let bytes = serde_json::to_vec(&(previous, input_ids, observation))
        .map_err(|_| StrategyError::Invalid("observation identity"))?;
    let mut hash = Sha256::new();
    hash.update(b"trinityr-strategy-observation-v2\0");
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

/// Descriptive, worker-facing simulation; its report is explicitly ineligible for confirmation.
pub fn simulate_untrusted_strategy(
    spec: &StrategySpec,
    manifest: &ConfirmedInputManifest,
    observations: &[StrategyObservation],
    config: &SimulationConfig,
) -> Result<SimulationReport, StrategyError> {
    let causal = observations
        .iter()
        .map(|observation| CausalStrategyObservation {
            observation: observation.clone(),
            input_ids: manifest
                .inputs()
                .iter()
                .map(|input| input.input_id().to_owned())
                .collect(),
        })
        .collect::<Vec<_>>();
    simulate(spec, manifest, &causal, config, false)
}
pub fn simulate_strategy(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    config: &SimulationConfig,
) -> Result<SimulationReport, StrategyError> {
    simulate(spec, manifest.manifest(), observations, config, true)
}
pub fn simulate_causal_strategy(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    config: &SimulationConfig,
) -> Result<SimulationReport, StrategyError> {
    simulate_strategy(spec, manifest, observations, config)
}
pub fn simulate_custodied_strategy(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    config: &SimulationConfig,
) -> Result<SimulationReport, StrategyError> {
    simulate_strategy(spec, manifest, observations, config)
}
