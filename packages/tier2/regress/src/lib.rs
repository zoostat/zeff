//! Package regress (tier 2): linear regression (Engine and Project Specification, section 13;
//! release 0.4.0). At this step it registers `interact`, the formula term of section 7.2, so that
//! the checker admits it inside a formula; `reg` arrives with step 10.

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// `interact([a, b, c], order: 2)` is read by the model matrix inside a formula (section 7.2,
/// "Expansion"); called anywhere else it has no value to give.
pub fn interact(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error(
            "Z0401",
            "`interact` is a term of a formula and has no value outside one",
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "called outside a formula")
        .help("write it on the right of `~`: `y ~ interact([a, b, c], order: 2)`"),
    ))
}

/// The functions of `regress` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "interact",
            package: "regress",
            method: false,
            params: vec![
                ParamSpec::required("vars", "List"),
                ParamSpec::option_with_default("order", "Int", "2"),
            ],
            rest: None,
            returns: "Column",
            diagnostics: vec![DiagnosticDecl::new(
                "Z0401",
                "called outside a formula, or with an order below 2 or above the number of variables",
            )],
            help: include_str!("../help/interact.md"),
            accuracy: None,
        },
        body: interact,
    }]
}
