//! The hypergeometric distribution (catalogue row 533; release 0.10.0, step 6). Held to exact
//! rational arithmetic within 1e-12 relative for populations up to a few thousand
//! (`tests/reference/hypergeometric.json`, from Python's `fractions`), tails of 1e-200 among
//! them; to R's `dhyper` family around the means of populations up to a thousand million
//! (`tests/reference/r/distributions.json`); to the identities the distribution satisfies; and,
//! through the engine, to its printed forms, its edges and every refusal.

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

fn hypergeometric(successes: f64, failures: f64, draws: f64) -> Distribution {
    Distribution::Family {
        name: "hypergeometric",
        params: vec![
            ("successes", successes),
            ("failures", failures),
            ("draws", draws),
        ],
    }
}

/// A reference number: a JSON number, or text holding a number or one of three words.
fn number(json: &Json) -> f64 {
    if let Some(x) = json.as_f64() {
        return x;
    }
    match json.as_str().expect("a number") {
        "NA" => f64::NAN,
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        other => other.parse().expect("a number"),
    }
}

/// Whether `got` is the reference within `tolerance` relative. Among the smallest numbers a
/// Float holds (below 2.2e-308 they are whole multiples of 4.9e-324, so that 2.5e-323 is five
/// of them and has one digit) the allowance is two of those steps: a tail there is a
/// probability times a sum, and each is rounded to that grid once.
fn agrees(got: f64, want: f64, tolerance: f64) -> bool {
    let step = f64::from_bits(1);
    (got - want).abs() <= tolerance * want.abs() + 2.0 * step
}

#[test]
fn small_populations_agree_with_exact_rational_arithmetic() {
    let text = std::fs::read_to_string(root().join("tests/reference/hypergeometric.json")).unwrap();
    let exact: Json = serde_json::from_str(&text).unwrap();
    let mut compared = 0;
    let mut smallest = f64::INFINITY;
    let mut failures = Vec::new();
    for case in exact["cases"].as_array().unwrap() {
        let (m, n, d) = (
            number(&case["successes"]),
            number(&case["failures"]),
            number(&case["draws"]),
        );
        let dist = hypergeometric(m, n, d);
        for (i, k) in case["k"].as_array().unwrap().iter().enumerate() {
            let k = number(k);
            let tails = families::cdf(&dist, k);
            for (what, got, want) in [
                ("pdf", families::pdf(&dist, k), number(&case["pdf"][i])),
                ("lower tail", tails.lower, number(&case["lower"][i])),
                ("upper tail", tails.upper, number(&case["upper"][i])),
            ] {
                compared += 1;
                if want > 0.0 {
                    smallest = smallest.min(want);
                }
                if !agrees(got, want, 1e-12) {
                    failures.push(format!(
                        "({m}, {n}, {d}): {what} at {k}: {got:e} against {want:e}"
                    ));
                }
            }
        }
    }
    assert_eq!(compared, 708);
    assert!(
        smallest < 1e-200,
        "the far tails are among the cases: {smallest:e}"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Around the means of populations up to a thousand million, and at twelve standard deviations
/// from them, against R, which sums in doubles as the engine does: 1e-10 relative, the tolerance
/// the other count distributions are held to at large parameters. The quantiles are whole
/// numbers and must be the reference's, which is the smallest number at which R's distribution
/// function reaches the probability, found by halving: R's own `qhyper` adds the probabilities
/// up from the lower end and is wrong in the upper tail of a large population (it gives the
/// end of the range, a million, where the answer is 503,179), and its answers are kept in the
/// file beside the reference only to show where.
#[test]
fn large_populations_agree_with_r() {
    let text =
        std::fs::read_to_string(root().join("tests/reference/r/distributions.json")).unwrap();
    let r: Json = serde_json::from_str(&text).unwrap();
    let cases = r["hypergeometric"].as_array().unwrap();
    assert_eq!(cases.len(), 7);
    let mut failures = Vec::new();
    for case in cases {
        let param = |key: &str| number(&case["params"][key]);
        let (m, n, d) = (param("successes"), param("failures"), param("draws"));
        let dist = hypergeometric(m, n, d);
        for (i, x) in case["x"].as_array().unwrap().iter().enumerate() {
            let x = number(x);
            let tails = families::cdf(&dist, x);
            for (what, got, want) in [
                ("pdf", families::pdf(&dist, x), number(&case["pdf"][i])),
                ("lower tail", tails.lower, number(&case["lower"][i])),
                ("upper tail", tails.upper, number(&case["upper"][i])),
            ] {
                if !agrees(got, want, 1e-10) {
                    failures.push(format!(
                        "({m}, {n}, {d}): {what} at {x}: {got:e} against R's {want:e}"
                    ));
                }
            }
        }
        for (i, p) in case["p"].as_array().unwrap().iter().enumerate() {
            let p = number(p);
            let (got, want) = (families::quantile(&dist, p), number(&case["quantile"][i]));
            if got != want {
                failures.push(format!(
                    "({m}, {n}, {d}): quantile at {p:e}: {got} against R's {want}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// What the distribution owes whatever its parameters: its probabilities add to one and have
/// the mean d m / N; exchanging the draws with the successes changes nothing, and counting
/// failures instead of successes mirrors it; the two tails add to one and the distribution
/// function is the running sum of the probabilities.
#[test]
fn the_identities_of_the_distribution() {
    let cases = [
        (10.0, 7.0, 8.0),
        (3.0, 40.0, 12.0),
        (40.0, 3.0, 12.0),
        (250.0, 750.0, 300.0),
        (2000.0, 3000.0, 1.0),
        (17.0, 0.0, 9.0),
        (0.0, 17.0, 9.0),
        (600.0, 400.0, 999.0),
    ];
    for (m, n, d) in cases {
        let dist = hypergeometric(m, n, d);
        let (first, last) = ((d - n).max(0.0), d.min(m));
        let exchanged = hypergeometric(d, m + n - d, m);
        let mirrored = hypergeometric(n, m, d);
        let (mut total, mut mean, mut running) = (0.0, 0.0, 0.0);
        let mut k = first;
        while k <= last {
            let mass = families::pdf(&dist, k);
            total += mass;
            mean += k * mass;
            running += mass;
            let tails = families::cdf(&dist, k);
            assert!(
                (tails.lower + tails.upper - 1.0).abs() <= 4e-16,
                "({m}, {n}, {d}) at {k}"
            );
            assert!(
                (tails.lower - running).abs() <= 1e-13,
                "({m}, {n}, {d}) at {k}: {} against the running sum {running}",
                tails.lower
            );
            let other = families::pdf(&exchanged, k);
            assert!(
                (mass - other).abs() <= 1e-13 * mass,
                "({m}, {n}, {d}) at {k}: {mass:e} and, draws and successes exchanged, {other:e}"
            );
            let reflected = families::pdf(&mirrored, d - k);
            assert!(
                (mass - reflected).abs() <= 1e-13 * mass,
                "({m}, {n}, {d}) at {k}: {mass:e} and, counted as failures, {reflected:e}"
            );
            k += 1.0;
        }
        assert!((total - 1.0).abs() <= 1e-13, "({m}, {n}, {d}): {total}");
        assert!(
            (mean - d * m / (m + n)).abs() <= 1e-12 * (1.0 + mean),
            "({m}, {n}, {d}): mean {mean}"
        );
        // Outside the range, and between whole numbers.
        assert_eq!(families::pdf(&dist, first - 1.0), 0.0);
        assert_eq!(families::pdf(&dist, last + 1.0), 0.0);
        assert_eq!(families::pdf(&dist, first + 0.5), 0.0);
        assert_eq!(families::cdf(&dist, first - 1.0).lower, 0.0);
        assert_eq!(families::cdf(&dist, last).lower, 1.0);
        assert_eq!(families::cdf(&dist, last + 5.0).upper, 0.0);
        if last > first {
            assert_eq!(
                families::cdf(&dist, first + 0.5).lower,
                families::cdf(&dist, first).lower
            );
        }
    }
}

/// The quantile is the smallest whole number whose distribution function reaches p: at every
/// probability the distribution function at the answer reaches it and at the number before it
/// does not. A symmetric distribution, whose distribution function is exactly one half at a
/// whole number, answers that number.
#[test]
fn the_quantile_is_the_smallest_number_that_reaches_the_probability() {
    let ps = [
        1e-300, 1e-12, 0.001, 0.025, 0.1, 0.3, 0.5, 0.7, 0.9, 0.975, 0.999,
    ];
    for (m, n, d) in [
        (10.0, 7.0, 8.0),
        (50.0, 950.0, 100.0),
        (300.0, 700.0, 400.0),
        (1e6, 3e6, 5e4),
        (5e8, 5e8, 1e6),
    ] {
        let dist = hypergeometric(m, n, d);
        let (first, last) = ((d - n).max(0.0), d.min(m));
        assert_eq!(families::quantile(&dist, 0.0), first);
        assert_eq!(families::quantile(&dist, 1.0), last);
        let mut before = first;
        for p in ps {
            let k = families::quantile(&dist, p);
            assert!(k >= before && k <= last && k.fract() == 0.0, "{k} at {p:e}");
            before = k;
            // The allowance the quantile gives a computed tie: about a thousand roundings of
            // the smaller of p and 1 − p.
            let reached = p - 1024.0 * f64::EPSILON * p.min(1.0 - p);
            assert!(
                families::cdf(&dist, k).lower >= reached,
                "({m}, {n}, {d}) at {p:e}: {k}"
            );
            if k > first {
                assert!(
                    families::cdf(&dist, k - 1.0).lower < reached,
                    "({m}, {n}, {d}) at {p:e}: {k} is not the smallest"
                );
            }
        }
    }
    // Three draws from five and five: 0, 1, 2 or 3 successes with probabilities 1/12, 5/12,
    // 5/12 and 1/12, so that the distribution function is one half at 1, exactly.
    let even = hypergeometric(5.0, 5.0, 3.0);
    assert_eq!(families::quantile(&even, 0.5), 1.0);
    assert_eq!(families::quantile(&even, 0.5 + 1e-9), 2.0);
    assert_eq!(families::quantile(&even, 1.0 / 12.0), 0.0);
    // And 999 draws from a thousand and a thousand: one half at 499.
    assert_eq!(
        families::quantile(&hypergeometric(1000.0, 1000.0, 999.0), 0.5),
        499.0
    );
}

#[test]
fn the_printed_forms_the_edges_and_the_draws() {
    let (code, stdout, stderr) = go(concat!(
        "let h = dist.hypergeometric(successes: 10, failures: 7, draws: 8)\n",
        "h\n",
        // The range runs from 1 to 8; values outside it and between whole numbers.
        "[h.pdf(0), h.pdf(1), h.pdf(8), h.pdf(9), h.pdf(4.5), h.pdf(-1)]\n",
        "[h.cdf(0), h.cdf(0.5), h.cdf(1), h.cdf(4.5), h.cdf(8), h.cdf(1e300), h.cdf(-3, upper: true), h.cdf(8, upper: true)]\n",
        "[h.quantile(0), h.quantile(0.5), h.quantile(1)]\n",
        "[h.pdf(NA), h.cdf(NA), h.quantile(NA)]\n",
        // The degenerate urns: nothing drawn, nothing to succeed, nothing to fail, all drawn.
        "let none = dist.hypergeometric(successes: 9, failures: 4, draws: 0)\n",
        "[none.pdf(0), none.cdf(0), none.quantile(0.3)]\n",
        "let never = dist.hypergeometric(successes: 0, failures: 6, draws: 4)\n",
        "[never.pdf(0), never.pdf(1), never.quantile(0.99)]\n",
        "let always = dist.hypergeometric(successes: 6, failures: 0, draws: 4)\n",
        "[always.pdf(4), always.pdf(3), always.quantile(0.01)]\n",
        "let emptied = dist.hypergeometric(successes: 9, failures: 4, draws: 13)\n",
        "[emptied.pdf(9), emptied.cdf(8), emptied.quantile(0.5)]\n",
        "dist.hypergeometric(successes: 0, failures: 0, draws: 0).pdf(0)\n",
        // Draws are whole numbers, the same under a seed.
        "let a = h.draw(6, seed: 11)\n",
        "a\n",
        "equal(a, h.draw(6, seed: 11))\n",
        "frame(k = [1, 3, NA, 8]).add(p = h.pdf(k), below = h.cdf(k))\n",
        // A truncation counts whole numbers; a mixture of counts draws counts.
        "let t = dist.truncated(h, lower: 4, upper: 6)\n",
        "t\n",
        "[t.pdf(3), t.pdf(4), t.cdf(5), t.quantile(0.5), t.cdf(6)]\n",
        "let m = dist.mixture([h, dist.binomial(n: 8, p: 0.2)], probs: [0.5, 0.5])\n",
        "[m.pdf(2), m.cdf(2), m.quantile(0.5)]\n",
        "m.draw(4, seed: 2)\n",
        // A population of a thousand million.
        "let big = dist.hypergeometric(successes: 500000000, failures: 500000000, draws: 1000000)\n",
        "[big.pdf(500000), big.cdf(499000), big.cdf(501000, upper: true), big.quantile(0.975)]\n",
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}{stderr}"));
}

#[test]
fn every_refusal() {
    let mut all = String::new();
    for script in [
        "dist.hypergeometric(successes: 10, failures: 7)\n",
        "dist.hypergeometric(successes: 10, failures: 7, draws: 20)\n",
        "dist.hypergeometric(successes: -1, failures: 7, draws: 3)\n",
        "dist.hypergeometric(successes: 10, failures: -7, draws: 3)\n",
        "dist.hypergeometric(successes: 10, failures: 7, draws: -3)\n",
        "dist.hypergeometric(successes: 10.5, failures: 7, draws: 3)\n",
        "dist.hypergeometric(successes: 10, failures: 7, draws: 2.5)\n",
        "dist.hypergeometric(successes: NA, failures: 7, draws: 3)\n",
        "dist.hypergeometric(successes: 10, failures: 7, draws: 3, replace: true)\n",
        "dist.hypergeometric(successes: 10, failures: 7, draws: 3).quantile(1.2)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}
