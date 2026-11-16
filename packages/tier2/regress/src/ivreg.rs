//! `ivreg` (Language Reference, sections 7.1 and 7.2; Engine and Project Specification, section
//! 13; catalogue row 239; release 0.8.0, step 13b): two-stage least squares as a method of a
//! frame. The estimator is `iv`; this file reads the arguments as `reg` reads its own, builds
//! the designs, fits, takes the variance of the kind asked for from the variance layer, says
//! what was changed on the way, and makes the fit a script holds: the structural table in
//! `reg`'s layout, under it what was instrumented by what and the weak-instrument F of the
//! release's ruling, and above it the first stages when `first: true` asks for them.

use crate::iv::{Column, Designs, Projection, TwoStage, estimate};
use crate::ols::{Ols, internal};
use crate::reg::{
    At, CONDITION_WARNING, Choices, checked_variance, choices, cluster_and_weight_warnings,
    dropped_notes, observations, se_line, weights_line,
};
use std::rc::Rc;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{FitSource, FixedEffect, Instrumented, Table, Term, Value, terms_text};
use zeff_diagnostics::{Diagnostic, count};
use zeff_results::absorb::{EffectGroups, effect_values};
use zeff_results::estimator::Estimator;
use zeff_results::summary::{self, Setting, rows};
use zeff_results::variance::{self, Clusters, Kind};

/// The positions of `ivreg`'s arguments after the frame and the formula.
const FE: usize = 2;
const INSTRUMENTS: usize = 3;
const SE: usize = 4;
const CLUSTER: usize = 5;
const WEIGHTS: usize = 6;
const WEIGHT_TYPE: usize = 7;
const LEVEL: usize = 8;
const FIRST: usize = 9;

const IV_AT: At = At {
    se: SE,
    cluster: CLUSTER,
    weights: WEIGHTS,
    weight_type: WEIGHT_TYPE,
    level: LEVEL,
};

fn listed(names: &[String]) -> String {
    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A statistic as the lines under a table write it.
fn two_decimals(value: f64) -> String {
    format!("{value:.2}")
}

/// `data.ivreg(formula, fe:, instruments:, se:, cluster:, weights:, weight_type:, level:,
/// first:)`.
pub fn ivreg(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "ivreg was called without a frame"));
    };
    let Some(Value::Formula(formula)) = args.get(1) else {
        return Err(internal(ctx, "ivreg was called without a formula"));
    };
    let frame = frame.clone();
    let formula = formula.clone();
    let effects: Vec<Term> = match args.get(FE) {
        Some(Value::Terms(terms)) => terms.clone(),
        _ => Vec::new(),
    };
    let Some(Value::Instruments {
        endogenous,
        excluded,
        included,
    }) = args.get(INSTRUMENTS)
    else {
        return Err(fail(
            Diagnostic::error(
                "Z0406",
                "`ivreg` needs `instruments:`: what is instrumented, and by what",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no `instruments:`")
            .help("name the instrumented regressor on the left of `~` and its instruments on the right: `instruments: educ ~ nearc4`; without one, the model is `reg`"),
        ));
    };
    let (endogenous, excluded, included) = (endogenous.clone(), excluded.clone(), included.clone());
    let first_asked = matches!(args.get(FIRST), Some(Value::Bool(true)));
    let Choices {
        cluster,
        weights,
        weight_kind,
        kind,
        level,
    } = choices(ctx, &args, &IV_AT, !effects.is_empty())?;
    // The designs and the two stages.
    let extra: Vec<&str> = weights
        .iter()
        .chain(cluster.iter())
        .map(String::as_str)
        .collect();
    let weighting = weights.as_deref().map(|name| (name, weight_kind));
    // An instrument the checker moved to the included ones is given with the excluded: the
    // designs leave out any that is a regressor under its own name, and name it.
    let mut given = excluded.clone();
    given.extend(included.iter().cloned());
    let (designs, projection): (Designs, Option<Projection>) = if effects.is_empty() {
        (
            Designs::build(
                ctx,
                &frame,
                &formula,
                &endogenous,
                &given,
                &extra,
                weighting,
            )?,
            None,
        )
    } else {
        let (designs, projection) = Designs::build_absorbing(
            ctx,
            &frame,
            &formula,
            &endogenous,
            &given,
            &extra,
            weighting,
            &effects,
        )?;
        (designs, Some(projection))
    };
    let effects_text = terms_text(&effects);
    // What the designs changed on the way, said once.
    let mut said: Vec<String> = Vec::new();
    let design_notes: Vec<String> = match &projection {
        Some(p) => p.original.notes.clone(),
        None => designs.structural.notes.clone(),
    };
    for note in design_notes.iter().chain(&designs.excluded.notes) {
        if !said.contains(note) {
            said.push(note.clone());
            ctx.message(Diagnostic::plain_note(note.clone()));
        }
    }
    let mut notes: Vec<String> = designs
        .excluded
        .notes
        .iter()
        .filter(|n| !design_notes.contains(n))
        .cloned()
        .collect();
    let zero_weights = projection
        .as_ref()
        .map_or(designs.structural.zero_weights, |p| p.original.zero_weights);
    if zero_weights > 0 {
        let text = format!(
            "{} with a zero weight left out of the estimate and the observation count",
            rows(zero_weights)
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    if !designs.included.is_empty() {
        let text = format!(
            "{} already in the formula, so {} an included instrument and not an excluded one",
            match designs.included.as_slice() {
                [one] => format!("`{one}` is"),
                many => format!("{} are", listed(many)),
            },
            if designs.included.len() == 1 {
                "it is"
            } else {
                "each is"
            }
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    let fit: TwoStage = estimate(ctx, designs)?;
    let designs = &fit.designs;
    let absorbed_columns: Vec<usize> = projection
        .as_ref()
        .map(|p| p.absorbed.absorbed_columns.clone())
        .unwrap_or_default();
    dropped_notes(
        ctx,
        &mut notes,
        &designs.structural,
        &fit.fitted,
        &absorbed_columns,
        &effects_text,
    );
    for text in &fit.notes {
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text.clone()));
    }
    // An instrumented regressor the fit dropped is no longer instrumented: the model is then
    // least squares on what remains, which is worth saying.
    let dropped_instrumented: Vec<String> = designs
        .endogenous
        .iter()
        .filter(|c| fit.fitted.omitted.contains(c))
        .map(|&c| designs.structural.names[c].clone())
        .collect();
    let instrumented_columns: Vec<String> = designs
        .endogenous
        .iter()
        .filter(|c| !fit.fitted.omitted.contains(c))
        .map(|&c| designs.structural.names[c].clone())
        .collect();
    if !dropped_instrumented.is_empty() && instrumented_columns.is_empty() {
        let text = format!(
            "{} dropped, so nothing is instrumented and the estimates are those of least squares",
            match dropped_instrumented.as_slice() {
                [one] => format!("`{one}` was"),
                many => format!("{} were", listed(many)),
            }
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    let mut warnings: Vec<String> = Vec::new();
    if fit.fitted.condition_number > CONDITION_WARNING {
        let text = format!(
            "near-singular regressors: the condition number of the scaled design is {:.3e}",
            fit.fitted.condition_number
        );
        warnings.push(format!("warning[W1001]: {text}"));
        ctx.message(
            Diagnostic::warning("W1001", text)
                .help("two regressors are almost the same column once the instrumented ones are replaced by their fitted values; drop one, or find an instrument that tells them apart"),
        );
    }
    let clusters = clusters_for(
        ctx,
        &frame,
        cluster.as_deref(),
        projection.as_ref(),
        &designs.structural.rows,
    )?;
    let absorbed = projection
        .as_ref()
        .map_or(0, |p| p.parameters(clusters.as_ref()));
    let (vcov, df) = checked_variance(
        ctx,
        fit.variance(kind, clusters.as_ref(), absorbed),
        kind,
        cluster.as_deref(),
        CLUSTER,
    )?;
    cluster_and_weight_warnings(
        ctx,
        &mut notes,
        &mut warnings,
        clusters.as_ref(),
        &designs.structural,
    );
    // How strong the instruments are: the statistic of the ruling, and the warning.
    let weak = fit.weak(ctx, kind, clusters.as_ref(), projection.as_ref())?;
    // With nothing left to instrument there is no instrument to call weak.
    if !instrumented_columns.is_empty()
        && let Some(warning) = weak.warning()
    {
        warnings.push(format!("warning[W1007]: {}", warning.message));
        ctx.message(warning);
    }
    let (strength_name, strength) = if weak.robust {
        ("Kleibergen–Paap F", weak.kleibergen_paap)
    } else {
        ("Cragg–Donald F", weak.cragg_donald)
    };
    // The summary.
    let line = se_line(kind, clusters.as_ref(), cluster.as_deref());
    let singletons = projection.as_ref().map(|p| p.absorbed.singletons);
    let (n_obs, n_note) = match &projection {
        // The rows dropped for `NA` are counted on the design before the projection.
        Some(p) => {
            let (_, note) = observations(&p.original, singletons);
            (observations(&designs.structural, None).0, note)
        }
        None => observations(&designs.structural, None),
    };
    let more_lines = weights_line(weights.as_deref(), &designs.structural);
    let formula_text = formula.to_string();
    let fixed_effects: Vec<FixedEffect> = match &projection {
        Some(p) => {
            let values = effect_values(&p.original, &designs.structural, &fit.fitted, &p.absorbed);
            p.absorbed
                .effects
                .iter()
                .zip(values)
                .map(|(e, values)| FixedEffect {
                    name: e.name.clone(),
                    variables: e.variables.clone(),
                    keys: e.keys.clone(),
                    values,
                })
                .collect()
        }
        None => Vec::new(),
    };
    let levels = projection.as_ref().map_or_else(
        || designs.structural.category_levels(),
        |p| p.original.category_levels(),
    );
    let setting = Setting {
        method: "ivreg",
        formula: &formula_text,
        se_line: &line,
        df,
        level,
        n: n_obs,
        n_note,
        more_lines,
        model: formula.clone(),
        levels,
        frame_rows: frame.table.rows(),
        effects: fixed_effects.clone(),
        absorbed_parameters: projection.as_ref().map_or(0, |p| p.parameters(None)),
        singletons: singletons.unwrap_or(0),
    };
    let mut value = summary::linear_fit(&fit.second, &fit.fitted, &vcov, &setting);
    value.kind = "IvFit";
    value.source = Some(FitSource {
        frame: frame.clone(),
        weights: weights.clone(),
        cluster: cluster.clone(),
    });
    // The spread of each regressor as observed, not of its fitted values.
    value.term_sd = summary::deviations(&designs.structural, &fit.fitted).0;
    value.warnings = warnings;
    value.notes = said;
    value.notes.extend(notes);
    // The residuals and fitted values of the rows, unweighted; with fixed effects the fitted
    // values include the effects' part.
    if let Some(w) = &designs.structural.weights {
        for (i, root) in w.iter().map(|w| w.sqrt()).enumerate() {
            value.residuals[i] /= root;
            value.fitted[i] /= root;
        }
    }
    if let Some(original) = &designs.structural.original_outcome {
        for (i, y) in original.iter().enumerate() {
            value.fitted[i] = y - value.residuals[i];
        }
    }
    // Under the table: what was instrumented by what, and how strong the instruments are. A
    // fit by instrumental variables prints no R²: with a regressor replaced by its fitted
    // values the residuals are not those of a projection, and the figure measures no fit.
    let instruments_kept: Vec<String> = fit.first.as_ref().map_or_else(Vec::new, |first| {
        first
            .kept
            .iter()
            .filter(|&&i| matches!(designs.instruments[i], Column::Excluded(_)))
            .map(|&i| designs.name(designs.instruments[i]).to_string())
            .collect()
    });
    let mut after = vec![format!(
        "instrumented: {} · instruments: {}",
        if instrumented_columns.is_empty() {
            "nothing".to_string()
        } else {
            instrumented_columns.join(", ")
        },
        if instruments_kept.is_empty() {
            "none".to_string()
        } else {
            instruments_kept.join(", ")
        }
    )];
    let mut strength_line = format!(
        "{strength_name} {}",
        strength.map_or("not available".to_string(), two_decimals)
    );
    if let Some(e) = weak.effective {
        strength_line.push_str(&format!(
            " · effective F {} (critical value {})",
            two_decimals(e.f),
            two_decimals(e.critical)
        ));
    }
    if !instrumented_columns.is_empty() {
        after.push(strength_line);
    }
    value.summary.after = after;
    // The first stages, when asked for: each in `reg`'s layout, under the fit's standard
    // errors, with the F of its excluded instruments.
    let first_tables: Vec<Table> = if first_asked && !instrumented_columns.is_empty() {
        let shared = Shared {
            kind,
            clusters: clusters.as_ref(),
            cluster: cluster.as_deref(),
            absorbed,
            line: &line,
            level,
            n_obs,
            model: &formula,
            frame_rows: frame.table.rows(),
            effects: &fixed_effects,
            absorbed_parameters: setting.absorbed_parameters,
            singletons: setting.singletons,
        };
        stage_tables(ctx, &fit, &weak, &shared, None)?
    } else {
        Vec::new()
    };
    value.instrumented = Some(Rc::new(Instrumented {
        endogenous,
        excluded,
        included,
        effects,
        instrumented_columns,
        instruments_kept,
        strength: (strength_name.to_string(), strength),
        effective: weak.effective.map(|e| (e.f, e.critical)),
        first: first_tables,
    }));
    Ok(Value::Fit(Rc::new(value)))
}

/// The two stages of a fit, computed again from what the fit keeps (open decision #71, as
/// `design` does for `reg`): the same frame, formula, instruments, weights, clusters and fixed
/// effects, through the steps `ivreg` took, so that the rows and the columns are the fit's.
/// What the steps say on the way was said when the fit was made and is not said again. `None`
/// for a fit that keeps no source or is not one of `ivreg`.
pub(crate) fn refit(
    ctx: &mut CallContext<'_>,
    fit: &zeff_core::value::Fit,
) -> Option<Result<(TwoStage, Option<Projection>), Fail>> {
    let source = fit.source.as_ref()?;
    let instrumented = fit.instrumented.as_ref()?;
    let extra: Vec<&str> = source
        .weights
        .iter()
        .chain(source.cluster.iter())
        .map(String::as_str)
        .collect();
    let weight_kind = fit
        .weight_kind
        .unwrap_or(zeff_numeric::order::WeightKind::Sampling);
    let weighting = source.weights.as_deref().map(|name| (name, weight_kind));
    let mut given = instrumented.excluded.clone();
    given.extend(instrumented.included.iter().cloned());
    let said = std::mem::take(ctx.messages);
    let built = if instrumented.effects.is_empty() {
        Designs::build(
            ctx,
            &source.frame,
            &fit.model,
            &instrumented.endogenous,
            &given,
            &extra,
            weighting,
        )
        .map(|designs| (designs, None))
    } else {
        Designs::build_absorbing(
            ctx,
            &source.frame,
            &fit.model,
            &instrumented.endogenous,
            &given,
            &extra,
            weighting,
            &instrumented.effects,
        )
        .map(|(designs, projection)| (designs, Some(projection)))
    };
    let fitted =
        built.and_then(|(designs, projection)| estimate(ctx, designs).map(|two| (two, projection)));
    *ctx.messages = said;
    Some(fitted)
}

/// What the tables of the first stages share with the fit they belong to.
struct Shared<'a> {
    kind: Kind,
    clusters: Option<&'a Clusters>,
    cluster: Option<&'a str>,
    /// The parameters of the absorbed effects in every N − k, under `kind`.
    absorbed: usize,
    /// The fit's line for its standard errors.
    line: &'a str,
    level: f64,
    n_obs: usize,
    model: &'a zeff_core::value::Formula,
    frame_rows: usize,
    effects: &'a [FixedEffect],
    absorbed_parameters: usize,
    singletons: usize,
}

/// The clusters of the rows used, by the cluster variable: the index of the effect on the same
/// variable when there is one, else one built the same way (section 8.3, point 10, "Reused
/// fixed-effect work").
fn clusters_for(
    ctx: &CallContext<'_>,
    frame: &zeff_core::value::Frame,
    cluster: Option<&str>,
    projection: Option<&Projection>,
    rows: &[usize],
) -> Result<Option<Clusters>, Fail> {
    let Some(name) = cluster else {
        return Ok(None);
    };
    let reused = projection.and_then(|p| {
        p.absorbed
            .effects
            .iter()
            .find(|e| e.variables.len() == 1 && e.variables[0] == name)
            .map(|e| Clusters::from_groups(&e.group_of_row, e.levels()))
    });
    Ok(Some(match reused {
        Some(clusters) => clusters,
        None => {
            let column = frame
                .table
                .column(name)
                .ok_or_else(|| internal(ctx, "the cluster variable vanished"))?;
            let name = name.to_string();
            let groups = EffectGroups::over(&name, std::slice::from_ref(&name), &[column], rows);
            Clusters::from_groups(&groups.group_of_row, groups.levels())
        }
    }))
}

/// How a line says which standard errors a statistic stands on, when they are not classical.
fn on_errors(kind: Kind, cluster: Option<&str>) -> String {
    match kind {
        Kind::Classical => String::new(),
        Kind::Hc1 => " (robust)".to_string(),
        Kind::Hc2 => " (robust, hc2)".to_string(),
        Kind::Hc3 => " (robust, hc3)".to_string(),
        Kind::Cluster => format!(" (clustered by {})", cluster.unwrap_or("")),
    }
}

/// The table of each first stage, in `reg`'s layout under the fit's standard errors: the
/// instrumented regressor on the instruments kept, and under it the F of its excluded
/// instruments, which is the Wald form when the errors are not classical and says so. With
/// `diagnostics`, the partial R² of the stage stands before the F, and Shea's beside it when
/// several regressors are instrumented.
fn stage_tables(
    ctx: &mut CallContext<'_>,
    fit: &TwoStage,
    weak: &crate::iv::Weak,
    shared: &Shared<'_>,
    diagnostics: Option<&crate::iv::Diagnostics>,
) -> Result<Vec<Table>, Fail> {
    let designs = &fit.designs;
    let stages = fit.first_stages(ctx)?;
    let mut tables = Vec::with_capacity(stages.len());
    for (q, stage) in stages.iter().enumerate() {
        // A regressor the fit dropped is not instrumented and has no first stage to show.
        if fit.fitted.omitted.contains(&designs.endogenous[q]) {
            continue;
        }
        let bread = Ols.bread(&stage.fitted);
        let (stage_vcov, stage_df) = checked_variance(
            ctx,
            variance::variance(
                shared.kind,
                &Ols,
                &stage.design,
                &stage.fitted,
                &bread,
                shared.clusters,
                shared.absorbed,
            ),
            shared.kind,
            shared.cluster,
            CLUSTER,
        )?;
        let outcome = designs.structural.names[designs.endogenous[q]].clone();
        let regressors: Vec<&str> = stage
            .design
            .names
            .iter()
            .map(String::as_str)
            .filter(|n| *n != "(intercept)")
            .collect();
        let stage_formula = format!("{outcome} ~ {}", regressors.join(" + "));
        let stage_setting = Setting {
            method: "first stage",
            formula: &stage_formula,
            se_line: shared.line,
            df: stage_df,
            level: shared.level,
            n: shared.n_obs,
            n_note: None,
            more_lines: Vec::new(),
            model: shared.model.clone(),
            levels: Vec::new(),
            frame_rows: shared.frame_rows,
            effects: shared.effects.to_vec(),
            absorbed_parameters: shared.absorbed_parameters,
            singletons: shared.singletons,
        };
        let mut table =
            summary::linear_fit(&stage.design, &stage.fitted, &stage_vcov, &stage_setting).summary;
        table.before = Vec::new();
        let f = if weak.robust { weak.wald[q] } else { weak.f[q] };
        let mut after = Vec::new();
        if let Some(d) = diagnostics {
            let measure = |x: Option<f64>| {
                x.map_or("not available".to_string(), |v| {
                    zeff_core::display::fixed(v, 4)
                })
            };
            let mut line = format!("partial R² {}", measure(d.partial[q]));
            if designs.endogenous.len() > 1 {
                line.push_str(&format!(" · Shea's partial R² {}", measure(d.shea[q])));
            }
            after.push(line);
        }
        after.push(format!(
            "F of the excluded instruments {} on {} and {} degrees of freedom{}",
            f.map_or("not available".to_string(), two_decimals),
            count(weak.df1 as u64),
            count(if shared.kind == Kind::Cluster {
                stage_df as u64
            } else {
                weak.df2.round() as u64
            }),
            on_errors(shared.kind, shared.cluster)
        ));
        table.after = after;
        tables.push(table);
    }
    Ok(tables)
}

/// `p = 0.103`, or `p < 0.001`.
fn p_text(p: f64) -> String {
    let text = zeff_core::display::p_value_text(p);
    match text.strip_prefix('<') {
        Some(bound) => format!("p < {bound}"),
        None => format!("p = {text}"),
    }
}

/// `iv.first_stage()` (catalogue row 251; the owner's answer to question 3(b) of release
/// 0.8.0): the first stage of each instrumented regressor as a table, with its partial R²
/// (and Shea's when several are instrumented) and the F of its excluded instruments; then the
/// weak-instrument statistic of the fit, the test of the over-identifying restrictions (or
/// that there is none to test), and the Durbin–Wu–Hausman test. Everything stands on the
/// standard errors the fit was given.
pub fn first_stage(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Fit(fit)) = args.get(0) else {
        return Err(internal(ctx, "first_stage was called without a fit"));
    };
    let fit = Rc::clone(fit);
    let unbuilt = |ctx: &CallContext<'_>, why: &str| {
        internal(ctx, &format!("the first stage of the fit: {why}"))
    };
    let Some(again) = refit(ctx, &fit) else {
        return Err(unbuilt(ctx, "the fit keeps no frame or no instruments"));
    };
    let (two, projection) = again?;
    let Some(source) = &fit.source else {
        return Err(unbuilt(ctx, "the fit keeps no frame"));
    };
    if two.designs.structural.rows != fit.rows {
        return Err(unbuilt(ctx, "its rows are not the fit's"));
    }
    let kind = if fit.se_kind.starts_with("clustered") {
        Kind::Cluster
    } else if fit.se_kind.starts_with("robust") {
        Kind::Hc1
    } else if fit.se_kind == "hc2" {
        Kind::Hc2
    } else if fit.se_kind == "hc3" {
        Kind::Hc3
    } else {
        Kind::Classical
    };
    let cluster = if kind == Kind::Cluster {
        source.cluster.as_deref()
    } else {
        None
    };
    let clusters = clusters_for(
        ctx,
        &source.frame,
        cluster,
        projection.as_ref(),
        &two.designs.structural.rows,
    )?;
    let absorbed = projection
        .as_ref()
        .map_or(0, |p| p.parameters(clusters.as_ref()));
    let weak = two.weak(ctx, kind, clusters.as_ref(), projection.as_ref())?;
    let diagnostics = two.diagnostics(ctx, kind, clusters.as_ref(), projection.as_ref())?;
    let shared = Shared {
        kind,
        clusters: clusters.as_ref(),
        cluster,
        absorbed,
        line: &fit.se_kind,
        level: fit.level,
        n_obs: fit.n,
        model: &fit.model,
        frame_rows: fit.frame_rows,
        effects: &fit.effects,
        absorbed_parameters: fit.absorbed_parameters,
        singletons: fit.singletons,
    };
    let mut tables = if two.first.is_some() {
        stage_tables(ctx, &two, &weak, &shared, Some(&diagnostics))?
    } else {
        Vec::new()
    };
    // The lines for the model as a whole.
    let on = on_errors(kind, cluster);
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("standard errors: {}", fit.se_kind));
    let (name, strength) = if weak.robust {
        ("Kleibergen–Paap F", weak.kleibergen_paap)
    } else {
        ("Cragg–Donald F", weak.cragg_donald)
    };
    let mut strength_line = format!(
        "weak instruments: {name} {}",
        strength.map_or("not available".to_string(), two_decimals)
    );
    if let Some(e) = weak.effective {
        strength_line.push_str(&format!(
            " · effective F {} (critical value {})",
            two_decimals(e.f),
            two_decimals(e.critical)
        ));
    }
    lines.push(strength_line);
    lines.push(match diagnostics.over_identification {
        None => "over-identification: exactly identified, no over-identification test".to_string(),
        Some(test) => {
            let name = if test.hansen {
                "Hansen's J"
            } else {
                "Sargan's statistic"
            };
            match test.statistic {
                Some(value) => {
                    let p =
                        zeff_numeric::special::incomplete_gamma(test.df as f64 / 2.0, value / 2.0)
                            .upper;
                    format!(
                        "over-identification: {name}, chi2({}) = {}   {}{}",
                        count(test.df as u64),
                        two_decimals(value),
                        p_text(p),
                        if test.hansen { on.as_str() } else { "" }
                    )
                }
                None => format!("over-identification: {name} not available"),
            }
        }
    });
    lines.push(match diagnostics.endogeneity {
        Some(test) => {
            // The upper tail of F(df1, df2) at the statistic.
            let (d1, d2) = (test.df1 as f64, test.df2 as f64);
            let p = zeff_numeric::special::incomplete_beta(
                d2 / 2.0,
                d1 / 2.0,
                d2 / (d2 + d1 * test.statistic),
            )
            .lower;
            format!(
                "endogeneity: Durbin–Wu–Hausman F({}, {}) = {}   {}{on}",
                count(test.df1 as u64),
                count(test.df2 as u64),
                two_decimals(test.statistic),
                p_text(p)
            )
        }
        None => "endogeneity: Durbin–Wu–Hausman test not available".to_string(),
    });
    let whole = Table {
        title: String::new(),
        before: lines,
        columns: Vec::new(),
        rows: Vec::new(),
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    };
    if tables.is_empty() {
        return Ok(Value::Table(Table {
            title: format!("first stage · {}", fit.formula),
            before: vec![
                "nothing is instrumented: the fit dropped every instrumented regressor".to_string(),
            ],
            ..whole
        }));
    }
    let mut main = tables.remove(0);
    main.more = tables;
    main.more.push(whole);
    Ok(Value::Table(main))
}
