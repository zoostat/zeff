//! `anova`, the one-way form (release 0.7.0, step 16; Language Reference, sections 4, 6 and
//! 10.2; open decision #91): the reference (`tests/reference/r/anova.json`, R 4.6.0's
//! `oneway.test` for Welch's F and `aov` for the classical F, with unequal variances, ties, a
//! group of one, `NA`, text groups and a tiny p-value) within a relative 1e-9; the kernels'
//! closed forms (two groups give the square of Welch's and of the pooled t; the refusals of the
//! kernel); the properties (F is never negative, the p-value and eta squared lie in [0, 1], the
//! answer does not depend on the order of the rows or on a common shift or scale of the values,
//! and frequency weights repeat rows); the edge cases of the checklist (a group of one, a group
//! of zero variance); and snapshots of every form and every message.
//!
//! `effect:` (step 17b; open decision #99, question 2): omega squared and epsilon squared against
//! the same reference, where they are written out on `aov`'s table and epsilon squared is checked
//! against `summary(lm)$adj.r.squared`; their closed forms in F; the properties (epsilon squared
//! never exceeds eta squared, omega squared lies between 0 and epsilon squared, both are negative
//! exactly when the classical F is below 1, and `effect: eta` is the default); and the edge cases
//! (F below 1 with its note, two groups).

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::anova::{GroupSummary, Undefined, anova, classical, welch};
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

fn with_context<T>(
    values: Vec<Option<Value>>,
    text: &str,
    body: impl FnOnce(&mut CallContext<'_>, Args) -> Result<Value, Fail>,
    unwrap: impl FnOnce(Value) -> T,
) -> Result<T, Fail> {
    let slots = values.len();
    let args = Args {
        values,
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
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
    body(&mut ctx, args).map(unwrap)
}

/// The arguments by slot: data, var, group, equal_var, weights, weight_type, type, effect.
fn call(frame: &Frame, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; 8];
    values[0] = Some(Value::Frame(frame.clone()));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    with_context(values, "d.anova(x, group: g)", anova, |v| match v {
        Value::TestResult(r) => (*r).clone(),
        other => panic!("not a TestResult: {other:?}"),
    })
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

fn equal_var() -> (usize, Value) {
    (3, Value::Bool(true))
}

fn effect(word: &str) -> (usize, Value) {
    (7, Value::Word(word.to_string()))
}

/// The effect size of `anova` on `x` by `g` under `effect: word`, after its label is checked.
fn measure(frame: &Frame, word: &str, form: &[(usize, Value)]) -> f64 {
    let mut slots = vec![(1, var("x")), (2, var("g")), effect(word)];
    slots.extend(form.iter().cloned());
    let result = call(frame, slots).unwrap();
    let estimate = result.effect.as_ref().unwrap();
    let named = match word {
        "omega_f" => "omega squared from F".to_string(),
        word => format!("{word} squared"),
    };
    assert_eq!(estimate.label, named);
    assert!(estimate.ci.is_none());
    estimate.value
}

fn number(v: &Json) -> Option<f64> {
    match v {
        Json::String(s) => match s.as_str() {
            "NA" => None,
            "Inf" => Some(f64::INFINITY),
            "-Inf" => Some(f64::NEG_INFINITY),
            s => Some(s.parse().unwrap()),
        },
        other => Some(other.as_f64().unwrap()),
    }
}

fn floats(v: &Json) -> Vec<Option<f64>> {
    v.as_array().unwrap().iter().map(number).collect()
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/anova.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(1e-300)
}

/// The grouping column of a reference case: text groups as they are, others as Int codes.
fn group_column(case: &Json) -> Column {
    let g = case["g"].as_array().unwrap();
    if case["text_groups"].as_bool().unwrap() {
        Column::strs(g.iter().map(|v| match v.as_str().unwrap() {
            "NA" => None,
            s => Some(s.to_string()),
        }))
    } else {
        Column::ints(g.iter().map(|v| match v.as_str().unwrap() {
            "NA" => None,
            s => Some(s.parse::<i64>().unwrap()),
        }))
    }
}

/// The group summaries of plain values by group code.
fn summaries(groups: &[Vec<f64>]) -> Vec<GroupSummary> {
    groups
        .iter()
        .map(|g| {
            let n = g.len() as f64;
            let mean = g.iter().sum::<f64>() / n;
            let variance = if g.len() > 1 {
                g.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0)
            } else {
                f64::NAN
            };
            GroupSummary { n, mean, variance }
        })
        .collect()
}

// ----- the reference -----

#[test]
fn both_forms_agree_with_oneway_test_and_aov() {
    let r = reference();
    let mut problems = Vec::new();
    let mut check = |label: &str, what: &str, got: f64, want: f64| {
        if !close(got, want, 1e-9) {
            problems.push(format!("{label} {what}: Zeff {got:e}, R {want:e}"));
        }
    };
    let mut welch_defined = 0;
    for (i, case) in r["anova"].as_array().unwrap().iter().enumerate() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
        let n = case["n"].as_u64().unwrap() as usize;
        let groups = case["groups"].as_u64().unwrap() as usize;
        // The classical form, from `aov`.
        let result = call(&frame, vec![(1, var("x")), (2, var("g")), equal_var()]).unwrap();
        let label = format!("anova[{i}] classical");
        check(
            &label,
            "F",
            result.statistic.value,
            number(&case["f"]).unwrap(),
        );
        check(
            &label,
            "df1",
            result.statistic.df[0],
            number(&case["df1"]).unwrap(),
        );
        check(
            &label,
            "df2",
            result.statistic.df[1],
            number(&case["df2"]).unwrap(),
        );
        check(
            &label,
            "p",
            result.p_value,
            number(&case["p_value"]).unwrap(),
        );
        check(
            &label,
            "eta squared",
            result.effect.as_ref().unwrap().value,
            number(&case["eta_squared"]).unwrap(),
        );
        check(
            &label,
            "omega squared",
            measure(&frame, "omega", &[equal_var()]),
            number(&case["omega_squared"]).unwrap(),
        );
        check(
            &label,
            "epsilon squared",
            measure(&frame, "epsilon", &[equal_var()]),
            number(&case["epsilon_squared"]).unwrap(),
        );
        assert_eq!(result.n, n, "{label} n");
        assert_eq!(
            result.table.as_ref().unwrap().rows.len(),
            groups,
            "{label} groups"
        );
        let more = &result.table.as_ref().unwrap().more;
        assert_eq!(more.len(), 1, "{label}: one table of sums of squares");
        // The Welch form, from `oneway.test`, undefined where a group has one value.
        let welch = call(&frame, vec![(1, var("x")), (2, var("g"))]);
        match number(&case["welch_f"]) {
            None => assert!(welch.is_err(), "anova[{i}]: Welch should be undefined"),
            Some(f) => {
                let result = welch.unwrap();
                let label = format!("anova[{i}] Welch");
                welch_defined += 1;
                check(&label, "F", result.statistic.value, f);
                check(
                    &label,
                    "df1",
                    result.statistic.df[0],
                    number(&case["welch_df1"]).unwrap(),
                );
                check(
                    &label,
                    "df2",
                    result.statistic.df[1],
                    number(&case["welch_df2"]).unwrap(),
                );
                check(
                    &label,
                    "p",
                    result.p_value,
                    number(&case["welch_p"]).unwrap(),
                );
                check(
                    &label,
                    "eta squared",
                    result.effect.as_ref().unwrap().value,
                    number(&case["eta_squared"]).unwrap(),
                );
                check(
                    &label,
                    "omega squared",
                    measure(&frame, "omega", &[]),
                    number(&case["omega_squared"]).unwrap(),
                );
                check(
                    &label,
                    "epsilon squared",
                    measure(&frame, "epsilon", &[]),
                    number(&case["epsilon_squared"]).unwrap(),
                );
                assert!(result.table.as_ref().unwrap().more.is_empty());
                assert_eq!(result.n, n, "{label} n");
            }
        }
    }
    assert!(welch_defined >= 12);
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the kernels -----

#[test]
fn two_groups_give_the_square_of_t_and_the_kernel_refuses_what_it_cannot_compute() {
    let x = vec![0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46];
    let y = vec![1.15, 0.88, 0.90, 0.74, 1.21];
    let s = summaries(&[x.clone(), y.clone()]);
    let (n1, n2) = (s[0].n, s[1].n);
    let (m1, m2) = (s[0].mean, s[1].mean);
    let (v1, v2) = (s[0].variance, s[1].variance);
    // Welch's F is Welch's t squared, on the Welch–Satterthwaite degrees of freedom.
    let w = welch(&s).unwrap();
    let (a, b) = (v1 / n1, v2 / n2);
    let t = (m1 - m2) / (a + b).sqrt();
    let df = (a + b) * (a + b) / (a * a / (n1 - 1.0) + b * b / (n2 - 1.0));
    assert!((w.f - t * t).abs() < 1e-12, "{} against {}", w.f, t * t);
    assert_eq!(w.df1, 1.0);
    assert!((w.df2 - df).abs() < 1e-12);
    // The classical F is the pooled t squared on n₁ + n₂ − 2.
    let c = classical(&s).unwrap();
    let pooled = ((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0);
    let t = (m1 - m2) / (pooled * (1.0 / n1 + 1.0 / n2)).sqrt();
    assert!((c.f - t * t).abs() < 1e-12, "{} against {}", c.f, t * t);
    assert_eq!((c.df1, c.df2), (1.0, 13.0));
    // Eta squared is the same decomposition in both forms.
    assert_eq!(w.eta_squared, c.eta_squared);
    assert!((0.0..=1.0).contains(&c.eta_squared));
    assert_eq!(c.eta_squared, c.ss_between / (c.ss_between + c.ss_within));
    // Omega and epsilon squared, written in F: with two groups (F − 1) / (F + N − 1) and
    // (F − 1) / (F + N − 2); epsilon squared is the adjusted R² of the one-way model. Welch's
    // form reports the same two, from the same sums of squares.
    let n = n1 + n2;
    assert!((c.omega_squared - (c.f - 1.0) / (c.f + n - 1.0)).abs() < 1e-12);
    assert!((c.epsilon_squared - (c.f - 1.0) / (c.f + n - 2.0)).abs() < 1e-12);
    let adjusted = 1.0 - (1.0 - c.eta_squared) * (n - 1.0) / (n - 2.0);
    assert!((c.epsilon_squared - adjusted).abs() < 1e-12);
    assert_eq!(c.classical_f, c.f);
    assert_eq!(w.classical_f, c.f);
    assert_eq!(w.omega_squared, c.omega_squared);
    assert_eq!(w.epsilon_squared, c.epsilon_squared);
    assert!(c.f > 1.0 && c.omega_squared > 0.0 && c.omega_squared < c.epsilon_squared);
    assert!(c.epsilon_squared < c.eta_squared);
    // Three groups of unequal size: (k − 1)(F − 1) over (k − 1) F + N − k + 1, and + N − k.
    let three = summaries(&[x.clone(), y.clone(), vec![0.2, 0.9, 1.6, 2.4]]);
    let c3 = classical(&three).unwrap();
    let n3 = 19.0;
    assert!((c3.omega_squared - 2.0 * (c3.f - 1.0) / (2.0 * c3.f + n3 - 2.0)).abs() < 1e-12);
    assert!((c3.epsilon_squared - 2.0 * (c3.f - 1.0) / (2.0 * c3.f + n3 - 3.0)).abs() < 1e-12);
    // The refusals.
    assert_eq!(welch(&s[..1]), Err(Undefined::TooFewGroups));
    assert_eq!(classical(&[]), Err(Undefined::TooFewGroups));
    let one = summaries(&[x.clone(), vec![2.0]]);
    assert_eq!(welch(&one), Err(Undefined::GroupOfOne(1)));
    assert!(classical(&one).is_ok());
    let flat = summaries(&[vec![2.0, 2.0, 2.0], y.clone()]);
    assert_eq!(welch(&flat), Err(Undefined::GroupWithoutSpread(0)));
    assert!(classical(&flat).is_ok());
    let singles = summaries(&[vec![1.0], vec![2.0], vec![3.0]]);
    assert_eq!(classical(&singles), Err(Undefined::NoResidualFreedom));
    let constant = summaries(&[vec![1.0, 1.0], vec![2.0, 2.0]]);
    assert_eq!(classical(&constant), Err(Undefined::NoPooledVariance));
    // No difference between the groups: F = 0 and p = 1.
    let same = summaries(&[vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]]);
    assert_eq!(classical(&same).unwrap().f, 0.0);
    assert_eq!(classical(&same).unwrap().p, 1.0);
    assert_eq!(welch(&same).unwrap().f, 0.0);
    // There eta squared is 0 and the two corrected measures are at their lowest: epsilon squared
    // −(k − 1) / (N − k) = −1/4 and omega squared −(k − 1) / (N − k + 1) = −1/5.
    let lowest = classical(&same).unwrap();
    assert_eq!(lowest.eta_squared, 0.0);
    assert!((lowest.epsilon_squared + 0.25).abs() < 1e-15);
    assert!((lowest.omega_squared + 0.2).abs() < 1e-15);
    assert_eq!(
        welch(&same).unwrap().epsilon_squared,
        lowest.epsilon_squared
    );
    assert_eq!(welch(&same).unwrap().classical_f, 0.0);
}

// ----- the properties -----

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        * 64
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    #[test]
    fn the_statistic_is_in_range_and_ignores_row_order_shift_and_scale(
        values in prop::collection::vec((-3.0..3.0f64, 0..4i64, 1..4i64), 8..40),
        shift in -50.0..50.0f64,
        scale in 0.1..10.0f64,
        seed in 0..1000u64,
    ) {
        // Every group needs two distinct values for Welch's form.
        let mut by_group: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
        for (x, g, _) in &values {
            by_group.entry(*g).or_default().push(*x);
        }
        prop_assume!(by_group.len() >= 2);
        prop_assume!(by_group.values().all(|v| v.len() >= 2 && v.iter().any(|x| *x != v[0])));
        let frame = frame_of(vec![
            ("x", Column::floats(values.iter().map(|(x, _, _)| Some(*x))).0),
            ("g", Column::ints(values.iter().map(|(_, g, _)| Some(*g)))),
            ("w", Column::ints(values.iter().map(|(_, _, w)| Some(*w)))),
        ]);
        let classical_f = call(&frame, vec![(1, var("x")), (2, var("g")), equal_var()])
            .unwrap()
            .statistic
            .value;
        for form in [vec![], vec![equal_var()]] {
            let mut slots = vec![(1, var("x")), (2, var("g"))];
            slots.extend(form.clone());
            let plain = call(&frame, slots.clone()).unwrap();
            prop_assert!(plain.statistic.value >= 0.0);
            prop_assert!((0.0..=1.0).contains(&plain.p_value));
            let eta = plain.effect.as_ref().unwrap().value;
            prop_assert!((0.0..=1.0).contains(&eta));
            // `effect: eta` is the default; epsilon squared never exceeds eta squared; omega
            // squared lies between 0 and epsilon squared; both are negative exactly when the
            // classical F of the same sums of squares is below 1, whichever form is asked for.
            prop_assert_eq!(plain.effect.as_ref().unwrap().label.as_str(), "eta squared");
            prop_assert_eq!(measure(&frame, "eta", &form), eta);
            let omega = measure(&frame, "omega", &form);
            let epsilon = measure(&frame, "epsilon", &form);
            prop_assert!(epsilon <= eta);
            prop_assert!(omega.abs() <= epsilon.abs());
            prop_assert!(omega == 0.0 || (omega < 0.0) == (epsilon < 0.0));
            prop_assert!(epsilon < 1.0 && omega < 1.0);
            if classical_f < 1.0 - 1e-9 {
                prop_assert!(omega < 0.0 && epsilon < 0.0);
            }
            if classical_f > 1.0 + 1e-9 {
                prop_assert!(omega > 0.0 && epsilon > 0.0);
            }
            // With one term the omega squared from F is omega squared itself (open decision
            // #101), under either F.
            prop_assert_eq!(measure(&frame, "omega_f", &form).to_bits(), omega.to_bits());
            // A shuffle of the rows.
            let mut order: Vec<usize> = (0..values.len()).collect();
            let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            for i in (1..order.len()).rev() {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                let j = (state >> 33) as usize % (i + 1);
                order.swap(i, j);
            }
            let shuffled = frame_of(vec![
                ("x", Column::floats(order.iter().map(|&i| Some(values[i].0))).0),
                ("g", Column::ints(order.iter().map(|&i| Some(values[i].1)))),
            ]);
            let moved = call(&shuffled, slots.clone()).unwrap();
            prop_assert!((moved.statistic.value - plain.statistic.value).abs() <= 1e-9 * plain.statistic.value.max(1.0));
            // A common shift and scale of the values leave F, its degrees of freedom and eta squared.
            let changed = frame_of(vec![
                ("x", Column::floats(values.iter().map(|(x, _, _)| Some(shift + scale * x))).0),
                ("g", Column::ints(values.iter().map(|(_, g, _)| Some(*g)))),
            ]);
            let other = call(&changed, slots.clone()).unwrap();
            prop_assert!((other.statistic.value - plain.statistic.value).abs() <= 1e-7 * plain.statistic.value.max(1.0));
            prop_assert!((other.statistic.df[1] - plain.statistic.df[1]).abs() <= 1e-7 * plain.statistic.df[1]);
            prop_assert!((other.effect.as_ref().unwrap().value - eta).abs() <= 1e-7);
            prop_assert!((measure(&changed, "omega", &form) - omega).abs() <= 1e-7);
            prop_assert!((measure(&changed, "epsilon", &form) - epsilon).abs() <= 1e-7);
            // Frequency weights against the rows written out.
            let mut weighted_slots = slots.clone();
            weighted_slots.push((4, var("w")));
            weighted_slots.push((5, Value::Word("frequency".to_string())));
            let weighted = call(&frame, weighted_slots).unwrap();
            let mut long: Vec<(f64, i64)> = Vec::new();
            for (x, g, w) in &values {
                for _ in 0..*w {
                    long.push((*x, *g));
                }
            }
            let expanded = frame_of(vec![
                ("x", Column::floats(long.iter().map(|(x, _)| Some(*x))).0),
                ("g", Column::ints(long.iter().map(|(_, g)| Some(*g)))),
            ]);
            let written = call(&expanded, slots).unwrap();
            let mut weighted_form = form.clone();
            weighted_form.push((4, var("w")));
            weighted_form.push((5, Value::Word("frequency".to_string())));
            for word in ["omega", "epsilon"] {
                let got = measure(&frame, word, &weighted_form);
                let want = measure(&expanded, word, &form);
                prop_assert!((got - want).abs() <= 1e-9, "{} squared: {} against {}", word, got, want);
            }
            prop_assert_eq!(weighted.n, written.n);
            prop_assert!((weighted.statistic.value - written.statistic.value).abs() <= 1e-9 * written.statistic.value.max(1.0));
            prop_assert!((weighted.statistic.df[1] - written.statistic.df[1]).abs() <= 1e-9 * written.statistic.df[1]);
        }
    }
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
fn a_group_of_one_and_a_group_without_spread_are_refused_by_welch_and_admitted_by_the_classical_form()
 {
    // A group of one.
    let (exit, _, stderr) =
        go("frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g)\n");
    assert_eq!(exit, 1);
    assert!(stderr.contains("g=3 has one"), "{stderr}");
    assert!(stderr.contains("`equal_var: true`"), "{stderr}");
    let (exit, stdout, stderr) = go(
        "frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g, equal_var: true)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("F(2, 2) ="), "{stdout}");
    assert!(stdout.contains("g=3     1   9.000      NA"), "{stdout}");
    // A group of zero variance.
    let (exit, _, stderr) =
        go("frame(x = [2, 2, 2, 3.5, 4.5, 6], g = [1, 1, 1, 2, 2, 2]).anova(x, group: g)\n");
    assert_eq!(exit, 1);
    assert!(stderr.contains("in g=1 are all equal"), "{stderr}");
    let (exit, stdout, stderr) = go(
        "frame(x = [2, 2, 2, 3.5, 4.5, 6], g = [1, 1, 1, 2, 2, 2]).anova(x, group: g, equal_var: true)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("F(1, 4) ="), "{stdout}");
}

#[test]
fn f_below_one_gives_a_negative_estimate_that_is_printed_with_a_note() {
    let low = "let low = frame(x = [1, 2, 3, 1, 2, 3.3], g = [1, 1, 1, 2, 2, 2])\n";
    // Eta squared cannot be negative and has no note.
    let (exit, stdout, stderr) = go(&format!("{low}low.anova(x, group: g, equal_var: true)\n"));
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("eta squared = 0.0"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    // The classical form: the F printed is the F below 1.
    for (word, named) in [
        ("omega", "omega squared"),
        ("omega_f", "omega squared from F"),
        ("epsilon", "epsilon squared"),
    ] {
        let (exit, stdout, stderr) = go(&format!(
            "{low}low.anova(x, group: g, equal_var: true, effect: {word})\n"
        ));
        assert_eq!(exit, 0, "{stderr}");
        assert!(stdout.contains(&format!("{named} = -0.")), "{stdout}");
        assert!(!stdout.contains("eta squared"), "{stdout}");
        assert!(
            stderr.contains(&format!(
                "note: F is below 1, so {named} is negative; read it as no evidence of an effect"
            )),
            "{stderr}"
        );
    }
    // In the one-way form the two omegas are one figure, and the line under the table says so.
    let (exit, stdout, stderr) = go(&format!(
        "{low}let a = low.anova(x, group: g, equal_var: true, effect: omega)\nlet b = low.anova(x, group: g, equal_var: true, effect: omega_f)\na.effect_size() == b.effect_size()\nlow.anova(x, group: g, effect: omega_f).effect_size() == a.effect_size()\nb\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.starts_with("true\ntrue\n"), "{stdout}");
    assert!(
        stdout.contains("with one term it is omega squared itself"),
        "{stdout}"
    );
    // Welch's form prints another F, so the note gives the classical one.
    let (exit, stdout, stderr) = go(&format!("{low}low.anova(x, group: g, effect: omega)\n"));
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("omega squared = -0."), "{stdout}");
    assert!(
        stderr.contains("note: the classical F of the same sums of squares is 0.0")
            && stderr.contains("below 1, so omega squared is negative"),
        "{stderr}"
    );
    // The accessor returns the figure as it is printed, not 0.
    let (exit, stdout, stderr) = go(&format!(
        "{low}let a = low.anova(x, group: g, effect: epsilon)\na.effect_size()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.starts_with("-0."), "{stdout}");
    // A word that is not a measure is refused before anything runs.
    let (exit, _, stderr) = go(&format!("{low}low.anova(x, group: g, effect: cohen)\n"));
    assert_eq!(exit, 1);
    assert!(stderr.contains("Z0402"), "{stderr}");
    assert!(stderr.contains("eta, omega, omega_f, epsilon"), "{stderr}");
}

#[test]
fn two_groups_give_the_measures_of_the_pooled_t() {
    // With two groups the classical F is the pooled t squared, so omega squared is
    // (t² − 1) / (t² + N − 1) and epsilon squared (t² − 1) / (t² + N − 2).
    let x = [0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46];
    let y = [1.15, 0.88, 0.90, 0.74, 1.21];
    let frame = frame_of(vec![
        ("x", Column::floats(x.iter().chain(&y).map(|v| Some(*v))).0),
        ("g", Column::ints((0..15).map(|i| Some(i64::from(i >= 10))))),
    ]);
    let s = summaries(&[x.to_vec(), y.to_vec()]);
    let pooled = (9.0 * s[0].variance + 4.0 * s[1].variance) / 13.0;
    let t = (s[0].mean - s[1].mean) / (pooled * (1.0 / 10.0 + 1.0 / 5.0)).sqrt();
    let f = t * t;
    for form in [vec![], vec![equal_var()]] {
        let omega = measure(&frame, "omega", &form);
        let epsilon = measure(&frame, "epsilon", &form);
        assert!((omega - (f - 1.0) / (f + 14.0)).abs() < 1e-12, "{omega}");
        assert!(
            (epsilon - (f - 1.0) / (f + 13.0)).abs() < 1e-12,
            "{epsilon}"
        );
    }
}

#[test]
fn category_groups_and_labelled_groups_are_named_in_the_table() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage, group: industry)\nwages.anova(hours, group: female, equal_var: true)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stdout.contains("anova · Welch one-way · lwage by industry"),
        "{stdout}"
    );
    assert!(stdout.contains("μ₁ = μ₂ = … = μ₁₂"), "{stdout}");
    assert!(
        stdout.contains("F(11, ") && stdout.contains("p < 0.001"),
        "{stdout}"
    );
    assert!(
        stderr.contains("rows with NA in lwage or industry dropped"),
        "{stderr}"
    );
    // The two levels of female are labelled by the dataset's value labels, and the classical
    // form prints its sums of squares.
    let after = stdout.split("hours by female").nth(1).unwrap();
    assert!(after.contains("  group  "), "{after}");
    assert!(after.contains("between groups"), "{after}");
    assert!(after.contains("μ₁ = μ₂"), "{after}");
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.anova(s, group: site)\ng.anova(s, group: site, equal_var: true)\ng.where(site < 3).anova(s, group: site)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.anova(v, group: k)\nt.anova(v, group: k, weights: w, weight_type: frequency)\nt.anova(v, group: k, weights: w, weight_type: analytic, equal_var: true)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 3.5], g = [1, 1, 2, 2, NA, 3, 3, 2])\nm.anova(x, group: g)\nframe(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g, equal_var: true)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage, group: industry)\nwages.where(year == 2010).anova(lwage, group: industry, equal_var: true)\nlet a = wages.anova(lwage, group: industry)\na.statistic()\na.df()\na.effect_size()\na.estimates()\na.summary()\ng.anova(s, group: site, effect: omega)\ng.anova(s, group: site, equal_var: true, effect: epsilon)\nt.anova(v, group: k, weights: w, weight_type: frequency, effect: omega)\nlet low = frame(x = [1, 2, 3, 1, 2, 3.3], g = [1, 1, 1, 2, 2, 2])\nlow.anova(x, group: g, effect: omega)\nlow.anova(x, group: g, equal_var: true, effect: epsilon)\nlet b = low.anova(x, group: g, effect: epsilon)\nb.effect_size()\nb.estimates()\nb.summary()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [1, 2, 3]).anova(x)\n",
        "frame(x = [1, 2, 3]).anova(y, group: x)\n",
        "frame(x = [1, 2, 3], g = [1, 1, 1]).anova(x, group: g)\n",
        "frame(x = [1, NA, 3], g = [NA, 1, NA]).anova(x, group: g)\n",
        "frame(s = [\"a\", \"b\"], g = [1, 2]).anova(s, group: g)\n",
        "frame(x = [1, 2, 3], d = [date(\"2024-01-01\"), date(\"2024-01-02\"), date(\"2024-01-01\")]).anova(x, group: d)\n",
        "frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g)\n",
        "frame(x = [2, 2, 2, 3.5, 4.5, 6], g = [1, 1, 1, 2, 2, 2]).anova(x, group: g)\n",
        "frame(x = [1, 2, 3], g = [1, 2, 3]).anova(x, group: g, equal_var: true)\n",
        "frame(x = [2, 2, 3, 3], g = [1, 1, 2, 2]).anova(x, group: g, equal_var: true)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2], w = [1, 1, 2, 1]).anova(x, group: g, weights: w)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2], w = [1.5, 1, 2, 1]).anova(x, group: g, weights: w, weight_type: frequency)\n",
        "frame(x = [1, 2, 3, 4.5], g = [1, 1, 2, 2]).anova(x, group: g, effect: cohen)\n",
        "frame(x = [1, 2, 3, 4.5], g = [1, 1, 2, 2]).anova(x, group: g, effect: 2)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
