//! Package regress (tier 2): linear regression (Engine and Project Specification, section 13;
//! release 0.4.0): `reg`, ordinary least squares on the kernels with the variance layer of
//! `results`; `interact` and `event`, the formula terms of section 7.2 and catalogue row 252;
//! and the methods of regression fits, `r_squared`, `influence` and `vif`; and, from release
//! 0.8.0, two-stage least squares (`iv`), the estimator behind `ivreg`.

pub mod diagnostics;
pub mod iv;
pub mod ivreg;
pub mod ols;
pub mod reg;

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec};

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

/// `event(time, treat, ref:)` is read by the model matrix inside a formula (catalogue row 252;
/// question 2 of release 0.6.0); called anywhere else it has no value to give.
pub fn event(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error(
            "Z0401",
            "`event` is a term of a formula and has no value outside one",
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "called outside a formula")
        .help("write it on the right of `~`: `y ~ event(year, treated_in)`, usually with `fe: unit + year`"),
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
                    ParamSpec::option("fe", "Terms"),
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
                        "a term written twice kept once; a collinear column, a level with no observations, a zero column or one too small to scale dropped; a regressor absorbed by the fixed effects; a category of one level; rows with a zero weight left out; clusters of one observation; an outcome without variation; an essentially perfect fit",
                    ),
                    DiagnosticDecl::new(
                        "W1001",
                        "near-singular regressors, with the condition number",
                    ),
                    DiagnosticDecl::new("W1002", "fewer than 30 clusters"),
                    DiagnosticDecl::new("W1003", "extreme weights, with the effective sample size"),
                    DiagnosticDecl::new(
                        "W1004",
                        "little variation left after the fixed effects: a regressor keeping less than one per cent of its norm",
                    ),
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
                        "text in the model, a category as the outcome, a level outside 0 to 1, a weight that is not a number, a frequency weight with a fraction, `fe:` with `*`, a range, an expression or the intercept, or a fixed effect that is a model variable too",
                    ),
                    DiagnosticDecl::new(
                        "Z0410",
                        "`se: classical` with sampling weights, `se: hc2` or `hc3` with `cluster:`, or `se: hc2` or `hc3` with `fe:`",
                    ),
                    DiagnosticDecl::new(
                        "Z0801",
                        "no row remains after the rows with NA are dropped, or every row is a singleton of the fixed effects",
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
                name: "ivreg",
                package: "regress",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("formula", "Formula"),
                    ParamSpec::option("fe", "Terms"),
                    ParamSpec::option("instruments", "Instruments"),
                    ParamSpec::option("se", "classical | robust | hc2 | hc3"),
                    ParamSpec::option("cluster", "Var"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                    ParamSpec::option_with_default("first", "Bool", "false"),
                ],
                rest: None,
                returns: "IvFit",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "an instrument that is a regressor of the formula already; an instrument dropped as zero, as collinear with the regressors and instruments before it, or as absorbed by the fixed effects; a regressor dropped as `reg` drops it",
                    ),
                    DiagnosticDecl::new(
                        "W1001",
                        "near-singular regressors, with the condition number",
                    ),
                    DiagnosticDecl::new("W1002", "fewer than 30 clusters"),
                    DiagnosticDecl::new("W1003", "extreme weights, with the effective sample size"),
                    DiagnosticDecl::new(
                        "W1007",
                        "weak instruments: the F under the table is under 10, or the effective F is under its critical value",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "a variable of the formula, `instruments:`, `fe:`, `cluster:` or `weights:` is not in the frame",
                    ),
                    DiagnosticDecl::new("Z0223", "an infinite value in a model variable"),
                    DiagnosticDecl::new("Z0224", "a negative weight"),
                    DiagnosticDecl::new(
                        "Z0304",
                        "`se: bootstrap` before release 1.1",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "text in the model, a category as the outcome, a level outside 0 to 1, a weight that is not a number, a fixed effect that is a variable of the formula or an instrument, or a product instrumented as one term",
                    ),
                    DiagnosticDecl::new("Z0406", "no `instruments:`"),
                    DiagnosticDecl::new(
                        "Z0410",
                        "`se: classical` with sampling weights, `se: hc2` or `hc3` with `cluster:`, or `se: hc2` or `hc3` with `fe:`",
                    ),
                    DiagnosticDecl::new(
                        "Z0504",
                        "a term instrumented in `instruments:` that is not a regressor of the formula",
                    ),
                    DiagnosticDecl::new("Z0505", "a variable on both sides of `~` in `instruments:`"),
                    DiagnosticDecl::new(
                        "Z0801",
                        "no row remains after the rows with NA are dropped, or every row is a singleton of the fixed effects",
                    ),
                    DiagnosticDecl::new(
                        "Z0802",
                        "no regressor remains, no residual degrees of freedom, one cluster, or an observation with leverage one under `se: hc2` or `hc3`",
                    ),
                    DiagnosticDecl::new(
                        "Z0803",
                        "an outcome with values too large to compute with (above 1e150 in size) or too small against its regressors",
                    ),
                    DiagnosticDecl::new(
                        "Z0804",
                        "fewer instruments outside the formula than instrumented regressors, counted after the instruments dropped, or instruments that move the instrumented regressors only together",
                    ),
                ],
                help: include_str!("../help/ivreg.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 with fixest 0.14.2 feols() and two-stage least squares written out in base R (tests/reference/r/iv.R)".to_string(),
                    last_run: None,
                }),
            },
            body: ivreg::ivreg,
        },
        Function {
            spec: FunctionSpec {
                name: "first_stage",
                package: "regress",
                method: true,
                params: vec![ParamSpec::required("iv", "IvFit")],
                rest: None,
                returns: "Table",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "the receiver is not a fit by `ivreg`",
                )],
                help: include_str!("../help/first_stage.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 with fixest 0.14.2 fitstat() and the statistics written out in base R (tests/reference/r/iv.R)".to_string(),
                    last_run: None,
                }),
            },
            body: ivreg::first_stage,
        },
        Function {
            spec: FunctionSpec {
                name: "ar_test",
                package: "regress",
                method: true,
                params: vec![
                    ParamSpec::required("iv", "IvFit"),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "the receiver is not a fit by `ivreg`, or a level outside 0 to 1",
                    ),
                    DiagnosticDecl::new(
                        "Z0802",
                        "nothing is instrumented in the fit, or the test cannot be computed for it",
                    ),
                ],
                help: include_str!("../help/ar_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0, the Anderson-Rubin test and set written out in base R two ways (tests/reference/r/ar.R)".to_string(),
                    last_run: None,
                }),
            },
            body: ivreg::ar_test,
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
                name: "event",
                package: "regress",
                method: false,
                params: vec![
                    ParamSpec::required("time", "Var"),
                    ParamSpec::required("treat", "Var"),
                    ParamSpec::option_with_default("ref", "Int", "-1"),
                ],
                rest: None,
                returns: "Column",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "called outside a formula; `time` and `treat` not both Int or both Date; no treated row at all",
                    ),
                    DiagnosticDecl::new(
                        "Z0507",
                        "`ref:` is not a relative period of the data; the message lists the periods present",
                    ),
                ],
                help: include_str!("../help/event.md"),
                accuracy: None,
            },
            body: event,
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
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0304",
                    "a fit with fixed effects, whose influence measures arrive with release 0.8.0",
                )],
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
        Function {
            spec: FunctionSpec {
                name: "design",
                package: "regress",
                method: true,
                params: vec![ParamSpec::required("fit", "RegFit | IvFit")],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "the receiver is not a regression fit",
                )],
                help: include_str!("../help/design.md"),
                accuracy: None,
            },
            body: diagnostics::design,
        },
    ]
}

/// The functions of `regress` written in Zeff, one `functions/*.zeff` file each (Engine and Project
/// Specification, section 9.1, "Files"), compiled in; the engine parses, checks and lowers them
/// when it starts. None yet: they arrive with the function steps of release 0.5.0.
pub fn zeff_sources() -> Vec<zeff_registry::ZeffSource> {
    Vec::new()
}
