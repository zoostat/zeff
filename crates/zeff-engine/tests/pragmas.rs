//! Deliberate exceptions and `--deny-warnings` (Language Reference, section 11.1; release
//! 0.7.0, step 2): `// zeff: allow(CODE) reason` silences one warning on one statement, at the
//! top of a script, in a loop, in a function and in a module, for the warnings of the run and of
//! the check, and every warning silenced is recorded with its reason; `--deny-warnings` makes
//! every warning that no pragma allows an error with exit code 3. The edge cases: two pragmas on
//! one statement, a pragma that silences nothing, a pragma on a continuation line, a code no
//! warning has; and the property that pragmas change the messages of a run and never what it
//! prints.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, check_in_project, run_script_with};

const SCRIPT: &str = "scripts/main.zeff";

/// A module whose function warns where a value is zero, with a pragma on the statement.
const MODULE: &str = "/// The logarithm of each value.\n/// x: The values.\n/// returns: Their logarithms.\n/// > quiet_log(1)\n/// 0\nfn quiet_log(x: Float) -> Float {\n    // zeff: allow(W0204) a zero is a worker in the first year\n    log(x)\n}\n";

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/logs.zeff", MODULE);
    // 6 November 2026, 12:00: the clock `today()` reads.
    fs.set_clock(1_794_052_800_000_000);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn run_with(script: &str, deny_warnings: bool) -> Outcome {
    let (fs, project) = project();
    let options = RunOptions {
        deny_warnings,
        ..RunOptions::default()
    };
    run_script_with(&fs, &project, SCRIPT, script, &options)
}

fn run(script: &str) -> Outcome {
    run_with(script, false)
}

fn deny(script: &str) -> Outcome {
    run_with(script, true)
}

/// How many warnings and notes a run printed.
fn messages(outcome: &Outcome) -> usize {
    outcome
        .output
        .stderr
        .lines()
        .filter(|l| l.starts_with("warning") || l.starts_with("note"))
        .count()
}

/// A run with no message at all: what it printed.
fn quiet(script: &str) -> Outcome {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "", "{script}: no message expected");
    outcome
}

/// A frame with a zero and a negative value: `log(x)` warns of both (W0204 and W0205).
const FRAME: &str = "let d = frame(x = [0, -1, 2])\n";

// ----- one warning on one statement -----

#[test]
fn the_example_of_section_11_1_is_silenced_and_recorded() {
    let bare = run("let age = to(\"varies\", Float)\nage\n");
    assert_eq!(bare.exit_code, 0);
    assert_eq!(messages(&bare), 1, "{}", bare.output.stderr);
    assert!(bare.output.allowed.is_empty());

    let outcome = quiet(
        "// zeff: allow(W0203) \"varies\" coded as NA by design\nlet age = to(\"varies\", Float)\nage\n",
    );
    assert_eq!(outcome.output.stdout, bare.output.stdout);
    assert_eq!(outcome.output.allowed.len(), 1);
    let allowed = &outcome.output.allowed[0];
    assert_eq!(allowed.code, "W0203");
    assert_eq!(allowed.file, SCRIPT);
    assert_eq!(allowed.line, 2);
    assert_eq!(allowed.reason, "\"varies\" coded as NA by design");
    assert!(
        bare.output.stderr.contains(&allowed.message),
        "the record keeps the warning as it would have printed: {}",
        allowed.message
    );
}

#[test]
fn a_pragma_silences_its_code_only() {
    let both = run(&format!("{FRAME}let z = d.add(l = log(x))\n"));
    assert_eq!(messages(&both), 2, "{}", both.output.stderr);
    let outcome = run(&format!(
        "{FRAME}// zeff: allow(W0204) a zero is real\nlet z = d.add(l = log(x))\n"
    ));
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(messages(&outcome), 1, "{}", outcome.output.stderr);
    assert!(outcome.output.stderr.contains("negative"));
    assert_eq!(outcome.output.allowed.len(), 1);
}

#[test]
fn a_pragma_silences_its_statement_only() {
    let outcome = run(&format!(
        "{FRAME}let a = d.add(l = log(x)) // zeff: allow(W0204) a zero is real\nlet b = d.add(l = log(x))\n"
    ));
    // Line 2 keeps its W0205; line 3 keeps both.
    assert_eq!(messages(&outcome), 3, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stderr.matches("(line 3)").count(),
        2,
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn two_pragmas_on_one_statement_silence_two_warnings() {
    let outcome = quiet(&format!(
        "{FRAME}// zeff: allow(W0204) a zero is real\n// zeff: allow(W0205) and so is a loss\nlet z = d.add(l = log(x))\nz.n_rows()\n"
    ));
    assert_eq!(outcome.output.stdout, "3\n");
    let codes: Vec<&str> = outcome
        .output
        .allowed
        .iter()
        .map(|a| a.code.as_str())
        .collect();
    assert_eq!(codes.len(), 2);
    assert!(codes.contains(&"W0204") && codes.contains(&"W0205"));
}

#[test]
fn a_pragma_covers_the_continuation_lines() {
    quiet(&format!(
        "{FRAME}// zeff: allow(W0204) a zero is real\n// zeff: allow(W0205) and so is a loss\nlet z = d\n    .where(x < 5)\n    .add(l = log(x))\n"
    ));
    quiet(&format!(
        "{FRAME}// zeff: allow(W0205) and so is a loss\nlet z = d // zeff: allow(W0204) a zero is real\n    .where(x < 5)\n    .add(l = log(x))\n"
    ));
}

#[test]
fn a_pragma_that_silences_nothing_is_passed_over() {
    // A warning of the run depends on the data, so a pragma without one is no mistake.
    let outcome = quiet("// zeff: allow(W0204) in case a zero arrives\nlet a = log(2)\n");
    assert!(outcome.output.allowed.is_empty());
}

#[test]
fn a_note_with_a_warning_code_is_silenced() {
    let bare = run("let t = today()\n");
    assert_eq!(messages(&bare), 1, "{}", bare.output.stderr);
    assert!(bare.output.stderr.starts_with("note"));
    let outcome = quiet("let t = today() // zeff: allow(W0703) only for the title of a table\n");
    assert_eq!(outcome.output.allowed[0].code, "W0703");
}

// ----- at every depth -----

#[test]
fn a_pragma_in_a_loop_silences_each_pass() {
    let outcome = run(&format!(
        "{FRAME}for i in [1, 2, 3] {{\n    // zeff: allow(W0204) a zero is real\n    let z = d.add(l = log(x))\n}}\n"
    ));
    // Each pass keeps its W0205 and records its W0204.
    assert_eq!(messages(&outcome), 3, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.allowed.len(), 3);
    assert!(outcome.output.allowed.iter().all(|a| a.line == 4));
}

#[test]
fn a_pragma_on_a_loop_covers_its_body() {
    quiet(&format!(
        "{FRAME}// zeff: allow(W0205) a loss is real\nfor i in [1, 2] {{ // zeff: allow(W0204) a zero is real\n    let z = d.add(l = log(x))\n}}\n"
    ));
}

#[test]
fn a_pragma_in_a_function_silences_every_call() {
    let function = "/// Logs.\nfn logs(f: Frame) -> Frame {\n    // zeff: allow(W0204) a zero is real\n    // zeff: allow(W0205) and so is a loss\n    f.add(l = log(x))\n}\n";
    let outcome = quiet(&format!(
        "{FRAME}{function}let a = logs(d)\nlet b = d.logs()\n"
    ));
    assert_eq!(outcome.output.allowed.len(), 4);
    assert!(outcome.output.allowed.iter().all(|a| a.line == 6));
}

#[test]
fn a_pragma_above_a_function_covers_its_body() {
    let function = "// zeff: allow(W0204) a zero is real\n// zeff: allow(W0205) and so is a loss\n/// Logs.\nfn logs(f: Frame) -> Frame {\n    let g = f.where(x < 5)\n    g.add(l = log(x))\n}\n";
    quiet(&format!("{FRAME}{function}let a = logs(d)\n"));
}

#[test]
fn a_pragma_on_a_statement_covers_what_it_calls() {
    let function = "/// Logs.\nfn logs(f: Frame) -> Frame {\n    f.add(l = log(x))\n}\n";
    let bare = run(&format!("{FRAME}{function}let a = logs(d)\n"));
    assert_eq!(messages(&bare), 2);
    quiet(&format!(
        "{FRAME}{function}// zeff: allow(W0204) a zero is real\n// zeff: allow(W0205) and so is a loss\nlet a = logs(d)\n"
    ));
}

#[test]
fn a_pragma_in_a_module_silences_the_warning_there() {
    let outcome = quiet("use functions::logs\nlet a = quiet_log(0)\na\n");
    assert_eq!(outcome.output.stdout, "-Inf\n");
    assert_eq!(outcome.output.allowed.len(), 1);
    assert_eq!(outcome.output.allowed[0].file, "functions/logs.zeff");
    assert_eq!(outcome.output.allowed[0].line, 8);
    assert_eq!(
        outcome.output.allowed[0].reason,
        "a zero is a worker in the first year"
    );
    // The pragma is the module's: the same warning raised by the script is not covered.
    let outcome = run("use functions::logs\nlet a = quiet_log(0)\nlet b = log(0)\n");
    assert_eq!(messages(&outcome), 1, "{}", outcome.output.stderr);
}

// ----- the warnings of the check -----

#[test]
fn a_warning_of_the_check_is_silenced_and_recorded() {
    let (fs, project) = project();
    let registry = zeff_engine::registry::registry().unwrap();
    let bare = "fn double(x: Float) -> Float { x * 2 }\ndouble(2)\n";
    let checked = check_in_project(&fs, &project, SCRIPT, bare, registry);
    let codes: Vec<&str> = checked
        .checked
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["W0303"]);
    assert!(checked.allowed.is_empty());

    let allowed = format!("// zeff: allow(W0303) a helper of one line\n{bare}");
    let checked = check_in_project(&fs, &project, SCRIPT, &allowed, registry);
    assert!(checked.checked.diagnostics.is_empty());
    assert_eq!(checked.allowed.len(), 1);
    assert_eq!(checked.allowed[0].code, "W0303");
    assert_eq!(checked.allowed[0].line, 2);
    assert_eq!(checked.allowed[0].reason, "a helper of one line");

    // The run carries the record of the check with its own.
    let outcome = quiet(&allowed);
    assert_eq!(outcome.output.stdout, "4\n");
    assert_eq!(outcome.output.allowed.len(), 1);

    // Another code does not cover it.
    let other = format!("// zeff: allow(W0204) not this one\n{bare}");
    let checked = check_in_project(&fs, &project, SCRIPT, &other, registry);
    assert_eq!(checked.checked.diagnostics.len(), 1);
}

// ----- Z0317 -----

#[test]
fn a_wrong_pragma_stops_the_run() {
    for script in [
        "// zeff: allow(W0204)\nlet a = log(0)\n",
        "// zeff: allow(W9999) no such warning\nlet a = log(0)\n",
        "let a = log(\n    0) // zeff: allow(W0204) on a continuation line\n",
    ] {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{script}");
        assert!(outcome.output.stderr.contains("error[Z0317]"), "{script}");
        assert_eq!(outcome.output.stdout, "");
    }
}

#[test]
fn a_code_no_warning_has_is_refused_in_a_module_too() {
    let (fs, project) = project();
    fs.add_file(
        "/p/functions/more.zeff",
        "/// One.\n/// returns: One.\n/// > one()\n/// 1\nfn one() -> Int {\n    // zeff: allow(W0299) no such warning\n    1\n}\n",
    );
    let registry = zeff_engine::registry::registry().unwrap();
    let checked = check_in_project(&fs, &project, SCRIPT, "let a = 1\n", registry);
    let found: Vec<(&str, Option<&str>)> = checked
        .checked
        .diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.file.as_deref()))
        .collect();
    assert_eq!(found, [("Z0317", Some("functions/more.zeff"))]);
}

// ----- `--deny-warnings` -----

#[test]
fn deny_warnings_changes_nothing_in_a_run_without_warnings() {
    let script = "let a = log(2)\na\nprint(\"done\")\n";
    assert_eq!(deny(script), run(script));
    assert_eq!(deny(script).exit_code, 0);
}

#[test]
fn deny_warnings_stops_at_the_first_warning_with_exit_code_3() {
    let script = "print(\"before\")\nlet a = log(0)\nprint(\"after\")\n";
    assert_eq!(run(script).output.stdout, "before\nafter\n");
    let outcome = deny(script);
    assert_eq!(outcome.exit_code, 3);
    assert_eq!(outcome.output.stdout, "before\n");
    assert!(outcome.output.stderr.starts_with("error[W0204]: "));
    assert!(!outcome.output.stderr.contains("warning: "));
}

#[test]
fn deny_warnings_lets_an_allowed_warning_pass() {
    let script = "// zeff: allow(W0204) a zero is real\nlet a = log(0)\nprint(\"after\")\n";
    let outcome = deny(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "");
    assert_eq!(outcome.output.stdout, "after\n");
    assert_eq!(outcome.output.allowed.len(), 1);
    // The other warning of the statement is still refused.
    let outcome = deny(&format!(
        "{FRAME}// zeff: allow(W0204) a zero is real\nlet z = d.add(l = log(x))\n"
    ));
    assert_eq!(outcome.exit_code, 3);
    assert!(outcome.output.stderr.starts_with("error[W0205]: "));
}

#[test]
fn deny_warnings_refuses_the_clock_and_a_warning_of_the_script() {
    let outcome = deny("let t = today()\n");
    assert_eq!(outcome.exit_code, 3);
    assert!(outcome.output.stderr.starts_with("error[W0703]: "));
    // A warning raised by `warn` has no code and cannot be allowed.
    let outcome = deny("warn(\"check the units\")\nprint(1)\n");
    assert_eq!(outcome.exit_code, 3);
    assert_eq!(outcome.output.stdout, "");
}

#[test]
fn deny_warnings_refuses_a_warning_of_the_check_before_the_run() {
    let script = "print(\"before\")\nfn double(x: Float) -> Float { x * 2 }\ndouble(2)\n";
    assert_eq!(run(script).exit_code, 0);
    let outcome = deny(script);
    assert_eq!(outcome.exit_code, 3);
    assert_eq!(outcome.output.stdout, "", "the run does not start");
    assert!(outcome.output.stderr.starts_with("error[W0303]: "));
    // Allowed, the script runs.
    let outcome = deny(
        "print(\"before\")\n// zeff: allow(W0303) a helper\nfn double(x: Float) -> Float { x * 2 }\ndouble(2)\n",
    );
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.output.stdout, "before\n4\n");
}

#[test]
fn an_error_of_the_script_keeps_its_exit_code_under_deny_warnings() {
    // The statement warns and then fails: its own error stops the run.
    let outcome = deny("let a = log(0) + nothing_here\n");
    assert_eq!(outcome.exit_code, 1);
    let outcome = deny(&format!(
        "{FRAME}let z = d.add(l = log(x)).where(missing_column > 1)\n"
    ));
    assert_eq!(outcome.exit_code, 1, "{}", outcome.output.stderr);
}

#[test]
fn every_message_of_pragmas_and_deny_warnings() {
    let cases: &[(&str, &str, bool, i32)] = &[
        (
            "unknown_code",
            "// zeff: allow(W9999) no such warning\nlet a = log(0)\n",
            false,
            1,
        ),
        (
            "denied_at_run_time",
            "let d = frame(x = [0, -1, 2])\nlet z = d.add(l = log(x))\n",
            true,
            3,
        ),
        (
            "denied_at_check_time",
            "fn double(x: Float) -> Float { x * 2 }\ndouble(2)\n",
            true,
            3,
        ),
        (
            "denied_without_a_code",
            "warn(\"check the units\")\n",
            true,
            3,
        ),
    ];
    for (name, script, deny_warnings, exit_code) in cases {
        let outcome = run_with(script, *deny_warnings);
        assert_eq!(outcome.exit_code, *exit_code, "{name}");
        insta::assert_snapshot!(*name, outcome.output.stderr);
    }
}

// ----- property -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Pragmas change the messages of a run and never what it prints: with a pragma on any
    /// subset of the statements that warn, the output is the same, the warnings left are those
    /// of the other statements, and each warning silenced is recorded once.
    #[test]
    fn pragmas_change_the_messages_and_never_the_output(
        allowed in proptest::collection::vec(any::<bool>(), 1..7),
        trailing in proptest::collection::vec(any::<bool>(), 7),
    ) {
        let mut bare = String::new();
        let mut with_pragmas = String::new();
        for (i, allow) in allowed.iter().enumerate() {
            let statement = format!("let a{i} = log({i} - {i})\nprint(a{i})\n");
            bare.push_str(&statement);
            match (allow, trailing[i]) {
                (false, _) => with_pragmas.push_str(&statement),
                (true, false) => {
                    with_pragmas.push_str("// zeff: allow(W0204) a zero is real\n");
                    with_pragmas.push_str(&statement);
                }
                (true, true) => with_pragmas.push_str(&format!(
                    "let a{i} = log({i} - {i}) // zeff: allow(W0204) a zero is real\nprint(a{i})\n"
                )),
            }
        }
        let plain = run(&bare);
        let outcome = run(&with_pragmas);
        let silenced = allowed.iter().filter(|a| **a).count();
        prop_assert_eq!(plain.exit_code, 0);
        prop_assert_eq!(outcome.exit_code, 0);
        prop_assert_eq!(&outcome.output.stdout, &plain.output.stdout);
        prop_assert_eq!(messages(&plain), allowed.len());
        prop_assert_eq!(messages(&outcome), allowed.len() - silenced);
        prop_assert_eq!(outcome.output.allowed.len(), silenced);
        // Under `--deny-warnings` the run passes exactly when every warning is allowed.
        let denied = deny(&with_pragmas);
        prop_assert_eq!(denied.exit_code, if silenced == allowed.len() { 0 } else { 3 });
    }
}
