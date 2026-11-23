//! `row(vars, stat: n_missing)` (release 0.9.0, step 31c; open decision #166): the number of
//! the variables whose value is missing in each row. Against R's `rowSums(is.na())`
//! (`tests/reference/r/rows.json`) on the four numeric matrices of `row`, on a frame of
//! numbers, text, logical values and dates, and on the shipped wages; the edge cases (nothing
//! missing, everything missing, one variable, an empty text, which is not missing, an empty
//! frame); `min_valid:`, which changes nothing and says so; that `mean` and `sum` are as they
//! were; every message; and the properties that the count lies between 0 and the number of
//! variables, is the sum of the counts of two halves, does not depend on the order of the
//! variables, and that a row with no value missing is a row `drop_missing` keeps.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The lines a script prints, which must run clean.
fn printed(script: &str) -> Vec<String> {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    assert_eq!(stderr, "", "{script}");
    stdout.lines().map(str::to_string).collect()
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/rows.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// A column of whole numbers as it prints: its heading, then a value to a line.
fn column(values: &Json) -> Vec<String> {
    let values = values.as_array().unwrap();
    let mut lines = vec![format!("Column<Int> · {} values", values.len())];
    lines.extend(values.iter().map(|v| v.as_i64().unwrap().to_string()));
    lines
}

// ----- against R -----

#[test]
fn the_numeric_matrices_agree_with_r() {
    let r = reference();
    let cases = r["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for case in cases {
        let mut names = Vec::new();
        let columns: Vec<String> = case["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                let name = c["name"].as_str().unwrap().to_string();
                names.push(name.clone());
                format!("{name} = [{}]", strings(&c["values"]).join(", "))
            })
            .collect();
        let vars = format!("[{}]", names.join(", "));
        let lines = printed(&format!(
            "frame({}).add(gaps = row({vars}, stat: n_missing))[\"gaps\"]\n",
            columns.join(", ")
        ));
        assert_eq!(lines, column(&case["row_n_missing"]), "{vars}");
    }
}

/// The frame of the reference with five types, as a script writes it.
fn mixed_frame(mixed: &Json) -> String {
    let list = |key: &str, wrap: &dyn Fn(&str) -> String| -> String {
        strings(&mixed[key])
            .iter()
            .map(|v| if v == "NA" { "NA".to_string() } else { wrap(v) })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let plain = |v: &str| v.to_string();
    let quoted = |v: &str| format!("\"{v}\"");
    let dated = |v: &str| format!("date(\"{v}\")");
    format!(
        "frame(q1 = [{}], q2 = [{}], name = [{}], ok = [{}], day = [{}])",
        list("q1", &plain),
        list("q2", &plain),
        list("name", &quoted),
        list("ok", &plain),
        list("day", &dated)
    )
}

#[test]
fn variables_of_five_types_agree_with_r() {
    let r = reference();
    let mixed = &r["n_missing"]["mixed"];
    let lines = printed(&format!(
        "let d = {}\nlet e = d.add(all = row([q1, q2, name, ok, day], stat: n_missing), ranged = row([q1..day], stat: n_missing), numbers = row([q1, q2], stat: n_missing), others = row([name, day], stat: n_missing), one = row(ok, stat: n_missing))\ne[\"all\"]\ne[\"ranged\"]\ne[\"numbers\"]\ne[\"others\"]\ne[\"one\"]\nprint(type_of(e[\"all\"]))\n",
        mixed_frame(mixed)
    ));
    let mut expected = column(&mixed["all"]);
    // The variables as a range by position are the same variables.
    expected.extend(column(&mixed["all"]));
    expected.extend(column(&mixed["numbers"]));
    expected.extend(column(&mixed["text_and_dates"]));
    expected.extend(column(&mixed["one"]));
    expected.push("Column<Int>".to_string());
    assert_eq!(lines, expected);
    // The case holds a row with nothing missing, one with everything, and an empty text,
    // which is a value: the fifth row misses `ok` alone.
    let all: Vec<i64> = mixed["all"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert_eq!(all, [0, 4, 3, 2, 1, 5]);
    assert_eq!(strings(&mixed["name"])[4], "");
}

#[test]
fn the_shipped_wages_agree_with_r() {
    let r = reference();
    let wages = &r["n_missing"]["wages"];
    let whole = |v: &Json| v.as_i64().unwrap().to_string();
    assert_eq!(strings(&wages["variables"]), ["wage", "hours", "industry"]);
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).add(gaps = row([wage, hours, industry], stat: n_missing), every = row([worker..firm], stat: n_missing))\nprint(w.n_rows())\nprint(w.sum(gaps))\nfor k in 0..3 { print(w.where(gaps == k).n_rows()) }\nprint(w.sum(every))\nprint(w.where(every == 0).n_rows())\nprint(w.where(row([worker..firm], stat: n_missing) == 0).n_rows())\nprint(w.max(gaps))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    // No row misses all three, which `where` says of that pass of the loop.
    assert_eq!(stderr, "note: no row kept: the result is empty (line 4)\n");
    let lines: Vec<&str> = stdout.lines().collect();
    let rows_with = wages["rows_with"].as_array().unwrap();
    assert_eq!(
        lines,
        [
            whole(&wages["rows"]),
            whole(&wages["sum"]),
            whole(&rows_with[0]),
            whole(&rows_with[1]),
            whole(&rows_with[2]),
            whole(&rows_with[3]),
            // Over every variable of the file: the missing values in all, and the rows that
            // have none, by a variable and by the expression inside `where`.
            whole(&wages["sum_all"]),
            whole(&wages["complete"]),
            whole(&wages["complete"]),
            "2".to_string(),
        ]
    );
}

// ----- the edge cases, and what is unchanged -----

#[test]
fn the_edges_and_the_statistics_that_were_there() {
    let lines = printed(
        "let d = frame(a = [1.5, NA, NA], b = [2, NA, 4], c = [\"x\", NA, \"\"])\nd.add(g = row([a, b, c], stat: n_missing))[\"g\"]\nd.add(g = row(c, stat: n_missing))[\"g\"]\nd.add(g = row([a, a], stat: n_missing))[\"g\"]\nd.update(b = row([a, c], stat: n_missing))[\"b\"]\nd.add(m = row([a, b], stat: mean), s = row([a, b], stat: sum))[\"m\"]\nd.add(k = 3 - row([a, b, c], stat: n_missing))[\"k\"]\n",
    );
    assert_eq!(
        lines,
        vec![
            "Column<Int> · 3 values",
            "0",
            "3",
            // The empty text is a value.
            "1",
            "Column<Int> · 3 values",
            "0",
            "1",
            "0",
            // A variable named twice is one variable, as everywhere a list of variables
            // is taken.
            "Column<Int> · 3 values",
            "0",
            "1",
            "1",
            // `update` keeps the variable's type.
            "Column<Int> · 3 values",
            "0",
            "2",
            "1",
            // `mean` is as it was: over the values present, `NA` where there is none.
            "Column<Float> · 3 values",
            "1.75",
            "NA",
            "4",
            // The number of values present, from the count of those missing.
            "Column<Int> · 3 values",
            "3",
            "0",
            "2",
        ]
    );
    // A frame with no row has no count, and nothing is said of the count.
    let (code, stdout, stderr) = go(
        "let d = frame(a = [1.5, 2.5], b = [2, 4])\nd.where(a > 99).add(g = row([a, b], stat: n_missing))[\"g\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "Column<Int> · 0 values\n[]\n");
    assert_eq!(stderr, "note: no row kept: the result is empty (line 2)\n");
}

#[test]
fn min_valid_changes_nothing_and_a_note_says_so() {
    let data = "let d = frame(a = [1.5, NA, NA], b = [2, NA, 4])\n";
    let (code, stdout, stderr) = go(&format!(
        "{data}d.add(g = row([a, b], stat: n_missing, min_valid: 2))[\"g\"]\nd.add(g = row([a, b], stat: n_missing))[\"g\"]\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    // The same counts with it and without.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[..4], lines[4..]);
    assert_eq!(lines[1..4], ["0", "2", "1"]);
    assert_eq!(
        stderr,
        "note[W0401]: `min_valid:` changes nothing here: `stat: n_missing` counts the missing values and gives a number in every row (line 2)\n  = help: `min_valid:` is the number of values a row needs for a mean or a sum\n"
    );
    // A pragma allows it, as any coded note.
    let (code, _, stderr) = go(&format!(
        "{data}// zeff: allow(W0401) kept beside the mean above\nd.add(g = row([a, b], stat: n_missing, min_valid: 2))[\"g\"]\n"
    ));
    assert_eq!((code, stderr.as_str()), (0, ""));
}

#[test]
fn every_message() {
    let scripts = [
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, b]))\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, b], stat: missing))\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, b], stat: mean))\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, b], stat: sum))\n",
        "let d = frame(a = [1.5, NA], b = [true, NA])\nd.add(g = row([a, b], stat: sum))\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, zz], stat: n_missing))\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row([a, b], stat: n_missing, min_valid: 0))\n",
        "row([1, 2], stat: n_missing)\n",
        "let d = frame(a = [1.5, NA], b = [\"x\", NA])\nd.add(g = row(stat: n_missing))\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}{stdout}{stderr}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("row_n_missing_messages", out);
}

// ----- properties -----

/// One row of a random frame: a Float, an Int and a text, each perhaps missing.
type Row = (Option<i32>, Option<i32>, Option<u8>);

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// On any frame of three variables with gaps: each count lies between 0 and 3; it is the
    /// count over two of the variables plus that of the third; the order of the variables
    /// does not matter; and the rows with a count of 0 are the rows `drop_missing` keeps.
    #[test]
    fn the_count_adds_over_the_variables_and_ignores_their_order(
        rows in prop::collection::vec((prop::option::of(-9i32..9), prop::option::of(-9i32..9), prop::option::of(0u8..3)), 1..18),
    ) {
        let list = |pick: &dyn Fn(&Row) -> String| -> String {
            rows.iter().map(pick).collect::<Vec<_>>().join(", ")
        };
        let a = list(&|r| r.0.map_or("NA".to_string(), |v| format!("{v}.5")));
        let b = list(&|r| r.1.map_or("NA".to_string(), |v| v.to_string()));
        let c = list(&|r| r.2.map_or("NA".to_string(), |v| format!("\"t{v}\"")));
        // A last row with every value, so that each list has a type.
        let (code, stdout, stderr) = go(&format!(
            "let d = frame(a = [{a}, 0.5], b = [{b}, 1], c = [{c}, \"end\"])\nlet e = d.add(g = row([a, b, c], stat: n_missing), ab = row([a, b], stat: n_missing), only_c = row(c, stat: n_missing), turned = row([c, a, b], stat: n_missing))\ne[\"g\"]\nprint(e.where(g != ab + only_c).n_rows())\nprint(e.where(g != turned).n_rows())\nprint(e.where(g < 0 || g > 3).n_rows())\nprint(e.where(g == 0).n_rows() == d.drop_missing([a, b, c]).n_rows())\n"
        ));
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert!(stderr.lines().all(|line| line.starts_with("note: ")), "{}", stderr);
        let lines: Vec<&str> = stdout.lines().collect();
        let expected: Vec<String> = rows
            .iter()
            .map(|r| (usize::from(r.0.is_none()) + usize::from(r.1.is_none()) + usize::from(r.2.is_none())).to_string())
            .chain(std::iter::once("0".to_string()))
            .collect();
        let n = expected.len();
        prop_assert_eq!(&lines[1..=n], &expected.iter().map(String::as_str).collect::<Vec<_>>()[..]);
        prop_assert_eq!(&lines[n + 1..], &["0", "0", "0", "true"][..]);
    }
}
