//! Engines from the signed index (Engine and Project Specification, section 2.1; release
//! 0.9.0, step 19b), against a web server on this machine that serves an index, archives and a
//! key list as a mirror does. Every key is made in the test.

mod common;

use common::{Server, Signer, release, site, write_signed};
use std::path::{Path, PathBuf};
use zeff_launcher::archive::{Placed, Target};
use zeff_launcher::engines::{Machine, for_the_engine, install_pinned, read, run, usage};
use zeff_launcher::net::{Http, Source};
use zeff_launcher::remote::{Release, Wanted, fetch_engine, pick, read_index};
use zeff_launcher::{Choice, Platform, Version, choose};

const HOST: Target = Target::LinuxX86_64;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

/// A temporary store, a site and the server that serves it.
struct World {
    dir: tempfile::TempDir,
    signer: Signer,
    server: Server,
}

#[derive(Debug)]
struct Ended {
    code: u8,
    out: String,
    err: String,
}

impl World {
    fn new(versions: &[&str]) -> World {
        let dir = tempfile::tempdir().unwrap();
        let signer = Signer::new();
        site(&dir.path().join("site"), versions, HOST, &signer);
        let server = Server::start(&dir.path().join("site"));
        World {
            dir,
            signer,
            server,
        }
    }

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    fn site(&self) -> PathBuf {
        self.dir.path().join("site")
    }

    fn env(&self, more: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let mut pairs = vec![
            ("ZEFF_HOME".to_string(), self.home().display().to_string()),
            ("ZEFF_MIRROR".to_string(), self.server.base.clone()),
        ];
        for (name, value) in more {
            pairs.retain(|(known, _)| known != name);
            pairs.push(((*name).to_string(), (*value).to_string()));
        }
        move |name| {
            pairs
                .iter()
                .find(|(known, _)| known == name)
                .map(|(_, value)| value.clone())
        }
    }

    /// The temporary folder and the server's port written out of a text.
    fn clean(&self, text: &str) -> String {
        text.replace(&self.dir.path().display().to_string(), "<tmp>")
            .replace(&self.server.base, "http://mirror/")
            .replace('\\', "/")
    }

    fn zeff_with(&self, args: &[&str], key: Option<&str>, more: &[(&str, &str)]) -> Ended {
        let env = self.env(more);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = match read(&strings(args)).expect("an engine command") {
            Ok(asked) => run(
                &asked,
                &Machine {
                    env: &env,
                    cwd: self.dir.path(),
                    platform: Platform::Unix,
                    host: Some(HOST),
                    key,
                    fetch: None,
                },
                &mut out,
                &mut err,
            ),
            Err(wrong) => usage(&mut err, &wrong),
        };
        Ended {
            code,
            out: self.clean(&String::from_utf8(out).unwrap()),
            err: self.clean(&String::from_utf8(err).unwrap()),
        }
    }

    fn zeff(&self, args: &[&str]) -> Ended {
        let key = self.signer.public();
        self.zeff_with(args, Some(&key), &[])
    }

    fn installed(&self) -> Vec<String> {
        let mut found: Vec<String> = std::fs::read_dir(self.home().join("engines"))
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        found.sort();
        found
    }
}

fn shown(title: &str, ended: &Ended) -> String {
    format!(
        "----- {title} [exit {}]\n{}{}\n",
        ended.code, ended.out, ended.err
    )
}

// ----- the index -----

#[test]
fn the_index_is_read_and_the_engine_asked_for_is_picked() {
    let index = "# the engines of Zeff\nformat = 1\nnote = \"a key a later launcher reads\"\n\n\
        [[engine]]\nversion = \"0.10.0-rc.1\"\nat = \"engines/0.10.0-rc.1/\"\ntargets = [\"linux-x86-64\", \"macos-arm64\"]\n\n\
        [[engine]]\nversion = \"0.9.0\"\nat = \"engines/0.9.0/\"\ntargets = [\"linux-x86-64\", \"macos-arm64\"]\n\n\
        [[engine]]\nversion = \"0.9.2\"\nat = \"https://elsewhere.example/v0.9.2\"\ntargets = [\"macos-arm64\"]\nsize = 3\n\n\
        [[engine]]\nversion = \"0.9.1\"\nat = \"engines/0.9.1/\"\ntargets = [\"linux-x86-64\", \"macos-arm64\"]\n\n\
        [[engine]]\nversion = \"0.8.4\"\nat = \"engines/0.8.4/\"\ntargets = [\"linux-x86-64\"]\n";
    let releases = read_index(index).unwrap();
    let versions: Vec<&str> = releases
        .iter()
        .map(|release| release.version.text.as_str())
        .collect();
    assert_eq!(
        versions,
        ["0.8.4", "0.9.0", "0.9.1", "0.9.2", "0.10.0-rc.1"]
    );
    assert_eq!(releases[3].at, "https://elsewhere.example/v0.9.2");
    let picked = |word: Option<&str>, host: Target| {
        pick(
            &releases,
            &Wanted::parse(word).expect("a version or a series"),
            host,
        )
        .map(|release: &Release| release.version.text.as_str())
    };
    // The newest release, never a version before its release unless it is named.
    assert_eq!(picked(None, Target::MacosArm64), Some("0.9.2"));
    assert_eq!(picked(None, Target::LinuxX86_64), Some("0.9.1"));
    assert_eq!(picked(Some("0.9"), Target::LinuxX86_64), Some("0.9.1"));
    assert_eq!(picked(Some("0.9"), Target::MacosArm64), Some("0.9.2"));
    assert_eq!(picked(Some("0.8"), Target::MacosArm64), None);
    assert_eq!(picked(Some("0.10"), Target::LinuxX86_64), None);
    assert_eq!(
        picked(Some("0.10.0-rc.1"), Target::LinuxX86_64),
        Some("0.10.0-rc.1")
    );
    assert_eq!(picked(Some("0.9.0"), Target::LinuxX86_64), Some("0.9.0"));
    assert_eq!(picked(Some("0.9.2"), Target::LinuxX86_64), None);
    assert_eq!(picked(None, Target::WindowsArm64), None);
    for not in [
        "newest", "0", "0.", ".9", "0.9.x", "^0.9", "0.9.1.2", "v0.9", "0.-9",
    ] {
        assert_eq!(Wanted::parse(Some(not)), None, "{not}");
    }
    assert_eq!(Wanted::parse(Some("0.11")), Some(Wanted::Series(0, 11)));
    assert_eq!(read_index("format = 1\n"), Ok(Vec::new()));
    for (index, reason) in [
        ("format = 1\n[[engine]\n", "it is not valid TOML"),
        ("[[engine]]\nversion = \"0.9.0\"\n", "it has no `format`"),
        (
            "format = 2\n",
            "its `format` is not 1, the one this launcher reads",
        ),
        (
            "format = 1\nengine = 3\n",
            "its `engine` is not a list of tables",
        ),
        (
            "format = 1\n[[engine]]\nversion = \"0.9\"\nat = \"a\"\ntargets = []\n",
            "its engine 1 has no `version` of three numbers",
        ),
        (
            "format = 1\n[[engine]]\nversion = \"0.9.0\"\ntargets = []\n",
            "its engine 1 has no `at`",
        ),
        (
            "format = 1\n[[engine]]\nversion = \"0.9.0\"\nat = \"a\"\ntargets = [1]\n",
            "its engine 1 has no `targets`, a list of names",
        ),
    ] {
        assert_eq!(read_index(index), Err(reason.to_string()), "{index}");
    }
}

#[test]
fn a_mirror_replaces_the_host_and_nothing_else() {
    let none = |_: &str| None;
    assert_eq!(
        Source::of(&none, None),
        Ok(Source {
            base: "https://zeff-lang.dev/".to_string(),
            mirror: false
        })
    );
    let from_settings = Source::of(
        &none,
        Some("[network]\nmirror = \"https://mirror.univ.example/zeff\"\n"),
    )
    .unwrap();
    assert_eq!(
        (from_settings.base.as_str(), from_settings.mirror),
        ("https://mirror.univ.example/zeff/", true)
    );
    assert_eq!(
        from_settings.at("engines/index.toml"),
        "https://mirror.univ.example/zeff/engines/index.toml"
    );
    // The environment before the settings; an empty variable is no variable.
    let set = |name: &str| (name == "ZEFF_MIRROR").then(|| "http://10.0.0.7:8080/".to_string());
    let config = Some("[network]\nmirror = \"https://mirror.univ.example/zeff\"\n");
    assert_eq!(
        Source::of(&set, config).unwrap().base,
        "http://10.0.0.7:8080/"
    );
    let empty = |name: &str| (name == "ZEFF_MIRROR").then(String::new);
    assert_eq!(
        Source::of(&empty, config).unwrap().base,
        "https://mirror.univ.example/zeff/"
    );
    for not in [
        "mirror.univ.example",
        "ftp://mirror.univ.example",
        "https://",
        "http:///zeff",
    ] {
        let named = |name: &str| (name == "ZEFF_MIRROR").then(|| not.to_string());
        assert!(Source::of(&named, None).is_err(), "{not}");
    }
}

// ----- fetching -----

#[test]
fn an_engine_is_fetched_proved_and_installed() {
    let world = World::new(&["0.9.0", "0.9.1", "0.10.0", "0.11.0-rc.1"]);
    let mut out = String::new();
    let mut step = |title: &str, args: &[&str]| {
        let ended = world.zeff(args);
        assert_eq!(ended.code, 0, "{title}: {ended:?}");
        out.push_str(&shown(title, &ended));
        out.push_str(&format!(
            "  fetched: {}\n\n",
            world.server.asked().join(" ")
        ));
    };
    step("a version", &["engine", "install", "0.9.0"]);
    assert_eq!(
        std::fs::read(world.home().join("engines/0.9.0/zeff")).unwrap(),
        common::engine_script("0.9.0")
    );
    step("a series: the newest 0.9.x", &["engine", "install", "0.9"]);
    step("nothing named: the newest release", &["engine", "install"]);
    step(
        "one before its release, named in full",
        &["engine", "install", "0.11.0-rc.1"],
    );
    step(
        "one that is installed: no archive is fetched",
        &["engine", "install", "0.9.1"],
    );
    assert_eq!(
        world.installed(),
        ["0.10.0", "0.11.0-rc.1", "0.9.0", "0.9.1"]
    );
    assert!(
        std::fs::read_dir(world.home().join("cache"))
            .unwrap()
            .count()
            == 0,
        "what was downloaded is gone once it is installed"
    );
    out.push_str(&shown("list", &world.zeff(&["engine", "list"])));
    insta::assert_snapshot!(out);
}

/// The ruling "the launcher sends nothing but its request": the whole of what the server
/// receives, for the first file the launcher fetches.
#[test]
fn the_launcher_sends_its_request_and_nothing_else() {
    let world = World::new(&["0.9.1"]);
    assert_eq!(world.zeff(&["engine", "install", "0.9.1"]).code, 0);
    let heads = world.server.heads();
    assert_eq!(heads.len(), 6, "{heads:?}");
    for head in &heads {
        let mut lines: Vec<String> = head
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| world.clean(line).to_lowercase())
            .collect();
        let request = lines.remove(0);
        assert!(
            request.starts_with("get /") && request.ends_with(" http/1.1"),
            "{request}"
        );
        lines.sort();
        let names: Vec<&str> = lines
            .iter()
            .map(|line| line.split(':').next().unwrap_or_default())
            .collect();
        assert_eq!(names, ["accept", "host", "user-agent"], "{head}");
        assert!(
            lines[2] == format!("user-agent: zeff-launcher/{}", zeff_launcher::VERSION),
            "{head}"
        );
    }
}

#[test]
fn every_refusal_of_a_fetch_says_why_and_installs_nothing() {
    let world = World::new(&["0.9.0", "0.9.1", "0.10.0-rc.1"]);
    let index = world.site().join("engines/index.toml");
    let signed_index = std::fs::read_to_string(&index).unwrap();
    let mark = |path: &Path, mark: &str, text: &str| {
        let mut name = path.to_path_buf().into_os_string();
        name.push(mark);
        std::fs::write(PathBuf::from(name), text).unwrap();
    };
    let unmark = |path: &Path, mark: &str| {
        let mut name = path.to_path_buf().into_os_string();
        name.push(mark);
        std::fs::remove_file(PathBuf::from(name)).unwrap();
    };
    let mut out = String::new();
    let mut step = |title: &str, ended: Ended, code: u8| {
        assert_eq!(ended.code, code, "{title}: {ended:?}");
        assert!(ended.out.is_empty(), "{title}");
        out.push_str(&shown(title, &ended));
        assert!(
            world.installed().is_empty(),
            "{title}: {:?}",
            world.installed()
        );
    };
    let key = world.signer.public();
    step(
        "a version that is not published",
        world.zeff(&["engine", "install", "0.9.5"]),
        4,
    );
    step(
        "a series with no release",
        world.zeff(&["engine", "install", "0.10"]),
        4,
    );
    step(
        "a word that is no version",
        world.zeff(&["engine", "install", "newest"]),
        1,
    );
    step(
        "two versions",
        world.zeff(&["engine", "install", "0.9.0", "0.9.1"]),
        1,
    );
    world.server.asked();
    step(
        "frozen",
        world.zeff(&["engine", "install", "0.9.1", "--frozen"]),
        4,
    );
    step(
        "offline",
        world.zeff(&["--offline", "engine", "install"]),
        4,
    );
    step(
        "frozen by the environment",
        world.zeff_with(
            &["engine", "install", "0.9.1"],
            Some(&key),
            &[("ZEFF_FROZEN", "1")],
        ),
        4,
    );
    step(
        "no key built in",
        world.zeff_with(&["engine", "install", "0.9.1"], None, &[]),
        1,
    );
    step(
        "a mirror that is not an address",
        world.zeff_with(
            &["engine", "install"],
            Some(&key),
            &[("ZEFF_MIRROR", "mirror.univ.example")],
        ),
        1,
    );
    assert_eq!(
        world.server.asked(),
        Vec::<String>::new(),
        "none of these asks the server anything"
    );
    // The index: not there, refused by the server, without its signature, changed, signed by
    // another, and signed but not an index.
    mark(&index, ".status", "500");
    step(
        "the server fails",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    unmark(&index, ".status");
    std::fs::remove_file(&index).unwrap();
    step(
        "no index on the server",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    std::fs::write(&index, signed_index.replace("0.9.1", "0.9.7")).unwrap();
    step(
        "an index changed after it was signed",
        world.zeff(&["engine", "install", "0.9.7"]),
        1,
    );
    write_signed(&index, &signed_index, &Signer::new());
    step(
        "an index signed by another key",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    write_signed(&index, "format = 3\n", &world.signer);
    step(
        "an index of a later form",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    write_signed(&index, &signed_index, &world.signer);
    let signature = world.site().join("engines/index.toml.minisig");
    let kept = std::fs::read(&signature).unwrap();
    std::fs::remove_file(&signature).unwrap();
    step(
        "an index without its signature",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    std::fs::write(&signature, kept).unwrap();
    // The archive: changed on the server; and a release whose checksums another key signed.
    let archive = world
        .site()
        .join("engines/0.9.1/zeff-0.9.1-linux-x86-64.tar.zst");
    let good = std::fs::read(&archive).unwrap();
    let mut bad = good.clone();
    bad[good.len() / 2] ^= 0x01;
    std::fs::write(&archive, &bad).unwrap();
    step(
        "an archive changed on the server",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    std::fs::write(&archive, &good).unwrap();
    release(
        &world.site().join("engines/0.9.0"),
        "0.9.0",
        HOST,
        &Signer::new(),
    );
    step(
        "a release another key signed",
        world.zeff(&["engine", "install", "0.9.0"]),
        1,
    );
    std::fs::remove_file(&archive).unwrap();
    step(
        "an archive the server does not have",
        world.zeff(&["engine", "install", "0.9.1"]),
        1,
    );
    std::fs::write(&archive, &good).unwrap();
    insta::assert_snapshot!(out);
    // A download that stops half way: said in the system's words, so not in the snapshot.
    mark(&archive, ".cut", "");
    let cut = world.zeff(&["engine", "install", "0.9.1"]);
    assert_eq!(cut.code, 1, "{cut:?}");
    assert!(
        cut.err.starts_with(
            "error[Z0708]: `zeff-0.9.1-linux-x86-64.tar.zst` could not be fetched from `http://mirror/engines/0.9.1/zeff-0.9.1-linux-x86-64.tar.zst`: "
        ),
        "{}",
        cut.err
    );
    assert!(world.installed().is_empty());
    assert!(
        std::fs::read_dir(world.home().join("cache"))
            .map_or(true, |mut left| left.next().is_none())
    );
    // And once the server is whole again, the engine is installed.
    unmark(&archive, ".cut");
    assert_eq!(world.zeff(&["engine", "install", "0.9.1"]).code, 0);
    assert_eq!(world.installed(), ["0.9.1"]);
}

/// Without a mirror the index says where a release's files are: under the host, or at a whole
/// address of its own.
#[test]
fn the_index_says_where_a_releases_files_are() {
    let dir = tempfile::tempdir().unwrap();
    let signer = Signer::new();
    let root = dir.path().join("site");
    release(&root.join("releases/v0.9.1"), "0.9.1", HOST, &signer);
    release(&root.join("moved/0.9.2"), "0.9.2", HOST, &signer);
    let server = Server::start(&root);
    let index = format!(
        "format = 1\n\n[[engine]]\nversion = \"0.9.1\"\nat = \"releases/v0.9.1\"\ntargets = [\"linux-x86-64\"]\n\n\
         [[engine]]\nversion = \"0.9.2\"\nat = \"{}moved/0.9.2/\"\ntargets = [\"linux-x86-64\"]\n",
        server.base
    );
    write_signed(&root.join("engines/index.toml"), &index, &signer);
    let source = Source {
        base: server.base.clone(),
        mirror: false,
    };
    let http = Http::new(None);
    let home = dir.path().join("home");
    for (version, folder) in [("0.9.1", "/releases/v0.9.1/"), ("0.9.2", "/moved/0.9.2/")] {
        let wanted = Wanted::Exact(Version::parse(version).unwrap());
        let fetched = fetch_engine(&home, &wanted, &signer.public(), &source, &http, HOST).unwrap();
        assert!(matches!(fetched.placed, Placed::New(_)), "{version}");
        let asked = server.asked();
        assert_eq!(
            asked.last().map(String::as_str),
            Some(format!("{folder}zeff-{version}-linux-x86-64.tar.zst").as_str()),
            "{asked:?}"
        );
    }
}

// ----- the key list -----

#[test]
fn the_key_list_is_fetched_kept_and_never_replaced_by_an_older_one() {
    let dir = tempfile::tempdir().unwrap();
    let root_key = Signer::new();
    let second = Signer::new();
    let root = dir.path().join("site");
    // The publisher signs releases with the second key, which the key list names.
    site(&root, &["0.9.1", "0.9.2", "0.9.3"], HOST, &second);
    let list = |serial: u64, keys: &[&Signer]| {
        let mut text = format!("zeff keys 1\nserial {serial}\n");
        for key in keys {
            text.push_str(&format!("key {}\n", key.public()));
        }
        text
    };
    let list_path = root.join("keys/list.txt");
    write_signed(&list_path, &list(2, &[&second]), &root_key);
    let server = Server::start(&root);
    let source = Source {
        base: server.base.clone(),
        mirror: true,
    };
    let http = Http::new(None);
    let home = dir.path().join("home");
    let install = |version: &str| {
        let wanted = Wanted::Exact(Version::parse(version).unwrap());
        fetch_engine(&home, &wanted, &root_key.public(), &source, &http, HOST)
    };
    assert!(install("0.9.1").is_ok());
    assert_eq!(
        std::fs::read_to_string(home.join("keys/list.txt")).unwrap(),
        list(2, &[&second]),
        "the list is kept in the store"
    );
    // The server then offers an older list: refused, and the store's list stays.
    write_signed(&list_path, &list(1, &[&root_key]), &root_key);
    let older = install("0.9.2").unwrap_err();
    assert_eq!(
        older,
        zeff_launcher::remote::NotFetched::Untrusted(zeff_launcher::trust::Refusal::KeyListOlder {
            serial: 1,
            held: 2
        })
    );
    // The server has no list at all: the store's list is the list, so the second key still
    // signs, though the built-in key alone would not accept it.
    std::fs::remove_file(&list_path).unwrap();
    assert!(install("0.9.2").is_ok());
    let fresh_home = dir.path().join("another-home");
    let alone = fetch_engine(
        &fresh_home,
        &Wanted::Exact(Version::parse("0.9.3").unwrap()),
        &root_key.public(),
        &source,
        &http,
        HOST,
    );
    assert!(
        matches!(
            alone,
            Err(zeff_launcher::remote::NotFetched::Untrusted(
                zeff_launcher::trust::Refusal::UntrustedKey { by_list: false, .. }
            ))
        ),
        "{alone:?}"
    );
    // A newer list replaces the store's.
    write_signed(&list_path, &list(3, &[&second, &root_key]), &root_key);
    assert!(install("0.9.3").is_ok());
    assert!(
        std::fs::read_to_string(home.join("keys/list.txt"))
            .unwrap()
            .contains("serial 3")
    );
}

// ----- a pinned engine that is missing -----

fn project(root: &Path, pin: &str) -> PathBuf {
    let dir = root.join("study");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(
        dir.join("zeff.toml"),
        format!("[project]\nname = \"study\"\nzeff = \"{pin}\"\n"),
    )
    .unwrap();
    dir
}

/// The step's finish line: a project pinned to an engine that is not installed runs after one
/// answer.
#[test]
fn a_project_pinned_to_a_missing_engine_runs_after_one_answer() {
    let world = World::new(&["0.9.0", "0.9.1"]);
    let study = project(world.dir.path(), "0.9.1");
    let key = world.signer.public();
    let env = world.env(&[]);
    let machine = Machine {
        env: &env,
        cwd: &study,
        platform: Platform::Unix,
        host: Some(HOST),
        key: Some(&key),
        fetch: None,
    };
    let args = strings(&["run", "scripts/main.zeff"]);
    let pinned = |world: &World| match choose(&args, &study, &world.env(&[]), Platform::Unix) {
        Choice::PinMissing { version, .. } => Err(version),
        Choice::Run { version, engine } => Ok((version, engine)),
        other => panic!("{other:?}"),
    };
    assert_eq!(pinned(&world), Err("0.9.1".to_string()));
    let mut asked = Vec::new();
    let mut err = Vec::new();
    // Off a terminal, with no flag: nothing is asked and nothing is fetched.
    assert_eq!(
        install_pinned("0.9.1", &args, &machine, None, &mut err),
        Ok(None)
    );
    // On a terminal, the answer no.
    let mut no = |question: &str| {
        asked.push(question.to_string());
        "n\n".to_string()
    };
    assert_eq!(
        install_pinned("0.9.1", &args, &machine, Some(&mut no), &mut err),
        Ok(None)
    );
    assert_eq!(world.server.asked(), Vec::<String>::new());
    assert!(err.is_empty() && world.installed().is_empty());
    // One answer, yes: the engine is installed and the project runs on it.
    let mut yes = |question: &str| {
        asked.push(question.to_string());
        "y\n".to_string()
    };
    let engine = install_pinned("0.9.1", &args, &machine, Some(&mut yes), &mut err)
        .expect("installed")
        .expect("an engine");
    assert_eq!(
        asked,
        ["this project needs Zeff 0.9.1, which is not installed. Install it now? [Y/n] "; 2]
    );
    assert_eq!(String::from_utf8_lossy(&err), "installed Zeff 0.9.1\n");
    assert_eq!(pinned(&world), Ok(("0.9.1".to_string(), engine.clone())));
    #[cfg(unix)]
    {
        let ran = std::process::Command::new(&engine)
            .args(for_the_engine(&args))
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&ran.stdout),
            "engine 0.9.1: run scripts/main.zeff\n"
        );
    }
}

#[test]
fn a_flag_installs_without_a_question_and_another_forbids_it() {
    let world = World::new(&["0.9.0", "0.9.1"]);
    let study = project(world.dir.path(), "0.9.0");
    let key = world.signer.public();
    let mut out = String::new();
    let mut never = |_: &str| -> String { panic!("no question under a flag") };
    for (title, args, more, installs) in [
        ("frozen", &["run", "--frozen"][..], &[][..], false),
        (
            "offline, with `--install`",
            &["--offline", "run", "--install"],
            &[],
            false,
        ),
        (
            "offline by the environment",
            &["run", "-y"],
            &[("ZEFF_OFFLINE", "1")],
            false,
        ),
        (
            "`--install` after `--` is the script's",
            &["run", "--", "--install"],
            &[],
            false,
        ),
        ("`--install`", &["run", "--install"], &[], true),
    ] {
        let env = world.env(more);
        let machine = Machine {
            env: &env,
            cwd: &study,
            platform: Platform::Unix,
            host: Some(HOST),
            key: Some(&key),
            fetch: None,
        };
        let mut err = Vec::new();
        let off_a_terminal = title.starts_with("`--install` after");
        let result = install_pinned(
            "0.9.0",
            &strings(args),
            &machine,
            if off_a_terminal {
                None
            } else {
                Some(&mut never)
            },
            &mut err,
        );
        let said = world.clean(&String::from_utf8(err).unwrap());
        out.push_str(&format!(
            "----- {title}: {}\n{said}\n",
            match &result {
                Ok(Some(_)) => "installed".to_string(),
                Ok(None) => "not asked for".to_string(),
                Err(code) => format!("exit {code}"),
            }
        ));
        assert_eq!(
            result.as_ref().is_ok_and(Option::is_some),
            installs,
            "{title}"
        );
        assert_eq!(!world.installed().is_empty(), installs, "{title}");
        assert_eq!(world.server.asked().is_empty(), !installs, "{title}");
    }
    // `-y` and `--yes` likewise; the launcher's flags are not the engine's to see.
    let study = project(&world.dir.path().join("second"), "0.9.1");
    let env = world.env(&[]);
    let machine = Machine {
        env: &env,
        cwd: &study,
        platform: Platform::Unix,
        host: Some(HOST),
        key: Some(&key),
        fetch: None,
    };
    let installed = install_pinned(
        "0.9.1",
        &strings(&["-y", "run"]),
        &machine,
        None,
        &mut Vec::new(),
    );
    assert!(matches!(installed, Ok(Some(_))), "{installed:?}");
    assert_eq!(
        for_the_engine(&strings(&[
            "-y",
            "run",
            "--install",
            "main.zeff",
            "--yes",
            "--",
            "-y"
        ])),
        strings(&["run", "main.zeff", "--", "-y"])
    );
    insta::assert_snapshot!(out);
}

/// The network itself, over TLS with this system's certificates: run by hand, with
/// `ZEFF_TRIAL_URL` an address that is served over HTTPS. The tests above speak plain HTTP to
/// a server on this machine, which shows everything but this.
#[test]
#[ignore = "asks a server outside this machine"]
fn the_network_itself_speaks_tls_with_the_systems_certificates() {
    use zeff_launcher::net::{Fetch, FetchError, SMALL};
    let url = std::env::var("ZEFF_TRIAL_URL").expect("ZEFF_TRIAL_URL");
    let http = Http::new(None);
    let bytes = http.get(&url, SMALL).expect("an answer");
    println!("{} bytes from {url}", bytes.len());
    assert!(!bytes.is_empty());
    // A certificate that is not the host's is refused, whatever is behind it.
    if let Ok(wrong) = std::env::var("ZEFF_TRIAL_WRONG_HOST") {
        let refused = http.get(&wrong, SMALL);
        println!("{wrong}: {refused:?}");
        assert!(matches!(refused, Err(FetchError::Failed(_))), "{refused:?}");
    }
}
