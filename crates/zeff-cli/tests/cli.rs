//! Script tests of the real `zeff` program: arguments in, standard output, standard error and the
//! exit code out, in a temporary folder.

use std::path::Path;
use std::process::{Command, Output};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn zeff(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zeff"))
        .args(args)
        .current_dir(cwd)
        .env_remove("NO_COLOR")
        .output()
        .expect("zeff runs")
}

/// `zeff` with `ZEFF_HOME` set to `home` and `input` on standard input.
fn zeff_with(cwd: &Path, home: &Path, input: &str, args: &[&str]) -> Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_zeff"))
        .args(args)
        .current_dir(cwd)
        .env("ZEFF_HOME", home)
        .env_remove("NO_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zeff starts");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().expect("zeff runs")
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exited normally")
}

/// A new project called `demo` in a fresh temporary folder.
fn demo() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let made = zeff(dir.path(), &["new", "demo"]);
    assert_eq!(code(&made), 0, "{}", err(&made));
    let project = dir.path().join("demo");
    (dir, project)
}

#[test]
fn version_prints_the_version_on_standard_output() {
    let dir = tempfile::tempdir().unwrap();
    for flag in ["version", "--version", "-V"] {
        let o = zeff(dir.path(), &[flag]);
        assert_eq!(code(&o), 0);
        assert_eq!(out(&o), format!("zeff {VERSION}\n"));
        assert_eq!(err(&o), "");
    }
}

#[test]
fn format_json_gives_records_for_version_and_check() {
    let (_dir, project) = demo();
    let o = zeff(&project, &["version", "--format", "json"]);
    assert_eq!(code(&o), 0);
    assert_eq!(out(&o), format!("{{\"version\": \"{VERSION}\"}}\n"));
    let o = zeff(&project, &["--format=json", "check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "[\n]\n");
    assert_eq!(err(&o), "");
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\", heade: false)\nwges.maen(wage)\nlet mean = 3\n",
    )
    .unwrap();
    let o = zeff(&project, &["check", "--format", "json"]);
    assert_eq!(code(&o), 1);
    assert_eq!(err(&o), "");
    let text = out(&o);
    // The file does not exist in the new project: the first record is the run-time mark, a
    // note at the path (release 0.6.0, step 20); the errors follow in position order.
    assert!(
        text.starts_with("[\n  {\"code\": \"\", \"severity\": \"note\""),
        "{text}"
    );
    assert!(text.contains("{\"code\": \"Z0404\""), "{text}");
    assert!(
        text.contains("\"span\": {\"line\": 2, \"column\": 1, \"end_line\": 2, \"end_column\": 5}"),
        "{text}"
    );
    assert!(text.contains("\"severity\": \"warning\""), "{text}");
    insta::assert_snapshot!("check_json", text);
    for (args, message) in [
        (&["check", "--format"][..], "needs a value"),
        (&["check", "--format", "xml"][..], "not a format"),
        (
            &["run", "--format", "json"][..],
            "applies to `zeff check`, `zeff test`, `zeff version` and `zeff selftest`",
        ),
    ] {
        let o = zeff(&project, args);
        assert_eq!(code(&o), 1, "{args:?}");
        assert!(err(&o).contains(message), "{args:?}: {}", err(&o));
    }
}

#[test]
fn help_explains_a_code() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["help", "Z0101"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "");
    let text = out(&o);
    assert!(text.starts_with("Z0101 — "), "{text}");
    for part in ["MEANING", "COMMON CAUSES", "EXAMPLE", "FIX"] {
        assert!(text.contains(&format!("\n{part}\n")), "{part} in\n{text}");
    }
    insta::assert_snapshot!("help_Z0101", text);
    let o = zeff(dir.path(), &["help", "Z0000"]);
    assert_eq!(code(&o), 1);
    assert_eq!(err(&o), "error: no code `Z0000`\n");
    assert_eq!(out(&o), "");
}

#[test]
fn no_arguments_and_help_show_the_commands() {
    let dir = tempfile::tempdir().unwrap();
    for args in [&[][..], &["help"], &["--help"], &["-h"]] {
        let o = zeff(dir.path(), args);
        assert_eq!(code(&o), 0);
        assert!(out(&o).contains("new <name>"), "{}", out(&o));
        assert_eq!(err(&o), "");
    }
}

#[test]
fn zeff_new_creates_the_project() {
    let (_dir, project) = demo();
    for path in ["zeff.toml", ".gitignore", "scripts/main.zeff", "data"] {
        assert!(project.join(path).exists(), "{path}");
    }
    let toml = std::fs::read_to_string(project.join("zeff.toml")).unwrap();
    assert!(toml.contains("name = \"demo\"") && toml.contains(&format!("zeff = \"{VERSION}\"")));
}

#[test]
fn zeff_new_says_what_it_made_and_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let first = zeff(dir.path(), &["new", "demo"]);
    assert_eq!(code(&first), 0);
    assert!(out(&first).contains("created the project `demo`"));
    let again = zeff(dir.path(), &["new", "demo"]);
    assert_eq!(code(&again), 1);
    assert!(err(&again).contains("already exists"), "{}", err(&again));
    assert_eq!(out(&again), "");
}

#[test]
fn zeff_new_refuses_bad_names_with_advice() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["new", "My Project"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("zeff new my_project"), "{}", err(&o));
    assert!(!dir.path().join("My Project").exists());
    for args in [&["new"][..], &["new", "a", "b"]] {
        let o = zeff(dir.path(), args);
        assert_eq!(code(&o), 1);
        assert!(err(&o).contains("takes one name"));
    }
}

#[test]
fn commands_that_need_a_project_say_so_outside_one() {
    let dir = tempfile::tempdir().unwrap();
    for command in ["run", "check", "fmt", "test"] {
        let o = zeff(dir.path(), &[command]);
        assert_eq!(code(&o), 1, "{command}");
        assert_eq!(out(&o), "");
        assert!(
            err(&o).starts_with(
                "error[Z0709]: not inside a Zeff project; `zeff new <name>` creates one"
            ),
            "{}",
            err(&o)
        );
    }
    insta::assert_snapshot!("outside_a_project", err(&zeff(dir.path(), &["run"])));
}

#[test]
fn a_project_is_found_from_a_folder_inside_it() {
    let (_dir, project) = demo();
    let o = zeff(&project.join("scripts"), &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/main.zeff: no problems\n");
    assert_eq!(out(&o), "");
}

#[test]
fn zeff_check_passes_a_sound_script() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\", na_values: [-99])\nwages.mean(wage)\nprint(wages.mean(hours), \"hours\")\n",
    )
    .unwrap();
    // The file does not exist in the new project: the frame is checked at run time, which the
    // check says in a note, and is no problem.
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let text = err(&o);
    assert!(
        text.starts_with(
            "note: `data/wages.csv` is checked at run time: the file does not exist yet"
        ),
        "{text}"
    );
    assert!(
        text.ends_with("checked scripts/main.zeff: no problems\n"),
        "{text}"
    );
    // With the file in place the frame's variables are checked, and nothing is said.
    std::fs::write(project.join("data/wages.csv"), "wage,hours\n10,40\n12,38\n").unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/main.zeff: no problems\n");
}

/// The schema pass (release 0.6.0, step 20): a misspelt variable in the last line of a long
/// script is reported by `zeff check` and by `zeff run` before the first import runs; two
/// errors in one script are both listed; a missing file carries the run-time mark; a stale
/// cache is refreshed when the file changes.
#[test]
fn zeff_check_reads_the_schemas_before_anything_runs() {
    let (_dir, project) = demo();
    std::fs::write(project.join("data/small.csv"), "a,b\n1,2.5\n3,4.5\n").unwrap();
    let mut script = String::from("let d = import(\"data/small.csv\")\nprint(\"started\")\n");
    for _ in 0..40 {
        script.push_str("d.mean(a)\n");
    }
    script.push_str("d.mean(aa)\n");
    std::fs::write(project.join("scripts/main.zeff"), &script).unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    let text = err(&o);
    assert!(
        text.contains("error[Z0101]: no variable `aa` in `d`"),
        "{text}"
    );
    assert!(text.contains("did you mean `a`?"), "{text}");
    assert!(
        text.ends_with("scripts/main.zeff: 1 error, 0 warnings\n"),
        "{text}"
    );
    assert!(
        project.join(".zeff/schemas/schemas.json").is_file(),
        "the cache is written"
    );
    // `zeff run` stops at the check: nothing is printed, not even the first statement.
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    assert!(
        err(&o).contains("error[Z0101]: no variable `aa` in `d`"),
        "{}",
        err(&o)
    );
    // Two errors in one script: both listed.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let d = import(\"data/small.csv\")\nd.mean(aa)\nd.summary([a, bb])\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    let text = err(&o);
    assert!(
        text.contains("no variable `aa` in `d`") && text.contains("no variable `bb` in `d`"),
        "{text}"
    );
    assert!(
        text.ends_with("scripts/main.zeff: 2 errors, 0 warnings\n"),
        "{text}"
    );
    // A missing file: the run-time mark, in the text and in `--format json`.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let m = import(\"data/missing.csv\")\nm.mean(zzz)\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        err(&o).contains("is checked at run time: the file does not exist yet"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["check", "--format", "json"]);
    assert_eq!(code(&o), 0);
    assert!(out(&o).contains("\"severity\": \"note\""), "{}", out(&o));
    assert!(out(&o).contains("checked at run time"), "{}", out(&o));
    // A stale cache: the file gains a column and loses one; the check sees the new file.
    std::fs::write(project.join("data/small.csv"), "a,c\n1,x\n3,y\n").unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let d = import(\"data/small.csv\")\nd.mean(a)\nd.mean(b)\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    let text = err(&o);
    assert!(text.contains("no variable `b` in `d`"), "{text}");
    assert!(
        text.contains("did you mean `a`?") || text.contains("the variables of `d` are"),
        "{text}"
    );
    let cache = std::fs::read_to_string(project.join(".zeff/schemas/schemas.json")).unwrap();
    assert!(
        cache.contains("[\"a\", \"Int\"], [\"c\", \"Str\"]"),
        "{cache}"
    );
    assert!(!cache.contains("\"b\""), "{cache}");
}

#[test]
fn zeff_check_reports_every_problem_and_exits_1() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "use stats\nlet wages = import(\"data/wages.csv\", heade: false)\nwges.maen(wage)\nlet mean = 3\nwages.mean(\"wage\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    let text = err(&o);
    for code in ["Z0404", "Z0101", "Z0103", "W0101", "Z0401", "Z0304"] {
        assert!(text.contains(&format!("[{code}]")), "{code} in\n{text}");
    }
    assert!(
        text.ends_with("scripts/main.zeff: 5 errors, 1 warning\n"),
        "{text}"
    );
    insta::assert_snapshot!("check_with_problems", text);
}

#[test]
fn zeff_check_takes_a_script_and_refuses_one_that_is_not_there() {
    let (_dir, project) = demo();
    std::fs::write(project.join("scripts/clean.zeff"), "print(1)\n").unwrap();
    let o = zeff(&project, &["check", "scripts/clean.zeff"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/clean.zeff: no problems\n");
    let o = zeff(&project, &["check", "scripts/other.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).starts_with("error[Z0714]: no script `scripts/other.zeff` in the project"),
        "{}",
        err(&o)
    );
    insta::assert_snapshot!("no_such_script", err(&o));
    let o = zeff(&project, &["check", "../outside.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).starts_with("error[Z0701]"), "{}", err(&o));
    let o = zeff(&project, &["check", "a", "b"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("takes at most one script"));
}

#[test]
fn a_parse_error_is_reported_by_zeff_check_too() {
    let (_dir, project) = demo();
    std::fs::write(project.join("scripts/main.zeff"), "let a = 5.\n").unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("error[Z0310]"), "{}", err(&o));
    assert!(
        err(&o).ends_with("scripts/main.zeff: 1 error, 0 warnings\n"),
        "{}",
        err(&o)
    );
}

#[test]
fn a_project_that_pins_another_engine_is_exit_code_4() {
    let (_dir, project) = demo();
    let path = project.join("zeff.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace(VERSION, "9.9.9");
    std::fs::write(&path, text).unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 4);
    assert!(
        err(&o).starts_with("error[Z0710]: this project needs Zeff 9.9.9"),
        "{}",
        err(&o)
    );
    assert!(err(&o).contains("--> zeff.toml:3:8"));
}

#[test]
fn an_edition_this_engine_does_not_know_is_exit_code_4() {
    let (_dir, project) = demo();
    let path = project.join("zeff.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("preview", "2027");
    std::fs::write(&path, text).unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 4);
    assert!(err(&o).contains("error[Z0706]"), "{}", err(&o));
}

#[test]
fn a_broken_project_file_is_reported_in_full_with_exit_code_1() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("zeff.toml"),
        "[project]\nname = 5\nnetwrk = true\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    let text = err(&o);
    assert!(
        text.contains("error[Z0713]: `name` in [project] must be text"),
        "{text}"
    );
    assert!(text.contains("unknown key `netwrk`"), "{text}");
    assert!(text.contains("[project] needs `zeff`"), "{text}");
    std::fs::write(project.join("zeff.toml"), "[project\n").unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("error[Z0712]"));
}

#[test]
fn unknown_and_later_commands() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["frobnicate"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("unknown command `frobnicate`") && err(&o).contains("zeff help"));
    for later in ["doctor", "engine", "verify"] {
        let o = zeff(dir.path(), &[later]);
        assert_eq!(code(&o), 1);
        assert!(
            err(&o).contains("not available in this release yet"),
            "{later}"
        );
    }
    let o = zeff(dir.path(), &["version", "extra"]);
    assert_eq!(code(&o), 1);
}

// ----- zeff run -----

const WAGES: &str = "id,wage,hours\n1,23.4,40\n2,19.0,35\n3,31.2,\n";

#[test]
fn zeff_run_prints_the_mean_of_a_csv_column() {
    let (_dir, project) = demo();
    std::fs::write(project.join("data/wages.csv"), WAGES).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\")\nwages.mean(wage)\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "24.5333\n");
    assert_eq!(
        err(&o),
        "note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)\n"
    );
}

#[test]
fn zeff_run_writes_output_to_stdout_and_messages_to_stderr() {
    let (_dir, project) = demo();
    std::fs::write(project.join("data/wages.csv"), WAGES).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\")\nwages.mean(hours)\nprint(\"done\")\n",
    )
    .unwrap();
    let o = zeff(&project.join("data"), &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "37.5\ndone\n");
    assert_eq!(
        err(&o),
        "note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)\nwarning[W0201]: 1 row with NA in hours was skipped (line 2)\n"
    );
}

#[test]
fn zeff_run_stops_at_the_first_error_with_exit_code_1() {
    let (_dir, project) = demo();
    std::fs::write(project.join("data/wages.csv"), WAGES).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\")\nwages.mean(wge)\nprint(\"not reached\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    assert!(
        err(&o).starts_with("error[Z0101]: no variable `wge` in `wages`"),
        "{}",
        err(&o)
    );
    std::fs::write(project.join("scripts/main.zeff"), "foo\nbar\n").unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1);
    assert_eq!(err(&o).matches("error[").count(), 1, "the first error only");
}

#[test]
fn zeff_run_takes_a_script_and_needs_a_project() {
    let (_dir, project) = demo();
    std::fs::write(project.join("scripts/other.zeff"), "print(1, 2)\n").unwrap();
    let o = zeff(&project, &["run", "scripts/other.zeff"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "1\n2\n");
    let o = zeff(&project, &["run", "scripts/none.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).starts_with("error[Z0714]"));
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["run"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).starts_with("error[Z0709]"));
    let o = zeff(&project, &["run", "a", "b"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("takes at most one script"));
}

#[test]
fn a_new_project_runs_to_nothing() {
    let (_dir, project) = demo();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "");
    assert_eq!(err(&o), "");
}

// ----- zeff help -----

#[test]
fn zeff_help_prints_a_functions_page() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["help", "mean"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let text = out(&o);
    assert!(text.starts_with("mean — Mean of a variable"), "{text}");
    assert!(
        text.contains("\nUSAGE\n    data.mean(var, by: Vars, weights: Var, weight_type: frequency | analytic | sampling = sampling)\n"),
        "{text}"
    );
    assert!(text.contains("\nEXAMPLES\n"), "{text}");
    assert_eq!(err(&o), "");
    insta::assert_snapshot!("help_mean", text);
    insta::assert_snapshot!("help_import", out(&zeff(dir.path(), &["help", "import"])));
    insta::assert_snapshot!("help_print", out(&zeff(dir.path(), &["help", "print"])));
    // A function returning a result type lists the methods of that type under Value (section
    // 9.1), as sentences after the written text, grouped as the catalogue groups them.
    let reg = out(&zeff(dir.path(), &["help", "reg"]));
    let value = reg
        .split("\nVALUE\n")
        .nth(1)
        .and_then(|rest| rest.split("\nDETAILS\n").next())
        .unwrap_or("")
        .replace("\n    ", " ");
    assert_eq!(
        value.trim(),
        "A `RegFit`, which prints as its summary. Result interface: summary, estimates, std_errors, p_values, ci, n_obs, export. Result plots: plot, coef_plot. Estimator interface: coef, vcov, fit_measures, predict, residuals, fitted, compare, test, derive. Result methods: r_squared, influence, vif, design.",
        "{reg}"
    );
    insta::assert_snapshot!("help_reg", reg);
}

#[test]
fn zeff_help_lists_a_package_and_says_when_a_function_arrives() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["help", "data"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    insta::assert_snapshot!("help_data", out(&o));
    let core = zeff(dir.path(), &["help", "core"]);
    assert!(
        out(&core).starts_with("core · tier 0 · 72 of 75 functions in this release"),
        "{}",
        out(&core)
    );
    let later = zeff(dir.path(), &["help", "version"]);
    assert_eq!(code(&later), 0);
    assert!(
        out(&later).starts_with("`version` arrives with release 0.11.0, in the `core` package: "),
        "{}",
        out(&later)
    );
    // A name with two homes, one shipped (`data`'s `aggregate`, release 0.6.0) and one to come
    // (`plot`'s, after 1.0): the page, then the line that says when the other arrives.
    let two_homes = zeff(dir.path(), &["help", "aggregate"]);
    let text = out(&two_homes);
    assert!(
        text.starts_with("aggregate — One row per group with reducers"),
        "{text}"
    );
    assert!(
        text.trim_end().lines().last().is_some_and(|l| l
            .starts_with("`aggregate` arrives with a release after 1.0, in the `plot` package: ")),
        "{text}"
    );
    // A name with two homes shipped in two tiers (`core`'s `append` on lists, `data`'s on
    // frames): both pages, and no line about a later release.
    let two_tiers = zeff(dir.path(), &["help", "append"]);
    let text = out(&two_tiers);
    assert!(
        text.starts_with("append — A new list with an item added"),
        "{text}"
    );
    assert!(
        text.contains("append — Stack another frame's rows below")
            && !text.contains("arrives with"),
        "{text}"
    );
    // A name with two homes shipped (frames in `data`, results in `results`): both pages, and no
    // line about a later release.
    let two_shipped = zeff(dir.path(), &["help", "summary"]);
    assert!(
        out(&two_shipped).starts_with("summary — Table of N, missing, mean, SD, min and max"),
        "{}",
        out(&two_shipped)
    );
    assert!(
        out(&two_shipped).contains("\nsummary — The printed summary of a result as a table"),
        "{}",
        out(&two_shipped)
    );
    assert!(
        !out(&two_shipped).contains("arrives with"),
        "{}",
        out(&two_shipped)
    );
    let package_later = zeff(dir.path(), &["help", "glm"]);
    assert!(
        out(&package_later)
            .contains("has no function in this release; its first arrives with release 0.9.0"),
        "{}",
        out(&package_later)
    );
}

#[test]
fn zeff_help_refuses_an_unknown_name_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["help", "maen"]);
    assert_eq!(code(&o), 1);
    assert_eq!(
        err(&o),
        "error: no function or package is called `maen`; did you mean `mean`?\n"
    );
    assert_eq!(out(&o), "");
    let o = zeff(dir.path(), &["help", "zzzz"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("`zeff help data` the functions of a package"));
    let o = zeff(dir.path(), &["help", "a", "b"]);
    assert_eq!(code(&o), 1);
}

#[test]
fn every_sample_project_runs_to_its_stored_output() {
    for name in [
        "first_engine",
        "data_cleaning",
        "simulation",
        "regression",
        "user_functions",
        "panel",
    ] {
        let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../samples/{name}"));
        let o = zeff(&sample, &["run"]);
        assert_eq!(code(&o), 0, "{name}: {}", err(&o));
        let stdout = std::fs::read(sample.join("expected/stdout.txt")).unwrap();
        let stderr = std::fs::read(sample.join("expected/stderr.txt")).unwrap();
        assert!(
            o.stdout == stdout,
            "the output of samples/{name} changed; regenerate expected/stdout.txt with `zeff run` there if that is intended:\n{}",
            out(&o)
        );
        assert!(
            o.stderr == stderr,
            "the messages of samples/{name} changed:\n{}",
            err(&o)
        );
        let checked = zeff(&sample, &["check"]);
        assert_eq!(code(&checked), 0, "{name}: {}", err(&checked));
    }
}

// ----- the options of section 2.1 -----

/// A file wide enough to wrap at a narrow page.
const WIDE: &str = "a_long_variable_name_01,a_long_variable_name_02,a_long_variable_name_03,a_long_variable_name_04\n1,2,3,4\n5,6,7,8\n";

#[test]
fn width_and_ascii_change_how_a_frame_prints() {
    let (_dir, project) = demo();
    std::fs::write(project.join("data/wide.csv"), WIDE).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wide = import(\"data/wide.csv\")\nwide\n",
    )
    .unwrap();
    let plain = zeff(&project, &["run", "-q"]);
    assert_eq!(code(&plain), 0, "{}", err(&plain));
    assert_eq!(err(&plain), "", "quiet hides the note");
    assert!(
        out(&plain).contains("wide · 2 rows × 4 variables"),
        "{}",
        out(&plain)
    );
    assert_eq!(out(&plain).lines().count(), 5, "{}", out(&plain));
    let narrow = zeff(&project, &["run", "--width", "60", "--ascii", "--quiet"]);
    assert_eq!(code(&narrow), 0, "{}", err(&narrow));
    let text = out(&narrow);
    assert!(text.starts_with("wide | 2 rows x 4 variables\n"), "{text}");
    assert!(text.lines().all(|l| l.chars().count() <= 60), "{text}");
    assert!(text.lines().count() > 5, "blocks of columns:\n{text}");
    insta::assert_snapshot!("run_width_60_ascii", text);
    for args in [
        &["run", "--width", "20"][..],
        &["run", "--width"][..],
        &["run", "--width=wide"][..],
    ] {
        let o = zeff(&project, args);
        assert_eq!(code(&o), 1, "{args:?}");
        assert!(err(&o).contains("--width"), "{}", err(&o));
    }
}

#[test]
fn colour_quiet_verbose_and_project_options() {
    let (dir, project) = demo();
    std::fs::write(project.join("data/wages.csv"), WAGES).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\")\nwages.mean(hours)\n",
    )
    .unwrap();
    // Off a terminal there is no colour; `--color always` paints the word before the colon.
    let o = zeff(&project, &["run"]);
    assert!(
        err(&o).starts_with("note: data/wages.csv read as CSV"),
        "{}",
        err(&o)
    );
    assert!(
        err(&o).contains("warning[W0201]: 1 row with NA in hours"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["run", "--color", "always"]);
    assert!(
        err(&o).contains("\x1b[1;36mnote\x1b[0m: data/wages.csv"),
        "{}",
        err(&o)
    );
    assert!(
        err(&o).contains("\x1b[1;33mwarning[W0201]\x1b[0m: 1 row"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["run", "--color", "never"]);
    assert!(!err(&o).contains("\x1b["), "{}", err(&o));
    let o = zeff(&project, &["run", "--color", "sometimes"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("--color"), "{}", err(&o));
    std::fs::write(project.join("scripts/main.zeff"), "wges\n").unwrap();
    let o = zeff(&project, &["check", "--color=always"]);
    assert!(
        err(&o).starts_with("\x1b[1;31merror[Z0101]\x1b[0m: nothing is called `wges`"),
        "{}",
        err(&o)
    );
    // Quiet hides notes and nothing else.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\")\nwages.mean(hours)\n",
    )
    .unwrap();
    let o = zeff(&project, &["--quiet", "run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(
        err(&o),
        "warning[W0201]: 1 row with NA in hours was skipped (line 2)\n"
    );
    assert_eq!(out(&o), "37.5\n");
    // Verbose adds a timing line.
    let o = zeff(&project, &["run", "-v"]);
    assert!(
        err(&o)
            .lines()
            .last()
            .unwrap()
            .starts_with("ran scripts/main.zeff: 2 statements in "),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["version", "--verbose"]);
    assert!(
        out(&o).starts_with(&format!("zeff {VERSION}\ntarget: ")),
        "{}",
        out(&o)
    );
    // `--project` names the project from anywhere.
    let o = zeff(dir.path(), &["--project", "demo", "check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/main.zeff: no problems\n");
    let o = zeff(dir.path(), &["-p", "nowhere", "check"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("is not a folder"), "{}", err(&o));
    // Options of later releases and unknown ones are refused by name.
    let o = zeff(&project, &["run", "--frozen"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`--frozen` is not available in this release yet"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["run", "--bogus"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("unknown option `--bogus`"), "{}", err(&o));
}

// ----- zeff eval and the scratch project -----

#[test]
fn eval_runs_a_statement_in_the_scratch_project() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("zeff-home");
    let o = zeff_with(dir.path(), &home, "", &["eval", "2 + 2"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "4\n");
    assert_eq!(err(&o), "");
    assert!(home.join("scratch/zeff.toml").is_file());
    assert!(home.join("scratch/scripts/main.zeff").is_file());
    let o = zeff_with(dir.path(), &home, "", &["eval", "foo"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    assert!(
        err(&o).starts_with("error[Z0101]: nothing is called `foo` here\n --> eval:1:1"),
        "{}",
        err(&o)
    );
    // A scratch project left by another engine is repinned, not refused.
    std::fs::write(
        home.join("scratch/zeff.toml"),
        "[project]\nname = \"scratch\"\nzeff = \"0.0.1\"\nedition = \"preview\"\n",
    )
    .unwrap();
    let o = zeff_with(dir.path(), &home, "", &["eval", "1.5 * 2"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "3\n");
    for args in [&["eval"][..], &["eval", "2", "+", "2"][..]] {
        let o = zeff_with(dir.path(), &home, "", args);
        assert_eq!(code(&o), 1, "{args:?}");
        assert!(err(&o).contains("zeff eval \"2 + 2\""), "{}", err(&o));
    }
}

// ----- zeff repl -----

#[test]
fn the_console_keeps_a_session_in_the_project() {
    let (_dir, project) = demo();
    let home = project.join("unused-home");
    std::fs::write(project.join("data/wages.csv"), WAGES).unwrap();
    std::fs::write(project.join("scripts/clean.zeff"), "let two = 1 + 1\ntwo\n").unwrap();
    let input = "\
let x = 2
x * 21
let x = 3
:vars
wages
print(1 +
1)
let w = import(\"data/wages.csv\")
w
.describe()
:undo
:save scripts/explore.zeff
:save scripts/explore.zeff
:save scripts/main.zeff
:load scripts/clean.zeff
:help
:bogus
:quit
let never = 1
";
    let o = zeff_with(&project, &home, input, &["repl", "-q"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let (text, messages) = (out(&o), err(&o));
    assert!(
        messages.starts_with(&format!("Zeff {VERSION} console in the project `demo`")),
        "{messages}"
    );
    assert!(text.starts_with("42\nx  Int\n2\n"), "{text}");
    assert!(
        text.contains("w · 3 rows × 3 variables\n\n      id   wage   hours\n"),
        "{text}"
    );
    assert!(
        text.contains("  variable   type    non-missing"),
        "the chain was continued:\n{text}"
    );
    assert!(
        text.contains(":undo             drop the last statement"),
        "{text}"
    );
    assert!(
        messages.contains("error[Z0101]: nothing is called `wages` here"),
        "{messages}"
    );
    assert!(messages.contains("undid: w\n"), "{messages}");
    assert!(
        messages.contains("saved the 5 statements of the session to scripts/explore.zeff\n"),
        "{messages}"
    );
    assert!(
        messages.contains("error: `scripts/main.zeff` already exists"),
        "{messages}"
    );
    assert!(
        messages.contains("loaded 2 statements from scripts/clean.zeff"),
        "{messages}"
    );
    assert!(
        messages.contains("unknown console command `:bogus`"),
        "{messages}"
    );
    assert!(
        !home.exists(),
        "a project's console does not touch the scratch project"
    );
    let session = std::fs::read_to_string(project.join(".zeff/session.zeff")).unwrap();
    assert_eq!(
        session,
        "let x = 2\nx * 21\nlet x = 3\nprint(1 +\n1)\nlet w = import(\"data/wages.csv\")\nlet two = 1 + 1\ntwo\n"
    );
    let explore = std::fs::read_to_string(project.join("scripts/explore.zeff")).unwrap();
    assert_eq!(
        explore,
        "let x = 2\nx * 21\nlet x = 3\nprint(1 +\n1)\nlet w = import(\"data/wages.csv\")\n"
    );
    // A second console resumes the session and `:vars` shows the bindings as they are now.
    let o = zeff_with(&project, &home, "x + two\n:vars\n", &["repl", "--quiet"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        err(&o).contains("resumed 7 statements from .zeff/session.zeff"),
        "{}",
        err(&o)
    );
    assert_eq!(out(&o), "5\nx    Int\nw    Frame\ntwo  Int\n");
    // `zeff repl <script>` loads the script first.
    std::fs::remove_file(project.join(".zeff/session.zeff")).unwrap();
    let o = zeff_with(
        &project,
        &home,
        "two + 1\n",
        &["repl", "scripts/clean.zeff"],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "2\n3\n");
}

#[test]
fn the_console_outside_a_project_uses_the_scratch_project_and_can_make_one() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("zeff-home");
    let input = "let q = 7\nq * 6\n:save\n:save bad name\n:save fresh\n:save fresh\n";
    let o = zeff_with(dir.path(), &home, input, &["repl"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "42\n");
    let messages = err(&o);
    assert!(
        messages.contains("console in the scratch project"),
        "{messages}"
    );
    assert!(messages.contains("`:save` needs a path"), "{messages}");
    assert!(
        messages.contains("is not a good project name"),
        "{messages}"
    );
    assert!(
        messages.contains("created the project `fresh`"),
        "{messages}"
    );
    assert!(messages.contains("already exists"), "{messages}");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("fresh/scripts/main.zeff")).unwrap(),
        "let q = 7\nq * 6\n"
    );
    assert_eq!(
        std::fs::read_to_string(home.join("scratch/.zeff/session.zeff")).unwrap(),
        "let q = 7\nq * 6\n"
    );
    let o = zeff_with(&dir.path().join("fresh"), &home, "", &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "42\n");
}

// ----- zeff selftest (release 0.3.0, step 5) -----

/// `zeff` with one environment variable set.
fn zeff_env(cwd: &Path, key: &str, value: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zeff"))
        .args(args)
        .current_dir(cwd)
        .env(key, value)
        .env_remove("NO_COLOR")
        .output()
        .expect("zeff runs")
}

#[test]
fn selftest_says_this_build_is_conformant() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["selftest"]);
    assert_eq!(code(&o), 0, "{}", out(&o));
    let text = out(&o);
    assert!(
        text.starts_with("conformant: ")
            && text.contains(&format!(
                " cases match the published results of Zeff {VERSION} on this machine ("
            )),
        "{text}"
    );
    assert_eq!(err(&o), "");
    let o = zeff(dir.path(), &["selftest", "--format", "json"]);
    assert_eq!(code(&o), 0);
    let lines: Vec<&str> = out(&o)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .leak()
        .iter()
        .map(String::as_str)
        .collect();
    assert!(lines.len() > 70, "{}", lines.len());
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("{\"case\": \"") && l.ends_with("\"match\": true}")),
        "{}",
        lines[0]
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("\"case\": \"random/uniform\""))
    );
    let o = zeff(dir.path(), &["selftest", "now"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`zeff selftest` takes no arguments"),
        "{}",
        err(&o)
    );
}

#[test]
fn selftest_names_a_case_that_differs_and_fails() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff_env(
        dir.path(),
        "ZEFF_SELFTEST_FORCE_MISMATCH",
        "maths/exp",
        &["selftest"],
    );
    assert_eq!(code(&o), 1);
    let text = out(&o);
    assert!(text.starts_with("not conformant: 1 of "), "{text}");
    assert!(
        text.contains("  maths/exp: published 30bcae2e6caaaf20, this build 30bcae2e6caaaf21\n"),
        "{text}"
    );
    assert!(
        text.contains("please report it with `zeff version --verbose`"),
        "{text}"
    );
    let o = zeff_env(
        dir.path(),
        "ZEFF_SELFTEST_FORCE_MISMATCH",
        "maths/exp",
        &["selftest", "--format", "json"],
    );
    assert_eq!(code(&o), 1);
    assert!(out(&o).contains("{\"case\": \"maths/exp\", \"published\": \"30bcae2e6caaaf20\", \"actual\": \"30bcae2e6caaaf21\", \"match\": false}"), "{}", out(&o));
}

// ----- modules (release 0.5.0, step 8) -----

const CLEAN_MODULE: &str = "/// Share of rows where a variable is missing.\n///\n/// Counts the rows where `var` is `NA` and divides by the row count.\n///\n/// data: The frame to check.\n/// var: The variable whose missing values are counted.\n/// returns: The share, a Float between 0 and 1.\n/// > dataset(wages).share_missing(wage)\n/// 0.0114599\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n";

#[test]
fn zeff_check_and_run_read_the_project_s_modules() {
    let (_dir, project) = demo();
    std::fs::create_dir_all(project.join("functions")).unwrap();
    std::fs::write(project.join("functions/clean.zeff"), CLEAN_MODULE).unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "use functions::clean\ndataset(wages).share_missing(wage)\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/main.zeff: no problems\n");
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "0.0114599\n");
    // A module whose function lacks its page fails the check, under the module's file.
    std::fs::write(
        project.join("functions/bare.zeff"),
        "/// Halves.\nfn half(x: Float) -> Float { x / 2 }\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    let text = err(&o);
    assert!(text.contains("error[Z0319]"), "{text}");
    assert!(text.contains("--> functions/bare.zeff:2:4"), "{text}");
    insta::assert_snapshot!("check_with_a_module_problem", text);
}

#[test]
fn zeff_help_prints_a_module_function_s_page_inside_the_project() {
    let (_dir, project) = demo();
    std::fs::create_dir_all(project.join("functions")).unwrap();
    std::fs::write(project.join("functions/clean.zeff"), CLEAN_MODULE).unwrap();
    let o = zeff(&project, &["help", "share_missing"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        out(&o).starts_with("share_missing — Share of rows where a variable is missing."),
        "{}",
        out(&o)
    );
    assert!(
        out(&o).contains("functions/clean.zeff · a function of this project"),
        "{}",
        out(&o)
    );
    insta::assert_snapshot!("help_module_function", out(&o));
    // The full form, and a built-in's page is still the built-in's.
    let o = zeff(&project, &["help", "clean::share_missing"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let o = zeff(&project, &["help", "mean"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(out(&o).contains("data · tier 1"), "{}", out(&o));
}

#[test]
fn a_failed_assert_stops_the_run_with_exit_code_1() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = dataset(wages)\nassert(wages.n_rows() > 0, \"no rows\")\nprint(\"checked\")\nassert(wages.n_rows() > 100000, \"the sample is too small\")\nprint(\"never\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1, "{}", err(&o));
    assert_eq!(out(&o), "checked\n");
    insta::assert_snapshot!("failed_assert", err(&o));
}

#[test]
fn verbose_prints_the_memory_record_of_a_run() {
    // The memory record of section 6 (release 0.6.0, step 6): what each frame held when bound
    // and when a binding was freed after its last use; the byte counts are Arrow's and masked
    // here, since they may move with its version.
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = dataset(wages)\nlet kept = wages.where(hours > 30)\nlet few = wages.where(hours > 44)\nkept.n_rows()\nfew.n_rows()\n",
    )
    .unwrap();
    let o = zeff(&project, &["run", "--verbose"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let memory: Vec<String> = err(&o)
        .lines()
        .filter(|l| l.starts_with("memory:"))
        .map(|l| {
            let mut masked = String::new();
            let mut words = l.split(' ').peekable();
            while let Some(word) = words.next() {
                if word.parse::<f64>().is_ok()
                    && words
                        .peek()
                        .is_some_and(|unit| matches!(unit.trim_end_matches(','), "B" | "kB" | "MB"))
                {
                    masked.push_str("<size> ");
                    let unit = words.next().unwrap();
                    masked.push_str(if unit.ends_with(',') {
                        "<unit>,"
                    } else {
                        "<unit>"
                    });
                } else {
                    masked.push_str(word);
                }
                masked.push(' ');
            }
            masked.trim_end().to_string()
        })
        .collect();
    insta::assert_snapshot!("run_verbose_memory", memory.join("\n"));
    assert!(
        err(&o)
            .lines()
            .last()
            .unwrap()
            .starts_with("ran scripts/main.zeff: 5 statements in"),
        "{}",
        err(&o)
    );
}

/// The schema pass, part 2 (release 0.6.0, step 21): a mistake after a verb is caught at the
/// check; after a run with `to_wide`, a misspelt new column is a warning from last run, not an
/// error; a changed data-root file is warned about; `--format json` carries both marks.
#[test]
fn zeff_check_carries_schemas_through_verbs_and_the_last_run() {
    let (dir, project) = demo();
    std::fs::write(
        project.join("data/sales.csv"),
        "firm,year,sales\n1,2010,5\n1,2011,6\n2,2010,7\n2,2011,9\n",
    )
    .unwrap();
    // After a verb.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let long = import(\"data/sales.csv\")\nlong.where(year > 2010).add(k = sales * 2).mean(k)\nlong.where(year > 2010).mean(sale)\n",
    )
    .unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("error[Z0101]: no variable `sale` in `the frame`"),
        "{}",
        err(&o)
    );
    // `to_wide`: the run records its columns; the next check warns from last run and exits 0.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let long = import(\"data/sales.csv\")\nlet wide = long.to_wide(id: firm, names_from: year, values_from: sales)\nwide.mean(sales_2010)\nwide.mean(sales_2012)\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("error[Z0101]: no variable `sales_2012` in `wide`"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let text = err(&o);
    assert!(
        text.contains("warning[Z0101]: no variable `sales_2012` in `wide` (from last run)"),
        "{text}"
    );
    assert!(
        text.contains("the columns of the last run are used, marked from last run"),
        "{text}"
    );
    assert!(
        text.ends_with("scripts/main.zeff: 0 errors, 1 warning\n"),
        "{text}"
    );
    let o = zeff(&project, &["check", "--format", "json"]);
    assert_eq!(code(&o), 0);
    let json = out(&o);
    assert!(json.contains("\"mark\": \"checked at run time\""), "{json}");
    assert!(json.contains("\"mark\": \"from last run\""), "{json}");
    assert!(json.contains("\"severity\": \"warning\""), "{json}");
    // A data root's file that changes between runs.
    let shared = dir.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("r.csv"), "a,b\n1,2\n").unwrap();
    let manifest = std::fs::read_to_string(project.join("zeff.toml")).unwrap();
    std::fs::write(
        project.join("zeff.toml"),
        format!("{manifest}\n[data]\nlfs = {{ path = \"../shared\" }}\n"),
    )
    .unwrap();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let r = import(\"lfs:r.csv\")\nr.mean(a)\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(!err(&o).contains("W0701"), "{}", err(&o));
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    std::fs::write(shared.join("r.csv"), "a,b\n1,2\n3,4\n").unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        err(&o)
            .contains("warning[W0701]: `lfs:r.csv` has changed since the last run: it was read on"),
        "{}",
        err(&o)
    );
    assert_eq!(out(&o), "2\n");
    let o = zeff(&project, &["run"]);
    assert!(!err(&o).contains("has changed"), "{}", err(&o));
}

/// Pragmas and `--deny-warnings` through the program (Language Reference, section 11.1; the
/// exit codes of section 3.8): a warning refused is exit code 3 from `run` and from `check`, a
/// warning allowed passes and is listed by `--verbose` with its reason, and the option applies
/// to those two commands.
#[test]
fn deny_warnings_and_pragmas_through_the_program() {
    let (_dir, project) = demo();
    let main = project.join("scripts/main.zeff");

    // A warning of the run.
    std::fs::write(
        &main,
        "print(\"before\")\nlet a = log(0)\nprint(\"after\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "before\nafter\n");
    assert!(
        err(&o).starts_with("warning[W0204]: log of zero"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["run", "--deny-warnings"]);
    assert_eq!(code(&o), 3, "{}", err(&o));
    assert_eq!(out(&o), "before\n");
    insta::assert_snapshot!("run_deny_warnings", err(&o));
    // The check finds nothing: the warning is the data's.
    let o = zeff(&project, &["check", "--deny-warnings"]);
    assert_eq!(code(&o), 0, "{}", err(&o));

    // Allowed, the run passes, and `--verbose` lists what was silenced.
    std::fs::write(
        &main,
        "print(\"before\")\n// zeff: allow(W0204) a zero is a worker in the first year\nlet a = log(0)\nprint(\"after\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["--deny-warnings", "run", "-v"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "before\nafter\n");
    assert!(
        err(&o).contains(
            "allowed: W0204 at line 3, 1 warning: a zero is a worker in the first year\n"
        ),
        "{}",
        err(&o)
    );
    assert!(
        !err(&o).lines().any(|l| l.starts_with("warning")),
        "{}",
        err(&o)
    );

    // A warning of the check: reported as an error, in the text and in the records.
    std::fs::write(&main, "fn double(x: Float) -> Float { x * 2 }\ndouble(2)\n").unwrap();
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0);
    assert!(err(&o).contains("0 errors, 1 warning"), "{}", err(&o));
    let o = zeff(&project, &["check", "--deny-warnings"]);
    assert_eq!(code(&o), 3);
    insta::assert_snapshot!("check_deny_warnings", err(&o));
    let o = zeff(&project, &["check", "--deny-warnings", "--format", "json"]);
    assert_eq!(code(&o), 3);
    assert!(
        out(&o).contains("{\"code\": \"W0303\", \"severity\": \"error\""),
        "{}",
        out(&o)
    );
    // An error of the script beside it keeps exit code 1.
    std::fs::write(
        &main,
        "fn double(x: Float) -> Float { x * 2 }\ndouble(nothing)\n",
    )
    .unwrap();
    let o = zeff(&project, &["check", "--deny-warnings"]);
    assert_eq!(code(&o), 1);
    assert!(err(&o).contains("2 errors, 0 warnings"), "{}", err(&o));

    // A pragma that cannot be applied, and its page.
    std::fs::write(&main, "let a = log(0) // zeff: allow(W0204)\n").unwrap();
    let o = zeff(&project, &["run"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).starts_with("error[Z0317]: the pragma gives no reason"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["help", "Z0317"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        out(&o).starts_with("Z0317 — a pragma that cannot be applied"),
        "{}",
        out(&o)
    );

    // The option belongs to `run`, `check` and `test`.
    let o = zeff(&project, &["--deny-warnings", "eval", "1 + 1"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`--deny-warnings` applies to `zeff run`, `zeff check` and `zeff test`"),
        "{}",
        err(&o)
    );
}

// ----- zeff fmt (release 0.7.0, step 31b) -----

const UNTIDY: &str = "// Wages and schooling\nlet w=dataset(wages).where(year>=2010&&age>=18).add(lwage=log(wage),exper2=exper^2)   // the sample\n\n\n\nlet m=w.reg(lwage~educ+exper+exper2,fe:firm+year,cluster:firm,se:robust)\nlet means = w.aggregate(mean_lwage=mean(lwage),se=sd(lwage)/sqrt(n()),by:[industry,female])\n";

const TIDY: &str = "// Wages and schooling\nlet w = dataset(wages).where(year >= 2010 && age >= 18).add(lwage = log(wage), exper2 = exper^2)  // the sample\n\n\nlet m = w.reg(lwage ~ educ + exper + exper2, fe: firm + year, cluster: firm, se: robust)\nlet means = w.aggregate(mean_lwage = mean(lwage),\n                        se = sd(lwage) / sqrt(n()),\n                        by: [industry, female])\n";

/// A project with an untidy script, an untidy function, a script in a folder of `scripts/`
/// written with carriage returns, and the script of `zeff new`, which is tidy.
fn untidy() -> (tempfile::TempDir, std::path::PathBuf) {
    let (dir, project) = demo();
    std::fs::write(project.join("scripts/wages.zeff"), UNTIDY).unwrap();
    std::fs::create_dir_all(project.join("functions")).unwrap();
    std::fs::write(
        project.join("functions/twice.zeff"),
        "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > twice(2.0)\n/// 4\nfn twice(x:Float)->Float{x*2}\n",
    )
    .unwrap();
    std::fs::create_dir_all(project.join("scripts/old")).unwrap();
    std::fs::write(
        project.join("scripts/old/first.zeff"),
        "let a = 1  \r\nlet b = a + 1\r\n",
    )
    .unwrap();
    (dir, project)
}

#[test]
fn zeff_fmt_writes_every_script_of_the_project_in_the_one_style() {
    let (_dir, project) = untidy();
    let read = |name: &str| std::fs::read_to_string(project.join(name)).unwrap();
    let main = read("scripts/main.zeff");
    let o = zeff(&project, &["fmt"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "");
    assert_eq!(
        err(&o),
        "formatted functions/twice.zeff\nformatted scripts/old/first.zeff\n\
         formatted scripts/wages.zeff\nformatted 3 of 4 scripts\n"
    );
    assert_eq!(read("scripts/wages.zeff"), TIDY);
    assert_eq!(
        read("functions/twice.zeff"),
        "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > twice(2.0)\n/// 4\nfn twice(x: Float) -> Float { x * 2 }\n"
    );
    assert_eq!(read("scripts/old/first.zeff"), "let a = 1\nlet b = a + 1\n");
    assert_eq!(read("scripts/main.zeff"), main);
    // A second time there is nothing to do, from any folder of the project, or named.
    let o = zeff(&project.join("scripts"), &["fmt"]);
    assert_eq!(code(&o), 0);
    assert_eq!(err(&o), "4 scripts: formatted already\n");
    let parent = project.parent().unwrap();
    let o = zeff(parent, &["fmt", "--check", "--project", "demo"]);
    assert_eq!(code(&o), 0);
    assert_eq!(err(&o), "4 scripts: formatted\n");
    // The script still runs as it did: the formatter changed its spaces alone.
    let o = zeff(&project, &["check", "scripts/wages.zeff"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
}

#[test]
fn zeff_fmt_check_writes_nothing_and_exits_1_when_a_script_would_change() {
    let (_dir, project) = untidy();
    let o = zeff(&project, &["fmt", "--check"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    assert_eq!(
        err(&o),
        "would format functions/twice.zeff\nwould format scripts/old/first.zeff\n\
         would format scripts/wages.zeff\n\
         3 of 4 scripts would change: `zeff fmt` formats them\n"
    );
    assert_eq!(
        std::fs::read_to_string(project.join("scripts/wages.zeff")).unwrap(),
        UNTIDY
    );
    assert_eq!(
        std::fs::read(project.join("scripts/old/first.zeff")).unwrap(),
        b"let a = 1  \r\nlet b = a + 1\r\n"
    );
    // One script alone.
    std::fs::remove_file(project.join("functions/twice.zeff")).unwrap();
    std::fs::remove_file(project.join("scripts/old/first.zeff")).unwrap();
    let o = zeff(&project, &["fmt", "--check"]);
    assert_eq!(code(&o), 1);
    assert_eq!(
        err(&o),
        "would format scripts/wages.zeff\n1 of 2 scripts would change: `zeff fmt` formats it\n"
    );
    // The option is before or after the command.
    let o = zeff(&project, &["--check", "fmt"]);
    assert_eq!(code(&o), 1);
    // A project of one script.
    std::fs::remove_file(project.join("scripts/wages.zeff")).unwrap();
    let o = zeff(&project, &["fmt", "--check"]);
    assert_eq!(code(&o), 0);
    assert_eq!(err(&o), "1 script: formatted\n");
    let o = zeff(&project, &["fmt"]);
    assert_eq!(code(&o), 0);
    assert_eq!(err(&o), "1 script: formatted already\n");
    // And of none.
    std::fs::remove_file(project.join("scripts/main.zeff")).unwrap();
    for args in [&["fmt"][..], &["fmt", "--check"]] {
        let o = zeff(&project, args);
        assert_eq!(code(&o), 0);
        assert_eq!(
            err(&o),
            "no scripts to format: the project has no `.zeff` file in `scripts/` or `functions/`\n"
        );
    }
}

#[test]
fn zeff_fmt_leaves_a_script_that_does_not_parse_and_says_why() {
    let (_dir, project) = untidy();
    std::fs::write(project.join("scripts/broken.zeff"), "let a=1\nlet = 3\n").unwrap();
    let o = zeff(&project, &["fmt"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    insta::assert_snapshot!("fmt_with_a_script_that_does_not_parse", err(&o));
    // The script is as it was, and the others are formatted.
    assert_eq!(
        std::fs::read_to_string(project.join("scripts/broken.zeff")).unwrap(),
        "let a=1\nlet = 3\n"
    );
    assert_eq!(
        std::fs::read_to_string(project.join("scripts/wages.zeff")).unwrap(),
        TIDY
    );
    let o = zeff(&project, &["fmt", "--check"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).ends_with("4 scripts: formatted\n1 script does not parse and was left as it is\n"),
        "{}",
        err(&o)
    );
    // Two of them, and nothing else in the project.
    std::fs::write(project.join("scripts/main.zeff"), "x.\n").unwrap();
    std::fs::remove_file(project.join("scripts/wages.zeff")).unwrap();
    std::fs::remove_file(project.join("scripts/old/first.zeff")).unwrap();
    std::fs::remove_file(project.join("functions/twice.zeff")).unwrap();
    let o = zeff(&project, &["fmt"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).ends_with("\n2 scripts do not parse and were left as they are\n"),
        "{}",
        err(&o)
    );
    assert!(!err(&o).contains("formatted"), "{}", err(&o));
}

#[test]
fn zeff_fmt_says_what_it_could_not_read_or_write_or_vouch_for() {
    let (_dir, project) = untidy();
    // A file that is not text.
    std::fs::write(project.join("scripts/latin.zeff"), b"let caf\xe9 = 1\n").unwrap();
    let o = zeff(&project, &["fmt", "--check"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("error: cannot read scripts/latin.zeff: "),
        "{}",
        err(&o)
    );
    assert!(
        err(&o).ends_with("3 of 4 scripts would change: `zeff fmt` formats them\n"),
        "{}",
        err(&o)
    );
    std::fs::remove_file(project.join("scripts/latin.zeff")).unwrap();
    // A result the formatter cannot vouch for is not written: an internal error, exit code 2.
    let o = zeff_env(
        &project,
        "ZEFF_FMT_FORCE_INTERNAL",
        "scripts/wages.zeff",
        &["fmt"],
    );
    assert_eq!(code(&o), 2);
    assert!(
        err(&o).contains(
            "error[Z9001]: internal error: this is a bug in Zeff, not in your script\n  \
             the formatter left scripts/wages.zeff as it is: forced by ZEFF_FMT_FORCE_INTERNAL\n"
        ),
        "{}",
        err(&o)
    );
    assert!(
        err(&o).ends_with("formatted 2 of 3 scripts\n"),
        "{}",
        err(&o)
    );
    assert_eq!(
        std::fs::read_to_string(project.join("scripts/wages.zeff")).unwrap(),
        UNTIDY
    );
    // A script that cannot be written (where the system refuses the owner of a read-only
    // file, which it does not for an administrator).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = project.join("scripts/wages.zeff");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
        if std::fs::OpenOptions::new().write(true).open(&path).is_err() {
            let o = zeff(&project, &["fmt"]);
            assert_eq!(code(&o), 1);
            assert!(
                err(&o).starts_with("error: cannot write scripts/wages.zeff: "),
                "{}",
                err(&o)
            );
            assert!(
                err(&o).ends_with("3 scripts: formatted already\n"),
                "{}",
                err(&o)
            );
        }
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
}

#[test]
fn zeff_fmt_takes_no_script_and_check_is_its_option_alone() {
    let (_dir, project) = demo();
    let o = zeff(&project, &["fmt", "scripts/main.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).starts_with(
            "error: `zeff fmt` takes no script: it formats every script of the project\n"
        ),
        "{}",
        err(&o)
    );
    for command in [&["check", "--check"][..], &["run", "--check"], &["--check"]] {
        let o = zeff(&project, command);
        assert_eq!(code(&o), 1);
        assert!(
            err(&o).starts_with("error: `--check` applies to `zeff fmt`\n"),
            "{}",
            err(&o)
        );
    }
    let o = zeff(&project, &["fmt", "--format", "json"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`--format json` applies to"),
        "{}",
        err(&o)
    );
    // The usage text names the command and its option.
    let usage = out(&zeff(&project, &["help"]));
    assert!(
        usage.contains("\n  fmt               write every script of the project"),
        "{usage}"
    );
    assert!(
        usage.contains("\n  --check                 for `fmt`:"),
        "{usage}"
    );
}

// ----- zeff test (release 0.7.0, step 32b) -----

const TWICE: &str = "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > twice(2.5)\n/// 5\n/// > twice(4) + 1\n/// 9\nfn twice(x: Float) -> Float {\n    x * 2\n}\n";

const CHECKS: &str = "let w = dataset(wages)\nassert(w.n_rows() > 0, \"the wages file has rows\")\nassert(w.min(educ) >= 0, \"schooling is never negative\")\nassert(w.mean(wage) > 0, \"wages are positive on average\")\n";

/// A project whose tests pass: three checks in a script and two examples of a function.
fn tested() -> (tempfile::TempDir, std::path::PathBuf) {
    let (dir, project) = demo();
    std::fs::create_dir_all(project.join("functions")).unwrap();
    std::fs::write(project.join("functions/clean.zeff"), TWICE).unwrap();
    std::fs::write(project.join("scripts/checks.zeff"), CHECKS).unwrap();
    (dir, project)
}

#[test]
fn zeff_test_runs_the_checks_and_the_examples_of_the_project() {
    let (_dir, project) = tested();
    let o = zeff(&project, &["test"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "");
    assert_eq!(err(&o), "5 tests passed\n");
    // With `--verbose`, each test, and the scripts that hold no check.
    let o = zeff(&project, &["test", "--verbose"]);
    assert_eq!(code(&o), 0);
    assert_eq!(
        err(&o),
        "passed: scripts/checks.zeff, 3 checks\n\
         passed: functions/clean.zeff:7, example 1 of `twice`\n\
         passed: functions/clean.zeff:9, example 2 of `twice`\n\
         not run: scripts/main.zeff calls no `assert`\n\
         5 tests passed\n"
    );
    // One test.
    std::fs::remove_file(project.join("functions/clean.zeff")).unwrap();
    std::fs::write(
        project.join("scripts/checks.zeff"),
        "assert(1 + 1 == 2, \"arithmetic\")\n",
    )
    .unwrap();
    let o = zeff(&project, &["test", "-v"]);
    assert_eq!(code(&o), 0);
    assert!(
        err(&o).starts_with("passed: scripts/checks.zeff, 1 check\n")
            && err(&o).ends_with("1 test passed\n"),
        "{}",
        err(&o)
    );
    // None: the project of `zeff new`.
    let (_dir, project) = demo();
    let o = zeff(&project, &["test"]);
    assert_eq!(code(&o), 0);
    assert_eq!(
        err(&o),
        "no tests: no script of the project calls `assert`, and no function of `functions/` has an example\n"
    );
}

/// The same project with two checks that fail, an example that prints something else and a
/// script that stops on an error after its first check.
fn failing() -> (tempfile::TempDir, std::path::PathBuf) {
    let (dir, project) = tested();
    std::fs::write(
        project.join("functions/clean.zeff"),
        TWICE.replace("/// 9\n", "/// 10\n"),
    )
    .unwrap();
    std::fs::write(
        project.join("scripts/checks.zeff"),
        CHECKS
            .replace("w.n_rows() > 0", "w.n_rows() == 12")
            .replace("w.mean(wage) > 0", "w.mean(wage) > 1000"),
    )
    .unwrap();
    std::fs::write(
        project.join("scripts/later.zeff"),
        "assert(1 + 1 == 2, \"arithmetic\")\nerror(\"the data are not ready\")\nassert(false, \"never reached\")\n",
    )
    .unwrap();
    (dir, project)
}

#[test]
fn zeff_test_reports_every_failure_and_exits_1() {
    let (_dir, project) = failing();
    let o = zeff(&project, &["test"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "");
    let text = err(&o);
    // Both failed checks of the one script, the script that stopped, and the example.
    assert!(
        text.contains("failed: scripts/checks.zeff, 2 of 3 checks\n"),
        "{text}"
    );
    assert!(
        text.contains("stopped: scripts/later.zeff, after 1 check\n"),
        "{text}"
    );
    assert!(
        text.contains("failed: functions/clean.zeff:9, example 2 of `twice` prints something else than its page says\n"),
        "{text}"
    );
    assert!(text.ends_with("\n7 tests: 3 passed, 4 failed\n"), "{text}");
    insta::assert_snapshot!("test_with_failures", text);
    // `zeff run` is as it was: the first failed check stops the script.
    let o = zeff(&project, &["run", "scripts/checks.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("error[Z0001]: the wages file has rows")
            && !err(&o).contains("wages are positive on average"),
        "{}",
        err(&o)
    );
}

#[test]
fn zeff_test_writes_its_report_as_json() {
    let (_dir, project) = failing();
    let o = zeff(&project, &["test", "--format", "json"]);
    assert_eq!(code(&o), 1);
    assert_eq!(err(&o), "");
    insta::assert_snapshot!("test_json", out(&o));
    let (_dir, project) = tested();
    let o = zeff(&project, &["--format=json", "test"]);
    assert_eq!(code(&o), 0);
    assert!(
        out(&o).starts_with("{\n  \"tests\": 5,\n  \"failed\": 0,\n"),
        "{}",
        out(&o)
    );
    assert!(
        out(&o).contains("\"without_checks\": [\"scripts/main.zeff\"]"),
        "{}",
        out(&o)
    );
}

#[test]
fn zeff_test_under_deny_warnings_stops_a_script_at_its_first_warning() {
    let (_dir, project) = tested();
    std::fs::write(
        project.join("scripts/checks.zeff"),
        "let d = frame(x = [1, NA, 3])\nassert(d.n_rows() == 3, \"three rows\")\nlet m = d.mean(x)\nassert(m == 2, \"the mean of the values present\")\n",
    )
    .unwrap();
    // The warning is a warning, and the checks hold.
    let o = zeff(&project, &["test"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "4 tests passed\n");
    // Refused, it stops the script: exit code 3, since nothing else failed.
    let o = zeff(&project, &["test", "--deny-warnings"]);
    assert_eq!(code(&o), 3);
    let text = err(&o);
    assert!(
        text.starts_with("stopped: scripts/checks.zeff, after 1 check\n"),
        "{text}"
    );
    assert!(text.contains("W0201"), "{text}");
    assert!(text.ends_with("\n4 tests: 3 passed, 1 failed\n"), "{text}");
}

#[test]
fn zeff_test_takes_no_script_and_says_what_it_cannot_read() {
    let (_dir, project) = tested();
    let o = zeff(&project, &["test", "scripts/checks.zeff"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o)
            .starts_with("error: `zeff test` takes no script: it runs every test of the project\n"),
        "{}",
        err(&o)
    );
    std::fs::write(project.join("scripts/latin.zeff"), b"let caf\xe9 = 1\n").unwrap();
    let o = zeff(&project, &["test"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).starts_with("error: cannot read scripts/latin.zeff: "),
        "{}",
        err(&o)
    );
    assert!(err(&o).ends_with("\n5 tests passed\n"), "{}", err(&o));
    // The usage text names the command.
    let usage = out(&zeff(&project, &["help"]));
    assert!(
        usage.contains("\n  test              run the project's tests"),
        "{usage}"
    );
}

// ----- the run as it goes, and its interrupt (release 0.7.0, step 33b) -----

/// `zeff run` writes what a statement prints when the statement ends, and an interrupt
/// cancels the run between two steps: what was printed stays, and the error is Z0705.
#[cfg(unix)]
#[test]
fn zeff_run_prints_as_it_goes_and_an_interrupt_stops_it_between_steps() {
    use std::io::{BufRead, BufReader, Read};
    use std::process::Stdio;
    let (_dir, project) = demo();
    // A statement that prints, then a loop that would take minutes: few passes, so that its
    // list of items is small, each of them reading a dataset.
    std::fs::write(
        project.join("scripts/main.zeff"),
        "print(\"started\")\nfor i in 1..200000 {\n    let rows = dataset(wages).where(year == 2015).n_rows()\n}\nprint(\"never\")\n",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_zeff"))
        .arg("run")
        .current_dir(&project)
        .env_remove("NO_COLOR")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zeff starts");
    // The first statement's output arrives while the run is still going.
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    stdout.read_line(&mut first).unwrap();
    assert_eq!(first, "started\n");
    assert!(
        child.try_wait().unwrap().is_none(),
        "the run ended by itself"
    );
    let interrupted = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("kill runs");
    assert!(interrupted.success());
    let status = child.wait().unwrap();
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let mut messages = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut messages)
        .unwrap();
    assert_eq!(status.code(), Some(1), "{messages}");
    assert_eq!(rest, "");
    assert!(
        messages.starts_with("error[Z0705]: the run was cancelled before pass "),
        "{messages}"
    );
    assert!(messages.contains(" --> scripts/main.zeff:2:"), "{messages}");
}

#[test]
fn only_zeff_run_is_cancelled_by_an_interrupt() {
    let args = |words: &[&str]| -> Vec<String> { words.iter().map(|w| w.to_string()).collect() };
    for words in [
        &["run"][..],
        &["run", "scripts/clean.zeff"],
        &["--project", "demo", "run"],
        &["-q", "run", "--deny-warnings"],
    ] {
        assert!(zeff_cli::interruptible(&args(words)), "{words:?}");
    }
    for words in [
        &["repl"][..],
        &["check"],
        &["test"],
        &["eval", "run"],
        &["help", "run"],
        &[],
        &["--no-such-option", "run"],
    ] {
        assert!(!zeff_cli::interruptible(&args(words)), "{words:?}");
    }
}

// ----- the memory budget (release 0.7.0, step 33c) -----

#[test]
fn zeff_run_under_a_memory_budget_refuses_a_statement_that_would_pass_it() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "print(\"start\")\nlet w = import(\"zeff:wages.csv\")\nprint(w.n_rows())\n",
    )
    .unwrap();
    // No budget: the run it was.
    let o = zeff(&project, &["run", "-q"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), "start\n18412\n");
    // `--memory`: the file counts five times its 875 kB, which 2 MB does not hold.
    let o = zeff(&project, &["run", "-q", "--memory", "2MB"]);
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o), "start\n");
    insta::assert_snapshot!("run_over_the_memory_budget", err(&o));
    let o = zeff(&project, &["--memory=64MB", "run", "-q"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    // `memory` of the project, and the option over it.
    let manifest = std::fs::read_to_string(project.join("zeff.toml")).unwrap();
    std::fs::write(
        project.join("zeff.toml"),
        manifest.replace("[project]\n", "[project]\nmemory = \"2MB\"\n"),
    )
    .unwrap();
    let o = zeff(&project, &["run", "-q"]);
    assert_eq!(code(&o), 1, "{}", err(&o));
    assert!(
        err(&o).contains("more than the budget of 2.0 MB"),
        "{}",
        err(&o)
    );
    let o = zeff(&project, &["run", "-q", "--memory", "1GB"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    // The explained page.
    let o = zeff(&project, &["help", "Z0704"]);
    assert_eq!(code(&o), 0);
    assert!(
        out(&o).starts_with("Z0704 — a step would pass the memory budget\n"),
        "{}",
        out(&o)
    );
}

#[test]
fn memory_takes_a_size() {
    let (_dir, project) = demo();
    for (args, said) in [
        (
            &["run", "--memory", "2gb"][..],
            "`--memory 2gb` is not a size: write a number and KB, MB or GB, as in `--memory 2GB`",
        ),
        (
            &["run", "--memory", "lots"],
            "`--memory lots` is not a size",
        ),
        (&["run", "--memory", "0GB"], "`--memory 0GB` is not a size"),
        (&["run", "--memory"], "`--memory` needs a value: a size"),
    ] {
        let o = zeff(&project, args);
        assert_eq!(code(&o), 1, "{args:?}");
        assert!(err(&o).contains(said), "{args:?}: {}", err(&o));
    }
    let usage = out(&zeff(&project, &["help"]));
    assert!(
        usage.contains("\n  --memory <size>         the memory budget of the run"),
        "{usage}"
    );
}

// ----- zeff lsp -----

/// Messages of the Language Server Protocol, each with its header, one after another.
fn lsp_messages(bodies: &[String]) -> String {
    bodies
        .iter()
        .map(|body| format!("Content-Length: {}\r\n\r\n{body}", body.len()))
        .collect()
}

/// A text as JSON writes it.
fn json_text(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The language server as an editor starts it: the conversation on standard input, the
/// answers on standard output, each with its header; diagnostics for a script as it is typed,
/// not as it is saved; and the exit codes the protocol names.
#[test]
fn zeff_lsp_serves_an_editor_on_standard_input_and_output() {
    let (dir, project) = demo();
    let uri = format!("file://{}/scripts/main.zeff", project.display());
    let saved = std::fs::read_to_string(project.join("scripts/main.zeff")).unwrap();
    let open = |text: &str| {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{{\"textDocument\":{{\"uri\":{},\"languageId\":\"zeff\",\"version\":1,\"text\":{}}}}}}}",
            json_text(&uri),
            json_text(text)
        )
    };
    let change = |version: u32, text: &str| {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didChange\",\"params\":{{\"textDocument\":{{\"uri\":{},\"version\":{version}}},\"contentChanges\":[{{\"text\":{}}}]}}}}",
            json_text(&uri),
            json_text(text)
        )
    };
    let initialize =
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"capabilities\":{}}}"
            .to_string();
    let conversation = [
        initialize.clone(),
        "{\"jsonrpc\":\"2.0\",\"method\":\"initialized\",\"params\":{}}".to_string(),
        open(&saved),
        change(2, "let s = \"😀\"; print(1)\n"),
        change(3, "let d = dataset(wages)\nd.mean(wagee)\n"),
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"shutdown\"}".to_string(),
        "{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}".to_string(),
    ];
    let o = zeff_with(&project, dir.path(), &lsp_messages(&conversation), &["lsp"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "");
    let said = out(&o);
    // Five answers, each under its header: the capabilities, the diagnostics of the script
    // three times, and the answer to `shutdown`.
    assert_eq!(said.matches("Content-Length: ").count(), 5, "{said}");
    assert!(said.contains("\"positionEncoding\":\"utf-16\""), "{said}");
    assert!(
        said.contains(&format!(
            "\"serverInfo\":{{\"name\":\"zeff\",\"version\":\"{VERSION}\"}}"
        )),
        "{said}"
    );
    // The script the project was made with has nothing said of it.
    assert!(
        said.contains(&format!(
            "\"params\":{{\"diagnostics\":[],\"uri\":{},\"version\":1}}",
            json_text(&uri)
        )),
        "{said}"
    );
    // The semicolon after the face, at the column the protocol counts: 12, not 11 and not 14.
    assert!(
        said.contains("\"code\":\"Z0315\"")
            && said.contains("\"range\":{\"end\":{\"character\":13,\"line\":0},\"start\":{\"character\":12,\"line\":0}}"),
        "{said}"
    );
    // The misspelt variable, with the suggestion, though the file on disk was never changed.
    assert!(
        said.contains("\"code\":\"Z0101\",\"message\":\"no variable `wagee` in `d`\\nhelp: did you mean `wage`?"),
        "{said}"
    );
    assert!(
        said.ends_with("{\"id\":2,\"jsonrpc\":\"2.0\",\"result\":null}"),
        "{said}"
    );
    assert_eq!(
        std::fs::read_to_string(project.join("scripts/main.zeff")).unwrap(),
        saved
    );

    // The editor goes away without `exit`: what came is answered, and the program ends with 1.
    let o = zeff_with(
        &project,
        dir.path(),
        &lsp_messages(&conversation[..3]),
        &["lsp"],
    );
    assert_eq!(code(&o), 1);
    assert_eq!(out(&o).matches("Content-Length: ").count(), 2);
    // `exit` without `shutdown` before it ends with 1 too.
    let hasty = [initialize, conversation[6].clone()];
    let o = zeff_with(&project, dir.path(), &lsp_messages(&hasty), &["lsp"]);
    assert_eq!(code(&o), 1);
    // Bytes that are no messages are answered with an error of the protocol, not with a
    // failure of the program.
    let o = zeff_with(
        &project,
        dir.path(),
        "hello\r\n\r\nContent-Length: 3\r\n\r\n{{{",
        &["lsp"],
    );
    assert_eq!(code(&o), 1);
    assert_eq!(err(&o), "");
    assert!(
        out(&o).contains("{\"error\":{\"code\":-32700,\"message\":\"a header line without `:`\"},\"id\":null,\"jsonrpc\":\"2.0\"}"),
        "{}",
        out(&o)
    );
    assert!(
        out(&o).contains("\"message\":\"the message is not JSON\""),
        "{}",
        out(&o)
    );
    // Outside any project it serves all the same: a file is checked alone.
    let alone = [
        conversation[0].clone(),
        "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"untitled:Untitled-1\",\"version\":1,\"text\":\"print(nope)\\n\"}}}".to_string(),
        conversation[5].clone(),
        conversation[6].clone(),
    ];
    let o = zeff_with(dir.path(), dir.path(), &lsp_messages(&alone), &["lsp"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(
        out(&o).contains("\"code\":\"Z0101\",\"message\":\"nothing is called `nope` here\""),
        "{}",
        out(&o)
    );
    // An editor that starts its servers with `--stdio` is served the same.
    let o = zeff_with(
        &project,
        dir.path(),
        &lsp_messages(&conversation),
        &["lsp", "--stdio"],
    );
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(out(&o), said);
    let o = zeff(&project, &["check", "--stdio"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`--stdio` applies to `zeff lsp`"),
        "{}",
        err(&o)
    );
    // It takes no arguments.
    let o = zeff(&project, &["lsp", "now"]);
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("`zeff lsp` takes no arguments: an editor starts it and talks to it"),
        "{}",
        err(&o)
    );
}
