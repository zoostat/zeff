//! `iqr` and `mad` against R (testing handbook, section 1.1; release 0.5.0, step 19): `IQR` under
//! the nine quantile types and `mad` under three constants, on the vectors of
//! `tests/reference/r/spread.R`, within a relative 1e-12 (the quantile definitions agree to the
//! bit; the median of the deviations and the constant's product may differ in the last bit).

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/spread.json");
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

/// One line per check, `true` when Zeff's value is R's within 1e-12 relative (`NA` against `NA`).
fn checks(frame: &str, calls: &[(String, String)]) -> Vec<String> {
    let body: String = calls
        .iter()
        .map(|(call, expected)| {
            if expected == "NA" {
                format!("is_missing({call})\n")
            } else if expected == "Inf" || expected == "-Inf" {
                format!("{call} == {expected}\n")
            } else {
                format!("abs({call} - {expected}) <= 1e-12 * abs({expected})\n")
            }
        })
        .collect();
    let out = run(&format!("let d = frame(x = {frame})\n{body}"));
    assert_eq!(out.exit_code, 0, "{frame}\n{}", out.output.stderr);
    out.output.stdout.lines().map(str::to_string).collect()
}

#[test]
fn iqr_agrees_with_r_under_the_nine_types() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let frame = format!("[{}]", items.join(", "));
        let calls: Vec<(String, String)> = case["iqr"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                (
                    format!("d.iqr(x, type: {})", c["type"]),
                    c["value"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        let lines = checks(&frame, &calls);
        for (line, (call, expected)) in lines.iter().zip(&calls) {
            assert_eq!(line, "true", "{call} on {frame} against R's {expected}");
        }
    }
}

#[test]
fn mad_agrees_with_r_under_the_constants() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let frame = format!("[{}]", items.join(", "));
        let calls: Vec<(String, String)> = case["mad"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                (
                    format!("d.mad(x, constant: {})", c["constant"].as_str().unwrap()),
                    c["value"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        let lines = checks(&frame, &calls);
        for (line, (call, expected)) in lines.iter().zip(&calls) {
            assert_eq!(line, "true", "{call} on {frame} against R's {expected}");
        }
    }
}
