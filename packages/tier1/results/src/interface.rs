//! The result interface (Engine and Project Specification, section 8.1, rule 5; Language
//! Reference, sections 3.7, 3.8 and 10.2; release 0.4.0, step 15; questions 3 (d) and 4 as the
//! owner decided them): the six methods every result answers, on a fitted model or a test's
//! result. `summary` gives the printed summary as a `Table`, `estimates` a `Frame`, `std_errors`
//! and `p_values` a `Mapping` by term or one `Float` for a named term, `ci` a `Table` of
//! intervals or `[lower, upper]` for one term, and `n_obs` an `Int`. The estimator interface's
//! first part (step 16) follows: `coef`, `vcov`, `fit_measures`, `predict`, `residuals` and
//! `fitted`.

use crate::design::Design;
use crate::summary::rows;
use std::rc::Rc;
use zeff_columns::{Column, Table as Columns};
use zeff_core::display::{fixed, fixed_grouped, format_float, list_levels, statistic_line};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Distribution, Fit, Frame, Table, TestResult, Value};
use zeff_diagnostics::Diagnostic;
use zeff_dist::families;

/// The receiver of a result method: a fitted model or a test's result.
enum Receiver {
    Fit(Rc<Fit>),
    Test(Rc<TestResult>),
}

fn receiver(ctx: &CallContext<'_>, args: &Args, name: &str) -> Result<Receiver, Fail> {
    match args.get(0) {
        Some(Value::Fit(fit)) => Ok(Receiver::Fit(Rc::clone(fit))),
        Some(Value::TestResult(test)) => Ok(Receiver::Test(Rc::clone(test))),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` is a method of a result, but this is {}",
                    other.map_or("nothing".to_string(), Value::type_word)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not a result"),
        )),
    }
}

/// The term a `Term` argument names, when one is given: its position among the terms with an
/// estimate, or `None` for a term omitted as collinear. The interpreter has already checked the
/// name against the fit's terms (Z0414).
fn named_term(fit: &Fit, value: Option<&Value>) -> Option<Option<usize>> {
    match value {
        Some(Value::Term(name)) => Some(fit.terms.iter().position(|t| t == name)),
        _ => None,
    }
}

/// A mapping from each term with an estimate to its value, in the model's order.
fn mapping(fit: &Fit, values: &[f64]) -> Value {
    Value::Mapping(
        fit.terms
            .iter()
            .zip(values)
            .map(|(t, v)| (Value::Str(t.clone()), Value::Float(*v)))
            .collect(),
    )
}

/// One value when a term is named (`NA` for an omitted term), the mapping over every term
/// otherwise (question 4).
fn by_term(fit: &Fit, args: &Args, values: &[f64]) -> Value {
    match named_term(fit, args.get(1)) {
        Some(Some(i)) => Value::Float(values[i]),
        Some(None) => Value::Na,
        None => mapping(fit, values),
    }
}

/// `fit.std_errors(term?)`: the standard errors by term, or one; a test's result has none.
pub fn std_errors(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "std_errors")? {
        Receiver::Fit(fit) => Ok(by_term(&fit, &args, &fit.std_errors)),
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.p_values(term?)`: the p-values by term, or one; a test's result gives its single
/// p-value.
pub fn p_values(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "p_values")? {
        Receiver::Fit(fit) => Ok(by_term(&fit, &args, &fit.p_values)),
        Receiver::Test(test) => Ok(Value::Float(test.p_value)),
    }
}

/// `fit.n_obs()`: the observations used, after the rows with `NA` are dropped (the sum of the
/// frequency weights under them).
pub fn n_obs(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "n_obs")? {
        Receiver::Fit(fit) => Ok(Value::Int(fit.n as i64)),
        Receiver::Test(test) => Ok(Value::Int(test.n as i64)),
    }
}

/// The t quantile of the fit's degrees of freedom at a two-sided `level`.
fn quantile(df: usize, level: f64) -> f64 {
    let student = Distribution::Family {
        name: "t",
        params: vec![("df", df as f64)],
    };
    families::quantile(&student, 1.0 - (1.0 - level) / 2.0)
}

/// The confidence level as a title writes it: `95%`, `99.9%`.
fn level_text(level: f64) -> String {
    let percent = level * 100.0;
    if (percent - percent.round()).abs() < 1e-9 {
        format!("{}%", percent.round() as i64)
    } else {
        format!("{}%", fixed(percent, 1))
    }
}

/// `fit.ci(term?, level: 0.95)`: the intervals of every term as a table, or `[lower, upper]` for
/// one term, at the level asked for; recomputed from the estimates, the standard errors and the
/// t table of the fit, so that any level is available after the fit. A test's result gives `NA`.
pub fn ci(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let level = match args.get(2) {
        Some(Value::Float(l)) => *l,
        Some(Value::Int(i)) => *i as f64,
        _ => 0.95,
    };
    if !(level > 0.0 && level < 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`level:` is a probability strictly between 0 and 1, but this is {level}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "outside (0, 1)")
            .help("write a level such as 0.95 or 0.99"),
        ));
    }
    let title = format!("ci · {}", level_text(level));
    let columns = vec![
        ("term".to_string(), Align::Left),
        ("lower".to_string(), Align::Right),
        ("upper".to_string(), Align::Right),
    ];
    match receiver(ctx, &args, "ci")? {
        Receiver::Fit(fit) => {
            let q = quantile(fit.df_residual, level);
            let intervals: Vec<(f64, f64)> = fit
                .estimates
                .iter()
                .zip(&fit.std_errors)
                .map(|(b, se)| (b - q * se, b + q * se))
                .collect();
            Ok(match named_term(&fit, args.get(1)) {
                Some(Some(i)) => Value::List(vec![
                    Value::Float(intervals[i].0),
                    Value::Float(intervals[i].1),
                ]),
                Some(None) => Value::Na,
                None => Value::Table(Table {
                    title,
                    before: Vec::new(),
                    columns,
                    rows: fit
                        .terms
                        .iter()
                        .zip(&intervals)
                        .map(|(t, (lo, hi))| vec![t.clone(), fixed(*lo, 4), fixed(*hi, 4)])
                        .collect(),
                    after: Vec::new(),
                    more: Vec::new(),
                    rules: Vec::new(),
                }),
            })
        }
        // A test's result has no intervals at this release (the tests of `stats`, 0.7.0, add
        // the interval of their estimate).
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.estimates()`: a frame of the terms with an estimate and their estimate, standard error,
/// statistic, p-value and interval at the fit's level; for a test's result one row with its
/// statistic and p-value.
pub fn estimates(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let floats = |values: Vec<Option<f64>>| Column::floats(values).0;
    let columns = match receiver(ctx, &args, "estimates")? {
        Receiver::Fit(fit) => vec![
            (
                "term".to_string(),
                Column::strs(fit.terms.iter().map(|t| Some(t.as_str()))),
            ),
            (
                "estimate".to_string(),
                floats(fit.estimates.iter().map(|v| Some(*v)).collect()),
            ),
            (
                "std_error".to_string(),
                floats(fit.std_errors.iter().map(|v| Some(*v)).collect()),
            ),
            (
                "statistic".to_string(),
                floats(fit.statistics.iter().map(|v| Some(*v)).collect()),
            ),
            (
                "p_value".to_string(),
                floats(fit.p_values.iter().map(|v| Some(*v)).collect()),
            ),
            (
                "ci_lower".to_string(),
                floats(fit.intervals.iter().map(|v| Some(v.0)).collect()),
            ),
            (
                "ci_upper".to_string(),
                floats(fit.intervals.iter().map(|v| Some(v.1)).collect()),
            ),
        ],
        Receiver::Test(test) => vec![
            (
                "term".to_string(),
                Column::strs([Some(test.statistic.name.as_str())]),
            ),
            ("estimate".to_string(), floats(vec![None])),
            ("std_error".to_string(), floats(vec![None])),
            (
                "statistic".to_string(),
                floats(vec![Some(test.statistic.value)]),
            ),
            ("p_value".to_string(), floats(vec![Some(test.p_value)])),
            ("ci_lower".to_string(), floats(vec![None])),
            ("ci_upper".to_string(), floats(vec![None])),
        ],
    };
    let table = Columns::new(columns).map_err(|e| {
        fail(
            Diagnostic::error("Z9001", format!("internal error: this is a bug in Zeff, not in your script (the estimates frame: {e:?})"))
                .in_file(ctx.script)
                .primary(ctx.callee_span, "the call that failed"),
        )
    })?;
    Ok(Value::Frame(Frame::new(table)))
}

/// The standardized coefficients of question 3 (d): each estimate times the standard deviation
/// of its regressor over that of the outcome; `None` for a column without variation (the
/// intercept) or an outcome without.
pub fn standardized(fit: &Fit) -> Vec<Option<f64>> {
    fit.estimates
        .iter()
        .zip(&fit.term_sd)
        .map(|(b, sd)| {
            if *sd > 0.0 && fit.outcome_sd > 0.0 {
                Some(b * sd / fit.outcome_sd)
            } else {
                None
            }
        })
        .collect()
}

/// Akaike's and Schwarz's criteria, counting the variance as a parameter as R's `AIC` and `BIC`
/// do: −2ℓ + 2(k + 1) and −2ℓ + ln(N)(k + 1), with k the terms with an estimate.
pub fn information_criteria(fit: &Fit) -> (f64, f64) {
    let parameters = fit.terms.len() as f64 + 1.0;
    let aic = -2.0 * fit.log_likelihood + 2.0 * parameters;
    let bic = -2.0 * fit.log_likelihood + zeff_numeric::maths::log(fit.n as f64) * parameters;
    (aic, bic)
}

/// The column `beta` after `estimate` (question 3 (d)), blank for the intercept and for an
/// omitted term.
fn insert_betas(fit: &Fit, table: &mut Table) {
    let betas = standardized(fit);
    let at = table
        .columns
        .iter()
        .position(|(h, _)| h == "estimate")
        .map_or(table.columns.len(), |i| i + 1);
    table.columns.insert(at, ("beta".to_string(), Align::Right));
    for row in &mut table.rows {
        let text = fit
            .terms
            .iter()
            .position(|t| *t == row[0])
            .and_then(|i| betas[i])
            .map_or(String::new(), |b| fixed(b, 4));
        if row.len() >= at {
            row.insert(at, text);
        } else {
            row.push(text);
        }
    }
}

/// The line of question 3 (d): `log-likelihood -12,034.2 · AIC 24,078.4 · BIC 24,117.5`.
fn fit_measures_line(fit: &Fit) -> String {
    let (aic, bic) = information_criteria(fit);
    format!(
        "log-likelihood {} · AIC {} · BIC {}",
        fixed_grouped(fit.log_likelihood, 1),
        fixed_grouped(aic, 1),
        fixed_grouped(bic, 1)
    )
}

/// A test's result as a table: its title, its header lines, its table if it has one, and its
/// statistic line with the lines after it.
fn test_table(test: &TestResult) -> Table {
    let (columns, rows) = match &test.table {
        Some(table) => (table.columns.clone(), table.rows.clone()),
        None => (Vec::new(), Vec::new()),
    };
    let mut after = vec![statistic_line(&test.statistic, test.p_value)];
    after.extend(test.after.iter().cloned());
    Table {
        title: test.title.clone(),
        before: test.header.clone(),
        columns,
        rows,
        after,
        more: Vec::new(),
        rules: Vec::new(),
    }
}

/// `fit.summary(standardized: false, fit_measures: false)`: the printed summary as a table, with
/// the column of standardized coefficients and the line of fit measures when asked for; a test's
/// result gives its printed form.
pub fn summary(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let flag = |i: usize| matches!(args.get(i), Some(Value::Bool(true)));
    let (standardized, fit_measures) = (flag(1), flag(2));
    match receiver(ctx, &args, "summary")? {
        Receiver::Fit(fit) => {
            let mut table = fit.summary.clone();
            if standardized {
                insert_betas(&fit, &mut table);
            }
            if fit_measures {
                table.after.push(fit_measures_line(&fit));
            }
            Ok(Value::Table(table))
        }
        Receiver::Test(test) => Ok(Value::Table(test_table(&test))),
    }
}

// ----- the estimator interface, part 1 (step 16) -----

/// `fit.coef(term?)`: the estimates by term, or one; a test's result has none.
pub fn coef(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "coef")? {
        Receiver::Fit(fit) => Ok(by_term(&fit, &args, &fit.estimates)),
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.vcov()`: the variance matrix of the estimates as a table over the terms with an
/// estimate, titled with the kind of standard error it was computed under; a test's result has
/// none.
pub fn vcov(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "vcov")? {
        Receiver::Fit(fit) => {
            let k = fit.terms.len();
            let mut columns = vec![("term".to_string(), Align::Left)];
            columns.extend(fit.terms.iter().map(|t| (t.clone(), Align::Right)));
            let rows = (0..k)
                .map(|i| {
                    let mut row = vec![fit.terms[i].clone()];
                    row.extend((0..k).map(|j| format_float(fit.vcov[i * k + j])));
                    row
                })
                .collect();
            Ok(Value::Table(Table {
                title: format!("vcov · {}", fit.se_kind),
                before: Vec::new(),
                columns,
                rows,
                after: Vec::new(),
                more: Vec::new(),
                rules: Vec::new(),
            }))
        }
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.fit_measures()`: the log-likelihood, AIC, BIC, R² and adjusted R² as a record (question
/// 4); R² is `NA` when the outcome has no variation. A test's result has none.
pub fn fit_measures(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "fit_measures")? {
        Receiver::Fit(fit) => {
            let (aic, bic) = information_criteria(&fit);
            let measure = |x: Option<f64>| x.map_or(Value::Na, Value::Float);
            Ok(Value::Record(vec![
                (
                    "log_likelihood".to_string(),
                    Value::Float(fit.log_likelihood),
                ),
                ("aic".to_string(), Value::Float(aic)),
                ("bic".to_string(), Value::Float(bic)),
                ("r_squared".to_string(), measure(fit.r_squared)),
                ("adjusted".to_string(), measure(fit.adjusted_r_squared)),
            ]))
        }
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// A column of the fitted frame's full length: `values` in the rows the fit used, `NA` in the
/// rows it dropped (question 4, as R's `na.exclude` and Stata's `predict` fill them).
fn full_length(fit: &Fit, values: &[f64]) -> Value {
    let mut all: Vec<Option<f64>> = vec![None; fit.frame_rows];
    for (i, &row) in fit.rows.iter().enumerate() {
        all[row] = Some(values[i]);
    }
    Value::Column(Column::floats(all).0)
}

/// `fit.residuals()`: the residuals, one per row of the fitted frame.
pub fn residuals(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "residuals")? {
        Receiver::Fit(fit) => Ok(full_length(&fit, &fit.residuals)),
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.fitted()`: the fitted values, one per row of the fitted frame.
pub fn fitted(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match receiver(ctx, &args, "fitted")? {
        Receiver::Fit(fit) => Ok(full_length(&fit, &fit.fitted)),
        Receiver::Test(_) => Ok(Value::Na),
    }
}

/// `fit.predict(newdata:)`: the model's prediction for each row of `newdata`, as a column of its
/// length. The design is rebuilt on the new frame from the fit's formula without its outcome,
/// with the fit's category levels, so that the columns are the fit's: a variable the frame
/// lacks is Z0101, a row with `NA` in a model variable predicts `NA`, and a row whose level the
/// fit never saw predicts `NA` with a warning (W0207). A term omitted as collinear contributes
/// nothing, as R's `predict` on a rank-deficient fit does.
pub fn predict(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let newdata = match args.get(1) {
        Some(Value::Frame(frame)) => frame.clone(),
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`newdata:` of `predict` must be a Frame, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), other.type_word()),
            ));
        }
        None => {
            return Err(fail(
                Diagnostic::error(
                    "Z0406",
                    "`predict` needs `newdata:`, the frame to predict for",
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`newdata:` is missing")
                .help("`fitted()` gives the predictions for the rows the fit used"),
            ));
        }
    };
    let fit = match receiver(ctx, &args, "predict")? {
        Receiver::Fit(fit) => fit,
        Receiver::Test(_) => return Ok(Value::Na),
    };
    let mut model = fit.model.clone();
    model.outcome = None;
    let design = Design::build_with(ctx, &newdata, &model, &[], &fit.levels)?;
    for (var, n) in &design.unseen {
        let levels = fit
            .levels
            .iter()
            .find(|(v, _)| v == var)
            .map(|(_, l)| l.as_slice())
            .unwrap_or(&[]);
        let (hold, its, is) = if *n == 1 {
            ("holds", "its prediction is", "")
        } else {
            ("hold", "their predictions are", "")
        };
        let _ = is;
        ctx.message(
            Diagnostic::warning(
                "W0207",
                format!(
                    "{} of newdata {hold} a level of `{var}` the fit never saw, so {its} NA",
                    rows(*n)
                ),
            )
            .help(format!(
                "the fit's levels of `{var}` are {}",
                list_levels(levels)
            )),
        );
    }
    let estimate_of: Vec<Option<f64>> = design
        .names
        .iter()
        .map(|name| {
            fit.terms
                .iter()
                .position(|t| t == name)
                .map(|i| fit.estimates[i])
        })
        .collect();
    if let Some(name) = design
        .names
        .iter()
        .find(|n| !fit.terms.contains(n) && !fit.omitted.contains(n))
    {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (the new design has a column `{name}` the fit does not)"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    }
    let mut values: Vec<Option<f64>> = vec![None; newdata.table.rows()];
    for block in design.blocks() {
        for r in 0..block.x.rows() {
            let mut yhat = 0.0;
            for (c, estimate) in estimate_of.iter().enumerate() {
                if let Some(b) = estimate {
                    yhat += b * block.x.get(r, c);
                }
            }
            values[design.rows[block.start + r]] = Some(yhat);
        }
    }
    Ok(Value::Column(Column::floats(values).0))
}
