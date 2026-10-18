//! `sd` (Language Reference 4.1, "Statistics"): the reference on every dataset, the edge-case
//! datasets with the outcome each must give, properties of the standard deviation, and a snapshot
//! of every message the function can give.

use proptest::prelude::*;
use zeff_core::display::format_float;
use zeff_testkit::{Accuracy, assert_prints, in_code, reference, run, run_with};

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

// ----- the reference -----

#[test]
fn sd_matches_the_reference_on_every_numeric_column_of_every_dataset() {
    let reference = reference();
    assert!(reference.datasets.len() >= 2);
    let mut compared = 0;
    for (name, dataset) in &reference.datasets {
        for (column, stats) in &dataset.columns {
            let Some(written) = in_code(column) else {
                continue;
            };
            let types = if stats.n == 0 {
                format!(", types: {{{written}: Float}}")
            } else {
                String::new()
            };
            let script = format!("let d = {}\nd.sd({written})\n", dataset.import_call(&types));
            let outcome = run(&script);
            assert_eq!(
                outcome.exit_code, 0,
                "{name}.{column}:\n{script}\n{}",
                outcome.output.stderr
            );
            assert_prints(
                &format!("sd of {name}.{column}"),
                &outcome.output.statements[1].stdout,
                &stats.sd,
                Accuracy::ClosedForm,
            );
            compared += 1;
        }
    }
    assert!(compared > 20, "{compared} columns compared");
}

// ----- the edge-case datasets -----

#[test]
fn one_value_all_missing_and_no_rows_give_na_with_a_warning() {
    let (code, stdout, stderr) = go("let d = import(\"data/one_row.csv\")\nd.sd(x)\nd.sd(flag)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\nNA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: sd needs at least two values, but x has one: the result is NA (line 2)\nwarning: sd needs at least two values, but flag has one: the result is NA (line 3)\n"
    );
    // Two values with one missing is one value left: skipped, then too few.
    let (code, stdout, stderr) = go("let d = import(\"data/infinities.csv\")\nd.sd(nan_mixed)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "note: 2 values of nan_mixed read as NA from NaN (line 1)\nwarning: 2 rows with NA in nan_mixed were skipped (line 2)\nwarning: sd needs at least two values, but nan_mixed has one: the result is NA (line 2)\n"
    );
    let (code, stdout, stderr) = go(
        "let a = import(\"data/all_na.csv\", types: {x: Float})\na.sd(x)\nlet e = import(\"data/empty.csv\", types: {x: Float})\ne.sd(x)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\nNA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: all 3 rows have NA in x: the result is NA (line 2)\nwarning: x has no values: the result is NA (line 4)\n"
    );
}

#[test]
fn a_constant_column_has_sd_zero_whatever_its_magnitude() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/zero_variance.csv\")\nd.sd(constant)\nd.sd(constant_with_na)\nd.sd(big_constant)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0\n0\n0\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 1 row with NA in constant_with_na was skipped (line 3)\n"
    );
}

#[test]
fn infinities_make_the_sd_na_with_a_warning() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nd.sd(plus)\nd.sd(minus)\nd.sd(both)\nd.sd(finite)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\nNA\nNA\n1\n");
    assert_eq!(
        without_format_notes(&stderr),
        "note: 2 values of nan_mixed read as NA from NaN (line 1)\nwarning: plus holds an infinite value, so its sd is NA (line 2)\nwarning: minus holds an infinite value, so its sd is NA (line 3)\nwarning: both holds an infinite value, so its sd is NA (line 4)\n"
    );
}

#[test]
fn bools_the_large_file_and_the_magnitudes() {
    let (code, stdout, stderr) = go(
        "let b = import(\"data/bools.csv\")\nb.sd(treated)\nlet w = import(\"data/wages_large.csv\")\nw.sd(hours)\nlet m = import(\"data/magnitudes.csv\")\nm.sd(offset)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    // Three true and one false: the share is 0.75 and the sample sd is sqrt(0.75 * 0.25 * 4 / 3).
    assert_eq!(lines[0], format_float((0.75f64 * 0.25 * 4.0 / 3.0).sqrt()));
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 61 rows with NA in hours were skipped (line 4)\n"
    );
    // The offset column is a billion plus a spread below one: the spread, not the offset, is
    // what the two-pass algorithm keeps.
    let offset_sd: f64 = lines[2].parse().unwrap();
    assert!(offset_sd > 0.0 && offset_sd < 1.0, "{}", lines[2]);
}

// ----- properties -----

/// A CSV of one Float column `x`, `NA` where missing.
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

/// The printed sd of a one-column file.
fn sd_of(values: &[Option<f64>]) -> String {
    let file = column_file(values);
    let outcome = run_with(
        &[("data/p.csv", &file)],
        "let d = import(\"data/p.csv\", types: {x: Float})\nd.sd(x)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    outcome.output.statements[1].stdout.trim().to_string()
}

/// Whether `printed` is the six-digit form of `x` or of a float next to it.
fn prints_as(printed: &str, x: f64) -> bool {
    [x, x.next_up(), x.next_down()]
        .iter()
        .any(|v| format_float(*v) == printed)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn the_sd_of_integers_is_the_exact_sd_rounded(
        values in prop::collection::vec(prop::option::weighted(0.85, -100_000i64..100_000), 2..60)
    ) {
        let present: Vec<i64> = values.iter().flatten().copied().collect();
        let mut text = String::from("x\n");
        for v in &values {
            text.push_str(&v.map_or("NA".to_string(), |i| i.to_string()));
            text.push('\n');
        }
        let outcome = run_with(&[("data/p.csv", text.as_bytes())], "let d = import(\"data/p.csv\", types: {x: Int})\nd.sd(x)\n");
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let printed = outcome.output.statements[1].stdout.trim().to_string();
        if present.len() < 2 {
            prop_assert_eq!(printed, "NA");
        } else {
            // n Σx² − (Σx)² is exact in 128-bit integers; one division and one square root follow.
            let n = present.len() as i128;
            let sum: i128 = present.iter().map(|&v| v as i128).sum();
            let squares: i128 = present.iter().map(|&v| (v as i128) * (v as i128)).sum();
            let variance = (n * squares - sum * sum) as f64 / (n * (n - 1)) as f64;
            prop_assert!(prints_as(&printed, variance.sqrt()), "printed {printed}, exact {}", format_float(variance.sqrt()));
        }
    }

    #[test]
    fn shifting_leaves_the_sd_scaling_multiplies_it_and_order_does_not_matter(
        values in prop::collection::vec(prop::option::weighted(0.9, -1000.0f64..1000.0), 3..40),
        shift in -500.0f64..500.0,
        scale in prop::sample::select(vec![-3.0f64, 0.5, 2.0, 7.0]),
    ) {
        prop_assume!(values.iter().flatten().count() >= 2);
        let base: f64 = sd_of(&values).parse().unwrap();
        let shifted: Vec<Option<f64>> = values.iter().map(|v| v.map(|x| x + shift)).collect();
        let moved: f64 = sd_of(&shifted).parse().unwrap();
        prop_assert!((moved - base).abs() <= 1e-4 * (1.0 + base), "shift: {moved} vs {base}");
        let scaled: Vec<Option<f64>> = values.iter().map(|v| v.map(|x| x * scale)).collect();
        let times: f64 = sd_of(&scaled).parse().unwrap();
        prop_assert!((times - base * scale.abs()).abs() <= 1e-4 * (1.0 + base * scale.abs()), "scale: {times} vs {}", base * scale.abs());
        let mut reversed = values.clone();
        reversed.reverse();
        prop_assert_eq!(sd_of(&reversed).parse::<f64>().unwrap(), base, "order");
    }

    #[test]
    fn two_values_have_sd_their_distance_over_root_two(a in -1e6f64..1e6, b in -1e6f64..1e6) {
        let printed = sd_of(&[Some(a), Some(b)]);
        prop_assert!(prints_as(&printed, (a - b).abs() / 2f64.sqrt()), "printed {printed} for {a}, {b}");
    }
}

// ----- every message -----

#[test]
fn every_message_sd_can_give() {
    let cases = [
        (
            "rows_skipped",
            "let w = import(\"data/wages.csv\")\nw.sd(hours)\n",
        ),
        (
            "all_missing",
            "let d = import(\"data/all_na.csv\", types: {x: Float})\nd.sd(x)\n",
        ),
        (
            "no_values",
            "let d = import(\"data/empty.csv\", types: {x: Float})\nd.sd(x)\n",
        ),
        (
            "one_value",
            "let d = import(\"data/one_row.csv\")\nd.sd(y)\n",
        ),
        (
            "infinite",
            "let d = import(\"data/infinities.csv\")\nd.sd(plus)\n",
        ),
        (
            "no_such_variable",
            "let wages = import(\"data/wages.csv\")\nwages.sd(wge)\n",
        ),
        (
            "case_differs",
            "let wages = import(\"data/wages.csv\")\nwages.sd(Wage)\n",
        ),
        (
            "text_column",
            "let wages = import(\"data/wages.csv\")\nwages.sd(name)\n",
        ),
        (
            "no_variable",
            "let w = import(\"data/wages.csv\")\nw.sd()\n",
        ),
        (
            "text_for_the_variable",
            "let w = import(\"data/wages.csv\")\nw.sd(\"wage\")\n",
        ),
        (
            "two_variables",
            "let w = import(\"data/wages.csv\")\nw.sd(wage, hours)\n",
        ),
        ("not_a_frame", "sd(3, wage)\n"),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code,
            outcome.output.stdout,
            without_format_notes(&outcome.output.stderr)
        ));
    }
    insta::assert_snapshot!("every_message", all);
}
