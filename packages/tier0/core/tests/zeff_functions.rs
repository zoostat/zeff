//! The functions of `core` written in Zeff against R (testing handbook, section 1.1; release
//! 0.5.0, step 14): `logit` against `qlogis`, `inv_logit` against `plogis` and `clamp` against
//! `pmin(pmax())`, from `tests/reference/r/zeff_functions.R`, with probabilities down to 1e-12
//! from both ends and log odds out to 1e12. Each value is compared inside the engine with `==`,
//! so the comparison is of the exact doubles; `clamp` must match exactly, and `logit` and
//! `inv_logit` exactly or within a relative 1e-15 (the two logarithms and exponentials of the
//! two platforms' maths libraries may differ in the last bit).

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/zeff_functions.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A number or `NA`/`Inf`/`-Inf` as R wrote it, as a Zeff expression.
fn expression(value: &Json) -> String {
    let text = value.as_str().unwrap();
    match text {
        "Inf" | "-Inf" | "NA" => text.to_string(),
        other => other.to_string(),
    }
}

/// For each case two lines: whether the result is exactly R's, and whether it is exactly or
/// within a relative 1e-15 of it. Warnings go to the error stream and are not asked about.
fn compare(call: impl Fn(&Json) -> String, cases: &[Json]) -> (usize, usize) {
    let mut script = String::new();
    for case in cases {
        let got = call(case);
        let want = expression(&case["expected"]);
        script.push_str(&format!(
            "let r = {got}\nlet e = {want}\nr == e\nr == e || abs(r - e) <= 1e-15 * abs(e)\n"
        ));
    }
    let out = run(&script);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    let lines: Vec<&str> = out.output.stdout.lines().collect();
    assert_eq!(lines.len(), cases.len() * 2, "{}", out.output.stdout);
    let mut exact = 0;
    let mut close = 0;
    for (i, case) in cases.iter().enumerate() {
        if lines[2 * i] == "true" {
            exact += 1;
        }
        assert_eq!(
            lines[2 * i + 1],
            "true",
            "{} differs from R's {} by more than 1e-15 relative",
            call(case),
            case["expected"]
        );
        close += 1;
    }
    (exact, close)
}

#[test]
fn logit_agrees_with_qlogis() {
    let reference = r_reference();
    let cases = reference["logit"].as_array().unwrap();
    let (exact, close) = compare(|c| format!("logit({})", expression(&c["p"])), cases);
    eprintln!("logit: {exact} of {close} exactly R's qlogis");
    assert!(exact * 10 >= close * 9, "{exact} of {close} exact");
}

#[test]
fn inv_logit_agrees_with_plogis() {
    let reference = r_reference();
    let cases = reference["inv_logit"].as_array().unwrap();
    let (exact, close) = compare(|c| format!("inv_logit({})", expression(&c["x"])), cases);
    eprintln!("inv_logit: {exact} of {close} exactly R's plogis");
    assert!(exact * 10 >= close * 9, "{exact} of {close} exact");
}

#[test]
fn clamp_agrees_with_pmin_pmax_exactly() {
    let reference = r_reference();
    let cases = reference["clamp"].as_array().unwrap();
    let (exact, total) = compare(
        |c| {
            format!(
                "clamp({}, {}, {})",
                expression(&c["x"]),
                expression(&c["low"]),
                expression(&c["high"])
            )
        },
        cases,
    );
    assert_eq!(exact, total, "clamp is exact: {exact} of {total}");
}
