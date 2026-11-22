//! `margins` after `glm` (catalogue row 260; open decisions #88, #195 and #202; release 0.9.0,
//! step 12a): the effects and the adjusted predictions on the scale of the probability, against
//! R on the cases of `tests/reference/r/margins_binary.json` (the formulas of `marginaleffects`
//! written out in base R on fits iterated to their maximum: slopes, differences, predictions,
//! averaged and at the means, with variables held, under each kind of variance and under
//! weights), and the printed forms through the engine.

use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_results::margins::margins;
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The options of a case: the kind of error, the cluster and the weights.
#[derive(Default)]
struct With<'a> {
    se: Option<&'a str>,
    cluster: Option<&'a str>,
    weights: Option<&'a str>,
    weight_type: Option<&'a str>,
}

/// A body called with `values`, in a context of its own.
fn call(body: zeff_core::runtime::Body, script: &str, values: Vec<Option<Value>>) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", script));
    let spans = vec![Some(Span::new(0, 1)); values.len()];
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(0, 1),
        arg_spans: spans,
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
        width: zeff_core::display::PAGE_WIDTH,
        shown: None,
    };
    match body(
        &mut ctx,
        Args {
            values,
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    ) {
        Ok(value) => value,
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// `glm` through its body: data, formula, family, link, fe, se, cluster, weights, weight_type,
/// level.
fn fit(table: &Table, text: &str, link: &str, with: &With<'_>) -> Rc<Fit> {
    let values = vec![
        Some(Value::Frame(Frame::new(table.clone()))),
        Some(Value::Formula(formula(text))),
        Some(Value::Word("binomial".to_string())),
        Some(Value::Word(link.to_string())),
        None,
        with.se.map(|word| Value::Word(word.to_string())),
        with.cluster.map(|name| Value::Var(name.to_string())),
        with.weights.map(|name| Value::Var(name.to_string())),
        with.weight_type.map(|word| Value::Word(word.to_string())),
        Some(Value::Float(0.95)),
    ];
    match call(
        zeff_glm::binary::glm,
        "d.glm(y ~ x, family: binomial)",
        values,
    ) {
        Value::Fit(fit) => fit,
        other => panic!("not a fit: {other:?}"),
    }
}

/// `fit.margins(vars, at:, type:)` through the function's body.
fn effects(fit: &Rc<Fit>, vars: &[&str], at: Option<Value>, kind: &str) -> Rc<TestResult> {
    let vars =
        (!vars.is_empty()).then(|| Value::Vars(vars.iter().map(|v| (*v).to_string()).collect()));
    let values = vec![
        Some(Value::Fit(Rc::clone(fit))),
        vars,
        at,
        Some(Value::Word(kind.to_string())),
    ];
    match call(margins, "m.margins(x)", values) {
        Value::TestResult(result) => result,
        other => panic!("not a result: {other:?}"),
    }
}

fn text(value: &Json) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("not text: {value}"))
}

fn number(value: &Json) -> f64 {
    text(value).parse().unwrap()
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}

#[test]
fn the_effects_on_the_probability_agree_with_r() {
    let file = root().join("tests/reference/r/margins_binary.json");
    let reference: Json = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 15);
    for (key, case) in cases {
        let cluster = text(&case["cluster"]);
        let weights = text(&case["weights"]);
        let with = With {
            se: match text(&case["se"]) {
                "classical" => Some("classical"),
                "HC1" => Some("robust"),
                "HC3" => Some("hc3"),
                _ => None,
            },
            cluster: (!cluster.is_empty()).then_some(cluster),
            weights: (!weights.is_empty()).then_some(weights),
            weight_type: (!weights.is_empty()).then_some(text(&case["weight_type"])),
        };
        let fit = fit(&wages, text(&case["formula"]), text(&case["link"]), &with);
        assert_eq!(fit.n as u64, case["nobs"].as_u64().unwrap(), "{key}: N");
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
        let result = effects(
            &fit,
            &vars,
            (!at.is_empty()).then_some(Value::Record(at)),
            text(&case["type"]),
        );
        let rows = case["rows"].as_array().unwrap();
        assert_eq!(result.terms.len(), rows.len(), "{key}: the rows");
        // Under the logit with an intercept the average fitted probability is the share of
        // ones among the rows used, so the average adjusted prediction is that share.
        if key == "prediction" {
            let outcome = &fit.likelihood.as_ref().expect("a likelihood fit").outcome;
            let share = outcome.iter().sum::<f64>() / outcome.len() as f64;
            let ours = result.terms[0].estimate.as_ref().unwrap().value;
            assert!(
                close(ours, share, 1e-10, 0.0),
                "the average prediction {ours:e}, the share of ones {share:e}"
            );
        }
        for (i, (row, expected)) in result.terms.iter().zip(rows).enumerate() {
            let term = text(&expected["term"]);
            assert_eq!(row.term, term, "{key}: the name of row {i}");
            assert_eq!(row.statistic.name, "z", "{key}, {term}: a z table");
            assert!(
                row.statistic.df.is_empty(),
                "{key}, {term}: no degrees of freedom"
            );
            let estimate = row.estimate.as_ref().expect("an estimate");
            let (lower, upper) = estimate.ci.expect("an interval");
            let std_error = number(&expected["std_error"]);
            // An effect near nought is compared on the scale of its standard error.
            let checks = [
                ("estimate", estimate.value, number(&expected["estimate"])),
                ("std_error", row.std_error.unwrap(), std_error),
                ("ci_lower", lower, number(&expected["ci_lower"])),
                ("ci_upper", upper, number(&expected["ci_upper"])),
            ];
            for (what, ours, theirs) in checks {
                assert!(
                    close(ours, theirs, 1e-7, 1e-7 * std_error),
                    "{key}, {term}: {what} {ours:e} against R's {theirs:e}"
                );
            }
            let z = number(&expected["z_value"]);
            assert!(
                close(row.statistic.value, z, 1e-7, 1e-7),
                "{key}, {term}: z {:e} against R's {z:e}",
                row.statistic.value
            );
            let p = number(&expected["p_value"]);
            assert!(
                close(row.p_value, p, 1e-5, 1e-300),
                "{key}, {term}: p {:e} against R's {p:e}",
                row.p_value
            );
        }
    }
}

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

#[test]
fn the_effects_of_a_binary_model_printed() {
    const W: &str = "let wages = dataset(wages)\n";
    let scripts = [
        // A logit and a probit: the average effects, the result read as any result is.
        format!(
            "{W}let g = wages.glm(union ~ educ + exper + exper^2 + female, family: binomial)\ng.margins([educ, exper, female])\nlet e = g.margins([exper, female])\ne.estimates()\ne.ci()\ne.p_values(female)\ne.statistic(exper)\ne.df(exper)\nlet p = wages.glm(union ~ educ * female + exper + log(hours), family: binomial, link: probit, se: robust, level: 0.9)\np.margins([educ, female, hours])\n"
        ),
        // A category, clustered errors; held values; at the means.
        format!(
            "{W}let c = wages.glm(union ~ industry + educ + female + female:educ, family: binomial, cluster: worker)\nc.margins([industry, female])\nc.margins([female, educ], at: {{industry: \"Mining\", educ: 12}})\nc.margins([female, educ, industry], type: mem)\n"
        ),
        // The adjusted prediction: the average probability; at values; at the means.
        format!(
            "{W}let g = wages.glm(union ~ educ * female + exper + exper^2 + industry, family: binomial)\ng.margins()\ng.margins(at: {{exper: 10, industry: \"Mining\", female: 1}})\ng.margins(type: mem)\ng.margins(type: mem, at: {{educ: 16}})\n"
        ),
        // Weights: sampling, then counts.
        format!(
            "{W}wages.glm(union ~ educ * female + exper, family: binomial, weights: hours).margins([educ, female])\nwages.glm(union ~ exper + exper^2 + female, family: binomial, weights: educ, weight_type: frequency).margins(exper)\n"
        ),
        // Several values in `at:`: the probability along experience, the effect of sex at two
        // levels of schooling at the means.
        format!(
            "{W}let g = wages.glm(union ~ educ * female + exper + exper^2, family: binomial)\ng.margins(at: {{exper: [0, 10, 20, 30]}})\ng.margins(female, type: mem, at: {{educ: [12, 16]}})\ng.margins(at: {{exper: [0, 10]}}).estimates()\n"
        ),
        // The coefficients of a logit beside its average effects, as a table for a paper.
        format!(
            "{W}let g = wages.glm(union ~ educ + exper + exper^2 + female, family: binomial, se: robust)\nlet e = g.margins([educ, exper, female])\nreport([g, e])\ne.std_errors(exper)\ne.ci(female)\n"
        ),
        // A term omitted as collinear: the effect that rests on it is not estimable.
        format!(
            "{W}let w = wages.add(twice = educ * 2)\nw.glm(union ~ educ + twice + female, family: binomial).margins([educ, twice, female])\n"
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
