//! Pragmas (Language Reference, section 11.1, "Deliberate exceptions"; release 0.7.0, step 2):
//! what `// zeff: allow(CODE) reason` says, the statement it belongs to (above it, or at the end
//! of its first line, at any depth), every way it can be wrong as a snapshot (Z0317), and the
//! edge cases: a pragma on a continuation line, two pragmas on one statement, a pragma with
//! nothing below it, and a file that does not parse.

use zeff_diagnostics::{Source, Sources};
use zeff_syntax::{Allow, parse};

const FILE: &str = "scripts/main.zeff";

fn allows(text: &str) -> Vec<Allow> {
    let parsed = parse(FILE, text);
    assert!(parsed.diagnostics.is_empty(), "{}", render(text));
    parsed.allows
}

/// The code, the reason and the text of the statement of each pragma.
fn placed(text: &str) -> Vec<(String, String, String)> {
    allows(text)
        .into_iter()
        .map(|a| {
            (
                a.code,
                a.reason,
                text[a.statement.start..a.statement.end].to_string(),
            )
        })
        .collect()
}

fn render(text: &str) -> String {
    let parsed = parse(FILE, text);
    let mut sources = Sources::new();
    sources.add(Source::new(FILE, text));
    parsed
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

fn codes(text: &str) -> Vec<String> {
    parse(FILE, text)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

fn one(code: &str, reason: &str, statement: &str) -> Vec<(String, String, String)> {
    vec![(code.to_string(), reason.to_string(), statement.to_string())]
}

// ----- what a pragma says -----

#[test]
fn the_example_of_section_11_1_is_read() {
    let text = "// zeff: allow(W0203) \"varies\" coded as NA by design\nlet age = to(raw, Float)\n";
    assert_eq!(
        placed(text),
        one(
            "W0203",
            "\"varies\" coded as NA by design",
            "let age = to(raw, Float)"
        )
    );
    let allow = &allows(text)[0];
    assert_eq!(&text[allow.code_span.start..allow.code_span.end], "W0203");
    assert_eq!(
        &text[allow.span.start..allow.span.end],
        "// zeff: allow(W0203) \"varies\" coded as NA by design"
    );
}

#[test]
fn spaces_in_a_pragma_do_not_matter() {
    for text in [
        "//zeff:allow(W0203)by design\nlet a = 1\n",
        "//   zeff:   allow ( W0203 )   by design  \nlet a = 1\n",
        "// zeff: allow(W0203) by design\r\nlet a = 1\r\n",
    ] {
        assert_eq!(
            placed(text),
            one("W0203", "by design", "let a = 1"),
            "{text:?}"
        );
    }
}

#[test]
fn an_ordinary_comment_is_not_a_pragma() {
    for text in [
        "// zeffy: allow(W0203) x\nlet a = 1\n",
        "// see zeff: allow(W0203) x\nlet a = 1\n",
        "/// zeff: allow(W0203) x\nfn f() -> Int { 1 }\n",
    ] {
        assert!(allows(text).is_empty(), "{text:?}");
    }
}

// ----- the statement a pragma belongs to -----

#[test]
fn a_pragma_above_a_statement_covers_its_continuation_lines() {
    let text = "let w = 1\n// zeff: allow(W0204) zeros are real\nlet z = w\n    .add(a = 1)\n    .where(a > 0)\nlet y = 2\n";
    assert_eq!(
        placed(text),
        one(
            "W0204",
            "zeros are real",
            "let z = w\n    .add(a = 1)\n    .where(a > 0)"
        )
    );
}

#[test]
fn a_pragma_at_the_end_of_the_first_line_covers_the_statement() {
    let text = "let z = w // zeff: allow(W0204) zeros are real\n    .add(a = 1)\nlet y = 2\n";
    assert_eq!(
        placed(text),
        one(
            "W0204",
            "zeros are real",
            "let z = w // zeff: allow(W0204) zeros are real\n    .add(a = 1)"
        )
    );
}

#[test]
fn blank_lines_and_comments_may_stand_between() {
    let text = "// zeff: allow(W0204) zeros are real\n\n// the logs\nlet z = log(w)\n";
    assert_eq!(
        placed(text),
        one("W0204", "zeros are real", "let z = log(w)")
    );
}

#[test]
fn two_pragmas_stand_on_one_statement() {
    let text = "// zeff: allow(W0204) zeros\n// zeff: allow(W0205) negatives\nlet z = log(w) // zeff: allow(W0201) and missing values\n";
    let found = placed(text);
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|(_, _, s)| s == "let z = log(w)"));
    assert_eq!(
        found.iter().map(|(c, _, _)| c.as_str()).collect::<Vec<_>>(),
        ["W0204", "W0205", "W0201"]
    );
}

#[test]
fn a_pragma_inside_a_block_belongs_to_the_statement_there() {
    // In a loop, in a function, and on the value of a block.
    let text = "for i in [1, 2] {\n    // zeff: allow(W0204) in a loop\n    let e = log(i)\n    print(e)\n}\n/// Logs.\nfn logs(x: Float) -> Float {\n    let y = x // zeff: allow(W0205) in a function\n    // zeff: allow(W0204) on the value\n    log(y)\n}\n";
    assert_eq!(
        placed(text),
        vec![
            (
                "W0204".to_string(),
                "in a loop".to_string(),
                "let e = log(i)".to_string()
            ),
            (
                "W0205".to_string(),
                "in a function".to_string(),
                "let y = x".to_string()
            ),
            (
                "W0204".to_string(),
                "on the value".to_string(),
                "log(y)".to_string()
            ),
        ]
    );
}

#[test]
fn a_pragma_on_a_line_that_opens_a_block_belongs_to_the_outer_statement() {
    let text = "for i in [1, 2] { // zeff: allow(W0204) the whole loop\n    print(log(i))\n}\n";
    let found = placed(text);
    assert_eq!(found.len(), 1);
    assert!(found[0].2.starts_with("for i in") && found[0].2.ends_with('}'));
    // On one line the outermost statement takes it.
    let text = "for i in [1, 2] { print(log(i)) } // zeff: allow(W0204) the whole loop\n";
    assert_eq!(placed(text)[0].2, "for i in [1, 2] { print(log(i)) }");
}

#[test]
fn a_pragma_above_a_function_covers_it_with_its_help() {
    let text = "// zeff: allow(W0204) zeros are real\n/// Logs.\nfn logs(x: Float) -> Float {\n    log(x)\n}\n";
    let found = placed(text);
    assert_eq!(found.len(), 1);
    assert!(found[0].2.starts_with("/// Logs.") && found[0].2.ends_with('}'));
    let text = "// zeff: allow(W0303) a helper\nfn logs(x: Float) -> Float { log(x) }\n";
    assert_eq!(placed(text)[0].2, "fn logs(x: Float) -> Float { log(x) }");
}

#[test]
fn a_pragma_above_a_use_line_belongs_to_it() {
    let text = "// zeff: allow(W0101) kept for the next script\nuse clean\nlet a = 1\n";
    assert_eq!(
        placed(text),
        one("W0101", "kept for the next script", "use clean")
    );
}

// ----- Z0317 -----

#[test]
fn a_pragma_on_a_continuation_line_belongs_to_no_statement() {
    for text in [
        "let z = w\n    .add(a = 1) // zeff: allow(W0204) zeros\n",
        "let z = f(\n    // zeff: allow(W0204) zeros\n    w,\n)\n",
        "for i in [1, 2] {\n    print(i)\n} // zeff: allow(W0204) zeros\n",
    ] {
        assert_eq!(codes(text), ["Z0317"], "{text:?}");
        assert!(parse(FILE, text).allows.is_empty());
    }
}

#[test]
fn a_pragma_with_nothing_below_it_belongs_to_no_statement() {
    for text in [
        "let a = 1\n// zeff: allow(W0204) zeros\n",
        "// zeff: allow(W0204) zeros\n",
        "for i in [1, 2] {\n    print(i)\n    // zeff: allow(W0204) zeros\n}\n",
    ] {
        assert_eq!(codes(text), ["Z0317"], "{text:?}");
    }
}

#[test]
fn a_file_that_does_not_parse_reports_only_what_the_pragma_says() {
    // The statement below the pragma is broken: the syntax error is the one to put right.
    let text = "// zeff: allow(W0204) zeros\nlet = 1\n";
    let found = codes(text);
    assert!(
        !found.is_empty() && !found.contains(&"Z0317".to_string()),
        "{found:?}"
    );
    // What the pragma says is still checked.
    let text = "// zeff: allow(W0204)\nlet = 1\n";
    assert!(codes(text).contains(&"Z0317".to_string()));
}

#[test]
fn every_message_of_pragmas() {
    let cases: &[(&str, &str)] = &[
        ("pragma_no_reason", "// zeff: allow(W0203)\nlet a = 1\n"),
        (
            "pragma_error_code",
            "let a = 1 // zeff: allow(Z0101) it is fine\n",
        ),
        (
            "pragma_not_a_code",
            "// zeff: allow(skipped) it is fine\nlet a = 1\n",
        ),
        (
            "pragma_lower_case_code",
            "// zeff: allow(w0203) it is fine\nlet a = 1\n",
        ),
        (
            "pragma_two_codes",
            "// zeff: allow(W0203, W0204) both\nlet a = 1\n",
        ),
        ("pragma_no_code", "// zeff: allow() nothing\nlet a = 1\n"),
        (
            "pragma_no_parentheses",
            "// zeff: allow W0203 by design\nlet a = 1\n",
        ),
        (
            "pragma_not_closed",
            "// zeff: allow(W0203 by design\nlet a = 1\n",
        ),
        (
            "pragma_unknown",
            "// zeff: deny(W0203) by design\nlet a = 1\n",
        ),
        ("pragma_empty", "// zeff:\nlet a = 1\n"),
        (
            "pragma_on_a_continuation_line",
            "let z = w\n    .add(a = 1) // zeff: allow(W0204) zeros are real\n",
        ),
        (
            "pragma_with_nothing_below",
            "let a = 1\n// zeff: allow(W0204) zeros are real\n",
        ),
    ];
    for (name, text) in cases {
        assert_eq!(codes(text), ["Z0317"], "{name}");
        insta::assert_snapshot!(*name, render(text));
    }
}
