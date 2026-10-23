//! `reg` (Language Reference, sections 6, 7.1, 7.2 and 10.2; question 3 (a) of release 0.4.0):
//! the design of the formula on the frame, the least-squares fit, the classical variance, the
//! notes of what was changed and the warning of a near-singular design, and the fit value with
//! its printed summary.

use crate::ols::Ols;
use std::rc::Rc;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_results::design::Design;
use zeff_results::estimator::Estimator;
use zeff_results::{summary, variance};

/// The condition number above which the design is warned about (W1001, section 11.1).
pub const CONDITION_WARNING: f64 = 1e8;

/// `data.reg(formula, se: classical, level: Float = 0.95)`.
pub fn reg(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "reg was called without a frame"));
    };
    let Some(Value::Formula(formula)) = args.get(1) else {
        return Err(internal(ctx, "reg was called without a formula"));
    };
    let se_kind = match args.get(2) {
        Some(Value::Word(word)) => word.clone(),
        _ => "classical".to_string(),
    };
    if se_kind != "classical" {
        return Err(fail(
            Diagnostic::error(
                "Z0304",
                format!("`se: {se_kind}` is not available in this release"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "arrives with the variance layer of Zeff 0.4.0")
            .help("classical standard errors are available now; robust and clustered errors follow at step 11 of this release"),
        ));
    }
    let level = match args.get(3) {
        Some(Value::Float(level)) => *level,
        Some(Value::Int(level)) => *level as f64,
        _ => 0.95,
    };
    if !(level > 0.0 && level < 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`level:` is a probability strictly between 0 and 1, but this is {level}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(3), "not a confidence level")
            .help("the usual levels are 0.9, 0.95 and 0.99"),
        ));
    }
    let frame = frame.clone();
    let formula = formula.clone();
    let design = Design::build(ctx, &frame, &formula, &[])?;
    for note in &design.notes {
        ctx.message(Diagnostic::plain_note(note.clone()));
    }
    let fitted = Ols.fit(ctx, &design)?;
    if !fitted.omitted.is_empty() {
        let names: Vec<String> = fitted
            .omitted
            .iter()
            .map(|&c| format!("`{}`", design.names[c]))
            .collect();
        ctx.message(Diagnostic::plain_note(if names.len() == 1 {
            format!(
                "{} is collinear with the columns before it and was dropped",
                names[0]
            )
        } else {
            format!(
                "{} are collinear with the columns before them and were dropped",
                names.join(", ")
            )
        }));
    }
    if fitted.condition_number > CONDITION_WARNING {
        ctx.message(
            Diagnostic::warning(
                "W1001",
                format!(
                    "near-singular regressors: the condition number of the scaled design is {:.3e}",
                    fitted.condition_number
                ),
            )
            .help("two regressors are almost the same column; drop one, or combine them"),
        );
    }
    let df = design.n() - fitted.kept.len();
    let bread = Ols.bread(&fitted);
    let vcov = variance::classical(&fitted, &bread, df);
    let fit = summary::linear_fit(
        &design,
        &fitted,
        &vcov,
        "reg",
        &formula.to_string(),
        &se_kind,
        level,
    );
    if fit.r_squared.is_none() {
        let outcome = formula
            .outcome
            .as_ref()
            .map_or(String::new(), |o| o.text.clone());
        ctx.message(Diagnostic::plain_note(format!(
            "`{outcome}` has no variation, so R² is undefined"
        )));
    }
    Ok(Value::Fit(Rc::new(fit)))
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
