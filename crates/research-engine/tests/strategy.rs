use std::collections::BTreeMap;

use research_contracts::{
    AttackKind, ChallengeResult, CostModel, DecisionRule, EvidenceState, ExecutionAssumption,
    HoldoutLeaf, HoldoutPolicy, HoldoutRole, RiskRule, ScopeInterval, StrategyHypothesis,
};
use research_engine::{
    method_challenge, run_walk_forward, simulate_strategy, ConfirmedBehavioralInput,
    ConfirmedInputManifest, SimulationConfig, StrategyArchetype, StrategyObservation, StrategySpec,
    WalkForwardPlan,
};
use serde_json::json;

fn params() -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([(String::from("entry_threshold"), json!(0.5))])
}

fn scope(id: &str, start: i64, end: i64) -> ScopeInterval {
    ScopeInterval {
        instrument_id: "XAUUSD".into(),
        scope_id: id.into(),
        dataset_identity: "synthetic-v1".into(),
        start_time_ns: start,
        end_time_ns: end,
    }
}

fn policy() -> HoldoutPolicy {
    HoldoutPolicy {
        policy_id: "holdout-v1".into(),
        frozen_at_ns: 0,
        dataset_identity: "synthetic-v1".into(),
        leaves: vec![
            HoldoutLeaf {
                leaf_id: "dev".into(),
                scope: scope("dev-scope", 0, 10),
                role: HoldoutRole::Development,
            },
            HoldoutLeaf {
                leaf_id: "strategy".into(),
                scope: scope("strategy-scope", 10, 20),
                role: HoldoutRole::StrategyHoldout,
            },
            HoldoutLeaf {
                leaf_id: "future".into(),
                scope: scope("future-scope", 20, 30),
                role: HoldoutRole::FutureData,
            },
        ],
        future_data_policy: Some(research_contracts::FutureDataPolicy {
            policy_id: "future-policy".into(),
            allowed: true,
        }),
    }
}

fn spec() -> StrategySpec {
    StrategySpec {
        archetype: StrategyArchetype::Continuation,
        hypothesis: StrategyHypothesis {
            strategy_id: "strategy-1".into(),
            hypothesis_id: "hypothesis-1".into(),
            confirmed_finding_ids: vec!["finding-1".into()],
            parameters: params(),
        },
        decision_rule: DecisionRule {
            rule_id: "rule-1".into(),
            entry: "signal threshold".into(),
            exit: "one observation".into(),
            sizing: "unit analytical exposure".into(),
            management: "abstain on missing input".into(),
            parameters: params(),
        },
        execution_assumption: ExecutionAssumption {
            assumption_id: "assumption-1".into(),
            assumptions: params(),
        },
        cost_model: CostModel {
            cost_model_id: "cost-1".into(),
            parameters: BTreeMap::from([(String::from("per_observation"), json!(0.1))]),
        },
        risk_rule: RiskRule {
            risk_rule_id: "risk-1".into(),
            parameters: BTreeMap::from([(String::from("max_abs_outcome"), json!(2.0))]),
        },
    }
}

fn manifest() -> ConfirmedInputManifest {
    ConfirmedInputManifest {
        strategy_id: "strategy-1".into(),
        holdout_policy_identity: policy().identity_hash().unwrap(),
        inputs: vec![ConfirmedBehavioralInput {
            input_id: "finding-1".into(),
            confirmed: true,
            available_time_ns: 0,
        }],
    }
}

fn observations() -> Vec<StrategyObservation> {
    vec![
        StrategyObservation {
            timestamp_ns: 1,
            available_time_ns: 1,
            signal: Some(1.0),
            outcome: Some(1.0),
        },
        StrategyObservation {
            timestamp_ns: 11,
            available_time_ns: 11,
            signal: Some(-1.0),
            outcome: Some(0.5),
        },
        StrategyObservation {
            timestamp_ns: 21,
            available_time_ns: 21,
            signal: None,
            outcome: Some(1.0),
        },
    ]
}

#[test]
fn simulation_requires_confirmed_behavior_and_accounts_abstention_and_cost() {
    let report = simulate_strategy(
        &spec(),
        &manifest(),
        &observations()[1..2],
        &SimulationConfig {
            entry_threshold: 0.5,
            per_observation_cost: 0.1,
            max_abs_outcome: 2.0,
        },
    )
    .unwrap();
    assert_eq!(report.acted, 1);
    assert_eq!(report.abstentions, 0);
    assert!((report.gross_score + 0.5).abs() < 1e-12);
    assert!((report.costs - 0.1).abs() < 1e-12);
    assert!((report.net_score + 0.6).abs() < 1e-12);
}

#[test]
fn simulation_rejects_future_available_input() {
    let mut input = observations()[1].clone();
    input.available_time_ns = input.timestamp_ns + 1;
    assert!(
        simulate_strategy(&spec(), &manifest(), &[input], &SimulationConfig::default(),).is_err()
    );
}

#[test]
fn simulation_rejects_missing_or_unconfirmed_behavioral_input() {
    let mut absent = manifest();
    absent.inputs.clear();
    assert!(simulate_strategy(
        &spec(),
        &absent,
        &observations()[..1],
        &SimulationConfig::default(),
    )
    .is_err());
    let mut unconfirmed = manifest();
    unconfirmed.inputs[0].confirmed = false;
    assert!(simulate_strategy(
        &spec(),
        &unconfirmed,
        &observations()[..1],
        &SimulationConfig::default(),
    )
    .is_err());
    let mut manifest = manifest();
    manifest.inputs[0].available_time_ns = 100;
    assert!(simulate_strategy(
        &spec(),
        &manifest,
        &observations()[..1],
        &SimulationConfig::default(),
    )
    .is_ok());
}

#[test]
fn walk_forward_requires_nested_strategy_and_future_identity() {
    let plan = WalkForwardPlan::from_policy(&policy(), "dev", "strategy", Some("future")).unwrap();
    let report = run_walk_forward(
        &spec(),
        &manifest(),
        &observations(),
        &plan,
        &SimulationConfig::default(),
    )
    .unwrap();
    assert_eq!(report.strategy_holdout_observations, 1);
    assert_eq!(report.future_observations, 1);
    assert_eq!(report.validation.state, EvidenceState::Partial);

    let mut bad = policy();
    bad.leaves[1].scope = scope("strategy-scope", 9, 20);
    assert!(WalkForwardPlan::from_policy(&bad, "dev", "strategy", Some("future")).is_err());
}

#[test]
fn method_challenge_is_separate_from_confirmation() {
    let plan = WalkForwardPlan::from_policy(&policy(), "dev", "strategy", Some("future")).unwrap();
    let challenge = method_challenge("strategy-1", &plan, &observations()).unwrap();
    assert_eq!(challenge.result, ChallengeResult::Passed);
    assert_eq!(challenge.attack_kind, AttackKind::TimestampLeakage);
}
