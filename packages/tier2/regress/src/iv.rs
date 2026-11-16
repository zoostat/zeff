//! Two-stage least squares (Engine and Project Specification, section 13; Language Reference,
//! sections 7.1 and 7.2; release 0.8.0, step 10). The instruments are the regressors that are
//! not instrumented, the intercept among them, followed by the excluded instruments. The first
//! stage is one blocked QR of the instruments, scaled to unit norm, with the instrumented
//! regressors beside them: the pivoted R drops an instrument collinear with the ones before it
//! and gives every first-stage coefficient by back substitution. The second stage is least
//! squares on the fitted regressors, a regressor that is not instrumented keeping its own
//! values, so that the estimates, the bread and the leverages are those of that design; the
//! residuals are then the structural ones, the outcome less the regressors as observed times
//! the estimates, and the variance layer computes every kind of standard error from the fitted
//! regressors' scores with them. A model whose fitted regressors are collinear where the
//! regressors themselves are not is under-identified (Z0804).

use crate::ols::{Ols, PIVOT_TOLERANCE, internal, scaled_norm};
use crate::reg::{
    absorbed_parameters, effect_specs, formula_variables, project, refuse_all_absorbed,
};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use zeff_core::runtime::{CallContext, Fail, fail};
use zeff_core::value::{Formula, Frame, Term, TermKind, terms_text};
use zeff_diagnostics::Diagnostic;
use zeff_kernels::cholesky::{cholesky, solve_with_factor};
use zeff_kernels::demean::Options;
use zeff_kernels::eigen::symmetric_eigen;
use zeff_kernels::product::{matmul, transpose_matmul};
use zeff_kernels::qr::{TallSkinnyQr, pivoted_qr};
use zeff_kernels::svd::condition_number;
use zeff_kernels::triangular::solve_upper;
use zeff_kernels::{LaneSum, Matrix, dot, sum_of_squares};
use zeff_numeric::order::WeightKind;
use zeff_results::absorb::Absorbed;
use zeff_results::design::{Block, Design};
use zeff_results::estimator::{Estimator, Fitted};
use zeff_results::variance::{self, Clusters, Kind, VarianceError};

/// A column of the model: one of the structural design's, or one of the excluded instruments'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    /// A column of the structural design, by its index there: a regressor.
    Structural(usize),
    /// A column of the excluded instruments' design, by its index there.
    Excluded(usize),
}

/// The designs of an instrumental-variables model on the same rows: the structural design, as
/// `reg` would build it, and the design of the excluded instruments.
pub struct Designs {
    /// The outcome and the regressors as observed.
    pub structural: Design,
    /// The excluded instruments, coded as terms of a formula with the structural formula's
    /// intercept; its own `(intercept)` column, when it has one, is no instrument (the
    /// structural design's is).
    pub excluded: Design,
    /// The structural columns that are instrumented, in the design's order.
    pub endogenous: Vec<usize>,
    /// The instruments in order: the structural columns that are not instrumented, then the
    /// excluded instruments.
    pub instruments: Vec<Column>,
    /// The instruments given that are regressors of the formula already, by name: included
    /// instruments, which identify nothing and are not among the excluded ones.
    pub included: Vec<String>,
    /// With fixed effects projected out: the effects as written, and the columns of the
    /// excluded instruments' design that the effects absorbed (an instrument that does not
    /// vary within the effects' levels).
    pub effects: Option<(String, Vec<usize>)>,
}

/// What projecting the fixed effects out of an instrumental-variables model found, beside the
/// designs.
pub struct Projection {
    /// The structural design before the projection, on the rows before the singletons were
    /// dropped: the effects' estimates and the levels of its categories are read from it.
    pub original: Design,
    /// What the projection of the structural design found: the effects over the rows kept, the
    /// singletons dropped, the regressors absorbed.
    pub absorbed: Absorbed,
}

impl Projection {
    /// The parameters the absorbed effects stand for in every N − k (the ruling of release
    /// 0.6.0): the levels of every effect, less one for each effect after the first, an effect
    /// nested within the clusters left out.
    pub fn parameters(&self, clusters: Option<&Clusters>) -> usize {
        absorbed_parameters(&self.absorbed.effects, clusters)
    }
}

impl Designs {
    /// The name of a column.
    pub fn name(&self, column: Column) -> &str {
        match column {
            Column::Structural(c) => &self.structural.names[c],
            Column::Excluded(c) => &self.excluded.names[c],
        }
    }

    /// The values of a column, one for each row used, unweighted.
    pub fn values(&self, column: Column) -> Vec<f64> {
        match column {
            Column::Structural(c) => self.structural.column_values(c),
            Column::Excluded(c) => self.excluded.column_values(c),
        }
    }

    /// The instruments that are not regressors, as positions among `instruments`.
    pub fn excluded_instruments(&self) -> Vec<usize> {
        (0..self.instruments.len())
            .filter(|&i| matches!(self.instruments[i], Column::Excluded(_)))
            .collect()
    }

    /// The designs of `formula` with the terms of `endogenous` instrumented by those of
    /// `excluded`, on the rows of `frame` without `NA` in any variable of either and in the
    /// variables of `extra` (weights, a cluster); weighted by `weights` when given. An
    /// instrumented term is a term of the formula, as written or as the formula's products
    /// expand to it (Z0504 otherwise); an excluded instrument that is a regressor already is no
    /// excluded instrument and is left out here, and one that is instrumented is refused
    /// (Z0505).
    pub fn build(
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        formula: &Formula,
        endogenous: &[Term],
        excluded: &[Term],
        extra: &[&str],
        weights: Option<(&str, WeightKind)>,
    ) -> Result<Designs, Fail> {
        Designs::build_full(
            ctx, frame, formula, endogenous, excluded, extra, weights, None,
        )
        .map(|(designs, _)| designs)
    }

    /// The designs with the fixed effects `effects` (as `fe:` gives them) projected out of the
    /// outcome, the regressors and the instruments together (section 7.2, "Fixed effects"): the
    /// rows with `NA` in an effect dropped with the model variables', singletons dropped and
    /// counted, the intercept absorbed. A fixed effect that is a variable of the formula or of
    /// the instruments is refused, as `reg` refuses it; a regressor or an instrument that does
    /// not vary within the effects' levels is absorbed.
    #[allow(clippy::too_many_arguments)]
    pub fn build_absorbing(
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        formula: &Formula,
        endogenous: &[Term],
        excluded: &[Term],
        extra: &[&str],
        weights: Option<(&str, WeightKind)>,
        effects: &[Term],
    ) -> Result<(Designs, Projection), Fail> {
        let (designs, projection) = Designs::build_full(
            ctx,
            frame,
            formula,
            endogenous,
            excluded,
            extra,
            weights,
            Some(effects),
        )?;
        let projection =
            projection.ok_or_else(|| internal(ctx, "fixed effects without a projection"))?;
        Ok((designs, projection))
    }

    #[allow(clippy::too_many_arguments)]
    fn build_full(
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        formula: &Formula,
        endogenous: &[Term],
        excluded: &[Term],
        extra: &[&str],
        weights: Option<(&str, WeightKind)>,
        effects: Option<&[Term]>,
    ) -> Result<(Designs, Option<Projection>), Fail> {
        let columns: Vec<String> = frame.table.names().to_vec();
        // Each design drops the rows with `NA` in the other's variables too.
        let of_terms = |terms: &[Term], outcome: Option<&Term>| -> Vec<String> {
            formula_variables(
                &Formula {
                    outcome: outcome.cloned(),
                    terms: terms.to_vec(),
                    intercept: false,
                },
                &columns,
            )
        };
        let structural_variables = of_terms(&formula.terms, formula.outcome.as_ref());
        let excluded_variables = of_terms(excluded, None);
        let with = |more: &[String]| -> Vec<String> {
            let mut all: Vec<String> = extra.iter().map(|e| e.to_string()).collect();
            for name in more {
                if !all.contains(name) {
                    all.push(name.clone());
                }
            }
            all
        };
        let structural_extra = with(&excluded_variables);
        let excluded_extra = with(&structural_variables);
        let structural_extra: Vec<&str> = structural_extra.iter().map(String::as_str).collect();
        let excluded_extra: Vec<&str> = excluded_extra.iter().map(String::as_str).collect();
        // A fixed effect that is a variable of the model or of the instruments is refused.
        let specs = effects.map(effect_specs).unwrap_or_default();
        let effects_text = effects.map(terms_text);
        let effect_variables: Vec<&str> = specs
            .iter()
            .flat_map(|(_, vars)| vars.iter().map(String::as_str))
            .collect();
        for name in &effect_variables {
            let role = if structural_variables.iter().any(|v| v == name) {
                Some(("a variable of the formula", "a regressor or the outcome"))
            } else if excluded_variables.iter().any(|v| v == name) {
                Some(("an instrument", "an instrument"))
            } else {
                None
            };
            if let Some((what, cannot)) = role {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`{name}` is a fixed effect of `fe:` and {what} too"),
                    )
                    .in_file(ctx.script)
                    .primary(
                        ctx.callee_span,
                        if what == "an instrument" {
                            "absorbed and an instrument"
                        } else {
                            "absorbed and in the model"
                        },
                    )
                    .help(format!("an absorbed effect cannot be {cannot} as well; drop it from `fe:` or from the {}", if what == "an instrument" { "instruments" } else { "formula" })),
                ));
            }
        }
        let instruments_formula = Formula {
            outcome: None,
            terms: excluded.to_vec(),
            intercept: formula.intercept,
        };
        let (mut structural, mut instruments) = if effects.is_some() {
            (
                Design::build_absorbing(ctx, frame, formula, &structural_extra, &effect_variables)?,
                Design::build_absorbing(
                    ctx,
                    frame,
                    &instruments_formula,
                    &excluded_extra,
                    &effect_variables,
                )?,
            )
        } else {
            (
                Design::build(ctx, frame, formula, &structural_extra)?,
                Design::build(ctx, frame, &instruments_formula, &excluded_extra)?,
            )
        };
        // The rows of both: a term computed from its variables (`log(x)`) may be `NA` where
        // they are not, in one design alone.
        if structural.rows != instruments.rows {
            let rows: Vec<usize> = structural
                .rows
                .iter()
                .copied()
                .filter(|row| instruments.rows.binary_search(row).is_ok())
                .collect();
            let dropped = frame.table.rows() - rows.len();
            for design in [&mut structural, &mut instruments] {
                design.rows = rows.clone();
                design.dropped = dropped;
            }
        }
        for (name, count) in &instruments.dropped_in {
            if !structural.dropped_in.iter().any(|(n, _)| n == name) {
                structural.dropped_in.push((name.clone(), *count));
            }
        }
        if let Some((name, kind)) = weights {
            structural = structural.weighted(ctx, frame, name, kind)?;
            instruments = instruments.weighted(ctx, frame, name, kind)?;
        }
        // The instrumented terms among the structural design's, by what each is made of.
        let terms = structural.terms();
        let made_of = |parts: &BTreeSet<String>| -> BTreeSet<String> {
            parts
                .iter()
                .map(|p| p.strip_prefix("category:").unwrap_or(p).to_string())
                .collect()
        };
        let mut wanted: Vec<(String, BTreeSet<String>)> = Vec::new();
        for term in endogenous {
            parts_of(ctx, term, &columns, &mut wanted)?;
        }
        let mut instrumented: Vec<usize> = Vec::new();
        for (text, parts) in &wanted {
            let Some(found) = terms.iter().find(|t| made_of(&t.parts) == *parts) else {
                let mut with: Vec<String> = formula.terms.iter().map(|t| t.text.clone()).collect();
                with.push(text.clone());
                return Err(fail(
                    Diagnostic::error(
                        "Z0504",
                        format!("`{text}` is instrumented but is not a regressor"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "instrumented, missing from the formula")
                    .help(format!(
                        "add it to the regressors: `{} ~ {}`",
                        formula.outcome.as_ref().map_or("y", |o| o.text.as_str()),
                        with.join(" + ")
                    )),
                ));
            };
            for c in found.columns.clone() {
                if !instrumented.contains(&c) {
                    instrumented.push(c);
                }
            }
        }
        instrumented.sort_unstable();
        // The instruments: the regressors not instrumented, then the excluded ones, less those
        // that are regressors under the same name.
        let mut listed: Vec<Column> = (0..structural.k())
            .filter(|c| !instrumented.contains(c))
            .map(Column::Structural)
            .collect();
        let mut included: Vec<String> = Vec::new();
        for c in usize::from(instruments.intercept)..instruments.k() {
            let name = &instruments.names[c];
            match structural.names.iter().position(|n| n == name) {
                Some(j) if instrumented.contains(&j) => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0505",
                            format!("`{name}` is on both sides of `~` in `instruments:`"),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.callee_span, "instrumented by itself")
                        .help("an instrument is a variable outside the model that moves the instrumented regressor; a regressor cannot instrument itself"),
                    ));
                }
                Some(_) => included.push(name.clone()),
                None => listed.push(Column::Excluded(c)),
            }
        }
        let Some(effects_text) = effects_text else {
            return Ok((
                Designs {
                    structural,
                    excluded: instruments,
                    endogenous: instrumented,
                    instruments: listed,
                    included,
                    effects: None,
                },
                None,
            ));
        };
        // The effects out of both designs, on the same rows: the projection of the structural
        // design decides the rows kept (the singletons go), and the instruments' design, over
        // the same groups, keeps the same.
        if structural.n() == 0 {
            // The design's own refusal (Z0801) comes from the fit.
            Ols.fit(ctx, &structural)?;
        }
        let (projected, absorbed, groups) =
            project(ctx, frame, &structural, &specs, &effects_text)?;
        refuse_all_absorbed(ctx, &projected, &absorbed, &effects_text)?;
        // A model without an excluded instrument has nothing to project there: its design of
        // no columns stands on the rows the structural design kept.
        let (projected_instruments, absorbed_instruments) = if instruments.k() == 0 {
            (
                projected.with_model(
                    Vec::new(),
                    Vec::new(),
                    Rc::new(vec![0.0; projected.n()]),
                    "",
                    false,
                ),
                Vec::new(),
            )
        } else {
            let (design, found) = instruments.absorb(&groups, Options::default(), &BTreeMap::new());
            if !found.converged {
                return Err(internal(
                    ctx,
                    "the demeaning of the instruments did not converge",
                ));
            }
            if found.kept != absorbed.kept {
                return Err(internal(
                    ctx,
                    "the instruments and the regressors kept different rows under the effects",
                ));
            }
            (design, found.absorbed_columns)
        };
        Ok((
            Designs {
                structural: projected,
                excluded: projected_instruments,
                endogenous: instrumented,
                instruments: listed,
                included,
                effects: Some((effects_text, absorbed_instruments)),
            },
            Some(Projection {
                original: structural,
                absorbed,
            }),
        ))
    }
}

/// What each instrumented term is made of, by its written form: a variable, the parts of an
/// interaction, an expression as written, or each variable of a range.
fn parts_of(
    ctx: &CallContext<'_>,
    term: &Term,
    columns: &[String],
    out: &mut Vec<(String, BTreeSet<String>)>,
) -> Result<(), Fail> {
    match &term.kind {
        TermKind::Var(name) => out.push((term.text.clone(), BTreeSet::from([name.clone()]))),
        TermKind::Expr(_) => out.push((term.text.clone(), BTreeSet::from([term.text.clone()]))),
        TermKind::Range(a, b) => {
            let ends = (
                columns.iter().position(|c| c == a),
                columns.iter().position(|c| c == b),
            );
            let named: Vec<String> = match ends {
                (Some(i), Some(j)) if i <= j => columns[i..=j].to_vec(),
                _ => vec![a.clone(), b.clone()],
            };
            for name in named {
                out.push((name.clone(), BTreeSet::from([name])));
            }
        }
        TermKind::Interaction(parts) => {
            let mut set = BTreeSet::new();
            for part in parts {
                let mut inner = Vec::new();
                parts_of(ctx, part, columns, &mut inner)?;
                for (_, names) in inner {
                    set.extend(names);
                }
            }
            out.push((term.text.clone(), set));
        }
        TermKind::Product(_) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{}` cannot be instrumented as one term: a product stands for several",
                        term.text
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a product on the left of `~`")
                .help("name each instrumented term: `instruments: a + a:b ~ z + z:b`"),
            ));
        }
    }
    Ok(())
}

/// What the first stage found.
#[derive(Debug, Clone, PartialEq)]
pub struct FirstStage {
    /// The instruments with a coefficient, as positions among the designs' `instruments`.
    pub kept: Vec<usize>,
    /// The instruments dropped as collinear with the ones before them, the zero, vanishing and
    /// oversized ones among them.
    pub omitted: Vec<usize>,
    /// The omitted instruments that are zero in every row used.
    pub zero: Vec<usize>,
    /// The omitted instruments whose values are too small to be scaled.
    pub vanishing: Vec<usize>,
    /// The omitted instruments whose values are too large to be scaled.
    pub oversized: Vec<usize>,
    /// For each instrumented regressor, in the order of the designs' `endogenous`, one
    /// coefficient per kept instrument, in the units of the data.
    pub coefficients: Vec<Vec<f64>>,
    /// The R factor of the kept instruments scaled to unit norm.
    pub r: Matrix,
    /// The scale of each kept instrument: the reciprocal of its norm.
    pub scales: Vec<f64>,
    /// The condition number of the scaled R.
    pub condition_number: f64,
}

/// The scaled QR of some columns of the designs: the sums of squares, the largest sizes, the
/// scales and the R factor, as least squares computes them for its own design.
struct Factor {
    sums: Vec<f64>,
    largest: Vec<f64>,
    scales: Vec<f64>,
    r: Matrix,
}

fn column_of<'a>(structural: &'a Block, excluded: &'a Block, column: Column) -> &'a [f64] {
    match column {
        Column::Structural(c) => structural.x.column(c),
        Column::Excluded(c) => excluded.x.column(c),
    }
}

fn factor(designs: &Designs, columns: &[Column]) -> Factor {
    let width = columns.len();
    let mut accumulators: Vec<LaneSum> = (0..width).map(|_| LaneSum::new()).collect();
    let mut largest: Vec<f64> = vec![0.0; width];
    for (bx, bz) in designs.structural.blocks().zip(designs.excluded.blocks()) {
        for (c, accumulator) in accumulators.iter_mut().enumerate() {
            let column = column_of(&bx, &bz, columns[c]);
            accumulator.push_squares(column);
            largest[c] = column.iter().fold(largest[c], |m, x| m.max(x.abs()));
        }
    }
    let sums: Vec<f64> = accumulators.iter().map(LaneSum::value).collect();
    let scales: Vec<f64> = sums
        .iter()
        .map(|s| {
            if *s > 0.0 && s.is_finite() {
                1.0 / s.sqrt()
            } else if s.is_infinite() {
                0.0
            } else {
                1.0
            }
        })
        .collect();
    let mut qr = TallSkinnyQr::new(width);
    for (bx, bz) in designs.structural.blocks().zip(designs.excluded.blocks()) {
        let mut scaled = Matrix::zeros(bx.x.rows(), width);
        for (c, &scale) in scales.iter().enumerate() {
            if scale == 0.0 {
                continue;
            }
            let column = scaled.column_mut(c);
            column.copy_from_slice(column_of(&bx, &bz, columns[c]));
            for v in column.iter_mut() {
                *v *= scale;
            }
        }
        qr.push_rows(&scaled);
    }
    Factor {
        sums,
        largest,
        scales,
        r: qr.finish(),
    }
}

/// The first stage: every instrumented regressor on the instruments, by one QR.
fn first_stage(ctx: &CallContext<'_>, designs: &Designs) -> Result<FirstStage, Fail> {
    let l = designs.instruments.len();
    let mut columns = designs.instruments.clone();
    columns.extend(designs.endogenous.iter().map(|&c| Column::Structural(c)));
    let factor = factor(designs, &columns);
    let pivoted = pivoted_qr(&factor.r, l, PIVOT_TOLERANCE);
    let rank = pivoted.rank;
    let kept: Vec<usize> = pivoted.order[..rank].to_vec();
    let mut omitted: Vec<usize> = pivoted.order[rank..l].to_vec();
    omitted.sort_unstable();
    let among = |test: &dyn Fn(usize) -> bool| -> Vec<usize> {
        omitted.iter().copied().filter(|&c| test(c)).collect()
    };
    let zero = among(&|c| factor.sums[c] == 0.0 && factor.largest[c] == 0.0);
    let vanishing = among(&|c| factor.sums[c] == 0.0 && factor.largest[c] > 0.0);
    let oversized = among(&|c| !factor.sums[c].is_finite());
    let r11 = Matrix::from_fn(rank, rank, |i, j| pivoted.r.get(i, j));
    let kept_scales: Vec<f64> = kept.iter().map(|&c| factor.scales[c]).collect();
    let mut coefficients = Vec::with_capacity(designs.endogenous.len());
    for q in 0..designs.endogenous.len() {
        let own = factor.scales[l + q];
        let qty: Vec<f64> = (0..rank).map(|i| pivoted.r.get(i, l + q)).collect();
        let scaled = solve_upper(&r11, &qty)
            .map_err(|_| internal(ctx, "a zero pivot among the accepted instruments"))?;
        // An instrumented regressor too large to be scaled has no first stage: its fitted
        // values are zero, and the second stage drops it as it drops the regressor itself.
        coefficients.push(
            scaled
                .iter()
                .zip(&kept_scales)
                .map(|(b, s)| if own == 0.0 { 0.0 } else { b * s / own })
                .collect(),
        );
    }
    Ok(FirstStage {
        kept,
        omitted,
        zero,
        vanishing,
        oversized,
        coefficients,
        condition_number: condition_number(&r11),
        r: r11,
        scales: kept_scales,
    })
}

/// The second stage as an estimator: least squares on the design of the fitted regressors,
/// with the residuals of the structural equation. Its scores are a row of fitted regressors
/// times that row's structural residual and its bread the inverse cross-product of the fitted
/// regressors, which are least squares' own on that design.
pub struct SecondStage<'a> {
    /// The outcome and the regressors as observed, on the rows of the design fitted.
    pub structural: &'a Design,
    /// The condition number of the first stage, which the self-check allows for.
    pub first_condition: f64,
}

impl Estimator for SecondStage<'_> {
    fn fit(&self, ctx: &mut CallContext<'_>, design: &Design) -> Result<Fitted, Fail> {
        let mut fitted = Ols.fit(ctx, design)?;
        let n = design.n();
        let rank = fitted.kept.len();
        let mut residuals = Vec::with_capacity(n);
        let mut values = Vec::with_capacity(n);
        let mut outcome = Vec::with_capacity(n);
        // The products of the fitted regressors with the structural residuals, zero in exact
        // arithmetic, and the norms of the regressors as observed.
        let mut products = vec![0.0; rank];
        let mut squares: Vec<LaneSum> = (0..rank).map(|_| LaneSum::new()).collect();
        let mut largest = vec![0.0_f64; rank];
        let mut row_values = vec![0.0; rank];
        for (observed, fit) in self.structural.blocks().zip(design.blocks()) {
            for (p, &c) in fitted.kept.iter().enumerate() {
                let column = observed.x.column(c);
                squares[p].push_squares(column);
                largest[p] = column.iter().fold(largest[p], |m, x| m.max(x.abs()));
            }
            for r in 0..observed.x.rows() {
                for (p, &c) in fitted.kept.iter().enumerate() {
                    row_values[p] = observed.x.get(r, c);
                }
                let value = dot(&row_values, &fitted.estimates);
                let e = observed.y[r] - value;
                for (p, &c) in fitted.kept.iter().enumerate() {
                    products[p] += fit.x.get(r, c) * e;
                }
                residuals.push(e);
                values.push(value);
                outcome.push(observed.y[r]);
            }
        }
        // Self-check (section 11.1): the structural residuals are orthogonal to the fitted
        // regressors, to within the rounding of the two stages, which grows with the size of
        // the outcome and of each regressor's part of the fitted values and with the condition
        // numbers of both stages.
        let slack =
            (1e-8_f64).max(10.0 * f64::EPSILON * (fitted.condition_number + self.first_condition));
        let mut size = scaled_norm(&residuals) + scaled_norm(&outcome);
        for ((squares, largest), estimate) in squares.iter().zip(&largest).zip(&fitted.estimates) {
            let sum = squares.value();
            let norm = if sum > 0.0 && sum.is_finite() {
                sum.sqrt()
            } else {
                largest * (n as f64).sqrt()
            };
            size += estimate.abs() * norm;
        }
        for (product, scale) in products.iter().zip(&fitted.scales) {
            let bound = slack * size / scale;
            if product.abs() > bound {
                return Err(internal(
                    ctx,
                    "the structural residuals are not orthogonal to the fitted regressors",
                ));
            }
        }
        fitted.residual_sum_of_squares = sum_of_squares(&residuals);
        fitted.residuals = residuals;
        fitted.fitted = values;
        Ok(fitted)
    }

    fn scores(&self, fitted: &Fitted, block: &Block, first_row: usize) -> Matrix {
        Ols.scores(fitted, block, first_row)
    }

    fn bread(&self, fitted: &Fitted) -> Matrix {
        Ols.bread(fitted)
    }
}

/// A two-stage least-squares fit.
pub struct TwoStage {
    pub designs: Designs,
    /// The first stage, or `None` when no regressor is instrumented.
    pub first: Option<FirstStage>,
    /// The design of the fitted regressors: the structural design's columns, an instrumented
    /// one replaced by its fitted values from the first stage.
    pub second: Design,
    /// The fit: the estimates, bread and leverages of least squares on the fitted regressors,
    /// with the residuals and fitted values of the structural equation.
    pub fitted: Fitted,
    /// What the first stage changed, for the notes under the fit.
    pub notes: Vec<String>,
}

impl TwoStage {
    /// The variance of the estimates under `kind`, and the degrees of freedom of the t table,
    /// from the variance layer; `absorbed` as there.
    pub fn variance(
        &self,
        kind: Kind,
        clusters: Option<&Clusters>,
        absorbed: usize,
    ) -> Result<(Matrix, usize), VarianceError> {
        let estimator = SecondStage {
            structural: &self.designs.structural,
            first_condition: self.first.as_ref().map_or(1.0, |f| f.condition_number),
        };
        variance::variance(
            kind,
            &estimator,
            &self.second,
            &self.fitted,
            &estimator.bread(&self.fitted),
            clusters,
            absorbed,
        )
    }
}

/// One first stage as a model of its own: an instrumented regressor on the instruments kept.
pub struct StageFit {
    /// The instruments kept for columns, in the order of the first stage's `kept`, and the
    /// instrumented regressor for outcome, on the rows and with the weights of the fit.
    pub design: Design,
    pub fitted: Fitted,
}

/// How strong the instruments are (release 0.8.0, step 12b; the ruling of the release on the
/// weak-instrument statistics): the F of the excluded instruments in each first stage, and the
/// Cragg–Donald F of them all together. A statistic that cannot be computed is `None`: a
/// first stage without residual degrees of freedom, a variance of the instruments'
/// coefficients that cannot be inverted (fewer clusters than instruments), first-stage
/// residuals that are collinear.
#[derive(Debug, Clone, PartialEq)]
pub struct Weak {
    /// The excluded instruments kept: the numerator's degrees of freedom.
    pub df1: usize,
    /// N less the instruments kept and the parameters of the absorbed effects: the classical
    /// denominator's degrees of freedom.
    pub df2: f64,
    /// For each instrumented regressor, the F of the excluded instruments in its first stage
    /// under classical errors.
    pub f: Vec<Option<f64>>,
    /// The same as a Wald statistic over `df1` on the variance asked for: robust, clustered,
    /// or classical, where it is `f`.
    pub wald: Vec<Option<f64>>,
    /// The Cragg–Donald F in Stock and Yogo's form: the smallest eigenvalue of the
    /// concentration matrix over `df1`; with one instrumented regressor it is `f`.
    pub cragg_donald: Option<f64>,
    /// The Kleibergen–Paap rk Wald F on the variance asked for: the test that the first-stage
    /// coefficients have less than full rank, over `df1`. With one instrumented regressor it
    /// is that regressor's `wald`; under classical errors it is the Cragg–Donald F.
    pub kleibergen_paap: Option<f64>,
    /// The effective F of Montiel Olea and Pflueger, for one instrumented regressor under
    /// robust or clustered errors.
    pub effective: Option<Effective>,
    /// Whether the variance asked for is robust or clustered: the F printed under a fit is then
    /// the Kleibergen–Paap one, and the Cragg–Donald one otherwise (the ruling of the release).
    pub robust: bool,
}

/// The effective F with what it is judged by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effective {
    /// The statistic: the first stage's explained sum of squares over the trace of the
    /// variance of its coefficients in the instruments' own metric.
    pub f: f64,
    /// The effective degrees of freedom, between one and the number of excluded instruments.
    pub degrees: f64,
    /// The critical value at the 5% level for a bias above a tenth of the worst case.
    pub critical: f64,
}

/// A first-stage F under this is taken for a weak instrument (Staiger and Stock's rule of
/// thumb, which the release's ruling adopts for the warning).
pub const WEAK_F: f64 = 10.0;
/// The share of the worst-case bias the effective F's critical value guards against, and the
/// level of its test.
pub const EFFECTIVE_BIAS: f64 = 0.1;
pub const EFFECTIVE_LEVEL: f64 = 0.05;

/// The value a non-central chi-square variable with `degrees` degrees of freedom and
/// non-centrality `noncentrality` exceeds with probability `upper`. The distribution is a
/// Poisson mixture of central chi-squares, with weights e^(−λ/2)(λ/2)^j / j! on the degrees
/// of freedom `degrees` + 2j; the point is found by halving an interval that holds it.
pub fn noncentral_chi_square_point(degrees: f64, noncentrality: f64, upper: f64) -> f64 {
    let half = noncentrality / 2.0;
    // The weights that matter: around the mean of the Poisson, forty of its standard
    // deviations and fifty terms either way, which leaves a tail far below rounding.
    let reach = 40.0 * half.sqrt() + 50.0;
    let first = (half - reach).max(0.0).floor() as usize;
    let last = (half + reach).ceil() as usize;
    let weights: Vec<f64> = (first..=last)
        .map(|j| {
            let j = j as f64;
            let log = if half > 0.0 {
                -half + j * zeff_numeric::maths::log(half)
                    - zeff_numeric::special::log_gamma(j + 1.0)
            } else if j == 0.0 {
                0.0
            } else {
                f64::NEG_INFINITY
            };
            zeff_numeric::maths::exp(log)
        })
        .collect();
    let above = |x: f64| -> f64 {
        let terms: Vec<f64> = weights
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let a = degrees / 2.0 + (first + i) as f64;
                w * zeff_numeric::special::incomplete_gamma(a, x / 2.0).upper
            })
            .collect();
        zeff_kernels::sum(&terms)
    };
    // The point lies between zero and the mean plus many standard deviations.
    let mut low = 0.0;
    let mut high =
        degrees + noncentrality + 60.0 * (2.0 * (degrees + 2.0 * noncentrality)).sqrt() + 60.0;
    for _ in 0..200 {
        let middle = 0.5 * (low + high);
        if above(middle) > upper {
            low = middle;
        } else {
            high = middle;
        }
        if high - low <= 1e-14 * high {
            break;
        }
    }
    0.5 * (low + high)
}

/// The symmetric power `m^exponent` of a symmetric positive-definite matrix, by its
/// eigenvalues; `None` when an eigenvalue is not positive beyond rounding.
fn symmetric_power(m: &Matrix, exponent: f64) -> Option<Matrix> {
    let k = m.rows();
    let spectrum = symmetric_eigen(m);
    let largest = spectrum.values.last().copied().unwrap_or(0.0);
    if spectrum
        .values
        .iter()
        .any(|v| v.is_nan() || *v <= COLLINEAR_RESIDUALS * largest)
    {
        return None;
    }
    let powers: Vec<f64> = spectrum
        .values
        .iter()
        .map(|v| {
            if exponent == 0.5 {
                v.sqrt()
            } else if exponent == -0.5 {
                1.0 / v.sqrt()
            } else {
                1.0 / v
            }
        })
        .collect();
    Some(Matrix::from_fn(k, k, |i, j| {
        (0..k)
            .map(|l| spectrum.vectors.get(i, l) * spectrum.vectors.get(j, l) * powers[l])
            .sum()
    }))
}

impl Weak {
    /// The statistical warning for weak instruments (W1007; the ruling of the release): the F
    /// printed under the fit is under [`WEAK_F`], or the effective F is under its critical
    /// value. `None` when neither is.
    pub fn warning(&self) -> Option<Diagnostic> {
        let (printed, name) = if self.robust {
            (self.kleibergen_paap, "Kleibergen–Paap")
        } else {
            (self.cragg_donald, "Cragg–Donald")
        };
        let by_f = printed
            .filter(|f| *f < WEAK_F)
            .map(|f| format!("the {name} F is {f:.2}, under {WEAK_F:.0}"));
        let by_effective = self.effective.filter(|e| e.f < e.critical).map(|e| {
            format!(
                "the effective F is {:.2}, under its critical value of {:.2}",
                e.f, e.critical
            )
        });
        let reasons: Vec<String> = by_f.into_iter().chain(by_effective).collect();
        if reasons.is_empty() {
            return None;
        }
        Some(
            Diagnostic::warning(
                "W1007",
                format!("weak instruments: {}", reasons.join(", and ")),
            )
            .help("with weak instruments the estimate leans towards the least-squares one and its confidence interval is too narrow; `ar_test()` gives a test and a confidence set that hold however weak the instruments are"),
        )
    }
}

/// vᵀ M⁻¹ v for a symmetric positive-definite M, each row and column first scaled by its
/// diagonal entry so that variables in very different units do not meet in one factorisation;
/// `None` when M is not positive definite to the factorisation.
fn quadratic_form(m: &Matrix, v: &[f64]) -> Option<f64> {
    let k = v.len();
    let scale: Vec<f64> = (0..k).map(|i| m.get(i, i)).collect();
    if scale.iter().any(|d| !(*d > 0.0 && d.is_finite())) {
        return None;
    }
    let scale: Vec<f64> = scale.iter().map(|d| 1.0 / d.sqrt()).collect();
    let scaled = Matrix::from_fn(k, k, |i, j| m.get(i, j) * scale[i] * scale[j]);
    let u: Vec<f64> = v.iter().zip(&scale).map(|(x, s)| x * s).collect();
    let factor = cholesky(&scaled).ok()?;
    let solved = solve_with_factor(&factor, &u).ok()?;
    let value = dot(&u, &solved);
    value.is_finite().then_some(value)
}

/// The block of `m` on the rows and columns `at`.
fn block(m: &Matrix, at: &[usize]) -> Matrix {
    Matrix::from_fn(at.len(), at.len(), |i, j| m.get(at[i], at[j]))
}

impl TwoStage {
    /// The first stages as models of their own, one per instrumented regressor in the order of
    /// the designs' `endogenous`: least squares of the regressor on the instruments kept.
    pub fn first_stages(&self, ctx: &mut CallContext<'_>) -> Result<Vec<StageFit>, Fail> {
        let Some(first) = &self.first else {
            return Ok(Vec::new());
        };
        let designs = &self.designs;
        let names: Vec<String> = first
            .kept
            .iter()
            .map(|&i| designs.name(designs.instruments[i]).to_string())
            .collect();
        let columns: Vec<Rc<Vec<f64>>> = first
            .kept
            .iter()
            .map(|&i| Rc::new(designs.values(designs.instruments[i])))
            .collect();
        let leverage = designs.structural.leverage_available();
        let mut stages = Vec::with_capacity(designs.endogenous.len());
        for &c in &designs.endogenous {
            let design = designs.structural.with_model(
                names.clone(),
                columns.clone(),
                Rc::new(designs.structural.column_values(c)),
                &designs.structural.names[c],
                leverage,
            );
            let fitted = Ols.fit(ctx, &design)?;
            stages.push(StageFit { design, fitted });
        }
        Ok(stages)
    }

    /// The weak-instrument statistics, with the Wald form on the variance of `kind` (with
    /// `clusters` when clustered) and the degrees of freedom of the fit; `projection` when the
    /// model has fixed effects.
    pub fn weak(
        &self,
        ctx: &mut CallContext<'_>,
        kind: Kind,
        clusters: Option<&Clusters>,
        projection: Option<&Projection>,
    ) -> Result<Weak, Fail> {
        let designs = &self.designs;
        let p = designs.endogenous.len();
        let Some(first) = &self.first else {
            return Ok(Weak {
                df1: 0,
                df2: f64::NAN,
                f: Vec::new(),
                wald: Vec::new(),
                cragg_donald: None,
                kleibergen_paap: None,
                effective: None,
                robust: kind != Kind::Classical,
            });
        };
        // The excluded instruments among the kept ones, by their position there.
        let excluded: Vec<usize> = (0..first.kept.len())
            .filter(|&e| matches!(designs.instruments[first.kept[e]], Column::Excluded(_)))
            .collect();
        let m = excluded.len();
        let plain = projection.map_or(0, |p| p.parameters(None));
        let n = match designs.structural.weight_kind {
            Some(WeightKind::Frequency) => designs.structural.weight_total(),
            _ => designs.structural.n() as f64,
        };
        let df2 = n - (first.kept.len() + plain) as f64;
        let unavailable = Weak {
            df1: m,
            df2,
            f: vec![None; p],
            wald: vec![None; p],
            cragg_donald: None,
            kleibergen_paap: None,
            effective: None,
            robust: kind != Kind::Classical,
        };
        if m == 0 || df2 <= 0.0 {
            return Ok(unavailable);
        }
        // A first stage that least squares refuses (no residual degrees of freedom) has no
        // statistic; an internal error is still one.
        let said = ctx.messages.len();
        let stages = match self.first_stages(ctx) {
            Ok(stages) => stages,
            Err(e) if e.code == "Z9001" => return Err(e),
            Err(_) => {
                ctx.messages.truncate(said);
                return Ok(unavailable);
            }
        };
        let mut f = Vec::with_capacity(p);
        let mut wald = Vec::with_capacity(p);
        // The coefficients of each first stage on the excluded instruments.
        let mut on_excluded: Vec<Vec<f64>> = Vec::with_capacity(p);
        for stage in &stages {
            let fitted = &stage.fitted;
            if fitted.kept.len() != first.kept.len() {
                return Err(internal(
                    ctx,
                    "a first stage kept other instruments than the first stages together",
                ));
            }
            let pi: Vec<f64> = excluded.iter().map(|&e| fitted.estimates[e]).collect();
            let bread = Ols.bread(fitted);
            let of = |kind: Kind,
                      clusters: Option<&Clusters>,
                      absorbed: usize|
             -> Result<Option<f64>, VarianceError> {
                match variance::variance(
                    kind,
                    &Ols,
                    &stage.design,
                    fitted,
                    &bread,
                    clusters,
                    absorbed,
                ) {
                    Ok((v, _)) => {
                        Ok(quadratic_form(&block(&v, &excluded), &pi).map(|w| w / m as f64))
                    }
                    Err(VarianceError::Asymmetric) => Err(VarianceError::Asymmetric),
                    Err(_) => Ok(None),
                }
            };
            let classical = of(Kind::Classical, None, plain)
                .map_err(|_| internal(ctx, "an asymmetric first-stage variance"))?;
            let asked = if kind == Kind::Classical {
                classical
            } else {
                let absorbed = projection.map_or(0, |p| p.parameters(clusters));
                of(kind, clusters, absorbed)
                    .map_err(|_| internal(ctx, "an asymmetric first-stage variance"))?
            };
            f.push(classical);
            wald.push(asked);
            on_excluded.push(pi);
        }
        // The Cragg–Donald F: the smallest eigenvalue of Σ^(-1/2) Πᵀ(Z₂ᵀM Z₂)Π Σ^(-1/2) over m,
        // where Z₂ᵀM Z₂, the excluded instruments with the others projected out, is the inverse
        // of their block of the bread, and Σ is the covariance of the first-stage residuals
        // with df2 for divisor. Each regressor is scaled by its residual variance first, which
        // changes no eigenvalue of the pair.
        let cragg_donald = (|| {
            let bread = block(&Ols.bread(&stages[0].fitted), &excluded);
            let scale: Vec<f64> = (0..m).map(|i| bread.get(i, i)).collect();
            if scale.iter().any(|d| !(*d > 0.0 && d.is_finite())) {
                return None;
            }
            let scale: Vec<f64> = scale.iter().map(|d| 1.0 / d.sqrt()).collect();
            let scaled = Matrix::from_fn(m, m, |i, j| bread.get(i, j) * scale[i] * scale[j]);
            let factor = cholesky(&scaled).ok()?;
            let u: Vec<Vec<f64>> = on_excluded
                .iter()
                .map(|pi| pi.iter().zip(&scale).map(|(x, s)| x * s).collect())
                .collect();
            let solved: Vec<Vec<f64>> = u
                .iter()
                .map(|u| solve_with_factor(&factor, u).ok())
                .collect::<Option<_>>()?;
            let sigma = Matrix::from_fn(p, p, |j, k| {
                dot(&stages[j].fitted.residuals, &stages[k].fitted.residuals) / df2
            });
            let own: Vec<f64> = (0..p).map(|j| sigma.get(j, j)).collect();
            if own.iter().any(|d| !(*d > 0.0 && d.is_finite())) {
                return None;
            }
            let own: Vec<f64> = own.iter().map(|d| 1.0 / d.sqrt()).collect();
            let sigma = Matrix::from_fn(p, p, |j, k| sigma.get(j, k) * own[j] * own[k]);
            let moved = Matrix::from_fn(p, p, |j, k| dot(&u[j], &solved[k]) * own[j] * own[k]);
            let spectrum = symmetric_eigen(&sigma);
            let (smallest, largest) = (spectrum.values[0], spectrum.values[p - 1]);
            // Residuals that are collinear have no covariance to invert.
            if smallest.is_nan() || smallest <= COLLINEAR_RESIDUALS * largest {
                return None;
            }
            // Σ^(-1/2) = U diag(λ^(-1/2)) Uᵀ.
            let root = Matrix::from_fn(p, p, |j, k| {
                (0..p)
                    .map(|l| {
                        spectrum.vectors.get(j, l) * spectrum.vectors.get(k, l)
                            / spectrum.values[l].sqrt()
                    })
                    .sum()
            });
            let concentration = matmul(&matmul(&root, &moved), &root);
            let value = symmetric_eigen(&concentration).values[0] / m as f64;
            value.is_finite().then_some(value)
        })();
        // The joint variance of the first-stage coefficients on the excluded instruments, the
        // regressors in order, under the variance asked for; `None` when it cannot be had.
        let absorbed = projection.map_or(0, |p| p.parameters(clusters));
        let fits: Vec<&Fitted> = stages.iter().map(|s| &s.fitted).collect();
        let bread = Ols.bread(&stages[0].fitted);
        let kept = first.kept.len();
        let joint = match variance::joint_variance(
            kind,
            &Ols,
            &stages[0].design,
            &fits,
            &bread,
            clusters,
            absorbed,
        ) {
            Ok(v) => Some(Matrix::from_fn(p * m, p * m, |a, b| {
                v.get(
                    (a / m) * kept + excluded[a % m],
                    (b / m) * kept + excluded[b % m],
                )
            })),
            Err(VarianceError::Asymmetric) => {
                return Err(internal(ctx, "an asymmetric joint first-stage variance"));
            }
            Err(_) => None,
        };
        let (kleibergen_paap, effective) = (|| {
            let none = (None, None);
            let Some(joint) = joint else {
                return none;
            };
            // Every instrument scaled by the root of its entry of the bread, and every
            // regressor by its residual standard deviation: no statistic depends on a unit, and
            // no matrix factorised here holds variables of very different sizes.
            let own_bread = block(&bread, &excluded);
            let by_instrument: Vec<f64> = (0..m).map(|i| own_bread.get(i, i)).collect();
            let by_regressor: Vec<f64> = (0..p)
                .map(|j| stages[j].fitted.residual_sum_of_squares / df2)
                .collect();
            if by_instrument
                .iter()
                .chain(&by_regressor)
                .any(|d| !(*d > 0.0 && d.is_finite()))
            {
                return none;
            }
            let by_instrument: Vec<f64> = by_instrument.iter().map(|d| 1.0 / d.sqrt()).collect();
            let by_regressor: Vec<f64> = by_regressor.iter().map(|d| 1.0 / d.sqrt()).collect();
            let scaled_bread = Matrix::from_fn(m, m, |i, j| {
                own_bread.get(i, j) * by_instrument[i] * by_instrument[j]
            });
            // (Z₂ᵀM Z₂), its root and the coefficients, all in the scaled units.
            let Some(cross) = symmetric_power(&scaled_bread, -1.0) else {
                return none;
            };
            let Some(root) = symmetric_power(&scaled_bread, -0.5) else {
                return none;
            };
            let coefficients = Matrix::from_fn(p, m, |j, e| {
                on_excluded[j][e] * by_instrument[e] * by_regressor[j]
            });
            let variance = Matrix::from_fn(p * m, p * m, |a, b| {
                joint.get(a, b)
                    * by_instrument[a % m]
                    * by_instrument[b % m]
                    * by_regressor[a / m]
                    * by_regressor[b / m]
            });
            // The regressors' cross-products with the other instruments projected out: what
            // the excluded instruments explain and what they leave.
            let explained = matmul(&matmul(&coefficients, &cross), &coefficients.transpose());
            let regressors = Matrix::from_fn(p, p, |j, l| {
                explained.get(j, l)
                    + dot(&stages[j].fitted.residuals, &stages[l].fitted.residuals)
                        * by_regressor[j]
                        * by_regressor[l]
            });
            let kleibergen_paap = (|| {
                let inverse_root = symmetric_power(&regressors, -0.5)?;
                // Θ = (XᵀX)^(-1/2) Πᵀ (ZᵀZ)^(1/2), p × m: its singular values are the
                // canonical correlations of the regressors and the instruments.
                let theta = matmul(&matmul(&inverse_root, &coefficients), &root);
                let left = symmetric_eigen(&matmul(&theta, &theta.transpose()));
                let right = symmetric_eigen(&transpose_matmul(&theta, &theta));
                // a: the left vector of the smallest singular value; the rows of B: the right
                // vectors of the m − p + 1 smallest, which span what the larger ones leave.
                let a: Vec<f64> = left.vectors.column(0).to_vec();
                let r = m - p + 1;
                let b = Matrix::from_fn(r, m, |i, e| right.vectors.get(e, i));
                let theta_a: Vec<f64> = (0..m)
                    .map(|e| (0..p).map(|j| theta.get(j, e) * a[j]).sum())
                    .collect();
                let lambda: Vec<f64> = (0..r)
                    .map(|i| (0..m).map(|e| b.get(i, e) * theta_a[e]).sum())
                    .collect();
                // λ = K vec(Π) with K = (aᵀG) ⊗ (B F), so its variance is K V Kᵀ.
                let ga: Vec<f64> = (0..p)
                    .map(|j| (0..p).map(|l| inverse_root.get(l, j) * a[l]).sum())
                    .collect();
                let bf = matmul(&b, &root);
                let k = Matrix::from_fn(r, p * m, |i, c| ga[c / m] * bf.get(i, c % m));
                let of_lambda = matmul(&matmul(&k, &variance), &k.transpose());
                quadratic_form(&of_lambda, &lambda).map(|rk| rk / m as f64)
            })();
            // The effective F, for one instrumented regressor under robust or clustered errors.
            let effective = (|| {
                if p != 1 || kind == Kind::Classical {
                    return None;
                }
                let w = matmul(&matmul(&root, &variance), &root);
                let trace: f64 = (0..m).map(|i| w.get(i, i)).sum();
                let squares = sum_of_squares(w.data());
                let largest = *symmetric_eigen(&w).values.last()?;
                let f = explained.get(0, 0) / trace;
                let x = 1.0 / EFFECTIVE_BIAS;
                let degrees =
                    trace * trace * (1.0 + 2.0 * x) / (squares + 2.0 * x * trace * largest);
                if !(f.is_finite() && degrees.is_finite() && degrees > 0.0) {
                    return None;
                }
                let critical =
                    noncentral_chi_square_point(degrees, x * degrees, EFFECTIVE_LEVEL) / degrees;
                Some(Effective {
                    f,
                    degrees,
                    critical,
                })
            })();
            (kleibergen_paap, effective)
        })();
        Ok(Weak {
            df1: m,
            df2,
            f,
            wald,
            cragg_donald,
            kleibergen_paap,
            effective,
            robust: kind != Kind::Classical,
        })
    }
}

/// First-stage residuals whose scaled covariance has an eigenvalue below this share of its
/// largest are collinear: the instrumented regressors differ by a combination of the
/// instruments, and the statistics that invert the covariance are not computed.
const COLLINEAR_RESIDUALS: f64 = 1e-10;

/// The columns of the structural design that least squares on it would drop: the regressors
/// collinear among themselves as observed.
fn collinear_as_observed(designs: &Designs) -> Vec<usize> {
    let k = designs.structural.k();
    let columns: Vec<Column> = (0..k).map(Column::Structural).collect();
    let pivoted = pivoted_qr(&factor(designs, &columns).r, k, PIVOT_TOLERANCE);
    let mut omitted = pivoted.order[pivoted.rank..k].to_vec();
    omitted.sort_unstable();
    omitted
}

fn listed(names: &[String]) -> String {
    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn counted(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The two stages on `designs`.
pub fn estimate(ctx: &mut CallContext<'_>, designs: Designs) -> Result<TwoStage, Fail> {
    let n = designs.structural.n();
    if n == 0 || designs.endogenous.is_empty() {
        // No row, which least squares refuses in its own words; or nothing instrumented, and
        // the model is least squares on the regressors.
        let fitted = Ols.fit(ctx, &designs.structural)?;
        let k = designs.structural.k();
        let columns: Vec<Vec<f64>> = (0..k)
            .map(|c| designs.structural.column_values(c))
            .collect();
        let second = designs
            .structural
            .with_columns(columns, designs.structural.leverage_available());
        return Ok(TwoStage {
            designs,
            first: None,
            second,
            fitted,
            notes: Vec::new(),
        });
    }
    let first = first_stage(ctx, &designs)?;
    // The fitted regressors: a regressor that is not instrumented keeps its own values; an
    // instrumented one is the kept instruments times its first-stage coefficients, summed in
    // the instruments' order.
    let k = designs.structural.k();
    let mut columns: Vec<Vec<f64>> = (0..k)
        .map(|c| {
            if designs.endogenous.contains(&c) {
                vec![0.0; n]
            } else {
                designs.structural.column_values(c)
            }
        })
        .collect();
    for (p, &i) in first.kept.iter().enumerate() {
        let instrument = designs.values(designs.instruments[i]);
        for (q, &c) in designs.endogenous.iter().enumerate() {
            let coefficient = first.coefficients[q][p];
            for (sum, z) in columns[c].iter_mut().zip(&instrument) {
                *sum += coefficient * z;
            }
        }
    }
    let second = designs
        .structural
        .with_columns(columns, designs.structural.leverage_available());
    let estimator = SecondStage {
        structural: &designs.structural,
        first_condition: first.condition_number,
    };
    let fitted = estimator.fit(ctx, &second)?;
    // What the first stage dropped among the excluded instruments.
    let names_of = |positions: &[usize]| -> Vec<String> {
        positions
            .iter()
            .filter(|&&i| matches!(designs.instruments[i], Column::Excluded(_)))
            .map(|&i| designs.name(designs.instruments[i]).to_string())
            .collect()
    };
    let mut notes: Vec<String> = Vec::new();
    let mut note = |names: Vec<String>, one: &str, several: &str| match names.len() {
        0 => {}
        1 => notes.push(format!("`{}` {one}", names[0])),
        _ => notes.push(format!("{} {several}", listed(&names))),
    };
    // An instrument the fixed effects absorbed is zeroed by the projection; it is named so, and
    // not as a zero column.
    let absorbed_by_effects = |i: &usize| match (&designs.effects, designs.instruments[*i]) {
        (Some((_, columns)), Column::Excluded(c)) => columns.contains(&c),
        _ => false,
    };
    let (absorbed, zeros): (Vec<usize>, Vec<usize>) =
        first.zero.iter().partition(|i| absorbed_by_effects(i));
    if let Some((effects_text, _)) = &designs.effects {
        note(
            names_of(&absorbed),
            &format!("is absorbed by fe: {effects_text} and was dropped from the instruments"),
            &format!("are absorbed by fe: {effects_text} and were dropped from the instruments"),
        );
    }
    note(
        names_of(&zeros),
        "is zero in every row used and was dropped from the instruments",
        "are zero in every row used and were dropped from the instruments",
    );
    note(
        names_of(&first.vanishing),
        "has values too small to be scaled (below 1e-150 in size) and was dropped from the instruments",
        "have values too small to be scaled (below 1e-150 in size) and were dropped from the instruments",
    );
    note(
        names_of(&first.oversized),
        "has values too large to be scaled (above 1e150 in size) and was dropped from the instruments",
        "have values too large to be scaled (above 1e150 in size) and were dropped from the instruments",
    );
    let collinear: Vec<usize> = first
        .omitted
        .iter()
        .copied()
        .filter(|c| {
            !first.zero.contains(c) && !first.vanishing.contains(c) && !first.oversized.contains(c)
        })
        .collect();
    note(
        names_of(&collinear),
        "is collinear with the regressors and instruments before it and was dropped from the instruments",
        "are collinear with the regressors and instruments before them and were dropped from the instruments",
    );
    // A fitted regressor collinear with the ones before it where the regressor as observed is
    // not: the instruments do not identify the model (section 7.1; Z0804), by their count once
    // the collinear ones are gone, or because they move the instrumented regressors only
    // together.
    if !fitted.omitted.is_empty() {
        let observed = collinear_as_observed(&designs);
        let unidentified: Vec<String> = fitted
            .omitted
            .iter()
            .filter(|c| !observed.contains(c))
            .map(|&c| designs.structural.names[c].clone())
            .collect();
        if !unidentified.is_empty() {
            let excluded = designs.excluded_instruments();
            let remaining = excluded.iter().filter(|i| first.kept.contains(i)).count();
            let instrumented = designs
                .endogenous
                .iter()
                .filter(|c| !observed.contains(c))
                .count();
            let dropped = excluded.len() - remaining;
            let mut d = if remaining < instrumented {
                Diagnostic::error(
                    "Z0804",
                    format!(
                        "the model is under-identified: {} for {}",
                        if dropped == 0 {
                            counted(remaining, "excluded instrument", "excluded instruments")
                        } else {
                            counted(
                                remaining,
                                "excluded instrument remains",
                                "excluded instruments remain",
                            )
                        },
                        counted(
                            instrumented,
                            "instrumented regressor",
                            "instrumented regressors"
                        )
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "fewer instruments than instrumented regressors")
                .help("each instrumented regressor needs at least one instrument that is not a regressor of the formula")
            } else {
                Diagnostic::error(
                    "Z0804",
                    format!(
                        "the model is under-identified: the instruments do not tell {} from the regressors before {}",
                        listed(&unidentified),
                        if unidentified.len() == 1 { "it" } else { "them" }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "instruments that move the regressors only together")
                .help("the first stage's fitted values are collinear: each instrumented regressor needs an instrument that moves it apart from the others")
            };
            for text in &notes {
                d = d.help(text.clone());
            }
            if !designs.included.is_empty() {
                d = d.help(format!(
                    "{} already in the formula, so {} an included instrument and identifies nothing",
                    match designs.included.as_slice() {
                        [one] => format!("`{one}` is"),
                        many => format!("{} are", listed(many)),
                    },
                    if designs.included.len() == 1 {
                        "it is"
                    } else {
                        "each is"
                    }
                ));
            }
            return Err(fail(d));
        }
    }
    Ok(TwoStage {
        designs,
        first: Some(first),
        second,
        fitted,
        notes,
    })
}
