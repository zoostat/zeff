//! `vif` against base R (testing handbook, section 1.1; release 0.5.0, step 21): 1 / (1 − R²ⱼ)
//! from `lm` for every column of the model matrix, on the shipped `wages` (a textbook model, a
//! categorical regressor, a fit without an intercept) and Wooldridge's `wage1`, within a relative
//! 1e-10 (`tests/reference/r/vif.R`); and the properties that rescaling a regressor changes no
//! factor and that an orthogonal regressor has factor 1.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_regress::diagnostics::vifs;
use zeff_testkit::{formula, root};

struct NoFiles;
impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("the tests read no file through the engine")
    }
    fn write(&self, _: &str, _: &[u8], _: Span, _: &str) -> Result<(), Fail> {
        unreachable!("the tests write no file")
    }
}

/// A fit of `text` on `table`, as `reg` makes it.
fn fit_of(table: &Table, text: &str) -> Fit {
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
            None,
            None,
            None,
            None,
            None,
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => (*fit).clone(),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        other => {
            let path = root()
                .join("tests/reference/data")
                .join(format!("{other}.csv"));
            let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            read_csv(&bytes, &CsvOptions::default())
                .unwrap_or_else(|e| panic!("{other}: {e:?}"))
                .table
        }
    }
}

/// R's model-matrix column name for one of Zeff's term names: `industry=Mining` is
/// `industryMining`, `exper^2` is `I(exper^2)`.
fn r_name(term: &str) -> String {
    if let Some((var, level)) = term.split_once('=') {
        format!("{var}{level}")
    } else if term.contains('^') {
        format!("I({term})")
    } else {
        term.to_string()
    }
}

#[test]
fn the_factors_agree_with_lm() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/vif.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    for case in reference["cases"].as_array().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let fit = fit_of(&table, case["formula"].as_str().unwrap());
        let found = vifs(&fit);
        let expected: Vec<(String, f64)> = case["factors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f["term"].as_str().unwrap().to_string(),
                    f["vif"].as_str().unwrap().parse().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            found.len(),
            expected.len(),
            "{}: {found:?}",
            case["formula"]
        );
        for (term, value) in &found {
            let name = r_name(term);
            let (_, r) = expected
                .iter()
                .find(|(t, _)| *t == name)
                .unwrap_or_else(|| {
                    panic!("{}: no R column {name} among {expected:?}", case["formula"])
                });
            assert!(
                (value - r).abs() <= 1e-10 * r.abs(),
                "{} / {term}: {value} against R's {r}",
                case["formula"]
            );
        }
    }
}

fn floats(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

fn table(columns: Vec<(&str, Column)>) -> Table {
    Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap()
}

#[test]
fn one_regressor_and_an_orthogonal_one_have_factor_one() {
    let t = table(vec![
        ("y", floats(&[3.0, 5.0, 8.0, 9.0, 12.0, 12.0])),
        ("x", floats(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
        ("z", floats(&[1.0, -1.0, -1.0, 1.0, 1.0, -1.0])),
    ]);
    let factors = vifs(&fit_of(&t, "y ~ x"));
    assert_eq!(factors.len(), 1);
    assert!((factors[0].1 - 1.0).abs() <= 1e-10, "{factors:?}");
    // z sums to zero and Σxz = 1 − 2 − 3 + 4 = 0, so z is orthogonal to x and to the intercept.
    let t = table(vec![
        ("y", floats(&[3.0, 5.0, 8.0, 9.0])),
        ("x", floats(&[1.0, 2.0, 3.0, 4.0])),
        ("z", floats(&[1.0, -1.0, -1.0, 1.0])),
    ]);
    let factors = vifs(&fit_of(&t, "y ~ x + z"));
    for (term, value) in &factors {
        assert!((value - 1.0).abs() <= 1e-10, "{term}: {value}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Rescaling a regressor changes no factor.
    #[test]
    fn rescaling_a_regressor_changes_no_factor(
        values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 8..30),
        scale in prop::sample::select(vec![0.001, 0.5, 3.0, 1000.0]),
    ) {
        let y: Vec<f64> = values.iter().map(|v| v.0 + v.1 - v.2).collect();
        let x: Vec<f64> = values.iter().map(|v| v.1).collect();
        let z: Vec<f64> = values.iter().map(|v| v.2 + 0.5 * v.1).collect();
        let scaled: Vec<f64> = x.iter().map(|v| v * scale).collect();
        let a = vifs(&fit_of(&table(vec![("y", floats(&y)), ("x", floats(&x)), ("z", floats(&z))]), "y ~ x + z"));
        let b = vifs(&fit_of(&table(vec![("y", floats(&y)), ("x", floats(&scaled)), ("z", floats(&z))]), "y ~ x + z"));
        prop_assert_eq!(a.len(), b.len());
        for ((_, p), (_, q)) in a.iter().zip(&b) {
            prop_assert!((p - q).abs() <= 1e-9 * p.abs().max(1.0), "{p} against {q}");
        }
    }
}
