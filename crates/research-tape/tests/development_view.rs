use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use research_tape::development_view;
use research_tape::development_view::{
    classify_batch, filter_batch, hash_file, hash_view_identity, Classification,
    ExpectedDevelopmentScope, Gate, GateStats, Manifest, ManifestFile, RowStats,
};
use std::{io::Write, sync::Arc};

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
        std::slice::from_ref(&stats),
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
    let head = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert!(development_view::validate_code_identity(head.trim()).is_ok());
    assert!(
        development_view::validate_code_identity("0123456789012345678901234567890123456789")
            .is_err()
    );
    assert!(development_view::validate_code_identity("not-a-sha").is_err());
}

fn valid_view(root: &std::path::Path) -> Manifest {
    std::fs::create_dir_all(root).unwrap();
    let path = root.join("bars.parquet");
    let schema = Arc::new(Schema::new(vec![Field::new(
        "bar_close_ts",
        DataType::Int64,
        false,
    )]));
    let batch =
        RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1]))]).unwrap();
    let mut writer =
        ArrowWriter::try_new(std::fs::File::create(&path).unwrap(), schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    let hash = hash_file(&path).unwrap();
    let stats = RowStats::Bar {
        rows: 1,
        bar_close_min: Some(1),
        bar_close_max: Some(1),
    };
    let code = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let source_id = "a".repeat(64);
    let payload_id = "b".repeat(64);
    let identity = hash_view_identity(
        "scope",
        &source_id,
        &payload_id,
        B,
        &["bars.parquet"],
        &[stats],
        &[&hash],
        &["COPIED_FULL_DEVELOPMENT"],
        &code,
        0,
    );
    let manifest = Manifest {
        schema_version: 1,
        scope_id: "scope".into(),
        instrument: "XAUUSD".into(),
        source_manifest_identity: source_id,
        payload_manifest_identity: payload_id,
        boundary_rule: "x".into(),
        boundary_ms: Some(B),
        development_end_exclusive: "x".into(),
        builder_code_identity: code,
        source_groups: std::collections::BTreeMap::from([(
            "m1".into(),
            vec![ManifestFile {
                logical_path: "bars.parquet".into(),
                materialization_kind: "COPIED_FULL_DEVELOPMENT".into(),
                development_row_count: 1,
                bar_close_min: Some(1),
                bar_close_max: Some(1),
                event_min: None,
                event_max: None,
                received_min: None,
                received_max: None,
                sha256: Some(hash),
                source_part: None,
            }],
        )]),
        logical_view_identity: identity,
    };
    serde_json::to_writer(
        std::fs::File::create(root.join("view_manifest.json")).unwrap(),
        &manifest,
    )
    .unwrap();
    manifest
}

#[test]
fn forged_builder_code_identity_is_rejected() {
    let root = std::env::temp_dir().join(format!("dev_view_code_{}", std::process::id()));
    let mut manifest = valid_view(&root);
    let other_code = "0".repeat(40);
    manifest.builder_code_identity = other_code.clone();
    manifest.logical_view_identity = hash_view_identity(
        &manifest.scope_id,
        &manifest.source_manifest_identity,
        &manifest.payload_manifest_identity,
        B,
        &["bars.parquet"],
        &[RowStats::Bar {
            rows: 1,
            bar_close_min: Some(1),
            bar_close_max: Some(1),
        }],
        &[&manifest.source_groups["m1"][0].sha256.clone().unwrap()],
        &["COPIED_FULL_DEVELOPMENT"],
        &other_code,
        0,
    );
    assert!(development_view::verify_view(&root, &manifest, 64).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn payload_hash_tampering_fails_verification() {
    let root = std::env::temp_dir().join(format!("dev_view_hash_{}", std::process::id()));
    let manifest = valid_view(&root);
    std::fs::OpenOptions::new()
        .append(true)
        .open(root.join("bars.parquet"))
        .unwrap()
        .write_all(b"tampered")
        .unwrap();
    assert!(development_view::verify_view(&root, &manifest, 64).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn manifest_identity_mismatch_fails_verification() {
    let root = std::env::temp_dir().join(format!("dev_view_identity_{}", std::process::id()));
    let mut manifest = valid_view(&root);
    manifest.logical_view_identity = "wrong".into();
    assert!(development_view::verify_view(&root, &manifest, 64).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn revoked_view_fails_verification() {
    let root = std::env::temp_dir().join(format!("dev_view_revoked_{}", std::process::id()));
    let manifest = valid_view(&root);
    std::fs::write(root.join("REVOKED"), b"revoked").unwrap();
    assert!(development_view::verify_view(&root, &manifest, 64).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn malformed_received_time_fails_exposed_view_verification() {
    use parquet::arrow::ArrowWriter;
    use std::fs::File;
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
        boundary_ms: Some(B),
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
    let _ = std::fs::remove_dir_all(root);
}

fn expected_scope(manifest: &Manifest) -> ExpectedDevelopmentScope {
    ExpectedDevelopmentScope {
        instrument: manifest.instrument.clone(),
        scope_id: manifest.scope_id.clone(),
        source_manifest_identity: manifest.source_manifest_identity.clone(),
        payload_manifest_identity: manifest.payload_manifest_identity.clone(),
        boundary_ms: B,
        development_end_exclusive: manifest.development_end_exclusive.clone(),
    }
}

#[test]
fn verification_rejects_mismatched_expected_scope() {
    let root = std::env::temp_dir().join(format!("dev_view_scope_{}", std::process::id()));
    let manifest = valid_view(&root);
    let mut expected = expected_scope(&manifest);
    expected.instrument = "OTHER".into();
    assert!(development_view::verify_view_with_expected(&root, &manifest, 64, &expected).is_err());
    expected.instrument = manifest.instrument.clone();
    expected.scope_id = "OTHER_SCOPE".into();
    assert!(development_view::verify_view_with_expected(&root, &manifest, 64, &expected).is_err());
    expected.scope_id = manifest.scope_id.clone();
    expected.source_manifest_identity = "c".repeat(64);
    assert!(development_view::verify_view_with_expected(&root, &manifest, 64, &expected).is_err());
    expected.source_manifest_identity = manifest.source_manifest_identity.clone();
    expected.payload_manifest_identity = "d".repeat(64);
    assert!(development_view::verify_view_with_expected(&root, &manifest, 64, &expected).is_err());
    expected.payload_manifest_identity = manifest.payload_manifest_identity.clone();
    expected.boundary_ms += 1;
    assert!(development_view::verify_view_with_expected(&root, &manifest, 64, &expected).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn verification_rejects_reparse_root_and_ancestor() {
    use std::os::unix::fs::symlink;
    let outer = std::env::temp_dir().join(format!("dev_view_reparse_{}", std::process::id()));
    let real = outer.join("real");
    let alias = outer.join("alias");
    let nested = outer.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let manifest = valid_view(&real);
    symlink(&real, &alias).unwrap();
    let expected = expected_scope(&manifest);
    assert!(development_view::verify_view_with_expected(&alias, &manifest, 64, &expected).is_err());
    symlink(&outer, nested.join("redirect")).unwrap();
    let redirected = nested.join("redirect/real");
    assert!(
        development_view::verify_view_with_expected(&redirected, &manifest, 64, &expected).is_err()
    );
    let _ = std::fs::remove_dir_all(outer);
}

#[cfg(windows)]
#[test]
fn verification_rejects_reparse_root_and_ancestor() {
    use std::os::windows::fs::symlink_dir;
    let outer = std::env::temp_dir().join(format!("dev_view_reparse_{}", std::process::id()));
    let real = outer.join("real");
    let alias = outer.join("alias");
    let nested = outer.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let manifest = valid_view(&real);
    if symlink_dir(&real, &alias).is_err() {
        let _ = std::fs::remove_dir_all(outer);
        return;
    }
    let expected = expected_scope(&manifest);
    assert!(development_view::verify_view_with_expected(&alias, &manifest, 64, &expected).is_err());
    if symlink_dir(&outer, nested.join("redirect")).is_err() {
        let _ = std::fs::remove_dir_all(outer);
        return;
    }
    let redirected = nested.join("redirect/real");
    assert!(
        development_view::verify_view_with_expected(&redirected, &manifest, 64, &expected).is_err()
    );
    let _ = std::fs::remove_dir_all(outer);
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
