//! Authenticated confirmation boundary for R, S, and P.
//!
//! Workers produce descriptive challenge evidence. Only an externally signed
//! artifact, verified against the deployment trust root, can create opaque
//! confirmation values. Private signing keys are outside this crate.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use research_contracts::{
    AttackKind, ChallengeResult, ConfirmationContract, ConfirmationState, EvidenceState,
    MethodChallenge,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::{ConfirmedInputManifest, StrategyError, StrategyObservation, WalkForwardPlan};

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

// RFC 8032 vector-one public key. This is a deployment trust-root placeholder;
// deployments must replace it through the build/release process.
const CUSTODIAN_PUBLIC_KEY: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
];

pub fn custodian_public_key_hex() -> String {
    hex(&CUSTODIAN_PUBLIC_KEY)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationRequest {
    pub contract: ConfirmationContract,
    pub validation: research_contracts::StrategyValidation,
    pub manifest: ConfirmedInputManifest,
    pub plan: WalkForwardPlan,
}

/// Legacy descriptive receipt retained for migration. It is never accepted
/// by an authority function.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustodianReceipt {
    pub authorization_id: String,
    pub policy_identity: String,
    pub authenticated: bool,
}

#[doc(hidden)]
pub trait ExternalCustodian {
    fn authorize(
        &self,
        request: &ConfirmationRequest,
    ) -> Result<CustodianReceipt, ConfirmationError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategyConfirmation {
    pub confirmation_id: String,
    pub holdout_policy_identity: String,
    pub externally_authorized: bool,
    pub activation_locked: bool,
}

/// Untrusted wire artifact emitted by an external custodian.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedCustodianArtifact {
    pub authorization_id: String,
    pub custodian_policy_identity: String,
    pub request_identity: String,
    pub challenge_identity: String,
    pub actor_identity: String,
    pub nonce: String,
    pub expires_at_ns: i64,
    pub replay_identity: String,
    pub signature_hex: String,
    pub public_key_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundCustodianReceipt {
    authorization_id: String,
    custodian_policy_identity: String,
    request_identity: String,
    challenge_identity: String,
    actor_identity: String,
    nonce: String,
    expires_at_ns: i64,
    replay_identity: String,
}

impl BoundCustodianReceipt {
    pub fn authorization_id(&self) -> &str {
        &self.authorization_id
    }
    pub fn custodian_policy_identity(&self) -> &str {
        &self.custodian_policy_identity
    }
    pub fn request_identity(&self) -> &str {
        &self.request_identity
    }
    pub fn challenge_identity(&self) -> &str {
        &self.challenge_identity
    }
    pub fn actor_identity(&self) -> &str {
        &self.actor_identity
    }
    pub fn nonce(&self) -> &str {
        &self.nonce
    }
    pub fn expires_at_ns(&self) -> i64 {
        self.expires_at_ns
    }
    pub fn replay_identity(&self) -> &str {
        &self.replay_identity
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedConfirmation {
    confirmation_id: String,
    request_identity: String,
    challenge_identity: String,
    holdout_policy_identity: String,
    manifest_identity: Option<String>,
}

impl LockedConfirmation {
    pub fn confirmation_id(&self) -> &str {
        &self.confirmation_id
    }
    pub fn request_identity(&self) -> &str {
        &self.request_identity
    }
    pub fn challenge_identity(&self) -> &str {
        &self.challenge_identity
    }
    pub fn holdout_policy_identity(&self) -> &str {
        &self.holdout_policy_identity
    }
    pub fn manifest_identity(&self) -> Option<&str> {
        self.manifest_identity.as_deref()
    }
    pub const fn activation_locked(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectorConfirmationRequest {
    pub contract: ConfirmationContract,
    pub target_identity: String,
    pub evidence_state: EvidenceState,
    pub holdout_policy_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectorConfirmation {
    pub confirmation_id: String,
    pub target_identity: String,
    pub challenge_identity: String,
    pub activation_locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortfolioConfirmationRequest {
    pub contract: ConfirmationContract,
    pub component_identity: String,
    pub strategy_confirmation_ids: Vec<String>,
    pub holdout_policy_identity: String,
    pub evidence_state: EvidenceState,
}

#[derive(Debug, Default)]
pub struct ReplayGuard(Mutex<BTreeSet<String>>);

impl ReplayGuard {
    pub fn consume(&self, replay_identity: &str) -> Result<(), ConfirmationError> {
        let mut seen = self.0.lock().map_err(|_| ConfirmationError::ReplayStore)?;
        if !seen.insert(replay_identity.to_owned()) {
            return Err(ConfirmationError::Replay);
        }
        Ok(())
    }
}

fn global_replay_guard() -> &'static ReplayGuard {
    static GUARD: OnceLock<ReplayGuard> = OnceLock::new();
    GUARD.get_or_init(ReplayGuard::default)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeBundle {
    target_identity: String,
    protocol_identity: String,
    challenges: Vec<MethodChallenge>,
    evidence_identity: String,
    identity: String,
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
    pub fn challenges(&self) -> &[MethodChallenge] {
        &self.challenges
    }
    pub fn validate(&self) -> Result<(), ConfirmationError> {
        if self.protocol_identity != CHALLENGE_PROTOCOL
            || self.challenges.len() != REQUIRED_ATTACKS.len()
            || self.challenges.iter().any(|c| {
                c.protocol_identity != CHALLENGE_PROTOCOL
                    || c.target_identity != self.target_identity
                    || c.validate().is_err()
            })
        {
            return Err(ConfirmationError::Invalid("challenge bundle"));
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
        ))
        .map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
        (expected == self.identity)
            .then_some(())
            .ok_or(ConfirmationError::Invalid("challenge identity"))
    }
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
    ))
    .map_err(|_| ConfirmationError::Invalid("challenge identity"))?;
    let bundle = ChallengeBundle {
        target_identity: input.target_identity.clone(),
        protocol_identity: CHALLENGE_PROTOCOL.into(),
        challenges,
        evidence_identity,
        identity,
    };
    bundle.validate()?;
    Ok(bundle)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmationError {
    Invalid(&'static str),
    Contract(String),
    Custodian(String),
    WorkerCannotAuthorize,
    UnconfirmedInput,
    InvalidSignature,
    Replay,
    ReplayStore,
}
impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(x) => write!(f, "invalid {x}"),
            Self::Contract(x) | Self::Custodian(x) => f.write_str(x),
            Self::WorkerCannotAuthorize => f.write_str("worker cannot authorize confirmation"),
            Self::UnconfirmedInput => f.write_str("confirmation requires confirmed inputs"),
            Self::InvalidSignature => f.write_str("custodian signature is invalid"),
            Self::Replay => f.write_str("custodian receipt replayed"),
            Self::ReplayStore => f.write_str("custodian replay store unavailable"),
        }
    }
}
impl std::error::Error for ConfirmationError {}

#[deprecated(note = "use confirm_strategy_from_artifact")]
pub fn confirm_strategy<C: ExternalCustodian>(
    _request: ConfirmationRequest,
    _custodian: &C,
) -> Result<StrategyConfirmation, ConfirmationError> {
    Err(ConfirmationError::WorkerCannotAuthorize)
}

pub fn confirm_strategy_bound(
    request: &ConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    confirm_strategy_with_guard(request, bundle, receipt, now_ns, global_replay_guard())
}
pub fn confirm_strategy_with_guard(
    request: &ConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<LockedConfirmation, ConfirmationError> {
    validate_request(request)?;
    validate_passed_bundle(bundle, &request.validation.validation_id)?;
    let identity = strategy_request_identity(request)?;
    verify_receipt(
        receipt,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    replay_guard.consume(receipt.replay_identity())?;
    Ok(LockedConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        request_identity: identity,
        challenge_identity: bundle.identity().into(),
        holdout_policy_identity: request.plan.policy_identity.clone(),
        manifest_identity: Some(hash_manifest(&request.manifest)),
    })
}
pub fn confirm_strategy_report(
    request: &ConfirmationRequest,
    report: &crate::WalkForwardReport,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    if request.validation != report.validation {
        return Err(ConfirmationError::Invalid(
            "validation must come from completed run",
        ));
    }
    confirm_strategy_bound(request, bundle, receipt, now_ns)
}

pub fn confirm_detector(
    request: &DetectorConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<DetectorConfirmation, ConfirmationError> {
    confirm_detector_with_guard(request, bundle, receipt, now_ns, global_replay_guard())
}
pub fn confirm_detector_with_guard(
    request: &DetectorConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<DetectorConfirmation, ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|e| ConfirmationError::Contract(e.to_string()))?;
    if request.target_identity.is_empty()
        || request.holdout_policy_identity != request.contract.holdout_policy_identity
        || request.evidence_state != EvidenceState::Known
    {
        return Err(ConfirmationError::Invalid("detector confirmation inputs"));
    }
    validate_passed_bundle(bundle, &request.target_identity)?;
    let identity = digest_json(&(
        &request.contract,
        &request.target_identity,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    verify_receipt(
        receipt,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    replay_guard.consume(receipt.replay_identity())?;
    Ok(DetectorConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        target_identity: request.target_identity.clone(),
        challenge_identity: bundle.identity().into(),
        activation_locked: true,
    })
}

pub fn confirm_portfolio(
    request: &PortfolioConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
) -> Result<LockedConfirmation, ConfirmationError> {
    confirm_portfolio_with_guard(request, bundle, receipt, now_ns, global_replay_guard())
}
pub fn confirm_portfolio_with_guard(
    request: &PortfolioConfirmationRequest,
    bundle: &ChallengeBundle,
    receipt: &BoundCustodianReceipt,
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<LockedConfirmation, ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|e| ConfirmationError::Contract(e.to_string()))?;
    if request.component_identity.is_empty()
        || request.strategy_confirmation_ids.is_empty()
        || request
            .strategy_confirmation_ids
            .windows(2)
            .any(|p| p[0] >= p[1])
        || request.holdout_policy_identity != request.contract.holdout_policy_identity
        || request.evidence_state != EvidenceState::Known
    {
        return Err(ConfirmationError::Invalid("portfolio confirmation inputs"));
    }
    validate_passed_bundle(bundle, &request.component_identity)?;
    let identity = digest_json(&(
        &request.contract,
        &request.component_identity,
        &request.strategy_confirmation_ids,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    verify_receipt(
        receipt,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    replay_guard.consume(receipt.replay_identity())?;
    Ok(LockedConfirmation {
        confirmation_id: request.contract.confirmation_id.clone(),
        request_identity: identity,
        challenge_identity: bundle.identity().into(),
        holdout_policy_identity: request.holdout_policy_identity.clone(),
        manifest_identity: None,
    })
}

pub fn confirm_strategy_from_artifact(
    request: &ConfirmationRequest,
    bundle: &ChallengeBundle,
    artifact: &[u8],
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<LockedConfirmation, ConfirmationError> {
    let identity = strategy_request_identity(request)?;
    let receipt = verify_artifact(
        artifact,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    confirm_strategy_with_guard(request, bundle, &receipt, now_ns, replay_guard)
}
pub fn confirm_detector_from_artifact(
    request: &DetectorConfirmationRequest,
    bundle: &ChallengeBundle,
    artifact: &[u8],
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<DetectorConfirmation, ConfirmationError> {
    let identity = digest_json(&(
        &request.contract,
        &request.target_identity,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    let receipt = verify_artifact(
        artifact,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    confirm_detector_with_guard(request, bundle, &receipt, now_ns, replay_guard)
}

pub fn confirm_portfolio_from_artifact(
    request: &PortfolioConfirmationRequest,
    bundle: &ChallengeBundle,
    artifact: &[u8],
    now_ns: i64,
    replay_guard: &ReplayGuard,
) -> Result<LockedConfirmation, ConfirmationError> {
    let identity = digest_json(&(
        &request.contract,
        &request.component_identity,
        &request.strategy_confirmation_ids,
        &request.holdout_policy_identity,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))?;
    let receipt = verify_artifact(
        artifact,
        &request.contract.custodian_policy_identity,
        &identity,
        bundle.identity(),
        now_ns,
    )?;
    confirm_portfolio_with_guard(request, bundle, &receipt, now_ns, replay_guard)
}

fn validate_passed_bundle(bundle: &ChallengeBundle, target: &str) -> Result<(), ConfirmationError> {
    bundle.validate()?;
    if bundle.target_identity() != target
        || bundle
            .challenges()
            .iter()
            .any(|c| c.result != ChallengeResult::Passed)
    {
        return Err(ConfirmationError::Invalid(
            "all challenge categories must pass",
        ));
    }
    Ok(())
}
fn strategy_request_identity(request: &ConfirmationRequest) -> Result<String, ConfirmationError> {
    digest_json(&(
        &request.contract,
        &request.validation,
        &request.manifest,
        &request.plan.policy,
        &request.plan.development_leaf_id,
        &request.plan.strategy_holdout_leaf_id,
        &request.plan.future_leaf_id,
    ))
    .map_err(|_| ConfirmationError::Invalid("request identity"))
}

fn hash_manifest(manifest: &ConfirmedInputManifest) -> String {
    digest_json(manifest).unwrap_or_default()
}

fn validate_request(request: &ConfirmationRequest) -> Result<(), ConfirmationError> {
    request
        .contract
        .validate()
        .map_err(|e| ConfirmationError::Contract(e.to_string()))?;
    if request.contract.actor == research_contracts::ConfirmationActor::Worker {
        return Err(ConfirmationError::WorkerCannotAuthorize);
    }
    if request.contract.state != ConfirmationState::Confirmed {
        return Err(ConfirmationError::Invalid("confirmation state"));
    }
    request
        .validation
        .validate()
        .map_err(|e| ConfirmationError::Contract(e.to_string()))?;
    request
        .plan
        .validate()
        .map_err(|e| ConfirmationError::Contract(e.to_string()))?;
    if request.validation.state != EvidenceState::Known
        || request.validation.strategy_id != request.manifest.strategy_id
        || request.contract.holdout_policy_identity != request.plan.policy_identity
        || request.manifest.holdout_policy_identity != request.plan.policy_identity
    {
        return Err(ConfirmationError::Invalid("confirmation identity"));
    }
    let manifest: BTreeSet<_> = request
        .manifest
        .inputs
        .iter()
        .map(|i| i.input_id.as_str())
        .collect();
    let validation: BTreeSet<_> = request
        .validation
        .confirmed_input_ids
        .iter()
        .map(String::as_str)
        .collect();
    if manifest.is_empty()
        || manifest != validation
        || request.manifest.inputs.iter().any(|i| !i.confirmed)
    {
        return Err(ConfirmationError::UnconfirmedInput);
    }
    Ok(())
}
fn verify_receipt(
    receipt: &BoundCustodianReceipt,
    policy: &str,
    request: &str,
    challenge: &str,
    now_ns: i64,
) -> Result<(), ConfirmationError> {
    if receipt.authorization_id().is_empty()
        || receipt.custodian_policy_identity() != policy
        || receipt.request_identity() != request
        || receipt.challenge_identity() != challenge
        || receipt.actor_identity().is_empty()
        || receipt.nonce().is_empty()
        || receipt.replay_identity().is_empty()
        || receipt.expires_at_ns() <= now_ns
    {
        return Err(ConfirmationError::Custodian(
            "unbound or expired custodian receipt".into(),
        ));
    }
    Ok(())
}
fn verify_artifact(
    bytes: &[u8],
    policy: &str,
    request: &str,
    challenge: &str,
    now_ns: i64,
) -> Result<BoundCustodianReceipt, ConfirmationError> {
    let artifact: SignedCustodianArtifact = serde_json::from_slice(bytes)
        .map_err(|_| ConfirmationError::Invalid("custodian artifact"))?;
    if artifact.public_key_hex.to_ascii_lowercase() != hex(&CUSTODIAN_PUBLIC_KEY) {
        return Err(ConfirmationError::InvalidSignature);
    }
    let signature = Signature::from_slice(
        &decode_hex(&artifact.signature_hex).ok_or(ConfirmationError::InvalidSignature)?,
    )
    .map_err(|_| ConfirmationError::InvalidSignature)?;
    let key = VerifyingKey::from_bytes(&CUSTODIAN_PUBLIC_KEY)
        .map_err(|_| ConfirmationError::InvalidSignature)?;
    key.verify(signed_payload(&artifact).as_bytes(), &signature)
        .map_err(|_| ConfirmationError::InvalidSignature)?;
    let receipt = BoundCustodianReceipt {
        authorization_id: artifact.authorization_id,
        custodian_policy_identity: artifact.custodian_policy_identity,
        request_identity: artifact.request_identity,
        challenge_identity: artifact.challenge_identity,
        actor_identity: artifact.actor_identity,
        nonce: artifact.nonce,
        expires_at_ns: artifact.expires_at_ns,
        replay_identity: artifact.replay_identity,
    };
    verify_receipt(&receipt, policy, request, challenge, now_ns)?;
    Ok(receipt)
}
fn signed_payload(a: &SignedCustodianArtifact) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        a.authorization_id,
        a.custodian_policy_identity,
        a.request_identity,
        a.challenge_identity,
        a.actor_identity,
        a.nonce,
        a.expires_at_ns,
        a.replay_identity
    )
}
fn decode_hex(value: &str) -> Option<Vec<u8>> {
    value.len().is_multiple_of(2).then_some(())?;
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).ok())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn digest_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, serde_json::Error> {
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(value)?);
    Ok(format!("{:x}", h.finalize()))
}

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
