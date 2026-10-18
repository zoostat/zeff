//! Running scripts in an in-memory project: the first milestone, every function of this step, the
//! messages under the output, and the errors that stop a run.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

const WAGES: &str = "id,wage,hours,name\n1,23.4,40,Ann\n2,19.0,35,Bob\n3,31.2,,Cy\n";

/// A project at /p with `data/wages.csv` and the extra files given; run `script`.
fn run_with(files: &[(&str, &str)], script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/wages.csv", WAGES);
    for (path, text) in files {
        fs.add_file(format!("/p/{path}"), text);
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn run(script: &str) -> Outcome {
    run_with(&[], script)
}

/// The one note every import prints (section 6.1), removed where a test looks at other messages.
fn without_format_notes(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn ok(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "stderr:\n{}", outcome.output.stderr);
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "",
        "no messages expected"
    );
    outcome.output.stdout
}

// ----- the milestone -----

#[test]
fn the_first_milestone_prints_the_mean_of_a_csv_column() {
    let outcome = run("let wages = import(\"data/wages.csv\")\nwages.mean(wage)\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "24.5333\n");
    assert_eq!(
        outcome.output.stderr,
        "note: data/wages.csv read as CSV: 3 rows × 4 variables, types guessed; CSV carries no labels, value labels or categories (line 1)\n",
        "the one note of every import"
    );
}

#[test]
fn the_pipeline_runs_in_order_and_prints_each_statement() {
    let out = ok(
        "let wages = import(\"data/wages.csv\")\nprint(\"rows:\", 3)\nwages.mean(id)\nprint(wages)\nwages\n",
    );
    assert_eq!(
        out,
        "rows:\n3\n2\nwages · 3 rows × 4 variables\nwages · 3 rows × 4 variables\n"
    );
}

#[test]
fn a_let_prints_nothing_and_a_later_let_shadows() {
    let out =
        ok("let x = 1\nlet x = \"two\"\nx\nlet w = import(\"data/wages.csv\")\nlet w2 = w\nw2\n");
    assert_eq!(
        out, "two\nw2 · 3 rows × 4 variables\n",
        "a frame takes the name it is bound to"
    );
}

#[test]
fn literals_lists_and_records_print_as_section_3_8_says() {
    let out = ok(
        "42\nprint(-7)\n2.5\n0.1\n1e-6\n\"text\"\ntrue\nNA\nInf\nprint(-Inf)\n[1, 2.5]\n[\"a\", NA]\n{a: 1, b: \"x\"}\nprint([[1], [2, 3]])\n",
    );
    assert_eq!(
        out,
        "42\n-7\n2.5\n0.1\n1e-06\ntext\ntrue\nNA\nInf\n-Inf\n1\n2.5\na\nNA\na: 1\nb: x\n[1]\n[2, 3]\n"
    );
}

// ----- mean -----

#[test]
fn mean_skips_missing_values_with_a_warning_under_the_output() {
    let outcome = run("let wages = import(\"data/wages.csv\")\nwages.mean(hours)\n");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.output.stdout, "37.5\n");
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "warning: 1 row with NA in hours was skipped (line 2)\n"
    );
}

#[test]
fn mean_of_a_bool_column_is_the_share_and_of_all_missing_is_na() {
    let outcome = run_with(
        &[("data/flags.csv", "treated,empty\ntrue,\nfalse,NA\ntrue,.\n")],
        "let f = import(\"data/flags.csv\", types: {empty: Float})\nf.mean(treated)\nf.mean(empty)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "0.666667\nNA\n");
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "warning: all 3 rows have NA in empty: the result is NA (line 3)\n"
    );
}

#[test]
fn infinities_in_a_mean() {
    let outcome = run_with(
        &[("data/inf.csv", "a,b\n1,1\nInf,Inf\n2,-Inf\n")],
        "let d = import(\"data/inf.csv\")\nd.mean(a)\nd.mean(b)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "Inf\nNA\n");
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "warning: b holds both Inf and -Inf, so its mean is NA (line 3)\n"
    );
}

#[test]
fn a_variable_that_is_not_there_stops_the_run_with_the_frames_name() {
    let outcome =
        run("let wages = import(\"data/wages.csv\")\nwages.mean(wge)\nprint(\"not reached\")\n");
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(
        outcome.output.stdout, "",
        "the run stops at the first error"
    );
    let text = &without_format_notes(&outcome.output.stderr);
    assert!(
        text.starts_with("error[Z0101]: no variable `wge` in `wages`"),
        "{text}"
    );
    assert!(text.contains("did you mean `wage`?"), "{text}");
    assert!(text.contains(" --> scripts/main.zeff:2:12"), "{text}");
    let case = run("let wages = import(\"data/wages.csv\")\nwages.mean(Wage)\n");
    assert!(
        without_format_notes(&case.output.stderr)
            .starts_with("error[Z0102]: no variable `Wage` in `wages`; names are case-sensitive"),
        "{}",
        case.output.stderr
    );
    let unnamed = run("import(\"data/wages.csv\").mean(wge)\n");
    assert!(
        without_format_notes(&unnamed.output.stderr)
            .starts_with("error[Z0101]: no variable `wge` in `the frame`"),
        "{}",
        unnamed.output.stderr
    );
}

#[test]
fn a_text_column_cannot_have_a_mean() {
    let outcome = run("let wages = import(\"data/wages.csv\")\nwages.mean(name)\n");
    assert_eq!(outcome.exit_code, 1);
    let text = &without_format_notes(&outcome.output.stderr);
    assert!(
        text.starts_with("error[Z0401]: `mean` needs a Bool, Int or Float, but `name` is Str"),
        "{text}"
    );
    assert!(
        text.contains("`wages.describe()` lists the types"),
        "{text}"
    );
}

// ----- import -----

#[test]
fn import_with_every_option_of_this_release() {
    let csv = "# produced 2026\nID;Gross Wage;Sold;flag\n1;-99;16/10/2026;0\n2;19,5;04/05/2026;1\n3;31;;0\n";
    let outcome = run_with(
        &[("data/odd.csv", csv)],
        "let d = import(\"data/odd.csv\", delim: \";\", skip: 1, na_values: [-99], types: {Sold: Date(\"%d/%m/%Y\"), flag: Bool}, names: snake)\nd\nd.mean(id)\n",
    );
    assert_eq!(outcome.exit_code, 1, "{}", outcome.output.stderr);
    assert!(outcome.output.stderr.contains("error[Z0606]: row 3 of `data/odd.csv` has a value that is not true or false in a Bool column"), "{}", outcome.output.stderr);
    // Without the wrong declaration the file reads, and import's notes print under its statement.
    let outcome = run_with(
        &[("data/odd.csv", csv)],
        "let d = import(\"data/odd.csv\", delim: \";\", skip: 1, na_values: [-99], types: {Sold: Date(\"%d/%m/%Y\")}, names: snake)\nd\nd.mean(id)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "d · 3 rows × 4 variables\n2\n");
    assert_eq!(
        without_format_notes(&outcome.output.stderr),
        "note: names: snake renamed 3 columns: `ID` → `id`, `Gross Wage` → `gross_wage`, `Sold` → `sold` (line 1)\nnote: 1 value of gross_wage read as NA from na_values -99 (line 1)\n"
    );
    assert!(
        outcome
            .output
            .stderr
            .starts_with("note: data/odd.csv read as CSV: 3 rows × 4 variables"),
        "the format note comes first:\n{}",
        outcome.output.stderr
    );
}

#[test]
fn a_frame_is_an_import_without_a_let() {
    let out = ok("import(\"data/wages.csv\")\n");
    assert_eq!(out, "frame · 3 rows × 4 variables\n");
}

#[test]
fn the_file_errors_of_import_point_at_the_call() {
    let missing = run("let d = import(\"data/missing.csv\")\n");
    assert_eq!(missing.exit_code, 1);
    assert!(
        missing
            .output
            .stderr
            .starts_with("error[Z0715]: no file `data/missing.csv` in the project"),
        "{}",
        missing.output.stderr
    );
    assert!(
        missing
            .output
            .stderr
            .contains(" --> scripts/main.zeff:1:16"),
        "the path is marked"
    );
    let outside = run("let d = import(\"../shared/wages.csv\")\n");
    assert!(
        outside
            .output
            .stderr
            .starts_with("error[Z0701]: path `../shared/wages.csv` is outside the project"),
        "{}",
        outside.output.stderr
    );
    let case = run("let d = import(\"data/Wages.csv\")\n");
    assert!(
        case.output.stderr.starts_with("error[Z0702]"),
        "{}",
        case.output.stderr
    );
    let mut long = String::from("id,hours\n");
    for i in 1..=1000 {
        long.push_str(&format!("{i},{}\n", 30 + i % 3));
    }
    long.push_str("1001,varies\n");
    let bad = run_with(
        &[("data/long.csv", &long)],
        "let d = import(\"data/long.csv\")\n",
    );
    assert!(
        bad.output
            .stderr
            .starts_with("error[Z0606]: row 1,002 of `data/long.csv` has text in a numeric column"),
        "{}",
        bad.output.stderr
    );
    assert!(
        bad.output
            .stderr
            .contains("  |         ^^^^^^ column `hours` read as Int"),
        "the call is marked:\n{}",
        bad.output.stderr
    );
}

#[test]
fn import_refuses_what_the_checker_cannot_see() {
    let delim = run("let d = import(\"data/wages.csv\", delim: \"ab\")\n");
    assert!(
        delim
            .output
            .stderr
            .starts_with("error[Z0401]: `delim:` takes one character"),
        "{}",
        delim.output.stderr
    );
    let skip = run("let d = import(\"data/wages.csv\", skip: -1)\n");
    assert!(
        skip.output
            .stderr
            .starts_with("error[Z0401]: `skip:` cannot be -1"),
        "{}",
        skip.output.stderr
    );
    let types = run("let d = import(\"data/wages.csv\", types: {wage: 1})\n");
    assert!(
        types
            .output
            .stderr
            .starts_with("error[Z0401]: `types:` takes type names, but `wage` is an Int"),
        "{}",
        types.output.stderr
    );
    let column = run("let d = import(\"data/wages.csv\", types: {Wage: Str})\n");
    assert!(
        column
            .output
            .stderr
            .starts_with("error[Z0101]: no column `Wage` in `data/wages.csv`"),
        "{}",
        column.output.stderr
    );
}

// ----- the run as a whole -----

#[test]
fn a_check_error_stops_the_run_before_anything_prints() {
    let outcome = run("print(\"first\")\nwages.maen(x)\nfoo\n");
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.output.stdout, "");
    assert!(
        outcome
            .output
            .stderr
            .starts_with("error[Z0101]: nothing is called `wages` here"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(
        outcome.output.stderr.matches("error[").count(),
        1,
        "zeff run stops at the first error"
    );
}

#[test]
fn a_check_warning_is_shown_and_the_script_runs() {
    let outcome = run("let mean = 3\nmean\n");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.output.stdout, "3\n");
    assert!(
        outcome
            .output
            .stderr
            .starts_with("warning[W0101]: `mean` hides the function `data::mean`"),
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn the_messages_of_a_run_as_a_snapshot() {
    let script = "let wages = import(\"data/wages.csv\", na_values: [-99])\nwages.mean(hours)\nwages.mean(wge)\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 1);
    insta::assert_snapshot!("run_stdout", outcome.output.stdout);
    insta::assert_snapshot!("run_stderr", outcome.output.stderr);
}
