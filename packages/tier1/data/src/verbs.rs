//! The frame verbs of Language Reference section 6 that choose rows and variables: `head`,
//! `tail`, `where`, `select`, `drop` and `rename`. Each returns a new frame that shares its
//! parent's columns (the memory table of section 6: nothing is copied by `select`, `drop` and
//! `rename`; `where` gathers the kept rows), keeps every column's metadata and the frame's, and
//! carries opaque columns through untouched (section 4.3).

use std::rc::Rc;
use zeff_columns::ColumnType;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

/// The frame a verb was called on.
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

/// A new frame over `table`, unnamed until a `let` names it.
fn derived(table: zeff_columns::Table) -> Value {
    Value::Frame(Frame {
        table: Rc::new(table),
        name: None,
    })
}

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// The positions of the variables `names` name in `frame`, in order, each once: a bare name, or a
/// range `a..b` of every variable from `a` to `b` by position (section 3.7, "Variable ranges").
pub fn resolve_vars(
    ctx: &CallContext<'_>,
    frame: &Frame,
    names: &[String],
    at: usize,
) -> Result<Vec<usize>, Fail> {
    let span = ctx.span_of(at);
    let shown = frame.shown_name();
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let position = |name: &str| -> Result<usize, Fail> {
        frame
            .table
            .position(name)
            .ok_or_else(|| fail(no_variable(ctx.script, span, &shown, name, &columns)))
    };
    let mut positions: Vec<usize> = Vec::new();
    let mut push = |p: usize| {
        if !positions.contains(&p) {
            positions.push(p);
        }
    };
    for name in names {
        match name.split_once("..") {
            Some((first, last)) => {
                let (a, b) = (position(first)?, position(last)?);
                if b < a {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!("the range `{name}` runs backwards: `{last}` comes before `{first}` in {shown}"),
                        )
                        .in_file(ctx.script)
                        .primary(span, "a range runs from an earlier variable to a later one")
                        .help(format!("the variables of {shown}, in order: {}", columns.join(", "))),
                    ));
                }
                for p in a..=b {
                    push(p);
                }
            }
            None => push(position(name)?),
        }
    }
    Ok(positions)
}

/// The variables an argument names: one, several, or a range.
fn vars_of(args: &Args, at: usize) -> Vec<String> {
    match args.get(at) {
        Some(Value::Vars(names)) => names.clone(),
        Some(Value::Var(name)) => vec![name.clone()],
        _ => Vec::new(),
    }
}

/// `n:` of `head` and `tail`: a count of rows, 0 or more.
fn row_count(ctx: &CallContext<'_>, name: &str, args: &Args) -> Result<usize, Fail> {
    match args.get(1) {
        None => Ok(10),
        Some(Value::Int(n)) if *n >= 0 => Ok(usize::try_from(*n).unwrap_or(usize::MAX)),
        Some(Value::Int(n)) => Err(fail(
            Diagnostic::error("Z0401", format!("`n:` of `{name}` cannot be {n}"))
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "a count of rows")
                .help(format!(
                    "write how many rows to show, 0 or more: `{name}(n: 5)`"
                )),
        )),
        Some(Value::Na) => Err(fail(
            Diagnostic::error("Z0401", format!("`n:` of `{name}` cannot be NA"))
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "a count of rows")
                .help(format!("write how many rows to show: `{name}(n: 5)`")),
        )),
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`n:` of `{name}` must be an Int, but this is {}",
                    article(other.type_name())
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "a count of rows")
            .help(format!("write how many rows to show: `{name}(n: 5)`")),
        )),
    }
}

fn article(type_name: &str) -> String {
    let vowel = type_name == "NA"
        || (type_name.starts_with(['A', 'E', 'I', 'O', 'U']) && !type_name.starts_with("Union"));
    format!("{} {type_name}", if vowel { "an" } else { "a" })
}

/// `data.head(n: Int = 10)`: the first `n` rows.
pub fn head(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "head", &args)?;
    let n = row_count(ctx, "head", &args)?;
    Ok(derived(frame.table.head(n)))
}

/// `data.tail(n: Int = 10)`: the last `n` rows.
pub fn tail(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "tail", &args)?;
    let n = row_count(ctx, "tail", &args)?;
    Ok(derived(frame.table.tail(n)))
}

/// `data.where(condition)`: the rows where the condition is true; rows where it is `NA` are
/// dropped and counted (section 4.1), with a statement that lists them.
pub fn where_(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "where", &args)?.clone();
    let shown = frame.shown_name();
    let refused = |what: String, help: &str| {
        fail(
            Diagnostic::error(
                "Z0203",
                format!("`where` needs a condition that is true or false in every row, but this is {what}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "not a condition over the rows")
            .help(help),
        )
    };
    let column = match args.get(1) {
        Some(Value::Column(c)) if c.ty() == ColumnType::Bool => c.clone(),
        Some(Value::Column(c)) => {
            return Err(refused(
                format!("a Column<{}>", c.ty().name()),
                "write a comparison or a Bool variable: `where(hours > 40)`, `where(female)`, `where(is_missing(hours))`",
            ));
        }
        Some(Value::Bool(_)) => {
            return Err(refused(
                "a single Bool".to_string(),
                "a condition is an expression over the frame's variables, such as `hours > 40`; `where(true)` would keep every row and `where(false)` none",
            ));
        }
        Some(Value::Na) => {
            return Err(refused(
                "NA".to_string(),
                "`x == NA` is always NA (section 4.1); `is_missing(x)` tests for a missing value",
            ));
        }
        Some(other) => {
            return Err(refused(
                article(other.type_name()),
                "write a comparison or a Bool variable: `where(hours > 40)`, `where(female)`",
            ));
        }
        None => {
            return Err(refused(
                "nothing".to_string(),
                "write a condition: `where(hours > 40)`",
            ));
        }
    };
    if column.len() != frame.table.rows() {
        return Err(fail(
            Diagnostic::error(
                "Z0217",
                format!(
                    "the condition has {} and {shown} has {}",
                    rows(column.len()),
                    rows(frame.table.rows())
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "a column of another frame")
            .help("a condition is built from the frame's own variables; `merge` joins frames (release 0.6.0)"),
        ));
    }
    let mut kept: Vec<usize> = Vec::new();
    let mut dropped = 0usize;
    if let Some(values) = column.as_bools() {
        for (row, value) in values.enumerate() {
            match value {
                Some(true) => kept.push(row),
                Some(false) => {}
                None => dropped += 1,
            }
        }
    }
    if dropped > 0 {
        let condition = ctx.text_of(1).trim().to_string();
        // The first variable the condition used that has NA in it names the rows to inspect.
        let culprit = ctx
            .columns_used
            .iter()
            .find(|name| frame.table.column(name).is_some_and(|c| c.n_missing() > 0))
            .cloned();
        let listing = match (&frame.name, culprit) {
            (Some(name), Some(var)) => format!("; `{name}.where(is_missing({var}))` lists them"),
            (None, Some(var)) => format!("; `where(is_missing({var}))` lists them"),
            _ => String::new(),
        };
        ctx.message(Diagnostic::plain_note(format!(
            "{} dropped where `{condition}` was NA{listing}",
            rows(dropped)
        )));
    }
    if kept.is_empty() {
        ctx.message(Diagnostic::plain_note("no row kept: the result is empty"));
    }
    Ok(derived(frame.table.take_rows(&kept)))
}

/// `data.select(vars)`: only the named variables, in the order named.
pub fn select(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "select", &args)?;
    let positions = resolve_vars(ctx, frame, &vars_of(&args, 1), 1)?;
    Ok(derived(frame.table.select(&positions)))
}

/// `data.drop(vars)`: every variable but the named ones.
pub fn drop(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "drop", &args)?;
    let positions = resolve_vars(ctx, frame, &vars_of(&args, 1), 1)?;
    Ok(derived(frame.table.without(&positions)))
}

/// `data.rename(new = old, ...)`: the variables renamed, the new name on the left, metadata kept.
pub fn rename(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "rename", &args)?;
    let shown = frame.shown_name();
    let mut table = (*frame.table).clone();
    for (i, (new, old)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        let Value::Var(old) = old else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`{new} = ...` must name the variable to rename, without quotes"),
                )
                .in_file(ctx.script)
                .primary(span, "not a variable name")
                .help("the new name is on the left and the old on the right: `rename(pay = wage)`"),
            ));
        };
        let columns: Vec<&str> = table.names().iter().map(String::as_str).collect();
        let Some(position) = table.position(old) else {
            return Err(fail(no_variable(ctx.script, span, &shown, old, &columns)));
        };
        table = table.rename(position, new).map_err(|_| {
            fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`rename` would give two variables of {shown} the name `{new}`"),
                )
                .in_file(ctx.script)
                .primary(span, "a name another variable has")
                .help(format!("`drop({new})` first, or choose another name")),
            )
        })?;
    }
    Ok(derived(table))
}
