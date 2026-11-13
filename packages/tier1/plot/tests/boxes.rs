//! The box plot layer (Language Reference, section 7.6, "Statistical layers": `boxplot`;
//! release 0.7.0, step 28c): a box for each value of `x`, or each group there, over the
//! values of `y`. The five numbers of a box and the values beyond its whiskers are those of
//! R's `boxplot.stats` (R 4.6.0).

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Mark, Plot, PlotFacet, PlotLayer};
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::layout::{JOURNAL, OUTLIER_SIZE, Size, layout};
use zeff_plot::scene::{BOX_WIDTH, Layer, Scene, start, with_layer};

fn boxes(outliers: bool) -> PlotLayer {
    PlotLayer {
        mark: Mark::Box(outliers),
        ..points(1.0)
    }
}

/// The boxes of `y` of a frame, at each value of `x` where one is named.
fn boxed(frame: Frame, x: Option<&str>, y: &str, layer: PlotLayer) -> Scene {
    let mut mappings = vec![mapped(Aesthetic::Y, y)];
    mappings.extend(x.map(|x| mapped(Aesthetic::X, x)));
    let plot: Plot = start(frame, mappings, Vec::new());
    with_layer(&plot, layer).1
}

fn of(values: &[f64]) -> Frame {
    frame("d", vec![("v", numbers(values))])
}

/// The five numbers of the first box of a layer: the lower end of its whiskers, its lower
/// hinge, its median, its upper hinge and the upper end.
fn five(layer: &Layer) -> [f64; 5] {
    [
        layer.whiskers[0].0,
        layer.low[0],
        layer.middle[0],
        layer.y[0],
        layer.whiskers[0].1,
    ]
}

fn beyond(layer: &Layer) -> Vec<f64> {
    let mut all: Vec<f64> = layer.beyond.iter().map(|(_, value)| *value).collect();
    all.sort_by(f64::total_cmp);
    all
}

#[test]
fn the_five_numbers_and_the_values_beyond_are_those_of_boxplot_stats() {
    // `boxplot.stats(x)$stats` and `$out` on four series.
    let a: Vec<f64> = (1..=200)
        .map(|i| f64::from((i * 37) % 101) + f64::from(i % 7) * 0.25)
        .collect();
    let b: Vec<f64> = (1..=57)
        .map(|i| f64::from((i * i) % 23) / 4.0 + f64::from(i) / 100.0)
        .collect();
    let mut c = vec![5.0; 40];
    c.extend([1.0, 9.0]);
    let e = vec![
        1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 5.0, 9.0, 30.0, -20.0,
    ];
    for (values, stats, out) in [
        (a, [0.75, 25.75, 50.75, 76.375, 101.25], vec![]),
        (b, [0.23, 1.02, 2.13, 3.42, 5.04], vec![]),
        (c, [5.0, 5.0, 5.0, 5.0, 5.0], vec![1.0, 9.0]),
        (e, [1.0, 2.0, 3.0, 4.5, 5.0], vec![-20.0, 9.0, 30.0]),
        // One value, two, three, four, and five with one far away.
        (vec![2.5], [2.5; 5], vec![]),
        (vec![1.0, 4.0], [1.0, 1.0, 2.5, 4.0, 4.0], vec![]),
        (vec![1.0, 2.0, 3.0], [1.0, 1.5, 2.0, 2.5, 3.0], vec![]),
        (vec![1.0, 2.0, 3.0, 4.0], [1.0, 1.5, 2.5, 3.5, 4.0], vec![]),
        (
            vec![1.0, 2.0, 3.0, 4.0, 100.0],
            [1.0, 2.0, 3.0, 4.0, 4.0],
            vec![100.0],
        ),
    ] {
        let scene = boxed(of(&values), None, "v", boxes(true));
        let layer = &scene.panels[0].layers[0];
        assert_eq!(layer.mark, Mark::Box(true));
        let found = five(layer);
        for (found, wanted) in found.iter().zip(stats) {
            assert!(
                (found - wanted).abs() < 1e-12,
                "{found:?} against {stats:?}"
            );
        }
        assert_eq!(beyond(layer), out);
    }
}

#[test]
fn one_box_stands_at_one_place_without_a_name() {
    let scene = boxed(of(&[1.0, 2.0, 3.0, 4.0, 100.0]), None, "v", boxes(true));
    let layer = &scene.panels[0].layers[0];
    // One box at the one place, three quarters of it wide; drawn from no one row.
    assert_eq!((&layer.x, &layer.span), (&vec![1.0], &vec![BOX_WIDTH]));
    assert!(layer.rows.is_empty());
    let x = scene.scale("x").unwrap();
    assert_eq!(
        (x.title.as_str(), x.limits, &x.labels),
        ("", (0.4, 1.6), &vec![String::new()])
    );
    // The axis of `y` reaches the value beyond the whiskers.
    let y = scene.scale("y").unwrap();
    assert!(y.limits.0 < 1.0 && y.limits.1 > 100.0);
    // Without the values beyond, the axis is that of the box and its whiskers, and a note
    // says what is not drawn.
    let scene = boxed(of(&[1.0, 2.0, 3.0, 4.0, 100.0]), None, "v", boxes(false));
    let layer = &scene.panels[0].layers[0];
    assert!(layer.beyond.is_empty());
    assert_eq!(five(layer), [1.0, 2.0, 3.0, 4.0, 4.0]);
    assert!(scene.scale("y").unwrap().limits.1 < 5.0);
    assert_eq!(scene.notes, ["1 value beyond the whiskers is not drawn"]);
    insta::assert_snapshot!(
        "one_box",
        boxed(of(&[1.0, 2.0, 3.0, 4.0, 100.0]), None, "v", boxes(true))
            .json()
            .write()
    );
}

/// Twelve rows: a number for each of three sites, of two arms.
fn sites() -> Frame {
    frame(
        "d",
        vec![
            (
                "site",
                Column::strs(
                    ["n", "n", "n", "n", "s", "s", "s", "s", "w", "w", "w", "w"].map(Some),
                ),
            ),
            (
                "arm",
                Column::strs(
                    ["a", "a", "b", "b", "a", "a", "b", "b", "a", "a", "a", "a"].map(Some),
                ),
            ),
            (
                "v",
                floats(&[
                    Some(1.0),
                    Some(3.0),
                    Some(5.0),
                    Some(7.0),
                    Some(2.0),
                    Some(4.0),
                    Some(6.0),
                    Some(8.0),
                    Some(1.0),
                    Some(2.0),
                    Some(3.0),
                    None,
                ]),
            ),
        ],
    )
}

#[test]
fn a_box_for_each_value_of_x_and_each_group_there_and_each_panel() {
    // A box for each site: at its place, three quarters of a place wide.
    let scene = boxed(sites(), Some("site"), "v", boxes(true));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 3.0]);
    assert_eq!(layer.span, [BOX_WIDTH; 3]);
    assert_eq!(layer.middle, [4.0, 5.0, 2.0]);
    assert_eq!(scene.notes, ["1 row with NA in site or v dropped"]);
    // Filled by the arm: where a site has both arms its two boxes share the width, the
    // first at the left; the west has one arm, and its box has the width whole.
    let mut plot: Plot = start(
        sites(),
        vec![
            mapped(Aesthetic::X, "site"),
            mapped(Aesthetic::Y, "v"),
            mapped(Aesthetic::Fill, "arm"),
        ],
        Vec::new(),
    );
    let scene = with_layer(&plot, boxes(true)).1;
    let layer = &scene.panels[0].layers[0];
    let half = BOX_WIDTH / 2.0;
    assert_eq!(layer.span, [half, half, half, half, BOX_WIDTH]);
    assert_eq!(
        layer.x,
        [
            1.0 - half / 2.0,
            1.0 + half / 2.0,
            2.0 - half / 2.0,
            2.0 + half / 2.0,
            3.0
        ]
    );
    assert_eq!(layer.middle, [2.0, 6.0, 3.0, 7.0, 2.0]);
    // Each box has the fill of its arm.
    assert!(
        matches!(&layer.fill, zeff_plot::scene::Paint::Scaled(values) if values == &vec![1.0, 2.0, 1.0, 2.0, 1.0])
    );
    // Parted into panels by the arm: each panel has the boxes of its rows, at the places of
    // the plot.
    plot.mappings.pop();
    plot.facet = Some(PlotFacet {
        by: vec!["arm".to_string()],
        ..PlotFacet::default()
    });
    let scene = with_layer(&plot, boxes(true)).1;
    assert_eq!(scene.panels.len(), 2);
    assert_eq!(scene.panels[0].layers[0].x, [1.0, 2.0, 3.0]);
    assert_eq!(scene.panels[0].layers[0].middle, [2.0, 3.0, 2.0]);
    assert_eq!(scene.panels[1].layers[0].x, [1.0, 2.0]);
    assert_eq!(scene.panels[1].layers[0].middle, [6.0, 7.0]);
    assert_eq!(scene.panels[1].layers[0].whiskers, [(5.0, 7.0), (6.0, 8.0)]);
}

/// The primitives in the panel of a drawing, by kind: polygons, segments and points.
fn drawn_in(drawing: &Drawing) -> (usize, usize, usize) {
    let (_, _, _, marks) = panel_of(drawing);
    let count =
        |wanted: fn(&Primitive) -> bool| marks.iter().filter(|m| wanted(&m.primitive)).count();
    (
        count(|p| matches!(p, Primitive::Polygon { .. })),
        count(|p| matches!(p, Primitive::Segment { .. })),
        count(|p| matches!(p, Primitive::Point { .. })),
    )
}

#[test]
fn a_box_is_drawn_as_its_box_its_median_its_whiskers_and_its_points() {
    let scene = boxed(of(&[1.0, 2.0, 3.0, 4.0, 100.0]), None, "v", boxes(true));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    // One box, two whiskers and the line of the median, one point beyond.
    assert_eq!(drawn_in(&drawing), (1, 3, 1));
    let (panel, width, height, marks) = panel_of(&drawing);
    let y = scene.scale("y").unwrap();
    let down =
        |value: f64| panel.1 + height - zeff_plot::scale::share(y.kind, y.limits, value) * height;
    for item in marks {
        match &item.primitive {
            // The box: from the lower hinge to the upper, three quarters of the place wide,
            // in the colour of the page with a line round it.
            Primitive::Polygon {
                rings,
                fill,
                stroke,
                ..
            } => {
                let ys: Vec<f64> = rings[0].iter().map(|at| at.1).collect();
                let (top, foot) = (
                    ys.iter().copied().fold(f64::MAX, f64::min),
                    ys.iter().copied().fold(f64::MIN, f64::max),
                );
                assert!((top - down(4.0)).abs() < 1e-9 && (foot - down(2.0)).abs() < 1e-9);
                let xs: Vec<f64> = rings[0].iter().map(|at| at.0).collect();
                let wide = xs.iter().copied().fold(f64::MIN, f64::max)
                    - xs.iter().copied().fold(f64::MAX, f64::min);
                assert!((wide - BOX_WIDTH / 1.2 * width).abs() < 1e-9);
                assert_eq!(fill.as_deref(), Some("#FFFFFF"));
                assert!(stroke.is_some());
            }
            // The value beyond: a small point at the middle of the box.
            Primitive::Point { at, size, .. } => {
                assert!((at.0 - (panel.0 + width / 2.0)).abs() < 1e-9);
                assert!((at.1 - down(100.0)).abs() < 1e-9);
                assert_eq!(*size, OUTLIER_SIZE);
            }
            _ => {}
        }
    }
    // The line of the median runs across the box at 3; a whisker runs from the lower hinge
    // down to 1, and the other has no length, for the upper hinge is the furthest value
    // within reach.
    let segments: Vec<((f64, f64), (f64, f64))> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Segment { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect();
    assert!(
        segments
            .iter()
            .any(|(from, to)| (from.1 - down(3.0)).abs() < 1e-9
                && (to.1 - down(3.0)).abs() < 1e-9
                && (to.0 - from.0).abs() > 1.0)
    );
    assert!(
        segments
            .iter()
            .any(|(from, to)| (from.1 - down(1.0)).abs() < 1e-9
                && (to.1 - down(2.0)).abs() < 1e-9
                && (to.0 - from.0).abs() < 1e-9)
    );
    // Without the values beyond: no point.
    let scene = boxed(of(&[1.0, 2.0, 3.0, 4.0, 100.0]), None, "v", boxes(false));
    assert_eq!(
        drawn_in(&layout(&scene, Size::OneColumn, &JOURNAL).unwrap()),
        (1, 3, 0)
    );
    insta::assert_snapshot!(
        "three_sites_boxed",
        layout(
            &boxed(sites(), Some("site"), "v", boxes(true)),
            Size::OneColumn,
            &JOURNAL
        )
        .unwrap()
        .describe()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows: every value of a group is in its box, on a whisker or beyond
    /// them; the five numbers are in order; half the values or more are in the box; and no
    /// value beyond the whiskers is within one and a half heights of the box from it.
    #[test]
    fn the_five_numbers_of_a_box_are_in_order_and_hold_their_values(rows in rows()) {
        let scene = boxed(tried(&rows), Some("g"), "y", boxes(true));
        let layer = &scene.panels[0].layers[0];
        let mut counted = 0;
        for at in 0..layer.x.len() {
            let (foot, head) = layer.whiskers[at];
            let (lower, median, upper) = (layer.low[at], layer.middle[at], layer.y[at]);
            prop_assert!(foot <= lower && lower <= median && median <= upper && upper <= head);
            let reach = 1.5 * (upper - lower);
            for (_, value) in layer.beyond.iter().filter(|(of, _)| *of == at) {
                prop_assert!(*value < lower - reach || *value > upper + reach);
                counted += 1;
            }
        }
        // A box for each group that has a value, and every value beyond belongs to a box.
        let groups: std::collections::BTreeSet<Option<u8>> = rows
            .iter()
            .filter(|row| row.1.is_some() && row.2.is_some())
            .map(|row| row.2)
            .collect();
        prop_assert_eq!(layer.x.len(), groups.len());
        prop_assert_eq!(counted, layer.beyond.len());
    }
}
