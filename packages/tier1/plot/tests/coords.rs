//! The coordinates of a plot (Language Reference, section 7.6, "Coordinates": `coord`;
//! release 0.7.0, step 27a): cartesian, and flipped, where `x` runs up the side of the panel
//! and `y` along its foot and every mark is turned with them. The scene carries the
//! coordinates and puts each axis on its side; the layout draws the marks in the frame of
//! the data and turns them onto the page.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{
    Aesthetic, Arrangement, AxisKind, Coord, Frame, LegendPlace, Mark, Plot, PlotAxis,
    PlotConstant, PlotLayer, PlotSetting, Sides, Stacking, Stairs,
};
use zeff_plot::drawing::{Drawing, Primitive, Step};
use zeff_plot::layout::{JOURNAL, LayoutError, SMALLEST_PANEL, Size, layout};
use zeff_plot::scale::share;
use zeff_plot::scene::{Position, RUG_LENGTH, Scene, start, with_layer};

/// The rows of a property, or a small frame, as a plot of `x` and `y` in the coordinates.
fn plot_of(frame: Frame, x: &str, y: &str, coord: Coord) -> Plot {
    let mut plot: Plot = start(
        frame,
        vec![mapped(Aesthetic::X, x), mapped(Aesthetic::Y, y)],
        Vec::new(),
    );
    plot.coord = coord;
    plot
}

fn with(plot: &Plot, mark: Mark) -> Scene {
    let layer = PlotLayer {
        mark,
        ..points(1.0)
    };
    with_layer(plot, layer).1
}

fn drawn(scene: &Scene) -> Drawing {
    layout(scene, Size::OneColumn, &JOURNAL).unwrap()
}

/// The centres of the points in the panel of a drawing.
fn centres(drawing: &Drawing) -> Vec<(f64, f64)> {
    let (_, _, _, marks) = panel_of(drawing);
    marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { at, .. } => Some(*at),
            _ => None,
        })
        .collect()
}

/// The texts of the axis of a scale: each with where it stands.
fn axis_texts(drawing: &Drawing, id: &str) -> Vec<(String, (f64, f64))> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some(&format!("axis:{id}")))
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, spans, .. } => spans.first().map(|span| (span.text.clone(), *at)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_scene_carries_the_coordinates_and_puts_each_axis_on_its_side() {
    let usual = with(
        &plot_of(small("d"), "dose", "response", Coord::Cartesian),
        Mark::Points,
    );
    let side = |scene: &Scene, id: &str| {
        scene
            .axes
            .iter()
            .find(|axis| axis.scale == id)
            .map(|axis| axis.position)
    };
    assert_eq!(usual.coord, Coord::Cartesian);
    assert_eq!(
        (side(&usual, "x"), side(&usual, "y")),
        (Some(Position::Bottom), Some(Position::Left))
    );
    assert!(!usual.json().write().contains("\"coord\""));
    let flipped = with(
        &plot_of(small("d"), "dose", "response", Coord::Flip),
        Mark::Points,
    );
    assert_eq!(flipped.coord, Coord::Flip);
    assert_eq!(
        (side(&flipped, "x"), side(&flipped, "y")),
        (Some(Position::Left), Some(Position::Bottom))
    );
    assert!(flipped.json().write().contains("\"coord\": \"flip\""));
    // The scales and the marks are those of the data: only the page differs.
    assert_eq!(
        (&flipped.scales, &flipped.panels),
        (&usual.scales, &usual.panels)
    );
}

#[test]
fn bars_lie_along_the_page_with_every_category_written() {
    let names: Vec<String> = (0..12)
        .map(|i| format!("an industry of some length {i:02}"))
        .collect();
    let frame = frame(
        "d",
        vec![
            (
                "group",
                Column::strs(names.iter().map(|name| Some(name.as_str()))),
            ),
            (
                "n",
                numbers(&(0..12).map(|i| (i % 5) as f64 + 1.0).collect::<Vec<f64>>()),
            ),
        ],
    );
    let scene = with(
        &plot_of(frame, "group", "n", Coord::Flip),
        Mark::Bars(Arrangement::Stack),
    );
    let drawing = drawn(&scene);
    let (panel, width, height, marks) = panel_of(&drawing);
    // Twelve bars, each a rectangle that starts at the zero of `n` and is longer than it
    // is high; one above the other in the order of the categories, the first at the foot.
    let (n, x) = (scene.scale("y").unwrap(), scene.scale("x").unwrap());
    let zero = panel.0 + share(n.kind, n.limits, 0.0) * width;
    let bars: Vec<(f64, f64, f64, f64)> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Polygon { rings, .. } => {
                let ring = &rings[0];
                let (xs, ys): (Vec<f64>, Vec<f64>) = ring.iter().copied().unzip();
                let low = |v: &[f64]| v.iter().copied().fold(f64::MAX, f64::min);
                let high = |v: &[f64]| v.iter().copied().fold(f64::MIN, f64::max);
                Some((low(&xs), high(&xs), low(&ys), high(&ys)))
            }
            _ => None,
        })
        .collect();
    assert_eq!(bars.len(), 12);
    for (i, (left, right, top, bottom)) in bars.iter().enumerate() {
        assert!(
            (left - zero).abs() < 1e-9,
            "bar {i} starts at {left}, not {zero}"
        );
        assert!(right - left > bottom - top);
        let place = share(x.kind, x.limits, (i + 1) as f64);
        let middle = panel.1 + height - place * height;
        assert!(((top + bottom) / 2.0 - middle).abs() < 1e-9);
    }
    // Every category is written beside its bar, on one line, at the left of the panel.
    let written = axis_texts(&drawing, "x");
    for name in &names {
        let at = written
            .iter()
            .find(|(text, _)| text == name)
            .unwrap_or_else(|| panic!("`{name}` is not written"))
            .1;
        assert!(at.0 < panel.0);
    }
    // The numbers are along the foot.
    let numbers_at: Vec<f64> = axis_texts(&drawing, "y")
        .iter()
        .filter(|(text, _)| text.parse::<f64>().is_ok())
        .map(|(_, at)| at.1)
        .collect();
    assert!(numbers_at.len() >= 3 && numbers_at.iter().all(|y| *y > panel.1 + height));
    insta::assert_snapshot!("twelve_bars_lying", drawing.describe());
}

#[test]
fn a_line_across_becomes_a_line_down_and_a_text_stays_upright() {
    let plot = plot_of(small("d"), "dose", "response", Coord::Flip);
    let (plot, _) = with_layer(&plot, points(1.0));
    let mut level = PlotLayer {
        mark: Mark::Hline,
        ..points(1.0)
    };
    level.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Y,
        value: PlotSetting::Place(6.0),
    }];
    let (plot, _) = with_layer(&plot, level);
    let mut words = PlotLayer {
        mark: Mark::Text,
        ..points(1.0)
    };
    words.mappings = vec![mapped(Aesthetic::Label, "arm")];
    let (_, scene) = with_layer(&plot, words);
    let drawing = drawn(&scene);
    let (panel, width, height, marks) = panel_of(&drawing);
    // The line at `y` = 6 runs from the top of the panel to its foot, at the place of 6
    // along the foot.
    let y = scene.scale("y").unwrap();
    let lines: Vec<((f64, f64), (f64, f64))> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Segment { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 1);
    let (from, to) = lines[0];
    let at = panel.0 + share(y.kind, y.limits, 6.0) * width;
    assert!((from.0 - at).abs() < 1e-9 && (to.0 - at).abs() < 1e-9);
    assert!((from.1 - to.1).abs() > height - 1e-9);
    // Each text is not turned, and stands at the place of its point.
    let dots = centres(&drawing);
    let texts: Vec<(f64, (f64, f64))> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, turn, .. } => Some((*turn, *at)),
            _ => None,
        })
        .collect();
    assert_eq!((dots.len(), texts.len()), (6, 6));
    for ((turn, at), dot) in texts.iter().zip(&dots) {
        assert_eq!(*turn, 0.0);
        assert!((at.0 - dot.0).abs() < 1e-9);
        assert!((at.1 - dot.1).abs() < 5.0);
    }
}

#[test]
fn a_log_scale_and_a_legend_go_with_the_flip() {
    // A log scale of `x` runs up the side: equal ratios are equal distances there.
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 10.0, 100.0, 1000.0])),
            ("y", numbers(&[1.0, 2.0, 3.0, 4.0])),
        ],
    );
    let mut plot = plot_of(frame, "x", "y", Coord::Flip);
    plot.axes.push(PlotAxis::of(Aesthetic::X, AxisKind::Log));
    let drawing = drawn(&with(&plot, Mark::Points));
    let dots = centres(&drawing);
    let steps: Vec<f64> = dots.windows(2).map(|pair| pair[0].1 - pair[1].1).collect();
    assert!(steps[0] > 0.0);
    assert!((steps[0] - steps[1]).abs() < 1e-9 && (steps[1] - steps[2]).abs() < 1e-9);
    // `direct` names lines at their right ends: flipped, the legend stands at the right.
    let mut plot: Plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
            mapped(Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    plot.coord = Coord::Flip;
    plot.legend = LegendPlace::Direct;
    let direct = drawn(&with(&plot, Mark::Lines));
    plot.legend = LegendPlace::Right;
    let right = drawn(&with(&plot, Mark::Lines));
    assert_eq!(direct.describe(), right.describe());
}

/// Every corner of the marks in the panel of a drawing, each as its share of the panel
/// across and up, in order.
fn corners(drawing: &Drawing) -> Vec<(f64, f64)> {
    let (panel, width, height, marks) = panel_of(drawing);
    let mut all: Vec<(f64, f64)> = Vec::new();
    for item in marks {
        match &item.primitive {
            Primitive::Point { at, .. } => all.push(*at),
            Primitive::Segment { from, to, .. } => all.extend([*from, *to]),
            Primitive::Polygon { rings, .. } => all.extend(rings.iter().flatten().copied()),
            Primitive::Rectangle {
                at, width, height, ..
            } => all.extend([*at, (at.0 + width, at.1 + height)]),
            Primitive::Path { steps, .. } => {
                all.extend(steps.iter().filter_map(|step| match step {
                    Step::Move(to) | Step::Line(to) => Some(*to),
                    _ => None,
                }))
            }
            _ => {}
        }
    }
    let mut shares: Vec<(f64, f64)> = all
        .iter()
        .map(|at| ((at.0 - panel.0) / width, (panel.1 + height - at.1) / height))
        .collect();
    shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
    shares
}

#[test]
fn every_mark_is_turned_with_the_axes() {
    // Each mark that is drawn in the units of the data: its corners in the flipped panel
    // are its corners in the plain one, the share across and the share up exchanged.
    let layer = |mark: Mark| PlotLayer {
        mark,
        ..points(1.0)
    };
    let band = PlotLayer {
        mappings: vec![
            mapped(Aesthetic::Low, "response"),
            mapped(Aesthetic::High, "weight"),
        ],
        ..layer(Mark::Ribbon)
    };
    let mut level = layer(Mark::Hline);
    level.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Y,
        value: PlotSetting::Place(6.0),
    }];
    let mut upright = layer(Mark::Vline);
    upright.constants = vec![PlotConstant {
        aesthetic: Aesthetic::X,
        value: PlotSetting::Place(2.5),
    }];
    for (name, x, layer) in [
        ("points", "dose", layer(Mark::Points)),
        ("lines", "dose", layer(Mark::Lines)),
        ("step", "dose", layer(Mark::Step(Stairs::LevelFirst))),
        (
            "step, upright first",
            "dose",
            layer(Mark::Step(Stairs::UprightFirst)),
        ),
        ("step, halfway", "dose", layer(Mark::Step(Stairs::Halfway))),
        ("area", "dose", layer(Mark::Area(Stacking::Overlay))),
        ("ribbon", "dose", band),
        ("hline", "dose", level),
        ("vline", "dose", upright),
        ("bars", "arm", layer(Mark::Bars(Arrangement::Stack))),
        (
            "bars side by side",
            "arm",
            layer(Mark::Bars(Arrangement::Dodge)),
        ),
        ("tile", "arm", layer(Mark::Tile)),
    ] {
        let y = if name == "tile" { "dose" } else { "response" };
        let draw = |coord: Coord| {
            let mut plot = plot_of(small("d"), x, y, coord);
            if name == "tile" {
                plot.mappings.push(mapped(Aesthetic::Fill, "weight"));
            }
            drawn(&with_layer(&plot, layer.clone()).1)
        };
        let (plain, flipped) = (
            corners(&draw(Coord::Cartesian)),
            corners(&draw(Coord::Flip)),
        );
        assert!(!plain.is_empty(), "{name} draws nothing");
        let mut exchanged: Vec<(f64, f64)> = plain.iter().map(|at| (at.1, at.0)).collect();
        exchanged.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(exchanged.len(), flipped.len(), "{name}");
        for (wanted, found) in exchanged.iter().zip(&flipped) {
            assert!(
                (wanted.0 - found.0).abs() < 1e-9 && (wanted.1 - found.1).abs() < 1e-9,
                "{name}: {found:?} where {wanted:?} was looked for"
            );
        }
    }
}

#[test]
fn a_flipped_plot_with_its_two_positions_exchanged_is_the_upright_picture() {
    // `plot(x: response, y: arm)` flipped is `plot(x: arm, y: response)`: the same marks,
    // the same axes with their labels and titles, the same gridlines. Only the names of
    // the axes, which no figure shows, tell the two drawings apart.
    let lines = |drawing: &Drawing| {
        let text = drawing
            .describe()
            .replace("[axis:x]", "[axis]")
            .replace("[axis:y]", "[axis]");
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        lines.sort();
        lines
    };
    for (across, up) in [("arm", "response"), ("dose", "response"), ("dose", "arm")] {
        let upright = drawn(&with(
            &plot_of(small("d"), across, up, Coord::Cartesian),
            Mark::Points,
        ));
        let flipped = drawn(&with(
            &plot_of(small("d"), up, across, Coord::Flip),
            Mark::Points,
        ));
        assert_eq!(lines(&upright), lines(&flipped), "{across} and {up}");
    }
}

#[test]
fn a_rug_of_x_stands_at_the_left_of_a_flipped_panel() {
    let plot = plot_of(small("d"), "dose", "response", Coord::Flip);
    let rug = PlotLayer {
        mark: Mark::Rug(Sides::X),
        width: Some(0.5),
        ..points(1.0)
    };
    let scene = with_layer(&plot, rug).1;
    let drawing = drawn(&scene);
    let (panel, _, height, marks) = panel_of(&drawing);
    let x = scene.scale("x").unwrap();
    let mut ends: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for item in marks {
        match &item.primitive {
            Primitive::Segment { from, to, .. } => ends.push((*from, *to)),
            Primitive::Path { steps, .. } => {
                let at: Vec<(f64, f64)> = steps
                    .iter()
                    .filter_map(|step| match step {
                        Step::Move(to) | Step::Line(to) => Some(*to),
                        _ => None,
                    })
                    .collect();
                ends.extend(at.chunks(2).map(|pair| (pair[0], pair[1])));
            }
            _ => {}
        }
    }
    assert_eq!(ends.len(), 6);
    for (i, (from, to)) in ends.iter().enumerate() {
        // Each tick runs across from the left edge of the panel, at the height of its dose.
        let (near, far) = (from.0.min(to.0), from.0.max(to.0));
        assert!((near - panel.0).abs() < 1e-9, "{from:?} to {to:?}");
        assert!((far - near - RUG_LENGTH).abs() < 1e-9);
        let wanted = panel.1 + height - share(x.kind, x.limits, (i + 1) as f64) * height;
        assert!((from.1 - wanted).abs() < 1e-9 && (to.1 - wanted).abs() < 1e-9);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows: a point of the flipped plot stands where the point of the plain
    /// plot stands, the two coordinates exchanged. Its distance from the foot of the panel
    /// is the share of `x`, and its distance from the left the share of `y`.
    #[test]
    fn a_flipped_point_stands_at_the_shares_of_its_two_values(rows in rows()) {
        let frame = tried(&rows);
        let plain = with(&plot_of(frame.clone(), "x", "y", Coord::Cartesian), Mark::Points);
        let flipped = with(&plot_of(frame, "x", "y", Coord::Flip), Mark::Points);
        prop_assume!(!plain.panels[0].layers[0].x.is_empty());
        let (Ok(usual), Ok(turned)) = (
            layout(&plain, Size::OneColumn, &JOURNAL),
            layout(&flipped, Size::OneColumn, &JOURNAL),
        ) else {
            return Ok(());
        };
        let (a, b) = (centres(&usual), centres(&turned));
        prop_assert_eq!(a.len(), b.len());
        let (panel_a, width_a, height_a, _) = panel_of(&usual);
        let (panel_b, width_b, height_b, _) = panel_of(&turned);
        for (plain, flipped) in a.iter().zip(&b) {
            let across = (plain.0 - panel_a.0) / width_a;
            let up = (panel_a.1 + height_a - plain.1) / height_a;
            let across_flipped = (flipped.0 - panel_b.0) / width_b;
            let up_flipped = (panel_b.1 + height_b - flipped.1) / height_b;
            prop_assert!((across - up_flipped).abs() < 1e-9, "{} against {}", across, up_flipped);
            prop_assert!((up - across_flipped).abs() < 1e-9, "{} against {}", up, across_flipped);
        }
    }
}

// ----- fixed coordinates (step 27b) -----

/// How long one unit of each axis is on the page, in points: the panel's width over the
/// units `x` runs over, and its height over the units `y` runs over.
fn lengths_of_a_unit(scene: &Scene, drawing: &Drawing) -> (f64, f64) {
    let (_, width, height, _) = panel_of(drawing);
    let (x, y) = (scene.scale("x").unwrap(), scene.scale("y").unwrap());
    (
        width / zeff_plot::scale::units(x.kind, x.limits),
        height / zeff_plot::scale::units(y.kind, y.limits),
    )
}

/// A frame of two numbers, `x` and `y`.
fn pairs(x: &[f64], y: &[f64]) -> Frame {
    frame("d", vec![("x", numbers(x)), ("y", numbers(y))])
}

#[test]
fn the_scene_carries_the_ratio_and_is_otherwise_the_cartesian_scene() {
    let plain = with(
        &plot_of(small("d"), "dose", "response", Coord::Cartesian),
        Mark::Points,
    );
    let fixed = with(
        &plot_of(small("d"), "dose", "response", Coord::Fixed(2.0)),
        Mark::Points,
    );
    assert_eq!(fixed.coord, Coord::Fixed(2.0));
    assert_eq!(
        (&fixed.scales, &fixed.axes, &fixed.panels),
        (&plain.scales, &plain.axes, &plain.panels)
    );
    let json = fixed.json().write();
    assert!(json.contains("\"coord\": \"fixed\""), "{json}");
    assert!(json.contains("\"ratio\": 2"), "{json}");
    assert!(!plain.json().write().contains("\"ratio\""));
}

#[test]
fn a_square_of_the_data_is_a_square_on_the_page() {
    // A unit square: both axes run over the same units, so the panel is as high as it is
    // wide, and the four corners of the square are the corners of a square on the page.
    let square = pairs(&[0.0, 1.0, 1.0, 0.0], &[0.0, 0.0, 1.0, 1.0]);
    let scene = with(
        &plot_of(square.clone(), "x", "y", Coord::Fixed(1.0)),
        Mark::Points,
    );
    let drawing = drawn(&scene);
    let (_, width, height, _) = panel_of(&drawing);
    assert!((width - height).abs() < 1e-9, "{width} by {height}");
    let at = centres(&drawing);
    let (across, up) = (at[1].0 - at[0].0, at[1].1 - at[2].1);
    assert!(
        across > 0.0 && (across - up).abs() < 1e-9,
        "{across} and {up}"
    );
    // At a ratio of 2 a unit of `y` is twice as long as a unit of `x`, and at a half, half
    // as long: the square is a rectangle twice as high, or twice as wide.
    for ratio in [2.0, 0.5, 3.0, 0.2] {
        let scene = with(
            &plot_of(square.clone(), "x", "y", Coord::Fixed(ratio)),
            Mark::Points,
        );
        let drawing = drawn(&scene);
        let (x, y) = lengths_of_a_unit(&scene, &drawing);
        assert!((y / x - ratio).abs() < 1e-9, "{ratio}: {y} against {x}");
        let at = centres(&drawing);
        let (across, up) = (at[1].0 - at[0].0, at[1].1 - at[2].1);
        assert!((up / across - ratio).abs() < 1e-9, "{ratio}");
    }
    insta::assert_snapshot!("a_square_of_the_data", drawn(&scene).describe());
}

#[test]
fn the_fixed_panel_stands_in_the_middle_of_its_room_with_all_that_goes_with_it() {
    // The room is the panel the same plot has without fixed coordinates. Twice as many
    // units along `y` as along `x`: the panel keeps the height of its room and stands in
    // the middle of its width; half as many, and it keeps the width.
    let colours = |coord: Coord, y: &[f64]| {
        let frame = frame(
            "d",
            vec![
                ("x", numbers(&[0.0, 1.0, 2.0, 3.0])),
                ("y", numbers(y)),
                ("g", Column::strs(["a", "b", "a", "b"].map(Some))),
            ],
        );
        let mut plot: Plot = start(
            frame,
            vec![
                mapped(Aesthetic::X, "x"),
                mapped(Aesthetic::Y, "y"),
                mapped(Aesthetic::Color, "g"),
            ],
            Vec::new(),
        );
        plot.coord = coord;
        drawn(&with(&plot, Mark::Points))
    };
    // What goes with the panel: the title of `y` at its left, and the legend at its right.
    let beside = |drawing: &Drawing| {
        let (panel, width, _, _) = panel_of(drawing);
        let mut offsets: Vec<f64> = Vec::new();
        for item in &drawing.items {
            let name = item.name.as_deref().unwrap_or("");
            let edge = if name == "axis:y" {
                panel.0
            } else if name.starts_with("legend") {
                panel.0 + width
            } else {
                continue;
            };
            match &item.primitive {
                Primitive::Text { at, .. } | Primitive::Point { at, .. } => {
                    offsets.push(at.0 - edge);
                }
                Primitive::Segment { from, .. } => offsets.push(from.0 - edge),
                _ => {}
            }
        }
        offsets
    };
    for (y, tall) in [
        (&[0.0, 2.0, 4.0, 6.0][..], true),
        (&[0.0, 0.5, 1.0, 1.5][..], false),
    ] {
        let (room, fixed) = (colours(Coord::Cartesian, y), colours(Coord::Fixed(1.0), y));
        let (at, wide, high, _) = panel_of(&room);
        let (to, width, height, _) = panel_of(&fixed);
        if tall {
            assert!((height - high).abs() < 1e-9 && width < wide);
        } else {
            assert!((width - wide).abs() < 1e-9 && height < high);
        }
        assert!((at.0 + wide / 2.0 - (to.0 + width / 2.0)).abs() < 1e-9);
        assert!((at.1 + high / 2.0 - (to.1 + height / 2.0)).abs() < 1e-9);
        // The labels and the title of `y`, and the legend, are as far from the panel as
        // they were.
        let (was, is) = (beside(&room), beside(&fixed));
        assert!(was.len() > 6 && was.len() == is.len());
        for (a, b) in was.iter().zip(&is) {
            assert!((a - b).abs() < 1e-9, "{a} against {b}");
        }
    }
}

#[test]
fn a_unit_is_a_place_a_power_of_ten_a_square_root_or_a_day() {
    // Categories: a place of `x` is as long as a unit of `y`.
    let scene = with(
        &plot_of(small("d"), "arm", "dose", Coord::Fixed(1.0)),
        Mark::Points,
    );
    let drawing = drawn(&scene);
    let at = centres(&drawing);
    // Row 1 is `placebo` at a dose of 1 and row 2 `drug` at a dose of 2: one place apart
    // and one unit apart.
    let (across, up) = ((at[0].0 - at[1].0).abs(), (at[0].1 - at[1].1).abs());
    assert!(
        across > 0.0 && (across - up).abs() < 1e-9,
        "{across} and {up}"
    );
    // A log axis: from 1 to 10 and from 10 to 100 are each as long as a unit of `y`.
    let mut plot = plot_of(
        pairs(&[1.0, 10.0, 100.0], &[0.0, 1.0, 2.0]),
        "x",
        "y",
        Coord::Fixed(1.0),
    );
    plot.axes.push(PlotAxis::of(Aesthetic::X, AxisKind::Log));
    let at = centres(&drawn(&with(&plot, Mark::Points)));
    for pair in at.windows(2) {
        let (across, up) = (pair[1].0 - pair[0].0, pair[0].1 - pair[1].1);
        assert!(
            across > 0.0 && (across - up).abs() < 1e-9,
            "{across} and {up}"
        );
    }
    // A square-root axis: from 0 to 1, from 1 to 4 and from 4 to 9 are each a unit.
    let mut plot = plot_of(
        pairs(&[0.0, 1.0, 2.0, 3.0], &[0.0, 1.0, 4.0, 9.0]),
        "x",
        "y",
        Coord::Fixed(1.0),
    );
    plot.axes.push(PlotAxis::of(Aesthetic::Y, AxisKind::Sqrt));
    let at = centres(&drawn(&with(&plot, Mark::Points)));
    for pair in at.windows(2) {
        let (across, up) = (pair[1].0 - pair[0].0, pair[0].1 - pair[1].1);
        assert!(
            across > 0.0 && (across - up).abs() < 1e-9,
            "{across} and {up}"
        );
    }
    // An axis of dates: a day is as long as a unit of `y`.
    let days = frame(
        "d",
        vec![
            (
                "day",
                Column::dates([Some(19_800), Some(19_803), Some(19_810)]),
            ),
            ("y", numbers(&[0.0, 3.0, 10.0])),
        ],
    );
    let at = centres(&drawn(&with(
        &plot_of(days, "day", "y", Coord::Fixed(1.0)),
        Mark::Points,
    )));
    for pair in at.windows(2) {
        let (across, up) = (pair[1].0 - pair[0].0, pair[0].1 - pair[1].1);
        assert!(
            across > 0.0 && (across - up).abs() < 1e-6,
            "{across} and {up}"
        );
    }
}

#[test]
fn a_panel_too_thin_is_refused_with_the_ratio_that_fills_the_page() {
    // A thousand units along `x` and one along `y`, a unit of each the same length: a
    // panel a thousand times as wide as it is high.
    let frame = pairs(&[0.0, 1000.0], &[0.0, 1.0]);
    let thin = with(
        &plot_of(frame.clone(), "x", "y", Coord::Fixed(1.0)),
        Mark::Points,
    );
    let Err(LayoutError::TooThin { panel, filling }) = layout(&thin, Size::OneColumn, &JOURNAL)
    else {
        panic!("a panel a thousand times as wide as it is high was drawn");
    };
    assert!(
        panel.1 < SMALLEST_PANEL && panel.0 > 100.0 * panel.1,
        "{panel:?}"
    );
    // At the ratio it names the panel is the room: that of the plot without fixed
    // coordinates.
    let room = drawn(&with(
        &plot_of(frame.clone(), "x", "y", Coord::Cartesian),
        Mark::Points,
    ));
    let filled = drawn(&with(
        &plot_of(frame.clone(), "x", "y", Coord::Fixed(filling)),
        Mark::Points,
    ));
    let ((_, wide, high, _), (_, width, height, _)) = (panel_of(&room), panel_of(&filled));
    assert!((wide - width).abs() < 1e-6 && (high - height).abs() < 1e-6);
    // The other way: a panel too narrow.
    let narrow = with(&plot_of(frame, "x", "y", Coord::Fixed(1e6)), Mark::Points);
    assert!(matches!(
        layout(&narrow, Size::OneColumn, &JOURNAL),
        Err(LayoutError::TooThin { panel, .. }) if panel.0 < SMALLEST_PANEL
    ));
    // A page too small for any panel is still said to be too small.
    assert!(matches!(
        layout(&thin, Size::Millimetres(12.0, 12.0), &JOURNAL),
        Err(LayoutError::TooSmall { .. })
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows and the ratio: where the figure is drawn, a unit of `y` is as long
    /// as `ratio` units of `x`, the panel is no larger than its room either way and as
    /// large as it one way; and where it is not drawn, the panel was too thin.
    #[test]
    fn a_unit_of_y_is_as_long_as_the_ratio_says(rows in rows(), ratio in 0.05..20.0f64) {
        let frame = tried(&rows);
        let plain = with(&plot_of(frame.clone(), "x", "y", Coord::Cartesian), Mark::Points);
        prop_assume!(!plain.panels[0].layers[0].x.is_empty());
        let Ok(room) = layout(&plain, Size::OneColumn, &JOURNAL) else {
            return Ok(());
        };
        let fixed = with(&plot_of(frame, "x", "y", Coord::Fixed(ratio)), Mark::Points);
        match layout(&fixed, Size::OneColumn, &JOURNAL) {
            Ok(drawing) => {
                let (x, y) = lengths_of_a_unit(&fixed, &drawing);
                prop_assert!((y / x / ratio - 1.0).abs() < 1e-9, "{} against {}", y / x, ratio);
                let ((_, wide, high, _), (_, width, height, _)) =
                    (panel_of(&room), panel_of(&drawing));
                prop_assert!(width <= wide + 1e-9 && height <= high + 1e-9);
                prop_assert!((width - wide).abs() < 1e-9 || (height - high).abs() < 1e-9);
            }
            Err(LayoutError::TooThin { panel, .. }) => {
                prop_assert!(panel.0 < SMALLEST_PANEL || panel.1 < SMALLEST_PANEL);
            }
            Err(other) => prop_assert!(false, "{:?}", other),
        }
    }
}
