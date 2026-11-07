//! The text functions of release 0.7.0, step 5 (Language Reference, section 3.8, "Text"):
//! `upper`, `lower`, `trim`, `starts_with`, `ends_with`, `slice`, `pad` and `join` against R's
//! `toupper`, `tolower`, `trimws`, `startsWith`, `endsWith`, `substr`, `formatC` and `paste`
//! on accented, combining and non-Latin text (`tests/reference/r/text.json`), exactly; the edge
//! cases (`NA`, the empty text, Unicode white space, a slice past the end, a fill of two
//! characters, a list that is not text); the column forms; every message; and the properties
//! that the length never changes under `upper` and `lower`, that `trim` is idempotent and that
//! a slice of the whole is the whole.

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

/// The lines a script prints, which must run clean.
fn printed(script: &str) -> Vec<String> {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    assert_eq!(stderr, "", "{script}");
    stdout.lines().map(str::to_string).collect()
}

/// The code of the first error a script stops at.
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
    let path = root().join("tests/reference/r/text.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff string literal for any text: every character outside printable ASCII written as a
/// `\u{}` escape, so that the script file holds plain ASCII whatever the text.
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
fn case_trim_and_length_agree_with_r() {
    for case in reference()["cases"].as_array().unwrap() {
        let s = literal(text_of(&case["text"]));
        // Each result is wrapped so that a text with line breaks prints on one line.
        let lines = printed(&format!(
            "print(len(upper({s})))\nprint(equal(upper({s}), {}))\nprint(equal(lower({s}), {}))\nprint(equal(trim({s}), {}))\nprint(len({s}))\n",
            literal(text_of(&case["upper"])),
            literal(text_of(&case["lower"])),
            literal(text_of(&case["trimmed"])),
        ));
        let len = case["len"].as_i64().unwrap().to_string();
        assert_eq!(
            lines,
            vec![
                len.clone(),
                "true".into(),
                "true".into(),
                "true".into(),
                len
            ],
            "{s}"
        );
    }
}

#[test]
fn prefixes_and_suffixes_agree_with_r() {
    for case in reference()["affixes"].as_array().unwrap() {
        let s = literal(text_of(&case["text"]));
        let affix = literal(text_of(&case["affix"]));
        let lines = printed(&format!(
            "starts_with({s}, {affix})\nends_with({s}, {affix})\n"
        ));
        assert_eq!(lines[0], case["starts"].to_string(), "{s} {affix}");
        assert_eq!(lines[1], case["ends"].to_string(), "{s} {affix}");
    }
}

#[test]
fn slices_agree_with_r() {
    for case in reference()["slices"].as_array().unwrap() {
        let s = literal(text_of(&case["text"]));
        let (start, len) = (
            case["start"].as_i64().unwrap(),
            case["len"].as_i64().unwrap(),
        );
        let lines = printed(&format!(
            "print(equal(slice({s}, start: {start}, len: {len}), {}))\n",
            literal(text_of(&case["out"]))
        ));
        assert_eq!(lines, vec!["true"], "{s} {start} {len}");
    }
}

#[test]
fn padding_agrees_with_r() {
    for case in reference()["pads"].as_array().unwrap() {
        let s = literal(text_of(&case["text"]));
        let width = case["width"].as_i64().unwrap();
        let lines = printed(&format!(
            "print(equal(pad({s}, width: {width}), {}))\n",
            literal(text_of(&case["out"]))
        ));
        assert_eq!(lines, vec!["true"], "{s} {width}");
    }
}

#[test]
fn joining_agrees_with_r() {
    for case in reference()["joins"].as_array().unwrap() {
        let items: Vec<String> = case["items"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["na"].as_array().unwrap())
            .map(|(item, na)| {
                if na.as_bool().unwrap() {
                    "NA".to_string()
                } else {
                    literal(text_of(item))
                }
            })
            .collect();
        // An empty list takes its type from the parameter.
        let list = format!("[{}]", items.join(", "));
        let sep = literal(text_of(&case["sep"]));
        let lines = printed(&format!(
            "print(equal(join({list}, sep: {sep}), {}))\n",
            literal(text_of(&case["out"]))
        ));
        assert_eq!(lines, vec!["true"], "{list} {sep}");
    }
}

// ----- edge cases -----

#[test]
fn na_and_the_empty_text() {
    let lines = printed(
        "let na = to(NA, Str)\nupper(na)\nlower(na)\ntrim(na)\nstarts_with(na, \"a\")\nstarts_with(\"a\", na)\nends_with(na, \"\")\nslice(na, start: 1, len: 1)\npad(na, width: 3)\nprint(len(upper(\"\")))\nprint(len(trim(\"\")))\nstarts_with(\"\", \"\")\nends_with(\"\", \"a\")\nprint(len(slice(\"\", start: 1, len: 3)))\nprint(len(pad(\"\", width: 2)))\nprint(len(join([], sep: \", \")))\n",
    );
    assert_eq!(
        lines,
        vec![
            "NA", "NA", "NA", "NA", "NA", "NA", "NA", "NA", "0", "0", "true", "false", "0", "2",
            "0"
        ]
    );
}

#[test]
fn trim_removes_unicode_white_space_and_keeps_the_inside() {
    let lines = printed(
        "print(len(trim(\"\\u{a0}\\u{2003}x y\\t\\n\\u{3000}\")))\nprint(equal(trim(\"\\u{a0}\\u{2003}x y\\t\\n\\u{3000}\"), \"x y\"))\nprint(equal(trim(\"  a  b  \"), \"a  b\"))\n",
    );
    assert_eq!(lines, vec!["3", "true", "true"]);
}

#[test]
fn case_mapping_is_one_scalar_value_to_one() {
    // `ß` and `ﬁ` have longer full upper-case forms and are kept; a combining mark stays after
    // its letter; a final sigma lowers like any sigma.
    let lines = printed(
        "print(equal(upper(\"Stra\\u{df}e \\u{fb01}\"), \"STRA\\u{df}E \\u{fb01}\"))\nprint(equal(upper(\"e\\u{301}\"), \"E\\u{301}\"))\nprint(equal(lower(\"\\u{3a3}\\u{3a3}\"), \"\\u{3c3}\\u{3c3}\"))\nprint(len(upper(\"\\u{df}\\u{fb01}\\u{149}\")))\n",
    );
    assert_eq!(lines, vec!["true", "true", "true", "3"]);
}

#[test]
fn a_slice_past_the_end_gives_what_is_there() {
    let lines = printed(
        "slice(\"abc\", start: 2, len: 10)\nprint(len(slice(\"abc\", start: 4, len: 1)))\nslice(\"\\u{1f1ec}\\u{1f1e7}x\", start: 3, len: 1)\nprint(len(slice(\"abc\", start: 1, len: 0)))\n",
    );
    assert_eq!(lines, vec!["bc", "0", "x", "0"]);
}

#[test]
fn padding_counts_scalar_values_and_never_cuts() {
    let lines = printed(
        "pad(\"caf\\u{e9}\", width: 6, fill: \".\")\nprint(len(pad(\"\\u{65e5}\\u{672c}\", width: 4)))\npad(\"abcdef\", width: 3)\npad(\"7\", width: 3, fill: \"0\")\nprint(len(pad(\"x\", width: 0)))\n",
    );
    assert_eq!(lines, vec!["..café", "4", "abcdef", "007", "1"]);
}

#[test]
fn the_refusals() {
    assert_eq!(error_code("slice(\"abc\", start: 0, len: 1)\n"), "Z0401");
    assert_eq!(error_code("slice(\"abc\", start: 1, len: -1)\n"), "Z0401");
    assert_eq!(error_code("slice(\"abc\", start: 1)\n"), "Z0406");
    assert_eq!(error_code("pad(\"abc\")\n"), "Z0406");
    assert_eq!(error_code("pad(\"abc\", width: -1)\n"), "Z0401");
    assert_eq!(
        error_code("pad(\"abc\", width: 5, fill: \"ab\")\n"),
        "Z0401"
    );
    assert_eq!(error_code("pad(\"abc\", width: 5, fill: \"\")\n"), "Z0401");
    assert_eq!(error_code("join([1, 2], sep: \", \")\n"), "Z0401");
    assert_eq!(error_code("upper(3)\n"), "Z0401");
}

// ----- columns -----

#[test]
fn the_functions_apply_row_by_row_to_a_text_column() {
    let lines = printed(
        "let d = frame(name = [\"  Ada \", \"grace\", NA], code = [\"ab-1\", \"cd-22\", \"ef-3\"])\nlet e = d.add(clean = upper(trim(name)), prefix = slice(code, start: 1, len: 2), two = ends_with(code, \"2\"), padded = pad(code, width: 6, fill: \".\"))\ne[\"clean\"]\ne[\"prefix\"]\ne[\"two\"]\ne[\"padded\"]\nd.where(starts_with(code, \"c\")).n_rows()\nlower(d[\"name\"])\n",
    );
    assert_eq!(
        lines,
        vec![
            "Column<Str> · 3 values",
            "ADA",
            "GRACE",
            "NA",
            "Column<Str> · 3 values",
            "ab",
            "cd",
            "ef",
            "Column<Bool> · 3 values",
            "false",
            "true",
            "false",
            "Column<Str> · 3 values",
            "..ab-1",
            ".cd-22",
            "..ef-3",
            "1",
            "Column<Str> · 3 values",
            "  ada ",
            "grace",
            "NA",
        ]
    );
}

// ----- every message -----

#[test]
fn every_message() {
    let scripts = [
        "slice(\"abc\", start: 0, len: 1)\n",
        "slice(\"abc\", start: 1, len: -1)\n",
        "slice(\"abc\", start: 1)\n",
        "pad(\"abc\")\n",
        "pad(\"abc\", width: -1)\n",
        "pad(\"abc\", width: 5, fill: \"ab\")\n",
        "pad(\"abc\", width: 5, fill: \"\")\n",
        "join([1, 2], sep: \", \")\n",
        "upper(3)\n",
        "starts_with(\"abc\")\n",
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

    /// The length never changes under `upper` and `lower`; `trim` is idempotent; a slice from
    /// the first character of the whole length is the whole text.
    #[test]
    fn lengths_trim_and_whole_slices(s in "\\PC{0,12}") {
        let lit = literal(&s);
        let n = s.chars().count();
        let lines = printed(&format!(
            "print(len(upper({lit})))\nprint(len(lower({lit})))\nprint(equal(trim(trim({lit})), trim({lit})))\nprint(equal(slice({lit}, start: 1, len: {n}), {lit}))\nprint(len(pad({lit}, width: {n})))\n"
        ));
        prop_assert_eq!(&lines, &vec![n.to_string(), n.to_string(), "true".to_string(), "true".to_string(), n.to_string()]);
    }
}
