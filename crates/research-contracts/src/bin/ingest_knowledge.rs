use research_contracts::{
    validate_knowledge_record, AcceptedKnowledgeRecord, ChallengeRecord, FindingRecord,
    KnowledgeRecord, QuestionRecord,
};
use serde_json::Value;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process,
};

fn record_id(record: &KnowledgeRecord) -> &str {
    match record {
        KnowledgeRecord::Question(QuestionRecord { record_id, .. })
        | KnowledgeRecord::Finding(FindingRecord { record_id, .. })
        | KnowledgeRecord::Challenge(ChallengeRecord { record_id, .. })
        | KnowledgeRecord::Knowledge(AcceptedKnowledgeRecord { record_id, .. }) => record_id,
    }
}

fn reject_operational_identity(value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(
                    key.to_ascii_lowercase().as_str(),
                    "provider"
                        | "provider_id"
                        | "provider_identity"
                        | "model"
                        | "model_id"
                        | "model_identity"
                ) {
                    return Err(format!("provider/model identity field is forbidden: {key}").into());
                }
                reject_operational_identity(value)?;
            }
        }
        Value::Array(values) => values.iter().try_for_each(reject_operational_identity)?,
        _ => {}
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().enumerate().all(|(i, byte)| {
            byte.is_ascii_alphanumeric() || (i > 0 && matches!(byte, b'.' | b'_' | b'-'))
        })
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .ok_or("usage: ingest_knowledge <record.json> [knowledge/records]")?;
    let records_dir = args.next().unwrap_or_else(|| "knowledge/records".into());
    if args.next().is_some() {
        return Err("usage: ingest_knowledge <record.json> [knowledge/records]".into());
    }
    let raw: Value = serde_json::from_str(&fs::read_to_string(input)?)?;
    reject_operational_identity(&raw)?;
    let record: KnowledgeRecord = serde_json::from_value(raw)?;
    validate_knowledge_record(&record)?;
    let id = record_id(&record);
    if !valid_id(id) {
        return Err("record_id has an unsafe filename grammar".into());
    }
    fs::create_dir_all(&records_dir)?;
    let path = Path::new(&records_dir).join(format!("{id}.json"));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    serde_json::to_writer_pretty(&mut output, &record)?;
    output.write_all(b"\n")?;
    output.sync_all()?;
    println!("ingested {id}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("knowledge ingestion failed: {error}");
        process::exit(1);
    }
}
