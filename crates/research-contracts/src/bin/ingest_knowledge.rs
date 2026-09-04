use research_contracts::{
    validate_knowledge_record, AcceptedKnowledgeRecord, ChallengeRecord, FindingRecord,
    KnowledgeRecord, QuestionRecord,
};
use serde_json::Value;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
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
        Value::Array(values) => {
            for value in values {
                reject_operational_identity(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .ok_or("usage: ingest_knowledge <record.json> [knowledge/records.jsonl]")?;
    let store = args
        .next()
        .unwrap_or_else(|| "knowledge/records.jsonl".into());
    if args.next().is_some() {
        return Err("usage: ingest_knowledge <record.json> [knowledge/records.jsonl]".into());
    }
    let record: KnowledgeRecord = serde_json::from_str(&fs::read_to_string(input)?)?;
    reject_operational_identity(&serde_json::to_value(&record)?)?;
    validate_knowledge_record(&record)?;
    let id = record_id(&record);
    if id.is_empty() {
        return Err("record_id cannot be empty".into());
    }
    if Path::new(&store).exists() {
        for line in BufReader::new(fs::File::open(&store)?).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let existing: KnowledgeRecord = serde_json::from_str(&line)?;
            if record_id(&existing) == id {
                return Err(format!("record_id already exists: {id}").into());
            }
        }
    } else if let Some(parent) = Path::new(&store).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = OpenOptions::new().create(true).append(true).open(&store)?;
    serde_json::to_writer(&mut output, &record)?;
    writeln!(output)?;
    println!("ingested {id}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("knowledge ingestion failed: {error}");
        process::exit(1);
    }
}
