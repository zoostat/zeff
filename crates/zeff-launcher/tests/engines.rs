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
            project: None,
            doctor: false
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
            project: None,
            doctor: false
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
            project: Some(PathBuf::from("study")),
            doctor: false
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

/// Each engine is asked what its build is; one that does not say is listed without it.
#[cfg(unix)]
#[test]
fn the_list_shows_what_each_engine_says_its_build_is() {
    use std::os::unix::fs::PermissionsExt;
    let world = World::new();
    for (version, answer) in [
        (
            "0.9.0",
            r#"echo '{"version": "0.9.0", "build": "official"}'"#,
        ),
        (
            "0.9.1",
            r#"echo '{"version": "0.9.1", "launcher": null, "build": "source"}'"#,
        ),
        ("0.9.2", "echo zeff 0.9.2"),
        (
            "0.9.3",
            r#"echo '{"version": "0.9.3", "build": "a word of a later launcher"}'"#,
        ),
        ("0.9.4", "exit 3"),
    ] {
        let folder = world.home().join("engines").join(version);
        std::fs::create_dir_all(&folder).unwrap();
        let engine = folder.join("zeff");
        std::fs::write(&engine, format!("#!/bin/sh\n{answer}\n")).unwrap();
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let listed = world.zeff(&["engine", "list"]);
    assert_eq!(listed.code, 0);
    let json = world.zeff(&["engine", "list", "--format", "json"]);
    for (version, build) in [
        ("0.9.0", "\"official\""),
        ("0.9.1", "\"source\""),
        ("0.9.2", "null"),
        ("0.9.3", "null"),
        ("0.9.4", "null"),
    ] {
        assert!(
            json.out.contains(&format!(
                "\"version\": \"{version}\", \"path\": \"<tmp>/home/engines/{version}\""
            )) && json
                .out
                .split("{\"version\": ")
                .any(|record| record.starts_with(&format!("\"{version}\""))
                    && record.contains(&format!("\"build\": {build},"))),
            "{version}: {}",
            json.out
        );
    }
    insta::assert_snapshot!(listed.out);
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
    // `zeff audit` goes to the newest engine installed, whatever the project pins, and that
    // engine is told it is not there for the pin.
    for version in ["0.9.0", "0.9.2", "0.10.0-rc.1"] {
        let folder = home.join("engines").join(version);
        std::fs::create_dir_all(&folder).unwrap();
        let engine = folder.join("zeff");
        std::fs::write(
            &engine,
            format!("#!/bin/sh\necho \"engine {version}: $* [tried as $ZEFF_ENGINE_USE]\"\n"),
        )
        .unwrap();
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    std::fs::write(
        dir.path().join("zeff.toml"),
        "[project]\nname = \"study\"\nzeff = \"0.9.0\"\n",
    )
    .unwrap();
    assert_eq!(
        zeff(&["audit", "--format", "json"]),
        (
            0,
            "engine 0.9.2: audit --format json [tried as 0.9.2]\n".to_string(),
            String::new()
        )
    );
    assert_eq!(
        zeff(&["-p", ".", "audit"]),
        (
            0,
            "engine 0.9.2: -p . audit [tried as 0.9.2]\n".to_string(),
            String::new()
        )
    );
    // Every other command stays with the engine the project pins.
    assert_eq!(
        zeff(&["run"]),
        (
            0,
            "engine 0.9.0: run [tried as ]\n".to_string(),
            String::new()
        )
    );
    std::fs::remove_file(dir.path().join("zeff.toml")).unwrap();
    // The publisher's key is built in, so this binary reads an archive it is given, and
    // refuses what is not for this machine and what is not there, each in its own words.
    let here = zeff_launcher::archive::Target::host().expect("a machine engines are built for");
    let elsewhere = [
        zeff_launcher::archive::Target::LinuxX86_64,
        zeff_launcher::archive::Target::LinuxArm64,
    ]
    .into_iter()
    .find(|target| *target != here)
    .unwrap();
    let other = format!("zeff-0.9.2-{}.tar.zst", elsewhere.name());
    let (code, out, err) = zeff(&["engine", "install", "--from", &other]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(
        err.starts_with(&format!(
            "error[Z0717]: `{other}` holds an engine for another kind of machine"
        )),
        "{err}"
    );
    let own = format!("zeff-0.9.2-{}{}", here.name(), here.kind().extension());
    let (code, out, err) = zeff(&["engine", "install", "--from", &own]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(
        err.starts_with("error[Z0717]: `") && err.contains("cannot be read: there is no such file"),
        "{err}"
    );
    let (code, _, err) = zeff(&["engine", "list", "--format", "yaml"]);
    assert_eq!(code, 1);
    assert!(
        err.starts_with("error: `--format yaml` is not a format"),
        "{err}"
    );
}

// ----- what no test had asked of the engine commands (release 0.9.0, step 42c3) -----

/// `zeff <args>` on a machine whose environment is `env` and which is of the kind `host`.
fn zeff_on(
    world: &World,
    args: &[&str],
    env: &dyn Fn(&str) -> Option<String>,
    host: Option<Target>,
) -> Ended {
    let args = strings(args);
    let asked = read(&args).expect("an engine command").expect("well asked");
    let key = world.signer.public();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        &asked,
        &Machine {
            env,
            cwd: world.dir.path(),
            platform: Platform::Unix,
            host,
            key: Some(&key),
            fetch: None,
        },
        &mut out,
        &mut err,
    );
    let clean = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .unwrap()
            .replace(&world.dir.path().display().to_string(), "<tmp>")
    };
    Ended {
        code,
        out: clean(out),
        err: clean(err),
    }
}

/// No home folder; a kind of machine no engine is published for, asked for an archive on
/// this machine and for one to fetch; a default named when nothing is installed; and the
/// only engine removed while it is the default.
#[test]
fn refusals_and_endings_the_coverage_report_showed_unsaid() {
    let world = World::new();
    let mut out = String::new();
    let nothing = |_: &str| None;
    let no_home = zeff_on(&world, &["engine", "list"], &nothing, Some(world.host));
    assert_eq!(no_home.code, 4, "{no_home:?}");
    out.push_str(&shown("no home folder", &no_home));

    let home = world.home().display().to_string();
    let env = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
    let archive = world.download("0.9.1");
    let from = zeff_on(
        &world,
        &["engine", "install", "--from", archive.to_str().unwrap()],
        &env,
        None,
    );
    assert_eq!(from.code, 1, "{from:?}");
    out.push_str(&shown(
        "an archive, on a machine of no published kind",
        &from,
    ));
    let fetched = zeff_on(&world, &["engine", "install", "0.9.1"], &env, None);
    assert_eq!(fetched.code, 1, "{fetched:?}");
    out.push_str(&shown(
        "a version to fetch, on a machine of no published kind",
        &fetched,
    ));
    assert!(!world.home().join("engines").exists());

    let none = world.zeff(&["engine", "default", "0.9.1"]);
    assert_eq!(none.code, 4, "{none:?}");
    out.push_str(&shown("a default named when nothing is installed", &none));

    let installed = world.zeff(&["engine", "install", "--from", archive.to_str().unwrap()]);
    assert_eq!(installed.code, 0, "{installed:?}");
    assert_eq!(world.zeff(&["engine", "default", "0.9.1"]).code, 0);
    let older = world.download("0.9.0");
    assert_eq!(
        world
            .zeff(&["engine", "install", "--from", older.to_str().unwrap()])
            .code,
        0
    );
    let other = world.zeff(&["engine", "default", "0.7.0"]);
    assert_eq!(other.code, 4, "{other:?}");
    out.push_str(&shown(
        "a default that is not among the two installed",
        &other,
    ));
    assert_eq!(world.zeff(&["engine", "remove", "0.9.0"]).code, 0);
    let removed = world.zeff(&["engine", "remove", "0.9.1"]);
    assert_eq!(removed.code, 0, "{removed:?}");
    out.push_str(&shown(
        "the only engine removed while it is the default",
        &removed,
    ));
    assert_eq!(
        default_setting(&std::fs::read_to_string(world.home().join("config.toml")).unwrap()),
        None
    );
    insta::assert_snapshot!(out);
}

/// A settings file that cannot be read is said to be so, and the default is not changed.
#[cfg(unix)]
#[test]
fn a_settings_file_that_cannot_be_read_stops_the_default() {
    let world = World::new();
    let archive = world.download("0.9.1");
    assert_eq!(
        world
            .zeff(&["engine", "install", "--from", archive.to_str().unwrap()])
            .code,
        0
    );
    // A folder where the file should be: it is there, and it is not a file to read.
    std::fs::create_dir_all(world.home().join("config.toml")).unwrap();
    let ended = world.zeff(&["engine", "default", "0.9.1"]);
    assert_eq!(ended.code, 1, "{ended:?}");
    assert!(
        ended
            .err
            .contains("`<tmp>/home/config.toml` cannot be read: "),
        "{ended:?}"
    );
    assert!(world.home().join("config.toml").is_dir());
}

/// `zeff doctor` through the launcher's own entry: the report is written and the exit code is
/// that of its problems. A folder inside an engine's folder counts in the engine's size.
#[cfg(unix)]
#[test]
fn doctor_is_answered_by_the_engine_commands_and_a_size_counts_folders_within() {
    let world = World::new();
    let home = world.home().display().to_string();
    let env = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
    let doctor = zeff_on(&world, &["doctor"], &env, Some(world.host));
    assert!(doctor.code <= 1, "{doctor:?}");
    assert!(doctor.out.contains("store"), "{doctor:?}");

    let engine = world.home().join("engines").join("0.9.1");
    std::fs::create_dir_all(engine.join("share").join("deep")).unwrap();
    std::fs::write(engine.join("zeff"), vec![b'x'; 1_000]).unwrap();
    std::fs::write(
        engine.join("share").join("deep").join("notes"),
        vec![b'y'; 234],
    )
    .unwrap();
    let listed = world.zeff(&["engine", "list", "--format", "json"]);
    assert_eq!(listed.code, 0, "{listed:?}");
    assert!(listed.out.contains("\"bytes\": 1234"), "{listed:?}");
    assert!(listed.out.contains("\"store\": \"user\""), "{listed:?}");
}

/// The JSON list says which store an engine is in.
#[test]
fn the_list_as_json_names_the_shared_store() {
    let world = World::new();
    let programs = world.dir.path().join("Program Files");
    let system = programs.join("Zeff").join("engines").join("0.8.0");
    std::fs::create_dir_all(&system).unwrap();
    std::fs::write(system.join("zeff.exe"), "engine").unwrap();
    let key = world.signer.public();
    let listed = world.zeff_with(
        &["engine", "list", "--format", "json"],
        Some(&key),
        Some(&programs),
    );
    assert_eq!(listed.code, 0, "{listed:?}");
    assert!(listed.out.contains("\"version\": \"0.8.0\""), "{listed:?}");
    assert!(listed.out.contains("\"store\": \"system\""), "{listed:?}");
}
