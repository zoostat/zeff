//! The `Failed` type and `attempt` (Language Reference, sections 3.7, 4, 5.4 and 5.6; release
//! 0.5.0, step 5): a failure kept as a value, where it may stand and where it may not, the
//! guard and its warning, `report` skipping failed fits with a count, every message, and the
//! property that `attempt(e)` is `e` whenever `e` succeeds.

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

/// A small frame whose regression fails for a cut that keeps no row.
const D: &str = "let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])\n";

#[test]
fn attempt_gives_the_value_when_the_expression_succeeds() {
    assert_eq!(prints("attempt(1 + 1)\n"), "2\n");
    assert_eq!(prints("let v = attempt(\"a\")\nv\n"), "a\n");
    assert_eq!(
        prints(&format!("{D}attempt(d.reg(y ~ x)).coef(x)\n")),
        "2.2\n"
    );
    // The value keeps its type: `attempt(e)` has the type of `e`.
    assert_eq!(prints("let k = attempt(2) * 3\nk\n"), "6\n");
}

#[test]
fn a_failure_becomes_a_failed_value_that_prints_its_cause() {
    let outcome = run(&format!(
        "{D}let m = attempt(d.where(x > 10).reg(y ~ x))\nm\nm.is(Failed)\ntype_of(m)\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "failed · Z0801 at line 2: no row remains to fit the model: the frame has no rows\ntrue\nFailed\n"
    );
    // The warnings and notes raised before the failure stay.
    assert!(
        outcome
            .output
            .stderr
            .contains("note: no row kept: the result is empty (line 2)"),
        "{}",
        outcome.output.stderr
    );
    // Inside a loop, each pass keeps its own outcome.
    let looped = run(&format!(
        "{D}let fits = [for cut in [0, 10] {{\n    attempt(d.where(x > cut).reg(y ~ x))\n}}]\nfits[2].is(Failed)\nfits[1].is(Failed)\nfits[2]\n"
    ));
    assert_eq!(looped.exit_code, 0, "{}", looped.output.stderr);
    assert_eq!(
        looped.output.stdout,
        "true\nfalse\nfailed · Z0801 at line 3: no row remains to fit the model: the frame has no rows\n"
    );
}

#[test]
fn an_internal_error_is_never_caught() {
    // `attempt` of an expression the checker refuses is still refused; a run-time bug (Z9xxx)
    // is not a script failure and stops the run.
    assert_eq!(error_code("attempt(1 + \"a\")\n"), "Z0215");
    let outcome = run("attempt(print(1))\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome.output.stderr.contains("error[Z0206]"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(error_code("attempt()\n"), "Z0406");
    assert_eq!(error_code("attempt(1, 2)\n"), "Z0407");
    assert_eq!(error_code("attempt(expr: 1)\n"), "Z0407");
}

#[test]
fn a_failed_value_may_stand_in_a_list_and_nowhere_else() {
    // A failed fit (its type is a RegFit, so arithmetic on it is the checker's Z0221) and a
    // failed Int, whose misuse is found when the statement runs.
    let fit = format!("{D}let m = attempt(d.where(x > 10).reg(y ~ x))\n");
    let int = "let k = attempt([1, 2][5])\n";
    // In a list: fine, and the list's type ignores it.
    assert_eq!(
        prints(&format!(
            "{int}let xs = [k, k, 3]\ntype_of(xs)\nxs[1].is(Failed)\n"
        )),
        "List<Int>\ntrue\n"
    );
    assert_eq!(error_code(&format!("{fit}m.coef(x)\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}k + 1\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}print(-k)\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}let r = {{a: k}}\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}let g = [1 => k]\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}frame(a = [k])\n")), "Z0220");
    assert_eq!(error_code(&format!("{int}abs(k)\n")), "Z0220");
    assert_eq!(
        error_code("let b = attempt([true][2])\nif b { 1 }\n"),
        "Z0203"
    );
    // The functions that look at a failed value take it.
    assert_eq!(
        prints(&format!("{int}print(k)\nk.is(Failed)\ntype_of(k)\n")),
        "failed · Z0207 at line 1: index 5 is outside 1..2\ntrue\nFailed\n"
    );
}

#[test]
fn the_guard_warning_and_the_narrowing_of_section_3_7() {
    let fit = format!("{D}let m = attempt(d.reg(y ~ x))\n");
    // Unguarded: W0209, and the script runs when the fit succeeded.
    let outcome = run(&format!("{fit}m.coef(x)\n"));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stderr.starts_with("warning[W0209]"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(outcome.output.stdout, "2.2\n");
    // Guarded three ways: no warning.
    for guarded in [
        format!("{fit}if !m.is(Failed) {{ print(m.coef(x)) }}\n"),
        format!("{fit}if m.is(Failed) {{ error(\"the fit failed\") }}\nm.coef(x)\n"),
        format!(
            "{fit}let slopes = [for f in [m] {{\n    if f.is(Failed) {{ continue }}\n    f.coef(x)\n}}]\nslopes\n"
        ),
    ] {
        assert_eq!(prints(&guarded), "2.2\n", "{guarded}");
    }
    // The narrowing ends with the block: after the loop the binding is unguarded again.
    let after = run(&format!(
        "{fit}for i in 1..1 {{\n    if m.is(Failed) {{ break }}\n    print(m.coef(x))\n}}\nm.coef(x)\n"
    ));
    assert_eq!(after.exit_code, 0, "{}", after.output.stderr);
    assert_eq!(
        after.output.stderr.matches("warning[W0209]").count(),
        1,
        "{}",
        after.output.stderr
    );
    // `is`, `type_of` and `print` never warn.
    assert_eq!(prints(&format!("{fit}m.is(Failed)\n")), "false\n");
}

#[test]
fn report_skips_failed_fits_with_a_count() {
    let script = format!(
        "{D}let fits = [for cut in [0, 2, 10, 20] {{\n    attempt(d.where(x > cut).reg(y ~ x))\n}}]\nreport(fits)\n"
    );
    let outcome = run(&script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stdout.contains("(1)         (2)"),
        "{}",
        outcome.output.stdout
    );
    assert!(
        !outcome.output.stdout.contains("(3)"),
        "{}",
        outcome.output.stdout
    );
    assert!(
        outcome
            .output
            .stderr
            .contains("warning: 2 of 4 fits failed (Z0801 at line 3) and were skipped (line 5)"),
        "{}",
        outcome.output.stderr
    );
    // Every fit failed: an error that names the failures.
    assert_eq!(
        error_code(&format!(
            "{D}report([for cut in [10, 20] {{ attempt(d.where(x > cut).reg(y ~ x)) }}])\n"
        )),
        "Z0406"
    );
}

#[test]
fn every_message_of_attempt_and_failed_values() {
    let failed = format!("{D}let m = attempt(d.where(x > 10).reg(y ~ x))\n");
    let cases = [
        ("failed_as_argument", format!("{failed}m.coef(x)\n")),
        (
            "failed_in_arithmetic",
            "let k = attempt([1, 2][5])\nk + 1\n".to_string(),
        ),
        (
            "failed_in_a_record",
            "let k = attempt([1, 2][5])\nlet r = {a: k}\n".to_string(),
        ),
        (
            "failed_in_a_mapping",
            "let k = attempt([1, 2][5])\nlet g = [1 => k]\n".to_string(),
        ),
        (
            "failed_in_a_list_argument",
            "let k = attempt([1, 2][5])\nframe(a = [k])\n".to_string(),
        ),
        (
            "failed_as_condition",
            "let b = attempt([true][2])\nif b { 1 }\n".to_string(),
        ),
        ("attempt_without_expression", "attempt()\n".to_string()),
        ("attempt_with_two", "attempt(1, 2)\n".to_string()),
        ("attempt_of_nothing", "attempt(print(1))\n".to_string()),
        (
            "unguarded_method",
            format!("{D}let m = attempt(d.reg(y ~ x))\nm.coef(x)\n"),
        ),
        (
            "report_all_failed",
            format!(
                "{D}report([for cut in [10, 20] {{ attempt(d.where(x > cut).reg(y ~ x)) }}])\n"
            ),
        ),
        (
            "report_some_failed",
            format!("{D}report([for cut in [0, 10] {{ attempt(d.where(x > cut).reg(y ~ x)) }}])\n"),
        ),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(&script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}

#[test]
fn the_loop_of_section_5_4_with_reg() {
    let script = "let w = dataset(wages)\nlet fits = [for cut in [10, 20, 30, 1000] {\n    attempt(w.where(wage > cut).reg(log(wage) ~ educ + female))\n}]\nreport(fits)\nlet slopes = [for m in fits {\n    if m.is(Failed) { continue }\n    m.coef(educ)\n}]\nslopes\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "section_5_4",
        format!("{}{}", outcome.output.stdout, outcome.output.stderr)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// `attempt(e)` prints exactly what `e` prints when `e` succeeds, and the count W0801 reports
    /// is the number of failed items.
    #[test]
    fn attempt_is_transparent_and_report_counts_the_failures(
        a in -1000i64..1000,
        b in -1000i64..1000,
        cuts in prop::collection::vec(0i64..12, 1..6),
    ) {
        let plain = run(&format!("let a = {a}\nlet b = {b}\na * b - a\n"));
        let tried = run(&format!("let a = {a}\nlet b = {b}\nattempt(a * b - a)\n"));
        prop_assert_eq!(&plain.output.stdout, &tried.output.stdout);
        prop_assert_eq!(&plain.output.stderr, &tried.output.stderr);
        let list: Vec<String> = cuts.iter().map(|c| c.to_string()).collect();
        let script = format!(
            "{D}let fits = [for cut in [{}] {{\n    attempt(d.where(x > cut).reg(y ~ x))\n}}]\nreport(fits)\n",
            list.join(", ")
        );
        let outcome = run(&script);
        // A cut of 3 keeps two rows, which the regression refuses (no residual degrees of
        // freedom); 4 and above keep one or none.
        let failing = cuts.iter().filter(|c| **c >= 3).count();
        if failing == cuts.len() {
            prop_assert_eq!(outcome.exit_code, 1);
            prop_assert!(outcome.output.stderr.contains("error[Z0406]"), "{}", outcome.output.stderr);
        } else {
            prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
            if failing == 0 {
                prop_assert!(!outcome.output.stderr.contains("fits failed"), "{}", outcome.output.stderr);
            } else {
                let expected = format!("warning: {failing} of {} fits failed", cuts.len());
                prop_assert!(outcome.output.stderr.contains(&expected), "{}", outcome.output.stderr);
            }
        }
    }
}
