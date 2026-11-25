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
use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::value::{Fit, TestResult, Value};
use zeff_results::margins::margins;
use zeff_testkit::run;

/// `fit.margins(vars)` through the function's body.
fn effects(fit: &Rc<Fit>, vars: &[&str]) -> Rc<TestResult> {
    effects_at(fit, vars, None, "ame")
}

/// `fit.margins(vars, at:, type:)` through the function's body; no variable asks for the
/// adjusted prediction.
fn effects_at(fit: &Rc<Fit>, vars: &[&str], at: Option<Value>, kind: &str) -> Rc<TestResult> {
    let vars =
        (!vars.is_empty()).then(|| Value::Vars(vars.iter().map(|v| (*v).to_string()).collect()));
    match call(
        margins,
        vec![
            Some(Value::Fit(Rc::clone(fit))),
            vars,
            at,
            Some(Value::Word(kind.to_string())),
        ],
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
    assert_eq!(cases.len(), 20);
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
        // The values of `at:`: a level as text, a number as R wrote it; first the variables
        // given several values, each as a list, then those held at one.
        let value_of = |written: &str| -> Value {
            match written.parse::<f64>() {
                Ok(x) if x == x.trunc() => Value::Int(x as i64),
                Ok(x) => Value::Float(x),
                Err(_) => Value::Str(written.to_string()),
            }
        };
        let mut at: Vec<(String, Value)> = Vec::new();
        if let Some(grid) = case.get("grid").and_then(Json::as_object) {
            for (name, values) in grid {
                let values = values.as_array().unwrap().iter().map(|v| value_of(text(v)));
                at.push((name.clone(), Value::List(values.collect())));
            }
        }
        if let Some(fields) = case["at"].as_object() {
            for (name, value) in fields {
                at.push((name.clone(), value_of(text(value))));
            }
        }
        let result = effects_at(
            &fit,
            &vars,
            (!at.is_empty()).then_some(Value::Record(at)),
            text(&case["type"]),
        );
        let rows = case["rows"].as_array().unwrap();
        assert_eq!(result.terms.len(), rows.len(), "{key}: the rows");
        let table = result.table.as_ref().expect("a table");
        for (i, (row, expected)) in result.terms.iter().zip(rows).enumerate() {
            let term = text(&expected["term"]);
            assert_eq!(row.term, term, "{key}: the name of row {i}");
            // R's file writes the two values of a 0/1 variable as numbers; the shipped wages
            // carry value labels for them.
            // The contrast stands after the columns of the variables given several values.
            let leading = case
                .get("grid")
                .and_then(Json::as_object)
                .map_or(0, |g| g.len());
            let contrast = if vars.is_empty() {
                ""
            } else {
                &table.rows[i][leading + 1]
            };
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Where every effect is linear in each of the other variables (interactions of two
    /// variables, a square), the effect at the means is the average effect: the mean of a
    /// linear function is the function of the means, a category's share being the mean of its
    /// indicator. This holds the shares of `type: mem` without R.
    #[test]
    fn at_the_means_is_the_average_where_effects_are_linear(
        rows in prop::collection::vec(
            (-5.0..5.0f64, 0.0..10.0f64, any::<bool>(), 0u8..3, -3.0..3.0f64),
            40..70,
        ),
    ) {
        prop_assume!((0..3).all(|l| rows.iter().filter(|r| r.3 == l).count() >= 6));
        prop_assume!(rows.iter().filter(|r| r.2).count() >= 6);
        prop_assume!(rows.iter().filter(|r| !r.2).count() >= 6);
        let table = generated(&rows);
        let options = Options { se: Some("robust"), level: Some(0.95), ..Default::default() };
        let fit = fit_with(&table, "y ~ x1 * d + x2 + x2^2 + g + g:x1", &options);
        prop_assume!(fit.omitted.is_empty());
        let vars = ["x1", "x2", "d", "g"];
        let average = effects_at(&fit, &vars, None, "ame");
        let at_means = effects_at(&fit, &vars, None, "mem");
        prop_assert_eq!(average.terms.len(), at_means.terms.len());
        for (a, m) in average.terms.iter().zip(&at_means.terms) {
            prop_assert_eq!(&a.term, &m.term);
            let (ea, em) = (a.estimate.as_ref().unwrap().value, m.estimate.as_ref().unwrap().value);
            prop_assert!(close(ea, em, 1e-9, 1e-11), "{}: average {ea:e}, at the means {em:e}", a.term);
            let (sa, sm) = (a.std_error.unwrap(), m.std_error.unwrap());
            prop_assert!(close(sa, sm, 1e-9, 1e-11), "{}: std errors {sa:e} and {sm:e}", a.term);
        }
        // The average prediction is the mean of the fitted values.
        let prediction = effects_at(&fit, &[], None, "ame");
        let mean = fit.fitted.iter().sum::<f64>() / fit.fitted.len() as f64;
        let ours = prediction.terms[0].estimate.as_ref().unwrap().value;
        prop_assert!(close(ours, mean, 1e-10, 1e-11), "prediction {ours:e}, mean fitted {mean:e}");
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

/// `at:`, `type: mem` and the adjusted prediction (step 11b).
#[test]
fn held_values_the_means_and_predictions() {
    const M: &str = "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ * female + exper + exper^2 + industry + industry:exper, se: robust)\n";
    let scripts = [
        // The forms: variables held, the others as observed; at the means; both.
        format!(
            "{M}m.margins([exper, female, educ], at: {{exper: 10, educ: 12}})\nm.margins([educ, female], at: {{industry: \"Mining\"}})\nm.margins([educ, exper, female, industry], type: mem)\nm.margins([exper, female], type: mem, at: {{exper: 10.5, industry: \"Finance\"}})\n"
        ),
        // The adjusted prediction: the average, at values, at the means; read as a result.
        format!(
            "{M}m.margins()\nm.margins(at: {{exper: 10, industry: \"Mining\", female: 1}})\nm.margins(type: mem)\nm.margins(type: mem, at: {{educ: 16}})\nlet p = m.margins(at: {{educ: 16}})\np.estimates()\np.ci()\np.p_values(prediction)\n"
        ),
        // Where the effect at the means is not the average: a logarithm; a hinge at the means;
        // weights, which weight the means; a Bool held; a category made in the formula held.
        format!(
            "{W}let h = wages.reg(log(wage) ~ log(hours) + educ)\nh.margins(hours)\nh.margins(hours, type: mem)\nh.margins(hours, at: {{hours: 40}})\nwages.reg(log(wage) ~ educ + hours + max(hours - 40, 0)).margins(hours, type: mem)\nwages.reg(log(wage) ~ educ * female + exper + exper^2, weights: hours).margins([educ, exper], type: mem)\nwages.reg(log(wage) ~ exper + exper^2 + female + female:exper, weights: educ, weight_type: frequency).margins(at: {{female: 0}})\nlet w = wages.add(long = hours > 40)\nw.reg(log(wage) ~ educ * long).margins(educ, at: {{long: true}})\nw.reg(log(wage) ~ educ * long).margins(educ, type: mem)\nwages.reg(log(wage) ~ category(educ) * female).margins(female, at: {{educ: 12}})\n"
        ),
        // After fixed effects: effects as before, held and at the means; a term omitted as
        // collinear counts as nought in a prediction.
        format!(
            "{W}let f = wages.reg(log(wage) ~ educ * female + exper + exper^2, fe: firm)\nf.margins([exper, female], at: {{educ: 12}})\nf.margins([exper, female], type: mem)\nlet c = wages.add(twice = educ * 2).reg(log(wage) ~ educ + twice + female)\nc.margins()\n"
        ),
        // The refusals.
        format!("{M}m.margins(exper, at: {{hours: 40}})\n"),
        format!("{M}m.margins(exper, at: {{expr: 10}})\n"),
        format!("{M}m.margins(exper, at: {{exper: \"ten\"}})\n"),
        format!("{M}m.margins(exper, at: {{industry: \"Minin\"}})\n"),
        format!("{M}m.margins(exper, at: {{industry: 3}})\n"),
        // Several values: the effects at each, and at each combination; a prediction at each.
        format!(
            "{M}m.margins(female, at: {{exper: [5, 10]}})\nm.margins([exper, female], at: {{exper: [5, 20], industry: [\"Mining\", \"Finance\"], educ: 12}})\nm.margins(at: {{exper: [5, 10.5, 20]}})\nm.margins(educ, type: mem, at: {{female: [0, 1]}})\nlet e = m.margins(at: {{exper: [5, 20]}})\ne.estimates()\ne.p_values(\"prediction @ exper=20\")\ne.ci()\n"
        ),
        format!("{M}m.margins(female, at: {{exper: [5, 5]}})\n"),
        format!("{M}m.margins(female, at: {{exper: []}})\n"),
        format!("{M}m.margins(female, at: {{exper: [[5, 10], [20]]}})\n"),
        format!("{M}m.margins(female, at: {{industry: [\"Mining\", \"Minin\"]}})\n"),
        format!("{M}m.margins(exper, at: 10)\n"),
        format!("{M}m.margins(female, at: {{female: 1}})\n"),
        format!("{M}m.margins(industry, at: {{industry: \"Mining\"}})\n"),
        format!("{M}m.margins(exper, type: median)\n"),
        format!("{W}wages.reg(log(wage) ~ educ + exper, fe: firm).margins()\n"),
        format!("{W}wages.reg(log(wage) ~ educ + exper, fe: firm).margins(at: {{educ: 12}})\n"),
        format!("{W}wages.reg(log(wage) ~ log(hours) + educ).margins(educ, at: {{hours: 0}})\n"),
        format!(
            "{W}wages.reg(log(wage) ~ educ + category(educ) + female).margins(female, type: mem)\n"
        ),
        format!(
            "{W}wages.reg(log(wage) ~ educ + category(educ) + female).margins(female, at: {{educ: 12}})\n"
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

/// The result of `margins` read by row, set beside fits by `report`, and exported (step 13).
#[test]
fn the_result_read_by_row_reported_and_exported() {
    const M: &str = "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + exper^2 + female + industry, se: robust)\nlet a = m.margins([educ, exper, female])\n";
    let scripts = [
        // A row's standard error and interval by its name; without a name, nothing of the
        // whole result.
        format!(
            "{M}a.std_errors(exper)\na.ci(exper)\na.std_errors()\na.p_values()\nlet i = m.margins(industry)\ni.std_errors(\"industry=Mining\")\ni.ci(\"industry=Mining\")\nlet p = m.margins(at: {{exper: [5, 20]}})\np.std_errors(\"prediction @ exper=20\")\n"
        ),
        // In `report`: the effects beside their fit; two results side by side with a title,
        // labels and digits; effects alone, held values in the footer; a row not estimable.
        format!(
            "{M}report([m, a])\nlet c = wages.reg(log(wage) ~ educ * female + exper, cluster: worker)\nreport([a, c.margins([educ, female, exper])], title: \"Average effects on log wages\", digits: 4, labels: {{exper: \"Experience\", female: \"Female\"}})\nreport([m.margins([exper, female], at: {{industry: \"Mining\", exper: [5, 20]}})], stars: false)\nreport([m.margins()], stats: [n_obs, r_squared, aic])\nlet t = wages.add(twice = educ * 2).reg(log(wage) ~ educ + twice + female)\nreport([t, t.margins([educ, twice, female])])\n"
        ),
        // Exported as any result is.
        format!(
            "{M}a.export(\"output/margins.tex\")\na.export(\"output/margins.docx\")\na.export(\"output/margins.typ\")\nreport([m, a]).export(\"output/both.tex\")\n"
        ),
        // The refusals: a row the result does not have; an interval at another level; a test
        // among effects.
        format!("{M}a.std_errors(hours)\n"),
        format!("{M}a.ci(exper, level: 0.9)\n"),
        format!("{M}report([a, m.test(educ == 0)])\n"),
        format!("{M}a.coef(exper)\n"),
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

// ----- places no test reached (release 0.9.0, step 42c2) -----

/// Slopes through terms the differentiator had not been sent through by any test: a minus
/// sign before the variable, inside a function and under a power, which are differentiated
/// term by term, and `min` and a logarithm to a base, which are differenced. Against R 4.6.0
/// on the shipped wages, the effect written out from each term's derivative on an `lm` fit
/// (`mean(d) * b` with the delta-method standard error): −exp(−x/10)/10; 2x; one below 20,
/// a half at 20 and nought above, which is what a central difference gives at the corner;
/// and 1/((x + 1) log 10).
#[test]
fn slopes_through_a_minus_sign_a_corner_and_a_logarithm_to_a_base_agree_with_r() {
    let wages = table_of("wages");
    let cases = [
        (
            "log(wage) ~ educ + exp(-exper / 10)",
            0.022716660679,
            0.000453640706,
            1e-9,
        ),
        (
            "log(wage) ~ educ + (-exper)^2",
            0.0187508727231,
            0.0004411905913,
            1e-9,
        ),
        (
            "log(wage) ~ educ + min(exper, 20)",
            0.0203061840118,
            0.0004062558075,
            1e-7,
        ),
        (
            "log(wage) ~ educ + log(exper + 1, 10)",
            0.025726274184,
            0.000525196698,
            1e-7,
        ),
    ];
    for (formula, estimate, std_error, tolerance) in cases {
        let fit = fit_with(
            &wages,
            formula,
            &Options {
                level: Some(0.95),
                ..Options::default()
            },
        );
        assert_eq!(fit.n, 18_179, "{formula}");
        let result = effects(&fit, &["exper"]);
        let row = &result.terms[0];
        let ours = row.estimate.as_ref().expect("an estimate").value;
        assert!(
            close(ours, estimate, tolerance, 0.0),
            "{formula}: {ours:e} against R's {estimate:e}"
        );
        let ours = row.std_error.expect("a standard error");
        assert!(
            close(ours, std_error, tolerance, 0.0),
            "{formula}: standard error {ours:e} against R's {std_error:e}"
        );
    }
}

/// What the coverage report of the release's code showed that no test printed: `at:` with a
/// number that is not finite; a category held at a number, at a Bool and at a record; two
/// effects that rest on omitted terms, said in the plural; the interval of an effect that is
/// not estimable, which is `NA`; the slope through a term dropped as nought; and the slopes of
/// the test above with the lines that say how each was taken.
#[test]
fn the_forms_the_coverage_report_showed_unprinted() {
    const M: &str =
        "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + industry)\n";
    let scripts = [
        format!("{M}m.margins(educ, at: {{exper: exp(1000)}})\n"),
        format!("{M}m.margins(educ, at: {{industry: 2.5}})\n"),
        format!("{M}m.margins(educ, at: {{industry: true}})\n"),
        format!("{M}m.margins(educ, at: {{industry: {{a: 1}}}})\n"),
        format!(
            "{W}let w = wages.add(twice = educ * 2, thrice = exper * 3)\nw.reg(log(wage) ~ educ + exper + twice + thrice).margins([twice, thrice])\n"
        ),
        // An effect that is not estimable has no interval, and a term that is nought in
        // every row is dropped: the slope through it is nought, with nothing to test.
        format!(
            "{W}let w = wages.add(twice = educ * 2)\nlet e = w.reg(log(wage) ~ educ + twice).margins([educ, twice])\nprint(e.ci(twice))\nprint(e.std_errors(twice))\nwages.reg(log(wage) ~ educ + abs(exper - exper)).margins(exper)\n"
        ),
        // A 0/1 variable of which one value alone is labelled: the contrast is `1 − 0`.
        format!(
            "{W}let w = wages.add(one = to(union, Int) + 0).label(one, [1 => \"member\"])\nw.reg(log(wage) ~ educ + one).margins(one)\n"
        ),
        format!(
            "{W}wages.reg(log(wage) ~ educ + exp(-exper / 10)).margins(exper)\nwages.reg(log(wage) ~ educ + (-exper)^2).margins(exper)\nwages.reg(log(wage) ~ educ + min(exper, 20)).margins(exper)\nwages.reg(log(wage) ~ educ + log(exper + 1, 10)).margins(exper)\n"
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
