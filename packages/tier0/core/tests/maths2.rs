//! Maths, part 2 (Language Reference, sections 3.7, 4.1 and 5.6): `exp`, `log` with `base:`,
//! `log2`, `log10`, `log1p`, `expm1`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan` and `atan2`
//! through the engine, on scalars and on columns. The reference is `tests/reference/maths.json`
//! (Python's exact arithmetic) and R's `tests/reference/r/maths.json`; the edge cases are the
//! datasets of `tests/datasets/`; the properties are the identities of the functions; and every
//! message is a snapshot.

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

/// A script that prints each expression on its own line, and the lines it printed.
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

fn reference(path: &str) -> Json {
    let text = std::fs::read_to_string(root().join(path)).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// What a reference value prints as.
fn shown(text: &str) -> String {
    match text {
        "NA" | "Inf" | "-Inf" => text.to_string(),
        other => format_float(other.parse().unwrap()),
    }
}

fn literal(text: &str) -> String {
    match text {
        "Inf" => "1 / 0".to_string(),
        "-Inf" => "-1 / 0".to_string(),
        other => other.to_string(),
    }
}

const FUNCTIONS: [&str; 13] = [
    "exp", "log", "log2", "log10", "log1p", "expm1", "sin", "cos", "tan", "asin", "acos", "atan",
    "sqrt",
];

#[test]
fn every_function_prints_the_exact_reference_on_the_whole_grid() {
    let reference = reference("tests/reference/maths.json");
    let grid: Vec<&str> = reference["grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    for name in FUNCTIONS {
        let expected: Vec<&str> = reference["functions"][name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let expressions: Vec<String> = grid
            .iter()
            .map(|x| format!("{name}({})", literal(x)))
            .collect();
        let lines = printed(&expressions);
        for (i, x) in grid.iter().enumerate() {
            assert_eq!(lines[i], shown(expected[i]), "{name}({x})");
        }
    }
    // `log` with a base against log2 and log10.
    for (base, name) in [("2", "log2"), ("10", "log10")] {
        let expected: Vec<&str> = reference["functions"][name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let expressions: Vec<String> = grid
            .iter()
            .map(|x| format!("log({}, base: {base})", literal(x)))
            .collect();
        let lines = printed(&expressions);
        for (i, x) in grid.iter().enumerate() {
            assert_eq!(lines[i], shown(expected[i]), "log({x}, base: {base})");
        }
    }
    let expressions: Vec<String> = reference["atan2"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            format!(
                "atan2({}, {})",
                literal(c[0].as_str().unwrap()),
                literal(c[1].as_str().unwrap())
            )
        })
        .collect();
    let lines = printed(&expressions);
    for (i, c) in reference["atan2"].as_array().unwrap().iter().enumerate() {
        assert_eq!(lines[i], shown(c[2].as_str().unwrap()), "{c}");
    }
}

#[test]
fn every_function_agrees_with_r_at_six_digits() {
    let r = reference("tests/reference/r/maths.json");
    let grid: Vec<&str> = r["grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    for name in FUNCTIONS {
        let expected: Vec<&str> = r["functions"][name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let expressions: Vec<String> = grid
            .iter()
            .map(|x| format!("{name}({})", literal(x)))
            .collect();
        let lines = printed(&expressions);
        for (i, x) in grid.iter().enumerate() {
            // R's values are stored with 15 digits and Zeff prints 6: a display tie could differ
            // in the last digit, so the comparison is numeric.
            let (a, b) = (parse(&lines[i]), parse(&shown(expected[i])));
            let close = match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) => a == b || (a - b).abs() <= 2e-5 * a.abs().max(b.abs()),
                _ => false,
            };
            assert!(close, "{name}({x}): Zeff {} R {}", lines[i], expected[i]);
        }
    }
}

fn parse(text: &str) -> Option<f64> {
    match text {
        "NA" => None,
        "Inf" => Some(f64::INFINITY),
        "-Inf" => Some(f64::NEG_INFINITY),
        other => other.parse().ok(),
    }
}

#[test]
fn scalars_and_the_edges_of_section_4_1() {
    let (code, stdout, stderr) = go(
        "exp(0)\nexp(1)\nexp(NA)\nlog(1)\nlog(E)\nlog(8, base: 2)\nlog(1000, base: 10)\nlog(81, base: 3)\nlog(NA)\nlog(2, base: NA)\n\
         log2(8)\nlog10(0.01)\nlog1p(0)\nlog1p(1e-10)\nexpm1(0)\nexpm1(1e-10)\n\
         sin(0)\ncos(0)\ntan(0)\nasin(1)\nacos(1)\natan(1)\natan2(1, 1)\natan2(0, -1)\natan2(0, 0)\natan2(NA, 1)\natan(1e300)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "1\n2.71828\nNA\n0\n1\n3\n3\n4\nNA\nNA\n\
         3\n-2\n0\n1e-10\n0\n1e-10\n\
         0\n1\n0\n1.5708\n0\n0.785398\n0.785398\n3.14159\n0\nNA\n1.5708\n"
    );
    assert_eq!(stderr, "");
    // The warnings of section 4.1 on scalars.
    let (code, stdout, stderr) = go(
        "log(0)\nlog(-1)\nlog(0, base: 0.5)\nlog(2, base: 1)\nlog(2, base: -2)\nlog2(0)\nlog10(-3)\nlog1p(-1)\nlog1p(-2)\n\
         let inf = 1 / 0\nsin(inf)\ncos(-inf)\ntan(inf)\nasin(2)\nacos(-1.5)\nexp(1000)\nexp(-1000)\natan(inf)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "-Inf\nNA\nInf\nNA\nNA\n-Inf\nNA\n-Inf\nNA\nNA\nNA\nNA\nNA\nNA\nInf\n0\n1.5708\n"
    );
    insta::assert_snapshot!("scalar_warnings", stderr);
}

#[test]
fn columns_apply_row_by_row_and_count_their_warnings() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nlog(w[\"hours\"])\nexp(w[\"id\"] - 1)\natan2(w[\"wage\"], w[\"hours\"])\nlog(w[\"hours\"] - 35)\nlog(w[\"wage\"] - 23.4, base: 10)\nasin(w[\"id\"] - 2)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Float> · 3 values\n3.68888\n3.55535\nNA\n\
         Column<Float> · 3 values\n1\n2.71828\n7.38906\n\
         Column<Float> · 3 values\n0.529317\n0.497343\nNA\n\
         Column<Float> · 3 values\n1.60944\n-Inf\nNA\n\
         Column<Float> · 3 values\n-Inf\nNA\n0.892095\n\
         Column<Float> · 3 values\n-1.5708\n0\n1.5708\n"
    );
    insta::assert_snapshot!("column_warnings", without_format_notes(&stderr));
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nsin(d[\"plus\"])\nexp(d[\"minus\"])\natan(d[\"both\"])\nlog(d[\"nan_mixed\"])\n\
         let e = import(\"data/empty.csv\", types: {x: Float, y: Int})\nexp(e[\"x\"])\nlog(e[\"y\"])\n\
         let a = import(\"data/all_na.csv\", types: {x: Float, y: Int})\natan2(a[\"x\"], a[\"y\"])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Float> · 3 values\n0.841471\nNA\n0.909297\n\
         Column<Float> · 3 values\n2.71828\n0\n7.38906\n\
         Column<Float> · 3 values\n0.785398\n1.5708\n-1.5708\n\
         Column<Float> · 3 values\n0.405465\nNA\nNA\n\
         Column<Float> · 0 values\n[]\n\
         Column<Float> · 0 values\n[]\n\
         Column<Float> · 3 values\nNA\nNA\nNA\n"
    );
    assert_eq!(
        without_format_notes(&stderr),
        "note: 2 values of nan_mixed read as NA from NaN (line 1)\nwarning: sin of an infinite value gave NA in 1 row (line 2)\n  = help: an angle must be finite; `where(is_finite(x))` keeps the rows that are\n"
    );
}

#[test]
fn the_wrong_kinds_of_argument_are_refused_before_the_run() {
    let mut all = String::new();
    for script in [
        "exp(\"a\")\n",
        "log(2, base: \"e\")\n",
        "atan2(1)\n",
        "sin(true)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("wrong_arguments", all);
}

fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![-50.0..50.0f64, -1e-6..1e-6f64, 1e-3..1e6f64, -1e6..-1e-3f64]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(12)))]

    #[test]
    fn the_identities_hold(xs in prop::collection::vec(finite(), 1..8)) {
        let mut expressions = Vec::new();
        for x in &xs {
            let lit = format!("({x:?})");
            let ax = format!("abs({lit})");
            expressions.push(format!("abs(sin({lit}) ^ 2 + cos({lit}) ^ 2 - 1) <= 1e-12"));
            expressions.push(format!("abs(tan({lit}) - sin({lit}) / cos({lit})) <= 1e-9 * (1 + abs(tan({lit})))"));
            expressions.push(format!("abs(exp(log({ax} + 1)) - ({ax} + 1)) <= 1e-12 * ({ax} + 1)"));
            expressions.push(format!("abs(log({ax} + 1, base: 2) - log2({ax} + 1)) <= 1e-12 * (1 + abs(log2({ax} + 1)))"));
            expressions.push(format!("abs(log10({ax} + 1) - log({ax} + 1) / log(10)) <= 1e-12 * (1 + abs(log10({ax} + 1)))"));
            expressions.push(format!("abs(expm1(log1p({lit} / 1e7)) - {lit} / 1e7) <= 1e-20 + 1e-15 * abs({lit})"));
            expressions.push(format!("abs(atan(tan(atan({lit}))) - atan({lit})) <= 1e-12"));
            expressions.push(format!("abs(asin(sin(atan({lit}) / 2)) - atan({lit}) / 2) <= 1e-12"));
            // acos near 1 is ill-conditioned (a 1e-8 angle has a cosine that rounds to 1), so the
            // angle is kept in (0.5, 2.1).
            expressions.push(format!("abs(acos(cos(abs(atan({lit})) + 0.5)) - (abs(atan({lit})) + 0.5)) <= 1e-12"));
            expressions.push(format!("abs(atan2({lit}, 1) - atan({lit})) <= 1e-15"));
            expressions.push(format!("abs(atan2({lit}, -1) - atan2(-{lit}, -1) * sign(-{lit}) * sign({lit})) <= 1e-15 || {lit} == 0"));
        }
        let lines = printed(&expressions);
        for (expression, line) in expressions.iter().zip(&lines) {
            prop_assert_eq!(line, "true", "{}", expression);
        }
    }
}
