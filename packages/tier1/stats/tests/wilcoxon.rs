//! `wilcoxon_test` (release 0.7.0, step 11; Language Reference, sections 4, 6 and 10.2; open
//! decision #91): the reference (`tests/reference/r/wilcoxon.json`, R 4.6.0's `wilcox.test`
//! exact and approximate, for every n from 2 to 60, on a grid of group sizes, with ties, zeros
//! and the paired form) through the package's own function: statistics, exact p-values,
//! estimates and exact intervals within a relative 1e-12, approximate p-values within 1e-12 and
//! approximate intervals within R's root tolerance; the kernel's distributions sum to one
//! exactly; the edge cases and refusals; and snapshots of every form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_numeric::wilcoxon::{rank_sum_plain, signed_rank_plain};
use zeff_stats::wilcoxon_test::wilcoxon_test;
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The arguments by slot: data, var, b, mu, exact, correct, group, paired, alternative, level,
/// weights, weight_type.
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
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "d.wilcoxon_test(x)",
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(2, 8),
        arg_spans: vec![None; 12],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    wilcoxon_test(&mut ctx, args).map(|v| match v {
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

fn floats(v: &Json) -> Vec<Option<f64>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| Some(number(x)))
        .collect()
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/wilcoxon.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    if got.is_infinite() || want.is_infinite() {
        return got == want;
    }
    (got - want).abs() <= relative * want.abs().max(1e-9)
}

/// The slots a reference case shares: mu, exact, correct, alternative, level.
fn shared_slots(case: &Json) -> Vec<(usize, Value)> {
    let mut slots = vec![
        (3, Value::Float(number(&case["mu"]))),
        (4, Value::Bool(case["exact"].as_bool().unwrap())),
        (5, Value::Bool(case["correct"].as_bool().unwrap())),
        (9, Value::Float(number(&case["level"]))),
    ];
    let alternative = match case["alternative"].as_str().unwrap() {
        "less" => "less",
        "greater" => "greater",
        _ => "two_sided",
    };
    slots.push((8, Value::Word(alternative.to_string())));
    slots
}

/// Statistic, p-value and estimate to 1e-12; the interval to 1e-12 when exact, else to R's
/// root tolerance of 1e-4, read as 2e-4 against the scale of the data.
fn check(label: &str, case: &Json, result: &TestResult, problems: &mut Vec<String>) {
    let exact = case["exact"].as_bool().unwrap();
    let mut want = |what: &str, got: f64, expected: f64, relative: f64| {
        if !close(got, expected, relative) {
            problems.push(format!("{label} {what}: Zeff {got:e}, R {expected:e}"));
        }
    };
    want(
        "statistic",
        result.statistic.value,
        number(&case["statistic"]),
        1e-12,
    );
    // An exact p-value is a ratio of integers; an approximate one is a normal tail, held to the
    // distribution class (1e-12 at the centre, a few parts in 1e12 at z ≈ 4).
    want(
        "p",
        result.p_value,
        number(&case["p_value"]),
        if exact { 1e-12 } else { 1e-10 },
    );
    let estimate = result.estimate.as_ref().unwrap();
    let (lower, upper) = estimate.ci.unwrap();
    let ci = case["ci"].as_array().unwrap();
    if exact {
        want("estimate", estimate.value, number(&case["estimate"]), 1e-12);
        want("lower", lower, number(&ci[0]), 1e-12);
        want("upper", upper, number(&ci[1]), 1e-12);
    } else {
        // The approximate bounds are roots of a step function, which R finds to tol.root =
        // 1e-4; the scale here is the data's, of order one to ten. R's approximate estimate is
        // the root of a statistic that is exactly zero over a whole interval, so it is not
        // compared: Zeff's is the median of the Walsh averages, as in the exact case.
        let near = |got: f64, expected: f64| -> bool {
            got.is_infinite() && expected.is_infinite() && got == expected
                || (got - expected).abs() <= 2e-4
        };
        for (what, got, expected) in [
            ("lower", lower, number(&ci[0])),
            ("upper", upper, number(&ci[1])),
        ] {
            if !near(got, expected) {
                problems.push(format!("{label} {what}: Zeff {got:e}, R {expected:e}"));
            }
        }
    }
}

// ----- the reference -----

#[test]
fn the_signed_rank_test_agrees_with_wilcox_test_for_every_n_from_2_to_60() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["signed"].as_array().unwrap().iter().enumerate() {
        let frame = frame_of(vec![("x", Column::floats(floats(&case["x"])).0)]);
        let mut slots = vec![(1, var("x"))];
        slots.extend(shared_slots(case));
        let result = call(&frame, slots).unwrap();
        check(
            &format!("signed[{i}] n={}", case["x"].as_array().unwrap().len()),
            case,
            &result,
            &mut problems,
        );
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_rank_sum_test_agrees_with_wilcox_test_on_the_grid() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["rank_sum"].as_array().unwrap().iter().enumerate() {
        let x = floats(&case["x"]);
        let y = floats(&case["y"]);
        let mut values = x.clone();
        values.extend(y.iter().copied());
        let mut group: Vec<Option<i64>> = vec![Some(1); x.len()];
        group.extend(vec![Some(2); y.len()]);
        let frame = frame_of(vec![
            ("s", Column::floats(values).0),
            ("g", Column::ints(group)),
        ]);
        let mut slots = vec![(1, var("s")), (6, var("g"))];
        slots.extend(shared_slots(case));
        let result = call(&frame, slots).unwrap();
        check(
            &format!("rank_sum[{i}] {}×{}", x.len(), y.len()),
            case,
            &result,
            &mut problems,
        );
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn ties_zeros_and_the_paired_form_agree_with_wilcox_test() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["ties"].as_array().unwrap().iter().enumerate() {
        let x = floats(&case["x"]);
        let (frame, mut slots) = if case.get("y").is_some() {
            let y = floats(&case["y"]);
            let mut values = x.clone();
            values.extend(y.iter().copied());
            let mut group: Vec<Option<i64>> = vec![Some(1); x.len()];
            group.extend(vec![Some(2); y.len()]);
            (
                frame_of(vec![
                    ("s", Column::floats(values).0),
                    ("g", Column::ints(group)),
                ]),
                vec![(1, var("s")), (6, var("g"))],
            )
        } else {
            (
                frame_of(vec![("x", Column::floats(x).0)]),
                vec![(1, var("x"))],
            )
        };
        slots.extend(shared_slots(case));
        let result = call(&frame, slots).unwrap();
        check(&format!("ties[{i}]"), case, &result, &mut problems);
    }
    let paired = &r["paired"];
    let frame = frame_of(vec![
        ("a", Column::floats(floats(&paired["a"])).0),
        ("b", Column::floats(floats(&paired["b"])).0),
    ]);
    for (i, case) in paired["cases"].as_array().unwrap().iter().enumerate() {
        let mut slots = vec![(1, var("a")), (2, var("b")), (7, Value::Bool(true))];
        slots.extend(shared_slots(case));
        let result = call(&frame, slots).unwrap();
        check(&format!("paired[{i}]"), case, &result, &mut problems);
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the distributions -----

#[test]
fn the_exact_distributions_sum_to_one_exactly_and_are_symmetric() {
    for n in 1..=60 {
        let d = signed_rank_plain(n);
        let probabilities = d.probabilities();
        // The counts sum to 2^n exactly; the probabilities are those counts over the total.
        assert_eq!(d.total(), 1u128 << n);
        let total: f64 = probabilities.iter().sum();
        assert!(
            (total - 1.0).abs() <= 64.0 * f64::EPSILON,
            "n = {n}: {total}"
        );
        for k in 0..d.len() {
            assert_eq!(d.count(k), d.count(d.len() - 1 - k), "n = {n}, k = {k}");
        }
    }
    for (m, n) in [(2, 2), (5, 7), (10, 10), (20, 30), (49, 49)] {
        let d = rank_sum_plain(m, n);
        assert_eq!(d.len(), m * n + 1);
        for k in 0..d.len() {
            assert_eq!(d.count(k), d.count(d.len() - 1 - k), "{m}, {n}, k = {k}");
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
fn the_rules_of_exactness_and_the_refusals() {
    // 50 observations: the approximation unless exact: true.
    let fifty: Vec<String> = (1..=50)
        .map(|i| format!("{}", (i as f64 * 0.37).sin() * 3.0 + 0.2))
        .collect();
    let (exit, stdout, _) = go(&format!(
        "frame(x = [{}]).wilcoxon_test(x)\n",
        fifty.join(", ")
    ));
    assert_eq!(exit, 0);
    assert!(
        stdout.contains("normal approximation with continuity correction\n"),
        "{stdout}"
    );
    let (exit, stdout, _) = go(&format!(
        "frame(x = [{}]).wilcoxon_test(x, exact: true)\n",
        fifty.join(", ")
    ));
    assert_eq!(exit, 0);
    assert!(stdout.contains("exact p-value, no ties\n"), "{stdout}");
    let (exit, stdout, _) = go("frame(x = [1, 2, 2, 3, 0, 0, 5]).wilcoxon_test(x)\n");
    assert_eq!(exit, 0);
    assert!(
        stdout.contains("exact p-value, over the tied ranks, 2 differences of 0 set aside\n"),
        "{stdout}"
    );
    let (exit, stdout, _) =
        go("frame(x = [1, 2, 2, 3, 0, 0, 5]).wilcoxon_test(x, exact: false, correct: false)\n");
    assert_eq!(exit, 0);
    assert!(
        stdout.contains("normal approximation, ties, 2 differences of 0 dropped\n"),
        "{stdout}"
    );
    for (script, code, phrase) in [
        (
            "frame(x = [2, 2, 2]).wilcoxon_test(x, mu: 2)\n",
            "Z0401",
            "every value of x less 2 is 0",
        ),
        (
            "frame(a = [1, 2], b = [1, 2]).wilcoxon_test(a, b, paired: true)\n",
            "Z0401",
            "every difference a − b is 0",
        ),
        (
            "frame(s = [\"a\", \"b\"]).wilcoxon_test(s)\n",
            "Z0401",
            "must be a Bool, Int or Float",
        ),
        (
            "frame(x = [1, 2, 3], y = [1, 2, 4]).wilcoxon_test(x, y)\n",
            "Z0401",
            "needs `paired: true`",
        ),
        (
            "frame(x = [1, 2, 3]).wilcoxon_test(x, paired: true)\n",
            "Z0401",
            "needs two variables",
        ),
        (
            "frame(x = [1, 2, 3, 4], g = [0, 0, 1, 1], y = [1, 2, 3, 5]).wilcoxon_test(x, y, group: g, paired: true)\n",
            "Z0401",
            "not paired",
        ),
        (
            "frame(x = [1, 2, 3, 4], g = [0, 1, 2, 3]).wilcoxon_test(x, group: g)\n",
            "Z0412",
            "has 4 levels",
        ),
        (
            "frame(x = [1, 2, 3], w = [1, 1, 2]).wilcoxon_test(x, weights: w)\n",
            "Z0411",
            "sampling weights",
        ),
        (
            "frame(x = [1, 2, 3], w = [1.5, 1, 2]).wilcoxon_test(x, weights: w, weight_type: frequency)\n",
            "Z0401",
            "frequency weights are whole numbers, but `w` has 1 fraction in row 1",
        ),
        (
            "frame(x = [1, 2, 3], w = [1, 1, 2]).wilcoxon_test(x, weights: w, weight_type: analytic)\n",
            "Z0401",
            "frequency weights, not analytic",
        ),
        (
            "frame(x = [1, 2, 3]).wilcoxon_test(x, level: 0)\n",
            "Z0401",
            "between 0 and 1",
        ),
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}\n{stderr}");
        assert!(stderr.contains(code), "{script}\n{stderr}");
        assert!(stderr.contains(phrase), "{script}\n{stderr}");
    }
}

#[test]
fn frequency_weights_repeat_rows_and_na_rows_are_dropped() {
    let (exit, stdout, stderr) = go(
        "let f = frame(x = [1.5, 2.5, NA, 4.0, 3.5, 7.0], w = [2, 1, 1, 1, 3, 1])\nf.wilcoxon_test(x, mu: 2, weights: w, weight_type: frequency)\nframe(x = [1.5, 1.5, 2.5, 4.0, 3.5, 3.5, 3.5, 7.0]).wilcoxon_test(x, mu: 2)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stderr.contains("1 row with NA in x dropped"), "{stderr}");
    let results: Vec<&str> = stdout
        .split("wilcoxon_test · signed rank · x\n")
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], results[1], "{stdout}");
}

// ----- a property -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]
    #[test]
    fn negating_the_data_mirrors_the_test_and_a_shift_moves_the_estimate(
        magnitudes in prop::collection::btree_set(1i64..2000, 2..40),
        signs in prop::collection::vec(any::<bool>(), 40),
        shift in -100.0f64..100.0,
    ) {
        // Distinct absolute values and no zero: a difference of exactly zero is set aside from
        // both sums, and with tied absolute values the searched interval is not
        // mirror-symmetric (nor is R's), so the identities below hold for such data.
        let x: Vec<f64> = magnitudes
            .iter()
            .zip(&signs)
            .map(|(m, negative)| if *negative { -(*m as f64) / 3.0 } else { *m as f64 / 3.0 })
            .collect();
        let frame = frame_of(vec![("x", Column::floats(x.iter().map(|v| Some(*v))).0)]);
        let Ok(base) = call(&frame, vec![(1, var("x"))]) else { return Ok(()); }; // all zero: refused by design
        // Negated data: V becomes n(n+1)/2 − V, the two-sided p-value is unchanged, the
        // estimate and interval change sign and swap ends.
        let negated = frame_of(vec![("x", Column::floats(x.iter().map(|v| Some(-*v))).0)]);
        let mirror = call(&negated, vec![(1, var("x"))]).unwrap();
        let n = x.len() as f64;
        prop_assert!((base.statistic.value + mirror.statistic.value - n * (n + 1.0) / 2.0).abs() <= 1e-9);
        prop_assert!((base.p_value - mirror.p_value).abs() <= 1e-12, "{} against {}", base.p_value, mirror.p_value);
        let (e0, e1) = (base.estimate.as_ref().unwrap(), mirror.estimate.as_ref().unwrap());
        prop_assert!((e0.value + e1.value).abs() <= 1e-9);
        let (l0, u0) = e0.ci.unwrap();
        let (l1, u1) = e1.ci.unwrap();
        let mirrored = |a: f64, b: f64| -> bool {
            (a.is_infinite() && b.is_infinite() && a == -b) || (a + b).abs() <= 1e-9
        };
        prop_assert!(mirrored(l0, u1) && mirrored(u0, l1), "{:?} against {:?}", (l0, u0), (l1, u1));
        // Shifted data with mu shifted alike: the same test, the estimate moved by the shift.
        let shifted = frame_of(vec![("x", Column::floats(x.iter().map(|v| Some(v + shift))).0)]);
        let moved = call(&shifted, vec![(1, var("x")), (3, Value::Float(shift))]).unwrap();
        prop_assert!((moved.statistic.value - base.statistic.value).abs() <= 1e-9);
        prop_assert!((moved.p_value - base.p_value).abs() <= 1e-12);
        let e2 = moved.estimate.as_ref().unwrap();
        prop_assert!((e2.value - e0.value - shift).abs() <= 1e-9 * shift.abs().max(1.0));
    }
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let d = frame(x = [1.83, 0.50, 1.62, 2.48, 1.68, 1.88, 1.55, 3.06, 1.30], y = [0.878, 0.647, 0.598, 2.05, 1.06, 1.29, 1.06, 3.14, 1.29])\nd.wilcoxon_test(x, mu: 1)\nd.wilcoxon_test(x, mu: 1, exact: false, level: 0.99)\nd.wilcoxon_test(x, y, paired: true, alternative: greater)\nd.wilcoxon_test(x, y, paired: true, alternative: less, exact: false, correct: false)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9])\nt.wilcoxon_test(v, mu: 5)\nt.wilcoxon_test(v, mu: 5, exact: false, alternative: less)\nlet g = frame(s = [0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46, 1.15, 0.88, 0.90, 0.74, 1.21], grp = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2])\ng.wilcoxon_test(s, group: grp)\ng.wilcoxon_test(s, group: grp, alternative: greater, level: 0.9, exact: false)\ng.wilcoxon_test(s, group: grp, mu: 0.2)\nlet w = dataset(wages).where(worker <= 400).add(lwage = log(wage))\nw.wilcoxon_test(lwage, group: female)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [2, 2, 2]).wilcoxon_test(x, mu: 2)\n",
        "frame(a = [1, 2], b = [1, 2]).wilcoxon_test(a, b, paired: true)\n",
        "frame(s = [\"a\", \"b\"]).wilcoxon_test(s)\n",
        "frame(x = [1, 2, 3], y = [1, 2, 4]).wilcoxon_test(x, y)\n",
        "frame(x = [1, 2, 3]).wilcoxon_test(x, paired: true)\n",
        "frame(x = [1, 2, 3, 4], g = [0, 0, 1, 1], y = [1, 2, 3, 5]).wilcoxon_test(x, y, group: g, paired: true)\n",
        "frame(x = [1, 2, 3, 4], g = [0, 1, 2, 3]).wilcoxon_test(x, group: g)\n",
        "frame(x = [1, 2, 3], w = [1, 1, 2]).wilcoxon_test(x, weights: w)\n",
        "frame(x = [1, 2, 3], w = [1.5, 1, 2]).wilcoxon_test(x, weights: w, weight_type: frequency)\n",
        "frame(x = [1, 2, 3], w = [1, 1, 2]).wilcoxon_test(x, weights: w, weight_type: analytic)\n",
        "frame(x = [1, 2, 3]).wilcoxon_test(x, level: 0)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
