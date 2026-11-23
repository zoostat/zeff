//! `distinct`, `duplicates` and `sample` (catalogue rows 112 to 114; Language Reference, section
//! 6, "Rows" and "Row order"; release 0.6.0, step 11). `distinct` and `duplicates` group the rows
//! by identity (section 3.7: `NA` identical to `NA`) through the kernel's groups and keep the
//! first occurrence in the current row order; `duplicates` prints Stata's `duplicates report` as
//! a table, in Rust rather than the planned Zeff because `aggregate` and `table` were not yet
//! built at this step. `sample` draws from the run's random stream (Engine and Project
//! Specification, section 8.3, point 3), a partial Fisher–Yates over the row positions without
//! replacement and `n` draws with it, and returns the kept rows in their original order as a
//! selection, so the panel declaration and every column's metadata survive.

use zeff_columns::{Column, GroupError, Groups};
use zeff_core::display::labelled_cell;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::{Diagnostic, count};

use crate::verbs::{derived, resolve_vars, the_frame, vars_of};

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// The key columns a verb groups by: the variables named.
struct Keys<'a> {
    names: Vec<String>,
    columns: Vec<&'a Column>,
    /// How the keys read in a note: `firm, year`.
    listed: String,
}

fn keys<'a>(ctx: &CallContext<'_>, frame: &'a Frame, args: &Args) -> Result<Keys<'a>, Fail> {
    let positions = resolve_vars(ctx, frame, &vars_of(args, 1), 1)?;
    let names: Vec<String> = positions
        .iter()
        .map(|&p| frame.table.names()[p].clone())
        .collect();
    let columns: Vec<&Column> = positions
        .iter()
        .filter_map(|&p| frame.table.column_at(p))
        .collect();
    let listed = names.join(", ");
    Ok(Keys {
        names,
        columns,
        listed,
    })
}

/// The groups of the rows under the keys, or Z0401 for a column type that has no identity yet.
fn groups_of(ctx: &CallContext<'_>, verb: &str, keys: &Keys<'_>) -> Result<Groups, Fail> {
    Groups::by(&keys.columns).map_err(|e| match e {
        GroupError::OpaqueKey(ty) => fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{verb}` compares values by identity, which a Column<{}> does not have yet",
                    ty.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "no identity to compare")
            .help("name the computable variables; the opaque column is kept through the rows"),
        ),
        other => fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script ({other:?})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ),
    })
}

/// `data.distinct(vars)`: the rows with distinct values of the variables, the first occurrence in the current row order kept with every column, and a
/// note of how many repeats were dropped.
pub fn distinct(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "distinct", &args)?;
    let keys = keys(ctx, frame, &args)?;
    let groups = groups_of(ctx, "distinct", &keys)?;
    let mut kept: Vec<usize> = (0..groups.len()).map(|g| groups.rows(g)[0]).collect();
    kept.sort_unstable();
    let dropped = frame.table.rows() - kept.len();
    if dropped > 0 {
        ctx.message(Diagnostic::plain_note(format!(
            "{} dropped as {} of {}; {} distinct",
            rows(dropped),
            if dropped == 1 { "a repeat" } else { "repeats" },
            keys.listed,
            rows(kept.len())
        )));
    }
    Ok(derived(frame.table.select_rows(kept, frame.table.token())))
}

/// `data.duplicates(vars)`: the table of repeated key values, one row per value with more than
/// one copy, in key order, as Stata's `duplicates report`; a note when there is none.
pub fn duplicates(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "duplicates", &args)?;
    let keys = keys(ctx, frame, &args)?;
    let groups = groups_of(ctx, "duplicates", &keys)?;
    let mut rows_out: Vec<Vec<String>> = Vec::new();
    // The same rows as data (open decision #170; release 0.9.0, step 33c): the first row of
    // each repeated value, for the key variables in their own types, and its copies.
    let mut firsts: Vec<usize> = Vec::new();
    let mut copies: Vec<Option<i64>> = Vec::new();
    for g in 0..groups.len() {
        let members = groups.rows(g);
        if members.len() > 1 {
            let mut row: Vec<String> = keys
                .columns
                .iter()
                .map(|c| labelled_cell(c, members[0]))
                .collect();
            row.push(count(members.len() as u64));
            rows_out.push(row);
            firsts.push(members[0]);
            copies.push(Some(members.len() as i64));
        }
    }
    let mut behind: Vec<(String, Column)> = keys
        .names
        .iter()
        .zip(&keys.columns)
        .map(|(name, column)| (name.clone(), column.take(&firsts)))
        .collect();
    // A key variable already called `copies` keeps its name; the count takes the next free one.
    let mut count_name = "copies".to_string();
    while behind.iter().any(|(name, _)| *name == count_name) {
        count_name.push('_');
    }
    behind.push((count_name, Column::ints(copies)));
    let behind = zeff_columns::Table::new(behind)
        .ok()
        .map(|table| Box::new(zeff_core::value::Behind(Frame::new(table))));
    if rows_out.is_empty() {
        ctx.message(Diagnostic::plain_note(format!(
            "no repeated values of {}",
            keys.listed
        )));
    }
    let mut columns: Vec<(String, Align)> = keys
        .names
        .iter()
        .map(|k| (k.clone(), Align::Left))
        .collect();
    columns.push(("copies".to_string(), Align::Right));
    Ok(Value::Table(Table {
        behind,
        title: format!("duplicates · {}", keys.names.join(", ")),
        columns,
        rows: rows_out,
        more: Vec::new(),
        rules: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    }))
}

/// `data.sample(n: | frac:, with_replacement:, seed:)`: a random sample of the rows from the
/// run's stream, or from a stream of `seed:`, the kept rows in their original order, with a note
/// giving the count and the seed.
pub fn sample(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "sample", &args)?.clone();
    let shown = frame.shown_name();
    let total = frame.table.rows();
    let refuse = |at: usize, message: String, label: &str, help: &str| {
        fail(
            Diagnostic::error("Z0401", message)
                .in_file(ctx.script)
                .primary(ctx.span_of(at), label)
                .help(help),
        )
    };
    let with_replacement = matches!(args.get(3), Some(Value::Bool(true)));
    let n: usize = match (args.get(1), args.get(2)) {
        (Some(_), Some(_)) => {
            return Err(fail(
                Diagnostic::error("Z0410", "`sample` takes `n:` or `frac:`, not both")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "`frac:` beside `n:`")
                    .help("`n:` counts rows and `frac:` is a share of them; keep the one you mean"),
            ));
        }
        (None, None) => {
            return Err(fail(
                Diagnostic::error("Z0406", "`sample` needs `n:` or `frac:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "how many rows?")
                    .help(
                        "usage: `data.sample(n: 500, seed: 20261015)` or `data.sample(frac: 0.1)`",
                    ),
            ));
        }
        (Some(Value::Int(n)), None) if *n >= 0 => *n as usize,
        (Some(other), None) => {
            return Err(refuse(
                1,
                format!(
                    "`n:` of `sample` is a count of rows of at least 0, but this is {}",
                    match other {
                        Value::Int(n) => n.to_string(),
                        Value::Na => "NA".to_string(),
                        other => other.type_word(),
                    }
                ),
                "not a count of rows",
                "`sample(n: 500)` keeps 500 rows chosen at random",
            ));
        }
        (None, Some(frac)) => {
            let share = match frac {
                Value::Float(f) => Some(*f),
                Value::Int(i) => Some(*i as f64),
                _ => None,
            };
            match share {
                Some(f) if (0.0..=1.0).contains(&f) => (f * total as f64).round() as usize,
                _ => {
                    return Err(refuse(
                        2,
                        format!(
                            "`frac:` of `sample` is a share between 0 and 1, but this is {}",
                            match frac {
                                Value::Float(f) => zeff_core::display::format_float(*f),
                                Value::Int(i) => i.to_string(),
                                Value::Na => "NA".to_string(),
                                other => other.type_word(),
                            }
                        ),
                        "not a share",
                        "`sample(frac: 0.1)` keeps a tenth of the rows, rounded to a whole number",
                    ));
                }
            }
        }
    };
    if !with_replacement && n > total {
        return Err(refuse(
            1,
            format!(
                "`n:` of `sample` is {}, but {shown} has {}",
                count(n as u64),
                rows(total)
            ),
            "more rows than the frame has",
            "a sample without replacement is at most the frame; `with_replacement: true` allows repeats",
        ));
    }
    let seed = match args.get(4) {
        Some(Value::Int(s)) => Some(*s),
        _ => None,
    };
    let mut stream = ctx.stream(seed);
    let index = |u: f64, k: usize| ((u * k as f64) as usize).min(k - 1);
    let mut kept: Vec<usize> = if with_replacement {
        (0..n)
            .map(|_| {
                if total == 0 {
                    0
                } else {
                    index(stream.uniform(), total)
                }
            })
            .collect()
    } else {
        // A partial Fisher–Yates: the first `n` positions of a shuffle, each swapped in from the
        // positions not yet chosen.
        let mut positions: Vec<usize> = (0..total).collect();
        for i in 0..n {
            let j = i + index(stream.uniform(), total - i);
            positions.swap(i, j);
        }
        positions.truncate(n);
        positions
    };
    if total == 0 {
        kept.clear();
    }
    kept.sort_unstable();
    let seed_used = seed.or(ctx.random.seed).unwrap_or_default();
    // A draw from the run's seed in a pass of a loop, or after another draw of the same
    // statement, has a stream of its own, derived from that seed (Engine and Project
    // Specification, section 8.3, point 3): the note says so, and does not give the run's seed
    // as if it were the draw's (release 0.8.0, step 8o). With `seed:` the draw is that seed's.
    let derived_stream = seed.is_none()
        && (ctx.random.pass > 0 || ctx.random.repeating > 0 || ctx.random.calls_in_statement > 1);
    ctx.message(Diagnostic::plain_note(format!(
        "{} of {} sampled{} with {}",
        count(kept.len() as u64),
        rows(total),
        if with_replacement {
            " with replacement"
        } else {
            ""
        },
        if derived_stream {
            format!("a seed of its own, derived from {seed_used}")
        } else {
            format!("seed {seed_used}")
        }
    )));
    let repeated = kept.windows(2).any(|w| w[0] == w[1]);
    let mut table = frame.table.select_rows(kept, frame.table.token());
    if repeated && let Some((unit, time)) = table.metadata().panel.clone() {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{unit} × {time}` dropped: `sample` with replacement repeated rows"
        )));
        let mut metadata = table.metadata().clone();
        metadata.panel = None;
        table = table.with_metadata(metadata);
    }
    Ok(derived(table))
}
