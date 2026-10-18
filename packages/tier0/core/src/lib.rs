//! Package core (tier 0): the language and the types every package shares.

pub mod display;
pub mod operators;
pub mod runtime;
pub mod value;

use runtime::{Args, CallContext, Fail, Function};
use value::Value;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// `print(x, ...)`: each value through the display interface.
fn print(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    for value in args.values.into_iter().flatten().chain(args.rest) {
        let text = display::show(&value);
        ctx.print(&text);
    }
    Ok(Value::Nothing)
}

/// The functions of `core` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "print",
            package: "core",
            method: false,
            params: vec![ParamSpec::required("x", "Any")],
            rest: Some(Rest::Values("Any")),
            returns: "Nothing",
            diagnostics: vec![DiagnosticDecl::new(
                "Z0206",
                "the result of print is bound or passed; it has none",
            )],
            help: include_str!("../help/print.md"),
            accuracy: None,
        },
        body: print,
    }]
}
