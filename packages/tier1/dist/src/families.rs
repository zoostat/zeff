//! The twelve families' density, distribution function, quantile and draw (Engine and Project
//! Specification, section 13: tails computed directly; every draw by inverse CDF from the uniform
//! stream, section 8.3, point 3). This release's families: normal, log-normal, uniform and
//! exponential (step 7); the rest follow at steps 8 to 10.

use zeff_core::value::Distribution;
use zeff_numeric::maths;
use zeff_numeric::random::Stream;
use zeff_numeric::special::{self, Tails};

const LN_SQRT_2PI: f64 = 0.918_938_533_204_672_8;

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
    let (name, params) = match d {
        Distribution::Family { name, params } => (name, params),
        Distribution::Truncated {
            inner,
            lower,
            upper,
        } => {
            if x < *lower || x > *upper {
                return 0.0;
            }
            return pdf(inner, x) / truncated_mass(inner, *lower, *upper);
        }
        Distribution::Mixture { parts } => {
            return parts.iter().map(|(d, p)| p * pdf(d, x)).sum();
        }
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
        "binomial" => {
            let (n, p) = (param(params, "n"), param(params, "p"));
            match whole(x) {
                Some(k) if (0.0..=n).contains(&k) => binomial_mass(k, n, p),
                _ => 0.0,
            }
        }
        "poisson" => {
            let lambda = param(params, "lambda");
            match whole(x) {
                Some(k) if k >= 0.0 => poisson_mass(k, lambda),
                _ => 0.0,
            }
        }
        "negbinomial" => {
            let (size, p) = negbinomial_size_p(params);
            match whole(x) {
                Some(k) if k >= 0.0 && p == 1.0 => f64::from(u8::from(k == 0.0)),
                // size / (size + k) times the binomial mass of `size` successes in k + size trials.
                Some(k) if k >= 0.0 => size / (size + k) * binomial_mass(size, k + size, p),
                _ => 0.0,
            }
        }
        _ => f64::NAN,
    }
}

/// A discrete distribution's argument as a whole number: a value within 1e-7 (relative) of one
/// counts as it, as R's `dbinom` and its kin treat floating-point noise; `None` otherwise.
fn whole(x: f64) -> Option<f64> {
    if x < 0.0 {
        return None;
    }
    let k = x.round();
    if (x - k).abs() <= 1e-7 * x.abs().max(1.0) {
        Some(k)
    } else {
        None
    }
}

/// The largest whole number not above `x`, with the same tolerance for noise just below one.
fn floor_whole(x: f64) -> f64 {
    if x < 0.0 {
        return -1.0;
    }
    (x + 1e-7 * x.abs().max(1.0)).floor()
}

/// The negative binomial's size and success probability, from either parameterisation.
fn negbinomial_size_p(params: &[(&'static str, f64)]) -> (f64, f64) {
    let size = param(params, "size");
    let p = param(params, "p");
    if p.is_nan() {
        let mu = param(params, "mu");
        (size, size / (size + mu))
    } else {
        (size, p)
    }
}

/// C(n, k) p^k (1 − p)^(n − k) for a real n and k by Loader's saddle-point form: the Stirling
/// remainders and the deviance terms keep every digit where the logarithm of a large binomial
/// coefficient would not.
fn binomial_mass(k: f64, n: f64, p: f64) -> f64 {
    if p == 0.0 {
        return f64::from(u8::from(k == 0.0));
    }
    if p == 1.0 {
        return f64::from(u8::from(k == n));
    }
    if k == 0.0 {
        return maths::exp(n * maths::log1p(-p));
    }
    if k == n {
        return maths::exp(n * maths::log(p));
    }
    let remainder = special::stirling_remainder(n)
        - special::stirling_remainder(k)
        - special::stirling_remainder(n - k);
    let deviance = special::deviance(k, n * p) + special::deviance(n - k, n * (1.0 - p));
    let log_scale = -0.5 * maths::log(k * (n - k) / n) - LN_SQRT_2PI;
    maths::exp(remainder - deviance + log_scale)
}

/// e^−λ λ^k / k! by the same saddle-point form.
fn poisson_mass(k: f64, lambda: f64) -> f64 {
    if k == 0.0 {
        return maths::exp(-lambda);
    }
    maths::exp(
        -special::stirling_remainder(k)
            - special::deviance(k, lambda)
            - 0.5 * maths::log(k)
            - LN_SQRT_2PI,
    )
}

/// The smallest whole number k with F(k) ≥ p, found from a start near the mean by widening a
/// bracket and then halving it, so that a far quantile costs a few dozen evaluations of F.
fn discrete_quantile(d: &Distribution, p: f64, start: f64, upper_bound: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return upper_bound;
    }
    let f = |k: f64| cdf(d, k).lower;
    // A start beyond the largest Float (a mean of Inf) is searched from the largest Float.
    let mut k = if start.is_nan() {
        0.0
    } else {
        start.round().clamp(0.0, upper_bound.min(f64::MAX))
    };
    // The bracket widens by doubling from a step that is one at small start and a fraction of the
    // start at large, so that a quantile near 1e300 is bracketed in about forty evaluations of F
    // rather than a thousand.
    let first_step = (k * 2.0f64.powi(-40)).max(1.0);
    // Bracket: lo has F(lo) < p (or lo = -1), hi has F(hi) >= p.
    let (mut lo, mut hi);
    if f(k) >= p {
        hi = k;
        let mut step = first_step;
        loop {
            let next = hi - step;
            if next < 0.0 {
                lo = -1.0;
                break;
            }
            if f(next) < p {
                lo = next;
                break;
            }
            hi = next;
            step *= 2.0;
        }
    } else {
        lo = k;
        let mut step = first_step;
        loop {
            let next = lo + step;
            if next >= upper_bound {
                hi = upper_bound;
                break;
            }
            if f(next) >= p {
                hi = next;
                break;
            }
            lo = next;
            step *= 2.0;
        }
    }
    if hi.is_infinite() {
        // No whole number a Float can hold reaches p: the quantile is beyond the Floats.
        let reach = f(f64::MAX);
        if reach.is_nan() || reach < p {
            return hi;
        }
        hi = f64::MAX;
    }
    while hi - lo > 1.0 {
        k = (lo / 2.0 + hi / 2.0).floor();
        // Above 2^53 neighbouring Floats are more than 1 apart and the middle is one of them.
        if k <= lo || k >= hi {
            break;
        }
        if f(k) >= p {
            hi = k;
        } else {
            lo = k;
        }
    }
    hi
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
    } else if t * t < df {
        // Near the centre the ratio ν / (ν + t²) rounds towards one and loses the digits of
        // F(t) − ½, so the complement y = t² / (ν + t²) serves: I_x(ν/2, ½) = 1 − I_y(½, ν/2).
        let y = t * t / (df + t * t);
        0.5 * special::incomplete_beta(0.5, df / 2.0, y).upper
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

/// P(X < x): the lower tail just below x, which for a discrete distribution leaves out the mass at
/// x itself.
fn below(d: &Distribution, x: f64) -> f64 {
    let at = cdf(d, x);
    if is_discrete(d) {
        (at.lower - pdf(d, x)).max(0.0)
    } else {
        at.lower
    }
}

/// The probability a distribution gives to [lower, upper].
pub fn truncated_mass(inner: &Distribution, lower: f64, upper: f64) -> f64 {
    let top = if upper.is_infinite() {
        1.0
    } else {
        cdf(inner, upper).lower
    };
    let bottom = if lower.is_infinite() {
        0.0
    } else {
        below(inner, lower)
    };
    top - bottom
}

/// P(X ≤ x) and P(X > x), each computed on its own side.
pub fn cdf(d: &Distribution, x: f64) -> Tails {
    let (name, params) = match d {
        Distribution::Family { name, params } => (name, params),
        Distribution::Truncated {
            inner,
            lower,
            upper,
        } => {
            if x < *lower {
                return Tails {
                    lower: 0.0,
                    upper: 1.0,
                };
            }
            if x >= *upper {
                return Tails {
                    lower: 1.0,
                    upper: 0.0,
                };
            }
            let mass = truncated_mass(inner, *lower, *upper);
            let at = cdf(inner, x);
            // Each tail from the inner tails on its own side: (F(x) − F(lower⁻)) and
            // (S(x) − S(upper)), divided by the interval's mass.
            let bottom = if lower.is_infinite() {
                0.0
            } else {
                below(inner, *lower)
            };
            let top_upper = if upper.is_infinite() {
                0.0
            } else {
                cdf(inner, *upper).upper
            };
            return Tails {
                lower: ((at.lower - bottom) / mass).clamp(0.0, 1.0),
                upper: ((at.upper - top_upper) / mass).clamp(0.0, 1.0),
            };
        }
        Distribution::Mixture { parts } => {
            let mut lower = 0.0;
            let mut upper = 0.0;
            for (d, p) in parts {
                let tails = cdf(d, x);
                lower += p * tails.lower;
                upper += p * tails.upper;
            }
            return Tails { lower, upper };
        }
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
        "binomial" => {
            let (n, p) = (param(params, "n"), param(params, "p"));
            let k = floor_whole(x);
            if k < 0.0 {
                exact(0.0)
            } else if k >= n {
                exact(1.0)
            } else {
                // P(X ≤ k) = I_{1−p}(n − k, k + 1), both tails on their own side.
                special::incomplete_beta(n - k, k + 1.0, 1.0 - p)
            }
        }
        "poisson" => {
            let lambda = param(params, "lambda");
            let k = floor_whole(x);
            if k < 0.0 {
                exact(0.0)
            } else {
                // P(X ≤ k) = Q(k + 1, λ).
                let tails = special::incomplete_gamma(k + 1.0, lambda);
                Tails {
                    lower: tails.upper,
                    upper: tails.lower,
                }
            }
        }
        "negbinomial" => {
            let (size, p) = negbinomial_size_p(params);
            let k = floor_whole(x);
            if k < 0.0 {
                exact(0.0)
            } else if p == 1.0 {
                exact(1.0)
            } else {
                // P(X ≤ k) = I_p(size, k + 1).
                special::incomplete_beta(size, k + 1.0, p)
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
    let (name, params) = match d {
        Distribution::Family { name, params } => (name, params),
        Distribution::Truncated {
            inner,
            lower,
            upper,
        } => {
            if p <= 0.0 {
                return if is_discrete(inner) {
                    quantile(inner, below(inner, *lower) + f64::EPSILON * 0.0).max(*lower)
                } else {
                    *lower
                };
            }
            if p >= 1.0 {
                return *upper;
            }
            // The inner quantile at F(lower⁻) + p · mass, held inside the interval.
            let bottom = if lower.is_infinite() {
                0.0
            } else {
                below(inner, *lower)
            };
            let target = bottom + p * truncated_mass(inner, *lower, *upper);
            return quantile(inner, target.min(1.0)).clamp(*lower, *upper);
        }
        Distribution::Mixture { parts } => return mixture_quantile(parts, p),
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
        "binomial" => {
            let (n, q) = (param(params, "n"), param(params, "p"));
            let mean = n * q;
            let sd = (n * q * (1.0 - q)).sqrt();
            let start = mean + sd * special::normal_quantile(p.clamp(1e-300, 1.0 - 1e-16), false);
            discrete_quantile(d, p, start, n)
        }
        "poisson" => {
            let lambda = param(params, "lambda");
            if p >= 1.0 {
                return f64::INFINITY;
            }
            let start = lambda + lambda.sqrt() * special::normal_quantile(p.max(1e-300), false);
            discrete_quantile(d, p, start, f64::INFINITY)
        }
        "negbinomial" => {
            let (size, q) = negbinomial_size_p(params);
            if p >= 1.0 {
                return if q == 1.0 { 0.0 } else { f64::INFINITY };
            }
            let mean = size * (1.0 - q) / q;
            let sd = (mean / q).sqrt();
            let start = mean + sd * special::normal_quantile(p.max(1e-300), false);
            discrete_quantile(d, p, start, f64::INFINITY)
        }
        _ => f64::NAN,
    }
}

/// The smallest x with the mixed distribution function at or above p: bracketed by the
/// components' own quantiles, then halved until the bracket closes, which lands on a jump when the
/// components are discrete.
fn mixture_quantile(parts: &[(Distribution, f64)], p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        let ends = parts.iter().map(|(d, _)| quantile(d, p));
        return if p <= 0.0 {
            ends.fold(f64::INFINITY, f64::min)
        } else {
            ends.fold(f64::NEG_INFINITY, f64::max)
        };
    }
    let mixed = Distribution::Mixture {
        parts: parts.to_vec(),
    };
    let f = |x: f64| cdf(&mixed, x).lower;
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (d, _) in parts {
        let q = quantile(d, p);
        lo = lo.min(q);
        hi = hi.max(q);
    }
    if lo == hi {
        return lo;
    }
    // Widen the bracket where a component's quantile does not bound the mixture's.
    while lo.is_finite() && f(lo) >= p {
        lo -= (hi - lo).max(1.0);
    }
    while hi.is_finite() && f(hi) < p {
        hi += (hi - lo).max(1.0);
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if f(mid) >= p {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    if all_discrete(parts) { hi.ceil() } else { hi }
}

fn all_discrete(parts: &[(Distribution, f64)]) -> bool {
    parts.iter().all(|(d, _)| is_discrete(d))
}

/// One draw from the stream: the inverse CDF at a uniform shifted by half a grid step, so that a
/// draw never meets the quantile's endpoints; a mixture spends one uniform choosing the component
/// and one drawing from it (Engine and Project Specification, section 8.3, point 3).
pub fn draw(d: &Distribution, stream: &mut Stream) -> f64 {
    const HALF_STEP: f64 = 5.551_115_123_125_783e-17;
    match d {
        Distribution::Mixture { parts } => {
            let choice = stream.uniform();
            let mut cumulative = 0.0;
            for (component, p) in parts {
                cumulative += p;
                if choice < cumulative {
                    return draw(component, stream);
                }
            }
            // Rounding at the top: the last component.
            draw(&parts[parts.len() - 1].0, stream)
        }
        _ => quantile(d, stream.uniform() + HALF_STEP),
    }
}
