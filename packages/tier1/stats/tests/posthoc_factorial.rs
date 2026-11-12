//! `posthoc` after the factorial `anova` (release 0.7.0, step 19b; Language Reference, section
//! 4; open decision #104, option A): the reference (`tests/reference/r/posthoc.json`, key
//! `marginal`: emmeans' `pairs(emmeans(lm(), ~ term))` and `confint` under each adjustment, on
//! thirty models: balanced and unbalanced layouts, the terms in both orders, a covariate alone,
//! in an interaction and inside `log()`, three factors, an empty cell, rows with `NA`,
//! frequency and analytic weights), the closed forms within a relative 1e-9 and Tukey's figures
//! within the accuracy of R's `ptukey` and `qtukey`; on a balanced layout, `TukeyHSD`; the
//! properties (one factor alone gives the one-way form on the pooled variance; neither the
//! order of the terms nor the type of the sums of squares changes a row; a balanced layout
//! compares the plain means; frequency weights repeat rows; every figure is in range and the
//! methods keep their order); the edge cases (an empty cell, a factor of two levels, `term:`
//! naming no term, `term:` left out); and snapshots of every form and every message.

use std::rc::Rc;

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{FollowUp, Frame, ModelFit, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::anova::anova;
use zeff_stats::marginal;
use zeff_stats::posthoc::{Method, posthoc};
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The slots of `anova`: data, var, group, equal_var, weights, weight_type, type, effect.
const SLOTS: usize = 8;
const GROUP: usize = 2;
const EQUAL_VAR: usize = 3;
const WEIGHTS: usize = 4;
const WEIGHT_TYPE: usize = 5;
const TYPE: usize = 6;

/// A function called with its arguments by slot: its result and the notes it raised.
fn invoke(
    body: Body,
    values: Vec<Option<Value>>,
    text: &str,
) -> Result<(TestResult, Vec<Diagnostic>), Fail> {
    let slots = values.len();
    let args = Args {
        values,
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let value = {
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text,
            columns_used: Vec::new(),
            frame: None,
            group_rows: None,
            callee_span: Span::new(2, 7),
            arg_spans: vec![None; slots],
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
        };
        body(&mut ctx, args)?
    };
    match value {
        Value::TestResult(r) => Ok(((*r).clone(), messages)),
        other => panic!("not a TestResult: {other:?}"),
    }
}

/// `anova` of the formula `text` on `frame`, with the options given by slot.
fn fit(frame: &Frame, text: &str, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; SLOTS];
    values[0] = Some(Value::Frame(frame.clone()));
    values[1] = Some(Value::Formula(formula(text)));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    invoke(anova, values, text).map(|(r, _)| r)
}

/// The model an `anova` of a formula hands on.
fn model(test: &TestResult) -> &ModelFit {
    match test.follow_up.as_deref() {
        Some(FollowUp::Factorial(fit)) => fit,
        other => panic!("not the factorial form: {other:?}"),
    }
}

/// One row of a `posthoc` result.
#[derive(Debug, Clone)]
struct Row {
    term: String,
    difference: f64,
    std_error: f64,
    t: f64,
    df: f64,
    p: f64,
    ci: Option<(f64, f64)>,
}

/// `posthoc` on a result: its rows, its notes and the result itself.
fn follow(
    test: &TestResult,
    term: Option<&str>,
    word: Option<&str>,
) -> Result<(Vec<Row>, Vec<Diagnostic>, TestResult), Fail> {
    let values = vec![
        Some(Value::TestResult(Rc::new(test.clone()))),
        word.map(|w| Value::Word(w.to_string())),
        term.map(|t| Value::Term(t.to_string())),
    ];
    let (result, notes) = invoke(posthoc, values, "a.posthoc(method: holm, term: a)")?;
    assert_eq!(result.method, "posthoc");
    assert_eq!(result.level, 0.95);
    assert_eq!(result.n, test.n);
    assert!(result.follow_up.is_none());
    let rows = result
        .terms
        .iter()
        .map(|row| {
            let estimate = row.estimate.as_ref().unwrap();
            assert_eq!(estimate.label, row.term);
            assert_eq!(row.statistic.name, "t");
            assert!(row.effect.is_none());
            Row {
                term: row.term.clone(),
                difference: estimate.value,
                std_error: row.std_error.unwrap(),
                t: row.statistic.value,
                df: row.statistic.df[0],
                p: row.p_value,
                ci: estimate.ci,
            }
        })
        .collect::<Vec<Row>>();
    assert_eq!(result.table.as_ref().unwrap().rows.len(), rows.len());
    Ok((rows, notes, result))
}

/// The rows alone, of a call that must succeed.
fn rows_of(test: &TestResult, term: &str, word: &str) -> Vec<Row> {
    follow(test, Some(term), Some(word)).unwrap().0
}

fn frame_of(columns: Vec<(&str, Column)>) -> Frame {
    let table = Columns::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap();
    Frame::new(table)
}

/// A category column of `levels`, each row by the position of its level.
fn category(levels: &[&str], codes: impl Iterator<Item = Option<i32>>) -> Column {
    Column::categories(levels.iter().map(|l| (*l).to_string()).collect(), codes).unwrap()
}

fn number(v: &Json) -> Option<f64> {
    match v {
        Json::String(s) => match s.as_str() {
            "NA" => None,
            s => Some(s.parse().unwrap()),
        },
        other => Some(other.as_f64().unwrap()),
    }
}

fn text_of(v: &Json) -> Option<String> {
    match v {
        Json::String(s) if s == "NA" => None,
        Json::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

fn values(v: &Json) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| number(x).unwrap())
        .collect()
}

fn strings(v: &Json) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/posthoc.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// The frame of a reference case: the factors as categories whose levels are the values seen,
/// in alphabetical order as R's `factor`; frequency weights as whole numbers; the rest as
/// numbers.
fn case_frame(case: &Json) -> Frame {
    let factors = strings(&case["factors"]);
    let frequency = case["weight_type"].as_str() == Some("frequency");
    let weights = text_of(&case["weights"]);
    let columns = case["columns"].as_object().unwrap();
    frame_of(
        columns
            .iter()
            .map(|(name, values)| {
                let values = values.as_array().unwrap();
                let column = if factors.contains(name) {
                    let cells: Vec<Option<String>> = values.iter().map(text_of).collect();
                    let mut levels: Vec<&str> =
                        cells.iter().flatten().map(String::as_str).collect();
                    levels.sort_unstable();
                    levels.dedup();
                    category(
                        &levels,
                        cells.iter().map(|cell| {
                            cell.as_ref()
                                .map(|c| levels.iter().position(|l| l == c).unwrap() as i32)
                        }),
                    )
                } else if frequency && weights.as_deref() == Some(name.as_str()) {
                    Column::ints(values.iter().map(|v| number(v).map(|w| w as i64)))
                } else {
                    Column::floats(values.iter().map(number)).0
                };
                (name.as_str(), column)
            })
            .collect(),
    )
}

/// The `anova` of a reference case.
fn case_fit(case: &Json) -> TestResult {
    let mut slots = Vec::new();
    if let Some(name) = text_of(&case["weights"]) {
        slots.push((WEIGHTS, Value::Var(name)));
        slots.push((
            WEIGHT_TYPE,
            Value::Word(case["weight_type"].as_str().unwrap().to_string()),
        ));
    }
    fit(&case_frame(case), case["formula"].as_str().unwrap(), slots).unwrap()
}

// ----- the reference -----

/// Collects the disagreements of one case with R.
struct Against<'a> {
    label: String,
    problems: &'a mut Vec<String>,
}

impl Against<'_> {
    /// `got` within a relative `relative` of `want`, or within `absolute` of it.
    fn close(&mut self, what: &str, got: f64, want: f64, relative: f64, absolute: f64) {
        // A figure that is not a number is never within the tolerance, so it is reported.
        let within = (got - want).abs() <= relative * want.abs() + absolute;
        if !within {
            self.problems
                .push(format!("{} {what}: Zeff {got:e}, R {want:e}", self.label));
        }
    }
}

#[test]
fn every_method_agrees_with_emmeans_on_thirty_models() {
    let reference = reference();
    let cases = reference["marginal"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    let mut problems: Vec<String> = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let text = case["formula"].as_str().unwrap();
        let term = strings(&case["term"]);
        let written = term.join(":");
        let label = format!("case {index}, `{text}`, term {written}");
        let fitted = case_fit(case);
        let fit = model(&fitted);
        // The means themselves, and which of them the rows determine.
        let factors: Vec<usize> = term
            .iter()
            .map(|name| fit.factors.iter().position(|f| f.name == *name).unwrap())
            .collect();
        let means = marginal::means(fit, &factors);
        let estimable: Vec<bool> = case["estimable"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_bool().unwrap())
            .collect();
        assert_eq!(
            means.iter().map(|m| m.label.clone()).collect::<Vec<_>>(),
            strings(&case["means"]),
            "{label}"
        );
        for (i, mean) in means.iter().enumerate() {
            assert_eq!(mean.estimable, estimable[i], "{label}: {}", mean.label);
            if estimable[i] {
                Against {
                    label: format!("{label}, mean {}", mean.label),
                    problems: &mut problems,
                }
                .close(
                    "estimate",
                    mean.estimate,
                    number(&case["estimates"][i]).unwrap(),
                    1e-9,
                    1e-12,
                );
            }
        }
        let left_out = estimable.iter().filter(|e| !**e).count();
        let terms: Vec<String> = strings(&case["first"])
            .iter()
            .zip(strings(&case["second"]))
            .map(|(a, b)| format!("{a} − {b}"))
            .collect();
        let df = number(&case["df"]).unwrap();
        let difference = values(&case["difference"]);
        let se = values(&case["se"]);
        let t = values(&case["t"]);
        for word in Method::WORDS {
            let mut against = Against {
                label: format!("{label}, {word}"),
                problems: &mut problems,
            };
            let (rows, notes, result) = follow(&fitted, Some(&written), Some(word)).unwrap();
            // A mean that cannot be estimated is left out with a note, and only then.
            let said = notes
                .iter()
                .filter(|note| note.message.contains("cannot be estimated"))
                .count();
            assert_eq!(said, usize::from(left_out > 0), "{label} {word}");
            // The test of the whole is the F of the term in the table of the `anova`.
            let own = fitted
                .terms
                .iter()
                .find(|row| {
                    let mut a: Vec<&str> = row.term.split(':').collect();
                    let mut b: Vec<&str> = term.iter().map(String::as_str).collect();
                    a.sort_unstable();
                    b.sort_unstable();
                    a == b
                })
                .unwrap();
            assert_eq!(result.statistic, own.statistic, "{label}");
            assert_eq!(result.p_value.to_bits(), own.p_value.to_bits(), "{label}");
            assert_eq!(rows.len(), terms.len(), "{label} {word}");
            let p = values(&case[word]);
            let limits = match word {
                "tukey" | "scheffe" | "bonferroni" | "sidak" | "none" => {
                    Some(&case[format!("ci_{word}").as_str()])
                }
                _ => None,
            };
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(row.term, terms[i], "{label}");
                assert_eq!(row.df, df, "{label}");
                against.close("difference", row.difference, difference[i], 1e-9, 1e-12);
                against.close("std.error", row.std_error, se[i], 1e-9, 0.0);
                against.close("t", row.t, t[i], 1e-9, 1e-12);
                // R's `ptukey` is out by up to 3e-5 on 5 degrees of freedom and 1e-7 on 20, and
                // `qtukey` stops at about four digits (the help page of `dist.tukey`); emmeans
                // writes Šidák's p-value as 1 − (1 − p)^m, which loses the digits of a small p.
                let (relative, absolute, limit) = match word {
                    "tukey" => (
                        0.0,
                        if df >= 20.0 { 1e-6 } else { 1e-4 },
                        1e-3 * row.std_error,
                    ),
                    "sidak" => (1e-9, 1e-13, 1e-9 * row.std_error),
                    _ => (1e-9, 1e-14, 1e-9 * row.std_error),
                };
                against.close("p", row.p, p[i], relative, absolute);
                match limits {
                    Some(limits) => {
                        let (lo, hi) = row.ci.unwrap();
                        let relative = if word == "tukey" { 0.0 } else { 1e-9 };
                        against.close(
                            "lower",
                            lo,
                            number(&limits["lower"][i]).unwrap(),
                            relative,
                            limit,
                        );
                        against.close(
                            "upper",
                            hi,
                            number(&limits["upper"][i]).unwrap(),
                            relative,
                            limit,
                        );
                    }
                    None => assert!(
                        row.ci.is_none(),
                        "{label} {word}: a step-wise method has no interval"
                    ),
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} disagreements with R:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn a_balanced_layout_gives_the_comparisons_of_tukeyhsd() {
    let reference = reference();
    let mut checked = 0;
    for case in reference["marginal"].as_array().unwrap() {
        let Some(hsd) = case.get("hsd") else {
            continue;
        };
        checked += 1;
        let term = strings(&case["term"]).join(":");
        let rows = rows_of(&case_fit(case), &term, "tukey");
        let difference = values(&hsd["difference"]);
        let p = values(&hsd["p"]);
        let lower = values(&hsd["lower"]);
        let upper = values(&hsd["upper"]);
        assert_eq!(rows.len(), p.len());
        for (i, row) in rows.iter().enumerate() {
            let (lo, hi) = row.ci.unwrap();
            assert!((row.difference - difference[i]).abs() <= 1e-9, "{term}");
            // 18 degrees of freedom: R's `ptukey` holds four decimals there.
            assert!((row.p - p[i]).abs() <= 1e-4, "{term}: {} {}", row.p, p[i]);
            assert!((lo - lower[i]).abs() <= 1e-3 * row.std_error, "{term}");
            assert!((hi - upper[i]).abs() <= 1e-3 * row.std_error, "{term}");
        }
    }
    assert_eq!(checked, 2);
}

// ----- the numeric parts -----

#[test]
fn a_numeric_part_is_held_at_the_means_of_its_variables() {
    let reference = reference();
    let cases = reference["marginal"].as_array().unwrap();
    let column = |case: &Json, name: &str| -> Vec<f64> { values(&case["columns"][name]) };
    // A plain covariate: its mean.
    let plain = &cases[13];
    assert_eq!(plain["formula"], "y ~ a * x");
    let x = column(plain, "x");
    let mean = x.iter().sum::<f64>() / x.len() as f64;
    let fitted = case_fit(plain);
    let numbers = &model(&fitted).numbers;
    assert_eq!(numbers.len(), 1);
    assert_eq!(numbers[0].0, "x");
    assert!((numbers[0].1 - mean).abs() <= 1e-12, "{}", numbers[0].1);
    // Inside a function: the function of the mean, not the mean of the function.
    let logged = &cases[16];
    assert_eq!(logged["formula"], "y ~ a * log(x)");
    let x = column(logged, "x");
    let mean = x.iter().sum::<f64>() / x.len() as f64;
    let mean_of_logs = x.iter().map(|v| v.ln()).sum::<f64>() / x.len() as f64;
    let fitted = case_fit(logged);
    let numbers = &model(&fitted).numbers;
    assert_eq!(numbers[0].0, "log(x)");
    assert!(
        (numbers[0].1 - mean.ln()).abs() <= 1e-12,
        "{}",
        numbers[0].1
    );
    assert!((numbers[0].1 - mean_of_logs).abs() > 1e-3);
    // Weighted: the weighted mean, under frequency and under analytic weights.
    for (index, weight) in [(27, "w"), (29, "v")] {
        let case = &cases[index];
        assert_eq!(case["formula"], "y ~ a * x + c");
        let x = column(case, "x");
        let w = column(case, weight);
        let mean = x.iter().zip(&w).map(|(x, w)| x * w).sum::<f64>() / w.iter().sum::<f64>();
        let fitted = case_fit(case);
        let numbers = &model(&fitted).numbers;
        assert!(
            (numbers[0].1 - mean).abs() <= 1e-12,
            "{weight}: {}",
            numbers[0].1
        );
    }
    // A `Bool` is a number in a model, held at the share of its rows that are true; a logical
    // operation on it cannot be computed from that share, so it is held at its own mean.
    let d = frame_of(vec![
        (
            "y",
            Column::floats([1.0, 2.5, 2.0, 4.0, 3.5, 5.0, 4.5, 7.0].map(Some)).0,
        ),
        (
            "a",
            category(&["p", "q"], [0, 0, 0, 0, 1, 1, 1, 1].into_iter().map(Some)),
        ),
        (
            "s",
            Column::bools([true, false, false, true, true, true, false, true].map(Some)),
        ),
    ]);
    let fitted = fit(&d, "y ~ a * s", Vec::new()).unwrap();
    assert_eq!(model(&fitted).numbers, vec![("s".to_string(), 0.625)]);
    let fitted = fit(&d, "y ~ a * !s", Vec::new()).unwrap();
    assert_eq!(model(&fitted).numbers, vec![("!s".to_string(), 0.375)]);
}

// ----- the properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

const LEVELS: [&str; 3] = ["p", "q", "r"];

type Line = (f64, i32, i32, f64, i64);

/// Rows of an outcome, two category codes, a covariate and a whole weight.
fn lines() -> impl Strategy<Value = Vec<Line>> {
    prop::collection::vec(
        (-3.0..3.0f64, 0..3i32, 0..3i32, -2.0..2.0f64, 1..4i64),
        16..40,
    )
}

fn layout(lines: &[Line]) -> Frame {
    frame_of(vec![
        ("y", Column::floats(lines.iter().map(|r| Some(r.0))).0),
        ("a", category(&LEVELS, lines.iter().map(|r| Some(r.1)))),
        ("b", category(&LEVELS, lines.iter().map(|r| Some(r.2)))),
        ("x", Column::floats(lines.iter().map(|r| Some(r.3))).0),
        ("w", Column::ints(lines.iter().map(|r| Some(r.4)))),
    ])
}

/// Two sets of rows agree: the same comparisons, and every figure within a relative 1e-8.
fn same_rows(a: &[Row], b: &[Row]) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.len(), b.len());
    let near = |x: f64, y: f64| (x - y).abs() <= 1e-8 * y.abs().max(1.0);
    for (r, s) in a.iter().zip(b) {
        prop_assert_eq!(&r.term, &s.term);
        prop_assert!(
            near(r.difference, s.difference),
            "difference {} against {}",
            r.difference,
            s.difference
        );
        prop_assert!(
            near(r.std_error, s.std_error),
            "std.error {} against {}",
            r.std_error,
            s.std_error
        );
        prop_assert!(near(r.t, s.t), "t {} against {}", r.t, s.t);
        prop_assert!(near(r.df, s.df), "df {} against {}", r.df, s.df);
        prop_assert!(near(r.p, s.p), "p {} against {}", r.p, s.p);
        match (r.ci, s.ci) {
            (Some(c), Some(d)) => prop_assert!(near(c.0, d.0) && near(c.1, d.1)),
            (None, None) => {}
            _ => prop_assert!(false, "an interval on one side only"),
        }
    }
    Ok(())
}

/// Two calls agree: the same rows, or the same refusal.
fn same_outcome(
    a: Result<(Vec<Row>, Vec<Diagnostic>, TestResult), Fail>,
    b: Result<(Vec<Row>, Vec<Diagnostic>, TestResult), Fail>,
) -> Result<(), TestCaseError> {
    match (a, b) {
        (Ok(a), Ok(b)) => same_rows(&a.0, &b.0),
        (Err(a), Err(b)) => {
            prop_assert_eq!(a.code, b.code);
            Ok(())
        }
        (a, b) => {
            prop_assert!(false, "one refused: {} {}", a.is_ok(), b.is_ok());
            Ok(())
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    #[test]
    fn one_factor_alone_gives_the_one_way_form_on_the_pooled_variance(lines in lines()) {
        let frame = layout(&lines);
        let Ok(model) = fit(&frame, "y ~ a", Vec::new()) else {
            return Ok(());
        };
        let mut values: Vec<Option<Value>> = vec![None; SLOTS];
        values[0] = Some(Value::Frame(frame.clone()));
        values[1] = Some(Value::Var("y".to_string()));
        values[GROUP] = Some(Value::Var("a".to_string()));
        values[EQUAL_VAR] = Some(Value::Bool(true));
        let Ok((one_way, _)) = invoke(anova, values, "d.anova(y, group: a, equal_var: true)") else {
            return Ok(());
        };
        for word in Method::WORDS {
            // `term:` may be left out: the model has one term of categories.
            let after_formula = follow(&model, None, Some(word)).unwrap().0;
            let after_groups = follow(&one_way, None, Some(word)).unwrap().0;
            same_rows(&after_formula, &after_groups)?;
        }
    }

    #[test]
    fn neither_the_order_of_the_terms_nor_the_type_changes_a_row(
        lines in lines(),
        word in prop::sample::select(Method::WORDS.to_vec()),
    ) {
        let frame = layout(&lines);
        let Ok(written) = fit(&frame, "y ~ a * b + x", Vec::new()) else {
            return Ok(());
        };
        let turned = fit(&frame, "y ~ x + b * a", Vec::new()).unwrap();
        for term in ["a", "b", "a:b"] {
            same_outcome(
                follow(&written, Some(term), Some(word)),
                follow(&turned, Some(term), Some(word)),
            )?;
        }
        // The type of the sums of squares changes the table of the `anova` and no comparison;
        // type 3 is refused when a cell is empty.
        for kind in [1, 3] {
            let Ok(other) = fit(&frame, "y ~ a * b + x", vec![(TYPE, Value::Int(kind))]) else {
                continue;
            };
            for term in ["a", "a:b"] {
                match (follow(&written, Some(term), Some(word)), follow(&other, Some(term), Some(word))) {
                    (Ok(a), Ok(b)) => {
                        prop_assert_eq!(a.0.len(), b.0.len());
                        for (r, s) in a.0.iter().zip(&b.0) {
                            prop_assert_eq!(r.difference.to_bits(), s.difference.to_bits());
                            prop_assert_eq!(r.std_error.to_bits(), s.std_error.to_bits());
                            prop_assert_eq!(r.p.to_bits(), s.p.to_bits());
                        }
                    }
                    (Err(a), Err(b)) => prop_assert_eq!(a.code, b.code),
                    _ => prop_assert!(false, "one type refused"),
                }
            }
        }
        // The cells in the other order are the same comparisons under other names.
        if let (Ok(a), Ok(b)) = (
            follow(&written, Some("a:b"), Some(word)),
            follow(&written, Some("b:a"), Some(word)),
        ) {
            prop_assert_eq!(a.0.len(), b.0.len());
            let mut seen: Vec<f64> = a.0.iter().map(|r| r.difference.abs()).collect();
            let mut turned: Vec<f64> = b.0.iter().map(|r| r.difference.abs()).collect();
            seen.sort_by(f64::total_cmp);
            turned.sort_by(f64::total_cmp);
            for (x, y) in seen.iter().zip(&turned) {
                prop_assert!((x - y).abs() <= 1e-9 * x.max(1.0));
            }
        }
    }

    #[test]
    fn a_balanced_layout_compares_the_plain_means(
        a_levels in 2..4usize,
        b_levels in 2..4usize,
        replicates in 2..4usize,
        outcome in prop::collection::vec(-3.0..3.0f64, 27),
    ) {
        let cells = a_levels * b_levels;
        let n = cells * replicates;
        let a: Vec<i32> = (0..n).map(|i| (i / (b_levels * replicates)) as i32).collect();
        let b: Vec<i32> = (0..n).map(|i| ((i / replicates) % b_levels) as i32).collect();
        let y: Vec<f64> = (0..n).map(|i| outcome[i % 27] + 0.37 * (i as f64).sin()).collect();
        let frame = frame_of(vec![
            ("y", Column::floats(y.iter().map(|v| Some(*v))).0),
            ("a", category(&LEVELS, a.iter().map(|c| Some(*c)))),
            ("b", category(&LEVELS, b.iter().map(|c| Some(*c)))),
        ]);
        let Ok(full) = fit(&frame, "y ~ a * b", Vec::new()) else {
            return Ok(());
        };
        let rows = rows_of(&full, "a", "none");
        let mean = |level: usize| {
            let chosen: Vec<f64> = (0..n).filter(|&i| a[i] as usize == level).map(|i| y[i]).collect();
            chosen.iter().sum::<f64>() / chosen.len() as f64
        };
        let per_level = (b_levels * replicates) as f64;
        let ms = model(&full).residual_ms;
        let mut at = 0;
        for (i, first) in LEVELS.iter().enumerate().take(a_levels) {
            for (j, second) in LEVELS.iter().enumerate().take(a_levels).skip(i + 1) {
                let row = &rows[at];
                at += 1;
                prop_assert_eq!(&row.term, &format!("{first} − {second}"));
                prop_assert!((row.difference - (mean(i) - mean(j))).abs() <= 1e-9);
                let se = (ms * 2.0 / per_level).sqrt();
                prop_assert!((row.std_error - se).abs() <= 1e-9 * se);
                prop_assert_eq!(row.df, (n - cells) as f64);
            }
        }
        prop_assert_eq!(at, rows.len());
    }

    #[test]
    fn frequency_weights_repeat_rows(lines in lines()) {
        let frame = layout(&lines);
        let written: Vec<Line> = lines
            .iter()
            .flat_map(|line| std::iter::repeat_n(*line, line.4 as usize))
            .collect();
        let repeated = layout(&written);
        let weights = vec![
            (WEIGHTS, Value::Var("w".to_string())),
            (WEIGHT_TYPE, Value::Word("frequency".to_string())),
        ];
        // The covariate interacts with the term compared, so the value it is held at matters.
        let Ok(weighted) = fit(&frame, "y ~ a * x + b", weights) else {
            return Ok(());
        };
        let plain = fit(&repeated, "y ~ a * x + b", Vec::new()).unwrap();
        for word in ["tukey", "holm", "sidak"] {
            same_outcome(
                follow(&weighted, Some("a"), Some(word)),
                follow(&plain, Some("a"), Some(word)),
            )?;
        }
    }

    #[test]
    fn every_figure_is_in_range_and_the_methods_keep_their_order(lines in lines()) {
        let frame = layout(&lines);
        // Without the interaction every mean can be estimated, whatever cells are empty.
        let Ok(model) = fit(&frame, "y ~ a + b + x", Vec::new()) else {
            return Ok(());
        };
        let by = |word: &str| rows_of(&model, "a", word);
        let none = by("none");
        for word in Method::WORDS {
            let rows = by(word);
            prop_assert_eq!(rows.len(), none.len());
            let single_step = matches!(word, "tukey" | "scheffe" | "bonferroni" | "sidak" | "none");
            for (row, raw) in rows.iter().zip(&none) {
                prop_assert!((0.0..=1.0).contains(&row.p), "{}: p {}", word, row.p);
                prop_assert!(row.p >= raw.p - 1e-12, "{} below the unadjusted p-value", word);
                prop_assert!(row.std_error > 0.0);
                prop_assert_eq!(row.ci.is_some(), single_step);
                if let Some((lo, hi)) = row.ci {
                    let half = (hi - lo) / 2.0;
                    prop_assert!((lo + half - row.difference).abs() <= 1e-9 * half.max(1.0));
                    // An interval leaves out zero exactly when its p-value is below 0.05, away
                    // from the edge where rounding decides.
                    if (row.p - 0.05).abs() > 1e-6 {
                        prop_assert_eq!(
                            lo > 0.0 || hi < 0.0,
                            row.p < 0.05,
                            "{}: {} [{}, {}]",
                            word,
                            row.p,
                            lo,
                            hi
                        );
                    }
                }
            }
        }
        let p = |word: &str| by(word).iter().map(|r| r.p).collect::<Vec<f64>>();
        let ordered = |larger: &str, smaller: &str| -> Result<(), TestCaseError> {
            for (a, b) in p(larger).iter().zip(p(smaller)) {
                prop_assert!(*a >= b - 1e-12, "{} {} below {} {}", larger, a, smaller, b);
            }
            Ok(())
        };
        ordered("bonferroni", "holm")?;
        ordered("holm", "hochberg")?;
        ordered("hochberg", "hommel")?;
        ordered("bonferroni", "sidak")?;
        ordered("by", "bh")?;
        ordered("hochberg", "bh")?;
        ordered("scheffe", "tukey")?;
    }
}

// ----- the edge cases and the refusals -----

/// Twelve rows on two categories: `a` of two levels, `b` of three, with a covariate.
fn small() -> Frame {
    frame_of(vec![
        (
            "y",
            Column::floats([1.0, 2.5, 2.0, 4.0, 3.5, 5.0, 4.5, 7.0, 6.0, 5.5, 8.0, 7.5].map(Some))
                .0,
        ),
        (
            "a",
            category(
                &["p", "q"],
                [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1].into_iter().map(Some),
            ),
        ),
        (
            "b",
            category(
                &["u", "v", "w"],
                [0, 0, 1, 1, 2, 2, 0, 0, 1, 1, 2, 2].into_iter().map(Some),
            ),
        ),
        (
            "x",
            Column::floats([0.5, 1.5, 1.0, 2.0, 0.0, 2.5, 1.5, 0.5, 2.0, 1.0, 3.0, 0.5].map(Some))
                .0,
        ),
    ])
}

#[test]
fn a_factor_of_two_levels_gives_one_comparison_whose_square_is_the_type_3_f() {
    let fitted = fit(&small(), "y ~ a * b + x", vec![(TYPE, Value::Int(3))]).unwrap();
    for word in Method::WORDS {
        let (rows, _, result) = follow(&fitted, Some("a"), Some(word)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].term, "p − q");
        // Type 3 tests the equality of the marginal means, which with two of them is this t.
        let f = result.statistic.value;
        assert!(
            (rows[0].t * rows[0].t - f).abs() <= 1e-9 * f,
            "{word}: t² {} against F {f}",
            rows[0].t * rows[0].t
        );
        // One comparison: every method gives the p-value of that F.
        assert!(
            (rows[0].p - result.p_value).abs() <= 1e-9,
            "{word}: {} against {}",
            rows[0].p,
            result.p_value
        );
    }
}

#[test]
fn an_empty_cell_leaves_its_means_out_and_says_so() {
    // No row with a = q and b = w.
    let d = frame_of(vec![
        (
            "y",
            Column::floats([1.0, 2.5, 2.0, 4.0, 3.5, 5.0, 4.5, 7.0, 6.0, 5.5, 6.5].map(Some)).0,
        ),
        (
            "a",
            category(
                &["p", "q"],
                [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1].into_iter().map(Some),
            ),
        ),
        (
            "b",
            category(
                &["u", "v", "w"],
                [0, 0, 1, 1, 2, 2, 0, 0, 1, 1, 0].into_iter().map(Some),
            ),
        ),
    ]);
    let full = fit(&d, "y ~ a * b", Vec::new()).unwrap();
    // The cells: five of the six, ten comparisons, the family the five.
    let (rows, notes, _) = follow(&full, Some("a:b"), Some("bonferroni")).unwrap();
    assert_eq!(rows.len(), 10);
    assert!(rows.iter().all(|row| !row.term.contains("q:w")));
    assert_eq!(notes.len(), 1);
    assert_eq!(
        notes[0].message,
        "the marginal mean of `q:w` cannot be estimated (no row holds the combination) and is left out; the comparisons are among the other 5"
    );
    let plain = rows_of(&full, "a:b", "none");
    for (adjusted, raw) in rows.iter().zip(&plain) {
        assert!((adjusted.p - (raw.p * 10.0).min(1.0)).abs() <= 1e-12);
    }
    // The levels of b: w meets the empty cell, so u and v are compared.
    let (rows, notes, _) = follow(&full, Some("b"), None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].term, "u − v");
    assert!(
        notes[0].message.starts_with(
            "the marginal mean of `w` cannot be estimated (a combination with `a` holds no row, and the model has their interaction) and is left out; the comparisons are among the other 2"
        ),
        "{}",
        notes[0].message
    );
    // The levels of a: q meets it, so one mean is left and there is no pair.
    let failed = follow(&full, Some("a"), None).unwrap_err();
    assert_eq!(failed.code, "Z0802");
    assert_eq!(
        failed.message,
        "fewer than two marginal means of `a` can be estimated, so there is no pair to compare"
    );
    // Without the interaction every mean is estimated.
    let additive = fit(&d, "y ~ a + b", Vec::new()).unwrap();
    let (rows, notes, _) = follow(&additive, Some("b"), None).unwrap();
    assert_eq!(rows.len(), 3);
    assert!(notes.is_empty());
}

#[test]
fn the_term_is_checked_against_the_model() {
    let fitted = fit(&small(), "y ~ a * b + a:x + x", Vec::new()).unwrap();
    // Left out with several terms of categories.
    let failed = follow(&fitted, None, None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`posthoc` needs `term:` here: `y ~ a * b + a:x + x` has 3 terms whose levels it can compare, `a`, `b` and `a:b`"
    );
    // No term of that name.
    let failed = follow(&fitted, Some("c"), None).unwrap_err();
    assert_eq!(failed.code, "Z0414");
    assert_eq!(failed.message, "`y ~ a * b + a:x + x` has no term `c`");
    let failed = follow(&fitted, Some("a:b:x"), None).unwrap_err();
    assert_eq!(failed.code, "Z0414");
    // A term with a numeric part, and a numeric term.
    let failed = follow(&fitted, Some("a:x"), None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`a:x` has the numeric part `x`, so it has slopes and no levels to compare"
    );
    let failed = follow(&fitted, Some("x"), None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`x` is numeric, so it has a slope and no levels to compare"
    );
    // The parts of an interaction in either order, and spaces around the colon.
    let straight = rows_of(&fitted, "a:b", "holm");
    let spaced = rows_of(&fitted, "a : b", "holm");
    assert_eq!(straight.len(), 15);
    for (r, s) in straight.iter().zip(&spaced) {
        assert_eq!(r.term, s.term);
        assert_eq!(r.p.to_bits(), s.p.to_bits());
    }
    let turned = rows_of(&fitted, "b:a", "holm");
    assert_eq!(turned[0].term, "u:p − u:q");
    assert_eq!(straight[0].term, "p:u − p:v");
    // One term of categories: `term:` may be left out, or name it.
    let single = fit(&small(), "y ~ b + x", Vec::new()).unwrap();
    let (unnamed, _, _) = follow(&single, None, Some("sidak")).unwrap();
    let named = rows_of(&single, "b", "sidak");
    assert_eq!(unnamed.len(), 3);
    for (r, s) in unnamed.iter().zip(&named) {
        assert_eq!(r.p.to_bits(), s.p.to_bits());
    }
    // No category at all.
    let slopes = fit(&small(), "y ~ x", Vec::new()).unwrap();
    let failed = follow(&slopes, None, None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`posthoc` compares the levels of a category, but no term of `y ~ x` is a category or an interaction of categories"
    );
    // After the one-way form `term:` has nothing to choose.
    let mut values: Vec<Option<Value>> = vec![None; SLOTS];
    values[0] = Some(Value::Frame(small()));
    values[1] = Some(Value::Var("y".to_string()));
    values[GROUP] = Some(Value::Var("b".to_string()));
    let (one_way, _) = invoke(anova, values, "d.anova(y, group: b)").unwrap();
    let failed = follow(&one_way, Some("b"), None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`term:` chooses among the terms of a formula, but this `anova` compares the groups of `b` alone"
    );
}

#[test]
fn a_term_inside_a_larger_one_says_what_its_differences_average() {
    let fitted = fit(&small(), "y ~ a * b + a:x + x", Vec::new()).unwrap();
    let (_, notes, _) = follow(&fitted, Some("a"), None).unwrap();
    assert_eq!(notes.len(), 1);
    assert!(
        notes[0]
            .message
            .starts_with("`a` is also part of `a:b` and `a:x`: "),
        "{}",
        notes[0].message
    );
    let (_, notes, _) = follow(&fitted, Some("b"), None).unwrap();
    assert!(
        notes[0].message.starts_with("`b` is also part of `a:b`: "),
        "{}",
        notes[0].message
    );
    // The cells are part of nothing larger.
    let (_, notes, _) = follow(&fitted, Some("a:b"), None).unwrap();
    assert!(notes.is_empty());
    // Nor is a factor of a model without interactions.
    let additive = fit(&small(), "y ~ a + b + x", Vec::new()).unwrap();
    let (_, notes, _) = follow(&additive, Some("b"), None).unwrap();
    assert!(notes.is_empty());
}

// ----- the snapshots -----

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const DATA: &str = "let d = frame(gain = [4.2, 3.9, 4.6, 4.1, 4.4, 5.1, 5.6, 4.9, 5.4, 5.8, 4.5, 4.0, 4.8, 4.3, 5.0, 4.7], dose = [\"low\", \"low\", \"low\", \"low\", \"mid\", \"mid\", \"mid\", \"mid\", \"mid\", \"high\", \"high\", \"high\", \"high\", \"high\", \"low\", \"mid\"], diet = [\"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"b\", \"b\"], age = [3, 5, 4, 6, 5, 7, 6, 4, 5, 3, 6, 4, 7, 5, 4, 6], lot = [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2], w = [1, 2, 1, 1, 3, 1, 1, 2, 1, 1, 1, 2, 1, 1, 1, 1]).add(dose = category(dose, levels: [\"low\", \"mid\", \"high\"]), diet = category(diet))\n";

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "{DATA}let fit = d.anova(gain ~ dose * diet + age)\nfit.posthoc(term: dose)\nfit.posthoc(term: dose, method: scheffe)\nfit.posthoc(term: dose, method: bonferroni)\nfit.posthoc(term: dose, method: sidak)\nfit.posthoc(term: dose, method: none)\nfit.posthoc(term: dose, method: holm)\nfit.posthoc(term: dose, method: hochberg)\nfit.posthoc(term: dose, method: hommel)\nfit.posthoc(term: dose, method: bh)\nfit.posthoc(term: dose, method: by)\nfit.posthoc(term: diet)\nfit.posthoc(term: \"dose:diet\")\nfit.posthoc(term: \"diet:dose\", method: holm)\nd.anova(gain ~ dose + age).posthoc()\nd.anova(gain ~ dose * age).posthoc(method: holm)\nd.anova(gain ~ dose * log(age)).posthoc()\nd.anova(gain ~ dose * category(lot)).posthoc(term: lot, method: none)\nd.anova(gain ~ dose * category(lot), type: 3).posthoc(term: \"dose:lot\", method: sidak)\nd.anova(gain ~ dose * diet, weights: w, weight_type: frequency).posthoc(term: dose)\nd.anova(gain ~ dose + diet, weights: w, weight_type: analytic).posthoc(term: diet, method: bonferroni)\nd.where(!(dose == \"high\" && diet == \"a\")).anova(gain ~ dose * diet).posthoc(term: \"dose:diet\", method: bonferroni)\nd.where(!(dose == \"high\" && diet == \"a\")).anova(gain ~ dose * diet).posthoc(term: dose)\nlet p = fit.posthoc(term: dose, method: bonferroni)\np.estimates()\np.ci()\np.statistic()\np.df()\np.p_values()\np.effect_size()\np.summary()\nlet h = fit.posthoc(term: \"dose:diet\", method: holm)\nh.estimates()\nh.ci()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for statement in [
        "d.anova(gain ~ dose * diet + age).posthoc()",
        "d.anova(gain ~ dose + diet).posthoc(method: holm)",
        "d.anova(gain ~ dose * diet + age).posthoc(term: lot)",
        "d.anova(gain ~ dose * diet + age).posthoc(term: \"dose:lot\")",
        "d.anova(gain ~ dose * diet + age).posthoc(term: age)",
        "d.anova(gain ~ dose * age).posthoc(term: \"dose:age\")",
        "d.anova(gain ~ age + lot).posthoc()",
        "d.anova(gain ~ age + lot).posthoc(term: age)",
        "d.anova(gain, group: dose).posthoc(term: dose)",
        "d.anova(gain ~ dose * diet).posthoc(term: dose, method: dunnett)",
        "d.anova(gain ~ dose * diet).posthoc(term: 3)",
        "d.anova(gain ~ dose * diet).posthoc(dose)",
        "d.where(!(dose == \"high\" && diet == \"a\") && dose != \"mid\").anova(gain ~ dose * diet).posthoc(term: diet)",
        "d.anova(gain ~ dose * diet).p_values(term: dose)",
    ] {
        let (exit, _, stderr) = go(&format!("{DATA}{statement}\n"));
        assert_eq!(exit, 1, "{statement}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
