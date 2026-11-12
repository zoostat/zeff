//! The scene of a plot (Language Reference, section 7.6, "Rendering and export"; Engine and
//! Project Specification, section 8.3, point 5): the marks in data coordinates with their
//! scales, guides and layout, in a versioned format.
//!
//! [`scene`] derives the scene from a plot value and is a pure function of it: the same plot
//! gives the same scene, bit for bit, on every target. The writers of SVG, PDF and PNG and Zemp
//! read the scene alone, never the plot value, so everything that decides what is drawn is
//! settled here: which rows are drawn and which are dropped and counted, where the jitter moves
//! each point, the limits and breaks of each axis and the title it carries, the categories of a
//! colour scale, their order and the colour of each, the size of each mark and its shape.
//!
//! A scene has no size. It is drawn at the size of the pane or of the exported figure, and the
//! theme of the writer gives it its fonts, its margins and the colour of a mark that has none of
//! its own.
//!
//! Version 1 holds one panel, the scales `x` and `y` with their axes, the colour scales `color`
//! and `fill` and the scales `size` and `shape` with the legends that show them, and layers of
//! points.
//!
//! **Colour.** A mark's colour is kept as the positions are, in the units of the data, and the
//! scale `color` says what each value is drawn as. On a scale of kind `category` the value is
//! the place of the category, counted from 1; the breaks are the places, the labels the
//! categories as written, and `range` holds one colour for each break. On a scale of kind
//! `linear` the value is the value of the variable and `range` is a ramp of colours laid evenly
//! from the lower limit to the upper: with `n` colours a value `v` takes entry
//! `floor(((v/2 − lo/2) / (hi/2 − lo/2)) × n)`, counted from 0, and the upper limit itself the
//! last ([`crate::colour::ramp_entry`]). The `color` of a layer is `null` for the colour of the
//! theme, one colour for a constant, or a list with a value on the scale `color` for each mark.
//! A colour is `#` and six hexadecimal digits in capitals.
//!
//! **Size.** The `size` of a layer is one number, the diameter of every mark in points of 1/72
//! inch, or a list with the value of the mapped variable for each mark, which the scale `size`
//! turns into a diameter. That scale is of kind `sqrt`: its limits are 0 and the largest value
//! drawn, its `range` the diameters at the two limits, and a value `v` is drawn
//! `range[0] + (range[1] − range[0]) × √((v − lo) / (hi − lo))` across ([`diameter`]), so that
//! the area of a mark is in proportion to its value and a value of 0 has no size.
//!
//! **Shape.** The `shape` of a layer is one name ([`crate::shape`]) or a list with the place of
//! each mark's category on the scale `shape`, which is of kind `category` and whose `range`
//! holds one name for each break.
//!
//! **Legends.** A legend is the guide of every scale that shows one variable: with `color` and
//! `shape` mapped to one variable there is one legend, whose keys carry both.
//!
//! **Where a mark came from** (open decision #118, F2). Each layer names the frame it draws,
//! `data`, by the binding the script gave it, and holds in `rows` the row of that frame behind
//! each mark, counted from 1, so that a viewer can show the row under the pointer and select
//! the rows of the marks brushed. They belong to the viewer's scene alone: a drawing made from
//! a scene ([`crate::drawing`], from step 21b) has neither, and so no exported figure carries a
//! row of the data.
//!
//! **Reserved for later** (F3 and F4). Version 1 keeps four names for what later releases
//! draw: `frame` on the scene, the place of a scene in an animation; `z` on a layer, a third
//! coordinate, with `projection` on its panel; and `crs` on a panel, the coordinate system of a
//! map. No scene of this release holds one, and their JSON is written only when present. A
//! reader that meets one it does not draw must refuse the scene and say so, never draw it
//! without ([`Scene::undrawn`]). A layer also has the flag `raster`, false in this release:
//! true asks a writer of vector figures to draw that one layer as a grid of pixels.

use std::borrow::Cow;

use zeff_columns::{Column, ColumnType, MetadataKey, Value as Cell};
use zeff_core::display::{format_float, labelled_cell};
use zeff_core::value::{
    Aesthetic, Align, Arrangement, Frame, Mark, Plot, PlotConstant, PlotLayer, PlotMapping,
    PlotSetting, Sides, Stacking, Stairs, Table,
};
use zeff_diagnostics::count;

use crate::colour;
use crate::drawing::LineType;
use crate::json::Json;
use crate::scale;
use crate::shape;

/// The version of the scene format, the value of `zeff_scene` in its JSON. It rises when a
/// reader of the earlier version would misread the new one.
pub const SCENE_VERSION: u32 = 1;

/// The diameter of a point when `size:` is not given, in points of 1/72 inch: the default of
/// Observable Plot, a radius of 3 pixels at 96 to the inch.
pub const POINT_SIZE: f64 = 4.5;

/// The diameter of the mark of the largest value on a scale of sizes, in points of 1/72 inch:
/// four times the default, the share ggplot2 has between its largest size, 6, and its default,
/// 1.5.
pub const LARGEST_SIZE: f64 = 18.0;

/// The width of a line when its layer says nothing, in points of 1/72 inch.
pub const LINE_WIDTH: f64 = 1.0;

/// The width of a bar as a share of the gap between two values of `x`, where the layer says
/// nothing.
pub const BAR_WIDTH: f64 = 0.9;

/// The length of a tick of a rug, in points of 1/72 inch: the diameter of a point.
pub const RUG_LENGTH: f64 = 4.5;

/// The diameter the scale of sizes with these limits and this range gives the value `v`: the
/// share of the way from the lower limit to the upper, its square root taken, laid on the range.
/// A value outside the limits takes the diameter of the nearer one, and where the limits are
/// one value, or the share cannot be computed, the lower end of the range.
pub fn diameter(v: f64, limits: (f64, f64), range: (f64, f64)) -> f64 {
    let (lo, hi) = limits;
    let share = (v - lo) / (hi - lo);
    if !(hi > lo && share.is_finite()) {
        return range.0;
    }
    range.0 + (range.1 - range.0) * share.clamp(0.0, 1.0).sqrt()
}

/// A plot as it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// The grid of panels: its rows and columns.
    pub rows: usize,
    pub columns: usize,
    pub scales: Vec<Scale>,
    pub axes: Vec<Axis>,
    /// What each legend shows: the content only, since where a legend stands is the business of
    /// `legend` and of the writer.
    pub legends: Vec<Legend>,
    pub panels: Vec<Panel>,
    /// What the reader must be told about the data drawn: the rows dropped and why, each
    /// sentence once.
    pub notes: Vec<String>,
    /// Reserved: the place of this scene in an animation. `None` in this release.
    pub frame: Option<usize>,
}

/// A scale: how the values of an aesthetic are placed, coloured, sized or shaped, with the
/// breaks a guide marks.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    /// The name the panels and guides refer to it by: `x`, `y`, `color`, `fill`, `size`, `shape`.
    pub id: &'static str,
    pub kind: ScaleKind,
    /// The title of its guide: the label of the variable, or its name; an expression as written.
    pub title: String,
    /// The two ends of the scale, in data coordinates: on a scale of categories, 1 and the
    /// number of categories.
    pub limits: (f64, f64),
    pub breaks: Vec<f64>,
    /// The text at each break.
    pub labels: Vec<String>,
    /// What the values are drawn as.
    pub range: Range,
}

/// The kinds of scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleKind {
    /// Numbers, placed or coloured in proportion to their value.
    Linear,
    /// Categories, each at its place in their order, counted from 1.
    Category,
    /// Numbers of 0 or more, drawn in proportion to the square root of their value: the scale
    /// of sizes, on which the area of a mark is in proportion to the value.
    Sqrt,
}

impl ScaleKind {
    pub fn name(self) -> &'static str {
        match self {
            ScaleKind::Linear => "linear",
            ScaleKind::Category => "category",
            ScaleKind::Sqrt => "sqrt",
        }
    }
}

/// What a scale draws its values as.
#[derive(Debug, Clone, PartialEq)]
pub enum Range {
    /// A position, along the side of the panel that the writer gives the scale.
    Position,
    /// Colours: on a scale of categories one for each break, on a linear scale a ramp laid
    /// evenly from the lower limit to the upper.
    Colours(Vec<&'static str>),
    /// Sizes: the diameter of a mark at the lower limit and at the upper, in points of 1/72
    /// inch.
    Sizes(f64, f64),
    /// Shapes, by name: one for each break.
    Shapes(Vec<&'static str>),
    /// Types of line, by name: one for each break.
    LineTypes(Vec<&'static str>),
}

/// What a legend shows: its title, the scales it is the guide of, which are those of one
/// variable, and the marks drawn by them, which give the legend its keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legend {
    pub title: String,
    pub scales: Vec<&'static str>,
    pub marks: Vec<Mark>,
}

/// An axis: the guide of a position scale and the side of the panel it is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    pub scale: &'static str,
    pub position: Position,
}

/// The side of a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Bottom,
    Left,
}

impl Position {
    pub fn name(self) -> &'static str {
        match self {
            Position::Bottom => "bottom",
            Position::Left => "left",
        }
    }
}

/// One panel: its place in the grid, counted from 1, the scales its marks are placed by and its
/// layers from the bottom up.
#[derive(Debug, Clone, PartialEq)]
pub struct Panel {
    pub row: usize,
    pub column: usize,
    pub x: Option<&'static str>,
    pub y: Option<&'static str>,
    pub layers: Vec<Layer>,
    /// Reserved: how a third coordinate is brought to the page. `None` in this release.
    pub projection: Option<String>,
    /// Reserved: the coordinate system of a map. `None` in this release.
    pub crs: Option<String>,
}

/// One layer of marks, in data coordinates: mark `i` is at (`x[i]`, `y[i]`).
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub mark: Mark,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: Paint,
    pub size: Sizing,
    pub shape: Shaping,
    /// The opacity, from 0 to 1.
    pub alpha: f64,
    /// The frame the layer draws, by the binding the script gave it; `None` for a frame
    /// written in place.
    pub data: Option<String>,
    /// The row of that frame behind each mark, counted from 1: mark `i` is row `rows[i]`.
    pub rows: Vec<usize>,
    /// Reserved: a third coordinate for each mark. `None` in this release.
    pub z: Option<Vec<f64>>,
    /// Whether a writer of vector figures draws this layer as a grid of pixels. False in this
    /// release.
    pub raster: bool,
    /// For a mark that is a line, or that is filled under one: the line each mark belongs to,
    /// counted from 1. The marks of one line follow one another in the order they are joined,
    /// and a line has two marks or more. Empty for the marks that are drawn one by one.
    pub line: Vec<usize>,
    /// The type of the lines of the layer.
    pub linetype: Lining,
    /// The width of a line of the layer, in points of 1/72 inch.
    pub width: f64,
    /// For a mark that is filled: the lower edge at each mark, in data coordinates, the upper
    /// edge being `y`. Zero at every mark of an area; the lower variable of a band. Empty for
    /// the other marks.
    pub low: Vec<f64>,
    /// The colour a filled mark is filled with.
    pub fill: Paint,
    /// For a mark that is a bar: how far each reaches along `x`, in data coordinates, half to
    /// either side of its `x`. Empty for the other marks.
    pub span: Vec<f64>,
}

/// The type of the lines of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Lining {
    /// One type for every line, by its name: `solid`.
    Constant(&'static str),
    /// The place of each mark's category on the scale `linetype`, counted from 1.
    Scaled(Vec<f64>),
}

/// The colour of the marks of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    /// The colour the theme gives a mark.
    Theme,
    /// One colour for every mark, `#4682B4`.
    Constant(String),
    /// A value on the scale `color` for each mark, in data coordinates.
    Scaled(Vec<f64>),
}

/// The size of the marks of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Sizing {
    /// One diameter for every mark, in points of 1/72 inch.
    Constant(f64),
    /// A value on the scale `size` for each mark, in data coordinates.
    Scaled(Vec<f64>),
}

/// The shape of the marks of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Shaping {
    /// One shape for every mark, by its name.
    Constant(&'static str),
    /// The place of each mark's category on the scale `shape`, counted from 1.
    Scaled(Vec<f64>),
}

/// Whether a mark reads an aesthetic. A point is placed by `x` and `y`, coloured by `color`,
/// sized by `size` and shaped by `shape`. `fill` is the colour of the inside of the marks that
/// have one, `group` gathers the rows of the marks drawn through several, and `label` is the
/// text of the marks that write one, which come at a later step.
pub fn reads(mark: Mark, aesthetic: Aesthetic) -> bool {
    match mark {
        Mark::Points => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Y | Aesthetic::Color | Aesthetic::Size | Aesthetic::Shape
        ),
        // A line is placed by `x` and `y`, coloured by `color`, of the type `linetype` says,
        // and drawn through the rows of one `group`.
        Mark::Lines | Mark::Step(_) => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Y | Aesthetic::Color | Aesthetic::Group | Aesthetic::Linetype
        ),
        // An area is placed by `x` and `y`, filled in the colour `fill` says, and drawn through
        // the rows of one `group`.
        Mark::Area(_) => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Y | Aesthetic::Fill | Aesthetic::Group
        ),
        // A bar stands at `x`, as high as `y`, filled in the colour `fill` says; the rows of
        // one `group` are one set of bars.
        Mark::Bars(_) => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Y | Aesthetic::Fill | Aesthetic::Group
        ),
        // A rug has a tick for each row at its place on the axes of its sides, in its colour.
        Mark::Rug(sides) => match aesthetic {
            Aesthetic::X => sides != Sides::Y,
            Aesthetic::Y => sides != Sides::X,
            Aesthetic::Color => true,
            _ => false,
        },
        // A reference line stands at a place on one axis, in a colour and a type of its own.
        Mark::Hline => matches!(
            aesthetic,
            Aesthetic::Y | Aesthetic::Color | Aesthetic::Linetype
        ),
        Mark::Vline => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Color | Aesthetic::Linetype
        ),
        // A band is an area between two edges, `low` and `high`, in the place of `y`.
        Mark::Ribbon => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Low | Aesthetic::High | Aesthetic::Fill | Aesthetic::Group
        ),
    }
}

/// Whether a layer draws an aesthetic from a variable, its own or the plot's: its mark reads
/// it, and the layer does not set it to one value. A layer with a `size:` of its own that is a
/// number does not read a size.
pub fn layer_reads(layer: &PlotLayer, aesthetic: Aesthetic) -> bool {
    reads(layer.mark, aesthetic) && !layer.sets(aesthetic)
}

/// Whether a layer takes an aesthetic from the plot: it draws it from a variable, and names
/// none of its own.
fn takes_from_plot(layer: &PlotLayer, aesthetic: Aesthetic) -> bool {
    layer_reads(layer, aesthetic) && layer.mapping(aesthetic).is_none() && !layer.mark.spans()
}

/// What a layer draws an aesthetic from (open decision #106): its own mapping where it names
/// one, nothing where it sets the aesthetic to one value, and the plot's mapping otherwise.
pub fn mapping_of<'a>(
    plot: &'a Plot,
    layer: &'a PlotLayer,
    aesthetic: Aesthetic,
) -> Option<&'a PlotMapping> {
    match layer.mapping(aesthetic) {
        Some(own) => Some(own),
        None if layer.sets(aesthetic) => None,
        // A reference line takes nothing from the plot but its axes.
        None if layer.mark.spans() => None,
        None => plot.mapping(aesthetic),
    }
}

/// How the scale of an aesthetic shows its variable so far, as categories or on a ramp, with
/// the variable it is named by: the plot's, or the first a layer names. `None` while nothing
/// is mapped to it.
pub(crate) fn kind_of(plot: &Plot, aesthetic: Aesthetic) -> Option<(Shading, &str)> {
    let (mapping, _, layer) = first_mapping(plot, aesthetic)?;
    let kind = shading(values(plot, layer, aesthetic)?)?;
    Some((kind, mapping.variable.as_str()))
}

/// The frame a layer draws: its own, or the plot's.
fn frame_of<'a>(plot: &'a Plot, layer: &'a PlotLayer) -> &'a Frame {
    layer.data.as_ref().unwrap_or(&plot.frame)
}

/// What an aesthetic is mapped from for the whole plot, with the frame it is read in: the
/// plot's mapping, or where the plot maps nothing the first a layer names of its own.
fn first_mapping(
    plot: &Plot,
    aesthetic: Aesthetic,
) -> Option<(&PlotMapping, &Frame, Option<&PlotLayer>)> {
    if let Some(mapping) = plot.mapping(aesthetic) {
        return Some((mapping, &plot.frame, None));
    }
    // The edges of a band are positions along `y`, and stand for it where nothing is mapped
    // to `y` itself: the lower edge first.
    let stands: &[Aesthetic] = if aesthetic == Aesthetic::Y {
        &[Aesthetic::Y, Aesthetic::Low, Aesthetic::High]
    } else {
        std::slice::from_ref(&aesthetic)
    };
    plot.layers.iter().find_map(|layer| {
        stands.iter().find_map(|each| {
            let own = layer.mapping(*each)?;
            reads(layer.mark, *each).then(|| (own, frame_of(plot, layer), Some(layer)))
        })
    })
}

/// How a variable mapped to a colour, a size or a shape is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shading {
    /// Each category in a colour, or a shape, of its own.
    Categories,
    /// Each value in the colour of its place between the smallest and the largest, or in the
    /// size of its amount.
    Ramp,
}

/// How a variable is shown when it is mapped to a colour: a Category, a Str or a Bool as
/// categories, and so is a variable with value labels, whose values are names; any other Int or
/// Float on a ramp. `None` for the types that have no colour scale yet. A shape is given to
/// what is shown as categories and a size to what is shown on a ramp, and to nothing else.
pub fn shading(column: &Column) -> Option<Shading> {
    match column.ty() {
        ColumnType::Category | ColumnType::Str | ColumnType::Bool => Some(Shading::Categories),
        ColumnType::Int | ColumnType::Float => {
            let labelled = column
                .metadata()
                .value_labels()
                .is_some_and(|labels| !labels.is_empty());
            Some(if labelled {
                Shading::Categories
            } else {
                Shading::Ramp
            })
        }
        _ => None,
    }
}

/// A variable read as categories: its categories in their order, each by its label, and for
/// each row which of them it is, `None` where the row is `NA`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Categories {
    labels: Vec<String>,
    of: Vec<Option<usize>>,
}

/// The categories of a variable. Those of a Category are its levels, in their order. Those of
/// any other variable are the values it holds, in the order of the language (section 3.7:
/// `false` before `true`, numbers by size, text by code point), each named by its value label
/// where it has one and as it prints where it has none; values with one label are one category.
fn categories(column: &Column) -> Categories {
    if let (Some(levels), Some(codes)) = (column.levels(), column.as_codes()) {
        return Categories {
            labels: levels.iter().map(|level| (*level).to_string()).collect(),
            of: codes
                .map(|code| code.and_then(|c| usize::try_from(c).ok()))
                .collect(),
        };
    }
    // A missing cell, and a Float that is not a number, has no category.
    let present = |row: usize| match column.get(row) {
        None | Some(Cell::Na) => None,
        Some(Cell::Float(x)) if x.is_nan() => None,
        Some(cell) => Some(cell),
    };
    let order = |a: &Cell<'_>, b: &Cell<'_>| match (a, b) {
        (Cell::Bool(a), Cell::Bool(b)) => a.cmp(b),
        (Cell::Int(a), Cell::Int(b)) => a.cmp(b),
        // Adding zero makes the two zeros one value, as identity has them (section 3.7).
        (Cell::Float(a), Cell::Float(b)) => (a + 0.0).total_cmp(&(b + 0.0)),
        (Cell::Str(a), Cell::Str(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    };
    let mut rows: Vec<(usize, Cell<'_>)> = (0..column.len())
        .filter_map(|row| present(row).map(|cell| (row, cell)))
        .collect();
    rows.sort_by(|a, b| order(&a.1, &b.1));
    let mut out = Categories {
        labels: Vec::new(),
        of: vec![None; column.len()],
    };
    let mut last: Option<(&Cell<'_>, usize)> = None;
    for (row, cell) in &rows {
        let place = match last {
            Some((previous, place)) if order(previous, cell).is_eq() => place,
            _ => {
                let label = labelled_cell(column, *row);
                match out.labels.iter().position(|known| *known == label) {
                    Some(place) => place,
                    None => {
                        out.labels.push(label);
                        out.labels.len() - 1
                    }
                }
            }
        };
        out.of[*row] = Some(place);
        last = Some((cell, place));
    }
    out
}

/// Whether a variable can be placed on a linear scale: a Float, an Int, or a Bool, which reads
/// as 0 and 1 (section 3.6).
pub fn is_numeric(column: &Column) -> bool {
    matches!(
        column.ty(),
        ColumnType::Bool | ColumnType::Int | ColumnType::Float
    )
}

/// Whether a variable makes an axis of categories when it is mapped to a position: a Str or
/// a Category (open decision #128). A number is placed on a linear scale, with value labels
/// or without.
pub fn on_categories(column: &Column) -> bool {
    matches!(column.ty(), ColumnType::Category | ColumnType::Str)
}

/// How an axis places its variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placing {
    /// On a linear scale.
    Linear,
    /// Each category at a place of its own, 1, 2, 3 in their order.
    Categories,
}

/// How a variable is placed on an axis, `None` for the types no axis holds yet.
pub fn placing(column: &Column) -> Option<Placing> {
    if is_numeric(column) {
        Some(Placing::Linear)
    } else if on_categories(column) {
        Some(Placing::Categories)
    } else {
        None
    }
}

/// How the axis of `aesthetic`, `x` or `y`, places its variable so far, with the variable it
/// is named by: the plot's, or the first a layer names. `None` while nothing is mapped to it.
pub(crate) fn placing_of(plot: &Plot, aesthetic: Aesthetic) -> Option<(Placing, &str)> {
    let (mapping, _, layer) = first_mapping(plot, aesthetic)?;
    // The edges of a band, and an expression, are numbers.
    if mapping.aesthetic != aesthetic || mapping.computed.is_some() {
        return Some((Placing::Linear, mapping.variable.as_str()));
    }
    let kind = placing(values(plot, layer, aesthetic)?)?;
    Some((kind, mapping.variable.as_str()))
}

/// The categories of an axis of categories, in their order: those of the variable the axis
/// is named by, then those that only a layer's own variable has. `None` for a linear axis and
/// for one nothing is mapped to.
fn axis_categories(plot: &Plot, aesthetic: Aesthetic) -> Option<Vec<String>> {
    if placing_of(plot, aesthetic)?.0 != Placing::Categories {
        return None;
    }
    let (_, _, first) = first_mapping(plot, aesthetic)?;
    let mut labels = categories(values(plot, first, aesthetic)?).labels;
    for layer in &plot.layers {
        if mapping_of(plot, layer, aesthetic).is_none() || !reads(layer.mark, aesthetic) {
            continue;
        }
        let Some(column) = values(plot, Some(layer), aesthetic).filter(|c| on_categories(c)) else {
            continue;
        };
        for label in categories(column).labels {
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    Some(labels)
}

/// The values a layer is placed by on an axis: the variable itself on a linear axis, and on
/// an axis of categories the place of each row's category, counted from 1, `NA` where the row
/// has none. A variable that is no categories gives nothing on such an axis; the marks refuse
/// it before it comes to this.
fn placed<'a>(column: Option<&'a Column>, on: Option<&[String]>) -> Option<Cow<'a, Column>> {
    let column = column?;
    let Some(labels) = on else {
        return Some(Cow::Borrowed(column));
    };
    if !on_categories(column) {
        return None;
    }
    let own = categories(column);
    let at: Vec<Option<usize>> = own
        .labels
        .iter()
        .map(|label| labels.iter().position(|known| known == label))
        .collect();
    let places = own.of.iter().map(|place| {
        place
            .and_then(|place| at.get(place).copied().flatten())
            .map(|place| (place + 1) as f64)
    });
    Some(Cow::Owned(Column::floats(places).0))
}

/// A cell of a numeric column as a number, `None` where it is `NA`.
fn number(column: &Column, row: usize) -> Option<f64> {
    match column.get(row) {
        Some(Cell::Float(x)) if !x.is_nan() => Some(x),
        Some(Cell::Int(i)) => Some(i as f64),
        Some(Cell::Bool(b)) => Some(f64::from(u8::from(b))),
        _ => None,
    }
}

/// The rows of a layer that are drawn, and the counts of those that are not.
#[derive(Default)]
struct Drawn {
    x: Vec<f64>,
    y: Vec<f64>,
    /// The row of the layer's frame each mark is drawn from.
    rows: Vec<usize>,
    missing: usize,
    infinite: usize,
    /// The rows with a size below 0.
    negative: usize,
}

/// The values a layer draws for an aesthetic: the variable of the layer's frame, which is the
/// plot's unless the layer has its own, or the values of the mapped expression computed on that
/// frame. A layer that names a mapping of its own draws that: its variable in its frame, or its
/// expression as computed there when the layer was added. `None` when nothing is mapped, or
/// when the frame has no such variable.
fn values<'a>(
    plot: &'a Plot,
    layer: Option<&'a PlotLayer>,
    aesthetic: Aesthetic,
) -> Option<&'a Column> {
    if let Some(layer) = layer
        && let Some(own) = layer.mapping(aesthetic)
    {
        return match &own.computed {
            Some(computed) => Some(&computed.values),
            None => frame_of(plot, layer).table.column(&own.variable),
        };
    }
    if layer.is_some_and(|layer| layer.mark.spans()) {
        return None;
    }
    let mapping = plot.mapping(aesthetic)?;
    let own = layer.filter(|layer| layer.data.is_some());
    match (&mapping.computed, own) {
        (None, _) => {
            let frame = own
                .and_then(|layer| layer.data.as_ref())
                .unwrap_or(&plot.frame);
            frame.table.column(&mapping.variable)
        }
        (Some(_), Some(layer)) => layer
            .computed
            .iter()
            .find(|(of, _)| *of == aesthetic)
            .map(|(_, column)| &**column),
        (Some(computed), None) => Some(&computed.values),
    }
}

/// The points of a layer: one for each row with a finite value of both positions, of each
/// variable in `ramps`, those the layer shows on a ramp of colours, and of `size`, the variable
/// it is sized by, which must not be below 0 either; in the order of the rows. A row with `NA`
/// in any of them is counted as missing; a row with an infinite value, which no scale can hold,
/// is counted apart, and so is a row with a size below 0, which has no area to be drawn with. A
/// position that is not there, or is not numeric, gives no point; the functions that build a
/// plot refuse such a frame before it comes to this.
fn drawn(
    cx: Option<&Column>,
    cy: Option<&Column>,
    ramps: &[&Column],
    size: Option<&Column>,
) -> Drawn {
    let mut out = Drawn::default();
    let (Some(cx), Some(cy)) = (cx, cy) else {
        return out;
    };
    if !(is_numeric(cx) && is_numeric(cy)) || cx.len() != cy.len() {
        return out;
    }
    if ramps.iter().chain(size.iter()).any(|c| c.len() != cx.len()) {
        return out;
    }
    for row in 0..cx.len() {
        let mut missing = false;
        let mut infinite = false;
        let mut read = |column: &Column| match number(column, row) {
            Some(v) => {
                infinite |= !v.is_finite();
                v
            }
            None => {
                missing = true;
                0.0
            }
        };
        let (a, b) = (read(cx), read(cy));
        for ramp in ramps {
            read(ramp);
        }
        let amount = size.map(&mut read);
        if missing {
            out.missing += 1;
        } else if infinite {
            out.infinite += 1;
        } else if amount.is_some_and(|amount| amount < 0.0) {
            out.negative += 1;
        } else {
            out.x.push(a);
            out.y.push(b);
            out.rows.push(row);
        }
    }
    out
}

/// The rows of a layer of lines, joined: the marks in the order they are joined, the line of
/// each, and what was not drawn.
#[derive(Default)]
struct Joined {
    points: Drawn,
    /// The line of each mark, counted from 1.
    line: Vec<usize>,
    /// The lower edge at each mark, for the marks that are filled: the second variable, or
    /// zero where there is none.
    low: Vec<f64>,
    /// The places where a line is broken because a row between two that are drawn has no `y`.
    breaks: usize,
    /// Whether a row was left out for an infinite `y`, which breaks a line as `NA` does.
    infinite_y: bool,
    /// The rows that have no neighbour to be joined to.
    alone: usize,
    /// For stacked areas: the rows left out because, at their `x`, not every area has one
    /// value, and at how many values of `x`.
    left: (usize, usize),
    /// For areas stacked as shares: the rows left out because, at their `x`, a value is below
    /// zero or the total is not above it, and at how many values of `x`.
    unfit: (usize, usize),
}

/// The lines of a layer. The rows are parted into groups by the categories of `keys`, row by
/// row; the rows of a group are put in the order of `x`, those of one `x` in the order of the
/// frame; and each row is joined to the next. A row with no `x`, or an infinite one, has no
/// place in that order and is left out. A row with no `y`, or an infinite one, is left out
/// too, and the line is broken there: the rows before it and after it are not joined to each
/// other. A row left with no neighbour draws nothing and is counted. A band has a second
/// variable along `y`, its lower edge (`second`), and a row needs both to be drawn.
fn joined(
    cx: Option<&Column>,
    cy: Option<&Column>,
    second: Option<&Column>,
    keys: &[Categories],
) -> Joined {
    let mut out = Joined::default();
    let (Some(cx), Some(cy)) = (cx, cy) else {
        return out;
    };
    if !(is_numeric(cx) && is_numeric(cy)) || cx.len() != cy.len() {
        return out;
    }
    if second.is_some_and(|low| !is_numeric(low) || low.len() != cx.len()) {
        return out;
    }
    if keys.iter().any(|key| key.of.len() != cx.len()) {
        return out;
    }
    // The groups in the order their first row comes, each with its rows: `x`, `y` and the
    // lower edge where they can be drawn, and the row.
    type Row = (f64, Option<(f64, f64)>, usize);
    let mut groups: Vec<(Vec<Option<usize>>, Vec<Row>)> = Vec::new();
    for row in 0..cx.len() {
        let x = match number(cx, row) {
            None => {
                out.points.missing += 1;
                continue;
            }
            Some(x) if !x.is_finite() => {
                out.points.infinite += 1;
                continue;
            }
            Some(x) => x,
        };
        // The upper edge and the lower one: zero where the mark has no second variable.
        let low = match second {
            Some(low) => number(low, row),
            None => Some(0.0),
        };
        let y = match (number(cy, row), low) {
            (None, _) | (_, None) => {
                out.points.missing += 1;
                None
            }
            (Some(y), Some(low)) if !(y.is_finite() && low.is_finite()) => {
                out.points.infinite += 1;
                out.infinite_y = true;
                None
            }
            (Some(y), Some(low)) => Some((y, low)),
        };
        let key: Vec<Option<usize>> = keys.iter().map(|key| key.of[row]).collect();
        match groups.iter_mut().find(|(known, _)| *known == key) {
            Some((_, rows)) => rows.push((x, y, row)),
            None => groups.push((key, vec![(x, y, row)])),
        }
    }
    let mut lines = 0;
    for (_, mut rows) in groups {
        // The sort keeps rows of one `x` in the order of the frame.
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut run: Vec<(f64, (f64, f64), usize)> = Vec::new();
        let mut finish = |run: &mut Vec<(f64, (f64, f64), usize)>, out: &mut Joined| {
            match run.len() {
                0 => {}
                1 => out.alone += 1,
                _ => {
                    lines += 1;
                    for &(x, (y, low), row) in run.iter() {
                        out.points.x.push(x);
                        out.points.y.push(y);
                        out.points.rows.push(row);
                        out.line.push(lines);
                        out.low.push(low);
                    }
                }
            }
            run.clear();
        };
        // A break is a row without `y` that has rows with one on both sides of it.
        let (mut before, mut gap) = (false, false);
        for (x, y, row) in rows {
            match y {
                Some(y) => {
                    if gap && before {
                        out.breaks += 1;
                    }
                    gap = false;
                    before = true;
                    run.push((x, y, row));
                }
                None => {
                    gap = true;
                    finish(&mut run, &mut out);
                }
            }
        }
        finish(&mut run, &mut out);
    }
    out
}

/// The areas of a layer, stacked. The rows are parted into groups as for a line, and the groups
/// put in the order of their categories, `NA` last. At each `x` the areas are put on one
/// another from the last group up, so that the first is on top as it is first in a legend; a
/// value below zero is stacked downwards from zero, apart from those above. With `share`, each
/// height is divided by the total at its `x`.
///
/// Stacking needs one value of each area at an `x`. An `x` at which a group has no row, a row
/// without `y`, or more than one row is left out for every group, and the areas are broken
/// there; with `share`, so is an `x` with a value below zero or a total that is not above it,
/// where there are no shares to show. A row left with no neighbour draws nothing.
fn stacked(cx: Option<&Column>, cy: Option<&Column>, keys: &[Categories], share: bool) -> Joined {
    let mut out = Joined::default();
    let (Some(cx), Some(cy)) = (cx, cy) else {
        return out;
    };
    if !(is_numeric(cx) && is_numeric(cy)) || cx.len() != cy.len() {
        return out;
    }
    if keys.iter().any(|key| key.of.len() != cx.len()) {
        return out;
    }
    // The groups, each with its rows: `x`, `y` where it can be drawn, and the row.
    type Row = (f64, Option<f64>, usize);
    let mut groups: Vec<(Vec<Option<usize>>, Vec<Row>)> = Vec::new();
    for row in 0..cx.len() {
        let x = match number(cx, row) {
            None => {
                out.points.missing += 1;
                continue;
            }
            Some(x) if !x.is_finite() => {
                out.points.infinite += 1;
                continue;
            }
            // Adding zero makes the two zeros one place.
            Some(x) => x + 0.0,
        };
        let y = match number(cy, row) {
            None => {
                out.points.missing += 1;
                None
            }
            Some(y) if !y.is_finite() => {
                out.points.infinite += 1;
                out.infinite_y = true;
                None
            }
            Some(y) => Some(y),
        };
        let key: Vec<Option<usize>> = keys.iter().map(|key| key.of[row]).collect();
        match groups.iter_mut().find(|(known, _)| *known == key) {
            Some((_, rows)) => rows.push((x, y, row)),
            None => groups.push((key, vec![(x, y, row)])),
        }
    }
    // In the order of the categories, a row without one after the others.
    let placed = |key: &[Option<usize>]| -> Vec<(bool, usize)> {
        key.iter()
            .map(|place| (place.is_none(), place.unwrap_or(0)))
            .collect()
    };
    groups.sort_by_key(|group| placed(&group.0));
    // Every `x` at which a row stands, in order.
    let mut places: Vec<f64> = groups
        .iter()
        .flat_map(|(_, rows)| rows.iter().map(|row| row.0))
        .collect();
    places.sort_by(f64::total_cmp);
    places.dedup();
    // For each group, what it has at each place: nothing, one value, or what cannot be
    // stacked (more than one row, or a row without `y`), with the rows that have a value.
    #[derive(Clone, Copy, PartialEq)]
    enum Cell {
        Empty,
        One(f64, usize),
        Not,
    }
    let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(groups.len());
    let mut valid = vec![0_usize; places.len()];
    for (_, rows) in &groups {
        let mut of_group = vec![Cell::Empty; places.len()];
        for &(x, y, row) in rows {
            let Ok(at) = places.binary_search_by(|place| place.total_cmp(&x)) else {
                continue;
            };
            if y.is_some() {
                valid[at] += 1;
            }
            of_group[at] = match (of_group[at], y) {
                (Cell::Empty, Some(y)) => Cell::One(y, row),
                _ => Cell::Not,
            };
        }
        cells.push(of_group);
    }
    // The places that are stacked.
    let mut whole = vec![false; places.len()];
    for at in 0..places.len() {
        let values: Option<Vec<f64>> = cells
            .iter()
            .map(|of_group| match of_group[at] {
                Cell::One(y, _) => Some(y),
                _ => None,
            })
            .collect();
        let Some(values) = values else {
            if valid[at] > 0 {
                out.left.0 += valid[at];
                out.left.1 += 1;
            }
            continue;
        };
        if share && (values.iter().any(|y| *y < 0.0) || values.iter().sum::<f64>() <= 0.0) {
            out.unfit.0 += valid[at];
            out.unfit.1 += 1;
            continue;
        }
        whole[at] = true;
    }
    // The two edges of each area at each stacked place: from the last group up.
    let mut edges: Vec<Vec<(f64, f64)>> = vec![vec![(0.0, 0.0); places.len()]; groups.len()];
    for at in (0..places.len()).filter(|at| whole[*at]) {
        let (mut above, mut below) = (0.0_f64, 0.0_f64);
        for group in (0..groups.len()).rev() {
            let Cell::One(y, _) = cells[group][at] else {
                continue;
            };
            edges[group][at] = if y < 0.0 {
                let low = below;
                below += y;
                (low, below)
            } else {
                let low = above;
                above += y;
                (low, above)
            };
        }
        if share {
            // No value is below zero here, and the total is above it.
            let total = above;
            for of_group in &mut edges {
                of_group[at] = (of_group[at].0 / total, of_group[at].1 / total);
            }
        }
    }
    // The runs of stacked places side by side, the same for every group.
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    while at < places.len() {
        if !whole[at] {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        while end < places.len() && whole[end] {
            end += 1;
        }
        runs.push((at, end));
        at = end;
    }
    let mut lines = 0;
    for (group, of_group) in cells.iter().enumerate() {
        for &(from, to) in &runs {
            if to - from < 2 {
                out.alone += 1;
                continue;
            }
            lines += 1;
            for at in from..to {
                let Cell::One(_, row) = of_group[at] else {
                    continue;
                };
                let (low, high) = edges[group][at];
                out.points.x.push(places[at]);
                out.points.y.push(high);
                out.points.rows.push(row);
                out.line.push(lines);
                out.low.push(low);
            }
        }
    }
    out
}

/// The bars of a layer: each with its `x`, its two ends along `y`, how far it reaches along
/// `x`, and the row it is drawn from, or for a count the first row of those it counts.
#[derive(Default)]
struct Bars {
    points: Drawn,
    low: Vec<f64>,
    span: Vec<f64>,
    /// For bars shown as shares: the bars left out because, at their `x`, a value is below
    /// zero or the total is not above it, and at how many values of `x`.
    unfit: (usize, usize),
}

/// The bars of a layer. Each row with an `x` and a `y` is a bar from zero to its `y`; where
/// nothing is mapped to `y` (`cy` is `None`) there is a bar for each `x` of each group, as high
/// as the number of its rows. The rows are parted into groups as for a line, the groups in the
/// order of their categories, and the bars that share an `x` are arranged as `how` says:
/// stacked from the last group up, a value below zero downwards from zero, as areas are;
/// stacked as shares of the total; or side by side, the bars of the groups that are there
/// sharing the width equally, the first at the left. A bar is `width` of the smallest gap
/// between two values of `x` wide, and of a place where `x` is categories.
fn bars(
    cx: Option<&Column>,
    cy: Option<Option<&Column>>,
    keys: &[Categories],
    how: Arrangement,
    width: f64,
) -> Bars {
    let mut out = Bars::default();
    let Some(cx) = cx else {
        return out;
    };
    let cy = match cy {
        Some(None) => return out,
        Some(Some(cy)) => Some(cy),
        None => None,
    };
    if !is_numeric(cx) || cy.is_some_and(|cy| !is_numeric(cy) || cy.len() != cx.len()) {
        return out;
    }
    if keys.iter().any(|key| key.of.len() != cx.len()) {
        return out;
    }
    // The rows that can be drawn, each with its group.
    struct Item {
        x: f64,
        group: Vec<(bool, usize)>,
        value: f64,
        row: usize,
    }
    let mut items: Vec<Item> = Vec::new();
    for row in 0..cx.len() {
        let x = number(cx, row);
        let y = match cy {
            Some(cy) => number(cy, row),
            None => Some(1.0),
        };
        let (Some(x), Some(y)) = (x, y) else {
            out.points.missing += 1;
            continue;
        };
        if !(x.is_finite() && y.is_finite()) {
            out.points.infinite += 1;
            continue;
        }
        items.push(Item {
            // Adding zero makes the two zeros one place.
            x: x + 0.0,
            // In the order of the categories, a row without one after the others.
            group: keys
                .iter()
                .map(|key| (key.of[row].is_none(), key.of[row].unwrap_or(0)))
                .collect(),
            value: y,
            row,
        });
    }
    // By place, then by group, then in the order of the frame.
    items.sort_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then_with(|| a.group.cmp(&b.group))
            .then(a.row.cmp(&b.row))
    });
    // A count: one bar for each group at each place, as high as the number of its rows.
    if cy.is_none() {
        let mut counted: Vec<Item> = Vec::new();
        for item in items {
            match counted.last_mut() {
                Some(last) if last.x == item.x && last.group == item.group => last.value += 1.0,
                _ => counted.push(item),
            }
        }
        items = counted;
    }
    let places: Vec<f64> = items.iter().map(|item| item.x).collect();
    let wide = width * resolution(&places);
    let mut start = 0;
    while start < items.len() {
        let mut end = start + 1;
        while end < items.len() && items[end].x == items[start].x {
            end += 1;
        }
        let here = &items[start..end];
        let x = here[0].x;
        let push = |out: &mut Bars, item: &Item, at: f64, low: f64, high: f64, span: f64| {
            out.points.x.push(at);
            out.points.y.push(high);
            out.points.rows.push(item.row);
            out.low.push(low);
            out.span.push(span);
        };
        match how {
            Arrangement::Dodge => {
                // The groups that are here, in their order, share the width.
                let mut groups: Vec<&Vec<(bool, usize)>> = Vec::new();
                for item in here {
                    if groups.last() != Some(&&item.group) {
                        groups.push(&item.group);
                    }
                }
                let slot = wide / groups.len() as f64;
                for item in here {
                    let place = groups
                        .iter()
                        .position(|group| **group == item.group)
                        .unwrap_or(0);
                    let at = x - wide / 2.0 + (place as f64 + 0.5) * slot;
                    push(&mut out, item, at, 0.0, item.value, slot);
                }
            }
            Arrangement::Stack | Arrangement::Share => {
                let share = how == Arrangement::Share;
                let total: f64 = here.iter().map(|item| item.value).sum();
                if share && (here.iter().any(|item| item.value < 0.0) || total <= 0.0) {
                    out.unfit.0 += here.len();
                    out.unfit.1 += 1;
                    start = end;
                    continue;
                }
                // From the last bar up, so that the first group is on top.
                let mut edges = vec![(0.0, 0.0); here.len()];
                let (mut above, mut below) = (0.0_f64, 0.0_f64);
                for (i, item) in here.iter().enumerate().rev() {
                    edges[i] = if item.value < 0.0 {
                        let low = below;
                        below += item.value;
                        (low, below)
                    } else {
                        let low = above;
                        above += item.value;
                        (low, above)
                    };
                }
                for (item, (low, high)) in here.iter().zip(edges) {
                    let (low, high) = if share {
                        (low / total, high / total)
                    } else {
                        (low, high)
                    };
                    push(&mut out, item, x, low, high, wide);
                }
            }
        }
        start = end;
    }
    out
}

/// The smallest gap between two different values, the unit of `jitter:`; 1 when there are not
/// two different values to take a gap between.
fn resolution(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut smallest: Option<f64> = None;
    for pair in sorted.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > 0.0 && smallest.is_none_or(|s| gap < s) {
            smallest = Some(gap);
        }
    }
    smallest.unwrap_or(1.0)
}

/// Move each point by a uniform draw on each axis, at most `share` of the axis's resolution
/// either way: one draw for `x` and then one for `y`, point by point in the order drawn, from the
/// layer's stream at its start. A point that the move would take beyond the doubles stays where
/// it is.
fn jitter(layer: &PlotLayer, points: &mut Drawn) {
    let Some(stream) = &layer.stream else {
        return;
    };
    if !(layer.jitter > 0.0 && layer.jitter.is_finite()) {
        return;
    }
    let mut stream = stream.clone();
    let reach_x = layer.jitter * resolution(&points.x);
    let reach_y = layer.jitter * resolution(&points.y);
    for i in 0..points.x.len() {
        let moved = points.x[i] + (2.0 * stream.uniform() - 1.0) * reach_x;
        if moved.is_finite() {
            points.x[i] = moved;
        }
        let moved = points.y[i] + (2.0 * stream.uniform() - 1.0) * reach_y;
        if moved.is_finite() {
            points.y[i] = moved;
        }
    }
}

/// `educ or wage`, `educ, wage or tenure`: the variables or expressions a dropped row may lack,
/// each named once.
fn either(names: &[&str]) -> String {
    let mut distinct: Vec<&str> = Vec::new();
    for name in names {
        if !distinct.contains(name) {
            distinct.push(name);
        }
    }
    match distinct.split_last() {
        None => String::new(),
        Some((only, [])) => (*only).to_string(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
    }
}

/// `1 row` or `1,204 rows`.
fn rows_text(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// The smallest and largest of some finite values.
fn extent<'a>(values: impl Iterator<Item = &'a f64>) -> Option<(f64, f64)> {
    let mut found: Option<(f64, f64)> = None;
    for &v in values {
        if !v.is_finite() {
            continue;
        }
        found = Some(match found {
            None => (v, v),
            Some((lo, hi)) => (if v < lo { v } else { lo }, if v > hi { v } else { hi }),
        });
    }
    found
}

/// The extent of what the plot's frame gives an aesthetic, for an axis no mark has yet been
/// drawn on; where the plot maps nothing to it, of what the first layer that names it reads.
fn frame_extent(plot: &Plot, aesthetic: Aesthetic) -> Option<(f64, f64)> {
    let (mapping, _, layer) = first_mapping(plot, aesthetic)?;
    let column = values(plot, layer, mapping.aesthetic)?;
    let values = column.numeric_values()?.values;
    extent(values.iter())
}

/// The title of an axis: the label of its variable in the frame it is read in, or the
/// variable's name; for an expression, the expression as written.
fn axis_title(frame: &Frame, mapping: &PlotMapping) -> String {
    if mapping.computed.is_some() {
        return mapping.variable.clone();
    }
    frame
        .table
        .column(&mapping.variable)
        .and_then(|c| c.metadata().get(MetadataKey::Label))
        .filter(|label| !label.is_empty())
        .unwrap_or(&mapping.variable)
        .to_string()
}

/// An aesthetic with a scale of its own that a variable is mapped to, a colour, the size or the
/// shape: how the variable is shown, and which layers take the aesthetic from it.
struct Colouring<'a> {
    aesthetic: Aesthetic,
    /// The mapping the scale is named by: the plot's, or the first a layer names.
    mapping: &'a PlotMapping,
    /// The frame that mapping is read in.
    frame: &'a Frame,
    /// The variable in that frame.
    base: &'a Column,
    kind: Shading,
    /// For each layer the variable its marks are coloured, sized or shaped by, in the layer's
    /// frame; `None` for a layer that does not read the aesthetic.
    readers: Vec<Option<&'a Column>>,
}

/// The colouring of `aesthetic`, if a variable is mapped to it: a size only by a variable shown
/// on a ramp and a shape only by one shown as categories, which `plot` sees to. A layer whose
/// own frame holds the variable as another kind of scale than the plot's is not coloured; the
/// marks refuse such a frame before it comes to this.
fn colouring(plot: &Plot, aesthetic: Aesthetic) -> Option<Colouring<'_>> {
    let (mapping, frame, layer) = first_mapping(plot, aesthetic)?;
    let base = values(plot, layer, aesthetic)?;
    let kind = shading(base)?;
    match (aesthetic, kind) {
        (Aesthetic::Size, Shading::Categories)
        | (Aesthetic::Shape | Aesthetic::Linetype, Shading::Ramp) => return None,
        _ => {}
    }
    let readers = plot
        .layers
        .iter()
        .map(|layer| {
            if !layer_reads(layer, aesthetic) {
                return None;
            }
            values(plot, Some(layer), aesthetic).filter(|column| shading(column) == Some(kind))
        })
        .collect();
    Some(Colouring {
        aesthetic,
        mapping,
        frame,
        base,
        kind,
        readers,
    })
}

/// A scale of colours, of sizes or of shapes as the scene holds it: the scale, for each layer
/// the value of each of its marks on it (`None` for a layer that does not read it), and what
/// there is to say about it.
struct Shaded {
    scale: Scale,
    values: Vec<Option<Vec<f64>>>,
    note: Option<String>,
}

/// The scale of a colouring. `kept` holds, for each layer, the rows its marks are drawn from.
fn shaded(plot: &Plot, colouring: &Colouring<'_>, kept: &[Vec<usize>]) -> Shaded {
    let id = colouring.aesthetic.name();
    let title = axis_title(colouring.frame, colouring.mapping);
    let layers = || colouring.readers.iter().zip(kept);
    match colouring.kind {
        Shading::Ramp => {
            // Every row kept has a finite value: `drawn` dropped the others.
            let values: Vec<Option<Vec<f64>>> = layers()
                .map(|(column, rows)| {
                    column
                        .map(|column| rows.iter().filter_map(|&row| number(column, row)).collect())
                })
                .collect();
            let found = extent(values.iter().flatten().flatten())
                .or_else(|| frame_extent(plot, colouring.aesthetic));
            if colouring.aesthetic == Aesthetic::Size {
                // The sizes run from 0, where a mark has no area, to the largest value drawn,
                // with no margin; with nothing above 0 to draw, to 1.
                let hi = found
                    .map(|(_, hi)| hi + 0.0)
                    .filter(|hi| *hi > 0.0)
                    .unwrap_or(1.0);
                // A key of no size shows nothing, so the breaks begin above 0.
                let breaks: Vec<f64> = scale::breaks(0.0, hi)
                    .into_iter()
                    .filter(|b| *b > 0.0)
                    .collect();
                let labels = scale::labels(&breaks);
                return Shaded {
                    scale: Scale {
                        id,
                        kind: ScaleKind::Sqrt,
                        title,
                        limits: (0.0, hi),
                        breaks,
                        labels,
                        range: Range::Sizes(0.0, LARGEST_SIZE),
                    },
                    values,
                    note: None,
                };
            }
            // The ramp runs from the smallest value to the largest, with no margin: a colour
            // beyond the data would never be seen.
            let (lo, hi) = match found {
                // Adding zero makes the two zeros one limit, whichever row came first.
                Some((lo, hi)) if lo < hi => (lo + 0.0, hi + 0.0),
                other => scale::limits(other),
            };
            let breaks = scale::breaks(lo, hi);
            let labels = scale::labels(&breaks);
            Shaded {
                scale: Scale {
                    id,
                    kind: ScaleKind::Linear,
                    title,
                    limits: (lo, hi),
                    breaks,
                    labels,
                    range: Range::Colours(colour::VIRIDIS.to_vec()),
                },
                values,
                note: None,
            }
        }
        Shading::Categories => {
            // The categories of the plot's frame come first, in their order; one that only a
            // layer's own frame has follows them.
            let own = categories(colouring.base);
            let mut labels = own.labels.clone();
            let mut used = vec![false; labels.len()];
            let mut na = false;
            let mut seen = |place: Option<usize>, used: &mut Vec<bool>| match place {
                Some(place) => {
                    if let Some(slot) = used.get_mut(place) {
                        *slot = true;
                    }
                }
                None => na = true,
            };
            let mut placed: Vec<Option<Vec<Option<usize>>>> = Vec::new();
            for (column, rows) in layers() {
                let Some(column) = column else {
                    placed.push(None);
                    continue;
                };
                let theirs = if std::ptr::eq(*column, colouring.base) {
                    own.clone()
                } else {
                    categories(column)
                };
                let at: Vec<usize> = theirs
                    .labels
                    .iter()
                    .map(|label| match labels.iter().position(|l| l == label) {
                        Some(place) => place,
                        None => {
                            labels.push(label.clone());
                            used.push(false);
                            labels.len() - 1
                        }
                    })
                    .collect();
                let marks: Vec<Option<usize>> = rows
                    .iter()
                    .map(|&row| {
                        theirs
                            .of
                            .get(row)
                            .copied()
                            .flatten()
                            .and_then(|place| at.get(place).copied())
                    })
                    .collect();
                for mark in &marks {
                    seen(*mark, &mut used);
                }
                placed.push(Some(marks));
            }
            // While no mark is drawn, the scale shows what the plot's frame holds, as an axis
            // does.
            if !placed.iter().flatten().any(|marks| !marks.is_empty()) {
                for place in &own.of {
                    seen(*place, &mut used);
                }
            }
            // The place of each category that is there, counted from 1; `NA` is the last.
            let shapes = colouring.aesthetic == Aesthetic::Shape;
            let lines = colouring.aesthetic == Aesthetic::Linetype;
            // The type of line of a place among the categories: the six in their order, and
            // from the seventh the six again.
            let line_type = |at: usize| LineType::ALL[at % LineType::ALL.len()].name();
            let mut position = vec![0_usize; labels.len()];
            let mut shown: Vec<String> = Vec::new();
            let mut colours: Vec<&'static str> = Vec::new();
            for (place, label) in labels.iter().enumerate() {
                if used[place] {
                    colours.push(if shapes {
                        shape::category(shown.len())
                    } else if lines {
                        line_type(shown.len())
                    } else {
                        colour::category(shown.len()).0
                    });
                    shown.push(label.clone());
                    position[place] = shown.len();
                }
            }
            let named = shown.len();
            if na {
                // The rows without a category have the type of line after the last given.
                colours.push(if shapes {
                    shape::NA_SHAPE
                } else if lines {
                    line_type(shown.len())
                } else {
                    colour::NA_COLOUR.0
                });
                shown.push("NA".to_string());
            }
            let total = shown.len();
            let values = placed
                .into_iter()
                .map(|marks| {
                    marks.map(|marks| {
                        marks
                            .into_iter()
                            .map(|mark| mark.map_or(total, |place| position[place]) as f64)
                            .collect()
                    })
                })
                .collect();
            let read = colouring.readers.iter().any(Option::is_some);
            let palette = if shapes {
                shape::PALETTE.len()
            } else if lines {
                LineType::ALL.len()
            } else {
                colour::OKABE_ITO.len()
            };
            let note = (read && named > palette).then(|| {
                let (variable, named) = (&colouring.mapping.variable, count(named as u64));
                if lines {
                    format!(
                        "`{variable}` has {named} categories and there are {palette} types of line, so the types repeat from the seventh category"
                    )
                } else if shapes {
                    format!(
                        "`{variable}` has {named} categories and there are {palette} shapes, so the shapes repeat from the ninth category"
                    )
                } else {
                    format!(
                        "`{variable}` has {named} categories and the palette {palette} colours, so the colours repeat from the ninth category"
                    )
                }
            });
            Shaded {
                scale: Scale {
                    id,
                    kind: ScaleKind::Category,
                    title,
                    limits: (1.0, total.max(1) as f64),
                    breaks: (1..=total).map(|place| place as f64).collect(),
                    labels: shown,
                    range: if shapes {
                        Range::Shapes(colours)
                    } else if lines {
                        Range::LineTypes(colours)
                    } else {
                        Range::Colours(colours)
                    },
                },
                values,
                note,
            }
        }
    }
}

/// The scene of a plot.
pub fn scene<'a>(plot: &'a Plot) -> Scene {
    let colourings: Vec<Colouring<'_>> = [
        Aesthetic::Color,
        Aesthetic::Fill,
        Aesthetic::Size,
        Aesthetic::Shape,
        Aesthetic::Linetype,
    ]
    .into_iter()
    .filter_map(|aesthetic| colouring(plot, aesthetic))
    .collect();
    // The axes of categories, and what each layer is placed by on its two axes.
    let across_categories = axis_categories(plot, Aesthetic::X);
    let up_categories = axis_categories(plot, Aesthetic::Y);
    let place = |layer: &'a PlotLayer, aesthetic: Aesthetic| -> Option<Cow<'a, Column>> {
        let on = if aesthetic == Aesthetic::X {
            across_categories.as_deref()
        } else {
            up_categories.as_deref()
        };
        placed(values(plot, Some(layer), aesthetic), on)
    };
    let mut notes: Vec<String> = Vec::new();
    let mut note = |text: String| {
        if !notes.contains(&text) {
            notes.push(text);
        }
    };
    let mut layers: Vec<Layer> = Vec::new();
    // For each layer, the rows its marks are drawn from.
    let mut kept: Vec<Vec<usize>> = Vec::new();
    for (at, layer) in plot.layers.iter().enumerate() {
        // What the layer draws each position from: its own mapping, or the plot's.
        let named = |aesthetic: Aesthetic| {
            mapping_of(plot, layer, aesthetic).map(|mapping| mapping.variable.as_str())
        };
        // For a layer of lines, the line of each mark; for a filled one, its lower edge too.
        let mut line: Vec<usize> = Vec::new();
        let mut low: Vec<f64> = Vec::new();
        // A band has two edges along `y`, the upper in the place of `y`.
        let (top, bottom) = if layer.mark == Mark::Ribbon {
            (Aesthetic::High, Some(Aesthetic::Low))
        } else {
            (Aesthetic::Y, None)
        };
        // A reference line: one at the place the layer sets, or one at each value of the
        // variable it names, which must be there and finite, as must a value it is coloured
        // by on a ramp.
        // A rug along one axis is placed the same way: a mark for each row with a value on
        // that axis, whatever the row holds on the other.
        let single = match layer.mark {
            Mark::Hline | Mark::Rug(Sides::Y) => Some(Aesthetic::Y),
            Mark::Vline | Mark::Rug(Sides::X) => Some(Aesthetic::X),
            _ => None,
        };
        let along = single.unwrap_or(Aesthetic::X);
        let across = |mut lines: Drawn| {
            if along == Aesthetic::Y {
                lines.x.clear();
            } else {
                lines.y.clear();
            }
            lines
        };
        // What parts the rows of a layer into groups: the group, and the categories a
        // colour, a fill or a type of line shows, since one line has one colour and one type
        // and one filled shape one fill. A filled mark draws no colour of its own, but where
        // the plot colours by categories that the layer's frame holds, each category has its
        // own shape, so that an area lies under each line of the plot.
        let parted = || {
            let mut keys: Vec<Categories> = Vec::new();
            if mapping_of(plot, layer, Aesthetic::Group).is_some()
                && let Some(column) = values(plot, Some(layer), Aesthetic::Group)
            {
                keys.push(categories(column));
            }
            for colouring in &colourings {
                if colouring.kind == Shading::Categories
                    && matches!(
                        colouring.aesthetic,
                        Aesthetic::Color | Aesthetic::Linetype | Aesthetic::Fill
                    )
                    && let Some(column) = colouring.readers[at]
                {
                    keys.push(categories(column));
                }
            }
            if layer.mark.patches()
                && plot.mapping(Aesthetic::Color).is_some()
                && let Some(column) = values(plot, Some(layer), Aesthetic::Color)
                && shading(column) == Some(Shading::Categories)
            {
                keys.push(categories(column));
            }
            keys
        };
        // For bars, how far each reaches along `x`, and whether they count rows.
        let mut span: Vec<f64> = Vec::new();
        let mut counted = false;
        let mut points = if let Mark::Bars(how) = layer.mark {
            match named(Aesthetic::X) {
                Some(x) => {
                    let across = place(layer, Aesthetic::X);
                    let height = named(Aesthetic::Y);
                    let upper = place(layer, Aesthetic::Y);
                    counted = height.is_none();
                    let drawn = bars(
                        across.as_deref(),
                        height.map(|_| upper.as_deref()),
                        &parted(),
                        how,
                        layer.width.unwrap_or(BAR_WIDTH),
                    );
                    let mut needs = vec![x];
                    needs.extend(height);
                    if drawn.points.missing > 0 {
                        note(format!(
                            "{} with NA in {} dropped",
                            rows_text(drawn.points.missing),
                            either(&needs)
                        ));
                    }
                    if drawn.points.infinite > 0 {
                        note(format!(
                            "{} with an infinite value in {} dropped",
                            rows_text(drawn.points.infinite),
                            either(&needs)
                        ));
                    }
                    if drawn.unfit.0 > 0 {
                        note(format!(
                            "{} at {} left out: shares need values of 0 or more and a total above 0",
                            if drawn.unfit.0 == 1 {
                                "1 bar".to_string()
                            } else {
                                format!("{} bars", count(drawn.unfit.0 as u64))
                            },
                            if drawn.unfit.1 == 1 {
                                format!("1 value of {x}")
                            } else {
                                format!("{} values of {x}", count(drawn.unfit.1 as u64))
                            }
                        ));
                    }
                    low = drawn.low;
                    span = drawn.span;
                    Some(drawn.points)
                }
                None => Some(Drawn::default()),
            }
        } else if single.is_none() {
            None
        } else if let Some(PlotSetting::Place(at)) = layer.constant(along) {
            Some(across(Drawn {
                x: vec![*at],
                y: vec![*at],
                ..Drawn::default()
            }))
        } else if let Some(variable) = named(along) {
            let ramps: Vec<(&str, &Column)> = colourings
                .iter()
                .filter(|colouring| {
                    colouring.kind == Shading::Ramp && colouring.aesthetic == Aesthetic::Color
                })
                .filter_map(|colouring| {
                    colouring.readers[at].map(|column| {
                        (
                            named(colouring.aesthetic)
                                .unwrap_or(colouring.mapping.variable.as_str()),
                            column,
                        )
                    })
                })
                .collect();
            let columns: Vec<&Column> = ramps.iter().map(|(_, column)| *column).collect();
            let places = place(layer, along);
            let lines = drawn(places.as_deref(), places.as_deref(), &columns, None);
            let mut names = vec![variable];
            names.extend(ramps.iter().map(|(name, _)| *name));
            if lines.missing > 0 {
                note(format!(
                    "{} with NA in {} dropped",
                    rows_text(lines.missing),
                    either(&names)
                ));
            }
            if lines.infinite > 0 {
                note(format!(
                    "{} with an infinite value in {} dropped",
                    rows_text(lines.infinite),
                    either(&names)
                ));
            }
            Some(across(lines))
        } else {
            Some(Drawn::default())
        };
        let mut points = match points.take() {
            Some(lines) => lines,
            None => match (named(Aesthetic::X), named(top)) {
                (Some(x), Some(y)) if layer.mark.joins() || layer.mark.fills() => {
                    let keys = parted();
                    let lower = bottom.map(|edge| place(layer, edge));
                    let (across, upper) = (place(layer, Aesthetic::X), place(layer, top));
                    let lines = match layer.mark {
                        Mark::Area(Stacking::Stack | Stacking::Share) => stacked(
                            across.as_deref(),
                            upper.as_deref(),
                            &keys,
                            layer.mark == Mark::Area(Stacking::Share),
                        ),
                        _ => joined(
                            across.as_deref(),
                            // A band whose lower edge is not there draws nothing.
                            upper.as_deref().filter(|_| !matches!(lower, Some(None))),
                            lower.as_ref().and_then(|edge| edge.as_deref()),
                            &keys,
                        ),
                    };
                    // The variables a row needs, and the words for what is drawn.
                    let edge = bottom.and_then(named);
                    let mut needs = vec![x];
                    needs.extend(edge);
                    needs.push(y);
                    let upright = match edge {
                        Some(edge) => either(&[edge, y]),
                        None => y.to_string(),
                    };
                    let (one, many, none) = match layer.mark {
                        Mark::Area(_) => ("an area", "the areas", "area"),
                        Mark::Ribbon => ("a band", "the bands", "band"),
                        _ => ("a line", "the lines", "line"),
                    };
                    if lines.points.missing > 0 {
                        note(format!(
                            "{} with NA in {} dropped",
                            rows_text(lines.points.missing),
                            either(&needs)
                        ));
                    }
                    if lines.points.infinite > 0 {
                        note(format!(
                            "{} with an infinite value in {} dropped",
                            rows_text(lines.points.infinite),
                            either(&needs)
                        ));
                    }
                    if lines.breaks > 0 {
                        let lacking = if lines.infinite_y {
                            "NA or infinite"
                        } else {
                            "NA"
                        };
                        note(if lines.breaks == 1 {
                            format!("1 break in {one} where {upright} is {lacking}")
                        } else {
                            format!(
                                "{} breaks in {many} where {upright} is {lacking}",
                                count(lines.breaks as u64)
                            )
                        });
                    }
                    // What a stack leaves out: the places where it cannot be built.
                    let places = |n: usize| {
                        if n == 1 {
                            format!("1 value of {x}")
                        } else {
                            format!("{} values of {x}", count(n as u64))
                        }
                    };
                    if lines.left.0 > 0 {
                        note(format!(
                            "{} at {} left out of the stack: not every area has one value there",
                            rows_text(lines.left.0),
                            places(lines.left.1)
                        ));
                    }
                    if lines.unfit.0 > 0 {
                        note(format!(
                            "{} at {} left out of the stack: shares need values of 0 or more and a total above 0",
                            rows_text(lines.unfit.0),
                            places(lines.unfit.1)
                        ));
                    }
                    if lines.alone > 0 {
                        note(format!(
                            "{} with no neighbour to be joined to {} no {none}",
                            rows_text(lines.alone),
                            if lines.alone == 1 { "draws" } else { "draw" }
                        ));
                    }
                    line = lines.line;
                    if layer.mark.fills() {
                        low = lines.low;
                    }
                    lines.points
                }
                (Some(x), Some(y)) => {
                    // A value on a ramp of colours must be there and finite, as a position must; a
                    // row without a category is drawn, in the colour of `NA`.
                    let read = |colouring: &Colouring<'a>| {
                        colouring.readers[at].map(|column| {
                            (
                                named(colouring.aesthetic)
                                    .unwrap_or(colouring.mapping.variable.as_str()),
                                column,
                            )
                        })
                    };
                    let ramps: Vec<(&str, &Column)> = colourings
                        .iter()
                        .filter(|colouring| {
                            colouring.kind == Shading::Ramp
                                && colouring.aesthetic != Aesthetic::Size
                        })
                        .filter_map(read)
                        .collect();
                    // A size must be there and finite as well, and not below 0.
                    let sized = colourings
                        .iter()
                        .filter(|colouring| colouring.aesthetic == Aesthetic::Size)
                        .find_map(read);
                    let columns: Vec<&Column> = ramps.iter().map(|(_, column)| *column).collect();
                    let (across, upper) = (place(layer, Aesthetic::X), place(layer, Aesthetic::Y));
                    let points = drawn(
                        across.as_deref(),
                        upper.as_deref(),
                        &columns,
                        sized.map(|(_, column)| column),
                    );
                    let mut names = vec![x, y];
                    names.extend(ramps.iter().map(|(name, _)| *name));
                    names.extend(sized.map(|(name, _)| name));
                    if points.missing > 0 {
                        note(format!(
                            "{} with NA in {} dropped",
                            rows_text(points.missing),
                            either(&names)
                        ));
                    }
                    if points.infinite > 0 {
                        note(format!(
                            "{} with an infinite value in {} dropped",
                            rows_text(points.infinite),
                            either(&names)
                        ));
                    }
                    if let Some((name, column)) = sized {
                        if points.negative > 0 {
                            note(format!(
                                "{} with {name} below 0 dropped: `size` shows amounts of 0 or more",
                                rows_text(points.negative)
                            ));
                        }
                        // A mark of no size is drawn, and not seen.
                        let nothing = points
                            .rows
                            .iter()
                            .filter(|&&row| number(column, row) == Some(0.0))
                            .count();
                        if nothing > 0 {
                            note(format!(
                                "{} with {name} of 0 {} drawn with no size",
                                rows_text(nothing),
                                if nothing == 1 { "is" } else { "are" }
                            ));
                        }
                    }
                    points
                }
                _ => Drawn::default(),
            },
        };
        jitter(layer, &mut points);
        // A constant colour, for a mark that has one; a mapped colour takes its place below,
        // when its scale is known.
        // What the layer says of an aesthetic comes before what the plot says of it.
        let set = |aesthetic: Aesthetic| match layer.constant(aesthetic) {
            Some(own) => Some(own),
            None if layer.mapping(aesthetic).is_some() => None,
            None if layer.mark.spans() => None,
            None => plot.constant(aesthetic),
        };
        let color = match set(Aesthetic::Color) {
            Some(PlotSetting::Colour { hex, .. }) if reads(layer.mark, Aesthetic::Color) => {
                Paint::Constant(hex.clone())
            }
            _ => Paint::Theme,
        };
        // The layer's own size comes before the plot's; a mapped size and a mapped shape take
        // their place below, as a mapped colour does.
        let size = match (layer.size, set(Aesthetic::Size)) {
            (Some(own), _) => own,
            (None, Some(PlotSetting::Size(all))) if reads(layer.mark, Aesthetic::Size) => *all,
            _ => POINT_SIZE,
        };
        let outline = match set(Aesthetic::Shape) {
            Some(PlotSetting::Shape(name)) if reads(layer.mark, Aesthetic::Shape) => *name,
            _ => shape::DEFAULT,
        };
        let linetype = match set(Aesthetic::Linetype) {
            Some(PlotSetting::Line(name)) if reads(layer.mark, Aesthetic::Linetype) => *name,
            _ => LineType::Solid.name(),
        };
        let fill = match set(Aesthetic::Fill) {
            Some(PlotSetting::Colour { hex, .. }) if reads(layer.mark, Aesthetic::Fill) => {
                Paint::Constant(hex.clone())
            }
            _ => Paint::Theme,
        };
        layers.push(Layer {
            mark: layer.mark,
            x: points.x,
            y: points.y,
            color,
            // The size of a rug is the length of its ticks.
            size: Sizing::Constant(if matches!(layer.mark, Mark::Rug(_)) {
                RUG_LENGTH
            } else {
                size
            }),
            shape: Shaping::Constant(outline),
            alpha: layer.alpha,
            data: layer.data.as_ref().unwrap_or(&plot.frame).name.clone(),
            // A bar that counts rows is drawn from no one row.
            rows: if counted {
                Vec::new()
            } else {
                points.rows.iter().map(|row| row + 1).collect()
            },
            z: None,
            raster: false,
            line,
            linetype: Lining::Constant(linetype),
            width: layer
                .width
                .unwrap_or(if matches!(layer.mark, Mark::Bars(_)) {
                    BAR_WIDTH
                } else {
                    LINE_WIDTH
                }),
            low,
            fill,
            span,
        });
        kept.push(points.rows);
    }
    let mut scales: Vec<Scale> = Vec::new();
    let mut axes: Vec<Axis> = Vec::new();
    for (aesthetic, position) in [
        (Aesthetic::X, Position::Bottom),
        (Aesthetic::Y, Position::Left),
    ] {
        // Bars that count rows give the plot a `y` axis where nothing is mapped to `y`.
        let counting = aesthetic == Aesthetic::Y
            && plot.layers.iter().any(|layer| {
                matches!(layer.mark, Mark::Bars(_))
                    && mapping_of(plot, layer, Aesthetic::Y).is_none()
                    && mapping_of(plot, layer, Aesthetic::X).is_some()
            });
        let title = match first_mapping(plot, aesthetic) {
            Some((mapping, frame, _)) => axis_title(frame, mapping),
            None if counting => "count".to_string(),
            None => continue,
        };
        // Along `y` a filled mark reaches from its lower edge to its upper one, and along `x`
        // a bar half its width to either side of its place.
        let edges: Vec<f64> = layers
            .iter()
            .flat_map(|layer| {
                layer
                    .x
                    .iter()
                    .zip(&layer.span)
                    .flat_map(|(x, span)| [x - span / 2.0, x + span / 2.0])
            })
            .collect();
        let none: &[f64] = &[];
        let marks = extent(
            layers
                .iter()
                .flat_map(|layer| {
                    if aesthetic == Aesthetic::X {
                        layer.x.iter().chain(none)
                    } else {
                        layer.y.iter().chain(&layer.low)
                    }
                })
                .chain(if aesthetic == Aesthetic::X {
                    &edges[..]
                } else {
                    none
                }),
        );
        let categories = if aesthetic == Aesthetic::X {
            &across_categories
        } else {
            &up_categories
        };
        // An axis of categories: a place for each, 1, 2, 3, and 0.6 of a place beyond each
        // end, whatever is drawn; with no category, the room of one.
        if let Some(labels) = categories {
            let places = labels.len().max(1);
            scales.push(Scale {
                id: aesthetic.name(),
                kind: ScaleKind::Category,
                title,
                limits: (0.4, places as f64 + 0.6),
                breaks: (1..=labels.len()).map(|place| place as f64).collect(),
                labels: labels.clone(),
                range: Range::Position,
            });
            axes.push(Axis {
                scale: aesthetic.name(),
                position,
            });
            continue;
        }
        let (lo, hi) = scale::limits(marks.or_else(|| frame_extent(plot, aesthetic)));
        let breaks = scale::breaks(lo, hi);
        let labels = scale::labels(&breaks);
        scales.push(Scale {
            id: aesthetic.name(),
            kind: ScaleKind::Linear,
            title,
            limits: (lo, hi),
            breaks,
            labels,
            range: Range::Position,
        });
        axes.push(Axis {
            scale: aesthetic.name(),
            position,
        });
    }
    let mut legends: Vec<Legend> = Vec::new();
    // The variable each legend shows: one legend for all the scales of one variable.
    let mut legend_of: Vec<&str> = Vec::new();
    for colouring in &colourings {
        let Shaded {
            scale,
            values,
            note: said,
        } = shaded(plot, colouring, &kept);
        // A legend for a scale that a layer reads, with the marks that read it as its keys.
        let mut marks: Vec<Mark> = Vec::new();
        for (layer, reader) in plot.layers.iter().zip(&colouring.readers) {
            if reader.is_some() && !marks.contains(&layer.mark) {
                marks.push(layer.mark);
            }
        }
        let variable = colouring.mapping.variable.as_str();
        match legend_of.iter().position(|shown| *shown == variable) {
            _ if marks.is_empty() => {}
            Some(at) => {
                let legend = &mut legends[at];
                legend.scales.push(scale.id);
                for mark in marks {
                    if !legend.marks.contains(&mark) {
                        legend.marks.push(mark);
                    }
                }
            }
            None => {
                legend_of.push(variable);
                legends.push(Legend {
                    title: scale.title.clone(),
                    scales: vec![scale.id],
                    marks,
                });
            }
        }
        for (layer, values) in layers.iter_mut().zip(values) {
            let Some(values) = values else {
                continue;
            };
            match colouring.aesthetic {
                Aesthetic::Color => layer.color = Paint::Scaled(values),
                Aesthetic::Fill => layer.fill = Paint::Scaled(values),
                Aesthetic::Size => layer.size = Sizing::Scaled(values),
                Aesthetic::Shape => layer.shape = Shaping::Scaled(values),
                Aesthetic::Linetype => layer.linetype = Lining::Scaled(values),
                _ => {}
            }
        }
        if let Some(said) = said {
            note(said);
        }
        scales.push(scale);
    }
    let placed_by = |aesthetic: Aesthetic| {
        axes.iter()
            .any(|axis| axis.scale == aesthetic.name())
            .then(|| aesthetic.name())
    };
    let (x, y) = (placed_by(Aesthetic::X), placed_by(Aesthetic::Y));
    Scene {
        rows: 1,
        columns: 1,
        scales,
        axes,
        legends,
        panels: vec![Panel {
            row: 1,
            column: 1,
            x,
            y,
            layers,
            projection: None,
            crs: None,
        }],
        notes,
        frame: None,
    }
}

impl Scene {
    /// The scale called `id`.
    pub fn scale(&self, id: &str) -> Option<&Scale> {
        self.scales.iter().find(|s| s.id == id)
    }

    /// The scene as its JSON value, the keys in a fixed order.
    pub fn json(&self) -> Json {
        let name = |id: Option<&'static str>| id.map_or(Json::Null, Json::text);
        Json::Object(
            vec![
                ("zeff_scene", Json::Number(f64::from(SCENE_VERSION))),
                (
                    "layout",
                    Json::Object(vec![
                        ("rows", Json::whole(self.rows)),
                        ("columns", Json::whole(self.columns)),
                    ]),
                ),
                (
                    "scales",
                    Json::List(
                        self.scales
                            .iter()
                            .map(|s| {
                                Json::Object(vec![
                                    ("id", Json::text(s.id)),
                                    ("kind", Json::text(s.kind.name())),
                                    ("title", Json::text(s.title.as_str())),
                                    ("limits", Json::numbers(&[s.limits.0, s.limits.1])),
                                    ("breaks", Json::numbers(&s.breaks)),
                                    ("labels", Json::texts(&s.labels)),
                                    (
                                        "range",
                                        match &s.range {
                                            Range::Position => Json::Null,
                                            Range::Colours(names)
                                            | Range::Shapes(names)
                                            | Range::LineTypes(names) => Json::List(
                                                names.iter().map(|c| Json::text(*c)).collect(),
                                            ),
                                            Range::Sizes(least, largest) => {
                                                Json::numbers(&[*least, *largest])
                                            }
                                        },
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
                (
                    "guides",
                    Json::Object(vec![
                        (
                            "axes",
                            Json::List(
                                self.axes
                                    .iter()
                                    .map(|a| {
                                        Json::Object(vec![
                                            ("scale", Json::text(a.scale)),
                                            ("position", Json::text(a.position.name())),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                        (
                            "legends",
                            Json::List(
                                self.legends
                                    .iter()
                                    .map(|l| {
                                        Json::Object(vec![
                                            ("title", Json::text(l.title.as_str())),
                                            (
                                                "scales",
                                                Json::List(
                                                    l.scales
                                                        .iter()
                                                        .map(|s| Json::text(*s))
                                                        .collect(),
                                                ),
                                            ),
                                            (
                                                "marks",
                                                Json::List(
                                                    l.marks
                                                        .iter()
                                                        .map(|m| Json::text(m.name()))
                                                        .collect(),
                                                ),
                                            ),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                    ]),
                ),
                (
                    "panels",
                    Json::List(
                        self.panels
                            .iter()
                            .map(|p| {
                                Json::Object(
                                    vec![
                                        ("row", Json::whole(p.row)),
                                        ("column", Json::whole(p.column)),
                                        ("x", name(p.x)),
                                        ("y", name(p.y)),
                                        (
                                            "layers",
                                            Json::List(
                                                p.layers
                                                    .iter()
                                                    .map(|l| {
                                                        Json::Object(
                                                            vec![
                                                                ("mark", Json::text(l.mark.name())),
                                                                ("x", Json::numbers(&l.x)),
                                                                ("y", Json::numbers(&l.y)),
                                                                (
                                                                    "color",
                                                                    match &l.color {
                                                                        Paint::Theme => Json::Null,
                                                                        Paint::Constant(hex) => {
                                                                            Json::text(hex.as_str())
                                                                        }
                                                                        Paint::Scaled(values) => {
                                                                            Json::numbers(values)
                                                                        }
                                                                    },
                                                                ),
                                                                (
                                                                    "size",
                                                                    match &l.size {
                                                                        Sizing::Constant(
                                                                            across,
                                                                        ) => Json::Number(*across),
                                                                        Sizing::Scaled(values) => {
                                                                            Json::numbers(values)
                                                                        }
                                                                    },
                                                                ),
                                                                (
                                                                    "shape",
                                                                    match &l.shape {
                                                                        Shaping::Constant(name) => {
                                                                            Json::text(*name)
                                                                        }
                                                                        Shaping::Scaled(values) => {
                                                                            Json::numbers(values)
                                                                        }
                                                                    },
                                                                ),
                                                                ("alpha", Json::Number(l.alpha)),
                                                                (
                                                                    "data",
                                                                    match &l.data {
                                                                        Some(name) => Json::text(
                                                                            name.as_str(),
                                                                        ),
                                                                        None => Json::Null,
                                                                    },
                                                                ),
                                                                (
                                                                    "rows",
                                                                    Json::List(
                                                                        l.rows
                                                                            .iter()
                                                                            .map(|row| {
                                                                                Json::whole(*row)
                                                                            })
                                                                            .collect(),
                                                                    ),
                                                                ),
                                                            ]
                                                            .into_iter()
                                                            .chain(
                                                                l.z.as_ref().map(|z| {
                                                                    ("z", Json::numbers(z))
                                                                }),
                                                            )
                                                            .chain(l.raster.then_some((
                                                                "raster",
                                                                Json::Bool(true),
                                                            )))
                                                            // What a layer of lines holds
                                                            // more: the line of each mark,
                                                            // the type and the width.
                                                            .chain(if l.mark.joins() {
                                                                vec![
                                                                    (
                                                                        "line",
                                                                        Json::List(
                                                                            l.line
                                                                                .iter()
                                                                                .map(|n| {
                                                                                    Json::whole(*n)
                                                                                })
                                                                                .collect(),
                                                                        ),
                                                                    ),
                                                                    (
                                                                        "linetype",
                                                                        match &l.linetype {
                                                                            Lining::Constant(
                                                                                name,
                                                                            ) => Json::text(*name),
                                                                            Lining::Scaled(
                                                                                values,
                                                                            ) => Json::numbers(
                                                                                values,
                                                                            ),
                                                                        },
                                                                    ),
                                                                    (
                                                                        "width",
                                                                        Json::Number(l.width),
                                                                    ),
                                                                ]
                                                            } else {
                                                                Vec::new()
                                                            })
                                                            // A layer of reference lines
                                                            // says their type and width.
                                                            .chain(
                                                                if matches!(l.mark, Mark::Rug(_)) {
                                                                    Some((
                                                                        "width",
                                                                        Json::Number(l.width),
                                                                    ))
                                                                } else {
                                                                    None
                                                                },
                                                            )
                                                            // A layer of bars says the two
                                                            // ends of each along `y`, how
                                                            // far it reaches along `x`, and
                                                            // its fill.
                                                            .chain(
                                                                if matches!(l.mark, Mark::Bars(_)) {
                                                                    vec![
                                                                        (
                                                                            "low",
                                                                            Json::numbers(&l.low),
                                                                        ),
                                                                        (
                                                                            "span",
                                                                            Json::numbers(&l.span),
                                                                        ),
                                                                        (
                                                                            "fill",
                                                                            match &l.fill {
                                                                                Paint::Theme => {
                                                                                    Json::Null
                                                                                }
                                                                                Paint::Constant(
                                                                                    hex,
                                                                                ) => Json::text(
                                                                                    hex.as_str(),
                                                                                ),
                                                                                Paint::Scaled(
                                                                                    values,
                                                                                ) => Json::numbers(
                                                                                    values,
                                                                                ),
                                                                            },
                                                                        ),
                                                                    ]
                                                                } else {
                                                                    Vec::new()
                                                                },
                                                            )
                                                            .chain(if l.mark.spans() {
                                                                vec![
                                                                    (
                                                                        "linetype",
                                                                        match &l.linetype {
                                                                            Lining::Constant(
                                                                                name,
                                                                            ) => Json::text(*name),
                                                                            Lining::Scaled(
                                                                                values,
                                                                            ) => Json::numbers(
                                                                                values,
                                                                            ),
                                                                        },
                                                                    ),
                                                                    (
                                                                        "width",
                                                                        Json::Number(l.width),
                                                                    ),
                                                                ]
                                                            } else {
                                                                Vec::new()
                                                            })
                                                            // What a filled layer holds
                                                            // more: the line each mark is
                                                            // filled under, the lower edge
                                                            // and the fill.
                                                            .chain(if l.mark.fills() {
                                                                vec![
                                                                    (
                                                                        "line",
                                                                        Json::List(
                                                                            l.line
                                                                                .iter()
                                                                                .map(|n| {
                                                                                    Json::whole(*n)
                                                                                })
                                                                                .collect(),
                                                                        ),
                                                                    ),
                                                                    ("low", Json::numbers(&l.low)),
                                                                    (
                                                                        "fill",
                                                                        match &l.fill {
                                                                            Paint::Theme => {
                                                                                Json::Null
                                                                            }
                                                                            Paint::Constant(
                                                                                hex,
                                                                            ) => Json::text(
                                                                                hex.as_str(),
                                                                            ),
                                                                            Paint::Scaled(
                                                                                values,
                                                                            ) => Json::numbers(
                                                                                values,
                                                                            ),
                                                                        },
                                                                    ),
                                                                ]
                                                            } else {
                                                                Vec::new()
                                                            })
                                                            // And a layer of steps the way
                                                            // they go, a layer of areas how
                                                            // they are placed.
                                                            .chain(match l.mark {
                                                                Mark::Step(way) => Some((
                                                                    "direction",
                                                                    Json::text(way.name()),
                                                                )),
                                                                Mark::Area(placed) => Some((
                                                                    "position",
                                                                    Json::text(placed.name()),
                                                                )),
                                                                Mark::Rug(sides) => Some((
                                                                    "sides",
                                                                    Json::text(sides.name()),
                                                                )),
                                                                Mark::Bars(how) => Some((
                                                                    "position",
                                                                    Json::text(how.name()),
                                                                )),
                                                                _ => None,
                                                            })
                                                            .collect(),
                                                        )
                                                    })
                                                    .collect(),
                                            ),
                                        ),
                                    ]
                                    .into_iter()
                                    .chain(
                                        p.projection
                                            .as_ref()
                                            .map(|how| ("projection", Json::text(how.as_str()))),
                                    )
                                    .chain(
                                        p.crs.as_ref().map(|crs| ("crs", Json::text(crs.as_str()))),
                                    )
                                    .collect(),
                                )
                            })
                            .collect(),
                    ),
                ),
                ("notes", Json::texts(&self.notes)),
            ]
            .into_iter()
            .chain(self.frame.map(|frame| ("frame", Json::whole(frame))))
            .collect(),
        )
    }

    /// What this scene holds that this version of Zeff does not draw, as a sentence for the
    /// reader, or `None` when it can be drawn: a name reserved for a later release (`frame`,
    /// `z`, `projection`, `crs`) with a value. A writer asks before it draws, and refuses.
    pub fn undrawn(&self) -> Option<String> {
        let reserved = if self.frame.is_some() {
            Some(("frame", "the place of a scene in an animation"))
        } else if self.panels.iter().any(|p| p.crs.is_some()) {
            Some(("crs", "the coordinate system of a map"))
        } else if self.panels.iter().any(|p| p.projection.is_some()) {
            Some((
                "projection",
                "how a third coordinate is brought to the page",
            ))
        } else if self
            .panels
            .iter()
            .any(|p| p.layers.iter().any(|l| l.z.is_some()))
        {
            Some(("z", "a third coordinate"))
        } else {
            None
        };
        reserved.map(|(name, meaning)| {
            format!(
                "this scene holds `{name}`, {meaning}, which this version of Zeff does not draw; it was made by a later version, which can draw it"
            )
        })
    }

    /// The scene as JSON text, the form the writers and Zemp read.
    pub fn to_json(&self) -> String {
        self.json().write()
    }
}

/// A list as the printed plot shows it: all of it up to six items, and of a longer one the
/// first five and how many more there are.
fn listed(items: &[String]) -> String {
    const WHOLE: usize = 6;
    if items.len() <= WHOLE {
        return items.join(", ");
    }
    format!(
        "{} and {} more",
        items[..WHOLE - 1].join(", "),
        count((items.len() - (WHOLE - 1)) as u64)
    )
}

/// A table of the printed plot with no title of its own.
fn part(columns: Vec<(String, Align)>, rows: Vec<Vec<String>>) -> Table {
    Table {
        title: String::new(),
        before: Vec::new(),
        columns,
        rows,
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    }
}

/// The scene read as text, the printed form of a plot: a line for each axis with its title, its
/// limits and its breaks, a line for each other aesthetic with what it shows and as what, then
/// a line for each layer with the number of marks it draws.
pub fn summary(plot: &Plot, scene: &Scene) -> Table {
    // The frame is named when the script named it; a frame written in place has no name to give.
    let shown = plot.frame.shown_name();
    let named = match &plot.frame.name {
        Some(name) => format!("plot · {name}"),
        None => "plot".to_string(),
    };
    let mapped = |aesthetic: Aesthetic| {
        first_mapping(plot, aesthetic).map(|(mapping, _, _)| mapping.variable.as_str())
    };
    let title = match (mapped(Aesthetic::X), mapped(Aesthetic::Y)) {
        (Some(x), Some(y)) => format!("{named} · {y} against {x}"),
        (Some(one), None) | (None, Some(one)) => format!("{named} · {one}"),
        (None, None) => named,
    };
    let left = |name: &str| (name.to_string(), Align::Left);
    let right = |name: &str| (name.to_string(), Align::Right);
    let mut table = part(Vec::new(), Vec::new());
    table.title = title;
    let positions: Vec<&Scale> = scene
        .scales
        .iter()
        .filter(|s| s.range == Range::Position)
        .collect();
    if positions.is_empty() {
        table
            .after
            .push("no variable is mapped to `x` or `y`".to_string());
    } else {
        table.columns = vec![
            left("axis"),
            left("title"),
            left("scale"),
            left("limits"),
            left("breaks"),
        ];
        for s in positions {
            table.rows.push(vec![
                s.id.to_string(),
                s.title.clone(),
                s.kind.name().to_string(),
                format!(
                    "{} to {}",
                    format_float(s.limits.0),
                    format_float(s.limits.1)
                ),
                // The categories of an axis are listed as those of a colour are.
                if s.kind == ScaleKind::Category {
                    listed(&s.labels)
                } else {
                    s.labels.join(", ")
                },
            ]);
        }
    }
    let layers: Vec<&Layer> = scene.panels.iter().flat_map(|p| p.layers.iter()).collect();

    // The aesthetics other than the positions: a line for each that a variable is mapped to or
    // that is set to one value for every mark.
    let limits = |s: &Scale| {
        format!(
            "{} to {}",
            format_float(s.limits.0),
            format_float(s.limits.1)
        )
    };
    let mut lines: Vec<Vec<String>> = Vec::new();
    let mut unread: Vec<String> = Vec::new();
    for aesthetic in [
        Aesthetic::Color,
        Aesthetic::Fill,
        Aesthetic::Size,
        Aesthetic::Shape,
        Aesthetic::Linetype,
        Aesthetic::Group,
        Aesthetic::Label,
    ] {
        let id = aesthetic.name();
        let line = match (
            scene.scale(id),
            plot.constant(aesthetic),
            plot.mapping(aesthetic),
        ) {
            (Some(s), _, _) => {
                let (values, shown_as) = match (&s.range, s.kind) {
                    (Range::Colours(range), ScaleKind::Category) => (
                        listed(&s.labels),
                        listed(
                            &range
                                .iter()
                                .map(|hex| colour::name_of(hex).unwrap_or(hex).to_string())
                                .collect::<Vec<String>>(),
                        ),
                    ),
                    (Range::Colours(_), _) => {
                        (limits(s), "viridis, dark purple to yellow".to_string())
                    }
                    (Range::Sizes(_, largest), _) => (
                        limits(s),
                        format!("by area, up to {} across", format_float(*largest)),
                    ),
                    (Range::Shapes(range) | Range::LineTypes(range), _) => (
                        listed(&s.labels),
                        listed(
                            &range
                                .iter()
                                .map(|name| name.to_string())
                                .collect::<Vec<String>>(),
                        ),
                    ),
                    (Range::Position, _) => continue,
                };
                vec![
                    id.to_string(),
                    s.title.clone(),
                    s.kind.name().to_string(),
                    values,
                    shown_as,
                ]
            }
            (None, Some(setting), _) => vec![
                id.to_string(),
                String::new(),
                "constant".to_string(),
                "every mark".to_string(),
                match setting {
                    PlotSetting::Colour { written, .. } => written.clone(),
                    PlotSetting::Size(across) => format_float(*across),
                    PlotSetting::Shape(name) | PlotSetting::Line(name) => name.to_string(),
                    PlotSetting::Place(at) => format_float(*at),
                },
            ],
            // A group and a label have no scale: the values themselves are what a mark reads.
            (None, None, Some(mapping)) if aesthetic == Aesthetic::Group => vec![
                id.to_string(),
                axis_title(&plot.frame, mapping),
                String::new(),
                "each value a group".to_string(),
                String::new(),
            ],
            (None, None, Some(mapping)) if aesthetic == Aesthetic::Label => vec![
                id.to_string(),
                axis_title(&plot.frame, mapping),
                String::new(),
                "the text of a mark".to_string(),
                String::new(),
            ],
            _ => continue,
        };
        lines.push(line);
        // What the plot says of an aesthetic and no layer takes from it: its variable, where
        // the line above shows the scale of the plot's own variable, or the value it sets.
        let said = match scene.scale(id) {
            Some(_) => plot.mapping(aesthetic).is_some(),
            None => plot.mapping(aesthetic).is_some() || plot.constant(aesthetic).is_some(),
        };
        if said && !layers.is_empty() && !plot.layers.iter().any(|l| takes_from_plot(l, aesthetic))
        {
            // In the words of `points` while every layer is one; plainly otherwise.
            let only_points = plot.layers.iter().all(|l| l.mark == Mark::Points);
            unread.push(if only_points {
                match aesthetic {
                    Aesthetic::Color => {
                        "no layer draws `color`: every layer has a `color:` of its own"
                    }
                    Aesthetic::Fill => {
                        "no layer draws `fill`: `points` takes its colour from `color`"
                    }
                    Aesthetic::Size => {
                        "no layer draws `size`: every layer sets a `size:` of its own"
                    }
                    Aesthetic::Shape => {
                        "no layer draws `shape`: every layer has a `shape:` of its own"
                    }
                    Aesthetic::Group => "no layer draws `group`: `points` draws each row by itself",
                    Aesthetic::Label => "no layer draws `label`: `points` writes no text",
                    _ => continue,
                }
                .to_string()
            } else {
                format!("no layer draws `{id}`")
            });
        }
    }
    if !lines.is_empty() {
        let mut shown_as = part(
            vec![
                left("aesthetic"),
                left("title"),
                left("scale"),
                left("values"),
                left("shown as"),
            ],
            lines,
        );
        shown_as.after = unread;
        table.more.push(shown_as);
    }

    if layers.is_empty() {
        // The line goes under the last table printed.
        let last = table
            .more
            .last_mut()
            .map_or(&mut table.after, |t| &mut t.after);
        last.push("no layer yet: a mark such as `.points()` adds one".to_string());
        return table;
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    for (i, layer) in layers.iter().enumerate() {
        let written = plot.layers.get(i);
        let data = match written.and_then(|l| l.data.as_ref()) {
            Some(own) => own
                .name
                .clone()
                .unwrap_or_else(|| "its own frame".to_string()),
            None => shown.clone(),
        };
        let by = |aesthetic: Aesthetic| {
            written
                .and_then(|l| mapping_of(plot, l, aesthetic))
                .map_or(aesthetic.name(), |mapping| mapping.variable.as_str())
        };
        // What the layer names of its own comes first: its positions, then its colour.
        let mut own: Vec<String> = Vec::new();
        if let Some(l) = written {
            for aesthetic in [Aesthetic::X, Aesthetic::Y, Aesthetic::Low, Aesthetic::High] {
                // A reference line names its place by `at:`, a variable or a number.
                let name = if l.mark.spans() {
                    "at"
                } else {
                    aesthetic.name()
                };
                if let Some(mapping) = l.mapping(aesthetic) {
                    own.push(format!("{name} {}", mapping.variable));
                } else if let Some(PlotSetting::Place(at)) = l.constant(aesthetic) {
                    own.push(format!("{name} {}", format_float(*at)));
                }
            }
            for aesthetic in [Aesthetic::Color, Aesthetic::Fill] {
                match (l.mapping(aesthetic), l.constant(aesthetic)) {
                    (Some(mapping), _) => {
                        own.push(format!("{} by {}", aesthetic.name(), mapping.variable));
                    }
                    (None, Some(PlotSetting::Colour { written, .. })) => {
                        own.push(format!("{} {written}", aesthetic.name()));
                    }
                    _ => {}
                }
            }
        }
        let mut settings = if let Mark::Rug(sides) = layer.mark {
            // A rug: its sides, and how far its ticks let through what is under them.
            format!(
                "sides {}, alpha {}",
                sides.name(),
                format_float(layer.alpha)
            )
        } else if layer.mark.spans() {
            // A layer of reference lines: their width and their type.
            let style = match &layer.linetype {
                Lining::Constant(name) => name,
                Lining::Scaled(_) => LineType::Solid.name(),
            };
            format!("width {}, style {style}", format_float(layer.width))
        } else if layer.mark.joins() {
            // A layer of lines: their width, their type where it is not whole, how many; for
            // steps, the way they go where it is not level first.
            let mut settings = format!("width {}", format_float(layer.width));
            if let Mark::Step(way) = layer.mark
                && way != Stairs::LevelFirst
            {
                settings.push_str(&format!(", direction {}", way.name()));
            }
            match &layer.linetype {
                Lining::Constant(name) if *name == LineType::Solid.name() => {}
                Lining::Constant(name) => settings.push_str(&format!(", linetype {name}")),
                Lining::Scaled(_) => {
                    settings.push_str(&format!(", linetype by {}", by(Aesthetic::Linetype)));
                }
            }
            if let Some(mapping) = written.and_then(|l| l.mapping(Aesthetic::Group)) {
                settings.push_str(&format!(", group by {}", mapping.variable));
            }
            let lines = layer.line.last().copied().unwrap_or(0);
            settings.push_str(&format!(
                ", {} {}",
                count(lines as u64),
                if lines == 1 { "line" } else { "lines" }
            ));
            settings
        } else if let Mark::Bars(how) = layer.mark {
            // A layer of bars: their width, how they are arranged where it is not on one
            // another, how far they let through what is under them, their groups.
            let mut settings = format!("width {}", format_float(layer.width));
            if how != Arrangement::Stack {
                settings.push_str(&format!(", position {}", how.name()));
            }
            settings.push_str(&format!(", alpha {}", format_float(layer.alpha)));
            if let Some(mapping) = written.and_then(|l| l.mapping(Aesthetic::Group)) {
                settings.push_str(&format!(", group by {}", mapping.variable));
            }
            settings
        } else if layer.mark.fills() {
            // A filled layer: how far it lets through what is under it, its groups, how many.
            let mut settings = format!("alpha {}", format_float(layer.alpha));
            if let Mark::Area(placed) = layer.mark
                && placed != Stacking::Overlay
            {
                settings.push_str(&format!(", position {}", placed.name()));
            }
            if let Some(mapping) = written.and_then(|l| l.mapping(Aesthetic::Group)) {
                settings.push_str(&format!(", group by {}", mapping.variable));
            }
            let pieces = layer.line.last().copied().unwrap_or(0);
            let (one, many) = if layer.mark == Mark::Ribbon {
                ("band", "bands")
            } else {
                ("area", "areas")
            };
            settings.push_str(&format!(
                ", {} {}",
                count(pieces as u64),
                if pieces == 1 { one } else { many }
            ));
            settings
        } else {
            let mut settings = match &layer.size {
                Sizing::Constant(across) => format!("size {}", format_float(*across)),
                Sizing::Scaled(_) => format!("size by {}", by(Aesthetic::Size)),
            };
            // The shape is told where it is not the circle every plot begins with.
            match &layer.shape {
                Shaping::Constant(name) if *name == shape::DEFAULT => {}
                Shaping::Constant(name) => settings.push_str(&format!(", shape {name}")),
                Shaping::Scaled(_) => {
                    settings.push_str(&format!(", shape by {}", by(Aesthetic::Shape)));
                }
            }
            settings.push_str(&format!(", alpha {}", format_float(layer.alpha)));
            settings
        };
        if !own.is_empty() {
            settings = format!("{}, {settings}", own.join(", "));
        }
        if let Some(share) = written.map(|l| l.jitter).filter(|j| *j > 0.0) {
            settings.push_str(&format!(", jitter {}", format_float(share)));
        }
        rows.push(vec![
            (i + 1).to_string(),
            layer.mark.name().to_string(),
            data,
            count(layer.x.len().max(layer.y.len()) as u64),
            settings,
        ]);
    }
    table.more.push(part(
        vec![
            right("layer"),
            left("mark"),
            left("data"),
            right("marks"),
            left("settings"),
        ],
        rows,
    ));
    table
}

/// Write the printed form of a plot into it and return its scene: what every function that makes
/// or extends a plot does last (rule 5: `core` prints a plot without knowing the package).
pub fn describe(plot: &mut Plot) -> Scene {
    let scene = scene(plot);
    plot.summary = summary(plot, &scene);
    scene
}

/// A plot of `frame` with these mappings and constants and no layer yet, its printed form
/// written.
pub fn start(frame: Frame, mappings: Vec<PlotMapping>, constants: Vec<PlotConstant>) -> Plot {
    let mut plot = Plot {
        frame,
        mappings,
        constants,
        layers: Vec::new(),
        summary: Table {
            title: String::new(),
            before: Vec::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        },
        figure: Some(zeff_core::value::FigureWriter(crate::figure::write)),
    };
    describe(&mut plot);
    plot
}

/// The plot with one layer more on top, its printed form written, and its scene.
pub fn with_layer(plot: &Plot, layer: PlotLayer) -> (Plot, Scene) {
    let mut next = plot.clone();
    next.layers.push(layer);
    let scene = describe(&mut next);
    (next, scene)
}
