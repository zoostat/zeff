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

/// The program functions of `core` in this release, `print` apart.
pub fn functions() -> Vec<Function> {
    vec![
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
