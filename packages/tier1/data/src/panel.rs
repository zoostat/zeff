//! `panel` (catalogue row 120; Language Reference, section 6, "Panels"; release 0.6.0, step 7):
//! the declaration of a unit and a time variable, checked as Stata's `xtset` checks it and
//! written into the frame's metadata store, with the sorted index the panel functions use built
//! once and kept with the table.

use crate::verbs::{derived, resolve_vars, the_frame, vars_of};
use zeff_columns::{ColumnType, Groups};
use zeff_core::display::cell;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};

/// The one variable a `Var` option names, at parameter `at`; Z0406 when the option was not given.
fn one_var(ctx: &CallContext<'_>, args: &Args, at: usize, option: &str) -> Result<String, Fail> {
    match vars_of(args, at).as_slice() {
        [name] => Ok(name.clone()),
        [] => Err(fail(
            Diagnostic::error("Z0406", format!("`panel` needs `{option}`"))
                .in_file(ctx.script)
                .primary(ctx.callee_span, format!("`{option}` is missing"))
                .help("usage: `data.panel(unit: Var, time: Var)`"),
        )),
        _ => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{option}` of `panel` takes one variable in 1.0"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at), "several variables")
            .help("a composite unit (store and product) is an addition for later; `add(unit = ...)` builds one variable for now"),
        )),
    }
}

/// `data.panel(unit:, time:)`.
pub fn panel(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "panel", &args)?.clone();
    let unit = one_var(ctx, &args, 1, "unit:")?;
    let time = one_var(ctx, &args, 2, "time:")?;
    let unit_pos = resolve_vars(ctx, &frame, std::slice::from_ref(&unit), 1)?[0];
    let time_pos = resolve_vars(ctx, &frame, std::slice::from_ref(&time), 2)?[0];
    let unit_column = frame.table.column_at(unit_pos).cloned().expect("resolved");
    let time_column = frame.table.column_at(time_pos).cloned().expect("resolved");
    if let ColumnType::Opaque(_) = unit_column.ty() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`unit:` of `panel` cannot be a {} variable", unit_column.ty().name()),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "an opaque variable")
            .help("a unit is identified by a Bool, Int, Float, Str, Date, DateTime or Category variable"),
        ));
    }
    if !matches!(time_column.ty(), ColumnType::Int | ColumnType::Date) {
        return Err(fail(
            Diagnostic::error(
                "Z0601",
                format!(
                    "`time:` of `panel` must be an Int or a Date, but `{time}` is {} variable",
                    article(time_column.ty().name())
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "not a period")
            .help(format!(
                "a panel's time is a whole-number period or a calendar day; `to({time}, Int)` if the values are whole"
            )),
        ));
    }
    for (name, column, at) in [(&unit, &unit_column, 1), (&time, &time_column, 2)] {
        let missing = column.n_missing();
        if missing > 0 {
            return Err(fail(
                Diagnostic::error(
                    "Z0601",
                    format!(
                        "`{name}` has NA in {}, which a panel cannot order",
                        rows(missing)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(at), "NA among the values")
                .help(format!(
                    "`where(!is_missing({name}))` drops those rows, or `drop_missing([{unit}, {time}])`"
                )),
            ));
        }
    }
    let groups = Groups::by(&[&unit_column, &time_column]).map_err(|e| {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script ({e:?})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )
    })?;
    let repeated: Vec<usize> = (0..groups.len())
        .filter(|g| groups.rows(*g).len() > 1)
        .collect();
    if let Some(&first) = repeated.first() {
        let row = groups.rows(first)[0];
        let unit_text = cell(unit_column.get(row).unwrap_or(zeff_columns::Value::Na));
        let time_text = cell(time_column.get(row).unwrap_or(zeff_columns::Value::Na));
        let others = repeated.len() - 1;
        return Err(fail(
            Diagnostic::error(
                "Z0601",
                format!(
                    "`{unit}` {unit_text} is observed {} at `{time}` {time_text}{}",
                    times(groups.rows(first).len()),
                    match others {
                        0 => String::new(),
                        1 => "; 1 other pair repeats".to_string(),
                        n => format!("; {} other pairs repeat", count(n as u64)),
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "a unit with two rows at one time")
            .help(format!(
                "a unit has one row per time: `duplicates([{unit}, {time}])` lists the repeated pairs; drop or aggregate them first"
            )),
        ));
    }
    let mut metadata = frame.table.metadata().clone();
    metadata.panel = Some((unit, time));
    let table = (*frame.table).clone().with_metadata(metadata);
    // The index is built once, now, and kept with the table (section 8.3, point 10).
    let _ = table.panel_index();
    Ok(derived(table))
}

fn article(type_name: &str) -> String {
    let vowel = type_name.starts_with(['A', 'E', 'I', 'O', 'U']);
    format!("{} {type_name}", if vowel { "an" } else { "a" })
}

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

fn times(n: usize) -> String {
    match n {
        2 => "twice".to_string(),
        n => format!("{} times", count(n as u64)),
    }
}
