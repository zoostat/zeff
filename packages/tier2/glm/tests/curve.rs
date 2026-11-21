//! `curve` (catalogue row 515; open decision #112; release 0.9.0, step 14): the ROC curve of a
//! binary fit against R on the cases of `tests/reference/r/curve.json` (the points at every
//! distinct fitted probability and the area by the trapezoid rule, in base R, checked there
//! against the Mann–Whitney form), the properties of the area, and the printed forms, the plot
//! and every refusal through the engine.

use proptest::prelude::*;
use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, FollowUp, Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_glm::curve::{curve, roc};
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

/// `fit.curve(newdata:, type: roc)` through the function's body.
fn curve_of(fit: &Rc<Fit>, newdata: Option<&Table>) -> Rc<TestResult> {
    let values = vec![
        Some(Value::Fit(Rc::clone(fit))),
        newdata.map(|table| Value::Frame(Frame::new(table.clone()))),
        Some(Value::Word("roc".to_string())),
    ];
    match call(curve, "m.curve(type: roc)", values) {
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
fn the_curve_and_its_area_agree_with_r() {
    let file = root().join("tests/reference/r/curve.json");
    let reference: Json = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 6);
    for (key, case) in cases {
        let weights = text(&case["weights"]);
        let weights = (!weights.is_empty()).then(|| (weights, text(&case["weight_type"])));
        let new_rows = case["new_rows"].as_bool().unwrap();
        let result = if new_rows {
            let fitted = fit(
                &by_year(&wages, false),
                text(&case["formula"]),
                text(&case["link"]),
                weights,
            );
            curve_of(&fitted, Some(&by_year(&wages, true)))
        } else {
            let fitted = fit(&wages, text(&case["formula"]), text(&case["link"]), weights);
            curve_of(&fitted, None)
        };
        assert_eq!(result.n as u64, case["n"].as_u64().unwrap(), "{key}: N");
        let area = result.estimate.as_ref().expect("the area").value;
        let theirs = number(&case["area"]);
        // The area is a ratio of counts: it moves only if the order of two scores does.
        assert!(
            (area - theirs).abs() <= 1e-12,
            "{key}: the area {area:e} against R's {theirs:e}"
        );
        assert_eq!(
            result.statistic.value, area,
            "{key}: the statistic is the area"
        );
        let Some(FollowUp::Curve {
            thresholds,
            sensitivity,
            specificity,
            ..
        }) = result.follow_up.as_deref()
        else {
            panic!("{key}: no points");
        };
        assert_eq!(
            thresholds.len() as u64,
            case["thresholds"].as_u64().unwrap(),
            "{key}: the distinct thresholds"
        );
        for point in case["points"].as_array().unwrap() {
            let at = point["at"].as_u64().unwrap() as usize - 1;
            let checks = [
                (
                    "threshold",
                    thresholds[at],
                    number(&point["threshold"]),
                    1e-9,
                ),
                (
                    "sensitivity",
                    sensitivity[at],
                    number(&point["sensitivity"]),
                    1e-12,
                ),
                (
                    "specificity",
                    specificity[at],
                    number(&point["specificity"]),
                    1e-12,
                ),
            ];
            for (what, ours, theirs, tolerance) in checks {
                assert!(
                    (ours - theirs).abs() <= tolerance * theirs.abs().max(1.0),
                    "{key}, point {}: {what} {ours:e} against R's {theirs:e}",
                    at + 1
                );
            }
        }
        // The curve ends at (1, 1): every row is at or above the lowest threshold.
        assert_eq!(*sensitivity.last().unwrap(), 1.0, "{key}");
        assert_eq!(*specificity.last().unwrap(), 0.0, "{key}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The curve depends on the order of the scores alone: a rising change of them moves the
    /// thresholds and nothing else, and turning the order round gives one less the area.
    #[test]
    fn the_area_is_that_of_the_order_of_the_scores(
        rows in prop::collection::vec((0u8..40, any::<bool>(), 1u8..4), 2..80),
    ) {
        // Scores on a coarse grid, so that ties are common.
        let scores: Vec<f64> = rows.iter().map(|r| f64::from(r.0) / 8.0).collect();
        let outcomes: Vec<f64> = rows.iter().map(|r| f64::from(u8::from(r.1))).collect();
        let weights: Vec<f64> = rows.iter().map(|r| f64::from(r.2)).collect();
        prop_assume!(outcomes.contains(&1.0) && outcomes.contains(&0.0));
        for weights in [None, Some(weights.as_slice())] {
            let plain = roc(&scores, &outcomes, weights).expect("both outcomes");
            // 3s + 1 and s³ are exact on this grid and rise with s.
            for changed in [
                scores.iter().map(|s| 3.0 * s + 1.0).collect::<Vec<f64>>(),
                scores.iter().map(|s| s * s * s).collect::<Vec<f64>>(),
            ] {
                let other = roc(&changed, &outcomes, weights).expect("both outcomes");
                prop_assert_eq!(&other.sensitivity, &plain.sensitivity);
                prop_assert_eq!(&other.specificity, &plain.specificity);
                prop_assert_eq!(other.area, plain.area);
            }
            let turned: Vec<f64> = scores.iter().map(|s| -s).collect();
            let other = roc(&turned, &outcomes, weights).expect("both outcomes");
            prop_assert!((other.area - (1.0 - plain.area)).abs() <= 1e-14);
            // The area is the share of the pairs of a one and a nought that the score puts in
            // order, a tie counted half: counted pair by pair.
            let weight = |row: usize| weights.map_or(1.0, |w| w[row]);
            let (mut won, mut pairs) = (0.0, 0.0);
            for a in 0..scores.len() {
                for b in 0..scores.len() {
                    if outcomes[a] == 1.0 && outcomes[b] == 0.0 {
                        let both = weight(a) * weight(b);
                        pairs += both;
                        if scores[a] > scores[b] {
                            won += both;
                        } else if scores[a] == scores[b] {
                            won += both / 2.0;
                        }
                    }
                }
            }
            prop_assert_eq!(plain.area, won / pairs);
            // The points rise: more rows are at or above a lower threshold.
            prop_assert!(plain.sensitivity.windows(2).all(|w| w[0] <= w[1]));
            prop_assert!(plain.specificity.windows(2).all(|w| w[0] >= w[1]));
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
fn the_curve_printed_plotted_and_refused() {
    const W: &str = "let wages = dataset(wages)\n";
    let scripts = [
        // The result, read as any result is; its points; its plot.
        format!(
            "{W}let g = wages.glm(union ~ educ + exper + female, family: binomial)\nlet c = g.curve(type: roc)\nc\nc.statistic()\nc.estimates()\nc.n_obs()\nc.p_values()\nc.ci()\ntype_of(c)\nc.plot()\nc.export(\"output/curve.tex\")\nexport(c.plot(), \"output/roc.svg\")\n"
        ),
        // A probit; on new rows; under each kind of weight; one regressor of two values.
        format!(
            "{W}wages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial, link: probit).curve(type: roc)\nlet later = wages.where(year > 2017)\nwages.where(year <= 2017).glm(union ~ educ + exper + female + industry, family: binomial).curve(newdata: later, type: roc)\nwages.glm(union ~ educ + exper + female, family: binomial, weights: hours).curve(type: roc)\nwages.glm(union ~ exper + female, family: binomial, weights: educ, weight_type: frequency).curve(type: roc)\nlet f = wages.glm(union ~ female, family: binomial).curve(type: roc)\nf\nf.estimates()\n"
        ),
        // The refusals.
        format!("{W}wages.glm(union ~ educ, family: binomial).curve()\n"),
        format!("{W}wages.glm(union ~ educ, family: binomial).curve(type: pr)\n"),
        format!("{W}wages.reg(union ~ educ).curve(type: roc)\n"),
        format!("{W}wages.glm(union ~ educ, family: binomial).curve(newdata: 3, type: roc)\n"),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).curve(newdata: wages.select([educ, exper]), type: roc)\n"
        ),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).curve(newdata: wages.where(union == 1), type: roc)\n"
        ),
        format!(
            "{W}wages.glm(union ~ educ, family: binomial).curve(newdata: wages.add(union = union * 2), type: roc)\n"
        ),
        format!("{W}wages.glm(union ~ educ, family: binomial).curve(type: roc).coef_plot()\n"),
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
