//! `p_adjust` given a column (release 0.7.0, step 18c; open decision #111): inside `add` it gives
//! a column of the same length, and under `by:` each group is a family adjusted on its own. The
//! figures within the groups are R's (`tests/reference/r/kruskal.R`, the block `grouped`), read
//! through the engine so that the grouping tested is the one a script meets; the properties are
//! that the column form equals the list form, that each group equals the list form on that group
//! alone, and that the order of the rows changes no row's value.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::Column;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_stats::p_adjust::{Method, adjust, p_adjust};
use zeff_testkit::{root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// `p_adjust(p, method:)` called as the engine calls it, on a value.
fn call(p: Value, method: Option<&str>) -> Result<Value, Fail> {
    let args = Args {
        values: vec![Some(p), method.map(|m| Value::Word(m.to_string()))],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "p_adjust(p)",
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(0, 8),
        arg_spans: vec![Some(Span::new(9, 10)), None],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    p_adjust(&mut ctx, args)
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn number(v: &Json) -> Option<f64> {
    match v.as_str().unwrap() {
        "NA" => None,
        text => Some(text.parse().unwrap()),
    }
}

fn floats(v: &Json) -> Vec<Option<f64>> {
    v.as_array().unwrap().iter().map(number).collect()
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/kruskal.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// A list of numbers as a script writes it, each with every digit it needs.
fn written(values: &[Option<f64>]) -> String {
    let items: Vec<String> = values
        .iter()
        .map(|v| v.map_or("NA".to_string(), |x| format!("{x:?}")))
        .collect();
    format!("[{}]", items.join(", "))
}

/// The statement that counts the rows of `frame` where `got` is not `want`: a value against a
/// value within a relative 1e-12, `NA` against `NA` only.
fn count_of_differences(frame: &str, got: &str, want: &str) -> String {
    format!(
        "{frame}.add(bad = coalesce(abs({got} - {want}) > 1e-12 * {want}, is_missing({got}) != is_missing({want}))).where(bad).n_rows()\n"
    )
}

// ----- against R -----

#[test]
fn within_every_group_the_eight_adjustments_agree_with_r() {
    let r = reference();
    let cases = r["grouped"].as_array().unwrap();
    assert_eq!(cases.len(), 7, "the grouped cases of kruskal.R");
    for (i, case) in cases.iter().enumerate() {
        let g: Vec<String> = case["g"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| match v.as_str().unwrap() {
                "NA" => "NA".to_string(),
                text if case["text_groups"].as_bool().unwrap() => format!("\"{text}\""),
                text => text.to_string(),
            })
            .collect();
        let mut script = format!(
            "let t = frame(g = [{}], p = {}",
            g.join(", "),
            written(&floats(&case["p"]))
        );
        for method in Method::WORDS {
            script.push_str(&format!(
                ", want_{method} = {}",
                written(&floats(&case[method]))
            ));
        }
        script.push_str(")\n");
        for method in Method::WORDS {
            script.push_str(&format!(
                "let u_{method} = t.add(got = p_adjust(p, method: {method}), by: g)\n"
            ));
            script.push_str(&count_of_differences(
                &format!("u_{method}"),
                "got",
                &format!("want_{method}"),
            ));
        }
        let (exit, stdout, stderr) = go(&script);
        assert_eq!(exit, 0, "grouped[{i}]: {stderr}\n{script}");
        assert_eq!(stdout, "0\n".repeat(8), "grouped[{i}]: {stdout}\n{script}");
    }
}

#[test]
fn the_count_of_differences_sees_a_wrong_value_and_a_misplaced_na() {
    // The statement the comparisons rest on must be able to fail: one value out in the twelfth
    // digit, one `NA` where a value is due and one value where `NA` is due.
    let script = format!(
        "let t = frame(got = [0.5, 0.25, NA, 0.125, 0.3], want = [0.5, 0.25000000001, 0.1, NA, 0.3])\n{}",
        count_of_differences("t", "got", "want")
    );
    let (exit, stdout, stderr) = go(&script);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "3\n");
}

// ----- the column form and the list form -----

fn list(p: &[Option<f64>]) -> Value {
    Value::List(
        p.iter()
            .map(|v| v.map_or(Value::Na, Value::Float))
            .collect(),
    )
}

fn column_values(v: Value) -> Vec<Option<f64>> {
    match v {
        Value::Column(c) => c.as_floats().expect("a Float column").collect(),
        other => panic!("not a column: {other:?}"),
    }
}

fn list_values(v: Value) -> Vec<Option<f64>> {
    match v {
        Value::List(items) => items
            .iter()
            .map(|item| match item {
                Value::Float(x) => Some(*x),
                Value::Na => None,
                other => panic!("not a p-value: {other:?}"),
            })
            .collect(),
        other => panic!("not a list: {other:?}"),
    }
}

#[test]
fn a_column_gives_a_column_with_the_values_of_the_list_form() {
    let r = reference();
    for (i, case) in r["adjust"].as_array().unwrap().iter().enumerate() {
        let p = floats(&case["p"]);
        for method in Method::WORDS {
            let from_list = list_values(call(list(&p), Some(method)).unwrap());
            let from_column = column_values(
                call(Value::Column(Column::floats(p.clone()).0), Some(method)).unwrap(),
            );
            assert_eq!(from_column, from_list, "adjust[{i}] {method}");
            assert_eq!(from_column.len(), p.len());
        }
    }
}

#[test]
fn a_column_of_ints_or_of_bools_is_read_as_zeros_and_ones() {
    let want = list_values(call(list(&[Some(1.0), Some(0.0), None, Some(1.0)]), None).unwrap());
    let ints = Column::ints([Some(1), Some(0), None, Some(1)]);
    assert_eq!(
        column_values(call(Value::Column(ints), None).unwrap()),
        want
    );
    let bools = Column::bools([Some(true), Some(false), None, Some(true)]);
    assert_eq!(
        column_values(call(Value::Column(bools), None).unwrap()),
        want
    );
}

#[test]
fn a_column_of_no_rows_gives_a_column_of_no_rows() {
    let empty = Column::floats(Vec::<Option<f64>>::new()).0;
    assert!(column_values(call(Value::Column(empty), None).unwrap()).is_empty());
}

// ----- the properties, through the engine -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A p-value: `NA`, 0, 1, one of a few values that tie, or any number between 0 and 1.
fn p_value() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        2 => Just(None),
        1 => Just(Some(0.0)),
        1 => Just(Some(1.0)),
        3 => prop::sample::select(vec![0.01, 0.02, 0.04, 0.5]).prop_map(Some),
        6 => (0.0f64..=1.0).prop_map(Some),
        2 => (0.0f64..=1.0).prop_map(|x| Some(x * 1e-9)),
    ]
}

/// Rows of a group code, `NA` among them, and a p-value; the first row has both, so that the
/// frame's columns have a type.
fn rows() -> impl Strategy<Value = Vec<(Option<i64>, Option<f64>)>> {
    (
        (0i64..4, 0.0f64..=1.0),
        prop::collection::vec(
            (
                prop_oneof![1 => Just(None), 9 => (0i64..4).prop_map(Some)],
                p_value(),
            ),
            0..24,
        ),
    )
        .prop_map(|((g, p), rest)| {
            let mut rows = vec![(Some(g), Some(p))];
            rows.extend(rest);
            rows
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// Under `by:` every group has the values of the list form on that group alone, and without
    /// `by:` the column has the values of the list form on the whole column. The rows come in
    /// any order, so the order of the rows changes no row's value.
    #[test]
    fn each_group_is_adjusted_as_a_list_of_its_own(
        rows in rows(),
        method in prop::sample::select(Method::WORDS.to_vec()),
    ) {
        let chosen = Method::from_word(method).unwrap();
        let p: Vec<Option<f64>> = rows.iter().map(|(_, p)| *p).collect();
        let whole = adjust(&p, chosen);
        let mut within: Vec<Option<f64>> = vec![None; rows.len()];
        let mut keys: Vec<Option<i64>> = rows.iter().map(|(g, _)| *g).collect();
        keys.sort();
        keys.dedup();
        for key in keys {
            let members: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].0 == key).collect();
            let family: Vec<Option<f64>> = members.iter().map(|&i| p[i]).collect();
            for (&row, value) in members.iter().zip(adjust(&family, chosen)) {
                within[row] = value;
            }
        }
        let g: Vec<String> = rows
            .iter()
            .map(|(g, _)| g.map_or("NA".to_string(), |g| g.to_string()))
            .collect();
        let script = format!(
            "let t = frame(g = [{}], p = {}, whole = {}, within = {})\nlet u = t.add(a = p_adjust(p, method: {method})).add(b = p_adjust(p, method: {method}), by: g)\n{}{}",
            g.join(", "),
            written(&p),
            written(&whole),
            written(&within),
            count_of_differences("u", "a", "whole"),
            count_of_differences("u", "b", "within"),
        );
        let (exit, stdout, stderr) = go(&script);
        prop_assert_eq!(exit, 0, "{}\n{}", stderr, script);
        prop_assert_eq!(stdout, "0\n0\n", "{}", script);
    }
}

// ----- the snapshots -----

const T: &str = "let t = frame(outcome = [\"wage\", \"wage\", \"wage\", \"hours\", \"hours\", \"hours\", \"hours\"], arm = [1, 2, 3, 1, 2, 3, 4], p = [0.01, 0.04, 0.03, 0.2, NA, 0.001, 0.02])\n";

#[test]
fn every_form_of_the_column() {
    let mut all = String::new();
    for script in [
        // The statement of open decision #111, and several methods side by side.
        "t.add(p_holm = p_adjust(p))\n",
        "t.add(holm = p_adjust(p), bh = p_adjust(p, method: bh), none = p_adjust(p, method: none))\n",
        // Each outcome a family of its own; the method `by` beside the option `by:`.
        "t.add(p_holm = p_adjust(p), by: outcome)\n",
        "t.add(p_by = p_adjust(p, method: by), by: outcome)\n",
        // A family of one p-value, a family of nothing but `NA`, a missing group value.
        "frame(g = [1, 2, 2, 3, 3, NA, NA], p = [0.03, 0.2, 0.004, NA, NA, 0.04, 0.3]).add(h = p_adjust(p), by: g)\n",
        // After `where` the family is the rows kept.
        "t.where(arm < 3).add(p_holm = p_adjust(p))\n",
        "t.where(arm < 3).add(p_holm = p_adjust(p), by: outcome)\n",
        // In `update`, in a condition, and from a column taken out of the frame.
        "t.update(p = p_adjust(p, method: bonferroni), by: outcome)\n",
        "t.add(kept = p_adjust(p) < 0.05, by: outcome)\n",
        "p_adjust(t[\"p\"], method: hochberg)\n",
        // Whole numbers and truth values are read as 0 and 1.
        "frame(p = [1, 0, 1]).add(h = p_adjust(p))\n",
        "frame(p = [true, false]).add(h = p_adjust(p))\n",
        // A frame of no rows.
        "t.where(arm > 9).add(h = p_adjust(p), by: outcome)\n",
        // A list still gives a list.
        "p_adjust([0.01, 0.04, 0.03])\n",
    ] {
        let (exit, stdout, stderr) = go(&format!("{T}{script}"));
        assert_eq!(exit, 0, "{script}{stderr}");
        all.push_str(&format!("> {script}{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("column_forms", all);
}

#[test]
fn every_message_of_the_column() {
    let mut all = String::new();
    for script in [
        // A value that is not a p-value is named by its row of the frame: on the whole frame,
        // inside a group, after `where`, and for an expression.
        "frame(g = [\"a\", \"b\", \"a\", \"b\", \"b\"], p = [0.01, 0.2, 0.5, 0.3, 1.2]).add(h = p_adjust(p))\n",
        "frame(g = [\"a\", \"b\", \"a\", \"b\", \"b\"], p = [0.01, 0.2, 0.5, 0.3, 1.2]).add(h = p_adjust(p), by: g)\n",
        "frame(g = [\"a\", \"b\", \"a\", \"b\", \"b\"], p = [0.01, 0.2, -0.5, 0.3, 0.02]).add(h = p_adjust(p), by: g)\n",
        "frame(g = [\"a\", \"b\", \"a\", \"b\", \"b\"], p = [0.01, 0.2, 0.5, 0.3, 1.2]).where(p > 0.1).add(h = p_adjust(p))\n",
        "t.add(h = p_adjust(p * 30), by: outcome)\n",
        // A column that is not numbers.
        "t.add(h = p_adjust(outcome))\n",
        // `by:` belongs to the verb, not to `p_adjust`.
        "t.add(h = p_adjust(p, by: outcome))\n",
        // A word that is no method; a list inside `add`; one value per group asked of a column.
        "t.add(h = p_adjust(p, method: tukey), by: outcome)\n",
        "t.add(h = p_adjust([0.1, 0.2]))\n",
        "t.aggregate(h = p_adjust(p), by: outcome)\n",
        // Neither a list nor a column.
        "p_adjust(t)\n",
        "p_adjust(0.04)\n",
    ] {
        let (exit, stdout, stderr) = go(&format!("{T}{script}"));
        assert_eq!(exit, 1, "{script}{stdout}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("column_messages", all);
}

#[test]
fn a_row_refused_inside_a_group_is_counted_in_the_frame() {
    // Row 5 of the frame is the third row of group "b": the message names 5.
    let (exit, _, stderr) = go(
        "frame(g = [\"a\", \"b\", \"a\", \"b\", \"b\"], p = [0.01, 0.2, 0.5, 0.3, 1.2]).add(h = p_adjust(p), by: g)\n",
    );
    assert_eq!(exit, 1);
    assert!(stderr.contains("`p` is 1.2 in row 5"), "{stderr}");
}
