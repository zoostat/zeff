//! The methods of a test's result alone (open decision #94; release 0.7.0, step 12b): `statistic`,
//! `df`, `effect_size`, `h0`, `h1` and `alternative` on hand-built results, the shape of
//! `estimates` with its `df` columns and its effect-size row, the refusal on a fit, and every
//! method through the engine on `t_test`, `chi2_test`, `kruskal_test`, `test` and `fisher_test`,
//! with the sentence the step's definition of done asks for; and a result of several rows that
//! each rest on an estimate (the comparisons of `posthoc`; release 0.7.0, step 19), which
//! `estimates` and `ci` give row by row.

mod common;

use common::{Options, call, fit_with, table_of};
use std::rc::Rc;
use zeff_columns::Value as Cell;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Randomness};
use zeff_core::value::{Alternative, Estimate, Hypotheses, Statistic, TermTest, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_results::interface::{alternative, ci, df, effect_size, estimates, h0, h1, statistic};
use zeff_testkit::run;

/// A body called with one value, its failure returned.
fn try_call(body: Body, value: Value) -> Result<Value, Fail> {
    try_with(body, vec![Some(value)])
}

/// A body called with its arguments by slot, its failure returned.
fn try_with(body: Body, values: Vec<Option<Value>>) -> Result<Value, Fail> {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &common::NoFiles,
        script: "t.zeff",
        text: "f(x)",
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(0, 1),
        arg_spans: vec![Some(Span::new(2, 3)); values.len()],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    body(
        &mut ctx,
        Args {
            values,
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    )
}

fn method(body: Body, test: &Rc<TestResult>) -> Value {
    call(
        body,
        vec![Some(Value::TestResult(Rc::clone(test)))],
        Vec::new(),
        Vec::new(),
    )
}

/// A Welch t test with everything a result can carry.
fn welch() -> Rc<TestResult> {
    Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "t_test".to_string(),
        title: "t_test · Welch · lwage by female".to_string(),
        hypotheses: Some(Hypotheses {
            h0: "the population means of lwage are equal in male and female".to_string(),
            h1: "the population means of lwage differ between male and female".to_string(),
            symbols: Some(("μ₁ = μ₂".to_string(), "μ₁ ≠ μ₂".to_string())),
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: None,
        estimate: Some(Estimate {
            label: "difference (male − female)".to_string(),
            value: 0.137_707,
            ci: Some((0.123_754, 0.151_659)),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "t".to_string(),
            value: 19.345_8,
            df: vec![18_130.3],
        },
        p_value: 1.500_26e-82,
        effect: Some(Estimate {
            label: "Hedges' g".to_string(),
            value: 0.286_923,
            ci: Some((0.257_706, 0.316_14)),
            decimals: 3,
        }),
        level: 0.95,
        alpha: Some(0.05),
        after: Vec::new(),
        notes: Vec::new(),
        n: 18_201,
        dropped: 211,
    })
}

/// An F test of a restriction, as `test` gives it: two degrees of freedom, no hypotheses.
fn wald() -> Rc<TestResult> {
    Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "test".to_string(),
        title: "test · m · educ == 0".to_string(),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: None,
        estimate: None,
        statistic: Statistic {
            name: "F".to_string(),
            value: 1_576.79,
            df: vec![1.0, 18_198.0],
        },
        p_value: 0.0,
        effect: None,
        level: 0.95,
        alpha: None,
        after: Vec::new(),
        notes: Vec::new(),
        n: 18_201,
        dropped: 0,
    })
}

/// An exact test, as `fisher_test` gives it: an odds ratio, no statistic, one-sided.
fn exact() -> Rc<TestResult> {
    Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "fisher_test".to_string(),
        title: "fisher_test · exact · union × female".to_string(),
        hypotheses: Some(Hypotheses {
            h0: "union and female are independent in the population".to_string(),
            h1: "the odds of female = male are higher among union = no than among union = yes"
                .to_string(),
            symbols: Some(("OR = 1".to_string(), "OR > 1".to_string())),
        }),
        alternative: Alternative::Greater,
        header: Vec::new(),
        table: None,
        estimate: Some(Estimate {
            label: "odds ratio (conditional MLE)".to_string(),
            value: 1.044_72,
            ci: Some((0.991_214, f64::INFINITY)),
            decimals: 3,
        }),
        statistic: Statistic {
            name: String::new(),
            value: f64::NAN,
            df: Vec::new(),
        },
        p_value: 0.085_954_2,
        effect: None,
        level: 0.95,
        alpha: Some(0.05),
        after: Vec::new(),
        notes: Vec::new(),
        n: 18_412,
        dropped: 0,
    })
}

#[test]
fn the_methods_read_a_hand_built_result() {
    let t = welch();
    assert_eq!(method(statistic, &t), Value::Float(19.345_8));
    assert_eq!(method(df, &t), Value::List(vec![Value::Float(18_130.3)]));
    assert_eq!(method(effect_size, &t), Value::Float(0.286_923));
    assert_eq!(
        method(h0, &t),
        Value::Str("the population means of lwage are equal in male and female".to_string())
    );
    assert_eq!(
        method(h1, &t),
        Value::Str("the population means of lwage differ between male and female".to_string())
    );
    assert_eq!(method(alternative, &t), Value::Str("two_sided".to_string()));

    let f = wald();
    assert_eq!(method(statistic, &f), Value::Float(1_576.79));
    assert_eq!(
        method(df, &f),
        Value::List(vec![Value::Float(1.0), Value::Float(18_198.0)])
    );
    assert_eq!(method(effect_size, &f), Value::Na);
    assert_eq!(method(h0, &f), Value::Na);
    assert_eq!(method(h1, &f), Value::Na);
    assert_eq!(method(alternative, &f), Value::Str("two_sided".to_string()));

    let e = exact();
    assert_eq!(method(statistic, &e), Value::Na);
    assert_eq!(method(df, &e), Value::Na);
    assert_eq!(method(effect_size, &e), Value::Na);
    assert_eq!(method(alternative, &e), Value::Str("greater".to_string()));
    assert!(matches!(method(h1, &e), Value::Str(s) if s.starts_with("the odds of")));
}

/// The frame's columns as names, and its rows.
fn shape(value: Value) -> (Vec<String>, usize) {
    match value {
        Value::Frame(frame) => (frame.table.names().to_vec(), frame.table.rows()),
        other => panic!("not a frame: {other:?}"),
    }
}

#[test]
fn estimates_has_the_df_columns_and_the_effect_row() {
    let names = [
        "term",
        "estimate",
        "std_error",
        "statistic",
        "df",
        "df2",
        "p_value",
        "ci_lower",
        "ci_upper",
    ];
    let (columns, rows) = shape(method(estimates, &welch()));
    assert_eq!(columns, names);
    assert_eq!(rows, 2, "the estimate and the effect size");
    let (columns, rows) = shape(method(estimates, &wald()));
    assert_eq!(columns, names);
    assert_eq!(rows, 1, "no effect size, no row");
    let (_, rows) = shape(method(estimates, &exact()));
    assert_eq!(rows, 1);
    // A fit's frame has the same shape, its `df` the residual degrees of freedom.
    let fit = fit_with(&table_of("wages"), "log(wage) ~ educ", &Options::default());
    let (columns, rows) = shape(call(
        estimates,
        vec![Some(Value::Fit(Rc::clone(&fit)))],
        Vec::new(),
        Vec::new(),
    ));
    assert_eq!(columns, names);
    assert_eq!(rows, 2);
}

#[test]
fn a_fit_and_a_number_are_refused() {
    let fit = fit_with(&table_of("wages"), "log(wage) ~ educ", &Options::default());
    for (body, name) in [
        (statistic as Body, "statistic"),
        (df, "df"),
        (effect_size, "effect_size"),
        (h0, "h0"),
        (h1, "h1"),
        (alternative, "alternative"),
    ] {
        let failed = try_call(body, Value::Fit(Rc::clone(&fit))).expect_err(name);
        let d = &failed;
        assert_eq!(d.code, "Z0401");
        assert_eq!(
            d.message,
            format!("`{name}` is a method of a test's result, but this is a RegFit")
        );
        assert!(d.help.iter().any(|h| h.contains("`coef`")), "{name}");
    }
    let failed = try_call(statistic, Value::Int(1)).expect_err("a number");
    assert_eq!(
        failed.message,
        "`statistic` is a method of a test's result, but this is an Int"
    );
}

/// Three pairwise comparisons, as `posthoc` gives them: each row an estimate with its standard
/// error and, where `intervals` holds, its interval; the test of the whole an F.
fn comparisons(intervals: bool) -> Rc<TestResult> {
    let row = |term: &str, value: f64, se: f64, p: f64| TermTest {
        term: term.to_string(),
        statistic: Statistic {
            name: "t".to_string(),
            value: value / se,
            df: vec![12.0],
        },
        p_value: p,
        effect: None,
        estimate: Some(Estimate {
            label: term.to_string(),
            value,
            ci: intervals.then_some((value - 2.0 * se, value + 2.0 * se)),
            decimals: 3,
        }),
        std_error: Some(se),
    };
    Rc::new(TestResult {
        follow_up: None,
        terms: vec![
            row("a − b", -1.25, 0.5, 0.09),
            row("a − c", 0.5, 0.25, 0.25),
            row("b − c", 1.75, 0.5, 0.02),
        ],
        method: "posthoc".to_string(),
        title: "posthoc · Bonferroni · v by k".to_string(),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: None,
        estimate: None,
        statistic: Statistic {
            name: "F".to_string(),
            value: 3.5,
            df: vec![2.0, 12.0],
        },
        p_value: 0.0625,
        effect: None,
        level: 0.95,
        alpha: None,
        after: Vec::new(),
        notes: Vec::new(),
        n: 15,
        dropped: 0,
    })
}

#[test]
fn a_result_of_several_rows_gives_each_row_its_estimate_and_its_interval() {
    for intervals in [true, false] {
        let result = comparisons(intervals);
        let Value::Frame(frame) = method(estimates, &result) else {
            panic!("not a frame");
        };
        assert_eq!(frame.table.rows(), 3);
        let text = |row: usize| match frame.table.column("term").unwrap().get(row).unwrap() {
            Cell::Str(s) => s.to_string(),
            other => panic!("not a text: {other:?}"),
        };
        let cell = |name: &str, row: usize| match frame.table.column(name).unwrap().get(row) {
            Some(Cell::Float(v)) => Some(v),
            Some(Cell::Na) => None,
            other => panic!("{name}: {other:?}"),
        };
        assert_eq!([text(0), text(1), text(2)], ["a − b", "a − c", "b − c"]);
        for (i, (value, se, p)) in [(-1.25, 0.5, 0.09), (0.5, 0.25, 0.25), (1.75, 0.5, 0.02)]
            .into_iter()
            .enumerate()
        {
            assert_eq!(cell("estimate", i), Some(value));
            assert_eq!(cell("std_error", i), Some(se));
            assert_eq!(cell("statistic", i), Some(value / se));
            assert_eq!(cell("df", i), Some(12.0));
            assert_eq!(cell("df2", i), None);
            assert_eq!(cell("p_value", i), Some(p));
            assert_eq!(cell("ci_lower", i), intervals.then_some(value - 2.0 * se));
            assert_eq!(cell("ci_upper", i), intervals.then_some(value + 2.0 * se));
        }
        // `ci` gives the interval of each row at the level they were made at, and NA where the
        // method gives none.
        match method(ci, &result) {
            Value::Table(table) => {
                assert!(intervals);
                assert_eq!(table.title, "ci · 95%");
                assert_eq!(
                    table.rows,
                    vec![
                        vec!["a − b", "-2.250", "-0.250"],
                        vec!["a − c", "0.000", "1.000"],
                        vec!["b − c", "0.750", "2.750"],
                    ]
                );
            }
            Value::Na => assert!(!intervals),
            other => panic!("neither a table nor NA: {other:?}"),
        }
        // The single figures stay those of the test of the whole.
        assert_eq!(method(statistic, &result), Value::Float(3.5));
        assert_eq!(
            method(df, &result),
            Value::List(vec![Value::Float(2.0), Value::Float(12.0)])
        );
        assert_eq!(method(effect_size, &result), Value::Na);
    }
    // Another level is refused, with the help that fits the test.
    let at_90 = |test: &Rc<TestResult>| {
        try_with(
            ci,
            vec![
                Some(Value::TestResult(Rc::clone(test))),
                None,
                Some(Value::Float(0.9)),
            ],
        )
    };
    let failed = at_90(&comparisons(true)).expect_err("another level");
    assert_eq!(failed.code, "Z0401");
    assert_eq!(
        failed.message,
        "`ci` of a test's result gives the intervals the test was made at, 95%, not at 90%"
    );
    assert_eq!(
        failed.help,
        ["`posthoc` gives its intervals at 95% and takes no `level:`"]
    );
    let failed = at_90(&welch()).expect_err("another level");
    assert_eq!(
        failed.help,
        ["run the test with `level:` for intervals at another level"]
    );
    // Where there is no interval to give, the answer is NA whatever the level.
    assert_eq!(at_90(&comparisons(false)).unwrap(), Value::Na);
}

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

#[test]
fn every_method_through_the_engine() {
    let scripts = [
        // Every method on a two-group t test, and the sentence of the definition of done.
        "let wages = dataset(wages).add(lwage = log(wage))\nlet t = wages.t_test(lwage, group: female)\nt.statistic()\nt.df()\nt.effect_size()\nt.h0()\nt.h1()\nt.alternative()\nt.p_values()\nt.n_obs()\nt.estimates()\nt.ci()\njoin([\"t(\", format(t.df()[1], digits: 1), \") = \", format(t.statistic(), digits: 2), \", p \", format(t.p_values(), digits: 3), \", g = \", format(t.effect_size(), digits: 3)])\ntype_of(t.df())\ntype_of(t.h0())\n",
        // A chi-squared test: one degree of freedom, no estimate, no effect size.
        "let c = dataset(wages).chi2_test(union, female)\nc.statistic()\nc.df()\nc.effect_size()\nc.h0()\nc.h1()\nc.alternative()\nc.estimates()\n",
        // Kruskal–Wallis, and the directional forms.
        "let wages = dataset(wages).add(lwage = log(wage))\nlet k = wages.kruskal_test(lwage, group: industry)\nk.statistic()\nk.df()\nk.h1()\nk.estimates()\nwages.t_test(lwage, group: female, alternative: greater).alternative()\nwages.t_test(lwage, group: female, alternative: less).h1()\n",
        // A Wald test: two degrees of freedom, no hypotheses; and the fit's own frame.
        "let m = dataset(wages).reg(log(wage) ~ educ + female)\nlet w = m.test(educ == 0)\nw.df()\nw.h0()\nw.alternative()\nw.estimates()\nm.estimates()\n",
        // An exact test has no statistic.
        "let f = dataset(wages).fisher_test(union, female)\nf.statistic()\nf.df()\nf.estimates()\n",
        // The refusals: a fit, and a field read on a result.
        "let m = dataset(wages).reg(log(wage) ~ educ)\nm.df()\n",
        "let m = dataset(wages).reg(log(wage) ~ educ)\nstatistic(m)\n",
        "let t = dataset(wages).t_test(wage, group: female)\nt.statistic\n",
        "let m = dataset(wages).reg(log(wage) ~ educ)\nm.coef\n",
        "statistic(3)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(script)));
    }
    insta::assert_snapshot!("fields", all);
}
