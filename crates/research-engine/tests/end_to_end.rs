mod support;

use research_contracts::{
    CostModel, DecisionRule, ExecutionAssumption, HoldoutLeaf, HoldoutPolicy, HoldoutRole,
    RiskRule, ScopeInterval, StrategyHypothesis,
};
use research_engine::{
    run_challenge_subject, run_walk_forward, CausalStrategyObservation, ChallengeSubject,
    ConfirmedBehavioralInput, ConfirmedInputManifest, SimulationConfig, StrategyArchetype,
    StrategyObservation, StrategySpec, WalkForwardPlan,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, process::Command, time::SystemTime};

fn scope(id: &str, start: i64, end: i64) -> ScopeInterval {
    ScopeInterval {
        instrument_id: "XAUUSD".into(),
        scope_id: id.into(),
        dataset_identity: "synthetic".into(),
        start_time_ns: start,
        end_time_ns: end,
    }
}
fn policy() -> HoldoutPolicy {
    HoldoutPolicy {
        policy_id: "holdout".into(),
        frozen_at_ns: 0,
        dataset_identity: "synthetic".into(),
        leaves: vec![
            HoldoutLeaf {
                leaf_id: "dev".into(),
                scope: scope("dev", 0, 10),
                role: HoldoutRole::Development,
            },
            HoldoutLeaf {
                leaf_id: "strategy".into(),
                scope: scope("strategy", 10, 20),
                role: HoldoutRole::StrategyHoldout,
            },
        ],
        future_data_policy: None,
    }
}
fn spec() -> StrategySpec {
    StrategySpec {
        archetype: StrategyArchetype::Continuation,
        hypothesis: StrategyHypothesis {
            strategy_id: "s1".into(),
            hypothesis_id: "h1".into(),
            confirmed_finding_ids: vec!["finding".into()],
            parameters: BTreeMap::from([(String::from("bound"), json!(true))]),
        },
        decision_rule: DecisionRule {
            rule_id: "rule".into(),
            entry: "signal_threshold".into(),
            exit: "observation_end".into(),
            sizing: "unit".into(),
            management: "none".into(),
            parameters: BTreeMap::from([(String::from("entry_threshold"), json!(0.5))]),
        },
        execution_assumption: ExecutionAssumption {
            assumption_id: "execution".into(),
            assumptions: BTreeMap::from([(String::from("fill_model"), json!("analytical_close"))]),
        },
        cost_model: CostModel {
            cost_model_id: "cost".into(),
            parameters: BTreeMap::from([(String::from("per_observation"), json!(0.0))]),
        },
        risk_rule: RiskRule {
            risk_rule_id: "risk".into(),
            parameters: BTreeMap::from([(String::from("max_abs_outcome"), json!(2.0))]),
        },
    }
}
fn manifest() -> research_engine::CustodiedInputManifest {
    let plan_id = policy().identity_hash().unwrap();
    let lock = support::detector_lock("finding", &plan_id);
    let input = ConfirmedBehavioralInput::from_confirmation("finding", 0, &lock).unwrap();
    ConfirmedInputManifest::from_detector_confirmations("s1", plan_id, vec![input]).unwrap()
}
fn observations() -> Vec<CausalStrategyObservation> {
    [(1, 1.0), (11, -1.0)]
        .into_iter()
        .map(|(timestamp_ns, signal)| CausalStrategyObservation {
            observation: StrategyObservation {
                timestamp_ns,
                available_time_ns: timestamp_ns,
                signal: Some(signal),
                outcome: Some(signal),
            },
            input_ids: vec!["finding".into()],
        })
        .collect()
}

#[test]
fn real_stage_artifacts_are_bound_before_challenge() {
    let plan = WalkForwardPlan::from_policy(&policy(), "dev", "strategy", None).unwrap();
    let report = run_walk_forward(
        &spec(),
        &manifest(),
        &observations(),
        &plan,
        &SimulationConfig::default(),
    )
    .unwrap();
    let subject = ChallengeSubject::from_strategy_report(&report).unwrap();
    let bundle = run_challenge_subject(&subject).unwrap();
    assert_eq!(bundle.target_identity(), "s1");
    assert_eq!(bundle.report_identity(), report.report_identity);
    assert!(bundle.challenges().iter().all(|challenge| challenge
        .evidence_ids
        .iter()
        .all(|id| id.starts_with(bundle.report_identity()))));
}

#[test]
fn every_synthetic_stage_replays_in_two_fresh_processes() {
    let root = std::env::temp_dir().join(format!(
        "rsp-e2e-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let exe = env!("CARGO_BIN_EXE_research_engine");
    let run_stage = |instrument: &str, stage: &str| {
        let input = root.join(format!("{instrument}-{stage}.json"));
        fs::write(
            &input,
            serde_json::to_vec(&json!({"stage": stage, "instrument_id": instrument})).unwrap(),
        )
        .unwrap();
        let run = || {
            let output = Command::new(exe)
                .args(["synthetic-stage", input.to_str().unwrap()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{stage}/{instrument} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            serde_json::from_slice::<Value>(&output.stdout).unwrap()
        };
        let first = run();
        let second = run();
        assert_eq!(
            first, second,
            "{stage}/{instrument} changed across processes"
        );
        assert_eq!(first["stage"], stage);
        first
    };

    for instrument in ["XAUUSD", "EURUSD"] {
        for stage in ["r", "s", "p"] {
            let _ = run_stage(instrument, stage);
        }
    }
    let _ = fs::remove_dir_all(root);
}
