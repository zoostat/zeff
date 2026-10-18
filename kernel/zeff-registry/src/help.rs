//! Help files and their checks (Engine and Project Specification, section 9.1).
//!
//! A built-in function written in Rust has one Markdown file in its package crate's `help/`,
//! with the written parts as level-two headings:
//!
//! ```text
//! # mean
//!
//! ## Summary
//! One line under 80 characters that completes "mean — ...".
//!
//! ## Description
//! One to three paragraphs.
//!
//! ## Arguments
//! - data: The frame.
//! - var: The variable whose mean is taken.
//!
//! ## Value
//! ## Details        (statistical methods)
//! ## References     (statistical methods)
//! ## See also
//! ## Examples
//! > wages.mean(wage)
//! 23.4
//! ```
//!
//! Paragraphs, `code` and bullet lists are the only markup. Usage, Warnings and notes, Accuracy and
//! Availability are generated and may not be written. An example is a run of `> ` lines (its
//! statement, continued over several lines when needed) followed by the lines it prints.

use crate::catalogue::CatalogueRow;
use crate::spec::FunctionSpec;

/// An example: the statement (one or more lines, without the `> `) and what it prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub code: Vec<String>,
    pub output: Vec<String>,
}

/// A text block: a paragraph, or a bullet list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),
    Bullets(Vec<String>),
}

/// The written parts of a help page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelpDoc {
    pub title: String,
    pub summary: String,
    pub description: Vec<Block>,
    /// Parameter name and its text, in the order written.
    pub arguments: Vec<(String, String)>,
    pub value: Vec<Block>,
    pub details: Vec<Block>,
    /// One reference per line.
    pub references: Vec<String>,
    pub see_also: Vec<String>,
    pub examples: Vec<Example>,
}

/// What is wrong with a help page; each is a build failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpError {
    /// The file does not start with `# <name>`, or names another function.
    Title {
        found: String,
        expected: String,
    },
    MissingHeading(&'static str),
    /// A heading of a generated part (Usage, Warnings and notes, Accuracy, Availability), which
    /// may not be written.
    GeneratedPart(String),
    UnknownHeading(String),
    DuplicateHeading(String),
    SummaryEmpty,
    SummaryTooLong(usize),
    /// A summary of more than one line or ending with a full stop.
    SummaryNotOneLine,
    /// A parameter of the signature with no entry under Arguments (a shared argument may have
    /// none).
    ArgumentWithoutEntry(String),
    /// An entry under Arguments that names no parameter.
    EntryNamesNoParameter(String),
    /// An entry written twice.
    DuplicateEntry(String),
    /// An Arguments line that is not `- name: text`.
    MalformedArgument(String),
    EmptyPart(&'static str),
    /// A statistical method without Details or References.
    MissingForStatisticalMethod(&'static str),
    /// A name under See also that no function has.
    UnknownSeeAlso(String),
    NoExample,
    /// An example whose statement is empty or whose lines are not `> ` lines before output.
    MalformedExample(String),
}

impl std::fmt::Display for HelpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpError::Title { found, expected } => write!(
                f,
                "the page is titled `{found}`; it must start with `# {expected}`"
            ),
            HelpError::MissingHeading(h) => write!(f, "the part `{h}` is missing"),
            HelpError::GeneratedPart(h) => write!(f, "`{h}` is generated and may not be written"),
            HelpError::UnknownHeading(h) => write!(f, "`{h}` is not a part of a help page"),
            HelpError::DuplicateHeading(h) => write!(f, "the part `{h}` appears twice"),
            HelpError::SummaryEmpty => write!(f, "the summary is empty"),
            HelpError::SummaryTooLong(n) => {
                write!(f, "the summary is {n} characters; it must be under 80")
            }
            HelpError::SummaryNotOneLine => write!(
                f,
                "the summary is one line that completes \"name — ...\", without a full stop"
            ),
            HelpError::ArgumentWithoutEntry(p) => {
                write!(f, "the parameter `{p}` has no entry under Arguments")
            }
            HelpError::EntryNamesNoParameter(e) => {
                write!(f, "the Arguments entry `{e}` names no parameter")
            }
            HelpError::DuplicateEntry(e) => write!(f, "the Arguments entry `{e}` appears twice"),
            HelpError::MalformedArgument(l) => {
                write!(f, "an Arguments line must be `- name: text`, not `{l}`")
            }
            HelpError::EmptyPart(h) => write!(f, "the part `{h}` is empty"),
            HelpError::MissingForStatisticalMethod(h) => {
                write!(f, "a statistical method needs `{h}`")
            }
            HelpError::UnknownSeeAlso(n) => write!(f, "`{n}` under See also is not a function"),
            HelpError::NoExample => write!(f, "the page needs at least one example"),
            HelpError::MalformedExample(l) => {
                write!(f, "an example starts with `> ` lines; `{l}` does not")
            }
        }
    }
}

impl std::error::Error for HelpError {}

const WRITTEN: [&str; 8] = [
    "Summary",
    "Description",
    "Arguments",
    "Value",
    "Details",
    "References",
    "See also",
    "Examples",
];
const GENERATED: [&str; 4] = ["Usage", "Warnings and notes", "Accuracy", "Availability"];

/// Turn the lines of a section into paragraphs and bullet lists.
fn blocks(lines: &[&str]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut bullets: Vec<String> = Vec::new();
    let flush = |paragraph: &mut Vec<&str>, bullets: &mut Vec<String>, out: &mut Vec<Block>| {
        if !paragraph.is_empty() {
            out.push(Block::Paragraph(paragraph.join(" ")));
            paragraph.clear();
        }
        if !bullets.is_empty() {
            out.push(Block::Bullets(std::mem::take(bullets)));
        }
    };
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut paragraph, &mut bullets, &mut out);
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            if !paragraph.is_empty() {
                flush(&mut paragraph, &mut bullets, &mut out);
            }
            bullets.push(item.trim().to_string());
        } else if !bullets.is_empty() && line.starts_with("  ") {
            // A bullet continued on an indented line.
            if let Some(last) = bullets.last_mut() {
                last.push(' ');
                last.push_str(trimmed);
            }
        } else {
            if !bullets.is_empty() {
                flush(&mut paragraph, &mut bullets, &mut out);
            }
            paragraph.push(trimmed);
        }
    }
    flush(&mut paragraph, &mut bullets, &mut out);
    out
}

/// Parse the text of a help file. Structural problems are returned together.
pub fn parse_help(text: &str) -> Result<HelpDoc, Vec<HelpError>> {
    let mut errors = Vec::new();
    let mut doc = HelpDoc::default();
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            doc.title = title.trim().to_string();
        } else if let Some(heading) = line.strip_prefix("## ") {
            sections.push((heading.trim().to_string(), Vec::new()));
        } else if let Some((_, lines)) = sections.last_mut() {
            lines.push(line);
        } else if !line.trim().is_empty() {
            errors.push(HelpError::UnknownHeading(line.trim().to_string()));
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for (heading, lines) in &sections {
        let Some(part) = WRITTEN.iter().find(|w| w.eq_ignore_ascii_case(heading)) else {
            if GENERATED.iter().any(|g| g.eq_ignore_ascii_case(heading)) {
                errors.push(HelpError::GeneratedPart(heading.clone()));
            } else {
                errors.push(HelpError::UnknownHeading(heading.clone()));
            }
            continue;
        };
        if seen.contains(part) {
            errors.push(HelpError::DuplicateHeading(heading.clone()));
            continue;
        }
        seen.push(part);
        match *part {
            "Summary" => {
                let text: Vec<&str> = lines
                    .iter()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                match text.as_slice() {
                    [] => errors.push(HelpError::SummaryEmpty),
                    [one] => {
                        if one.ends_with('.') {
                            errors.push(HelpError::SummaryNotOneLine);
                        }
                        if one.chars().count() >= 80 {
                            errors.push(HelpError::SummaryTooLong(one.chars().count()));
                        }
                        doc.summary = (*one).to_string();
                    }
                    _ => errors.push(HelpError::SummaryNotOneLine),
                }
            }
            "Description" => doc.description = blocks(lines),
            "Value" => doc.value = blocks(lines),
            "Details" => doc.details = blocks(lines),
            "Arguments" => {
                for line in lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    match line.strip_prefix("- ").and_then(|l| l.split_once(':')) {
                        Some((name, text))
                            if !name.trim().is_empty()
                                && name
                                    .trim()
                                    .chars()
                                    .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
                        {
                            doc.arguments
                                .push((name.trim().to_string(), text.trim().to_string()));
                        }
                        _ if line.starts_with("- ") => {
                            errors.push(HelpError::MalformedArgument(line.to_string()))
                        }
                        _ => {
                            // A continuation of the previous entry.
                            match doc.arguments.last_mut() {
                                Some((_, text)) => {
                                    text.push(' ');
                                    text.push_str(line);
                                }
                                None => errors.push(HelpError::MalformedArgument(line.to_string())),
                            }
                        }
                    }
                }
            }
            "References" => {
                doc.references = lines
                    .iter()
                    .map(|l| l.trim().trim_start_matches("- ").trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
            }
            "See also" => {
                doc.see_also = lines
                    .iter()
                    .flat_map(|l| l.split(','))
                    .map(|n| n.trim().trim_start_matches("- ").trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect();
            }
            "Examples" => {
                let mut current: Option<Example> = None;
                for line in lines.iter() {
                    let trimmed = line.trim_end();
                    if trimmed.trim().is_empty() {
                        if let Some(example) = current.take() {
                            doc.examples.push(example);
                        }
                        continue;
                    }
                    if let Some(code) = trimmed.strip_prefix('>') {
                        let code = code.strip_prefix(' ').unwrap_or(code).to_string();
                        match current.as_mut() {
                            Some(example) if example.output.is_empty() => example.code.push(code),
                            Some(_) => {
                                // A new statement after output: a new example.
                                if let Some(example) = current.take() {
                                    doc.examples.push(example);
                                }
                                current = Some(Example {
                                    code: vec![code],
                                    output: Vec::new(),
                                });
                            }
                            None => {
                                current = Some(Example {
                                    code: vec![code],
                                    output: Vec::new(),
                                })
                            }
                        }
                    } else {
                        match current.as_mut() {
                            Some(example) => example.output.push(trimmed.trim_start().to_string()),
                            None => {
                                errors.push(HelpError::MalformedExample(trimmed.trim().to_string()))
                            }
                        }
                    }
                }
                if let Some(example) = current.take() {
                    doc.examples.push(example);
                }
            }
            _ => {}
        }
    }
    for required in [
        "Summary",
        "Description",
        "Arguments",
        "Value",
        "See also",
        "Examples",
    ] {
        if !seen.contains(&required) {
            errors.push(HelpError::MissingHeading(required));
        }
    }
    if errors.is_empty() {
        Ok(doc)
    } else {
        Err(errors)
    }
}

/// Check a parsed page against the function it documents and the catalogue. `known` says whether
/// a name under See also exists (a catalogue name, or `name:` for a shared argument).
pub fn check_help(
    spec: &FunctionSpec,
    doc: &HelpDoc,
    row: &CatalogueRow,
    known: &dyn Fn(&str) -> bool,
) -> Vec<HelpError> {
    let mut errors = Vec::new();
    if doc.title != spec.name {
        errors.push(HelpError::Title {
            found: doc.title.clone(),
            expected: spec.name.to_string(),
        });
    }
    if doc.description.is_empty() {
        errors.push(HelpError::EmptyPart("Description"));
    }
    if doc.value.is_empty() {
        errors.push(HelpError::EmptyPart("Value"));
    }
    if doc.see_also.is_empty() {
        errors.push(HelpError::EmptyPart("See also"));
    }
    for param in &spec.params {
        let has_entry = doc.arguments.iter().any(|(n, _)| n == param.name);
        if !has_entry && !param.is_shared() {
            errors.push(HelpError::ArgumentWithoutEntry(param.name.to_string()));
        }
    }
    for (i, (name, _)) in doc.arguments.iter().enumerate() {
        if spec.param(name).is_none() {
            errors.push(HelpError::EntryNamesNoParameter(name.clone()));
        } else if doc.arguments[..i].iter().any(|(n, _)| n == name) {
            errors.push(HelpError::DuplicateEntry(name.clone()));
        }
    }
    if row.is_statistical_method() {
        if doc.details.is_empty() {
            errors.push(HelpError::MissingForStatisticalMethod("Details"));
        }
        if doc.references.is_empty() {
            errors.push(HelpError::MissingForStatisticalMethod("References"));
        }
    }
    for name in &doc.see_also {
        if !known(name) {
            errors.push(HelpError::UnknownSeeAlso(name.clone()));
        }
    }
    if doc.examples.is_empty() {
        errors.push(HelpError::NoExample);
    }
    for example in &doc.examples {
        if example.code.iter().all(|l| l.trim().is_empty()) {
            errors.push(HelpError::MalformedExample(example.code.join(" ")));
        }
    }
    errors
}
