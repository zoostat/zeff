//! Engines from the signed index (Engine and Project Specification, section 2.1; release
//! 0.9.0, step 19b).
//!
//! ```text
//! format = 1
//!
//! [[engine]]
//! version = "0.9.1"
//! at = "engines/0.9.1/"
//! targets = ["linux-x86-64", "linux-arm64", "macos-arm64"]
//! ```
//!
//! The index, `engines/index.toml` under the host, names every engine published, the folder
//! its files are in and the machines it is built for; `index.toml.minisig` beside it is its
//! signature, by the keys archives are signed with. A release's folder holds its archives under
//! their own names, `SHA256SUMS` and its signature. `at` is an address under the host, or a
//! whole address when the files live elsewhere, so that where they live can change with the
//! index alone. Under a mirror the files are always at `engines/<version>/` of the mirror: a
//! mirror replaces the host and nothing else.
//!
//! Nothing here is trusted for being fetched: the key list must be signed by the built-in
//! key, the index and the checksums by a key that is trusted, and the archive must have the
//! checksum listed, exactly as for an archive brought by hand.

use crate::archive::{ArchiveName, Placed, Target};
use crate::install::{Refused, install_from};
use crate::net::{ARCHIVE, Fetch, FetchError, SMALL, Source};
use crate::trust::{ENGINE_INDEX, KEY_LIST, Refusal, SIGNATURE_SUFFIX, SUMS, Trust};
use crate::{Platform, Version, engine_in};
use std::path::{Path, PathBuf};
use toml::de::{DeTable, DeValue};

/// One engine of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    /// The folder its files are in, as the index gives it.
    pub at: String,
    /// The machines it is built for, by name.
    pub targets: Vec<String>,
}

/// Read the index. Strict about what it needs and silent about what it does not know, so that
/// an index may gain keys a launcher of today passes over.
pub fn read_index(text: &str) -> Result<Vec<Release>, String> {
    let document = DeTable::parse(text).map_err(|_| "it is not valid TOML".to_string())?;
    let table = document.get_ref();
    match table.get("format").map(|format| format.get_ref()) {
        Some(DeValue::Integer(format)) if format.as_str() == "1" => {}
        Some(_) => return Err("its `format` is not 1, the one this launcher reads".to_string()),
        None => return Err("it has no `format`".to_string()),
    }
    let Some(engines) = table.get("engine") else {
        return Ok(Vec::new());
    };
    let DeValue::Array(engines) = engines.get_ref() else {
        return Err("its `engine` is not a list of tables".to_string());
    };
    let mut releases = Vec::new();
    for (at, engine) in engines.iter().enumerate() {
        let wrong = |what: &str| format!("its engine {} has no {what}", at + 1);
        let DeValue::Table(engine) = engine.get_ref() else {
            return Err("its `engine` is not a list of tables".to_string());
        };
        let text_of = |key: &str| match engine.get(key).map(|value| value.get_ref()) {
            Some(DeValue::String(text)) => Some(text.to_string()),
            _ => None,
        };
        let version = text_of("version")
            .and_then(|version| Version::parse(&version))
            .ok_or_else(|| wrong("`version` of three numbers"))?;
        let folder = text_of("at").ok_or_else(|| wrong("`at`"))?;
        let targets = match engine.get("targets").map(|value| value.get_ref()) {
            Some(DeValue::Array(targets)) => targets
                .iter()
                .map(|target| match target.get_ref() {
                    DeValue::String(name) => Some(name.to_string()),
                    _ => None,
                })
                .collect::<Option<Vec<String>>>(),
            _ => None,
        }
        .ok_or_else(|| wrong("`targets`, a list of names"))?;
        releases.push(Release {
            version,
            at: folder,
            targets,
        });
    }
    releases.sort_by(|a, b| a.version.cmp(&b.version));
    Ok(releases)
}

/// Which engine is asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wanted {
    /// `zeff engine install`: the newest release.
    Newest,
    /// `zeff engine install 0.9`: the newest release 0.9.x.
    Series(u64, u64),
    /// `zeff engine install 0.9.1`, or a project's pin: that version and no other.
    Exact(Version),
}

impl Wanted {
    /// Read the word after `install`; `None` when it is neither a version nor a series.
    pub fn parse(word: Option<&str>) -> Option<Wanted> {
        let Some(word) = word else {
            return Some(Wanted::Newest);
        };
        if let Some(version) = Version::parse(word) {
            return Some(Wanted::Exact(version));
        }
        let (major, minor) = word.split_once('.')?;
        let number = |text: &str| {
            (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
                .then(|| text.parse::<u64>().ok())
                .flatten()
        };
        Some(Wanted::Series(number(major)?, number(minor)?))
    }

    /// As it is said in a message: `0.9.1`, `0.9.x`, `the newest`.
    pub fn words(&self) -> String {
        match self {
            Wanted::Newest => "a release".to_string(),
            Wanted::Series(major, minor) => format!("{major}.{minor}.x"),
            Wanted::Exact(version) => version.text.clone(),
        }
    }
}

/// The engine of `releases` that is asked for and built for `host`. A version before its
/// release is taken only when it is named exactly.
pub fn pick<'a>(releases: &'a [Release], wanted: &Wanted, host: Target) -> Option<&'a Release> {
    let fits = |release: &&Release| {
        release.targets.iter().any(|name| name == host.name())
            && match wanted {
                Wanted::Newest => release.version.pre.is_none(),
                Wanted::Series(major, minor) => {
                    release.version.pre.is_none()
                        && release.version.numbers[..2] == [*major, *minor]
                }
                Wanted::Exact(version) => release.version == *version,
            }
    };
    releases.iter().rev().find(fits)
}

/// Why an engine was not fetched and installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotFetched {
    /// A file could not be fetched.
    Network {
        what: String,
        url: String,
        error: FetchError,
    },
    /// The key list, the index or the checksums are not proved.
    Untrusted(Refusal),
    /// The index is the publisher's and does not read as an index.
    Index { reason: String },
    /// The index has no such engine for this machine.
    NoSuchVersion {
        wanted: Wanted,
        host: Target,
        published: Vec<String>,
    },
    /// The archive was fetched and not installed.
    Refused(Refused),
    /// The store could not hold the download.
    Store { path: PathBuf, reason: String },
}

/// What a run of [`fetch_engine`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub version: String,
    pub placed: Placed,
}

fn fetched(fetch: &dyn Fetch, what: &str, url: &str, limit: u64) -> Result<Vec<u8>, NotFetched> {
    fetch.get(url, limit).map_err(|error| NotFetched::Network {
        what: what.to_string(),
        url: url.to_string(),
        error,
    })
}

/// A fetched file as text; one that is not text is no signature and no list.
fn text(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes).into_owned()
}

/// The keys trusted now: those of the newest key list the built-in key signed, fetched or
/// held in the store; the built-in key alone when there is no list anywhere. A list that is
/// accepted and newer than the store's is kept, so that a withdrawn key stays withdrawn when
/// the network is not there.
pub fn trust_now(
    home: &Path,
    root: &str,
    source: &Source,
    fetch: &dyn Fetch,
) -> Result<Trust, NotFetched> {
    let keys = home.join("keys");
    let (list_path, signature_path) = (
        keys.join("list.txt"),
        keys.join(format!("list.txt{SIGNATURE_SUFFIX}")),
    );
    // The store's own copy is proved like any other; one that fails is as good as absent.
    let held = std::fs::read_to_string(&list_path)
        .ok()
        .zip(std::fs::read_to_string(&signature_path).ok())
        .and_then(|(list, signature)| Trust::of_key_list(root, &list, &signature, None).ok());
    let url = source.at(KEY_LIST);
    let list = match fetch.get(&url, SMALL) {
        Ok(list) => text(list),
        Err(FetchError::NotFound) => {
            return match held {
                Some(held) => Ok(held),
                None => Trust::of_key(root).map_err(NotFetched::Untrusted),
            };
        }
        Err(error) => {
            return Err(NotFetched::Network {
                what: "the key list".to_string(),
                url,
                error,
            });
        }
    };
    let signature_url = format!("{url}{SIGNATURE_SUFFIX}");
    let signature = text(fetched(
        fetch,
        "the signature of the key list",
        &signature_url,
        SMALL,
    )?);
    let held_serial = held.as_ref().and_then(Trust::serial);
    let trust =
        Trust::of_key_list(root, &list, &signature, held_serial).map_err(NotFetched::Untrusted)?;
    if trust.serial() > held_serial {
        // Kept for the next time; a store that cannot be written is no reason to stop.
        let _ = std::fs::create_dir_all(&keys)
            .and_then(|()| std::fs::write(&list_path, &list))
            .and_then(|()| std::fs::write(&signature_path, &signature));
    }
    Ok(trust)
}

/// The index, fetched and proved.
pub fn index_now(
    trust: &Trust,
    source: &Source,
    fetch: &dyn Fetch,
) -> Result<Vec<Release>, NotFetched> {
    let url = source.at(ENGINE_INDEX);
    let index = fetched(fetch, "the engine index", &url, SMALL)?;
    let signature = text(fetched(
        fetch,
        "the signature of the engine index",
        &format!("{url}{SIGNATURE_SUFFIX}"),
        SMALL,
    )?);
    trust
        .vouches(&index, &signature, "the engine index")
        .map_err(NotFetched::Untrusted)?;
    read_index(&text(index)).map_err(|reason| NotFetched::Index { reason })
}

/// Fetch the engine asked for and install it into the store at `home`: the key list, the
/// index, then the release's checksums, their signature and the archive for `host`, kept in
/// `cache/` until they are proved and unpacked. An engine already installed is fetched not at
/// all.
pub fn fetch_engine(
    home: &Path,
    wanted: &Wanted,
    root: &str,
    source: &Source,
    fetch: &dyn Fetch,
    host: Target,
) -> Result<Fetched, NotFetched> {
    let platform: Platform = host.platform();
    let trust = trust_now(home, root, source, fetch)?;
    let releases = index_now(&trust, source, fetch)?;
    let Some(release) = pick(&releases, wanted, host) else {
        return Err(NotFetched::NoSuchVersion {
            wanted: wanted.clone(),
            host,
            published: releases
                .iter()
                .filter(|release| release.targets.iter().any(|name| name == host.name()))
                .map(|release| release.version.text.clone())
                .collect(),
        });
    };
    let version = release.version.text.clone();
    let engine = engine_in(home, &version, platform);
    if engine.is_file() {
        return Ok(Fetched {
            version,
            placed: Placed::Already(engine),
        });
    }
    let folder = if source.mirror {
        source.at(&format!("engines/{version}/"))
    } else if release.at.contains("://") {
        format!("{}/", release.at.trim_end_matches('/'))
    } else {
        source.at(&format!("{}/", release.at.trim_end_matches('/')))
    };
    let name = ArchiveName::of(&release.version, host);
    let sums = fetched(
        fetch,
        &format!("`{SUMS}`"),
        &format!("{folder}{SUMS}"),
        SMALL,
    )?;
    let signature = fetched(
        fetch,
        &format!("`{SUMS}{SIGNATURE_SUFFIX}`"),
        &format!("{folder}{SUMS}{SIGNATURE_SUFFIX}"),
        SMALL,
    )?;
    let archive = fetched(
        fetch,
        &format!("`{}`", name.file),
        &format!("{folder}{}", name.file),
        ARCHIVE,
    )?;
    // Downloads awaiting verification: a folder of the store's cache, removed whatever
    // happens, for the store keeps an engine or nothing.
    let cache = home
        .join("cache")
        .join(format!("{version}-{}", std::process::id()));
    let store = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| NotFetched::Store {
            path,
            reason: crate::install::in_words(&e),
        }
    };
    let written = std::fs::create_dir_all(&cache)
        .and_then(|()| std::fs::write(cache.join(SUMS), &sums))
        .and_then(|()| std::fs::write(cache.join(format!("{SUMS}{SIGNATURE_SUFFIX}")), &signature))
        .and_then(|()| std::fs::write(cache.join(&name.file), &archive))
        .map_err(store(&cache));
    let placed = written.and_then(|()| {
        install_from(home, &cache.join(&name.file), &trust, host).map_err(NotFetched::Refused)
    });
    let _ = std::fs::remove_dir_all(&cache);
    Ok(Fetched {
        version,
        placed: placed?,
    })
}
