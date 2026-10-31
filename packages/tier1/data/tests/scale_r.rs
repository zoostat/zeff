//! `center`, `standardize` and `between` against R (testing handbook, section 1.1; release 0.5.0,
//! step 16): `scale(x, scale = FALSE)`, `scale(x)` and `low <= x & x <= high` on the vectors of
//! `tests/reference/r/scale.R`, each value within a relative 1e-12 (the sums are compensated here
//! and plain in R, so the last digits may differ); `between` exactly.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/scale.json");
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

#[test]
fn center_and_standardize_agree_with_scale() {
    let reference = r_reference();
    for case in reference["scale"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let list = format!("[{}]", items.join(", "));
        for (name, key) in [("center", "centered"), ("standardize", "standardized")] {
            let expected = strings(&case[key]);
            let r_values = format!("[{}]", expected.join(", "));
            // Row by row inside a frame: NA against NA agrees, a value within 1e-12 relative
            // agrees; `all` over the rows gives one Bool. A list of only NA has no type, so
            // that case asks whether every row is missing.
            let script = if expected.iter().all(|e| e == "NA") {
                format!(
                    "let d = frame(x = {list})\nall(frame(v = {name}(d[\"x\"])).add(ok = is_missing(v))[\"ok\"])\n"
                )
            } else {
                format!(
                    "let d = frame(x = {list})\nlet f = frame(v = {name}(d[\"x\"]), e = {r_values})\nall(f.add(ok = (is_missing(v) && is_missing(e)) || abs(v - e) <= 1e-12 * abs(e))[\"ok\"])\n"
                )
            };
            let out = run(&script);
            assert_eq!(out.exit_code, 0, "{name}({list}):\n{}", out.output.stderr);
            assert_eq!(
                out.output.stdout, "true\n",
                "{name}({list}) against R's {r_values}:\n{}",
                out.output.stderr
            );
        }
    }
}

#[test]
fn between_agrees_with_its_definition_in_r() {
    let reference = r_reference();
    for case in reference["between"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let list = format!("[{}]", items.join(", "));
        let (low, high) = (
            case["low"].as_str().unwrap(),
            case["high"].as_str().unwrap(),
        );
        let out = run(&format!(
            "let d = frame(x = {list})\nbetween(d[\"x\"], {low}, {high})\n"
        ));
        assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().skip(1).collect();
        assert_eq!(
            lines,
            strings(&case["inside"]),
            "between({list}, {low}, {high})"
        );
    }
}
