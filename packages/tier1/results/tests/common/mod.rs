//! Helpers shared by the tests of `results`: a regression fit through `reg`'s body, as the
//! regression tests make it, a function or method body called with values, the datasets and the
//! R reference.
#![allow(dead_code)]

use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::Table;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_testkit::{formula, root};

pub struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

#[derive(Default, Clone)]
pub struct Options<'a> {
    pub se: Option<&'a str>,
    pub cluster: Option<&'a str>,
    pub weights: Option<&'a str>,
    pub weight_type: Option<&'a str>,
    pub level: Option<f64>,
}

/// A fit through `reg`'s body, as the regression tests make it.
pub fn fit_with(table: &Table, text: &str, options: &Options<'_>) -> Rc<Fit> {
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

/// A function or method body called with the values given (a method's receiver first), the rest
/// values and the definitions.
pub fn call(
    body: Body,
    values: Vec<Option<Value>>,
    rest: Vec<Value>,
    definitions: Vec<(String, Value)>,
) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", "f(x)"));
    let spans = vec![Some(Span::new(2, 3)); values.len()];
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "f(x)",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(0, 1),
        arg_spans: spans,
        rest_spans: vec![Span::new(2, 3); rest.len()],
        definition_spans: vec![Span::new(2, 3); definitions.len()],
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    match body(
        &mut ctx,
        Args {
            values,
            rest,
            definitions,
        },
    ) {
        Ok(value) => value,
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

pub fn dataset(file: &str) -> Table {
    let path = root().join("tests/datasets").join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

pub fn reference_data(file: &str) -> Table {
    let path = root()
        .join("tests/reference/data")
        .join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

pub fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        "wage1" | "wage2" => reference_data(name),
        other => dataset(other),
    }
}

pub fn json(file: &str) -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference").join(file)).unwrap();
    serde_json::from_str(&text).unwrap()
}

pub fn number(value: &Json) -> f64 {
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

pub fn text(value: &Json) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("not text: {value}"))
}

pub fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}
