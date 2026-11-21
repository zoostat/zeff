//! What each column of a model matrix is a function of (release 0.9.0, step 10; open decision
//! #195, option A): the variables a column depends on, the form of its term, and its derivative
//! in one of them, row by row. `margins` differentiates a fitted model through every term that
//! is a function of the variable asked for, `exper` and `exper^2` alike, so the slope of a
//! quadratic is β₁ + 2β₂·exper and not β₁; that needs each column of the design to say what it
//! was made from.
//!
//! A column is the product of its parts (Language Reference, section 7.2): a variable, an
//! expression over variables, or the indicator of a category's level. The derivative is by the
//! product rule over the parts, and through an expression by the chain rule, exactly, with the
//! engine's own arithmetic: the functions `derive` differentiates (`+`, `-`, `*`, `/`, `^`,
//! `log`, `exp`, `sqrt`, `log1p`, `expm1`, `log10`, `log2`, `abs`). A column that depends on the
//! variable through anything else (a category made of it, a function that is not smooth) has no
//! derivative, and says why.

use crate::design::{Design, Part, Source};
use zeff_columns::{Column, ColumnType, Table};
use zeff_core::operators::Op;
use zeff_core::value::ColumnExpr;
use zeff_numeric::maths;

/// What a source of a design was made from.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Origin {
    /// A variable of the frame, by its name.
    Variable(String),
    /// An expression over variables of the frame: `exper^2`, `log(wage)`, `category(c)`.
    Expression(ColumnExpr),
    /// A column the design makes itself from several variables, which is no smooth function of
    /// any: the indicator of an event's relative period.
    Made,
}

impl Origin {
    /// The variables the source is written over, in the order they are written, each once.
    fn variables(&self) -> Vec<String> {
        let mut found = Vec::new();
        match self {
            Origin::Variable(name) => found.push(name.clone()),
            Origin::Expression(expr) => variables_of(expr, &mut found),
            Origin::Made => {}
        }
        found
    }
}

fn variables_of(expr: &ColumnExpr, found: &mut Vec<String>) {
    match expr {
        ColumnExpr::Var(name) => {
            if !found.contains(name) {
                found.push(name.clone());
            }
        }
        ColumnExpr::List(items) => items.iter().for_each(|item| variables_of(item, found)),
        ColumnExpr::Unary { operand, .. } => variables_of(operand, found),
        ColumnExpr::Binary { lhs, rhs, .. } => {
            variables_of(lhs, found);
            variables_of(rhs, found);
        }
        ColumnExpr::Call { args, rest, .. } => {
            args.iter()
                .flatten()
                .for_each(|arg| variables_of(arg, found));
            rest.iter().for_each(|arg| variables_of(arg, found));
        }
        ColumnExpr::Case(arms) => {
            for arm in arms {
                if let Some(condition) = &arm.condition {
                    variables_of(condition, found);
                }
                variables_of(&arm.value, found);
            }
        }
        ColumnExpr::Int(_)
        | ColumnExpr::Float(_)
        | ColumnExpr::Str(_)
        | ColumnExpr::Bool(_)
        | ColumnExpr::Na
        | ColumnExpr::Word(_)
        | ColumnExpr::Type(_) => {}
    }
}

/// The form of a column's term (section 7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `(intercept)`: a function of nothing.
    Intercept,
    /// A variable as it stands: `educ`.
    Variable,
    /// A variable raised to a constant power: `exper^2`.
    Power,
    /// Another expression over variables: `log(wage)`, `(a * 2)`.
    Function,
    /// The indicator, or contrast, of a category's level: `industry=Mining`.
    Level,
    /// A product of several of those: `educ:female`, `exper:industry=Mining`.
    Interaction,
}

/// What a column of a design is a function of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependence {
    /// The variables the column depends on, in the order its term writes them, each once.
    pub variables: Vec<String>,
    pub form: Form,
}

/// Why a column has no derivative in a variable it depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoDerivative {
    /// The variable enters the column as a category: its effect is a difference between
    /// levels, not a slope.
    Category { variable: String, column: String },
    /// The variable enters through something that is not differentiated; `what` says what.
    NotSmooth { column: String, what: String },
    /// The design's columns are held as values (the fixed effects projected out, the fitted
    /// regressors of two-stage least squares), so what they were made from is not at hand.
    Transformed,
}

/// How a variable enters a model (release 0.9.0, step 11a): what its effect is, a slope or a
/// difference between levels (open decision #88, definition (a)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    /// No column of the model is a function of it.
    Absent,
    /// Through numbers alone: the variable itself, or expressions over it.
    Number,
    /// As one category: the source that holds its codes, and its levels in the fit's order, the
    /// first of them the base.
    Category { source: usize, levels: Vec<String> },
    /// In a way that gives no single effect: as a number in one term and a category in
    /// another, in two categories, or in a category made of several variables. `through` is
    /// the written form of the category.
    Mixed { through: String },
}

/// The levels categories are held at, by source, in place of each row's own (release 0.9.0,
/// step 11a): for a source held, one share for each of its levels, in their order. A share of
/// one for a level and nought for the others sets every row to that level; the shares the
/// levels have in the sample hold each indicator at its mean. A source with `None`, or beyond
/// the end, keeps each row's own level.
pub type Held = [Option<Vec<f64>>];

impl Design {
    /// How `variable` enters the model's columns.
    pub fn role(&self, variable: &str) -> Role {
        if self.dense.is_some() {
            return Role::Absent;
        }
        let mut used: Vec<usize> = Vec::new();
        for column in &self.columns {
            for part in column.parts() {
                let source = Design::source_of(part);
                if !used.contains(&source) {
                    used.push(source);
                }
            }
        }
        used.sort_unstable();
        let mut number = false;
        let mut categories: Vec<usize> = Vec::new();
        for source in used {
            let variables = self.origins[source].variables();
            if !variables.iter().any(|v| v == variable) {
                continue;
            }
            match &self.sources[source] {
                Source::Numeric { .. } => number = true,
                Source::Coded { .. } => categories.push(source),
            }
        }
        match (number, categories.as_slice()) {
            (false, []) => Role::Absent,
            (true, []) => Role::Number,
            (false, [source]) if self.origins[*source].variables().len() == 1 => {
                match &self.sources[*source] {
                    Source::Coded { levels, .. } => Role::Category {
                        source: *source,
                        levels: levels.clone(),
                    },
                    Source::Numeric { .. } => Role::Absent,
                }
            }
            (_, [first, ..]) => Role::Mixed {
                through: self.keys[*first].clone(),
            },
        }
    }

    /// How many sources the design has: the length a [`Held`] may have.
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// The written forms of the sources, by source: two designs of one formula have the same.
    pub fn source_keys(&self) -> &[String] {
        &self.keys
    }

    /// For a source that is a category, the share of the rows used at each of its levels, by
    /// `weights` (one for each row used) when given: the mean of each level's indicator, which
    /// holds the category "at its means" (release 0.9.0, step 11b; open decision #202 (e)).
    /// `None` for a source that is a number.
    pub fn level_shares(&self, source: usize, weights: Option<&[f64]>) -> Option<Vec<f64>> {
        let Some(Source::Coded { codes, levels, .. }) = self.sources.get(source) else {
            return None;
        };
        // The weight of the rows at each level, summed in the order of the rows.
        let mut by_level: Vec<Vec<f64>> = vec![Vec::new(); levels.len()];
        for (index, &row) in self.rows.iter().enumerate() {
            if let Some(code) = codes[row] {
                by_level[code as usize].push(weights.map_or(1.0, |w| w[index]));
            }
        }
        let totals: Vec<f64> = by_level.iter().map(|w| zeff_kernels::sum(w)).collect();
        let total = zeff_kernels::sum(&totals);
        Some(totals.iter().map(|t| t / total).collect())
    }

    /// The value of a part at the frame row `row`, a category held as `held` says.
    fn part_value_held(&self, part: &Part, row: usize, held: &Held) -> f64 {
        if let Part::Weighted { source, weights } = part
            && let Some(Some(shares)) = held.get(*source)
        {
            // Σ over the levels of the level's share times its weight in this column, in the
            // order of the levels.
            let mut total = 0.0;
            for (share, weight) in shares.iter().zip(weights) {
                if *share != 0.0 {
                    total += share * weight;
                }
            }
            return total;
        }
        self.part_value(part, row)
    }

    /// The values of column `column` of `names` over the rows used, unweighted, the categories
    /// of `held` held as it says.
    pub fn column_held(&self, column: usize, held: &Held) -> Vec<f64> {
        if self.dense.is_some() {
            return self.column_values(column);
        }
        let parts = self.parts_of(column);
        self.rows
            .iter()
            .map(|&row| {
                let mut product = 1.0;
                for part in parts {
                    product *= self.part_value_held(part, row, held);
                }
                product
            })
            .collect()
    }

    /// The parts of column `column` of `names`: none for the intercept.
    fn parts_of(&self, column: usize) -> &[Part] {
        let offset = usize::from(self.intercept);
        match column.checked_sub(offset) {
            Some(at) => self.columns.get(at).map_or(&[], |c| c.parts()),
            None => &[],
        }
    }

    fn source_of(part: &Part) -> usize {
        match part {
            Part::Numeric(source) | Part::Weighted { source, .. } => *source,
        }
    }

    /// What column `column` of `names` is a function of; `None` for a design whose columns are
    /// held as values, or a column it does not have.
    pub fn dependence(&self, column: usize) -> Option<Dependence> {
        if self.dense.is_some() || column >= self.names.len() {
            return None;
        }
        let parts = self.parts_of(column);
        let mut variables: Vec<String> = Vec::new();
        for part in parts {
            for name in self.origins[Design::source_of(part)].variables() {
                if !variables.contains(&name) {
                    variables.push(name);
                }
            }
        }
        let form = match parts {
            [] => Form::Intercept,
            [Part::Weighted { .. }] => Form::Level,
            [Part::Numeric(source)] => match &self.origins[*source] {
                Origin::Variable(_) => Form::Variable,
                Origin::Expression(ColumnExpr::Binary {
                    op: Op::Pow,
                    lhs,
                    rhs,
                }) if matches!(**lhs, ColumnExpr::Var(_))
                    && matches!(**rhs, ColumnExpr::Int(_) | ColumnExpr::Float(_)) =>
                {
                    Form::Power
                }
                Origin::Expression(_) | Origin::Made => Form::Function,
            },
            _ => Form::Interaction,
        };
        Some(Dependence { variables, form })
    }

    /// The columns of `names` that `variable` enters, in order: `exper` enters `exper`,
    /// `exper^2` and `exper:female`. Empty for a variable the model does not have, and for a
    /// design whose columns are held as values.
    pub fn columns_of(&self, variable: &str) -> Vec<usize> {
        (0..self.names.len())
            .filter(|&column| {
                self.dependence(column)
                    .is_some_and(|d| d.variables.iter().any(|v| v == variable))
            })
            .collect()
    }

    /// The variables of the model's right-hand side, in the order its columns first use them.
    pub fn model_variables(&self) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for column in 0..self.names.len() {
            for name in self
                .dependence(column)
                .map(|d| d.variables)
                .unwrap_or_default()
            {
                if !found.contains(&name) {
                    found.push(name);
                }
            }
        }
        found
    }

    /// The derivative of column `column` in `variable`, for each row used, unweighted: `None`
    /// when the column does not depend on the variable (its derivative is zero in every row).
    /// By the product rule over the column's parts and the chain rule through an expression.
    pub fn derivative(
        &self,
        column: usize,
        variable: &str,
    ) -> Result<Option<Vec<f64>>, NoDerivative> {
        self.derivative_held(column, variable, &[])
    }

    /// [`Design::derivative`] with the categories of `held` held as it says: the slope at a
    /// level, or at the levels' shares, in place of each row's own level.
    pub fn derivative_held(
        &self,
        column: usize,
        variable: &str,
        held: &Held,
    ) -> Result<Option<Vec<f64>>, NoDerivative> {
        if self.dense.is_some() {
            return Err(NoDerivative::Transformed);
        }
        let parts = self.parts_of(column);
        let depends: Vec<bool> = parts
            .iter()
            .map(|part| {
                self.origins[Design::source_of(part)]
                    .variables()
                    .iter()
                    .any(|v| v == variable)
            })
            .collect();
        if !depends.iter().any(|d| *d) {
            return Ok(None);
        }
        let name = || self.names[column].clone();
        // Each part's values over the rows used, and the derivative of those that depend on the
        // variable.
        let n = self.rows.len();
        let mut values: Vec<Vec<f64>> = Vec::with_capacity(parts.len());
        let mut slopes: Vec<Option<Vec<f64>>> = Vec::with_capacity(parts.len());
        for (part, &dependent) in parts.iter().zip(&depends) {
            let source = Design::source_of(part);
            values.push(
                self.rows
                    .iter()
                    .map(|&row| self.part_value_held(part, row, held))
                    .collect(),
            );
            if !dependent {
                slopes.push(None);
                continue;
            }
            if matches!(part, Part::Weighted { .. }) {
                return Err(NoDerivative::Category {
                    variable: variable.to_string(),
                    column: name(),
                });
            }
            let slope = match &self.origins[source] {
                Origin::Variable(_) => vec![1.0; n],
                Origin::Expression(expr) => {
                    let Some(table) = &self.table else {
                        return Err(NoDerivative::Transformed);
                    };
                    let dual =
                        differentiate(expr, variable, table, &self.rows).map_err(|what| {
                            NoDerivative::NotSmooth {
                                column: name(),
                                what,
                            }
                        })?;
                    dual.slope
                }
                Origin::Made => {
                    return Err(NoDerivative::NotSmooth {
                        column: name(),
                        what: "a column the design makes from several variables".to_string(),
                    });
                }
            };
            slopes.push(Some(slope));
        }
        // Σ over the dependent parts of the part's slope times the other parts' values.
        let mut total = vec![0.0; n];
        for (p, slope) in slopes.iter().enumerate() {
            let Some(slope) = slope else { continue };
            for (row, slot) in total.iter_mut().enumerate() {
                let mut term = slope[row];
                for (q, value) in values.iter().enumerate() {
                    if q != p {
                        term *= value[row];
                    }
                }
                *slot += term;
            }
        }
        Ok(Some(total))
    }
}

/// An expression's values and its slopes in one variable, over the rows asked for.
struct Dual {
    value: Vec<f64>,
    slope: Vec<f64>,
}

impl Dual {
    fn constant(c: f64, n: usize) -> Dual {
        Dual {
            value: vec![c; n],
            slope: vec![0.0; n],
        }
    }

    /// f(a), with f and its derivative given row by row.
    fn map(&self, f: impl Fn(f64) -> (f64, f64)) -> Dual {
        let (value, slope) = self
            .value
            .iter()
            .zip(&self.slope)
            .map(|(&v, &s)| {
                let (fv, dfv) = f(v);
                (fv, dfv * s)
            })
            .unzip();
        Dual { value, slope }
    }

    /// f(a, b), with f and its two partial derivatives given row by row.
    fn combine(&self, other: &Dual, f: impl Fn(f64, f64) -> (f64, f64, f64)) -> Dual {
        let mut value = Vec::with_capacity(self.value.len());
        let mut slope = Vec::with_capacity(self.value.len());
        for row in 0..self.value.len() {
            let (v, da, db) = f(self.value[row], other.value[row]);
            value.push(v);
            slope.push(da * self.slope[row] + db * other.slope[row]);
        }
        Dual { value, slope }
    }

    fn is_constant(&self) -> bool {
        self.slope.iter().all(|s| *s == 0.0)
    }
}

/// A numeric variable of the frame over the rows asked for; `Err` for one that is not a number.
fn variable_values(table: &Table, name: &str, rows: &[usize]) -> Result<Vec<f64>, String> {
    let Some(column) = table.column(name) else {
        return Err(format!("`{name}` is not a variable of the frame"));
    };
    let all: Vec<Option<f64>> =
        numbers(column).ok_or_else(|| format!("`{name}` is not a number"))?;
    Ok(rows
        .iter()
        .map(|&row| all[row].unwrap_or(f64::NAN))
        .collect())
}

fn numbers(column: &Column) -> Option<Vec<Option<f64>>> {
    match column.ty() {
        ColumnType::Float => Some(column.as_floats()?.collect()),
        ColumnType::Int => Some(column.as_ints()?.map(|v| v.map(|i| i as f64)).collect()),
        ColumnType::Bool => Some(
            column
                .as_bools()?
                .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
                .collect(),
        ),
        _ => None,
    }
}

/// `expr` and its derivative in `variable` over `rows` of `table`, by forward-mode
/// differentiation with the engine's own arithmetic; `Err` says what in it is not smooth.
fn differentiate(
    expr: &ColumnExpr,
    variable: &str,
    table: &Table,
    rows: &[usize],
) -> Result<Dual, String> {
    let n = rows.len();
    match expr {
        ColumnExpr::Int(i) => Ok(Dual::constant(*i as f64, n)),
        ColumnExpr::Float(x) => Ok(Dual::constant(*x, n)),
        ColumnExpr::Var(name) => {
            let value = variable_values(table, name, rows)?;
            let slope = vec![if name == variable { 1.0 } else { 0.0 }; n];
            Ok(Dual { value, slope })
        }
        ColumnExpr::Unary { negate, operand } => {
            let inner = differentiate(operand, variable, table, rows)?;
            Ok(if *negate {
                inner.map(|v| (-v, -1.0))
            } else {
                inner
            })
        }
        ColumnExpr::Binary { op, lhs, rhs } => {
            let a = differentiate(lhs, variable, table, rows)?;
            let b = differentiate(rhs, variable, table, rows)?;
            Ok(match op {
                Op::Add => a.combine(&b, |x, y| (x + y, 1.0, 1.0)),
                Op::Sub => a.combine(&b, |x, y| (x - y, 1.0, -1.0)),
                Op::Mul => a.combine(&b, |x, y| (x * y, y, x)),
                Op::Div => a.combine(&b, |x, y| (x / y, 1.0 / y, -x / (y * y))),
                Op::Pow if b.is_constant() => {
                    // A constant power: b a^(b − 1), and nothing at all for the power one.
                    a.combine(&b, |x, y| {
                        let slope = if y == 1.0 {
                            1.0
                        } else {
                            y * zeff_numeric::pow(x, y - 1.0)
                        };
                        (zeff_numeric::pow(x, y), slope, 0.0)
                    })
                }
                // a^b: b a^(b − 1) da + a^b ln a db.
                Op::Pow => a.combine(&b, |x, y| {
                    let value = zeff_numeric::pow(x, y);
                    (
                        value,
                        y * zeff_numeric::pow(x, y - 1.0),
                        value * maths::log(x),
                    )
                }),
                other => return Err(format!("`{}` is not arithmetic", other.symbol())),
            })
        }
        ColumnExpr::Call {
            package,
            name,
            args,
            rest,
            ..
        } => {
            let given = args.iter().flatten().count() + rest.len();
            if package != "core" || given != 1 {
                return Err(format!("`{name}` is not differentiated"));
            }
            let arg = args
                .iter()
                .flatten()
                .next()
                .or(rest.first())
                .expect("one argument");
            let inner = differentiate(arg, variable, table, rows)?;
            Ok(match name.as_str() {
                "log" => inner.map(|v| (maths::log(v), 1.0 / v)),
                "exp" => inner.map(|v| {
                    let e = maths::exp(v);
                    (e, e)
                }),
                "sqrt" => inner.map(|v| {
                    let root = maths::sqrt(v);
                    (root, 0.5 / root)
                }),
                "log1p" => inner.map(|v| (maths::log1p(v), 1.0 / (1.0 + v))),
                "expm1" => inner.map(|v| (maths::expm1(v), maths::exp(v))),
                "log10" => inner.map(|v| (maths::log10(v), 1.0 / (v * maths::log(10.0)))),
                "log2" => inner.map(|v| (maths::log2(v), 1.0 / (v * maths::log(2.0)))),
                "abs" => inner.map(|v| (v.abs(), maths::sign(v))),
                _ => return Err(format!("`{name}` is not differentiated")),
            })
        }
        ColumnExpr::Case(_) => Err("a `case` is not differentiated".to_string()),
        ColumnExpr::Str(_)
        | ColumnExpr::Bool(_)
        | ColumnExpr::Na
        | ColumnExpr::Word(_)
        | ColumnExpr::Type(_)
        | ColumnExpr::List(_) => Err("a value that is not a number".to_string()),
    }
}
