//! One value out of a column (release 0.9.0, step 32a; open decision #168): `first(x)`,
//! `last(x)` and `x[i]` take a column and give the value of one row, and `to(x, List)` gives
//! the list of a column's values. Before this step each was refused, and one prediction had
//! to be read as `frame(p = m.predict(newdata: b.head(1))).mean(p)`.
//!
//! Held to the file itself: the first and the last row of the shipped wages, read here from
//! `wages.csv` as text. Then the types of the values, missing values, the forms inside `add`,
//! under `by:` and in `aggregate`, the review's case of one prediction, the refusals with
//! their messages stored, and the properties that `x[i]` is the i-th item of `to(x, List)`,
//! that `first` and `last` are its ends, and that lists are as they were.

use proptest::prelude::*;
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

// ----- against the file -----

#[test]
fn the_first_and_the_last_row_of_the_shipped_wages_are_the_files() {
    let text =
        std::fs::read_to_string(root().join("packages/tier1/datasets/data/wages.csv")).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let rows: Vec<Vec<&str>> = lines.map(|line| line.split(',').collect()).collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (first, last, tenth) = (&rows[0], &rows[rows.len() - 1], &rows[9]);
    let printed = printed(
        "let w = dataset(wages)\nprint(first(w[\"wage\"]))\nprint(last(w[\"wage\"]))\nprint(w[\"wage\"][10])\nprint(first(w[\"year\"]))\nprint(last(w[\"year\"]))\nprint(w[\"year\"][10])\nprint(first(w[\"industry\"]))\nprint(last(w[\"industry\"]))\nprint(w[\"industry\"][10])\nprint(w[\"worker\"][w.n_rows()])\nprint(len(to(w[\"wage\"], List)))\nprint(to(w[\"year\"], List)[10])\n",
    );
    let number = |cell: &str| format!("{}", cell.parse::<f64>().unwrap());
    assert_eq!(
        printed,
        vec![
            number(first[at("wage")]),
            number(last[at("wage")]),
            number(tenth[at("wage")]),
            first[at("year")].to_string(),
            last[at("year")].to_string(),
            tenth[at("year")].to_string(),
            // A category gives its label, as text.
            first[at("industry")].to_string(),
            last[at("industry")].to_string(),
            tenth[at("industry")].to_string(),
            last[at("worker")].to_string(),
            rows.len().to_string(),
            tenth[at("year")].to_string(),
        ]
    );
}

// ----- the types, and missing values -----

#[test]
fn one_value_has_the_type_of_its_column_and_a_missing_one_is_na() {
    let lines = printed(
        "let d = frame(x = [1.5, NA, 3.5], k = [7, 8, NA], s = [\"a\", NA, \"c\"], b = [true, false, NA], day = [date(\"2026-01-02\"), NA, date(\"2026-01-04\")])\nprint(type_of(first(d[\"x\"])), type_of(d[\"k\"][1]), type_of(first(d[\"s\"])), type_of(d[\"b\"][2]), type_of(last(d[\"day\"])))\nprint(d[\"x\"][2], last(d[\"k\"]), d[\"s\"][2], last(d[\"b\"]), d[\"day\"][2])\nprint(first(d[\"x\"]) + d[\"k\"][2], d[\"s\"][3] == \"c\", d[\"day\"][3] - d[\"day\"][1])\nprint(type_of(to(d[\"x\"], List)), type_of(to(d[\"s\"], List)), type_of(to(d[\"day\"], List)))\nprint(to(d[\"k\"], List))\nprint(to([1, 2], List), type_of(to([1.5], List)))\nprint(first(to(d[\"x\"], List)), last(to(d[\"s\"], List)), len(to(d[\"b\"], List)))\n",
    );
    assert_eq!(
        lines,
        vec![
            "Float Int Str Bool Date",
            // A missing value is `NA`, whatever the column's type.
            "NA NA NA NA NA",
            "9.5 true 2",
            "List<Float> List<Str> List<Date>",
            "7",
            "8",
            "NA",
            // A list given to `to(x, List)` is itself.
            "1",
            "2",
            "List<Float>",
            "1.5 c 3",
        ]
    );
}

// ----- inside frame methods -----

#[test]
fn inside_add_by_and_aggregate() {
    let (code, stdout, stderr) = go(
        "let d = frame(firm = [\"a\", \"a\", \"b\", \"b\", \"b\"], year = [2020, 2021, 2020, 2021, 2022], price = [10.0, 12.5, 7.0, NA, 9.0])\nd.add(start = first(price), end = last(price), second = price[2])\nd.add(start = first(price), end = last(price), change = last(price) - first(price), by: firm)\nd.aggregate(open = first(price), close = last(price), years = last(year) - first(year), by: firm)\nd.sort(year, desc: true).aggregate(latest = first(year), by: firm).sort(firm)\nd.where(price > first(price)).n_rows()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("inside_frame_methods", format!("{stdout}---\n{stderr}"));
}

// ----- the review's case -----

#[test]
fn one_prediction_is_read_from_the_column() {
    // A fit, a prediction for one row of new data, and its value read three ways; the
    // review's workaround, the mean of a column of one value, gives the same number.
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).drop_missing([wage, educ, exper])\nlet m = w.reg(log(wage) ~ educ + exper)\nlet p = m.predict(newdata: w.head(1))\nprint(type_of(p))\nprint(p[1] == first(p), first(p) == last(p), p[1] == to(p, List)[1])\nprint(abs(p[1] - frame(p = p).mean(p)) < 1e-12)\nprint(type_of(p[1]))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stderr,
        "note: 233 rows with NA in wage, educ or exper dropped (line 1)\n"
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines,
        vec!["Column<Float>", "true true true", "true", "Float"]
    );
}

// ----- the refusals -----

/// A position outside a column of one row: the count is said in the singular (release 0.9.0,
/// step 42c3; the messages were tested on columns of several rows).
#[test]
fn a_position_outside_a_column_of_one_row() {
    let (code, _, stderr) = go("let one = frame(a = [5])\none.add(b = a[3])\n");
    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("error[Z0207]: index 3 is outside 1..1\n"),
        "{stderr}"
    );
    assert!(stderr.contains("^ the column has 1 row\n"), "{stderr}");
}

#[test]
fn every_message() {
    let data = "let d = frame(x = [1.5, 2.5, NA], s = [\"a\", \"b\", \"c\"])\n";
    let scripts = [
        "d[\"x\"][0]\n",
        "d[\"x\"][4]\n",
        "d[\"x\"][-1]\n",
        "d[\"x\"][NA]\n",
        "d[\"x\"][1.5]\n",
        "d[\"x\"][\"a\"]\n",
        "d[\"x\"][1..2]\n",
        "d.add(v = x[9])\n",
        "first(d.where(x > 99)[\"x\"])\n",
        "last(d.where(x > 99)[\"x\"])\n",
        "d.where(x > 99)[\"x\"][1]\n",
        "to(3, List)\n",
        "to(\"abc\", List)\n",
        "to(d, List)\n",
        "to(d[\"x\"], List(\"%d\"))\n",
        "first(d)\n",
        "first(3)\n",
        "first([])\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(&format!("{data}{script}"));
        assert_eq!(code, 1, "{script}{stdout}{stderr}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("one_value_messages", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// For any column of numbers with gaps: `x[i]` is the i-th item of `to(x, List)`;
    /// `first` and `last` are its ends; the list has a value for every row; and a list is
    /// indexed, and gives its first and last, as it did.
    #[test]
    fn a_position_of_a_column_is_that_item_of_its_list(
        values in prop::collection::vec(prop::option::of(-99i32..99), 1..12),
        at in 0usize..12,
    ) {
        let n = values.len();
        let i = at % n + 1;
        let items: Vec<String> = values.iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect();
        // A last value that is not missing, so that the list has a type.
        let lines = printed(&format!(
            "let d = frame(x = [{}, 0.5])\nlet x = d[\"x\"]\nlet l = to(x, List)\nprint(len(l))\nprint(x[{i}])\nprint(l[{i}])\nprint(first(x))\nprint(first(l))\nprint(last(x))\nprint(x[{}])\n",
            items.join(", "),
            n + 1
        ));
        let shown = |v: &Option<i32>| v.map_or("NA".to_string(), |v| format!("{v}.5"));
        prop_assert_eq!(
            lines,
            vec![
                (n + 1).to_string(),
                shown(&values[i - 1]),
                shown(&values[i - 1]),
                shown(&values[0]),
                shown(&values[0]),
                "0.5".to_string(),
                "0.5".to_string(),
            ]
        );
    }
}
