//! The printed help page (Engine and Project Specification, section 9.1; the layout of
//! `docs/spec/zeff_help_examples.md`).
//!
//! ```text
//! where — Keep the rows where a condition is true                   data · tier 1 · since 0.2.0 · 1.0 · exact
//!
//! USAGE
//!     data.where(condition)
//!
//! DESCRIPTION
//!     ...
//! ```
//!
//! The header line is [`PAGE_WIDTH`] columns wide, with the availability at its right end; text
//! is wrapped at [`TEXT_WIDTH`] columns.

use crate::catalogue::CatalogueRow;
use crate::help::{Block, HelpDoc};
use crate::shared::shared_argument;
use crate::spec::{FunctionSpec, Rest};

/// The width of the header line, as the examples of section 9.1 have it.
pub const PAGE_WIDTH: usize = 118;
/// The column at which text is wrapped.
pub const TEXT_WIDTH: usize = 108;
const INDENT: &str = "    ";

fn width(text: &str) -> usize {
    text.chars().count()
}

/// Greedy wrapping on spaces; a word longer than the width stands alone on its line.
pub(crate) fn wrap_text(
    text: &str,
    first_indent: &str,
    rest_indent: &str,
    max: usize,
) -> Vec<String> {
    wrap(text, first_indent, rest_indent, max)
}

fn wrap(text: &str, first_indent: &str, rest_indent: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = first_indent.to_string();
    let mut line_has_words = false;
    for word in text.split_whitespace() {
        let candidate = if line_has_words {
            width(&line) + 1 + width(word)
        } else {
            width(&line) + width(word)
        };
        if line_has_words && candidate > max {
            lines.push(line);
            line = format!("{rest_indent}{word}");
        } else {
            if line_has_words {
                line.push(' ');
            }
            line.push_str(word);
        }
        line_has_words = true;
    }
    if line_has_words {
        lines.push(line);
    }
    lines
}

pub(crate) fn blocks(out: &mut String, blocks: &[Block]) {
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        match block {
            Block::Paragraph(text) => {
                for line in wrap(text, INDENT, INDENT, TEXT_WIDTH) {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
            Block::Bullets(items) => {
                for item in items {
                    for line in wrap(item, "    - ", "      ", TEXT_WIDTH) {
                        out.push_str(&line);
                        out.push('\n');
                    }
                }
            }
        }
    }
}

/// A count with thousands separators, for the header of a dataset page.
pub(crate) fn count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn heading(out: &mut String, title: &str) {
    out.push('\n');
    out.push_str(title);
    out.push('\n');
}

/// The availability as the header prints it: `data · tier 1 · since 0.2.0 · 1.0 · exact`.
pub fn availability(row: &CatalogueRow) -> String {
    let status = match row.status.as_str() {
        // Planned for 1.0 or already implemented: the function is part of 1.0.
        "planned" | "implemented" => "1.0",
        other => other,
    };
    format!(
        "{} · tier {} · since {} · {status} · {}",
        row.package,
        row.tier,
        row.release,
        row.accuracy_words()
    )
}

/// The first line: name and summary on the left, availability on the right.
pub fn header(spec: &FunctionSpec, doc: &HelpDoc, row: &CatalogueRow) -> String {
    let left = format!("{} — {}", spec.name, doc.summary);
    let right = availability(row);
    let gap = PAGE_WIDTH
        .saturating_sub(width(&left) + width(&right))
        .max(2);
    format!("{left}{}{right}", " ".repeat(gap))
}

/// The usage, wrapped after commas with the continuation aligned under the first argument.
pub fn usage_lines(spec: &FunctionSpec) -> Vec<String> {
    let usage = spec.usage();
    if width(&usage) + INDENT.len() <= TEXT_WIDTH {
        return vec![format!("{INDENT}{usage}")];
    }
    let open = usage.find('(').map_or(0, |p| p + 1);
    let rest_indent = " ".repeat(INDENT.len() + open);
    let mut lines = Vec::new();
    let mut line = INDENT.to_string();
    for (i, piece) in usage.split(", ").enumerate() {
        if i == 0 {
            line.push_str(piece);
        } else if width(&line) + 2 + width(piece) > TEXT_WIDTH {
            line.push(',');
            lines.push(line);
            line = format!("{rest_indent}{piece}");
        } else {
            line.push_str(", ");
            line.push_str(piece);
        }
    }
    lines.push(line);
    lines
}

/// The whole page.
pub fn render_page(spec: &FunctionSpec, doc: &HelpDoc, row: &CatalogueRow) -> String {
    let mut out = header(spec, doc, row);
    out.push('\n');

    heading(&mut out, "USAGE");
    for line in usage_lines(spec) {
        out.push_str(&line);
        out.push('\n');
    }

    heading(&mut out, "DESCRIPTION");
    blocks(&mut out, &doc.description);

    heading(&mut out, "ARGUMENTS");
    // Each parameter, then the open-ended arguments as `...` when the function takes them.
    let mut rows: Vec<(String, String, Option<&str>, bool)> = spec
        .params
        .iter()
        .map(|p| {
            let name = if p.option {
                format!("{}:", p.name)
            } else {
                p.name.to_string()
            };
            let ty = match (p.option, p.default) {
                (true, Some(default)) => format!("{} = {default}", p.ty),
                _ => p.ty.to_string(),
            };
            (name, ty, Some(p.name), p.is_shared())
        })
        .collect();
    match &spec.rest {
        Some(Rest::Values(ty)) => rows.push(("...".to_string(), (*ty).to_string(), None, false)),
        Some(Rest::Definitions(ty)) => {
            rows.push(("...".to_string(), format!("name = {ty}"), None, false))
        }
        None => {}
    }
    let shown: Vec<String> = rows.iter().map(|(name, _, _, _)| name.clone()).collect();
    let column = shown.iter().map(|s| width(s)).max().unwrap_or(0).max(10) + 2;
    let hanging = " ".repeat(INDENT.len() + column);
    for ((name, ty, param_name, is_shared), _) in rows.iter().zip(&shown) {
        let key = param_name.unwrap_or("...");
        let written = doc
            .arguments
            .iter()
            .find(|(n, _)| n == key)
            .map(|(_, t)| t.as_str())
            .unwrap_or("");
        let shared = is_shared
            .then(|| param_name.and_then(shared_argument))
            .flatten();
        let meaning = match shared {
            Some(s) if written.is_empty() => s.meaning.to_string(),
            Some(s) => format!("{} {written}", s.meaning),
            None => written.to_string(),
        };
        let text = format!("{ty}. {meaning}");
        let first = format!("{INDENT}{name}{}", " ".repeat(column - width(name)));
        let mut lines = wrap(&text, &first, &hanging, TEXT_WIDTH);
        if shared.is_some() {
            let tag = "(shared)";
            if let Some(last) = lines.last_mut() {
                if width(last) + 2 + tag.len() <= PAGE_WIDTH {
                    let pad = PAGE_WIDTH - width(last) - tag.len();
                    last.push_str(&" ".repeat(pad));
                    last.push_str(tag);
                } else {
                    lines.push(format!("{}{tag}", " ".repeat(PAGE_WIDTH - tag.len())));
                }
            }
        }
        for line in lines {
            out.push_str(&line);
            out.push('\n');
        }
    }

    heading(&mut out, "VALUE");
    blocks(&mut out, &doc.value);

    if !doc.details.is_empty() {
        heading(&mut out, "DETAILS");
        blocks(&mut out, &doc.details);
    }

    if !spec.diagnostics.is_empty() {
        heading(&mut out, "WARNINGS AND NOTES");
        for decl in &spec.diagnostics {
            let code = if decl.code.is_empty() {
                "Note"
            } else {
                decl.code
            };
            let first = format!("{INDENT}{code:<8}");
            for line in wrap(decl.when, &first, &" ".repeat(INDENT.len() + 8), TEXT_WIDTH) {
                out.push_str(&line);
                out.push('\n');
            }
        }
    }

    heading(&mut out, "ACCURACY");
    let class = {
        let words = row.accuracy_words();
        let mut chars = words.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    };
    let accuracy = match &spec.accuracy {
        Some(record) => {
            let mut text = format!("{class}: {}.", record.reference.trim_end_matches('.'));
            if let Some(run) = &record.last_run {
                text.push_str(&format!(" Last run: {}.", run.trim_end_matches('.')));
            }
            text
        }
        None => format!("{class}."),
    };
    for line in wrap(&accuracy, INDENT, INDENT, TEXT_WIDTH) {
        out.push_str(&line);
        out.push('\n');
    }

    if !doc.references.is_empty() {
        heading(&mut out, "REFERENCES");
        for reference in &doc.references {
            out.push_str(INDENT);
            out.push_str(reference);
            out.push('\n');
        }
    }

    heading(&mut out, "SEE ALSO");
    for line in wrap(&doc.see_also.join(", "), INDENT, INDENT, TEXT_WIDTH) {
        out.push_str(&line);
        out.push('\n');
    }

    heading(&mut out, "EXAMPLES");
    for example in &doc.examples {
        for code in &example.code {
            out.push_str(INDENT);
            out.push_str("> ");
            out.push_str(code);
            out.push('\n');
        }
        for output in &example.output {
            if !output.is_empty() {
                out.push_str(INDENT);
                out.push_str(output);
            }
            out.push('\n');
        }
    }
    out
}
