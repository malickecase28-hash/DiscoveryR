use research_contracts::submission::{publish_atomic_no_replace, reject_unsafe_ancestors};
use research_contracts::{ContractError, KnowledgeEnvelope, KnowledgeQuery, KnowledgeRecord};
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct KnowledgeStore {
    records: BTreeMap<String, KnowledgeEnvelope>,
}

impl KnowledgeStore {
    pub fn append(&mut self, envelope: KnowledgeEnvelope) -> Result<(), ContractError> {
        envelope.validate()?;
        let record_id = match &envelope.record {
            KnowledgeRecord::Question(record) => &record.record_id,
            KnowledgeRecord::Finding(record) => &record.record_id,
            KnowledgeRecord::Challenge(record) => &record.record_id,
            KnowledgeRecord::Knowledge(record) => &record.record_id,
        };
        if record_id != &envelope.knowledge_id {
            return Err(ContractError::Invalid(
                "knowledge_id must match embedded record_id".into(),
            ));
        }
        match self.records.entry(envelope.knowledge_id.clone()) {
            Entry::Vacant(slot) => {
                slot.insert(envelope);
                Ok(())
            }
            Entry::Occupied(_) => Err(ContractError::Invalid(
                "knowledge identity already exists".into(),
            )),
        }
    }

    pub fn get(&self, knowledge_id: &str) -> Option<&KnowledgeEnvelope> {
        self.records.get(knowledge_id)
    }

    pub fn query(&self, query: &KnowledgeQuery) -> Vec<&KnowledgeEnvelope> {
        self.records
            .values()
            .filter(|record| query.matches(record))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Durable append-only knowledge publication with restart recovery. Each
/// record has its own no-replace file, so corrections are new records linked
/// by the record's supersession fields.
#[derive(Debug)]
pub struct DurableKnowledgeStore {
    root: PathBuf,
    memory: KnowledgeStore,
}

impl DurableKnowledgeStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ContractError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).map_err(|error| ContractError::Invalid(error.to_string()))?;
        reject_unsafe_ancestors(&root)
            .map_err(|error| ContractError::Invalid(error.to_string()))?;
        let mut memory = KnowledgeStore::default();
        for entry in
            fs::read_dir(&root).map_err(|error| ContractError::Invalid(error.to_string()))?
        {
            let path = entry
                .map_err(|error| ContractError::Invalid(error.to_string()))?
                .path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            reject_unsafe_ancestors(&path)
                .map_err(|error| ContractError::Invalid(error.to_string()))?;
            let envelope = serde_json::from_slice(
                &fs::read(&path).map_err(|error| ContractError::Invalid(error.to_string()))?,
            )
            .map_err(|error| ContractError::Invalid(error.to_string()))?;
            memory.append(envelope)?;
        }
        Ok(Self { root, memory })
    }

    pub fn append(&mut self, envelope: KnowledgeEnvelope) -> Result<(), ContractError> {
        envelope.validate()?;
        let bytes = serde_json::to_vec_pretty(&envelope)
            .map_err(|error| ContractError::Invalid(error.to_string()))?;
        let id = envelope.knowledge_id.clone();
        let record_id = match &envelope.record {
            KnowledgeRecord::Question(record) => &record.record_id,
            KnowledgeRecord::Finding(record) => &record.record_id,
            KnowledgeRecord::Challenge(record) => &record.record_id,
            KnowledgeRecord::Knowledge(record) => &record.record_id,
        };
        if record_id != &id {
            return Err(ContractError::Invalid(
                "knowledge_id must match embedded record_id".into(),
            ));
        }
        if self.memory.get(&id).is_some() {
            return Err(ContractError::Invalid(
                "knowledge identity already exists".into(),
            ));
        }
        publish_atomic_no_replace(&self.root, &format!("{id}.json"), &bytes)
            .map_err(|error| ContractError::Invalid(error.to_string()))?;
        self.memory.append(envelope)
    }

    pub fn query(&self, query: &KnowledgeQuery) -> Vec<&KnowledgeEnvelope> {
        self.memory.query(query)
    }

    pub fn query_as_of(&self, query: &KnowledgeQuery, as_of_utc: &str) -> Vec<&KnowledgeEnvelope> {
        self.memory
            .query(query)
            .into_iter()
            .filter(|envelope| {
                record_created_utc(envelope).is_some_and(|created| created <= as_of_utc)
            })
            .collect()
    }
}

fn record_created_utc(envelope: &KnowledgeEnvelope) -> Option<&str> {
    match &envelope.record {
        KnowledgeRecord::Question(record) => Some(&record.created_utc),
        KnowledgeRecord::Finding(record) => Some(&record.created_utc),
        KnowledgeRecord::Challenge(record) => Some(&record.created_utc),
        KnowledgeRecord::Knowledge(record) => Some(&record.created_utc),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_contracts::{
        BarScale, KnowledgeFacets, KnowledgeRecord, NativeScale, Provenance, QuestionRecord,
        QuestionStatus,
    };
    use std::collections::BTreeSet;

    fn record(id: &str, instrument: &str) -> KnowledgeEnvelope {
        KnowledgeEnvelope {
            envelope_version: 1,
            knowledge_id: id.into(),
            record: KnowledgeRecord::Question(QuestionRecord {
                record_id: id.into(),
                content: serde_json::json!({"instrument": instrument}),
                status: QuestionStatus::Open,
                provenance: Provenance {
                    experiment_id: None,
                    run_id: None,
                    evidence_ids: vec![],
                    code_identity: Some("code-v1".into()),
                    researcher_id: None,
                },
                created_utc: "2026-01-01T00:00:00Z".into(),
                supersedes: None,
                superseded_by: None,
            }),
            facets: KnowledgeFacets {
                instrument_ids: vec![instrument.into()],
                detector_ids: vec!["detector".into()],
                anchor_ids: vec!["anchor".into()],
                lifecycle_states: vec!["forming".into()],
                context_ids: vec!["context".into()],
                native_scales: BTreeSet::from([NativeScale::Bar(BarScale::M15)]),
                evidence_ids: vec!["evidence".into()],
                strategy_ids: vec![],
                portfolio_ids: vec![],
            },
        }
    }

    #[test]
    fn append_is_immutable_and_queryable() {
        let mut store = KnowledgeStore::default();
        store.append(record("k1", "xauusd")).unwrap();
        assert_eq!(store.len(), 1);
        let original = store.get("k1").unwrap().knowledge_id.clone();
        assert!(store.append(record("k1", "xauusd")).is_err());
        assert_eq!(store.get("k1").unwrap().knowledge_id, original);
        let matches = store.query(&KnowledgeQuery {
            instrument_id: Some("xauusd".into()),
            ..KnowledgeQuery::default()
        });
        assert_eq!(matches.len(), 1);
    }
}
