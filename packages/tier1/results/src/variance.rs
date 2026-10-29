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
    /// HC2 and HC3 divide by one minus each observation's leverage, and this many observations
    /// have leverage one (each alone in a cell of the design), so the factor is infinite.
    LeverageOne(usize),
    /// The computed matrix differs from its transpose by more than rounding can explain: a wrong
    /// product, reported as an internal error.
    Asymmetric,
}

/// The matrix made symmetric from its lower triangle, after a check that the two triangles agree
/// to within rounding. A sandwich B M B is symmetric in exact arithmetic, but its two triangles
/// are summed in different orders, and the cancellation in an ill-conditioned bread (a regressor
/// such as a calendar year, far from zero and barely varying) magnifies the difference, so the
/// check is relative to each entry's scale, √(Vᵢᵢ Vⱼⱼ), and to the condition number.
fn symmetrised(
    v: &Matrix,
    condition_number: f64,
    magnitudes: Option<&Matrix>,
) -> Result<Matrix, VarianceError> {
    let k = v.rows();
    let tolerance = (1e-6_f64).max(100.0 * f64::EPSILON * condition_number);
    // An entry that is rounding residue of an exact zero (a rank-one meat, where a column's
    // scores cancel) has no scale of its own: the matrix's largest variance lends it one, and so
    // does the sum of the sizes of the terms that made the entry (`magnitudes`, the product of
    // the absolute bread and meat), which bounds the rounding of a sum however much it cancels;
    // a perfect fit's sandwich is nothing but such cancellation (the formula fuzz target found
    // both cases).
    let largest = (0..k).fold(0.0_f64, |m, i| m.max(v.get(i, i)));
    for i in 0..k {
        for j in 0..i {
            let (a, b) = (v.get(i, j), v.get(j, i));
            let scale = (v.get(i, i) * v.get(j, j))
                .sqrt()
                .max(a.abs().max(b.abs()))
                .max(1e-8 * largest)
                .max(magnitudes.map_or(0.0, |m| m.get(i, j).max(m.get(j, i))));
            if !(a - b).abs().is_finite() || (a - b).abs() > tolerance * scale + f64::MIN_POSITIVE {
                return Err(VarianceError::Asymmetric);
            }
        }
    }
    // A variance that is an exact zero in arithmetic (a coefficient whose scores cancel) may be
    // a negative residue in floating point: within the same bound it is zero, so that a true
    // negative variance, an error of the order of the terms, stays an internal error.
    let diagonal = |i: usize| {
        let value = v.get(i, i);
        let bound = tolerance * (1e-8 * largest).max(magnitudes.map_or(0.0, |m| m.get(i, i)))
            + f64::MIN_POSITIVE;
        if value < 0.0 && -value <= bound {
            0.0
        } else {
            value
        }
    };
    Ok(Matrix::from_fn(k, k, |i, j| {
        if i == j {
            diagonal(i)
        } else if i > j {
            v.get(i, j)
        } else {
            v.get(j, i)
        }
    }))
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
            Ok((symmetrised(&vcov, fitted.condition_number, None)?, df))
        }
        Kind::Hc1 | Kind::Hc2 | Kind::Hc3 => {
            if matches!(kind, Kind::Hc2 | Kind::Hc3) {
                let at_one = (0..fitted.leverage.len())
                    .filter(|&row| 1.0 - observation_leverage(design, fitted, row) <= LEVERAGE_ONE)
                    .count();
                if at_one > 0 {
                    return Err(VarianceError::LeverageOne(at_one));
                }
            }
            // The meat, and beside it the sum of the sizes of its terms, which bounds the
            // rounding of each entry however much the terms cancel.
            let mut meat = Matrix::zeros(k, k);
            let mut meat_bound = Matrix::zeros(k, k);
            for block in design.blocks() {
                let scores = estimator.scores(fitted, &block, block.start);
                let scaled = Matrix::from_fn(scores.rows(), k, |r, c| {
                    let row = block.start + r;
                    scores.get(r, c) * row_factor(kind, design, fitted, row)
                });
                let part = transpose_matmul(&scaled, &scores);
                meat = Matrix::from_fn(k, k, |i, j| meat.get(i, j) + part.get(i, j));
                let sizes = transpose_matmul(&absolute(&scaled), &absolute(&scores));
                meat_bound = Matrix::from_fn(k, k, |i, j| meat_bound.get(i, j) + sizes.get(i, j));
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
            let magnitudes = term_magnitudes(bread, &meat_bound, adjustment);
            Ok((
                symmetrised(&vcov, fitted.condition_number, Some(&magnitudes))?,
                df,
            ))
        }
        Kind::Cluster => {
            let clusters = clusters.expect("clusters for clustered errors");
            if clusters.count < 2 {
                return Err(VarianceError::OneCluster);
            }
            // Each cluster's summed scores, and the sums of their sizes for the rounding bound.
            let mut sums: Vec<Vec<f64>> = vec![vec![0.0; k]; clusters.count];
            let mut sizes: Vec<Vec<f64>> = vec![vec![0.0; k]; clusters.count];
            for block in design.blocks() {
                let scores = estimator.scores(fitted, &block, block.start);
                for r in 0..scores.rows() {
                    let cluster = clusters.of_row[block.start + r];
                    for c in 0..k {
                        sums[cluster][c] += scores.get(r, c);
                        sizes[cluster][c] += scores.get(r, c).abs();
                    }
                }
            }
            let meat = Matrix::from_fn(k, k, |i, j| {
                let products: Vec<f64> = sums.iter().map(|s| s[i] * s[j]).collect();
                sum(&products)
            });
            let meat_bound = Matrix::from_fn(k, k, |i, j| {
                let products: Vec<f64> = sizes.iter().map(|s| s[i] * s[j]).collect();
                sum(&products)
            });
            let sandwich = matmul(&matmul(bread, &meat), bread);
            let g = clusters.count as f64;
            let adjustment = g / (g - 1.0) * (n - 1.0) / (n - k as f64);
            let vcov = Matrix::from_fn(k, k, |i, j| adjustment * sandwich.get(i, j));
            let magnitudes = term_magnitudes(bread, &meat_bound, adjustment);
            Ok((
                symmetrised(&vcov, fitted.condition_number, Some(&magnitudes))?,
                clusters.count - 1,
            ))
        }
    }
}

/// A matrix's entries in size.
fn absolute(m: &Matrix) -> Matrix {
    Matrix::from_fn(m.rows(), m.cols(), |i, j| m.get(i, j).abs())
}

/// The sum of the sizes of the terms of each entry of a sandwich, |B| times the sizes of the
/// meat's terms times |B|, times the adjustment: what the rounding of the entry is bounded by,
/// however much its terms cancel.
fn term_magnitudes(bread: &Matrix, meat_bound: &Matrix, adjustment: f64) -> Matrix {
    let product = matmul(&matmul(&absolute(bread), meat_bound), &absolute(bread));
    Matrix::from_fn(product.rows(), product.cols(), |i, j| {
        adjustment * product.get(i, j)
    })
}

/// The factor a row's score product takes in the meat: 1 under HC1; 1/(1 − h) and 1/(1 − h)²
/// under HC2 and HC3, with h the leverage of one observation; under frequency weights a row
/// stands for w observations each of leverage h/w, whose products sum to the row's divided by w.
/// How close to one an observation's leverage may come before HC2 and HC3 are refused: nearer
/// than this, the factor 1 / (1 − h) is beyond a hundred million and the variance is noise.
const LEVERAGE_ONE: f64 = 1e-8;

/// The leverage of one observation of `row`: the row's leverage, or under frequency weights the
/// row's leverage shared among the observations it stands for.
fn observation_leverage(design: &Design, fitted: &Fitted, row: usize) -> f64 {
    match (design.weight_kind, &design.weights) {
        (Some(WeightKind::Frequency), Some(w)) => fitted.leverage[row] / w[row],
        _ => fitted.leverage[row],
    }
}

fn row_factor(kind: Kind, design: &Design, fitted: &Fitted, row: usize) -> f64 {
    let copies = match (design.weight_kind, &design.weights) {
        (Some(WeightKind::Frequency), Some(w)) => w[row],
        _ => 1.0,
    };
    let leverage = observation_leverage(design, fitted, row);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A matrix from its rows.
    fn of(rows: &[&[f64]]) -> Matrix {
        Matrix::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>())
    }

    /// The two triangles must agree to within rounding at the entry's scale: a difference of a
    /// thousandth is a fault, one at rounding size is not, and the lower triangle is the one
    /// kept.
    #[test]
    fn a_difference_beyond_rounding_is_a_fault() {
        let far = of(&[&[1.0, 1.0], &[1.001, 1.0]]);
        assert!(matches!(
            symmetrised(&far, 1.0, None),
            Err(VarianceError::Asymmetric)
        ));
        let near = of(&[&[1.0, 1.0], &[1.0 + 1e-12, 1.0]]);
        let s = symmetrised(&near, 1.0, None).unwrap();
        assert_eq!((s.get(0, 1), s.get(1, 0)), (1.0 + 1e-12, 1.0 + 1e-12));
    }

    /// An ill-conditioned bread widens the tolerance in proportion to its condition number: the
    /// same thousandth is rounding at a condition number of 10¹² and a fault at 10³.
    #[test]
    fn the_condition_number_widens_the_tolerance() {
        let v = of(&[&[1.0, 1.0], &[1.001, 1.0]]);
        assert!(symmetrised(&v, 1e12, None).is_ok());
        assert!(matches!(
            symmetrised(&v, 1e3, None),
            Err(VarianceError::Asymmetric)
        ));
    }

    /// The scale of an entry is the geometric mean of its two variances: at variances of 10⁻⁶ a
    /// difference of 2 × 10⁻¹¹ is a fault, far beyond a millionth of that scale.
    #[test]
    fn a_difference_is_judged_at_the_entry_scale() {
        let v = of(&[&[1e-6, 1e-6], &[1e-6 + 2e-11, 1e-6]]);
        assert!(matches!(
            symmetrised(&v, 1.0, None),
            Err(VarianceError::Asymmetric)
        ));
    }

    /// An entry with no scale of its own (the residue of an exact zero) borrows a
    /// hundred-millionth of the largest variance: with a largest of 100 that lends 10⁻⁶, so a
    /// difference of 3 × 10⁻¹³ is rounding and one of 10⁻⁸ is a fault.
    #[test]
    fn an_entry_without_scale_borrows_from_the_largest_variance() {
        let residue = of(&[&[100.0, 0.0], &[3e-13, 0.0]]);
        assert!(symmetrised(&residue, 1.0, None).is_ok());
        let fault = of(&[&[100.0, 0.0], &[1e-8, 0.0]]);
        assert!(matches!(
            symmetrised(&fault, 1.0, None),
            Err(VarianceError::Asymmetric)
        ));
    }

    /// Only the difference of the two triangles is judged: entries at the edge of the floating
    /// range, whose sum would overflow, are accepted when they agree.
    #[test]
    fn entries_at_the_edge_of_the_range_are_accepted() {
        let v = of(&[&[1e308, 1e308], &[1e308, 1e308]]);
        let s = symmetrised(&v, 1.0, None).unwrap();
        assert_eq!(s.get(0, 1), 1e308);
    }

    /// The bound is inclusive: a difference of exactly a millionth of the scale is accepted.
    #[test]
    fn a_difference_at_the_bound_is_accepted() {
        let v = of(&[&[1.0, 1e-6], &[0.0, 1.0]]);
        let s = symmetrised(&v, 1.0, None).unwrap();
        assert_eq!((s.get(0, 1), s.get(1, 0)), (0.0, 0.0));
    }

    /// A negative variance within the rounding bound of an exact zero is zero, a larger one
    /// stays what it is for the caller to refuse: with a largest variance of 100 the bound is
    /// 10⁻¹², so −10⁻¹⁴ is zero and −10⁻⁹ is kept.
    #[test]
    fn a_negative_residue_within_the_bound_is_zero() {
        let residue = of(&[&[-1e-14, 0.0], &[0.0, 100.0]]);
        assert_eq!(symmetrised(&residue, 1.0, None).unwrap().get(0, 0), 0.0);
        let negative = of(&[&[-1e-9, 0.0], &[0.0, 100.0]]);
        assert_eq!(symmetrised(&negative, 1.0, None).unwrap().get(0, 0), -1e-9);
    }

    /// With no scale at all (a zero matrix with a residue of the smallest normal size), the
    /// smallest normal number is the bound, so that the residue is zero rather than a negative
    /// variance.
    #[test]
    fn a_residue_without_scale_is_zero() {
        let v = of(&[&[-f64::MIN_POSITIVE]]);
        assert_eq!(symmetrised(&v, 1.0, None).unwrap().get(0, 0), 0.0);
    }
}
