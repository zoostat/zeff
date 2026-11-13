//! `r_squared` and `influence` (catalogue rows 242 and 247; Language Reference, section 11.1;
//! Engine and Project Specification, section 8.3, point 10; release 0.4.0, step 17; questions 4
//! and 7 as the owner decided them): the fit measures of a regression as a record, and the
//! influence of each observation, its leverage, Cook's distance and externally studentized
//! residual, as a frame of the fitted frame's full length.

use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Fit, Frame, Table as PrintedTable, Value};
use zeff_diagnostics::Diagnostic;
use zeff_numeric::order::WeightKind;

/// The receiver of a method of regression fits.
fn regression(ctx: &CallContext<'_>, args: &Args, name: &str) -> Result<Rc<Fit>, Fail> {
    match args.get(0) {
        Some(Value::Fit(fit)) => Ok(Rc::clone(fit)),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` is a method of a regression fit, but this is {}",
                    other.map_or("nothing".to_string(), Value::type_word)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not a regression fit"),
        )),
    }
}

/// `fit.r_squared()`: R² and adjusted R² as a record (question 4), `NA` when the outcome has no
/// variation. With fixed effects (release 0.6.0, step 18) `r_squared` and `adjusted` are the
/// overall forms, the effects' fit included, and `within` and `adjusted_within` the forms of
/// the regression on the demeaned columns, as fixest's `r2`, `ar2`, `wr2` and `war2`.
pub fn r_squared(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, "r_squared")?;
    let measure = |x: Option<f64>| x.map_or(Value::Na, Value::Float);
    let mut fields = vec![
        ("r_squared".to_string(), measure(fit.r_squared)),
        ("adjusted".to_string(), measure(fit.adjusted_r_squared)),
    ];
    if !fit.effects.is_empty() {
        fields.push(("within".to_string(), measure(fit.r_squared_within)));
        fields.push((
            "adjusted_within".to_string(),
            measure(fit.adjusted_r_squared_within),
        ));
    }
    Ok(Value::Record(fields))
}

/// The variance inflation factor of each regressor (catalogue row 248; release 0.5.0, step
/// 21): 1 / (1 − R²ⱼ), with R²ⱼ from the regression of column j of the fitted design on the
/// other columns and the intercept, over the rows the fit used. Computed from the Gram matrix the
/// fit keeps: with G = XᵀX, the factor is (G⁻¹)ⱼⱼ times the centred sum of squares of column j,
/// Gⱼⱼ − G₀ⱼ² / G₀₀ with column 0 the intercept, or the plain Gⱼⱼ for a fit without one (the
/// uncentred factor Stata's `estat vif` gives after `noconstant`). Under weights the design is
/// the weighted one, as the fit's own variances are. The intercept has no factor; a term dropped
/// as collinear has no estimate and no row.
pub fn vifs(fit: &Fit) -> Vec<(String, f64)> {
    let k = fit.terms.len();
    let g = |i: usize, j: usize| fit.gram[i * k + j];
    let g_inverse = |i: usize| fit.inverse_gram[i * k + i];
    let intercept = fit.terms.iter().position(|t| t == "(intercept)");
    fit.terms
        .iter()
        .enumerate()
        .filter(|(j, _)| Some(*j) != intercept)
        .map(|(j, term)| {
            let centred = match intercept {
                Some(c) => g(j, j) - g(c, j) * g(c, j) / g(c, c),
                None => g(j, j),
            };
            (term.clone(), g_inverse(j) * centred)
        })
        .collect()
}

/// `fit.vif()`: the table of question 3 (b), `vif · m1` with the columns `term` and `vif` to
/// three decimals, one row per regressor column, the intercept left out as `car::vif` leaves it
/// out and a categorical regressor's columns each on their own row.
pub fn vif(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, "vif")?;
    let factors = vifs(&fit);
    let rows: Vec<Vec<String>> = factors
        .iter()
        .map(|(term, value)| vec![term.clone(), zeff_core::display::fixed(*value, 3)])
        .collect();
    let mut after = Vec::new();
    if factors.is_empty() {
        after.push("no regressor beside the intercept".to_string());
    }
    if !fit.omitted.is_empty() {
        after.push(format!(
            "omitted as collinear, with no factor: {}",
            fit.omitted.join(", ")
        ));
    }
    Ok(Value::Table(PrintedTable {
        title: format!(
            "vif · {}",
            fit.name.clone().unwrap_or_else(|| fit.formula.clone())
        ),
        before: Vec::new(),
        columns: vec![
            ("term".to_string(), Align::Left),
            ("vif".to_string(), Align::Right),
        ],
        rows,
        after,
        more: Vec::new(),
        rules: Vec::new(),
    }))
}

/// The influence of each row the fit used (question 7): its leverage, the diagonal of the hat
/// matrix; Cook's distance, (e / (σ(1 − h)))² · h / k; and the externally studentized residual,
/// e / (σ₍ᵢ₎ √(1 − h)) with σ₍ᵢ₎² = (RSS − e² / (1 − h)) / (N − k − 1), the error variance
/// without the row, as R's `hatvalues`, `cooks.distance` and `rstudent` give them. Under
/// analytic and sampling weights the residual is weighted by √w, as R's `weighted.residuals`
/// is; under frequency weights a row stands for w observations, each with leverage h / w and
/// the row's residual, as R computes them on the replicated rows. A row of leverage one has no
/// distance and no studentized residual.
pub fn measures(fit: &Fit) -> Vec<(f64, f64, f64)> {
    let k = fit.terms.len() as f64;
    let n = fit.n as f64;
    let df = n - k;
    let rss = fit.residual_sum_of_squares;
    (0..fit.rows.len())
        .map(|i| {
            let w = fit.weights.as_ref().map_or(1.0, |w| w[i]);
            let (h, e) = match fit.weight_kind {
                Some(WeightKind::Frequency) => (fit.leverage[i] / w, fit.residuals[i]),
                Some(_) => (fit.leverage[i], fit.residuals[i] * w.sqrt()),
                None => (fit.leverage[i], fit.residuals[i]),
            };
            if h >= 1.0 || df <= 1.0 {
                return (h, f64::NAN, f64::NAN);
            }
            let cooks = (e / (fit.sigma * (1.0 - h))).powi(2) * h / k;
            let sigma2_without = (rss - e * e / (1.0 - h)) / (df - 1.0);
            let studentized = if sigma2_without > 0.0 {
                e / ((1.0 - h).sqrt() * sigma2_without.sqrt())
            } else {
                f64::NAN
            };
            (h, cooks, studentized)
        })
        .collect()
}

/// `fit.influence()`: the frame of question 7, `leverage`, `cooks_distance` and `studentized`,
/// of the fitted frame's full length with `NA` in the rows the fit dropped.
pub fn influence(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, "influence")?;
    // With absorbed effects the leverage of the full design cannot be had from the demeaned
    // factorisation (the ruling of release 0.6.0, step 18), so the measures wait.
    if !fit.effects.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0304",
                "`influence` on a fit with fixed effects is not available in this release",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a fit with `fe:`")
            .help("the leverage, Cook's distance and studentized residuals of a fit with absorbed effects arrive with release 0.8.0; `residuals()` and `fitted()` are available now"),
        ));
    }
    let measured = measures(&fit);
    let full = |pick: &dyn Fn(&(f64, f64, f64)) -> f64| -> Column {
        let mut all: Vec<Option<f64>> = vec![None; fit.frame_rows];
        for (i, &row) in fit.rows.iter().enumerate() {
            let value = pick(&measured[i]);
            all[row] = if value.is_nan() { None } else { Some(value) };
        }
        Column::floats(all).0
    };
    let table = Table::new(vec![
        ("leverage".to_string(), full(&|m| m.0)),
        ("cooks_distance".to_string(), full(&|m| m.1)),
        ("studentized".to_string(), full(&|m| m.2)),
    ])
    .map_err(|e| {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (the influence frame: {e:?})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )
    })?;
    Ok(Value::Frame(Frame::new(table)))
}

/// The name of the variable of `fit.design()` that holds the weights of a weighted fit.
pub const DESIGN_WEIGHTS: &str = "(weights)";

/// `fit.design()` (open decision #71): the design the fit was computed on, as a frame of the
/// fitted frame's full length, a variable for each term of the fit under the term's name,
/// the intercept and the columns of the categories among them, with `NA` in the rows the fit
/// dropped; and for a weighted fit the weights it used, in `(weights)`. The values are those
/// of the model, unweighted; with fixed effects they are what is left of each column after
/// the effects are projected out, the design the estimates were computed on. A term omitted
/// as collinear has no estimate and no variable.
pub fn design(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, "design")?;
    let unbuilt = |ctx: &CallContext<'_>, why: &str| {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (the design of the fit: {why})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )
    };
    let Some(built) = crate::reg::design_of(ctx, &fit) else {
        return Err(unbuilt(ctx, "the fit keeps no frame"));
    };
    let built = built?;
    // The design built again is the fit's own: the same rows, in the same order.
    if built.rows != fit.rows {
        return Err(unbuilt(ctx, "its rows are not the fit's"));
    }
    let full = |values: &[f64]| -> Column {
        let mut all: Vec<Option<f64>> = vec![None; fit.frame_rows];
        for (i, &row) in fit.rows.iter().enumerate() {
            all[row] = Some(values[i]);
        }
        Column::floats(all).0
    };
    let mut columns: Vec<(String, Column)> = Vec::new();
    for term in &fit.terms {
        let Some(at) = built.names.iter().position(|name| name == term) else {
            return Err(unbuilt(ctx, "a term of the fit is not a column of it"));
        };
        columns.push((term.clone(), full(&built.column_values(at))));
    }
    if let Some(weights) = &fit.weights {
        columns.push((DESIGN_WEIGHTS.to_string(), full(weights)));
    }
    let table = Table::new(columns).map_err(|e| unbuilt(ctx, &format!("{e:?}")))?;
    Ok(Value::Frame(Frame::new(table)))
}
