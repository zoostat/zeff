//! `margins` (catalogue row 260; open decisions #88 and #195; release 0.9.0, step 11a): the
//! average marginal effects of a fitted regression, each with its delta-method standard error.
//!
//! The effect of a number is the slope of the prediction in it, taken through every term that
//! is a function of the variable (`exper` and `exper^2` alike, #195) and averaged over the rows
//! the fit used. The effect of a Bool or a 0/1 variable is the average difference between the
//! predictions at its two values, and that of a category the difference between each level and
//! the first, not a derivative (#88, definition (a)). For a linear model each is a linear
//! function J·b of the coefficients, J the average over the rows of the derivative of the model
//! matrix (or of its difference between the two values), so the standard error is √(J V Jᵀ) with
//! V the fit's own variance, classical, robust or clustered, the sample's regressors held fixed
//! (definition (b)). The average is weighted by the fit's weights when it has them.
//!
//! Step 11b (definition (c); open decision #202 (b) and (e)): `at:` sets a variable to a value
//! in every row, the others as observed; `type: mem` takes each effect at one point, every
//! number at its mean and every category at the shares of its levels; and with no variable
//! named the result is the adjusted prediction, J the averaged row of the model matrix itself.
//!
//! Step 12a: after `glm` everything is on the scale of the probability, p = g(η) the inverse
//! link of the linear predictor η = x·b. The slope of a number is g′(η) times the slope of η;
//! the effect of a 0/1 variable or of a level is the difference of g(η) at the two values; the
//! prediction is g(η); each averaged over the rows, or taken at the one point of the means. The
//! gradient in b, of which the standard error is √(J V Jᵀ), is the average of
//! g″(η)(dx·b)x + g′(η)dx for a slope, of g′(η₁)x₁ − g′(η₀)x₀ for a difference and of g′(η)x
//! for a prediction; the z table is the normal distribution's, as the fit's own.

use crate::dependence::{Held, NoDerivative, Role};
use crate::design::Design;
use crate::likelihood::Link;
use std::rc::Rc;
use zeff_columns::{Column, ColumnType, Table as Columns};
use zeff_core::display::{fixed, fixed_grouped, format_float, level_text, p_value_text};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Fit, FollowUp, Frame, Statistic, Table, TermTest,
    TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, did_you_mean};
use zeff_dist::families;
use zeff_numeric::order::WeightKind;

/// The decimals of an effect, its standard error and its interval: those of a fit's own table.
const DECIMALS: usize = 4;
/// The step of a central difference, as a share of the largest magnitude of the variable (or of
/// one, if that is larger): the cube root of the arithmetic's precision, at which the error of
/// truncation and the error of rounding of a central difference are of one size.
const STEP: f64 = 6.055_454_452_393_343e-6;

/// One effect: the row's name, what it compares, and the average over the rows of the
/// derivative (or difference) of each column of the model matrix.
struct Effect {
    /// The row's name, as the fit names the term: `educ`, `industry=Mining`.
    term: String,
    /// The variable, which the table prints.
    variable: String,
    contrast: String,
    /// The gradient of the quantity in the coefficients, one for each column of the model
    /// matrix: what the delta method's variance is taken through.
    jacobian: Vec<f64>,
    /// The average of the derivative (or difference) of each column: nought on every column
    /// the variable does not enter, so that it says whether the effect rests on a coefficient
    /// the fit omitted. For a linear model it is the gradient.
    linear: Vec<f64>,
    /// The quantity itself on the scale of the probability; `None` for a linear model, where
    /// it is the gradient times the coefficients.
    estimate: Option<f64>,
    /// Whether the row is the adjusted prediction and not an effect.
    prediction: bool,
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

/// The fit's model matrix rebuilt on `table`, the rows the fit used or the one row of the means:
/// the fit's columns, with the fit's levels for every category.
fn design_on(ctx: &mut CallContext<'_>, fit: &Fit, table: Columns) -> Result<Design, Fail> {
    let rows = table.rows();
    let mut model = fit.model.clone();
    model.outcome = None;
    // Whatever the second pass says was said when the model was fitted.
    let said = ctx.messages.len();
    let built = Design::build_full(
        ctx,
        &Frame::new(table),
        &model,
        &[],
        &fit.levels,
        &[],
        !fit.effects.is_empty(),
    );
    ctx.messages.truncate(said);
    let design = built?;
    if design.rows.len() != rows {
        return Err(internal(
            ctx,
            "the model matrix rebuilt for `margins` has other rows than it was given",
        ));
    }
    if let Some(name) = design
        .names
        .iter()
        .find(|n| !fit.terms.contains(n) && !fit.omitted.contains(n))
    {
        return Err(internal(
            ctx,
            &format!(
                "the model matrix rebuilt for `margins` has a column `{name}` the fit does not"
            ),
        ));
    }
    Ok(design)
}

/// The mean of `values` over the rows the fit used, by the fit's weights when it has them.
fn average(values: &[f64], weights: Option<&[f64]>) -> f64 {
    match weights {
        Some(w) => {
            let weighted: Vec<f64> = values.iter().zip(w).map(|(v, w)| v * w).collect();
            zeff_kernels::sum(&weighted) / zeff_kernels::sum(w)
        }
        None => zeff_kernels::sum(values) / values.len() as f64,
    }
}

/// `table` with the numeric variable `name` set to `value` in every row: in the variable's own
/// type where the value is one of that type, as a Float otherwise.
fn set_to(table: &Columns, name: &str, value: f64) -> Option<Columns> {
    let n = table.rows();
    let column = match table.column(name)?.ty() {
        ColumnType::Bool if value == 0.0 || value == 1.0 => {
            Column::bools(std::iter::repeat_n(Some(value != 0.0), n))
        }
        ColumnType::Int if value == value.trunc() && value.abs() < 9.0e15 => {
            Column::ints(std::iter::repeat_n(Some(value as i64), n))
        }
        _ => Column::floats(std::iter::repeat_n(Some(value), n)).0,
    };
    table.with_column(name, column).ok()
}

/// The values of the numeric variable `name` of `table`, row by row; `None` for a variable that
/// is not a number.
fn numbers_of(table: &Columns, name: &str) -> Option<Vec<f64>> {
    let column = table.column(name)?;
    let values: Vec<Option<f64>> = match column.ty() {
        ColumnType::Float => column.as_floats()?.collect(),
        ColumnType::Int => column.as_ints()?.map(|v| v.map(|i| i as f64)).collect(),
        ColumnType::Bool => column
            .as_bools()?
            .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
            .collect(),
        _ => return None,
    };
    // A row the fit used has no `NA` in a variable of the model.
    Some(values.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect())
}

/// What the two values of a 0/1 variable are called: `true − false` for a Bool, the value
/// labels when both values carry one (`female − male`), `1 − 0` otherwise.
fn values_named(column: &Column) -> String {
    if column.ty() == ColumnType::Bool {
        return "true \u{2212} false".to_string();
    }
    if let Some(labels) = column.metadata().value_labels() {
        let label = |code: &str| {
            labels
                .iter()
                .find(|(c, _)| c == code)
                .map(|(_, text)| text.clone())
        };
        if let (Some(one), Some(nought)) = (label("1"), label("0")) {
            return format!("{one} \u{2212} {nought}");
        }
    }
    "1 \u{2212} 0".to_string()
}

/// A level as a term's name writes it (section 7.2): quoted when it holds `=`, `:` or `+`.
fn level_name(level: &str) -> String {
    if level.contains(['=', ':', '+']) {
        format!("\"{}\"", level.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        level.to_string()
    }
}

/// Everything the effects of one call are computed from.
struct Model<'a> {
    fit: &'a Fit,
    /// The rows the fit used, as they are: what says whether a number is a 0/1 variable.
    observed: &'a Columns,
    /// The rows the effects are taken at: the rows the fit used, or the one row of the means,
    /// with the variables of `at:` set.
    table: Columns,
    /// The model matrix on those rows.
    design: Design,
    /// The categories held at a level (`at:`) or at the shares of their levels (`type: mem`).
    held: Vec<Option<Vec<f64>>>,
    /// The weight of each row of `table` in the average; `None` for equal weights.
    weights: Option<Vec<f64>>,
    /// The variables `at:` holds.
    at: Vec<String>,
    /// The link of a binary model, whose effects are on the scale of the probability; `None`
    /// for a linear model.
    link: Option<Link>,
    /// The coefficient of each column of the model matrix, nought for a column the fit omitted.
    beta: Vec<f64>,
    /// What is said under the table: the terms a slope was taken through, a central difference.
    after: Vec<String>,
}

impl Model<'_> {
    /// Z0417: the effect of `variable` cannot be computed, and why.
    fn no_effect(&self, ctx: &CallContext<'_>, variable: &str, why: String) -> Fail {
        fail(
            Diagnostic::error(
                "Z0417",
                format!("`margins` cannot take the effect of `{variable}`: {why}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "no effect of this variable")
            .help("`margins` takes the slope of a number through arithmetic and the functions of `core`, and the difference between the levels of a category"),
        )
    }

    /// The effects of `variable`: one row for a number or a 0/1 variable, one for each level of
    /// a category after the first.
    fn effects(&mut self, ctx: &mut CallContext<'_>, variable: &str) -> Result<Vec<Effect>, Fail> {
        match self.design.role(variable) {
            Role::Absent => Err(self.unknown(ctx, variable)),
            Role::Mixed { through } => Err(self.no_effect(
                ctx,
                variable,
                format!("it enters the model through the category `{through}` and through another term, so no one difference or slope is its effect"),
            )),
            Role::Category { source, levels } => {
                self.not_held(ctx, variable)?;
                Ok(self.levels(variable, source, &levels))
            }
            Role::Number => {
                let Some(values) = numbers_of(self.observed, variable) else {
                    return Err(self.no_effect(
                        ctx,
                        variable,
                        "it is not a number, a Bool or a category".to_string(),
                    ));
                };
                let column = self
                    .observed
                    .column(variable)
                    .expect("a variable with values");
                if column.ty() == ColumnType::Bool || values.iter().all(|v| *v == 0.0 || *v == 1.0)
                {
                    self.not_held(ctx, variable)?;
                    let contrast = values_named(column);
                    Ok(vec![self.two_values(ctx, variable, contrast)?])
                } else {
                    // The slope is taken where the rows are: at the value of `at:`, at the mean.
                    let at = numbers_of(&self.table, variable).unwrap_or(values);
                    Ok(vec![self.slope(ctx, variable, &at)?])
                }
            }
        }
    }

    /// Z0401 for a 0/1 variable or a category that `at:` holds at a value: its effect is the
    /// difference between its values, which holding it leaves nothing of.
    fn not_held(&self, ctx: &CallContext<'_>, variable: &str) -> Result<(), Fail> {
        if !self.at.iter().any(|name| name == variable) {
            return Ok(());
        }
        Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{variable}` is held at a value by `at:`, and its effect is the difference between its values"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "held here")
            .help(format!("leave `{variable}` out of `at:` to see its effect, or out of the variables to see the others' effects at that value")),
        ))
    }

    /// The adjusted prediction: the model matrix itself, averaged over the rows.
    fn prediction(&self) -> Effect {
        let linear: Vec<f64> = (0..self.design.names.len())
            .map(|column| {
                average(
                    &self.design.column_held(column, &self.held),
                    self.weights.as_deref(),
                )
            })
            .collect();
        let (jacobian, estimate) = match self.link {
            None => (linear.clone(), None),
            Some(link) => {
                let eta = self.eta(&self.design, &self.held);
                let slopes: Vec<f64> = eta.iter().map(|e| link.mean_slope(*e)).collect();
                let means: Vec<f64> = eta.iter().map(|e| link.mean(*e)).collect();
                (
                    self.columns_by(&self.design, &self.held, &slopes),
                    Some(average(&means, self.weights.as_deref())),
                )
            }
        };
        Effect {
            term: "prediction".to_string(),
            variable: "prediction".to_string(),
            contrast: String::new(),
            jacobian,
            linear,
            estimate,
            prediction: true,
        }
    }

    /// The linear predictor of each row of `design`, its categories held as `held` says: the
    /// columns times their coefficients, in the order of the columns.
    fn eta(&self, design: &Design, held: &Held) -> Vec<f64> {
        let mut eta = vec![0.0; design.rows.len()];
        for (column, b) in self.beta.iter().enumerate() {
            if *b == 0.0 {
                continue;
            }
            for (e, x) in eta.iter_mut().zip(design.column_held(column, held)) {
                *e += b * x;
            }
        }
        eta
    }

    /// For each column of `design`, the average over the rows of the column times `by`.
    fn columns_by(&self, design: &Design, held: &Held, by: &[f64]) -> Vec<f64> {
        (0..design.names.len())
            .map(|column| {
                let product: Vec<f64> = design
                    .column_held(column, held)
                    .iter()
                    .zip(by)
                    .map(|(x, m)| x * m)
                    .collect();
                average(&product, self.weights.as_deref())
            })
            .collect()
    }

    /// The gradient and the value of a difference between the predictions at two settings of
    /// the model matrix, on the scale of the probability; for a linear model the gradient is
    /// `linear` and the value is left to it.
    fn difference(
        &self,
        high: (&Design, &Held),
        low: (&Design, &Held),
        linear: &[f64],
    ) -> (Vec<f64>, Option<f64>) {
        let Some(link) = self.link else {
            return (linear.to_vec(), None);
        };
        let eta_high = self.eta(high.0, high.1);
        let eta_low = self.eta(low.0, low.1);
        let slope_high: Vec<f64> = eta_high.iter().map(|e| link.mean_slope(*e)).collect();
        let slope_low: Vec<f64> = eta_low.iter().map(|e| link.mean_slope(*e)).collect();
        let jacobian: Vec<f64> = self
            .columns_by(high.0, high.1, &slope_high)
            .iter()
            .zip(self.columns_by(low.0, low.1, &slope_low))
            .map(|(h, l)| h - l)
            .collect();
        let differences: Vec<f64> = eta_high
            .iter()
            .zip(&eta_low)
            .map(|(h, l)| link.mean(*h) - link.mean(*l))
            .collect();
        (
            jacobian,
            Some(average(&differences, self.weights.as_deref())),
        )
    }

    /// Z0414: `variable` is no variable of the model.
    fn unknown(&self, ctx: &CallContext<'_>, variable: &str) -> Fail {
        let variables = self.design.model_variables();
        let listed = variables
            .iter()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let hint = match did_you_mean(variable, variables.iter().map(String::as_str)) {
            Some(near) => format!("did you mean `{near}`? "),
            None => String::new(),
        };
        fail(
            Diagnostic::error(
                "Z0414",
                format!("`{variable}` is not a variable of this model"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "no term of the model is a function of it")
            .help(format!("{hint}the variables of the model are {listed}")),
        )
    }

    /// The difference between each level of a category and the first: the model matrix with
    /// every row set to the level, less the matrix with every row set to the first.
    fn levels(&self, variable: &str, source: usize, levels: &[String]) -> Vec<Effect> {
        let columns = self.design.columns_of(variable);
        let k = self.design.names.len();
        let held_at = |level: usize| -> Vec<Option<Vec<f64>>> {
            let mut held = self.held.clone();
            if held.len() <= source {
                held.resize(source + 1, None);
            }
            let mut shares = vec![0.0; levels.len()];
            shares[level] = 1.0;
            held[source] = Some(shares);
            held
        };
        let base = held_at(0);
        let low: Vec<Vec<f64>> = columns
            .iter()
            .map(|&c| self.design.column_held(c, &base))
            .collect();
        (1..levels.len())
            .map(|level| {
                let held = held_at(level);
                let mut jacobian = vec![0.0; k];
                for (at, &column) in columns.iter().enumerate() {
                    let high = self.design.column_held(column, &held);
                    let difference: Vec<f64> =
                        high.iter().zip(&low[at]).map(|(h, l)| h - l).collect();
                    jacobian[column] = average(&difference, self.weights.as_deref());
                }
                let linear = jacobian;
                let (jacobian, estimate) =
                    self.difference((&self.design, &held), (&self.design, &base), &linear);
                Effect {
                    term: format!("{variable}={}", level_name(&levels[level])),
                    variable: variable.to_string(),
                    contrast: format!("{} \u{2212} {}", levels[level], levels[0]),
                    jacobian,
                    linear,
                    estimate,
                    prediction: false,
                }
            })
            .collect()
    }

    /// The model matrix with the numeric `variable` replaced by `column` in every row.
    fn design_with(
        &self,
        ctx: &mut CallContext<'_>,
        variable: &str,
        table: Option<Columns>,
    ) -> Result<Design, Fail> {
        let Some(table) = table else {
            return Err(internal(
                ctx,
                "a variable of the model that the frame lacks",
            ));
        };
        design_on(ctx, self.fit, table).map_err(|_| {
            self.no_effect(
                ctx,
                variable,
                "a term of the model cannot be computed at another value of it".to_string(),
            )
        })
    }

    /// The difference between the predictions at one and at nought of a 0/1 variable.
    fn two_values(
        &self,
        ctx: &mut CallContext<'_>,
        variable: &str,
        contrast: String,
    ) -> Result<Effect, Fail> {
        let high = self.design_with(ctx, variable, set_to(&self.table, variable, 1.0))?;
        let low = self.design_with(ctx, variable, set_to(&self.table, variable, 0.0))?;
        let mut jacobian = vec![0.0; self.design.names.len()];
        for column in self.design.columns_of(variable) {
            let difference: Vec<f64> = high
                .column_held(column, &self.held)
                .iter()
                .zip(low.column_held(column, &self.held))
                .map(|(h, l)| h - l)
                .collect();
            jacobian[column] = average(&difference, self.weights.as_deref());
        }
        let linear = jacobian;
        let (jacobian, estimate) =
            self.difference((&high, &self.held), (&low, &self.held), &linear);
        Ok(Effect {
            term: variable.to_string(),
            variable: variable.to_string(),
            contrast,
            jacobian,
            linear,
            estimate,
            prediction: false,
        })
    }

    /// The slope in a number: the derivative of each column it enters, exactly where the
    /// column's term is arithmetic or a function with a known derivative, and by a central
    /// difference where it is not, which is said under the table.
    fn slope(
        &mut self,
        ctx: &mut CallContext<'_>,
        variable: &str,
        values: &[f64],
    ) -> Result<Effect, Fail> {
        let columns = self.design.columns_of(variable);
        let mut jacobian = vec![0.0; self.design.names.len()];
        let held: &Held = &self.held;
        // The model matrix a step above and a step below, built once if a column needs them.
        let mut shifted: Option<(Design, Design, f64)> = None;
        let mut by_difference: Vec<String> = Vec::new();
        // The derivative of each column the variable enters, row by row.
        let mut derivatives: Vec<(usize, Vec<f64>)> = Vec::new();
        for &column in &columns {
            let derivative = match self.design.derivative_held(column, variable, held) {
                Ok(Some(derivative)) => derivative,
                Ok(None) => continue,
                Err(NoDerivative::NotSmooth { .. }) => {
                    if shifted.is_none() {
                        let largest = values.iter().fold(1.0_f64, |m, v| m.max(v.abs()));
                        let step = STEP * largest;
                        let moved = |by: f64| -> Option<Columns> {
                            let column = Column::floats(values.iter().map(|v| Some(v + by))).0;
                            self.table.with_column(variable, column).ok()
                        };
                        let above = self.design_with(ctx, variable, moved(step))?;
                        let below = self.design_with(ctx, variable, moved(-step))?;
                        shifted = Some((above, below, step));
                    }
                    let (above, below, step) = shifted.as_ref().expect("built above");
                    by_difference.push(format!("`{}`", self.design.names[column]));
                    above
                        .column_held(column, held)
                        .iter()
                        .zip(below.column_held(column, held))
                        .map(|(a, b)| (a - b) / (2.0 * step))
                        .collect()
                }
                Err(NoDerivative::Category { column, .. }) => {
                    return Err(self.no_effect(
                        ctx,
                        variable,
                        format!("it enters `{column}` as a category"),
                    ));
                }
                Err(NoDerivative::Transformed) => {
                    return Err(internal(ctx, "a model matrix held as values in `margins`"));
                }
            };
            if derivative.iter().any(|d| !d.is_finite()) {
                return Err(self.no_effect(
                    ctx,
                    variable,
                    format!(
                        "the slope of `{}` is not finite where it is asked for",
                        self.design.names[column]
                    ),
                ));
            }
            jacobian[column] = average(&derivative, self.weights.as_deref());
            derivatives.push((column, derivative));
        }
        // What the slope was taken through, when that is more than the variable's own term:
        // the terms, each once, so that an interaction with a category is one name.
        let terms = self.design.terms();
        let mut through: Vec<String> = Vec::new();
        for &column in &columns {
            let label = terms
                .iter()
                .find(|term| term.columns.contains(&column))
                .map_or_else(
                    || self.design.names[column].clone(),
                    |term| term.label.clone(),
                );
            let shown = format!("`{label}`");
            if !through.contains(&shown) {
                through.push(shown);
            }
        }
        if through.len() > 1
            || through
                .first()
                .is_some_and(|t| *t != format!("`{variable}`"))
        {
            self.after.push(format!(
                "the slope of `{variable}` is taken through {}",
                join_and(&through)
            ));
        }
        if let Some((_, _, step)) = &shifted {
            self.after.push(format!(
                "the slope of `{variable}` through {} is a central difference with a step of {}, right where the term is smooth",
                join_and(&by_difference),
                significant(*step)
            ));
        }
        let linear = jacobian;
        let (jacobian, estimate) = match self.link {
            None => (linear.clone(), None),
            Some(link) => {
                // The slope of the linear predictor in the variable, then of the probability:
                // g′(η)·s, whose gradient in b is g″(η)·s·x + g′(η)·dx.
                let eta = self.eta(&self.design, &self.held);
                let mut s = vec![0.0; eta.len()];
                for (column, derivative) in &derivatives {
                    let b = self.beta[*column];
                    for (total, d) in s.iter_mut().zip(derivative) {
                        *total += b * d;
                    }
                }
                let slopes: Vec<f64> = eta.iter().map(|e| link.mean_slope(*e)).collect();
                let curved: Vec<f64> = eta
                    .iter()
                    .zip(&s)
                    .map(|(e, s)| link.mean_curve(*e) * s)
                    .collect();
                let mut jacobian = self.columns_by(&self.design, &self.held, &curved);
                for (column, derivative) in &derivatives {
                    let direct: Vec<f64> =
                        derivative.iter().zip(&slopes).map(|(d, g)| d * g).collect();
                    jacobian[*column] += average(&direct, self.weights.as_deref());
                }
                let effects: Vec<f64> = slopes.iter().zip(&s).map(|(g, s)| g * s).collect();
                (jacobian, Some(average(&effects, self.weights.as_deref())))
            }
        };
        Ok(Effect {
            term: variable.to_string(),
            variable: variable.to_string(),
            contrast: "dy/dx".to_string(),
            jacobian,
            linear,
            estimate,
            prediction: false,
        })
    }
}

/// `a`, `a and b`, `a, b and c`.
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// A small number to two significant digits: `0.00042`.
fn significant(x: f64) -> String {
    if x == 0.0 || !x.is_finite() {
        return fixed(x, 1);
    }
    let magnitude = zeff_numeric::maths::log10(x.abs()).floor() as i32;
    let decimals = (1 - magnitude).clamp(0, 15) as usize;
    fixed(x, decimals)
}

/// `95% CI`, `99.5% CI`.
fn level_heading(level: f64) -> String {
    format!("{} CI", level_text(level))
}

/// What `at:` holds one variable at.
#[derive(Clone, Copy)]
enum Setting {
    /// A number at a value.
    Number(f64),
    /// A category at a level: the source of its codes, the level and how many levels it has.
    Level {
        source: usize,
        level: usize,
        levels: usize,
    },
}

/// One field of `at:`: the variable and the values it is held at, one after another, each
/// with the value as the table prints it.
struct Held1 {
    name: String,
    settings: Vec<(Setting, String)>,
}

/// The fields of `at:` read against the model: each a variable of the model, a number held at
/// a number and a category at one of its levels, or at each of a list of them in turn.
fn read_at(
    ctx: &CallContext<'_>,
    design: &Design,
    value: Option<&Value>,
) -> Result<Vec<Held1>, Fail> {
    let refuse = |code: &'static str, message: String, help: String| -> Fail {
        fail(
            Diagnostic::error(code, message)
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "in `at:`")
                .help(help),
        )
    };
    let fields = match value {
        None | Some(Value::Na) => return Ok(Vec::new()),
        Some(Value::Record(fields)) => fields,
        Some(other) => {
            return Err(refuse(
                "Z0401",
                format!(
                    "`at:` of `margins` is a record of values, but this is {}",
                    other.type_word()
                ),
                "write the values to hold variables at as a record: `at: {exper: 10, industry: \"Mining\"}`".to_string(),
            ));
        }
    };
    let mut held = Vec::new();
    for (name, value) in fields {
        // One value, or a list of them: the effects are then given at each (step 12b).
        let values: Vec<&Value> = match value {
            Value::List(items) => items.iter().collect(),
            one => vec![one],
        };
        if values.is_empty() || values.iter().any(|v| matches!(v, Value::List(_))) {
            return Err(refuse(
                "Z0401",
                format!("`at:` holds `{name}` at a value or at each value of a list of them"),
                format!("write `at: {{{name}: 10}}` or `at: {{{name}: [5, 10, 20]}}`"),
            ));
        }
        let mut settings: Vec<(Setting, String)> = Vec::new();
        for value in values {
            match design.role(name) {
                Role::Absent => {
                    let variables = design.model_variables();
                    let listed = variables
                        .iter()
                        .map(|v| format!("`{v}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let hint = match did_you_mean(name, variables.iter().map(String::as_str)) {
                        Some(near) => format!("did you mean `{near}`? "),
                        None => String::new(),
                    };
                    return Err(refuse(
                        "Z0414",
                        format!("`{name}` is not a variable of this model"),
                        format!("{hint}the variables of the model are {listed}"),
                    ));
                }
                Role::Mixed { through } => {
                    return Err(refuse(
                    "Z0417",
                    format!(
                        "`margins` cannot hold `{name}` at a value: it enters the model through the category `{through}` and through another term"
                    ),
                    "hold a variable that is a number in every term, or a category in every term"
                        .to_string(),
                ));
                }
                Role::Number => {
                    let number = match value {
                        Value::Int(i) => *i as f64,
                        Value::Float(x) => *x,
                        Value::Bool(b) => f64::from(u8::from(*b)),
                        other => {
                            return Err(refuse(
                                "Z0401",
                                format!(
                                    "`at:` holds the number `{name}` at a number, but this is {}",
                                    other.type_word()
                                ),
                                format!("write a number: `at: {{{name}: 10}}`"),
                            ));
                        }
                    };
                    if !number.is_finite() {
                        return Err(refuse(
                            "Z0401",
                            format!("`at:` holds `{name}` at a finite number"),
                            format!("write a number: `at: {{{name}: 10}}`"),
                        ));
                    }
                    settings.push((
                        Setting::Number(number),
                        match value {
                            Value::Bool(b) => b.to_string(),
                            _ => format_float(number),
                        },
                    ));
                }
                Role::Category { source, levels } => {
                    let wanted = match value {
                        Value::Str(text) | Value::Word(text) => text.clone(),
                        Value::Int(i) => i.to_string(),
                        Value::Float(x) => format_float(*x),
                        Value::Bool(b) => b.to_string(),
                        other => {
                            return Err(refuse(
                                "Z0401",
                                format!(
                                    "`at:` holds the category `{name}` at one of its levels, but this is {}",
                                    other.type_word()
                                ),
                                format!(
                                    "write the level as text: `at: {{{name}: \"{}\"}}`",
                                    levels[0]
                                ),
                            ));
                        }
                    };
                    let Some(level) = levels.iter().position(|l| *l == wanted) else {
                        let listed = levels
                            .iter()
                            .map(|l| format!("`{l}`"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let hint = match did_you_mean(&wanted, levels.iter().map(String::as_str)) {
                            Some(near) => format!("did you mean `{near}`? "),
                            None => String::new(),
                        };
                        return Err(refuse(
                            "Z0401",
                            format!("`{wanted}` is not a level of `{name}` in this model"),
                            format!("{hint}its levels are {listed}"),
                        ));
                    };
                    settings.push((
                        Setting::Level {
                            source,
                            level,
                            levels: levels.len(),
                        },
                        wanted,
                    ));
                }
            }
        }
        // A value given twice would give two rows of one name.
        for (i, (_, shown)) in settings.iter().enumerate() {
            if settings[..i].iter().any(|(_, earlier)| earlier == shown) {
                return Err(refuse(
                    "Z0401",
                    format!("`at:` gives `{name}` the value `{shown}` twice"),
                    "give each value once".to_string(),
                ));
            }
        }
        held.push(Held1 {
            name: name.clone(),
            settings,
        });
    }
    Ok(held)
}

/// `fit.margins(vars, at:, type:)`: the marginal effect of each variable named, a slope for a
/// number and a difference for a 0/1 variable or a category, averaged over the rows the fit
/// used (`type: ame`) or taken at the means (`type: mem`), the variables of `at:` held at its
/// values; with no variable named, the adjusted prediction. Each with its delta-method standard
/// error, its t on the fit's degrees of freedom and its interval at the fit's level.
pub fn margins(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = match args.get(0) {
        Some(Value::Fit(fit)) if fit.kind == "RegFit" || fit.kind == "GlmFit" => Rc::clone(fit),
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`margins` needs a `reg` or a `glm` fit, but this is {}",
                        other.map_or("nothing".to_string(), Value::type_word)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a `reg` or a `glm` fit"),
            ));
        }
    };
    // After `glm` the effects are on the scale of the probability, through the fit's link.
    let link = match fit.likelihood.as_deref() {
        Some(likelihood) => match Link::named(&likelihood.link) {
            Some(link) => Some(link),
            None => {
                return Err(internal(
                    ctx,
                    "a `glm` fit of a link `margins` does not know",
                ));
            }
        },
        None => None,
    };
    let mut names: Vec<String> = Vec::new();
    match args.get(1) {
        Some(Value::Vars(given)) => {
            for name in given {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        Some(Value::Var(name)) => names.push(name.clone()),
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`vars` of `margins` names variables of the model, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), other.type_word())
                .help("name one variable or a list of them, without quotes: `m.margins(educ)`, `m.margins([educ, exper])`"),
            ));
        }
        None => {}
    }
    let at_means = matches!(args.get(3), Some(Value::Word(word)) if word == "mem");
    let Some(source) = &fit.source else {
        return Err(internal(ctx, "a regression fit that keeps no frame"));
    };
    // With no variable named, the adjusted prediction (open decision #202 (b)), which after
    // fixed effects would need the level of each effect.
    if names.is_empty() && !fit.effects.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`margins` gives no adjusted prediction after `fe:`",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no variable named")
            .help("a prediction needs the level of each fixed effect; name the variables whose effects are wanted, which do not depend on the effects, or use `fitted` for each row's prediction"),
        ));
    }
    let observed = source.frame.table.take_rows(&fit.rows);
    let as_observed = design_on(ctx, &fit, observed.clone())?;
    let at = read_at(ctx, &as_observed, args.get(2))?;
    // The position of each column's coefficient among the fit's terms; `None` for a column
    // omitted as collinear, which has none.
    let position: Vec<Option<usize>> = as_observed
        .names
        .iter()
        .map(|name| fit.terms.iter().position(|t| t == name))
        .collect();
    let beta: Vec<f64> = position
        .iter()
        .map(|p| p.map_or(0.0, |p| fit.estimates[p]))
        .collect();
    // The rows the effects are taken at before `at:` sets anything, the categories held and
    // the weights of the rows.
    let mut table = observed.clone();
    let mut held: Vec<Option<Vec<f64>>> = vec![None; as_observed.source_count()];
    let mut weights: Option<Vec<f64>> = fit.weights.clone();
    if at_means {
        // One row: every number at its mean over the rows the fit used, by the fit's weights;
        // every category at the shares of its levels (open decision #202 (e)).
        let mut one = observed.take_rows(&[0]);
        for name in as_observed.model_variables() {
            match as_observed.role(&name) {
                Role::Number => {
                    let Some(values) = numbers_of(&observed, &name) else {
                        return Err(fail(
                            Diagnostic::error(
                                "Z0417",
                                format!("`margins` cannot hold `{name}` at its mean: it is not a number, a Bool or a category"),
                            )
                            .in_file(ctx.script)
                            .primary(ctx.span_of(3), "at the means")
                            .help("`type: ame`, the default, averages over the rows as they are"),
                        ));
                    };
                    let mean = average(&values, fit.weights.as_deref());
                    let Ok(replaced) = one.with_column(&name, Column::floats([Some(mean)]).0)
                    else {
                        return Err(internal(
                            ctx,
                            "a variable of the model that the frame lacks",
                        ));
                    };
                    one = replaced;
                }
                Role::Mixed { through } => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0417",
                            format!("`margins` cannot hold `{name}` at its mean: it enters the model through the category `{through}` and through another term"),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(3), "at the means")
                        .help("`type: ame`, the default, averages over the rows as they are"),
                    ));
                }
                Role::Category { .. } | Role::Absent => {}
            }
        }
        for (source, slot) in held.iter_mut().enumerate() {
            *slot = as_observed.level_shares(source, fit.weights.as_deref());
        }
        table = one;
        weights = None;
    }
    // The combinations of the values of `at:`, the first variable changing slowest; one, of
    // nothing, without `at:`. The variables given several values have a column in the table.
    let varying: Vec<usize> = (0..at.len())
        .filter(|&field| at[field].settings.len() > 1)
        .collect();
    let mut combinations: Vec<Vec<usize>> = vec![Vec::new()];
    for field in &at {
        combinations = combinations
            .iter()
            .flat_map(|chosen| {
                (0..field.settings.len()).map(move |value| {
                    let mut next = chosen.clone();
                    next.push(value);
                    next
                })
            })
            .collect();
    }
    let predicting = names.is_empty();
    let source_keys: Vec<String> = as_observed.source_keys().to_vec();
    let mut as_observed = Some(as_observed);
    // Each effect with the values of the varying variables it was taken at.
    let mut effects: Vec<(Vec<String>, Effect)> = Vec::new();
    let mut said: Vec<String> = Vec::new();
    for chosen in &combinations {
        let mut table = table.clone();
        let mut held = held.clone();
        for (field, &value) in at.iter().zip(chosen) {
            match field.settings[value].0 {
                Setting::Number(number) => {
                    let Some(set) = set_to(&table, &field.name, number) else {
                        return Err(internal(ctx, "a variable of `at:` that the frame lacks"));
                    };
                    table = set;
                }
                Setting::Level {
                    source,
                    level,
                    levels,
                } => {
                    let mut shares = vec![0.0; levels];
                    shares[level] = 1.0;
                    held[source] = Some(shares);
                }
            }
        }
        let design = match as_observed.take() {
            Some(design) if !at_means && at.is_empty() => design,
            _ => {
                let built = design_on(ctx, &fit, table.clone()).map_err(|_| {
                    fail(
                        Diagnostic::error(
                            "Z0417",
                            "`margins` cannot compute a term of the model at the values asked for",
                        )
                        .in_file(ctx.script)
                        .primary(ctx.callee_span, "a term with no value there")
                        .help("a term such as `log(x)` has no value where `x` is held at nought or below; `type: ame` without `at:` takes the rows as they are"),
                    )
                })?;
                if built.source_keys() != source_keys.as_slice() {
                    return Err(internal(
                        ctx,
                        "the model matrix rebuilt for `margins` has other sources than the fit's",
                    ));
                }
                built
            }
        };
        let mut model = Model {
            fit: &fit,
            observed: &observed,
            table,
            design,
            held,
            weights: weights.clone(),
            at: at.iter().map(|field| field.name.clone()).collect(),
            link,
            beta: beta.clone(),
            after: Vec::new(),
        };
        let here: Vec<String> = varying
            .iter()
            .map(|&field| at[field].settings[chosen[field]].1.clone())
            .collect();
        if predicting {
            effects.push((here.clone(), model.prediction()));
        }
        for name in &names {
            for effect in model.effects(ctx, name)? {
                effects.push((here.clone(), effect));
            }
        }
        for line in model.after {
            if !said.contains(&line) {
                said.push(line);
            }
        }
    }
    // The reference distribution of the fit's own table: Student's t on its degrees of freedom
    // for a regression, the standard normal for a fit by maximum likelihood.
    let reference = if link.is_some() {
        Distribution::Family {
            name: "normal",
            params: vec![("mean", 0.0), ("sd", 1.0)],
        }
    } else {
        Distribution::Family {
            name: "t",
            params: vec![("df", fit.df_residual as f64)],
        }
    };
    let statistic_of = |value: f64| -> Statistic {
        if link.is_some() {
            Statistic {
                name: "z".to_string(),
                value,
                df: Vec::new(),
            }
        } else {
            Statistic {
                name: "t".to_string(),
                value,
                df: vec![fit.df_residual as f64],
            }
        }
    };
    let quantile = families::quantile(&reference, 1.0 - (1.0 - fit.level) / 2.0);
    let k = fit.terms.len();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut terms: Vec<TermTest> = Vec::new();
    let mut not_estimable: Vec<String> = Vec::new();
    for (here, effect) in &effects {
        // The row's name: the term, and where several values of `at:` are asked for, the
        // values it was taken at (`exper @ exper=5`).
        let term = if here.is_empty() {
            effect.term.clone()
        } else {
            format!(
                "{} @ {}",
                effect.term,
                varying
                    .iter()
                    .zip(here)
                    .map(|(&field, value)| format!("{}={value}", at[field].name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        // An effect that rests on a coefficient the fit omitted as collinear is not estimable;
        // a prediction takes such a coefficient as nought, as `predict` does.
        let estimable = effect.prediction
            || effect
                .linear
                .iter()
                .zip(&position)
                .all(|(j, p)| *j == 0.0 || p.is_some());
        let lead = |effect: &Effect| -> Vec<String> {
            let mut cells = here.clone();
            if effect.prediction {
                cells.push(effect.term.clone());
            } else {
                cells.extend([effect.variable.clone(), effect.contrast.clone()]);
            }
            cells
        };
        let mut gradient = vec![0.0; k];
        for (j, p) in effect.jacobian.iter().zip(&position) {
            if let Some(p) = p {
                gradient[*p] = *j;
            }
        }
        let mut linear_estimate = 0.0;
        let mut variance = 0.0;
        for (a, weight) in gradient.iter().enumerate() {
            linear_estimate += weight * fit.estimates[a];
            let mut row = 0.0;
            for (b, other) in gradient.iter().enumerate() {
                row += fit.vcov[a * k + b] * other;
            }
            variance += weight * row;
        }
        let estimate = effect.estimate.unwrap_or(linear_estimate);
        let std_error = variance.max(0.0).sqrt();
        if !estimable || !estimate.is_finite() || !std_error.is_finite() || std_error == 0.0 {
            let na = || "NA".to_string();
            if !estimable && !not_estimable.contains(&format!("`{}`", effect.term)) {
                not_estimable.push(format!("`{}`", effect.term));
            }
            let shown = if estimable && estimate.is_finite() {
                fixed(estimate, DECIMALS)
            } else {
                na()
            };
            let mut cells = lead(effect);
            cells.extend([shown, na(), na(), na(), na()]);
            rows.push(cells);
            terms.push(TermTest {
                term: term.clone(),
                statistic: statistic_of(f64::NAN),
                p_value: f64::NAN,
                effect: None,
                estimate: Some(Estimate {
                    label: term,
                    value: if estimable { estimate } else { f64::NAN },
                    ci: None,
                    decimals: DECIMALS,
                }),
                std_error: None,
            });
            continue;
        }
        let t = estimate / std_error;
        let p = 2.0 * families::cdf(&reference, t.abs()).upper;
        let interval = (
            estimate - quantile * std_error,
            estimate + quantile * std_error,
        );
        let mut cells = lead(effect);
        cells.extend([
            fixed(estimate, DECIMALS),
            fixed(std_error, DECIMALS),
            fixed(t, 2),
            p_value_text(p),
            format!(
                "[{}, {}]",
                fixed(interval.0, DECIMALS),
                fixed(interval.1, DECIMALS)
            ),
        ]);
        rows.push(cells);
        terms.push(TermTest {
            term: term.clone(),
            statistic: statistic_of(t),
            p_value: p,
            effect: None,
            estimate: Some(Estimate {
                label: term,
                value: estimate,
                ci: Some(interval),
                decimals: DECIMALS,
            }),
            std_error: Some(std_error),
        });
    }
    let mut after = said;
    if !not_estimable.is_empty() {
        after.push(format!(
            "{} {} on a term the fit omitted as collinear, so {} not estimable",
            join_and(&not_estimable),
            if not_estimable.len() == 1 {
                "rests"
            } else {
                "rest"
            },
            if not_estimable.len() == 1 {
                "it is"
            } else {
                "they are"
            },
        ));
    }
    // The lines under the title: the model, when the title names the fit by its binding; what
    // the effects are averaged over; where the standard errors come from.
    let mut header = Vec::new();
    if fit.name.is_some() {
        header.push(match fit.likelihood.as_deref() {
            Some(likelihood) => format!(
                "{} \u{b7} {} \u{b7} {}, {}",
                fit.method, fit.formula, likelihood.family, likelihood.link
            ),
            None => format!("{} \u{b7} {}", fit.method, fit.formula),
        });
    }
    let observations = fixed_grouped(fit.n as f64, 0);
    // After `glm` everything is a probability, or a change of one.
    let scale = if link.is_some() {
        "on the scale of the probability, "
    } else {
        ""
    };
    let over = if at_means {
        format!("{scale}at the means over the {observations} observations the fit used")
    } else {
        format!("{scale}averaged over the {observations} observations the fit used")
    };
    header.push(match (fit.weight_kind, source.weights.as_deref()) {
        (Some(WeightKind::Frequency), Some(w)) => format!("{over}, the rows counted by `{w}`"),
        (Some(_), Some(w)) => format!("{over}, weighted by `{w}`"),
        _ => over,
    });
    if !at.is_empty() {
        // `at: exper = 10, educ = 12`; with several values of a variable,
        // `at: exper = 5, 10, 20; educ = 12`.
        let fields: Vec<String> = at
            .iter()
            .map(|field| {
                format!(
                    "{} = {}",
                    field.name,
                    field
                        .settings
                        .iter()
                        .map(|(_, shown)| shown.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();
        header.push(format!(
            "at: {}",
            fields.join(if varying.is_empty() { ", " } else { "; " })
        ));
    }
    header.push(format!(
        "standard errors: {}, by the delta method",
        fit.se_kind
    ));
    let title = format!(
        "margins \u{b7} {} \u{b7} {}",
        fit.name.clone().unwrap_or_else(|| fit.formula.clone()),
        match (predicting, at_means) {
            (false, false) => "average marginal effects",
            (false, true) => "marginal effects at the means",
            (true, false) => "average adjusted prediction",
            (true, true) => "adjusted prediction at the means",
        }
    );
    // A column for each variable of `at:` given several values, then what the row is.
    let mut columns: Vec<(String, Align)> = varying
        .iter()
        .map(|&field| {
            // Numbers to the right, levels to the left, as every table has them.
            let align = match at[field].settings[0].0 {
                Setting::Number(_) => Align::Right,
                Setting::Level { .. } => Align::Left,
            };
            (at[field].name.clone(), align)
        })
        .collect();
    if predicting {
        columns.push(("quantity".to_string(), Align::Left));
    } else {
        columns.extend([
            ("variable".to_string(), Align::Left),
            ("contrast".to_string(), Align::Left),
        ]);
    }
    columns.extend([
        ("estimate".to_string(), Align::Right),
        ("std.error".to_string(), Align::Right),
        (
            if link.is_some() { "z value" } else { "t value" }.to_string(),
            Align::Right,
        ),
        ("p value".to_string(), Align::Right),
        (level_heading(fit.level), Align::Left),
    ]);
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "margins".to_string(),
        title,
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns,
            rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        statistic: statistic_of(f64::NAN),
        p_value: f64::NAN,
        terms,
        effect: None,
        level: fit.level,
        alpha: None,
        after,
        notes: Vec::new(),
        n: fit.n,
        dropped: fit.dropped,
        follow_up: Some(Rc::new(FollowUp::Margins {
            se_kind: fit.se_kind.clone(),
        })),
    })))
}
