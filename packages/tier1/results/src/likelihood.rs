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
use zeff_kernels::qr::TallSkinnyQr;
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

    /// The slope of the probability in the linear predictor, dp/dη: p(1 − p) under the logit,
    /// the normal density under the probit (`margins`; release 0.9.0, step 12a).
    pub fn mean_slope(self, eta: f64) -> f64 {
        match self {
            Link::Logit => {
                let (p, q) = logistic(eta);
                p * q
            }
            Link::Probit => normal_density(eta),
        }
    }

    /// The second derivative of the probability in the linear predictor, d²p/dη²:
    /// p(1 − p)(1 − 2p) under the logit, −η φ(η) under the probit.
    pub fn mean_curve(self, eta: f64) -> f64 {
        match self {
            Link::Logit => {
                let (p, q) = logistic(eta);
                p * q * (q - p)
            }
            Link::Probit => -eta * normal_density(eta),
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

    /// The model with a weight on each row's log-likelihood: the number of observations a row
    /// stands for (frequency weights), or its sampling weight.
    pub fn weighted(y: Vec<f64>, prior: Vec<f64>, link: Link) -> Binomial {
        assert_eq!(y.len(), prior.len(), "one weight per row");
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
                // y − p, taken as y q − (1 − y) p: for an outcome of 1 it is q itself, which
                // keeps its digits where 1 − p has lost them (a probability near 1).
                RowTerms {
                    log_likelihood: m * (y * eta - softplus(eta)),
                    score: m * (y * q - (1.0 - y) * p),
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

/// The kinds of residual of a model fitted by maximum likelihood (`residuals(type:)`; McCullagh
/// and Nelder 1989, section 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidualKind {
    /// y − μ, on the scale of the outcome.
    Response,
    /// (y − μ) √w / √V(μ), whose squares sum to Pearson's chi-squared.
    Pearson,
    /// The signed square root of the row's contribution to the deviance.
    Deviance,
    /// (y − μ) / (dμ/dη), the residual of the last weighted least-squares step.
    Working,
}

impl ResidualKind {
    pub fn named(word: &str) -> Option<ResidualKind> {
        match word {
            "response" => Some(ResidualKind::Response),
            "pearson" => Some(ResidualKind::Pearson),
            "deviance" => Some(ResidualKind::Deviance),
            "working" => Some(ResidualKind::Working),
            _ => None,
        }
    }
}

impl Link {
    /// The link of a name, as a fit records it.
    pub fn named(word: &str) -> Option<Link> {
        match word {
            "logit" => Some(Link::Logit),
            "probit" => Some(Link::Probit),
            _ => None,
        }
    }
}

impl Binomial {
    /// The fitted probability at `eta` and its complement, each with its relative accuracy far
    /// into its tail.
    fn both(&self, eta: f64) -> (f64, f64) {
        match self.link {
            Link::Logit => logistic(eta),
            Link::Probit => {
                let tails = normal_tails(eta);
                (tails.lower, tails.upper)
            }
        }
    }

    /// The derivative of the mean in the linear predictor, dμ/dη, at `eta`.
    fn slope(&self, eta: f64) -> f64 {
        match self.link {
            Link::Logit => {
                let (p, q) = logistic(eta);
                p * q
            }
            Link::Probit => normal_density(eta),
        }
    }

    /// The residual of `row` at its linear predictor `eta`, of the kind asked for. The outcome
    /// less the mean is taken from the complement where the outcome is 1, so that a probability
    /// near 1 leaves its residual its digits.
    pub fn residual(&self, row: usize, eta: f64, kind: ResidualKind) -> f64 {
        let y = self.y[row];
        let m = self.prior[row];
        let (p, q) = self.both(eta);
        // y − μ = y q − (1 − y) p.
        let response = y * q - (1.0 - y) * p;
        match kind {
            ResidualKind::Response => response,
            ResidualKind::Pearson => response * m.sqrt() / (p * q).sqrt(),
            ResidualKind::Working => response / self.slope(eta),
            ResidualKind::Deviance => {
                let d = self.deviance(row, eta).max(0.0).sqrt();
                if response < 0.0 { -d } else { d }
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
    /// The columns the fit kept, one row per row used, unweighted: the model carries the
    /// weights in each row's score, where a least-squares design carries them in its rows.
    pub x: &'a Matrix,
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
            self.x.get(first_row + r, p) * scores[r]
        })
    }

    fn bread(&self, _fitted: &Fitted) -> Matrix {
        self.bread.clone()
    }
}

/// How far the log-likelihood may fall from one iteration to the next, as a share of its size,
/// before the fall is more than rounding.
const FALL_TOLERANCE: f64 = 1e-9;
/// How large the Newton decrement may be at the maximum, as a share of the size of the
/// log-likelihood. The decrement, sᵀX(XᵀWX)⁻¹Xᵀs, is twice the gain in log-likelihood that one
/// more step would bring, to second order: the score measured by what the information can
/// resolve. The iterations stop when a step gains no more than 1e-13 of the log-likelihood, so
/// a sound fit is some four orders inside.
const DECREMENT_TOLERANCE: f64 = 1e-9;
/// How far two mirrored entries of the variance may differ, as a share of the two standard
/// errors' product.
const SYMMETRY_TOLERANCE: f64 = 1e-12;

/// The Newton decrement of a model at the linear predictor `eta`, sᵀX(XᵀWX)⁻¹Xᵀs, with s the
/// rows' scores and W their information: the squared length of W^(−½)s along the columns of
/// W^(½)X, read from one QR factorisation of the weighted columns beside it, so that nothing
/// is inverted and a nearly singular design costs it no digits. `scales` are the columns'
/// scales, as the fit used them. Beside it, the sum of the sizes of the scores of the rows
/// that carry no information, which no step can reduce.
fn newton_decrement(x: &Matrix, model: &dyn IndexModel, eta: &[f64], scales: &[f64]) -> (f64, f64) {
    let k = x.cols();
    let mut qr = TallSkinnyQr::new(k + 1);
    let mut row = vec![0.0; k + 1];
    let mut unweighted = Vec::new();
    for (i, &e) in eta.iter().enumerate() {
        let terms = model.at(i, e, false);
        if terms.weight.is_nan() || terms.weight <= 0.0 {
            unweighted.push(terms.score.abs());
            continue;
        }
        let root = terms.weight.sqrt();
        for (j, slot) in row.iter_mut().enumerate().take(k) {
            *slot = root * x.get(i, j) * scales[j];
        }
        row[k] = terms.score / root;
        qr.push_row(&row);
    }
    let r = qr.finish();
    let parts: Vec<f64> = (0..k).map(|j| r.get(j, k) * r.get(j, k)).collect();
    (sum(&parts), sum(&unweighted))
}

/// The self-checks of a fit by maximum likelihood (section 11.1 of the Language Reference,
/// "Self-checks"): a log-likelihood that never fell during the iterations, a score of zero at
/// the maximum, and a variance of the estimates that is symmetric with no negative entry on its
/// diagonal. `Err` says which check failed; the caller reports it as an internal error, never
/// as a result.
///
/// The score is judged by the Newton decrement, the gain that one more step would bring: a fit
/// has converged when the likelihood can rise no further, and where the likelihood is nearly
/// flat in a direction (a regressor that all but separates the outcomes, a nearly singular
/// design) the score along it is small by that measure though not by the size of its terms
/// (the formula fuzz target found such a fit refused, release 0.9.0, step 9).
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
    let (decrement, unweighted) = newton_decrement(x, model, &solution.eta, &solution.scales);
    let bound = DECREMENT_TOLERANCE * (solution.log_likelihood.abs() + 0.1);
    if decrement.is_nan() || decrement > bound || unweighted.is_nan() || unweighted > 1e-6 {
        return Err(format!(
            "the score is not zero at the maximum: one more step would raise the log-likelihood by {:.1e}",
            0.5 * decrement
        ));
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
        assert!(
            failed.starts_with("the score is not zero at the maximum: one more step would raise the log-likelihood by "),
            "{failed}"
        );
        // A linear predictor out by a part in a million is within what the likelihood can
        // resolve: one more step would gain some 1e-12, far inside the tolerance.
        let (x, model, solution, vcov) = small_fit();
        let mut nudged = solution.clone();
        for eta in &mut nudged.eta {
            *eta += 1e-6;
        }
        assert_eq!(self_check(&x, &model, &nudged, &vcov), Ok(()));
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
