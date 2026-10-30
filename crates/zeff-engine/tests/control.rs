//! `if` and `case` (Language Reference, sections 3.1, 3.7 and 5.1; release 0.5.0, step 2): the
//! examples of section 5.1, blocks and their scope, `case` over one value and row by row inside
//! `add` and `where`, every warning and error, a snapshot of every message, and the properties
//! that a `case` over a column equals the `case` applied to each row and that `if` and a two-arm
//! `case` agree on one value.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

const SALES: &str = "id,sold,amount\n1,2026-01-05,10\n2,2026-02-09,20\n";

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/sales.csv", SALES);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// What a script prints, with no message at all.
fn prints(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "", "{script}: no message expected");
    outcome.output.stdout
}

/// The code of the error that stopped a script.
fn error_code(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 1, "{script}:\n{}", outcome.output.stderr);
    let stderr = &outcome.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

// ----- if (section 5.1) -----

#[test]
fn if_as_a_statement_runs_its_block_only_when_the_condition_is_true() {
    let outcome = run(
        "let count = 3\nif count < 30 { warn(\"fewer than 30 rows: treat large-sample tests with care\") }\nif count > 30 { warn(\"many rows\") }\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "");
    assert_eq!(
        outcome.output.stderr,
        "warning: fewer than 30 rows: treat large-sample tests with care (line 2)\n"
    );
}

#[test]
fn if_without_else_prints_nothing_and_if_with_else_is_a_value() {
    assert_eq!(prints("if true { 5 }\n"), "");
    assert_eq!(prints("if false { 5 }\n"), "");
    assert_eq!(prints("if true { 5 } else { 6 }\n"), "5\n");
    assert_eq!(prints("if false { 5 } else { 6 }\n"), "6\n");
    assert_eq!(
        prints("let count = 3\nlet reps = if count < 500 { 5000 } else { 1000 }\nreps\n"),
        "5000\n"
    );
    // Int and Float join as Float (section 3.7, "Joining types").
    assert_eq!(prints("let v = if true { 1 } else { 2.5 }\nv\n"), "1\n");
    assert_eq!(prints("let v = if false { 1 } else { 2.5 }\nv\n"), "2.5\n");
}

#[test]
fn else_if_chains_choose_the_first_true_branch() {
    for (count, printed) in [(10, "big"), (3, "medium"), (0, "small")] {
        assert_eq!(
            prints(&format!(
                "let count = {count}\nif count > 5 {{ \"big\" }} else if count > 1 {{ \"medium\" }} else {{ \"small\" }}\n"
            )),
            format!("{printed}\n")
        );
    }
    // An `else if` without a final `else` is a statement and gives nothing.
    assert_eq!(prints("if false { 1 } else if true { 2 }\n"), "");
}

#[test]
fn blocks_have_their_own_scope_and_the_value_of_their_last_expression() {
    assert_eq!(
        prints("let x = if true {\n    let y = 2\n    y * 3\n} else {\n    0\n}\nx\n"),
        "6\n"
    );
    // A statement before the last is evaluated for its effects and does not print.
    assert_eq!(
        prints("let v = if true {\n    print(\"side\")\n    5\n    7\n} else {\n    0\n}\nv\n"),
        "side\n7\n"
    );
    // A `let` inside a block is not visible after it (section 3.7, "Block scope").
    assert_eq!(
        error_code("if true {\n    let inner = 1\n}\ninner\n"),
        "Z0101"
    );
    // A block that ends in a `let` gives nothing.
    assert_eq!(prints("if true {\n    let y = 1\n}\n"), "");
}

#[test]
fn an_if_condition_must_be_one_bool() {
    assert_eq!(error_code("if 1 { 2 }\n"), "Z0203");
    assert_eq!(error_code("if \"yes\" { 2 }\n"), "Z0203");
    assert_eq!(
        error_code("let d = frame(x = [1, 2])\nif d[\"x\"] > 1 { 2 }\n"),
        "Z0203"
    );
    // `NA` is found when the statement runs (section 5.6).
    assert_eq!(error_code("if [NA, true][1] { 2 }\n"), "Z0203");
    // A name that is both a variable and a binding is a column when the statement runs.
    let outcome =
        run("let x = true\nlet d = frame(x = [1, 2])\nd.add(y = if x { 1 } else { 0 })\n");
    assert_eq!(outcome.exit_code, 1, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stderr
            .contains("error[Z0203]: `if` needs one Bool, but `x` is an Int column"),
        "{}",
        outcome.output.stderr
    );
    // `Nothing` cannot be a value or a condition (section 3.7).
    assert_eq!(error_code("let x = if true { 1 }\n"), "Z0206");
    assert_eq!(error_code("if print(1) { 2 }\n"), "Z0206");
    // The branches of an `if` used as a value give one type.
    assert_eq!(
        error_code("let x = if true { 1 } else { \"a\" }\n"),
        "Z0214"
    );
    assert_eq!(
        error_code("let x = if true { let y = 1 } else { 2 }\n"),
        "Z0206"
    );
    assert_eq!(
        error_code("let x = if true { 1 } else if false { 2 }\n"),
        "Z0206"
    );
}

// ----- case over one value (section 5.1) -----

#[test]
fn case_over_one_value_takes_the_first_true_arm() {
    for (count, printed) in [(10, "big"), (3, "medium"), (0, "small")] {
        assert_eq!(
            prints(&format!(
                "let count = {count}\ncase {{ count > 5 => \"big\", count > 1 => \"medium\", _ => \"small\" }}\n"
            )),
            format!("{printed}\n")
        );
    }
    // Only the winning arm's value is evaluated: the others may be anything.
    assert_eq!(prints("case { 1 < 2 => 1, _ => [1, 2, 3][9] }\n"), "1\n");
    // An arm whose condition is `NA` does not match.
    assert_eq!(
        prints("case { [NA, true][1] => \"x\", _ => \"y\" }\n"),
        "y\n"
    );
    // Arms may be blocks with their own scope.
    assert_eq!(
        prints(
            "case {\n    1 < 2 => {\n        let y = 4\n        y * 2\n    },\n    _ => 0,\n}\n"
        ),
        "8\n"
    );
}

#[test]
fn case_over_one_value_without_a_match_is_na_with_a_warning() {
    let outcome = run("let count = 0\ncase { count > 5 => \"big\", count > 1 => \"medium\" }\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "NA\n");
    assert_eq!(
        outcome.output.stderr,
        "warning: no arm of the `case` matched, so its value is NA (line 2)\n  = help: add a last arm for the rest, `_ => value`\n"
    );
}

#[test]
fn case_conditions_and_values_are_checked() {
    assert_eq!(error_code("case { 1 => 2, _ => 3 }\n"), "Z0203");
    assert_eq!(error_code("case { \"a\" => 2, _ => 3 }\n"), "Z0203");
    assert_eq!(error_code("case { 1 < 2 => 1, _ => \"a\" }\n"), "Z0214");
    assert_eq!(error_code("case { 1 < 2 => print(1), _ => 2 }\n"), "Z0206");
    assert_eq!(error_code("case { _ => 1, true => 2 }\n"), "Z0316");
    assert_eq!(error_code("case { }\n"), "Z0301");
    let outcome = run("case { true => 1, false => 2 }\n");
    assert_eq!(outcome.exit_code, 0);
    assert!(
        outcome.output.stderr.starts_with("warning[W0301]"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(outcome.output.stdout, "1\n");
}

// ----- case row by row (sections 3.7 and 5.1) -----

#[test]
fn case_row_by_row_inside_add_follows_section_5_1() {
    let printed = prints(
        "let people = frame(age = [24, 51, 60, NA])\nlet people = people.add(age_group = case {\n    age < 25 => \"young\",\n    age < 55 => \"prime\",\n    _        => \"older\",\n})\npeople\n",
    );
    assert_eq!(
        printed,
        "people · 4 rows × 2 variables\n\n      age   age_group\n  1    24   young\n  2    51   prime\n  3    60   older\n  4    NA   older\n"
    );
}

#[test]
fn case_row_by_row_without_a_match_gives_na_with_the_count() {
    let outcome = run(
        "let people = frame(age = [24, 51, 60, NA])\npeople.add(age_group = case { age < 25 => \"young\", age < 55 => \"prime\" })\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .contains("  3    60   NA\n  4    NA   NA\n"),
        "{}",
        outcome.output.stdout
    );
    assert_eq!(
        outcome.output.stderr,
        "warning: 2 rows matched no arm of the `case` and are NA (line 2)\n  = help: add a last arm for the rest, `_ => value`\n"
    );
    let one = run("let d = frame(x = [1, 5])\nd.add(y = case { x < 3 => 0 })\n");
    assert!(
        one.output
            .stderr
            .starts_with("warning: 1 row matched no arm of the `case` and is NA"),
        "{}",
        one.output.stderr
    );
}

#[test]
fn case_mixes_scalars_and_columns_and_works_in_where() {
    // A scalar value beside a column condition stands for every row; a column value supplies
    // the rows its arm wins; `true` last catches the rest.
    assert_eq!(
        prints("let d = frame(x = [1, 5, 9])\nd.add(y = case { x < 3 => 0, true => x })\n"),
        "frame · 3 rows × 2 variables\n\n      x   y\n  1   1   0\n  2   5   5\n  3   9   9\n"
    );
    // Int and Float arms join as Float across the rows.
    assert_eq!(
        prints("let d = frame(x = [1, 5, 9])\nd.add(y = case { x < 3 => 0.5, _ => x })\n"),
        "frame · 3 rows × 2 variables\n\n      x     y\n  1   1   0.5\n  2   5     5\n  3   9     9\n"
    );
    // A scalar condition that is true gives its value, a column, as it is.
    assert_eq!(
        prints("let d = frame(x = [1, 5, 9])\ncase { 1 < 2 => d[\"x\"], _ => 0 }\n"),
        "Column<Int> · 3 values\n1\n5\n9\n"
    );
    // A `case` of Bools is a condition for `where`.
    assert_eq!(
        prints(
            "let d = frame(x = [1, 5, 9])\nd.where(case { x > 4 => true, _ => false }).n_rows()\n"
        ),
        "2\n"
    );
    // Outside a frame method, columns are values (section 3.7, "Columns and scalars").
    assert_eq!(
        prints(
            "let d = frame(x = [1, 5, 9])\nlet x = d[\"x\"]\ncase { x > 4 => x * 10, _ => x }\n"
        ),
        "Column<Int> · 3 values\n1\n50\n90\n"
    );
}

#[test]
fn case_row_by_row_refuses_what_it_cannot_combine() {
    // Columns of two lengths (Z0217).
    assert_eq!(
        error_code(
            "let d = frame(x = [1, 2])\nlet e = frame(y = [1, 2, 3])\ncase { d[\"x\"] > 1 => e[\"y\"], _ => 0 }\n"
        ),
        "Z0217"
    );
    assert_eq!(
        error_code(
            "let d = frame(x = [1, 2])\nlet e = frame(y = [true, false, true])\ncase { e[\"y\"] => 1, d[\"x\"] > 1 => 2, _ => 0 }\n"
        ),
        "Z0217"
    );
    // A condition column that is not Bool, seen when the statement runs.
    assert_eq!(
        error_code("let x = true\nlet d = frame(x = [1, 2])\nd.add(y = case { x => 1, _ => 0 })\n"),
        "Z0203"
    );
    // Types that do not join across the rows, seen when the statement runs.
    assert_eq!(
        error_code(
            "let d = frame(x = [1, 2], s = [\"a\", \"b\"])\nd.add(y = case { x > 1 => s, _ => x })\n"
        ),
        "Z0214"
    );
    // A Category is not turned into its labels unasked.
    assert_eq!(
        error_code("dataset(wages).add(g = case { wage < 15 => industry, _ => \"other\" })\n"),
        "Z0214"
    );
    // The column types of 0.6.0 wait for it.
    assert_eq!(
        error_code(
            "let s = import(\"data/sales.csv\")\ns.add(d = case { amount > 15 => sold, _ => NA })\n"
        ),
        "Z0304"
    );
}

#[test]
fn every_message_of_if_and_case() {
    let cases = [
        ("if_on_an_int", "if 1 { 2 }\n"),
        ("if_on_text", "if \"yes\" { 2 }\n"),
        (
            "if_on_a_column",
            "let d = frame(x = [1, 2])\nif d[\"x\"] > 1 { 2 }\n",
        ),
        ("if_on_na", "if [NA, true][1] { 2 }\n"),
        (
            "if_on_a_column_found_at_run_time",
            "let x = true\nlet d = frame(x = [1, 2])\nd.add(y = if x { 1 } else { 0 })\n",
        ),
        ("if_without_else_bound", "let x = if true { 1 }\n"),
        ("if_on_nothing", "if print(1) { 2 }\n"),
        (
            "if_branches_differ",
            "let x = if true { 1 } else { \"a\" }\n",
        ),
        (
            "if_branch_without_a_value",
            "let x = if true { let y = 1 } else { 2 }\n",
        ),
        ("case_arm_not_a_condition", "case { 1 => 2, _ => 3 }\n"),
        ("case_arms_differ", "case { 1 < 2 => 1, _ => \"a\" }\n"),
        (
            "case_arm_gives_nothing",
            "case { 1 < 2 => print(1), _ => 2 }\n",
        ),
        ("case_catch_all_first", "case { _ => 1, true => 2 }\n"),
        ("case_empty", "case { }\n"),
        ("case_unreachable_arm", "case { true => 1, false => 2 }\n"),
        (
            "case_no_match",
            "let count = 0\ncase { count > 5 => \"big\" }\n",
        ),
        (
            "case_rows_no_match",
            "let d = frame(x = [1, 5, 9, NA])\nd.add(y = case { x < 3 => 0 })\n",
        ),
        (
            "case_rows_of_two_lengths",
            "let d = frame(x = [1, 2])\nlet e = frame(y = [1, 2, 3])\ncase { d[\"x\"] > 1 => e[\"y\"], _ => 0 }\n",
        ),
        (
            "case_condition_column_not_bool",
            "let x = true\nlet d = frame(x = [1, 2])\nd.add(y = case { x => 1, _ => 0 })\n",
        ),
        (
            "case_rows_of_two_types",
            "let d = frame(x = [1, 2], s = [\"a\", \"b\"])\nd.add(y = case { x > 1 => s, _ => x })\n",
        ),
        (
            "case_category",
            "dataset(wages).add(g = case { wage < 15 => industry, _ => \"other\" })\n",
        ),
        (
            "case_date",
            "let s = import(\"data/sales.csv\")\ns.add(d = case { amount > 15 => sold, _ => NA })\n",
        ),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}

#[test]
fn the_examples_of_section_5_1() {
    let script = "let wages = dataset(wages)\nif wages.n_rows() < 30 {\n    warn(\"fewer than 30 rows: treat large-sample tests with care\")\n}\nlet reps = if wages.n_rows() < 500 { 5000 } else { 1000 }\nreps\nlet wages = wages.add(age_group = case {\n    age < 25 => \"young\",\n    age < 55 => \"prime\",\n    _        => \"older\",\n})\nwages.head(5)\nwages.where(age_group == \"young\").n_rows()\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "section_5_1",
        format!("{}{}", outcome.output.stdout, outcome.output.stderr)
    );
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// A `case` over a column equals the `case` applied to each row's value, and with `_` it
    /// never warns.
    #[test]
    fn a_case_over_a_column_equals_the_case_row_by_row(
        xs in prop::collection::vec(prop::option::weighted(0.85, -50i64..50), 1..12),
        a in -50i64..50,
        b in -50i64..50,
    ) {
        let mut xs = xs;
        if xs.iter().all(Option::is_none) {
            xs[0] = Some(1);
        }
        let text = |v: &Option<i64>| v.map_or("NA".to_string(), |i| i.to_string());
        let list: Vec<String> = xs.iter().map(text).collect();
        let arms = format!("x < {a} => \"low\", x < {b} => \"mid\", _ => \"high\"");
        let outcome = run(&format!(
            "let d = frame(x = [{}])\nlet e = d.add(g = case {{ {arms} }})\ne[\"g\"]\n",
            list.join(", ")
        ));
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        prop_assert_eq!(outcome.output.stderr.as_str(), "");
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        prop_assert_eq!(lines.len(), 1 + xs.len());
        let mut per_row = String::new();
        for v in &list {
            let x = if v == "NA" { "[NA, 0][1]".to_string() } else { v.clone() };
            per_row.push_str(&format!("let x = {x}\ncase {{ {arms} }}\n"));
        }
        let values = run(&per_row);
        prop_assert_eq!(values.exit_code, 0, "{}", values.output.stderr);
        prop_assert_eq!(values.output.stderr.as_str(), "");
        let value_rows: Vec<&str> = values.output.stdout.lines().collect();
        prop_assert_eq!(lines[1..].to_vec(), value_rows);
        // The same in Rust: the first true arm, NA matching nothing.
        let expected: Vec<&str> = xs
            .iter()
            .map(|x| match x {
                Some(x) if *x < a => "low",
                Some(x) if *x < b => "mid",
                _ => "high",
            })
            .collect();
        prop_assert_eq!(lines[1..].to_vec(), expected);
    }

    /// `if c { a } else { b }` and `case { c => a, _ => b }` agree on one value.
    #[test]
    fn an_if_and_a_two_arm_case_agree(c in any::<bool>(), a in -1000i64..1000, b in -1000i64..1000) {
        // A statement may not begin with a unary minus (Z0305), so the values are bracketed.
        let with_if = run(&format!("let c = {c}\nif c {{ ({a}) }} else {{ ({b}) }}\n"));
        let with_case = run(&format!("let c = {c}\ncase {{ c => ({a}), _ => ({b}) }}\n"));
        prop_assert_eq!(with_if.exit_code, 0, "{}", with_if.output.stderr);
        prop_assert_eq!(with_case.exit_code, 0, "{}", with_case.output.stderr);
        prop_assert_eq!(with_if.output.stderr.as_str(), "");
        prop_assert_eq!(with_case.output.stderr.as_str(), "");
        prop_assert_eq!(&with_if.output.stdout, &with_case.output.stdout);
        prop_assert_eq!(with_if.output.stdout.trim(), (if c { a } else { b }).to_string());
    }
}
