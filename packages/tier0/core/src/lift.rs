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
    let (mut dates, mut datetimes) = (false, false);
    for v in values {
        match v {
            Value::Float(_) => kinds.0 = true,
            Value::Int(_) => kinds.1 = true,
            Value::Bool(_) => kinds.2 = true,
            Value::Str(_) => kinds.3 = true,
            Value::Date(_) => dates = true,
            Value::DateTime(_) => datetimes = true,
            Value::Na => {}
            _ => return None,
        }
    }
    let number = kinds.0 || kinds.1;
    if (number as u8 + kinds.2 as u8 + kinds.3 as u8 + dates as u8 + datetimes as u8) != 1 {
        return None;
    }
    if dates {
        return Some(Column::dates(values.iter().map(|v| match v {
            Value::Date(d) => Some(*d),
            _ => None,
        })));
    }
    if datetimes {
        return Some(Column::datetimes(values.iter().map(|v| match v {
            Value::DateTime(t) => Some(*t),
            _ => None,
        })));
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
        Value::Date(d) => Column::dates(std::iter::repeat_n(Some(*d), rows)),
        Value::DateTime(t) => Column::datetimes(std::iter::repeat_n(Some(*t), rows)),
        _ => return None,
    })
}

/// The values of one cell, as the body sees them.
pub fn cell_value(column: &Column, row: usize) -> Value {
    match column.get(row).unwrap_or(Cell::Na) {
        Cell::Na => Value::Na,
        Cell::Bool(b) => Value::Bool(b),
        Cell::Int(i) => Value::Int(i),
        Cell::Float(x) => Value::Float(x),
        Cell::Str(s) | Cell::Category(s) => Value::Str(s.to_string()),
        Cell::Date(days) => Value::Date(days),
        Cell::DateTime(micros) => Value::DateTime(micros),
        // An opaque cell reaches no scalar function: the checker admits only computable
        // columns to one (section 4.3).
        Cell::Opaque(_) => Value::Na,
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
    date: bool,
    datetime: bool,
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
                ColumnType::Date => inputs.date = true,
                ColumnType::DateTime => inputs.datetime = true,
                _ => {}
            },
            Value::Float(_) => inputs.float = true,
            Value::Int(_) => inputs.int = true,
            Value::Bool(_) => inputs.bool = true,
            Value::Str(_) => inputs.str = true,
            Value::Date(_) => inputs.date = true,
            Value::DateTime(_) => inputs.datetime = true,
            _ => {}
        }
    }
    inputs
}

/// The function a row-wise call applies, for the check of its columns' types: its name and, for
/// each positional parameter, the parameter's name and declared type as the language writes it
/// (`Float`, `Int | Float`, `Str`, `Date | DateTime`).
#[derive(Debug, Clone, Copy)]
pub struct Params<'a> {
    pub function: &'a str,
    pub names: &'a [&'a str],
    pub types: &'a [&'a str],
}

/// Whether a column of type `column` may be given to a parameter declared `ty`: the scalar types
/// of section 4 by name, `Float` and `Number` taking an Int as the checker does and `Str` a
/// category, whose rows are its labels; a declaration
/// that is no scalar type (`Any`, `Join`, `Same`, a `Column<…>`, a list, a word) is left to the
/// body. An opaque column is taken by no scalar parameter (section 4.3, rule 2).
pub(crate) fn accepts_column(ty: &str, column: ColumnType) -> bool {
    let mut scalar = false;
    for alternative in ty.split('|').map(str::trim) {
        let accepted: Option<&[ColumnType]> = match alternative {
            "Float" | "Number" => Some(&[ColumnType::Float, ColumnType::Int]),
            "Int" => Some(&[ColumnType::Int]),
            "Bool" => Some(&[ColumnType::Bool]),
            // A category's rows reach a function as their labels (`cell_value`), so a
            // category column fits a parameter that takes text: `is_in(industry, ["Retail"])`.
            "Str" => Some(&[ColumnType::Str, ColumnType::Category]),
            "Date" => Some(&[ColumnType::Date]),
            "DateTime" => Some(&[ColumnType::DateTime]),
            "Category" => Some(&[ColumnType::Category]),
            _ => None,
        };
        match accepted {
            Some(kinds) => {
                scalar = true;
                if kinds.contains(&column) {
                    return true;
                }
            }
            None => return true,
        }
    }
    !scalar
}

/// A type as a message names it: `a Float`, `an Int or a Float`.
fn with_articles(ty: &str) -> String {
    let parts: Vec<String> = ty
        .split('|')
        .map(str::trim)
        .map(|name| {
            let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
                "an"
            } else {
                "a"
            };
            format!("{article} {name}")
        })
        .collect();
    match parts.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        Some((last, _)) => last.clone(),
        None => String::new(),
    }
}

/// The conversion that would make a column fit a parameter, for the help line of Z0401: chosen
/// by the type the parameter wants where that names one conversion (a date and time from Unix
/// time or from text, a date from text or from its parts, whole numbers from a Float, text from
/// anything), and by the column's own type otherwise (numbers from text, from a category, from a
/// Bool, from a date).
fn conversion_for(wanted: &str, column: ColumnType) -> &'static str {
    let wants = |name: &str| wanted.split('|').any(|part| part.trim() == name);
    let numeric = matches!(column, ColumnType::Int | ColumnType::Float);
    if wants("Date") && wants("DateTime") && numeric {
        return "`date(year, month, day)` makes a date of three numbers, and `datetime(x, epoch: seconds)` reads a number of seconds since 1970";
    }
    if wants("Date") && wants("DateTime") && column == ColumnType::Str {
        return "`date(x)` reads a date written as text, and `datetime(x)` a date and time";
    }
    if wants("DateTime") && numeric {
        return "`datetime(x, epoch: seconds)` reads a number of seconds since 1970 as a date and time";
    }
    if wants("DateTime") && column == ColumnType::Str {
        return "`datetime(x)` reads a date and time written as text";
    }
    if wants("DateTime") && !wants("Date") && column == ColumnType::Date {
        return "a Date holds no time of day; `datetime` reads a date and time from text or from Unix time";
    }
    if wants("Date") && numeric {
        return "`date(year, month, day)` makes a date of three numbers";
    }
    if wants("Date") && column == ColumnType::Str {
        return "`date(x)` reads a date written as text";
    }
    if wants("DateTime") && !wants("Date") && column == ColumnType::Category {
        return "`datetime(to(x, Str))` reads a category's labels as dates and times";
    }
    if wants("Date") && column == ColumnType::Category {
        return "`date(to(x, Str))` reads a category's labels as dates";
    }
    if wants("Int") && !wants("Float") && column == ColumnType::Float {
        return "`to(x, Int)` reads a Float column of whole numbers as an Int";
    }
    if wants("Str") && !wants("Int") && !wants("Float") && column != ColumnType::Str {
        return "`to(x, Str)` writes a value as text";
    }
    match column {
        ColumnType::Str => "`to(x, Float)` or `to(x, Int)` reads numbers written as text",
        ColumnType::Category => {
            "`to(x, Int)` gives the positions of a category's levels and `to(x, Str)` their labels"
        }
        ColumnType::Bool => "`to(x, Int)` gives 1 for true and 0 for false",
        ColumnType::Date | ColumnType::DateTime => {
            "`year`, `month` and the other functions of `time` read a date's parts as numbers"
        }
        _ => "`to(x, Str)` writes a value as text and `to(x, Float)` reads one",
    }
}

/// Z0401: a column of a type the parameter does not take, refused before any row runs (open
/// decision #108: what R and Stata do, and what the checker does for one value), with the
/// conversion that would make it fit.
fn wrong_column(ctx: &CallContext<'_>, params: Params<'_>, i: usize, column: ColumnType) -> Fail {
    let name = params.names.get(i).copied().unwrap_or("x");
    let ty = params.types.get(i).copied().unwrap_or("");
    let span = ctx
        .arg_spans
        .get(i)
        .copied()
        .flatten()
        .unwrap_or(ctx.callee_span);
    let conversion = conversion_for(ty, column);
    fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{name}` of `{}` must be {}, but this is a column of {}",
                params.function,
                with_articles(ty),
                column.name()
            ),
        )
        .in_file(ctx.script)
        .primary(span, format!("a column of {}", column.name()))
        .help(format!(
            "a function given a column applies to every row, and the rows must be of a type it takes; {conversion}"
        )),
    )
}

/// Call `body` once per row of the columns among `args`, and return the column of its results.
/// `returns` is the function's declared return type (`Float`, `Int`, `Bool`, `Str` or `Number`),
/// which decides the column's type when no row gives a value; `params` names the function and
/// its parameters' types, against which each column is checked before any row runs.
pub fn call_rowwise(
    body: Body,
    ctx: &mut CallContext<'_>,
    args: Args,
    returns: &str,
    params: Params<'_>,
) -> Result<Value, Fail> {
    for (i, value) in args.values.iter().enumerate() {
        if let Some(Value::Column(column)) = value
            && let Some(ty) = params.types.get(i)
            && !accepts_column(ty, column.ty())
        {
            return Err(wrong_column(ctx, params, i, column.ty()));
        }
    }
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
                group_rows: ctx.group_rows.clone(),
                callee_span: ctx.callee_span,
                arg_spans: ctx.arg_spans.clone(),
                rest_spans: ctx.rest_spans.clone(),
                definition_spans: Vec::new(),
                output: ctx.output,
                messages: &mut messages,
                random: ctx.random,
                statement: ctx.statement,
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
        } else if any(|v| matches!(v, Value::Date(_))) {
            "Date"
        } else if any(|v| matches!(v, Value::DateTime(_))) {
            "DateTime"
        } else {
            "Str"
        }
    } else {
        match returns {
            "Float" | "Int" | "Bool" | "Str" | "Date" | "DateTime" => returns,
            _ if inputs.float => "Float",
            _ if inputs.int => "Int",
            _ if inputs.str => "Str",
            _ if inputs.bool => "Bool",
            _ if inputs.date => "Date",
            _ if inputs.datetime => "DateTime",
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
        "Date" => Column::dates(results.iter().map(|v| match v {
            Value::Date(d) => Some(*d),
            _ => None,
        })),
        "DateTime" => Column::datetimes(results.iter().map(|v| match v {
            Value::DateTime(t) => Some(*t),
            _ => None,
        })),
        _ => Column::strs(results.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        })),
    }
}
