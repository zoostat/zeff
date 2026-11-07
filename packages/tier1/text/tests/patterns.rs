//! The pattern functions of release 0.7.0, step 6 (Language Reference, section 3.8, "Text"):
//! `contains`, `replace`, `extract` and `split` against R's `grepl`, `gsub`, `regmatches` and
//! `strsplit` with `perl = TRUE` on patterns both engines read alike
//! (`tests/reference/r/text_regex.json`), exactly; the edge cases (a pattern that cannot be read,
//! look-around and a back-reference refused by name, an empty match, a trailing separator, an
//! empty literal `from`, `NA`); the column forms; every message; and the properties that
//! splitting and joining round-trip, that replacing a text by itself changes nothing and that a
//! text contains every slice of itself.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn printed(script: &str) -> Vec<String> {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    assert_eq!(stderr, "", "{script}");
    stdout.lines().map(str::to_string).collect()
}

fn error_code(script: &str) -> String {
    let (code, _, stderr) = go(script);
    assert_ne!(code, 0, "{script} should fail");
    stderr
        .lines()
        .next()
        .unwrap_or_default()
        .trim_start_matches("error[")
        .split(']')
        .next()
        .unwrap_or_default()
        .to_string()
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/text_regex.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff string literal for any text, every character outside printable ASCII and every
/// backslash or quote escaped, so that a pattern reads in Zeff as R read it.
fn literal(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ' '..='~' => out.push(c),
            _ => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
        }
    }
    out.push('"');
    out
}

fn text_of(value: &Json) -> &str {
    value.as_str().unwrap()
}

// ----- against R -----

#[test]
fn contains_agrees_with_r() {
    // Many cases, so one script prints them all.
    let cases = reference()["contains"].as_array().unwrap().clone();
    let script: String = cases
        .iter()
        .map(|c| {
            format!(
                "contains({}, {}, regex: {}, ignore_case: {})\n",
                literal(text_of(&c["text"])),
                literal(text_of(&c["pattern"])),
                c["regex"],
                c["ignore"]
            )
        })
        .collect();
    let lines = printed(&script);
    assert_eq!(lines.len(), cases.len());
    for (line, case) in lines.iter().zip(&cases) {
        assert_eq!(
            line,
            &case["out"].to_string(),
            "{} {} {} {}",
            case["text"],
            case["pattern"],
            case["regex"],
            case["ignore"]
        );
    }
}

#[test]
fn replace_agrees_with_r() {
    for case in reference()["replaces"].as_array().unwrap() {
        let lines = printed(&format!(
            "print(equal(replace({}, {}, {}, regex: {}), {}))\n",
            literal(text_of(&case["text"])),
            literal(text_of(&case["from"])),
            literal(text_of(&case["to"])),
            case["regex"],
            literal(text_of(&case["out"]))
        ));
        assert_eq!(
            lines,
            vec!["true"],
            "{} {} {}",
            case["text"],
            case["from"],
            case["to"]
        );
    }
}

#[test]
fn extract_agrees_with_r() {
    for case in reference()["extracts"].as_array().unwrap() {
        let call = format!(
            "extract({}, {})",
            literal(text_of(&case["text"])),
            literal(text_of(&case["pattern"]))
        );
        let lines = if case["found"].as_bool().unwrap() {
            printed(&format!(
                "print(equal({call}, {}))\n",
                literal(text_of(&case["out"]))
            ))
        } else {
            printed(&format!("print(is_missing({call}))\n"))
        };
        assert_eq!(lines, vec!["true"], "{call}");
    }
}

#[test]
fn split_agrees_with_r() {
    for case in reference()["splits"].as_array().unwrap() {
        let pieces: Vec<String> = case["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| literal(text_of(p)))
            .collect();
        let lines = printed(&format!(
            "print(equal(split({}, {}), [{}]))\n",
            literal(text_of(&case["text"])),
            literal(text_of(&case["sep"])),
            pieces.join(", ")
        ));
        assert_eq!(lines, vec!["true"], "{} {}", case["text"], case["sep"]);
    }
}

// ----- edge cases -----

#[test]
fn na_gives_na() {
    let lines = printed(
        "let na = to(NA, Str)\ncontains(na, \"a\")\ncontains(\"a\", na)\nreplace(na, \"a\", \"b\")\nreplace(\"a\", na, \"b\")\nreplace(\"a\", \"a\", na)\nextract(na, \"a\")\nextract(\"a\", na)\nsplit(na, \",\")\nsplit(\"a,b\", na)\n",
    );
    assert_eq!(
        lines,
        vec!["NA", "NA", "NA", "NA", "NA", "NA", "NA", "NA", "NA"]
    );
}

#[test]
fn a_pattern_that_cannot_be_read_is_refused_by_name() {
    assert_eq!(
        error_code("contains(\"abc\", \"(ab\", regex: true)\n"),
        "Z0419"
    );
    assert_eq!(error_code("extract(\"abc\", \"*a\")\n"), "Z0419");
    assert_eq!(error_code("split(\"abc\", \"[a\")\n"), "Z0419");
    let (_, _, err) = go("contains(\"abc\", \"a(?=b)\", regex: true)\n");
    assert!(err.contains("look-around"), "{err}");
    let (_, _, err) = go("replace(\"abab\", \"(a)\\\\1\", \"x\", regex: true)\n");
    assert!(err.contains("back-reference"), "{err}");
    // A bad pattern is refused even where the text is NA, and a literal one is never read.
    assert_eq!(
        error_code("contains(to(NA, Str), \"(ab\", regex: true)\n"),
        "Z0419"
    );
    assert_eq!(printed("contains(\"x(ab\", \"(ab\")\n"), vec!["true"]);
}

#[test]
fn empty_matches_and_trailing_separators() {
    let lines = printed(
        "replace(\"abc\", \"\", \"-\", regex: true)\nreplace(\"abc\", \"x*\", \"-\", regex: true)\nprint(len(split(\"a,b,\", \",\")))\nprint(len(split(\",a\", \",\")))\nsplit(\"abc\", \"\")\nprint(len(split(\"\", \",\")))\nprint(len(extract(\"abc\", \"x*\")))\n",
    );
    assert_eq!(
        lines,
        vec!["-a-b-c-", "-a-b-c-", "3", "2", "a", "b", "c", "1", "0"]
    );
    assert_eq!(error_code("replace(\"abc\", \"\", \"-\")\n"), "Z0401");
}

#[test]
fn case_folding_is_simple() {
    let lines = printed(
        "contains(\"Stra\\u{df}e\", \"STRASSE\", ignore_case: true)\ncontains(\"Stra\\u{df}e\", \"stra\\u{df}e\", ignore_case: true)\ncontains(\"\\u{3a3}\\u{3af}\\u{3c3}\\u{3c5}\\u{3c6}\\u{3bf}\\u{3c2}\", \"\\u{3c3}\\u{3af}\\u{3a3}\", ignore_case: true)\ncontains(\"a.b\", \".\")\ncontains(\"ab\", \".\")\ncontains(\"ab\", \".\", regex: true)\n",
    );
    // `ß` folds to itself under simple folding, not to `ss`; a final sigma folds with the others.
    assert_eq!(
        lines,
        vec!["false", "true", "true", "true", "false", "true"]
    );
}

// ----- columns -----

#[test]
fn the_scalar_functions_apply_row_by_row() {
    let lines = printed(
        "let d = frame(code = [\"ab-1\", \"CD-22\", NA, \"ef-333\"])\nlet e = d.add(digits = extract(code, \"[0-9]+\"), letters = replace(code, \"[^a-z]\", \"\", regex: true), has_two = contains(code, \"2\"), cd = contains(code, \"cd\", ignore_case: true))\ne[\"digits\"]\ne[\"letters\"]\ne[\"has_two\"]\ne[\"cd\"]\nd.where(!is_missing(code) && contains(code, \"^[a-z]+-\\\\d$\", regex: true)).n_rows()\n",
    );
    assert_eq!(
        lines,
        vec![
            "Column<Str> · 4 values",
            "1",
            "22",
            "NA",
            "333",
            "Column<Str> · 4 values",
            "ab",
            "",
            "NA",
            "ef",
            "Column<Bool> · 4 values",
            "false",
            "true",
            "NA",
            "false",
            "Column<Bool> · 4 values",
            "false",
            "true",
            "NA",
            "false",
            "1",
        ]
    );
}

#[test]
fn split_takes_one_text() {
    let (code, _, err) = go("let d = frame(s = [\"a,b\", \"c\"])\nsplit(d[\"s\"], \",\")\n");
    assert_ne!(code, 0, "{err}");
    let lines = printed(
        "print(len([for v in [\"a,b\", \"c\"] { split(v, \",\") }]))\nprint(len([for v in [\"a,b\", \"c\"] { split(v, \",\") }][1]))\n",
    );
    assert_eq!(lines, vec!["2", "2"]);
}

// ----- every message -----

#[test]
fn every_message() {
    let scripts = [
        "contains(\"abc\", \"(ab\", regex: true)\n",
        "contains(\"abc\", \"a(?=b)\", regex: true)\n",
        "replace(\"abab\", \"(a)\\\\1\", \"x\", regex: true)\n",
        "extract(\"abc\", \"*a\")\n",
        "split(\"abc\", \"[a\")\n",
        "replace(\"abc\", \"\", \"-\")\n",
        "contains(\"abc\")\n",
        "contains(3, \"a\")\n",
        "split(frame(s = [\"a\"])[\"s\"], \",\")\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Splitting on a literal separator and joining with it gives the text back; replacing a
    /// text by itself changes nothing; a text contains every slice of itself.
    #[test]
    fn round_trips_and_identities(s in "[a-c,]{0,10}", start in 1usize..4, len in 0usize..4) {
        let lit = literal(&s);
        let lines = printed(&format!(
            "print(equal(join(split({lit}, \",\"), sep: \",\"), {lit}))\nprint(equal(replace({lit}, \"a\", \"a\"), {lit}))\nprint(contains({lit}, slice({lit}, start: {start}, len: {len})))\nprint(equal(replace({lit}, \"[a-c]\", \"\", regex: true), replace(replace(replace({lit}, \"a\", \"\"), \"b\", \"\"), \"c\", \"\")))\n"
        ));
        prop_assert_eq!(&lines, &vec!["true".to_string(), "true".to_string(), "true".to_string(), "true".to_string()]);
    }
}
