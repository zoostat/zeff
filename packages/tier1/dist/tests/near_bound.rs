//! A truncation near its bounds (release 0.8.0, step 20i): the probability between a bound and a
//! near point taken without the subtraction that loses its digits, and the quantile inverted on
//! the same form. The reference is the `near_bound` list of `tests/reference/r/distributions.json`
//! (the series of the integral of the density, and the exponential's closed forms); the property
//! is that the distribution function and the quantile undo each other from deep in one tail to
//! deep in the other.

use serde_json::Value as Json;
use zeff_core::value::Distribution;
use zeff_dist::families;
use zeff_numeric::random::Stream;
use zeff_testkit::root;

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn number(json: &Json) -> f64 {
    match json.as_str().unwrap() {
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        other => other.parse().unwrap(),
    }
}

fn family(name: &'static str, params: Vec<(&'static str, f64)>) -> Distribution {
    Distribution::Family { name, params }
}

fn truncated(inner: Distribution, lower: f64, upper: f64) -> Distribution {
    Distribution::Truncated {
        inner: Box::new(inner),
        lower,
        upper,
    }
}

fn standard_normal() -> Distribution {
    family("normal", vec![("mean", 0.0), ("sd", 1.0)])
}

/// The inner family of a reference case.
fn inner_of(case: &Json) -> Distribution {
    let param = |name: &str| number(&case["params"][name]);
    match case["inner"].as_str().unwrap() {
        "normal" => family("normal", vec![("mean", param("mean")), ("sd", param("sd"))]),
        "exponential" => family("exponential", vec![("rate", param("rate"))]),
        "t" => family("t", vec![("df", param("df"))]),
        other => panic!("no inner family `{other}` in this test"),
    }
}

fn relative(got: f64, want: f64) -> f64 {
    if got == want {
        0.0
    } else {
        ((got - want) / want).abs()
    }
}

/// The reference is written with fifteen significant digits, so half a unit of the fifteenth is
/// its own rounding; the engine is held to 1e-13 beyond that.
const HELD_TO: f64 = 1e-13;

#[test]
fn both_tails_and_the_quantile_agree_with_the_series_near_each_bound() {
    let text =
        std::fs::read_to_string(root().join("tests/reference/r/distributions.json")).unwrap();
    let r: Json = serde_json::from_str(&text).unwrap();
    let cases = r["near_bound"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    let mut misses = Vec::new();
    for case in cases {
        let (lower, upper) = (
            number(&case["bounds"]["lower"]),
            number(&case["bounds"]["upper"]),
        );
        let d = truncated(inner_of(case), lower, upper);
        let name = format!("{} on [{lower}, {upper}]", case["inner"].as_str().unwrap());
        for (i, x) in case["x"].as_array().unwrap().iter().enumerate() {
            let x = number(x);
            let got = families::cdf(&d, x);
            for (side, got, want) in [
                ("lower", got.lower, number(&case["lower"][i])),
                ("upper", got.upper, number(&case["upper"][i])),
            ] {
                if relative(got, want) > HELD_TO {
                    misses.push(format!("{name}: {side} tail at {x:e}: {got:e}, R {want:e}"));
                }
            }
        }
        for (i, p) in case["p"].as_array().unwrap().iter().enumerate() {
            let p = number(p);
            let got = families::quantile(&d, p);
            let want = number(&case["quantile"][i]);
            // The median of an interval even about zero is zero, and R's root is the rounding
            // of its last step: both are held to zero at the scale of the interval.
            let nothing = f64::EPSILON * (upper - lower);
            if lower < 0.0 && upper > 0.0 && want.abs() < nothing {
                if got.abs() >= nothing {
                    misses.push(format!("{name}: quantile at {p:e}: {got:e}, R {want:e}"));
                }
                continue;
            }
            if relative(got, want) > HELD_TO {
                misses.push(format!("{name}: quantile at {p:e}: {got:e}, R {want:e}"));
            }
        }
    }
    assert!(misses.is_empty(), "{}", misses.join("\n"));
}

/// The case of the checklist: a standard normal truncated to [0, 2] at 1.196288e-12, where the
/// difference of the two tails gave 1.0000734e-12. The first two terms of the series are exact
/// to far more than sixteen digits there.
#[test]
fn the_case_that_lost_its_digits() {
    let d = truncated(standard_normal(), 0.0, 2.0);
    let x = 1.196288e-12_f64;
    // phi(0) * x / (Phi(2) - 1/2), with R's 0.39894228040143268 and 0.47724986805182079.
    let want = 0.398_942_280_401_432_7 * x / 0.477_249_868_051_820_8;
    let got = families::cdf(&d, x).lower;
    assert!(relative(got, want) < 1e-14, "{got:e} against {want:e}");
    let back = families::quantile(&d, got);
    assert!(relative(back, x) < 1e-14, "{back:e} against {x:e}");
    let at = families::quantile(&d, 1e-12);
    let want = 1e-12 * 0.477_249_868_051_820_8 / 0.398_942_280_401_432_7;
    assert!(relative(at, want) < 1e-14, "{at:e} against {want:e}");
}

/// An interval far out in a tail holds probability, and is a distribution like any other: the
/// difference of the lower tails gave these no probability at all.
#[test]
fn an_interval_far_in_the_upper_tail_holds_its_probability() {
    let normal = truncated(standard_normal(), 9.0, f64::INFINITY);
    let mass = families::truncated_mass(&standard_normal(), 9.0, f64::INFINITY);
    // R: pnorm(9, lower.tail = FALSE) = 1.1285884059538408e-19.
    assert!(
        relative(mass, 1.128_588_405_953_840_8e-19) < 1e-13,
        "{mass:e}"
    );
    // R: qnorm(pnorm(9, lower.tail = FALSE) / 2, lower.tail = FALSE) = 9.0757870654918325.
    let median = families::quantile(&normal, 0.5);
    assert!(relative(median, 9.075_787_065_491_832) < 1e-13, "{median}");

    // A Poisson with mean 3 at 40 and above. R: dpois(40, 3) / ppois(39, 3, lower.tail = FALSE)
    // = 0.92696577094831922, and dpois(41, 3) over the same = 0.067826763727925773.
    let poisson = truncated(
        family("poisson", vec![("lambda", 3.0)]),
        40.0,
        f64::INFINITY,
    );
    assert!(relative(families::pdf(&poisson, 40.0), 0.926_965_770_948_319_2) < 1e-12);
    assert!(relative(families::pdf(&poisson, 41.0), 0.067_826_763_727_925_77) < 1e-12);
    assert!(relative(families::cdf(&poisson, 40.0).lower, 0.926_965_770_948_319_2) < 1e-12);
    assert_eq!(families::quantile(&poisson, 0.5), 40.0);
    assert_eq!(families::quantile(&poisson, 0.93), 41.0);
    assert_eq!(families::quantile(&poisson, 0.999_999), 45.0);
}

/// A discrete truncation's quantile is the smallest whole number whose probability at or below
/// reaches p, so the quantile of a point's own probability is the point.
#[test]
fn a_discrete_quantile_gives_back_the_point_of_its_probability() {
    let cases = [
        (family("poisson", vec![("lambda", 3.0)]), 1.0, 6.0),
        (
            family("poisson", vec![("lambda", 3.0)]),
            40.0,
            f64::INFINITY,
        ),
        (family("poisson", vec![("lambda", 250.0)]), 300.0, 340.0),
        (
            family("binomial", vec![("n", 50.0), ("p", 0.2)]),
            25.0,
            50.0,
        ),
        (
            family("negbinomial", vec![("size", 3.0), ("p", 0.4)]),
            f64::NEG_INFINITY,
            12.0,
        ),
    ];
    for (inner, lower, upper) in cases {
        let d = truncated(inner, lower, upper);
        let from = lower.max(0.0) as i64;
        let to = if upper.is_finite() {
            upper as i64
        } else {
            from + 30
        };
        for k in from..=to {
            let k = k as f64;
            if families::pdf(&d, k) < 1e-300 {
                continue;
            }
            let p = families::cdf(&d, k).lower;
            if p >= 1.0 {
                // The quantile at 1 is the upper bound, whatever the point.
                continue;
            }
            let back = families::quantile(&d, p);
            // Two points whose probabilities at or below round to the same double share the first.
            assert!(
                back == k || (back < k && families::cdf(&d, back).lower == p),
                "{d:?}: the quantile of P(X <= {k}) = {p:e} is {back}"
            );
        }
    }
}

/// One of the continuous families with parameters drawn from the stream.
fn any_inner(stream: &mut Stream) -> Distribution {
    let u = |stream: &mut Stream, low: f64, high: f64| low + (high - low) * stream.uniform();
    match (stream.uniform() * 8.0) as usize {
        0 => family(
            "normal",
            vec![("mean", u(stream, -5.0, 5.0)), ("sd", u(stream, 0.1, 4.0))],
        ),
        1 => family(
            "lognormal",
            vec![
                ("meanlog", u(stream, -1.0, 1.0)),
                ("sdlog", u(stream, 0.2, 1.5)),
            ],
        ),
        2 => family("exponential", vec![("rate", u(stream, 0.1, 5.0))]),
        3 => family("t", vec![("df", u(stream, 1.0, 40.0))]),
        4 => family("chi2", vec![("df", u(stream, 0.5, 30.0))]),
        5 => family(
            "gamma",
            vec![
                ("shape", u(stream, 0.3, 9.0)),
                ("rate", u(stream, 0.2, 4.0)),
            ],
        ),
        6 => family(
            "beta",
            vec![("a", u(stream, 0.4, 6.0)), ("b", u(stream, 0.4, 6.0))],
        ),
        _ => family(
            "f",
            vec![("df1", u(stream, 2.0, 20.0)), ("df2", u(stream, 3.0, 40.0))],
        ),
    }
}

/// The distribution function and the quantile undo each other, on the lower tail up to one half
/// and on the upper tail beyond, for probabilities from 1e-300 to 1 - 1e-12: wherever the
/// truncation is cut (in the lower tail, across the middle, in the upper tail, or to a sliver).
#[test]
fn the_distribution_function_and_the_quantile_undo_each_other() {
    let mut stream = Stream::new(20_261_119, 1, 0);
    let cases = property_cases(300);
    let mut tried = 0;
    for case in 0..cases {
        let inner = any_inner(&mut stream);
        // Two inner probabilities for the bounds: anywhere, or both deep in one tail, or close
        // together.
        let kind = case % 4;
        let (low, high) = match kind {
            0 => (stream.uniform() * 0.5, 0.5 + stream.uniform() * 0.5),
            1 => (1e-9 * stream.uniform(), 1e-6 * (1.0 + stream.uniform())),
            2 => {
                let at = 0.05 + 0.9 * stream.uniform();
                (at, at + 1e-7 * (1.0 + stream.uniform()))
            }
            _ => (
                1.0 - 1e-6 * (1.0 + stream.uniform()),
                1.0 - 1e-9 * stream.uniform(),
            ),
        };
        let lower = families::quantile(&inner, low);
        let upper = families::quantile(&inner, high);
        if !(lower.is_finite() && upper.is_finite() && lower < upper) {
            continue;
        }
        let mass = families::truncated_mass(&inner, lower, upper);
        assert!(mass > 0.0, "{inner:?} on [{lower}, {upper}]");
        let d = truncated(inner, lower, upper);
        for exponent in [300.0, 100.0, 30.0, 12.0, 6.0, 3.0, 1.0, 0.4] {
            let small = 10f64.powf(-exponent * (0.5 + 0.5 * stream.uniform()));
            for p in [small, 1.0 - small.max(1e-12)] {
                let x = families::quantile(&d, p);
                assert!((lower..=upper).contains(&x), "{d:?} at {p:e}: {x}");
                let back = families::cdf(&d, x);
                let (got, want) = if p <= 0.5 {
                    (back.lower, p)
                } else {
                    (back.upper, 1.0 - p)
                };
                // A quantile is a double. Beside a bound that is not zero it cannot be placed
                // more finely than its last digit, and the probability it gives back moves by
                // that digit's share of the distance to the bound.
                let distance = if p <= 0.5 { x - lower } else { upper - x };
                let grain = f64::EPSILON * x.abs() / distance;
                if grain > 1e-3 {
                    // Within a few doubles of the bound: the quantile is a double beside the
                    // root, so the doubles on either side of it stand on either side of the
                    // target.
                    let miss_at = |at: f64| {
                        let tails = families::cdf(&d, at.clamp(lower, upper));
                        if p <= 0.5 {
                            tails.lower - want
                        } else {
                            want - tails.upper
                        }
                    };
                    assert!(
                        miss_at(x.next_down()) <= 0.0 && miss_at(x.next_up()) >= 0.0,
                        "{d:?} at {p:e}: the quantile {x:e} gives {got:e} back"
                    );
                    continue;
                }
                let miss = relative(got, want);
                assert!(
                    miss <= 2.0 * grain + 1e-13,
                    "{d:?} at {p:e}: the quantile {x:e} gives {got:e} back ({miss:e}, of which \
                     the last digit of the quantile explains {grain:e})"
                );
                tried += 1;
            }
        }
    }
    // Most trials are held to thirteen digits; the rest stood within a few doubles of a bound.
    assert!(tried >= 8 * cases, "{tried} of {} trials", 16 * cases);
}

/// A mixture with a component far narrower than the interval between a bound and a point: the
/// rule that integrates the density steps over it, the disagreement with the tails shows, and
/// the difference of the tails is kept.
#[test]
fn a_narrow_component_of_a_mixture_is_not_stepped_over() {
    let narrow = family("normal", vec![("mean", 0.0004), ("sd", 1e-9)]);
    let mixture = Distribution::Mixture {
        parts: vec![(standard_normal(), 0.999), (narrow, 0.001)],
    };
    let d = truncated(mixture, 0.0, 2.0);
    // The wide component gives [0, 0.001] 0.999 * (Phi(0.001) - 1/2) and the narrow one all of
    // its 0.001; the interval's whole probability is 0.999 * (Phi(2) - 1/2) + 0.001.
    let part = 0.999 * 0.000_398_942_213_911_062_64 + 0.001;
    let whole = 0.999 * 0.477_249_868_051_820_8 + 0.001;
    let got = families::cdf(&d, 0.001).lower;
    assert!(relative(got, part / whole) < 1e-12, "{got:e}");
}

/// The same for a jump: a uniform component that begins inside the interval. The density steps
/// up at 0.0001, where the rule can pass it unseen.
#[test]
fn a_jump_in_the_density_of_a_mixture_is_not_passed_over() {
    let uniform = family("uniform", vec![("min", 0.0001), ("max", 1.0)]);
    let mixture = Distribution::Mixture {
        parts: vec![(standard_normal(), 0.5), (uniform, 0.5)],
    };
    let d = truncated(mixture, 0.0, 2.0);
    // At 0.00025: half of Phi(0.00025) - 1/2 = 9.9735569061445989e-5 (R: the series of the
    // integral), and half of the uniform's 0.00015 / 0.9999.
    let part = 0.5 * 9.973_556_906_144_599e-5 + 0.5 * 0.00015 / 0.9999;
    let whole = 0.5 * 0.477_249_868_051_820_8 + 0.5;
    let got = families::cdf(&d, 0.00025).lower;
    assert!(relative(got, part / whole) < 1e-11, "{got:e}");
}

/// The F distribution far in its upper tail, where w = d1 x / (d1 x + d2) nears 1: the tail and
/// the density are taken at 1 - w, and agree with R's `pf` and `df`.
#[test]
fn the_f_distribution_keeps_its_digits_far_in_the_upper_tail() {
    let f = |d1: f64, d2: f64| family("f", vec![("df1", d1), ("df2", d2)]);
    let heavy = f(19.709_146_212_320_235, 3.554_567_581_940_035_7);
    let upper = families::cdf(&heavy, 1e6).upper;
    assert!(
        relative(upper, 3.919_422_049_075_479e-11) < 1e-13,
        "{upper:e}"
    );
    let density = families::pdf(&heavy, 1e6);
    assert!(
        relative(density, 6.965_920_016_088_323e-17) < 1e-13,
        "{density:e}"
    );
    let usual = f(7.0, 12.0);
    for (x, tail, density) in [
        (3.0, 0.045_675_436_467_533_34, 0.047_392_200_218_599_995),
        (50.0, 6.155_430_991_923_965e-8, 7.054_719_561_964_961e-9),
        (
            1e4,
            1.263_564_301_931_994_4e-21,
            7.579_622_294_790_473_5e-25,
        ),
        (1e8, 1.265_329_031_438_441_3e-45, 7.591_974_012_001_08e-53),
    ] {
        let got = families::cdf(&usual, x);
        assert!(relative(got.upper, tail) < 1e-13, "{x}: {:e}", got.upper);
        let got = families::pdf(&usual, x);
        assert!(relative(got, density) < 1e-13, "{x}: {got:e}");
    }
    // A value so large that d1 x overflows has no upper tail left, and all of the lower.
    let far = families::cdf(&usual, 1e308);
    assert_eq!((far.lower, far.upper), (1.0, 0.0));
}
