//! `reg` (Language Reference, sections 7.2 and 10.2; Engine and Project Specification, section
//! 13; release 0.4.0, step 10): against R's `lm`, `summary` and `confint` on the shipped `wages`,
//! `wages_large` and the edge-case files (`tests/reference/r/regression.json`); the exact small
//! cases of `tests/reference/kernels.json`; the edge cases; the properties of scaling, row
//! order, duplicates and block size; and the summary with every note and error.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_regress::ols::Ols;
use zeff_results::design::Design;
use zeff_results::estimator::Estimator;
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("reg reads no file")
    }
}

/// `reg` through its body: the fit, with the messages, or the error rendered.
fn fit_of(table: &Table, text: &str, level: Option<f64>) -> Result<(Fit, Vec<String>), String> {
    let f = formula(text);
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.reg({text})\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(10, 13),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(14, 14 + text.len()))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(f)),
            None,
            level.map(Value::Float),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let result = zeff_regress::reg::reg(&mut ctx, args);
    let rendered: Vec<String> = messages.iter().map(|m| m.render(&sources)).collect();
    match result {
        Ok(Value::Fit(fit)) => Ok(((*fit).clone(), rendered)),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => Err(e.render(&sources)),
    }
}

fn dataset(file: &str) -> Table {
    let path = root().join("tests/datasets").join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        other => dataset(other),
    }
}

fn json(file: &str) -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference").join(file)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(value: &Json) -> f64 {
    match value {
        Json::Number(n) => n.as_f64().unwrap(),
        Json::String(s) => match s.as_str() {
            "NA" => f64::NAN,
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            t => t.parse().unwrap(),
        },
        other => panic!("not a number: {other}"),
    }
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}

fn floats(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

fn table(columns: Vec<(&str, Column)>) -> Table {
    Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap()
}

// ----- the reference: R's lm -----

#[test]
fn fits_agree_with_r() {
    let reference = json("r/regression.json");
    for (name, case) in reference["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
        let level = number(&case["level"]);
        let (fit, _) = fit_of(&table, text, Some(level)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let terms: Vec<String> = case["terms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();
        let omitted: Vec<String> = case["omitted"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();
        // R orders terms by degree; Zeff keeps the order written, so the sets are compared and
        // the values matched by name.
        let sorted = |mut v: Vec<String>| {
            v.sort();
            v
        };
        assert_eq!(
            sorted(fit.terms.clone()),
            sorted(terms),
            "{name}: the terms"
        );
        assert_eq!(
            sorted(fit.omitted.clone()),
            sorted(omitted),
            "{name}: the omitted terms"
        );
        assert_eq!(fit.n as u64, case["nobs"].as_u64().unwrap(), "{name}: N");
        assert_eq!(
            fit.df_residual as u64,
            case["df_residual"].as_u64().unwrap(),
            "{name}: df"
        );
        let columns = case["columns"].as_object().unwrap();
        for (i, term) in fit.terms.iter().enumerate() {
            let r = &columns[term];
            let check = |what: &str, zeff: f64, key: &str, relative: f64, absolute: f64| {
                let r_value = number(&r[key]);
                assert!(
                    close(zeff, r_value, relative, absolute),
                    "{name}: {term} {what}: Zeff {zeff:e}, R {r_value:e}"
                );
            };
            check("estimate", fit.estimates[i], "estimate", 1e-10, 1e-14);
            check("std.error", fit.std_errors[i], "std_error", 1e-10, 1e-14);
            check("t value", fit.statistics[i], "t_value", 1e-10, 1e-12);
            check("p value", fit.p_values[i], "p_value", 1e-8, 1e-14);
            check("ci lower", fit.intervals[i].0, "ci_lower", 1e-10, 1e-14);
            check("ci upper", fit.intervals[i].1, "ci_upper", 1e-10, 1e-14);
        }
        let measure = |what: &str, zeff: f64, key: &str| {
            let r_value = number(&case[key]);
            assert!(
                close(zeff, r_value, 1e-10, 1e-14),
                "{name}: {what}: Zeff {zeff:e}, R {r_value:e}"
            );
        };
        measure("R²", fit.r_squared.unwrap(), "r_squared");
        measure(
            "adjusted R²",
            fit.adjusted_r_squared.unwrap(),
            "adj_r_squared",
        );
        measure("sigma", fit.sigma, "sigma");
        measure(
            "residual sum of squares",
            fit.residual_sum_of_squares,
            "rss",
        );
        measure("log-likelihood", fit.log_likelihood, "log_likelihood");
    }
}

/// The exact least-squares cases of `kernels.py`, each a small matrix with its rational
/// solution: fitted without an intercept on the matrix's own columns.
#[test]
fn the_exact_small_cases_fit_within_rounding() {
    let reference = json("kernels.json");
    for (name, case) in reference["least_squares"].as_object().unwrap() {
        let x: Vec<Vec<f64>> = case["x"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row.as_array().unwrap().iter().map(number).collect())
            .collect();
        let y: Vec<f64> = case["y"].as_array().unwrap().iter().map(number).collect();
        let rows = x.len();
        let cols = x[0].len();
        let mut columns: Vec<(String, Column)> = vec![("y".to_string(), floats(&y))];
        for c in 0..cols {
            let values: Vec<f64> = x.iter().map(|row| row[c]).collect();
            columns.push((format!("c{c}"), floats(&values)));
        }
        let t = Table::new(columns).unwrap();
        let text = format!(
            "y ~ {} - 1",
            (0..cols)
                .map(|c| format!("c{c}"))
                .collect::<Vec<_>>()
                .join(" + ")
        );
        let kept: Vec<usize> = case["kept"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let rank = case["rank"].as_u64().unwrap() as usize;
        match fit_of(&t, &text, None) {
            Ok((fit, _)) => {
                assert!(
                    rows > rank,
                    "{name}: a fit with no residual degrees of freedom"
                );
                let expected_terms: Vec<String> = kept.iter().map(|c| format!("c{c}")).collect();
                assert_eq!(fit.terms, expected_terms, "{name}: the kept columns");
                // A column left out has no coefficient (`null`).
                let coefficients: Vec<f64> = case["coefficients"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| if v.is_null() { f64::NAN } else { number(v) })
                    .collect();
                // A near-collinear design carries its condition number into the solution.
                let relative = (1e-13 * fit.condition_number).max(1e-12);
                for (i, &c) in kept.iter().enumerate() {
                    assert!(
                        close(fit.estimates[i], coefficients[c], relative, 1e-13),
                        "{name}: c{c}: {} against {}",
                        fit.estimates[i],
                        coefficients[c]
                    );
                }
                let rss = number(&case["residual_sum_of_squares"]);
                assert!(
                    close(fit.residual_sum_of_squares, rss, 1e-10, 1e-12),
                    "{name}: rss {} against {rss}",
                    fit.residual_sum_of_squares
                );
                let leverage: Vec<f64> = case["leverage"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .collect();
                for (i, (z, r)) in fit.leverage.iter().zip(&leverage).enumerate() {
                    assert!(
                        close(*z, *r, 1e-10, 1e-12),
                        "{name}: leverage {i}: {z} against {r}"
                    );
                }
            }
            Err(e) => {
                assert!(rows <= rank, "{name}: refused, {e}");
                assert!(e.contains("Z0802"), "{name}: {e}");
            }
        }
    }
}

// ----- the summary, the edge cases and every message -----

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

#[test]
fn the_summary_and_the_edge_cases() {
    let scripts = [
        "let m = dataset(wages).reg(log(wage) ~ educ + exper + exper^2 + female)\nm\ntype_of(m)\nm.is(Result)\nm.is(RegFit)\n[m]\n",
        "dataset(wages).reg(log(wage) ~ category(industry, contrast: sum) + educ, level: 0.9)\n",
        "dataset(wages).reg(wage ~ educ + exper - 1)\n",
        "let w = import(\"data/wages_large.csv\")\nw.reg(wage ~ educ + hours + (educ * 2))\n",
        "dataset(wages).add(one = 1).reg(wage ~ educ + one)\n",
        "dataset(wages).reg(wage ~ educ + exper + educ)\n",
        "dataset(wages).head(3).reg(wage ~ educ + exper + female)\n",
        "dataset(wages).head(1).reg(wage ~ educ)\n",
        "dataset(wages).add(c = 5).reg(c ~ educ)\n",
        "dataset(wages).where(wage > 1e9).reg(wage ~ educ)\n",
        "let w = import(\"data/wages_large.csv\")\nw.where(is_missing(hours)).reg(wage ~ hours)\n",
        "dataset(wages).reg(log(wage) ~ exper + exper^2 + exper^3 + exper^4 + exper^5)\n",
        "dataset(wages).reg(wage ~ educ, level: 1.5)\n",
        "dataset(wages).reg(wage ~ educ, se: robust)\n",
        "dataset(wages).reg(wage ~ region)\n",
        "let w = import(\"data/wages_large.csv\")\nw.reg(wage ~ region)\n",
        "dataset(wages).reg(industry ~ educ)\n",
        "let f = log(wage) ~ educ\ndataset(wages).reg(f + female - 1)\n",
        // Near-singular without any column being dropped: `y` is `a` plus a little of `u`, and `z`
        // is `u` plus a little of `v`, so a large combination of the three is almost zero while each
        // column keeps a tenth of a thousandth of its norm after the ones before it.
        "let d = frame(a = [1.1, -0.7, 2.3, 0.4, -1.9, 0.8, 1.5, -2.2, 0.3, 1.7, -0.5, 2.9], u = [0.3, 1.4, -0.8, 2.1, 0.6, -1.3, 0.9, 0.2, -1.7, 1.1, 2.4, -0.4], v = [-1.2, 0.5, 0.7, -0.3, 1.8, 0.1, -0.9, 1.3, 0.6, -2.1, 0.8, 1.0], w = [2.0, 1.0, 3.0, 0.5, 1.5, 2.5, 3.5, 0.0, 1.2, 2.2, 0.7, 1.9])\nlet e = d.add(y = a + 1e-5 * u, z = u + 1e-5 * v)\ne.reg(w ~ a + y + z)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(script)));
    }
    insta::assert_snapshot!("summary_and_edge_cases", all);
}

// ----- properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A small frame of an outcome and two regressors from random values, with enough rows.
fn frame_of(values: &[(f64, f64, f64)]) -> Table {
    table(vec![
        ("y", floats(&values.iter().map(|v| v.0).collect::<Vec<_>>())),
        ("a", floats(&values.iter().map(|v| v.1).collect::<Vec<_>>())),
        ("b", floats(&values.iter().map(|v| v.2).collect::<Vec<_>>())),
    ])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(25)))]

    /// Doubling the outcome doubles every estimate to the bit, and scaling it by three scales
    /// them within rounding; the t statistics do not move.
    #[test]
    fn scaling_the_outcome_scales_the_coefficients(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..40)) {
        let t = frame_of(&values);
        let (base, _) = fit_of(&t, "y ~ a + b", None).unwrap();
        let (doubled, _) = fit_of(&t, "(y * 2) ~ a + b", None).unwrap();
        for (b, d) in base.estimates.iter().zip(&doubled.estimates) {
            prop_assert_eq!((b * 2.0).to_bits(), d.to_bits());
        }
        let (tripled, _) = fit_of(&t, "(y * 3) ~ a + b", None).unwrap();
        for ((b, d), (tb, td)) in base.estimates.iter().zip(&tripled.estimates).zip(base.statistics.iter().zip(&tripled.statistics)) {
            prop_assert!(close(b * 3.0, *d, 1e-12, 1e-14));
            prop_assert!(close(*tb, *td, 1e-10, 1e-12));
        }
    }

    /// Rescaling a regressor by a million changes no t statistic, p-value or R².
    #[test]
    fn rescaling_a_regressor_changes_no_t_statistic(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..40)) {
        let t = frame_of(&values);
        let (base, _) = fit_of(&t, "y ~ a + b", None).unwrap();
        let (scaled, _) = fit_of(&t, "y ~ (a * 1e6) + b", None).unwrap();
        for (b, s) in base.statistics.iter().zip(&scaled.statistics) {
            prop_assert!(close(*b, *s, 1e-10, 1e-12), "{} against {}", b, s);
        }
        for (b, s) in base.p_values.iter().zip(&scaled.p_values) {
            prop_assert!(close(*b, *s, 1e-8, 1e-14));
        }
        prop_assert!(close(base.r_squared.unwrap(), scaled.r_squared.unwrap(), 1e-10, 1e-14));
        prop_assert!(close(base.estimates[1] / 1e6, scaled.estimates[1], 1e-10, 1e-20));
    }

    /// Reordering the rows changes no printed value.
    #[test]
    fn reordering_rows_changes_no_printed_value(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..40), seed in any::<u64>()) {
        let t = frame_of(&values);
        let n = values.len();
        let mut order: Vec<usize> = (0..n).collect();
        let mut state = seed | 1;
        for i in (1..n).rev() {
            state ^= state << 13; state ^= state >> 7; state ^= state << 17;
            let j = (state % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        let permuted = t.take_rows(&order);
        let (a, _) = fit_of(&t, "y ~ a + b", None).unwrap();
        let (b, _) = fit_of(&permuted, "y ~ a + b", None).unwrap();
        prop_assert_eq!(zeff_core::display::show(&Value::Fit(std::rc::Rc::new(a))), zeff_core::display::show(&Value::Fit(std::rc::Rc::new(b))));
    }

    /// A duplicate regressor is dropped with a note, whatever the data.
    #[test]
    fn a_duplicate_regressor_is_dropped_with_a_note(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..30)) {
        let t = frame_of(&values);
        let (fit, messages) = fit_of(&t, "y ~ a + b + (a * 2)", None).unwrap();
        prop_assert_eq!(&fit.omitted, &["(a * 2)".to_string()]);
        prop_assert_eq!(&fit.terms, &["(intercept)", "a", "b"]);
        prop_assert!(messages.iter().any(|m| m.contains("`(a * 2)` is collinear with the columns before it and was dropped")), "{:?}", messages);
    }

    /// The design in blocks of any size gives the same bits.
    #[test]
    fn any_block_size_gives_the_same_bits(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..60), block_rows in 1usize..20) {
        let t = frame_of(&values);
        let f = formula("y ~ a + b");
        let frame = Frame::new(t);
        let mut fits = Vec::new();
        for rows in [block_rows, 65_536] {
            let mut output = String::new();
            let mut messages: Vec<Diagnostic> = Vec::new();
            let mut random = Randomness::default();
            let mut ctx = CallContext {
                files: &NoFiles, script: "t.zeff", text: "", columns_used: Vec::new(), frame: None,
                callee_span: Span::new(0, 1), arg_spans: Vec::new(), rest_spans: Vec::new(), definition_spans: Vec::new(),
                output: &mut output, messages: &mut messages, random: &mut random, statement: 1,
            };
            let design = Design::build(&mut ctx, &frame, &f, &[]).unwrap().with_block_rows(rows);
            fits.push(Ols.fit(&mut ctx, &design).unwrap());
        }
        for (a, b) in fits[0].estimates.iter().zip(&fits[1].estimates) {
            prop_assert_eq!(a.to_bits(), b.to_bits());
        }
        for (a, b) in fits[0].residuals.iter().zip(&fits[1].residuals) {
            prop_assert_eq!(a.to_bits(), b.to_bits());
        }
        prop_assert_eq!(fits[0].residual_sum_of_squares.to_bits(), fits[1].residual_sum_of_squares.to_bits());
    }
}
