//! `TestResult` and the shared arguments of the tests (release 0.7.0, step 7; Language
//! Reference, sections 4 and 6; open decision #91): a hand-built result prints with its
//! hypothesis block, estimate, statistic, effect size and decision line, exports as a table and
//! LaTeX, and answers the result interface; `level:`, `alternative:`, the refusal of sampling
//! weights (Z0411), the rule of two groups (Z0412) and the complete rows with their note; and
//! the fields the types table of section 4 names.

use std::rc::Rc;
use zeff_columns::{Column, Table as Columns};
use zeff_core::display::{decision_line, estimate_line, statistic_line, test_result_text};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Alternative, Estimate, Frame, Hypotheses, Statistic, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_results::export::{result_table, tex};
use zeff_results::interface::{ci, estimates, n_obs, p_values, summary};
use zeff_results::report::report;
use zeff_stats::testing;
use zeff_testkit::root;

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The two-group `t_test` of section 10.2's example, built by hand with its numbers.
fn welch() -> TestResult {
    TestResult {
        method: "t_test".to_string(),
        title: "t_test · Welch two-sample · lwage by female".to_string(),
        hypotheses: Some(Hypotheses {
            h0: "the population means of lwage are equal in male and female".to_string(),
            h1: "the population means of lwage differ between male and female".to_string(),
            symbols: Some((
                "μ(male) − μ(female) = 0".to_string(),
                "μ(male) − μ(female) ≠ 0".to_string(),
            )),
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(zeff_core::value::Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), zeff_core::value::Align::Left),
                ("N".to_string(), zeff_core::value::Align::Right),
                ("mean".to_string(), zeff_core::value::Align::Right),
                ("sd".to_string(), zeff_core::value::Align::Right),
            ],
            rows: vec![
                vec![
                    "male".into(),
                    "9,604".into(),
                    "3.214".into(),
                    "0.512".into(),
                ],
                vec![
                    "female".into(),
                    "8,597".into(),
                    "3.061".into(),
                    "0.498".into(),
                ],
            ],
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: Some(Estimate {
            label: "difference (male − female)".to_string(),
            value: 0.153,
            ci: Some((0.138, 0.168)),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "t".to_string(),
            value: 20.42,
            df: vec![18_074.2],
        },
        p_value: 1e-90,
        effect: Some(Estimate {
            label: "Hedges' g".to_string(),
            value: 0.30,
            ci: Some((0.27, 0.33)),
            decimals: 2,
        }),
        level: 0.95,
        alpha: Some(0.05),
        after: Vec::new(),
        notes: Vec::new(),
        n: 18_201,
        dropped: 211,
    }
}

/// A result without a standard symbolic form, one-sided, kept at a 99% level.
fn signed_rank() -> TestResult {
    TestResult {
        method: "wilcoxon_test".to_string(),
        title: "wilcoxon_test · signed rank · gain".to_string(),
        hypotheses: Some(Hypotheses {
            h0: "the distribution of gain is symmetric about 0".to_string(),
            h1: "values of gain tend to lie above 0".to_string(),
            symbols: None,
        }),
        alternative: Alternative::Greater,
        header: vec!["N = 12".to_string()],
        table: None,
        estimate: None,
        statistic: Statistic {
            name: "V".to_string(),
            value: 61.0,
            df: Vec::new(),
        },
        p_value: 0.0231,
        effect: None,
        level: 0.99,
        alpha: Some(0.01),
        after: vec!["exact p-value, no ties".to_string()],
        notes: Vec::new(),
        n: 12,
        dropped: 0,
    }
}

// ----- printing -----

#[test]
fn a_hand_built_result_prints_as_section_10_2_shows() {
    let text = test_result_text(&welch(), 100);
    insta::assert_snapshot!("welch_printed", text);
    let expected = "\
t_test · Welch two-sample · lwage by female
  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
  H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

  group       N    mean      sd
  male    9,604   3.214   0.512
  female  8,597   3.061   0.498
difference (male − female) = 0.153   95% CI [0.138, 0.168]
t = 20.42   df = 18,074.2   p < 0.001
Hedges' g = 0.30   95% CI [0.27, 0.33]
decision at α = 0.05: reject H0 (p < 0.001)
";
    // The hypothesis block, the lines after the table and the decision line are exactly the
    // example's; the grid's own spacing is the engine's.
    for line in expected.lines().filter(|l| {
        !l.starts_with("  group") && !l.starts_with("  male") && !l.starts_with("  female")
    }) {
        assert!(text.contains(line), "missing line: {line}\n{text}");
    }
}

#[test]
fn a_result_without_symbols_prints_sentences_only_and_its_decision() {
    let text = test_result_text(&signed_rank(), 100);
    insta::assert_snapshot!("signed_rank_printed", text);
    assert!(text.contains("  H0: the distribution of gain is symmetric about 0\n  H1: values of gain tend to lie above 0\n"), "{text}");
    assert!(
        text.contains(
            "V = 61.00   p = 0.023\ndecision at α = 0.01: fail to reject H0 (p = 0.023)\n"
        ),
        "{text}"
    );
}

#[test]
fn the_lines_round_as_the_display_rules_say() {
    assert_eq!(
        decision_line(0.05, 0.0004),
        "decision at α = 0.05: reject H0 (p < 0.001)"
    );
    assert_eq!(
        decision_line(0.1, 0.231),
        "decision at α = 0.1: fail to reject H0 (p = 0.231)"
    );
    assert_eq!(
        decision_line(testing::alpha(0.95), 0.05),
        "decision at α = 0.05: fail to reject H0 (p = 0.050)"
    );
    assert_eq!(
        decision_line(testing::alpha(0.999), 0.0004),
        "decision at α = 0.001: reject H0 (p < 0.001)"
    );
    assert_eq!(
        estimate_line(
            &Estimate {
                label: "mean".into(),
                value: 2.5,
                ci: None,
                decimals: 2
            },
            0.95
        ),
        "mean = 2.50"
    );
    assert_eq!(
        estimate_line(
            &Estimate {
                label: "π".into(),
                value: 0.4,
                ci: Some((0.35, 0.45)),
                decimals: 3
            },
            0.9
        ),
        "π = 0.400   90% CI [0.350, 0.450]"
    );
    assert_eq!(
        statistic_line(
            &Statistic {
                name: "z".into(),
                value: 1.959_9,
                df: vec![]
            },
            0.05
        ),
        "z = 1.96   p = 0.050"
    );
    assert_eq!(
        statistic_line(
            &Statistic {
                name: "chi2".into(),
                value: 7.81,
                df: vec![3.0]
            },
            0.05
        ),
        "chi2(3) = 7.81   p = 0.050"
    );
}

// ----- the interface, export and report -----

/// A call context over no file, for a method on a hand-built result.
fn with_context<T>(f: impl FnOnce(&mut CallContext<'_>) -> T) -> T {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "r.method()",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 8),
        arg_spans: vec![Some(Span::new(0, 1)), None, None],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    f(&mut ctx)
}

fn args(values: Vec<Option<Value>>) -> Args {
    Args {
        values,
        rest: Vec::new(),
        definitions: Vec::new(),
    }
}

#[test]
fn the_result_interface_answers_for_a_hand_built_result() {
    let result = Value::TestResult(Rc::new(welch()));
    with_context(|ctx| {
        let table = summary(ctx, args(vec![Some(result.clone())])).unwrap();
        let Value::Table(table) = table else {
            panic!("a table")
        };
        assert_eq!(table.title, "t_test · Welch two-sample · lwage by female");
        assert_eq!(table.before.len(), 2, "{:?}", table.before);
        assert_eq!(table.after.len(), 4, "{:?}", table.after);
        assert_eq!(
            table.after[3],
            "decision at α = 0.05: reject H0 (p < 0.001)"
        );

        let Value::Frame(frame) = estimates(ctx, args(vec![Some(result.clone())])).unwrap() else {
            panic!("a frame")
        };
        assert_eq!(frame.table.rows(), 1);
        let cell = |name: &str| {
            frame
                .table
                .column(name)
                .unwrap()
                .get(0)
                .map(|v| format!("{v:?}"))
        };
        assert_eq!(
            cell("term").as_deref(),
            Some("Str(\"difference (male − female)\")")
        );
        assert_eq!(cell("estimate").as_deref(), Some("Float(0.153)"));
        assert_eq!(cell("ci_upper").as_deref(), Some("Float(0.168)"));

        assert_eq!(
            p_values(ctx, args(vec![Some(result.clone())])).unwrap(),
            Value::Float(1e-90)
        );
        assert_eq!(
            n_obs(ctx, args(vec![Some(result.clone())])).unwrap(),
            Value::Int(18_201)
        );

        let Value::Table(intervals) =
            ci(ctx, args(vec![Some(result.clone()), None, None])).unwrap()
        else {
            panic!("a table")
        };
        assert_eq!(intervals.title, "ci · 95%");
        assert_eq!(intervals.rows.len(), 2);
        assert_eq!(intervals.rows[1], vec!["Hedges' g", "0.27", "0.33"]);
        let other = ci(
            ctx,
            args(vec![Some(result.clone()), None, Some(Value::Float(0.9))]),
        );
        assert_eq!(other.unwrap_err().code, "Z0401");
    });
}

#[test]
fn a_hand_built_result_exports_as_a_table_and_as_latex() {
    let table = result_table(&Value::TestResult(Rc::new(welch()))).expect("a table");
    assert_eq!(
        table.before[0],
        "  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0"
    );
    assert_eq!(table.rows.len(), 2);
    let latex = tex(&table);
    insta::assert_snapshot!("welch_tex", latex);
    assert!(latex.contains("decision at"), "{latex}");
    let plain = result_table(&Value::TestResult(Rc::new(signed_rank()))).expect("a table");
    // Without an estimate the statistic stands in a one-row grid, its line not repeated.
    assert_eq!(plain.rows.len(), 1);
    assert!(
        plain.after.iter().all(|l| !l.starts_with("V = ")),
        "{:?}",
        plain.after
    );
}

#[test]
fn hand_built_results_report_side_by_side() {
    let list = Value::List(vec![
        Value::TestResult(Rc::new(welch())),
        Value::TestResult(Rc::new(signed_rank())),
    ]);
    with_context(|ctx| {
        let Value::Table(table) =
            report(ctx, args(vec![Some(list), None, None, None, None, None])).unwrap()
        else {
            panic!("a table")
        };
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.rows[0], vec!["estimate", "0.153", ""]);
        assert_eq!(table.rows[1], vec!["statistic", "t = 20.42", "V = 61.00"]);
        assert_eq!(table.rows[3], vec!["p value", "<0.001", "0.023"]);
        assert_eq!(table.rows[5], vec!["N", "18,201", "12"]);
        assert_eq!(table.after.len(), 2, "{:?}", table.after);
        assert!(
            table.after[0].starts_with("(1) H0: the population means of lwage"),
            "{}",
            table.after[0]
        );
        insta::assert_snapshot!(
            "tests_report",
            zeff_core::display::show(&Value::Table(table))
        );
    });
}

// ----- the shared arguments -----

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

#[test]
fn level_and_alternative_are_read_with_their_defaults() {
    with_context(|ctx| {
        assert_eq!(
            testing::level(ctx, &args(vec![None, None]), 1, "t_test").unwrap(),
            0.95
        );
        assert_eq!(
            testing::level(ctx, &args(vec![None, Some(Value::Float(0.9))]), 1, "t_test").unwrap(),
            0.9
        );
        let bad = testing::level(ctx, &args(vec![None, Some(Value::Float(1.5))]), 1, "t_test")
            .unwrap_err();
        assert_eq!(bad.code, "Z0401");
        assert_eq!(
            testing::alternative(ctx, &args(vec![None, None]), 1).unwrap(),
            Alternative::TwoSided
        );
        assert_eq!(
            testing::alternative(ctx, &args(vec![None, Some(Value::Word("less".into()))]), 1)
                .unwrap(),
            Alternative::Less
        );
        assert_eq!(Alternative::Greater.word(), "greater");
    });
    assert_eq!(testing::alpha(0.95), 0.05);
    assert_eq!(testing::alpha(0.99), 0.01);
}

#[test]
fn sampling_weights_are_refused_and_the_other_kinds_taken() {
    let frame = frame_of(vec![
        ("y", Column::floats(vec![Some(1.0), Some(2.0)]).0),
        ("w", Column::floats(vec![Some(1.0), Some(3.0)]).0),
    ]);
    with_context(|ctx| {
        let none =
            testing::weights(ctx, &frame, &args(vec![None, None, None]), 1, 2, "t_test").unwrap();
        assert!(none.is_none());
        let sampling = testing::weights(
            ctx,
            &frame,
            &args(vec![None, Some(Value::Var("w".into())), None]),
            1,
            2,
            "t_test",
        )
        .unwrap_err();
        assert_eq!(sampling.code, "Z0411");
        assert!(
            sampling
                .message
                .contains("sampling weights, which a test cannot use"),
            "{}",
            sampling.message
        );
        insta::assert_snapshot!(
            "sampling_weights_refused",
            sampling.render(&zeff_diagnostics::Sources::new())
        );
        let frequency = testing::weights(
            ctx,
            &frame,
            &args(vec![
                None,
                Some(Value::Var("w".into())),
                Some(Value::Word("frequency".into())),
            ]),
            1,
            2,
            "t_test",
        )
        .unwrap();
        assert!(frequency.is_some());
        let unknown = testing::weights(
            ctx,
            &frame,
            &args(vec![
                None,
                Some(Value::Var("nope".into())),
                Some(Value::Word("analytic".into())),
            ]),
            1,
            2,
            "t_test",
        )
        .unwrap_err();
        assert_eq!(unknown.code, "Z0101");
    });
}

#[test]
fn two_groups_by_label_and_the_refusals() {
    let codes = Column::ints(vec![Some(1), Some(0), Some(1), None, Some(0)]);
    with_context(|ctx| {
        let groups = testing::two_groups(ctx, &codes, "female", 1, "t_test").unwrap();
        assert_eq!(
            groups.labels,
            ["female=0".to_string(), "female=1".to_string()]
        );
        assert_eq!(groups.rows, [vec![1, 4], vec![0, 2]]);
        let texts = Column::strs(["b", "a", "b"].iter().map(|s| Some(*s)));
        let groups = testing::two_groups(ctx, &texts, "arm", 1, "t_test").unwrap();
        assert_eq!(groups.labels, ["a".to_string(), "b".to_string()]);
        let one = testing::two_groups(ctx, &Column::ints(vec![Some(1), Some(1)]), "g", 1, "t_test")
            .unwrap_err();
        assert_eq!(one.code, "Z0412");
        assert!(one.message.contains("has one level"), "{}", one.message);
        let three = testing::two_groups(
            ctx,
            &Column::ints(vec![Some(1), Some(2), Some(3)]),
            "g",
            1,
            "t_test",
        )
        .unwrap_err();
        assert_eq!(three.code, "Z0412");
        assert!(three.message.contains("has 3 levels"), "{}", three.message);
        insta::assert_snapshot!(
            "two_groups_refused",
            format!(
                "{}{}",
                one.render(&zeff_diagnostics::Sources::new()),
                three.render(&zeff_diagnostics::Sources::new())
            )
        );
    });
}

#[test]
fn complete_rows_drop_and_count_with_a_note() {
    let x = Column::floats(vec![Some(1.0), None, Some(3.0), Some(4.0)]).0;
    let y = Column::floats(vec![Some(1.0), Some(2.0), None, Some(4.0)]).0;
    let frame = frame_of(vec![("x", x.clone()), ("y", y.clone())]);
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "d.t_test(x)",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 8),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let (kept, dropped) = testing::complete_rows(&mut ctx, &frame, &[("x", &x), ("y", &y)]);
    assert_eq!(kept, vec![0, 3]);
    assert_eq!(dropped, 2);
    let (kept_one, dropped_one) = testing::complete_rows(&mut ctx, &frame, &[("x", &x)]);
    assert_eq!((kept_one.len(), dropped_one), (3, 1));
    drop(ctx);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].message, "2 rows with NA in x or y dropped");
    assert_eq!(messages[1].message, "1 row with NA in x dropped");
}

// ----- the types table of section 4 -----

#[test]
fn the_fields_the_types_table_names_exist() {
    let text =
        std::fs::read_to_string(root().join("docs/spec/zeff_language_reference.md")).unwrap();
    let row = text
        .lines()
        .find(|l| l.starts_with("| Result | `TestResult`"))
        .expect("the Result row of the types table");
    let result = welch();
    // Each name the row gives is a field of the value, read here so that a renamed field fails.
    let fields: Vec<(&str, bool)> = vec![
        ("statistic", result.statistic.value.is_finite()),
        ("df", !result.statistic.df.is_empty()),
        ("p_value", result.p_value.is_finite()),
        ("estimate", result.estimate.is_some()),
        (
            "ci",
            result.estimate.as_ref().is_some_and(|e| e.ci.is_some()),
        ),
        ("effect_size", result.effect.is_some()),
        ("n_obs", result.n > 0),
        (
            "h0",
            result.hypotheses.as_ref().is_some_and(|h| !h.h0.is_empty()),
        ),
        (
            "h1",
            result.hypotheses.as_ref().is_some_and(|h| !h.h1.is_empty()),
        ),
        ("alternative", result.alternative == Alternative::TwoSided),
    ];
    for (name, present) in fields {
        assert!(row.contains(&format!("`{name}`")), "the row names `{name}`");
        assert!(present, "`{name}` is carried");
    }
}
