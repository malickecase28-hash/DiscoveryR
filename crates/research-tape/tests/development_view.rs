#[path = "../src/development_view.rs"]
mod development_view;

use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use development_view::{
    classify_batch, filter_batch, hash_view_identity, Classification, Gate, GateStats, RowStats,
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
    let stats = RowStats::Bar {
        rows: 1,
        bar_close_min: Some(1),
        bar_close_max: Some(1),
    };
    let a = hash_view_identity(
        "scope",
        "source",
        "payload",
        B,
        &["x"],
        &[stats.clone()],
        &["h"],
        &["FILTERED_MIXED_PART"],
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
        &["FILTERED_MIXED_PART"],
        "commit",
        99,
    );
    assert_eq!(a, b);
    assert_ne!(
        a,
        hash_view_identity(
            "scope",
            "source",
            "payload",
            B,
            &["x"],
            &[RowStats::Bar {
                rows: 1,
                bar_close_min: Some(1),
                bar_close_max: Some(1)
            }],
            &["h"],
            &["HARDLINK_FULL_DEVELOPMENT"],
            "commit",
            99
        )
    );
}

#[test]
fn code_identity_is_exactly_a_git_sha() {
    assert!(
        development_view::validate_code_identity("0123456789012345678901234567890123456789")
            .is_ok()
    );
    assert!(development_view::validate_code_identity("not-a-sha").is_err());
}

#[test]
fn malformed_received_time_fails_exposed_view_verification() {
    use parquet::arrow::ArrowWriter;
    use std::{fs::File, path::PathBuf};
    let root = std::env::temp_dir().join(format!("dev_view_verify_{}", std::process::id()));
    std::fs::create_dir_all(root.join("tick")).unwrap();
    let path = root.join("tick/part.parquet");
    let schema = Arc::new(Schema::new(vec![
        Field::new("event_ts_ns", DataType::Int64, false),
        Field::new("received_ts_ns", DataType::Int64, false),
    ]));
    let boundary_ns = B * 1_000_000;
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![boundary_ns - 10])),
            Arc::new(Int64Array::from(vec![boundary_ns + 10])),
        ],
    )
    .unwrap();
    let mut writer = ArrowWriter::try_new(File::create(&path).unwrap(), schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    std::fs::write(root.join("view_manifest.json"), "{}").unwrap();
    let manifest = development_view::Manifest {
        schema_version: 1,
        scope_id: "x".into(),
        instrument: "XAUUSD".into(),
        source_manifest_identity: "x".into(),
        payload_manifest_identity: "x".into(),
        boundary_rule: "x".into(),
        development_end_exclusive: "x".into(),
        builder_code_identity: "x".into(),
        source_groups: std::collections::BTreeMap::from([(
            String::from("tick"),
            vec![development_view::ManifestFile {
                logical_path: "tick/part.parquet".into(),
                materialization_kind: "FILTERED_MIXED_PART".into(),
                development_row_count: 1,
                bar_close_min: None,
                bar_close_max: None,
                event_min: Some(boundary_ns - 10),
                event_max: Some(boundary_ns - 10),
                received_min: Some(boundary_ns + 10),
                received_max: Some(boundary_ns + 10),
                sha256: None,
                source_part: None,
            }],
        )]),
        logical_view_identity: "x".into(),
    };
    assert!(development_view::verify_view(&root, &manifest, 64).is_err());
    let _ = std::fs::remove_dir_all(PathBuf::from(root));
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

#[test]
fn tick_stats_keep_event_and_received_ranges_separate() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("event_ts_ns", DataType::Int64, false),
        Field::new("received_ts_ns", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![1, 2])),
            Arc::new(Int64Array::from(vec![3, 4])),
        ],
    )
    .unwrap();
    let (_, stats) = classify_batch(&batch, Gate::Tick { boundary_ns: 10 }).unwrap();
    assert_eq!(
        stats,
        GateStats::Tick {
            rows: 2,
            event_min: Some(1),
            event_max: Some(2),
            received_min: Some(3),
            received_max: Some(4)
        }
    );
}
