//! Progress and cancellation between the steps of a run (release 0.7.0, step 33a; Engine and
//! Project Specification, section 8.3, point 7; Language Reference, section 11, Z0705).

use std::path::Path;
use std::sync::{Arc, Mutex};
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

fn under(script: &str, control: &Control) -> Outcome {
    let (fs, project) = project();
    run_script_under(
        &fs,
        &project,
        "scripts/main.zeff",
        script,
        &RunOptions::default(),
        control,
    )
}

/// A handle that keeps every event, and cancels the run when `stop` says so.
fn watching(
    stop: impl Fn(&Event) -> bool + Send + Sync + 'static,
) -> (Control, Arc<Mutex<Vec<Event>>>) {
    let seen: Arc<Mutex<Vec<Event>>> = Arc::default();
    let kept = seen.clone();
    let flag: Arc<Mutex<Option<Control>>> = Arc::default();
    let handle = flag.clone();
    let control = Control::with_listener(move |event| {
        kept.lock().unwrap().push(event.clone());
        if stop(event)
            && let Some(control) = handle.lock().unwrap().as_ref()
        {
            control.cancel();
        }
    });
    *flag.lock().unwrap() = Some(control.clone());
    (control, seen)
}

const THREE: &str = "print(\"one\")\nlet a = 2\nprint(\"three\")\n";

#[test]
fn a_run_that_is_not_cancelled_is_the_run_it_was_and_tells_each_statement() {
    let (control, seen) = watching(|_| false);
    let out = under(THREE, &control);
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
    assert_eq!(
        *seen.lock().unwrap(),
        [
            started(1, 1),
            finished(1, 1),
            started(2, 2),
            finished(2, 2),
            started(3, 3),
            finished(3, 3)
        ]
    );
    assert!(!control.is_cancelled());
}

#[test]
fn a_run_cancelled_before_it_starts_runs_nothing() {
    let (control, seen) = watching(|_| false);
    control.cancel();
    let out = under(THREE, &control);
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "");
    assert!(
        out.output
            .stderr
            .starts_with("error[Z0705]: the run was cancelled before this statement\n --> scripts/main.zeff:1:1\n"),
        "{}",
        out.output.stderr
    );
    // Nothing started, so nothing is reported.
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn a_run_cancelled_between_two_statements_keeps_what_ran() {
    // Cancelled while the first statement runs: it finishes, and the second does not start.
    let (control, seen) = watching(|event| matches!(event, Event::Started { statement: 1, .. }));
    let out = under(THREE, &control);
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "one\n");
    insta::assert_snapshot!("cancelled_before_a_statement", out.output.stderr);
    assert_eq!(
        *seen.lock().unwrap(),
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
    let (control, seen) = watching(|event| matches!(event, Event::Pass { pass: 3, .. }));
    let out = under(script, &control);
    assert_eq!(out.exit_code, 1);
    // The pass that was running when the run was cancelled finishes.
    assert_eq!(out.output.stdout, "1\n2\n3\n");
    insta::assert_snapshot!("cancelled_inside_a_loop", out.output.stderr);
    let events = seen.lock().unwrap().clone();
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
    let (control, _) = watching(|event| matches!(event, Event::Pass { pass: 2, .. }));
    let out = under(
        "let squares = [for i in 1..4 { i * i }]\nprint(squares)\n",
        &control,
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
    let (control, seen) = watching(|event| matches!(event, Event::Pass { pass: 4, .. }));
    let out = under(script, &control);
    assert_eq!(out.exit_code, 1, "{}", out.output.stderr);
    insta::assert_snapshot!("cancelled_inside_iterate", out.output.stderr);
    let passes: Vec<Event> = seen
        .lock()
        .unwrap()
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
    let (control, seen) = watching(|_| false);
    let out = under(script, &control);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "10\n");
    let passes: Vec<Event> = seen
        .lock()
        .unwrap()
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
    let (control, seen) = watching(|_| false);
    let out = under(
        "print(\"one\")\nerror(\"stop\")\nprint(\"never\")\n",
        &control,
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(
        seen.lock().unwrap().last(),
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
    let (first, _) = watching(|_| false);
    let other = first.clone();
    other.cancel();
    assert!(first.is_cancelled());
    assert!(format!("{first:?}").contains("cancelled: true"));
}
