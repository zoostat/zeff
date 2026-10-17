//! `zeff new <name>`: a new project folder (Engine and Project Specification, section 2).

use crate::VERSION;
use crate::fs::FileSystem;
use crate::project::MANIFEST;
use std::path::{Path, PathBuf};

/// Why a project could not be created.
#[derive(Debug, PartialEq, Eq)]
pub enum NewError {
    /// The name is not a project name; the text says what to write instead.
    BadName { name: String, advice: String },
    /// The folder is already there; `zeff new` never changes an existing folder.
    Exists(PathBuf),
    /// A file or folder could not be created.
    Io(String),
}

/// A project name: lowercase letters and digits, with `_` or `-` between words (the naming
/// convention of Language Reference section 3.6, "Projects").
pub fn check_name(name: &str) -> Result<(), NewError> {
    let bad = |advice: String| {
        Err(NewError::BadName {
            name: name.to_string(),
            advice,
        })
    };
    if name.is_empty() {
        return bad("give the project a name, as in `zeff new schooling`".into());
    }
    let tidy: String = name
        .chars()
        .map(|c| {
            if c.is_whitespace() {
                '_'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    let ok = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';
    let starts = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if name.chars().all(ok) && starts && name.len() <= 64 {
        return Ok(());
    }
    if tidy != name
        && tidy.chars().all(ok)
        && tidy
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return bad(format!(
            "write it in lowercase with `_` or `-` between words: `zeff new {tidy}`"
        ));
    }
    bad("use lowercase letters and digits, with `_` or `-` between words, starting with a letter or a digit, in at most 64 characters".into())
}

/// The text of the new project's `zeff.toml`.
pub fn manifest_text(name: &str) -> String {
    let line = |setting: String, comment: &str| format!("{setting:<26}# {comment}\n");
    format!(
        "[project]\n{}{}{}",
        line(format!("name = \"{name}\""), "the project's name"),
        line(
            format!("zeff = \"{VERSION}\""),
            "the engine this project runs on, exactly"
        ),
        line(
            "edition = \"preview\"".to_string(),
            "\"2027\" from Zeff 1.0.0"
        ),
    )
}

/// The text of the new project's `.gitignore`: what is rebuilt is left out, and so is the data
/// unless the user adds it on purpose (section 2, `zeff new`).
pub const GITIGNORE: &str = "# Rebuilt on every run, so not shared.\n.zeff/cache/\n.zeff/fetched/\n.zeff/runs/\n.zeff/open.lock\n.zeff/session.zeff\n.zemp/views.json\n\n# Data stays out of Git unless you add it on purpose.\ndata/\n";

/// The text of the new project's `scripts/main.zeff`.
pub const MAIN_SCRIPT: &str = "// scripts/main.zeff — the script `zeff run` starts from.\n// Put your data in data/ and read it with import(\"data/yourfile.csv\").\n";

/// Create the folder `parent/name` with `zeff.toml`, `scripts/main.zeff`, `data/` and a
/// `.gitignore`. Returns the new project folder. If something goes wrong half-way, what was made
/// is removed again.
pub fn create_project(fs: &dyn FileSystem, parent: &Path, name: &str) -> Result<PathBuf, NewError> {
    check_name(name)?;
    let root = parent.join(name);
    if fs.is_dir(&root) || fs.is_file(&root) {
        return Err(NewError::Exists(root));
    }
    let io = |e: std::io::Error| NewError::Io(e.to_string());
    fs.create_dir(&root).map_err(io)?;
    let build = || -> std::io::Result<()> {
        fs.write(&root.join(MANIFEST), manifest_text(name).as_bytes())?;
        fs.write(&root.join(".gitignore"), GITIGNORE.as_bytes())?;
        fs.create_dir(&root.join("scripts"))?;
        fs.write(
            &root.join("scripts").join("main.zeff"),
            MAIN_SCRIPT.as_bytes(),
        )?;
        fs.create_dir(&root.join("data"))
    };
    if let Err(error) = build() {
        let _ = fs.remove_dir_all(&root);
        return Err(io(error));
    }
    Ok(root)
}
