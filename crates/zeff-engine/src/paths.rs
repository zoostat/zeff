//! The path rules (Engine and Project Specification, section 2, "Paths from the root" and
//! "Writes"; Language Reference, section 3.8, "Paths").
//!
//! A script writes paths with `/` on every system, relative to the project folder. Other paths are
//! refused unless they sit under a data root, written `root:file`. Each path's case is compared
//! with the folder listing, so a script that works on Windows or macOS does not fail on Linux.
//! Writes go only under the project folder.

use crate::fs::FileSystem;
use crate::project::Project;
use std::path::{Path, PathBuf};
use zeff_diagnostics::{Diagnostic, Span};

/// What is wrong with a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathProblem {
    /// The path is empty (Z0701).
    Empty,
    /// The path uses `\` (Z0701).
    Backslash,
    /// The path is absolute, climbs out of the project with `..`, or leaves it through a symbolic
    /// link (Z0701).
    Outside,
    /// `name:file` where no data root `name` is declared (Z0701).
    UnknownRoot(String),
    /// A part of the path differs from the file's real name only in case (Z0702).
    CaseMismatch { written: String, actual: String },
    /// A write outside the project folder (Z0707).
    Unwritable(String),
}

impl PathProblem {
    /// The message for the string `written` at `span` in the file `file`.
    pub fn diagnostic(&self, written: &str, span: Span, file: &str) -> Diagnostic {
        let last = written
            .rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("file");
        let d = match self {
            PathProblem::Empty => Diagnostic::error("Z0701", "the path is empty")
                .primary(span, "no path")
                .help("write a path from the project folder, such as `data/wages.csv`"),
            PathProblem::Backslash => Diagnostic::error("Z0701", format!("path `{written}` uses `\\`"))
                .primary(span, "paths are written with `/` on every system")
                .help("write `/` between folders; `\\` is neither a separator nor a valid escape in a string"),
            PathProblem::Outside => Diagnostic::error("Z0701", format!("path `{written}` is outside the project"))
                .primary(span, "projects read only their own files and declared data roots")
                .help(format!(
                    "declare the folder under [data] in zeff.toml and write `import(\"shared:{last}\")`, or copy the file into data/"
                )),
            PathProblem::UnknownRoot(name) => {
                Diagnostic::error("Z0701", format!("path `{written}` is outside the project: there is no data root `{name}`"))
                    .primary(span, "unknown data root")
                    .help(format!("declare it under [data] in zeff.toml, as `{name} = {{ path = \"...\" }}`"))
            }
            PathProblem::CaseMismatch { written: shown, actual } => {
                Diagnostic::error("Z0702", format!("path `{shown}` differs only in case from `{actual}`"))
                    .primary(span, "the file is called differently")
                    .help(format!("write `{actual}`, so that the script also works on systems that tell upper and lower case apart"))
            }
            PathProblem::Unwritable(why) => Diagnostic::error("Z0707", format!("cannot write `{written}`: {why}"))
                .primary(span, "files are written only under the project folder")
                .help(format!("write under the project, as in `output/{last}`")),
        };
        d.in_file(file)
    }
}

fn is_absolute(text: &str) -> bool {
    let bytes = text.as_bytes();
    text.starts_with('/')
        || (bytes.len() >= 2
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes.get(2).is_none_or(|b| *b == b'/'))
}

/// The folders and file of `text` with `.` and empty parts dropped and `..` applied; `None` if
/// a `..` would climb above the start.
fn normalise(text: &str) -> Option<Vec<&str>> {
    let mut parts: Vec<&str> = Vec::new();
    for part in text.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts)
}

/// `root:file` split into the root name and the file, if `text` has that form (and is not a
/// drive letter such as `C:/x`).
fn split_root(text: &str) -> Option<(&str, &str)> {
    let (name, rest) = text.split_once(':')?;
    let identifier =
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let drive = name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic();
    if identifier && !drive {
        Some((name, rest))
    } else {
        None
    }
}

impl Project {
    /// The real path of a file a script reads: `written` is the string in the script.
    pub fn resolve_read(&self, fs: &dyn FileSystem, written: &str) -> Result<PathBuf, PathProblem> {
        if written.is_empty() {
            return Err(PathProblem::Empty);
        }
        if written.contains('\\') {
            return Err(PathProblem::Backslash);
        }
        if is_absolute(written) {
            return Err(PathProblem::Outside);
        }
        let (base, rest) = match split_root(written) {
            Some((name, rest)) => {
                let root = self
                    .manifest
                    .data_roots
                    .iter()
                    .find(|r| r.name == name)
                    .ok_or_else(|| PathProblem::UnknownRoot(name.to_string()))?;
                let path = Path::new(&root.path);
                let base = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.root.join(path)
                };
                (base, rest)
            }
            None => (self.root.clone(), written),
        };
        // After a data root's name the path is relative to that root; an absolute one is a mistake.
        if is_absolute(rest) {
            return Err(PathProblem::Outside);
        }
        let parts = normalise(rest).ok_or(PathProblem::Outside)?;
        let mut path = base.clone();
        path.extend(&parts);
        check_case(fs, &base, &parts)?;
        check_links(fs, &base, &path)?;
        Ok(path)
    }

    /// The real path of a file a script writes (`save`, `export`): only under the project folder.
    pub fn resolve_write(
        &self,
        fs: &dyn FileSystem,
        written: &str,
    ) -> Result<PathBuf, PathProblem> {
        let unwritable = |why: &str| PathProblem::Unwritable(why.to_string());
        if written.is_empty() {
            return Err(unwritable("the path is empty"));
        }
        if written.contains('\\') {
            return Err(unwritable("paths are written with `/`, not `\\`"));
        }
        if let Some((name, _)) = split_root(written)
            && self.manifest.data_roots.iter().any(|r| r.name == name)
        {
            return Err(unwritable(&format!("the data root `{name}` is read-only")));
        }
        if is_absolute(written) || split_root(written).is_some() {
            return Err(unwritable("it is outside the project"));
        }
        let parts = normalise(written).ok_or_else(|| unwritable("it is outside the project"))?;
        if parts.is_empty() {
            return Err(unwritable("it names the project folder itself"));
        }
        let mut path = self.root.clone();
        path.extend(&parts);
        check_links(fs, &self.root, &path)
            .map_err(|_| unwritable("it leaves the project through a link"))?;
        Ok(path)
    }
}

/// Compare each part of the path with the folder listing (Z0702). A part that does not exist ends
/// the check: a missing file is for the reader to report.
fn check_case(fs: &dyn FileSystem, base: &Path, parts: &[&str]) -> Result<(), PathProblem> {
    let mut folder = base.to_path_buf();
    let mut actual: Vec<String> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let Ok(names) = fs.list(&folder) else {
            return Ok(());
        };
        if names.iter().any(|n| n == part) {
            actual.push((*part).to_string());
        } else if let Some(real) = names
            .iter()
            .find(|n| n.to_lowercase() == part.to_lowercase())
        {
            actual.push(real.clone());
            actual.extend(parts[i + 1..].iter().map(|p| (*p).to_string()));
            return Err(PathProblem::CaseMismatch {
                written: parts.join("/"),
                actual: actual.join("/"),
            });
        } else {
            return Ok(());
        }
        folder.push(part);
    }
    Ok(())
}

/// A symbolic link must not lead out of `base`: the deepest part of `path` that exists is resolved
/// and must still be inside it.
fn check_links(fs: &dyn FileSystem, base: &Path, path: &Path) -> Result<(), PathProblem> {
    let Ok(real_base) = fs.canonicalize(base) else {
        return Ok(());
    };
    let mut probe = path.to_path_buf();
    loop {
        if let Ok(real) = fs.canonicalize(&probe) {
            return if real.starts_with(&real_base) {
                Ok(())
            } else {
                Err(PathProblem::Outside)
            };
        }
        if !probe.pop() || probe.as_os_str().is_empty() {
            return Ok(());
        }
    }
}
