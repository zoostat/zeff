//! `assert`, `n_missing` and `fill_missing` against R (testing handbook, section 1.1; release
//! 0.5.0, step 15): `stopifnot`, `sum(is.na(x))` and `replace(x, is.na(x), v)` on the vectors
//! of `tests/reference/r/missing.R`, exact. R's `replace` of an integer vector by a double gives
//! doubles, as Zeff's joining rule does; where R would coerce numbers to text to fill them, Zeff
//! refuses (Z0214), which the engine's tests cover.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/missing.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn printed(script: &str) -> Vec<String> {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}");
    out.output.stdout.lines().map(str::to_string).collect()
}

fn same(a: &str, b: &str) -> bool {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

#[test]
fn counts_and_filling_agree_with_r() {
    let reference = r_reference();
    for case in reference["numbers"].as_array().unwrap() {
        let items = strings(&case["items"]);
        if items.is_empty() {
            continue; // an empty list alone has no type
        }
        let list = format!("[{}]", items.join(", "));
        let n = case["n_missing"].to_string();
        if items.iter().all(|s| s == "NA") {
            // A list or column of only `NA` has no type of its own (Z0219); the count still
            // works where a parameter types the list.
            assert_eq!(
                printed(&format!("n_missing({list})\n")),
                vec![n.clone()],
                "{list}"
            );
            continue;
        }
        let lines = printed(&format!(
            "let xs = {list}\nn_missing(xs)\nlet d = frame(x = xs)\nn_missing(d[\"x\"])\nn_missing(fill_missing(d[\"x\"], 0))\nfill_missing(d[\"x\"], 0)\n"
        ));
        assert_eq!(lines[0], n, "{list}");
        assert_eq!(lines[1], n, "{list}");
        assert_eq!(lines[2], case["n_missing_after"].to_string(), "{list}");
        let filled = strings(&case["filled"]);
        assert_eq!(lines.len(), 4 + filled.len(), "{list}: {lines:?}");
        for (f, e) in lines[4..].iter().zip(&filled) {
            assert!(same(f, e), "{list}: {f} against {e}");
        }
    }
    for case in reference["texts"].as_array().unwrap() {
        let items: Vec<String> = strings(&case["items"])
            .iter()
            .map(|s| {
                if s == "NA" {
                    "NA".to_string()
                } else {
                    format!("\"{s}\"")
                }
            })
            .collect();
        let list = format!("[{}]", items.join(", "));
        if items.iter().all(|s| s == "NA") {
            assert_eq!(
                printed(&format!("n_missing({list})\n")),
                vec![case["n_missing"].to_string()],
                "{list}"
            );
            continue;
        }
        let lines = printed(&format!(
            "let d = frame(s = {list})\nn_missing(d[\"s\"])\nfill_missing(d[\"s\"], \"none\")\n"
        ));
        assert_eq!(lines[0], case["n_missing"].to_string(), "{list}");
        assert_eq!(&lines[2..], strings(&case["filled"]).as_slice(), "{list}");
    }
}

#[test]
fn assert_agrees_with_stopifnot() {
    let reference = r_reference();
    for case in reference["checks"].as_array().unwrap() {
        // R's `c(1, 2) > 0` reduced by `all` is Zeff's `all([1, 2] > 0)`... written out as values.
        let condition = match case["condition"].as_str().unwrap() {
            "1 + 1 == 2" => "1 + 1 == 2",
            "2 > 3" => "2 > 3",
            "all(c(1, 2) > 0)" => "all([1 > 0, 2 > 0])",
            other => panic!("{other}"),
        };
        let out = run(&format!("assert({condition}, \"check\")\n"));
        let holds = case["holds"].as_bool().unwrap();
        assert_eq!(
            out.exit_code == 0,
            holds,
            "{condition}: {}",
            out.output.stderr
        );
        if !holds {
            assert!(
                out.output.stderr.contains("error[Z0001]: check"),
                "{}",
                out.output.stderr
            );
        }
    }
}
