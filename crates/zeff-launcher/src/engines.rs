//! `zeff engine`, the one command the launcher answers itself (Engine and Project
//! Specification, section 2.1, "Engines"; release 0.9.0, step 19a): `list` the engines of the
//! store, `install` one from an archive on this machine, `remove` one, and say or set the
//! `default`, the engine that runs outside a project.
//!
//! Like the rest of the launcher, the command is a function of its arguments, the environment
//! and the files of the store, and writes to the two streams it is given, so that it is tested
//! on a temporary store.

use crate::archive::{Placed, Target};
use crate::install::{in_words, install_from, said, untrusted};
use crate::net::{Fetch, FetchError, Http, Source};
use crate::remote::{Fetched, NotFetched, Wanted, fetch_engine};
use crate::trust::Trust;
use crate::{EXIT_ENGINE, Platform, Store, Version, default_setting, installed, store};
use std::io::Write;
use std::path::{Path, PathBuf};
use zeff_diagnostics::{Diagnostic, Sources, json_string};

/// The exit code of a command that was understood and refused, and of one that was not
/// understood: the engine's own for a usage error.
pub const EXIT_REFUSED: u8 = 1;

/// The user's settings, in the store.
const CONFIG: &str = "config.toml";

/// Options of the command line that take a value in the word after them.
const WITH_VALUE: [&str; 8] = [
    "-p",
    "--project",
    "--format",
    "--color",
    "--width",
    "--memory",
    "--threads",
    "--from",
];

/// What was asked of `zeff engine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// The words after `engine`: the command and what it acts on.
    pub words: Vec<String>,
    /// `--format json`.
    pub json: bool,
    /// `--from <file>`.
    pub from: Option<PathBuf>,
    /// `--frozen` or `--offline`, as it was written: nothing is fetched.
    pub nothing_fetched: Option<&'static str>,
}

/// Read the command line: `None` when the command is not `engine`, which is then the engine's
/// to answer; otherwise what was asked, or what is wrong with how it was asked.
pub fn read(args: &[String]) -> Option<Result<Asked, String>> {
    let mut words = Vec::new();
    let mut json = false;
    let mut from = None;
    let mut nothing_fetched = None;
    let mut wrong = None;
    let mut at = 0;
    while at < args.len() {
        let arg = args[at].as_str();
        at += 1;
        if arg == "--" {
            words.extend(args[at..].iter().cloned());
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            words.push(arg.to_string());
            continue;
        }
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (arg, None),
        };
        if !WITH_VALUE.contains(&name) {
            if let Some(flag) = no_fetching(name) {
                nothing_fetched.get_or_insert(flag);
            }
            continue;
        }
        let value = inline.or_else(|| {
            at += 1;
            args.get(at - 1).cloned()
        });
        match (name, value) {
            ("--format", Some(value)) => match value.as_str() {
                "json" => json = true,
                "text" => json = false,
                other => {
                    wrong.get_or_insert(format!(
                        "`--format {other}` is not a format: write `--format text` or `--format json`"
                    ));
                }
            },
            ("--from", Some(value)) => from = Some(PathBuf::from(value)),
            (_, Some(_)) => {}
            (name, None) => {
                wrong.get_or_insert(format!("`{name}` needs a value after it"));
            }
        }
    }
    if words.first().map(String::as_str) != Some("engine") {
        return None;
    }
    Some(match wrong {
        Some(wrong) => Err(wrong),
        None => Ok(Asked {
            words: words.split_off(1),
            json,
            from,
            nothing_fetched,
        }),
    })
}

/// `--frozen` and `--offline`, each as it is named in a message.
fn no_fetching(flag: &str) -> Option<&'static str> {
    match flag {
        "--frozen" => Some("`--frozen`"),
        "--offline" => Some("`--offline`"),
        _ => None,
    }
}

/// The same from the environment: `ZEFF_FROZEN=1` and `ZEFF_OFFLINE=1` act as the flags.
fn no_fetching_set(env: &dyn Fn(&str) -> Option<String>) -> Option<&'static str> {
    let set = |name: &str| env(name).is_some_and(|value| value == "1");
    if set("ZEFF_FROZEN") {
        Some("`ZEFF_FROZEN`")
    } else if set("ZEFF_OFFLINE") {
        Some("`ZEFF_OFFLINE`")
    } else {
        None
    }
}

/// What the command finds around it.
pub struct Machine<'a> {
    pub env: &'a dyn Fn(&str) -> Option<String>,
    /// The folder a path on the command line is read from.
    pub cwd: &'a Path,
    pub platform: Platform,
    /// The machine engines must be for; `None` on one no engine is published for.
    pub host: Option<Target>,
    /// The key archives are proved against: the launcher's built-in key, a test's own.
    pub key: Option<&'a str>,
    /// The network; `None` for the real one, made when it is first needed.
    pub fetch: Option<&'a dyn Fetch>,
}

/// Say that the command was not understood, as the engine says it; the exit code.
pub fn usage(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(
        err,
        "error: {message}\n\nrun `zeff help` to see the commands"
    );
    EXIT_REFUSED
}

fn say(err: &mut dyn Write, diagnostic: &Diagnostic) {
    let _ = write!(err, "{}", diagnostic.render(&Sources::new()));
}

/// A size as it is read: `21.4 MB`.
pub fn size_text(bytes: u64) -> String {
    let b = bytes as f64;
    if bytes < 1_000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{:.1} kB", b / 1e3)
    } else if bytes < 1_000_000_000 {
        format!("{:.1} MB", b / 1e6)
    } else {
        format!("{:.1} GB", b / 1e9)
    }
}

/// The bytes of every file under `folder`.
fn size_of(folder: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => size_of(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |data| data.len()),
            _ => 0,
        })
        .sum()
}

/// The newest release among `versions`, which are in rising order; a version before its
/// release only when there is nothing else.
pub fn newest_release(versions: &[Version]) -> Option<&Version> {
    versions
        .iter()
        .rev()
        .find(|version| version.pre.is_none())
        .or(versions.last())
}

/// The engine that runs outside a project, and why it is that one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Default {
    /// `[engine] default` of `config.toml` names it; it may not be installed.
    Set { version: String, installed: bool },
    /// Nothing is set: the newest installed.
    Newest(String),
    /// No engine is installed and nothing is set.
    None,
}

pub fn default_of(store: &Store, platform: Platform) -> Default {
    let versions = installed(store, platform);
    let setting = std::fs::read_to_string(store.user.join(CONFIG))
        .ok()
        .and_then(|config| default_setting(&config));
    match setting {
        Some(version) => Default::Set {
            installed: versions.iter().any(|found| found.text == version),
            version,
        },
        None => match newest_release(&versions) {
            Some(newest) => Default::Newest(newest.text.clone()),
            None => Default::None,
        },
    }
}

/// `config` with `[engine] default` set to `version`, or taken out for `None`; every other
/// line as it was. `None` when the file sets the default in a form this does not rewrite, or
/// is not TOML: the result is read back, and anything but the setting asked for is refused.
pub fn with_default(config: &str, version: Option<&str>) -> Option<String> {
    let setting = version.map(|version| format!("default = \"{version}\""));
    let mut lines: Vec<String> = config.lines().map(str::to_string).collect();
    let mut in_engine = false;
    let mut header = None;
    let mut found = None;
    for (at, line) in lines.iter().enumerate() {
        let text = line.trim();
        if text.starts_with('[') {
            let name = text.split('#').next().unwrap_or_default().trim();
            in_engine = name == "[engine]";
            if in_engine && header.is_none() {
                header = Some(at);
            }
        } else if in_engine
            && found.is_none()
            && text
                .strip_prefix("default")
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        {
            found = Some(at);
        }
    }
    match (found, header, setting) {
        (Some(at), _, Some(setting)) => lines[at] = setting,
        (Some(at), _, None) => {
            lines.remove(at);
        }
        (None, Some(at), Some(setting)) => lines.insert(at + 1, setting),
        (None, None, Some(setting)) => {
            if lines.last().is_some_and(|last| !last.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push("[engine]".to_string());
            lines.push(setting);
        }
        (None, _, None) => {}
    }
    let mut written = lines.join("\n");
    if !written.is_empty() {
        written.push('\n');
    }
    let valid = toml::de::DeTable::parse(&written).is_ok();
    (valid && default_setting(&written).as_deref() == version).then_some(written)
}

/// Write `config.toml` whole or not at all.
fn write_config(home: &Path, text: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(home)?;
    let being_written = home.join(format!(".{CONFIG}-{}", std::process::id()));
    std::fs::write(&being_written, text)?;
    std::fs::rename(&being_written, home.join(CONFIG)).inspect_err(|_| {
        let _ = std::fs::remove_file(&being_written);
    })
}

fn installed_help(versions: &[Version]) -> String {
    match versions {
        [] => "no engine is installed".to_string(),
        [one] => format!("the engine installed is {}", one.text),
        many => format!(
            "the engines installed are {}",
            many.iter()
                .map(|version| version.text.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn not_installed(version: &str, versions: &[Version]) -> Diagnostic {
    Diagnostic::error("Z0710", format!("Zeff {version} is not installed")).help(format!(
        "{}; `zeff engine install {version}` installs it",
        installed_help(versions)
    ))
}

/// One version from the words after a command, or what to say.
fn one_version<'a>(words: &'a [String], command: &str, example: &str) -> Result<&'a str, String> {
    match words {
        [version] if Version::parse(version).is_some() => Ok(version),
        [word] => Err(format!(
            "`{word}` is not the version of an engine: three numbers, as {example}"
        )),
        _ => Err(format!(
            "`zeff engine {command}` takes one version, as `zeff engine {command} {example}`"
        )),
    }
}

/// Answer `zeff engine ...`; the exit code.
pub fn run(asked: &Asked, machine: &Machine<'_>, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Some(store) = store(machine.env, machine.platform) else {
        say(
            err,
            &Diagnostic::error(
                "Z0710",
                "no Zeff engine can be found: this user has no home folder",
            )
            .help("set `ZEFF_HOME` to the folder that holds `engines/`"),
        );
        return EXIT_ENGINE;
    };
    let words: Vec<&str> = asked.words.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["list"] => list(asked, &store, machine.platform, out),
        ["list", ..] => usage(err, "`zeff engine list` takes nothing after it"),
        ["install", rest @ ..] => install(asked, rest, &store, machine, out, err),
        ["remove", ..] => remove(&asked.words[1..], &store, machine.platform, out, err),
        ["default"] => say_default(&store, machine.platform, out, err),
        ["default", ..] => set_default(&asked.words[1..], &store, machine.platform, out, err),
        [] => usage(
            err,
            "`zeff engine` takes a command: `list`, `install`, `remove` or `default`",
        ),
        [word, ..] => usage(
            err,
            &format!(
                "`zeff engine {word}` is not a command: `list`, `install`, `remove` or `default`"
            ),
        ),
    }
}

fn list(asked: &Asked, store: &Store, platform: Platform, out: &mut dyn Write) -> u8 {
    let default = default_of(store, platform);
    let runs = match &default {
        Default::Set {
            version,
            installed: true,
        } => Some(version.as_str()),
        Default::Newest(version) => Some(version.as_str()),
        _ => None,
    };
    // Each engine once, from the user's store before the system's, as the launcher finds it.
    let rows: Vec<(Version, PathBuf, bool)> = installed(store, platform)
        .into_iter()
        .map(|version| {
            let own = store.user.join("engines").join(&version.text);
            let in_user = own.join(platform.engine_file()).is_file();
            let folder = match (&store.system, in_user) {
                (Some(system), false) => system.join("engines").join(&version.text),
                _ => own,
            };
            (version, folder, in_user)
        })
        .collect();
    if asked.json {
        let engines: Vec<String> = rows
            .iter()
            .map(|(version, folder, in_user)| {
                format!(
                    "{{\"version\": {}, \"path\": {}, \"bytes\": {}, \"store\": \"{}\", \"default\": {}}}",
                    json_string(&version.text),
                    json_string(&folder.display().to_string()),
                    size_of(folder),
                    if *in_user { "user" } else { "system" },
                    runs == Some(version.text.as_str())
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"home\": {}, \"default\": {}, \"default_set\": {}, \"engines\": [{}]}}",
            json_string(&store.user.display().to_string()),
            runs.map_or("null".to_string(), json_string),
            matches!(default, Default::Set { .. }),
            engines.join(", ")
        );
        return 0;
    }
    if rows.is_empty() {
        let _ = writeln!(
            out,
            "no engine is installed in `{}`",
            store.user.join("engines").display()
        );
    } else {
        let _ = writeln!(
            out,
            "Zeff engines in `{}`",
            store.user.join("engines").display()
        );
        let width = rows
            .iter()
            .map(|(version, ..)| version.text.chars().count())
            .max()
            .unwrap_or(0);
        for (version, folder, in_user) in &rows {
            let mut remarks = Vec::new();
            if runs == Some(version.text.as_str()) {
                remarks.push(match default {
                    Default::Set { .. } => "the default, set in `config.toml`".to_string(),
                    _ => "the default: the newest, none is set".to_string(),
                });
            }
            if !in_user && let Some(system) = &store.system {
                remarks.push(format!("in the system-wide store `{}`", system.display()));
            }
            let line = format!(
                "  {:<width$}  {:>9}   {}",
                version.text,
                size_text(size_of(folder)),
                remarks.join("; ")
            );
            let _ = writeln!(out, "{}", line.trim_end());
        }
    }
    if let Default::Set {
        version,
        installed: false,
    } = &default
    {
        let _ = writeln!(
            out,
            "the default set in `config.toml`, Zeff {version}, is not installed"
        );
    }
    0
}

fn install(
    asked: &Asked,
    rest: &[&str],
    store: &Store,
    machine: &Machine<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let Some(from) = &asked.from else {
        let wanted = match rest {
            [] => Some(Wanted::Newest),
            [word] => Wanted::parse(Some(word)),
            _ => None,
        };
        let Some(wanted) = wanted else {
            return usage(
                err,
                "`zeff engine install` takes one version, as `zeff engine install 0.9.1`, a series, as `zeff engine install 0.9` for the newest 0.9.x, or nothing for the newest of all",
            );
        };
        let nothing_fetched = asked.nothing_fetched.or(no_fetching_set(machine.env));
        return match fetch_into_store(&wanted, store, machine, nothing_fetched, err) {
            Ok(fetched) => {
                say_placed(out, &fetched.placed);
                0
            }
            Err(code) => code,
        };
    };
    if !rest.is_empty() {
        return usage(
            err,
            "`zeff engine install --from <file>` takes no version: the archive's name carries it",
        );
    }
    let Some(host) = machine.host else {
        say(
            err,
            &Diagnostic::error("Z0717", "no engine is published for this kind of machine").help(
                format!(
                    "engines are built for {}",
                    Target::ALL
                        .iter()
                        .map(|target| target.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        );
        return EXIT_REFUSED;
    };
    let trust = match machine.key {
        Some(key) => Trust::of_key(key),
        None => Err(crate::trust::Refusal::NoBuiltInKey),
    };
    let trust = match trust {
        Ok(trust) => trust,
        Err(refusal) => {
            say(err, &untrusted(&refusal));
            return EXIT_REFUSED;
        }
    };
    let path = if from.is_absolute() {
        from.clone()
    } else {
        machine.cwd.join(from)
    };
    match install_from(&store.user, &path, &trust, host) {
        Ok(placed) => {
            say_placed(out, &placed);
            0
        }
        Err(refused) => {
            say(err, &said(&refused));
            EXIT_REFUSED
        }
    }
}

fn remove(
    words: &[String],
    store: &Store,
    platform: Platform,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let version = match one_version(words, "remove", "0.9.0") {
        Ok(version) => version,
        Err(message) => return usage(err, &message),
    };
    let folder = store.user.join("engines").join(version);
    if !folder.join(platform.engine_file()).is_file() {
        let elsewhere = store
            .system
            .as_ref()
            .filter(|system| crate::engine_in(system, version, platform).is_file());
        return match elsewhere {
            Some(system) => usage(
                err,
                &format!(
                    "Zeff {version} is in the system-wide store `{}`, which `zeff engine remove` does not change",
                    system.display()
                ),
            ),
            None => {
                say(err, &not_installed(version, &installed(store, platform)));
                EXIT_ENGINE
            }
        };
    }
    let was_default = matches!(
        default_of(store, platform),
        Default::Set { version: set, .. } if set == version
    );
    if let Err(e) = std::fs::remove_dir_all(&folder) {
        say(
            err,
            &Diagnostic::error(
                "Z0717",
                format!(
                    "Zeff {version} could not be removed from `{}`: {}",
                    folder.display(),
                    in_words(&e)
                ),
            )
            .help("an engine that is running cannot be removed on every system; close what runs on it and remove it again"),
        );
        return EXIT_REFUSED;
    }
    let _ = writeln!(out, "removed Zeff {version} from `{}`", folder.display());
    say(
        err,
        &Diagnostic::plain_note(format!(
            "a project that pins Zeff {version} will ask to install it again"
        )),
    );
    if was_default {
        // The setting would name an engine that is gone, and every command outside a project
        // would stop on it.
        let cleared = std::fs::read_to_string(store.user.join(CONFIG))
            .ok()
            .and_then(|config| with_default(&config, None))
            .is_some_and(|config| write_config(&store.user, &config).is_ok());
        let now = match default_of(store, platform) {
            Default::Newest(newest) if cleared => {
                format!("the default is now the newest installed, Zeff {newest}")
            }
            _ if cleared => "no engine is installed now".to_string(),
            _ => "`config.toml` still names it: `zeff engine default <version>` chooses another"
                .to_string(),
        };
        say(
            err,
            &Diagnostic::plain_note(format!("Zeff {version} was the default engine; {now}")),
        );
    }
    0
}

fn say_default(store: &Store, platform: Platform, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match default_of(store, platform) {
        Default::Set {
            version,
            installed: true,
        } => {
            let _ = writeln!(
                out,
                "the default engine is Zeff {version}, set in `config.toml`"
            );
            0
        }
        Default::Newest(version) => {
            let _ = writeln!(
                out,
                "the default engine is Zeff {version}, the newest installed; none is set"
            );
            0
        }
        Default::Set {
            version,
            installed: false,
        } => {
            say(
                err,
                &Diagnostic::error(
                    "Z0710",
                    format!("the default engine, Zeff {version}, is not installed"),
                )
                .help(format!(
                    "`zeff engine install {version}` installs it, and `zeff engine default <version>` chooses another; {}",
                    installed_help(&installed(store, platform))
                )),
            );
            EXIT_ENGINE
        }
        Default::None => {
            say(
                err,
                &Diagnostic::error("Z0710", "no Zeff engine is installed").help(format!(
                    "`zeff engine install` installs one into `{}`",
                    store.user.join("engines").display()
                )),
            );
            EXIT_ENGINE
        }
    }
}

fn set_default(
    words: &[String],
    store: &Store,
    platform: Platform,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let version = match one_version(words, "default", "0.9.1") {
        Ok(version) => version,
        Err(message) => return usage(err, &message),
    };
    let versions = installed(store, platform);
    if !versions.iter().any(|found| found.text == version) {
        say(err, &not_installed(version, &versions));
        return EXIT_ENGINE;
    }
    let path = store.user.join(CONFIG);
    let config = match std::fs::read_to_string(&path) {
        Ok(config) => config,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return usage(
                err,
                &format!("`{}` cannot be read: {}", path.display(), in_words(&e)),
            );
        }
    };
    let Some(written) = with_default(&config, Some(version)) else {
        return usage(
            err,
            &format!(
                "`{}` is not valid TOML, or sets the default engine in a form this command does not rewrite; write `default = \"{version}\"` under `[engine]` there by hand",
                path.display()
            ),
        );
    };
    if let Err(e) = write_config(&store.user, &written) {
        return usage(
            err,
            &format!("`{}` cannot be written: {}", path.display(), in_words(&e)),
        );
    }
    let _ = writeln!(
        out,
        "the default engine is now Zeff {version}: it runs outside a project, and `zeff new` pins it"
    );
    0
}

/// Say where an engine was installed, or that it was there already.
fn say_placed(out: &mut dyn Write, placed: &Placed) {
    let engine = match placed {
        Placed::New(engine) | Placed::Already(engine) => engine,
    };
    let folder = engine.parent().unwrap_or(engine);
    let version = folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let _ = match placed {
        Placed::New(_) => writeln!(out, "installed Zeff {version} in `{}`", folder.display()),
        Placed::Already(_) => writeln!(
            out,
            "Zeff {version} is already installed in `{}`",
            folder.display()
        ),
    };
}

const FROM_A_FILE: &str =
    "`zeff engine install --from <file>` installs an archive that is already on this machine";

/// What is said when an engine was not fetched, and the exit code that goes with it.
pub fn not_fetched(refused: &NotFetched, source: &Source) -> (Diagnostic, u8) {
    match refused {
        NotFetched::Network { what, url, error } => {
            let why = match error {
                FetchError::NotFound => "the server has no such file".to_string(),
                FetchError::Status(status) => format!("the server answered {status}"),
                FetchError::TooLarge => "it is larger than anything the launcher fetches".to_string(),
                FetchError::Failed(reason) => reason.clone(),
            };
            let from = if source.mirror {
                format!(
                    "the launcher fetches from the mirror `{}`, set by `ZEFF_MIRROR` or by `[network] mirror` of `config.toml`",
                    source.base
                )
            } else {
                format!(
                    "the launcher fetches from `{}`; `ZEFF_MIRROR` names a mirror of it",
                    source.base
                )
            };
            (
                Diagnostic::error(
                    "Z0708",
                    format!("{what} could not be fetched from `{url}`: {why}"),
                )
                .help(format!("{from}; {FROM_A_FILE}")),
                EXIT_REFUSED,
            )
        }
        NotFetched::Untrusted(refusal) => (untrusted(refusal), EXIT_REFUSED),
        NotFetched::Index { reason } => (
            Diagnostic::error(
                "Z0717",
                format!("the engine index cannot be read: {reason}; the store is as it was"),
            )
            .help(format!(
                "the index is the publisher's and is faulty as published; `zeff report-bug` writes a report, and {FROM_A_FILE}"
            )),
            EXIT_REFUSED,
        ),
        NotFetched::NoSuchVersion {
            wanted,
            host,
            published,
        } => {
            let message = match wanted {
                Wanted::Newest => format!("no release of Zeff is published for {}", host.name()),
                Wanted::Series(..) => format!(
                    "no release of Zeff {} is published for {}",
                    wanted.words(),
                    host.name()
                ),
                Wanted::Exact(_) => format!(
                    "Zeff {} is not published for {}",
                    wanted.words(),
                    host.name()
                ),
            };
            let help = if published.is_empty() {
                "nothing is published for this kind of machine yet".to_string()
            } else {
                format!(
                    "published for it: {}; a version before its release is installed only when it is named in full",
                    published.join(", ")
                )
            };
            (Diagnostic::error("Z0710", message).help(help), EXIT_ENGINE)
        }
        NotFetched::Refused(refused) => (said(refused), EXIT_REFUSED),
        NotFetched::Store { path, reason } => (
            Diagnostic::error(
                "Z0717",
                format!(
                    "the engine store cannot be written at `{}`: {reason}; the store is as it was",
                    path.display()
                ),
            )
            .help("the store is the folder `ZEFF_HOME` names, by default `.zeff` in the home folder; it must be writable and have room for the engine"),
            EXIT_REFUSED,
        ),
    }
}

/// Fetch the engine asked for into the user's store, saying on `err` what stops it; the exit
/// code when it is stopped. `nothing_fetched` is the flag or the setting that forbids it.
pub fn fetch_into_store(
    wanted: &Wanted,
    store: &Store,
    machine: &Machine<'_>,
    nothing_fetched: Option<&'static str>,
    err: &mut dyn Write,
) -> Result<Fetched, u8> {
    if let Some(flag) = nothing_fetched {
        let what = match wanted {
            Wanted::Exact(version) => format!("Zeff {} is not installed, and", version.text),
            _ => "an engine is asked for, and".to_string(),
        };
        say(
            err,
            &Diagnostic::error("Z0710", format!("{what} nothing is fetched under {flag}"))
                .help(FROM_A_FILE),
        );
        return Err(EXIT_ENGINE);
    }
    let Some(host) = machine.host else {
        say(
            err,
            &Diagnostic::error("Z0717", "no engine is published for this kind of machine").help(
                format!(
                    "engines are built for {}",
                    Target::ALL
                        .iter()
                        .map(|target| target.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        );
        return Err(EXIT_REFUSED);
    };
    let Some(key) = machine.key else {
        say(err, &untrusted(&crate::trust::Refusal::NoBuiltInKey));
        return Err(EXIT_REFUSED);
    };
    let config = std::fs::read_to_string(store.user.join(CONFIG)).ok();
    let source = match Source::of(machine.env, config.as_deref()) {
        Ok(source) => source,
        Err(crate::net::NotAnAddress(written)) => {
            return Err(usage(
                err,
                &format!(
                    "the mirror is set to `{written}`, which is not an address: it begins with `https://` or `http://`; it is set by `ZEFF_MIRROR` or by `[network] mirror` of `config.toml`"
                ),
            ));
        }
    };
    let own;
    let fetch: &dyn Fetch = match machine.fetch {
        Some(fetch) => fetch,
        None => {
            let proxy = config
                .as_deref()
                .and_then(|config| crate::text_at(config, "network", "proxy"));
            own = Http::new(proxy.as_deref());
            &own
        }
    };
    fetch_engine(&store.user, wanted, key, &source, fetch, host).map_err(|refused| {
        let (diagnostic, code) = not_fetched(&refused, &source);
        say(err, &diagnostic);
        code
    })
}

/// The flags that are the launcher's alone, which an engine is not handed.
const OWN_FLAGS: [&str; 3] = ["--install", "-y", "--yes"];

/// The command line as the engine is to see it: without the launcher's own flags, which stand
/// before any `--`.
pub fn for_the_engine<T: AsRef<std::ffi::OsStr> + Clone>(args: &[T]) -> Vec<T> {
    let end = args
        .iter()
        .position(|arg| arg.as_ref() == "--")
        .unwrap_or(args.len());
    args.iter()
        .enumerate()
        .filter(|(at, arg)| *at >= end || !OWN_FLAGS.iter().any(|flag| arg.as_ref() == *flag))
        .map(|(_, arg)| arg.clone())
        .collect()
}

/// A project pins an engine that is not installed (section 2.1, "Running"): it is installed
/// after a question on a terminal, without one under `--install` or `-y`, and not at all when
/// nothing may be fetched. `ask` puts the question and gives the answer typed; it is `None`
/// off a terminal. `Ok(Some(engine))` when the engine is now installed, `Ok(None)` when it was
/// not asked for or was declined, and the exit code when the install was tried and stopped.
pub fn install_pinned(
    version: &str,
    args: &[String],
    machine: &Machine<'_>,
    ask: Option<&mut dyn FnMut(&str) -> String>,
    err: &mut dyn Write,
) -> Result<Option<PathBuf>, u8> {
    let Some(store) = store(machine.env, machine.platform) else {
        return Ok(None);
    };
    let Some(pinned) = Version::parse(version) else {
        return Ok(None);
    };
    let end = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    let flags = &args[..end];
    let nothing_fetched = flags
        .iter()
        .find_map(|flag| no_fetching(flag))
        .or(no_fetching_set(machine.env));
    let told = flags.iter().any(|flag| OWN_FLAGS.contains(&flag.as_str()));
    if nothing_fetched.is_none() && !told {
        let Some(ask) = ask else {
            return Ok(None);
        };
        let answer = ask(&format!(
            "this project needs Zeff {version}, which is not installed. Install it now? [Y/n] "
        ));
        if !matches!(answer.trim().to_lowercase().as_str(), "" | "y" | "yes") {
            return Ok(None);
        }
    }
    let fetched = fetch_into_store(
        &Wanted::Exact(pinned),
        &store,
        machine,
        nothing_fetched,
        err,
    )?;
    let engine = match fetched.placed {
        Placed::New(engine) => {
            // On the error stream: what the command prints is the command's own.
            let _ = writeln!(err, "installed Zeff {version}");
            engine
        }
        Placed::Already(engine) => engine,
    };
    Ok(Some(engine))
}
