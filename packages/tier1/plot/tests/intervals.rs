//! The layer of intervals (Language Reference, section 7.6, "Statistical layers":
//! `errorbars`; release 0.7.0, step 28d): at each row's `x` an upright line from `low` to
//! `high`, with a cap across each end; side by side where the bars or the boxes of the plot
//! stand side by side; and what the interval is, recorded in the scene (decision #118, F11).

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Arrangement, Frame, Mark, Plot, PlotFacet, PlotLayer};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{BAR_WIDTH, BOX_WIDTH, CAP_WIDTH, GIVEN, Scene, start, with_layer};

/// A layer of intervals from `low` to `high`, with caps of `width`.
fn intervals(low: &str, high: &str, width: f64) -> PlotLayer {
    PlotLayer {
        mark: Mark::Errorbars,
        width: Some(width),
        mappings: vec![mapped(Aesthetic::Low, low), mapped(Aesthetic::High, high)],
        ..points(1.0)
    }
}

/// The intervals of a frame at each value of `x`.
fn drawn_at(frame: Frame, x: &str, layer: PlotLayer) -> Scene {
    let plot: Plot = start(frame, vec![mapped(Aesthetic::X, x)], Vec::new());
    with_layer(&plot, layer).1
}

/// Four doses with an interval each.
fn doses() -> Frame {
    frame(
        "d",
        vec![
            ("dose", numbers(&[10.0, 20.0, 30.0, 50.0])),
            ("lower", numbers(&[1.0, 2.5, 2.0, 4.0])),
            ("upper", numbers(&[3.0, 3.5, 6.0, 4.5])),
        ],
    )
}

#[test]
fn an_interval_stands_at_each_row_from_low_to_high() {
    let scene = drawn_at(doses(), "dose", intervals("lower", "upper", CAP_WIDTH));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Errorbars);
    // Each interval at its dose, from its lower end to its upper one, drawn from its row.
    assert_eq!(layer.x, [10.0, 20.0, 30.0, 50.0]);
    assert_eq!(layer.low, [1.0, 2.5, 2.0, 4.0]);
    assert_eq!(layer.y, [3.0, 3.5, 6.0, 4.5]);
    assert_eq!(layer.rows, [1, 2, 3, 4]);
    // A cap is three tenths of the smallest gap between two doses wide.
    assert_eq!(layer.span, [CAP_WIDTH * 10.0; 4]);
    assert!(scene.notes.is_empty());
    // The axis of `y` is named by the lower end and reaches every end; that of `x` reaches
    // the caps.
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "lower");
    assert!(y.limits.0 < 1.0 && y.limits.1 > 6.0);
    let x = scene.scale("x").unwrap();
    assert!(x.limits.0 < 8.5 && x.limits.1 > 51.5);
    // The scene says what the intervals are: the two ends the script gives.
    let json = scene.json().write();
    assert!(
        json.contains(&format!("\"interval\": \"{GIVEN}\""))
            || json.contains(&format!("\"interval\":\"{GIVEN}\""))
    );
    insta::assert_snapshot!("four_doses", json);
    // Without caps: a width of nothing.
    let scene = drawn_at(doses(), "dose", intervals("lower", "upper", 0.0));
    assert_eq!(scene.panels[0].layers[0].span, [0.0; 4]);
}

#[test]
fn an_end_above_the_other_no_length_and_an_end_that_is_not_there() {
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
            (
                "lower",
                floats(&[
                    Some(5.0),
                    Some(2.0),
                    None,
                    Some(1.0),
                    Some(f64::NEG_INFINITY),
                    Some(7.0),
                ]),
            ),
            (
                "upper",
                floats(&[Some(3.0), Some(2.0), Some(4.0), None, Some(2.0), Some(6.0)]),
            ),
        ],
    );
    let scene = drawn_at(frame, "x", intervals("lower", "upper", CAP_WIDTH));
    let layer = &scene.panels[0].layers[0];
    // The first and the last have their lower end above their upper one: they are drawn
    // between the two. The second has no length. The third and the fourth lack an end, and
    // the fifth has an infinite one: they are not drawn.
    assert_eq!(layer.x, [1.0, 2.0, 6.0]);
    assert_eq!(layer.low, [3.0, 2.0, 6.0]);
    assert_eq!(layer.y, [5.0, 2.0, 7.0]);
    assert_eq!(layer.rows, [1, 2, 6]);
    assert_eq!(
        scene.notes,
        [
            "2 rows with NA in x, lower or upper dropped",
            "1 row with an infinite value in x, lower or upper dropped",
            "2 intervals have lower above upper: they are drawn between the two",
        ]
    );
    // One alone is told as one.
    let one = common::frame(
        "d",
        vec![
            ("x", numbers(&[1.0])),
            ("lower", numbers(&[2.0])),
            ("upper", numbers(&[1.0])),
        ],
    );
    let scene = drawn_at(one, "x", intervals("lower", "upper", CAP_WIDTH));
    assert_eq!(
        scene.notes,
        ["1 interval has lower above upper: it is drawn between the two"]
    );
    // One place alone has a gap of one.
    assert_eq!(scene.panels[0].layers[0].span, [CAP_WIDTH]);
}

/// Two arms at each of two sites, and a third site with one arm: a mean and an interval.
fn sites() -> Frame {
    frame(
        "d",
        vec![
            ("site", Column::strs(["n", "n", "s", "s", "w"].map(Some))),
            ("arm", Column::strs(["a", "b", "a", "b", "a"].map(Some))),
            ("mean", numbers(&[2.0, 3.0, 4.0, 2.5, 1.0])),
            ("lower", numbers(&[1.5, 2.2, 3.1, 1.9, 0.5])),
            ("upper", numbers(&[2.5, 3.8, 4.9, 3.1, 1.5])),
        ],
    )
}

fn bars(how: Arrangement) -> PlotLayer {
    PlotLayer {
        mark: Mark::Bars(how),
        width: Some(BAR_WIDTH),
        ..points(1.0)
    }
}

#[test]
fn intervals_stand_side_by_side_where_the_bars_or_the_boxes_do() {
    let mut plot: Plot = start(
        sites(),
        vec![
            mapped(Aesthetic::X, "site"),
            mapped(Aesthetic::Y, "mean"),
            mapped(Aesthetic::Fill, "arm"),
        ],
        Vec::new(),
    );
    // Over bars side by side: each interval at the middle of its bar, its cap as much
    // narrower as the bar is; the site with one arm has its interval at its place.
    let (with_bars, _) = with_layer(&plot, bars(Arrangement::Dodge));
    let scene = with_layer(&with_bars, intervals("lower", "upper", CAP_WIDTH)).1;
    let (under, over) = (&scene.panels[0].layers[0], &scene.panels[0].layers[1]);
    assert_eq!(over.x, under.x);
    // The places are those the bars are given: from the left edge of the bars of a site,
    // half a bar in for the first arm and a bar and a half for the second.
    let slot = BAR_WIDTH / 2.0;
    let left = |site: f64| site - BAR_WIDTH / 2.0;
    assert_eq!(
        over.x,
        [
            left(1.0) + 0.5 * slot,
            left(1.0) + 1.5 * slot,
            left(2.0) + 0.5 * slot,
            left(2.0) + 1.5 * slot,
            3.0
        ]
    );
    let half = CAP_WIDTH / 2.0;
    assert_eq!(over.span, [half, half, half, half, CAP_WIDTH]);
    assert_eq!(over.low, [1.5, 2.2, 3.1, 1.9, 0.5]);
    // The intervals are in the colour of the text, and read no fill.
    assert!(matches!(over.color, zeff_plot::scene::Paint::Theme));
    // The same where the plot colours by the arm and the bars take their fill from it.
    let coloured: Plot = start(
        sites(),
        vec![
            mapped(Aesthetic::X, "site"),
            mapped(Aesthetic::Y, "mean"),
            mapped(Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    let (with_bars, _) = with_layer(&coloured, bars(Arrangement::Dodge));
    let scene = with_layer(&with_bars, intervals("lower", "upper", CAP_WIDTH)).1;
    assert_eq!(scene.panels[0].layers[1].x, scene.panels[0].layers[0].x);
    // Over bars on one another, and over points, every interval stands at its place.
    for under in [bars(Arrangement::Stack), points(1.0)] {
        let (with_under, _) = with_layer(&plot, under);
        let scene = with_layer(&with_under, intervals("lower", "upper", CAP_WIDTH)).1;
        let over = &scene.panels[0].layers[1];
        assert_eq!(over.x, [1.0, 1.0, 2.0, 2.0, 3.0]);
        assert_eq!(over.span, [CAP_WIDTH; 5]);
    }
    // Beside boxes the intervals share the width of a box.
    let (with_boxes, _) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Box(true),
            ..points(1.0)
        },
    );
    let scene = with_layer(&with_boxes, intervals("lower", "upper", CAP_WIDTH)).1;
    assert_eq!(scene.panels[0].layers[1].x, scene.panels[0].layers[0].x);
    assert_eq!(scene.panels[0].layers[1].x[0], 1.0 - BOX_WIDTH / 4.0);
    // Without a fill nothing parts the rows, and nothing stands side by side.
    plot.mappings.pop();
    let (with_bars, _) = with_layer(&plot, bars(Arrangement::Dodge));
    let scene = with_layer(&with_bars, intervals("lower", "upper", CAP_WIDTH)).1;
    assert_eq!(scene.panels[0].layers[1].x, [1.0, 1.0, 2.0, 2.0, 3.0]);
    // Parted into panels by the arm, each panel has the intervals of its rows.
    plot.facet = Some(PlotFacet {
        by: vec!["arm".to_string()],
        ..PlotFacet::default()
    });
    let scene = with_layer(&plot, intervals("lower", "upper", CAP_WIDTH)).1;
    assert_eq!(scene.panels.len(), 2);
    assert_eq!(scene.panels[0].layers[0].x, [1.0, 2.0, 3.0]);
    assert_eq!(scene.panels[0].layers[0].low, [1.5, 3.1, 0.5]);
    assert_eq!(scene.panels[1].layers[0].x, [1.0, 2.0]);
    assert_eq!(scene.panels[1].layers[0].y, [3.8, 3.1]);
    assert_eq!(scene.panels[1].layers[0].span, [CAP_WIDTH; 2]);
}

/// The segments in the panel of a drawing.
fn segments(scene: &Scene) -> Vec<((f64, f64), (f64, f64))> {
    let drawing = layout(scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    assert!(
        marks
            .iter()
            .all(|item| !matches!(item.primitive, Primitive::Point { .. }))
    );
    marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Segment { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect()
}

#[test]
fn an_interval_is_drawn_as_an_upright_line_and_two_caps() {
    let scene = drawn_at(doses(), "dose", intervals("lower", "upper", CAP_WIDTH));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (panel, width, height, _) = panel_of(&drawing);
    let (x, y) = (scene.scale("x").unwrap(), scene.scale("y").unwrap());
    let across = |value: f64| panel.0 + zeff_plot::scale::share(x.kind, x.limits, value) * width;
    let down =
        |value: f64| panel.1 + height - zeff_plot::scale::share(y.kind, y.limits, value) * height;
    let drawn = segments(&scene);
    // Three segments for each interval, and no point.
    assert_eq!(drawn.len(), 12);
    let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
    // The first interval: upright at 10 from 1 to 3; a cap at each end, 3 wide.
    assert!(drawn.iter().any(|(from, to)| near(from.0, across(10.0))
        && near(to.0, across(10.0))
        && near(from.1, down(1.0))
        && near(to.1, down(3.0))));
    for end in [1.0, 3.0] {
        assert!(drawn.iter().any(|(from, to)| near(from.0, across(8.5))
            && near(to.0, across(11.5))
            && near(from.1, down(end))
            && near(to.1, down(end))));
    }
    // Without caps: the upright line alone.
    let scene = drawn_at(doses(), "dose", intervals("lower", "upper", 0.0));
    assert_eq!(segments(&scene).len(), 4);
    // Lying along the page under `coord(flip)`: the line of an interval is level.
    let mut plot: Plot = start(doses(), vec![mapped(Aesthetic::X, "dose")], Vec::new());
    plot.coord = zeff_core::value::Coord::Flip;
    let scene = with_layer(&plot, intervals("lower", "upper", 0.0)).1;
    let lying = segments(&scene);
    assert_eq!(lying.len(), 4);
    assert!(
        lying
            .iter()
            .all(|(from, to)| near(from.1, to.1) && !near(from.0, to.0))
    );
    insta::assert_snapshot!(
        "four_doses_drawn",
        layout(
            &drawn_at(doses(), "dose", intervals("lower", "upper", CAP_WIDTH)),
            Size::OneColumn,
            &JOURNAL
        )
        .unwrap()
        .describe()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows: an interval for each row with a finite `x`, `low` and `high`, and
    /// for no other; each from the smaller of its two ends to the larger, at the `x` of its
    /// row; the axis of `y` holding every end; and the note counting the ends exchanged.
    #[test]
    fn every_row_with_its_three_values_has_its_interval(rows in rows(), width in 0.0..=1.0f64) {
        let scene = drawn_at(tried(&rows), "x", intervals("y", "z", width));
        let layer = &scene.panels[0].layers[0];
        let whole: Vec<(usize, f64, f64, f64)> = rows
            .iter()
            .enumerate()
            .filter_map(|(at, row)| Some((at + 1, row.0?, row.1?, row.3?)))
            .collect();
        prop_assert_eq!(layer.x.len(), whole.len());
        prop_assert_eq!(layer.low.len(), whole.len());
        prop_assert_eq!(layer.span.len(), whole.len());
        for at in 0..layer.x.len() {
            let row = whole.iter().find(|row| row.0 == layer.rows[at]).unwrap();
            prop_assert_eq!(layer.x[at], row.1 + 0.0);
            prop_assert_eq!(layer.low[at], row.2.min(row.3));
            prop_assert_eq!(layer.y[at], row.2.max(row.3));
            prop_assert!(layer.span[at] >= 0.0);
        }
        if let Some(y) = scene.scale("y") {
            for at in 0..layer.x.len() {
                prop_assert!(y.limits.0 <= layer.low[at] && layer.y[at] <= y.limits.1);
            }
        }
        let turned = whole.iter().filter(|row| row.2 > row.3).count();
        let told = scene.notes.iter().any(|note| note.contains("above"));
        prop_assert_eq!(told, turned > 0);
        // And the figure can be laid out, or is refused for its size alone.
        let _ = layout(&scene, Size::OneColumn, &JOURNAL);
    }
}
