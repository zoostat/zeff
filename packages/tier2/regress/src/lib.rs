//! Package regress (tier 2): linear regression (Engine and Project Specification, section 13;
//! release 0.4.0): `reg`, ordinary least squares on the kernels with the variance layer of
//! `results`, and `interact`, the formula term of section 7.2.

pub mod ols;
pub mod reg;

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
    vec![
        Function {
            spec: FunctionSpec {
                name: "reg",
                package: "regress",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("formula", "Formula"),
                    ParamSpec::option_with_default(
                        "se",
                        "classical | robust | hc2 | hc3",
                        "classical",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                ],
                rest: None,
                returns: "RegFit",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "a term written twice kept once; a collinear column dropped; a category of one level",
                    ),
                    DiagnosticDecl::new(
                        "W1001",
                        "near-singular regressors, with the condition number",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable of the formula is not in the frame"),
                    DiagnosticDecl::new("Z0223", "an infinite value in a model variable"),
                    DiagnosticDecl::new(
                        "Z0304",
                        "a kind of standard error not yet available, or a date in the model before release 0.6.0",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "text in the model, a category as the outcome, or a level outside 0 to 1",
                    ),
                    DiagnosticDecl::new(
                        "Z0801",
                        "no row remains after the rows with NA are dropped",
                    ),
                    DiagnosticDecl::new(
                        "Z0802",
                        "no regressor remains, or no residual degrees of freedom",
                    ),
                ],
                help: include_str!("../help/reg.md"),
                accuracy: None,
            },
            body: reg::reg,
        },
        Function {
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
        },
    ]
}
