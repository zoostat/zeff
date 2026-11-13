//! The memory budget of a run (release 0.7.0, step 33c; Engine and Project Specification,
//! section 8.3, point 7; Language Reference, section 11, Z0704; open decision #143).

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::interpreter::MemoryNote;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, run_script_with};

/// A column of ten thousand whole numbers as a CSV file: 58,894 bytes of text.
fn one_column() -> String {
    let mut text = String::from("x\n");
    for i in 0..10_000 {
        text.push_str(&format!("{}\n", 10_000 + i * 3));
    }
    text
}

fn project(manifest_memory: Option<&str>) -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    let memory = manifest_memory.map_or(String::new(), |m| format!("memory = \"{m}\"\n"));
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n{memory}"),
    );
    fs.add_file("/p/data/one.csv", one_column());
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn run(script: &str, memory: Option<u64>, manifest_memory: Option<&str>) -> Outcome {
    let (fs, project) = project(manifest_memory);
    let options = RunOptions {
        memory,
        ..RunOptions::default()
    };
    run_script_with(&fs, &project, "scripts/main.zeff", script, &options)
}

const READ: &str = "print(\"start\")\nlet d = import(\"data/one.csv\")\nprint(d.n_rows())\n";

/// The bytes of the file, and what the run holds once the frame is bound.
fn sizes() -> (u64, u64) {
    let file = one_column().len() as u64;
    let out = run(READ, None, None);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    let held = out
        .output
        .memory
        .iter()
        .find_map(|note| match note {
            MemoryNote::Bound {
                name, own, kept, ..
            } if name == "d" => Some((own + kept) as u64),
            _ => None,
        })
        .expect("the frame is bound");
    (file, held)
}

#[test]
fn a_run_without_a_budget_is_the_run_it_was() {
    let out = run(READ, None, None);
    assert_eq!(out.exit_code, 0);
    assert!(
        out.output.stdout.starts_with("start\n10000\n"),
        "{}",
        out.output.stdout
    );
    assert!(!out.output.stderr.contains("Z0704"));
    // A budget far above the need changes nothing either.
    let roomy = run(READ, Some(1_000_000_000), None);
    assert_eq!(roomy, out);
}

#[test]
fn a_budget_below_one_column_refuses_the_file_before_it_is_read() {
    // The one column is 80,000 bytes of numbers at the least; the budget is 64 kB.
    let (fs, project) = project(None);
    let options = RunOptions {
        memory: Some(64_000),
        ..RunOptions::default()
    };
    let out = run_script_with(&fs, &project, "scripts/main.zeff", READ, &options);
    assert_eq!(out.exit_code, 1);
    // The statement before it ran; the file was not read, or the reader would have said so.
    assert_eq!(out.output.stdout, "start\n");
    assert!(
        !out.output.stderr.contains("read as CSV"),
        "{}",
        out.output.stderr
    );
    insta::assert_snapshot!("a_file_over_the_budget", out.output.stderr);
}

#[test]
fn an_estimate_that_meets_the_budget_exactly_runs_and_one_byte_less_does_not() {
    let (file, _) = sizes();
    // Rule 1: nothing is held yet, and the file counts five times.
    let exact = run(READ, Some(file * 5), None);
    assert_eq!(exact.exit_code, 0, "{}", exact.output.stderr);
    let short = run(READ, Some(file * 5 - 1), None);
    assert_eq!(short.exit_code, 1);
    assert!(
        short.output.stderr.starts_with("error[Z0704]"),
        "{}",
        short.output.stderr
    );
}

#[test]
fn a_statement_that_builds_a_frame_from_another_counts_it_once_more() {
    let (file, held) = sizes();
    // Rule 3: `add` reads `d`, so the estimate is what the run holds and `d` once more.
    let script = "let d = import(\"data/one.csv\")\nlet e = d.add(y = x * 2)\nprint(e.n_cols())\n";
    let roomy = (file * 5).max(held * 2);
    let out = run(script, Some(roomy), None);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "2\n");
    // A budget that lets the file in but not the second frame.
    assert!(
        held * 2 > file * 5,
        "the test needs the copy to be the larger need"
    );
    let out = run(script, Some(held * 2 - 1), None);
    assert_eq!(out.exit_code, 1);
    insta::assert_snapshot!("a_copy_over_the_budget", out.output.stderr);
}

#[test]
fn frames_that_share_their_columns_are_counted_once() {
    // `e` keeps the column of `d` and adds one; `f` keeps both and adds a third. Counted
    // frame by frame the three would hold six columns; they hold three.
    let script = "let d = import(\"data/one.csv\")\nlet e = d.add(y = x * 2)\nlet f = e.add(z = x * 3)\nprint(d.n_cols() + e.n_cols() + f.n_cols())\nprint(f.mean(z))\n";
    let out = run(script, None, None);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    let sizes: Vec<u64> = out
        .output
        .memory
        .iter()
        .filter_map(|note| match note {
            MemoryNote::Bound { own, kept, .. } => Some((own + kept) as u64),
            _ => None,
        })
        .collect();
    assert_eq!(sizes.len(), 3);
    let (one, naive) = (sizes[0], sizes.iter().sum::<u64>());
    // The fourth statement reads the three frames: what the run holds is three columns, and
    // a copy of what it reads is three more. Six columns fit; the naive count would be twelve.
    let budget = one * 6 + one / 2;
    assert!(budget < naive * 2);
    let out = run(script, Some(budget), None);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "6\n74995.5\n");
}

#[test]
fn a_loop_over_a_range_holds_one_item_and_a_list_it_builds_is_counted() {
    // Rule 2 since step 38g: the range is walked, so a loop of a hundred million passes is
    // within a budget of 64 kB (listed first, its items would be 18.4 GB), and it begins at
    // once: this one leaves at its third pass.
    let out = run(
        "for i in 1..100000000 {\n    if i == 3 {\n        break\n    }\n    print(i)\n}\n",
        Some(64_000),
        None,
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "1\n2\n");
    // A million passes run to the end within one megabyte.
    let out = run(
        "for i in 1..1000000 {\n    let j = i\n}\nprint(\"done\")\n",
        Some(1_000_000),
        None,
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "done\n");
    // A list built by the loop holds a value for each pass, and is counted with the room a
    // growing list takes.
    let out = run(
        "let squares = [for i in 1..1000000 { i * i }]\n",
        Some(1_000_000),
        None,
    );
    assert_eq!(out.exit_code, 1);
    insta::assert_snapshot!("a_loop_over_the_budget", out.output.stderr);
    // The same list fits a budget above the estimate, and a short one any budget worth the
    // name.
    let out = run(
        "let squares = [for i in 1..1000 { i * i }]\nprint(len(squares))\nprint(squares[1000])\n",
        Some(1_000_000),
        None,
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "1000\n1000000\n");
    let out = run("for i in 1..10 {\n    print(i)\n}\n", Some(64_000), None);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    // A loop that reads frames is still judged by rule 3, and its message counts the loop's
    // one item.
    let out = run(
        "let d = import(\"data/one.csv\")\nlet e = d.add(y = x * 2, z = x * 3)\nfor i in 1..3 {\n    print(e.add(w = y * i).n_rows())\n}\n",
        Some(400_000),
        None,
    );
    assert_eq!(out.exit_code, 1, "{}", out.output.stdout);
    assert!(
        out.output
            .stderr
            .contains("184 B for the one item its loop holds at a time"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn the_project_sets_the_budget_and_the_option_overrides_it() {
    // `memory = "64KB"` in zeff.toml refuses the file.
    let out = run(READ, None, Some("64KB"));
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output
            .stderr
            .contains("more than the budget of 64.0 kB"),
        "{}",
        out.output.stderr
    );
    // The option of the run comes first, in either direction.
    let out = run(READ, Some(2_000_000), Some("64KB"));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    let out = run(READ, Some(64_000), Some("2GB"));
    assert_eq!(out.exit_code, 1);
    let out = run(READ, None, Some("2GB"));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
}

#[test]
fn a_file_that_is_not_there_is_the_readers_error_and_not_the_budgets() {
    let out = run("let d = import(\"data/none.csv\")\n", Some(64_000), None);
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.starts_with("error[Z0715]"),
        "{}",
        out.output.stderr
    );
}
