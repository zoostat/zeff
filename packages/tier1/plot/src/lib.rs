//! Package `plot` (tier 1): the layered grammar of plots (Language Reference, section 7.6).
//!
//! A plot is a value: a frame, the variables mapped to its aesthetics and its layers, declared
//! in `core` (Engine and Project Specification, section 8.1, rule 4) and made and extended here.
//! What is drawn is the *scene* derived from it ([`scene::scene`]): the marks in data coordinates
//! with their scales, guides and layout, a versioned format of the engine (section 8.3, point 5)
//! that the writers of SVG, PDF and PNG and Zemp read, never the plot value itself.
//!
//! At release 0.7.0, step 20: `plot` with `x:` and `y:` as variables, the mark `points`, the
//! linear scale, one panel, and the scene as JSON and as the printed text of a plot.

pub mod grammar;
pub mod json;
pub mod marks;
pub mod scale;
pub mod scene;

pub use scene::{SCENE_VERSION, Scene, scene};

use zeff_core::runtime::Function;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `plot` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "plot",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("x", "Var"),
                    ParamSpec::option("y", "Var"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "no such variable in the frame"),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable mapped to x: or y: that is not a Bool, an Int or a Float",
                    ),
                ],
                help: include_str!("../help/plot.md"),
                accuracy: None,
            },
            body: grammar::plot,
        },
        Function {
            spec: FunctionSpec {
                name: "points",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option_with_default("alpha", "Float", "1"),
                    ParamSpec::option("size", "Float"),
                    ParamSpec::option_with_default("jitter", "Float", "0"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the seed chosen for jitter: when the script set none",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame of data: lacks a variable the plot maps",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame of data: has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the plot maps no variable to x or to y; a mapped variable of data: that is not numeric; alpha: outside [0, 1]; size: not above 0; jitter: below 0",
                    ),
                ],
                help: include_str!("../help/points.md"),
                accuracy: None,
            },
            body: marks::points,
        },
    ]
}
