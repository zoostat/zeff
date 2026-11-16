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
use crate::reg::formula_variables;
use std::collections::BTreeSet;
use zeff_core::runtime::{CallContext, Fail, fail};
use zeff_core::value::{Formula, Frame, Term, TermKind};
use zeff_diagnostics::Diagnostic;
use zeff_kernels::qr::{TallSkinnyQr, pivoted_qr};
use zeff_kernels::svd::condition_number;
use zeff_kernels::triangular::solve_upper;
use zeff_kernels::{LaneSum, Matrix, dot, sum_of_squares};
use zeff_numeric::order::WeightKind;
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
        let mut structural = Design::build(ctx, frame, formula, &structural_extra)?;
        let instruments_formula = Formula {
            outcome: None,
            terms: excluded.to_vec(),
            intercept: formula.intercept,
        };
        let mut instruments = Design::build(ctx, frame, &instruments_formula, &excluded_extra)?;
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
        Ok(Designs {
            structural,
            excluded: instruments,
            endogenous: instrumented,
            instruments: listed,
            included,
        })
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
        let second = designs.structural.with_columns(columns, true);
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
    let second = designs.structural.with_columns(columns, true);
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
    note(
        names_of(&first.zero),
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
