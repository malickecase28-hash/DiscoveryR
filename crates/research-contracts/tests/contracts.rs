use research_contracts::{
    generated_schemas, parse_and_validate_contract, parse_and_validate_registry,
    validate_context_available, BarScale, KnowledgeRecord, NativeScaleScope,
};
use serde_json::Value;

#[test]
fn example_contract_is_valid() {
    let json = include_str!("../../../templates/experiment_contract.example.json");
    assert!(parse_and_validate_contract(json).is_ok());
}

#[test]
fn future_context_is_rejected() {
    assert!(validate_context_available(100, 101).is_err());
}

#[test]
fn registry_is_complete_and_typed() {
    let entries =
        parse_and_validate_registry(include_str!("../../../registry/detectors_v1.json")).unwrap();
    assert_eq!(entries.len(), 23);
    assert_eq!(entries.iter().filter(|e| e.domain == "bar").count(), 16);
    assert_eq!(entries.iter().filter(|e| e.domain == "tick").count(), 7);
    let expected = NativeScaleScope::Explicit {
        scales: vec![
            BarScale::S15,
            BarScale::S30,
            BarScale::M1,
            BarScale::M5,
            BarScale::M15,
            BarScale::H1,
            BarScale::H4,
        ],
    };
    assert!(entries
        .iter()
        .filter(|e| e.domain == "bar")
        .all(|e| e.native_scales == expected));
    assert!(entries
        .iter()
        .filter(|e| e.domain == "tick")
        .all(|e| e.native_scales == NativeScaleScope::Tick));
}

#[test]
fn generated_schemas_match_committed_files() {
    for (name, schema) in generated_schemas() {
        let committed = match name {
            "experiment_contract_v1.schema.json" => {
                include_str!("../../../contracts/experiment_contract_v1.schema.json")
            }
            "run_manifest_v1.schema.json" => {
                include_str!("../../../contracts/run_manifest_v1.schema.json")
            }
            "detector_registry_entry_v1.schema.json" => {
                include_str!("../../../contracts/detector_registry_entry_v1.schema.json")
            }
            "knowledge_record_v1.schema.json" => {
                include_str!("../../../contracts/knowledge_record_v1.schema.json")
            }
            _ => unreachable!(),
        };
        assert_eq!(schema, committed, "schema drift: {name}");
    }
}

#[test]
fn contract_rejects_unknown_top_level_and_nested_fields() {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../templates/experiment_contract.example.json"
    ))
    .unwrap();
    value["provider"] = Value::String("some-model".into());
    assert!(parse_and_validate_contract(&value.to_string()).is_err());
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../templates/experiment_contract.example.json"
    ))
    .unwrap();
    value["anchor"]["provider"] = Value::String("some-model".into());
    assert!(parse_and_validate_contract(&value.to_string()).is_err());
}

#[test]
fn knowledge_record_is_tagged_and_type_safe() {
    let finding: KnowledgeRecord = serde_json::from_str(r#"{"record_type":"FINDING","record_id":"F-1","content":null,"status":"NULL","provenance":{"evidence_ids":[]},"created_utc":"t"}"#).unwrap();
    assert!(matches!(finding, KnowledgeRecord::Finding(_)));
    assert!(serde_json::from_str::<KnowledgeRecord>(r#"{"record_type":"QUESTION","record_id":"Q-1","content":null,"status":"CONFIRMED","provenance":{"evidence_ids":[]},"created_utc":"t"}"#).is_err());
    assert!(serde_json::from_str::<KnowledgeRecord>(r#"{"record_type":"CHALLENGE","record_id":"C-1","content":null,"decision":"SURVIVED","provenance":{"evidence_ids":[]},"created_utc":"t"}"#).is_ok());
}
