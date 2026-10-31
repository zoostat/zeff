//! `assert`, `n_missing` and `fill_missing` (catalogue rows 62, 58, 56; Language Reference,
//! sections 3.7, 4.1, 5.4, 5.6 and 3.8; release 0.5.0, step 15): a check that holds prints
//! nothing, a check that fails stops with Z0001 and exit code 1, `NA` and a column are Z0203;
//! the counts of missing values on lists and columns, all `NA` and none; filling with the type
//! joined; every message; and the properties of the checklist.

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
fn assert_prints_nothing_when_the_check_holds_and_stops_when_it_fails() {
    assert_eq!(
        prints("assert(1 + 1 == 2, \"arithmetic\")\nassert(true, \"plain\")\nprint(\"after\")\n"),
        "after\n"
    );
    let out =
        run("print(\"before\")\nassert(2 > 3, \"two is not above three\")\nprint(\"never\")\n");
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "before\n");
    assert!(
        out.output
            .stderr
            .contains("error[Z0001]: two is not above three"),
        "{}",
        out.output.stderr
    );
    assert!(
        out.output.stderr.contains("main.zeff:2"),
        "{}",
        out.output.stderr
    );
    // The way a project tests its own functions.
    assert_eq!(
        prints(
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nassert(equal([double(1), double(2)], [2, 4]), \"double\")\n"
        ),
        ""
    );
    // Inside a function, as section 3.7 allows.
    assert_eq!(
        error_code(
            "/// Checked.\nfn f(x: Float) -> Float {\n    assert(x > 0, \"x must be positive\")\n    x\n}\nf(-1)\n"
        ),
        "Z0001"
    );
}

#[test]
fn assert_on_na_or_a_column_is_z0203() {
    // NA at run time, named for `assert`, not for the `if` inside its body.
    let out = run(
        "let d = frame(x = [1, NA])\nassert(first(reversed([d.mean(x), to(NA, Float)])) > 0, \"positive\")\n",
    );
    assert_eq!(out.exit_code, 1, "{}", out.output.stderr);
    assert!(
        out.output
            .stderr
            .contains("error[Z0203]: `assert` needs one Bool, but the condition is NA"),
        "{}",
        out.output.stderr
    );
    // A column at check time.
    assert_eq!(
        error_code("let d = frame(x = [1, 2])\nassert(d[\"x\"] > 0, \"positive\")\n"),
        "Z0203"
    );
    // The repair the message names.
    assert_eq!(
        prints("let d = frame(x = [1, 2])\nassert(all(d[\"x\"] > 0), \"positive\")\n"),
        ""
    );
    // A value that is not a Bool at all.
    assert_eq!(error_code("assert(1, \"one\")\n"), "Z0401");
}

#[test]
fn n_missing_counts_na_in_lists_and_columns() {
    assert_eq!(
        prints("n_missing([1, NA, 3, NA])\nn_missing([1, 2, 3])\nn_missing([NA, NA])\n"),
        "2\n0\n2\n"
    );
    assert_eq!(
        prints("n_missing([\"a\", NA])\nn_missing([true, NA, false])\n"),
        "1\n1\n"
    );
    assert_eq!(
        prints(
            "let d = frame(x = [1, NA, 3], s = [\"a\", NA, \"c\"])\nn_missing(d[\"x\"])\nn_missing(d[\"s\"])\n"
        ),
        "1\n1\n"
    );
    assert_eq!(
        prints("let w = dataset(wages)\nn_missing(w[\"wage\"])\nn_missing(w[\"educ\"])\n"),
        "211\n0\n"
    );
    // A list of failed items counts them neither way, with the warning of `len`.
    let out = run(
        "let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])\nlet fits = [for cut in [0, 10] { attempt(d.where(x > cut).reg(y ~ x)) }]\nn_missing(fits)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "0\n");
    assert!(
        out.output.stderr.contains("1 of 2 items failed"),
        "{}",
        out.output.stderr
    );
    assert_eq!(error_code("n_missing(3)\n"), "Z0401");
}

#[test]
fn fill_missing_joins_the_types_and_removes_every_na() {
    assert_eq!(
        prints(
            "fill_missing(to(NA, Int), 0)\nfill_missing(2.5, 0)\nfill_missing(to(NA, Str), \"none\")\n"
        ),
        "0\n2.5\nnone\n"
    );
    // Int fills Floats as a Float; the type is the joined one.
    assert_eq!(
        prints("type_of(fill_missing(2.5, 0))\ntype_of(fill_missing(2, 0))\n"),
        "Float\nInt\n"
    );
    let out = prints(
        "let d = frame(x = [1, NA, 3], s = [\"a\", NA, \"c\"])\nfill_missing(d[\"x\"], 0)\nfill_missing(d[\"s\"], \"none\")\nn_missing(fill_missing(d[\"x\"], 0))\n",
    );
    assert_eq!(
        out,
        "Column<Int> · 3 values\n1\n0\n3\nColumn<Str> · 3 values\na\nnone\nc\n0\n"
    );
    // Inside `add`, as a cleaning step the analyst chose.
    assert_eq!(
        prints("let d = frame(x = [1, NA, 3])\nd.add(y = fill_missing(x, 0)).n_rows()\n"),
        "3\n"
    );
    // A text cannot fill numbers.
    assert_eq!(error_code("fill_missing(2.5, \"none\")\n"), "Z0214");
    assert_eq!(
        error_code("let d = frame(x = [1, NA, 3])\nfill_missing(d[\"x\"], \"none\")\n"),
        "Z0214"
    );
}

#[test]
fn every_message_of_the_three() {
    let cases: Vec<(&str, &str)> = vec![
        (
            "assert_fails",
            "assert(2 > 3, \"two is not above three\")\n",
        ),
        ("assert_na", "assert(to(NA, Bool), \"missing\")\n"),
        (
            "assert_column",
            "let d = frame(x = [1, 2])\nassert(d[\"x\"] > 0, \"positive\")\n",
        ),
        ("assert_not_bool", "assert(1, \"one\")\n"),
        ("assert_no_message", "assert(true)\n"),
        ("n_missing_not_a_list", "n_missing(3)\n"),
        ("fill_missing_no_join", "fill_missing(2.5, \"none\")\n"),
        (
            "fill_missing_column_no_join",
            "let d = frame(x = [1, NA, 3])\nfill_missing(d[\"x\"], \"none\")\n",
        ),
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

#[test]
fn the_page_of_assert_comes_from_its_zeff_source() {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let f = registry.function("core", "assert").expect("assert");
    assert_eq!(f.spec.returns, "Nothing");
    let codes: Vec<&str> = f.spec.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&"Z0001"), "{codes:?}");
    insta::assert_snapshot!(
        "assert_page",
        zeff_engine::help::help_text(registry, "assert").unwrap()
    );
}

/// A list literal of Ints with `NA`, as a script writes it.
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

    /// `n_missing(fill_missing(x, v))` is 0, and `len(x)` is `n_missing(x)` plus the count of
    /// the values present, on a list and on the column made from it.
    #[test]
    fn the_counts_add_up_and_filling_removes_every_na(
        xs in prop::collection::vec(prop::option::weighted(0.6, -100i64..100), 1..12),
        v in -100i64..100,
    ) {
        let present = xs.iter().filter(|x| x.is_some()).count();
        // A column of only `NA` has no type (Z0219); the all-`NA` list is a unit test's case.
        prop_assume!(present > 0);
        let list = literal(&xs);
        let missing = xs.len() - present;
        let out = run(&format!(
            "let xs = {list}\nn_missing(xs)\nn_missing(fill_missing(frame(x = xs)[\"x\"], {v}))\nlen(xs) == n_missing(xs) + {present}\nlet d = frame(x = xs)\nn_missing(d[\"x\"])\nn_missing(d.add(y = fill_missing(x, {v}))[\"y\"])\n"
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, format!("{missing}\n0\ntrue\n{missing}\n0\n"));
    }
}
