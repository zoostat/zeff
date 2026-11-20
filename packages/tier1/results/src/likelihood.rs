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

use zeff_kernels::irls::{IndexModel, RowTerms};
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

#[cfg(test)]
mod tests {
    use super::*;

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
