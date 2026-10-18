//! The printed form of a diagnostic, in the style of the examples of Language Reference section 11:
//!
//! ```text
//! error[Z0101]: no variable `incme` in `poll`
//!  --> scripts/main.zeff:4:11
//!   |
//! 4 | poll.mean(incme)
//!   |           ^^^^^ not a variable of poll
//!   |
//!   = help: did you mean `income`?
//! ```
//!
//! The arrow line starts with one space whatever the width of the line numbers, as every example
//! of the specification has it. Columns are counted in scalar values, with a tab shown as four
//! spaces; the marker line therefore lines up except under characters a terminal draws wide.

use crate::{Diagnostic, Label, Source, Sources};
use std::fmt::Write;

const TAB: &str = "    ";

/// The width a text takes when drawn: a tab is four columns, every other scalar value one.
fn display_width(text: &str) -> usize {
    text.chars()
        .map(|c| if c == '\t' { TAB.len() } else { 1 })
        .sum()
}

struct Placed<'a> {
    line: u32,
    /// Offset of the marker from the start of the displayed line, in columns.
    offset: usize,
    length: usize,
    label: &'a Label,
}

fn place<'a>(source: &Source, label: &'a Label) -> Placed<'a> {
    let line = source.line_of(label.span.start);
    let line_start = source.line_start(line);
    let text = source.line_text(line);
    let start = label.span.start.clamp(line_start, line_start + text.len());
    let end = label.span.end.clamp(start, line_start + text.len());
    // Offsets are counted on the line's text, which only holds whole characters.
    let before = text.get(..start - line_start).unwrap_or("");
    let covered = text.get(start - line_start..end - line_start).unwrap_or("");
    Placed {
        line,
        offset: display_width(before),
        length: display_width(covered).max(1),
        label,
    }
}

pub(crate) fn render(diagnostic: &Diagnostic, sources: &Sources) -> String {
    let mut out = String::new();
    if diagnostic.code.is_empty() {
        let _ = writeln!(
            out,
            "{}: {}",
            diagnostic.severity.word(),
            diagnostic.message
        );
    } else {
        let _ = writeln!(
            out,
            "{}[{}]: {}",
            diagnostic.severity.word(),
            diagnostic.code,
            diagnostic.message
        );
    }

    let source = diagnostic
        .file
        .as_deref()
        .and_then(|name| sources.get(name));
    let mut placed: Vec<Placed> = match source {
        Some(source) => diagnostic.labels.iter().map(|l| place(source, l)).collect(),
        None => Vec::new(),
    };
    // By line, then by position on the line; the sort is stable, so equal places keep their order.
    placed.sort_by_key(|p| (p.line, p.offset));

    let width = placed.last().map_or(1, |p| p.line.to_string().len());
    let pad = " ".repeat(width + 1);

    if let (Some(name), Some(source)) = (diagnostic.file.as_deref(), source) {
        let primary = placed
            .iter()
            .find(|p| p.label.primary)
            .or_else(|| placed.first());
        match primary {
            Some(p) => {
                let position = source.position(p.label.span.start);
                let _ = writeln!(out, " --> {name}:{}:{}", position.line, position.column);
            }
            None => {
                let _ = writeln!(out, " --> {name}");
            }
        }
        if !placed.is_empty() {
            let _ = writeln!(out, "{pad}|");
            let mut previous: Option<u32> = None;
            let mut last_line = None;
            for p in &placed {
                if last_line != Some(p.line) {
                    if previous.is_some_and(|before| p.line > before + 1) {
                        let _ = writeln!(out, "...");
                    }
                    let text = source.line_text(p.line).replace('\t', TAB);
                    let _ = writeln!(out, "{:>width$} | {text}", p.line);
                    previous = Some(p.line);
                    last_line = Some(p.line);
                }
                let mark = if p.label.primary { '^' } else { '-' };
                let marks: String = std::iter::repeat_n(mark, p.length).collect();
                let space = " ".repeat(p.offset);
                let words = if p.label.text.is_empty() {
                    String::new()
                } else {
                    format!(" {}", p.label.text)
                };
                let _ = writeln!(out, "{pad}| {space}{marks}{words}");
            }
        }
    } else if let Some(name) = diagnostic.file.as_deref() {
        let _ = writeln!(out, " --> {name}");
    }

    if !diagnostic.help.is_empty() {
        if !placed.is_empty() {
            let _ = writeln!(out, "{pad}|");
        }
        for line in &diagnostic.help {
            let _ = writeln!(out, "{pad}= help: {line}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;

    fn sources(text: &str) -> Sources {
        let mut sources = Sources::new();
        sources.add(Source::new("a.zeff", text));
        sources
    }

    #[test]
    fn a_diagnostic_with_no_place_prints_its_message_and_help() {
        let d = Diagnostic::error("Z0709", "not inside a Zeff project")
            .help("`zeff new <name>` creates one");
        assert_eq!(
            d.render(&Sources::new()),
            "error[Z0709]: not inside a Zeff project\n  = help: `zeff new <name>` creates one\n"
        );
    }

    #[test]
    fn a_warning_and_a_note_use_their_own_word() {
        let w = Diagnostic::warning("W0203", "3 values became NA");
        let n = Diagnostic::note("W0703", "uses today()");
        assert!(w.render(&Sources::new()).starts_with("warning[W0203]: "));
        assert!(n.render(&Sources::new()).starts_with("note[W0703]: "));
    }

    #[test]
    fn tabs_are_drawn_as_four_spaces_and_the_marker_follows() {
        let s = sources("\tx = oops\n");
        let d = Diagnostic::error("Z0101", "no `oops`")
            .in_file("a.zeff")
            .primary(Span::new(5, 9), "here");
        assert_eq!(
            d.render(&s),
            "error[Z0101]: no `oops`\n --> a.zeff:1:6\n  |\n1 |     x = oops\n  |         ^^^^ here\n"
        );
    }

    #[test]
    fn a_label_across_lines_is_drawn_to_the_end_of_its_first_line() {
        let s = sources("f(1,\n  2\n");
        let d = Diagnostic::error("Z0302", "never closed")
            .in_file("a.zeff")
            .primary(Span::new(1, 8), "opened here");
        let text = d.render(&s);
        assert!(text.contains("  |  ^^^ opened here\n"), "{text}");
    }

    #[test]
    fn lines_far_apart_are_separated_by_dots_and_numbers_line_up_on_the_right() {
        let mut text = String::new();
        for i in 1..=12 {
            text.push_str(&format!("line{i}\n"));
        }
        let s = sources(&text);
        let d = Diagnostic::error("Z0101", "m")
            .in_file("a.zeff")
            .primary(
                Span::new(
                    s.get("a.zeff").unwrap().line_start(12),
                    s.get("a.zeff").unwrap().line_start(12) + 4,
                ),
                "late",
            )
            .secondary(Span::new(0, 4), "early");
        let out = d.render(&s);
        assert!(out.contains(" 1 | line1\n"), "{out}");
        assert!(out.contains("...\n12 | line12\n"), "{out}");
    }

    #[test]
    fn two_labels_on_one_line_get_a_marker_line_each() {
        let s = sources("abcdef\n");
        let d = Diagnostic::error("Z0101", "m")
            .in_file("a.zeff")
            .primary(Span::new(0, 2), "first")
            .secondary(Span::new(3, 5), "second");
        let out = d.render(&s);
        assert_eq!(out.matches("1 | abcdef").count(), 1);
        assert!(out.contains("  | ^^ first\n  |    -- second\n"), "{out}");
    }

    #[test]
    fn a_span_past_the_text_does_not_panic() {
        let s = sources("ab\n");
        let d = Diagnostic::error("Z0101", "m")
            .in_file("a.zeff")
            .primary(Span::new(50, 90), "far");
        let _ = d.render(&s);
        let d = Diagnostic::error("Z0101", "m")
            .in_file("missing.zeff")
            .primary(Span::new(0, 1), "x");
        assert!(d.render(&s).contains("--> missing.zeff"));
    }
}
