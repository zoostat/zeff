//! Reference tests: `sum`, `mean`, `variance` and `sd` against Python's `statistics` module, which
//! computes in exact rational arithmetic (tests/reference/descriptive.py writes the expected
//! values). The accuracy class of these reducers is "closed form", 1e-6 relative; the kernel is
//! held to 1e-12.

use serde_json::Value;
use zeff_numeric::{mean, sd, sum, variance};

const TOLERANCE: f64 = 1e-12;

fn reference() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/reference/descriptive.json"
    );
    let text = std::fs::read_to_string(path).expect("the reference file is present");
    serde_json::from_str(&text).expect("the reference file is JSON")
}

fn number(value: &Value) -> f64 {
    value
        .as_str()
        .expect("numbers are stored as text")
        .parse()
        .expect("a valid number")
}

fn assert_close(what: &str, dataset: &str, actual: f64, expected: f64) {
    let error = (actual - expected).abs();
    let allowed = TOLERANCE * expected.abs();
    assert!(
        actual == expected || error <= allowed,
        "{what} of `{dataset}`: got {actual:e}, the reference has {expected:e} (relative error {:e})",
        error / expected.abs()
    );
}

#[test]
fn every_dataset_matches_the_exact_reference() {
    let reference = reference();
    let datasets = reference["datasets"].as_object().expect("datasets");
    assert!(datasets.len() >= 2, "at least two datasets are required");
    for (name, entry) in datasets {
        let values: Vec<f64> = entry["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        assert_close("sum", name, sum(&values), number(&entry["sum"]));
        assert_close("mean", name, mean(&values).unwrap(), number(&entry["mean"]));
        assert_close(
            "variance",
            name,
            variance(&values).unwrap(),
            number(&entry["variance"]),
        );
        assert_close("sd", name, sd(&values).unwrap(), number(&entry["sd"]));
    }
}

#[test]
fn the_reference_says_which_software_made_it() {
    let reference = reference();
    assert!(reference["reference"]["python"].as_str().is_some());
    assert!(
        reference["reference"]["software"]
            .as_str()
            .unwrap()
            .contains("statistics")
    );
}
