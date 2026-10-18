//! The missing-value functions of `core` (Language Reference, section 4.1): `is_missing` and
//! `coalesce`, written for one row each and lifted to columns by `lift`. `x == NA` is always `NA`,
//! so `is_missing` is the test; `coalesce` takes the first value that is not `NA`, its arguments
//! joined to one type by the checker (section 3.7, "Joining types").

use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{FunctionSpec, ParamSpec, Rest};

/// The computable scalar types a value may have.
const SCALAR: &str = "Bool | Int | Float | Str";

fn is_missing(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(Value::Bool(matches!(args.get(0), None | Some(Value::Na))))
}

/// The kind of a value for the joining rule: numbers join, everything else only with itself.
fn kind(value: &Value) -> Option<&'static str> {
    match value {
        Value::Int(_) | Value::Float(_) => Some("number"),
        Value::Bool(_) => Some("Bool"),
        Value::Str(_) => Some("Str"),
        _ => None,
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "an Int",
        Value::Float(_) => "a Float",
        Value::Bool(_) => "a Bool",
        Value::Str(_) => "a Str",
        _ => "an NA",
    }
}

/// The first argument that is not `NA`; an Int becomes a Float when any argument is a Float, so
/// that the value has the type the checker joined. A column's type is not known before the run,
/// so the joining rule is checked here again (Z0214).
fn coalesce(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let values: Vec<&Value> = args
        .values
        .iter()
        .flatten()
        .chain(args.rest.iter())
        .collect();
    let mut first_kind: Option<(&'static str, &Value)> = None;
    for value in &values {
        if let Some(k) = kind(value) {
            match first_kind {
                None => first_kind = Some((k, value)),
                Some((seen, seen_value)) if seen != k => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0214",
                            format!(
                                "`coalesce` needs values of one type, but these are {} and {}",
                                type_name(seen_value),
                                type_name(value)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.callee_span, "values of two types")
                        .help("Int and Float join as Float, and NA fits any type; convert with `to` where a mixture is meant"),
                    ));
                }
                _ => {}
            }
        }
    }
    let any_float = values.iter().any(|v| matches!(v, Value::Float(_)));
    let first = values.iter().find(|v| !matches!(v, Value::Na));
    Ok(match first {
        Some(Value::Int(i)) if any_float => Value::Float(*i as f64),
        Some(value) => (*value).clone(),
        None => Value::Na,
    })
}

/// The missing-value functions of `core` in this release.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "is_missing",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("x", SCALAR)],
                rest: None,
                returns: "Bool",
                diagnostics: vec![],
                help: include_str!("../help/is_missing.md"),
                accuracy: None,
            },
            body: is_missing,
        },
        Function {
            spec: FunctionSpec {
                name: "coalesce",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", SCALAR),
                    ParamSpec::required("b", SCALAR),
                ],
                rest: Some(Rest::Values(SCALAR)),
                returns: "Join",
                diagnostics: vec![zeff_registry::DiagnosticDecl::new(
                    "Z0214",
                    "the arguments are of types that do not join",
                )],
                help: include_str!("../help/coalesce.md"),
                accuracy: None,
            },
            body: coalesce,
        },
    ]
}
