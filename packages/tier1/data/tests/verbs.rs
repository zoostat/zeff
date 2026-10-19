//! The frame verbs `head`, `tail`, `where`, `select`, `drop` and `rename` (Language Reference,
//! sections 3.7, 4.1 and 6) through the engine: against R's `tests/reference/r/frames.json`
//! (dplyr's answers, via base R), the edge-case datasets, the name lookup inside a frame method,
//! every message as a snapshot, and a property of `where`.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run, run_with};

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

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/frames.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn ids(json: &Json) -> Vec<i64> {
    json.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect()
}

// ----- the reference -----

/// The printed Int column at the front of `lines`: its header, then its values.
fn take_column<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Vec<i64> {
    let header = lines.next().expect("a column header");
    let n: usize = header
        .strip_prefix("Column<Int> · ")
        .and_then(|h| h.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("not a column header: {header}"));
    (0..n)
        .map(|_| lines.next().unwrap().parse().unwrap())
        .collect()
}

#[test]
fn head_tail_and_where_agree_with_r_on_the_large_file() {
    let r = reference();
    let large = &r["datasets"]["wages_large"];
    let (code, stdout, stderr) = go(
        "let d = import(\"data/wages_large.csv\")\nd.n_rows()\nd.n_cols()\nd.head(3)[\"id\"]\nd.tail(3)[\"id\"]\nd.head(10)[\"id\"]\nd.tail(10)[\"id\"]\n\
         let over = d.where(hours > 40)\nover.n_rows()\nover.head(10)[\"id\"]\nd.where(female).n_rows()\n\
         let both = d.where(educ >= 16 && wage < 20)\nboth.n_rows()\nboth.head(10)[\"id\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let mut lines = stdout.lines();
    let number =
        |lines: &mut std::str::Lines<'_>| -> i64 { lines.next().unwrap().parse().unwrap() };
    assert_eq!(number(&mut lines), large["n_rows"].as_i64().unwrap());
    assert_eq!(number(&mut lines), large["n_cols"].as_i64().unwrap());
    assert_eq!(take_column(&mut lines), ids(&large["head_3"]));
    assert_eq!(take_column(&mut lines), ids(&large["tail_3"]));
    assert_eq!(take_column(&mut lines), ids(&large["head_10"]));
    assert_eq!(take_column(&mut lines), ids(&large["tail_10"]));
    let over = &large["where"]["hours_over_40"];
    assert_eq!(number(&mut lines), over["count"].as_i64().unwrap());
    assert_eq!(take_column(&mut lines), ids(&over["first_10"]));
    assert_eq!(
        number(&mut lines),
        large["where"]["female"]["count"].as_i64().unwrap()
    );
    let both = &large["where"]["educ_at_least_16_and_wage_under_20"];
    assert_eq!(number(&mut lines), both["count"].as_i64().unwrap());
    assert_eq!(take_column(&mut lines), ids(&both["first_10"]));
    assert_eq!(lines.next(), None);
    let na_dropped = over["na_dropped"].as_i64().unwrap();
    assert_eq!(
        without_format_notes(&stderr),
        format!(
            "note: {na_dropped} rows dropped where `hours > 40` was NA; `d.where(is_missing(hours))` lists them (line 8)\n"
        )
    );
}

// ----- the shape of each verb -----

#[test]
fn head_and_tail_take_counts_from_zero_to_everything() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.head(2)\nw.tail(2)\nw.head(0).n_rows()\nw.head(100).n_rows()\nw.tail(n: 1)[\"id\"]\nw.head().n_rows()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "frame · 2 rows × 4 variables\n\n      id   wage   hours   name\n  1    1   23.4      40   Ann\n  2    2     19      35   Bob\n\
         frame · 2 rows × 4 variables\n\n      id   wage   hours   name\n  1    2     19      35   Bob\n  2    3   31.2      NA   Cy\n\
         0\n3\nColumn<Int> · 1 value\n3\n3\n"
    );
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.head(-1)\n",
        "let w = import(\"data/wages.csv\")\nw.tail(n: 2.5)\n",
        "let w = import(\"data/wages.csv\")\nw.head(NA)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("head_tail_counts", all);
}

#[test]
fn where_keeps_true_rows_drops_na_rows_with_a_note_and_carries_metadata() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nlet kept = w.where(hours > 36)\nkept\nw.where(is_missing(hours)).n_rows()\nw.where(wage > 100).n_rows()\n\
         w.where(!(hours > 36)).n_rows()\nlet limit = 36\nw.where(hours > limit).n_rows()\nw.where(hours > w.mean(hours)).n_rows()\nw.where(abs(hours - 40) < 3)[\"id\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "kept · 1 row × 4 variables\n\n      id   wage   hours   name\n  1    1   23.4      40   Ann\n1\n0\n1\n1\n1\nColumn<Int> · 1 value\n1\n"
    );
    insta::assert_snapshot!("where_notes", without_format_notes(&stderr));
    // Opaque and every other column, and the frame's metadata, come through a selection.
    let (code, stdout, stderr) = go(
        "let d = import(\"data/infinities.csv\")\nd.where(finite > 1)\nd.where(finite > 1).describe()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.starts_with("frame · 2 rows × 5 variables\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("\n  1    Inf    -Inf    Inf          NA        2\n  2      2       2   -Inf          NA        3\n"),
        "{stdout}"
    );
}

#[test]
fn where_refuses_what_is_not_a_condition() {
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.where(hours)\n",
        "let w = import(\"data/wages.csv\")\nw.where(true)\n",
        "let w = import(\"data/wages.csv\")\nw.where(NA)\n",
        "let w = import(\"data/wages.csv\")\nw.where(hour > 36)\n",
        "let w = import(\"data/wages.csv\")\nlet o = import(\"data/one_row.csv\")\nw.where(o[\"x\"] > 1)\n",
        "let w = import(\"data/wages.csv\")\nw.where(hours == NA).n_rows()\n",
    ] {
        let (code, stdout, stderr) = go(script);
        if script.contains("== NA") {
            // A comparison with NA is NA in every row: all dropped, with the note.
            assert_eq!(code, 0, "{stderr}");
            assert_eq!(stdout, "0\n");
            all.push_str(&without_format_notes(&stderr));
        } else {
            assert_eq!(code, 1, "{script}");
            all.push_str(&without_format_notes(&stderr));
        }
        all.push('\n');
    }
    assert_eq!(all.matches("error[Z0203]").count(), 3, "{all}");
    assert!(
        all.contains("error[Z0101]: no variable `hour` in `w`"),
        "{all}"
    );
    assert!(all.contains("error[Z0217]"), "{all}");
    insta::assert_snapshot!("where_refusals", all);
}

#[test]
fn a_name_that_is_both_a_variable_and_a_binding_is_the_variable_with_a_warning() {
    let (code, stdout, stderr) =
        go("let w = import(\"data/wages.csv\")\nlet hours = 0\nw.where(hours > 36).n_rows()\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "1\n");
    assert!(
        stderr.contains(
            "warning: `hours` is both a variable of w and a binding; the variable was used (line 3)"
        ),
        "{stderr}"
    );
    insta::assert_snapshot!("variable_and_binding", without_format_notes(&stderr));
}

#[test]
fn select_drop_and_rename_choose_and_name_variables() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.select([hours, id]).variables()\nw.select(wage).variables()\nw.select(wage..hours).variables()\nw.select([id, id..wage]).variables()\n\
         w.drop(hours).variables()\nw.drop([id, hours]).variables()\nw.drop(id..hours).n_cols()\nw.rename(pay = wage, worker = id).variables()\nw.rename(pay = wage).mean(pay)\nw.select(hours).where(hours > 36)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "hours\nid\nwage\nwage\nhours\nid\nwage\nid\nwage\nname\nwage\nname\n1\nworker\npay\nhours\nname\n24.5333\nframe · 1 row × 1 variable\n\n      hours\n  1      40\n"
    );
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.select(wge)\n",
        "let w = import(\"data/wages.csv\")\nw.drop([id, Hours])\n",
        "let w = import(\"data/wages.csv\")\nw.select(hours..id)\n",
        "let w = import(\"data/wages.csv\")\nw.rename(wage = id)\n",
        "let w = import(\"data/wages.csv\")\nw.rename(pay = wge)\n",
        "let w = import(\"data/wages.csv\")\nw.rename(pay = \"wage\")\n",
        "let w = import(\"data/wages.csv\")\nw.select(\"wage\")\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("select_drop_rename_errors", all);
}

#[test]
fn the_edge_case_datasets() {
    let (code, stdout, stderr) = go(
        "let e = import(\"data/empty.csv\", types: {x: Float, y: Bool})\ne.where(x > 1).n_rows()\ne.head(3).n_rows()\ne.select(y).variables()\n\
         let a = import(\"data/all_na.csv\", types: {x: Float, y: Int, z: Str})\na.where(x > 1).n_rows()\na.where(is_missing(x)).n_rows()\n\
         let o = import(\"data/one_row.csv\")\no.where(flag)\no.tail(5).n_rows()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "0\n0\ny\n0\n3\nframe · 1 row × 3 variables\n\n       x     y   flag\n  1   42   3.5   true\n1\n"
    );
    insta::assert_snapshot!("edge_case_notes", without_format_notes(&stderr));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(12)))]

    #[test]
    fn where_and_its_negation_partition_the_rows_that_are_not_na(values in prop::collection::vec(prop::option::of(0i64..10), 1..12), cut in 0i64..10) {
        // Two columns, so that a row with a missing value is never a blank line, which the
        // reader skips.
        let csv: String = std::iter::once("id,x".to_string())
            .chain(values.iter().enumerate().map(|(i, v)| format!("{},{}", i + 1, v.map_or(String::new(), |i| i.to_string()))))
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let script = format!(
            "let d = import(\"data/p.csv\", types: {{id: Int, x: Int}})\nd.where(x > {cut}).n_rows()\nd.where(!(x > {cut})).n_rows()\nd.where(is_missing(x)).n_rows()\nd.head(3).n_rows()\n"
        );
        let outcome = run_with(&[("data/p.csv", csv.as_bytes())], &script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let above = values.iter().flatten().filter(|&&v| v > cut).count();
        let below = values.iter().flatten().filter(|&&v| v <= cut).count();
        let missing = values.iter().filter(|v| v.is_none()).count();
        let expected = format!("{above}\n{below}\n{missing}\n{}\n", values.len().min(3));
        prop_assert_eq!(outcome.output.stdout, expected);
    }
}
