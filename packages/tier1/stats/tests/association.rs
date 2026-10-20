//! `correlation` and `covariance` (release 0.3.0, step 11) against R's `cor`, `cov` and
//! `cor.test` (`tests/reference/r/association.json`): the printed matrices at three decimals, the
//! estimates and p-values of every pair within 1e-12 through the package's own function, the N
//! per pair, and the refusals and warnings as snapshots.

use serde_json::Value as Json;
use zeff_stats::{Method, correlate};
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/association.json")).unwrap();
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

/// The large wages file with `female` as a number, as the R script reads it.
const LOAD: &str = "let w = import(\"data/wages_large.csv\", types: {female: Bool})\n";

/// The cells of a printed square table, by row, after its title and heading.
fn cells(text: &str, title_lines: usize, k: usize) -> Vec<Vec<String>> {
    text.lines()
        .skip(title_lines + 2)
        .take(k)
        .map(|l| l.split_whitespace().skip(1).map(str::to_string).collect())
        .collect()
}

#[test]
fn the_matrices_print_r_at_three_decimals() {
    let r = reference();
    let vars = "[id, wage, hours, female, educ]";
    for m in r["matrices"].as_array().unwrap() {
        let (method, missing) = (
            m["method"].as_str().unwrap(),
            m["missing"].as_str().unwrap(),
        );
        let (code, stdout, stderr) = go(&format!(
            "{LOAD}w.correlation({vars}, method: {method}, missing: {missing})\n"
        ));
        assert_eq!(code, 0, "{stderr}");
        let title_lines = if missing == "listwise" { 2 } else { 1 };
        let got = cells(&stdout, title_lines, 5);
        for (i, row) in m["correlation"].as_array().unwrap().iter().enumerate() {
            for (j, value) in row.as_array().unwrap().iter().enumerate() {
                assert_eq!(
                    got[i][j],
                    format!("{:.3}", number(value)),
                    "{method} {missing} [{i}][{j}]"
                );
            }
        }
    }
    for c in r["covariances"].as_array().unwrap() {
        let missing = c["missing"].as_str().unwrap();
        let (code, stdout, stderr) =
            go(&format!("{LOAD}w.covariance({vars}, missing: {missing})\n"));
        assert_eq!(code, 0, "{stderr}");
        let title_lines = if missing == "listwise" { 2 } else { 1 };
        let got = cells(&stdout, title_lines, 5);
        for (i, row) in c["covariance"].as_array().unwrap().iter().enumerate() {
            for (j, value) in row.as_array().unwrap().iter().enumerate() {
                assert_eq!(
                    got[i][j],
                    format!("{:.3}", number(value)),
                    "covariance {missing} [{i}][{j}]"
                );
            }
        }
    }
    // The N per pair under pairwise deletion.
    let (code, stdout, stderr) = go(&format!("{LOAD}w.correlation({vars})\n"));
    assert_eq!(code, 0, "{stderr}");
    let after = stdout.split("N per pair\n").nth(1).unwrap();
    let got = cells(after, 0, 5);
    for (i, row) in r["n_pairs"].as_array().unwrap().iter().enumerate() {
        for (j, value) in row.as_array().unwrap().iter().enumerate() {
            let n = number(value) as u64;
            assert_eq!(got[i][j].replace(',', ""), n.to_string(), "N [{i}][{j}]");
        }
    }
}

#[test]
fn every_estimate_and_p_value_agrees_with_cor_test_to_twelve_digits() {
    let r = reference();
    let mut mismatches = Vec::new();
    for pair in r["pairs"].as_array().unwrap() {
        let label = pair["label"].as_str().unwrap();
        let x: Vec<f64> = pair["x"].as_array().unwrap().iter().map(number).collect();
        let y: Vec<f64> = pair["y"].as_array().unwrap().iter().map(number).collect();
        for (method, name) in [
            (Method::Pearson, "pearson"),
            (Method::Spearman, "spearman"),
            (Method::Kendall, "kendall"),
        ] {
            let (estimate, p) = correlate(&x, &y, method, true);
            let want_estimate = number(&pair[name]["estimate"]);
            let want_p = number(&pair[name]["p"]);
            let close = |got: Option<f64>, want: f64, absolute: f64| -> bool {
                match got {
                    None => want.is_nan(),
                    // R's Edgeworth series for Spearman's statistic clamps to 0 deep in a tail;
                    // a p-value below 1e-15 on both sides is the same answer.
                    Some(g) => {
                        (g - want).abs() <= 1e-12 * want.abs() + absolute
                            || (want == 0.0 && g < 1e-15)
                    }
                }
            };
            // An estimate lies in [−1, 1]: R leaves a residue of 1e-17 where the exact answer is 0.
            if !close(estimate, want_estimate, 1e-15) {
                mismatches.push(format!(
                    "{label}, {name} estimate: Zeff {estimate:?}, R {want_estimate:e}"
                ));
            }
            if !close(p, want_p, 0.0) {
                mismatches.push(format!("{label}, {name} p: Zeff {p:?}, R {want_p:e}"));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn the_layout_the_warnings_and_the_refusals() {
    let (code, stdout, stderr) = go(
        "let f = frame(x = [1, 2, 3, 4, 5, 6, 7, 8], y = [2, 1, 4, 3, 7, 5, 8, 6], c = [3, 3, 3, 3, 3, 3, 3, 3], g = [NA, 1, 2, 3, 4, 5, 6, 7])\nf.correlation([x, y], p_values: true)\nf.correlation([x, y, c])\nf.correlation([x, y, g])\nf.correlation([x, g], missing: listwise)\nf.covariance([x, y])\nf.covariance([x, g, y])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("layout", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "let f = frame(x = [1, 2, 3], y = [2, 1, 4], s = [\"a\", \"b\", \"c\"])\nf.correlation(x)\n",
        "let f = frame(x = [1, 2, 3], y = [2, 1, 4], s = [\"a\", \"b\", \"c\"])\nf.correlation([x, s])\n",
        "let f = frame(x = [1, 2, 3], y = [2, 1, 4], s = [\"a\", \"b\", \"c\"])\nf.correlation([x, z])\n",
        "let f = frame(x = [1, 2, 3], y = [2, 1, 4], s = [\"a\", \"b\", \"c\"])\nf.correlation([x, y], method: cosine)\n",
        "let f = frame(x = [1, 2, 3], y = [2, 1, 4], s = [\"a\", \"b\", \"c\"])\nf.covariance([x, y], missing: never)\n",
        "correlation([x, y])\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}
