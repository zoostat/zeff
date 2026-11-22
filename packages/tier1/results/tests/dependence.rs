//! What each column of a model matrix is a function of (release 0.9.0, step 10; open decision
//! #195): on every form of term of section 7.2 of the Language Reference, the variables a
//! column depends on, its form, the columns a variable enters, and its derivative, exactly; and
//! as a property, that the derivative recorded for a column equals the central difference of
//! the column when the variable is moved, on generated data.

use proptest::prelude::*;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{CallContext, Fail, Files, Randomness};
use zeff_core::value::Frame;
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_results::dependence::{Form, NoDerivative};
use zeff_results::design::Design;
use zeff_testkit::formula;

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("a design reads no file")
    }
}

/// The design of a formula on a table, or its refusal rendered.
fn build(table: &Table, text: &str) -> Result<Design, String> {
    let f = formula(text);
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let d = {text}\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(8, 8 + text.len()),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
        width: zeff_core::display::PAGE_WIDTH,
        shown: None,
    };
    Design::build(&mut ctx, &Frame::new(table.clone()), &f, &[]).map_err(|e| e.render(&sources))
}

fn floats(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

/// Six rows: an outcome, two positive regressors, a 0/1 variable and a category of three levels.
fn small() -> Table {
    let levels = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let c = Column::categories(levels, [0, 1, 2, 0, 1, 2].map(Some)).expect("a category");
    Table::new(vec![
        ("y".to_string(), floats(&[1.0, 2.0, 0.5, 3.0, 2.5, 1.5])),
        ("a".to_string(), floats(&[1.0, 2.0, 3.0, 4.0, 0.5, 2.5])),
        ("b".to_string(), floats(&[0.3, 1.7, 0.9, 2.2, 1.1, 0.6])),
        ("d".to_string(), floats(&[0.0, 1.0, 0.0, 1.0, 1.0, 0.0])),
        ("c".to_string(), c),
    ])
    .unwrap()
}

/// The column of a name.
fn at(design: &Design, name: &str) -> usize {
    design
        .names
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| panic!("no column `{name}` among {:?}", design.names))
}

fn close(ours: &[f64], theirs: &[f64], what: &str) {
    assert_eq!(ours.len(), theirs.len(), "{what}");
    for (a, b) in ours.iter().zip(theirs) {
        assert!(
            (a - b).abs() <= 1e-14 * b.abs().max(1.0),
            "{what}: {a} against {b}"
        );
    }
}

/// Every form of term of section 7.2: a variable, a power, a function, arithmetic, an
/// interaction, a product, a category by each way of writing one, an interaction with a level,
/// and the intercept; with what each column is a function of and its derivative.
#[test]
fn every_form_of_term_says_what_it_is_a_function_of() {
    let table = small();
    let a = [1.0, 2.0, 3.0, 4.0, 0.5, 2.5];
    let b = [0.3, 1.7, 0.9, 2.2, 1.1, 0.6];

    let design = build(
        &table,
        "y ~ a + a^2 + log(b) + log(a * 2 + b) + a:b + d + c + a:c",
    )
    .unwrap();
    let dependence = |name: &str| design.dependence(at(&design, name)).unwrap();

    assert_eq!(dependence("(intercept)").form, Form::Intercept);
    assert!(dependence("(intercept)").variables.is_empty());
    assert_eq!(dependence("a").form, Form::Variable);
    assert_eq!(dependence("a").variables, ["a"]);
    assert_eq!(dependence("a^2").form, Form::Power);
    assert_eq!(dependence("a^2").variables, ["a"]);
    assert_eq!(dependence("log(b)").form, Form::Function);
    assert_eq!(dependence("log(b)").variables, ["b"]);
    assert_eq!(dependence("log(a * 2 + b)").form, Form::Function);
    assert_eq!(dependence("log(a * 2 + b)").variables, ["a", "b"]);
    assert_eq!(dependence("a:b").form, Form::Interaction);
    assert_eq!(dependence("a:b").variables, ["a", "b"]);
    assert_eq!(dependence("d").form, Form::Variable);
    assert_eq!(dependence("c=y").form, Form::Level);
    assert_eq!(dependence("c=y").variables, ["c"]);
    assert_eq!(dependence("a:c=z").form, Form::Interaction);
    assert_eq!(dependence("a:c=z").variables, ["a", "c"]);

    // The columns a variable enters, in the order of the table.
    let names = |variable: &str| -> Vec<&str> {
        design
            .columns_of(variable)
            .into_iter()
            .map(|j| design.names[j].as_str())
            .collect()
    };
    assert_eq!(
        names("a"),
        ["a", "a^2", "log(a * 2 + b)", "a:b", "a:c=y", "a:c=z"]
    );
    assert_eq!(names("b"), ["log(b)", "log(a * 2 + b)", "a:b"]);
    assert_eq!(names("d"), ["d"]);
    assert_eq!(names("c"), ["c=y", "c=z", "a:c=y", "a:c=z"]);
    assert!(names("nobody").is_empty());
    assert_eq!(design.model_variables(), ["a", "b", "d", "c"]);

    // The derivatives, exactly.
    let derivative = |name: &str, variable: &str| design.derivative(at(&design, name), variable);
    close(
        &derivative("a", "a").unwrap().unwrap(),
        &[1.0; 6],
        "d a / d a",
    );
    let twice: Vec<f64> = a.iter().map(|v| 2.0 * v).collect();
    close(
        &derivative("a^2", "a").unwrap().unwrap(),
        &twice,
        "d a^2 / d a",
    );
    let inverse: Vec<f64> = b.iter().map(|v| 1.0 / v).collect();
    close(
        &derivative("log(b)", "b").unwrap().unwrap(),
        &inverse,
        "d log(b) / d b",
    );
    let inner: Vec<f64> = a.iter().zip(&b).map(|(x, y)| x * 2.0 + y).collect();
    let in_a: Vec<f64> = inner.iter().map(|v| 2.0 / v).collect();
    let in_b: Vec<f64> = inner.iter().map(|v| 1.0 / v).collect();
    close(
        &derivative("log(a * 2 + b)", "a").unwrap().unwrap(),
        &in_a,
        "d log(a * 2 + b) / d a",
    );
    close(
        &derivative("log(a * 2 + b)", "b").unwrap().unwrap(),
        &in_b,
        "d log(a * 2 + b) / d b",
    );
    close(&derivative("a:b", "a").unwrap().unwrap(), &b, "d a:b / d a");
    close(&derivative("a:b", "b").unwrap().unwrap(), &a, "d a:b / d b");
    // The slope of an interaction with a level is the level's indicator.
    close(
        &derivative("a:c=z", "a").unwrap().unwrap(),
        &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        "d a:c=z / d a",
    );
    // A column the variable does not enter has no derivative to add.
    assert_eq!(derivative("a", "b"), Ok(None));
    assert_eq!(derivative("(intercept)", "a"), Ok(None));
    assert_eq!(derivative("c=y", "a"), Ok(None));
    // A category's effect is a difference between levels, not a slope.
    assert_eq!(
        derivative("c=y", "c"),
        Err(NoDerivative::Category {
            variable: "c".to_string(),
            column: "c=y".to_string()
        })
    );
    assert_eq!(
        derivative("a:c=z", "c"),
        Err(NoDerivative::Category {
            variable: "c".to_string(),
            column: "a:c=z".to_string()
        })
    );
}

/// The other ways of writing a term: a product, a category made in the formula with a contrast,
/// the smooth functions, a power that is itself an expression, and the intercept removed.
#[test]
fn the_other_forms_and_what_is_not_smooth() {
    let table = small();
    let a = [1.0, 2.0, 3.0, 4.0, 0.5, 2.5];
    let b = [0.3, 1.7, 0.9, 2.2, 1.1, 0.6];

    // `a * b` is a, b and a:b; without an intercept the columns keep their places.
    let design = build(&table, "y ~ a * b - 1").unwrap();
    assert_eq!(design.names, ["a", "b", "a:b"]);
    assert_eq!(design.columns_of("a"), [0, 2]);
    close(
        &design.derivative(2, "a").unwrap().unwrap(),
        &b,
        "d a:b / d a",
    );

    // A category made in the formula, with a contrast: still a category.
    let design = build(&table, "y ~ category(d) + category(c, contrast: sum) + a").unwrap();
    let level = design
        .names
        .iter()
        .position(|n| n.starts_with("d="))
        .unwrap();
    assert_eq!(design.dependence(level).unwrap().form, Form::Level);
    assert_eq!(design.dependence(level).unwrap().variables, ["d"]);
    assert!(matches!(
        design.derivative(level, "d"),
        Err(NoDerivative::Category { .. })
    ));
    assert_eq!(design.columns_of("a"), [design.names.len() - 1]);

    // The functions differentiated, each against its derivative written out.
    let design = build(
        &table,
        "y ~ exp(a) + sqrt(b) + log1p(a) + expm1(b) + log10(a) + log2(b) + abs(a - 2) + (a / b) + a^b + log(a + b)^3 + a^1",
    )
    .unwrap();
    let derivative = |name: &str, variable: &str| {
        design
            .derivative(at(&design, name), variable)
            .unwrap_or_else(|e| panic!("{name} in {variable}: {e:?}"))
            .unwrap_or_else(|| panic!("{name} does not depend on {variable}"))
    };
    let each = |f: &dyn Fn(f64, f64) -> f64| -> Vec<f64> {
        a.iter().zip(&b).map(|(x, y)| f(*x, *y)).collect()
    };
    let near = |ours: &[f64], theirs: &[f64], what: &str| {
        for (o, t) in ours.iter().zip(theirs) {
            assert!(
                (o - t).abs() <= 1e-12 * t.abs().max(1.0),
                "{what}: {o} against {t}"
            );
        }
    };
    near(&derivative("exp(a)", "a"), &each(&|x, _| x.exp()), "exp");
    near(
        &derivative("sqrt(b)", "b"),
        &each(&|_, y| 0.5 / y.sqrt()),
        "sqrt",
    );
    near(
        &derivative("log1p(a)", "a"),
        &each(&|x, _| 1.0 / (1.0 + x)),
        "log1p",
    );
    near(
        &derivative("expm1(b)", "b"),
        &each(&|_, y| y.exp()),
        "expm1",
    );
    near(
        &derivative("log10(a)", "a"),
        &each(&|x, _| 1.0 / (x * std::f64::consts::LN_10)),
        "log10",
    );
    near(
        &derivative("log2(b)", "b"),
        &each(&|_, y| 1.0 / (y * std::f64::consts::LN_2)),
        "log2",
    );
    near(
        &derivative("abs(a - 2)", "a"),
        &each(&|x, _| {
            if x > 2.0 {
                1.0
            } else if x < 2.0 {
                -1.0
            } else {
                0.0
            }
        }),
        "abs",
    );
    near(
        &derivative("(a / b)", "a"),
        &each(&|_, y| 1.0 / y),
        "a / b in a",
    );
    near(
        &derivative("(a / b)", "b"),
        &each(&|x, y| -x / (y * y)),
        "a / b in b",
    );
    near(
        &derivative("a^b", "a"),
        &each(&|x, y| y * x.powf(y - 1.0)),
        "a^b in a",
    );
    near(
        &derivative("a^b", "b"),
        &each(&|x, y| x.powf(y) * x.ln()),
        "a^b in b",
    );
    near(
        &derivative("log(a + b)^3", "a"),
        &each(&|x, y| 3.0 * (x + y).ln() * (x + y).ln() / (x + y)),
        "log(a + b)^3",
    );
    near(&derivative("a^1", "a"), &[1.0; 6], "a^1");
    assert_eq!(
        design.dependence(at(&design, "a^b")).unwrap().form,
        Form::Function
    );
    assert_eq!(
        design.dependence(at(&design, "a^1")).unwrap().form,
        Form::Power
    );

    // What is not smooth says so, naming the column and what in it.
    let design = build(&table, "y ~ round(a) + (a > 2)").unwrap();
    for (name, said) in [
        ("round(a)", "`round` is not differentiated"),
        ("(a > 2)", "`>` is not arithmetic"),
    ] {
        assert_eq!(
            design.derivative(at(&design, name), "a"),
            Err(NoDerivative::NotSmooth {
                column: name.to_string(),
                what: said.to_string()
            })
        );
    }
}

/// A design whose columns are held as values has no record of what they were made from.
#[test]
fn a_transformed_design_says_that_it_cannot_say() {
    let table = small();
    let design = build(&table, "y ~ a + b").unwrap();
    let columns: Vec<Vec<f64>> = (0..design.names.len())
        .map(|j| design.column_values(j))
        .collect();
    let held = design.with_columns(columns, true);
    assert_eq!(held.dependence(1), None);
    assert!(held.columns_of("a").is_empty());
    assert_eq!(held.derivative(1, "a"), Err(NoDerivative::Transformed));
}

/// The formulas of the property: every smooth form, alone and multiplied together.
const FORMULAS: [&str; 6] = [
    "y ~ a + a^2 + a^3",
    "y ~ log(a) + sqrt(b) + exp(a / 4)",
    "y ~ a * b + log(a + b)^2",
    "y ~ a:b:d + a:c + b:c",
    "y ~ log1p(a * b) + a^b + (a / b)",
    "y ~ a + b + d + c - 1",
];

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// The derivative recorded for a column equals the central difference of the column when
    /// the variable is moved by a small step and the design built again, for every column and
    /// each of the two continuous variables, on generated data.
    #[test]
    fn the_derivative_is_the_central_difference(
        rows in proptest::collection::vec((0.5f64..4.0, 0.5f64..3.0, any::<bool>(), 0usize..3), 5..12),
        which in 0usize..FORMULAS.len(),
    ) {
        let table_at = |shift_a: f64, shift_b: f64| -> Table {
            let labels = ["x", "y", "z"];
            Table::new(vec![
                ("y".to_string(), floats(&vec![1.0; rows.len()])),
                ("a".to_string(), floats(&rows.iter().map(|r| r.0 + shift_a).collect::<Vec<_>>())),
                ("b".to_string(), floats(&rows.iter().map(|r| r.1 + shift_b).collect::<Vec<_>>())),
                ("d".to_string(), floats(&rows.iter().map(|r| f64::from(u8::from(r.2))).collect::<Vec<_>>())),
                (
                    "c".to_string(),
                    Column::categories(
                        labels.iter().map(|l| l.to_string()).collect(),
                        rows.iter().map(|r| Some(r.3 as i32)),
                    )
                    .expect("a category"),
                ),
            ])
            .unwrap()
        };
        let text = FORMULAS[which];
        let design = build(&table_at(0.0, 0.0), text).unwrap();
        let h = 1e-5;
        for (variable, up, down) in [
            ("a", table_at(h, 0.0), table_at(-h, 0.0)),
            ("b", table_at(0.0, h), table_at(0.0, -h)),
        ] {
            let (above, below) = (build(&up, text).unwrap(), build(&down, text).unwrap());
            prop_assert_eq!(&above.names, &design.names);
            for column in 0..design.names.len() {
                let (high, low) = (above.column_values(column), below.column_values(column));
                let recorded = design.derivative(column, variable).unwrap();
                for row in 0..rows.len() {
                    let difference = (high[row] - low[row]) / (2.0 * h);
                    let ours = recorded.as_ref().map_or(0.0, |d| d[row]);
                    prop_assert!(
                        (ours - difference).abs() <= 1e-6 * difference.abs().max(1.0),
                        "{}: column `{}` in {}, row {}: {} against the difference {}",
                        text, design.names[column], variable, row, ours, difference
                    );
                }
            }
        }
    }
}
