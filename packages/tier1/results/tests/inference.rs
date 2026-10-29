//! `test`, `compare` and `derive` (Language Reference, sections 3.7, 4 and 7.2; Engine and
//! Project Specification, section 13; release 0.4.0, step 18; questions 3 (b) and (c)): the
//! three against R on the fits of the `inference` block of `tests/reference/r/regression.json`
//! (the Wald F written out in base R under each fit's variance matrix, `anova` on nested classical
//! fits and the Wald F of the extra terms otherwise, the delta method through `deriv`), then the
//! shapes a restriction and a definition may take, the printed forms and every message through
//! the engine.

mod common;

use common::{NoFiles, Options, close, fit_with, json, number, table_of, text};
use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::rc::Rc;
use zeff_core::display::{fixed, p_value_text};
use zeff_core::operators::Op;
use zeff_core::runtime::{Args, Body, CallContext, Randomness};
use zeff_core::value::{ColumnExpr, Fit, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_results::inference::{compare, delta, derive, test};
use zeff_testkit::run;

/// A result method called with the receiver, the other arguments, the rest values and the
/// definitions given.
fn call_with(
    body: Body,
    fit: &Rc<Fit>,
    args: Vec<Option<Value>>,
    rest: Vec<Value>,
    definitions: Vec<(String, Value)>,
) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", "m.method()"));
    let mut values = vec![Some(Value::Fit(Rc::clone(fit)))];
    values.extend(args);
    let spans = vec![Some(Span::new(0, 1)); values.len()];
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "m.method()",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 8),
        arg_spans: spans,
        rest_spans: vec![Span::new(9, 10); rest.len()],
        definition_spans: vec![Span::new(9, 10); definitions.len()],
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    match body(
        &mut ctx,
        Args {
            values,
            rest,
            definitions,
        },
    ) {
        Ok(value) => value,
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// The reference fit `key`, bound to its key as a script's `let` would bind it.
fn fit_named(reference: &Json, key: &str) -> Rc<Fit> {
    let info = &reference["inference"]["fits"][key];
    let table = table_of(text(&info["data"]));
    let cluster = info["cluster"].as_str().unwrap_or("");
    let options = Options {
        se: match text(&info["se"]) {
            "classical" => Some("classical"),
            "HC1" => Some("robust"),
            "HC2" => Some("hc2"),
            "HC3" => Some("hc3"),
            _ => None,
        },
        cluster: (!cluster.is_empty()).then_some(cluster),
        level: Some(0.95),
        ..Default::default()
    };
    let mut fit = fit_with(&table, text(&info["formula"]), &options);
    let terms: Vec<&str> = info["terms"].as_array().unwrap().iter().map(text).collect();
    assert_eq!(fit.terms, terms, "{key}: the terms of R and Zeff");
    Rc::make_mut(&mut fit).name = Some(key.to_string());
    fit
}

// ----- expressions over the terms, as the interpreter hands them to the functions -----

fn var(name: &str) -> ColumnExpr {
    ColumnExpr::Var(name.to_string())
}

fn num(x: f64) -> ColumnExpr {
    ColumnExpr::Float(x)
}

fn int(i: i64) -> ColumnExpr {
    ColumnExpr::Int(i)
}

fn bin(op: Op, lhs: ColumnExpr, rhs: ColumnExpr) -> ColumnExpr {
    ColumnExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn neg(operand: ColumnExpr) -> ColumnExpr {
    ColumnExpr::Unary {
        negate: true,
        operand: Box::new(operand),
    }
}

fn call(name: &str, arg: ColumnExpr) -> ColumnExpr {
    ColumnExpr::Call {
        package: "core".to_string(),
        name: name.to_string(),
        args: vec![Some(arg)],
        rest: Vec::new(),
        lifted: false,
        returns: "Float".to_string(),
    }
}

fn term_expr(expr: ColumnExpr, text: &str) -> Value {
    Value::TermExpr {
        expr,
        text: text.to_string(),
    }
}

/// A restriction of the reference, `Σ coef · term == constant`, in the mechanical form.
fn restriction(case: &Json) -> Value {
    let mut lhs: Option<ColumnExpr> = None;
    for (term, coef) in case["coefs"].as_object().unwrap() {
        let part = bin(Op::Mul, num(number(coef)), var(term));
        lhs = Some(match lhs {
            Some(sum) => bin(Op::Add, sum, part),
            None => part,
        });
    }
    term_expr(
        bin(
            Op::Eq,
            lhs.expect("a coefficient"),
            num(number(&case["constant"])),
        ),
        text(&case["text"]),
    )
}

/// The definitions of the reference by name, as the checker lowers them.
fn definition(name: &str) -> ColumnExpr {
    match name {
        "ratio" => bin(Op::Div, var("educ"), var("exper")),
        "gain" => bin(
            Op::Sub,
            call("exp", bin(Op::Mul, var("educ"), int(4))),
            int(1),
        ),
        "root" => call("sqrt", var("educ")),
        "logged" => call("log", bin(Op::Div, var("educ"), var("exper"))),
        "square" => bin(Op::Pow, var("educ"), int(2)),
        "product" => bin(
            Op::Add,
            bin(Op::Mul, var("educ"), var("female")),
            var("exper"),
        ),
        "scaled" => bin(Op::Sub, bin(Op::Mul, int(100), var("educ")), int(2)),
        "tenth" => call("log10", var("educ")),
        "l1p" => call("log1p", var("exper")),
        "e1" => call("expm1", var("educ")),
        "l2" => call("log2", var("educ")),
        "neg" => bin(
            Op::Div,
            neg(var("educ")),
            bin(Op::Sub, var("exper"), var("female")),
        ),
        "power" => bin(Op::Pow, bin(Op::Add, var("educ"), var("exper")), num(1.5)),
        "ratio2" => bin(Op::Div, var("exper"), var("exper^2")),
        "turning" => bin(
            Op::Div,
            neg(var("exper")),
            bin(Op::Mul, int(2), var("exper^2")),
        ),
        "years" => bin(Op::Div, var("tenure"), var("exper")),
        other => panic!("no definition `{other}` in the test"),
    }
}

fn test_result(value: Value) -> Rc<zeff_core::value::TestResult> {
    match value {
        Value::TestResult(t) => t,
        other => panic!("not a test's result: {other:?}"),
    }
}

#[test]
fn test_agrees_with_the_wald_f_written_out_in_r() {
    let reference = json("r/regression.json");
    let mut fits: BTreeMap<String, Rc<Fit>> = BTreeMap::new();
    let cases = reference["inference"]["tests"].as_array().unwrap();
    assert!(cases.len() >= 14, "{} tests", cases.len());
    for case in cases {
        let key = text(&case["fit"]);
        let fit = fits
            .entry(key.to_string())
            .or_insert_with(|| fit_named(&reference, key));
        let restrictions = case["restrictions"].as_array().unwrap();
        let rest: Vec<Value> = restrictions.iter().map(restriction).collect();
        let texts: Vec<&str> = restrictions.iter().map(|r| text(&r["text"])).collect();
        let what = format!("{key}: {}", texts.join(", "));
        let result = test_result(call_with(test, fit, vec![], rest, vec![]));
        let (f, p) = (number(&case["f"]), number(&case["p"]));
        assert!(
            close(result.statistic.value, f, 1e-10, 0.0),
            "{what}: F {} against R's {f}",
            result.statistic.value
        );
        assert!(
            close(result.p_value, p, 1e-9, 1e-300),
            "{what}: p {} against R's {p}",
            result.p_value
        );
        assert_eq!(result.statistic.name, "F", "{what}");
        assert_eq!(
            result.statistic.df,
            vec![number(&case["q"]), number(&case["df"])],
            "{what}"
        );
        assert_eq!(result.title, format!("test · {key} · {}", texts.join(", ")));
        assert_eq!(result.method, "test");
        assert_eq!(result.n, fit.n, "{what}");
    }
}

#[test]
fn compare_agrees_with_anova_and_the_wald_f_of_the_extra_terms() {
    let reference = json("r/regression.json");
    let cases = reference["inference"]["compare"].as_array().unwrap();
    assert!(cases.len() >= 5, "{} comparisons", cases.len());
    for case in cases {
        let (small_key, big_key) = (text(&case["small"]), text(&case["big"]));
        let small = fit_named(&reference, small_key);
        let big = fit_named(&reference, big_key);
        let what = format!("{small_key} in {big_key}");
        let (f, p) = (number(&case["f"]), number(&case["p"]));
        // The receiver may be either model.
        for (a, b) in [(&big, &small), (&small, &big)] {
            let result = test_result(call_with(
                compare,
                a,
                vec![Some(Value::Fit(Rc::clone(b)))],
                vec![],
                vec![],
            ));
            assert!(
                close(result.statistic.value, f, 1e-10, 0.0),
                "{what}: F {} against R's {f}",
                result.statistic.value
            );
            assert!(
                close(result.p_value, p, 1e-9, 1e-300),
                "{what}: p {} against R's {p}",
                result.p_value
            );
            assert_eq!(
                result.statistic.df,
                vec![number(&case["q"]), number(&case["df"])],
                "{what}"
            );
            assert_eq!(
                result.title,
                format!(
                    "compare · {} against {}",
                    a.name.as_deref().unwrap(),
                    b.name.as_deref().unwrap()
                )
            );
            let table = result.table.as_ref().expect("the table of the two models");
            assert_eq!(table.rows.len(), 2, "{what}");
            assert_eq!(table.rows[0][0], a.name.clone().unwrap(), "{what}");
            assert_eq!(table.rows[1][0], b.name.clone().unwrap(), "{what}");
            assert_eq!(result.method, "compare");
        }
    }
}

#[test]
fn derive_agrees_with_the_delta_method_through_deriv() {
    let reference = json("r/regression.json");
    let mut fits: BTreeMap<String, Rc<Fit>> = BTreeMap::new();
    let cases = reference["inference"]["derive"].as_array().unwrap();
    assert!(cases.len() >= 40, "{} definitions", cases.len());
    for case in cases {
        let key = text(&case["fit"]);
        let fit = fits
            .entry(key.to_string())
            .or_insert_with(|| fit_named(&reference, key));
        let name = text(&case["name"]);
        let what = format!("{key}: {name} = {}", text(&case["text"]));
        let expr = definition(name);
        let (value, se) = delta(fit, &expr).unwrap_or_else(|_| panic!("{what}: refused"));
        let (r_value, r_se) = (number(&case["value"]), number(&case["se"]));
        assert!(
            close(value, r_value, 1e-10, 0.0),
            "{what}: {value} against R's {r_value}"
        );
        assert!(
            close(se, r_se, 1e-10, 0.0),
            "{what}: standard error {se} against R's {r_se}"
        );
        // The printed row, in the coefficient table's columns (question 3 (c)).
        let table = match call_with(
            derive,
            fit,
            vec![],
            vec![],
            vec![(name.to_string(), term_expr(expr, text(&case["text"])))],
        ) {
            Value::Table(t) => t,
            other => panic!("{what}: not a table: {other:?}"),
        };
        assert_eq!(table.title, format!("derive · {key}"));
        let headings: Vec<&str> = table.columns.iter().map(|(h, _)| h.as_str()).collect();
        assert_eq!(
            headings,
            [
                "term",
                "estimate",
                "std.error",
                "t value",
                "p value",
                "95% CI"
            ],
            "{what}"
        );
        let expected = vec![
            name.to_string(),
            fixed(r_value, 4),
            fixed(r_se, 4),
            fixed(number(&case["t"]), 2),
            p_value_text(number(&case["p"])),
            format!(
                "[{}, {}]",
                fixed(number(&case["lower"]), 4),
                fixed(number(&case["upper"]), 4)
            ),
        ];
        assert_eq!(table.rows, vec![expected], "{what}");
    }
}

#[test]
fn a_restriction_in_any_linear_shape_gives_the_same_test() {
    let reference = json("r/regression.json");
    let fit = fit_named(&reference, "A");
    let f_of = |expr: ColumnExpr| {
        test_result(call_with(
            test,
            &fit,
            vec![],
            vec![term_expr(expr, "r")],
            vec![],
        ))
        .statistic
        .value
    };
    let plain = f_of(bin(Op::Eq, var("educ"), var("exper")));
    let shapes = [
        bin(Op::Eq, var("exper"), var("educ")),
        bin(Op::Eq, bin(Op::Sub, var("educ"), var("exper")), int(0)),
        bin(Op::Eq, int(0), bin(Op::Sub, var("educ"), var("exper"))),
        bin(
            Op::Eq,
            bin(Op::Mul, int(2), var("educ")),
            bin(Op::Mul, var("exper"), int(2)),
        ),
        bin(
            Op::Eq,
            bin(Op::Div, var("educ"), int(2)),
            bin(Op::Div, var("exper"), int(2)),
        ),
        bin(Op::Eq, neg(var("educ")), neg(var("exper"))),
        bin(
            Op::Eq,
            bin(Op::Add, var("educ"), num(0.5)),
            bin(Op::Add, var("exper"), bin(Op::Pow, num(0.5), int(1))),
        ),
    ];
    for shape in shapes {
        let f = f_of(shape);
        assert!(close(f, plain, 1e-12, 0.0), "{f} against {plain}");
    }
    // A single restriction is the square of the term's t statistic.
    let i = fit.terms.iter().position(|t| t == "educ").unwrap();
    let t = fit.estimates[i] / fit.std_errors[i];
    let one = f_of(bin(Op::Eq, var("educ"), int(0)));
    assert!(close(one, t * t, 1e-10, 0.0), "{one} against t² {}", t * t);
}

#[test]
fn the_gradient_of_every_function_derive_knows() {
    let reference = json("r/regression.json");
    let fit = fit_named(&reference, "A");
    let i = fit.terms.iter().position(|t| t == "educ").unwrap();
    let b = fit.estimates[i];
    let v = fit.vcov[i * fit.terms.len() + i];
    // Each function's value and slope at the estimate, the standard error being |f'(b)| √v.
    type Slope = fn(f64) -> f64;
    let functions: [(&str, Slope, Slope); 8] = [
        ("log", f64::ln, |x| 1.0 / x),
        ("exp", f64::exp, f64::exp),
        ("sqrt", f64::sqrt, |x| 0.5 / x.sqrt()),
        ("log1p", f64::ln_1p, |x| 1.0 / (1.0 + x)),
        ("expm1", f64::exp_m1, f64::exp),
        ("log10", f64::log10, |x| 1.0 / (x * std::f64::consts::LN_10)),
        ("log2", f64::log2, |x| 1.0 / (x * std::f64::consts::LN_2)),
        ("abs", f64::abs, f64::signum),
    ];
    for (name, value, slope) in functions {
        let (got, se) = delta(&fit, &call(name, var("educ"))).unwrap_or_else(|_| panic!("{name}"));
        assert!(close(got, value(b), 1e-12, 0.0), "{name}: {got}");
        let expected = slope(b).abs() * v.sqrt();
        assert!(
            close(se, expected, 1e-12, 0.0),
            "{name}: {se} against {expected}"
        );
    }
    // A coefficient raised to another: b^c with both gradients.
    let j = fit.terms.iter().position(|t| t == "exper").unwrap();
    let c = fit.estimates[j];
    let (got, _) =
        delta(&fit, &bin(Op::Pow, var("educ"), var("exper"))).unwrap_or_else(|_| panic!("pow"));
    assert!(close(got, b.powf(c), 1e-12, 0.0));
    let k = fit.terms.len();
    let mut g = vec![0.0; k];
    g[i] = c * b.powf(c - 1.0);
    g[j] = b.powf(c) * b.ln();
    let mut variance = 0.0;
    for a in 0..k {
        for b2 in 0..k {
            variance += g[a] * fit.vcov[a * k + b2] * g[b2];
        }
    }
    let (_, se) =
        delta(&fit, &bin(Op::Pow, var("educ"), var("exper"))).unwrap_or_else(|_| panic!("pow"));
    assert!(
        close(se, variance.sqrt(), 1e-12, 0.0),
        "{se} against {}",
        variance.sqrt()
    );
}

// ----- the printed forms and every message, through the engine -----

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

const M: &str =
    "let wages = dataset(wages)\nlet m = wages.reg(log(wage) ~ educ + exper + female)\n";

#[test]
fn the_printed_forms_and_every_message() {
    let scripts = [
        format!("{M}m.test(educ == exper)\nm.test(educ + exper == 0.1, female == -0.1)\nm.test(educ == exper, female == 0, educ + female == exper)\nm.test(`(intercept)` == 1)\nlet t = m.test(educ == 0.1 ^ 2)\nt\nt.p_values()\ntype_of(t)\nlet educ = 3\nm.test(educ == exper)\nwages.reg(log(wage) ~ educ + exper + female).test(educ == 0)\n"),
        "let wages = dataset(wages)\nlet m1 = wages.reg(log(wage) ~ educ + exper)\nlet m2 = wages.reg(log(wage) ~ educ + exper + female)\nm2.compare(m1)\nm1.compare(m2)\nlet c = m2.compare(m1)\nc.p_values()\nc.summary()\ntype_of(c)\nwages.reg(log(wage) ~ educ + exper + female).compare(m1)\n".to_string(),
        "let wages = dataset(wages)\nlet r = wages.reg(log(wage) ~ educ + exper + female, se: robust)\nlet r1 = wages.reg(log(wage) ~ educ + exper, se: robust)\nr.test(educ == exper)\nr.compare(r1)\nr.derive(ratio = educ / exper)\nlet c = wages.reg(log(wage) ~ educ + exper + female, cluster: worker)\nc.test(educ == exper, female == 0)\nc.derive(ratio = educ / exper)\nc.compare(wages.reg(log(wage) ~ educ + exper, cluster: worker))\n".to_string(),
        format!("{M}m.derive(ratio = educ / exper, gain = exp(educ * 4) - 1)\nm.derive(a = abs(female), b = -educ, c = educ ^ exper, d = (educ + exper) ^ 1.5, k = 2)\nm.derive(s = educ / 0, l = log(female))\nm.derive(r = educ / exper).export(\"output/derive.tex\")\nwages.reg(log(wage) ~ educ + exper + female).derive(r = educ / exper)\n"),
        // An omitted term has no coefficient; a weighted fit tests under its own errors.
        "let w = dataset(wages).add(twice = educ * 2)\nlet m = w.reg(log(wage) ~ educ + twice + female)\nm.test(educ == 0)\nm.derive(r = educ / female)\nlet f = dataset(wages).reg(log(wage) ~ educ + exper + female, weights: hours)\nf.test(educ == exper)\nf.derive(r = educ / exper)\n".to_string(),
        "let w = dataset(wages).add(twice = educ * 2)\nlet m = w.reg(log(wage) ~ educ + twice + female)\nm.test(twice == 0)\n".to_string(),
        "let w = dataset(wages).add(twice = educ * 2)\nlet m = w.reg(log(wage) ~ educ + twice + female)\nm.derive(r = twice / educ)\n".to_string(),
        format!("{M}m.test(educ / exper == 1)\n"),
        format!("{M}m.test(educ == exper, 2 * educ == 2 * exper)\n"),
        format!("{M}m.test(hours == 0)\n"),
        format!("{M}m.test(exper == 0, femal == 0)\n"),
        format!("{M}m.test(true)\n"),
        format!("{M}m.test()\n"),
        format!("{M}m.test(educ == exper && female == 0)\n"),
        format!("{M}m.test(educ != exper)\n"),
        format!("{M}m.test(educ * exper == 0)\n"),
        format!("{M}m.test(1 == 1)\n"),
        format!("{M}m.test(educ / 0 == 1)\n"),
        format!("{M}m.test(educ ^ 2 == 1)\n"),
        format!("{M}m.test(exp(educ) == 1)\n"),
        format!("{M}m.test(educ)\n"),
        format!("{M}m.test(\"educ\" == 1)\n"),
        format!("{M}m.test(educ == exper).test(educ == 0)\n"),
        format!("{M}m.compare(m)\n"),
        format!("{M}m.compare(m.test(educ == 0))\n"),
        format!("{M}wages.reg(log(wage) ~ educ + hours).compare(m)\n"),
        format!("{M}wages.where(educ > 10).reg(log(wage) ~ educ + exper + female).compare(m)\n"),
        format!("{M}wages.reg(log(wage) ~ educ + exper + exper^2).compare(wages.reg(log(wage) ~ educ + exper + educ^2))\n"),
        format!("{M}m.derive(r = round(educ / exper))\n"),
        format!("{M}m.derive(r = min(educ, exper))\n"),
        format!("{M}m.derive(r = educ > exper)\n"),
        format!("{M}m.derive()\n"),
        format!("{M}m.derive(r = educ / exper, r = educ)\n"),
        format!("{M}m.derive(r = educ / nope)\n"),
        "dataset(wages).test(educ == 0)\n".to_string(),
        "dataset(wages).derive(r = educ)\n".to_string(),
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(&script)));
    }
    insta::assert_snapshot!("inference", all);
}

// ----- the survivors of the weekly mutation run (release 0.4.0, step 22a) -----

/// Every linear form of a restriction gives the Wald F of its row and constant: a coefficient
/// scaled by a constant on one side, negated, shifted by a constant on one side, or divided by
/// a constant, each checked against the F computed by hand from the estimates and the variance.
#[test]
fn restrictions_in_every_linear_form_give_the_wald_f_of_their_row() {
    let fit = fit_with(
        &table_of("wages"),
        "log(wage) ~ educ + exper",
        &Options {
            level: Some(0.95),
            ..Default::default()
        },
    );
    let k = fit.terms.len();
    let by_hand = |r: &[f64], c: f64| {
        let rb = r
            .iter()
            .zip(&fit.estimates)
            .map(|(x, b)| x * b)
            .sum::<f64>()
            - c;
        let rvr: f64 = (0..k)
            .flat_map(|i| (0..k).map(move |j| (i, j)))
            .map(|(i, j)| r[i] * fit.vcov[i * k + j] * r[j])
            .sum();
        rb * rb / rvr
    };
    let cases: Vec<(ColumnExpr, &str, Vec<f64>, f64)> = vec![
        (
            bin(Op::Eq, bin(Op::Mul, int(2), var("educ")), var("exper")),
            "2 * educ == exper",
            vec![0.0, 2.0, -1.0],
            0.0,
        ),
        (
            bin(Op::Eq, neg(var("educ")), var("exper")),
            "-educ == exper",
            vec![0.0, -1.0, -1.0],
            0.0,
        ),
        (
            bin(Op::Eq, bin(Op::Add, var("educ"), num(0.1)), var("exper")),
            "educ + 0.1 == exper",
            vec![0.0, 1.0, -1.0],
            -0.1,
        ),
        (
            bin(Op::Eq, bin(Op::Sub, var("educ"), num(0.1)), var("exper")),
            "educ - 0.1 == exper",
            vec![0.0, 1.0, -1.0],
            0.1,
        ),
        (
            bin(Op::Eq, bin(Op::Div, var("educ"), int(2)), var("exper")),
            "educ / 2 == exper",
            vec![0.0, 0.5, -1.0],
            0.0,
        ),
        (
            bin(
                Op::Eq,
                bin(Op::Mul, int(2), bin(Op::Add, var("educ"), num(0.1))),
                var("exper"),
            ),
            "2 * (educ + 0.1) == exper",
            vec![0.0, 2.0, -1.0],
            -0.2,
        ),
        (
            bin(
                Op::Eq,
                bin(Op::Div, bin(Op::Sub, var("educ"), num(0.1)), int(2)),
                var("exper"),
            ),
            "(educ - 0.1) / 2 == exper",
            vec![0.0, 0.5, -1.0],
            0.05,
        ),
    ];
    for (expr, text, r, c) in cases {
        let result = test_result(call_with(
            test,
            &fit,
            vec![],
            vec![term_expr(expr, text)],
            vec![],
        ));
        let expected = by_hand(&r, c);
        assert!(
            close(result.statistic.value, expected, 1e-10, 0.0),
            "{text}: F {} against {expected}",
            result.statistic.value
        );
        assert_eq!(result.statistic.df[0], 1.0, "{text}");
    }
}
