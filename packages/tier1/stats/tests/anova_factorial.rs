//! `anova`, the factorial form (release 0.7.0, step 17; Language Reference, sections 4, 6, 7.2
//! and 10.2; open decisions #91 and #98): the reference (`tests/reference/r/anova.json`, key
//! `factorial`: R 4.6.0's `anova(lm())` for type 1 and car 3.1-5's `Anova()` for types 2 and 3,
//! on balanced and unbalanced layouts, the terms in both orders, a covariate alone and in an
//! interaction, three factors, an empty cell, rows with `NA`, frequency and analytic weights)
//! within a relative 1e-9, figure by figure and again through the function; the properties (the
//! three types agree on a balanced layout; the type 1 sums of squares add up to the model's;
//! types 2 and 3 do not depend on the order of the terms; frequency weights repeat rows); the
//! edge cases (every refusal and every note); and snapshots of every form and every message.
//!
//! `effect:` (step 17b; open decision #99, question 2): partial omega squared and partial epsilon
//! squared of every row of the same reference, where they are written out on the three tables
//! and the whole model's epsilon squared is checked against `summary(lm)$adj.r.squared`; the
//! properties (partial epsilon squared never exceeds partial eta squared, partial omega squared
//! lies between 0 and partial epsilon squared, both are negative exactly when the row's F is
//! below 1, frequency weights repeat rows); and the edge cases (one category gives the measures
//! of the classical one-way form; the note that names the rows whose estimate is negative).

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table as Columns};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_numeric::order::WeightKind;
use zeff_results::design::Design;
use zeff_stats::anova::anova;
use zeff_stats::anova_factorial::{Factorial, Kind, Undefined, table_of};
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The slots of `anova`: data, var, group, equal_var, weights, weight_type, type, effect.
const SLOTS: usize = 8;
const WEIGHTS: usize = 4;
const WEIGHT_TYPE: usize = 5;
const TYPE: usize = 6;
const EFFECT: usize = 7;

fn with_context<T>(
    text: &str,
    body: impl FnOnce(&mut CallContext<'_>) -> Result<T, Fail>,
) -> Result<T, Fail> {
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
        arg_spans: vec![None; SLOTS],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    body(&mut ctx)
}

/// The design of `text` on `frame`, its categories coded by deviations, as `anova` builds it.
fn design(frame: &Frame, text: &str, weights: Option<(&str, WeightKind)>) -> Design {
    let model = formula(text);
    let built = with_context(text, |ctx| {
        let extra: Vec<&str> = weights.iter().map(|(name, _)| *name).collect();
        let mut design = Design::build(ctx, frame, &model, &extra)?;
        if let Some((name, kind)) = weights {
            design = design.weighted(ctx, frame, name, kind)?;
        }
        Ok(design.sum_coded().0)
    });
    match built {
        Ok(design) => design,
        Err(_) => panic!("the design of `{text}` builds"),
    }
}

/// `anova` called with a formula and the options given by slot.
fn call(frame: &Frame, text: &str, slots: Vec<(usize, Value)>) -> Result<TestResult, Fail> {
    let mut values: Vec<Option<Value>> = vec![None; SLOTS];
    values[0] = Some(Value::Frame(frame.clone()));
    values[1] = Some(Value::Formula(formula(text)));
    for (slot, value) in slots {
        values[slot] = Some(value);
    }
    let args = Args {
        values,
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    with_context(text, |ctx| anova(ctx, args)).map(|v| match v {
        Value::TestResult(r) => (*r).clone(),
        other => panic!("not a TestResult: {other:?}"),
    })
}

/// One row of a result read by its name, as a script reads it: `statistic(term)`, `df(term)`,
/// `p_values(term)` and `effect_size(term)`.
fn by_name(result: &TestResult, term: &str) -> (f64, Vec<f64>, f64, f64) {
    use zeff_results::interface::{df, effect_size, p_values, statistic};
    let read = |body: fn(&mut CallContext<'_>, Args) -> Result<Value, Fail>| {
        let args = Args {
            values: vec![
                Some(Value::TestResult(std::rc::Rc::new(result.clone()))),
                Some(Value::Term(term.to_string())),
            ],
            rest: Vec::new(),
            definitions: Vec::new(),
        };
        match with_context("t.statistic(term)", |ctx| body(ctx, args)) {
            Ok(value) => value,
            Err(_) => panic!("the row `{term}` is not read"),
        }
    };
    let float = |value: Value| match value {
        Value::Float(x) => x,
        other => panic!("`{term}`: not a number: {other:?}"),
    };
    let dfs = match read(df) {
        Value::List(items) => items.into_iter().map(float).collect(),
        other => panic!("`{term}`: not a list: {other:?}"),
    };
    (
        float(read(statistic)),
        dfs,
        float(read(p_values)),
        float(read(effect_size)),
    )
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

/// A category column of `levels`, each row by the position of its level.
fn category(levels: &[&str], codes: impl Iterator<Item = Option<i32>>) -> Column {
    Column::categories(levels.iter().map(|l| (*l).to_string()).collect(), codes).unwrap()
}

fn number(v: &Json) -> Option<f64> {
    match v {
        Json::String(s) => match s.as_str() {
            "NA" => None,
            s => Some(s.parse().unwrap()),
        },
        other => Some(other.as_f64().unwrap()),
    }
}

fn text_of(v: &Json) -> Option<String> {
    match v {
        Json::String(s) if s == "NA" => None,
        Json::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/anova.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(1e-300)
}

/// The frame of a reference case: the factors as categories whose levels are the values seen,
/// in alphabetical order as R's `factor`; frequency weights as whole numbers; the rest as
/// numbers.
fn case_frame(case: &Json) -> Frame {
    let factors: Vec<&str> = case["factors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let frequency = case["weight_type"].as_str() == Some("frequency");
    let weights = text_of(&case["weights"]);
    let columns = case["columns"].as_object().unwrap();
    frame_of(
        columns
            .iter()
            .map(|(name, values)| {
                let values = values.as_array().unwrap();
                let column = if factors.contains(&name.as_str()) {
                    let cells: Vec<Option<String>> = values.iter().map(text_of).collect();
                    let mut levels: Vec<&str> =
                        cells.iter().flatten().map(String::as_str).collect();
                    levels.sort_unstable();
                    levels.dedup();
                    category(
                        &levels,
                        cells.iter().map(|cell| {
                            cell.as_ref()
                                .map(|c| levels.iter().position(|l| l == c).unwrap() as i32)
                        }),
                    )
                } else if frequency && weights.as_deref() == Some(name.as_str()) {
                    Column::ints(values.iter().map(|v| number(v).map(|w| w as i64)))
                } else {
                    Column::floats(values.iter().map(number)).0
                };
                (name.as_str(), column)
            })
            .collect(),
    )
}

/// The weights of a reference case, as the design and as the function take them.
fn case_weights(case: &Json) -> Option<(String, WeightKind)> {
    let name = text_of(&case["weights"])?;
    let kind = match case["weight_type"].as_str().unwrap() {
        "frequency" => WeightKind::Frequency,
        "analytic" => WeightKind::Analytic,
        other => panic!("no weight type {other}"),
    };
    Some((name, kind))
}

const KINDS: [(Kind, &str, i64); 3] = [
    (Kind::Sequential, "type1", 1),
    (Kind::Marginal, "type2", 2),
    (Kind::Partial, "type3", 3),
];

// ----- the reference -----

#[test]
fn the_three_types_agree_with_anova_lm_and_with_car() {
    let reference = reference();
    let cases = reference["factorial"].as_array().unwrap();
    assert!(cases.len() >= 15, "the reference has its cases");
    let mut refused = 0;
    for case in cases {
        let text = case["formula"].as_str().unwrap();
        let frame = case_frame(case);
        let weights = case_weights(case);
        let built = design(
            &frame,
            text,
            weights.as_ref().map(|(n, k)| (n.as_str(), *k)),
        );
        let residual_df = number(&case["residual_df"]).unwrap();
        let residual_ss = number(&case["residual_ss"]).unwrap();
        let total_ss = number(&case["total_ss"]).unwrap();
        let n = number(&case["n"]).unwrap();
        for (kind, key, type_number) in KINDS {
            let what = format!("{text}, {key}, weights {:?}", case["weights"]);
            let mut slots = vec![(TYPE, Value::Int(type_number))];
            if let Some((name, weight_kind)) = &weights {
                slots.push((WEIGHTS, Value::Var(name.clone())));
                let word = match weight_kind {
                    WeightKind::Frequency => "frequency",
                    _ => "analytic",
                };
                slots.push((WEIGHT_TYPE, Value::Word(word.to_string())));
            }
            let Some(rows) = case[key].as_array() else {
                // car refuses type 3 where a cell is empty, and so does Zeff, by both routes.
                assert_eq!(case[key].as_str(), Some("NA"), "{what}");
                assert!(
                    matches!(table_of(&built, kind), Err(Undefined::Aliased { .. })),
                    "{what}"
                );
                assert!(call(&frame, text, slots).is_err(), "{what}");
                refused += 1;
                continue;
            };
            // Figure by figure, before any rounding.
            let table: Factorial = match table_of(&built, kind) {
                Ok(table) => table,
                Err(why) => panic!("{what}: {why:?}"),
            };
            assert_eq!(table.terms.len(), rows.len(), "{what}");
            assert!(close(table.residual_df, residual_df, 1e-12), "{what}");
            assert!(
                close(table.residual_ss, residual_ss, 1e-9),
                "{what}: residual SS {} against {residual_ss}",
                table.residual_ss
            );
            assert!(
                close(table.total_ss, total_ss, 1e-9),
                "{what}: total SS {} against {total_ss}",
                table.total_ss
            );
            assert_eq!(
                table.model.df as f64,
                number(&case["model_df"]).unwrap(),
                "{what}"
            );
            assert!(
                close(table.model.f, number(&case["model_f"]).unwrap(), 1e-9),
                "{what}: model F {}",
                table.model.f
            );
            assert!(
                close(table.model.p, number(&case["model_p"]).unwrap(), 1e-9),
                "{what}: model p {}",
                table.model.p
            );
            assert!(
                close(
                    table.model.ss / table.total_ss,
                    number(&case["r_squared"]).unwrap(),
                    1e-9
                ),
                "{what}: R squared"
            );
            for (got, key, name) in [
                (table.model.partial_omega_squared, "model_omega", "omega"),
                (
                    table.model.partial_omega_squared_f,
                    "model_omega_f",
                    "omega from F",
                ),
                (
                    table.model.partial_epsilon_squared,
                    "model_epsilon",
                    "epsilon",
                ),
            ] {
                let want = number(&case[key]).unwrap();
                assert!(
                    close(got, want, 1e-9),
                    "{what}: the model's {name} squared {got} against {want}"
                );
            }
            for (row, want) in table.terms.iter().zip(rows) {
                let term = want["term"].as_str().unwrap();
                assert_eq!(row.label, term, "{what}");
                assert_eq!(
                    row.df as f64,
                    number(&want["df"]).unwrap(),
                    "{what}: {term}"
                );
                let ss = number(&want["ss"]).unwrap();
                assert!(
                    close(row.ss, ss, 1e-9),
                    "{what}: {term}: SS {} against {ss}",
                    row.ss
                );
                let f = number(&want["f"]).unwrap();
                assert!(
                    close(row.f, f, 1e-9),
                    "{what}: {term}: F {} against {f}",
                    row.f
                );
                let p = number(&want["p"]).unwrap();
                assert!(
                    close(row.p, p, 1e-9),
                    "{what}: {term}: p {} against {p}",
                    row.p
                );
                assert!(
                    close(row.partial_eta_squared, ss / (ss + residual_ss), 1e-9),
                    "{what}: {term}: partial eta squared"
                );
                let omega = number(&want["omega"]).unwrap();
                assert!(
                    close(row.partial_omega_squared, omega, 1e-9),
                    "{what}: {term}: partial omega squared {} against {omega}",
                    row.partial_omega_squared
                );
                // The figure from F alone, as Stata's `estat esize, omega` and R's
                // `F_to_omega2` give it (open decision #101; step 38e).
                let omega_f = number(&want["omega_f"]).unwrap();
                assert!(
                    close(row.partial_omega_squared_f, omega_f, 1e-9),
                    "{what}: {term}: partial omega squared from F {} against {omega_f}",
                    row.partial_omega_squared_f
                );
                let epsilon = number(&want["epsilon"]).unwrap();
                assert!(
                    close(row.partial_epsilon_squared, epsilon, 1e-9),
                    "{what}: {term}: partial epsilon squared {} against {epsilon}",
                    row.partial_epsilon_squared
                );
            }
            // The same figures through the function, as a script reads them.
            // Under `effect:` the rows carry the measure asked for, by its name.
            for word in ["omega", "omega_f", "epsilon"] {
                let mut asked = slots.clone();
                asked.push((EFFECT, Value::Word(word.to_string())));
                let result = match call(&frame, text, asked) {
                    Ok(result) => result,
                    Err(_) => panic!("{what}: the function refuses `effect: {word}`"),
                };
                assert_eq!(result.terms.len(), rows.len(), "{what}");
                for (row, want) in result.terms.iter().zip(rows) {
                    let effect = row.effect.as_ref().unwrap();
                    let named = match word {
                        "omega_f" => "partial omega squared from F".to_string(),
                        word => format!("partial {word} squared"),
                    };
                    assert_eq!(effect.label, named);
                    // Read by its name, the row gives the same figure.
                    assert_eq!(
                        by_name(&result, &row.term).3.to_bits(),
                        effect.value.to_bits()
                    );
                    assert!(
                        close(effect.value, number(&want[word]).unwrap(), 1e-9),
                        "{what}: {}: partial {word} squared {}",
                        row.term,
                        effect.value
                    );
                    assert!(effect.ci.is_none());
                }
            }
            let result = match call(&frame, text, slots) {
                Ok(result) => result,
                Err(_) => panic!("{what}: the function refuses"),
            };
            assert_eq!(result.n as f64, n, "{what}");
            assert_eq!(result.statistic.name, "F");
            assert!(close(
                result.statistic.value,
                number(&case["model_f"]).unwrap(),
                1e-9
            ));
            assert_eq!(result.statistic.df.len(), 2);
            assert!(close(result.statistic.df[1], residual_df, 1e-12), "{what}");
            assert!(close(
                result.p_value,
                number(&case["model_p"]).unwrap(),
                1e-9
            ));
            assert_eq!(result.terms.len(), rows.len(), "{what}");
            // Each row read by its name (open decision #100; step 38d): `statistic`, `df`,
            // `p_values` and `effect_size` of a term are R's F, degrees of freedom and
            // p-value of that term, and its partial eta squared.
            for want in rows {
                let term = want["term"].as_str().unwrap();
                let ss = number(&want["ss"]).unwrap();
                let (f, df, p, effect) = by_name(&result, term);
                assert!(
                    close(f, number(&want["f"]).unwrap(), 1e-9),
                    "{what}: statistic(\"{term}\") {f}"
                );
                assert_eq!(
                    df,
                    vec![number(&want["df"]).unwrap(), table.residual_df],
                    "{what}: df(\"{term}\")"
                );
                assert!(
                    close(p, number(&want["p"]).unwrap(), 1e-9),
                    "{what}: p_values(\"{term}\") {p}"
                );
                assert!(
                    close(effect, ss / (ss + residual_ss), 1e-9),
                    "{what}: effect_size(\"{term}\") {effect}"
                );
            }
            for (row, want) in result.terms.iter().zip(rows) {
                assert_eq!(row.term, want["term"].as_str().unwrap(), "{what}");
                assert!(close(
                    row.statistic.value,
                    number(&want["f"]).unwrap(),
                    1e-9
                ));
                assert_eq!(
                    row.statistic.df,
                    vec![number(&want["df"]).unwrap(), table.residual_df],
                    "{what}"
                );
                assert!(close(row.p_value, number(&want["p"]).unwrap(), 1e-9));
                let ss = number(&want["ss"]).unwrap();
                let effect = row.effect.as_ref().unwrap();
                assert_eq!(effect.label, "partial eta squared");
                assert!(close(effect.value, ss / (ss + residual_ss), 1e-9), "{what}");
            }
        }
    }
    assert_eq!(refused, 1, "one case has an empty cell");
}

// ----- the properties -----

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        * 48
}

const LEVELS: [&str; 4] = ["p", "q", "r", "s"];

/// Rows of an outcome, two category codes, a covariate and a whole weight.
fn rows() -> impl Strategy<Value = Vec<(f64, i32, i32, f64, i64)>> {
    prop::collection::vec(
        (-3.0..3.0f64, 0..3i32, 0..3i32, -2.0..2.0f64, 1..4i64),
        14..40,
    )
}

fn layout(rows: &[(f64, i32, i32, f64, i64)]) -> Frame {
    frame_of(vec![
        ("y", Column::floats(rows.iter().map(|r| Some(r.0))).0),
        ("a", category(&LEVELS, rows.iter().map(|r| Some(r.1)))),
        ("b", category(&LEVELS, rows.iter().map(|r| Some(r.2)))),
        ("x", Column::floats(rows.iter().map(|r| Some(r.3))).0),
        ("w", Column::ints(rows.iter().map(|r| Some(r.4)))),
    ])
}

/// Whether each of the two categories has two levels or more among the rows.
fn two_levels_each(rows: &[(f64, i32, i32, f64, i64)]) -> bool {
    let distinct = |pick: fn(&(f64, i32, i32, f64, i64)) -> i32| {
        let mut seen: Vec<i32> = rows.iter().map(pick).collect();
        seen.sort_unstable();
        seen.dedup();
        seen.len() >= 2
    };
    distinct(|r| r.1) && distinct(|r| r.2)
}

fn by_label<'a>(table: &'a Factorial, label: &str) -> &'a zeff_stats::anova_factorial::Row {
    table
        .terms
        .iter()
        .find(|row| row.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    #[test]
    fn the_three_types_agree_on_a_balanced_layout(
        a_levels in 2..4usize,
        b_levels in 2..4usize,
        replicates in 2..4usize,
        values in prop::collection::vec(-3.0..3.0f64, 48),
    ) {
        let n = a_levels * b_levels * replicates;
        let values = &values[..n];
        prop_assume!(values.iter().any(|v| *v != values[0]));
        let frame = frame_of(vec![
            ("y", Column::floats(values.iter().map(|v| Some(*v))).0),
            ("a", category(&LEVELS, (0..n).map(|i| Some((i % a_levels) as i32)))),
            ("b", category(&LEVELS, (0..n).map(|i| Some(((i / a_levels) % b_levels) as i32)))),
        ]);
        let built = design(&frame, "y ~ a * b", None);
        let one = table_of(&built, Kind::Sequential);
        prop_assume!(one.is_ok());
        let one = one.unwrap();
        let two = table_of(&built, Kind::Marginal).unwrap();
        let three = table_of(&built, Kind::Partial).unwrap();
        let scale = one.total_ss;
        for ((r1, r2), r3) in one.terms.iter().zip(&two.terms).zip(&three.terms) {
            prop_assert_eq!(&r1.label, &r2.label);
            prop_assert_eq!(&r1.label, &r3.label);
            prop_assert_eq!(r1.df, r2.df);
            prop_assert_eq!(r1.df, r3.df);
            prop_assert!((r1.ss - r2.ss).abs() <= 1e-9 * scale, "{} {} {}", r1.label, r1.ss, r2.ss);
            prop_assert!((r1.ss - r3.ss).abs() <= 1e-9 * scale, "{} {} {}", r1.label, r1.ss, r3.ss);
        }
        // On a balanced layout the terms' sums of squares add up to the model's under every type.
        let sum: f64 = three.terms.iter().map(|r| r.ss).sum();
        prop_assert!((sum - three.model.ss).abs() <= 1e-9 * scale);
    }

    #[test]
    fn the_sums_of_squares_add_up_and_the_marginal_types_ignore_the_order_of_the_terms(
        rows in rows(),
    ) {
        prop_assume!(two_levels_each(&rows));
        let frame = layout(&rows);
        let built = design(&frame, "y ~ a * b + x", None);
        let one = table_of(&built, Kind::Sequential);
        prop_assume!(one.is_ok());
        let one = one.unwrap();
        let scale = one.total_ss;
        // Type 1: the terms' sums of squares are the model's, and the model's and the residual
        // are the total.
        let sum: f64 = one.terms.iter().map(|r| r.ss).sum();
        prop_assert!((sum - one.model.ss).abs() <= 1e-9 * scale, "{sum} {}", one.model.ss);
        prop_assert!((one.model.ss + one.residual_ss - one.total_ss).abs() <= 1e-9 * scale);
        let df: usize = one.terms.iter().map(|r| r.df).sum();
        prop_assert_eq!(df, one.model.df);
        prop_assert_eq!(one.model.df + 1, one.rank);
        // The last term in is adjusted for all the others under every type.
        let two = table_of(&built, Kind::Marginal).unwrap();
        prop_assert!((by_label(&one, "a:b").ss - by_label(&two, "a:b").ss).abs() <= 1e-9 * scale);
        for table in [&one, &two] {
            for row in table.terms.iter().filter(|row| row.df > 0) {
                prop_assert!(row.ss >= -1e-9 * scale);
                prop_assert!(row.f >= -1e-9, "{}", row.f);
                prop_assert!((0.0..=1.0).contains(&row.p), "{}", row.p);
                prop_assert!((-1e-9..=1.0).contains(&row.partial_eta_squared));
            }
            // Partial epsilon squared never exceeds partial eta squared; partial omega squared
            // lies between 0 and partial epsilon squared; both are negative exactly when the
            // row's F is below 1. The same holds on the row of the whole model.
            for row in std::iter::once(&table.model)
                .chain(&table.terms)
                .filter(|row| row.df > 0)
            {
                let (eta, omega, epsilon) = (
                    row.partial_eta_squared,
                    row.partial_omega_squared,
                    row.partial_epsilon_squared,
                );
                prop_assert!(epsilon <= eta, "{}: {epsilon} {eta}", row.label);
                prop_assert!(omega.abs() <= epsilon.abs(), "{}: {omega} {epsilon}", row.label);
                // The figure from F (open decision #101): the formula of Stata's manual on the
                // row's F; never the smaller in size, of the sign of omega squared, and below 1.
                let from_f = row.partial_omega_squared_f;
                let written = (row.f - 1.0) / (row.f + (table.residual_df + 1.0) / row.df as f64);
                prop_assert!(
                    (from_f - written).abs() <= 1e-9 * (1.0 + written.abs()),
                    "{}: {from_f} against {written}",
                    row.label
                );
                prop_assert!(
                    from_f.abs() >= omega.abs() - 1e-12,
                    "{}: {from_f} {omega}",
                    row.label
                );
                prop_assert!(from_f == 0.0 || omega == 0.0 || (from_f < 0.0) == (omega < 0.0));
                prop_assert!(from_f < 1.0);
                prop_assert!(omega == 0.0 || (omega < 0.0) == (epsilon < 0.0));
                prop_assert!(epsilon < 1.0 && omega < 1.0);
                if row.f < 1.0 - 1e-9 {
                    prop_assert!(omega < 0.0 && epsilon < 0.0, "{}: F {}", row.label, row.f);
                }
                if row.f > 1.0 + 1e-9 {
                    prop_assert!(omega > 0.0 && epsilon > 0.0, "{}: F {}", row.label, row.f);
                }
            }
            // On the row of the whole model partial eta squared is R² and partial epsilon
            // squared the adjusted R².
            let r_squared = table.model.ss / table.total_ss;
            let n = table.residual_df + table.rank as f64;
            let adjusted = 1.0 - (1.0 - r_squared) * (n - 1.0) / table.residual_df;
            prop_assert!((table.model.partial_eta_squared - r_squared).abs() <= 1e-9);
            // On that row the two omegas are one figure: N less the model's degrees of freedom
            // is the residual's plus one.
            prop_assert!(
                (table.model.partial_omega_squared_f - table.model.partial_omega_squared).abs()
                    <= 1e-9 * (1.0 + table.model.partial_omega_squared.abs()),
                "{} against {}",
                table.model.partial_omega_squared_f,
                table.model.partial_omega_squared
            );
            prop_assert!(
                (table.model.partial_epsilon_squared - adjusted).abs() <= 1e-9 * (1.0 + adjusted.abs()),
                "{} against {adjusted}",
                table.model.partial_epsilon_squared
            );
        }
        // Types 2 and 3 with the terms written in another order.
        let turned = design(&frame, "y ~ x + b * a", None);
        let two_turned = table_of(&turned, Kind::Marginal).unwrap();
        for (label, other) in [("a", "a"), ("b", "b"), ("x", "x"), ("a:b", "b:a")] {
            let (r, t) = (by_label(&two, label), by_label(&two_turned, other));
            prop_assert_eq!(r.df, t.df);
            prop_assert!((r.ss - t.ss).abs() <= 1e-8 * scale, "{label}: {} {}", r.ss, t.ss);
        }
        match (table_of(&built, Kind::Partial), table_of(&turned, Kind::Partial)) {
            (Ok(three), Ok(three_turned)) => {
                prop_assert_eq!(three.rank, three.columns);
                for (label, other) in [("a", "a"), ("b", "b"), ("x", "x"), ("a:b", "b:a")] {
                    let (r, t) = (by_label(&three, label), by_label(&three_turned, other));
                    prop_assert!((r.ss - t.ss).abs() <= 1e-8 * scale, "{label}: {} {}", r.ss, t.ss);
                }
                // Each term's partial test is the last step of some sequence, so no type 3 sum
                // of squares exceeds what the model explains with the residual.
                prop_assert!((by_label(&three, "a:b").ss - by_label(&two, "a:b").ss).abs() <= 1e-9 * scale);
                prop_assert!((by_label(&three, "x").ss - by_label(&two, "x").ss).abs() <= 1e-8 * scale);
            }
            (Err(Undefined::Aliased { rank, columns }), Err(Undefined::Aliased { .. })) => {
                // An empty cell: fewer independent columns than the model writes.
                prop_assert!(rank < columns);
                prop_assert!(two.rank < two.columns);
            }
            (left, right) => prop_assert!(false, "{left:?} against {right:?}"),
        }
    }

    #[test]
    fn frequency_weights_count_each_row_that_many_times(rows in rows()) {
        prop_assume!(two_levels_each(&rows));
        let weighted = design(&layout(&rows), "y ~ a + b + x", Some(("w", WeightKind::Frequency)));
        let written: Vec<(f64, i32, i32, f64, i64)> = rows
            .iter()
            .flat_map(|r| std::iter::repeat_n((r.0, r.1, r.2, r.3, 1), r.4 as usize))
            .collect();
        let expanded = design(&layout(&written), "y ~ a + b + x", None);
        for kind in [Kind::Sequential, Kind::Marginal, Kind::Partial] {
            match (table_of(&weighted, kind), table_of(&expanded, kind)) {
                (Ok(w), Ok(e)) => {
                    let scale = e.total_ss;
                    prop_assert_eq!(w.residual_df, e.residual_df);
                    prop_assert!((w.residual_ss - e.residual_ss).abs() <= 1e-9 * scale);
                    prop_assert!((w.total_ss - e.total_ss).abs() <= 1e-9 * scale);
                    for (rw, re) in w.terms.iter().zip(&e.terms) {
                        prop_assert_eq!(rw.df, re.df);
                        prop_assert!((rw.ss - re.ss).abs() <= 1e-9 * scale, "{} {}", rw.ss, re.ss);
                        if rw.df > 0 && re.ss > 1e-6 * scale {
                            for (got, want) in [
                                (rw.partial_omega_squared, re.partial_omega_squared),
                                (rw.partial_epsilon_squared, re.partial_epsilon_squared),
                            ] {
                                prop_assert!(
                                    (got - want).abs() <= 1e-7 * (1.0 + want.abs()),
                                    "{}: {got} against {want}",
                                    rw.label
                                );
                            }
                        }
                    }
                }
                (Err(w), Err(e)) => prop_assert_eq!(w, e),
                (w, e) => prop_assert!(false, "{w:?} against {e:?}"),
            }
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

/// A small layout: twelve rows, a of three levels, b and s of two, a covariate and a weight.
const D: &str = "let d = frame(y = [3.1, 4.2, 5.0, 6.3, 2.2, 7.9, 4.4, 5.1, 6.6, 3.8, 4.9, 7.2], a = [\"p\", \"p\", \"p\", \"p\", \"q\", \"q\", \"q\", \"q\", \"r\", \"r\", \"r\", \"r\"], b = [\"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\"], x = [1.0, 2.5, 0.3, 4.1, 2.2, 3.3, 1.9, 0.7, 2.8, 3.9, 1.1, 2.0], w = [1, 2, 1, 0, 1, 3, 1, 1, 2, 1, 1, 1], s = [\"k\", \"l\", \"k\", \"l\", \"k\", \"l\", \"k\", \"l\", \"k\", \"l\", \"k\", \"l\"]).add(a = category(a), b = category(b))\n";

fn with_d(line: &str) -> (i32, String, String) {
    go(&format!("{D}{line}\n"))
}

#[test]
fn each_form_keeps_its_own_arguments() {
    for (line, code, part) in [
        (
            "d.anova(y ~ a * b, group: a)",
            "Z0401",
            "`group:` belongs to the one-way form",
        ),
        (
            "d.anova(y ~ a * b, equal_var: true)",
            "Z0401",
            "`equal_var:` belongs to the one-way form",
        ),
        (
            "d.anova(y, group: a, type: 3)",
            "Z0401",
            "`type:` belongs to the factorial form",
        ),
        (
            "d.anova(y ~ a * b, type: 4)",
            "Z0401",
            "must be 1, 2 or 3, not 4",
        ),
        (
            "d.anova(y ~ a * b, type: 0)",
            "Z0401",
            "must be 1, 2 or 3, not 0",
        ),
        ("d.anova(y ~ a * b, type: \"2\")", "Z0401", "must be an Int"),
        (
            "d.anova(y ~ a * b, effect: cohen)",
            "Z0402",
            "`cohen` is not one of the allowed words",
        ),
        (
            "d.anova(y ~ a * b - 1)",
            "Z0401",
            "the formula removes the intercept",
        ),
        ("d.anova(y ~ 1)", "Z0401", "has no term to test"),
        ("d.anova(~ a * b)", "Z0322", "one-sided formula"),
        (
            "d.anova(y ~ a + x, weights: w)",
            "Z0411",
            "sampling weights",
        ),
        ("d.anova(y ~ a + s)", "Z0401", "`s` is text"),
        ("d.anova(y ~ a + nosuch)", "Z0101", "no variable `nosuch`"),
    ] {
        let (exit, _, stderr) = with_d(line);
        assert_eq!(exit, 1, "{line}");
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{line}: {stderr}"
        );
        assert!(stderr.contains(part), "{line}: {stderr}");
    }
}

#[test]
fn a_table_that_cannot_be_computed_is_refused_with_the_reason() {
    for (line, code, part) in [
        (
            "d.where(y > 100).anova(y ~ a)",
            "Z0801",
            "no row remains for `anova`",
        ),
        (
            "d.where(a == \"p\").anova(y ~ a + b)",
            "Z0401",
            "`a` has one level among the rows used",
        ),
        (
            "d.add(k = 2.5).anova(k ~ a + b)",
            "Z0401",
            "`k` has the same value in every row used",
        ),
        (
            "d.add(big = y * 1e160).anova(big ~ a)",
            "Z0803",
            "an outcome too large to square",
        ),
        (
            "d.add(big = x * 1e160).anova(y ~ a + big)",
            "Z0803",
            "a term too large to square",
        ),
        (
            "d.anova(y ~ a * b * x)",
            "Z0802",
            "no residual degrees of freedom: 12 observations and 12 estimated columns",
        ),
        (
            "d.add(z = 2 * y + 1).anova(y ~ z)",
            "Z0802",
            "reproduces `y` exactly",
        ),
        (
            "d.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b, type: 3)",
            "Z0401",
            "`type: 3` cannot adjust each term for every other: 1 of the model's 6 columns is determined by the others",
        ),
        (
            "d.add(x2 = 2 * x).anova(y ~ x + x2, type: 3)",
            "Z0401",
            "a term that repeats the others",
        ),
    ] {
        let (exit, _, stderr) = with_d(line);
        assert_eq!(exit, 1, "{line}");
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{line}: {stderr}"
        );
        assert!(stderr.contains(part), "{line}: {stderr}");
    }
}

#[test]
fn an_empty_cell_a_repeated_term_and_a_model_of_slopes_are_said_in_notes() {
    // An empty cell: types 1 and 2 give the table, the interaction on the degrees of freedom
    // the cells leave it.
    let (exit, stdout, stderr) = with_d("d.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b)");
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("  a:b         1 "), "{stdout}");
    assert!(stdout.contains("  model       4 "), "{stdout}");
    assert!(
        stderr.contains("1 of the model's 6 columns is determined by the others (a combination of levels with no observations, or a term that repeats the others) and carries no degree of freedom"),
        "{stderr}"
    );
    // A variable that repeats another has no test once the other is in the model.
    let (exit, stdout, stderr) = with_d("d.add(x2 = 2 * x).anova(y ~ x + x2)");
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("  x           0    0.000\n"), "{stdout}");
    assert!(stdout.contains("  x2          0    0.000\n"), "{stdout}");
    assert!(
        stderr.contains("`x` has no degrees of freedom once the other terms are in the model"),
        "{stderr}"
    );
    assert!(
        stderr.contains("(a term that repeats the others)"),
        "{stderr}"
    );
    assert!(
        stderr.contains("no term of the formula is a category, so each row tests a slope"),
        "{stderr}"
    );
    // Under type 1 the first of the two keeps its degree of freedom.
    let (exit, stdout, stderr) = with_d("d.add(x2 = 2 * x).anova(y ~ x + x2, type: 1)");
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("  x           1 "), "{stdout}");
    assert!(stdout.contains("  x2          0 "), "{stdout}");
    // Rows with NA and rows of zero weight are counted.
    let (exit, stdout, stderr) = go(
        "let m = frame(y = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 3.5, 2.0, 6.0], g = [\"a\", \"a\", \"b\", \"b\", NA, \"c\", \"c\", \"b\", \"a\", \"c\"], w = [1, 1, 1, 0, 1, 2, 1, 1, 1, 1])\nm.anova(y ~ category(g), weights: w, weight_type: frequency)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("N = 8, frequency weights w"), "{stdout}");
    assert!(
        stderr.contains(
            "2 rows with NA in a model variable dropped; 1 row with zero weight left out"
        ),
        "{stderr}"
    );
}

#[test]
fn one_category_gives_the_classical_one_way_table() {
    // `anova(y ~ g)` and `anova(y, group: g, equal_var: true)` are the same test.
    let (exit, stdout, stderr) = with_d(
        "let f = d.anova(y ~ a)\nlet o = d.anova(y, group: a, equal_var: true)\nf.statistic() - o.statistic()\nf.p_values() - o.p_values()\nf.df()\no.df()",
    );
    assert_eq!(exit, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].parse::<f64>().unwrap().abs() < 1e-12, "{stdout}");
    assert!(lines[1].parse::<f64>().unwrap().abs() < 1e-12, "{stdout}");
    // `df()` prints its two numbers on two lines.
    assert_eq!(lines.len(), 6, "{stdout}");
    assert_eq!(lines[2..4], lines[4..6], "{stdout}");
    assert_eq!(lines[2..4], ["2", "9"], "{stdout}");
}

#[test]
fn one_category_gives_the_measures_of_the_classical_one_way_form() {
    // `anova(y ~ a, effect:)` and `anova(y, group: a, equal_var: true, effect:)` report the same
    // omega squared and the same epsilon squared: with one term the partial measure is the plain
    // one, on the term's row and on the row of the whole model.
    let y = [
        3.1, 4.2, 5.0, 6.3, 2.2, 7.9, 4.4, 5.1, 6.6, 3.8, 4.9, 7.2, 9.5,
    ];
    let frame = frame_of(vec![
        ("y", Column::floats(y.iter().map(|v| Some(*v))).0),
        (
            "a",
            category(&["p", "q", "r"], (0..13).map(|i| Some(i * 3 / 13))),
        ),
    ]);
    let table = table_of(&design(&frame, "y ~ a", None), Kind::Marginal).unwrap();
    let mut omegas: Vec<f64> = Vec::new();
    for word in ["eta", "omega", "omega_f", "epsilon"] {
        let asked = Value::Word(word.to_string());
        let factorial = call(&frame, "y ~ a", vec![(EFFECT, asked.clone())]).unwrap();
        let mut values: Vec<Option<Value>> = vec![None; SLOTS];
        values[0] = Some(Value::Frame(frame.clone()));
        values[1] = Some(Value::Var("y".to_string()));
        values[2] = Some(Value::Var("a".to_string()));
        values[3] = Some(Value::Bool(true));
        values[EFFECT] = Some(asked);
        let args = Args {
            values,
            rest: Vec::new(),
            definitions: Vec::new(),
        };
        let one_way = match with_context("d.anova(y, group: a)", |ctx| anova(ctx, args)) {
            Ok(Value::TestResult(result)) => (*result).clone(),
            _ => panic!("the one-way form refuses"),
        };
        let plain = one_way.effect.as_ref().unwrap();
        let partial = factorial.terms[0].effect.as_ref().unwrap();
        let named = match word {
            "omega_f" => "omega squared from F".to_string(),
            word => format!("{word} squared"),
        };
        assert_eq!(plain.label, named);
        assert_eq!(partial.label, format!("partial {named}"));
        if word.starts_with("omega") {
            omegas.push(plain.value);
            omegas.push(partial.value);
        }
        assert!(
            (plain.value - partial.value).abs() < 1e-12,
            "{word}: {} against {}",
            plain.value,
            partial.value
        );
        let model = match word {
            "eta" => table.model.partial_eta_squared,
            "omega" => table.model.partial_omega_squared,
            "omega_f" => table.model.partial_omega_squared_f,
            _ => table.model.partial_epsilon_squared,
        };
        assert!((model - partial.value).abs() < 1e-12, "{word}");
    }
    // With one term the figure from F is omega squared itself (open decision #101): in the
    // one-way form bit for bit, in a model of one term within rounding.
    assert_eq!(omegas.len(), 4);
    assert_eq!(omegas[0].to_bits(), omegas[2].to_bits());
    assert!((omegas[1] - omegas[3]).abs() < 1e-12, "{omegas:?}");
}

/// `effect: omega_f` (open decision #101, option A; step 38e): the partial omega squared from
/// F, beside `omega`, which stays the ruled formula.
#[test]
fn the_partial_omega_squared_from_f_is_named_and_is_the_larger_in_size() {
    // For each term of a model of several terms the figure from F is the larger in size, and
    // it is the formula of Stata's manual on the row's own F and degrees of freedom.
    let script = [
        "let w = d.anova(y ~ a * b, effect: omega)",
        "let f = d.anova(y ~ a * b, effect: omega_f)",
        "abs(f.effect_size(\"a\")) > abs(w.effect_size(\"a\"))",
        "abs(f.effect_size(\"b\")) > abs(w.effect_size(\"b\"))",
        "abs(f.effect_size(\"a:b\")) > abs(w.effect_size(\"a:b\"))",
        "let stata = (f.statistic(\"b\") - 1) / (f.statistic(\"b\") + (f.df(\"b\")[2] + 1) / f.df(\"b\")[1])",
        "abs(f.effect_size(\"b\") - stata) < 1e-12",
        "f.effect_size()",
        "f.estimates().where(term == \"partial omega squared from F (b)\").n_rows()",
    ]
    .join("\n");
    let (exit, stdout, stderr) = with_d(&script);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["true", "true", "true", "true", "NA", "1"],
        "{stdout}"
    );
    // F below 1: the figure is negative, printed as it is, with the note of #99 under its name.
    assert_eq!(
        notes_of(&stderr),
        vec![
            "note: F is below 1 for the model and `a`, so their partial omega squared is negative; read each as no evidence of an effect (line 2)",
            "note: F is below 1 for the model and `a`, so their partial omega squared from F is negative; read each as no evidence of an effect (line 3)",
        ],
        "{stderr}"
    );
    // The table names the measure in its heading and under it, with the formula.
    let (exit, stdout, stderr) = with_d("d.anova(y ~ a * b, effect: omega_f)");
    assert_eq!(exit, 0, "{stderr}");
    assert!(
        stdout.contains("p value   partial omega squared from F\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "partial omega squared from F = (F − 1) / (F + (df residual + 1) / df); N = 12"
        ),
        "{stdout}"
    );
    // The word is refused where it is misspelt, with the four words.
    let (exit, _, stderr) = with_d("d.anova(y ~ a * b, effect: omega_g)");
    assert_eq!(exit, 1);
    assert!(stderr.contains("Z0402"), "{stderr}");
    assert!(stderr.contains("eta, omega, omega_f, epsilon"), "{stderr}");
    assert!(stderr.contains("did you mean `omega_f`?"), "{stderr}");
}

/// The notes of a run on `d` without the two its own first line raises, where `category`
/// replaces `a` and `b`.
fn notes_of(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| !l.ends_with("replaced the variable of that name (line 1)"))
        .collect()
}

#[test]
fn the_rows_whose_estimate_is_negative_are_named_in_one_note() {
    // Partial eta squared cannot be negative, so it has no note.
    let (exit, stdout, stderr) = with_d("d.anova(y ~ a * b)");
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("partial eta squared"), "{stdout}");
    assert!(notes_of(&stderr).is_empty(), "{stderr}");
    // F is 0.94 for the model and 0.35 for a, 1.96 for b and 1.02 for a:b.
    for word in ["omega", "epsilon"] {
        let (exit, stdout, stderr) = with_d(&format!("d.anova(y ~ a * b, effect: {word})"));
        assert_eq!(exit, 0, "{stderr}");
        assert!(
            stdout.contains(&format!("p value   partial {word} squared")),
            "{stdout}"
        );
        assert!(!stdout.contains("eta squared"), "{stdout}");
        assert_eq!(
            notes_of(&stderr),
            vec![format!(
                "note: F is below 1 for the model and `a`, so their partial {word} squared is negative; read each as no evidence of an effect (line 2)"
            )],
            "{stderr}"
        );
        let lines: Vec<&str> = stdout.lines().collect();
        let row = |name: &str| -> f64 {
            let line = lines
                .iter()
                .find(|l| l.trim_start().starts_with(&format!("{name} ")))
                .unwrap_or_else(|| panic!("no row {name}: {stdout}"));
            line.split_whitespace().last().unwrap().parse().unwrap()
        };
        assert!(row("model") < 0.0 && row("a") < 0.0, "{stdout}");
        assert!(row("b") > 0.0 && row("a:b") > 0.0, "{stdout}");
    }
    // One row alone: F is 1.04 for the model, 2.03 for b and 0.05 for x.
    let (exit, _, stderr) = with_d("d.anova(y ~ b + x, type: 1, effect: omega)");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(
        notes_of(&stderr),
        vec![
            "note: F is below 1 for `x`, so its partial omega squared is negative; read it as no evidence of an effect (line 2)"
        ],
        "{stderr}"
    );
    // `estimates()` carries the figure of the table, -0.121, as it is.
    let (exit, stdout, stderr) = with_d(
        "d.anova(y ~ a * b, effect: omega).estimates().where(term == \"partial omega squared (a)\")",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("frame · 1 row × 9 variables"), "{stdout}");
    assert!(stdout.contains("-0.121451"), "{stdout}");
}

#[test]
fn the_methods_of_the_result_read_the_whole_model_and_estimates_reads_the_terms() {
    let (exit, stdout, stderr) = with_d(
        "let t = d.anova(y ~ a * b + x)\nt.statistic()\nt.df()\nt.n_obs()\nt.estimates().n_rows()",
    );
    assert_eq!(exit, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[1], "6", "{stdout}");
    assert_eq!(lines[2], "5", "{stdout}");
    assert_eq!(lines[3], "12", "{stdout}");
    assert_eq!(lines[4], "8", "{stdout}");
}

/// One term of the table read by its name (open decision #100, option A; step 38d).
#[test]
fn a_term_of_the_table_is_read_by_its_name() {
    // Each figure is the one `estimates()` holds in the row of that term, bit for bit; a bare
    // name reads as text does, and an interaction is named as the table prints it.
    let script = [
        "let t = d.anova(y ~ a * b + x)",
        "let e = t.estimates()",
        "let a = e.where(term == \"a\")",
        "t.statistic(\"a\") == a.max(statistic)",
        "t.p_values(\"a\") == a.max(p_value)",
        "t.df(\"a\")[1] == a.max(df)",
        "t.df(\"a\")[2] == a.max(df2)",
        "t.effect_size(\"a\") == e.where(term == \"partial eta squared (a)\").max(estimate)",
        "let ab = e.where(term == \"a:b\")",
        "t.statistic(\"a:b\") == ab.max(statistic)",
        "t.p_values(\"a:b\") == ab.max(p_value)",
        "t.df(\"a:b\")[1] == ab.max(df)",
        "t.effect_size(\"a:b\") == e.where(term == \"partial eta squared (a:b)\").max(estimate)",
        "t.statistic(a) == t.statistic(\"a\")",
        "t.p_values(b) == t.p_values(\"b\")",
        "t.df(x)[1] == t.df(\"x\")[1]",
        "t.effect_size(term: b) == t.effect_size(\"b\")",
        "len(t.df(\"a\"))",
    ]
    .join("\n");
    let (exit, stdout, stderr) = with_d(&script);
    assert_eq!(exit, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[..13], ["true"; 13], "{stdout}");
    assert_eq!(lines[13], "2", "{stdout}");
    // Without a name the four are the whole model's, as before; under `effect:` the row's
    // measure is the one asked for.
    let (exit, stdout, stderr) = with_d(
        "let t = d.anova(y ~ a * b, effect: omega)\nt.effect_size(\"a\") == t.estimates().where(term == \"partial omega squared (a)\").max(estimate)\nt.effect_size()\nt.statistic() == t.statistic(\"a\")\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "true\nNA\nfalse\n");
    // A name that is no term of the table, with the terms; the row of the whole model is the
    // call without a name.
    for (line, said) in [
        (
            "d.anova(y ~ a * b).p_values(\"c\")",
            "`c` is not a row of this table",
        ),
        (
            "d.anova(y ~ a * b).statistic(\"a:c\")",
            "did you mean `a:b`? the rows are `a`, `b`, `a:b`",
        ),
        (
            "d.anova(y ~ a * b).df(\"model\")",
            "without a name the method gives the figure of the whole test",
        ),
        (
            "d.anova(y ~ a * b).effect_size(\"residual\")",
            "`residual` is not a row of this table",
        ),
        // A one-way result has one row and no name for it.
        (
            "d.anova(y, group: a).statistic(\"a\")",
            "a test's result has no terms",
        ),
        (
            "d.anova(y, group: a).p_values(a)",
            "a test's result has no terms",
        ),
        // The methods that read no row of a test's result say which do.
        (
            "d.anova(y ~ a * b).std_errors(\"a\")",
            "`p_values`, `statistic`, `df` and `effect_size` read one row by its name",
        ),
    ] {
        let (exit, _, stderr) = with_d(line);
        assert_eq!(exit, 1, "{line}");
        assert!(stderr.contains("error[Z0414]"), "{line}: {stderr}");
        assert!(stderr.contains(said), "{line}: {stderr}");
    }
}

// ----- the snapshots -----

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "{D}d.anova(y ~ a * b)\nd.anova(y ~ a * b + x, type: 1)\nd.anova(y ~ x + a * b, type: 1)\nd.anova(y ~ a * b + x, type: 3)\nd.anova(y ~ a + category(s))\nd.anova(y ~ a + x, weights: w, weight_type: frequency)\nd.anova(y ~ a + x, weights: w, weight_type: analytic)\nd.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b)\nd.add(x2 = 2 * x).anova(y ~ x + x2)\nd.anova(y ~ x)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage ~ industry * category(female) + educ)\nwages.anova(lwage ~ industry * category(female) + educ, type: 3)\nwages.where(year == 2010).anova(lwage ~ industry + category(union) + educ * exper, type: 1)\nlet t = d.anova(y ~ a * b + x)\nt.statistic()\nt.df()\nt.p_values()\nt.n_obs()\nt.effect_size()\nt.estimates()\nt.ci()\nt.summary()\nd.anova(y ~ a * b, effect: omega)\nd.anova(y ~ a * b, effect: omega_f)\nd.anova(y ~ a * b + x, type: 3, effect: epsilon)\nd.anova(y ~ a + x, weights: w, weight_type: frequency, effect: omega)\nd.anova(y ~ b, effect: epsilon)\nwages.anova(lwage ~ industry * category(female) + educ, effect: omega)\nwages.anova(lwage ~ industry * category(female) + educ, type: 3, effect: epsilon)\nlet u = d.anova(y ~ a * b + x, effect: epsilon)\nu.estimates()\nu.summary()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for line in [
        "d.anova(y ~ a * b, group: a)",
        "d.anova(y ~ a * b, equal_var: true)",
        "d.anova(y, group: a, type: 3)",
        "d.anova(y ~ a * b, type: 4)",
        "d.anova(y ~ a * b, type: \"2\")",
        "d.anova(y ~ a * b - 1)",
        "d.anova(y ~ 1)",
        "d.anova(y ~ a + x, weights: w)",
        "d.where(y > 100).anova(y ~ a)",
        "d.where(a == \"p\").anova(y ~ a + b)",
        "d.add(k = 2.5).anova(k ~ a + b)",
        "d.add(big = y * 1e160).anova(big ~ a)",
        "d.add(big = x * 1e160).anova(y ~ a + big)",
        "d.anova(y ~ a * b * x)",
        "d.add(z = 2 * y + 1).anova(y ~ z)",
        "d.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b, type: 3)",
        "d.add(x2 = 2 * x).anova(y ~ x + x2, type: 3)",
        "d.anova(y ~ a + s)",
        "d.anova(y ~ a * b).p_values(\"c\")",
        "d.anova(y ~ a * b).statistic(\"a:c\")",
        "d.anova(y, group: a).df(\"a\")",
        "d.anova(y ~ a * b).std_errors(\"a\")",
        "d.anova(y ~ a * b, effect: cohen)",
        "d.anova(y ~ a * b, effect: omga)",
    ] {
        let (exit, _, stderr) = with_d(line);
        assert_eq!(exit, 1, "{line}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
