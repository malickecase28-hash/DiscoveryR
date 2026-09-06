mod support;

use std::collections::BTreeMap;

use research_contracts::{
    evidence::{ConfirmationState, EvidenceState},
    ContextPermission, EvidenceAccess, PortfolioComponent, ProgramConsumer,
};
use research_engine::portfolio::*;
use research_engine::ConfirmationKind;

fn stream(id: &str, returns: Vec<Option<f64>>, signals: Vec<Option<f64>>) -> StrategyStream {
    let n = returns.len();
    StrategyStream {
        strategy_id: id.into(),
        confirmation: ConfirmationState::Confirmed,
        evidence_state: EvidenceState::Known,
        returns,
        signals,
        regimes: vec![Some("risk_on".into()); n],
        capital: Some(if id == "s1" { 100.0 } else { 50.0 }),
        capacity: Some(if id == "s1" { 80.0 } else { 120.0 }),
        liquidity: vec![Some(10.0); n],
        risk_budget: Some(if id == "s1" { 2.0 } else { 1.0 }),
    }
}

fn input(strategies: Vec<StrategyStream>) -> PortfolioInput {
    PortfolioInput {
        component: PortfolioComponent {
            component_id: "p1".into(),
            confirmed_strategy_ids: strategies.iter().map(|s| s.strategy_id.clone()).collect(),
            parameters: BTreeMap::from([(String::from("scope"), serde_json::json!("synthetic"))]),
        },
        strategies,
        constraints: vec![],
    }
}

fn aligned_permission() -> ContextPermission {
    ContextPermission {
        permission_id: "p-permission".into(),
        consumer: ProgramConsumer::PortfolioResearch,
        allowed_detector_ids: vec!["strategy".into()],
        allowed_scales: std::collections::BTreeSet::from([research_contracts::NativeScale::Tick]),
        evidence_access: EvidenceAccess::ConfirmedOnly,
    }
}

#[test]
fn synthetic_two_strategy_reports_are_analytic_and_reproducible() {
    let data = input(vec![
        stream(
            "s1",
            vec![Some(1.0), Some(2.0), Some(-1.0), Some(3.0)],
            vec![Some(1.0), Some(0.0), Some(1.0), Some(1.0)],
        ),
        stream(
            "s2",
            vec![Some(-1.0), Some(-2.0), Some(1.0), Some(-3.0)],
            vec![Some(1.0), Some(1.0), Some(0.0), Some(1.0)],
        ),
    ]);
    let correlation = correlation_report(&data, "s1", "s2").unwrap();
    assert_eq!(correlation.status, PortfolioStatus::Known);
    assert!((correlation.value.unwrap() + 1.0).abs() < 1e-12);
    assert_eq!(correlation.observations, 4);

    let conditional = conditional_correlation_report(&data, "s1", "s2").unwrap();
    assert_eq!(conditional.len(), 1);
    assert_eq!(conditional[0].regime, "risk_on");
    assert!((conditional[0].value.unwrap() + 1.0).abs() < 1e-12);

    let overlap = overlap_report(&data, "s1", "s2").unwrap();
    assert_eq!(overlap.union, 4);
    assert_eq!(overlap.intersection, 2);
    assert!((overlap.jaccard - 0.5).abs() < 1e-12);

    let capital = capital_allocation_report(&data).unwrap();
    assert!((capital.allocations[0].fraction.unwrap() - 2.0 / 3.0).abs() < 1e-12);
    assert_eq!(capital.total_capital, 150.0);

    let concentration = concentration_report(&data).unwrap();
    assert!((concentration.herfindahl - 5.0 / 9.0).abs() < 1e-12);
    assert_eq!(concentration.status, PortfolioStatus::Known);

    let again = portfolio_report(&data, &[]).unwrap();
    assert_eq!(again, portfolio_report(&data, &[]).unwrap());
}

#[test]
fn missing_and_non_confirmed_inputs_remain_visible_or_fail_closed() {
    let mut unknown = stream("s1", vec![Some(1.0), None, Some(2.0)], vec![Some(1.0); 3]);
    unknown.evidence_state = EvidenceState::Unknown;
    let mut contradictory = stream(
        "s2",
        vec![Some(1.0), Some(2.0), Some(3.0)],
        vec![Some(1.0); 3],
    );
    let known = stream(
        "s2",
        vec![Some(1.0), Some(2.0), Some(3.0)],
        vec![Some(1.0); 3],
    );
    let data = input(vec![unknown.clone(), known]);
    assert_eq!(
        correlation_report(&data, "s1", "s2").unwrap().status,
        PortfolioStatus::Unknown
    );
    contradictory.evidence_state = EvidenceState::Contradictory;
    let data = input(vec![unknown, contradictory]);
    assert_eq!(
        turnover_report(&data).unwrap().status,
        PortfolioStatus::Contradictory
    );

    let mut unconfirmed = stream("s1", vec![Some(1.0)], vec![Some(1.0)]);
    unconfirmed.confirmation = ConfirmationState::Unknown;
    assert!(matches!(
        input(vec![unconfirmed]).validate(),
        Err(PortfolioError::UnconfirmedStrategy { .. })
    ));
}

#[test]
fn descriptive_risk_reports_do_not_optimize_or_allocate_live_capital() {
    let data = input(vec![
        stream(
            "s1",
            vec![Some(0.1), Some(-0.2), Some(0.1)],
            vec![Some(1.0); 3],
        ),
        stream(
            "s2",
            vec![Some(0.0), Some(0.2), Some(-0.1)],
            vec![Some(1.0); 3],
        ),
    ]);
    let drawdown = drawdown_report(&data).unwrap();
    assert!(drawdown.max_drawdown >= 0.0);
    let capacity = capacity_report(&data).unwrap();
    assert_eq!(capacity.bottleneck, Some(80.0));
    let liquidity = liquidity_report(&data).unwrap();
    assert_eq!(liquidity.minimum, Some(10.0));
    let risk = risk_budget_report(&data).unwrap();
    assert_eq!(risk.total_budget, 3.0);
    let stress = stress_report(
        &data,
        &[StressScenario {
            scenario_id: "shock".into(),
            returns: BTreeMap::from([(String::from("s1"), -0.5), (String::from("s2"), -0.2)]),
        }],
    )
    .unwrap();
    assert_eq!(stress.scenarios.len(), 1);
    assert!(stress.scenarios[0].portfolio_return < 0.0);
}

#[test]
fn aligned_portfolio_binds_stream_report_and_time_grid() {
    let permission = aligned_permission();
    let stream = stream("s1", vec![Some(1.0), Some(2.0)], vec![Some(1.0), Some(1.0)]);
    let timestamps = vec![1, 2];
    let availability = vec![1, 2];
    let scope = "scope-s1";
    let source = "source-s1";
    let report_identity = strategy_stream_identity(
        &stream,
        &timestamps,
        &availability,
        scope,
        source,
        &permission,
    );
    let confirmation = support::strategy_lock("s1", &report_identity, "holdout-s1");
    assert_eq!(confirmation.kind(), ConfirmationKind::Strategy);
    let aligned = AlignedStrategyStream::new(
        stream.clone(),
        timestamps.clone(),
        availability.clone(),
        scope.into(),
        source.into(),
        permission.clone(),
        confirmation,
    )
    .unwrap();
    let component = PortfolioComponent {
        component_id: "p1".into(),
        confirmed_strategy_ids: vec!["s1".into()],
        parameters: BTreeMap::from([(String::from("scope"), serde_json::json!("synthetic"))]),
    };
    let report =
        portfolio_report_aligned(component, std::slice::from_ref(&aligned), vec![], &[]).unwrap();
    assert!(!report.identity.is_empty());
    report.validate_identity().unwrap();
    let mut changed = aligned;
    changed.stream.returns[0] = Some(9.0);
    assert!(portfolio_report_aligned(
        PortfolioComponent {
            component_id: "p1".into(),
            confirmed_strategy_ids: vec!["s1".into()],
            parameters: BTreeMap::from([(String::from("scope"), serde_json::json!("synthetic"))])
        },
        &[changed],
        vec![],
        &[]
    )
    .is_err());
}
