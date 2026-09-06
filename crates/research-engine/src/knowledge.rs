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
        validate_links(&self.records, &envelope)?;
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
        let mut envelopes = Vec::new();
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
            envelopes.push(envelope);
        }
        let mut pending = envelopes;
        while !pending.is_empty() {
            let mut next = Vec::new();
            let mut progressed = false;
            for envelope in pending {
                let predecessor_ready = record_links(&envelope)
                    .1
                    .is_none_or(|id| memory.get(id).is_some());
                if predecessor_ready {
                    memory.append(envelope)?;
                    progressed = true;
                } else {
                    next.push(envelope);
                }
            }
            if !progressed {
                return Err(ContractError::Invalid(
                    "knowledge supersession chain has an absent predecessor".into(),
                ));
            }
            pending = next;
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
        validate_links(&self.memory.records, &envelope)?;
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
        self.query_as_of_checked(query, as_of_utc)
            .unwrap_or_default()
    }

    pub fn query_as_of_checked(
        &self,
        query: &KnowledgeQuery,
        as_of_utc: &str,
    ) -> Result<Vec<&KnowledgeEnvelope>, ContractError> {
        let as_of = parse_utc(as_of_utc)
            .ok_or_else(|| ContractError::Invalid("invalid RFC3339 UTC value".into()))?;
        Ok(self
            .memory
            .query(query)
            .into_iter()
            .filter(|envelope| {
                record_created_utc(envelope)
                    .and_then(parse_utc)
                    .is_some_and(|created| created <= as_of)
            })
            .collect())
    }
}

fn record_links(envelope: &KnowledgeEnvelope) -> (&str, Option<&str>, Option<&str>) {
    match &envelope.record {
        KnowledgeRecord::Question(record) => (
            &record.record_id,
            record.supersedes.as_deref(),
            record.superseded_by.as_deref(),
        ),
        KnowledgeRecord::Finding(record) => (
            &record.record_id,
            record.supersedes.as_deref(),
            record.superseded_by.as_deref(),
        ),
        KnowledgeRecord::Challenge(record) => (
            &record.record_id,
            record.supersedes.as_deref(),
            record.superseded_by.as_deref(),
        ),
        KnowledgeRecord::Knowledge(record) => (
            &record.record_id,
            record.supersedes.as_deref(),
            record.superseded_by.as_deref(),
        ),
    }
}

fn validate_links(
    records: &BTreeMap<String, KnowledgeEnvelope>,
    envelope: &KnowledgeEnvelope,
) -> Result<(), ContractError> {
    let (_, supersedes, superseded_by) = record_links(envelope);
    if let Some(previous_id) = supersedes {
        let previous = records
            .get(previous_id)
            .ok_or_else(|| ContractError::Invalid("supersedes predecessor is absent".into()))?;
        if previous.facets != envelope.facets
            || record_links(previous)
                .2
                .is_some_and(|id| id != envelope.knowledge_id.as_str())
        {
            return Err(ContractError::Invalid(
                "supersession link is not reciprocal and compatible".into(),
            ));
        }
    }
    let _ = superseded_by; // forward links are descriptive; `supersedes` is the authoritative predecessor edge.
    Ok(())
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
    if !value.is_ascii()
        || value.len() < 20
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
        || value.as_bytes().get(10) != Some(&b'T')
    {
        return None;
    }
    if value.as_bytes().get(13) != Some(&b':') || value.as_bytes().get(16) != Some(&b':') {
        return None;
    }
    let year: i128 = value[0..4].parse().ok()?;
    let month: i128 = value[5..7].parse().ok()?;
    let day: i128 = value[8..10].parse().ok()?;
    let hour: i128 = value[11..13].parse().ok()?;
    let minute: i128 = value[14..16].parse().ok()?;
    let second: i128 = value[17..19].parse().ok()?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month)
        || day < 1
        || day > month_days[(month - 1) as usize]
        || hour > 23
        || minute > 59
        || second > 59
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
        if start == end || end - start > 9 {
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
            if hours > 23 || minutes > 59 || (hours == 23 && minutes != 59) {
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

    #[test]
    fn as_of_parser_rejects_invalid_dates_unicode_and_long_fractions() {
        let store = DurableKnowledgeStore::open(
            std::env::temp_dir().join(format!("rsp-knowledge-{}", std::process::id())),
        )
        .unwrap();
        let query = KnowledgeQuery::default();
        for value in [
            "2026-02-31T00:00:00Z",
            "2026-01-01T00:00:00.1234567890Z",
            "２０２６-01-01T00:00:00Z",
            "2026-01-01T00:00:00+24:00",
        ] {
            assert!(
                store.query_as_of_checked(&query, value).is_err(),
                "accepted invalid UTC value: {value}"
            );
        }
    }

    #[test]
    fn supersession_requires_existing_compatible_reciprocal_records() {
        let mut store = KnowledgeStore::default();
        let mut predecessor = record("k1", "xauusd");
        if let KnowledgeRecord::Question(question) = &mut predecessor.record {
            question.superseded_by = Some("k2".into());
        }
        let mut successor = record("k2", "xauusd");
        if let KnowledgeRecord::Question(question) = &mut successor.record {
            question.supersedes = Some("k1".into());
        }
        store.append(predecessor).unwrap();
        store.append(successor).unwrap();

        let mut orphan = record("k3", "xauusd");
        if let KnowledgeRecord::Question(question) = &mut orphan.record {
            question.supersedes = Some("missing".into());
        }
        assert!(store.append(orphan).is_err());
    }
}
