//! `proportion_test` and `chi2_test` (release 0.7.0, step 10; Language Reference, sections 4
//! and 10.2; open decision #91): the reference (`tests/reference/r/counts.json`: R's
//! `prop.test` uncorrected and corrected, `binom.test`, `chisq.test`, Newcombe's interval by
//! hand) within the closed-form class through the package's own functions; the edge cases (an
//! empty cell, a proportion of 0 or 1, the refusals); properties (swapping the groups flips the
//! sign; the cross-table's statistic is symmetric in its two variables; frequency weights
//! scaled by k scale chi-squared by k); and snapshots of every form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::chi2_test::chi2_test;
use zeff_stats::proportion_test::proportion_test;
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

type Body = fn(&mut CallContext<'_>, Args) -> Result<Value, Fail>;

/// A call of `body` with the arguments by slot; the messages it gave come back with the result.
fn call(
    body: Body,
    frame: &Frame,
    slots: Vec<(usize, Value)>,
) -> Result<(TestResult, Vec<Diagnostic>), Fail> {
    let mut values: Vec<Option<Value>> = vec![None; 12];
    values[0] = Some(Value::Frame(frame.clone()));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    let args = Args {
        values,
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let result = {
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text: "d.test(x)",
            columns_used: Vec::new(),
            frame: None,
            callee_span: Span::new(2, 8),
            arg_spans: vec![None; 12],
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
        };
        body(&mut ctx, args)?
    };
    match result {
        Value::TestResult(r) => Ok(((*r).clone(), messages)),
        other => panic!("not a TestResult: {other:?}"),
    }
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

fn var(name: &str) -> Value {
    Value::Var(name.to_string())
}

fn word(w: &str) -> Value {
    Value::Word(w.to_string())
}

fn number(v: &Json) -> f64 {
    match v {
        Json::String(s) => match s.as_str() {
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            "NA" => f64::NAN,
            s => s.parse().unwrap(),
        },
        other => other.as_f64().unwrap(),
    }
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/counts.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64) -> bool {
    (got - want).abs() <= 1e-6 * want.abs().max(1e-9)
}

/// A Bool column of `x` successes in `n` trials.
fn trials(x: usize, n: usize) -> Column {
    Column::bools((0..n).map(|i| Some(i < x)))
}

/// The shipped wages: union (0/1), female (0/1) and industry (text).
fn wages() -> Frame {
    let text =
        std::fs::read_to_string(root().join("packages/tier1/datasets/data/wages.csv")).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (union, female, industry) = (at("union"), at("female"), at("industry"));
    let (mut u, mut f, mut i) = (Vec::new(), Vec::new(), Vec::new());
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        u.push(cells[union].parse::<i64>().ok());
        f.push(cells[female].parse::<i64>().ok());
        i.push(Some(cells[industry].to_string()));
    }
    frame_of(vec![
        ("union", Column::ints(u)),
        ("female", Column::ints(f)),
        ("industry", Column::strs(i)),
    ])
}

fn alternative_of(name: &str) -> Option<&'static str> {
    if name.contains("less") {
        Some("less")
    } else if name.contains("greater") {
        Some("greater")
    } else {
        None
    }
}

fn level_of(name: &str) -> Option<f64> {
    if name.ends_with("_99") {
        Some(0.99)
    } else {
        None
    }
}

// ----- the reference -----

#[test]
fn one_proportion_agrees_with_prop_test_and_binom_test() {
    let r = reference();
    let w = wages();
    let mut problems = Vec::new();
    for case in r["one_sample"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let x = case["x"].as_u64().unwrap() as usize;
        let n = case["n"].as_u64().unwrap() as usize;
        let frame = if name.starts_with("union") {
            w.clone()
        } else {
            frame_of(vec![("union", trials(x, n))])
        };
        for interval in ["wilson", "wald", "exact"] {
            let mut slots = vec![
                (1, var("union")),
                (2, Value::Float(number(&case["p"]))),
                (4, word(interval)),
            ];
            if name.contains("corrected") {
                slots.push((3, Value::Bool(true)));
            }
            if let Some(a) = alternative_of(name) {
                slots.push((6, word(a)));
            }
            if let Some(l) = level_of(name) {
                slots.push((7, Value::Float(l)));
            }
            let (result, _) = call(proportion_test, &frame, slots).unwrap();
            let mut want = |label: &str, got: f64, expected: f64| {
                if !close(got, expected) {
                    problems.push(format!(
                        "{name} {interval} {label}: Zeff {got:e}, R {expected:e}"
                    ));
                }
            };
            want("z", result.statistic.value, number(&case["z"]));
            want("p", result.p_value, number(&case["p_value"]));
            let estimate = result.estimate.as_ref().unwrap();
            want("proportion", estimate.value, x as f64 / n as f64);
            let (lower, upper) = estimate.ci.unwrap();
            let ci = case[interval].as_array().unwrap();
            want("lower", lower, number(&ci[0]));
            want("upper", upper, number(&ci[1]));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn two_proportions_agree_with_prop_test_and_newcombe() {
    let r = reference();
    let w = wages();
    let mut problems = Vec::new();
    for case in r["two_sample"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let xs: Vec<usize> = case["x"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let ns: Vec<usize> = case["n"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let frame = if name.starts_with("union") {
            w.clone()
        } else {
            let mut outcome = Vec::new();
            let mut group = Vec::new();
            for g in 0..2 {
                for i in 0..ns[g] {
                    outcome.push(Some(i < xs[g]));
                    group.push(Some(g as i64));
                }
            }
            frame_of(vec![
                ("union", Column::bools(outcome)),
                ("female", Column::ints(group)),
            ])
        };
        for interval in ["wilson", "wald"] {
            let mut slots = vec![(1, var("union")), (5, var("female")), (4, word(interval))];
            if name.contains("corrected") {
                slots.push((3, Value::Bool(true)));
            }
            if let Some(a) = alternative_of(name) {
                slots.push((6, word(a)));
            }
            if let Some(l) = level_of(name) {
                slots.push((7, Value::Float(l)));
            }
            let (result, _) = call(proportion_test, &frame, slots).unwrap();
            let mut want = |label: &str, got: f64, expected: f64| {
                if !close(got, expected) {
                    problems.push(format!(
                        "{name} {interval} {label}: Zeff {got:e}, R {expected:e}"
                    ));
                }
            };
            want("z", result.statistic.value, number(&case["z"]));
            want("p", result.p_value, number(&case["p_value"]));
            let estimate = result.estimate.as_ref().unwrap();
            want("delta", estimate.value, number(&case["delta"]));
            let (lower, upper) = estimate.ci.unwrap();
            let key = if interval == "wald" {
                "wald"
            } else {
                "newcombe"
            };
            let ci = case[key].as_array().unwrap();
            want("lower", lower, number(&ci[0]));
            want("upper", upper, number(&ci[1]));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn chi_squared_agrees_with_chisq_test() {
    let r = reference();
    let w = wages();
    let mut problems = Vec::new();
    for case in r["chi2"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let (frame, slots): (Frame, Vec<(usize, Value)>) = match name {
            "industry_female" => (w.clone(), vec![(1, var("industry")), (2, var("female"))]),
            "union_female" => (w.clone(), vec![(1, var("union")), (2, var("female"))]),
            "union_female_corrected" => (
                w.clone(),
                vec![
                    (1, var("union")),
                    (2, var("female")),
                    (3, Value::Bool(true)),
                ],
            ),
            "small_empty_cell" => (
                frame_of(vec![
                    ("a", Column::strs(["x", "x", "y", "y", "z"].map(Some))),
                    ("b", Column::ints([1, 2, 1, 2, 1].map(Some))),
                ]),
                vec![(1, var("a")), (2, var("b"))],
            ),
            "female_half" => (
                w.clone(),
                vec![
                    (1, var("female")),
                    (4, Value::List(vec![Value::Float(0.5), Value::Float(0.5)])),
                ],
            ),
            "industry_equal" => (
                w.clone(),
                vec![
                    (1, var("industry")),
                    (4, Value::List(vec![Value::Float(1.0 / 12.0); 12])),
                ],
            ),
            other => panic!("unknown case {other}"),
        };
        let (result, messages) = call(chi2_test, &frame, slots).unwrap();
        let mut want = |label: &str, got: f64, expected: f64| {
            if !close(got, expected) {
                problems.push(format!("{name} {label}: Zeff {got:e}, R {expected:e}"));
            }
        };
        want("chi2", result.statistic.value, number(&case["statistic"]));
        want("df", result.statistic.df[0], number(&case["df"]));
        want("p", result.p_value, number(&case["p_value"]));
        let low = case["expected"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| number(e) < 5.0)
            .count();
        let warned = messages.iter().any(|m| m.code == "W0211");
        assert_eq!(
            warned,
            low > 0,
            "{name}: W0211 against {low} low expected counts"
        );
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the edge cases -----

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn proportions_of_zero_and_one_have_bounded_intervals() {
    let none = frame_of(vec![("y", trials(0, 10))]);
    let all = frame_of(vec![("y", trials(10, 10))]);
    for interval in ["wilson", "wald", "exact"] {
        let (r0, _) = call(
            proportion_test,
            &none,
            vec![(1, var("y")), (2, Value::Float(0.5)), (4, word(interval))],
        )
        .unwrap();
        let (lower, upper) = r0.estimate.as_ref().unwrap().ci.unwrap();
        assert_eq!(lower, 0.0, "{interval}");
        // Wald's interval collapses to a point at a proportion of 0 or 1, its known flaw.
        if interval == "wald" {
            assert_eq!(upper, 0.0);
        } else {
            assert!(upper > 0.0 && upper < 1.0, "{interval}: {upper}");
        }
        assert_eq!(r0.estimate.as_ref().unwrap().value, 0.0);
        let (r1, _) = call(
            proportion_test,
            &all,
            vec![(1, var("y")), (2, Value::Float(0.5)), (4, word(interval))],
        )
        .unwrap();
        let (lower, upper) = r1.estimate.as_ref().unwrap().ci.unwrap();
        assert_eq!(upper, 1.0, "{interval}");
        if interval == "wald" {
            assert_eq!(lower, 1.0);
        } else {
            assert!(lower > 0.0 && lower < 1.0, "{interval}: {lower}");
        }
        assert!(r1.statistic.value > 0.0 && r0.statistic.value < 0.0);
    }
}

#[test]
fn the_refusals_of_both_tests() {
    for (script, code, phrase) in [
        (
            "frame(y = [1, 2, 3]).proportion_test(y, p: 0.5)\n",
            "Z0401",
            "holds 2",
        ),
        (
            "frame(s = [\"a\", \"b\"]).proportion_test(s, p: 0.5)\n",
            "Z0401",
            "is a Str",
        ),
        (
            "frame(y = [true, false]).proportion_test(y, p: 1.5)\n",
            "Z0401",
            "between 0 and 1",
        ),
        (
            "frame(y = [true, false]).proportion_test(y)\n",
            "Z0401",
            "needs `p:`",
        ),
        (
            "frame(y = [true, false], g = [0, 1]).proportion_test(y, p: 0.5, group: g)\n",
            "Z0401",
            "not both",
        ),
        (
            "frame(y = [true, false, true, false], g = [0, 0, 1, 1]).proportion_test(y, group: g, ci: exact)\n",
            "Z0401",
            "no exact interval",
        ),
        (
            "frame(y = [true, false, true], g = [0, 1, 2]).proportion_test(y, group: g)\n",
            "Z0412",
            "has 3 levels",
        ),
        (
            "frame(y = [true, false], w = [1.5, 2.5]).proportion_test(y, p: 0.5, weights: w, weight_type: analytic)\n",
            "Z0401",
            "frequency weights, not analytic",
        ),
        (
            "frame(y = [true, false], w = [1, 2]).proportion_test(y, p: 0.5, weights: w)\n",
            "Z0411",
            "sampling weights",
        ),
        (
            "frame(y = [NA, NA]).proportion_test(y, p: 0.5)\n",
            "Z0219",
            "holds only `NA`",
        ),
        (
            "frame(a = [1, 2], b = [1, 2]).chi2_test(a, b, p: [0.5, 0.5])\n",
            "Z0401",
            "not both",
        ),
        (
            "frame(a = [1, 2]).chi2_test(a)\n",
            "Z0401",
            "needs a second variable",
        ),
        (
            "frame(a = [1, 1], b = [1, 2]).chi2_test(a, b)\n",
            "Z0401",
            "at least two values of `a`",
        ),
        (
            "frame(a = [1, 2, 3]).chi2_test(a, p: [0.5, 0.5])\n",
            "Z0401",
            "has 2 proportions, but `a` has 3 values: 1, 2, 3",
        ),
        (
            "frame(a = [1, 2]).chi2_test(a, p: [0.5, 0.3])\n",
            "Z0401",
            "sum to 0.8",
        ),
        (
            "frame(a = [1, 2]).chi2_test(a, p: [\"x\", \"y\"])\n",
            "Z0401",
            "holds a Str",
        ),
        (
            "frame(a = [1, 2], w = [1.5, 2.5]).chi2_test(a, p: [0.5, 0.5], weights: w, weight_type: analytic)\n",
            "Z0401",
            "frequency weights, not analytic",
        ),
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}\n{stderr}");
        assert!(stderr.contains(code), "{script}\n{stderr}");
        assert!(stderr.contains(phrase), "{script}\n{stderr}");
    }
}

#[test]
fn an_empty_cell_warns_and_na_rows_are_dropped() {
    let (exit, stdout, stderr) = go(
        "let f = frame(a = [\"x\", \"x\", \"y\", \"y\", \"z\", NA], b = [1, 2, 1, 2, 1, 1])\nf.chi2_test(a, b)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stderr.contains("warning[W0211]: 6 of 6 expected counts are below 5"),
        "{stderr}"
    );
    assert!(
        stderr.contains("1 row with NA in a or b dropped"),
        "{stderr}"
    );
    assert!(stdout.contains("chi2(2) = 0.83   p = 0.659"), "{stdout}");
}

// ----- properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]
    #[test]
    fn swapping_the_groups_flips_the_sign_and_the_table_is_symmetric(
        outcomes in prop::collection::vec(any::<bool>(), 4..60),
        groups in prop::collection::vec(0i64..2, 4..60),
        k in 2u64..6,
    ) {
        let n = outcomes.len().min(groups.len());
        let y: Vec<Option<bool>> = outcomes[..n].iter().map(|b| Some(*b)).collect();
        let g: Vec<Option<i64>> = groups[..n].iter().map(|v| Some(*v)).collect();
        let swapped: Vec<Option<i64>> = g.iter().map(|v| v.map(|x| 1 - x)).collect();
        let frame = frame_of(vec![("y", Column::bools(y.clone())), ("g", Column::ints(g.clone())), ("h", Column::ints(swapped))]);
        let first = call(proportion_test, &frame, vec![(1, var("y")), (5, var("g"))]);
        let Ok((a, _)) = first else { return Ok(()); }; // one group only, or no trials: refused by design
        let (b, _) = call(proportion_test, &frame, vec![(1, var("y")), (5, var("h"))]).unwrap();
        prop_assert!((a.statistic.value + b.statistic.value).abs() <= 1e-9, "z {} and {}", a.statistic.value, b.statistic.value);
        prop_assert!((a.p_value - b.p_value).abs() <= 1e-12);
        let (la, ua) = a.estimate.as_ref().unwrap().ci.unwrap();
        let (lb, ub) = b.estimate.as_ref().unwrap().ci.unwrap();
        prop_assert!((la + ub).abs() <= 1e-9 && (ua + lb).abs() <= 1e-9, "intervals {:?} and {:?}", (la, ua), (lb, ub));
        // The cross-table's statistic does not depend on which variable is the row. An outcome
        // with one value has no cross-table, which `chi2_test` refuses by design.
        if y.iter().all(|b| *b == Some(true)) || y.iter().all(|b| *b == Some(false)) {
            return Ok(());
        }
        let yi: Vec<Option<i64>> = y.iter().map(|b| b.map(i64::from)).collect();
        let table = frame_of(vec![("y", Column::ints(yi)), ("g", Column::ints(g))]);
        let (ab, _) = call(chi2_test, &table, vec![(1, var("y")), (2, var("g"))]).unwrap();
        let (ba, _) = call(chi2_test, &table, vec![(1, var("g")), (2, var("y"))]).unwrap();
        prop_assert!((ab.statistic.value - ba.statistic.value).abs() <= 1e-9 * ab.statistic.value.abs().max(1.0));
        prop_assert_eq!(ab.statistic.df, ba.statistic.df);
        // Frequency weights of k on every row multiply chi-squared by k.
        let weights = Column::ints(vec![Some(k as i64); n]);
        let weighted = frame_of(vec![("y", table.table.column("y").unwrap().clone()), ("g", table.table.column("g").unwrap().clone()), ("w", weights)]);
        let (scaled, _) = call(chi2_test, &weighted, vec![(1, var("y")), (2, var("g")), (5, var("w")), (6, word("frequency"))]).unwrap();
        prop_assert!((scaled.statistic.value - k as f64 * ab.statistic.value).abs() <= 1e-9 * (k as f64 * ab.statistic.value).abs().max(1.0), "{} against {} × {}", scaled.statistic.value, k, ab.statistic.value);
    }
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages)\nwages.proportion_test(union, p: 0.3)\nwages.proportion_test(union, p: 0.3, correct: true, ci: exact, alternative: greater)\nwages.proportion_test(union, group: female)\nwages.proportion_test(union, group: female, ci: wald, correct: true, level: 0.99)\nwages.proportion_test(union, group: female, alternative: less)\nwages.chi2_test(industry, female)\nwages.chi2_test(union, female, correct: true)\nwages.chi2_test(female, p: [0.5, 0.5])\nlet small = frame(a = [\"x\", \"x\", \"y\", \"y\", \"z\"], b = [1, 2, 1, 2, 1])\nsmall.chi2_test(a, b)\nlet coin = frame(heads = [true, true, false, true, true, true, false, true, true, false], w = [1, 1, 1, 2, 1, 1, 1, 1, 1, 1])\ncoin.proportion_test(heads, p: 0.5, ci: exact)\ncoin.proportion_test(heads, p: 0.5, ci: wald, alternative: less)\ncoin.proportion_test(heads, p: 0.5, weights: w, weight_type: frequency)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(y = [1, 2, 3]).proportion_test(y, p: 0.5)\n",
        "frame(s = [\"a\", \"b\"]).proportion_test(s, p: 0.5)\n",
        "frame(y = [true, false]).proportion_test(y, p: 1.5)\n",
        "frame(y = [true, false]).proportion_test(y)\n",
        "frame(y = [true, false], g = [0, 1]).proportion_test(y, p: 0.5, group: g)\n",
        "frame(y = [true, false, true, false], g = [0, 0, 1, 1]).proportion_test(y, group: g, ci: exact)\n",
        "frame(y = [true, false], w = [1.5, 2.5]).proportion_test(y, p: 0.5, weights: w, weight_type: analytic)\n",
        "frame(a = [1, 2], b = [1, 2]).chi2_test(a, b, p: [0.5, 0.5])\n",
        "frame(a = [1, 2]).chi2_test(a)\n",
        "frame(a = [1, 1], b = [1, 2]).chi2_test(a, b)\n",
        "frame(a = [1, 2, 3]).chi2_test(a, p: [0.5, 0.5])\n",
        "frame(a = [1, 2]).chi2_test(a, p: [0.5, 0.3])\n",
        "frame(a = [1, 2]).chi2_test(a, p: [\"x\", \"y\"])\n",
        "frame(a = [1, 2], w = [1.5, 2.5]).chi2_test(a, p: [0.5, 0.5], weights: w, weight_type: analytic)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
