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
//! factorial `anova` (step 19b) has its own file, `posthoc_factorial.rs`. The Games–Howell
//! comparison (release 0.9.0, step 29; open decision #85) is held to
//! `tests/reference/r/posthoc_games_howell.json`, its formula written out in base R and checked
//! there against `rstatix::games_howell_test`, at the end of this file.

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
            width: zeff_core::display::PAGE_WIDTH,
            shown: None,
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
            width: zeff_core::display::PAGE_WIDTH,
            shown: None,
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
    follow_at(test, word, None)
}

/// `posthoc` on a result at a level, 95% when none is given, with its notes.
fn follow_at(
    test: &TestResult,
    word: Option<&str>,
    level: Option<f64>,
) -> Result<(Vec<Row>, Vec<Diagnostic>), Fail> {
    // The slots of `posthoc`: fit, method, term, level.
    let values = vec![
        Some(Value::TestResult(Rc::new(test.clone()))),
        word.map(|w| Value::Word(w.to_string())),
        None,
        level.map(Value::Float),
    ];
    let (result, notes) = invoke(posthoc, values, "a.posthoc(method: holm)")?;
    assert_eq!(result.method, "posthoc");
    assert_eq!(result.level, level.unwrap_or(0.95));
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

/// `level:` (open decision #105, option A; step 38f): the intervals of the five single-step
/// methods at 0.90 and 0.99 against R (`tests/reference/r/posthoc_level.json`): `TukeyHSD` and
/// `DescTools::ScheffeTest` with `conf.level = `, and the formulas of `none`, Bonferroni and
/// Šidák written out with `qt` on the pooled variance and through `t.test(conf.level = )` for
/// Welch's pairs.
#[test]
fn the_intervals_at_other_levels_agree_with_r() {
    let text =
        std::fs::read_to_string(root().join("tests/reference/r/posthoc_level.json")).unwrap();
    let r: Json = serde_json::from_str(&text).unwrap();
    let cases = r["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    let mut problems = Vec::new();
    let mut compared = 0;
    for (i, case) in cases.iter().enumerate() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
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
        let k = {
            let mut names: Vec<&str> = first.iter().chain(&second).copied().collect();
            names.sort_unstable();
            names.dedup();
            names.len()
        };
        let difference = values(&case["difference"]);
        for at in case["levels"].as_array().unwrap() {
            let level = number(&at["level"]).unwrap();
            for pooled in [true, false] {
                let form = if pooled { "pooled" } else { "welch" };
                if !pooled && at.get("welch").is_none() {
                    continue;
                }
                let fitted = one_way(&frame, "x", "g", pooled).unwrap();
                for word in ["tukey", "scheffe", "none", "bonferroni", "sidak"] {
                    let mut against = Against {
                        label: format!("cases[{i}] at {level} after {form} {word}"),
                        problems: &mut problems,
                    };
                    let (rows, _) = follow_at(&fitted, Some(word), Some(level)).unwrap();
                    let (same_p, _) = follow_with_notes(&fitted, Some(word)).unwrap();
                    // Tukey and Scheffé pool whichever F the `anova` computed.
                    let block = match word {
                        "tukey" | "scheffe" => &at[word],
                        "none" => &at[form]["none"],
                        word => &at[form][word],
                    };
                    let lower = by_pair(&block["lower"], &first, &second);
                    let upper = by_pair(&block["upper"], &first, &second);
                    assert_eq!(rows.len(), lower.len());
                    for (j, row) in rows.iter().enumerate() {
                        assert_eq!(row.p.to_bits(), same_p[j].p.to_bits(), "{word}");
                        assert_eq!(row.t.to_bits(), same_p[j].t.to_bits(), "{word}");
                        against.close("difference", row.difference, difference[j], 1e-9, 1e-12);
                        let (lo, hi) = row.ci.unwrap();
                        if word == "tukey" {
                            // R's `qtukey` stops at about four digits; what it cannot settle is
                            // held exactly: the range leaves 1 − level above the edge.
                            against.close("lower", lo, lower[j], 0.0, 1e-3 * row.std_error);
                            against.close("upper", hi, upper[j], 0.0, 1e-3 * row.std_error);
                            let edge = (hi - row.difference) / row.std_error * SQRT_2;
                            let tail = families::cdf(&range(k, row.df), edge).upper;
                            against.close("the tail at the edge", tail, 1.0 - level, 1e-9, 0.0);
                        } else {
                            against.close("lower", lo, lower[j], 1e-9, 1e-9 * row.std_error);
                            against.close("upper", hi, upper[j], 1e-9, 1e-9 * row.std_error);
                        }
                        compared += 1;
                    }
                }
            }
        }
    }
    assert!(compared > 1000, "{compared} intervals compared");
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// At 0.95 every interval is what it was before `level:`, bit for bit; a higher level gives a
/// wider interval round the same difference; a step-wise method has none at any level.
#[test]
fn the_level_widens_the_intervals_and_leaves_the_rest() {
    let r = reference();
    for case in r["cases"].as_array().unwrap() {
        let frame = frame_of(vec![
            ("x", Column::floats(floats(&case["x"])).0),
            ("g", group_column(case)),
        ]);
        for pooled in [true, false] {
            let Ok(fitted) = one_way(&frame, "x", "g", pooled) else {
                continue;
            };
            for word in ["tukey", "scheffe", "none", "bonferroni", "sidak"] {
                let (plain, _) = follow_with_notes(&fitted, Some(word)).unwrap();
                let (at_95, _) = follow_at(&fitted, Some(word), Some(0.95)).unwrap();
                let mut before: Option<Vec<f64>> = None;
                for level in [0.5, 0.8, 0.9, 0.95, 0.99, 0.999] {
                    let (rows, _) = follow_at(&fitted, Some(word), Some(level)).unwrap();
                    let widths: Vec<f64> = rows
                        .iter()
                        .zip(&plain)
                        .map(|(row, usual)| {
                            let (lo, hi) = row.ci.unwrap();
                            assert_eq!(row.difference.to_bits(), usual.difference.to_bits());
                            assert_eq!(row.p.to_bits(), usual.p.to_bits());
                            assert!(lo < row.difference && row.difference < hi, "{word} {level}");
                            // The interval is centred on the difference.
                            let centre = (lo + hi) / 2.0;
                            assert!(
                                (centre - row.difference).abs() <= 1e-9 * (1.0 + hi - lo),
                                "{word} {level}"
                            );
                            hi - lo
                        })
                        .collect();
                    if let Some(narrower) = &before {
                        for (wide, narrow) in widths.iter().zip(narrower) {
                            assert!(wide > narrow, "{word} at {level}: {wide} against {narrow}");
                        }
                    }
                    before = Some(widths);
                }
                for (row, usual) in at_95.iter().zip(&plain) {
                    let ((lo, hi), (lo0, hi0)) = (row.ci.unwrap(), usual.ci.unwrap());
                    assert_eq!((lo.to_bits(), hi.to_bits()), (lo0.to_bits(), hi0.to_bits()));
                }
            }
            for word in ["holm", "hochberg", "hommel", "bh", "by"] {
                let (rows, _) = follow_at(&fitted, Some(word), Some(0.99)).unwrap();
                assert!(rows.iter().all(|row| row.ci.is_none()), "{word}");
            }
        }
    }
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
    // `t_test` by slot: var 1, group 4, equal_var 5.
    let t_test_of = |pooled: bool| {
        let mut values: Vec<Option<Value>> = vec![None; 12];
        values[0] = Some(Value::Frame(frame.clone()));
        values[1] = Some(var("x"));
        values[4] = Some(var("g"));
        if pooled {
            values[5] = Some(Value::Bool(true));
        }
        invoke(t_test, values, "d.t_test(x, group: g)").unwrap().0
    };
    for pooled in [true, false] {
        let fitted = one_way(&frame, "x", "g", pooled).unwrap();
        for word in Method::WORDS {
            let method = Method::from_word(word).unwrap();
            // Games–Howell is Welch's test of the pair whichever `anova` it follows.
            let test = t_test_of(pooled && !method.own_variances());
            let interval = test.estimate.as_ref().unwrap().ci.unwrap();
            let rows = follow(&fitted, word);
            assert_eq!(rows.len(), 1, "{word}");
            let row = &rows[0];
            assert_eq!(row.term, "g=0 − g=1");
            assert!((row.difference - test.estimate.as_ref().unwrap().value).abs() < 1e-14);
            if !pooled && method.pools() {
                // After Welch's `anova` Tukey and Scheffé give the pooled test, not Welch's.
                assert_eq!(row.df, 13.0);
                continue;
            }
            assert!((row.t - test.statistic.value).abs() < 1e-12, "{word}");
            assert!((row.df - test.statistic.df[0]).abs() < 1e-12, "{word}");
            // One comparison: no method has anything to adjust for. The range of two values is
            // |t|·√2 and F on 1 and N − 2 is t², each from its own integral or series.
            let slack = if method.pools() || method.own_variances() {
                1e-11
            } else {
                1e-15
            };
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
    // After the classical form every method but one compares the three pairs on 2 degrees of
    // freedom. Games–Howell takes each group's own variance, and a group of one has none: it
    // is refused in words that name the group and the method that admits it.
    for word in Method::WORDS {
        let (exit, stdout, stderr) = go(&format!(
            "{data}d.anova(x, group: g, equal_var: true).posthoc(method: {word})\n"
        ));
        if word == "games_howell" {
            assert_eq!(exit, 1, "{stdout}");
            assert!(
                stderr.contains("error[Z0401]: `games_howell` takes each group's own variance, which needs at least two values of `x` in every group, but g=3 has one"),
                "{stderr}"
            );
            assert!(stderr.contains("`method: tukey`"), "{stderr}");
            continue;
        }
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
        "`method:` of `posthoc` is one of tukey, games_howell, bonferroni, holm, hochberg, hommel, bh, by, sidak, scheffe, none, not `dunnett`"
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

/// `level:` through a script (open decision #105, option A; step 38f): the heading, the line
/// under the table, `ci()` on the result, and the note where the method has no intervals.
#[test]
fn the_level_is_named_in_the_table_and_a_method_without_intervals_says_so() {
    let data = "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\nlet a = g.anova(s, group: site, equal_var: true)\n";
    let (exit, stdout, stderr) = go(&format!(
        "{data}a.posthoc(level: 0.99)\na.posthoc(method: scheffe, level: 0.9)\na.posthoc(method: bonferroni, level: 0.9)\na.posthoc(method: sidak, level: 0.99)\na.posthoc(method: none, level: 0.9)\na.posthoc(method: none, level: 0.9).ci()\ng.anova(s, group: site).posthoc(method: bonferroni, level: 0.99)\na.posthoc(method: holm, level: 0.99)\na.posthoc(method: bh, level: 0.9)\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("levels", format!("{stdout}---\n{stderr}"));
    for said in [
        "p value   99% CI\n",
        "p value   90% CI\n",
        "each interval at the level 1 − 0.1/3",
        "each interval at the level 0.99^(1/3)",
        "ci · 90%\n",
    ] {
        assert!(stdout.contains(said), "{said}\n{stdout}");
    }
    // The note, once for each call that wrote a level where there is no interval, and for no
    // other.
    assert_eq!(stderr.matches("note[W0401]").count(), 2, "{stderr}");
    assert!(
        stderr.contains("note[W0401]: `level:` changes nothing here: Holm adjusts the p-values step by step and has no intervals (line 10)"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "Benjamini–Hochberg adjusts the p-values step by step and has no intervals (line 11)"
        ),
        "{stderr}"
    );
    // Left out, or with a method that has intervals, there is no note; and the p-values and
    // the test of the whole are the same at every level.
    let (exit, stdout, stderr) = go(&format!(
        "{data}let plain = a.posthoc()\nlet wide = a.posthoc(level: 0.99)\nlet h = a.posthoc(method: holm)\nplain.p_values(\"site=1 − site=2\") == wide.p_values(\"site=1 − site=2\")\nplain.statistic() == wide.statistic()\nh.ci()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stderr, "");
    assert_eq!(stdout, "true\ntrue\nNA\n");
    // A pragma allows the note, and `--deny-warnings` is not needed to see it.
    let (exit, _, stderr) = go(&format!(
        "{data}a.posthoc(method: holm, level: 0.99) // zeff: allow(W0401) the level is kept for the day the method changes\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert!(!stderr.contains("W0401"), "{stderr}");
    // After a formula the level reaches the marginal means' intervals too.
    let (exit, stdout, stderr) = go(
        "let f = frame(gain = [4.1, 4.9, 3.6, 4.6, 5.3, 4.0, 5.0, 5.8, 4.4, 5.2, 6.4, 5.1, 5.2, 5.9, 4.6, 6.9, 7.8, 6.1], dose = [\"low\", \"low\", \"low\", \"low\", \"low\", \"low\", \"mid\", \"mid\", \"mid\", \"mid\", \"mid\", \"mid\", \"top\", \"top\", \"top\", \"top\", \"top\", \"top\"])\nlet m = f.anova(gain ~ category(dose))\nlet wide = m.posthoc(level: 0.99).estimates()\nlet usual = m.posthoc().estimates()\nwide.min(ci_lower) < usual.min(ci_lower)\nwide.max(ci_upper) > usual.max(ci_upper)\nm.posthoc(level: 0.99)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.starts_with("true\ntrue\n"), "{stdout}");
    assert!(stdout.contains("p value   99% CI\n"), "{stdout}");
}

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
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(level: 1.5)\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(level: 0)\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(level: \"high\")\n",
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(level: 0.9).ci(level: 0.95)\n",
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

// ----- the Games–Howell comparison (open decision #85; release 0.9.0, step 29) -----

const GAMES_HOWELL: Method = Method::GamesHowell;

fn games_howell_reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/posthoc_games_howell.json"))
        .unwrap();
    serde_json::from_str(&text).unwrap()
}

/// One case against R: the difference, the standard error, t and the degrees of freedom to
/// nine digits; the p-value within the accuracy of R's `ptukey` on those degrees of freedom;
/// the limits at three levels within that of `qtukey`, and held exactly by the tail the range
/// leaves above each interval's edge. After either form of `anova` the rows are the same to
/// the bit. Gives the problems and the number of p-values and of intervals compared with R.
fn check_games_howell(
    label: &str,
    case: &Json,
    frame: &Frame,
    x: &str,
    g: &str,
    prefix: &str,
) -> (Vec<String>, usize, usize) {
    let mut problems = Vec::new();
    let k = case["groups"].as_u64().unwrap() as usize;
    let names = |key: &str| -> Vec<String> {
        case[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    let (first, second) = (names("first"), names("second"));
    let m = k * (k - 1) / 2;
    assert_eq!(first.len(), m, "{label}");
    let difference = values(&case["difference"]);
    let se = values(&case["se"]);
    let t = values(&case["t"]);
    let df = values(&case["df"]);
    // R has no value below two degrees of freedom.
    let p = floats(&case["p"]);
    let (mut p_compared, mut intervals_compared) = (0, 0);
    let welch = one_way(frame, x, g, false).unwrap();
    let classical = one_way(frame, x, g, true).unwrap();
    let (rows, notes) = follow_with_notes(&welch, Some("games_howell")).unwrap();
    // After Welch's `anova` nothing is said: the comparison assumes what the `anova` did.
    assert!(notes.is_empty(), "{label}: {:?}", notes[0].message);
    // After the classical form the rows are the same to the bit, and a note says that the
    // `anova` assumed equal variances.
    let (after_classical, notes) = follow_with_notes(&classical, Some("games_howell")).unwrap();
    assert_eq!(notes.len(), 1, "{label}");
    assert!(
        notes[0].message.starts_with(
            "`games_howell` takes each group's own variance, where the `anova` assumed them equal"
        ),
        "{}",
        notes[0].message
    );
    assert_eq!(rows.len(), m, "{label}");
    for (row, same) in rows.iter().zip(&after_classical) {
        assert_eq!(row.term, same.term, "{label}");
        for (a, b) in [
            (row.difference, same.difference),
            (row.std_error, same.std_error),
            (row.t, same.t),
            (row.df, same.df),
            (row.p, same.p),
            (row.ci.unwrap().0, same.ci.unwrap().0),
            (row.ci.unwrap().1, same.ci.unwrap().1),
        ] {
            assert_eq!(a.to_bits(), b.to_bits(), "{label} {}", row.term);
        }
    }
    {
        let mut against = Against {
            label: format!("{label} games_howell"),
            problems: &mut problems,
        };
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(
                row.term,
                format!("{prefix}{} − {prefix}{}", first[i], second[i]),
                "{label}"
            );
            against.close("difference", row.difference, difference[i], 1e-9, 1e-12);
            against.close("std.error", row.std_error, se[i], 1e-9, 0.0);
            against.close("t", row.t, t[i], 1e-9, 1e-12);
            against.close("df", row.df, df[i], 1e-9, 0.0);
            assert!((0.0..=1.0).contains(&row.p), "{label} {}", row.p);
            match p[i] {
                // R's `ptukey` is out by up to 5e-3 on 2 degrees of freedom, 3e-5 on 5 and
                // 1e-7 on 20 (the help page of `dist.tukey`).
                Some(theirs) => {
                    let p_error = if row.df >= 20.0 {
                        1e-6
                    } else if row.df >= 5.0 {
                        1e-4
                    } else {
                        5e-3
                    };
                    against.close("p", row.p, theirs, 0.0, p_error);
                    p_compared += 1;
                }
                None => assert!(row.df < 2.0, "{label}: R has no p-value on {}", row.df),
            }
        }
    }
    for at in case["levels"].as_array().unwrap() {
        let level = number(&at["level"]).unwrap();
        let lower = floats(&at["lower"]);
        let upper = floats(&at["upper"]);
        let (at_level, _) = follow_at(&welch, Some("games_howell"), Some(level)).unwrap();
        let mut against = Against {
            label: format!("{label} games_howell at {level}"),
            problems: &mut problems,
        };
        for (i, row) in at_level.iter().enumerate() {
            // The level moves the interval alone.
            assert_eq!(row.p.to_bits(), rows[i].p.to_bits());
            assert_eq!(row.t.to_bits(), rows[i].t.to_bits());
            let (lo, hi) = row.ci.unwrap();
            assert!(lo < row.difference && row.difference < hi, "{label}");
            // The interval is the difference with the same reach on both sides, and the
            // range on the pair's own degrees of freedom leaves 1 − level above its edge.
            against.close(
                "the centre",
                (lo + hi) / 2.0,
                row.difference,
                1e-9,
                1e-9 * row.std_error,
            );
            let edge = (hi - row.difference) / row.std_error * SQRT_2;
            let tail = families::cdf(&range(k, row.df), edge).upper;
            against.close("the tail at the edge", tail, 1.0 - level, 1e-9, 0.0);
            if let (Some(theirs_lower), Some(theirs_upper)) = (lower[i], upper[i]) {
                // R's `qtukey` stops at about four digits, and at fewer on few degrees of
                // freedom, where the quantile is large and its search stops early.
                let reach = if row.df >= 5.0 { 1e-3 } else { 2e-2 } * (hi - lo);
                against.close("lower", lo, theirs_lower, 0.0, reach);
                against.close("upper", hi, theirs_upper, 0.0, reach);
                intervals_compared += 1;
            } else {
                assert!(row.df < 2.0, "{label}: R has no interval on {}", row.df);
            }
        }
    }
    (problems, p_compared, intervals_compared)
}

#[test]
fn games_howell_agrees_with_r_on_seventeen_sets_of_groups() {
    let r = games_howell_reference();
    let cases = r["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 17);
    let mut problems = Vec::new();
    let (mut p_compared, mut intervals_compared) = (0, 0);
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
        let (found, p, intervals) =
            check_games_howell(&format!("cases[{i}]"), case, &frame, "x", "g", prefix);
        problems.extend(found);
        p_compared += p;
        intervals_compared += intervals;
    }
    // Every pair but the six of the groups of two values, where R has none.
    assert_eq!(p_compared, 82);
    assert_eq!(intervals_compared, 3 * 82);
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn games_howell_agrees_with_r_on_hours_by_industry() {
    let r = games_howell_reference();
    let case = &r["wages"];
    assert_eq!(case["groups"].as_u64(), Some(12));
    let frame = wages(case["x"].as_str().unwrap(), case["g"].as_str().unwrap());
    let (problems, p_compared, intervals_compared) =
        check_games_howell("wages", case, &frame, "hours", "industry", "");
    assert_eq!((p_compared, intervals_compared), (66, 198));
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_games_howell_kernel_follows_its_closed_form_and_what_it_cannot_compute_is_refused() {
    let groups = summaries(&[
        vec![2.9, 3.0, 2.5, 2.6, 3.2],
        vec![3.8, 2.7, 4.0, 2.4],
        vec![2.8, 3.4, 3.7, 2.2, 2.0, 5.5],
    ]);
    // Whichever F the `anova` computed, the comparison is the same.
    let result = compare(&groups, true, GAMES_HOWELL, 0.95).unwrap();
    assert_eq!(result, compare(&groups, false, GAMES_HOWELL, 0.95).unwrap());
    assert_eq!(result.pooled_df, None);
    assert_eq!(result.rows.len(), 3);
    let welch = compare(&groups, true, NONE, 0.95).unwrap();
    for (row, pair) in result.rows.iter().zip(&welch.rows) {
        let (a, b) = (groups[row.first], groups[row.second]);
        let (u, v) = (a.variance / a.n, b.variance / b.n);
        let df = (u + v) * (u + v) / (u * u / (a.n - 1.0) + v * v / (b.n - 1.0));
        assert!((row.difference - (a.mean - b.mean)).abs() < 1e-15);
        assert!((row.std_error - (u + v).sqrt()).abs() < 1e-15);
        assert!((row.df - df).abs() < 1e-12);
        // The standard error, t and the degrees of freedom are Welch's test of the pair.
        assert_eq!(row.std_error.to_bits(), pair.std_error.to_bits());
        assert_eq!(row.t.to_bits(), pair.t.to_bits());
        assert_eq!(row.df.to_bits(), pair.df.to_bits());
        // The p-value and the interval are the range's, for three groups, on those degrees
        // of freedom.
        let p = families::cdf(&range(3, row.df), row.t.abs() * SQRT_2).upper;
        assert!((row.p - p).abs() < 1e-15);
        let q = families::quantile(&range(3, row.df), 0.95) / SQRT_2;
        let (lo, hi) = row.ci.unwrap();
        assert!((lo - (row.difference - q * row.std_error)).abs() < 1e-12);
        assert!((hi - (row.difference + q * row.std_error)).abs() < 1e-12);
        // Of three means the range asks more than the t-test of the pair alone, and no more
        // than Bonferroni's three times its p-value.
        assert!(row.p > pair.p, "{} against {}", row.p, pair.p);
        assert!(row.p <= (3.0 * pair.p).min(1.0));
        let (welch_lo, welch_hi) = pair.ci.unwrap();
        assert!(lo < welch_lo && welch_hi < hi);
    }
    // Groups of one size and one variance: the standard errors are Tukey's, the degrees of
    // freedom 2(n − 1) for each pair against Tukey's k(n − 1), so the p-values are larger and
    // the intervals wider.
    let equal = summaries(&[
        vec![1.0, 2.0, 3.0, 4.0],
        vec![2.5, 3.5, 4.5, 5.5],
        vec![0.0, 1.0, 2.0, 3.0],
    ]);
    let ours = compare(&equal, true, GAMES_HOWELL, 0.95).unwrap();
    let tukey = compare(&equal, true, Method::Tukey, 0.95).unwrap();
    for (row, pooled) in ours.rows.iter().zip(&tukey.rows) {
        assert!((row.std_error - pooled.std_error).abs() < 1e-14);
        assert!((row.t - pooled.t).abs() < 1e-12);
        assert!((row.df - 6.0).abs() < 1e-12);
        assert_eq!(pooled.df, 9.0);
        assert!(row.p > pooled.p);
        assert!(row.ci.unwrap().0 < pooled.ci.unwrap().0);
    }
    // A group of one value, or of equal values, has no variance of its own: refused whichever
    // `anova` came before, where Tukey's on the pooled variance goes through.
    let with_one = summaries(&[vec![1.5, 2.5], vec![3.5, 4.5], vec![9.0]]);
    for welch in [true, false] {
        assert_eq!(
            compare(&with_one, welch, GAMES_HOWELL, 0.95),
            Err(Undefined::GroupOfOne(2))
        );
    }
    assert!(compare(&with_one, false, Method::Tukey, 0.95).is_ok());
    let flat = summaries(&[vec![1.5, 2.5], vec![4.0, 4.0, 4.0], vec![9.0, 7.0]]);
    for welch in [true, false] {
        assert_eq!(
            compare(&flat, welch, GAMES_HOWELL, 0.95),
            Err(Undefined::GroupWithoutSpread(1))
        );
    }
    assert_eq!(
        compare(&groups[..1], true, GAMES_HOWELL, 0.95),
        Err(Undefined::TooFewGroups)
    );
    // The word, the name and what the method is.
    assert_eq!(Method::from_word("games_howell"), Some(GAMES_HOWELL));
    assert_eq!(GAMES_HOWELL.name(), "Games–Howell");
    assert!(GAMES_HOWELL.has_intervals() && GAMES_HOWELL.own_variances() && !GAMES_HOWELL.pools());
    assert!(!Method::Tukey.own_variances() && !HOLM.own_variances());
    assert_eq!(Method::WORDS.len(), 11);
    assert_eq!(Method::WORDS[1], "games_howell");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// On any groups of two values or more with some spread: every p-value lies in [0, 1], is
    /// no smaller than that of Welch's test of the pair and no larger than Bonferroni's over
    /// the pairs; an interval at 95% leaves out zero exactly when the p-value is below 0.05;
    /// a higher level gives a wider interval round the same difference; and the answer does
    /// not depend on the order of the rows, nor on the form of the `anova` it follows.
    #[test]
    fn games_howell_keeps_its_bounds_and_its_intervals_agree_with_its_p_values(
        groups in prop::collection::vec(
            prop::collection::vec(-50i32..50, 2..9),
            2..6,
        ),
        scales in prop::collection::vec(1u32..40, 6),
    ) {
        // Values on a grid, each group on its own scale: variances that differ a great deal.
        let values: Vec<Vec<f64>> = groups
            .iter()
            .enumerate()
            .map(|(i, g)| g.iter().map(|v| f64::from(*v) * f64::from(scales[i]) / 8.0).collect())
            .collect();
        prop_assume!(values.iter().all(|g| g.iter().any(|v| *v != g[0])));
        let k = values.len();
        let m = (k * (k - 1) / 2) as f64;
        let s = summaries(&values);
        let ours = compare(&s, true, GAMES_HOWELL, 0.95).unwrap();
        let welch = compare(&s, true, NONE, 0.95).unwrap();
        let wider = compare(&s, false, GAMES_HOWELL, 0.99).unwrap();
        for ((row, pair), wide) in ours.rows.iter().zip(&welch.rows).zip(&wider.rows) {
            prop_assert!((0.0..=1.0).contains(&row.p), "{}", row.p);
            prop_assert!(row.p >= pair.p - 1e-9, "{} below Welch's {}", row.p, pair.p);
            prop_assert!(row.p <= (m * pair.p).min(1.0) + 1e-9, "{} above Bonferroni's", row.p);
            prop_assert!(row.df >= 1.0 - 1e-9);
            let (lo, hi) = row.ci.unwrap();
            // Away from the edge, where the two could differ by rounding.
            if (row.p - 0.05).abs() > 1e-7 {
                prop_assert_eq!(lo > 0.0 || hi < 0.0, row.p < 0.05, "p = {}, [{}, {}]", row.p, lo, hi);
            }
            prop_assert_eq!(wide.p.to_bits(), row.p.to_bits());
            let (wide_lo, wide_hi) = wide.ci.unwrap();
            prop_assert!(wide_lo < lo && hi < wide_hi);
        }
        // The groups in the reverse order: each pair has its figures, with the sign turned.
        let reversed: Vec<GroupSummary> = s.iter().rev().copied().collect();
        let back = compare(&reversed, true, GAMES_HOWELL, 0.95).unwrap();
        for row in &ours.rows {
            let same = back
                .rows
                .iter()
                .find(|r| r.first == k - 1 - row.second && r.second == k - 1 - row.first)
                .unwrap();
            prop_assert!((same.difference + row.difference).abs() <= 1e-12 * (1.0 + row.difference.abs()));
            prop_assert!((same.p - row.p).abs() <= 1e-12);
            prop_assert!((same.df - row.df).abs() <= 1e-9 * row.df);
        }
    }
}

/// Every form of the Games–Howell comparison through a script, and every message.
#[test]
fn games_howell_in_every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\nlet w = g.anova(s, group: site)\nw.posthoc(method: games_howell)\nw.posthoc()\nw.posthoc(method: games_howell, level: 0.99)\nlet a = g.anova(s, group: site, equal_var: true)\na.posthoc(method: games_howell)\ng.where(site < 3).anova(s, group: site).posthoc(method: games_howell)\ng.where(site < 3).t_test(s, group: site)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.anova(v, group: k, weights: w, weight_type: frequency).posthoc(method: games_howell)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 3.5, 1.0], g = [1, 1, 2, 2, NA, 3, 3, 2, 3])\nm.anova(x, group: g).posthoc(method: games_howell)\nlet wages = dataset(wages)\nwages.anova(hours, group: industry).posthoc(method: games_howell)\nlet p = w.posthoc(method: games_howell)\np.estimates()\np.ci()\np.statistic()\np.df()\np.p_values()\np.p_values(\"site=1 - site=2\")\np.df(\"site=1 - site=2\")\np.summary()\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("games_howell_forms", format!("{stdout}---\n{stderr}"));
    // The table has the degrees of freedom of each pair, and the lines under it say what the
    // comparison rests on.
    for said in [
        "posthoc · Games–Howell · s by site\n",
        "t value    df   p value   95% CI\n",
        "p value   99% CI\n",
        "Games–Howell over 3 comparisons of 3 groups: p-values and intervals from the studentized range on each pair's own degrees of freedom, family-wise\n",
        "each pair on its two groups' own variances and Welch's degrees of freedom; N = 14\n",
        "Games–Howell over 66 comparisons of 12 groups",
    ] {
        assert!(stdout.contains(said), "{said}\n{stdout}");
    }
    // After Welch's `anova`, Tukey's note names the comparison that does not pool; after the
    // classical form Games–Howell says what the `anova` assumed; the only other notes are of
    // the rows dropped for `NA`.
    let notes: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        notes[..2],
        [
            "note: `tukey` pools the variances of the groups, which the `anova` did not assume equal; `method: games_howell` compares the pairs without that assumption (line 4)",
            "note: `games_howell` takes each group's own variance, where the `anova` assumed them equal; `method: tukey` compares the pairs on the pooled variance (line 7)",
        ]
    );
    assert_eq!(notes.len(), 4, "{stderr}");
    assert!(
        notes[2..].iter().all(|note| note.contains("dropped")),
        "{stderr}"
    );
    let mut all = String::new();
    for script in [
        // After a formula the means compared have one residual variance.
        "let d = frame(y = [1, 2, 4, 3, 5.5, 7, 6, 8], a = [1, 1, 2, 2, 1, 1, 2, 2])\nd.anova(y ~ category(a)).posthoc(method: games_howell)\n",
        "let d = frame(y = [1, 2, 4, 3, 5.5, 7, 6, 8], a = [1, 1, 2, 2, 1, 1, 2, 2], b = [1, 2, 1, 2, 1, 2, 1, 2])\nd.anova(y ~ category(a) * category(b)).posthoc(method: games_howell, term: a)\n",
        // After the classical form: a group of one value, and a group of equal values.
        "let d = frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3])\nd.anova(x, group: g, equal_var: true).posthoc(method: games_howell)\n",
        "let d = frame(x = [1.5, 2.5, 4.0, 4.0, 4.0, 9.0, 7.0], g = [\"a\", \"a\", \"b\", \"b\", \"b\", \"c\", \"c\"])\nd.anova(x, group: g, equal_var: true).posthoc(method: games_howell)\n",
        // After Welch's form such a group stops the `anova` itself.
        "let d = frame(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3])\nd.anova(x, group: g).posthoc(method: games_howell)\n",
        // A word near it.
        "let d = frame(x = [1, 2, 4, 3, 5.5, 7], g = [1, 1, 1, 2, 2, 2])\nd.anova(x, group: g).posthoc(method: games)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("games_howell_messages", all);
}
