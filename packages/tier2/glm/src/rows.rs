//! The rows a classifier's results rest on (release 0.9.0, steps 14 and 15): for `curve` and
//! `confusion`, the fitted probability, the outcome and the weight of each row, on the rows
//! the fit used or on the rows of `newdata:`.

use std::rc::Rc;
use zeff_core::display::{fixed_grouped, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::order::WeightKind;
use zeff_results::design::Design;
use zeff_results::likelihood::Link;

pub(crate) fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The fit of a binary model at the receiver of `name`, with its link.
pub(crate) fn binary_fit(
    ctx: &CallContext<'_>,
    args: &Args,
    name: &str,
) -> Result<(Rc<Fit>, Link), Fail> {
    let fit = match args.get(0) {
        Some(Value::Fit(fit)) if fit.likelihood.is_some() => Rc::clone(fit),
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` needs the fit of a binary model, but this is {}",
                        other.map_or("nothing".to_string(), Value::type_word)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a binary fit")
                .help("fit the model with `glm(y ~ x, family: binomial)`"),
            ));
        }
    };
    let link = fit
        .likelihood
        .as_deref()
        .and_then(|likelihood| Link::named(&likelihood.link));
    match link {
        Some(link) => Ok((fit, link)),
        None => Err(internal(ctx, "a `glm` fit of a link that is not known")),
    }
}

/// The outcome of the model as its formula writes it: what stands before `~`.
pub(crate) fn outcome_name(fit: &Fit) -> String {
    fit.formula
        .split('~')
        .next()
        .map_or(String::new(), |text| text.trim().to_string())
}

/// The scores, the outcomes and the weights of the rows, with the rows left out.
pub(crate) struct Rows {
    /// The fitted probability of each row.
    pub scores: Vec<f64>,
    /// The outcome of each row, 1 or 0.
    pub outcomes: Vec<f64>,
    pub weights: Option<Vec<f64>>,
    /// The observations: the rows, or the sum of the frequency weights.
    pub n: usize,
    pub dropped: usize,
    /// Whether the rows are those of `newdata:`.
    pub of_newdata: bool,
}

/// The rows of `newdata` with nothing missing in a variable of the model: their outcomes and
/// the probabilities the fit predicts for them.
fn new_rows(
    ctx: &mut CallContext<'_>,
    fit: &Fit,
    link: Link,
    newdata: &Frame,
    at: usize,
    name: &str,
) -> Result<Rows, Fail> {
    // Whatever the model matrix says of the new rows is said in the line of the rows used.
    let said = ctx.messages.len();
    let built = Design::build_with(ctx, newdata, &fit.model, &[], &fit.levels);
    ctx.messages.truncate(said);
    let design = built?;
    if let Some(column) = design
        .names
        .iter()
        .find(|n| !fit.terms.contains(n) && !fit.omitted.contains(n))
    {
        return Err(internal(
            ctx,
            &format!("the model matrix of `newdata` has a column `{column}` the fit does not"),
        ));
    }
    let Some(outcomes) = design.outcome() else {
        return Err(internal(ctx, "a binary fit without an outcome"));
    };
    if let Some(other) = outcomes.iter().find(|y| **y != 0.0 && **y != 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{}` is {} in a row of `newdata`, and `{name}` needs an outcome of 0 and 1",
                    outcome_name(fit),
                    format_float(*other)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at), "an outcome that is not 0 or 1")
            .help("the outcome of `newdata` is coded as the outcome the model was fitted to"),
        ));
    }
    // A coefficient omitted as collinear counts as nought, as `predict` has it.
    let beta: Vec<f64> = design
        .names
        .iter()
        .map(|column| {
            fit.terms
                .iter()
                .position(|t| t == column)
                .map_or(0.0, |at| fit.estimates[at])
        })
        .collect();
    let mut eta = vec![0.0; design.rows.len()];
    for (column, b) in beta.iter().enumerate() {
        if *b == 0.0 {
            continue;
        }
        for (e, x) in eta.iter_mut().zip(design.column_values(column)) {
            *e += b * x;
        }
    }
    Ok(Rows {
        scores: eta.iter().map(|e| link.mean(*e)).collect(),
        outcomes,
        weights: None,
        n: design.rows.len(),
        dropped: newdata.table.rows() - design.rows.len(),
        of_newdata: true,
    })
}

/// The rows of `name`: those of `newdata:` at argument `at` when it is given, the rows the
/// fit used otherwise.
pub(crate) fn rows_of(
    ctx: &mut CallContext<'_>,
    fit: &Fit,
    link: Link,
    newdata: Option<&Value>,
    at: usize,
    name: &str,
) -> Result<Rows, Fail> {
    let Some(likelihood) = fit.likelihood.as_deref() else {
        return Err(internal(ctx, "a binary fit without its likelihood"));
    };
    match newdata {
        Some(Value::Frame(newdata)) => {
            let newdata = newdata.clone();
            new_rows(ctx, fit, link, &newdata, at, name)
        }
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`newdata:` of `{name}` must be a Frame, but this is {}",
                    other.type_word()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at), other.type_word()),
        )),
        None => Ok(Rows {
            scores: likelihood.eta.iter().map(|e| link.mean(*e)).collect(),
            outcomes: likelihood.outcome.clone(),
            weights: likelihood.weights.clone(),
            n: fit.n,
            dropped: fit.dropped,
            of_newdata: false,
        }),
    }
}

/// A count of rows as a table prints it: a whole number grouped in thousands, or, under
/// weights that are not whole, one decimal.
pub(crate) fn counted(x: f64) -> String {
    if x == x.round() {
        fixed_grouped(x, 0)
    } else {
        fixed_grouped(x, 1)
    }
}

/// The line that says which rows a result rests on: `N = 18,390: 5,849 with `union` 1 and
/// 12,541 with 0`; under sampling weights the rows and the weight variable; for `newdata:`,
/// that the rows are its, and how many were left out.
pub(crate) fn rows_line(fit: &Fit, rows: &Rows, outcome: &str, ones: f64, noughts: f64) -> String {
    let observations = fixed_grouped(rows.n as f64, 0);
    let source_weights = fit.source.as_ref().and_then(|s| s.weights.clone());
    let sampled = !rows.of_newdata
        && rows.weights.is_some()
        && fit.weight_kind != Some(WeightKind::Frequency);
    let mut line = if sampled {
        format!(
            "N = {observations}, weighted by `{}`",
            source_weights.as_deref().unwrap_or("the weights")
        )
    } else {
        format!(
            "N = {observations}: {} with `{outcome}` 1 and {} with 0",
            fixed_grouped(ones, 0),
            fixed_grouped(noughts, 0)
        )
    };
    if rows.of_newdata {
        line.push_str(" among the rows of `newdata`");
        if rows.dropped > 0 {
            line.push_str(&format!(
                " ({} left out for NA in a variable of the model)",
                count(rows.dropped as u64)
            ));
        }
    }
    line
}
