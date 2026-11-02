//! `fe:` in `reg` (Language Reference, sections 6, 7.1, 7.2 and 10.2; Engine and Project
//! Specification, section 8.3, point 10, and section 13; release 0.6.0, step 18): against
//! fixest's `feols` on the shipped `wages` and Wooldridge's `wagepan`
//! (`tests/reference/r/fixed_effects.json`); the edge cases of the checklist as a snapshot; the
//! properties that `fe: g` is `category(g)` written out, that the row order changes nothing,
//! that a regressor constant within the groups is absorbed and named, and that fits sharing
//! their effects and rows reuse the group indexes; and `predict` with the effects' part.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_testkit::{formula, root, run, terms};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("reg reads no file")
    }
}

/// The options of a fit besides its formula.
#[derive(Default, Clone)]
struct Options<'a> {
    fe: Option<&'a str>,
    se: Option<&'a str>,
    cluster: Option<&'a str>,
    weights: Option<&'a str>,
    weight_type: Option<&'a str>,
    level: Option<f64>,
}

/// A call context over a one-line script, for a package body called by hand.
fn with_context<T>(
    script: &str,
    f: impl FnOnce(&mut CallContext<'_>, &Sources) -> Result<T, Fail>,
) -> Result<(T, Vec<String>), String> {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: script,
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(10, 13),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(14, 20))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let result = f(&mut ctx, &sources);
    let rendered: Vec<String> = messages.iter().map(|m| m.render(&sources)).collect();
    match result {
        Ok(value) => Ok((value, rendered)),
        Err(e) => Err(e.render(&sources)),
    }
}

/// `reg` through its body: the fit, with the messages, or the error rendered.
fn fit_with(
    table: &Table,
    text: &str,
    options: &Options<'_>,
) -> Result<(Fit, Vec<String>), String> {
    fit_frame(&Frame::new(table.clone()), text, options)
}

/// The same on a frame, which a script's binding shares between its fits.
fn fit_frame(
    frame: &Frame,
    text: &str,
    options: &Options<'_>,
) -> Result<(Fit, Vec<String>), String> {
    let script = format!("let m = d.reg({text})\n");
    with_context(&script, |ctx, _| {
        let args = Args {
            values: vec![
                Some(Value::Frame(frame.clone())),
                Some(Value::Formula(formula(text))),
                options.fe.map(|t| Value::Terms(terms(t))),
                options.se.map(|w| Value::Word(w.to_string())),
                options.cluster.map(|v| Value::Var(v.to_string())),
                options.weights.map(|v| Value::Var(v.to_string())),
                options.weight_type.map(|w| Value::Word(w.to_string())),
                options.level.map(Value::Float),
            ],
            rest: Vec::new(),
            definitions: Vec::new(),
        };
        match zeff_regress::reg::reg(ctx, args)? {
            Value::Fit(fit) => Ok((*fit).clone()),
            other => panic!("not a fit: {other:?}"),
        }
    })
}

/// `fit.predict(newdata:)` through its body.
fn predict_with(fit: &Fit, newdata: &Table) -> Result<(Vec<Option<f64>>, Vec<String>), String> {
    with_context("let p = m.predict(newdata: n)\n", |ctx, _| {
        let args = Args {
            values: vec![
                Some(Value::Fit(std::rc::Rc::new(fit.clone()))),
                Some(Value::Frame(Frame::new(newdata.clone()))),
            ],
            rest: Vec::new(),
            definitions: Vec::new(),
        };
        match zeff_results::interface::predict(ctx, args)? {
            Value::Column(column) => Ok(column.as_floats().expect("floats").collect()),
            other => panic!("not a column: {other:?}"),
        }
    })
}

fn json(file: &str) -> Json {
    let path = root().join("tests/reference").join(file);
    serde_json::from_slice(
        &std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap()
}

fn number(value: &Json) -> f64 {
    match value {
        Json::String(s) => match s.as_str() {
            "NA" => f64::NAN,
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            other => other.parse().unwrap(),
        },
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other}"),
    }
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a.is_nan() && b.is_nan())
        || (a - b).abs() <= relative * a.abs().max(b.abs())
        || (a - b).abs() <= absolute
}

/// Wooldridge's panel as `fixed_effects.R` writes it beside the references: the same rows fixest
/// fitted.
fn reference_data(file: &str) -> Table {
    let path = root()
        .join("tests/reference/data")
        .join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        other => reference_data(other),
    }
}

/// The tolerance of the comparison: the class is closed form (1e-6 relative); the projections
/// stop at 1e-8 here and 1e-11 in fixest, and the slowest case (worker and year effects over
/// 3,878 workers) agrees to 1e-8 on its estimates and errors, so the comparison holds every
/// case to 1e-7, ten times tighter than the class.
const RELATIVE: f64 = 1e-7;

#[test]
fn fits_agree_with_fixest() {
    let reference = json("r/fixed_effects.json");
    for (name, case) in reference["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
        let non_empty = |key: &str| {
            let v = case[key].as_str().unwrap_or("");
            (!v.is_empty()).then_some(v)
        };
        let se = match case["se"].as_str().unwrap() {
            "HC1" => Some("robust"),
            _ => None,
        };
        let options = Options {
            fe: Some(case["fe"].as_str().unwrap()),
            se,
            cluster: non_empty("cluster"),
            weights: non_empty("weights"),
            weight_type: non_empty("weight_type"),
            level: Some(number(&case["level"])),
        };
        let (fit, _) = fit_with(&table, text, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
        let strings = |key: &str| -> Vec<String> {
            let mut v: Vec<String> = case[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t.as_str().unwrap().to_string())
                .collect();
            v.sort();
            v
        };
        let sorted = |mut v: Vec<String>| {
            v.sort();
            v
        };
        assert_eq!(
            sorted(fit.terms.clone()),
            strings("terms"),
            "{name}: the terms"
        );
        assert_eq!(
            sorted(fit.omitted.clone()),
            strings("omitted"),
            "{name}: the omitted terms"
        );
        assert_eq!(fit.n as u64, case["nobs"].as_u64().unwrap(), "{name}: N");
        assert_eq!(
            fit.singletons as u64,
            case["singletons"].as_u64().unwrap(),
            "{name}: singletons"
        );
        let sizes: Vec<u64> = fit.effects.iter().map(|e| e.keys.len() as u64).collect();
        let expected_sizes: Vec<u64> = case["fixef_sizes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(sizes, expected_sizes, "{name}: the levels of each effect");
        assert_eq!(
            fit.df_residual as u64,
            case["df_t"].as_u64().unwrap(),
            "{name}: the t table's df"
        );
        // The residual degrees of freedom of the fit measures count every effect; under
        // `cluster:` fixest's residual df leaves the nested effects out, which the standard
        // errors test below through the small-sample adjustment.
        if options.cluster.is_none() {
            let df_residual = fit.n - fit.terms.len() - fit.absorbed_parameters;
            assert_eq!(
                df_residual as u64,
                case["df_residual"].as_u64().unwrap(),
                "{name}: residual df"
            );
        }
        let columns = case["columns"].as_object().unwrap();
        for (i, term) in fit.terms.iter().enumerate() {
            let r = &columns[term];
            let check = |what: &str, zeff: f64, key: &str, relative: f64, absolute: f64| {
                let r_value = number(&r[key]);
                assert!(
                    close(zeff, r_value, relative, absolute),
                    "{name}: {term} {what}: Zeff {zeff:e}, fixest {r_value:e}"
                );
            };
            check("estimate", fit.estimates[i], "estimate", RELATIVE, 1e-12);
            check("std.error", fit.std_errors[i], "std_error", RELATIVE, 1e-12);
            check("t value", fit.statistics[i], "t_value", RELATIVE, 1e-10);
            check("p value", fit.p_values[i], "p_value", 1e-6, 1e-12);
            check("ci lower", fit.intervals[i].0, "ci_lower", RELATIVE, 1e-12);
            check("ci upper", fit.intervals[i].1, "ci_upper", RELATIVE, 1e-12);
        }
        let measure = |what: &str, zeff: Option<f64>, key: &str| {
            let r_value = number(&case[key]);
            let zeff = zeff.unwrap_or(f64::NAN);
            assert!(
                close(zeff, r_value, RELATIVE, 1e-12),
                "{name}: {what}: Zeff {zeff:e}, fixest {r_value:e}"
            );
        };
        measure("R² overall", fit.r_squared, "r_squared");
        measure(
            "adjusted R² overall",
            fit.adjusted_r_squared,
            "adj_r_squared",
        );
        measure("R² within", fit.r_squared_within, "within_r_squared");
        measure(
            "adjusted R² within",
            fit.adjusted_r_squared_within,
            "adj_within_r_squared",
        );
        // The weighted residual sum of squares is in the data's units in fixest and in the
        // scaled weights' here; without weights they agree.
        if options.weights.is_none() {
            measure(
                "residual sum of squares",
                Some(fit.residual_sum_of_squares),
                "rss",
            );
        }
        // The fitted values (the effects' part included) and residuals of the first twenty rows
        // fixest used, matched by frame row since the fit holds its rows in the kernel's order;
        // a residual near zero is judged at the outcome's scale (an absolute 1e-7).
        let rows: Vec<usize> = case["rows_20"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize - 1)
            .collect();
        for (j, &row) in rows.iter().enumerate() {
            let i = fit
                .rows
                .iter()
                .position(|&r| r == row)
                .unwrap_or_else(|| panic!("{name}: row {row} not used"));
            let (fitted, residual) = (
                number(&case["fitted_20"][j]),
                number(&case["residuals_20"][j]),
            );
            assert!(
                close(fit.fitted[i], fitted, RELATIVE, 1e-7),
                "{name}: fitted of row {row}: Zeff {} fixest {fitted}",
                fit.fitted[i]
            );
            assert!(
                close(fit.residuals[i], residual, RELATIVE, 1e-7),
                "{name}: residual of row {row}: Zeff {} fixest {residual}",
                fit.residuals[i]
            );
        }
        // `predict` on the first ten rows of the data, the effects' part included, `NA` where a
        // row has `NA` in a model variable.
        let head = head_of(&table, 10);
        let (predicted, _) = predict_with(&fit, &head).unwrap_or_else(|e| panic!("{name}: {e}"));
        for (j, value) in predicted.iter().enumerate() {
            let expected = number(&case["predict_10"][j]);
            match value {
                Some(v) => assert!(
                    close(*v, expected, RELATIVE, 1e-7),
                    "{name}: prediction of row {}: Zeff {v} fixest {expected}",
                    j + 1
                ),
                None => assert!(
                    expected.is_nan(),
                    "{name}: prediction of row {} is NA, fixest {expected}",
                    j + 1
                ),
            }
        }
    }
}

/// The first `n` rows of a table.
fn head_of(table: &Table, n: usize) -> Table {
    table.take_rows(&(0..n.min(table.rows())).collect::<Vec<_>>())
}

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

/// The edge cases of the checklist, through the engine: every row a singleton; one group; an
/// effect collinear with a regressor, absorbed; an effect that is a model variable too; `- 1`
/// with `fe:`; a Date effect; zero weights with `fe:`; `cluster:` nested and not nested in an
/// effect; the refusals of `*`, a range, an expression and the intercept in `fe:`; `hc2` and
/// `influence` with `fe:`; and the printed fit of section 10.2 with its `report` table.
#[test]
fn the_edge_cases() {
    let mut all = String::new();
    let scripts = [
        // Every row a singleton: the unit effect of a cross-section.
        "let d = frame(id = [1, 2, 3, 4], y = [1.0, 2.0, 3.0, 5.0], x = [1.0, 3.0, 2.0, 4.0])\nd.reg(y ~ x, fe: id)\n",
        // One group: the effect is the intercept.
        "let d = frame(g = [1, 1, 1, 1, 1], y = [1.0, 2.0, 3.0, 5.0, 4.0], x = [1.0, 3.0, 2.0, 4.0, 5.0])\nd.reg(y ~ x, fe: g)\nd.reg(y ~ x)\n",
        // A regressor constant within the groups is absorbed and named; `- 1` has no effect.
        "let d = frame(g = [1, 1, 2, 2, 3, 3], z = [10, 10, 20, 20, 30, 30], y = [1.0, 2.0, 3.0, 5.0, 4.0, 7.0], x = [1.0, 3.0, 2.0, 4.0, 5.0, 8.0])\nlet m = d.reg(y ~ x + z, fe: g)\nm\nd.reg(y ~ x - 1, fe: g)\n",
        // A Date effect, and an effect of text.
        "let d = frame(text = [\"2024-01-01\", \"2024-01-01\", \"2024-01-02\", \"2024-01-02\", \"2024-01-03\", \"2024-01-03\"], shop = [\"a\", \"b\", \"a\", \"b\", \"a\", \"b\"], y = [1.0, 2.5, 3.0, 5.0, 4.0, 7.5], x = [1.0, 3.0, 2.0, 4.0, 5.0, 8.0]).add(day = to(text, Date))\nd.reg(y ~ x, fe: day)\nd.reg(y ~ x, fe: shop + day)\n",
        // Zero weights leave their rows out before the groups are made; NA in an effect drops the row.
        "let d = frame(g = [1, 1, 2, 2, 3, 3, 3], w = [1, 0, 1, 2, 1, 1, 1], y = [1.0, 2.5, 3.0, 5.0, 4.0, 7.5, 6.0], x = [1.0, 3.0, 2.0, 4.0, 5.0, 8.0, 7.0])\nd.reg(y ~ x, fe: g, weights: w, weight_type: analytic)\nlet e = frame(g = [1, 1, 2, 2, 3, 3, NA], y = [1.0, 2.5, 3.0, 5.0, 4.0, 7.5, 6.0], x = [1.0, 3.0, 2.0, 4.0, 5.0, 8.0, 7.0])\ne.reg(y ~ x, fe: g)\n",
        // The cluster variable nested in the effect, and the effect nested in the cluster.
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + female, fe: firm, cluster: worker)\nw.reg(log(wage) ~ educ + female, fe: firm, cluster: industry)\n",
        // The refusals of the checker and of the fit.
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: firm * year)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: educ..exper)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: log(firm))\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: firm - 1)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: 1)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + firm, fe: firm)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: firm:year, se: hc3)\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: firm).influence()\n",
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, fe: nowhere)\n",
        // The printed fit of section 10.2, `r_squared`, `report` with and without the within row,
        // and `predict` with the effects' part.
        "let w = dataset(wages)\nlet m1 = w.reg(log(wage) ~ educ + exper + exper^2 + female, se: robust)\nlet m2 = w.reg(log(wage) ~ educ + exper + exper^2 + female, fe: industry + year, cluster: worker)\nm2\nm2.r_squared()\nreport([m1, m2])\nreport([m1, m2], stats: [n_obs, r_squared, r_squared_within])\nm2.predict(newdata: w.head(3))\nm2.fitted()\nm2.residuals()\n",
        // W1004: a regressor that barely varies within the effects' groups.
        "let w = dataset(wages)\nw.reg(log(wage) ~ exper + exper^2, fe: worker + year, cluster: worker)\n",
        // A loop variable as an effect, and an effect given in a bound formula's frame.
        "let w = dataset(wages)\nlet fits = [for g in vars(firm, industry) { w.reg(log(wage) ~ educ, fe: g) }]\nreport(fits)\n",
    ];
    for script in scripts {
        all.push_str(&printed(script));
        all.push_str("----\n");
    }
    insta::assert_snapshot!("edge_cases", all);
}

/// The frame of the property tests: `n` rows in `groups` groups with a group effect, a regressor
/// and a second regressor constant within the groups.
fn panel(n: usize, groups: usize, seed: u64) -> Table {
    let mut stream = zeff_numeric::random::Stream::new(seed, 1, 0);
    let effect: Vec<f64> = (0..groups).map(|_| 4.0 * stream.uniform() - 2.0).collect();
    let mut g = Vec::new();
    let mut x = Vec::new();
    let mut z = Vec::new();
    let mut y = Vec::new();
    for i in 0..n {
        let group = i % groups;
        let xi = 2.0 * stream.uniform() - 1.0;
        g.push(Some((group + 1) as i64));
        x.push(Some(xi));
        z.push(Some(effect[group] * 10.0));
        y.push(Some(
            1.5 * xi + effect[group] + 0.1 * (stream.uniform() - 0.5),
        ));
    }
    Table::new(vec![
        ("g".to_string(), Column::ints(g)),
        ("x".to_string(), Column::floats(x).0),
        ("z".to_string(), Column::floats(z).0),
        ("y".to_string(), Column::floats(y).0),
    ])
    .unwrap()
}

fn property_cases(usual: u32) -> u32 {
    if std::env::var_os("CI").is_some() {
        usual
    } else {
        usual / 4
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// A fit with `fe: g` equals the fit with `category(g)` written out, within 1e-10 on the
    /// shared coefficients and their classical errors: one effect is centring by group, exact in
    /// one pass, and the degrees of freedom are the same.
    #[test]
    fn one_effect_is_the_category_written_out(n in 12usize..60, groups in 2usize..5, seed in 1u64..1000) {
        let table = panel(n, groups, seed);
        let (with_fe, _) = fit_with(&table, "y ~ x", &Options { fe: Some("g"), ..Options::default() }).unwrap();
        let (written, _) = fit_with(&table, "y ~ x + category(g)", &Options::default()).unwrap();
        let i = written.terms.iter().position(|t| t == "x").unwrap();
        let j = with_fe.terms.iter().position(|t| t == "x").unwrap();
        prop_assert!(close(with_fe.estimates[j], written.estimates[i], 1e-10, 1e-12), "{} vs {}", with_fe.estimates[j], written.estimates[i]);
        prop_assert!(close(with_fe.std_errors[j], written.std_errors[i], 1e-10, 1e-12), "{} vs {}", with_fe.std_errors[j], written.std_errors[i]);
        prop_assert_eq!(with_fe.df_residual, written.df_residual);
        prop_assert!(close(with_fe.r_squared.unwrap(), written.r_squared.unwrap(), 1e-10, 1e-12));
        prop_assert!(close(with_fe.residual_sum_of_squares, written.residual_sum_of_squares, 1e-10, 1e-12));
    }

    /// Reordering the rows changes nothing: the rows are ordered once by the effect, so a
    /// rotation of the frame gives the same estimates, errors and measures.
    #[test]
    fn the_row_order_changes_nothing(n in 12usize..60, groups in 2usize..5, seed in 1u64..1000, shift in 1usize..11) {
        let table = panel(n, groups, seed);
        let rotated: Vec<usize> = (0..n).map(|i| (i + shift) % n).collect();
        let turned = table.take_rows(&rotated);
        let options = Options { fe: Some("g"), ..Options::default() };
        let (a, _) = fit_with(&table, "y ~ x", &options).unwrap();
        let (b, _) = fit_with(&turned, "y ~ x", &options).unwrap();
        prop_assert!(close(a.estimates[0], b.estimates[0], 1e-12, 1e-14));
        prop_assert!(close(a.std_errors[0], b.std_errors[0], 1e-12, 1e-14));
        prop_assert!(close(a.r_squared_within.unwrap(), b.r_squared_within.unwrap(), 1e-12, 1e-14));
        prop_assert_eq!(a.n, b.n);
    }

    /// A regressor constant within the groups is absorbed by the effect, dropped with the note
    /// that says so, and shown as omitted.
    #[test]
    fn a_regressor_constant_within_groups_is_absorbed(n in 12usize..60, groups in 2usize..5, seed in 1u64..1000) {
        let table = panel(n, groups, seed);
        let (fit, messages) = fit_with(&table, "y ~ x + z", &Options { fe: Some("g"), ..Options::default() }).unwrap();
        prop_assert_eq!(fit.omitted.clone(), vec!["z".to_string()]);
        prop_assert!(messages.iter().any(|m| m.contains("`z` is absorbed by fe: g and was dropped")), "{messages:?}");
    }
}

/// Two fits sharing their effects and rows reuse the group indexes (section 8.3, point 10): the
/// counts of built and reused sets, which `--verbose` prints, move by one each.
#[test]
fn fits_sharing_effects_and_rows_reuse_the_group_indexes() {
    let table = panel(40, 4, 7);
    let frame = Frame::new(table.clone());
    zeff_regress::reg::reset_fixed_effect_work();
    let options = Options {
        fe: Some("g"),
        ..Options::default()
    };
    fit_frame(&frame, "y ~ x", &options).unwrap();
    assert_eq!(zeff_regress::reg::fixed_effect_work(), (1, 0));
    // Another formula over the same frame, effects and rows: the indexes and `x` reused.
    let (second, _) = fit_frame(&frame, "y ~ x + z", &options).unwrap();
    assert_eq!(zeff_regress::reg::fixed_effect_work(), (1, 1));
    assert_eq!(second.omitted, vec!["z".to_string()]);
    // The same again with robust errors: the whole fit is reused, nothing rebuilt.
    fit_frame(
        &frame,
        "y ~ x + z",
        &Options {
            se: Some("robust"),
            ..options.clone()
        },
    )
    .unwrap();
    assert_eq!(zeff_regress::reg::fixed_effect_work(), (1, 1));
    // Other rows (a row with NA) build new indexes.
    let mut values: Vec<Option<f64>> = table.column("x").unwrap().as_floats().unwrap().collect();
    values[3] = None;
    let other = Frame::new(table.with_column("x", Column::floats(values).0).unwrap());
    fit_frame(&other, "y ~ x", &options).unwrap();
    assert_eq!(zeff_regress::reg::fixed_effect_work(), (2, 1));
}

/// `predict` adds each row's effects to the regressors' part: on the rows the fit used it gives
/// the fitted values, and a level the fit never saw predicts `NA` with a warning.
#[test]
fn predict_includes_the_effects() {
    let table = panel(30, 3, 11);
    let (fit, _) = fit_with(
        &table,
        "y ~ x",
        &Options {
            fe: Some("g"),
            ..Options::default()
        },
    )
    .unwrap();
    let (predicted, messages) = predict_with(&fit, &table).unwrap();
    assert!(messages.is_empty(), "{messages:?}");
    for (i, &row) in fit.rows.iter().enumerate() {
        let p = predicted[row].unwrap();
        assert!(
            close(p, fit.fitted[i], 1e-12, 1e-14),
            "row {row}: {p} vs {}",
            fit.fitted[i]
        );
    }
    let mut g: Vec<Option<i64>> = table.column("g").unwrap().as_ints().unwrap().collect();
    g[0] = Some(99);
    let unseen = table.with_column("g", Column::ints(g)).unwrap();
    let (predicted, messages) = predict_with(&fit, &unseen).unwrap();
    assert!(predicted[0].is_none());
    assert!(predicted[1].is_some());
    assert!(
        messages
            .iter()
            .any(|m| m.contains("W0207") && m.contains("`g`")),
        "{messages:?}"
    );
}
