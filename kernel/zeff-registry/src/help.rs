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
//! (a function that takes any number of further arguments documents them under `- ...:`)
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
//! statement, continued over several lines when needed) followed by the lines it prints, kept
//! as written, since a table's alignment is part of what it prints.

use crate::catalogue::CatalogueRow;
use crate::spec::FunctionSpec;

/// An example: its caption (the comment line above it, saying what it shows; required by the
/// contract of 0.8.0), the statement (one or more lines, without the `> `) and what it prints.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Example {
    pub caption: Option<String>,
    pub code: Vec<String>,
    pub output: Vec<String>,
}

/// A text block: a paragraph, a bullet list, a short heading inside Details (`### `), or a table
/// of three columns under Coming from R or Stata (`| zeff | R | Stata |`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),
    Bullets(Vec<String>),
    Heading(String),
    Table(Vec<[String; 3]>),
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
    /// The parts of the contract of 0.8.0 (open decision #103), each empty on a page that has
    /// none. In brief: the commonest forms of the call, one bullet each.
    pub in_brief: Vec<Block>,
    /// What the function does with `NA`.
    pub missing_values: Vec<Block>,
    /// The wrong call a newcomer is most likely to write, with the diagnostic the engine prints
    /// for it as a run of `> ` lines (`mistake_example`) and the corrected call.
    pub mistake: Vec<Block>,
    pub mistake_example: Option<Example>,
    /// The equivalent calls as a table of three columns and the differences that would surprise.
    pub coming_from: Vec<Block>,
    /// "To do this other thing, use that function", one bullet each.
    pub related: Vec<Block>,
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
    /// A part the contract of 0.8.0 requires of this function, with the reason it is required.
    MissingPart {
        part: &'static str,
        why: &'static str,
    },
    /// An example without the comment line above it that says what it shows.
    ExampleWithoutCaption(usize),
    /// An optional argument of the function's own that no example writes.
    ArgumentNotInExamples(String),
    /// An example that reads a file of its own instead of a shipped dataset.
    ExampleOnOwnFile(String),
    /// A table row under Coming from R or Stata without three cells.
    MalformedTable(String),
    /// A common mistake whose `> ` run does not end in a diagnostic the engine prints.
    MistakeWithoutDiagnostic,
    /// A related task that names no function in backticks, or one that does not exist.
    UnknownRelated(String),
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
            HelpError::MissingPart { part, why } => {
                write!(f, "the part `{part}` is required: {why}")
            }
            HelpError::ExampleWithoutCaption(n) => write!(
                f,
                "example {n} has no caption: a `// ` line above it saying what it shows"
            ),
            HelpError::ArgumentNotInExamples(p) => {
                write!(f, "no example writes the argument `{p}:`")
            }
            HelpError::ExampleOnOwnFile(l) => write!(
                f,
                "an example runs on the shipped datasets (`dataset(wages)`, `zeff:wages.csv`), not on a file of its own: `{l}`"
            ),
            HelpError::MalformedTable(l) => write!(
                f,
                "a row under Coming from R or Stata is `| zeff | R | Stata |`, not `{l}`"
            ),
            HelpError::MistakeWithoutDiagnostic => write!(
                f,
                "A common mistake shows the wrong call as `> ` lines and, under them, the diagnostic the engine prints"
            ),
            HelpError::UnknownRelated(l) => write!(
                f,
                "a related task names its function in backticks, and the function must exist: `{l}`"
            ),
        }
    }
}

impl std::error::Error for HelpError {}

const WRITTEN: [&str; 13] = [
    "Summary",
    "In brief",
    "Description",
    "Arguments",
    "Value",
    "Missing values",
    "Details",
    "A common mistake",
    "Coming from R or Stata",
    "Related tasks",
    "References",
    "See also",
    "Examples",
];
const GENERATED: [&str; 4] = ["Usage", "Warnings and notes", "Accuracy", "Availability"];

/// Turn the lines of a section into paragraphs and bullet lists.
pub(crate) fn blocks(lines: &[&str]) -> Vec<Block> {
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
        } else if let Some(heading) = trimmed.strip_prefix("### ") {
            // A short heading inside Details.
            flush(&mut paragraph, &mut bullets, &mut out);
            out.push(Block::Heading(heading.trim().to_string()));
        } else if trimmed.starts_with('|') {
            // A row of the three-column table of Coming from R or Stata; the cells are checked
            // by `check_contract`, which reports a row without three.
            flush(&mut paragraph, &mut bullets, &mut out);
            let cells: Vec<String> = trimmed
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            let row: [String; 3] = match cells.len() {
                3 => [cells[0].clone(), cells[1].clone(), cells[2].clone()],
                _ => [trimmed.to_string(), String::new(), String::new()],
            };
            match out.last_mut() {
                Some(Block::Table(rows)) => rows.push(row),
                _ => out.push(Block::Table(vec![row])),
            }
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
            "In brief" => doc.in_brief = blocks(lines),
            "Missing values" => doc.missing_values = blocks(lines),
            "Coming from R or Stata" => doc.coming_from = blocks(lines),
            "Related tasks" => doc.related = blocks(lines),
            "A common mistake" => {
                // The text, and the one run of `> ` lines with the diagnostic under it.
                let mut text: Vec<&str> = Vec::new();
                let mut run: Vec<&str> = Vec::new();
                let mut in_run = false;
                for line in lines {
                    if line.trim_start().starts_with('>') {
                        in_run = true;
                    } else if in_run && line.trim().is_empty() {
                        in_run = false;
                        continue;
                    }
                    if in_run {
                        run.push(line);
                    } else {
                        text.push(line);
                    }
                }
                doc.mistake = blocks(&text);
                let mut examples = Vec::new();
                parse_examples(&run, &mut examples, &mut errors);
                doc.mistake_example = examples.into_iter().next();
            }
            "Arguments" => {
                for line in lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    match line.strip_prefix("- ").and_then(|l| l.split_once(':')) {
                        Some((name, text))
                            if name.trim() == "..."
                                || (!name.trim().is_empty()
                                    && name
                                        .trim()
                                        .chars()
                                        .all(|c| c.is_ascii_alphanumeric() || c == '_')) =>
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
            "Examples" => parse_examples(lines, &mut doc.examples, &mut errors),
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
    if spec.rest.is_some() && !doc.arguments.iter().any(|(n, _)| n == "...") {
        errors.push(HelpError::ArgumentWithoutEntry("...".to_string()));
    }
    for (i, (name, _)) in doc.arguments.iter().enumerate() {
        if name == "..." {
            if spec.rest.is_none() {
                errors.push(HelpError::EntryNamesNoParameter(name.clone()));
            }
        } else if spec.param(name).is_none() {
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

/// The examples of a page: each a run of `> ` lines, then the output it prints (the help
/// contract of section 9.1).
pub(crate) fn parse_examples(
    lines: &[&str],
    examples: &mut Vec<Example>,
    errors: &mut Vec<HelpError>,
) {
    let mut current: Option<Example> = None;
    // An example ends at the next `> ` line after its output, or at a blank line
    // before any output; a blank line inside the output is part of it (a printed
    // frame or table has one after its title), and trailing blank lines are dropped.
    let close = |example: Option<Example>, examples: &mut Vec<Example>| {
        if let Some(mut example) = example {
            while example.output.last().is_some_and(|l| l.is_empty()) {
                example.output.pop();
            }
            examples.push(example);
        }
    };
    // The caption of the example about to start: a `// ` line before its `> ` lines.
    let mut caption: Option<String> = None;
    for line in lines.iter() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            match current.as_mut() {
                Some(example) if !example.output.is_empty() => example.output.push(String::new()),
                _ => close(current.take(), examples),
            }
            continue;
        }
        if let Some(text) = trimmed.trim().strip_prefix("//")
            && current.as_ref().is_none_or(|e| !e.output.is_empty())
        {
            close(current.take(), examples);
            caption = Some(text.trim().to_string());
            continue;
        }
        if let Some(code) = trimmed.strip_prefix('>') {
            let code = code.strip_prefix(' ').unwrap_or(code).to_string();
            match current.as_mut() {
                Some(example) if example.output.is_empty() => example.code.push(code),
                _ => {
                    // A new statement after output, or the first: a new example.
                    close(current.take(), examples);
                    current = Some(Example {
                        caption: caption.take(),
                        code: vec![code],
                        output: Vec::new(),
                    });
                }
            }
        } else {
            match current.as_mut() {
                Some(example) => example.output.push(trimmed.to_string()),
                None => errors.push(HelpError::MalformedExample(trimmed.trim().to_string())),
            }
        }
    }
    close(current.take(), examples);
}

/// Whether a parameter's type reads data: a frame, a variable, a column, terms or a formula.
fn reads_data(ty: &str) -> bool {
    [
        "Frame",
        "Var",
        "Vars",
        "Column",
        "Terms",
        "Formula",
        "Result",
        "RegFit",
        "IvFit",
        "TestResult",
    ]
    .iter()
    .any(|word| {
        ty.split(['<', '>', '|', ' ', ','])
            .any(|part| part == *word)
    })
}

/// The function a related task names: the last word in backticks, `use \`chi2_test\``.
fn backticked(text: &str) -> Option<&str> {
    let end = text.rfind('`')?;
    let start = text[..end].rfind('`')? + 1;
    Some(&text[start..end])
}

/// The checks of the contract of 0.8.0 (Engine and Project Specification, section 9.1, "The
/// contract of 0.8.0"; open decision #103), beside those of [`check_help`]: the parts the
/// function must carry (In brief where it has several ways of being called, Missing values where
/// it reads data, A common mistake where it has a coded diagnostic, Coming from R or Stata where
/// the catalogue names an equivalent), a caption on every example, every optional argument of the
/// function's own in some example, every example on the shipped datasets, the shape of the table
/// and of the mistake, and the function of each related task. Run on every page not on the list
/// of unrevised pages.
pub fn check_contract(
    spec: &FunctionSpec,
    doc: &HelpDoc,
    row: &CatalogueRow,
    known: &dyn Fn(&str) -> bool,
) -> Vec<HelpError> {
    let mut errors = Vec::new();
    let own_options: Vec<&str> = spec
        .params
        .iter()
        .filter(|p| p.option && !p.is_shared())
        .map(|p| p.name)
        .collect();
    let several_forms = row.usage.contains(" or ") || own_options.len() >= 2;
    if several_forms && doc.in_brief.is_empty() {
        errors.push(HelpError::MissingPart {
            part: "In brief",
            why: "the function has more than one way of being called",
        });
    }
    let reads =
        spec.method || spec.params.iter().any(|p| reads_data(p.ty)) || reads_data(spec.returns);
    if reads && doc.missing_values.is_empty() {
        errors.push(HelpError::MissingPart {
            part: "Missing values",
            why: "the function reads data, so the page says what it does with NA",
        });
    }
    let coded = spec.diagnostics.iter().any(|d| !d.code.is_empty());
    if coded && doc.mistake.is_empty() && doc.mistake_example.is_none() {
        errors.push(HelpError::MissingPart {
            part: "A common mistake",
            why: "the function has a coded diagnostic of its own",
        });
    }
    if !doc.mistake.is_empty() || doc.mistake_example.is_some() {
        let shown = doc.mistake_example.as_ref().is_some_and(|e| {
            e.output
                .iter()
                .any(|l| l.starts_with("error[") || l.starts_with("warning["))
        });
        if !shown {
            errors.push(HelpError::MistakeWithoutDiagnostic);
        }
    }
    let equivalent = row.equivalent.trim();
    if !equivalent.is_empty() && !equivalent.starts_with("None") && doc.coming_from.is_empty() {
        errors.push(HelpError::MissingPart {
            part: "Coming from R or Stata",
            why: "the catalogue names an equivalent in R or Stata",
        });
    }
    for block in &doc.coming_from {
        if let Block::Table(rows) = block {
            for cells in rows {
                if cells[1].is_empty() && cells[2].is_empty() {
                    errors.push(HelpError::MalformedTable(cells[0].clone()));
                }
            }
        }
    }
    for block in &doc.related {
        if let Block::Bullets(items) = block {
            for item in items {
                match backticked(item) {
                    Some(name) if known(name) => {}
                    _ => errors.push(HelpError::UnknownRelated(item.clone())),
                }
            }
        }
    }
    for (i, example) in doc.examples.iter().enumerate() {
        if example.caption.as_deref().is_none_or(|c| c.is_empty()) {
            errors.push(HelpError::ExampleWithoutCaption(i + 1));
        }
    }
    let all_code = || {
        doc.examples
            .iter()
            .chain(doc.mistake_example.iter())
            .flat_map(|e| e.code.iter())
    };
    for option in own_options {
        let written = format!("{option}:");
        if !all_code().any(|code| code.contains(&written)) {
            errors.push(HelpError::ArgumentNotInExamples(option.to_string()));
        }
    }
    for code in all_code() {
        let mut rest = code.as_str();
        while let Some(at) = rest.find("import(\"") {
            let path = &rest[at + 8..];
            if !path.starts_with("zeff:") {
                errors.push(HelpError::ExampleOnOwnFile(code.trim().to_string()));
                break;
            }
            rest = path;
        }
    }
    errors
}
