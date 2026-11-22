//! A record of each run (open decision #206; release 0.9.0, step 22b): the calls a script
//! makes read from its checked form, the record written and read back, and kept in the
//! project's folder of runs under its day and time.

use std::io;
use std::path::{Path, PathBuf};
use zeff_engine::VERSION;
use zeff_engine::fs::{FileInfo, FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::runs::{RunRecord, day_and_time, keep, kept};

/// The files of a project in memory, with a clock that stands at one reading.
struct Clocked {
    files: MemoryFileSystem,
    now: Option<i64>,
}

impl FileSystem for Clocked {
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
        self.now
    }
}

/// 22 November 2026, 14:03:22, as the clock gives it: microseconds since 1970.
const A_SUNDAY: i64 = 1_795_356_202_000_000;

const HELPERS: &str = "/// The share of union members.\n/// data: The frame.\n/// returns: The share.\n/// > dataset(wages).union_share()\n/// 0.1\nfn union_share(data: Frame) -> Float {\n    data.mean(union, weights: hours)\n}\n";

const SCRIPT: &str = "use functions::helpers\nlet w = dataset(wages)\nlet fit = w.reg(log(wage) ~ educ + exper, weights: hours, se: robust)\nlet plain = w.reg(wage ~ educ)\nlet opts = {cluster: industry}\nlet spread = w.reg(wage ~ educ, ..opts)\nif false { print(w.correlation([wage, educ])) }\nprint(w.union_share())\n";

fn project(now: Option<i64>) -> (Clocked, Project) {
    let files = MemoryFileSystem::new();
    files.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    files.add_file("/p/scripts/main.zeff", SCRIPT);
    files.add_file("/p/functions/helpers.zeff", HELPERS);
    let fs = Clocked { files, now };
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    (fs, project)
}

fn record(fs: &Clocked, project: &Project, exit_code: i32) -> RunRecord {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    RunRecord::of(
        fs,
        project,
        "scripts/main.zeff",
        SCRIPT,
        exit_code,
        "source",
        registry,
    )
}

#[test]
fn the_day_and_the_time_of_a_clocks_reading() {
    for (micros, day, time) in [
        (0, "1970-01-01", "00:00:00"),
        (A_SUNDAY, "2026-11-22", "14:03:22"),
        (A_SUNDAY + 999_999, "2026-11-22", "14:03:22"),
        (1_709_251_199_000_000, "2024-02-29", "23:59:59"),
        (1_709_251_200_000_000, "2024-03-01", "00:00:00"),
        (951_782_400_000_000, "2000-02-29", "00:00:00"),
        (4_102_444_799_000_000, "2099-12-31", "23:59:59"),
        (-1_000_000, "1969-12-31", "23:59:59"),
    ] {
        assert_eq!(
            day_and_time(micros),
            (day.to_string(), time.to_string()),
            "{micros}"
        );
    }
}

#[test]
fn the_record_of_a_run_names_the_calls_of_the_script_and_of_its_functions() {
    let (fs, project) = project(Some(A_SUNDAY));
    let made = record(&fs, &project, 0);
    assert_eq!(
        (
            made.engine.as_str(),
            made.build.as_str(),
            made.script.as_str(),
            made.exit_code
        ),
        (VERSION, "source", "scripts/main.zeff", 0)
    );
    assert_eq!(
        (made.on.as_deref(), made.at.as_deref()),
        (Some("2026-11-22"), Some("14:03:22"))
    );
    let call = |function: &str, options: &[&str]| {
        made.calls
            .iter()
            .any(|made| made.function == function && made.options == options)
    };
    // `reg` three ways: with two options written, with none, and with one a spread fills.
    assert!(call("reg", &["se", "weights"]), "{:?}", made.calls);
    assert!(call("reg", &[]), "{:?}", made.calls);
    assert!(call("reg", &["cluster"]), "{:?}", made.calls);
    // A call in a branch that is not taken is a call of the script.
    assert!(call("correlation", &[]), "{:?}", made.calls);
    // A call in a function of the project.
    assert!(call("mean", &["weights"]), "{:?}", made.calls);
    assert!(
        call("dataset", &[]) && call("print", &[]),
        "{:?}",
        made.calls
    );
    let packages: Vec<&str> = made
        .calls
        .iter()
        .filter(|made| made.function == "reg")
        .map(|made| made.package.as_str())
        .collect();
    assert_eq!(packages, ["regress"; 3]);
    insta::assert_snapshot!(made.text().replace(VERSION, "<version>"));
}

#[test]
fn a_record_is_read_back_as_it_was_written() {
    let (fs, project) = project(Some(A_SUNDAY));
    let made = record(&fs, &project, 1);
    assert_eq!(RunRecord::read(&made.text()), Some(made.clone()));
    // A run without a clock has no day; a script's name may hold anything.
    let (fs, project) = project_named();
    let odd = RunRecord {
        script: "scripts/a \"quoted\" \\ name\t.zeff".to_string(),
        ..record(&fs, &project, 3)
    };
    assert_eq!((odd.on.as_deref(), odd.at.as_deref()), (None, None));
    assert_eq!(RunRecord::read(&odd.text()), Some(odd));
    let none = RunRecord {
        engine: "0.9.1".to_string(),
        build: "official".to_string(),
        on: None,
        at: None,
        script: "scripts/empty.zeff".to_string(),
        exit_code: 0,
        calls: Vec::new(),
    };
    assert_eq!(RunRecord::read(&none.text()), Some(none));
    for not in [
        "",
        "format = 1\nengine = \"0.9.1\"\n",
        "format = 0\nengine = \"0.9.1\"\nbuild = \"source\"\nscript = \"s\"\n",
        "format = 0\nengine = 9\nbuild = \"source\"\nscript = \"s\"\nexit_code = 0\n",
        "not a record",
    ] {
        assert_eq!(RunRecord::read(not), None, "{not}");
    }
}

/// The same project on a file system without a clock.
fn project_named() -> (Clocked, Project) {
    project(None)
}

#[test]
fn records_are_kept_by_their_day_and_time_and_read_in_order() {
    let (fs, project) = project(Some(A_SUNDAY));
    let first = record(&fs, &project, 0);
    assert_eq!(
        keep(&fs, &project.root, &first).unwrap(),
        PathBuf::from("/p/.zeff/runs/20261122-140322.toml")
    );
    // A second run within the same second, and a third.
    let second = RunRecord {
        exit_code: 1,
        ..first.clone()
    };
    assert_eq!(
        keep(&fs, &project.root, &second).unwrap(),
        PathBuf::from("/p/.zeff/runs/20261122-140322-2.toml")
    );
    assert!(
        keep(&fs, &project.root, &first)
            .unwrap()
            .ends_with("20261122-140322-3.toml")
    );
    // A run the next day; a run without a clock; a file that is no record.
    let later = RunRecord {
        on: Some("2026-11-23".to_string()),
        at: Some("09:00:00".to_string()),
        ..first.clone()
    };
    keep(&fs, &project.root, &later).unwrap();
    let undated = RunRecord {
        on: None,
        at: None,
        ..first.clone()
    };
    assert!(
        keep(&fs, &project.root, &undated)
            .unwrap()
            .ends_with("undated.toml")
    );
    assert!(
        keep(&fs, &project.root, &undated)
            .unwrap()
            .ends_with("undated-2.toml")
    );
    fs.write(Path::new("/p/.zeff/runs/notes.toml"), b"mine = true\n")
        .unwrap();
    fs.write(Path::new("/p/.zeff/runs/readme.txt"), b"hello\n")
        .unwrap();
    let found = kept(&fs, &project.root);
    let names: Vec<&str> = found.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "20261122-140322-2.toml",
            "20261122-140322-3.toml",
            "20261122-140322.toml",
            "20261123-090000.toml",
            "undated-2.toml",
            "undated.toml",
        ]
    );
    assert_eq!(found[0].1, second);
    assert_eq!(found[3].1, later);
    // A project with no run has none.
    let (fresh, fresh_project) = project_named();
    assert_eq!(
        kept(&fresh, &fresh_project.root),
        Vec::<(String, RunRecord)>::new()
    );
}
