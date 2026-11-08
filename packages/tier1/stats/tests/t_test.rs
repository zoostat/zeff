//! `t_test` (release 0.7.0, step 9; Language Reference, sections 4, 6 and 10.2; open decision
//! #91): the reference (`tests/reference/r/t_test.json`, R's `t.test` in its three forms on the
//! shipped wages and R's sleep data, the effect sizes by hand) within the closed-form class
//! through the package's own function; the edge cases of the checklist (zero variance, one row,
//! all `NA`) and the refusals; two properties (shifting both groups changes nothing but the
//! means; scaling scales the difference and its interval); and snapshots of every form and
//! every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::t_test::t_test;
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

fn with_context<T>(f: impl FnOnce(&mut CallContext<'_>) -> T) -> T {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "d.t_test(x)",
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
    f(&mut ctx)
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

/// The arguments of `t_test` by slot: data, var, b, mu, group, equal_var, effect, paired,
/// alternative, level, weights, weight_type.
fn call(frame: &Frame, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
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
    with_context(|ctx| t_test(ctx, args)).map(|v| match v {
        Value::TestResult(r) => (*r).clone(),
        other => panic!("not a TestResult: {other:?}"),
    })
}

fn var(name: &str) -> Value {
    Value::Var(name.to_string())
}

fn word(w: &str) -> Value {
    Value::Word(w.to_string())
}

fn number(v: &Json) -> f64 {
    match v.as_str().unwrap() {
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        "NA" => f64::NAN,
        s => s.parse().unwrap(),
    }
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/t_test.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Within the closed-form class: a relative 1e-6, with an absolute floor for values near zero
/// and for p-values deep in a tail, which both sides compute from their own tail functions.
fn close(got: f64, want: f64) -> bool {
    if got.is_infinite() || want.is_infinite() {
        return got == want;
    }
    (got - want).abs() <= 1e-6 * want.abs().max(1e-9)
}

/// The shipped wages, the columns the reference uses: lwage, female and union.
fn wages() -> Frame {
    let text =
        std::fs::read_to_string(root().join("packages/tier1/datasets/data/wages.csv")).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (wage, female, union) = (at("wage"), at("female"), at("union"));
    let mut lwage = Vec::new();
    let mut f = Vec::new();
    let mut u = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        lwage.push(match cells[wage] {
            "" | "NA" => None,
            w => Some(w.parse::<f64>().unwrap().ln()),
        });
        f.push(cells[female].parse::<i64>().ok());
        u.push(cells[union].parse::<i64>().ok());
    }
    frame_of(vec![
        ("lwage", Column::floats(lwage).0),
        ("female", Column::ints(f)),
        ("union", Column::ints(u)),
    ])
}

fn check(case: &Json, result: &TestResult, mu: Option<f64>) {
    let name = case["name"].as_str().unwrap();
    let mut problems = Vec::new();
    let mut want = |label: &str, got: f64, expected: f64| {
        if !close(got, expected) {
            problems.push(format!("{name} {label}: Zeff {got:e}, R {expected:e}"));
        }
    };
    want("t", result.statistic.value, number(&case["statistic"]));
    want("df", result.statistic.df[0], number(&case["df"]));
    want("p", result.p_value, number(&case["p_value"]));
    let shift = mu.unwrap_or(0.0);
    let estimate = result.estimate.as_ref().unwrap();
    want(
        "estimate",
        estimate.value,
        number(&case["estimate"]) - shift,
    );
    let (lower, upper) = estimate.ci.unwrap();
    want("ci lower", lower, number(&case["ci_lower"]) - shift);
    want("ci upper", upper, number(&case["ci_upper"]) - shift);
    if let Some(effect) = &result.effect {
        let key = if effect.label.starts_with("Cohen") {
            "d"
        } else {
            "g"
        };
        want(key, effect.value, number(&case[key]));
        let (el, eu) = effect.ci.unwrap();
        let ci = case[format!("{key}_ci")].as_array().unwrap();
        want("effect ci lower", el, number(&ci[0]));
        want("effect ci upper", eu, number(&ci[1]));
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the reference -----

#[test]
fn the_three_forms_agree_with_t_test_on_the_wages() {
    let r = reference();
    let w = wages();
    let cases = r["wages"].as_array().unwrap();
    let by_name = |name: &str| {
        cases
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("no case {name}"))
    };
    check(
        by_name("welch_female"),
        &call(&w, vec![(1, var("lwage")), (4, var("female"))]).unwrap(),
        None,
    );
    check(
        by_name("welch_female_less"),
        &call(
            &w,
            vec![(1, var("lwage")), (4, var("female")), (8, word("less"))],
        )
        .unwrap(),
        None,
    );
    check(
        by_name("pooled_union_99"),
        &call(
            &w,
            vec![
                (1, var("lwage")),
                (4, var("union")),
                (5, Value::Bool(true)),
                (6, word("d")),
                (9, Value::Float(0.99)),
            ],
        )
        .unwrap(),
        None,
    );
    check(
        by_name("welch_union_greater_90"),
        &call(
            &w,
            vec![
                (1, var("lwage")),
                (4, var("union")),
                (8, word("greater")),
                (9, Value::Float(0.90)),
            ],
        )
        .unwrap(),
        None,
    );
    check(
        by_name("mu3_greater"),
        &call(
            &w,
            vec![
                (1, var("lwage")),
                (3, Value::Float(3.0)),
                (8, word("greater")),
            ],
        )
        .unwrap(),
        Some(3.0),
    );
    check(
        by_name("mu3_90"),
        &call(
            &w,
            vec![
                (1, var("lwage")),
                (3, Value::Float(3.0)),
                (9, Value::Float(0.9)),
            ],
        )
        .unwrap(),
        Some(3.0),
    );
    check(
        by_name("mu0"),
        &call(&w, vec![(1, var("lwage"))]).unwrap(),
        Some(0.0),
    );
}

#[test]
fn the_paired_form_agrees_with_t_test_on_the_sleep_data() {
    let r = reference();
    let a: Vec<Option<f64>> = r["sleep_a"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Some(number(v)))
        .collect();
    let b: Vec<Option<f64>> = r["sleep_b"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| Some(number(v)))
        .collect();
    let frame = frame_of(vec![("a", Column::floats(a).0), ("b", Column::floats(b).0)]);
    let cases = r["sleep"].as_array().unwrap();
    check(
        &cases[0],
        &call(
            &frame,
            vec![(1, var("a")), (2, var("b")), (7, Value::Bool(true))],
        )
        .unwrap(),
        None,
    );
    check(
        &cases[1],
        &call(
            &frame,
            vec![
                (1, var("a")),
                (2, var("b")),
                (7, Value::Bool(true)),
                (8, word("less")),
            ],
        )
        .unwrap(),
        None,
    );
    check(
        &cases[2],
        &call(
            &frame,
            vec![
                (1, var("a")),
                (2, var("b")),
                (7, Value::Bool(true)),
                (8, word("greater")),
                (9, Value::Float(0.99)),
            ],
        )
        .unwrap(),
        None,
    );
}

#[test]
fn frequency_weights_replicate_rows() {
    let r = reference();
    let frame = frame_of(vec![
        ("x", Column::floats(vec![Some(1.0), Some(2.0), Some(4.0)]).0),
        ("f", Column::ints(vec![Some(2), Some(1), Some(3)])),
    ]);
    let result = call(
        &frame,
        vec![
            (1, var("x")),
            (3, Value::Float(1.0)),
            (10, var("f")),
            (11, word("frequency")),
        ],
    )
    .unwrap();
    check(&r["weighted"], &result, Some(1.0));
    assert_eq!(result.n, 6);
    // Analytic weights all equal are no weights at all.
    let unweighted = call(&frame, vec![(1, var("x"))]).unwrap();
    let analytic = frame_of(vec![
        ("x", Column::floats(vec![Some(1.0), Some(2.0), Some(4.0)]).0),
        ("w", Column::floats(vec![Some(0.5), Some(0.5), Some(0.5)]).0),
    ]);
    let weighted = call(
        &analytic,
        vec![(1, var("x")), (10, var("w")), (11, word("analytic"))],
    )
    .unwrap();
    assert!(close(weighted.statistic.value, unweighted.statistic.value));
    assert!(close(weighted.p_value, unweighted.p_value));
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
fn zero_variance_one_row_and_all_na_are_refused_with_their_reasons() {
    for (script, code, phrase) in [
        (
            "frame(x = [2, 2, 2, 2]).t_test(x)\n",
            "Z0401",
            "all equal, so the t statistic is undefined",
        ),
        (
            "frame(x = [2.5]).t_test(x)\n",
            "Z0401",
            "at least two values of x, but there is one",
        ),
        (
            "frame(x = [NA, NA, 1.5]).t_test(x)\n",
            "Z0401",
            "at least two values of x, but there is one",
        ),
        (
            "frame(x = [1, 2, 3, 4], g = [0, 0, 1, 1]).where(x > 2).t_test(x, group: g)\n",
            "Z0412",
            "has one level",
        ),
        (
            "frame(x = [1, 2, 3, 4], g = [0, 1, 2, 3]).t_test(x, group: g)\n",
            "Z0412",
            "has 4 levels",
        ),
        (
            "frame(x = [1, 2, 3, 4], g = [0, 0, 1, 1]).t_test(x, group: g, mu: 1)\n",
            "Z0401",
            "neither `mu:` nor `paired:`",
        ),
        (
            "frame(x = [1, 2, 3, 4], y = [1, 2, 3, 5]).t_test(x, y)\n",
            "Z0401",
            "needs `paired: true`",
        ),
        (
            "frame(x = [1, 2, 3, 4]).t_test(x, paired: true)\n",
            "Z0401",
            "needs two variables",
        ),
        (
            "frame(x = [1, 2, 3, 4], w = [1, 1, 2, 2]).t_test(x, weights: w)\n",
            "Z0411",
            "sampling weights",
        ),
        (
            "frame(s = [\"a\", \"b\"]).t_test(s)\n",
            "Z0401",
            "must be a Bool, Int or Float variable",
        ),
        (
            "frame(x = [1, 2, 3]).t_test(x, level: 1.5)\n",
            "Z0401",
            "between 0 and 1",
        ),
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}\n{stderr}");
        assert!(stderr.contains(code), "{script}\n{stderr}");
        assert!(stderr.contains(phrase), "{script}\n{stderr}");
    }
    // All `NA` in one group is the group's shortage, named.
    let (exit, _, stderr) = go("frame(x = [1, 2, NA, NA], g = [0, 0, 1, 1]).t_test(x, group: g)\n");
    assert_eq!(exit, 1);
    assert!(
        stderr.contains("at least two values of x in g=1, but there is none"),
        "{stderr}"
    );
}

#[test]
fn na_rows_are_dropped_and_counted_once() {
    let (exit, stdout, stderr) = go(
        "let f = frame(x = [1.5, NA, 2.5, 4.0, 3.5, NA, 1.0], g = [0, 0, 0, 1, 1, 1, NA])\nf.t_test(x, group: g)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stderr.contains("3 rows with NA in x or g dropped"),
        "{stderr}"
    );
    let row = |level: &str| -> Vec<String> {
        stdout
            .lines()
            .find(|l| l.starts_with(&format!("  {level} ")))
            .unwrap_or_else(|| panic!("no row {level} in {stdout}"))
            .split_whitespace()
            .map(str::to_string)
            .collect()
    };
    assert_eq!(row("g=0"), vec!["g=0", "2", "2.000", "0.707"]);
    assert_eq!(row("g=1"), vec!["g=1", "2", "3.750", "0.354"]);
}

// ----- two properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn two_group_frame(a: &[f64], b: &[f64]) -> Frame {
    let mut x = Vec::new();
    let mut g = Vec::new();
    for v in a {
        x.push(Some(*v));
        g.push(Some(0));
    }
    for v in b {
        x.push(Some(*v));
        g.push(Some(1));
    }
    frame_of(vec![("x", Column::floats(x).0), ("g", Column::ints(g))])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]
    #[test]
    fn shifting_both_groups_changes_nothing_but_the_means_and_scaling_scales_the_difference(
        a in prop::collection::vec(-50.0f64..50.0, 3..25),
        b in prop::collection::vec(-50.0f64..50.0, 3..25),
        shift in -1000.0f64..1000.0,
        scale in 0.01f64..100.0,
    ) {
        let base = call(&two_group_frame(&a, &b), vec![(1, var("x")), (4, var("g"))]);
        let Ok(base) = base else { return Ok(()); }; // constant data, refused by design
        let shifted_a: Vec<f64> = a.iter().map(|v| v + shift).collect();
        let shifted_b: Vec<f64> = b.iter().map(|v| v + shift).collect();
        let shifted = call(&two_group_frame(&shifted_a, &shifted_b), vec![(1, var("x")), (4, var("g"))]).unwrap();
        let tol = |x: f64| 1e-6 * x.abs().max(1.0);
        prop_assert!((shifted.statistic.value - base.statistic.value).abs() <= tol(base.statistic.value) * 1e3, "t {} against {}", shifted.statistic.value, base.statistic.value);
        prop_assert!((shifted.p_value - base.p_value).abs() <= 1e-6, "p {} against {}", shifted.p_value, base.p_value);
        let (e0, e1) = (base.estimate.as_ref().unwrap().value, shifted.estimate.as_ref().unwrap().value);
        prop_assert!((e0 - e1).abs() <= tol(e0) * 1e3, "difference {} against {}", e1, e0);
        prop_assert!((base.effect.as_ref().unwrap().value - shifted.effect.as_ref().unwrap().value).abs() <= 1e-6 * 1e3);
        let scaled_a: Vec<f64> = a.iter().map(|v| v * scale).collect();
        let scaled_b: Vec<f64> = b.iter().map(|v| v * scale).collect();
        let scaled = call(&two_group_frame(&scaled_a, &scaled_b), vec![(1, var("x")), (4, var("g"))]).unwrap();
        prop_assert!((scaled.statistic.value - base.statistic.value).abs() <= tol(base.statistic.value), "t under scaling");
        let scaled_estimate = scaled.estimate.as_ref().unwrap();
        prop_assert!((scaled_estimate.value - e0 * scale).abs() <= tol(e0 * scale), "difference under scaling");
        let (l0, u0) = base.estimate.as_ref().unwrap().ci.unwrap();
        let (l1, u1) = scaled_estimate.ci.unwrap();
        prop_assert!((l1 - l0 * scale).abs() <= tol(l0 * scale) && (u1 - u0 * scale).abs() <= tol(u0 * scale), "interval under scaling");
        prop_assert!((scaled.effect.as_ref().unwrap().value - base.effect.as_ref().unwrap().value).abs() <= 1e-6, "g under scaling");
    }
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages).add(lwage = log(wage))\nwages.t_test(lwage, group: female)\nwages.t_test(lwage, mu: 3, alternative: greater)\nwages.t_test(lwage, group: union, equal_var: true, effect: d, level: 0.99)\nwages.t_test(lwage, group: female, alternative: less)\nlet pairs = frame(before = [0.7, -1.6, -0.2, -1.2, -0.1, 3.4, 3.7, 0.8, 0.0, 2.0], after = [1.9, 0.8, 1.1, 0.1, -0.1, 4.4, 5.5, 1.6, 4.6, 3.4])\npairs.t_test(before, after, paired: true)\npairs.t_test(before, after, paired: true, alternative: less, level: 0.9)\nlet w = frame(x = [1.0, 2.0, 4.0], f = [2, 1, 3])\nw.t_test(x, mu: 1, weights: f, weight_type: frequency)\nlet g = frame(x = [1.5, 2.5, 2.0, 4.0, 3.5, 5.0], s = [\"a\", \"a\", \"a\", \"b\", \"b\", \"b\"])\ng.t_test(x, group: s)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [2, 2, 2, 2]).t_test(x)\n",
        "frame(x = [2.5]).t_test(x)\n",
        "frame(x = [1, 2, 3, 4], g = [0, 1, 2, 3]).t_test(x, group: g)\n",
        "frame(x = [1, 2, 3, 4], g = [0, 0, 1, 1]).t_test(x, group: g, mu: 1)\n",
        "frame(x = [1, 2, 3, 4], y = [1, 2, 3, 5]).t_test(x, y)\n",
        "frame(x = [1, 2, 3, 4]).t_test(x, paired: true)\n",
        "frame(x = [1, 2, 3, 4], y = [1, 2, 3, 5]).t_test(x, y, paired: true, mu: 2)\n",
        "frame(x = [1, 2, 3, 4], w = [1, 1, 2, 2]).t_test(x, weights: w)\n",
        "frame(s = [\"a\", \"b\"]).t_test(s)\n",
        "frame(x = [1, 2, 3]).t_test(x, level: 1.5)\n",
        "frame(x = [1, 2, 3]).t_test(y)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
