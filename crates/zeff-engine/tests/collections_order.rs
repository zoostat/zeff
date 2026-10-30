//! The order functions (Language Reference, sections 3.7 "Identity", "Mappings" and 4.1
//! "Sorting"; section 6 "Row order"; release 0.5.0, step 12): `unique`, `sorted`, `is_in`,
//! `keys` and `equal` by the identity relation; all `NA`, `-0.0` beside `0.0`, an empty list, a
//! category column; every message; and the properties of the checklist.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn prints(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}: no message expected");
    out.output.stdout
}

fn error_code(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 1, "{script}:\n{}", out.output.stderr);
    let stderr = &out.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

#[test]
fn unique_keeps_first_occurrences_by_identity() {
    assert_eq!(prints("unique([3, 1, 3, 2, 1])\n"), "3\n1\n2\n");
    assert_eq!(prints("unique([NA, NA])\n"), "NA\n");
    assert_eq!(prints("unique([NA, 2, NA])\n"), "NA\n2\n");
    // -0.0 is identical to 0.0: one of them is kept (both print as 0).
    assert_eq!(prints("unique([-0.0, 0.0, 1.0])\n"), "0\n1\n");
    assert_eq!(prints("unique([0.0, -0.0])\n"), "0\n");
    assert_eq!(prints("unique([])\n"), "[]\n");
    assert_eq!(prints("unique([\"b\", \"a\", \"b\"])\n"), "b\na\n");
    // An Int and the Float of the same number are one item.
    assert_eq!(prints("unique([1, 1.0, 2])\n"), "1\n2\n");
}

#[test]
fn sorted_follows_the_order_rules_with_na_last_either_way() {
    assert_eq!(prints("sorted([3, 1, 2])\n"), "1\n2\n3\n");
    assert_eq!(prints("sorted([3, NA, 1, NA])\n"), "1\n3\nNA\nNA\n");
    assert_eq!(
        prints("sorted([3, NA, 1, NA], desc: true)\n"),
        "3\n1\nNA\nNA\n"
    );
    assert_eq!(prints("sorted([NA, NA])\n"), "NA\nNA\n");
    assert_eq!(prints("sorted([])\n"), "[]\n");
    assert_eq!(
        prints("sorted([true, false, true])\n"),
        "false\ntrue\ntrue\n"
    );
    // Text by code point: capitals before lower case, an accented letter after.
    assert_eq!(
        prints("sorted([\"b\", \"a\", \"\\u{e9}\", \"B\"])\n"),
        "B\na\nb\né\n"
    );
    // Stable: -0.0 and 0.0 keep their order; Int and Float order as numbers.
    assert_eq!(prints("sorted([0.0, -0.0, -1.0])\n"), "-1\n0\n0\n");
    assert_eq!(prints("sorted([2.5, 1, 3])\n"), "1\n2.5\n3\n");
    // A list of records cannot be ordered.
    assert_eq!(error_code("sorted([{a: 1}, {a: 2}])\n"), "Z0401");
}

#[test]
fn is_in_uses_identity_and_lifts_over_a_column() {
    assert_eq!(
        prints("is_in(2, [1, 2, 3])\nis_in(5, [1, 2, 3])\n"),
        "true\nfalse\n"
    );
    assert_eq!(
        prints("is_in(NA, [1, NA])\nis_in(NA, [1, 2])\n"),
        "true\nfalse\n"
    );
    assert_eq!(
        prints("is_in(0.0, [-0.0])\nis_in(1, [1.0])\n"),
        "true\ntrue\n"
    );
    assert_eq!(prints("is_in(\"a\", [\"a\", \"b\"])\n"), "true\n");
    assert_eq!(prints("is_in(1, [])\n"), "false\n");
    let out = prints(
        "let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])\nd.where(is_in(x, [1, 3, 5]))[\"y\"]\nd.add(f = is_in(x, [2]))[\"f\"]\n",
    );
    assert_eq!(
        out,
        "Column<Int> · 3 values\n2\n7\n11\nColumn<Bool> · 5 values\nfalse\ntrue\nfalse\nfalse\nfalse\n"
    );
    // A category column matches by label, as its rows are its labels.
    let out = run(
        "let w = dataset(wages)\nw.where(is_in(industry, [\"Retail\"])).n_rows() > 0\nw.where(is_in(industry, [\"Nowhere\"])).n_rows()\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert!(
        out.output.stdout.starts_with("true\n0\n"),
        "{}",
        out.output.stdout
    );
}

#[test]
fn keys_of_a_record_and_a_mapping() {
    assert_eq!(prints("keys({mu: 0.0, sd: 1.0})\n"), "mu\nsd\n");
    assert_eq!(prints("keys([2 => \"b\", 1 => \"a\"])\n"), "2\n1\n");
    assert_eq!(
        prints("let m = [0 => \"male\", 1 => \"female\"]\nis_in(2, keys(m))\nis_in(1, keys(m))\n"),
        "false\ntrue\n"
    );
    // The keys of a record are text, so they sort as text.
    assert_eq!(prints("sorted(keys({b: 1, a: 2}))\n"), "a\nb\n");
    assert_eq!(error_code("keys([1, 2])\n"), "Z0401");
}

#[test]
fn equal_is_identity_over_values_and_collections() {
    assert_eq!(
        prints("equal(1, 1)\nequal(1, 1.0)\nequal(NA, NA)\nequal(-0.0, 0.0)\n"),
        "true\ntrue\ntrue\ntrue\n"
    );
    assert_eq!(
        prints("equal(1, 2)\nequal(1, \"1\")\nequal(true, 1)\nequal(NA, 1)\n"),
        "false\nfalse\nfalse\nfalse\n"
    );
    assert_eq!(
        prints(
            "equal([1, 2, NA], [1, 2, NA])\nequal([1, 2], [2, 1])\nequal([1], [1, 1])\nequal([], [])\n"
        ),
        "true\nfalse\nfalse\ntrue\n"
    );
    assert_eq!(
        prints(
            "equal({a: 1, b: 2}, {b: 2, a: 1})\nequal({a: 1}, {a: 2})\nequal({a: 1}, {a: 1, b: 2})\n"
        ),
        "true\nfalse\nfalse\n"
    );
    assert_eq!(
        prints(
            "equal([1 => \"a\"], [1 => \"a\"])\nequal([1 => \"a\", 2 => \"b\"], [2 => \"b\", 1 => \"a\"])\n"
        ),
        "true\nfalse\n"
    );
    assert_eq!(
        prints("equal([[1, 2], [3]], [[1, 2], [3]])\nequal([1, 2], {a: 1})\n"),
        "true\nfalse\n"
    );
    // A column is not among the values equal compares.
    assert_eq!(
        error_code("let d = frame(x = [1, 2])\nequal(d[\"x\"], d[\"x\"])\n"),
        "Z0401"
    );
}

#[test]
fn every_message_of_the_order_functions() {
    let cases: Vec<(&str, &str)> = vec![
        ("sorted_records", "sorted([{a: 1}, {a: 2}])\n"),
        ("sorted_desc_not_bool", "sorted([1, 2], desc: 1)\n"),
        ("is_in_not_a_list", "is_in(1, 2)\n"),
        ("is_in_record", "is_in({a: 1}, [1])\n"),
        ("keys_of_a_list", "keys([1, 2])\n"),
        (
            "equal_column",
            "let d = frame(x = [1, 2])\nequal(d[\"x\"], d[\"x\"])\n",
        ),
        ("unique_not_a_list", "unique(\"abc\")\n"),
    ];
    let mut text = String::new();
    for (name, script) in &cases {
        let out = run(script);
        text.push_str(&format!("## {name} (exit {})\n", out.exit_code));
        text.push_str(&out.output.stdout);
        text.push_str(&out.output.stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("every_message", text);
}

/// A list literal of Ints, some `NA`, as a script writes it.
fn literal(xs: &[Option<i64>]) -> String {
    format!(
        "[{}]",
        xs.iter()
            .map(|x| x.map_or("NA".to_string(), |v| v.to_string()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// `sorted(xs)` is a permutation of `xs` in order with `NA` last; `sorted(xs, desc: true)`
    /// is `reversed(sorted(xs))` apart from `NA` staying last; `unique` is idempotent;
    /// `is_in(x, xs)` iff `x` is in `unique(xs)`; `equal(xs, xs)` is `true`.
    #[test]
    fn the_properties_of_the_checklist(
        xs in prop::collection::vec(prop::option::weighted(0.85, -50i64..50), 1..14),
        x in -50i64..50,
    ) {
        let list = literal(&xs);
        let out = run(&format!(
            "let xs = {list}\nsorted(xs)\nequal(unique(unique(xs)), unique(xs))\nequal(xs, xs)\nis_in({x}, xs)\nis_in({x}, unique(xs))\nequal(sorted(xs, desc: true), reversed(sorted(xs)))\n"
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        let n = xs.len();
        let sorted_lines = &lines[..n];
        // A permutation: the same multiset, written as Ints and NA.
        let mut expected: Vec<String> = xs.iter().map(|x| x.map_or("NA".to_string(), |v| v.to_string())).collect();
        let mut found: Vec<String> = sorted_lines.iter().map(|l| l.to_string()).collect();
        expected.sort();
        found.sort();
        prop_assert_eq!(&found, &expected, "not a permutation");
        // In order, NA last.
        let values: Vec<Option<i64>> = sorted_lines.iter().map(|l| l.parse::<i64>().ok()).collect();
        let first_na = values.iter().position(Option::is_none).unwrap_or(n);
        prop_assert!(values[first_na..].iter().all(Option::is_none), "NA not last: {:?}", values);
        prop_assert!(values[..first_na].windows(2).all(|w| w[0] <= w[1]), "not ascending: {:?}", values);
        prop_assert_eq!(lines[n], "true", "unique is not idempotent");
        prop_assert_eq!(lines[n + 1], "true", "equal(xs, xs)");
        let in_xs = xs.contains(&Some(x));
        prop_assert_eq!(lines[n + 2], if in_xs { "true" } else { "false" });
        prop_assert_eq!(lines[n + 2], lines[n + 3], "is_in differs between xs and unique(xs)");
        // Descending is the reverse of ascending when no NA is present.
        if xs.iter().all(Option::is_some) {
            prop_assert_eq!(lines[n + 4], "true", "desc is not the reverse");
        }
    }
}
