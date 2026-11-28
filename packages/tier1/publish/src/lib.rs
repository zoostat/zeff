//! Package publish (tier 1; Engine and Project Specification, section 8.2, package 13; open
//! decision #141, option A): say what was found, and cite it. At release 0.10.0 it holds
//! `report`, the publication table of several results, moved here from `results` at step 24
//! with nothing it prints changed; `cite` joins it at step 27, and the reports written in
//! Markdown after 1.0.

pub mod report;

use zeff_core::runtime::Function;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `publish` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "report",
            package: "publish",
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
    }]
}
