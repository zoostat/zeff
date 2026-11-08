//! `tabulate` (Language Reference, section 6, "Look"; release 0.7.0, step 8): the reference
//! (`tests/reference/r/tabulate.json`, R's `table` with `useNA = "ifany"` and `prop.table`) on
//! the shipped wages and on a small frame with `NA`, counts exactly and percentages to the
//! printed decimal; the edge cases; a property on random frames (the counts add up to the rows);
//! and a snapshot of every form of question 5 and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

/// Exit code, standard output and messages of a script.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/tabulate.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap()
}

fn strings(values: &Json) -> Vec<String> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn integers(values: &Json) -> Vec<u64> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect()
}

fn numbers(values: &Json) -> Vec<f64> {
    strings(values)
        .iter()
        .map(|s| s.parse::<f64>().unwrap())
        .collect()
}

/// A printed count, without its thousands separators.
fn count_of(cell: &str) -> u64 {
    cell.replace(',', "")
        .parse()
        .unwrap_or_else(|_| panic!("not a count: {cell}"))
}

/// Whether a printed one-decimal percentage agrees with the exact one: within half a unit of
/// the last place.
fn percent_agrees(cell: &str, exact: f64) -> bool {
    cell.parse::<f64>()
        .is_ok_and(|v| (v - exact).abs() <= 0.05 + 1e-9)
}

/// One printed table: its lines after the title and the blank line, the heading first.
struct Printed {
    lines: Vec<String>,
}

impl Printed {
    fn of(stdout: &str) -> Printed {
        Printed {
            lines: stdout.lines().skip(2).map(str::to_string).collect(),
        }
    }

    fn heading(&self) -> Vec<String> {
        self.lines[0]
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    /// The position of the row whose first cell is `level`, which may hold spaces; the heading
    /// is skipped, since a level may share the variable's name (`female`).
    fn position(&self, level: &str) -> usize {
        let start = format!("  {level} ");
        self.lines
            .iter()
            .skip(1)
            .position(|l| l.starts_with(&start) || *l == format!("  {level}"))
            .map(|at| at + 1)
            .unwrap_or_else(|| panic!("no row {level} in\n{}", self.lines.join("\n")))
    }

    /// The cells after the level in the row of `level`.
    fn cells(&self, level: &str) -> Vec<String> {
        let at = self.position(level);
        self.lines[at][2 + level.len()..]
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    /// The cells of the row under the row of `level` (the percentages of a two-way table).
    fn under(&self, level: &str) -> Vec<String> {
        let at = self.position(level);
        self.lines[at + 1]
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }
}

// ----- the reference -----

fn check_one_way(frame_script: &str, case: &Json) {
    let var = case["var"].as_str().unwrap();
    let levels = strings(&case["levels"]);
    let counts = integers(&case["counts"]);
    let percents = numbers(&case["percent"]);
    let (code, stdout, stderr) = go(&format!("{frame_script}d.tabulate({var})\n"));
    assert_eq!(code, 0, "{stderr}");
    let printed = Printed::of(&stdout);
    assert_eq!(printed.heading(), vec![var, "N", "%"]);
    let mut total = 0;
    for ((level, count), percent) in levels.iter().zip(&counts).zip(&percents) {
        let cells = printed.cells(level);
        assert_eq!(count_of(&cells[0]), *count, "{var} {level}");
        assert!(
            percent_agrees(&cells[1], *percent),
            "{var} {level}: {} against {percent}",
            cells[1]
        );
        total += count;
    }
    let total_row = printed.cells("Total");
    assert_eq!(count_of(&total_row[0]), total);
    assert_eq!(total_row[1], "100.0");
    // Levels present in Zeff's table and R's are the same set: the rows between the heading
    // and the rule are R's levels.
    assert_eq!(printed.lines.len(), levels.len() + 3, "{stdout}");
}

fn check_two_way(frame_script: &str, case: &Json) {
    let vars = strings(&case["vars"]);
    let rows = strings(&case["rows"]);
    let cols = strings(&case["cols"]);
    let counts: Vec<Vec<u64>> = case["counts"]
        .as_array()
        .unwrap()
        .iter()
        .map(integers)
        .collect();
    for (word, key) in [
        ("row", "row_percent"),
        ("col", "col_percent"),
        ("total", "total_percent"),
    ] {
        let percents: Vec<Vec<f64>> = case[key].as_array().unwrap().iter().map(numbers).collect();
        let (code, stdout, stderr) = go(&format!(
            "{frame_script}d.tabulate([{}, {}], percent: {word})\n",
            vars[0], vars[1]
        ));
        assert_eq!(code, 0, "{stderr}");
        let printed = Printed::of(&stdout);
        let heading = printed.heading();
        assert_eq!(heading[0], vars[0]);
        assert_eq!(heading.last().map(String::as_str), Some("Total"));
        // Zeff's columns are R's levels in the same order (numbers in order, text by code
        // point), printed by their labels where the variable has them (`female`, `union`), so
        // they are matched by position.
        let column_of = |level: &str| -> usize {
            let at = cols.iter().position(|c| c == level).unwrap();
            if heading[1 + at] != *level {
                assert!(
                    vars[1] == "female" || vars[1] == "union",
                    "column {level} printed as {}",
                    heading[1 + at]
                );
            }
            at
        };
        assert_eq!(heading.len(), cols.len() + 2, "{heading:?}");
        let mut column_totals = vec![0u64; cols.len()];
        let mut grand = 0u64;
        for (i, level) in rows.iter().enumerate() {
            let cells = printed.cells(level);
            let shown_percents = printed.under(level);
            assert_eq!(cells.len(), cols.len() + 1);
            assert_eq!(shown_percents.len(), cols.len() + 1);
            let mut row_total = 0;
            for (j, col) in cols.iter().enumerate() {
                let at = column_of(col);
                assert_eq!(count_of(&cells[at]), counts[i][j], "{level} × {col}");
                assert!(
                    percent_agrees(&shown_percents[at], percents[i][j]),
                    "{word} % of {level} × {col}: {} against {}",
                    shown_percents[at],
                    percents[i][j]
                );
                row_total += counts[i][j];
                column_totals[at] += counts[i][j];
            }
            assert_eq!(count_of(cells.last().unwrap()), row_total);
            grand += row_total;
        }
        let total_row = printed.cells("Total");
        for (j, t) in column_totals.iter().enumerate() {
            assert_eq!(count_of(&total_row[j]), *t);
        }
        assert_eq!(count_of(total_row.last().unwrap()), grand);
    }
}

#[test]
fn the_shipped_wages_tabulate_as_r_counts_them() {
    let reference = reference();
    let script = "let d = dataset(wages)\n";
    for case in reference["wages"]["one_way"].as_array().unwrap() {
        check_one_way(script, case);
    }
    for case in reference["wages"]["two_way"].as_array().unwrap() {
        check_two_way(script, case);
    }
}

#[test]
fn a_small_frame_with_na_tabulates_as_r_counts_it() {
    let reference = reference();
    let script = "let d = frame(a = [1, NA, 1, 2, 2, 2, NA], b = [\"x\", \"y\", \"x\", \"y\", \"x\", NA, \"y\"])\n";
    for case in reference["small"]["one_way"].as_array().unwrap() {
        check_one_way(script, case);
    }
    for case in reference["small"]["two_way"].as_array().unwrap() {
        check_two_way(script, case);
    }
}

// ----- the edge cases -----

#[test]
fn an_empty_frame_gives_a_total_of_zero_and_na_percentages() {
    let (code, stdout, stderr) = go(
        "let e = frame(a = [1, 2]).where(a > 5)\ne.tabulate(a)\ne.tabulate([a, a], percent: row)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("  Total   0   NA\n"), "{stdout}");
    assert!(
        stdout.contains("  Total       0\n             NA\n"),
        "{stdout}"
    );
}

#[test]
fn floats_dates_bools_and_labels_tabulate_by_their_printed_values() {
    let (code, stdout, stderr) = go(
        "let f = frame(x = [1.5, 2.5, 1.5, NA], d = [to(\"2026-01-02\", Date), to(\"2026-01-01\", Date), to(\"2026-01-02\", Date), NA], b = [true, false, true, true])\nf.tabulate(x)\nf.tabulate([d, b])\nlet w = dataset(wages).where(worker <= 3)\nw.tabulate(female)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let x = Printed::of(stdout.split("tabulate · f · d").next().unwrap());
    assert_eq!(x.cells("1.5"), vec!["2", "50.0"]);
    assert_eq!(x.cells("NA"), vec!["1", "25.0"]);
    let two = Printed::of(stdout.split("tabulate · f · d × b").nth(1).unwrap());
    assert_eq!(two.heading(), vec!["d", "false", "true", "Total"]);
    assert_eq!(two.cells("2026-01-01"), vec!["1", "0", "1"]);
    assert_eq!(two.cells("NA"), vec!["0", "1", "1"]);
    // The wages' `female` prints its labels, as the frame does.
    let female = Printed::of(stdout.split("tabulate · w · female").nth(1).unwrap());
    assert_eq!(female.heading(), vec!["female", "N", "%"]);
    assert_eq!(female.cells("male"), vec!["7", "53.8"]);
    assert_eq!(female.cells("female"), vec!["6", "46.2"]);
}

#[test]
fn one_variable_takes_only_the_percentage_of_the_total() {
    let (code, stdout, _) = go("frame(a = [1, 2]).tabulate(a, percent: total)\n");
    assert_eq!(code, 0);
    assert_eq!(Printed::of(&stdout).cells("1"), vec!["1", "50.0"]);
    let (code, _, stderr) = go("frame(a = [1, 2]).tabulate(a, percent: row)\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("Z0401"), "{stderr}");
    let (code, _, stderr) = go("frame(a = [1, 2]).tabulate(a, percent: col)\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("need two variables"), "{stderr}");
}

#[test]
fn the_refusals() {
    for (script, code) in [
        (
            "let f = frame(a = [1], b = [2], c = [3])\nf.tabulate([a, b, c])\n",
            "Z0401",
        ),
        ("let f = frame(a = [1])\nf.tabulate(b)\n", "Z0101"),
        ("let f = frame(Age = [1])\nf.tabulate(age)\n", "Z0102"),
        (
            "let f = frame(a = [1])\nf.tabulate(a, percent: cell)\n",
            "Z0402",
        ),
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        assert!(stderr.contains(code), "{script}\n{stderr}");
    }
}

// ----- a property -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]
    #[test]
    fn the_counts_add_up_to_the_rows(
        a in prop::collection::vec(prop::option::weighted(0.8, -3i64..4), 1..40),
        b in prop::collection::vec(prop::option::weighted(0.7, 0i64..3), 1..40),
    ) {
        let rows = a.len().min(b.len());
        // A column of `NA` alone has no type, so each keeps one value.
        let mut a = a;
        let mut b = b;
        if a[..rows].iter().all(Option::is_none) { a[0] = Some(1); }
        if b[..rows].iter().all(Option::is_none) { b[0] = Some(1); }
        let text = |c: &[Option<i64>]| -> String {
            c[..rows].iter().map(|v| v.map_or("NA".to_string(), |x| x.to_string())).collect::<Vec<_>>().join(", ")
        };
        let (code, stdout, stderr) = go(&format!(
            "let f = frame(a = [{}], b = [{}])\nf.tabulate(a)\nf.tabulate([a, b], percent: total)\n",
            text(&a), text(&b)
        ));
        prop_assert_eq!(code, 0, "{}", stderr);
        let mut parts = stdout.split("tabulate · f · a × b");
        let one = Printed::of(parts.next().unwrap());
        let two = Printed::of(parts.next().unwrap());
        // One way: every level present once, in order, the counts summing to the rows and the
        // percentages to 100 within rounding.
        let mut levels: Vec<Option<i64>> = a[..rows].to_vec();
        levels.sort_by(|x, y| match (x, y) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
            (Some(p), Some(q)) => p.cmp(q),
        });
        levels.dedup();
        prop_assert_eq!(one.lines.len(), levels.len() + 3);
        let mut sum = 0u64;
        let mut percent_sum = 0.0;
        for (i, level) in levels.iter().enumerate() {
            let shown = level.map_or("NA".to_string(), |x| x.to_string());
            let line = &one.lines[1 + i];
            prop_assert!(line.starts_with(&format!("  {shown} ")), "{} for {}", line, shown);
            let cells = one.cells(&shown);
            let expected = a[..rows].iter().filter(|v| **v == *level).count() as u64;
            prop_assert_eq!(count_of(&cells[0]), expected);
            sum += expected;
            percent_sum += cells[1].parse::<f64>().unwrap();
        }
        prop_assert_eq!(sum, rows as u64);
        prop_assert_eq!(count_of(&one.cells("Total")[0]), rows as u64);
        prop_assert!((percent_sum - 100.0).abs() <= 0.05 * levels.len() as f64 + 1e-9, "{}", percent_sum);
        // Two way: the grand total is the rows, and the corner percentage is 100.
        let total = two.cells("Total");
        prop_assert_eq!(count_of(total.last().unwrap()), rows as u64);
        let corner = two.under("Total");
        prop_assert_eq!(corner.last().map(String::as_str), Some("100.0"));
    }
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (code, stdout, stderr) = go(
        "let wages = dataset(wages)\nwages.tabulate(industry)\nwages.tabulate([female, union])\nwages.tabulate([female, union], percent: row)\nwages.tabulate([female, union], percent: col)\nwages.tabulate([female, union], percent: total)\nlet small = frame(sex = [\"f\", \"m\", NA, \"f\", \"m\", \"f\"], smoker = [true, false, true, NA, false, false])\nsmall.tabulate(sex)\nsmall.tabulate([sex, smoker])\nsmall.tabulate([sex, smoker], percent: col)\nsmall.tabulate(smoker, percent: total)\nlet e = frame(a = [1, 2]).where(a > 5)\ne.tabulate(a)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "let f = frame(a = [1], b = [2], c = [3])\nf.tabulate([a, b, c])\n",
        "let f = frame(a = [1])\nf.tabulate(a, percent: row)\n",
        "let f = frame(a = [1])\nf.tabulate(a, percent: col)\n",
        "let f = frame(a = [1])\nf.tabulate(a, percent: cell)\n",
        "let f = frame(a = [1])\nf.tabulate(b)\n",
        "let f = frame(Age = [1])\nf.tabulate(age)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
