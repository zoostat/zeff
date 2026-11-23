//! `row_number` (release 0.9.0, step 31b; open decision #167): the position of each row, from
//! 1, in the order the rows stand, and within each group under `by:`. Against base R
//! (`tests/reference/r/row_number.json`: `seq_len` and `ave(seq_along(g), g, FUN = seq_along)`)
//! on nine layouts of groups, two grouping variables, rows sorted first, and the shipped wages
//! by worker and by industry; the edge cases (an empty frame, one row, a missing group, every
//! row its own group); where it may stand (`add`, `update`, `where`, an expression) and where
//! it may not; every message; and the properties that the numbers of a group are 1 to its
//! size, each once, that the last of a group is `n()`, that it is a running sum of ones, and
//! that the rows' own values never matter.

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
    let path = root().join("tests/reference/r/row_number.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// The items of a grouping variable as a script writes them: text in quotes, `NA` bare.
fn items(values: &Json, text: bool) -> String {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| match v.as_str().unwrap() {
            "NA" => "NA".to_string(),
            s if text => format!("\"{s}\""),
            s => s.to_string(),
        })
        .collect::<Vec<String>>()
        .join(", ")
}

/// A column of whole numbers as it prints: its heading, then a value to a line.
fn column(values: &Json) -> Vec<String> {
    let values = values.as_array().unwrap();
    let mut lines = vec![format!(
        "Column<Int> · {} {}",
        values.len(),
        if values.len() == 1 { "value" } else { "values" }
    )];
    lines.extend(values.iter().map(|v| v.as_i64().unwrap().to_string()));
    lines
}

// ----- against R -----

#[test]
fn the_rows_and_the_rows_of_each_group_agree_with_r() {
    let r = reference();
    let cases = r["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 9);
    for (name, case) in cases {
        let rows = case["rows"].as_u64().unwrap();
        if rows == 0 {
            continue;
        }
        let g = items(&case["g"], case["text_groups"].as_bool().unwrap());
        // A second variable that plays no part, so that a frame of missing groups has a type.
        let lines = printed(&format!(
            "let d = frame(g = [{g}], x = [{}])\nd.add(k = row_number())[\"k\"]\nd.add(k = row_number(), by: g)[\"k\"]\n",
            (0..rows)
                .map(|i| format!("{}.5", i * 3 % 7))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let mut expected = column(&case["whole"]);
        expected.extend(column(&case["by"]));
        assert_eq!(lines, expected, "{name}");
    }
    // The empty frame: no row, no number, and nothing said of it but that it is empty.
    assert_eq!(cases["empty"]["rows"], 0);
    let (code, stdout, stderr) = go(
        "let e = frame(g = [1, 2], x = [1.5, 2.5]).where(g > 2)\ne.add(k = row_number()).n_rows()\ne.add(k = row_number(), by: g).n_rows()\ne.add(k = row_number())[\"k\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0\n0\nColumn<Int> · 0 values\n[]\n");
    assert_eq!(stderr, "note: no row kept: the result is empty (line 1)\n");
}

#[test]
fn two_grouping_variables_and_rows_sorted_first_agree_with_r() {
    let r = reference();
    let two = &r["two"];
    let lines = printed(&format!(
        "let d = frame(g1 = [{}], g2 = [{}])\nd.add(k = row_number(), by: [g1, g2])[\"k\"]\n",
        items(&two["g1"], true),
        items(&two["g2"], false)
    ));
    assert_eq!(lines, column(&two["by"]));
    // Sorted by day first: the rows are R's in R's order, and the numbers follow them.
    let sorted = &r["sorted"];
    let days = |v: &Json| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_i64().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    let lines = printed(&format!(
        "let d = frame(patient = [{}], day = [{}])\nlet s = d.sort(day).add(k = row_number(), by: patient)\ns[\"day\"]\ns[\"k\"]\ns[\"patient\"]\n",
        items(&sorted["patient"], true),
        days(&sorted["day"]).join(", ")
    ));
    let mut expected = column(&sorted["day_sorted"]);
    expected.extend(column(&sorted["by"]));
    expected.push("Column<Str> · 5 values".to_string());
    expected.extend(
        sorted["patient_sorted"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string()),
    );
    assert_eq!(lines, expected);
}

#[test]
fn the_shipped_wages_agree_with_r() {
    let r = reference();
    let wages = &r["wages"];
    let whole = |v: &Json| match v {
        Json::String(s) => s.clone(),
        other => other.as_i64().unwrap().to_string(),
    };
    // The whole frame; each worker's rows in the file's order; each worker's rows from the
    // latest year; and the rows of each industry.
    let mut script = String::from(
        "let w = dataset(wages)\nlet a = w.add(k = row_number())\nprint(a.sum(k))\nprint(a.max(k))\nlet b = w.add(k = row_number(), by: worker)\nprint(b.sum(k))\nprint(b.max(k))\nprint(b.where(k == 1).n_rows())\nlet c = w.sort(year, desc: true).add(k = row_number(), by: worker)\nprint(c.sum(k))\nprint(c.max(k))\nprint(c.where(k == 1).sum(year))\nlet d = w.add(k = row_number(), by: industry)\nprint(d.sum(k))\nprint(d.max(k))\nprint(d.where(k == 1).n_rows())\n",
    );
    // The first forty rows of each, one value to a line, read by their position.
    for frame in ["b", "c", "d"] {
        script.push_str(&format!(
            "let first_{frame} = {frame}.head(40)\nfor i in 1..40 {{ print(first_{frame}.where(row_number() == i).max(k)) }}\n"
        ));
    }
    let lines = printed(&script);
    let by_worker = &wages["by_worker"];
    let latest = &wages["by_worker_latest_first"];
    let by_industry = &wages["by_industry"];
    assert_eq!(
        lines[..11],
        [
            whole(&wages["whole"]["sum"]),
            whole(&wages["whole"]["last"]),
            whole(&by_worker["sum"]),
            whole(&by_worker["largest"]),
            whole(&by_worker["ones"]),
            whole(&latest["sum"]),
            whole(&latest["largest"]),
            whole(&latest["years_of_ones"]),
            whole(&by_industry["sum"]),
            whole(&by_industry["largest"]),
            whole(&by_industry["ones"]),
        ]
    );
    for (at, case) in [by_worker, latest, by_industry].into_iter().enumerate() {
        let first: Vec<String> = case["first"]
            .as_array()
            .unwrap()
            .iter()
            .map(whole)
            .collect();
        assert_eq!(first.len(), 40);
        assert_eq!(lines[11 + 40 * at..11 + 40 * (at + 1)], first[..], "{at}");
    }
    assert_eq!(lines.len(), 11 + 120);
}

// ----- where it stands -----

#[test]
fn it_stands_in_add_update_where_and_in_an_expression() {
    let lines = printed(
        "let d = frame(g = [\"a\", \"a\", \"b\", \"a\", \"b\"], x = [3.0, 1.0, 2.0, 9.0, 5.0])\nd.where(row_number() <= 2).n_rows()\nd.where(row_number() == n()).sum(x)\nd.add(share = row_number() / n(), back = n() - row_number() + 1, by: g)[\"back\"]\nd.update(x = row_number())[\"x\"]\nd.add(k = row_number(), by: g).where(k == 1)[\"x\"]\nd.add(id = row_number()).add(k = row_number(), by: g).where(id == 4)[\"k\"]\nd.add(k = row_number(), m = row_number() * 10)[\"m\"]\nprint(type_of(d.add(k = row_number())[\"k\"]))\n",
    );
    assert_eq!(
        lines,
        vec![
            "2",
            // The last row of the frame.
            "5",
            // Counted from the end of each group.
            "Column<Int> · 5 values",
            "3",
            "2",
            "2",
            "1",
            "1",
            // `update` keeps the variable's type, Float.
            "Column<Float> · 5 values",
            "1",
            "2",
            "3",
            "4",
            "5",
            // The first row of each group.
            "Column<Float> · 2 values",
            "3",
            "2",
            // The fourth row of the frame is the third of its group.
            "Column<Int> · 1 value",
            "3",
            "Column<Int> · 5 values",
            "10",
            "20",
            "30",
            "40",
            "50",
            "Column<Int>",
        ]
    );
}

#[test]
fn on_a_panel_the_numbers_follow_the_rows_and_not_the_time() {
    // A panel declared by worker and year, its rows out of time order: the numbers are the
    // rows' as they stand, and the declaration is kept.
    let (code, stdout, stderr) = go(
        "let p = frame(worker = [1, 1, 2, 2, 1], year = [2012, 2010, 2011, 2010, 2011], wage = [3.0, 1.0, 5.0, 4.0, 2.0]).panel(unit: worker, time: year)\nlet q = p.add(k = row_number(), by: worker)\nq[\"k\"]\nq.sort(year)[\"k\"]\nq.sort(year).add(k = row_number(), by: worker)[\"k\"]\nq\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("row_number_on_a_panel", format!("{stdout}---\n{stderr}"));
    assert!(stdout.contains("panel worker × year"), "{stdout}");
}

// ----- the messages -----

#[test]
fn every_message() {
    let scripts = [
        "row_number()\n",
        "let k = row_number()\n",
        "print(row_number() + 1)\n",
        "let d = frame(x = [1.5, 2.5])\nd.add(k = row_number(x))\n",
        "let d = frame(x = [1.5, 2.5])\nd.add(k = row_number(by: x))\n",
        "let d = frame(x = [1.5, 2.5])\nd.row_number()\n",
        "let d = frame(x = [1.5, 2.5])\nd.aggregate(k = row_number())\n",
        "let d = frame(x = [1.5, 2.5])\nd.sort(row_number())\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}{stdout}{stderr}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("row_number_messages", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Whatever the groups: in the whole frame the numbers are 1 to the number of rows in
    /// order; within a group they are 1 to its size, each once, rising down the rows; the
    /// last of a group is `n()`; they are a running sum of ones; and the rows' own values,
    /// missing or not, never matter.
    #[test]
    fn the_numbers_of_a_group_are_one_to_its_size(
// At most twenty rows with the one added: a column prints its first twenty values.
        groups in prop::collection::vec(prop::option::of(0u8..4), 1..20),
        values in prop::collection::vec(prop::option::of(-50i32..50), 24),
    ) {
        let n = groups.len();
        let g: Vec<String> = groups.iter().map(|g| g.map_or("NA".to_string(), |g| g.to_string())).collect();
        let x: Vec<String> = values[..n].iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect();
        // A group and a value that are not missing, so that each list has a type.
        let (code, stdout, stderr) = go(&format!(
            "let d = frame(g = [{}, 9], x = [{}, 0.5], one = [{}1])\nlet e = d.add(id = row_number()).add(k = row_number(), size = n(), run = running(one, stat: sum), by: g)\ne[\"id\"]\ne[\"k\"]\nprint(e.where(k != run).n_rows())\nprint(e.where(k > size).n_rows())\nprint(e.where(k == size).n_rows() == e.distinct(g).n_rows())\nprint(e.where(k == 1).n_rows() == e.distinct(g).n_rows())\n",
            g.join(", "),
            x.join(", "),
            "1, ".repeat(n)
        ));
        // `where` and `distinct` say what they dropped; nothing warns and nothing fails.
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert!(stderr.lines().all(|line| line.starts_with("note: ")), "{}", stderr);
        let lines: Vec<String> = stdout.lines().map(str::to_string).collect();
        let rows = n + 1;
        let ids: Vec<String> = (1..=rows).map(|i| i.to_string()).collect();
        prop_assert_eq!(&lines[1..=rows], &ids[..]);
        // Within each group: the count of the rows of that group so far.
        let mut seen: std::collections::HashMap<Option<u8>, usize> = std::collections::HashMap::new();
        let mut expected: Vec<String> = Vec::new();
        for group in groups.iter().copied().chain(std::iter::once(Some(9))) {
            let k = seen.entry(group).or_insert(0);
            *k += 1;
            expected.push(k.to_string());
        }
        prop_assert_eq!(&lines[rows + 2..2 * rows + 2], &expected[..]);
        prop_assert_eq!(&lines[2 * rows + 2..], &["0".to_string(), "0".to_string(), "true".to_string(), "true".to_string()][..]);
    }
}
