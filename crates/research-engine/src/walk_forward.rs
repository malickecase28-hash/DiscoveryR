use research_contracts::{HoldoutPolicy, HoldoutRole, ScopeInterval};

use crate::{
    simulate_strategy, ConfirmedInputManifest, SimulationConfig, SimulationReport, StrategyError,
    StrategyObservation, StrategySpec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
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
        Ok(Self {
            policy: policy.clone(),
            policy_identity: policy.policy_id.clone(),
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
        if self.policy.policy_id != self.policy_identity {
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

#[derive(Clone, Debug, PartialEq)]
pub struct WalkForwardReport {
    pub development_observations: usize,
    pub strategy_holdout_observations: usize,
    pub future_observations: usize,
    pub strategy_holdout: SimulationReport,
    pub future: Option<SimulationReport>,
    pub validation: research_contracts::StrategyValidation,
}

pub fn run_walk_forward(
    spec: &StrategySpec,
    manifest: &ConfirmedInputManifest,
    observations: &[StrategyObservation],
    plan: &WalkForwardPlan,
    config: &SimulationConfig,
) -> Result<WalkForwardReport, StrategyError> {
    plan.validate()?;
    if manifest.holdout_policy_identity != plan.policy_identity {
        return Err(StrategyError::IdentityMismatch("holdout policy"));
    }
    let mut development = Vec::new();
    let mut strategy = Vec::new();
    let mut future = Vec::new();
    for observation in observations {
        let bucket = if contains(&plan.development_scope, observation.timestamp_ns) {
            &mut development
        } else if contains(&plan.strategy_holdout_scope, observation.timestamp_ns) {
            &mut strategy
        } else if plan
            .future_scope
            .as_ref()
            .is_some_and(|scope| contains(scope, observation.timestamp_ns))
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
        .inputs
        .iter()
        .map(|input| input.input_id.clone())
        .collect();
    confirmed_input_ids.sort();
    let validation = research_contracts::StrategyValidation {
        validation_id: format!("validation-{}", spec.hypothesis.strategy_id),
        strategy_id: spec.hypothesis.strategy_id.clone(),
        confirmed_input_ids,
        code_identity: "strategy-engine-simulation-v1".into(),
        data_policy_identity: format!("data-policy-{}", plan.policy_identity),
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
    Ok(WalkForwardReport {
        development_observations: development.len(),
        strategy_holdout_observations: strategy.len(),
        future_observations: future.len(),
        strategy_holdout,
        future: future_report,
        validation,
    })
}

pub type WalkForwardResult = WalkForwardReport;

pub fn walk_forward(
    spec: &StrategySpec,
    manifest: &ConfirmedInputManifest,
    observations: &[StrategyObservation],
    plan: &WalkForwardPlan,
    config: &SimulationConfig,
) -> Result<WalkForwardReport, StrategyError> {
    run_walk_forward(spec, manifest, observations, plan, config)
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
