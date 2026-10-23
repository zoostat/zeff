//! `label` and `recode` (Language Reference, sections 4.2 and 4.3 rule 3; release 0.4.0, step 6):
//! `label` sets a variable's label or its value labels, which frames, tables and `describe` then
//! print; `recode` maps the values of a variable, merges the levels of a category and moves the
//! codes of a labelled Int with their labels, by the rules of question 6 of the release.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use zeff_columns::{Column, ColumnType, MetadataKey, format_value_labels};
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

/// The frame and the variable a method was called on: the frame, the variable's name and its
/// column; a name the frame does not have is Z0101.
fn frame_and_var(
    ctx: &CallContext<'_>,
    method: &str,
    args: &Args,
) -> Result<(Frame, String, Column), Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(
            ctx,
            format!("{method} was called without a frame"),
        ));
    };
    let Some(Value::Var(name)) = args.get(1) else {
        return Err(internal(
            ctx,
            format!("{method} was called without a variable name"),
        ));
    };
    let shown = frame.shown_name();
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let Some(column) = frame.table.column(name) else {
        return Err(fail(no_variable(
            ctx.script,
            ctx.span_of(1),
            &shown,
            name,
            &columns,
        )));
    };
    Ok((frame.clone(), name.clone(), column.clone()))
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

fn derived(table: zeff_columns::Table) -> Value {
    Value::Frame(Frame {
        table: Rc::new(table),
        name: None,
    })
}

/// `an Int`, `a Float`, `a Category`.
fn article(type_name: &str) -> String {
    if type_name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        format!("an {type_name}")
    } else {
        format!("a {type_name}")
    }
}

/// A key or value as a message writes it: text in quotes, numbers as they print, `NA`.
fn shown(value: &Value) -> String {
    match value {
        Value::Str(s) => format!("\"{s}\""),
        Value::Int(i) => i.to_string(),
        Value::Float(x) => format_float(*x),
        Value::Bool(b) => b.to_string(),
        Value::Na => "NA".to_string(),
        other => other.type_word(),
    }
}

/// The frame with `column` in place of the variable `name`.
fn replaced(
    ctx: &CallContext<'_>,
    frame: &Frame,
    name: &str,
    column: Column,
) -> Result<Value, Fail> {
    let table = frame
        .table
        .with_column(name, column)
        .map_err(|e| internal(ctx, format!("{e:?}")))?;
    Ok(derived(table))
}

// ----- label -----

/// `data.label(var, labels)`: `labels` as text sets the variable label, an empty text removing it;
/// as a mapping of code to text it sets the value labels of an Int variable, stored in code order
/// and replacing any it had.
pub fn label(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (frame, name, column) = frame_and_var(ctx, "label", &args)?;
    let labelled = match args.get(2) {
        Some(Value::Str(text)) => {
            let mut store = column.metadata().clone();
            if text.trim().is_empty() {
                store.remove(MetadataKey::Label);
            } else {
                store.set(MetadataKey::Label, text.trim());
            }
            column.with_metadata(store)
        }
        Some(Value::Mapping(pairs)) => {
            if column.ty() != ColumnType::Int {
                let help = match column.ty() {
                    ColumnType::Category => format!(
                        "a category's levels are its labels; `recode({name}, [\"old\" => \"new\"])` renames them"
                    ),
                    ColumnType::Str => format!(
                        "`category({name})` makes a category whose levels are the texts; `to({name}, Int)` gives codes to label"
                    ),
                    _ => format!(
                        "`to({name}, Int)` gives whole numbers to label when the values are whole"
                    ),
                };
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "value labels name the codes of an Int variable, but `{name}` is {} variable",
                            article(column.ty().name())
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(1), "not an Int variable")
                    .help(help),
                ));
            }
            let mut codes: Vec<(i64, String)> = Vec::with_capacity(pairs.len());
            for (key, value) in pairs {
                let (Value::Int(code), Value::Str(text)) = (key, value) else {
                    let (what, found) = if matches!(key, Value::Int(_)) {
                        ("a value label is text", value)
                    } else {
                        ("a code is a whole number", key)
                    };
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!("{what}, but the mapping holds {}", shown(found)),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(2), "the labels")
                        .help("write `[0 => \"male\", 1 => \"female\"]`: each code once, its label in quotes"),
                    ));
                };
                codes.push((*code, text.clone()));
            }
            codes.sort_by_key(|(code, _)| *code);
            let pairs: Vec<(String, String)> = codes
                .into_iter()
                .map(|(code, text)| (code.to_string(), text))
                .collect();
            column.with_entry(MetadataKey::ValueLabels, format_value_labels(&pairs))
        }
        other => {
            return Err(internal(
                ctx,
                format!(
                    "label was given {:?} as its labels",
                    other.map(Value::type_name)
                ),
            ));
        }
    };
    replaced(ctx, &frame, &name, labelled)
}

// ----- recode -----

/// What a recoding did: the new column, the rows whose value changed, the keys that matched
/// nothing (as messages write them), the level count of a category, and whether value labels
/// moved.
struct Recoded {
    column: Column,
    changed: usize,
    unknown: Vec<String>,
    levels: Option<String>,
    labels_followed: bool,
}

/// `data.recode(var, mapping)`: the values, levels or labelled codes of a variable mapped from
/// old to new by identity, the type and metadata kept, with a note of what changed and W0206 for
/// a key that matched nothing.
pub fn recode(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (frame, name, column) = frame_and_var(ctx, "recode", &args)?;
    let Some(Value::Mapping(pairs)) = args.get(2) else {
        return Err(internal(ctx, "recode was called without a mapping"));
    };
    let outcome = match column.ty() {
        ColumnType::Category => recode_category(ctx, &name, &column, pairs)?,
        ColumnType::Int => recode_ints(ctx, &name, &column, pairs)?,
        ColumnType::Float => recode_floats(ctx, &name, &column, pairs)?,
        ColumnType::Str => recode_strs(ctx, &name, &column, pairs)?,
        ColumnType::Bool => recode_bools(ctx, &name, &column, pairs)?,
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0304",
                    format!(
                        "`recode` on a {} variable arrives at 0.6.0 with the `time` package",
                        other.name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "a date or time variable")
                .help("`to(x, Str)` gives text to recode until then"),
            ));
        }
    };
    if !outcome.unknown.is_empty() {
        let what = if column.ty() == ColumnType::Category {
            "level"
        } else {
            "value"
        };
        let message = if outcome.unknown.len() == 1 {
            format!(
                "`recode`: {} is not a {what} of `{name}`",
                outcome.unknown[0]
            )
        } else {
            format!(
                "`recode`: {} keys are none of the {what}s of `{name}`: {}",
                count(outcome.unknown.len() as u64),
                outcome.unknown.join(", ")
            )
        };
        let help = if column.ty() == ColumnType::Category {
            format!("`levels({name})` lists the levels; the other keys were applied")
        } else {
            "check the spelling or the type of the key; the other keys were applied".to_string()
        };
        ctx.message(Diagnostic::warning("W0206", message).help(help));
    }
    let rows = match outcome.changed {
        0 => "no row changed".to_string(),
        1 => "1 row changed".to_string(),
        n => format!("{} rows changed", count(n as u64)),
    };
    let mut note = format!("`{name}` recoded: {rows}");
    if let Some(levels) = &outcome.levels {
        note.push_str(&format!("; {levels}"));
    }
    if outcome.labels_followed {
        note.push_str("; the value labels follow their codes");
    }
    ctx.message(Diagnostic::plain_note(note));
    replaced(ctx, &frame, &name, outcome.column)
}

/// Z0401 for a key or value of another kind than the variable's.
fn wrong_kind(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    what: &str,
    found: &Value,
) -> Fail {
    let ty = column.ty();
    let help = if ty == ColumnType::Category {
        format!(
            "the keys are levels and the values new names, both in quotes, or `NA`; `levels({name})` lists the levels"
        )
    } else if ty == ColumnType::Int
        && matches!(found, Value::Str(_))
        && column.metadata().value_labels().is_some()
    {
        format!(
            "write the code a label stands for, `[0 => 2]`; `category({name})` makes a category whose levels are the labels, which `recode` merges by name"
        )
    } else {
        let kind = match ty {
            ColumnType::Int => "whole numbers",
            ColumnType::Float => "numbers",
            ColumnType::Str => "texts in quotes",
            _ => "`true` or `false`",
        };
        format!(
            "`recode` keeps a variable's type, so the keys and values are {kind} or `NA`; `add({name} = ...)` makes a variable of another type"
        )
    };
    fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{name}` is {} variable, but {what} of the mapping is {}",
                article(ty.name()),
                found.type_word()
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(2), "a key or value of another kind")
        .help(help),
    )
}

/// The value labels of an Int column with every labelled code mapped, a code merged into another
/// keeping the label of the first of its old codes in code order; `None` when the column has no
/// value labels. The second value says whether the labels changed.
fn labels_following(
    column: &Column,
    map: &HashMap<Option<i64>, Option<i64>>,
) -> Option<(Option<String>, bool)> {
    let labels = column.metadata().value_labels()?;
    let mut by_code: Vec<(i64, String)> = Vec::new();
    let mut others: Vec<(String, String)> = Vec::new();
    for (code, text) in &labels {
        match code.parse::<i64>() {
            Ok(c) => by_code.push((c, text.clone())),
            Err(_) => others.push((code.clone(), text.clone())),
        }
    }
    by_code.sort_by_key(|(code, _)| *code);
    let mut new_labels: Vec<(i64, String)> = Vec::new();
    for (code, text) in &by_code {
        let target = map.get(&Some(*code)).copied().unwrap_or(Some(*code));
        if let Some(target) = target
            && !new_labels.iter().any(|(c, _)| *c == target)
        {
            new_labels.push((target, text.clone()));
        }
    }
    new_labels.sort_by_key(|(code, _)| *code);
    let mut pairs: Vec<(String, String)> = new_labels
        .into_iter()
        .map(|(code, text)| (code.to_string(), text))
        .collect();
    pairs.extend(others);
    let entry = if pairs.is_empty() {
        None
    } else {
        Some(format_value_labels(&pairs))
    };
    let changed = entry.as_deref() != column.metadata().get(MetadataKey::ValueLabels);
    Some((entry, changed))
}

fn recode_ints(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    pairs: &[(Value, Value)],
) -> Result<Recoded, Fail> {
    let mut map: HashMap<Option<i64>, Option<i64>> = HashMap::new();
    let mut keys: Vec<Option<i64>> = Vec::new();
    for (key, value) in pairs {
        let k = match key {
            Value::Int(i) => Some(*i),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a key", other)),
        };
        let v = match value {
            Value::Int(i) => Some(*i),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a value", other)),
        };
        map.insert(k, v);
        keys.push(k);
    }
    let mut seen: HashSet<Option<i64>> = HashSet::new();
    let mut changed = 0;
    let new: Vec<Option<i64>> = column
        .as_ints()
        .expect("an Int column")
        .map(|old| match map.get(&old) {
            Some(new) => {
                seen.insert(old);
                if *new != old {
                    changed += 1;
                }
                *new
            }
            None => old,
        })
        .collect();
    let unknown = keys
        .iter()
        .filter(|k| !seen.contains(k))
        .map(|k| k.map_or("NA".to_string(), |i| i.to_string()))
        .collect();
    let mut store = column.metadata().clone();
    let mut labels_followed = false;
    if let Some((entry, followed)) = labels_following(column, &map) {
        match entry {
            Some(text) => store.set(MetadataKey::ValueLabels, text),
            None => {
                store.remove(MetadataKey::ValueLabels);
            }
        }
        labels_followed = followed;
    }
    Ok(Recoded {
        column: Column::ints(new).with_metadata(store),
        changed,
        unknown,
        levels: None,
        labels_followed,
    })
}

/// A Float as identity sees it: `-0.0` is `0.0`.
fn identity_bits(x: f64) -> u64 {
    if x == 0.0 {
        0.0f64.to_bits()
    } else {
        x.to_bits()
    }
}

fn recode_floats(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    pairs: &[(Value, Value)],
) -> Result<Recoded, Fail> {
    let mut map: HashMap<Option<u64>, Option<f64>> = HashMap::new();
    let mut keys: Vec<(Option<u64>, String)> = Vec::new();
    for (key, value) in pairs {
        let (k, text) = match key {
            Value::Int(i) => (Some(identity_bits(*i as f64)), i.to_string()),
            Value::Float(x) => (Some(identity_bits(*x)), format_float(*x)),
            Value::Na => (None, "NA".to_string()),
            other => return Err(wrong_kind(ctx, name, column, "a key", other)),
        };
        let v = match value {
            Value::Int(i) => Some(*i as f64),
            Value::Float(x) => Some(*x),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a value", other)),
        };
        map.insert(k, v);
        keys.push((k, text));
    }
    let mut seen: HashSet<Option<u64>> = HashSet::new();
    let mut changed = 0;
    let new: Vec<Option<f64>> = column
        .as_floats()
        .expect("a Float column")
        .map(|old| {
            let bits = old.map(identity_bits);
            match map.get(&bits) {
                Some(new) => {
                    seen.insert(bits);
                    if new.map(identity_bits) != bits {
                        changed += 1;
                    }
                    *new
                }
                None => old,
            }
        })
        .collect();
    let unknown = keys
        .iter()
        .filter(|(k, _)| !seen.contains(k))
        .map(|(_, text)| text.clone())
        .collect();
    Ok(Recoded {
        column: Column::floats(new)
            .0
            .with_metadata(column.metadata().clone()),
        changed,
        unknown,
        levels: None,
        labels_followed: false,
    })
}

fn recode_strs(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    pairs: &[(Value, Value)],
) -> Result<Recoded, Fail> {
    let mut map: HashMap<Option<String>, Option<String>> = HashMap::new();
    let mut keys: Vec<Option<String>> = Vec::new();
    for (key, value) in pairs {
        let k = match key {
            Value::Str(s) => Some(s.clone()),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a key", other)),
        };
        let v = match value {
            Value::Str(s) => Some(s.clone()),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a value", other)),
        };
        map.insert(k.clone(), v);
        keys.push(k);
    }
    let mut seen: HashSet<Option<String>> = HashSet::new();
    let mut changed = 0;
    let new: Vec<Option<String>> = column
        .as_strs()
        .expect("a Str column")
        .map(|old| {
            let old: Option<String> = old.map(str::to_string);
            match map.get(&old) {
                Some(new) => {
                    seen.insert(old.clone());
                    if *new != old {
                        changed += 1;
                    }
                    new.clone()
                }
                None => old,
            }
        })
        .collect();
    let unknown = keys
        .iter()
        .filter(|k| !seen.contains(k))
        .map(|k| k.as_ref().map_or("NA".to_string(), |s| format!("\"{s}\"")))
        .collect();
    Ok(Recoded {
        column: Column::strs(new.iter().map(Option::as_deref))
            .with_metadata(column.metadata().clone()),
        changed,
        unknown,
        levels: None,
        labels_followed: false,
    })
}

fn recode_bools(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    pairs: &[(Value, Value)],
) -> Result<Recoded, Fail> {
    let mut map: HashMap<Option<bool>, Option<bool>> = HashMap::new();
    let mut keys: Vec<Option<bool>> = Vec::new();
    for (key, value) in pairs {
        let k = match key {
            Value::Bool(b) => Some(*b),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a key", other)),
        };
        let v = match value {
            Value::Bool(b) => Some(*b),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a value", other)),
        };
        map.insert(k, v);
        keys.push(k);
    }
    let mut seen: HashSet<Option<bool>> = HashSet::new();
    let mut changed = 0;
    let new: Vec<Option<bool>> = column
        .as_bools()
        .expect("a Bool column")
        .map(|old| match map.get(&old) {
            Some(new) => {
                seen.insert(old);
                if *new != old {
                    changed += 1;
                }
                *new
            }
            None => old,
        })
        .collect();
    let unknown = keys
        .iter()
        .filter(|k| !seen.contains(k))
        .map(|k| k.map_or("NA".to_string(), |b| b.to_string()))
        .collect();
    Ok(Recoded {
        column: Column::bools(new).with_metadata(column.metadata().clone()),
        changed,
        unknown,
        levels: None,
        labels_followed: false,
    })
}

/// The levels merged, renamed or removed: a merged level stands where the first of its old levels
/// stood and the others vanish; a new name is a new level in that place; `NA` as a value removes
/// the level and makes its rows `NA`; `NA` as a key gives the missing rows a level, appended when
/// it is new, and adds nothing when no row is missing. The order, the contrast and the metadata
/// are kept.
fn recode_category(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    pairs: &[(Value, Value)],
) -> Result<Recoded, Fail> {
    let levels: Vec<String> = column
        .levels()
        .expect("a category")
        .iter()
        .map(|l| (*l).to_string())
        .collect();
    let mut map: HashMap<Option<String>, Option<String>> = HashMap::new();
    let mut keys: Vec<Option<String>> = Vec::new();
    for (key, value) in pairs {
        let k = match key {
            Value::Str(s) => Some(s.clone()),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a key", other)),
        };
        let v = match value {
            Value::Str(s) => Some(s.clone()),
            Value::Na => None,
            other => return Err(wrong_kind(ctx, name, column, "a value", other)),
        };
        map.insert(k.clone(), v);
        keys.push(k);
    }
    let old_codes: Vec<Option<i32>> = column.as_codes().expect("a category").collect();
    let has_na = old_codes.iter().any(Option::is_none);
    let unknown: Vec<String> = keys
        .iter()
        .filter(|k| match k {
            Some(level) => !levels.contains(level),
            None => !has_na,
        })
        .map(|k| k.as_ref().map_or("NA".to_string(), |s| format!("\"{s}\"")))
        .collect();
    // The new levels in order, and where each old level and the missing rows go.
    let mut new_levels: Vec<String> = Vec::new();
    let mut place = |level: String| -> usize {
        match new_levels.iter().position(|l| *l == level) {
            Some(p) => p,
            None => {
                new_levels.push(level);
                new_levels.len() - 1
            }
        }
    };
    let new_of_old: Vec<Option<usize>> = levels
        .iter()
        .map(|level| {
            let target = match map.get(&Some(level.clone())) {
                Some(target) => target.clone(),
                None => Some(level.clone()),
            };
            target.map(&mut place)
        })
        .collect();
    // The missing rows' level, when there are missing rows: a key that matches nothing changes
    // nothing, so no level is added for it.
    let na_target: Option<usize> = match map.get(&None) {
        Some(Some(level)) if has_na => Some(place(level.clone())),
        _ => None,
    };
    let mut changed = 0;
    let codes: Vec<Option<i32>> = old_codes
        .iter()
        .map(|code| {
            let new = match code {
                Some(c) => new_of_old[*c as usize],
                None => na_target,
            };
            let old_label = code.map(|c| levels[c as usize].as_str());
            let new_label = new.map(|p| new_levels[p].as_str());
            if old_label != new_label {
                changed += 1;
            }
            new.map(|p| p as i32)
        })
        .collect();
    let count_now = new_levels.len();
    let levels_note = if count_now == levels.len() {
        format!(
            "{} {}",
            count(count_now as u64),
            if count_now == 1 { "level" } else { "levels" }
        )
    } else {
        format!(
            "{} {}, from {}",
            count(count_now as u64),
            if count_now == 1 { "level" } else { "levels" },
            count(levels.len() as u64)
        )
    };
    let recoded = Column::categories(new_levels, codes)
        .map_err(|e| internal(ctx, format!("{e:?}")))?
        .with_category_info(column.category_info().expect("a category"))
        .with_metadata(column.metadata().clone());
    Ok(Recoded {
        column: recoded,
        changed,
        unknown,
        levels: Some(levels_note),
        labels_followed: false,
    })
}
