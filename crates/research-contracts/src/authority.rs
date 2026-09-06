use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{identity::validate_identity, ContractError, InstrumentScope, NativeScale};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct DetectorVersion {
    pub detector_id: String,
    pub detector_version: String,
    pub state_version: String,
}

impl DetectorVersion {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_identity(&self.detector_id, "detector_id")?;
        validate_identity(&self.detector_version, "detector_version")?;
        validate_identity(&self.state_version, "state_version")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ProducerAuthority {
    pub producer_id: String,
    pub producer_version: String,
    pub producer_commit: String,
    pub state_version: String,
    pub parameter_identity: String,
    pub payload_contract_identity: String,
    pub source_schema_identity: String,
    pub availability_contract_identity: String,
    pub source_blob_identity: String,
    pub detector_version: DetectorVersion,
    pub required_native_scales: BTreeSet<NativeScale>,
}

impl ProducerAuthority {
    pub fn validate(&self) -> Result<(), ContractError> {
        for (field, value) in [
            ("producer_id", &self.producer_id),
            ("producer_version", &self.producer_version),
            ("producer_commit", &self.producer_commit),
            ("state_version", &self.state_version),
            ("parameter_identity", &self.parameter_identity),
            ("payload_contract_identity", &self.payload_contract_identity),
            ("source_schema_identity", &self.source_schema_identity),
            (
                "availability_contract_identity",
                &self.availability_contract_identity,
            ),
            ("source_blob_identity", &self.source_blob_identity),
        ] {
            validate_identity(value, field)?;
        }
        self.detector_version.validate()?;
        if self.required_native_scales.is_empty() {
            return Err(ContractError::Invalid(
                "required native scales cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct AuthorityDelta {
    pub surface: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "status")]
#[schemars(deny_unknown_fields)]
pub enum AuthorityCompatibility {
    #[serde(rename = "AUTHORITY_COMPATIBLE")]
    Compatible,
    #[serde(rename = "AUTHORITY_DELTA_REQUIRED")]
    DeltaRequired {
        changed_surfaces: Vec<AuthorityDelta>,
    },
}

fn add_delta(
    deltas: &mut Vec<AuthorityDelta>,
    surface: &str,
    expected: impl Into<String>,
    actual: impl Into<String>,
) {
    deltas.push(AuthorityDelta {
        surface: surface.into(),
        expected: expected.into(),
        actual: actual.into(),
    });
}

pub fn validate_compatibility(
    expected: &ProducerAuthority,
    actual: &ProducerAuthority,
) -> Result<AuthorityCompatibility, ContractError> {
    expected.validate()?;
    actual.validate()?;
    let mut deltas = Vec::new();
    if expected.producer_id != actual.producer_id {
        add_delta(
            &mut deltas,
            "producer_id",
            expected.producer_id.clone(),
            actual.producer_id.clone(),
        );
    }
    for (surface, left, right) in [
        (
            "producer_commit",
            &expected.producer_commit,
            &actual.producer_commit,
        ),
        (
            "producer_version",
            &expected.producer_version,
            &actual.producer_version,
        ),
        (
            "state_version",
            &expected.state_version,
            &actual.state_version,
        ),
        (
            "parameter_identity",
            &expected.parameter_identity,
            &actual.parameter_identity,
        ),
        (
            "payload_contract_identity",
            &expected.payload_contract_identity,
            &actual.payload_contract_identity,
        ),
        (
            "source_schema_identity",
            &expected.source_schema_identity,
            &actual.source_schema_identity,
        ),
        (
            "availability_contract_identity",
            &expected.availability_contract_identity,
            &actual.availability_contract_identity,
        ),
        (
            "source_blob_identity",
            &expected.source_blob_identity,
            &actual.source_blob_identity,
        ),
    ] {
        if left != right {
            add_delta(&mut deltas, surface, left.clone(), right.clone());
        }
    }
    if expected.detector_version.detector_id != actual.detector_version.detector_id {
        add_delta(
            &mut deltas,
            "detector_id",
            expected.detector_version.detector_id.clone(),
            actual.detector_version.detector_id.clone(),
        );
    }
    if expected.detector_version.detector_version != actual.detector_version.detector_version {
        add_delta(
            &mut deltas,
            "detector_version",
            expected.detector_version.detector_version.clone(),
            actual.detector_version.detector_version.clone(),
        );
    }
    if expected.detector_version.state_version != actual.detector_version.state_version {
        add_delta(
            &mut deltas,
            "detector_state_version",
            expected.detector_version.state_version.clone(),
            actual.detector_version.state_version.clone(),
        );
    }
    if expected.required_native_scales != actual.required_native_scales {
        add_delta(
            &mut deltas,
            "required_native_scales",
            format!("{:?}", expected.required_native_scales),
            format!("{:?}", actual.required_native_scales),
        );
    }
    if deltas.is_empty() {
        Ok(AuthorityCompatibility::Compatible)
    } else {
        Ok(AuthorityCompatibility::DeltaRequired {
            changed_surfaces: deltas,
        })
    }
}

pub fn validate_compatibility_for_scope(
    expected: &ProducerAuthority,
    actual: &ProducerAuthority,
    expected_scope: &InstrumentScope,
    actual_scope: &InstrumentScope,
) -> Result<AuthorityCompatibility, ContractError> {
    expected_scope.validate()?;
    actual_scope.validate()?;
    let mut result = validate_compatibility(expected, actual)?;
    if expected_scope.instrument.instrument_id != actual_scope.instrument.instrument_id
        || expected_scope.instrument.dataset_identity != actual_scope.instrument.dataset_identity
    {
        let delta = AuthorityDelta {
            surface: "instrument_or_dataset_identity".into(),
            expected: format!(
                "{}:{}",
                expected_scope.instrument.instrument_id, expected_scope.instrument.dataset_identity
            ),
            actual: format!(
                "{}:{}",
                actual_scope.instrument.instrument_id, actual_scope.instrument.dataset_identity
            ),
        };
        result = match result {
            AuthorityCompatibility::Compatible => AuthorityCompatibility::DeltaRequired {
                changed_surfaces: vec![delta],
            },
            AuthorityCompatibility::DeltaRequired {
                mut changed_surfaces,
            } => {
                changed_surfaces.push(delta);
                AuthorityCompatibility::DeltaRequired { changed_surfaces }
            }
        };
    }
    Ok(result)
}
