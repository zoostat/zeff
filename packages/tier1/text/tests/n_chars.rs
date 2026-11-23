//! `n_chars` (release 0.9.0, step 31a; open decision #165): the number of Unicode scalar
//! values in a text, and row by row the length of each value of a variable. Against R's
//! `nchar(type = "chars")` (`tests/reference/r/text.json`) on fifteen texts with accented,
//! combining and non-Latin letters, on a short variable with a missing value and an empty
//! text, and on the industry of every row of the shipped wages; the edge cases (`NA`, the
//! empty text, white space, a flag of two code points); the column forms, under `where` and
//! `by:`; the refusals; the note that `add` gives when `len` of a text variable is spread over
//! its rows; and the properties that `n_chars` of one text is `len` of it, is unchanged by
//! `upper` and `lower`, and adds over `join`.

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

/// R's count as the engine prints it: a number, or `NA`.
fn count_text(value: &Json) -> String {
    match value {
        Json::String(s) => s.clone(),
        other => other.as_i64().unwrap().to_string(),
    }
}

// ----- against R -----

#[test]
fn one_text_agrees_with_r_on_fifteen_texts() {
    let r = reference();
    let cases = r["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 15);
    for case in cases {
        let s = literal(case["text"].as_str().unwrap());
        let lines = printed(&format!(
            "print(n_chars({s}))\nprint(n_chars({s}) == len({s}))\n"
        ));
        assert_eq!(
            lines,
            vec![count_text(&case["len"]), "true".to_string()],
            "{s}"
        );
    }
}

#[test]
fn a_variable_agrees_with_r_value_by_value() {
    let r = reference();
    let short = &r["n_chars"]["short"];
    let values = short["values"].as_array().unwrap();
    let missing = short["na"].as_array().unwrap();
    let items: Vec<String> = values
        .iter()
        .zip(missing)
        .map(|(value, na)| {
            if na.as_bool().unwrap() {
                "NA".to_string()
            } else {
                literal(value.as_str().unwrap())
            }
        })
        .collect();
    let lines = printed(&format!(
        "let d = frame(s = [{}])\nd.add(n = n_chars(s))[\"n\"]\nn_chars(d[\"s\"])\n",
        items.join(", ")
    ));
    // The variable inside `add`, and the column given to the function: the same lengths.
    let mut expected = vec![format!("Column<Int> · {} values", values.len())];
    expected.extend(short["chars"].as_array().unwrap().iter().map(count_text));
    let twice: Vec<String> = expected.iter().chain(&expected).cloned().collect();
    assert_eq!(lines, twice);
    // The missing text has no length, and the empty text has length 0: both are in the case.
    assert!(expected.contains(&"NA".to_string()) && expected.contains(&"0".to_string()));
}

#[test]
fn the_industries_of_the_shipped_wages_agree_with_r() {
    let r = reference();
    let wages = &r["n_chars"]["wages"];
    assert_eq!(wages["variable"], "industry");
    let rows = wages["rows"].as_i64().unwrap();
    let labels = wages["labels"].as_array().unwrap();
    let chars = wages["chars"].as_array().unwrap();
    assert_eq!(labels.len(), 12);
    // Over every row of the file: the rows, the sum of the lengths, the longest, and the
    // rows that have it; then, label by label, the shortest and the longest length among the
    // rows of that industry, which are one number.
    let mut script = String::from(
        "let w = dataset(wages).add(letters = n_chars(industry))\nprint(w.n_rows())\nprint(w.sum(letters))\nprint(w.max(letters))\nprint(w.where(letters == w.max(letters)).n_rows())\n",
    );
    for label in labels {
        let label = literal(label.as_str().unwrap());
        script.push_str(&format!(
            "print(w.where(industry == {label}).min(letters))\nprint(w.where(industry == {label}).max(letters))\n"
        ));
    }
    // Nothing is said: no length is missing, so no reducer skips a row.
    let lines = printed(&script);
    assert_eq!(lines[0], rows.to_string());
    assert_eq!(lines[1], count_text(&wages["total"]));
    assert_eq!(lines[2], count_text(&wages["longest"]));
    assert_eq!(lines[3], count_text(&wages["rows_of_longest"]));
    for (at, (label, n)) in labels.iter().zip(chars).enumerate() {
        let n = count_text(n);
        assert_eq!(lines[4 + 2 * at], n, "{label}");
        assert_eq!(lines[5 + 2 * at], n, "{label}");
    }
    assert_eq!(lines.len(), 4 + 2 * 12);
}

// ----- the edge cases -----

#[test]
fn na_the_empty_text_and_what_counts_as_one() {
    let lines = printed(
        "print(n_chars(NA))\nprint(n_chars(\"\"))\nprint(n_chars(\" \"))\nprint(n_chars(\"a\\tb\\nc\"))\nprint(n_chars(\"\\u{1F1EC}\\u{1F1E7}\"))\nprint(n_chars(\"e\\u{301}\"))\nprint(n_chars(\"\\u{e9}\"))\nprint(n_chars(trim(\"  padded  \")))\nprint(n_chars(\"x\") + 1)\nprint(type_of(n_chars(\"x\")))\n",
    );
    assert_eq!(
        lines,
        vec![
            "NA", // a missing text has no length
            "0",  // the empty text
            "1",  // a space is a character
            "5",  // a tab and a line break are one each
            "2",  // a flag is two code points
            "2",  // a letter and its combining accent
            "1",  // the same letter as one code point
            "6", "2", "Int",
        ]
    );
}

#[test]
fn row_by_row_under_where_and_by() {
    let lines = printed(
        "let d = frame(job = [\"clerk\", \"manager\", \"chef\", \"cook\", \"\"], site = [1, 1, 2, 2, 2])\nd.where(n_chars(job) > 4).n_rows()\nd.where(n_chars(job) == 0).n_rows()\nlet e = d.add(n = n_chars(job))\ne.add(longest = max(n), by: site)[\"longest\"]\ne.aggregate(letters = sum(n), by: site)[\"letters\"]\ne.sort(n)[\"job\"]\n",
    );
    assert_eq!(
        lines,
        vec![
            "2",
            "1",
            // The longest of each site.
            "Column<Int> · 5 values",
            "7",
            "7",
            "4",
            "4",
            "4",
            "Column<Int> · 2 values",
            "12",
            "8",
            // Sorted by length, the empty text first.
            "Column<Str> · 5 values",
            "",
            "chef",
            "cook",
            "clerk",
            "manager",
        ]
    );
    // A row whose text is missing has no length, so a condition on it is `NA` and the row
    // is dropped, with the note `where` gives for any such row.
    let (code, stdout, stderr) =
        go("frame(job = [\"clerk\", NA]).where(n_chars(job) > 0).n_rows()\n");
    assert_eq!((code, stdout.as_str()), (0, "1\n"), "{stderr}");
    assert_eq!(
        stderr,
        "note: 1 row dropped where `n_chars(job) > 0` was NA; `where(is_missing(job))` lists them (line 1)\n"
    );
}

// ----- the note of `len`, and the messages -----

/// `len` of a text variable as a whole definition of `add` is the number of rows on every
/// row: the note says so and names `n_chars`. It is given for a text variable alone, for the
/// whole definition alone, and not where `n_chars` or `len` of one text was written.
#[test]
fn add_notes_len_of_a_text_variable_spread_over_its_rows() {
    let data = "let d = frame(job = [\"clerk\", \"manager\", NA, \"cook\"], site = [1, 1, 2, 2], pay = [1.5, 2.5, 3.5, 4.5])\n";
    let (code, stdout, stderr) = go(&format!(
        "{data}d.add(l = len(job))\nd.add(l = len(job), by: site)\nd.add(l = n_chars(job), rows = len(job))\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("len_spread_over_rows", format!("{stdout}---\n{stderr}"));
    assert_eq!(
        stderr,
        "note: `len(job)` is the number of rows of `job`, the same on every row; `n_chars(job)` gives the length of each value (line 2)\nnote: `len(job)` is the number of rows of `job` in each group, the same on every row of the group; `n_chars(job)` gives the length of each value (line 3)\nnote: `len(job)` is the number of rows of `job`, the same on every row; `n_chars(job)` gives the length of each value (line 4)\n"
    );
    // Not for a number, not inside a larger expression, not outside `add`, not in `update`.
    for quiet in [
        "d.add(l = len(pay))\n",
        "d.add(share = 1 / len(job))\n",
        "d.add(l = n_chars(job))\n",
        "let rows_of_job = len(d[\"job\"])\n",
        "d.add(l = len(\"clerk\"))\n",
        "d.update(pay = len(job))\n",
    ] {
        let (code, _, stderr) = go(&format!("{data}{quiet}"));
        assert_eq!(code, 0, "{quiet}{stderr}");
        assert_eq!(stderr, "", "{quiet}");
    }
}

#[test]
fn every_message() {
    let scripts = [
        "n_chars(3)\n",
        "n_chars()\n",
        "n_chars(\"a\", \"b\")\n",
        "n_chars([\"a\", \"bc\"])\n",
        "let d = frame(x = [1.5, 2.5])\nd.add(n = n_chars(x))\n",
        "let d = frame(x = [1.5, 2.5])\nn_chars(d)\n",
        "n_chars(date(\"2026-11-23\"))\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("n_chars_messages", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Of one text `n_chars` is `len`; it is the number of scalar values, never of bytes;
    /// `upper` and `lower` leave it as it is; and it adds over `join`.
    #[test]
    fn n_chars_counts_scalar_values_and_adds_over_join(a in "\\PC{0,12}", b in "\\PC{0,12}") {
        let (la, lb) = (literal(&a), literal(&b));
        let (na, nb) = (a.chars().count(), b.chars().count());
        let lines = printed(&format!(
            "print(n_chars({la}))\nprint(n_chars({la}) == len({la}))\nprint(n_chars(upper({la})))\nprint(n_chars(lower({la})))\nprint(n_chars(join([{la}, {lb}], sep: \"\")))\nprint(n_chars(join([{la}, {lb}], sep: \"--\")))\n"
        ));
        prop_assert_eq!(
            &lines,
            &vec![
                na.to_string(),
                "true".to_string(),
                na.to_string(),
                na.to_string(),
                (na + nb).to_string(),
                (na + nb + 2).to_string(),
            ]
        );
    }

    /// In a variable each row has the length of its own value, whatever the others are, and
    /// a missing row stays missing.
    #[test]
    fn each_row_has_the_length_of_its_own_value(
        values in prop::collection::vec(prop::option::of("\\PC{0,8}"), 1..8),
    ) {
        let items: Vec<String> = values
            .iter()
            .map(|v| v.as_deref().map_or("NA".to_string(), literal))
            .collect();
        // A text among them, so that the list has a type when every drawn value is missing.
        let lines = printed(&format!(
            "let d = frame(s = [{}, \"end\"])\nd.add(n = n_chars(s))[\"n\"]\n",
            items.join(", ")
        ));
        let mut expected = vec![format!("Column<Int> · {} values", values.len() + 1)];
        expected.extend(
            values
                .iter()
                .map(|v| v.as_ref().map_or("NA".to_string(), |s| s.chars().count().to_string())),
        );
        expected.push("3".to_string());
        prop_assert_eq!(&lines, &expected);
    }
}
