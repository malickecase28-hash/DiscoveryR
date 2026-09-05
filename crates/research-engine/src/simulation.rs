use crate::stats::MethodMetadata;
use crate::{finite, ConfirmedInputManifest, StrategyError, StrategySpec};

#[derive(Clone, Debug, PartialEq)]
pub struct StrategyObservation {
    pub timestamp_ns: i64,
    pub available_time_ns: i64,
    pub signal: Option<f64>,
    pub outcome: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationReport {
    pub observations: usize,
    pub acted: usize,
    pub abstentions: usize,
    pub risk_breaches: usize,
    pub gross_score: f64,
    pub costs: f64,
    pub net_score: f64,
    pub metadata: MethodMetadata,
}

pub type SimulationResult = SimulationReport;

pub fn simulate_strategy(
    spec: &StrategySpec,
    manifest: &ConfirmedInputManifest,
    observations: &[StrategyObservation],
    config: &SimulationConfig,
) -> Result<SimulationReport, StrategyError> {
    spec.validate(manifest)?;
    config.validate()?;
    let latest_input_available = manifest
        .inputs
        .iter()
        .map(|input| input.available_time_ns)
        .max()
        .ok_or(StrategyError::ConfirmedInputRequired)?;
    let mut report = SimulationReport {
        observations: observations.len(),
        acted: 0,
        abstentions: 0,
        risk_breaches: 0,
        gross_score: 0.0,
        costs: 0.0,
        net_score: 0.0,
        metadata: MethodMetadata {
            method: "unit-score-strategy-simulation".into(),
            assumptions: vec![
                "confirmed behavioral inputs are required".into(),
                "input and observation availability are no later than observation time".into(),
                "score is analytical unit exposure, not P&L".into(),
                "missing signal or outcome abstains".into(),
            ],
            approximate: false,
            allocation_bound: observations.len(),
            seed: None,
        },
    };
    for observation in observations {
        if observation.timestamp_ns < 0 || observation.available_time_ns < 0 {
            return Err(StrategyError::Invalid("observation timestamp"));
        }
        if observation.available_time_ns > observation.timestamp_ns
            || latest_input_available > observation.timestamp_ns
        {
            return Err(StrategyError::FutureInput {
                timestamp_ns: observation.timestamp_ns,
                available_time_ns: observation.available_time_ns.max(latest_input_available),
            });
        }
        let (Some(signal), Some(outcome)) = (observation.signal, observation.outcome) else {
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
    }
    report.net_score = report.gross_score - report.costs;
    finite(report.net_score, "net score")?;
    Ok(report)
}
