//! `timer(expr)` (catalogue row 64; Language Reference, section 3.7, "Effects"; release 0.9.0,
//! step 16): the expression's value given back and the time it took noted, from the clock the
//! run is given. A clock that moves by a known amount at each reading shows the time noted; a
//! fixed clock, as every test and conformance case has, notes nought; a function may not call
//! it (Z0110).

use std::cell::Cell;
use std::io;
use std::path::{Path, PathBuf};
use zeff_engine::fs::{FileInfo, FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::run::run_script;
use zeff_testkit::run;

/// The files of a project in memory, with a clock that moves on by `step` microseconds each
/// time it is read.
struct Ticking {
    files: MemoryFileSystem,
    readings: Cell<i64>,
    step: i64,
}

impl FileSystem for Ticking {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files.read(path)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileInfo> {
        self.files.metadata(path)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        self.files.list(dir)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.files.is_dir(path)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.files.canonicalize(path)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.files.create_dir(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.files.write(path, bytes)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.files.remove_dir_all(path)
    }

    fn now(&self) -> Option<i64> {
        let reading = self.readings.get();
        self.readings.set(reading + 1);
        Some(1_795_000_000_000_000 + reading * self.step)
    }
}

/// What a script prints and says in a project whose clock moves by `step` microseconds a
/// reading.
fn timed(script: &str, step: i64) -> (i32, String, String) {
    let files = MemoryFileSystem::new();
    files.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    files.add_file("/p/scripts/main.zeff", script);
    let fs = Ticking {
        files,
        readings: Cell::new(0),
        step,
    };
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn the_time_between_two_readings_of_the_clock_is_noted() {
    // Two readings, a second and a half apart.
    let (exit, stdout, stderr) = timed("timer(sqrt(2))\n", 1_500_000);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "1.41421\n");
    assert_eq!(stderr, "note: elapsed time 1.500 s (line 1)\n");
    // The value is the expression's: bound, it is used as the expression would be.
    let (exit, stdout, stderr) = timed("let x = timer(2 + 3)\nx * 2\n", 250_000);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "10\n");
    assert_eq!(stderr, "note: elapsed time 0.250 s (line 1)\n");
    // Minutes, and hours.
    let (_, _, stderr) = timed("timer(1)\n", 123_400_000);
    assert_eq!(stderr, "note: elapsed time 2 min 3.4 s (line 1)\n");
    let (_, _, stderr) = timed("timer(1)\n", 3_723_000_000);
    assert_eq!(stderr, "note: elapsed time 1 h 2 min 3 s (line 1)\n");
}

#[test]
fn a_timer_inside_a_timer_is_timed_by_the_outer_one() {
    // The outer timer reads the clock first and last: four readings, three steps between its
    // two; the inner one's two are one step apart.
    let (exit, stdout, stderr) = timed("timer(timer(1) + 1)\n", 1_000_000);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "2\n");
    assert_eq!(
        stderr,
        "note: elapsed time 1.000 s (line 1)\nnote: elapsed time 3.000 s (line 1)\n"
    );
}

#[test]
fn an_expression_that_fails_stops_the_script_and_notes_no_time() {
    let (exit, stdout, stderr) = timed("timer(error(\"no\"))\n1 + 1\n", 1_000_000);
    assert_eq!(exit, 1);
    assert_eq!(stdout, "");
    assert!(!stderr.contains("elapsed time"), "{stderr}");
    assert!(stderr.contains("no"), "{stderr}");
    // Kept by `attempt`, the failure is a value, and the time of the attempt is noted.
    let (exit, _, stderr) = timed("let r = timer(attempt([1, 2][5]))\n", 2_000_000);
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stderr.contains("note: elapsed time 2.000 s (line 1)"),
        "{stderr}"
    );
}

#[test]
fn every_form_and_every_refusal_on_the_fixed_clock() {
    let scripts = [
        // On the fixed clock of the tests every time is nought: a fit bound, a value printed,
        // a call that gives nothing, a block of statements each timed.
        "let wages = dataset(wages)\nlet m = timer(wages.reg(log(wage) ~ educ + exper))\nm.n_obs()\ntimer(sqrt(2))\ntimer(m.export(\"output/timed.tex\"))\nfor k in [1, 2] {\n    timer(k * 2)\n}\ntype_of(timer(1.5))\n",
        // The refusals.
        "/// Twice a number.\nfn twice(x: Float) -> Float {\n    timer(x * 2)\n}\ntwice(2.0)\n",
        "timer()\n",
        "timer(1, 2)\n",
        "timer(expr: 1)\n",
        "timer(undefined_name)\n",
        // Written as a method, `x.timer()` is `timer(x)`; with a further argument it is too many.
        "let x = 3\nx.timer()\n(x + 1).timer(2)\n",
    ];
    let mut out = String::new();
    for script in scripts {
        // A clock that does not move: step nought.
        let (exit, stdout, stderr) = timed(script, 0);
        out.push_str(&format!(
            "=== {} ===\n{stdout}--- messages ---\n{stderr}[exit {exit}]\n\n",
            script.trim_end()
        ));
    }
    insta::assert_snapshot!(out);
}

#[test]
fn a_run_with_no_clock_cannot_time() {
    // The engine never reads a clock of its own: the host gives the run one, as it does for
    // `today()` and `now()`, and a run without one says so as they do.
    let outcome = run("timer(1)\n");
    assert_eq!(outcome.exit_code, 2, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stderr
            .contains("`timer` was called in a run that has no clock"),
        "{}",
        outcome.output.stderr
    );
}
