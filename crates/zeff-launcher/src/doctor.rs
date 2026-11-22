//! `zeff doctor` (Engine and Project Specification, section 2.1, "Diagnosis and settings";
//! release 0.9.0, step 21b): what this installation is, read from the machine and said line
//! by line: the launcher and the engines with their builds, the store and the user's folder,
//! PATH and completions, the settings in effect and where each comes from, whether the release
//! host answers, the free disk space, and the last `selftest` of each engine. A line that is a
//! problem carries what to do about it, and the report ends with their count: exit code 0
//! when there is none, 1 otherwise.

use crate::engines::{Asked, Default, Machine, build_of, default_of, size_text};
use crate::net::{Fetch, FetchError, Http, SMALL, Source};
use crate::trust::ENGINE_INDEX;
use crate::{Platform, installed, store, text_at};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::de::{DeTable, DeValue};
use zeff_diagnostics::json_string;

/// The commit and the toolchain the launcher was built from, and whether the release workflow
/// built it.
pub const COMMIT: &str = env!("ZEFF_BUILD_COMMIT");
pub const TOOLCHAIN: &str = env!("ZEFF_BUILD_TOOLCHAIN");

pub fn build_kind() -> &'static str {
    match option_env!("ZEFF_OFFICIAL_BUILD") {
        Some("1") => "official",
        _ => "source",
    }
}

/// Less free space than this where the store is, is a problem: an engine needs room.
const TIGHT: u64 = 1_000_000_000;

/// One line of the report: what it is about, what is found, and, when that is a problem, what
/// to do about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub about: &'static str,
    pub found: String,
    pub fix: Option<String>,
}

fn fine(about: &'static str, found: impl Into<String>) -> Line {
    Line {
        about,
        found: found.into(),
        fix: None,
    }
}

fn problem(about: &'static str, found: impl Into<String>, fix: impl Into<String>) -> Line {
    Line {
        about,
        found: found.into(),
        fix: Some(fix.into()),
    }
}

/// The last `zeff selftest` of an engine, as the engine recorded it in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proved {
    pub conformant: bool,
    pub cases: u64,
    pub differing: u64,
    pub on: String,
}

pub fn last_selftest(home: &Path, version: &str) -> Option<Proved> {
    let text =
        std::fs::read_to_string(home.join("selftest").join(format!("{version}.toml"))).ok()?;
    let document = DeTable::parse(&text).ok()?;
    let table = document.get_ref();
    let number = |key: &str| match table.get(key)?.get_ref() {
        DeValue::Integer(number) => number.as_str().parse::<u64>().ok(),
        _ => None,
    };
    Some(Proved {
        conformant: matches!(table.get("conformant")?.get_ref(), DeValue::Boolean(true)),
        cases: number("cases")?,
        differing: number("differing")?,
        on: match table.get("on")?.get_ref() {
            DeValue::String(on) => on.to_string(),
            _ => return None,
        },
    })
}

/// The completion file of `shell`, where one is usually kept for a user.
fn completion_file(shell: &str, user: &Path) -> Option<PathBuf> {
    let places: &[&str] = match shell {
        "bash" => &[".local/share/bash-completion/completions/zeff"],
        "zsh" => &[".zfunc/_zeff", ".zsh/completions/_zeff"],
        "fish" => &[".config/fish/completions/zeff.fish"],
        _ => &[],
    };
    places
        .iter()
        .map(|place| user.join(place))
        .find(|file| file.is_file())
}

/// The free space of the disk a folder is on, in bytes, when it can be told.
pub type FreeSpace<'a> = &'a dyn Fn(&Path) -> Option<u64>;

/// What the report needs beyond the command's own surroundings.
pub struct Around<'a> {
    /// Where the launcher's own file is.
    pub own_path: Option<PathBuf>,
    /// The free space of the disk a folder is on; `None` for the machine's own answer.
    pub free_space: Option<FreeSpace<'a>>,
}

fn free_space(around: &Around<'_>, folder: &Path) -> Option<u64> {
    match around.free_space {
        Some(asked) => asked(folder),
        None => {
            // The nearest folder that exists: a store not yet made is on its parent's disk.
            let mut at = Some(folder);
            while let Some(here) = at {
                if here.exists() {
                    return fs4::available_space(here).ok();
                }
                at = here.parent();
            }
            None
        }
    }
}

/// Every line of the report, in its order.
pub fn lines(asked: &Asked, machine: &Machine<'_>, around: &Around<'_>) -> Vec<Line> {
    let mut report = Vec::new();
    let env = machine.env;
    let set = |name: &str| env(name).filter(|value| !value.is_empty());
    // The launcher.
    report.push(fine(
        "launcher",
        format!(
            "Zeff {}, {} build, commit {COMMIT}{}",
            crate::VERSION,
            build_kind(),
            around
                .own_path
                .as_ref()
                .map_or(String::new(), |path| format!(", at `{}`", path.display()))
        ),
    ));
    let Some(store) = store(env, machine.platform) else {
        report.push(problem(
            "store",
            "this user has no home folder, so there is no engine store",
            "set `ZEFF_HOME` to the folder that is to hold `engines/`",
        ));
        return report;
    };
    let home = &store.user;
    let config_path = home.join("config.toml");
    let config = std::fs::read_to_string(&config_path).ok();
    // The engines.
    let versions = installed(&store, machine.platform);
    let default = default_of(&store, machine.platform);
    if versions.is_empty() {
        report.push(problem(
            "engines",
            "none is installed",
            "`zeff engine install` installs the newest, and `zeff engine install --from <file>` one from an archive on this machine",
        ));
    } else {
        let runs = match &default {
            Default::Set {
                version,
                installed: true,
            } => Some(version.as_str()),
            Default::Newest(version) => Some(version.as_str()),
            _ => None,
        };
        let listed: Vec<String> = versions
            .iter()
            .map(|version| {
                let engine = crate::find_engine(&store, &version.text, machine.platform);
                let mut said = Vec::new();
                if let Some(build) = engine.as_deref().and_then(build_of) {
                    said.push(format!("{build} build"));
                }
                if runs == Some(version.text.as_str()) {
                    said.push("the default".to_string());
                }
                if said.is_empty() {
                    version.text.clone()
                } else {
                    format!("{} ({})", version.text, said.join(", "))
                }
            })
            .collect();
        report.push(fine("engines", listed.join("; ")));
    }
    if let Default::Set {
        version,
        installed: false,
    } = &default
    {
        report.push(problem(
            "default engine",
            format!("`config.toml` names Zeff {version}, which is not installed"),
            format!(
                "`zeff engine install {version}` installs it, and `zeff engine default <version>` chooses another"
            ),
        ));
    }
    // The store and the user's folder.
    let named = if set("ZEFF_HOME").is_some() {
        "named by `ZEFF_HOME`"
    } else {
        "the default, `.zeff` in the user's folder"
    };
    let writable = |folder: &Path| {
        let probe = folder.join(format!(".doctor-{}", std::process::id()));
        let wrote = std::fs::write(&probe, "").is_ok();
        let _ = std::fs::remove_file(&probe);
        wrote
    };
    if !home.exists() {
        report.push(fine(
            "store",
            format!(
                "`{}`, {named}; not made yet, the first install makes it",
                home.display()
            ),
        ));
    } else if writable(home) {
        report.push(fine("store", format!("`{}`, {named}", home.display())));
    } else {
        report.push(problem(
            "store",
            format!(
                "`{}`, {named}, cannot be written by this user",
                home.display()
            ),
            "give this user the right to write there, or name another folder with `ZEFF_HOME`",
        ));
    }
    if let Some(system) = &store.system
        && system.join("engines").is_dir()
    {
        report.push(fine(
            "shared store",
            format!("`{}`, read after the user's own", system.display()),
        ));
    }
    let user = match machine.platform {
        Platform::Unix => set("HOME"),
        Platform::Windows => set("USERPROFILE"),
    };
    match &user {
        Some(user) => report.push(fine("user folder", format!("`{user}`"))),
        None => report.push(fine("user folder", "not named by the environment")),
    }
    // PATH and completions.
    if let Some(folder) = around.own_path.as_deref().and_then(Path::parent) {
        let separator = match machine.platform {
            Platform::Unix => ':',
            Platform::Windows => ';',
        };
        let on_path = set("PATH").is_some_and(|path| {
            path.split(separator)
                .any(|entry| Path::new(entry) == folder)
        });
        if on_path {
            report.push(fine("PATH", format!("`{}` is on PATH", folder.display())));
        } else {
            let fix = match machine.platform {
                Platform::Unix => format!(
                    "add this line to the shell's start-up file: export PATH=\"{}:$PATH\"",
                    folder.display()
                ),
                Platform::Windows => format!(
                    "add `{}` to the user's PATH in the system's settings",
                    folder.display()
                ),
            };
            report.push(problem(
                "PATH",
                format!(
                    "`{}`, where the launcher is, is not on PATH",
                    folder.display()
                ),
                fix,
            ));
        }
    }
    let shell = set("SHELL").and_then(|shell| {
        Path::new(&shell)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    });
    match (&shell, &user) {
        (Some(shell), Some(user)) if ["bash", "zsh", "fish"].contains(&shell.as_str()) => {
            match completion_file(shell, Path::new(user)) {
                Some(file) => {
                    report.push(fine("completions", format!("for {shell}, at `{}`", file.display())))
                }
                None => report.push(fine(
                    "completions",
                    format!(
                        "none found for {shell} in the usual place; `zeff completions {shell}` writes the file"
                    ),
                )),
            }
        }
        _ => report.push(fine(
            "completions",
            "`zeff completions <shell>` writes the file for bash, zsh, fish or powershell",
        )),
    }
    // The settings in effect, each with where it comes from: the flag, the environment, the
    // user's settings, the default.
    if config
        .as_deref()
        .is_some_and(|text| DeTable::parse(text).is_err())
    {
        report.push(problem(
            "settings",
            format!(
                "`{}` is not valid TOML, so nothing in it is read",
                config_path.display()
            ),
            "correct the file, or move it aside",
        ));
    }
    report.push(fine(
        "default engine",
        match &default {
            Default::Set { version, .. } => {
                format!("Zeff {version}, from `[engine] default` of `config.toml`")
            }
            Default::Newest(version) => {
                format!("Zeff {version}, the newest installed: the default")
            }
            Default::None => "none: no engine is installed".to_string(),
        },
    ));
    let nothing_fetched = asked
        .nothing_fetched
        .map(|flag| format!("forbidden by {flag}, on the command line"))
        .or_else(|| {
            ["ZEFF_FROZEN", "ZEFF_OFFLINE"]
                .into_iter()
                .find(|name| env(name).is_some_and(|value| value == "1"))
                .map(|name| format!("forbidden by `{name}`, from the environment"))
        });
    report.push(fine(
        "fetching",
        nothing_fetched
            .clone()
            .unwrap_or_else(|| "allowed: the default".to_string()),
    ));
    let source = Source::of(env, config.as_deref());
    let from_env = set("ZEFF_MIRROR").is_some();
    match &source {
        Ok(source) if source.mirror => report.push(fine(
            "mirror",
            format!(
                "`{}`, from {}",
                source.base,
                if from_env {
                    "`ZEFF_MIRROR`"
                } else {
                    "`[network] mirror` of `config.toml`"
                }
            ),
        )),
        Ok(_) => report.push(fine("mirror", "none: the default")),
        Err(crate::net::NotAnAddress(written)) => report.push(problem(
            "mirror",
            format!(
                "`{written}`, from {}, is not an address",
                if from_env {
                    "`ZEFF_MIRROR`"
                } else {
                    "`[network] mirror` of `config.toml`"
                }
            ),
            "a mirror begins with `https://` or `http://`",
        )),
    }
    let proxy = config
        .as_deref()
        .and_then(|config| text_at(config, "network", "proxy"));
    report.push(fine(
        "proxy",
        match (&proxy, set("HTTPS_PROXY").or_else(|| set("https_proxy"))) {
            (Some(proxy), _) => format!("`{proxy}`, from `[network] proxy` of `config.toml`"),
            (None, Some(proxy)) => format!("`{proxy}`, from `HTTPS_PROXY`"),
            (None, None) => "none: the default".to_string(),
        },
    ));
    report.push(fine(
        "colour",
        match set("NO_COLOR") {
            Some(_) => "never, from `NO_COLOR`".to_string(),
            None => "on a terminal: the default".to_string(),
        },
    ));
    // The release host.
    let before_one = crate::VERSION.starts_with("0.");
    match (&source, &nothing_fetched) {
        (_, Some(_)) => report.push(fine("release host", "not asked: fetching is forbidden")),
        (Err(_), _) => {}
        (Ok(source), None) if !source.mirror && before_one => report.push(fine(
            "release host",
            format!(
                "not asked: nothing is published at `{}` before Zeff 1.0; an engine comes from a mirror (`ZEFF_MIRROR`) or from an archive (`zeff engine install --from <file>`)",
                source.base
            ),
        )),
        (Ok(source), None) => {
            let own;
            let fetch: &dyn Fetch = match machine.fetch {
                Some(fetch) => fetch,
                None => {
                    own = Http::new(proxy.as_deref());
                    &own
                }
            };
            let address = source.at(ENGINE_INDEX);
            match fetch.get(&address, SMALL) {
                Ok(_) => report.push(fine("release host", format!("`{}` answers, with its engine index", source.base))),
                Err(error) => {
                    let why = match error {
                        FetchError::NotFound => "answers and has no engine index".to_string(),
                        FetchError::Status(status) => format!("answered {status}"),
                        FetchError::TooLarge => "sent more than an index is".to_string(),
                        FetchError::Failed(reason) => format!("did not answer: {reason}"),
                    };
                    report.push(problem(
                        "release host",
                        format!("`{address}`: the server {why}"),
                        "check the network, the mirror and the proxy above; `--offline` works without it, and `zeff engine install --from <file>` installs an archive already on this machine",
                    ));
                }
            }
        }
    }
    // The disk and the last selftest.
    match free_space(around, home) {
        Some(free) if free < TIGHT => report.push(problem(
            "disk",
            format!("{} free where the store is", size_text(free)),
            "make room: an engine needs some tens of megabytes, and `zeff engine remove <version>` removes one that no project pins",
        )),
        Some(free) => report.push(fine("disk", format!("{} free where the store is", size_text(free)))),
        None => report.push(fine("disk", "the free space where the store is could not be read")),
    }
    for version in &versions {
        match last_selftest(home, &version.text) {
            Some(proved) if proved.conformant => report.push(fine(
                "selftest",
                format!(
                    "Zeff {}: conformant, {} of {} cases, on {}",
                    version.text, proved.cases, proved.cases, proved.on
                ),
            )),
            Some(proved) => report.push(problem(
                "selftest",
                format!(
                    "Zeff {}: not conformant, {} of {} cases differ, on {}",
                    version.text, proved.differing, proved.cases, proved.on
                ),
                "this engine does not reproduce the published results on this machine; run `zeff selftest` again for the cases, and report it with `zeff version --verbose`",
            )),
            None => report.push(fine(
                "selftest",
                format!(
                    "Zeff {}: never run here; `zeff selftest` proves the engine on this machine",
                    version.text
                ),
            )),
        }
    }
    if machine.key.is_none() {
        report.push(fine(
            "signing key",
            "none is built into this launcher, a source build: it can prove no archive and installs no engine",
        ));
    }
    report
}

/// Answer `zeff doctor`; the exit code.
pub fn doctor(
    asked: &Asked,
    machine: &Machine<'_>,
    around: &Around<'_>,
    out: &mut dyn Write,
) -> u8 {
    let report = lines(asked, machine, around);
    let problems = report.iter().filter(|line| line.fix.is_some()).count();
    if asked.json {
        let records: Vec<String> = report
            .iter()
            .map(|line| {
                format!(
                    "{{\"about\": {}, \"found\": {}, \"problem\": {}, \"fix\": {}}}",
                    json_string(line.about),
                    json_string(&line.found),
                    line.fix.is_some(),
                    line.fix.as_deref().map_or("null".to_string(), json_string)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "{{\"problems\": {problems}, \"lines\": [\n  {}\n]}}",
            records.join(",\n  ")
        );
    } else {
        let width = report
            .iter()
            .map(|line| line.about.len())
            .max()
            .unwrap_or(0);
        for line in &report {
            let mark = if line.fix.is_some() { "!" } else { " " };
            let _ = writeln!(out, "{mark} {:<width$}  {}", line.about, line.found);
            if let Some(fix) = &line.fix {
                let _ = writeln!(out, "  {:<width$}  fix: {fix}", "");
            }
        }
        let _ = writeln!(
            out,
            "\n{}",
            match problems {
                0 => "no problem found".to_string(),
                1 => "1 problem, marked `!`, with what to do about it".to_string(),
                n => format!("{n} problems, marked `!`, each with what to do about it"),
            }
        );
    }
    u8::from(problems > 0)
}
