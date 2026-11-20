//! `zeff report-bug`, the report (release 0.8.0, step 20a): a report of each kind of failure,
//! what it keeps of the script and what it leaves out, and what is written.

use std::path::Path;
use zeff_engine::bug_report::{Asked, Kind, NoReport, Report, build, write};
use zeff_engine::fs::{FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::run::RunOptions;

const PAY: &str = "worker,firm,wage,hours,industry
1,10,23.5,40,Retail
2,10,19.0,,Retail
3,20,31.25,38,Health
4,20,27.0,41,Health
5,30,22.0,36,Retail
";

const FIRMS: &str = "firm,size,industry
10,120,Retail
20,45,Health
30,300,Retail
";

const MODULE: &str = "/// Twice a number.
///
/// x: The number.
/// returns: Twice it.
///
/// > twice(2)
/// 4
fn twice(x: Float) -> Float {
    2 * x
}

/// A number plus one.
///
/// x: The number.
/// returns: One more.
///
/// > plus_one(2)
/// 3
fn plus_one(x: Float) -> Float {
    x + 1
}
";

fn project() -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file("/p/data/pay.csv", PAY);
    fs.add_file("/p/data/firms.csv", FIRMS);
    fs.add_file("/p/functions/clean.zeff", MODULE);
    fs
}

fn asked<'a>(text: &'a str) -> Asked<'a> {
    Asked {
        script: "scripts/main.zeff",
        text,
        line: None,
        reference: None,
        include_data: false,
        synthetic: false,
        trials: zeff_engine::bug_report::TRIALS,
        platform: "the test's machine",
        options: RunOptions::default(),
    }
}

fn report(fs: &MemoryFileSystem, asked: &Asked<'_>) -> Result<Report, NoReport> {
    let project = Project::open(fs, Path::new("/p")).unwrap();
    build(fs, &project, asked)
}

/// A script of twelve statements, of which the failing one needs four, a function of a module
/// and the seed set before it: the merge of section 6 without suffixes, an error of the run.
const MERGE: &str = "use functions::clean
let wages = dataset(wages)
wages.summary([wage, hours])
let pay = import(\"data/pay.csv\")
let firms = import(\"data/firms.csv\")
seed(11)
let few = wages.sample(n: 3)
print(few.n_rows())
let factor = twice(1.5)
let dear = pay.add(wage2 = wage * factor)
// zeff: allow(W0201) the missing hours are known
print(dear.mean(hours))
let joined = dear.merge(firms, on: firm)
joined
";

/// An error of the script: the report names the error as printed, keeps the failing statement
/// and what it depends on, the function it calls and the seed set before it, says what was
/// said before, and gives the schema of the frames and the names of the files, and no value.
#[test]
fn a_report_of_an_error() {
    let fs = project();
    let report = report(&fs, &asked(MERGE)).unwrap();
    assert_eq!(report.kind, Kind::Error);
    assert_eq!(report.code.as_deref(), Some("Z0604"));
    assert!(report.reduced);
    // The graph keeps six statements, the seed set before the failure among them; no trial
    // leaves one of them out.
    assert_eq!((report.kept, report.of), (6, 12));
    assert!(report.shrunk.statements.is_empty());
    assert!(
        report.shrunk.files.is_empty(),
        "no data is written, so no file is shrunk"
    );
    assert!(!report.shrunk.stopped);
    assert!(report.name.starts_with("main-Z0604-") && report.name.ends_with(".md"));
    // No value of the data is in it: not a wage, not a firm's size, not a level.
    for value in ["23.5", "31.25", "120", "Health", "Retail"] {
        assert!(!report.text.contains(value), "{value}:\n{}", report.text);
    }
    // The statements left out are left out.
    for left in [
        "wages.summary",
        "few",
        "print(dear.mean(hours))",
        "plus_one",
    ] {
        assert!(!report.text.contains(left), "{left}:\n{}", report.text);
    }
    assert_eq!(
        report.files,
        [
            ("data/firms.csv".to_string(), None),
            ("data/pay.csv".to_string(), None)
        ]
    );
    insta::assert_snapshot!("report_of_an_error", masked(&report.text));
    // The same failure gives the same report under the same name.
    let again = self::report(&fs, &asked(MERGE)).unwrap();
    assert_eq!(again, report);
    // Written: one file, in `.zeff/reports`, and nothing beside it.
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let written = write(&fs, &project, &report).unwrap();
    assert_eq!(written, [format!(".zeff/reports/{}", report.name)]);
    assert_eq!(
        fs.read_to_string(&Path::new("/p").join(&written[0]))
            .unwrap(),
        report.text
    );
    assert_eq!(fs.list(Path::new("/p/.zeff/reports")).unwrap().len(), 1);
}

/// With `--include-data` the files the kept statements read are written beside the report, as
/// they are; the report says so.
#[test]
fn the_data_comes_only_when_asked_for() {
    let fs = project();
    let with = Asked {
        include_data: true,
        ..asked(MERGE)
    };
    let report = report(&fs, &with).unwrap();
    // Each file is shrunk to the lines the failure needs: its first line and one row, the
    // last, which is what is left when the first half goes at every halving.
    let firms = "firm,size,industry\n30,300,Retail\n";
    let pay = "worker,firm,wage,hours,industry\n5,30,22.0,36,Retail\n";
    assert_eq!(
        report.files,
        [
            (
                "data/firms.csv".to_string(),
                Some(firms.as_bytes().to_vec())
            ),
            ("data/pay.csv".to_string(), Some(pay.as_bytes().to_vec()))
        ]
    );
    let shrunk: Vec<(&str, usize, usize)> = report
        .shrunk
        .files
        .iter()
        .map(|file| (file.name.as_str(), file.lines, file.kept))
        .collect();
    assert_eq!(shrunk, [("data/firms.csv", 4, 2), ("data/pay.csv", 6, 2)]);
    assert!(report.text.contains("`--include-data` asked for\nthem"));
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let written = write(&fs, &project, &report).unwrap();
    let stem = report.name.trim_end_matches(".md");
    assert_eq!(
        written,
        [
            format!(".zeff/reports/{}", report.name),
            format!(".zeff/reports/{stem}-data/data/firms.csv"),
            format!(".zeff/reports/{stem}-data/data/pay.csv"),
        ]
    );
    assert_eq!(
        fs.read_to_string(&Path::new("/p").join(&written[2]))
            .unwrap(),
        pay
    );
    // The project's own files are as they were.
    assert_eq!(
        fs.read_to_string(Path::new("/p/data/pay.csv")).unwrap(),
        PAY
    );
    assert_eq!(
        fs.read_to_string(Path::new("/p/data/firms.csv")).unwrap(),
        FIRMS
    );
    insta::assert_snapshot!("report_with_its_data", masked(&report.text));
}

/// An internal error, planted by the test hook at the line of a statement: the report says it
/// is a bug in Zeff, and keeps what that statement needs.
#[test]
fn a_report_of_an_internal_error() {
    let fs = project();
    let planted = Asked {
        options: RunOptions {
            panic_at_line: Some(10),
            ..RunOptions::default()
        },
        ..asked(MERGE)
    };
    let report = report(&fs, &planted).unwrap();
    assert_eq!(report.kind, Kind::InternalError);
    assert_eq!(report.code.as_deref(), Some("Z9001"));
    assert!(report.reduced);
    assert_eq!((report.kept, report.of), (4, 12));
    assert!(report.name.starts_with("main-Z9001-"));
    insta::assert_snapshot!("report_of_an_internal_error", masked(&report.text));
}

/// A result believed wrong: nothing failed; the statement is named by its line and the
/// reference is given, and both are in the report.
#[test]
fn a_report_of_a_wrong_result_with_its_reference() {
    let fs = project();
    let script = "let pay = import(\"data/pay.csv\")
let firms = import(\"data/firms.csv\")
pay.summary([wage])
let m = pay.reg(wage ~ hours,
                se: robust)
m
";
    let reference =
        "Call: lm(formula = wage ~ hours, data = pay)\n(Intercept)  hours\n    -19.137  1.151\n";
    let wrong = Asked {
        line: Some(6),
        reference: Some(("checks/r_output.txt", reference)),
        ..asked(script)
    };
    let report = report(&fs, &wrong).unwrap();
    assert_eq!(report.kind, Kind::WrongResult);
    assert_eq!(report.code, None);
    assert!(report.reduced);
    assert_eq!((report.kept, report.of), (3, 5));
    assert!(report.name.starts_with("main-wrong-result-"));
    // A line inside a statement of several lines names that statement, by its first line.
    let inside = self::report(
        &fs,
        &Asked {
            line: Some(5),
            reference: Some(("r.txt", reference)),
            ..asked(script)
        },
    )
    .unwrap();
    assert!(
        inside
            .text
            .contains("The statement at line 4 ran without an error")
    );
    assert_eq!((inside.kept, inside.of), (2, 5));
    assert!(report.text.contains("    -19.137  1.151"));
    assert_eq!(report.files, [("data/pay.csv".to_string(), None)]);
    insta::assert_snapshot!("report_of_a_wrong_result", masked(&report.text));
    // What cannot be reported, and why.
    let no = |asked: Asked<'_>| self::report(&fs, &asked).unwrap_err();
    assert_eq!(no(asked(script)), NoReport::NothingFailed);
    assert_eq!(
        no(Asked {
            line: Some(5),
            ..asked(script)
        }),
        NoReport::NoReference
    );
    assert_eq!(
        no(Asked {
            reference: Some(("r.txt", reference)),
            ..asked(script)
        }),
        NoReport::NoLine
    );
    assert_eq!(
        no(Asked {
            line: Some(40),
            reference: Some(("r.txt", reference)),
            ..asked(script)
        }),
        NoReport::NoStatementAt { line: 40 }
    );
    assert_eq!(
        no(Asked {
            line: Some(3),
            reference: Some(("r.txt", reference)),
            ..asked("let pay = import(\"data/pay.csv\")\nprint(nope)\npay\n")
        }),
        NoReport::FailedInstead { line: 3 }
    );
    let mut all = String::new();
    for none in [
        NoReport::NothingFailed,
        NoReport::FailedInstead { line: 3 },
        NoReport::NoStatementAt { line: 40 },
        NoReport::NoReference,
        NoReport::NoLine,
    ] {
        all.push_str(&format!("error: {none}\n\n"));
    }
    insta::assert_snapshot!("no_report", all);
}

/// An error the check finds before anything runs, and a script that cannot be read into
/// statements: each has its report, and the second keeps the script whole.
#[test]
fn a_report_of_an_error_found_before_the_run() {
    let fs = project();
    let script = "let pay = import(\"data/pay.csv\")\nlet firms = import(\"data/firms.csv\")\nprint(firms.n_rows())\npay.mean(wagee)\n";
    let report = self::report(&fs, &asked(script)).unwrap();
    assert_eq!(report.kind, Kind::Error);
    assert_eq!(report.code.as_deref(), Some("Z0101"));
    assert!(report.reduced);
    assert_eq!((report.kept, report.of), (2, 4));
    insta::assert_snapshot!("report_of_an_error_of_the_check", masked(&report.text));
    let unread = self::report(
        &fs,
        &asked("let pay = import(\"data/pay.csv\"\nlet x = (\n"),
    )
    .unwrap();
    assert_eq!(unread.kind, Kind::Error);
    assert!(!unread.reduced);
    insta::assert_snapshot!(
        "report_of_a_script_that_cannot_be_read",
        masked(&unread.text)
    );
}

/// A statement that depends on another through a file, which the graph does not see: the
/// statements the graph gives do not fail when run alone, so every statement up to the failing
/// one is kept, and the report says why.
#[test]
fn a_dependence_through_a_file_keeps_every_statement_up_to_the_failure() {
    let fs = project();
    let script = "let small = frame(x = [1.0, 2.0])
small.save(\"output/t.csv\")
let t = import(\"output/t.csv\")
frame(x = [1.0, 2.0, 3.0]).save(\"output/t.csv\")
print(\"between\")
assert(t.n_rows() == 3, \"three rows\")
print(\"after\")
";
    let report = self::report(&fs, &asked(script)).unwrap();
    assert_eq!(report.kind, Kind::Error);
    assert!(!report.reduced, "{}", report.text);
    // Of the six statements up to the failure, the trials leave out the two that the failure
    // persists without: the second `save` and the `print`.
    assert_eq!((report.kept, report.of), (4, 7));
    assert_eq!(report.shrunk.statements, [4, 5]);
    // Nothing a run of the report wrote is in the project.
    assert!(!fs.is_file(Path::new("/p/output/t.csv")));
    assert!(!report.text.contains("print(\"after\")"));
    insta::assert_snapshot!("report_kept_up_to_the_failure", masked(&report.text));
}

/// The specification's sample script (`docs/spec/zeff_samples.zeff`), which this release does
/// not yet run to its end: its report reads as section 11.1 of the Language Reference
/// describes one. It names the engine's version, edition and platform; it gives the failing
/// statement and only the statements it depends on, a few of the script's seventy-four; it
/// gives what the run said; and it holds the schema of the data and no value of it.
#[test]
fn the_report_of_the_sample_scripts_failure_reads_as_the_specification_describes() {
    let fs = project();
    let script = include_str!("../../../docs/spec/zeff_samples.zeff");
    for (file, bytes) in [
        (
            "data/wages.csv",
            &include_bytes!("../../../samples/first_engine/data/wages.csv")[..],
        ),
        (
            "data/sales_wide.csv",
            include_bytes!("../../../samples/panel/data/sales_wide.csv"),
        ),
        (
            "data/counties.csv",
            include_bytes!("../../../samples/panel/data/counties.csv"),
        ),
        (
            "data/card.csv",
            include_bytes!("../../../tests/reference/data/card.csv"),
        ),
    ] {
        fs.add_file(format!("/p/{file}"), bytes);
    }
    // The module the script uses, which it shows in a comment of its section 6.
    let mut module = String::new();
    let mut lines = script
        .lines()
        .skip_while(|l| !l.contains("functions/robustness.zeff contains"));
    lines.next();
    for line in lines {
        if let Some(code) = line.strip_prefix("//     ") {
            module.push_str(code);
            module.push('\n');
        } else if line.trim() != "//" {
            break;
        }
    }
    assert!(module.contains("fn robustness("), "{module}");
    fs.add_file("/p/functions/robustness.zeff", module);
    let report = self::report(&fs, &asked(script)).unwrap();
    assert_eq!(report.kind, Kind::Error);
    assert!(report.code.is_some());
    // Seventy-five items: a `use` line and seventy-four statements.
    assert_eq!(report.of, 74);
    assert!(
        report.kept < 15,
        "{} of {} statements are kept:\n{}",
        report.kept,
        report.of,
        report.text
    );
    for part in [
        "# Zeff bug report: an error, Z",
        "It holds no rows of your data.",
        "## What went wrong",
        &format!(
            "- version: {}\n- edition: preview\n- platform: the test's machine\n",
            zeff_engine::VERSION
        ),
        "## The statements",
        "## The run",
        "## The data",
    ] {
        assert!(report.text.contains(part), "{part}:\n{}", report.text);
    }
    insta::assert_snapshot!("report_of_the_sample_script", masked(&report.text));
}

/// The report with the engine's version masked, so that its snapshot does not change at every
/// opening and close of a release with nothing else changed (release 0.9.0, step 1).
fn masked(report: &str) -> String {
    report.replace(
        &format!("- version: {}\n", zeff_engine::VERSION),
        "- version: (the engine's version)\n",
    )
}
