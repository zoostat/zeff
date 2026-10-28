//! The result interface (Engine and Project Specification, section 8.1, rule 5; Language
//! Reference, sections 3.7, 3.8 and 10.2; release 0.4.0, step 15; questions 3 (d) and 4): the
//! six methods against R on every reference regression (`tests/reference/r/regression.json`:
//! `coef(summary())`, `confint` at each case's level, `nobs`, the standardized coefficients,
//! `AIC` and `BIC`), and their printed forms, both homes of `summary` and every message through
//! the engine.

use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::Table;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_results::interface::{
    ci, estimates, information_criteria, n_obs, p_values, standardized, std_errors,
};
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

#[derive(Default, Clone)]
struct Options<'a> {
    se: Option<&'a str>,
    cluster: Option<&'a str>,
    weights: Option<&'a str>,
    weight_type: Option<&'a str>,
    level: Option<f64>,
}

/// A fit through `reg`'s body, as the regression tests make it.
fn fit_with(table: &Table, text: &str, options: &Options<'_>) -> Rc<Fit> {
    let f = formula(text);
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.reg({text})\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(10, 13),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(14, 14 + text.len()))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(f)),
            options.se.map(|w| Value::Word(w.to_string())),
            options.cluster.map(|v| Value::Var(v.to_string())),
            options.weights.map(|v| Value::Var(v.to_string())),
            options.weight_type.map(|w| Value::Word(w.to_string())),
            options.level.map(Value::Float),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => fit,
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// A result method called on a fit with the arguments given (the receiver added).
fn call(body: Body, fit: &Rc<Fit>, args: Vec<Option<Value>>) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", "m.method()"));
    let mut values = vec![Some(Value::Fit(Rc::clone(fit)))];
    values.extend(args);
    let spans = vec![Some(Span::new(0, 1)); values.len()];
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "m.method()",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 8),
        arg_spans: spans,
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
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

fn dataset(file: &str) -> Table {
    let path = root().join("tests/datasets").join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn reference_data(file: &str) -> Table {
    let path = root()
        .join("tests/reference/data")
        .join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        "wage1" | "wage2" => reference_data(name),
        other => dataset(other),
    }
}

fn json(file: &str) -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference").join(file)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(value: &Json) -> f64 {
    match value {
        Json::Number(n) => n.as_f64().unwrap(),
        Json::String(s) => match s.as_str() {
            "NA" => f64::NAN,
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            t => t.parse().unwrap(),
        },
        other => panic!("not a number: {other}"),
    }
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}

fn float(value: &Value) -> f64 {
    match value {
        Value::Float(x) => *x,
        Value::Int(i) => *i as f64,
        Value::Na => f64::NAN,
        other => panic!("not a number: {other:?}"),
    }
}

fn term(name: &str) -> Option<Value> {
    Some(Value::Term(name.to_string()))
}

#[test]
fn the_interface_agrees_with_r_on_every_reference_fit() {
    let reference = json("r/regression.json");
    for (name, case) in reference["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
        let level = number(&case["level"]);
        let non_empty = |key: &str| {
            let v = case[key].as_str().unwrap_or("");
            (!v.is_empty()).then_some(v)
        };
        let se = match case["se"].as_str().unwrap_or("classical") {
            "classical" => Some("classical"),
            "HC1" => Some("robust"),
            "HC2" => Some("hc2"),
            "HC3" => Some("hc3"),
            _ => None,
        };
        let options = Options {
            se: if non_empty("cluster").is_some() {
                None
            } else {
                se
            },
            cluster: non_empty("cluster"),
            weights: non_empty("weights"),
            weight_type: non_empty("weight_type"),
            level: Some(level),
        };
        let fit = fit_with(&table, text, &options);
        let columns = case["columns"].as_object().unwrap();
        // The mappings: every term with an estimate, in the model's order.
        let errors = match call(std_errors, &fit, vec![None]) {
            Value::Mapping(entries) => entries,
            other => panic!("{name}: std_errors() gave {other:?}"),
        };
        let p = match call(p_values, &fit, vec![None]) {
            Value::Mapping(entries) => entries,
            other => panic!("{name}: p_values() gave {other:?}"),
        };
        assert_eq!(errors.len(), fit.terms.len(), "{name}");
        for (i, t) in fit.terms.iter().enumerate() {
            let r = &columns[t];
            assert_eq!(errors[i].0, Value::Str(t.clone()), "{name}: the order");
            let (se_zeff, se_r) = (float(&errors[i].1), number(&r["std_error"]));
            assert!(
                close(se_zeff, se_r, 1e-10, 1e-14),
                "{name}: {t} std_error {se_zeff:e} against {se_r:e}"
            );
            let (p_zeff, p_r) = (float(&p[i].1), number(&r["p_value"]));
            assert!(
                close(p_zeff, p_r, 1e-8, 1e-14),
                "{name}: {t} p_value {p_zeff:e} against {p_r:e}"
            );
            // One term named: the same values, and the interval at the case's level.
            assert!(close(
                float(&call(std_errors, &fit, vec![term(t)])),
                se_r,
                1e-10,
                1e-14
            ));
            assert!(close(
                float(&call(p_values, &fit, vec![term(t)])),
                p_r,
                1e-8,
                1e-14
            ));
            match call(ci, &fit, vec![term(t), Some(Value::Float(level))]) {
                Value::List(limits) => {
                    let (lo, hi) = (float(&limits[0]), float(&limits[1]));
                    assert!(
                        close(lo, number(&r["ci_lower"]), 1e-10, 1e-14)
                            && close(hi, number(&r["ci_upper"]), 1e-10, 1e-14),
                        "{name}: {t} ci [{lo}, {hi}]"
                    );
                }
                other => panic!("{name}: ci({t}) gave {other:?}"),
            }
        }
        // The table of intervals: one row per term, titled with the level.
        match call(ci, &fit, vec![None, Some(Value::Float(level))]) {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), fit.terms.len(), "{name}");
                assert!(table.title.starts_with("ci · "), "{name}: {}", table.title);
            }
            other => panic!("{name}: ci() gave {other:?}"),
        }
        // The observations.
        assert_eq!(
            call(n_obs, &fit, vec![]),
            Value::Int(case["nobs"].as_i64().unwrap()),
            "{name}"
        );
        // The estimates frame.
        match call(estimates, &fit, vec![]) {
            Value::Frame(frame) => {
                assert_eq!(frame.table.rows(), fit.terms.len(), "{name}");
                let names: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
                assert_eq!(
                    names,
                    [
                        "term",
                        "estimate",
                        "std_error",
                        "statistic",
                        "p_value",
                        "ci_lower",
                        "ci_upper"
                    ],
                    "{name}"
                );
            }
            other => panic!("{name}: estimates() gave {other:?}"),
        }
        // The standardized coefficients and the information criteria.
        let betas = standardized(&fit);
        for (i, t) in fit.terms.iter().enumerate() {
            let r_beta = number(&columns[t]["beta"]);
            match betas[i] {
                Some(beta) => assert!(
                    close(beta, r_beta, 1e-9, 1e-12),
                    "{name}: {t} beta {beta:e} against {r_beta:e}"
                ),
                // A column without variation (the intercept): R writes 0.
                None => assert!(
                    r_beta == 0.0 || r_beta.is_nan(),
                    "{name}: {t} beta {r_beta}"
                ),
            }
        }
        {
            let (aic, bic) = information_criteria(&fit);
            let (aic_r, bic_r) = (number(&case["aic"]), number(&case["bic"]));
            assert!(
                close(aic, aic_r, 1e-10, 1e-12),
                "{name}: AIC {aic} against {aic_r}"
            );
            assert!(
                close(bic, bic_r, 1e-10, 1e-12),
                "{name}: BIC {bic} against {bic_r}"
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
fn the_printed_forms_both_homes_of_summary_and_every_message() {
    let scripts = [
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.summary()\nm.summary(standardized: true)\nm.summary(fit_measures: true)\nm.summary(standardized: true, fit_measures: true)\n",
        "let wages = dataset(wages)\nwages.summary([wage, educ])\nlet m = wages.reg(log(wage) ~ educ + female)\nm.summary()\ntype_of(m.summary())\ntype_of(wages.summary([wage]))\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.estimates()\nm.estimates().where(p_value < 0.05)\nm.estimates().n_rows()\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.std_errors()\nm.std_errors(educ)\nm.std_errors(\"exper\")\nlet educ = 5\nm.std_errors(educ)\nm.std_errors()[\"female\"] * 2\nstd_errors(m, female)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.p_values()\nm.p_values(female)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.ci()\nm.ci(educ)\nm.ci(educ, level: 0.9)\nm.ci(level: 0.999)\nm.ci().export(\"output/ci.tex\")\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.n_obs()\ndataset(wages).reg(log(wage) ~ educ, weights: exper, weight_type: frequency).n_obs()\n",
        // An omitted term has no estimate: NA by name, absent from the mappings.
        "let m = dataset(wages).add(twice = educ * 2).reg(log(wage) ~ educ + twice + female)\nm.std_errors()\nm.std_errors(twice)\nm.p_values(twice)\nm.ci(twice)\nm.summary(standardized: true)\n",
        // A weighted and a clustered fit keep their standard errors in the accessors.
        "let w = dataset(wages)\nlet c = w.reg(log(wage) ~ educ + female, cluster: worker)\nc.std_errors()\nc.ci(educ)\nlet f = w.reg(log(wage) ~ educ + female, weights: hours)\nf.std_errors(educ)\nf.summary(standardized: true, fit_measures: true)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.ci(level: 1.5)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.ci(educ, level: 0)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.std_errors(nope)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.p_values(\"educ:female\")\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.summary(standardized: 1)\n",
        "dataset(wages).std_errors()\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nm.std_errors(educ) + m.ci()\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(script)));
    }
    insta::assert_snapshot!("interface", all);
}

// ----- the estimator interface, part 1 (step 16) -----

fn same(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a.is_nan() && b.is_nan()) || close(a, b, relative, absolute)
}

fn column_values(value: &Value) -> Vec<f64> {
    match value {
        Value::Column(c) => c
            .as_floats()
            .expect("a Float column")
            .map(|v| v.unwrap_or(f64::NAN))
            .collect(),
        other => panic!("not a column: {other:?}"),
    }
}

#[test]
fn the_estimator_interface_agrees_with_r_on_every_reference_fit() {
    use zeff_results::interface::{coef, fit_measures, fitted, predict, residuals, vcov};
    let reference = json("r/regression.json");
    for (name, case) in reference["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
        let level = number(&case["level"]);
        let non_empty = |key: &str| {
            let v = case[key].as_str().unwrap_or("");
            (!v.is_empty()).then_some(v)
        };
        let se = match case["se"].as_str().unwrap_or("classical") {
            "classical" => Some("classical"),
            "HC1" => Some("robust"),
            "HC2" => Some("hc2"),
            "HC3" => Some("hc3"),
            _ => None,
        };
        let options = Options {
            se: if non_empty("cluster").is_some() {
                None
            } else {
                se
            },
            cluster: non_empty("cluster"),
            weights: non_empty("weights"),
            weight_type: non_empty("weight_type"),
            level: Some(level),
        };
        let fit = fit_with(&table, text, &options);
        let columns = case["columns"].as_object().unwrap();
        // The coefficients, by term and one by one.
        let estimates = match call(coef, &fit, vec![None]) {
            Value::Mapping(entries) => entries,
            other => panic!("{name}: coef() gave {other:?}"),
        };
        for (i, t) in fit.terms.iter().enumerate() {
            let r = number(&columns[t]["estimate"]);
            assert_eq!(estimates[i].0, Value::Str(t.clone()), "{name}");
            assert!(
                close(float(&estimates[i].1), r, 1e-10, 1e-14),
                "{name}: coef {t}"
            );
            assert!(close(
                float(&call(coef, &fit, vec![term(t)])),
                r,
                1e-10,
                1e-14
            ));
        }
        // The variance matrix: the fit's values against R's by name, and the table's cells the
        // printed form of those values.
        let r_terms: Vec<String> = case["vcov_terms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();
        let r_vcov = case["vcov"].as_array().unwrap();
        let k = fit.terms.len();
        assert_eq!(r_terms.len(), k, "{name}: the terms of vcov");
        // R writes each row of the matrix as an object keyed by the column's term.
        let position = |t: &str| r_terms.iter().position(|x| x == t).unwrap();
        for (i, ti) in fit.terms.iter().enumerate() {
            for (j, tj) in fit.terms.iter().enumerate() {
                let r = number(&r_vcov[position(ti)][tj.as_str()]);
                let z = fit.vcov[i * k + j];
                assert!(
                    close(z, r, 1e-10, 1e-16),
                    "{name}: vcov[{ti}, {tj}] {z:e} against {r:e}"
                );
            }
        }
        match call(vcov, &fit, vec![]) {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), k, "{name}");
                assert_eq!(table.columns.len(), k + 1, "{name}");
                assert!(
                    table.title.starts_with("vcov · "),
                    "{name}: {}",
                    table.title
                );
                for (i, row) in table.rows.iter().enumerate() {
                    assert_eq!(row[0], fit.terms[i]);
                    for j in 0..k {
                        assert_eq!(
                            row[j + 1],
                            zeff_core::display::format_float(fit.vcov[i * k + j]),
                            "{name}"
                        );
                    }
                }
            }
            other => panic!("{name}: vcov() gave {other:?}"),
        }
        // The fit measures.
        match call(fit_measures, &fit, vec![]) {
            Value::Record(fields) => {
                let field = |n: &str| {
                    float(
                        &fields
                            .iter()
                            .find(|(f, _)| f == n)
                            .unwrap_or_else(|| panic!("{name}: no field {n}"))
                            .1,
                    )
                };
                assert!(close(
                    field("r_squared"),
                    number(&case["r_squared"]),
                    1e-10,
                    1e-14
                ));
                assert!(close(
                    field("adjusted"),
                    number(&case["adj_r_squared"]),
                    1e-10,
                    1e-14
                ));
                assert!(
                    close(
                        field("log_likelihood"),
                        number(&case["log_likelihood"]),
                        1e-10,
                        1e-12
                    ),
                    "{name}: log-likelihood {} against {}",
                    field("log_likelihood"),
                    number(&case["log_likelihood"])
                );
                assert!(close(field("aic"), number(&case["aic"]), 1e-10, 1e-12));
                assert!(close(field("bic"), number(&case["bic"]), 1e-10, 1e-12));
            }
            other => panic!("{name}: fit_measures() gave {other:?}"),
        }
        // Residuals and fitted values: full length, NA in the dropped rows, the first twenty of
        // the rows used as R gives them (R replicates the rows under frequency weights, so those
        // cases are compared in length alone).
        let frequency = options.weight_type == Some("frequency") && options.weights.is_some();
        for (body, key, own) in [
            (residuals as Body, "residuals_20", &fit.residuals),
            (fitted as Body, "fitted_20", &fit.fitted),
        ] {
            let values = column_values(&call(body, &fit, vec![]));
            assert_eq!(values.len(), table.rows(), "{name}: the length");
            for (i, &row) in fit.rows.iter().enumerate() {
                assert!(
                    values[row].to_bits() == own[i].to_bits(),
                    "{name}: row {row}"
                );
            }
            let dropped = (0..table.rows()).filter(|r| values[*r].is_nan()).count();
            assert_eq!(dropped, table.rows() - fit.rows.len(), "{name}: NA rows");
            if !frequency {
                let r_values: Vec<f64> = case[key].as_array().unwrap().iter().map(number).collect();
                for (i, r) in r_values.iter().enumerate() {
                    let z = values[fit.rows[i]];
                    assert!(
                        same(z, *r, 1e-9, 1e-12),
                        "{name}: {key} [{i}] {z:e} against {r:e}"
                    );
                }
            }
        }
        // Predictions for the first ten rows, as R's `predict` on the same rows.
        let ten: Vec<usize> = (0..10).collect();
        let newdata = Value::Frame(Frame::new(table.take_rows(&ten)));
        let predicted = column_values(&call(predict, &fit, vec![Some(newdata)]));
        let r_predicted: Vec<f64> = case["predict_10"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        assert_eq!(predicted.len(), 10, "{name}");
        for (i, r) in r_predicted.iter().enumerate() {
            assert!(
                same(predicted[i], *r, 1e-9, 1e-12),
                "{name}: predict [{i}] {:e} against {r:e}",
                predicted[i]
            );
        }
    }
}

#[test]
fn the_estimator_methods_print_and_every_message() {
    let scripts = [
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.coef()\nm.coef(educ)\nm.coef(\"female\")\nm.vcov()\nm.fit_measures()\nm.fit_measures().aic\nm.fit_measures().r_squared\n",
        "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + female)\nlet r = m.residuals()\ntype_of(r)\nwages.add(res = m.residuals(), yhat = m.fitted()).summary([res, yhat])\nwages.add(res = m.residuals()).where(is_missing(res)).n_rows()\n",
        "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + female + industry)\nm.predict(newdata: wages.head(5))\nm.predict(newdata: frame(educ = [12, 16], exper = [5, 10], female = [0, 1], industry = [\"Retail\", \"Finance\"]))\nm.predict(newdata: frame(educ = [12, 16, NA], exper = [5, 10, 2], female = [0, 1, 1], industry = [\"Retail\", \"Space\", \"Health\"]))\n",
        // A sum contrast and a base: the new design uses the fit's levels in the fit's order, and
        // the two parameterisations predict the same values.
        "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + category(industry, contrast: sum))\nm.predict(newdata: wages.head(3))\nlet b = wages.reg(log(wage) ~ educ + base(industry, \"Retail\"))\nb.predict(newdata: wages.head(3))\n",
        // An omitted term: no coefficient, and no part in a prediction.
        "let wages = dataset(wages)\nlet m = wages.add(twice = educ * 2).reg(log(wage) ~ educ + twice)\nm.coef()\nm.coef(twice)\nm.vcov()\nm.predict(newdata: frame(educ = [12], twice = [99]))\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.predict()\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.predict(newdata: frame(educ = [12], exper = [5]))\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.predict(newdata: 5)\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.predict(newdata: frame(educ = [\"twelve\"], exper = [5], female = [0]))\n",
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.coef(nope)\n",
        "dataset(wages).vcov()\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(script)));
    }
    insta::assert_snapshot!("estimator", all);
}
