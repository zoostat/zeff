//! The layout of a figure (Language Reference, section 7.6, "Defaults" and "Rendering and
//! export"; release 0.7.0, step 21b): a scene and a size become a drawing, the list of
//! primitives a writer turns into a file ([`crate::drawing`]).
//!
//! A scene has no size; here it is given one. The page is `one_column`, `two_column` or
//! `slide`, or a width and a height in millimetres. Inside a margin stand, from the left: the
//! title of the y axis, read upwards; the labels of its breaks; the panel; and, at the right,
//! the legends, one under the other, a legend that does not fit under the one before it
//! starting a new column. Under the panel stand the labels of the x axis and its
//! title. What is left for the panel is what the titles, labels and legends do not take, so
//! that a longer label makes the panel smaller and never runs off the page. Every width is
//! measured from the embedded fonts ([`crate::font`]), so the same scene at the same size is
//! the same drawing on every machine.
//!
//! The journal theme: a white page, the major gridlines alone in a light grey, the two axes
//! as thin dark lines with their ticks outside the panel, text in IBM Plex Sans, 8 points for
//! labels and 9 for titles, and no frame round the panel or the legend. No text is smaller
//! than 8 points ([`SMALLEST_TEXT`]). Its constants are in one place, [`JOURNAL`]. The owner
//! read them on the six sample figures and froze them on 12 November 2026 (step 22): they,
//! the three named sizes and the stroke of an outline shape change only by a decision of his,
//! and a test holds them.
//!
//! A title that is longer than the side it runs along is broken between words. Labels of
//! breaks that would touch are thinned, every second or third being written; the ticks and
//! gridlines stay. Marks are placed by their scales and clipped to the panel. The notes of the
//! scene (the rows not drawn, and why) are not part of the figure; [`layout_with_notes`]
//! writes them under it, for `export(notes: true)`.
//!
//! A drawing holds no row of the data (open decision #118, F2): of a mark it keeps the place,
//! the shape, the size and the colour.

use zeff_core::value::{LegendPlace, Mark, Stairs};

use crate::colour;
use crate::drawing::{Anchor, Drawing, Head, Item, LineType, Primitive, Span, Step, Stroke, rgb};
use crate::font::{FaceName, face};
use crate::raster;
use crate::scale;
use crate::scene::{
    Legend, Lining, Paint, Range, Scale, ScaleKind, Scene, Shaping, Sizing, diameter,
};
use crate::shape;

/// Points of 1/72 inch to the millimetre.
pub const MM: f64 = 72.0 / 25.4;

/// The smallest type a figure is written in, in points (section 7.6, "Text").
pub const SMALLEST_TEXT: f64 = 8.0;

/// The smallest panel a figure is drawn with, in points each way: below it the page is too
/// small for its labels.
pub const SMALLEST_PANEL: f64 = 36.0;

/// The size of the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// One column of a journal page: 89 mm wide, 66 mm high.
    OneColumn,
    /// The full width of a journal page: 183 mm wide, 110 mm high.
    TwoColumn,
    /// A slide of 16 to 9: 254 mm wide, 143 mm high.
    Slide,
    /// A width and a height in millimetres.
    Millimetres(f64, f64),
}

impl Size {
    /// The width and the height, in millimetres.
    pub fn millimetres(self) -> (f64, f64) {
        match self {
            Size::OneColumn => (89.0, 66.0),
            Size::TwoColumn => (183.0, 110.0),
            Size::Slide => (254.0, 143.0),
            Size::Millimetres(width, height) => (width, height),
        }
    }

    /// The width and the height, in points: 72 to the inch of 25.4 mm, multiplied before it
    /// is divided, so that 254 mm is 720 points exactly.
    pub fn points(self) -> (f64, f64) {
        let (width, height) = self.millimetres();
        (width * 72.0 / 25.4, height * 72.0 / 25.4)
    }
}

/// What a theme fixes: colours as `#RRGGBB`, lengths in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    /// The page.
    pub background: &'static str,
    /// Text, the axes and their ticks.
    pub ink: &'static str,
    /// The gridlines.
    pub grid: &'static str,
    /// A mark that has no colour of its own.
    pub mark: &'static str,
    /// The type of the labels of breaks and of legend keys.
    pub label_size: f64,
    /// The type of the titles of axes and legends.
    pub title_size: f64,
    /// From one baseline to the next, as a multiple of the size of the type.
    pub leading: f64,
    pub axis_width: f64,
    pub grid_width: f64,
    pub tick_length: f64,
    /// Round the whole figure.
    pub margin: f64,
    /// Between a tick and its label, and between a key and its label.
    pub label_gap: f64,
    /// Between the labels of an axis and its title.
    pub title_gap: f64,
    /// Between the panel and the legends, and between two legends.
    pub legend_gap: f64,
    /// The key of a legend whose marks are of the default size.
    pub key_size: f64,
    /// The bar of a continuous colour: across and along.
    pub ramp_width: f64,
    pub ramp_length: f64,
}

/// The journal theme, frozen by the owner on 12 November 2026 (release 0.7.0, step 22).
pub const JOURNAL: Theme = Theme {
    name: "journal",
    background: "#FFFFFF",
    ink: "#222222",
    grid: "#E3E3E3",
    mark: "#222222",
    label_size: 8.0,
    title_size: 9.0,
    leading: 1.25,
    axis_width: 0.5,
    grid_width: 0.35,
    tick_length: 3.0,
    margin: 4.0,
    label_gap: 3.0,
    title_gap: 5.0,
    legend_gap: 10.0,
    key_size: 4.5,
    ramp_width: 8.0,
    ramp_length: 72.0,
};

// The theme writes no text below 8 points, or the engine does not build.
const _: () = assert!(
    JOURNAL.label_size >= SMALLEST_TEXT
        && JOURNAL.title_size >= SMALLEST_TEXT
        && JOURNAL.leading >= 1.0
);

/// Why a scene cannot be laid out.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    /// The scene holds what this version does not draw: the sentence of [`Scene::undrawn`].
    Undrawn(String),
    /// A width or a height that is not a length above zero.
    NotASize,
    /// The page leaves the panel less than [`SMALLEST_PANEL`] each way: what it leaves, in
    /// points.
    TooSmall { panel: (f64, f64) },
}

/// Measures text and remembers the characters no face has.
#[derive(Default)]
struct Setter {
    lacking: Vec<char>,
}

impl Setter {
    /// The width of `text` in `name` at `size` points.
    fn width(&mut self, text: &str, name: FaceName, size: f64) -> f64 {
        let shaped = face(name).shape(text);
        for character in shaped.missing {
            if !self.lacking.contains(&character) {
                self.lacking.push(character);
            }
        }
        f64::from(shaped.width) * size / f64::from(face(name).units_per_em())
    }

    /// `text` broken between words into lines no wider than `limit`; a word wider than the
    /// limit stands alone on its line.
    fn wrap(&mut self, text: &str, name: FaceName, size: f64, limit: f64) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for word in text.split(' ').filter(|word| !word.is_empty()) {
            let longer = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if line.is_empty() || self.width(&longer, name, size) <= limit {
                line = longer;
            } else {
                lines.push(std::mem::take(&mut line));
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }
}

/// The height of a capital letter of the labels' face at `size` points.
fn capital(size: f64) -> f64 {
    let sans = face(FaceName::Sans);
    f64::from(sans.cap_height()) * size / f64::from(sans.units_per_em())
}

fn text(
    at: (f64, f64),
    anchor: Anchor,
    turn: f64,
    words: &str,
    size: f64,
    theme: &Theme,
) -> Primitive {
    Primitive::Text {
        at,
        anchor,
        turn,
        spans: vec![Span {
            text: words.to_string(),
            face: FaceName::Sans,
            size,
            colour: theme.ink.to_string(),
        }],
        frame: None,
    }
}

fn named(name: &str, primitive: Primitive) -> Item {
    Item {
        name: Some(name.to_string()),
        primitive,
    }
}

/// How many labels to pass over from one written to the next, so that no two written touch:
/// 1 writes them all. `centres` are their places along the axis and `extents` how much room
/// each takes along it.
fn thinning(centres: &[f64], extents: &[f64], apart: f64) -> usize {
    let fits = |step: usize| {
        let kept: Vec<usize> = (0..centres.len()).step_by(step).collect();
        kept.windows(2).all(|pair| {
            let (a, b) = (pair[0], pair[1]);
            (centres[b] - centres[a]).abs() >= (extents[a] + extents[b]) / 2.0 + apart
        })
    };
    (1..=centres.len().max(1))
        .find(|&step| fits(step))
        .unwrap_or(1)
}

/// The colour a value takes on a scale of colours.
fn colour_on(scale: &Scale, value: f64) -> String {
    let Range::Colours(colours) = &scale.range else {
        return colour::NA_COLOUR.0.to_string();
    };
    if colours.is_empty() {
        return colour::NA_COLOUR.0.to_string();
    }
    let entry = match scale.kind {
        ScaleKind::Category => (value as usize).saturating_sub(1).min(colours.len() - 1),
        _ => colour::ramp_entry(value, scale.limits.0, scale.limits.1, colours.len()),
    };
    colours[entry].to_string()
}

/// The shape a place takes on the scale of shapes.
fn shape_on(scale: &Scale, value: f64) -> &'static str {
    match &scale.range {
        Range::Shapes(shapes) if !shapes.is_empty() => {
            shapes[(value as usize).saturating_sub(1).min(shapes.len() - 1)]
        }
        _ => shape::DEFAULT,
    }
}

/// The type of line of a value's place on a scale of types; whole for what is no place.
fn line_on(scale: &Scale, value: f64) -> LineType {
    match &scale.range {
        Range::LineTypes(types) if !types.is_empty() => {
            let place = (value.round().max(1.0) as usize - 1).min(types.len() - 1);
            LineType::parse(types[place]).unwrap_or_default()
        }
        _ => LineType::Solid,
    }
}

/// How long the piece of line that is a legend key is, in points: two rounds of the longest
/// of the six rhythms at the width of a line.
const KEY_LINE: f64 = 24.0;

/// The patch of fill that is the key of a filled mark in a legend: its width where no piece of
/// line makes the key wider, and its height.
const KEY_PATCH: (f64, f64) = (12.0, 8.0);

/// One key of a legend: a mark and what it stands for.
struct Key {
    label: String,
    shape: &'static str,
    size: f64,
    colour: String,
    /// For a legend of lines: the type and the width of the piece of line that is the key.
    line: Option<(LineType, f64)>,
    /// Whether the key shows a point: the legend is of marks drawn one by one.
    point: bool,
    /// For a legend of filled marks: the colour and the opacity of the patch that is the key.
    patch: Option<(String, f64)>,
}

/// What a legend shows, measured.
enum Content {
    /// Keys, in columns of `rows` keys.
    Keys {
        keys: Vec<Key>,
        rows: usize,
        /// The room each key and its label take: how wide the column of marks is, how wide
        /// each column of labels, and the height of a row.
        mark_width: f64,
        label_widths: Vec<f64>,
        row_height: f64,
    },
    /// A bar of colours with the labels of its breaks at their share of the way along it:
    /// standing, from the largest value at the top, or lying (`across`), from the smallest
    /// value at the left, for a legend above or under the panel.
    Ramp {
        colours: Vec<String>,
        length: f64,
        ticks: Vec<(f64, String)>,
        across: bool,
    },
}

struct LegendPlan {
    name: String,
    title: Vec<String>,
    content: Content,
    width: f64,
    height: f64,
}

/// Between two columns of keys.
const COLUMN_GAP: f64 = 8.0;

/// A legend, measured. `across` is the width it may take where it stands above or under the
/// panel: its keys are then laid in as few rows as that width holds, and the bar of a ramp
/// lies down. Without it the legend stands at the right, its keys in columns as tall as the
/// page holds.
fn plan_legend(
    scene: &Scene,
    legend: &Legend,
    theme: &Theme,
    set: &mut Setter,
    page: (f64, f64),
    across: Option<f64>,
) -> LegendPlan {
    let scales: Vec<&Scale> = legend
        .scales
        .iter()
        .filter_map(|id| scene.scale(id))
        .collect();
    let of = |id: &str| scales.iter().copied().find(|s| s.id == id);
    // What the marks of the legend look like where no scale of the legend says: the
    // constants of the first layer drawn.
    let layer = scene
        .panels
        .first()
        .and_then(|p| p.layers.iter().find(|l| legend.marks.contains(&l.mark)));
    let base_colour = match layer.map(|l| &l.color) {
        Some(Paint::Constant(hex)) => hex.clone(),
        _ => theme.mark.to_string(),
    };
    let base_shape = match layer.map(|l| &l.shape) {
        Some(Shaping::Constant(name)) => *name,
        _ => shape::DEFAULT,
    };
    // A legend of lines has a piece of line for a key, and a point on it when points read
    // the scale too.
    let pointed = legend.marks.iter().any(|mark| {
        !mark.joins() && !mark.patches() && !mark.spans() && !matches!(mark, Mark::Rug(_))
    });
    // The ticks of a rug are small lines, and its key a piece of line, unless marks of
    // another kind share the legend: a rug under points changes nothing in their key.
    let rug_alone = legend.marks.iter().all(|mark| matches!(mark, Mark::Rug(_)));
    let lined = scene.panels.first().and_then(|p| {
        p.layers.iter().find(|l| {
            (l.mark.joins() || l.mark.spans() || (rug_alone && matches!(l.mark, Mark::Rug(_))))
                && legend.marks.contains(&l.mark)
        })
    });
    // A legend of filled marks has a patch of the fill for a key, as see-through as the marks.
    let filled = scene.panels.first().and_then(|p| {
        p.layers
            .iter()
            .find(|l| l.mark.patches() && legend.marks.contains(&l.mark))
    });
    let base_patch = filled.map(|l| {
        (
            match &l.fill {
                Paint::Constant(hex) => hex.clone(),
                _ => theme.mark.to_string(),
            },
            l.alpha,
        )
    });
    let fills = of("fill");
    let base_line = lined.map(|l| {
        (
            match &l.linetype {
                Lining::Constant(name) => LineType::parse(name).unwrap_or_default(),
                Lining::Scaled(_) => LineType::Solid,
            },
            l.width,
        )
    });
    let line_types = of("linetype");
    // A title no wider than a third of the page.
    let title = set.wrap(
        &legend.title,
        FaceName::Sans,
        theme.title_size,
        (page.0 / 3.0).max(48.0),
    );
    let title_width = title
        .iter()
        .map(|line| set.width(line, FaceName::Sans, theme.title_size))
        .fold(0.0, f64::max);
    let title_height = if title.is_empty() {
        0.0
    } else {
        title.len() as f64 * theme.title_size * theme.leading + theme.label_gap
    };
    let room = (page.1 - 2.0 * theme.margin - capital(theme.label_size) - title_height).max(1.0);
    let colours = of("color");
    let sizes = of("size");
    let shapes = of("shape");
    let categories = scales
        .iter()
        .copied()
        .find(|s| s.kind == ScaleKind::Category);
    let keys: Option<Vec<Key>> = if let Some(named) = categories {
        Some(
            named
                .labels
                .iter()
                .enumerate()
                .map(|(i, label)| {
                    let place = (i + 1) as f64;
                    Key {
                        label: label.clone(),
                        shape: shapes.map_or(base_shape, |s| shape_on(s, place)),
                        size: theme.key_size,
                        colour: match colours {
                            Some(s) if s.kind == ScaleKind::Category => colour_on(s, place),
                            _ => base_colour.clone(),
                        },
                        line: base_line.map(|(whole, width)| {
                            (line_types.map_or(whole, |s| line_on(s, place)), width)
                        }),
                        point: pointed,
                        patch: base_patch.clone().map(|(whole, opacity)| {
                            (
                                match fills {
                                    Some(s) if s.kind == ScaleKind::Category => colour_on(s, place),
                                    _ => whole,
                                },
                                opacity,
                            )
                        }),
                    }
                })
                .collect(),
        )
    } else {
        sizes.map(|sized| {
            let range = match sized.range {
                Range::Sizes(least, most) => (least, most),
                _ => (0.0, theme.key_size),
            };
            sized
                .breaks
                .iter()
                .zip(&sized.labels)
                .map(|(value, label)| Key {
                    label: label.clone(),
                    shape: base_shape,
                    size: diameter(*value, sized.limits, range),
                    colour: colours.map_or(base_colour.clone(), |s| colour_on(s, *value)),
                    line: None,
                    point: true,
                    patch: None,
                })
                .collect()
        })
    };
    // The bar of a ramp is that of the colours, or of the fills where no colour has one.
    let (content, width, height) = match (keys, colours.or(fills)) {
        (Some(keys), _) => {
            let line = theme.label_size * theme.leading;
            // A key is as tall as its point or its patch, and as wide as its point, its patch
            // or its piece of line.
            let tallest_mark = keys
                .iter()
                .map(|k| {
                    if k.patch.is_some() {
                        k.size.max(KEY_PATCH.1)
                    } else {
                        k.size
                    }
                })
                .fold(theme.key_size, f64::max);
            let widest_mark = keys
                .iter()
                .map(|k| {
                    if k.line.is_some() {
                        KEY_LINE
                    } else if k.patch.is_some() {
                        KEY_PATCH.0
                    } else {
                        k.size
                    }
                })
                .fold(tallest_mark, f64::max);
            let row_height = line.max(tallest_mark + 2.0);
            let most = ((room / row_height).floor() as usize).clamp(1, keys.len().max(1));
            let labels: Vec<f64> = keys
                .iter()
                .map(|k| set.width(&k.label, FaceName::Sans, theme.label_size))
                .collect();
            // The widths of the columns of labels, and of the whole, with `rows` keys to a
            // column.
            let laid = |rows: usize| -> (Vec<f64>, f64) {
                let label_widths: Vec<f64> = labels
                    .chunks(rows)
                    .map(|column| column.iter().copied().fold(0.0, f64::max))
                    .collect();
                let width = label_widths
                    .iter()
                    .map(|w| widest_mark + theme.label_gap + w)
                    .sum::<f64>()
                    + COLUMN_GAP * label_widths.len().saturating_sub(1) as f64;
                (label_widths, width)
            };
            // At the right: columns as tall as the page holds. Above or under the panel: as
            // few rows as the width holds, and as many as the page where none does.
            let rows = match across {
                None => most,
                Some(wide) => (1..=most)
                    .find(|&rows| laid(rows).1 <= wide)
                    .unwrap_or(most),
            };
            let (label_widths, width) = laid(rows);
            let height = rows.min(keys.len()) as f64 * row_height;
            (
                Content::Keys {
                    keys,
                    rows,
                    mark_width: widest_mark,
                    label_widths,
                    row_height,
                },
                width,
                height,
            )
        }
        (None, Some(ramp)) => {
            let lying = across.is_some();
            let mut colours: Vec<String> = match &ramp.range {
                Range::Colours(list) => list.clone(),
                _ => Vec::new(),
            };
            // Standing, the largest value is at the top.
            if !lying {
                colours.reverse();
            }
            let length = match across {
                Some(wide) => theme.ramp_length.min(wide),
                None => theme.ramp_length.min(room),
            };
            let (lo, hi) = ramp.limits;
            let ticks: Vec<(f64, String)> = ramp
                .breaks
                .iter()
                .zip(&ramp.labels)
                .filter(|(value, _)| hi > lo && **value >= lo && **value <= hi)
                .map(|(value, label)| {
                    let up = (value - lo) / (hi - lo);
                    (if lying { up } else { 1.0 - up }, label.clone())
                })
                .collect();
            let widths: Vec<f64> = ticks
                .iter()
                .map(|(_, label)| set.width(label, FaceName::Sans, theme.label_size))
                .collect();
            let widest = widths.iter().copied().fold(0.0, f64::max);
            let (width, height) = if lying {
                // Half a label beyond each end of the bar, for the labels at its ends.
                (
                    length + widest,
                    theme.ramp_width
                        + theme.tick_length
                        + theme.label_gap
                        + capital(theme.label_size),
                )
            } else {
                // Half a label above the bar and half below.
                (
                    theme.ramp_width + theme.tick_length + theme.label_gap + widest,
                    length + capital(theme.label_size),
                )
            };
            (
                Content::Ramp {
                    colours,
                    length,
                    ticks,
                    across: lying,
                },
                width,
                height,
            )
        }
        (None, None) => (
            Content::Keys {
                keys: Vec::new(),
                rows: 1,
                mark_width: 0.0,
                label_widths: Vec::new(),
                row_height: 0.0,
            },
            0.0,
            0.0,
        ),
    };
    LegendPlan {
        name: format!("legend:{}", legend.scales.join("+")),
        title,
        content,
        width: width.max(title_width),
        height: title_height + height,
    }
}

/// The name of a line, written at its end in the place of a key of a legend
/// (`legend(direct)`).
struct EndLabel {
    /// The legend whose key it stands for, by the name of its items.
    name: String,
    text: String,
    colour: String,
    /// Where the line ends along `y`, in data coordinates.
    y: f64,
}

/// What `legend(direct)` writes: for each legend of categories that a layer of lines reads by
/// its colour or its type of line, the name of each category at the end of its line, in the
/// colour of the line; and, for each legend of the scene, whether its keys are replaced so.
/// A category drawn as several lines is named once, at the line that ends furthest along
/// `x`. A legend no line reads, the bar of a continuous scale, the keys of points or of
/// sizes, is not replaced, and stands at the right.
fn end_labels(scene: &Scene, theme: &Theme) -> (Vec<bool>, Vec<EndLabel>) {
    let mut replaced = vec![false; scene.legends.len()];
    let mut labels: Vec<EndLabel> = Vec::new();
    let layers: &[crate::scene::Layer] = scene.panels.first().map_or(&[], |p| &p.layers[..]);
    for (at, legend) in scene.legends.iter().enumerate() {
        let categories = |id: &str| {
            legend
                .scales
                .contains(&id)
                .then(|| scene.scale(id))
                .flatten()
                .filter(|scale| scale.kind == ScaleKind::Category)
        };
        let (colours, types) = (categories("color"), categories("linetype"));
        // The first layer of lines that reads one of the two scales, and the place of each
        // of its marks among the categories.
        let read = layers
            .iter()
            .filter(|layer| layer.mark.joins())
            .find_map(
                |layer| match (&layer.color, &layer.linetype, colours, types) {
                    (Paint::Scaled(places), _, Some(scale), _) => Some((layer, places, scale)),
                    (_, Lining::Scaled(places), _, Some(scale)) => Some((layer, places, scale)),
                    _ => None,
                },
            );
        let Some((layer, places, scale)) = read else {
            continue;
        };
        let whole = match &layer.color {
            Paint::Constant(hex) => hex.clone(),
            _ => theme.mark.to_string(),
        };
        // For each category, the end of its line that reaches furthest along `x`.
        let mut ends: Vec<Option<(f64, f64, f64)>> = vec![None; scale.labels.len()];
        let marks = layer
            .x
            .len()
            .min(layer.y.len())
            .min(layer.line.len())
            .min(places.len());
        let mut start = 0;
        while start < marks {
            let mut end = start + 1;
            while end < marks && layer.line[end] == layer.line[start] {
                end += 1;
            }
            let last = end - 1;
            let place = places[last];
            let slot = (place as usize).saturating_sub(1);
            if place >= 1.0
                && let Some(found) = ends.get_mut(slot)
                && found.is_none_or(|(x, _, _)| layer.x[last] > x)
            {
                *found = Some((layer.x[last], layer.y[last], place));
            }
            start = end;
        }
        let name = format!("legend:{}", legend.scales.join("+"));
        for (slot, end) in ends.iter().enumerate() {
            let Some((_, y, place)) = end else {
                continue;
            };
            labels.push(EndLabel {
                name: name.clone(),
                text: scale.labels[slot].clone(),
                colour: match colours {
                    Some(scale) => colour_on(scale, *place),
                    None => whole.clone(),
                },
                y: *y,
            });
        }
        replaced[at] = true;
    }
    (replaced, labels)
}

/// The heights at which names are written beside a panel from `top` to `bottom`, each as
/// near as may be to the height asked for, `wanted`, and no two nearer than `apart`: the
/// names keep their order down the page, and where they would touch they are moved apart,
/// downwards first and then, where the foot of the panel is passed, back up.
fn spread(wanted: &[f64], apart: f64, top: f64, bottom: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..wanted.len()).collect();
    order.sort_by(|a, b| wanted[*a].total_cmp(&wanted[*b]));
    let mut placed: Vec<f64> = order
        .iter()
        .map(|i| wanted[*i].clamp(top.min(bottom), bottom.max(top)))
        .collect();
    for i in 1..placed.len() {
        placed[i] = placed[i].max(placed[i - 1] + apart);
    }
    if let Some(last) = placed.last_mut() {
        *last = last.min(bottom.max(top));
    }
    for i in (0..placed.len().saturating_sub(1)).rev() {
        placed[i] = placed[i].min(placed[i + 1] - apart);
    }
    let mut out = vec![0.0; wanted.len()];
    for (at, i) in order.into_iter().enumerate() {
        out[i] = placed[at];
    }
    out
}

/// The items of a legend with its top left corner at `at`.
fn draw_legend(plan: &LegendPlan, at: (f64, f64), theme: &Theme, items: &mut Vec<Item>) {
    let (left, mut y) = at;
    for line in &plan.title {
        y += theme.title_size * theme.leading;
        items.push(named(
            &plan.name,
            text(
                (left, y - theme.title_size * (theme.leading - 1.0)),
                Anchor::Start,
                0.0,
                line,
                theme.title_size,
                theme,
            ),
        ));
    }
    if !plan.title.is_empty() {
        y += theme.label_gap;
    }
    let half_capital = capital(theme.label_size) / 2.0;
    match &plan.content {
        Content::Keys {
            keys,
            rows,
            mark_width,
            label_widths,
            row_height,
        } => {
            let mut x = left;
            for (column, part) in keys.chunks(*rows).enumerate() {
                for (row, key) in part.iter().enumerate() {
                    let middle = y + (row as f64 + 0.5) * row_height;
                    // The patch is under the piece of line and the point, as the fill is.
                    if let Some((colour, opacity)) = &key.patch {
                        let (top, bottom) =
                            (middle - KEY_PATCH.1 / 2.0, middle + KEY_PATCH.1 / 2.0);
                        items.push(named(
                            &plan.name,
                            Primitive::Polygon {
                                rings: vec![vec![
                                    (x, top),
                                    (x + mark_width, top),
                                    (x + mark_width, bottom),
                                    (x, bottom),
                                ]],
                                fill: Some(colour.clone()),
                                stroke: None,
                                opacity: *opacity,
                            },
                        ));
                    }
                    if let Some((dash, width)) = key.line {
                        items.push(named(
                            &plan.name,
                            Primitive::Segment {
                                from: (x, middle),
                                to: (x + KEY_LINE, middle),
                                stroke: Stroke {
                                    colour: key.colour.clone(),
                                    width,
                                    dash,
                                },
                                heads: (Head::Plain, Head::Plain),
                            },
                        ));
                    }
                    if key.point {
                        items.push(named(
                            &plan.name,
                            Primitive::Point {
                                at: (x + mark_width / 2.0, middle),
                                shape: key.shape,
                                size: key.size,
                                colour: key.colour.clone(),
                                opacity: 1.0,
                            },
                        ));
                    }
                    items.push(named(
                        &plan.name,
                        text(
                            (x + mark_width + theme.label_gap, middle + half_capital),
                            Anchor::Start,
                            0.0,
                            &key.label,
                            theme.label_size,
                            theme,
                        ),
                    ));
                }
                x += mark_width + theme.label_gap + label_widths[column] + COLUMN_GAP;
            }
        }
        Content::Ramp {
            colours,
            length,
            ticks,
            across: true,
        } => {
            // Lying: the smallest value at the left, a tick and a label under each break.
            let widest = (plan.width - length).max(0.0);
            let start = left + widest / 2.0;
            items.push(named(
                &plan.name,
                Primitive::Raster {
                    at: (start, y),
                    width: *length,
                    height: theme.ramp_width,
                    columns: colours.len(),
                    rows: 1,
                    cells: colours.iter().map(|colour| rgb(colour)).collect(),
                },
            ));
            let under = y + theme.ramp_width;
            for (share, label) in ticks {
                let at = start + share * length;
                items.push(named(
                    &plan.name,
                    Primitive::Segment {
                        from: (at, under),
                        to: (at, under + theme.tick_length),
                        stroke: Stroke {
                            colour: theme.ink.to_string(),
                            width: theme.axis_width,
                            dash: LineType::Solid,
                        },
                        heads: (Head::Plain, Head::Plain),
                    },
                ));
                items.push(named(
                    &plan.name,
                    text(
                        (
                            at,
                            under + theme.tick_length + theme.label_gap + capital(theme.label_size),
                        ),
                        Anchor::Middle,
                        0.0,
                        label,
                        theme.label_size,
                        theme,
                    ),
                ));
            }
        }
        Content::Ramp {
            colours,
            length,
            ticks,
            across: false,
        } => {
            let top = y + half_capital;
            items.push(named(
                &plan.name,
                Primitive::Raster {
                    at: (left, top),
                    width: theme.ramp_width,
                    height: *length,
                    columns: 1,
                    rows: colours.len(),
                    cells: colours.iter().map(|colour| rgb(colour)).collect(),
                },
            ));
            for (share, label) in ticks {
                let at = top + share * length;
                let right = left + theme.ramp_width;
                items.push(named(
                    &plan.name,
                    Primitive::Segment {
                        from: (right, at),
                        to: (right + theme.tick_length, at),
                        stroke: Stroke {
                            colour: theme.ink.to_string(),
                            width: theme.axis_width,
                            dash: LineType::Solid,
                        },
                        heads: (Head::Plain, Head::Plain),
                    },
                ));
                items.push(named(
                    &plan.name,
                    text(
                        (
                            right + theme.tick_length + theme.label_gap,
                            at + half_capital,
                        ),
                        Anchor::Start,
                        0.0,
                        label,
                        theme.label_size,
                        theme,
                    ),
                ));
            }
        }
    }
}

/// The drawing of `scene` on a page of `size` under `theme`. The notes of the scene travel
/// beside the drawing and are not drawn (open decision #118, F7).
pub fn layout(scene: &Scene, size: Size, theme: &Theme) -> Result<Drawing, LayoutError> {
    lay(scene, size, theme, false)
}

/// The same, with the notes of the scene written under the figure, each as the console prints
/// it: what `export(notes: true)` asks for.
pub fn layout_with_notes(scene: &Scene, size: Size, theme: &Theme) -> Result<Drawing, LayoutError> {
    lay(scene, size, theme, true)
}

fn lay(scene: &Scene, size: Size, theme: &Theme, noted: bool) -> Result<Drawing, LayoutError> {
    if let Some(why) = scene.undrawn() {
        return Err(LayoutError::Undrawn(why));
    }
    let (wide, high) = size.millimetres();
    if !(wide.is_finite() && high.is_finite() && wide > 0.0 && high > 0.0) {
        return Err(LayoutError::NotASize);
    }
    let (width, height) = size.points();
    let mut set = Setter::default();
    let panel = scene.panels.first();
    let xs = panel.and_then(|p| p.x).and_then(|id| scene.scale(id));
    let ys = panel.and_then(|p| p.y).and_then(|id| scene.scale(id));
    let label = theme.label_size;
    let title_line = theme.title_size * theme.leading;
    let half_capital = capital(label) / 2.0;

    // The notes under the figure, when they are asked for: each broken to the width of the
    // page, in the type of the labels, and the room they take.
    let noted_lines: Vec<String> = if noted {
        scene
            .notes
            .iter()
            .flat_map(|note| {
                set.wrap(
                    &format!("note: {note}"),
                    FaceName::Sans,
                    label,
                    (width - 2.0 * theme.margin).max(1.0),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let noted_room = if noted_lines.is_empty() {
        0.0
    } else {
        noted_lines.len() as f64 * label * theme.leading + theme.title_gap
    };

    // The legends: at the right, one under the other; above or under the panel, side by
    // side; or not drawn.
    let place = scene.legend;
    let banded = matches!(place, LegendPlace::Top | LegendPlace::Bottom);
    // The room of a band: the width of the page less its margins and the room the labels and
    // the title of the y axis take at the left, which is not yet known; a sixth of the page
    // is allowed for it.
    let band_room = (width - 2.0 * theme.margin - width / 6.0).max(1.0);
    // With `direct`, the legends that lines read are replaced by the names at the ends of
    // the lines; any other stands at the right.
    let (replaced, ends) = if place == LegendPlace::Direct {
        end_labels(scene, theme)
    } else {
        (vec![false; scene.legends.len()], Vec::new())
    };
    let ends_width = ends
        .iter()
        .map(|end| set.width(&end.text, FaceName::Sans, label))
        .fold(0.0, f64::max);
    let plans: Vec<LegendPlan> = if place == LegendPlace::None {
        Vec::new()
    } else {
        scene
            .legends
            .iter()
            .zip(&replaced)
            .filter(|(_, replaced)| !**replaced)
            .map(|(legend, _)| {
                plan_legend(
                    scene,
                    legend,
                    theme,
                    &mut set,
                    (width, height - noted_room),
                    banded.then_some(band_room),
                )
            })
            .collect()
    };
    // Above or under the panel, a legend that does not fit beside the one before it starts
    // a new band under it.
    let mut bands: Vec<(f64, Vec<usize>)> = Vec::new();
    if banded {
        let mut used = 0.0;
        for (at, plan) in plans.iter().enumerate() {
            match bands.last_mut() {
                Some((tallest, members)) if used + theme.legend_gap + plan.width <= band_room => {
                    *tallest = tallest.max(plan.height);
                    members.push(at);
                    used += theme.legend_gap + plan.width;
                }
                _ => {
                    bands.push((plan.height, vec![at]));
                    used = plan.width;
                }
            }
        }
    }
    let bands_height = bands.iter().map(|(tallest, _)| tallest).sum::<f64>()
        + theme.legend_gap * bands.len() as f64;
    // A legend that does not fit under the one before it starts a new column to its right.
    let legends_room = height - noted_room - 2.0 * theme.margin;
    let mut legend_columns: Vec<(f64, Vec<usize>)> = Vec::new();
    let mut used = 0.0;
    for (at, plan) in plans.iter().enumerate().filter(|_| !banded) {
        match legend_columns.last_mut() {
            Some((widest, members)) if used + theme.legend_gap + plan.height <= legends_room => {
                *widest = widest.max(plan.width);
                members.push(at);
                used += theme.legend_gap + plan.height;
            }
            _ => {
                legend_columns.push((plan.width, vec![at]));
                used = plan.height;
            }
        }
    }
    let legends_width = legend_columns.iter().map(|(widest, _)| widest).sum::<f64>()
        + theme.legend_gap * legend_columns.len().saturating_sub(1) as f64;

    // The widths of the labels of the two axes.
    let widths = |scale: Option<&Scale>, set: &mut Setter| -> Vec<f64> {
        scale.map_or(Vec::new(), |s| {
            s.labels
                .iter()
                .map(|l| set.width(l, FaceName::Sans, label))
                .collect()
        })
    };
    let x_widths = widths(xs, &mut set);
    let y_widths = widths(ys, &mut set);
    let widest_y = y_widths.iter().copied().fold(0.0, f64::max);

    // The room each side takes. A label at the end of the x axis may reach past the panel by
    // half its width, and the top label of the y axis by half its height.
    let top = theme.margin
        + half_capital
        + 1.0
        + if place == LegendPlace::Top {
            bands_height
        } else {
            0.0
        };
    // At the right of the panel: the names at the ends of the lines, then the legends; or
    // half the last label of the x axis, which may reach past the panel.
    let ends_room = if ends.is_empty() {
        0.0
    } else {
        theme.label_gap + ends_width
    };
    let right = theme.margin
        + if plans.is_empty() || banded {
            x_widths.last().map_or(0.0, |w| w / 2.0).max(ends_room)
        } else {
            ends_room + theme.legend_gap + legends_width
        };
    let sides = |x_lines: usize, y_lines: usize| -> (f64, f64) {
        let left = theme.margin
            + match ys {
                Some(_) => {
                    let title = if y_lines > 0 {
                        y_lines as f64 * title_line + theme.title_gap
                    } else {
                        0.0
                    };
                    title + widest_y + theme.label_gap + theme.tick_length
                }
                None => x_widths.first().map_or(0.0, |w| w / 2.0),
            };
        let bottom = theme.margin
            + noted_room
            + if place == LegendPlace::Bottom {
                bands_height
            } else {
                0.0
            }
            + match xs {
                Some(_) => {
                    let title = if x_lines > 0 {
                        x_lines as f64 * title_line + theme.title_gap
                    } else {
                        0.0
                    };
                    title + capital(label) + theme.label_gap + theme.tick_length
                }
                None => half_capital,
            };
        (left, bottom)
    };
    // A title is broken to the side it runs along, which depends on the room the titles
    // take: settled in two rounds, from one line each.
    let titled = |scale: Option<&Scale>| scale.is_some_and(|s| !s.title.trim().is_empty());
    let mut x_title: Vec<String> = Vec::new();
    let mut y_title: Vec<String> = Vec::new();
    let (mut left, mut bottom) = sides(usize::from(titled(xs)), usize::from(titled(ys)));
    for _ in 0..2 {
        let (panel_width, panel_height) = (width - left - right, height - top - bottom);
        if let Some(s) = xs.filter(|_| titled(xs)) {
            x_title = set.wrap(
                &s.title,
                FaceName::Sans,
                theme.title_size,
                panel_width.max(1.0),
            );
        }
        if let Some(s) = ys.filter(|_| titled(ys)) {
            y_title = set.wrap(
                &s.title,
                FaceName::Sans,
                theme.title_size,
                panel_height.max(1.0),
            );
        }
        (left, bottom) = sides(x_title.len(), y_title.len());
    }
    let (panel_width, panel_height) = (width - left - right, height - top - bottom);
    if !(panel_width >= SMALLEST_PANEL && panel_height >= SMALLEST_PANEL) {
        return Err(LayoutError::TooSmall {
            panel: (panel_width, panel_height),
        });
    }
    // A value is placed by the share of the way it stands between the limits of its axis, as
    // the kind of the axis has it: linear, log or square root.
    let across = |s: &Scale, v: f64| left + scale::share(s.kind, s.limits, v) * panel_width;
    let down =
        |s: &Scale, v: f64| top + panel_height - scale::share(s.kind, s.limits, v) * panel_height;
    let line = |colour: &str, width: f64| Stroke {
        colour: colour.to_string(),
        width,
        dash: LineType::Solid,
    };
    let segment = |from: (f64, f64), to: (f64, f64), stroke: Stroke| Primitive::Segment {
        from,
        to,
        stroke,
        heads: (Head::Plain, Head::Plain),
    };

    let mut items: Vec<Item> = vec![named(
        "background",
        Primitive::Rectangle {
            at: (0.0, 0.0),
            width,
            height,
            radius: 0.0,
            fill: Some(theme.background.to_string()),
            stroke: None,
        },
    )];
    // The gridlines, under the marks.
    if let Some(s) = xs {
        for &value in &s.breaks {
            let x = across(s, value);
            items.push(named(
                "grid",
                segment(
                    (x, top),
                    (x, top + panel_height),
                    line(theme.grid, theme.grid_width),
                ),
            ));
        }
    }
    if let Some(s) = ys {
        for &value in &s.breaks {
            let y = down(s, value);
            items.push(named(
                "grid",
                segment(
                    (left, y),
                    (left + panel_width, y),
                    line(theme.grid, theme.grid_width),
                ),
            ));
        }
    }
    // The marks, clipped to the panel.
    let mut marks: Vec<Item> = Vec::new();
    // A reference line goes from one edge of the panel to the other, at each place of its
    // layer on the one axis it needs, in the colour of the place and the type of the layer.
    let reference = |layer: &crate::scene::Layer, name: &str| -> Vec<Item> {
        let level = layer.mark == Mark::Hline;
        let (places, scale) = if level {
            (&layer.y, ys)
        } else {
            (&layer.x, xs)
        };
        let Some(scale) = scale else {
            return Vec::new();
        };
        let dash = match &layer.linetype {
            Lining::Constant(name) => LineType::parse(name).unwrap_or_default(),
            Lining::Scaled(_) => LineType::Solid,
        };
        places
            .iter()
            .enumerate()
            .map(|(i, place)| {
                let colour = match &layer.color {
                    Paint::Theme => theme.mark.to_string(),
                    Paint::Constant(hex) => hex.clone(),
                    Paint::Scaled(values) => match (scene.scale("color"), values.get(i)) {
                        (Some(scale), Some(value)) => colour_on(scale, *value),
                        _ => theme.mark.to_string(),
                    },
                };
                let (from, to) = if level {
                    let y = down(scale, *place);
                    ((left, y), (left + panel_width, y))
                } else {
                    let x = across(scale, *place);
                    ((x, top), (x, top + panel_height))
                };
                named(
                    name,
                    Primitive::Segment {
                        from,
                        to,
                        stroke: Stroke {
                            colour,
                            width: layer.width,
                            dash,
                        },
                        heads: (Head::Plain, Head::Plain),
                    },
                )
            })
            .collect()
    };
    // A rug has a tick for each mark, from the edge of the panel inwards: at the bottom for a
    // place on `x`, at the left for one on `y`.
    let ticks = |layer: &crate::scene::Layer, name: &str| -> Vec<Item> {
        let long = match &layer.size {
            Sizing::Constant(points) => *points,
            Sizing::Scaled(_) => theme.key_size,
        };
        let tick = |i: usize, from: (f64, f64), to: (f64, f64)| {
            let colour = match &layer.color {
                Paint::Theme => theme.mark.to_string(),
                Paint::Constant(hex) => hex.clone(),
                Paint::Scaled(values) => match (scene.scale("color"), values.get(i)) {
                    (Some(scale), Some(value)) => colour_on(scale, *value),
                    _ => theme.mark.to_string(),
                },
            };
            named(
                name,
                Primitive::Path {
                    steps: vec![Step::Move(from), Step::Line(to)],
                    stroke: Some(Stroke {
                        colour,
                        width: layer.width,
                        dash: LineType::Solid,
                    }),
                    fill: None,
                    opacity: layer.alpha,
                },
            )
        };
        let mut drawn: Vec<Item> = Vec::new();
        if let Some(scale) = xs {
            let base = top + panel_height;
            for (i, value) in layer.x.iter().enumerate() {
                let x = across(scale, *value);
                drawn.push(tick(i, (x, base), (x, base - long)));
            }
        }
        if let Some(scale) = ys {
            for (i, value) in layer.y.iter().enumerate() {
                let y = down(scale, *value);
                drawn.push(tick(i, (left, y), (left + long, y)));
            }
        }
        drawn
    };
    // With one axis alone, the reference lines across it and the rugs along it are all that
    // can be placed.
    if let (Some(p), true) = (panel, xs.is_none() || ys.is_none()) {
        for (at, layer) in p.layers.iter().enumerate() {
            let name = format!("layer:{}", at + 1);
            if layer.mark.spans() {
                marks.extend(reference(layer, &name));
            }
            if matches!(layer.mark, Mark::Rug(_)) {
                marks.extend(ticks(layer, &name));
            }
        }
    }
    if let (Some(p), Some(sx), Some(sy)) = (panel, xs, ys) {
        for (at, layer) in p.layers.iter().enumerate() {
            let name = format!("layer:{}", at + 1);
            let mut drawn: Vec<Item> = Vec::new();
            if layer.mark.spans() {
                drawn.extend(reference(layer, &name));
            }
            if matches!(layer.mark, Mark::Rug(_)) {
                drawn.extend(ticks(layer, &name));
            }
            let colour_of = |i: usize| match &layer.color {
                Paint::Theme => theme.mark.to_string(),
                Paint::Constant(hex) => hex.clone(),
                Paint::Scaled(values) => match (scene.scale("color"), values.get(i)) {
                    (Some(scale), Some(value)) => colour_on(scale, *value),
                    _ => theme.mark.to_string(),
                },
            };
            let marks_of = layer.x.len().min(layer.y.len());
            if matches!(layer.mark, Mark::Bars(_) | Mark::Tile) {
                // A bar: from its lower end to its upper one along `y`, half its width to
                // either side of its place along `x`, in its own fill.
                let bars = marks_of.min(layer.low.len()).min(layer.span.len());
                for i in 0..bars {
                    let (left, right) = (
                        across(sx, layer.x[i] - layer.span[i] / 2.0),
                        across(sx, layer.x[i] + layer.span[i] / 2.0),
                    );
                    let (bottom, upper) = (down(sy, layer.low[i]), down(sy, layer.y[i]));
                    let fill = match &layer.fill {
                        Paint::Theme => theme.mark.to_string(),
                        Paint::Constant(hex) => hex.clone(),
                        Paint::Scaled(values) => match (scene.scale("fill"), values.get(i)) {
                            (Some(scale), Some(value)) => colour_on(scale, *value),
                            _ => theme.mark.to_string(),
                        },
                    };
                    drawn.push(named(
                        &name,
                        Primitive::Polygon {
                            rings: vec![vec![
                                (left, bottom),
                                (right, bottom),
                                (right, upper),
                                (left, upper),
                            ]],
                            fill: Some(fill),
                            stroke: None,
                            opacity: layer.alpha,
                        },
                    ));
                }
            }
            if layer.mark.fills() {
                // The shape under each line of the layer: along its upper edge in the order of
                // the marks and back along its lower edge, in the fill of its first mark.
                let ends = marks_of.min(layer.line.len()).min(layer.low.len());
                let mut start = 0;
                while start < ends {
                    let mut end = start + 1;
                    while end < ends && layer.line[end] == layer.line[start] {
                        end += 1;
                    }
                    let mut ring: Vec<(f64, f64)> = (start..end)
                        .map(|i| (across(sx, layer.x[i]), down(sy, layer.y[i])))
                        .collect();
                    ring.extend(
                        (start..end)
                            .rev()
                            .map(|i| (across(sx, layer.x[i]), down(sy, layer.low[i]))),
                    );
                    let fill = match &layer.fill {
                        Paint::Theme => theme.mark.to_string(),
                        Paint::Constant(hex) => hex.clone(),
                        Paint::Scaled(values) => match (scene.scale("fill"), values.get(start)) {
                            (Some(scale), Some(value)) => colour_on(scale, *value),
                            _ => theme.mark.to_string(),
                        },
                    };
                    drawn.push(named(
                        &name,
                        Primitive::Polygon {
                            rings: vec![ring],
                            fill: Some(fill),
                            stroke: None,
                            opacity: layer.alpha,
                        },
                    ));
                    start = end;
                }
            }
            if layer.mark.joins() {
                // A line through the marks of each line of the layer, in their order, in the
                // colour and the type of its first mark: one line has one of each.
                let mut start = 0;
                while start < marks_of.min(layer.line.len()) {
                    let mut end = start + 1;
                    while end < marks_of.min(layer.line.len())
                        && layer.line[end] == layer.line[start]
                    {
                        end += 1;
                    }
                    let dash = match &layer.linetype {
                        Lining::Constant(name) => LineType::parse(name).unwrap_or_default(),
                        Lining::Scaled(values) => {
                            match (scene.scale("linetype"), values.get(start)) {
                                (Some(scale), Some(value)) => line_on(scale, *value),
                                _ => LineType::Solid,
                            }
                        }
                    };
                    // The places of the marks on the page, joined straight for a line and by
                    // level and upright pieces for steps.
                    let places: Vec<(f64, f64)> = (start..end)
                        .map(|i| (across(sx, layer.x[i]), down(sy, layer.y[i])))
                        .collect();
                    let mut steps: Vec<Step> = Vec::with_capacity(3 * places.len());
                    for (i, &at) in places.iter().enumerate() {
                        if i == 0 {
                            steps.push(Step::Move(at));
                            continue;
                        }
                        let from = places[i - 1];
                        let corners: Vec<(f64, f64)> = match layer.mark {
                            Mark::Step(Stairs::LevelFirst) => vec![(at.0, from.1)],
                            Mark::Step(Stairs::UprightFirst) => vec![(from.0, at.1)],
                            Mark::Step(Stairs::Halfway) => {
                                let half = (from.0 + at.0) / 2.0;
                                vec![(half, from.1), (half, at.1)]
                            }
                            _ => Vec::new(),
                        };
                        // A corner that stands where the line already is adds nothing.
                        let mut last = from;
                        for corner in corners {
                            if corner != last && corner != at {
                                steps.push(Step::Line(corner));
                                last = corner;
                            }
                        }
                        steps.push(Step::Line(at));
                    }
                    drawn.push(named(
                        &name,
                        Primitive::Path {
                            steps,
                            stroke: Some(Stroke {
                                colour: colour_of(start),
                                width: layer.width,
                                dash,
                            }),
                            fill: None,
                            opacity: layer.alpha,
                        },
                    ));
                    start = end;
                }
            }
            if layer.mark == Mark::Text {
                // A text: the label of its mark, centred on its place and moved by the
                // layer's nudge, to the right and up, in its colour and the layer's size.
                let type_size = match &layer.size {
                    Sizing::Constant(points) => *points,
                    Sizing::Scaled(_) => theme.label_size,
                };
                for i in 0..marks_of.min(layer.label.len()) {
                    drawn.push(named(
                        &name,
                        Primitive::Text {
                            at: (
                                across(sx, layer.x[i]) + layer.nudge.0,
                                down(sy, layer.y[i]) - layer.nudge.1 + capital(type_size) / 2.0,
                            ),
                            anchor: Anchor::Middle,
                            turn: 0.0,
                            spans: vec![Span {
                                text: layer.label[i].clone(),
                                face: FaceName::Sans,
                                size: type_size,
                                colour: colour_of(i),
                            }],
                            frame: None,
                        },
                    ));
                }
            }
            let one_by_one = !(layer.mark.joins()
                || layer.mark.patches()
                || matches!(layer.mark, Mark::Rug(_) | Mark::Text));
            for i in 0..if one_by_one { marks_of } else { 0 } {
                let colour = colour_of(i);
                let across_mark = match &layer.size {
                    Sizing::Constant(points) => *points,
                    Sizing::Scaled(values) => match (scene.scale("size"), values.get(i)) {
                        (Some(scale), Some(value)) => match scale.range {
                            Range::Sizes(least, most) => {
                                diameter(*value, scale.limits, (least, most))
                            }
                            _ => theme.key_size,
                        },
                        _ => theme.key_size,
                    },
                };
                let outline = match &layer.shape {
                    Shaping::Constant(name) => *name,
                    Shaping::Scaled(values) => match (scene.scale("shape"), values.get(i)) {
                        (Some(scale), Some(value)) => shape_on(scale, *value),
                        _ => shape::DEFAULT,
                    },
                };
                drawn.push(named(
                    &name,
                    Primitive::Point {
                        at: (across(sx, layer.x[i]), down(sy, layer.y[i])),
                        shape: outline,
                        size: across_mark,
                        colour,
                        opacity: layer.alpha,
                    },
                ));
            }
            // A layer flagged to be drawn in dots (open decision #118, F4): the panel as it
            // stands with this layer on it, the page, the gridlines and the layers under it,
            // as one picture at the dots to the inch of a figure, which takes the place of
            // the layer's marks. A picture has no way to let through what is under it, so it
            // holds it. A panel of more dots than a picture may have keeps its marks.
            if layer.raster {
                // The picture is as many whole dots as cover the panel. Its last dots are
                // drawn whole, and the edge of the panel, which is a line, cuts them where
                // the panel ends.
                let to_a_point = raster::DOTS / 72.0;
                let covering = |points: f64| (points * to_a_point - 1e-6).ceil() / to_a_point;
                let (wide, high) = (covering(panel_width), covering(panel_height));
                let mut under = items.clone();
                let mut inside = marks.clone();
                inside.extend(drawn.iter().cloned());
                under.push(named(
                    "panel",
                    Primitive::Clip {
                        at: (left, top),
                        width: wide,
                        height: high,
                        items: inside,
                    },
                ));
                if let Ok(picture) =
                    raster::render_part(&under, (left, top), wide, high, raster::DOTS)
                {
                    marks.push(named(
                        &name,
                        Primitive::Raster {
                            at: (left, top),
                            width: picture.width as f64 / to_a_point,
                            height: picture.height as f64 / to_a_point,
                            columns: picture.width,
                            rows: picture.height,
                            cells: picture.rgb.as_chunks::<3>().0.to_vec(),
                        },
                    ));
                    continue;
                }
            }
            marks.extend(drawn);
        }
    }
    items.push(named(
        "panel",
        Primitive::Clip {
            at: (left, top),
            width: panel_width,
            height: panel_height,
            items: marks,
        },
    ));
    // The x axis: its line, a tick at each break, the labels that do not touch, its title.
    if let Some(s) = xs {
        let base = top + panel_height;
        items.push(named(
            "axis:x",
            segment(
                (left, base),
                (left + panel_width, base),
                line(theme.ink, theme.axis_width),
            ),
        ));
        let centres: Vec<f64> = s.breaks.iter().map(|v| across(s, *v)).collect();
        let step = thinning(&centres, &x_widths, label / 2.0);
        let baseline = base + theme.tick_length + theme.label_gap + capital(label);
        for (i, &x) in centres.iter().enumerate() {
            items.push(named(
                "axis:x",
                segment(
                    (x, base),
                    (x, base + theme.tick_length),
                    line(theme.ink, theme.axis_width),
                ),
            ));
            if i % step == 0
                && let Some(words) = s.labels.get(i)
            {
                items.push(named(
                    "axis:x",
                    text((x, baseline), Anchor::Middle, 0.0, words, label, theme),
                ));
            }
        }
        for (i, words) in x_title.iter().enumerate() {
            let y = baseline + theme.title_gap + (i + 1) as f64 * title_line
                - theme.title_size * (theme.leading - 1.0);
            items.push(named(
                "axis:x",
                text(
                    (left + panel_width / 2.0, y),
                    Anchor::Middle,
                    0.0,
                    words,
                    theme.title_size,
                    theme,
                ),
            ));
        }
    }
    // The y axis: the same, its labels ending at the ticks and its title read upwards.
    if let Some(s) = ys {
        items.push(named(
            "axis:y",
            segment(
                (left, top),
                (left, top + panel_height),
                line(theme.ink, theme.axis_width),
            ),
        ));
        let centres: Vec<f64> = s.breaks.iter().map(|v| down(s, *v)).collect();
        let heights = vec![capital(label); centres.len()];
        let step = thinning(&centres, &heights, label / 4.0);
        for (i, &y) in centres.iter().enumerate() {
            items.push(named(
                "axis:y",
                segment(
                    (left - theme.tick_length, y),
                    (left, y),
                    line(theme.ink, theme.axis_width),
                ),
            ));
            if i % step == 0
                && let Some(words) = s.labels.get(i)
            {
                items.push(named(
                    "axis:y",
                    text(
                        (left - theme.tick_length - theme.label_gap, y + half_capital),
                        Anchor::End,
                        0.0,
                        words,
                        label,
                        theme,
                    ),
                ));
            }
        }
        // Turned to read upwards, a line's letters rise towards the left: the first line is
        // the leftmost.
        for (i, words) in y_title.iter().enumerate() {
            let x = theme.margin + (i + 1) as f64 * title_line
                - theme.title_size * (theme.leading - 1.0);
            items.push(named(
                "axis:y",
                text(
                    (x, top + panel_height / 2.0),
                    Anchor::Middle,
                    90.0,
                    words,
                    theme.title_size,
                    theme,
                ),
            ));
        }
    }
    // The names at the ends of the lines, beside the right edge of the panel, each at the
    // height its line ends at and moved apart where two would touch.
    if let Some(s) = ys.filter(|_| !ends.is_empty()) {
        let wanted: Vec<f64> = ends.iter().map(|end| down(s, end.y)).collect();
        let heights = spread(&wanted, label * theme.leading, top, top + panel_height);
        for (end, y) in ends.iter().zip(heights) {
            items.push(named(
                &end.name,
                Primitive::Text {
                    at: (left + panel_width + theme.label_gap, y + half_capital),
                    anchor: Anchor::Start,
                    turn: 0.0,
                    spans: vec![Span {
                        text: end.text.clone(),
                        face: FaceName::Sans,
                        size: label,
                        colour: end.colour.clone(),
                    }],
                    frame: None,
                },
            ));
        }
    }
    // The legends above or under the panel: each band from the left edge of the panel.
    if banded {
        let mut y = if place == LegendPlace::Top {
            theme.margin
        } else {
            height - theme.margin - noted_room - bands_height + theme.legend_gap
        };
        for (tallest, members) in &bands {
            let mut x = left;
            for &at in members {
                draw_legend(&plans[at], (x, y), theme, &mut items);
                x += plans[at].width + theme.legend_gap;
            }
            y += tallest + theme.legend_gap;
        }
    }
    // The legends at the right, each column from the top of the panel down.
    let mut x = width - theme.margin - legends_width;
    for (widest, members) in &legend_columns {
        let mut y = top - half_capital;
        for &at in members {
            draw_legend(&plans[at], (x, y), theme, &mut items);
            y += plans[at].height + theme.legend_gap;
        }
        x += widest + theme.legend_gap;
    }
    // The notes, at the foot of the page.
    for (i, line) in noted_lines.iter().enumerate() {
        let baseline = height - theme.margin - noted_room
            + theme.title_gap
            + (i + 1) as f64 * label * theme.leading
            - label * (theme.leading - 1.0);
        items.push(named(
            "notes",
            text(
                (theme.margin, baseline),
                Anchor::Start,
                0.0,
                line,
                label,
                theme,
            ),
        ));
    }
    let mut notes = scene.notes.clone();
    if !set.lacking.is_empty() {
        let shown: Vec<String> = set
            .lacking
            .iter()
            .map(|c| match c {
                '\t' => "a tab".to_string(),
                '\n' | '\r' => "a line break".to_string(),
                c if c.is_control() => format!("U+{:04X}", u32::from(*c)),
                c => format!("`{c}`"),
            })
            .collect();
        let (list, verb) = match shown.split_last() {
            Some((last, [])) => (last.clone(), "is"),
            Some((last, others)) => (format!("{} and {last}", others.join(", ")), "are"),
            None => (String::new(), "are"),
        };
        notes.push(format!(
            "{list} {verb} not in the font of the figure and {verb} drawn as an empty box"
        ));
    }
    Ok(Drawing {
        width,
        height,
        items,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_at_the_ends_of_lines_are_moved_apart_and_stay_beside_the_panel() {
        // Far enough apart: each at the height asked for.
        assert_eq!(
            spread(&[10.0, 40.0, 70.0], 10.0, 0.0, 100.0),
            [10.0, 40.0, 70.0]
        );
        // Two that would touch: the lower is moved down, and the order asked for is kept.
        assert_eq!(spread(&[50.0, 52.0], 10.0, 0.0, 100.0), [50.0, 60.0]);
        assert_eq!(spread(&[52.0, 50.0], 10.0, 0.0, 100.0), [60.0, 50.0]);
        // At the foot of the panel there is no room below: they move up.
        assert_eq!(
            spread(&[95.0, 96.0, 97.0], 10.0, 0.0, 100.0),
            [80.0, 90.0, 100.0]
        );
        // A height asked for beyond the panel is brought to its edge.
        assert_eq!(spread(&[-20.0, 300.0], 10.0, 0.0, 100.0), [0.0, 100.0]);
        // More names than the panel holds keep their distance and run past its top.
        let many = spread(&[50.0; 15], 10.0, 0.0, 100.0);
        let mut sorted = many.clone();
        sorted.sort_by(f64::total_cmp);
        assert!(
            sorted
                .windows(2)
                .all(|pair| pair[1] - pair[0] >= 10.0 - 1e-9)
        );
        assert_eq!(sorted.last(), Some(&100.0));
        assert!(spread(&[], 10.0, 0.0, 100.0).is_empty());
    }

    #[test]
    fn labels_that_would_touch_are_passed_over_evenly() {
        // Room for all: every label is written.
        assert_eq!(thinning(&[0.0, 20.0, 40.0], &[10.0, 10.0, 10.0], 4.0), 1);
        // Too close: every second, then every third.
        assert_eq!(thinning(&[0.0, 10.0, 20.0, 30.0], &[10.0; 4], 4.0), 2);
        assert_eq!(
            thinning(&[0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0], &[10.0; 7], 4.0),
            3
        );
        // A wide label among narrow ones decides for its neighbours alone.
        assert_eq!(thinning(&[0.0, 12.0, 24.0], &[4.0, 30.0, 4.0], 2.0), 2);
        // Places that fall (an axis that runs downwards) are read the same.
        assert_eq!(thinning(&[40.0, 20.0, 0.0], &[10.0; 3], 4.0), 1);
        // Labels that cannot be told apart at all: the first alone.
        assert_eq!(thinning(&[5.0, 5.0, 5.0], &[10.0; 3], 4.0), 3);
        // One label, and none.
        assert_eq!(thinning(&[7.0], &[100.0], 4.0), 1);
        assert_eq!(thinning(&[], &[], 4.0), 1);
    }

    #[test]
    fn a_text_is_broken_between_words_and_never_inside_one() {
        let mut set = Setter::default();
        let width = |set: &mut Setter, words: &str| set.width(words, FaceName::Sans, 9.0);
        let whole = "Years of schooling completed";
        let all = width(&mut set, whole);
        // Room for all of it: one line.
        assert_eq!(set.wrap(whole, FaceName::Sans, 9.0, all), [whole]);
        // A little less: the last word goes to a second line.
        assert_eq!(
            set.wrap(whole, FaceName::Sans, 9.0, all - 0.01),
            ["Years of schooling", "completed"]
        );
        // No line is wider than the limit unless it is one word.
        let limit = width(&mut set, "schooling").max(width(&mut set, "completed")) + 1.0;
        let lines = set.wrap(whole, FaceName::Sans, 9.0, limit);
        assert_eq!(lines, ["Years of", "schooling", "completed"]);
        assert!(lines.iter().all(|line| width(&mut set, line) <= limit));
        // A word wider than the limit stands alone; spaces in a row are one; nothing gives none.
        assert_eq!(
            set.wrap("a extraordinarily b", FaceName::Sans, 9.0, 12.0),
            ["a", "extraordinarily", "b"]
        );
        assert_eq!(
            set.wrap("  two   spaces ", FaceName::Sans, 9.0, 500.0),
            ["two spaces"]
        );
        assert!(set.wrap("", FaceName::Sans, 9.0, 50.0).is_empty());
        assert!(set.wrap("   ", FaceName::Sans, 9.0, 50.0).is_empty());
        assert!(set.lacking.is_empty());
        // A character no face has is measured as the empty box and remembered once.
        let boxed = set.width("中中", FaceName::Sans, 10.0);
        assert_eq!(
            boxed,
            2.0 * f64::from(face(FaceName::Sans).advance(0)) * 10.0 / 1000.0
        );
        assert_eq!(set.lacking, ['中']);
    }

    #[test]
    fn a_value_takes_the_colour_and_the_shape_of_its_place_on_a_scale() {
        assert_eq!(JOURNAL.background, "#FFFFFF");
        // A value on a scale of categories takes the colour of its place; beyond the last, the
        // last; on a ramp, its share of the way.
        let categories = Scale {
            id: "color",
            kind: ScaleKind::Category,
            title: String::new(),
            limits: (1.0, 3.0),
            breaks: vec![1.0, 2.0, 3.0],
            labels: vec!["a".to_string(), "b".to_string(), "NA".to_string()],
            range: Range::Colours(["#E69F00", "#56B4E9", "#999999"].map(String::from).to_vec()),
        };
        assert_eq!(colour_on(&categories, 1.0), "#E69F00");
        assert_eq!(colour_on(&categories, 3.0), "#999999");
        assert_eq!(colour_on(&categories, 9.0), "#999999");
        assert_eq!(colour_on(&categories, 0.0), "#E69F00");
        let ramp = Scale {
            kind: ScaleKind::Linear,
            limits: (0.0, 10.0),
            range: Range::Colours(
                ["#000001", "#000002", "#000003", "#000004"]
                    .map(String::from)
                    .to_vec(),
            ),
            ..categories.clone()
        };
        assert_eq!(colour_on(&ramp, 0.0), "#000001");
        assert_eq!(colour_on(&ramp, 4.9), "#000002");
        assert_eq!(colour_on(&ramp, 10.0), "#000004");
        let shapes = Scale {
            range: Range::Shapes(vec!["circle", "triangle"]),
            ..categories
        };
        assert_eq!(shape_on(&shapes, 2.0), "triangle");
        assert_eq!(shape_on(&shapes, 5.0), "triangle");
        assert_eq!(shape_on(&ramp, 2.0), shape::DEFAULT);
    }
}
