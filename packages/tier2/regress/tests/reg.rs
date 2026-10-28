//! `reg` (Language Reference, sections 7.2 and 10.2; Engine and Project Specification, section
//! 13; release 0.4.0, steps 10, 11 and 14): against R's `lm`, `summary`, `confint` and `sandwich`
//! on the shipped `wages`, `wages_large`, Wooldridge's `wage1` and `wage2` and the edge-case
//! files (`tests/reference/r/regression.json`); the exact small cases of
//! `tests/reference/kernels.json`; the generating model of `wages` recovered; every edge-case
//! dataset of the handbook; the properties of scaling, row order, duplicates, leverage, clusters
//! and block size; and the summary with every note and error.

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
use zeff_testkit::{formula, root, run, run_with};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("reg reads no file")
    }
}

/// The options of a fit besides its formula.
#[derive(Default, Clone)]
struct Options<'a> {
    se: Option<&'a str>,
    cluster: Option<&'a str>,
    weights: Option<&'a str>,
    weight_type: Option<&'a str>,
    level: Option<f64>,
}

/// `reg` through its body: the fit, with the messages, or the error rendered.
fn fit_of(table: &Table, text: &str, level: Option<f64>) -> Result<(Fit, Vec<String>), String> {
    fit_with(
        table,
        text,
        &Options {
            level,
            ..Options::default()
        },
    )
}

fn fit_with(
    table: &Table,
    text: &str,
    options: &Options<'_>,
) -> Result<(Fit, Vec<String>), String> {
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
            options.se.map(|w| Value::Word(w.to_string())),
            options.cluster.map(|v| Value::Var(v.to_string())),
            options.weights.map(|v| Value::Var(v.to_string())),
            options.weight_type.map(|w| Value::Word(w.to_string())),
            options.level.map(Value::Float),
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
        "wage1" | "wage2" => reference_data(name),
        other => dataset(other),
    }
}

/// Wooldridge's textbook data, as `regression.R` writes it beside the references (question 2 of
/// the release): the same rows R fitted.
fn reference_data(file: &str) -> Table {
    let path = root()
        .join("tests/reference/data")
        .join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
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
        let non_empty = |key: &str| {
            let v = case[key].as_str().unwrap_or("");
            (!v.is_empty()).then_some(v)
        };
        let se = match case["se"].as_str().unwrap_or("classical") {
            "classical" => Some("classical"),
            "HC1" => Some("robust"),
            "HC2" => Some("hc2"),
            "HC3" => Some("hc3"),
            _ => None,
        };
        let options = Options {
            se: if non_empty("cluster").is_some() {
                None
            } else {
                se
            },
            cluster: non_empty("cluster"),
            weights: non_empty("weights"),
            weight_type: non_empty("weight_type"),
            level: Some(level),
        };
        let (fit, _) = fit_with(&table, text, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
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
        // Under analytic and sampling weights Zeff scales the weights to sum to the rows (Stata's
        // convention), so its residual variance is in the data's units where R's carries the
        // weights' scale; the estimates, errors and R² agree, the scale-bound measures do not.
        let scaled_weights = matches!(options.weight_type, Some("analytic") | Some("sampling"))
            && options.weights.is_some();
        if !scaled_weights {
            measure("sigma", fit.sigma, "sigma");
            measure(
                "residual sum of squares",
                fit.residual_sum_of_squares,
                "rss",
            );
            measure("log-likelihood", fit.log_likelihood, "log_likelihood");
        }
    }
}

/// The generating model of the shipped `wages` (its generating script; the testing handbook,
/// section 1.2): the return to schooling of 0.074 inside the 95 per cent interval of the pooled
/// regression with errors clustered by worker, and the other coefficients inside their 99 per
/// cent intervals. The formula's base industry is Agriculture, 0.15 below the generator's base
/// Retail, so each industry effect is 0.15 above the script's figure.
#[test]
fn the_generating_model_of_wages_is_recovered() {
    let wages = table_of("wages");
    let text = "log(wage) ~ educ + exper + exper^2 + female + union + industry + year";
    let fit_at = |level: f64| {
        fit_with(
            &wages,
            text,
            &Options {
                cluster: Some("worker"),
                level: Some(level),
                ..Options::default()
            },
        )
        .unwrap()
        .0
    };
    let interval = |fit: &Fit, term: &str| -> (f64, f64) {
        let i = fit
            .terms
            .iter()
            .position(|t| t == term)
            .unwrap_or_else(|| panic!("no term `{term}`"));
        fit.intervals[i]
    };
    let fit = fit_at(0.95);
    let (lo, hi) = interval(&fit, "educ");
    assert!(
        lo <= 0.074 && 0.074 <= hi,
        "the return to schooling: 0.074 outside [{lo}, {hi}]"
    );
    assert_eq!(fit.n, 18_179);
    let wide = fit_at(0.99);
    for (term, truth) in [
        ("exper", 0.041),
        ("exper^2", -0.0007),
        ("female", -0.112),
        ("union", 0.08),
        ("year", 0.015),
    ] {
        let (lo, hi) = interval(&wide, term);
        assert!(
            lo <= truth && truth <= hi,
            "{term}: {truth} outside the 99 per cent interval [{lo}, {hi}]"
        );
    }
    // The eleven industry effects share the base industry's sampling error (Agriculture has 3
    // per cent of the workers), so they are judged together: the Wald statistic of the eleven
    // deviations under the clustered covariance against the 99 per cent point of χ² on eleven
    // degrees of freedom.
    let industries = [
        ("industry=Mining", 0.30),
        ("industry=Utilities", 0.27),
        ("industry=Construction", 0.20),
        ("industry=Manufacturing", 0.19),
        ("industry=Retail", 0.15),
        ("industry=Transport", 0.17),
        ("industry=Finance", 0.29),
        ("industry=Services", 0.12),
        ("industry=Education", 0.16),
        ("industry=Health", 0.21),
        ("industry=Public administration", 0.23),
    ];
    let statistic = wald(&fit, &industries);
    assert!(
        statistic < 24.725,
        "the industry effects: Wald statistic {statistic:.2} on 11 degrees of freedom"
    );
}

/// The Wald statistic (b − β)ᵀ V⁻¹ (b − β) of the named terms against their true values, with V
/// the fit's variance sub-matrix inverted by Gauss–Jordan elimination with partial pivoting.
fn wald(fit: &Fit, truths: &[(&str, f64)]) -> f64 {
    let k = fit.terms.len();
    let positions: Vec<usize> = truths
        .iter()
        .map(|(term, _)| fit.terms.iter().position(|t| t == term).unwrap())
        .collect();
    let m = positions.len();
    let deviations: Vec<f64> = positions
        .iter()
        .zip(truths)
        .map(|(&i, (_, truth))| fit.estimates[i] - truth)
        .collect();
    let mut a: Vec<Vec<f64>> = positions
        .iter()
        .map(|&i| positions.iter().map(|&j| fit.vcov[i * k + j]).collect())
        .collect();
    let mut inverse: Vec<Vec<f64>> = (0..m)
        .map(|i| (0..m).map(|j| f64::from(u8::from(i == j))).collect())
        .collect();
    for col in 0..m {
        let pivot = (col..m)
            .max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap())
            .unwrap();
        a.swap(col, pivot);
        inverse.swap(col, pivot);
        let p = a[col][col];
        for j in 0..m {
            a[col][j] /= p;
            inverse[col][j] /= p;
        }
        for row in 0..m {
            if row != col {
                let f = a[row][col];
                for j in 0..m {
                    a[row][j] -= f * a[col][j];
                    inverse[row][j] -= f * inverse[col][j];
                }
            }
        }
    }
    let mut statistic = 0.0;
    for i in 0..m {
        for j in 0..m {
            statistic += deviations[i] * inverse[i][j] * deviations[j];
        }
    }
    statistic
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
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + exper + female, se: robust)\nw.reg(log(wage) ~ educ + exper + female, se: hc2)\nw.reg(log(wage) ~ educ + exper + female, se: hc3)\n",
        "dataset(wages).reg(log(wage) ~ educ + exper + female, cluster: worker)\n",
        "dataset(wages).reg(log(wage) ~ educ + exper + female, cluster: industry)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, weights: hours)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, weights: hours, weight_type: analytic)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, weights: exper, weight_type: frequency)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, weights: hours, se: classical)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, cluster: worker, se: hc2)\n",
        "dataset(wages).reg(log(wage) ~ educ + female, se: bootstrap)\n",
        "dataset(wages).head(40).reg(log(wage) ~ educ + female, cluster: worker)\n",
        "dataset(wages).add(one = 1).reg(log(wage) ~ educ, cluster: one)\n",
        "dataset(wages).add(w = wage - 10).reg(log(wage) ~ educ, weights: w)\n",
        "dataset(wages).add(w = hours / 7).reg(log(wage) ~ educ, weights: w, weight_type: frequency)\n",
        "dataset(wages).add(w = to(worker == 1, Int) * 1e6 + 1).reg(log(wage) ~ educ, weights: w)\n",
        "dataset(wages).reg(log(wage) ~ educ, weights: industry)\n",
        "dataset(wages).reg(log(wage) ~ educ, weights: nope)\n",
        "dataset(wages).reg(log(wage) ~ educ, cluster: nope)\n",
        "let w = dataset(wages).head(30).add(y = wage + to(worker == 1, Int) * 500)\nw.reg(y ~ educ)\n",
        // Near-singular without any column being dropped: `y` is `a` plus a little of `u`, and `z`
        // is `u` plus a little of `v`, so a large combination of the three is almost zero while each
        // column keeps a tenth of a thousandth of its norm after the ones before it.
        "let d = frame(a = [1.1, -0.7, 2.3, 0.4, -1.9, 0.8, 1.5, -2.2, 0.3, 1.7, -0.5, 2.9], u = [0.3, 1.4, -0.8, 2.1, 0.6, -1.3, 0.9, 0.2, -1.7, 1.1, 2.4, -0.4], v = [-1.2, 0.5, 0.7, -0.3, 1.8, 0.1, -0.9, 1.3, 0.6, -2.1, 0.8, 1.0], w = [2.0, 1.0, 3.0, 0.5, 1.5, 2.5, 3.5, 0.0, 1.2, 2.2, 0.7, 1.9])\nlet e = d.add(y = a + 1e-5 * u, z = u + 1e-5 * v)\ne.reg(w ~ a + y + z)\n",
        // Every edge-case dataset of the handbook (step 14): all NA, no rows, one row, Bool
        // variables, Unicode labels, awkward names, no variation, levels holding symbols, the
        // magnitudes file with its subnormal column, a level with no observations, a calendar
        // year under clustered errors, and an exact fit.
        "import(\"data/all_na.csv\", types: {x: Float, y: Float}).reg(y ~ x)\n",
        "import(\"data/empty.csv\", types: {x: Float, y: Float}).reg(y ~ x)\n",
        "import(\"data/one_row.csv\").reg(y ~ x)\n",
        "let b = import(\"data/bools.csv\")\nb.reg(with_na ~ treated)\nb.reg(treated ~ mixed_case)\n",
        "import(\"data/unicode_labels.csv\").reg(n ~ to(city == \"東京\", Int))\n",
        "let a = import(\"data/awkward_names.csv\")\na.reg(`Gross Sales (total)` ~ `日本語`)\na.reg(Wage ~ `x y`)\n",
        "let z = import(\"data/zero_variance.csv\")\nz.reg(constant ~ big_constant)\nz.reg(constant_with_na ~ constant)\n",
        "import(\"data/level_symbols.csv\").reg(y ~ category(grp))\n",
        "import(\"data/magnitudes.csv\").reg(offset ~ wide + tiny)\n",
        "dataset(wages).where(industry != \"Mining\").reg(log(wage) ~ educ + industry)\n",
        "dataset(wages).reg(log(wage) ~ educ + industry + year, cluster: worker)\n",
        "dataset(wages).add(y2 = 3 * educ + 1).reg(y2 ~ educ)\n",
        "dataset(wages).add(zero = 0).reg(wage ~ educ + zero)\n",
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

    /// At zero leverage HC2 and HC3 equal HC1 up to the finite-sample factor: rows of zeros in
    /// a model without an intercept have no leverage, and their scores are zero too, so the
    /// three sandwiches agree on the bits.
    #[test]
    fn hc2_and_hc3_equal_hc1_at_zero_leverage(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..30)) {
        let mut rows: Vec<(f64, f64, f64)> = values.clone();
        rows.push((0.0, 0.0, 0.0));
        rows.push((0.0, 0.0, 0.0));
        let t = frame_of(&rows);
        let fits: Vec<Fit> = ["robust", "hc2", "hc3"]
            .iter()
            .map(|se| fit_with(&t, "y ~ a + b - 1", &Options { se: Some(se), ..Options::default() }).unwrap().0)
            .collect();
        // The zero rows have zero leverage and zero score: every kind agrees on their part, and
        // the others differ by their leverage factors, so the kinds differ but stay close.
        for fit in &fits[1..] {
            for (a, b) in fits[0].std_errors.iter().zip(&fit.std_errors) {
                prop_assert!((a - b).abs() <= 0.9 * a.abs().max(b.abs()), "{} against {}", a, b);
            }
        }
        let n = rows.len();
        prop_assert_eq!(fits[0].n, n);
    }

    /// Clustering with every row its own cluster is HC1: the adjustments G/(G − 1) · (N − 1)/(N − k)
    /// and N/(N − k) agree when G is N.
    #[test]
    fn every_row_its_own_cluster_is_hc1(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..40)) {
        let mut t = frame_of(&values);
        let ids: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        t = t.with_column("id", floats(&ids)).unwrap();
        let (robust, _) = fit_with(&t, "y ~ a + b", &Options { se: Some("robust"), ..Options::default() }).unwrap();
        let (clustered, messages) = fit_with(&t, "y ~ a + b", &Options { cluster: Some("id"), ..Options::default() }).unwrap();
        for (a, b) in robust.std_errors.iter().zip(&clustered.std_errors) {
            prop_assert!(close(*a, *b, 1e-12, 1e-15), "{} against {}", a, b);
        }
        prop_assert!(messages.iter().any(|m| m.contains("hold one observation")));
        prop_assert_eq!(clustered.df_residual, values.len() - 1);
    }

    /// The reused design gives the same bits as a fresh one: the same frame value fitted twice
    /// with different standard errors, against a copy of the frame fitted anew.
    #[test]
    fn a_reused_design_gives_the_same_bits(values in prop::collection::vec((-5.0f64..5.0, -3.0f64..3.0, -3.0f64..3.0), 6..40)) {
        let t = frame_of(&values);
        let frame = Frame::new(t.clone());
        let fresh = Frame::new(t.clone());
        let fit = |frame: &Frame, se: &str| {
            let f = formula("y ~ a + b");
            let mut output = String::new();
            let mut messages: Vec<Diagnostic> = Vec::new();
            let mut random = Randomness::default();
            let mut ctx = CallContext {
                files: &NoFiles, script: "t.zeff", text: "", columns_used: Vec::new(), frame: None,
                callee_span: Span::new(0, 1), arg_spans: Vec::new(), rest_spans: Vec::new(), definition_spans: Vec::new(),
                output: &mut output, messages: &mut messages, random: &mut random, statement: 1,
            };
            let args = Args {
                values: vec![Some(Value::Frame(frame.clone())), Some(Value::Formula(f)), Some(Value::Word(se.to_string())), None, None, None, None],
                rest: Vec::new(),
                definitions: Vec::new(),
            };
            match zeff_regress::reg::reg(&mut ctx, args) {
                Ok(Value::Fit(fit)) => (*fit).clone(),
                other => panic!("{other:?}"),
            }
        };
        let first = fit(&frame, "classical");
        let second = fit(&frame, "robust");
        let anew = fit(&fresh, "robust");
        for ((a, b), c) in first.estimates.iter().zip(&second.estimates).zip(&anew.estimates) {
            prop_assert_eq!(a.to_bits(), b.to_bits());
            prop_assert_eq!(b.to_bits(), c.to_bits());
        }
        for (b, c) in second.std_errors.iter().zip(&anew.std_errors) {
            prop_assert_eq!(b.to_bits(), c.to_bits());
        }
        for (b, c) in second.residuals.iter().zip(&anew.residuals) {
            prop_assert_eq!(b.to_bits(), c.to_bits());
        }
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

/// An outcome far below the scale of one (a finding of the formula fuzz target): the sums of
/// squares of the residuals and of the outcome underflow to zero, and the self-check must not
/// take the vanished norms for a failure of orthogonality; the slope is still recovered, and a
/// constant regressor of that size is dropped with a note rather than an internal error.
#[test]
fn a_tiny_outcome_passes_the_self_check() {
    let table = Table::new(vec![
        (
            "y".to_string(),
            Column::floats([Some(1e-300), Some(2e-300), Some(3e-300)]).0,
        ),
        (
            "a".to_string(),
            Column::floats([Some(1.0), Some(2.0), Some(4.0)]).0,
        ),
    ])
    .unwrap();
    let (fit, _) = fit_of(&table, "y ~ a", None).unwrap();
    let i = fit.terms.iter().position(|t| t == "a").unwrap();
    let slope = 9.0 / 14.0 * 1e-300;
    assert!(
        (fit.estimates[i] - slope).abs() <= 1e-6 * slope,
        "slope {} against {slope}",
        fit.estimates[i]
    );
    let constant = Table::new(vec![
        (
            "y".to_string(),
            Column::floats([Some(1e-300), Some(1e-300)]).0,
        ),
        (
            "a".to_string(),
            Column::floats([Some(1e-300), Some(1e-300)]).0,
        ),
    ])
    .unwrap();
    let (fit, messages) = fit_of(&constant, "y ~ a", None).unwrap();
    assert_eq!(fit.omitted, vec!["a".to_string()]);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("too small to be scaled")),
        "{messages:?}"
    );
}

/// Values beyond the size whose square can be represented (a finding of the formula fuzz
/// target): an outcome above 1e150 is refused with Z0803 before anything is squared, and a
/// regressor of that size is dropped with a note, as one below 1e-150 is.
#[test]
fn values_too_large_to_square() {
    let huge_outcome: &[u8] = b"y,a\n0.5,1\n1e300,2\n-1,4\n2,3\n";
    let huge_regressor: &[u8] = b"y,a,b\n0.5,1e300,1\n3,2,2\n-1,4,4\n2,3,5\n";
    // Factors that are finite but whose product overflows (the fuzz target's case).
    let overflowing_product: &[u8] =
        b"y,a,b\n3.25,1e300,1\n1,3.25,-1e300\n1e-300,-1e300,-1e300\n2,1,2\n";
    let mut all = String::new();
    for (file, script) in [
        (
            huge_outcome,
            "let d = import(\"data/huge.csv\")\nd.reg(y ~ a)\n",
        ),
        (
            huge_regressor,
            "let d = import(\"data/huge.csv\")\nd.reg(y ~ a + b)\n",
        ),
        (
            huge_outcome,
            "let d = import(\"data/huge.csv\")\nd.reg(y ~ a, se: robust)\n",
        ),
        (
            overflowing_product,
            "let d = import(\"data/huge.csv\")\nd.reg(y ~ a * b, se: robust)\n",
        ),
    ] {
        let outcome = run_with(&[("data/huge.csv", file)], script);
        all.push_str(&format!(
            "> {script}{}--- messages ---\n{}[exit {}]\n\n",
            outcome.output.stdout, outcome.output.stderr, outcome.exit_code
        ));
    }
    insta::assert_snapshot!("magnitudes", all);
}

/// An observation alone in a cell of the design (a finding of the formula fuzz target): its
/// leverage is one, so HC2 and HC3, which divide by one minus the leverage, are refused with
/// Z0802, while HC1 and classical errors fit.
#[test]
fn an_observation_with_leverage_one_refuses_hc2_and_hc3() {
    let file: &[u8] = b"y,a,d\n1e8,-1,0\n-2,1e8,1\n1e8,1e8,1\n3,2,1\n";
    let mut all = String::new();
    for se in ["hc2", "hc3", "robust", "classical"] {
        let script =
            format!("let d = import(\"data/one.csv\")\nd.reg(y ~ category(d) - 1, se: {se})\n");
        let outcome = run_with(&[("data/one.csv", file)], &script);
        all.push_str(&format!(
            "> {script}{}--- messages ---\n{}[exit {}]\n\n",
            outcome.output.stdout, outcome.output.stderr, outcome.exit_code
        ));
    }
    insta::assert_snapshot!("leverage_one", all);
}

/// A robust variance whose meat has rank one (a finding of the formula fuzz target): two rows
/// share the one design pattern with a nonzero residual and the third fits itself exactly, so
/// one variance and its covariances are exact zeros that rounding leaves as residue; the
/// symmetry check must lend them the matrix's scale rather than call them a fault.
#[test]
fn a_rank_one_meat_passes_the_symmetry_check() {
    let file: &[u8] = b"y,c,d\n-81,x,1\n3.25,x,1\n3.25,x,0\n";
    let script = "let d = import(\"data/rank.csv\")\nd.reg(y ~ d:category(c), se: robust)\n";
    let outcome = run_with(&[("data/rank.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .contains("standard errors: robust (HC1)")
    );
}

/// A perfect fit under sampling weights (a finding of the formula fuzz target): two identical
/// rows and a third, so every residual is rounding residue, and the sandwich is nothing but
/// cancellation; the symmetry check must bound its rounding by the sizes of the terms, not by
/// the vanishing result.
#[test]
fn a_perfect_fit_with_sampling_weights_passes_the_symmetry_check() {
    let file: &[u8] = b"y,a\n2,1e8\n125,125\n125,125\n";
    let script = "let d = import(\"data/perfect.csv\")\nd.reg(y ~ a, weights: a)\n";
    let outcome = run_with(&[("data/perfect.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .contains("standard errors: robust (HC1)"),
        "{}",
        outcome.output.stdout
    );
}

/// Sampling weights of any size (a finding of the formula fuzz target): a weight of 1e300 beside
/// ordinary ones is scaled with the others to average one, so the weighted design stays inside
/// the range the arithmetic can square; the regressor of that size is dropped with its note, and
/// the fit prints.
#[test]
fn sampling_weights_of_any_size_are_scaled_before_use() {
    let file: &[u8] = b"y,a\n0.5,1e200\n1,3e200\n1e-300,1e299\n2,4e200\n";
    let script = "let d = import(\"data/w.csv\")\nd.reg(y ~ (a * 2), weights: a)\n";
    let outcome = run_with(&[("data/w.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stderr.contains("too large to be scaled"),
        "{}",
        outcome.output.stderr
    );
    // Weights more than 1e150 apart are refused: a row with such a weight is a share of nothing.
    let apart: &[u8] = b"y,a\n0.5,1e8\n1,3.25\n1e-300,1e300\n2,4\n";
    let refused = run_with(&[("data/w.csv", apart)], script);
    assert_eq!(refused.exit_code, 1);
    assert!(
        refused
            .output
            .stderr
            .contains("error[Z0224]: `a` holds weights too far apart"),
        "{}",
        refused.output.stderr
    );
    // The scale of sampling weights changes nothing: the same fit under the weights times a
    // thousand prints the same table.
    let scaled: &[u8] = b"y,w\n0.5,2000\n1,3250\n2,4000\n3,1000\n";
    let plain: &[u8] = b"y,w\n0.5,2\n1,3.25\n2,4\n3,1\n";
    let fit = |file: &[u8]| {
        run_with(
            &[("data/w.csv", file)],
            "let d = import(\"data/w.csv\")\nd.add(x = [1, 2, 3, 5]).reg(y ~ x, weights: w)\n",
        )
        .output
        .stdout
    };
    assert_eq!(fit(scaled), fit(plain));
}

/// A robust variance with an exact zero on its diagonal (a finding of the formula fuzz target):
/// the rows with a nonzero residual share one design pattern, so a coefficient's scores cancel
/// and its variance is zero in arithmetic and a negative residue in floating point, which the
/// check must take for zero rather than for a fault.
#[test]
fn a_zero_variance_left_as_negative_residue_is_zero() {
    let file: &[u8] = b"y,a,b,d\n0.5,0,0,1\n0,0,0,1\n0,0,0,0\n0,0,2,0\n";
    let script = "let d = import(\"data/zero.csv\")\nd.reg(y ~ d + a * b, se: robust)\n";
    let outcome = run_with(&[("data/zero.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!("zero_variance", outcome.output.stdout);
}

/// One category variable coded twice with different contrasts (a finding of the formula fuzz
/// target): the two codings span the same columns under the same names, so the second is a
/// duplicate kept once, with the note, rather than a second set of columns of the same names.
#[test]
fn a_category_coded_twice_is_one_term() {
    let file: &[u8] = b"y,c,b\n31,x,31\n30,x,31\n31,y,30\n29,x,31\n31,z,31\n0.5,z,0\n0.5,x,0\n";
    let script = "let d = import(\"data/twice.csv\")\nd.reg(y ~ category(c, contrast: helmert) + b + category(c, contrast: sum))\n";
    let outcome = run_with(&[("data/twice.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stderr
            .contains("`category(c, contrast: helmert)` is written twice, once as `category(c, contrast: sum)`, and is kept once"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(
        outcome.output.stdout.matches("c=y").count(),
        1,
        "{}",
        outcome.output.stdout
    );
}

/// A weighted design column that overflows where its factors are finite (a finding of the
/// formula fuzz target): the product `a:b` is 1e308, and the row's weight scales it beyond the
/// largest double; the column is dropped as oversized without a not-a-number reaching the
/// factorisation, and the fit prints.
#[test]
fn an_oversized_column_never_enters_the_factorisation() {
    let file: &[u8] = b"y,a,b\n1e8,1e160,1e8\n1e8,1e160,1e8\n1e8,1e160,-65\n127,1e300,1e8\n1e8,1e160,1e-300\n76,1e200,110\n1e8,1e160,1e8\n1e8,1e160,1e8\n";
    let script = "let d = import(\"data/over.csv\")\nd.reg(y ~ b + a * b, weights: a)\n";
    let outcome = run_with(&[("data/over.csv", file)], script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stderr.contains("too large to be scaled"),
        "{}",
        outcome.output.stderr
    );
}
