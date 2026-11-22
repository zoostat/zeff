//! `confusion` (catalogue row 516; open decision #88; release 0.9.0, step 15): the
//! classification table of a binary fit against R on the cases of
//! `tests/reference/r/confusion.json` (the four cells counted in base R, with the shares made
//! of them), and the printed forms and every refusal through the engine.

use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_glm::confusion::confusion;
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
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
fn fit(table: &Table, text: &str, link: &str, weights: Option<(&str, &str)>) -> Rc<Fit> {
    let values = vec![
        Some(Value::Frame(Frame::new(table.clone()))),
        Some(Value::Formula(formula(text))),
        Some(Value::Word("binomial".to_string())),
        Some(Value::Word(link.to_string())),
        None,
        None,
        None,
        weights.map(|(name, _)| Value::Var(name.to_string())),
        weights.map(|(_, kind)| Value::Word(kind.to_string())),
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

/// `fit.confusion(newdata:, cutoff:)` through the function's body.
fn confusion_of(fit: &Rc<Fit>, newdata: Option<&Table>, cutoff: f64) -> Rc<TestResult> {
    let values = vec![
        Some(Value::Fit(Rc::clone(fit))),
        newdata.map(|table| Value::Frame(Frame::new(table.clone()))),
        Some(Value::Float(cutoff)),
    ];
    match call(confusion, "m.confusion()", values) {
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

/// A count as the table prints it, read back as a number.
fn cell(text: &str) -> f64 {
    text.replace(',', "").parse().unwrap()
}

/// The rows of `table` whose `year` is at most, or after, 2017.
fn by_year(table: &Table, later: bool) -> Table {
    let years: Vec<Option<i64>> = table
        .column("year")
        .and_then(|column| column.as_ints())
        .expect("the year, a whole number")
        .collect();
    let rows: Vec<usize> = (0..years.len())
        .filter(|&row| years[row].is_some_and(|year| (year > 2017) == later))
        .collect();
    table.take_rows(&rows)
}

#[test]
fn the_table_and_its_shares_agree_with_r() {
    let file = root().join("tests/reference/r/confusion.json");
    let reference: Json = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 7);
    for (key, case) in cases {
        let weights = text(&case["weights"]);
        let weights = (!weights.is_empty()).then(|| (weights, text(&case["weight_type"])));
        let cutoff = number(&case["cutoff"]);
        let result = if case["new_rows"].as_bool().unwrap() {
            let fitted = fit(
                &by_year(&wages, false),
                text(&case["formula"]),
                text(&case["link"]),
                weights,
            );
            confusion_of(&fitted, Some(&by_year(&wages, true)), cutoff)
        } else {
            let fitted = fit(&wages, text(&case["formula"]), text(&case["link"]), weights);
            confusion_of(&fitted, None, cutoff)
        };
        assert_eq!(result.n as u64, case["n"].as_u64().unwrap(), "{key}: N");
        // The shares, each a row of the result: exact ratios of counts without weights.
        let names = [
            ("sensitivity", "sensitivity"),
            ("specificity", "specificity"),
            ("accuracy", "accuracy"),
            ("positive predictive value", "positive_predictive_value"),
            ("negative predictive value", "negative_predictive_value"),
        ];
        assert_eq!(result.terms.len(), names.len(), "{key}: the shares");
        for (row, (name, theirs)) in result.terms.iter().zip(names) {
            assert_eq!(row.term, name, "{key}");
            let ours = row.estimate.as_ref().expect("a share").value;
            assert_eq!(
                row.statistic.value.to_bits(),
                ours.to_bits(),
                "{key}, {name}"
            );
            match text(&case[theirs]) {
                "NA" => assert!(ours.is_nan(), "{key}: {name} is {ours}, and R has NA"),
                written => {
                    let theirs: f64 = written.parse().unwrap();
                    assert!(
                        (ours - theirs).abs() <= 1e-12,
                        "{key}: {name} {ours:e} against R's {theirs:e}"
                    );
                }
            }
        }
        // The cells as printed. Under sampling weights Zeff's weights are scaled to sum to the
        // rows used, so the cells are compared as shares of their total.
        let table = result.table.as_ref().expect("the table");
        let ours = [
            cell(&table.rows[0][1]),
            cell(&table.rows[0][2]),
            cell(&table.rows[1][1]),
            cell(&table.rows[1][2]),
        ];
        let theirs = [
            number(&case["true_positive"]),
            number(&case["false_negative"]),
            number(&case["false_positive"]),
            number(&case["true_negative"]),
        ];
        let scaled = weights.is_some_and(|(_, kind)| kind == "sampling");
        let (sum_ours, sum_theirs): (f64, f64) = (ours.iter().sum(), theirs.iter().sum());
        for (o, t) in ours.iter().zip(&theirs) {
            if scaled {
                assert!(
                    (o / sum_ours - t / sum_theirs).abs() <= 1e-5,
                    "{key}: a cell's share {} against R's {}",
                    o / sum_ours,
                    t / sum_theirs
                );
            } else {
                assert_eq!(o, t, "{key}: a cell");
            }
        }
        assert_eq!(
            cell(&table.rows[2][3]),
            sum_ours.round(),
            "{key}: the total"
        );
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
fn the_table_printed_and_refused() {
    const W: &str = "let wages = dataset(wages)\n";
    let scripts = [
        // The table at one half and at another cutoff; the result read as any result is.
        format!(
            "{W}let g = wages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial)\nlet c = g.confusion()\nc\nc.estimates()\nc.statistic(\"sensitivity\")\nc.statistic(\"positive predictive value\")\nc.statistic()\nc.n_obs()\ntype_of(c)\ng.confusion(cutoff: 0.3)\nc.export(\"output/confusion.tex\")\n"
        ),
        // A probit; nothing predicted 1; on new rows; under each kind of weight.
        format!(
            "{W}wages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial, link: probit).confusion(cutoff: 0.4)\nwages.glm(union ~ educ + exper + female, family: binomial).confusion(cutoff: 0.9)\nwages.where(year <= 2017).glm(union ~ educ + exper + exper^2 + female + industry, family: binomial).confusion(newdata: wages.where(year > 2017), cutoff: 0.4)\nwages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial, weights: hours).confusion(cutoff: 0.35)\nwages.glm(union ~ exper + female + industry, family: binomial, weights: educ, weight_type: frequency).confusion(cutoff: 0.35)\n"
        ),
        // The refusals.
        format!("{W}wages.glm(union ~ educ, family: binomial).confusion(cutoff: 50)\n"),
        format!("{W}wages.glm(union ~ educ, family: binomial).confusion(cutoff: 0)\n"),
        format!("{W}wages.glm(union ~ educ, family: binomial).confusion(cutoff: \"half\")\n"),
        format!("{W}wages.reg(union ~ educ).confusion()\n"),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).confusion(newdata: wages.select([educ, exper]))\n"
        ),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).confusion(newdata: wages.add(union = union * 2))\n"
        ),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).confusion().statistic(\"precision\")\n"
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
