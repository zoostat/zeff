//! The program functions of `core` that a script uses to speak for itself (Language Reference,
//! section 5.4): `warn(message)` prints a warning under the statement's output and carries on;
//! `error(message)` ends the script with Z0001. `print` is in `lib.rs`.

use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The text of a `message` argument; `NA` reads as the word.
fn message(args: &Args) -> String {
    match args.get(0) {
        Some(Value::Str(s)) => s.clone(),
        _ => "NA".to_string(),
    }
}

fn warn(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    ctx.message(Diagnostic::plain_warning(message(&args)));
    Ok(Value::Nothing)
}

fn error(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error("Z0001", message(&args))
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the script stops here"),
    ))
}

/// `seed(n)`: the run's seed, set once per script before any draw (Engine and Project
/// Specification, section 8.3, point 3; the catalogue's row 63). A second `seed`, or one after a
/// statement has drawn, is Z0111: the draws before it would not be reproduced by the seed printed.
fn seed(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let n = match args.get(0) {
        Some(Value::Int(n)) => *n,
        _ => {
            return Err(fail(
                Diagnostic::error("Z0401", "`n` of `seed` must be an Int, but this is NA")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(0), "not a seed"),
            ));
        }
    };
    if ctx.random.used {
        let chosen = ctx.random.seed.unwrap_or_default();
        return Err(fail(
            Diagnostic::error(
                "Z0111",
                "`seed` comes after a statement that has already drawn random numbers",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "too late to set the seed")
            .help(format!(
                "put `seed({n})` at the top of the script, before the first draw; the draws so far used the seed {chosen}"
            )),
        ));
    }
    if ctx.random.seed.is_some() {
        return Err(fail(
            Diagnostic::error("Z0111", "`seed` is set once per script, and it is set already")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a second seed")
                .help("one seed makes the whole run repeatable; a `seed:` given to one `draw` has a stream of its own"),
        ));
    }
    ctx.random.seed = Some(n);
    ctx.random.chosen = false;
    Ok(Value::Nothing)
}

/// The program functions of `core` in this release, `print` apart.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "seed",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("n", "Int")],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0111", "a second seed, or one after a draw"),
                    DiagnosticDecl::new(
                        "Z0206",
                        "the result of seed is bound or passed; it has none",
                    ),
                ],
                help: include_str!("../help/seed.md"),
                accuracy: None,
            },
            body: seed,
        },
        Function {
            spec: FunctionSpec {
                name: "warn",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("message", "Str")],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::note("the script's own warning, printed with its line"),
                    DiagnosticDecl::new(
                        "Z0206",
                        "the result of warn is bound or passed; it has none",
                    ),
                ],
                help: include_str!("../help/warn.md"),
                accuracy: None,
            },
            body: warn,
        },
        Function {
            spec: FunctionSpec {
                name: "error",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("message", "Str")],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0001",
                    "the script stopped itself with the message given",
                )],
                help: include_str!("../help/error.md"),
                accuracy: None,
            },
            body: error,
        },
    ]
}
