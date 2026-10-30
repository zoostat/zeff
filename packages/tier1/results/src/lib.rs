//! Package results (tier 1): the model matrix of section 7.2 of the Language Reference, the
//! estimator trait and the variance layer (Engine and Project Specification, section 8.3, point
//! 2), and the shared result value assembled with its printed summary (section 8.1, rule 5). It
//! registers `export` on tables and results (steps 13 and 19), the result interface (step 15),
//! the estimator interface (steps 16 and 18) and `report` (step 19).

pub mod design;
pub mod estimator;
pub mod export;
pub mod inference;
pub mod interface;
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
            vec![
                term(),
                ParamSpec::option_with_default("level", "Float", "0.95"),
            ],
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
            "Record",
            vec![],
            include_str!("../help/fit_measures.md"),
            interface::fit_measures,
        ),
        result_method(
            "predict",
            vec![ParamSpec::option("newdata", "Frame")],
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
            vec![],
            "Column<Float>",
            vec![],
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
                        "a receiver that is not a regression fit, or a restriction that is not an equality over the terms",
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
                        "two regressions on different rows, with the same terms, or not nested",
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
                name: "export",
                package: "results",
                method: true,
                params: vec![
                    ParamSpec::required("x", "Table | Result"),
                    ParamSpec::required("path", "Str"),
                ],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0707",
                        "a path outside the project or in a data root, a folder other than `output/` that does not exist, or a file that cannot be written",
                    ),
                    DiagnosticDecl::new(
                        "Z0716",
                        "a path with no extension, or an extension that is not a format `export` writes",
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
                    ParamSpec::option("stats", "List<n_obs | r_squared | aic | bic>"),
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
