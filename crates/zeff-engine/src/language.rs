//! The language services (Engine and Project Specification, section 12, "Editing Zeff"): what
//! an editor asks of the engine while a script is being written. They are plain functions, so
//! that `zeff lsp` calls them over the Language Server Protocol and Zemp, in the browser, calls
//! them directly. They reuse the checker, the registry and the cached schemas, and never run a
//! model.
//!
//! This is part 1 (release 0.7.0, step 34a): the documents an editor has open, and the
//! diagnostics of a document, the checker's own with their codes, suggestions and run-time
//! marks (Language Reference, section 11), placed as the protocol places things: by line, and
//! by the column counted in UTF-16 code units, both from 0.

use crate::fs::{FileInfo, FileSystem, MemoryFileSystem};
use crate::project::Project;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use zeff_diagnostics::{Diagnostic, Severity, Span};

pub mod complete;

pub use complete::{Completion, CompletionKind, Completions, completions};

/// A place in a document as the Language Server Protocol counts it: the line from 0, and the
/// column in UTF-16 code units from 0 (a letter outside the basic plane counts two).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// From a place up to, and not including, another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Where the line that holds a byte begins, and how many lines begin before it. A line ends
/// with a line feed, with a carriage return and a line feed, or with a carriage return alone,
/// as the protocol has it.
fn line_of(text: &str, byte: usize) -> (usize, u32) {
    let bytes = text.as_bytes();
    let (mut start, mut line) = (0, 0u32);
    for (at, &b) in bytes[..byte].iter().enumerate() {
        if b == b'\n' || (b == b'\r' && bytes.get(at + 1) != Some(&b'\n')) {
            start = at + 1;
            line = line.saturating_add(1);
        }
    }
    (start, line)
}

/// The place of a byte of a text. A byte past the end is the end; one inside a character is
/// the start of that character; and one between a carriage return and its line feed is the
/// end of the line.
pub fn position(text: &str, byte: usize) -> Position {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    let bytes = text.as_bytes();
    if byte > 0 && bytes[byte - 1] == b'\r' && bytes.get(byte) == Some(&b'\n') {
        byte -= 1;
    }
    let (start, line) = line_of(text, byte);
    Position {
        line,
        character: u32::try_from(text[start..byte].encode_utf16().count()).unwrap_or(u32::MAX),
    }
}

/// The byte of a text a place names; a column past the end of its line is the end of the line,
/// a line past the end is the end of the text, and a column inside a pair of code units is the
/// start of their character.
pub fn offset(text: &str, at: Position) -> usize {
    let bytes = text.as_bytes();
    let (mut start, mut line) = (0, 0u32);
    while line < at.line {
        match bytes[start..]
            .iter()
            .position(|&b| b == b'\n' || b == b'\r')
        {
            Some(end) => {
                let end = start + end;
                let pair = bytes[end] == b'\r' && bytes.get(end + 1) == Some(&b'\n');
                start = end + if pair { 2 } else { 1 };
                line += 1;
            }
            None => return text.len(),
        }
    }
    let mut units = 0u32;
    for (byte, c) in text[start..].char_indices() {
        let wide = c.len_utf16() as u32;
        if c == '\n' || c == '\r' || units.saturating_add(wide) > at.character {
            return start + byte;
        }
        units += wide;
    }
    text.len()
}

/// What a diagnostic is to an editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
}

/// A diagnostic of a document, as an editor shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// What the editor underlines: the place the diagnostic points at.
    pub range: Range,
    pub level: Level,
    /// The code of section 11, `Z0101`; empty for a note without one.
    pub code: String,
    /// The message, as `zeff check` prints it after the code.
    pub message: String,
    /// What the diagnostic says of the place underlined, which `zeff check` prints under it;
    /// empty where it says nothing.
    pub label: String,
    /// The suggestions of the diagnostic, its `help` lines.
    pub help: Vec<String>,
    /// The other places the diagnostic points at, each with what it says of it.
    pub related: Vec<(Range, String)>,
    /// The mark of section 11: `checked at run time`, or `from last run`.
    pub mark: Option<String>,
}

/// A document an editor has open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Its text as last typed, saved or not.
    pub text: String,
    /// The editor's count of its changes.
    pub version: i64,
}

/// The documents an editor has open, by their paths.
#[derive(Debug, Clone, Default)]
pub struct Documents {
    open: BTreeMap<PathBuf, Document>,
}

impl Documents {
    pub fn new() -> Documents {
        Documents::default()
    }

    /// A document was opened, with its text.
    pub fn open(&mut self, path: impl Into<PathBuf>, text: impl Into<String>, version: i64) {
        self.open.insert(
            path.into(),
            Document {
                text: text.into(),
                version,
            },
        );
    }

    /// An open document was changed: its whole text as it now is. `false` where no such
    /// document is open, and nothing is kept.
    pub fn change(&mut self, path: &Path, text: impl Into<String>, version: i64) -> bool {
        match self.open.get_mut(path) {
            Some(document) => {
                document.text = text.into();
                document.version = version;
                true
            }
            None => false,
        }
    }

    /// A document was closed: the file on disk is its text again. `false` where it was not
    /// open.
    pub fn close(&mut self, path: &Path) -> bool {
        self.open.remove(path).is_some()
    }

    pub fn get(&self, path: &Path) -> Option<&Document> {
        self.open.get(path)
    }

    /// The open documents, in the order of their paths.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.open.keys().map(PathBuf::as_path)
    }
}

/// The files as the editor sees them: those on disk, with each open document in place of its
/// saved text, so that a script is checked against the module beside it as it is being typed.
pub(crate) struct Seen<'a> {
    pub(crate) disk: &'a dyn FileSystem,
    pub(crate) open: &'a Documents,
}

impl FileSystem for Seen<'_> {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        match self.open.get(path) {
            Some(document) => Ok(document.text.clone().into_bytes()),
            None => self.disk.read(path),
        }
    }

    fn metadata(&self, path: &Path) -> io::Result<FileInfo> {
        match self.open.get(path) {
            Some(document) => Ok(FileInfo {
                size: document.text.len() as u64,
                modified: None,
            }),
            None => self.disk.metadata(path),
        }
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        // A document opened and not yet saved is in its folder all the same.
        let unsaved: Vec<String> = self
            .open
            .paths()
            .filter(|path| path.parent() == Some(dir))
            .filter_map(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        let mut names = match self.disk.list(dir) {
            Ok(names) => names,
            Err(error) if unsaved.is_empty() => return Err(error),
            Err(_) => Vec::new(),
        };
        for name in unsaved {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        Ok(names)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.open.get(path).is_some() || self.disk.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.disk.is_dir(path)
            || self
                .open
                .paths()
                .any(|open| open.parent().is_some_and(|parent| parent.starts_with(path)))
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        match self.disk.canonicalize(path) {
            Ok(found) => Ok(found),
            Err(_) if self.open.get(path).is_some() => Ok(path.to_path_buf()),
            Err(error) => Err(error),
        }
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.disk.create_dir(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.disk.write(path, bytes)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.disk.remove_dir_all(path)
    }

    fn now(&self) -> Option<i64> {
        self.disk.now()
    }
}

/// The name of the empty script a module is checked with.
const NO_SCRIPT: &str = "scripts/(none).zeff";

/// The diagnostics of a document, in the order of their places: what the checker says of it
/// before anything runs (section 11), with the same codes and suggestions as `zeff check`.
/// The document's text is the one the editor has open, or the file's where it is not open.
///
/// A script of a project is checked in the project, against its modules (as they are being
/// typed) and its cached schemas. A module of `functions/` is checked as the project's scripts
/// would have it. A file outside any project, or in one whose `zeff.toml` cannot be read, is
/// checked on its own: its syntax, its names and its types, with nothing known, and nothing
/// said, of the files it reads. A text that cannot be read has none.
///
/// The changes a diagnostic offers (its fixes) are not passed on yet: the release serves no
/// code actions.
pub fn diagnostics(disk: &dyn FileSystem, open: &Documents, path: &Path) -> Vec<Finding> {
    let seen = Seen { disk, open };
    let Ok(text) = seen.read_to_string(path) else {
        return Vec::new();
    };
    let Some(check) = check_document(&seen, path, &text) else {
        return Vec::new();
    };
    let mut found = of_file(check.checked.diagnostics.iter(), &check.name, &check.script);
    if check.alone {
        // Nothing is known of the files such a script reads, and nothing is said of them: the
        // note that a file "does not exist yet" would be of the project in memory, not of the
        // script.
        found.retain(|diagnostic| !about_a_file(&text, diagnostic));
    }
    let mut findings: Vec<Finding> = found
        .iter()
        .map(|diagnostic| finding(&text, diagnostic))
        .collect();
    findings.sort_by(|a, b| a.range.cmp(&b.range).then(a.code.cmp(&b.code)));
    findings
}

/// The check a document gets, and how the document is named in it.
pub(crate) struct Check {
    pub checked: zeff_ir::Checked,
    /// The document in the check: its path from the project's folder, with `/` between the
    /// parts.
    pub name: String,
    /// The script the check ran on: the document, or the empty one that stands in for a
    /// module's.
    pub script: String,
    /// Whether the document was checked alone, outside any project.
    pub alone: bool,
}

impl Check {
    /// Whether the document is the script of the check, and not a module.
    pub fn of_script(&self) -> bool {
        self.name == self.script
    }
}

/// Check a document whose text, as the files are seen, is `text`: a script in its project, a
/// module with an empty script standing in, a file outside a project alone in a project of
/// its own in memory. `None` where no check can be made at all.
pub(crate) fn check_document(seen: &Seen<'_>, path: &Path, text: &str) -> Option<Check> {
    let registry = crate::registry::registry().ok()?;
    let folder = path.parent().unwrap_or(Path::new("/"));
    match Project::open(seen, folder) {
        Ok(project) => {
            let name = path
                .strip_prefix(&project.root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            // A module is checked with the project's modules, as any script has them; no
            // script is needed for that, so an empty one stands in.
            let module = name.starts_with("functions/");
            let (script, script_text) = if module {
                (NO_SCRIPT.to_string(), "")
            } else {
                (name.clone(), text)
            };
            let checked =
                crate::run::check_in_project(seen, &project, &script, script_text, registry);
            Some(Check {
                checked: checked.checked,
                name,
                script,
                alone: false,
            })
        }
        Err(_) => {
            // Outside a project, or in one whose `zeff.toml` cannot be read: the file alone,
            // in a project of its own in memory.
            let alone = MemoryFileSystem::new();
            alone.add_file(
                "/alone/zeff.toml",
                format!(
                    "[project]\nname = \"alone\"\nzeff = \"{}\"\nedition = \"preview\"\n",
                    crate::VERSION
                ),
            );
            let name = format!(
                "scripts/{}",
                path.file_name()
                    .map_or("script.zeff".into(), |n| n.to_string_lossy())
            );
            let project = Project::open(&alone, Path::new("/alone")).ok()?;
            let checked = crate::run::check_in_project(&alone, &project, &name, text, registry);
            Some(Check {
                checked: checked.checked,
                script: name.clone(),
                name,
                alone: true,
            })
        }
    }
}

/// The diagnostics that are of one file, among those of a check. One that names no file is of
/// the script checked.
fn of_file<'d>(
    all: impl Iterator<Item = &'d Diagnostic>,
    name: &str,
    script: &str,
) -> Vec<Diagnostic> {
    all.filter(|diagnostic| diagnostic.file.as_deref().unwrap_or(script) == name)
        .cloned()
        .collect()
}

/// Whether a diagnostic is the note that a file read is checked at run time: a note without a
/// code that points at a text between quotes, the file's name.
fn about_a_file(text: &str, diagnostic: &Diagnostic) -> bool {
    diagnostic.severity == Severity::Note
        && diagnostic.code.is_empty()
        && diagnostic.mark.is_some()
        && diagnostic.labels.iter().any(|label| {
            text.get(label.span.start..)
                .is_some_and(|rest| rest.starts_with('"'))
        })
}

/// A diagnostic as an editor shows it, placed in the text it is about.
fn finding(text: &str, diagnostic: &Diagnostic) -> Finding {
    let range = |span: Span| Range {
        start: position(text, span.start),
        end: position(text, span.end),
    };
    let primary = diagnostic
        .labels
        .iter()
        .find(|label| label.primary)
        .or(diagnostic.labels.first());
    Finding {
        // A diagnostic that points at no place is of the document as a whole: its first
        // character.
        range: primary.map_or(range(Span::new(0, 0)), |label| range(label.span)),
        level: match diagnostic.severity {
            Severity::Error => Level::Error,
            Severity::Warning => Level::Warning,
            Severity::Note => Level::Note,
        },
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        label: primary.map_or(String::new(), |label| label.text.clone()),
        help: diagnostic.help.clone(),
        related: diagnostic
            .labels
            .iter()
            .filter(|label| !label.primary && Some(*label) != primary)
            .map(|label| (range(label.span), label.text.clone()))
            .collect(),
        mark: diagnostic.mark.clone(),
    }
}
