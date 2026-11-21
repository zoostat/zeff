//! `margins` after `reg` (catalogue row 260; open decisions #88 and #195; release 0.9.0, step
//! 11a): the average marginal effects against R on the cases of
//! `tests/reference/r/margins.json` (the formulas of `marginaleffects` written out in base R:
//! slopes through every term a variable enters, differences for a 0/1 variable and for the
//! levels of a category, delta-method standard errors under each kind of variance, weighted
//! averages under weights), the property that the effect in a model without transformed terms
//! is the coefficient, and the printed forms and every message through the engine.

mod common;

use common::{Options, call, close, fit_with, json, number, table_of, text};
use proptest::prelude::*;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::value::{Fit, TestResult, Value};
use zeff_results::margins::margins;
use zeff_testkit::run;

/// `fit.margins(vars)` through the function's body.
fn effects(fit: &Rc<Fit>, vars: &[&str]) -> Rc<TestResult> {
    let vars = Value::Vars(vars.iter().map(|v| (*v).to_string()).collect());
    match call(
        margins,
        vec![Some(Value::Fit(Rc::clone(fit))), Some(vars)],
        Vec::new(),
        Vec::new(),
    ) {
        Value::TestResult(result) => result,
        other => panic!("not a result: {other:?}"),
    }
}

#[test]
fn the_average_effects_agree_with_r() {
    let reference = json("r/margins.json");
    let wages = table_of("wages");
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 7);
    for (key, case) in cases {
        let cluster = text(&case["cluster"]);
        let weights = text(&case["weights"]);
        let options = Options {
            se: match text(&case["se"]) {
                "classical" => Some("classical"),
                "HC1" => Some("robust"),
                "HC3" => Some("hc3"),
                _ => None,
            },
            cluster: (!cluster.is_empty()).then_some(cluster),
            weights: (!weights.is_empty()).then_some(weights),
            weight_type: (!weights.is_empty()).then_some(text(&case["weight_type"])),
            level: Some(0.95),
        };
        let fit = fit_with(&wages, text(&case["formula"]), &options);
        assert_eq!(fit.n as u64, case["nobs"].as_u64().unwrap(), "{key}: N");
        assert_eq!(
            fit.df_residual as u64,
            case["df"].as_u64().unwrap(),
            "{key}: the degrees of freedom"
        );
        let vars: Vec<&str> = case["vars"].as_array().unwrap().iter().map(text).collect();
        let result = effects(&fit, &vars);
        let rows = case["rows"].as_array().unwrap();
        assert_eq!(result.terms.len(), rows.len(), "{key}: the rows");
        let table = result.table.as_ref().expect("a table");
        for (i, (row, expected)) in result.terms.iter().zip(rows).enumerate() {
            let term = text(&expected["term"]);
            assert_eq!(row.term, term, "{key}: the name of row {i}");
            // R's file writes the two values of a 0/1 variable as numbers; the shipped wages
            // carry value labels for them.
            let contrast = &table.rows[i][1];
            match text(&expected["contrast"]) {
                "1 \u{2212} 0" => assert!(
                    contrast == "female \u{2212} male" || contrast == "yes \u{2212} no",
                    "{key}, {term}: {contrast}"
                ),
                other => assert_eq!(contrast, other, "{key}, {term}: the contrast"),
            }
            let estimate = row.estimate.as_ref().expect("an estimate");
            let (lower, upper) = estimate.ci.expect("an interval");
            let checks = [
                ("estimate", estimate.value, &expected["estimate"]),
                ("std_error", row.std_error.unwrap(), &expected["std_error"]),
                ("t_value", row.statistic.value, &expected["t_value"]),
                ("ci_lower", lower, &expected["ci_lower"]),
                ("ci_upper", upper, &expected["ci_upper"]),
            ];
            for (what, ours, theirs) in checks {
                let theirs = number(theirs);
                assert!(
                    close(ours, theirs, 1e-8, 0.0),
                    "{key}, {term}: {what} {ours:e} against R's {theirs:e}"
                );
            }
            // A p-value far in the tail is compared on its own scale.
            let p = number(&expected["p_value"]);
            assert!(
                close(row.p_value, p, 1e-6, 1e-300),
                "{key}, {term}: p {:e} against R's {p:e}",
                row.p_value
            );
            assert_eq!(row.statistic.df, vec![fit.df_residual as f64]);
        }
    }
}

// ----- the effect in a model without transformed terms is the coefficient -----

/// A frame of `n` rows: two numbers, a 0/1 variable, a category of three levels and an outcome.
fn generated(values: &[(f64, f64, bool, u8, f64)]) -> Table {
    let levels = ["a", "b", "c"];
    Table::new(vec![
        (
            "y".to_string(),
            Column::floats(values.iter().map(|v| Some(v.4))).0,
        ),
        (
            "x1".to_string(),
            Column::floats(values.iter().map(|v| Some(v.0))).0,
        ),
        (
            "x2".to_string(),
            Column::floats(values.iter().map(|v| Some(v.1))).0,
        ),
        (
            "d".to_string(),
            Column::ints(values.iter().map(|v| Some(i64::from(v.2)))),
        ),
        (
            "g".to_string(),
            Column::categories(
                levels.iter().map(|l| (*l).to_string()).collect(),
                values.iter().map(|v| Some(i32::from(v.3))),
            )
            .expect("codes within the levels"),
        ),
    ])
    .expect("columns of one length")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// With every term a variable as it stands, the average effect of a number is its
    /// coefficient, that of a 0/1 variable its coefficient, and that of a level the level's
    /// coefficient, each with the coefficient's standard error.
    #[test]
    fn without_transformed_terms_the_effect_is_the_coefficient(
        rows in prop::collection::vec(
            (-5.0..5.0f64, 0.0..10.0f64, any::<bool>(), 0u8..3, -3.0..3.0f64),
            24..60,
        ),
        robust in any::<bool>(),
    ) {
        // Every level and both values of `d` must be there, or a term is omitted.
        prop_assume!((0..3).all(|l| rows.iter().filter(|r| r.3 == l).count() >= 3));
        prop_assume!(rows.iter().filter(|r| r.2).count() >= 3);
        prop_assume!(rows.iter().filter(|r| !r.2).count() >= 3);
        let table = generated(&rows);
        let options = Options {
            se: Some(if robust { "robust" } else { "classical" }),
            level: Some(0.95),
            ..Default::default()
        };
        let fit = fit_with(&table, "y ~ x1 + x2 + d + g", &options);
        prop_assume!(fit.omitted.is_empty());
        let result = effects(&fit, &["x1", "x2", "d", "g"]);
        prop_assert_eq!(result.terms.len(), 5);
        for row in &result.terms {
            let at = fit.terms.iter().position(|t| *t == row.term).expect("a term of the fit");
            let estimate = row.estimate.as_ref().unwrap().value;
            prop_assert!(
                close(estimate, fit.estimates[at], 1e-12, 1e-12),
                "{}: {estimate:e} against the coefficient {:e}", row.term, fit.estimates[at]
            );
            let std_error = row.std_error.unwrap();
            prop_assert!(
                close(std_error, fit.std_errors[at], 1e-12, 1e-12),
                "{}: {std_error:e} against the coefficient's {:e}", row.term, fit.std_errors[at]
            );
        }
    }
}

// ----- the printed forms and every message, through the engine -----

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

const W: &str = "let wages = dataset(wages)\n";

#[test]
fn the_printed_forms_and_every_message() {
    let scripts = [
        // The forms: a quadratic, a 0/1 variable with value labels and a category, under each
        // kind of standard error; one variable; the result read as any result is.
        format!(
            "{W}let m = wages.reg(log(wage) ~ educ + exper + exper^2 + female + industry, se: robust)\nm.margins([educ, exper, female, industry])\nm.margins(exper)\nlet e = m.margins([exper, industry])\ne.estimates()\ne.ci()\ne.p_values(\"industry=Mining\")\ne.p_values(exper)\ne.n_obs()\ntype_of(e)\n"
        ),
        format!(
            "{W}let c = wages.reg(log(wage) ~ educ * female + exper, cluster: worker, level: 0.9)\nc.margins([educ, female])\nwages.reg(log(wage) ~ educ * female + exper).margins([female, educ, female])\n"
        ),
        // Weights, and fixed effects: the effects do not need the levels of the effects.
        format!(
            "{W}wages.reg(log(wage) ~ educ * female + exper, weights: hours).margins([educ, female])\nwages.reg(log(wage) ~ exper + exper^2 + female, weights: educ, weight_type: frequency).margins(exper)\nwages.reg(log(wage) ~ educ + exper + exper^2 + female, fe: firm).margins([exper, female])\nwages.reg(log(wage) ~ educ + exper + exper^2 + female + category(firm)).margins([exper, female])\n"
        ),
        // A Bool, a 0/1 variable without labels, a power computed beforehand, which stays a
        // regressor of its own, and a hinge, whose slope is a central difference.
        format!(
            "{W}let w = wages.add(long = hours > 40, f = to(female, Int) * 1, exper2 = exper^2)\nw.reg(log(wage) ~ educ + long + f).margins([long, f])\nw.reg(log(wage) ~ educ + exper + exper2).margins(exper)\nw.reg(log(wage) ~ educ + hours + max(hours - 40, 0)).margins(hours)\n"
        ),
        // A term omitted as collinear: the effect that rests on it is not estimable.
        format!(
            "{W}let w = wages.add(twice = educ * 2)\nlet m = w.reg(log(wage) ~ educ + twice + female)\nm.margins([educ, twice, female])\n"
        ),
        // The refusals.
        format!("{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins(hours)\n"),
        format!("{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins(expr)\n"),
        format!("{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins()\n"),
        format!("{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins(\"educ\")\n"),
        format!("{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins([])\n"),
        format!(
            "{W}let m = wages.reg(log(wage) ~ educ + category(educ) + female)\nm.margins(educ)\n"
        ),
        format!(
            "{W}let m = wages.reg(log(wage) ~ educ + sqrt(exper) + female)\nm.margins(exper)\n"
        ),
        format!(
            "{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.test(educ == 0).margins(educ)\n"
        ),
        format!("{W}let g = wages.glm(union ~ educ + exper, family: binomial)\ng.margins(educ)\n"),
        format!(
            "{W}let v = wages.ivreg(log(wage) ~ educ + exper, instruments: educ ~ age)\nv.margins(exper)\n"
        ),
        format!(
            "{W}let m = wages.reg(log(wage) ~ educ + exper + female)\nm.margins(educ).p_values(exper)\n"
        ),
    ];
    let mut out = String::new();
    for script in &scripts {
        out.push_str(&format!(
            "=== {} ===\n{}\n",
            script.trim_end(),
            printed(script)
        ));
    }
    insta::assert_snapshot!(out);
}
