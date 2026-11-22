//! Archives and their signatures (Engine and Project Specification, section 2.1, "Trust";
//! release 0.9.0, step 18): an engine is installed only from an archive proved to be the
//! publisher's, and whatever is refused leaves the store untouched. Every key here is made in
//! the test and thrown away with it; the launcher has none of them built in.

mod common;

use common::{Signer, archive, engine_script, release, version};
use std::path::{Path, PathBuf};
use zeff_launcher::archive::{ArchiveName, Contents, Kind, Placed, Target, pack};
use zeff_launcher::install::{Refused, install_from, said};
use zeff_launcher::trust::{
    BUILT_IN_KEY, ENGINE_INDEX, HOST, KEY_LIST, Refusal, Trust, listed_checksum, sha256_hex,
    sums_of,
};
use zeff_launcher::{Platform, Store, find_engine, installed};

fn nothing_installed(home: &Path) -> bool {
    let store = Store {
        user: home.to_path_buf(),
        system: None,
    };
    installed(&store, Platform::Unix).is_empty()
        && installed(&store, Platform::Windows).is_empty()
        && std::fs::read_dir(home.join("engines"))
            .map(|entries| entries.count() == 0)
            .unwrap_or(true)
}

// ----- names and checksums -----

#[test]
fn an_archives_name_says_its_version_machine_and_kind() {
    for target in Target::ALL {
        for text in ["0.9.1", "0.10.0-rc.1", "1.0.0-dev"] {
            let name = ArchiveName::of(&version(text), target);
            assert_eq!(ArchiveName::parse(&name.file), Some(name.clone()));
            assert_eq!(name.version.text, text);
            assert_eq!(
                name.kind,
                if target.platform() == Platform::Windows {
                    Kind::Zip
                } else {
                    Kind::TarZst
                }
            );
        }
    }
    let name = ArchiveName::parse("zeff-0.9.1-macos-arm64.tar.zst").unwrap();
    assert_eq!(
        (name.target, name.top()),
        (Target::MacosArm64, "zeff-0.9.1-macos-arm64")
    );
    assert_eq!(
        ArchiveName::parse("zeff-0.9.1-windows-x86-64.zip")
            .unwrap()
            .target,
        Target::WindowsX86_64
    );
    for not in [
        "zeff-0.9.1-macos-arm64.tar.gz",
        "zeff-0.9.1-macos-arm64",
        "zeff-0.9-macos-arm64.tar.zst",
        "zeff-0.9.1-solaris-sparc.tar.zst",
        "zeff-0.9.1macos-arm64.tar.zst",
        "zemp-0.9.1-macos-arm64.tar.zst",
        "zeff--macos-arm64.zip",
        "SHA256SUMS",
        "",
    ] {
        assert_eq!(ArchiveName::parse(not), None, "{not}");
    }
}

#[test]
fn the_same_contents_give_the_same_archive_and_its_checksum_is_listed() {
    for target in [Target::LinuxArm64, Target::WindowsArm64] {
        let (name, bytes) = archive("0.9.1", target);
        assert_eq!(archive("0.9.1", target).1, bytes, "{}", name.file);
        let other = archive("0.9.2", target);
        let sums = sums_of(&[(&name.file, &bytes), (&other.0.file, &other.1)]);
        assert_eq!(
            listed_checksum(&sums, &name.file),
            Some(sha256_hex(&bytes)),
            "{sums}"
        );
        assert_eq!(
            listed_checksum(&sums, &other.0.file),
            Some(sha256_hex(&other.1))
        );
        assert_eq!(
            listed_checksum(&sums, "zeff-0.9.3-linux-arm64.tar.zst"),
            None
        );
        // The line of another file whose name ends with this one's is not this one's.
        assert_eq!(listed_checksum(&sums, &name.file[1..]), None);
    }
    // The checksum of nothing, as `sha256sum` prints it; the binary form of a line.
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    let line = format!("{} *zeff.zip\n", sha256_hex(b"x").to_uppercase());
    assert_eq!(listed_checksum(&line, "zeff.zip"), Some(sha256_hex(b"x")));
    assert_eq!(listed_checksum("abc  zeff.zip\n", "zeff.zip"), None);
}

// ----- an archive that is the publisher's -----

#[test]
fn a_signed_archive_is_installed_and_found() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let signer = Signer::new();
    let trust = Trust::of_key(&signer.public()).unwrap();
    let path = release(
        &dir.path().join("downloads"),
        "0.9.1",
        Target::MacosArm64,
        &signer,
    );
    let engine = home.join("engines").join("0.9.1").join("zeff");
    assert_eq!(
        install_from(&home, &path, &trust, Target::MacosArm64),
        Ok(Placed::New(engine.clone()))
    );
    let store = Store {
        user: home.clone(),
        system: None,
    };
    assert_eq!(
        find_engine(&store, "0.9.1", Platform::Unix),
        Some(engine.clone())
    );
    assert_eq!(std::fs::read(&engine).unwrap(), engine_script("0.9.1"));
    assert!(
        !home.join("bin").exists(),
        "the engine alone: the archive's launcher is not this store's"
    );
    // A second time it says the engine is there and changes nothing.
    assert_eq!(
        install_from(&home, &path, &trust, Target::MacosArm64),
        Ok(Placed::Already(engine.clone()))
    );
    // Another version beside it; the first is as it was.
    let next = release(
        &dir.path().join("downloads-2"),
        "0.10.0-rc.1",
        Target::MacosArm64,
        &signer,
    );
    assert!(matches!(
        install_from(&home, &next, &trust, Target::MacosArm64),
        Ok(Placed::New(_))
    ));
    let listed: Vec<String> = installed(&store, Platform::Unix)
        .into_iter()
        .map(|version| version.text)
        .collect();
    assert_eq!(listed, ["0.9.1", "0.10.0-rc.1"]);
}

#[cfg(unix)]
#[test]
fn an_installed_engine_runs() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let signer = Signer::new();
    let host = Target::host().expect("a machine engines are built for");
    let path = release(&dir.path().join("downloads"), "0.9.1", host, &signer);
    let trust = Trust::of_key(&signer.public()).unwrap();
    let Ok(Placed::New(engine)) = install_from(&home, &path, &trust, host) else {
        panic!("not installed");
    };
    let out = std::process::Command::new(engine)
        .args(["run", "main.zeff"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "engine 0.9.1: run main.zeff\n"
    );
}

#[test]
fn a_zip_is_installed_as_a_tar_is() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let signer = Signer::new();
    let trust = Trust::of_key(&signer.public()).unwrap();
    let path = release(
        &dir.path().join("downloads"),
        "0.9.1",
        Target::WindowsX86_64,
        &signer,
    );
    let engine = home.join("engines").join("0.9.1").join("zeff.exe");
    assert_eq!(
        install_from(&home, &path, &trust, Target::WindowsX86_64),
        Ok(Placed::New(engine.clone()))
    );
    assert_eq!(std::fs::read(&engine).unwrap(), engine_script("0.9.1"));
    assert_eq!(
        std::fs::read(engine.with_file_name("LICENSE")).unwrap(),
        b"the licence\n"
    );
}

/// A release made by other programs than the launcher's own packer, read by the launcher: run
/// by hand, and by the release workflow on the archives it has just signed (step 23), with
/// `ZEFF_TRIAL_ARCHIVE` the archive's path (its `SHA256SUMS` and signature beside it) and
/// `ZEFF_TRIAL_KEY` the public key in base64. A copy with one byte changed must be refused.
#[test]
#[ignore = "needs an archive and a key made outside the tests"]
fn an_archive_signed_by_the_minisign_program_is_installed_and_a_tampered_copy_refused() {
    let path = PathBuf::from(std::env::var("ZEFF_TRIAL_ARCHIVE").expect("ZEFF_TRIAL_ARCHIVE"));
    let key = std::env::var("ZEFF_TRIAL_KEY").expect("ZEFF_TRIAL_KEY");
    let trust = Trust::of_key(&key).expect("a minisign public key");
    let host = Target::host().expect("a machine engines are built for");
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    // The tampered copy first, in a folder of its own with the same checksums and signature.
    let copies = dir.path().join("tampered");
    std::fs::create_dir_all(&copies).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x01;
    let copy = copies.join(path.file_name().unwrap());
    std::fs::write(&copy, &bytes).unwrap();
    for file in ["SHA256SUMS", "SHA256SUMS.minisig"] {
        std::fs::copy(path.with_file_name(file), copies.join(file)).unwrap();
    }
    assert!(matches!(
        install_from(&home, &copy, &trust, host),
        Err(Refused::Untrusted(Refusal::ChecksumDiffers { .. }))
    ));
    assert!(nothing_installed(&home));
    let started = std::time::Instant::now();
    let Ok(Placed::New(engine)) = install_from(&home, &path, &trust, host) else {
        panic!("not installed");
    };
    println!(
        "installed {} in {:.2} s",
        engine.display(),
        started.elapsed().as_secs_f64()
    );
    let out = std::process::Command::new(&engine)
        .arg("version")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&out.stdout));
    assert!(out.status.success(), "the installed engine runs");
}

// ----- what is refused -----

#[test]
fn every_refusal_says_why_and_unpacks_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let signer = Signer::new();
    let stranger = Signer::new();
    let trust = Trust::of_key(&signer.public()).unwrap();
    let host = Target::LinuxX86_64;
    let out = std::cell::RefCell::new(String::new());
    let case = |title: &str, result: Result<Placed, Refused>| {
        let refused = result.expect_err(title);
        let text = said(&refused)
            .render(&zeff_diagnostics::Sources::new())
            .replace(&dir.path().display().to_string(), "<tmp>")
            .replace('\\', "/");
        out.borrow_mut()
            .push_str(&format!("----- {title}\n{text}\n"));
        assert!(nothing_installed(&home), "{title}: something was unpacked");
        refused
    };
    let fresh = |folder: &str| {
        let folder = dir.path().join(folder);
        (release(&folder, "0.9.1", host, &signer), folder)
    };

    // A tampered archive: one byte of it changed after the checksums were signed.
    let (path, _) = fresh("tampered");
    let mut bytes = std::fs::read(&path).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x01;
    std::fs::write(&path, &bytes).unwrap();
    let refused = case(
        "a tampered archive",
        install_from(&home, &path, &trust, host),
    );
    assert!(matches!(
        refused,
        Refused::Untrusted(Refusal::ChecksumDiffers { .. })
    ));

    // A truncated download: the archive stops early.
    let (path, _) = fresh("truncated");
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..bytes.len() - 7]).unwrap();
    let refused = case(
        "a truncated download",
        install_from(&home, &path, &trust, host),
    );
    assert!(matches!(
        refused,
        Refused::Untrusted(Refusal::ChecksumDiffers { .. })
    ));

    // A wrong signature: the checksums were changed to fit a changed archive, and the
    // signature is the old one.
    let (path, folder) = fresh("resigned");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[middle] ^= 0x01;
    std::fs::write(&path, &bytes).unwrap();
    let file = path.file_name().unwrap().to_str().unwrap();
    std::fs::write(folder.join("SHA256SUMS"), sums_of(&[(file, &bytes)])).unwrap();
    let refused = case(
        "a wrong signature",
        install_from(&home, &path, &trust, host),
    );
    assert_eq!(
        refused,
        Refused::Untrusted(Refusal::SignatureMismatch {
            of: "`SHA256SUMS`".to_string()
        })
    );

    // Everything in order, and signed by someone else.
    let folder = dir.path().join("stranger");
    let path = release(&folder, "0.9.1", host, &stranger);
    let refused = case(
        "signed with a key the launcher does not have",
        install_from(&home, &path, &trust, host),
    );
    assert_eq!(
        refused,
        Refused::Untrusted(Refusal::UntrustedKey {
            of: "`SHA256SUMS`".to_string(),
            by_list: false
        })
    );

    // The same archive when the launcher has a key list: a key not on the list.
    let list = format!("zeff keys 1\nserial 1\nkey {}\n", signer.public());
    let listed =
        Trust::of_key_list(&signer.public(), &list, &signer.sign(list.as_bytes()), None).unwrap();
    let refused = case(
        "signed with a key that is not on the key list",
        install_from(&home, &path, &listed, host),
    );
    assert_eq!(
        refused,
        Refused::Untrusted(Refusal::UntrustedKey {
            of: "`SHA256SUMS`".to_string(),
            by_list: true
        })
    );

    // A signature cut short, and one that is not a signature.
    let (path, folder) = fresh("half-signature");
    let signature = std::fs::read_to_string(folder.join("SHA256SUMS.minisig")).unwrap();
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        &signature[..signature.len() / 2],
    )
    .unwrap();
    case(
        "a signature cut short",
        install_from(&home, &path, &trust, host),
    );
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        "<html>not found</html>\n",
    )
    .unwrap();
    case(
        "a page where the signature should be",
        install_from(&home, &path, &trust, host),
    );

    // The checksums of another release.
    let (path, folder) = fresh("other-sums");
    let (other, other_bytes) = archive("0.9.2", host);
    let sums = sums_of(&[(&other.file, &other_bytes)]);
    std::fs::write(folder.join("SHA256SUMS"), &sums).unwrap();
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        signer.sign(sums.as_bytes()),
    )
    .unwrap();
    case(
        "the checksums of another release",
        install_from(&home, &path, &trust, host),
    );

    // A launcher with no key built in proves nothing.
    assert_eq!(BUILT_IN_KEY, None, "no key before the release workflow's");
    let (path, _) = fresh("no-key");
    let no_key = Trust::built_in().expect_err("no key is built in yet");
    out.borrow_mut().push_str(&format!(
        "----- a launcher built without a key\n{}\n",
        zeff_launcher::install::untrusted(&no_key).render(&zeff_diagnostics::Sources::new())
    ));

    // The archive alone, without its checksums; a file that is no archive; another machine's.
    std::fs::remove_file(path.with_file_name("SHA256SUMS")).unwrap();
    case(
        "an archive without its checksums",
        install_from(&home, &path, &trust, host),
    );
    let renamed = path.with_file_name("engine.tar.zst");
    std::fs::rename(&path, &renamed).unwrap();
    case(
        "a file renamed",
        install_from(&home, &renamed, &trust, host),
    );
    let (path, _) = fresh("other-machine");
    case(
        "an engine for another machine",
        install_from(&home, &path, &trust, Target::MacosArm64),
    );

    // An archive that is the publisher's and holds no engine: proved, and still not installed.
    let folder = dir.path().join("empty");
    std::fs::create_dir_all(&folder).unwrap();
    let name = ArchiveName::of(&version("0.9.1"), host);
    let wrong = ArchiveName::of(&version("0.9.2"), host);
    // Packed under another version's name, so the engine is not where this name puts it.
    let mut bytes = pack(
        &wrong,
        &Contents {
            engine: b"engine",
            launcher: None,
            beside: &[],
        },
    )
    .unwrap();
    let sums = sums_of(&[(&name.file, &bytes)]);
    std::fs::write(folder.join(&name.file), &bytes).unwrap();
    std::fs::write(folder.join("SHA256SUMS"), &sums).unwrap();
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        signer.sign(sums.as_bytes()),
    )
    .unwrap();
    case(
        "a signed archive with no engine in it",
        install_from(&home, &folder.join(&name.file), &trust, host),
    );
    // And one that is signed and is not an archive.
    bytes.truncate(bytes.len() / 2);
    let sums = sums_of(&[(&name.file, &bytes)]);
    std::fs::write(folder.join(&name.file), &bytes).unwrap();
    std::fs::write(folder.join("SHA256SUMS"), &sums).unwrap();
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        signer.sign(sums.as_bytes()),
    )
    .unwrap();
    case(
        "a signed archive that is half an archive",
        install_from(&home, &folder.join(&name.file), &trust, host),
    );
    insta::assert_snapshot!(out.into_inner());
}

// ----- the key list -----

fn key_list(serial: u64, keys: &[&Signer]) -> String {
    let mut list = format!("# the keys engines are signed with\nzeff keys 1\nserial {serial}\n");
    for key in keys {
        list.push_str(&format!("key {}\n", key.public()));
    }
    list
}

#[test]
fn the_key_list_replaces_a_key_without_a_new_launcher() {
    let dir = tempfile::tempdir().unwrap();
    let host = Target::LinuxArm64;
    let root = Signer::new();
    let second = Signer::new();
    let by_root = release(&dir.path().join("by-root"), "0.9.1", host, &root);
    let by_second = release(&dir.path().join("by-second"), "0.9.2", host, &second);
    let install = |trust: &Trust, path: &Path, home: &str| {
        install_from(&dir.path().join(home), path, trust, host)
    };

    // Without a list, the built-in key alone: the second key signs nothing yet.
    let alone = Trust::of_key(&root.public()).unwrap();
    assert_eq!(alone.serial(), None);
    assert!(install(&alone, &by_root, "a").is_ok());
    assert!(matches!(
        install(&alone, &by_second, "a"),
        Err(Refused::Untrusted(Refusal::UntrustedKey {
            by_list: false,
            ..
        }))
    ));

    // A list signed by the built-in key that names both: both sign.
    let both = key_list(2, &[&root, &second]);
    let trust =
        Trust::of_key_list(&root.public(), &both, &root.sign(both.as_bytes()), None).unwrap();
    assert_eq!(trust.serial(), Some(2));
    assert!(install(&trust, &by_root, "b").is_ok());
    assert!(install(&trust, &by_second, "b").is_ok());

    // A later list that names the second alone: the first key signs nothing any more, though
    // it still signs the list.
    let rotated = key_list(3, &[&second]);
    let trust = Trust::of_key_list(
        &root.public(),
        &rotated,
        &root.sign(rotated.as_bytes()),
        Some(2),
    )
    .unwrap();
    assert!(install(&trust, &by_second, "c").is_ok());
    assert_eq!(
        install(&trust, &by_root, "c"),
        Err(Refused::Untrusted(Refusal::UntrustedKey {
            of: "`SHA256SUMS`".to_string(),
            by_list: true
        }))
    );
    assert!(nothing_installed_of(&dir.path().join("c"), "0.9.1"));
}

fn nothing_installed_of(home: &Path, version: &str) -> bool {
    !home.join("engines").join(version).exists()
}

#[test]
fn a_key_list_is_refused_unless_the_built_in_key_signed_it_as_it_stands() {
    let root = Signer::new();
    let second = Signer::new();
    let list = key_list(5, &[&root, &second]);
    let signature = root.sign(list.as_bytes());
    let read = |list: &str, signature: &str, held: Option<u64>| {
        Trust::of_key_list(&root.public(), list, signature, held).map(|trust| trust.serial())
    };
    assert_eq!(read(&list, &signature, None), Ok(Some(5)));
    // The same list again, and a newer store's: never an older list than the one held.
    assert_eq!(read(&list, &signature, Some(5)), Ok(Some(5)));
    let mut out = String::new();
    let mut case = |title: &str, result: Result<Option<u64>, Refusal>| {
        let refusal = result.expect_err(title);
        out.push_str(&format!(
            "----- {title}\n{}\n",
            zeff_launcher::install::untrusted(&refusal).render(&zeff_diagnostics::Sources::new())
        ));
        refusal
    };
    assert_eq!(
        case("an older list", read(&list, &signature, Some(6))),
        Refusal::KeyListOlder { serial: 5, held: 6 }
    );
    // A list the second key signed for itself: only the built-in key makes a list.
    let own = key_list(9, &[&second]);
    assert_eq!(
        case(
            "a list signed with a key that is on the list and not built in",
            read(&own, &second.sign(own.as_bytes()), None)
        ),
        Refusal::UntrustedKey {
            of: "the key list".to_string(),
            by_list: false
        }
    );
    // A key added to a signed list.
    let added = format!("{list}key {}\n", Signer::new().public());
    assert_eq!(
        case(
            "a list changed after it was signed",
            read(&added, &signature, None)
        ),
        Refusal::SignatureMismatch {
            of: "the key list".to_string()
        }
    );
    // Lists that are signed and do not read as lists.
    for (title, text) in [
        ("an empty list", String::new()),
        (
            "a list in another form",
            "zeff keys 2\nserial 1\n".to_string(),
        ),
        (
            "a list with no serial",
            format!("zeff keys 1\nkey {}\n", root.public()),
        ),
        ("a list with no key", "zeff keys 1\nserial 4\n".to_string()),
        (
            "a serial that is not a number",
            format!("zeff keys 1\nserial four\nkey {}\n", root.public()),
        ),
        (
            "a serial given twice",
            format!("zeff keys 1\nserial 4\nserial 5\nkey {}\n", root.public()),
        ),
        (
            "a key that is not a key",
            "zeff keys 1\nserial 4\nkey RWQnotakey\n".to_string(),
        ),
        (
            "a line of another kind",
            format!("zeff keys 1\nserial 4\nkey {}\nrevoke all\n", root.public()),
        ),
    ] {
        let refusal = case(title, read(&text, &root.sign(text.as_bytes()), None));
        assert!(
            matches!(refusal, Refusal::KeyListMalformed { .. }),
            "{title}: {refusal:?}"
        );
    }
    // A built-in key that is not a key at all.
    let unreadable = Trust::of_key("not a key").map(|trust| trust.serial());
    case("a built-in key that cannot be read", unreadable);
    insta::assert_snapshot!(out);
}

// ----- the addresses -----

/// The ruling of 20 November 2026: the launcher compiles in one host, `https://zeff-lang.dev/`,
/// and two paths under it; no address of a code-hosting site and no organisation's name stands
/// in its sources or in its binary, so that the repository can move and no installed launcher
/// breaks.
#[test]
fn every_address_the_launcher_holds_is_under_the_one_host() {
    assert_eq!(HOST, "https://zeff-lang.dev/");
    for path in [KEY_LIST, ENGINE_INDEX] {
        assert!(
            !path.starts_with('/') && !path.contains("://") && !path.contains(".."),
            "{path}: a path under the host"
        );
    }
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0;
    for entry in std::fs::read_dir(&sources).unwrap().flatten() {
        let text = std::fs::read_to_string(entry.path()).unwrap();
        read += 1;
        let file = entry.file_name().to_string_lossy().into_owned();
        for (at, _) in text.match_indices("://") {
            let from = text[..at]
                .rfind(|c: char| !c.is_ascii_alphabetic())
                .map_or(0, |i| i + 1);
            assert!(
                text[from..].starts_with(HOST),
                "{file}: an address that is not under {HOST}: {}",
                text[from..].lines().next().unwrap_or_default()
            );
        }
        for forbidden in forbidden_names() {
            assert!(!text.contains(&forbidden), "{file}: names `{forbidden}`");
        }
        // The domain's own name is the one use of its first label.
        assert_eq!(
            text.matches("zeff-lang").count(),
            text.matches("zeff-lang.dev").count(),
            "{file}: `zeff-lang` other than as the domain"
        );
    }
    assert!(read >= 5, "the launcher's sources were read: {read} files");
    let binary = std::fs::read(env!("CARGO_BIN_EXE_zeff-launcher")).unwrap();
    for forbidden in forbidden_names() {
        assert!(
            !binary
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes()),
            "the launcher's binary names `{forbidden}`"
        );
    }
}

/// A code-hosting site and the organisation the repository is under, spelt in pieces so that
/// this file does not itself fail a search for them.
fn forbidden_names() -> [String; 3] {
    [
        ["git", "hub.com"].concat(),
        ["git", "lab.com"].concat(),
        ["zoo", "stat"].concat(),
    ]
}
