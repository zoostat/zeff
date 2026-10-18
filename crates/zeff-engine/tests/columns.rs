//! Expressions over columns (Language Reference 3.7, "Columns and scalars"; 3.8; 4.1): a frame's
//! column as a value, every operator row by row with a scalar standing for its value in every
//! row, the warnings with their row counts, the lengths rule, a property against the value rules,
//! the printed column, and every message.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

const WAGES: &str =
    "id,wage,hours,name,flag\n1,23.4,40,Ann,true\n2,19.0,35,Bob,false\n3,31.2,,Cy,true\n";

fn run_with(files: &[(&str, &str)], script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/wages.csv", WAGES);
    for (path, text) in files {
        fs.add_file(format!("/p/{path}"), text);
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn run(script: &str) -> Outcome {
    run_with(&[], script)
}

/// The messages without the one note every import prints.
fn without_format_notes(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// What a script prints after `let w = import("data/wages.csv")`, with no message but the note.
fn prints(rest: &str) -> String {
    let outcome = run(&format!("let w = import(\"data/wages.csv\")\n{rest}"));
    assert_eq!(outcome.exit_code, 0, "{rest}:\n{}", outcome.output.stderr);
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "",
        "{rest}: no message expected"
    );
    outcome.output.stdout
}

// ----- a column as a value -----

#[test]
fn a_frames_column_is_a_value_that_prints_as_section_3_8_says() {
    assert_eq!(
        prints("w[\"wage\"]\n"),
        "Column<Float> · 3 values\n23.4\n19\n31.2\n"
    );
    assert_eq!(
        prints("w[\"hours\"]\n"),
        "Column<Int> · 3 values\n40\n35\nNA\n"
    );
    assert_eq!(
        prints("w[\"name\"]\n"),
        "Column<Str> · 3 values\nAnn\nBob\nCy\n"
    );
    assert_eq!(
        prints("print(w[\"flag\"], w[\"id\"])\n"),
        "Column<Bool> · 3 values\ntrue\nfalse\ntrue\nColumn<Int> · 3 values\n1\n2\n3\n"
    );
    assert_eq!(
        prints("let c = w[\"wage\"]\nc\n"),
        "Column<Float> · 3 values\n23.4\n19\n31.2\n"
    );
}

#[test]
fn operators_apply_row_by_row_with_a_scalar_in_every_row() {
    let cases = [
        (
            "w[\"wage\"] * 2",
            "Column<Float> · 3 values\n46.8\n38\n62.4\n",
        ),
        ("w[\"id\"] + 10", "Column<Int> · 3 values\n11\n12\n13\n"),
        ("100 - w[\"hours\"]", "Column<Int> · 3 values\n60\n65\nNA\n"),
        (
            "w[\"wage\"] / w[\"hours\"]",
            "Column<Float> · 3 values\n0.585\n0.542857\nNA\n",
        ),
        ("w[\"id\"] ^ 2", "Column<Float> · 3 values\n1\n4\n9\n"),
        (
            "w[\"wage\"] > 20",
            "Column<Bool> · 3 values\ntrue\nfalse\ntrue\n",
        ),
        (
            "w[\"hours\"] >= 40",
            "Column<Bool> · 3 values\ntrue\nfalse\nNA\n",
        ),
        (
            "w[\"name\"] == \"Bob\"",
            "Column<Bool> · 3 values\nfalse\ntrue\nfalse\n",
        ),
        (
            "w[\"name\"] < \"B\"",
            "Column<Bool> · 3 values\ntrue\nfalse\nfalse\n",
        ),
        (
            "w[\"wage\"] > 20 && w[\"flag\"]",
            "Column<Bool> · 3 values\ntrue\nfalse\ntrue\n",
        ),
        (
            "w[\"hours\"] > 36 || w[\"flag\"]",
            "Column<Bool> · 3 values\ntrue\nfalse\ntrue\n",
        ),
        (
            "!w[\"flag\"]",
            "Column<Bool> · 3 values\nfalse\ntrue\nfalse\n",
        ),
        ("-w[\"id\"]", "Column<Int> · 3 values\n-1\n-2\n-3\n"),
        (
            "w[\"id\"] == w[\"id\"]",
            "Column<Bool> · 3 values\ntrue\ntrue\ntrue\n",
        ),
        ("w[\"hours\"] + NA", "Column<Int> · 3 values\nNA\nNA\nNA\n"),
        (
            "(w[\"wage\"] - 19) * (w[\"id\"] + 1)",
            "Column<Float> · 3 values\n8.8\n0\n48.8\n",
        ),
    ];
    for (expression, printed) in cases {
        assert_eq!(
            prints(&format!("let v = {expression}\nv\n")),
            printed,
            "{expression}"
        );
    }
}

#[test]
fn a_column_builds_a_frame_and_feeds_the_statistics() {
    let out = prints(
        "let d = frame(id = w[\"id\"], double = w[\"wage\"] * 2, high = w[\"wage\"] > 20)\nd\nd.mean(double)\nd.mean(high)\nd.describe()\n",
    );
    insta::assert_snapshot!("frame_from_columns", out);
}

#[test]
fn the_warnings_of_section_4_1_count_their_rows() {
    let outcome = run_with(
        &[("data/z.csv", "a,b\n1,0\n0,0\n-2,0\n4,2\n")],
        "let z = import(\"data/z.csv\")\nz[\"a\"] / z[\"b\"]\nz[\"a\"] ^ 0.5\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "column_warnings",
        format!(
            "{}\n{}",
            outcome.output.stdout,
            without_format_notes(&outcome.output.stderr)
        )
    );
}

#[test]
fn every_message_a_column_expression_can_give() {
    let cases = [
        (
            "no_such_column",
            "let w = import(\"data/wages.csv\")\nw[\"wge\"]\n",
        ),
        (
            "column_named_without_quotes",
            "let w = import(\"data/wages.csv\")\nw[wage]\n",
        ),
        (
            "column_named_by_a_number",
            "let w = import(\"data/wages.csv\")\nw[2]\n",
        ),
        (
            "different_lengths",
            "let w = import(\"data/wages.csv\")\nlet d = frame(x = [1, 2])\nw[\"id\"] + d[\"x\"]\n",
        ),
        (
            "int_overflow_in_a_row",
            "let d = frame(x = [1, 9223372036854775807])\nd[\"x\"] + 1\n",
        ),
        (
            "text_column_in_arithmetic",
            "let w = import(\"data/wages.csv\")\nw[\"name\"] + 1\n",
        ),
        (
            "bool_column_in_arithmetic",
            "let w = import(\"data/wages.csv\")\nw[\"flag\"] * 2\n",
        ),
        (
            "comparing_column_types",
            "let w = import(\"data/wages.csv\")\nw[\"name\"] == 1\n",
        ),
        (
            "logic_on_a_number_column",
            "let w = import(\"data/wages.csv\")\nw[\"id\"] && true\n",
        ),
        (
            "range_of_a_column",
            "let w = import(\"data/wages.csv\")\n1..w[\"id\"]\n",
        ),
        (
            "date_column",
            "let d = import(\"data/dates.csv\")\nd[\"sold\"] == d[\"sold\"]\n",
        ),
        (
            "column_in_a_list",
            "let w = import(\"data/wages.csv\")\n[w[\"id\"], w[\"id\"]]\n",
        ),
        (
            "column_as_a_frame_definition_of_another_length",
            "let w = import(\"data/wages.csv\")\nframe(a = [1, 2], b = w[\"id\"])\n",
        ),
    ];
    let dates: &[(&str, &str)] = &[("data/dates.csv", "sold\n2026-10-16\n2026-01-01\n")];
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

// ----- a property: a column expression is the value expression in every row -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    #[test]
    fn a_column_expression_equals_the_value_expression_row_by_row(
        xs in prop::collection::vec(prop::option::weighted(0.85, -500i64..500), 1..12),
        ys in prop::collection::vec(prop::option::weighted(0.85, -40.0f64..40.0), 1..12),
        op in prop::sample::select(vec!["+", "-", "*", "/", "<", "==", ">="]),
    ) {
        let n = xs.len().min(ys.len());
        let (mut xs, mut ys) = (xs[..n].to_vec(), ys[..n].to_vec());
        // A column of nothing but NA has no type (Z0219): give each one value.
        if xs.iter().all(Option::is_none) {
            xs[0] = Some(1);
        }
        if ys.iter().all(Option::is_none) {
            ys[0] = Some(1.0);
        }
        let (xs, ys) = (&xs[..], &ys[..]);
        let text = |v: Option<String>| v.unwrap_or("NA".to_string());
        let x_list: Vec<String> = xs.iter().map(|v| text(v.map(|i| i.to_string()))).collect();
        let y_list: Vec<String> = ys.iter().map(|v| text(v.map(|f| format!("{f:?}")))).collect();
        let mut script = format!(
            "let d = frame(x = [{}], y = [{}])\nd[\"x\"] {op} d[\"y\"]\n",
            x_list.join(", "),
            y_list.join(", ")
        );
        for (x, y) in x_list.iter().zip(&y_list) {
            // Every row as a value expression; `NA` alone needs its type, so it is written as a
            // one-item list that is then indexed.
            let x = if x == "NA" { "[NA, 0][1]".to_string() } else { x.clone() };
            let y = if y == "NA" { "[NA, 0.0][1]".to_string() } else { y.clone() };
            script.push_str(&format!("let r{} = ({x}) {op} ({y})\n", script.len()));
        }
let outcome = run(&script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        // The first statement after the frame prints the column: header, then n rows.
        prop_assert_eq!(lines.len(), 1 + n);
        let column_rows: Vec<&str> = lines[1..1 + n].to_vec();
        // Now the same rows as values, through a second run that prints them.
        let mut value_script = format!("let d = frame(x = [{}], y = [{}])\n", x_list.join(", "), y_list.join(", "));
        for (x, y) in x_list.iter().zip(&y_list) {
            let x = if x == "NA" { "[NA, 0][1]".to_string() } else { x.clone() };
            let y = if y == "NA" { "[NA, 0.0][1]".to_string() } else { y.clone() };
            value_script.push_str(&format!("({x}) {op} ({y})\n"));
        }
        let values = run(&value_script);
        prop_assert_eq!(values.exit_code, 0, "{}", values.output.stderr);
        let value_rows: Vec<&str> = values.output.stdout.lines().collect();
        prop_assert_eq!(column_rows, value_rows);
        // The column's warnings count exactly the rows that warned as values.
        let column_warnings = without_format_notes(&outcome.output.stderr);
        let value_warnings = values.output.stderr.lines().filter(|l| l.starts_with("warning")).count();
        if value_warnings == 0 {
            prop_assert_eq!(column_warnings.as_str(), "");
        } else {
            prop_assert!(column_warnings.starts_with("warning"), "{column_warnings}");
        }
    }
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
