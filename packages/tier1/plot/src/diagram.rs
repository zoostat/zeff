//! The drawing of a diagram (Language Reference, sections 7.3 and 7.6, "Diagrams"; open
//! decisions #121 and #191; release 0.10.0, step 22a): a scene that holds a graph form, laid
//! out at the size of the page as a list of the same primitives every figure is drawn with.
//!
//! The form is placed as it asks ([`crate::placed::place_form`]) in the type of the theme's
//! labels; a diagram too large for the page is placed again in smaller type, so that it
//! fits, and one that leaves room stands in the middle of it at the size it has. Then, from
//! the bottom up: the page; each edge, a line from border to border with a head where its
//! kind has one, a common cause dashed; each node, its outline and over it its label; the
//! title and the subtitle at the top; and at the foot a line that says what the marks
//! mean, and the caption.
//!
//! What a node is shows without colour: a latent variable is an ellipse with a dashed
//! outline; a variable the data are selected on has a second outline inside the first; the
//! exposure and the outcome are filled, with their labels in the colour of the page and in
//! the heavier face, the exposure in the first colour of the palette and the outcome in the
//! ink. The line at the foot names them, since two dark fills in a printed figure are told
//! apart by their names.
//!
//! Every primitive of a node carries the node's element name and every primitive of an
//! edge the edge's (`node:tenure`, `edge:training->tenure`), so that an exported SVG groups
//! them and a viewer finds what is under the pointer.

use crate::colour::OKABE_ITO;
use crate::drawing::{Anchor, Drawing, Item, LineType, Place, Primitive, Span, Step, Stroke};
use crate::font::FaceName;
use crate::graph::{EdgeKind, GraphForm, NodeKind, NodeShape, Role};
use crate::layout::{LayoutError, SMALLEST_PANEL, Setter, Size, Theme, capital, named};
use crate::placed::{Measure, Placed, PlacedEdge, PlacedNode, place_form};
use crate::scene::Scene;

/// The width of the outlines and the edges, as a share of the size of the type.
const LINE: f64 = 0.09;

/// The length of an arrow head and half its width, as shares of the size of the type.
const HEAD: (f64, f64) = (0.8, 0.3);

/// The colour of the exposure: the blue of the palette, which reads as dark in grey.
fn exposure_colour() -> &'static str {
    OKABE_ITO
        .iter()
        .find(|(_, name)| *name == "blue")
        .map_or("#0072B2", |(hex, _)| *hex)
}

/// The line under a diagram that says what its marks mean: only what the diagram has.
pub fn key(form: &GraphForm) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    for (role, word) in [(Role::Exposure, "exposure"), (Role::Outcome, "outcome")] {
        if let Some(node) = form.nodes.iter().find(|node| node.role == Some(role)) {
            parts.push(format!("{word} {}", node.id));
        }
    }
    if form.nodes.iter().any(|node| node.kind == NodeKind::Latent) {
        parts.push("dashed outline: latent".to_string());
    }
    if form.nodes.iter().any(|node| node.selected) {
        parts.push("double outline: selected on".to_string());
    }
    if form
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::CommonCause)
    {
        parts.push("dashed arrow: a common cause not observed".to_string());
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// The outline of a node of `width` by `height` about `centre`.
fn outline(
    shape: NodeShape,
    centre: Place,
    width: f64,
    height: f64,
    radius: f64,
    fill: Option<String>,
    stroke: Stroke,
) -> Primitive {
    match shape {
        NodeShape::Box => Primitive::Rectangle {
            at: (centre.0 - width / 2.0, centre.1 - height / 2.0),
            width,
            height,
            radius,
            fill,
            stroke: Some(stroke),
        },
        NodeShape::Ellipse => {
            // Two half turns, from its left to its right and back.
            let half = |to: Place| Step::Arc {
                radii: (width / 2.0, height / 2.0),
                turn: 0.0,
                larger: false,
                clockwise: true,
                to,
            };
            Primitive::Path {
                steps: vec![
                    Step::Move((centre.0 - width / 2.0, centre.1)),
                    half((centre.0 + width / 2.0, centre.1)),
                    half((centre.0 - width / 2.0, centre.1)),
                    Step::Close,
                ],
                stroke: Some(stroke),
                fill,
                opacity: 1.0,
            }
        }
    }
}

/// The head of an arrow whose point is at `tip`, the line coming to it from `from`.
fn head(tip: Place, from: Place, size: f64, colour: &str) -> Option<Primitive> {
    let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt();
    if length == 0.0 {
        return None;
    }
    let (ux, uy) = (dx / length, dy / length);
    let (long, half) = (HEAD.0 * size, HEAD.1 * size);
    let back = (tip.0 - ux * long, tip.1 - uy * long);
    Some(Primitive::Polygon {
        rings: vec![vec![
            tip,
            (back.0 - uy * half, back.1 + ux * half),
            (back.0 + uy * half, back.1 - ux * half),
        ]],
        fill: Some(colour.to_string()),
        stroke: None,
        opacity: 1.0,
    })
}

/// The place a line comes to its first end from, and to its last end from: the next place
/// along it, a control of a curve where it starts or ends with one.
fn approaches(edge: &PlacedEdge) -> Option<(Place, Place)> {
    let (first, last) = edge.ends();
    let towards_first = match edge.steps.get(1)? {
        Step::Line(p) => *p,
        Step::Curve(c1, c2, p) => {
            if *c1 != first {
                *c1
            } else if *c2 != first {
                *c2
            } else {
                *p
            }
        }
        _ => return None,
    };
    let towards_last = match edge.steps.last()? {
        Step::Line(_) => match edge.steps.len() {
            2 => first,
            _ => return None,
        },
        Step::Curve(c1, c2, _) => {
            if *c2 != last {
                *c2
            } else if *c1 != last {
                *c1
            } else {
                first
            }
        }
        _ => return None,
    };
    Some((towards_first, towards_last))
}

/// A placed diagram moved by `by`.
fn moved(placed: &Placed, by: Place) -> (Vec<PlacedNode>, Vec<PlacedEdge>) {
    let shift = |p: Place| -> Place { (p.0 + by.0, p.1 + by.1) };
    let nodes = placed
        .nodes
        .iter()
        .map(|node| PlacedNode {
            centre: shift(node.centre),
            ..node.clone()
        })
        .collect();
    let edges = placed
        .edges
        .iter()
        .map(|edge| PlacedEdge {
            steps: edge
                .steps
                .iter()
                .map(|step| match *step {
                    Step::Move(p) => Step::Move(shift(p)),
                    Step::Line(p) => Step::Line(shift(p)),
                    Step::Curve(c1, c2, p) => Step::Curve(shift(c1), shift(c2), shift(p)),
                    Step::Arc {
                        radii,
                        turn,
                        larger,
                        clockwise,
                        to,
                    } => Step::Arc {
                        radii,
                        turn,
                        larger,
                        clockwise,
                        to: shift(to),
                    },
                    Step::Close => Step::Close,
                })
                .collect(),
            heads: edge.heads,
            label: edge.label.as_ref().map(|label| crate::placed::PlacedLabel {
                centre: shift(label.centre),
                ..label.clone()
            }),
        })
        .collect();
    (nodes, edges)
}

/// The size of the type a diagram is drawn in on a page of `size` under `theme`: the size
/// of the theme's labels, or smaller where the diagram would not fit. `None` for a scene
/// that holds no diagram, or one the page is too small for.
pub fn type_size(scene: &Scene, size: Size, theme: &Theme) -> Option<f64> {
    let form = scene.graph.as_ref()?;
    let plan = Plan::of(scene, form, size, theme, false).ok()?;
    Some(plan.type_size)
}

/// Where everything of a diagram goes on the page.
struct Plan {
    width: f64,
    height: f64,
    heading: Vec<String>,
    subheading: Vec<String>,
    caption: Vec<String>,
    key: Vec<String>,
    notes: Vec<String>,
    /// The size of the type of the labels of the nodes.
    type_size: f64,
    nodes: Vec<PlacedNode>,
    edges: Vec<PlacedEdge>,
    lacking: Vec<char>,
}

impl Plan {
    fn of(
        scene: &Scene,
        form: &GraphForm,
        size: Size,
        theme: &Theme,
        noted: bool,
    ) -> Result<Plan, LayoutError> {
        let (wide, high) = size.millimetres();
        if !(wide.is_finite() && high.is_finite() && wide > 0.0 && high > 0.0) {
            return Err(LayoutError::NotASize);
        }
        let (width, height) = size.points();
        let mut set = Setter::default();
        let label = theme.label_size;
        let page_room = (width - 2.0 * theme.margin).max(1.0);
        let mut broken = |words: Option<&str>, face: FaceName, size: f64| -> Vec<String> {
            words.map_or(Vec::new(), |words| set.wrap(words, face, size, page_room))
        };
        let heading = broken(
            scene.title.as_deref(),
            FaceName::SansSemiBold,
            theme.heading_size,
        );
        let subheading = broken(scene.subtitle.as_deref(), FaceName::Sans, theme.title_size);
        let caption = broken(scene.caption.as_deref(), FaceName::Sans, label);
        let key_line = key(form);
        let key = broken(key_line.as_deref(), FaceName::Sans, label);
        let mut notes = Vec::new();
        if noted {
            for note in &scene.notes {
                notes.extend(broken(Some(note.as_str()), FaceName::Sans, label));
            }
        }
        let heading_room = if heading.is_empty() && subheading.is_empty() {
            0.0
        } else {
            heading.len() as f64 * theme.heading_size * theme.leading
                + subheading.len() as f64 * theme.title_size * theme.leading
                + theme.title_gap
        };
        let foot_lines = key.len() + caption.len() + notes.len();
        let foot_room = if foot_lines == 0 {
            0.0
        } else {
            foot_lines as f64 * label * theme.leading + theme.title_gap
        };
        // The room the diagram has.
        let room = (
            width - 2.0 * theme.margin,
            height - 2.0 * theme.margin - heading_room - foot_room,
        );
        if room.0 < SMALLEST_PANEL || room.1 < SMALLEST_PANEL {
            return Err(LayoutError::TooSmall { panel: room });
        }
        // The labels of the nodes are measured too, for the characters no face has.
        for node in &form.nodes {
            for line in &node.label {
                set.width(line, FaceName::Sans, label);
            }
        }
        // Placed in the type of the labels; where that is too large for the room, again in
        // type as much smaller as it is too large. The margin of the placing is left out:
        // the page has its own.
        let unlaid = |_| {
            LayoutError::Undrawn(
            "this scene holds a diagram whose arrows go round in a ring, or whose placing sets apart nodes it asks to keep together, which cannot be laid out in layers"
                .to_string(),
        )
        };
        let inner = |placed: &Placed, size: f64| -> (f64, f64) {
            let margin = Measure::at(size).margin;
            (
                (placed.width - 2.0 * margin).max(0.0),
                (placed.height - 2.0 * margin).max(0.0),
            )
        };
        let mut type_size = label;
        let mut placed = place_form(form, type_size).map_err(unlaid)?;
        let (w, h) = inner(&placed, type_size);
        let fits = if w > 0.0 && h > 0.0 {
            (room.0 / w).min(room.1 / h)
        } else {
            1.0
        };
        if fits < 1.0 {
            // Scaled as the type is: the gaps a script gives in millimetres are scaled
            // with it, so that the diagram keeps its shape.
            type_size = label * fits;
            let mut smaller = form.clone();
            if let Some((layers, nodes)) = form.placing.spacing {
                smaller.placing.spacing = Some((layers * fits, nodes * fits));
            }
            placed = place_form(&smaller, type_size).map_err(unlaid)?;
        }
        let (w, h) = inner(&placed, type_size);
        let margin = Measure::at(type_size).margin;
        // In the middle of its room.
        let by = (
            theme.margin + (room.0 - w) / 2.0 - margin,
            theme.margin + heading_room + (room.1 - h) / 2.0 - margin,
        );
        let (nodes, edges) = moved(&placed, by);
        Ok(Plan {
            width,
            height,
            heading,
            subheading,
            caption,
            key,
            notes,
            type_size,
            nodes,
            edges,
            lacking: set.lacking,
        })
    }
}

/// The drawing of a scene that holds a diagram, on a page of `size` under `theme`; with
/// `noted`, the notes of the scene are written at the foot.
pub fn draw(scene: &Scene, size: Size, theme: &Theme, noted: bool) -> Result<Drawing, LayoutError> {
    let Some(form) = scene.graph.as_ref() else {
        return Err(LayoutError::Undrawn(
            "this scene holds no diagram".to_string(),
        ));
    };
    let plan = Plan::of(scene, form, size, theme, noted)?;
    let size_of_type = plan.type_size;
    let line = (LINE * size_of_type).max(0.3);
    let ink = theme.ink.to_string();
    let mut items: Vec<Item> = vec![named(
        "background",
        Primitive::Rectangle {
            at: (0.0, 0.0),
            width: plan.width,
            height: plan.height,
            radius: 0.0,
            fill: Some(theme.background.to_string()),
            stroke: None,
        },
    )];
    // The edges, under the nodes: in the order of their names, as the nodes are, so that
    // one graph written in two orders is one figure to the byte.
    let mut edge_order: Vec<usize> = (0..plan.edges.len()).collect();
    edge_order.sort_by_key(|&e| form.edge_name(e));
    for e in edge_order {
        let edge = &plan.edges[e];
        let name = form.edge_name(e);
        let dash = if form.edges[e].kind == EdgeKind::CommonCause {
            LineType::Dashed
        } else {
            LineType::Solid
        };
        items.push(named(
            &name,
            Primitive::Path {
                steps: edge.steps.clone(),
                stroke: Some(Stroke {
                    colour: ink.clone(),
                    width: line,
                    dash,
                }),
                fill: None,
                opacity: 1.0,
            },
        ));
        if let Some((towards_first, towards_last)) = approaches(edge) {
            let (first, last) = edge.ends();
            if edge.heads.0
                && let Some(drawn) = head(first, towards_first, size_of_type, &ink)
            {
                items.push(named(&name, drawn));
            }
            if edge.heads.1
                && let Some(drawn) = head(last, towards_last, size_of_type, &ink)
            {
                items.push(named(&name, drawn));
            }
        }
        if let Some(label) = &edge.label {
            let lines = &form.edges[e].label;
            let step = size_of_type * theme.leading;
            let first = label.centre.1 - (lines.len() as f64 - 1.0) * step / 2.0
                + capital(size_of_type) / 2.0;
            for (i, text) in lines.iter().enumerate() {
                items.push(named(
                    &name,
                    Primitive::Text {
                        at: (label.centre.0, first + i as f64 * step),
                        anchor: Anchor::Middle,
                        turn: 0.0,
                        spans: vec![Span {
                            text: text.clone(),
                            face: FaceName::Sans,
                            size: size_of_type,
                            colour: ink.clone(),
                        }],
                        frame: Some(crate::drawing::TextBox {
                            padding: 0.2 * size_of_type,
                            radius: 0.0,
                            fill: Some(theme.background.to_string()),
                            border: None,
                        }),
                    },
                ));
            }
        }
    }
    // The nodes: the outline, a second inside it for a selected node, and the label.
    let mut node_order: Vec<usize> = (0..plan.nodes.len()).collect();
    node_order.sort_by(|&a, &b| form.nodes[a].id.cmp(&form.nodes[b].id));
    for v in node_order {
        let placed = &plan.nodes[v];
        let node = &form.nodes[v];
        let name = node.name();
        let (fill, text_colour, face) = match node.role {
            Some(Role::Exposure) => (
                exposure_colour().to_string(),
                theme.background.to_string(),
                FaceName::SansSemiBold,
            ),
            Some(Role::Outcome) => (
                ink.clone(),
                theme.background.to_string(),
                FaceName::SansSemiBold,
            ),
            None => (theme.background.to_string(), ink.clone(), FaceName::Sans),
        };
        let border = match node.role {
            Some(Role::Exposure) => exposure_colour().to_string(),
            _ => ink.clone(),
        };
        let stroke = |colour: &str| Stroke {
            colour: colour.to_string(),
            width: line,
            dash: if node.kind == NodeKind::Latent {
                LineType::Dashed
            } else {
                LineType::Solid
            },
        };
        let radius = 0.3 * size_of_type;
        items.push(named(
            &name,
            outline(
                node.shape,
                placed.centre,
                placed.width,
                placed.height,
                radius,
                Some(fill),
                stroke(&border),
            ),
        ));
        if node.selected {
            let inset = 0.28 * size_of_type;
            items.push(named(
                &name,
                outline(
                    node.shape,
                    placed.centre,
                    (placed.width - 2.0 * inset).max(0.0),
                    (placed.height - 2.0 * inset).max(0.0),
                    (radius - inset).max(0.0),
                    None,
                    stroke(&text_colour),
                ),
            ));
        }
        let step = size_of_type * theme.leading;
        let first = placed.centre.1 - (node.label.len() as f64 - 1.0) * step / 2.0
            + capital(size_of_type) / 2.0;
        for (i, text) in node.label.iter().enumerate() {
            items.push(named(
                &name,
                Primitive::Text {
                    at: (placed.centre.0, first + i as f64 * step),
                    anchor: Anchor::Middle,
                    turn: 0.0,
                    spans: vec![Span {
                        text: text.clone(),
                        face,
                        size: size_of_type,
                        colour: text_colour.clone(),
                    }],
                    frame: None,
                },
            ));
        }
    }
    // The title and the subtitle, from the top of the page.
    let mut baseline = theme.margin;
    for (lines, face, size, name) in [
        (
            &plan.heading,
            FaceName::SansSemiBold,
            theme.heading_size,
            "title",
        ),
        (
            &plan.subheading,
            FaceName::Sans,
            theme.title_size,
            "subtitle",
        ),
    ] {
        for text in lines {
            baseline += size * theme.leading;
            items.push(named(
                name,
                Primitive::Text {
                    at: (theme.margin, baseline - size * (theme.leading - 1.0)),
                    anchor: Anchor::Start,
                    turn: 0.0,
                    spans: vec![Span {
                        text: text.clone(),
                        face,
                        size,
                        colour: ink.clone(),
                    }],
                    frame: None,
                },
            ));
        }
    }
    // At the foot, from the bottom up: the notes, the caption, and the line of the marks.
    let label = theme.label_size;
    let foot: Vec<(&str, &String)> = plan
        .key
        .iter()
        .map(|text| ("key", text))
        .chain(plan.caption.iter().map(|text| ("caption", text)))
        .chain(plan.notes.iter().map(|text| ("notes", text)))
        .collect();
    let lines = foot.len() as f64;
    for (i, (name, text)) in foot.iter().enumerate() {
        let baseline = plan.height
            - theme.margin
            - (lines - 1.0 - i as f64) * label * theme.leading
            - label * (theme.leading - 1.0);
        items.push(named(
            name,
            Primitive::Text {
                at: (theme.margin, baseline),
                anchor: Anchor::Start,
                turn: 0.0,
                spans: vec![Span {
                    text: (*text).clone(),
                    face: FaceName::Sans,
                    size: label,
                    colour: ink.clone(),
                }],
                frame: None,
            },
        ));
    }
    let mut notes = scene.notes.clone();
    if !plan.lacking.is_empty() {
        let shown: Vec<String> = plan.lacking.iter().map(|c| format!("`{c}`")).collect();
        let (list, verb) = match shown.split_last() {
            Some((last, [])) => (last.clone(), "is"),
            Some((last, others)) => (format!("{} and {last}", others.join(", ")), "are"),
            None => (String::new(), "are"),
        };
        notes.push(format!(
            "{list} {verb} not in the fonts of the figure and {verb} drawn as an empty box"
        ));
    }
    Ok(Drawing {
        width: plan.width,
        height: plan.height,
        items,
        notes,
    })
}

/// The refusal of a method of a plot that has no meaning for a diagram: a mark, a scale, an
/// axis, a legend, coordinates or panels. `labels`, `theme` and `export` are not refused. The
/// method is named as the script wrote it.
pub fn marks_refused(
    ctx: &zeff_core::runtime::CallContext<'_>,
    plot: &zeff_core::value::Plot,
) -> Result<(), zeff_core::runtime::Fail> {
    if plot.diagram.is_none() {
        return Ok(());
    }
    let name = ctx
        .text
        .get(ctx.callee_span.start..ctx.callee_span.end)
        .unwrap_or("this method");
    Err(zeff_core::runtime::fail(
        zeff_diagnostics::Diagnostic::error(
            "Z0401",
            format!("`{name}` has no meaning for a diagram"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the plot draws a causal graph")
        .help("a diagram has no marks, scales, axes or panels; it takes `labels`, `theme` and `export`, and its nodes are placed by the options of the `plot` that made it: `layout:`, `at:`, `direction:`, `same_rank:` and `spacing:`"),
    ))
}
