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

use zeff_columns::{Column, ColumnType, MetadataKey, Value as Cell};
use zeff_core::display::{format_float, labelled_cell};
use zeff_core::value::{
    Aesthetic, Align, Frame, Mark, Plot, PlotConstant, PlotLayer, PlotMapping, PlotSetting, Table,
};
use zeff_diagnostics::count;

use crate::colour;
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
/// text of the marks that write one; those marks come at later steps.
pub fn reads(mark: Mark, aesthetic: Aesthetic) -> bool {
    match mark {
        Mark::Points => matches!(
            aesthetic,
            Aesthetic::X | Aesthetic::Y | Aesthetic::Color | Aesthetic::Size | Aesthetic::Shape
        ),
    }
}

/// Whether a layer takes an aesthetic from the plot: its mark reads it, and the layer does not
/// say otherwise. A layer with a `size:` of its own does not read the plot's.
pub fn layer_reads(layer: &PlotLayer, aesthetic: Aesthetic) -> bool {
    reads(layer.mark, aesthetic) && !(aesthetic == Aesthetic::Size && layer.size.is_some())
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
/// frame. `None` when nothing is mapped, or when the frame has no such variable.
fn values<'a>(
    plot: &'a Plot,
    layer: Option<&'a PlotLayer>,
    aesthetic: Aesthetic,
) -> Option<&'a Column> {
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
/// drawn on.
fn frame_extent(plot: &Plot, aesthetic: Aesthetic) -> Option<(f64, f64)> {
    let column = values(plot, None, aesthetic)?;
    let values = column.numeric_values()?.values;
    extent(values.iter())
}

/// The title of an axis: the label of its variable in the plot's frame, or the variable's name;
/// for an expression, the expression as written.
fn axis_title(plot: &Plot, mapping: &PlotMapping) -> String {
    if mapping.computed.is_some() {
        return mapping.variable.clone();
    }
    plot.frame
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
    mapping: &'a PlotMapping,
    /// The variable in the plot's frame.
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
    let mapping = plot.mapping(aesthetic)?;
    let base = values(plot, None, aesthetic)?;
    let kind = shading(base)?;
    match (aesthetic, kind) {
        (Aesthetic::Size, Shading::Categories) | (Aesthetic::Shape, Shading::Ramp) => return None,
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
    let title = axis_title(plot, colouring.mapping);
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
            let mut position = vec![0_usize; labels.len()];
            let mut shown: Vec<String> = Vec::new();
            let mut colours: Vec<&'static str> = Vec::new();
            for (place, label) in labels.iter().enumerate() {
                if used[place] {
                    colours.push(if shapes {
                        shape::category(shown.len())
                    } else {
                        colour::category(shown.len()).0
                    });
                    shown.push(label.clone());
                    position[place] = shown.len();
                }
            }
            let named = shown.len();
            if na {
                colours.push(if shapes {
                    shape::NA_SHAPE
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
            } else {
                colour::OKABE_ITO.len()
            };
            let note = (read && named > palette).then(|| {
                let (variable, named) = (&colouring.mapping.variable, count(named as u64));
                if shapes {
                    format!(
                        "`{variable}` has {named} categories and there are {palette} shapes, so the shapes repeat from the seventh category"
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
    let x = plot.mapped(Aesthetic::X);
    let y = plot.mapped(Aesthetic::Y);
    let colourings: Vec<Colouring<'_>> = [
        Aesthetic::Color,
        Aesthetic::Fill,
        Aesthetic::Size,
        Aesthetic::Shape,
    ]
    .into_iter()
    .filter_map(|aesthetic| colouring(plot, aesthetic))
    .collect();
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
        let mut points = match (x, y) {
            (Some(x), Some(y)) => {
                // A value on a ramp of colours must be there and finite, as a position must; a
                // row without a category is drawn, in the colour of `NA`.
                let read = |colouring: &Colouring<'a>| {
                    colouring.readers[at]
                        .map(|column| (colouring.mapping.variable.as_str(), column))
                };
                let ramps: Vec<(&str, &Column)> = colourings
                    .iter()
                    .filter(|colouring| {
                        colouring.kind == Shading::Ramp && colouring.aesthetic != Aesthetic::Size
                    })
                    .filter_map(read)
                    .collect();
                // A size must be there and finite as well, and not below 0.
                let sized = colourings
                    .iter()
                    .filter(|colouring| colouring.aesthetic == Aesthetic::Size)
                    .find_map(read);
                let columns: Vec<&Column> = ramps.iter().map(|(_, column)| *column).collect();
                let points = drawn(
                    values(plot, Some(layer), Aesthetic::X),
                    values(plot, Some(layer), Aesthetic::Y),
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
        };
        jitter(layer, &mut points);
        // A constant colour, for a mark that has one; a mapped colour takes its place below,
        // when its scale is known.
        let color = match plot.constant(Aesthetic::Color) {
            Some(PlotSetting::Colour { hex, .. }) if reads(layer.mark, Aesthetic::Color) => {
                Paint::Constant(hex.clone())
            }
            _ => Paint::Theme,
        };
        // The layer's own size comes before the plot's; a mapped size and a mapped shape take
        // their place below, as a mapped colour does.
        let size = match (layer.size, plot.constant(Aesthetic::Size)) {
            (Some(own), _) => own,
            (None, Some(PlotSetting::Size(all))) if reads(layer.mark, Aesthetic::Size) => *all,
            _ => POINT_SIZE,
        };
        let outline = match plot.constant(Aesthetic::Shape) {
            Some(PlotSetting::Shape(name)) if reads(layer.mark, Aesthetic::Shape) => *name,
            _ => shape::DEFAULT,
        };
        layers.push(Layer {
            mark: layer.mark,
            x: points.x,
            y: points.y,
            color,
            size: Sizing::Constant(size),
            shape: Shaping::Constant(outline),
            alpha: layer.alpha,
        });
        kept.push(points.rows);
    }
    let mut scales: Vec<Scale> = Vec::new();
    let mut axes: Vec<Axis> = Vec::new();
    for (aesthetic, position) in [
        (Aesthetic::X, Position::Bottom),
        (Aesthetic::Y, Position::Left),
    ] {
        let Some(mapping) = plot.mapping(aesthetic) else {
            continue;
        };
        let marks = extent(layers.iter().flat_map(|layer| {
            if aesthetic == Aesthetic::X {
                layer.x.iter()
            } else {
                layer.y.iter()
            }
        }));
        let (lo, hi) = scale::limits(marks.or_else(|| frame_extent(plot, aesthetic)));
        let breaks = scale::breaks(lo, hi);
        let labels = scale::labels(&breaks);
        scales.push(Scale {
            id: aesthetic.name(),
            kind: ScaleKind::Linear,
            title: axis_title(plot, mapping),
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
                Aesthetic::Size => layer.size = Sizing::Scaled(values),
                Aesthetic::Shape => layer.shape = Shaping::Scaled(values),
                _ => {}
            }
        }
        if let Some(said) = said {
            note(said);
        }
        scales.push(scale);
    }
    Scene {
        rows: 1,
        columns: 1,
        scales,
        axes,
        legends,
        panels: vec![Panel {
            row: 1,
            column: 1,
            x: x.map(|_| Aesthetic::X.name()),
            y: y.map(|_| Aesthetic::Y.name()),
            layers,
        }],
        notes,
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
        Json::Object(vec![
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
                                        Range::Colours(names) | Range::Shapes(names) => Json::List(
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
                                                l.scales.iter().map(|s| Json::text(*s)).collect(),
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
                            Json::Object(vec![
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
                                                Json::Object(vec![
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
                                                            Sizing::Constant(across) => {
                                                                Json::Number(*across)
                                                            }
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
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("notes", Json::texts(&self.notes)),
        ])
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
    let title = match (plot.mapped(Aesthetic::X), plot.mapped(Aesthetic::Y)) {
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
                s.labels.join(", "),
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
                    (Range::Shapes(range), _) => (
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
                    PlotSetting::Shape(name) => name.to_string(),
                },
            ],
            // A group and a label have no scale: the values themselves are what a mark reads.
            (None, None, Some(mapping)) if aesthetic == Aesthetic::Group => vec![
                id.to_string(),
                axis_title(plot, mapping),
                String::new(),
                "each value a group".to_string(),
                String::new(),
            ],
            (None, None, Some(mapping)) if aesthetic == Aesthetic::Label => vec![
                id.to_string(),
                axis_title(plot, mapping),
                String::new(),
                "the text of a mark".to_string(),
                String::new(),
            ],
            _ => continue,
        };
        lines.push(line);
        if !layers.is_empty() && !plot.layers.iter().any(|l| layer_reads(l, aesthetic)) {
            unread.push(
                match aesthetic {
                    Aesthetic::Fill => {
                        "no layer draws `fill`: `points` takes its colour from `color`"
                    }
                    Aesthetic::Size => {
                        "no layer draws `size`: every layer sets a `size:` of its own"
                    }
                    Aesthetic::Group => "no layer draws `group`: `points` draws each row by itself",
                    Aesthetic::Label => "no layer draws `label`: `points` writes no text",
                    _ => continue,
                }
                .to_string(),
            );
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
        let by = |aesthetic: Aesthetic| plot.mapped(aesthetic).unwrap_or(aesthetic.name());
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
        if let Some(share) = written.map(|l| l.jitter).filter(|j| *j > 0.0) {
            settings.push_str(&format!(", jitter {}", format_float(share)));
        }
        rows.push(vec![
            (i + 1).to_string(),
            layer.mark.name().to_string(),
            data,
            count(layer.x.len() as u64),
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
