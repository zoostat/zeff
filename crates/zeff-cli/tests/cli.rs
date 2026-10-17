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
    // The project is found and accepted; the command itself arrives in a later step.
    assert_eq!(code(&o), 1);
    assert!(
        err(&o).contains("not available in this release yet"),
        "{}",
        err(&o)
    );
    assert!(!err(&o).contains("Z0709"));
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
