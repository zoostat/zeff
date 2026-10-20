//! The frame methods that look at a frame without computing on it (Language Reference, section
//! 6, "Look"): `n_rows`, `n_cols`, `variables` and `describe`; `head` and `tail` follow.
//!
//! Each takes the frame alone and gives no message of its own.

use zeff_columns::Value as Cell;
use zeff_core::display::{cell, frame_header, labelled_cell};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::{Diagnostic, count};

/// A group's key cells and the rows it holds, for `describe(by:)`.
type GroupBlock = (Vec<String>, Vec<usize>);

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
    let frame = the_frame(ctx, "describe", &args)?.clone();
    // With `by:`, one block of rows per group, the group's keys in the first columns.
    let grouped = crate::reducers::groups_for(ctx, &frame, &args, 1)?;
    let (key_names, group_rows): (Vec<String>, Vec<GroupBlock>) = match &grouped {
        None => (
            Vec::new(),
            vec![(Vec::new(), (0..frame.table.rows()).collect())],
        ),
        Some((keys, groups)) => {
            let key_columns: Vec<&zeff_columns::Column> =
                keys.iter().filter_map(|k| frame.table.column(k)).collect();
            let blocks = (0..groups.len())
                .map(|g| {
                    let first = groups.rows(g)[0];
                    let cells: Vec<String> = key_columns
                        .iter()
                        .map(|c| labelled_cell(c, first))
                        .collect();
                    (cells, groups.rows(g).to_vec())
                })
                .collect();
            (keys.clone(), blocks)
        }
    };
    let mut rows: Vec<Vec<String>> = Vec::new();
    for (key_cells, members) in &group_rows {
        let table = if grouped.is_some() {
            frame.table.take_rows(members)
        } else {
            (*frame.table).clone()
        };
        for row in describe_rows(&table) {
            let mut full = key_cells.clone();
            full.extend(row);
            rows.push(full);
        }
    }
    let mut columns: Vec<(String, Align)> =
        key_names.iter().map(|k| (k.clone(), Align::Left)).collect();
    columns.extend([
        ("variable".to_string(), Align::Left),
        ("type".to_string(), Align::Left),
        ("non-missing".to_string(), Align::Right),
        ("missing".to_string(), Align::Right),
        ("example".to_string(), Align::Left),
    ]);
    let title = match &grouped {
        Some((keys, _)) => format!("{} · by {}", frame_header(&frame), keys.join(", ")),
        None => frame_header(&frame),
    };
    Ok(Value::Table(Table {
        title,
        columns,
        rows,
        more: Vec::new(),
    }))
}

/// One row per variable of `table`: name, type, counts and an example value.
fn describe_rows(table: &zeff_columns::Table) -> Vec<Vec<String>> {
    table
        .names()
        .iter()
        .enumerate()
        .filter_map(|(i, name)| table.column_at(i).map(|column| (name, column)))
        .map(|(name, column)| {
            let first = (0..column.len()).find_map(|r| match column.get(r) {
                Some(Cell::Na) | None => None,
                Some(value) => Some(value),
            });
            let first_row = (0..column.len()).find(|r| !column.is_na(*r));
            let example = match (first, first_row) {
                (None, _) => "NA".to_string(),
                (Some(Cell::Category(label)), _) => {
                    let levels = column.levels().map_or(0, |l| l.len());
                    format!(
                        "{label} ({} {})",
                        count(levels as u64),
                        if levels == 1 { "level" } else { "levels" }
                    )
                }
                (Some(_), Some(row)) => labelled_cell(column, row),
                (Some(value), None) => cell(value),
            };
            // A labelled variable (value labels in its store) is marked, as section 10.2 prints it.
            let ty = if column
                .metadata()
                .get(zeff_columns::MetadataKey::ValueLabels)
                .is_some()
            {
                format!("{} (lab)", column.ty().name())
            } else {
                column.ty().name().to_string()
            };
            vec![
                name.clone(),
                ty,
                count(column.n_valid() as u64),
                count(column.n_missing() as u64),
                example,
            ]
        })
        .collect()
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
