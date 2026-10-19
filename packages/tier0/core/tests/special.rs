//! The special functions of `core` (release 0.3.0, step 6): `factorial`, `choose`, `gamma`,
//! `log_gamma` and `beta` through the engine, on scalars and on columns. The reference is R's
//! `tests/reference/r/special.json` (the kernel itself is held to the exact decimal reference in
//! `zeff-numeric`); the edge cases of section 4.1 and every message are snapshots; the properties
//! are the identities of the functions.

use proptest::prelude::*;
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

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/special.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// What R's value prints as in Zeff: six significant digits, or the special word.
fn shown(json: &Json) -> String {
    let text = json.as_str().unwrap();
    match text {
        "NA" | "Inf" | "-Inf" => text.to_string(),
        other => format_float(other.parse().unwrap()),
    }
}

/// A number written so that the script reads it back exactly.
fn literal(text: &str) -> String {
    let x: f64 = text.parse().unwrap();
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{x:.1}")
    } else {
        format!("{x:e}")
    }
}

#[test]
fn gamma_log_gamma_factorial_choose_and_beta_agree_with_r_at_six_digits() {
    let r = reference();
    let grid: Vec<&str> = r["gamma_grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    let mut expressions = Vec::new();
    let mut expected = Vec::new();
    for (i, x) in grid.iter().enumerate() {
        expressions.push(format!("gamma({})", literal(x)));
        expected.push(shown(&r["gamma"][i]));
        expressions.push(format!("log_gamma({})", literal(x)));
        expected.push(shown(&r["log_gamma"][i]));
    }
    for case in r["factorial"].as_array().unwrap() {
        expressions.push(format!("factorial({})", case[0]));
        expected.push(shown(&case[1]));
    }
    for case in r["choose"].as_array().unwrap() {
        let (n, k): (f64, f64) = (
            case[0].as_str().unwrap().parse().unwrap(),
            case[1].as_str().unwrap().parse().unwrap(),
        );
        expressions.push(format!("choose({}, {})", n as i64, k as i64));
        expected.push(shown(&case[2]));
    }
    for case in r["beta"].as_array().unwrap() {
        expressions.push(format!(
            "beta({}, {})",
            literal(case[0].as_str().unwrap()),
            literal(case[1].as_str().unwrap())
        ));
        expected.push(shown(&case[3]));
    }
    let got = printed(&expressions);
    assert_eq!(got.len(), expected.len());
    for ((expression, want), got) in expressions.iter().zip(&expected).zip(&got) {
        assert_eq!(got, want, "{expression}");
    }
}

#[test]
fn the_edges_of_section_4_1_warn_and_give_na_or_inf() {
    let (code, stdout, stderr) = go(
        "factorial(0)\nfactorial(-3)\nfactorial(171)\nchoose(5, 7)\nchoose(5, -1)\nchoose(-5, 2)\nchoose(1100, 550)\ngamma(0)\ngamma(-2)\ngamma(200)\ngamma(-1.5)\nlog_gamma(0)\nlog_gamma(1)\nbeta(0, 1)\nbeta(2, -1)\nbeta(1e-320, 1e-320)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "1\nNA\nInf\n0\n0\nNA\nInf\nNA\nNA\nInf\n2.36327\nInf\n0\nNA\nNA\nInf\n"
    );
    insta::assert_snapshot!("edge_warnings", stderr);
}

#[test]
fn columns_apply_row_by_row_and_count_their_warnings() {
    let (code, stdout, stderr) = go(
        "let x = frame(v = [0.5, 0, 5, -2, NA], n = [3, -1, 171, 20, NA])\ngamma(x[\"v\"])\nfactorial(x[\"n\"])\nchoose(x[\"n\"], 2)\nbeta(x[\"v\"], 2)\nlog_gamma(x[\"v\"])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "columns",
        format!("{stdout}---\n{}", without_format_notes(&stderr))
    );
}

#[test]
fn the_wrong_kinds_of_argument_are_refused_before_the_run() {
    let mut all = String::new();
    for script in [
        "factorial(2.5)\n",
        "choose(5, \"two\")\n",
        "gamma(true)\n",
        "beta(1)\n",
        "log_gamma()\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(20)))]

    /// Γ(n + 1) = n! exactly; C(n, k) = C(n, n − k) exactly; B(a, b) = B(b, a) exactly; and
    /// ln Γ(x + 1) − ln Γ(x) = ln x within rounding.
    #[test]
    fn the_identities_hold(n in 0i64..=170, k in 0i64..=60, a in 0.1f64..50.0, b in 0.1f64..50.0, x in 0.5f64..200.0) {
        let k = k.min(n);
        let lines = printed(&[
            format!("gamma({n} + 1) == factorial({n})"),
            format!("choose({n}, {k}) == choose({n}, {n} - {k})"),
            format!("beta({a:e}, {b:e}) == beta({b:e}, {a:e})"),
            format!("log_gamma({x:e} + 1) - log_gamma({x:e}) - log({x:e})"),
        ]);
        prop_assert_eq!(&lines[0], "true");
        prop_assert_eq!(&lines[1], "true");
        prop_assert_eq!(&lines[2], "true");
        let residue: f64 = lines[3].parse().unwrap();
        prop_assert!(residue.abs() < 1e-9, "{}", lines[3]);
    }
}
