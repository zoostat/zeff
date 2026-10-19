//! The frame verbs `add`, `update` and `sort` (Language Reference, sections 3.7, 3.8 and 6)
//! through the engine: `sort` against R's `tests/reference/r/frames.json`, definitions that use
//! earlier ones, replacement with its note, `update` with and without `where:`, the type rules,
//! every message as a snapshot, and properties of `sort`.

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

// ----- sort against R -----

#[test]
fn sort_agrees_with_r_on_every_key_and_direction() {
    let r = reference();
    let sorts = &r["datasets"]["wages_large"]["sort"];
    let mut script = String::from("let d = import(\"data/wages_large.csv\")\n");
    let mut expected: Vec<Vec<i64>> = Vec::new();
    for key in ["wage", "hours", "region", "educ"] {
        for (direction, desc) in [("asc", "false"), ("desc", "true")] {
            script.push_str(&format!(
                "d.sort({key}, desc: {desc}).head(10)[\"id\"]\nd.sort({key}, desc: {desc}).tail(10)[\"id\"]\n"
            ));
            let case = &sorts[format!("{key}_{direction}")];
            expected.push(ids(&case["first_10"]));
            expected.push(ids(&case["last_10"]));
        }
    }
    script.push_str(
        "d.sort([region, wage]).head(10)[\"id\"]\nd.sort([region, wage]).tail(10)[\"id\"]\n",
    );
    expected.push(ids(&sorts["region_then_wage_asc"]["first_10"]));
    expected.push(ids(&sorts["region_then_wage_asc"]["last_10"]));
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    let mut lines = stdout.lines();
    for (i, want) in expected.iter().enumerate() {
        assert_eq!(&take_column(&mut lines), want, "selection {i}");
    }
    assert_eq!(lines.next(), None);
    assert_eq!(without_format_notes(&stderr), "");
}

// ----- add -----

#[test]
fn add_defines_variables_left_to_right_and_replaces_with_a_note() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.add(pay = wage * hours, double = pay * 2, year = 2026, flag = hours > 36, who = \"x\").variables()\n\
         w.add(pay = wage * hours)[\"pay\"]\nw.add(wage = wage / 10)[\"wage\"]\nlet x = 5\nw.add(five = x, hours = hours + x)[\"hours\"]\nw.add(lw = log(wage), high = lw > 3)[\"high\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "id\nwage\nhours\nname\npay\ndouble\nyear\nflag\nwho\n\
         Column<Float> · 3 values\n936\n665\nNA\n\
         Column<Float> · 3 values\n2.34\n1.9\n3.12\n\
         Column<Int> · 3 values\n45\n40\nNA\n\
         Column<Bool> · 3 values\ntrue\nfalse\ntrue\n"
    );
    insta::assert_snapshot!("add_notes", without_format_notes(&stderr));
    // Metadata of the untouched variables survives, and the frame's source does too.
    let (code, stdout, stderr) =
        go("let w = import(\"data/wages.csv\")\nw.add(one = 1).describe()\n");
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("\n  one        Int               3         0   1\n"),
        "{stdout}"
    );
}

#[test]
fn add_refuses_what_is_not_a_variable() {
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.add(z = NA)\n",
        "let w = import(\"data/wages.csv\")\nw.add(z = [1, 2, 3])\n",
        "let w = import(\"data/wages.csv\")\nlet o = import(\"data/one_row.csv\")\nw.add(z = o[\"x\"])\n",
        "let w = import(\"data/wages.csv\")\nw.add(z = wge + 1)\n",
        "let w = import(\"data/wages.csv\")\nw.add()\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("add_refusals", all);
}

// ----- update -----

#[test]
fn update_changes_values_where_asked_and_keeps_the_type() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nw.update(hours = 38, where: is_missing(hours))[\"hours\"]\nw.update(wage = wage * 2, where: hours > 36)[\"wage\"]\n\
         w.update(wage = 1)[\"wage\"]\nw.update(name = \"n\", where: id == 2)[\"name\"]\nw.update(hours = hours * 2, wage = hours)[\"wage\"]\nw.update(wage = 0, where: hours == NA)[\"wage\"]\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Int> · 3 values\n40\n35\n38\n\
         Column<Float> · 3 values\n46.8\n19\n31.2\n\
         Column<Float> · 3 values\n1\n1\n1\n\
         Column<Str> · 3 values\nAnn\nn\nCy\n\
         Column<Float> · 3 values\n80\n70\nNA\n\
         Column<Float> · 3 values\n23.4\n19\n31.2\n"
    );
    insta::assert_snapshot!("update_notes", without_format_notes(&stderr));
    // The variable keeps its metadata and position.
    let (code, stdout, stderr) =
        go("let w = import(\"data/wages.csv\")\nw.update(hours = 0).variables()\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "id\nwage\nhours\nname\n");
}

#[test]
fn update_refuses_new_variables_wrong_types_and_bad_conditions() {
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.update(pay = 1)\n",
        "let w = import(\"data/wages.csv\")\nw.update(hours = 2.5)\n",
        "let w = import(\"data/wages.csv\")\nw.update(name = 1)\n",
        "let w = import(\"data/wages.csv\")\nw.update(wage = 1, where: hours)\n",
        "let w = import(\"data/wages.csv\")\nw.update(wage = 1, where: true)\n",
        "let w = import(\"data/wages.csv\")\nlet o = import(\"data/one_row.csv\")\nw.update(wage = 1, where: o[\"x\"] > 1)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("update_refusals", all);
}

// ----- sort: the rules -----

#[test]
fn sort_is_stable_puts_na_last_and_orders_text_by_code_point() {
    let csv = "id,g,t,b\n1,2,b,true\n2,1,a,false\n3,2,,true\n4,1,B,\n5,,é,false\n6,2,Zz,true\n";
    let outcome = run_with(
        &[("data/s.csv", csv.as_bytes())],
        "let d = import(\"data/s.csv\")\nd.sort(g)[\"id\"]\nd.sort(g, desc: true)[\"id\"]\nd.sort(t)[\"id\"]\nd.sort(t, desc: true)[\"id\"]\nd.sort(b)[\"id\"]\nd.sort([g, t])[\"id\"]\nd.sort(id..g).n_rows()\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "Column<Int> · 6 values\n2\n4\n1\n3\n6\n5\n\
         Column<Int> · 6 values\n1\n3\n6\n2\n4\n5\n\
         Column<Int> · 6 values\n4\n6\n2\n1\n5\n3\n\
         Column<Int> · 6 values\n5\n1\n2\n6\n4\n3\n\
         Column<Int> · 6 values\n2\n5\n1\n3\n6\n4\n\
         Column<Int> · 6 values\n4\n2\n6\n1\n3\n5\n\
         6\n"
    );
    let mut all = String::new();
    for script in [
        "let w = import(\"data/wages.csv\")\nw.sort(wge)\n",
        "let w = import(\"data/wages.csv\")\nw.sort(wage, desc: 1)\n",
        "let w = import(\"data/wages.csv\")\nw.sort(\"wage\")\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    insta::assert_snapshot!("sort_refusals", all);
    let (code, stdout, stderr) = go(
        "let e = import(\"data/empty.csv\", types: {x: Float, y: Bool})\ne.sort(x).n_rows()\ne.add(z = 1).n_cols()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "0\n3\n");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(12)))]

    #[test]
    fn sort_is_a_stable_permutation_with_na_last(values in prop::collection::vec(prop::option::of(-5i64..5), 1..14), desc in any::<bool>()) {
        let csv: String = std::iter::once("id,x".to_string())
            .chain(values.iter().enumerate().map(|(i, v)| format!("{},{}", i + 1, v.map_or(String::new(), |i| i.to_string()))))
            .collect::<Vec<_>>()
            .join("\n") + "\n";
        let script = format!("let d = import(\"data/p.csv\", types: {{id: Int, x: Int}})\nd.sort(x, desc: {desc})[\"id\"]\n");
        let outcome = run_with(&[("data/p.csv", csv.as_bytes())], &script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let mut lines = outcome.output.stdout.lines();
        let got = take_column(&mut lines);
        // The expected order: a stable sort of the row ids by their value, NA last.
        let mut expected: Vec<(usize, Option<i64>)> = values.iter().cloned().enumerate().collect();
        expected.sort_by(|(_, a), (_, b)| match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => if desc { b.cmp(a) } else { a.cmp(b) },
        });
        let expected: Vec<i64> = expected.iter().map(|(i, _)| *i as i64 + 1).collect();
        prop_assert_eq!(got, expected);
    }
}
