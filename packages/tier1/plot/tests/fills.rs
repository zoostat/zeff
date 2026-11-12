//! `area` and `ribbon` in the scene and on the page (Language Reference, section 7.6, "Marks";
//! release 0.7.0, step 23d): the rows of a filled layer are parted into groups, put in the
//! order of `x` and joined as a line is, and the space between that line and zero, or between
//! two variables, is filled; a missing value breaks the shape; the layout draws each shape as
//! one polygon in one fill, and the key of a legend of fills is a patch.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::{Aesthetic, Mark, PlotConstant, PlotLayer, PlotSetting, Stacking};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Paint, Scene, start, with_layer};

/// A layer of areas that says nothing of its own.
fn area() -> PlotLayer {
    PlotLayer {
        mark: Mark::Area(Stacking::Overlay),
        ..points(0.6)
    }
}

/// A band from `y` up to `z`, the two numbers of the frame a property is tried on.
fn ribbon() -> PlotLayer {
    PlotLayer {
        mark: Mark::Ribbon,
        mappings: vec![mapped(Aesthetic::Low, "y"), mapped(Aesthetic::High, "z")],
        ..points(0.3)
    }
}

/// The scene of one layer over a frame with `x`, `y`, a category `g` and a number `z`, the
/// plot mapping `x` and `y`.
fn filled(rows: &[Row], layer: PlotLayer) -> Scene {
    let plot = start(
        tried(rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    with_layer(&plot, layer).1
}

fn row(x: f64, y: f64) -> Row {
    (Some(x), Some(y), None, None)
}

fn band(x: f64, low: f64, high: f64) -> Row {
    (Some(x), Some(low), None, Some(high))
}

/// A polygon of the panel: its corners, its fill and its opacity.
type Shape = (Vec<(f64, f64)>, String, f64);

/// The polygons of the panel, each as its corners in the units of the data, its fill and its
/// opacity: the page is a straight scale of the data between the limits of the two axes.
fn shapes_of(scene: &Scene) -> Vec<Shape> {
    let drawing = layout(scene, Size::OneColumn, &JOURNAL).unwrap();
    let (at, width, height, marks) = panel_of(&drawing);
    let (x, y) = (
        scene.scale("x").unwrap().limits,
        scene.scale("y").unwrap().limits,
    );
    marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Polygon {
                rings,
                fill: Some(fill),
                stroke: None,
                opacity,
            } => {
                assert_eq!(rings.len(), 1, "a filled mark has no hole");
                let corners = rings[0]
                    .iter()
                    .map(|place| {
                        (
                            x.0 + (place.0 - at.0) / width * (x.1 - x.0),
                            y.1 - (place.1 - at.1) / height * (y.1 - y.0),
                        )
                    })
                    .collect();
                Some((corners, fill.clone(), *opacity))
            }
            _ => None,
        })
        .collect()
}

fn close(ours: &[(f64, f64)], theirs: &[(f64, f64)]) {
    assert_eq!(ours.len(), theirs.len(), "{ours:?}");
    for (a, b) in ours.iter().zip(theirs) {
        assert!(
            (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9,
            "{a:?} is not {b:?} in {ours:?}"
        );
    }
}

#[test]
fn an_area_reaches_from_its_line_to_zero_as_ggplot2_has_it() {
    // Five rows, out of order in the frame. The reference is ggplot2 4.0.3,
    // `geom_area(stat = "identity", position = "identity")` on the same rows: the corners of
    // its polygon in the units of the data, and the limits of its two axes.
    let rows = [
        row(3.0, 3.0),
        row(1.0, 2.0),
        row(5.0, 5.0),
        row(2.0, 4.0),
        row(4.0, 1.0),
    ];
    let scene = filled(&rows, area());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Area(Stacking::Overlay));
    assert_eq!(layer.x, [1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(layer.y, [2.0, 4.0, 3.0, 1.0, 5.0]);
    assert_eq!(layer.low, [0.0; 5]);
    assert_eq!(layer.rows, [2, 4, 1, 5, 3]);
    assert_eq!(layer.line, [1; 5]);
    assert_eq!(layer.fill, Paint::Theme);
    assert_eq!(layer.alpha, 0.6);
    assert!(scene.notes.is_empty());
    assert!(scene.legends.is_empty());
    // The axis holds zero, where the area starts, and its margin beyond.
    assert_eq!(scene.scale("x").unwrap().limits, (0.8, 5.2));
    assert_eq!(scene.scale("y").unwrap().limits, (-0.25, 5.25));
    let shapes = shapes_of(&scene);
    assert_eq!(shapes.len(), 1);
    close(
        &shapes[0].0,
        &[
            (1.0, 2.0),
            (2.0, 4.0),
            (3.0, 3.0),
            (4.0, 1.0),
            (5.0, 5.0),
            (5.0, 0.0),
            (4.0, 0.0),
            (3.0, 0.0),
            (2.0, 0.0),
            (1.0, 0.0),
        ],
    );
    // In the colour of the theme's marks, let through as the layer says.
    assert_eq!((shapes[0].1.as_str(), shapes[0].2), (JOURNAL.mark, 0.6));

    // Below zero the area hangs from zero, which is then the top of the axis.
    let under = [row(1.0, -2.0), row(2.0, -4.0)];
    let scene = filled(&under, area());
    assert_eq!(scene.scale("y").unwrap().limits, (-4.2, 0.2));
    close(
        &shapes_of(&scene)[0].0,
        &[(1.0, -2.0), (2.0, -4.0), (2.0, 0.0), (1.0, 0.0)],
    );
}

#[test]
fn a_band_reaches_from_low_to_high_as_ggplot2_has_it() {
    // The reference is `geom_ribbon` of ggplot2 4.0.3 on the same five rows: its polygon
    // goes along the upper edge and back along the lower one, and its limits are 0.8 to 5.2
    // and 0.75 to 6.25.
    let rows = [
        band(1.0, 1.0, 3.0),
        band(2.0, 3.0, 5.0),
        band(3.0, 2.0, 4.5),
        band(4.0, 2.5, 5.0),
        band(5.0, 4.0, 6.0),
    ];
    // The plot maps `x` alone: a band is placed by its two edges.
    let plot = start(tried(&rows), vec![mapped(Aesthetic::X, "x")], Vec::new());
    let (_, scene) = with_layer(&plot, ribbon());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Ribbon);
    assert_eq!(layer.y, [3.0, 5.0, 4.5, 5.0, 6.0]);
    assert_eq!(layer.low, [1.0, 3.0, 2.0, 2.5, 4.0]);
    assert_eq!(layer.line, [1; 5]);
    assert_eq!(layer.alpha, 0.3);
    assert_eq!(scene.scale("x").unwrap().limits, (0.8, 5.2));
    let y = scene.scale("y").unwrap();
    assert_eq!(y.limits, (0.75, 6.25));
    // With nothing mapped to `y`, the axis is named by the lower edge.
    assert_eq!(y.title, "y");
    let shapes = shapes_of(&scene);
    assert_eq!(shapes.len(), 1);
    close(
        &shapes[0].0,
        &[
            (1.0, 3.0),
            (2.0, 5.0),
            (3.0, 4.5),
            (4.0, 5.0),
            (5.0, 6.0),
            (5.0, 4.0),
            (4.0, 2.5),
            (3.0, 2.0),
            (2.0, 3.0),
            (1.0, 1.0),
        ],
    );
    assert_eq!(shapes[0].2, 0.3);

    // On a plot that maps `y`, the band does not read it: `y` is its own lower edge here,
    // and the axis keeps the title of the plot's mapping and holds both layers.
    let scene = filled(&rows, ribbon());
    assert_eq!(scene.panels[0].layers[0].y, [3.0, 5.0, 4.5, 5.0, 6.0]);
    assert_eq!(scene.scale("y").unwrap().limits, (0.75, 6.25));

    // The edges are drawn as they are given: a lower edge above the upper one says nothing.
    let crossed = [band(1.0, 2.0, 1.0), band(2.0, 1.0, 2.0)];
    let scene = filled(&crossed, ribbon());
    assert!(scene.notes.is_empty());
    close(
        &shapes_of(&scene)[0].0,
        &[(1.0, 1.0), (2.0, 2.0), (2.0, 1.0), (1.0, 2.0)],
    );
}

#[test]
fn a_missing_value_breaks_the_shape_and_a_row_left_alone_fills_nothing() {
    let rows: Vec<Row> = vec![
        row(1.0, 1.0),
        row(2.0, 2.0),
        (Some(3.0), None, None, None),
        row(4.0, 4.0),
        row(5.0, 3.0),
        (Some(6.0), None, None, None),
        row(7.0, 5.0),
        (None, Some(9.0), None, None),
    ];
    let scene = filled(&rows, area());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 4.0, 5.0]);
    assert_eq!(layer.line, [1, 1, 2, 2]);
    assert_eq!(layer.rows, [1, 2, 4, 5]);
    assert_eq!(
        scene.notes,
        [
            "3 rows with NA in x or y dropped",
            "2 breaks in the areas where y is NA",
            "1 row with no neighbour to be joined to draws no area",
        ]
    );
    // Two polygons, each with an upright edge where it ends.
    let shapes = shapes_of(&scene);
    assert_eq!(shapes.len(), 2);
    close(
        &shapes[0].0,
        &[(1.0, 1.0), (2.0, 2.0), (2.0, 0.0), (1.0, 0.0)],
    );
    close(
        &shapes[1].0,
        &[(4.0, 4.0), (5.0, 3.0), (5.0, 0.0), (4.0, 0.0)],
    );

    // A band needs both its edges: a row without either is left out and breaks it.
    let rows: Vec<Row> = vec![
        band(1.0, 1.0, 3.0),
        band(2.0, 3.0, 5.0),
        band(3.0, 2.0, 4.5),
        (Some(4.0), Some(2.5), None, None),
        band(5.0, 4.0, 6.0),
        (Some(6.0), None, None, Some(7.0)),
        (Some(7.0), None, None, None),
        (Some(8.0), Some(f64::INFINITY), None, Some(1.0)),
    ];
    let scene = filled(&rows, ribbon());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 3.0]);
    assert_eq!(layer.line, [1, 1, 1]);
    assert_eq!(
        scene.notes,
        [
            "3 rows with NA in x, y or z dropped",
            "1 row with an infinite value in x, y or z dropped",
            "1 break in a band where y or z is NA or infinite",
            "1 row with no neighbour to be joined to draws no band",
        ]
    );
}

#[test]
fn the_rows_are_parted_by_the_group_the_fill_and_the_colour_of_the_plot() {
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), Some(y + 1.0));
    let rows: Vec<Row> = vec![
        cell(1.0, 1.0, 0),
        cell(1.0, 5.0, 1),
        cell(2.0, 2.0, 0),
        cell(2.0, 6.0, 1),
        cell(3.0, 3.0, 0),
    ];
    // By the group: the areas in the order their first row comes, in one fill, no legend.
    let mut grouped = area();
    grouped.mappings = vec![mapped(Aesthetic::Group, "g")];
    let scene = filled(&rows, grouped);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 3.0, 1.0, 2.0]);
    assert_eq!(layer.y, [1.0, 2.0, 3.0, 5.0, 6.0]);
    assert_eq!(layer.line, [1, 1, 1, 2, 2]);
    assert_eq!(layer.fill, Paint::Theme);
    assert!(scene.legends.is_empty());
    // By a fill: the same areas, each mark with the place of its category, and a legend.
    for layer in [area(), ribbon()] {
        let mut by_fill = layer;
        by_fill.mappings.push(mapped(Aesthetic::Fill, "g"));
        let mark = by_fill.mark;
        let scene = filled(&rows, by_fill);
        let layer = &scene.panels[0].layers[0];
        assert_eq!(layer.line, [1, 1, 1, 2, 2]);
        assert_eq!(layer.fill, Paint::Scaled(vec![1.0, 1.0, 1.0, 2.0, 2.0]));
        // A filled mark has no colour of its own.
        assert_eq!(layer.color, Paint::Theme);
        assert_eq!(scene.legends.len(), 1);
        assert_eq!(scene.legends[0].scales, ["fill"]);
        assert_eq!(scene.legends[0].marks, [mark]);
        let shapes = shapes_of(&scene);
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].1, "#E69F00");
        assert_eq!(shapes[1].1, "#56B4E9");
    }
    // By the colour the plot gives its lines: an area under each, and, since nothing says
    // anything of `fill`, filled in the colour of its line (section 6 of the Language
    // Reference: `color:` colours "points, lines or bars by group"), with one legend.
    let plot = start(
        tried(&rows),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
        ],
        Vec::new(),
    );
    let (plot, scene) = with_layer(&plot, area());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.line, [1, 1, 1, 2, 2]);
    assert_eq!(layer.fill, Paint::Scaled(vec![1.0, 1.0, 1.0, 2.0, 2.0]));
    assert_eq!(layer.color, Paint::Theme);
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].scales, ["fill"]);
    assert_eq!(scene.legends[0].title, "g");
    assert_eq!(shapes_of(&scene)[1].1, "#56B4E9");
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Lines,
            ..points(1.0)
        },
    );
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].scales, ["color", "fill"]);
    assert_eq!(
        scene.legends[0].marks,
        [Mark::Lines, Mark::Area(Stacking::Overlay)]
    );
    // What the plot or the layer says of `fill` comes first: a fill of its own, and the
    // areas are parted by the colour still.
    let grey = PlotConstant {
        aesthetic: Aesthetic::Fill,
        value: PlotSetting::Colour {
            hex: "#BEBEBE".to_string(),
            written: "grey".to_string(),
        },
    };
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            constants: vec![grey],
            ..area()
        },
    );
    let own = &scene.panels[0].layers[1];
    assert_eq!(own.fill, Paint::Constant("#BEBEBE".to_string()));
    assert_eq!(own.line, [1, 1, 1, 2, 2]);
    // The one colour a plot sets is the fill of its filled marks too.
    let red = PlotConstant {
        aesthetic: Aesthetic::Color,
        value: PlotSetting::Colour {
            hex: "#B22222".to_string(),
            written: "firebrick".to_string(),
        },
    };
    let plain = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        vec![red],
    );
    let (_, scene) = with_layer(&plain, area());
    assert_eq!(
        scene.panels[0].layers[0].fill,
        Paint::Constant("#B22222".to_string())
    );
    // A colour of numbers parts nothing and fills nothing: one area through every row, in
    // the fill of the theme.
    let plot = start(
        tried(&rows),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "z"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, area());
    assert_eq!(scene.panels[0].layers[0].line, [1; 5]);
    assert_eq!(scene.panels[0].layers[0].fill, Paint::Theme);
}

#[test]
fn one_fill_for_every_shape_from_the_layer_or_from_the_plot() {
    let rows = [row(1.0, 1.0), row(2.0, 2.0)];
    let colour = |hex: &str, written: &str| PlotConstant {
        aesthetic: Aesthetic::Fill,
        value: PlotSetting::Colour {
            hex: hex.to_string(),
            written: written.to_string(),
        },
    };
    // The plot's fill is read by the area, and not by the points over it.
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        vec![colour("#4682B4", "steelblue")],
    );
    let (plot, _) = with_layer(&plot, area());
    let (plot, scene) = with_layer(&plot, points(1.0));
    assert_eq!(
        scene.panels[0].layers[0].fill,
        Paint::Constant("#4682B4".to_string())
    );
    assert_eq!(scene.panels[0].layers[1].fill, Paint::Theme);
    // The layer's own comes before the plot's.
    let mut own = area();
    own.constants = vec![colour("#B22222", "firebrick")];
    let (_, scene) = with_layer(&plot, own);
    assert_eq!(
        scene.panels[0].layers[2].fill,
        Paint::Constant("#B22222".to_string())
    );
    let shapes = shapes_of(&scene);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].1, "#4682B4");
    assert_eq!(shapes[1].1, "#B22222");
}

#[test]
fn a_filled_shape_is_one_polygon_on_the_page_and_its_key_a_patch() {
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![
        cell(1.0, 1.0, 0),
        cell(1.0, 5.0, 1),
        cell(2.0, 2.0, 0),
        cell(2.0, 6.0, 1),
        cell(3.0, 3.0, 0),
    ];
    let mut layer = area();
    layer.mappings = vec![mapped(Aesthetic::Fill, "g")];
    let scene = filled(&rows, layer);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 2);
    assert!(
        marks
            .iter()
            .all(|item| matches!(item.primitive, Primitive::Polygon { .. }))
    );
    // The legend: for each category a patch of its fill, 12 points wide and 8 high, as
    // see-through as the areas, and neither a point nor a piece of line.
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:fill"))
        .collect();
    let patches: Vec<_> = keys
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Polygon {
                rings,
                fill: Some(fill),
                stroke: None,
                opacity,
            } => {
                let ring = &rings[0];
                Some((
                    ring[1].0 - ring[0].0,
                    ring[2].1 - ring[1].1,
                    fill.clone(),
                    *opacity,
                ))
            }
            _ => None,
        })
        .collect();
    assert_eq!(patches.len(), 2);
    assert!((patches[0].0 - 12.0).abs() < 1e-9 && (patches[0].1 - 8.0).abs() < 1e-9);
    assert_eq!((patches[0].2.as_str(), patches[0].3), ("#E69F00", 0.6));
    assert_eq!((patches[1].2.as_str(), patches[1].3), ("#56B4E9", 0.6));
    assert!(!keys.iter().any(|item| matches!(
        item.primitive,
        Primitive::Point { .. } | Primitive::Segment { .. }
    )));
    insta::assert_snapshot!("two_areas", drawing.describe());

    // Areas and lines of one variable, by fill and by colour: one legend, and each key a
    // patch as wide as the piece of line that lies on it.
    let plot = start(
        tried(&rows),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
            mapped(Aesthetic::Fill, "g"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, area());
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Lines,
            ..points(1.0)
        },
    );
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].scales, ["color", "fill"]);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:color+fill"))
        .collect();
    let wide: Vec<f64> = keys
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Polygon { rings, .. } => Some(rings[0][1].0 - rings[0][0].0),
            _ => None,
        })
        .collect();
    assert_eq!(wide, [24.0, 24.0]);
    let lines_in = keys
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Segment { .. }))
        .count();
    assert_eq!(lines_in, 2);
    // The marks of the panel: the two areas, and over them the two lines.
    let (_, _, _, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 4);
    assert!(matches!(marks[0].primitive, Primitive::Polygon { .. }));
    assert!(matches!(marks[3].primitive, Primitive::Path { .. }));
}

#[test]
fn a_band_of_its_own_frame_lies_on_the_axis_of_the_plot() {
    // Points of one frame, and over them a band from a frame that holds `x` and the two
    // edges under other names: nothing of the plot's `y` is asked of it.
    let rows = [row(1.0, 4.0), row(2.0, 5.0), row(3.0, 6.0)];
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let own = frame(
        "band",
        vec![
            ("x", numbers(&[1.0, 3.0])),
            ("a", numbers(&[2.0, 5.0])),
            ("b", numbers(&[7.0, 9.0])),
        ],
    );
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Ribbon,
            data: Some(own),
            mappings: vec![mapped(Aesthetic::Low, "a"), mapped(Aesthetic::High, "b")],
            ..points(0.3)
        },
    );
    let layer = &scene.panels[0].layers[1];
    assert_eq!(layer.data.as_deref(), Some("band"));
    assert_eq!(
        (layer.low.as_slice(), layer.y.as_slice()),
        (&[2.0, 5.0][..], &[7.0, 9.0][..])
    );
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "y");
    // The axis holds the points and both edges: 2 to 9, and 5% beyond.
    assert_eq!(y.limits, (1.65, 9.35));
}

#[test]
fn the_scene_of_a_filled_layer_says_its_lower_edge_and_its_fill() {
    let rows = [row(3.0, 30.0), row(1.0, 10.0), row(2.0, 20.0)];
    let json = filled(&rows, area()).json().write();
    assert!(json.contains("\"mark\": \"area\""), "{json}");
    assert!(json.contains("\"line\": [1, 1, 1]"), "{json}");
    assert!(json.contains("\"low\": [0, 0, 0]"), "{json}");
    assert!(json.contains("\"fill\": null"), "{json}");
    // A filled layer has no type of line and no width.
    assert!(!json.contains("linetype") && !json.contains("\"width\""));
    let bands = [band(1.0, 1.0, 3.0), band(2.0, 2.0, 5.0)];
    let mut blue = ribbon();
    blue.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Fill,
        value: PlotSetting::Colour {
            hex: "#4682B4".to_string(),
            written: "steelblue".to_string(),
        },
    }];
    let json = filled(&bands, blue).json().write();
    assert!(json.contains("\"mark\": \"ribbon\""), "{json}");
    assert!(json.contains("\"y\": [3, 5]"), "{json}");
    assert!(json.contains("\"low\": [1, 2]"), "{json}");
    assert!(json.contains("\"fill\": \"#4682B4\""), "{json}");
    // A layer of points, or of lines, says neither.
    let json = filled(&rows, points(1.0)).json().write();
    assert!(!json.contains("\"low\"") && !json.contains("\"fill\""));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_shape_has_two_marks_or_more_in_the_order_of_x_between_its_two_edges(
        rows in rows(),
        by in 0u8..4,
        banded in any::<bool>(),
    ) {
        let mut layer = if banded { ribbon() } else { area() };
        match by {
            1 => layer.mappings.push(mapped(Aesthetic::Group, "g")),
            2 => layer.mappings.push(mapped(Aesthetic::Fill, "g")),
            _ => {}
        }
        // By the colour of the plot, for the third way of parting.
        let mut mappings = vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")];
        if by == 3 {
            mappings.push(mapped(Aesthetic::Color, "g"));
        }
        let plot = start(tried(&rows), mappings, Vec::new());
        let (_, scene) = with_layer(&plot, layer);
        let drawn = &scene.panels[0].layers[0];
        prop_assert_eq!(drawn.x.len(), drawn.line.len());
        prop_assert_eq!(drawn.x.len(), drawn.rows.len());
        prop_assert_eq!(drawn.x.len(), drawn.low.len());
        prop_assert_eq!(drawn.x.len(), drawn.y.len());
        // The shapes are numbered from 1 without a gap, their marks side by side.
        let mut seen = 0;
        let mut start_at = 0;
        while start_at < drawn.line.len() {
            let mut end = start_at + 1;
            while end < drawn.line.len() && drawn.line[end] == drawn.line[start_at] {
                end += 1;
            }
            seen += 1;
            prop_assert_eq!(drawn.line[start_at], seen);
            prop_assert!(end - start_at >= 2, "a shape of one mark");
            for i in start_at + 1..end {
                prop_assert!(drawn.x[i - 1] <= drawn.x[i], "not in the order of x");
                if drawn.x[i - 1] == drawn.x[i] {
                    prop_assert!(drawn.rows[i - 1] < drawn.rows[i]);
                }
                // One group to a shape.
                if by != 0 {
                    prop_assert_eq!(rows[drawn.rows[i - 1] - 1].2, rows[drawn.rows[i] - 1].2);
                }
            }
            start_at = end;
        }
        // Each mark is its row: the upper edge and the lower one are the values of the row,
        // or the value and zero.
        let mut used = vec![false; rows.len()];
        for (i, &at) in drawn.rows.iter().enumerate() {
            prop_assert!(!used[at - 1], "a row drawn twice");
            used[at - 1] = true;
            let (x, y, _, z) = rows[at - 1];
            prop_assert_eq!(Some(drawn.x[i]), x);
            if banded {
                prop_assert_eq!((Some(drawn.low[i]), Some(drawn.y[i])), (y, z));
            } else {
                prop_assert_eq!((drawn.low[i], Some(drawn.y[i])), (0.0, y));
            }
        }
        // The axis holds both edges of every mark.
        if !drawn.x.is_empty() {
            let (lo, hi) = scene.scale("y").unwrap().limits;
            for value in drawn.y.iter().chain(&drawn.low) {
                prop_assert!(lo <= *value && *value <= hi, "{} outside {} to {}", value, lo, hi);
            }
        }
        // On the page: a polygon for each shape, with a corner on each edge for each mark.
        let shapes = shapes_of(&scene);
        prop_assert_eq!(shapes.len(), seen);
        let corners: usize = shapes.iter().map(|shape| shape.0.len()).sum();
        prop_assert_eq!(corners, 2 * drawn.x.len());
    }
}

// ----- stacked areas (step 23d2) -----

fn stack(placed: Stacking) -> PlotLayer {
    PlotLayer {
        mark: Mark::Area(placed),
        mappings: vec![mapped(Aesthetic::Fill, "g")],
        ..points(0.6)
    }
}

/// The two edges of each mark of the first layer, by its row: (lower, upper).
fn edges_of(scene: &Scene) -> Vec<(usize, f64, f64)> {
    let layer = &scene.panels[0].layers[0];
    let mut all: Vec<(usize, f64, f64)> = layer
        .rows
        .iter()
        .zip(layer.low.iter().zip(&layer.y))
        .map(|(row, (low, high))| (*row, *low, *high))
        .collect();
    all.sort_by_key(|mark| mark.0);
    all
}

#[test]
fn areas_are_stacked_from_the_last_group_up_as_ggplot2_stacks_them() {
    // Three groups at four values of `x`. The reference is ggplot2 4.0.3,
    // `geom_area(position = "stack")` and `position = "fill"` on the same twelve rows: `ymin`
    // and `ymax` of each row, and the limits of the `y` axis.
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let values = [
        [2.0, 4.0, 3.0, 1.0],
        [1.0, 1.0, 2.0, 2.0],
        [3.0, 0.0, 1.0, 5.0],
    ];
    let mut rows: Vec<Row> = Vec::new();
    for (g, of_group) in values.iter().enumerate() {
        for (i, v) in of_group.iter().enumerate() {
            rows.push(cell((i + 1) as f64, *v, g as u8));
        }
    }
    let scene = filled(&rows, stack(Stacking::Stack));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Area(Stacking::Stack));
    assert_eq!(layer.line, [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    assert_eq!(layer.rows, (1..=12).collect::<Vec<usize>>());
    let theirs = [
        (4.0, 6.0),
        (1.0, 5.0),
        (3.0, 6.0),
        (7.0, 8.0),
        (3.0, 4.0),
        (0.0, 1.0),
        (1.0, 3.0),
        (5.0, 7.0),
        (0.0, 3.0),
        (0.0, 0.0),
        (0.0, 1.0),
        (0.0, 5.0),
    ];
    for (ours, theirs) in edges_of(&scene).iter().zip(theirs) {
        assert_eq!((ours.1, ours.2), theirs, "row {}", ours.0);
    }
    assert_eq!(scene.scale("y").unwrap().limits, (-0.4, 8.4));
    assert!(scene.notes.is_empty());
    // On the page the first group is on top, and each area stands on the one under it.
    let shapes = shapes_of(&scene);
    assert_eq!(shapes.len(), 3);
    close(
        &shapes[0].0,
        &[
            (1.0, 6.0),
            (2.0, 5.0),
            (3.0, 6.0),
            (4.0, 8.0),
            (4.0, 7.0),
            (3.0, 3.0),
            (2.0, 1.0),
            (1.0, 4.0),
        ],
    );
    assert_eq!(shapes[0].1, "#E69F00");

    // As shares of the total at each `x`.
    let scene = filled(&rows, stack(Stacking::Share));
    let theirs = [
        (2.0 / 3.0, 1.0),
        (0.2, 1.0),
        (0.5, 1.0),
        (0.875, 1.0),
        (0.5, 2.0 / 3.0),
        (0.0, 0.2),
        (1.0 / 6.0, 0.5),
        (0.625, 0.875),
        (0.0, 0.5),
        (0.0, 0.0),
        (0.0, 1.0 / 6.0),
        (0.0, 0.625),
    ];
    for (ours, theirs) in edges_of(&scene).iter().zip(theirs) {
        assert!(
            (ours.1 - theirs.0).abs() < 1e-15 && (ours.2 - theirs.1).abs() < 1e-15,
            "row {}: {ours:?} is not {theirs:?}",
            ours.0
        );
    }
    assert_eq!(scene.scale("y").unwrap().limits, (-0.05, 1.05));
    let json = scene.json().write();
    assert!(json.contains("\"position\": \"fill\""), "{json}");
    // An area that is not stacked says so too.
    let json = filled(&rows, area()).json().write();
    assert!(json.contains("\"position\": \"overlay\""), "{json}");
}

#[test]
fn values_below_zero_are_stacked_downwards_apart_from_those_above() {
    // The reference is `geom_area(stat = "identity", position = "stack")` of ggplot2 4.0.3.
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![
        cell(1.0, 2.0, 0),
        cell(2.0, -1.0, 0),
        cell(3.0, 3.0, 0),
        cell(1.0, 1.0, 1),
        cell(2.0, -2.0, 1),
        cell(3.0, -1.0, 1),
    ];
    let scene = filled(&rows, stack(Stacking::Stack));
    let edges: Vec<(f64, f64)> = edges_of(&scene).iter().map(|m| (m.1, m.2)).collect();
    assert_eq!(
        edges,
        [
            (1.0, 3.0),
            (-2.0, -3.0),
            (0.0, 3.0),
            (0.0, 1.0),
            (0.0, -2.0),
            (0.0, -1.0)
        ]
    );
    assert!(scene.notes.is_empty());
    // Shares need amounts: the places with a value below zero are left out, and the one
    // that is left has no neighbour.
    let scene = filled(&rows, stack(Stacking::Share));
    assert!(scene.panels[0].layers[0].x.is_empty());
    assert_eq!(
        scene.notes,
        [
            "4 rows at 2 values of x left out of the stack: shares need values of 0 or more and a total above 0",
            "2 rows with no neighbour to be joined to draw no area",
        ]
    );
}

#[test]
fn a_stack_is_built_only_where_every_area_has_one_value() {
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![
        cell(1.0, 1.0, 0),
        cell(2.0, 1.0, 0),
        cell(3.0, 1.0, 0),
        cell(4.0, 1.0, 0),
        cell(5.0, 1.0, 0),
        cell(6.0, 1.0, 0),
        cell(7.0, 1.0, 0),
        cell(1.0, 2.0, 1),
        cell(2.0, 2.0, 1),
        // No row of the second group at 3; no value at 4; two rows at 7.
        (Some(4.0), None, Some(1), None),
        cell(5.0, 2.0, 1),
        cell(6.0, 2.0, 1),
        cell(7.0, 2.0, 1),
        cell(7.0, 9.0, 1),
    ];
    let scene = filled(&rows, stack(Stacking::Stack));
    let layer = &scene.panels[0].layers[0];
    // Two runs, 1 to 2 and 5 to 6, for each of the two groups.
    assert_eq!(layer.x, [1.0, 2.0, 5.0, 6.0, 1.0, 2.0, 5.0, 6.0]);
    assert_eq!(layer.line, [1, 1, 2, 2, 3, 3, 4, 4]);
    assert_eq!(layer.low, [2.0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(layer.y, [3.0, 3.0, 3.0, 3.0, 2.0, 2.0, 2.0, 2.0]);
    assert_eq!(
        scene.notes,
        [
            "1 row with NA in x or y dropped",
            "5 rows at 3 values of x left out of the stack: not every area has one value there",
        ]
    );
    // One group: a stack is the area itself.
    let one: Vec<Row> = rows.iter().take(7).copied().collect();
    let stacked = filled(&one, stack(Stacking::Stack));
    let plain = filled(
        &one,
        PlotLayer {
            mappings: vec![mapped(Aesthetic::Fill, "g")],
            ..area()
        },
    );
    let (a, b) = (&stacked.panels[0].layers[0], &plain.panels[0].layers[0]);
    assert_eq!((&a.x, &a.y, &a.low, &a.line), (&b.x, &b.y, &b.low, &b.line));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn stacked_heights_sum_to_the_totals_and_each_area_stands_on_the_one_under_it(
        table in prop::collection::vec(prop::collection::vec(0.0..50.0f64, 4), 1..5),
        holes in prop::collection::vec((0usize..5, 0usize..4), 0..3),
        share in any::<bool>(),
    ) {
        // A value for each group at each of four places, less a few holes.
        let mut rows: Vec<Row> = Vec::new();
        for (g, of_group) in table.iter().enumerate() {
            for (i, v) in of_group.iter().enumerate() {
                if !holes.contains(&(g, i)) {
                    rows.push((Some(i as f64), Some(*v), Some(g as u8), None));
                }
            }
        }
        let placed = if share { Stacking::Share } else { Stacking::Stack };
        let scene = filled(&rows, stack(placed));
        let layer = &scene.panels[0].layers[0];
        let groups = table.len();
        for place in 0..4 {
            // The marks at this place, from the top group down.
            let marks: Vec<usize> = (0..layer.x.len())
                .filter(|i| layer.x[*i] == place as f64)
                .collect();
            let whole = (0..groups).all(|g| !holes.contains(&(g, place)));
            let total: f64 = table.iter().map(|of_group| of_group[place]).sum();
            if !whole || (share && total <= 0.0) {
                prop_assert!(marks.is_empty(), "a place that cannot be stacked is drawn");
                continue;
            }
            // Drawn, unless the place is left with no neighbour.
            if marks.is_empty() {
                continue;
            }
            prop_assert_eq!(marks.len(), groups);
            // The last group stands on zero, each other on the one after it, and the top of
            // the first is the total, or 1.
            let top = if share { 1.0 } else { total };
            let slack = 1e-9 * top.max(1.0);
            prop_assert_eq!(layer.low[marks[groups - 1]], 0.0);
            for pair in marks.windows(2) {
                prop_assert_eq!(layer.low[pair[0]], layer.y[pair[1]]);
            }
            prop_assert!((layer.y[marks[0]] - top).abs() <= slack);
            // The thickness of each area is its value, or its share.
            for (g, mark) in marks.iter().enumerate() {
                let value = table[g][place] / if share { total } else { 1.0 };
                prop_assert!((layer.y[*mark] - layer.low[*mark] - value).abs() <= slack);
            }
        }
    }
}
