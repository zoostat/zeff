//! The model matrix (Language Reference, section 7.2; Engine and Project Specification, section
//! 8.3, point 2 `design` and point 10): a formula and a frame become the named columns of the
//! design, produced in row blocks so that no n × k matrix is held, with the rows used, the rows
//! dropped for `NA` and the notes of what was changed on the way.
//!
//! The rules, in the order section 7.2 applies them: expansion of `a * b` and `interact`; a term
//! written twice kept once; the intercept, and the first category fully coded without it; a
//! category's indicator columns by its contrast, the base the first level or `base()`'s, and full
//! coding for a category in an interaction whose margin is absent, as R codes it; arithmetic
//! computed by the column kernels; rows with `NA` dropped and counted, `Inf` refused; and the term
//! names, `industry=Retail`, `educ:female`, `exper^2`, `(intercept)`, with the names of question 5
//! of release 0.4.0 under the `sum`, `helmert` and `poly` contrasts.

use std::collections::BTreeSet;
use zeff_columns::{Column, ColumnType, Contrast, Table};
use zeff_core::display::list_levels;
use zeff_core::lift;
use zeff_core::operators::{self, Op, Operand};
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::{ColumnExpr, Formula, Frame, Term, TermKind, TypeValue, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::Matrix;
use zeff_numeric::order::WeightKind;

/// The rows of one block of the design, at most: the kernels' shape (open decision #48).
pub const BLOCK_ROWS: usize = zeff_kernels::qr::BLOCK_ROWS;

/// A column of the frame, or an expression over its columns, evaluated once: numbers with their
/// missing rows, or the codes of a category with its levels and contrast.
enum Source {
    Numeric {
        values: Vec<f64>,
        na: Vec<bool>,
    },
    Coded {
        codes: Vec<Option<i32>>,
        levels: Vec<String>,
        contrast: Contrast,
        /// The name the indicator columns carry: the variable's.
        var: String,
    },
}

impl Source {
    fn is_na(&self, row: usize) -> bool {
        match self {
            Source::Numeric { na, .. } => na[row],
            Source::Coded { codes, .. } => codes[row].is_none(),
        }
    }
}

/// A part of a design column: a number, or the weight of the row's level.
enum Part {
    Numeric(usize),
    Weighted { source: usize, weights: Vec<f64> },
}

/// One column of the design: its name and the parts whose product it is; no part is the
/// intercept.
struct DesignColumn {
    name: String,
    parts: Vec<Part>,
}

/// A term after expansion: the sources of its parts, in order.
struct ExpandedTerm {
    sources: Vec<usize>,
}

/// The largest ratio of a positive weight to the smallest the design accepts: beyond it the rows
/// with the small weights underflow the arithmetic into noise.
const WEIGHT_SPREAD: f64 = 1e150;

/// The model matrix of a formula on a frame.
pub struct Design {
    /// The columns' names, in order (section 7.2, "Term names"; question 5).
    pub names: Vec<String>,
    /// The rows of the frame the design uses, in order: those without `NA` in any model
    /// variable.
    pub rows: Vec<usize>,
    /// How many rows were dropped for `NA`.
    pub dropped: usize,
    /// The rows dropped for `NA` by variable, in the order the variables are used; a row with
    /// `NA` in two variables counts in both.
    pub dropped_in: Vec<(String, usize)>,
    /// What was changed on the way (section 7.2: "Any rule that changes what the user wrote is
    /// reported in a note").
    pub notes: Vec<String>,
    /// Whether `(intercept)` is the first column.
    pub intercept: bool,
    /// The weight of each row used, positive, when the model is weighted (section 6): the blocks
    /// carry each row scaled by the square root of its weight; analytic weights are scaled to
    /// sum to the rows used.
    pub weights: Option<Vec<f64>>,
    /// The kind of the weights, when weighted.
    pub weight_kind: Option<WeightKind>,
    /// Rows left out for a zero weight (section 4.1, "Weights").
    pub zero_weights: usize,
    /// When built on new data with a fit's levels (`predict`): by category variable, the rows
    /// whose label the fit never saw, which count as `NA`.
    pub unseen: Vec<(String, usize)>,
    columns: Vec<DesignColumn>,
    sources: Vec<Source>,
    outcome: Option<usize>,
    /// The outcome's term as written, for messages that name it.
    outcome_text: Option<String>,
    block_rows: usize,
}

/// One block of the design: the rows of `rows` from `start`, their design as a matrix and their
/// outcome.
pub struct Block {
    pub start: usize,
    pub x: Matrix,
    pub y: Vec<f64>,
}

impl Design {
    /// The design of `formula` on `frame`, the variables of `extra` (weights, a cluster) joining
    /// the model variables whose `NA` rows are dropped.
    pub fn build(
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        formula: &Formula,
        extra: &[&str],
    ) -> Result<Design, Fail> {
        Design::build_with(ctx, frame, formula, extra, &[])
    }

    /// The design of `formula` on `frame` with the levels of `plan` for its category variables,
    /// each in the plan's order: how `predict` rebuilds a fit's design on new data, so that the
    /// columns are the fit's whatever the new data hold. A label outside the plan is `NA` and
    /// counted in `unseen`; a text variable named in the plan is coded by its labels.
    pub fn build_with(
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        formula: &Formula,
        extra: &[&str],
        plan: &[(String, Vec<String>)],
    ) -> Result<Design, Fail> {
        let mut builder = Builder {
            ctx,
            frame: &frame.table,
            shown: frame.shown_name(),
            sources: Vec::new(),
            keys: Vec::new(),
            notes: Vec::new(),
            plan,
            unseen: Vec::new(),
        };
        let outcome = match &formula.outcome {
            Some(term) => Some(builder.outcome(term)?),
            None => None,
        };
        let mut terms: Vec<ExpandedTerm> = Vec::new();
        for term in &formula.terms {
            builder.expand(term, &mut terms)?;
        }
        builder.drop_duplicates(&mut terms);
        let extra_sources: Vec<usize> = extra
            .iter()
            .map(|name| builder.variable(name))
            .collect::<Result<_, _>>()?;
        let columns = builder.columns(&terms, formula.intercept);
        let mut names: Vec<String> = Vec::new();
        if formula.intercept {
            names.push("(intercept)".to_string());
        }
        names.extend(columns.iter().map(|c| c.name.clone()));
        builder.refuse_infinities()?;
        let (rows, dropped, dropped_in) = builder.rows_without_na(&extra_sources, outcome);
        builder.refuse_infinite_products(&columns, &rows)?;
        Ok(Design {
            names,
            rows,
            dropped,
            dropped_in,
            notes: builder.notes,
            intercept: formula.intercept,
            weights: None,
            weight_kind: None,
            zero_weights: 0,
            unseen: builder.unseen,
            columns,
            sources: builder.sources,
            outcome,
            outcome_text: formula.outcome.as_ref().map(|t| t.text.clone()),
            block_rows: BLOCK_ROWS,
        })
    }

    /// The design weighted by the variable `name` of `frame` under `kind` (section 6; section
    /// 4.1, "Weights"): the variable was given among `extra` when the design was built, so its
    /// `NA` rows are already dropped; a row with zero weight is left out and counted; a negative
    /// weight is an error listing the rows; a frequency weight must be a whole number; analytic
    /// and sampling weights are scaled to sum to the rows kept, which changes no estimate and no
    /// standard error (both are invariant to the scale of such weights) and keeps a weight of any
    /// size inside the range the arithmetic can square.
    pub fn weighted(
        mut self,
        ctx: &mut CallContext<'_>,
        frame: &Frame,
        name: &str,
        kind: WeightKind,
    ) -> Result<Design, Fail> {
        let Some(column) = frame.table.column(name) else {
            let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
            return Err(fail(no_variable(
                ctx.script,
                ctx.callee_span,
                &frame.shown_name(),
                name,
                &columns,
            )));
        };
        let values: Vec<Option<f64>> = match column.ty() {
            ColumnType::Float => column.as_floats().expect("a Float column").collect(),
            ColumnType::Int => column
                .as_ints()
                .expect("an Int column")
                .map(|v| v.map(|i| i as f64))
                .collect(),
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`weights:` needs a numeric variable, but `{name}` is {} variable", article(other.name())),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "not a weight")
                    .help("a weight is a number for each row: a count, a precision or a sampling weight"),
                ));
            }
        };
        let mut negative: Vec<usize> = Vec::new();
        let mut fractional: Vec<usize> = Vec::new();
        for &row in &self.rows {
            let w = values[row].unwrap_or(0.0);
            if w < 0.0 {
                negative.push(row + 1);
            } else if kind == WeightKind::Frequency && w.fract() != 0.0 {
                fractional.push(row + 1);
            }
        }
        if !negative.is_empty() {
            return Err(fail(
                Diagnostic::error(
                    "Z0224",
                    format!(
                        "`{name}` holds a negative weight in {} {}",
                        if negative.len() == 1 { "row" } else { "rows" },
                        list_rows(&negative)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a weight below zero")
                .help(
                    "weights are zero or positive; `where(w >= 0)` drops the rows, or recode them",
                ),
            ));
        }
        if !fractional.is_empty() {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("frequency weights are whole numbers, but `{name}` holds a fraction in {} {}", if fractional.len() == 1 { "row" } else { "rows" }, list_rows(&fractional)),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a frequency weight with a fraction")
                .help("a frequency weight counts how many times a row stands; `weight_type: analytic` or `sampling` takes any positive weight"),
            ));
        }
        let before = self.rows.len();
        self.rows.retain(|&row| values[row].unwrap_or(0.0) > 0.0);
        self.zero_weights = before - self.rows.len();
        let mut weights: Vec<f64> = self
            .rows
            .iter()
            .map(|&row| values[row].unwrap_or(0.0))
            .collect();
        // Weights too far apart (the formula fuzz target's finding): below one part in 1e150 of
        // the largest, a row's share of the fit underflows into noise, so the spread is refused.
        let smallest = weights.iter().copied().fold(f64::INFINITY, f64::min);
        let largest = weights.iter().copied().fold(0.0_f64, f64::max);
        if !weights.is_empty() && largest > WEIGHT_SPREAD * smallest {
            return Err(fail(
                Diagnostic::error(
                    "Z0224",
                    format!(
                        "`{name}` holds weights too far apart: the largest, {}, is more than 1e150 times the smallest, {}",
                        zeff_core::display::format_float(largest),
                        zeff_core::display::format_float(smallest)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "weights no fit can use together")
                .help("a weight that small is a share of nothing; `where` the rows with such weights out, or trim the weights"),
            ));
        }
        if matches!(kind, WeightKind::Analytic | WeightKind::Sampling) && !weights.is_empty() {
            let total = zeff_kernels::sum(&weights);
            let n = weights.len() as f64;
            for w in &mut weights {
                *w = *w * n / total;
            }
        }
        self.weights = Some(weights);
        self.weight_kind = Some(kind);
        Ok(self)
    }

    /// The square root of the weight of the row used at `index`, or 1 without weights.
    fn root_weight(&self, index: usize) -> f64 {
        self.weights.as_ref().map_or(1.0, |w| w[index].sqrt())
    }

    /// The sum of the weights of the rows used (the observation count under frequency weights),
    /// or the number of rows without weights.
    pub fn weight_total(&self) -> f64 {
        self.weights
            .as_ref()
            .map_or(self.rows.len() as f64, |w| zeff_kernels::sum(w))
    }

    /// The same design in blocks of `rows` rows (a test runs every kernel at several sizes).
    pub fn with_block_rows(mut self, rows: usize) -> Design {
        self.block_rows = rows.max(1);
        self
    }

    /// The number of columns, the intercept included.
    pub fn k(&self) -> usize {
        self.names.len()
    }

    /// The number of rows used.
    pub fn n(&self) -> usize {
        self.rows.len()
    }

    /// The levels of each category variable of the design, in the order the columns use them,
    /// by variable: what a fit keeps so that `predict` codes new data the same way.
    pub fn category_levels(&self) -> Vec<(String, Vec<String>)> {
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        for source in &self.sources {
            if let Source::Coded { levels, var, .. } = source
                && !out.iter().any(|(v, _)| v == var)
            {
                out.push((var.clone(), levels.clone()));
            }
        }
        out
    }

    /// The outcome's term as written (`log(wage)`), for messages that name it.
    pub fn outcome_name(&self) -> Option<String> {
        self.outcome_text.clone()
    }

    /// The value of column `column` (the intercept counted) at the frame row `row`.
    fn value(&self, column: usize, row: usize) -> f64 {
        let column = if self.intercept {
            if column == 0 {
                return 1.0;
            }
            column - 1
        } else {
            column
        };
        let mut product = 1.0;
        for part in &self.columns[column].parts {
            product *= match part {
                Part::Numeric(source) => match &self.sources[*source] {
                    Source::Numeric { values, .. } => values[row],
                    Source::Coded { .. } => 0.0,
                },
                Part::Weighted { source, weights } => match &self.sources[*source] {
                    Source::Coded { codes, .. } => codes[row].map_or(0.0, |c| weights[c as usize]),
                    Source::Numeric { .. } => 0.0,
                },
            };
        }
        product
    }

    /// The outcome at the frame row `row`, or 0 without one.
    fn outcome_at(&self, row: usize) -> f64 {
        match self.outcome.map(|s| &self.sources[s]) {
            Some(Source::Numeric { values, .. }) => values[row],
            _ => 0.0,
        }
    }

    /// The design block by block, each block's columns computed for its rows.
    pub fn blocks(&self) -> impl Iterator<Item = Block> + '_ {
        let k = self.k();
        (0..self.rows.len())
            .step_by(self.block_rows)
            .map(move |start| {
                let end = (start + self.block_rows).min(self.rows.len());
                let rows = &self.rows[start..end];
                let x = Matrix::from_fn(rows.len(), k, |r, c| {
                    self.value(c, rows[r]) * self.root_weight(start + r)
                });
                let y = rows
                    .iter()
                    .enumerate()
                    .map(|(r, &row)| self.outcome_at(row) * self.root_weight(start + r))
                    .collect();
                Block { start, x, y }
            })
    }

    /// The whole design as one matrix, the rows scaled by the square roots of their weights:
    /// for tests and for small designs only, since it holds n × k values.
    pub fn matrix(&self) -> Matrix {
        let k = self.k();
        Matrix::from_fn(self.rows.len(), k, |r, c| {
            self.value(c, self.rows[r]) * self.root_weight(r)
        })
    }

    /// The outcome of every row used, unweighted, or `None` for a formula without one.
    pub fn outcome(&self) -> Option<Vec<f64>> {
        self.outcome
            .map(|_| self.rows.iter().map(|&row| self.outcome_at(row)).collect())
    }
}

/// The work of building a design.
struct Builder<'a, 'b> {
    ctx: &'a mut CallContext<'b>,
    frame: &'a Table,
    /// The frame as messages name it.
    shown: String,
    sources: Vec<Source>,
    /// The written form each source stands for, by source index.
    keys: Vec<String>,
    notes: Vec<String>,
    /// The levels to code each category variable against, when predicting on new data.
    plan: &'a [(String, Vec<String>)],
    /// By variable, the rows whose label the plan does not hold.
    unseen: Vec<(String, usize)>,
}

impl Builder<'_, '_> {
    // ----- sources -----

    /// The planned levels of `var`, when predicting.
    fn planned(&self, var: &str) -> Option<Vec<String>> {
        self.plan
            .iter()
            .find(|(v, _)| v == var)
            .map(|(_, levels)| levels.clone())
    }

    /// Labels coded against the planned levels: a label the plan does not hold is `NA`, counted
    /// as unseen.
    fn recode(
        &mut self,
        var: &str,
        labels: impl Iterator<Item = Option<String>>,
        planned: Vec<String>,
    ) -> (Vec<Option<i32>>, Vec<String>) {
        let mut unseen = 0usize;
        let codes: Vec<Option<i32>> = labels
            .map(|label| {
                label.and_then(|l| match planned.iter().position(|p| *p == l) {
                    Some(i) => Some(i as i32),
                    None => {
                        unseen += 1;
                        None
                    }
                })
            })
            .collect();
        if unseen > 0 {
            self.unseen.push((var.to_string(), unseen));
        }
        (codes, planned)
    }

    /// The source of a variable of the frame (Z0101 when there is none).
    fn variable(&mut self, name: &str) -> Result<usize, Fail> {
        if let Some(i) = self.keys.iter().position(|k| k == name) {
            return Ok(i);
        }
        let Some(column) = self.frame.column(name) else {
            let columns: Vec<&str> = self.frame.names().iter().map(String::as_str).collect();
            return Err(fail(no_variable(
                self.ctx.script,
                self.ctx.callee_span,
                &self.shown,
                name,
                &columns,
            )));
        };
        let source = self.source_of_column(name, column.clone(), name)?;
        self.keys.push(name.to_string());
        self.sources.push(source);
        Ok(self.sources.len() - 1)
    }

    /// A column as a source: numbers (Int, Float, Bool, a labelled Int as the number it holds) or
    /// a category; text and dates are refused.
    fn source_of_column(&mut self, text: &str, column: Column, var: &str) -> Result<Source, Fail> {
        let rows = column.len();
        let source = match column.ty() {
            ColumnType::Float => {
                let values: Vec<Option<f64>> =
                    column.as_floats().expect("a Float column").collect();
                numeric(values)
            }
            ColumnType::Int => {
                let values: Vec<Option<f64>> = column
                    .as_ints()
                    .expect("an Int column")
                    .map(|v| v.map(|i| i as f64))
                    .collect();
                numeric(values)
            }
            ColumnType::Bool => {
                let values: Vec<Option<f64>> = column
                    .as_bools()
                    .expect("a Bool column")
                    .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
                    .collect();
                numeric(values)
            }
            ColumnType::Category => {
                let own: Vec<String> = column
                    .levels()
                    .expect("a category")
                    .iter()
                    .map(|l| (*l).to_string())
                    .collect();
                let contrast = column
                    .category_info()
                    .map(|i| i.contrast)
                    .unwrap_or_default();
                let codes: Vec<Option<i32>> = column.as_codes().expect("a category").collect();
                match self.planned(var) {
                    // Predicting: the fit's levels in the fit's order, whatever the new column's.
                    Some(planned) => {
                        let labels = codes.iter().map(|c| c.map(|c| own[c as usize].clone()));
                        let (codes, levels) = self.recode(var, labels, planned);
                        Source::Coded {
                            codes,
                            levels,
                            contrast,
                            var: var.to_string(),
                        }
                    }
                    None => Source::Coded {
                        codes,
                        levels: own,
                        contrast,
                        var: var.to_string(),
                    },
                }
            }
            // Predicting: text coded by the fit's labels, as the fit's category.
            ColumnType::Str if self.planned(var).is_some() => {
                let planned = self.planned(var).expect("planned");
                let labels: Vec<Option<String>> = column
                    .as_strs()
                    .expect("text")
                    .map(|s| s.map(|s| s.to_string()))
                    .collect();
                let (codes, levels) = self.recode(var, labels.into_iter(), planned);
                Source::Coded {
                    codes,
                    levels,
                    contrast: Contrast::default(),
                    var: var.to_string(),
                }
            }
            ColumnType::Str => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`{text}` is text, and a model needs numbers or a category"),
                    )
                    .in_file(self.ctx.script)
                    .primary(self.ctx.callee_span, "a text variable in the model")
                    .help(format!(
                        "`category({text})` makes it a category, in the formula or with `add`"
                    )),
                ));
            }
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0304",
                        format!(
                            "`{text}` is a {} variable, which models take from release 0.6.0",
                            other.name()
                        ),
                    )
                    .in_file(self.ctx.script)
                    .primary(self.ctx.callee_span, "a date or time in the model")
                    .help("the `time` package of 0.6.0 brings dates and times to models"),
                ));
            }
        };
        debug_assert_eq!(rows, self.frame.rows());
        Ok(source)
    }

    /// The source of a part of a term: a variable, or an expression over variables.
    fn part_source(&mut self, term: &Term) -> Result<usize, Fail> {
        match &term.kind {
            TermKind::Var(name) => self.variable(name),
            TermKind::Expr(expr) => self.expression(&term.text, expr),
            TermKind::Range(..) | TermKind::Interaction(_) | TermKind::Product(_) => {
                Err(self.internal("a range, an interaction or a product as a part"))
            }
        }
    }

    /// An expression over variables as a source: evaluated once over the whole frame, by the
    /// column kernels and the functions of `core` (section 7.2, "Arithmetic").
    fn expression(&mut self, text: &str, expr: &ColumnExpr) -> Result<usize, Fail> {
        if let Some(i) = self.keys.iter().position(|k| k == text) {
            return Ok(i);
        }
        let value = self.evaluate(expr)?;
        let rows = self.frame.rows();
        let column = match value {
            Value::Column(column) => column,
            scalar @ (Value::Int(_) | Value::Float(_) | Value::Bool(_)) => {
                lift::broadcast(&scalar, rows).expect("a scalar broadcasts")
            }
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`{text}` gives {}, not a column, so it cannot be a term", other.type_word()),
                    )
                    .in_file(self.ctx.script)
                    .primary(self.ctx.callee_span, "not a column")
                    .help("a term computes a column of the frame: a variable, `log(x)`, `x^2`, `category(x)`"),
                ));
            }
        };
        if column.len() != rows {
            return Err(self.internal("a term of another number of rows"));
        }
        // The indicator columns of `category(x, ...)` and `base(x, ...)` are named by `x`.
        let var = match expr {
            ColumnExpr::Call { name, args, .. }
                if (name == "category" || name == "base")
                    && matches!(args.first(), Some(Some(ColumnExpr::Var(_)))) =>
            {
                match &args[0] {
                    Some(ColumnExpr::Var(v)) => v.clone(),
                    _ => text.to_string(),
                }
            }
            _ => text.to_string(),
        };
        let source = self.source_of_column(text, column, &var)?;
        self.keys.push(text.to_string());
        self.sources.push(source);
        Ok(self.sources.len() - 1)
    }

    /// An expression over the frame's variables, evaluated to a value.
    fn evaluate(&mut self, expr: &ColumnExpr) -> Result<Value, Fail> {
        Ok(match expr {
            ColumnExpr::Var(name) => match self.frame.column(name) {
                Some(column) => Value::Column(column.clone()),
                None => {
                    let columns: Vec<&str> =
                        self.frame.names().iter().map(String::as_str).collect();
                    return Err(fail(no_variable(
                        self.ctx.script,
                        self.ctx.callee_span,
                        &self.shown,
                        name,
                        &columns,
                    )));
                }
            },
            ColumnExpr::Int(i) => Value::Int(*i),
            ColumnExpr::Float(x) => Value::Float(*x),
            ColumnExpr::Str(s) => Value::Str(s.clone()),
            ColumnExpr::Bool(b) => Value::Bool(*b),
            ColumnExpr::Na => Value::Na,
            ColumnExpr::Word(w) => Value::Word(w.clone()),
            ColumnExpr::Type(name) => Value::Type(TypeValue {
                name: name.clone(),
                format: None,
            }),
            ColumnExpr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.evaluate(item))
                    .collect::<Result<_, _>>()?,
            ),
            ColumnExpr::Unary { negate, operand } => {
                let value = self.evaluate(operand)?;
                match &value {
                    Value::Column(column) => Value::Column(
                        operators::column_unary(*negate, column)
                            .map_err(|t| self.column_trouble(t))?,
                    ),
                    _ => if *negate {
                        operators::negate(&value)
                    } else {
                        operators::not(&value)
                    }
                    .map_err(|t| self.trouble(t))?,
                }
            }
            ColumnExpr::Binary { op, lhs, rhs } => {
                let left = self.evaluate(lhs)?;
                let right = self.evaluate(rhs)?;
                self.binary(*op, &left, &right)?
            }
            ColumnExpr::Call {
                package,
                name,
                args,
                rest,
                lifted,
                returns,
            } => {
                let values = args
                    .iter()
                    .map(|a| a.as_ref().map(|a| self.evaluate(a)).transpose())
                    .collect::<Result<Vec<_>, _>>()?;
                let rest = rest
                    .iter()
                    .map(|a| self.evaluate(a))
                    .collect::<Result<Vec<_>, _>>()?;
                let call = Args {
                    values,
                    rest,
                    definitions: Vec::new(),
                };
                let body = self.function(package, name)?;
                if *lifted {
                    lift::call_rowwise(body, self.ctx, call, returns)?
                } else {
                    body(self.ctx, call)?
                }
            }
        })
    }

    /// An operator of section 3.3 on two values, row by row when either is a column; the
    /// warnings of section 4.1 are raised, counted.
    fn binary(&mut self, op: Op, left: &Value, right: &Value) -> Result<Value, Fail> {
        fn operand(v: &Value) -> Option<Operand<'_>> {
            match v {
                Value::Column(c) => Some(Operand::Column(c)),
                _ => None,
            }
        }
        if operand(left).is_some() || operand(right).is_some() {
            let a = operand(left).unwrap_or(Operand::Scalar(left));
            let b = operand(right).unwrap_or(Operand::Scalar(right));
            let (column, notes) =
                operators::column_binary(op, a, b).map_err(|t| self.column_trouble(t))?;
            operators::warnings_of(notes, self.ctx.messages);
            return Ok(Value::Column(column));
        }
        let (value, note) = operators::binary(op, left, right).map_err(|t| self.trouble(t))?;
        if let Some(note) = note {
            self.ctx.message(operators::warning_of(note, &value));
        }
        Ok(value)
    }

    /// The body of a function a term calls: those of `core`, which every package may call.
    fn function(&self, package: &str, name: &str) -> Result<zeff_core::runtime::Body, Fail> {
        if package == "core"
            && let Some(f) = zeff_core::functions()
                .iter()
                .find(|f: &&Function| f.spec.name == name)
        {
            return Ok(f.body);
        }
        Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{name}` cannot be computed inside a formula"),
            )
            .in_file(self.ctx.script)
            .primary(self.ctx.callee_span, "a call a term cannot make")
            .help("a term calls the scalar functions of `core`, `category`, `base` and `interact`; compute anything else with `add` first"),
        ))
    }

    fn trouble(&self, trouble: operators::Trouble) -> Fail {
        fail(operators::trouble_diagnostic(
            trouble,
            self.ctx.script,
            self.ctx.callee_span,
        ))
    }

    fn column_trouble(&self, trouble: operators::ColumnTrouble) -> Fail {
        fail(operators::column_trouble_diagnostic(
            trouble,
            self.ctx.script,
            self.ctx.callee_span,
        ))
    }

    fn internal(&self, what: &str) -> Fail {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (the model matrix met {what})"),
            )
            .in_file(self.ctx.script)
            .primary(self.ctx.callee_span, "the call that failed"),
        )
    }

    // ----- the outcome and the terms -----

    /// The outcome: a numeric source (a category or text is refused).
    fn outcome(&mut self, term: &Term) -> Result<usize, Fail> {
        let source = self.part_source(term)?;
        if let Source::Coded { .. } = &self.sources[source] {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("the outcome `{}` is a category, and this model needs a number", term.text),
                )
                .in_file(self.ctx.script)
                .primary(self.ctx.callee_span, "a category on the left of `~`")
                .help("`to(y, Int)` gives a category's codes; a model for a categorical outcome arrives with `logistic` (release 0.5.0)"),
            ));
        }
        Ok(source)
    }

    /// A term expanded into the terms of the design (section 7.2, "Expansion"): a variable, each
    /// variable of a range, an interaction, every interaction of a product or of `interact` up to
    /// its order, or an expression.
    fn expand(&mut self, term: &Term, out: &mut Vec<ExpandedTerm>) -> Result<(), Fail> {
        match &term.kind {
            TermKind::Var(_) => {
                let source = self.part_source(term)?;
                out.push(ExpandedTerm {
                    sources: vec![source],
                });
            }
            TermKind::Range(first, last) => {
                for name in self.range(first, last)? {
                    let source = self.variable(&name)?;
                    out.push(ExpandedTerm {
                        sources: vec![source],
                    });
                }
            }
            TermKind::Interaction(parts) => {
                let sources = parts
                    .iter()
                    .map(|p| self.part_source(p))
                    .collect::<Result<Vec<_>, _>>()?;
                out.push(ExpandedTerm { sources });
            }
            TermKind::Product(parts) => {
                let units = parts
                    .iter()
                    .map(|p| match &p.kind {
                        TermKind::Interaction(inner) => inner
                            .iter()
                            .map(|q| self.part_source(q))
                            .collect::<Result<Vec<_>, _>>(),
                        _ => self.part_source(p).map(|s| vec![s]),
                    })
                    .collect::<Result<Vec<Vec<usize>>, _>>()?;
                interactions_up_to(&units, units.len(), out);
            }
            TermKind::Expr(ColumnExpr::Call {
                package,
                name,
                args,
                ..
            }) if package == "regress" && name == "interact" => {
                let variables = match args.first() {
                    Some(Some(ColumnExpr::List(items))) => items.clone(),
                    _ => return Err(self.internal("`interact` without its list")),
                };
                let order = match args.get(1) {
                    Some(Some(ColumnExpr::Int(n))) => *n,
                    _ => 2,
                };
                if variables.len() < 2 || order < 2 || order > variables.len() as i64 {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`interact` takes two or more variables and an order from 2 to their number; this has {} and order {order}",
                                count(variables.len() as u64)
                            ),
                        )
                        .in_file(self.ctx.script)
                        .primary(self.ctx.callee_span, "the variables and the order")
                        .help("`interact([a, b, c], order: 2)` is `a + b + c + a:b + a:c + b:c`"),
                    ));
                }
                let units = variables
                    .iter()
                    .map(|v| match v {
                        ColumnExpr::Var(name) => self.variable(name).map(|s| vec![s]),
                        other => {
                            let text = column_expr_text(other);
                            self.expression(&text, other).map(|s| vec![s])
                        }
                    })
                    .collect::<Result<Vec<Vec<usize>>, _>>()?;
                interactions_up_to(&units, order as usize, out);
            }
            TermKind::Expr(expr) => {
                let source = self.expression(&term.text, expr)?;
                out.push(ExpandedTerm {
                    sources: vec![source],
                });
            }
        }
        Ok(())
    }

    /// The variables from `first` to `last` in the frame's order (section 7.1, `x1..x9`).
    fn range(&self, first: &str, last: &str) -> Result<Vec<String>, Fail> {
        let names = self.frame.names();
        let columns: Vec<&str> = names.iter().map(String::as_str).collect();
        let position = |name: &str| -> Result<usize, Fail> {
            self.frame.position(name).ok_or_else(|| {
                fail(no_variable(
                    self.ctx.script,
                    self.ctx.callee_span,
                    &self.shown,
                    name,
                    &columns,
                ))
            })
        };
        let (a, b) = (position(first)?, position(last)?);
        if b < a {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("the range `{first}..{last}` runs backwards: `{last}` comes before `{first}` in the frame"),
                )
                .in_file(self.ctx.script)
                .primary(self.ctx.callee_span, "a range runs from an earlier variable to a later one")
                .help(format!("the variables, in order: {}", columns.join(", "))),
            ));
        }
        Ok(names[a..=b].to_vec())
    }

    /// A term written twice is kept once, with a note (section 7.2, "Duplicates"); `a:b` and
    /// `b:a` are one term.
    fn drop_duplicates(&mut self, terms: &mut Vec<ExpandedTerm>) {
        let mut seen: Vec<BTreeSet<String>> = Vec::new();
        let mut kept: Vec<ExpandedTerm> = Vec::new();
        for term in terms.drain(..) {
            // Two codings of one category variable (`category(c, contrast: helmert)` and
            // `category(c, contrast: sum)`) span the same columns under the same names, so they
            // are one term too (the formula fuzz target found the pair).
            let set: BTreeSet<String> = term
                .sources
                .iter()
                .map(|&s| match &self.sources[s] {
                    Source::Coded { var, .. } => format!("category:{var}"),
                    Source::Numeric { .. } => self.keys[s].clone(),
                })
                .collect();
            if let Some(first) = seen.iter().position(|s| *s == set) {
                let (kept_text, again) = (self.term_text(&kept[first]), self.term_text(&term));
                self.notes.push(if kept_text == again {
                    format!("`{kept_text}` is written twice and is kept once")
                } else {
                    format!("`{kept_text}` is written twice, once as `{again}`, and is kept once")
                });
                continue;
            }
            seen.push(set);
            kept.push(term);
        }
        *terms = kept;
    }

    /// A term's name from its parts: `educ:female`.
    fn term_text(&self, term: &ExpandedTerm) -> String {
        term.sources
            .iter()
            .map(|&s| self.keys[s].clone())
            .collect::<Vec<_>>()
            .join(":")
    }

    // ----- the columns -----

    /// The columns of every term (section 7.2, "Intercept" and "Categories"): a numeric part is
    /// one column; a category is coded by its contrast when the term's margin without it is in
    /// the model (an empty margin counts), and by every level otherwise; without an intercept,
    /// the first category met is coded by every level, as R codes it.
    fn columns(&mut self, terms: &[ExpandedTerm], intercept: bool) -> Vec<DesignColumn> {
        let sets: Vec<BTreeSet<usize>> = terms
            .iter()
            .map(|t| t.sources.iter().copied().collect())
            .collect();
        // Without an intercept, the first category with more than one level, scanning the terms
        // in order and the sources by their order of appearance.
        let mut fully_coded: Option<(usize, usize)> = None;
        if !intercept {
            'terms: for (j, term) in terms.iter().enumerate() {
                let mut candidates: Vec<usize> = term
                    .sources
                    .iter()
                    .copied()
                    .filter(|&s| matches!(&self.sources[s], Source::Coded { levels, .. } if levels.len() > 1))
                    .collect();
                candidates.sort_unstable();
                if let Some(&s) = candidates.first() {
                    fully_coded = Some((j, s));
                    break 'terms;
                }
            }
        }
        let mut columns = Vec::new();
        for (j, term) in terms.iter().enumerate() {
            // The columns of each part, as (name, part).
            let mut parts_columns: Vec<Vec<(String, Part)>> = Vec::new();
            for &s in &term.sources {
                match &self.sources[s] {
                    Source::Numeric { .. } => {
                        parts_columns.push(vec![(self.keys[s].clone(), Part::Numeric(s))]);
                    }
                    Source::Coded {
                        levels,
                        contrast,
                        var,
                        ..
                    } => {
                        let mut margin = sets[j].clone();
                        margin.remove(&s);
                        let contrasts = fully_coded != Some((j, s))
                            && (margin.is_empty() || sets.contains(&margin));
                        let coded = if contrasts {
                            contrast_columns(var, levels, *contrast)
                        } else {
                            full_columns(var, levels)
                        };
                        if levels.len() == 1 && contrasts {
                            self.notes.push(format!(
                                "`{var}` has one level, {}, and adds no column",
                                list_levels(levels)
                            ));
                        }
                        parts_columns.push(
                            coded
                                .into_iter()
                                .map(|(name, weights)| {
                                    (name, Part::Weighted { source: s, weights })
                                })
                                .collect(),
                        );
                    }
                }
            }
            // The product of the parts' column lists, the first part varying slowest.
            let mut combos: Vec<(Vec<String>, Vec<usize>)> = vec![(Vec::new(), Vec::new())];
            for part in &parts_columns {
                let mut next = Vec::new();
                for (names, indices) in &combos {
                    for (i, (name, _)) in part.iter().enumerate() {
                        let mut names = names.clone();
                        names.push(name.clone());
                        let mut indices = indices.clone();
                        indices.push(i);
                        next.push((names, indices));
                    }
                }
                combos = next;
            }
            for (names, indices) in combos {
                let parts = indices
                    .iter()
                    .enumerate()
                    .map(|(p, &i)| match &parts_columns[p][i].1 {
                        Part::Numeric(s) => Part::Numeric(*s),
                        Part::Weighted { source, weights } => Part::Weighted {
                            source: *source,
                            weights: weights.clone(),
                        },
                    })
                    .collect();
                columns.push(DesignColumn {
                    name: names.join(":"),
                    parts,
                });
            }
        }
        columns
    }

    // ----- rows -----

    /// Z0223 (section 4.1): an infinite value in a model variable, with its rows.
    fn refuse_infinities(&self) -> Result<(), Fail> {
        for (s, source) in self.sources.iter().enumerate() {
            if let Source::Numeric { values, na } = source {
                let rows: Vec<usize> = values
                    .iter()
                    .zip(na)
                    .enumerate()
                    .filter(|(_, (v, na))| !**na && v.is_infinite())
                    .map(|(i, _)| i + 1)
                    .collect();
                if !rows.is_empty() {
                    let listed = rows
                        .iter()
                        .take(10)
                        .map(|r| count(*r as u64))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let more = if rows.len() > 10 {
                        format!(" and {} more", count((rows.len() - 10) as u64))
                    } else {
                        String::new()
                    };
                    let name = &self.keys[s];
                    return Err(fail(
                        Diagnostic::error(
                            "Z0223",
                            format!(
                                "`{name}` holds an infinite value in {} {listed}{more}",
                                if rows.len() == 1 { "row" } else { "rows" }
                            ),
                        )
                        .in_file(self.ctx.script)
                        .primary(self.ctx.callee_span, "a model variable with Inf")
                        .help(format!("a model cannot use an infinite value; `where({name} != Inf && {name} != -Inf)` drops the rows, or recode them")),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Products of parts (an interaction, `a:b`) can overflow where their factors are finite: a
    /// row where a column's product is infinite is refused as an infinite value in a model
    /// variable is (Z0223), naming the column (the formula fuzz target found the case).
    fn refuse_infinite_products(
        &self,
        columns: &[DesignColumn],
        rows: &[usize],
    ) -> Result<(), Fail> {
        for column in columns.iter().filter(|c| c.parts.len() > 1) {
            let overflowing: Vec<usize> = rows
                .iter()
                .filter(|&&row| !self.product(column, row).is_finite())
                .map(|&row| row + 1)
                .collect();
            if overflowing.is_empty() {
                continue;
            }
            let listed = overflowing
                .iter()
                .take(10)
                .map(|r| count(*r as u64))
                .collect::<Vec<_>>()
                .join(", ");
            let more = if overflowing.len() > 10 {
                format!(" and {} more", count((overflowing.len() - 10) as u64))
            } else {
                String::new()
            };
            let name = &column.name;
            return Err(fail(
                Diagnostic::error(
                    "Z0223",
                    format!(
                        "`{name}` holds an infinite value in {} {listed}{more}: the product of its factors overflows",
                        if overflowing.len() == 1 { "row" } else { "rows" }
                    ),
                )
                .in_file(self.ctx.script)
                .primary(self.ctx.callee_span, "a model column with Inf")
                .help("a model cannot use an infinite value; rescale a factor first, `add(a_m = a / 1e6)`, or drop the rows with `where`"),
            ));
        }
        Ok(())
    }

    /// The product of a column's parts at the frame row `row`, as the design computes it.
    fn product(&self, column: &DesignColumn, row: usize) -> f64 {
        let mut product = 1.0;
        for part in &column.parts {
            product *= match part {
                Part::Numeric(source) => match &self.sources[*source] {
                    Source::Numeric { values, .. } => values[row],
                    Source::Coded { .. } => 0.0,
                },
                Part::Weighted { source, weights } => match &self.sources[*source] {
                    Source::Coded { codes, .. } => codes[row].map_or(0.0, |c| weights[c as usize]),
                    Source::Numeric { .. } => 0.0,
                },
            };
        }
        product
    }

    /// The rows without `NA` in any source used, with the count dropped and the counts by
    /// variable (section 4.1, "Models").
    fn rows_without_na(
        &self,
        extra: &[usize],
        outcome: Option<usize>,
    ) -> (Vec<usize>, usize, Vec<(String, usize)>) {
        let rows = self.frame.rows();
        let mut counts: Vec<(String, usize)> = Vec::new();
        let mut kept = Vec::with_capacity(rows);
        let mut dropped = 0;
        let used: Vec<usize> = outcome
            .into_iter()
            .chain(0..self.sources.len())
            .chain(extra.iter().copied())
            .fold(Vec::new(), |mut acc, s| {
                if !acc.contains(&s) {
                    acc.push(s);
                }
                acc
            });
        let _ = extra;
        for row in 0..rows {
            let mut missing = false;
            for &s in &used {
                if self.sources[s].is_na(row) {
                    missing = true;
                    let name = &self.keys[s];
                    match counts.iter_mut().find(|(n, _)| n == name) {
                        Some((_, c)) => *c += 1,
                        None => counts.push((name.clone(), 1)),
                    }
                }
            }
            if missing {
                dropped += 1;
            } else {
                kept.push(row);
            }
        }
        (kept, dropped, counts)
    }
}

/// `an Int`, `a Float`.
fn article(type_name: &str) -> String {
    if type_name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        format!("an {type_name}")
    } else {
        format!("a {type_name}")
    }
}

/// Up to ten row numbers, then the count of the rest.
fn list_rows(rows: &[usize]) -> String {
    let listed = rows
        .iter()
        .take(10)
        .map(|r| count(*r as u64))
        .collect::<Vec<_>>()
        .join(", ");
    if rows.len() > 10 {
        format!("{listed} and {} more", count((rows.len() - 10) as u64))
    } else {
        listed
    }
}

fn numeric(values: Vec<Option<f64>>) -> Source {
    let na: Vec<bool> = values.iter().map(Option::is_none).collect();
    Source::Numeric {
        values: values.into_iter().map(|v| v.unwrap_or(0.0)).collect(),
        na,
    }
}

/// Every interaction of the units up to `order`: the units themselves, then the pairs, then the
/// triples, in order, as R expands `a * b * c`. The walk through the combinations of each size
/// runs exactly as many times as there are combinations, so that no fault in the stepping can
/// make it run on without end.
fn interactions_up_to(units: &[Vec<usize>], order: usize, out: &mut Vec<ExpandedTerm>) {
    let m = units.len();
    for size in 1..=order.min(m) {
        let mut chosen: Vec<usize> = (0..size).collect();
        for _ in 0..combinations(m, size) {
            let sources: Vec<usize> = chosen
                .iter()
                .flat_map(|&u| units[u].iter().copied())
                .collect();
            out.push(ExpandedTerm { sources });
            next_combination(&mut chosen, m);
        }
    }
}

/// The number of combinations of `size` of `m`, for `size` at most `m`: each step multiplies the
/// count so far by `(m - i) / (i + 1)`, which leaves a whole number, and a count too large for
/// the machine saturates rather than wrapping round.
fn combinations(m: usize, size: usize) -> usize {
    (0..size).fold(1, |count, i| count.saturating_mul(m - i) / (i + 1))
}

/// Advances `chosen` to the next combination of `chosen.len()` of `m` in lexicographic order, and
/// leaves it as it is after the last.
fn next_combination(chosen: &mut [usize], m: usize) {
    let size = chosen.len();
    let mut i = size;
    while i > 0 {
        i -= 1;
        if chosen[i] < m - size + i {
            chosen[i] += 1;
            for j in i + 1..size {
                chosen[j] = chosen[j - 1] + 1;
            }
            return;
        }
    }
}

/// A level inside a term name: quoted when it holds `=`, `:` or `+`, so that it cannot read as
/// an interaction (section 7.2, "Term names").
fn level_name(level: &str) -> String {
    if level.contains(['=', ':', '+']) {
        format!("\"{}\"", level.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        level.to_string()
    }
}

/// The K − 1 columns of a category coded by its contrast (section 4.2; R's `contr.*`): each
/// column's name and the weight of every level.
fn contrast_columns(var: &str, levels: &[String], contrast: Contrast) -> Vec<(String, Vec<f64>)> {
    let k = levels.len();
    let named = |level: &str| format!("{var}={}", level_name(level));
    match contrast {
        // The base, the first level, has no column; a level's column is 1 at that level.
        Contrast::Treatment => (1..k)
            .map(|l| {
                let mut w = vec![0.0; k];
                w[l] = 1.0;
                (named(&levels[l]), w)
            })
            .collect(),
        // Deviations from the mean: a level's column is 1 at that level and −1 at the base.
        Contrast::Sum => (1..k)
            .map(|l| {
                let mut w = vec![0.0; k];
                w[l] = 1.0;
                w[0] = -1.0;
                (named(&levels[l]), w)
            })
            .collect(),
        // Each level against the mean of those before it: −1 for the levels before, j at the
        // level, 0 after.
        Contrast::Helmert => (1..k)
            .map(|j| {
                let mut w = vec![0.0; k];
                for i in w.iter_mut().take(j) {
                    *i = -1.0;
                }
                w[j] = j as f64;
                (named(&levels[j]), w)
            })
            .collect(),
        // Orthonormal polynomial trends over the levels' positions.
        Contrast::Poly => {
            let polynomials = orthonormal_polynomials(k);
            polynomials
                .into_iter()
                .enumerate()
                .map(|(d, w)| {
                    let degree = d + 1;
                    let name = match degree {
                        1 => "linear".to_string(),
                        2 => "quadratic".to_string(),
                        3 => "cubic".to_string(),
                        n => format!("degree{n}"),
                    };
                    (format!("{var}={name}"), w)
                })
                .collect()
        }
    }
}

/// Every level's indicator column (full coding).
fn full_columns(var: &str, levels: &[String]) -> Vec<(String, Vec<f64>)> {
    let k = levels.len();
    (0..k)
        .map(|l| {
            let mut w = vec![0.0; k];
            w[l] = 1.0;
            (format!("{var}={}", level_name(&levels[l])), w)
        })
        .collect()
}

/// The orthonormal polynomials of degree 1 to K − 1 over the positions 1..K, as R's
/// `contr.poly` gives them: the powers of the centred and scaled positions, orthogonalised
/// against the lower degrees twice (classical Gram–Schmidt with reorthogonalisation, in a fixed
/// order) and scaled to unit norm, so that each has a positive leading coefficient and the signs
/// are R's. Pure IEEE arithmetic in a fixed order, so the same bits on every target.
pub fn orthonormal_polynomials(k: usize) -> Vec<Vec<f64>> {
    if k < 2 {
        return Vec::new();
    }
    let mean = (k as f64 + 1.0) / 2.0;
    let scale = (k as f64 - 1.0) / 2.0;
    let x: Vec<f64> = (1..=k).map(|i| (i as f64 - mean) / scale).collect();
    let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(p, q)| p * q).sum::<f64>();
    let mut basis: Vec<Vec<f64>> = vec![vec![1.0 / (k as f64).sqrt(); k]];
    let mut out = Vec::with_capacity(k - 1);
    for degree in 1..k {
        let mut v: Vec<f64> = x.iter().map(|xi| xi.powi(degree as i32)).collect();
        for _ in 0..2 {
            for q in &basis {
                let c = dot(&v, q);
                for (vi, qi) in v.iter_mut().zip(q) {
                    *vi -= c * qi;
                }
            }
        }
        let norm = dot(&v, &v).sqrt();
        let q: Vec<f64> = v.iter().map(|vi| vi / norm).collect();
        out.push(q.clone());
        basis.push(q);
    }
    out
}

/// An expression's written form when the term text is not at hand (the items of `interact`).
fn column_expr_text(expr: &ColumnExpr) -> String {
    match expr {
        ColumnExpr::Var(name) => name.clone(),
        ColumnExpr::Int(i) => i.to_string(),
        ColumnExpr::Float(x) => zeff_core::display::format_float(*x),
        ColumnExpr::Str(s) => format!("\"{s}\""),
        ColumnExpr::Bool(b) => b.to_string(),
        ColumnExpr::Na => "NA".to_string(),
        ColumnExpr::Word(w) | ColumnExpr::Type(w) => w.clone(),
        ColumnExpr::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(column_expr_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ColumnExpr::Unary { negate, operand } => {
            format!(
                "{}{}",
                if *negate { "-" } else { "!" },
                column_expr_text(operand)
            )
        }
        ColumnExpr::Binary { op, lhs, rhs } => match op {
            Op::Pow => format!("{}^{}", column_expr_text(lhs), column_expr_text(rhs)),
            _ => format!(
                "{} {} {}",
                column_expr_text(lhs),
                op.symbol(),
                column_expr_text(rhs)
            ),
        },
        ColumnExpr::Call { name, args, .. } => format!(
            "{name}({})",
            args.iter()
                .flatten()
                .map(column_expr_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
