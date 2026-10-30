//! The order functions against R (testing handbook, section 1.1; release 0.5.0, step 12):
//! `unique`, `sort(na.last = TRUE)` both ways, `%in%`, `names` and `identical` on the lists and
//! strings of `tests/reference/r/collections_order.R`, exact. R's `%in%`, `unique` and
//! `identical` treat `NA` as one value, as Zeff's identity relation does, and `identical(-0, 0)`
//! is `TRUE` in both; R's `sort` drops `NA` unless `na.last` is set, so the reference sets it.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/collections_order.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn printed(script: &str) -> Vec<String> {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}");
    out.output.stdout.lines().map(str::to_string).collect()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// Two printed numbers or `NA`s are the same value; R writes 15 digits and Zeff six.
fn same(a: &str, b: &str) -> bool {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

fn assert_same_lines(found: &[String], expected: &[String], what: &str) {
    assert_eq!(
        found.len(),
        expected.len(),
        "{what}: {found:?} against {expected:?}"
    );
    for (f, e) in found.iter().zip(expected) {
        assert!(same(f, e), "{what}: {f} against {e}");
    }
}

/// A number as R wrote it, as a Zeff literal (`-0` as `-0.0` so that it stays a Float).
fn literal(items: &[String]) -> String {
    let written: Vec<String> = items
        .iter()
        .map(|s| {
            if s == "-0" {
                "-0.0".to_string()
            } else {
                s.clone()
            }
        })
        .collect();
    format!("[{}]", written.join(", "))
}

#[test]
fn numeric_lists_agree_with_r() {
    let reference = r_reference();
    let probes = strings(&reference["probes"]);
    for case in reference["lists"].as_array().unwrap() {
        let list = literal(&strings(&case["items"]));
        if list == "[]" {
            // An empty list alone has no type; its functions are tested on `[]` directly.
            let lines = printed("unique([])\nsorted([])\n");
            assert_eq!(lines, vec!["[]", "[]"]);
            continue;
        }
        let lines = printed(&format!("unique({list})\n"));
        assert_same_lines(
            &lines,
            &strings(&case["unique"]),
            &format!("unique of {list}"),
        );
        let lines = printed(&format!("sorted({list})\n"));
        assert_same_lines(&lines, &strings(&case["sorted"]), &format!("sorted {list}"));
        let lines = printed(&format!("sorted({list}, desc: true)\n"));
        assert_same_lines(
            &lines,
            &strings(&case["sorted_desc"]),
            &format!("sorted desc {list}"),
        );
        let script: String = probes
            .iter()
            .map(|p| format!("is_in({}, {list})\n", if p == "-0" { "-0.0" } else { p }))
            .collect();
        let lines = printed(&script);
        assert_eq!(lines, strings(&case["is_in"]), "is_in on {list}");
        let lines = printed(&format!("equal({list}, {list})\n"));
        assert_eq!(lines, vec![case["identical_self"].to_string()], "{list}");
    }
}

#[test]
fn string_lists_agree_with_r_in_code_point_order() {
    let reference = r_reference();
    for case in reference["strings"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let list = format!(
            "[{}]",
            items
                .iter()
                .map(|s| format!(
                    "\"{}\"",
                    s.chars()
                        .map(|c| if c.is_ascii() {
                            c.to_string()
                        } else {
                            format!("\\u{{{:X}}}", c as u32)
                        })
                        .collect::<String>()
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        assert_eq!(
            printed(&format!("unique({list})\n")),
            strings(&case["unique"]),
            "{list}"
        );
        assert_eq!(
            printed(&format!("sorted({list})\n")),
            strings(&case["sorted"]),
            "{list}"
        );
        assert_eq!(
            printed(&format!("sorted({list}, desc: true)\n")),
            strings(&case["sorted_desc"]),
            "{list}"
        );
    }
}

#[test]
fn record_keys_and_identical_pairs_agree_with_r() {
    let reference = r_reference();
    assert_eq!(
        printed("keys({mu: 0.0, sd: 1.0, share: 0.5})\n"),
        strings(&reference["record_keys"])
    );
    for pair in reference["identical_pairs"].as_array().unwrap() {
        let a = literal(&strings(&pair["a"]));
        let b = literal(&strings(&pair["b"]));
        assert_eq!(
            printed(&format!("equal({a}, {b})\n")),
            vec![pair["identical"].to_string()],
            "equal({a}, {b})"
        );
    }
}
