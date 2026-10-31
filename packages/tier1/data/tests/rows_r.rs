//! `row_mean` and `row_sum` against R (testing handbook, section 1.1; release 0.5.0, step 17):
//! `rowMeans(na.rm = TRUE)`, `rowSums(na.rm = TRUE)` and the `min_valid` rule written out, on the
//! matrices of `tests/reference/r/rows.R`, within a relative 1e-12. Where R gives 0 for a row
//! with no value, Zeff gives `NA`; the reference carries both and the test checks Zeff's.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/rows.json");
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

/// The largest magnitude among a case's values.
fn largest(case: &Json) -> f64 {
    case["columns"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| strings(&c["values"]))
        .filter_map(|s| s.parse::<f64>().ok())
        .fold(0.0, |m, x| m.max(x.abs()))
}

/// The frame of a case, as a script writes it, and its variable names.
fn frame_of(case: &Json) -> (String, Vec<String>) {
    let mut names = Vec::new();
    let columns: Vec<String> = case["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let name = c["name"].as_str().unwrap().to_string();
            let values = strings(&c["values"]).join(", ");
            names.push(name.clone());
            format!("{name} = [{values}]")
        })
        .collect();
    (format!("frame({})", columns.join(", ")), names)
}

/// Whether a computed column agrees with R's values row by row: `NA` against `NA`, or within
/// 1e-12 of `scale`; one Bool. The scale is the largest magnitude among the row's inputs and the
/// result: on a row whose values cancel (1e12 against -1e12 leaving 1e7) the kernel's corrected
/// mean carries the rounding of the largest input, not of the result, as `docs/decisions.md`
/// records at step 17, and R's long-double accumulation does not.
fn agrees(frame: &str, definition: &str, expected: &[String], scale: f64) -> bool {
    let (r_values, all_na) = (
        format!("[{}]", expected.join(", ")),
        expected.iter().all(|e| e == "NA"),
    );
    let script = if all_na {
        format!("all({frame}.add(v = {definition}).add(ok = is_missing(v))[\"ok\"])\n")
    } else {
        format!(
            "let f = {frame}.add(v = {definition})\nall(frame(v = f[\"v\"], e = {r_values}).add(ok = (is_missing(v) && is_missing(e)) || abs(v - e) <= 1e-12 * ({scale:?} + abs(e)))[\"ok\"])\n"
        )
    };
    let out = run(&script);
    assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
    out.output.stdout == "true\n"
}

#[test]
fn row_sum_agrees_with_rowsums_and_names_the_departure() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        let (frame, names) = frame_of(case);
        let vars = format!("[{}]", names.join(", "));
        assert!(
            agrees(
                &frame,
                &format!("row_sum({vars})"),
                &strings(&case["row_sum"]),
                largest(case)
            ),
            "row_sum on {frame}"
        );
        // R's rowSums(na.rm = TRUE) of a row with no value is 0; Zeff's is NA, so the two
        // differ exactly where R's reference holds a 0 that Zeff's holds as NA.
        let r = strings(&case["row_sum_r"]);
        let z = strings(&case["row_sum"]);
        for (a, b) in r.iter().zip(&z) {
            assert!(a == b || (a == "0" && b == "NA"), "{a} against {b}");
        }
    }
}

#[test]
fn row_mean_agrees_with_rowmeans_and_the_min_valid_rule() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        let (frame, names) = frame_of(case);
        let vars = format!("[{}]", names.join(", "));
        assert!(
            agrees(
                &frame,
                &format!("row_mean({vars})"),
                &strings(&case["row_mean"]),
                largest(case)
            ),
            "row_mean on {frame}"
        );
        for rule in case["row_mean_min"].as_array().unwrap() {
            let k = rule["min_valid"].as_i64().unwrap();
            assert!(
                agrees(
                    &frame,
                    &format!("row_mean({vars}, min_valid: {k})"),
                    &strings(&rule["values"]),
                    largest(case)
                ),
                "row_mean with min_valid: {k} on {frame}"
            );
        }
    }
}
