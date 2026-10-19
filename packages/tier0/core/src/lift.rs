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

/// A column from one value per row (a definition evaluated group by group and put back in row
/// order): Floats when any value is a Float, else Ints, Bools or Strs; `None` when the values are
/// of kinds that do not join, or are all `NA` (whose type nothing fixes).
pub fn collect_column(values: &[Value]) -> Option<Column> {
    let mut kinds = (false, false, false, false);
    for v in values {
        match v {
            Value::Float(_) => kinds.0 = true,
            Value::Int(_) => kinds.1 = true,
            Value::Bool(_) => kinds.2 = true,
            Value::Str(_) => kinds.3 = true,
            Value::Na => {}
            _ => return None,
        }
    }
    let number = kinds.0 || kinds.1;
    if (number as u8 + kinds.2 as u8 + kinds.3 as u8) != 1 {
        return None;
    }
    Some(if kinds.0 {
        Column::floats(values.iter().map(|v| match v {
            Value::Float(x) => Some(*x),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }))
        .0
    } else if kinds.1 {
        Column::ints(values.iter().map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        }))
    } else if kinds.2 {
        Column::bools(values.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        }))
    } else {
        Column::strs(values.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }))
    })
}

/// A scalar as a column of `rows` copies of it (a scalar beside a column stands for its value in
/// every row, section 3.7); a column is itself; `None` for a value no column holds (`NA` alone,
/// whose type is unknown, a list, a record).
pub fn broadcast(value: &Value, rows: usize) -> Option<Column> {
    Some(match value {
        Value::Column(c) => c.clone(),
        Value::Int(i) => Column::ints(std::iter::repeat_n(Some(*i), rows)),
        Value::Float(x) => Column::floats(std::iter::repeat_n(Some(*x), rows)).0,
        Value::Bool(b) => Column::bools(std::iter::repeat_n(Some(*b), rows)),
        Value::Str(s) => Column::strs(std::iter::repeat_n(Some(s.as_str()), rows)),
        _ => return None,
    })
}

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

/// The kinds of value among the arguments, for the type of a result column that no row fills.
#[derive(Debug, Clone, Copy, Default)]
struct Inputs {
    float: bool,
    int: bool,
    bool: bool,
    str: bool,
}

fn inputs_of(args: &Args) -> Inputs {
    let mut inputs = Inputs::default();
    for value in args.values.iter().flatten().chain(args.rest.iter()) {
        match value {
            Value::Column(c) => match c.ty() {
                ColumnType::Float => inputs.float = true,
                ColumnType::Int => inputs.int = true,
                ColumnType::Bool => inputs.bool = true,
                ColumnType::Str | ColumnType::Category => inputs.str = true,
                _ => {}
            },
            Value::Float(_) => inputs.float = true,
            Value::Int(_) => inputs.int = true,
            Value::Bool(_) => inputs.bool = true,
            Value::Str(_) => inputs.str = true,
            _ => {}
        }
    }
    inputs
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
    let inputs = inputs_of(&args);
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
                text: ctx.text,
                columns_used: Vec::new(),
                frame: ctx.frame.clone(),
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
    Ok(Value::Column(column_of(results, returns, inputs)))
}

fn row_value(value: &Value, row: usize) -> Value {
    match value {
        Value::Column(c) => cell_value(c, row),
        other => other.clone(),
    }
}

/// The column holding `results`, typed by the values present, or, when every row is `NA`, by the
/// declared result type (`Float`, `Int`, `Bool`, `Str`) or, for `Number` and `Join`, by the kinds
/// of the inputs as the checker joins them.
fn column_of(results: Vec<Value>, returns: &str, inputs: Inputs) -> Column {
    let any = |f: fn(&Value) -> bool| results.iter().any(f);
    let present = any(|v| !matches!(v, Value::Na));
    let joins = matches!(returns, "Number" | "Join");
    let kind = if present {
        if any(|v| matches!(v, Value::Float(_))) || (joins && inputs.float) {
            // A Float among the inputs makes the joined type Float even where every Float cell
            // was NA in the rows that gave a value (section 3.7, "Joining types").
            "Float"
        } else if any(|v| matches!(v, Value::Int(_))) {
            "Int"
        } else if any(|v| matches!(v, Value::Bool(_))) {
            "Bool"
        } else {
            "Str"
        }
    } else {
        match returns {
            "Float" | "Int" | "Bool" | "Str" => returns,
            _ if inputs.float => "Float",
            _ if inputs.int => "Int",
            _ if inputs.str => "Str",
            _ if inputs.bool => "Bool",
            _ => "Float",
        }
    };
    match kind {
        "Float" => {
            let (column, _) = Column::floats(results.iter().map(|v| match v {
                Value::Float(x) => Some(*x),
                Value::Int(i) => Some(*i as f64),
                _ => None,
            }));
            column
        }
        "Int" => Column::ints(results.iter().map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })),
        "Bool" => Column::bools(results.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })),
        _ => Column::strs(results.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        })),
    }
}
