//! Package measure (tier 2): reliability and validity of measures (Engine and Project
//! Specification, sections 8.2 and 14). At this release one function, `reliability`:
//! Cronbach's alpha of a set of items with its interval, the alpha of the standardized items,
//! and for each item its correlation with the rest of the scale and the alpha of the scale
//! without it. McDonald's omega joins it from release 1.3, with the factor model it needs.

pub mod reliability;

use zeff_core::runtime::Function;
use zeff_registry::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `measure` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "reliability",
            package: "measure",
            method: true,
            params: vec![
                ParamSpec::required("data", "Frame"),
                ParamSpec::required("items", "Vars"),
                ParamSpec::option_with_default("standardize", "Bool", "false"),
                ParamSpec::option("reverse", "Vars"),
                ParamSpec::option_with_default("ci", "feldt | duhachek", "feldt"),
                ParamSpec::option_with_default("level", "Float", "0.95"),
            ],
            rest: None,
            returns: "TestResult",
            diagnostics: vec![
                DiagnosticDecl::note("rows with NA in an item dropped, with the count"),
                DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                DiagnosticDecl::new(
                    "Z0401",
                    "fewer than two items, an item written twice, or an item that is not a number or a Bool; an item of `reverse:` that is not among the items; fewer than two rows with every item; a sum of the items that does not vary; an item that does not vary under standardize: true; a level outside 0 and 1",
                ),
                DiagnosticDecl::new("Z0803", "an item with a value above 1e150 in size"),
                DiagnosticDecl::new(
                    "W0213",
                    "an item that runs against the rest of its scale and is not named in reverse:; an item that does not vary",
                ),
            ],
            help: include_str!("../help/reliability.md"),
            accuracy: Some(AccuracyRecord {
                reference: "R 4.6.0 with psych 2.6.9, alpha(), and the formulas in base R (tests/reference/r/reliability.R)"
                    .to_string(),
                last_run: None,
            }),
        },
        body: reliability::reliability,
    }]
}
