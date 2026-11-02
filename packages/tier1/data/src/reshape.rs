//! `append`, `to_long` and `to_wide` (catalogue rows 116 to 118; Language Reference, section 6,
//! "Combine", "Reshape and summarise", "Row order" and "Panels"; section 4.3; release 0.6.0,
//! step 14). `append` stacks a frame's rows below another's by column name, filling a column
//! one frame lacks with `NA`; `to_long` turns the columns that begin with a stub into rows, the
//! rest of each name parsed as `names_type:` says; `to_wide` turns rows into columns named
//! `<values_from>_<value>`, the inverse of the stub, so that the two round-trip (question 5 (d)
//! of the release). The three are exact: nothing is computed, values are moved.

use zeff_columns::{Column, ColumnType, Groups, Table, Value as Cell};
use zeff_core::display::labelled_cell;
use zeff_core::lift::collect_column;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::{Diagnostic, count};

use crate::verbs::{derived, resolve_vars, the_frame, vars_of};

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
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

/// A cell of the kernel as a value of the language.
fn to_value(cell: Option<Cell<'_>>) -> Value {
    match cell {
        None | Some(Cell::Na) | Some(Cell::Opaque(_)) => Value::Na,
        Some(Cell::Bool(b)) => Value::Bool(b),
        Some(Cell::Int(i)) => Value::Int(i),
        Some(Cell::Float(x)) => Value::Float(x),
        Some(Cell::Str(t)) | Some(Cell::Category(t)) => Value::Str(t.to_string()),
        Some(Cell::Date(d)) => Value::Date(d),
        Some(Cell::DateTime(t)) => Value::DateTime(t),
    }
}

/// Whether two column types join in a stack: the same type, or Int with Float.
fn joins(a: ColumnType, b: ColumnType) -> bool {
    a == b
        || matches!(
            (a, b),
            (ColumnType::Int, ColumnType::Float) | (ColumnType::Float, ColumnType::Int)
        )
}

/// The values of several columns stacked, each column's rows in turn (`None` for a column that
/// is absent, giving `n` rows of `NA`): a category's levels the union in first-seen order,
/// everything else through the language's values, the first column's metadata kept.
fn stack(parts: &[(Option<&Column>, usize)], ty: ColumnType) -> Option<Column> {
    let total: usize = parts.iter().map(|(_, n)| n).sum();
    if ty == ColumnType::Category {
        let mut levels: Vec<String> = Vec::new();
        let mut codes: Vec<Option<i32>> = Vec::with_capacity(total);
        for (column, n) in parts {
            match column {
                Some(c) => {
                    let own: Vec<String> = c
                        .levels()
                        .unwrap_or_default()
                        .iter()
                        .map(|l| l.to_string())
                        .collect();
                    for level in &own {
                        if !levels.contains(level) {
                            levels.push(level.clone());
                        }
                    }
                    let positions: Vec<i32> = own
                        .iter()
                        .map(|l| levels.iter().position(|x| x == l).unwrap_or(0) as i32)
                        .collect();
                    let own_codes: Vec<Option<i32>> =
                        c.as_codes().map(|it| it.collect()).unwrap_or_default();
                    codes.extend(
                        own_codes
                            .into_iter()
                            .map(|code| code.map(|k| positions[k as usize])),
                    );
                }
                None => codes.extend(std::iter::repeat_n(None, *n)),
            }
        }
        return Column::categories(levels, codes).ok();
    }
    let mut values: Vec<Value> = Vec::with_capacity(total);
    for (column, n) in parts {
        match column {
            Some(c) => values.extend((0..c.len()).map(|r| to_value(c.get(r)))),
            None => values.extend(std::iter::repeat_n(Value::Na, *n)),
        }
    }
    collect_column(&values).or_else(|| Some(Column::missing(ty, total)))
}

/// `data.append(frame)`: the right frame's rows below the left's, columns matched by name.
pub fn append(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let left = the_frame(ctx, "append", &args)?.clone();
    let right: Frame = match args.get(1) {
        Some(Value::Frame(f)) => f.clone(),
        _ => return Err(internal(ctx, "append was called without the frame to add")),
    };
    let (left_rows, right_rows) = (left.table.rows(), right.table.rows());
    let mut names: Vec<String> = left.table.names().to_vec();
    for name in right.table.names() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    let mut columns: Vec<(String, Column)> = Vec::with_capacity(names.len());
    let (mut left_only, mut right_only): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for name in &names {
        let l = left.table.column(name);
        let r = right.table.column(name);
        let ty = match (l, r) {
            (Some(l), Some(r)) => {
                if !joins(l.ty(), r.ty()) {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0609",
                            format!(
                                "`{name}` is a column of {} in {} and of {} in {}",
                                l.ty().name(),
                                left.shown_name(),
                                r.ty().name(),
                                right.shown_name()
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(1), "a column of another type")
                        .help("the rows of one column have one type; convert one side first with `to`, or `drop` the column from one frame"),
                    ));
                }
                if l.ty() == ColumnType::Float || r.ty() == ColumnType::Float {
                    ColumnType::Float
                } else {
                    l.ty()
                }
            }
            (Some(l), None) => {
                left_only.push(name.clone());
                l.ty()
            }
            (None, Some(r)) => {
                right_only.push(name.clone());
                r.ty()
            }
            (None, None) => unreachable!("a name of one of the frames"),
        };
        let stacked = stack(&[(l, left_rows), (r, right_rows)], ty)
            .ok_or_else(|| internal(ctx, format!("`{name}` could not be stacked")))?;
        let metadata = l.or(r).map(|c| c.metadata().clone());
        let stacked = match metadata {
            Some(m) => stacked.with_metadata(m),
            None => stacked,
        };
        columns.push((name.clone(), stacked));
    }
    let listed = |names: &[String]| {
        names
            .iter()
            .map(|n| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut note = format!(
        "append: {} added below {}",
        rows(right_rows),
        rows(left_rows)
    );
    if !left_only.is_empty() {
        note.push_str(&format!(
            "; {} only in {}, filled with NA below",
            listed(&left_only),
            left.shown_name()
        ));
    }
    if !right_only.is_empty() {
        note.push_str(&format!(
            "; {} only in {}, filled with NA above",
            listed(&right_only),
            right.shown_name()
        ));
    }
    ctx.message(Diagnostic::plain_note(note));
    let table = Table::new(columns).map_err(|e| internal(ctx, e))?;
    let mut metadata = left.table.metadata().clone();
    if let Some(panel) = metadata.panel.clone()
        && right.table.metadata().panel.as_ref() != Some(&panel)
    {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{} × {}` dropped: {} has no such declaration",
            panel.0,
            panel.1,
            right.shown_name()
        )));
        metadata.panel = None;
    }
    Ok(derived(table.with_metadata(metadata)))
}

/// A required text option, or Z0406.
fn text_option(
    ctx: &CallContext<'_>,
    verb: &str,
    args: &Args,
    at: usize,
    name: &str,
    usage: &str,
) -> Result<String, Fail> {
    match args.get(at) {
        Some(Value::Str(s)) => Ok(s.clone()),
        _ => Err(fail(
            Diagnostic::error("Z0406", format!("`{verb}` needs `{name}:`"))
                .in_file(ctx.script)
                .primary(ctx.callee_span, format!("`{name}:` is missing"))
                .help(format!("usage: `{usage}`")),
        )),
    }
}

/// `data.to_long(stub, id:, names_to:, values_to:, names_type:)`.
pub fn to_long(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "to_long", &args)?.clone();
    let usage = "data.to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Int)";
    let stub = match args.get(1) {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(internal(ctx, "to_long was called without its stub")),
    };
    let ids = vars_of(&args, 2);
    let id_positions = resolve_vars(ctx, &frame, &ids, 2)?;
    let names_to = text_option(ctx, "to_long", &args, 3, "names_to", usage)?;
    let values_to = text_option(ctx, "to_long", &args, 4, "values_to", usage)?;
    let names_type = match args.get(5) {
        Some(Value::Type(t)) if matches!(t.name.as_str(), "Str" | "Int" | "Date") => t.name.clone(),
        Some(Value::Type(t)) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`names_type:` of `to_long` is Str, Int or Date, not {}", t.name),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(5), "a type the names cannot take")
                .help("the rest of each name reads as text, a whole number or a date written 2010-01-31"),
            ));
        }
        _ => "Str".to_string(),
    };
    let names = frame.table.names();
    let stub_positions: Vec<usize> = (0..names.len())
        .filter(|&p| names[p].starts_with(&stub) && !ids.contains(&names[p]))
        .collect();
    if stub_positions.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0610",
                format!("no variable of {} begins with `{stub}`", frame.shown_name()),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "a stub no name begins with")
            .help(format!(
                "the variables are: {}; the stub is the start the repeated names share, such as `sales_` for `sales_2010`",
                names.join(", ")
            )),
        ));
    }
    let kept_positions: Vec<usize> = (0..names.len())
        .filter(|p| !stub_positions.contains(p) && !id_positions.contains(p))
        .collect();
    for taken in [&names_to, &values_to] {
        if id_positions
            .iter()
            .chain(&kept_positions)
            .any(|&p| &names[p] == taken)
        {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`{taken}` is already a variable of {}", frame.shown_name()),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a new name that is taken")
                .help("give `names_to:` and `values_to:` names the frame does not have"),
            ));
        }
    }
    // The stub columns share a type, Int with Float joining.
    let stub_columns: Vec<&Column> = stub_positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    let mut ty = stub_columns[0].ty();
    for (&p, c) in stub_positions.iter().zip(&stub_columns) {
        if !joins(ty, c.ty()) {
            return Err(fail(
                Diagnostic::error(
                    "Z0610",
                    format!(
                        "the columns beginning with `{stub}` do not share a type: `{}` is {} and `{}` is {}",
                        names[stub_positions[0]],
                        stub_columns[0].ty().name(),
                        names[p],
                        c.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "columns of several types")
                .help("one value column has one type: convert the columns with `to` first, or leave the odd one out of the stub"),
            ));
        }
        if c.ty() == ColumnType::Float {
            ty = ColumnType::Float;
        }
    }
    // The names, parsed as `names_type:` says.
    let mut name_values: Vec<Value> = Vec::with_capacity(stub_positions.len());
    for &p in &stub_positions {
        let rest = &names[p][stub.len()..];
        let parsed = match names_type.as_str() {
            "Int" => rest.parse::<i64>().ok().map(Value::Int),
            "Date" => zeff_dates::parse_iso_date(rest).map(Value::Date),
            _ => Some(Value::Str(rest.to_string())),
        };
        match parsed {
            Some(v) => name_values.push(v),
            None => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0610",
                        format!(
                            "`{}` does not end in {}: `{rest}` after `{stub}`",
                            names[p],
                            match names_type.as_str() {
                                "Int" => "a whole number",
                                "Date" => "a date written as 2010-01-31",
                                _ => "text",
                            }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(5), "the type the names do not fit")
                    .help("`names_type: Str` keeps the rest of each name as text; `rename` a column whose name is wrong"),
                ));
            }
        }
    }
    let n = frame.table.rows();
    let k = stub_positions.len();
    // Each original row, each stub column in column order.
    let row_of: Vec<usize> = (0..n).flat_map(|r| std::iter::repeat_n(r, k)).collect();
    let mut columns: Vec<(String, Column)> = Vec::new();
    for &p in id_positions.iter().chain(&kept_positions) {
        let column = frame.table.column_at(p).expect("a column of the frame");
        columns.push((names[p].clone(), column.take(&row_of)));
    }
    let name_column: Vec<Value> = (0..n).flat_map(|_| name_values.iter().cloned()).collect();
    columns.push((
        names_to.clone(),
        collect_column(&name_column).ok_or_else(|| internal(ctx, "the names column"))?,
    ));
    let mut values: Vec<Value> = Vec::with_capacity(n * k);
    for r in 0..n {
        for c in &stub_columns {
            values.push(to_value(c.get(r)));
        }
    }
    let value_column = if ty == ColumnType::Category {
        stack(
            &stub_columns
                .iter()
                .map(|c| (Some(*c), c.len()))
                .collect::<Vec<_>>(),
            ty,
        )
        .map(|stacked| {
            // Stacked column by column; reordered row-major.
            let order: Vec<usize> = (0..n)
                .flat_map(|r| (0..k).map(move |j| j * n + r))
                .collect();
            stacked.take(&order)
        })
    } else {
        collect_column(&values)
    }
    .unwrap_or_else(|| Column::missing(ty, n * k));
    columns.push((values_to.clone(), value_column));
    ctx.message(Diagnostic::plain_note(format!(
        "to_long: {} of `{stub}` into {} as `{names_to}` and `{values_to}`",
        match k {
            1 => "1 column".to_string(),
            k => format!("{k} columns"),
        },
        rows(n * k)
    )));
    let table = Table::new(columns).map_err(|e| internal(ctx, e))?;
    let mut metadata = frame.table.metadata().clone();
    if let Some((unit, time)) = metadata.panel.clone()
        && (!ids.contains(&unit) && !kept_positions.iter().any(|&p| names[p] == unit)
            || !ids.contains(&time) && !kept_positions.iter().any(|&p| names[p] == time))
    {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{unit} × {time}` dropped: `to_long` reshaped the rows"
        )));
        metadata.panel = None;
    } else if metadata.panel.is_some() {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{}` dropped: `to_long` repeated the rows",
            metadata
                .panel
                .as_ref()
                .map(|(u, t)| format!("{u} × {t}"))
                .unwrap_or_default()
        )));
        metadata.panel = None;
    }
    Ok(derived(table.with_metadata(metadata)))
}

/// `data.to_wide(values_from:, id:, names_from:)`.
pub fn to_wide(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "to_wide", &args)?.clone();
    let usage = "data.to_wide(values_from: sales, id: firm, names_from: year)";
    let values_from = match vars_of(&args, 1).as_slice() {
        [name] => name.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`to_wide` needs `values_from:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`values_from:` is missing")
                    .help(format!("usage: `{usage}`")),
            ));
        }
    };
    let ids = vars_of(&args, 2);
    if ids.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`to_wide` needs `id:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`id:` is missing")
                .help(format!("usage: `{usage}`")),
        ));
    }
    let names_from = match vars_of(&args, 3).as_slice() {
        [name] => name.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`to_wide` needs `names_from:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`names_from:` is missing")
                    .help(format!("usage: `{usage}`")),
            ));
        }
    };
    let value_position = resolve_vars(ctx, &frame, std::slice::from_ref(&values_from), 1)?[0];
    let id_positions = resolve_vars(ctx, &frame, &ids, 2)?;
    let name_position = resolve_vars(ctx, &frame, std::slice::from_ref(&names_from), 3)?[0];
    let names = frame.table.names();
    let value_column = frame
        .table
        .column_at(value_position)
        .expect("the values column");
    let name_column = frame
        .table
        .column_at(name_position)
        .expect("the names column");
    let id_columns: Vec<&Column> = id_positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    let id_groups = Groups::by(&id_columns).map_err(|e| internal(ctx, format!("{e:?}")))?;
    let name_groups = Groups::by(&[name_column]).map_err(|e| internal(ctx, format!("{e:?}")))?;
    // The ids in order of first appearance; the names in key order, `NA` last.
    let mut id_order: Vec<usize> = (0..id_groups.len()).collect();
    id_order.sort_by_key(|&g| id_groups.rows(g)[0]);
    let id_of_row = id_groups.group_of_row();
    let name_of_row = name_groups.group_of_row();
    let (n_ids, n_names) = (id_groups.len(), name_groups.len());
    let mut slot: Vec<Option<usize>> = vec![None; n_ids * n_names];
    for r in 0..frame.table.rows() {
        let at = id_of_row[r] * n_names + name_of_row[r];
        if let Some(first) = slot[at] {
            let id_text: Vec<String> = id_columns.iter().map(|c| labelled_cell(c, first)).collect();
            return Err(fail(
                Diagnostic::error(
                    "Z0611",
                    format!(
                        "`{}` {} appears twice at `{names_from}` {}",
                        ids.join(", "),
                        id_text.join(", "),
                        labelled_cell(name_column, first)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "an id with two rows for one name")
                .help(format!(
                    "`duplicates([{}, {names_from}])` lists the repeated pairs; `aggregate` them or add the variable that tells them apart to `id:`",
                    ids.join(", ")
                )),
            ));
        }
        slot[at] = Some(r);
    }
    let mut columns: Vec<(String, Column)> = Vec::new();
    let firsts: Vec<usize> = id_order.iter().map(|&g| id_groups.rows(g)[0]).collect();
    for (&p, column) in id_positions.iter().zip(&id_columns) {
        columns.push((names[p].clone(), column.take(&firsts)));
    }
    let mut new_names: Vec<String> = Vec::with_capacity(n_names);
    for g in 0..n_names {
        let first = name_groups.rows(g)[0];
        let name = format!("{values_from}_{}", labelled_cell(name_column, first));
        let picks: Vec<Option<usize>> = id_order.iter().map(|&id| slot[id * n_names + g]).collect();
        new_names.push(name.clone());
        columns.push((name, value_column.take_optional(&picks)));
    }
    let left_out: Vec<&String> = (0..names.len())
        .filter(|&p| p != value_position && p != name_position && !id_positions.contains(&p))
        .map(|p| &names[p])
        .collect();
    let mut note = format!(
        "to_wide: {} of `{values_from}` by `{names_from}`, {} of {}",
        match n_names {
            1 => "1 column".to_string(),
            k => format!("{k} columns"),
        },
        rows(n_ids),
        match ids.len() {
            1 => format!("`{}`", ids[0]),
            _ => format!("`{}`", ids.join(", ")),
        }
    );
    if !left_out.is_empty() {
        note.push_str(&format!(
            "; {} left out: name {} in `id:` to keep {}",
            left_out
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if left_out.len() == 1 { "it" } else { "them" },
            if left_out.len() == 1 { "it" } else { "them" }
        ));
    }
    ctx.message(Diagnostic::plain_note(note));
    let table = Table::new(columns).map_err(|e| {
        fail(
            Diagnostic::error(
                "Z0611",
                format!("two values of `{names_from}` give one column name: {e}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(3), "values that print alike")
            .help("the new columns are named `<values_from>_<value>`; make the values distinct as text first"),
        )
    })?;
    let mut metadata = frame.table.metadata().clone();
    if let Some((unit, time)) = metadata.panel.take() {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{unit} × {time}` dropped: `to_wide` reshaped the rows"
        )));
    }
    Ok(derived(table.with_metadata(metadata)))
}
