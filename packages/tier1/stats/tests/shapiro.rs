//! `shapiro_test` (release 0.7.0, step 14; Language Reference, sections 4, 6 and 10.2; open
//! decision #91): the reference (`tests/reference/r/shapiro.json`, R 4.6.0's `shapiro.test` at
//! n = 3, 4, 5, 11, 12, 50, 2,000 and 5,000 on normal, skewed, tied and smooth data, and with
//! `NA`) through the package's own function within a relative 1e-9; the kernel's bounds; the
//! properties (W in (0, 1], p in [0, 1], the same W and p after a change of location and scale
//! and under any order of the rows); the refusals; and snapshots of every form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::shapiro_test::{LARGEST, shapiro, shapiro_test};
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

/// `d.shapiro_test(x)` on a frame with the one column `x`.
fn call(column: Column) -> Result<TestResult, Fail> {
    let table = Columns::new(vec![("x".to_string(), column)]).unwrap();
    let frame = Frame::new(table);
    let values = vec![Some(Value::Frame(frame)), Some(Value::Var("x".to_string()))];
    with_context(values, "d.shapiro_test(x)", shapiro_test, |v| match v {
        Value::TestResult(r) => (*r).clone(),
        other => panic!("not a TestResult: {other:?}"),
    })
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
    let text = std::fs::read_to_string(root().join("tests/reference/r/shapiro.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(1e-300)
}

// ----- the reference -----

#[test]
fn w_and_the_p_value_agree_with_shapiro_test_at_every_size() {
    let r = reference();
    let mut problems = Vec::new();
    let mut sizes = std::collections::BTreeSet::new();
    for (i, case) in r["shapiro"].as_array().unwrap().iter().enumerate() {
        let x = floats(&case["x"]);
        let missing = x.iter().filter(|v| v.is_none()).count();
        let result = call(Column::floats(x).0).unwrap();
        let label = format!("shapiro[{i}] ({}, n = {})", case["shape"], case["n"]);
        let w = number(&case["w"]).unwrap();
        if !close(result.statistic.value, w, 1e-9) {
            problems.push(format!(
                "{label} W: Zeff {:.15e}, R {w:.15e}",
                result.statistic.value
            ));
        }
        let p = number(&case["p_value"]).unwrap();
        if !close(result.p_value, p, 1e-9) {
            problems.push(format!(
                "{label} p: Zeff {:.15e}, R {p:.15e}",
                result.p_value
            ));
        }
        let n = case["n"].as_u64().unwrap() as usize;
        if result.n != n || result.dropped != missing {
            problems.push(format!(
                "{label} n: Zeff {} with {} dropped, R {n} with {missing} NA",
                result.n, result.dropped
            ));
        }
        assert!(result.statistic.df.is_empty());
        assert_eq!(result.statistic.name, "W");
        sizes.insert(n);
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    for n in [3, 4, 5, 11, 12, 50, 2000, 5000] {
        assert!(sizes.contains(&n), "no case of size {n}");
    }
}

// ----- the kernel -----

#[test]
fn the_kernel_answers_only_for_three_to_five_thousand_distinct_finite_values() {
    assert!(shapiro(&[]).is_none());
    assert!(shapiro(&[1.0, 2.0]).is_none());
    assert!(shapiro(&[2.0, 2.0, 2.0]).is_none());
    assert!(shapiro(&[1.0, 2.0, f64::INFINITY]).is_none());
    assert!(shapiro(&[1.0, 2.0, f64::NAN]).is_none());
    let too_many: Vec<f64> = (0..=LARGEST).map(|i| i as f64).collect();
    assert!(shapiro(&too_many).is_none());
    assert!(shapiro(&too_many[..LARGEST]).is_some());
    // Three equally spaced values are a perfect normal sample of three: W = 1 and p = 1.
    let three = shapiro(&[1.0, 2.0, 3.0]).unwrap();
    assert!((three.w - 1.0).abs() < 1e-12, "{}", three.w);
    assert!((three.p - 1.0).abs() < 1e-12, "{}", three.p);
    // Three values with two the same give the smallest W of three, 3/4, and p = 0.
    let tied = shapiro(&[1.0, 1.0, 2.0]).unwrap();
    assert!((tied.w - 0.75).abs() < 1e-12, "{}", tied.w);
    assert_eq!(tied.p, 0.0);
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
    fn w_lies_in_the_unit_interval_and_ignores_location_scale_and_order(
        values in prop::collection::vec(-5.0..5.0f64, 3..60),
        shift in -100.0..100.0f64,
        scale in 0.01..100.0f64,
        seed in 0..1000u64,
    ) {
        let mut sorted = values.clone();
        sorted.sort_by(f64::total_cmp);
        prop_assume!(sorted[sorted.len() - 1] > sorted[0]);
        let plain = shapiro(&sorted).unwrap();
        prop_assert!(plain.w > 0.0 && plain.w <= 1.0 + 1e-12, "W = {}", plain.w);
        prop_assert!((0.0..=1.0).contains(&plain.p), "p = {}", plain.p);
        // A change of location and scale.
        let moved: Vec<f64> = sorted.iter().map(|x| x * scale + shift).collect();
        let changed = shapiro(&moved).unwrap();
        prop_assert!((changed.w - plain.w).abs() <= 1e-9, "W {} against {}", changed.w, plain.w);
        prop_assert!((changed.p - plain.p).abs() <= 1e-9 * plain.p.max(1e-6), "p {} against {}", changed.p, plain.p);
        // The function sorts for itself, so the order of the rows does not matter.
        let mut order: Vec<usize> = (0..values.len()).collect();
        let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        for i in (1..order.len()).rev() {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            let j = (state >> 33) as usize % (i + 1);
            order.swap(i, j);
        }
        let shuffled = call(Column::floats(order.iter().map(|&i| Some(values[i]))).0).unwrap();
        prop_assert_eq!(shuffled.statistic.value.to_bits(), plain.w.to_bits());
        prop_assert_eq!(shuffled.p_value.to_bits(), plain.p.to_bits());
        prop_assert_eq!(shuffled.n, values.len());
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
fn integers_and_booleans_are_tested_and_an_infinite_value_is_refused() {
    let ints = call(Column::ints([3, 1, 4, 1, 5, 9, 2, 6].into_iter().map(Some))).unwrap();
    let floats = call(
        Column::floats(
            [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0]
                .into_iter()
                .map(Some),
        )
        .0,
    )
    .unwrap();
    assert_eq!(
        ints.statistic.value.to_bits(),
        floats.statistic.value.to_bits()
    );
    assert_eq!(ints.p_value.to_bits(), floats.p_value.to_bits());
    let infinite =
        call(Column::floats([1.0, 2.0, f64::INFINITY, 4.0].into_iter().map(Some)).0).unwrap_err();
    assert_eq!(infinite.code, "Z0401");
    assert!(
        infinite.message.contains("infinite"),
        "{}",
        infinite.message
    );
}

#[test]
fn the_wages_subset_prints_and_the_whole_dataset_is_refused() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.shapiro_test(lwage)\nwages.shapiro_test(lwage).df()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("shapiro_test · lwage"), "{stdout}");
    assert!(stdout.contains("W = 0."), "{stdout}");
    assert!(stdout.ends_with("NA\n"), "{stdout}");
    assert!(stderr.contains("rows with NA in lwage dropped"), "{stderr}");
    let (exit, _, stderr) = go("dataset(wages).shapiro_test(educ)\n");
    assert_eq!(exit, 1);
    assert!(
        stderr.contains("error[Z0421]") && stderr.contains("18,412"),
        "{stderr}"
    );
    assert!(
        stderr.contains("`qq_plot`") && stderr.contains("`ks_test`"),
        "{stderr}"
    );
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0])\ng.shapiro_test(s)\nframe(x = [1.2, 3.4, 2.2]).shapiro_test(x)\nframe(x = [1, 1, 2]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 10]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 40]).shapiro_test(x)\nframe(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 2.0, 3.0, 9.0, 1.0, 2.5]).shapiro_test(x)\nframe(b = [true, false, true, true, false, false, true, false]).shapiro_test(b)\nlet wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.shapiro_test(lwage)\nwages.shapiro_test(educ).statistic()\nwages.shapiro_test(educ).p_values()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [1, 2, 3]).shapiro_test(y)\n",
        "frame(x = [1, 2, 3]).shapiro_test()\n",
        "frame(s = [\"a\", \"b\", \"c\"]).shapiro_test(s)\n",
        "frame(x = [1, 2]).shapiro_test(x)\n",
        "frame(x = [1, NA, NA]).shapiro_test(x)\n",
        "frame(x = [2, 2, 2, 2]).shapiro_test(x)\n",
        "frame(x = seq(1, 5001)).shapiro_test(x)\n",
        "dataset(wages).shapiro_test(educ)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
