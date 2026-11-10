//! The scene of a plot (Language Reference, section 7.6, "Rendering and export"; Engine and
//! Project Specification, section 8.3, point 5): the marks in data coordinates with their
//! scales, guides and layout, in a versioned format.
//!
//! [`scene`] derives the scene from a plot value and is a pure function of it: the same plot
//! gives the same scene, bit for bit, on every target. The writers of SVG, PDF and PNG and Zemp
//! read the scene alone, never the plot value, so everything that decides what is drawn is
//! settled here: which rows are drawn and which are dropped and counted, where the jitter moves
//! each point, the limits and breaks of each axis and the title it carries.
//!
//! A scene has no size. It is drawn at the size of the pane or of the exported figure, and the
//! theme of the writer gives it its fonts, colours and margins.
//!
//! Version 1 holds one panel, the linear scales `x` and `y`, their axes and layers of points.

use zeff_columns::{Column, ColumnType, MetadataKey, Value as Cell};
use zeff_core::display::format_float;
use zeff_core::value::{Aesthetic, Align, Frame, Mark, Plot, PlotLayer, PlotMapping, Table};
use zeff_diagnostics::count;

use crate::json::Json;
use crate::scale;

/// The version of the scene format, the value of `zeff_scene` in its JSON. It rises when a
/// reader of the earlier version would misread the new one.
pub const SCENE_VERSION: u32 = 1;

/// The diameter of a point when `size:` is not given, in points of 1/72 inch: the default of
/// Observable Plot, a radius of 3 pixels at 96 to the inch.
pub const POINT_SIZE: f64 = 4.5;

/// A plot as it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// The grid of panels: its rows and columns.
    pub rows: usize,
    pub columns: usize,
    pub scales: Vec<Scale>,
    pub axes: Vec<Axis>,
    pub panels: Vec<Panel>,
    /// What the reader must be told about the data drawn: the rows dropped and why, each
    /// sentence once.
    pub notes: Vec<String>,
}

/// A scale: how the values of an aesthetic are placed, with the breaks a guide marks.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    /// The name the panels and guides refer to it by: `x`, `y`.
    pub id: &'static str,
    pub kind: ScaleKind,
    /// The title of its guide: the label of the variable, or its name; an expression as written.
    pub title: String,
    /// The two ends of the axis, in data coordinates.
    pub limits: (f64, f64),
    pub breaks: Vec<f64>,
    /// The text at each break.
    pub labels: Vec<String>,
}

/// The kinds of scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleKind {
    Linear,
}

impl ScaleKind {
    pub fn name(self) -> &'static str {
        match self {
            ScaleKind::Linear => "linear",
        }
    }
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
    /// The diameter of a point, in points of 1/72 inch.
    pub size: f64,
    /// The opacity, from 0 to 1.
    pub alpha: f64,
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
    missing: usize,
    infinite: usize,
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

/// The points of a layer: one for each row with a finite value of both positions, in the order
/// of the rows. A row with `NA` in either is counted as missing; a row with an infinite value,
/// which no axis can hold, is counted apart. A position that is not there, or is not numeric,
/// gives no point; the functions that build a plot refuse such a frame before it comes to this.
fn drawn(cx: Option<&Column>, cy: Option<&Column>) -> Drawn {
    let mut out = Drawn::default();
    let (Some(cx), Some(cy)) = (cx, cy) else {
        return out;
    };
    if !(is_numeric(cx) && is_numeric(cy)) || cx.len() != cy.len() {
        return out;
    }
    for row in 0..cx.len() {
        match (number(cx, row), number(cy, row)) {
            (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
                out.x.push(a);
                out.y.push(b);
            }
            (Some(_), Some(_)) => out.infinite += 1,
            _ => out.missing += 1,
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

/// `educ or wage`, the variables or expressions a dropped row may lack.
fn either(x: &str, y: &str) -> String {
    if x == y {
        x.to_string()
    } else {
        format!("{x} or {y}")
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

/// The scene of a plot.
pub fn scene(plot: &Plot) -> Scene {
    let x = plot.mapped(Aesthetic::X);
    let y = plot.mapped(Aesthetic::Y);
    let mut notes: Vec<String> = Vec::new();
    let mut note = |text: String| {
        if !notes.contains(&text) {
            notes.push(text);
        }
    };
    let mut layers: Vec<Layer> = Vec::new();
    for layer in &plot.layers {
        let mut points = match (x, y) {
            (Some(x), Some(y)) => {
                let points = drawn(
                    values(plot, Some(layer), Aesthetic::X),
                    values(plot, Some(layer), Aesthetic::Y),
                );
                if points.missing > 0 {
                    note(format!(
                        "{} with NA in {} dropped",
                        rows_text(points.missing),
                        either(x, y)
                    ));
                }
                if points.infinite > 0 {
                    note(format!(
                        "{} with an infinite value in {} dropped",
                        rows_text(points.infinite),
                        either(x, y)
                    ));
                }
                points
            }
            _ => Drawn::default(),
        };
        jitter(layer, &mut points);
        layers.push(Layer {
            mark: layer.mark,
            x: points.x,
            y: points.y,
            size: layer.size.unwrap_or(POINT_SIZE),
            alpha: layer.alpha,
        });
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
        let marks = extent(layers.iter().flat_map(|layer| match aesthetic {
            Aesthetic::X => layer.x.iter(),
            Aesthetic::Y => layer.y.iter(),
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
        });
        axes.push(Axis {
            scale: aesthetic.name(),
            position,
        });
    }
    Scene {
        rows: 1,
        columns: 1,
        scales,
        axes,
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
                    ("legends", Json::List(Vec::new())),
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
                                                    ("size", Json::Number(l.size)),
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

/// The scene read as text, the printed form of a plot: a line for each axis with its title, its
/// limits and its breaks, then a line for each layer with the number of marks it draws.
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
    let mut table = Table {
        title,
        before: Vec::new(),
        columns: Vec::new(),
        rows: Vec::new(),
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    };
    if scene.scales.is_empty() {
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
        for s in &scene.scales {
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
    if layers.is_empty() {
        table
            .after
            .push("no layer yet: a mark such as `.points()` adds one".to_string());
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
        let mut settings = format!(
            "size {}, alpha {}",
            format_float(layer.size),
            format_float(layer.alpha)
        );
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
    table.more.push(Table {
        title: String::new(),
        before: Vec::new(),
        columns: vec![
            right("layer"),
            left("mark"),
            left("data"),
            right("marks"),
            left("settings"),
        ],
        rows,
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    });
    table
}

/// Write the printed form of a plot into it and return its scene: what every function that makes
/// or extends a plot does last (rule 5: `core` prints a plot without knowing the package).
pub fn describe(plot: &mut Plot) -> Scene {
    let scene = scene(plot);
    plot.summary = summary(plot, &scene);
    scene
}

/// A plot of `frame` with these mappings and no layer yet, its printed form written.
pub fn start(frame: Frame, mappings: Vec<PlotMapping>) -> Plot {
    let mut plot = Plot {
        frame,
        mappings,
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
