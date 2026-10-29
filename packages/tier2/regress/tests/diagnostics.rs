//! `r_squared` and `influence` (catalogue rows 242 and 247; release 0.4.0, step 17; questions 4
//! and 7): the record and the measures against R's `summary()$r.squared`, `hatvalues`,
//! `cooks.distance` and `rstudent` on every reference regression, the rows replicated under
//! frequency weights as R sees them, and the printed forms and messages through the engine.

use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_regress::diagnostics::{influence, measures, r_squared};
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

fn call(body: Body, fit: &Rc<Fit>) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", "m.method()"));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "m.method()",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 8),
        arg_spans: vec![Some(Span::new(0, 1))],
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
            values: vec![Some(Value::Fit(Rc::clone(fit)))],
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
            "NA" | "NaN" => f64::NAN,
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            t => t.parse().unwrap(),
        },
        other => panic!("not a number: {other}"),
    }
}

fn numbers(value: &Json) -> Vec<f64> {
    value.as_array().unwrap().iter().map(number).collect()
}

fn same(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}

fn float(value: &Value) -> f64 {
    match value {
        Value::Float(x) => *x,
        Value::Na => f64::NAN,
        other => panic!("not a number: {other:?}"),
    }
}

#[test]
fn the_record_and_the_measures_agree_with_r_on_every_reference_fit() {
    let reference = json("r/regression.json");
    for (name, case) in reference["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
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
            level: Some(number(&case["level"])),
        };
        let fit = fit_with(&table, text, &options);
        // The record.
        match call(r_squared, &fit) {
            Value::Record(fields) => {
                let field = |n: &str| float(&fields.iter().find(|(f, _)| f == n).unwrap().1);
                assert!(
                    same(field("r_squared"), number(&case["r_squared"]), 1e-10, 1e-14),
                    "{name}"
                );
                assert!(
                    same(
                        field("adjusted"),
                        number(&case["adj_r_squared"]),
                        1e-10,
                        1e-14
                    ),
                    "{name}"
                );
            }
            other => panic!("{name}: r_squared() gave {other:?}"),
        }
        // The measures of the rows used, in R's order: under frequency weights R's rows are the
        // replicates, each with the leverage h / w and the row's residual.
        let measured = measures(&fit);
        let counts: Vec<usize> = match (fit.weight_kind, &fit.weights) {
            (Some(zeff_numeric::order::WeightKind::Frequency), Some(w)) => {
                w.iter().map(|w| w.round() as usize).collect()
            }
            _ => vec![1; measured.len()],
        };
        let mut in_r_order: Vec<(f64, f64, f64)> = Vec::new();
        for (m, &count) in measured.iter().zip(&counts) {
            for _ in 0..count {
                in_r_order.push(*m);
            }
        }
        let (hat, cooks, rstudent) = (
            numbers(&case["hat_20"]),
            numbers(&case["cooks_20"]),
            numbers(&case["rstudent_20"]),
        );
        for i in 0..hat.len() {
            let (h, d, t) = in_r_order[i];
            assert!(
                same(h, hat[i], 1e-9, 1e-14),
                "{name}: leverage [{i}] {h:e} against {:e}",
                hat[i]
            );
            assert!(
                same(d, cooks[i], 1e-8, 1e-14),
                "{name}: Cook's distance [{i}] {d:e} against {:e}",
                cooks[i]
            );
            assert!(
                same(t, rstudent[i], 1e-8, 1e-12),
                "{name}: studentized [{i}] {t:e} against {:e}",
                rstudent[i]
            );
        }
        // The frame: full length, NA in the dropped rows, the measures in the rows used.
        match call(influence, &fit) {
            Value::Frame(frame) => {
                assert_eq!(frame.table.rows(), table.rows(), "{name}");
                assert_eq!(
                    frame.table.names(),
                    ["leverage", "cooks_distance", "studentized"]
                );
                let leverage: Vec<Option<f64>> = frame
                    .table
                    .column("leverage")
                    .unwrap()
                    .as_floats()
                    .unwrap()
                    .collect();
                for (i, &row) in fit.rows.iter().enumerate() {
                    assert_eq!(leverage[row], Some(measured[i].0), "{name}: row {row}");
                }
                assert_eq!(
                    leverage.iter().filter(|v| v.is_none()).count(),
                    table.rows() - fit.rows.len(),
                    "{name}: NA rows"
                );
            }
            other => panic!("{name}: influence() gave {other:?}"),
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
fn the_printed_forms_and_every_message() {
    let scripts = [
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + female)\nm.r_squared()\nm.r_squared().adjusted\ntype_of(m.r_squared())\n",
        "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + female)\nm.influence()\nm.influence().summary([leverage, cooks_distance, studentized])\nm.influence().where(cooks_distance > 0.001).n_rows()\nwages.add(h = m.influence()[\"leverage\"]).where(is_missing(h)).n_rows()\n",
        // Weighted fits: the residual weighted, or the row standing for its count.
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + female, weights: hours).influence().summary([leverage, cooks_distance, studentized])\nw.reg(log(wage) ~ educ + female, weights: exper, weight_type: frequency).influence().summary([leverage, cooks_distance, studentized])\n",
        // An outcome without variation: R² undefined, the measures of a perfect fit absent.
        "dataset(wages).add(c = 5).reg(c ~ educ).r_squared()\n",
        "dataset(wages).add(y2 = 3 * educ + 1).reg(y2 ~ educ).influence().summary([leverage, cooks_distance, studentized])\n",
        "dataset(wages).r_squared()\n",
        "dataset(wages).describe().influence()\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(script)));
    }
    insta::assert_snapshot!("diagnostics", all);
}

/// A small table of float columns.
fn small(columns: &[(&str, &[f64])]) -> Table {
    Table::new(
        columns
            .iter()
            .map(|(name, values)| {
                (
                    name.to_string(),
                    Column::floats(values.iter().map(|v| Some(*v))).0,
                )
            })
            .collect(),
    )
    .unwrap()
}

/// With a single residual degree of freedom there is no error variance without a row, so no
/// Cook's distance and no studentized residual, whatever the leverage.
#[test]
fn one_residual_degree_of_freedom_leaves_no_influence_measures() {
    let t = small(&[("y", &[1.0, 2.0, 4.0]), ("x", &[1.0, 2.0, 3.0])]);
    let fit = fit_with(&t, "y ~ x", &Options::default());
    assert_eq!(fit.n - fit.terms.len(), 1);
    for (h, cooks, studentized) in measures(&fit) {
        assert!(h.is_finite() && h < 1.0, "{h}");
        assert!(
            cooks.is_nan() && studentized.is_nan(),
            "{cooks} {studentized}"
        );
    }
}

/// Three zeros and a four on an intercept alone: the fourth row's removal leaves a perfect fit,
/// so its error variance without the row is exactly zero and it has no studentized residual,
/// while its Cook's distance is one and the other rows' measures are finite.
#[test]
fn a_row_whose_removal_leaves_a_perfect_fit_has_no_studentized_residual() {
    let t = small(&[("y", &[0.0, 0.0, 0.0, 4.0])]);
    let fit = fit_with(&t, "y ~ 1", &Options::default());
    let measured = measures(&fit);
    assert_eq!(measured.len(), 4);
    for (h, cooks, studentized) in &measured[..3] {
        assert!(same(*h, 0.25, 1e-12, 0.0) && cooks.is_finite() && studentized.is_finite());
    }
    let (h, cooks, studentized) = measured[3];
    assert!(
        same(h, 0.25, 1e-12, 0.0) && same(cooks, 1.0, 1e-12, 0.0),
        "{h} {cooks}"
    );
    assert!(studentized.is_nan(), "{studentized}");
}
