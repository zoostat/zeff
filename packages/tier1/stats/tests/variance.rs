//! `variance_test` (release 0.8.0, step 8j; Language Reference, sections 4, 6 and 10.2; open
//! decisions #91 and #97): the reference (`tests/reference/r/variance.json`, R 4.6.0's
//! `var.test` and `bartlett.test` and `leveneTest` of car 3.1-5 centred on the median and on the
//! mean, on a series split into 2 to 6 groups, with ties, a group of one value, `NA` and text
//! groups, and on the two shipped datasets) within a relative 1e-10 for the statistics and 1e-9
//! for the p-values; the property that moving the variable or scaling it changes no p-value
//! and no statistic; the edge cases; the refusals; and snapshots of every form and every
//! message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::variance_test::{Center, bartlett, levene, ratio, variance_test};
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

/// The arguments by slot: data, var, group, method, center.
fn call(frame: &Frame, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; 5];
    values[0] = Some(Value::Frame(frame.clone()));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    with_context(
        values,
        "d.variance_test(x, group: g)",
        variance_test,
        |v| match v {
            Value::TestResult(r) => (*r).clone(),
            other => panic!("not a TestResult: {other:?}"),
        },
    )
}

/// `p_adjust(p, method:)` on a list of optional numbers.
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
    let text = std::fs::read_to_string(root().join("tests/reference/r/variance.json")).unwrap();
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

fn word(text: &str) -> Value {
    Value::Word(text.to_string())
}

/// One test of a case against R's: the statistic, the degrees of freedom and the p-value.
fn compare(label: &str, got: &TestResult, want: &Json, df: &[&str], problems: &mut Vec<String>) {
    let statistic = number(&want["statistic"]).unwrap();
    if !close(got.statistic.value, statistic, 1e-10) {
        problems.push(format!(
            "{label} statistic: Zeff {:e}, R {statistic:e}",
            got.statistic.value
        ));
    }
    let p = number(&want["p_value"]).unwrap();
    if !close(got.p_value, p, 1e-9) {
        problems.push(format!("{label} p: Zeff {:e}, R {p:e}", got.p_value));
    }
    let wanted: Vec<f64> = df.iter().map(|k| want[*k].as_f64().unwrap()).collect();
    if got.statistic.df != wanted {
        problems.push(format!(
            "{label} df: Zeff {:?}, R {wanted:?}",
            got.statistic.df
        ));
    }
}

#[test]
fn the_three_tests_agree_with_r_on_every_case() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["cases"].as_array().unwrap().iter().enumerate() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
        let label = format!("case[{i}]");
        let median = call(&frame, vec![(1, var("x")), (2, var("g"))]).unwrap();
        compare(
            &format!("{label} levene, median"),
            &median,
            &case["levene_median"],
            &["df1", "df2"],
            &mut problems,
        );
        let n = case["n"].as_u64().unwrap() as usize;
        if median.n != n {
            problems.push(format!("{label} n: Zeff {}, R {n}", median.n));
        }
        // The table: the groups in R's order, each with its size and its variance.
        let table = median.table.as_ref().unwrap();
        let sizes: Vec<String> = case["sizes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_u64().unwrap().to_string())
            .collect();
        let shown: Vec<String> = table.rows.iter().map(|row| row[1].clone()).collect();
        if shown != sizes {
            problems.push(format!("{label} sizes: Zeff {shown:?}, R {sizes:?}"));
        }
        for (row, v) in table.rows.iter().zip(case["variances"].as_array().unwrap()) {
            let want = number(v).map_or("NA".to_string(), |v| format!("{v:.3}"));
            if row[3] != want {
                problems.push(format!("{label} variance: Zeff {}, R {want}", row[3]));
            }
        }
        let mean = call(
            &frame,
            vec![(1, var("x")), (2, var("g")), (4, word("mean"))],
        )
        .unwrap();
        compare(
            &format!("{label} levene, mean"),
            &mean,
            &case["levene_mean"],
            &["df1", "df2"],
            &mut problems,
        );
        let bartlett = call(
            &frame,
            vec![(1, var("x")), (2, var("g")), (3, word("bartlett"))],
        );
        match (bartlett, case.get("bartlett")) {
            (Ok(got), Some(want)) => compare(
                &format!("{label} bartlett"),
                &got,
                want,
                &["df"],
                &mut problems,
            ),
            // R refuses a group of one value, and so does the engine.
            (Err(_), None) => {}
            (Ok(_), None) => {
                problems.push(format!("{label}: Bartlett's test given where R has none"))
            }
            (Err(e), Some(_)) => problems.push(format!("{label} bartlett refused: {e:?}")),
        }
        let groups = case["groups"].as_u64().unwrap();
        let f = call(&frame, vec![(1, var("x")), (2, var("g")), (3, word("f"))]);
        match (f, case.get("f")) {
            (Ok(got), Some(want)) => {
                compare(
                    &format!("{label} f"),
                    &got,
                    want,
                    &["df1", "df2"],
                    &mut problems,
                );
                let estimate = got.estimate.as_ref().unwrap();
                let (low, high) = estimate.ci.unwrap();
                for (name, got, key) in [
                    ("ratio", estimate.value, "ratio"),
                    ("lower", low, "ci_low"),
                    ("upper", high, "ci_high"),
                ] {
                    let want = number(&want[key]).unwrap();
                    if !close(got, want, 1e-9) {
                        problems.push(format!("{label} f {name}: Zeff {got:e}, R {want:e}"));
                    }
                }
            }
            (Err(_), None) => {}
            (Ok(_), None) => problems.push(format!(
                "{label}: a ratio given for {groups} groups, or for a group of one"
            )),
            (Err(e), Some(_)) => problems.push(format!("{label} f refused: {e:?}")),
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// On the two shipped datasets, through the language: each statistic and p-value within the
/// tolerance of R's, printed as `true`, and the degrees of freedom as they print.
#[test]
fn the_three_tests_agree_with_r_on_the_two_datasets() {
    let r = reference();
    let wages = "dataset(wages).where(year == 2019)";
    for (key, frame, var, group) in [
        ("wage_by_union", wages, "wage", "union"),
        ("wage_by_female", wages, "wage", "female"),
        ("wage_by_industry", wages, "wage", "industry"),
        ("size_by_industry", "dataset(firms)", "size", "industry"),
    ] {
        let case = &r["datasets"][key];
        let mut script = format!("let d = {frame}\n");
        let mut expected = String::new();
        let mut check = |options: &str, want: &Json, df: &[&str]| {
            script.push_str(&format!(
                "let t = d.variance_test({var}, group: {group}{options})\nprint(abs(t.statistic() / {} - 1) < 1e-10)\nprint(abs(t.p_values() / {} - 1) < 1e-9)\nprint(t.df())\nprint(t.n_obs())\n",
                want["statistic"].as_str().unwrap(),
                want["p_value"].as_str().unwrap(),
            ));
            expected.push_str("true\ntrue\n");
            for k in df {
                expected.push_str(&format!("{}\n", want[*k]));
            }
            expected.push_str(&format!("{}\n", case["n"]));
        };
        check("", &case["levene_median"], &["df1", "df2"]);
        check(", center: mean", &case["levene_mean"], &["df1", "df2"]);
        check(", method: bartlett", &case["bartlett"], &["df"]);
        if let Some(f) = case.get("f") {
            check(", method: f", f, &["df1", "df2"]);
        }
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{key}\n{stderr}");
        assert_eq!(stdout, expected, "{key}\n{script}\n{stderr}");
    }
}

// ----- the properties -----

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        * 64
}

fn same(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-7 * a.abs().max(b.abs()).max(1e-12)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// A variance does not depend on where the values lie, and equal variances stay equal in
    /// any unit: moving every value by one amount and scaling every value by one positive
    /// factor change no statistic and no p-value of the three tests, and scale the interval of
    /// no ratio. The order of the rows within a group changes nothing either.
    #[test]
    fn moving_or_scaling_the_variable_changes_no_test(
        groups in prop::collection::vec(prop::collection::vec(-50i32..50, 3..9), 2..5),
        shift in -1000i32..1000,
        scale in 1u32..400,
    ) {
        // Quarters, so that the scaled values are exact; a group needs a spread.
        let samples: Vec<Vec<f64>> = groups
            .iter()
            .map(|g| g.iter().map(|v| f64::from(*v) / 4.0).collect())
            .collect();
        prop_assume!(samples.iter().all(|s| s.iter().any(|v| *v != s[0])));
        let factor = f64::from(scale) / 8.0;
        let moved: Vec<Vec<f64>> = samples
            .iter()
            .map(|s| s.iter().rev().map(|v| (v + f64::from(shift)) * factor).collect())
            .collect();
        for center in [Center::Median, Center::Mean] {
            match (levene(&samples, center), levene(&moved, center)) {
                (Some(a), Some(b)) => {
                    prop_assert!(same(a.statistic, b.statistic), "levene {:?} {} {}", center, a.statistic, b.statistic);
                    prop_assert!(same(a.p, b.p), "levene p {} {}", a.p, b.p);
                    prop_assert_eq!(a.df, b.df);
                }
                (None, None) => {}
                (a, b) => prop_assert!(false, "levene given once: {:?} {:?}", a, b),
            }
        }
        let (a, b) = (bartlett(&samples).unwrap(), bartlett(&moved).unwrap());
        // Bartlett's statistic is a difference of sums of logarithms, each of size N ln s²: when
        // the groups' variances are nearly equal it is the rounding left of that difference,
        // and no unit keeps its seventh digit. Two groups with variances 65.967 and 65.981 give
        // 6.5258966e-8 in one unit and 6.5258959e-8 in another, here and in R's
        // `bartlett.test` to the last digit (the seed kept in `variance.proptest-regressions`,
        // met by CI on 19 November 2026). So the statistic is held to seven digits or to 1e-12
        // of a chi-square, whichever is the larger; the clause for a statistic below 1e-9,
        // which this replaces, drew the line too low.
        prop_assert!(
            same(a.statistic, b.statistic) || (a.statistic - b.statistic).abs() <= 1e-12,
            "bartlett {} {}", a.statistic, b.statistic
        );
        prop_assert!(same(a.p, b.p) || (a.p > 1.0 - 1e-6 && b.p > 1.0 - 1e-6), "bartlett p {} {}", a.p, b.p);
        let (a, b) = (
            ratio(&samples[0], &samples[1], 0.95).unwrap(),
            ratio(&moved[0], &moved[1], 0.95).unwrap(),
        );
        prop_assert!(same(a.ratio, b.ratio), "ratio {} {}", a.ratio, b.ratio);
        prop_assert!(same(a.tested.p, b.tested.p), "ratio p {} {}", a.tested.p, b.tested.p);
        prop_assert!(same(a.ci.0, b.ci.0) && same(a.ci.1, b.ci.1));
        // The ratio the other way round is the reciprocal, with the same p-value.
        let turned = ratio(&samples[1], &samples[0], 0.95).unwrap();
        prop_assert!(same(turned.ratio * a.ratio, 1.0));
        prop_assert!(same(turned.tested.p, a.tested.p));
        prop_assert!(same(turned.ci.0 * a.ci.1, 1.0) && same(turned.ci.1 * a.ci.0, 1.0));
        prop_assert!(a.ci.0 <= a.ratio && a.ratio <= a.ci.1);
    }
}

// ----- the edge cases -----

#[test]
fn groups_of_one_value_and_groups_with_no_spread() {
    // A group of one value: Levene's test stands; a group with no spread too.
    let one = frame_of(vec![
        (
            "x",
            Column::floats([1.0, 4.0, 2.0, 8.0, 3.0, 9.0, 5.0].map(Some)).0,
        ),
        ("g", Column::ints([1, 1, 1, 2, 2, 2, 3].map(Some))),
    ]);
    let t = call(&one, vec![(1, var("x")), (2, var("g"))]).unwrap();
    assert_eq!(t.statistic.df, vec![2.0, 4.0]);
    assert_eq!(t.table.as_ref().unwrap().rows[2][2], "NA");
    assert!(
        call(
            &one,
            vec![(1, var("x")), (2, var("g")), (3, word("bartlett"))]
        )
        .is_err()
    );
    let flat = frame_of(vec![
        (
            "x",
            Column::floats([2.0, 2.0, 2.0, 1.0, 5.0, 9.0].map(Some)).0,
        ),
        ("g", Column::ints([1, 1, 1, 2, 2, 2].map(Some))),
    ]);
    let t = call(&flat, vec![(1, var("x")), (2, var("g"))]).unwrap();
    assert!(t.statistic.value.is_finite() && t.p_value > 0.0 && t.p_value < 1.0);
    assert_eq!(t.table.as_ref().unwrap().rows[0][3], "0.000");
    for method in ["bartlett", "f"] {
        assert!(call(&flat, vec![(1, var("x")), (2, var("g")), (3, word(method))]).is_err());
    }
    // Equal variances exactly: the ratio is 1 and its p-value 1; Bartlett's statistic is 0.
    let equal = frame_of(vec![
        (
            "x",
            Column::floats([1.0, 2.0, 3.0, 11.0, 12.0, 13.0].map(Some)).0,
        ),
        ("g", Column::ints([1, 1, 1, 2, 2, 2].map(Some))),
    ]);
    let f = call(&equal, vec![(1, var("x")), (2, var("g")), (3, word("f"))]).unwrap();
    assert_eq!(f.statistic.value, 1.0);
    assert!((f.p_value - 1.0).abs() < 1e-12);
    let b = call(
        &equal,
        vec![(1, var("x")), (2, var("g")), (3, word("bartlett"))],
    )
    .unwrap();
    assert!(b.statistic.value.abs() < 1e-12 && (b.p_value - 1.0).abs() < 1e-6);
    // The kernel functions refuse what the tests cannot take.
    assert!(levene(&[vec![1.0], vec![2.0]], Center::Median).is_none());
    assert!(levene(&[vec![1.0, 1.0], vec![2.0, 2.0]], Center::Mean).is_none());
    assert!(levene(&[vec![1.0, 3.0], vec![2.0, 8.0]], Center::Median).is_none());
    assert!(bartlett(&[vec![1.0, 2.0], vec![3.0]]).is_none());
    assert!(ratio(&[1.0, 1.0], &[2.0, 3.0], 0.95).is_none());
}

/// A group of exactly two values is enough for each test: R's `bartlett.test` gives K² =
/// 1.08436438663089 with p = 0.581477969538514 on two degrees of freedom, and car's
/// `leveneTest` about the median F = 0.706013363028953 with p = 0.53044800612945. (Three
/// surviving mutants of release 0.8.0, step 24a2c, refused a group of two, or took one of
/// fewer; every group in the tests had three values or more, or one.)
#[test]
fn a_group_of_exactly_two_values_is_enough() {
    let (exit, stdout, stderr) = go(
        "let d = frame(x = [1, 3, 2, 5, 9, 4, 4.5, 7, 8], g = [1, 1, 2, 2, 2, 3, 3, 3, 3])\nlet b = d.variance_test(x, group: g, method: bartlett)\nprint(abs(b.statistic() - 1.08436438663089) < 1e-12)\nprint(abs(b.p_values() - 0.581477969538514) < 1e-12)\nprint(b.df())\nlet l = d.variance_test(x, group: g)\nprint(abs(l.statistic() - 0.706013363028953) < 1e-12)\nprint(abs(l.p_values() - 0.53044800612945) < 1e-12)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "true\ntrue\n2\ntrue\ntrue\n", "{stdout}{stderr}");
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.variance_test(s, group: site)\ng.variance_test(s, group: site, center: mean)\ng.variance_test(s, group: site, method: levene, center: median)\ng.variance_test(s, group: site, method: bartlett)\nlet two = g.where(site < 3)\ntwo.variance_test(s, group: site)\nlet f = two.variance_test(s, group: site, method: f)\nf\nf.statistic()\nf.df()\nf.p_values()\nf.ci()\nf.n_obs()\nf.h0()\nf.h1()\nf.alternative()\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"])\nt.variance_test(v, group: k)\nt.variance_test(v, group: k, method: bartlett)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 9.5, 3.0, 8.0], g = [1, 1, 2, 2, NA, 3, 3, 2, 1, 3])\nm.variance_test(x, group: g)\nframe(x = [1.0, 4, 2, 8, 3, 9, 5], g = [1, 1, 1, 2, 2, 2, 3]).variance_test(x, group: g)\nframe(x = [2.0, 2, 2, 1, 5, 9], g = [true, true, true, false, false, false]).variance_test(x, group: g)\nlet w = dataset(wages).where(year == 2019)\nw.variance_test(wage, group: industry)\nw.variance_test(wage, group: female, method: f)\nreport([w.variance_test(wage, group: union), w.variance_test(wage, group: union, method: bartlett), w.variance_test(wage, group: union, method: f)])\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [1, 2, 3]).variance_test(x)\n",
        "frame(x = [1, 2, 3], g = [1, 1, 1]).variance_test(x, group: g)\n",
        "frame(x = [1.0, NA, 3], g = [1, 1, NA]).variance_test(x, group: g)\n",
        "frame(s = [\"a\", \"b\"], g = [1, 2]).variance_test(s, group: g)\n",
        "frame(x = [1, 2, 3], d = [date(\"2024-01-01\"), date(\"2024-01-02\"), date(\"2024-01-01\")]).variance_test(x, group: d)\n",
        "frame(x = [1, 2, 3, 4, 5, 6], g = [1, 1, 2, 2, 3, 3]).variance_test(x, group: g, method: f)\n",
        "frame(x = [1, 2, 3, 4, 5], g = [1, 1, 2, 2, 3]).variance_test(x, group: g, method: bartlett)\n",
        "frame(x = [1, 2, 3, 4, 5], g = [1, 1, 1, 1, 2]).variance_test(x, group: g, method: f)\n",
        "frame(x = [2, 2, 2, 4, 5], g = [1, 1, 1, 2, 2]).variance_test(x, group: g, method: bartlett)\n",
        "frame(x = [2, 2, 2, 4, 5], g = [1, 1, 1, 2, 2]).variance_test(x, group: g, method: f)\n",
        "frame(x = [1, 2, 3], g = [1, 2, 3]).variance_test(x, group: g)\n",
        // Added at release 0.8.0, step 24a2b, from the coverage report: no group with a value
        // at all, and Levene's test about the mean where every group is a pair.
        "frame(x = [1.0, NA, NA, NA], g = [NA, \"a\", \"b\", \"b\"]).variance_test(x, group: g)\n",
        "frame(x = [1.0, 2.0, 3.0, 5.0, 2.0, 2.5], g = [1, 1, 2, 2, 3, 3]).variance_test(x, group: g, center: mean)\n",
        "frame(x = [2, 2, 2, 2], g = [1, 1, 2, 2]).variance_test(x, group: g)\n",
        "frame(x = [1, 3, 2, 8], g = [1, 1, 2, 2]).variance_test(x, group: g)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(x, group: g, method: bartlett, center: mean)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(x, group: g, method: f, center: median)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(x, group: g, method: fligner)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(x, group: g, center: trimmed)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(y, group: g)\n",
        "frame(x = [1, 2, 3, 4], g = [1, 1, 2, 2]).variance_test(x, group: h)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(script);
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
