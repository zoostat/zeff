//! The reducers of step 17 (Language Reference, sections 3.7, 4.1, 6 and 10.2): `variance`,
//! `median`, `quantile`, `min`, `max`, `sum`, `n`, `n_distinct`, and `by:` and `weights:` on every
//! statistic, through the engine. The references are R's `tests/reference/r/descriptive.json`
//! (the nine quantile types, the weighted statistics) and Python's exact `descriptive.json`.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_core::display::format_float;
use zeff_testkit::{root, run, run_with};

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

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn reference(path: &str) -> Json {
    let text = std::fs::read_to_string(root().join(path)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(json: &Json) -> f64 {
    match json.as_str().unwrap() {
        "NA" => f64::NAN,
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        other => other.parse().unwrap(),
    }
}

/// A printed Float compared with a reference at the six digits printed.
fn assert_printed(what: &str, printed: &str, expected: f64) {
    let want = format_float(expected);
    if printed == want {
        return;
    }
    let got: f64 = printed.parse().unwrap_or(f64::NAN);
    assert!(
        (got - expected).abs() <= 2e-5 * expected.abs().max(1e-300),
        "{what}: printed {printed}, reference {expected}"
    );
}

// ----- the references -----

#[test]
fn the_statistics_agree_with_r_on_the_large_file() {
    let r = reference("tests/reference/r/descriptive.json");
    let columns = &r["datasets"]["wages_large"]["columns"];
    let mut script = String::from("let d = import(\"data/wages_large.csv\")\n");
    let mut expected: Vec<(String, f64)> = Vec::new();
    for var in ["wage", "hours", "educ"] {
        let stats = &columns[var];
        for stat in ["mean", "sd", "variance", "median", "min", "max", "sum"] {
            script.push_str(&format!("d.{stat}({var})\n"));
            expected.push((format!("{stat}({var})"), number(&stats[stat])));
        }
        let probs: Vec<f64> = stats["quantiles"]["probs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        for kind in 1..=9 {
            let values = stats["quantiles"][format!("type{kind}")]
                .as_array()
                .unwrap();
            for (p, want) in probs.iter().zip(values) {
                script.push_str(&format!("d.quantile({var}, p: {p:?}, type: {kind})\n"));
                expected.push((format!("quantile({var}, {p}, type {kind})"), number(want)));
            }
        }
    }
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), expected.len());
    for (line, (what, want)) in lines.iter().zip(&expected) {
        assert_printed(what, line, *want);
    }
    // Counts: n_distinct and the rows.
    let (code, stdout, stderr) = go(
        "let d = import(\"data/wages_large.csv\")\nn_distinct(d[\"region\"])\nn_distinct(d[\"hours\"])\nd.add(k = n(), r = n_distinct(region))[\"k\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.starts_with("5\n"), "{stdout}");
    assert!(
        stdout.contains("\nColumn<Int> · 1,200 values\n1200\n"),
        "{stdout}"
    );
}

#[test]
fn the_weighted_statistics_agree_with_r() {
    let r = reference("tests/reference/r/descriptive.json");
    let weighted = &r["datasets"]["wages_large"]["weighted"];
    let mut script = String::from("let d = import(\"data/wages_large.csv\")\n");
    let mut expected: Vec<(String, f64)> = Vec::new();
    for var in ["wage", "hours"] {
        let w = &weighted[var];
        for (call, key) in [
            ("mean", "mean"),
            ("variance", "variance_analytic"),
            ("sd", "sd_analytic"),
            ("sum", "sum_analytic"),
        ] {
            script.push_str(&format!(
                "d.{call}({var}, weights: educ, weight_type: analytic)\n"
            ));
            expected.push((format!("analytic {call}({var})"), number(&w[key])));
        }
        for (call, key) in [
            ("mean", "mean"),
            ("variance", "variance_frequency"),
            ("sd", "sd_frequency"),
            ("sum", "sum_frequency"),
            ("median", "median"),
        ] {
            script.push_str(&format!(
                "d.{call}({var}, weights: educ, weight_type: frequency)\n"
            ));
            expected.push((format!("frequency {call}({var})"), number(&w[key])));
        }
        script.push_str(&format!(
            "d.sum({var}, weights: educ)\nd.mean({var}, weights: educ)\n"
        ));
        expected.push((format!("sampling sum({var})"), number(&w["sum_frequency"])));
        expected.push((format!("sampling mean({var})"), number(&w["mean"])));
        let probs: Vec<f64> = w["quantiles"]["probs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        for (p, want) in probs
            .iter()
            .zip(w["quantiles"]["values"].as_array().unwrap())
        {
            script.push_str(&format!("d.quantile({var}, p: {p:?}, weights: educ)\n"));
            expected.push((format!("weighted quantile({var}, {p})"), number(want)));
        }
    }
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), expected.len(), "{stdout}");
    for (line, (what, want)) in lines.iter().zip(&expected) {
        assert_printed(what, line, *want);
    }
}

#[test]
fn the_exact_python_reference_on_the_descriptive_datasets() {
    let p = reference("tests/reference/descriptive.json");
    for (name, dataset) in p["datasets"].as_object().unwrap() {
        let csv = std::fs::read(root().join(format!("tests/reference/data/{name}.csv"))).unwrap();
        let outcome = run_with(
            &[("data/x.csv", &csv)],
            "let d = import(\"data/x.csv\")\nd.sum(x)\nd.mean(x)\nd.variance(x)\nd.sd(x)\n",
        );
        assert_eq!(outcome.exit_code, 0, "{name}: {}", outcome.output.stderr);
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        for (line, key) in lines.iter().zip(["sum", "mean", "variance", "sd"]) {
            let want = dataset[key].as_str().unwrap();
            if want == "NA" {
                assert_eq!(*line, "NA", "{name}: {key}");
            } else {
                assert_printed(&format!("{name}: {key}"), line, want.parse().unwrap());
            }
        }
    }
}

// ----- groups and the reducers inside add -----

#[test]
fn by_gives_a_table_in_key_order_with_na_last() {
    let csv = "g,x,w\na,1,1\nb,10,2\na,3,1\n,7,1\nb,20,2\na,,1\n";
    let outcome = run_with(
        &[("data/g.csv", csv.as_bytes())],
        "let d = import(\"data/g.csv\")\nd.mean(x, by: g)\nd.sum(x, by: g)\nd.n_rows()\nd.quantile(x, p: [0.5, 1], by: g)\nd.summary(x, by: g, stats: [n, mean])\nd.mean(x, by: g, weights: w)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!("by_tables", outcome.output.stdout);
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "warning: 1 row with NA in x was skipped (line 2)\nwarning: 1 row with NA in x was skipped (line 3)\nwarning: 1 row with NA in x was skipped (line 5)\nwarning: 1 row with NA in x was skipped (line 6)\nwarning: 1 row with NA in x was skipped (line 7)\n"
    );
}

#[test]
fn reducers_inside_add_work_per_group() {
    let csv = "g,x\na,1\nb,10\na,3\nb,20\na,2\n";
    let outcome = run_with(
        &[("data/g.csv", csv.as_bytes())],
        "let d = import(\"data/g.csv\")\nd.add(dev = x - mean(x), by: g)[\"dev\"]\nd.add(share = x / sum(x), rows = n(), kinds = n_distinct(g), by: g)\nd.add(m = mean(x), big = x > median(x))[\"big\"]\nd.add(x = x - mean(x), by: g)[\"x\"]\nd.add(top = max(x), by: g)[\"top\"]\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "Column<Float> · 5 values\n-1\n-5\n1\n5\n0\n\
         frame · 5 rows × 5 variables\n\n      g    x      share   rows   kinds\n  1   a    1   0.166667      3       1\n  2   b   10   0.333333      2       1\n  3   a    3        0.5      3       1\n  4   b   20   0.666667      2       1\n  5   a    2   0.333333      3       1\n\
         Column<Bool> · 5 values\nfalse\ntrue\nfalse\ntrue\nfalse\n\
         Column<Float> · 5 values\n-1\n-5\n1\n5\n0\n\
         Column<Int> · 5 values\n3\n20\n3\n20\n3\n"
    );
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "note: `x` replaced the variable of that name (line 5)\n"
    );
}

#[test]
fn describe_by_group_lists_each_group_s_variables() {
    let csv = "g,x\na,1\nb,\na,3\n";
    let outcome = run_with(
        &[("data/g.csv", csv.as_bytes())],
        "let d = import(\"data/g.csv\")\nd.describe(by: g)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "d · 3 rows × 2 variables · by g\n\n  g   variable   type   non-missing   missing   example\n  a   g          Str              2         0   a\n  a   x          Int              2         0   1\n  b   g          Str              1         0   b\n  b   x          Int              0         1   NA\n"
    );
}

// ----- the rules of section 3.7 and 4.1 -----

#[test]
fn int_stays_int_bool_counts_true_and_all_missing_is_na() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.sum(hours) + 1\nw.min(hours)\nw.max(id) + 9223372036854775000\nw.sum(wage)\n\
         let b = import(\"data/one_row.csv\")\nb.sum(flag)\nb.mean(flag)\n\
         let a = import(\"data/all_na.csv\", types: {x: Float, y: Int})\na.sum(x)\na.median(y)\na.min(x)\n\
         let e = import(\"data/empty.csv\", types: {x: Float, y: Int})\ne.sum(x)\ne.variance(y)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "76\n35\n9223372036854775003\n73.6\n1\n1\nNA\nNA\nNA\nNA\nNA\n"
    );
    insta::assert_snapshot!("reducer_warnings", without_format_notes(&stderr));
    let (code, _, stderr) =
        go("let w = import(\"data/wages.csv\")\nw.add(big = 4611686018427387904).sum(big)\n");
    assert_eq!(code, 1);
    assert!(
        stderr.contains("error[Z0211]: Int overflow: the sum of big does not fit in 64 bits"),
        "{stderr}"
    );
}

#[test]
fn the_weights_rules_of_section_4_1() {
    let csv = "x,w,neg,frac,zero,wna\n1,1,1,1,0,1\n2,2,-1,1.5,1,\n3,1,1,1,1,1\n";
    let files: &[(&str, &[u8])] = &[("data/w.csv", csv.as_bytes())];
    let outcome = run_with(
        files,
        "let d = import(\"data/w.csv\")\nd.mean(x, weights: w)\nd.mean(x, weights: zero)\nd.mean(x, weights: wna)\nd.sum(x, weights: w, weight_type: frequency)\nd.sum(x, weights: w)\nd.sum(x, weights: w, weight_type: analytic)\nd.n_rows()\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "2\n2.5\n2\n8\n8\n6\n3\n");
    insta::assert_snapshot!("weight_notes", without_format_notes(&outcome.output.stderr));
    let mut all = String::new();
    for script in [
        "let d = import(\"data/w.csv\")\nd.mean(x, weights: neg)\n",
        "let d = import(\"data/w.csv\")\nd.mean(x, weights: frac, weight_type: frequency)\n",
        "let d = import(\"data/w.csv\")\nd.mean(x, weights: nope)\n",
        "let d = import(\"data/w.csv\")\nd.mean(x, weights: x, weight_type: heavy)\n",
    ] {
        let outcome = run_with(files, script);
        assert_eq!(outcome.exit_code, 1, "{script}");
        all.push_str(&without_format_notes(&outcome.output.stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("weight_errors", all);
}

#[test]
fn quantile_options_are_checked_and_n_needs_a_frame() {
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.quantile(wage)\n",
        "let w = import(\"data/wages.csv\")\nw.quantile(wage, p: 1.5)\n",
        "let w = import(\"data/wages.csv\")\nw.quantile(wage, p: 0.5, type: 10)\n",
        "let w = import(\"data/wages.csv\")\nw.quantile(wage, p: \"half\")\n",
        "n()\n",
        "let w = import(\"data/wages.csv\")\nw.mean(name)\n",
        "let w = import(\"data/wages.csv\")\nw.mean(wage, by: nope)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("reducer_refusals", all);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(10)))]

    #[test]
    fn group_means_weight_back_to_the_grand_mean(values in prop::collection::vec((0u8..3, -50i64..50), 2..16)) {
        let csv: String = std::iter::once("g,x".to_string())
            .chain(values.iter().map(|(g, x)| format!("{g},{x}")))
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let outcome = run_with(
            &[("data/p.csv", csv.as_bytes())],
            "let d = import(\"data/p.csv\")\nd.add(m = mean(x), by: g).mean(m)\nd.mean(x)\nd.add(dev = x - mean(x), by: g).sum(dev)\nd.sum(x, by: g)\nd.sum(x)\n",
        );
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        // The mean of the group means put back on every row is the grand mean, and the deviations
        // from group means sum to zero.
        prop_assert_eq!(lines[0], lines[1]);
        let dev: f64 = lines[2].parse().unwrap();
        prop_assert!(dev.abs() < 1e-9, "{}", lines[2]);
        let total: i64 = values.iter().map(|(_, x)| *x).sum();
        prop_assert_eq!(*lines.last().unwrap(), total.to_string());
    }
}
