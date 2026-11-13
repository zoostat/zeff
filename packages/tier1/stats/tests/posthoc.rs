//! `posthoc` after the one-way `anova` (release 0.7.0, step 19; Language Reference, section 4;
//! open decisions #97 and #98): the reference (`tests/reference/r/posthoc.json`, R 4.6.0's
//! `TukeyHSD`, `pairwise.t.test` and `t.test`, DescTools' `ScheffeTest` and emmeans' `pairs` and
//! `confint`, on fifteen sets of groups and on hours by industry in the shipped wages), the
//! closed forms within a relative 1e-9 and Tukey's figures within the accuracy of R's `ptukey`
//! and `qtukey`; the kernel's closed forms and refusals; the properties (every p-value lies in
//! [0, 1], the methods keep their known order, an interval leaves out zero exactly when its
//! p-value is below 0.05, `none` after Welch's `anova` is the `t_test` of each pair, the answer
//! does not depend on the order of the rows, and frequency weights repeat rows); the edge cases
//! of the checklist (two groups, groups of unequal sizes, a group of one after `equal_var:
//! true`); the refusals; and snapshots of every form and every message. The form after a
//! factorial `anova` (step 19b) has its own file, `posthoc_factorial.rs`.

use std::collections::BTreeMap;
use std::f64::consts::SQRT_2;
use std::rc::Rc;

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Distribution, Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_dist::families;
use zeff_stats::anova::{GroupSummary, Undefined, anova};
use zeff_stats::p_adjust;
use zeff_stats::posthoc::{Method, compare, posthoc};
use zeff_stats::t_test::t_test;
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

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

/// `anova` by slot: data, var, group, equal_var, weights, weight_type, type, effect.
fn fit(frame: &Frame, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; 8];
    values[0] = Some(Value::Frame(frame.clone()));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    invoke(anova, values, "d.anova(x, group: g)").map(|(r, _)| r)
}

/// The one-way `anova` of `x` by `g`, Welch's or the classical form.
fn one_way(frame: &Frame, x: &str, g: &str, pooled: bool) -> Result<TestResult, Fail> {
    let mut slots = vec![(1, var(x)), (2, var(g))];
    if pooled {
        slots.push((3, Value::Bool(true)));
    }
    fit(frame, slots)
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

/// One comparison of a result read by its name, as a script reads it (open decision #100; step
/// 38d): `statistic(name)`, `df(name)` and `p_values(name)`; `effect_size(name)` is `NA`, a
/// comparison having none.
fn by_name(result: &TestResult, name: &str) -> (f64, f64, f64) {
    use zeff_results::interface::{df, effect_size, p_values, statistic};
    let read = |body: Body| {
        let args = Args {
            values: vec![
                Some(Value::TestResult(Rc::new(result.clone()))),
                Some(Value::Term(name.to_string())),
            ],
            rest: Vec::new(),
            definitions: Vec::new(),
        };
        let mut output = String::new();
        let mut messages: Vec<Diagnostic> = Vec::new();
        let mut random = Randomness::default();
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text: "t.statistic(name)",
            columns_used: Vec::new(),
            frame: None,
            group_rows: None,
            callee_span: Span::new(2, 11),
            arg_spans: vec![None; 2],
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
        };
        match body(&mut ctx, args) {
            Ok(value) => value,
            Err(_) => panic!("the comparison `{name}` is not read"),
        }
    };
    let float = |value: Value| match value {
        Value::Float(x) => x,
        other => panic!("`{name}`: not a number: {other:?}"),
    };
    let one_df = match read(df) {
        Value::List(items) if items.len() == 1 => float(items[0].clone()),
        other => panic!("`{name}`: not one number of degrees of freedom: {other:?}"),
    };
    assert_eq!(read(effect_size), Value::Na, "{name}");
    (float(read(statistic)), one_df, float(read(p_values)))
}

/// `posthoc` on a result, with its notes.
fn follow_with_notes(
    test: &TestResult,
    word: Option<&str>,
) -> Result<(Vec<Row>, Vec<Diagnostic>), Fail> {
    let values = vec![
        Some(Value::TestResult(Rc::new(test.clone()))),
        word.map(|w| Value::Word(w.to_string())),
    ];
    let (result, notes) = invoke(posthoc, values, "a.posthoc(method: holm)")?;
    assert_eq!(result.method, "posthoc");
    assert_eq!(result.level, 0.95);
    // The test of the whole stays the F of the `anova`.
    assert_eq!(result.statistic, test.statistic);
    assert_eq!(result.p_value.to_bits(), test.p_value.to_bits());
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
            // The comparison read by its name, as its table prints it and with the hyphen of
            // a keyboard: the figures every reference below is held to.
            let (t, df, p) = by_name(&result, &row.term);
            let typed = row.term.replace('\u{2212}', "-");
            assert_ne!(typed, row.term);
            let (t2, df2, p2) = by_name(&result, &typed);
            assert_eq!(
                (t.to_bits(), df.to_bits(), p.to_bits()),
                (t2.to_bits(), df2.to_bits(), p2.to_bits())
            );
            assert_eq!(t.to_bits(), row.statistic.value.to_bits());
            assert_eq!(p.to_bits(), row.p_value.to_bits());
            Row {
                term: row.term.clone(),
                difference: estimate.value,
                std_error: row.std_error.unwrap(),
                t,
                df,
                p,
                ci: estimate.ci,
            }
        })
        .collect::<Vec<Row>>();
    // The table has a line for every comparison.
    assert_eq!(result.table.as_ref().unwrap().rows.len(), rows.len());
    Ok((rows, notes))
}

fn follow(test: &TestResult, word: &str) -> Vec<Row> {
    follow_with_notes(test, Some(word)).unwrap().0
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

fn values(v: &Json) -> Vec<f64> {
    floats(v).into_iter().map(Option::unwrap).collect()
}

/// A figure for each comparison. R names the rows of `TukeyHSD` and `ScheffeTest` as
/// `second-first` when there are several, and the file keeps the names; one row alone is bare.
fn by_pair(v: &Json, first: &[&str], second: &[&str]) -> Vec<f64> {
    match v {
        Json::Object(_) => first
            .iter()
            .zip(second)
            .map(|(a, b)| number(&v[format!("{b}-{a}").as_str()]).unwrap())
            .collect(),
        other => values(other),
    }
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/posthoc.json")).unwrap();
    serde_json::from_str(&text).unwrap()
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

/// Two columns of the shipped wages: a number and a text.
fn wages(x: &str, g: &str) -> Frame {
    let text =
        std::fs::read_to_string(root().join("packages/tier1/datasets/data/wages.csv")).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (xi, gi) = (at(x), at(g));
    let mut xs = Vec::new();
    let mut gs = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        xs.push(cells[xi].parse::<f64>().ok());
        gs.push(match cells[gi] {
            "" | "NA" => None,
            s => Some(s.to_string()),
        });
    }
    frame_of(vec![(x, Column::floats(xs).0), (g, Column::strs(gs))])
}

/// The group summaries of plain values by group.
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

/// The studentized range of `k` groups on `df` degrees of freedom.
fn range(k: usize, df: f64) -> Distribution {
    Distribution::Family {
        name: "tukey",
        params: vec![("groups", k as f64), ("df", df)],
    }
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

/// One case against R: `x` by `g` in `frame`, the groups named in the table as `prefix` and
/// their value.
fn check_case(
    label: &str,
    case: &Json,
    frame: &Frame,
    x: &str,
    g: &str,
    prefix: &str,
) -> Vec<String> {
    let mut problems = Vec::new();
    let k = case["groups"].as_u64().unwrap() as usize;
    let n = case["n"].as_u64().unwrap() as usize;
    let first: Vec<&str> = case["first"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let second: Vec<&str> = case["second"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let m = k * (k - 1) / 2;
    assert_eq!(first.len(), m, "{label}");
    let terms: Vec<String> = first
        .iter()
        .zip(&second)
        .map(|(a, b)| format!("{prefix}{a} − {prefix}{b}"))
        .collect();
    let difference = values(&case["difference"]);
    let pooled_df = number(&case["pooled"]["df"]).unwrap();
    let pooled_se = values(&case["pooled"]["se"]);
    let pooled_t = values(&case["pooled"]["t"]);
    for pooled in [true, false] {
        let form = if pooled { "pooled" } else { "welch" };
        let fitted = one_way(frame, x, g, pooled);
        if !pooled && case.get("welch").is_none() {
            // A group of one value: Welch's `anova` is refused, so there is nothing to follow.
            assert!(
                fitted.is_err(),
                "{label}: Welch's anova should be undefined"
            );
            continue;
        }
        let fitted = fitted.unwrap();
        assert_eq!(fitted.n, n, "{label} n");
        // Tukey and Scheffé pool the variance whichever F the `anova` computed, and say so
        // after Welch's.
        for word in ["tukey", "scheffe"] {
            let mut against = Against {
                label: format!("{label} after {form} {word}"),
                problems: &mut problems,
            };
            let (rows, notes) = follow_with_notes(&fitted, Some(word)).unwrap();
            assert_eq!(notes.len(), usize::from(!pooled), "{label} {word} notes");
            if let Some(note) = notes.first() {
                assert!(
                    note.message
                        .starts_with(&format!("`{word}` pools the variances")),
                    "{}",
                    note.message
                );
            }
            assert_eq!(rows.len(), m, "{label}");
            let p = by_pair(&case[word]["p"], &first, &second);
            let lower = by_pair(&case[word]["lower"], &first, &second);
            let upper = by_pair(&case[word]["upper"], &first, &second);
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(row.term, terms[i], "{label}");
                assert_eq!(row.df, pooled_df, "{label}");
                against.close("difference", row.difference, difference[i], 1e-9, 1e-12);
                against.close("std.error", row.std_error, pooled_se[i], 1e-9, 0.0);
                against.close("t", row.t, pooled_t[i], 1e-9, 1e-12);
                let (lo, hi) = row.ci.unwrap();
                if word == "tukey" {
                    // R's `ptukey` is out by up to 3e-5 on 5 degrees of freedom and 1e-7 on 20,
                    // and `qtukey` stops at about four digits (the help page of `dist.tukey`).
                    let p_error = if pooled_df >= 20.0 { 1e-6 } else { 1e-4 };
                    against.close("p", row.p, p[i], 0.0, p_error);
                    against.close("lower", lo, lower[i], 0.0, 1e-3 * row.std_error);
                    against.close("upper", hi, upper[i], 0.0, 1e-3 * row.std_error);
                    // What R cannot settle: the range leaves 5% above the edge of the interval.
                    let edge = (hi - row.difference) / row.std_error * SQRT_2;
                    let tail = families::cdf(&range(k, pooled_df), edge).upper;
                    against.close(
                        "the tail at the edge of the interval",
                        tail,
                        0.05,
                        1e-9,
                        0.0,
                    );
                } else {
                    // DescTools takes the p-value from the lower tail, 1 − pf().
                    against.close("p", row.p, p[i], 1e-9, 1e-14);
                    against.close("lower", lo, lower[i], 1e-9, 1e-9 * row.std_error);
                    against.close("upper", hi, upper[i], 1e-9, 1e-9 * row.std_error);
                }
            }
        }
        // The pairwise t-tests under each method of `p_adjust`.
        let block = &case[form];
        let se = values(&block["se"]);
        let t = values(&block["t"]);
        let df: Vec<f64> = if pooled {
            vec![pooled_df; m]
        } else {
            values(&block["df"])
        };
        for word in [
            "bonferroni",
            "holm",
            "hochberg",
            "hommel",
            "bh",
            "by",
            "sidak",
            "none",
        ] {
            let mut against = Against {
                label: format!("{label} after {form} {word}"),
                problems: &mut problems,
            };
            let (rows, notes) = follow_with_notes(&fitted, Some(word)).unwrap();
            assert!(notes.is_empty(), "{label} {word}");
            assert_eq!(rows.len(), m, "{label}");
            let p = values(&block[word]);
            let limits = match word {
                "bonferroni" | "sidak" | "none" => Some(&block[format!("ci_{word}").as_str()]),
                _ => None,
            };
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(row.term, terms[i], "{label}");
                against.close("difference", row.difference, difference[i], 1e-9, 1e-12);
                against.close("std.error", row.std_error, se[i], 1e-9, 0.0);
                against.close("t", row.t, t[i], 1e-9, 1e-12);
                against.close("df", row.df, df[i], 1e-9, 0.0);
                // R writes Šidák's p-value as 1 − (1 − p)^m, which loses the digits of a small
                // p; the others keep theirs.
                let floor = if word == "sidak" { 1e-13 } else { 0.0 };
                against.close("p", row.p, p[i], 1e-9, floor);
                match limits {
                    Some(limits) => {
                        let (lo, hi) = row.ci.unwrap();
                        against.close(
                            "lower",
                            lo,
                            number(&limits["lower"][i]).unwrap(),
                            1e-9,
                            1e-9 * row.std_error,
                        );
                        against.close(
                            "upper",
                            hi,
                            number(&limits["upper"][i]).unwrap(),
                            1e-9,
                            1e-9 * row.std_error,
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
    problems
}

#[test]
fn every_method_agrees_with_r_on_fifteen_sets_of_groups() {
    let r = reference();
    let cases = r["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 15);
    let mut problems = Vec::new();
    let mut welch_defined = 0;
    for (i, case) in cases.iter().enumerate() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
        let prefix = if case["text_groups"].as_bool().unwrap() {
            ""
        } else {
            "g="
        };
        welch_defined += usize::from(case.get("welch").is_some());
        problems.extend(check_case(
            &format!("cases[{i}]"),
            case,
            &frame,
            "x",
            "g",
            prefix,
        ));
    }
    assert_eq!(welch_defined, 14);
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn every_method_agrees_with_r_on_hours_by_industry() {
    let r = reference();
    let case = &r["wages"];
    assert_eq!(case["groups"].as_u64(), Some(12));
    let frame = wages(case["x"].as_str().unwrap(), case["g"].as_str().unwrap());
    let problems = check_case("wages", case, &frame, "hours", "industry", "");
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

// ----- the kernel -----

const HOLM: Method = Method::Adjusted(p_adjust::Method::Holm);
const NONE: Method = Method::Adjusted(p_adjust::Method::Unadjusted);
const BONFERRONI: Method = Method::Adjusted(p_adjust::Method::Bonferroni);
const SIDAK: Method = Method::Adjusted(p_adjust::Method::Sidak);

fn t_dist(df: f64) -> Distribution {
    Distribution::Family {
        name: "t",
        params: vec![("df", df)],
    }
}

#[test]
fn the_kernel_follows_its_closed_forms_and_refuses_what_it_cannot_compute() {
    let x = vec![0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46];
    let y = vec![1.15, 0.88, 0.90, 0.74, 1.21];
    let z = vec![0.2, 0.9, 1.6, 2.4];
    let s = summaries(&[x.clone(), y.clone(), z.clone()]);
    // The words and the names.
    for word in Method::WORDS {
        assert!(Method::from_word(word).is_some(), "{word}");
    }
    assert_eq!(Method::from_word("tukey"), Some(Method::Tukey));
    assert_eq!(Method::from_word("holm"), Some(HOLM));
    assert_eq!(Method::from_word("dunnett"), None);
    assert_eq!(Method::Tukey.name(), "Tukey HSD");
    assert!(Method::Tukey.pools() && Method::Scheffe.pools() && !HOLM.pools());
    assert!(!HOLM.has_intervals() && BONFERRONI.has_intervals() && NONE.has_intervals());
    // The pooled form: every standard error is √(MS within · (1/nᵢ + 1/nⱼ)) on N − k.
    let ms: f64 = s.iter().map(|g| (g.n - 1.0) * g.variance).sum::<f64>() / 16.0;
    let plain = compare(&s, false, NONE, 0.95).unwrap();
    assert_eq!(plain.pooled_df, Some(16.0));
    assert_eq!(
        plain
            .rows
            .iter()
            .map(|r| (r.first, r.second))
            .collect::<Vec<_>>(),
        vec![(0, 1), (0, 2), (1, 2)]
    );
    let q = families::quantile(&t_dist(16.0), 0.975);
    for row in &plain.rows {
        let (a, b) = (&s[row.first], &s[row.second]);
        let se = (ms * (1.0 / a.n + 1.0 / b.n)).sqrt();
        assert!((row.difference - (a.mean - b.mean)).abs() < 1e-15);
        assert!(
            (row.std_error - se).abs() < 1e-14,
            "{} against {se}",
            row.std_error
        );
        assert!((row.t - (a.mean - b.mean) / se).abs() < 1e-12);
        assert_eq!(row.df, 16.0);
        let p = 2.0 * families::cdf(&t_dist(16.0), row.t.abs()).upper;
        assert!((row.p - p).abs() < 1e-15);
        let (lo, hi) = row.ci.unwrap();
        assert!((lo - (row.difference - q * se)).abs() < 1e-12);
        assert!((hi - (row.difference + q * se)).abs() < 1e-12);
    }
    // Bonferroni multiplies each p-value by 3 and takes the t quantile at 1 − 0.05/6; Šidák
    // gives 1 − (1 − p)³ and the quantile at 1 − (1 − 0.95^(1/3))/2; Holm has no interval.
    let bonferroni = compare(&s, false, BONFERRONI, 0.95).unwrap();
    let sidak = compare(&s, false, SIDAK, 0.95).unwrap();
    let holm = compare(&s, false, HOLM, 0.95).unwrap();
    let qb = families::quantile(&t_dist(16.0), 1.0 - 0.05 / 6.0);
    let qs = families::quantile(&t_dist(16.0), 1.0 - (1.0 - 0.95f64.powf(1.0 / 3.0)) / 2.0);
    for (i, row) in plain.rows.iter().enumerate() {
        let b = &bonferroni.rows[i];
        assert!((b.p - (3.0 * row.p).min(1.0)).abs() < 1e-15);
        assert!((b.ci.unwrap().1 - (row.difference + qb * row.std_error)).abs() < 1e-12);
        let k = &sidak.rows[i];
        assert!((k.p - (1.0 - (1.0 - row.p).powi(3))).abs() < 1e-14);
        assert!((k.ci.unwrap().1 - (row.difference + qs * row.std_error)).abs() < 1e-10);
        assert!(holm.rows[i].ci.is_none());
        assert!(holm.rows[i].p >= row.p && holm.rows[i].p <= b.p);
        // The interval is narrower on its own, wider under Šidák, widest under Bonferroni.
        assert!(row.ci.unwrap().1 < k.ci.unwrap().1 && k.ci.unwrap().1 < b.ci.unwrap().1);
    }
    // Tukey's p-value is the upper tail of the studentized range at |t|·√2, its interval the
    // quantile over √2; Scheffé's p-value is the upper tail of F(k − 1, N − k) at t²/(k − 1).
    let tukey = compare(&s, false, Method::Tukey, 0.95).unwrap();
    let scheffe = compare(&s, false, Method::Scheffe, 0.95).unwrap();
    let half = families::quantile(&range(3, 16.0), 0.95) / SQRT_2;
    let f = Distribution::Family {
        name: "f",
        params: vec![("df1", 2.0), ("df2", 16.0)],
    };
    let wide = (2.0 * families::quantile(&f, 0.95)).sqrt();
    for (i, row) in plain.rows.iter().enumerate() {
        let t = &tukey.rows[i];
        assert_eq!(
            (t.difference, t.std_error, t.t, t.df),
            (row.difference, row.std_error, row.t, row.df)
        );
        assert_eq!(
            t.p,
            families::cdf(&range(3, 16.0), row.t.abs() * SQRT_2).upper
        );
        assert!((t.ci.unwrap().0 - (row.difference - half * row.std_error)).abs() < 1e-12);
        let s = &scheffe.rows[i];
        let p = families::cdf(&f, row.t * row.t / 2.0).upper;
        assert!((s.p - p).abs() <= 1e-12 * p);
        assert!((s.ci.unwrap().1 - (row.difference + wide * row.std_error)).abs() < 1e-12);
        // Each is between the comparison on its own and certainty, and Scheffé's, which covers
        // every contrast, is the more cautious.
        assert!(row.p < t.p && t.p < s.p && s.p <= 1.0);
        assert!(t.ci.unwrap().1 < s.ci.unwrap().1);
    }
    // The methods that pool do so after Welch's `anova` too; the others turn to Welch's test of
    // each pair, on the Welch–Satterthwaite degrees of freedom.
    assert_eq!(compare(&s, true, Method::Tukey, 0.95), Ok(tukey.clone()));
    assert_eq!(
        compare(&s, true, Method::Scheffe, 0.95),
        Ok(scheffe.clone())
    );
    let welch = compare(&s, true, NONE, 0.95).unwrap();
    assert_eq!(welch.pooled_df, None);
    for row in &welch.rows {
        let (a, b) = (&s[row.first], &s[row.second]);
        let (u, v) = (a.variance / a.n, b.variance / b.n);
        let df = (u + v) * (u + v) / (u * u / (a.n - 1.0) + v * v / (b.n - 1.0));
        assert!((row.std_error - (u + v).sqrt()).abs() < 1e-15);
        assert!((row.df - df).abs() < 1e-12);
        assert!(row.df > a.n.min(b.n) - 1.0 && row.df < a.n + b.n - 2.0);
        let q = families::quantile(&t_dist(row.df), 0.975);
        assert!((row.ci.unwrap().1 - (row.difference + q * row.std_error)).abs() < 1e-12);
    }
    // Another level moves the intervals and leaves the p-values.
    let wider = compare(&s, false, Method::Tukey, 0.99).unwrap();
    for (a, b) in tukey.rows.iter().zip(&wider.rows) {
        assert_eq!(a.p, b.p);
        assert!(b.ci.unwrap().0 < a.ci.unwrap().0 && b.ci.unwrap().1 > a.ci.unwrap().1);
    }
    // The refusals.
    assert_eq!(
        compare(&s[..1], false, Method::Tukey, 0.95),
        Err(Undefined::TooFewGroups)
    );
    assert_eq!(compare(&[], true, HOLM, 0.95), Err(Undefined::TooFewGroups));
    let one = summaries(&[x.clone(), vec![2.0], z.clone()]);
    assert_eq!(
        compare(&one, true, HOLM, 0.95),
        Err(Undefined::GroupOfOne(1))
    );
    assert!(compare(&one, false, HOLM, 0.95).is_ok());
    assert!(compare(&one, true, Method::Tukey, 0.95).is_ok());
    let flat = summaries(&[vec![2.0, 2.0, 2.0], y.clone(), z.clone()]);
    assert_eq!(
        compare(&flat, true, NONE, 0.95),
        Err(Undefined::GroupWithoutSpread(0))
    );
    assert!(compare(&flat, false, NONE, 0.95).is_ok());
    let singles = summaries(&[vec![1.0], vec![2.0], vec![3.0]]);
    assert_eq!(
        compare(&singles, false, Method::Tukey, 0.95),
        Err(Undefined::NoResidualFreedom)
    );
    let constant = summaries(&[vec![1.0, 1.0], vec![2.0, 2.0]]);
    assert_eq!(
        compare(&constant, false, HOLM, 0.95),
        Err(Undefined::NoPooledVariance)
    );
}

#[test]
fn two_groups_give_the_t_test_under_every_method() {
    let x = vec![0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46];
    let y = vec![1.15, 0.88, 0.90, 0.74, 1.21];
    let frame = frame_of(vec![
        ("x", Column::floats(x.iter().chain(&y).map(|v| Some(*v))).0),
        ("g", Column::ints((0..15).map(|i| Some(i64::from(i >= 10))))),
    ]);
    for pooled in [true, false] {
        // `t_test` by slot: var 1, group 4, equal_var 5.
        let mut values: Vec<Option<Value>> = vec![None; 12];
        values[0] = Some(Value::Frame(frame.clone()));
        values[1] = Some(var("x"));
        values[4] = Some(var("g"));
        if pooled {
            values[5] = Some(Value::Bool(true));
        }
        let (test, _) = invoke(t_test, values, "d.t_test(x, group: g)").unwrap();
        let interval = test.estimate.as_ref().unwrap().ci.unwrap();
        let fitted = one_way(&frame, "x", "g", pooled).unwrap();
        for word in Method::WORDS {
            let rows = follow(&fitted, word);
            assert_eq!(rows.len(), 1, "{word}");
            let row = &rows[0];
            assert_eq!(row.term, "g=0 − g=1");
            assert!((row.difference - test.estimate.as_ref().unwrap().value).abs() < 1e-14);
            let method = Method::from_word(word).unwrap();
            if !pooled && method.pools() {
                // After Welch's `anova` Tukey and Scheffé give the pooled test, not Welch's.
                assert_eq!(row.df, 13.0);
                continue;
            }
            assert!((row.t - test.statistic.value).abs() < 1e-12, "{word}");
            assert!((row.df - test.statistic.df[0]).abs() < 1e-12, "{word}");
            // One comparison: no method has anything to adjust for. The range of two values is
            // |t|·√2 and F on 1 and N − 2 is t², each from its own integral or series.
            let slack = if method.pools() { 1e-11 } else { 1e-15 };
            assert!(
                (row.p - test.p_value).abs() <= slack,
                "{word}: {} against {}",
                row.p,
                test.p_value
            );
            match row.ci {
                Some((lo, hi)) => {
                    assert!(method.has_intervals(), "{word}");
                    assert!(
                        (lo - interval.0).abs() < 1e-9,
                        "{word}: {lo} against {}",
                        interval.0
                    );
                    assert!(
                        (hi - interval.1).abs() < 1e-9,
                        "{word}: {hi} against {}",
                        interval.1
                    );
                }
                None => assert!(!method.has_intervals(), "{word}"),
            }
        }
    }
}

// ----- the properties -----

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        * 48
}

/// Two sets of rows agree: the same comparisons, and every figure within a relative 1e-9.
fn same_rows(a: &[Row], b: &[Row]) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.len(), b.len());
    let near = |x: f64, y: f64| (x - y).abs() <= 1e-9 * y.abs().max(1.0);
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    #[test]
    fn the_comparisons_are_in_range_keep_the_order_of_the_methods_and_ignore_the_order_of_the_rows(
        values in prop::collection::vec((-3.0..3.0f64, 0..4i64, 1..4i64), 8..40),
        seed in 0..1000u64,
    ) {
        // Every group needs two distinct values for Welch's form.
        let mut by_group: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
        for (x, g, _) in &values {
            by_group.entry(*g).or_default().push(*x);
        }
        prop_assume!(by_group.len() >= 2);
        prop_assume!(by_group.values().all(|v| v.len() >= 2 && v.iter().any(|x| *x != v[0])));
        let k = by_group.len();
        let m = k * (k - 1) / 2;
        let frame = frame_of(vec![
            ("x", Column::floats(values.iter().map(|(x, _, _)| Some(*x))).0),
            ("g", Column::ints(values.iter().map(|(_, g, _)| Some(*g)))),
            ("w", Column::ints(values.iter().map(|(_, _, w)| Some(*w)))),
        ]);
        // A shuffle of the rows, and the rows written out as often as their weight says.
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
        for pooled in [false, true] {
            let fitted = one_way(&frame, "x", "g", pooled).unwrap();
            // Tukey and Scheffé are the same after either form, so once is enough.
            let words: Vec<&str> = Method::WORDS
                .into_iter()
                .filter(|w| pooled || !Method::from_word(w).unwrap().pools())
                .collect();
            let mut by_word: BTreeMap<&str, Vec<Row>> = BTreeMap::new();
            for word in &words {
                let rows = follow(&fitted, word);
                prop_assert_eq!(rows.len(), m);
                let method = Method::from_word(word).unwrap();
                for row in &rows {
                    prop_assert!((0.0..=1.0).contains(&row.p), "{}: p = {}", word, row.p);
                    prop_assert!(row.std_error > 0.0 && row.df > 0.0);
                    prop_assert_eq!(row.ci.is_some(), method.has_intervals());
                    if let Some((lo, hi)) = row.ci {
                        prop_assert!(lo < row.difference && row.difference < hi);
                        prop_assert!(((lo + hi) / 2.0 - row.difference).abs() <= 1e-9 * (hi - lo));
                        // The interval and the p-value tell one story at 5%.
                        if (row.p - 0.05).abs() > 1e-9 {
                            prop_assert_eq!(lo > 0.0 || hi < 0.0, row.p < 0.05, "{}: p = {}, [{}, {}]", word, row.p, lo, hi);
                        }
                    }
                }
                by_word.insert(*word, rows);
            }
            // The order of the methods, comparison by comparison.
            let p = |word: &str, i: usize| by_word[word][i].p;
            let slack = 1e-12;
            for i in 0..m {
                prop_assert!(p("none", i) <= p("sidak", i) + slack);
                prop_assert!(p("sidak", i) <= p("bonferroni", i) + slack);
                prop_assert!(p("none", i) <= p("hommel", i) + slack);
                prop_assert!(p("hommel", i) <= p("hochberg", i) + slack);
                prop_assert!(p("hochberg", i) <= p("holm", i) + slack);
                prop_assert!(p("holm", i) <= p("bonferroni", i) + slack);
                prop_assert!(p("none", i) <= p("bh", i) + slack);
                prop_assert!(p("bh", i) <= p("hochberg", i) + slack);
                prop_assert!(p("bh", i) <= p("by", i) + slack);
                if pooled {
                    prop_assert!(p("none", i) <= p("tukey", i) + slack);
                    prop_assert!(p("tukey", i) <= p("scheffe", i) + 1e-10);
                }
            }
            // `none` after Welch's `anova` is the `t_test` of each pair of groups.
            if !pooled {
                let codes: Vec<i64> = by_group.keys().copied().collect();
                let mut at = 0;
                for (i, a) in codes.iter().enumerate() {
                    for b in &codes[i + 1..] {
                        let pair: Vec<&(f64, i64, i64)> = values.iter().filter(|(_, g, _)| g == a || g == b).collect();
                        let two = frame_of(vec![
                            ("x", Column::floats(pair.iter().map(|(x, _, _)| Some(*x))).0),
                            ("g", Column::ints(pair.iter().map(|(_, g, _)| Some(*g)))),
                        ]);
                        let mut slots: Vec<Option<Value>> = vec![None; 12];
                        slots[0] = Some(Value::Frame(two));
                        slots[1] = Some(var("x"));
                        slots[4] = Some(var("g"));
                        let (test, _) = invoke(t_test, slots, "d.t_test(x, group: g)").unwrap();
                        let row = &by_word["none"][at];
                        prop_assert_eq!(&row.term, &format!("g={a} − g={b}"));
                        prop_assert!((row.t - test.statistic.value).abs() <= 1e-9 * test.statistic.value.abs().max(1.0));
                        prop_assert!((row.df - test.statistic.df[0]).abs() <= 1e-9 * row.df);
                        prop_assert!((row.p - test.p_value).abs() <= 1e-9);
                        let interval = test.estimate.as_ref().unwrap().ci.unwrap();
                        let (lo, hi) = row.ci.unwrap();
                        prop_assert!((lo - interval.0).abs() <= 1e-9 * (hi - lo) && (hi - interval.1).abs() <= 1e-9 * (hi - lo));
                        at += 1;
                    }
                }
            }
            // The order of the rows does not matter, and frequency weights repeat rows.
            let moved = one_way(&shuffled, "x", "g", pooled).unwrap();
            let mut weighted_slots = vec![(1, var("x")), (2, var("g")), (4, var("w")), (5, Value::Word("frequency".to_string()))];
            if pooled {
                weighted_slots.push((3, Value::Bool(true)));
            }
            let weighted = fit(&frame, weighted_slots).unwrap();
            let written = one_way(&expanded, "x", "g", pooled).unwrap();
            for word in ["holm", "sidak"] {
                same_rows(&follow(&moved, word), &by_word[word])?;
                same_rows(&follow(&weighted, word), &follow(&written, word))?;
            }
            if pooled {
                same_rows(&follow(&weighted, "scheffe"), &follow(&written, "scheffe"))?;
            }
        }
    }
}

// ----- the edge cases and the refusals -----

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn a_group_of_one_is_compared_on_the_pooled_variance() {
    let data = "let d = frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3])\n";
    // Welch's `anova` is refused, so there is nothing to follow.
    let (exit, _, stderr) = go(&format!("{data}d.anova(x, group: g).posthoc()\n"));
    assert_eq!(exit, 1);
    assert!(stderr.contains("g=3 has one"), "{stderr}");
    // After the classical form every method compares the three pairs on 2 degrees of freedom.
    for word in Method::WORDS {
        let (exit, stdout, stderr) = go(&format!(
            "{data}d.anova(x, group: g, equal_var: true).posthoc(method: {word})\n"
        ));
        assert_eq!(exit, 0, "{word}: {stderr}");
        assert!(stderr.is_empty(), "{word}: {stderr}");
        assert!(stdout.contains("g=1 − g=3"), "{word}: {stdout}");
        assert!(stdout.contains("g=2 − g=3"), "{word}: {stdout}");
        assert!(
            stdout.contains("pooled within-group variance on 2 degrees of freedom; N = 5"),
            "{word}: {stdout}"
        );
        assert!(!stdout.contains("   df   "), "{word}: {stdout}");
    }
}

#[test]
fn two_groups_and_groups_of_unequal_sizes_print_their_comparisons() {
    // Two groups: one comparison.
    let (exit, stdout, stderr) = go(
        "let d = frame(x = [0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.15, 0.88, 0.90, 0.74, 1.21], g = [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2])\nd.anova(x, group: g, equal_var: true).posthoc()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("g=1 − g=2"), "{stdout}");
    assert!(stdout.contains("1 comparison of 2 groups"), "{stdout}");
    // Unequal sizes: the standard error of each comparison follows its two groups.
    let (exit, stdout, stderr) = go(
        "let d = frame(x = [1, 2, 3, 4, 5, 6, 7, 8.5, 2, 3.5, 9, 4], g = [1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3])\nd.anova(x, group: g, equal_var: true).posthoc(method: bonferroni).estimates()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("frame · 3 rows × 9 variables"), "{stdout}");
}

#[test]
fn what_is_not_a_one_way_anova_is_refused() {
    let test = one_way(
        &frame_of(vec![
            (
                "x",
                Column::floats([1.0, 2.0, 4.0, 3.0, 5.5, 7.0].map(Some)).0,
            ),
            ("g", Column::ints([1, 1, 1, 2, 2, 2].map(Some))),
        ]),
        "x",
        "g",
        true,
    )
    .unwrap();
    // A word that is not a method, on a direct call (the checker refuses it before that).
    let failed = follow_with_notes(&test, Some("dunnett")).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`method:` of `posthoc` is one of tukey, bonferroni, holm, hochberg, hommel, bh, by, sidak, scheffe, none, not `dunnett`"
    );
    // A result of another test.
    let mut other = test.clone();
    other.method = "kruskal_test".to_string();
    let failed = follow_with_notes(&other, None).unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`posthoc` compares the group means of an `anova`, but this is the result of `kruskal_test`"
    );
    // `term:` chooses among the terms of a formula; the one-way form has none to choose.
    let failed = invoke(
        posthoc,
        vec![
            Some(Value::TestResult(Rc::new(test.clone()))),
            None,
            Some(Value::Term("g".to_string())),
        ],
        "a.posthoc(term: g)",
    )
    .unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`term:` chooses among the terms of a formula, but this `anova` compares the groups of `g` alone"
    );
    // A value that is no result at all.
    let failed = invoke(posthoc, vec![Some(Value::Int(3)), None], "a.posthoc()").unwrap_err();
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`posthoc` follows an `anova`, but this is an Int"
    );
    // A `posthoc` does not follow a `posthoc`.
    let (again, _) = invoke(
        posthoc,
        vec![Some(Value::TestResult(Rc::new(test.clone()))), None],
        "a.posthoc()",
    )
    .unwrap();
    let failed = follow_with_notes(&again, None).unwrap_err();
    assert!(
        failed.message.ends_with("the result of `posthoc`"),
        "{}",
        failed.message
    );
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\nlet a = g.anova(s, group: site, equal_var: true)\na.posthoc()\na.posthoc(method: scheffe)\na.posthoc(method: bonferroni)\na.posthoc(method: sidak)\na.posthoc(method: none)\na.posthoc(method: holm)\na.posthoc(method: hochberg)\na.posthoc(method: hommel)\na.posthoc(method: bh)\na.posthoc(method: by)\nlet w = g.anova(s, group: site)\nw.posthoc()\nw.posthoc(method: scheffe)\nw.posthoc(method: holm)\nw.posthoc(method: bonferroni)\nw.posthoc(method: sidak)\nw.posthoc(method: none)\ng.where(site < 3).anova(s, group: site).posthoc(method: holm)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.anova(v, group: k, equal_var: true).posthoc()\nt.anova(v, group: k, weights: w, weight_type: frequency).posthoc(method: holm)\nt.anova(v, group: k, weights: w, weight_type: analytic, equal_var: true).posthoc(method: sidak)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 3.5, 1.0], g = [1, 1, 2, 2, NA, 3, 3, 2, 3])\nm.anova(x, group: g).posthoc(method: bh)\nframe(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g, equal_var: true).posthoc()\nlet wages = dataset(wages)\nwages.anova(hours, group: industry, equal_var: true).posthoc()\nwages.where(year == 2010).anova(hours, group: female).posthoc(method: none)\nlet p = a.posthoc(method: bonferroni)\np.estimates()\np.ci()\np.statistic()\np.df()\np.p_values()\np.effect_size()\np.summary()\nlet h = w.posthoc(method: holm)\nh.estimates()\nh.ci()\nh.summary()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(method: dunnett)\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(method: 2)\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.kruskal_test(x, group: g).posthoc()\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.t_test(x, group: g).posthoc(method: holm)\n",
        "let d = frame(y = [1, 2, 4, 3, 5.5, 7, 6, 8], a = [1, 1, 2, 2, 1, 1, 2, 2], b = [1, 2, 1, 2, 1, 2, 1, 2])\nd.anova(y ~ a + b).posthoc()\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.posthoc()\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.reg(x ~ g).posthoc()\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc().posthoc()\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(level: 0.9)\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(method: none).ci(level: 0.9)\n",
        "let d = frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3])\nd.anova(x, group: g).posthoc()\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
