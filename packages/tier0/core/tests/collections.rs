//! The collection functions against R (testing handbook, section 1.1; release 0.5.0, step 11):
//! `length`, `nchar(type = "chars")`, `x[[1]]`, `tail(x, 1)`, `c` and `rev` on the lists and
//! strings of `tests/reference/r/collections.R`, exact.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/collections.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// The lines a script prints, with no message.
fn printed(script: &str) -> Vec<String> {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}");
    out.output.stdout.lines().map(str::to_string).collect()
}

/// R writes a number with 15 significant digits; Zeff prints six, so the comparison reads both
/// as numbers and asks for equality, which these exact values give.
fn number(text: &str) -> f64 {
    text.parse()
        .unwrap_or_else(|_| panic!("`{text}` is not a number"))
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn lists_agree_with_r() {
    let reference = r_reference();
    for case in reference["lists"].as_array().unwrap() {
        let items = strings(&case["items"]);
        // The items are written with every digit R gave, so the list is R's to the bit.
        let list = format!("[{}]", items.join(", "));
        let lines = printed(&format!("let xs = {list}\nlen(xs)\nfirst(xs)\nlast(xs)\n"));
        assert_eq!(lines[0], case["len"].to_string(), "{list}");
        assert_eq!(
            number(&lines[1]),
            number(case["first"].as_str().unwrap()),
            "{list}"
        );
        assert_eq!(
            number(&lines[2]),
            number(case["last"].as_str().unwrap()),
            "{list}"
        );
        let appended = printed(&format!("append({list}, 9)\n"));
        let expected: Vec<f64> = strings(&case["appended"])
            .iter()
            .map(|s| number(s))
            .collect();
        assert_eq!(
            appended.iter().map(|s| number(s)).collect::<Vec<_>>(),
            expected,
            "{list}"
        );
        let reversed = printed(&format!("reversed({list})\n"));
        let expected: Vec<f64> = strings(&case["reversed"])
            .iter()
            .map(|s| number(s))
            .collect();
        assert_eq!(
            reversed.iter().map(|s| number(s)).collect::<Vec<_>>(),
            expected,
            "{list}"
        );
    }
}

#[test]
fn string_lengths_agree_with_r_in_code_points() {
    let reference = r_reference();
    for case in reference["strings"].as_array().unwrap() {
        let text = case["text"].as_str().unwrap();
        // Written with `\u{}` escapes, so the script file holds plain ASCII.
        let escaped: String = text
            .chars()
            .map(|c| {
                if c.is_ascii() && c != '"' && c != '\\' {
                    c.to_string()
                } else {
                    format!("\\u{{{:X}}}", c as u32)
                }
            })
            .collect();
        let lines = printed(&format!("len(\"{escaped}\")\n"));
        assert_eq!(lines[0], case["len"].to_string(), "{text:?}");
    }
}
