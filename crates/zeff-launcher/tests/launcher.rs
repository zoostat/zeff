//! The launcher on a temporary engine store (Engine and Project Specification, section 2.1;
//! release 0.9.0, step 17): two engines side by side, each a small program that says which it
//! is and what it was given; two projects, each pinned to one; the default engine outside a
//! project; the command line, the streams and the exit code handed over unchanged; and Z0710,
//! exit code 4, for an engine that is not there. The choices are also tested as functions, for
//! the names a Windows machine uses.

use std::path::{Path, PathBuf};
use zeff_launcher::{
    Choice, Platform, Store, Version, choose, default_setting, engine_in, find_engine, installed,
    manifest_above, pin, project_argument, refusal, store,
};

/// A store with the engines of `versions`, each a file that is there and no more.
fn store_with(root: &Path, versions: &[&str], platform: Platform) {
    for version in versions {
        let engine = engine_in(root, version, platform);
        std::fs::create_dir_all(engine.parent().unwrap()).unwrap();
        std::fs::write(&engine, "").unwrap();
    }
}

/// A project folder whose `zeff.toml` pins `version`.
fn project(root: &Path, name: &str, version: &str) -> PathBuf {
    let dir = root.join(name);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(
        dir.join("zeff.toml"),
        format!("[project]\nname = \"{name}\"\nzeff = \"{version}\"\nedition = \"preview\"\n"),
    )
    .unwrap();
    dir
}

fn env_of(pairs: &[(&str, &Path)]) -> impl Fn(&str) -> Option<String> + use<> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(name, value)| ((*name).to_string(), value.display().to_string()))
        .collect();
    move |name| {
        pairs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, value)| value.clone())
    }
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

// ----- the pieces, as functions -----

#[test]
fn versions_are_read_and_ordered() {
    let v = |text: &str| Version::parse(text).unwrap_or_else(|| panic!("{text}"));
    assert_eq!(v("0.9.1").numbers, [0, 9, 1]);
    assert_eq!(v("0.9.0-dev").pre.as_deref(), Some("dev"));
    for not in [
        "0.9", "0.9.x", "^0.9.1", "0.9.1.2", "", "0.9.-1", "0.9.1-", "v0.9.1", "0..1",
    ] {
        assert!(Version::parse(not).is_none(), "{not}");
    }
    // By their numbers, not their letters; a version before its release before the release.
    let mut sorted = [
        v("0.10.0"),
        v("0.9.1"),
        v("0.9.0"),
        v("0.9.0-dev"),
        v("1.0.0"),
    ];
    sorted.sort();
    let texts: Vec<&str> = sorted.iter().map(|v| v.text.as_str()).collect();
    assert_eq!(texts, ["0.9.0-dev", "0.9.0", "0.9.1", "0.10.0", "1.0.0"]);
}

#[test]
fn the_store_is_the_users_folder_first_and_then_the_systems() {
    let home = PathBuf::from("/home/ada");
    let own = store(&env_of(&[("HOME", &home)]), Platform::Unix).unwrap();
    assert_eq!(own.user, home.join(".zeff"));
    assert_eq!(own.system, Some(PathBuf::from("/opt/zeff")));
    // ZEFF_HOME names the folder itself, and an empty one is as none.
    let elsewhere = PathBuf::from("/data/zeff");
    let named = store(
        &env_of(&[("HOME", &home), ("ZEFF_HOME", &elsewhere)]),
        Platform::Unix,
    );
    assert_eq!(named.unwrap().user, elsewhere);
    let empty = store(
        &env_of(&[("HOME", &home), ("ZEFF_HOME", Path::new(""))]),
        Platform::Unix,
    );
    assert_eq!(empty.unwrap().user, home.join(".zeff"));
    // A shared install is its own home: the system store is not searched twice.
    let shared = store(
        &env_of(&[("ZEFF_HOME", Path::new("/opt/zeff"))]),
        Platform::Unix,
    );
    assert_eq!(shared.unwrap().system, None);
    // No home at all.
    assert!(store(&env_of(&[]), Platform::Unix).is_none());
}

#[test]
fn a_windows_machine_has_its_own_names() {
    assert_eq!(Platform::Windows.engine_file(), "zeff.exe");
    assert_eq!(Platform::Unix.engine_file(), "zeff");
    let profile = PathBuf::from("C:\\Users\\Ada");
    let programs = PathBuf::from("C:\\Program Files");
    let found = store(
        &env_of(&[
            ("USERPROFILE", &profile),
            ("ProgramFiles", &programs),
            ("HOME", Path::new("/ignored")),
        ]),
        Platform::Windows,
    )
    .unwrap();
    assert_eq!(found.user, profile.join(".zeff"));
    assert_eq!(found.system, Some(programs.join("Zeff")));
    // Without a programs folder there is no system store; without a profile, no store.
    let alone = store(&env_of(&[("USERPROFILE", &profile)]), Platform::Windows).unwrap();
    assert_eq!(alone.system, None);
    assert!(
        store(
            &env_of(&[("HOME", Path::new("/home/ada"))]),
            Platform::Windows
        )
        .is_none()
    );
    // The engine's file in its folder.
    let engine = engine_in(&found.user, "0.9.1", Platform::Windows);
    assert_eq!(engine.file_name().unwrap(), "zeff.exe");
    assert_eq!(engine.parent().unwrap().file_name().unwrap(), "0.9.1");
    // A store of Windows engines, found by their own file name.
    let dir = tempfile::tempdir().unwrap();
    store_with(dir.path(), &["0.9.0", "0.9.1"], Platform::Windows);
    let here = Store {
        user: dir.path().to_path_buf(),
        system: None,
    };
    assert_eq!(installed(&here, Platform::Windows).len(), 2);
    assert!(installed(&here, Platform::Unix).is_empty());
    assert!(find_engine(&here, "0.9.1", Platform::Windows).is_some());
}

#[test]
fn the_project_is_the_folder_named_or_the_nearest_above() {
    assert_eq!(
        project_argument(&strings(&["run", "-p", "a/b"])),
        Some(PathBuf::from("a/b"))
    );
    assert_eq!(
        project_argument(&strings(&["--project", "x", "run"])),
        Some(PathBuf::from("x"))
    );
    assert_eq!(
        project_argument(&strings(&["run", "--project=y z"])),
        Some(PathBuf::from("y z"))
    );
    assert_eq!(project_argument(&strings(&["run"])), None);
    // After `--` nothing is an option; a `-p` with nothing after it names nothing.
    assert_eq!(project_argument(&strings(&["eval", "--", "-p", "x"])), None);
    assert_eq!(project_argument(&strings(&["run", "-p"])), None);
    let dir = tempfile::tempdir().unwrap();
    let root = project(dir.path(), "study", "0.9.1");
    let deep = root.join("scripts").join("more");
    std::fs::create_dir_all(&deep).unwrap();
    assert_eq!(manifest_above(&deep), Some(root.join("zeff.toml")));
    assert_eq!(manifest_above(&root), Some(root.join("zeff.toml")));
    assert_eq!(manifest_above(dir.path()), None);
}

#[test]
fn the_pin_and_the_default_are_read_as_toml() {
    assert_eq!(
        pin("[project]\nname = \"p\"\nzeff = \"0.9.1\"\n").as_deref(),
        Some("0.9.1")
    );
    assert_eq!(
        pin("# a comment\n[project]\nzeff = '0.9.0-dev' # exact\n").as_deref(),
        Some("0.9.0-dev")
    );
    assert_eq!(pin("project.zeff = \"0.9.2\"\n").as_deref(), Some("0.9.2"));
    // Nothing readable: the engine reports the file's faults in full.
    assert_eq!(pin("[project]\nname = \"p\"\n"), None);
    assert_eq!(pin("[project]\nzeff = 0.9\n"), None);
    assert_eq!(pin("[other]\nzeff = \"0.9.1\"\n"), None);
    assert_eq!(pin("not toml ["), None);
    assert_eq!(
        default_setting("[engine]\ndefault = \"0.9.0\"\n").as_deref(),
        Some("0.9.0")
    );
    assert_eq!(default_setting("[colour]\nuse = \"never\"\n"), None);
}

#[test]
fn the_engine_is_the_pinned_one_or_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    store_with(&home, &["0.9.0", "0.9.1", "0.10.0-dev"], Platform::Unix);
    let env = env_of(&[("ZEFF_HOME", &home)]);
    let first = project(dir.path(), "first", "0.9.0");
    let second = project(dir.path(), "second", "0.9.1");
    let ran = |args: &[&str], cwd: &Path| match choose(&strings(args), cwd, &env, Platform::Unix) {
        Choice::Run { version, engine } => {
            assert_eq!(engine, engine_in(&home, &version, Platform::Unix));
            version
        }
        other => panic!("{other:?}"),
    };
    // Each project on its own engine, from inside it, from a folder of it, and by name.
    assert_eq!(ran(&["run"], &first), "0.9.0");
    assert_eq!(ran(&["run"], &second), "0.9.1");
    assert_eq!(ran(&["run"], &second.join("scripts")), "0.9.1");
    assert_eq!(ran(&["run", "-p", "first"], dir.path()), "0.9.0");
    assert_eq!(
        ran(&["run", &format!("--project={}", second.display())], &first),
        "0.9.1"
    );
    // Outside a project: the newest release installed, a version before its release last.
    assert_eq!(ran(&["help"], dir.path()), "0.9.1");
    // The user's setting chooses another.
    std::fs::write(home.join("config.toml"), "[engine]\ndefault = \"0.9.0\"\n").unwrap();
    assert_eq!(ran(&["help"], dir.path()), "0.9.0");
    // A project whose file pins nothing readable goes to the default, which reports it.
    let broken = dir.path().join("broken");
    std::fs::create_dir_all(&broken).unwrap();
    std::fs::write(broken.join("zeff.toml"), "[project\n").unwrap();
    assert_eq!(ran(&["run"], &broken), "0.9.0");
}

#[test]
fn a_release_is_the_default_before_a_version_not_yet_released() {
    let dir = tempfile::tempdir().unwrap();
    store_with(dir.path(), &["0.9.0-dev"], Platform::Unix);
    let env = env_of(&[("ZEFF_HOME", dir.path())]);
    // Alone, it is the newest there is.
    assert!(matches!(
        choose(&strings(&["help"]), dir.path(), &env, Platform::Unix),
        Choice::Run { version, .. } if version == "0.9.0-dev"
    ));
}

#[test]
fn the_users_engine_comes_before_the_systems() {
    let dir = tempfile::tempdir().unwrap();
    let (user, system) = (dir.path().join("user"), dir.path().join("system"));
    store_with(&user, &["0.9.1"], Platform::Unix);
    store_with(&system, &["0.9.0", "0.9.1"], Platform::Unix);
    let both = Store {
        user: user.clone(),
        system: Some(system.clone()),
    };
    assert_eq!(
        find_engine(&both, "0.9.1", Platform::Unix),
        Some(engine_in(&user, "0.9.1", Platform::Unix))
    );
    assert_eq!(
        find_engine(&both, "0.9.0", Platform::Unix),
        Some(engine_in(&system, "0.9.0", Platform::Unix))
    );
    assert_eq!(find_engine(&both, "0.8.0", Platform::Unix), None);
    let texts: Vec<String> = installed(&both, Platform::Unix)
        .into_iter()
        .map(|v| v.text)
        .collect();
    assert_eq!(texts, ["0.9.0", "0.9.1"]);
    // A folder that is no version, or a version's folder without its engine, is not one.
    std::fs::create_dir_all(user.join("engines").join("notes")).unwrap();
    std::fs::create_dir_all(user.join("engines").join("0.9.2")).unwrap();
    assert_eq!(installed(&both, Platform::Unix).len(), 2);
}

#[test]
fn every_refusal_says_what_to_do() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let said = |choice: &Choice| {
        refusal(choice)
            .expect("a refusal")
            .render(&zeff_diagnostics::Sources::new())
            .replace(&dir.path().display().to_string(), "<tmp>")
    };
    let mut out = String::new();
    // No engine at all; then one, and a project that pins another; two, and a third; a default
    // that is not there; no home.
    let env = env_of(&[("ZEFF_HOME", &home)]);
    let outside = choose(&strings(&["help"]), dir.path(), &env, Platform::Unix);
    out.push_str(&said(&outside));
    let study = project(dir.path(), "study", "0.9.2");
    out.push_str(&said(&choose(
        &strings(&["run"]),
        &study,
        &env,
        Platform::Unix,
    )));
    store_with(&home, &["0.9.0"], Platform::Unix);
    out.push_str(&said(&choose(
        &strings(&["run"]),
        &study,
        &env,
        Platform::Unix,
    )));
    store_with(&home, &["0.9.1", "0.10.0"], Platform::Unix);
    out.push_str(&said(&choose(
        &strings(&["run"]),
        &study,
        &env,
        Platform::Unix,
    )));
    std::fs::write(home.join("config.toml"), "[engine]\ndefault = \"0.8.0\"\n").unwrap();
    out.push_str(&said(&choose(
        &strings(&["help"]),
        dir.path(),
        &env,
        Platform::Unix,
    )));
    out.push_str(&said(&choose(
        &strings(&["help"]),
        dir.path(),
        &env_of(&[]),
        Platform::Unix,
    )));
    assert!(
        refusal(&Choice::Run {
            version: "0.9.0".to_string(),
            engine: PathBuf::new()
        })
        .is_none()
    );
    insta::assert_snapshot!(out);
}

// ----- the launcher itself, handing a command to an engine -----

#[cfg(unix)]
mod handed_over {
    use super::project;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::{Command, Output, Stdio};

    /// An engine that says which it is, the launcher that started it and what it was given, one
    /// argument to a line; copies its input to its output when its first argument is `echo`;
    /// writes to its error stream when it is `complain`; and ends with the code that follows
    /// `exit`.
    fn engine(home: &Path, version: &str) {
        let dir = home.join("engines").join(version);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("zeff");
        std::fs::write(
            &file,
            format!(
                "#!/bin/sh\necho \"engine {version} under launcher $ZEFF_LAUNCHER\"\nfor a in \"$@\"; do echo \"arg: $a\"; done\ncase \"$1\" in\n  echo) cat ;;\n  complain) echo \"said on the error stream\" >&2 ;;\n  exit) exit \"$2\" ;;\nesac\n"
            ),
        )
        .unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn launch(home: &Path, cwd: &Path, args: &[&str], input: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_zeff-launcher"))
            .args(args)
            .current_dir(cwd)
            .env("ZEFF_HOME", home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the launcher starts");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    #[test]
    fn two_projects_each_run_on_the_engine_they_pin() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        engine(&home, "0.9.0");
        engine(&home, "0.9.1");
        let first = project(dir.path(), "first", "0.9.0");
        let second = project(dir.path(), "second", "0.9.1");
        let launcher = format!("under launcher {}", zeff_launcher::VERSION);
        let o = launch(&home, &first, &["run", "scripts/main.zeff"], "");
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(
            text(&o.stdout),
            format!("engine 0.9.0 {launcher}\narg: run\narg: scripts/main.zeff\n")
        );
        assert_eq!(text(&o.stderr), "");
        let o = launch(&home, &second, &["run"], "");
        assert_eq!(
            text(&o.stdout),
            format!("engine 0.9.1 {launcher}\narg: run\n")
        );
        // From elsewhere, by name; and outside a project, the newest.
        let o = launch(&home, dir.path(), &["check", "--project", "first"], "");
        assert!(
            text(&o.stdout).starts_with("engine 0.9.0 "),
            "{}",
            text(&o.stdout)
        );
        let o = launch(&home, dir.path(), &["help", "mean"], "");
        assert!(
            text(&o.stdout).starts_with("engine 0.9.1 "),
            "{}",
            text(&o.stdout)
        );
    }

    #[test]
    fn the_arguments_the_streams_and_the_exit_code_are_the_engines_own() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        engine(&home, "0.9.1");
        // Arguments with spaces, quotes and an empty one arrive as they were given.
        let o = launch(
            &home,
            dir.path(),
            &["eval", "1 + 1", "", "it's \"quoted\"", "--", "-p"],
            "",
        );
        assert_eq!(
            text(&o.stdout).lines().skip(1).collect::<Vec<_>>(),
            [
                "arg: eval",
                "arg: 1 + 1",
                "arg: ",
                "arg: it's \"quoted\"",
                "arg: --",
                "arg: -p"
            ]
        );
        // What is typed reaches the engine; what it says on each stream comes back on that one.
        let o = launch(&home, dir.path(), &["echo"], "typed in\nand more\n");
        assert!(
            text(&o.stdout).ends_with("arg: echo\ntyped in\nand more\n"),
            "{}",
            text(&o.stdout)
        );
        let o = launch(&home, dir.path(), &["complain"], "");
        assert_eq!(text(&o.stderr), "said on the error stream\n");
        assert_eq!(o.status.code(), Some(0));
        // Every exit code of section 3.8, and one that is the engine's alone.
        for code in ["0", "1", "2", "3", "4", "42"] {
            let o = launch(&home, dir.path(), &["exit", code], "");
            assert_eq!(o.status.code(), code.parse().ok(), "exit {code}");
        }
    }

    #[test]
    fn an_engine_that_is_not_there_is_exit_code_4() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        engine(&home, "0.9.0");
        let study = project(dir.path(), "study", "0.9.1");
        let o = launch(&home, &study, &["run"], "");
        assert_eq!(o.status.code(), Some(4));
        assert_eq!(text(&o.stdout), "");
        let said = text(&o.stderr);
        assert!(
            said.starts_with(
                "error[Z0710]: this project needs Zeff 0.9.1, which is not installed\n"
            ),
            "{said}"
        );
        assert!(
            said.contains("`zeff engine install 0.9.1` installs it"),
            "{said}"
        );
        assert!(said.contains("the engine installed is 0.9.0"), "{said}");
        // No engine at all, outside a project.
        let empty = dir.path().join("empty");
        let o = launch(&empty, dir.path(), &["help"], "");
        assert_eq!(o.status.code(), Some(4));
        assert!(text(&o.stderr).starts_with("error[Z0710]: no Zeff engine is installed\n"));
        // An engine whose file cannot be run.
        let file = home.join("engines").join("0.9.0").join("zeff");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let o = launch(&home, dir.path(), &["help"], "");
        assert_eq!(o.status.code(), Some(4));
        assert!(
            text(&o.stderr)
                .starts_with("error[Z0710]: the engine Zeff 0.9.0 could not be started: "),
            "{}",
            text(&o.stderr)
        );
    }
}
