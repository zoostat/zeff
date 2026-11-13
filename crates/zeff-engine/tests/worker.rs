//! The engine worker, natively (release 0.7.0, step 33b; Engine and Project Specification,
//! section 8.3, point 7): a run on a thread of its own, told as it goes, cancelled from
//! outside.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::control::Event;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{RunOptions, run_script};
use zeff_engine::worker::{Message, Run, start};

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn started(script: &str) -> Run {
    let (fs, project) = project();
    start(
        fs,
        project,
        "scripts/main.zeff",
        script,
        RunOptions::default(),
    )
    .expect("a thread")
}

const SCRIPT: &str = "print(\"one\")\nlet d = frame(x = [1.0, NA, 3])\nprint(d.mean(x))\nfor i in 1..3 {\n    print(i)\n}\n";

#[test]
fn a_run_on_its_own_thread_is_the_run_it_was() {
    let (fs, project) = project();
    let plain = run_script(&fs, &project, "scripts/main.zeff", SCRIPT);
    let run = started(SCRIPT);
    // Everything it prints and says arrives in pieces, in order, before its outcome.
    let (mut stdout, mut stderr) = (String::new(), String::new());
    let mut passes = 0;
    let mut outcome = None;
    while let Some(message) = run.next() {
        match message {
            Message::Event(Event::Output {
                stdout: o,
                stderr: e,
            }) => {
                assert!(outcome.is_none(), "a piece after the outcome");
                stdout.push_str(&o);
                stderr.push_str(&e);
            }
            Message::Event(Event::Pass { .. }) => passes += 1,
            Message::Event(_) => {}
            Message::Done(done) => outcome = Some(*done),
        }
    }
    let outcome = outcome.expect("the run ends with its outcome");
    assert_eq!(outcome, plain);
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(stdout, plain.output.stdout);
    assert_eq!(stderr, plain.output.stderr);
    assert_eq!(stdout, "one\n2\n1\n2\n3\n");
    assert!(stderr.contains("W0201"), "{stderr}");
    assert_eq!(passes, 3);
    // And waited for without reading the messages, it gives the same outcome.
    assert_eq!(started(SCRIPT).wait(), plain);
}

#[test]
fn a_cancel_during_a_long_regression_stops_the_run_after_it() {
    // The regression of the third statement is one step: cancelled while it runs, the run
    // lets it finish and stops before the fourth statement.
    let script = "print(\"before\")\nlet w = dataset(wages)\nlet m = w.reg(log(wage) ~ educ + exper + exper^2 + female + union + industry, fe: worker + year, cluster: worker)\nprint(\"after\")\nprint(m.n_obs())\n";
    let run = started(script);
    let mut regression_finished = false;
    let mut outcome = None;
    while let Some(message) = run.next() {
        match message {
            Message::Event(Event::Started { statement: 3, .. }) => run.cancel(),
            Message::Event(Event::Finished {
                statement: 3,
                failed,
                ..
            }) => {
                assert!(!failed);
                regression_finished = true;
            }
            Message::Event(Event::Started { statement, .. }) => {
                assert!(
                    statement < 3,
                    "statement {statement} started after the cancel"
                );
            }
            Message::Event(_) => {}
            Message::Done(done) => outcome = Some(*done),
        }
    }
    let outcome = outcome.expect("the run ends with its outcome");
    assert!(regression_finished);
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.output.stdout, "before\n");
    assert!(
        outcome
            .output
            .stderr
            .contains("error[Z0705]: the run was cancelled before this statement\n --> scripts/main.zeff:4:1\n"),
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn a_run_is_cancelled_from_its_handle_on_another_thread() {
    // A loop that would run long; the handle is cloned to a thread that cancels it.
    let script = "let total = [for i in 1..2000000 { i * 2 }]\nprint(\"never\")\n";
    let run = started(script);
    let handle = run.control();
    // Wait until the loop is under way, then cancel from elsewhere.
    loop {
        match run.next() {
            Some(Message::Event(Event::Pass { pass, .. })) if pass >= 10 => break,
            Some(Message::Done(_)) | None => panic!("the run ended before it was cancelled"),
            Some(_) => {}
        }
    }
    std::thread::spawn(move || handle.cancel()).join().unwrap();
    let outcome = run.wait();
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.output.stdout, "");
    assert!(
        outcome
            .output
            .stderr
            .starts_with("error[Z0705]: the run was cancelled before pass "),
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn a_run_nobody_waits_for_ends_by_itself_and_one_can_ask_without_waiting() {
    // Dropped at once: the thread runs to its end and is not waited for.
    drop(started(SCRIPT));
    // Asked without waiting, a run answers with what is there, or with nothing yet.
    let run = started("print(\"one\")\n");
    let mut seen = 0;
    let mut done = false;
    while !done {
        match run.try_next() {
            Some(Message::Done(outcome)) => {
                assert_eq!(outcome.output.stdout, "one\n");
                done = true;
            }
            Some(Message::Event(_)) => seen += 1,
            None => std::thread::yield_now(),
        }
    }
    assert_eq!(seen, 3);
    assert!(run.try_next().is_none());
    assert!(run.next().is_none());
}

#[test]
fn an_error_of_the_script_and_an_internal_error_come_back_as_outcomes() {
    let outcome = started("print(nothing_here)\n").wait();
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.output.stderr.starts_with("error[Z0101]"));
    let (fs, project) = project();
    let options = RunOptions {
        panic_at_line: Some(2),
        ..RunOptions::default()
    };
    let run = start(
        fs,
        project,
        "scripts/main.zeff",
        "print(1)\nprint(2)\n",
        options,
    )
    .unwrap();
    let outcome = run.wait();
    assert_eq!(outcome.exit_code, 2);
    assert_eq!(outcome.output.stdout, "1\n");
    assert!(
        outcome.output.stderr.contains("error[Z9001]"),
        "{}",
        outcome.output.stderr
    );
}
