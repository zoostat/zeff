//! The frame methods that look at a frame without computing on it (Language Reference, section
//! 6, "Look"): `n_rows`, `n_cols` and `variables` in this step; `describe` follows.
//!
//! Each takes the frame alone and gives no message of its own.

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::Diagnostic;

/// The frame a look method was called on.
fn the_frame<'a>(ctx: &CallContext<'_>, name: &str, args: &'a Args) -> Result<&'a Frame, Fail> {
    match args.get(0) {
        Some(Value::Frame(frame)) => Ok(frame),
        _ => Err(fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script ({name} was called without a frame)"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )),
    }
}

/// `data.n_rows()`: the number of rows, as an Int.
pub fn n_rows(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "n_rows", &args)?;
    Ok(Value::Int(frame.table.rows() as i64))
}

/// `data.n_cols()`: the number of variables, as an Int.
pub fn n_cols(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "n_cols", &args)?;
    Ok(Value::Int(frame.table.n_cols() as i64))
}

/// `data.variables()`: the variable names in the frame's order, as a list of Str.
pub fn variables(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "variables", &args)?;
    Ok(Value::List(
        frame
            .table
            .names()
            .iter()
            .map(|n| Value::Str(n.clone()))
            .collect(),
    ))
}
