//! `kruskal_test` and `p_adjust` (release 0.7.0, steps 12 and 18b; Language Reference, sections
//! 4, 6 and 10.2; open decisions #91 and #99): the reference (`tests/reference/r/kruskal.json`,
//! R 4.6.0's `kruskal.test` with ties, a group of one, `NA` and text groups, and `p.adjust` under
//! its methods with `NA`, beside Šidák's formula) through the package's own functions within a
//! relative 1e-12; the properties (an adjusted p-value is never below the raw one, never above 1,
//! and the order is kept; `none` returns its input; Šidák never exceeds Bonferroni, Hommel never
//! Hochberg, Hochberg never Holm; Hommel is the closed test with Simes's rule; the statistic does
//! not depend on the order of the rows and frequency weights repeat rows); the edge cases; the
//! refusals; and snapshots of every form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::kruskal_test::{kruskal, kruskal_test};
use zeff_stats::p_adjust::{Method, adjust, p_adjust};
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
        callee_span: Span::new(2, 8),
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

/// The arguments by slot: data, var, group, weights, weight_type.
fn call(frame: &Frame, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; 5];
    values[0] = Some(Value::Frame(frame.clone()));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    with_context(
        values,
        "d.kruskal_test(x, group: g)",
        kruskal_test,
        |v| match v {
            Value::TestResult(r) => (*r).clone(),
            other => panic!("not a TestResult: {other:?}"),
        },
    )
}

/// `p_adjust(p, method:)` on a list of optional numbers.
fn call_adjust(p: &[Option<f64>], method: Option<&str>) -> Result<Vec<Option<f64>>, Fail> {
    let list = Value::List(
        p.iter()
            .map(|v| v.map_or(Value::Na, Value::Float))
            .collect(),
    );
    let values = vec![Some(list), method.map(|m| Value::Word(m.to_string()))];
    with_context(values, "p_adjust(p)", p_adjust, |v| match v {
        Value::List(items) => items
            .iter()
            .map(|item| match item {
                Value::Float(x) => Some(*x),
                Value::Na => None,
                other => panic!("not a p-value: {other:?}"),
            })
            .collect(),
        other => panic!("not a list: {other:?}"),
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
    let text = std::fs::read_to_string(root().join("tests/reference/r/kruskal.json")).unwrap();
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

// ----- the reference -----

#[test]
fn the_statistic_and_p_value_agree_with_kruskal_test() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["kruskal"].as_array().unwrap().iter().enumerate() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
        let result = call(&frame, vec![(1, var("x")), (2, var("g"))]).unwrap();
        let label = format!("kruskal[{i}]");
        let statistic = number(&case["statistic"]).unwrap();
        if !close(result.statistic.value, statistic, 1e-12) {
            problems.push(format!(
                "{label} H: Zeff {:e}, R {statistic:e}",
                result.statistic.value
            ));
        }
        let p = number(&case["p_value"]).unwrap();
        if !close(result.p_value, p, 1e-12) {
            problems.push(format!("{label} p: Zeff {:e}, R {p:e}", result.p_value));
        }
        let df = case["df"].as_f64().unwrap();
        if result.statistic.df != vec![df] {
            problems.push(format!(
                "{label} df: Zeff {:?}, R {df}",
                result.statistic.df
            ));
        }
        let n = case["n"].as_u64().unwrap() as usize;
        if result.n != n {
            problems.push(format!("{label} n: Zeff {}, R {n}", result.n));
        }
        let groups = case["groups"].as_u64().unwrap() as usize;
        let shown = result.table.as_ref().unwrap().rows.len();
        if shown != groups {
            problems.push(format!("{label} groups: Zeff {shown}, R {groups}"));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_eight_adjustments_agree_with_p_adjust() {
    let r = reference();
    let cases = r["adjust"].as_array().unwrap();
    assert_eq!(cases.len(), 14, "the cases of kruskal.R");
    let mut problems = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let p = floats(&case["p"]);
        // `sidak` is not a method of `p.adjust`: the script writes its formula beside the others.
        for method in Method::WORDS {
            let got = call_adjust(&p, Some(method)).unwrap();
            let want = floats(&case[method]);
            assert_eq!(got.len(), want.len(), "adjust[{i}] {method}: length");
            if method == "none" {
                assert_eq!(got, p, "adjust[{i}]: `none` returns its input");
            }
            for (j, (g, w)) in got.iter().zip(&want).enumerate() {
                match (g, w) {
                    (None, None) => {}
                    (Some(g), Some(w)) if close(*g, *w, 1e-12) => {}
                    _ => problems.push(format!(
                        "adjust[{i}] {method} item {j}: Zeff {g:?}, R {w:?}"
                    )),
                }
            }
        }
        // The default is Holm.
        assert_eq!(
            call_adjust(&p, None).unwrap(),
            call_adjust(&p, Some("holm")).unwrap()
        );
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// A list of p-values as `p_adjust` takes it, `None` for `NA`.
type Listed<'a> = &'a [Option<f64>];

#[test]
fn sidak_agrees_with_its_formula_by_hand() {
    // 1 − (1 − p)^n, with n the number of p-values that are not `NA`.
    let cases: [(Listed, Listed); 4] = [
        (&[Some(0.5), Some(0.5)], &[Some(0.75), Some(0.75)]),
        (
            &[Some(0.1), Some(0.2), Some(0.3)],
            &[Some(0.271), Some(0.488), Some(0.657)],
        ),
        (
            &[Some(0.1), None, Some(0.2), Some(0.3)],
            &[Some(0.271), None, Some(0.488), Some(0.657)],
        ),
        // A p-value too small for 1 − p to hold it keeps its digits: n·p to first order.
        (
            &[Some(1e-20), Some(0.5), Some(0.5), Some(0.5)],
            &[Some(4e-20), Some(0.9375), Some(0.9375), Some(0.9375)],
        ),
    ];
    for (p, want) in cases {
        let got = call_adjust(p, Some("sidak")).unwrap();
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want) {
            match (g, w) {
                (None, None) => {}
                (Some(g), Some(w)) => assert!(close(*g, *w, 1e-14), "{g} against {w}"),
                other => panic!("NA moved: {other:?}"),
            }
        }
    }
}

/// Hommel's adjusted p-values by the definition: the closed test with Simes's rule. The adjusted
/// p-value of a hypothesis is the largest Simes p-value among the sets of hypotheses that hold
/// it, and the Simes p-value of a set of size m is the smallest of m·p₍ⱼ₎/j over its ordered
/// p-values. Every set is visited, so this is for short lists only.
fn hommel_by_the_closed_test(p: &[f64]) -> Vec<f64> {
    let n = p.len();
    let mut adjusted = vec![0.0f64; n];
    for set in 1u32..(1 << n) {
        let mut members: Vec<f64> = (0..n)
            .filter(|i| set & (1 << i) != 0)
            .map(|i| p[i])
            .collect();
        members.sort_by(f64::total_cmp);
        let m = members.len() as f64;
        let simes = members
            .iter()
            .enumerate()
            .map(|(j, x)| m * x / (j as f64 + 1.0))
            .fold(f64::INFINITY, f64::min);
        for (i, a) in adjusted.iter_mut().enumerate() {
            if set & (1 << i) != 0 {
                *a = a.max(simes);
            }
        }
    }
    adjusted.iter().map(|a| a.min(1.0)).collect()
}

// ----- the properties -----

/// A p-value: any number from 0 to 1, or one of a coarse grid so that ties, 0 and 1 turn up.
fn p_value() -> impl Strategy<Value = f64> {
    prop_oneof![
        0.0..=1.0f64,
        (0u32..=20).prop_map(|k| f64::from(k) / 20.0),
        (0u32..=20).prop_map(|k| f64::from(k) / 400.0),
    ]
}

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
    fn an_adjusted_p_value_is_never_below_the_raw_one_and_the_order_is_kept(
        p in prop::collection::vec(prop::option::weighted(0.85, p_value()), 1..24),
    ) {
        for word in Method::WORDS {
            let method = Method::from_word(word).unwrap();
            let adjusted = adjust(&p, method);
            prop_assert_eq!(adjusted.len(), p.len());
            for (raw, adj) in p.iter().zip(&adjusted) {
                match (raw, adj) {
                    (None, None) => {}
                    (Some(r), Some(a)) => {
                        prop_assert!(*a >= *r - 1e-15, "{method:?}: {a} below {r}");
                        prop_assert!(*a <= 1.0, "{method:?}: {a} above 1");
                    }
                    other => prop_assert!(false, "NA moved: {other:?}"),
                }
            }
            // A larger raw p-value never gets a smaller adjusted one.
            let present: Vec<(f64, f64)> = p
                .iter()
                .zip(&adjusted)
                .filter_map(|(r, a)| Some((r.as_ref().copied()?, a.as_ref().copied()?)))
                .collect();
            for (r1, a1) in &present {
                for (r2, a2) in &present {
                    if r1 < r2 {
                        prop_assert!(a1 <= a2, "{method:?}: raw {r1} < {r2} but adjusted {a1} > {a2}");
                    }
                    // Equal raw p-values get the same adjusted one, to the last bit.
                    if r1 == r2 {
                        prop_assert!(a1 == a2, "{method:?}: raw {r1} twice but adjusted {a1} and {a2}");
                    }
                }
            }
        }
        // `none` returns its input.
        prop_assert_eq!(adjust(&p, Method::Unadjusted), p.clone());
        // Bonferroni is never below Holm, which is never below Hochberg, which is never below
        // Hommel; Holm is never below Benjamini–Hochberg; BY is never below BH; Bonferroni is
        // never below Šidák.
        let (b, h, hochberg, hommel, bh, by, sidak) = (
            adjust(&p, Method::Bonferroni),
            adjust(&p, Method::Holm),
            adjust(&p, Method::Hochberg),
            adjust(&p, Method::Hommel),
            adjust(&p, Method::BenjaminiHochberg),
            adjust(&p, Method::BenjaminiYekutieli),
            adjust(&p, Method::Sidak),
        );
        for i in 0..p.len() {
            if let (Some(b), Some(h), Some(bh), Some(by)) = (b[i], h[i], bh[i], by[i]) {
                prop_assert!(b >= h - 1e-15);
                prop_assert!(h >= bh - 1e-15);
                prop_assert!(by >= bh - 1e-15);
            }
            if let (Some(b), Some(h), Some(hochberg), Some(hommel), Some(sidak)) =
                (b[i], h[i], hochberg[i], hommel[i], sidak[i])
            {
                prop_assert!(h >= hochberg - 1e-15, "Holm {h} below Hochberg {hochberg}");
                prop_assert!(hochberg >= hommel - 1e-15, "Hochberg {hochberg} below Hommel {hommel}");
                prop_assert!(b >= sidak - 1e-15, "Bonferroni {b} below Šidák {sidak}");
            }
        }
    }

    #[test]
    fn hommel_is_the_closed_test_with_the_rule_of_simes(
        p in prop::collection::vec(p_value(), 1..=8),
    ) {
        let listed: Vec<Option<f64>> = p.iter().copied().map(Some).collect();
        let got = adjust(&listed, Method::Hommel);
        let want = hommel_by_the_closed_test(&p);
        for (i, (g, w)) in got.iter().zip(&want).enumerate() {
            let g = g.unwrap();
            prop_assert!(close(g, *w, 1e-13), "item {i} of {p:?}: {g} against {w}");
        }
    }

    #[test]
    fn the_statistic_ignores_row_order_and_frequency_weights_repeat_rows(
        values in prop::collection::vec((-3.0..3.0f64, 0..4i64, 1..4i64), 4..40),
        seed in 0..1000u64,
    ) {
        let distinct_groups = values.iter().map(|(_, g, _)| g).collect::<std::collections::BTreeSet<_>>().len();
        prop_assume!(distinct_groups >= 2);
        let frame = frame_of(vec![
            ("x", Column::floats(values.iter().map(|(x, _, _)| Some(*x))).0),
            ("g", Column::ints(values.iter().map(|(_, g, _)| Some(*g)))),
            ("w", Column::ints(values.iter().map(|(_, _, w)| Some(*w)))),
        ]);
        let plain = call(&frame, vec![(1, var("x")), (2, var("g"))]).unwrap();
        prop_assert!(plain.statistic.value >= -1e-12);
        prop_assert!((0.0..=1.0).contains(&plain.p_value));
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
        let moved = call(&shuffled, vec![(1, var("x")), (2, var("g"))]).unwrap();
        prop_assert!((moved.statistic.value - plain.statistic.value).abs() <= 1e-9);
        // Frequency weights against the rows written out.
        let weighted = call(
            &frame,
            vec![(1, var("x")), (2, var("g")), (3, var("w")), (4, Value::Word("frequency".to_string()))],
        )
        .unwrap();
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
        let written = call(&expanded, vec![(1, var("x")), (2, var("g"))]).unwrap();
        prop_assert_eq!(weighted.n, written.n);
        prop_assert!((weighted.statistic.value - written.statistic.value).abs() <= 1e-9);
        prop_assert_eq!(weighted.p_value.to_bits(), written.p_value.to_bits());
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
fn two_groups_give_the_uncorrected_rank_sum_approximation_and_the_kernel_refuses_constants() {
    // With two groups H is the square of the rank-sum z without continuity correction.
    let x = vec![0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46];
    let y = vec![1.15, 0.88, 0.90, 0.74, 1.21];
    let h = kruskal(&[x.clone(), y.clone()]).unwrap();
    let (m, n) = (x.len() as f64, y.len() as f64);
    let pooled: Vec<f64> = x.iter().chain(&y).copied().collect();
    let r = zeff_numeric::association::ranks(&pooled);
    let w: f64 = r[..x.len()].iter().sum::<f64>() - m * (m + 1.0) / 2.0;
    let z = (w - m * n / 2.0) / (m * n * (m + n + 1.0) / 12.0).sqrt();
    assert!(
        (h.statistic - z * z).abs() < 1e-12,
        "{} against {}",
        h.statistic,
        z * z
    );
    assert_eq!(h.df, 1.0);
    assert!(!h.ties);
    // Every value the same: no statistic.
    assert!(kruskal(&[vec![2.0, 2.0], vec![2.0, 2.0, 2.0]]).is_none());
    // An empty list adjusts to an empty list; all NA stays all NA.
    assert_eq!(call_adjust(&[], None).unwrap(), Vec::<Option<f64>>::new());
    assert_eq!(
        call_adjust(&[None, None], Some("by")).unwrap(),
        vec![None, None]
    );
    // A single p-value is unchanged by every method, with an `NA` beside it or without.
    for method in Method::WORDS {
        assert_eq!(
            call_adjust(&[Some(0.3)], Some(method)).unwrap(),
            vec![Some(0.3)],
            "{method}"
        );
        assert_eq!(
            call_adjust(&[None, Some(0.3)], Some(method)).unwrap(),
            vec![None, Some(0.3)],
            "{method}"
        );
        assert_eq!(
            call_adjust(&[None, None], Some(method)).unwrap(),
            vec![None, None],
            "{method}"
        );
        assert_eq!(
            call_adjust(&[], Some(method)).unwrap(),
            Vec::<Option<f64>>::new(),
            "{method}"
        );
    }
}

#[test]
fn ties_and_p_values_of_zero_and_one_under_every_method() {
    // A p-value of 0 stays 0, a plain zero and not a negative one, and a p-value of 1 stays 1.
    for method in Method::WORDS {
        let got = call_adjust(&[Some(0.0), Some(1.0), Some(0.2), Some(0.0)], Some(method)).unwrap();
        for i in [0, 3] {
            let zero = got[i].unwrap();
            assert!(zero == 0.0 && zero.is_sign_positive(), "{method}: {zero:?}");
        }
        assert_eq!(got[1], Some(1.0), "{method}");
        let all_one = call_adjust(&[Some(1.0); 3], Some(method)).unwrap();
        assert_eq!(all_one, vec![Some(1.0); 3], "{method}");
        let all_zero = call_adjust(&[Some(0.0); 3], Some(method)).unwrap();
        assert_eq!(all_zero, vec![Some(0.0); 3], "{method}");
    }
    // Three p-values of 0.02. The step-up methods leave them alone, since the largest is
    // multiplied by one and the others cannot exceed it; the step-down and single-step ones
    // multiply by three; Šidák gives 1 − 0.98³.
    let tied = [Some(0.02); 3];
    for (method, want) in [
        ("holm", 0.06),
        ("bonferroni", 0.06),
        ("hochberg", 0.02),
        ("hommel", 0.02),
        ("bh", 0.02),
        ("by", 0.02 * (1.0 + 0.5 + 1.0 / 3.0)),
        ("sidak", 0.058808),
        ("none", 0.02),
    ] {
        for got in call_adjust(&tied, Some(method)).unwrap() {
            let got = got.unwrap();
            assert!(close(got, want, 1e-14), "{method}: {got} against {want}");
        }
    }
    // Two p-values: Hommel is Hochberg, as in R.
    for p in [
        [Some(0.04), Some(0.03)],
        [Some(0.01), Some(0.9)],
        [Some(0.5), Some(0.5)],
    ] {
        assert_eq!(
            call_adjust(&p, Some("hommel")).unwrap(),
            call_adjust(&p, Some("hochberg")).unwrap()
        );
    }
    // A word that is no method is refused by the function itself when the checker has not been
    // asked first.
    let other = call_adjust(&[Some(0.01), Some(0.2)], Some("tukey")).unwrap_err();
    assert_eq!(other.code, "Z0401");
}

#[test]
fn category_groups_and_labelled_groups_are_named_in_the_table() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages).add(lwage = log(wage))\nwages.kruskal_test(lwage, group: industry)\nwages.kruskal_test(hours, group: female)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stdout.contains("kruskal_test · lwage by industry"),
        "{stdout}"
    );
    assert!(
        stdout.contains("H(") && stdout.contains("p < 0.001"),
        "{stdout}"
    );
    assert!(
        stderr.contains("rows with NA in lwage or industry dropped"),
        "{stderr}"
    );
    // The two levels of female are labelled by the dataset's value labels.
    let after = stdout.split("hours by female").nth(1).unwrap();
    assert!(after.contains("  group  "), "{after}");
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.kruskal_test(s, group: site)\ng.where(site < 3).kruskal_test(s, group: site)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.kruskal_test(v, group: k)\nt.kruskal_test(v, group: k, weights: w, weight_type: frequency)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5], g = [1, 1, 2, 2, NA, 3, 3])\nm.kruskal_test(x, group: g)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.kruskal_test(lwage, group: industry)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2])\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: bonferroni)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: bh)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: by)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: hochberg)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: hommel)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: sidak)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: none)\np_adjust([])\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [1, 2, 3]).kruskal_test(x)\n",
        "frame(x = [1, 2, 3], g = [1, 1, 1]).kruskal_test(x, group: g)\n",
        "frame(x = [2, 2, 2, 2], g = [1, 1, 2, 2]).kruskal_test(x, group: g)\n",
        "frame(s = [\"a\", \"b\"], g = [1, 2]).kruskal_test(s, group: g)\n",
        "frame(x = [1, 2, 3], d = [date(\"2024-01-01\"), date(\"2024-01-02\"), date(\"2024-01-01\")]).kruskal_test(x, group: d)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2], w = [1, 1, 2, 1]).kruskal_test(x, group: g, weights: w)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2], w = [1.5, 1, 2, 1]).kruskal_test(x, group: g, weights: w, weight_type: frequency)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2], w = [1, 1, 2, 1]).kruskal_test(x, group: g, weights: w, weight_type: analytic)\n",
        "p_adjust([0.01, \"a\"])\n",
        "p_adjust([0.01, 1.2])\n",
        "p_adjust([0.01, 0.2], method: tukey)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
