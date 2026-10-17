//! Finding and opening a project (Engine and Project Specification, section 2).
//!
//! `zeff` walks up from the current folder to the nearest `zeff.toml`, as Cargo and Git do. Before
//! 0.9.0 launcher and engine are one binary, so a project that pins another engine version cannot
//! be run and is refused with Z0710.

use crate::VERSION;
use crate::fs::FileSystem;
use crate::manifest::{Manifest, parse_manifest};
use std::path::{Path, PathBuf};
use zeff_diagnostics::{Diagnostic, Diagnostics};

/// The name of the project file.
pub const MANIFEST: &str = "zeff.toml";

/// An open project: its folder and its settings.
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
    /// The text of `zeff.toml`, which the messages about it quote.
    pub manifest_text: String,
}

/// Why a project could not be opened.
#[derive(Debug)]
pub enum OpenError {
    /// There is no `zeff.toml` in the folder or above it (Z0709).
    NotInProject,
    /// The project file exists but could not be read.
    Unreadable { path: PathBuf, message: String },
    /// The project file is not valid; `diagnostics` refer to `text`.
    Invalid {
        text: String,
        diagnostics: Diagnostics,
    },
}

/// The nearest folder at or above `start` that holds a `zeff.toml`.
pub fn find_root(fs: &dyn FileSystem, start: &Path) -> Option<PathBuf> {
    let mut folder = Some(start);
    while let Some(dir) = folder {
        if fs.is_file(&dir.join(MANIFEST)) {
            return Some(dir.to_path_buf());
        }
        folder = dir.parent();
    }
    None
}

/// Z0709: a command that needs a project was run outside one.
pub fn not_in_project() -> Diagnostic {
    Diagnostic::error(
        "Z0709",
        "not inside a Zeff project; `zeff new <name>` creates one",
    )
}

/// The exit code a diagnostic leads to (Language Reference, section 3.8): 4 when the engine, the
/// edition or the lock does not match the project, 1 for any other error in the project or script.
pub fn exit_code(diagnostics: &[&Diagnostic]) -> i32 {
    let mismatch = |d: &&Diagnostic| matches!(d.code.as_str(), "Z0706" | "Z0710" | "Z0703");
    if diagnostics.iter().any(mismatch) {
        4
    } else {
        1
    }
}

impl Project {
    /// Find the project that holds `start` and read its `zeff.toml`.
    pub fn open(fs: &dyn FileSystem, start: &Path) -> Result<Project, OpenError> {
        let root = find_root(fs, start).ok_or(OpenError::NotInProject)?;
        let path = root.join(MANIFEST);
        let text = fs
            .read_to_string(&path)
            .map_err(|error| OpenError::Unreadable {
                path: path.clone(),
                message: error.to_string(),
            })?;
        match parse_manifest(MANIFEST, &text) {
            Ok(manifest) => Ok(Project {
                root,
                manifest,
                manifest_text: text,
            }),
            Err(diagnostics) => Err(OpenError::Invalid { text, diagnostics }),
        }
    }

    /// The project must run on exactly the engine it pins (Z0710, exit code 4).
    pub fn check_pin(&self) -> Result<(), Box<Diagnostic>> {
        if self.manifest.zeff == VERSION {
            return Ok(());
        }
        let pinned = &self.manifest.zeff;
        Err(Box::new(Diagnostic::error("Z0710", format!("this project needs Zeff {pinned}, but this is Zeff {VERSION}"))
            .in_file(MANIFEST)
            .primary(self.manifest.zeff_span, "the engine the project pins")
            .help(format!(
                "run it with Zeff {pinned}, or change `zeff` in zeff.toml to {VERSION}; installing other engines (`zeff engine install {pinned}`) arrives with release 0.9.0"
            ))))
    }
}
