//! Package results (tier 1): the model matrix of section 7.2 of the Language Reference, the
//! estimator trait and the variance layer (Engine and Project Specification, section 8.3, point
//! 2), and the shared result value assembled with its printed summary (section 8.1, rule 5). It
//! registers `export` on tables and results (steps 13 and 19), the result interface (step 15),
//! the estimator interface (steps 16 and 18) and `report` (step 19).

pub mod absorb;
pub mod dependence;
pub mod design;
pub mod estimator;
pub mod export;
pub mod inference;
pub mod interface;
pub mod likelihood;
pub mod margins;
pub mod options;
pub mod report;
pub mod summary;
pub mod variance;

use zeff_core::runtime::Function;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// A method of every result: the receiver `fit` of type `Result`.
fn result_method(
    name: &'static str,
    params: Vec<ParamSpec>,
    returns: &'static str,
    diagnostics: Vec<DiagnosticDecl>,
    help: &'static str,
    body: zeff_core::runtime::Body,
) -> Function {
    let mut all = vec![ParamSpec::required("fit", "Result")];
    all.extend(params);
    Function {
        spec: FunctionSpec {
            name,
            package: "results",
            method: true,
            params: all,
            rest: None,
            returns,
            diagnostics,
            help,
            accuracy: None,
        },
        body,
    }
}

/// A method of a test's result alone (open decision #94): the receiver `t` of type `TestResult`,
/// refused on a fit (Z0401).
fn test_method(
    name: &'static str,
    returns: &'static str,
    help: &'static str,
    body: zeff_core::runtime::Body,
) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "results",
            method: true,
            params: vec![ParamSpec::required("t", "TestResult")],
            rest: None,
            returns,
            diagnostics: vec![DiagnosticDecl::new(
                "Z0401",
                "a receiver that is not a test's result",
            )],
            help,
            accuracy: None,
        },
        body,
    }
}

/// A method of a test's result that reads one figure, of the whole test or of the row of its
/// table that `term` names (open decision #100).
fn row_method(
    name: &'static str,
    returns: &'static str,
    help: &'static str,
    body: zeff_core::runtime::Body,
) -> Function {
    let mut method = test_method(name, returns, help, body);
    method.spec.params.push(ParamSpec::option("term", "Term"));
    method.spec.diagnostics.push(DiagnosticDecl::new(
        "Z0414",
        "a name that is no row of the result's table",
    ));
    method
}

/// The functions of `results` in this release, with their help pages: `export` on a table, and
/// the result interface on every result; `export` on a result arrives with step 19.
pub fn functions() -> Vec<Function> {
    let term = || ParamSpec::option("term", "Term");
    let unknown_term = || DiagnosticDecl::new("Z0414", "a term the result does not have");
    vec![
        result_method(
            "summary",
            vec![
                ParamSpec::option_with_default("standardized", "Bool", "false"),
                ParamSpec::option_with_default("fit_measures", "Bool", "false"),
                ParamSpec::option_with_default("exponentiate", "Bool", "false"),
            ],
            "Table",
            vec![],
            include_str!("../help/summary.md"),
            interface::summary,
        ),
        result_method(
            "estimates",
            vec![],
            "Frame",
            vec![],
            include_str!("../help/estimates.md"),
            interface::estimates,
        ),
        result_method(
            "std_errors",
            vec![term()],
            "ByTerm<Float, Mapping<Str, Float>>",
            vec![unknown_term()],
            include_str!("../help/std_errors.md"),
            interface::std_errors,
        ),
        result_method(
            "p_values",
            vec![term()],
            "ByTerm<Float, Mapping<Str, Float>>",
            vec![unknown_term()],
            include_str!("../help/p_values.md"),
            interface::p_values,
        ),
        result_method(
            "ci",
            vec![term(), ParamSpec::option("level", "Float")],
            "ByTerm<List<Float>, Table>",
            vec![
                unknown_term(),
                DiagnosticDecl::new("Z0401", "a level outside 0 to 1"),
            ],
            include_str!("../help/ci.md"),
            interface::ci,
        ),
        result_method(
            "n_obs",
            vec![],
            "Int",
            vec![],
            include_str!("../help/n_obs.md"),
            interface::n_obs,
        ),
        row_method(
            "statistic",
            "Float",
            include_str!("../help/statistic.md"),
            interface::statistic,
        ),
        row_method(
            "df",
            "List<Float>",
            include_str!("../help/df.md"),
            interface::df,
        ),
        row_method(
            "effect_size",
            "Float",
            include_str!("../help/effect_size.md"),
            interface::effect_size,
        ),
        test_method("h0", "Str", include_str!("../help/h0.md"), interface::h0),
        test_method("h1", "Str", include_str!("../help/h1.md"), interface::h1),
        test_method(
            "alternative",
            "Str",
            include_str!("../help/alternative.md"),
            interface::alternative,
        ),
        result_method(
            "coef",
            vec![term()],
            "ByTerm<Float, Mapping<Str, Float>>",
            vec![unknown_term()],
            include_str!("../help/coef.md"),
            interface::coef,
        ),
        result_method(
            "vcov",
            vec![],
            "Table",
            vec![],
            include_str!("../help/vcov.md"),
            interface::vcov,
        ),
        result_method(
            "fit_measures",
            vec![],
            "ByFit<{log_likelihood: Float, aic: Float, bic: Float, r_squared: Float, adjusted: Float}, {log_likelihood: Float, aic: Float, bic: Float, deviance: Float, null_deviance: Float, r_squared_mcfadden: Float}>",
            vec![],
            include_str!("../help/fit_measures.md"),
            interface::fit_measures,
        ),
        result_method(
            "predict",
            vec![
                ParamSpec::option("newdata", "Frame"),
                ParamSpec::option_with_default("type", "response | link", "response"),
            ],
            "Column<Float>",
            vec![
                DiagnosticDecl::new(
                    "W0207",
                    "rows of `newdata` with a level the fit never saw, whose predictions are NA",
                ),
                DiagnosticDecl::new("Z0101", "a variable of the model that `newdata` lacks"),
                DiagnosticDecl::new(
                    "Z0401",
                    "a variable of `newdata` of a type the model cannot take, or `newdata:` not a frame",
                ),
                DiagnosticDecl::new("Z0406", "`newdata:` not given"),
            ],
            include_str!("../help/predict.md"),
            interface::predict,
        ),
        result_method(
            "residuals",
            vec![ParamSpec::option_with_default(
                "type",
                "response | pearson | deviance | working",
                "response",
            )],
            "Column<Float>",
            vec![DiagnosticDecl::new(
                "Z0401",
                "`type:` other than `response` on a fit that is not by maximum likelihood",
            )],
            include_str!("../help/residuals.md"),
            interface::residuals,
        ),
        result_method(
            "fitted",
            vec![],
            "Column<Float>",
            vec![],
            include_str!("../help/fitted.md"),
            interface::fitted,
        ),
        Function {
            spec: FunctionSpec {
                name: "test",
                package: "results",
                method: true,
                params: vec![ParamSpec::required("fit", "Result")],
                rest: Some(Rest::Values("Term<Bool>")),
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a receiver that is not a fit, or a restriction that is not an equality over the terms",
                    ),
                    DiagnosticDecl::new("Z0406", "no restriction given"),
                    DiagnosticDecl::new(
                        "Z0414",
                        "a term the fit does not have, or one omitted as collinear",
                    ),
                    DiagnosticDecl::new(
                        "Z0415",
                        "a restriction that is not linear in the coefficients, or restrictions that repeat one another",
                    ),
                ],
                help: include_str!("../help/test.md"),
                accuracy: None,
            },
            body: inference::test,
        },
        Function {
            spec: FunctionSpec {
                name: "compare",
                package: "results",
                method: true,
                params: vec![
                    ParamSpec::required("fit", "Result"),
                    ParamSpec::required("other", "Result"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a receiver or an argument that is not a regression fit",
                    ),
                    DiagnosticDecl::new(
                        "Z0416",
                        "two fits on different rows, with the same terms, or not nested; a regression beside a `glm` fit; two `glm` fits of another link, or with robust or clustered errors or sampling weights",
                    ),
                ],
                help: include_str!("../help/compare.md"),
                accuracy: None,
            },
            body: inference::compare,
        },
        Function {
            spec: FunctionSpec {
                name: "derive",
                package: "results",
                method: true,
                params: vec![ParamSpec::required("fit", "Result")],
                rest: Some(Rest::Definitions("Term<Float>")),
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a receiver that is not a regression fit, or a definition that is not an expression over the terms",
                    ),
                    DiagnosticDecl::new("Z0406", "no definition given"),
                    DiagnosticDecl::new(
                        "Z0414",
                        "a term the fit does not have, or one omitted as collinear",
                    ),
                    DiagnosticDecl::new("Z0417", "an expression `derive` cannot differentiate"),
                ],
                help: include_str!("../help/derive.md"),
                accuracy: None,
            },
            body: inference::derive,
        },
        Function {
            spec: FunctionSpec {
                name: "margins",
                package: "results",
                method: true,
                params: vec![
                    ParamSpec::required("fit", "RegFit | GlmFit"),
                    // Left out, the adjusted prediction (open decision #202 (b)).
                    ParamSpec {
                        name: "vars",
                        ty: "Vars",
                        default: Some("NA"),
                        option: false,
                    },
                    ParamSpec::option("at", "Record"),
                    ParamSpec::option_with_default("type", "ame | mem", "ame"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a receiver that is not a `reg` or a `glm` fit; `vars` that does not name variables; a value of `at:` that is not a number for a number or a level for a category, or a list; a 0/1 variable or a category both asked for and held by `at:`; no variable named after `fe:`",
                    ),
                    DiagnosticDecl::new(
                        "Z0414",
                        "a variable, asked for or held by `at:`, that no term of the model is a function of",
                    ),
                    DiagnosticDecl::new(
                        "Z0417",
                        "a variable whose effect cannot be computed or that cannot be held: one that enters the model both as a number and as a category, a slope that is not finite where it is asked for, or a term with no value at the values of `at:` or at the means",
                    ),
                ],
                help: include_str!("../help/margins.md"),
                accuracy: None,
            },
            body: margins::margins,
        },
        Function {
            spec: FunctionSpec {
                name: "export",
                package: "results",
                method: true,
                params: vec![
                    ParamSpec::required("x", "Table | Result | Plot"),
                    ParamSpec::required("path", "Str"),
                    ParamSpec::option("size", "one_column | two_column | slide | List<Float>"),
                    ParamSpec::option_with_default("notes", "Bool", "false"),
                ],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "size: or notes: given for a table or a result; a size that is not one of the three words or a width and a height in millimetres above 0; a page too small for the plot's labels at 8 points, larger than a page of a PDF may be, or of more dots than a PNG holds",
                    ),
                    DiagnosticDecl::new(
                        "Z0707",
                        "a path outside the project or in a data root, a folder other than `output/` that does not exist, or a file that cannot be written",
                    ),
                    DiagnosticDecl::new(
                        "Z0716",
                        "a path with no extension, an extension that is not a format `export` writes, or the format of a figure for a table or of a table for a plot",
                    ),
                ],
                help: include_str!("../help/export.md"),
                accuracy: None,
            },
            body: export::export,
        },
        Function {
            spec: FunctionSpec {
                name: "report",
                package: "results",
                method: false,
                params: vec![
                    ParamSpec::required("fits", "List<Result>"),
                    ParamSpec::option(
                        "stats",
                        "List<n_obs | r_squared | r_squared_within | aic | bic>",
                    ),
                    ParamSpec::option_with_default("digits", "Int", "3"),
                    ParamSpec::option_with_default("stars", "Bool", "true"),
                    ParamSpec::option("labels", "Record"),
                    ParamSpec::option("title", "Str"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a fit that is not a regression (a test's result), `digits:` outside 0 to 10, or a label that is not text",
                    ),
                    DiagnosticDecl::new(
                        "Z0406",
                        "an empty list of fits, or a list whose fits all failed",
                    ),
                    DiagnosticDecl::new("W0801", "failed values in the list, skipped and counted"),
                    DiagnosticDecl::new("Z0414", "a label for a term none of the fits has"),
                ],
                help: include_str!("../help/report.md"),
                accuracy: None,
            },
            body: report::report,
        },
    ]
}
