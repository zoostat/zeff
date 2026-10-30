//! The collection functions of `core` (Language Reference, sections 3.7 and 4; the catalogue's
//! Collections group). `vars(name, ..., x1..x50)` (row 39) names variables outside a frame
//! method, where a bare list of names has no frame to be looked up in (section 6, "Names inside
//! a frame's methods"); a range among them stays unexpanded until the list meets a frame
//! (section 3.7, "Variable ranges"). `len`, `first`, `last`, `append` and `reversed` (rows 43,
//! 44, 45, 51, 48; release 0.5.0, step 11) work on lists as values: a list is never changed,
//! `append` gives a new one, and `len` is the one function that skips failed items with a count
//! (section 3.7, "Failed values").

use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
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

pub fn functions() -> Vec<Function> {
    vec![
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
