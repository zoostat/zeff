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

pub mod bins;
pub mod colour;
pub mod dates;
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
pub mod scales;
pub mod scene;
pub mod shape;
pub mod shorthands;
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
                        "a variable mapped to x: or y: of a type that no axis holds; an expression that does not give a number, text or a date for each row, or that holds what a mapping cannot compute; a string for color: or fill: that is not a colour; a Date or DateTime variable mapped to color: or fill:; a variable mapped to size: that is not an Int or a Float without value labels; a size: that is not above 0; a variable mapped to shape: that is not shown as categories; a string for shape: that is not the name of a shape; an expression given to color:, fill:, size:, shape:, group: or label:, which take a variable by its name",
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
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for color: that is not a colour, or for shape: that is not the name of a shape; a variable the layer names for color:, size: or shape: that the aesthetic cannot show; a variable the layer maps to color: of another kind than the one the scale already shows, or the variable the plot maps to color:, size: or shape: of another kind in data: than in the frame of the plot; an expression given to color:, size: or shape:, which take a variable by its name; alpha: outside [0, 1]; size: not above 0; jitter: below 0",
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
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for color: that is not a colour, or for linetype: that is not a type of line; a variable for linetype: that is not shown as categories; a colour by a number without value labels, where a line has one colour; a variable the layer maps to color: of another kind than the one the scale already shows; an expression given to color:, group: or linetype:, which take a variable by its name; width: not above 0",
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
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for color: that is not a colour, or for linetype: that is not a type of line; a variable for linetype: that is not shown as categories; a colour by a number without value labels, where a line has one colour; a variable the layer maps to color: of another kind than the one the scale already shows; an expression given to color:, group: or linetype:, which take a variable by its name; width: not above 0",
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
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y:, or a mapped variable of data:, of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for fill: that is not a colour; a fill by a number without value labels, where an area has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; alpha: outside [0, 1]",
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
                        "neither the plot nor the layer maps anything to x; a variable the layer names for x:, low: or high:, or a mapped variable of data:, that is not numeric, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for fill: that is not a colour; a fill by a number without value labels, where a band has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; alpha: outside [0, 1]",
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
                    ParamSpec::option("at", "Float | Date | DateTime | Var"),
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
                        "a variable for at: that is not a number or a date, or of another kind than the axis holds; a number for at: on an axis of dates, or a date on an axis of numbers; an at: that is not a finite number; a number for at: when nothing is mapped to y; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows, or given with an at: that is one number; an expression given to color:, which takes a variable by its name; width: not above 0",
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
                    ParamSpec::option("at", "Float | Date | DateTime | Var"),
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
                        "a variable for at: that is not a number or a date, or of another kind than the axis holds; a number for at: on an axis of dates, or a date on an axis of numbers; an at: that is not a finite number; a number for at: when nothing is mapped to x; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows, or given with an at: that is one number; an expression given to color:, which takes a variable by its name; width: not above 0",
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
                        "neither the plot nor the layer maps anything to the position of a side; a variable the layer names for x: or y:, or a mapped variable of data:, of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows; an expression given to color:, which takes a variable by its name; alpha: outside [0, 1]",
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
                        "neither the plot nor the layer maps anything to x; a y that is categories or dates, where a bar is drawn from zero; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for fill: that is not a colour; a fill by a number without value labels, where a bar has one fill; a variable the layer maps to fill: of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill: or group:, which take a variable by its name; width: not above 0 or above 1; alpha: outside [0, 1]",
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
                        "neither the plot nor the layer maps anything to x or to y; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for fill: that is not a colour; a variable for fill: that a colour cannot show, or of another kind than the one the scale already shows, or the variable the plot maps to fill: of another kind in data: than in the frame of the plot; an expression given to fill:, which takes a variable by its name; alpha: outside [0, 1]",
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
                        "neither the plot nor the layer maps anything to x, to y or to label; a variable the layer names for x: or y: of another kind than the axis already has, or one that no axis holds, or an expression that gives nothing an axis places for each row, or another kind of value on the frame of data: than on the frame of the plot; a string for color: that is not a colour; a variable for color: that a colour cannot show, or of another kind than the one the scale already shows; an expression given to label: or color:, which take a variable by its name; size: not above 0; a nudge: that is not a list of two numbers",
                    ),
                ],
                help: include_str!("../help/text.md"),
                accuracy: None,
            },
            body: marks::text,
        },
        Function {
            spec: FunctionSpec {
                name: "scale_x",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option_with_default("type", "linear | log | sqrt | date", "linear"),
                    ParamSpec::option("limits", "List"),
                    ParamSpec::option("breaks", "List"),
                    ParamSpec::option("labels", "List | Str"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the marks beyond the limits set for the axis, which the edge of the panel cuts off",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a value of x at or below 0 on a log scale, or below 0 on a square-root scale, with the rows named; a bar or an area on a log scale; an axis of categories; a log or a square-root scale on an axis of dates, or a scale of dates on an axis of numbers; limits: or breaks: in numbers on an axis of dates, or in dates on an axis of numbers; a pattern for labels: that cannot be used, or one on an axis of numbers; limits: that is not a list of two numbers with the lower below the upper, or a limit with no place on the scale; breaks: that is not a list of numbers, or a break with no place on the scale; labels: without breaks:, or not one for each break; date and a pattern for labels:, which are built at the next step",
                    ),
                ],
                help: include_str!("../help/scale_x.md"),
                accuracy: None,
            },
            body: scales::scale_x,
        },
        Function {
            spec: FunctionSpec {
                name: "scale_y",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option_with_default("type", "linear | log | sqrt | date", "linear"),
                    ParamSpec::option("limits", "List"),
                    ParamSpec::option("breaks", "List"),
                    ParamSpec::option("labels", "List | Str"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the marks beyond the limits set for the axis, which the edge of the panel cuts off",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a value of y at or below 0 on a log scale, or below 0 on a square-root scale, with the rows named; a bar or an area on a log scale; an axis of categories; a log or a square-root scale on an axis of dates, or a scale of dates on an axis of numbers; limits: or breaks: in numbers on an axis of dates, or in dates on an axis of numbers; a pattern for labels: that cannot be used, or one on an axis of numbers; limits: that is not a list of two numbers with the lower below the upper, or a limit with no place on the scale; breaks: that is not a list of numbers, or a break with no place on the scale; labels: without breaks:, or not one for each break; date and a pattern for labels:, which are built at the next step",
                    ),
                ],
                help: include_str!("../help/scale_y.md"),
                accuracy: None,
            },
            body: scales::scale_y,
        },
        Function {
            spec: FunctionSpec {
                name: "scale",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("color", "List | viridis | okabe_ito"),
                    ParamSpec::option("fill", "List | viridis | okabe_ito"),
                    ParamSpec::option("size", "List"),
                    ParamSpec::option("shape", "List"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "more categories than the scale has colours or shapes, which repeat",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a list of colours that is empty or holds what is not a colour; okabe_ito, or a list of one colour, for a variable shown on a continuous scale; sizes that are not one number or two above 0, or the smaller after the larger; a list of shapes that is empty or holds what is not the name of a shape",
                    ),
                    DiagnosticDecl::new(
                        "Z0406",
                        "none of color:, fill:, size: and shape: is given",
                    ),
                ],
                help: include_str!("../help/scale.md"),
                accuracy: None,
            },
            body: scales::scale,
        },
        Function {
            spec: FunctionSpec {
                name: "legend",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option_with_default(
                        "position",
                        "right | top | bottom | none | direct",
                        "right",
                    ),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![],
                help: include_str!("../help/legend.md"),
                accuracy: None,
            },
            body: scales::legend,
        },
        Function {
            spec: FunctionSpec {
                name: "labels",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("title", "Str"),
                    ParamSpec::option("subtitle", "Str"),
                    ParamSpec::option("x", "Str"),
                    ParamSpec::option("y", "Str"),
                    ParamSpec::option("color", "Str"),
                    ParamSpec::option("caption", "Str"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0406",
                    "none of title:, subtitle:, x:, y:, color: and caption: is given",
                )],
                help: include_str!("../help/labels.md"),
                accuracy: None,
            },
            body: scales::labels,
        },
        Function {
            spec: FunctionSpec {
                name: "theme",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option_with_default("name", "journal | minimal | slide", "journal"),
                    ParamSpec::option_with_default("base_size", "Float", "9"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "a base_size: below 9, at which the labels would be set below 8 points",
                )],
                help: include_str!("../help/theme.md"),
                accuracy: None,
            },
            body: scales::theme,
        },
        Function {
            spec: FunctionSpec {
                name: "annotate",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("text", "Str"),
                    ParamSpec::option("x", "Float | Str | Date | DateTime"),
                    ParamSpec::option("y", "Float | Str | Date | DateTime"),
                    ParamSpec::option("size", "Float"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a text that is empty; a place of another kind than the axis holds, a number on an axis of dates or of categories and the like; a category the axis does not have; nothing mapped to x or to y; a place with no place on a log or a square-root scale; a size: not above 0",
                    ),
                    DiagnosticDecl::new("Z0406", "text:, x: or y: is not given"),
                ],
                help: include_str!("../help/annotate.md"),
                accuracy: None,
            },
            body: marks::annotate,
        },
        Function {
            spec: FunctionSpec {
                name: "histogram",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option_with_default("bins", "Int | sturges | scott | fd", "fd"),
                    ParamSpec::option("binwidth", "Float"),
                    ParamSpec::option("start", "Float"),
                    ParamSpec::option_with_default(
                        "scale",
                        "count | density | fraction | percent",
                        "count",
                    ),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "no x, a y, an x that is not numbers, bins below 1, a width not above 0, a start above the smallest value",
                )],
                help: include_str!("../help/histogram.md"),
                accuracy: None,
            },
            body: marks::histogram,
        },
        Function {
            spec: FunctionSpec {
                name: "density",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("bandwidth", "Float"),
                    ParamSpec::option_with_default("adjust", "Float", "1"),
                    ParamSpec::option_with_default(
                        "kernel",
                        "gaussian | epanechnikov | rectangular | triangular | biweight | cosine",
                        "gaussian",
                    ),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "no x, a y, an x that is not numbers, a bandwidth or an adjustment not above 0",
                )],
                help: include_str!("../help/density.md"),
                accuracy: None,
            },
            body: marks::density,
        },
        Function {
            spec: FunctionSpec {
                name: "boxplot",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option_with_default("outliers", "Bool", "true"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "no y, and a y that is not numbers",
                )],
                help: include_str!("../help/boxplot.md"),
                accuracy: None,
            },
            body: marks::boxplot,
        },
        Function {
            spec: FunctionSpec {
                name: "errorbars",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("data", "Frame"),
                    ParamSpec::option("low", "Var | Expr"),
                    ParamSpec::option("high", "Var | Expr"),
                    ParamSpec::option_with_default("width", "Float", "0.3"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x, low or high; the intervals whose low is above their high",
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
                        "the plot maps nothing to x; a variable the layer names for low: or high:, or a mapped variable of data:, that is not numeric, or an expression that gives nothing an axis places for each row; a y axis of categories or of dates; width: outside [0, 1]",
                    ),
                    DiagnosticDecl::new("Z0406", "low: or high: is not given"),
                ],
                help: include_str!("../help/errorbars.md"),
                accuracy: None,
            },
            body: marks::errorbars,
        },
        Function {
            spec: FunctionSpec {
                name: "line",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("x", "Var | Expr"),
                    ParamSpec::required("y", "Var | Expr"),
                    ParamSpec::option("color", "Var"),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the places where a line is broken because y is NA; the rows with no neighbour to be joined to; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable for x or y that no axis places; a colour by a number without value labels, where a line has one colour; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/line.md"),
                accuracy: None,
            },
            body: shorthands::line,
        },
        Function {
            spec: FunctionSpec {
                name: "bar",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Expr"),
                    ParamSpec::option("color", "Var"),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not counted for NA, or for an infinite value, in var; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that no axis places; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/bar.md"),
                accuracy: None,
            },
            body: shorthands::bar,
        },
        Function {
            spec: FunctionSpec {
                name: "scatter",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("x", "Var | Expr"),
                    ParamSpec::required("y", "Var | Expr"),
                    ParamSpec::option_with_default("fit", "none | linear", "none"),
                    ParamSpec::option("color", "Var"),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in x or y; the groups too small for a fitted line; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable for x or y that no axis places; fit: linear with categories along x or y, or dates along y; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/scatter.md"),
                accuracy: None,
            },
            body: shorthands::scatter,
        },
        Function {
            spec: FunctionSpec {
                name: "histogram",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Expr"),
                    ParamSpec::option_with_default("bins", "Int | sturges | scott | fd", "fd"),
                    ParamSpec::option("binwidth", "Float"),
                    ParamSpec::option("start", "Float"),
                    ParamSpec::option_with_default(
                        "scale",
                        "count | density | fraction | percent",
                        "count",
                    ),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not counted for NA, or for an infinite value, in var; the rows below start:; more bins than a histogram may have",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numbers, bins below 1, a width not above 0, a start above the smallest value; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/histogram_of_a_frame.md"),
                accuracy: None,
            },
            body: shorthands::histogram,
        },
        Function {
            spec: FunctionSpec {
                name: "density",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Expr"),
                    ParamSpec::option("color", "Var"),
                    ParamSpec::option("bandwidth", "Float"),
                    ParamSpec::option_with_default("adjust", "Float", "1"),
                    ParamSpec::option_with_default(
                        "kernel",
                        "gaussian | epanechnikov | rectangular | triangular | biweight | cosine",
                        "gaussian",
                    ),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not used for NA, or for an infinite value, in var; the groups with fewer than two values; more categories than the palette has colours",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numbers, a bandwidth or an adjustment not above 0; a colour by a number without value labels; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/density_of_a_frame.md"),
                accuracy: None,
            },
            body: shorthands::density,
        },
        Function {
            spec: FunctionSpec {
                name: "boxplot",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Expr"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not used for NA, or for an infinite value, in var or group",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numbers; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/boxplot_of_a_frame.md"),
                accuracy: None,
            },
            body: shorthands::boxplot,
        },
        Function {
            spec: FunctionSpec {
                name: "qq_plot",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Expr"),
                    ParamSpec::option("distribution", "Distribution"),
                    ParamSpec::option("by", "Var | Vars"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows not drawn for NA, or for an infinite value, in var; the groups with fewer than two values, which have no reference line",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in an expression is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "no such variable in the frame, mapped, read by an expression or named by by:",
                    ),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numbers; a frame that already has a variable called theoretical; a variable named twice in by:",
                    ),
                ],
                help: include_str!("../help/qq_plot.md"),
                accuracy: None,
            },
            body: shorthands::qq_plot,
        },
        Function {
            spec: FunctionSpec {
                name: "coord",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option_with_default(
                        "type",
                        "cartesian | flip | polar | fixed",
                        "cartesian",
                    ),
                    ParamSpec::option("ratio", "Float"),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "a ratio that is not above 0 or goes without fixed, a rug in polar coordinates, and fixed or polar coordinates for panels with free scales",
                )],
                help: include_str!("../help/coord.md"),
                accuracy: None,
            },
            body: scales::coord,
        },
        Function {
            spec: FunctionSpec {
                name: "facet",
                package: "plot",
                method: true,
                params: vec![
                    ParamSpec::required("p", "Plot"),
                    ParamSpec::option("by", "Var | Vars"),
                    ParamSpec::option("rows", "Var"),
                    ParamSpec::option("cols", "Var"),
                    ParamSpec::option_with_default(
                        "scales",
                        "fixed | free | free_x | free_y",
                        "fixed",
                    ),
                ],
                rest: None,
                returns: "Plot",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable named twice, by: with rows: or cols:, one variable for both, and free scales in fixed or polar coordinates",
                    ),
                    DiagnosticDecl::new("Z0406", "no by:, rows: or cols:"),
                ],
                help: include_str!("../help/facet.md"),
                accuracy: None,
            },
            body: scales::facet,
        },
    ]
}
