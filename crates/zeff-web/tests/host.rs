//! The host of the engine's worker (release 0.7.0, step 33e1), tested natively: a project in
//! memory, a run that tells its events as lines of JSON, a cancel that is asked for, and the
//! browser's default memory budget. The worker script that carries these as messages is tested
//! under Node (`js/worker.test.mjs`).

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use zeff_engine::control::Event;
use zeff_web::{Host, LARGEST_DEFAULT, default_budget, event_json, explain, explained_codes};

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
        r#"{"type": "done", "exit_code": 0, "stdout": "one\n", "stderr": "", "statements": [{"index": 1, "line": 1, "end_line": 1, "ran": true, "name": null, "type": null, "rows": null, "columns": null, "length": null, "n": null, "stdout": "one\n", "stderr": "", "reads": [], "calls": [], "seeds": false, "draws": false, "reads_file": false, "writes_file": false, "drew": false, "error": null}, {"index": 2, "line": 2, "end_line": 2, "ran": true, "name": "a", "type": "Int", "rows": null, "columns": null, "length": null, "n": null, "stdout": "", "stderr": "", "reads": [], "calls": [], "seeds": false, "draws": false, "reads_file": false, "writes_file": false, "drew": false, "error": null}], "plots": [], "frames": []}"#
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

/// The explained page of a code, for Zemp's log and Help pane (release 0.8.0, step 19): the
/// record of `zeff explain --format json` inside a line of type `explained`, by the code or by
/// a diagnostic's link; an error for a code no page explains; and the list by family.
#[test]
fn a_code_is_explained_and_the_codes_are_listed() {
    let line = explain("W1007");
    assert!(!line.contains('\n'));
    assert!(
        line.starts_with(r#"{"type": "explained", "page": {"code": "W1007", "kind": "warning", "title": "weak instruments", "#),
        "{line}"
    );
    assert_eq!(explain("zeff://explain/W1007"), line);
    assert_eq!(explain("w1007"), line);
    assert_eq!(
        explain("Z9999"),
        r#"{"type": "error", "message": "no code `Z9999` has an explained page"}"#
    );
    assert_eq!(
        explain(""),
        r#"{"type": "error", "message": "no code `` has an explained page"}"#
    );
    let codes = explained_codes();
    assert!(!codes.contains('\n'));
    assert!(
        codes.starts_with(
            r#"{"type": "codes", "families": [{"name": "Z00xx", "area": "Errors a script raises itself", "codes": [{"code": "Z0001", "kind": "error", "title": "the script stopped itself with error()"}]}, {"name": "Z01xx", "#
        ),
        "{codes}"
    );
    // Every code of the engine is in the list, once.
    for code in zeff_engine::explain::codes() {
        assert_eq!(
            codes.matches(&format!(r#"{{"code": "{code}", "#)).count(),
            1,
            "{code}"
        );
    }
}

/// "Report a problem" (release 0.8.0, step 20a): the report of a failing script as a line of
/// type `report`, with nothing written into the project; a result believed wrong with its
/// reference; and a line of type `error` where there is nothing to report.
#[test]
fn a_problem_is_reported_as_a_line_of_json_and_nothing_is_written() {
    let host = Host::new("demo");
    host.add_file("data/pay.csv", b"id,wage\n1,23.5\n2,19\n3,31.25\n");
    let script = "let pay = import(\"data/pay.csv\")\nprint(\"between\")\npay.mean(wagee)\n";
    let line = host.report_bug("scripts/main.zeff", script, None, None, "a test", None);
    assert!(!line.contains('\n'));
    assert!(
        line.starts_with(r#"{"type": "report", "name": "main-Z0101-"#),
        "{line}"
    );
    for part in [
        r##""kind": "error", "code": "Z0101", "kept": 2, "of": 3, "reduced": true, "files": ["data/pay.csv"], "text": "# Zeff bug report: an error, Z0101\n"##,
        r"- platform: a test\n",
        r"no variable `wagee` in `pay`",
    ] {
        assert!(line.contains(part), "{part}:\n{line}");
    }
    assert!(!line.contains("31.25"), "{line}");
    // A result believed wrong, with what it should be.
    let fine = "let pay = import(\"data/pay.csv\")\npay.mean(wage)\n";
    let wrong = host.report_bug(
        "scripts/main.zeff",
        fine,
        Some(2),
        Some(("r.txt", "[1] 24.58333\n")),
        "a test",
        None,
    );
    assert!(
        wrong.contains(r#""kind": "wrong result", "code": null, "kept": 2, "of": 2"#),
        "{wrong}"
    );
    assert!(wrong.contains(r"[1] 24.58333"), "{wrong}");
    // Nothing to report.
    let none = host.report_bug("scripts/main.zeff", fine, None, None, "a test", None);
    assert!(
        none.starts_with(
            r#"{"type": "error", "message": "the script ran without an error: there is nothing to report"#
        ),
        "{none}"
    );
    // Nothing was written into the project's reports.
    assert!(host.holds("data/pay.csv"));
    assert!(!host.holds(".zeff/reports"));
}

// ----- the plots of a run (open decision #197; release 0.9.0, step 25) -----

const PLOTS: &str = "let d = frame(x = [1.0, 2, 3, 4, 5, 6], y = [2.0, 1, 4, 3, 6, 5], g = [\"a\", \"b\", \"a\", \"b\", \"a\", \"b\"])\nlet p = d.scatter(x, y, color: g)\nlet n = d.n_rows()\nd.histogram(x, bins: 3)\nd.plot(x: x, y: y).lines().points().theme(minimal)\nprint(p)\nlet q = p\n";

/// The list of plots in a run's outcome.
fn plots_of(done: &str) -> &str {
    let from = done
        .find("\"plots\": [")
        .expect("the outcome lists its plots");
    let to = done.find("], \"frames\"").expect("the frames follow");
    &done[from..=to]
}

#[test]
fn the_outcome_of_a_run_lists_the_plots_its_statements_made() {
    let host = Host::new("demo");
    let (_, done) = run(&host, PLOTS, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    // A plot a `let` binds, by its name; one a statement gives, by its line; a number is no
    // plot; a plot that is printed is listed where it was made and not again; a second name
    // for a plot is a plot.
    assert_eq!(
        plots_of(&done),
        r#""plots": [{"index": 0, "name": "p", "line": 2, "kind": "points", "theme": "journal"}, {"index": 1, "name": null, "line": 4, "kind": "bars", "theme": "journal"}, {"index": 2, "name": null, "line": 5, "kind": "lines, points", "theme": "minimal"}, {"index": 3, "name": "q", "line": 7, "kind": "points", "theme": "journal"}]"#
    );
    // The frames are listed as before, after them.
    assert!(
        done.ends_with(r#""frames": [{"name": "d", "rows": 6, "variables": 3}]}"#),
        "{done}"
    );
    // A run with no plot lists none, and the plots are those of the last run.
    let (_, done) = run(&host, "let n = 1\n", None, never);
    assert_eq!(plots_of(&done), r#""plots": []"#);
    assert!(
        host.plot_svg(0, 400.0, 300.0, None)
            .contains("the last run made no plot")
    );
    // A run that stops keeps the plots made before it stopped.
    let (_, done) = run(
        &host,
        "let d = frame(x = [1.0, 2, 3], y = [3.0, 1, 2])\nlet p = d.scatter(x, y)\nassert(d.n_rows() > 5, \"three rows are too few\")\nlet late = d.scatter(y, x)\n",
        None,
        never,
    );
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    assert_eq!(
        plots_of(&done),
        r#""plots": [{"index": 0, "name": "p", "line": 2, "kind": "points", "theme": "journal"}]"#
    );
}

/// The SVG text of a line of JSON of type `plot`.
fn svg_of(line: &str) -> String {
    assert!(line.starts_with(r#"{"type": "plot", "#), "{line}");
    let from = line.find("\"svg\": \"").expect("an svg") + 8;
    let escaped = &line[from..line.len() - 2];
    // The few escapes a JSON string of an SVG has.
    escaped
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

/// The finish line of the step: a plot laid out again at the size of a page is, byte for
/// byte, the file `export` writes at that size, with no `export` in the script that made it.
#[test]
fn a_plot_laid_out_again_is_the_file_export_writes_at_that_size() {
    let host = Host::new("demo");
    let exporting = format!(
        "{PLOTS}p.export(\"output/one.svg\", size: one_column)\np.export(\"output/two.svg\", size: two_column)\np.export(\"output/slide.svg\", size: slide)\np.export(\"output/own.svg\", size: [120, 80])\n"
    );
    let (_, done) = run(&host, &exporting, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    // The same plot from a run that exports nothing.
    let plain = Host::new("demo");
    let (_, done) = run(&plain, PLOTS, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    for (file, wide, high) in [
        ("output/one.svg", 89.0, 66.0),
        ("output/two.svg", 183.0, 110.0),
        ("output/slide.svg", 254.0, 143.0),
        ("output/own.svg", 120.0, 80.0),
    ] {
        let exported = String::from_utf8(host.file(file).expect("the exported file")).unwrap();
        // Millimetres to points as the layout has it: multiplied before it is divided.
        let (width, height) = (wide * 72.0 / 25.4, high * 72.0 / 25.4);
        let laid = svg_of(&plain.plot_svg(0, width, height, None));
        assert!(
            exported.starts_with("<svg") || exported.starts_with("<?xml"),
            "{file}"
        );
        assert_eq!(laid, exported, "{file}: not the same bytes");
        // The second name of the same plot gives the same picture.
        assert_eq!(
            svg_of(&plain.plot_svg(3, width, height, None)),
            exported,
            "{file}"
        );
    }
    assert!(host.file("output/nothing.svg").is_none());
}

#[test]
fn a_plot_is_drawn_at_any_size_and_theme_without_a_new_run() {
    let host = Host::new("demo");
    let (told, done) = run(&host, PLOTS, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    let events = told.len();
    // Three sizes of a pane, one after another: each is a page of that size, and no run is
    // made for it.
    let mut seen = Vec::new();
    for (width, height) in [(320.0, 240.0), (640.5, 480.25), (1280.0, 720.0)] {
        let line = host.plot_svg(0, width, height, None);
        assert!(
            line.starts_with(&format!(
                r#"{{"type": "plot", "index": 0, "width": {width}, "height": {height}, "theme": "journal", "svg": "#
            )),
            "{}",
            &line[..line.len().min(160)]
        );
        let svg = svg_of(&line);
        assert!(
            svg.contains(&format!("width=\"{width}\""))
                || svg.contains(&format!("0 0 {width} {height}")),
            "{}",
            &svg[..svg.len().min(300)]
        );
        assert!(svg.trim_end().ends_with("</svg>"));
        seen.push(svg);
    }
    assert!(seen[0] != seen[1] && seen[1] != seen[2]);
    // The same size twice is the same bytes.
    assert_eq!(svg_of(&host.plot_svg(0, 320.0, 240.0, None)), seen[0]);
    // Another theme than the plot's own, and the plot's own named.
    let minimal = svg_of(&host.plot_svg(0, 320.0, 240.0, Some("minimal")));
    assert_ne!(minimal, seen[0]);
    assert_eq!(
        svg_of(&host.plot_svg(0, 320.0, 240.0, Some("journal"))),
        seen[0]
    );
    assert!(
        host.plot_svg(0, 320.0, 240.0, Some("slide")).starts_with(
            r#"{"type": "plot", "index": 0, "width": 320, "height": 240, "theme": "slide""#
        ) || host
            .plot_svg(0, 320.0, 240.0, Some("slide"))
            .contains("\"of_size\": true")
    );
    // The plot whose script gave it a theme keeps it where none is asked for.
    assert!(
        host.plot_svg(2, 400.0, 300.0, None)
            .contains(r#""theme": "minimal""#)
    );
    // What cannot be given is said, and whether the size is at fault.
    let small = host.plot_svg(0, 40.0, 30.0, None);
    assert!(
        small.starts_with(
            r#"{"type": "error", "message": "a page of 14.1 by 10.6 mm is too small for this plot"#
        ),
        "{small}"
    );
    assert!(small.ends_with(r#""of_size": true}"#), "{small}");
    assert!(
        host.plot_svg(0, 0.0, 300.0, None)
            .contains(r#""of_size": true"#)
    );
    assert!(
        host.plot_svg(0, f64::NAN, 300.0, None)
            .contains(r#""type": "error""#)
    );
    assert_eq!(
        host.plot_svg(0, 400.0, 300.0, Some("dark")),
        r#"{"type": "error", "message": "`dark` is no theme of a plot: `journal`, `minimal` or `slide`"}"#
    );
    assert_eq!(
        host.plot_svg(9, 400.0, 300.0, None),
        r#"{"type": "error", "message": "the last run made 4 plots, numbered 0 to 3, and none is numbered 9"}"#
    );
    // No run was made for any of it: nothing more was told.
    assert_eq!(told.len(), events);
}

#[test]
fn a_plot_is_given_as_its_scene() {
    let host = Host::new("demo");
    let (_, done) = run(&host, PLOTS, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    let line = host.plot_scene(0);
    assert!(
        line.starts_with(r#"{"type": "scene", "index": 0, "scene": {"zeff_scene": "#),
        "{}",
        &line[..200]
    );
    // The scene holds what an interface reads a mark by: the scales with their limits and
    // breaks, and the values of the layer in the units of the data.
    for part in [
        r#""scales": ["#,
        r#""id": "x""#,
        r#""limits": ["#,
        r#""breaks": ["#,
        r#""panels": ["#,
        r#""layers": ["#,
    ] {
        assert!(
            line.contains(part),
            "{part} in {}",
            &line[..line.len().min(600)]
        );
    }
    assert!(line.ends_with("}}"));
    // The plot of two layers has two.
    assert!(
        host.plot_scene(2).matches(r#""mark": "#).count() >= 2,
        "{}",
        host.plot_scene(2)
    );
    assert_eq!(
        host.plot_scene(7),
        r#"{"type": "error", "message": "the last run made no plot numbered 7"}"#
    );
}

// ----- each statement of a run (open decision #175, C5; release 0.9.0, step 26a) -----

#[test]
fn the_outcome_of_a_run_says_of_each_statement_what_its_result_is_and_what_it_depends_on() {
    let host = Host::new("demo");
    let text = "seed(5)\nlet d = frame(x = [1.0, 2, 3, 4], y = [2.0, 1, 4, 3])\nlet some = d.sample(n: 2)\nlet fit = d.reg(y ~ x)\nprint(some.n_rows())\n";
    let (_, done) = run(&host, text, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    let from = done.find("\"statements\": [").expect("the statements");
    let to = done.find("], \"plots\"").expect("the plots follow");
    let statements = &done[from..to];
    for part in [
        r#"{"index": 1, "line": 1, "end_line": 1, "ran": true, "name": null, "type": null,"#,
        r#""seeds": true, "draws": false,"#,
        r#"{"index": 2, "line": 2, "end_line": 2, "ran": true, "name": "d", "type": "Frame", "rows": 4, "columns": 2, "length": null, "n": null,"#,
        r#"{"index": 3, "line": 3, "end_line": 3, "ran": true, "name": "some", "type": "Frame", "rows": 2, "columns": 2,"#,
        r#""reads": [2], "calls": [], "seeds": false, "draws": true, "reads_file": false, "writes_file": false, "drew": true, "error": null}"#,
        r#"{"index": 4, "line": 4, "end_line": 4, "ran": true, "name": "fit", "type": "RegFit", "rows": null, "columns": null, "length": null, "n": 4,"#,
        r#"{"index": 5, "line": 5, "end_line": 5, "ran": true, "name": null, "type": null, "rows": null, "columns": null, "length": null, "n": null, "stdout": "2\n","#,
        r#""reads": [3], "calls": []"#,
    ] {
        assert!(statements.contains(part), "{part}\n in {statements}");
    }
    // A project whose file cannot be read has no statement to tell of.
    let broken = Host::new("demo");
    broken.add_file("zeff.toml", b"not toml [");
    let (_, done) = run(&broken, "print(1)\n", None, never);
    assert!(
        done.contains(r#""statements": [], "plots": [], "frames": []}"#),
        "{done}"
    );
}

// ----- an error's chain as fields (open decision #175, C22; release 0.9.0, step 27) -----

#[test]
fn the_statement_that_stopped_a_run_carries_its_error_with_the_chain_as_fields() {
    let host = Host::new("demo");
    let text = "/// Half of a positive number.\nfn half(x: Float) -> Float {\n    assert(x > 0, \"a positive number is needed\")\n    x / 2\n}\nlet two = half(4.0)\nlet halves = [for x in [3.0, -1.0, 5.0] { half(x) }]\nprint(\"never\")\n";
    let (_, done) = run(&host, text, None, never);
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    // The message is the one a run prints, with its one line for the call.
    assert!(
        done.contains(r#"= in: `half(x: -1)`, called at scripts/main.zeff:7:43"#),
        "{done}"
    );
    // The first statement ran and has no error; the second stopped the run: its record has
    // the call with what it was given and the pass with what the loop variable held; the
    // third did not run.
    for part in [
        r#"{"index": 1, "line": 6, "end_line": 6, "ran": true, "name": "two", "type": "Float","#,
        r#""drew": false, "error": null}, {"index": 2, "line": 7, "end_line": 7, "ran": true, "name": "halves", "type": null,"#,
        r#""error": {"code": "Z0001", "severity": "error", "message": "a positive number is needed", "file": "scripts/main.zeff", "span": {"line": 3, "column": 5, "end_line": 3, "end_column": 11}"#,
        r#""trace": ["`half(x: -1)`, called at scripts/main.zeff:7:43"], "chain": [{"kind": "call", "function": "half", "given": [{"name": "x", "value": "-1"}], "file": "scripts/main.zeff", "line": 7, "column": 43}, {"kind": "pass", "pass": 2, "of": 3, "variable": "x", "value": "-1", "file": "scripts/main.zeff", "line": 7, "column": 19}], "mark": null, "fixes": []}}, {"index": 3, "line": 8, "end_line": 8, "ran": false,"#,
    ] {
        assert!(done.contains(part), "{part}\n in {done}");
    }
}

// ----- a statement's print at a width (open decision #175, C6; release 0.9.0, step 26b) -----

/// Statements of the kinds the specification's sample script is made of, which this release
/// does not run whole: frames printed by `print`, as the value of a statement, beside other
/// values and inside a loop; a table; a fit and a test; text and numbers; a `let`, which
/// prints nothing; and a plot.
const PRINTS: &str = "let w = dataset(wages)\nprint(w.head(4))\nw.head(3)\nprint(\"the first rows\", w.head(2), \"and their count\", 2)\nfor k in 2015..2016 { print(w.where(year == k).head(2)) }\nprint(w.describe())\nlet fit = w.reg(log(wage) ~ educ + exper + female + industry)\nprint(fit)\nfit.coef()\nprint(w.t_test(wage, group: female))\nprint(\"a line of text that is longer than forty characters and is never wrapped\")\nprint(1, 2.5, \"three\")\nlet silent = w.n_rows()\nw.scatter(educ, wage)\nprint(w.correlation([wage, educ, exper, age, hours]))\n";

/// What each statement of `text` prints in a run at `width`, by the engine itself.
fn printed_in_a_run_at(text: &str, width: usize) -> Vec<String> {
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::{RunOptions, run_script_with};
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"demo\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = Project::open(&fs, std::path::Path::new("/p")).expect("the project opens");
    let options = RunOptions {
        width,
        ..RunOptions::default()
    };
    let outcome = run_script_with(&fs, &project, "scripts/main.zeff", text, &options);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    outcome
        .output
        .statements
        .iter()
        .map(|statement| statement.stdout.clone())
        .collect()
}

/// The text of a line of JSON of type `printed`.
fn stdout_of(line: &str) -> String {
    assert!(line.starts_with(r#"{"type": "printed", "#), "{line}");
    let from = line.find("\"stdout\": \"").expect("its text") + 11;
    let mut text = String::new();
    let mut chars = line[from..line.len() - 2].chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            text.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => text.push('\n'),
            Some('t') => text.push('\t'),
            Some('u') => {
                let code: String = chars.by_ref().take(4).collect();
                text.push(char::from_u32(u32::from_str_radix(&code, 16).unwrap()).unwrap());
            }
            Some(other) => text.push(other),
            None => {}
        }
    }
    text
}

/// The finish line of the step: what the host gives for a statement at a width, with no new
/// run, is what `zeff run --width n` prints for that statement, character for character.
#[test]
fn a_statements_print_at_a_width_is_what_a_run_at_that_width_prints() {
    let host = Host::new("demo");
    let (told, done) = run(&host, PRINTS, None, never);
    assert!(done.contains(r#""exit_code": 0"#), "{done}");
    let events = told.len();
    let statements = printed_in_a_run_at(PRINTS, 118).len();
    assert_eq!(statements, 15);
    let mut wrapped_somewhere = 0;
    for width in [40, 55, 72, 90, 118, 160, 400] {
        let fresh = printed_in_a_run_at(PRINTS, width);
        for (at, expected) in fresh.iter().enumerate() {
            let line = host.print_at(at + 1, width);
            assert!(
                line.starts_with(&format!(
                    r#"{{"type": "printed", "statement": {}, "width": {width}, "stdout": "#,
                    at + 1
                )),
                "{}",
                &line[..line.len().min(120)]
            );
            assert_eq!(
                &stdout_of(&line),
                expected,
                "statement {} at width {width}",
                at + 1
            );
        }
        // The widths differ in what they print: the test is not of one text seven times.
        if fresh != printed_in_a_run_at(PRINTS, 118) {
            wrapped_somewhere += 1;
        }
        // The rows of a frame keep to the width, in blocks of columns; its title is one line.
        for line in fresh[1].lines().filter(|line| line.starts_with("  ")) {
            assert!(line.chars().count() <= width, "at width {width}: {line}");
        }
    }
    assert!(
        wrapped_somewhere >= 4,
        "only {wrapped_somewhere} widths differ from the default"
    );
    // A `let` printed nothing at any width, and the text of a line is the same at every width.
    assert_eq!(stdout_of(&host.print_at(13, 40)), "");
    assert_eq!(
        stdout_of(&host.print_at(11, 40)),
        stdout_of(&host.print_at(11, 400))
    );
    // No run was made for any of it.
    assert_eq!(told.len(), events);
}

#[test]
fn a_print_that_cannot_be_given_is_said() {
    let host = Host::new("demo");
    assert_eq!(
        host.print_at(1, 80),
        r#"{"type": "error", "message": "the last run ran no statement, and none is numbered 1"}"#
    );
    let (_, done) = run(
        &host,
        "let a = 1\nprint(a)\nassert(a > 5, \"too small\")\nprint(\"never\")\n",
        None,
        never,
    );
    assert!(done.contains(r#""exit_code": 1"#), "{done}");
    // The statements that ran, the one that stopped the run among them, have a print; the
    // one that did not run has none.
    assert_eq!(stdout_of(&host.print_at(2, 80)), "1\n");
    assert_eq!(stdout_of(&host.print_at(3, 80)), "");
    assert_eq!(
        host.print_at(4, 80),
        r#"{"type": "error", "message": "the last run ran 3 statements, numbered 1 to 3, and none is numbered 4"}"#
    );
    assert_eq!(
        host.print_at(0, 80),
        host.print_at(4, 80).replace("numbered 4", "numbered 0")
    );
    assert_eq!(
        host.print_at(2, 39),
        r#"{"type": "error", "message": "a print is at least 40 characters wide, and 39 was asked for"}"#
    );
}
