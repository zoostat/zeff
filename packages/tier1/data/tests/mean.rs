//! `mean` complete (Language Reference 4.1, "Statistics", and 3.7, the Bool exception): the
//! edge-case datasets with the outcome each must give, properties of the mean, and a snapshot of
//! every message the function can give. The reference on every dataset is `reference_mean.rs`.

use proptest::prelude::*;
use zeff_core::display::format_float;
use zeff_testkit::{run, run_with};

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

// ----- the edge-case datasets -----

#[test]
fn all_missing_and_no_rows_give_na_with_a_warning() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/all_na.csv\", types: {x: Float, y: Int})\nd.mean(x)\nd.mean(y)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\nNA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: all 3 rows have NA in x: the result is NA (line 2)\nwarning: all 3 rows have NA in y: the result is NA (line 3)\n"
    );
    let (code, stdout, stderr) = go(
        "let d = import(\"data/empty.csv\", types: {x: Float, y: Bool})\nd.mean(x)\nd.mean(y)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\nNA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: x has no values: the result is NA (line 2)\nwarning: y has no values: the result is NA (line 3)\n"
    );
}

#[test]
fn one_row_zero_variance_and_a_large_constant() {
    let (code, stdout, stderr) =
        go("let d = import(\"data/one_row.csv\")\nd.mean(x)\nd.mean(y)\nd.mean(flag)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "42\n3.5\n1\n");
    assert_eq!(without_format_notes(&stderr), "");
    let (code, stdout, stderr) = go(
        "let d = import(\"data/zero_variance.csv\")\nd.mean(constant)\nd.mean(constant_with_na)\nd.mean(big_constant)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout, "3.25\n7\n1000000000\n",
        "a constant's mean is the constant"
    );
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 1 row with NA in constant_with_na was skipped (line 3)\n"
    );
}

#[test]
fn a_bool_column_gives_the_share_of_true() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/bools.csv\")\nd.mean(treated)\nd.mean(mixed_case)\nd.mean(with_na)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0.75\n0.5\n0.5\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 2 rows with NA in with_na were skipped (line 4)\n"
    );
}

#[test]
fn infinities_follow_the_rules_of_section_4_1() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nd.mean(plus)\nd.mean(minus)\nd.mean(both)\nd.mean(nan_mixed)\nd.mean(finite)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "Inf\n-Inf\nNA\n1.5\n2\n");
    assert_eq!(
        without_format_notes(&stderr),
        "note: 2 values of nan_mixed read as NA from NaN (line 1)\nwarning: both holds both Inf and -Inf, so its mean is NA (line 4)\nwarning: 2 rows with NA in nan_mixed were skipped (line 5)\n"
    );
}

#[test]
fn the_large_file_and_the_magnitudes_print_six_significant_digits() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages_large.csv\")\nw.mean(wage)\nw.mean(hours)\nw.mean(female)\nlet m = import(\"data/magnitudes.csv\")\nm.mean(offset)\nm.mean(tiny)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(
        lines[3].starts_with("1000000000"),
        "the offset keeps its digits before the point: {}",
        lines[3]
    );
    assert!(
        lines[4].contains("e-"),
        "a tiny mean is in exponent form: {}",
        lines[4]
    );
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 61 rows with NA in hours were skipped (line 3)\n"
    );
}

// ----- properties -----

/// A CSV of one Float column `x` with the values given, `NA` where missing.
fn column_file(values: &[Option<f64>]) -> Vec<u8> {
    let mut text = String::from("x\n");
    for v in values {
        match v {
            Some(x) => text.push_str(&format!("{x:?}\n")),
            None => text.push_str("NA\n"),
        }
    }
    text.into_bytes()
}

/// The printed mean of a one-column file, and its messages.
fn mean_of(values: &[Option<f64>]) -> (String, String) {
    let file = column_file(values);
    let outcome = run_with(
        &[("data/p.csv", &file)],
        "let d = import(\"data/p.csv\", types: {x: Float})\nd.mean(x)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    (
        outcome.output.statements[1].stdout.trim().to_string(),
        outcome.output.statements[1].stderr.clone(),
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn the_mean_of_integers_is_the_exact_mean_rounded(
        values in prop::collection::vec(prop::option::weighted(0.85, -100_000i64..100_000), 1..60)
    ) {
        let present: Vec<i64> = values.iter().flatten().copied().collect();
        let mut text = String::from("x\n");
        for v in &values {
            text.push_str(&v.map_or("NA".to_string(), |i| i.to_string()));
            text.push('\n');
        }
        let file = text.into_bytes();
        let outcome = run_with(&[("data/p.csv", &file)], "let d = import(\"data/p.csv\", types: {x: Int})\nd.mean(x)\n");
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let printed = outcome.output.statements[1].stdout.trim().to_string();
        if present.is_empty() {
            prop_assert_eq!(printed, "NA");
        } else {
            // The engine's mean is within a rounding of the exact mean (its help page's promise),
            // so at a tie of the sixth digit it may print either neighbour of the exact value.
            let exact = present.iter().map(|&v| v as i128).sum::<i128>() as f64 / present.len() as f64;
            let accepted = [format_float(exact), format_float(exact.next_up()), format_float(exact.next_down())];
            prop_assert!(accepted.contains(&printed), "printed {printed}, exact {}", format_float(exact));
        }
        let skipped = values.len() - present.len();
        let messages = &outcome.output.statements[1].stderr;
        if skipped == 0 {
            prop_assert_eq!(messages.as_str(), "");
        } else {
            prop_assert!(messages.contains(&format!("{skipped} ")) || messages.contains("all "), "{}", messages);
        }
    }

    #[test]
    fn shifting_adds_scaling_multiplies_and_order_does_not_matter(
        values in prop::collection::vec(prop::option::weighted(0.9, -1000.0f64..1000.0), 2..40),
        shift in -500.0f64..500.0,
        scale in prop::sample::select(vec![-3.0f64, 0.5, 2.0, 7.0]),
    ) {
        prop_assume!(values.iter().any(Option::is_some));
        let (base, _) = mean_of(&values);
        let base: f64 = base.parse().unwrap();
        let shifted: Vec<Option<f64>> = values.iter().map(|v| v.map(|x| x + shift)).collect();
        let (moved, _) = mean_of(&shifted);
        let moved: f64 = moved.parse().unwrap();
        // Printed values carry six significant digits, so the comparison is at that precision.
        prop_assert!((moved - (base + shift)).abs() <= 1e-4 * (1.0 + (base + shift).abs()), "shift: {moved} vs {}", base + shift);
        let scaled: Vec<Option<f64>> = values.iter().map(|v| v.map(|x| x * scale)).collect();
        let (times, _) = mean_of(&scaled);
        let times: f64 = times.parse().unwrap();
        prop_assert!((times - base * scale).abs() <= 1e-4 * (1.0 + (base * scale).abs()), "scale: {times} vs {}", base * scale);
        let mut reversed = values.clone();
        reversed.reverse();
        let (back, _) = mean_of(&reversed);
        prop_assert_eq!(back.parse::<f64>().unwrap(), base, "the order of the rows does not change the printed mean");
    }

    #[test]
    fn the_mean_of_bools_is_the_share_exactly(
        flags in prop::collection::vec(prop::option::weighted(0.8, any::<bool>()), 1..50)
    ) {
        let mut text = String::from("flag\n");
        for f in &flags {
            text.push_str(match f { Some(true) => "true\n", Some(false) => "false\n", None => "NA\n" });
        }
        let outcome = run_with(&[("data/b.csv", text.as_bytes())], "let d = import(\"data/b.csv\", types: {flag: Bool})\nd.mean(flag)\n");
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let present = flags.iter().flatten().count();
        let trues = flags.iter().flatten().filter(|&&f| f).count();
        let want = if present == 0 { "NA".to_string() } else { format_float(trues as f64 / present as f64) };
        prop_assert_eq!(outcome.output.statements[1].stdout.trim(), want.as_str());
    }
}

/// Forty integers whose exact mean is 8834.725, a tie at six significant digits: the GitHub run of
/// 18 October 2026 found the engine one unit in the last place on the other side of the tie from
/// the rounded quotient, which the promise "within a rounding" allows.
#[test]
fn a_tie_at_the_sixth_digit_prints_one_of_its_neighbours() {
    let values = [
        -96538i64, 7348, 22436, -45345, -1315, 27004, 92122, 80317, -12216, 42710, 11074, 78114,
        -26098, 33108, 27914, 20724, -9559, -17499, 40071, -81500, 13996, -769, -75812, -50937,
        -94214, -9443, 88112, 31503, 29215, 14779, 5577, 66379, 43005, 35966, 95385, -7053, 19013,
        -78217, 98318, -64286,
    ];
    assert_eq!(
        values.iter().sum::<i64>(),
        353_389,
        "the mean is 353389/40 = 8834.725"
    );
    let mut text = String::from("x\n");
    for v in values {
        text.push_str(&format!("{v}\n"));
    }
    let outcome = run_with(
        &[("data/tie.csv", text.as_bytes())],
        "let d = import(\"data/tie.csv\")\nd.mean(x)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let printed = outcome.output.statements[1].stdout.trim().to_string();
    assert!(
        printed == "8834.72" || printed == "8834.73",
        "printed {printed}"
    );
}

// ----- every message -----

#[test]
fn every_message_mean_can_give() {
    let cases = [
        (
            "one_row_skipped",
            "let w = import(\"data/wages.csv\")\nw.mean(hours)\n",
        ),
        (
            "rows_skipped",
            "let w = import(\"data/wages_large.csv\")\nw.mean(hours)\n",
        ),
        (
            "all_missing",
            "let d = import(\"data/all_na.csv\", types: {x: Float})\nd.mean(x)\n",
        ),
        (
            "no_values",
            "let d = import(\"data/empty.csv\", types: {x: Float})\nd.mean(x)\n",
        ),
        (
            "inf_and_minus_inf",
            "let d = import(\"data/infinities.csv\")\nd.mean(both)\n",
        ),
        (
            "no_such_variable",
            "let wages = import(\"data/wages.csv\")\nwages.mean(wge)\n",
        ),
        (
            "case_differs",
            "let wages = import(\"data/wages.csv\")\nwages.mean(Wage)\n",
        ),
        ("unnamed_frame", "import(\"data/wages.csv\").mean(wge)\n"),
        (
            "text_column",
            "let wages = import(\"data/wages.csv\")\nwages.mean(name)\n",
        ),
        (
            "date_column",
            "let d = import(\"data/dates.csv\")\nd.mean(sold)\n",
        ),
        ("no_arguments", "mean()\n"),
        (
            "no_variable",
            "let w = import(\"data/wages.csv\")\nw.mean()\n",
        ),
        (
            "a_number_for_the_variable",
            "let w = import(\"data/wages.csv\")\nw.mean(1)\n",
        ),
        (
            "text_for_the_variable",
            "let w = import(\"data/wages.csv\")\nw.mean(\"wage\")\n",
        ),
        (
            "two_variables",
            "let w = import(\"data/wages.csv\")\nw.mean(wage, hours)\n",
        ),
        (
            "an_option",
            "let w = import(\"data/wages.csv\")\nw.mean(wage, by: id)\n",
        ),
        ("not_a_frame", "mean(3, wage)\n"),
    ];
    let dates: &[(&str, &[u8])] = &[("data/dates.csv", b"sold\n2026-10-16\n2026-01-01\n")];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run_with(dates, script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code,
            outcome.output.stdout,
            without_format_notes(&outcome.output.stderr)
        ));
    }
    insta::assert_snapshot!("every_message", all);
}
