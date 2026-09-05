use research_contracts::{ContractError, KnowledgeEnvelope, KnowledgeQuery};
use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Debug, Default)]
pub struct KnowledgeStore {
    records: BTreeMap<String, KnowledgeEnvelope>,
}

impl KnowledgeStore {
    pub fn append(&mut self, envelope: KnowledgeEnvelope) -> Result<(), ContractError> {
        envelope.validate()?;
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
