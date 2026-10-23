//! The variance layer (Engine and Project Specification, section 8.3, point 2): one computation
//! of every kind of standard error from an estimator's scores and bread, for every model. The
//! rulings of release 0.4.0: `classical` is σ²(XᵀX)⁻¹; `robust` is HC1, N/(N − k) times the
//! sandwich of the scores; `hc2` and `hc3` scale each observation's score by its leverage, with
//! no further factor; a
//! cluster is a dependence structure, the scores summed within it (CR1), with Stata's
//! small-sample adjustment G/(G − 1) · (N − 1)/(N − k); the t table uses N − k degrees of
//! freedom, or G − 1 when clustered. Under frequency weights every row stands for its weight's
//! worth of observations, so N is the sum of the weights and each row's score sum carries its
//! weight once; analytic and sampling weights enter the scores as the weighted design carries
//! them.

use crate::design::Design;
use crate::estimator::{Estimator, Fitted};
use zeff_kernels::product::{matmul, transpose_matmul};
use zeff_kernels::{Matrix, sum};
use zeff_numeric::order::WeightKind;

/// The kinds of standard error (Language Reference, section 6, `se:` and `cluster:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Classical,
    Hc1,
    Hc2,
    Hc3,
    /// CR1 over the clusters given.
    Cluster,
}

/// The clusters of the rows used: each row's cluster, the number of clusters and how many hold
/// one observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clusters {
    pub of_row: Vec<usize>,
    pub count: usize,
    pub singletons: usize,
}

impl Clusters {
    /// The clusters from a key per row used, in the order the keys first appear.
    pub fn from_keys(keys: &[String]) -> Clusters {
        let mut index: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        let mut sizes: Vec<usize> = Vec::new();
        let of_row = keys
            .iter()
            .map(|key| {
                let next = sizes.len();
                let cluster = *index.entry(key.as_str()).or_insert(next);
                if cluster == sizes.len() {
                    sizes.push(0);
                }
                sizes[cluster] += 1;
                cluster
            })
            .collect();
        Clusters {
            of_row,
            count: sizes.len(),
            singletons: sizes.iter().filter(|s| **s == 1).count(),
        }
    }
}

/// Why no variance could be computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarianceError {
    /// Clustered errors need two clusters at least.
    OneCluster,
}

/// The variance of the estimates under `kind`, and the degrees of freedom of the t table.
pub fn variance(
    kind: Kind,
    estimator: &dyn Estimator,
    design: &Design,
    fitted: &Fitted,
    bread: &Matrix,
    clusters: Option<&Clusters>,
) -> Result<(Matrix, usize), VarianceError> {
    let k = fitted.kept.len();
    // N: the rows used, or the sum of the frequency weights.
    let n = match design.weight_kind {
        Some(WeightKind::Frequency) => design.weight_total(),
        _ => design.n() as f64,
    };
    let df = (n - k as f64).max(0.0) as usize;
    match kind {
        Kind::Classical => {
            let sigma2 = fitted.residual_sum_of_squares / (n - k as f64);
            let vcov = Matrix::from_fn(k, k, |i, j| sigma2 * bread.get(i, j));
            Ok((vcov, df))
        }
        Kind::Hc1 | Kind::Hc2 | Kind::Hc3 => {
            let mut meat = Matrix::zeros(k, k);
            for block in design.blocks() {
                let scores = estimator.scores(fitted, &block, block.start);
                let scaled = Matrix::from_fn(scores.rows(), k, |r, c| {
                    let row = block.start + r;
                    scores.get(r, c) * row_factor(kind, design, fitted, row)
                });
                let part = transpose_matmul(&scaled, &scores);
                meat = Matrix::from_fn(k, k, |i, j| meat.get(i, j) + part.get(i, j));
            }
            let sandwich = matmul(&matmul(bread, &meat), bread);
            // HC1 is HC0 scaled by N/(N − k); HC2 and HC3 correct by the leverage alone, as R's
            // sandwich and Stata's hc2 and hc3 options do.
            let adjustment = if kind == Kind::Hc1 {
                n / (n - k as f64)
            } else {
                1.0
            };
            let vcov = Matrix::from_fn(k, k, |i, j| adjustment * sandwich.get(i, j));
            Ok((vcov, df))
        }
        Kind::Cluster => {
            let clusters = clusters.expect("clusters for clustered errors");
            if clusters.count < 2 {
                return Err(VarianceError::OneCluster);
            }
            let mut sums: Vec<Vec<f64>> = vec![vec![0.0; k]; clusters.count];
            for block in design.blocks() {
                let scores = estimator.scores(fitted, &block, block.start);
                for r in 0..scores.rows() {
                    let cluster = clusters.of_row[block.start + r];
                    for (c, total) in sums[cluster].iter_mut().enumerate() {
                        *total += scores.get(r, c);
                    }
                }
            }
            let meat = Matrix::from_fn(k, k, |i, j| {
                let products: Vec<f64> = sums.iter().map(|s| s[i] * s[j]).collect();
                sum(&products)
            });
            let sandwich = matmul(&matmul(bread, &meat), bread);
            let g = clusters.count as f64;
            let adjustment = g / (g - 1.0) * (n - 1.0) / (n - k as f64);
            let vcov = Matrix::from_fn(k, k, |i, j| adjustment * sandwich.get(i, j));
            Ok((vcov, clusters.count - 1))
        }
    }
}

/// The factor a row's score product takes in the meat: 1 under HC1; 1/(1 − h) and 1/(1 − h)²
/// under HC2 and HC3, with h the leverage of one observation; under frequency weights a row
/// stands for w observations each of leverage h/w, whose products sum to the row's divided by w.
fn row_factor(kind: Kind, design: &Design, fitted: &Fitted, row: usize) -> f64 {
    let (copies, leverage) = match (design.weight_kind, &design.weights) {
        (Some(WeightKind::Frequency), Some(w)) => (w[row], fitted.leverage[row] / w[row]),
        _ => (1.0, fitted.leverage[row]),
    };
    let base = 1.0 / copies;
    match kind {
        Kind::Hc2 => base / (1.0 - leverage),
        Kind::Hc3 => base / ((1.0 - leverage) * (1.0 - leverage)),
        _ => base,
    }
}

/// The effective sample size of the weights, (Σw)² / Σw² (Kish), and the rows: the extreme-weight
/// warning (W1003) compares the two.
pub fn effective_sample_size(weights: &[f64]) -> f64 {
    let total = sum(weights);
    let squares: Vec<f64> = weights.iter().map(|w| w * w).collect();
    let total_squares = sum(&squares);
    if total_squares > 0.0 {
        total * total / total_squares
    } else {
        0.0
    }
}
