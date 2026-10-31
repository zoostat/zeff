//! The frame verbs of Language Reference section 6 that choose rows and variables: `head`,
//! `tail`, `where`, `select`, `drop` and `rename`. Each returns a new frame that shares its
//! parent's columns (the memory table of section 6: nothing is copied by `select`, `drop` and
//! `rename`; `where` gathers the kept rows), keeps every column's metadata and the frame's, and
//! carries opaque columns through untouched (section 4.3).

use std::rc::Rc;
use zeff_columns::{Column, ColumnType, GroupError, Value as Cell, sort_rows};
use zeff_core::lift::broadcast;
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

/// `data.drop_missing(vars)` (catalogue row 121; release 0.5.0, step 18): the rows with no
/// `NA` in any of the variables named, in their order, with one note giving the rows dropped and
/// the variables (the owner's form, question 3 (c) of the release: `228 rows with NA in educ or
/// exper dropped`; `no row had NA in educ` when nothing is dropped). Metadata and the frame's
/// declaration travel with the table, as in every frame verb.
pub fn drop_missing(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "drop_missing", &args)?;
    let positions = resolve_vars(ctx, frame, &vars_of(&args, 1), 1)?;
    let columns: Vec<&Column> = positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    let names: Vec<&str> = positions
        .iter()
        .map(|&p| frame.table.names()[p].as_str())
        .collect();
    let total = frame.table.rows();
    let kept: Vec<usize> = (0..total)
        .filter(|&row| {
            columns
                .iter()
                .all(|c| !matches!(c.get(row), None | Some(Cell::Na)))
        })
        .collect();
    let dropped = total - kept.len();
    let listed = match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        n => format!("{} or {}", names[..n - 1].join(", "), names[n - 1]),
    };
    if dropped > 0 {
        ctx.message(Diagnostic::plain_note(format!(
            "{} with NA in {listed} dropped",
            rows(dropped)
        )));
    } else {
        ctx.message(Diagnostic::plain_note(format!("no row had NA in {listed}")));
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

/// A definition's value as a column of the frame's length: a column of that length, or a scalar
/// in every row (section 3.7); `NA` alone has no type (Z0219), and anything else is refused.
fn definition_column(
    ctx: &CallContext<'_>,
    verb: &str,
    name: &str,
    value: &Value,
    rows: usize,
    span: zeff_diagnostics::Span,
) -> Result<Column, Fail> {
    match value {
        Value::Column(c) if c.len() != rows => Err(fail(
            Diagnostic::error(
                "Z0217",
                format!("`{name}` has {} and the frame has {}", self::rows(c.len()), self::rows(rows)),
            )
            .in_file(ctx.script)
            .primary(span, "a column of another length")
            .help("build the new variable from the frame's own variables; `merge` joins frames (release 0.6.0)"),
        )),
        Value::Na => Err(fail(
            Diagnostic::error("Z0219", format!("`{name} = NA` has no type"))
                .in_file(ctx.script)
                .primary(span, "NA alone")
                .help("a variable of only NA needs a type: build it from one of the frame's, as in `x * NA`; `to(NA, Float)` arrives with release 0.5.0"),
        )),
        other => broadcast(other, rows).ok_or_else(|| {
            fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` of `{verb}` must be a value or an expression over the frame's variables, not {}",
                        article(other.type_name())
                    ),
                )
                .in_file(ctx.script)
                .primary(span, "not a value a variable can hold")
                .help("write an expression such as `lwage = log(wage)` or a value such as `year = 2026`"),
            )
        }),
    }
}

fn internal(ctx: &CallContext<'_>, what: impl std::fmt::Display) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// `data.add(name = expr, ...)`: new variables, each an expression over the frame's variables and
/// the definitions before it; a name the frame has is replaced, with a note.
pub fn add(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "add", &args)?.clone();
    let mut table = (*frame.table).clone();
    let rows = table.rows();
    for (i, (name, value)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        let column = definition_column(ctx, "add", name, value, rows, span)?;
        let replaced = table.position(name).is_some();
        table = table
            .with_column(name, column)
            .map_err(|e| internal(ctx, e))?;
        if replaced {
            ctx.message(Diagnostic::plain_note(format!(
                "`{name}` replaced the variable of that name"
            )));
        }
    }
    Ok(derived(table))
}

/// `data.update(name = expr, ..., where: condition)`: the values of existing variables changed,
/// everywhere or only in the rows where `where:` is true; the variable keeps its type and its
/// metadata.
pub fn update(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "update", &args)?.clone();
    let shown = frame.shown_name();
    let mut table = (*frame.table).clone();
    let rows = table.rows();
    // The rows to change: every row, or those where the condition is true.
    let mask: Option<Vec<bool>> = match args.get(1) {
        None => None,
        Some(Value::Column(c)) if c.ty() == ColumnType::Bool && c.len() == rows => {
            let values: Vec<Option<bool>> = c.as_bools().map(Iterator::collect).unwrap_or_default();
            let na = values.iter().filter(|v| v.is_none()).count();
            if na > 0 {
                let condition = ctx.text_of(1).trim().to_string();
                ctx.message(Diagnostic::plain_note(format!(
                    "{} where `{condition}` was NA {} left unchanged",
                    self::rows(na),
                    if na == 1 { "was" } else { "were" }
                )));
            }
            Some(values.into_iter().map(|v| v == Some(true)).collect())
        }
        Some(Value::Column(c)) if c.ty() == ColumnType::Bool => {
            return Err(fail(
                Diagnostic::error(
                    "Z0217",
                    format!(
                        "`where:` has {} and {shown} has {}",
                        self::rows(c.len()),
                        self::rows(rows)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "a column of another frame")
                .help("a condition is built from the frame's own variables"),
            ));
        }
        Some(other) => {
            let what = match other {
                Value::Column(c) => format!("a Column<{}>", c.ty().name()),
                Value::Bool(_) => "a single Bool".to_string(),
                Value::Na => "NA".to_string(),
                other => article(other.type_name()),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0203",
                    format!("`where:` of `update` needs a condition that is true or false in every row, but this is {what}"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "not a condition over the rows")
                .help("write a comparison or a Bool variable: `where: hours > 40`, `where: is_missing(hours)`"),
            ));
        }
    };
    for (i, (name, value)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        let columns: Vec<&str> = table.names().iter().map(String::as_str).collect();
        let Some(position) = table.position(name) else {
            let mut d = no_variable(ctx.script, span, &shown, name, &columns);
            d = d.help("`update` changes a variable the frame has; `add` creates a new one");
            return Err(fail(d));
        };
        let old = table
            .column_at(position)
            .cloned()
            .ok_or_else(|| internal(ctx, "a column position without a column"))?;
        let new = definition_column(ctx, "update", name, value, rows, span)?;
        let changed = |row: usize| mask.as_ref().is_none_or(|m| m[row]);
        let refuse = |found: &str| {
            fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`{name}` is {} variable, but the new values are {found}", article(old.ty().name())),
                )
                .in_file(ctx.script)
                .primary(span, "values of another type")
                .help(format!("`update` keeps a variable's type; `add({name} = ...)` replaces the variable with one of the new type")),
            )
        };
        let merged = match (old.ty(), new.ty()) {
            (ColumnType::Bool, ColumnType::Bool) => {
                let (o, n): (Vec<_>, Vec<_>) = (
                    old.as_bools().unwrap().collect(),
                    new.as_bools().unwrap().collect(),
                );
                Column::bools((0..rows).map(|r| if changed(r) { n[r] } else { o[r] }))
            }
            (ColumnType::Int, ColumnType::Int) => {
                let (o, n): (Vec<_>, Vec<_>) = (
                    old.as_ints().unwrap().collect(),
                    new.as_ints().unwrap().collect(),
                );
                Column::ints((0..rows).map(|r| if changed(r) { n[r] } else { o[r] }))
            }
            (ColumnType::Float, ColumnType::Float | ColumnType::Int) => {
                let o: Vec<Option<f64>> = old.as_floats().unwrap().collect();
                let n: Vec<Option<f64>> = match new.as_floats() {
                    Some(values) => values.collect(),
                    None => new
                        .as_ints()
                        .unwrap()
                        .map(|v| v.map(|i| i as f64))
                        .collect(),
                };
                Column::floats((0..rows).map(|r| if changed(r) { n[r] } else { o[r] })).0
            }
            (ColumnType::Str, ColumnType::Str) => {
                let (o, n): (Vec<Option<&str>>, Vec<Option<&str>>) = (
                    old.as_strs().unwrap().collect(),
                    new.as_strs().unwrap().collect(),
                );
                Column::strs((0..rows).map(|r| if changed(r) { n[r] } else { o[r] }))
            }
            (ColumnType::Bool | ColumnType::Int | ColumnType::Float | ColumnType::Str, found) => {
                return Err(refuse(&format!("{}s", found.name())));
            }
            (other, _) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0304",
                        format!(
                            "updating a {} variable is not available in this release",
                            other.name()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(span, "a variable `update` does not change")
                    .help(if other == ColumnType::Category {
                        "`recode` merges or renames a category's levels; `add` replaces the variable"
                    } else {
                        "dates and times gain their functions at 0.6.0 with the `time` package"
                    }),
                ));
            }
        };
        let merged = merged.with_metadata(old.metadata().clone());
        table = table
            .with_column(name, merged)
            .map_err(|e| internal(ctx, e))?;
    }
    Ok(derived(table))
}

/// `data.sort(vars, desc: Bool = false)`: the rows in key order, stable, `NA` last.
pub fn sort(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "sort", &args)?;
    let positions = resolve_vars(ctx, frame, &vars_of(&args, 1), 1)?;
    let descending = matches!(args.get(2), Some(Value::Bool(true)));
    let keys: Vec<&Column> = positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    let order = sort_rows(&keys, descending).map_err(|e| match e {
        GroupError::OpaqueKey(ty) => fail(
            Diagnostic::error(
                "Z0401",
                format!("a {} variable cannot order rows", ty.name()),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "an opaque variable")
            .help("sort by a Bool, Int, Float, Str, Date, DateTime or Category variable"),
        ),
        other => internal(ctx, format!("{other:?}")),
    })?;
    Ok(derived(frame.table.take_rows(&order)))
}
