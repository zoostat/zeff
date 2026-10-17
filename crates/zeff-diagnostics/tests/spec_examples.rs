//! The renderer against the specification itself: every printed error in the Language Reference
//! (the ten examples of sections 3.6 and 11) is read from the specification file, and the same
//! diagnostic, built here as data, must print as exactly that text. The source lines are taken
//! from the example's own numbered lines.

use zeff_diagnostics::{Diagnostic, Source, Sources, Span};

const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
const FILE: &str = "scripts/main.zeff";

/// The fenced blocks of the specification that start with `error[Z`.
fn error_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&str>> = None;
    for line in SPEC.lines() {
        if line.starts_with("```") {
            match current.take() {
                Some(lines) => {
                    if lines.first().is_some_and(|l| l.starts_with("error[Z")) {
                        blocks.push(lines.join("\n") + "\n");
                    }
                }
                None => current = Some(Vec::new()),
            }
        } else if let Some(lines) = current.as_mut() {
            lines.push(line);
        }
    }
    blocks
}

/// The source text implied by the numbered lines of an example (`7 | let m = ...`).
fn source_of(block: &str) -> Source {
    let mut lines: Vec<String> = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim_start();
        let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() || !trimmed[digits.len()..].starts_with(" | ") {
            continue;
        }
        // Line numbers in the examples may carry a thousands comma; none of these lines does.
        let number: usize = digits.parse().unwrap();
        let text = &trimmed[digits.len() + 3..];
        if lines.len() < number {
            lines.resize(number, String::new());
        }
        lines[number - 1] = text.to_string();
    }
    Source::new(FILE, lines.join("\n") + "\n")
}

/// The span of the first `needle` on a 1-based line.
fn at(source: &Source, line: u32, needle: &str) -> Span {
    let text = source.line_text(line);
    let found = text
        .find(needle)
        .unwrap_or_else(|| panic!("`{needle}` is not on line {line}: {text}"));
    let start = source.line_start(line) + found;
    Span::new(start, start + needle.len())
}

fn diagnostic(code: &str, s: &Source) -> Diagnostic {
    let d = |message: &str| Diagnostic::error(code, message).in_file(FILE);
    match code {
        "Z0101" => d("no variable `incme` in `poll`")
            .primary(at(s, 4, "incme"), "not a variable of poll")
            .help("did you mean `income`? If you meant a category level, write it in quotes: `\"income\"`"),
        "Z0102" => d("no variable `Wage` in `wages`; names are case-sensitive")
            .primary(at(s, 6, "Wage"), "differs only in case from `wage`")
            .help("write `wage`"),
        "Z0203" => d("`if` needs one Bool, but `wage > 20` is a Bool column")
            .primary(at(s, 12, "wage > 20"), "a column: one value per row")
            .help("for row-by-row values use `add(high = case { wage > 20 => 1, _ => 0 })`"),
        "Z0302" => d("this `(` is never closed")
            .primary(at(s, 7, "("), "opened here")
            .secondary(at(s, 8, "report"), "the statement seems to end before this line"),
        "Z0402" => d("`robus` is not a value of `se:`")
            .primary(at(s, 9, "robus"), "expected one of classical, robust, hc2, hc3 (bootstrap from 1.1)")
            .help("did you mean `robust`?"),
        "Z0504" => d("`educ` is instrumented but is not a regressor")
            .primary(at(s, 15, "educ ~ nearc4").shrink_to(4), "instrumented here, missing from the formula")
            .help("add it to the regressors: `lwage ~ exper + educ`"),
        "Z0603" => d("many-to-many merge refused")
            .primary(at(s, 21, "merge"), "firm_id repeats in both frames (412 keys on the left, 37 on the right)")
            .help("find the repeated keys with `firms.duplicates(firm_id)` and remove them, then state `relation: many_to_one`"),
        "Z0606" => d("row 14,207 of `data/wages.csv` has text in a numeric column")
            .primary(at(s, 5, "import"), "column `hours` read as Float from the first 1,000 rows; row 14,207 holds \"varies\"")
            .help("declare it with `types: {hours: Str}`, add \"varies\" to `na_values:`, or fix the file"),
        "Z0701" => d("path `../shared/wages.csv` is outside the project")
            .primary(at(s, 2, "\"../shared/wages.csv\""), "projects read only their own files and declared data roots")
            .help("declare the folder under [data] in zeff.toml and write `import(\"shared:wages.csv\")`, or copy the file into data/"),
        "Z0803" => d("logistic: `female` perfectly predicts `union` (separation)")
            .primary(at(s, 49, "logistic"), "estimates diverge; maximum likelihood does not exist")
            .help("drop or combine the predictor, or check how `union` was coded"),
        other => panic!("the specification has an example for {other} that this test does not build"),
    }
}

trait Shrink {
    fn shrink_to(self, length: usize) -> Span;
}

impl Shrink for Span {
    fn shrink_to(self, length: usize) -> Span {
        Span::new(self.start, self.start + length)
    }
}

fn code_of(block: &str) -> &str {
    &block[6..11]
}

#[test]
fn the_specification_has_the_ten_examples() {
    let blocks = error_blocks();
    let mut codes: Vec<&str> = blocks.iter().map(|b| code_of(b)).collect();
    codes.sort_unstable();
    assert_eq!(
        codes,
        [
            "Z0101", "Z0102", "Z0203", "Z0302", "Z0402", "Z0504", "Z0603", "Z0606", "Z0701",
            "Z0803"
        ]
    );
}

#[test]
fn every_printed_error_of_the_specification_is_reproduced_exactly() {
    for block in error_blocks() {
        let code = code_of(&block);
        let source = source_of(&block);
        let mut sources = Sources::new();
        sources.add(source.clone());
        let printed = diagnostic(code, &source).render(&sources);
        assert_eq!(printed, block, "the example of {code} is not reproduced");
    }
}
