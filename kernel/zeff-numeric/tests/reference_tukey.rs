//! The studentized range against `tests/reference/tukey.json` (Python's `decimal` module at 70
//! digits, rounded once; testing handbook, section 1.1): both tails, each from its own integral,
//! and the density, within the class tolerance of the distribution functions, 1e-12 relative
//! (Engine and Project Specification, section 13), from 2 to 200 groups, on 1 to 1e12 degrees of
//! freedom and down to a tail of 4.6e-99; and the quantile, which must return the point of each
//! reference probability.
//!
//! R is the second check, from `tests/reference/r/tukey.json`, and only at R's own accuracy:
//! `ptukey` sums fixed Gauss-Legendre rules and is out by up to 5e-3 on 2 degrees of freedom,
//! 3e-5 on 5 and 2e-6 from 20 upwards (measured against the exact reference), and `qtukey` stops
//! a secant search at about four digits (its grid keeps to 5 or more degrees of freedom and to
//! the median and above, where the search converges). Where `ptukey` answers NA (fewer than 2
//! degrees of freedom) there is nothing to compare.

use serde_json::Value as Json;
use std::path::Path;
use zeff_numeric::tukey;

fn read(file: &str) -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/reference")
        .join(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{file} is JSON: {e}"))
}

fn number(json: &Json) -> f64 {
    match json {
        Json::String(s) => match s.as_str() {
            "NA" => f64::NAN,
            other => other
                .parse()
                .unwrap_or_else(|_| panic!("not a number: {other}")),
        },
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other}"),
    }
}

fn list(json: &Json) -> &Vec<Json> {
    json.as_array().expect("a list")
}

/// One case of either file: the tail or the density it names, where it is taken, and the value.
struct Case<'a> {
    kind: &'a str,
    q: f64,
    groups: f64,
    df: f64,
    value: f64,
}

fn case(json: &Json) -> Case<'_> {
    Case {
        kind: json["kind"].as_str().expect("a kind"),
        q: number(&json["q"]),
        groups: number(&json["groups"]),
        df: number(&json["df"]),
        value: number(&json["value"]),
    }
}

fn ours(c: &Case) -> f64 {
    match c.kind {
        "lower" => tukey::tails(c.q, c.groups, c.df).lower,
        "upper" => tukey::tails(c.q, c.groups, c.df).upper,
        "density" => tukey::density(c.q, c.groups, c.df),
        other => panic!("unknown kind {other}"),
    }
}

const CLASS: f64 = 1e-12;

#[test]
fn both_tails_and_the_density_agree_with_the_exact_reference() {
    let reference = read("tukey.json");
    let cases = list(&reference["cases"]);
    assert!(cases.len() >= 279, "only {} cases", cases.len());
    for c in cases.iter().map(case) {
        let got = ours(&c);
        let error = ((got - c.value) / c.value).abs();
        assert!(
            error <= CLASS,
            "{} at {} on {} groups and {} degrees of freedom: {got:e} against {:e}, relative \
             error {error:e}",
            c.kind,
            c.q,
            c.groups,
            c.df,
            c.value
        );
    }
}

/// A quantile costs some twenty values of the distribution, so the round trip is made on every
/// tail below one in a thousand and on one in six of the other cases. A probability above 0.9 is
/// left out: its quantile is found from the other tail, whose reference value is a different
/// case.
#[test]
fn the_quantile_returns_the_point_of_each_reference_probability() {
    let reference = read("tukey.json");
    let eligible = list(&reference["cases"])
        .iter()
        .map(case)
        .filter(|c| c.kind != "density" && c.value <= 0.9);
    let mut tried = 0;
    for (i, c) in eligible.enumerate() {
        if i % 6 != 0 && c.value >= 1e-3 {
            continue;
        }
        tried += 1;
        let back = tukey::quantile(c.value, c.kind == "upper", c.groups, c.df);
        let error = ((back - c.q) / c.q).abs();
        assert!(
            error <= CLASS,
            "the {} probability {:e} on {} groups and {} degrees of freedom: {back:e} against \
             {:e}, relative error {error:e}",
            c.kind,
            c.value,
            c.groups,
            c.df,
            c.q
        );
    }
    assert!(tried >= 40, "only {tried} quantiles tried");
}

/// What R's `ptukey` can be held to, as an absolute error, by the degrees of freedom.
fn r_accuracy(df: f64) -> f64 {
    if df < 5.0 {
        1e-2
    } else if df < 20.0 {
        1e-4
    } else {
        1e-5
    }
}

#[test]
fn r_agrees_within_its_own_accuracy() {
    let r = read("r/tukey.json");
    let mut compared = 0;
    for c in list(&r["ptukey"]).iter().map(case) {
        if c.value.is_nan() {
            continue;
        }
        compared += 1;
        let got = ours(&c);
        let error = (got - c.value).abs();
        assert!(
            error <= r_accuracy(c.df),
            "{} at {} on {} groups and {} degrees of freedom: {got:e} against R's {:e}, \
             absolute error {error:e}",
            c.kind,
            c.q,
            c.groups,
            c.df,
            c.value
        );
    }
    assert!(compared >= 190, "only {compared} values compared with R");

    let quantiles = list(&r["qtukey"]);
    assert!(quantiles.len() >= 36, "only {} quantiles", quantiles.len());
    for entry in quantiles {
        let (p, groups, df) = (
            number(&entry["p"]),
            number(&entry["groups"]),
            number(&entry["df"]),
        );
        let want = number(&entry["value"]);
        let got = tukey::quantile(p, false, groups, df);
        let error = ((got - want) / want).abs();
        assert!(
            error <= 1e-4,
            "the quantile of {p} on {groups} groups and {df} degrees of freedom: {got:e} against \
             R's {want:e}, relative error {error:e}"
        );
    }
}
