//! `zeff engine` (Engine and Project Specification, section 2.1, "Engines"; release 0.9.0, step
//! 19a): an engine installed from an archive on this machine, listed, made the default and
//! removed, on a temporary store; every message kept as a snapshot.

mod common;

use common::{Signer, release};
use std::path::{Path, PathBuf};
use zeff_launcher::archive::Target;
use zeff_launcher::engines::{
    Asked, Default, Machine, default_of, read, run, size_text, with_default,
};
use zeff_launcher::{Platform, Store, default_setting};

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

/// What a command wrote and how it ended.
#[derive(Debug)]
struct Ended {
    code: u8,
    out: String,
    err: String,
}

/// A temporary store, a folder of downloads and a key.
struct World {
    dir: tempfile::TempDir,
    signer: Signer,
    host: Target,
}

impl World {
    fn new() -> World {
        World {
            dir: tempfile::tempdir().unwrap(),
            signer: Signer::new(),
            host: Target::LinuxX86_64,
        }
    }

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    fn store(&self) -> Store {
        Store {
            user: self.home(),
            system: None,
        }
    }

    /// The archive of `version`, signed, in a folder of its own; its path.
    fn download(&self, version: &str) -> PathBuf {
        release(
            &self.dir.path().join("downloads").join(version),
            version,
            self.host,
            &self.signer,
        )
    }

    /// `zeff <args>` with the test's key as the launcher's, the temporary folder's name
    /// written as `<tmp>`.
    fn zeff_with(&self, args: &[&str], key: Option<&str>, system: Option<&Path>) -> Ended {
        let args = strings(args);
        let asked = read(&args).expect("an engine command");
        let home = self.home().display().to_string();
        let system = system.map(|path| path.display().to_string());
        let env = move |name: &str| match name {
            "ZEFF_HOME" => Some(home.clone()),
            // On Windows the system-wide store is `Zeff` under the programs folder.
            "ProgramFiles" => system.clone(),
            _ => None,
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = match asked {
            Ok(asked) => run(
                &asked,
                &Machine {
                    env: &env,
                    cwd: self.dir.path(),
                    platform: if system_given(&env) {
                        Platform::Windows
                    } else {
                        Platform::Unix
                    },
                    host: Some(self.host),
                    key,
                    fetch: None,
                },
                &mut out,
                &mut err,
            ),
            Err(wrong) => zeff_launcher::engines::usage(&mut err, &wrong),
        };
        let clean = |bytes: Vec<u8>| {
            String::from_utf8(bytes)
                .unwrap()
                .replace(&self.dir.path().display().to_string(), "<tmp>")
                .replace('\\', "/")
        };
        Ended {
            code,
            out: clean(out),
            err: clean(err),
        }
    }

    fn zeff(&self, args: &[&str]) -> Ended {
        let key = self.signer.public();
        self.zeff_with(args, Some(&key), None)
    }
}

fn system_given(env: &dyn Fn(&str) -> Option<String>) -> bool {
    env("ProgramFiles").is_some()
}

fn shown(title: &str, ended: &Ended) -> String {
    let mut text = format!("----- {title} [exit {}]\n", ended.code);
    if !ended.out.is_empty() {
        text.push_str(&ended.out);
    }
    if !ended.err.is_empty() {
        text.push_str(&ended.err);
    }
    text.push('\n');
    text
}

// ----- the command line -----

#[test]
fn only_the_engine_command_is_the_launchers() {
    for not in [
        &["run"][..],
        &["run", "engine"],
        &["help", "engine"],
        &[],
        &["--format", "engine", "run"],
        &["-p", "engine", "check"],
        &["--", "run", "engine"],
    ] {
        assert_eq!(read(&strings(not)), None, "{not:?}");
    }
    let asked = |args: &[&str]| read(&strings(args)).expect("engine").expect("well asked");
    assert_eq!(
        asked(&["engine", "list"]),
        Asked {
            words: strings(&["list"]),
            json: false,
            from: None,
            nothing_fetched: None,
            write: false,
            project: None
        }
    );
    // Options before, between and after; a value in the next word or after `=`.
    assert_eq!(
        asked(&["--format", "json", "-q", "engine", "--color=never", "list"]),
        Asked {
            words: strings(&["list"]),
            json: true,
            from: None,
            nothing_fetched: None,
            write: false,
            project: None
        }
    );
    assert_eq!(
        asked(&["-p", "study", "engine", "install", "--from=a b/zeff.zip"]),
        Asked {
            words: strings(&["install"]),
            json: false,
            from: Some(PathBuf::from("a b/zeff.zip")),
            nothing_fetched: None,
            write: false,
            project: Some(PathBuf::from("study"))
        }
    );
    assert_eq!(
        asked(&["engine", "install", "--from", "zeff.zip", "--format=text"]).from,
        Some(PathBuf::from("zeff.zip"))
    );
    // What is wrong with how it was asked is said, once the command is known to be `engine`.
    assert!(
        read(&strings(&["engine", "list", "--format", "yaml"]))
            .unwrap()
            .is_err()
    );
    assert!(
        read(&strings(&["engine", "install", "--from"]))
            .unwrap()
            .is_err()
    );
}

#[test]
fn sizes_are_read_as_people_read_them() {
    assert_eq!(size_text(0), "0 B");
    assert_eq!(size_text(999), "999 B");
    assert_eq!(size_text(1_000), "1.0 kB");
    assert_eq!(size_text(21_449_000), "21.4 MB");
    assert_eq!(size_text(3_200_000_000), "3.2 GB");
}

// ----- the default, kept in config.toml -----

#[test]
fn the_default_is_written_and_every_other_line_is_as_it_was() {
    let set = |config: &str, version: Option<&str>| with_default(config, version);
    // No file yet; a file with other settings; a table with other keys; a default to replace.
    assert_eq!(
        set("", Some("0.9.1")).as_deref(),
        Some("[engine]\ndefault = \"0.9.1\"\n")
    );
    assert_eq!(
        set(
            "# mine\n[network]\nmirror = \"https://mirror.example/zeff\"\n",
            Some("0.9.1")
        )
        .as_deref(),
        Some(
            "# mine\n[network]\nmirror = \"https://mirror.example/zeff\"\n\n[engine]\ndefault = \"0.9.1\"\n"
        )
    );
    assert_eq!(
        set(
            "[engine]\n# which runs outside a project\nother = 1\n\n[network]\nproxy = \"p\"\n",
            Some("0.9.1")
        )
        .as_deref(),
        Some(
            "[engine]\ndefault = \"0.9.1\"\n# which runs outside a project\nother = 1\n\n[network]\nproxy = \"p\"\n"
        )
    );
    assert_eq!(
        set(
            "[engine] # the engine\ndefault = \"0.9.0\" # old\n[colour]\ndefault = \"auto\"\n",
            Some("0.10.0-rc.1")
        )
        .as_deref(),
        Some("[engine] # the engine\ndefault = \"0.10.0-rc.1\"\n[colour]\ndefault = \"auto\"\n")
    );
    // Taken out again; nothing to take out.
    assert_eq!(
        set(
            "[engine]\ndefault = \"0.9.0\"\n[colour]\ndefault = \"auto\"\n",
            None
        )
        .as_deref(),
        Some("[engine]\n[colour]\ndefault = \"auto\"\n")
    );
    assert_eq!(
        set("[colour]\ndefault = \"auto\"\n", None).as_deref(),
        Some("[colour]\ndefault = \"auto\"\n")
    );
    // Whatever is written reads back as the setting asked for.
    for config in [
        "",
        "[engine]\n",
        "[a]\nb = 1\n",
        "[engine]\ndefault = \"0.1.0\"\n",
    ] {
        let written = set(config, Some("0.9.1")).unwrap();
        assert_eq!(
            default_setting(&written).as_deref(),
            Some("0.9.1"),
            "{config}"
        );
    }
    // Forms this does not rewrite, and a file that is not TOML: refused, never half-written.
    for config in [
        "engine.default = \"0.9.0\"\n",
        "engine = { default = \"0.9.0\" }\n",
        "[engine\ndefault = 1\n",
        "this is not toml\n",
    ] {
        assert_eq!(set(config, Some("0.9.1")), None, "{config}");
    }
}

// ----- the commands -----

/// The step's finish line: an engine is installed from a file, listed, made the default and
/// removed.
#[test]
fn an_engine_is_installed_listed_made_default_and_removed() {
    let world = World::new();
    let mut out = String::new();
    let mut step = |title: &str, args: &[&str], code: u8| {
        let ended = world.zeff(args);
        out.push_str(&shown(title, &ended));
        assert_eq!(ended.code, code, "{title}: {ended:?}");
        ended
    };
    step("nothing installed", &["engine", "list"], 0);
    step(
        "nothing installed, as JSON",
        &["engine", "list", "--format", "json"],
        0,
    );
    step("no default to name", &["engine", "default"], 4);
    let first = world.download("0.9.0");
    let second = world.download("0.9.1");
    let early = world.download("0.10.0-rc.1");
    step(
        "install",
        &["engine", "install", "--from", first.to_str().unwrap()],
        0,
    );
    assert!(world.home().join("engines/0.9.0/zeff").is_file());
    step(
        "install again",
        &["engine", "install", "--from", first.to_str().unwrap()],
        0,
    );
    // A path as it is typed, read from the folder the command is run in.
    step(
        "install a second, by a relative path",
        &[
            "engine",
            "install",
            "--from",
            "downloads/0.9.1/zeff-0.9.1-linux-x86-64.tar.zst",
        ],
        0,
    );
    step(
        "install one before its release",
        &["engine", "install", "--from", early.to_str().unwrap()],
        0,
    );
    assert!(second.is_file());
    step("list", &["engine", "list"], 0);
    step("the default, none set", &["engine", "default"], 0);
    assert_eq!(
        default_of(&world.store(), Platform::Unix),
        Default::Newest("0.9.1".to_string())
    );
    step("make one the default", &["engine", "default", "0.9.0"], 0);
    assert_eq!(
        std::fs::read_to_string(world.home().join("config.toml")).unwrap(),
        "[engine]\ndefault = \"0.9.0\"\n"
    );
    step("the default, set", &["engine", "default"], 0);
    step("list with a default set", &["engine", "list"], 0);
    let json = step("list as JSON", &["--format=json", "engine", "list"], 0);
    assert!(
        json.out
            .contains("\"default\": \"0.9.0\", \"default_set\": true"),
        "{}",
        json.out
    );
    step("remove another", &["engine", "remove", "0.10.0-rc.1"], 0);
    assert!(!world.home().join("engines/0.10.0-rc.1").exists());
    step("remove the default", &["engine", "remove", "0.9.0"], 0);
    assert!(!world.home().join("engines/0.9.0").exists());
    assert_eq!(
        default_of(&world.store(), Platform::Unix),
        Default::Newest("0.9.1".to_string())
    );
    step("list after the removals", &["engine", "list"], 0);
    step("remove the last", &["engine", "remove", "0.9.1"], 0);
    step("list with nothing left", &["engine", "list"], 0);
    insta::assert_snapshot!(out);
}

#[test]
fn every_refusal_of_the_engine_commands_says_what_to_do() {
    let world = World::new();
    let mut out = String::new();
    let mut step = |title: &str, ended: Ended, code: u8| {
        out.push_str(&shown(title, &ended));
        assert_eq!(ended.code, code, "{title}: {ended:?}");
        assert!(
            ended.out.is_empty(),
            "{title}: a refusal writes to the error stream alone"
        );
    };
    let path = world.download("0.9.1");
    let file = path.to_str().unwrap();
    assert_eq!(world.zeff(&["engine", "install", "--from", file]).code, 0);
    step("no command", world.zeff(&["engine"]), 1);
    step(
        "a command there is not",
        world.zeff(&["engine", "upgrade"]),
        1,
    );
    step(
        "list with a word after it",
        world.zeff(&["engine", "list", "all"]),
        1,
    );
    step(
        "a format there is not",
        world.zeff(&["engine", "list", "--format", "yaml"]),
        1,
    );
    step(
        "install with a version and a file",
        world.zeff(&["engine", "install", "0.9.1", "--from", file]),
        1,
    );
    step(
        "`--from` with no file",
        world.zeff(&["engine", "install", "--from"]),
        1,
    );
    step("remove with nothing", world.zeff(&["engine", "remove"]), 1);
    step(
        "remove a range",
        world.zeff(&["engine", "remove", "0.9"]),
        1,
    );
    step(
        "remove two",
        world.zeff(&["engine", "remove", "0.9.0", "0.9.1"]),
        1,
    );
    step(
        "remove one that is not there",
        world.zeff(&["engine", "remove", "0.9.0"]),
        4,
    );
    step(
        "default one that is not there",
        world.zeff(&["engine", "default", "0.8.0"]),
        4,
    );
    step(
        "default a word",
        world.zeff(&["engine", "default", "newest"]),
        1,
    );
    // A launcher with no key; a stranger's key; a file that is not there; another machine's.
    step(
        "install with no key built in",
        world.zeff_with(&["engine", "install", "--from", file], None, None),
        1,
    );
    let stranger = Signer::new().public();
    let other = world.download("0.9.2");
    step(
        "install what another key signed",
        world.zeff_with(
            &["engine", "install", "--from", other.to_str().unwrap()],
            Some(&stranger),
            None,
        ),
        1,
    );
    step(
        "install a file that is not there",
        world.zeff(&[
            "engine",
            "install",
            "--from",
            "zeff-0.9.3-linux-x86-64.tar.zst",
        ]),
        1,
    );
    step(
        "install a file that is no archive",
        world.zeff(&["engine", "install", "--from", "notes.txt"]),
        1,
    );
    assert!(!world.home().join("engines/0.9.2").exists());
    // A settings file this does not rewrite is left exactly as it was.
    let config = world.home().join("config.toml");
    std::fs::write(&config, "engine = { default = \"0.9.0\" }\n").unwrap();
    step(
        "a default set in a form that is not rewritten",
        world.zeff(&["engine", "default", "0.9.1"]),
        1,
    );
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        "engine = { default = \"0.9.0\" }\n"
    );
    step(
        "the default set is not installed",
        world.zeff(&["engine", "default"]),
        4,
    );
    let listed = world.zeff(&["engine", "list"]);
    assert_eq!(listed.code, 0);
    out.push_str(&shown(
        "list when the default set is not installed",
        &listed,
    ));
    insta::assert_snapshot!(out);
}

#[test]
fn an_engine_of_the_system_wide_store_is_listed_and_never_removed() {
    let world = World::new();
    let programs = world.dir.path().join("Program Files");
    let system = programs.join("Zeff").join("engines").join("0.8.0");
    std::fs::create_dir_all(&system).unwrap();
    std::fs::write(system.join("zeff.exe"), "engine").unwrap();
    let own = world.home().join("engines").join("0.9.1");
    std::fs::create_dir_all(&own).unwrap();
    std::fs::write(own.join("zeff.exe"), "engine 0.9.1").unwrap();
    let key = world.signer.public();
    let zeff = |args: &[&str]| world.zeff_with(args, Some(&key), Some(&programs));
    let mut out = String::new();
    let listed = zeff(&["engine", "list"]);
    assert_eq!(listed.code, 0);
    out.push_str(&shown("list", &listed));
    let removed = zeff(&["engine", "remove", "0.8.0"]);
    assert_eq!(removed.code, 1);
    assert!(
        system.join("zeff.exe").is_file(),
        "the shared engine is still there"
    );
    out.push_str(&shown("remove a shared engine", &removed));
    // It can be the default: the setting is the user's own.
    let made = zeff(&["engine", "default", "0.8.0"]);
    assert_eq!(made.code, 0, "{made:?}");
    out.push_str(&shown("make a shared engine the default", &made));
    out.push_str(&shown("list", &zeff(&["engine", "list"])));
    insta::assert_snapshot!(out);
}

// ----- the launcher itself -----

/// The binary answers `zeff engine` on its own and hands everything else over.
#[cfg(unix)]
#[test]
fn the_launcher_answers_engine_commands_itself_and_hands_the_rest_over() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    for version in ["0.9.0", "0.9.1"] {
        let folder = home.join("engines").join(version);
        std::fs::create_dir_all(&folder).unwrap();
        let engine = folder.join("zeff");
        std::fs::write(
            &engine,
            format!("#!/bin/sh\necho \"engine {version}: $*\"\n"),
        )
        .unwrap();
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let zeff = |args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_zeff-launcher"))
            .args(args)
            .env_clear()
            .env("ZEFF_HOME", &home)
            .current_dir(dir.path())
            .output()
            .unwrap();
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
            String::from_utf8(out.stderr).unwrap(),
        )
    };
    let (code, out, err) = zeff(&["engine", "default", "0.9.0"]);
    assert_eq!((code, err.as_str()), (0, ""), "{out}");
    assert!(
        out.starts_with("the default engine is now Zeff 0.9.0"),
        "{out}"
    );
    // The engine it has just made the default answers what is not the launcher's own.
    assert_eq!(
        zeff(&["help", "engine"]),
        (0, "engine 0.9.0: help engine\n".to_string(), String::new())
    );
    let (code, out, _) = zeff(&["engine", "list"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("0.9.0") && out.contains("the default, set in `config.toml`"),
        "{out}"
    );
    let (code, out, err) = zeff(&["engine", "remove", "0.9.0"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("removed Zeff 0.9.0"), "{out}");
    assert!(err.contains("will ask to install it again"), "{err}");
    assert_eq!(
        zeff(&["version"]),
        (0, "engine 0.9.1: version\n".to_string(), String::new())
    );
    // No key is built in: nothing is installed by this binary yet, and it says why.
    let (code, out, err) = zeff(&[
        "engine",
        "install",
        "--from",
        "zeff-0.9.2-linux-x86-64.tar.zst",
    ]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(
        err.starts_with("error[Z0711]: this launcher has no signing key built in"),
        "{err}"
    );
    let (code, _, err) = zeff(&["engine", "list", "--format", "yaml"]);
    assert_eq!(code, 1);
    assert!(
        err.starts_with("error: `--format yaml` is not a format"),
        "{err}"
    );
}
