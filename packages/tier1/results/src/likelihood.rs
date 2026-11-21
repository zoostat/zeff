//! The likelihood trait (Engine and Project Specification, section 14, the row of 0.9.0; section
//! 8.3, point 2): a model fitted by maximum likelihood gives, for each row and each value of its
//! linear predictor, the row's log-likelihood, its score and its information, which the shared
//! optimiser of the kernels (`zeff_kernels::irls`) maximises, and the fitted mean and the
//! deviance, which the result and the variance layer read. It lives here, beside the estimator
//! trait, so that every model package fits through one optimiser; the families live here too, so
//! that a later package with a `family:` (`mixed`, `penalized`) shares them without depending on
//! `glm` (section 8.1, rule 5).
//!
//! At 1.0 the family is the binomial, with the logit and probit links (open decision #112). Its
//! arithmetic is the engine's own (`zeff_numeric`): the exponential and the logarithm, and the
//! normal distribution's two tails from `erfc`, each tail keeping its relative accuracy, so that
//! a probability near 0 or near 1 loses no digit of its log and no row's information vanishes
//! before it must.

use crate::design::{Block, Design};
use crate::estimator::{Estimator, Fitted};
use zeff_core::runtime::{CallContext, Fail, fail};
use zeff_diagnostics::Diagnostic;
use zeff_kernels::irls::{IndexModel, RowTerms, Solution};
use zeff_kernels::{Matrix, sum};
use zeff_numeric::maths::{exp, log, log1p};
use zeff_numeric::special::{normal_density, normal_quantile, normal_tails};

/// A model fitted by maximum likelihood through one linear predictor per row.
pub trait Likelihood: IndexModel {
    /// The fitted mean of a row at its linear predictor.
    fn mean(&self, eta: f64) -> f64;
    /// The row's contribution to the deviance, 2(ℓ of the saturated model − ℓ), at `eta`.
    fn deviance(&self, row: usize, eta: f64) -> f64;
    /// The linear predictor at which the iterations start for `row`.
    fn start(&self, row: usize) -> f64;
}

/// The link of a binary model (`glm(link:)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Logit,
    Probit,
}

impl Link {
    /// The word `link:` takes.
    pub fn name(self) -> &'static str {
        match self {
            Link::Logit => "logit",
            Link::Probit => "probit",
        }
    }

    /// The probability at a linear predictor: the inverse of the link.
    pub fn mean(self, eta: f64) -> f64 {
        match self {
            Link::Logit => logistic(eta).0,
            Link::Probit => normal_tails(eta).lower,
        }
    }

    /// The linear predictor of a probability strictly between 0 and 1: the link.
    pub fn link(self, p: f64) -> f64 {
        match self {
            Link::Logit => log(p / (1.0 - p)),
            Link::Probit => normal_quantile(p, false),
        }
    }
}

/// The binomial family: an outcome in [0, 1] per row (0 or 1 for a binary outcome, a share of
/// successes with the number of trials as the prior weight), each row with a prior weight.
#[derive(Clone, Debug, PartialEq)]
pub struct Binomial {
    pub y: Vec<f64>,
    /// One prior weight per row: 1, or the row's frequency weight.
    pub prior: Vec<f64>,
    pub link: Link,
}

impl Binomial {
    /// The family of rows `y`, each of prior weight one.
    pub fn new(y: Vec<f64>, link: Link) -> Binomial {
        let prior = vec![1.0; y.len()];
        Binomial { y, prior, link }
    }
}

/// p = 1/(1 + e^−η) and 1 − p, each computed so that it keeps its relative accuracy far into its
/// tail (the smaller of the two from e^−|η| directly).
fn logistic(eta: f64) -> (f64, f64) {
    let e = exp(-eta.abs());
    let small = e / (1.0 + e);
    let large = 1.0 / (1.0 + e);
    if eta >= 0.0 {
        (large, small)
    } else {
        (small, large)
    }
}

/// log(1 + e^η), without overflow and without losing the digits of a small value.
fn softplus(eta: f64) -> f64 {
    if eta > 0.0 {
        eta + log1p(exp(-eta))
    } else {
        log1p(exp(eta))
    }
}

/// Beyond this distance from zero the normal density is below 1e-297 and the tail ratios are
/// taken from their asymptotic series, where the density itself would underflow.
const FAR: f64 = 37.0;

/// √(2π), as `normal_density` writes it.
const SQRT_2PI: f64 = 2.506_628_274_631_000_5;

/// The Mills ratio of the upper tail, (1 − Φ(x))/φ(x), for x ≥ FAR, by its asymptotic series
/// 1/x · (1 − 1/x² + 3/x⁴ − 15/x⁶ + …), eight terms: at x = 37 the next term is below 1e-19 of
/// the sum.
fn mills_far(x: f64) -> f64 {
    let inverse_square = 1.0 / (x * x);
    let mut term = 1.0;
    let mut total = 1.0;
    for k in 1..=8 {
        term *= -((2 * k - 1) as f64) * inverse_square;
        total += term;
    }
    total / x
}

/// λ(x) = φ(x)/Φ(x), the score of log Φ at x, and log Φ(x), each to full precision in both tails.
fn lower_tail(x: f64) -> (f64, f64) {
    if x < -FAR {
        // Φ(x) = φ(x)·M(−x), M the Mills ratio: λ = 1/M(−x), log Φ = log φ(x) + log M(−x).
        let m = mills_far(-x);
        let log_density = -0.5 * x * x - log(SQRT_2PI);
        (1.0 / m, log_density + log(m))
    } else {
        let tails = normal_tails(x);
        let ratio = normal_density(x) / tails.lower;
        let log_lower = if tails.lower > 0.5 {
            log1p(-tails.upper)
        } else {
            log(tails.lower)
        };
        (ratio, log_lower)
    }
}

impl IndexModel for Binomial {
    fn rows(&self) -> usize {
        self.y.len()
    }

    fn at(&self, row: usize, eta: f64, observed: bool) -> RowTerms {
        let y = self.y[row];
        let m = self.prior[row];
        match self.link {
            Link::Logit => {
                // The canonical link: the observed information is the expected.
                let (p, q) = logistic(eta);
                RowTerms {
                    log_likelihood: m * (y * eta - softplus(eta)),
                    score: m * (y - p),
                    weight: m * p * q,
                }
            }
            Link::Probit => {
                // log Φ(η) for the successes and log Φ(−η) = log(1 − Φ(η)) for the failures,
                // with their scores λ(η) and −λ(−η).
                let (one, log_one) = lower_tail(eta);
                let (zero, log_zero) = lower_tail(-eta);
                let mut log_likelihood = 0.0;
                if y > 0.0 {
                    log_likelihood += y * log_one;
                }
                if y < 1.0 {
                    log_likelihood += (1.0 - y) * log_zero;
                }
                let weight = if observed {
                    y * one * (one + eta) + (1.0 - y) * zero * (zero - eta)
                } else {
                    // φ² / (Φ(1 − Φ)) = λ(η)·λ(−η).
                    one * zero
                };
                RowTerms {
                    log_likelihood: m * log_likelihood,
                    score: m * (y * one - (1.0 - y) * zero),
                    weight: m * weight,
                }
            }
        }
    }
}

impl Likelihood for Binomial {
    fn mean(&self, eta: f64) -> f64 {
        self.link.mean(eta)
    }

    fn deviance(&self, row: usize, eta: f64) -> f64 {
        let y = self.y[row];
        let m = self.prior[row];
        let ll = self.at(row, eta, false).log_likelihood;
        // The saturated model's log-likelihood: y log y + (1 − y) log(1 − y), zero for 0 and 1.
        let saturated = if y > 0.0 && y < 1.0 {
            m * (y * log(y) + (1.0 - y) * log1p(-y))
        } else {
            0.0
        };
        2.0 * (saturated - ll)
    }

    /// R's start (`binomial()$initialize`): the mean (m·y + ½)/(m + 1), on the link's scale.
    fn start(&self, row: usize) -> f64 {
        let m = self.prior[row];
        let mean = (m * self.y[row] + 0.5) / (m + 1.0);
        self.link.link(mean)
    }
}

/// A model fitted by maximum likelihood, as the variance layer reads it (section 8.3, point 2):
/// each observation's score is its row of X times the derivative of its log-likelihood with
/// respect to its linear predictor, at the maximum, and the bread is the inverse of the
/// information, which the caller holds. The robust and clustered variances of every likelihood
/// model come from the one computation of `variance::variance` through this.
pub struct AtMaximum<'a> {
    pub model: &'a dyn IndexModel,
    /// The linear predictor of each row used, at the maximum.
    pub eta: &'a [f64],
    /// The inverse of the information at the maximum, in the units of the data.
    pub bread: &'a Matrix,
}

impl Estimator for AtMaximum<'_> {
    fn fit(&self, ctx: &mut CallContext<'_>, _design: &Design) -> Result<Fitted, Fail> {
        // The shared optimiser fits a likelihood model; this is its result, read afterwards.
        Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (a likelihood fit asked to refit through the variance layer)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ))
    }

    fn scores(&self, fitted: &Fitted, block: &Block, first_row: usize) -> Matrix {
        let scores: Vec<f64> = (0..block.x.rows())
            .map(|r| {
                let row = first_row + r;
                self.model.at(row, self.eta[row], false).score
            })
            .collect();
        Matrix::from_fn(block.x.rows(), fitted.kept.len(), |r, p| {
            block.x.get(r, fitted.kept[p]) * scores[r]
        })
    }

    fn bread(&self, _fitted: &Fitted) -> Matrix {
        self.bread.clone()
    }
}

/// How far the log-likelihood may fall from one iteration to the next, as a share of its size,
/// before the fall is more than rounding.
const FALL_TOLERANCE: f64 = 1e-9;
/// How large a column's score may be at the maximum, as a share of the sum of the sizes of its
/// terms: the iterations stop when the estimates move by less than a relative 1e-11.
const SCORE_TOLERANCE: f64 = 1e-6;
/// How far two mirrored entries of the variance may differ, as a share of the two standard
/// errors' product.
const SYMMETRY_TOLERANCE: f64 = 1e-12;

/// The self-checks of a fit by maximum likelihood (section 11.1 of the Language Reference,
/// "Self-checks"): a log-likelihood that never fell during the iterations, a score of zero at
/// the maximum in every column of `x`, and a variance of the estimates that is symmetric with no
/// negative entry on its diagonal. `Err` says which check failed; the caller reports it as an
/// internal error, never as a result.
///
/// The first value of the history is the log-likelihood of the start, which no coefficients
/// gave, so the first step may leave it in either direction; the check begins after it.
pub fn self_check(
    x: &Matrix,
    model: &dyn IndexModel,
    solution: &Solution,
    vcov: &Matrix,
) -> Result<(), String> {
    for (at, pair) in solution.history.windows(2).enumerate().skip(1) {
        let floor = pair[0] - FALL_TOLERANCE * (pair[0].abs() + 1.0);
        if pair[1].is_nan() || pair[1] < floor {
            return Err(format!("the log-likelihood fell at iteration {}", at + 1));
        }
    }
    let scores: Vec<f64> = solution
        .eta
        .iter()
        .enumerate()
        .map(|(row, &eta)| model.at(row, eta, false).score)
        .collect();
    for j in 0..x.cols() {
        let terms: Vec<f64> = x
            .column(j)
            .iter()
            .zip(&scores)
            .map(|(a, s)| a * s)
            .collect();
        let sizes: Vec<f64> = terms.iter().map(|t| t.abs()).collect();
        let (score, bound) = (sum(&terms).abs(), SCORE_TOLERANCE * sum(&sizes));
        if score.is_nan() || score > bound {
            return Err(format!(
                "the score of column {} is not zero at the maximum",
                j + 1
            ));
        }
    }
    let k = vcov.rows();
    for i in 0..k {
        let variance = vcov.get(i, i);
        if !variance.is_finite() || variance < 0.0 {
            return Err(format!(
                "the variance of estimate {} is negative or not a number",
                i + 1
            ));
        }
    }
    for i in 0..k {
        for j in 0..i {
            let size = vcov.get(i, i).sqrt() * vcov.get(j, j).sqrt();
            let apart = (vcov.get(i, j) - vcov.get(j, i)).abs();
            if apart.is_nan() || apart > SYMMETRY_TOLERANCE * size {
                return Err("an asymmetric variance matrix".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small logistic fit with its variance, for the self-checks.
    fn small_fit() -> (Matrix, Binomial, Solution, Matrix) {
        use zeff_kernels::irls::{Control, irls};
        use zeff_kernels::triangular::inverse_upper;
        let x = Matrix::from_columns(&[
            vec![1.0; 8],
            vec![-2.0, -1.5, -1.0, -0.5, 0.5, 1.0, 1.5, 2.0],
        ]);
        let model = Binomial::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0], Link::Logit);
        let start: Vec<f64> = (0..8).map(|row| model.start(row)).collect();
        let solution = irls(&x, &model, &start, &Control::default()).unwrap();
        assert!(solution.converged);
        let inverse = inverse_upper(&solution.r).unwrap();
        let vcov = Matrix::from_fn(2, 2, |i, j| {
            (0..2)
                .map(|l| inverse.get(i, l) * inverse.get(j, l))
                .sum::<f64>()
                * solution.scales[i]
                * solution.scales[j]
        });
        (x, model, solution, vcov)
    }

    #[test]
    fn a_sound_fit_passes_its_self_checks() {
        let (x, model, solution, vcov) = small_fit();
        assert_eq!(self_check(&x, &model, &solution, &vcov), Ok(()));
    }

    #[test]
    fn a_log_likelihood_that_fell_fails_the_self_check() {
        let (x, model, mut solution, vcov) = small_fit();
        let last = *solution.history.last().unwrap();
        solution.history.push(last - 0.5);
        let failed = self_check(&x, &model, &solution, &vcov).unwrap_err();
        assert!(
            failed.starts_with("the log-likelihood fell at iteration"),
            "{failed}"
        );
        // The first step leaves a start that no coefficients gave: a fall there is no failure.
        let (x, model, mut solution, vcov) = small_fit();
        solution.history[0] = solution.history[1] + 10.0;
        assert_eq!(self_check(&x, &model, &solution, &vcov), Ok(()));
    }

    #[test]
    fn a_score_that_is_not_zero_fails_the_self_check() {
        let (x, model, mut solution, vcov) = small_fit();
        for eta in &mut solution.eta {
            *eta += 0.25;
        }
        let failed = self_check(&x, &model, &solution, &vcov).unwrap_err();
        assert_eq!(failed, "the score of column 1 is not zero at the maximum");
    }

    #[test]
    fn a_variance_that_is_not_one_fails_the_self_check() {
        let (x, model, solution, vcov) = small_fit();
        let mut lopsided = vcov.clone();
        lopsided.set(0, 1, 2.0 * vcov.get(0, 1) + 1.0);
        assert_eq!(
            self_check(&x, &model, &solution, &lopsided).unwrap_err(),
            "an asymmetric variance matrix"
        );
        let mut negative = vcov.clone();
        negative.set(1, 1, -vcov.get(1, 1));
        assert_eq!(
            self_check(&x, &model, &solution, &negative).unwrap_err(),
            "the variance of estimate 2 is negative or not a number"
        );
        let mut missing = vcov;
        missing.set(0, 0, f64::NAN);
        assert_eq!(
            self_check(&x, &model, &solution, &missing).unwrap_err(),
            "the variance of estimate 1 is negative or not a number"
        );
    }

    #[test]
    fn the_two_tails_keep_their_digits() {
        // log Φ(−40) by the series, against R's `pnorm(-40, log.p = TRUE)`, −804.6084420137538.
        let (_, log_lower) = lower_tail(-40.0);
        assert!((log_lower - -804.608_442_013_753_8).abs() < 1e-13 * 805.0);
        // Either side of the switch to the series, against R's φ/Φ (from its log density and
        // log tail) and its log Φ at the same points: 37.026987685127729 and −689.0305855398637
        // above, 37.026987687125143 and −689.0305856139174 below.
        let (a, la) = lower_tail(-FAR + 1e-9);
        let (b, lb) = lower_tail(-FAR - 1e-9);
        assert!((a - 37.026_987_685_127_73).abs() < 1e-12 * a);
        assert!((b - 37.026_987_687_125_14).abs() < 1e-12 * b);
        assert!((la - -689.030_585_539_863_7).abs() < 1e-13 * 690.0);
        assert!((lb - -689.030_585_613_917_4).abs() < 1e-13 * 690.0);
        // Far in the upper tail, log Φ is −(1 − Φ), not zero.
        let (_, log_near_one) = lower_tail(8.0);
        assert!(log_near_one < 0.0 && log_near_one > -1e-14);
    }

    #[test]
    fn the_logit_terms_are_finite_far_out() {
        let family = Binomial::new(vec![1.0, 0.0], Link::Logit);
        for eta in [-800.0, -40.0, 0.0, 40.0, 800.0] {
            for row in 0..2 {
                let t = family.at(row, eta, false);
                assert!(t.log_likelihood.is_finite() && t.score.is_finite() && t.weight >= 0.0);
            }
        }
    }
}
