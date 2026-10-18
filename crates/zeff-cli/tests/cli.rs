//! Script tests of the real `zeff` program: arguments in, standard output, standard error and the
//! exit code out, in a temporary folder.

use std::path::Path;
use std::process::{Command, Output};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn zeff(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zeff"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("zeff runs")
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
    for later in ["lsp", "repl", "selftest", "engine", "verify"] {
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
    assert_eq!(err(&o), "");
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
        "warning: 1 row with NA in hours was skipped (line 2)\n"
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
