//! `rug` in the scene and on the page (Language Reference, section 7.6, "Marks"; release
//! 0.7.0, step 23e2): a tick at the margin of the panel for each row, at its place on the
//! axis of each side; a rug along one axis asks nothing of the other; the layout draws each
//! tick from the edge of the panel inwards.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::{Aesthetic, Mark, PlotLayer, Sides};
use zeff_plot::drawing::{Primitive, Step};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Paint, RUG_LENGTH, Scene, Sizing, start, with_layer};

fn rug(sides: Sides, alpha: f64) -> PlotLayer {
    PlotLayer {
        mark: Mark::Rug(sides),
        width: Some(0.5),
        ..points(alpha)
    }
}

/// The scene of one rug over a frame with `x`, `y`, a category `g` and a number `z`.
fn rugged(rows: &[Row], sides: Sides) -> Scene {
    let plot = start(
        tried(rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    with_layer(&plot, rug(sides, 1.0)).1
}

/// A tick on the page: its two ends, its colour and its opacity.
type Tick = ((f64, f64), (f64, f64), String, f64);

/// The ticks of the panel.
fn ticks_of(scene: &Scene) -> Vec<Tick> {
    let drawing = layout(scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Path {
                steps,
                stroke: Some(stroke),
                fill: None,
                opacity,
            } => match steps.as_slice() {
                [Step::Move(from), Step::Line(to)] => {
                    assert_eq!(stroke.width, 0.5);
                    Some((*from, *to, stroke.colour.clone(), *opacity))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn a_rug_has_a_tick_for_each_row_on_the_axis_of_its_side() {
    // A row without `y`, one without `x`, and one with an infinite `x`.
    let rows: Vec<Row> = vec![
        (Some(1.0), Some(-1.2), None, None),
        (Some(2.0), None, None, None),
        (None, Some(1.1), None, None),
        (Some(4.0), Some(2.3), None, None),
        (Some(f64::INFINITY), Some(0.5), None, None),
    ];
    // Along `x`: every row with a finite `x`, whatever its `y`.
    let scene = rugged(&rows, Sides::X);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Rug(Sides::X));
    assert_eq!(layer.x, [1.0, 2.0, 4.0]);
    assert!(layer.y.is_empty());
    assert_eq!(layer.rows, [1, 2, 4]);
    assert_eq!(layer.size, Sizing::Constant(RUG_LENGTH));
    assert_eq!(layer.width, 0.5);
    assert_eq!(
        scene.notes,
        [
            "1 row with NA in x dropped",
            "1 row with an infinite value in x dropped"
        ]
    );
    // Along `y`: every row with a finite `y`.
    let scene = rugged(&rows, Sides::Y);
    let layer = &scene.panels[0].layers[0];
    assert!(layer.x.is_empty());
    assert_eq!(layer.y, [-1.2, 1.1, 2.3, 0.5]);
    assert_eq!(layer.rows, [1, 3, 4, 5]);
    assert_eq!(scene.notes, ["1 row with NA in y dropped"]);
    // On both: the rows that have both, which are the rows `points` draws.
    let scene = rugged(&rows, Sides::Both);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 4.0]);
    assert_eq!(layer.y, [-1.2, 2.3]);
    assert_eq!(layer.rows, [1, 4]);
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    let (_, dots) = with_layer(&plot, points(1.0));
    assert_eq!(dots.panels[0].layers[0].rows, layer.rows);
    assert_eq!(dots.notes, scene.notes);
    let json = scene.json().write();
    assert!(json.contains("\"mark\": \"rug\""), "{json}");
    assert!(json.contains("\"sides\": \"both\""), "{json}");
    assert!(json.contains("\"size\": 4.5"), "{json}");
    assert!(json.contains("\"width\": 0.5"), "{json}");
}

#[test]
fn the_ticks_start_at_the_axes_and_point_into_the_panel() {
    // The limits are ggplot2's for the same four rows with `geom_point` and `geom_rug`:
    // 0.85 to 4.15 and -1.375 to 2.475.
    let row = |x: f64, y: f64| (Some(x), Some(y), None, None);
    let rows = [row(1.0, -1.2), row(2.0, 0.4), row(2.0, 1.1), row(4.0, 2.3)];
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let (_, scene) = with_layer(&plot, rug(Sides::Both, 0.5));
    let x = scene.scale("x").unwrap().limits;
    let y = scene.scale("y").unwrap().limits;
    assert!((x.0 - 0.85).abs() < 1e-12 && (x.1 - 4.15).abs() < 1e-12);
    assert!((y.0 + 1.375).abs() < 1e-12 && (y.1 - 2.475).abs() < 1e-12);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (corner, width, height, marks) = panel_of(&drawing);
    // Four points, and over them four ticks at the bottom and four at the left.
    assert_eq!(marks.len(), 12);
    let ticks = ticks_of(&scene);
    assert_eq!(ticks.len(), 8);
    let base = corner.1 + height;
    for (i, (from, to, colour, opacity)) in ticks.iter().enumerate() {
        assert_eq!((colour.as_str(), *opacity), (JOURNAL.mark, 0.5));
        if i < 4 {
            // Upright, from the bottom of the panel, at the `x` of its row.
            assert_eq!((from.1, to.1), (base, base - RUG_LENGTH));
            assert_eq!(from.0, to.0);
            let place = corner.0 + (rows[i].0.unwrap() - x.0) / (x.1 - x.0) * width;
            assert!((from.0 - place).abs() < 1e-9);
        } else {
            // Level, from the left of the panel, at the `y` of its row.
            assert_eq!((from.0, to.0), (corner.0, corner.0 + RUG_LENGTH));
            assert_eq!(from.1, to.1);
            let place = base - (rows[i - 4].1.unwrap() - y.0) / (y.1 - y.0) * height;
            assert!((from.1 - place).abs() < 1e-9);
        }
    }
    // Two rows with one `x` draw two ticks in one place.
    assert_eq!(ticks[1].0, ticks[2].0);
    insta::assert_snapshot!("points_and_a_rug", drawing.describe());
}

#[test]
fn a_rug_along_one_axis_is_a_plot_by_itself() {
    let rows: Vec<Row> = (0..5)
        .map(|i| (Some(f64::from(i)), None, Some((i % 2) as u8), None))
        .collect();
    // Nothing is mapped to `y`: one axis, and the ticks along it.
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Color, "g")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, rug(Sides::X, 1.0));
    assert!(scene.scale("y").is_none());
    assert_eq!(scene.scale("x").unwrap().limits, (-0.2, 4.2));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [0.0, 1.0, 2.0, 3.0, 4.0]);
    // Coloured by the plot's variable, each tick by its row, with a legend of pieces of
    // line.
    assert_eq!(layer.color, Paint::Scaled(vec![1.0, 2.0, 1.0, 2.0, 1.0]));
    assert_eq!(scene.legends[0].marks, [Mark::Rug(Sides::X)]);
    let ticks = ticks_of(&scene);
    assert_eq!(ticks.len(), 5);
    assert_eq!(ticks[0].2, "#E69F00");
    assert_eq!(ticks[1].2, "#56B4E9");
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:color"))
        .collect();
    let pieces = keys
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Segment { .. }))
        .count();
    let dots = keys
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Point { .. }))
        .count();
    assert_eq!((pieces, dots), (2, 0));

    // Under points that share the legend, the key stays a point.
    let both: Vec<Row> = rows.iter().map(|r| (r.0, r.0, r.2, None)).collect();
    let plot = start(
        tried(&both),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let (_, scene) = with_layer(&plot, rug(Sides::Both, 1.0));
    assert_eq!(
        scene.legends[0].marks,
        [Mark::Points, Mark::Rug(Sides::Both)]
    );
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:color"))
        .collect();
    assert!(
        !keys
            .iter()
            .any(|i| matches!(i.primitive, Primitive::Segment { .. }))
    );
    assert_eq!(
        keys.iter()
            .filter(|i| matches!(i.primitive, Primitive::Point { .. }))
            .count(),
        2
    );
}

#[test]
fn a_rug_names_a_variable_of_its_own_and_drops_a_row_without_its_colour() {
    // A rug of `z` along `x`, coloured by the number `y` on the continuous scale.
    let rows: Vec<Row> = vec![
        (Some(1.0), Some(5.0), None, Some(10.0)),
        (Some(2.0), None, None, Some(20.0)),
        (Some(3.0), Some(7.0), None, None),
        (Some(4.0), Some(9.0), None, Some(40.0)),
    ];
    let plot = start(tried(&rows), vec![mapped(Aesthetic::X, "x")], Vec::new());
    let mut layer = rug(Sides::X, 1.0);
    layer.mappings = vec![mapped(Aesthetic::X, "z"), mapped(Aesthetic::Color, "y")];
    let (_, scene) = with_layer(&plot, layer);
    let drawn = &scene.panels[0].layers[0];
    assert_eq!(drawn.x, [10.0, 40.0]);
    assert_eq!(drawn.rows, [1, 4]);
    assert_eq!(drawn.color, Paint::Scaled(vec![5.0, 9.0]));
    assert_eq!(scene.notes, ["2 rows with NA in z or y dropped"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_row_with_a_place_has_its_tick_within_the_panel(
        rows in rows(),
        which in 0u8..3,
    ) {
        let sides = [Sides::X, Sides::Y, Sides::Both][which as usize];
        let scene = rugged(&rows, sides);
        let layer = &scene.panels[0].layers[0];
        let has = |value: Option<f64>| value.is_some_and(f64::is_finite);
        let expected: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| match sides {
                Sides::X => has(r.0),
                Sides::Y => has(r.1),
                Sides::Both => has(r.0) && has(r.1),
            })
            .map(|(i, _)| i + 1)
            .collect();
        prop_assert_eq!(&layer.rows, &expected);
        let (along_x, along_y) = (sides != Sides::Y, sides != Sides::X);
        prop_assert_eq!(layer.x.len(), if along_x { expected.len() } else { 0 });
        prop_assert_eq!(layer.y.len(), if along_y { expected.len() } else { 0 });
        for (i, row) in expected.iter().enumerate() {
            if along_x {
                prop_assert_eq!(Some(layer.x[i]), rows[row - 1].0);
            }
            if along_y {
                prop_assert_eq!(Some(layer.y[i]), rows[row - 1].1);
            }
        }
        // On the page: a tick for each place, starting on an edge of the panel and inside it.
        let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
        let (corner, width, height, _) = panel_of(&drawing);
        let ticks = ticks_of(&scene);
        prop_assert_eq!(ticks.len(), layer.x.len() + layer.y.len());
        for (i, (from, to, ..)) in ticks.iter().enumerate() {
            if i < layer.x.len() {
                prop_assert_eq!(from.1, corner.1 + height);
                prop_assert!(from.0 == to.0 && from.0 >= corner.0 - 1e-9 && from.0 <= corner.0 + width + 1e-9);
            } else {
                prop_assert_eq!(from.0, corner.0);
                prop_assert!(from.1 == to.1 && from.1 >= corner.1 - 1e-9 && from.1 <= corner.1 + height + 1e-9);
            }
        }
    }
}
