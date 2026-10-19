//! The first four families and the four methods (release 0.3.0, step 7) through the engine. The
//! reference is R's `tests/reference/r/distributions.json` at six printed digits (the kernel is held
//! to the exact decimal reference in `zeff-numeric`); the edges of each support and every message
//! are snapshots; draws are reproducible under a seed, differ between statements, and recover
//! their distribution's moments and shape (the stochastic class of section 13).

use serde_json::Value as Json;
use zeff_core::display::format_float;
use zeff_testkit::{root, run};

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
        if !["normal", "lognormal", "uniform", "exponential"].contains(&name) {
            continue;
        }
        let params: Vec<String> = family["params"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| format!("{k}: {}", literal(v)))
            .collect();
        let d = format!("dist.{name}({})", params.join(", "));
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
        for ((e, want), got) in expressions.iter().zip(&expected).zip(&got) {
            assert_eq!(got, want, "{e}");
        }
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
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
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
    use zeff_core::value::Distribution;
    use zeff_dist::families;
    use zeff_numeric::random::Stream;
    let n = 100_000usize;
    // A family, its parameters, its mean and its variance.
    type Case = (&'static str, Vec<(&'static str, f64)>, f64, f64);
    let cases: [Case; 4] = [
        ("normal", vec![("mean", 100.0), ("sd", 15.0)], 100.0, 225.0),
        (
            "lognormal",
            vec![("meanlog", 0.0), ("sdlog", 0.5)],
            (0.125f64).exp(),
            ((0.25f64).exp() - 1.0) * (0.25f64).exp(),
        ),
        ("uniform", vec![("min", 2.0), ("max", 5.0)], 3.5, 0.75),
        ("exponential", vec![("rate", 0.1)], 10.0, 100.0),
    ];
    for (seed, (name, params, mean, variance)) in cases.iter().enumerate() {
        let d = Distribution::Family {
            name,
            params: params.clone(),
        };
        let mut stream = Stream::new(seed as u64 + 1, 1, 0);
        let mut draws: Vec<f64> = (0..n)
            .map(|_| families::draw(&d, stream.uniform()))
            .collect();
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
        for (i, x) in draws.iter().enumerate() {
            let f = families::cdf(&d, *x).lower;
            largest = largest
                .max((f - i as f64 / n as f64).abs())
                .max(((i + 1) as f64 / n as f64 - f).abs());
        }
        assert!(
            largest < 1.95 / (n as f64).sqrt(),
            "{name}: Kolmogorov–Smirnov distance {largest}"
        );
    }
}
