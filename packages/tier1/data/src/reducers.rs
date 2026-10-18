//! The descriptive reducers as frame methods: `data.mean(var)` and `data.sd(var)`.
//!
//! The rules of Language Reference section 4.1, "Statistics": `NA` skipped with a warning giving
//! the count; any `Inf` gives `Inf`; `Inf` and `-Inf` together give `NA` with a warning; nothing
//! left gives `NA`. A Bool column counts `true` as 1 (section 3.7).

use zeff_columns::{Column, ColumnType};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::messages::{no_variable, wrong_column_type};
use zeff_diagnostics::{Diagnostic, count};

/// The frame and the numeric column a reducer works on, with the messages of section 3.6 when the
/// variable is not there or not numeric.
fn numeric_column<'a>(
    ctx: &CallContext<'_>,
    function: &str,
    frame: &'a Frame,
    var: &str,
    var_span_index: usize,
) -> Result<&'a Column, Fail> {
    let span = ctx.span_of(var_span_index);
    let name = frame.shown_name();
    let Some(column) = frame.table.column(var) else {
        let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
        return Err(fail(no_variable(ctx.script, span, &name, var, &columns)));
    };
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(column),
        other => Err(fail(wrong_column_type(
            ctx.script,
            span,
            function,
            &name,
            var,
            other.name(),
            "a Bool, Int or Float",
        ))),
    }
}

/// W0201: `n` rows with NA in `var` were skipped (section 10.2's wording); all of them, or a
/// column with no rows at all, gives NA with the warning too.
fn skipped(ctx: &mut CallContext<'_>, var: &str, missing: usize, of: usize) {
    if of == 0 {
        ctx.message(Diagnostic::warning(
            "W0201",
            format!("{var} has no values: the result is NA"),
        ));
        return;
    }
    if missing == 0 {
        return;
    }
    let message = if missing == of {
        format!(
            "all {} rows have NA in {var}: the result is NA",
            count(of as u64)
        )
    } else {
        format!(
            "{} {} with NA in {var} {} skipped",
            count(missing as u64),
            if missing == 1 { "row" } else { "rows" },
            if missing == 1 { "was" } else { "were" }
        )
    };
    ctx.message(Diagnostic::warning("W0201", message));
}

pub fn mean(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(Value::Frame(frame)), Some(Value::Var(var))) = (args.get(0), args.get(1)) else {
        return Err(fail(Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (mean was called without a frame and a variable)").in_file(ctx.script).primary(ctx.callee_span, "the call that failed")));
    };
    let column = numeric_column(ctx, "mean", frame, var, 1)?;
    let values = column
        .numeric_values()
        .unwrap_or(zeff_columns::NumericValues {
            values: Vec::new(),
            missing: column.len(),
        });
    skipped(ctx, var, values.missing, column.len());
    if values.values.is_empty() {
        return Ok(Value::Na);
    }
    match zeff_numeric::mean(&values.values) {
        Some(x) if x.is_nan() => {
            ctx.message(Diagnostic::warning(
                "W0204",
                format!("{var} holds both Inf and -Inf, so its mean is NA"),
            ));
            Ok(Value::Na)
        }
        Some(x) => Ok(Value::Float(x)),
        None => Ok(Value::Na),
    }
}

/// `data.sd(var)`: the sample standard deviation (divisor n − 1) of the non-missing values.
/// Fewer than two values, or an infinite value, make it undefined: `NA` with a warning (W0205,
/// the cause named); a constant column gives 0.
pub fn sd(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(Value::Frame(frame)), Some(Value::Var(var))) = (args.get(0), args.get(1)) else {
        return Err(fail(Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (sd was called without a frame and a variable)").in_file(ctx.script).primary(ctx.callee_span, "the call that failed")));
    };
    let column = numeric_column(ctx, "sd", frame, var, 1)?;
    let values = column
        .numeric_values()
        .unwrap_or(zeff_columns::NumericValues {
            values: Vec::new(),
            missing: column.len(),
        });
    skipped(ctx, var, values.missing, column.len());
    if values.values.is_empty() {
        return Ok(Value::Na);
    }
    if values.values.len() == 1 {
        ctx.message(Diagnostic::warning(
            "W0205",
            format!("sd needs at least two values, but {var} has one: the result is NA"),
        ));
        return Ok(Value::Na);
    }
    if values.values.iter().any(|x| x.is_infinite()) {
        ctx.message(Diagnostic::warning(
            "W0205",
            format!("{var} holds an infinite value, so its sd is NA"),
        ));
        return Ok(Value::Na);
    }
    match zeff_numeric::sd(&values.values) {
        Some(x) if x.is_finite() => Ok(Value::Float(x)),
        _ => {
            ctx.message(Diagnostic::warning(
                "W0205",
                format!("the squared deviations of {var} overflow, so its sd is NA"),
            ));
            Ok(Value::Na)
        }
    }
}
