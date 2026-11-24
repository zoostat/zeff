//! Odds ratios: `fit.summary(exponentiate: true)` on a `glm` fit with a logit link (release
//! 0.9.0, step 8d; open decision #217). The estimate is exp(b); its standard error exp(b) × se,
//! by the delta method, as Stata's `logit, or`; its interval the interval of b with each end
//! exponentiated; z and p unchanged. Held to R: `exp(coef(m))`, `exp(coef(m)) * se` and
//! `exp(confint.default(m))`, the Wald interval that `ci()` of a `glm` fit is, from R's own
//! coefficients and variance matrix of `tests/reference/r/binary.json`.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

/// The 97.5th percentile of the standard normal, as `qnorm(0.975)` gives it.
const Z975: f64 = 1.959963984540054;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The rows of a printed summary by term: the estimate (or odds ratio), its standard error, z,
/// the p-value as printed, and the two ends of the interval.
fn rows(printed: &str) -> Vec<(String, f64, f64, String, String, f64, f64)> {
    let mut found = Vec::new();
    for line in printed.lines() {
        let Some((cells, interval)) = line.split_once('[') else {
            continue;
        };
        let cells: Vec<&str> = cells.split_whitespace().collect();
        if cells.len() < 5 {
            continue;
        }
        let ends: Vec<f64> = interval
            .trim_end_matches(']')
            .split(", ")
            .map(|end| end.trim().parse().unwrap())
            .collect();
        let n = cells.len();
        found.push((
            cells[..n - 4].join(" "),
            cells[n - 4].parse().unwrap(),
            cells[n - 3].parse().unwrap(),
            cells[n - 2].to_string(),
            cells[n - 1].to_string(),
            ends[0],
            ends[1],
        ));
    }
    found
}

fn number(value: &Json) -> f64 {
    match value {
        Json::Number(n) => n.as_f64().unwrap(),
        Json::String(s) => s.parse().unwrap(),
        other => panic!("not a number: {other}"),
    }
}

const FORMULA: &str = "union ~ educ + exper + female";

#[test]
fn the_odds_ratios_are_r_s_exponentiated_coefficients_and_wald_interval() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let case = json["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["link"] == "logit" && c["name"].as_str().is_some_and(|n| n.starts_with("wages"))
        })
        .expect("R's logit on the shipped wages");
    let (code, stdout, stderr) = go(&format!(
        "dataset(wages).glm({FORMULA}, family: binomial).summary(exponentiate: true)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("  term          odds ratio   std.error"),
        "{stdout}"
    );
    assert!(
        stdout
            .contains("odds ratios: exp of the coefficients, standard errors by the delta method"),
        "{stdout}"
    );
    let ours = rows(&stdout);
    let coefficients = case["coefficients"].as_array().unwrap();
    assert_eq!(ours.len(), coefficients.len(), "{stdout}");
    for (j, (term, ratio, error, _, _, lower, upper)) in ours.iter().enumerate() {
        let b = number(&coefficients[j]);
        let se = number(&case["vcov"][j][j]).sqrt();
        // Four decimals are printed: half a unit of the last, and a little for R's own rounding.
        let close = |printed: f64, r: f64, what: &str| {
            assert!(
                (printed - r).abs() <= 0.000_051,
                "{term} {what}: Zeff {printed}, R {r}"
            );
        };
        close(*ratio, b.exp(), "odds ratio, exp(coef)");
        close(*error, b.exp() * se, "standard error, exp(b) × se");
        close(
            *lower,
            (b - Z975 * se).exp(),
            "lower end, exp(confint.default)",
        );
        close(
            *upper,
            (b + Z975 * se).exp(),
            "upper end, exp(confint.default)",
        );
    }
}

#[test]
fn z_and_p_are_those_of_the_coefficients_and_coefficients_stay_the_default() {
    let (code, stdout, stderr) = go(&format!(
        "let m = dataset(wages).glm({FORMULA}, family: binomial)\nm.summary()\nprint(\"----\")\nm.summary(exponentiate: true)\nprint(\"----\")\nm.summary(exponentiate: false)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    let parts: Vec<&str> = stdout.split("----\n").collect();
    let (plain, ratios, off) = (rows(parts[0]), rows(parts[1]), rows(parts[2]));
    assert!(parts[0].contains("estimate") && !parts[0].contains("odds ratio"));
    // Without the option, and with it false, the summary is the fit's own.
    assert_eq!(parts[0], parts[2]);
    assert_eq!(plain, off);
    assert_eq!(plain.len(), 4);
    for (b, r) in plain.iter().zip(&ratios) {
        assert_eq!(b.0, r.0);
        // z and p, as printed, are unchanged.
        assert_eq!((&b.3, &b.4), (&r.3, &r.4), "{}", b.0);
        // The ratio is positive and lies inside its interval, which is not symmetric about it.
        assert!(r.1 > 0.0 && r.5 <= r.1 && r.1 <= r.6, "{r:?}");
        // A coefficient above zero is a ratio above one, and below zero below one.
        assert_eq!(b.1 > 0.0, r.1 > 1.0, "{}", b.0);
    }
}

#[test]
fn it_follows_the_fit_s_own_errors_and_level() {
    // Robust errors and a 90 per cent interval: the same rule on the fit's own figures.
    let (code, stdout, stderr) = go(&format!(
        "let m = dataset(wages).glm({FORMULA}, family: binomial, se: robust, level: 0.9)\nm.summary()\nprint(\"----\")\nm.summary(exponentiate: true, fit_measures: true)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    let parts: Vec<&str> = stdout.split("----\n").collect();
    assert!(
        parts[1].contains("90% CI") && parts[1].contains("BIC"),
        "{stdout}"
    );
    for (b, r) in rows(parts[0]).iter().zip(rows(parts[1])) {
        // From the printed coefficient, to the digits its four decimals leave.
        assert!(
            (r.1 - b.1.exp()).abs() <= 0.0002 * b.1.exp().max(1.0),
            "{}",
            b.0
        );
        assert!(
            (r.5 - b.5.exp()).abs() <= 0.0002 * b.5.exp().max(1.0),
            "{}",
            b.0
        );
        assert!(
            (r.6 - b.6.exp()).abs() <= 0.0002 * b.6.exp().max(1.0),
            "{}",
            b.0
        );
        assert!((r.2 - b.1.exp() * b.2).abs() <= 0.0002, "{}", b.0);
    }
    insta::assert_snapshot!("odds_ratios_robust", parts[1]);
}

#[test]
fn it_is_refused_where_the_coefficients_are_not_logarithms_of_a_ratio() {
    let mut all = String::new();
    for script in [
        "dataset(wages).glm(union ~ educ, family: binomial, link: probit).summary(exponentiate: true)\n",
        "dataset(wages).reg(log(wage) ~ educ).summary(exponentiate: true)\n",
        "dataset(staff).ivreg(productivity ~ training + age, instruments: training ~ offer).summary(exponentiate: true)\n",
        "dataset(sleep).t_test(extra, group: group).summary(exponentiate: true)\n",
        // `standardized:` is refused on a `glm` fit as before, whatever else is asked.
        "dataset(wages).glm(union ~ educ, family: binomial).summary(standardized: true, exponentiate: true)\n",
    ] {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}{stdout}{stderr}");
        assert!(stderr.contains("error[Z0401]"), "{script}{stderr}");
        all.push_str(&format!("{script}{stderr}\n====\n"));
    }
    insta::assert_snapshot!("odds_ratios_refused", all);
}

#[test]
fn an_omitted_term_keeps_its_row() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).add(twice = educ * 2)\nw.glm(union ~ educ + twice + exper, family: binomial).summary(exponentiate: true)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("(omitted)"), "{stdout}");
    assert_eq!(rows(&stdout).len(), 3, "{stdout}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// On any year of the wages and at any usual level: the ratio lies in its interval, its
    /// interval is the coefficient's exponentiated, and z and p do not move.
    #[test]
    fn the_exponentiated_table_is_the_coefficients_table_carried_to_ratios(year in 2010i64..=2021, level in prop_oneof![Just(0.9), Just(0.95), Just(0.99)]) {
        let (code, stdout, stderr) = go(&format!(
            "let m = dataset(wages).where(year == {year}).glm(union ~ educ + female, family: binomial, level: {level})\nm.summary()\nprint(\"----\")\nm.summary(exponentiate: true)\n"
        ));
        prop_assert_eq!(code, 0, "{}", stderr);
        let parts: Vec<&str> = stdout.split("----\n").collect();
        let (plain, ratios) = (rows(parts[0]), rows(parts[1]));
        prop_assert_eq!(plain.len(), 3);
        for (b, r) in plain.iter().zip(&ratios) {
            prop_assert_eq!((&b.3, &b.4), (&r.3, &r.4));
            prop_assert!(r.5 <= r.1 && r.1 <= r.6);
            prop_assert!((r.5 - b.5.exp()).abs() <= 0.0002 * b.5.exp().max(1.0));
            prop_assert!((r.6 - b.6.exp()).abs() <= 0.0002 * b.6.exp().max(1.0));
        }
    }
}
