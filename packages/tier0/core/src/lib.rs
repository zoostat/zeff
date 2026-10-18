//! Package core (tier 0): the language and the types every package shares.

use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// The functions of `core` in this release, with their help pages.
pub fn functions() -> Vec<FunctionSpec> {
    vec![FunctionSpec {
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
    }]
}
