//! Package regress (tier 2): linear regression (Engine and Project Specification, section 13;
//! release 0.4.0): `reg`, ordinary least squares on the kernels with the variance layer of
//! `results`; `interact`, the formula term of section 7.2; and the methods of regression fits,
//! `r_squared` and `influence`.

pub mod diagnostics;
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
                    ParamSpec::option("se", "classical | robust | hc2 | hc3"),
                    ParamSpec::option("cluster", "Var"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                ],
                rest: None,
                returns: "RegFit",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "a term written twice kept once; a collinear column, a level with no observations, a zero column or one too small to scale dropped; a category of one level; rows with a zero weight left out; clusters of one observation; an outcome without variation; an essentially perfect fit",
                    ),
                    DiagnosticDecl::new(
                        "W1001",
                        "near-singular regressors, with the condition number",
                    ),
                    DiagnosticDecl::new("W1002", "fewer than 30 clusters"),
                    DiagnosticDecl::new("W1003", "extreme weights, with the effective sample size"),
                    DiagnosticDecl::new(
                        "W1006",
                        "an observation whose removal moves an estimate by more than its standard error",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "a variable of the formula, `cluster:` or `weights:` is not in the frame",
                    ),
                    DiagnosticDecl::new("Z0223", "an infinite value in a model variable"),
                    DiagnosticDecl::new("Z0224", "a negative weight"),
                    DiagnosticDecl::new(
                        "Z0304",
                        "`se: bootstrap` before release 1.1, or a date in the model before 0.6.0",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "text in the model, a category as the outcome, a level outside 0 to 1, a weight that is not a number, or a frequency weight with a fraction",
                    ),
                    DiagnosticDecl::new(
                        "Z0410",
                        "`se: classical` with sampling weights, or `se: hc2` or `hc3` with `cluster:`",
                    ),
                    DiagnosticDecl::new(
                        "Z0801",
                        "no row remains after the rows with NA are dropped",
                    ),
                    DiagnosticDecl::new(
                        "Z0802",
                        "no regressor remains, no residual degrees of freedom, one cluster, or an observation with leverage one under `se: hc2` or `hc3`",
                    ),
                    DiagnosticDecl::new(
                        "Z0803",
                        "an outcome with values too large to compute with (above 1e150 in size) or too small against its regressors (an estimate below 2.2e-308)",
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
        Function {
            spec: FunctionSpec {
                name: "r_squared",
                package: "regress",
                method: true,
                params: vec![ParamSpec::required("fit", "RegFit")],
                rest: None,
                returns: "Record",
                diagnostics: vec![],
                help: include_str!("../help/r_squared.md"),
                accuracy: None,
            },
            body: diagnostics::r_squared,
        },
        Function {
            spec: FunctionSpec {
                name: "influence",
                package: "regress",
                method: true,
                params: vec![ParamSpec::required("fit", "RegFit")],
                rest: None,
                returns: "Frame",
                diagnostics: vec![],
                help: include_str!("../help/influence.md"),
                accuracy: None,
            },
            body: diagnostics::influence,
        },
        Function {
            spec: FunctionSpec {
                name: "vif",
                package: "regress",
                method: true,
                params: vec![ParamSpec::required("fit", "RegFit")],
                rest: None,
                returns: "Table",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "the receiver is not a regression fit",
                )],
                help: include_str!("../help/vif.md"),
                accuracy: None,
            },
            body: diagnostics::vif,
        },
    ]
}

/// The functions of `regress` written in Zeff, one `functions/*.zeff` file each (Engine and Project
/// Specification, section 9.1, "Files"), compiled in; the engine parses, checks and lowers them
/// when it starts. None yet: they arrive with the function steps of release 0.5.0.
pub fn zeff_sources() -> Vec<zeff_registry::ZeffSource> {
    Vec::new()
}
