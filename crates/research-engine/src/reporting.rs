//! Deterministic report-manifest infrastructure.
//!
//! This layer packages already-produced machine artifacts. It never interprets
//! market outcomes or manufactures scientific claims.

use research_contracts::{CompleteReproducibilityIdentity, ContractError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const REPORT_MANIFEST_DOMAIN: &[u8] = b"trinityr-report-manifest-v1\0";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportProgram {
    MarketResearch,
    StrategyResearch,
    PortfolioResearch,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReportManifestInput {
    pub program: ReportProgram,
    pub artifact_kind: String,
    pub artifact_identity: String,
    pub reproducibility: CompleteReproducibilityIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReportManifest {
    pub manifest_version: u16,
    pub program: ReportProgram,
    pub artifact_kind: String,
    pub artifact_identity: String,
    pub artifact_sha256: String,
    pub artifact_bytes: u64,
    pub reproducibility_identity: String,
    pub report_identity: String,
}

impl ReportManifest {
    pub fn validate_identity(&self) -> Result<(), ContractError> {
        if self.manifest_version != 1 || self.report_identity != manifest_identity(self)? {
            return Err(ContractError::Invalid(
                "report manifest identity mismatch".into(),
            ));
        }
        Ok(())
    }
}

pub fn generate_report_manifest(
    input: &ReportManifestInput,
    artifact_bytes: &[u8],
) -> Result<ReportManifest, ContractError> {
    if input.artifact_kind.is_empty()
        || input.artifact_identity.is_empty()
        || input.artifact_kind.contains('/')
        || input.artifact_kind.contains('\\')
        || input.artifact_identity.contains('/')
        || input.artifact_identity.contains('\\')
    {
        return Err(ContractError::Invalid(
            "report artifact identities must be nonempty and path-independent".into(),
        ));
    }
    input.reproducibility.validate()?;
    let artifact_sha256 = sha256_hex(artifact_bytes);
    let reproducibility_identity = input.reproducibility.identity_hash()?;
    let mut report = ReportManifest {
        manifest_version: 1,
        program: input.program,
        artifact_kind: input.artifact_kind.clone(),
        artifact_identity: input.artifact_identity.clone(),
        artifact_sha256,
        artifact_bytes: artifact_bytes.len() as u64,
        reproducibility_identity,
        report_identity: String::new(),
    };
    report.report_identity = manifest_identity(&report)?;
    Ok(report)
}

fn manifest_identity(report: &ReportManifest) -> Result<String, ContractError> {
    let bytes = serde_json::to_vec(&(
        report.manifest_version,
        report.program,
        &report.artifact_kind,
        &report.artifact_identity,
        &report.artifact_sha256,
        report.artifact_bytes,
        &report.reproducibility_identity,
    ))
    .map_err(|error| ContractError::Invalid(error.to_string()))?;
    let mut hash = Sha256::new();
    hash.update(REPORT_MANIFEST_DOMAIN);
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}
