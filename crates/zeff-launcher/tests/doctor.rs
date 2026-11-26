//! `zeff doctor` (Engine and Project Specification, section 2.1, "Diagnosis and settings";
//! release 0.9.0, step 21b): every line of the report on a temporary home, first with nothing
//! wrong and then with each problem made.

use std::path::{Path, PathBuf};
use zeff_launcher::Platform;
use zeff_launcher::doctor::{Around, Proved, doctor, last_selftest, lines};
use zeff_launcher::engines::{Machine, read};
use zeff_launcher::net::{Fetch, FetchError};

/// A network that answers every request alike.
struct Answers(Result<Vec<u8>, FetchError>);

impl Fetch for Answers {
    fn get(&self, _: &str, _: u64) -> Result<Vec<u8>, FetchError> {
        self.0.clone()
    }
}

/// A temporary user's folder with a store in it.
struct World {
    dir: tempfile::TempDir,
}

impl World {
    fn new() -> World {
        let world = World {
            dir: tempfile::tempdir().unwrap(),
        };
        std::fs::create_dir_all(world.user().join(".zeff/bin")).unwrap();
        world
    }

    fn user(&self) -> PathBuf {
        self.dir.path().join("ada")
    }

    fn home(&self) -> PathBuf {
        self.user().join(".zeff")
    }

    /// An engine that says its build when asked, as an engine of this release does.
    fn engine(&self, version: &str, build: &str) {
        let folder = self.home().join("engines").join(version);
        std::fs::create_dir_all(&folder).unwrap();
        let engine = folder.join("zeff");
        std::fs::write(
            &engine,
            format!("#!/bin/sh\necho '{{\"version\": \"{version}\", \"build\": \"{build}\"}}'\n"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    fn selftest(&self, version: &str, conformant: bool, differing: u64) {
        let folder = self.home().join("selftest");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(format!("{version}.toml")),
            format!(
                "# the last `zeff selftest` of this engine on this machine\nversion = \"{version}\"\nconformant = {conformant}\ncases = 558\ndiffering = {differing}\non = \"2026-11-22\"\n"
            ),
        )
        .unwrap();
    }

    /// `zeff <args>` with the environment given; the exit code and the report, the temporary
    /// folder's name written as `<tmp>`.
    fn zeff(
        &self,
        args: &[&str],
        more: &[(&str, &str)],
        network: &Answers,
        free: u64,
        platform: Platform,
    ) -> (u8, String) {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        let asked = read(&args)
            .expect("the launcher's command")
            .expect("well asked");
        assert!(asked.doctor);
        let user = self.user().display().to_string();
        let mut pairs: Vec<(String, String)> = vec![
            ("HOME".to_string(), user.clone()),
            ("USERPROFILE".to_string(), user.clone()),
            ("SHELL".to_string(), "/bin/zsh".to_string()),
            (
                "PATH".to_string(),
                format!("/usr/bin:{user}/.zeff/bin:/bin;{user}/.zeff/bin"),
            ),
        ];
        for (name, value) in more {
            pairs.retain(|(known, _)| known != name);
            pairs.push(((*name).to_string(), (*value).to_string()));
        }
        let env = move |name: &str| {
            pairs
                .iter()
                .find(|(known, _)| known == name)
                .map(|(_, value)| value.clone())
        };
        let machine = Machine {
            env: &env,
            cwd: self.dir.path(),
            platform,
            host: None,
            key: None,
            fetch: Some(network),
        };
        let free_space = move |_: &Path| Some(free);
        let around = Around {
            own_path: Some(self.home().join("bin").join("zeff")),
            free_space: Some(&free_space),
        };
        let mut out = Vec::new();
        let code = doctor(&asked, &machine, &around, &mut out);
        // The count at the end is the count of the lines that carry a fix.
        let problems = lines(&asked, &machine, &around)
            .iter()
            .filter(|line| line.fix.is_some())
            .count();
        assert_eq!(code, u8::from(problems > 0));
        // The launcher's own version is hidden on its own line alone: at a release it is also
        // the version of an engine these stores hold (0.9.0), which is to be read as it is.
        let text = String::from_utf8(out)
            .unwrap()
            .replace(&self.dir.path().display().to_string(), "<tmp>")
            .replace(zeff_launcher::doctor::COMMIT, "<commit>")
            .replace(
                &format!("launcher        Zeff {},", zeff_launcher::VERSION),
                "launcher        Zeff <launcher>,",
            )
            .replace('\\', "/");
        (code, text)
    }
}

const GB: u64 = 1_000_000_000;

fn up() -> Answers {
    Answers(Ok(b"format = 1\n".to_vec()))
}

#[test]
fn the_record_of_a_selftest_is_read_back() {
    let world = World::new();
    world.selftest("0.9.1", true, 0);
    world.selftest("0.9.0", false, 3);
    assert_eq!(
        last_selftest(&world.home(), "0.9.1"),
        Some(Proved {
            conformant: true,
            cases: 558,
            differing: 0,
            on: "2026-11-22".to_string()
        })
    );
    assert_eq!(
        last_selftest(&world.home(), "0.9.0").map(|proved| (proved.conformant, proved.differing)),
        Some((false, 3))
    );
    assert_eq!(last_selftest(&world.home(), "0.9.2"), None);
    std::fs::write(
        world.home().join("selftest/0.9.3.toml"),
        "conformant = \"yes\"\n",
    )
    .unwrap();
    assert_eq!(last_selftest(&world.home(), "0.9.3"), None);
}

/// An installation with nothing wrong: every line of the report, and exit code 0.
#[cfg(unix)]
#[test]
fn an_installation_with_nothing_wrong() {
    let world = World::new();
    world.engine("0.9.0", "official");
    world.engine("0.9.1", "official");
    world.selftest("0.9.1", true, 0);
    std::fs::write(
        world.home().join("config.toml"),
        "[engine]\ndefault = \"0.9.1\"\n\n[network]\nmirror = \"https://mirror.univ.example/zeff\"\nproxy = \"http://proxy.univ.example:3128\"\n",
    )
    .unwrap();
    let completions = world.user().join(".zfunc");
    std::fs::create_dir_all(&completions).unwrap();
    std::fs::write(completions.join("_zeff"), "#compdef zeff\n").unwrap();
    let (code, text) = world.zeff(
        &["doctor"],
        &[],
        &up(),
        212 * GB + 400_000_000,
        Platform::Unix,
    );
    assert_eq!(code, 0, "{text}");
    let (json_code, json) = world.zeff(
        &["doctor", "--format", "json"],
        &[],
        &up(),
        212 * GB,
        Platform::Unix,
    );
    assert_eq!(json_code, 0);
    assert!(
        json.starts_with("{\"problems\": 0, \"lines\": [\n  {\"about\": \"launcher\", "),
        "{json}"
    );
    assert!(
        json.contains("{\"about\": \"mirror\", \"found\": \"`https://mirror.univ.example/zeff/`, from `[network] mirror` of `config.toml`\", \"problem\": false, \"fix\": null}"),
        "{json}"
    );
    insta::assert_snapshot!(text);
}

/// Each problem made in turn: the line that names it carries what to do, and the report ends
/// with the count.
#[cfg(unix)]
#[test]
fn each_problem_is_named_with_what_to_do_about_it() {
    let mut out = String::new();
    let mut case = |title: &str, problems: u8, made: &dyn Fn(&World) -> (u8, String)| {
        let world = World::new();
        world.engine("0.9.1", "source");
        let (code, text) = made(&world);
        assert_eq!(code, u8::from(problems > 0), "{title}: {text}");
        let marked = text.lines().filter(|line| line.starts_with("! ")).count();
        assert_eq!(marked, usize::from(problems), "{title}: {text}");
        // Only what the case is about: the lines that are marked, with their fixes, and the
        // count.
        let mut shown = String::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            if line.starts_with("! ") {
                shown.push_str(line);
                shown.push('\n');
                if let Some(fix) = lines.peek() {
                    shown.push_str(fix);
                    shown.push('\n');
                }
            }
        }
        shown.push_str(text.lines().last().unwrap_or_default());
        out.push_str(&format!("----- {title}\n{shown}\n\n"));
    };
    let plain = |world: &World, more: &[(&str, &str)]| {
        world.zeff(&["doctor"], more, &up(), 100 * GB, Platform::Unix)
    };
    case("nothing wrong", 0, &|world| plain(world, &[]));
    case("no engine", 1, &|world| {
        std::fs::remove_dir_all(world.home().join("engines")).unwrap();
        plain(world, &[])
    });
    case("a default that is not installed", 1, &|world| {
        std::fs::write(
            world.home().join("config.toml"),
            "[engine]\ndefault = \"0.8.0\"\n",
        )
        .unwrap();
        plain(world, &[])
    });
    case("settings that are not TOML", 1, &|world| {
        std::fs::write(world.home().join("config.toml"), "[engine\ndefault = 1\n").unwrap();
        plain(world, &[])
    });
    case("the launcher's folder is not on PATH", 1, &|world| {
        plain(world, &[("PATH", "/usr/bin:/bin")])
    });
    case("a mirror that is not an address", 1, &|world| {
        plain(world, &[("ZEFF_MIRROR", "mirror.univ.example")])
    });
    case("a mirror that does not answer", 1, &|world| {
        world.zeff(
            &["doctor"],
            &[("ZEFF_MIRROR", "https://mirror.univ.example/zeff")],
            &Answers(Err(FetchError::Failed("no route to host".to_string()))),
            100 * GB,
            Platform::Unix,
        )
    });
    case("a mirror that fails", 1, &|world| {
        world.zeff(
            &["doctor"],
            &[("ZEFF_MIRROR", "https://mirror.univ.example/zeff")],
            &Answers(Err(FetchError::Status(503))),
            100 * GB,
            Platform::Unix,
        )
    });
    case("a mirror without an index", 1, &|world| {
        world.zeff(
            &["doctor"],
            &[("ZEFF_MIRROR", "https://mirror.univ.example/zeff")],
            &Answers(Err(FetchError::NotFound)),
            100 * GB,
            Platform::Unix,
        )
    });
    case("a full disk", 1, &|world| {
        world.zeff(&["doctor"], &[], &up(), 312_000_000, Platform::Unix)
    });
    case("an engine that is not conformant here", 1, &|world| {
        world.selftest("0.9.1", false, 3);
        plain(world, &[])
    });
    case("a store that cannot be written", 1, &|world| {
        use std::os::unix::fs::PermissionsExt;
        let home = world.home();
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o555)).unwrap();
        let said = plain(world, &[]);
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o755)).unwrap();
        said
    });
    case("three at once", 3, &|world| {
        std::fs::remove_dir_all(world.home().join("engines")).unwrap();
        world.zeff(
            &["doctor"],
            &[("PATH", "/usr/bin"), ("ZEFF_MIRROR", "ftp://mirror")],
            &up(),
            100 * GB,
            Platform::Unix,
        )
    });
    insta::assert_snapshot!(out);
}

/// What is not a problem and still changes a line: fetching forbidden, a mirror from the
/// environment, a proxy, no colour, a store named by the environment, another shell, Windows.
#[cfg(unix)]
#[test]
fn the_settings_in_effect_say_where_each_comes_from() {
    let world = World::new();
    world.engine("0.9.1", "source");
    let mut out = String::new();
    let never = Answers(Err(FetchError::Failed("the network was asked".to_string())));
    let mut shown =
        |title: &str, args: &[&str], more: &[(&str, &str)], network: &Answers, platform| {
            let (code, text) = world.zeff(args, more, network, 100 * GB, platform);
            assert_eq!(code, 0, "{title}: {text}");
            out.push_str(&format!("----- {title}\n{text}\n"));
        };
    shown(
        "offline, on the command line",
        &["doctor", "--offline"],
        &[],
        &never,
        Platform::Unix,
    );
    shown(
        "frozen by the environment, with a mirror, a proxy and no colour",
        &["doctor"],
        &[
            ("ZEFF_FROZEN", "1"),
            ("ZEFF_MIRROR", "http://10.0.0.7:8080"),
            ("HTTPS_PROXY", "http://proxy.univ.example:3128"),
            ("NO_COLOR", "1"),
            ("SHELL", "/usr/bin/fish"),
        ],
        &never,
        Platform::Unix,
    );
    // On Windows an engine's file is `zeff.exe`.
    let folder = world.home().join("engines").join("0.9.1");
    std::fs::copy(folder.join("zeff"), folder.join("zeff.exe")).unwrap();
    let named = world.home().display().to_string();
    shown(
        "a store named by the environment, on Windows",
        &["doctor"],
        &[("ZEFF_HOME", &named), ("SHELL", "")],
        &never,
        Platform::Windows,
    );
    insta::assert_snapshot!(out);
}

#[test]
fn doctor_takes_nothing_after_it_and_is_the_launchers() {
    use zeff_launcher::engines::run;
    let args = vec!["doctor".to_string(), "everything".to_string()];
    let asked = read(&args).unwrap().unwrap();
    let env = |_: &str| None;
    let machine = Machine {
        env: &env,
        cwd: Path::new("."),
        platform: Platform::Unix,
        host: None,
        key: None,
        fetch: None,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(run(&asked, &machine, &mut out, &mut err), 1);
    assert!(
        String::from_utf8(err)
            .unwrap()
            .starts_with("error: `zeff doctor` takes nothing after it")
    );
    // A user with no home folder at all: one line says so, with what to do.
    let asked = read(&["doctor".to_string()]).unwrap().unwrap();
    let around = Around {
        own_path: None,
        free_space: None,
    };
    let mut out = Vec::new();
    assert_eq!(doctor(&asked, &machine, &around, &mut out), 1);
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("! store     this user has no home folder"),
        "{text}"
    );
    assert!(text.contains("fix: set `ZEFF_HOME`"), "{text}");
}

// ----- lines of the report no test had asked for (release 0.9.0, step 42c3) -----

/// The lines of `zeff doctor` on a machine described by `pairs`, with the launcher at
/// `own_path` and the free space as `free` tells it (`None`: the machine's own answer).
fn report_of(
    pairs: &[(&str, String)],
    platform: Platform,
    own_path: Option<PathBuf>,
    network: &Answers,
    free: Option<zeff_launcher::doctor::FreeSpace<'_>>,
) -> Vec<zeff_launcher::doctor::Line> {
    let args = vec!["doctor".to_string()];
    let asked = read(&args).unwrap().unwrap();
    let env = |name: &str| {
        pairs
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, value)| value.clone())
    };
    let machine = Machine {
        env: &env,
        cwd: Path::new("."),
        platform,
        host: None,
        key: None,
        fetch: Some(network),
    };
    lines(
        &asked,
        &machine,
        &Around {
            own_path,
            free_space: free,
        },
    )
}

fn found<'a>(report: &'a [zeff_launcher::doctor::Line], about: &str) -> &'a str {
    report
        .iter()
        .find(|line| line.about == about)
        .unwrap_or_else(|| panic!("no line about {about} in {report:?}"))
        .found
        .as_str()
}

#[test]
fn a_store_not_made_yet_and_a_user_folder_the_environment_does_not_name() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("not-yet").join(".zeff");
    let pairs = [("ZEFF_HOME", home.display().to_string())];
    // The free space is the machine's own answer here: that of the nearest folder that
    // exists, since the store does not yet.
    let report = report_of(&pairs, Platform::Unix, None, &up(), None);
    let store = found(&report, "store");
    assert!(
        store.ends_with("not made yet, the first install makes it"),
        "{store}"
    );
    assert_eq!(
        found(&report, "user folder"),
        "not named by the environment"
    );
    assert_eq!(
        found(&report, "completions"),
        "`zeff completions <shell>` writes the file for bash, zsh, fish or powershell"
    );
    let disk = found(&report, "disk");
    assert!(disk.ends_with(" free where the store is"), "{disk}");
    // No launcher path was given, so nothing is said of PATH.
    assert!(report.iter().all(|line| line.about != "PATH"));
}

#[test]
fn free_space_that_cannot_be_told_is_said_and_is_no_problem() {
    let dir = tempfile::tempdir().unwrap();
    let pairs = [("ZEFF_HOME", dir.path().display().to_string())];
    let untold = |_: &Path| None;
    let report = report_of(&pairs, Platform::Unix, None, &up(), Some(&untold));
    let line = report.iter().find(|line| line.about == "disk").unwrap();
    assert_eq!(
        line.found,
        "the free space where the store is could not be read"
    );
    assert_eq!(line.fix, None);
}

#[test]
fn a_shared_store_the_fix_for_path_on_windows_and_completions_for_bash() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("ada");
    let home = user.join(".zeff");
    let programs = dir.path().join("programs");
    std::fs::create_dir_all(home.join("bin")).unwrap();
    std::fs::create_dir_all(programs.join("Zeff").join("engines")).unwrap();
    let completions = user.join(".local/share/bash-completion/completions");
    std::fs::create_dir_all(&completions).unwrap();
    std::fs::write(completions.join("zeff"), "").unwrap();
    let pairs = [
        ("ZEFF_HOME", home.display().to_string()),
        ("USERPROFILE", user.display().to_string()),
        ("ProgramFiles", programs.display().to_string()),
        ("PATH", "C:/Windows".to_string()),
        ("SHELL", "/usr/bin/bash".to_string()),
    ];
    let free = |_: &Path| Some(50 * GB);
    let own = home.join("bin").join("zeff.exe");
    let report = report_of(&pairs, Platform::Windows, Some(own), &up(), Some(&free));
    let shared = found(&report, "shared store");
    assert!(
        shared.ends_with("read after the user's own") && shared.contains("Zeff"),
        "{shared}"
    );
    let path = report.iter().find(|line| line.about == "PATH").unwrap();
    assert!(path.found.ends_with("is not on PATH"), "{}", path.found);
    let fix = path.fix.as_deref().unwrap();
    assert!(
        fix.starts_with("add `") && fix.ends_with("` to the user's PATH in the system's settings"),
        "{fix}"
    );
    let completions = found(&report, "completions");
    assert!(completions.starts_with("for bash, at `"), "{completions}");
}

#[test]
fn a_mirror_of_the_settings_file_that_is_no_address_and_a_server_that_sends_too_much() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join(".zeff");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        home.join("config.toml"),
        "[network]\nmirror = \"mirror.example\"\n",
    )
    .unwrap();
    let pairs = [("ZEFF_HOME", home.display().to_string())];
    let free = |_: &Path| Some(50 * GB);
    let report = report_of(&pairs, Platform::Unix, None, &up(), Some(&free));
    let mirror = report.iter().find(|line| line.about == "mirror").unwrap();
    assert_eq!(
        mirror.found,
        "`mirror.example`, from `[network] mirror` of `config.toml`, is not an address"
    );
    assert_eq!(
        mirror.fix.as_deref(),
        Some("a mirror begins with `https://` or `http://`")
    );
    // With a mirror that is an address, a server that sends more than an index is.
    std::fs::remove_file(home.join("config.toml")).unwrap();
    let pairs = [
        ("ZEFF_HOME", home.display().to_string()),
        ("ZEFF_MIRROR", "https://mirror.example/zeff".to_string()),
    ];
    let too_much = Answers(Err(FetchError::TooLarge));
    let report = report_of(&pairs, Platform::Unix, None, &too_much, Some(&free));
    let host = report
        .iter()
        .find(|line| line.about == "release host")
        .unwrap();
    assert!(
        host.found
            .ends_with(": the server sent more than an index is"),
        "{}",
        host.found
    );
    assert!(host.fix.is_some());
}

#[test]
fn a_record_of_a_selftest_that_is_not_one_is_not_read() {
    let world = World::new();
    let folder = world.home().join("selftest");
    std::fs::create_dir_all(&folder).unwrap();
    let records = [
        (
            "0.9.4",
            "conformant = true\ncases = \"many\"\ndiffering = 0\non = \"2026-11-22\"\n",
        ),
        (
            "0.9.5",
            "conformant = true\ncases = 558\non = \"2026-11-22\"\n",
        ),
        (
            "0.9.6",
            "conformant = true\ncases = 558\ndiffering = 0\non = 20261122\n",
        ),
        ("0.9.7", "conformant = true\ncases = 558\ndiffering = 0\n"),
        ("0.9.8", "cases = 558\ndiffering = 0\non = \"2026-11-22\"\n"),
        ("0.9.9", "conformant = [true\n"),
    ];
    for (version, text) in records {
        std::fs::write(folder.join(format!("{version}.toml")), text).unwrap();
        assert_eq!(last_selftest(&world.home(), version), None, "{version}");
    }
}
