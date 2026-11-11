//! `ks_test` (release 0.7.0, step 15; Language Reference, sections 4, 6 and 10.2; open decision
//! #91; question 4 (c) of the release): the reference (`tests/reference/r/ks.json`, R 4.6.0's
//! `ks.test` against eleven families at sizes on both sides of the exact rule, with ties and
//! `NA`, and `nortest`'s `lillie.test`) through the package's own function, D within a relative
//! 1e-10, the exact and Lilliefors p-values within 1e-9, the asymptotic ones within 1e-9 of the
//! limit summed to convergence and within 1e-3 of R's truncated sum; the kernels' closed forms;
//! the properties (D in [1/(2n), 1], p in [0, 1], the same answer under any order of the rows and
//! under a common shift of the data and the distribution); the refusals; and snapshots of every
//! form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Distribution, Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::ks_test::{
    EXACT_BELOW, Method, kolmogorov_exact_cdf, kolmogorov_limit, ks_test, lilliefors, p_value,
    statistic, tied,
};
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
    unwrap: impl FnOnce(Value, &[Diagnostic]) -> T,
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
    let value = {
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
        body(&mut ctx, args)?
    };
    Ok(unwrap(value, &messages))
}

/// `d.ks_test(x, distribution, estimated:)` on a frame with the one column `x`; the result and
/// the codes of the messages given.
fn call(
    column: Column,
    distribution: Distribution,
    estimated: bool,
) -> Result<(TestResult, Vec<String>), Fail> {
    let table = Columns::new(vec![("x".to_string(), column)]).unwrap();
    let frame = Frame::new(table);
    let values = vec![
        Some(Value::Frame(frame)),
        Some(Value::Var("x".to_string())),
        Some(Value::Distribution(distribution)),
        Some(Value::Bool(estimated)),
    ];
    with_context(
        values,
        "d.ks_test(x, dist.normal())",
        ks_test,
        |v, m| match v {
            Value::TestResult(r) => ((*r).clone(), m.iter().map(|d| d.code.to_string()).collect()),
            other => panic!("not a TestResult: {other:?}"),
        },
    )
}

fn normal() -> Distribution {
    Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    }
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
    let text = std::fs::read_to_string(root().join("tests/reference/r/ks.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(1e-300)
}

fn static_name(s: &str) -> &'static str {
    for name in [
        "normal",
        "lognormal",
        "uniform",
        "exponential",
        "t",
        "chi2",
        "f",
        "gamma",
        "beta",
        "poisson",
        "mean",
        "sd",
        "meanlog",
        "sdlog",
        "min",
        "max",
        "rate",
        "df",
        "df1",
        "df2",
        "shape",
        "a",
        "b",
        "lambda",
    ] {
        if name == s {
            return name;
        }
    }
    panic!("no static name for {s}")
}

fn distribution_of(case: &Json) -> Distribution {
    let params = case["params"].as_object().unwrap();
    Distribution::Family {
        name: static_name(case["family"].as_str().unwrap()),
        params: params
            .iter()
            .map(|(k, v)| (static_name(k), number(v).unwrap()))
            .collect(),
    }
}

// ----- the reference -----

#[test]
fn d_and_the_p_value_agree_with_ks_test_against_every_family() {
    let r = reference();
    let mut problems = Vec::new();
    let (mut exact_cases, mut asymptotic_cases, mut tied_cases) = (0, 0, 0);
    for (i, case) in r["ks"].as_array().unwrap().iter().enumerate() {
        let x = floats(&case["x"]);
        let missing = x.iter().filter(|v| v.is_none()).count();
        let (result, codes) = call(Column::floats(x).0, distribution_of(case), false).unwrap();
        let label = format!("ks[{i}] ({} n = {})", case["family"], case["n"]);
        let d = number(&case["statistic"]).unwrap();
        if !close(result.statistic.value, d, 1e-10) {
            problems.push(format!(
                "{label} D: Zeff {:.15e}, R {d:.15e}",
                result.statistic.value
            ));
        }
        let p = number(&case["p_value"]).unwrap();
        let tight = number(&case["p_tight"]).unwrap();
        let exact = case["exact"].as_bool().unwrap();
        let ties = case["ties"].as_bool().unwrap();
        if exact {
            exact_cases += 1;
            if !close(result.p_value, p, 1e-9) {
                problems.push(format!(
                    "{label} exact p: Zeff {:.15e}, R {p:.15e}",
                    result.p_value
                ));
            }
            if !result.after[0].starts_with("exact") {
                problems.push(format!("{label}: not marked exact: {}", result.after[0]));
            }
        } else {
            asymptotic_cases += 1;
            if !close(result.p_value, tight, 1e-9) {
                problems.push(format!(
                    "{label} asymptotic p: Zeff {:.15e}, limit {tight:.15e}",
                    result.p_value
                ));
            }
            if !close(result.p_value, p, 1e-3) && (result.p_value - p).abs() > 1e-6 {
                problems.push(format!(
                    "{label} asymptotic p against R: Zeff {:.15e}, R {p:.15e}",
                    result.p_value
                ));
            }
            if !result.after[0].starts_with("asymptotic") {
                problems.push(format!(
                    "{label}: not marked asymptotic: {}",
                    result.after[0]
                ));
            }
        }
        if ties {
            tied_cases += 1;
        }
        if ties != codes.contains(&"W0212".to_string()) {
            problems.push(format!("{label}: ties {ties} but codes {codes:?}"));
        }
        let n = case["n"].as_u64().unwrap() as usize;
        if result.n != n || result.dropped != missing {
            problems.push(format!(
                "{label} n: Zeff {} with {} dropped, R {n} with {missing} NA",
                result.n, result.dropped
            ));
        }
        assert_eq!(result.statistic.name, "D");
        assert!(result.statistic.df.is_empty());
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    assert!(exact_cases >= 20 && asymptotic_cases >= 15 && tied_cases >= 3);
}

#[test]
fn d_and_the_p_value_agree_with_lillie_test() {
    let r = reference();
    let mut problems = Vec::new();
    for (i, case) in r["lilliefors"].as_array().unwrap().iter().enumerate() {
        let x = floats(&case["x"]);
        let (result, _) = call(Column::floats(x).0, normal(), true).unwrap();
        let label = format!("lilliefors[{i}] (n = {})", case["n"]);
        let d = number(&case["statistic"]).unwrap();
        if !close(result.statistic.value, d, 1e-10) {
            problems.push(format!(
                "{label} D: Zeff {:.15e}, R {d:.15e}",
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
        if result.n != n {
            problems.push(format!("{label} n: Zeff {}, R {n}", result.n));
        }
        assert!(result.header[0].contains("estimated from the data"));
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the kernels -----

#[test]
fn the_kernels_give_their_closed_forms() {
    // One value: P(D ≤ d) = 2d − 1 from 1/2.
    assert!((kolmogorov_exact_cdf(1, 0.75) - 0.5).abs() < 1e-15);
    assert!((kolmogorov_exact_cdf(1, 0.9) - 0.8).abs() < 1e-15);
    assert_eq!(kolmogorov_exact_cdf(1, 0.4), 0.0);
    assert_eq!(kolmogorov_exact_cdf(5, 0.0), 0.0);
    assert_eq!(kolmogorov_exact_cdf(5, 1.5), 1.0);
    // Marsaglia, Tsang and Wang's own example: n = 10, d = 0.274 gives 0.6284796154565043.
    assert!(
        (kolmogorov_exact_cdf(10, 0.274) - 0.628_479_615_456_504_3).abs() < 1e-12,
        "{}",
        kolmogorov_exact_cdf(10, 0.274)
    );
    // The limit: K(1) = 0.73000, K(2) = 0.99933; the two series agree where they meet; both
    // tails sum to one.
    let k1 = kolmogorov_limit(1.0);
    assert!(
        (k1.lower - 0.730_000_328_322_420).abs() < 1e-12,
        "{}",
        k1.lower
    );
    assert!((kolmogorov_limit(2.0).lower - 0.999_329_074_744_220).abs() < 1e-12);
    let below = kolmogorov_limit(1.0 - 1e-12);
    assert!((below.lower - k1.lower).abs() < 1e-9);
    for x in [0.3, 0.6, 0.9, 1.0, 1.2, 1.8, 3.0] {
        let t = kolmogorov_limit(x);
        assert!((t.lower + t.upper - 1.0).abs() < 1e-14, "{x}");
    }
    assert_eq!(kolmogorov_limit(0.0).upper, 1.0);
    assert!(kolmogorov_limit(f64::NAN).lower.is_nan());
    // D of the midpoints of n equal cells against the uniform is 1/(2n).
    let n = 8;
    let grid: Vec<f64> = (0..n).map(|i| (i as f64 + 0.5) / n as f64).collect();
    assert!((statistic(&grid, |x| x) - 0.5 / n as f64).abs() < 1e-15);
    // The rule: exact below 100 without ties, asymptotic otherwise.
    assert_eq!(p_value(99, 0.1, false).1, Method::Exact);
    assert_eq!(p_value(99, 0.1, true).1, Method::Asymptotic);
    assert_eq!(p_value(EXACT_BELOW, 0.1, false).1, Method::Asymptotic);
    // Ties.
    assert_eq!(tied(&[1.0, 2.0, 3.0]), 0);
    assert_eq!(tied(&[1.0, 1.0, 2.0, 3.0, 3.0, 3.0]), 5);
    // Lilliefors needs five values and a spread.
    assert!(lilliefors(&[1.0, 2.0, 3.0, 4.0]).is_none());
    assert!(lilliefors(&[2.0, 2.0, 2.0, 2.0, 2.0]).is_none());
    let (d, p, m, s) = lilliefors(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    assert!(d > 0.0 && (0.0..=1.0).contains(&p));
    assert_eq!(m, 3.0);
    assert!((s - 2.5f64.sqrt()).abs() < 1e-15);
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
    fn d_and_p_lie_in_their_ranges_and_ignore_order_and_a_common_shift(
        values in prop::collection::vec(-4.0..4.0f64, 1..70),
        shift in -50.0..50.0f64,
        seed in 0..1000u64,
    ) {
        let n = values.len();
        let (plain, codes) = call(Column::floats(values.iter().map(|v| Some(*v))).0, normal(), false).unwrap();
        prop_assert!(plain.statistic.value >= 0.5 / n as f64 - 1e-12 && plain.statistic.value <= 1.0);
        prop_assert!((0.0..=1.0).contains(&plain.p_value));
        prop_assert!(codes.is_empty(), "{codes:?}");
        // The exact p-value is one minus the exact distribution function below 100 values.
        let expected = 1.0 - kolmogorov_exact_cdf(n, plain.statistic.value);
        prop_assert!((plain.p_value - expected.clamp(0.0, 1.0)).abs() < 1e-15);
        // Any order of the rows.
        let mut order: Vec<usize> = (0..n).collect();
        let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        for i in (1..order.len()).rev() {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            let j = (state >> 33) as usize % (i + 1);
            order.swap(i, j);
        }
        let (shuffled, _) = call(Column::floats(order.iter().map(|&i| Some(values[i]))).0, normal(), false).unwrap();
        prop_assert_eq!(shuffled.statistic.value.to_bits(), plain.statistic.value.to_bits());
        prop_assert_eq!(shuffled.p_value.to_bits(), plain.p_value.to_bits());
        // The data and the distribution shifted together.
        let moved = Distribution::Family { name: "normal", params: vec![("mean", shift), ("sd", 1.0)] };
        let (shifted, _) = call(Column::floats(values.iter().map(|v| Some(*v + shift))).0, moved, false).unwrap();
        prop_assert!((shifted.statistic.value - plain.statistic.value).abs() < 1e-9);
        prop_assert!((shifted.p_value - plain.p_value).abs() < 1e-8);
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
fn ties_are_warned_and_an_infinite_value_is_refused() {
    let (tied_result, codes) = call(
        Column::ints([3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5].into_iter().map(Some)),
        Distribution::Family {
            name: "normal",
            params: vec![("mean", 4.0), ("sd", 2.5)],
        },
        false,
    )
    .unwrap();
    assert_eq!(codes, vec!["W0212".to_string()]);
    assert!(tied_result.after[0].ends_with("with ties present"));
    let infinite = call(
        Column::floats([1.0, 2.0, f64::INFINITY, 4.0].into_iter().map(Some)).0,
        normal(),
        false,
    )
    .unwrap_err();
    assert_eq!(infinite.code, "Z0401");
    assert!(
        infinite.message.contains("infinite"),
        "{}",
        infinite.message
    );
    let other = call(
        Column::floats([1.0, 2.0, 3.0, 4.0, 5.0].into_iter().map(Some)).0,
        Distribution::Family {
            name: "exponential",
            params: vec![("rate", 1.0)],
        },
        true,
    )
    .unwrap_err();
    assert_eq!(other.code, "Z0422");
}

#[test]
fn the_wages_print_both_forms() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5))\nwages.ks_test(lwage, dist.normal(), estimated: true)\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5)).df()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stdout.contains("ks_test · lwage · normal distribution · mean: 3, sd: 0.5"),
        "{stdout}"
    );
    assert!(
        stdout.contains("ks_test · lwage · normal family, parameters estimated"),
        "{stdout}"
    );
    assert!(stdout.contains("D = 0."), "{stdout}");
    assert!(stdout.ends_with("NA\n"), "{stdout}");
    assert!(stderr.contains("rows with NA in lwage dropped"), "{stderr}");
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(u = [0.11, 0.32, 0.45, 0.58, 0.63, 0.77, 0.84, 0.92])\ng.ks_test(u, dist.uniform())\ng.ks_test(u, dist.normal(mean: 0.5, sd: 0.3))\nframe(x = [1.2, 3.4, 2.2, 2.2, 5.1]).ks_test(x, dist.normal(mean: 2, sd: 1))\nframe(x = [1.2, 3.4, 2.2, 0.8, 5.1, 2.9, 3.3]).ks_test(x, dist.normal(), estimated: true)\nframe(x = [0.4]).ks_test(x, dist.uniform())\nframe(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5]).ks_test(x, dist.exponential(rate: 0.25))\nlet wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5))\nwages.ks_test(lwage, dist.normal(), estimated: true)\nwages.ks_test(lwage, dist.truncated(dist.normal(mean: 3, sd: 0.5), lower: 1.5, upper: 4.6))\nwages.ks_test(educ, dist.poisson(lambda: 12))\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5)).statistic()\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5)).p_values()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "frame(x = [1, 2, 3]).ks_test(y, dist.normal())\n",
        "frame(x = [1, 2, 3]).ks_test(x)\n",
        "frame(x = [1, 2, 3]).ks_test(x, 5)\n",
        "frame(s = [\"a\", \"b\", \"c\"]).ks_test(s, dist.normal())\n",
        "frame(x = [1, NA]).where(x > 1).ks_test(x, dist.normal())\n",
        "frame(x = [1, 2, 3, 4]).ks_test(x, dist.normal(), estimated: true)\n",
        "frame(x = [2, 2, 2, 2, 2]).ks_test(x, dist.normal(), estimated: true)\n",
        "frame(x = [1, 2, 3, 4, 5]).ks_test(x, dist.exponential(), estimated: true)\n",
        "frame(x = [1, 2, 3, 4, 5]).ks_test(x, dist.truncated(dist.normal(), lower: 0, upper: 10), estimated: true)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
