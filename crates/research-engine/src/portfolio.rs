//! Descriptive, deterministic portfolio research over confirmed strategy streams.
//!
//! This module reports portfolio relationships and declared constraints. It does
//! not optimize weights, authorize confirmation, place orders, or allocate live
//! capital.

use std::collections::{BTreeMap, BTreeSet};

use research_contracts::{
    evidence::{ConfirmationState, EvidenceState},
    PortfolioComponent, PortfolioConstraint, PortfolioConstraintKind,
};

use crate::stats::{StatsError, StreamingMoments};

#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioInput {
    pub component: PortfolioComponent,
    pub strategies: Vec<StrategyStream>,
    pub constraints: Vec<PortfolioConstraint>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct MetricReport {
    pub strategy_ids: Vec<String>,
    pub metric: String,
    pub value: Option<f64>,
    pub observations: usize,
    pub status: PortfolioStatus,
}
pub type CorrelationReport = MetricReport;

#[derive(Clone, Debug, PartialEq)]
pub struct CapacityReport {
    pub total_capacity: Option<f64>,
    pub bottleneck: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TurnoverReport {
    pub total: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiquidityReport {
    pub mean: Option<f64>,
    pub minimum: Option<f64>,
    pub per_strategy: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RiskBudgetReport {
    pub total_budget: f64,
    pub allocations: Vec<MetricReport>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalCorrelationReport {
    pub strategy_ids: [String; 2],
    pub regime: String,
    pub value: Option<f64>,
    pub observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlapReport {
    pub strategy_ids: [String; 2],
    pub intersection: usize,
    pub union: usize,
    pub jaccard: f64,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AllocationShare {
    pub strategy_id: String,
    pub capital: Option<f64>,
    pub fraction: Option<f64>,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AllocationReport {
    pub allocations: Vec<AllocationShare>,
    pub total_capital: f64,
    pub status: PortfolioStatus,
}
pub type CapitalAllocationReport = AllocationReport;

#[derive(Clone, Debug, PartialEq)]
pub struct DrawdownReport {
    pub max_drawdown: f64,
    pub strategy_drawdowns: Vec<(String, Option<f64>)>,
    pub observations: usize,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConcentrationReport {
    pub herfindahl: f64,
    pub largest_fraction: Option<f64>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintOutcome {
    pub constraint_id: String,
    pub kind: PortfolioConstraintKind,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintReport {
    pub constraints: Vec<ConstraintOutcome>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StressScenario {
    pub scenario_id: String,
    pub returns: BTreeMap<String, f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StressOutcome {
    pub scenario_id: String,
    pub portfolio_return: f64,
    pub status: PortfolioStatus,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StressReport {
    pub scenarios: Vec<StressOutcome>,
    pub status: PortfolioStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioReport {
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

fn status(states: impl Iterator<Item = EvidenceState>) -> PortfolioStatus {
    let states: Vec<_> = states.collect();
    if states.iter().any(|s| *s == EvidenceState::Contradictory) {
        PortfolioStatus::Contradictory
    } else if states.iter().any(|s| *s == EvidenceState::Unknown) {
        PortfolioStatus::Unknown
    } else if states.iter().any(|s| *s == EvidenceState::Null) {
        PortfolioStatus::Null
    } else if states.iter().any(|s| *s == EvidenceState::Rejected) {
        PortfolioStatus::Abstained
    } else if states.iter().any(|s| *s == EvidenceState::Inconclusive) {
        PortfolioStatus::Inconclusive
    } else if states.iter().any(|s| *s == EvidenceState::Partial) {
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
        .ok_or_else(|| PortfolioError::InvalidParameter("constant series"))
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
    Ok(grouped
        .into_iter()
        .map(|(regime, (x, y, observations))| match pearson(&x, &y) {
            Ok(value) => ConditionalCorrelationReport {
                strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
                regime,
                value: Some(value),
                observations,
                status: base.clone(),
            },
            Err(_) => ConditionalCorrelationReport {
                strategy_ids: [left.strategy_id.clone(), right.strategy_id.clone()],
                regime,
                value: None,
                observations,
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
            base
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
    let mut portfolio = Vec::new();
    for i in 0..n {
        let values: Vec<_> = input
            .strategies
            .iter()
            .filter_map(|s| s.returns.get(i).and_then(|v| *v))
            .collect();
        if values.len() == input.strategies.len() {
            portfolio.push(values.iter().sum::<f64>() / values.len() as f64);
        }
    }
    let base = status(input.strategies.iter().map(|s| s.evidence_state.clone()));
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
        status: with_missing(base, portfolio.len(), n),
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
            base
        } else if base == PortfolioStatus::Known {
            PortfolioStatus::Unknown
        } else {
            base
        },
        allocations,
    })
}

pub fn constraints_report(input: &PortfolioInput) -> Result<ConstraintReport, PortfolioError> {
    input.validate()?;
    Ok(ConstraintReport {
        constraints: input
            .constraints
            .iter()
            .map(|constraint| ConstraintOutcome {
                constraint_id: constraint.constraint_id.clone(),
                kind: constraint.kind.clone(),
                status: PortfolioStatus::Known,
            })
            .collect(),
        status: PortfolioStatus::Known,
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
    Ok(PortfolioReport {
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
        constraints: constraints_report(input)?,
        stress: stress_report(input, scenarios)?,
    })
}
