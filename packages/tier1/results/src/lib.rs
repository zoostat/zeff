//! Package results (tier 1): the model matrix of section 7.2 of the Language Reference, the
//! estimator trait and the variance layer (Engine and Project Specification, section 8.3, point
//! 2), and the shared result value assembled with its printed summary (section 8.1, rule 5). It
//! registers `export` on tables (step 13); the result methods arrive with step 15.

pub mod design;
pub mod estimator;
pub mod export;
pub mod summary;
pub mod variance;

use zeff_core::runtime::Function;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `results` in this release, with their help pages: `export` on a table; the
/// result methods arrive with step 15 and `export` on a result with step 19.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "export",
            package: "results",
            method: true,
            params: vec![
                ParamSpec::required("x", "Table"),
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
    }]
}
