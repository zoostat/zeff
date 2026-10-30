//! The collection functions of `core` (Language Reference, sections 3.7 and 4; the catalogue's
//! Collections group). `vars(name, ..., x1..x50)` (row 39) names variables outside a frame
//! method, where a bare list of names has no frame to be looked up in (section 6, "Names inside
//! a frame's methods"); a range among them stays unexpanded until the list meets a frame
//! (section 3.7, "Variable ranges"). `len`, `first`, `last`, `append` and `reversed` (rows 43,
//! 44, 45, 51, 48; release 0.5.0, step 11) work on lists as values: a list is never changed,
//! `append` gives a new one, and `len` is the one function that skips failed items with a count
//! (section 3.7, "Failed values"). `unique`, `sorted`, `is_in`, `keys` and `equal` (rows 46, 47,
//! 53, 52, 529; step 12) use the identity relation and the order rules of section 3.7: `NA` is
//! identical to `NA` and `-0.0` to `0.0`, `NA` sorts last either way, and `equal` is identity
//! over lists, records and mappings as well.

use crate::operators::identical;
use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
use std::cmp::Ordering;
use zeff_diagnostics::{Diagnostic, count};
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

fn vars(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let mut names: Vec<String> = Vec::new();
    let mut push = |value: &Value| match value {
        Value::Var(name) => names.push(name.clone()),
        Value::Vars(more) => names.extend(more.iter().cloned()),
        _ => {}
    };
    if let Some(first) = args.get(0) {
        push(first);
    }
    for value in &args.rest {
        push(value);
    }
    Ok(Value::Vars(names))
}

/// `len(x)`: the items of a list (failed ones skipped and counted, W0801), the Unicode scalar
/// values of a string (section 3.8, "Unit"), the rows of a column, the entries of a mapping.
fn len(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let n = match args.take(0) {
        Some(Value::List(items)) => {
            let failed: Vec<&crate::value::Failure> = items
                .iter()
                .filter_map(|v| match v {
                    Value::Failed(f) => Some(f),
                    _ => None,
                })
                .collect();
            if !failed.is_empty() {
                let mut causes: Vec<String> = Vec::new();
                for f in &failed {
                    let cause = format!("{} at {}", f.code, f.place());
                    if !causes.contains(&cause) {
                        causes.push(cause);
                    }
                }
                let shown = causes.len().min(3);
                let more = if causes.len() > 3 { ", …" } else { "" };
                let causes = causes[..shown].join(", ");
                ctx.message(
                    Diagnostic::warning(
                        "W0801",
                        format!(
                            "{} of {} items failed ({causes}{more}) and {} skipped",
                            count(failed.len() as u64),
                            count(items.len() as u64),
                            if failed.len() == 1 { "was" } else { "were" }
                        ),
                    )
                    .help("the count is of the items that succeeded; `x.is(Failed)` tells which failed"),
                );
            }
            items.len() - failed.len()
        }
        Some(Value::Str(s)) => s.chars().count(),
        Some(Value::Column(c)) => c.len(),
        Some(Value::Mapping(entries)) => entries.len(),
        Some(Value::Na) => return Ok(Value::Na),
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z9001",
                    format!(
                        "internal error: this is a bug in Zeff, not in your script (`len` of {} reached the body)",
                        other.map_or("nothing".to_string(), |v| crate::types::type_name(&v))
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "the call that failed"),
            ));
        }
    };
    Ok(Value::Int(n as i64))
}

/// Z0207 (section 5.6): `first([])` or `last([])`.
fn empty(ctx: &CallContext<'_>, name: &str) -> Fail {
    fail(
        Diagnostic::error("Z0207", format!("`{name}` of an empty list"))
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "the list has 0 items")
            .help(format!(
                "a list must have an item for `{name}` to give one; test `len(xs) > 0` first, or build the list so that it is never empty"
            )),
    )
}

fn list_arg(ctx: &CallContext<'_>, args: &mut Args, name: &str) -> Result<Vec<Value>, Fail> {
    match args.take(0) {
        Some(Value::List(items)) => Ok(items),
        other => Err(fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script (`{name}` of {} reached the body)",
                    other.map_or("nothing".to_string(), |v| crate::types::type_name(&v))
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )),
    }
}

/// `first(list)`: the first item, in the list's current order (section 6, "Row order").
fn first(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let mut items = list_arg(ctx, &mut args, "first")?;
    if items.is_empty() {
        return Err(empty(ctx, "first"));
    }
    Ok(items.swap_remove(0))
}

/// `last(list)`: the last item.
fn last(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let mut items = list_arg(ctx, &mut args, "last")?;
    match items.pop() {
        Some(item) => Ok(item),
        None => Err(empty(ctx, "last")),
    }
}

/// `append(list, x)`: a new list with `x` after the items; the list given is not changed.
fn append(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let mut items = list_arg(ctx, &mut args, "append")?;
    let x = args.take(1).unwrap_or(Value::Na);
    items.push(x);
    Ok(Value::List(items))
}

/// `reversed(list)`: a new list with the items in the opposite order.
fn reversed(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let mut items = list_arg(ctx, &mut args, "reversed")?;
    items.reverse();
    Ok(Value::List(items))
}

/// `unique(list)`: the items with later occurrences of an identical item dropped, the first kept
/// in its place (section 6, "Row order").
fn unique(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let items = list_arg(ctx, &mut args, "unique")?;
    let mut kept: Vec<Value> = Vec::with_capacity(items.len());
    for item in items {
        if !kept.iter().any(|k| identical(k, &item)) {
            kept.push(item);
        }
    }
    Ok(Value::List(kept))
}

/// The order of section 3.7 ("Identity"): `false < true`, numbers numerically, text by code
/// point, and `NA` after everything; items of different kinds, which one list never holds, by
/// kind.
fn order(a: &Value, b: &Value) -> Ordering {
    fn kind(v: &Value) -> u8 {
        match v {
            Value::Bool(_) => 0,
            Value::Int(_) | Value::Float(_) => 1,
            Value::Str(_) => 2,
            Value::Na => 4,
            _ => 3,
        }
    }
    match (a, b) {
        (Value::Na, Value::Na) => Ordering::Equal,
        (Value::Na, _) => Ordering::Greater,
        (_, Value::Na) => Ordering::Less,
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => {
            if x == y {
                Ordering::Equal
            } else {
                x.total_cmp(y)
            }
        }
        (Value::Int(x), Value::Float(y)) => (*x as f64).partial_cmp(y).unwrap_or(Ordering::Less),
        (Value::Float(x), Value::Int(y)) => {
            x.partial_cmp(&(*y as f64)).unwrap_or(Ordering::Greater)
        }
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (x, y) => kind(x).cmp(&kind(y)),
    }
}

/// `sorted(list, desc:)`: a sorted copy under the order rules, `NA` last either way; the sort is
/// stable, so identical items keep their order (`-0.0` beside `0.0`).
fn sorted(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let mut items = list_arg(ctx, &mut args, "sorted")?;
    let descending = matches!(args.get(1), Some(Value::Bool(true)));
    items.sort_by(|a, b| match (a, b) {
        (Value::Na, _) | (_, Value::Na) => order(a, b),
        _ if descending => order(b, a),
        _ => order(a, b),
    });
    Ok(Value::List(items))
}

/// `is_in(x, list)`: whether `x` is identical to an item; row by row on a column, through the
/// lifting of section 3.7, where a category's row is its label.
fn is_in(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = args.get(0).cloned().unwrap_or(Value::Na);
    let found = match args.get(1) {
        Some(Value::List(items)) => items.iter().any(|item| identical(item, &x)),
        _ => false,
    };
    Ok(Value::Bool(found))
}

/// `keys(x)`: the field names of a record, or the keys of a mapping in insertion order.
fn keys(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    match args.take(0) {
        Some(Value::Record(fields)) => Ok(Value::List(
            fields.into_iter().map(|(n, _)| Value::Str(n)).collect(),
        )),
        Some(Value::Mapping(entries)) => {
            Ok(Value::List(entries.into_iter().map(|(k, _)| k).collect()))
        }
        other => Err(fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script (`keys` of {} reached the body)",
                    other.map_or("nothing".to_string(), |v| crate::types::type_name(&v))
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )),
    }
}

/// Identity over every value `equal` takes (section 3.7, "Identity"; catalogue row 529): scalars
/// by `identical`, lists item by item, records field by field whatever the order the fields are
/// written in, mappings entry by entry in insertion order; `false` across kinds.
pub fn deep_identical(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| deep_identical(p, q))
        }
        (Value::Record(x), Value::Record(y)) => {
            x.len() == y.len()
                && x.iter().all(|(name, value)| {
                    y.iter()
                        .find(|(n, _)| n == name)
                        .is_some_and(|(_, other)| deep_identical(value, other))
                })
        }
        (Value::Mapping(x), Value::Mapping(y)) => {
            x.len() == y.len()
                && x.iter()
                    .zip(y)
                    .all(|((k1, v1), (k2, v2))| deep_identical(k1, k2) && deep_identical(v1, v2))
        }
        (Value::List(_) | Value::Record(_) | Value::Mapping(_), _)
        | (_, Value::List(_) | Value::Record(_) | Value::Mapping(_)) => false,
        (x, y) => identical(x, y),
    }
}

/// `equal(a, b)`: identity over scalars, lists, records and mappings, `false` across types; the
/// way a project tests its own functions, `assert(equal(xs, [1, 2]), ...)`.
fn equal(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let a = args.get(0).cloned().unwrap_or(Value::Na);
    let b = args.get(1).cloned().unwrap_or(Value::Na);
    Ok(Value::Bool(deep_identical(&a, &b)))
}

/// The scalar types a list may hold, which `is_in` looks for; and the values `equal` compares.
const SCALARS: &str = "Bool | Int | Float | Str";
const COMPARABLE: &str = "Bool | Int | Float | Str | List | Record | Mapping";

pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "unique",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("list", "List")],
                rest: None,
                returns: "Same",
                diagnostics: vec![],
                help: include_str!("../help/unique.md"),
                accuracy: None,
            },
            body: unique,
        },
        Function {
            spec: FunctionSpec {
                name: "sorted",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("list", "List<Bool | Int | Float | Str>"),
                    ParamSpec::option_with_default("desc", "Bool", "false"),
                ],
                rest: None,
                returns: "Same",
                diagnostics: vec![],
                help: include_str!("../help/sorted.md"),
                accuracy: None,
            },
            body: sorted,
        },
        Function {
            spec: FunctionSpec {
                name: "is_in",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("x", SCALARS),
                    ParamSpec::required("list", "List"),
                ],
                rest: None,
                returns: "Bool",
                diagnostics: vec![],
                help: include_str!("../help/is_in.md"),
                accuracy: None,
            },
            body: is_in,
        },
        Function {
            spec: FunctionSpec {
                name: "keys",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("record", "Record | Mapping")],
                rest: None,
                returns: "Keys",
                diagnostics: vec![],
                help: include_str!("../help/keys.md"),
                accuracy: None,
            },
            body: keys,
        },
        Function {
            spec: FunctionSpec {
                name: "equal",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", COMPARABLE),
                    ParamSpec::required("b", COMPARABLE),
                ],
                rest: None,
                returns: "Bool",
                diagnostics: vec![],
                help: include_str!("../help/equal.md"),
                accuracy: None,
            },
            body: equal,
        },
        Function {
            spec: FunctionSpec {
                name: "vars",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("name", "Var | Vars")],
                rest: Some(Rest::Values("Var | Vars")),
                returns: "Vars",
                diagnostics: vec![],
                help: include_str!("../help/vars.md"),
                accuracy: None,
            },
            body: vars,
        },
        Function {
            spec: FunctionSpec {
                name: "len",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("x", "List | Str | Column | Mapping")],
                rest: None,
                returns: "Int",
                diagnostics: vec![DiagnosticDecl::new(
                    "W0801",
                    "the list holds failed values, which are skipped and counted",
                )],
                help: include_str!("../help/len.md"),
                accuracy: None,
            },
            body: len,
        },
        Function {
            spec: FunctionSpec {
                name: "first",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("list", "List")],
                rest: None,
                returns: "Element",
                diagnostics: vec![DiagnosticDecl::new("Z0207", "the list is empty")],
                help: include_str!("../help/first.md"),
                accuracy: None,
            },
            body: first,
        },
        Function {
            spec: FunctionSpec {
                name: "last",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("list", "List")],
                rest: None,
                returns: "Element",
                diagnostics: vec![DiagnosticDecl::new("Z0207", "the list is empty")],
                help: include_str!("../help/last.md"),
                accuracy: None,
            },
            body: last,
        },
        Function {
            spec: FunctionSpec {
                name: "append",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("list", "List"),
                    ParamSpec::required("x", "Any"),
                ],
                rest: None,
                returns: "Appended",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0214",
                    "x has a type that does not join with the list's items",
                )],
                help: include_str!("../help/append.md"),
                accuracy: None,
            },
            body: append,
        },
        Function {
            spec: FunctionSpec {
                name: "reversed",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("list", "List")],
                rest: None,
                returns: "Same",
                diagnostics: vec![],
                help: include_str!("../help/reversed.md"),
                accuracy: None,
            },
            body: reversed,
        },
    ]
}
