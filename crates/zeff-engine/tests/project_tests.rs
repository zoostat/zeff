//! The tests of a project (release 0.7.0, step 32a): the `assert` checks of its scripts and
//! the examples of the functions of `functions/`, run by the engine as `zeff test` will.

use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::RunOptions;
use zeff_engine::testing::{ProjectTests, calls_assert, test_project};

/// A project in memory with the files given, by their paths from its root.
fn project(files: &[(&str, &str)]) -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    for (name, text) in files {
        fs.add_file(format!("/p/{name}"), text);
    }
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    (fs, project)
}

fn tested(files: &[(&str, &str)]) -> ProjectTests {
    let (fs, project) = project(files);
    test_project(&fs, &project, &RunOptions::default())
}

/// The tests of a project as a text, for a snapshot.
fn shown(tests: &ProjectTests) -> String {
    let mut out = String::new();
    for script in &tests.scripts {
        out.push_str(&format!(
            "script {}: {} checks, {} failed, exit code {}\n",
            script.file,
            script.checks,
            script.failed.len(),
            script.exit_code
        ));
        for failure in &script.failed {
            out.push_str(&format!("-- a check failed:\n{failure}"));
        }
        if let Some(stopped) = &script.stopped {
            out.push_str(&format!("-- stopped:\n{stopped}"));
        }
    }
    for example in &tests.examples {
        out.push_str(&format!(
            "example {} of {} ({}, line {}): {}\n",
            example.example,
            example.function,
            example.file,
            example.line,
            if example.passed() { "passed" } else { "failed" }
        ));
        if !example.passed() {
            for line in &example.code {
                out.push_str(&format!("    > {line}\n"));
            }
            out.push_str("  the page says:\n");
            for line in &example.expected {
                out.push_str(&format!("    {line}\n"));
            }
            out.push_str("  it printed:\n");
            for line in &example.found {
                out.push_str(&format!("    {line}\n"));
            }
        }
    }
    for file in &tests.without_checks {
        out.push_str(&format!("no checks: {file}\n"));
    }
    for problem in &tests.problems {
        out.push_str(&format!("problem: {problem}\n"));
    }
    let (all, failed) = tests.counts();
    out.push_str(&format!(
        "{all} tests, {failed} failed, exit code {}\n",
        tests.exit_code()
    ));
    out
}

const CLEAN: &str = "/// Share of the values of a column that are missing.
///
/// The count of `NA` over the count of rows.
///
/// x: The column.
/// returns: The share, a Float between 0 and 1.
/// > let d = frame(x = [1.5, NA, 3, NA])
/// > share_missing(d[\"x\"])
/// 0.5
/// > share_missing(frame(x = [1.5, NA, 3])[\"x\"])
/// 0.333333
fn share_missing(x: Column<Float>) -> Float {
    frame(value = x).where(is_missing(value)).n_rows() / frame(value = x).n_rows()
}

/// Twice a number.
///
/// The number doubled.
///
/// x: The number.
/// returns: The number times two, a Float.
/// > twice(2.5)
/// 5
/// > twice(4) + 1
/// 10
fn twice(x: Float) -> Float {
    x * 2
}
";

#[test]
fn every_check_of_a_script_is_evaluated_and_a_failed_one_does_not_stop_it() {
    let tests = tested(&[(
        "scripts/main.zeff",
        "let d = frame(x = [1, 2, 3])\nassert(d.n_rows() == 3, \"three rows\")\nassert(d.n_rows() > 5, \"more than five rows\")\nprint(\"after\")\nassert(d.mean(x) == 2, \"the mean is 2\")\nassert(d.max(x) == 4, \"the maximum is 4\")\n",
    )]);
    assert_eq!(tests.scripts.len(), 1);
    let script = &tests.scripts[0];
    assert_eq!((script.checks, script.failed.len()), (4, 2));
    assert_eq!(script.exit_code, 0);
    assert!(script.stopped.is_none());
    assert!(!script.passed());
    assert_eq!(tests.counts(), (4, 2));
    assert_eq!(tests.exit_code(), 1);
    insta::assert_snapshot!("two_failed_checks", shown(&tests));
}

#[test]
fn a_script_whose_checks_hold_passes() {
    let tests = tested(&[(
        "scripts/main.zeff",
        "let d = frame(x = [1, 2, 3])\nassert(d.n_rows() == 3, \"three rows\")\nfor i in 1..3 {\n    assert(i > 0, \"positive\")\n}\n",
    )]);
    let script = &tests.scripts[0];
    assert!(script.passed());
    // One check before the loop and one each time round it.
    assert_eq!(script.checks, 4);
    assert_eq!(tests.counts(), (4, 0));
    assert_eq!(tests.exit_code(), 0);
}

#[test]
fn checks_inside_functions_are_counted_where_they_are_reached() {
    let module = "/// A positive number, checked.\n///\n/// The number itself, after a check that it is positive.\n///\n/// x: The number.\n/// returns: The number, a Float.\n/// > positive(2.0)\n/// 2\nfn positive(x: Float) -> Float {\n    assert(x > 0, \"a positive number is needed\")\n    x\n}\n";
    let tests = tested(&[
        ("functions/checks.zeff", module),
        (
            "scripts/main.zeff",
            "use functions::checks\n/// A number below ten, checked.\nfn below_ten(x: Float) -> Float {\n    assert(x < 10, \"below ten\")\n    x\n}\nlet a = positive(below_ten(3.0))\nlet b = positive(below_ten(-1.0))\nlet c = positive(below_ten(20.0))\n",
        ),
    ]);
    let script = &tests.scripts[0];
    assert_eq!(
        (script.checks, script.failed.len()),
        (6, 2),
        "{}",
        shown(&tests)
    );
    assert!(script.stopped.is_none());
    // The failure inside the module names the module's file and line.
    assert!(
        script.failed[0].contains("error[Z0001]: a positive number is needed")
            && script.failed[0].contains("functions/checks.zeff:10"),
        "{}",
        script.failed[0]
    );
    assert!(
        script.failed[1].contains("below ten"),
        "{}",
        script.failed[1]
    );
    // The example of the module ran too, and passed.
    assert_eq!(tests.examples.len(), 1);
    assert!(tests.examples[0].passed(), "{}", shown(&tests));
}

#[test]
fn a_script_that_stops_on_an_error_fails_with_the_error() {
    let tests = tested(&[(
        "scripts/main.zeff",
        "let d = frame(x = [1, 2, 3])\nassert(d.n_rows() == 3, \"three rows\")\nerror(\"stopped on purpose\")\nassert(d.n_rows() == 4, \"never reached\")\n",
    )]);
    let script = &tests.scripts[0];
    assert_eq!((script.checks, script.failed.len()), (1, 0));
    assert_eq!(script.exit_code, 1);
    assert!(
        script
            .stopped
            .as_deref()
            .is_some_and(|s| s.starts_with("error[Z0001]: stopped on purpose")),
        "{:?}",
        script.stopped
    );
    // The checks that held, and the script that stopped.
    assert_eq!(tests.counts(), (2, 1));
    assert_eq!(tests.exit_code(), 1);
    // An error of the check, before anything runs: a condition that is no Bool.
    let tests = tested(&[(
        "scripts/main.zeff",
        "let d = frame(x = [1, 2, 3])\nassert(d[\"x\"] > 0, \"a column is no condition\")\n",
    )]);
    let script = &tests.scripts[0];
    assert_eq!(script.checks, 0);
    assert!(
        script
            .stopped
            .as_deref()
            .is_some_and(|s| s.starts_with("error[Z0203]")),
        "{:?}",
        script.stopped
    );
    assert_eq!(tests.exit_code(), 1);
}

#[test]
fn a_script_without_a_check_is_not_run() {
    let tests = tested(&[
        (
            "scripts/analysis.zeff",
            "// assert(1 == 2, \"in a comment\")\nprint(\"assert(1 == 2) in text\")\nerror(\"this script is never run by the tests\")\n",
        ),
        (
            "scripts/old/first.zeff",
            "assert(1 + 1 == 2, \"arithmetic\")\n",
        ),
    ]);
    assert_eq!(tests.without_checks, ["scripts/analysis.zeff"]);
    assert_eq!(tests.scripts.len(), 1);
    assert_eq!(tests.scripts[0].file, "scripts/old/first.zeff");
    assert!(tests.scripts[0].passed());
    assert_eq!(tests.exit_code(), 0);
    assert!(calls_assert("s.zeff", "assert (x, \"spaced\")\n"));
    assert!(calls_assert(
        "s.zeff",
        "for i in 1..2 { assert(i > 0, \"m\") }\n"
    ));
    assert!(!calls_assert(
        "s.zeff",
        "let assert_all = 1\nprint(\"assert(\")\n// assert(x)\n"
    ));
    assert!(!calls_assert(
        "s.zeff",
        "/// assert(x, \"in a page\")\nfn f(x: Float) -> Float { x }\n"
    ));
}

#[test]
fn the_examples_of_a_module_are_run_and_compared_with_their_pages() {
    let tests = tested(&[("functions/clean.zeff", CLEAN)]);
    assert!(tests.scripts.is_empty());
    assert_eq!(tests.examples.len(), 4);
    // The first example of `share_missing` has two statements; the last of `twice` prints
    // 9, and its page says 10.
    let lines: Vec<(&str, usize, u32, bool)> = tests
        .examples
        .iter()
        .map(|e| (e.function.as_str(), e.example, e.line, e.passed()))
        .collect();
    assert_eq!(
        lines,
        [
            ("share_missing", 1, 7, true),
            ("share_missing", 2, 10, true),
            ("twice", 1, 22, true),
            ("twice", 2, 24, false),
        ],
        "{}",
        shown(&tests)
    );
    assert_eq!(tests.examples[0].code.len(), 2);
    assert_eq!(tests.examples[3].expected, ["10"]);
    assert_eq!(tests.examples[3].found, ["9"]);
    assert_eq!(tests.counts(), (4, 1));
    assert_eq!(tests.exit_code(), 1);
    insta::assert_snapshot!("an_example_that_prints_something_else", shown(&tests));
}

#[test]
fn an_example_that_does_not_run_or_warns_is_reported_with_what_it_said() {
    let page = "/// The mean of a column, by a frame.\n///\n/// The mean of the values present.\n///\n/// x: The column.\n/// returns: The mean, a Float.\n";
    let body = "fn mean_of(x: Column<Float>) -> Float {\n    frame(value = x).mean(value)\n}\n";
    let hidden = "\n/// Private to this file.\n///\n/// The number itself.\n///\n/// x: The number.\n/// returns: The number, a Float.\n/// > _hidden(1.0)\n/// 2\nfn _hidden(x: Float) -> Float {\n    x\n}\n";
    // A warning is part of what an example prints; an error at run time is what the example
    // that raised it printed, and the examples after it print nothing.
    let module = format!(
        "{page}/// > mean_of(frame(x = [1, NA, 3])[\"x\"])\n/// 2\n/// warning[W0201]: 1 row with NA in value was skipped (line 2)\n/// > error(\"the example stops here\")\n/// 1\n/// > mean_of(frame(x = [4.0])[\"x\"])\n/// 4\n{body}{hidden}"
    );
    let tests = tested(&[("functions/summaries.zeff", &module)]);
    insta::assert_snapshot!("examples_that_warn_and_stop", shown(&tests));
    // The examples of the private function are not run: no script reaches it.
    assert!(tests.examples.iter().all(|e| e.function == "mean_of"));
    assert_eq!(
        tests
            .examples
            .iter()
            .map(|e| e.passed())
            .collect::<Vec<_>>(),
        [true, false, false]
    );
    assert_eq!(tests.exit_code(), 1);
    // An error of the check, a name that does not exist: nothing runs, and the first
    // example shows the error.
    let module = format!(
        "{page}/// > mean_of(frame(x = [4.0])[\"x\"])\n/// 4\n/// > mean_of(nothing_here)\n/// 1\n{body}"
    );
    let tests = tested(&[("functions/summaries.zeff", &module)]);
    assert_eq!(tests.examples.len(), 2);
    assert!(
        tests.examples[0].found[0].starts_with("error[Z0101]"),
        "{}",
        shown(&tests)
    );
    assert!(tests.examples[1].found.is_empty());
    assert_eq!(tests.counts(), (2, 2));
}

#[test]
fn a_project_without_tests_has_none() {
    let tests = tested(&[("scripts/main.zeff", "print(1)\n")]);
    assert!(tests.scripts.is_empty() && tests.examples.is_empty() && tests.problems.is_empty());
    assert_eq!(tests.without_checks, ["scripts/main.zeff"]);
    assert_eq!(tests.counts(), (0, 0));
    assert_eq!(tests.exit_code(), 0);
    let tests = tested(&[]);
    assert_eq!(tests, ProjectTests::default());
}

#[test]
fn warnings_refused_and_internal_errors_have_their_exit_codes() {
    let script = "let d = frame(x = [1, NA, 3])\nassert(d.n_rows() == 3, \"three rows\")\nlet m = d.mean(x)\nassert(m == 2, \"the mean of the values present\")\n";
    let module = "/// The mean of a column, by a frame.\n///\n/// The mean of the values present.\n///\n/// x: The column.\n/// returns: The mean, a Float.\n/// > mean_of(frame(x = [1, NA, 3])[\"x\"])\n/// 2\n/// warning[W0201]: 1 row with NA in value was skipped (line 2)\nfn mean_of(x: Column<Float>) -> Float {\n    frame(value = x).mean(value)\n}\n";
    let files = [
        ("scripts/main.zeff", script),
        ("functions/summaries.zeff", module),
    ];
    // As it is, the warning is a warning and both checks hold.
    let tests = tested(&files);
    assert_eq!(tests.counts(), (3, 0), "{}", shown(&tests));
    assert_eq!(tests.exit_code(), 0);
    // Under `--deny-warnings` the script stops at the warning, with exit code 3; the example,
    // whose page shows its warning, is not held to the option.
    let (fs, project) = project(&files);
    let deny = RunOptions {
        deny_warnings: true,
        ..RunOptions::default()
    };
    let tests = test_project(&fs, &project, &deny);
    let script = &tests.scripts[0];
    assert_eq!(
        (script.checks, script.exit_code),
        (1, 3),
        "{}",
        shown(&tests)
    );
    assert!(
        script
            .stopped
            .as_deref()
            .is_some_and(|s| s.contains("W0201")),
        "{:?}",
        script.stopped
    );
    assert!(tests.examples[0].passed());
    assert_eq!(tests.exit_code(), 3);
    // An internal error at a statement: exit code 2.
    let panic = RunOptions {
        panic_at_line: Some(3),
        ..RunOptions::default()
    };
    let tests = test_project(&fs, &project, &panic);
    assert_eq!(tests.scripts[0].exit_code, 2, "{}", shown(&tests));
    assert_eq!(tests.exit_code(), 2);
}

#[test]
fn a_script_that_cannot_be_read_is_a_problem() {
    let (fs, project) = project(&[("scripts/main.zeff", "assert(1 == 1, \"one\")\n")]);
    fs.add_file("/p/scripts/latin.zeff", b"let caf\xe9 = 1\n");
    let tests = test_project(&fs, &project, &RunOptions::default());
    assert_eq!(tests.problems.len(), 1);
    assert!(
        tests.problems[0].starts_with("cannot read scripts/latin.zeff: "),
        "{}",
        tests.problems[0]
    );
    assert_eq!(tests.scripts.len(), 1);
    assert_eq!(tests.exit_code(), 1);
}
