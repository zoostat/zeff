//! Formatting (Engine and Project Specification, section 12: "`zeff fmt` on save; format
//! selection"; release 0.7.0, step 36c): the changes that put a document, or the statements
//! a selection touches, in the one style of the language (Language Reference, section 3.5).
//!
//! The style is the formatter's, `zeff_syntax::format`, which `zeff fmt` writes with; nothing
//! here decides how a script looks. What is decided here is how little of the document an
//! editor is asked to replace: whole lines, and only those that differ, so that the cursor,
//! the marks and the history of the lines that were right already stay as they are.

use super::{Documents, Position, Range, Seen, offset, position};
use crate::fs::FileSystem;
use std::path::Path;
use zeff_diagnostics::Span;
use zeff_syntax::ast::Item;
use zeff_syntax::format::{Unformatted, format};

/// One change to a document: a stretch of its text, and what stands there afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range,
    pub text: String,
}

/// Why a document was not formatted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotFormatted {
    /// It cannot be read.
    Unreadable,
    /// It does not parse, and is left as it is: the first thing the parser said, and where.
    Syntax { at: Position, message: String },
    /// The formatter could not vouch for what it wrote: a fault of Zeff, with what differed.
    /// The document is left as it is.
    Internal(String),
}

/// The lines of a text, each with its line end, so that together they are the text.
fn lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

/// The stretches where two lists of lines differ, each as the lines of the first it replaces
/// (from, up to) and the lines of the second that stand there instead (from, up to): the
/// fewest lines replaced, by the longest run of lines the two have in common.
fn hunks(old: &[&str], new: &[&str]) -> Vec<(usize, usize, usize, usize)> {
    // What the two begin and end with alike needs no table.
    let mut head = 0;
    while head < old.len() && head < new.len() && old[head] == new[head] {
        head += 1;
    }
    let mut tail = 0;
    while tail < old.len() - head
        && tail < new.len() - head
        && old[old.len() - 1 - tail] == new[new.len() - 1 - tail]
    {
        tail += 1;
    }
    let (a, b) = (&old[head..old.len() - tail], &new[head..new.len() - tail]);
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    // A script is some hundreds of lines; one far longer is replaced between its first and
    // last differing lines without the table.
    if a.len().saturating_mul(b.len()) > 4_000_000 {
        return vec![(head, head + a.len(), head, head + b.len())];
    }
    // The length of the longest common run of lines of `a[i..]` and `b[j..]`.
    let width = b.len() + 1;
    let mut common = vec![0u32; (a.len() + 1) * width];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            common[i * width + j] = if a[i] == b[j] {
                common[(i + 1) * width + j + 1] + 1
            } else {
                common[(i + 1) * width + j].max(common[i * width + j + 1])
            };
        }
    }
    let mut found = Vec::new();
    let (mut i, mut j) = (0, 0);
    let mut open: Option<(usize, usize)> = None;
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            if let Some((from_old, from_new)) = open.take() {
                found.push((head + from_old, head + i, head + from_new, head + j));
            }
            i += 1;
            j += 1;
            continue;
        }
        open.get_or_insert((i, j));
        if j == b.len() || (i < a.len() && common[(i + 1) * width + j] >= common[i * width + j + 1])
        {
            i += 1;
        } else {
            j += 1;
        }
    }
    if let Some((from_old, from_new)) = open {
        found.push((
            head + from_old,
            head + a.len(),
            head + from_new,
            head + b.len(),
        ));
    }
    found
}

/// The byte at which each line of a text begins, and after them the length of the text.
fn starts(lines: &[&str]) -> Vec<usize> {
    let mut at = 0;
    let mut found = Vec::with_capacity(lines.len() + 1);
    for line in lines {
        found.push(at);
        at += line.len();
    }
    found.push(at);
    found
}

/// The first and the last line (from 0) of the text a span covers.
fn lines_of(text: &str, span: Span) -> (usize, usize) {
    let line = |byte: usize| text[..byte.min(text.len())].matches('\n').count();
    (line(span.start), line(span.end.max(span.start + 1) - 1))
}

fn span_of(item: &Item) -> Span {
    match item {
        Item::Use(used) => used.span,
        Item::Fn(function) => function.span,
        Item::Statement(statement) => statement.span,
    }
}

/// The changes that format a document: the whole of it, or with a selection the statements
/// the selection touches and no others. No change where it is formatted already.
///
/// A change replaces whole lines, and only lines that differ. Applied together they give,
/// for the whole document, exactly what `zeff fmt` writes; for a selection, the document with
/// the lines of those statements as `zeff fmt` writes them and every other line as typed. A
/// selection that touches no statement (blank lines, comments between statements) changes
/// nothing.
pub fn formatting(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    selection: Option<Range>,
) -> Result<Vec<TextEdit>, NotFormatted> {
    let seen = Seen { disk, open };
    let text = seen
        .read_to_string(path)
        .map_err(|_| NotFormatted::Unreadable)?;
    let name = path
        .file_name()
        .map_or("document.zeff".into(), |n| n.to_string_lossy());
    let formatted = match format(&name, &text) {
        Ok(formatted) => formatted,
        Err(Unformatted::Syntax(found)) => {
            let first = found
                .iter()
                .find(|d| !d.labels.is_empty())
                .or(found.iter().next());
            return Err(NotFormatted::Syntax {
                at: first.and_then(|d| d.labels.first()).map_or(
                    Position {
                        line: 0,
                        character: 0,
                    },
                    |label| position(&text, label.span.start),
                ),
                message: first.map_or(String::new(), |d| d.message.clone()),
            });
        }
        Err(Unformatted::Internal(what)) => return Err(NotFormatted::Internal(what)),
    };
    let (old, new) = (lines(&text), lines(&formatted));
    let old_starts = starts(&old);
    let edit = |from_old: usize, to_old: usize, from_new: usize, to_new: usize| TextEdit {
        range: Range {
            start: position(&text, old_starts[from_old]),
            end: position(&text, old_starts[to_old]),
        },
        text: new[from_new..to_new].concat(),
    };
    let Some(selection) = selection else {
        return Ok(hunks(&old, &new)
            .into_iter()
            .map(|(a, b, c, d)| edit(a, b, c, d))
            .collect());
    };

    // A selection: the statements it touches, each one whole. The formatted script has the
    // same statements in the same order (the formatter keeps the tree), so the lines of
    // those statements there are what stands in place of their lines here.
    let (from, to) = (offset(&text, selection.start), offset(&text, selection.end));
    let (from, to) = (from.min(to), from.max(to));
    let typed = zeff_syntax::parse(&name, &text);
    let written = zeff_syntax::parse(&name, &formatted);
    if typed.script.items.len() != written.script.items.len() {
        return Err(NotFormatted::Internal(
            "the formatted script has other statements than the script".to_string(),
        ));
    }
    let touched: Vec<usize> = typed
        .script
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            let span = span_of(item);
            // An empty selection touches the statement it stands in or at an end of.
            span.start <= to && from <= span.end
        })
        .map(|(at, _)| at)
        .collect();
    let (Some(first), Some(last)) = (touched.first(), touched.last()) else {
        return Ok(Vec::new());
    };
    let (old_from, _) = lines_of(&text, span_of(&typed.script.items[*first]));
    let (_, old_to) = lines_of(&text, span_of(&typed.script.items[*last]));
    let (new_from, _) = lines_of(&formatted, span_of(&written.script.items[*first]));
    let (_, new_to) = lines_of(&formatted, span_of(&written.script.items[*last]));
    // Within those lines, only the lines that differ.
    let edits: Vec<TextEdit> = hunks(&old[old_from..=old_to], &new[new_from..=new_to])
        .into_iter()
        .map(|(a, b, c, d)| edit(old_from + a, old_from + b, new_from + c, new_from + d))
        .collect();
    // A statement written on the line of another, or whose lines hold more than itself,
    // cannot be changed alone by whole lines: the result must still be the script.
    let result = applied(&text, &edits);
    let again = zeff_syntax::parse(&name, &result);
    if again.diagnostics.has_errors()
        || zeff_syntax::sexp::script(&again.script) != zeff_syntax::sexp::script(&typed.script)
    {
        return Err(NotFormatted::Internal(
            "the selection cannot be formatted apart from the rest of the script".to_string(),
        ));
    }
    Ok(edits)
}

/// A text with changes made to it, as an editor makes them: each at its place in the text
/// as it was, none overlapping another.
pub fn applied(text: &str, edits: &[TextEdit]) -> String {
    let mut ordered: Vec<&TextEdit> = edits.iter().collect();
    ordered.sort_by_key(|edit| edit.range.start);
    let mut out = String::new();
    let mut at = 0;
    for edit in ordered {
        let (start, end) = (offset(text, edit.range.start), offset(text, edit.range.end));
        out.push_str(&text[at..start.max(at)]);
        out.push_str(&edit.text);
        at = end.max(at);
    }
    out.push_str(&text[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn differing(old: &str, new: &str) -> Vec<(usize, usize, usize, usize)> {
        hunks(&lines(old), &lines(new))
    }

    #[test]
    fn the_fewest_lines_are_replaced() {
        assert_eq!(differing("a\nb\nc\n", "a\nb\nc\n"), []);
        assert_eq!(differing("", ""), []);
        // One line changed, one put in, one taken out.
        assert_eq!(differing("a\nb\nc\n", "a\nB\nc\n"), [(1, 2, 1, 2)]);
        assert_eq!(differing("a\nc\n", "a\nb\nc\n"), [(1, 1, 1, 2)]);
        assert_eq!(differing("a\nb\nc\n", "a\nc\n"), [(1, 2, 1, 1)]);
        // Two stretches apart are two changes, the lines between them left alone.
        assert_eq!(
            differing("a\nb\nc\nd\ne\n", "A\nb\nc\nd\nE\n"),
            [(0, 1, 0, 1), (4, 5, 4, 5)]
        );
        assert_eq!(
            differing("a\nx\nb\nc\ny\nd\n", "a\nb\nc\nz\nz\nd\n"),
            [(1, 2, 1, 1), (4, 5, 3, 5)]
        );
        // All of it, and from nothing.
        assert_eq!(differing("a\nb\n", "c\n"), [(0, 2, 0, 1)]);
        assert_eq!(differing("", "a\n"), [(0, 0, 0, 1)]);
        assert_eq!(differing("a\n", ""), [(0, 1, 0, 0)]);
        // A last line without its line end differs from the same with one.
        assert_eq!(differing("a\nb", "a\nb\n"), [(1, 2, 1, 2)]);
        // A carriage return before the line feed is part of the line.
        assert_eq!(differing("a\r\nb\r\n", "a\nb\n"), [(0, 2, 0, 2)]);
    }

    #[test]
    fn changes_are_made_at_their_places_in_the_text_as_it_was() {
        let text = "one\ntwo\nthree\n";
        let at = |line: u32| Position { line, character: 0 };
        let edits = [
            TextEdit {
                range: Range {
                    start: at(2),
                    end: at(3),
                },
                text: "3\n".to_string(),
            },
            TextEdit {
                range: Range {
                    start: at(0),
                    end: at(1),
                },
                text: "1\n1b\n".to_string(),
            },
        ];
        assert_eq!(applied(text, &edits), "1\n1b\ntwo\n3\n");
        assert_eq!(applied(text, &[]), text);
    }
}
