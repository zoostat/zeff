//! The shorthands (Language Reference, section 7.6, "Shorthands"; release 0.7.0, step 29):
//! each gives the `Plot` of its grammar form, value for value, and so the same scene byte
//! for byte, with the same notes; and where the grammar refuses, the shorthand refuses with
//! the same words.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_plot::{grammar, marks, scales, scene, shorthands};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("a plot reads no file")
    }
}

/// What a call gave: the value, or the words of its refusal; and the notes it raised.
type Gave = (Result<Value, String>, Vec<String>);

/// A chain of calls, each given the value of the one before as its first argument, as a
/// script's statement runs them: `steps` are the bodies with the arguments after the first.
fn chain(first: Value, steps: Vec<(Body, Vec<Option<Value>>)>) -> Gave {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut value = first;
    let mut refusal = None;
    for (body, more) in steps {
        let mut values = vec![Some(value.clone())];
        values.extend(more);
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text: "",
            columns_used: Vec::new(),
            frame: None,
            group_rows: None,
            callee_span: Span::new(0, 1),
            arg_spans: (0..values.len())
                .map(|at| Some(Span::new(at, at + 1)))
                .collect(),
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
        };
        match body(
            &mut ctx,
            Args {
                values,
                rest: Vec::new(),
                definitions: Vec::new(),
            },
        ) {
            Ok(next) => value = next,
            Err(refused) => {
                refusal = Some(format!("{}: {}", refused.code, refused.message));
                break;
            }
        }
    }
    let notes = messages.iter().map(|m| m.message.clone()).collect();
    (refusal.map_or(Ok(value), Err), notes)
}

fn var(name: &str) -> Option<Value> {
    Some(Value::Var(name.to_string()))
}

/// The scene of what a call gave, as it is written.
fn written(gave: &Gave) -> Result<String, String> {
    match &gave.0 {
        Ok(Value::Plot(plot)) => Ok(scene(plot).json().write()),
        Ok(_) => Err("no plot".to_string()),
        Err(words) => Err(words.clone()),
    }
}

/// `data.line(x, y, color:, by:)` and its grammar form.
fn lines_both(frame: &Frame, color: Option<&str>, by: Option<&str>) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(
        data.clone(),
        vec![(
            shorthands::line,
            vec![var("x"), var("y"), color.and_then(var), by.and_then(var)],
        )],
    );
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![
        (grammar::plot, vec![var("x"), var("y"), color.and_then(var)]),
        (marks::lines, Vec::new()),
    ];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

/// `data.bar(var, color:, by:)` and its grammar form.
fn bars_both(frame: &Frame, of: &str, color: Option<&str>, by: Option<&str>) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(
        data.clone(),
        vec![(
            shorthands::bar,
            vec![var(of), color.and_then(var), by.and_then(var)],
        )],
    );
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![
        (grammar::plot, vec![var(of), None, color.and_then(var)]),
        (marks::bars, Vec::new()),
    ];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

/// The two gave one thing: the same `Plot`, the same scene as written, the same notes; or
/// the same refusal.
fn same(short: &Gave, long: &Gave) -> Result<(), String> {
    if short.0 != long.0 {
        return Err(format!("{:?}\nagainst\n{:?}", short.0, long.0));
    }
    if written(short) != written(long) {
        return Err("two scenes".to_string());
    }
    if short.1 != long.1 {
        return Err(format!("{:?}\nagainst\n{:?}", short.1, long.1));
    }
    Ok(())
}

#[test]
fn line_and_bar_are_their_grammar_forms() {
    let frame = small("d");
    // `small` has `dose`, `response`, `arm` and `weight`; the property's frames have `x`,
    // `y`, `g` and `z`, so this one is renamed to them.
    let frame = common::frame(
        "d",
        vec![
            ("x", frame.table.column("dose").unwrap().clone()),
            ("y", frame.table.column("response").unwrap().clone()),
            ("g", frame.table.column("arm").unwrap().clone()),
            ("z", frame.table.column("weight").unwrap().clone()),
        ],
    );
    for (color, by) in [
        (None, None),
        (Some("g"), None),
        (None, Some("g")),
        (Some("g"), Some("g")),
    ] {
        let (short, long) = lines_both(&frame, color, by);
        assert!(short.0.is_ok(), "{:?}", short.0);
        same(&short, &long).unwrap();
        let (short, long) = bars_both(&frame, "g", color, by);
        assert!(short.0.is_ok(), "{:?}", short.0);
        same(&short, &long).unwrap();
    }
    // What the grammar refuses the shorthand refuses, in the same words: a line coloured by
    // a number, a variable the frame lacks, a panel by one it lacks.
    for (color, by) in [(Some("z"), None), (Some("w"), None), (None, Some("w"))] {
        let (short, long) = lines_both(&frame, color, by);
        assert!(short.0.is_err(), "{color:?} {by:?}");
        same(&short, &long).unwrap();
    }
    for (color, by) in [(Some("w"), None), (None, Some("w"))] {
        let (short, long) = bars_both(&frame, "g", color, by);
        assert!(short.0.is_err(), "{color:?} {by:?}");
        same(&short, &long).unwrap();
    }
    // A bar has one fill: coloured by a number its bars are whole, as the grammar's are.
    let (short, long) = bars_both(&frame, "g", Some("z"), None);
    assert!(short.0.is_ok());
    same(&short, &long).unwrap();
    // A scene of each, kept.
    let (short, _) = lines_both(&frame, Some("g"), None);
    insta::assert_snapshot!("a_line_by_arm", written(&short).unwrap());
    let (short, _) = bars_both(&frame, "g", None, None);
    insta::assert_snapshot!("a_bar_for_each_arm", written(&short).unwrap());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    /// Whatever the rows, with a colour or without, in panels or not: the shorthand gives
    /// what its grammar form gives, the plot, the scene byte for byte and the notes, or the
    /// same refusal.
    #[test]
    fn a_shorthand_gives_the_scene_of_its_grammar_form(
        rows in rows(),
        color in prop::sample::select(vec![None, Some("g"), Some("z")]),
        by in prop::sample::select(vec![None, Some("g")]),
    ) {
        let frame = tried(&rows);
        let (short, long) = lines_both(&frame, color, by);
        prop_assert_eq!(same(&short, &long), Ok(()));
        for of in ["g", "x"] {
            let (short, long) = bars_both(&frame, of, color, by);
            prop_assert_eq!(same(&short, &long), Ok(()));
        }
    }
}
