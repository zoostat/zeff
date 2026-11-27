//! The Weibull and the logistic distributions (catalogue rows 531 and 532; release 0.10.0, step
//! 5). Against R's `dweibull` and `dlogis` families within 1e-12 relative, at parameters from the
//! everyday to 1e7 and 1e12 (`tests/reference/r/distributions.json`); the two places where R is
//! the one in error, held to the value worked out directly; the properties a distribution owes
//! (the quantile undoes the distribution function, a change of scale, symmetry); and, through
//! the engine, the printed forms, the edges of the supports and every refusal.

use serde_json::Value as Json;
use zeff_core::value::Distribution;
use zeff_dist::families;
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn weibull(shape: f64, scale: f64) -> Distribution {
    Distribution::Family {
        name: "weibull",
        params: vec![("shape", shape), ("scale", scale)],
    }
}

fn logistic(location: f64, scale: f64) -> Distribution {
    Distribution::Family {
        name: "logistic",
        params: vec![("location", location), ("scale", scale)],
    }
}

/// R's number, written as text with 15 significant digits or as one of three words.
fn number(json: &Json) -> f64 {
    match json.as_str().expect("a number written as text") {
        "NA" => f64::NAN,
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        other => other.parse().expect("a number"),
    }
}

/// Whether `got` is R's value within the tolerance of the distribution class, 1e-12 relative.
/// Nought and the infinities must be met exactly.
fn agrees(got: f64, r: f64) -> bool {
    if r == 0.0 || r.is_infinite() {
        got == r
    } else {
        (got - r).abs() <= 1e-12 * r.abs()
    }
}

#[test]
fn both_families_agree_with_r_within_the_distribution_class() {
    let text =
        std::fs::read_to_string(root().join("tests/reference/r/distributions.json")).unwrap();
    let r: Json = serde_json::from_str(&text).unwrap();
    let mut compared = 0;
    let mut sets = 0;
    let mut failures = Vec::new();
    for family in r["families"].as_array().unwrap() {
        let name = family["family"].as_str().unwrap();
        let param = |key: &str| number(&family["params"][key]);
        let d = match name {
            "weibull" => weibull(param("shape"), param("scale")),
            "logistic" => logistic(param("location"), param("scale")),
            _ => continue,
        };
        sets += 1;
        for (i, x) in family["x"].as_array().unwrap().iter().enumerate() {
            let x = number(x);
            let tails = families::cdf(&d, x);
            for (what, got, want) in [
                ("pdf", families::pdf(&d, x), number(&family["pdf"][i])),
                ("cdf", tails.lower, number(&family["lower"][i])),
                ("upper tail", tails.upper, number(&family["upper"][i])),
            ] {
                compared += 1;
                if !agrees(got, want) {
                    failures.push(format!(
                        "{name} {}: {what} at {x}: {got:e} against R's {want:e}",
                        family["params"]
                    ));
                }
            }
        }
        for (i, p) in family["p"].as_array().unwrap().iter().enumerate() {
            let p = number(p);
            let (got, want) = (families::quantile(&d, p), number(&family["quantile"][i]));
            compared += 1;
            if !agrees(got, want) {
                failures.push(format!(
                    "{name} {}: quantile at {p:e}: {got:e} against R's {want:e}",
                    family["params"]
                ));
            }
        }
    }
    assert_eq!(sets, 10, "five parameter sets for each family");
    assert_eq!(compared, 10 * (21 * 3 + 15));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Far beyond the scale at a very large shape, R's `dweibull` gives NaN (an overflowed power
/// times an underflowed exponential); the density is nought. And at the origin the density is
/// unbounded, 1 / scale or nought as the shape is below, at or above one.
#[test]
fn the_weibull_density_where_r_gives_nan_and_at_the_origin() {
    assert_eq!(families::pdf(&weibull(1e7, 1.0), 1.5), 0.0);
    assert_eq!(families::pdf(&weibull(1e7, 1e6), 2e6), 0.0);
    assert_eq!(families::pdf(&weibull(3.0, 1.0), 1e300), 0.0);
    assert_eq!(families::pdf(&weibull(0.5, 3.0), 0.0), f64::INFINITY);
    assert_eq!(families::pdf(&weibull(1.0, 3.0), 0.0), 1.0 / 3.0);
    assert_eq!(families::pdf(&weibull(2.0, 3.0), 0.0), 0.0);
    assert_eq!(families::pdf(&weibull(2.0, 3.0), -1.0), 0.0);
    // A shape of one is the exponential with rate 1 / scale, to the last bit of each tail.
    let exponential = Distribution::Family {
        name: "exponential",
        params: vec![("rate", 0.25)],
    };
    for x in [1e-10, 0.3, 1.0, 7.5, 40.0, 3000.0] {
        let (w, e) = (
            families::cdf(&weibull(1.0, 4.0), x),
            families::cdf(&exponential, x),
        );
        assert_eq!((w.lower, w.upper), (e.lower, e.upper), "at {x}");
    }
}

/// Close to one half the log-odds are close to nought, and R's `qlogis` keeps four digits at
/// 0.5 + 1e-12. For a step δ that a double holds exactly, the quantile at 1/2 + δ is
/// 2 atanh(2δ) = 4δ (1 + (2δ)²/3 + ...), which for a small δ is 4δ to the last digit.
#[test]
fn the_logistic_quantile_keeps_its_digits_near_one_half() {
    let d = logistic(0.0, 1.0);
    for exponent in [-45, -40, -30, -20] {
        let delta = 2f64.powi(exponent);
        let exact = 4.0 * delta * (1.0 + 4.0 * delta * delta / 3.0);
        for (p, want) in [(0.5 + delta, exact), (0.5 - delta, -exact)] {
            let got = families::quantile(&d, p);
            assert!(
                (got - want).abs() <= 4e-16 * want.abs(),
                "at 1/2 + 2^{exponent}: {got:e} against {want:e}"
            );
        }
    }
    assert_eq!(families::quantile(&d, 0.5), 0.0);
    assert_eq!(families::quantile(&d, 0.0), f64::NEG_INFINITY);
    assert_eq!(families::quantile(&d, 1.0), f64::INFINITY);
    // With a centre and a scale: the centre at one half, and the scale times the log-odds.
    let wide = logistic(100.0, 15.0);
    assert_eq!(families::quantile(&wide, 0.5), 100.0);
    let quartile = families::quantile(&wide, 0.75);
    assert!((quartile - (100.0 + 15.0 * 3f64.ln())).abs() <= 1e-13 * quartile);
}

/// More than 709 scales below the centre R's `plogis` gives nought; the probability is e^z to
/// every digit a double so small can hold, and nought only where e^z itself is.
#[test]
fn the_logistic_tails_far_out_are_computed_directly() {
    let d = logistic(0.0, 1.0);
    for z in [-40.0, -300.0, -700.0, -720.0, -740.0] {
        let tails = families::cdf(&d, z);
        let want = zeff_numeric::maths::exp(z);
        assert!(tails.lower > 0.0, "at {z}");
        assert!(
            (tails.lower - want).abs() <= 1e-15 * want + f64::from_bits(1),
            "at {z}: {:e} against {want:e}",
            tails.lower
        );
        assert_eq!(tails.upper, 1.0, "at {z}");
        // The other side is the mirror image, bit for bit.
        let mirrored = families::cdf(&d, -z);
        assert_eq!(
            (mirrored.upper, mirrored.lower),
            (tails.lower, tails.upper),
            "at {z}"
        );
    }
    assert_eq!(families::cdf(&d, -800.0).lower, 0.0);
    assert_eq!(families::cdf(&d, 800.0).upper, 0.0);
    assert_eq!(families::pdf(&d, -800.0), 0.0);
}

/// What every continuous distribution owes: the two tails sum to one, the distribution function
/// rises, the quantile undoes it, and the density is the slope of the distribution function.
#[test]
fn the_quantile_undoes_the_distribution_function_and_the_density_is_its_slope() {
    let ps = [
        1e-300,
        1e-20,
        1e-9,
        0.001,
        0.2,
        0.37,
        0.5,
        0.63,
        0.9,
        0.999,
        1.0 - 1e-9,
    ];
    let cases = [
        weibull(0.3, 1.0),
        weibull(1.0, 4.0),
        weibull(1.5, 2.0),
        weibull(12.0, 0.01),
        weibull(700.0, 1e7),
        logistic(0.0, 1.0),
        logistic(-2.5, 0.01),
        logistic(1e7, 1e3),
    ];
    for d in &cases {
        let mut last = f64::NEG_INFINITY;
        for &p in &ps {
            let x = families::quantile(d, p);
            assert!(x.is_finite() && x >= last, "{d:?} at {p:e}: {x:e}");
            last = x;
            // A Weibull with a shape of 0.3 has its quantile at 1e-300 near 1e-1000, which no
            // double holds: it is nought, rightly, and nothing comes back from nought.
            if matches!(
                d,
                Distribution::Family {
                    name: "weibull",
                    ..
                }
            ) && x < f64::MIN_POSITIVE
            {
                assert_eq!(x, 0.0);
                continue;
            }
            let tails = families::cdf(d, x);
            assert!(
                (tails.lower + tails.upper - 1.0).abs() <= 4e-16,
                "{d:?} at {x:e}"
            );
            // Back to p on the smaller tail, where a relative error can be read.
            let (back, want) = if p <= 0.5 {
                (tails.lower, p)
            } else {
                (tails.upper, 1.0 - p)
            };
            assert!(
                (back - want).abs() <= 1e-9 * want,
                "{d:?}: p {p:e} came back as {back:e}"
            );
        }
        // The density against a central difference of the distribution function, in the body.
        for &p in &[0.2, 0.5, 0.9] {
            let x = families::quantile(d, p);
            let h = (families::quantile(d, p + 1e-6) - families::quantile(d, p - 1e-6)) / 2.0;
            let slope = (families::cdf(d, x + h).lower - families::cdf(d, x - h).lower) / (2.0 * h);
            let density = families::pdf(d, x);
            assert!(
                (slope - density).abs() <= 1e-6 * density,
                "{d:?} at p {p}: slope {slope:e}, density {density:e}"
            );
        }
    }
}

/// A change of scale: a Weibull variable divided by its scale has a scale of one, and a logistic
/// variable less its centre and divided by its scale is the standard logistic.
#[test]
fn a_change_of_scale_scales_the_distribution() {
    for (shape, scale) in [(0.5, 8.0), (1.5, 2.0), (4.0, 0.125)] {
        let (scaled, unit) = (weibull(shape, scale), weibull(shape, 1.0));
        // The scales are powers of two, so that the division is exact and the bits agree.
        for x in [0.03, 0.4, 1.0, 2.7, 9.0] {
            let (a, b) = (families::cdf(&scaled, x * scale), families::cdf(&unit, x));
            assert_eq!(
                (a.lower, a.upper),
                (b.lower, b.upper),
                "{shape}, {scale}, {x}"
            );
            assert_eq!(
                families::pdf(&scaled, x * scale) * scale,
                families::pdf(&unit, x),
                "{shape}, {scale}, {x}"
            );
        }
        for p in [0.01, 0.5, 0.99] {
            assert_eq!(
                families::quantile(&scaled, p),
                scale * families::quantile(&unit, p)
            );
        }
    }
    let (moved, standard) = (logistic(3.0, 4.0), logistic(0.0, 1.0));
    for z in [-9.0, -1.5, 0.0, 0.25, 6.0] {
        let (a, b) = (
            families::cdf(&moved, 3.0 + 4.0 * z),
            families::cdf(&standard, z),
        );
        assert_eq!((a.lower, a.upper), (b.lower, b.upper), "at {z}");
        assert_eq!(
            families::pdf(&moved, 3.0 + 4.0 * z) * 4.0,
            families::pdf(&standard, z)
        );
    }
}

#[test]
fn the_printed_forms_the_edges_and_the_draws() {
    let (code, stdout, stderr) = go(concat!(
        "dist.weibull(shape: 1.5)\n",
        "dist.weibull(shape: 2, scale: 0.5)\n",
        "dist.logistic()\n",
        "dist.logistic(location: 100, scale: 15)\n",
        // The edges of the Weibull's support, and its quantile at the ends.
        "let w = dist.weibull(shape: 1.5, scale: 2)\n",
        "[w.pdf(-1), w.cdf(-1), w.cdf(-1, upper: true), w.pdf(0), w.cdf(0), w.quantile(0), w.quantile(1)]\n",
        "[dist.weibull(shape: 0.5).pdf(0), dist.weibull(shape: 1, scale: 4).pdf(0), w.cdf(Inf), w.pdf(Inf)]\n",
        // The logistic at the ends.
        "let d = dist.logistic()\n",
        "[d.quantile(0), d.quantile(0.5), d.quantile(1), d.cdf(-Inf), d.cdf(Inf), d.pdf(Inf)]\n",
        // NA goes through as NA.
        "[w.pdf(NA), w.cdf(NA), w.quantile(NA), d.pdf(NA), d.quantile(NA)]\n",
        // Draws are Floats, the same under a seed, inside the support.
        "let a = w.draw(4, seed: 7)\n",
        "let b = w.draw(4, seed: 7)\n",
        "a\n",
        "equal(a, b)\n",
        "d.draw(3, seed: 7)\n",
        // A column goes through row by row.
        "frame(x = [0.5, 1, NA, 4]).add(f = w.cdf(x), g = d.pdf(x))\n",
        // Inside a truncation and a mixture.
        "let t = dist.truncated(w, lower: 0.5, upper: 4)\n",
        "t\n",
        "[t.pdf(0.4), t.cdf(0.5), t.cdf(4), t.quantile(0.5), t.cdf(1)]\n",
        "let m = dist.mixture([d, dist.weibull(shape: 2)], probs: [0.4, 0.6])\n",
        "m\n",
        "[m.pdf(0.5), m.cdf(0.5), m.quantile(0.5)]\n",
        "type_of(w)\n",
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}{stderr}"));
}

#[test]
fn every_refusal() {
    let mut all = String::new();
    for script in [
        "dist.weibull()\n",
        "dist.weibull(shape: 0)\n",
        "dist.weibull(shape: -2)\n",
        "dist.weibull(shape: NA)\n",
        "dist.weibull(shape: 2, scale: 0)\n",
        "dist.weibull(shape: 2, scale: Inf)\n",
        "dist.weibull(shape: 2, rate: 0.5)\n",
        "dist.weibull(shape: \"two\")\n",
        "dist.logistic(scale: 0)\n",
        "dist.logistic(scale: -1)\n",
        "dist.logistic(location: Inf)\n",
        "dist.logistic(location: NA)\n",
        "dist.logistic(mean: 0, sd: 1)\n",
        "dist.weibull(shape: 2).quantile(1.5)\n",
        "dist.logistic().quantile(-0.1)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}
