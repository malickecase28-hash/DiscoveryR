use research_tape::ap001_drift_burst::{DState, Snapshot};
use research_tape::ap001_wp2::Capture;

fn payload(state: &str, event_id: i64, weak: Option<i64>) -> String {
    serde_json::json!({
        "drift_burst_state": {
            "state": state,
            "event_id": event_id,
            "milestones": {
                "weak": weak.map(|ts_ms| serde_json::json!({"ts_ms": ts_ms})),
                "online": null
            },
            "operational": {"direction": "UP"},
            "multiscale": {},
            "authority": {},
            "emitted_events": []
        }
    })
    .to_string()
}

fn observe(capture: &mut Capture, ts: i64, state: DState, event_id: i64, weak: Option<i64>) {
    let raw = payload(state.label(), event_id, weak);
    let snapshot = Snapshot {
        state,
        eid_pos: (event_id >= 0).then_some(event_id),
        weak_ms: weak,
        online_ms: None,
        emissions: Vec::new(),
    };
    capture.observe(ts, Some(ts), Some(ts), 100.0, 100.1, &raw, &snapshot);
}

#[test]
fn wp1_boundary_rules_reconcile_first_entry_anchor_counts() {
    let mut capture = Capture::default();
    observe(&mut capture, 1, DState::Developing, -1, Some(10));
    observe(&mut capture, 2, DState::Burst, 1, Some(10));
    observe(&mut capture, 3, DState::Developing, -1, None);
    observe(&mut capture, 4, DState::Burst, 2, Some(20));
    observe(&mut capture, 5, DState::DecayRisk, 2, Some(20));
    observe(&mut capture, 6, DState::Burst, 2, Some(20));
    capture.finish();

    let counts = capture.anchor_counts();
    assert_eq!(counts["STRONG"], 2);
    assert_eq!(counts["WEAK"], 2);
    assert_eq!(counts["DECAY_RISK"], 1);
    assert_eq!(capture.episode_count(), 2);
}

#[test]
fn first_observed_burst_is_a_strong_anchor() {
    let mut capture = Capture::default();
    observe(&mut capture, 1, DState::Burst, 7, Some(1));
    capture.finish();
    assert_eq!(capture.anchor_counts()["STRONG"], 1);
    assert_eq!(capture.anchor_counts()["WEAK"], 1);
}
