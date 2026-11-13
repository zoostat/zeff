//! Formatting a document and a selection (release 0.7.0, step 36c; Engine and Project
//! Specification, section 12): the changes that put a document, or the statements a
//! selection touches, in the one style of the language, as whole lines and only those that
//! differ.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::language::{
    Documents, NotFormatted, Position, Range, TextEdit, applied, formatting, position,
};
use zeff_syntax::ast::Item;
use zeff_syntax::format::format;

const PATH: &str = "/work/demo/scripts/main.zeff";

/// The changes that format a text typed into a document: the whole, or a selection.
fn edits(text: &str, selection: Option<Range>) -> Result<Vec<TextEdit>, NotFormatted> {
    let fs = MemoryFileSystem::new();
    let mut open = Documents::new();
    open.open(PATH, text, 1);
    formatting(&fs, &open, Path::new(PATH), selection)
}

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

/// A selection from one place to another.
fn from_to(start: (u32, u32), end: (u32, u32)) -> Option<Range> {
    Some(Range {
        start: at(start.0, start.1),
        end: at(end.0, end.1),
    })
}

/// The changes as a text: the lines replaced, and what stands there instead.
fn shown(found: &[TextEdit]) -> String {
    if found.is_empty() {
        return "no change\n".to_string();
    }
    let mut out = String::new();
    for edit in found {
        out.push_str(&format!(
            "lines {} to {} become:\n",
            edit.range.start.line, edit.range.end.line
        ));
        if edit.text.is_empty() {
            out.push_str("    (nothing)\n");
        }
        for line in edit.text.lines() {
            out.push_str(&format!("    |{line}\n"));
        }
    }
    out
}

/// The scripts and modules of the sample projects.
fn samples() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut found = Vec::new();
    for name in [
        "first_engine",
        "data_cleaning",
        "regression",
        "simulation",
        "user_functions",
        "panel",
        "plots",
    ] {
        for sub in ["scripts", "functions"] {
            let mut files: Vec<_> = std::fs::read_dir(root.join(name).join(sub))
                .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
                .unwrap_or_default();
            files.sort();
            for file in files {
                if file.extension().is_some_and(|e| e == "zeff") {
                    let text = std::fs::read_to_string(&file).expect("a script");
                    found.push((format!("{name}/{sub}"), text));
                }
            }
        }
    }
    assert_eq!(found.len(), 9);
    found
}

const UNTIDY: &str = "use functions::clean
let   d = import( \"data/small.csv\" )


// The first model.
let fit=d.reg(b~a,se:robust)   // with robust errors
fit.coef( )
for i in 1..3 {
print(i)
        print( i*2 )
}
/// Halves.
fn half(x:Float)->Float{x/2}
let last = [1,2,
  3]
";

#[test]
fn a_document_is_formatted_by_replacing_only_the_lines_that_differ() {
    let found = edits(UNTIDY, None).expect("formatted");
    // Applied, the changes give what `zeff fmt` writes.
    let formatted = format("main.zeff", UNTIDY).expect("it parses");
    assert_eq!(applied(UNTIDY, &found), formatted);
    // Each replaces whole lines, in order, none touching another; the lines that were right
    // already (`use`, the comment, the braces, the documentation line) are in none of them.
    let mut before = 0;
    for edit in &found {
        assert_eq!(
            (edit.range.start.character, edit.range.end.character),
            (0, 0)
        );
        assert!(edit.range.start.line >= before && edit.range.end.line >= edit.range.start.line);
        before = edit.range.end.line;
        assert!(edit.text.is_empty() || edit.text.ends_with('\n'));
    }
    let untouched = |line: u32| {
        found
            .iter()
            .all(|edit| !(edit.range.start.line <= line && line < edit.range.end.line))
    };
    for (line, text) in UNTIDY.lines().enumerate() {
        let right_already = formatted.lines().any(|formatted| formatted == text);
        if [
            "use functions::clean",
            "// The first model.",
            "}",
            "/// Halves.",
        ]
        .contains(&text)
        {
            assert!(
                right_already && untouched(line as u32),
                "line {line}: {text}"
            );
        }
    }
    insta::assert_snapshot!("a_document", shown(&found));
    // Formatted already: nothing to change.
    assert_eq!(edits(&formatted, None), Ok(Vec::new()));
    assert_eq!(edits("", None), Ok(Vec::new()));
}

#[test]
fn the_sample_scripts_are_formatted_already_and_a_disturbed_one_is_put_right() {
    let mut all = String::new();
    for (name, text) in samples() {
        assert_eq!(edits(&text, None), Ok(Vec::new()), "{name}");
        // Disturbed in three ways a hand does it: a space lost after each comma of one line,
        // a line indented too far, blank lines piled up.
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        let with_comma = lines
            .iter()
            .position(|line| line.contains(", ") && !line.trim_start().starts_with("//"))
            .expect("a line with a comma");
        lines[with_comma] = lines[with_comma].replace(", ", ",");
        // A statement of the script that begins a line; a module has none, only functions
        // under their help lines, which blank lines must not part from them.
        let statement = lines.iter().rposition(|line| {
            !line.is_empty() && !line.starts_with([' ', '/', '}']) && !line.starts_with("fn ")
        });
        if let Some(code) = statement {
            lines[code] = format!("      {}", lines[code]);
            for _ in 0..4 {
                lines.insert(code, String::new());
            }
        }
        let disturbed = format!("{}\n", lines.join("\n"));
        let found = edits(&disturbed, None).unwrap_or_else(|why| panic!("{name}: {why:?}"));
        assert_eq!(
            applied(&disturbed, &found),
            format("main.zeff", &disturbed).expect("it parses"),
            "{name}"
        );
        assert!(
            !found.is_empty() && found.len() <= 3,
            "{name}: {}",
            shown(&found)
        );
        all.push_str(&format!("==== {name}\n{}", shown(&found)));
    }
    insta::assert_snapshot!("sample_scripts_disturbed", all);
}

#[test]
fn a_selection_formats_the_statements_it_touches_and_no_others() {
    let lines: Vec<&str> = UNTIDY.lines().collect();
    // The cursor in one statement, with nothing selected: that statement.
    let found = edits(UNTIDY, from_to((1, 4), (1, 4))).expect("formatted");
    assert_eq!(
        found,
        [TextEdit {
            range: Range {
                start: at(1, 0),
                end: at(2, 0)
            },
            text: "let d = import(\"data/small.csv\")\n".to_string(),
        }]
    );
    // A selection inside one line of a statement of several lines: the whole statement.
    let found = edits(UNTIDY, from_to((9, 2), (9, 5))).expect("formatted");
    let result = applied(UNTIDY, &found);
    assert_eq!(
        result.lines().skip(7).take(4).collect::<Vec<_>>(),
        ["for i in 1..3 {", "    print(i)", "    print(i * 2)", "}"]
    );
    // Everything else is as it was typed, the untidy lines above and below among it.
    assert_eq!(result.lines().take(7).collect::<Vec<_>>(), lines[..7]);
    assert_eq!(result.lines().skip(11).collect::<Vec<_>>(), lines[11..]);
    // Two statements, and what stands between them.
    let found = edits(UNTIDY, from_to((5, 0), (6, 3))).expect("formatted");
    let result = applied(UNTIDY, &found);
    assert_eq!(
        result.lines().skip(5).take(2).collect::<Vec<_>>(),
        [
            "let fit = d.reg(b ~ a, se: robust)                    // with robust errors",
            "fit.coef()"
        ]
    );
    assert_eq!(result.lines().take(5).collect::<Vec<_>>(), lines[..5]);
    // A function and its body, from a place in its name.
    let found = edits(UNTIDY, from_to((12, 4), (12, 4))).expect("formatted");
    let result = applied(UNTIDY, &found);
    // A body short enough for its line stays on it.
    assert!(
        result.contains("/// Halves.\nfn half(x: Float) -> Float { x / 2 }\nlet last = [1,2,\n"),
        "{result}"
    );
    assert!(
        result.starts_with("use functions::clean\nlet   d = import"),
        "{result}"
    );
    // Written backwards a selection is the same selection.
    assert_eq!(
        edits(UNTIDY, from_to((6, 3), (5, 0))),
        edits(UNTIDY, from_to((5, 0), (6, 3)))
    );
    // A selection that touches no statement changes nothing: blank lines, a comment.
    assert_eq!(edits(UNTIDY, from_to((3, 0), (3, 0))), Ok(Vec::new()));
    assert_eq!(edits(UNTIDY, from_to((4, 3), (4, 9))), Ok(Vec::new()));
    // The whole document selected: every statement, as the whole document gives them.
    let all = edits(UNTIDY, from_to((0, 0), (99, 0))).expect("formatted");
    assert_eq!(
        applied(UNTIDY, &all),
        format("main.zeff", UNTIDY).expect("it parses")
    );
    // A statement that was right already: nothing.
    assert_eq!(edits(UNTIDY, from_to((0, 2), (0, 2))), Ok(Vec::new()));
}

#[test]
fn a_document_that_does_not_parse_is_left_as_it_is() {
    let broken = "let d = frame(x = [1.0, 2\nprint(d)\n";
    let Err(NotFormatted::Syntax { at: place, message }) = edits(broken, None) else {
        panic!("formatted: {:?}", edits(broken, None));
    };
    assert_eq!(
        (place.line, message.as_str()),
        (0, "this `(` is never closed")
    );
    // The same for a selection, wherever it stands.
    assert!(matches!(
        edits(broken, from_to((1, 0), (1, 3))),
        Err(NotFormatted::Syntax { .. })
    ));
    // A document that is neither open nor on disk.
    let fs = MemoryFileSystem::new();
    assert_eq!(
        formatting(&fs, &Documents::new(), Path::new(PATH), None),
        Err(NotFormatted::Unreadable)
    );
    // On disk and not open, it is read from the disk; lines ended with a carriage return
    // and a line feed are written with a line feed, as `zeff fmt` writes them.
    fs.add_file(PATH, "let   x = 1\r\nprint( x )\r\n");
    let found = formatting(&fs, &Documents::new(), Path::new(PATH), None).expect("formatted");
    assert_eq!(
        applied("let   x = 1\r\nprint( x )\r\n", &found),
        "let x = 1\nprint(x)\n"
    );
}

/// A script disturbed as a hand disturbs it, without changing what it says: spaces lost
/// after commas, lines indented too far, spaces left at the ends of lines, blank lines piled
/// up before lines that begin a statement.
fn disturbed(text: &str, choices: &[u8]) -> String {
    let mut out = String::new();
    for (line, choice) in text.lines().zip(choices.iter().cycle()) {
        let comment = line.trim_start().starts_with("//");
        match choice % 7 {
            0 if !comment && !line.contains('"') => out.push_str(&line.replace(", ", ",")),
            1 if !line.is_empty() => {
                out.push_str("   ");
                out.push_str(line);
            }
            2 if !comment => {
                out.push_str(line);
                out.push_str("  ");
            }
            3 if !line.is_empty() && !line.starts_with([' ', '}', ')', ']', '.', '/']) => {
                out.push_str("\n\n\n");
                out.push_str(line);
            }
            _ => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

fn text_of<'t>(text: &'t str, item: &Item) -> &'t str {
    let span = match item {
        Item::Use(used) => used.span,
        Item::Fn(function) => function.span,
        Item::Statement(statement) => statement.span,
    };
    &text[span.start..span.end]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// For a sample script disturbed at random and any selection in it: the changes of the
    /// whole document give what `zeff fmt` writes, and nothing is left to change after
    /// them; the changes of the selection leave the script saying what it said, write each
    /// statement the selection touches as `zeff fmt` writes it, and leave every other
    /// statement exactly as typed.
    #[test]
    fn the_changes_format_what_they_should_and_nothing_else(
        sample in 0usize..9,
        choices in proptest::collection::vec(any::<u8>(), 1..40),
        a in (0u32..120, 0u32..60),
        b in (0u32..120, 0u32..60),
    ) {
        let (_, script) = &samples()[sample];
        let text = disturbed(script, &choices);
        let typed = zeff_syntax::parse("main.zeff", &text);
        // The disturbance must leave the script the script.
        prop_assume!(!typed.diagnostics.has_errors());
        let original = zeff_syntax::parse("main.zeff", script);
        prop_assume!(
            zeff_syntax::sexp::script(&typed.script) == zeff_syntax::sexp::script(&original.script)
        );
        let formatted = format("main.zeff", &text).expect("it parses");

        let whole = edits(&text, None).expect("formatted");
        let result = applied(&text, &whole);
        prop_assert_eq!(&result, &formatted);
        prop_assert_eq!(edits(&result, None), Ok(Vec::new()));

        let lines = text.lines().count() as u32;
        let selection = Range {
            start: at(a.0 % (lines + 1), a.1),
            end: at(b.0 % (lines + 1), b.1),
        };
        let part = edits(&text, Some(selection)).expect("formatted");
        let result = applied(&text, &part);
        let after = zeff_syntax::parse("main.zeff", &result);
        prop_assert!(!after.diagnostics.has_errors());
        prop_assert_eq!(
            zeff_syntax::sexp::script(&after.script),
            zeff_syntax::sexp::script(&typed.script)
        );
        let written = zeff_syntax::parse("main.zeff", &formatted);
        let (from, to) = {
            let (x, y) = (
                zeff_engine::language::offset(&text, selection.start),
                zeff_engine::language::offset(&text, selection.end),
            );
            (x.min(y), x.max(y))
        };
        for (k, item) in typed.script.items.iter().enumerate() {
            let before = text_of(&text, item);
            let start = before.as_ptr() as usize - text.as_ptr() as usize;
            let touched = start <= to && from <= start + before.len();
            let now = text_of(&result, &after.script.items[k]);
            if touched {
                prop_assert_eq!(now, text_of(&formatted, &written.script.items[k]), "touched {}", k);
            } else {
                prop_assert_eq!(now, before, "not touched {}", k);
            }
        }
        // And a place in the text as `position` gives it is a place the changes are made at.
        prop_assert!(part.iter().all(|edit| edit.range.start <= edit.range.end));
        let _ = position(&text, from);
    }
}
