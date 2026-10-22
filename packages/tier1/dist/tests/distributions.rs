//! The first four families and the four methods (release 0.3.0, step 7) through the engine. The
//! reference is R's `tests/reference/r/distributions.json` at six printed digits (the kernel is held
//! to the exact decimal reference in `zeff-numeric`); the edges of each support and every message
//! are snapshots; draws are reproducible under a seed, differ between statements, and recover
//! their distribution's moments and shape (the stochastic class of section 13).

use serde_json::Value as Json;
use zeff_core::display::format_float;
use zeff_testkit::{root, run};

mod common;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn without_format_notes(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn printed(expressions: &[String]) -> Vec<String> {
    let script: String = expressions
        .iter()
        .map(|e| format!("print({e})\n"))
        .collect();
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    stdout.lines().map(str::to_string).collect()
}

fn shown(json: &Json) -> String {
    let text = json.as_str().unwrap();
    match text {
        "NA" | "Inf" | "-Inf" => text.to_string(),
        other => format_float(other.parse().unwrap()),
    }
}

/// A number written so that the script reads it back exactly.
fn literal(json: &Json) -> String {
    let x: f64 = json.as_str().unwrap().parse().unwrap();
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{x:.1}")
    } else {
        format!("{x:e}")
    }
}

#[test]
fn every_family_agrees_with_r_at_six_digits() {
    let text =
        std::fs::read_to_string(root().join("tests/reference/r/distributions.json")).unwrap();
    let r: Json = serde_json::from_str(&text).unwrap();
    for family in r["families"].as_array().unwrap() {
        let name = family["family"].as_str().unwrap();
        if ![
            "normal",
            "lognormal",
            "uniform",
            "exponential",
            "t",
            "chi2",
            "f",
            "gamma",
            "beta",
            "binomial",
            "poisson",
            "negbinomial",
            "composite",
        ]
        .contains(&name)
        {
            continue;
        }
        let d = if name == "composite" {
            family["expression"].as_str().unwrap().to_string()
        } else {
            let params: Vec<String> = family["params"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| {
                    // The binomial's `n` is an Int parameter.
                    if k == "n" {
                        format!(
                            "{k}: {}",
                            v.as_str().unwrap().parse::<f64>().unwrap() as i64
                        )
                    } else {
                        format!("{k}: {}", literal(v))
                    }
                })
                .collect();
            format!("dist.{name}({})", params.join(", "))
        };
        let mut expressions = Vec::new();
        let mut expected = Vec::new();
        let xs = family["x"].as_array().unwrap();
        for (i, x) in xs.iter().enumerate() {
            let x = literal(x);
            expressions.push(format!("{d}.pdf({x})"));
            expected.push(shown(&family["pdf"][i]));
            expressions.push(format!("{d}.cdf({x})"));
            expected.push(shown(&family["lower"][i]));
            expressions.push(format!("{d}.cdf({x}, upper: true)"));
            expected.push(shown(&family["upper"][i]));
        }
        for (i, p) in family["p"].as_array().unwrap().iter().enumerate() {
            expressions.push(format!("{d}.quantile({})", literal(p)));
            expected.push(shown(&family["quantile"][i]));
        }
        let got = printed(&expressions);
        let mut mismatches = Vec::new();
        let mut round_trips = Vec::new();
        for ((e, want), got) in expressions.iter().zip(&expected).zip(&got) {
            if got == want {
                continue;
            }
            // A truncation or mixture subtracts near-equal probabilities on both sides, so its
            // last printed digit may differ: within one part in a hundred thousand.
            if name == "composite"
                && let (Ok(g), Ok(w)) = (got.parse::<f64>(), want.parse::<f64>())
                && (g - w).abs() <= 1e-5 * w.abs().max(1e-300)
            {
                continue;
            }
            // R's `qbeta` (and so `qf`) loses digits, or gives up at 0, deep in a tail; a quantile
            // that differs from R's is held instead to the distribution function it inverts: the
            // probability must come back at six digits.
            if let Some((d, p)) = e.split_once(".quantile(") {
                let p = p.trim_end_matches(')');
                round_trips.push((
                    format!("{d}.cdf({got})"),
                    shown(&Json::String(p.to_string())),
                    e.clone(),
                    got.clone(),
                    want.clone(),
                ));
            } else {
                mismatches.push(format!("{e}: Zeff {got}, R {want}"));
            }
        }
        if !round_trips.is_empty() {
            // A discrete quantile is the smallest k with F(k) ≥ p: F(k) ≥ p and F(k − 1) < p.
            let discrete = ["binomial", "poisson", "negbinomial"]
                .iter()
                .any(|f| d.contains(f));
            if discrete {
                let mut trips = Vec::new();
                for (_, p, _, got, _) in &round_trips {
                    trips.push(format!("{d}.cdf({got}) >= {p}"));
                    trips.push(format!("{d}.cdf({got} - 1) < {p}"));
                }
                let back = printed(&trips);
                for (i, (_, p, e, got, want)) in round_trips.iter().enumerate() {
                    if back[2 * i] != "true" || back[2 * i + 1] != "true" {
                        mismatches.push(format!(
                            "{e}: Zeff {got}, R {want}; not the smallest k with F(k) >= {p}"
                        ));
                    }
                }
                continue;
            }
            let trips: Vec<String> = round_trips.iter().map(|r| r.0.clone()).collect();
            let back = printed(&trips);
            for ((trip, p, e, got, want), back) in round_trips.iter().zip(&back) {
                // The quantile fed back carries six printed digits, so the probability returns
                // within a few parts in a million.
                let (b, want_p): (f64, f64) = (
                    back.parse().unwrap_or(f64::NAN),
                    p.parse().unwrap_or(f64::NAN),
                );
                if (b - want_p).abs() > 1e-5 * want_p.abs() {
                    mismatches.push(format!(
                        "{e}: Zeff {got}, R {want}; {trip} gives {back}, not {p}"
                    ));
                }
            }
        }
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }
}

#[test]
fn the_edges_of_the_supports_and_the_refusals() {
    let (code, stdout, stderr) = go(
        "dist.lognormal().pdf(0)\ndist.lognormal().cdf(0)\ndist.lognormal().cdf(0, upper: true)\ndist.uniform().pdf(2)\ndist.uniform().cdf(-1)\ndist.exponential().pdf(-1)\ndist.exponential().cdf(-1, upper: true)\ndist.normal().quantile(0)\ndist.normal().quantile(1)\ndist.exponential().quantile(0)\ndist.uniform().quantile(1)\ndist.normal().cdf(NA)\ndist.normal().pdf(1e300)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0\n0\n1\n0\n0\n0\n1\n-Inf\nInf\n0\n1\nNA\n0\n");
    let mut all = String::new();
    for script in [
        "dist.normal().quantile(1.5)\n",
        "dist.normal().quantile(-0.1)\n",
        "dist.uniform(min: 3, max: 1)\n",
        "dist.lognormal(sdlog: -1)\n",
        "dist.exponential(rate: 0)\n",
        "dist.normal().draw(-1)\n",
        "dist.normal().draw(2.5)\n",
        "dist.normal().draw(3, seed: 1.5)\n",
        "dist.normal().cdf(\"x\")\n",
        "dist.t(df: 0)\n",
        "dist.f(df1: 3, df2: -1)\n",
        "dist.gamma(shape: 2, rate: 0)\n",
        "dist.beta(a: 1)\n",
        "dist.binomial(n: 10.5, p: 0.3)\n",
        "dist.binomial(n: 10, p: 1.5)\n",
        "dist.binomial(n: -2, p: 0.3)\n",
        "dist.poisson(lambda: -1)\n",
        "dist.negbinomial(size: 3, p: 0.4, mu: 2)\n",
        "dist.negbinomial(size: 3)\n",
        "dist.negbinomial(size: 3, p: 0)\n",
        "dist.truncated(dist.normal(), lower: 2, upper: 1)\n",
        "dist.truncated(dist.uniform(), lower: 5, upper: 6)\n",
        "dist.truncated(dist.normal(), lower: sqrt(-1))\n",
        "dist.mixture([dist.normal()], probs: [0.3, 0.7])\n",
        "dist.mixture([dist.normal(), dist.normal(mean: 3)], probs: [0.3, 0.6])\n",
        "dist.mixture([dist.normal(), dist.normal(mean: 3)])\n",
        "dist.mixture([], probs: [])\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}

#[test]
fn the_families_on_the_incomplete_functions_at_their_edges() {
    let (code, stdout, stderr) = go(
        "dist.chi2(df: 5).pdf(-1)\ndist.chi2(df: 5).cdf(0)\ndist.f(df1: 2, df2: 10).pdf(0)\ndist.f(df1: 1, df2: 10).pdf(0)\ndist.gamma(shape: 1).pdf(0)\ndist.beta(a: 2, b: 3).pdf(1.5)\ndist.beta(a: 2, b: 3).cdf(1)\ndist.beta(a: 0.5, b: 0.5).pdf(0)\ndist.t(df: 3).quantile(0.5)\ndist.t(df: 3).quantile(0)\ndist.t(df: 3).cdf(-30)\ndist.t(df: 1000000).cdf(5, upper: true)\ndist.t(df: 24).quantile(0.975)\ndist.chi2(df: 5).quantile(0.95)\ndist.f(df1: 3, df2: 20).quantile(0.95)\ndist.gamma(shape: 9).quantile(0.99)\ndist.beta(a: 2, b: 3).quantile(0.9)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "0\n0\n1\nInf\n1\n0\n1\nInf\n0\n-Inf\n4.06764e-05\n2.867e-07\n2.0639\n11.0705\n3.09839\n17.4027\n0.679539\n"
    );
}

#[test]
fn the_discrete_families_at_their_edges_and_their_int_draws() {
    let (code, stdout, stderr) = go(
        "let b = dist.binomial(n: 10, p: 0.3)\nb.pdf(2.5)\nb.pdf(11)\nb.cdf(-1)\nb.cdf(10)\nb.cdf(3.7)\nb.quantile(0)\nb.quantile(1)\nb.quantile(0.5)\ndist.poisson(lambda: 2).quantile(1)\ndist.poisson(lambda: 2).pdf(0)\ndist.negbinomial(size: 3, p: 1).pdf(0)\ndist.negbinomial(size: 3, p: 1).quantile(1)\ndist.negbinomial(mu: 4.5, size: 3).pdf(2)\ndist.negbinomial(size: 3, p: 0.4).pdf(2)\nb.draw(4, seed: 2)\ndist.poisson(lambda: 4).draw(3, seed: 2)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "0\n0\n0\n1\n0.649611\n0\n10\n3\nInf\n0.135335\n1\n0\n0.13824\n0.13824\nColumn<Int> · 4 values\n3\n6\n1\n6\nColumn<Int> · 3 values\n3\n8\n1\n"
    );
}

#[test]
fn truncations_and_mixtures_print_compute_and_draw() {
    let (code, stdout, stderr) = go(
        "let tn = dist.truncated(dist.normal(), lower: -1, upper: 2)\ntn\ntn.pdf(-2)\ntn.cdf(-1)\ntn.cdf(2)\ntn.quantile(0)\ntn.quantile(1)\ntn.cdf(0.5)\ntn.cdf(0.5, upper: true)\nlet tp = dist.truncated(dist.poisson(lambda: 3), lower: 1, upper: 6)\ntp\ntp.pdf(0)\ntp.pdf(1)\ntp.quantile(0.5)\ntp.draw(4, seed: 3)\nlet m = dist.mixture([dist.normal(), dist.normal(mean: 3, sd: 0.5)], probs: [0.3, 0.7])\nm\nm.pdf(3)\nm.cdf(1.5)\nm.quantile(0.3)\nm.draw(3, seed: 5)\ndist.mixture([dist.poisson(lambda: 1), dist.poisson(lambda: 10)], probs: [0.5, 0.5]).draw(3, seed: 5)\ndist.truncated(dist.exponential(rate: 2), upper: 1).quantile(0.5)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("composite", stdout);
}

#[test]
fn columns_apply_row_by_row() {
    let (code, stdout, stderr) = go(
        "let f = frame(z = [-1.96, 0, 1.96, NA], p = [0.025, 0.5, 0.975, NA])\nlet nd = dist.normal()\nnd.cdf(f[\"z\"])\nnd.cdf(f[\"z\"], upper: true)\nnd.pdf(f[\"z\"])\nnd.quantile(f[\"p\"])\nf.add(q = nd.quantile(p), density = nd.pdf(z))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "columns",
        format!("{stdout}---\n{}", without_format_notes(&stderr))
    );
}

#[test]
fn draws_are_reproducible_under_a_seed_and_differ_between_statements() {
    let (code, stdout, stderr) = go(
        "dist.normal().draw(3, seed: 1)\ndist.normal().draw(3, seed: 1)\ndist.normal().draw(3, seed: 2)\ndist.normal().draw(0)\ndist.uniform().draw(1)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let blocks: Vec<&str> = stdout
        .split("Column<Float>")
        .filter(|b| !b.is_empty())
        .collect();
    assert_eq!(blocks.len(), 5, "{stdout}");
    assert_eq!(blocks[0], blocks[1], "the same seed gives the same draws");
    assert_ne!(blocks[0], blocks[2], "another seed gives other draws");
    assert!(blocks[3].starts_with(" · 0 values\n"), "{}", blocks[3]);
    assert!(blocks[4].starts_with(" · 1 value\n"), "{}", blocks[4]);
    // Without a seed, one is chosen at the first draw and noted once; the second statement draws
    // other numbers from the same seed.
    let (code, stdout, stderr) = go("dist.normal().draw(2)\ndist.normal().draw(2)\n");
    assert_eq!(code, 0, "{stderr}");
    let notes: Vec<&str> = stderr
        .lines()
        .filter(|l| l.contains("was chosen for this run"))
        .collect();
    assert_eq!(notes.len(), 1, "{stderr}");
    assert!(
        notes[0].starts_with("note: no seed was set, so ")
            && notes[0].ends_with("at the top of the script repeats it (line 1)"),
        "{}",
        notes[0]
    );
    let blocks: Vec<&str> = stdout
        .split("Column<Float>")
        .filter(|b| !b.is_empty())
        .collect();
    assert_ne!(blocks[0], blocks[1]);
}

/// The stochastic class (Engine and Project Specification, section 13): 100,000 draws recover the
/// mean and variance within five standard errors, and the empirical distribution lies within the
/// Kolmogorov–Smirnov band at the 0.1 per cent level (1.95 / sqrt(n)).
#[test]
fn draws_recover_their_distribution() {
    use zeff_dist::families;
    use zeff_numeric::random::Stream;
    let n = 100_000usize;
    let all = common::recovery_cases();
    for (seed, (name, d, mean, variance)) in all.iter().enumerate() {
        let mut stream = Stream::new(seed as u64 + 1, 1, 0);
        let mut draws: Vec<f64> = (0..n).map(|_| families::draw(d, &mut stream)).collect();
        let sample_mean = draws.iter().sum::<f64>() / n as f64;
        let se = (variance / n as f64).sqrt();
        assert!(
            (sample_mean - mean).abs() < 5.0 * se,
            "{name}: mean {sample_mean} against {mean}"
        );
        let sample_variance =
            draws.iter().map(|x| (x - sample_mean).powi(2)).sum::<f64>() / (n as f64 - 1.0);
        // The variance's own standard error, from the fourth moment, is at most a few per cent here.
        assert!(
            (sample_variance - variance).abs() < 0.05 * variance,
            "{name}: variance {sample_variance} against {variance}"
        );
        draws.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut largest = 0.0f64;
        if families::is_discrete(d) {
            // At each distinct value: the empirical share at or below it against F.
            let mut i = 0;
            while i < n {
                let x = draws[i];
                let mut j = i;
                while j < n && draws[j] == x {
                    j += 1;
                }
                let f = families::cdf(d, x).lower;
                largest = largest.max((f - j as f64 / n as f64).abs());
                i = j;
            }
        } else {
            for (i, x) in draws.iter().enumerate() {
                let f = families::cdf(d, *x).lower;
                largest = largest
                    .max((f - i as f64 / n as f64).abs())
                    .max(((i + 1) as f64 / n as f64 - f).abs());
            }
        }
        assert!(
            largest < 1.95 / (n as f64).sqrt(),
            "{name}: Kolmogorov–Smirnov distance {largest}"
        );
    }
}

#[test]
fn a_quantile_where_the_distribution_function_is_na_answers_instead_of_searching() {
    // Found by the nightly fuzz target: a negative binomial with a mean beyond the largest Float
    // sent the bracket search round a loop that never moved (size 13,827, p 1.2e-321); another
    // (size 4.3e232, p 2.3e-322) took eight seconds a draw, a thousand evaluations of a function
    // that is NA; a Poisson near 1e300 needed a thousand doublings. Now the first NA ends the
    // search with NA, and a draw beyond the Ints is NA too.
    let (code, out, stderr) = go(
        "let a = dist.negbinomial(size: 13827.029, p: 1.2e-321)\nprint(a.quantile(0.9))\nprint(a.draw(2, seed: 1))\nprint(dist.negbinomial(size: 4.291379375431708e232, p: 2.3e-322).draw(2, seed: 0))\nprint(dist.poisson(lambda: 1e300).draw(2, seed: 1))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let na_draws = "Column<Int> · 2 values\nNA\nNA\n";
    assert_eq!(out, format!("NA\n{na_draws}{na_draws}{na_draws}"));
}

#[test]
fn the_discrete_families_hold_r_at_large_parameters() {
    // Found in the review before the 0.3.0 tag: the point x was floored with a fuzz of 1e-7 of x,
    // so P(X ≤ 1e9) was computed as P(X ≤ 1e9 + 100), wrong by 2.5e-3. R 4.6.0's values, from
    // `ppois`, `pnbinom`, `pbinom` and their quantiles, written with fifteen digits.
    use zeff_core::value::Distribution;
    use zeff_dist::families;
    let fam = |name: &'static str, params: Vec<(&'static str, f64)>| Distribution::Family {
        name,
        params,
    };
    let poisson = |lambda: f64| fam("poisson", vec![("lambda", lambda)]);
    let negbinomial = |size: f64| fam("negbinomial", vec![("size", size), ("p", 0.5)]);
    let binomial = |n: f64| fam("binomial", vec![("n", n), ("p", 0.5)]);
    // The gamma path holds 1e-10 here; the incomplete beta behind the binomial loses digits at
    // n = 1e12 (about 3e-10 near its centre, a known issue), so those two rows allow 1e-9.
    let close_to = |got: f64, r: f64, tolerance: f64, what: &str| {
        assert!(
            ((got - r) / r).abs() < tolerance,
            "{what}: {got} against R's {r}"
        );
    };
    let close = |got: f64, r: f64, what: &str| close_to(got, r, 1e-10, what);
    close(
        families::cdf(&poisson(1e7), 1e7).lower,
        0.500084104416326,
        "ppois(1e7, 1e7)",
    );
    close(
        families::cdf(&poisson(1e9), 1e9).lower,
        0.5000084104417389,
        "ppois(1e9, 1e9)",
    );
    close(
        families::cdf(&poisson(1e9), 1e9).upper,
        0.499991589558261,
        "ppois(1e9, 1e9, upper)",
    );
    close(
        families::cdf(&poisson(1e9), 1e9 + 3e4).upper,
        0.1713866995188297,
        "ppois(1e9 + 3e4, 1e9, upper)",
    );
    close(
        families::cdf(&poisson(1e10), 1e10).lower,
        0.5000026596152026,
        "ppois(1e10, 1e10)",
    );
    close(
        families::cdf(&negbinomial(1e9), 1e9).lower,
        0.5000089206205796,
        "pnbinom(1e9, 1e9, 0.5)",
    );
    close(
        families::cdf(&negbinomial(1e9), 1e9 - 5e4).lower,
        0.1317780292251666,
        "pnbinom(1e9 - 5e4, 1e9, 0.5)",
    );
    close_to(
        families::cdf(&binomial(1e12), 5e11).lower,
        0.5000003989422803,
        1e-9,
        "pbinom(5e11, 1e12, 0.5)",
    );
    close_to(
        families::cdf(&binomial(1e12), 5e11 + 1e6).upper,
        0.02275007795723969,
        1e-9,
        "pbinom(5e11 + 1e6, 1e12, 0.5, upper)",
    );
    assert_eq!(
        families::quantile(&poisson(1e9), 0.5),
        1e9,
        "qpois(0.5, 1e9)"
    );
    assert_eq!(
        families::quantile(&negbinomial(1e9), 0.9),
        1_000_057_313.0,
        "qnbinom(0.9, 1e9, 0.5)"
    );
    assert_eq!(
        families::quantile(&binomial(1e12), 0.1),
        499_999_359_224.0,
        "qbinom(0.1, 1e12, 0.5)"
    );
    // Where the gamma series cannot converge the answer is NA, not a wrong number.
    assert!(families::cdf(&poisson(1e12), 1e12).lower.is_nan());
    assert!(families::quantile(&poisson(1e12), 0.5).is_nan());
}
