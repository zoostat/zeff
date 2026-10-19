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
        "t" => {
            let df = param(params, "df");
            // (1 + t²/ν)^(-(ν+1)/2) / (√ν B(½, ν/2)), in logarithms.
            maths::exp(
                -special::log_beta(0.5, df / 2.0)
                    - 0.5 * maths::log(df)
                    - (df + 1.0) / 2.0 * maths::log1p(x * x / df),
            )
        }
        "chi2" => {
            let df = param(params, "df");
            if x < 0.0 {
                0.0
            } else {
                special::gamma_density(df / 2.0, x / 2.0) / 2.0
            }
        }
        "f" => {
            let (d1, d2) = (param(params, "df1"), param(params, "df2"));
            if x < 0.0 {
                return 0.0;
            }
            if x == 0.0 {
                return if d1 < 2.0 {
                    f64::INFINITY
                } else if d1 == 2.0 {
                    1.0
                } else {
                    0.0
                };
            }
            // The beta density of w = d1 x / (d1 x + d2) times dw/dx.
            let w = d1 * x / (d1 * x + d2);
            special::beta_density(d1 / 2.0, d2 / 2.0, w) * d1 * d2 / ((d1 * x + d2) * (d1 * x + d2))
        }
        "gamma" => {
            let (shape, rate) = (param(params, "shape"), param(params, "rate"));
            if x < 0.0 {
                0.0
            } else {
                rate * special::gamma_density(shape, rate * x)
            }
        }
        "beta" => {
            let (a, b) = (param(params, "a"), param(params, "b"));
            if !(0.0..=1.0).contains(&x) {
                0.0
            } else {
                special::beta_density(a, b, x)
            }
        }
        _ => f64::NAN,
    }
}

/// The t distribution's tails: half the incomplete beta function at ν / (ν + t²), on the side of
/// t; above 400,000 degrees of freedom the normal approximation R uses, so that the reference and
/// the engine agree where the continued fraction would lose digits.
fn t_tails(df: f64, t: f64) -> Tails {
    if df > 400_000.0 {
        let correction = 1.0 / (4.0 * df);
        // t (1 − c) / √(1 + 2c t²), written as (1 − c) sign(t) / √(1/t² + 2c) so that a huge t
        // does not overflow to a ratio of infinities.
        let z = if t == 0.0 {
            0.0
        } else {
            (1.0 - correction) * t.signum() / (1.0 / (t * t) + 2.0 * correction).sqrt()
        };
        return special::normal_tails(z);
    }
    if t.is_infinite() {
        return if t > 0.0 {
            Tails {
                lower: 1.0,
                upper: 0.0,
            }
        } else {
            Tails {
                lower: 0.0,
                upper: 1.0,
            }
        };
    }
    // Past |t| = 1e150 the ratio ν / (ν + t²) is below the smallest Float, and the tail is its
    // leading term in logarithms: ½ x^(ν/2) / ((ν/2) B(ν/2, ½)) with ln x = ln ν − 2 ln |t|.
    let half = if t.abs() > 1e150 {
        let a = df / 2.0;
        let ln_x = maths::log(df) - 2.0 * maths::log(t.abs());
        0.5 * maths::exp(a * ln_x - maths::log(a) - special::log_beta(a, 0.5))
    } else {
        let x = df / (df + t * t);
        0.5 * special::incomplete_beta(df / 2.0, 0.5, x).lower
    };
    if t >= 0.0 {
        Tails {
            lower: 1.0 - half,
            upper: half,
        }
    } else {
        Tails {
            lower: half,
            upper: 1.0 - half,
        }
    }
}

/// The t distribution's quantile: from the inverse incomplete beta on the smaller tail.
fn t_quantile(df: f64, p: f64) -> f64 {
    if df > 400_000.0 {
        // Invert the normal approximation of `t_tails` by a few Newton steps from the normal.
        let mut t = special::normal_quantile(p, false);
        for _ in 0..4 {
            let f = t_tails(df, t).lower - p;
            let density = pdf(
                &Distribution::Family {
                    name: "t",
                    params: vec![("df", df)],
                },
                t,
            );
            if density == 0.0 {
                break;
            }
            t -= f / density;
        }
        return t;
    }
    let (q, negative) = if p <= 0.5 {
        (p, true)
    } else {
        (1.0 - p, false)
    };
    let a = df / 2.0;
    // In the deep tail the ratio x = ν / (ν + t²) lies below the smallest Float, and the tail's
    // leading term inverts in logarithms: ln x = (ln 2q + ln a + ln B(a, ½)) / a,
    // |t| = √ν e^(−ln x / 2).
    let ln_x = (maths::log(2.0 * q) + maths::log(a) + special::log_beta(a, 0.5)) / a;
    if ln_x < -660.0 {
        let magnitude = df.sqrt() * maths::exp(-ln_x / 2.0);
        return if negative { -magnitude } else { magnitude };
    }
    // I_x(ν/2, ½) = 2q, with x = ν / (ν + t²); for 2q near one, solve for 1 − x instead.
    let magnitude = if 2.0 * q < 0.5 {
        let x = special::incomplete_beta_inverse(df / 2.0, 0.5, 2.0 * q, false);
        (df * (1.0 - x) / x).sqrt()
    } else {
        let y = special::incomplete_beta_inverse(0.5, df / 2.0, 1.0 - 2.0 * q, false);
        (df * y / (1.0 - y)).sqrt()
    };
    if negative { -magnitude } else { magnitude }
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
        "t" => t_tails(param(params, "df"), x),
        "chi2" => {
            if x <= 0.0 {
                exact(0.0)
            } else {
                special::incomplete_gamma(param(params, "df") / 2.0, x / 2.0)
            }
        }
        "f" => {
            let (d1, d2) = (param(params, "df1"), param(params, "df2"));
            if x <= 0.0 {
                exact(0.0)
            } else if x.is_infinite() {
                exact(1.0)
            } else {
                special::incomplete_beta(d1 / 2.0, d2 / 2.0, d1 * x / (d1 * x + d2))
            }
        }
        "gamma" => {
            let (shape, rate) = (param(params, "shape"), param(params, "rate"));
            if x <= 0.0 {
                exact(0.0)
            } else {
                special::incomplete_gamma(shape, rate * x)
            }
        }
        "beta" => {
            let (a, b) = (param(params, "a"), param(params, "b"));
            if x <= 0.0 {
                exact(0.0)
            } else if x >= 1.0 {
                exact(1.0)
            } else {
                special::incomplete_beta(a, b, x)
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
        "t" => t_quantile(param(params, "df"), p),
        "chi2" => 2.0 * special::incomplete_gamma_inverse(param(params, "df") / 2.0, p, false),
        "f" => {
            let (d1, d2) = (param(params, "df1"), param(params, "df2"));
            // x = (d2 / d1) · w / (1 − w) with I_w(d1/2, d2/2) = p; for p past one half, 1 − w
            // comes from the mirrored inverse so that it keeps its digits.
            if p <= 0.5 {
                let w = special::incomplete_beta_inverse(d1 / 2.0, d2 / 2.0, p, false);
                d2 / d1 * w / (1.0 - w)
            } else {
                let y = special::incomplete_beta_inverse(d2 / 2.0, d1 / 2.0, 1.0 - p, false);
                d2 / d1 * (1.0 - y) / y
            }
        }
        "gamma" => {
            let (shape, rate) = (param(params, "shape"), param(params, "rate"));
            special::incomplete_gamma_inverse(shape, p, false) / rate
        }
        "beta" => {
            special::incomplete_beta_inverse(param(params, "a"), param(params, "b"), p, false)
        }
        _ => f64::NAN,
    }
}

/// One draw from a uniform number in [0, 1): the inverse CDF at the uniform shifted by half a grid
/// step, so that a draw never meets the quantile's endpoints.
pub fn draw(d: &Distribution, uniform: f64) -> f64 {
    quantile(d, uniform + 5.551_115_123_125_783e-17)
}
