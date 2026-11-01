//! `aggregate` (catalogue row 119; Language Reference, section 6, the opening example,
//! "Statistics as methods and reducers", "Row order" and "Panels"; section 3.7, "Definitions in
//! one call"; section 4.3, rule 3; release 0.6.0, step 12). The interpreter evaluates each
//! definition once per group of `by:` on the group's rows, as it does for `add`, and hands the
//! verb each result spread back over the rows; the verb regroups the rows through the kernel's
//! groups, keeps the first row of each group of every definition and of the key columns, and
//! builds the frame: the keys first, in key order with `NA` last, then the definitions in order.
//! A definition that gives a column is refused by the checker (the definitions type of the
//! registry) and, should one arrive, here. The keys keep their metadata; a definition starts
//! empty; the panel declaration is dropped with a note.

use zeff_columns::{Column, ColumnType, Groups, Key, Table};
use zeff_core::lift::broadcast;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;

use crate::verbs::{derived, resolve_vars, the_frame, vars_of};

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

/// `data.aggregate(name = expr, ..., by:)`: one row per group with the definitions' values, or
/// one row without `by:`.
pub fn aggregate(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "aggregate", &args)?.clone();
    let rows = frame.table.rows();
    let by = vars_of(&args, 1);
    let positions = resolve_vars(ctx, &frame, &by, 1)?;
    let key_columns: Vec<&Column> = positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    // The first row of each group, in key order; one row without `by:`, none for an empty frame.
    let (groups, firsts): (Option<Groups>, Vec<usize>) = if by.is_empty() {
        (None, Vec::new())
    } else {
        let groups = Groups::by(&key_columns).map_err(|e| internal(ctx, format!("{e:?}")))?;
        let firsts = (0..groups.len()).map(|g| groups.rows(g)[0]).collect();
        (Some(groups), firsts)
    };
    let result_rows = if by.is_empty() { 1 } else { firsts.len() };
    let mut columns: Vec<(String, Column)> = Vec::with_capacity(by.len() + args.definitions.len());
    for (&p, column) in positions.iter().zip(&key_columns) {
        columns.push((frame.table.names()[p].clone(), column.take(&firsts)));
    }
    for (i, (name, value)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        if by.contains(name) {
            return Err(fail(
                Diagnostic::error(
                    "Z0405",
                    format!("`{name}` is both a `by:` variable and a definition of `aggregate`"),
                )
                .in_file(ctx.script)
                .primary(span, "defined again")
                .help("the `by:` variables are the first columns of the result; give the definition another name"),
            ));
        }
        let column = match (value, &groups) {
            (Value::Column(c), Some(groups)) if c.len() == rows => {
                // One value per group: every row of a group holds the group's value.
                for g in 0..groups.len() {
                    let members = groups.rows(g);
                    let first = Key::of(c, members[0]);
                    if members.iter().any(|&r| Key::of(c, r) != first) {
                        return Err(column_valued(ctx, name, span));
                    }
                }
                c.take(&firsts)
            }
            // Without `by:` an earlier definition reaches a later one as a column of the frame's
            // length holding its one value in every row: one group of every row.
            (Value::Column(c), None) if c.len() == rows => {
                if rows == 0 {
                    Column::missing(c.ty(), 1)
                } else {
                    let first = Key::of(c, 0);
                    if (1..rows).any(|r| Key::of(c, r) != first) {
                        return Err(column_valued(ctx, name, span));
                    }
                    c.take(&[0])
                }
            }
            (Value::Column(_), _) => return Err(column_valued(ctx, name, span)),
            // `NA` in every group, or on an empty frame, has no type of its own: a Float column.
            (Value::Na, _) => Column::missing(ColumnType::Float, result_rows),
            (scalar, _) => broadcast(scalar, result_rows)
                .ok_or_else(|| internal(ctx, format!("`{name}` is not a value a column holds")))?,
        };
        columns.push((name.clone(), column));
    }
    let table = Table::new(columns).map_err(|e| internal(ctx, e))?;
    let mut metadata = frame.table.metadata().clone();
    if let Some((unit, time)) = metadata.panel.take() {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{unit} × {time}` dropped: `aggregate` summarised the rows"
        )));
    }
    Ok(derived(table.with_metadata(metadata)))
}

/// Z0401: a definition that gives a column rather than one value per group.
fn column_valued(ctx: &CallContext<'_>, name: &str, span: zeff_diagnostics::Span) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("`{name}` of `aggregate` gives a column, not one value per group"),
        )
        .in_file(ctx.script)
        .primary(span, "a value for every row")
        .help("a definition of `aggregate` reduces the group to one value, as `mean(wage)` or `sd(wage) / sqrt(n())` do; `add` keeps a value per row"),
    )
}
