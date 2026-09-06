use research_engine::stats::*;

#[test]
fn moments_are_streaming_and_mergeable() {
    let mut m = StreamingMoments::new();
    m.extend(&[1.0, 2.0, 3.0]).unwrap();
    assert_eq!(m.mean(), Some(2.0));
    assert_eq!(m.variance(), Some(1.0));
    let mut left = StreamingMoments::new();
    left.extend(&[1.0]).unwrap();
    let mut right = StreamingMoments::new();
    right.extend(&[2.0, 3.0]).unwrap();
    left.merge(&right).unwrap();
    assert!((left.population_variance().unwrap() - 2.0 / 3.0).abs() < 1e-12);
}

#[test]
fn quantile_interpolates_and_enforces_capacity() {
    assert_eq!(
        exact_quantile(&[1.0, 4.0, 2.0, 3.0], 0.25, 4)
            .unwrap()
            .value,
        1.75
    );
    assert!(matches!(
        exact_quantile(&[1.0, 2.0], 0.5, 1),
        Err(StatsError::CapacityExceeded { .. })
    ));
    assert!(matches!(
        exact_quantile(&[1.0, f64::NAN], 0.5, 2),
        Err(StatsError::NonFinite { .. })
    ));
}

#[test]
fn effects_have_sign_symmetry_and_zero_scale_is_no_estimate() {
    let ab = effect_size(&[3.0, 4.0, 5.0], &[1.0, 2.0, 3.0]).unwrap();
    let ba = effect_size(&[1.0, 2.0, 3.0], &[3.0, 4.0, 5.0]).unwrap();
    assert_eq!(ab.difference, -ba.difference);
    assert_eq!(
        ab.standardized_difference,
        Some(-ba.standardized_difference.unwrap())
    );
    assert!(effect_size(&[1.0, 1.0], &[1.0, 1.0])
        .unwrap()
        .standardized_difference
        .is_none());
}

fn obs(stratum: &str, id: &str, outcome: f64, x: f64) -> MatchedObservation {
    MatchedObservation {
        stratum: stratum.into(),
        id: id.into(),
        outcome,
        covariates: vec![x],
    }
}

#[test]
fn matching_is_exact_stratum_deterministic_and_no_reuse_by_default() {
    let treated = vec![
        obs("A", "t1", 10.0, 0.0),
        obs("A", "t2", 8.0, 0.1),
        obs("B", "t3", 9.0, 0.0),
    ];
    let controls = vec![obs("A", "c1", 6.0, 0.0), obs("B", "c2", 7.0, 0.0)];
    let config = MatchConfig {
        caliper: Some(0.2),
        allow_reuse: false,
        max_pairs: 10,
    };
    let result = exact_strata_match(&treated, &controls, &config).unwrap();
    assert_eq!(
        result
            .pairs
            .iter()
            .map(|p| p.control_id.as_str())
            .collect::<Vec<_>>(),
        vec!["c1", "c2"]
    );
    assert_eq!(result.pairs.len(), 2);
    let reused = exact_strata_match(
        &treated[..2],
        &controls[..1],
        &MatchConfig {
            allow_reuse: true,
            ..config
        },
    )
    .unwrap();
    assert_eq!(reused.pairs.len(), 2);
}

#[test]
fn block_nulls_are_seeded_and_preserve_declared_units() {
    let values = vec![
        BlockObservation {
            block: "d1".into(),
            value: 1.0,
        },
        BlockObservation {
            block: "d1".into(),
            value: 2.0,
        },
        BlockObservation {
            block: "d2".into(),
            value: 10.0,
        },
        BlockObservation {
            block: "d2".into(),
            value: 11.0,
        },
    ];
    let a = bootstrap_by_block(&values, 64, 7, 64).unwrap();
    let b = bootstrap_by_block(&values, 64, 7, 64).unwrap();
    let c = bootstrap_by_block(&values, 64, 8, 64).unwrap();
    assert_eq!(a, b);
    assert_ne!(a.metadata.seed, c.metadata.seed);
    assert!((a.p_value - (a.exceedances + 1) as f64 / 65.0).abs() < 1e-12);

    let grouped = vec![
        GroupedObservation {
            block: "d1".into(),
            group: true,
            value: 3.0,
        },
        GroupedObservation {
            block: "d1".into(),
            group: false,
            value: 1.0,
        },
        GroupedObservation {
            block: "d2".into(),
            group: true,
            value: 5.0,
        },
        GroupedObservation {
            block: "d2".into(),
            group: false,
            value: 2.0,
        },
    ];
    let p = block_permutation(&grouped, 32, 2, 32).unwrap();
    assert!(p.p_value >= 1.0 / 33.0 && p.p_value <= 1.0);
}

#[test]
fn time_shift_respects_group_boundaries_and_is_reproducible() {
    let x = [1.0, 2.0, 3.0, 10.0, 20.0, 30.0];
    let y = [1.0, 2.0, 3.0, 30.0, 20.0, 10.0];
    let a = circular_time_shift_null(&x, &y, &[3, 3], 20, 11, 20).unwrap();
    let b = circular_time_shift_null(&x, &y, &[3, 3], 20, 11, 20).unwrap();
    assert_eq!(a, b);
    assert!(a.metadata.assumptions.iter().any(|s| s.contains("group")));
    assert!(circular_time_shift_null(&x, &y, &[2, 3], 20, 11, 20).is_err());
}

#[test]
fn fdr_keeps_missing_family_members_and_matches_known_vectors() {
    let entries = vec![
        FamilyEntry {
            p_value: Some(0.01),
            status: EvidenceStatus::Known,
        },
        FamilyEntry {
            p_value: Some(0.04),
            status: EvidenceStatus::Null,
        },
        FamilyEntry {
            p_value: Some(0.20),
            status: EvidenceStatus::Rejected,
        },
        FamilyEntry {
            p_value: None,
            status: EvidenceStatus::Missing,
        },
    ];
    let bh = fdr_bh_by(&entries, 0.05, false).unwrap();
    assert_eq!(bh.family_size, 4);
    assert_eq!(bh.tested, 3);
    assert_eq!(bh.adjusted_p_values[0], Some(0.03));
    assert_eq!(bh.adjusted_p_values[3], None);
    assert!(bh.rejected[0]);
    let by = fdr_bh_by(&entries, 0.05, true).unwrap();
    assert!(by.adjusted_p_values[0].unwrap() > bh.adjusted_p_values[0].unwrap());
}

#[test]
fn max_statistic_uses_joint_null_and_support_reports_every_cluster() {
    let p = max_statistic_p_values(
        &[2.0, 0.5],
        &[vec![1.0, 0.2], vec![2.5, 0.1], vec![0.5, 4.0]],
        3,
    )
    .unwrap();
    assert_eq!(p, vec![0.75, 1.0]);
    let support = support_concentration(&["d2".into(), "d1".into(), "d2".into()], 3).unwrap();
    assert_eq!(support.clusters[0].cluster, "d1");
    assert_eq!(support.clusters[1].count, 2);
    let rich = max_statistic(&[2.0, 0.5], &[vec![1.0, 0.2]], 1).unwrap();
    assert_eq!(rich.draws, 1);
    assert!(rich
        .metadata
        .assumptions
        .iter()
        .any(|s| s.contains("same joint")));
    let mut stream = MaxStatisticAccumulator::new(&[2.0, 0.5], 3).unwrap();
    stream.push_world(&[1.0, 0.2]).unwrap();
    stream.push_world(&[2.5, 0.1]).unwrap();
    stream.push_world(&[0.5, 4.0]).unwrap();
    assert_eq!(stream.finish().unwrap(), p);
}

#[test]
fn survival_handles_ties_and_censor_only_without_dropping_risk_sets() {
    let km = kaplan_meier(
        &[
            SurvivalObservation {
                time: 1.0,
                event: true,
            },
            SurvivalObservation {
                time: 1.0,
                event: false,
            },
            SurvivalObservation {
                time: 2.0,
                event: true,
            },
            SurvivalObservation {
                time: 3.0,
                event: false,
            },
        ],
        4,
    )
    .unwrap();
    assert_eq!(
        (
            km.points[0].at_risk,
            km.points[0].events,
            km.points[0].censored
        ),
        (4, 1, 1)
    );
    assert!((km.points[0].survival - 0.75).abs() < 1e-12);
    assert_eq!(km.points[1].at_risk, 2);
    assert!((km.points[1].survival - 0.375).abs() < 1e-12);
    let censored = kaplan_meier(
        &[SurvivalObservation {
            time: 1.0,
            event: false,
        }],
        1,
    )
    .unwrap();
    assert_eq!(censored.points[0].survival, 1.0);
    assert_eq!(censored.points[0].hazard_increment, 0.0);
}
