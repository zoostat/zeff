//! The maths kernel against `tests/reference/maths.json` (Python's `math` and `decimal` modules;
//! testing handbook, section 1.1): the elementary functions within two units in the last place and
//! exact where the answer is exact, the roundings and the divisions exactly.

use serde_json::Value as Json;
use std::path::Path;
use zeff_numeric::maths::{self, RoundingMode};

fn reference() -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/maths.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("maths.json is JSON")
}

/// A number as the reference writes it.
fn number(text: &str) -> f64 {
    match text {
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        "NA" => f64::NAN,
        other => other
            .parse()
            .unwrap_or_else(|_| panic!("not a number: {other}")),
    }
}

/// How many representable doubles lie between `a` and `b`.
fn ulps_apart(a: f64, b: f64) -> u64 {
    if a == b {
        return 0;
    }
    let key = |x: f64| -> i64 {
        let bits = x.to_bits() as i64;
        if bits < 0 { i64::MIN - bits } else { bits }
    };
    key(a).abs_diff(key(b))
}

fn agree(what: &str, got: f64, want: f64) {
    if want.is_nan() {
        assert!(got.is_nan(), "{what}: expected NA, got {got:e}");
    } else if want.is_infinite() || want == 0.0 || want.fract() == 0.0 && want.abs() < 1e15 {
        // Exact answers: infinities, zeros and whole numbers.
        assert_eq!(got, want, "{what}");
    } else {
        let ulps = ulps_apart(got, want);
        assert!(
            ulps <= 2,
            "{what}: {got:e} is {ulps} ulps from the reference {want:e}"
        );
    }
}

#[test]
fn the_elementary_functions_agree_with_the_reference_within_two_ulps() {
    let reference = reference();
    let grid: Vec<f64> = reference["grid"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| number(v.as_str().unwrap()))
        .collect();
    let functions = reference["functions"].as_object().unwrap();
    assert!(functions.len() >= 13);
    for (name, values) in functions {
        let f: fn(f64) -> f64 = match name.as_str() {
            "exp" => maths::exp,
            "log" => maths::log,
            "log2" => maths::log2,
            "log10" => maths::log10,
            "log1p" => maths::log1p,
            "expm1" => maths::expm1,
            "sin" => maths::sin,
            "cos" => maths::cos,
            "tan" => maths::tan,
            "asin" => maths::asin,
            "acos" => maths::acos,
            "atan" => maths::atan,
            "sqrt" => maths::sqrt,
            other => panic!("no kernel function called {other}"),
        };
        for (x, want) in grid.iter().zip(values.as_array().unwrap()) {
            let want = number(want.as_str().unwrap());
            agree(&format!("{name}({x:e})"), f(*x), want);
        }
    }
    for case in reference["atan2"].as_array().unwrap() {
        let (y, x, want) = (
            number(case[0].as_str().unwrap()),
            number(case[1].as_str().unwrap()),
            number(case[2].as_str().unwrap()),
        );
        agree(&format!("atan2({y}, {x})"), maths::atan2(y, x), want);
    }
    for case in reference["pow"].as_array().unwrap() {
        let (x, y, want) = (
            number(case[0].as_str().unwrap()),
            number(case[1].as_str().unwrap()),
            number(case[2].as_str().unwrap()),
        );
        agree(&format!("{x} ^ {y}"), zeff_numeric::pow(x, y), want);
    }
}

#[test]
fn rounding_agrees_with_python_and_the_decimal_module_exactly() {
    let reference = reference();
    let mut checked = 0;
    for case in reference["round"].as_array().unwrap() {
        let x = number(case[0].as_str().unwrap());
        let digits = case[1].as_i64().unwrap() as i32;
        let even = number(case[2].as_str().unwrap());
        let away = number(case[3].as_str().unwrap());
        assert_eq!(
            maths::round(x, digits, RoundingMode::Even).to_bits(),
            even.to_bits(),
            "round({x:?}, {digits}) half to even: got {}, Python gives {even}",
            maths::round(x, digits, RoundingMode::Even)
        );
        assert_eq!(
            maths::round(x, digits, RoundingMode::Away).to_bits(),
            away.to_bits(),
            "round({x:?}, {digits}) half away: got {}, decimal gives {away}",
            maths::round(x, digits, RoundingMode::Away)
        );
        checked += 1;
    }
    assert!(checked > 100);
    for case in reference["round_significant"].as_array().unwrap() {
        let x = number(case[0].as_str().unwrap());
        let sig = case[1].as_u64().unwrap() as u32;
        let want = number(case[2].as_str().unwrap());
        assert_eq!(
            maths::round_significant(x, sig, RoundingMode::Even),
            want,
            "{x} to {sig} significant digits"
        );
    }
}

#[test]
fn floored_division_agrees_with_python_exactly() {
    let reference = reference();
    for case in reference["div_mod_int"].as_array().unwrap() {
        let (a, b) = (case[0].as_i64().unwrap(), case[1].as_i64().unwrap());
        let (q, r) = (case[2].as_i64(), case[3].as_i64().unwrap());
        // Python's quotient of i64::MIN by -1 does not fit an i64; the kernel says so.
        assert_eq!(maths::div_int(a, b), q, "div({a}, {b})");
        assert_eq!(maths::mod_int(a, b), Some(r), "mod({a}, {b})");
    }
    for case in reference["div_mod_float"].as_array().unwrap() {
        let (a, b) = (
            number(case[0].as_str().unwrap()),
            number(case[1].as_str().unwrap()),
        );
        let (q, r) = (
            number(case[2].as_str().unwrap()),
            number(case[3].as_str().unwrap()),
        );
        assert_eq!(maths::div_float(a, b), q, "div({a}, {b})");
        agree(&format!("mod({a}, {b})"), maths::mod_float(a, b), r);
    }
}
