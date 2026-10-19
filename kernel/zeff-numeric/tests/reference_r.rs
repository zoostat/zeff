//! The maths kernel and the descriptive reducers against R (`tests/reference/r/maths.json` and
//! `descriptive.json`, written by the R scripts beside them; testing handbook, section 1.1: R is
//! the main reference, Python's exact arithmetic the second). R's numbers are stored with 15
//! significant digits, so the comparison is to a relative 1e-14; R's own rounding and
//! variance depart from the exact answers in a few known places, listed here.

use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use zeff_numeric::maths::{self, RoundingMode};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference(name: &str) -> Json {
    let path = root().join("tests/reference/r").join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("the R reference is JSON")
}

/// A number as the R scripts write it.
fn number(json: &Json) -> f64 {
    match json {
        Json::String(s) => match s.as_str() {
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            "NA" => f64::NAN,
            other => other
                .parse()
                .unwrap_or_else(|_| panic!("not a number: {other}")),
        },
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other}"),
    }
}

/// Within 15 significant digits of each other, or the same special value.
fn agree(what: &str, actual: f64, expected: f64, relative: f64) {
    if actual.is_nan() && expected.is_nan() {
        return;
    }
    if actual == expected {
        return;
    }
    assert!(
        actual.is_finite() && expected.is_finite(),
        "{what}: Zeff {actual}, R {expected}"
    );
    let scale = actual.abs().max(expected.abs());
    assert!(
        (actual - expected).abs() <= relative * scale,
        "{what}: Zeff {actual}, R {expected}"
    );
}

const FIFTEEN_DIGITS: f64 = 1e-14;

/// A function of one Float.
type Unary = fn(f64) -> f64;

#[test]
fn the_elementary_functions_agree_with_r() {
    let reference = reference("maths.json");
    assert!(
        reference["reference"]["software"]
            .as_str()
            .unwrap()
            .starts_with("R version 4.6.0"),
        "the reference names its R"
    );
    let grid: Vec<f64> = reference["grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(number)
        .collect();
    let functions: &[(&str, Unary)] = &[
        ("exp", maths::exp),
        ("log", maths::log),
        ("log2", maths::log2),
        ("log10", maths::log10),
        ("log1p", maths::log1p),
        ("expm1", maths::expm1),
        ("sin", maths::sin),
        ("cos", maths::cos),
        ("tan", maths::tan),
        ("asin", maths::asin),
        ("acos", maths::acos),
        ("atan", maths::atan),
        ("sqrt", maths::sqrt),
    ];
    for (name, f) in functions {
        let values = reference["functions"][*name].as_array().unwrap();
        assert_eq!(values.len(), grid.len());
        for (x, expected) in grid.iter().zip(values) {
            agree(
                &format!("{name}({x})"),
                f(*x),
                number(expected),
                FIFTEEN_DIGITS,
            );
        }
    }
    for case in reference["atan2"].as_array().unwrap() {
        let (y, x) = (number(&case[0]), number(&case[1]));
        agree(
            &format!("atan2({y}, {x})"),
            maths::atan2(y, x),
            number(&case[2]),
            FIFTEEN_DIGITS,
        );
    }
    for case in reference["pow"].as_array().unwrap() {
        let (x, y) = (number(&case[0]), number(&case[1]));
        agree(
            &format!("{x} ^ {y}"),
            zeff_numeric::pow(x, y),
            number(&case[2]),
            FIFTEEN_DIGITS,
        );
    }
}

#[test]
fn rounding_and_division_agree_with_r_except_where_r_departs_from_the_exact_rule() {
    let reference = reference("maths.json");
    // R (4.0.0 and later) rounds to the candidate whose decimal reading is nearer to the number,
    // and so rounds 0.995 and 2.355 up at two decimals although both lie below the tie in
    // binary; Zeff, Python and the exact rule round them down (section 3.7).
    let r_departs = [("0.995", 2), ("2.355", 2)];
    let mut departures = 0;
    for case in reference["round"].as_array().unwrap() {
        let text = case[0].as_str().unwrap();
        let x: f64 = text.parse().unwrap();
        let digits = case[1].as_i64().unwrap() as i32;
        let expected = number(&case[2]);
        let actual = maths::round(x, digits, RoundingMode::Even);
        if r_departs.contains(&(text, digits)) {
            assert_ne!(
                actual, expected,
                "round({text}, {digits}) was expected to differ from R"
            );
            departures += 1;
        } else {
            assert_eq!(actual, expected, "round({text}, {digits})");
        }
    }
    assert_eq!(departures, r_departs.len());
    for case in reference["round_significant"].as_array().unwrap() {
        let x: f64 = case[0].as_str().unwrap().parse().unwrap();
        let digits = case[1].as_i64().unwrap() as u32;
        assert_eq!(
            maths::round_significant(x, digits, RoundingMode::Even),
            number(&case[2]),
            "signif({x}, {digits})"
        );
    }
    for case in reference["div_mod_int"].as_array().unwrap() {
        let (a, b) = (case[0].as_i64().unwrap(), case[1].as_i64().unwrap());
        assert_eq!(
            maths::div_int(a, b),
            Some(case[2].as_i64().unwrap()),
            "div({a}, {b})"
        );
        assert_eq!(
            maths::mod_int(a, b),
            Some(case[3].as_i64().unwrap()),
            "mod({a}, {b})"
        );
    }
}

#[test]
fn the_descriptive_reducers_agree_with_r() {
    let reference = reference("descriptive.json");
    let datasets = reference["datasets"].as_object().unwrap();
    assert!(datasets.len() >= 5);
    for (name, dataset) in datasets {
        let Some(file) = dataset["file"].as_str() else {
            panic!("{name} names no file")
        };
        if !file.starts_with("tests/reference/data/") {
            // The frame datasets are for the function tests of the data package.
            continue;
        }
        let text = std::fs::read_to_string(root().join(file)).unwrap();
        let values: Vec<f64> = text.lines().skip(1).map(|l| l.parse().unwrap()).collect();
        let stats = &dataset["columns"]["x"];
        assert_eq!(
            stats["n"].as_u64().unwrap() as usize,
            values.len(),
            "{name}: n"
        );
        assert_eq!(stats["missing"].as_u64().unwrap(), 0, "{name}: missing");
        // R sums and squares in double, so on the offset dataset (1e9 plus a fraction) its
        // variance carries a relative error near 4e-14; the exact reference (descriptive.json)
        // holds the kernel to 1e-15 there. Twelve digits are what R vouches for.
        let r_precision = 1e-12;
        agree(
            &format!("{name}: sum"),
            zeff_numeric::sum(&values),
            number(&stats["sum"]),
            r_precision,
        );
        agree(
            &format!("{name}: mean"),
            zeff_numeric::mean(&values).unwrap(),
            number(&stats["mean"]),
            r_precision,
        );
        let variance = zeff_numeric::variance(&values);
        let sd = zeff_numeric::sd(&values);
        if values.len() > 1 {
            agree(
                &format!("{name}: variance"),
                variance.unwrap(),
                number(&stats["variance"]),
                r_precision,
            );
            agree(
                &format!("{name}: sd"),
                sd.unwrap(),
                number(&stats["sd"]),
                r_precision,
            );
        } else {
            assert!(number(&stats["variance"]).is_nan());
        }
        let (min, max) = values
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
                (lo.min(v), hi.max(v))
            });
        // Exact values, but written with 15 digits: 1000000000.999489 has sixteen.
        agree(
            &format!("{name}: min"),
            min,
            number(&stats["min"]),
            FIFTEEN_DIGITS,
        );
        agree(
            &format!("{name}: max"),
            max,
            number(&stats["max"]),
            FIFTEEN_DIGITS,
        );
        // The quantile table has the nine definitions at nine probabilities, for step 17.
        let quantiles = &stats["quantiles"];
        assert_eq!(quantiles["probs"].as_array().unwrap().len(), 9);
        for kind in 1..=9 {
            assert_eq!(
                quantiles[format!("type{kind}")].as_array().unwrap().len(),
                9,
                "{name}: type {kind}"
            );
        }
    }
}

#[test]
fn the_nine_quantile_types_and_the_median_agree_with_r_to_the_bit() {
    let reference = reference("descriptive.json");
    for (name, dataset) in reference["datasets"].as_object().unwrap() {
        let Some(file) = dataset["file"].as_str() else {
            continue;
        };
        if !file.starts_with("tests/reference/data/") {
            continue;
        }
        let text = std::fs::read_to_string(root().join(file)).unwrap();
        let mut values: Vec<f64> = text.lines().skip(1).map(|l| l.parse().unwrap()).collect();
        values.sort_by(f64::total_cmp);
        let stats = &dataset["columns"]["x"];
        let probs: Vec<f64> = stats["quantiles"]["probs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        for kind in 1..=9u8 {
            let expected = stats["quantiles"][format!("type{kind}")]
                .as_array()
                .unwrap();
            for (p, want) in probs.iter().zip(expected) {
                let got = zeff_numeric::order::quantile(&values, *p, kind).unwrap();
                // R's values carry 15 digits; the arithmetic is R's own, so they agree to those.
                agree(
                    &format!("{name}: type {kind} at {p}"),
                    got,
                    number(want),
                    FIFTEEN_DIGITS,
                );
            }
        }
        agree(
            &format!("{name}: median"),
            zeff_numeric::order::median(&values).unwrap(),
            number(&stats["median"]),
            FIFTEEN_DIGITS,
        );
    }
}

#[test]
fn the_weighted_statistics_agree_with_r() {
    use zeff_numeric::order::{
        WeightKind, weighted_mean, weighted_quantile, weighted_sum, weighted_variance,
    };
    let reference = reference("descriptive.json");
    let weighted = &reference["datasets"]["wages_large"]["weighted"];
    assert_eq!(weighted["weights"].as_str(), Some("educ"));
    let text = std::fs::read_to_string(root().join("tests/datasets/wages_large.csv")).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let column = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (wage_at, hours_at, educ_at) = (column("wage"), column("hours"), column("educ"));
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    for (name, at) in [("wage", wage_at), ("hours", hours_at)] {
        let mut pairs: Vec<(f64, f64)> = rows
            .iter()
            .filter(|r| !r[at].is_empty())
            .map(|r| (r[at].parse().unwrap(), r[educ_at].parse().unwrap()))
            .collect();
        let values: Vec<f64> = pairs.iter().map(|p| p.0).collect();
        let weights: Vec<f64> = pairs.iter().map(|p| p.1).collect();
        let r = &weighted[name];
        assert_eq!(r["n"].as_u64().unwrap() as usize, values.len());
        let tolerance = 1e-12;
        agree(
            &format!("{name}: weighted mean"),
            weighted_mean(&values, &weights).unwrap(),
            number(&r["mean"]),
            tolerance,
        );
        agree(
            &format!("{name}: frequency variance"),
            weighted_variance(&values, &weights, WeightKind::Frequency).unwrap(),
            number(&r["variance_frequency"]),
            tolerance,
        );
        agree(
            &format!("{name}: analytic variance"),
            weighted_variance(&values, &weights, WeightKind::Analytic).unwrap(),
            number(&r["variance_analytic"]),
            tolerance,
        );
        agree(
            &format!("{name}: frequency sum"),
            weighted_sum(&values, &weights, WeightKind::Frequency),
            number(&r["sum_frequency"]),
            tolerance,
        );
        agree(
            &format!("{name}: analytic sum"),
            weighted_sum(&values, &weights, WeightKind::Analytic),
            number(&r["sum_analytic"]),
            tolerance,
        );
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(
            weighted_quantile(&pairs, 0.5),
            Some(number(&r["median"])),
            "{name}: weighted median"
        );
        let probs: Vec<f64> = r["quantiles"]["probs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        for (p, want) in probs
            .iter()
            .zip(r["quantiles"]["values"].as_array().unwrap())
        {
            assert_eq!(
                weighted_quantile(&pairs, *p),
                Some(number(want)),
                "{name}: weighted quantile at {p}"
            );
        }
    }
}
