//! The row functions `row_mean` and `row_sum` (catalogue rows 136 and 137; release 0.5.0, step
//! 17): a mean or a sum across several variables in each row, inside `add` or `update`, where
//! the enclosing frame supplies the variables. They are Rust rather than Zeff because a body
//! written in Zeff has no handle on the frame whose method encloses the call (open decision #69).

use crate::verbs::resolve_vars;
use zeff_columns::{Column, ColumnType, Value as Cell};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::Diagnostic;

/// The variables an argument names: one, several, or a range.
fn vars_of(args: &Args, at: usize) -> Vec<String> {
    match args.get(at) {
        Some(Value::Vars(names)) => names.clone(),
        Some(Value::Var(name)) => vec![name.clone()],
        _ => Vec::new(),
    }
}

/// The numeric columns the variables name, in the enclosing frame; Z0401 outside a frame method
/// or for a variable that is not an Int or a Float.
fn numeric_columns<'a>(
    ctx: &'a CallContext<'_>,
    name: &str,
    args: &Args,
) -> Result<(&'a Frame, Vec<&'a Column>), Fail> {
    let Some(frame) = ctx.frame.as_ref() else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` reads the variables of a frame and stands inside `add` or `update`"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no frame here")
            .help(format!(
                "write it inside a definition, as in `wages.add(score = {name}([q1, q2, q3]))`"
            )),
        ));
    };
    let positions = resolve_vars(ctx, frame, &vars_of(args, 0), 0)?;
    let mut columns = Vec::with_capacity(positions.len());
    for position in positions {
        let Some(column) = frame.table.column_at(position) else {
            continue;
        };
        if !matches!(column.ty(), ColumnType::Int | ColumnType::Float) {
            let variable = frame.table.names()[position].clone();
            let ty = column.ty();
            let hint = match ty {
                ColumnType::Bool => {
                    "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                        .to_string()
                }
                _ => format!("`{name}` adds numbers; convert the variable first, or leave it out"),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` takes Int and Float variables, but `{variable}` is {}",
                        with_article(ty.name())
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), format!("`{variable}` is not a number"))
                .help(hint),
            ));
        }
        columns.push(column);
    }
    Ok((frame, columns))
}

fn with_article(ty: &str) -> String {
    let article = if ty.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {ty}")
}

/// The values present in `row` across the columns, as Floats (an Int exactly, below 2^53).
fn present(columns: &[&Column], row: usize) -> Vec<f64> {
    columns
        .iter()
        .filter_map(|c| match c.get(row) {
            Some(Cell::Int(i)) => Some(i as f64),
            Some(Cell::Float(x)) => Some(x),
            _ => None,
        })
        .collect()
}

/// `row_sum(vars)`: the compensated sum of the values present in each row; `NA` where none is.
pub fn row_sum(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (frame, columns) = numeric_columns(ctx, "row_sum", &args)?;
    let rows = frame.table.rows();
    let (column, _) = Column::floats((0..rows).map(|row| {
        let values = present(&columns, row);
        if values.is_empty() {
            None
        } else {
            Some(zeff_numeric::sum(&values))
        }
    }));
    Ok(Value::Column(column))
}

/// `row_mean(vars, min_valid: 1)`: the mean of the values present in each row; `NA` where fewer
/// than `min_valid` are present.
pub fn row_mean(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let min_valid = match args.get(1) {
        Some(Value::Int(k)) => *k,
        Some(Value::Na) => {
            return Err(fail(
                Diagnostic::error("Z0401", "`min_valid:` of `row_mean` cannot be NA")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(1), "NA")
                    .help("`min_valid:` is the number of values a row needs for a mean; 1 unless given"),
            ));
        }
        _ => 1,
    };
    if min_valid < 1 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`min_valid:` of `row_mean` cannot be {min_valid}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "fewer than one value")
            .help("a row needs at least one value for a mean; `min_valid:` is 1 unless given"),
        ));
    }
    let (frame, columns) = numeric_columns(ctx, "row_mean", &args)?;
    let rows = frame.table.rows();
    let (column, _) = Column::floats((0..rows).map(|row| {
        let values = present(&columns, row);
        if (values.len() as i64) < min_valid {
            None
        } else {
            zeff_numeric::mean(&values)
        }
    }));
    Ok(Value::Column(column))
}
