//! The host of the engine's worker (release 0.7.0, step 33e1), tested natively: a project in
//! memory, a run that tells its events as lines of JSON, a cancel that is asked for, and the
//! browser's default memory budget. The worker script that carries these as messages is tested
//! under Node (`js/worker.test.mjs`).

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use zeff_engine::control::Event;
use zeff_web::{Host, LARGEST_DEFAULT, default_budget, event_json};

fn never() -> bool {
    false
}

/// A run with what it told, line by line, and its outcome.
fn run(
    host: &Host,
    text: &str,
    device: Option<u64>,
    cancelled: fn() -> bool,
) -> (Vec<String>, String) {
    let told: RefCell<Vec<String>> = RefCell::default();
    let done = host.run("scripts/main.zeff", text, device, cancelled, &|json| {
        told.borrow_mut().push(json.to_string());
    });
    (told.into_inner(), done)
}

#[test]
fn a_run_tells_each_event_as_a_line_of_json_and_gives_its_outcome() {
    let host = Host::new("demo");
    let (told, done) = run(&host, "print(\"one\")\nlet a = 2\n", None, never);
    assert_eq!(
        told,
        [
            r#"{"type": "started", "statement": 1, "of": 2, "line": 1}"#,
            r#"{"type": "output", "stdout": "one\n", "stderr": ""}"#,
            r#"{"type": "finished", "statement": 1, "of": 2, "line": 1, "failed": false}"#,
            r#"{"type": "started", "statement": 2, "of": 2, "line": 2}"#,
            r#"{"type": "finished", "statement": 2, "of": 2, "line": 2, "failed": false}"#,
        ]
    );
    assert_eq!(
        done,
        r#"{"type": "done", "exit_code": 0, "stdout": "one\n", "stderr": "", "frames": []}"#
    );
    // A pass of a loop, with its total where the loop knows it and `null` where it does not.
    assert_eq!(
        event_json(&Event::Pass {
            pass: 3,
            of: Some(5),
            line: 2
        }),
        r#"{"type": "pass", "pass": 3, "of": 5, "line": 2}"#
    );
    assert_eq!(
        event_json(&Event::Pass {
            pass: 3,
            of: None,
            line: 2
        }),
        r#"{"type": "pass", "pass": 3, "of": null, "line": 2}"#
    );
    // Text is escaped as JSON asks.
    let (_, done) = run(&host, "print(\"a \\\"quoted\\\" word\")\n", None, never);
    assert!(
        done.contains(r#""stdout": "a \"quoted\" word\n""#),
        "{done}"
    );
}

#[test]
fn the_files_of_the_project_are_put_in_by_their_paths() {
    let host = Host::new("demo");
    host.add_file("data/pay.csv", b"id,wage\n1,23.5\n2,19\n3,31.25\n");
    // A leading slash is the project's folder all the same.
    host.add_file("/data/more.csv", b"id\n7\n");
    let (_, done) = run(
        &host,
        "print(import(\"data/pay.csv\").mean(wage))\nprint(import(\"data/more.csv\").n_rows())\n",
        None,
        never,
    );
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    assert!(done.contains(r#""stdout": "24.5833\n1\n""#), "{done}");
    // A file put in again replaces the one before.
    host.add_file("data/more.csv", b"id\n7\n8\n");
    let (_, done) = run(
        &host,
        "print(import(\"data/more.csv\").n_rows())\n",
        None,
        never,
    );
    assert!(done.contains(r#""stdout": "2\n""#), "{done}");
}

static STOP: AtomicBool = AtomicBool::new(false);

fn stop() -> bool {
    STOP.load(Ordering::SeqCst)
}

#[test]
fn a_run_is_cancelled_when_the_flag_it_asks_for_is_set() {
    let host = Host::new("demo");
    let told: RefCell<Vec<String>> = RefCell::default();
    STOP.store(false, Ordering::SeqCst);
    // The flag is set from outside while the loop goes: here, by the listener at its third
    // pass, as the interface would set it in the memory it shares with the worker.
    let done = host.run(
        "scripts/main.zeff",
        "for i in 1..10 {\n    print(i)\n}\nprint(\"never\")\n",
        None,
        stop,
        &|json| {
            if json.contains(r#""type": "pass", "pass": 3,"#) {
                STOP.store(true, Ordering::SeqCst);
            }
            told.borrow_mut().push(json.to_string());
        },
    );
    STOP.store(false, Ordering::SeqCst);
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    assert!(done.contains(r#""stdout": "1\n2\n3\n""#), "{done}");
    assert!(
        done.contains("error[Z0705]: the run was cancelled before pass 4 of this loop"),
        "{done}"
    );
}

#[test]
fn a_window_of_the_browser_has_a_budget_by_default() {
    // The smaller of 3.5 GB and half of what the device reports; 3.5 GB where it reports none.
    assert_eq!(default_budget(None), LARGEST_DEFAULT);
    assert_eq!(default_budget(Some(8_000_000_000)), LARGEST_DEFAULT);
    assert_eq!(default_budget(Some(4_000_000_000)), 2_000_000_000);
    assert_eq!(default_budget(Some(7_000_000_000)), LARGEST_DEFAULT);
    assert_eq!(default_budget(Some(6_999_999_998)), 3_499_999_999);
    // On a device said to have 4 MB the shipped wages file, counted five times, does not fit.
    let host = Host::new("demo");
    let text = "let w = import(\"zeff:wages.csv\")\nprint(w.n_rows())\n";
    let (_, done) = run(&host, text, Some(4_000_000), never);
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    assert!(done.contains("more than the budget of 2.0 MB"), "{done}");
    // The project's own `memory` comes before the default.
    host.add_file(
        "zeff.toml",
        format!(
            "[project]\nname = \"demo\"\nzeff = \"{}\"\nedition = \"preview\"\nmemory = \"64MB\"\n",
            zeff_engine::VERSION
        )
        .as_bytes(),
    );
    let (_, done) = run(&host, text, Some(4_000_000), never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    // A project file that cannot be read is said, not passed over.
    host.add_file("zeff.toml", b"this is not a project file");
    let (told, done) = run(&host, text, None, never);
    assert!(told.is_empty());
    assert!(
        done.contains(r#""exit_code": 1"#) && done.contains("Z0709"),
        "{done}"
    );
}

// ----- the frames of a run, as Arrow bytes (step 33e2) -----

#[test]
fn a_frame_of_the_last_run_is_given_as_an_arrow_stream_with_its_labels_and_categories() {
    let host = Host::new("demo");
    let text = "let pay = frame(region = [\"north\", \"south\", \"north\"], wage = [23.5, NA, 31.25])\n    .add(region = category(region))\n    .label(wage, \"Hourly wage\")\nlet small = pay.where(wage > 30)\nlet n = pay.n_rows()\n";
    let (_, done) = run(&host, text, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    // The outcome lists the frames its bindings hold, with their shapes; a number is no frame.
    assert!(
        done.ends_with(
            r#""frames": [{"name": "pay", "rows": 3, "variables": 2}, {"name": "small", "rows": 1, "variables": 2}]}"#
        ),
        "{done}"
    );
    let bytes = host.frame("pay").expect("the frame is there");
    // The bytes are an Arrow stream: read back, the table is the frame, type by type, value
    // by value, with the label and the category's levels.
    let table = zeff_columns::read_ipc(&bytes).expect("an Arrow stream");
    assert_eq!(table.names(), ["region", "wage"]);
    assert_eq!(table.rows(), 3);
    let region = table.column("region").unwrap();
    assert_eq!(region.ty(), zeff_columns::ColumnType::Category);
    assert_eq!(region.levels().unwrap(), ["north", "south"]);
    let wage = table.column("wage").unwrap();
    assert_eq!(wage.ty(), zeff_columns::ColumnType::Float);
    assert_eq!(
        wage.metadata().get(zeff_columns::MetadataKey::Label),
        Some("Hourly wage")
    );
    assert!(wage.is_na(1), "the missing value stays missing");
    assert!(!wage.is_na(0) && !wage.is_na(2));
    // Written again, the table is the same bytes: nothing was lost on the way.
    assert_eq!(zeff_columns::write_ipc(&table).unwrap(), bytes);
    // The selection is a frame of its own.
    let small = zeff_columns::read_ipc(&host.frame("small").unwrap()).unwrap();
    assert_eq!(small.rows(), 1);
    // A name that holds no frame gives none.
    assert!(host.frame("n").is_none());
    assert!(host.frame("nothing").is_none());
}

#[test]
fn the_frames_are_those_of_the_last_run_and_stay_where_a_run_stops() {
    let host = Host::new("demo");
    // A run that stops: the frame bound before the error is there, the one after is not.
    let (_, done) = run(
        &host,
        "let a = frame(x = [1, 2])\nerror(\"stop\")\nlet b = frame(y = [3])\n",
        None,
        never,
    );
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    assert!(
        done.ends_with(r#""frames": [{"name": "a", "rows": 2, "variables": 1}]}"#),
        "{done}"
    );
    assert!(host.frame("a").is_some() && host.frame("b").is_none());
    // The next run replaces them.
    let (_, done) = run(&host, "let c = frame(z = [1.5])\n", None, never);
    assert!(
        done.ends_with(r#""frames": [{"name": "c", "rows": 1, "variables": 1}]}"#),
        "{done}"
    );
    assert!(host.frame("a").is_none() && host.frame("c").is_some());
    // A run with no frame has none.
    let (_, done) = run(&host, "print(1)\n", None, never);
    assert!(done.ends_with(r#""frames": []}"#), "{done}");
}
