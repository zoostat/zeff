//! `hline` and `vline` in the scene and on the page (Language Reference, section 7.6, "Marks";
//! release 0.7.0, step 23e): a reference line stands at a number, or at each value of a
//! variable, on one axis; it takes nothing from the plot but its axes, which grow to hold it;
//! the layout draws it from one edge of the panel to the other, in a type of line of its own.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::{Aesthetic, Mark, PlotConstant, PlotLayer, PlotSetting};
use zeff_plot::drawing::{Head, LineType, Primitive};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Lining, Paint, Scene, start, with_layer};

fn style(name: &'static str) -> PlotConstant {
    PlotConstant {
        aesthetic: Aesthetic::Linetype,
        value: PlotSetting::Line(name),
    }
}

/// A reference line at one place: level for `Mark::Hline`, upright for `Mark::Vline`.
fn at(mark: Mark, place: f64) -> PlotLayer {
    let along = if mark == Mark::Hline {
        Aesthetic::Y
    } else {
        Aesthetic::X
    };
    PlotLayer {
        mark,
        width: Some(0.5),
        constants: vec![
            PlotConstant {
                aesthetic: along,
                value: PlotSetting::Place(place),
            },
            style("dashed"),
        ],
        ..points(1.0)
    }
}

/// Reference lines at each value of a variable.
fn at_each(mark: Mark, variable: &str) -> PlotLayer {
    let along = if mark == Mark::Hline {
        Aesthetic::Y
    } else {
        Aesthetic::X
    };
    PlotLayer {
        mark,
        width: Some(0.5),
        mappings: vec![mapped(along, variable)],
        constants: vec![style("dashed")],
        ..points(1.0)
    }
}

fn row(x: f64, y: f64) -> Row {
    (Some(x), Some(y), None, None)
}

/// Four doses and the change at each, with points on them.
fn doses() -> zeff_core::value::Plot {
    let rows = [row(1.0, -1.2), row(2.0, 0.4), row(3.0, 1.1), row(4.0, 2.3)];
    let plot = start(
        tried(&rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    with_layer(&plot, points(1.0)).0
}

/// A reference line on the page: its two ends, its colour, its width and its type.
type Rule = ((f64, f64), (f64, f64), String, f64, LineType);

/// The reference lines of the panel.
fn lines_of(scene: &Scene) -> Vec<Rule> {
    let drawing = layout(scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Segment {
                from,
                to,
                stroke,
                heads: (Head::Plain, Head::Plain),
            } => Some((*from, *to, stroke.colour.clone(), stroke.width, stroke.dash)),
            _ => None,
        })
        .collect()
}

/// The limits of an axis are R's to within the last digits of the arithmetic.
fn near(scene: &Scene, axis: &str, theirs: (f64, f64)) {
    let ours = scene.scale(axis).unwrap().limits;
    assert!(
        (ours.0 - theirs.0).abs() < 1e-12 && (ours.1 - theirs.1).abs() < 1e-12,
        "{axis}: {ours:?} is not {theirs:?}"
    );
}

#[test]
fn a_reference_line_stands_at_its_place_and_the_axis_holds_it_as_ggplot2_does() {
    // The reference is ggplot2 4.0.3 on the same four rows with `geom_point`: the limits of
    // the axes with `geom_hline(yintercept = 0)`, which is among the points, with
    // `yintercept = 10`, which is beyond them, and with `geom_vline(xintercept = -2)` and
    // `geom_hline(yintercept = c(-3, 3))`.
    let plot = doses();
    let (_, scene) = with_layer(&plot, at(Mark::Hline, 0.0));
    let layer = &scene.panels[0].layers[1];
    assert_eq!(layer.mark, Mark::Hline);
    assert!(layer.x.is_empty());
    assert_eq!(layer.y, [0.0]);
    // One line, with no row behind it.
    assert!(layer.rows.is_empty());
    assert_eq!(layer.linetype, Lining::Constant("dashed"));
    assert_eq!(layer.width, 0.5);
    assert_eq!(layer.color, Paint::Theme);
    near(&scene, "x", (0.85, 4.15));
    near(&scene, "y", (-1.375, 2.475));
    assert!(scene.notes.is_empty());
    let (_, scene) = with_layer(&plot, at(Mark::Hline, 10.0));
    near(&scene, "y", (-1.76, 10.56));
    let (more, _) = with_layer(&plot, at(Mark::Vline, -2.0));
    let (more, _) = with_layer(&more, at(Mark::Hline, -3.0));
    let (_, scene) = with_layer(&more, at(Mark::Hline, 3.0));
    assert_eq!(scene.scale("x").unwrap().limits, (-2.3, 4.3));
    let y = scene.scale("y").unwrap().limits;
    assert!(
        (y.0 + 3.3).abs() < 1e-12 && (y.1 - 3.3).abs() < 1e-12,
        "{y:?}"
    );
    let upright = &scene.panels[0].layers[1];
    assert_eq!(upright.mark, Mark::Vline);
    assert_eq!((upright.x.as_slice(), upright.y.len()), (&[-2.0][..], 0));
}

#[test]
fn a_reference_line_goes_from_edge_to_edge_of_the_panel() {
    let plot = doses();
    let (plot, _) = with_layer(&plot, at(Mark::Hline, 0.0));
    let mut red = at(Mark::Vline, 2.5);
    red.width = Some(1.0);
    red.constants[1] = style("dotted");
    red.constants.push(PlotConstant {
        aesthetic: Aesthetic::Color,
        value: PlotSetting::Colour {
            hex: "#B22222".to_string(),
            written: "firebrick".to_string(),
        },
    });
    let (_, scene) = with_layer(&plot, red);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (corner, width, height, marks) = panel_of(&drawing);
    // Four points, then the two lines over them.
    assert_eq!(marks.len(), 6);
    let lines = lines_of(&scene);
    assert_eq!(lines.len(), 2);
    let (from, to, colour, wide, dash) = &lines[0];
    assert_eq!((from.0, to.0), (corner.0, corner.0 + width));
    assert_eq!(from.1, to.1);
    // Zero is 1.375 of the 3.85 the axis spans above its lower end.
    let zero = corner.1 + height - 1.375 / 3.85 * height;
    assert!((from.1 - zero).abs() < 1e-9, "{from:?}");
    assert_eq!(
        (colour.as_str(), *wide, *dash),
        (JOURNAL.mark, 0.5, LineType::Dashed)
    );
    let (from, to, colour, wide, dash) = &lines[1];
    assert_eq!((from.1, to.1), (corner.1, corner.1 + height));
    assert_eq!(from.0, to.0);
    let place = corner.0 + (2.5 - 0.85) / 3.3 * width;
    assert!((from.0 - place).abs() < 1e-9, "{from:?}");
    assert_eq!(
        (colour.as_str(), *wide, *dash),
        ("#B22222", 1.0, LineType::Dotted)
    );
    insta::assert_snapshot!("two_reference_lines", drawing.describe());
}

#[test]
fn a_line_at_each_value_of_a_variable_of_a_frame_of_its_own() {
    // The frame of the lines holds none of the plot's variables: `z` for the places, `g` for
    // a colour.
    let own = frame(
        "limits",
        vec![
            (
                "level",
                floats(&[Some(-1.0), None, Some(1.0), Some(f64::INFINITY), Some(1.0)]),
            ),
            (
                "kind",
                zeff_columns::Column::strs(
                    ["lower", "lost", "upper", "far", "upper"]
                        .into_iter()
                        .map(|s| Some(s.to_string())),
                ),
            ),
        ],
    );
    let mut layer = at_each(Mark::Hline, "level");
    layer.data = Some(own);
    layer.mappings.push(mapped(Aesthetic::Color, "kind"));
    let plot = doses();
    let (_, scene) = with_layer(&plot, layer);
    let lines = &scene.panels[0].layers[1];
    assert_eq!(lines.y, [-1.0, 1.0, 1.0]);
    assert!(lines.x.is_empty());
    assert_eq!(lines.rows, [1, 3, 5]);
    assert_eq!(lines.data.as_deref(), Some("limits"));
    assert_eq!(
        scene.notes,
        [
            "1 row with NA in level dropped",
            "1 row with an infinite value in level dropped",
        ]
    );
    // Coloured by the category of each row, with a legend of pieces of line; the points,
    // which name no colour, stay as they were.
    assert_eq!(lines.color, Paint::Scaled(vec![1.0, 2.0, 2.0]));
    assert_eq!(scene.panels[0].layers[0].color, Paint::Theme);
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].marks, [Mark::Hline]);
    assert_eq!(scene.scale("color").unwrap().labels, ["lower", "upper"]);
    let drawn = lines_of(&scene);
    assert_eq!(drawn.len(), 3);
    assert_eq!(drawn[0].2, "#E69F00");
    assert_eq!(drawn[2].2, "#56B4E9");
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
}

#[test]
fn a_reference_line_takes_nothing_from_the_plot_but_its_axes() {
    // The plot colours by `g` and sets a size; the points read both, the line neither.
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![cell(1.0, 1.0, 0), cell(2.0, 2.0, 1), cell(3.0, 3.0, 0)];
    let plot = start(
        tried(&rows),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let (plot, scene) = with_layer(&plot, at(Mark::Vline, 2.0));
    assert_eq!(scene.panels[0].layers[1].color, Paint::Theme);
    assert_eq!(scene.legends[0].marks, [Mark::Points]);
    // Nor does a line at each value of a variable of the plot's own frame.
    let (_, scene) = with_layer(&plot, at_each(Mark::Hline, "y"));
    let each = &scene.panels[0].layers[2];
    assert_eq!(each.y, [1.0, 2.0, 3.0]);
    assert_eq!(each.color, Paint::Theme);
    assert_eq!(scene.legends[0].marks, [Mark::Points]);

    // A plot that maps nothing to `y` takes its axis from the variable of the lines.
    let plot = start(tried(&rows), vec![mapped(Aesthetic::X, "x")], Vec::new());
    let (_, scene) = with_layer(&plot, at_each(Mark::Hline, "y"));
    let y = scene.scale("y").unwrap();
    assert_eq!((y.title.as_str(), y.limits), ("y", (0.9, 3.1)));
    // With one axis alone the lines are still drawn across the panel.
    assert_eq!(lines_of(&scene).len(), 3);
}

#[test]
fn the_scene_of_reference_lines_says_their_type_and_width() {
    let (_, scene) = with_layer(&doses(), at(Mark::Hline, 0.0));
    let json = scene.json().write();
    assert!(json.contains("\"mark\": \"hline\""), "{json}");
    assert!(
        json.contains("\"x\": [], \"y\": [0]") || json.contains("\"x\": [],"),
        "{json}"
    );
    assert!(json.contains("\"linetype\": \"dashed\""), "{json}");
    assert!(json.contains("\"width\": 0.5"), "{json}");
    assert!(!json.contains("\"line\":"), "{json}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_reference_line_is_within_its_axis_and_spans_the_panel(
        rows in rows(),
        place in -1e6..1e6f64,
        level in any::<bool>(),
        each in any::<bool>(),
    ) {
        let mark = if level { Mark::Hline } else { Mark::Vline };
        let plot = start(
            tried(&rows),
            vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
            Vec::new(),
        );
        let (plot, _) = with_layer(&plot, points(1.0));
        let layer = if each { at_each(mark, "z") } else { at(mark, place) };
        let (_, scene) = with_layer(&plot, layer);
        let drawn = &scene.panels[0].layers[1];
        let (places, none) = if level { (&drawn.y, &drawn.x) } else { (&drawn.x, &drawn.y) };
        prop_assert!(none.is_empty());
        // One line for the number; one for each row with a finite value of the variable.
        let finite: Vec<f64> = rows.iter().filter_map(|r| r.3).filter(|z| z.is_finite()).collect();
        if each {
            prop_assert_eq!(places, &finite);
            prop_assert_eq!(drawn.rows.len(), finite.len());
        } else {
            prop_assert_eq!(places, &vec![place]);
            prop_assert!(drawn.rows.is_empty());
        }
        // The axis holds every line.
        let (lo, hi) = scene.scale(if level { "y" } else { "x" }).unwrap().limits;
        for value in places {
            prop_assert!(lo <= *value && *value <= hi, "{} outside {} to {}", value, lo, hi);
        }
        // On the page each goes from one edge of the panel to the other, inside it.
        let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
        let (corner, width, height, _) = panel_of(&drawing);
        let lines = lines_of(&scene);
        prop_assert_eq!(lines.len(), places.len());
        for (from, to, ..) in lines {
            if level {
                prop_assert_eq!((from.0, to.0), (corner.0, corner.0 + width));
                prop_assert!(from.1 == to.1 && from.1 >= corner.1 - 1e-9 && from.1 <= corner.1 + height + 1e-9);
            } else {
                prop_assert_eq!((from.1, to.1), (corner.1, corner.1 + height));
                prop_assert!(from.0 == to.0 && from.0 >= corner.0 - 1e-9 && from.0 <= corner.0 + width + 1e-9);
            }
        }
    }
}
