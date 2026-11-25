//! What the coverage report of release 0.9.0 showed that no test had made the launcher say or
//! decide (step 42c3; the weekly run of 24 November 2026): messages built from a refusal that
//! no archive or server of the other tests brings about, each printed here from the refusal
//! itself; the order of versions before their release; the command of a command line; the
//! engine that answers `zeff audit`; an index that is not one; and the lines of a checksum
//! file that are not lines of one.

mod common;

use std::cmp::Ordering;
use std::path::PathBuf;
use zeff_diagnostics::Sources;
use zeff_launcher::archive::{Fault, Target};
use zeff_launcher::engines::{Machine, command_of, engine_for_audit, install_pinned, not_fetched};
use zeff_launcher::install::{Refused, said, untrusted};
use zeff_launcher::net::{FetchError, Source};
use zeff_launcher::remote::{NotFetched, Wanted, read_index};
use zeff_launcher::trust::{Refusal, Trust, listed_checksum};
use zeff_launcher::{Platform, Version, engine_in, pin};

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}

#[test]
fn the_messages_of_refusals_no_other_test_brings_about() {
    let mut out = String::new();
    let mut add = |title: &str, text: String| {
        out.push_str(&format!("----- {title}\n{text}\n"));
    };
    let plain = Sources::new();
    add(
        "a key on the key list that cannot be read",
        untrusted(&Refusal::KeyUnreadable { on_list: true }).render(&plain),
    );
    let archive = "zeff-0.9.1-linux-x86_64.tar.zst".to_string();
    let faults = [
        (
            "an entry that is a link",
            Fault::Unsafe {
                entry: "zeff-0.9.1/bin/zeff".to_string(),
                reason: "it is a link",
            },
        ),
        (
            "a folder in the way",
            Fault::InTheWay {
                folder: PathBuf::from("/home/ada/.zeff/engines/0.9.1"),
            },
        ),
        (
            "a store that cannot be written",
            Fault::Store {
                path: PathBuf::from("/home/ada/.zeff/engines"),
                reason: "this user may not".to_string(),
            },
        ),
    ];
    for (title, fault) in faults {
        add(
            title,
            said(&Refused::NotUnpacked {
                archive: archive.clone(),
                fault,
            })
            .render(&plain),
        );
    }
    let publisher = Source {
        base: "https://zeff-lang.dev/".to_string(),
        mirror: false,
    };
    let not_fetched_cases = [
        (
            "a file larger than anything the launcher fetches, from the publisher's host",
            NotFetched::Network {
                what: "the index".to_string(),
                url: "https://zeff-lang.dev/engines.toml".to_string(),
                error: FetchError::TooLarge,
            },
        ),
        (
            "no release at all for this kind of machine",
            NotFetched::NoSuchVersion {
                wanted: Wanted::Newest,
                host: Target::LinuxArm64,
                published: Vec::new(),
            },
        ),
        (
            "a store that cannot hold the download",
            NotFetched::Store {
                path: PathBuf::from("/home/ada/.zeff/downloads"),
                reason: "the disk is full".to_string(),
            },
        ),
    ];
    for (title, refused) in not_fetched_cases {
        let (diagnostic, code) = not_fetched(&refused, &publisher);
        add(&format!("{title} [exit {code}]"), diagnostic.render(&plain));
    }
    insta::assert_snapshot!(out);
}

#[test]
fn what_is_wanted_in_words() {
    assert_eq!(Wanted::Newest.words(), "a release");
    assert_eq!(Wanted::Series(0, 9).words(), "0.9.x");
    assert_eq!(
        Wanted::Exact(Version::parse("0.9.1").unwrap()).words(),
        "0.9.1"
    );
}

#[test]
fn a_version_before_its_release_is_ordered_whichever_side_it_stands_on() {
    let v = |text: &str| Version::parse(text).unwrap_or_else(|| panic!("{text}"));
    assert_eq!(v("0.9.1").cmp(&v("0.9.1")), Ordering::Equal);
    assert_eq!(v("0.9.1-dev").cmp(&v("0.9.1")), Ordering::Less);
    assert_eq!(v("0.9.1").cmp(&v("0.9.1-dev")), Ordering::Greater);
    // Two versions before the same release, by their words.
    assert_eq!(v("0.9.1-dev").cmp(&v("0.9.1-rc")), Ordering::Less);
    assert_eq!(v("0.9.1-rc").cmp(&v("0.9.1-dev")), Ordering::Greater);
    assert_eq!(v("0.9.1-dev").cmp(&v("0.9.1-dev")), Ordering::Equal);
    // The numbers come first: a version before a later release is the later one.
    assert_eq!(v("0.9.2-dev").cmp(&v("0.9.1")), Ordering::Greater);
    // A number no machine word holds is not a version.
    assert!(Version::parse("99999999999999999999999.0.0").is_none());
    assert!(Version::parse("0.9.1-").is_none());
}

#[test]
fn a_pin_is_text_under_the_project_table_and_nothing_else() {
    assert_eq!(
        pin("[project]\nzeff = \"0.9.1\"\n"),
        Some("0.9.1".to_string())
    );
    // `project` that is not a table, a pin that is not text, and a file that is not TOML.
    assert_eq!(pin("project = 3\n"), None);
    assert_eq!(pin("[project]\nzeff = 9\n"), None);
    assert_eq!(pin("[project\n"), None);
}

#[test]
fn the_command_is_the_first_word_that_is_no_option_and_no_value_of_one() {
    let command = |args: &[&str]| command_of(&strings(args)).map(str::to_string);
    assert_eq!(command(&["audit"]), Some("audit".to_string()));
    assert_eq!(
        command(&["--quiet", "run", "a.zeff"]),
        Some("run".to_string())
    );
    // An option that takes a value takes the next word, unless the value is joined to it.
    assert_eq!(
        command(&["--project", "study", "audit"]),
        Some("audit".to_string())
    );
    assert_eq!(
        command(&["--project=study", "audit"]),
        Some("audit".to_string())
    );
    // After `--` the next word is the command whatever it looks like; `-` alone is a word.
    assert_eq!(command(&["--", "--run"]), Some("--run".to_string()));
    assert_eq!(command(&["-"]), Some("-".to_string()));
    assert_eq!(command(&["--quiet"]), None);
    assert_eq!(command(&["--"]), None);
    assert_eq!(command(&[]), None);
}

/// A store with an engine file for each version named.
fn store_with(versions: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for version in versions {
        let engine = engine_in(dir.path(), version, Platform::Unix);
        std::fs::create_dir_all(engine.parent().unwrap()).unwrap();
        std::fs::write(&engine, common::engine_script(version)).unwrap();
    }
    dir
}

#[test]
fn audit_is_answered_by_the_newest_release_installed() {
    let dir = store_with(&["0.9.0", "0.9.1", "0.9.2-dev"]);
    let home = dir.path().display().to_string();
    let env = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
    let (version, engine) = engine_for_audit(&env, Platform::Unix).expect("an engine");
    assert_eq!(version, "0.9.1");
    assert_eq!(engine, engine_in(dir.path(), "0.9.1", Platform::Unix));
    // Nothing installed, and no home folder at all: no engine answers.
    let empty = tempfile::tempdir().unwrap();
    let home = empty.path().display().to_string();
    let env = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
    assert_eq!(engine_for_audit(&env, Platform::Unix), None);
    assert_eq!(engine_for_audit(&|_| None, Platform::Unix), None);
}

#[test]
fn a_pin_is_not_installed_without_a_home_or_for_a_pin_that_is_no_version() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().display().to_string();
    let with_home = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
    let without = |_: &str| None;
    let mut err = Vec::new();
    let pinned = |env: &dyn Fn(&str) -> Option<String>, version: &str, err: &mut Vec<u8>| {
        install_pinned(
            version,
            &strings(&["run"]),
            &Machine {
                env,
                cwd: std::path::Path::new("."),
                platform: Platform::Unix,
                host: Some(Target::LinuxX86_64),
                key: None,
                fetch: None,
            },
            None,
            err,
        )
    };
    assert_eq!(pinned(&without, "0.9.1", &mut err), Ok(None));
    assert_eq!(pinned(&with_home, ">=0.9", &mut err), Ok(None));
    assert!(err.is_empty());
}

#[test]
fn an_index_whose_engines_are_not_tables_or_whose_targets_are_not_names_is_refused() {
    assert_eq!(
        read_index("format = 1\nengine = [1, 2]\n"),
        Err("its `engine` is not a list of tables".to_string())
    );
    assert_eq!(
        read_index("format = 1\nengine = 3\n"),
        Err("its `engine` is not a list of tables".to_string())
    );
    assert_eq!(
        read_index(
            "format = 1\n[[engine]]\nversion = \"0.9.1\"\nat = \"0.9.1/\"\ntargets = \"linux-x86_64\"\n"
        ),
        Err("its engine 1 has no `targets`, a list of names".to_string())
    );
    assert_eq!(
        read_index("format = 1\n[[engine]]\nversion = \"0.9.1\"\nat = \"0.9.1/\"\ntargets = [1]\n"),
        Err("its engine 1 has no `targets`, a list of names".to_string())
    );
}

#[test]
fn a_line_that_is_not_a_checksum_line_lists_nothing() {
    let sum = "a".repeat(64);
    // The two forms of `sha256sum`: two spaces, and a space and a star.
    assert_eq!(
        listed_checksum(&format!("{sum}  zeff.tar.zst\n"), "zeff.tar.zst"),
        Some(sum.clone())
    );
    assert_eq!(
        listed_checksum(&format!("{sum} *zeff.tar.zst\n"), "zeff.tar.zst"),
        Some(sum.clone())
    );
    // No space at all, and one space with neither a second nor a star.
    assert_eq!(listed_checksum(&format!("{sum}\n"), "zeff.tar.zst"), None);
    assert_eq!(
        listed_checksum(&format!("{sum} zeff.tar.zst\n"), "zeff.tar.zst"),
        None
    );
    // Such lines are passed over: a good line after them is still found.
    assert_eq!(
        listed_checksum(
            &format!("a comment\n\n{sum}  zeff.tar.zst\n"),
            "zeff.tar.zst"
        ),
        Some(sum)
    );
}

#[test]
fn a_set_of_keys_shows_how_many_it_holds_and_never_the_keys() {
    let key = common::Signer::new().public();
    let shown = format!("{:?}", Trust::of_key(&key).unwrap());
    assert_eq!(shown, "Trust { 1 keys, serial None }");
}
