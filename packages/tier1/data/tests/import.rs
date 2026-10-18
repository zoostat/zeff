//! `import`: the format note of section 6.1, the reference on every dataset through the function,
//! the edge-case datasets with the outcome each must give, every option, the path rules, a
//! property on random tables, and a snapshot of every message the function can give.

use proptest::prelude::*;
use zeff_core::display::format_float;
use zeff_diagnostics::count;
use zeff_testkit::{Accuracy, assert_prints, reference, root, run, run_with};

/// Exit code, standard output and messages of a script run in the project of every dataset.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The same, with files added to the project.
fn go_with(files: &[(&str, &[u8])], script: &str) -> (i32, String, String) {
    let outcome = run_with(files, script);
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

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", count(n as u64), if n == 1 { one } else { many })
}

/// The note of section 6.1 for a CSV file of `rows` rows and `cols` variables.
fn format_note(path: &str, rows: usize, cols: usize, line: usize) -> String {
    format!(
        "note: {path} read as CSV: {} × {}, types guessed; CSV carries no labels, value labels or categories (line {line})\n",
        plural(rows, "row", "rows"),
        plural(cols, "variable", "variables")
    )
}

/// What `n_rows()` and `n_cols()` print, one under the other.
fn shape(rows: usize, cols: usize) -> String {
    format!("{rows}\n{cols}\n")
}

// ----- the reference: every dataset through the function -----

#[test]
fn every_dataset_imports_and_the_note_gives_its_rows_and_variables() {
    let reference = reference();
    assert!(reference.datasets.len() >= 2, "the handbook asks for two");
    for (name, dataset) in &reference.datasets {
        let file = root().join(&dataset.file);
        let text = std::fs::read_to_string(&file).unwrap();
        let cols = text.lines().next().unwrap().split(',').count();
        let script = format!(
            "let d = {}\nd.n_rows()\nd.n_cols()\n",
            dataset.import_call("")
        );
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{name}:\n{stderr}");
        assert_eq!(stdout, shape(dataset.rows, cols), "{name}");
        assert!(
            stderr.starts_with(&format_note(&dataset.script_path(), dataset.rows, cols, 1)),
            "{name}: the format note comes first:\n{stderr}"
        );
        // The reference means through `import` itself, for every numeric column a script can name.
        for (column, stats) in &dataset.columns {
            let Some(written) = zeff_testkit::in_code(column) else {
                continue;
            };
            // A column with no value is text at import; the reference declares it Float.
            let types = if stats.n == 0 {
                format!(", types: {{{written}: Float}}")
            } else {
                String::new()
            };
            let script = format!(
                "let d = {}\nd.mean({written})\n",
                dataset.import_call(&types)
            );
            let outcome = run(&script);
            assert_eq!(
                outcome.exit_code, 0,
                "{name}.{column}:\n{}",
                outcome.output.stderr
            );
            assert_prints(
                &format!("mean of {name}.{column} after import"),
                &outcome.output.statements[1].stdout,
                &stats.mean,
                Accuracy::ClosedForm,
            );
        }
    }
}

#[test]
fn the_note_prints_once_per_import_under_the_statement_that_imports() {
    let (code, stdout, stderr) = go("print(import(\"data/wages.csv\"))\n");
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("printed_import", stdout);
    assert_eq!(stderr, format_note("data/wages.csv", 3, 4, 1));
    // The same file read twice is two imports and two notes; a bound frame used twice is one.
    let (_, _, stderr) = go(
        "import(\"data/wages.csv\").mean(wage)\nlet w = import(\"data/wages.csv\")\nw.mean(wage)\nw.mean(id)\n",
    );
    assert_eq!(
        stderr,
        format!(
            "{}{}",
            format_note("data/wages.csv", 3, 4, 1),
            format_note("data/wages.csv", 3, 4, 2)
        )
    );
}

// ----- the edge-case datasets -----

#[test]
fn a_file_of_one_row_and_a_file_of_none() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/one_row.csv\")\nd.n_rows()\nd.n_cols()\nd.mean(x)\nd.mean(flag)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, format!("{}42\n1\n", shape(1, 3)));
    assert_eq!(stderr, format_note("data/one_row.csv", 1, 3, 1));
    // A header alone is a frame of no rows; its columns have no values to type, so they are text.
    let (code, stdout, stderr) =
        go("let d = import(\"data/empty.csv\")\nd.n_rows()\nd.n_cols()\nd.mean(x)\n");
    assert_eq!(code, 1);
    assert_eq!(stdout, shape(0, 2));
    assert!(
        without_format_notes(&stderr)
            .starts_with("error[Z0401]: `mean` needs a Bool, Int or Float, but `x` is Str"),
        "{stderr}"
    );
    insta::assert_snapshot!("empty_file", stderr);
}

#[test]
fn a_file_whose_columns_hold_only_missing_values() {
    let (code, stdout, stderr) =
        go("let d = import(\"data/all_na.csv\")\nd.n_rows()\nd.n_cols()\nd.mean(y)\n");
    assert_eq!(code, 1);
    assert_eq!(stdout, shape(3, 3));
    assert!(
        without_format_notes(&stderr)
            .starts_with("error[Z0401]: `mean` needs a Bool, Int or Float, but `y` is Str"),
        "{stderr}"
    );
    // Declared Float, the column reads and its mean is NA with the warning.
    let (code, stdout, stderr) =
        go("let d = import(\"data/all_na.csv\", types: {y: Float})\nd.mean(y)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\n");
    assert_eq!(
        without_format_notes(&stderr),
        "warning: all 3 rows have NA in y: the result is NA (line 2)\n"
    );
}

#[test]
fn the_guess_boundary_file_needs_its_declaration() {
    let (code, stdout, stderr) = go("let d = import(\"data/guess_boundary.csv\")\n");
    assert_eq!(code, 1);
    assert_eq!(stdout, "");
    assert!(
        stderr.starts_with(
            "error[Z0606]: row 1,002 of `data/guess_boundary.csv` has text in a numeric column"
        ),
        "{stderr}"
    );
    insta::assert_snapshot!("guess_boundary_undeclared", stderr);
    let (code, stdout, stderr) = go(
        "let d = import(\"data/guess_boundary.csv\", types: {hours: Str})\nd.n_rows()\nd.n_cols()\nd.mean(id)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, format!("{}501\n", shape(1001, 2)));
    assert_eq!(stderr, format_note("data/guess_boundary.csv", 1001, 2, 1));
}

#[test]
fn infinities_and_nan_in_a_file() {
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nd.mean(plus)\nd.mean(minus)\nd.mean(both)\nd.mean(nan_mixed)\nd.mean(finite)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "Inf\n-Inf\nNA\n1.5\n2\n");
    insta::assert_snapshot!("infinities_messages", stderr);
}

#[test]
fn bool_columns_in_every_spelling() {
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
fn unicode_text_and_awkward_names_are_kept_as_the_file_has_them() {
    let (code, stdout, stderr) =
        go("let d = import(\"data/unicode_labels.csv\")\nd.n_rows()\nd.n_cols()\nd.mean(n)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, format!("{}2.5\n", shape(4, 3)));
    // The file holds "café" in both normalisation forms on purpose: the warning of section 3.8.
    assert!(
        without_format_notes(&stderr).starts_with(
            "warning: city holds values that differ only in their Unicode normalisation form"
        ),
        "{stderr}"
    );
    insta::assert_snapshot!("unicode_labels", stderr);
    let (code, stdout, stderr) = go(
        "let d = import(\"data/awkward_names.csv\")\nd.n_rows()\nd.n_cols()\nd.mean(`Gross Sales (total)`)\nd.mean(`x y`)\nd.mean(`1st`)\nd.mean(`日本語`)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, format!("{}1226.92\n2\n5\n4\n", shape(3, 7)));
    assert_eq!(stderr, format_note("data/awkward_names.csv", 3, 7, 1));
    // `names: snake` makes every name plain and says what it changed.
    let (code, stdout, stderr) =
        go("let d = import(\"data/awkward_names.csv\", names: snake)\nd.mean(gross_sales_total)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "1226.92\n");
    insta::assert_snapshot!("awkward_names_snake", stderr);
}

#[test]
fn the_large_and_the_degenerate_datasets_read_with_the_note_alone() {
    for (file, rows, cols) in [
        ("wages_large.csv", 1200, 6),
        ("magnitudes.csv", 500, 3),
        ("zero_variance.csv", 4, 3),
    ] {
        let (code, stdout, stderr) = go(&format!(
            "let d = import(\"data/{file}\")\nd.n_rows()\nd.n_cols()\n"
        ));
        assert_eq!(code, 0, "{file}: {stderr}");
        assert_eq!(stdout, shape(rows, cols), "{file}");
        assert_eq!(
            stderr,
            format_note(&format!("data/{file}"), rows, cols, 1),
            "{file}"
        );
    }
}

// ----- the options, each on its own -----

#[test]
fn every_option_of_this_release() {
    let csv = b"# produced 2026\nID;Gross Wage;Sold;flag\n1;-99;16/10/2026;true\n2;19,5;04/05/2026;false\n3;31;;true\n";
    let files: &[(&str, &[u8])] = &[("data/odd.csv", csv)];
    // The delimiter is detected from the header and noted; `delim:` declares it.
    let (code, _, stderr) = go_with(
        files,
        "let d = import(\"data/odd.csv\", skip: 1)\nd.mean(ID)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("delimiter_detected", stderr);
    let (code, stdout, stderr) = go_with(
        files,
        "let d = import(\"data/odd.csv\", delim: \";\", skip: 1, na_values: [-99], types: {Sold: Date(\"%d/%m/%Y\"), flag: Bool}, names: snake)\nd.n_rows()\nd.n_cols()\nd.mean(flag)\nd.mean(gross_wage)\n",
    );
    // `19,5` is text, so the column is Str and its mean stops the run.
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(stdout, format!("{}0.666667\n", shape(3, 4)));
    assert!(
        without_format_notes(&stderr)
            .contains("error[Z0401]: `mean` needs a Bool, Int or Float, but `gross_wage` is Str"),
        "{stderr}"
    );
    insta::assert_snapshot!("every_option", stderr);
    // `header: false` names the columns v1, v2, ...
    let plain: &[(&str, &[u8])] = &[("data/plain.csv", b"1,2\n3,4\n")];
    let (code, stdout, stderr) = go_with(
        plain,
        "let d = import(\"data/plain.csv\", header: false)\nd.n_rows()\nd.n_cols()\nd.mean(v1)\nd.mean(v2)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, format!("{}2\n3\n", shape(2, 2)));
    // `na_values:` takes numbers or text, one type per list (section 3.7).
    let coded: &[(&str, &[u8])] = &[("data/coded.csv", b"x,y\n1,a\n-99,none\n3,c\n")];
    let (code, stdout, stderr) = go_with(
        coded,
        "let d = import(\"data/coded.csv\", na_values: [-99])\nd.mean(x)\nlet e = import(\"data/coded.csv\", na_values: [\"none\"])\ne.mean(x)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "2\n-31.6667\n");
    insta::assert_snapshot!("na_values", stderr);
}

// ----- paths -----

#[test]
fn paths_are_resolved_from_the_project_root_and_stay_inside_it() {
    let cases = [
        ("missing", "let d = import(\"data/missing.csv\")\n", "Z0715"),
        (
            "outside",
            "let d = import(\"../shared/wages.csv\")\n",
            "Z0701",
        ),
        ("absolute", "let d = import(\"/etc/passwd\")\n", "Z0701"),
        ("case", "let d = import(\"data/Wages.csv\")\n", "Z0702"),
        ("folder", "let d = import(\"data\")\n", "Z0715"),
    ];
    let mut all = String::new();
    for (name, script, code) in cases {
        let (exit, stdout, stderr) = go(script);
        assert_eq!(exit, 1, "{name}");
        assert_eq!(stdout, "", "{name}");
        assert!(
            stderr.starts_with(&format!("error[{code}]")),
            "{name}:\n{stderr}"
        );
        all.push_str(&format!("## {name}\n{stderr}\n"));
    }
    insta::assert_snapshot!("path_errors", all);
}

// ----- every message -----

#[test]
fn every_message_import_can_give() {
    let cases: Vec<(&str, &[u8], &str)> = vec![
        (
            "bom_removed",
            b"\xEF\xBB\xBFx,y\n1,2\n",
            "let d = import(\"data/f.csv\")\nd.mean(x)\n",
        ),
        (
            "blank_lines_skipped",
            b"x,y\n1,2\n\n\n3,4\n",
            "let d = import(\"data/f.csv\")\nd.n_rows()\nd.n_cols()\n",
        ),
        (
            "nan_becomes_na",
            b"x\n1\nNaN\nnan\n",
            "let d = import(\"data/f.csv\")\nd.mean(x)\n",
        ),
        (
            "date_like_text_proposes_types",
            b"sold\n16/10/2026\n04/05/2026\n01/01/2026\n",
            "let d = import(\"data/f.csv\")\nd.n_rows()\nd.n_cols()\n",
        ),
        (
            "datetime_offsets_to_utc",
            b"at\n2026-10-16T10:00:00+02:00\n2026-10-16T09:00:00Z\n",
            "let d = import(\"data/f.csv\")\nd.n_rows()\nd.n_cols()\n",
        ),
        (
            "normalisation_differs",
            "x\ncaf\u{e9}\ncafe\u{301}\n".as_bytes(),
            "let d = import(\"data/f.csv\")\nd.n_rows()\nd.n_cols()\n",
        ),
        ("no_header", b"", "let d = import(\"data/f.csv\")\n"),
        (
            "empty_name",
            b"x,,z\n1,2,3\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "repeated_name",
            b"x,y,x\n1,2,3\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "case_clash",
            b"Wage,wage\n1,2\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "snake_clash",
            b"Gross Sales,gross_sales\n1,2\n",
            "let d = import(\"data/f.csv\", names: snake)\n",
        ),
        (
            "field_count",
            b"x,y\n1,2\n3\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "unclosed_quote",
            b"x,y\n1,\"two\n3,4\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "not_utf8",
            b"x,y\n1,2\n3,\xFF\n",
            "let d = import(\"data/f.csv\")\n",
        ),
        (
            "declared_type_does_not_fit",
            b"x\n1\ntwo\n",
            "let d = import(\"data/f.csv\", types: {x: Int})\n",
        ),
        (
            "types_names_a_missing_column",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", types: {y: Int})\n",
        ),
        (
            "bad_delim",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", delim: \"ab\")\n",
        ),
        (
            "bad_skip",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", skip: -1)\n",
        ),
        (
            "bad_types_value",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", types: {x: 1})\n",
        ),
        (
            "bad_na_values",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", na_values: [true])\n",
        ),
        (
            "bad_date_format",
            b"x\n1\n",
            "let d = import(\"data/f.csv\", types: {x: Date(\"%Q\")})\n",
        ),
    ];
    let mut all = String::new();
    for (name, bytes, script) in cases {
        let (code, stdout, stderr) = go_with(&[("data/f.csv", bytes)], script);
        all.push_str(&format!("## {name} (exit {code})\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", all);
}

// ----- a property: random tables come back with their shape and their means -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]
    #[test]
    fn a_random_table_of_integers_imports_with_its_shape_and_means(
        // Two columns at least: a one-column row with no value is a blank line, which is skipped.
        table in (2usize..6).prop_flat_map(|cols| {
            (Just(cols), prop::collection::vec(prop::collection::vec(prop::option::weighted(0.8, -1000i64..1000), cols), 1..40))
        })
    ) {
        let (cols, rows) = table;
        let mut csv = (0..cols).map(|c| format!("c{c}")).collect::<Vec<_>>().join(",");
        csv.push('\n');
        for row in &rows {
            let line: Vec<String> = row.iter().map(|v| v.map(|x| x.to_string()).unwrap_or_default()).collect();
            csv.push_str(&line.join(","));
            csv.push('\n');
        }
        // A column with no value at all is text at import (section 3.8), so only the others have a mean.
        let numeric: Vec<usize> = (0..cols).filter(|&c| rows.iter().any(|r| r[c].is_some())).collect();
        let mut script = String::from("let d = import(\"data/r.csv\")\nd.n_rows()\nd.n_cols()\n");
        for c in &numeric {
            script.push_str(&format!("d.mean(c{c})\n"));
        }
        let outcome = run_with(&[("data/r.csv", csv.as_bytes())], &script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let statements = &outcome.output.statements;
        prop_assert_eq!(format!("{}{}", statements[1].stdout, statements[2].stdout), shape(rows.len(), cols));
        for (i, &c) in numeric.iter().enumerate() {
            let present: Vec<i64> = rows.iter().filter_map(|r| r[c]).collect();
            let want = format_float(present.iter().sum::<i64>() as f64 / present.len() as f64);
            prop_assert_eq!(statements[3 + i].stdout.trim(), want.as_str(), "column c{}", c);
            let skipped = rows.len() - present.len();
            if skipped > 0 {
                prop_assert!(
                    statements[3 + i].stderr.contains(&format!("{} with NA in c{c}", plural(skipped, "row", "rows"))),
                    "{}", statements[3 + i].stderr
                );
            } else {
                prop_assert_eq!(statements[3 + i].stderr.as_str(), "");
            }
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
