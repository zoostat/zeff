//! `table` (release 0.6.0, step 16; open decision #72; Language Reference, section 3.8): the
//! printed tables from lists and from a frame's columns as snapshots; every message; and the
//! property that a table built from a frame's columns prints the same cells as the frame.

use proptest::prelude::*;
use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn tables_from_lists_and_columns() {
    let script = r#"table(title: "sales by year", columns: {year: [2010, 2011, 2012], sales: [120.5, 130, NA], region: ["North", "South", "West"], flag: [true, false, true], day: [date("2024-01-01"), NA, date("1999-12-31")]})
let w = dataset(wages).where(worker <= 2)
table(title: "two workers", columns: {worker: w["worker"], year: w["year"], wage: w["wage"], female: w["female"], industry: w["industry"]})
table(title: "one row", columns: {k: [1]})
table(title: "dates", columns: {d: [date("2024-01-01"), NA]})
table(title: "all missing", columns: {k: [NA, NA], t: ["a", "b"]})
table(title: "empty columns", columns: {k: w.head(0)["wage"]})
let t = table(title: "t", columns: {k: [1, 2]})
t
type_of(t)
t.export("output/t.tex")
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("tables", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "table(columns: {k: [1]})\n",
        "table(title: \"t\")\n",
        "table(title: \"t\", columns: {})\n",
        "table(title: \"t\", columns: {a: [1, 2], b: [1]})\n",
        "table(title: \"t\", columns: {a: 3})\n",
        "table(title: \"t\", columns: {a: [[1], [2]]})\n",
        "table(title: \"t\", columns: {a: [attempt(frame(x = [1, 2]).reg(x ~ x))]})\n",
        "table(title: \"t\", columns: {a: [frame(x = [1, 2], y = [2, 5]).reg(y ~ x)]})\n",
        "table(title: 3, columns: {a: [1]})\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// A table built from a frame's columns prints the same cells as the frame, row for row.
    #[test]
    fn a_table_from_a_frame_prints_its_cells(
        ints in proptest::collection::vec(prop_oneof![Just(None), (-500i64..500).prop_map(Some)], 1..8),
        floats in proptest::collection::vec(prop_oneof![Just(None), (-1000.0f64..1000.0).prop_map(Some)], 8),
        words in proptest::collection::vec(prop_oneof![Just(None), "[a-z]{1,6}".prop_map(Some)], 8),
    ) {
        let n = ints.len();
        let mut ints = ints;
        ints[0] = ints[0].or(Some(1));
        let mut floats = floats[..n].to_vec();
        floats[0] = floats[0].or(Some(0.5));
        let mut words: Vec<Option<String>> = words[..n].to_vec();
        words[0] = words[0].clone().or(Some("w".to_string()));
        let list = |items: Vec<String>| items.join(", ");
        let script = format!(
            "let f = frame(i = [{}], x = [{}], s = [{}])\nf\ntable(title: \"t\", columns: {{i: f[\"i\"], x: f[\"x\"], s: f[\"s\"]}})\n",
            list(ints.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect()),
            list(floats.iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v:?}"))).collect()),
            list(words.iter().map(|v| v.as_ref().map_or("NA".to_string(), |t| format!("{t:?}"))).collect()),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        // The frame prints a header, a blank, the names, then rows numbered down the left and
        // stops at ten rows; the table prints its title, a blank, the names, then the rows.
        let lines: Vec<&str> = stdout.lines().collect();
        let frame_rows: Vec<Vec<&str>> = lines
            .iter()
            .skip(3)
            .take(n)
            .map(|l| l.split_whitespace().skip(1).collect())
            .collect();
        let table_start = lines.iter().position(|l| *l == "t").expect("the table's title");
        let table_rows: Vec<Vec<&str>> = lines
            .iter()
            .skip(table_start + 3)
            .take(n)
            .map(|l| l.split_whitespace().collect())
            .collect();
        prop_assert_eq!(frame_rows, table_rows, "{}", stdout);
    }
}
