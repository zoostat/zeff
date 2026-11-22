//! The launcher (Engine and Project Specification, section 2.1; release 0.9.0, step 17): the
//! small half of the `zeff` command. It finds the project the command acts on and the engine
//! version its `zeff.toml` pins, or the default engine outside a project; finds that engine in
//! the store, the user's own first and then a system-wide one; and hands the command line over
//! unchanged. A pinned engine that is not installed is Z0710, exit code 4.
//!
//! ```text
//! ~/.zeff/                   ZEFF_HOME
//!     engines/0.9.0/zeff     one self-contained folder for each engine version
//!     engines/0.9.1/zeff
//!     config.toml            [engine] default = "0.9.1"
//! ```
//!
//! Everything here is a function of the command line, the environment and the files of the
//! store, so that it is tested on a temporary store; `main` alone starts a process.

pub mod archive;
pub mod doctor;
pub mod engines;
pub mod install;
pub mod net;
pub mod remote;
pub mod switch;
pub mod trust;

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use toml::de::{DeTable, DeValue};
use zeff_diagnostics::Diagnostic;

/// The launcher's own version, which the engine prints beside its own (`zeff version
/// --verbose`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The exit code of a command that cannot run because its engine is not there (Language
/// Reference, section 3.8, "Exit codes").
pub const EXIT_ENGINE: u8 = 4;

/// The project's file, which pins the engine: `[project] zeff = "0.9.1"`.
pub const MANIFEST: &str = "zeff.toml";

/// The kind of machine, which decides the names of a few files and folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    Windows,
}

impl Platform {
    /// The machine the launcher was built for.
    pub fn host() -> Platform {
        if cfg!(windows) {
            Platform::Windows
        } else {
            Platform::Unix
        }
    }

    /// The engine's binary in its folder.
    pub fn engine_file(self) -> &'static str {
        match self {
            Platform::Unix => "zeff",
            Platform::Windows => "zeff.exe",
        }
    }
}

/// The engine store: the user's own folder, searched first, and a system-wide one shared by
/// every user, read-only (section 2.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    pub user: PathBuf,
    pub system: Option<PathBuf>,
}

/// The store of this machine from its environment: `ZEFF_HOME` when it is set, else `.zeff`
/// in the user's home folder; and the system-wide store, `/opt/zeff` or `Zeff` in the
/// programs folder of Windows. `None` when the environment names no home at all.
pub fn store(env: &dyn Fn(&str) -> Option<String>, platform: Platform) -> Option<Store> {
    let set = |name: &str| env(name).filter(|value| !value.is_empty());
    let user = match set("ZEFF_HOME") {
        Some(home) => PathBuf::from(home),
        None => {
            let home = match platform {
                Platform::Unix => set("HOME"),
                Platform::Windows => set("USERPROFILE"),
            }?;
            PathBuf::from(home).join(".zeff")
        }
    };
    let system = match platform {
        Platform::Unix => Some(PathBuf::from("/opt/zeff")),
        Platform::Windows => {
            set("ProgramFiles").map(|programs| PathBuf::from(programs).join("Zeff"))
        }
    };
    // A shared install names the system store as its home: it is searched once.
    let system = system.filter(|system| *system != user);
    Some(Store { user, system })
}

/// An engine version: three numbers, and the word after a hyphen of a version before its
/// release (`0.9.0-dev`), which comes before the release itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub numbers: [u64; 3],
    pub pre: Option<String>,
    /// As written, which is the name of the engine's folder.
    pub text: String,
}

impl Version {
    /// `0.9.1`, `0.9.0-dev`; `None` for anything else, a range among them.
    pub fn parse(text: &str) -> Option<Version> {
        let (numbers, pre) = match text.split_once('-') {
            Some((numbers, pre)) if !pre.is_empty() => (numbers, Some(pre.to_string())),
            Some(_) => return None,
            None => (text, None),
        };
        let mut parts = numbers.split('.');
        let mut parsed = [0u64; 3];
        for slot in &mut parsed {
            let part = parts.next()?;
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            *slot = part.parse().ok()?;
        }
        if parts.next().is_some() {
            return None;
        }
        Some(Version {
            numbers: parsed,
            pre,
            text: text.to_string(),
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.numbers
            .cmp(&other.numbers)
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                // A version before its release comes before the release.
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Where the engine `version` is in the store root `root`, whether or not it is there.
pub fn engine_in(root: &Path, version: &str, platform: Platform) -> PathBuf {
    root.join("engines")
        .join(version)
        .join(platform.engine_file())
}

/// The engine `version` in the store: the user's own if it is there, else the system's.
pub fn find_engine(store: &Store, version: &str, platform: Platform) -> Option<PathBuf> {
    std::iter::once(&store.user)
        .chain(store.system.iter())
        .map(|root| engine_in(root, version, platform))
        .find(|engine| engine.is_file())
}

/// The versions installed in the store, the oldest first, each once: a folder of `engines/`
/// named as a version that holds the engine's binary.
pub fn installed(store: &Store, platform: Platform) -> Vec<Version> {
    let mut versions: Vec<Version> = Vec::new();
    for root in std::iter::once(&store.user).chain(store.system.iter()) {
        let Ok(entries) = std::fs::read_dir(root.join("engines")) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(version) = name.to_str().and_then(Version::parse) else {
                continue;
            };
            if entry.path().join(platform.engine_file()).is_file() && !versions.contains(&version) {
                versions.push(version);
            }
        }
    }
    versions.sort();
    versions
}

/// The folder `-p <dir>`, `--project <dir>` or `--project=<dir>` names among the arguments,
/// before any `--` that ends the options; `None` when the command names none.
pub fn project_argument(args: &[String]) -> Option<PathBuf> {
    let mut at = 0;
    while at < args.len() {
        let arg = args[at].as_str();
        if arg == "--" {
            return None;
        }
        if arg == "-p" || arg == "--project" {
            return args.get(at + 1).map(PathBuf::from);
        }
        if let Some(dir) = arg.strip_prefix("--project=") {
            return Some(PathBuf::from(dir));
        }
        at += 1;
    }
    None
}

/// The `zeff.toml` of the project that holds `start`: the nearest at or above it, as Cargo
/// and Git look for theirs.
pub fn manifest_above(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join(MANIFEST))
        .find(|manifest| manifest.is_file())
}

/// The engine version a project's `zeff.toml` pins, `[project] zeff = "0.9.1"`: `None` for
/// a file that cannot be read as TOML or that has no such line, whose faults the engine
/// itself reports in full.
pub fn pin(manifest: &str) -> Option<String> {
    text_at(manifest, "project", "zeff")
}

/// The text at `[table] key = "..."` of a TOML file, read as the engine reads its own
/// `zeff.toml`; `None` when the file is not TOML or has no such text.
pub(crate) fn text_at(file: &str, table: &str, key: &str) -> Option<String> {
    let document = DeTable::parse(file).ok()?;
    let (_, inner) = document.get_ref().get_key_value(table)?;
    let DeValue::Table(inner) = inner.get_ref() else {
        return None;
    };
    let (_, value) = inner.get_key_value(key)?;
    match value.get_ref() {
        DeValue::String(text) => Some(text.to_string()),
        _ => None,
    }
}

/// The default engine of the user's `config.toml`, `[engine] default = "0.9.1"`, which `zeff
/// engine default` writes; `None` when there is no such setting.
pub fn default_setting(config: &str) -> Option<String> {
    text_at(config, "engine", "default")
}

/// What the launcher does with a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// Hand the command to this engine.
    Run { version: String, engine: PathBuf },
    /// The project pins an engine that is not in the store.
    PinMissing {
        version: String,
        manifest: PathBuf,
        installed: Vec<String>,
    },
    /// The default engine of `config.toml` is not in the store.
    DefaultMissing {
        version: String,
        installed: Vec<String>,
    },
    /// No engine is installed at all.
    NoEngine { home: PathBuf },
    /// The environment names no home folder, so there is no store to look in.
    NoHome,
}

/// The engine for a command: the one the project pins, the project being the folder
/// `--project` names or the nearest above `cwd`; outside a project, or in a project whose
/// `zeff.toml` pins nothing readable (which the engine then reports), the default engine: the
/// one `config.toml` names, else the newest release installed.
pub fn choose(
    args: &[String],
    cwd: &Path,
    env: &dyn Fn(&str) -> Option<String>,
    platform: Platform,
) -> Choice {
    let Some(store) = store(env, platform) else {
        return Choice::NoHome;
    };
    let names = |versions: &[Version]| versions.iter().map(|v| v.text.clone()).collect();
    let start = match project_argument(args) {
        Some(dir) if dir.is_absolute() => dir,
        Some(dir) => cwd.join(dir),
        None => cwd.to_path_buf(),
    };
    let pinned = manifest_above(&start).and_then(|manifest| {
        let text = std::fs::read_to_string(&manifest).ok()?;
        Some((pin(&text)?, manifest))
    });
    if let Some((version, manifest)) = pinned {
        return match find_engine(&store, &version, platform) {
            Some(engine) => Choice::Run { version, engine },
            None => Choice::PinMissing {
                version,
                manifest,
                installed: names(&installed(&store, platform)),
            },
        };
    }
    let versions = installed(&store, platform);
    let setting = std::fs::read_to_string(store.user.join("config.toml"))
        .ok()
        .and_then(|config| default_setting(&config));
    if let Some(version) = setting {
        return match find_engine(&store, &version, platform) {
            Some(engine) => Choice::Run { version, engine },
            None => Choice::DefaultMissing {
                version,
                installed: names(&versions),
            },
        };
    }
    // The newest release; a version before its release only when nothing else is installed.
    match engines::newest_release(&versions) {
        Some(newest) => match find_engine(&store, &newest.text, platform) {
            Some(engine) => Choice::Run {
                version: newest.text.clone(),
                engine,
            },
            None => Choice::NoEngine { home: store.user },
        },
        None => Choice::NoEngine { home: store.user },
    }
}

/// `0.9.0`; `0.9.0 and 0.9.1`; `0.9.0, 0.9.1 and 0.9.2`.
fn listed(versions: &[String]) -> String {
    match versions {
        [] => String::new(),
        [one] => one.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// What is said of a command that has no engine to run on (Z0710, exit code 4); `None` for a
/// command that runs.
pub fn refusal(choice: &Choice) -> Option<Diagnostic> {
    let installed_help = |installed: &[String]| {
        if installed.is_empty() {
            "no engine is installed".to_string()
        } else {
            format!(
                "the {} installed {} {}",
                if installed.len() == 1 {
                    "engine"
                } else {
                    "engines"
                },
                if installed.len() == 1 { "is" } else { "are" },
                listed(installed)
            )
        }
    };
    match choice {
        Choice::Run { .. } => None,
        Choice::PinMissing {
            version,
            manifest,
            installed,
        } => Some(
            Diagnostic::error(
                "Z0710",
                format!("this project needs Zeff {version}, which is not installed"),
            )
            .help(format!(
                "`zeff engine install {version}` installs it; {}; the pin is in `{}`",
                installed_help(installed),
                manifest.display()
            )),
        ),
        Choice::DefaultMissing { version, installed } => Some(
            Diagnostic::error(
                "Z0710",
                format!("the default engine, Zeff {version}, is not installed"),
            )
            .help(format!(
                "`zeff engine install {version}` installs it, and `zeff engine default <version>` chooses another; {}",
                installed_help(installed)
            )),
        ),
        Choice::NoEngine { home } => Some(
            Diagnostic::error("Z0710", "no Zeff engine is installed")
                .help(format!(
                    "`zeff engine install` installs the newest into `{}`",
                    home.join("engines").display()
                )),
        ),
        Choice::NoHome => Some(
            Diagnostic::error("Z0710", "no Zeff engine can be found: this user has no home folder")
                .help("set `ZEFF_HOME` to the folder that holds `engines/`"),
        ),
    }
}
