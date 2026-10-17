//! `zeff.toml`, the file that makes a folder a project (Engine and Project Specification,
//! section 2).
//!
//! ```toml
//! [project]
//! name = "schooling"
//! zeff = "0.1.0"          # the exact engine version; never a range
//! edition = "preview"     # "2027" from 1.0.0
//! memory = "3.5GB"        # optional
//! network = false         # optional
//!
//! [data]
//! lfs = { path = "D:/surveys/lfs" }
//! ```
//!
//! Every problem in the file is reported at once, each at the value that causes it. An unknown key
//! is an error, not a warning: a setting that is silently ignored would break the promise that a
//! copied project re-runs unchanged.

use toml::de::{DeTable, DeValue};
use zeff_diagnostics::{Diagnostic, Diagnostics, Span, did_you_mean};

/// The editions this engine knows. Before 1.0.0 the only one is `"preview"` (section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Preview,
}

impl Edition {
    pub fn name(self) -> &'static str {
        match self {
            Edition::Preview => "preview",
        }
    }
}

/// A folder outside the project that scripts may read, declared under `[data]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRoot {
    pub name: String,
    /// As written; relative to the project folder unless absolute.
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    /// The engine version the project pins, exactly.
    pub zeff: String,
    /// Where the pin is written in `zeff.toml`, for messages about it.
    pub zeff_span: Span,
    pub edition: Edition,
    /// The memory budget as written (`"3.5GB"`); it is applied by the engine worker (0.7.0).
    pub memory: Option<String>,
    pub network: bool,
    pub data_roots: Vec<DataRoot>,
}

const PROJECT_KEYS: [&str; 5] = ["name", "zeff", "edition", "memory", "network"];
const TOP_KEYS: [&str; 3] = ["project", "data", "packages"];

/// Whether `text` is an exact version: three numbers, and an optional pre-release tag (a
/// development build is `0.1.0-dev`).
pub fn is_exact_version(text: &str) -> bool {
    let (numbers, tag) = match text.split_once('-') {
        Some((numbers, tag)) => (numbers, Some(tag)),
        None => (text, None),
    };
    let parts: Vec<&str> = numbers.split('.').collect();
    let numeric = |p: &&str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
    parts.len() == 3
        && parts.iter().all(numeric)
        && tag.is_none_or(|t| {
            !t.is_empty()
                && t.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        })
}

/// Whether `text` is a size such as `3.5GB`: a positive number and KB, MB or GB.
fn is_size(text: &str) -> bool {
    let digits_end = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(digits_end);
    matches!(unit, "KB" | "MB" | "GB")
        && number
            .parse::<f64>()
            .is_ok_and(|n| n > 0.0 && n.is_finite())
}

/// A name a data root may have: letters, digits and `_`, not starting with a digit.
fn is_root_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The `[project]` table once it is known to be sound.
struct Project {
    name: String,
    zeff: String,
    zeff_span: Span,
    edition: Edition,
    memory: Option<String>,
    network: bool,
}

struct Parser<'a> {
    file: &'a str,
    found: Diagnostics,
}

fn span_of<T>(spanned: &toml::Spanned<T>) -> Span {
    let range = spanned.span();
    Span::new(range.start, range.end)
}

fn kind(value: &DeValue<'_>) -> &'static str {
    match value {
        DeValue::String(_) => "text",
        DeValue::Integer(_) => "a whole number",
        DeValue::Float(_) => "a number",
        DeValue::Boolean(_) => "true or false",
        DeValue::Datetime(_) => "a date",
        DeValue::Array(_) => "a list",
        DeValue::Table(_) => "a table",
    }
}

impl Parser<'_> {
    fn error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        span: Span,
        label: impl Into<String>,
        help: Option<String>,
    ) {
        let mut diagnostic = Diagnostic::error(code, message)
            .in_file(self.file)
            .primary(span, label);
        if let Some(help) = help {
            diagnostic = diagnostic.help(help);
        }
        self.found.push(diagnostic);
    }

    fn content(
        &mut self,
        message: impl Into<String>,
        span: Span,
        label: impl Into<String>,
        help: Option<String>,
    ) {
        self.error("Z0713", message, span, label, help);
    }

    /// Every key of `table` that is not in `allowed` is an error, with a suggestion if one is near.
    fn unknown_keys(&mut self, table: &DeTable<'_>, allowed: &[&str], place: &str) {
        for (key, _) in table.iter() {
            let name: &str = key.get_ref();
            if !allowed.contains(&name) {
                let help = did_you_mean(name, allowed.iter().copied())
                    .map(|s| format!("did you mean `{s}`?"))
                    .or_else(|| Some(format!("{place} takes: {}", allowed.join(", "))));
                self.content(
                    format!("unknown key `{name}` in {place}"),
                    span_of(key),
                    "not a setting of Zeff",
                    help,
                );
            }
        }
    }

    fn string(&mut self, table: &DeTable<'_>, key: &str, place: &str) -> Option<(String, Span)> {
        let (_, value) = table.get_key_value(key)?;
        match value.get_ref() {
            DeValue::String(text) => Some((text.to_string(), span_of(value))),
            other => {
                self.content(
                    format!("`{key}` in {place} must be text"),
                    span_of(value),
                    format!("this is {}", kind(other)),
                    Some(format!(
                        "write it between quotation marks, as in `{key} = \"...\"`"
                    )),
                );
                None
            }
        }
    }

    fn project(&mut self, table: &DeTable<'_>, whole: Span) -> Option<Project> {
        self.unknown_keys(table, &PROJECT_KEYS, "[project]");
        let name = self.string(table, "name", "[project]");
        if let Some((text, span)) = &name
            && text.trim().is_empty()
        {
            self.content(
                "the project needs a name",
                *span,
                "empty",
                Some("write a name such as `schooling`".into()),
            );
        }
        let zeff = self.string(table, "zeff", "[project]");
        if let Some((text, span)) = &zeff
            && !is_exact_version(text)
        {
            let exact = text.trim_start_matches(|c: char| !c.is_ascii_digit());
            self.content(
                format!("`zeff = \"{text}\"` is not an exact version"),
                *span,
                "pin one exact engine version",
                Some(if is_exact_version(exact) {
                    format!("write `zeff = \"{exact}\"`; a project never takes a range, so that it re-runs unchanged")
                } else {
                    "write three numbers, as in `zeff = \"0.1.0\"`; a project never takes a range".to_string()
                }),
            );
        }
        let edition = self.string(table, "edition", "[project]");
        let edition = edition.and_then(|(text, span)| self.edition(&text, span));
        let memory = self.string(table, "memory", "[project]");
        if let Some((text, span)) = &memory
            && !is_size(text)
        {
            self.content(
                format!("`memory = \"{text}\"` is not a size"),
                *span,
                "expected a number and KB, MB or GB",
                Some("write a size such as `memory = \"3.5GB\"`".into()),
            );
        }
        let network = match table.get_key_value("network") {
            None => Some(false),
            Some((_, value)) => match value.get_ref() {
                DeValue::Boolean(b) => Some(*b),
                other => {
                    self.content(
                        "`network` in [project] must be true or false",
                        span_of(value),
                        format!("this is {}", kind(other)),
                        Some("write `network = true` to let `fetch` reach the network, or leave it out".into()),
                    );
                    None
                }
            },
        };
        for key in ["name", "zeff", "edition"] {
            if !table.contains_key(key) {
                self.content(
                    format!("[project] needs `{key}`"),
                    whole,
                    format!("`{key}` is missing"),
                    Some(match key {
                        "name" => "add `name = \"schooling\"`".to_string(),
                        "zeff" => format!(
                            "add `zeff = \"{}\"`, the engine version the project runs on",
                            crate::VERSION
                        ),
                        _ => "add `edition = \"preview\"`".to_string(),
                    }),
                );
            }
        }
        let (zeff, zeff_span) = zeff?;
        Some(Project {
            name: name?.0,
            zeff,
            zeff_span,
            edition: edition?,
            memory: memory.map(|m| m.0),
            network: network?,
        })
    }

    /// An edition this engine knows, or Z0706 naming the engine that does (section 2, "Editions").
    fn edition(&mut self, text: &str, span: Span) -> Option<Edition> {
        match text {
            "preview" => Some(Edition::Preview),
            "2027" => {
                self.error(
                    "Z0706",
                    "edition 2027 needs a newer Zeff",
                    span,
                    format!("this is Zeff {}", crate::VERSION),
                    Some("edition 2027 is declared by Zeff 1.0.0; until then the only edition is `\"preview\"`".into()),
                );
                None
            }
            other => {
                self.error(
                    "Z0706",
                    format!("`{other}` is not an edition this Zeff knows"),
                    span,
                    "unknown edition",
                    Some("before Zeff 1.0.0 the only edition is `\"preview\"`".into()),
                );
                None
            }
        }
    }

    fn data(&mut self, table: &DeTable<'_>) -> Vec<DataRoot> {
        let mut roots = Vec::new();
        for (key, value) in table.iter() {
            let name: &str = key.get_ref();
            if !is_root_name(name) || name == "zeff" {
                let (message, help) = if name == "zeff" {
                    (
                        "`zeff` is the name of the data root that ships with the engine",
                        "choose another name for this folder",
                    )
                } else {
                    (
                        "a data root is named with letters, digits and `_`",
                        "write a name such as `lfs`",
                    )
                };
                self.content(
                    message,
                    span_of(key),
                    "cannot be used as a data root",
                    Some(help.to_string()),
                );
                continue;
            }
            let DeValue::Table(inner) = value.get_ref() else {
                self.content(
                    format!("the data root `{name}` must be a table with a `path`"),
                    span_of(value),
                    format!("this is {}", kind(value.get_ref())),
                    Some(format!(
                        "write `{name} = {{ path = \"D:/surveys/{name}\" }}`"
                    )),
                );
                continue;
            };
            for (inner_key, _) in inner.iter() {
                let allowed = ["path"];
                if !allowed.contains(&inner_key.get_ref().as_ref()) {
                    self.content(
                        format!(
                            "unknown key `{}` in the data root `{name}`",
                            inner_key.get_ref()
                        ),
                        span_of(inner_key),
                        "a data root has only a `path`",
                        None,
                    );
                }
            }
            match self.string(inner, "path", &format!("the data root `{name}`")) {
                Some((path, span)) if path.trim().is_empty() => {
                    self.content("a data root needs a folder", span, "empty", None);
                }
                Some((path, _)) => roots.push(DataRoot {
                    name: name.to_string(),
                    path,
                }),
                None if !inner.contains_key("path") => {
                    self.content(
                        format!("the data root `{name}` needs a `path`"),
                        span_of(value),
                        "`path` is missing",
                        Some(format!(
                            "write `{name} = {{ path = \"D:/surveys/{name}\" }}`"
                        )),
                    );
                }
                None => {}
            }
        }
        roots
    }
}

/// Read the text of a `zeff.toml`. `file` is the name used in the messages. On any problem every
/// problem found is returned.
pub fn parse_manifest(file: &str, text: &str) -> Result<Manifest, Diagnostics> {
    let mut parser = Parser {
        file,
        found: Diagnostics::new(),
    };
    let document = match DeTable::parse(text) {
        Ok(document) => document,
        Err(error) => {
            let message = error.message().to_string();
            let span = error
                .span()
                .map(|r| Span::new(r.start, r.end))
                .unwrap_or(Span::new(0, 0));
            parser.error(
                "Z0712",
                format!("zeff.toml is not valid TOML: {message}"),
                span,
                message,
                Some(
                    "a project file looks like `[project]` followed by `name = \"...\"` lines"
                        .into(),
                ),
            );
            return Err(parser.found);
        }
    };
    let root = document.get_ref();
    let first_line = text.find('\n').unwrap_or(text.len()).max(1).min(text.len());
    let whole = Span::new(0, first_line);
    parser.unknown_keys(root, &TOP_KEYS, "zeff.toml");

    let project = match root.get_key_value("project") {
        None => {
            parser.content(
                "zeff.toml needs a [project] table",
                whole,
                "no [project]",
                Some(format!(
                    "start the file with `[project]`, `name = \"...\"`, `zeff = \"{}\"` and `edition = \"preview\"`",
                    crate::VERSION
                )),
            );
            None
        }
        Some((key, value)) => match value.get_ref() {
            DeValue::Table(table) => parser.project(table, span_of(key)),
            other => {
                parser.content(
                    "`project` must be a table",
                    span_of(value),
                    format!("this is {}", kind(other)),
                    Some("write `[project]` on a line of its own".into()),
                );
                None
            }
        },
    };

    let mut data_roots = Vec::new();
    if let Some((key, value)) = root.get_key_value("data") {
        match value.get_ref() {
            DeValue::Table(table) => data_roots = parser.data(table),
            other => parser.content(
                "`data` must be a table of data roots",
                span_of(key),
                format!("this is {}", kind(other)),
                Some("write `[data]` and then `lfs = { path = \"...\" }`".into()),
            ),
        }
    }

    if let Some((key, value)) = root.get_key_value("packages") {
        let empty = matches!(value.get_ref(), DeValue::Table(t) if t.is_empty());
        if !empty {
            parser.content(
                "third-party packages are not available yet",
                span_of(key),
                "[packages] must be empty",
                Some("third-party packages arrive after Zeff 1.0; every function of this release is built in".into()),
            );
        }
    }

    if !parser.found.is_empty() {
        // In the order of the file, not the order in which the checks ran.
        let mut all = parser.found.into_vec();
        all.sort_by_key(|d| d.labels.first().map_or(0, |l| l.span.start));
        let mut sorted = Diagnostics::new();
        all.into_iter().for_each(|d| sorted.push(d));
        return Err(sorted);
    }
    match project {
        Some(project) => Ok(Manifest {
            name: project.name,
            zeff: project.zeff,
            zeff_span: project.zeff_span,
            edition: project.edition,
            memory: project.memory,
            network: project.network,
            data_roots,
        }),
        None => Err(parser.found),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_versions() {
        for good in ["0.1.0", "1.0.0", "12.3.45", "0.1.0-dev", "0.1.0-rc.1"] {
            assert!(is_exact_version(good), "{good}");
        }
        for bad in [
            "0.1", "^0.1.0", "~0.1.0", ">=0.1.0", "0.1.x", "1.0.0-", "", "v0.1.0", "0.1.0.0",
        ] {
            assert!(!is_exact_version(bad), "{bad}");
        }
    }

    #[test]
    fn sizes() {
        for good in ["3.5GB", "512MB", "1GB", "64KB", "0.5GB"] {
            assert!(is_size(good), "{good}");
        }
        for bad in ["3.5 GB", "GB", "0GB", "-1GB", "3.5gb", "3.5TB", "3.5", ""] {
            assert!(!is_size(bad), "{bad}");
        }
    }

    #[test]
    fn root_names() {
        assert!(is_root_name("lfs") && is_root_name("_x1") && is_root_name("Survey"));
        assert!(
            !is_root_name("")
                && !is_root_name("1x")
                && !is_root_name("a-b")
                && !is_root_name("a b")
        );
    }
}
