//! `merge` (catalogue row 115; Language Reference, section 6, "Combine" and "Row order"; section
//! 4.1, "Merge keys"; section 3.7, "Resolution by parameter type"; section 11, Z0603; release
//! 0.6.0, step 13): a join of two frames on key variables present in both with the same type.
//! The right frame's rows are indexed by their key values, `NA` keys never matching; the result
//! keeps the left frame's order and, within one left row, the right frame's; `relation:` is
//! checked before joining and a many-to-many relation is always refused; a non-key column both
//! frames share is refused unless `suffixes:` tells the two apart. Every merge notes how many
//! rows matched and how many did not on each side.

use std::collections::HashMap;
use zeff_columns::{Column, ColumnType, Table, Value as Cell};
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

/// A key value as the join compares it: by value, a category by its label, `-0.0` as `0.0`;
/// `None` for `NA` or an opaque value, which never match.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MergeKey {
    Bool(bool),
    Int(i64),
    Float(u64),
    Text(String),
    Date(i32),
    DateTime(i64),
}

fn key_of(column: &Column, row: usize) -> Option<MergeKey> {
    Some(match column.get(row)? {
        Cell::Na | Cell::Opaque(_) => return None,
        Cell::Bool(b) => MergeKey::Bool(b),
        Cell::Int(i) => MergeKey::Int(i),
        Cell::Float(x) => MergeKey::Float(if x == 0.0 { 0.0f64 } else { x }.to_bits()),
        Cell::Str(s) => MergeKey::Text(s.to_string()),
        Cell::Date(d) => MergeKey::Date(d),
        Cell::DateTime(t) => MergeKey::DateTime(t),
        Cell::Category(label) => MergeKey::Text(label.to_string()),
    })
}

/// The keys of a row, or `None` when any is `NA`.
fn keys_of(columns: &[&Column], row: usize) -> Option<Vec<MergeKey>> {
    columns.iter().map(|c| key_of(c, row)).collect()
}

/// `data.merge(frame, on:, join:, relation:, suffixes:)`.
pub fn merge(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let left = the_frame(ctx, "merge", &args)?.clone();
    let right: Frame = match args.get(1) {
        Some(Value::Frame(f)) => f.clone(),
        _ => return Err(internal(ctx, "merge was called without the right frame")),
    };
    let (left_shown, right_shown) = (left.shown_name(), right.shown_name());
    let on = vars_of(&args, 2);
    if on.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`merge` needs `on:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`on:` is missing")
                .help(
                    "usage: `data.merge(frame, on: firm)`; the key variables exist in both frames",
                ),
        ));
    }
    let left_positions = resolve_vars(ctx, &left, &on, 2)?;
    let right_positions = resolve_vars(ctx, &right, &on, 2)?;
    let left_keys: Vec<&Column> = left_positions
        .iter()
        .filter_map(|&p| left.table.column_at(p))
        .collect();
    let right_keys: Vec<&Column> = right_positions
        .iter()
        .filter_map(|&p| right.table.column_at(p))
        .collect();
    for ((name, l), r) in on.iter().zip(&left_keys).zip(&right_keys) {
        if l.ty() != r.ty() || matches!(l.ty(), ColumnType::Opaque(_)) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` is a column of {} in {left_shown} and of {} in {right_shown}",
                        l.ty().name(),
                        r.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "keys of different types")
                .help("the key variables have one type in both frames: convert one side first, `to(x, Str)` for text or `category(x)` for a category"),
            ));
        }
    }
    let join = match args.get(3) {
        Some(Value::Word(w)) => w.clone(),
        _ => "left".to_string(),
    };
    let relation = match args.get(4) {
        Some(Value::Word(w)) => Some(w.clone()),
        _ => None,
    };
    let suffixes: Option<(String, String)> = match args.get(5) {
        None => None,
        Some(Value::List(items)) => match items.as_slice() {
            [Value::Str(a), Value::Str(b)] => Some((a.clone(), b.clone())),
            _ => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`suffixes:` of `merge` is a list of two texts, but this has {}",
                            match items.len() {
                                1 => "one item".to_string(),
                                n => format!("{n} items"),
                            }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(5), "not two suffixes")
                    .help("the first suffix goes on the left frame's column, the second on the right's: `suffixes: [\"_l\", \"_r\"]`"),
                ));
            }
        },
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`suffixes:` of `merge` is a list of two texts, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(5), "not a list of two texts")
                .help("`suffixes: [\"_l\", \"_r\"]`"),
            ));
        }
    };
    // The shared non-key columns.
    let shared: Vec<String> = left
        .table
        .names()
        .iter()
        .filter(|n| !on.contains(n) && right.table.position(n).is_some())
        .cloned()
        .collect();
    if !shared.is_empty() && suffixes.is_none() {
        return Err(fail(
            Diagnostic::error(
                "Z0604",
                format!(
                    "{} {} {} in both frames and {} not a key",
                    if shared.len() == 1 {
                        "column"
                    } else {
                        "columns"
                    },
                    shared
                        .iter()
                        .map(|s| format!("`{s}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if shared.len() == 1 { "exists" } else { "exist" },
                    if shared.len() == 1 { "is" } else { "are" }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "shares a column with the left frame")
            .help("give suffixes: [\"_l\", \"_r\"] to keep both, or drop one of them first"),
        ));
    }
    // The right rows by key, in row order; the keys that repeat on each side.
    let left_rows = left.table.rows();
    let right_rows = right.table.rows();
    let mut right_index: HashMap<Vec<MergeKey>, Vec<usize>> = HashMap::new();
    for r in 0..right_rows {
        if let Some(keys) = keys_of(&right_keys, r) {
            right_index.entry(keys).or_default().push(r);
        }
    }
    let mut left_count: HashMap<Vec<MergeKey>, usize> = HashMap::new();
    for l in 0..left_rows {
        if let Some(keys) = keys_of(&left_keys, l) {
            *left_count.entry(keys).or_default() += 1;
        }
    }
    let left_repeats = left_count.values().filter(|&&n| n > 1).count();
    let right_repeats = right_index.values().filter(|v| v.len() > 1).count();
    let both_repeat = left_count
        .iter()
        .filter(|(_, n)| **n > 1)
        .any(|(k, _)| right_index.get(k).is_some_and(|v| v.len() > 1));
    let on_text = on.join(", ");
    let on_call = if on.len() == 1 {
        on_text.clone()
    } else {
        format!("[{on_text}]")
    };
    // `wages.duplicates(firm)` for a bound frame, `duplicates(firm)` on an unnamed one.
    let duplicates_of = |shown: &str| {
        if shown.starts_with("the frame") {
            format!("`duplicates({on_call})`")
        } else {
            format!("`{shown}.duplicates({on_call})`")
        }
    };
    let repeated_where = |side: &str, shown: &str| {
        format!(
            "find the repeated keys with {} on the {side} frame and remove them",
            duplicates_of(shown)
        )
    };
    match relation.as_deref() {
        Some("one_to_one") if left_repeats > 0 || right_repeats > 0 => {
            let (side, shown, n) = if left_repeats > 0 {
                ("left", left_shown.as_str(), left_repeats)
            } else {
                ("right", right_shown.as_str(), right_repeats)
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0603",
                    format!(
                        "`relation: one_to_one` does not hold: {} on the {side}",
                        repeats(n, &on_text)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(4), "a relation the data do not have")
                .help(repeated_where(side, shown)),
            ));
        }
        Some("many_to_one") if right_repeats > 0 => {
            return Err(fail(
                Diagnostic::error(
                    "Z0603",
                    format!(
                        "`relation: many_to_one` does not hold: {} on the right",
                        repeats(right_repeats, &on_text)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(4), "a relation the data do not have")
                .help(repeated_where("right", &right_shown)),
            ));
        }
        Some("one_to_many") if left_repeats > 0 => {
            return Err(fail(
                Diagnostic::error(
                    "Z0603",
                    format!(
                        "`relation: one_to_many` does not hold: {} on the left",
                        repeats(left_repeats, &on_text)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(4), "a relation the data do not have")
                .help(repeated_where("left", &left_shown)),
            ));
        }
        None if both_repeat => {
            return Err(fail(
                Diagnostic::error("Z0603", "many-to-many merge refused")
                    .in_file(ctx.script)
                    .primary(
                        ctx.callee_span,
                        format!(
                            "{on_text} repeats in both frames ({} on the left, {} on the right)",
                            keys(left_repeats),
                            keys(right_repeats)
                        ),
                    )
                    .help(format!(
                        "find the repeated keys with {} on the right frame and remove them, then state `relation: many_to_one`",
                        duplicates_of(&right_shown)
                    )),
            ));
        }
        _ => {}
    }
    // The pairs, in the left order and the right order within one left row.
    let mut pairs: Vec<(Option<usize>, Option<usize>)> = Vec::with_capacity(left_rows);
    let mut right_matched = vec![false; right_rows];
    let mut matched = 0usize;
    let mut left_unmatched = 0usize;
    let mut multiplied = false;
    for l in 0..left_rows {
        let found = keys_of(&left_keys, l).and_then(|k| right_index.get(&k));
        match found {
            Some(rs) => {
                if rs.len() > 1 {
                    multiplied = true;
                }
                for &r in rs {
                    pairs.push((Some(l), Some(r)));
                    right_matched[r] = true;
                    matched += 1;
                }
            }
            None => {
                left_unmatched += 1;
                if join != "inner" {
                    pairs.push((Some(l), None));
                }
            }
        }
    }
    let right_unmatched = right_matched.iter().filter(|m| !**m).count();
    let appended = join == "full" && right_unmatched > 0;
    if appended {
        for (r, m) in right_matched.iter().enumerate() {
            if !m {
                pairs.push((None, Some(r)));
            }
        }
    }
    let left_idx: Vec<Option<usize>> = pairs.iter().map(|p| p.0).collect();
    let right_idx: Vec<Option<usize>> = pairs.iter().map(|p| p.1).collect();
    let mut columns: Vec<(String, Column)> = Vec::new();
    for (name, column) in left.table.columns() {
        let out_name = match &suffixes {
            Some((a, _)) if shared.contains(name) => format!("{name}{a}"),
            _ => name.clone(),
        };
        // A key column of a full join: the right's value where the left has none.
        let taken = if on.contains(name) && appended {
            let r_col = right.table.column(name).expect("a key of the right");
            fill_key(column, &left_idx, r_col, &right_idx)
        } else {
            column.take_optional(&left_idx)
        };
        columns.push((out_name, taken));
    }
    for (name, column) in right.table.columns() {
        if on.contains(name) {
            continue;
        }
        let out_name = match &suffixes {
            Some((_, b)) if shared.contains(name) => format!("{name}{b}"),
            _ => name.clone(),
        };
        columns.push((out_name, column.take_optional(&right_idx)));
    }
    ctx.message(Diagnostic::plain_note(format!(
        "merge: {} matched; {} unmatched; {} unmatched",
        rows(matched),
        left_rows_text(left_unmatched),
        right_rows_text(right_unmatched)
    )));
    let table = Table::new(columns).map_err(|e| internal(ctx, e))?;
    let mut metadata = left.table.metadata().clone();
    if let Some((unit, time)) = metadata.panel.clone()
        && (multiplied || appended)
    {
        ctx.message(Diagnostic::plain_note(format!(
            "the panel declaration `{unit} × {time}` dropped: `merge` {} rows",
            if multiplied { "repeated" } else { "added" }
        )));
        metadata.panel = None;
    }
    Ok(derived(table.with_metadata(metadata)))
}

fn repeats(n: usize, on: &str) -> String {
    format!(
        "{} of `{on}` {}",
        keys(n),
        if n == 1 { "repeats" } else { "repeat" }
    )
}

fn keys(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "key" } else { "keys" }
    )
}

fn left_rows_text(n: usize) -> String {
    format!(
        "{} left {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

fn right_rows_text(n: usize) -> String {
    format!(
        "{} right {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// A key column of a full join: the left's value where the left row exists, the right's for an
/// appended right row; a category keeps the left's levels, with any level of the right's that
/// the left lacks appended; the left's metadata is kept.
fn fill_key(
    left: &Column,
    left_idx: &[Option<usize>],
    right: &Column,
    right_idx: &[Option<usize>],
) -> Column {
    let n = left_idx.len();
    let column = if left.ty() == ColumnType::Category {
        let mut levels: Vec<String> = left
            .levels()
            .unwrap_or_default()
            .iter()
            .map(|l| l.to_string())
            .collect();
        let mut codes: Vec<Option<i32>> = Vec::with_capacity(n);
        for (l, r) in left_idx.iter().zip(right_idx) {
            let label = match (l, r) {
                (Some(l), _) => left.get(*l),
                (None, Some(r)) => right.get(*r),
                (None, None) => None,
            };
            codes.push(match label {
                Some(Cell::Category(label)) => Some(match levels.iter().position(|x| x == label) {
                    Some(p) => p as i32,
                    None => {
                        levels.push(label.to_string());
                        (levels.len() - 1) as i32
                    }
                }),
                _ => None,
            });
        }
        Column::categories(levels, codes)
            .unwrap_or_else(|_| Column::missing(ColumnType::Category, n))
    } else {
        let values: Vec<Value> = left_idx
            .iter()
            .zip(right_idx)
            .map(|(l, r)| match (l, r) {
                (Some(l), _) => to_value(left.get(*l)),
                (None, Some(r)) => to_value(right.get(*r)),
                (None, None) => Value::Na,
            })
            .collect();
        zeff_core::lift::collect_column(&values).unwrap_or_else(|| Column::missing(left.ty(), n))
    };
    column.with_metadata(left.metadata().clone())
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
