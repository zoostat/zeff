//! The studentized range (release 0.7.0, step 18): the range of `k` independent standard normals
//! divided by an independent estimate of their standard deviation on `nu` degrees of freedom,
//! the distribution behind `dist.tukey` and Tukey's honest significant difference.
//!
//! The range W of `k` standard normals has, with φ and Φ the normal density and distribution,
//!
//! * lower tail  P(W ≤ w) = k ∫ φ(z) [Φ(z) − Φ(z − w)]^(k−1) dz,
//! * upper tail  P(W > w) = k ∫ φ(z) Φ(z)^(k−1) {1 − [1 − Φ(z − w)/Φ(z)]^(k−1)} dz,
//! * density     p(w) = k (k − 1) ∫ φ(z) φ(z − w) [Φ(z) − Φ(z − w)]^(k−2) dz,
//!
//! and the studentized range Q = W / S, where ν S² is chi-squared on ν degrees of freedom, has
//! each of the three as the integral of the same function of the range at w = q s against the
//! density of S (times s for the density). Both integrals are taken over the whole line, the
//! outer one in t = ln s, where the integrand is smooth and dies away at both ends, so the
//! trapezoidal rule converges faster than any power of its step. Each tail is its own integral
//! of positive terms: the upper tail is never one minus the lower one (Engine and Project
//! Specification, section 13).
//!
//! Everything is carried as a logarithm, so an inner integral of 1e-300 is as accurate as one
//! of a half, and the arithmetic is the fixed-order arithmetic of this crate: the elementary
//! functions of `maths`, `erfc` from `libm`, sums from the first term to the last.

use crate::maths::{exp, expm1, log, log1p, sqrt};
use crate::special::{Tails, erfc, normal_quantile, stirling_remainder};

const LN_SQRT_2PI: f64 = 0.918_938_533_204_672_8; // ln(sqrt(2 pi))
const LN_SQRT_PI: f64 = 0.572_364_942_924_700_1; // ln(sqrt(pi))

/// The first step of a quadrature, in units of the width of the integrand's peak.
const FIRST_STEP: f64 = 0.8;
/// Three interleaved sums must agree to this before their mean is taken.
const AGREEMENT: f64 = 1e-6;
/// A term this small beside the running sum ends a sum.
const NEGLIGIBLE: f64 = 1e-17;
/// Past this the upper tail of the range of standard normals is below the smallest double.
const WIDEST: f64 = 60.0;
/// From this many degrees of freedom the estimate of the standard deviation is the standard
/// deviation to every digit a double holds, and the studentized range is the range itself.
const KNOWN_VARIANCE: f64 = 1e30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Lower,
    Upper,
    Density,
}

/// ln x, with ln of nothing (or of a rounding below nothing) as −∞.
fn ln(x: f64) -> f64 {
    if x > 0.0 { log(x) } else { f64::NEG_INFINITY }
}

/// 1 − Φ(x).
fn upper_tail(x: f64) -> f64 {
    0.5 * erfc(x * std::f64::consts::FRAC_1_SQRT_2)
}

/// e^x − 1 − x, by its series near zero, where the plain form cancels.
fn expm1_minus(x: f64) -> f64 {
    if x.abs() <= 1.0 {
        let mut term = 0.5 * x * x;
        let mut total = term;
        let mut n = 2.0;
        loop {
            n += 1.0;
            term *= x / n;
            total += term;
            if term.abs() <= 1e-18 * total.abs() {
                return total;
            }
        }
    }
    expm1(x) - x
}

/// ln ∫ exp(logf(x)) dx over the whole line, for a function with one peak near `guess`, whose
/// width is within a few doublings of `stride`.
///
/// Three steps. Find the peak: walk from the guess in doubling strides until three points
/// bracket it, then halve the bracket until the two outer points are within two units of the
/// middle one and the two halves are of like length (without the second condition the middle
/// and one outer point can straddle a narrow peak at equal heights far below it). Take the
/// width σ of the peak from the parabola through the three points. Then sum the trapezoidal
/// rule outwards from the peak, in each direction until two terms in a row are negligible, on
/// three grids of step 0.8 σ set a third of a step apart: their mean is the rule at a third of
/// the step, and their spread measures the error of each whatever the place of the grid (two
/// grids half a step apart agree by accident when the peak sits a quarter of a step from a
/// node). The step is divided by three until the spread is below 1e-6 of the mean, which for
/// a rule whose error is cubed when the step is divided by three puts the mean within about
/// 1e-18.
fn log_integral(logf: impl Fn(f64) -> f64, guess: f64, stride: f64) -> f64 {
    let (mut a, mut b, mut c) = (guess - stride, guess, guess + stride);
    let (mut fa, mut fb, mut fc) = (logf(a), logf(b), logf(c));
    if fa.is_nan() || fb.is_nan() || fc.is_nan() {
        return f64::NAN;
    }
    let mut stride = stride;
    for _ in 0..400 {
        if fb >= fa && fb >= fc {
            break;
        }
        stride *= 2.0;
        if fc > fb {
            (a, fa, b, fb) = (b, fb, c, fc);
            c = b + stride;
            fc = logf(c);
        } else {
            (c, fc, b, fb) = (b, fb, a, fa);
            a = b - stride;
            fa = logf(a);
        }
    }
    if fb == f64::NEG_INFINITY {
        return f64::NEG_INFINITY;
    }
    for _ in 0..400 {
        let (left, right) = (b - a, c - b);
        let balanced = left <= 2.0 * right && right <= 2.0 * left;
        if fb - fa <= 2.0 && fb - fc <= 2.0 && balanced {
            break;
        }
        if left > 2.0 * right || (right <= 2.0 * left && fb - fa > fb - fc) {
            let m = 0.5 * (a + b);
            let fm = logf(m);
            if fm > fb {
                (c, fc, b, fb) = (b, fb, m, fm);
            } else {
                (a, fa) = (m, fm);
            }
        } else {
            let m = 0.5 * (b + c);
            let fm = logf(m);
            if fm > fb {
                (a, fa, b, fb) = (b, fb, m, fm);
            } else {
                (c, fc) = (m, fm);
            }
        }
    }
    let curvature = -2.0 * ((fc - fb) / (c - b) - (fb - fa) / (b - a)) / (c - a);
    let sigma = if curvature > 0.0 && curvature.is_finite() {
        1.0 / sqrt(curvature)
    } else {
        0.5 * (c - a)
    };
    let (mode, top) = (b, fb);

    // The sum of exp(logf − top) over the nodes mode + (i + offset) step, i = …, −1, 0, 1, …,
    // times the step.
    let sweep = |step: f64, offset: f64| -> f64 {
        let mut total = 0.0;
        for direction in [1.0, -1.0] {
            let mut i = if direction > 0.0 { 0.0 } else { 1.0 };
            let mut previous = f64::INFINITY;
            let mut quiet = 0;
            loop {
                let term = exp(logf(mode + (direction * i + offset) * step) - top);
                total += term;
                if term <= NEGLIGIBLE * total && term <= previous {
                    quiet += 1;
                    if quiet == 2 {
                        break;
                    }
                } else {
                    quiet = 0;
                }
                previous = term;
                i += 1.0;
                if i > 200_000.0 {
                    return f64::NAN;
                }
            }
        }
        total * step
    };

    let mut step = FIRST_STEP * sigma;
    let mut parts = [
        sweep(step, 0.0),
        sweep(step, 1.0 / 3.0),
        sweep(step, 2.0 / 3.0),
    ];
    for _ in 0..7 {
        let mean = (parts[0] + parts[1] + parts[2]) / 3.0;
        let spread = parts[0].max(parts[1]).max(parts[2]) - parts[0].min(parts[1]).min(parts[2]);
        if spread <= AGREEMENT * mean {
            return log(mean) + top;
        }
        step /= 3.0;
        parts = [mean, sweep(step, 1.0 / 3.0), sweep(step, 2.0 / 3.0)];
    }
    log((parts[0] + parts[1] + parts[2]) / 3.0) + top
}

/// ln[Φ(z) − Φ(z − w)] for w > 0: one minus the two tails outside the band where they are
/// small; the Taylor series of the band about its middle, 2 φ(u) Σ He_2j(u) h^(2j+1)/(2j+1)!
/// with h = w/2 and u the distance of the middle from zero, where the band is narrow; and the
/// difference of the two tails on the same side where the band lies in one tail.
fn ln_band(z: f64, w: f64) -> f64 {
    let h = 0.5 * w;
    let u = (z - h).abs();
    let outside = upper_tail(u + h) + upper_tail(h - u);
    if outside <= 0.5 {
        return log1p(-outside);
    }
    if h <= 0.5 && u * h <= 1.0 {
        let (mut before, mut hermite) = (1.0, u); // He_0 and He_1
        let mut n = 1.0;
        let mut total = 1.0;
        let mut power = 1.0;
        for j in 1..40 {
            // Two steps of He_(n+1) = u He_n − n He_(n−1), to He_2j and on to He_(2j+1).
            (before, hermite) = (hermite, u * hermite - n * before);
            n += 1.0;
            let even = hermite;
            (before, hermite) = (hermite, u * hermite - n * before);
            n += 1.0;
            let j = f64::from(j);
            power *= h * h / ((2.0 * j) * (2.0 * j + 1.0));
            let term = even * power;
            total += term;
            if term.abs() <= 1e-18 * total.abs() {
                break;
            }
        }
        return log(2.0 * h) - 0.5 * u * u - LN_SQRT_2PI + ln(total);
    }
    if u >= h {
        return ln(upper_tail(u - h) - upper_tail(u + h));
    }
    ln(1.0 - outside)
}

/// The logarithm of the integrand over z of one of the three integrals for the range of `k`
/// standard normals at w, without its leading k or k (k − 1).
fn inner(kind: Kind, z: f64, w: f64, k: f64) -> f64 {
    match kind {
        Kind::Lower => -0.5 * z * z - LN_SQRT_2PI + (k - 1.0) * ln_band(z, w),
        Kind::Density => {
            let band = if k > 2.0 {
                (k - 2.0) * ln_band(z, w)
            } else {
                0.0
            };
            -0.5 * z * z - 0.5 * (z - w) * (z - w) - 2.0 * LN_SQRT_2PI + band
        }
        Kind::Upper => {
            // ln Φ(z), and d = −ln(1 − Φ(z − w)/Φ(z)), so that the brace is 1 − exp(−(k−1) d).
            let below = if z >= 0.0 {
                log1p(-upper_tail(z))
            } else {
                ln(upper_tail(-z))
            };
            if below == f64::NEG_INFINITY {
                return f64::NEG_INFINITY;
            }
            let ratio = upper_tail(w - z) / exp(below);
            let d = if ratio <= 0.5 {
                -log1p(-ratio)
            } else {
                below - ln_band(z, w)
            };
            let x = (k - 1.0) * d;
            if x <= 0.0 {
                return f64::NEG_INFINITY;
            }
            -0.5 * z * z - LN_SQRT_2PI + (k - 1.0) * below + ln(-expm1(-x))
        }
    }
}

/// The logarithm of P(W ≤ w), P(W > w) or the density of the range W of `k` standard normals;
/// `centre` is roughly where the largest of them sits.
fn range(kind: Kind, w: f64, k: f64, centre: f64) -> f64 {
    if w <= 0.0 {
        return match kind {
            Kind::Lower => f64::NEG_INFINITY,
            Kind::Upper => 0.0,
            // Two normals differ by nothing with density 1/sqrt(pi); three or more leave no
            // density at a range of nothing.
            Kind::Density if k > 2.0 => f64::NEG_INFINITY,
            Kind::Density => -LN_SQRT_PI,
        };
    }
    if w >= WIDEST {
        return match kind {
            Kind::Lower => 0.0,
            Kind::Upper | Kind::Density => f64::NEG_INFINITY,
        };
    }
    let h = 0.5 * w;
    let (guess, scale) = match kind {
        Kind::Lower => (centre.min(h), log(k)),
        Kind::Upper => (centre.max(h), log(k)),
        Kind::Density => (h, log(k) + log(k - 1.0)),
    };
    scale + log_integral(|z| inner(kind, z, w, k), guess, 1.0 / sqrt(k))
}

/// The logarithm of P(Q ≤ q), P(Q > q) or the density of the studentized range Q of `k` groups
/// on `nu` degrees of freedom, for q ≥ 0.
///
/// With a = ν/2 the density of t = ln S is c exp(−a (e^(2t) − 1 − 2t)), where
/// ln c = ln 2 + ½ ln a − ln √(2π) − δ(a) and δ is the remainder of Stirling's series, so the
/// constant loses nothing to the size of ν. The integrand peaks where the range function and
/// this weight meet: at t = 0 when the range function is flat there, near ln(2 m / q) (m the
/// centre of the largest normal) when q is so far out that only an unusual S brings q S back
/// to a typical range, and never to the right of ½ ln(1 + (k − 1)/ν), where the weight
/// overtakes the power law of a small range.
fn studentized(kind: Kind, q: f64, k: f64, nu: f64) -> f64 {
    let centre = normal_quantile(1.0 / (k + 1.0), true);
    if nu >= KNOWN_VARIANCE {
        return range(kind, q, k, centre);
    }
    let a = 0.5 * nu;
    let constant = std::f64::consts::LN_2 + 0.5 * log(a) - LN_SQRT_2PI - stirling_remainder(a);
    let typical = log(2.0 * centre / q);
    let rising = 0.5 * log1p((k - 1.0) / nu);
    let guess = match kind {
        Kind::Lower if typical > 0.0 => typical.min(rising),
        Kind::Lower => 0.0,
        Kind::Upper => typical.min(0.0),
        Kind::Density => typical.min(rising),
    };
    let logf = |t: f64| {
        let weight = constant - a * expm1_minus(2.0 * t);
        let value = range(kind, q * exp(t), k, centre) + weight;
        if kind == Kind::Density {
            value + t
        } else {
            value
        }
    };
    log_integral(logf, guess, 1.0 / sqrt(2.0 * nu + 2.0))
}

/// Whether `groups` and `df` are a studentized range's: 2 or more groups, 1 or more degrees of
/// freedom.
fn defined(groups: f64, df: f64) -> bool {
    groups >= 2.0 && groups.is_finite() && df >= 1.0
}

/// P(Q ≤ q) and P(Q > q) for the studentized range of `groups` means on `df` degrees of
/// freedom, each tail from its own integral where it is the smaller. Not-a-number unless
/// `groups` is 2 or more and `df` 1 or more.
pub fn tails(q: f64, groups: f64, df: f64) -> Tails {
    if q.is_nan() || !defined(groups, df) {
        return Tails {
            lower: f64::NAN,
            upper: f64::NAN,
        };
    }
    if q <= 0.0 {
        return Tails {
            lower: 0.0,
            upper: 1.0,
        };
    }
    if q == f64::INFINITY {
        return Tails {
            lower: 1.0,
            upper: 0.0,
        };
    }
    // Start with the tail that is likely the smaller; the other follows by subtraction when
    // the first is at most a half, and from its own integral otherwise.
    let far = q > 2.0 * normal_quantile(1.0 / (groups + 1.0), true);
    let (first, second) = if far {
        (Kind::Upper, Kind::Lower)
    } else {
        (Kind::Lower, Kind::Upper)
    };
    let mut small = exp(studentized(first, q, groups, df)).min(1.0);
    let mut kind = first;
    if small > 0.5 {
        small = exp(studentized(second, q, groups, df)).min(1.0);
        kind = second;
    }
    if kind == Kind::Lower {
        Tails {
            lower: small,
            upper: 1.0 - small,
        }
    } else {
        Tails {
            lower: 1.0 - small,
            upper: small,
        }
    }
}

/// The density of the studentized range at q.
pub fn density(q: f64, groups: f64, df: f64) -> f64 {
    if q.is_nan() || !defined(groups, df) {
        return f64::NAN;
    }
    if q < 0.0 || q == f64::INFINITY {
        return 0.0;
    }
    exp(studentized(Kind::Density, q, groups, df))
}

/// The q with P(Q ≤ q) = p, or with P(Q > q) = p for `upper`.
///
/// The root is found in the tail that is at most a half, so a probability near one keeps its
/// digits, and in the coordinates (ln q, ln tail), where both tails are close to straight
/// lines: doubling steps from the centre of the distribution until the root is bracketed, then
/// false position with the Illinois safeguard until a step moves the point by less than a few
/// units in its last place.
pub fn quantile(p: f64, upper: bool, groups: f64, df: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) || !defined(groups, df) {
        return f64::NAN;
    }
    let (kind, target) = match (upper, p <= 0.5) {
        (false, true) | (true, false) => (Kind::Lower, if upper { 1.0 - p } else { p }),
        _ => (Kind::Upper, if upper { p } else { 1.0 - p }),
    };
    if target == 0.0 {
        return if kind == Kind::Lower {
            0.0
        } else {
            f64::INFINITY
        };
    }
    let goal = log(target);
    let g = |x: f64| studentized(kind, exp(x), groups, df) - goal;

    let mut xa = log(2.0 * normal_quantile(1.0 / (groups + 1.0), true));
    let mut ga = g(xa);
    if ga == 0.0 {
        return exp(xa);
    }
    // The lower tail rises with q and the upper one falls.
    let toward = if (ga < 0.0) == (kind == Kind::Lower) {
        1.0
    } else {
        -1.0
    };
    let mut width = 0.5;
    let (mut xb, mut gb);
    loop {
        xb = xa + toward * width;
        gb = g(xb);
        if gb == 0.0 {
            return exp(xb);
        }
        if gb.is_nan() {
            return f64::NAN;
        }
        if (gb < 0.0) != (ga < 0.0) {
            break;
        }
        if xb.abs() > 800.0 {
            // No double is far enough out to have so small a tail.
            return if toward > 0.0 { f64::INFINITY } else { 0.0 };
        }
        (xa, ga) = (xb, gb);
        width *= 2.0;
    }
    for _ in 0..200 {
        let middle = 0.5 * (xa + xb);
        let mut x = if ga.is_finite() && gb.is_finite() {
            xb - gb * (xb - xa) / (gb - ga)
        } else {
            middle
        };
        if !(x > xa.min(xb) && x < xa.max(xb)) {
            x = middle;
        }
        if x == xa || x == xb {
            break;
        }
        // A step this short from the last point is the last: the iteration gains digits faster
        // than one for one, so the point it leads to is right to the last digits of a double,
        // and the bracket need not close from both sides.
        if (x - xb).abs() <= 8.0 * f64::EPSILON * x.abs().max(1.0) {
            return exp(x);
        }
        let gx = g(x);
        if gx == 0.0 {
            return exp(x);
        }
        if gx.is_nan() {
            return f64::NAN;
        }
        if (gx < 0.0) != (gb < 0.0) {
            (xa, ga) = (xb, gb);
        } else {
            ga *= 0.5;
        }
        (xb, gb) = (x, gx);
        if (xb - xa).abs() <= 4.0 * f64::EPSILON * xb.abs().max(1.0) {
            break;
        }
    }
    exp(xb)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(what: &str, got: f64, want: f64, relative: f64) {
        let error = ((got - want) / want).abs();
        assert!(
            error <= relative,
            "{what}: {got:e} against {want:e}, relative error {error:e}"
        );
    }

    #[test]
    fn two_groups_on_one_and_two_degrees_of_freedom_have_closed_forms() {
        // With t = q / sqrt(2): (2 / pi) atan(t) below q on one degree of freedom, and
        // 2 / ((2 + t²) + t sqrt(2 + t²)) above q on two.
        for q in [0.01, 0.3, 1.0, 2.5, 7.0, 40.0, 900.0] {
            let t = q * std::f64::consts::FRAC_1_SQRT_2;
            let one = tails(q, 2.0, 1.0);
            close(
                "lower, df 1",
                one.lower,
                2.0 / std::f64::consts::PI * t.atan(),
                1e-13,
            );
            close(
                "upper, df 1",
                one.upper,
                2.0 / std::f64::consts::PI * (1.0 / t).atan(),
                1e-13,
            );
            let two = tails(q, 2.0, 2.0);
            let above = 2.0 / ((2.0 + t * t) + t * (2.0 + t * t).sqrt());
            close("upper, df 2", two.upper, above, 1e-13);
            close("lower, df 2", two.lower, t / (2.0 + t * t).sqrt(), 1e-13);
        }
    }

    #[test]
    fn the_places_that_once_went_wrong_stay_right() {
        // A narrow peak that the search for it once stepped over.
        let t = 3.0 * std::f64::consts::FRAC_1_SQRT_2;
        close(
            "upper, 2 groups, df 1e10",
            tails(3.0, 2.0, 1e10).upper,
            erfc(t / std::f64::consts::SQRT_2),
            1e-9,
        );
        // A point at which two grids half a step apart agree before either has converged.
        close(
            "upper, 100 groups, df 1e6",
            tails(10.0, 100.0, 1e6).upper,
            7.592_545_909_318_968e-9,
            1e-13,
        );
        // A tail that R reports at twice its size.
        close(
            "upper, 10 groups, df 5",
            tails(50.0, 10.0, 5.0).upper,
            4.713_523_458_792_231e-6,
            1e-13,
        );
        // A range so small that the weight of the degrees of freedom decides where the mass is.
        close(
            "lower, 5 groups, df 10",
            tails(0.01, 5.0, 10.0).lower,
            6.796_609_600_512_868e-10,
            1e-13,
        );
        close(
            "lower, 3 groups, df 20",
            tails(3.5, 3.0, 20.0).lower,
            0.944_108_150_918_251_1,
            1e-13,
        );
        close(
            "density, 5 groups, df 10",
            density(3.0, 5.0, 10.0),
            0.266_969_388_307_438_66,
            1e-13,
        );
    }

    #[test]
    fn the_ends_and_the_undefined_cases() {
        let nothing = tails(0.0, 3.0, 10.0);
        assert_eq!((nothing.lower, nothing.upper), (0.0, 1.0));
        let below = tails(-2.0, 3.0, 10.0);
        assert_eq!((below.lower, below.upper), (0.0, 1.0));
        let all = tails(f64::INFINITY, 3.0, 10.0);
        assert_eq!((all.lower, all.upper), (1.0, 0.0));
        assert!(tails(f64::NAN, 3.0, 10.0).lower.is_nan());
        assert!(tails(1.0, 1.0, 10.0).lower.is_nan());
        assert!(tails(1.0, 3.0, 0.5).upper.is_nan());
        assert!(tails(1.0, 3.0, f64::NAN).upper.is_nan());
        assert_eq!(density(-1.0, 3.0, 10.0), 0.0);
        assert_eq!(density(0.0, 3.0, 10.0), 0.0);
        assert!(density(0.0, 2.0, 10.0) > 0.5);
        assert_eq!(quantile(0.0, false, 3.0, 10.0), 0.0);
        assert_eq!(quantile(1.0, false, 3.0, 10.0), f64::INFINITY);
        assert_eq!(quantile(0.0, true, 3.0, 10.0), f64::INFINITY);
        assert_eq!(quantile(1.0, true, 3.0, 10.0), 0.0);
        assert!(quantile(1.5, false, 3.0, 10.0).is_nan());
        assert!(quantile(f64::NAN, false, 3.0, 10.0).is_nan());
    }

    #[test]
    fn the_quantile_inverts_each_tail() {
        for (groups, df) in [
            (2.0, 1.0),
            (3.0, 20.0),
            (10.0, 5.0),
            (100.0, 1e6),
            (4.0, 1e40),
        ] {
            for p in [1e-12, 0.001, 0.3, 0.5, 0.95, 0.999] {
                let q = quantile(p, false, groups, df);
                close("lower", tails(q, groups, df).lower, p, 1e-12);
                let q = quantile(p, true, groups, df);
                close("upper", tails(q, groups, df).upper, p, 1e-12);
            }
        }
    }
}
