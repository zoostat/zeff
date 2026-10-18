//! The frame methods that look at a frame without computing on it (Language Reference, section
//! 6, "Look"): `n_rows`, `n_cols`, `variables` and `describe`; `head` and `tail` follow.
//!
//! Each takes the frame alone and gives no message of its own.

use zeff_columns::Value as Cell;
use zeff_core::display::{cell, frame_header};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::{Diagnostic, count};

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

/// `data.describe()`: the table of section 10.2, one row per variable: its name, type, counts of
/// non-missing and missing values, and an example, which is the first non-missing value (the
/// owner's decision of 18 October 2026), printed as the value prints elsewhere; a category's
/// example adds its number of levels.
pub fn describe(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "describe", &args)?;
    let table = &frame.table;
    let rows = table
        .names()
        .iter()
        .enumerate()
        .filter_map(|(i, name)| table.column_at(i).map(|column| (name, column)))
        .map(|(name, column)| {
            let first = (0..column.len()).find_map(|r| match column.get(r) {
                Some(Cell::Na) | None => None,
                Some(value) => Some(value),
            });
            let example = match first {
                None => "NA".to_string(),
                Some(Cell::Category(label)) => {
                    let levels = column.levels().map_or(0, |l| l.len());
                    format!(
                        "{label} ({} {})",
                        count(levels as u64),
                        if levels == 1 { "level" } else { "levels" }
                    )
                }
                Some(value) => cell(value),
            };
            vec![
                name.clone(),
                column.ty().name().to_string(),
                count(column.n_valid() as u64),
                count(column.n_missing() as u64),
                example,
            ]
        })
        .collect();
    Ok(Value::Table(Table {
        title: frame_header(frame),
        columns: vec![
            ("variable".to_string(), Align::Left),
            ("type".to_string(), Align::Left),
            ("non-missing".to_string(), Align::Right),
            ("missing".to_string(), Align::Right),
            ("example".to_string(), Align::Left),
        ],
        rows,
    }))
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
