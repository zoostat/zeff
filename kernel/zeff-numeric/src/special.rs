//! The special functions behind the distributions (Engine and Project Specification, sections 8.3
//! point 3 and 13; release 0.3.0, step 2): the gamma and beta functions, the error functions and
//! the normal tail, the regularised incomplete gamma and beta functions with their inverses. One
//! pure-Rust implementation on every target: `lgamma`, `tgamma`, `erf` and `erfc` come from the
//! `libm` crate like the elementary functions; the rest is written here with fixed algorithms and
//! a fixed order of operations. Tails are computed directly, never as one minus the other side
//! (section 13), so a probability of 1e-300 carries its relative accuracy.

use crate::maths;

const LN_SQRT_2PI: f64 = 0.918_938_533_204_672_8; // ln(sqrt(2 pi))
const SQRT_2: f64 = std::f64::consts::SQRT_2;
const EPSILON: f64 = 1e-16;
const TINY: f64 = 1e-300;
const MAX_ITERATIONS: usize = 2_000_000;

/// The two tails of a cumulative distribution, each computed directly where it is small.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tails {
    pub lower: f64,
    pub upper: f64,
}

impl Tails {
    fn direct_lower(lower: f64) -> Tails {
        Tails {
            lower,
            upper: 1.0 - lower,
        }
    }

    fn direct_upper(upper: f64) -> Tails {
        Tails {
            lower: 1.0 - upper,
            upper,
        }
    }

    fn nan() -> Tails {
        Tails {
            lower: f64::NAN,
            upper: f64::NAN,
        }
    }

    /// The tail asked for.
    pub fn tail(self, upper: bool) -> f64 {
        if upper { self.upper } else { self.lower }
    }
}

// ----- gamma and beta -----

/// ln |Γ(x)|; `+Inf` at zero and the negative integers.
pub fn log_gamma(x: f64) -> f64 {
    libm::lgamma(x)
}

/// Γ(x); not-a-number at zero and the negative integers, `Inf` past 171.6.
pub fn gamma(x: f64) -> f64 {
    if x == 0.0 || (x < 0.0 && x == x.floor()) {
        return f64::NAN;
    }
    if x > 0.0 && x <= 171.0 && x == x.floor() {
        // Γ(n) = (n - 1)! exactly, where libm's value may be a unit off.
        return FACTORIALS[x as usize - 1];
    }
    libm::tgamma(x)
}

/// n! as the double nearest the exact value, for 0 ≤ n ≤ 170; `Inf` above; not-a-number below.
pub fn factorial(n: i64) -> f64 {
    if n < 0 {
        f64::NAN
    } else if n > 170 {
        f64::INFINITY
    } else {
        FACTORIALS[n as usize]
    }
}

/// The number of ways to choose `k` of `n`: exact while the integers fit, then one rounding per
/// step; 0 outside 0 ≤ k ≤ n; not-a-number for a negative `n`.
pub fn choose(n: i64, k: i64) -> f64 {
    if n < 0 {
        return f64::NAN;
    }
    if k < 0 || k > n {
        return 0.0;
    }
    let k = k.min(n - k);
    if k == 0 {
        return 1.0;
    }
    // C(n, k) >= C(1200, 600) > 1e360 once k > 600 (then n >= 1200), past the largest double.
    if k > 600 {
        return f64::INFINITY;
    }
    let (n, k) = (n as u128, k as u128);
    let mut exact: u128 = 1;
    let mut i: u128 = 1;
    while i <= k {
        match exact.checked_mul(n - k + i) {
            Some(product) => exact = product / i, // C(n - k + i, i) is an integer
            None => break,
        }
        i += 1;
    }
    let mut value = exact as f64;
    while i <= k {
        // The ratio first: the running value never exceeds the result, so it cannot overflow
        // before the last step.
        value *= (n - k + i) as f64 / i as f64;
        i += 1;
    }
    value
}

/// ln B(a, b) for positive `a` and `b`, through the Stirling form so that large arguments lose no
/// digits to cancellation.
pub fn log_beta(a: f64, b: f64) -> f64 {
    if !(a > 0.0 && b > 0.0) {
        return f64::NAN;
    }
    if a.is_infinite() || b.is_infinite() {
        return f64::NEG_INFINITY;
    }
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    if large < 10.0 {
        return log_gamma(a) + log_gamma(b) - log_gamma(a + b);
    }
    // lgamma(small) + lgamma(large) - lgamma(small + large), the last two by their Stirling forms:
    // (large - 0.5) ln(large / (large + small)) - small ln(large + small) + small + S(large) - S(small + large)
    let sum = small + large;
    let difference = -(large - 0.5) * maths::log1p(small / large) - small * maths::log(sum)
        + small
        + stirling_remainder(large)
        - stirling_remainder(sum);
    log_gamma(small) + difference
}

/// B(a, b).
pub fn beta(a: f64, b: f64) -> f64 {
    maths::exp(log_beta(a, b))
}

/// lgamma(x) - [(x - 0.5) ln x - x + ln sqrt(2 pi)]: the Stirling series for x ≥ 10, the
/// difference itself below.
pub fn stirling_remainder(x: f64) -> f64 {
    if x < 10.0 {
        return log_gamma(x) - ((x - 0.5) * maths::log(x) - x + LN_SQRT_2PI);
    }
    // B_2k / (2k (2k - 1) x^(2k - 1)), k = 1..9; the tenth term is below 1e-18 at x = 10.
    const COEFFICIENTS: [f64; 9] = [
        1.0 / 12.0,
        -1.0 / 360.0,
        1.0 / 1260.0,
        -1.0 / 1680.0,
        1.0 / 1188.0,
        -691.0 / 360_360.0,
        1.0 / 156.0,
        -3617.0 / 122_400.0,
        43_867.0 / 244_188.0,
    ];
    let inverse_square = 1.0 / (x * x);
    let mut power = 1.0 / x;
    let mut total = 0.0;
    for c in COEFFICIENTS {
        total += c * power;
        power *= inverse_square;
    }
    total
}

/// x ln(x / m) + m − x, the deviance term of Loader's binomial and Poisson probabilities, by its
/// series when x is near m (where the plain form cancels) and plainly otherwise.
pub fn deviance(x: f64, m: f64) -> f64 {
    if (x - m).abs() < 0.1 * (x + m) {
        let v = (x - m) / (x + m);
        let mut total = (x - m) * v;
        let mut term = 2.0 * x * v;
        let v2 = v * v;
        let mut j = 1.0;
        loop {
            term *= v2;
            let next = term / (2.0 * j + 1.0);
            total += next;
            if next.abs() <= total.abs() * 1e-17 {
                return total;
            }
            j += 1.0;
        }
    }
    x * maths::log(x / m) + m - x
}

/// log1p(d) - d, accurate when `d` is small (the series) and plain otherwise.
fn log1p_minus(d: f64) -> f64 {
    if d.abs() < 0.1 {
        // -d^2/2 + d^3/3 - d^4/4 + ...
        let mut power = d * d;
        let mut sign = -1.0;
        let mut k = 2.0;
        let mut total = -power / 2.0;
        loop {
            power *= d;
            sign = -sign;
            k += 1.0;
            let next = sign * power / k;
            total += next;
            if next.abs() <= total.abs() * 1e-17 {
                return total;
            }
        }
    } else {
        maths::log1p(d) - d
    }
}

/// a (ln(x / a) - (x - a) / a): the exponent's data-dependent part, through the series near the
/// mean and the plain logarithm of the ratio away from it, where log1p of the deviation would
/// lose the digits of a small ratio.
fn scaled_log_ratio(a: f64, x: f64) -> f64 {
    let d = (x - a) / a;
    if d.abs() < 0.1 {
        a * log1p_minus(d)
    } else {
        a * (maths::log(x / a) - d)
    }
}

// ----- the error functions and the normal distribution -----

pub fn erf(x: f64) -> f64 {
    libm::erf(x)
}

pub fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

/// Φ(x) and 1 - Φ(x), each from `erfc` so that both tails keep their relative accuracy.
pub fn normal_tails(x: f64) -> Tails {
    if x.is_nan() {
        return Tails::nan();
    }
    let scaled = x / SQRT_2;
    Tails {
        lower: 0.5 * erfc(-scaled),
        upper: 0.5 * erfc(scaled),
    }
}

/// The standard normal density.
pub fn normal_density(x: f64) -> f64 {
    maths::exp(-0.5 * x * x) / (SQRT_2 * 1.772_453_850_905_516) // sqrt(2 pi)
}

/// The standard normal quantile: the `x` with Φ(x) = p (or 1 - Φ(x) = p for `upper`), to full
/// precision by Halley steps on `erfc` from Abramowitz and Stegun's 26.2.23 start.
pub fn normal_quantile(p: f64, upper: bool) -> f64 {
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return if upper {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        };
    }
    if p == 1.0 {
        return if upper {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    if p == 0.5 {
        return 0.0;
    }
    // Solve for the lower tail probability q < 0.5, then reflect.
    let (q, reflect) = if p < 0.5 { (p, false) } else { (1.0 - p, true) };
    let t = (-2.0 * maths::log(q)).sqrt();
    let mut x = -(t
        - (2.515517 + 0.802853 * t + 0.010328 * t * t)
            / (1.0 + 1.432788 * t + 0.189269 * t * t + 0.001308 * t * t * t));
    for _ in 0..6 {
        let f = normal_tails(x).lower - q;
        let density = normal_density(x);
        if density == 0.0 || f == 0.0 {
            break;
        }
        let ratio = f / density;
        // Halley: f''/f' = -x for the normal.
        let step = ratio / (1.0 - 0.5 * x * ratio);
        x -= step;
        if step.abs() <= 1e-16 * x.abs().max(1.0) {
            break;
        }
    }
    let lower_quantile = if reflect { -x } else { x };
    if upper {
        -lower_quantile
    } else {
        lower_quantile
    }
}

// ----- the regularised incomplete gamma function -----

/// exp(a ln x - x - lgamma(a)), through log1p and the Stirling remainder for a ≥ 10 so that a
/// large `a` loses no digits to cancellation.
fn gamma_prefix(a: f64, x: f64) -> f64 {
    let exponent = if a < 10.0 {
        a * maths::log(x) - x - log_gamma(a)
    } else {
        scaled_log_ratio(a, x) + 0.5 * maths::log(a) - LN_SQRT_2PI - stirling_remainder(a)
    };
    maths::exp(exponent)
}

/// P(a, x) and Q(a, x), the regularised lower and upper incomplete gamma functions, for a > 0
/// and x ≥ 0: the series below a + 1, the continued fraction above, the other side as the
/// complement.
pub fn incomplete_gamma(a: f64, x: f64) -> Tails {
    if a.is_nan() || a <= 0.0 || x.is_nan() || x < 0.0 {
        return Tails::nan();
    }
    if x == 0.0 {
        return Tails {
            lower: 0.0,
            upper: 1.0,
        };
    }
    if x.is_infinite() {
        return Tails {
            lower: 1.0,
            upper: 0.0,
        };
    }
    let prefix = gamma_prefix(a, x);
    if x < a + 1.0 {
        // P = prefix * sum_{n>=0} x^n / (a (a+1) ... (a+n))
        let mut term = 1.0 / a;
        let mut total = term;
        let mut n = 0.0;
        for _ in 0..MAX_ITERATIONS {
            n += 1.0;
            term *= x / (a + n);
            total += term;
            if term.abs() < total.abs() * EPSILON {
                break;
            }
        }
        Tails::direct_lower(prefix * total)
    } else {
        // Q = prefix * continued fraction (modified Lentz).
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / TINY;
        let mut d = 1.0 / b;
        let mut h = d;
        let mut i = 0.0;
        for _ in 0..MAX_ITERATIONS {
            i += 1.0;
            let an = -i * (i - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < TINY {
                d = TINY;
            }
            c = b + an / c;
            if c.abs() < TINY {
                c = TINY;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if (delta - 1.0).abs() < EPSILON {
                break;
            }
        }
        Tails::direct_upper(prefix * h)
    }
}

/// The density of the gamma distribution with shape `a` and unit rate at `x`: P'(a, x).
pub fn gamma_density(a: f64, x: f64) -> f64 {
    if x == 0.0 {
        return if a < 1.0 {
            f64::INFINITY
        } else if a == 1.0 {
            1.0
        } else {
            0.0
        };
    }
    gamma_prefix(a, x) / x
}

/// The `x` with P(a, x) = p (or Q(a, x) = p for `upper`), by safeguarded Newton steps from the
/// Wilson–Hilferty start on whichever tail is the smaller.
pub fn incomplete_gamma_inverse(a: f64, p: f64, upper: bool) -> f64 {
    if a.is_nan() || a <= 0.0 || !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return if upper { f64::INFINITY } else { 0.0 };
    }
    if p == 1.0 {
        return if upper { 0.0 } else { f64::INFINITY };
    }
    // Work on the smaller tail.
    let (target, target_upper) = if p <= 0.5 {
        (p, upper)
    } else {
        (1.0 - p, !upper)
    };
    // A lower tail whose leading term (p a Γ(a))^(1/a) lies below the smallest Float: the
    // quantile is zero to every digit a double has.
    if !target_upper && (maths::log(target) + maths::log(a) + log_gamma(a)) / a < -745.0 {
        return 0.0;
    }
    let z = normal_quantile(target, target_upper);
    let mut x = {
        let wh = 1.0 - 1.0 / (9.0 * a) + z / (3.0 * a.sqrt());
        let guess = a * wh * wh * wh;
        if guess > 0.0 && guess.is_finite() {
            guess
        } else if target_upper {
            // A deep upper tail: start past the mean.
            a + 1.0
        } else {
            // A deep lower tail: (p a Γ(a))^(1/a) from the leading term of the series.
            maths::exp((maths::log(target) + maths::log(a) + log_gamma(a)) / a)
        }
    };
    newton_with_bracket(
        x,
        0.0,
        f64::INFINITY,
        |x| incomplete_gamma(a, x).tail(target_upper) - target,
        |x| {
            let density = gamma_density(a, x);
            if target_upper { -density } else { density }
        },
        &mut x,
    );
    x
}

// ----- the regularised incomplete beta function -----

/// exp(a ln x + b ln(1 - x) - ln B(a, b)) in the form that keeps its digits for large `a` and
/// `b`: with s = a + b and one rounded deviation d = x s - a, the two data terms are
/// a log1p(d / a) + b log1p(-d / b), whose first-order errors in d cancel; the rest is
/// ln sqrt(ab / s) - ln sqrt(2 pi) - ΔS with the Stirling remainders. Far from the mean, where
/// log1p would meet an argument at -1, the plain logarithm of the ratio serves: the result is then
/// deep in a tail, and the exponent's own size, not the digits of d, bounds the error.
fn beta_prefix(a: f64, b: f64, x: f64) -> f64 {
    let sum = a + b;
    let mean = a / sum;
    let d = (x - mean) * sum;
    let term_a = if d / a > -0.5 {
        a * maths::log1p(d / a)
    } else {
        a * maths::log(x * sum / a)
    };
    let term_b = if -d / b > -0.5 {
        b * maths::log1p(-d / b)
    } else {
        b * maths::log((1.0 - x) * sum / b)
    };
    let exponent = term_a + term_b + 0.5 * maths::log(a * b / sum)
        - LN_SQRT_2PI
        - (stirling_remainder(a) + stirling_remainder(b) - stirling_remainder(sum));
    maths::exp(exponent)
}

/// The continued fraction of the regularised incomplete beta function (modified Lentz).
fn beta_fraction(a: f64, b: f64, x: f64) -> f64 {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    let mut m = 0.0;
    for _ in 0..MAX_ITERATIONS {
        m += 1.0;
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < EPSILON {
            break;
        }
    }
    h
}

/// I_x(a, b) and 1 - I_x(a, b), the regularised incomplete beta function and its complement, for
/// a, b > 0 and 0 ≤ x ≤ 1; whichever side the continued fraction converges on is computed
/// directly, the other as the complement.
pub fn incomplete_beta(a: f64, b: f64, x: f64) -> Tails {
    if !(a > 0.0 && b > 0.0) || !(0.0..=1.0).contains(&x) {
        return Tails::nan();
    }
    if x == 0.0 {
        return Tails {
            lower: 0.0,
            upper: 1.0,
        };
    }
    if x == 1.0 {
        return Tails {
            lower: 1.0,
            upper: 0.0,
        };
    }
    if x < (a + 1.0) / (a + b + 2.0) {
        Tails::direct_lower(beta_prefix(a, b, x) * beta_fraction(a, b, x) / a)
    } else {
        Tails::direct_upper(beta_prefix(b, a, 1.0 - x) * beta_fraction(b, a, 1.0 - x) / b)
    }
}

/// The density of the beta distribution at `x`: I'_x(a, b).
pub fn beta_density(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 || x >= 1.0 {
        let at_zero = x == 0.0 && a < 1.0 || x == 1.0 && b < 1.0;
        if at_zero {
            return f64::INFINITY;
        }
        if x == 0.0 && a == 1.0 {
            return b;
        }
        if x == 1.0 && b == 1.0 {
            return a;
        }
        return 0.0;
    }
    beta_prefix(a, b, x) / (x * (1.0 - x))
}

/// The `x` with I_x(a, b) = p (or 1 - I_x(a, b) = p for `upper`), by safeguarded Newton steps
/// from a normal start on whichever tail is the smaller.
pub fn incomplete_beta_inverse(a: f64, b: f64, p: f64, upper: bool) -> f64 {
    if !(a > 0.0 && b > 0.0) || !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return if upper { 1.0 } else { 0.0 };
    }
    if p == 1.0 {
        return if upper { 0.0 } else { 1.0 };
    }
    let (target, target_upper) = if p <= 0.5 {
        (p, upper)
    } else {
        (1.0 - p, !upper)
    };
    // A tail whose leading term lies below the smallest Float: the quantile is the end of the
    // interval to every digit a double has.
    if !target_upper && (maths::log(target) + maths::log(a) + log_beta(a, b)) / a < -745.0 {
        return 0.0;
    }
    if target_upper && (maths::log(target) + maths::log(b) + log_beta(a, b)) / b < -745.0 {
        return 1.0;
    }
    let sum = a + b;
    let mean = a / sum;
    let sd = (a * b / (sum * sum * (sum + 1.0))).sqrt();
    let z = normal_quantile(target, target_upper);
    let guess = mean + z * sd;
    let mut x = if guess > 0.0 && guess < 1.0 {
        guess
    } else if target_upper {
        // A deep upper tail: 1 - (p b B(a, b))^(1/b) from the leading term near one.
        1.0 - maths::exp((maths::log(target) + maths::log(b) + log_beta(a, b)) / b)
    } else {
        maths::exp((maths::log(target) + maths::log(a) + log_beta(a, b)) / a)
    };
    if !(x > 0.0 && x < 1.0) {
        x = mean;
    }
    newton_with_bracket(
        x,
        0.0,
        1.0,
        |x| incomplete_beta(a, b, x).tail(target_upper) - target,
        |x| {
            let density = beta_density(a, b, x);
            if target_upper { -density } else { density }
        },
        &mut x,
    );
    x
}

/// Newton's method on an increasing or decreasing `f` with a bracket kept at every step: a step
/// that leaves the bracket, or fails to shrink it, is replaced by a bisection (an expansion when
/// the bracket is open above). `f` is the function less its target; `derivative` its slope.
fn newton_with_bracket(
    start: f64,
    mut lo: f64,
    mut hi: f64,
    f: impl Fn(f64) -> f64,
    derivative: impl Fn(f64) -> f64,
    result: &mut f64,
) {
    let mut x = start;
    // The sign of f at the lower end tells which way the root lies.
    let increasing_sign = {
        let probe = if lo.is_finite() && hi.is_finite() {
            0.5 * (lo + hi)
        } else {
            start
        };
        derivative(probe) >= 0.0
    };
    for _ in 0..200 {
        let fx = f(x);
        if fx == 0.0 || fx.is_nan() {
            break;
        }
        // Tighten the bracket from the sign of f.
        let below = (fx < 0.0) == increasing_sign; // x is below the root
        if below {
            lo = x;
        } else {
            hi = x;
        }
        let slope = derivative(x);
        let mut next = if slope != 0.0 && slope.is_finite() {
            x - fx / slope
        } else {
            f64::NAN
        };
        if !(next > lo && next < hi) {
            next = if hi.is_infinite() {
                lo.max(1e-300) * 2.0 + 1.0
            } else {
                0.5 * (lo + hi)
            };
        }
        let step = (next - x).abs();
        x = next;
        if step <= 4.0 * f64::EPSILON * x.abs() || hi - lo <= 4.0 * f64::EPSILON * x.abs() {
            break;
        }
    }
    *result = x;
}

/// n! for n = 0 to 170, each the double nearest the exact value (written by Python's exact
/// integers, rounded once).
const FACTORIALS: [f64; 171] = [
    1.0,
    1.0,
    2.0,
    6.0,
    24.0,
    120.0,
    720.0,
    5040.0,
    40320.0,
    362880.0,
    3628800.0,
    39916800.0,
    479001600.0,
    6227020800.0,
    87178291200.0,
    1307674368000.0,
    20922789888000.0,
    355687428096000.0,
    6402373705728000.0,
    1.21645100408832e+17,
    2.43290200817664e+18,
    5.109094217170944e+19,
    1.1240007277776077e+21,
    2.585201673888498e+22,
    6.204484017332394e+23,
    1.5511210043330986e+25,
    4.0329146112660565e+26,
    1.0888869450418352e+28,
    3.0488834461171387e+29,
    8.841761993739702e+30,
    2.6525285981219107e+32,
    8.222838654177922e+33,
    2.631308369336935e+35,
    8.683317618811886e+36,
    2.9523279903960416e+38,
    1.0333147966386145e+40,
    3.7199332678990125e+41,
    1.3763753091226346e+43,
    5.230226174666011e+44,
    2.0397882081197444e+46,
    8.159152832478977e+47,
    3.345252661316381e+49,
    1.40500611775288e+51,
    6.041526306337383e+52,
    2.658271574788449e+54,
    1.1962222086548019e+56,
    5.502622159812089e+57,
    2.5862324151116818e+59,
    1.2413915592536073e+61,
    6.082818640342675e+62,
    3.0414093201713376e+64,
    1.5511187532873822e+66,
    8.065817517094388e+67,
    4.2748832840600255e+69,
    2.308436973392414e+71,
    1.2696403353658276e+73,
    7.109985878048635e+74,
    4.0526919504877214e+76,
    2.3505613312828785e+78,
    1.3868311854568984e+80,
    8.32098711274139e+81,
    5.075802138772248e+83,
    3.146997326038794e+85,
    1.98260831540444e+87,
    1.2688693218588417e+89,
    8.247650592082472e+90,
    5.443449390774431e+92,
    3.647111091818868e+94,
    2.4800355424368305e+96,
    1.711224524281413e+98,
    1.1978571669969892e+100,
    8.504785885678623e+101,
    6.1234458376886085e+103,
    4.4701154615126844e+105,
    3.307885441519386e+107,
    2.48091408113954e+109,
    1.8854947016660504e+111,
    1.4518309202828587e+113,
    1.1324281178206297e+115,
    8.946182130782976e+116,
    7.156945704626381e+118,
    5.797126020747368e+120,
    4.753643337012842e+122,
    3.945523969720659e+124,
    3.314240134565353e+126,
    2.81710411438055e+128,
    2.4227095383672734e+130,
    2.107757298379528e+132,
    1.8548264225739844e+134,
    1.650795516090846e+136,
    1.4857159644817615e+138,
    1.352001527678403e+140,
    1.2438414054641308e+142,
    1.1567725070816416e+144,
    1.087366156656743e+146,
    1.032997848823906e+148,
    9.916779348709496e+149,
    9.619275968248212e+151,
    9.426890448883248e+153,
    9.332621544394415e+155,
    9.332621544394415e+157,
    9.42594775983836e+159,
    9.614466715035127e+161,
    9.90290071648618e+163,
    1.0299016745145628e+166,
    1.081396758240291e+168,
    1.1462805637347084e+170,
    1.226520203196138e+172,
    1.324641819451829e+174,
    1.4438595832024937e+176,
    1.588245541522743e+178,
    1.7629525510902446e+180,
    1.974506857221074e+182,
    2.2311927486598138e+184,
    2.5435597334721877e+186,
    2.925093693493016e+188,
    3.393108684451898e+190,
    3.969937160808721e+192,
    4.684525849754291e+194,
    5.574585761207606e+196,
    6.689502913449127e+198,
    8.094298525273444e+200,
    9.875044200833601e+202,
    1.214630436702533e+205,
    1.506141741511141e+207,
    1.882677176888926e+209,
    2.372173242880047e+211,
    3.0126600184576594e+213,
    3.856204823625804e+215,
    4.974504222477287e+217,
    6.466855489220474e+219,
    8.47158069087882e+221,
    1.1182486511960043e+224,
    1.4872707060906857e+226,
    1.9929427461615188e+228,
    2.6904727073180504e+230,
    3.659042881952549e+232,
    5.012888748274992e+234,
    6.917786472619489e+236,
    9.615723196941089e+238,
    1.3462012475717526e+241,
    1.898143759076171e+243,
    2.695364137888163e+245,
    3.854370717180073e+247,
    5.5502938327393044e+249,
    8.047926057471992e+251,
    1.1749972043909107e+254,
    1.727245890454639e+256,
    2.5563239178728654e+258,
    3.80892263763057e+260,
    5.713383956445855e+262,
    8.62720977423324e+264,
    1.3113358856834524e+267,
    2.0063439050956823e+269,
    3.0897696138473508e+271,
    4.789142901463394e+273,
    7.471062926282894e+275,
    1.1729568794264145e+278,
    1.853271869493735e+280,
    2.9467022724950384e+282,
    4.7147236359920616e+284,
    7.590705053947219e+286,
    1.2296942187394494e+289,
    2.0044015765453026e+291,
    3.287218585534296e+293,
    5.423910666131589e+295,
    9.003691705778438e+297,
    1.503616514864999e+300,
    2.5260757449731984e+302,
    4.269068009004705e+304,
    7.257415615307999e+306,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edges_of_the_domains_are_answered_not_crashed() {
        assert!(gamma(0.0).is_nan() && gamma(-1.0).is_nan() && gamma(-2.0).is_nan());
        assert_eq!(gamma(171.0), FACTORIALS[170]);
        assert_eq!(gamma(172.0), f64::INFINITY);
        assert_eq!(log_gamma(1.0), 0.0);
        assert_eq!(log_gamma(2.0), 0.0);
        assert_eq!(log_gamma(0.0), f64::INFINITY);
        assert!(factorial(-1).is_nan());
        assert_eq!(factorial(171), f64::INFINITY);
        assert_eq!(factorial(0), 1.0);
        assert_eq!(choose(5, 7), 0.0);
        assert_eq!(choose(5, -1), 0.0);
        assert!(choose(-5, 2).is_nan());
        assert_eq!(choose(10, 3), 120.0);
        assert_eq!(choose(1000, 500), choose(1000, 500));
        assert_eq!(choose(1100, 550), f64::INFINITY);
        assert!(log_beta(0.0, 1.0).is_nan() && log_beta(1.0, -1.0).is_nan());
        assert_eq!(log_beta(1.0, f64::INFINITY), f64::NEG_INFINITY);
        assert!(incomplete_gamma(0.0, 1.0).lower.is_nan());
        assert!(incomplete_gamma(1.0, -1.0).lower.is_nan());
        assert_eq!(incomplete_gamma(2.0, 0.0).upper, 1.0);
        assert_eq!(incomplete_gamma(2.0, f64::INFINITY).lower, 1.0);
        assert!(incomplete_beta(1.0, 1.0, 1.5).lower.is_nan());
        assert_eq!(incomplete_beta(2.0, 2.0, 0.0).upper, 1.0);
        assert_eq!(incomplete_beta(2.0, 2.0, 1.0).lower, 1.0);
        assert_eq!(incomplete_gamma_inverse(2.0, 0.0, false), 0.0);
        assert_eq!(incomplete_gamma_inverse(2.0, 1.0, false), f64::INFINITY);
        assert_eq!(incomplete_beta_inverse(2.0, 3.0, 1.0, true), 0.0);
        assert!(incomplete_beta_inverse(2.0, 3.0, 1.5, true).is_nan());
        assert_eq!(normal_quantile(0.0, false), f64::NEG_INFINITY);
        assert_eq!(normal_quantile(1.0, true), f64::NEG_INFINITY);
        assert!(normal_quantile(-0.1, false).is_nan());
        assert_eq!(normal_quantile(0.5, false), 0.0);
        assert!(normal_tails(f64::NAN).lower.is_nan());
        assert_eq!(normal_tails(f64::NEG_INFINITY).lower, 0.0);
        assert_eq!(normal_tails(-40.0).lower, 0.0);
        assert_eq!(gamma_density(1.0, 0.0), 1.0);
        assert_eq!(gamma_density(0.5, 0.0), f64::INFINITY);
        assert_eq!(beta_density(1.0, 1.0, 0.0), 1.0);
        assert_eq!(beta_density(0.5, 2.0, 0.0), f64::INFINITY);
        assert_eq!(beta_density(2.0, 2.0, 1.0), 0.0);
    }

    #[test]
    fn deep_tails_keep_their_relative_accuracy() {
        // Q(1, x) = e^-x exactly: the far tail is not computed as one minus the other side.
        let q = incomplete_gamma(1.0, 600.0).upper;
        assert!((q - (-600.0f64).exp()).abs() <= 1e-12 * q, "{q:e}");
        // I_x(1, 1) = x: a probability of 1e-200 comes back as itself.
        assert!((incomplete_beta(1.0, 1.0, 1e-200).lower - 1e-200).abs() <= 1e-213);
        // The normal tail at -37 is 5.7e-300, not zero.
        assert!(normal_tails(-37.0).lower > 5e-300);
    }
}
