//! Progress and cancellation between the steps of a run (release 0.7.0, step 33a; Engine and
//! Project Specification, section 8.3, point 7; Language Reference, section 11, Z0705).

use std::cell::RefCell;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::control::{Control, Event};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, run_script, run_script_under};

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

/// A script run under a handle, with every event kept; the run is cancelled when `stop`
/// says so of an event, or before it starts.
fn watched(
    script: &str,
    cancel_first: bool,
    stop: impl Fn(&Event) -> bool,
) -> (Outcome, Vec<Event>, Control) {
    let (fs, project) = project();
    let control = Control::new();
    if cancel_first {
        control.cancel();
    }
    let seen: RefCell<Vec<Event>> = RefCell::default();
    let listener = |event: &Event| {
        seen.borrow_mut().push(event.clone());
        if stop(event) {
            control.cancel();
        }
    };
    let out = run_script_under(
        &fs,
        &project,
        "scripts/main.zeff",
        script,
        &RunOptions::default(),
        &control,
        Some(&listener),
    );
    (out, seen.into_inner(), control)
}

/// The events of a run without what was printed, which has its own test.
fn steps(events: &[Event]) -> Vec<Event> {
    events
        .iter()
        .filter(|event| !matches!(event, Event::Output { .. }))
        .cloned()
        .collect()
}

const THREE: &str = "print(\"one\")\nlet a = 2\nprint(\"three\")\n";

#[test]
fn a_run_that_is_not_cancelled_is_the_run_it_was_and_tells_each_statement() {
    let (out, seen, control) = watched(THREE, false, |_| false);
    let (fs, project) = project();
    let plain = run_script(&fs, &project, "scripts/main.zeff", THREE);
    assert_eq!(out, plain);
    assert_eq!(out.exit_code, 0);
    assert_eq!(out.output.stdout, "one\nthree\n");
    let started = |statement, line| Event::Started {
        statement,
        of: 3,
        line,
    };
    let finished = |statement, line| Event::Finished {
        statement,
        of: 3,
        line,
        failed: false,
    };
    let printed = |text: &str| Event::Output {
        stdout: text.to_string(),
        stderr: String::new(),
    };
    // What a statement printed is told before the statement is said to have finished.
    assert_eq!(
        seen,
        [
            started(1, 1),
            printed("one\n"),
            finished(1, 1),
            started(2, 2),
            finished(2, 2),
            started(3, 3),
            printed("three\n"),
            finished(3, 3)
        ]
    );
    assert!(!control.is_cancelled());
}

#[test]
fn the_pieces_told_are_the_whole_of_what_the_run_printed_and_said() {
    // A warning of the check before the run, a warning of a statement, output, and an error.
    let script = "/// Unused.\nfn spare(x: Float) -> Float { x }\nlet d = frame(x = [1.0, NA, 3])\nprint(d.mean(x))\nprint(\"between\")\nlet d = 2\nprint(\"never\")\n";
    let (out, seen, _) = watched(script, false, |_| false);
    let (mut stdout, mut stderr) = (String::new(), String::new());
    for event in &seen {
        if let Event::Output {
            stdout: o,
            stderr: e,
        } = event
        {
            stdout.push_str(o);
            stderr.push_str(e);
        }
    }
    assert_eq!(stdout, out.output.stdout);
    assert_eq!(stderr, out.output.stderr);
    assert!(!stderr.is_empty(), "the script was to say something");
    // And of a script that stops at the check, before anything runs.
    let (out, seen, _) = watched("print(nothing_here)\n", false, |_| false);
    assert_eq!(out.exit_code, 1);
    assert_eq!(
        seen,
        [Event::Output {
            stdout: String::new(),
            stderr: out.output.stderr.clone()
        }]
    );
}

#[test]
fn a_run_cancelled_before_it_starts_runs_nothing() {
    let (out, seen, _) = watched(THREE, true, |_| false);
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "");
    assert!(
        out.output
            .stderr
            .starts_with("error[Z0705]: the run was cancelled before this statement\n --> scripts/main.zeff:1:1\n"),
        "{}",
        out.output.stderr
    );
    // Nothing started: the one thing told is the error.
    assert_eq!(steps(&seen), []);
    assert_eq!(seen.len(), 1);
}

#[test]
fn a_run_cancelled_between_two_statements_keeps_what_ran() {
    // Cancelled while the first statement runs: it finishes, and the second does not start.
    let (out, seen, _) = watched(THREE, false, |event| {
        matches!(event, Event::Started { statement: 1, .. })
    });
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "one\n");
    insta::assert_snapshot!("cancelled_before_a_statement", out.output.stderr);
    assert_eq!(
        steps(&seen),
        [
            Event::Started {
                statement: 1,
                of: 3,
                line: 1
            },
            Event::Finished {
                statement: 1,
                of: 3,
                line: 1,
                failed: false
            },
        ]
    );
    // Each statement's own output is kept apart, as for any run.
    assert_eq!(out.output.statements.len(), 2);
    assert!(out.output.statements[1].stderr.contains("Z0705"));
}

#[test]
fn a_loop_is_cancelled_before_its_next_pass_and_keeps_the_passes_before() {
    let script = "let total = 0\nfor i in 1..5 {\n    print(i)\n}\nprint(\"after\")\n";
    let (out, seen, _) = watched(script, false, |event| {
        matches!(event, Event::Pass { pass: 3, .. })
    });
    assert_eq!(out.exit_code, 1);
    // The pass that was running when the run was cancelled finishes.
    assert_eq!(out.output.stdout, "1\n2\n3\n");
    insta::assert_snapshot!("cancelled_inside_a_loop", out.output.stderr);
    let events = steps(&seen);
    assert_eq!(
        events[2..],
        [
            Event::Started {
                statement: 2,
                of: 3,
                line: 2
            },
            Event::Pass {
                pass: 1,
                of: Some(5),
                line: 2
            },
            Event::Pass {
                pass: 2,
                of: Some(5),
                line: 2
            },
            Event::Pass {
                pass: 3,
                of: Some(5),
                line: 2
            },
            Event::Finished {
                statement: 2,
                of: 3,
                line: 2,
                failed: true
            },
        ]
    );
}

#[test]
fn a_list_built_by_a_loop_and_an_iteration_are_cancelled_the_same_way() {
    let (out, _, _) = watched(
        "let squares = [for i in 1..4 { i * i }]\nprint(squares)\n",
        false,
        |event| matches!(event, Event::Pass { pass: 2, .. }),
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "");
    assert!(
        out.output
            .stderr
            .starts_with("error[Z0705]: the run was cancelled before pass 3 of this loop\n"),
        "{}",
        out.output.stderr
    );
    // `iterate` does not know how many passes it will make.
    let script = "/// Halve.\nfn halve(state: Record) -> Record {\n    {x: state.x / 2}\n}\n/// Small enough.\nfn small(old: Record, new: Record) -> Bool {\n    new.x < 0.001\n}\nlet end = iterate(start: {x: 1.0}, step: halve, done: small, max: 100)\nprint(end)\n";
    let (out, seen, _) = watched(script, false, |event| {
        matches!(event, Event::Pass { pass: 4, .. })
    });
    assert_eq!(out.exit_code, 1, "{}", out.output.stderr);
    insta::assert_snapshot!("cancelled_inside_iterate", out.output.stderr);
    let passes: Vec<Event> = seen
        .iter()
        .filter(|event| matches!(event, Event::Pass { .. }))
        .cloned()
        .collect();
    assert_eq!(passes.len(), 4);
    assert_eq!(
        passes[0],
        Event::Pass {
            pass: 1,
            of: None,
            line: 9
        }
    );
}

#[test]
fn a_loop_inside_a_function_reports_under_the_line_of_the_scripts_statement() {
    let script = "/// The sum of the first numbers.\nfn sum_to(n: Int) -> Int {\n    reduce([for i in 1..n { i }], fn(acc, i) acc + i, 0)\n}\nlet a = 1\nlet s = sum_to(4)\nprint(s)\n";
    let (out, seen, _) = watched(script, false, |_| false);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "10\n");
    let passes: Vec<Event> = seen
        .iter()
        .filter(|event| matches!(event, Event::Pass { .. }))
        .cloned()
        .collect();
    assert_eq!(passes.len(), 4);
    assert!(passes.iter().all(|p| matches!(
        p,
        Event::Pass {
            of: Some(4),
            line: 6,
            ..
        }
    )));
}

#[test]
fn a_statement_that_fails_is_reported_as_failed_and_a_cancel_after_the_end_does_nothing() {
    let (out, seen, control) = watched(
        "print(\"one\")\nerror(\"stop\")\nprint(\"never\")\n",
        false,
        |_| false,
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(
        seen.last(),
        Some(&Event::Finished {
            statement: 2,
            of: 3,
            line: 2,
            failed: true
        })
    );
    assert!(out.output.stderr.starts_with("error[Z0001]: stop"));
    // A handle is one run's: cancelling it afterwards changes nothing that was printed.
    control.cancel();
    assert!(control.is_cancelled());
    // A clone is the same handle, as another thread holds it.
    let first = Control::new();
    let other = first.clone();
    other.cancel();
    assert!(first.is_cancelled());
    assert!(format!("{first:?}").contains("cancelled: true"));
}

#[test]
fn a_long_loop_tells_about_a_hundred_of_its_passes_and_is_cancelled_at_any() {
    let script = "let squares = [for i in 1..5000 { i * i }]\nprint(len(squares))\n";
    let (out, seen, _) = watched(script, false, |_| false);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "5000\n");
    let passes: Vec<usize> = seen
        .iter()
        .filter_map(|event| match event {
            Event::Pass {
                pass,
                of: Some(5000),
                ..
            } => Some(*pass),
            _ => None,
        })
        .collect();
    // The first, the last, and one for each hundredth of the loop.
    assert_eq!(passes.len(), 101);
    assert_eq!((passes[0], passes[1], passes[100]), (1, 50, 5000));
    // A cancel is honoured at the very next pass, told or not.
    let (out, _, _) = watched(script, false, |event| {
        matches!(event, Event::Pass { pass: 50, .. })
    });
    assert!(
        out.output
            .stderr
            .starts_with("error[Z0705]: the run was cancelled before pass 51 of this loop\n"),
        "{}",
        out.output.stderr
    );
}
