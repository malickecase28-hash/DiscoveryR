mod support;

use research_contracts::{
    CostModel, DecisionRule, EvidenceState, ExecutionAssumption, HoldoutLeaf, HoldoutPolicy,
    HoldoutRole, RiskRule, ScopeInterval, StrategyHypothesis,
};
use research_engine::{
    run_walk_forward, CausalStrategyObservation, ConfirmedBehavioralInput, ConfirmedInputManifest,
    SimulationConfig, StrategyArchetype, StrategyObservation, StrategySpec, WalkForwardPlan,
};
use serde_json::json;
use std::collections::BTreeMap;

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
            parameters: BTreeMap::from([(String::from("bound"), json!(true))]),
        },
        decision_rule: DecisionRule {
            rule_id: "rule-1".into(),
            entry: "signal_threshold".into(),
            exit: "observation_end".into(),
            sizing: "unit".into(),
            management: "none".into(),
            parameters: BTreeMap::from([(String::from("entry_threshold"), json!(0.5))]),
        },
        execution_assumption: ExecutionAssumption {
            assumption_id: "assumption-1".into(),
            assumptions: BTreeMap::from([(String::from("fill_model"), json!("analytical_close"))]),
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
fn manifest() -> research_engine::CustodiedInputManifest {
    let policy_id = policy().identity_hash().unwrap();
    let lock = support::detector_lock("finding-1", &policy_id);
    let input = ConfirmedBehavioralInput::from_confirmation("finding-1", 0, &lock).unwrap();
    ConfirmedInputManifest::from_detector_confirmations("strategy-1", policy_id, vec![input])
        .unwrap()
}
fn observations() -> Vec<CausalStrategyObservation> {
    [(1, 1.0, 1.0), (11, -1.0, 0.5), (21, 1.0, 1.0)]
        .into_iter()
        .map(
            |(timestamp_ns, signal, outcome)| CausalStrategyObservation {
                observation: StrategyObservation {
                    timestamp_ns,
                    available_time_ns: timestamp_ns,
                    signal: Some(signal),
                    outcome: Some(outcome),
                },
                input_ids: vec!["finding-1".into()],
            },
        )
        .collect()
}

#[test]
fn simulation_requires_custodied_inputs_and_accounts_cost() {
    let manifest = manifest();
    let report = research_engine::simulate_strategy(
        &spec(),
        &manifest,
        &observations()[1..2],
        &SimulationConfig {
            entry_threshold: 0.5,
            per_observation_cost: 0.1,
            max_abs_outcome: 2.0,
        },
    )
    .unwrap();
    assert!(report.authority_eligible);
    assert_eq!(report.acted, 1);
    assert!((report.net_score + 0.6).abs() < 1e-12);
}

#[test]
fn future_referenced_input_is_rejected() {
    let lock = support::detector_lock("future-finding", "holdout");
    let input = ConfirmedBehavioralInput::from_confirmation("future-finding", 100, &lock).unwrap();
    let manifest =
        ConfirmedInputManifest::from_detector_confirmations("strategy-1", "holdout", vec![input])
            .unwrap();
    let observation = CausalStrategyObservation {
        observation: StrategyObservation {
            timestamp_ns: 1,
            available_time_ns: 1,
            signal: Some(1.0),
            outcome: Some(1.0),
        },
        input_ids: vec!["future-finding".into()],
    };
    assert!(research_engine::simulate_strategy(
        &spec(),
        &manifest,
        &[observation],
        &SimulationConfig::default()
    )
    .is_err());
}

#[test]
fn descriptive_inputs_cannot_be_upgraded_to_custodied_manifest() {
    let input = ConfirmedBehavioralInput::untrusted_for_description("finding-1", 0).unwrap();
    assert!(ConfirmedInputManifest::from_detector_confirmations(
        "strategy-1",
        "holdout",
        vec![input]
    )
    .is_err());
}

#[test]
fn walk_forward_binds_nested_holdout_and_report_identity() {
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
    assert!(!report.report_identity.is_empty());
}
