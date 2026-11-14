//! `anova`, the factorial form (release 0.7.0, step 17; Language Reference, sections 4, 6 and
//! 10.2; open decisions #91, rule R9, and #98): the table of sums of squares of a linear model
//! written as a formula, `data.anova(y ~ a * b + x, type: 2)`. The terms may be categories,
//! numeric covariates and their interactions. `type:` chooses how each term is adjusted:
//!
//! - type 1, sequential, as R's `anova(lm(...))` and Stata's `anova, sequential`: each term
//!   adjusted for the terms above it;
//! - type 2, the default, as `car::Anova(type = 2)`: each term adjusted for every other term
//!   that does not contain it;
//! - type 3, as `car::Anova(type = 3)` with sum contrasts and Stata's `anova`: each term
//!   adjusted for every other term.
//!
//! Every category is coded by deviations from the mean over the levels the rows used hold,
//! whatever contrast it carries, which is the coding under which type 3 tests a term averaged
//! over the levels of the others; types 1 and 2 do not depend on the coding. The model is
//! factorised once, by the blocked QR of `[X y]` with X scaled to unit column norms, as the
//! least-squares fit of `reg`; each table entry is then read from a pivoted QR of that small
//! triangular factor with its columns reordered, so no pass over the rows is repeated and no
//! cross-product matrix is formed. Frequency and analytic weights give the weighted
//! least-squares table; sampling weights are refused (Z0411).
//!
//! `effect:` (step 17b; open decision #99, question 2) puts partial omega squared or partial
//! epsilon squared in the place of partial eta squared: both take the residual mean square off
//! the numerator, so both are negative for a term whose F is below 1, which is printed as it is
//! with a note.

use std::rc::Rc;
use zeff_core::display::{fixed, p_value_text};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Estimate, FollowUp, Formula, Frame, ModelFit, Statistic, Table, TermTest,
    TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::qr::{PivotedQr, TallSkinnyQr, pivoted_qr};
use zeff_kernels::{LaneSum, Matrix, sum_of_squares};
use zeff_numeric::order::WeightKind;
use zeff_results::design::{Design, DesignTerm};

use crate::anova::{
    EQUAL_VAR, Effect, GROUP, LEVEL, TYPE, VAR, WEIGHT_TYPE, WEIGHTS, count_text, effect,
    upper_tail,
};
use crate::testing;

/// The collinearity rule's relative tolerance, the one of the least-squares fit of `reg` and of
/// R's `lm`.
pub const PIVOT_TOLERANCE: f64 = 1e-7;

/// Above this size a value's square cannot be represented.
use crate::testing::OVERSIZED;

/// R's rule for an essentially perfect fit (`summary.lm`): a residual mean square below this
/// share of the mean square of the fitted values is rounding and nothing more.
const PERFECT: f64 = 1e-30;

/// How each term is adjusted for the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Type 1: for the terms before it.
    Sequential,
    /// Type 2: for every other term that does not contain it.
    Marginal,
    /// Type 3: for every other term.
    Partial,
}

impl Kind {
    /// The number the argument and the title write.
    pub fn number(self) -> u8 {
        match self {
            Kind::Sequential => 1,
            Kind::Marginal => 2,
            Kind::Partial => 3,
        }
    }
}

/// One row of the table: a term, or the whole model. A term without degrees of freedom has
/// not-a-number for F, p and the three effect sizes. With MS for the residual mean square and
/// N for the observations, partial eta squared is SS / (SS + SS residual), partial omega
/// squared (SS − df × MS) / (SS + (N − df) × MS) and partial epsilon squared (SS − df × MS) /
/// (SS + SS residual); on the row of the whole model the first is R² and the last the adjusted
/// R².
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub label: String,
    pub df: usize,
    pub ss: f64,
    pub f: f64,
    pub p: f64,
    pub partial_eta_squared: f64,
    pub partial_omega_squared: f64,
    /// Partial omega squared from F alone (open decision #101): with r for the residual degrees
    /// of freedom, (F − 1) / (F + (r + 1) / df), which is (SS − df × MS) / (SS + (r + 1) × MS).
    /// It is the ruled figure where the model has one term, and the larger in size otherwise.
    pub partial_omega_squared_f: f64,
    pub partial_epsilon_squared: f64,
}

impl Row {
    /// The partial effect size of `which`.
    pub fn effect(&self, which: Effect) -> f64 {
        match which {
            Effect::Eta => self.partial_eta_squared,
            Effect::Omega => self.partial_omega_squared,
            Effect::OmegaF => self.partial_omega_squared_f,
            Effect::Epsilon => self.partial_epsilon_squared,
        }
    }
}

/// The table: the whole model, each term in the order of its degree (main effects, then
/// two-way interactions, and so on, the written order kept within a degree, as R orders the
/// terms of a model), the residual and the corrected total.
#[derive(Debug, Clone, PartialEq)]
pub struct Factorial {
    pub model: Row,
    pub terms: Vec<Row>,
    pub residual_df: f64,
    pub residual_ss: f64,
    pub total_ss: f64,
    /// The columns of the design that carry an estimate, the intercept counted, and all its
    /// columns: fewer of the first when a combination of levels has no observation.
    pub rank: usize,
    pub columns: usize,
    /// The fit itself, for the call that follows (`posthoc`, step 19b).
    pub solution: Solution,
}

/// The least-squares fit as its triangular factor: with the columns of the scaled design taken
/// in the order of `order` (the design's own indices, the ones estimated first), `triangle`
/// holds the first `rank` rows of the R factor, row by row over all the columns, and `effects`
/// the entries of Qᵀy that go with those rows. The coefficients are never formed: a linear
/// function of them is read from the two (see `zeff_core::value::ModelFit`).
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub order: Vec<usize>,
    pub rank: usize,
    pub triangle: Vec<f64>,
    pub effects: Vec<f64>,
}

/// Why the table cannot be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undefined {
    /// No column but the intercept varies among the rows used.
    NoModel,
    /// As many estimated columns as observations, or more.
    NoResidualFreedom { rank: usize },
    /// The model reproduces the outcome to rounding, so no F is defined.
    PerfectFit,
    /// Type 3 with columns that are combinations of others (a cell with no observations).
    Aliased { rank: usize, columns: usize },
}

/// What one pivoted QR of the reordered factor gives: for each group of columns the sum of
/// squares it adds to the groups before it and the degrees of freedom it adds, with Qᵀy whole.
struct Sequence {
    ss: Vec<f64>,
    df: Vec<usize>,
    /// Qᵀy: the first `rank` entries belong to the accepted columns in order, the rest are the
    /// residual.
    effects: Vec<f64>,
    rank: usize,
    /// The factor the figures were read from, and the column of X at each of its positions
    /// before the pivoting.
    pivoted: PivotedQr,
    layout: Vec<usize>,
}

impl Sequence {
    /// The fit of the columns laid out, as the call that follows the table reads it.
    fn solution(&self) -> Solution {
        let columns = self.layout.len();
        let mut triangle = Vec::with_capacity(self.rank * columns);
        for row in 0..self.rank {
            for position in 0..columns {
                triangle.push(self.pivoted.r.column(position)[row]);
            }
        }
        Solution {
            order: self.pivoted.order[..columns]
                .iter()
                .map(|&position| self.layout[position])
                .collect(),
            rank: self.rank,
            triangle,
            effects: self.effects[..self.rank].to_vec(),
        }
    }
}

/// The sequential sums of squares of `groups` of columns of X, in that order, from `r`, the
/// triangular factor of `[X y]`: the factor's columns are laid out group after group with y
/// last and factorised again with the limited pivoting of R's `dqrdc2`, so that a column that
/// is a combination of the ones before it is left out; the squares of Qᵀy at the positions a
/// group's accepted columns hold add up to the group's sum of squares.
fn sequence(r: &Matrix, groups: &[Vec<usize>]) -> Sequence {
    let y = r.cols() - 1;
    let included: usize = groups.iter().map(Vec::len).sum();
    let mut reordered = Matrix::zeros(r.rows(), included + 1);
    let mut owner: Vec<usize> = Vec::with_capacity(included);
    let mut layout: Vec<usize> = Vec::with_capacity(included);
    for (g, group) in groups.iter().enumerate() {
        for &column in group {
            reordered
                .column_mut(owner.len())
                .copy_from_slice(r.column(column));
            owner.push(g);
            layout.push(column);
        }
    }
    reordered.column_mut(included).copy_from_slice(r.column(y));
    let pivoted = pivoted_qr(&reordered, included, PIVOT_TOLERANCE);
    let effects: Vec<f64> = pivoted.r.column(included).to_vec();
    let mut squares: Vec<Vec<f64>> = vec![Vec::new(); groups.len()];
    for (position, &effect) in effects.iter().enumerate().take(pivoted.rank) {
        squares[owner[pivoted.order[position]]].push(effect);
    }
    Sequence {
        ss: squares.iter().map(|s| sum_of_squares(s)).collect(),
        df: squares.iter().map(Vec::len).collect(),
        effects,
        rank: pivoted.rank,
        pivoted,
        layout,
    }
}

/// The table of sums of squares of the model whose `[X y]` has the triangular factor `r`: the
/// intercept is column 0, the columns of each term are `terms[i].columns`, y is the last
/// column; `n` is the number of observations (the weight sum under frequency weights).
pub fn factorial(
    r: &Matrix,
    terms: &[DesignTerm],
    n: f64,
    kind: Kind,
) -> Result<Factorial, Undefined> {
    let columns = r.cols() - 1;
    let mut ordered: Vec<&DesignTerm> = terms.iter().collect();
    ordered.sort_by_key(|t| t.parts.len());
    let mut groups: Vec<Vec<usize>> = vec![vec![0]];
    groups.extend(ordered.iter().map(|t| t.columns.clone().collect()));
    let full = sequence(r, &groups);
    let rank = full.rank;
    if rank <= 1 {
        return Err(Undefined::NoModel);
    }
    let residual_df = n - rank as f64;
    if residual_df < 1.0 {
        return Err(Undefined::NoResidualFreedom { rank });
    }
    let residual_ss = sum_of_squares(&full.effects[rank..]);
    let fitted_ss = sum_of_squares(&full.effects[..rank]);
    let residual_ms = residual_ss / residual_df;
    if residual_ss.is_nan() || residual_ss <= 0.0 || residual_ms < fitted_ss / n * PERFECT {
        return Err(Undefined::PerfectFit);
    }
    if kind == Kind::Partial && rank < columns {
        return Err(Undefined::Aliased { rank, columns });
    }
    let row = |label: &str, df: usize, ss: f64| {
        if df == 0 {
            return Row {
                label: label.to_string(),
                df,
                ss: 0.0,
                f: f64::NAN,
                p: f64::NAN,
                partial_eta_squared: f64::NAN,
                partial_omega_squared: f64::NAN,
                partial_omega_squared_f: f64::NAN,
                partial_epsilon_squared: f64::NAN,
            };
        }
        let f = (ss / df as f64) / residual_ms;
        let corrected = ss - df as f64 * residual_ms;
        Row {
            label: label.to_string(),
            df,
            ss,
            f,
            p: upper_tail(f, df as f64, residual_df),
            partial_eta_squared: ss / (ss + residual_ss),
            partial_omega_squared: corrected / (ss + (n - df as f64) * residual_ms),
            partial_omega_squared_f: corrected / (ss + (residual_df + 1.0) * residual_ms),
            partial_epsilon_squared: corrected / (ss + residual_ss),
        }
    };
    let rows: Vec<Row> = ordered
        .iter()
        .enumerate()
        .map(|(i, term)| {
            let (ss, df) = match kind {
                Kind::Sequential => (full.ss[i + 1], full.df[i + 1]),
                Kind::Marginal | Kind::Partial => {
                    // The columns the term is adjusted for, in the order of the table: under
                    // type 2 a term that contains this one (`a:b` for `a`) is left out of the
                    // model altogether; under type 3 every other column stays.
                    let mut before: Vec<usize> = vec![0];
                    for (j, other) in ordered.iter().enumerate() {
                        if j == i {
                            continue;
                        }
                        if kind == Kind::Marginal && term.parts.is_subset(&other.parts) {
                            continue;
                        }
                        before.extend(other.columns.clone());
                    }
                    let own: Vec<usize> = term.columns.clone().collect();
                    let adjusted = sequence(r, &[before, own]);
                    (adjusted.ss[1], adjusted.df[1])
                }
            };
            row(&term.label, df, ss)
        })
        .collect();
    let model = row("model", rank - 1, sum_of_squares(&full.effects[1..rank]));
    Ok(Factorial {
        model,
        terms: rows,
        residual_df,
        residual_ss,
        total_ss: sum_of_squares(&full.effects[1..]),
        rank,
        columns,
        solution: full.solution(),
    })
}

/// The triangular factor of `[X y]` of `design`, X scaled column by column to unit norm as the
/// least-squares fit scales it, by the blocked QR; with it the name of a column too large to
/// square, when there is one, the largest outcome in size, and the scale of each column.
fn factor(design: &Design) -> (Matrix, Option<String>, f64, Vec<f64>) {
    let k = design.k();
    let mut accumulators: Vec<LaneSum> = (0..k).map(|_| LaneSum::new()).collect();
    let mut largest_outcome = 0.0_f64;
    for block in design.blocks() {
        for (c, accumulator) in accumulators.iter_mut().enumerate() {
            accumulator.push_squares(block.x.column(c));
        }
        largest_outcome = block.y.iter().fold(largest_outcome, |m, y| m.max(y.abs()));
    }
    let sums: Vec<f64> = accumulators.iter().map(LaneSum::value).collect();
    let oversized = sums
        .iter()
        .position(|s| !s.is_finite())
        .map(|c| design.names[c].clone());
    let scales: Vec<f64> = sums
        .iter()
        .map(|s| {
            if *s > 0.0 && s.is_finite() {
                1.0 / s.sqrt()
            } else {
                1.0
            }
        })
        .collect();
    let mut qr = TallSkinnyQr::new(k + 1);
    if oversized.is_none() && largest_outcome <= OVERSIZED {
        for block in design.blocks() {
            let mut augmented = Matrix::zeros(block.x.rows(), k + 1);
            for (c, &scale) in scales.iter().enumerate() {
                let column = augmented.column_mut(c);
                column.copy_from_slice(block.x.column(c));
                for v in column.iter_mut() {
                    *v *= scale;
                }
            }
            augmented.column_mut(k).copy_from_slice(&block.y);
            qr.push_rows(&augmented);
        }
    }
    (qr.finish(), oversized, largest_outcome, scales)
}

/// The observations a design counts: the total of its frequency weights, and its rows otherwise.
fn observations(design: &Design) -> f64 {
    match design.weight_kind {
        Some(WeightKind::Frequency) => design.weight_total(),
        _ => design.n() as f64,
    }
}

/// The table of a design whose categories are already coded by deviations (`sum_coded`): the
/// figures `anova` prints, before any rounding, which a test compares with R's one by one.
pub fn table_of(design: &Design, kind: Kind) -> Result<Factorial, Undefined> {
    let (r, _, _, _) = factor(design);
    factorial(&r, &design.terms(), observations(design), kind)
}

/// An argument of the one-way form given with a formula, or the other way round.
fn misplaced(ctx: &CallContext<'_>, slot: usize, message: &str, label: &str, help: &str) -> Fail {
    fail(
        Diagnostic::error("Z0401", message)
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), label)
            .help(help),
    )
}

/// `type:` as a kind: 2 unless given; anything but 1, 2 or 3 is Z0401.
pub fn kind(ctx: &CallContext<'_>, args: &Args) -> Result<Kind, Fail> {
    match args.get(TYPE) {
        None | Some(Value::Int(2)) => Ok(Kind::Marginal),
        Some(Value::Int(1)) => Ok(Kind::Sequential),
        Some(Value::Int(3)) => Ok(Kind::Partial),
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`type:` of `anova` must be 1, 2 or 3, not {}",
                    match other {
                        Value::Int(k) => k.to_string(),
                        other => format!("a value of type {}", other.type_name()),
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(TYPE), "not a type of sums of squares")
            .help("`type: 1` adjusts each term for the terms above it, `type: 2` (the default) for every other term that does not contain it, `type: 3` for every other term"),
        )),
    }
}

/// `3 rows`, `1 row`.
fn rows_text(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// `a`, `a and b`, `a, b and c`.
fn listed(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// `data.anova(y ~ a * b, type:, effect:, weights:, weight_type:)`, the factorial form.
pub fn anova(
    ctx: &mut CallContext<'_>,
    args: &Args,
    frame: &Frame,
    formula: &Formula,
) -> Result<Value, Fail> {
    if args.get(GROUP).is_some() {
        return Err(misplaced(
            ctx,
            GROUP,
            "`group:` belongs to the one-way form of `anova`; with a formula the groups are its terms",
            "not used with a formula",
            "write the grouping variable as a term, `anova(y ~ g)`; or name the variable alone, `anova(y, group: g)`, for the one-way test",
        ));
    }
    if args.get(EQUAL_VAR).is_some() {
        return Err(misplaced(
            ctx,
            EQUAL_VAR,
            "`equal_var:` belongs to the one-way form of `anova`; the table of a formula always pools the variance",
            "not used with a formula",
            "leave `equal_var:` out; Welch's F, which does not pool, is `anova(y, group: g)`",
        ));
    }
    let kind = kind(ctx, args)?;
    let which = effect(ctx, args)?;
    let measure = format!("partial {}", which.label());
    let definition = match which {
        Effect::Eta => "SS / (SS + SS residual)",
        Effect::Omega => "(SS − df × MS residual) / (SS + (N − df) × MS residual)",
        Effect::OmegaF => "(F − 1) / (F + (df residual + 1) / df)",
        Effect::Epsilon => "(SS − df × MS residual) / (SS + SS residual)",
    };
    let Some(outcome) = &formula.outcome else {
        return Err(misplaced(
            ctx,
            VAR,
            "the formula of `anova` has no outcome",
            "nothing on the left of `~`",
            "write the variable whose means are compared on the left, `anova(y ~ a * b)`",
        ));
    };
    let outcome = outcome.text.clone();
    if !formula.intercept {
        return Err(misplaced(
            ctx,
            VAR,
            "`anova` adjusts every term for the grand mean, but the formula removes the intercept",
            "a model without an intercept",
            "leave `- 1` out of the formula; `reg` fits a model without an intercept",
        ));
    }
    if formula.terms.is_empty() {
        return Err(misplaced(
            ctx,
            VAR,
            "the formula of `anova` has no term to test",
            "an intercept alone",
            "write the terms on the right of `~`, `anova(y ~ a * b)`; a test of one mean against a value is `t_test` with `mu:`",
        ));
    }
    // The weights are checked as every test checks them (sampling weights refused, Z0411, and
    // the same messages for a weight that is not a number, negative or a fractional count),
    // then taken by the design.
    let weights = match args.get(WEIGHTS) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weight_kind = match args.get(WEIGHT_TYPE) {
        Some(Value::Word(word)) if word == "frequency" => WeightKind::Frequency,
        _ => WeightKind::Analytic,
    };
    if weights.is_some() {
        testing::weights(ctx, frame, args, WEIGHTS, WEIGHT_TYPE, "anova")?;
    }
    let extra: Vec<&str> = weights.iter().map(String::as_str).collect();
    let mut design = Design::build(ctx, frame, formula, &extra)?;
    if let Some(name) = &weights {
        design = design.weighted(ctx, frame, name, weight_kind)?;
    }
    for note in &design.notes {
        ctx.message(Diagnostic::plain_note(note.clone()));
    }
    let mut dropped_parts: Vec<String> = Vec::new();
    if design.dropped > 0 {
        dropped_parts.push(format!(
            "{} with NA in a model variable dropped",
            rows_text(design.dropped)
        ));
    }
    if design.zero_weights > 0 {
        dropped_parts.push(format!(
            "{} with zero weight left out",
            rows_text(design.zero_weights)
        ));
    }
    if !dropped_parts.is_empty() {
        ctx.message(Diagnostic::plain_note(dropped_parts.join("; ")));
    }
    if design.n() == 0 {
        let (message, help) = if design.dropped == 0 && design.zero_weights == 0 {
            (
                "no row remains for `anova`: the frame has no rows",
                "a model needs rows; the frame's own steps left none",
            )
        } else {
            (
                "no row remains for `anova`: every row has NA in a model variable or a zero weight",
                "`describe` shows each variable's missing count; `where(!is_missing(x))` or `recode` fills the gaps",
            )
        };
        return Err(fail(
            Diagnostic::error("Z0801", message)
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing to estimate")
                .help(help),
        ));
    }
    let (design, single) = design.sum_coded();
    if let Some(var) = single.first() {
        return Err(misplaced(
            ctx,
            VAR,
            &format!(
                "`{var}` has one level among the rows used, so `anova` has no groups of it to compare"
            ),
            "a category of one level",
            "a term needs two levels or more among the rows kept; leave the variable out of the formula, or keep more rows",
        ));
    }
    let terms = design.terms();
    let values = design.outcome().unwrap_or_default();
    let constant = values.windows(2).all(|pair| pair[0] == pair[1]);
    if constant {
        return Err(misplaced(
            ctx,
            VAR,
            &format!(
                "`{outcome}` has the same value in every row used, so there is no variation to divide among the terms"
            ),
            "an outcome that does not vary",
            "an analysis of variance needs an outcome that varies",
        ));
    }
    let (r, oversized, largest_outcome, scales) = factor(&design);
    if largest_outcome > OVERSIZED {
        return Err(fail(
            Diagnostic::error(
                "Z0803",
                format!("`{outcome}` has values too large to compute with (above 1e150 in size)"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "an outcome too large to square")
            .help(format!(
                "divide the outcome by a power of ten first, `add({outcome}_m = {outcome} / 1e6)`, and read the sums of squares in those units"
            )),
        ));
    }
    if let Some(column) = oversized {
        return Err(fail(
            Diagnostic::error(
                "Z0803",
                format!("`{column}` has values too large to compute with (above 1e150 in size)"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "a term too large to square")
            .help("divide the variable by a power of ten first; no test of the model changes"),
        ));
    }
    let n = observations(&design);
    // What leaves a column without a degree of freedom: an empty cell where two categories
    // meet, or a term that the others already give.
    let cells = terms.iter().any(|t| t.categories > 1);
    let why_aliased = if cells {
        "a combination of levels with no observations, or a term that repeats the others"
    } else {
        "a term that repeats the others"
    };
    let instead = if cells {
        "`type: 2` tests each term whatever cells are empty; `tabulate` shows the cells of two categories"
    } else {
        "leave the term that repeats the others out of the formula"
    };
    let table = factorial(&r, &terms, n, kind).map_err(|why| {
        let diagnostic = match why {
            Undefined::NoModel => Diagnostic::error(
                "Z0802",
                "no term of the formula varies among the rows used, so `anova` has nothing to test",
            )
            .primary(ctx.span_of(VAR), "nothing to estimate")
            .help("a term needs values that differ among the rows kept"),
            Undefined::NoResidualFreedom { rank } => Diagnostic::error(
                "Z0802",
                format!(
                    "no residual degrees of freedom: {} observations and {} estimated columns, the intercept counted",
                    count_text(n),
                    count(rank as u64)
                ),
            )
            .primary(ctx.span_of(VAR), "as many columns as observations, or more")
            .help("F divides by the residual mean square, which needs more observations than the model has columns; an interaction of two categories needs a cell with two observations or more, or drop the interaction"),
            Undefined::PerfectFit => Diagnostic::error(
                "Z0802",
                format!("the model reproduces `{outcome}` exactly, so the residual sum of squares is zero and no F is defined"),
            )
            .primary(ctx.span_of(VAR), "an essentially perfect fit")
            .help("an analysis of variance needs variation the terms do not explain; check that no term is the outcome under another name"),
            Undefined::Aliased { rank, columns } => Diagnostic::error(
                "Z0401",
                format!(
                    "`type: 3` cannot adjust each term for every other: {} of the model's {} columns {} determined by the others",
                    count((columns - rank) as u64),
                    count(columns as u64),
                    if columns - rank == 1 { "is" } else { "are" }
                ),
            )
            .primary(ctx.span_of(TYPE), why_aliased)
            .help(instead),
        };
        fail(diagnostic.in_file(ctx.script))
    })?;
    if table.rank < table.columns {
        let aliased = table.columns - table.rank;
        ctx.message(Diagnostic::plain_note(format!(
            "{} of the model's {} columns {} determined by the others ({why_aliased}) and {} no degree of freedom",
            count(aliased as u64),
            count(table.columns as u64),
            if aliased == 1 { "is" } else { "are" },
            if aliased == 1 { "carries" } else { "carry" }
        )));
    }
    for row in table.terms.iter().filter(|row| row.df == 0) {
        ctx.message(Diagnostic::plain_note(format!(
            "`{}` has no degrees of freedom once the other terms are in the model, so it has no test",
            row.label
        )));
    }
    if terms.iter().all(|t| t.categories == 0) {
        ctx.message(Diagnostic::plain_note(
            "no term of the formula is a category, so each row tests a slope; `reg` gives the estimates",
        ));
    }
    // Partial omega and epsilon squared are negative on a row whose F is below 1: printed as
    // they are (open decision #99, the ruling on negative values), with one note naming the rows.
    let mut negative: Vec<String> = Vec::new();
    if table.model.effect(which) < 0.0 {
        negative.push("the model".to_string());
    }
    negative.extend(
        table
            .terms
            .iter()
            .filter(|row| row.df > 0 && row.effect(which) < 0.0)
            .map(|row| format!("`{}`", row.label)),
    );
    if !negative.is_empty() {
        ctx.message(Diagnostic::plain_note(format!(
            "F is below 1 for {}, so {} {measure} is negative; read {} as no evidence of an effect",
            listed(&negative),
            if negative.len() == 1 { "its" } else { "their" },
            if negative.len() == 1 { "it" } else { "each" }
        )));
    }
    let residual_ms = table.residual_ss / table.residual_df;
    let line = |row: &Row| -> Vec<String> {
        if row.df == 0 {
            return vec![
                row.label.clone(),
                "0".to_string(),
                fixed(0.0, 3),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ];
        }
        vec![
            row.label.clone(),
            count(row.df as u64),
            fixed(row.ss, 3),
            fixed(row.ss / row.df as f64, 3),
            fixed(row.f, 2),
            p_value_text(row.p),
            fixed(row.effect(which), 3),
        ]
    };
    let mut rows: Vec<Vec<String>> = vec![line(&table.model)];
    rows.extend(table.terms.iter().map(line));
    rows.push(vec![
        "residual".to_string(),
        count_text(table.residual_df),
        fixed(table.residual_ss, 3),
        fixed(residual_ms, 3),
        String::new(),
        String::new(),
        String::new(),
    ]);
    let total_at = rows.len();
    rows.push(vec![
        "total".to_string(),
        count_text(n - 1.0),
        fixed(table.total_ss, 3),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ]);
    let statistic = |row: &Row| Statistic {
        name: "F".to_string(),
        value: row.f,
        df: vec![row.df as f64, table.residual_df],
    };
    let how = match kind {
        Kind::Sequential => {
            "type 1 sums of squares: each term adjusted for the terms above it, so the order of the terms matters"
        }
        Kind::Marginal => {
            "type 2 sums of squares: each term adjusted for every other term that does not contain it"
        }
        Kind::Partial => {
            "type 3 sums of squares: each term adjusted for every other term, the categories coded by deviations from the mean"
        }
    };
    let weighting = match (&weights, design.weight_kind) {
        (Some(name), Some(WeightKind::Frequency)) => format!(", frequency weights {name}"),
        (Some(name), Some(_)) => format!(", analytic weights {name}"),
        _ => String::new(),
    };
    // What `posthoc` reads: the coding of the columns and the fit (step 19b).
    let coding = design.coding(ctx, frame, formula);
    let fit = ModelFit {
        outcome: outcome.clone(),
        formula: formula.to_string(),
        factors: coding.factors,
        numbers: coding.numbers,
        terms: coding.terms,
        columns: coding.columns,
        scales,
        order: table.solution.order.clone(),
        rank: table.solution.rank,
        triangle: table.solution.triangle.clone(),
        effects: table.solution.effects.clone(),
        residual_df: table.residual_df,
        residual_ms,
        n,
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: Some(Rc::new(FollowUp::Factorial(fit))),
        terms: table
            .terms
            .iter()
            .map(|row| TermTest {
                term: row.label.clone(),
                statistic: statistic(row),
                p_value: row.p,
                effect: Some(Estimate {
                    label: measure.clone(),
                    value: row.effect(which),
                    ci: None,
                    decimals: 3,
                }),
                estimate: None,
                std_error: None,
            })
            .collect(),
        method: "anova".to_string(),
        title: format!("anova · factorial, type {} · {formula}", kind.number()),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: vec![
            "each row tests H0: the population effect of the term is zero, against H1: it is not"
                .to_string(),
        ],
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("source".to_string(), Align::Left),
                ("df".to_string(), Align::Right),
                ("SS".to_string(), Align::Right),
                ("MS".to_string(), Align::Right),
                ("F".to_string(), Align::Right),
                ("p value".to_string(), Align::Right),
                (measure.clone(), Align::Right),
            ],
            rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![total_at],
        }),
        estimate: None,
        statistic: statistic(&table.model),
        p_value: table.model.p,
        effect: None,
        level: LEVEL,
        alpha: None,
        after: vec![
            how.to_string(),
            format!("{measure} = {definition}; N = {}{weighting}", count_text(n)),
        ],
        notes: Vec::new(),
        n: n.round() as usize,
        dropped: design.dropped,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The factor of `[X y]` for a small design given by its columns, the intercept first.
    fn factor_of(columns: &[Vec<f64>], y: &[f64]) -> Matrix {
        let k = columns.len();
        let mut qr = TallSkinnyQr::new(k + 1);
        let scales: Vec<f64> = columns
            .iter()
            .map(|c| 1.0 / sum_of_squares(c).sqrt())
            .collect();
        for (i, &yi) in y.iter().enumerate() {
            let mut row: Vec<f64> = columns.iter().zip(&scales).map(|(c, s)| c[i] * s).collect();
            row.push(yi);
            qr.push_row(&row);
        }
        qr.finish()
    }

    fn term(label: &str, parts: &[&str], columns: std::ops::Range<usize>) -> DesignTerm {
        DesignTerm {
            label: label.to_string(),
            parts: parts.iter().map(|p| p.to_string()).collect::<BTreeSet<_>>(),
            columns,
            categories: parts.len(),
        }
    }

    /// A 2 × 2 design, sum coded, with unequal cells: 3, 2, 2 and 1 observations.
    fn two_by_two() -> (Matrix, Vec<DesignTerm>) {
        let a = [1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
        let b = [1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0];
        let y = [3.1, 2.9, 3.6, 4.4, 4.9, 5.2, 6.1, 9.0];
        let ab: Vec<f64> = a.iter().zip(&b).map(|(a, b)| a * b).collect();
        let r = factor_of(&[vec![1.0; 8], a.to_vec(), b.to_vec(), ab], &y);
        let terms = vec![
            term("a", &["category:a"], 1..2),
            term("b", &["category:b"], 2..3),
            term("a:b", &["category:a", "category:b"], 3..4),
        ];
        (r, terms)
    }

    #[test]
    fn the_three_types_match_r_on_an_unbalanced_two_by_two() {
        // R 4.6.0, car 3.1-5: anova(lm(y ~ a * b)), Anova(type = 2) and Anova(type = 3) with
        // contr.sum (scratch check of step 17; the reference run is tests/reference/r).
        let (r, terms) = two_by_two();
        let close = |x: f64, want: f64| (x - want).abs() <= 1e-10 * want.abs().max(1.0);
        let one = factorial(&r, &terms, 8.0, Kind::Sequential).expect("a table");
        let two = factorial(&r, &terms, 8.0, Kind::Marginal).expect("a table");
        let three = factorial(&r, &terms, 8.0, Kind::Partial).expect("a table");
        // The sequential sums of squares add up to the model's.
        let added: f64 = one.terms.iter().map(|t| t.ss).sum();
        assert!(close(added, one.model.ss));
        assert!(close(one.model.ss + one.residual_ss, one.total_ss));
        // The last term of the sequence is adjusted for every other under each type.
        assert!(close(one.terms[2].ss, two.terms[2].ss));
        assert!(close(one.terms[2].ss, three.terms[2].ss));
        // Type 2 of `b` is its sequential value after `a`.
        assert!(close(one.terms[1].ss, two.terms[1].ss));
        // The residual and the model are the same whatever the type.
        assert!(close(one.residual_ss, three.residual_ss));
        assert_eq!(one.residual_df, 4.0);
        assert_eq!(three.model.df, 3);
    }

    #[test]
    fn a_balanced_design_gives_one_table_under_every_type() {
        let a = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0];
        let b = [1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0];
        let y = [3.1, 2.9, 4.4, 4.9, 5.2, 6.1, 9.0, 8.2];
        let ab: Vec<f64> = a.iter().zip(&b).map(|(a, b)| a * b).collect();
        let r = factor_of(&[vec![1.0; 8], a.to_vec(), b.to_vec(), ab], &y);
        let terms = vec![
            term("a", &["category:a"], 1..2),
            term("b", &["category:b"], 2..3),
            term("a:b", &["category:a", "category:b"], 3..4),
        ];
        let one = factorial(&r, &terms, 8.0, Kind::Sequential).expect("a table");
        let two = factorial(&r, &terms, 8.0, Kind::Marginal).expect("a table");
        let three = factorial(&r, &terms, 8.0, Kind::Partial).expect("a table");
        for i in 0..3 {
            assert!((one.terms[i].ss - two.terms[i].ss).abs() < 1e-12);
            assert!((one.terms[i].ss - three.terms[i].ss).abs() < 1e-12);
        }
        // By hand: the effect of `a` is the squared contrast over 8.
        let contrast: f64 = a.iter().zip(&y).map(|(a, y)| a * y).sum();
        assert!((one.terms[0].ss - contrast * contrast / 8.0).abs() < 1e-12);
    }

    #[test]
    fn a_table_that_cannot_be_computed_says_why() {
        let (r, terms) = two_by_two();
        // As many columns as observations.
        assert_eq!(
            factorial(&r, &terms, 4.0, Kind::Marginal),
            Err(Undefined::NoResidualFreedom { rank: 4 })
        );
        // An outcome the model reproduces.
        let a = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
        let y = [2.0, 2.0, 2.0, 5.0, 5.0, 5.0];
        let exact = factor_of(&[vec![1.0; 6], a.to_vec()], &y);
        let one = vec![term("a", &["category:a"], 1..2)];
        assert_eq!(
            factorial(&exact, &one, 6.0, Kind::Marginal),
            Err(Undefined::PerfectFit)
        );
        // A column that repeats another: type 3 is refused, type 2 gives the term no freedom.
        let y = [2.0, 2.5, 1.0, 5.0, 4.0, 5.5];
        let twice = factor_of(&[vec![1.0; 6], a.to_vec(), a.to_vec()], &y);
        let two = vec![
            term("a", &["category:a"], 1..2),
            term("c", &["category:c"], 2..3),
        ];
        assert_eq!(
            factorial(&twice, &two, 6.0, Kind::Partial),
            Err(Undefined::Aliased {
                rank: 2,
                columns: 3
            })
        );
        let table = factorial(&twice, &two, 6.0, Kind::Marginal).expect("a table");
        assert_eq!(table.terms[0].df, 0);
        assert!(table.terms[0].f.is_nan());
        let sequential = factorial(&twice, &two, 6.0, Kind::Sequential).expect("a table");
        assert_eq!((sequential.terms[0].df, sequential.terms[1].df), (1, 0));
        // A model whose one column does not vary.
        let flat = factor_of(&[vec![1.0; 6], vec![1.0; 6]], &y);
        assert_eq!(
            factorial(&flat, &one, 6.0, Kind::Marginal),
            Err(Undefined::NoModel)
        );
    }
}
