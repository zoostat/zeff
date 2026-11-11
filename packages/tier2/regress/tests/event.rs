//! `event` (catalogue row 252; question 2 of release 0.6.0; Language Reference, sections 7.1
//! and 7.2; release 0.6.0, step 19): against fixest's `i(rel, ref = c(-1, -1000))` on a
//! generated staggered panel with never-treated units (`tests/reference/r/event.json`), under
//! two-way fixed effects with clustered errors, pooled, with another reference period and with a
//! Date time; the edge cases of the checklist as a snapshot; and the properties that the
//! indicator columns sum to the treated-rows indicator less the reference period, that
//! relabelling the units changes nothing, and that a shift of every `treat` and `time` by one
//! constant changes nothing.

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

/// `reg` through its body with `fe:` and `cluster:`: the fit, or the error rendered.
fn fit_with(
    table: &Table,
    text: &str,
    fe: Option<&str>,
    cluster: Option<&str>,
) -> Result<Fit, String> {
    let script = format!("let m = d.reg({text})\n");
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
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
            Some(Value::Formula(formula(text))),
            fe.map(|t| Value::Terms(terms(t))),
            None,
            cluster.map(|v| Value::Var(v.to_string())),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => Ok((*fit).clone()),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => Err(e.render(&sources)),
    }
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
            other => other.parse().unwrap(),
        },
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other}"),
    }
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) || (a - b).abs() <= absolute
}

/// The generated staggered panel as `event.R` writes it beside the references.
fn panel() -> Table {
    let path = root().join("tests/reference/data/event_panel.csv");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("event_panel: {e:?}"))
        .table
}

/// The comparison's tolerance: the class is closed form (1e-6 relative); the projections stop
/// at 1e-8 here and 1e-11 in fixest, and every case holds to 1e-7.
const RELATIVE: f64 = 1e-7;

#[test]
fn fits_agree_with_fixest() {
    let reference = json("r/event.json");
    let table = panel();
    for (name, case) in reference["cases"].as_object().unwrap() {
        let text = case["formula"].as_str().unwrap();
        let fe = case["fe"].as_str().filter(|f| !f.is_empty());
        let cluster = case["cluster"].as_str().filter(|c| !c.is_empty());
        let fit = fit_with(&table, text, fe, cluster).unwrap_or_else(|e| panic!("{name}: {e}"));
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
            fit.df_residual as u64,
            case["df_t"].as_u64().unwrap(),
            "{name}: df"
        );
        let columns = case["columns"].as_object().unwrap();
        for (i, term) in fit.terms.iter().enumerate() {
            let r = &columns[term];
            let check = |what: &str, zeff: f64, key: &str| {
                let r_value = number(&r[key]);
                assert!(
                    close(zeff, r_value, RELATIVE, 1e-12),
                    "{name}: {term} {what}: Zeff {zeff:e}, fixest {r_value:e}"
                );
            };
            check("estimate", fit.estimates[i], "estimate");
            check("std.error", fit.std_errors[i], "std_error");
            check("t value", fit.statistics[i], "t_value");
            check("ci lower", fit.intervals[i].0, "ci_lower");
            check("ci upper", fit.intervals[i].1, "ci_upper");
            assert!(
                close(fit.p_values[i], number(&r["p_value"]), 1e-6, 1e-12),
                "{name}: {term} p value"
            );
        }
        let r2 = number(&case["r_squared"]);
        assert!(
            close(fit.r_squared.unwrap(), r2, RELATIVE, 1e-12),
            "{name}: R² {} vs {r2}",
            fit.r_squared.unwrap()
        );
        if fe.is_some() {
            let wr2 = number(&case["within_r_squared"]);
            assert!(
                close(fit.r_squared_within.unwrap(), wr2, RELATIVE, 1e-12),
                "{name}: R² within"
            );
        }
    }
}

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

/// A small panel as a script writes it: four units over 2010 to 2013, cohorts `treat` (one
/// value per unit, `NA` never treated).
fn script_panel(treat: &str) -> String {
    format!(
        "let d = frame(unit = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4], year = [2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013], treat = [{treat}], y = [1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4])\n"
    )
}

/// The edge cases of the checklist, through the engine: every unit treated at once; no
/// never-treated unit, with the collinearity with the time effects named; `ref:` outside the
/// data; a Date time with a Date `treat`; `treat` all `NA`; the types apart; `event` outside a
/// formula; the printed fit of the page with `report` and `coef`; `event` written twice.
#[test]
fn the_edge_cases() {
    let staggered =
        "2011, 2011, 2011, 2011, 2012, 2012, 2012, 2012, 2013, 2013, 2013, 2013, NA, NA, NA, NA";
    let scripts = [
        script_panel(staggered) + "let m = d.reg(y ~ event(year, treat), fe: unit + year)\nm\nm.coef(\"event=0\")\nreport([m])\nd.reg(y ~ event(year, treat), fe: unit + year, cluster: unit)\n",
        // Every unit treated at once: with time effects the indicators are collinear.
        script_panel("2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012, 2012") + "d.reg(y ~ event(year, treat), fe: unit + year)\nd.reg(y ~ event(year, treat))\n",
        // No never-treated unit, staggered: the periods present at every cohort identify.
        script_panel("2011, 2011, 2011, 2011, 2012, 2012, 2012, 2012, 2013, 2013, 2013, 2013, 2011, 2011, 2011, 2011") + "d.reg(y ~ event(year, treat), fe: unit + year)\n",
        script_panel(staggered) + "d.reg(y ~ event(year, treat, ref: 5), fe: unit + year)\n",
        script_panel(staggered) + "d.reg(y ~ event(year, treat, ref: 0), fe: unit + year)\n",
        // A Date time with a Date `treat`: the relative periods in days.
        "let d = frame(unit = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4], year = [2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013], ds = [\"2020-01-01\", \"2020-01-08\", \"2020-01-15\", \"2020-01-22\", \"2020-01-01\", \"2020-01-08\", \"2020-01-15\", \"2020-01-22\", \"2020-01-01\", \"2020-01-08\", \"2020-01-15\", \"2020-01-22\", \"2020-01-01\", \"2020-01-08\", \"2020-01-15\", \"2020-01-22\"], ts = [\"2020-01-08\", \"2020-01-08\", \"2020-01-08\", \"2020-01-08\", \"2020-01-15\", \"2020-01-15\", \"2020-01-15\", \"2020-01-15\", \"2020-01-22\", \"2020-01-22\", \"2020-01-22\", \"2020-01-22\", NA, NA, NA, NA], y = [1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4]).add(day = to(ds, Date), treat_day = to(ts, Date))\nd.reg(y ~ event(day, treat_day, ref: -7), fe: unit + year)\n".to_string(),
        script_panel(staggered) + "let e = d.add(never = treat * NA)\ne.reg(y ~ event(year, never))\n",
        script_panel(staggered) + "let e = d.add(t = to(treat, Str))\ne.reg(y ~ event(year, t))\n",
        script_panel(staggered) + "event(year, treat)\n",
        script_panel(staggered) + "d.reg(y ~ event(year, treat) + event(year, treat), fe: unit + year)\n",
        script_panel(staggered) + "d.reg(y ~ event(year, treat, ref: \"before\"))\n",
    ];
    let mut all = String::new();
    for script in &scripts {
        all.push_str(&printed(script));
        all.push_str("----\n");
    }
    insta::assert_snapshot!("edge_cases", all);
}

/// A generated staggered panel: `units` units over `periods` periods, a share never treated, the
/// rest first treated at one of three cohorts; `y` the sum of unit and period effects, a treated
/// indicator and noise. The first unit is never treated and the second is treated from period
/// 2, whatever the draws: a panel whose units are all treated at once has its leads and lags
/// absorbed by the time effects and no fit to compare (Z0802, an edge case of its own), and a
/// draw of six units gave one now and then.
fn generated(units: usize, periods: usize, seed: u64) -> Table {
    let mut stream = zeff_numeric::random::Stream::new(seed, 2, 0);
    let (mut unit, mut time, mut treat, mut y) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..units {
        let drawn = match (stream.uniform() * 4.0) as usize {
            0 => None,
            k => Some(1 + k as i64),
        };
        let cohort = match i {
            0 => None,
            1 => Some(2),
            _ => drawn,
        };
        let effect = stream.uniform();
        for t in 0..periods {
            let treated = cohort.is_some_and(|c| t as i64 >= c);
            unit.push(Some(i as i64 + 1));
            time.push(Some(t as i64));
            treat.push(cohort);
            y.push(Some(
                effect
                    + 0.1 * t as f64
                    + if treated { 0.5 } else { 0.0 }
                    + 0.2 * (stream.uniform() - 0.5),
            ));
        }
    }
    Table::new(vec![
        ("unit".to_string(), Column::ints(unit)),
        ("time".to_string(), Column::ints(time)),
        ("treat".to_string(), Column::ints(treat)),
        ("y".to_string(), Column::floats(y).0),
    ])
    .unwrap()
}

/// Every panel the properties below can draw has a control and a treated unit, over all the
/// sizes and all the seeds, and at the smallest number of units, where two draws of the 27,944
/// once had every unit treated at the same period, each has a fit under unit and time effects:
/// no draw fails by chance.
#[test]
fn every_generated_panel_has_a_control_and_a_fit() {
    for units in 6usize..20 {
        for periods in 4usize..8 {
            for seed in 1u64..500 {
                let table = generated(units, periods, seed);
                let treat: Vec<Option<i64>> =
                    table.column("treat").unwrap().as_ints().unwrap().collect();
                assert!(
                    treat.contains(&None) && treat.contains(&Some(2)),
                    "units = {units}, periods = {periods}, seed = {seed}"
                );
                if units > 6 {
                    continue;
                }
                if let Err(message) =
                    fit_with(&table, "y ~ event(time, treat)", Some("unit + time"), None)
                {
                    panic!("units = {units}, periods = {periods}, seed = {seed}: {message}");
                }
            }
        }
    }
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

    /// The indicator columns sum, row by row, to the treated-rows indicator less the rows at the
    /// reference period: the design rebuilt by `predict` with unit estimates gives the sum.
    #[test]
    fn the_indicators_sum_to_the_treated_rows_less_the_reference(units in 6usize..20, periods in 4usize..8, seed in 1u64..500) {
        let table = generated(units, periods, seed);
        let fit = fit_with(&table, "y ~ event(time, treat)", None, None).unwrap();
        let time: Vec<Option<i64>> = table.column("time").unwrap().as_ints().unwrap().collect();
        let treat: Vec<Option<i64>> = table.column("treat").unwrap().as_ints().unwrap().collect();
        // Every relative period of a treated row but −1 has a column; so a treated row at any
        // other period has exactly one column at 1, and every other row has none.
        let event_terms: Vec<&String> = fit.terms.iter().filter(|t| t.starts_with("event=")).collect();
        let periods: Vec<i64> = event_terms.iter().map(|t| t["event=".len()..].parse().unwrap()).collect();
        let mut present: Vec<i64> = treat.iter().zip(&time).filter_map(|(c, t)| Some((*t)? - (*c)?)).collect();
        present.sort_unstable();
        present.dedup();
        present.retain(|p| *p != -1);
        prop_assert_eq!(periods, present);
    }

    /// Relabelling the units (their numbers permuted) changes no estimate and no error.
    #[test]
    fn relabelling_the_units_changes_nothing(units in 6usize..20, periods in 4usize..8, seed in 1u64..500) {
        let table = generated(units, periods, seed);
        let fit = fit_with(&table, "y ~ event(time, treat)", Some("unit + time"), Some("unit")).unwrap();
        let labels: Vec<Option<i64>> = table.column("unit").unwrap().as_ints().unwrap().map(|u| u.map(|u| 1000 - u)).collect();
        let relabelled = table.with_column("unit", Column::ints(labels)).unwrap();
        let other = fit_with(&relabelled, "y ~ event(time, treat)", Some("unit + time"), Some("unit")).unwrap();
        prop_assert_eq!(&fit.terms, &other.terms);
        for i in 0..fit.terms.len() {
            prop_assert!(close(fit.estimates[i], other.estimates[i], 1e-10, 1e-12));
            prop_assert!(close(fit.std_errors[i], other.std_errors[i], 1e-10, 1e-12));
        }
    }

    /// A shift of every `treat` and `time` by one constant changes nothing: the relative periods
    /// and the columns are the same.
    #[test]
    fn a_common_shift_of_treat_and_time_changes_nothing(units in 6usize..20, periods in 4usize..8, seed in 1u64..500, shift in -50i64..50) {
        let table = generated(units, periods, seed);
        let fit = fit_with(&table, "y ~ event(time, treat)", Some("unit + time"), None).unwrap();
        let shifted_time: Vec<Option<i64>> = table.column("time").unwrap().as_ints().unwrap().map(|t| t.map(|t| t + shift)).collect();
        let shifted_treat: Vec<Option<i64>> = table.column("treat").unwrap().as_ints().unwrap().map(|t| t.map(|t| t + shift)).collect();
        let shifted = table.with_column("time", Column::ints(shifted_time)).unwrap().with_column("treat", Column::ints(shifted_treat)).unwrap();
        let other = fit_with(&shifted, "y ~ event(time, treat)", Some("unit + time"), None).unwrap();
        prop_assert_eq!(&fit.terms, &other.terms);
        for i in 0..fit.terms.len() {
            prop_assert!(close(fit.estimates[i], other.estimates[i], 1e-10, 1e-12));
        }
    }
}
