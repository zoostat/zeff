//! The outline of a document (Engine and Project Specification, section 12; open decision #86,
//! release 0.8.0, step 8b): where it folds, the statement at a place and those a selection
//! touches, and the templates of common analyses. From release 0.9.0, step 28 (open decision
//! #178): the sections of a script, each begun by a comment written as a heading.

use proptest::prelude::*;
use zeff_engine::language::outline::{
    Fold, SNIPPETS, Section, StatementKind, at_statement_start, filled, folds_of, sections_of,
    snippets, statement_at, statements_in,
};
use zeff_engine::language::{Position, Range};
use zeff_engine::registry::registry;

/// A script with every kind of thing that folds: a `use` line, a function with its page, a
/// chain a step to a line, a call broken after its commas, a loop, a `case`, a record over
/// lines, and one-line statements between them.
const SCRIPT: &str = "use functions::clean
/// Twice a number.
fn twice(x: Float) -> Float {
    x * 2
}

let w = dataset(wages)
let m = w
    .where(year >= 2018)
    .add(lwage = log(wage))
    .reg(lwage ~ educ + exper,
         se: robust)
m

// A loop over two variables.
for v in vars(wage, hours) {
    print(w.mean(v))
}
let band = case {
    w.n_rows() > 10000 => \"large\",
    _ => \"small\",
}
let opts = {
    se: robust,
    level: 0.9,
}
print(band)
";

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

#[test]
fn chains_blocks_and_functions_fold() {
    let folds: Vec<(u32, u32)> = folds_of("main.zeff", SCRIPT)
        .iter()
        .map(|f| (f.start_line, f.end_line))
        .collect();
    assert_eq!(
        folds,
        [
            // The function, from its page to its closing brace: its braces start on a later
            // line and fold on their own.
            (1, 4),
            (2, 4),
            // The chain, five lines.
            (7, 11),
            // The loop, the `case` and the record, each by its statement.
            (15, 17),
            (18, 21),
            (22, 25),
        ]
    );
    // One-line statements do not fold, and a script of them has no fold at all.
    assert!(folds_of("main.zeff", "let a = 1\nprint(a)\n").is_empty());
    // Nested blocks fold each on its own line.
    let nested = "for i in [1, 2] {\n    if i > 1 {\n        print(i)\n    }\n}\n";
    let folds: Vec<(u32, u32)> = folds_of("main.zeff", nested)
        .iter()
        .map(|f| (f.start_line, f.end_line))
        .collect();
    assert_eq!(folds, [(0, 4), (1, 3)]);
    // A text that does not parse still folds by its braces, and never panics.
    let broken = "for i in [1, 2] {\n    print(i\n}\nlet x = \n";
    assert_eq!(
        folds_of("main.zeff", broken)
            .iter()
            .map(|f| (f.start_line, f.end_line))
            .collect::<Vec<_>>()
            .first(),
        Some(&(0, 2))
    );
    assert!(folds_of("main.zeff", "").is_empty());
    assert!(folds_of("main.zeff", "}}}{{{\n{\n").is_empty());
}

#[test]
fn the_statement_at_a_place_is_the_whole_chain_or_the_next_statement() {
    let lines = |found: Option<zeff_engine::language::StatementRange>| {
        found.map(|s| (s.range.start.line, s.range.end.line, s.kind))
    };
    // On each line of the chain: the chain whole.
    for line in 7..=11 {
        assert_eq!(
            lines(statement_at("main.zeff", SCRIPT, at(line, 6))),
            Some((7, 11, StatementKind::Statement)),
            "line {line}"
        );
    }
    // A one-line statement, at its start, in it and at its end.
    for character in [0, 4, 22] {
        assert_eq!(
            lines(statement_at("main.zeff", SCRIPT, at(6, character))),
            Some((6, 6, StatementKind::Statement))
        );
    }
    // On a blank line and on a comment: the next statement.
    assert_eq!(
        lines(statement_at("main.zeff", SCRIPT, at(13, 0))),
        Some((15, 17, StatementKind::Statement))
    );
    assert_eq!(
        lines(statement_at("main.zeff", SCRIPT, at(14, 5))),
        Some((15, 17, StatementKind::Statement))
    );
    // Inside the body of the loop: the loop, since a pass alone has no loop variable.
    assert_eq!(
        lines(statement_at("main.zeff", SCRIPT, at(16, 8))),
        Some((15, 17, StatementKind::Statement))
    );
    // A function and a `use` line say what they are.
    assert_eq!(
        lines(statement_at("main.zeff", SCRIPT, at(3, 4))).map(|s| s.2),
        Some(StatementKind::Function)
    );
    assert_eq!(
        lines(statement_at("main.zeff", SCRIPT, at(0, 2))),
        Some((0, 0, StatementKind::Use))
    );
    // After the last statement there is none; nor in an empty text.
    assert_eq!(statement_at("main.zeff", SCRIPT, at(40, 0)), None);
    assert_eq!(statement_at("main.zeff", "", at(0, 0)), None);
    assert_eq!(
        statement_at("main.zeff", "// only a comment\n", at(0, 3)),
        None
    );
}

#[test]
fn a_selection_gives_the_statements_it_touches_each_one_whole() {
    let touched = |from: Position, to: Position| -> Vec<(u32, u32)> {
        statements_in(
            "main.zeff",
            SCRIPT,
            Range {
                start: from,
                end: to,
            },
        )
        .iter()
        .map(|s| (s.range.start.line, s.range.end.line))
        .collect()
    };
    // From the middle of the chain to the middle of the loop: the chain, `m` and the loop.
    assert_eq!(touched(at(9, 4), at(16, 2)), [(7, 11), (12, 12), (15, 17)]);
    // A selection of blank lines and a comment touches nothing.
    assert_eq!(touched(at(13, 0), at(14, 10)), []);
    // An empty selection inside a statement is that statement; written backwards too.
    assert_eq!(touched(at(6, 3), at(6, 3)), [(6, 6)]);
    assert_eq!(touched(at(16, 2), at(9, 4)), [(7, 11), (12, 12), (15, 17)]);
    // The whole document: every statement, the function and the `use` line among them.
    assert_eq!(touched(at(0, 0), at(40, 0)).len(), 9);
}

#[test]
fn the_templates_are_statements_of_the_language_and_offered_by_an_engine_that_has_their_functions()
{
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    for snippet in SNIPPETS {
        // Filled with the names of its places, a template parses as written, in the one style.
        let text = filled(snippet.body);
        assert!(!text.contains("${"), "{text}");
        let parsed = zeff_syntax::parse("snippet.zeff", &text);
        assert!(
            parsed.diagnostics.is_empty(),
            "{}: {:?}",
            snippet.label,
            parsed.diagnostics
        );
        assert_eq!(
            zeff_syntax::format::format("snippet.zeff", &text)
                .ok()
                .as_deref(),
            Some(text.as_str()),
            "{} is written in the style of `zeff fmt`",
            snippet.label
        );
        // The same number names the same place throughout.
        let mut places: Vec<(&str, &str)> = Vec::new();
        let mut rest = snippet.body;
        while let Some(start) = rest.find("${") {
            let inner = &rest[start + 2..];
            let end = inner.find('}').expect("a place is closed");
            let (number, name) = inner[..end].split_once(':').expect("a place has a name");
            match places.iter().find(|(n, _)| *n == number) {
                Some((_, known)) => assert_eq!(*known, name, "{}", snippet.label),
                None => places.push((number, name)),
            }
            rest = &inner[end + 1..];
        }
    }
    // An engine offers the templates whose functions it has, and no other.
    let offered: Vec<&str> = snippets(registry).iter().map(|s| s.label).collect();
    let expected: Vec<&str> = SNIPPETS
        .iter()
        .filter(|s| s.needs.iter().all(|n| !registry.named(n).is_empty()))
        .map(|s| s.label)
        .collect();
    assert_eq!(offered, expected);
    assert!(offered.contains(&"regression table"));
    assert!(offered.contains(&"difference in differences, two periods"));
    // The binary models arrived with `glm` at release 0.9.0, and their template with them.
    assert!(offered.contains(&"logistic regression"));
    assert_eq!(offered.len(), SNIPPETS.len(), "every template is offered");
}

#[test]
fn a_template_is_offered_where_a_statement_may_begin() {
    let text = "let w = dataset(wages)\nreg\nlet m = w\n    .where(year > 2018)\n    reg\nw.reg(wage ~ educ, se: rob";
    // An empty line, a word begun at the start of a line, with white space before it.
    assert!(at_statement_start("main.zeff", "", at(0, 0)));
    assert!(at_statement_start("main.zeff", text, at(1, 3)));
    assert!(at_statement_start("main.zeff", "    reg", at(0, 7)));
    // Not after a dot or inside a call, nor on a later line of a chain.
    assert!(!at_statement_start("main.zeff", text, at(5, 25)));
    assert!(!at_statement_start("main.zeff", text, at(3, 10)));
    assert!(!at_statement_start("main.zeff", "let m = re", at(0, 10)));
}

#[test]
fn on_the_sample_script_every_place_has_its_statement_and_every_fold_its_lines() {
    // The specification's sample script, with statements of every kind: each line start and
    // each line end is asked for.
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let lines: Vec<&str> = text.lines().collect();
    let all = statements_in(
        "zeff_samples.zeff",
        text,
        Range {
            start: at(0, 0),
            end: at(lines.len() as u32 + 1, 0),
        },
    );
    assert!(all.len() > 50, "{}", all.len());
    // The statements stand in order and none runs into the next.
    for pair in all.windows(2) {
        let (a, b) = (pair[0].range, pair[1].range);
        assert!(
            (a.end.line, a.end.character) <= (b.start.line, b.start.character),
            "{a:?} {b:?}"
        );
    }
    for (line, written) in lines.iter().enumerate() {
        let line = line as u32;
        for character in [0, written.encode_utf16().count() as u32] {
            let place = at(line, character);
            match statement_at("zeff_samples.zeff", text, place) {
                // The statement the place is in, or the first that begins after it.
                Some(found) => {
                    assert!(all.contains(&found), "{place:?}");
                    assert!(found.range.end.line >= line, "{place:?} {found:?}");
                    let before = all
                        .iter()
                        .filter(|s| s.range.end.line >= line)
                        .map(|s| s.range.start)
                        .next();
                    assert_eq!(before, Some(found.range.start), "{place:?}");
                }
                None => assert!(
                    all.last().is_none_or(|last| last.range.end.line < line),
                    "{place:?}"
                ),
            }
        }
    }
    let folds = folds_of("zeff_samples.zeff", text);
    assert!(folds.len() > 10, "{}", folds.len());
    for fold in &folds {
        assert!(fold.start_line < fold.end_line && (fold.end_line as usize) < lines.len());
        // A fold starts on a line with something written on it, which stays in view.
        assert!(!lines[fold.start_line as usize].trim().is_empty());
    }
    // One fold to a line.
    let mut starts: Vec<u32> = folds.iter().map(|f| f.start_line).collect();
    starts.dedup();
    assert_eq!(starts.len(), folds.len());
}

// ----- the sections of a script (open decision #178; release 0.9.0, step 28) -----

/// A script in sections: two at the top with subsections under the second, a chain over
/// lines, and a heading-like comment where it is not a heading.
const IN_SECTIONS: &str = "// Returns to schooling.

// # Sample
let w = dataset(wages)
    .where(year >= 2012)

// # Models
// ## Pooled
let m1 = w.reg(log(wage) ~ educ)
// ## Fixed effects
let m3 = w.reg(log(wage) ~ educ, fe: year)


// # Figure
w.histogram(wage)
";

/// The sections as level, title, first line and last line.
fn told(sections: &[Section]) -> Vec<(u8, &str, u32, u32)> {
    sections
        .iter()
        .map(|s| (s.level, s.title.as_str(), s.line, s.end_line))
        .collect()
}

#[test]
fn a_comment_written_as_a_heading_begins_a_section() {
    let sections = sections_of("main.zeff", IN_SECTIONS);
    // A section runs to the line before the next heading of its level or above, and holds
    // its subsections; the last runs to the script's last line.
    assert_eq!(
        told(&sections),
        [
            (1, "Sample", 2, 5),
            (1, "Models", 6, 12),
            (2, "Pooled", 7, 8),
            (2, "Fixed effects", 9, 12),
            (1, "Figure", 13, 14),
        ]
    );
    // The whole of a section, and where its title is written.
    assert_eq!(
        sections[1].range,
        Range {
            start: at(6, 0),
            end: at(12, 0)
        }
    );
    assert_eq!(
        sections[3].range,
        Range {
            start: at(9, 0),
            end: at(12, 0)
        }
    );
    assert_eq!(
        sections[3].title_range,
        Range {
            start: at(9, 6),
            end: at(9, 19)
        }
    );
    assert_eq!(
        sections[4].range,
        Range {
            start: at(13, 0),
            end: at(14, 17)
        }
    );
    // None in a script without headings, in an empty one, or in one that is a line break.
    assert!(sections_of("main.zeff", "// a comment\nprint(1)\n").is_empty());
    assert!(sections_of("main.zeff", "").is_empty());
    assert!(sections_of("main.zeff", "\n").is_empty());
    // A heading alone, with and without a line break after it, is a section of one line.
    for text in ["// # Alone", "// # Alone\n"] {
        assert_eq!(told(&sections_of("main.zeff", text)), [(1, "Alone", 0, 0)]);
    }
}

#[test]
fn a_heading_is_taken_whatever_its_spaces_and_its_title_is_kept_as_written() {
    let sections = sections_of(
        "main.zeff",
        "//# Data\n//  ##   Cleaning the wages  \n// # Models: pooled, then fixed effects (#2)\n// # Größe und Lohn 📈 zusammen\n",
    );
    assert_eq!(
        told(&sections),
        [
            (1, "Data", 0, 1),
            (2, "Cleaning the wages", 1, 1),
            (1, "Models: pooled, then fixed effects (#2)", 2, 2),
            (1, "Größe und Lohn 📈 zusammen", 3, 3),
        ]
    );
    // The title's place is counted as the editor counts, a letter outside the basic plane
    // as two: 5 before the title, then 15 letters, one of two units, and 9 more.
    assert_eq!(
        sections[3].title_range,
        Range {
            start: at(3, 5),
            end: at(3, 31)
        }
    );
    assert_eq!(sections[1].title_range.start, at(1, 9));
    // Line ends of two characters: the title has no carriage return, and the lines count
    // as they did.
    let crlf = sections_of(
        "main.zeff",
        "// # One\r\nprint(1)\r\n// ## Two\r\nprint(2)\r\n",
    );
    assert_eq!(told(&crlf), [(1, "One", 0, 3), (2, "Two", 2, 3)]);
    assert_eq!(crlf[0].range.end, at(3, 8));
}

#[test]
fn what_is_not_a_heading_begins_no_section() {
    for line in [
        "// #",                       // no title
        "// #   ",                    // no title
        "// #Data",                   // no space after the mark
        "// ### Deeper",              // three marks
        "print(1) // # After code",   // not alone on its line
        "# Data",                     // not a comment
        "/// # Data",                 // a line of help text
        "// Data #",                  // the mark at the end
        "// zeff: allow(W0201) # x",  // a pragma
        "let s = \"// # In a text\"", // inside a text
    ] {
        assert!(
            sections_of("main.zeff", &format!("{line}\nprint(1)\n")).is_empty(),
            "{line}"
        );
    }
}

#[test]
fn a_heading_inside_a_statement_is_a_comment_and_no_more() {
    // Between the arguments of a call, in the body of a loop and in a function: none of
    // these stands between two statements of the script.
    let text = "// # Real
let fit = dataset(wages).reg(log(wage) ~ educ,
// # Not a section
    se: robust)
for i in 1..2 {
    // # Nor this
// ## Nor this, at the first column
    print(i)
}
/// Twice a number.
fn twice(x: Float) -> Float {
// # Nor inside a function
    x * 2
}
// ## A real one
print(fit)
";
    assert_eq!(
        told(&sections_of("main.zeff", text)),
        [(1, "Real", 0, 15), (2, "A real one", 14, 15)]
    );
    // A heading set in from the margin between two statements is one: where it stands
    // decides, not its spaces, and the formatter sets it at the margin.
    assert_eq!(
        told(&sections_of(
            "main.zeff",
            "print(1)\n    // # Set in\nprint(2)\n"
        )),
        [(1, "Set in", 1, 2)]
    );
    // A text that does not parse: the headings between the statements still read.
    let broken = "// # Before\nlet a = (1 +\n// # Inside\n\n// # After\nprint(2)\n";
    let found = sections_of("main.zeff", broken);
    assert!(found.iter().any(|s| s.title == "Before"), "{found:?}");
}

#[test]
fn a_section_folds_from_its_heading_to_its_last_written_line() {
    let folds = folds_of("main.zeff", IN_SECTIONS);
    let fold = |start_line, end_line| Fold {
        start_line,
        end_line,
    };
    assert_eq!(
        folds,
        [
            // Sample, without the blank line before the next heading; the chain inside it.
            fold(2, 4),
            fold(3, 4),
            // Models, with its two subsections; Fixed effects, without the two blank lines.
            fold(6, 10),
            fold(7, 8),
            fold(9, 10),
            fold(13, 14),
        ]
    );
    // A heading with nothing under it, or with blank lines alone, folds nothing.
    assert!(folds_of("main.zeff", "// # One\n// # Two\n\n\n").is_empty());
}

#[test]
fn the_formatter_keeps_the_sections() {
    // Headings written with other spaces, set in from the margin, after blank lines, and one
    // that is not a heading because it stands inside a call: formatted, the script has the
    // same sections with the same titles, and each heading's text is as it was written.
    let text = "// Returns to schooling.\n\n//# Sample\nlet w = dataset(wages)\n    .where(year >= 2012)\n//  ##   Cleaning the wages  \nlet k = w.where(wage > 0)\n\n\n\n\n   // # Models\nfor i in 1..2 {\n    // # inside\n    print(i)\n}\nlet fit = k.reg(log(wage) ~ educ,\n// # Not a section\n    se: robust)\n// ## Last\n";
    let formatted = zeff_syntax::format::format("main.zeff", text).expect("the script formats");
    let levels_and_titles = |text: &str| -> Vec<(u8, String)> {
        sections_of("main.zeff", text)
            .into_iter()
            .map(|s| (s.level, s.title))
            .collect()
    };
    assert_eq!(
        levels_and_titles(text),
        [
            (1, "Sample".to_string()),
            (2, "Cleaning the wages".to_string()),
            (1, "Models".to_string()),
            (2, "Last".to_string()),
        ]
    );
    assert_eq!(levels_and_titles(&formatted), levels_and_titles(text));
    // Each heading is on a line of its own at the margin, its words as they were typed.
    for heading in [
        "//# Sample",
        "//  ##   Cleaning the wages",
        "// # Models",
        "// ## Last",
    ] {
        assert!(
            formatted.lines().any(|line| line == heading),
            "{heading}\n{formatted}"
        );
    }
    // The one inside the call is still inside it, further in.
    assert!(
        formatted.contains("\n                // # Not a section\n"),
        "{formatted}"
    );
    // The specification's sample script, as it is and formatted, has the same sections.
    let sample = include_str!("../../../docs/spec/zeff_samples.zeff");
    let formatted = zeff_syntax::format::format("zeff_samples.zeff", sample).expect("it formats");
    assert_eq!(levels_and_titles(&formatted), levels_and_titles(sample));
}

#[test]
fn the_sample_script_is_in_its_sixteen_sections() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let sections = sections_of("zeff_samples.zeff", text);
    let titles: Vec<&str> = sections.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "1. Import and clean",
            "2. Describe",
            "3. Reshape, merge, aggregate",
            "4. Panel variables",
            "5. Regression",
            "6. Loops and your own functions",
            "7. Instrumental variables",
            "8. Two-period difference-in-differences",
            "9. Classical tests",
            "10. Binary outcomes",
            "11. A small simulation, keeping going when a fit fails",
            "12. Reliability of a survey scale",
            "13. Causal graph: check the model against its assumptions",
            "14. Contrasts and nonlinear combinations",
            "15. Layered plots: the grammar behind the quick plots",
            "Previews: syntax fixed now, available from 1.x",
        ]
    );
    assert!(sections.iter().all(|s| s.level == 1));
    // The sections follow one another with nothing between them, to the last line.
    for pair in sections.windows(2) {
        assert_eq!(pair[0].end_line + 1, pair[1].line);
    }
    let lines = text.lines().count() as u32;
    assert_eq!(sections.last().unwrap().end_line, lines - 1);
    // Each section but the last, which is comments alone, holds statements of the script.
    let all = statements_in(
        "zeff_samples.zeff",
        text,
        Range {
            start: at(0, 0),
            end: at(lines + 1, 0),
        },
    );
    for section in &sections[..15] {
        let held = all
            .iter()
            .filter(|s| section.line < s.range.start.line && s.range.start.line <= section.end_line)
            .count();
        assert!(held > 0, "{}", section.title);
    }
    // Each section folds, the heading staying in view.
    let folds = folds_of("zeff_samples.zeff", text);
    for section in &sections {
        assert!(
            folds.iter().any(|f| f.start_line == section.line),
            "{}",
            section.title
        );
    }
}

proptest! {
    /// Whatever is typed, the sections stand in order, each within the text, each title
    /// where its range says, each subsection ending no later than the section it is in.
    #[test]
    fn the_sections_of_any_text_are_in_order_and_within_it(
        lines in prop::collection::vec(
            prop_oneof![
                Just("// # One".to_string()),
                Just("// ## Two ö".to_string()),
                Just("  // # Set in".to_string()),
                Just("print(1)".to_string()),
                Just("let a = f(1,".to_string()),
                Just("    2)".to_string()),
                Just("for i in 1..2 {".to_string()),
                Just("}".to_string()),
                Just("".to_string()),
                Just("// ### no".to_string()),
                "[ -~]{0,12}",
            ],
            0..24,
        ),
        ends in any::<bool>(),
    ) {
        let mut text = lines.join("\n");
        if ends {
            text.push('\n');
        }
        let sections = sections_of("main.zeff", &text);
        let count = text.lines().count() as u32;
        let written: Vec<&str> = text.split('\n').collect();
        for section in &sections {
            prop_assert!(section.line <= section.end_line);
            prop_assert!(section.end_line < count.max(1));
            prop_assert!(section.level == 1 || section.level == 2);
            prop_assert!(!section.title.is_empty());
            // The title is the text at its place.
            let line = written[section.line as usize];
            let units: Vec<u16> = line.encode_utf16().collect();
            let (from, to) = (
                section.title_range.start.character as usize,
                section.title_range.end.character as usize,
            );
            prop_assert_eq!(String::from_utf16(&units[from..to]).unwrap(), section.title.clone());
        }
        for pair in sections.windows(2) {
            prop_assert!(pair[0].line < pair[1].line);
            if pair[1].level > pair[0].level {
                prop_assert!(pair[1].end_line <= pair[0].end_line);
            } else {
                prop_assert!(pair[0].end_line < pair[1].line);
            }
        }
        // The folds are within the text.
        for fold in folds_of("main.zeff", &text) {
            prop_assert!(fold.start_line < fold.end_line && fold.end_line < count.max(1));
        }
    }
}

proptest! {
    /// The formatter keeps the sections: a script put together of headings in several
    /// spellings, statements over one line and over several, a loop, a function with its
    /// help line and blank lines has, formatted, the sections it had.
    #[test]
    fn a_formatted_script_has_the_sections_it_had(
        pieces in prop::collection::vec(
            prop_oneof![
                Just("// # One"),
                Just("//# One and a half"),
                Just("//   ##   Two  "),
                Just("   // # Set in"),
                Just("\t// ## Set in by a tab"),
                Just("// ### not one"),
                Just("// a comment"),
                Just("print(1)"),
                Just("print(2) // # after code"),
                Just("let a = max(1,\n// # between arguments\n    2)"),
                Just("let b = [1,\n    // ## in a list\n    2]"),
                Just("for i in 1..2 {\n// # in a loop\n    print(i)\n}"),
                Just("/// Twice a number.\nfn twice(x: Float) -> Float {\n    // ## in a function\n    x * 2\n}"),
                Just(""),
                Just("\n\n"),
            ],
            0..16,
        ),
    ) {
        let text = pieces.join("\n") + "\n";
        let formatted = zeff_syntax::format::format("main.zeff", &text);
        prop_assert!(formatted.is_ok(), "{}", text);
        let formatted = formatted.unwrap();
        let of = |text: &str| -> Vec<(u8, String)> {
            sections_of("main.zeff", text)
                .into_iter()
                .map(|s| (s.level, s.title))
                .collect()
        };
        prop_assert_eq!(of(&formatted), of(&text), "{}\n---\n{}", text, formatted);
    }
}
