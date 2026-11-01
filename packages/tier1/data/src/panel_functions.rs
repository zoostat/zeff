//! The panel column functions `lag`, `lead`, `diff` and `fill_forward` (catalogue rows 125 to
//! 127 and 139; Language Reference, section 6, "Panels"; release 0.6.0, step 8), inside `add` or
//! `update`, where the enclosing frame supplies the panel index: each unit's rows in time order,
//! a gap in time giving `NA` as Stata's `L.` does. `diff` is Rust rather than Zeff because a
//! body written in Zeff has no handle on the enclosing frame (open decision #69), as `row_mean`.

use std::sync::Arc;
use zeff_columns::{Column, ColumnType, Groups};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::Diagnostic;

/// The enclosing frame, or Z0602 outside a frame method.
fn enclosing<'a>(ctx: &'a CallContext<'_>, name: &str) -> Result<&'a Frame, Fail> {
    ctx.frame.as_ref().ok_or_else(|| {
        fail(
            Diagnostic::error(
                "Z0602",
                format!("`{name}` follows a panel's time order and stands inside `add` or `update`"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no frame here")
            .help(format!(
                "write it inside a definition of a declared panel, as in `wages.add(prev = {name}(wage))`"
            )),
        )
    })
}

/// The panel of the enclosing frame: the sorted index, each row's unit as a group number and
/// each row's time as a count (periods for an Int, days for a Date); Z0602 without a
/// declaration.
struct PanelOrder {
    index: Arc<Vec<usize>>,
    unit_of_row: Vec<usize>,
    time_of_row: Vec<i64>,
}

fn panel_order(ctx: &CallContext<'_>, frame: &Frame, name: &str) -> Result<PanelOrder, Fail> {
    let undeclared = || {
        fail(
            Diagnostic::error(
                "Z0602",
                format!(
                    "`{name}` needs a panel, but {} has no declaration",
                    frame.shown_name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "which unit, and which time?")
            .help("declare the panel first: `data.panel(unit: firm, time: year)` (section 6 of the Language Reference)"),
        )
    };
    let Some((unit, time)) = frame.table.metadata().panel.clone() else {
        return Err(undeclared());
    };
    let (Some(unit_column), Some(time_column)) =
        (frame.table.column(&unit), frame.table.column(&time))
    else {
        return Err(undeclared());
    };
    let index = frame.table.panel_index().ok_or_else(undeclared)?;
    let groups = Groups::by(&[unit_column]).map_err(|e| internal(ctx, &format!("{e:?}")))?;
    let time_of_row: Vec<i64> = match time_column.ty() {
        ColumnType::Int => time_column
            .as_ints()
            .expect("an Int column")
            .map(|t| t.unwrap_or(i64::MIN))
            .collect(),
        ColumnType::Date => time_column
            .as_dates()
            .expect("a Date column")
            .map(|t| t.map_or(i64::MIN, i64::from))
            .collect(),
        _ => return Err(undeclared()),
    };
    Ok(PanelOrder {
        index,
        unit_of_row: groups.group_of_row().to_vec(),
        time_of_row,
    })
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The column argument, of the enclosing frame's length (Z0217 otherwise).
fn column_arg(
    ctx: &CallContext<'_>,
    frame: &Frame,
    name: &str,
    args: &Args,
) -> Result<Column, Fail> {
    match args.get(0) {
        Some(Value::Column(c)) if c.len() == frame.table.rows() => Ok(c.clone()),
        Some(Value::Column(c)) => Err(fail(
            Diagnostic::error(
                "Z0217",
                format!(
                    "`{name}` was given a column of {} rows, but {} has {}",
                    c.len(),
                    frame.shown_name(),
                    frame.table.rows()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a column of another frame")
            .help("name one of the frame's own variables"),
        )),
        _ => Err(internal(ctx, "a value that is not a column")),
    }
}

/// `n:`, a count of periods of at least 1 (1 unless given).
fn periods(ctx: &CallContext<'_>, name: &str, args: &Args) -> Result<i64, Fail> {
    match args.get(1) {
        None => Ok(1),
        Some(Value::Int(n)) if *n >= 1 => Ok(*n),
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`n:` of `{name}` is a count of periods of at least 1, but this is {}",
                    match other {
                        Value::Int(n) => n.to_string(),
                        Value::Na => "NA".to_string(),
                        other => other.type_word(),
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "not a count of periods")
            .help(format!(
                "`{name}(x)` looks one period {}; `{name}(x, n: 2)` two",
                if name == "lead" { "ahead" } else { "back" }
            )),
        )),
    }
}

/// For each row, the row of the same unit `n` periods away (`-n` back, `+n` ahead) in the
/// panel's order, or `None` at a gap or beyond the unit's first or last row.
fn shifted(order: &PanelOrder, n: i64) -> Vec<Option<usize>> {
    let index = &order.index;
    let mut source: Vec<Option<usize>> = vec![None; index.len()];
    // Within a unit the index runs in time order; a two-pointer walk finds the row whose time is
    // exactly `n` periods away, or none.
    let mut start = 0;
    while start < index.len() {
        let unit = order.unit_of_row[index[start]];
        let mut end = start;
        while end < index.len() && order.unit_of_row[index[end]] == unit {
            end += 1;
        }
        let run = &index[start..end];
        let mut j = 0usize;
        for &row in run {
            // The wanted time rises with the row whichever way the shift points, so the second
            // pointer only ever moves forward.
            let wanted = order.time_of_row[row].saturating_add(n);
            while j < run.len() && order.time_of_row[run[j]] < wanted {
                j += 1;
            }
            if j < run.len() && order.time_of_row[run[j]] == wanted {
                source[row] = Some(run[j]);
            }
        }
        start = end;
    }
    source
}

fn shift(ctx: &mut CallContext<'_>, args: Args, name: &str, ahead: bool) -> Result<Value, Fail> {
    let frame = enclosing(ctx, name)?.clone();
    let n = periods(ctx, name, &args)?;
    let x = column_arg(ctx, &frame, name, &args)?;
    let order = panel_order(ctx, &frame, name)?;
    let source = shifted(&order, if ahead { n } else { -n });
    Ok(Value::Column(x.take_optional(&source)))
}

/// `lag(x, n:)`: the value `n` periods earlier in the unit, `NA` at a gap or before the first.
pub fn lag(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    shift(ctx, args, "lag", false)
}

/// `lead(x, n:)`: the value `n` periods later in the unit.
pub fn lead(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    shift(ctx, args, "lead", true)
}

/// `diff(x, n:)`: `x - lag(x, n)`, so the panel's order and gaps are respected; Int for an Int
/// column, Float otherwise.
pub fn diff(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = enclosing(ctx, "diff")?.clone();
    let n = periods(ctx, "diff", &args)?;
    let x = column_arg(ctx, &frame, "diff", &args)?;
    let order = panel_order(ctx, &frame, "diff")?;
    let source = shifted(&order, -n);
    let lagged = x.take_optional(&source);
    let (column, notes) = zeff_core::operators::column_binary(
        zeff_core::operators::Op::Sub,
        zeff_core::operators::Operand::Column(&x),
        zeff_core::operators::Operand::Column(&lagged),
    )
    .map_err(|trouble| {
        fail(zeff_core::operators::column_trouble_diagnostic(
            trouble,
            ctx.script,
            ctx.span_of(0),
        ))
    })?;
    zeff_core::operators::warnings_of(notes, ctx.messages);
    Ok(Value::Column(column))
}

/// `fill_forward(x)`: `NA` replaced by the last value present, within each unit in the panel's
/// time order, or in the frame's row order without a declaration (under `by:` the frame is the
/// group).
pub fn fill_forward(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = enclosing(ctx, "fill_forward")?.clone();
    let x = column_arg(ctx, &frame, "fill_forward", &args)?;
    let rows = frame.table.rows();
    let mut source: Vec<Option<usize>> = (0..rows).map(Some).collect();
    let walk = |order: &[usize], unit_of_row: Option<&[usize]>, source: &mut Vec<Option<usize>>| {
        let mut last: Option<usize> = None;
        let mut unit: Option<usize> = None;
        for &row in order {
            let this_unit = unit_of_row.map(|u| u[row]);
            if this_unit != unit {
                unit = this_unit;
                last = None;
            }
            if x.get(row)
                .is_some_and(|v| !matches!(v, zeff_columns::Value::Na))
            {
                last = Some(row);
            } else {
                source[row] = last;
            }
        }
    };
    if frame.table.metadata().panel.is_some() {
        let order = panel_order(ctx, &frame, "fill_forward")?;
        walk(&order.index, Some(&order.unit_of_row), &mut source);
    } else {
        let order: Vec<usize> = (0..rows).collect();
        walk(&order, None, &mut source);
    }
    Ok(Value::Column(x.take_optional(&source)))
}
