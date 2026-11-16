//! Two-stage least squares (Language Reference, sections 7.1 and 7.2; Engine and Project
//! Specification, section 13; release 0.8.0, step 10): against the estimates and standard
//! errors of `tests/reference/r/iv.json`, written out in base R and held there to fixest, on
//! Card's schooling data and Wooldridge's `wage2`, with weights of the three kinds, clusters
//! and inputs of very different sizes; the edge cases of the handbook; the properties of
//! scaling, row order and block size; and every note and error.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, Term};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_numeric::order::WeightKind;
use zeff_regress::iv::{Column as ModelColumn, Designs, TwoStage, estimate};
use zeff_regress::ols::Ols;
use zeff_results::absorb::EffectGroups;
use zeff_results::design::Design;
use zeff_results::estimator::Estimator;
use zeff_results::variance::{Clusters, Kind};
use zeff_testkit::{formula, root, terms};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("a fit reads no file")
    }
}

/// A model: its formula, what is instrumented and by what, and the options of the fit.
#[derive(Default, Clone)]
struct Model<'a> {
    formula: &'a str,
    endogenous: &'a str,
    instruments: &'a str,
    weights: Option<(&'a str, WeightKind)>,
    cluster: Option<&'a str>,
    block_rows: Option<usize>,
}

fn model<'a>(formula: &'a str, endogenous: &'a str, instruments: &'a str) -> Model<'a> {
    Model {
        formula,
        endogenous,
        instruments,
        ..Model::default()
    }
}

fn terms_of(text: &str) -> Vec<Term> {
    if text.trim().is_empty() {
        Vec::new()
    } else {
        terms(text)
    }
}

/// The fit with the messages of building it, or the error rendered.
fn fit(table: &Table, model: &Model<'_>) -> Result<(TwoStage, Vec<String>), String> {
    let script = format!(
        "let m = d.ivreg({}, instruments: {} ~ {})\n",
        model.formula, model.endogenous, model.instruments
    );
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(10, 15),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let frame = Frame::new(table.clone());
    let mut extra: Vec<&str> = Vec::new();
    extra.extend(model.weights.map(|(name, _)| name));
    extra.extend(model.cluster);
    let result = Designs::build(
        &mut ctx,
        &frame,
        &formula(model.formula),
        &terms_of(model.endogenous),
        &terms_of(model.instruments),
        &extra,
        model.weights,
    )
    .and_then(|mut designs| {
        if let Some(rows) = model.block_rows {
            designs.structural = designs.structural.with_block_rows(rows);
            designs.excluded = designs.excluded.with_block_rows(rows);
        }
        estimate(&mut ctx, designs)
    });
    let rendered: Vec<String> = messages.iter().map(|m| m.render(&sources)).collect();
    match result {
        Ok(fit) => Ok((fit, rendered)),
        Err(e) => Err(e.render(&sources)),
    }
}

/// The standard errors of the kept columns under `kind`.
fn std_errors(table: &Table, fit: &TwoStage, kind: Kind, cluster: Option<&str>) -> Vec<f64> {
    let clusters = cluster.map(|name| {
        let column = table.column(name).unwrap();
        let groups = EffectGroups::over(
            name,
            &[name.to_string()],
            &[column],
            &fit.designs.structural.rows,
        );
        Clusters::from_groups(&groups.group_of_row, groups.levels())
    });
    let (vcov, _) = fit
        .variance(kind, clusters.as_ref(), 0)
        .unwrap_or_else(|e| panic!("{kind:?}: {e:?}"));
    (0..vcov.rows()).map(|i| vcov.get(i, i).sqrt()).collect()
}

fn reference_data(file: &str) -> Table {
    let path = root()
        .join("tests/reference/data")
        .join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn number(value: &Json) -> f64 {
    match value {
        Json::Number(n) => n.as_f64().unwrap(),
        Json::String(s) => match s.as_str() {
            "NA" => f64::NAN,
            t => t.parse().unwrap(),
        },
        other => panic!("not a number: {other}"),
    }
}

fn numbers(value: &Json) -> Vec<f64> {
    value.as_array().unwrap().iter().map(number).collect()
}

fn texts(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect()
}

fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    (a - b).abs() <= relative * a.abs().max(b.abs()) + absolute
}

fn floats(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

fn table(columns: Vec<(&str, Vec<f64>)>) -> Table {
    Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), floats(&c)))
            .collect(),
    )
    .unwrap()
}

/// The estimates by the name of their column.
fn estimates(fit: &TwoStage) -> Vec<(String, f64)> {
    fit.fitted
        .kept
        .iter()
        .zip(&fit.fitted.estimates)
        .map(|(&c, b)| (fit.designs.structural.names[c].clone(), *b))
        .collect()
}

// ----- the reference: R by hand, held to fixest -----

#[test]
fn fits_agree_with_r() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/iv.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 11);
    for (name, case) in cases {
        let data = reference_data(case["data"].as_str().unwrap());
        let weights = case["weights"].as_str().map(|w| {
            let kind = match case["weight_type"].as_str().unwrap() {
                "analytic" => WeightKind::Analytic,
                "sampling" => WeightKind::Sampling,
                "frequency" => WeightKind::Frequency,
                other => panic!("{name}: weights of kind {other}"),
            };
            (w, kind)
        });
        let cluster = case["cluster"].as_str();
        let asked = Model {
            formula: case["formula"].as_str().unwrap(),
            endogenous: case["endogenous"].as_str().unwrap(),
            instruments: case["instruments"].as_str().unwrap(),
            weights,
            cluster,
            block_rows: None,
        };
        let (fit, _) = fit(&data, &asked).unwrap_or_else(|e| panic!("{name}: {e}"));
        let design = &fit.designs.structural;
        // The magnitudes case holds its estimates to the others' within 1e-6 in R itself.
        let tolerance = if name == "card_magnitudes" {
            1e-6
        } else {
            1e-8
        };
        assert_eq!(design.names, texts(&case["names"]), "{name}: names");
        assert_eq!(design.n() as u64, case["n"].as_u64().unwrap(), "{name}: n");
        assert_eq!(
            (design.dropped + design.zero_weights) as u64,
            case["dropped"].as_u64().unwrap(),
            "{name}: dropped"
        );
        assert!(fit.fitted.omitted.is_empty(), "{name}: nothing is dropped");
        let all = |ours: &[f64], theirs: &Json, what: &str| {
            let theirs = numbers(theirs);
            assert_eq!(ours.len(), theirs.len(), "{name}: {what}");
            for (i, (a, b)) in ours.iter().zip(&theirs).enumerate() {
                assert!(
                    close(*a, *b, tolerance, 0.0),
                    "{name}: {what} of `{}`: {a} against {b}",
                    design.names.get(i).map_or("", String::as_str)
                );
            }
        };
        all(&fit.fitted.estimates, &case["coefficients"], "estimate");
        assert!(
            close(
                fit.fitted.residual_sum_of_squares,
                number(&case["rss"]),
                tolerance,
                0.0
            ),
            "{name}: rss {} against {}",
            fit.fitted.residual_sum_of_squares,
            number(&case["rss"])
        );
        for (kind, key) in [
            (Kind::Classical, "se_classical"),
            (Kind::Hc1, "se_hc1"),
            (Kind::Hc2, "se_hc2"),
            (Kind::Hc3, "se_hc3"),
        ] {
            all(&std_errors(&data, &fit, kind, None), &case[key], key);
        }
        if let Some(cluster) = cluster {
            all(
                &std_errors(&data, &fit, Kind::Cluster, Some(cluster)),
                &case["se_cluster"],
                "se_cluster",
            );
        }
        // The first stages, by the instruments' names.
        let first = fit.first.as_ref().unwrap();
        let instruments: Vec<String> = first
            .kept
            .iter()
            .map(|&i| fit.designs.name(fit.designs.instruments[i]).to_string())
            .collect();
        assert_eq!(
            instruments,
            texts(&case["first_stage_names"]),
            "{name}: instruments"
        );
        for (ours, theirs) in first
            .coefficients
            .iter()
            .zip(case["first_stage"].as_array().unwrap())
        {
            let theirs = numbers(theirs);
            for ((a, b), instrument) in ours.iter().zip(&theirs).zip(&instruments) {
                assert!(
                    close(*a, *b, tolerance, 0.0),
                    "{name}: first stage on `{instrument}`: {a} against {b}"
                );
            }
        }
    }
}

/// Card's estimate of the return to schooling with proximity to a college as the instrument,
/// as the textbooks print it.
#[test]
fn the_return_to_schooling_of_card() {
    let card = reference_data("card");
    let asked = model(
        "lwage ~ exper + expersq + black + south + smsa + educ",
        "educ",
        "nearc4",
    );
    let (fit, messages) = fit(&card, &asked).unwrap();
    assert!(messages.is_empty(), "{messages:?}");
    let educ = estimates(&fit).iter().find(|(n, _)| n == "educ").unwrap().1;
    assert!((educ - 0.1323).abs() < 5e-5, "{educ}");
    assert_eq!(fit.designs.endogenous, vec![6]);
    assert_eq!(
        fit.designs.instruments.last(),
        Some(&ModelColumn::Excluded(1))
    );
    assert_eq!(fit.designs.excluded_instruments(), vec![6]);
}

// ----- the edge cases of the handbook -----

/// A small frame with a regressor that needs an instrument: `x` moves with `z1` and `z2` and
/// with the error of `y`; `w` is a regressor of its own.
fn small() -> Table {
    let z1 = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ];
    let z2 = [
        2.0, 1.0, 4.0, 3.0, 6.0, 5.0, 9.0, 7.0, 8.0, 12.0, 10.0, 11.5,
    ];
    let w = [
        0.5, -1.0, 0.25, 2.0, -0.5, 1.5, 0.0, -2.0, 1.0, 0.75, -1.5, 0.1,
    ];
    let u = [
        0.3, -0.2, 0.5, -0.4, 0.1, 0.6, -0.5, 0.2, -0.1, 0.4, -0.3, -0.6,
    ];
    let v = [
        0.2, 0.1, -0.3, 0.4, -0.2, 0.3, 0.1, -0.4, 0.2, -0.1, 0.3, -0.2,
    ];
    let x: Vec<f64> = (0..12)
        .map(|i| 1.0 + 0.8 * z1[i] + 0.4 * z2[i] + u[i] + v[i])
        .collect();
    let y: Vec<f64> = (0..12)
        .map(|i| 2.0 + 1.5 * x[i] - 0.7 * w[i] + 2.0 * u[i])
        .collect();
    table(vec![
        ("y", y),
        ("x", x),
        ("w", w.to_vec()),
        ("z1", z1.to_vec()),
        ("z2", z2.to_vec()),
    ])
}

fn with(table: &Table, name: &str, values: Vec<f64>) -> Table {
    table.with_column(name, floats(&values)).unwrap()
}

/// An instrument that does not vary is collinear with the intercept: dropped with a note when
/// another identifies the model, and the estimates are those without it.
#[test]
fn a_constant_instrument_is_dropped_with_a_note() {
    let t = with(&small(), "one", vec![3.0; 12]);
    let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1")).unwrap();
    let (fit, _) = fit(&t, &model("y ~ w + x", "x", "one + z1")).unwrap();
    assert_eq!(
        fit.notes,
        [
            "`one` is collinear with the regressors and instruments before it and was dropped from the instruments"
        ]
    );
    // The dropped column still passes through the factorisation, so the last bits may differ.
    for (a, b) in base.fitted.estimates.iter().zip(&fit.fitted.estimates) {
        assert!(close(*a, *b, 1e-12, 0.0), "{a} against {b}");
    }
}

/// An instrument that is a regressor of the formula is no excluded instrument; a copy of a
/// regressor under another name, or a combination of regressors, is collinear with them and is
/// dropped with a note.
#[test]
fn an_instrument_that_is_a_regressor_identifies_nothing() {
    let t = small();
    let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1")).unwrap();
    let (same, _) = fit(&t, &model("y ~ w + x", "x", "w + z1")).unwrap();
    assert!(same.notes.is_empty(), "{:?}", same.notes);
    assert_eq!(same.designs.excluded_instruments().len(), 1);
    assert_eq!(base.fitted.estimates, same.fitted.estimates);
    let w: Vec<f64> = t
        .column("w")
        .unwrap()
        .as_floats()
        .unwrap()
        .map(|v| 2.0 * v.unwrap() + 1.0)
        .collect();
    let t = with(&t, "w2", w);
    let (copy, _) = fit(&t, &model("y ~ w + x", "x", "w2 + z1")).unwrap();
    assert_eq!(
        copy.notes,
        [
            "`w2` is collinear with the regressors and instruments before it and was dropped from the instruments"
        ]
    );
    assert_eq!(base.fitted.estimates, copy.fitted.estimates);
}

/// A row with `NA` in an instrument is dropped and counted with the instrument's name.
#[test]
fn a_row_with_na_in_an_instrument_is_dropped_and_counted() {
    let t = small();
    let mut z1: Vec<Option<f64>> = t.column("z1").unwrap().as_floats().unwrap().collect();
    z1[3] = None;
    z1[7] = None;
    let t = t.with_column("z1", Column::floats(z1).0).unwrap();
    let (fit, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
    assert_eq!(fit.designs.structural.n(), 10);
    assert_eq!(fit.designs.structural.dropped, 2);
    assert_eq!(fit.designs.excluded.n(), 10);
    assert!(
        fit.designs
            .structural
            .dropped_in
            .contains(&("z1".to_string(), 2)),
        "{:?}",
        fit.designs.structural.dropped_in
    );
    assert_eq!(fit.fitted.residuals.len(), 10);
}

/// A regressor that its instruments fit exactly is least squares: the fitted regressor is the
/// regressor.
#[test]
fn a_perfect_first_stage_gives_least_squares() {
    let t = small();
    let z1: Vec<f64> = t
        .column("z1")
        .unwrap()
        .as_floats()
        .unwrap()
        .flatten()
        .collect();
    let z2: Vec<f64> = t
        .column("z2")
        .unwrap()
        .as_floats()
        .unwrap()
        .flatten()
        .collect();
    let exact: Vec<f64> = z1
        .iter()
        .zip(&z2)
        .map(|(a, b)| 1.0 + 0.5 * a - 0.25 * b)
        .collect();
    let t = with(&t, "p", exact);
    let (iv, _) = fit(&t, &model("y ~ w + p", "p", "z1 + z2")).unwrap();
    let (ols, _) = fit(&t, &model("y ~ w + p", "", "")).unwrap();
    assert!(ols.first.is_none());
    for (a, b) in iv.fitted.estimates.iter().zip(&ols.fitted.estimates) {
        assert!(close(*a, *b, 1e-10, 1e-12), "{a} against {b}");
    }
    let pi = &iv.first.as_ref().unwrap().coefficients[0];
    // On the intercept, `w`, `z1` and `z2`.
    for (a, b) in pi.iter().zip([1.0, 0.0, 0.5, -0.25]) {
        assert!(close(*a, b, 1e-10, 1e-12), "{a} against {b}");
    }
}

/// Nothing instrumented is `reg`: the same bits as least squares on the design.
#[test]
fn nothing_instrumented_is_least_squares_to_the_bit() {
    let card = reference_data("card");
    let text = "lwage ~ exper + expersq + black + south + smsa + educ";
    let (iv, _) = fit(&card, &model(text, "", "")).unwrap();
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "",
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(0, 1),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let design = Design::build(&mut ctx, &Frame::new(card.clone()), &formula(text), &[]).unwrap();
    let ols = Ols.fit(&mut ctx, &design).unwrap();
    assert_eq!(iv.fitted, ols);
    for kind in [Kind::Classical, Kind::Hc1, Kind::Hc2, Kind::Hc3] {
        let ours = std_errors(&card, &iv, kind, None);
        let (vcov, _) =
            zeff_results::variance::variance(kind, &Ols, &design, &ols, &Ols.bread(&ols), None, 0)
                .unwrap();
        for (i, a) in ours.iter().enumerate() {
            assert_eq!(a.to_bits(), vcov.get(i, i).sqrt().to_bits(), "{kind:?}");
        }
    }
}

/// Every refusal and every note, as rendered.
#[test]
fn the_notes_and_the_errors() {
    let t = with(&small(), "one", vec![3.0; 12]);
    let x2: Vec<f64> = t
        .column("x")
        .unwrap()
        .as_floats()
        .unwrap()
        .flatten()
        .zip(t.column("w").unwrap().as_floats().unwrap().flatten())
        .map(|(x, w)| 0.5 * x + w)
        .collect();
    let t = with(&t, "x2", x2);
    let t = with(&t, "zero", vec![0.0; 12]);
    // Two instruments orthogonal to each other and to the constant, and two regressors of
    // which the second is three times the first plus a part orthogonal to both instruments:
    // the instruments move the two only together.
    let a: Vec<f64> = (0..12)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let b: Vec<f64> = (0..12)
        .map(|i| if i % 4 < 2 { 1.0 } else { -1.0 })
        .collect();
    let ab: Vec<f64> = a.iter().zip(&b).map(|(a, b)| a * b).collect();
    let xa: Vec<f64> = (0..12)
        .map(|i| 1.0 + a[i] + 0.5 * b[i] + 0.25 * ab[i])
        .collect();
    let xb: Vec<f64> = (0..12).map(|i| 3.0 * xa[i] + ab[i]).collect();
    let t = with(&t, "a", a);
    let t = with(&t, "b", b);
    let t = with(&t, "xa", xa);
    let t = with(&t, "xb", xb);
    let one_row = table(vec![
        ("y", vec![1.0]),
        ("x", vec![2.0]),
        ("w", vec![0.5]),
        ("z1", vec![3.0]),
    ]);
    let cases: Vec<(&str, &Table, Model<'_>)> = vec![
        (
            "the only instrument is constant",
            &t,
            model("y ~ w + x", "x", "one"),
        ),
        (
            "the only instrument is a regressor",
            &t,
            model("y ~ w + x", "x", "w"),
        ),
        (
            "one instrument for two regressors",
            &t,
            model("y ~ x + x2", "x + x2", "z1"),
        ),
        (
            "two instruments of which one is zero, for two regressors",
            &t,
            model("y ~ x + x2", "x + x2", "z1 + zero"),
        ),
        (
            "two regressors the instruments move only together",
            &t,
            model("y ~ xa + xb", "xa + xb", "a + b"),
        ),
        (
            "an instrumented regressor missing from the formula",
            &t,
            model("y ~ w", "x", "z1"),
        ),
        (
            "an instrument that is instrumented",
            &t,
            model("y ~ w + x", "x", "x + z1"),
        ),
        (
            "a product instrumented as one term",
            &t,
            model("y ~ w * x", "w * x", "z1 + z2"),
        ),
        ("one row", &one_row, model("y ~ w + x", "x", "z1")),
        (
            "an instrument that is not a variable",
            &t,
            model("y ~ w + x", "x", "nowhere"),
        ),
        (
            "instruments dropped, and the model still identified",
            &t,
            model("y ~ w + x", "x", "zero + one + z1 + (z1 * 3) + z2"),
        ),
        (
            "a regressor collinear with the ones before it",
            &t,
            model("y ~ w + (w * 2) + x", "x", "z1"),
        ),
    ];
    let mut all = String::new();
    for (what, data, asked) in cases {
        all.push_str(&format!(
            "## {what}\n{} with instruments: {} ~ {}\n",
            asked.formula, asked.endogenous, asked.instruments
        ));
        match fit(data, &asked) {
            Ok((fit, messages)) => {
                for m in messages {
                    all.push_str(&m);
                }
                for note in &fit.notes {
                    all.push_str(&format!("note: {note}\n"));
                }
                let omitted: Vec<&str> = fit
                    .fitted
                    .omitted
                    .iter()
                    .map(|&c| fit.designs.structural.names[c].as_str())
                    .collect();
                all.push_str(&format!(
                    "fitted: {} estimates, omitted {omitted:?}\n",
                    fit.fitted.estimates.len()
                ));
            }
            Err(e) => all.push_str(&e),
        }
        all.push('\n');
    }
    insta::assert_snapshot!("iv_notes_and_errors", all);
}

/// An interaction with an instrumented regressor is instrumented when it is named on the left,
/// in either order of its parts, and a variable of a category is instrumented by its
/// indicators.
#[test]
fn interactions_and_categories_are_instrumented_by_their_terms() {
    let t = small();
    let asked = model("y ~ w + x + x:w", "x + w:x", "z1 + z2 + z1:w");
    let (fit, _) = fit(&t, &asked).unwrap();
    assert_eq!(
        fit.designs.structural.names,
        ["(intercept)", "w", "x", "x:w"]
    );
    assert_eq!(fit.designs.endogenous, vec![2, 3]);
    let asked = model("y ~ w * x", "x + w:x", "z1 + z2 + z1:w");
    let (product, _) = super_fit(&t, &asked);
    assert_eq!(product.designs.endogenous, vec![2, 3]);
    assert_eq!(product.fitted.estimates, fit.fitted.estimates);
    // Only what is named is instrumented: the interaction left out of `instruments:` stays a
    // regressor taken as given.
    let asked = model("y ~ w + x + x:w", "x", "z1 + z2");
    let (partly, _) = super_fit(&t, &asked);
    assert_eq!(partly.designs.endogenous, vec![2]);
}

fn super_fit(table: &Table, asked: &Model<'_>) -> (TwoStage, Vec<String>) {
    fit(table, asked).unwrap_or_else(|e| panic!("{e}"))
}

/// Variables far from the scale of one, each way: the estimates are those of the same model in
/// working units, rescaled, and no self-check takes a vanished or an overflowing norm for a
/// failure.
#[test]
fn sizes_far_from_one_pass_the_self_checks() {
    let t = small();
    let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
    let column = |name: &str| -> Vec<f64> {
        t.column(name)
            .unwrap()
            .as_floats()
            .unwrap()
            .flatten()
            .collect()
    };
    for (sy, sx, sz) in [
        (1e-200, 1.0, 1.0),
        (1e140, 1.0, 1.0),
        (1.0, 1e120, 1e-120),
        (1.0, 1e-120, 1e120),
        (1e-140, 1e100, 1e140),
        (1e100, 1e-100, 1e-100),
        (1e-60, 1e60, 1e-30),
        (1e40, 1e-70, 1e90),
    ] {
        let scaled =
            |name: &str, by: f64| -> Vec<f64> { column(name).iter().map(|v| v * by).collect() };
        let u = table(vec![
            ("y", scaled("y", sy)),
            ("x", scaled("x", sx)),
            ("w", column("w")),
            ("z1", scaled("z1", sz)),
            ("z2", scaled("z2", sz)),
        ]);
        let (fit, _) = fit(&u, &model("y ~ w + x", "x", "z1 + z2"))
            .unwrap_or_else(|e| panic!("{sy:e}, {sx:e}, {sz:e}: {e}"));
        let expected = [
            base.fitted.estimates[0] * sy,
            base.fitted.estimates[1] * sy,
            base.fitted.estimates[2] * sy / sx,
        ];
        for (a, b) in fit.fitted.estimates.iter().zip(expected) {
            assert!(
                close(*a, b, 1e-9, 0.0),
                "{sy:e}, {sx:e}, {sz:e}: {a} against {b}"
            );
        }
        // A standard error is the root of a variance, which a double holds only when the
        // error's square is within its range (as for `reg`).
        let se = std_errors(&u, &fit, Kind::Hc1, None);
        let base_se = std_errors(&t, &base, Kind::Hc1, None);
        let expected = base_se[2] * sy / sx;
        if (1e-150..1e150).contains(&expected) {
            assert!(
                close(se[2], expected, 1e-8, 0.0),
                "{sy:e}, {sx:e}, {sz:e}: {} against {expected}",
                se[2]
            );
        }
    }
}

// ----- properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A frame from random values: two instruments, a regressor of its own, an instrumented
/// regressor that moves with the instruments and with the outcome's error.
fn frame_of(values: &[(f64, f64, f64, f64, f64)]) -> Table {
    let z1: Vec<f64> = values.iter().map(|v| v.0).collect();
    let z2: Vec<f64> = values.iter().map(|v| v.1).collect();
    let w: Vec<f64> = values.iter().map(|v| v.2).collect();
    let x: Vec<f64> = values
        .iter()
        .map(|v| 0.5 + v.0 - 0.6 * v.1 + 0.3 * v.2 + 0.4 * v.3 + 0.2 * v.4)
        .collect();
    let y: Vec<f64> = values
        .iter()
        .zip(&x)
        .map(|(v, x)| 1.0 + 2.0 * x - v.2 + v.3)
        .collect();
    table(vec![("y", y), ("x", x), ("w", w), ("z1", z1), ("z2", z2)])
}

fn rows() -> impl Strategy<Value = Vec<(f64, f64, f64, f64, f64)>> {
    prop::collection::vec(
        (
            -3.0f64..3.0,
            -3.0f64..3.0,
            -3.0f64..3.0,
            -1.0f64..1.0,
            -1.0f64..1.0,
        ),
        12..50,
    )
}

/// The instruments tell the model apart only when they vary on their own: a draw whose
/// instruments are nearly collinear among themselves or with `w` is no case of these
/// properties.
fn well_identified(fit: &TwoStage) -> bool {
    fit.fitted.condition_number < 1e3
        && fit.first.as_ref().is_some_and(|f| f.condition_number < 1e3)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(25)))]

    /// Doubling the outcome doubles every estimate to the bit, and scaling it by three scales
    /// them within rounding.
    #[test]
    fn scaling_the_outcome_scales_the_coefficients(values in rows()) {
        let t = frame_of(&values);
        let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&base));
        let (doubled, _) = fit(&t, &model("(y * 2) ~ w + x", "x", "z1 + z2")).unwrap();
        for (b, d) in base.fitted.estimates.iter().zip(&doubled.fitted.estimates) {
            prop_assert_eq!((b * 2.0).to_bits(), d.to_bits());
        }
        let (tripled, _) = fit(&t, &model("(y * 3) ~ w + x", "x", "z1 + z2")).unwrap();
        for (b, d) in base.fitted.estimates.iter().zip(&tripled.fitted.estimates) {
            prop_assert!(close(b * 3.0, *d, 1e-11, 1e-13), "{} against {}", b * 3.0, d);
        }
        let a = std_errors(&t, &base, Kind::Hc1, None);
        let b = std_errors(&t, &tripled, Kind::Hc1, None);
        for (a, b) in a.iter().zip(&b) {
            prop_assert!(close(a * 3.0, *b, 1e-9, 1e-13), "{} against {}", a * 3.0, b);
        }
    }

    /// An instrument in other units changes no estimate and no standard error.
    #[test]
    fn rescaling_an_instrument_changes_no_estimate(values in rows()) {
        let t = frame_of(&values);
        let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&base));
        let (scaled, _) = fit(&t, &model("y ~ w + x", "x", "(z1 * 1e6) + z2")).unwrap();
        for (a, b) in base.fitted.estimates.iter().zip(&scaled.fitted.estimates) {
            prop_assert!(close(*a, *b, 1e-9, 1e-12), "{} against {}", a, b);
        }
        for kind in [Kind::Classical, Kind::Hc1, Kind::Hc3] {
            let a = std_errors(&t, &base, kind, None);
            let b = std_errors(&t, &scaled, kind, None);
            for (a, b) in a.iter().zip(&b) {
                prop_assert!(close(*a, *b, 1e-8, 1e-12), "{:?}: {} against {}", kind, a, b);
            }
        }
    }

    /// The rows in another order give the same estimates and standard errors.
    #[test]
    fn reordering_the_rows_changes_nothing(values in rows()) {
        let t = frame_of(&values);
        let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&base));
        let mut turned = values.clone();
        turned.reverse();
        let u = frame_of(&turned);
        let (other, _) = fit(&u, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        for (a, b) in base.fitted.estimates.iter().zip(&other.fitted.estimates) {
            prop_assert!(close(*a, *b, 1e-9, 1e-12), "{} against {}", a, b);
        }
        let a = std_errors(&t, &base, Kind::Hc2, None);
        let b = std_errors(&u, &other, Kind::Hc2, None);
        for (a, b) in a.iter().zip(&b) {
            prop_assert!(close(*a, *b, 1e-8, 1e-12), "{} against {}", a, b);
        }
    }

    /// With one instrument for one regressor and nothing else, the estimate is the ratio of the
    /// two reduced forms: the slope of the outcome on the instrument over the slope of the
    /// regressor on it.
    #[test]
    fn one_instrument_gives_the_ratio_of_the_reduced_forms(values in rows()) {
        let t = frame_of(&values);
        let (iv, _) = fit(&t, &model("y ~ x", "x", "z1")).unwrap();
        prop_assume!(well_identified(&iv));
        let column = |name: &str| -> Vec<f64> { t.column(name).unwrap().as_floats().unwrap().flatten().collect() };
        let (y, x, z) = (column("y"), column("x"), column("z1"));
        let n = y.len() as f64;
        let mean = |v: &[f64]| v.iter().sum::<f64>() / n;
        let (my, mx, mz) = (mean(&y), mean(&x), mean(&z));
        let with_z = |v: &[f64], m: f64| -> f64 { v.iter().zip(&z).map(|(a, b)| (a - m) * (b - mz)).sum() };
        let ratio = with_z(&y, my) / with_z(&x, mx);
        prop_assume!(with_z(&x, mx).abs() > 1.0);
        let slope = iv.fitted.estimates[1];
        prop_assert!(close(slope, ratio, 1e-9, 1e-11), "{} against {}", slope, ratio);
        prop_assert!(close(iv.fitted.estimates[0], my - ratio * mx, 1e-9, 1e-10));
    }

    /// The designs in blocks of any size give the same bits, in both stages.
    #[test]
    fn any_block_size_gives_the_same_bits(values in rows(), block_rows in 1usize..20) {
        let t = frame_of(&values);
        let asked = model("y ~ w + x", "x", "z1 + z2");
        let (whole, _) = fit(&t, &asked).unwrap();
        prop_assume!(well_identified(&whole));
        let (blocked, _) = fit(&t, &Model { block_rows: Some(block_rows), ..asked }).unwrap();
        prop_assert_eq!(&whole.first.as_ref().unwrap().coefficients, &blocked.first.as_ref().unwrap().coefficients);
        for (a, b) in whole.fitted.estimates.iter().zip(&blocked.fitted.estimates) {
            prop_assert_eq!(a.to_bits(), b.to_bits());
        }
        for (a, b) in whole.fitted.residuals.iter().zip(&blocked.fitted.residuals) {
            prop_assert_eq!(a.to_bits(), b.to_bits());
        }
        prop_assert_eq!(whole.fitted.residual_sum_of_squares.to_bits(), blocked.fitted.residual_sum_of_squares.to_bits());
    }

    /// What two-stage least squares is: the first stage's residuals are orthogonal to every
    /// instrument, and the structural residuals to every fitted regressor and so to every
    /// regressor that is not instrumented.
    #[test]
    fn the_residuals_are_orthogonal_to_what_they_should_be(values in rows()) {
        let t = frame_of(&values);
        let (fit, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&fit));
        let x = fit.designs.structural.column_values(2);
        let fitted = fit.second.column_values(2);
        let scale: f64 = x.iter().map(|v| v * v).sum::<f64>().sqrt();
        for &i in &fit.first.as_ref().unwrap().kept {
            let z = fit.designs.values(fit.designs.instruments[i]);
            let norm: f64 = z.iter().map(|v| v * v).sum::<f64>().sqrt();
            let product: f64 = z.iter().zip(x.iter().zip(&fitted)).map(|(z, (x, f))| z * (x - f)).sum();
            prop_assert!(product.abs() <= 1e-10 * norm * scale, "{}", product);
        }
        let e = &fit.fitted.residuals;
        let size: f64 = e.iter().map(|v| v * v).sum::<f64>().sqrt();
        for c in 0..3 {
            let column = fit.second.column_values(c);
            let norm: f64 = column.iter().map(|v| v * v).sum::<f64>().sqrt();
            let product: f64 = column.iter().zip(e).map(|(a, b)| a * b).sum();
            prop_assert!(product.abs() <= 1e-9 * norm * (size + 1.0), "{}", product);
        }
    }
}
