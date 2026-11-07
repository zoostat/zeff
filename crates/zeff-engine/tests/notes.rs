//! A note, a coded warning and a variable's name (Language Reference, section 5.4; open
//! decision #70; release 0.7.0, step 3.1): `note(message)` prints a note and carries on;
//! `warn` and `note` take `code:`, a warning's code written bare or as text, which a pragma
//! allows and `--deny-warnings` refuses as any coded message; a code no warning has, or an
//! error's code, is Z0401; `name_of(v)` is a variable's name as text. Every message as a
//! snapshot, and the edge cases: `NA` for the code, `name_of`
//! of an optional variable left out, and a function raising both.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, run_script_with};

const SCRIPT: &str = "scripts/main.zeff";

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
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

// ----- notes and coded warnings -----

#[test]
fn a_note_prints_under_the_statement_and_carries_on() {
    let outcome = run("note(\"one row in ten is a re-test\")\nprint(\"carries on\")\n");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.output.stdout, "carries on\n");
    assert_eq!(
        outcome.output.stderr,
        "note: one row in ten is a re-test (line 1)\n"
    );
}

#[test]
fn a_code_written_bare_or_as_text_is_carried_and_printed() {
    for code in ["W0205", "\"W0205\""] {
        let script = format!(
            "warn(\"the sd of one value is NA\", code: {code})\nnote(\"one pass\", code: W0703)\n"
        );
        let outcome = run(&script);
        assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
        assert_eq!(
            outcome.output.stderr,
            "warning[W0205]: the sd of one value is NA (line 1)\nnote[W0703]: one pass (line 2)\n",
            "{script}"
        );
    }
}

#[test]
fn na_for_the_code_is_no_code() {
    let outcome = run("warn(\"plain\", code: NA)\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "warning: plain (line 1)\n");
}

#[test]
fn a_pragma_allows_a_coded_warning_and_deny_warnings_refuses_it() {
    let script = "// zeff: allow(W0205) the counts are known to be small\nwarn(\"small counts\", code: W0205)\nprint(1)\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "");
    assert_eq!(outcome.output.allowed.len(), 1);
    assert_eq!(outcome.output.allowed[0].code, "W0205");
    assert_eq!(outcome.output.allowed[0].message, "small counts");
    // Without the pragma `--deny-warnings` makes it an error, exit code 3, as any warning.
    let denied = run_with("warn(\"small counts\", code: W0205)\nprint(1)\n", true);
    assert_eq!(denied.exit_code, 3, "{}", denied.output.stderr);
    assert!(denied.output.stderr.contains("error[W0205]: small counts"));
    assert_eq!(denied.output.stdout, "");
    // A coded note is refused too; a note without a code never is.
    let denied = run_with("note(\"one pass\", code: W0703)\n", true);
    assert_eq!(denied.exit_code, 3, "{}", denied.output.stderr);
    let kept = run_with("note(\"one pass\")\nprint(2)\n", true);
    assert_eq!(kept.exit_code, 0, "{}", kept.output.stderr);
    assert_eq!(kept.output.stdout, "2\n");
}

#[test]
fn a_function_raises_a_note_and_a_coded_warning_at_its_call() {
    let script = "/// Says which variable it looked at.\nfn looked_at(f: Frame, v: Var) -> Int {\n    note(name_of(v))\n    warn(\"a coded warning from a function\", code: W0201)\n    f.n_rows()\n}\nlet d = frame(x = [1, 2, 3])\nd.looked_at(x)\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "3\n");
    assert_eq!(
        outcome.output.stderr,
        "note: x (line 8)\nwarning[W0201]: a coded warning from a function (line 8)\n"
    );
}

// ----- name_of -----

#[test]
fn name_of_gives_the_name_as_text() {
    let outcome = run("name_of(educ)\nprint(name_of(educ))\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "educ\neduc\n");
    // An optional variable left out is `NA`, and so is its name.
    let script =
        "/// The name, or NA.\nfn named(v: Var?) -> Str {\n    name_of(v)\n}\nnamed()\nnamed(x)\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "NA\nx\n");
}

// ----- every message -----

#[test]
fn every_message_of_notes() {
    let cases: &[(&str, &str)] = &[
        // A code no warning has.
        ("unknown_code", "warn(\"x\", code: W9999)\n"),
        // An error's code.
        ("error_code", "warn(\"x\", code: Z0101)\n"),
        // Text that is not a code at all.
        ("not_a_code", "note(\"x\", code: \"skipped\")\n"),
        // A bare name that is no code and no binding.
        ("bare_name", "warn(\"x\", code: skipped)\n"),
        // The result of `note` bound.
        ("note_bound", "let n = note(\"x\")\n"),
        // `name_of` of something that is not a variable.
        ("name_of_text", "name_of(\"educ\")\n"),
        // A `Var` parameter not given: Z0406, as any required argument.
        ("name_of_nothing", "name_of()\n"),
    ];
    let mut text = String::new();
    for (name, script) in cases {
        let outcome = run(script);
        assert_ne!(outcome.exit_code, 0, "{name} should fail");
        text.push_str(&format!("## {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("every_message", text);
}
