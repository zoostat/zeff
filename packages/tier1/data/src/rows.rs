//! The row function `row` (catalogue row 136; `row_mean` and `row_sum` of release 0.5.0, step
//! 17, until release 0.8.0, step 8k): a mean or a sum across several variables in each row, inside `add` or `update`, where
//! the enclosing frame supplies the variables; and, from release 0.9.0, step 31c (open
//! decision #166), the count of the values missing among them, `stat: n_missing`, which takes
//! variables of any type. It is Rust rather than Zeff because a body
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

/// The columns the variables name, in the enclosing frame; Z0401 outside a frame method, and,
/// where `numeric` asks for numbers, for a variable that is not an Int or a Float.
fn columns_of<'a>(
    ctx: &'a CallContext<'_>,
    name: &str,
    args: &Args,
    numeric: bool,
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
                "write it inside a definition, as in `wages.add(score = {name}([q1, q2, q3], stat: mean))`"
            )),
        ));
    };
    let positions = resolve_vars(ctx, frame, &vars_of(args, 0), 0)?;
    let mut columns = Vec::with_capacity(positions.len());
    for position in positions {
        let Some(column) = frame.table.column_at(position) else {
            continue;
        };
        if numeric && !matches!(column.ty(), ColumnType::Int | ColumnType::Float) {
            let variable = frame.table.names()[position].clone();
            let ty = column.ty();
            let hint = match ty {
                ColumnType::Bool => {
                    "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                        .to_string()
                }
                _ => format!(
                    "the mean and the sum of `{name}` are of numbers; convert the variable first, or leave it out (`stat: n_missing` counts the missing values of variables of any type)"
                ),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` with `stat: mean` or `stat: sum` takes Int and Float variables, but `{variable}` is {}",
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

/// What `row` computes across the variables of each row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stat {
    Mean,
    Sum,
    /// The count of the values that are missing (open decision #166).
    Missing,
}

/// `row(vars, stat: mean | sum | n_missing, min_valid: 1)` (open decision #99, question 4;
/// release 0.8.0, step 8k, where it took the place of `row_mean` and `row_sum`): the mean or
/// the compensated sum of the values present in each row; `NA` where fewer than `min_valid`
/// are present, so that with the default a row with no value is `NA`. `stat: n_missing` (open
/// decision #166; release 0.9.0, step 31c) is the number of the variables whose value is
/// missing in the row, an Int in every row, for variables of any type: R's
/// `rowSums(is.na(...))` and Stata's `egen rowmiss()`. `stat:` has no default.
pub fn row(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let stat = match args.get(1) {
        Some(Value::Word(w)) if w == "mean" => Stat::Mean,
        Some(Value::Word(w)) if w == "sum" => Stat::Sum,
        Some(Value::Word(w)) if w == "n_missing" => Stat::Missing,
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`row` needs `stat:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`stat:` is missing")
                    .help("usage: `row([q1, q2, q3], stat: mean)` for the mean across the variables, `stat: sum` for their sum, or `stat: n_missing` for the number of them that are missing"),
            ));
        }
    };
    let min_valid = match args.get(2) {
        Some(Value::Int(k)) => *k,
        Some(Value::Na) => {
            return Err(fail(
                Diagnostic::error("Z0401", "`min_valid:` of `row` cannot be NA")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "NA")
                    .help("`min_valid:` is the number of values a row needs for a result; 1 unless given"),
            ));
        }
        _ => 1,
    };
    if min_valid < 1 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`min_valid:` of `row` cannot be {min_valid}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "fewer than one value")
            .help("a row needs at least one value for a result; `min_valid:` is 1 unless given"),
        ));
    }
    if stat == Stat::Missing {
        // `min_valid:` written out changes nothing: a count of missing values is a number
        // in every row. Said in a note with a code, as `level:` of `posthoc` is (W0401).
        let written = matches!(args.get(2), Some(value) if !matches!(value, Value::Na))
            && ctx.arg_spans.get(2).is_some_and(Option::is_some);
        if written {
            ctx.message(
                Diagnostic::note(
                    "W0401",
                    "`min_valid:` changes nothing here: `stat: n_missing` counts the missing values and gives a number in every row",
                )
                .help("`min_valid:` is the number of values a row needs for a mean or a sum"),
            );
        }
        let (frame, columns) = columns_of(ctx, "row", &args, false)?;
        let rows = frame.table.rows();
        return Ok(Value::Column(Column::ints((0..rows).map(|row| {
            Some(columns.iter().filter(|c| c.is_na(row)).count() as i64)
        }))));
    }
    let (frame, columns) = columns_of(ctx, "row", &args, true)?;
    let rows = frame.table.rows();
    let (column, _) = Column::floats((0..rows).map(|row| {
        let values = present(&columns, row);
        if (values.len() as i64) < min_valid {
            None
        } else if stat == Stat::Mean {
            zeff_numeric::mean(&values)
        } else {
            Some(zeff_numeric::sum(&values))
        }
    }));
    Ok(Value::Column(column))
}
