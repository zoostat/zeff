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
use zeff_regress::iv::{
    AndersonRubin, Column as ModelColumn, Designs, Diagnostics, Projection, Shape, TwoStage, Weak,
    estimate,
};
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
    fe: Option<&'a str>,
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
    fit_projected(table, model).map(|(fit, _, messages)| (fit, messages))
}

/// The fit, what projecting the fixed effects out found when the model has `fe:`, and the
/// messages; or the error rendered.
fn fit_projected(
    table: &Table,
    model: &Model<'_>,
) -> Result<(TwoStage, Option<Projection>, Vec<String>), String> {
    in_context(table, model, |_, fit, projection| Ok((fit, projection)))
        .map(|((fit, projection), messages)| (fit, projection, messages))
}

/// The model fitted, then `then` with the context of the call, the fit and its projection; the
/// value with the messages, or the error rendered.
fn in_context<T>(
    table: &Table,
    model: &Model<'_>,
    then: impl FnOnce(&mut CallContext<'_>, TwoStage, Option<Projection>) -> Result<T, Fail>,
) -> Result<(T, Vec<String>), String> {
    let script = format!(
        "let m = d.ivreg({}, {}instruments: {} ~ {})\n",
        model.formula,
        model.fe.map_or(String::new(), |fe| format!("fe: {fe}, ")),
        model.endogenous,
        model.instruments
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
    let built = match model.fe {
        Some(fe) => Designs::build_absorbing(
            &mut ctx,
            &frame,
            &formula(model.formula),
            &terms_of(model.endogenous),
            &terms_of(model.instruments),
            &extra,
            model.weights,
            &terms(fe),
        )
        .map(|(designs, projection)| (designs, Some(projection))),
        None => Designs::build(
            &mut ctx,
            &frame,
            &formula(model.formula),
            &terms_of(model.endogenous),
            &terms_of(model.instruments),
            &extra,
            model.weights,
        )
        .map(|designs| (designs, None)),
    };
    let result = built.and_then(|(mut designs, projection)| {
        if let Some(rows) = model.block_rows {
            designs.structural = designs.structural.with_block_rows(rows);
            designs.excluded = designs.excluded.with_block_rows(rows);
        }
        let fit = estimate(&mut ctx, designs)?;
        then(&mut ctx, fit, projection)
    });
    let rendered: Vec<String> = messages.iter().map(|m| m.render(&sources)).collect();
    match result {
        Ok(value) => Ok((value, rendered)),
        Err(e) => Err(e.render(&sources)),
    }
}

/// The clusters of the rows a fit uses, by the variable `name`.
fn clusters_of(table: &Table, fit: &TwoStage, name: &str) -> Clusters {
    let column = table.column(name).unwrap();
    let groups = EffectGroups::over(
        name,
        &[name.to_string()],
        &[column],
        &fit.designs.structural.rows,
    );
    Clusters::from_groups(&groups.group_of_row, groups.levels())
}

/// The diagnostics of the first stage of a model, on the variance of `kind`.
fn diagnostics_of(
    table: &Table,
    model: &Model<'_>,
    kind: Kind,
    cluster: Option<&str>,
) -> Diagnostics {
    in_context(table, model, |ctx, fit, projection| {
        let clusters = cluster.map(|name| clusters_of(table, &fit, name));
        fit.diagnostics(ctx, kind, clusters.as_ref(), projection.as_ref())
    })
    .unwrap_or_else(|e| panic!("{e}"))
    .0
}

/// The Anderson–Rubin test of a model and its confidence set at `level`.
fn anderson_rubin_of(
    table: &Table,
    model: &Model<'_>,
    kind: Kind,
    cluster: Option<&str>,
    level: f64,
) -> AndersonRubin {
    in_context(table, model, |ctx, fit, projection| {
        let clusters = cluster.map(|name| clusters_of(table, &fit, name));
        fit.anderson_rubin(ctx, kind, clusters.as_ref(), projection.as_ref(), level)
    })
    .unwrap_or_else(|e| panic!("{e}"))
    .0
}

/// The weak-instrument statistics of a model, with the Wald form on the variance of `kind`.
fn weak_of(table: &Table, model: &Model<'_>, kind: Kind, cluster: Option<&str>) -> Weak {
    in_context(table, model, |ctx, fit, projection| {
        let clusters = cluster.map(|name| clusters_of(table, &fit, name));
        fit.weak(ctx, kind, clusters.as_ref(), projection.as_ref())
    })
    .unwrap_or_else(|e| panic!("{e}"))
    .0
}

/// The standard errors of the kept columns under `kind`.
fn std_errors(table: &Table, fit: &TwoStage, kind: Kind, cluster: Option<&str>) -> Vec<f64> {
    std_errors_projected(table, fit, None, kind, cluster)
}

/// The standard errors with the parameters of the absorbed effects in every N − k.
fn std_errors_projected(
    table: &Table,
    fit: &TwoStage,
    projection: Option<&Projection>,
    kind: Kind,
    cluster: Option<&str>,
) -> Vec<f64> {
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
    let absorbed = projection.map_or(0, |p| p.parameters(clusters.as_ref()));
    let (vcov, _) = fit
        .variance(kind, clusters.as_ref(), absorbed)
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
            ..Model::default()
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

// ----- with fixed effects: fixest -----

/// The cases with `fe:` against fixest: the estimates, the residual sum of squares, and the
/// classical, robust and clustered errors with the degrees of freedom of the ruling of 0.6.0;
/// the rows dropped for `NA` and as singletons counted. The first case is the model of the
/// specification's sample script.
#[test]
fn fits_with_fixed_effects_agree_with_fixest() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/iv.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let cases = reference["fixed_effects"].as_object().unwrap();
    assert_eq!(cases.len(), 7);
    let card = reference_data("card");
    for (name, case) in cases {
        let weights = case["weights"].as_str().map(|w| (w, WeightKind::Analytic));
        let asked = Model {
            formula: case["formula"].as_str().unwrap(),
            endogenous: case["endogenous"].as_str().unwrap(),
            instruments: case["instruments"].as_str().unwrap(),
            fe: case["fe"].as_str(),
            weights,
            ..Model::default()
        };
        let (fit, projection, _) =
            fit_projected(&card, &asked).unwrap_or_else(|e| panic!("{name}: {e}"));
        let projection = projection.unwrap();
        let design = &fit.designs.structural;
        // The effects are projected out by an iteration to 1e-8 on each column; one effect is
        // exact in a pass.
        let tolerance = 1e-7;
        assert_eq!(design.names, texts(&case["names"]), "{name}: names");
        assert_eq!(design.n() as u64, case["n"].as_u64().unwrap(), "{name}: n");
        assert_eq!(
            projection.original.dropped as u64,
            case["dropped"].as_u64().unwrap(),
            "{name}: dropped"
        );
        assert_eq!(
            projection.absorbed.singletons as u64,
            case["singletons"].as_u64().unwrap(),
            "{name}: singletons"
        );
        let levels: Vec<u64> = projection
            .absorbed
            .effects
            .iter()
            .map(|e| e.levels() as u64)
            .collect();
        let theirs: Vec<u64> = case["fixef_sizes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(levels, theirs, "{name}: levels");
        assert!(fit.fitted.omitted.is_empty(), "{name}: nothing is dropped");
        let all = |ours: &[f64], theirs: &Json, what: &str| {
            let theirs = numbers(theirs);
            assert_eq!(ours.len(), theirs.len(), "{name}: {what}");
            for (i, (a, b)) in ours.iter().zip(&theirs).enumerate() {
                assert!(
                    close(*a, *b, tolerance, 0.0),
                    "{name}: {what} of `{}`: {a} against {b}",
                    design.names[i]
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
        for (kind, key) in [(Kind::Classical, "se_classical"), (Kind::Hc1, "se_hc1")] {
            let ours = std_errors_projected(&card, &fit, Some(&projection), kind, None);
            all(&ours, &case[key], key);
        }
        if let Some(clustered) = case["se_cluster"].as_object() {
            for (cluster, theirs) in clustered {
                let ours = std_errors_projected(
                    &card,
                    &fit,
                    Some(&projection),
                    Kind::Cluster,
                    Some(cluster),
                );
                all(&ours, theirs, &format!("se clustered by {cluster}"));
            }
        }
        // Under absorbed effects an observation's leverage is not the design's to give.
        assert!(!fit.second.leverage_available(), "{name}");
    }
}

/// An effect by `fe:` is its indicators written out: the same estimates and the same classical
/// and robust errors, on a frame with an instrument that varies within the groups.
#[test]
fn one_effect_by_fe_is_its_indicators_written_out() {
    let card = reference_data("card");
    let by_fe = Model {
        fe: Some("region"),
        ..model("lwage ~ exper + black + educ", "educ", "nearc4 + nearc2")
    };
    let (absorbed, projection, _) = fit_projected(&card, &by_fe).unwrap();
    let written = model(
        "lwage ~ exper + black + educ + category(region)",
        "educ",
        "nearc4 + nearc2",
    );
    let (indicators, _) = fit(&card, &written).unwrap();
    let theirs = estimates(&indicators);
    for (name, ours) in estimates(&absorbed) {
        let other = theirs.iter().find(|(n, _)| *n == name).unwrap().1;
        assert!(
            close(ours, other, 1e-9, 0.0),
            "{name}: {ours} against {other}"
        );
    }
    for kind in [Kind::Classical, Kind::Hc1] {
        let ours = std_errors_projected(&card, &absorbed, projection.as_ref(), kind, None);
        let other = std_errors(&card, &indicators, kind, None);
        for (i, a) in ours.iter().enumerate() {
            let name = &absorbed.designs.structural.names[i];
            let j = indicators
                .designs
                .structural
                .names
                .iter()
                .position(|n| n == name)
                .unwrap();
            assert!(
                close(*a, other[j], 1e-8, 0.0),
                "{kind:?} of {name}: {a} against {}",
                other[j]
            );
        }
    }
}

/// With `fe:`: an instrument that does not vary within the effects' levels is absorbed with a
/// note, and alone it identifies nothing; a fixed effect that is a variable of the formula or
/// an instrument is refused; every row a singleton leaves nothing.
#[test]
fn the_notes_and_the_errors_with_fixed_effects() {
    let t = small();
    let g: Vec<f64> = (0..12).map(|i| f64::from(i % 3)).collect();
    let by_group: Vec<f64> = g.iter().map(|g| 10.0 * g + 1.0).collect();
    let each: Vec<f64> = (0..12).map(f64::from).collect();
    let t = with(&t, "g", g);
    let t = with(&t, "zg", by_group);
    let t = with(&t, "id", each);
    let cases: Vec<(&str, Model<'_>)> = vec![
        (
            "an instrument constant within every level, beside one that varies",
            Model {
                fe: Some("g"),
                ..model("y ~ w + x", "x", "zg + z1")
            },
        ),
        (
            "the only instrument constant within every level",
            Model {
                fe: Some("g"),
                ..model("y ~ w + x", "x", "zg")
            },
        ),
        (
            "a fixed effect that is a regressor",
            Model {
                fe: Some("w"),
                ..model("y ~ w + x", "x", "z1")
            },
        ),
        (
            "a fixed effect that is an instrument",
            Model {
                fe: Some("g + z1"),
                ..model("y ~ w + x", "x", "z1 + z2")
            },
        ),
        (
            "a fixed effect that is instrumented",
            Model {
                fe: Some("x"),
                ..model("y ~ w + x", "x", "z1")
            },
        ),
        (
            "every row a singleton",
            Model {
                fe: Some("id"),
                ..model("y ~ w + x", "x", "z1")
            },
        ),
        (
            "a fixed effect that is not a variable",
            Model {
                fe: Some("nowhere"),
                ..model("y ~ w + x", "x", "z1")
            },
        ),
    ];
    let mut all = String::new();
    for (what, asked) in cases {
        all.push_str(&format!(
            "## {what}\n{} with fe: {} and instruments: {} ~ {}\n",
            asked.formula,
            asked.fe.unwrap(),
            asked.endogenous,
            asked.instruments
        ));
        match fit_projected(&t, &asked) {
            Ok((fit, projection, messages)) => {
                for m in messages {
                    all.push_str(&m);
                }
                for note in &fit.notes {
                    all.push_str(&format!("note: {note}\n"));
                }
                all.push_str(&format!(
                    "fitted: {} estimates on {} rows, {} singletons\n",
                    fit.fitted.estimates.len(),
                    fit.designs.structural.n(),
                    projection.unwrap().absorbed.singletons
                ));
            }
            Err(e) => all.push_str(&e),
        }
        all.push('\n');
    }
    insta::assert_snapshot!("iv_notes_and_errors_with_fixed_effects", all);
}

// ----- the shipped `staff` data -----

/// The synthetic `staff` data (release 0.8.0, step 13a) do what they were made for. The course
/// raises productivity by 4.5; least squares within the sites gives 5.93, far above it, since
/// the able take the course more; two-stage least squares with the offer, drawn by lot, gives
/// 4.60 with a standard error of 0.43 (fixest's figures), within sampling error of the truth,
/// and so does the model with the distance as a second instrument; the first stage is strong.
#[test]
fn the_effect_of_the_course_in_staff_is_recovered() {
    let staff = zeff_datasets::table("staff").expect("the shipped staff");
    let by_offer = Model {
        fe: Some("site"),
        ..model("productivity ~ training", "training", "offer")
    };
    let (iv, projection, messages) = fit_projected(&staff, &by_offer).unwrap();
    assert!(messages.is_empty(), "{messages:?}");
    let estimate = iv.fitted.estimates[0];
    let se = std_errors_projected(&staff, &iv, projection.as_ref(), Kind::Classical, None)[0];
    assert!(close(estimate, 4.602_651_8, 1e-7, 0.0), "{estimate}");
    assert!(close(se, 0.429_463_4, 1e-6, 0.0), "{se}");
    assert!(
        (estimate - 4.5).abs() < se,
        "{estimate} with standard error {se}"
    );
    let least_squares = Model {
        fe: Some("site"),
        ..model("productivity ~ training", "", "")
    };
    let (ols, projection, _) = fit_projected(&staff, &least_squares).unwrap();
    let biased = ols.fitted.estimates[0];
    let its_se = std_errors_projected(&staff, &ols, projection.as_ref(), Kind::Classical, None)[0];
    assert!(close(biased, 5.926_464_5, 1e-7, 0.0), "{biased}");
    assert!(
        biased - 4.5 > 6.0 * its_se,
        "{biased} with standard error {its_se}"
    );
    let both = Model {
        fe: Some("site"),
        ..model(
            "productivity ~ age + training",
            "training",
            "offer + distance",
        )
    };
    let (over, projection, _) = fit_projected(&staff, &both).unwrap();
    let training = estimates(&over)
        .iter()
        .find(|(n, _)| n == "training")
        .unwrap()
        .1;
    let its_se = std_errors_projected(&staff, &over, projection.as_ref(), Kind::Hc1, None)[1];
    assert!(close(training, 4.581_589_0, 1e-7, 0.0), "{training}");
    assert!((training - 4.5).abs() < its_se);
    let strength = weak_of(&staff, &both, Kind::Hc1, None);
    assert!(
        close(strength.f[0].unwrap(), 386.049_79, 1e-7, 0.0),
        "{:?}",
        strength.f
    );
    assert!(
        close(strength.wald[0].unwrap(), 396.577_86, 1e-7, 0.0),
        "{:?}",
        strength.wald
    );
    assert_eq!(strength.warning(), None);
}

// ----- the weak-instrument statistics -----

/// The first-stage F of each instrumented regressor, its Wald form on the robust and the
/// clustered variance, and the Cragg–Donald F, against `iv.json` (base R held to fixest): one
/// and two instrumented regressors, one instrument, weights of the three kinds, an absorbed
/// effect with the clusters outside it and on it, and rows with `NA`.
#[test]
fn the_weak_instrument_statistics_agree_with_r() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/iv.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let cases = reference["weak"].as_object().unwrap();
    assert_eq!(cases.len(), 7);
    for (name, case) in cases {
        let data = reference_data(case["data"].as_str().unwrap());
        let weights = case["weights"].as_str().map(|w| {
            let kind = match case["weight_type"].as_str().unwrap() {
                "analytic" => WeightKind::Analytic,
                "sampling" => WeightKind::Sampling,
                _ => WeightKind::Frequency,
            };
            (w, kind)
        });
        let asked = Model {
            formula: case["formula"].as_str().unwrap(),
            endogenous: case["endogenous"].as_str().unwrap(),
            instruments: case["instruments"].as_str().unwrap(),
            fe: case["fe"].as_str(),
            weights,
            ..Model::default()
        };
        let tolerance = 1e-7;
        let all = |ours: &[Option<f64>], theirs: &Json, what: &str| {
            let theirs = numbers(theirs);
            assert_eq!(ours.len(), theirs.len(), "{name}: {what}");
            for (a, b) in ours.iter().zip(&theirs) {
                let a = a.unwrap_or_else(|| panic!("{name}: {what} is not available"));
                assert!(
                    close(a, *b, tolerance, 0.0),
                    "{name}: {what}: {a} against {b}"
                );
            }
        };
        let classical = weak_of(&data, &asked, Kind::Classical, None);
        assert_eq!(
            classical.df1 as u64,
            case["df1"].as_u64().unwrap(),
            "{name}: df1"
        );
        assert_eq!(classical.df2, number(&case["df2"]), "{name}: df2");
        all(&classical.f, &case["f"], "F");
        assert_eq!(
            classical.f, classical.wald,
            "{name}: the Wald form under classical errors"
        );
        all(
            &[classical.cragg_donald],
            &Json::Array(vec![case["cragg_donald"].clone()]),
            "Cragg-Donald F",
        );
        let robust = weak_of(&data, &asked, Kind::Hc1, None);
        assert_eq!(robust.f, classical.f, "{name}");
        assert_eq!(robust.cragg_donald, classical.cragg_donald, "{name}");
        all(&robust.wald, &case["wald_hc1"], "robust Wald F");
        if let Some(clustered) = case["wald_cluster"].as_object() {
            for (cluster, theirs) in clustered {
                let with = Model {
                    cluster: Some(cluster),
                    ..asked.clone()
                };
                let found = weak_of(&data, &with, Kind::Cluster, Some(cluster));
                all(
                    &found.wald,
                    theirs,
                    &format!("Wald F clustered by {cluster}"),
                );
            }
        }
        // With one instrumented regressor the Cragg-Donald F is its first-stage F.
        if classical.f.len() == 1 {
            let (cd, f) = (classical.cragg_donald.unwrap(), classical.f[0].unwrap());
            assert!(close(cd, f, 1e-10, 0.0), "{name}: {cd} against {f}");
        }
        // The Kleibergen-Paap rk Wald F (step 12c): under classical errors it is the
        // Cragg-Donald F; under robust and clustered errors R's by the formula.
        let (kp, cd) = (
            classical.kleibergen_paap.unwrap(),
            classical.cragg_donald.unwrap(),
        );
        assert!(
            close(kp, cd, 1e-8, 0.0),
            "{name}: rk F {kp} against the Cragg-Donald F {cd}"
        );
        assert_eq!(
            classical.effective, None,
            "{name}: no effective F under classical errors"
        );
        all(
            &[robust.kleibergen_paap],
            &Json::Array(vec![case["kp_hc1"].clone()]),
            "robust rk F",
        );
        let effective = |found: &Weak, theirs: &Json, what: &str| {
            let ours = found
                .effective
                .unwrap_or_else(|| panic!("{name}: no effective F, {what}"));
            for (a, key) in [
                (ours.f, "f"),
                (ours.degrees, "k"),
                (ours.critical, "critical"),
            ] {
                let b = number(&theirs[key]);
                assert!(
                    close(a, b, tolerance, 0.0),
                    "{name}: effective F, {what}, {key}: {a} against {b}"
                );
            }
        };
        if robust.f.len() == 1 {
            effective(&robust, &case["effective_hc1"], "robust");
        } else {
            assert_eq!(
                robust.effective, None,
                "{name}: the effective F is for one regressor"
            );
        }
        if let Some(clustered) = case["kp_cluster"].as_object() {
            for (cluster, theirs) in clustered {
                let with = Model {
                    cluster: Some(cluster),
                    ..asked.clone()
                };
                let found = weak_of(&data, &with, Kind::Cluster, Some(cluster));
                all(
                    &[found.kleibergen_paap],
                    &Json::Array(vec![theirs.clone()]),
                    &format!("rk F clustered by {cluster}"),
                );
                if found.f.len() == 1 {
                    effective(
                        &found,
                        &case["effective_cluster"][cluster],
                        &format!("clustered by {cluster}"),
                    );
                }
            }
        }
    }
}

/// The upper 5% points of non-central chi-square distributions, as the critical value of the
/// effective F reads them, against R's `qchisq`: for one degree of freedom and ten times the
/// non-centrality the point is 23.1, the figure of Montiel Olea and Pflueger's table.
#[test]
fn the_non_central_chi_square_points_agree_with_r() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/iv.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let points = reference["critical_points"].as_array().unwrap();
    assert_eq!(points.len(), 9);
    for point in points {
        let k = number(&point["k"]);
        let want = number(&point["critical"]);
        let got = zeff_regress::iv::noncentral_chi_square_point(k, 10.0 * k, 0.05) / k;
        assert!(
            close(got, want, 1e-10, 0.0),
            "{k} degrees: {got} against {want}"
        );
    }
    let one = zeff_regress::iv::noncentral_chi_square_point(1.0, 10.0, 0.05);
    assert!((one - 23.1085).abs() < 1e-4, "{one}");
    // Without non-centrality it is the chi-square's own point: 3.841 for one degree.
    let central = zeff_regress::iv::noncentral_chi_square_point(1.0, 0.0, 0.05);
    assert!((central - 3.841_458_820_694_124).abs() < 1e-10, "{central}");
}

/// The joint variance of the first stages holds each stage's own variance on its diagonal, to
/// the bit, under every kind; and with one stage it is that stage's variance.
#[test]
fn the_joint_variance_of_the_first_stages_holds_each_stage_s_own() {
    let card = reference_data("card");
    let asked = Model {
        cluster: Some("age"),
        ..model(
            "lwage ~ black + south + smsa + educ + KWW",
            "educ + KWW",
            "nearc4 + nearc2 + fatheduc + motheduc",
        )
    };
    in_context(&card, &asked, |ctx, fit, _| {
        let clusters = clusters_of(&card, &fit, "age");
        let stages = fit.first_stages(ctx)?;
        assert_eq!(stages.len(), 2);
        let fits: Vec<&zeff_results::estimator::Fitted> =
            stages.iter().map(|s| &s.fitted).collect();
        let bread = Ols.bread(&stages[0].fitted);
        let k = stages[0].fitted.kept.len();
        for kind in [
            Kind::Classical,
            Kind::Hc1,
            Kind::Hc2,
            Kind::Hc3,
            Kind::Cluster,
        ] {
            let given = (kind == Kind::Cluster).then_some(&clusters);
            let joint = zeff_results::variance::joint_variance(
                kind,
                &Ols,
                &stages[0].design,
                &fits,
                &bread,
                given,
                0,
            )
            .unwrap();
            assert_eq!(joint.rows(), 2 * k);
            for (j, stage) in stages.iter().enumerate() {
                let (own, _) = zeff_results::variance::variance(
                    kind,
                    &Ols,
                    &stage.design,
                    &stage.fitted,
                    &bread,
                    given,
                    0,
                )
                .unwrap();
                for a in 0..k {
                    for b in 0..k {
                        assert_eq!(
                            joint.get(j * k + a, j * k + b).to_bits(),
                            own.get(a, b).to_bits(),
                            "{kind:?}, stage {j}, entry ({a}, {b})"
                        );
                    }
                }
                let alone = zeff_results::variance::joint_variance(
                    kind,
                    &Ols,
                    &stage.design,
                    &[&stage.fitted],
                    &bread,
                    given,
                    0,
                )
                .unwrap();
                assert_eq!(alone, own, "{kind:?}, stage {j} alone");
            }
            // The covariance of the two stages' coefficients is the transpose of itself read
            // the other way.
            for a in 0..k {
                for b in 0..k {
                    assert_eq!(joint.get(a, k + b).to_bits(), joint.get(k + b, a).to_bits());
                }
            }
        }
        Ok(())
    })
    .unwrap();
}

/// The warning for weak instruments (W1007), in each of its forms, and its absence: an
/// instrument weak by construction under classical errors (the Cragg–Donald F) and under
/// robust ones (the Kleibergen–Paap F and the effective F); Card's two indicators, whose
/// robust F is just under ten; `wage2`, whose clustered F is above ten and whose effective F is
/// under its critical value; and strong instruments, which warn of nothing.
#[test]
fn the_warning_for_weak_instruments() {
    // `x` moves with the outcome's error far more than with `z1`.
    let n = 60u32;
    let z: Vec<f64> = (0..n).map(|i| f64::from((i * 37) % 11) - 5.0).collect();
    let u: Vec<f64> = (0..n).map(|i| f64::from((i * 53 + 7) % 17) - 8.0).collect();
    let w: Vec<f64> = (0..n).map(|i| f64::from((i * 29 + 3) % 7) - 3.0).collect();
    let x: Vec<f64> = (0..n as usize)
        .map(|i| 0.05 * z[i] + u[i] + 0.3 * w[i])
        .collect();
    let y: Vec<f64> = (0..n as usize)
        .map(|i| 1.0 + 0.5 * x[i] + w[i] + 2.0 * u[i])
        .collect();
    let weak = table(vec![("y", y), ("x", x), ("w", w), ("z1", z)]);
    let card = reference_data("card");
    let wage2 = reference_data("wage2");
    let strong = small();
    let card_model = model(
        "lwage ~ exper + expersq + black + south + smsa + educ",
        "educ",
        "nearc4 + nearc2",
    );
    let siblings = Model {
        cluster: Some("age"),
        ..model("lwage ~ exper + tenure + educ", "educ", "sibs + brthord")
    };
    let cases: Vec<(&str, &Table, Model<'_>, Kind, Option<&str>)> = vec![
        (
            "weak by construction, classical errors",
            &weak,
            model("y ~ w + x", "x", "z1"),
            Kind::Classical,
            None,
        ),
        (
            "weak by construction, robust errors",
            &weak,
            model("y ~ w + x", "x", "z1"),
            Kind::Hc1,
            None,
        ),
        (
            "Card's two indicators, classical errors",
            &card,
            card_model.clone(),
            Kind::Classical,
            None,
        ),
        (
            "Card's two indicators, robust errors",
            &card,
            card_model.clone(),
            Kind::Hc1,
            None,
        ),
        (
            "wage2 by siblings and birth order, clustered by age",
            &wage2,
            siblings.clone(),
            Kind::Cluster,
            Some("age"),
        ),
        (
            "wage2 by siblings and birth order, classical errors",
            &wage2,
            model("lwage ~ exper + tenure + educ", "educ", "sibs + brthord"),
            Kind::Classical,
            None,
        ),
        (
            "strong instruments, robust errors",
            &strong,
            model("y ~ w + x", "x", "z1 + z2"),
            Kind::Hc1,
            None,
        ),
    ];
    let mut all = String::new();
    for (what, data, asked, kind, cluster) in cases {
        let found = weak_of(data, &asked, kind, cluster);
        all.push_str(&format!("## {what}\n"));
        match found.warning() {
            Some(warning) => all.push_str(&warning.render(&Sources::new())),
            None => all.push_str("no warning\n"),
        }
        all.push('\n');
    }
    insta::assert_snapshot!("iv_weak_instruments_warning", all);
}

/// What cannot be computed is not: with as many instruments as rows no first stage has a
/// residual; with first-stage residuals that are collinear (in Card's data schooling and
/// experience sum to age less six, and age is an instrument) the covariance of the residuals
/// has no inverse and there is no Cragg–Donald F, while each first stage has its F; with fewer
/// clusters than instruments the clustered variance of their coefficients has no inverse.
#[test]
fn a_statistic_that_cannot_be_computed_is_not() {
    // Twelve rows; an intercept, `w` and ten instruments.
    let t = small();
    let mut wide = t.clone();
    let mut names: Vec<String> = Vec::new();
    for j in 0..10u32 {
        let values: Vec<f64> = (0..12u32)
            .map(|i| f64::from((i * (j + 3) * 7 + j * j) % 13) + f64::from(i == j) * 0.5)
            .collect();
        let name = format!("q{j}");
        wide = wide.with_column(&name, floats(&values)).unwrap();
        names.push(name);
    }
    let instruments = names.join(" + ");
    let found = weak_of(
        &wide,
        &model("y ~ w + x", "x", &instruments),
        Kind::Hc1,
        None,
    );
    assert_eq!(found.df1, 10);
    assert_eq!(found.df2, 0.0);
    assert_eq!(found.f, vec![None]);
    assert_eq!(found.wald, vec![None]);
    assert_eq!(found.cragg_donald, None);

    let card = reference_data("card");
    let asked = model(
        "lwage ~ black + south + smsa + educ + exper",
        "educ + exper",
        "nearc4 + nearc2 + age",
    );
    let found = weak_of(&card, &asked, Kind::Classical, None);
    assert!(
        found.f.iter().all(|f| f.is_some_and(|f| f > 1.0)),
        "{:?}",
        found.f
    );
    assert_eq!(found.cragg_donald, None);

    // Three clusters and four instruments.
    let groups: Vec<f64> = (0..12).map(|i| f64::from(i % 3)).collect();
    let t = with(&wide, "g", groups);
    let asked = Model {
        cluster: Some("g"),
        ..model("y ~ w + x", "x", "z1 + z2 + q0 + q1")
    };
    let found = weak_of(&t, &asked, Kind::Cluster, Some("g"));
    assert!(found.f[0].is_some());
    assert!(found.cragg_donald.is_some());
    assert_eq!(found.wald, vec![None]);
}

// ----- the diagnostics of the first stage -----

/// The partial R² of each first stage and Shea's, the test of the over-identifying
/// restrictions (Sargan's under classical errors, Hansen's J under robust and clustered ones)
/// and the Durbin–Wu–Hausman test, against `iv.json` (base R, held there to fixest's `sargan`
/// and `wh`): one and two instrumented regressors, a just-identified model, weights of two
/// kinds, an absorbed effect with the clusters outside it and on it, rows with `NA`.
#[test]
fn the_diagnostics_of_the_first_stage_agree_with_r() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/iv.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let cases = reference["diagnostics"].as_object().unwrap();
    assert_eq!(cases.len(), 7);
    for (name, case) in cases {
        let data = reference_data(case["data"].as_str().unwrap());
        let weights = case["weights"].as_str().map(|w| {
            let kind = match case["weight_type"].as_str().unwrap() {
                "analytic" => WeightKind::Analytic,
                "sampling" => WeightKind::Sampling,
                _ => WeightKind::Frequency,
            };
            (w, kind)
        });
        let asked = Model {
            formula: case["formula"].as_str().unwrap(),
            endogenous: case["endogenous"].as_str().unwrap(),
            instruments: case["instruments"].as_str().unwrap(),
            fe: case["fe"].as_str(),
            weights,
            ..Model::default()
        };
        let tolerance = 1e-7;
        let near = |ours: Option<f64>, theirs: &Json, what: &str| {
            let a = ours.unwrap_or_else(|| panic!("{name}: {what} is not available"));
            let b = number(theirs);
            assert!(
                close(a, b, tolerance, 0.0),
                "{name}: {what}: {a} against {b}"
            );
        };
        let classical = diagnostics_of(&data, &asked, Kind::Classical, None);
        for (j, theirs) in case["partial"].as_array().unwrap().iter().enumerate() {
            near(classical.partial[j], theirs, "partial R²");
            near(classical.shea[j], &case["shea"][j], "Shea's partial R²");
        }
        let over = case["df_over"].as_u64().unwrap() as usize;
        match classical.over_identification {
            None => assert_eq!(over, 0, "{name}: just identified"),
            Some(test) => {
                assert!(!test.hansen, "{name}: Sargan under classical errors");
                assert_eq!(test.df, over, "{name}: degrees of freedom");
                near(test.statistic, &case["sargan"], "Sargan's statistic");
            }
        }
        let dwh = classical.endogeneity.unwrap();
        near(Some(dwh.statistic), &case["dwh"], "Durbin-Wu-Hausman F");
        assert_eq!(dwh.df1 as u64, case["dwh_df1"].as_u64().unwrap(), "{name}");
        assert_eq!(dwh.df2 as f64, number(&case["dwh_df2"]), "{name}");
        let robust = diagnostics_of(&data, &asked, Kind::Hc1, None);
        assert_eq!(robust.partial, classical.partial, "{name}");
        assert_eq!(robust.shea, classical.shea, "{name}");
        near(
            robust.endogeneity.map(|e| e.statistic),
            &case["dwh_hc1"],
            "robust Durbin-Wu-Hausman",
        );
        if over > 0 {
            let test = robust.over_identification.unwrap();
            assert!(test.hansen && test.df == over, "{name}");
            near(test.statistic, &case["hansen_hc1"], "Hansen's J");
        } else {
            assert_eq!(robust.over_identification, None, "{name}");
        }
        if let Some(clustered) = case["dwh_cluster"].as_object() {
            for (cluster, theirs) in clustered {
                let with = Model {
                    cluster: Some(cluster),
                    ..asked.clone()
                };
                let found = diagnostics_of(&data, &with, Kind::Cluster, Some(cluster));
                near(
                    found.endogeneity.map(|e| e.statistic),
                    theirs,
                    &format!("Durbin-Wu-Hausman clustered by {cluster}"),
                );
                if over > 0 {
                    near(
                        found.over_identification.unwrap().statistic,
                        &case["hansen_cluster"][cluster],
                        &format!("Hansen's J clustered by {cluster}"),
                    );
                }
            }
        }
    }
}

/// What the diagnostics cannot say they do not: a just-identified model has no restriction to
/// test; first-stage residuals that are collinear (Card's schooling and experience with age
/// among the instruments) leave one coefficient to the endogeneity test, not two; fewer
/// clusters than instruments leave Hansen's J without a variance to invert; and with nothing
/// instrumented there is nothing to diagnose.
#[test]
fn a_diagnostic_that_cannot_be_computed_is_not() {
    let t = small();
    let just = diagnostics_of(&t, &model("y ~ w + x", "x", "z1"), Kind::Hc1, None);
    assert_eq!(just.over_identification, None);
    assert!(just.endogeneity.is_some() && just.partial[0].is_some());
    let none = diagnostics_of(&t, &model("y ~ w + x", "", ""), Kind::Classical, None);
    assert_eq!(none.partial, Vec::<Option<f64>>::new());
    assert_eq!((none.over_identification, none.endogeneity), (None, None));
    let card = reference_data("card");
    let asked = model(
        "lwage ~ black + south + smsa + educ + exper",
        "educ + exper",
        "nearc4 + nearc2 + age",
    );
    let found = diagnostics_of(&card, &asked, Kind::Classical, None);
    assert_eq!(found.endogeneity.unwrap().df1, 1);
    assert_eq!(found.over_identification.unwrap().df, 1);
    // Three clusters and four instruments.
    let groups: Vec<f64> = (0..12).map(|i| f64::from(i % 3)).collect();
    let z3: Vec<f64> = (0..12u32).map(|i| f64::from((i * 7 + 3) % 13)).collect();
    let z4: Vec<f64> = (0..12u32).map(|i| f64::from((i * 5 + 1) % 11)).collect();
    let t = with(&with(&with(&t, "g", groups), "z3", z3), "z4", z4);
    let asked = Model {
        cluster: Some("g"),
        ..model("y ~ w + x", "x", "z1 + z2 + z3 + z4")
    };
    let found = diagnostics_of(&t, &asked, Kind::Cluster, Some("g"));
    let test = found.over_identification.unwrap();
    assert!(test.hansen && test.df == 3);
    assert_eq!(test.statistic, None);
}

// ----- the Anderson–Rubin test and set -----

/// The Anderson–Rubin statistic and the confidence set against `ar.json` (base R: the classical
/// set by its quadratic, the robust and clustered ones by a search over regressions): an
/// interval, two rays (the weaker instrument alone), the whole line (a small sample with the
/// weaker instrument), an empty set (two instruments that disagree, at a low level), a wide
/// interval at a level of 0.999, with the region absorbed, with weights, with clusters, and
/// the joint test alone for two instrumented regressors. An end of a set is held to 1e-6 of
/// its size; the text of every set is stored.
#[test]
fn the_anderson_rubin_sets_agree_with_r() {
    let text = std::fs::read_to_string(root().join("tests/reference/r/ar.json")).unwrap();
    let reference: Json = serde_json::from_str(&text).unwrap();
    let cases = reference["cases"].as_object().unwrap();
    assert_eq!(cases.len(), 10);
    let mut all = String::new();
    for (name, case) in cases {
        let mut data = reference_data(case["data"].as_str().unwrap());
        if name == "card_first_400" {
            data = data.take_rows(&(0..400).collect::<Vec<_>>());
        }
        let weights = case["weights"].as_str().map(|w| (w, WeightKind::Analytic));
        let level = number(&case["level"]);
        for (kind_name, theirs) in case["kinds"].as_object().unwrap() {
            let (kind, cluster) = match kind_name.as_str() {
                "classical" => (Kind::Classical, None),
                "hc1" => (Kind::Hc1, None),
                other => (Kind::Cluster, Some(other)),
            };
            let asked = Model {
                formula: case["formula"].as_str().unwrap(),
                endogenous: case["endogenous"].as_str().unwrap(),
                instruments: case["instruments"].as_str().unwrap(),
                fe: case["fe"].as_str(),
                weights,
                cluster,
                ..Model::default()
            };
            let found = anderson_rubin_of(&data, &asked, kind, cluster, level);
            let what = format!("{name}, {kind_name}");
            let statistic = found
                .statistic
                .unwrap_or_else(|| panic!("{what}: no statistic"));
            let want = number(&theirs["statistic"]);
            assert!(
                close(statistic, want, 1e-7, 0.0),
                "{what}: {statistic} against {want}"
            );
            assert_eq!(found.df1 as u64, case["df1"].as_u64().unwrap(), "{what}");
            assert_eq!(found.df2 as u64, theirs["df2"].as_u64().unwrap(), "{what}");
            let Some(shape) = theirs["shape"].as_str() else {
                assert_eq!(found.set, None, "{what}: no set for several regressors");
                all.push_str(&format!("{what}: F = {statistic:.4}, no set\n"));
                continue;
            };
            let set = found.set.unwrap_or_else(|| panic!("{what}: no set"));
            let ours = match set.shape() {
                Shape::Interval => "interval",
                Shape::TwoRays => "two rays",
                Shape::WholeLine => "whole line",
                Shape::Empty => "empty",
                Shape::Other => "other",
            };
            assert_eq!(ours, shape, "{what}: {:?}", set.pieces);
            let ends: Vec<f64> = set
                .pieces
                .iter()
                .flat_map(|(lo, hi)| [*lo, *hi])
                .filter(|e| e.is_finite())
                .collect();
            let theirs_ends = numbers(&theirs["ends"]);
            assert_eq!(ends.len(), theirs_ends.len(), "{what}");
            for (a, b) in ends.iter().zip(&theirs_ends) {
                assert!(
                    close(*a, *b, 1e-6, 0.0),
                    "{what}: an end at {a} against {b}"
                );
            }
            all.push_str(&format!(
                "{what}: F = {statistic:.4}, {shape} {}\n",
                set.pieces
                    .iter()
                    .map(|(lo, hi)| format!("[{lo:.5}, {hi:.5}]"))
                    .collect::<Vec<_>>()
                    .join(" and ")
            ));
        }
    }
    insta::assert_snapshot!("anderson_rubin_sets", all);
}

/// On the shipped `staff`, where the offer is a strong instrument, the Anderson–Rubin set is an
/// interval close to the usual one, 4.60 ± 1.96 × 0.43, and holds the estimate; with nothing
/// instrumented there is no test.
#[test]
fn a_strong_instrument_gives_a_set_close_to_the_usual_interval() {
    let staff = zeff_datasets::table("staff").expect("the shipped staff");
    let asked = Model {
        fe: Some("site"),
        ..model("productivity ~ training", "training", "offer")
    };
    let found = anderson_rubin_of(&staff, &asked, Kind::Classical, None, 0.95);
    let set = found.set.unwrap();
    assert_eq!(set.shape(), Shape::Interval);
    let (lo, hi) = set.pieces[0];
    assert!(lo < 4.602_651_8 && 4.602_651_8 < hi);
    assert!((lo - (4.6027 - 1.961 * 0.4295)).abs() < 0.03, "{lo}");
    assert!((hi - (4.6027 + 1.961 * 0.4295)).abs() < 0.03, "{hi}");
    let none = anderson_rubin_of(
        &staff,
        &model("productivity ~ training", "", ""),
        Kind::Hc1,
        None,
        0.95,
    );
    assert_eq!((none.statistic, none.set), (None, None));
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

    /// The weak-instrument statistics do not depend on the units of any variable.
    #[test]
    fn the_weak_instrument_statistics_do_not_depend_on_units(values in rows()) {
        let t = frame_of(&values);
        let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&base));
        let plain = model("y ~ w + x", "x", "z1 + z2");
        let scaled = model("(y * 7) ~ (w * 1e3) + (x * 1e-4)", "(x * 1e-4)", "(z1 * 1e5) + (z2 * 3)");
        for kind in [Kind::Classical, Kind::Hc1, Kind::Hc3] {
            let a = weak_of(&t, &plain, kind, None);
            let b = weak_of(&t, &scaled, kind, None);
            prop_assert_eq!(a.df1, b.df1);
            prop_assert_eq!(a.df2, b.df2);
            for (x, y) in [(a.f[0], b.f[0]), (a.wald[0], b.wald[0]), (a.cragg_donald, b.cragg_donald), (a.kleibergen_paap, b.kleibergen_paap)] {
                let (x, y) = (x.unwrap(), y.unwrap());
                prop_assert!(close(x, y, 1e-8, 1e-12), "{:?}: {} against {}", kind, x, y);
            }
            // With one instrumented regressor the rk F is the Wald F of its first stage.
            prop_assert!(close(a.kleibergen_paap.unwrap(), a.wald[0].unwrap(), 1e-9, 1e-12));
            prop_assert_eq!(a.effective.is_some(), kind != Kind::Classical);
            if let (Some(x), Some(y)) = (a.effective, b.effective) {
                prop_assert!(close(x.f, y.f, 1e-8, 1e-12), "{:?}: {} against {}", kind, x.f, y.f);
                prop_assert!(close(x.degrees, y.degrees, 1e-8, 1e-12));
                prop_assert!(close(x.critical, y.critical, 1e-8, 1e-12));
                prop_assert!(x.degrees >= 1.0 - 1e-9 && x.degrees <= 2.0 + 1e-9, "{}", x.degrees);
            }
            prop_assert!(close(a.f[0].unwrap(), a.cragg_donald.unwrap(), 1e-10, 1e-12));
        }
    }

    /// The diagnostics of the first stage do not depend on the units of any variable.
    #[test]
    fn the_diagnostics_do_not_depend_on_units(values in rows()) {
        let t = frame_of(&values);
        let (base, _) = fit(&t, &model("y ~ w + x", "x", "z1 + z2")).unwrap();
        prop_assume!(well_identified(&base));
        let plain = model("y ~ w + x", "x", "z1 + z2");
        let scaled = model("(y * 7) ~ (w * 1e3) + (x * 1e-4)", "(x * 1e-4)", "(z1 * 1e5) + (z2 * 3)");
        for kind in [Kind::Classical, Kind::Hc1] {
            let a = diagnostics_of(&t, &plain, kind, None);
            let b = diagnostics_of(&t, &scaled, kind, None);
            let pairs = [
                (a.partial[0], b.partial[0]),
                (a.shea[0], b.shea[0]),
                (a.over_identification.unwrap().statistic, b.over_identification.unwrap().statistic),
                (a.endogeneity.map(|e| e.statistic), b.endogeneity.map(|e| e.statistic)),
            ];
            for (x, y) in pairs {
                let (x, y) = (x.unwrap(), y.unwrap());
                prop_assert!(close(x, y, 1e-6, 1e-9), "{:?}: {} against {}", kind, x, y);
            }
            // With one instrumented regressor Shea's partial R² is the partial R².
            prop_assert!(close(a.partial[0].unwrap(), a.shea[0].unwrap(), 1e-8, 1e-12));
            prop_assert_eq!(a.over_identification.unwrap().hansen, kind != Kind::Classical);
        }
    }

    /// The Anderson–Rubin set holds the two-stage estimate when the instruments are strong, is
    /// wider at a higher level, and is in the outcome's units: an outcome times seven has a
    /// set seven times as far out, and a regressor in other units a set in those.
    #[test]
    fn the_anderson_rubin_set_holds_the_estimate_and_follows_the_units(values in rows()) {
        let t = frame_of(&values);
        let plain = model("y ~ w + x", "x", "z1 + z2");
        let (base, _) = fit(&t, &plain).unwrap();
        prop_assume!(well_identified(&base));
        let scaled = model("(y * 7) ~ w + (x * 0.5)", "(x * 0.5)", "(z1 * 1e5) + z2");
        for kind in [Kind::Classical, Kind::Hc1] {
            let strength = weak_of(&t, &plain, kind, None);
            prop_assume!(strength.wald[0].is_some_and(|f| f > 30.0));
            let a = anderson_rubin_of(&t, &plain, kind, None, 0.95);
            let b = anderson_rubin_of(&t, &scaled, kind, None, 0.95);
            let wider = anderson_rubin_of(&t, &plain, kind, None, 0.99);
            prop_assert!(close(a.statistic.unwrap(), b.statistic.unwrap(), 1e-7, 1e-10));
            let (sa, sb, sw) = (a.set.unwrap(), b.set.unwrap(), wider.set.unwrap());
            prop_assert_eq!(sa.shape(), sb.shape());
            if sa.shape() == Shape::Interval && sw.shape() == Shape::Interval {
                let estimate = base.fitted.estimates[2];
                // The set need not hold the estimate when the instruments disagree; when it is
                // an interval at both levels the wider holds the narrower.
                prop_assert!(sw.pieces[0].0 <= sa.pieces[0].0 && sa.pieces[0].1 <= sw.pieces[0].1);
                if a.statistic.unwrap() > 0.0 {
                    let (lo, hi) = sa.pieces[0];
                    let j = diagnostics_of(&t, &plain, kind, None).over_identification.unwrap().statistic.unwrap();
                    if j < 1.0 {
                        prop_assert!(lo <= estimate && estimate <= hi, "{} outside [{}, {}]", estimate, lo, hi);
                    }
                }
                for ((x, y), _) in sa.pieces.iter().zip(&sb.pieces).zip(0..1) {
                    prop_assert!(close(x.0 * 14.0, y.0, 1e-6, 1e-9), "{} against {}", x.0 * 14.0, y.0);
                    prop_assert!(close(x.1 * 14.0, y.1, 1e-6, 1e-9), "{} against {}", x.1 * 14.0, y.1);
                }
            }
        }
    }

    /// An effect by `fe:` gives the estimates of its indicators written out, whatever the data.
    #[test]
    fn an_effect_by_fe_is_its_indicators(values in rows()) {
        let groups: Vec<Option<i64>> = (0..values.len()).map(|i| Some((i % 3) as i64)).collect();
        let t = frame_of(&values).with_column("g", Column::ints(groups)).unwrap();
        let written = model("y ~ w + x + category(g)", "x", "z1 + z2");
        let (indicators, _) = fit(&t, &written).unwrap();
        prop_assume!(well_identified(&indicators));
        let by_fe = Model { fe: Some("g"), ..model("y ~ w + x", "x", "z1 + z2") };
        let (absorbed, projection, _) = fit_projected(&t, &by_fe).unwrap();
        let theirs = estimates(&indicators);
        for (name, ours) in estimates(&absorbed) {
            let other = theirs.iter().find(|(n, _)| *n == name).unwrap().1;
            prop_assert!(close(ours, other, 1e-8, 1e-10), "{}: {} against {}", name, ours, other);
        }
        let ours = std_errors_projected(&t, &absorbed, projection.as_ref(), Kind::Hc1, None);
        let other = std_errors(&t, &indicators, Kind::Hc1, None);
        // `w` and `x` are the first two columns under `fe:` and follow the intercept without.
        for (i, a) in ours.iter().enumerate() {
            prop_assert!(close(*a, other[i + 1], 1e-7, 1e-10), "{} against {}", a, other[i + 1]);
        }
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
