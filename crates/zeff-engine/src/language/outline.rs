//! The outline of a document (Engine and Project Specification, section 12: "Folding", "Run" and
//! "Snippets"; open decision #86, release 0.8.0, step 8b): where it folds, the statement at a
//! place or the statements a selection touches, which an interface runs, and the templates of
//! common analyses. All of it is read from the text as typed, by the lexer and the parser the
//! checker uses, and nothing is run. From release 0.9.0, step 28 (open decision #178): the
//! sections of a script, which a comment written as a heading begins.

use super::{Documents, Position, Range, Seen, offset, position};
use crate::fs::FileSystem;
use std::path::Path;
use zeff_diagnostics::Span;
use zeff_registry::Registry;
use zeff_syntax::TokenKind;
use zeff_syntax::ast::Item;

/// A part of a document that folds: its first and its last line, counted from 0. The first
/// line stays in view when the part is folded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fold {
    pub start_line: u32,
    pub end_line: u32,
}

/// What a statement range is the range of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementKind {
    /// A statement of the script, a chain included: what **Run** runs.
    Statement,
    /// A `fn` with its `///` lines: a definition, which a run of the script takes in whole.
    Function,
    /// A `use` line.
    Use,
}

impl StatementKind {
    pub fn name(self) -> &'static str {
        match self {
            StatementKind::Statement => "statement",
            StatementKind::Function => "function",
            StatementKind::Use => "use",
        }
    }
}

/// One statement of a document and where it stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatementRange {
    pub range: Range,
    pub kind: StatementKind,
}

fn span_of(item: &Item) -> (Span, StatementKind) {
    match item {
        Item::Use(used) => (used.span, StatementKind::Use),
        Item::Fn(function) => (function.span, StatementKind::Function),
        Item::Statement(statement) => (statement.span, StatementKind::Statement),
    }
}

/// A section of a script (open decision #178): the part a comment written as a heading
/// begins, `// # Title`, or `// ## Title` for a subsection. It runs to the line before the
/// next heading of its own level or above, or to the script's last line; a section holds its
/// subsections. To the engine the comment is a comment like any other: sections are for an
/// editor's outline and folding, and for an interface that runs a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// 1 for `// # Title`, 2 for `// ## Title`.
    pub level: u8,
    /// The title as written, without the spaces round it.
    pub title: String,
    /// The line of the comment and the section's last line, counted from 0.
    pub line: u32,
    pub end_line: u32,
    /// The whole of the section: from the start of its comment's line to the end of its
    /// last line.
    pub range: Range,
    /// Where the title is written.
    pub title_range: Range,
}

/// The heading a line is, if it is one: its level and where its title begins and ends in
/// the line. A heading is a `//` comment alone on its line (a `///` line is help text and a
/// comment after code is not alone), of one or two `#`, a space, and a title.
fn heading(line: &str) -> Option<(u8, usize, usize)> {
    let comment = line.trim_start();
    let indent = line.len() - comment.len();
    let rest = comment.strip_prefix("//")?;
    if rest.starts_with('/') {
        return None;
    }
    let marks = rest.trim_start();
    let level = marks.chars().take_while(|&c| c == '#').count();
    if !(1..=2).contains(&level) {
        return None;
    }
    let after = &marks[level..];
    let title = after.trim();
    if title.is_empty() || !after.starts_with(char::is_whitespace) {
        return None;
    }
    let start = indent + (comment.len() - after.len()) + (after.len() - after.trim_start().len());
    Some((level as u8, start, start + title.len()))
}

/// The sections of a text, in the order written. A heading counts where it stands between
/// the statements of the script: a line of that look inside a statement that runs over
/// several lines (between the arguments of a call, in the body of a loop or of a function)
/// is a comment and no more. So the formatter, which sets a comment between two statements
/// at the first column and one inside a statement further in, never makes or unmakes a
/// section. A text that does not parse has the sections of the statements the parser could
/// still read.
pub fn sections_of(name: &str, text: &str) -> Vec<Section> {
    let inside = statements(name, text);
    // Each line with the byte it begins at; the last is left out where the text ends with a
    // line break, which has no line after it.
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut at = 0;
    for line in text.split('\n') {
        lines.push((at, line.strip_suffix('\r').unwrap_or(line)));
        at += line.len() + 1;
    }
    if text.ends_with('\n') {
        lines.pop();
    }
    let Some(last) = lines.len().checked_sub(1) else {
        return Vec::new();
    };
    let found: Vec<(usize, u8, usize, usize)> = lines
        .iter()
        .enumerate()
        .filter_map(|(number, (start, line))| {
            let (level, from, to) = heading(line)?;
            if inside
                .iter()
                .any(|(span, _)| span.start < *start && *start < span.end)
            {
                return None;
            }
            Some((number, level, start + from, start + to))
        })
        .collect();
    found
        .iter()
        .enumerate()
        .map(|(i, &(number, level, from, to))| {
            let end = found[i + 1..]
                .iter()
                .find(|next| next.1 <= level)
                .map_or(last, |next| next.0 - 1);
            let (end_start, end_line) = lines[end];
            Section {
                level,
                title: text[from..to].to_string(),
                line: number as u32,
                end_line: end as u32,
                range: Range {
                    start: position(text, lines[number].0),
                    end: position(text, end_start + end_line.len()),
                },
                title_range: Range {
                    start: position(text, from),
                    end: position(text, to),
                },
            }
        })
        .collect()
}

/// The line (from 0) a byte of the text is on.
fn line_of(text: &str, byte: usize) -> u32 {
    text[..byte.min(text.len())].matches('\n').count() as u32
}

/// Where a text folds: every statement, function or `use` that runs over more than one line (a
/// chain written a step to a line, a call broken after its commas), and every pair of braces
/// that does (the body of a function or of a loop, the arms of a `case`, a model block, a
/// record). A part inside another folds on its own; two parts never start on one line, the
/// outer one being kept. A section folds too, from its heading to the last line of it with
/// something written on it, so that the blank lines before the next heading stay in view.
/// A text that does not parse folds where it can still be read: by its braces, and by the
/// statements the parser recovered.
pub fn folds_of(name: &str, text: &str) -> Vec<Fold> {
    let mut folds: Vec<Fold> = Vec::new();
    let mut add = |start: usize, end: usize| {
        let (first, last) = (line_of(text, start), line_of(text, end.max(start + 1) - 1));
        if last > first && !folds.iter().any(|f| f.start_line == first) {
            folds.push(Fold {
                start_line: first,
                end_line: last,
            });
        }
    };
    let parsed = zeff_syntax::parse(name, text);
    for item in &parsed.script.items {
        let (span, _) = span_of(item);
        add(span.start, span.end);
    }
    // The braces, matched as the lexer finds them; one left open folds nothing.
    let lexed = zeff_syntax::lex(name, text);
    let mut open: Vec<usize> = Vec::new();
    for token in &lexed.tokens {
        match token.kind {
            TokenKind::LBrace | TokenKind::LBlock => open.push(token.span.start),
            TokenKind::RBrace => {
                if let Some(start) = open.pop() {
                    add(start, token.span.end);
                }
            }
            _ => {}
        }
    }
    // The sections: a heading is a line no statement begins on.
    let written: Vec<&str> = text.split('\n').collect();
    for section in sections_of(name, text) {
        let last = (section.line..=section.end_line)
            .rev()
            .find(|&line| {
                written
                    .get(line as usize)
                    .is_some_and(|l| !l.trim().is_empty())
            })
            .unwrap_or(section.line);
        if last > section.line {
            folds.push(Fold {
                start_line: section.line,
                end_line: last,
            });
        }
    }
    folds.sort_by_key(|f| (f.start_line, f.end_line));
    folds
}

/// The statements of a text, each with where it stands, in the order written.
fn statements(name: &str, text: &str) -> Vec<(Span, StatementKind)> {
    zeff_syntax::parse(name, text)
        .script
        .items
        .iter()
        .map(span_of)
        .collect()
}

fn ranged(text: &str, span: Span, kind: StatementKind) -> StatementRange {
    StatementRange {
        range: Range {
            start: position(text, span.start),
            end: position(text, span.end),
        },
        kind,
    }
}

/// The statement at a place: the one the place is in, a chain whole, or on one of its lines;
/// between two statements (a blank line, a comment) the next one, as an interface that runs
/// "the current statement" and moves on expects; `None` after the last.
pub fn statement_at(name: &str, text: &str, at: Position) -> Option<StatementRange> {
    let byte = offset(text, at);
    let line = line_of(text, byte);
    statements(name, text)
        .into_iter()
        .find(|(span, _)| {
            let last = line_of(text, span.end.max(span.start + 1) - 1);
            byte < span.end || line <= last
        })
        .map(|(span, kind)| ranged(text, span, kind))
}

/// The statements a selection touches, each one whole, in the order written: what "run the
/// selection" runs. A selection that touches none (blank lines, comments) gives none.
pub fn statements_in(name: &str, text: &str, selection: Range) -> Vec<StatementRange> {
    let (from, to) = (offset(text, selection.start), offset(text, selection.end));
    let (from, to) = (from.min(to), from.max(to));
    statements(name, text)
        .into_iter()
        .filter(|(span, _)| span.start < to.max(from + 1) && from < span.end)
        .map(|(span, kind)| ranged(text, span, kind))
        .collect()
}

/// A template of a common analysis (section 12, "Snippets"): what an editor shows, the word
/// that finds it, and the statements it writes, with the places to fill in as the protocol
/// writes them, `${1:name}`, the same number for the same name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snippet {
    pub label: &'static str,
    /// The word typed that brings the template up.
    pub word: &'static str,
    pub detail: &'static str,
    pub body: &'static str,
    /// The functions the template calls: it is offered by an engine that has them all, so
    /// that a template never writes a call the engine would refuse.
    pub needs: &'static [&'static str],
}

/// The templates, in the order shown.
pub const SNIPPETS: [Snippet; 4] = [
    Snippet {
        label: "regression table",
        word: "regression",
        detail: "Two nested regressions with robust errors, side by side in one table",
        body: "let ${1:m1} = ${2:data}.reg(${3:y} ~ ${4:x}, se: robust)\nlet ${5:m2} = ${2:data}.reg(${3:y} ~ ${4:x} + ${6:control}, se: robust)\nreport([${1:m1}, ${5:m2}]).export(\"output/${7:table1}.docx\")\n",
        needs: &["reg", "report", "export"],
    },
    Snippet {
        label: "instrumental variables with diagnostics",
        word: "ivreg",
        detail: "Two-stage least squares, its first stage and the Anderson–Rubin test",
        body: "let ${1:iv} = ${2:data}.ivreg(${3:y} ~ ${4:exog} + ${5:endog}, instruments: ${5:endog} ~ ${6:instrument}, se: robust)\n${1:iv}.first_stage()\n${1:iv}.ar_test()\n",
        needs: &["ivreg", "first_stage", "ar_test"],
    },
    Snippet {
        label: "difference in differences, two periods",
        word: "did",
        detail: "Treated by post with unit and time effects, clustered by unit",
        body: "let ${1:dd} = ${2:panel}.reg(${3:y} ~ ${4:treated} * ${5:post}, fe: ${6:unit} + ${7:time}, cluster: ${6:unit})\n${1:dd}\n",
        needs: &["reg"],
    },
    Snippet {
        label: "logistic regression",
        word: "logistic",
        detail: "A binary outcome by a generalized linear model with the logit link",
        body: "let ${1:fit} = ${2:data}.glm(${3:y} ~ ${4:x}, family: binomial)\n${1:fit}\n",
        needs: &["glm"],
    },
];

/// The templates this engine offers: those whose functions it has.
pub fn snippets(registry: &Registry) -> Vec<Snippet> {
    SNIPPETS
        .iter()
        .filter(|snippet| {
            snippet
                .needs
                .iter()
                .all(|name| !registry.named(name).is_empty())
        })
        .copied()
        .collect()
}

/// A template with its places filled by their names: the statements it writes when nothing
/// is typed over them.
pub fn filled(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("${") {
        out.push_str(&rest[..at]);
        let inner = &rest[at + 2..];
        let Some(end) = inner.find('}') else {
            out.push_str(&rest[at..]);
            return out;
        };
        let place = &inner[..end];
        out.push_str(place.split_once(':').map_or("", |(_, name)| name));
        rest = &inner[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Whether a place stands where a statement may begin: nothing before it on its line but
/// white space and the word being typed, and the line itself not inside a statement begun on
/// an earlier line. There the templates are offered.
pub fn at_statement_start(name: &str, text: &str, at: Position) -> bool {
    let byte = offset(text, at);
    let line_start = text[..byte.min(text.len())]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let before = &text[line_start..byte.min(text.len())];
    if !before
        .trim_start()
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }
    !statements(name, text)
        .iter()
        .any(|(span, _)| span.start < line_start && byte <= span.end && line_start < span.end)
}

fn read(disk: &dyn FileSystem, open: &Documents, path: &Path) -> Option<(String, String)> {
    let seen = Seen { disk, open };
    let text = seen.read_to_string(path).ok()?;
    let name = path
        .file_name()
        .map_or("document.zeff".into(), |n| n.to_string_lossy().into_owned());
    Some((name, text))
}

/// Where a document folds, as the editor has it; none for one that cannot be read.
pub fn folding_ranges(disk: &dyn FileSystem, open: &Documents, path: &Path) -> Vec<Fold> {
    read(disk, open, path).map_or(Vec::new(), |(name, text)| folds_of(&name, &text))
}

/// The sections of a document, as the editor has it; none for one that cannot be read.
pub fn sections(disk: &dyn FileSystem, open: &Documents, path: &Path) -> Vec<Section> {
    read(disk, open, path).map_or(Vec::new(), |(name, text)| sections_of(&name, &text))
}

/// The statement at a place of a document, as the editor has it.
pub fn statement_range(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    at: Position,
) -> Option<StatementRange> {
    let (name, text) = read(disk, open, path)?;
    statement_at(&name, &text, at)
}

/// The statements a selection of a document touches, as the editor has it.
pub fn statement_ranges(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    selection: Range,
) -> Vec<StatementRange> {
    read(disk, open, path).map_or(Vec::new(), |(name, text)| {
        statements_in(&name, &text, selection)
    })
}

/// The templates to offer at a place of a document: all the engine has, where a statement
/// may begin, and none elsewhere.
pub fn snippets_at(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    at: Position,
    registry: &Registry,
) -> Vec<Snippet> {
    match read(disk, open, path) {
        Some((name, text)) if at_statement_start(&name, &text, at) => snippets(registry),
        _ => Vec::new(),
    }
}
