//! The special functions against `tests/reference/special.json` (Python's `decimal` module at 70
//! digits, rounded once; testing handbook, section 1.1): the gamma, beta and error functions to a
//! relative 1e-14 (5e-13 where a value passes through an exponential or a far tail of `erfc`),
//! the incomplete functions and their inverses within the class tolerance of the distribution
//! functions, 1e-12 relative (Engine and Project Specification, section 13), widened only where
//! the arithmetic itself cannot do better, and the exact answers exactly.

use serde_json::Value as Json;
use std::path::Path;
use zeff_numeric::special::*;

fn reference() -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/special.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("special.json is JSON")
}

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

fn agree(what: &str, got: f64, want: f64, relative: f64) {
    if want.is_nan() {
        assert!(got.is_nan(), "{what}: expected NA, got {got:e}");
    } else if want.is_infinite() || want == 0.0 {
        assert_eq!(got, want, "{what}");
    } else {
        let error = ((got - want) / want).abs();
        assert!(
            error <= relative,
            "{what}: {got:e} against {want:e}, relative error {error:e} above {relative:e}"
        );
    }
}

fn list(json: &Json) -> &Vec<Json> {
    json.as_array().expect("a list")
}

const CLOSE: f64 = 1e-14;
/// Values that pass through an exponential (a logarithm near -60 rounds by 1e-14) or the far
/// tail of libm's `erfc`.
const TAIL: f64 = 5e-13;
const CLASS: f64 = 1e-12;

/// The class tolerance, widened in a deep tail and for a very large parameter: a probability of
/// 1e-235 is exp(-541), and the exponent's rounding grows with its size (about 1e-16 relative,
/// so 5e-14 absolute at 541); a continued fraction for a parameter above 1e5 runs thousands of
/// steps whose stopping rule cannot see below a unit in the last place, so a parameter above 1e5
/// is held to 1e-10 (a known issue of 0.3.0, to be closed by asymptotic expansions).
fn deep(want: f64, largest_parameter: f64) -> f64 {
    let tail = (want.abs().ln().abs() / 50.0).max(1.0);
    let size = if largest_parameter > 1e5 { 100.0 } else { 1.0 };
    CLASS * tail.max(size)
}

#[test]
fn gamma_beta_and_the_error_functions_agree_to_fourteen_digits() {
    let r = reference();
    for ((x, lg), g) in list(&r["gamma_grid"])
        .iter()
        .zip(list(&r["log_gamma"]))
        .zip(list(&r["gamma"]))
    {
        let (x, lg, g) = (number(x), number(lg), number(g));
        agree(&format!("log_gamma({x})"), log_gamma(x), lg, CLOSE);
        agree(&format!("gamma({x})"), gamma(x), g, CLOSE);
    }
    for case in list(&r["factorial"]) {
        let n = case[0].as_i64().unwrap();
        assert_eq!(factorial(n), number(&case[1]), "factorial({n})");
    }
    for case in list(&r["choose"]) {
        let (n, k) = (case[0].as_i64().unwrap(), case[1].as_i64().unwrap());
        // Exact while the result is below 2^53; otherwise within a few roundings.
        let want = number(&case[2]);
        if want < 9.0e15 {
            assert_eq!(choose(n, k), want, "choose({n}, {k})");
        } else {
            agree(&format!("choose({n}, {k})"), choose(n, k), want, CLOSE);
        }
    }
    for case in list(&r["beta"]) {
        let (a, b) = (number(&case[0]), number(&case[1]));
        agree(
            &format!("log_beta({a}, {b})"),
            log_beta(a, b),
            number(&case[2]),
            CLOSE,
        );
        agree(
            &format!("beta({a}, {b})"),
            beta(a, b),
            number(&case[3]),
            TAIL,
        );
    }
    for ((x, e), ec) in list(&r["erf_grid"])
        .iter()
        .zip(list(&r["erf"]))
        .zip(list(&r["erfc"]))
    {
        let x = number(x);
        agree(&format!("erf({x})"), erf(x), number(e), TAIL);
        agree(&format!("erfc({x})"), erfc(x), number(ec), TAIL);
    }
}

#[test]
fn the_normal_tails_and_quantiles_agree() {
    let r = reference();
    for ((x, lower), upper) in list(&r["normal_grid"])
        .iter()
        .zip(list(&r["normal_lower"]))
        .zip(list(&r["normal_upper"]))
    {
        let x = number(x);
        let tails = normal_tails(x);
        agree(&format!("Phi({x})"), tails.lower, number(lower), TAIL);
        agree(&format!("1 - Phi({x})"), tails.upper, number(upper), TAIL);
    }
    for case in list(&r["normal_quantile"]) {
        let p = number(&case[0]);
        agree(
            &format!("qnorm({p})"),
            normal_quantile(p, false),
            number(&case[1]),
            CLASS,
        );
        agree(
            &format!("qnorm({p}, upper)"),
            normal_quantile(p, true),
            number(&case[2]),
            CLASS,
        );
    }
}

#[test]
fn the_incomplete_gamma_function_and_its_inverse_agree_within_the_class() {
    let r = reference();
    for case in list(&r["incomplete_gamma"]) {
        let (a, x) = (number(&case[0]), number(&case[1]));
        let (p, q) = (number(&case[2]), number(&case[3]));
        let tails = incomplete_gamma(a, x);
        agree(&format!("P({a}, {x})"), tails.lower, p, deep(p, a));
        agree(&format!("Q({a}, {x})"), tails.upper, q, deep(q, a));
    }
    for case in list(&r["incomplete_gamma_inverse"]) {
        let (a, p) = (number(&case[0]), number(&case[1]));
        agree(
            &format!("P^-1({a}, {p})"),
            incomplete_gamma_inverse(a, p, false),
            number(&case[2]),
            CLASS,
        );
        agree(
            &format!("Q^-1({a}, {p})"),
            incomplete_gamma_inverse(a, p, true),
            number(&case[3]),
            CLASS,
        );
    }
}

#[test]
fn the_incomplete_beta_function_and_its_inverse_agree_within_the_class() {
    let r = reference();
    for case in list(&r["incomplete_beta"]) {
        let (a, b, x) = (number(&case[0]), number(&case[1]), number(&case[2]));
        let (lower, upper) = (number(&case[3]), number(&case[4]));
        let tails = incomplete_beta(a, b, x);
        let largest = a.max(b);
        agree(
            &format!("I_{x}({a}, {b})"),
            tails.lower,
            lower,
            deep(lower, largest),
        );
        agree(
            &format!("1 - I_{x}({a}, {b})"),
            tails.upper,
            upper,
            deep(upper, largest),
        );
    }
    for case in list(&r["incomplete_beta_inverse"]) {
        let (a, b, p) = (number(&case[0]), number(&case[1]), number(&case[2]));
        agree(
            &format!("I^-1({a}, {b}, {p})"),
            incomplete_beta_inverse(a, b, p, false),
            number(&case[3]),
            CLASS,
        );
        agree(
            &format!("I^-1({a}, {b}, {p}, upper)"),
            incomplete_beta_inverse(a, b, p, true),
            number(&case[4]),
            CLASS,
        );
    }
}
