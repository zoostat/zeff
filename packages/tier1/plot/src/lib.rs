//! Package `plot` (tier 1): the layered grammar of plots (Language Reference, section 7.6).
//!
//! A plot is a value: a frame, the variables mapped to its aesthetics and its layers, declared
//! in `core` (Engine and Project Specification, section 8.1, rule 4) and made and extended here.
//! What is drawn is the *scene* derived from it ([`scene::scene`]): the marks in data coordinates
//! with their scales, guides and layout, a versioned format of the engine (section 8.3, point 5)
//! that the writers of SVG, PDF and PNG and Zemp read, never the plot value itself.
//!
//! At release 0.7.0, step 20d: `plot` with `x:` and `y:` as variables or as expressions over the
//! frame, `color:` and `fill:` as a variable or as one colour for every mark, `size:` as a
//! variable or as one diameter, `shape:` as a variable or as one shape ([`shape`]), `group:` and
//! `label:` held for the marks that read them, the mark `points`, the linear scale, the colour
//! scales of categories and of numbers ([`colour`]), the scale of sizes by area and that of
//! shapes, with their legends, one panel, and the scene as JSON and as the printed text of a
//! plot.

pub mod colour;
pub mod drawing;
pub mod figure;
pub mod font;
pub mod grammar;
pub mod json;
pub mod layout;
mod mapping;
pub mod marks;
pub mod pdf;
pub mod png;
pub mod raster;
pub mod scale;
pub mod scene;
pub mod shape;
pub mod svg;
mod trace;

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
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option("fill", "Var | Str"),
                    ParamSpec::option("size", "Var | Float"),
                    ParamSpec::option("shape", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option("label", "Var"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped or read by an expression",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable mapped to x: or y: that is not a Bool, an Int or a Float; an expression that does not give one for each row, or that holds what a mapping cannot compute; a string for color: or fill: that is not a colour; a Date or DateTime variable mapped to color: or fill:; a variable mapped to size: that is not an Int or a Float without value labels; a size: that is not above 0; a variable mapped to shape: that is not shown as categories; a string for shape: that is not the name of a shape; an expression given to color:, fill:, size:, shape:, group: or label:, which take a variable by its name",
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
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option("size", "Var | Float"),
                    ParamSpec::option("shape", "Var | Str"),
                    ParamSpec::option_with_default("alpha", "Float", "1"),
                    ParamSpec::option_with_default("jitter", "Float", "0"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x, y, a variable shown on a continuous colour scale or the variable mapped to size; the rows not drawn for a size below 0, and those drawn with a size of 0; more categories than the palette has colours, or than there are shapes; the seed chosen for jitter: when the script set none",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for color: that is not a colour, or for shape: that is not the name of a shape; a variable the layer names for color:, size: or shape: that the aesthetic cannot show; a variable the layer maps to color: of another kind than the one the scale already shows, or the variable the plot maps to color:, size: or shape: of another kind in data: than in the frame of the plot; an expression given to color:, size: or shape:, which take a variable by its name; alpha: outside [0, 1]; size: not above 0; jitter: below 0",
                    ),
                ],
                help: include_str!("../help/points.md"),
                accuracy: None,
            },
            body: marks::points,
        },
        Function {
            spec: FunctionSpec {
                name: "lines",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option("linetype", "Var | Str"),
                    ParamSpec::option_with_default("width", "Float", "1"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the places where a line is broken because y is NA; the rows with no neighbour to be joined to; more categories than the palette has colours, or than there are types of line",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for color: that is not a colour, or for linetype: that is not a type of line; a variable for linetype: that is not shown as categories; a colour by a number without value labels, where a line has one colour; a variable the layer maps to color: of another kind than the one the scale already shows; an expression given to color:, group: or linetype:, which take a variable by its name; width: not above 0",
                    ),
                ],
                help: include_str!("../help/lines.md"),
                accuracy: None,
            },
            body: marks::lines,
        },
        Function {
            spec: FunctionSpec {
                name: "step",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option("linetype", "Var | Str"),
                    ParamSpec::option_with_default("width", "Float", "1"),
                    ParamSpec::option_with_default("direction", "hv | vh | mid", "hv"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the places where a line is broken because y is NA; the rows with no neighbour to be joined to; more categories than the palette has colours, or than there are types of line",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for color: that is not a colour, or for linetype: that is not a type of line; a variable for linetype: that is not shown as categories; a colour by a number without value labels, where a line has one colour; a variable the layer maps to color: of another kind than the one the scale already shows; an expression given to color:, group: or linetype:, which take a variable by its name; width: not above 0",
                    ),
                ],
                help: include_str!("../help/step.md"),
                accuracy: None,
            },
            body: marks::step,
        },
        Function {
            spec: FunctionSpec {
                name: "area",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("fill", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("alpha", "Float", "0.6"),
                    ParamSpec::option_with_default("position", "overlay | stack | fill", "overlay"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the places where an area is broken because y is NA; the rows with no neighbour to be joined to; with position: stack or fill, the rows at a value of x where not every area has one value, and with fill those where a value is below 0 or the total is not above 0; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for fill: that is not a colour; a fill by a number without value labels, where an area has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; alpha: outside [0, 1]",
                    ),
                ],
                help: include_str!("../help/area.md"),
                accuracy: None,
            },
            body: marks::area,
        },
        Function {
            spec: FunctionSpec {
                name: "ribbon",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("low", "Var | Expr"),
                    ParamSpec::option("high", "Var | Expr"),
                    ParamSpec::option("fill", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("alpha", "Float", "0.3"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x, low or high; the places where a band is broken because low or high is NA; the rows with no neighbour to be joined to; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x; a variable the layer names for x:, low: or high:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for fill: that is not a colour; a fill by a number without value labels, where a band has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; alpha: outside [0, 1]",
                    ),
                    DiagnosticDecl::new("Z0406", "low: or high: is not given"),
                ],
                help: include_str!("../help/ribbon.md"),
                accuracy: None,
            },
            body: marks::ribbon,
        },
        Function {
            spec: FunctionSpec {
                name: "hline",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("at", "Float | Var"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option_with_default(
                        "style",
                        "solid | dashed | dotted | dash_dot | long_dash | dash_dot_dot",
                        "dashed",
                    ),
                    ParamSpec::option_with_default("width", "Float", "0.5"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in the variable of at: or a variable shown on a continuous colour scale; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name given to at: or color: is both a variable of the frame and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable for at: that is not numeric; an at: that is not a finite number; a number for at: when nothing is mapped to y; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows, or given with an at: that is one number; an expression given to color:, which takes a variable by its name; width: not above 0",
                    ),
                    DiagnosticDecl::new("Z0406", "at: is not given"),
                ],
                help: include_str!("../help/hline.md"),
                accuracy: None,
            },
            body: marks::hline,
        },
        Function {
            spec: FunctionSpec {
                name: "vline",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("at", "Float | Var"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option_with_default(
                        "style",
                        "solid | dashed | dotted | dash_dot | long_dash | dash_dot_dot",
                        "dashed",
                    ),
                    ParamSpec::option_with_default("width", "Float", "0.5"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in the variable of at: or a variable shown on a continuous colour scale; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name given to at: or color: is both a variable of the frame and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable for at: that is not numeric; an at: that is not a finite number; a number for at: when nothing is mapped to x; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows, or given with an at: that is one number; an expression given to color:, which takes a variable by its name; width: not above 0",
                    ),
                    DiagnosticDecl::new("Z0406", "at: is not given"),
                ],
                help: include_str!("../help/vline.md"),
                accuracy: None,
            },
            body: marks::vline,
        },
        Function {
            spec: FunctionSpec {
                name: "rug",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option_with_default("sides", "x | y | both", "x"),
                    ParamSpec::option_with_default("alpha", "Float", "1"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in the position of a side or a variable shown on a continuous colour scale; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression the layer maps is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to the position of a side; a variable the layer names for x: or y:, or a mapped variable of data:, that is not numeric, or an expression that gives no number for each row; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows; an expression given to color:, which takes a variable by its name; alpha: outside [0, 1]",
                    ),
                ],
                help: include_str!("../help/rug.md"),
                accuracy: None,
            },
            body: marks::rug,
        },
        Function {
            spec: FunctionSpec {
                name: "bars",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("fill", "Var | Str"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("position", "stack | dodge | fill", "stack"),
                    ParamSpec::option_with_default("width", "Float", "0.9"),
                    ParamSpec::option_with_default("alpha", "Float", "1"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; with position: fill, the bars at a value of x where a value is below 0 or the total is not above 0; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name the layer maps, or one in an expression it maps, is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x; a y that is categories, where a bar is drawn from zero; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives no number for each row; a string for fill: that is not a colour; a fill by a number without value labels, where a bar has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; width: not above 0 or above 1; alpha: outside [0, 1]",
                    ),
                ],
                help: include_str!("../help/bars.md"),
                accuracy: None,
            },
            body: marks::bars,
        },
        Function {
            spec: FunctionSpec {
                name: "tile",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("fill", "Var | Str"),
                    ParamSpec::option_with_default("alpha", "Float", "1"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x, y or a variable shown on a continuous scale of fill; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name the layer maps, or one in an expression it maps, is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives no number for each row; a string for fill: that is not a colour; a variable for fill: that a colour cannot show, or of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill:, which takes a variable by its name; alpha: outside [0, 1]",
                    ),
                ],
                help: include_str!("../help/tile.md"),
                accuracy: None,
            },
            body: marks::tile,
        },
        Function {
            spec: FunctionSpec {
                name: "text",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("x", "Var | Expr"),
                    ParamSpec::option("y", "Var | Expr"),
                    ParamSpec::option("label", "Var"),
                    ParamSpec::option("color", "Var | Str"),
                    ParamSpec::option_with_default("size", "Float", "8"),
                    ParamSpec::option("nudge", "List"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x, y or a variable shown on a continuous colour scale; the rows with no label to write; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name the layer maps, or one in an expression it maps, is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the frame the layer draws lacks a variable the layer names, or one an expression it maps reads; the frame of data: lacks a variable the plot maps, or one a mapped expression reads",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "the frame the layer draws, or the frame of data:, has the variable in another case",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "neither the plot nor the layer maps anything to x, to y or to label; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives no number for each row; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows; an expression given to label: or color:, which take a variable by its name; size: not above 0; a nudge: that is not a list of two numbers",
                    ),
                ],
                help: include_str!("../help/text.md"),
                accuracy: None,
            },
            body: marks::text,
        },
    ]
}
