//! `frame`: the types of its columns, the rules of sections 3.7 and 4.1, every message, a property
//! against the kernel, and the reference on datasets built from their values.

use proptest::prelude::*;
use zeff_columns::ColumnType;
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, Files};
use zeff_core::value::Value;
use zeff_data::frame::frame;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_testkit::{Accuracy, assert_prints, in_code, reference, root, run};

// ----- the body on its own: the types a column takes -----

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("frame reads no file")
    }
}

/// Call the body with definitions; the frame's column types, or the error's code.
fn build(
    definitions: Vec<(&str, Value)>,
) -> Result<Vec<(String, ColumnType, usize, usize)>, String> {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let spans: Vec<Span> = (0..definitions.len())
        .map(|i| Span::new(i * 10, i * 10 + 5))
        .collect();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        callee_span: Span::new(0, 5),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: spans,
        output: &mut output,
        messages: &mut messages,
    };
    let args = Args {
        values: Vec::new(),
        rest: Vec::new(),
        definitions: definitions
            .into_iter()
            .map(|(n, v)| (n.to_string(), v))
            .collect(),
    };
    match frame(&mut ctx, args) {
        Ok(Value::Frame(f)) => Ok(f
            .table
            .names()
            .iter()
            .map(|n| {
                let c = f.table.column(n).unwrap();
                (n.clone(), c.ty(), c.len(), c.n_missing())
            })
            .collect()),
        Ok(other) => panic!("not a frame: {other:?}"),
        Err(e) => Err(e.code.clone()),
    }
}

fn list(values: Vec<Value>) -> Value {
    Value::List(values)
}

#[test]
fn each_kind_of_list_makes_its_column_type() {
    let built = build(vec![
        (
            "b",
            list(vec![Value::Bool(true), Value::Na, Value::Bool(false)]),
        ),
        ("i", list(vec![Value::Int(1), Value::Int(-2), Value::Na])),
        (
            "f",
            list(vec![
                Value::Float(1.5),
                Value::Float(2.5),
                Value::Float(0.0),
            ]),
        ),
        (
            "joined",
            list(vec![Value::Int(1), Value::Float(2.5), Value::Na]),
        ),
        (
            "s",
            list(vec![
                Value::Str("a".into()),
                Value::Na,
                Value::Str("".into()),
            ]),
        ),
    ])
    .unwrap();
    assert_eq!(
        built,
        [
            ("b".to_string(), ColumnType::Bool, 3, 1),
            ("i".to_string(), ColumnType::Int, 3, 1),
            ("f".to_string(), ColumnType::Float, 3, 0),
            ("joined".to_string(), ColumnType::Float, 3, 1),
            ("s".to_string(), ColumnType::Str, 3, 1),
        ]
    );
}

#[test]
fn one_row_and_one_column_are_frames_too() {
    assert_eq!(
        build(vec![("x", list(vec![Value::Int(42)]))]).unwrap(),
        [("x".to_string(), ColumnType::Int, 1, 0)]
    );
}

#[test]
fn what_the_body_refuses() {
    assert_eq!(
        build(vec![
            ("x", list(vec![Value::Int(1)])),
            ("y", list(vec![Value::Int(1), Value::Int(2)]))
        ]),
        Err("Z0217".into())
    );
    assert_eq!(build(vec![("x", list(vec![]))]), Err("Z0219".into()));
    assert_eq!(
        build(vec![("x", list(vec![Value::Na, Value::Na]))]),
        Err("Z0219".into())
    );
    assert_eq!(
        build(vec![(
            "x",
            list(vec![Value::Int(1), Value::Str("a".into())])
        )]),
        Err("Z0214".into())
    );
    assert_eq!(
        build(vec![("x", list(vec![Value::Bool(true), Value::Int(1)]))]),
        Err("Z0214".into())
    );
    assert_eq!(
        build(vec![("x", list(vec![list(vec![Value::Int(1)])]))]),
        Err("Z0401".into())
    );
    assert_eq!(build(vec![("x", Value::Na)]), Err("Z0401".into()));
    assert_eq!(build(vec![("x", Value::Int(1))]), Err("Z0401".into()));
}

// ----- through the language -----

fn stdout(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
    outcome.output.stdout
}

fn stderr(script: &str) -> String {
    let outcome = run(script);
    assert_ne!(outcome.exit_code, 0, "expected an error for {script}");
    outcome.output.stderr
}

#[test]
fn a_frame_from_lists_prints_and_computes() {
    let out = stdout(
        "let d = frame(wage = [23.4, 19.0, 31.2], female = [true, false, true])\nd\nd.mean(wage)\nd.mean(female)\nframe(x = [1, 2.5, NA]).mean(x)\n",
    );
    assert_eq!(
        out,
        "d · 3 rows × 2 variables\n\n      wage   female\n  1   23.4   true\n  2     19   false\n  3   31.2   true\n24.5333\n0.666667\n1.75\n"
    );
    assert_eq!(
        stdout("frame(x = [1])\n"),
        "frame · 1 row × 1 variable\n\n      x\n  1   1\n"
    );
    assert_eq!(
        stdout("frame(`gross wage` = [1, 2], `1st` = [3, 4]).mean(`gross wage`)\n"),
        "1.5\n"
    );
    assert_eq!(
        stdout("let d = frame(a = [\"x\", NA, \"z\"], n = [1, 2, 3])\nd\n"),
        "d · 3 rows × 2 variables\n\n      a    n\n  1   x    1\n  2   NA   2\n  3   z    3\n"
    );
}

#[test]
fn na_takes_the_type_of_its_list_and_is_skipped_with_a_warning() {
    let outcome = run("frame(x = [1, NA, 2.5, NA]).mean(x)\n");
    assert_eq!(outcome.output.stdout, "1.75\n");
    assert_eq!(
        outcome.output.stderr,
        "warning: 2 rows with NA in x were skipped (line 1)\n"
    );
}

#[test]
fn the_errors_of_frame_through_the_language() {
    let unequal = stderr("let d = frame(x = [1, 2, 3], y = [1, 2])\n");
    assert!(
        unequal.starts_with(
            "error[Z0217]: `frame` needs lists of one length: `x` has 3 values and `y` has 2"
        ),
        "{unequal}"
    );
    assert!(
        unequal.contains("2 here") && unequal.contains("3 here"),
        "both lists are marked:\n{unequal}"
    );
    let empty = stderr("frame(x = [])\n");
    assert!(empty.starts_with("error[Z0219]"), "{empty}");
    let all_na = stderr("frame(x = [NA, NA])\n");
    assert!(
        all_na.starts_with("error[Z0219]: `x` holds only `NA`"),
        "{all_na}"
    );
    let nested = stderr("frame(x = [[1], [2]])\n");
    assert!(
        nested.starts_with(
            "error[Z0401]: `x` holds a List; a column holds Bool, Int, Float or Str values"
        ),
        "{nested}"
    );
    let not_a_list = stderr("frame(x = NA)\n");
    assert!(
        not_a_list.starts_with("error[Z0401]: `x` must be a list of values, not an NA"),
        "{not_a_list}"
    );
    let none = stderr("frame()\n");
    assert!(
        none.starts_with("error[Z0406]: `frame` needs at least one definition"),
        "{none}"
    );
    let mixed = stderr("frame(x = [1, \"a\"])\n");
    assert!(mixed.starts_with("error[Z0214]"), "{mixed}");
    let twice = stderr("frame(x = [1], x = [2])\n");
    assert!(twice.starts_with("error[Z0405]"), "{twice}");
    let option = stderr("frame(x: [1])\n");
    assert!(option.starts_with("error[Z0404]"), "{option}");
}

#[test]
fn messages() {
    let cases: &[(&str, &str)] = &[
        (
            "unequal_lengths",
            "let d = frame(x = [1, 2, 3], y = [1, 2])\n",
        ),
        ("empty_list", "frame(x = [])\n"),
        ("only_na", "frame(x = [NA, NA])\n"),
        ("nested_list", "frame(x = [[1], [2]])\n"),
        ("not_a_list", "frame(x = NA)\n"),
        ("no_definition", "frame()\n"),
    ];
    for (name, script) in cases {
        insta::assert_snapshot!(*name, run(script).output.stderr);
    }
    insta::assert_snapshot!(
        "frame_printed",
        stdout("let d = frame(x = [1, 2], y = [\"a\", \"b\"])\nd\nprint(d)\n")
    );
}

// ----- a property: the mean of a frame built from values is the kernel's mean -----

fn literal(value: Option<f64>) -> String {
    match value {
        None => "NA".to_string(),
        Some(x) => format!("{x:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    #[test]
    fn a_frame_built_from_values_has_their_mean(
        values in prop::collection::vec(prop::option::weighted(0.85, -1.0e6..1.0e6_f64), 1..40),
    ) {
        prop_assume!(values.iter().any(Option::is_some));
        let items: Vec<String> = values.iter().map(|v| literal(*v)).collect();
        let script = format!("frame(x = [{}]).mean(x)\n", items.join(", "));
        let outcome = run(&script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let present: Vec<f64> = values.iter().flatten().copied().collect();
        let expected = format_float(zeff_numeric::mean(&present).unwrap());
        prop_assert_eq!(outcome.output.stdout.trim(), expected.as_str());
        let missing = values.len() - present.len();
        if missing > 0 {
            prop_assert!(outcome.output.stderr.contains("with NA in x"));
        } else {
            prop_assert_eq!(outcome.output.stderr.as_str(), "");
        }
    }
}

// ----- the reference: datasets rebuilt from their values with frame -----

/// The columns of a small dataset as `frame` literals: numbers as written, Bool words as written,
/// a missing cell as `NA`, text in quotes.
fn as_literals(text: &str) -> Vec<(String, Vec<String>)> {
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let mut columns: Vec<(String, Vec<String>)> =
        header.iter().map(|h| (h.to_string(), Vec::new())).collect();
    for line in lines {
        for (i, cell) in line.split(',').enumerate() {
            let t = cell.trim();
            let literal = if t.is_empty() || t == "NA" || t == "." {
                "NA".to_string()
            } else if t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("false") {
                t.to_lowercase()
            } else if t.parse::<f64>().is_ok() {
                t.to_string()
            } else {
                format!("\"{t}\"")
            };
            columns[i].1.push(literal);
        }
    }
    columns
}

#[test]
fn frames_built_from_the_datasets_values_match_the_reference() {
    let reference = reference();
    let mut compared = 0;
    for name in ["wages", "bools", "one_row", "zero_variance"] {
        let dataset = &reference.datasets[name];
        let text = std::fs::read_to_string(root().join(&dataset.file)).unwrap();
        let columns = as_literals(&text);
        let definitions: Vec<String> = columns
            .iter()
            .filter_map(|(n, values)| in_code(n).map(|w| format!("{w} = [{}]", values.join(", "))))
            .collect();
        for (column, stats) in &dataset.columns {
            let Some(written) = in_code(column) else {
                continue;
            };
            if stats.n == 0 {
                continue; // a list of NA alone has no type; the reader declares such a column
            }
            let script = format!(
                "let d = frame({})\nd.mean({written})\n",
                definitions.join(", ")
            );
            let outcome = run(&script);
            assert_eq!(
                outcome.exit_code, 0,
                "{name}.{column}:\n{}",
                outcome.output.stderr
            );
            assert_prints(
                &format!("mean of frame({name}).{column}"),
                &outcome.output.statements[1].stdout,
                &stats.mean,
                Accuracy::ClosedForm,
            );
            compared += 1;
        }
    }
    assert!(compared >= 10, "{compared}");
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
