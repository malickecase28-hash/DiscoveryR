#[path = "../src/development_view.rs"]
mod development_view;

use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use development_view::{
    classify_batch, filter_batch, hash_view_identity, Classification, Gate, RowStats,
};
use std::sync::Arc;

const B: i64 = 1_777_984_740_000;

fn bars(values: Vec<Option<i64>>) -> RecordBatch {
    let n = values.len();
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("bar_open_ts", DataType::Int64, true),
            Field::new("bar_close_ts", DataType::Int64, true),
            Field::new("payload", DataType::Utf8, true),
            Field::new("value", DataType::Float64, false),
        ])),
        vec![
            Arc::new(Int64Array::from(
                (0..n).map(|i| Some(i as i64)).collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(values)),
            Arc::new(StringArray::from(vec![Some("bytes"); n])),
            Arc::new(Float64Array::from(vec![1.0; n])),
        ],
    )
    .unwrap()
}

#[test]
fn bar_filter_is_end_exclusive_and_null_is_fatal() {
    let batch = bars(vec![Some(B - 1), Some(B), Some(B + 1)]);
    assert_eq!(
        classify_batch(&batch, Gate::Bar { boundary_ms: B })
            .unwrap()
            .0,
        Classification::Mixed
    );
    let filtered = filter_batch(&batch, Gate::Bar { boundary_ms: B })
        .unwrap()
        .unwrap();
    assert_eq!(filtered.num_rows(), 1);
    assert_eq!(
        filtered
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        B - 1
    );
    assert!(classify_batch(&bars(vec![None]), Gate::Bar { boundary_ms: B }).is_err());
}

#[test]
fn identity_is_deterministic_and_runtime_independent() {
    let stats = RowStats {
        rows: 1,
        gate_min: Some(1),
        gate_max: Some(1),
    };
    let a = hash_view_identity(
        "scope",
        "source",
        "payload",
        B,
        &["x"],
        &[stats.clone()],
        &["h"],
        "commit",
        10,
    );
    let b = hash_view_identity(
        "scope",
        "source",
        "payload",
        B,
        &["x"],
        &[stats],
        &["h"],
        "commit",
        99,
    );
    assert_eq!(a, b);
}

#[test]
fn tick_requires_both_physical_times_before_boundary() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("event_ts_ns", DataType::Int64, true),
        Field::new("received_ts_ns", DataType::Int64, true),
        Field::new("payload", DataType::Utf8, true),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![
                Some(B - 1),
                Some(B - 1),
                Some(B),
                Some(B - 1),
            ])),
            Arc::new(Int64Array::from(vec![
                Some(B - 1),
                Some(B),
                Some(B - 1),
                None,
            ])),
            Arc::new(StringArray::from(vec![
                Some("a"),
                Some("b"),
                Some("c"),
                Some("d"),
            ])),
        ],
    )
    .unwrap();
    let gate = Gate::Tick { boundary_ns: B };
    assert_eq!(
        classify_batch(&batch, gate).unwrap_err(),
        "null gating time at row 3"
    );
    let batch = batch.slice(0, 3);
    let filtered = filter_batch(&batch, gate).unwrap().unwrap();
    assert_eq!(filtered.num_rows(), 1);
    assert_eq!(
        filtered
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .value(0),
        "a"
    );
}
