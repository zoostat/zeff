//! `n_rows`, `n_cols` and `variables`: the reference on every dataset, the edge-case files, frames
//! built in the script, a property on random frames, the printed list, and every message the
//! checker gives for a call that is wrong.

use proptest::prelude::*;
use zeff_testkit::{reference, root, run, run_with};

/// Exit code, standard output and messages of a script run in the project of every dataset.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

// ----- the reference: every dataset -----

#[test]
fn every_dataset_gives_its_rows_its_variables_and_their_names() {
    let reference = reference();
    assert!(reference.datasets.len() >= 2, "the handbook asks for two");
    for (name, dataset) in &reference.datasets {
        let text = std::fs::read_to_string(root().join(&dataset.file)).unwrap();
        let header: Vec<&str> = text.lines().next().unwrap().split(',').collect();
        let script = format!(
            "let d = {}\nd.n_rows()\nd.n_cols()\nd.variables()\n",
            dataset.import_call("")
        );
        let outcome = run(&script);
        assert_eq!(outcome.exit_code, 0, "{name}:\n{}", outcome.output.stderr);
        let mut want = format!("{}\n{}\n", dataset.rows, header.len());
        for column in &header {
            want.push_str(column);
            want.push('\n');
        }
        assert_eq!(outcome.output.stdout, want, "{name}");
        // The import may have notes of its own; the three methods have none.
        for statement in &outcome.output.statements[1..] {
            assert_eq!(statement.stderr, "", "{name}: no message of their own");
        }
    }
}

// ----- edge cases -----

#[test]
fn a_frame_of_no_rows_and_a_frame_of_one() {
    let (code, stdout, _) =
        go("let d = import(\"data/empty.csv\")\nd.n_rows()\nd.n_cols()\nd.variables()\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "0\n2\nx\ny\n");
    let (code, stdout, _) = go("let d = import(\"data/one_row.csv\")\nd.n_rows()\nd.n_cols()\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "1\n3\n");
}

#[test]
fn names_are_the_frames_own() {
    // As the file has them, awkward or not; as `names: snake` made them; as `frame` was given them.
    let (code, stdout, _) = go("import(\"data/awkward_names.csv\").variables()\n");
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "Gross Sales (total)\nx y\n`case`\n日本語\n1st\nWage\nwage_2\n"
    );
    let (code, stdout, _) = go("import(\"data/awkward_names.csv\", names: snake).variables()\n");
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "gross_sales_total\nx_y\ncase\nv4\nx1st\nwage\nwage_2\n"
    );
    let (code, stdout, stderr) = go(
        "let d = frame(wage = [23.4, 19.0], `Gross Sales` = [1, 2])\nd.n_rows()\nd.n_cols()\nd.variables()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "2\n2\nwage\nGross Sales\n");
    assert_eq!(stderr, "");
}

#[test]
fn rows_with_missing_values_count_like_any_other() {
    let (code, stdout, _) = go("let d = import(\"data/all_na.csv\")\nd.n_rows()\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn the_results_are_values_a_script_can_use() {
    let (code, stdout, stderr) = go(
        "let wages = import(\"data/wages.csv\")\nlet n = wages.n_rows()\nprint(n, wages.n_cols(), wages.variables())\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "3\n4\nid\nwage\nhours\nname\n");
}

// ----- the printed list -----

#[test]
fn a_long_list_of_variables_prints_twenty_then_the_count() {
    let columns: Vec<String> = (1..=23).map(|i| format!("c{i} = [1]")).collect();
    let (code, stdout, stderr) = go(&format!("frame({}).variables()\n", columns.join(", ")));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("long_variables", stdout);
}

// ----- every message a wrong call gives -----

#[test]
fn every_message_a_wrong_call_gives() {
    let cases = [
        ("not_a_frame", "n_rows(3)\n"),
        ("no_frame", "n_cols()\n"),
        (
            "an_argument_too_many",
            "let w = import(\"data/wages.csv\")\nw.variables(wage)\n",
        ),
        (
            "an_option",
            "let w = import(\"data/wages.csv\")\nw.n_rows(by: id)\n",
        ),
        ("not_a_method_of_an_int", "let n = 3\nn.n_rows()\n"),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{name}:\n{stderr}");
        all.push_str(&format!("## {name}\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("wrong_calls", all);
}

// ----- a property: a random frame knows its shape and its names -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn a_random_frame_reports_its_rows_columns_and_names(
        cols in 1usize..8,
        rows in 1usize..30,
        seed in any::<u64>(),
    ) {
        let names: Vec<String> = (0..cols).map(|c| format!("v{}_{c}", seed % 1000)).collect();
        let definitions: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(c, name)| {
                let values: Vec<String> = (0..rows).map(|r| ((r * 7 + c) % 11).to_string()).collect();
                format!("{name} = [{}]", values.join(", "))
            })
            .collect();
        let script = format!(
            "let d = frame({})\nd.n_rows()\nd.n_cols()\nd.variables()\n",
            definitions.join(", ")
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let mut want = format!("{rows}\n{cols}\n");
        for name in &names {
            want.push_str(name);
            want.push('\n');
        }
        prop_assert_eq!(stdout, want);
        prop_assert_eq!(stderr, "");
    }
}

// ----- describe -----

#[test]
fn describe_counts_every_dataset_as_the_reference_does() {
    let reference = reference();
    for (name, dataset) in &reference.datasets {
        let script = format!("let d = {}\nd.describe()\n", dataset.import_call(""));
        let outcome = run(&script);
        assert_eq!(outcome.exit_code, 0, "{name}:\n{}", outcome.output.stderr);
        let text = &outcome.output.statements[1].stdout;
        let mut lines = text.lines();
        assert!(
            lines.next().unwrap().starts_with("d · "),
            "{name}: the title is the frame's header:\n{text}"
        );
        assert_eq!(
            lines.next(),
            Some(""),
            "{name}: a blank line after the title"
        );
        assert_eq!(
            lines.next().unwrap().split_whitespace().collect::<Vec<_>>(),
            ["variable", "type", "non-missing", "missing", "example"],
            "{name}"
        );
        let rows: Vec<Vec<&str>> = lines.map(|l| l.split_whitespace().collect()).collect();
        let file = std::fs::read_to_string(root().join(&dataset.file)).unwrap();
        let header: Vec<&str> = file.lines().next().unwrap().split(',').collect();
        assert_eq!(rows.len(), header.len(), "{name}: one row per variable");
        // The counts of the numeric columns are the reference's; names with spaces split, so
        // the counts are read from the end of the row.
        for (column, stats) in &dataset.columns {
            let Some(row) = rows
                .iter()
                .find(|r| r.join(" ").starts_with(&format!("{column} ")))
            else {
                panic!("{name}: no row for {column}:\n{text}");
            };
            let n = row.len();
            // ... example, missing, non-missing, type ... from the right: the example may hold
            // spaces only for text, and these columns are numeric.
            assert_eq!(
                (row[n - 3], row[n - 2]),
                (
                    zeff_diagnostics::count(stats.n as u64).as_str(),
                    zeff_diagnostics::count(stats.missing as u64).as_str()
                ),
                "{name}.{column}:\n{text}"
            );
        }
        assert_eq!(outcome.output.statements[1].stderr, "", "{name}");
    }
}

#[test]
fn describe_on_the_edge_cases_and_the_printed_frames() {
    let scripts = [
        ("wages", "import(\"data/wages.csv\").describe()\n"),
        ("all_na", "import(\"data/all_na.csv\").describe()\n"),
        ("empty", "import(\"data/empty.csv\").describe()\n"),
        ("one_row", "import(\"data/one_row.csv\").describe()\n"),
        (
            "bools_and_text",
            "import(\"data/unicode_labels.csv\").describe()\n",
        ),
        ("infinities", "import(\"data/infinities.csv\").describe()\n"),
        (
            "awkward_names",
            "import(\"data/awkward_names.csv\").describe()\n",
        ),
        ("large", "import(\"data/wages_large.csv\").describe()\n"),
        ("printed_wages", "import(\"data/wages.csv\")\n"),
        (
            "printed_large",
            "let w = import(\"data/wages_large.csv\")\nw\n",
        ),
        ("printed_empty", "let e = import(\"data/empty.csv\")\ne\n"),
        ("printed_one_row", "import(\"data/one_row.csv\")\n"),
        ("printed_awkward", "import(\"data/awkward_names.csv\")\n"),
        ("printed_magnitudes", "import(\"data/magnitudes.csv\")\n"),
        (
            "printed_wide",
            &format!(
                "frame({}).describe()\nframe({})\n",
                (1..=30)
                    .map(|c| format!("variable_number_{c:02} = [{c}, {}]", c * 1000))
                    .collect::<Vec<_>>()
                    .join(", "),
                (1..=30)
                    .map(|c| format!("variable_number_{c:02} = [{c}, {}]", c * 1000))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        (
            "printed_through_print",
            "let d = import(\"data/wages.csv\")\nprint(d.describe(), d, 1)\n",
        ),
    ];
    let mut all = String::new();
    // Dates, large and small numbers, and Bool with NA, from a file (no date function yet).
    let dates: &[(&str, &[u8])] = &[(
        "data/dates.csv",
        b"sold,big,flag\n2026-10-16,1000000.125,true\n,1e15,\n2026-01-01,0.00003,false\n",
    )];
    let outcome = run_with(
        dates,
        "let d = import(\"data/dates.csv\")\nd.describe()\nd\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    all.push_str(&format!(
        "## dates_and_large_numbers\n{}\n",
        outcome.output.stdout
    ));
    for (name, script) in scripts {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 0, "{name}:\n{stderr}");
        for line in stdout.lines() {
            assert!(
                line.chars().count() <= 118,
                "{name}: a line wider than the page:\n{line}"
            );
        }
        all.push_str(&format!("## {name}\n{stdout}\n"));
    }
    insta::assert_snapshot!("describe_and_printed_frames", all);
}

#[test]
fn describe_is_a_table_value() {
    let (code, stdout, stderr) =
        go("let d = import(\"data/wages.csv\")\nlet t = d.describe()\nprint(t)\nt\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout.matches("variable   type").count(), 2, "{stdout}");
    let (code, _, stderr) = go("let d = import(\"data/wages.csv\")\nd.describe().mean(x)\n");
    assert_eq!(code, 1);
    insta::assert_snapshot!("describe_is_not_a_frame", stderr);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    #[test]
    fn describe_of_a_random_frame_counts_its_values(
        columns in prop::collection::vec(prop::collection::vec(prop::option::weighted(0.7, -50i64..50), 1..25), 1..6)
    ) {
        let rows = columns[0].len();
        let columns: Vec<Vec<Option<i64>>> = columns.into_iter().map(|c| {
            let mut c = c;
            c.resize(rows, Some(1));
            c
        }).collect();
        // Every column needs a value: an all-NA list has no type.
        let columns: Vec<Vec<Option<i64>>> = columns.into_iter().map(|mut c| {
            if c.iter().all(Option::is_none) { c[0] = Some(7); }
            c
        }).collect();
        let definitions: Vec<String> = columns.iter().enumerate().map(|(i, c)| {
            let values: Vec<String> = c.iter().map(|v| v.map_or("NA".to_string(), |x| x.to_string())).collect();
            format!("c{i} = [{}]", values.join(", "))
        }).collect();
        let (code, stdout, stderr) = go(&format!("frame({}).describe()\n", definitions.join(", ")));
        prop_assert_eq!(code, 0, "{}", stderr);
        let lines: Vec<&str> = stdout.lines().collect();
        prop_assert_eq!(lines.len(), 3 + columns.len());
        for (i, c) in columns.iter().enumerate() {
            let cells: Vec<String> = lines[3 + i].split_whitespace().map(str::to_string).collect();
            let present = c.iter().filter(|v| v.is_some()).count();
            let first = c.iter().flatten().next().unwrap().to_string();
            let want = vec![format!("c{i}"), "Int".to_string(), present.to_string(), (rows - present).to_string(), first];
            prop_assert_eq!(cells, want);
        }
    }
}
