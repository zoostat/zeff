//! `summary` (Language Reference 10.2): the reference on every dataset, the edge-case datasets,
//! the `vars` and `stats:` arguments as the owner decided them, a property on random frames, and
//! a snapshot of every layout and every message.

use proptest::prelude::*;
use zeff_testkit::{Expected, in_code, reference, run, run_with};

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

/// The cells after the variable's name in its row of a printed summary table; the name may hold
/// spaces, so the row is found by its start.
fn row_of(text: &str, column: &str) -> Option<Vec<String>> {
    let start = format!("  {column} ");
    text.lines()
        .skip(3)
        .find(|l| l.starts_with(&start))
        .map(|l| {
            l[start.len()..]
                .split_whitespace()
                .map(str::to_string)
                .collect()
        })
}

/// The rows of a printed summary table: the cells of each row after the title, the blank line
/// and the headings, split on runs of spaces (for variable names without spaces).
fn table_rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .skip(3)
        .map(|l| l.split_whitespace().map(str::to_string).collect())
        .collect()
}

/// Whether a printed three-decimal cell agrees with the reference: `NA` with `NA` or nothing,
/// `Inf` with `Inf`, a number within half a unit of the third decimal.
fn agrees(cell: &str, expected: &Expected) -> bool {
    match expected {
        Expected::Nothing | Expected::Na => cell == "NA",
        Expected::Inf => cell == "Inf",
        Expected::NegInf => cell == "-Inf",
        Expected::Number(x) => cell
            .parse::<f64>()
            .is_ok_and(|v| (v - x).abs() <= 0.0005 + 1e-9 * x.abs()),
    }
}

// ----- the reference -----

#[test]
fn summary_matches_the_reference_on_every_dataset() {
    let reference = reference();
    let mut compared = 0;
    for (name, dataset) in &reference.datasets {
        let script = format!("let d = {}\nd.summary()\n", dataset.import_call(""));
        let outcome = run(&script);
        assert_eq!(outcome.exit_code, 0, "{name}:\n{}", outcome.output.stderr);
        let text = &outcome.output.statements[1].stdout;
        assert_eq!(text.lines().next(), Some("summary · d"), "{name}");
        assert_eq!(
            text.lines()
                .nth(2)
                .map(|l| l.split_whitespace().collect::<Vec<_>>()),
            Some(vec!["variable", "N", "missing", "mean", "sd", "min", "max"]),
            "{name}"
        );
        for (column, stats) in &dataset.columns {
            if in_code(column).is_none() {
                continue;
            }
            // A column with no value at all is text at import and is not summarised.
            let Some(row) = row_of(text, column) else {
                assert_eq!(
                    stats.n, 0,
                    "{name}.{column} is missing from the table:\n{text}"
                );
                continue;
            };
            assert_eq!(row.len(), 6, "{name}.{column}: {row:?}");
            assert_eq!(
                row[0].replace(',', "").parse::<usize>().unwrap(),
                stats.n,
                "{name}.{column}: N"
            );
            assert_eq!(
                row[1].replace(',', "").parse::<usize>().unwrap(),
                stats.missing,
                "{name}.{column}: missing"
            );
            for (i, (what, expected)) in [
                ("mean", &stats.mean),
                ("sd", &stats.sd),
                ("min", &stats.min),
                ("max", &stats.max),
            ]
            .iter()
            .enumerate()
            {
                assert!(
                    agrees(&row[2 + i], expected),
                    "{name}.{column}: {what} printed {} for {expected:?}",
                    row[2 + i]
                );
            }
            compared += 1;
        }
    }
    assert!(compared > 20, "{compared} columns compared");
}

// ----- the arguments -----

#[test]
fn vars_and_stats_choose_the_rows_and_the_columns() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.summary(wage)\nw.summary([hours, wage], stats: [max, n, n])\nw.summary(stats: [sd])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("vars_and_stats", stdout);
    assert_eq!(
        without_format_notes(&stderr),
        "warning: 1 row with NA in hours was skipped (line 3)\nwarning: 1 row with NA in hours was skipped (line 4)\n"
    );
}

#[test]
fn rows_skipped_in_several_variables_are_one_warning() {
    let (code, stdout, stderr) = go(
        "frame(a = [1, NA, 3, 4], b = [NA, NA, 5, 6], c = [1, 2, 3, 4], d = [NA, 2, 2, 2]).summary()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\n  a          3         1"), "{stdout}");
    assert_eq!(
        stderr,
        "warning: 1 row with NA in a, 2 with NA in b and 1 with NA in d were skipped (line 1)\n"
    );
}

// ----- the edge-case datasets -----

#[test]
fn the_edge_cases_print_na_where_a_statistic_is_undefined() {
    let scripts = [
        ("one_row", "import(\"data/one_row.csv\").summary()\n"),
        (
            "all_na",
            "import(\"data/all_na.csv\", types: {x: Float, y: Int, z: Bool}).summary()\n",
        ),
        (
            "empty",
            "import(\"data/empty.csv\", types: {x: Float}).summary()\n",
        ),
        (
            "zero_variance",
            "import(\"data/zero_variance.csv\").summary()\n",
        ),
        ("infinities", "import(\"data/infinities.csv\").summary()\n"),
        ("bools", "import(\"data/bools.csv\").summary()\n"),
        (
            "wages_large",
            "import(\"data/wages_large.csv\").summary()\n",
        ),
        (
            "magnitudes",
            "import(\"data/magnitudes.csv\").summary(stats: [mean, sd, min, max])\n",
        ),
        (
            "no_numeric_variable",
            "import(\"data/unicode_labels.csv\").summary([city])\n",
        ),
    ];
    let mut all = String::new();
    for (name, script) in scripts {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!(
            "## {name} (exit {code})\n{stdout}{}\n",
            without_format_notes(&stderr)
        ));
    }
    insta::assert_snapshot!("edge_cases", all);
}

// ----- a property -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]
    #[test]
    fn a_random_frame_summarises_to_consistent_counts_and_bounds(
        columns in prop::collection::vec(prop::collection::vec(prop::option::weighted(0.8, -500i64..500), 2..30), 1..5)
    ) {
        let rows = columns[0].len();
        let columns: Vec<Vec<Option<i64>>> = columns.into_iter().map(|mut c| {
            c.resize(rows, Some(3));
            if c.iter().all(Option::is_none) { c[0] = Some(7); }
            c
        }).collect();
        let definitions: Vec<String> = columns.iter().enumerate().map(|(i, c)| {
            let values: Vec<String> = c.iter().map(|v| v.map_or("NA".to_string(), |x| x.to_string())).collect();
            format!("c{i} = [{}]", values.join(", "))
        }).collect();
        let (code, stdout, stderr) = go(&format!("frame({}).summary()\n", definitions.join(", ")));
        prop_assert_eq!(code, 0, "{}", stderr);
        let table = table_rows(&stdout);
        prop_assert_eq!(table.len(), columns.len());
        for (i, c) in columns.iter().enumerate() {
            let row = &table[i];
            let present: Vec<i64> = c.iter().flatten().copied().collect();
            prop_assert_eq!(row[0].clone(), format!("c{i}"));
            let n: usize = row[1].parse().unwrap();
            let missing: usize = row[2].parse().unwrap();
            prop_assert_eq!(n + missing, rows);
            prop_assert_eq!(n, present.len());
            let exact_mean = present.iter().sum::<i64>() as f64 / present.len() as f64;
            let mean: f64 = row[3].parse().unwrap();
            prop_assert!((mean - exact_mean).abs() <= 0.0005 + 1e-9, "mean {} for {}", row[3], exact_mean);
            let min: f64 = row[5].parse().unwrap();
            let max: f64 = row[6].parse().unwrap();
            prop_assert_eq!(min, *present.iter().min().unwrap() as f64);
            prop_assert_eq!(max, *present.iter().max().unwrap() as f64);
            prop_assert!(min <= mean && mean <= max);
            if present.len() == 1 {
                prop_assert_eq!(row[4].as_str(), "NA");
            } else if present.iter().all(|&v| v == present[0]) {
                prop_assert_eq!(row[4].as_str(), "0.000");
            } else {
                prop_assert!(row[4].parse::<f64>().unwrap() > 0.0);
            }
        }
    }
}

// ----- every message -----

#[test]
fn every_message_summary_can_give() {
    let cases = [
        (
            "rows_skipped",
            "let w = import(\"data/wages.csv\")\nw.summary()\n",
        ),
        (
            "all_missing",
            "let d = import(\"data/all_na.csv\", types: {x: Float})\nd.summary([x])\n",
        ),
        ("one_value", "frame(b = [1, NA]).summary()\n"),
        (
            "no_values",
            "let e = import(\"data/empty.csv\", types: {x: Float})\ne.summary()\n",
        ),
        (
            "infinities",
            "let d = import(\"data/infinities.csv\")\nd.summary([plus, both])\n",
        ),
        (
            "no_such_variable",
            "let w = import(\"data/wages.csv\")\nw.summary([wage, hour])\n",
        ),
        (
            "case_differs",
            "let w = import(\"data/wages.csv\")\nw.summary(Wage)\n",
        ),
        (
            "text_variable",
            "let w = import(\"data/wages.csv\")\nw.summary([name])\n",
        ),
        (
            "unknown_statistic",
            "let w = import(\"data/wages.csv\")\nw.summary(stats: [mean, median])\n",
        ),
        (
            "statistic_in_quotes",
            "let w = import(\"data/wages.csv\")\nw.summary(stats: [\"mean\"])\n",
        ),
        (
            "empty_stats",
            "let w = import(\"data/wages.csv\")\nw.summary(stats: [])\n",
        ),
        (
            "stats_not_a_list",
            "let w = import(\"data/wages.csv\")\nw.summary(stats: mean)\n",
        ),
        (
            "vars_not_variables",
            "let w = import(\"data/wages.csv\")\nw.summary([1, 2])\n",
        ),
        ("not_a_frame", "summary(3)\n"),
        (
            "unknown_option",
            "let w = import(\"data/wages.csv\")\nw.summary(by: id)\n",
        ),
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
    // A summary is a table: it prints through `print` and refuses a frame method.
    let outcome = run_with(
        &[],
        "let w = import(\"data/wages.csv\")\nlet s = w.summary([wage])\nprint(s)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stdout.starts_with("summary · w\n"),
        "{}",
        outcome.output.stdout
    );
    // The check stops the run before anything prints.
    let outcome = run("let w = import(\"data/wages.csv\")\nw.summary([wage]).mean(wage)\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .output
            .stderr
            .contains("error[Z0401]: `mean` cannot be called on a Table"),
        "{}",
        outcome.output.stderr
    );
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
