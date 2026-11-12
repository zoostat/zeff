//! What the tests of the writers of figures share (release 0.7.0, steps 21c and 22b1): small
//! frames and the scenes drawn from them, a drawing that holds every primitive of the closed
//! set, the sample scenes of the shipped wages, and the frames a property is tried on.

#![allow(dead_code)]

use proptest::prelude::*;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::value::{Aesthetic, Frame, Mark, PlotConstant, PlotLayer, PlotMapping, PlotSetting};
use zeff_plot::drawing::{
    Anchor, Drawing, Head, Item, Primitive, Span, Step, Stroke, TextBox, rgb,
};
use zeff_plot::font::FaceName;
use zeff_plot::scene::{Scene, start, with_layer};

pub fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

pub fn frame(name: &str, columns: Vec<(&str, Column)>) -> Frame {
    let table = Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap();
    Frame {
        table: Rc::new(table),
        name: Some(name.to_string()),
    }
}

pub fn floats(values: &[Option<f64>]) -> Column {
    Column::floats(values.iter().copied()).0
}

pub fn numbers(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

pub fn mapped(aesthetic: Aesthetic, name: &str) -> PlotMapping {
    PlotMapping {
        aesthetic,
        variable: name.to_string(),
        computed: None,
    }
}

pub fn points(alpha: f64) -> PlotLayer {
    PlotLayer {
        mark: Mark::Points,
        data: None,
        alpha,
        size: None,
        jitter: 0.0,
        stream: None,
        computed: Vec::new(),
    }
}

pub fn drawn_with(
    frame: Frame,
    mappings: &[(Aesthetic, &str)],
    constants: Vec<PlotConstant>,
    alpha: f64,
) -> Scene {
    let plot = start(
        frame,
        mappings.iter().map(|(a, n)| mapped(*a, n)).collect(),
        constants,
    );
    with_layer(&plot, points(alpha)).1
}

pub fn drawn(frame: Frame, mappings: &[(Aesthetic, &str)]) -> Scene {
    drawn_with(frame, mappings, Vec::new(), 1.0)
}

pub fn wages() -> Frame {
    Frame {
        table: Rc::new(zeff_datasets::table("wages").expect("the shipped wages")),
        name: Some("wages".to_string()),
    }
}

pub fn small(name: &str) -> Frame {
    frame(
        name,
        vec![
            ("dose", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
            ("response", numbers(&[2.1, 3.9, 6.2, 7.8, 9.0, 11.5])),
            (
                "arm",
                Column::strs(["placebo", "drug", "placebo", "drug", "placebo", "drug"].map(Some)),
            ),
            ("weight", numbers(&[60.0, 72.5, 81.0, 55.0, 90.0, 68.0])),
        ],
    )
}

/// The marks of a drawing: the points inside its clip regions.
pub fn marks(drawing: &Drawing) -> usize {
    fn count(items: &[Item]) -> usize {
        items
            .iter()
            .map(|item| match &item.primitive {
                Primitive::Clip { items, .. } => items
                    .iter()
                    .filter(|i| matches!(i.primitive, Primitive::Point { .. }))
                    .count(),
                _ => 0,
            })
            .sum()
    }
    count(&drawing.items)
}

/// A drawing that holds every primitive of the closed set, in each of its forms.
pub fn every_primitive() -> Drawing {
    let line = |width: f64| Stroke {
        colour: "#222222".to_string(),
        width,
    };
    let unnamed = |primitive: Primitive| Item {
        name: None,
        primitive,
    };
    let named = |name: &str, primitive: Primitive| Item {
        name: Some(name.to_string()),
        primitive,
    };
    let point = |x: f64, shape: &'static str, colour: &str, opacity: f64| Primitive::Point {
        at: (x, 30.0),
        shape,
        size: 9.0,
        colour: colour.to_string(),
        opacity,
    };
    let span = |words: &str, face: FaceName, size: f64| Span {
        text: words.to_string(),
        face,
        size,
        colour: "#222222".to_string(),
    };
    Drawing {
        width: 200.0,
        height: 120.0,
        items: vec![
            named(
                "background",
                Primitive::Rectangle {
                    at: (0.0, 0.0),
                    width: 200.0,
                    height: 120.0,
                    radius: 0.0,
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                },
            ),
            // The fifteen shapes: two runs of one paint, then one mark of each kind.
            named(
                "panel",
                Primitive::Clip {
                    at: (5.0, 5.0),
                    width: 190.0,
                    height: 50.0,
                    items: vec![
                        named("layer:1", point(12.0, "circle", "#E69F00", 1.0)),
                        named("layer:1", point(24.0, "triangle", "#E69F00", 1.0)),
                        named("layer:1", point(36.0, "square", "#E69F00", 1.0)),
                        named("layer:1", point(48.0, "diamond", "#56B4E9", 0.5)),
                        named("layer:1", point(60.0, "triangle_down", "#56B4E9", 0.5)),
                        named("layer:1", point(72.0, "star", "#56B4E9", 0.5)),
                        named("layer:1", point(84.0, "plus", "#56B4E9", 0.5)),
                        named("layer:1", point(96.0, "cross", "#222222", 1.0)),
                        named("layer:1", point(108.0, "asterisk", "#222222", 1.0)),
                        named("layer:1", point(120.0, "circle_open", "#222222", 1.0)),
                        named("layer:1", point(132.0, "star_open", "#222222", 1.0)),
                        named(
                            "layer:2",
                            point(144.0, "triangle_down_open", "#009E73", 1.0),
                        ),
                        named("layer:2", point(156.0, "square_open", "#009E73", 1.0)),
                        named("layer:2", point(168.0, "diamond_open", "#009E73", 1.0)),
                        named("layer:2", point(180.0, "triangle_open", "#009E73", 1.0)),
                        // A name that is no shape is drawn as a circle.
                        named("layer:2", point(190.0, "hexagon", "#009E73", 1.0)),
                    ],
                },
            ),
            unnamed(Primitive::Path {
                steps: vec![
                    Step::Move((10.0, 70.0)),
                    Step::Line((30.0, 62.0)),
                    Step::Curve((40.0, 60.0), (50.0, 80.0), (60.0, 70.0)),
                    Step::Arc {
                        radii: (8.0, 5.0),
                        turn: 30.0,
                        larger: false,
                        clockwise: true,
                        to: (76.0, 70.0),
                    },
                    Step::Close,
                ],
                stroke: Some(line(0.75)),
                fill: Some("#F0E442".to_string()),
                opacity: 0.6,
            }),
            unnamed(Primitive::Polygon {
                rings: vec![
                    vec![(90.0, 60.0), (120.0, 60.0), (120.0, 80.0), (90.0, 80.0)],
                    vec![(98.0, 65.0), (112.0, 65.0), (112.0, 75.0), (98.0, 75.0)],
                ],
                fill: Some("#0072B2".to_string()),
                stroke: None,
                opacity: 1.0,
            }),
            unnamed(Primitive::Rectangle {
                at: (130.0, 60.0),
                width: 24.0,
                height: 20.0,
                radius: 3.0,
                fill: None,
                stroke: Some(line(0.5)),
            }),
            named(
                "axis:x",
                Primitive::Segment {
                    from: (10.0, 90.0),
                    to: (80.0, 90.0),
                    stroke: line(0.5),
                    heads: (Head::Plain, Head::Arrow),
                },
            ),
            named(
                "axis:x",
                Primitive::Segment {
                    from: (100.0, 96.0),
                    to: (150.0, 86.0),
                    stroke: line(1.0),
                    heads: (Head::Arrow, Head::Arrow),
                },
            ),
            named(
                "axis:x",
                Primitive::Text {
                    at: (45.0, 104.0),
                    anchor: Anchor::Middle,
                    turn: 0.0,
                    spans: vec![span("Wage & hours <2010>", FaceName::Sans, 9.0)],
                    frame: None,
                },
            ),
            // Two spans in two faces, in a box; and a text read upwards, in a box.
            unnamed(Primitive::Text {
                at: (120.0, 112.0),
                anchor: Anchor::Start,
                turn: 0.0,
                spans: vec![
                    span("R² = ", FaceName::SansSemiBold, 10.0),
                    span("0.47", FaceName::Mono, 10.0),
                ],
                frame: Some(TextBox {
                    padding: 2.0,
                    radius: 1.5,
                    fill: Some("#F7F7F7".to_string()),
                    border: Some(line(0.4)),
                }),
            }),
            unnamed(Primitive::Text {
                at: (190.0, 100.0),
                anchor: Anchor::End,
                turn: 90.0,
                spans: vec![span("up", FaceName::Sans, 8.0)],
                frame: Some(TextBox {
                    padding: 1.0,
                    radius: 0.0,
                    fill: None,
                    border: Some(line(0.4)),
                }),
            }),
            // A text of nothing is not written.
            unnamed(Primitive::Text {
                at: (0.0, 0.0),
                anchor: Anchor::Start,
                turn: 0.0,
                spans: vec![span("", FaceName::Sans, 8.0)],
                frame: None,
            }),
            // A grid of cells: neighbours of one colour in a row are one rectangle.
            named(
                "legend:color",
                Primitive::Raster {
                    at: (160.0, 60.0),
                    width: 30.0,
                    height: 20.0,
                    columns: 3,
                    rows: 2,
                    cells: [
                        "#440154", "#440154", "#21918C", "#FDE725", "#21918C", "#FDE725",
                    ]
                    .map(rgb)
                    .to_vec(),
                },
            ),
            // The name of the first group once more: its id is numbered.
            named(
                "background",
                Primitive::Segment {
                    from: (0.0, 119.0),
                    to: (200.0, 119.0),
                    stroke: line(0.25),
                    heads: (Head::Plain, Head::Plain),
                },
            ),
        ],
        notes: vec!["a note is not drawn".to_string()],
    }
}

/// The scenes of the shipped wages the release draws its sample figures from.
pub fn samples() -> Vec<(&'static str, Scene)> {
    let w = wages();
    let star = PlotConstant {
        aesthetic: Aesthetic::Shape,
        value: PlotSetting::Shape("star_open"),
    };
    vec![
        (
            "wage_schooling",
            drawn(w.clone(), &[(Aesthetic::X, "educ"), (Aesthetic::Y, "wage")]),
        ),
        (
            "wage_experience_translucent",
            drawn_with(
                w.clone(),
                &[(Aesthetic::X, "exper"), (Aesthetic::Y, "wage")],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "coloured_by_union",
            drawn_with(
                w.clone(),
                &[
                    (Aesthetic::X, "educ"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Color, "union"),
                ],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "coloured_by_hours",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "educ"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Color, "hours"),
                ],
            ),
        ),
        (
            "sized_coloured_shaped",
            drawn_with(
                w.clone(),
                &[
                    (Aesthetic::X, "exper"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Size, "hours"),
                    (Aesthetic::Color, "female"),
                    (Aesthetic::Shape, "female"),
                ],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "shaped_by_industry",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "age"),
                    (Aesthetic::Y, "hours"),
                    (Aesthetic::Shape, "industry"),
                ],
            ),
        ),
        (
            "one_shape",
            drawn_with(
                w,
                &[(Aesthetic::X, "educ"), (Aesthetic::Y, "hours")],
                vec![star],
                1.0,
            ),
        ),
    ]
}

// ----- the frames a property is tried on -----

pub fn cell() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        1 => Just(None),
        6 => (-50.0..50.0f64).prop_map(Some),
        1 => (-1e9..1e9f64).prop_map(Some),
        1 => (-1e-6..1e-6f64).prop_map(Some),
    ]
}

pub type Row = (Option<f64>, Option<f64>, Option<u8>, Option<f64>);

pub fn rows() -> impl Strategy<Value = Vec<Row>> {
    prop::collection::vec(
        (
            cell(),
            cell(),
            prop::option::weighted(0.85, 0u8..12),
            cell(),
        ),
        0..30,
    )
}

pub fn showing() -> impl Strategy<Value = Vec<(Aesthetic, &'static str)>> {
    prop::sample::select(vec![
        vec![],
        vec![(Aesthetic::Color, "g")],
        vec![(Aesthetic::Color, "z")],
        vec![(Aesthetic::Shape, "g")],
        vec![(Aesthetic::Size, "z")],
        vec![
            (Aesthetic::Color, "g"),
            (Aesthetic::Shape, "g"),
            (Aesthetic::Size, "z"),
        ],
    ])
    .prop_map(|more| {
        let mut all = vec![(Aesthetic::X, "x"), (Aesthetic::Y, "y")];
        all.extend(more);
        all
    })
}

/// The frame of the rows a property is tried on: `x`, `y`, a category `g` and a number `z`.
pub fn tried(rows: &[Row]) -> Frame {
    frame(
        "d",
        vec![
            ("x", floats(&rows.iter().map(|r| r.0).collect::<Vec<_>>())),
            ("y", floats(&rows.iter().map(|r| r.1).collect::<Vec<_>>())),
            (
                "g",
                Column::strs(
                    rows.iter()
                        .map(|r| r.2.map(|k| format!("group <{k:02}> & co"))),
                ),
            ),
            ("z", floats(&rows.iter().map(|r| r.3).collect::<Vec<_>>())),
        ],
    )
}

// ----- a layer drawn in dots inside a figure of lines (step 22b3b) -----

/// A small scene of two layers of points over the same six rows: large and half opaque
/// under, small and opaque over. With `under_in_dots` the first is flagged to be drawn in
/// dots, with `over_in_dots` the second; no script of this release can set the flag.
pub fn two_layers(under_in_dots: bool, over_in_dots: bool) -> Scene {
    let plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    let mut large = points(0.5);
    large.size = Some(12.0);
    let (plot, _) = with_layer(&plot, large);
    let mut little = points(1.0);
    little.size = Some(3.0);
    let (_, mut scene) = with_layer(&plot, little);
    scene.panels[0].layers[0].raster = under_in_dots;
    scene.panels[0].layers[1].raster = over_in_dots;
    scene
}

/// The panel of a drawing: where it starts, its width and height, and what is drawn in it.
pub fn panel_of(drawing: &Drawing) -> (zeff_plot::drawing::Place, f64, f64, &[Item]) {
    drawing
        .items
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } if item.name.as_deref() == Some("panel") => Some((*at, *width, *height, &items[..])),
            _ => None,
        })
        .expect("a drawing with a panel")
}

/// The pictures in the panel of a drawing: for each how many dots across and down, and the
/// red, green and blue of every dot in a row.
pub fn pictures_in(drawing: &Drawing) -> Vec<(usize, usize, Vec<u8>)> {
    panel_of(drawing)
        .3
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Raster {
                columns,
                rows,
                cells,
                ..
            } => Some((*columns, *rows, cells.iter().flatten().copied().collect())),
            _ => None,
        })
        .collect()
}
