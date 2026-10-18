//! Scalar functions on columns (Language Reference, section 3.7, "Columns and scalars"): every
//! built-in scalar function of `core` applies row by row when given a column and returns a
//! column. The body is written for one row; this module calls it once per row, a scalar beside a
//! column standing for its value in every row, collects the results into a column of the right
//! type, and counts the warnings, so that `sqrt` of a column with three negative values says
//! "gave NA in 3 rows" once rather than three times (section 4.1).

use crate::runtime::{Args, Body, CallContext, Fail, fail};
use crate::value::Value;
use zeff_columns::{Column, ColumnType, Value as Cell};
use zeff_diagnostics::{Diagnostic, count};

/// The values of one cell, as the body sees them.
fn cell_value(column: &Column, row: usize) -> Value {
    match column.get(row).unwrap_or(Cell::Na) {
        Cell::Na => Value::Na,
        Cell::Bool(b) => Value::Bool(b),
        Cell::Int(i) => Value::Int(i),
        Cell::Float(x) => Value::Float(x),
        Cell::Str(s) | Cell::Category(s) => Value::Str(s.to_string()),
        // Dates and opaque cells reach no scalar function of this release: the checker admits
        // only Bool, Int, Float and Str columns to one.
        Cell::Date(_) | Cell::DateTime(_) | Cell::Opaque(_) => Value::Na,
    }
}

/// The common length of the columns among the arguments, or the two lengths that disagree.
fn common_length(args: &Args) -> Result<usize, (usize, usize)> {
    let mut length: Option<usize> = None;
    for value in args.values.iter().flatten().chain(args.rest.iter()) {
        if let Value::Column(c) = value {
            match length {
                Some(n) if n != c.len() => return Err((n, c.len())),
                _ => length = Some(c.len()),
            }
        }
    }
    Ok(length.unwrap_or(0))
}

/// Whether every number among the arguments is an Int (a column of Ints, or an Int scalar).
fn all_ints(args: &Args) -> bool {
    args.values
        .iter()
        .flatten()
        .chain(args.rest.iter())
        .all(|v| match v {
            Value::Column(c) => c.ty() != ColumnType::Float,
            Value::Float(_) => false,
            _ => true,
        })
}

/// Call `body` once per row of the columns among `args`, and return the column of its results.
/// `returns` is the function's declared return type (`Float`, `Int`, `Bool`, `Str` or `Number`),
/// which decides the column's type when no row gives a value.
pub fn call_rowwise(
    body: Body,
    ctx: &mut CallContext<'_>,
    args: Args,
    returns: &str,
) -> Result<Value, Fail> {
    let len = common_length(&args).map_err(|(a, b)| {
        fail(
            Diagnostic::error(
                "Z0217",
                format!("columns of {a} and {b} rows cannot be combined"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "two lengths")
            .help("combine columns of one frame, or of frames with the same rows; `merge` joins frames (release 0.6.0)"),
        )
    })?;
    let ints = all_ints(&args);
    let mut results: Vec<Value> = Vec::with_capacity(len);
    // Warnings by code and wording, with the rows they were raised in, in the order first seen.
    let mut counted: Vec<(Diagnostic, usize)> = Vec::new();
    for row in 0..len {
        let row_args = Args {
            values: args
                .values
                .iter()
                .map(|v| v.as_ref().map(|v| row_value(v, row)))
                .collect(),
            rest: args.rest.iter().map(|v| row_value(v, row)).collect(),
            definitions: Vec::new(),
        };
        let mut messages: Vec<Diagnostic> = Vec::new();
        let result = {
            let mut row_ctx = CallContext {
                files: ctx.files,
                script: ctx.script,
                callee_span: ctx.callee_span,
                arg_spans: ctx.arg_spans.clone(),
                rest_spans: ctx.rest_spans.clone(),
                definition_spans: Vec::new(),
                output: ctx.output,
                messages: &mut messages,
            };
            body(&mut row_ctx, row_args)
        };
        let value = result.map_err(|mut error| {
            error.message = format!("{} (row {})", error.message, count(row as u64 + 1));
            error
        })?;
        for message in messages {
            match counted
                .iter_mut()
                .find(|(d, _)| d.code == message.code && d.message == message.message)
            {
                Some((_, n)) => *n += 1,
                None => counted.push((message, 1)),
            }
        }
        results.push(value);
    }
    for (mut diagnostic, n) in counted {
        diagnostic.message = format!(
            "{} in {} {}",
            diagnostic.message,
            count(n as u64),
            if n == 1 { "row" } else { "rows" }
        );
        ctx.message(diagnostic);
    }
    Ok(Value::Column(column_of(results, returns, ints)))
}

fn row_value(value: &Value, row: usize) -> Value {
    match value {
        Value::Column(c) => cell_value(c, row),
        other => other.clone(),
    }
}

/// The column holding `results`, typed by the values present, or by the declared return type
/// when every value is `NA`.
fn column_of(results: Vec<Value>, returns: &str, ints: bool) -> Column {
    let any = |f: fn(&Value) -> bool| results.iter().any(f);
    let float = any(|v| matches!(v, Value::Float(_)))
        || (!any(|v| matches!(v, Value::Int(_) | Value::Bool(_) | Value::Str(_)))
            && (returns == "Float" || (returns == "Number" && !ints)));
    if float {
        let (column, _) = Column::floats(results.iter().map(|v| match v {
            Value::Float(x) => Some(*x),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }));
        return column;
    }
    if any(|v| matches!(v, Value::Bool(_))) || returns == "Bool" {
        return Column::bools(results.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        }));
    }
    if any(|v| matches!(v, Value::Str(_))) || returns == "Str" {
        return Column::strs(results.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }));
    }
    Column::ints(results.iter().map(|v| match v {
        Value::Int(i) => Some(*i),
        _ => None,
    }))
}
