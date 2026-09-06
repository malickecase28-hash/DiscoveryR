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
        let Some(as_of) = parse_utc(as_of_utc) else {
            return Vec::new();
        };
        self.memory
            .query(query)
            .into_iter()
            .filter(|envelope| {
                record_created_utc(envelope)
                    .and_then(parse_utc)
                    .is_some_and(|created| created <= as_of)
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

fn parse_utc(value: &str) -> Option<i128> {
    if value.len() < 20
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
        || value.as_bytes().get(10) != Some(&b'T')
    {
        return None;
    }
    let year: i128 = value[0..4].parse().ok()?;
    let month: i128 = value[5..7].parse().ok()?;
    let day: i128 = value[8..10].parse().ok()?;
    let hour: i128 = value[11..13].parse().ok()?;
    let minute: i128 = value[14..16].parse().ok()?;
    let second: i128 = value[17..19].parse().ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let mut end = 19;
    let fraction = if value.as_bytes().get(end) == Some(&b'.') {
        end += 1;
        let start = end;
        while value
            .as_bytes()
            .get(end)
            .is_some_and(|b| b.is_ascii_digit())
        {
            end += 1;
        }
        if start == end {
            return None;
        }
        let digits = &value[start..end];
        let mut nanos: i128 = digits.parse().ok()?;
        for _ in digits.len()..9 {
            nanos *= 10;
        }
        nanos
    } else {
        0
    };
    let offset = match value.as_bytes().get(end) {
        Some(b'Z') if end + 1 == value.len() => 0,
        Some(b'+') | Some(b'-') => {
            let sign = if value.as_bytes()[end] == b'+' { 1 } else { -1 };
            if value.len() != end + 6 || value.as_bytes()[end + 3] != b':' {
                return None;
            }
            let hours: i128 = value[end + 1..end + 3].parse().ok()?;
            let minutes: i128 = value[end + 4..end + 6].parse().ok()?;
            if hours > 23 || minutes > 59 {
                return None;
            }
            sign * (hours * 3600 + minutes * 60)
        }
        _ => return None,
    };
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some((days * 86_400 + hour * 3600 + minute * 60 + second - offset) * 1_000_000_000 + fraction)
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
