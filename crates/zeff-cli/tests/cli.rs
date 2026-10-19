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
    assert!(text.starts_with("[\n  {\"code\": \"Z0404\""), "{text}");
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
            "applies to `zeff check`, `zeff version` and `zeff selftest`",
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
    let o = zeff(&project, &["check"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert_eq!(err(&o), "checked scripts/main.zeff: no problems\n");
}

#[test]
fn zeff_check_reports_every_problem_and_exits_1() {
    let (_dir, project) = demo();
    std::fs::write(
        project.join("scripts/main.zeff"),
        "let wages = import(\"data/wages.csv\", heade: false)\nwges.maen(wage)\nlet mean = 3\nwages.mean(\"wage\")\nfor i in [1] {\n    print(i)\n}\n",
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
    for later in ["lsp", "doctor", "engine", "verify"] {
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
        "note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)\nwarning: 1 row with NA in hours was skipped (line 2)\n"
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
        err(&o).contains("\nerror[Z0101]: no variable `wge` in `wages`"),
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
}

#[test]
fn zeff_help_lists_a_package_and_says_when_a_function_arrives() {
    let dir = tempfile::tempdir().unwrap();
    let o = zeff(dir.path(), &["help", "data"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    insta::assert_snapshot!("help_data", out(&o));
    let core = zeff(dir.path(), &["help", "core"]);
    assert!(
        out(&core).starts_with("core · tier 0 · 37 of 68 functions in this release"),
        "{}",
        out(&core)
    );
    let later = zeff(dir.path(), &["help", "reg"]);
    assert_eq!(code(&later), 0);
    assert!(
        out(&later).starts_with("`reg` arrives with release 0.4.0, in the `regress` package: "),
        "{}",
        out(&later)
    );
    let two_homes = zeff(dir.path(), &["help", "append"]);
    assert_eq!(
        out(&two_homes).lines().count(),
        2,
        "one line per home: {}",
        out(&two_homes)
    );
    // A name shipped in one package and planned in another: the page, then where the other arrives.
    let shipped_and_later = zeff(dir.path(), &["help", "summary"]);
    assert!(
        out(&shipped_and_later).starts_with("summary — Table of N, missing, mean, SD, min and max"),
        "{}",
        out(&shipped_and_later)
    );
    assert!(
        out(&shipped_and_later).ends_with(
            "\n`summary` arrives with release 0.4.0, in the `results` package: Full printed summary of a model or test\n"
        ),
        "{}",
        out(&shipped_and_later)
    );
    let package_later = zeff(dir.path(), &["help", "regress"]);
    assert!(
        out(&package_later)
            .contains("has no function in this release; its first arrives with release 0.4.0"),
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
    for name in ["first_engine", "data_cleaning"] {
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
        err(&o).contains("warning: 1 row with NA in hours"),
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
        err(&o).contains("\x1b[1;33mwarning\x1b[0m: 1 row"),
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
        "warning: 1 row with NA in hours was skipped (line 2)\n"
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
