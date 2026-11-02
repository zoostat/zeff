//! `reg` (Language Reference, sections 6, 7.1, 7.2 and 10.2; question 3 (a) of release 0.4.0;
//! release 0.6.0, step 18 for `fe:`): the design of the formula on the frame, weighted when
//! `weights:` is given, the fixed effects of `fe:` projected out of it by the kernel of step 17,
//! the least-squares fit, the variance of the kind `se:` and `cluster:` ask for with the degrees
//! of freedom the absorbed effects take, the notes of what was changed, the statistical warnings
//! of section 11.1, and the fit value with its printed summary. A fitted design is kept for the
//! run, so that a second `reg` with another `se:` on the same frame and formula reuses the
//! factorisation, and the group indexes and demeaned columns of a set of fixed effects are kept
//! under the effects and the rows used, so that a robustness table demeans each column once
//! (Engine and Project Specification, section 8.3, point 10, "Reused fixed-effect work").

use crate::ols::Ols;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use zeff_columns::Column;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{ColumnExpr, FixedEffect, Formula, Term, TermKind, Value, terms_text};
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::demean::Options;
use zeff_kernels::{Matrix, sum};
use zeff_numeric::order::WeightKind;
use zeff_results::absorb::{Absorbed, EffectGroups, effect_values};
use zeff_results::design::Design;
use zeff_results::estimator::{Estimator, Fitted};
use zeff_results::summary::{self, Setting, n_note, rows};
use zeff_results::variance::{self, Clusters, Kind, VarianceError, effective_sample_size};

/// The condition number above which the design is warned about (W1001, section 11.1).
pub const CONDITION_WARNING: f64 = 1e8;
/// Fewer clusters than this is warned about (W1002).
pub const FEW_CLUSTERS: usize = 30;
/// Weights whose effective sample size is below this share of the rows are warned about (W1003).
pub const EXTREME_WEIGHTS_SHARE: f64 = 0.25;
/// An observation whose removal moves an estimate by more than this many standard errors drives
/// it (W1006): a DFBETAS beyond 1.
pub const DRIVING_DFBETAS: f64 = 1.0;

/// A design and its fit, kept for the run under the frame, the formula and the weighting that
/// made them, so that another `se:` or `cluster:` reuses the factorisation. The frame's table is
/// held, so that its address cannot be reused by another frame while the entry lives.
struct Kept {
    table: Rc<zeff_columns::Table>,
    key: String,
    design: Rc<Design>,
    fitted: Rc<Fitted>,
    /// The fixed-effect work of the fit, when it has `fe:`.
    fe: Option<Rc<FixedEffectWork>>,
}

/// What a fit with `fe:` keeps beside its design: what the projection found, the effects'
/// estimates, and the category levels of the design before the projection (the transformed
/// design holds values alone).
struct FixedEffectWork {
    absorbed: Absorbed,
    effects: Vec<FixedEffect>,
    levels: Vec<(String, Vec<String>)>,
}

/// The group indexes and demeaned columns of one set of fixed effects over one set of rows
/// (section 8.3, point 10, "Reused fixed-effect work"): a later fit with the same frame, effects,
/// rows and weights reuses the indexes and every column it shares, demeaning only the rest.
struct GroupsKept {
    table: Rc<zeff_columns::Table>,
    effects_text: String,
    rows: Vec<usize>,
    weights: Option<Vec<f64>>,
    groups: Rc<Vec<EffectGroups>>,
    demeaned: BTreeMap<String, Rc<Vec<f64>>>,
}

thread_local! {
    static KEPT: RefCell<Option<Kept>> = const { RefCell::new(None) };
    static GROUPS: RefCell<Option<GroupsKept>> = const { RefCell::new(None) };
    static WORK: RefCell<(usize, usize)> = const { RefCell::new((0, 0)) };
}

/// How many sets of group indexes this thread built and how many it reused, for `--verbose`.
pub fn fixed_effect_work() -> (usize, usize) {
    WORK.with(|w| *w.borrow())
}

/// The counts of [`fixed_effect_work`] set back to zero, at the start of a run.
pub fn reset_fixed_effect_work() {
    WORK.with(|w| *w.borrow_mut() = (0, 0));
}

/// A fixed effect as written: its name and the variables it groups by, from a `Terms` value
/// (a variable, or an interaction of variables; the checker admits nothing else).
fn effect_specs(terms: &[Term]) -> Vec<(String, Vec<String>)> {
    terms
        .iter()
        .map(|term| match &term.kind {
            TermKind::Interaction(parts) => (
                term.text.clone(),
                parts
                    .iter()
                    .map(|p| match &p.kind {
                        TermKind::Var(name) => name.clone(),
                        _ => p.text.clone(),
                    })
                    .collect(),
            ),
            TermKind::Var(name) => (term.text.clone(), vec![name.clone()]),
            _ => (term.text.clone(), vec![term.text.clone()]),
        })
        .collect()
}

/// Every variable a formula names: the outcome, each term and the variables inside its
/// arithmetic. The variables of an `event` term are left out: its time variable is read for the
/// relative periods alone and is usually a fixed effect too (`event(year, treat)` with
/// `fe: unit + year`, as fixest's `i(rel) | unit + year`).
fn formula_variables(formula: &Formula) -> Vec<String> {
    fn of_expr(expr: &ColumnExpr, out: &mut Vec<String>) {
        match expr {
            ColumnExpr::Var(name) => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            ColumnExpr::List(items) => items.iter().for_each(|i| of_expr(i, out)),
            ColumnExpr::Unary { operand, .. } => of_expr(operand, out),
            ColumnExpr::Binary { lhs, rhs, .. } => {
                of_expr(lhs, out);
                of_expr(rhs, out);
            }
            ColumnExpr::Call { package, name, .. } if package == "regress" && name == "event" => {}
            ColumnExpr::Call { args, rest, .. } => {
                args.iter().flatten().for_each(|a| of_expr(a, out));
                rest.iter().for_each(|a| of_expr(a, out));
            }
            _ => {}
        }
    }
    fn of_term(term: &Term, out: &mut Vec<String>) {
        match &term.kind {
            TermKind::Var(name) => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            TermKind::Range(a, b) => {
                for name in [a, b] {
                    if !out.contains(name) {
                        out.push(name.clone());
                    }
                }
            }
            TermKind::Interaction(parts) | TermKind::Product(parts) => {
                parts.iter().for_each(|p| of_term(p, out));
            }
            TermKind::Expr(expr) => of_expr(expr, out),
        }
    }
    let mut out = Vec::new();
    if let Some(outcome) = &formula.outcome {
        of_term(outcome, &mut out);
    }
    formula.terms.iter().for_each(|t| of_term(t, &mut out));
    out
}

/// The parameters the absorbed effects stand for (the ruling of release 0.6.0 on degrees of
/// freedom, which reproduces fixest's `fixef.K = "nested"` and reghdfe): the levels of every
/// effect, less one for each effect after the first; an effect nested within the cluster
/// variable (every level inside one cluster) is left out, and when every effect is nested one
/// parameter remains for the grand mean.
fn absorbed_parameters(groups: &[EffectGroups], clusters: Option<&Clusters>) -> usize {
    let nested = |effect: &EffectGroups| -> bool {
        let Some(clusters) = clusters else {
            return false;
        };
        let mut cluster_of_level: Vec<Option<usize>> = vec![None; effect.levels()];
        for (row, &g) in effect.group_of_row.iter().enumerate() {
            match cluster_of_level[g] {
                None => cluster_of_level[g] = Some(clusters.of_row[row]),
                Some(c) if c != clusters.of_row[row] => return false,
                Some(_) => {}
            }
        }
        true
    };
    let counted: Vec<&EffectGroups> = groups.iter().filter(|e| !nested(e)).collect();
    let levels: usize = counted.iter().map(|e| e.levels()).sum();
    levels + 1 - counted.len()
}

/// `data.reg(formula, se:, cluster:, weights:, weight_type:, level:)`.
pub fn reg(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "reg was called without a frame"));
    };
    let Some(Value::Formula(formula)) = args.get(1) else {
        return Err(internal(ctx, "reg was called without a formula"));
    };
    let frame = frame.clone();
    let formula = formula.clone();
    let effects: Vec<Term> = match args.get(FE) {
        Some(Value::Terms(terms)) => terms.clone(),
        _ => Vec::new(),
    };
    let cluster = match args.get(CLUSTER) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weights = match args.get(WEIGHTS) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weight_kind = match args.get(WEIGHT_TYPE) {
        Some(Value::Word(word)) => match word.as_str() {
            "frequency" => WeightKind::Frequency,
            "analytic" => WeightKind::Analytic,
            _ => WeightKind::Sampling,
        },
        _ => WeightKind::Sampling,
    };
    let sampling = weights.is_some() && weight_kind == WeightKind::Sampling;
    let se_word = match args.get(SE) {
        Some(Value::Word(word)) => Some(word.clone()),
        _ => None,
    };
    // A fixed effect that is a model variable too is refused: an absorbed effect cannot also be
    // a regressor or the outcome.
    let specs = effect_specs(&effects);
    let model_variables = formula_variables(&formula);
    if let Some(name) = specs
        .iter()
        .flat_map(|(_, vars)| vars.iter())
        .find(|v| model_variables.contains(v))
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{name}` is a fixed effect of `fe:` and a variable of the formula too"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(FE), "absorbed and in the model")
            .help("an absorbed effect cannot be a regressor or the outcome as well; drop it from `fe:` or from the formula"),
        ));
    }
    if let (Some(word @ ("hc2" | "hc3")), false) = (se_word.as_deref(), effects.is_empty()) {
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                format!("`se: {word}` and `fe:` exclude each other: with absorbed effects an observation's leverage in the full design is not available"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(SE), "needs every observation's leverage")
            .help("`se: robust` (HC1) or `cluster:` with `fe:`; HC2 and HC3 with fixed effects arrive with release 0.8.0"),
        ));
    }
    let kind = match (se_word.as_deref(), cluster.is_some()) {
        (Some(word @ ("hc2" | "hc3")), true) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0410",
                    format!("`se: {word}` and `cluster:` exclude each other"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(SE), "two kinds of standard error")
                .help("clustered errors are CR1; leave `se:` out with `cluster:`"),
            ));
        }
        (_, true) => Kind::Cluster,
        (None, false) => {
            if sampling {
                Kind::Hc1
            } else {
                Kind::Classical
            }
        }
        (Some("classical"), false) => {
            if sampling {
                return Err(fail(
                    Diagnostic::error(
                        "Z0410",
                        "`se: classical` with sampling weights: classical errors treat the weights as precision weights",
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(SE), "not valid for probability weights")
                    .help("leave `se:` out, which gives `robust`, or write `weight_type: analytic` for precision weights"),
                ));
            }
            Kind::Classical
        }
        (Some("robust"), false) => Kind::Hc1,
        (Some("hc2"), false) => Kind::Hc2,
        (Some("hc3"), false) => Kind::Hc3,
        (Some(other), false) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0304",
                    format!("`se: {other}` is not available in this release"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(SE), "arrives with release 1.1")
                .help("`classical`, `robust`, `hc2`, `hc3` and `cluster:` are available now"),
            ));
        }
    };
    let level = match args.get(LEVEL) {
        Some(Value::Float(level)) => *level,
        Some(Value::Int(level)) => *level as f64,
        _ => 0.95,
    };
    if !(level > 0.0 && level < 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`level:` is a probability strictly between 0 and 1, but this is {level}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(LEVEL), "not a confidence level")
            .help("the usual levels are 0.9, 0.95 and 0.99"),
        ));
    }
    // The design and the fit, reused when this run already made them for this frame, formula
    // and weighting (the cluster's `NA` rows change the rows used, so it is part of the key).
    let effects_text = terms_text(&effects);
    let key = format!(
        "{formula}|{effects_text}|{}|{:?}|{}",
        weights.as_deref().unwrap_or(""),
        weights.as_ref().map(|_| weight_kind),
        cluster.as_deref().unwrap_or("")
    );
    let reused = KEPT.with(|kept| {
        kept.borrow().as_ref().and_then(|k| {
            (Rc::ptr_eq(&k.table, &frame.table) && k.key == key).then(|| {
                (
                    Rc::clone(&k.design),
                    Rc::clone(&k.fitted),
                    k.fe.as_ref().map(Rc::clone),
                )
            })
        })
    });
    let (design, fitted, fe) = match reused {
        Some(found) => found,
        None => {
            let extra: Vec<&str> = weights
                .iter()
                .chain(cluster.iter())
                .map(String::as_str)
                .collect();
            let (design, fitted, fe) = if effects.is_empty() {
                let mut design = Design::build(ctx, &frame, &formula, &extra)?;
                if let Some(name) = &weights {
                    design = design.weighted(ctx, &frame, name, weight_kind)?;
                }
                let fitted = Ols.fit(ctx, &design)?;
                (design, fitted, None)
            } else {
                let (design, fitted, work) = fit_with_effects(
                    ctx,
                    &frame,
                    &formula,
                    &extra,
                    &specs,
                    &effects_text,
                    weights.as_deref(),
                    weight_kind,
                )?;
                (design, fitted, Some(Rc::new(work)))
            };
            let (design, fitted) = (Rc::new(design), Rc::new(fitted));
            KEPT.with(|kept| {
                *kept.borrow_mut() = Some(Kept {
                    table: Rc::clone(&frame.table),
                    key,
                    design: Rc::clone(&design),
                    fitted: Rc::clone(&fitted),
                    fe: fe.as_ref().map(Rc::clone),
                });
            });
            (design, fitted, fe)
        }
    };
    for note in &design.notes {
        ctx.message(Diagnostic::plain_note(note.clone()));
    }
    // The notes printed under the fit are kept on it for the footer of a `report` table
    // (section 7.2); the design's own notes are there already.
    let mut notes: Vec<String> = Vec::new();
    if design.zero_weights > 0 {
        let text = format!(
            "{} with a zero weight left out of the estimate and the observation count",
            rows(design.zero_weights)
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    // The columns dropped, each kind with its own note: a category level with no observations
    // among the rows used, a column that is zero in every row, a column whose values are too
    // small to be scaled, and the rest collinear with the columns before them.
    let names = |columns: &[usize]| -> Vec<String> {
        columns
            .iter()
            .map(|&c| format!("`{}`", design.names[c]))
            .collect()
    };
    let note = |ctx: &mut CallContext<'_>,
                notes: &mut Vec<String>,
                listed: Vec<String>,
                one: &str,
                several: &str| {
        let text = match listed.len() {
            0 => return,
            1 => format!("{} {one}", listed[0]),
            _ => format!("{} {several}", listed.join(", ")),
        };
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    };
    // A column the fixed effects absorbed (section 7.2, "Collinearity") is zeroed by the
    // projection and dropped by the fit; it is named so, and not as a zero column.
    let absorbed_columns: Vec<usize> = fe
        .as_ref()
        .map(|w| w.absorbed.absorbed_columns.clone())
        .unwrap_or_default();
    note(
        ctx,
        &mut notes,
        names(&absorbed_columns),
        &format!("is absorbed by fe: {effects_text} and was dropped"),
        &format!("are absorbed by fe: {effects_text} and were dropped"),
    );
    let (levels, zeros): (Vec<usize>, Vec<usize>) = fitted
        .zero
        .iter()
        .filter(|c| !absorbed_columns.contains(c))
        .partition(|&&c| design.names[c].contains('='));
    note(
        ctx,
        &mut notes,
        names(&levels),
        "has no observations among the rows used and was dropped",
        "have no observations among the rows used and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&zeros),
        "is zero in every row used and was dropped",
        "are zero in every row used and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&fitted.vanishing),
        "has values too small to be scaled (below 1e-150 in size) and was dropped",
        "have values too small to be scaled (below 1e-150 in size) and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&fitted.oversized),
        "has values too large to be scaled (above 1e150 in size) and was dropped",
        "have values too large to be scaled (above 1e150 in size) and were dropped",
    );
    let collinear: Vec<usize> = fitted
        .omitted
        .iter()
        .copied()
        .filter(|c| {
            !fitted.zero.contains(c)
                && !fitted.vanishing.contains(c)
                && !fitted.oversized.contains(c)
        })
        .collect();
    note(
        ctx,
        &mut notes,
        names(&collinear),
        "is collinear with the columns before it and was dropped",
        "are collinear with the columns before them and were dropped",
    );
    // The statistical warnings, printed here and kept on the fit for the footer of a `report`
    // table (section 11.1).
    let mut warnings: Vec<String> = Vec::new();
    if fitted.condition_number > CONDITION_WARNING {
        let text = format!(
            "near-singular regressors: the condition number of the scaled design is {:.3e}",
            fitted.condition_number
        );
        warnings.push(format!("warning[W1001]: {text}"));
        ctx.message(
            Diagnostic::warning("W1001", text)
                .help("two regressors are almost the same column; drop one, or combine them"),
        );
    }
    // W1004: a regressor that keeps less than one per cent of its norm after the fixed effects
    // are projected out, so that its estimate rests on little variation.
    if let Some(work) = &fe
        && !work.absorbed.little_variation.is_empty()
    {
        let listed = work
            .absorbed
            .little_variation
            .iter()
            .filter(|(c, _)| fitted.kept.contains(c))
            .map(|(c, share)| {
                format!(
                    "`{}` keeps {}% of its variation",
                    design.names[*c],
                    zeff_core::display::fixed(share * 100.0, 2)
                )
            })
            .collect::<Vec<_>>();
        if !listed.is_empty() {
            let text = format!(
                "little variation left after fe: {effects_text}: {}",
                listed.join(", ")
            );
            warnings.push(format!("warning[W1004]: {text}"));
            ctx.message(
                Diagnostic::warning("W1004", text)
                    .help("the regressor barely varies within the effects' groups, so its estimate is poorly determined; a coarser effect, or the regressor's group mean as a control, keeps the variation"),
            );
        }
    }
    // The clusters of the rows used, from the cluster variable, by identity of its values: the
    // integer-coded index of the effect on the same variable when there is one, else one built
    // the same way (section 8.3, point 10, "Reused fixed-effect work").
    let clusters = match &cluster {
        Some(name) => {
            let reused = fe.as_ref().and_then(|w| {
                w.absorbed
                    .effects
                    .iter()
                    .find(|e| e.variables.len() == 1 && e.variables[0] == *name)
                    .map(|e| Clusters::from_groups(&e.group_of_row, e.levels()))
            });
            Some(match reused {
                Some(clusters) => clusters,
                None => {
                    let column = frame
                        .table
                        .column(name)
                        .ok_or_else(|| internal(ctx, "the cluster variable vanished"))?;
                    let groups = EffectGroups::over(
                        name,
                        std::slice::from_ref(name),
                        &[column],
                        &design.rows,
                    );
                    Clusters::from_groups(&groups.group_of_row, groups.levels())
                }
            })
        }
        None => None,
    };
    let bread = Ols.bread(&fitted);
    // The parameters of the absorbed effects in every N − k: the levels less one per effect
    // after the first, the effects nested within the cluster variable left out.
    let absorbed = fe.as_ref().map_or(0, |w| {
        absorbed_parameters(&w.absorbed.effects, clusters.as_ref())
    });
    let (vcov, df) = match variance::variance(
        kind,
        &Ols,
        &design,
        &fitted,
        &bread,
        clusters.as_ref(),
        absorbed,
    ) {
        Ok(pair) => pair,
        Err(VarianceError::Asymmetric) => {
            return Err(internal(ctx, "an asymmetric variance matrix"));
        }
        Err(VarianceError::LeverageOne(rows_at_one)) => {
            let word = match kind {
                Kind::Hc3 => "hc3",
                _ => "hc2",
            };
            return Err(fail(
                    Diagnostic::error(
                        "Z0802",
                        format!(
                            "`se: {word}` needs every observation's leverage below one, but {} leverage one: each is alone in a cell of the design",
                            if rows_at_one == 1 {
                                "1 row has".to_string()
                            } else {
                                format!("{} rows have", count(rows_at_one as u64))
                            }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "an observation that fits itself exactly")
                    .help("`se: robust` (HC1) keeps these rows; or drop the regressor that singles each out, such as a category level with one observation"),
                ));
        }
        Err(VarianceError::OneCluster) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0802",
                    format!(
                        "clustered errors need at least two clusters, and `{}` has one",
                        cluster.as_deref().unwrap_or("")
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(CLUSTER), "one cluster")
                .help("cluster by a variable with several values, or leave `cluster:` out"),
            ));
        }
    };
    if df == 0 {
        return Err(fail(
            Diagnostic::error(
                "Z0802",
                "no residual degrees of freedom: as many regressors as observations",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "nothing left to estimate the error from")
            .help("a model needs more observations than regressors; drop terms or use more data"),
        ));
    }
    // Self-check (section 11.1): a symmetric variance matrix with no negative variance.
    let k = fitted.kept.len();
    for i in 0..k {
        if vcov.get(i, i).is_nan() || vcov.get(i, i) < 0.0 {
            return Err(internal(ctx, "a negative variance"));
        }
        for j in 0..i {
            let (a, b) = (vcov.get(i, j), vcov.get(j, i));
            if (a - b).abs() > 1e-10 * a.abs().max(b.abs()).max(f64::MIN_POSITIVE) {
                return Err(internal(ctx, "an asymmetric variance matrix"));
            }
        }
    }
    // The statistical warnings: few clusters (W1002), extreme weights (W1003), an observation
    // that drives a coefficient (W1006).
    if let Some(clusters) = &clusters {
        if clusters.singletons > 0 {
            let text = format!(
                "{} of the {} clusters hold one observation",
                count(clusters.singletons as u64),
                count(clusters.count as u64)
            );
            notes.push(text.clone());
            ctx.message(Diagnostic::plain_note(text));
        }
        if clusters.count < FEW_CLUSTERS {
            let text = format!(
                "few clusters: {} clusters give clustered errors little to rest on",
                count(clusters.count as u64)
            );
            warnings.push(format!("warning[W1002]: {text}"));
            ctx.message(
                Diagnostic::warning("W1002", text)
                    .help("the wild cluster bootstrap (`se: bootstrap`, release 1.1) is the usual remedy below 30 clusters"),
            );
        }
    }
    if let Some(w) = &design.weights {
        let effective = effective_sample_size(w);
        let n = design.n() as f64;
        if n > 0.0 && effective < EXTREME_WEIGHTS_SHARE * n {
            let text = format!(
                "extreme weights: the effective sample size is {} of {} rows",
                count(effective.round() as u64),
                count(design.n() as u64)
            );
            warnings.push(format!("warning[W1003]: {text}"));
            ctx.message(
                Diagnostic::warning("W1003", text)
                    .help("a few rows carry most of the weight; check them, or trim the weights"),
            );
        }
    }
    // An observation's leverage in the full design is not available with absorbed effects, so
    // W1006 is judged without them.
    let driving = if fe.is_some() {
        Vec::new()
    } else {
        driving_observations(&design, &fitted, &bread)
    };
    if !driving.is_empty() {
        let listed = driving
            .iter()
            .take(5)
            .map(|r| count(*r as u64))
            .collect::<Vec<_>>()
            .join(", ");
        let more = if driving.len() > 5 {
            format!(" and {} more", count((driving.len() - 5) as u64))
        } else {
            String::new()
        };
        let (observations, verb, which, plural) = if driving.len() == 1 {
            ("1 observation".to_string(), "drives", "it", "")
        } else {
            (
                format!("{} observations", count(driving.len() as u64)),
                "drive",
                "one",
                "s",
            )
        };
        let text = format!(
            "{observations} {verb} a coefficient: removing {which} moves an estimate by more than its standard error (row{plural} {listed}{more})"
        );
        warnings.push(format!("warning[W1006]: {text}"));
        ctx.message(
            Diagnostic::warning("W1006", text)
                .help("check the row's values; `influence()` lists leverage, Cook's distance and studentized residuals by row"),
        );
    }
    // The summary.
    let se_line = match kind {
        Kind::Classical => "classical".to_string(),
        Kind::Hc1 => "robust (HC1)".to_string(),
        Kind::Hc2 => "hc2".to_string(),
        Kind::Hc3 => "hc3".to_string(),
        Kind::Cluster => {
            let c = clusters.as_ref().expect("clusters");
            format!(
                "clustered by {} ({} clusters, CR1)",
                cluster.as_deref().unwrap_or(""),
                count(c.count as u64)
            )
        }
    };
    let n_obs = match design.weight_kind {
        Some(WeightKind::Frequency) => design.weight_total().round() as usize,
        _ => design.n(),
    };
    let mut note = n_note(&design);
    if design.weight_kind == Some(WeightKind::Frequency) {
        let sum_text = format!("the sum of the frequency weights over {}", rows(design.n()));
        note = Some(match note {
            Some(rest) => format!("{sum_text}; {rest}"),
            None => sum_text,
        });
    }
    if let Some(work) = &fe {
        let singletons = match work.absorbed.singletons {
            0 => "no singletons".to_string(),
            1 => "1 row dropped as a singleton".to_string(),
            n => format!("{} dropped as singletons", rows(n)),
        };
        note = Some(match note {
            Some(rest) => format!("{rest}; {singletons}"),
            None => singletons,
        });
    }
    let more_lines = match (&weights, design.weight_kind) {
        (Some(name), Some(kind)) => vec![format!(
            "weights: {name} ({})",
            match kind {
                WeightKind::Frequency => "frequency",
                WeightKind::Analytic => "analytic",
                WeightKind::Sampling => "sampling",
            }
        )],
        _ => Vec::new(),
    };
    let formula_text = formula.to_string();
    let setting = Setting {
        method: "reg",
        formula: &formula_text,
        se_line: &se_line,
        df,
        level,
        n: n_obs,
        n_note: note,
        more_lines,
        model: formula.clone(),
        levels: fe
            .as_ref()
            .map_or_else(|| design.category_levels(), |w| w.levels.clone()),
        frame_rows: frame.table.rows(),
        effects: fe.as_ref().map_or_else(Vec::new, |w| w.effects.clone()),
        absorbed_parameters: fe
            .as_ref()
            .map_or(0, |w| absorbed_parameters(&w.absorbed.effects, None)),
        singletons: fe.as_ref().map_or(0, |w| w.absorbed.singletons),
    };
    let mut fit = summary::linear_fit(&design, &fitted, &vcov, &setting);
    fit.warnings = warnings;
    fit.notes.extend(notes);
    // The residuals and fitted values of the rows, unweighted.
    if let Some(w) = &design.weights {
        for (i, root) in w.iter().map(|w| w.sqrt()).enumerate() {
            fit.residuals[i] /= root;
            fit.fitted[i] /= root;
        }
    }
    // With fixed effects the fitted values include the effects' part, as fixest's `fitted`:
    // the outcome less the residual, which the projection leaves unchanged.
    if let Some(original) = &design.original_outcome {
        for (i, y) in original.iter().enumerate() {
            fit.fitted[i] = y - fit.residuals[i];
        }
    }
    if fit.r_squared.is_none() {
        let outcome = formula
            .outcome
            .as_ref()
            .map_or(String::new(), |o| o.text.clone());
        let text = format!("`{outcome}` has no variation, so R² is undefined");
        fit.notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    if df > 0 && essentially_perfect(fit.sigma, &fit.fitted) {
        let text =
            "the fit is essentially perfect, so the standard errors and t values say nothing";
        fit.notes.push(text.to_string());
        ctx.message(Diagnostic::plain_note(text));
    }
    Ok(Value::Fit(Rc::new(fit)))
}

/// The positions of `reg`'s arguments after the frame and the formula.
const FE: usize = 2;
const SE: usize = 3;
const CLUSTER: usize = 4;
const WEIGHTS: usize = 5;
const WEIGHT_TYPE: usize = 6;
const LEVEL: usize = 7;

/// The design with the fixed effects projected out, and its fit (section 7.2, "Fixed effects"
/// and "Rows"): the rows with `NA` in an effect dropped with the model variables', the effects'
/// groups over the rows used (reused from the last set of effects over the same frame, rows and
/// weights, with every demeaned column they share), the kernel of step 17 on the columns and the
/// outcome, singletons dropped and counted, the intercept absorbed, and the effects' estimates
/// recovered for `predict`.
#[allow(clippy::too_many_arguments)]
fn fit_with_effects(
    ctx: &mut CallContext<'_>,
    frame: &zeff_core::value::Frame,
    formula: &Formula,
    extra: &[&str],
    specs: &[(String, Vec<String>)],
    effects_text: &str,
    weights: Option<&str>,
    weight_kind: WeightKind,
) -> Result<(Design, Fitted, FixedEffectWork), Fail> {
    let effect_variables: Vec<&str> = specs
        .iter()
        .flat_map(|(_, vars)| vars.iter().map(String::as_str))
        .collect();
    let mut design = Design::build_absorbing(ctx, frame, formula, extra, &effect_variables)?;
    if let Some(name) = weights {
        design = design.weighted(ctx, frame, name, weight_kind)?;
    }
    if design.n() == 0 {
        // The design's own refusal (Z0801) comes from the fit; the projection cannot run on no
        // rows, so the fit is asked first.
        Ols.fit(ctx, &design)?;
    }
    // The group indexes, reused when the last ones were built over this frame, these effects,
    // these rows and these weights; the demeaned columns they share come with them.
    let reused = GROUPS.with(|kept| {
        kept.borrow().as_ref().and_then(|k| {
            (Rc::ptr_eq(&k.table, &frame.table)
                && k.effects_text == effects_text
                && k.rows == design.rows
                && k.weights == design.weights)
                .then(|| (Rc::clone(&k.groups), k.demeaned.clone()))
        })
    });
    let (groups, known) = match reused {
        Some(found) => {
            WORK.with(|w| w.borrow_mut().1 += 1);
            found
        }
        None => {
            WORK.with(|w| w.borrow_mut().0 += 1);
            let groups: Vec<EffectGroups> = specs
                .iter()
                .map(|(name, vars)| {
                    let columns: Vec<&Column> = vars
                        .iter()
                        .map(|v| {
                            frame
                                .table
                                .column(v)
                                .expect("the design checked the variable")
                        })
                        .collect();
                    EffectGroups::over(name, vars, &columns, &design.rows)
                })
                .collect();
            (Rc::new(groups), BTreeMap::new())
        }
    };
    let (absorbed_design, absorbed) = design.absorb(&groups, Options::default(), &known);
    GROUPS.with(|kept| {
        *kept.borrow_mut() = Some(GroupsKept {
            table: Rc::clone(&frame.table),
            effects_text: effects_text.to_string(),
            rows: design.rows.clone(),
            weights: design.weights.clone(),
            groups: Rc::clone(&groups),
            demeaned: absorbed.demeaned.clone(),
        });
    });
    // Self-check (section 11.1): converged demeaning.
    if !absorbed.converged {
        return Err(internal(
            ctx,
            &format!(
                "the demeaning did not converge in {} passes (largest change {:.3e})",
                count(absorbed.passes as u64),
                absorbed.largest_change
            ),
        ));
    }
    if absorbed_design.n() == 0 {
        return Err(fail(
            Diagnostic::error(
                "Z0801",
                format!(
                    "no row remains to fit the model: every row is a singleton of fe: {effects_text}"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "nothing to estimate within the effects")
            .help("an effect whose every level has one row absorbs every row; use a coarser effect, or leave it out"),
        ));
    }
    // Every regressor absorbed: nothing varies within the effects' levels (the indicators of
    // units all treated at once under time effects, a group-level regressor under its group).
    if absorbed_design.k() > 0 && absorbed.absorbed_columns.len() == absorbed_design.k() {
        let listed = absorbed
            .absorbed_columns
            .iter()
            .map(|&c| format!("`{}`", absorbed_design.names[c]))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(fail(
            Diagnostic::error(
                "Z0802",
                format!("no regressor remains: every one is absorbed by fe: {effects_text} ({listed})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "nothing varies within the effects' levels")
            .help("a regressor must vary within the levels of every effect; the leads and lags of units all treated at once are the time effects themselves, so drop the time effect or add never-treated units"),
        ));
    }
    let fitted = Ols.fit(ctx, &absorbed_design)?;
    let values = effect_values(&design, &absorbed_design, &fitted, &absorbed);
    let effects: Vec<FixedEffect> = absorbed
        .effects
        .iter()
        .zip(values)
        .map(|(e, values)| FixedEffect {
            name: e.name.clone(),
            variables: e.variables.clone(),
            keys: e.keys.clone(),
            values,
        })
        .collect();
    let work = FixedEffectWork {
        levels: design.category_levels(),
        absorbed,
        effects,
    };
    Ok((absorbed_design, fitted, work))
}

/// The rows (1-based, in the frame) whose removal moves some estimate by more than
/// `DRIVING_DFBETAS` standard errors: DFBETAS from the leave-one-out formulas, which need only
/// the row, the bread, its residual and its leverage.
fn driving_observations(design: &Design, fitted: &Fitted, bread: &Matrix) -> Vec<usize> {
    let k = fitted.kept.len();
    let n = design.n();
    if n <= k + 1 {
        return Vec::new();
    }
    let rss = fitted.residual_sum_of_squares;
    let df = (n - k) as f64;
    let mut driving = Vec::new();
    let mut xi = vec![0.0; k];
    for block in design.blocks() {
        for r in 0..block.x.rows() {
            let row = block.start + r;
            let e = fitted.residuals[row];
            let h = fitted.leverage[row];
            if h >= 1.0 {
                continue;
            }
            for (p, &c) in fitted.kept.iter().enumerate() {
                xi[p] = block.x.get(r, c);
            }
            // The error variance without the row.
            let sigma2_without = (rss - e * e / (1.0 - h)) / (df - 1.0);
            if sigma2_without.is_nan() || sigma2_without <= 0.0 {
                continue;
            }
            let sigma_without = sigma2_without.sqrt();
            let scale = e / (1.0 - h);
            let drives = (0..k).any(|j| {
                let mut bx = 0.0;
                for (p, x) in xi.iter().enumerate() {
                    bx += bread.get(j, p) * x;
                }
                let dfbetas = bx * scale / (sigma_without * bread.get(j, j).sqrt());
                dfbetas.abs() > DRIVING_DFBETAS
            });
            if drives {
                driving.push(design.rows[row] + 1);
            }
        }
    }
    driving
}

/// R's rule for an essentially perfect fit (`summary.lm`): the residual variance below 1e-30 of
/// the fitted values' scale, their squared mean plus their variance, when the standard errors
/// are rounding and nothing more. No fitted value is no fit.
fn essentially_perfect(sigma: f64, fitted: &[f64]) -> bool {
    if fitted.is_empty() {
        return false;
    }
    let n = fitted.len() as f64;
    let mean = sum(fitted) / n;
    let squares: Vec<f64> = fitted.iter().map(|f| (f - mean) * (f - mean)).collect();
    let variance = if n > 1.0 {
        sum(&squares) / (n - 1.0)
    } else {
        0.0
    };
    sigma * sigma < (mean * mean + variance) * 1e-30
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

#[cfg(test)]
mod tests {
    use super::essentially_perfect;

    /// Fitted values −1, 0, 1 have mean zero and variance one, so the threshold is 1e-30 of the
    /// residual variance: a divisor of the variance other than n − 1 (n + 1, n, or a product)
    /// moves it to 0.5, 0.667 or 4 times that.
    #[test]
    fn the_threshold_is_the_squared_mean_plus_the_variance_over_n_minus_one() {
        let fitted = [-1.0, 0.0, 1.0];
        assert!(essentially_perfect((0.9e-30_f64).sqrt(), &fitted));
        assert!(!essentially_perfect((1.1e-30_f64).sqrt(), &fitted));
        // The mean's share: constant fitted values of 10 have no variance and a squared mean of
        // a hundred.
        let constant = [10.0, 10.0, 10.0];
        assert!(essentially_perfect((99e-30_f64).sqrt(), &constant));
        assert!(!essentially_perfect((101e-30_f64).sqrt(), &constant));
    }

    /// One fitted value has no variance, not a division of zero by zero, and no fitted value
    /// is no fit.
    #[test]
    fn one_fitted_value_has_a_variance_of_zero() {
        assert!(essentially_perfect((3e-30_f64).sqrt(), &[2.0]));
        assert!(!essentially_perfect((5e-30_f64).sqrt(), &[2.0]));
        assert!(!essentially_perfect(0.0, &[]));
    }

    /// An outcome of zeros fits with fitted values of zero and a residual variance of zero:
    /// its scale is zero, and R's comparison is strict, so `summary.lm` gives no warning for
    /// it and nor does the rule here; the note that the outcome has no variation covers it.
    /// Constant fitted values that are not zero, with a residual variance of zero, are perfect.
    #[test]
    fn an_outcome_of_zeros_is_not_called_perfect() {
        assert!(!essentially_perfect(0.0, &[0.0, 0.0, 0.0]));
        assert!(essentially_perfect(0.0, &[5.0, 5.0, 5.0]));
    }
}
