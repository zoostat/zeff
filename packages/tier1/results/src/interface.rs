//! The result interface (Engine and Project Specification, section 8.1, rule 5; Language
//! Reference, sections 3.7, 3.8 and 10.2; release 0.4.0, step 15; questions 3 (d) and 4 as the
//! owner decided them): the six methods every result answers, on a fitted model or a test's
//! result. `summary` gives the printed summary as a `Table`, `estimates` a `Frame`, `std_errors`
//! and `p_values` a `Mapping` by term or one `Float` for a named term, `ci` a `Table` of
//! intervals or `[lower, upper]` for one term, and `n_obs` an `Int`. The estimator interface's
//! first part (step 16) follows: `coef`, `vcov`, `fit_measures`, `predict`, `residuals` and
//! `fitted`. The methods of a test's result alone (open decision #94; release 0.7.0, step 12b)
//! end the file: `statistic`, `df`, `effect_size`, `h0`, `h1` and `alternative`.

use crate::design::Design;
use crate::summary::rows;
use std::rc::Rc;
use zeff_columns::{Column, Table as Columns};
use zeff_core::display::{
    fixed, fixed_grouped, format_float, list_levels, test_result_after, test_result_before,
};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Distribution, Fit, Frame, Table, TestResult, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};
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
    // `level:` written out, or the level the fit was made at (open decision #67).
    let given = match args.get(2) {
        Some(Value::Float(l)) => Some(*l),
        Some(Value::Int(i)) => Some(*i as f64),
        _ => None,
    };
    if let Some(level) = given
        && !(level > 0.0 && level < 1.0)
    {
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
    let received = receiver(ctx, &args, "ci")?;
    let level = match &received {
        Receiver::Fit(fit) => given.unwrap_or(fit.level),
        Receiver::Test(test) => given.unwrap_or(test.level),
    };
    let title = format!("ci · {}", level_text(level));
    let columns = vec![
        ("term".to_string(), Align::Left),
        ("lower".to_string(), Align::Right),
        ("upper".to_string(), Align::Right),
    ];
    match received {
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
        // A test's result: the intervals of its estimate and its effect size, when it has them
        // (the tests of `stats`), and of the estimate of each of its rows (the comparisons of
        // `posthoc`); `test` and `compare` have none.
        Receiver::Test(test) => {
            let mut rows: Vec<Vec<String>> = Vec::new();
            for item in [&test.estimate, &test.effect].into_iter().flatten() {
                if let Some((lower, upper)) = item.ci {
                    rows.push(vec![
                        item.label.clone(),
                        fixed(lower, item.decimals),
                        fixed(upper, item.decimals),
                    ]);
                }
            }
            for row in &test.terms {
                if let Some(item) = &row.estimate
                    && let Some((lower, upper)) = item.ci
                {
                    rows.push(vec![
                        row.term.clone(),
                        fixed(lower, item.decimals),
                        fixed(upper, item.decimals),
                    ]);
                }
            }
            if rows.is_empty() {
                return Ok(Value::Na);
            }
            if (level - test.level).abs() > 1e-12 {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`ci` of a test's result gives the intervals the test was made at, {}, not at {}",
                            level_text(test.level),
                            level_text(level)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "another level")
                    .help(if test.method == "posthoc" {
                        "`posthoc` gives its intervals at 95% and takes no `level:`"
                    } else {
                        "run the test with `level:` for intervals at another level"
                    }),
                ));
            }
            Ok(Value::Table(Table {
                title,
                before: Vec::new(),
                columns: vec![
                    ("term".to_string(), Align::Left),
                    ("lower".to_string(), Align::Right),
                    ("upper".to_string(), Align::Right),
                ],
                rows,
                after: Vec::new(),
                more: Vec::new(),
                rules: Vec::new(),
            }))
        }
    }
}

/// A statistic's value as a frame holds it: `None` where the test has no statistic (the exact
/// tests), which `TestResult` carries as NaN.
fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

/// `fit.estimates()`: a frame of the terms with an estimate and their estimate, standard error,
/// statistic, degrees of freedom, p-value and interval at the fit's level; for a test's result
/// a row for its estimate with the statistic and the p-value, and one for its effect size.
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
            // The t statistics share the residual degrees of freedom; `df2` is for an F.
            (
                "df".to_string(),
                floats(vec![Some(fit.df_residual as f64); fit.terms.len()]),
            ),
            ("df2".to_string(), floats(vec![None; fit.terms.len()])),
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
        // A test's result (open decision #94): a row for its estimate with the statistic, the
        // degrees of freedom and the p-value (the estimate's label, or the statistic's name
        // where the test has no estimate: `test`, `compare`, `chi2_test`), and a row for its
        // effect size with its interval where it has one.
        // A test of several rows: a row per term with its statistic, degrees of freedom and
        // p-value (a factorial `anova`), and with its estimate, standard error and interval
        // where the row rests on one (a comparison of `posthoc`); then a row per term for its
        // effect size.
        Receiver::Test(test) if !test.terms.is_empty() => {
            let mut terms: Vec<Option<String>> = Vec::new();
            let mut values = Vec::new();
            let mut errors = Vec::new();
            let mut statistics = Vec::new();
            let mut dfs = Vec::new();
            let mut df2s = Vec::new();
            let mut p_values = Vec::new();
            let mut lowers = Vec::new();
            let mut uppers = Vec::new();
            for row in &test.terms {
                let interval = row.estimate.as_ref().and_then(|e| e.ci);
                terms.push(Some(row.term.clone()));
                values.push(row.estimate.as_ref().and_then(|e| finite(e.value)));
                errors.push(row.std_error.and_then(finite));
                statistics.push(finite(row.statistic.value));
                dfs.push(row.statistic.df.first().copied());
                df2s.push(row.statistic.df.get(1).copied());
                p_values.push(finite(row.p_value));
                lowers.push(interval.map(|c| c.0));
                uppers.push(interval.map(|c| c.1));
            }
            for row in &test.terms {
                let Some(effect) = &row.effect else {
                    continue;
                };
                terms.push(Some(format!("{} ({})", effect.label, row.term)));
                values.push(finite(effect.value));
                errors.push(None);
                statistics.push(None);
                dfs.push(None);
                df2s.push(None);
                p_values.push(None);
                lowers.push(effect.ci.map(|c| c.0));
                uppers.push(effect.ci.map(|c| c.1));
            }
            vec![
                (
                    "term".to_string(),
                    Column::strs(terms.iter().map(|t| t.as_deref())),
                ),
                ("estimate".to_string(), floats(values)),
                ("std_error".to_string(), floats(errors)),
                ("statistic".to_string(), floats(statistics)),
                ("df".to_string(), floats(dfs)),
                ("df2".to_string(), floats(df2s)),
                ("p_value".to_string(), floats(p_values)),
                ("ci_lower".to_string(), floats(lowers)),
                ("ci_upper".to_string(), floats(uppers)),
            ]
        }
        Receiver::Test(test) => {
            let first = test
                .estimate
                .as_ref()
                .map_or(test.statistic.name.clone(), |e| e.label.clone());
            let mut terms = vec![Some(first)];
            let mut values = vec![test.estimate.as_ref().map(|e| e.value)];
            let mut statistics = vec![finite(test.statistic.value)];
            let mut dfs = vec![test.statistic.df.first().copied()];
            let mut df2s = vec![test.statistic.df.get(1).copied()];
            let mut p_values = vec![Some(test.p_value)];
            let mut lowers = vec![test.estimate.as_ref().and_then(|e| e.ci).map(|c| c.0)];
            let mut uppers = vec![test.estimate.as_ref().and_then(|e| e.ci).map(|c| c.1)];
            if let Some(effect) = &test.effect {
                terms.push(Some(effect.label.clone()));
                values.push(Some(effect.value));
                statistics.push(None);
                dfs.push(None);
                df2s.push(None);
                p_values.push(None);
                lowers.push(effect.ci.map(|c| c.0));
                uppers.push(effect.ci.map(|c| c.1));
            }
            let rows = terms.len();
            vec![
                (
                    "term".to_string(),
                    Column::strs(terms.iter().map(|t| t.as_deref())),
                ),
                ("estimate".to_string(), floats(values)),
                ("std_error".to_string(), floats(vec![None; rows])),
                ("statistic".to_string(), floats(statistics)),
                ("df".to_string(), floats(dfs)),
                ("df2".to_string(), floats(df2s)),
                ("p_value".to_string(), floats(p_values)),
                ("ci_lower".to_string(), floats(lowers)),
                ("ci_upper".to_string(), floats(uppers)),
            ]
        }
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
/// do: −2ℓ + 2(k + 1) and −2ℓ + ln(N)(k + 1), with k the terms with an estimate and, with
/// fixed effects, the effects' parameters, as fixest counts them.
pub fn information_criteria(fit: &Fit) -> (f64, f64) {
    let parameters = (fit.terms.len() + fit.absorbed_parameters) as f64 + 1.0;
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
    let (columns, rows, more, rules) = match &test.table {
        Some(table) => (
            table.columns.clone(),
            table.rows.clone(),
            table.more.clone(),
            table.rules.clone(),
        ),
        None => (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
    };
    Table {
        title: test.title.clone(),
        before: test_result_before(test),
        columns,
        rows,
        after: test_result_after(test),
        more,
        rules,
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
    // A fit with fixed effects has no intercept column, its categories coded as with one.
    let design = Design::build_full(
        ctx,
        &newdata,
        &model,
        &[],
        &fit.levels,
        &[],
        !fit.effects.is_empty(),
    )?;
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
    // The fixed effects' part (section 7.2; fixest's `predict` with `fixef`): each row's level
    // of every effect looked up by its key, a level the fit never saw predicting `NA` with a
    // warning, as an unseen category level does.
    for effect in &fit.effects {
        let columns: Vec<&Column> = effect
            .variables
            .iter()
            .map(|name| {
                newdata.table.column(name).ok_or_else(|| {
                    let names: Vec<&str> =
                        newdata.table.names().iter().map(String::as_str).collect();
                    fail(no_variable(
                        ctx.script,
                        ctx.span_of(1),
                        &newdata.shown_name(),
                        name,
                        &names,
                    ))
                })
            })
            .collect::<Result<_, _>>()?;
        let index: std::collections::BTreeMap<&[zeff_columns::Key], usize> = effect
            .keys
            .iter()
            .enumerate()
            .map(|(g, key)| (key.as_slice(), g))
            .collect();
        let mut unseen = 0usize;
        for (row, value) in values.iter_mut().enumerate() {
            let Some(yhat) = value else { continue };
            let key: Vec<zeff_columns::Key> = columns
                .iter()
                .map(|c| crate::absorb::effect_key(c, row))
                .collect();
            match index.get(key.as_slice()) {
                Some(&g) => *yhat += effect.values[g],
                None => {
                    unseen += 1;
                    *value = None;
                }
            }
        }
        if unseen > 0 {
            let (hold, its) = if unseen == 1 {
                ("holds", "its prediction is")
            } else {
                ("hold", "their predictions are")
            };
            ctx.message(
                Diagnostic::warning(
                    "W0207",
                    format!(
                        "{} of newdata {hold} a level of `{}` the fit never saw, so {its} NA",
                        rows(unseen),
                        effect.name
                    ),
                )
                .help(format!(
                    "a fixed effect predicts only for the levels it was estimated on; the fit saw {} levels of `{}`",
                    count(effect.keys.len() as u64),
                    effect.name
                )),
            );
        }
    }
    Ok(Value::Column(Column::floats(values).0))
}

// ----- the methods of a test's result alone (open decision #94; release 0.7.0, step 12b) -----

/// The receiver of a method only a test's result answers: Z0401 on a fit, whose values the
/// estimator interface gives by term.
fn test_receiver(ctx: &CallContext<'_>, args: &Args, name: &str) -> Result<Rc<TestResult>, Fail> {
    match args.get(0) {
        Some(Value::TestResult(test)) => Ok(Rc::clone(test)),
        other => {
            let found = other.map_or("nothing".to_string(), Value::type_word);
            let mut d = Diagnostic::error(
                "Z0401",
                format!("`{name}` is a method of a test's result, but this is {found}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not a test's result");
            if matches!(other, Some(Value::Fit(_))) {
                d = d.help(
                    "a regression's values are read by term: `coef`, `std_errors`, `p_values` and `ci`; `estimates()` gives them all as a frame",
                );
            }
            Err(fail(d))
        }
    }
}

/// `t.statistic()`: the test statistic as a number; `NA` for a test that has none (the exact
/// tests).
pub fn statistic(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "statistic")?;
    Ok(finite(test.statistic.value).map_or(Value::Na, Value::Float))
}

/// `t.df()`: the degrees of freedom as a list of one or two numbers, in the order the statistic
/// writes them, `F(1, 18,179)`; `NA` for a statistic that has none.
pub fn df(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "df")?;
    if test.statistic.df.is_empty() {
        return Ok(Value::Na);
    }
    Ok(Value::List(
        test.statistic.df.iter().map(|d| Value::Float(*d)).collect(),
    ))
}

/// `t.effect_size()`: the effect size the test printed, as a number; `NA` for a test that has
/// none.
pub fn effect_size(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "effect_size")?;
    Ok(test
        .effect
        .as_ref()
        .map_or(Value::Na, |e| Value::Float(e.value)))
}

/// `t.h0()`: the null hypothesis as the sentence the test printed; `NA` for a result that prints
/// none (`test`, `compare`).
pub fn h0(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "h0")?;
    Ok(test
        .hypotheses
        .as_ref()
        .map_or(Value::Na, |h| Value::Str(h.h0.clone())))
}

/// `t.h1()`: the alternative hypothesis as the sentence the test printed; `NA` for a result
/// that prints none.
pub fn h1(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "h1")?;
    Ok(test
        .hypotheses
        .as_ref()
        .map_or(Value::Na, |h| Value::Str(h.h1.clone())))
}

/// `t.alternative()`: the alternative the p-value was computed for, as the word of the argument:
/// `"two_sided"`, `"less"` or `"greater"`.
pub fn alternative(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = test_receiver(ctx, &args, "alternative")?;
    Ok(Value::Str(test.alternative.word().to_string()))
}
