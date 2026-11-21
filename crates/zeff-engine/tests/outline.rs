//! The outline of a document (Engine and Project Specification, section 12; open decision #86,
//! release 0.8.0, step 8b): where it folds, the statement at a place and those a selection
//! touches, and the templates of common analyses.

use zeff_engine::language::outline::{
    SNIPPETS, StatementKind, at_statement_start, filled, folds_of, snippets, statement_at,
    statements_in,
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
