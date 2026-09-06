use research_contracts::{HoldoutPolicy, HoldoutRole, ScopeInterval};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    simulate_strategy, CausalStrategyObservation, CustodiedInputManifest, SimulationConfig,
    SimulationReport, StrategyError, StrategySpec,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WalkForwardPlan {
    pub policy: HoldoutPolicy,
    pub policy_identity: String,
    pub development_leaf_id: String,
    pub strategy_holdout_leaf_id: String,
    pub future_leaf_id: Option<String>,
    pub development_scope: ScopeInterval,
    pub strategy_holdout_scope: ScopeInterval,
    pub future_scope: Option<ScopeInterval>,
}

impl WalkForwardPlan {
    pub fn from_policy(
        policy: &HoldoutPolicy,
        development_leaf_id: &str,
        strategy_holdout_leaf_id: &str,
        future_leaf_id: Option<&str>,
    ) -> Result<Self, StrategyError> {
        policy
            .validate()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        let development = leaf(policy, development_leaf_id, HoldoutRole::Development)?;
        let strategy = leaf(
            policy,
            strategy_holdout_leaf_id,
            HoldoutRole::StrategyHoldout,
        )?;
        if development.scope.end_time_ns > strategy.scope.start_time_ns {
            return Err(StrategyError::Holdout(
                "development and strategy holdout are not temporally nested".into(),
            ));
        }
        let future = future_leaf(policy, future_leaf_id)?;
        if let Some(future) = &future {
            if strategy.scope.end_time_ns > future.scope.start_time_ns {
                return Err(StrategyError::Holdout(
                    "strategy holdout and future scope are not temporally ordered".into(),
                ));
            }
        }
        let policy_identity = policy
            .identity_hash()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        Ok(Self {
            policy: policy.clone(),
            policy_identity,
            development_leaf_id: development_leaf_id.into(),
            strategy_holdout_leaf_id: strategy_holdout_leaf_id.into(),
            future_leaf_id: future_leaf_id.map(str::to_owned),
            development_scope: development.scope.clone(),
            strategy_holdout_scope: strategy.scope.clone(),
            future_scope: future.map(|leaf| leaf.scope.clone()),
        })
    }

    pub fn validate(&self) -> Result<(), StrategyError> {
        self.policy
            .validate()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        let expected_policy_identity = self
            .policy
            .identity_hash()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        if expected_policy_identity != self.policy_identity {
            return Err(StrategyError::Holdout("policy identity mismatch".into()));
        }
        let development = leaf(
            &self.policy,
            &self.development_leaf_id,
            HoldoutRole::Development,
        )?;
        let strategy = leaf(
            &self.policy,
            &self.strategy_holdout_leaf_id,
            HoldoutRole::StrategyHoldout,
        )?;
        if development.scope != self.development_scope
            || strategy.scope != self.strategy_holdout_scope
        {
            return Err(StrategyError::Holdout(
                "holdout scope identity mismatch".into(),
            ));
        }
        let future = future_leaf(&self.policy, self.future_leaf_id.as_deref())?;
        if future.map(|leaf| leaf.scope.clone()) != self.future_scope {
            return Err(StrategyError::Holdout(
                "future scope identity mismatch".into(),
            ));
        }
        self.development_scope
            .validate()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        self.strategy_holdout_scope
            .validate()
            .map_err(|error| StrategyError::Holdout(error.to_string()))?;
        if self.development_scope.dataset_identity != self.strategy_holdout_scope.dataset_identity
            || self.development_scope.instrument_id != self.strategy_holdout_scope.instrument_id
        {
            return Err(StrategyError::Holdout("partition identity mismatch".into()));
        }
        if self.development_scope.end_time_ns > self.strategy_holdout_scope.start_time_ns {
            return Err(StrategyError::Holdout("partitions overlap".into()));
        }
        if let Some(future) = &self.future_scope {
            future
                .validate()
                .map_err(|error| StrategyError::Holdout(error.to_string()))?;
            if future.dataset_identity != self.strategy_holdout_scope.dataset_identity
                || future.instrument_id != self.strategy_holdout_scope.instrument_id
                || self.strategy_holdout_scope.end_time_ns > future.start_time_ns
            {
                return Err(StrategyError::Holdout(
                    "future scope identity or ordering mismatch".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalkForwardReport {
    pub development_observations: usize,
    pub strategy_holdout_observations: usize,
    pub future_observations: usize,
    pub strategy_holdout: SimulationReport,
    pub future: Option<SimulationReport>,
    pub validation: research_contracts::StrategyValidation,
    pub development_fit: Option<DevelopmentFit>,
    pub input_manifest_identity: String,
    pub report_identity: String,
}

impl WalkForwardReport {
    pub fn validate_identity(&self) -> Result<(), StrategyError> {
        if self.report_identity != report_identity(self)? {
            return Err(StrategyError::IdentityMismatch("walk-forward report"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DevelopmentFit {
    pub mean_signal: f64,
    pub scale_signal: f64,
    pub identity: String,
}

pub fn run_walk_forward(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    plan: &WalkForwardPlan,
    config: &SimulationConfig,
) -> Result<WalkForwardReport, StrategyError> {
    plan.validate()?;
    if manifest.manifest().holdout_policy_identity() != plan.policy_identity {
        return Err(StrategyError::IdentityMismatch("holdout policy"));
    }
    let mut development = Vec::new();
    let mut strategy = Vec::new();
    let mut future = Vec::new();
    for observation in observations {
        let timestamp_ns = observation.observation.timestamp_ns;
        let bucket = if contains(&plan.development_scope, timestamp_ns) {
            &mut development
        } else if contains(&plan.strategy_holdout_scope, timestamp_ns) {
            &mut strategy
        } else if plan
            .future_scope
            .as_ref()
            .is_some_and(|scope| contains(scope, timestamp_ns))
        {
            &mut future
        } else {
            return Err(StrategyError::Holdout(
                "observation lies outside declared walk-forward leaves".into(),
            ));
        };
        bucket.push(observation.clone());
    }
    let strategy_holdout = simulate_strategy(spec, manifest, &strategy, config)?;
    let future_report = plan
        .future_scope
        .as_ref()
        .map(|_| simulate_strategy(spec, manifest, &future, config))
        .transpose()?;
    let mut confirmed_input_ids: Vec<String> = manifest
        .manifest()
        .inputs()
        .iter()
        .map(|input| input.input_id().to_owned())
        .collect();
    confirmed_input_ids.sort();
    let validation = research_contracts::StrategyValidation {
        validation_id: format!("validation-{}", spec.hypothesis.strategy_id),
        strategy_id: spec.hypothesis.strategy_id.clone(),
        confirmed_input_ids,
        code_identity: strategy_holdout.runtime_spec_identity.clone(),
        data_policy_identity: plan.policy_identity.clone(),
        state: research_contracts::EvidenceState::Partial,
        parameters: std::collections::BTreeMap::from([
            (
                "development_observations".into(),
                serde_json::json!(development.len()),
            ),
            (
                "strategy_holdout_observations".into(),
                serde_json::json!(strategy.len()),
            ),
            (
                "future_observations".into(),
                serde_json::json!(future.len()),
            ),
            (
                "abstentions".into(),
                serde_json::json!(strategy_holdout.abstentions),
            ),
            (
                "risk_breaches".into(),
                serde_json::json!(strategy_holdout.risk_breaches),
            ),
            ("costs".into(), serde_json::json!(strategy_holdout.costs)),
            (
                "analytical_net_score".into(),
                serde_json::json!(strategy_holdout.net_score),
            ),
        ]),
    };
    let mut report = WalkForwardReport {
        development_observations: development.len(),
        strategy_holdout_observations: strategy.len(),
        future_observations: future.len(),
        strategy_holdout,
        future: future_report,
        validation,
        development_fit: None,
        input_manifest_identity: manifest.manifest().identity(),
        report_identity: String::new(),
    };
    report.report_identity = report_identity(&report)?;
    Ok(report)
}

pub type WalkForwardResult = WalkForwardReport;

pub fn walk_forward(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    plan: &WalkForwardPlan,
    config: &SimulationConfig,
) -> Result<WalkForwardReport, StrategyError> {
    run_walk_forward(spec, manifest, observations, plan, config)
}

/// Deterministic development fit followed by evaluation on the declared
/// strategy holdout. The fitted identity is recorded in validation parameters
/// so later confirmation cannot silently rebuild a different rule.
pub fn run_walk_forward_fitted(
    spec: &StrategySpec,
    manifest: &CustodiedInputManifest,
    observations: &[CausalStrategyObservation],
    plan: &WalkForwardPlan,
    config: &SimulationConfig,
) -> Result<WalkForwardReport, StrategyError> {
    let mut report = run_walk_forward(spec, manifest, observations, plan, config)?;
    if report.development_observations == 0 || report.strategy_holdout_observations == 0 {
        return Err(StrategyError::EmptySample);
    }
    let development = observations.iter().filter(|observation| {
        contains(
            &plan.development_scope,
            observation.observation.timestamp_ns,
        )
    });
    let development = development.collect::<Vec<_>>();
    let signals = development
        .iter()
        .filter_map(|o| o.observation.signal)
        .collect::<Vec<_>>();
    if signals.is_empty() {
        return Err(StrategyError::EmptySample);
    }
    let mean = signals.iter().sum::<f64>() / signals.len() as f64;
    let variance = signals
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / signals.len() as f64;
    let scale = variance.sqrt().max(f64::EPSILON);
    let mut hash = Sha256::new();
    hash.update(
        serde_json::to_vec(&(spec, plan, config, &development))
            .map_err(|_| StrategyError::Invalid("strategy fit identity"))?,
    );
    let fit_identity = format!("{:x}", hash.finalize());
    let fit = DevelopmentFit {
        mean_signal: mean,
        scale_signal: scale,
        identity: fit_identity.clone(),
    };
    let normalized = observations
        .iter()
        .map(|observation| {
            let mut normalized = observation.clone();
            normalized.observation.signal = observation
                .observation
                .signal
                .map(|signal| (signal - mean) / scale);
            normalized
        })
        .collect::<Vec<_>>();
    let strategy = normalized
        .iter()
        .filter(|o| contains(&plan.strategy_holdout_scope, o.observation.timestamp_ns))
        .cloned()
        .collect::<Vec<_>>();
    let future = normalized
        .iter()
        .filter(|o| {
            plan.future_scope
                .as_ref()
                .is_some_and(|scope| contains(scope, o.observation.timestamp_ns))
        })
        .cloned()
        .collect::<Vec<_>>();
    report.strategy_holdout = simulate_strategy(spec, manifest, &strategy, config)?;
    report.future = plan
        .future_scope
        .as_ref()
        .map(|_| simulate_strategy(spec, manifest, &future, config))
        .transpose()?;
    report.development_fit = Some(fit);
    report.validation.state = research_contracts::EvidenceState::Known;
    report.validation.parameters.insert(
        "development_fit_identity".into(),
        serde_json::json!(fit_identity),
    );
    report.validation.parameters.insert(
        "development_fit_observations".into(),
        serde_json::json!(development.len()),
    );
    report.report_identity = report_identity(&report)?;
    Ok(report)
}

fn report_identity(report: &WalkForwardReport) -> Result<String, StrategyError> {
    let bytes = serde_json::to_vec(&(
        report.development_observations,
        report.strategy_holdout_observations,
        report.future_observations,
        &report.strategy_holdout,
        &report.future,
        &report.validation,
        &report.development_fit,
        &report.input_manifest_identity,
    ))
    .map_err(|_| StrategyError::Invalid("walk-forward report identity"))?;
    let mut hash = Sha256::new();
    hash.update(b"trinityr-walk-forward-report-v2\0");
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

fn contains(scope: &ScopeInterval, timestamp_ns: i64) -> bool {
    scope.start_time_ns <= timestamp_ns && timestamp_ns < scope.end_time_ns
}

fn leaf<'a>(
    policy: &'a HoldoutPolicy,
    id: &str,
    role: HoldoutRole,
) -> Result<&'a research_contracts::HoldoutLeaf, StrategyError> {
    policy
        .leaves
        .iter()
        .find(|leaf| leaf.leaf_id == id && leaf.role == role)
        .ok_or_else(|| StrategyError::Holdout(format!("missing {role:?} leaf: {id}")))
}

fn future_leaf<'a>(
    policy: &'a HoldoutPolicy,
    id: Option<&str>,
) -> Result<Option<&'a research_contracts::HoldoutLeaf>, StrategyError> {
    id.map(|id| leaf(policy, id, HoldoutRole::FutureData))
        .transpose()
}
