//! `skewness` and `kurtosis` against the four forms of Joanes and Gill written out in base R
//! (testing handbook, section 1.1; release 0.5.0, step 20), on the samples of
//! `tests/reference/r/moments.R`, within a relative 1e-12.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/moments.json");
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
fn the_four_forms_agree_with_r() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        let items = strings(&case["items"]);
        let frame = format!("[{}]", items.join(", "));
        let forms = [
            ("d.skewness(x, adjusted: false)", "g1"),
            ("d.skewness(x)", "G1"),
            ("d.kurtosis(x, adjusted: false)", "g2"),
            ("d.kurtosis(x)", "G2"),
            ("d.kurtosis(x, excess: false, adjusted: false)", "g2_plus_3"),
            ("d.kurtosis(x, excess: false)", "G2_plus_3"),
        ];
        let body: String = forms
            .iter()
            .map(|(call, key)| {
                let expected = case[key].as_str().unwrap();
                if expected == "NA" {
                    format!("is_missing({call})\n")
                } else {
                    format!("abs({call} - {expected}) <= 1e-12 * abs({expected})\n")
                }
            })
            .collect();
        let out = run(&format!("let d = frame(x = {frame})\n{body}"));
        assert_eq!(out.exit_code, 0, "{frame}\n{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        for (line, (call, key)) in lines.iter().zip(&forms) {
            assert_eq!(
                *line, "true",
                "{call} on {frame} against R's {key} = {}",
                case[key]
            );
        }
    }
}
