//! The plots of results (Language Reference, section 7.6, "Result plots"; release 0.7.0,
//! step 30): `fit.coef_plot()`, the estimates of a fit as points with their confidence
//! intervals, and `fit.plot()`, the default view of a result. They are built from the fit's
//! own estimates with the layers of the grammar, `points` and `errorbars`, so that what is
//! drawn is what the fit printed; the intervals record what they are in the scene, and the
//! figure says it (decision #118, F11).
//!
//! By decision #120 a derived plot of a result is a method named `<kind>_plot`, one for each
//! kind of figure, and `plot` is each result type's default view.

use crate::grammar::{internal, plot};
use crate::scene::describe;
use crate::shorthands::{NONE, call};
use crate::{marks, scales};
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Fit, Frame, Plot, Stat, Value};
use zeff_diagnostics::Diagnostic;

/// The variables of the frame a coefficient plot draws: the term, its estimate and the two
/// limits of its confidence interval.
pub const COEF_TERM: &str = "term";
pub const COEF_ESTIMATE: &str = "estimate";
pub const COEF_LOWER: &str = "ci_lower";
pub const COEF_UPPER: &str = "ci_upper";

/// The name of the intercept among the terms of a fit.
const INTERCEPT: &str = "(intercept)";

/// The fit a result plot is of: a fitted model, and nothing else.
fn fitted(ctx: &CallContext<'_>, args: &Args, name: &str) -> Result<Rc<Fit>, Fail> {
    match args.get(0) {
        Some(Value::Fit(fit)) => Ok(Rc::clone(fit)),
        Some(Value::TestResult(_)) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{name}` draws the coefficients of a fitted model, and a test has none"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "the result of a test")
            .help("a test prints its statistic and its p-value; fit a model with `reg` to plot its coefficients"),
        )),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` is a method of a fitted model, but this is {}",
                    other.map_or("nothing".to_string(), Value::type_word)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not a fitted model"),
        )),
    }
}

/// The plot of the estimates of the terms at `kept`, in their order: a point at each
/// estimate and its confidence interval at the fit's level, a term on each line, the first
/// at the top, and a reference line at zero.
pub(crate) fn coefficients(
    ctx: &mut CallContext<'_>,
    fit: &Fit,
    kept: &[usize],
) -> Result<Value, Fail> {
    let unbuilt =
        |ctx: &CallContext<'_>| internal(ctx, "the frame of the coefficients could not be built");
    // The terms are categories in the order of the table, turned round: the first place of
    // an axis of categories is at the foot, and the first term stands at the top.
    let names: Vec<String> = kept.iter().map(|&at| fit.terms[at].clone()).collect();
    let upwards: Vec<String> = names.iter().rev().cloned().collect();
    let codes = (0..names.len()).map(|at| Some((names.len() - 1 - at) as i32));
    let Ok(term) = Column::categories(upwards, codes) else {
        return Err(unbuilt(ctx));
    };
    let of = |pick: &dyn Fn(usize) -> f64| Column::floats(kept.iter().map(|&at| Some(pick(at)))).0;
    let Ok(table) = Table::new(vec![
        (COEF_TERM.to_string(), term),
        (COEF_ESTIMATE.to_string(), of(&|at| fit.estimates[at])),
        (COEF_LOWER.to_string(), of(&|at| fit.intervals[at].0)),
        (COEF_UPPER.to_string(), of(&|at| fit.intervals[at].1)),
    ]) else {
        return Err(unbuilt(ctx));
    };
    let estimates = Frame {
        table: Rc::new(table),
        name: fit.name.clone(),
    };
    let var = |name: &str| (Some(Value::Var(name.to_string())), None);
    let mut drawn = call(
        ctx,
        plot,
        vec![
            (
                Some(Value::Frame(estimates)),
                ctx.arg_spans.first().copied().flatten(),
            ),
            var(COEF_TERM),
            var(COEF_ESTIMATE),
        ],
    )?;
    // The line of no effect, under the intervals.
    drawn = call(
        ctx,
        marks::hline,
        vec![(Some(drawn), None), NONE, (Some(Value::Float(0.0)), None)],
    )?;
    drawn = call(
        ctx,
        marks::errorbars,
        vec![
            (Some(drawn), None),
            NONE,
            var(COEF_LOWER),
            var(COEF_UPPER),
            (Some(Value::Float(0.0)), None),
        ],
    )?;
    // The intervals are the fit's confidence intervals, at its level: the layer says so.
    let Value::Plot(with_intervals) = drawn else {
        return Err(internal(ctx, "`errorbars` gave no plot"));
    };
    let mut marked: Plot = (*with_intervals).clone();
    if let Some(layer) = marked.layers.last_mut() {
        layer.stat = Some(Stat::Confidence { level: fit.level });
    }
    describe(&mut marked);
    drawn = call(
        ctx,
        marks::points,
        vec![(Some(Value::Plot(Rc::new(marked))), None)],
    )?;
    drawn = call(
        ctx,
        scales::coord,
        vec![
            (Some(drawn), None),
            (Some(Value::Word("flip".to_string())), None),
        ],
    )?;
    // The axis of the terms names them itself, and has no title.
    call(
        ctx,
        scales::labels,
        vec![
            (Some(drawn), None),
            NONE,
            NONE,
            (Some(Value::Str(String::new())), None),
        ],
    )
}

/// `fit.coef_plot()`: the estimates of a fitted model as points with their confidence
/// intervals at the level of the fit, a term on each line in the order of the table, and a
/// line at zero. The intercept is not drawn: it is on another scale than the slopes, and is
/// seldom a quantity of interest.
pub fn coef_plot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = fitted(ctx, &args, "coef_plot")?;
    let kept: Vec<usize> = (0..fit.terms.len())
        .filter(|&at| fit.terms[at] != INTERCEPT)
        .collect();
    if kept.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`coef_plot` draws the coefficients beside the intercept, and this fit has none",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a fit of the intercept alone")
            .help("`estimates()` gives the intercept with its interval as a frame"),
        ));
    }
    coefficients(ctx, &fit, &kept)
}
