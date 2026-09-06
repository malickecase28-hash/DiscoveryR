//! Bounded statistics and null generators.
//!
//! These routines are descriptive infrastructure. They do not select scientific
//! thresholds or imply independence when observations are blocked or clustered.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

#[derive(Clone, Debug, PartialEq)]
pub enum StatsError {
    EmptySample,
    NonFinite { index: usize },
    InvalidParameter(&'static str),
    DimensionMismatch,
    CapacityExceeded { requested: usize, maximum: usize },
    InvalidOrder,
    DuplicateUnit(String),
    Overflow,
}
impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySample => write!(f, "sample is empty"),
            Self::NonFinite { index } => write!(f, "non-finite value at index {index}"),
            Self::InvalidParameter(p) => write!(f, "invalid parameter: {p}"),
            Self::DimensionMismatch => write!(f, "dimensions do not match"),
            Self::CapacityExceeded { requested, maximum } => {
                write!(f, "requested {requested} exceeds bound {maximum}")
            }
            Self::InvalidOrder => write!(f, "timestamps are not finite, non-negative, or sortable"),
            Self::DuplicateUnit(u) => write!(f, "unit cannot be reused: {u}"),
            Self::Overflow => write!(f, "numeric overflow"),
        }
    }
}
impl Error for StatsError {}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodMetadata {
    pub method: String,
    pub assumptions: Vec<String>,
    pub approximate: bool,
    pub allocation_bound: usize,
    pub seed: Option<u64>,
}
impl MethodMetadata {
    fn new(
        method: &str,
        assumptions: &[&str],
        bound: usize,
        approximate: bool,
        seed: Option<u64>,
    ) -> Self {
        Self {
            method: method.into(),
            assumptions: assumptions.iter().map(|s| (*s).into()).collect(),
            approximate,
            allocation_bound: bound,
            seed,
        }
    }
}

fn finite(v: f64, index: usize) -> Result<(), StatsError> {
    v.is_finite()
        .then_some(())
        .ok_or(StatsError::NonFinite { index })
}
fn finite_slice(values: &[f64]) -> Result<(), StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    for (i, &v) in values.iter().enumerate() {
        finite(v, i)?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentEstimate {
    pub count: u64,
    pub mean: f64,
    pub variance: f64,
    pub population_variance: f64,
    pub metadata: MethodMetadata,
}

#[derive(Clone, Debug, Default)]
pub struct StreamingMoments {
    count: u64,
    mean: f64,
    m2: f64,
}
impl StreamingMoments {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, value: f64) -> Result<(), StatsError> {
        finite(value, self.count as usize)?;
        let n = self.count.checked_add(1).ok_or(StatsError::Overflow)?;
        let delta = value - self.mean;
        let mean = self.mean + delta / n as f64;
        self.m2 += delta * (value - mean);
        if !self.m2.is_finite() || !mean.is_finite() {
            return Err(StatsError::Overflow);
        }
        self.count = n;
        self.mean = mean;
        Ok(())
    }
    pub fn extend(&mut self, values: &[f64]) -> Result<(), StatsError> {
        for &v in values {
            self.update(v)?;
        }
        Ok(())
    }
    pub fn merge(&mut self, other: &Self) -> Result<(), StatsError> {
        if other.count == 0 {
            return Ok(());
        }
        if self.count == 0 {
            *self = other.clone();
            return Ok(());
        }
        let n = self
            .count
            .checked_add(other.count)
            .ok_or(StatsError::Overflow)?;
        let delta = other.mean - self.mean;
        self.m2 += other.m2 + delta * delta * self.count as f64 * other.count as f64 / n as f64;
        self.mean += delta * other.count as f64 / n as f64;
        if !self.m2.is_finite() || !self.mean.is_finite() {
            return Err(StatsError::Overflow);
        }
        self.count = n;
        Ok(())
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn mean(&self) -> Option<f64> {
        (self.count > 0).then_some(self.mean)
    }
    pub fn variance(&self) -> Option<f64> {
        (self.count > 1).then_some(self.m2 / (self.count - 1) as f64)
    }
    pub fn population_variance(&self) -> Option<f64> {
        (self.count > 0).then_some(self.m2 / self.count as f64)
    }
    pub fn estimate(&self) -> Option<MomentEstimate> {
        self.mean().map(|mean| MomentEstimate {
            count: self.count,
            mean,
            variance: self.variance().unwrap_or(0.0),
            population_variance: self.population_variance().unwrap_or(0.0),
            metadata: MethodMetadata::new(
                "welford",
                &["finite values", "sample variance uses n-1"],
                0,
                false,
                None,
            ),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuantileEstimate {
    pub probability: f64,
    pub value: f64,
    pub metadata: MethodMetadata,
}
pub fn exact_quantile(
    values: &[f64],
    probability: f64,
    capacity: usize,
) -> Result<QuantileEstimate, StatsError> {
    finite_slice(values)?;
    if !(0.0..=1.0).contains(&probability) {
        return Err(StatsError::InvalidParameter("probability"));
    }
    if values.len() > capacity {
        return Err(StatsError::CapacityExceeded {
            requested: values.len(),
            maximum: capacity,
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let pos = probability * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let value = sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64);
    Ok(QuantileEstimate {
        probability,
        value,
        metadata: MethodMetadata::new(
            "exact-linear-quantile",
            &[
                "bounded retained sample",
                "linear interpolation of order statistics",
            ],
            capacity,
            false,
            None,
        ),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectSize {
    pub difference: f64,
    pub standardized_difference: Option<f64>,
    pub hedges_g: Option<f64>,
    pub metadata: MethodMetadata,
}
pub fn effect_size(sample_a: &[f64], sample_b: &[f64]) -> Result<EffectSize, StatsError> {
    finite_slice(sample_a)?;
    finite_slice(sample_b)?;
    let mut a = StreamingMoments::new();
    a.extend(sample_a)?;
    let mut b = StreamingMoments::new();
    b.extend(sample_b)?;
    let difference =
        a.mean().ok_or(StatsError::EmptySample)? - b.mean().ok_or(StatsError::EmptySample)?;
    let pooled_df = sample_a
        .len()
        .checked_add(sample_b.len())
        .and_then(|n| n.checked_sub(2))
        .ok_or(StatsError::InvalidParameter("sample size"))?;
    let pooled = if pooled_df > 0 {
        (((sample_a.len() - 1) as f64 * a.variance().unwrap_or(0.0)
            + (sample_b.len() - 1) as f64 * b.variance().unwrap_or(0.0))
            / pooled_df as f64)
            .sqrt()
    } else {
        0.0
    };
    let standardized_difference = (pooled > 0.0).then_some(difference / pooled);
    let hedges_g = standardized_difference.map(|d| {
        let df = pooled_df as f64;
        let correction = 1.0 - 3.0 / (4.0 * df - 1.0);
        d * correction
    });
    Ok(EffectSize {
        difference,
        standardized_difference,
        hedges_g,
        metadata: MethodMetadata::new(
            "two-sample-effect-size",
            &[
                "finite independent samples for pooled standardization",
                "difference is mean(a)-mean(b)",
            ],
            0,
            false,
            None,
        ),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedObservation {
    pub stratum: String,
    pub id: String,
    pub outcome: f64,
    pub covariates: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MatchConfig {
    pub caliper: Option<f64>,
    pub allow_reuse: bool,
    pub max_pairs: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedPair {
    pub treated_id: String,
    pub control_id: String,
    pub distance: f64,
    pub outcome_difference: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedResult {
    pub pairs: Vec<MatchedPair>,
    pub mean_difference: Option<f64>,
    pub metadata: MethodMetadata,
}
pub fn exact_strata_match(
    treated: &[MatchedObservation],
    controls: &[MatchedObservation],
    config: &MatchConfig,
) -> Result<MatchedResult, StatsError> {
    if treated.is_empty() || controls.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if config.max_pairs == 0 {
        return Err(StatsError::InvalidParameter("max_pairs"));
    }
    if config.caliper.is_some_and(|v| !v.is_finite() || v < 0.0) {
        return Err(StatsError::InvalidParameter("caliper"));
    }
    let mut dimensions = None;
    for item in treated.iter().chain(controls) {
        finite(item.outcome, 0)?;
        if item.id.is_empty() || item.stratum.is_empty() {
            return Err(StatsError::InvalidParameter("nonempty id and stratum"));
        }
        if dimensions.is_none() {
            dimensions = Some(item.covariates.len());
        }
        if dimensions != Some(item.covariates.len()) {
            return Err(StatsError::DimensionMismatch);
        }
        for &v in &item.covariates {
            finite(v, 0)?;
        }
    }
    let mut used = BTreeSet::new();
    let mut pairs = Vec::new();
    for t in treated {
        if pairs.len() == config.max_pairs {
            break;
        }
        let mut best: Option<(usize, f64)> = None;
        for (i, c) in controls.iter().enumerate() {
            if c.stratum != t.stratum || (!config.allow_reuse && used.contains(&i)) {
                continue;
            }
            if c.covariates.len() != t.covariates.len() {
                return Err(StatsError::DimensionMismatch);
            }
            let distance = t
                .covariates
                .iter()
                .zip(&c.covariates)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            if config.caliper.is_some_and(|caliper| distance > caliper) {
                continue;
            }
            if best
                .is_none_or(|(bi, bd)| distance < bd || (distance == bd && c.id < controls[bi].id))
            {
                best = Some((i, distance));
            }
        }
        if let Some((i, distance)) = best {
            let c = &controls[i];
            if !config.allow_reuse {
                used.insert(i);
            }
            pairs.push(MatchedPair {
                treated_id: t.id.clone(),
                control_id: c.id.clone(),
                distance,
                outcome_difference: t.outcome - c.outcome,
            });
        }
    }
    let mean_difference = (!pairs.is_empty())
        .then(|| pairs.iter().map(|p| p.outcome_difference).sum::<f64>() / pairs.len() as f64);
    Ok(MatchedResult {
        pairs,
        mean_difference,
        metadata: MethodMetadata::new(
            "exact-strata-nearest-control",
            &[
                "stratum and pre-outcome covariates define eligibility",
                "controls are not reused unless allow_reuse is true",
                "unmatched treated observations remain represented by pair count",
            ],
            treated.len().min(config.max_pairs),
            false,
            None,
        ),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockObservation {
    pub block: String,
    pub value: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GroupedObservation {
    pub block: String,
    pub group: bool,
    pub value: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MonteCarloSummary {
    pub observed: f64,
    pub exceedances: usize,
    pub draws: usize,
    pub p_value: f64,
    pub metadata: MethodMetadata,
}

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn index(&mut self, upper: usize) -> usize {
        let upper = upper as u64;
        let limit = u64::MAX - u64::MAX % upper;
        loop {
            let draw = self.next();
            if draw < limit {
                return (draw % upper) as usize;
            }
        }
    }
    fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            values.swap(i, self.index(i + 1));
        }
    }
}
fn blocks<T>(values: &[T], block: impl Fn(&T) -> &str) -> Vec<String> {
    // ponytail: linear block discovery; use an indexed map only if huge block counts matter.
    let mut out = Vec::new();
    for v in values {
        let b = block(v).to_string();
        if !out.contains(&b) {
            out.push(b);
        }
    }
    out
}
fn mean(values: impl Iterator<Item = f64>, n: usize) -> Result<f64, StatsError> {
    if n == 0 {
        return Err(StatsError::EmptySample);
    }
    let total: f64 = values.sum();
    let result = total / n as f64;
    result
        .is_finite()
        .then_some(result)
        .ok_or(StatsError::Overflow)
}
pub fn bootstrap_by_block(
    values: &[BlockObservation],
    replicates: usize,
    seed: u64,
    max_replicates: usize,
) -> Result<MonteCarloSummary, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if replicates == 0 || replicates > max_replicates {
        return Err(StatsError::CapacityExceeded {
            requested: replicates,
            maximum: max_replicates,
        });
    }
    for (i, v) in values.iter().enumerate() {
        finite(v.value, i)?;
        if v.block.is_empty() {
            return Err(StatsError::InvalidParameter("block"));
        }
    }
    let names = blocks(values, |v| &v.block);
    let observed = mean(values.iter().map(|v| v.value), values.len())?;
    let mut rng = Rng::new(seed);
    let mut exceedances = 0;
    for _ in 0..replicates {
        let mut total = 0.0;
        let mut n = 0;
        while n < values.len() {
            let block = &names[rng.index(names.len())];
            for v in values.iter().filter(|v| &v.block == block) {
                // Center the resampled world at zero before the tail test;
                // otherwise a bootstrap distribution around the observation
                // is incorrectly presented as a null p-value.
                total += v.value - observed;
                n += 1;
                if n == values.len() {
                    break;
                }
            }
        }
        let replicate = total / n as f64;
        if replicate.abs() >= observed.abs() {
            exceedances += 1;
        }
    }
    Ok(MonteCarloSummary {
        observed,
        exceedances,
        draws: replicates,
        p_value: (exceedances + 1) as f64 / (replicates + 1) as f64,
        metadata: MethodMetadata::new(
            "block-bootstrap",
            &[
                "whole blocks sampled with replacement",
                "values are centered at the observed mean for a declared null tail",
                "each replicate retains exactly the input row count",
                "xorshift64* deterministic non-cryptographic RNG",
            ],
            replicates,
            true,
            Some(seed),
        ),
    })
}
pub fn block_permutation(
    values: &[GroupedObservation],
    replicates: usize,
    seed: u64,
    max_replicates: usize,
) -> Result<MonteCarloSummary, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if replicates == 0 || replicates > max_replicates {
        return Err(StatsError::CapacityExceeded {
            requested: replicates,
            maximum: max_replicates,
        });
    }
    for (i, v) in values.iter().enumerate() {
        finite(v.value, i)?;
        if v.block.is_empty() {
            return Err(StatsError::InvalidParameter("block"));
        }
    }
    let names = blocks(values, |v| &v.block);
    let observed = grouped_difference(values)?;
    let mut rng = Rng::new(seed);
    let mut exceedances = 0;
    for _ in 0..replicates {
        let mut shuffled = values.to_vec();
        for block in &names {
            let mut labels: Vec<bool> = shuffled
                .iter()
                .filter(|v| &v.block == block)
                .map(|v| v.group)
                .collect();
            rng.shuffle(&mut labels);
            for (j, v) in shuffled
                .iter_mut()
                .filter(|v| &v.block == block)
                .enumerate()
            {
                v.group = labels[j];
            }
        }
        let stat = grouped_difference(&shuffled)?;
        if stat.abs() >= observed.abs() {
            exceedances += 1;
        }
    }
    Ok(MonteCarloSummary {
        observed,
        exceedances,
        draws: replicates,
        p_value: (exceedances + 1) as f64 / (replicates + 1) as f64,
        metadata: MethodMetadata::new(
            "within-block-permutation",
            &[
                "labels are shuffled within declared blocks",
                "block membership and values remain intact",
                "finite-sample +1 p rule",
            ],
            replicates,
            true,
            Some(seed),
        ),
    })
}
fn grouped_difference(values: &[GroupedObservation]) -> Result<f64, StatsError> {
    let (mut a, mut na, mut b, mut nb) = (0.0, 0, 0.0, 0);
    for v in values {
        if v.group {
            a += v.value;
            na += 1;
        } else {
            b += v.value;
            nb += 1;
        }
    }
    if na == 0 || nb == 0 {
        return Err(StatsError::InvalidParameter("both groups required"));
    }
    Ok(a / na as f64 - b / nb as f64)
}

pub fn circular_time_shift_null(
    x: &[f64],
    y: &[f64],
    group_sizes: &[usize],
    replicates: usize,
    seed: u64,
    max_replicates: usize,
) -> Result<MonteCarloSummary, StatsError> {
    if x.is_empty() || y.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if x.len() != y.len() {
        return Err(StatsError::DimensionMismatch);
    }
    if replicates == 0 || replicates > max_replicates {
        return Err(StatsError::CapacityExceeded {
            requested: replicates,
            maximum: max_replicates,
        });
    }
    if group_sizes.is_empty()
        || group_sizes.iter().sum::<usize>() != x.len()
        || group_sizes.contains(&0)
    {
        return Err(StatsError::InvalidParameter("group_sizes"));
    }
    finite_slice(x)?;
    finite_slice(y)?;
    let observed = correlation(x, y)?;
    let mut rng = Rng::new(seed);
    let mut exceedances = 0;
    let mut shifted = vec![0.0; x.len()];
    for _ in 0..replicates {
        let mut start = 0;
        for &size in group_sizes {
            let shift = rng.index(size);
            for i in 0..size {
                shifted[start + i] = y[start + (i + shift) % size];
            }
            start += size;
        }
        if correlation(x, &shifted)?.abs() >= observed.abs() {
            exceedances += 1;
        }
    }
    Ok(MonteCarloSummary {
        observed,
        exceedances,
        draws: replicates,
        p_value: (exceedances + 1) as f64 / (replicates + 1) as f64,
        metadata: MethodMetadata::new(
            "grouped-circular-time-shift",
            &[
                "ordering is preserved within each declared group",
                "units do not cross group boundaries",
                "xorshift64* deterministic non-cryptographic RNG",
            ],
            replicates,
            true,
            Some(seed),
        ),
    })
}
fn correlation(x: &[f64], y: &[f64]) -> Result<f64, StatsError> {
    let mx = x.iter().sum::<f64>() / x.len() as f64;
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let (mut n, mut dx, mut dy) = (0.0, 0.0, 0.0);
    for (&a, &b) in x.iter().zip(y) {
        let aa = a - mx;
        let bb = b - my;
        n += aa * bb;
        dx += aa * aa;
        dy += bb * bb;
    }
    if dx == 0.0 || dy == 0.0 {
        return Err(StatsError::InvalidParameter("constant series"));
    }
    Ok(n / (dx * dy).sqrt())
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvidenceStatus {
    Known,
    Null,
    Rejected,
    Missing,
    Failed,
    Inconclusive,
    Contradictory,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FamilyEntry {
    pub p_value: Option<f64>,
    pub status: EvidenceStatus,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FdrResult {
    pub adjusted_p_values: Vec<Option<f64>>,
    pub rejected: Vec<bool>,
    pub tested: usize,
    pub family_size: usize,
    pub metadata: MethodMetadata,
}
pub fn fdr_bh_by(entries: &[FamilyEntry], q: f64, by: bool) -> Result<FdrResult, StatsError> {
    if entries.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if !(0.0..=1.0).contains(&q) {
        return Err(StatsError::InvalidParameter("q"));
    }
    let mut valid = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        if let Some(p) = e.p_value {
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                return Err(StatsError::InvalidParameter("p_value"));
            }
            valid.push((i, p));
        }
    }
    let m = valid.len();
    let c = if by {
        (1..=m).map(|i| 1.0 / i as f64).sum::<f64>()
    } else {
        1.0
    };
    let mut sorted = valid.clone();
    sorted.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    let mut adj = vec![None; entries.len()];
    let mut running = 1.0;
    for (rank, &(i, p)) in sorted.iter().enumerate().rev() {
        let value = (p * m as f64 * c / (rank + 1) as f64).min(running);
        running = value;
        adj[i] = Some(value);
    }
    let mut rejected = vec![false; entries.len()];
    let mut threshold = 0.0;
    for (rank, &(_, p)) in sorted.iter().enumerate() {
        if p <= q * (rank + 1) as f64 / (m.max(1) as f64) * 1.0 / c {
            threshold = p;
        }
    }
    for (i, e) in entries.iter().enumerate() {
        rejected[i] = e.p_value.is_some_and(|p| p <= threshold);
    }
    Ok(FdrResult {
        adjusted_p_values: adj,
        rejected,
        tested: m,
        family_size: entries.len(),
        metadata: MethodMetadata::new(
            if by { "BY-FDR" } else { "BH-FDR" },
            &[
                "missing and unsuccessful family members remain indexed",
                "q is caller supplied",
                "only finite p-values enter the ranked family",
            ],
            entries.len(),
            true,
            None,
        ),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaxStatisticResult {
    pub p_values: Vec<f64>,
    pub draws: usize,
    pub metadata: MethodMetadata,
}

pub fn max_statistic(
    observed: &[f64],
    null_worlds: &[Vec<f64>],
    max_worlds: usize,
) -> Result<MaxStatisticResult, StatsError> {
    if null_worlds.is_empty() {
        return Err(StatsError::EmptySample);
    }
    let mut accumulator = MaxStatisticAccumulator::new(observed, max_worlds)?;
    for world in null_worlds {
        accumulator.push_world(world)?;
    }
    Ok(MaxStatisticResult {
        p_values: accumulator.finish()?,
        draws: accumulator.draws(),
        metadata: MethodMetadata::new(
            "joint-null-max-statistic",
            &[
                "each family member uses the same joint null worlds",
                "tail count uses absolute statistics and the finite-sample +1 rule",
                "null worlds are streamed through a bounded accumulator",
            ],
            max_worlds,
            true,
            None,
        ),
    })
}

pub fn max_statistic_p_values(
    observed: &[f64],
    null_worlds: &[Vec<f64>],
    max_worlds: usize,
) -> Result<Vec<f64>, StatsError> {
    Ok(max_statistic(observed, null_worlds, max_worlds)?.p_values)
}

/// Streaming max-statistic family counter. Null worlds are consumed one at a
/// time, so memory is bounded by the declared statistic dimension.
#[derive(Clone, Debug)]
pub struct MaxStatisticAccumulator {
    observed: Vec<f64>,
    exceedances: Vec<usize>,
    draws: usize,
    max_worlds: usize,
}
impl MaxStatisticAccumulator {
    pub fn new(observed: &[f64], max_worlds: usize) -> Result<Self, StatsError> {
        finite_slice(observed)?;
        if max_worlds == 0 {
            return Err(StatsError::InvalidParameter("max_worlds"));
        }
        Ok(Self {
            observed: observed.to_vec(),
            exceedances: vec![0; observed.len()],
            draws: 0,
            max_worlds,
        })
    }
    pub fn push_world(&mut self, world: &[f64]) -> Result<(), StatsError> {
        if self.draws == self.max_worlds {
            return Err(StatsError::CapacityExceeded {
                requested: self.draws + 1,
                maximum: self.max_worlds,
            });
        }
        if world.len() != self.observed.len() {
            return Err(StatsError::DimensionMismatch);
        }
        let mut maximum: f64 = 0.0;
        for (i, &value) in world.iter().enumerate() {
            finite(value, i)?;
            maximum = maximum.max(value.abs());
        }
        for (i, &value) in self.observed.iter().enumerate() {
            if maximum >= value.abs() {
                self.exceedances[i] += 1;
            }
        }
        self.draws += 1;
        Ok(())
    }
    pub fn finish(&self) -> Result<Vec<f64>, StatsError> {
        if self.draws == 0 {
            return Err(StatsError::EmptySample);
        }
        Ok(self
            .exceedances
            .iter()
            .map(|&n| (n + 1) as f64 / (self.draws + 1) as f64)
            .collect())
    }
    pub fn draws(&self) -> usize {
        self.draws
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Concentration {
    pub cluster: String,
    pub count: usize,
    pub fraction: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SupportReport {
    pub total: usize,
    pub clusters: Vec<Concentration>,
    pub metadata: MethodMetadata,
}
pub fn support_concentration(
    clusters: &[String],
    max_clusters: usize,
) -> Result<SupportReport, StatsError> {
    if clusters.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if clusters.len() > max_clusters {
        return Err(StatsError::CapacityExceeded {
            requested: clusters.len(),
            maximum: max_clusters,
        });
    }
    let mut counts = BTreeMap::new();
    for c in clusters {
        if c.is_empty() {
            return Err(StatsError::InvalidParameter("cluster"));
        }
        *counts.entry(c.clone()).or_insert(0) += 1;
    }
    let total = clusters.len();
    Ok(SupportReport {
        total,
        clusters: counts
            .into_iter()
            .map(|(cluster, count)| Concentration {
                cluster,
                count,
                fraction: count as f64 / total as f64,
            })
            .collect(),
        metadata: MethodMetadata::new(
            "cluster-support-concentration",
            &[
                "support is reported for every observed cluster",
                "no threshold deletes a cluster",
            ],
            max_clusters,
            false,
            None,
        ),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalObservation {
    pub time: f64,
    pub event: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalPoint {
    pub time: f64,
    pub at_risk: usize,
    pub events: usize,
    pub censored: usize,
    pub survival: f64,
    pub hazard_increment: f64,
    pub cumulative_hazard: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalEstimate {
    pub points: Vec<SurvivalPoint>,
    pub metadata: MethodMetadata,
}
pub fn kaplan_meier(
    observations: &[SurvivalObservation],
    max_events: usize,
) -> Result<SurvivalEstimate, StatsError> {
    if observations.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if observations.len() > max_events {
        return Err(StatsError::CapacityExceeded {
            requested: observations.len(),
            maximum: max_events,
        });
    }
    let mut data = observations.to_vec();
    for (o, i) in data.iter().zip(0..) {
        if !o.time.is_finite() || o.time < 0.0 {
            return Err(StatsError::InvalidOrder);
        }
        finite(o.time, i)?;
    }
    data.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal));
    let mut points = Vec::new();
    let mut at_risk = data.len();
    let mut survival = 1.0;
    let mut cumulative = 0.0;
    let mut i = 0;
    while i < data.len() {
        let time = data[i].time;
        let mut events = 0;
        let mut censored = 0;
        while i < data.len() && data[i].time == time {
            if data[i].event {
                events += 1
            } else {
                censored += 1
            }
            i += 1;
        }
        if at_risk == 0 {
            break;
        }
        let hazard = events as f64 / at_risk as f64;
        survival *= 1.0 - hazard;
        cumulative += hazard;
        points.push(SurvivalPoint {
            time,
            at_risk,
            events,
            censored,
            survival,
            hazard_increment: hazard,
            cumulative_hazard: cumulative,
        });
        at_risk -= events + censored;
    }
    Ok(SurvivalEstimate {
        points,
        metadata: MethodMetadata::new(
            "kaplan-meier-nelson-aalen",
            &[
                "event and censor ties share the pre-time at-risk denominator",
                "censors leave survival unchanged",
                "stable time ordering with one pass over sorted observations",
            ],
            max_events,
            false,
            None,
        ),
    })
}
