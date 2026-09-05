use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::ContractError;

pub(crate) fn validate_identity(value: &str, field: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::Invalid(format!("{field} cannot be empty")));
    }
    let absolute_windows = value.as_bytes().get(1) == Some(&b':');
    if value.starts_with('/') || value.starts_with('\\') || absolute_windows {
        return Err(ContractError::Invalid(format!(
            "{field} cannot contain an absolute path"
        )));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(ContractError::Invalid(format!(
            "{field} must be a path-independent identity"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ObjectIdentity {
    pub object_id: String,
    pub detector_id: String,
    pub source_identity: String,
    pub scope_identity: String,
    pub parent_object_ids: Vec<String>,
}

impl ObjectIdentity {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.object_id, "object_id")?;
        validate_identity(&self.detector_id, "detector_id")?;
        validate_identity(&self.source_identity, "source_identity")?;
        validate_identity(&self.scope_identity, "scope_identity")?;
        let mut parents = HashSet::new();
        for parent in &self.parent_object_ids {
            validate_identity(parent, "parent_object_id")?;
            if parent == &self.object_id || !parents.insert(parent) {
                return Err(ContractError::Invalid(format!(
                    "invalid parent object for {}",
                    self.object_id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct DetectorLineage {
    pub detector_id: String,
    pub parent_detector_ids: Vec<String>,
}

impl DetectorLineage {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.detector_id, "detector_id")?;
        let mut parents = HashSet::new();
        for parent in &self.parent_detector_ids {
            validate_identity(parent, "parent_detector_id")?;
            if parent == &self.detector_id || !parents.insert(parent) {
                return Err(ContractError::Invalid(format!(
                    "invalid parent detector for {}",
                    self.detector_id
                )));
            }
        }
        Ok(())
    }
}

pub fn validate_detector_lineage(lineage: &[DetectorLineage]) -> Result<(), ContractError> {
    for entry in lineage {
        entry.validate()?;
    }
    let ids: HashSet<&str> = lineage
        .iter()
        .map(|entry| entry.detector_id.as_str())
        .collect();
    if ids.len() != lineage.len() {
        return Err(ContractError::Invalid(
            "duplicate detector lineage identity".into(),
        ));
    }
    let mut graph = HashMap::new();
    for entry in lineage {
        graph.insert(
            entry.detector_id.as_str(),
            entry
                .parent_detector_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
    }
    for entry in lineage {
        if entry
            .parent_detector_ids
            .iter()
            .any(|parent| !ids.contains(parent.as_str()))
        {
            return Err(ContractError::Invalid(format!(
                "missing lineage parent for {}",
                entry.detector_id
            )));
        }
    }
    detect_cycle(&graph)
}

pub fn validate_object_lineage(objects: &[ObjectIdentity]) -> Result<(), ContractError> {
    for object in objects {
        object.validate()?;
    }
    let ids: HashSet<&str> = objects
        .iter()
        .map(|entry| entry.object_id.as_str())
        .collect();
    if ids.len() != objects.len() {
        return Err(ContractError::Invalid("duplicate object identity".into()));
    }
    let mut graph = HashMap::new();
    for object in objects {
        if object
            .parent_object_ids
            .iter()
            .any(|parent| !ids.contains(parent.as_str()))
        {
            return Err(ContractError::Invalid(format!(
                "missing lineage parent for {}",
                object.object_id
            )));
        }
        graph.insert(
            object.object_id.as_str(),
            object
                .parent_object_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
    }
    detect_cycle(&graph)
}

fn detect_cycle(graph: &HashMap<&str, Vec<&str>>) -> Result<(), ContractError> {
    fn visit<'a>(
        node: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        active: &mut HashSet<&'a str>,
        done: &mut HashSet<&'a str>,
    ) -> bool {
        if active.contains(node) {
            return true;
        }
        if !done.insert(node) {
            return false;
        }
        active.insert(node);
        let cycle = graph.get(node).is_some_and(|parents| {
            parents
                .iter()
                .any(|parent| visit(parent, graph, active, done))
        });
        active.remove(node);
        cycle
    }
    let mut active = HashSet::new();
    let mut done = HashSet::new();
    if graph
        .keys()
        .any(|node| visit(node, graph, &mut active, &mut done))
    {
        Err(ContractError::Invalid("lineage contains a cycle".into()))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ReproducibilityIdentity {
    pub source_identity: String,
    pub binary_identity: String,
    pub configuration_identity: String,
    pub parameter_identity: String,
    pub seed_identity: String,
    pub output_identity: String,
    pub user_identities: Vec<String>,
}

impl ReproducibilityIdentity {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("source_identity", &self.source_identity),
            ("binary_identity", &self.binary_identity),
            ("configuration_identity", &self.configuration_identity),
            ("parameter_identity", &self.parameter_identity),
            ("seed_identity", &self.seed_identity),
            ("output_identity", &self.output_identity),
        ] {
            validate_identity(value, field)?;
        }
        for identity in &self.user_identities {
            validate_identity(identity, "user_identity")?;
        }
        Ok(())
    }

    pub fn identity_hash(&self) -> Result<String, ContractError> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(self).map_err(|error| ContractError::Invalid(error.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    }
}
