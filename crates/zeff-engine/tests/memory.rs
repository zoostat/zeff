//! The memory rules of section 6 (release 0.6.0, step 6): in `zeff run` a binding is freed after
//! its last use, a binding read inside a loop lives to the loop's end, a binding never read is
//! freed at once, and a frame made from a freed frame gathers what it still read from it; the
//! record the interpreter keeps of all this is what `--verbose` prints.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::interpreter::MemoryNote;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, run_script_with};

fn run(script: &str, free: bool) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let options = RunOptions {
        free_bindings: free,
        ..RunOptions::default()
    };
    run_script_with(&fs, &project, "scripts/main.zeff", script, &options)
}

/// The record as text: `name@line` for a binding, `-name@line(n)` for one freed with the
/// columns gathered from it.
fn record(outcome: &Outcome) -> Vec<String> {
    outcome
        .output
        .memory
        .iter()
        .map(|n| match n {
            MemoryNote::Bound {
                name, line, kept, ..
            } => format!("{name}@{line}{}", if *kept > 0 { "+" } else { "" }),
            MemoryNote::Freed {
                name,
                line,
                gathered,
            } => format!("-{name}@{line}({gathered})"),
        })
        .collect()
}

#[test]
fn bindings_are_freed_after_their_last_use() {
    let script = "let w = dataset(wages)\nlet big = w.where(wage > 10)\nlet small = w.where(wage > 60)\nbig.n_rows()\nlet k = 2\nsmall.n_rows()\n";
    let outcome = run(script, true);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "17552\n595\n");
    // `w` is last used at line 3, so it goes after that statement, and `big` (most rows kept,
    // so pending) gathers what it still read from it; `small` kept few rows and was gathered at
    // once. `k` is never read: freed at its own line.
    assert_eq!(
        record(&outcome),
        [
            "w@1",
            "big@2+",
            "small@3",
            "-w@3(11)",
            "-big@4(0)",
            "-k@5(0)",
            "-small@6(0)"
        ]
    );
}

#[test]
fn a_binding_read_in_a_loop_lives_to_the_loops_end_and_a_session_keeps_everything() {
    let script = "let w = dataset(wages)\nlet total = 0\nfor y in [2015, 2016] {\n    w.where(year == y).n_rows()\n}\nw.n_cols()\n";
    let outcome = run(script, true);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(record(&outcome), ["w@1", "-total@2(0)", "-w@6(0)"]);
    let kept = run(script, false);
    assert_eq!(kept.exit_code, 0);
    assert_eq!(record(&kept), ["w@1"]);
}

#[test]
fn a_frame_from_a_freed_frame_answers_from_its_own_buffers() {
    // `year >= 2012` keeps most rows, so the selection stays pending; the sort reads `wage`
    // through it, and the ten other columns are gathered when `w` goes.
    let script = "let w = dataset(wages)\nlet recent = w.where(year >= 2012).sort(wage)\nrecent.head(2)\nrecent.summary([wage])\n";
    let outcome = run(script, true);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stdout.contains("wage"),
        "{}",
        outcome.output.stdout
    );
    assert_eq!(
        record(&outcome),
        ["w@1", "recent@2+", "-w@2(10)", "-recent@4(0)"]
    );
}
