//! Descriptive, deterministic portfolio research over confirmed strategy streams.
//!
//! This module reports portfolio relationships and declared constraints. It does
//! not optimize weights, authorize confirmation, place orders, or allocate live
//! capital.

use std::collections::{BTreeMap, BTreeSet};

use crate::LockedConfirmation;
use research_contracts::{
    evidence::{ConfirmationState, EvidenceState},
    PortfolioComponent, PortfolioConstraint, PortfolioConstraintKind,
};

use crate::stats::{StatsError, StreamingMoments};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum PortfolioStatus {
    Known,
    Partial,
    Unknown,
    Null,
    Abstained,
    Inconclusive,
    Contradictory,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PortfolioError {
    EmptyInput,
    InvalidParameter(&'static str),
    NonFinite { strategy_id: String, index: usize },
    DimensionMismatch { strategy_id: String },
    DuplicateStrategy(String),
    MissingStrategy(String),
    UnconfirmedStrategy { strategy_id: String },
    InvalidContract(String),
    Stats(String),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("portfolio input is empty"),
            Self::InvalidParameter(p) => write!(f, "invalid parameter: {p}"),
            Self::NonFinite { strategy_id, index } => {
                write!(f, "non-finite value for {strategy_id} at index {index}")
            }
            Self::DimensionMismatch { strategy_id } => {
                write!(f, "dimension mismatch for strategy {strategy_id}")
            }
            Self::DuplicateStrategy(id) => write!(f, "duplicate strategy: {id}"),
            Self::MissingStrategy(id) => write!(f, "missing strategy: {id}"),
            Self::UnconfirmedStrategy { strategy_id } => {
                write!(f, "strategy is not confirmed: {strategy_id}")
            }
            Self::InvalidContract(error) => f.write_str(error),
            Self::Stats(error) => f.write_str(error),
        }
    }
}
impl std::error::Error for PortfolioError {}
impl From<StatsError> for PortfolioError {
    fn from(error: StatsError) -> Self {
        Self::Stats(error.to_string())
    }
}
impl From<research_contracts::ContractError> for PortfolioError {
    fn from(error: research_contracts::ContractError) -> Self {
        Self::InvalidContract(error.to_string())
    }
}

/// One confirmed strategy's aligned descriptive stream.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StrategyStream {
    pub strategy_id: String,
    pub confirmation: ConfirmationState,
    pub evidence_state: EvidenceState,
    pub returns: Vec<Option<f64>>,
    pub signals: Vec<Option<f64>>,
    pub regimes: Vec<Option<String>>,
    pub capital: Option<f64>,
    pub capacity: Option<f64>,
    pub liquidity: Vec<Option<f64>>,
    pub risk_budget: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PortfolioInput {
    pub component: PortfolioComponent,
    pub strategies: Vec<StrategyStream>,
    pub constraints: Vec<PortfolioConstraint>,
}

/// Boundary-safe portfolio input. The legacy `StrategyStream` remains a
/// descriptive fixture API; production P entry points should use this typed
/// wrapper so index alignment cannot stand in for a causal time grid.
#[derive(Clone, Debug, PartialEq)]
pub struct AlignedStrategyStream {
    pub stream: StrategyStream,
    pub timestamps_ns: Vec<i64>,
    pub availability_ns: Vec<i64>,
    pub instrument_scope_identity: String,
    pub source_identity: String,
    pub time_grid_identity: String,
    pub context_permission: research_contracts::ContextPermission,
    pub confirmation: LockedConfirmation,
    pub stream_identity: String,
}

impl AlignedStrategyStream {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stream: StrategyStream,
        timestamps_ns: Vec<i64>,
        availability_ns: Vec<i64>,
        instrument_scope_identity: String,
        source_identity: String,
        context_permission: research_contracts::ContextPermission,
        confirmation: LockedConfirmation,
    ) -> Result<Self, PortfolioError> {
        let stream_identity = canonical_stream_identity(
            &stream,
            &timestamps_ns,
            &availability_ns,
            &instrument_scope_identity,
            &source_identity,
            &context_permission,
        );
        if confirmation.kind() != crate::ConfirmationKind::Strategy
            || confirmation.confirmation_id() != stream.strategy_id
            || confirmation.target_identity() != stream.strategy_id
            || confirmation.report_identity() != stream_identity
        {
            return Err(PortfolioError::InvalidParameter(
                "strategy stream confirmation binding",
            ));
        }
        let time_grid_identity = canonical_time_grid_identity(&timestamps_ns, &availability_ns);
        Ok(Self {
            stream,
            timestamps_ns,
            availability_ns,
            instrument_scope_identity,
            source_identity,
            time_grid_identity,
            context_permission,
            confirmation,
            stream_identity,
        })
    }
}

pub fn validate_aligned_streams(streams: &[AlignedStrategyStream]) -> Result<(), PortfolioError> {
    if streams.is_empty() {
        return Err(PortfolioError::EmptyInput);
    }
    let first = &streams[0];
    first.context_permission.validate()?;
    if first.context_permission.consumer != research_contracts::ProgramConsumer::PortfolioResearch {
        return Err(PortfolioError::InvalidParameter(
            "portfolio context permission",
        ));
    }
    for stream in streams {
        if !stream.confirmation.activation_locked()
            || stream.confirmation.confirmation_id().is_empty()
            || stream.confirmation.kind() != crate::ConfirmationKind::Strategy
            || stream.confirmation.confirmation_id() != stream.stream.strategy_id
            || stream.confirmation.target_identity() != stream.stream.strategy_id
            || stream.confirmation.report_identity() != stream.stream_identity
            || stream.stream_identity
                != canonical_stream_identity(
                    &stream.stream,
                    &stream.timestamps_ns,
                    &stream.availability_ns,
                    &stream.instrument_scope_identity,
                    &stream.source_identity,
                    &stream.context_permission,
                )
            || stream.stream.evidence_state != EvidenceState::Known
            || stream.timestamps_ns.is_empty()
            || stream.timestamps_ns.len() != stream.availability_ns.len()
            || stream.timestamps_ns.len() != stream.stream.returns.len()
            || stream.instrument_scope_identity.is_empty()
            || stream.source_identity.is_empty()
            || stream.time_grid_identity != first.time_grid_identity
            || stream.time_grid_identity
                != canonical_time_grid_identity(&stream.timestamps_ns, &stream.availability_ns)
            || stream.instrument_scope_identity != first.instrument_scope_identity
            || stream.source_identity != first.source_identity
            || stream.context_permission != first.context_permission
            || stream.confirmation.holdout_policy_identity().is_empty()
        {
            return Err(PortfolioError::InvalidParameter("aligned strategy stream"));
        }
        if stream
            .timestamps_ns
            .windows(2)
            .any(|pair| pair[1] <= pair[0])
            || stream
                .timestamps_ns
                .iter()
                .zip(&stream.availability_ns)
                .any(|(time, available)| *time < 0 || *available < 0 || available > time)
        {
            return Err(PortfolioError::InvalidParameter("causal time grid"));
        }
    }
    Ok(())
}

pub fn canonical_time_grid_identity(timestamps_ns: &[i64], availability_ns: &[i64]) -> String {
    let mut hash = Sha256::new();
    hash.update(serde_json::to_vec(&(timestamps_ns, availability_ns)).unwrap_or_default());
    format!("{:x}", hash.finalize())
}

pub fn portfolio_report_aligned(
    component: PortfolioComponent,
    streams: &[AlignedStrategyStream],
    constraints: Vec<PortfolioConstraint>,
    scenarios: &[StressScenario],
) -> Result<PortfolioReport, PortfolioError> {
    validate_aligned_streams(streams)?;
    component.validate()?;
    let stream_ids: BTreeSet<_> = streams
        .iter()
        .map(|s| s.stream.strategy_id.clone())
        .collect();
    let component_ids: BTreeSet<_> = component.confirmed_strategy_ids.iter().cloned().collect();
    if stream_ids != component_ids {
        return Err(PortfolioError::InvalidParameter(
            "component confirmation identity",
        ));
    }
    let strategies = streams
        .iter()
        .map(|aligned| {
            let mut stream = aligned.stream.clone();
            stream.confirmation = ConfirmationState::Confirmed;
            stream.evidence_state = EvidenceState::Known;
            stream
        })
        .collect();
    let mut report = portfolio_report(
        &PortfolioInput {
            component: component.clone(),
            strategies,
            constraints: constraints.clone(),
        },
        scenarios,
    )?;
    report.input_identity =
        canonical_aligned_report_identity(&component, streams, &constraints, scenarios);
    report.identity = canonical_report_identity(&report);
    Ok(report)
}

pub fn require_confirmed_portfolio_report(
    report: &PortfolioReport,
    confirmation: &LockedConfirmation,
) -> Result<(), PortfolioError> {
    if confirmation.kind() != crate::ConfirmationKind::Portfolio
        || confirmation.target_identity() != report.identity
        || confirmation.report_identity() != report.identity
        || confirmation.contract().state != ConfirmationState::Confirmed
    {
        return Err(PortfolioError::InvalidParameter(
            "portfolio confirmation binding",
        ));
    }
    Ok(())
}

impl PortfolioInput {
    pub fn validate(&self) -> Result<(), PortfolioError> {
        if self.strategies.is_empty() {
            return Err(PortfolioError::EmptyInput);
        }
        self.component.validate()?;
        let mut ids = BTreeSet::new();
        for strategy in &self.strategies {
            if strategy.strategy_id.is_empty() || !ids.insert(strategy.strategy_id.clone()) {
                return Err(PortfolioError::DuplicateStrategy(
                    strategy.strategy_id.clone(),
                ));
            }
            if strategy.confirmation != ConfirmationState::Confirmed {
                return Err(PortfolioError::UnconfirmedStrategy {
                    strategy_id: strategy.strategy_id.clone(),
                });
            }
            let n = strategy.returns.len();
            if n == 0 {
                return Err(PortfolioError::InvalidParameter("returns"));
            }
            for (index, value) in strategy.returns.iter().flatten().enumerate() {
                if !value.is_finite() {
                    return Err(PortfolioError::NonFinite {
                        strategy_id: strategy.strategy_id.clone(),
                        index,
                    });
                }
            }
            for (field, length) in [
                ("signals", strategy.signals.len()),
                ("regimes", strategy.regimes.len()),
                ("liquidity", strategy.liquidity.len()),
            ] {
                if length != 0 && length != n {
                    return Err(PortfolioError::DimensionMismatch {
                        strategy_id: format!("{} ({field})", strategy.strategy_id),
                    });
                }
            }
            for (index, value) in strategy
                .signals
                .iter()
                .chain(strategy.liquidity.iter())
                .flatten()
                .enumerate()
            {
                if !value.is_finite() {
                    return Err(PortfolioError::NonFinite {
                        strategy_id: strategy.strategy_id.clone(),
                        index,
                    });
                }
            }
            for (name, value) in [
                ("capital", strategy.capital),
                ("capacity", strategy.capacity),
                ("risk_budget", strategy.risk_budget),
            ] {
                if value.is_some_and(|v| !v.is_finite() || v < 0.0) {
                    return Err(PortfolioError::InvalidParameter(name));
                }
            }
        }
        let declared: BTreeSet<_> = self.component.confirmed_strategy_ids.iter().collect();
        if declared != ids.iter().collect() {
            return Err(PortfolioError::InvalidParameter(
                "component strategy IDs must match streams",
            ));
        }
        for constraint in &self.constraints {
            constraint.validate()?
        }
        Ok(())
    }

    fn strategy(&self, id: &str) -> Result<&StrategyStream, PortfolioError> {
        self.strategies
            .iter()
            .find(|strategy| strategy.strategy_id == id)
            .ok_or_else(|| PortfolioError::MissingStrategy(id.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MetricReport {
    pub strategy_ids: Vec<String>,
    pub metric: String,
    pub value: Option<f64>,
    pub observations: usize,
    pub status: PortfolioStatus,
}
pub type CorrelationReport = MetricReport;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CapacityReport {
    pub total_capacity: Option<f64>,
    pub bottleneck: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TurnoverReport {
    pub total: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LiquidityReport {
    pub mean: Option<f64>,
    pub minimum: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RiskBudgetReport {
    pub total_budget: f64,
    pub allocations: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConditionalCorrelationReport {
    pub strategy_ids: [String; 2],
    pub regime: String,
    pub value: Option<f64>,
    pub observations: usize,
    pub excluded_observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverlapReport {
    pub strategy_ids: [String; 2],
    pub intersection: usize,
    pub union: usize,
    pub jaccard: f64,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AllocationShare {
    pub strategy_id: String,
    pub capital: Option<f64>,
    pub fraction: Option<f64>,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AllocationReport {
    pub allocations: Vec<AllocationShare>,
    pub total_capital: f64,
    pub status: PortfolioStatus,
}
pub type CapitalAllocationReport = AllocationReport;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DrawdownReport {
    pub max_drawdown: f64,
    pub strategy_drawdowns: Vec<(String, Option<f64>)>,
    pub observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConcentrationReport {
    pub herfindahl: f64,
    pub largest_fraction: Option<f64>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConstraintOutcome {
    pub constraint_id: String,
    pub kind: PortfolioConstraintKind,
    pub value: Option<f64>,
    pub limit: Option<f64>,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConstraintReport {
    pub constraints: Vec<ConstraintOutcome>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StressScenario {
    pub scenario_id: String,
    pub returns: BTreeMap<String, f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StressOutcome {
    pub scenario_id: String,
    pub portfolio_return: f64,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StressReport {
    pub scenarios: Vec<StressOutcome>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SyntheticOptimizationResult {
    pub strategy_ids: Vec<String>,
    pub weights: BTreeMap<String, f64>,
    pub objective: f64,
    pub status: PortfolioStatus,
}

/// Equal-weight synthetic optimizer used only to exercise the portfolio
/// plumbing. It accepts no research result authority and is never a live
/// allocation operation.
pub fn optimize_synthetic(
    input: &PortfolioInput,
) -> Result<SyntheticOptimizationResult, PortfolioError> {
    input.validate()?;
    let n = input.strategies.len() as f64;
    let weight = 1.0 / n;
    let mut weights = BTreeMap::new();
    for strategy in &input.strategies {
        weights.insert(strategy.strategy_id.clone(), weight);
    }
    let objective = input
        .strategies
        .iter()
        .map(|strategy| strategy.returns.iter().flatten().copied().sum::<f64>() * weight)
        .sum();
    Ok(SyntheticOptimizationResult {
        strategy_ids: input
            .strategies
            .iter()
            .map(|s| s.strategy_id.clone())
            .collect(),
        weights,
        objective,
        status: PortfolioStatus::Partial,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PortfolioReport {
    pub identity: String,
    pub input_identity: String,
    pub correlations: Vec<CorrelationReport>,
    pub conditional_correlations: Vec<ConditionalCorrelationReport>,
    pub overlaps: Vec<OverlapReport>,
    pub capital_allocation: CapitalAllocationReport,
    pub drawdown: DrawdownReport,
    pub capacity: CapacityReport,
    pub turnover: TurnoverReport,
    pub liquidity: LiquidityReport,
    pub concentration: ConcentrationReport,
    pub risk_budget: RiskBudgetReport,
    pub constraints: ConstraintReport,
    pub stress: StressReport,
}

impl PortfolioReport {
    pub fn validate_identity(&self) -> Result<(), PortfolioError> {
        if self.identity != canonical_report_identity(self) {
            return Err(PortfolioError::InvalidParameter(
                "portfolio report identity",
            ));
        }
        Ok(())
    }
}

fn canonical_stream_identity(
    stream: &StrategyStream,
    timestamps_ns: &[i64],
    availability_ns: &[i64],
    scope: &str,
    source: &str,
    permission: &research_contracts::ContextPermission,
) -> String {
    let bytes = serde_json::to_vec(&(
        stream,
        timestamps_ns,
        availability_ns,
        scope,
        source,
        permission,
    ))
    .expect("portfolio stream serializes");
    let mut hash = Sha256::new();
    hash.update(b"trinityr-portfolio-stream-v2\0");
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

pub fn strategy_stream_identity(
    stream: &StrategyStream,
    timestamps_ns: &[i64],
    availability_ns: &[i64],
    scope: &str,
    source: &str,
    permission: &research_contracts::ContextPermission,
) -> String {
    canonical_stream_identity(
        stream,
        timestamps_ns,
        availability_ns,
        scope,
        source,
        permission,
    )
}

fn canonical_aligned_report_identity(
    component: &PortfolioComponent,
    streams: &[AlignedStrategyStream],
    constraints: &[PortfolioConstraint],
    scenarios: &[StressScenario],
) -> String {
    let stream_ids = streams
        .iter()
        .map(|stream| stream.stream_identity.clone())
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&(component, stream_ids, constraints, scenarios))
        .expect("portfolio report serializes");
    let mut hash = Sha256::new();
    hash.update(b"trinityr-portfolio-report-v2\0");
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

fn canonical_report_identity(report: &PortfolioReport) -> String {
    let bytes = serde_json::to_vec(&(
        &report.input_identity,
        &report.correlations,
        &report.conditional_correlations,
        &report.overlaps,
        &report.capital_allocation,
        &report.drawdown,
        &report.capacity,
        &report.turnover,
        &report.liquidity,
        &report.concentration,
        &report.risk_budget,
        &report.constraints,
        &report.stress,
    ))
    .expect("portfolio report serializes");
    let mut hash = Sha256::new();
    hash.update(b"trinityr-portfolio-report-v3\0");
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

fn status(states: impl Iterator<Item = EvidenceState>) -> PortfolioStatus {
    let states: Vec<_> = states.collect();
    if states.contains(&EvidenceState::Contradictory) {
        PortfolioStatus::Contradictory
    } else if states.contains(&EvidenceState::Unknown) {
        PortfolioStatus::Unknown
    } else if states.contains(&EvidenceState::Null) {
        PortfolioStatus::Null
    } else if states.contains(&EvidenceState::Rejected) {
        PortfolioStatus::Abstained
    } else if states.contains(&EvidenceState::Inconclusive) {
        PortfolioStatus::Inconclusive
    } else if states.contains(&EvidenceState::Partial) {
        PortfolioStatus::Partial
    } else {
        PortfolioStatus::Known
    }
}

fn with_missing(mut state: PortfolioStatus, observations: usize, total: usize) -> PortfolioStatus {
    if observations == 0 && state == PortfolioStatus::Known {
        PortfolioStatus::Unknown
    } else if observations < total && state == PortfolioStatus::Known {
        state = PortfolioStatus::Partial;
        state
    } else {
        state
    }
}

fn combine_status(left: PortfolioStatus, right: PortfolioStatus) -> PortfolioStatus {
    use PortfolioStatus::*;
    if matches!(left, Contradictory) || matches!(right, Contradictory) {
        Contradictory
    } else if matches!(left, Failed) || matches!(right, Failed) {
        Failed
    } else if matches!(left, Unknown) || matches!(right, Unknown) {
        Unknown
    } else if matches!(left, Null) || matches!(right, Null) {
        Null
    } else if matches!(left, Abstained) || matches!(right, Abstained) {
        Abstained
    } else if matches!(left, Inconclusive) || matches!(right, Inconclusive) {
        Inconclusive
    } else if matches!(left, Partial) || matches!(right, Partial) {
        Partial
    } else {
        Known
    }
}

fn pearson(x: &[f64], y: &[f64]) -> Result<f64, PortfolioError> {
    if x.len() != y.len() || x.is_empty() {
        return Err(PortfolioError::InvalidParameter("paired observations"));
    }
    let mut xm = StreamingMoments::new();
    let mut ym = StreamingMoments::new();
    xm.extend(x)?;
    ym.extend(y)?;
    let mx = xm.mean().unwrap();
    let my = ym.mean().unwrap();
    let (mut covariance, mut vx, mut vy) = (0.0, 0.0, 0.0);
    for (&a, &b) in x.iter().zip(y) {
        let (da, db) = (a - mx, b - my);
        covariance += da * db;
        vx += da * da;
        vy += db * db;
    }
    (vx > 0.0 && vy > 0.0)
        .then_some(covariance / (vx * vy).sqrt())
        .ok_or(PortfolioError::InvalidParameter("constant series"))
}

fn validate_pair<'a>(
    input: &'a PortfolioInput,
    left: &str,
    right: &str,
) -> Result<(&'a StrategyStream, &'a StrategyStream), PortfolioError> {
    input.validate()?;
    if left == right {
        return Err(PortfolioError::InvalidParameter(
            "distinct strategies required",
        ));
    }
    Ok((input.strategy(left)?, input.strategy(right)?))
}

pub fn correlation_report(
    input: &PortfolioInput,
    left: &str,
    right: &str,
) -> Result<CorrelationReport, PortfolioError> {
    let (left, right) = validate_pair(input, left, right)?;
    let total = left.returns.len().min(right.returns.len());
    let mut x = Vec::new();
    let mut y = Vec::new();
    for (a, b) in left.returns.iter().zip(&right.returns) {
        if let (Some(a), Some(b)) = (a, b) {
            x.push(*a);
            y.push(*b);
        }
    }
    let base = status([left.evidence_state.clone(), right.evidence_state.clone()].into_iter());
    let (value, status) = if x.is_empty() {
        (None, with_missing(base, 0, total))
    } else {
        match pearson(&x, &y) {
            Ok(value) => (Some(value), with_missing(base, x.len(), total)),
            Err(_) => (None, PortfolioStatus::Failed),
        }
    };
    Ok(MetricReport {
        strategy_ids: vec![left.strategy_id.clone(), right.strategy_id.clone()],
        metric: "pearson_return_correlation".into(),
        value,
        observations: x.len(),
        status,
    })
}

pub fn conditional_correlation_report(
    input: &PortfolioInput,
    left: &str,
    right: &str,
) -> Result<Vec<ConditionalCorrelationReport>, PortfolioError> {
    let (left, right) = validate_pair(input, left, right)?;
    if left.regimes.is_empty() || right.regimes.is_empty() {
        return Ok(vec![ConditionalCorrelationReport {
            strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
            regime: "<unavailable>".into(),
            value: None,
            observations: 0,
            excluded_observations: left.returns.len().min(right.returns.len()),
            status: PortfolioStatus::Unknown,
        }]);
    }
    let mut grouped: BTreeMap<String, (Vec<f64>, Vec<f64>, usize)> = BTreeMap::new();
    for (((a, b), regime_a), regime_b) in left
        .returns
        .iter()
        .zip(&right.returns)
        .zip(&left.regimes)
        .zip(&right.regimes)
    {
        if let (Some(a), Some(b), Some(regime_a), Some(regime_b)) = (a, b, regime_a, regime_b) {
            if regime_a == regime_b {
                let entry = grouped.entry(regime_a.clone()).or_default();
                entry.0.push(*a);
                entry.1.push(*b);
                entry.2 += 1;
            }
        }
    }
    let base = status([left.evidence_state.clone(), right.evidence_state.clone()].into_iter());
    let total = left.returns.len().min(right.returns.len());
    Ok(grouped
        .into_iter()
        .map(|(regime, (x, y, observations))| match pearson(&x, &y) {
            Ok(value) => ConditionalCorrelationReport {
                strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
                regime,
                value: Some(value),
                observations,
                excluded_observations: total.saturating_sub(observations),
                status: with_missing(base.clone(), observations, total),
            },
            Err(_) => ConditionalCorrelationReport {
                strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
                regime,
                value: None,
                observations,
                excluded_observations: total.saturating_sub(observations),
                status: PortfolioStatus::Failed,
            },
        })
        .collect())
}

pub fn overlap_report(
    input: &PortfolioInput,
    left: &str,
    right: &str,
) -> Result<OverlapReport, PortfolioError> {
    let (left, right) = validate_pair(input, left, right)?;
    let total = left.returns.len().min(right.returns.len());
    let (mut intersection, mut union, mut observations) = (0, 0, 0);
    for i in 0..total {
        let (Some(a), Some(b)) = (
            left.signals.get(i).and_then(|v| *v),
            right.signals.get(i).and_then(|v| *v),
        ) else {
            continue;
        };
        observations += 1;
        let (a, b) = (a != 0.0, b != 0.0);
        intersection += usize::from(a && b);
        union += usize::from(a || b);
    }
    let base = status([left.evidence_state.clone(), right.evidence_state.clone()].into_iter());
    Ok(OverlapReport {
        strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
        intersection,
        union,
        jaccard: if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        },
        status: with_missing(base, observations, total),
    })
}

pub fn capital_allocation_report(
    input: &PortfolioInput,
) -> Result<CapitalAllocationReport, PortfolioError> {
    input.validate()?;
    let total_capital: f64 = input.strategies.iter().filter_map(|s| s.capital).sum();
    let capital_count = input
        .strategies
        .iter()
        .filter(|s| s.capital.is_some())
        .count();
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let allocations = input
        .strategies
        .iter()
        .map(|strategy| AllocationShare {
            strategy_id: strategy.strategy_id.clone(),
            capital: strategy.capital,
            fraction: (total_capital > 0.0).then(|| {
                strategy
                    .capital
                    .map_or(0.0, |capital| capital / total_capital)
            }),
            status: strategy_status(strategy),
        })
        .collect();
    Ok(AllocationReport {
        allocations,
        total_capital,
        status: if total_capital > 0.0 {
            with_missing(base, capital_count, input.strategies.len())
        } else if base == PortfolioStatus::Known {
            PortfolioStatus::Unknown
        } else {
            base
        },
    })
}

fn strategy_status(strategy: &StrategyStream) -> PortfolioStatus {
    status(std::iter::once(strategy.evidence_state.clone()))
}

fn drawdown(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let (mut wealth, mut peak, mut worst) = (1.0_f64, 1.0_f64, 0.0_f64);
    let mut count = 0;
    for value in values {
        wealth *= 1.0 + value;
        if !wealth.is_finite() {
            return None;
        }
        peak = peak.max(wealth);
        worst = worst.max((peak - wealth) / peak);
        count += 1;
    }
    (count > 0).then_some(worst)
}

pub fn drawdown_report(input: &PortfolioInput) -> Result<DrawdownReport, PortfolioError> {
    input.validate()?;
    let n = input
        .strategies
        .iter()
        .map(|s| s.returns.len())
        .min()
        .unwrap_or(0);
    let total_capital: f64 = input.strategies.iter().filter_map(|s| s.capital).sum();
    let weighted = total_capital > 0.0 && input.strategies.iter().all(|s| s.capital.is_some());
    let mut portfolio = Vec::new();
    for i in 0..n {
        let values: Vec<_> = input
            .strategies
            .iter()
            .filter_map(|s| s.returns.get(i).and_then(|v| *v))
            .collect();
        if values.len() == input.strategies.len() {
            portfolio.push(if weighted {
                input
                    .strategies
                    .iter()
                    .zip(values.iter())
                    .map(|(strategy, value)| {
                        value * strategy.capital.unwrap_or(0.0) / total_capital
                    })
                    .sum()
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            });
        }
    }
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let drawdown_status = with_missing(base, portfolio.len(), n);
    Ok(DrawdownReport {
        max_drawdown: drawdown(portfolio.iter().copied()).unwrap_or(0.0),
        strategy_drawdowns: input
            .strategies
            .iter()
            .map(|s| {
                (
                    s.strategy_id.clone(),
                    drawdown(s.returns.iter().flatten().copied()),
                )
            })
            .collect(),
        observations: portfolio.len(),
        status: if weighted {
            drawdown_status
        } else {
            combine_status(drawdown_status, PortfolioStatus::Partial)
        },
    })
}

pub fn capacity_report(input: &PortfolioInput) -> Result<CapacityReport, PortfolioError> {
    input.validate()?;
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let per_strategy = input
        .strategies
        .iter()
        .map(|s| MetricReport {
            strategy_ids: vec![s.strategy_id.clone()],
            metric: "declared_capacity".into(),
            value: s.capacity,
            observations: usize::from(s.capacity.is_some()),
            status: if s.capacity.is_some() {
                base.clone()
            } else {
                PortfolioStatus::Unknown
            },
        })
        .collect::<Vec<_>>();
    let values: Vec<_> = input.strategies.iter().filter_map(|s| s.capacity).collect();
    Ok(CapacityReport {
        total_capacity: (values.len() == input.strategies.len()).then(|| values.iter().sum()),
        bottleneck: values.iter().copied().reduce(f64::min),
        per_strategy,
        status: with_missing(base, values.len(), input.strategies.len()),
    })
}

pub fn turnover_report(input: &PortfolioInput) -> Result<TurnoverReport, PortfolioError> {
    input.validate()?;
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let per_strategy = input
        .strategies
        .iter()
        .map(|s| {
            let values: Vec<_> = s
                .signals
                .windows(2)
                .filter_map(|w| Some((w[0]?, w[1]?)))
                .collect();
            MetricReport {
                strategy_ids: vec![s.strategy_id.clone()],
                metric: "absolute_signal_turnover".into(),
                value: (!values.is_empty())
                    .then(|| values.iter().map(|(a, b)| (b - a).abs()).sum()),
                observations: values.len(),
                status: if values.is_empty() {
                    PortfolioStatus::Unknown
                } else {
                    base.clone()
                },
            }
        })
        .collect::<Vec<_>>();
    let values: Vec<_> = per_strategy.iter().filter_map(|r| r.value).collect();
    Ok(TurnoverReport {
        total: (values.len() == input.strategies.len()).then(|| values.iter().sum()),
        observations: per_strategy.iter().map(|r| r.observations).sum(),
        status: with_missing(base, values.len(), input.strategies.len()),
        per_strategy,
    })
}

pub fn liquidity_report(input: &PortfolioInput) -> Result<LiquidityReport, PortfolioError> {
    input.validate()?;
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let per_strategy = input
        .strategies
        .iter()
        .map(|s| {
            let values: Vec<_> = s.liquidity.iter().flatten().copied().collect();
            MetricReport {
                strategy_ids: vec![s.strategy_id.clone()],
                metric: "mean_liquidity".into(),
                value: (!values.is_empty())
                    .then(|| values.iter().sum::<f64>() / values.len() as f64),
                observations: values.len(),
                status: if values.is_empty() {
                    PortfolioStatus::Unknown
                } else {
                    base.clone()
                },
            }
        })
        .collect::<Vec<_>>();
    let values: Vec<_> = per_strategy.iter().filter_map(|r| r.value).collect();
    let raw_values: Vec<_> = input
        .strategies
        .iter()
        .flat_map(|s| s.liquidity.iter().flatten().copied())
        .collect();
    Ok(LiquidityReport {
        mean: (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64),
        minimum: raw_values.iter().copied().reduce(f64::min),
        status: with_missing(base, values.len(), input.strategies.len()),
        per_strategy,
    })
}

pub fn concentration_report(input: &PortfolioInput) -> Result<ConcentrationReport, PortfolioError> {
    let allocation = capital_allocation_report(input)?;
    let fractions: Vec<_> = allocation
        .allocations
        .iter()
        .filter_map(|a| a.fraction)
        .collect();
    Ok(ConcentrationReport {
        herfindahl: fractions.iter().map(|fraction| fraction * fraction).sum(),
        largest_fraction: fractions.iter().copied().reduce(f64::max),
        status: if fractions.is_empty() {
            PortfolioStatus::Unknown
        } else {
            allocation.status
        },
    })
}

pub fn risk_budget_report(input: &PortfolioInput) -> Result<RiskBudgetReport, PortfolioError> {
    input.validate()?;
    let total: f64 = input.strategies.iter().filter_map(|s| s.risk_budget).sum();
    let budget_count = input
        .strategies
        .iter()
        .filter(|s| s.risk_budget.is_some())
        .count();
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
    let allocations = input
        .strategies
        .iter()
        .map(|s| MetricReport {
            strategy_ids: vec![s.strategy_id.clone()],
            metric: "declared_risk_budget".into(),
            value: s
                .risk_budget
                .map(|budget| if total > 0.0 { budget / total } else { 0.0 }),
            observations: usize::from(s.risk_budget.is_some()),
            status: if s.risk_budget.is_some() {
                base.clone()
            } else {
                PortfolioStatus::Unknown
            },
        })
        .collect::<Vec<_>>();
    Ok(RiskBudgetReport {
        total_budget: total,
        status: if total > 0.0 {
            with_missing(base, budget_count, input.strategies.len())
        } else if base == PortfolioStatus::Known {
            PortfolioStatus::Unknown
        } else {
            base
        },
        allocations,
    })
}

pub fn constraints_report(input: &PortfolioInput) -> Result<ConstraintReport, PortfolioError> {
    constraints_report_with_scenarios(input, &[])
}

fn constraints_report_with_scenarios(
    input: &PortfolioInput,
    scenarios: &[StressScenario],
) -> Result<ConstraintReport, PortfolioError> {
    input.validate()?;
    let mut outcomes = Vec::new();
    for constraint in &input.constraints {
        let limit = constraint
            .parameters
            .get("max")
            .and_then(|v| v.as_f64())
            .or_else(|| constraint.parameters.get("limit").and_then(|v| v.as_f64()));
        let minimum = constraint.parameters.get("min").and_then(|v| v.as_f64());
        let value = match constraint.kind {
            PortfolioConstraintKind::Correlation => {
                let ids = &constraint.confirmed_strategy_ids;
                if ids.len() != 2 {
                    return Err(PortfolioError::InvalidParameter(
                        "correlation constraint strategies",
                    ));
                }
                correlation_report(input, &ids[0], &ids[1])?.value
            }
            PortfolioConstraintKind::Overlap => {
                let ids = &constraint.confirmed_strategy_ids;
                if ids.len() != 2 {
                    return Err(PortfolioError::InvalidParameter(
                        "overlap constraint strategies",
                    ));
                }
                Some(overlap_report(input, &ids[0], &ids[1])?.jaccard)
            }
            PortfolioConstraintKind::CapitalAllocation => {
                Some(capital_allocation_report(input)?.total_capital)
            }
            PortfolioConstraintKind::Drawdown => Some(drawdown_report(input)?.max_drawdown),
            PortfolioConstraintKind::Capacity => capacity_report(input)?.bottleneck,
            PortfolioConstraintKind::Liquidity => liquidity_report(input)?.minimum,
            PortfolioConstraintKind::RiskBudget => Some(risk_budget_report(input)?.total_budget),
            PortfolioConstraintKind::Concentration => Some(concentration_report(input)?.herfindahl),
            PortfolioConstraintKind::Turnover => turnover_report(input)?.total,
            PortfolioConstraintKind::Stress => stress_report(input, scenarios)?
                .scenarios
                .iter()
                .map(|scenario| scenario.portfolio_return)
                .reduce(f64::min),
        };
        let status = match (value, limit, minimum) {
            (Some(value), Some(limit), _) if value <= limit => PortfolioStatus::Known,
            (Some(value), _, Some(minimum)) if value >= minimum => PortfolioStatus::Known,
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => PortfolioStatus::Abstained,
            _ => PortfolioStatus::Unknown,
        };
        outcomes.push(ConstraintOutcome {
            constraint_id: constraint.constraint_id.clone(),
            kind: constraint.kind.clone(),
            value,
            limit: limit.or(minimum),
            status,
        });
    }
    let aggregate = outcomes.iter().fold(PortfolioStatus::Known, |left, right| {
        combine_status(left, right.status.clone())
    });
    Ok(ConstraintReport {
        constraints: outcomes,
        status: if input.constraints.is_empty() {
            PortfolioStatus::Unknown
        } else {
            aggregate
        },
    })
}

pub fn stress_report(
    input: &PortfolioInput,
    scenarios: &[StressScenario],
) -> Result<StressReport, PortfolioError> {
    input.validate()?;
    let allocation = capital_allocation_report(input)?;
    let mut outcomes = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        if scenario.scenario_id.is_empty() || scenario.returns.values().any(|v| !v.is_finite()) {
            return Err(PortfolioError::InvalidParameter("stress scenario"));
        }
        let mut portfolio_return = 0.0;
        let mut status = allocation.status.clone();
        for share in &allocation.allocations {
            match (share.fraction, scenario.returns.get(&share.strategy_id)) {
                (Some(fraction), Some(value)) => portfolio_return += fraction * value,
                _ => status = PortfolioStatus::Partial,
            }
        }
        outcomes.push(StressOutcome {
            scenario_id: scenario.scenario_id.clone(),
            portfolio_return,
            status,
        });
    }
    Ok(StressReport {
        status: outcomes.iter().map(|outcome| outcome.status.clone()).fold(
            if outcomes.is_empty() {
                PortfolioStatus::Unknown
            } else {
                PortfolioStatus::Known
            },
            combine_status,
        ),
        scenarios: outcomes,
    })
}

pub fn portfolio_report(
    input: &PortfolioInput,
    scenarios: &[StressScenario],
) -> Result<PortfolioReport, PortfolioError> {
    input.validate()?;
    let input_identity = canonical_portfolio_identity(input, scenarios);
    let mut correlations = Vec::new();
    let mut overlaps = Vec::new();
    for (i, left) in input.strategies.iter().enumerate() {
        for right in input.strategies.iter().skip(i + 1) {
            correlations.push(correlation_report(
                input,
                &left.strategy_id,
                &right.strategy_id,
            )?);
            overlaps.push(overlap_report(
                input,
                &left.strategy_id,
                &right.strategy_id,
            )?);
        }
    }
    let mut conditional_correlations = Vec::new();
    for (i, left) in input.strategies.iter().enumerate() {
        for right in input.strategies.iter().skip(i + 1) {
            conditional_correlations.extend(conditional_correlation_report(
                input,
                &left.strategy_id,
                &right.strategy_id,
            )?);
        }
    }
    let mut report = PortfolioReport {
        identity: String::new(),
        input_identity,
        correlations,
        conditional_correlations,
        overlaps,
        capital_allocation: capital_allocation_report(input)?,
        drawdown: drawdown_report(input)?,
        capacity: capacity_report(input)?,
        turnover: turnover_report(input)?,
        liquidity: liquidity_report(input)?,
        concentration: concentration_report(input)?,
        risk_budget: risk_budget_report(input)?,
        constraints: constraints_report_with_scenarios(input, scenarios)?,
        stress: stress_report(input, scenarios)?,
    };
    report.identity = canonical_report_identity(&report);
    Ok(report)
}

fn canonical_portfolio_identity(input: &PortfolioInput, scenarios: &[StressScenario]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"trinityr-portfolio-input-v2\0");
    hash.update(serde_json::to_vec(&(input, scenarios)).expect("portfolio input serializes"));
    format!("{:x}", hash.finalize())
}
