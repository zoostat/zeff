//! `zeff report-bug`, shrinking (release 0.8.0, step 20b): rows removed by halves while the
//! failure persists, then statements, within a count of trial runs; made-up values with
//! `--synthetic`; and what the report says of each.

use std::path::Path;
use zeff_engine::bug_report::{Asked, Kind, Report, TRIALS, build, write};
use zeff_engine::fs::{FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::run::{RunOptions, run_script};

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn project() -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs
}

fn asked<'a>(text: &'a str) -> Asked<'a> {
    Asked {
        script: "scripts/main.zeff",
        text,
        line: None,
        reference: None,
        include_data: true,
        synthetic: false,
        trials: TRIALS,
        platform: "the test's machine",
        options: RunOptions::default(),
    }
}

fn report(fs: &MemoryFileSystem, asked: &Asked<'_>) -> Report {
    let project = Project::open(fs, Path::new("/p")).unwrap();
    build(fs, &project, asked).unwrap()
}

/// The text of a file the report brings.
fn brought<'r>(report: &'r Report, name: &str) -> &'r str {
    let bytes = report
        .files
        .iter()
        .find(|(file, _)| file == name)
        .and_then(|(_, bytes)| bytes.as_deref())
        .unwrap_or_else(|| panic!("{name} is not brought by the report"));
    std::str::from_utf8(bytes).unwrap()
}

/// The statements a report keeps, as it writes them: the indented lines under its heading.
fn statements_of(report: &Report) -> String {
    let after = report.text.split("## The statements\n").nth(1).unwrap();
    let mut out = String::new();
    let mut started = false;
    for line in after.lines() {
        if let Some(statement) = line.strip_prefix("    ") {
            started = true;
            out.push_str(statement);
            out.push('\n');
        } else if started && !line.is_empty() {
            break;
        } else if started {
            out.push('\n');
        }
    }
    out
}

/// The first line of the error a run stopped with: its code and its message.
fn error_line(stderr: &str) -> String {
    stderr
        .lines()
        .rev()
        .find(|line| line.starts_with("error["))
        .unwrap_or("")
        .to_string()
}

/// A reproducer run where its reader would run it: a new project that holds the statements
/// the report keeps and the files it brings, and nothing else.
fn reproduced(report: &Report) -> (i32, String) {
    let fs = project();
    for (name, bytes) in &report.files {
        fs.add_file(format!("/p/{name}"), bytes.as_deref().unwrap());
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let outcome = run_script(&fs, &project, "scripts/main.zeff", &statements_of(report));
    (outcome.exit_code, error_line(&outcome.output.stderr))
}

/// A planted failure on a million rows: one value among them that a check of the script
/// refuses. The file the report brings has under a hundred lines (it has two: the names and
/// the row at fault), found in about forty trial runs, and the project's file is untouched.
#[test]
fn a_planted_failure_on_a_million_rows_shrinks_to_under_a_hundred() {
    let fs = project();
    let rows = 1_000_000;
    let at = 700_001;
    let mut data = String::with_capacity(12 * rows);
    data.push_str("id,x\n");
    for id in 1..=rows {
        let x = if id == at { 5000 } else { id % 97 };
        data.push_str(&format!("{id},{x}\n"));
    }
    fs.add_file("/p/data/big.csv", &data);
    let script = "let big = import(\"data/big.csv\")\nprint(big.n_rows())\nassert(big.max(x) < 1000, \"a value of x is 1000 or more\")\n";
    let report = report(&fs, &asked(script));
    assert_eq!(report.kind, Kind::Error);
    assert_eq!(report.code.as_deref(), Some("Z0001"));
    let file = &report.shrunk.files[0];
    assert_eq!((file.name.as_str(), file.lines), ("data/big.csv", rows + 1));
    assert!(file.kept < 100, "{} lines are kept", file.kept);
    assert_eq!(brought(&report, "data/big.csv"), "id,x\n700001,5000\n");
    assert!(
        report.shrunk.trials <= 60 && !report.shrunk.stopped,
        "{} trials",
        report.shrunk.trials
    );
    assert!(report.text.contains(
        "- `data/big.csv`: 2 of its 1,000,001 lines are kept, 17 bytes; the failure persists without the others"
    ));
    assert_eq!((report.kept, report.of), (2, 3));
    assert_eq!(
        reproduced(&report),
        (1, "error[Z0001]: a value of x is 1000 or more".to_string())
    );
    assert_eq!(
        fs.read(Path::new("/p/data/big.csv")).unwrap().len(),
        data.len()
    );
}

/// The fault of `mad` on a frame with a variable called `centre`, corrected at 0.7.0: its body
/// bound that name and read it inside a method of the frame, where the frame's variable came
/// first. The engine no longer has the fault, so its body is written here at the top level of
/// a script, where that rule still holds (and warns, W0102), beside the definition computed
/// without the name; the check that the two agree fails, as the fault made it fail. On five
/// thousand rows the report keeps a reproducer of a few: the names and a row or two.
#[test]
fn the_fault_of_mad_with_a_variable_called_centre_shrinks_to_a_few_rows() {
    let fs = project();
    let mut stream = zeff_numeric::random::Stream::new(20261119, 1, 0);
    let mut data = String::from("site,x,centre\n");
    for row in 0..5000 {
        let x = (1000.0 * stream.uniform()).round() / 10.0;
        let centre = (100.0 * stream.uniform()).round() / 10.0;
        data.push_str(&format!("{},{x},{centre}\n", row % 12 + 1));
    }
    fs.add_file("/p/data/measures.csv", &data);
    let script = "let d = import(\"data/measures.csv\")
d.summary([x])
// The body `mad` had until 0.7.0, written out: `centre` is read as the frame's variable.
let centre = d.median(x)
let scaled = 1.4826 * d.add(deviation = abs(x - centre)).median(deviation)
// Its definition, without the name.
let middle = d.median(x)
let by_definition = 1.4826 * d.add(deviation = abs(x - middle)).median(deviation)
print(by_definition)
assert(scaled == by_definition, \"mad differs from its definition\")
";
    let report = report(&fs, &asked(script));
    assert_eq!(report.kind, Kind::Error);
    assert_eq!(report.code.as_deref(), Some("Z0001"));
    assert!(report.reduced);
    let file = &report.shrunk.files[0];
    assert_eq!(file.lines, 5001);
    assert!(file.kept <= 4, "{} lines are kept", file.kept);
    assert!(!report.shrunk.stopped);
    // The trials leave out even the `let` of the name: without it `centre` can only be the
    // frame's variable, and the check fails in the same way.
    assert_eq!(report.shrunk.statements, [4]);
    assert_eq!((report.kept, report.of), (5, 8));
    assert_eq!(
        reproduced(&report),
        (
            1,
            "error[Z0001]: mad differs from its definition".to_string()
        )
    );
    // The report holds no row; its text is stored.
    insta::assert_snapshot!("report_of_the_fault_of_mad", report.text);
}

const STAFF: &str = "worker;site;wage;joined;grade
101;\"Leeds\";2310,50;2019-03-01;A
102;\"Leeds\";1980,00;2020-07-15;B
103;\"York\";2755,25;2018-11-30;A
104;\"York\";;2021-01-04;C
105;\"Hull\";2120,00;2022-05-20;B
106;\"Hull\";99999,00;2017-09-09;A
107;\"Leeds\";2240,75;2019-12-12;C
";

/// `--synthetic`: after the rows, the values. Every variable the failure does not read is
/// made up, a value by one of its kind and the same by the same; the variable at fault keeps
/// the file's own values, and the report says which is which.
#[test]
fn made_up_values_stand_for_the_users_own_where_the_failure_allows() {
    let fs = project();
    fs.add_file("/p/data/staff.csv", STAFF);
    let script = "let staff = import(\"data/staff.csv\", delim: \";\", decimal: \",\")
print(staff.n_rows())
assert(staff.max(wage) < 50000, \"a wage of 50,000 or more\")
";
    let made_up = Asked {
        synthetic: true,
        include_data: false,
        ..asked(script)
    };
    let report = report(&fs, &made_up);
    assert_eq!(report.code.as_deref(), Some("Z0001"));
    assert_eq!(
        brought(&report, "data/staff.csv"),
        "worker;site;wage;joined;grade\n1;\"v1\";99999,00;v1;v1\n"
    );
    let file = &report.shrunk.files[0];
    assert_eq!((file.lines, file.kept), (8, 2));
    assert_eq!(file.made_up, ["worker", "site", "joined", "grade"]);
    assert_eq!(file.real, ["wage"]);
    for own in ["106", "Hull", "2017-09-09"] {
        assert!(!brought(&report, "data/staff.csv").contains(own), "{own}");
    }
    assert_eq!(
        reproduced(&report),
        (1, "error[Z0001]: a wage of 50,000 or more".to_string())
    );
    insta::assert_snapshot!("report_with_made_up_values", report.text);
    // Written beside the report, as `--include-data` writes its files.
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let written = write(&fs, &project, &report).unwrap();
    assert_eq!(written.len(), 2);
    assert!(written[1].ends_with("-data/data/staff.csv"));
    // A failure that reads no value, and needs three rows whatever they hold: every value
    // is made up.
    let any = Asked {
        synthetic: true,
        ..asked(
            "let staff = import(\"data/staff.csv\", delim: \";\", decimal: \",\")\nassert(staff.n_rows() < 3, \"three rows or more\")\n",
        )
    };
    let report = self::report(&fs, &any);
    assert_eq!(report.shrunk.files[0].kept, 4);
    assert!(report.shrunk.files[0].real.is_empty());
    assert!(report.text.contains("Every value in it is made up"));
    assert_eq!(
        brought(&report, "data/staff.csv"),
        "worker;site;wage;joined;grade\n1;\"v1\";v1;v1;v1\n2;\"v1\";v2;v2;v2\n3;\"v2\";v3;v3;v3\n"
    );
    // An error the check finds needs no row at all: the file is shrunk to its first line.
    let checked = Asked {
        synthetic: true,
        ..asked(
            "let staff = import(\"data/staff.csv\", delim: \";\", decimal: \",\")\nstaff.mean(wagee)\n",
        )
    };
    let report = self::report(&fs, &checked);
    assert_eq!(report.code.as_deref(), Some("Z0101"));
    assert_eq!(
        brought(&report, "data/staff.csv"),
        "worker;site;wage;joined;grade\n"
    );
}

/// The trials are counted, and the shrinking stops at its limit: the report then says that a
/// smaller reproducer may exist, and what it brings still fails in the same way. The same
/// limit gives the same report, twice.
#[test]
fn the_shrinking_stops_at_its_limit_and_says_so() {
    let fs = project();
    let mut data = String::from("id,x\n");
    for id in 1..=4000 {
        data.push_str(&format!(
            "{id},{}\n",
            if id % 400 == 0 { 5000 } else { id % 97 }
        ));
    }
    fs.add_file("/p/data/big.csv", &data);
    // The failure needs ten rows spread over the file: every row at fault.
    let script = "let big = import(\"data/big.csv\")\nassert(big.where(x >= 1000).n_rows() < 10, \"ten values of x or more are 1000 or more\")\n";
    let limited = Asked {
        trials: 40,
        ..asked(script)
    };
    let report = report(&fs, &limited);
    assert!(report.shrunk.stopped);
    assert_eq!(report.shrunk.trials, 40);
    let kept = report.shrunk.files[0].kept;
    assert!(kept > 11 && kept < 4001, "{kept} lines are kept");
    assert!(report.text.contains(
        "The shrinking stopped at its limit of 40 trial runs: a smaller reproducer may exist."
    ));
    assert_eq!(
        reproduced(&report),
        (
            1,
            "error[Z0001]: ten values of x or more are 1000 or more".to_string()
        )
    );
    assert_eq!(self::report(&fs, &limited), report);
    // Without the limit: the ten rows and the names, and nothing else.
    let whole = self::report(&fs, &asked(script));
    assert!(!whole.shrunk.stopped);
    assert_eq!(whole.shrunk.files[0].kept, 11);
    assert!(whole.shrunk.trials <= TRIALS);
}

/// A result believed wrong keeps every row, which its reference was computed on; its
/// statements are still those it depends on.
#[test]
fn a_result_believed_wrong_keeps_its_rows() {
    let fs = project();
    fs.add_file("/p/data/staff.csv", STAFF);
    let script = "let staff = import(\"data/staff.csv\", delim: \";\", decimal: \",\")\nprint(\"between\")\nstaff.mean(wage)\n";
    let wrong = Asked {
        line: Some(3),
        reference: Some(("r.txt", "[1] 18567.75\n")),
        synthetic: true,
        ..asked(script)
    };
    let report = report(&fs, &wrong);
    assert_eq!(report.kind, Kind::WrongResult);
    assert!(report.shrunk.files.is_empty());
    assert_eq!(brought(&report, "data/staff.csv"), STAFF);
    assert_eq!((report.kept, report.of), (2, 3));
    assert!(
        report
            .text
            .contains("written beside this report as they are"),
        "{}",
        report.text
    );
}

/// Over planted failures drawn at random (their kind, the size of the file, the place of the
/// row at fault), the reproducer a report brings fails with the same code and the same
/// message as the script it was made from, in a project that holds nothing else; with real
/// values and with made-up ones. Run longer nightly.
#[test]
fn a_shrunk_reproducer_fails_as_the_original_did() {
    let cases = property_cases(6);
    for case in 0..cases {
        let mut stream = zeff_numeric::random::Stream::new(8000 + u64::from(case), 1, 0);
        let mut draw = |below: usize| -> usize { (stream.uniform() * below as f64) as usize };
        let rows = 20 + draw(600);
        let at = 1 + draw(rows);
        let second = 1 + draw(rows);
        let kind = draw(4);
        let mut data = String::from("id,group,x,note\n");
        for id in 1..=rows {
            let x = match kind {
                // One value too large for the check of the script.
                0 if id == at => "7777.5".to_string(),
                // One whole number whose square overflows.
                1 if id == at => "4000000000".to_string(),
                1 => format!("{}", id % 50),
                _ => format!("{}.25", id % 50),
            };
            // A key written twice, for a merge that needs it once.
            let group = if kind == 2 && id == second.max(2) {
                1
            } else {
                id
            };
            let note = if kind == 3 && id == at {
                "gone"
            } else {
                "kept"
            };
            data.push_str(&format!("{id},{group},{x},{note}\n"));
        }
        let fs = project();
        fs.add_file("/p/data/cases.csv", &data);
        let check = match kind {
            0 => "assert(d.max(x) < 1000, \"a value of x is 1000 or more\")",
            1 => "d.add(square = x * x * x).head(2)",
            2 => "frame(group = [1, 2]).merge(d, on: group, relation: one_to_one).head(2)",
            _ => "assert(d.where(note == \"gone\").n_rows() == 0, \"a row is marked gone\")",
        };
        let script = format!(
            "let d = import(\"data/cases.csv\")\nprint(d.n_rows())\nlet unused = d.head(3)\n{check}\n"
        );
        let project = Project::open(&fs, Path::new("/p")).unwrap();
        let original = run_script(&fs, &project, "scripts/main.zeff", &script);
        assert_ne!(
            original.exit_code, 0,
            "case {case}, kind {kind}: nothing failed"
        );
        // The same code and the same message, the numbers in it set apart: a message that
        // names the row at fault names another row once the rows before it are gone.
        let apart = |(code, line): (i32, String)| (code, zeff_engine::shrink::numbers_apart(&line));
        let expected = apart((original.exit_code, error_line(&original.output.stderr)));
        for synthetic in [false, true] {
            let report = report(
                &fs,
                &Asked {
                    synthetic,
                    ..asked(&script)
                },
            );
            assert!(!report.shrunk.stopped, "case {case}");
            let file = &report.shrunk.files[0];
            assert!(
                file.kept <= 4,
                "case {case}, kind {kind}: {} of {} lines are kept",
                file.kept,
                file.lines
            );
            assert_eq!(
                apart(reproduced(&report)),
                expected,
                "case {case}, kind {kind}, synthetic {synthetic}:\n{}",
                report.text
            );
            assert_eq!((report.kept, report.of), (2, 4), "case {case}");
        }
    }
}
