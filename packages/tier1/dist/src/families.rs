//! The twelve families' density, distribution function, quantile and draw (Engine and Project
//! Specification, section 13: tails computed directly; every draw by inverse CDF from the uniform
//! stream, section 8.3, point 3). This release's families: normal, log-normal, uniform and
//! exponential (step 7); the rest follow at steps 8 to 10.

use zeff_core::value::Distribution;
use zeff_numeric::maths;
use zeff_numeric::special::{self, Tails};

/// A family's parameter by name, which every constructor fills in.
fn param(params: &[(&'static str, f64)], name: &str) -> f64 {
    params
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(f64::NAN, |(_, v)| *v)
}

/// Whether the distribution takes whole numbers only (its draws are Ints).
pub fn is_discrete(d: &Distribution) -> bool {
    match d {
        Distribution::Family { name, .. } => {
            matches!(*name, "binomial" | "poisson" | "negbinomial")
        }
        Distribution::Truncated { inner, .. } => is_discrete(inner),
        Distribution::Mixture { parts } => parts.iter().all(|(d, _)| is_discrete(d)),
    }
}

/// The density at `x` (the probability mass for a discrete distribution).
pub fn pdf(d: &Distribution, x: f64) -> f64 {
    let Distribution::Family { name, params } = d else {
        return f64::NAN;
    };
    match *name {
        "normal" => {
            let (mean, sd) = (param(params, "mean"), param(params, "sd"));
            special::normal_density((x - mean) / sd) / sd
        }
        "lognormal" => {
            if x <= 0.0 {
                return 0.0;
            }
            let (meanlog, sdlog) = (param(params, "meanlog"), param(params, "sdlog"));
            special::normal_density((maths::log(x) - meanlog) / sdlog) / (x * sdlog)
        }
        "uniform" => {
            let (min, max) = (param(params, "min"), param(params, "max"));
            if x < min || x > max {
                0.0
            } else {
                1.0 / (max - min)
            }
        }
        "exponential" => {
            let rate = param(params, "rate");
            if x < 0.0 {
                0.0
            } else {
                rate * maths::exp(-rate * x)
            }
        }
        _ => f64::NAN,
    }
}

/// P(X ≤ x) and P(X > x), each computed on its own side.
pub fn cdf(d: &Distribution, x: f64) -> Tails {
    let Distribution::Family { name, params } = d else {
        return Tails {
            lower: f64::NAN,
            upper: f64::NAN,
        };
    };
    let exact = |lower: f64| Tails {
        lower,
        upper: 1.0 - lower,
    };
    match *name {
        "normal" => {
            let (mean, sd) = (param(params, "mean"), param(params, "sd"));
            special::normal_tails((x - mean) / sd)
        }
        "lognormal" => {
            if x <= 0.0 {
                return exact(0.0);
            }
            let (meanlog, sdlog) = (param(params, "meanlog"), param(params, "sdlog"));
            special::normal_tails((maths::log(x) - meanlog) / sdlog)
        }
        "uniform" => {
            let (min, max) = (param(params, "min"), param(params, "max"));
            if x <= min {
                exact(0.0)
            } else if x >= max {
                exact(1.0)
            } else {
                let lower = (x - min) / (max - min);
                Tails {
                    lower,
                    upper: (max - x) / (max - min),
                }
            }
        }
        "exponential" => {
            let rate = param(params, "rate");
            if x <= 0.0 {
                exact(0.0)
            } else {
                Tails {
                    lower: -maths::expm1(-rate * x),
                    upper: maths::exp(-rate * x),
                }
            }
        }
        _ => Tails {
            lower: f64::NAN,
            upper: f64::NAN,
        },
    }
}

/// The smallest x with P(X ≤ x) ≥ p, for p in [0, 1].
pub fn quantile(d: &Distribution, p: f64) -> f64 {
    let Distribution::Family { name, params } = d else {
        return f64::NAN;
    };
    match *name {
        "normal" => {
            let (mean, sd) = (param(params, "mean"), param(params, "sd"));
            mean + sd * special::normal_quantile(p, false)
        }
        "lognormal" => {
            let (meanlog, sdlog) = (param(params, "meanlog"), param(params, "sdlog"));
            maths::exp(meanlog + sdlog * special::normal_quantile(p, false))
        }
        "uniform" => {
            let (min, max) = (param(params, "min"), param(params, "max"));
            min + p * (max - min)
        }
        "exponential" => -maths::log1p(-p) / param(params, "rate"),
        _ => f64::NAN,
    }
}

/// One draw from a uniform number in [0, 1): the inverse CDF at the uniform shifted by half a grid
/// step, so that a draw never meets the quantile's endpoints.
pub fn draw(d: &Distribution, uniform: f64) -> f64 {
    quantile(d, uniform + 5.551_115_123_125_783e-17)
}
