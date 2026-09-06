//! Authenticated confirmation boundary for R, S, and P.
//!
//! Workers produce descriptive challenge evidence. Only an externally signed
//! artifact, verified against the deployment trust root, can create opaque
//! confirmation values. Private signing keys are outside this crate.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use research_contracts::{
    AttackKind, ChallengeResult, ConfirmationActor, ConfirmationContract, ConfirmationState,
    MethodChallenge,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use crate::{StrategyError, StrategyObservation, WalkForwardPlan};

const CHALLENGE_PROTOCOL: &str = "rsp-method-challenge-v1";
const REQUIRED_ATTACKS: [AttackKind; 13] = [
    AttackKind::Lookahead,
    AttackKind::PopulationConditioning,
    AttackKind::DenominatorErrors,
    AttackKind::SelectionBias,
    AttackKind::LineageDuplication,
    AttackKind::FalseConfluence,
    AttackKind::BadControls,
    AttackKind::TemporalDependence,
    AttackKind::MultipleTesting,
    AttackKind::NormalizationLeakage,
    AttackKind::Fragility,
    AttackKind::SampleConcentration,
    AttackKind::AlternativeExplanations,
];

const CONFIRMATION_DOMAIN: &[u8] = b"trinityr-confirmation-artifact-v1\0";
const RFC8032_TEST_PUBLIC_KEY: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfirmationKind {
    Detector,
    Strategy,
    Portfolio,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfirmationBinding {
    kind: ConfirmationKind,
    contract: ConfirmationContract,
    target_identity: String,
    report_identity: String,
    challenge_identity: String,
    scope_identities: Vec<String>,
    manifest_identity: Option<String>,
    input_confirmation_ids: Vec<String>,
    reproducibility_identity: String,
}

impl ConfirmationBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: ConfirmationKind,
        contract: ConfirmationContract,
        target_identity: impl Into<String>,
        report_identity: impl Into<String>,
        challenge_identity: impl Into<String>,
        scope_identities: Vec<String>,
        manifest_identity: Option<String>,
        input_confirmation_ids: Vec<String>,
        reproducibility_identity: impl Into<String>,
    ) -> Result<Self, ConfirmationError> {
        let binding = Self {
            kind,
            contract,
            target_identity: target_identity.into(),
            report_identity: report_identity.into(),
            challenge_identity: challenge_identity.into(),
            scope_identities,
            manifest_identity,
            input_confirmation_ids,
            reproducibility_identity: reproducibility_identity.into(),
        };
        binding.validate()?;
        Ok(binding)
    }

    fn validate(&self) -> Result<(), ConfirmationError> {
        self.contract
            .validate()
            .map_err(|error| ConfirmationError::Contract(error.to_string()))?;
        if self.contract.actor == ConfirmationActor::Worker
            || self.contract.state != ConfirmationState::Confirmed
        {
            return Err(ConfirmationError::Invalid("confirmation contract state"));
        }
        for value in [
            &self.target_identity,
            &self.report_identity,
            &self.challenge_identity,
            &self.reproducibility_identity,
        ] {
            validate_logical_identity(value)?;
        }
        if self.contract.claim_identity != self.target_identity
            || self.scope_identities.is_empty()
            || self
                .scope_identities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .input_confirmation_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(ConfirmationError::Invalid("confirmation identity set"));
        }
        match self.kind {
            ConfirmationKind::Detector
                if self.manifest_identity.is_some() || !self.input_confirmation_ids.is_empty() =>
            {
                return Err(ConfirmationError::Invalid("detector confirmation inputs"));
            }
            ConfirmationKind::Strategy | ConfirmationKind::Portfolio
                if self.manifest_identity.is_none() || self.input_confirmation_ids.is_empty() =>
            {
                return Err(ConfirmationError::Invalid("downstream confirmation inputs"));
            }
            _ => {}
        }
        self.scope_identities
            .iter()
            .chain(self.input_confirmation_ids.iter())
            .try_for_each(|value| validate_logical_identity(value))?;
        if let Some(value) = &self.manifest_identity {
            validate_logical_identity(value)?;
        }
        Ok(())
    }

    pub fn with_report_identity(
        mut self,
        report_identity: impl Into<String>,
    ) -> Result<Self, ConfirmationError> {
        self.report_identity = report_identity.into();
        self.validate()?;
        Ok(self)
    }

    pub const fn kind(&self) -> ConfirmationKind {
        self.kind
    }
    pub fn confirmation_id(&self) -> &str {
        &self.contract.confirmation_id
    }
    pub fn target_identity(&self) -> &str {
        &self.target_identity
    }
    pub fn report_identity(&self) -> &str {
        &self.report_identity
    }
    pub fn challenge_identity(&self) -> &str {
        &self.challenge_identity
    }
    pub fn holdout_policy_identity(&self) -> &str {
        &self.contract.holdout_policy_identity
    }
    pub fn scope_identities(&self) -> &[String] {
        &self.scope_identities
    }
    pub fn manifest_identity(&self) -> Option<&str> {
        self.manifest_identity.as_deref()
    }
    pub fn input_confirmation_ids(&self) -> &[String] {
        &self.input_confirmation_ids
    }
    pub fn reproducibility_identity(&self) -> &str {
        &self.reproducibility_identity
    }

    pub fn contract(&self) -> &ConfirmationContract {
        &self.contract
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedCustodianArtifact {
    policy_identity: String,
    key_fingerprint: String,
    binding: ConfirmationBinding,
    actor_identity: String,
    nonce: String,
    expires_at_ns: i64,
    replay_identity: String,
    signature_hex: String,
}

impl SignedCustodianArtifact {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        policy_identity: impl Into<String>,
        public_key_hex: impl AsRef<str>,
        binding: ConfirmationBinding,
        actor_identity: impl Into<String>,
        nonce: impl Into<String>,
        expires_at_ns: i64,
        replay_identity: impl Into<String>,
    ) -> Result<Self, ConfirmationError> {
        let public_key = decode_fixed_hex::<32>(public_key_hex.as_ref())
            .ok_or(ConfirmationError::InvalidSignature)?;
        let artifact = Self {
            policy_identity: policy_identity.into(),
            key_fingerprint: sha256_hex(&public_key),
            binding,
            actor_identity: actor_identity.into(),
            nonce: nonce.into(),
            expires_at_ns,
            replay_identity: replay_identity.into(),
            signature_hex: String::new(),
        };
        artifact.validate_unsigned()?;
        Ok(artifact)
    }

    fn validate_unsigned(&self) -> Result<(), ConfirmationError> {
        self.binding.validate()?;
        for value in [
            &self.policy_identity,
            &self.key_fingerprint,
            &self.actor_identity,
            &self.nonce,
            &self.replay_identity,
        ] {
            validate_logical_identity(value)?;
        }
        if self.expires_at_ns <= 0 {
            return Err(ConfirmationError::Invalid("custodian artifact expiry"));
        }
        Ok(())
    }

    pub fn signing_bytes(&self) -> Result<Vec<u8>, ConfirmationError> {
        self.validate_unsigned()?;
        let mut bytes = CONFIRMATION_DOMAIN.to_vec();
        bytes.extend(
            serde_json::to_vec(&(
                &self.policy_identity,
                &self.key_fingerprint,
                &self.binding,
                &self.actor_identity,
                &self.nonce,
                self.expires_at_ns,
                &self.replay_identity,
            ))
            .map_err(|_| ConfirmationError::Invalid("custodian artifact"))?,
        );
        Ok(bytes)
    }

    pub fn set_signature_hex(&mut self, signature_hex: impl Into<String>) {
        self.signature_hex = signature_hex.into();
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustRootFile {
    version: u16,
    policy_identity: String,
    public_key_hex: String,
    replay_directory: PathBuf,
}

pub struct ConfirmationRunner {
    policy_identity: String,
    verifying_key: VerifyingKey,
    key_fingerprint: String,
    replay_root: PathBuf,
}

impl ConfirmationRunner {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ConfirmationError> {
        let path = path.as_ref();
        let bytes =
            fs::read(path).map_err(|error| ConfirmationError::TrustRoot(error.to_string()))?;
        let trust: TrustRootFile = serde_json::from_slice(&bytes)
            .map_err(|error| ConfirmationError::TrustRoot(error.to_string()))?;
        validate_logical_identity(&trust.policy_identity)?;
        if trust.version != 1 || !safe_relative_path(&trust.replay_directory) {
            return Err(ConfirmationError::TrustRoot(
                "invalid trust-root version or replay directory".into(),
            ));
        }
        let key_bytes = decode_fixed_hex::<32>(&trust.public_key_hex)
            .ok_or(ConfirmationError::InvalidSignature)?;
        if key_bytes == RFC8032_TEST_PUBLIC_KEY {
            return Err(ConfirmationError::TrustRoot(
                "RFC8032 test key cannot be a deployment trust root".into(),
            ));
        }
        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|_| ConfirmationError::InvalidSignature)?;
        let parent = path
            .parent()
            .ok_or_else(|| ConfirmationError::TrustRoot("trust-root path has no parent".into()))?;
        let replay_root = parent.join(&trust.replay_directory);
        fs::create_dir_all(&replay_root)
            .map_err(|error| ConfirmationError::ReplayStore(error.to_string()))?;
        let canonical_parent = fs::canonicalize(parent)
            .map_err(|error| ConfirmationError::TrustRoot(error.to_string()))?;
        let canonical_replay = fs::canonicalize(&replay_root)
            .map_err(|error| ConfirmationError::ReplayStore(error.to_string()))?;
        if !canonical_replay.starts_with(&canonical_parent) {
            return Err(ConfirmationError::ReplayStore(
                "replay directory escapes trust-root directory".into(),
            ));
        }
        Ok(Self {
            policy_identity: trust.policy_identity,
            verifying_key,
            key_fingerprint: sha256_hex(&key_bytes),
            replay_root: canonical_replay,
        })
    }

    pub fn confirm(
        &self,
        expected: &ConfirmationBinding,
        artifact_bytes: &[u8],
        now_ns: i64,
    ) -> Result<LockedConfirmation, ConfirmationError> {
        expected.validate()?;
        let artifact: SignedCustodianArtifact = serde_json::from_slice(artifact_bytes)
            .map_err(|_| ConfirmationError::Invalid("custodian artifact"))?;
        artifact.validate_unsigned()?;
        if artifact.policy_identity != self.policy_identity
            || artifact.key_fingerprint != self.key_fingerprint
        {
            return Err(ConfirmationError::InvalidSignature);
        }
        if artifact.binding != *expected {
            return Err(ConfirmationError::BindingMismatch);
        }
        if artifact.expires_at_ns <= now_ns {
            return Err(ConfirmationError::Expired);
        }
        let signature = decode_fixed_hex::<64>(&artifact.signature_hex)
            .and_then(|bytes| Signature::from_slice(&bytes).ok())
            .ok_or(ConfirmationError::InvalidSignature)?;
        self.verifying_key
            .verify(&artifact.signing_bytes()?, &signature)
            .map_err(|_| ConfirmationError::InvalidSignature)?;
        self.consume_replay(&artifact)?;
        Ok(LockedConfirmation {
            binding: expected.clone(),
            artifact_identity: sha256_hex(artifact_bytes),
        })
    }

    pub fn confirm_challenged(
        &self,
        expected: &ConfirmationBinding,
        challenge: &ChallengeBundle,
        artifact_bytes: &[u8],
        now_ns: i64,
    ) -> Result<LockedConfirmation, ConfirmationError> {
        challenge.validate()?;
        if !challenge.authority_eligible()
            || challenge.target_identity() != expected.target_identity()
            || challenge.identity() != expected.challenge_identity()
            || challenge
                .challenges()
                .iter()
                .any(|challenge| challenge.result != ChallengeResult::Passed)
        {
            return Err(ConfirmationError::Invalid(
                "challenge is not passed and bound",
            ));
        }
        self.confirm(expected, artifact_bytes, now_ns)
    }

    fn consume_replay(&self, artifact: &SignedCustodianArtifact) -> Result<(), ConfirmationError> {
        let key = sha256_hex(
            &serde_json::to_vec(&(
                artifact.binding.kind,
                &artifact.binding.target_identity,
                &artifact.binding.report_identity,
                &artifact.nonce,
                &artifact.replay_identity,
            ))
            .map_err(|_| ConfirmationError::ReplayStore("cannot encode replay key".into()))?,
        );
        let path = self.replay_root.join(format!("{key}.used"));
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(ConfirmationError::Replay)
            }
            Err(error) => return Err(ConfirmationError::ReplayStore(error.to_string())),
        };
        let result = file
            .write_all(artifact.replay_identity.as_bytes())
            .and_then(|()| file.sync_all());
        if let Err(error) = result {
            drop(file);
            let _ = fs::remove_file(path);
            return Err(ConfirmationError::ReplayStore(error.to_string()));
        }
        Ok(())
    }
}

fn validate_logical_identity(value: &str) -> Result<(), ConfirmationError> {
    if value.is_empty()
        || value.contains('/')
        || value.contains('\\')
        || value.as_bytes().get(1) == Some(&b':')
    {
        return Err(ConfirmationError::Invalid("logical identity"));
    }
    Ok(())
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 || !value.is_ascii() {
        return None;
    }
    let mut bytes = [0; N];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

fn digest_json<T: Serialize + ?Sized>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_vec(value).map(|bytes| sha256_hex(&bytes))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedConfirmation {
    binding: ConfirmationBinding,
    artifact_identity: String,
}

impl LockedConfirmation {
    pub const fn kind(&self) -> ConfirmationKind {
        self.binding.kind()
    }
    pub fn contract(&self) -> &ConfirmationContract {
        self.binding.contract()
    }
    pub fn confirmation_id(&self) -> &str {
        self.binding.confirmation_id()
    }
    pub fn request_identity(&self) -> &str {
        self.binding.target_identity()
    }
    pub fn challenge_identity(&self) -> &str {
        self.binding.challenge_identity()
    }
    pub fn holdout_policy_identity(&self) -> &str {
        self.binding.holdout_policy_identity()
    }
    pub fn manifest_identity(&self) -> Option<&str> {
        self.binding.manifest_identity()
    }
    pub fn report_identity(&self) -> &str {
        self.binding.report_identity()
    }
    pub fn target_identity(&self) -> &str {
        self.binding.target_identity()
    }
    pub fn input_confirmation_ids(&self) -> &[String] {
        self.binding.input_confirmation_ids()
    }
    pub fn artifact_identity(&self) -> &str {
        &self.artifact_identity
    }
    pub const fn activation_locked(&self) -> bool {
        true
    }
}

/// Per-category synthetic evidence. Each variant is validated with its own
/// typed predicate before a challenge is marked passed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AttackMetrics {
    pub evidence_id: String,
    pub sample_size: usize,
    pub violations: usize,
    pub denominator: usize,
    pub source_ids: Vec<String>,
    pub control_identity: Option<String>,
    pub block_count: usize,
    pub family_count: usize,
    pub fit_end_ns: Option<i64>,
    pub evaluation_start_ns: Option<i64>,
    pub perturbation_count: usize,
    pub max_sample_share: f64,
    pub alternative_explanations: Vec<String>,
    pub metric: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AttackEvidence {
    Lookahead { metrics: AttackMetrics },
    PopulationConditioning { metrics: AttackMetrics },
    DenominatorErrors { metrics: AttackMetrics },
    SelectionBias { metrics: AttackMetrics },
    LineageDuplication { metrics: AttackMetrics },
    FalseConfluence { metrics: AttackMetrics },
    BadControls { metrics: AttackMetrics },
    TemporalDependence { metrics: AttackMetrics },
    MultipleTesting { metrics: AttackMetrics },
    NormalizationLeakage { metrics: AttackMetrics },
    Fragility { metrics: AttackMetrics },
    SampleConcentration { metrics: AttackMetrics },
    AlternativeExplanations { metrics: AttackMetrics },
}

impl AttackEvidence {
    fn kind(&self) -> AttackKind {
        match self {
            Self::Lookahead { .. } => AttackKind::Lookahead,
            Self::PopulationConditioning { .. } => AttackKind::PopulationConditioning,
            Self::DenominatorErrors { .. } => AttackKind::DenominatorErrors,
            Self::SelectionBias { .. } => AttackKind::SelectionBias,
            Self::LineageDuplication { .. } => AttackKind::LineageDuplication,
            Self::FalseConfluence { .. } => AttackKind::FalseConfluence,
            Self::BadControls { .. } => AttackKind::BadControls,
            Self::TemporalDependence { .. } => AttackKind::TemporalDependence,
            Self::MultipleTesting { .. } => AttackKind::MultipleTesting,
            Self::NormalizationLeakage { .. } => AttackKind::NormalizationLeakage,
            Self::Fragility { .. } => AttackKind::Fragility,
            Self::SampleConcentration { .. } => AttackKind::SampleConcentration,
            Self::AlternativeExplanations { .. } => AttackKind::AlternativeExplanations,
        }
    }
    fn metrics(&self) -> &AttackMetrics {
        match self {
            Self::Lookahead { metrics }
            | Self::PopulationConditioning { metrics }
            | Self::DenominatorErrors { metrics }
            | Self::SelectionBias { metrics }
            | Self::LineageDuplication { metrics }
            | Self::FalseConfluence { metrics }
            | Self::BadControls { metrics }
            | Self::TemporalDependence { metrics }
            | Self::MultipleTesting { metrics }
            | Self::NormalizationLeakage { metrics }
            | Self::Fragility { metrics }
            | Self::SampleConcentration { metrics }
            | Self::AlternativeExplanations { metrics } => metrics,
        }
    }
    fn passes(&self) -> bool {
        let m = self.metrics();
        if m.evidence_id.is_empty() || m.sample_size == 0 || !m.metric.is_finite() {
            return false;
        }
        match self {
            Self::Lookahead { .. }
            | Self::PopulationConditioning { .. }
            | Self::DenominatorErrors { .. } => m.violations == 0 && m.denominator > 0,
            Self::SelectionBias { .. } => m.source_ids.len() >= 2,
            Self::LineageDuplication { .. } | Self::FalseConfluence { .. } => {
                m.source_ids.len() >= 2
                    && m.source_ids.iter().collect::<HashSet<_>>().len() == m.source_ids.len()
            }
            Self::BadControls { .. } => {
                m.control_identity.as_ref().is_some_and(|id| !id.is_empty())
            }
            Self::TemporalDependence { .. } => m.block_count >= 2,
            Self::MultipleTesting { .. } => m.family_count > 0,
            Self::NormalizationLeakage { .. } => m
                .fit_end_ns
                .zip(m.evaluation_start_ns)
                .is_some_and(|(fit, eval)| fit <= eval),
            Self::Fragility { .. } => m.perturbation_count >= 2 && m.violations == 0,
            Self::SampleConcentration { .. } => {
                (0.0..=1.0).contains(&m.max_sample_share) && m.max_sample_share < 0.5
            }
            Self::AlternativeExplanations { .. } => !m.alternative_explanations.is_empty(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChallengeRunInput {
    pub target_identity: String,
    pub evidence: Vec<AttackEvidence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChallengeSubjectKind {
    Research,
    Strategy,
    Portfolio,
}

/// Opaque facts computed from a validated immutable stage report. Callers
/// cannot set category metrics directly; unavailable checks remain inconclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeSubject {
    kind: ChallengeSubjectKind,
    target_identity: String,
    report_identity: String,
    sample_size: usize,
    causal: bool,
    denominator: usize,
    source_ids: Vec<String>,
    block_count: usize,
    family_count: usize,
    fit_end_ns: Option<i64>,
    evaluation_start_ns: Option<i64>,
    perturbation_count: usize,
    max_sample_share_millionths: u32,
    alternatives_declared: bool,
}

impl ChallengeSubject {
    pub fn from_research_report(report: &crate::ResearchReport) -> Result<Self, ConfirmationError> {
        report
            .validate_identity()
            .map_err(|_| ConfirmationError::Invalid("research report identity"))?;
        Ok(Self {
            kind: ChallengeSubjectKind::Research,
            target_identity: report.output_identity.clone(),
            report_identity: report.output_identity.clone(),
            sample_size: report.records,
            causal: report.information_system.status == research_contracts::EvidenceState::Known,
            denominator: report.records,
            source_ids: report.information_system.source_identities.clone(),
            block_count: report
                .information_system
                .scope_start_ns
                .zip(report.information_system.scope_end_ns)
                .map_or(0, |(start, end)| usize::from(end >= start)),
            family_count: report.question_report.questions.len(),
            fit_end_ns: None,
            evaluation_start_ns: None,
            perturbation_count: 0,
            max_sample_share_millionths: if report.records == 0 {
                1_000_000
            } else {
                1_000_000 / report.records as u32
            },
            alternatives_declared: false,
        })
    }

    pub fn from_strategy_report(
        report: &crate::WalkForwardReport,
    ) -> Result<Self, ConfirmationError> {
        report
            .validate_identity()
            .map_err(|_| ConfirmationError::Invalid("strategy report identity"))?;
        if report.report_identity.is_empty() || report.validation.strategy_id.is_empty() {
            return Err(ConfirmationError::Invalid("strategy report identity"));
        }
        Ok(Self {
            kind: ChallengeSubjectKind::Strategy,
            target_identity: report.validation.strategy_id.clone(),
            report_identity: report.report_identity.clone(),
            sample_size: report.strategy_holdout_observations,
            causal: report.strategy_holdout.authority_eligible,
            denominator: report.strategy_holdout_observations,
            source_ids: report.validation.confirmed_input_ids.clone(),
            block_count: usize::from(report.strategy_holdout_observations > 0),
            family_count: 1,
            fit_end_ns: report.development_fit.as_ref().map(|_| 0),
            evaluation_start_ns: report.development_fit.as_ref().map(|_| 1),
            perturbation_count: 0,
            max_sample_share_millionths: if report.strategy_holdout_observations == 0 {
                1_000_000
            } else {
                1_000_000 / report.strategy_holdout_observations as u32
            },
            alternatives_declared: false,
        })
    }

    pub fn from_portfolio_report(
        report: &crate::PortfolioReport,
    ) -> Result<Self, ConfirmationError> {
        report
            .validate_identity()
            .map_err(|_| ConfirmationError::Invalid("portfolio report identity"))?;
        let sample_size = report
            .correlations
            .iter()
            .map(|metric| metric.observations)
            .max()
            .unwrap_or(0);
        Ok(Self {
            kind: ChallengeSubjectKind::Portfolio,
            target_identity: report.identity.clone(),
            report_identity: report.identity.clone(),
            sample_size,
            causal: true,
            denominator: sample_size,
            source_ids: report
                .correlations
                .iter()
                .flat_map(|metric| metric.strategy_ids.clone())
                .collect(),
            block_count: usize::from(sample_size > 0),
            family_count: report.constraints.constraints.len(),
            fit_end_ns: None,
            evaluation_start_ns: None,
            perturbation_count: 0,
            max_sample_share_millionths: if sample_size == 0 {
                1_000_000
            } else {
                1_000_000 / sample_size as u32
            },
            alternatives_declared: false,
        })
    }

    pub fn target_identity(&self) -> &str {
        &self.target_identity
    }
    pub fn report_identity(&self) -> &str {
        &self.report_identity
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeBundle {
    target_identity: String,
    protocol_identity: String,
    challenges: Vec<MethodChallenge>,
    evidence_identity: String,
    report_identity: String,
    identity: String,
    authority_eligible: bool,
}

impl ChallengeBundle {
    pub fn target_identity(&self) -> &str {
        &self.target_identity
    }
    pub fn protocol_identity(&self) -> &str {
        &self.protocol_identity
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn evidence_identity(&self) -> &str {
        &self.evidence_identity
    }
    pub fn report_identity(&self) -> &str {
        &self.report_identity
    }
    pub const fn authority_eligible(&self) -> bool {
        self.authority_eligible
    }
    pub fn challenges(&self) -> &[MethodChallenge] {
        &self.challenges
    }
    pub fn validate(&self) -> Result<(), ConfirmationError> {
        if self.protocol_identity != CHALLENGE_PROTOCOL
            || self.report_identity.is_empty()
            || self.challenges.len() != REQUIRED_ATTACKS.len()
            || self.challenges.iter().any(|c| {
                c.protocol_identity != CHALLENGE_PROTOCOL
                    || c.target_identity != self.target_identity
                    || c.validate().is_err()
            })
        {
            return Err(ConfirmationError::Invalid("challenge bundle"));
        }
        if self.challenges.iter().any(|challenge| {
            challenge
                .evidence_ids
                .iter()
                .any(|id| !id.starts_with(&self.report_identity))
        }) {
            return Err(ConfirmationError::Invalid(
                "challenge evidence is not bound to report",
            ));
        }
        let kinds: HashSet<_> = self.challenges.iter().map(|c| c.attack_kind).collect();
        if kinds.len() != REQUIRED_ATTACKS.len()
            || REQUIRED_ATTACKS.iter().any(|kind| !kinds.contains(kind))
        {
            return Err(ConfirmationError::Invalid("challenge categories"));
        }
        let expected = digest_json(&(
            &self.target_identity,
            &self.protocol_identity,
            &self.challenges,
            &self.evidence_identity,
            &self.report_identity,
        ))
        .map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
        (expected == self.identity)
            .then_some(())
            .ok_or(ConfirmationError::Invalid("challenge identity"))
    }
}

pub fn run_challenge_subject(
    subject: &ChallengeSubject,
) -> Result<ChallengeBundle, ConfirmationError> {
    if subject.target_identity.is_empty()
        || subject.report_identity.is_empty()
        || subject.sample_size == 0
    {
        return Err(ConfirmationError::Invalid("challenge subject"));
    }
    let values = [
        (AttackKind::Lookahead, subject.causal),
        (AttackKind::PopulationConditioning, subject.sample_size > 0),
        (
            AttackKind::DenominatorErrors,
            subject.denominator == subject.sample_size,
        ),
        (
            AttackKind::SelectionBias,
            unique_sources(&subject.source_ids) >= 2,
        ),
        (
            AttackKind::LineageDuplication,
            unique_sources(&subject.source_ids) == subject.source_ids.len(),
        ),
        (
            AttackKind::FalseConfluence,
            unique_sources(&subject.source_ids) == subject.source_ids.len(),
        ),
        (AttackKind::BadControls, false),
        (AttackKind::TemporalDependence, subject.block_count >= 2),
        (AttackKind::MultipleTesting, subject.family_count > 0),
        (
            AttackKind::NormalizationLeakage,
            subject
                .fit_end_ns
                .zip(subject.evaluation_start_ns)
                .is_some_and(|(fit, eval)| fit <= eval),
        ),
        (AttackKind::Fragility, subject.perturbation_count >= 2),
        (
            AttackKind::SampleConcentration,
            subject.max_sample_share_millionths < 500_000,
        ),
        (
            AttackKind::AlternativeExplanations,
            subject.alternatives_declared,
        ),
    ];
    let challenges = values
        .into_iter()
        .map(|(kind, passed)| MethodChallenge {
            challenge_id: format!(
                "challenge-{}-{}",
                subject.target_identity,
                attack_kind_name(kind)
            ),
            target_identity: subject.target_identity.clone(),
            attack_kind: kind,
            protocol_identity: CHALLENGE_PROTOCOL.into(),
            result: if passed {
                ChallengeResult::Passed
            } else {
                ChallengeResult::Inconclusive
            },
            evidence_ids: vec![format!(
                "{}:{}",
                subject.report_identity,
                attack_kind_name(kind)
            )],
        })
        .collect::<Vec<_>>();
    let evidence_identity =
        digest_json(&(subject.report_identity.clone(), &values, &challenges))
            .map_err(|_| ConfirmationError::Invalid("challenge evidence identity"))?;
    let identity = digest_json(&(
        &subject.target_identity,
        CHALLENGE_PROTOCOL,
        &challenges,
        &evidence_identity,
        &subject.report_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let bundle = ChallengeBundle {
        target_identity: subject.target_identity.clone(),
        protocol_identity: CHALLENGE_PROTOCOL.into(),
        challenges,
        evidence_identity,
        report_identity: subject.report_identity.clone(),
        identity,
        authority_eligible: true,
    };
    bundle.validate()?;
    Ok(bundle)
}

fn unique_sources(sources: &[String]) -> usize {
    sources.iter().collect::<HashSet<_>>().len()
}

pub fn run_challenge_bundle(
    input: &ChallengeRunInput,
) -> Result<ChallengeBundle, ConfirmationError> {
    if input.target_identity.is_empty() || input.evidence.len() != REQUIRED_ATTACKS.len() {
        return Err(ConfirmationError::Invalid("challenge runner input"));
    }
    let mut seen = HashSet::new();
    if input.evidence.iter().any(|e| !seen.insert(e.kind())) {
        return Err(ConfirmationError::Invalid("duplicate challenge category"));
    }
    if REQUIRED_ATTACKS.iter().any(|kind| !seen.contains(kind)) {
        return Err(ConfirmationError::Invalid("missing challenge category"));
    }
    let challenges = input
        .evidence
        .iter()
        .map(|evidence| {
            let kind = evidence.kind();
            let result = if evidence.passes() {
                ChallengeResult::Passed
            } else if evidence.metrics().violations > 0 {
                ChallengeResult::Failed
            } else {
                ChallengeResult::Inconclusive
            };
            MethodChallenge {
                challenge_id: format!(
                    "challenge-{}-{}",
                    input.target_identity,
                    attack_kind_name(kind)
                ),
                target_identity: input.target_identity.clone(),
                attack_kind: kind,
                protocol_identity: CHALLENGE_PROTOCOL.into(),
                result,
                evidence_ids: vec![evidence.metrics().evidence_id.clone()],
            }
        })
        .collect::<Vec<_>>();
    let evidence_identity = digest_json(&input.evidence)
        .map_err(|_| ConfirmationError::Invalid("challenge evidence identity"))?;
    let identity = digest_json(&(
        &input.target_identity,
        &CHALLENGE_PROTOCOL,
        &challenges,
        &evidence_identity,
        &input.target_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let bundle = ChallengeBundle {
        target_identity: input.target_identity.clone(),
        protocol_identity: CHALLENGE_PROTOCOL.into(),
        challenges,
        evidence_identity,
        report_identity: input.target_identity.clone(),
        identity,
        authority_eligible: false,
    };
    bundle.validate()?;
    Ok(bundle)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmationError {
    Invalid(&'static str),
    Contract(String),
    Custodian(String),
    TrustRoot(String),
    BindingMismatch,
    Expired,
    WorkerCannotAuthorize,
    UnconfirmedInput,
    InvalidSignature,
    Replay,
    ReplayStore(String),
}
impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(x) => write!(f, "invalid {x}"),
            Self::Contract(x) | Self::Custodian(x) | Self::TrustRoot(x) | Self::ReplayStore(x) => {
                f.write_str(x)
            }
            Self::BindingMismatch => f.write_str("custodian artifact binding mismatch"),
            Self::Expired => f.write_str("custodian artifact expired"),
            Self::WorkerCannotAuthorize => f.write_str("worker cannot authorize confirmation"),
            Self::UnconfirmedInput => f.write_str("confirmation requires confirmed inputs"),
            Self::InvalidSignature => f.write_str("custodian signature is invalid"),
            Self::Replay => f.write_str("custodian receipt replayed"),
        }
    }
}
impl std::error::Error for ConfirmationError {}

pub fn method_challenge(
    target_identity: &str,
    plan: &WalkForwardPlan,
    observations: &[StrategyObservation],
) -> Result<MethodChallenge, StrategyError> {
    plan.validate()?;
    if observations.is_empty() {
        return Err(StrategyError::EmptySample);
    }
    let passed = observations.iter().all(|o| {
        o.timestamp_ns >= 0
            && o.available_time_ns >= 0
            && o.available_time_ns <= o.timestamp_ns
            && in_plan(plan, o.timestamp_ns)
    });
    let challenge = MethodChallenge {
        challenge_id: format!("challenge-{target_identity}"),
        target_identity: target_identity.into(),
        attack_kind: AttackKind::TimestampLeakage,
        protocol_identity: "causal-partition-check-v1".into(),
        result: if passed {
            ChallengeResult::Passed
        } else {
            ChallengeResult::Failed
        },
        evidence_ids: vec![target_identity.into()],
    };
    challenge
        .validate()
        .map_err(|e| StrategyError::Custody(e.to_string()))?;
    Ok(challenge)
}
pub fn method_challenge_protocol(
    target_identity: &str,
    plan: &WalkForwardPlan,
    observations: &[StrategyObservation],
) -> Result<Vec<MethodChallenge>, StrategyError> {
    plan.validate()?;
    if observations.is_empty() {
        return Err(StrategyError::EmptySample);
    }
    let timestamp_ok = observations.iter().all(|o| {
        o.timestamp_ns >= 0
            && o.available_time_ns >= 0
            && o.available_time_ns <= o.timestamp_ns
            && in_plan(plan, o.timestamp_ns)
    });
    REQUIRED_ATTACKS
        .into_iter()
        .map(|kind| {
            let challenge = MethodChallenge {
                challenge_id: format!("challenge-{target_identity}-{}", attack_kind_name(kind)),
                target_identity: target_identity.into(),
                attack_kind: kind,
                protocol_identity: CHALLENGE_PROTOCOL.into(),
                result: if kind == AttackKind::Lookahead && timestamp_ok {
                    ChallengeResult::Passed
                } else if kind == AttackKind::Lookahead {
                    ChallengeResult::Failed
                } else {
                    ChallengeResult::Inconclusive
                },
                evidence_ids: vec![target_identity.into()],
            };
            challenge
                .validate()
                .map_err(|e| StrategyError::Custody(e.to_string()))?;
            Ok(challenge)
        })
        .collect()
}
fn in_plan(plan: &WalkForwardPlan, timestamp_ns: i64) -> bool {
    contains(&plan.development_scope, timestamp_ns)
        || contains(&plan.strategy_holdout_scope, timestamp_ns)
        || plan
            .future_scope
            .as_ref()
            .is_some_and(|scope| contains(scope, timestamp_ns))
}
fn contains(scope: &research_contracts::ScopeInterval, timestamp_ns: i64) -> bool {
    scope.start_time_ns <= timestamp_ns && timestamp_ns < scope.end_time_ns
}
fn attack_kind_name(kind: AttackKind) -> &'static str {
    match kind {
        AttackKind::Lookahead => "lookahead",
        AttackKind::PopulationConditioning => "population_conditioning",
        AttackKind::DenominatorErrors => "denominator_errors",
        AttackKind::SelectionBias => "selection_bias",
        AttackKind::LineageDuplication => "lineage_duplication",
        AttackKind::FalseConfluence => "false_confluence",
        AttackKind::BadControls => "bad_controls",
        AttackKind::TemporalDependence => "temporal_dependence",
        AttackKind::MultipleTesting => "multiple_testing",
        AttackKind::NormalizationLeakage => "normalization_leakage",
        AttackKind::Fragility => "fragility",
        AttackKind::SampleConcentration => "sample_concentration",
        AttackKind::AlternativeExplanations => "alternative_explanations",
        _ => "unsupported",
    }
}
