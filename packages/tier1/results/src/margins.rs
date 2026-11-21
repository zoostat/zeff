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

use crate::dependence::{Held, NoDerivative, Role};
use crate::design::Design;
use std::rc::Rc;
use zeff_columns::{Column, ColumnType, Table as Columns};
use zeff_core::display::{fixed, fixed_grouped, level_text, p_value_text};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Fit, Frame, Statistic, Table, TermTest, TestResult,
    Value,
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
    jacobian: Vec<f64>,
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

/// The fit's model matrix rebuilt on `table`, whose rows are the rows the fit used, in order:
/// the fit's columns, with the fit's levels for every category.
fn design_on(ctx: &mut CallContext<'_>, fit: &Fit, table: Columns) -> Result<Design, Fail> {
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
    if design.rows.len() != fit.rows.len() {
        return Err(internal(
            ctx,
            "the model matrix rebuilt for `margins` has other rows than the fit",
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

/// `table` with the numeric variable `name` set to `value` in every row, in the variable's own
/// type.
fn set_to(table: &Columns, name: &str, value: f64) -> Option<Columns> {
    let n = table.rows();
    let column = match table.column(name)?.ty() {
        ColumnType::Bool => Column::bools(std::iter::repeat_n(Some(value != 0.0), n)),
        ColumnType::Int => Column::ints(std::iter::repeat_n(Some(value as i64), n)),
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
    /// The rows the fit used, as a table.
    table: Columns,
    /// The model matrix on those rows.
    design: Design,
    weights: Option<&'a [f64]>,
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
            Role::Category { source, levels } => Ok(self.levels(variable, source, &levels)),
            Role::Number => {
                let Some(values) = numbers_of(&self.table, variable) else {
                    return Err(self.no_effect(
                        ctx,
                        variable,
                        "it is not a number, a Bool or a category".to_string(),
                    ));
                };
                let column = self
                    .table
                    .column(variable)
                    .expect("a variable with values");
                if column.ty() == ColumnType::Bool || values.iter().all(|v| *v == 0.0 || *v == 1.0)
                {
                    let contrast = values_named(column);
                    Ok(vec![self.two_values(ctx, variable, contrast)?])
                } else {
                    Ok(vec![self.slope(ctx, variable, &values)?])
                }
            }
        }
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
            let mut held: Vec<Option<Vec<f64>>> = vec![None; source + 1];
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
                    jacobian[column] = average(&difference, self.weights);
                }
                Effect {
                    term: format!("{variable}={}", level_name(&levels[level])),
                    variable: variable.to_string(),
                    contrast: format!("{} \u{2212} {}", levels[level], levels[0]),
                    jacobian,
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
                .column_held(column, &[])
                .iter()
                .zip(low.column_held(column, &[]))
                .map(|(h, l)| h - l)
                .collect();
            jacobian[column] = average(&difference, self.weights);
        }
        Ok(Effect {
            term: variable.to_string(),
            variable: variable.to_string(),
            contrast,
            jacobian,
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
        let held: &Held = &[];
        // The model matrix a step above and a step below, built once if a column needs them.
        let mut shifted: Option<(Design, Design, f64)> = None;
        let mut by_difference: Vec<String> = Vec::new();
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
                        "the slope of `{}` is not finite in some row the fit used",
                        self.design.names[column]
                    ),
                ));
            }
            jacobian[column] = average(&derivative, self.weights);
        }
        // What the slope was taken through, when that is more than the variable's own column.
        let through: Vec<String> = columns
            .iter()
            .map(|&c| format!("`{}`", self.design.names[c]))
            .collect();
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
        Ok(Effect {
            term: variable.to_string(),
            variable: variable.to_string(),
            contrast: "dy/dx".to_string(),
            jacobian,
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

/// `fit.margins(vars)`: the average marginal effect of each variable named, a slope for a
/// number and a difference for a 0/1 variable or a category, each with its delta-method
/// standard error, its t on the fit's degrees of freedom and its interval at the fit's level.
pub fn margins(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = match args.get(0) {
        Some(Value::Fit(fit)) if fit.kind == "RegFit" => Rc::clone(fit),
        Some(Value::Fit(fit)) if fit.kind == "GlmFit" => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    "`margins` does not yet read a `glm` fit",
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "a `glm` fit")
                .help("the effects of a binary model, on the scale of the probability, arrive with the next step of this release"),
            ));
        }
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
    if names.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0406",
                "`margins` needs the variables whose effects are wanted",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no variable named")
            .help("name one variable or a list of them: `m.margins(educ)`, `m.margins([educ, exper])`"),
        ));
    }
    let Some(source) = &fit.source else {
        return Err(internal(ctx, "a regression fit that keeps no frame"));
    };
    let table = source.frame.table.take_rows(&fit.rows);
    let design = design_on(ctx, &fit, table.clone())?;
    // The position of each column's coefficient among the fit's terms; `None` for a column
    // omitted as collinear, which has none.
    let position: Vec<Option<usize>> = design
        .names
        .iter()
        .map(|name| fit.terms.iter().position(|t| t == name))
        .collect();
    let mut model = Model {
        fit: &fit,
        table,
        design,
        weights: fit.weights.as_deref(),
        after: Vec::new(),
    };
    let mut effects: Vec<Effect> = Vec::new();
    for name in &names {
        effects.extend(model.effects(ctx, name)?);
    }
    // The reference distribution of the fit's own table: Student's t on its degrees of freedom.
    let reference = Distribution::Family {
        name: "t",
        params: vec![("df", fit.df_residual as f64)],
    };
    let quantile = families::quantile(&reference, 1.0 - (1.0 - fit.level) / 2.0);
    let k = fit.terms.len();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut terms: Vec<TermTest> = Vec::new();
    let mut not_estimable: Vec<String> = Vec::new();
    for effect in &effects {
        // An effect that rests on a coefficient the fit omitted as collinear is not estimable.
        let estimable = effect
            .jacobian
            .iter()
            .zip(&position)
            .all(|(j, p)| *j == 0.0 || p.is_some());
        let mut gradient = vec![0.0; k];
        for (j, p) in effect.jacobian.iter().zip(&position) {
            if let Some(p) = p {
                gradient[*p] = *j;
            }
        }
        let mut estimate = 0.0;
        let mut variance = 0.0;
        for (a, weight) in gradient.iter().enumerate() {
            estimate += weight * fit.estimates[a];
            let mut row = 0.0;
            for (b, other) in gradient.iter().enumerate() {
                row += fit.vcov[a * k + b] * other;
            }
            variance += weight * row;
        }
        let std_error = variance.max(0.0).sqrt();
        if !estimable || !estimate.is_finite() || !std_error.is_finite() || std_error == 0.0 {
            let na = || "NA".to_string();
            if !estimable {
                not_estimable.push(format!("`{}`", effect.term));
            }
            let shown = if estimable && estimate.is_finite() {
                fixed(estimate, DECIMALS)
            } else {
                na()
            };
            rows.push(vec![
                effect.variable.clone(),
                effect.contrast.clone(),
                shown,
                na(),
                na(),
                na(),
                na(),
            ]);
            terms.push(TermTest {
                term: effect.term.clone(),
                statistic: Statistic {
                    name: "t".to_string(),
                    value: f64::NAN,
                    df: vec![fit.df_residual as f64],
                },
                p_value: f64::NAN,
                effect: None,
                estimate: Some(Estimate {
                    label: effect.term.clone(),
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
        rows.push(vec![
            effect.variable.clone(),
            effect.contrast.clone(),
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
        terms.push(TermTest {
            term: effect.term.clone(),
            statistic: Statistic {
                name: "t".to_string(),
                value: t,
                df: vec![fit.df_residual as f64],
            },
            p_value: p,
            effect: None,
            estimate: Some(Estimate {
                label: effect.term.clone(),
                value: estimate,
                ci: Some(interval),
                decimals: DECIMALS,
            }),
            std_error: Some(std_error),
        });
    }
    let mut after = std::mem::take(&mut model.after);
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
        header.push(format!("{} \u{b7} {}", fit.method, fit.formula));
    }
    let observations = fixed_grouped(fit.n as f64, 0);
    header.push(match (fit.weight_kind, source.weights.as_deref()) {
        (Some(WeightKind::Frequency), Some(w)) => format!(
            "averaged over the {observations} observations the fit used, the rows counted by `{w}`"
        ),
        (Some(_), Some(w)) => {
            format!("averaged over the {observations} observations the fit used, weighted by `{w}`")
        }
        _ => format!("averaged over the {observations} observations the fit used"),
    });
    header.push(format!(
        "standard errors: {}, by the delta method",
        fit.se_kind
    ));
    let title = format!(
        "margins \u{b7} {} \u{b7} average marginal effects",
        fit.name.clone().unwrap_or_else(|| fit.formula.clone())
    );
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "margins".to_string(),
        title,
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("variable".to_string(), Align::Left),
                ("contrast".to_string(), Align::Left),
                ("estimate".to_string(), Align::Right),
                ("std.error".to_string(), Align::Right),
                ("t value".to_string(), Align::Right),
                ("p value".to_string(), Align::Right),
                (level_heading(fit.level), Align::Left),
            ],
            rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        statistic: Statistic {
            name: "t".to_string(),
            value: f64::NAN,
            df: vec![fit.df_residual as f64],
        },
        p_value: f64::NAN,
        terms,
        effect: None,
        level: fit.level,
        alpha: None,
        after,
        notes: Vec::new(),
        n: fit.n,
        dropped: fit.dropped,
        follow_up: None,
    })))
}
