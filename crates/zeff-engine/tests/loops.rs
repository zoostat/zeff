//! `for`, `[for ...]`, `break` and `continue` (Language Reference, sections 3.7 and 5.2; release
//! 0.5.0, step 3): loops over lists, ranges and variables, the loop variable in formulas and
//! variable arguments, each pass's own `let`s, the jumps, the draws of each pass, every message,
//! and the properties that `[for i in 1..n { i }]` is the range, that `break` at pass k leaves
//! k − 1 items, and that a loop over variables fits the formulas as written, bit for bit.

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

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
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

// ----- for over lists and ranges (section 5.2) -----

#[test]
fn a_for_loop_runs_its_body_once_per_item_and_prints_nothing_itself() {
    assert_eq!(prints("for i in 1..3 { print(i) }\n"), "1\n2\n3\n");
    assert_eq!(
        prints("for s in [\"a\", \"b\"] {\n    print(s)\n}\n"),
        "a\nb\n"
    );
    // The body's last expression is not printed: only the script's own statements print.
    assert_eq!(prints("for i in 1..3 { i * 10 }\n"), "");
    // An empty range runs no pass (section 5.6: `5..2` is an empty list).
    assert_eq!(prints("for i in 5..2 { print(i) }\n"), "");
}

#[test]
fn a_list_for_collects_one_value_per_pass() {
    assert_eq!(
        prints("let squares = [for i in 1..5 { i * i }]\nsquares\n"),
        "1\n4\n9\n16\n25\n"
    );
    // Int and Float join as Float across the list's type (one body, one type).
    assert_eq!(prints("[for x in [1, 2.5] { x * 2 }]\n"), "2\n5\n");
    // A nested loop collects a list of lists.
    assert_eq!(
        prints("let t = [for i in 1..2 { [for j in 1..2 { i * j }] }]\nt[2]\n"),
        "2\n4\n"
    );
}

#[test]
fn break_and_continue_shape_the_passes_and_the_list() {
    assert_eq!(
        prints(
            "let evens = [for i in 1..10 {\n    if i > 6 { break }\n    if i == 3 { continue }\n    let d = i * 2\n    d\n}]\nevens\n"
        ),
        "2\n4\n8\n10\n12\n"
    );
    // In a plain loop `continue` skips the rest of the pass and `break` ends the loop.
    assert_eq!(
        prints(
            "for i in 1..5 {\n    if i == 2 { continue }\n    if i == 4 { break }\n    print(i)\n}\n"
        ),
        "1\n3\n"
    );
    // A jump inside a nested block reaches its loop.
    assert_eq!(
        prints(
            "for i in 1..5 {\n    if i > 1 {\n        if i > 3 { break }\n    }\n    print(i)\n}\n"
        ),
        "1\n2\n3\n"
    );
    // A `break` in an inner loop ends the inner loop only.
    assert_eq!(
        prints(
            "for i in 1..2 {\n    for j in 1..5 {\n        if j > 2 { break }\n        print(i * 10 + j)\n    }\n}\n"
        ),
        "11\n12\n21\n22\n"
    );
}

#[test]
fn the_loop_variable_and_the_body_s_lets_are_fresh_each_pass_and_gone_after() {
    assert_eq!(
        prints("for i in 1..3 {\n    let d = i * 2\n    print(d)\n}\n"),
        "2\n4\n6\n"
    );
    assert_eq!(error_code("for i in 1..2 { print(i) }\ni\n"), "Z0101");
    assert_eq!(error_code("for i in 1..2 { let d = i }\nd\n"), "Z0101");
    // A `let` with the loop variable's name is allowed, with a warning (W0302).
    let outcome = run("for i in 1..2 {\n    let i = i * 10\n    print(i)\n}\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "10\n20\n");
    assert!(
        outcome.output.stderr.starts_with("warning[W0302]"),
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn a_pass_that_errors_stops_the_script_at_that_pass() {
    let outcome = run("for i in 1..5 {\n    print(i)\n    print([1, 2][i])\n}\n");
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.output.stdout, "1\n1\n2\n2\n3\n");
    assert!(
        outcome.output.stderr.contains("error[Z0207]"),
        "{}",
        outcome.output.stderr
    );
}

// ----- for over variables (sections 5.2 and 6) -----

#[test]
fn a_loop_over_variables_stands_for_each_variable_in_turn() {
    let printed = prints(
        "let d = frame(x = [1, 2, 3], y = [10, 20, 30])\nfor v in vars(x, y) {\n    print(d.mean(v))\n}\nvars(x, y)\n",
    );
    assert_eq!(printed, "2\n20\nx, y\n");
    // Inside `add`, the loop variable is the column it names.
    assert_eq!(
        prints(
            "let d = frame(x = [1, 2], y = [10, 20])\n[for v in vars(x, y) { d.add(z = v * 2).mean(z) }]\n"
        ),
        "3\n30\n"
    );
}

#[test]
fn a_loop_over_variables_fits_the_formulas_as_written() {
    let looped = run(
        "let w = dataset(wages)\nlet fits = [for y in vars(wage, hours) { w.reg(log(y) ~ educ + y:female) }]\nfits[1]\nfits[2]\n",
    );
    let written = run(
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + wage:female)\nw.reg(log(hours) ~ educ + hours:female)\n",
    );
    assert_eq!(looped.exit_code, 0, "{}", looped.output.stderr);
    assert_eq!(written.exit_code, 0, "{}", written.output.stderr);
    assert_eq!(looped.output.stdout, written.output.stdout);
    assert!(
        looped
            .output
            .stdout
            .contains("reg · log(hours) ~ educ + hours:female"),
        "{}",
        looped.output.stdout
    );
}

#[test]
fn a_loop_variable_named_like_a_column_is_a_clash() {
    assert_eq!(
        error_code(
            "let d = frame(x = [1, 2], y = [3, 4])\nfor x in vars(y) { print(d.mean(x)) }\n"
        ),
        "Z0106"
    );
    assert_eq!(
        error_code(
            "let d = frame(x = [1, 2], y = [3, 4])\nfor y in vars(x) { print(d.add(z = y * 2).n_rows()) }\n"
        ),
        "Z0106"
    );
    // No clash outside a frame method, or with a name the frame lacks.
    assert_eq!(
        prints(
            "let d = frame(x = [1, 2])\nfor x in 1..2 { print(x) }\nfor v in vars(x) { print(d.mean(v)) }\n"
        ),
        "1\n2\n1.5\n"
    );
}

#[test]
fn what_a_loop_may_run_over() {
    assert_eq!(error_code("for i in 5 { print(i) }\n"), "Z0221");
    assert_eq!(
        error_code("let d = frame(x = [1])\nfor r in d { print(r) }\n"),
        "Z0221"
    );
    assert_eq!(error_code("for v in vars(x1..x3) { print(v) }\n"), "Z0221");
    assert_eq!(
        error_code("let xs = [for i in 1..3 { print(i) }]\n"),
        "Z0206"
    );
    assert_eq!(error_code("break\n"), "Z0313");
    assert_eq!(error_code("for print in 1..2 { 1 }\n"), "Z0105");
}

// ----- draws inside a loop (Engine and Project Specification, section 8.3, point 3) -----

#[test]
fn each_pass_draws_its_own_numbers_and_a_rerun_repeats_them() {
    let script = "seed(42)\nlet draws = [for s in 1..3 { dist.normal().draw(2) }]\ndraws[1]\ndraws[2]\ndraws[3]\n";
    let first = prints(script);
    let second = prints(script);
    assert_eq!(first, second);
    let columns: Vec<&str> = first.split("Column<Float> · 2 values\n").collect();
    assert_eq!(columns.len(), 4, "{first}");
    assert_ne!(columns[1], columns[2]);
    assert_ne!(columns[2], columns[3]);
    // A draw inside a loop is keyed by its pass, so it differs from the same draw outside one.
    let outside = prints("seed(42)\nlet d = dist.normal().draw(2)\nd\n");
    assert_ne!(
        columns[1],
        outside.trim_start_matches("Column<Float> · 2 values\n")
    );
}

#[test]
fn every_message_of_loops() {
    let cases = [
        ("for_over_a_number", "for i in 5 {\n    print(i)\n}\n"),
        (
            "for_over_a_frame",
            "let d = frame(x = [1])\nfor r in d {\n    print(r)\n}\n",
        ),
        (
            "for_over_a_range_of_variables",
            "for v in vars(x1..x3) {\n    print(v)\n}\n",
        ),
        (
            "list_for_without_a_value",
            "let xs = [for i in 1..3 {\n    print(i)\n}]\n",
        ),
        (
            "let_shadows_loop_variable",
            "for i in 1..2 {\n    let i = i * 10\n    print(i)\n}\n",
        ),
        (
            "loop_variable_after_the_loop",
            "for i in 1..2 {\n    print(i)\n}\ni\n",
        ),
        (
            "loop_variable_is_a_column",
            "let d = frame(x = [1, 2], y = [3, 4])\nfor x in vars(y) {\n    print(d.mean(x))\n}\n",
        ),
        (
            "loop_variable_is_a_column_in_add",
            "let d = frame(x = [1, 2], y = [3, 4])\nfor y in vars(x) {\n    print(d.add(z = y * 2).n_rows())\n}\n",
        ),
        ("break_outside_a_loop", "break\n"),
        ("continue_outside_a_loop", "continue\n"),
        (
            "loop_variable_is_a_core_name",
            "for print in 1..2 {\n    1\n}\n",
        ),
        (
            "error_in_a_pass",
            "for i in 1..5 {\n    print([1, 2][i])\n}\n",
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
fn the_loop_of_section_5_2_without_fixed_effects() {
    let outcome = run(
        "let wages = dataset(wages)\nlet fits = [for y in vars(wage, hours, union) {\n    wages.reg(y ~ educ + exper, cluster: worker)\n}]\nreport(fits)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "section_5_2",
        format!("{}{}", outcome.output.stdout, outcome.output.stderr)
    );
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// `[for i in a..b { i }]` is the range itself, and `break` at pass k leaves k − 1 items.
    #[test]
    fn a_list_for_over_a_range_is_the_range(a in -20i64..20, len in 0usize..12, k in 1usize..12) {
        let b = a + len as i64 - 1;
        let collected = run(&format!("[for i in ({a})..({b}) {{ i }}]\n"));
        let range = run(&format!("({a})..({b})\n"));
        prop_assert_eq!(collected.exit_code, 0, "{}", collected.output.stderr);
        prop_assert_eq!(&collected.output.stdout, &range.output.stdout);
        let cut = run(&format!("let k = {k}\n[for i in ({a})..({b}) {{\n    if i - {a} + 1 == k {{ break }}\n    i\n}}]\n"));
        prop_assert_eq!(cut.exit_code, 0, "{}", cut.output.stderr);
        // An empty list prints as `[]`.
        let items = if cut.output.stdout.trim() == "[]" { 0 } else { cut.output.stdout.lines().count() };
        prop_assert_eq!(items, len.min(k - 1));
    }
}
