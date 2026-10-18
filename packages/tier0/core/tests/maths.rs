//! Maths, part 1 (Language Reference, sections 3.7 and 4.1): `abs`, `sign`, `sqrt`, `floor`,
//! `ceil`, `truncate`, `round`, `min`, `max`, `div`, `mod`, `is_finite`, `PI` and `E` through the
//! engine, on scalars and on columns. The reference is `tests/reference/maths.json` (Python's exact
//! arithmetic; the kernel's own tests hold it to the last bit, these hold the printed values); the
//! edge cases are the datasets of `tests/datasets/`; the properties are the identities of section
//! 3.7; and every message the functions can give is a snapshot.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_core::display::format_float;
use zeff_testkit::{root, run};

/// Exit code, standard output and messages of a script run in the project of every dataset.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The messages without the one note every import prints.
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

// ----- the type rules of section 3.7, on scalars -----

#[test]
fn scalars_follow_the_type_rules() {
    let (code, stdout, stderr) = go(
        "abs(-3)\nabs(-2.5)\nabs(NA)\nsign(-2.5)\nsign(0)\nsign(7)\nsign(NA)\nsqrt(16)\nsqrt(2)\nsqrt(0)\n\
         floor(2.7)\nfloor(-2.7)\nfloor(3)\nceil(2.1)\nceil(-2.7)\ntruncate(2.7)\ntruncate(-2.7)\n\
         round(2.5)\nround(3.5)\nround(-2.5)\nround(2.5, mode: away)\nround(2.675, digits: 2)\nround(1250, digits: -2)\n\
         round(123456.789, sig: 3)\nround(0.00123456, sig: 2)\nround(NA, digits: 2)\n\
         min(3, 7.5)\nmin(4, 2, 9)\nmin(1, NA)\nmax(3, 7.5)\nmax(4, 2, 9)\nmax(NA, 1)\n\
         div(7, 2)\ndiv(-7, 2)\ndiv(7, -2)\ndiv(7.5, 2)\ndiv(7.5, 0.1)\nmod(7, 2)\nmod(-7, 2)\nmod(7, -2)\nmod(7.5, 2)\nmod(NA, 2)\n\
         is_finite(2.5)\nis_finite(3)\nis_finite(NA)\nPI\nE\n2 * PI\nE ^ 2\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "3\n2.5\nNA\n-1\n0\n1\nNA\n4\n1.41421\n0\n\
         2\n-3\n3\n3\n-2\n2\n-2\n\
         2\n4\n-2\n3\n2.67\n1200\n\
         123000\n0.0012\nNA\n\
         3\n2\nNA\n7.5\n9\nNA\n\
         3\n-4\n-4\n3\n74\n1\n1\n-1\n1.5\nNA\n\
         true\ntrue\nfalse\n3.14159\n2.71828\n6.28319\n7.38906\n"
    );
    assert_eq!(stderr, "", "no message on a plain run");
}

#[test]
fn int_stays_int_and_a_float_makes_a_float() {
    // An Int result joins Int arithmetic without a conversion; a Float result does not.
    let (code, stdout, stderr) = go(
        "let a = abs(-3) + 9223372036854775000\nprint(a)\nlet b = div(10, 3) * 3 + mod(10, 3)\nprint(b)\n\
         let c = min(2, 3) + 9223372036854775000\nprint(c)\nprint(max(1, 2.0) + 0.5)\nprint(sign(-0.5) + 9223372036854775000)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "9223372036854775003\n10\n9223372036854775002\n2.5\n9223372036854774999\n"
    );
}

// ----- the reference -----

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/maths.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// What a reference value prints as: `NA`, `Inf`, `-Inf`, or the number to six digits.
fn shown(text: &str) -> String {
    match text {
        "NA" | "Inf" | "-Inf" => text.to_string(),
        other => format_float(other.parse().unwrap()),
    }
}

/// A grid value as a script writes it (a negative number inside the call's parentheses).
fn literal(text: &str) -> String {
    match text {
        "Inf" => "1 / 0".to_string(),
        "-Inf" => "-1 / 0".to_string(),
        other => other.to_string(),
    }
}

#[test]
fn the_elementary_functions_print_the_reference_values() {
    let reference = reference();
    let grid: Vec<&str> = reference["grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // sqrt against the reference; floor, ceil, truncate, sign and abs computed exactly here, as
    // they are exact operations (section 3.7).
    let sqrt: Vec<&str> = reference["functions"]["sqrt"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let expressions: Vec<String> = grid
        .iter()
        .flat_map(|x| {
            let x = literal(x);
            [
                format!("sqrt({x})"),
                format!("floor({x})"),
                format!("ceil({x})"),
                format!("truncate({x})"),
                format!("sign({x})"),
                format!("abs({x})"),
            ]
        })
        .collect();
    let lines = printed(&expressions);
    assert_eq!(lines.len(), grid.len() * 6);
    for (i, x) in grid.iter().enumerate() {
        let value: f64 = match *x {
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            other => other.parse().unwrap(),
        };
        let got = &lines[i * 6..i * 6 + 6];
        assert_eq!(got[0], shown(sqrt[i]), "sqrt({x})");
        assert_eq!(got[1], format_float(value.floor()), "floor({x})");
        assert_eq!(got[2], format_float(value.ceil()), "ceil({x})");
        assert_eq!(got[3], format_float(value.trunc()), "truncate({x})");
        let sign = if value > 0.0 {
            1
        } else if value < 0.0 {
            -1
        } else {
            0
        };
        assert_eq!(got[4], sign.to_string(), "sign({x})");
        assert_eq!(got[5], format_float(value.abs()), "abs({x})");
    }
}

#[test]
fn rounding_prints_the_reference_values_in_both_modes() {
    let reference = reference();
    let cases = reference["round"].as_array().unwrap();
    let expressions: Vec<String> = cases
        .iter()
        .flat_map(|c| {
            let (x, digits) = (c[0].as_str().unwrap(), c[1].as_i64().unwrap());
            [
                format!("round({x}, digits: {digits})"),
                format!("round({x}, digits: {digits}, mode: away)"),
            ]
        })
        .collect();
    let lines = printed(&expressions);
    for (i, c) in cases.iter().enumerate() {
        let (even, away) = (c[2].as_str().unwrap(), c[3].as_str().unwrap());
        assert_eq!(
            lines[2 * i],
            shown(even),
            "round({}, {}) half to even",
            c[0],
            c[1]
        );
        assert_eq!(
            lines[2 * i + 1],
            shown(away),
            "round({}, {}) half away from zero",
            c[0],
            c[1]
        );
    }
    let cases = reference["round_significant"].as_array().unwrap();
    let expressions: Vec<String> = cases
        .iter()
        .map(|c| {
            format!(
                "round({}, sig: {})",
                c[0].as_str().unwrap(),
                c[1].as_i64().unwrap()
            )
        })
        .collect();
    let lines = printed(&expressions);
    for (i, c) in cases.iter().enumerate() {
        assert_eq!(
            lines[i],
            shown(c[2].as_str().unwrap()),
            "round({}, sig: {})",
            c[0],
            c[1]
        );
    }
}

#[test]
fn floored_division_prints_the_reference_values() {
    let reference = reference();
    let ints: Vec<&Json> = reference["div_mod_int"]
        .as_array()
        .unwrap()
        .iter()
        // The two extreme values are written as Int literals the language has no form for.
        .filter(|c| c[0].as_i64().unwrap().unsigned_abs() < (1 << 62))
        .collect();
    let expressions: Vec<String> = ints
        .iter()
        .flat_map(|c| {
            let (a, b) = (c[0].as_i64().unwrap(), c[1].as_i64().unwrap());
            [format!("div({a}, {b})"), format!("mod({a}, {b})")]
        })
        .collect();
    let lines = printed(&expressions);
    for (i, c) in ints.iter().enumerate() {
        assert_eq!(lines[2 * i], c[2].as_i64().unwrap().to_string(), "{c}");
        assert_eq!(lines[2 * i + 1], c[3].as_i64().unwrap().to_string(), "{c}");
    }
    let floats = reference["div_mod_float"].as_array().unwrap();
    let expressions: Vec<String> = floats
        .iter()
        .flat_map(|c| {
            let (a, b) = (c[0].as_str().unwrap(), c[1].as_str().unwrap());
            [format!("div({a}, {b})"), format!("mod({a}, {b})")]
        })
        .collect();
    let lines = printed(&expressions);
    for (i, c) in floats.iter().enumerate() {
        assert_eq!(lines[2 * i], shown(c[2].as_str().unwrap()), "{c}");
        assert_eq!(lines[2 * i + 1], shown(c[3].as_str().unwrap()), "{c}");
    }
}

// ----- columns: row by row, with the type rules and the counted warnings -----

#[test]
fn columns_apply_row_by_row_with_the_type_of_one_row() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nabs(w[\"hours\"] - 38)\ndiv(w[\"hours\"], 7)\ndiv(w[\"hours\"], 7.0)\n\
         sign(w[\"wage\"] - 25)\nis_finite(w[\"hours\"])\nmin(w[\"hours\"], 38)\nmax(w[\"hours\"], w[\"wage\"])\nround(w[\"wage\"] / 3, digits: 1)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Int> · 3 values\n2\n3\nNA\n\
         Column<Int> · 3 values\n5\n5\nNA\n\
         Column<Float> · 3 values\n5\n5\nNA\n\
         Column<Int> · 3 values\n-1\n-1\n1\n\
         Column<Bool> · 3 values\ntrue\ntrue\nfalse\n\
         Column<Int> · 3 values\n38\n35\nNA\n\
         Column<Float> · 3 values\n40\n35\nNA\n\
         Column<Float> · 3 values\n7.8\n6.3\n10.4\n"
    );
    assert_eq!(without_format_notes(&stderr), "");
}

#[test]
fn the_edge_case_datasets_on_columns() {
    // Infinities, NaN read as NA, every value missing, no rows, one row.
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nis_finite(d[\"plus\"])\nsign(d[\"minus\"])\nabs(d[\"both\"])\nsqrt(d[\"nan_mixed\"])\nfloor(d[\"both\"])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Bool> · 3 values\ntrue\nfalse\ntrue\n\
         Column<Int> · 3 values\n1\n-1\n1\n\
         Column<Float> · 3 values\n1\nInf\nInf\n\
         Column<Float> · 3 values\n1.22474\nNA\nNA\n\
         Column<Float> · 3 values\n1\nInf\n-Inf\n"
    );
    let (code, stdout, stderr) = go(
        "let d = import(\"data/all_na.csv\", types: {x: Float, y: Int})\nsqrt(d[\"x\"])\nabs(d[\"y\"])\nis_finite(d[\"y\"])\nmin(d[\"x\"], d[\"y\"])\n\
         let e = import(\"data/empty.csv\", types: {x: Float, y: Int})\nsqrt(e[\"x\"])\nabs(e[\"y\"])\n\
         let o = import(\"data/one_row.csv\")\nround(o[\"y\"])\nmax(o[\"x\"], o[\"y\"])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Float> · 3 values\nNA\nNA\nNA\n\
         Column<Int> · 3 values\nNA\nNA\nNA\n\
         Column<Bool> · 3 values\nfalse\nfalse\nfalse\n\
         Column<Float> · 3 values\nNA\nNA\nNA\n\
         Column<Float> · 0 values\n[]\n\
         Column<Int> · 0 values\n[]\n\
         Column<Float> · 1 value\n4\n\
         Column<Float> · 1 value\n42\n"
    );
    assert_eq!(without_format_notes(&stderr), "");
}

#[test]
fn a_columns_warnings_are_counted_once() {
    let text = std::fs::read_to_string(root().join("tests/datasets/magnitudes.csv")).unwrap();
    let negatives = text
        .lines()
        .skip(1)
        .filter(|l| l.split(',').nth(1).unwrap().starts_with('-'))
        .count();
    assert!(negatives > 1);
    let (code, stdout, stderr) =
        go("let d = import(\"data/magnitudes.csv\")\nlet r = sqrt(d[\"wide\"])\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "");
    assert_eq!(
        without_format_notes(&stderr),
        format!(
            "warning: sqrt of a negative number gave NA in {negatives} rows (line 2)\n  = help: a negative number has no real square root; `where(x >= 0)` keeps the rows that have one\n"
        )
    );
    // Float division by zero and 0 / 0 in a column, each counted.
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\ndiv(w[\"wage\"] - 23.4, 0.0)\nmod(w[\"wage\"], 0.0)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Float> · 3 values\nNA\n-Inf\nInf\nColumn<Float> · 3 values\nNA\nNA\nNA\n"
    );
    insta::assert_snapshot!("column_division_warnings", without_format_notes(&stderr));
}

#[test]
fn scalar_division_by_zero_with_floats_follows_the_operator() {
    let (code, stdout, stderr) =
        go("div(1.0, 0)\ndiv(0.0, 0)\nmod(1.5, 0)\nlet inf = 1 / 0\nmod(inf, 2)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "Inf\nNA\nNA\nNA\n");
    insta::assert_snapshot!("scalar_division_warnings", stderr);
}

// ----- the errors -----

#[test]
fn whole_number_division_by_zero_is_an_error() {
    let (code, stdout, stderr) = go("print(1)\ndiv(7, 0)\n");
    assert_eq!(code, 1);
    assert_eq!(stdout, "1\n");
    assert!(
        stderr.starts_with("error[Z0212]: division by zero: `div(7, 0)` has no value"),
        "{stderr}"
    );
    insta::assert_snapshot!("div_by_zero", stderr);
    let (code, _, stderr) = go("mod(7, 0)\n");
    assert_eq!(code, 1);
    assert!(
        stderr.contains("error[Z0212]: division by zero: `mod(7, 0)` has no value"),
        "{stderr}"
    );
    // In a column, the row is named.
    let (code, _, stderr) =
        go("let w = import(\"data/wages.csv\")\ndiv(w[\"hours\"], w[\"id\"] - 2)\n");
    assert_eq!(code, 1);
    insta::assert_snapshot!("div_by_zero_in_a_row", without_format_notes(&stderr));
}

#[test]
fn the_one_overflow_of_abs_and_div() {
    let (code, _, stderr) = go("abs(-9223372036854775807 - 1)\n");
    assert_eq!(code, 1);
    insta::assert_snapshot!("abs_overflow", stderr);
    let (code, _, stderr) = go("div(-9223372036854775807 - 1, -1)\n");
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with(
            "error[Z0211]: Int overflow: div(-9223372036854775808, -1) does not fit in 64 bits"
        ),
        "{stderr}"
    );
    let (code, stdout, stderr) =
        go("mod(-9223372036854775807 - 1, -1)\nabs(-9223372036854775807)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0\n9223372036854775807\n");
}

#[test]
fn rounds_options_are_checked() {
    let (code, _, stderr) = go("round(123.456, digits: 2, sig: 3)\n");
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("error[Z0410]: `round` takes `digits:` or `sig:`, not both"),
        "{stderr}"
    );
    insta::assert_snapshot!("round_digits_and_sig", stderr);
    let (code, _, stderr) = go("round(123.456, sig: 0)\n");
    assert_eq!(code, 1);
    insta::assert_snapshot!("round_sig_zero", stderr);
    let (code, _, stderr) = go("round(2.5, mode: up)\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("error[Z0402]"), "{stderr}");
    let (code, _, stderr) = go("round(2.5, mode: \"away\")\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("error[Z0403]"), "{stderr}");
    insta::assert_snapshot!("round_mode_as_text", stderr);
}

#[test]
fn the_wrong_kinds_of_argument_are_refused_before_the_run() {
    // A run stops at its first error, so each wrong kind is tried on its own.
    let mut all = String::new();
    for script in [
        "abs(\"a\")\n",
        "abs(true)\n",
        "min(1, 2, \"a\")\n",
        "sqrt([1, 2])\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        assert!(stderr.starts_with("error[Z0401]"), "{stderr}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("wrong_argument_kinds", all);
    // Columns of two lengths cannot be combined.
    let (code, _, stderr) = go(
        "let w = import(\"data/wages.csv\")\nlet o = import(\"data/one_row.csv\")\nmin(w[\"hours\"], o[\"x\"])\n",
    );
    assert_eq!(code, 1);
    assert!(
        stderr.contains("error[Z0217]: columns of 3 and 1 rows cannot be combined"),
        "{stderr}"
    );
    insta::assert_snapshot!("columns_of_two_lengths", without_format_notes(&stderr));
}

#[test]
fn the_constants_are_names_of_core() {
    let (code, _, stderr) = go("let PI = 3\n");
    assert_eq!(code, 1);
    assert!(
        stderr.contains("error[Z0105]: `PI` is a name of `core` and cannot be rebound"),
        "{stderr}"
    );
    let (code, stdout, stderr) = go("let r = 2\nPI * r ^ 2\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "12.5664\n");
}

// ----- properties -----

fn finite() -> impl Strategy<Value = f64> {
    prop_oneof![
        -1e6..1e6f64,
        -1e-3..1e-3f64,
        -1e15..1e15f64,
        Just(0.0),
        Just(-0.0),
        Just(0.5),
        Just(-2.5),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(12)))]

    #[test]
    fn the_identities_of_section_3_7_hold(xs in prop::collection::vec(finite(), 1..8), ints in prop::collection::vec((-1_000_000i64..1_000_000, -50i64..50), 1..8)) {
        let mut expressions = Vec::new();
        for x in &xs {
            let lit = if *x < 0.0 || x.is_sign_negative() { format!("({x:?})") } else { format!("{x:?}") };
            expressions.push(format!("abs({lit}) - abs(-{lit})"));
            expressions.push(format!("abs({lit}) >= 0"));
            expressions.push(format!("floor({lit}) <= {lit} && {lit} < floor({lit}) + 1"));
            expressions.push(format!("ceil({lit}) >= {lit} && {lit} > ceil({lit}) - 1"));
            expressions.push(format!("truncate({lit}) == floor(abs({lit})) * sign({lit}) || {lit} == 0"));
            expressions.push(format!("abs(round({lit}) - {lit}) <= 0.5"));
            expressions.push(format!("round(round({lit}, digits: 2), digits: 2) == round({lit}, digits: 2)"));
            expressions.push(format!("min({lit}, 1) <= max({lit}, 1)"));
            expressions.push(format!("sign({lit}) * abs({lit}) == {lit}"));
            expressions.push(format!("is_finite({lit})"));
        }
        for (a, b) in &ints {
            if *b != 0 {
                expressions.push(format!("{b} * div({a}, {b}) + mod({a}, {b}) == {a}"));
                expressions.push(format!("mod({a}, {b}) == 0 || sign(mod({a}, {b})) == sign({b})"));
                expressions.push(format!("abs(mod({a}, {b})) < abs({b})"));
            }
        }
        let lines = printed(&expressions);
        for (expression, line) in expressions.iter().zip(&lines) {
            let expected = if expression.starts_with("abs(") && expression.contains(") - abs(-") { "0" } else { "true" };
            prop_assert_eq!(line, expected, "{}", expression);
        }
    }
}
