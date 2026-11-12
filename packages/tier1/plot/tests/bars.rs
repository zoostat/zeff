//! `bars` in the scene and on the page (Language Reference, section 7.6, "Marks"; release
//! 0.7.0, step 24b): a bar from zero at each value of `x`, as high as `y` or as the count of
//! rows; the bars that share an `x` stacked, side by side or as shares; on an axis of
//! categories or of numbers; the layout draws each bar as a rectangle.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Arrangement, Frame, Mark, PlotLayer};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{BAR_WIDTH, Paint, Scene, start, with_layer};

fn bars(how: Arrangement) -> PlotLayer {
    PlotLayer {
        mark: Mark::Bars(how),
        width: Some(BAR_WIDTH),
        ..points(1.0)
    }
}

fn texts(values: &[&str]) -> Column {
    Column::strs(values.iter().map(|v| Some(*v)))
}

/// Five sales: the shop and the kind as text, and the units sold.
fn shops() -> Frame {
    frame(
        "d",
        vec![
            ("shop", texts(&["north", "south", "east", "north", "south"])),
            ("units", numbers(&[12.0, 18.0, 15.0, 7.0, 9.0])),
            ("kind", texts(&["s", "s", "s", "l", "l"])),
        ],
    )
}

/// The bars of the first layer as ggplot2 lists them: the left and right edge and the lower
/// and upper end of each, in the order of the scene.
fn edges(scene: &Scene) -> Vec<(f64, f64, f64, f64)> {
    let layer = &scene.panels[0].layers[0];
    (0..layer.x.len())
        .map(|i| {
            (
                layer.x[i] - layer.span[i] / 2.0,
                layer.x[i] + layer.span[i] / 2.0,
                layer.low[i],
                layer.y[i],
            )
        })
        .collect()
}

fn close(ours: &[(f64, f64, f64, f64)], theirs: &[(f64, f64, f64, f64)]) {
    assert_eq!(ours.len(), theirs.len(), "{ours:?}");
    for (a, b) in ours.iter().zip(theirs) {
        let near = |x: f64, y: f64| (x - y).abs() < 1e-12;
        assert!(
            near(a.0, b.0) && near(a.1, b.1) && near(a.2, b.2) && near(a.3, b.3),
            "{a:?} is not {b:?} in {ours:?}"
        );
    }
}

fn limits(scene: &Scene, axis: &str, theirs: (f64, f64)) {
    let ours = scene.scale(axis).unwrap().limits;
    assert!(
        (ours.0 - theirs.0).abs() < 1e-12 && (ours.1 - theirs.1).abs() < 1e-12,
        "{axis}: {ours:?} is not {theirs:?}"
    );
}

fn by_kind(how: Arrangement) -> Scene {
    let plot = start(
        shops(),
        vec![
            mapped(Aesthetic::X, "shop"),
            mapped(Aesthetic::Y, "units"),
            mapped(Aesthetic::Fill, "kind"),
        ],
        Vec::new(),
    );
    with_layer(&plot, bars(how)).1
}

#[test]
fn bars_count_the_rows_at_each_value_of_x_as_ggplot2_counts_them() {
    // The reference is `geom_bar` of ggplot2 4.0.3 on the same five rows: a bar for each
    // shop, 0.9 of a place wide, as high as its count, and the limits 0.4 to 3.6 and -0.1 to
    // 2.1.
    let plot = start(shops(), vec![mapped(Aesthetic::X, "shop")], Vec::new());
    let (_, scene) = with_layer(&plot, bars(Arrangement::Stack));
    close(
        &edges(&scene),
        &[
            (0.55, 1.45, 0.0, 1.0),
            (1.55, 2.45, 0.0, 2.0),
            (2.55, 3.45, 0.0, 2.0),
        ],
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Bars(Arrangement::Stack));
    // A bar that counts is drawn from no one row.
    assert!(layer.rows.is_empty());
    assert_eq!(layer.fill, Paint::Theme);
    assert_eq!(scene.scale("x").unwrap().limits, (0.4, 3.6));
    // The axis of the counts is the plot's `y` axis, titled `count`.
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "count");
    limits(&scene, "y", (-0.1, 2.1));
    assert!(scene.notes.is_empty());

    // On numbers a bar is 0.9 of the smallest gap wide, and the axis holds the whole of the
    // first and the last: 5.46 to 19.54 in ggplot2.
    let plot = start(shops(), vec![mapped(Aesthetic::X, "units")], Vec::new());
    let (_, scene) = with_layer(&plot, bars(Arrangement::Stack));
    close(
        &edges(&scene),
        &[
            (6.1, 7.9, 0.0, 1.0),
            (8.1, 9.9, 0.0, 1.0),
            (11.1, 12.9, 0.0, 1.0),
            (14.1, 15.9, 0.0, 1.0),
            (17.1, 18.9, 0.0, 1.0),
        ],
    );
    limits(&scene, "x", (5.46, 19.54));
    limits(&scene, "y", (-0.05, 1.05));
}

#[test]
fn bars_of_values_are_stacked_set_side_by_side_or_shown_as_shares_as_in_ggplot2() {
    // The reference is `geom_col` of ggplot2 4.0.3 with `fill = kind`, in its three
    // positions: `xmin`, `xmax`, `ymin` and `ymax` of each bar, and the limits of `y`.
    let stacked = by_kind(Arrangement::Stack);
    close(
        &edges(&stacked),
        &[
            (0.55, 1.45, 0.0, 15.0),
            // At north and at south the first group, `l`, is on top.
            (1.55, 2.45, 12.0, 19.0),
            (1.55, 2.45, 0.0, 12.0),
            (2.55, 3.45, 18.0, 27.0),
            (2.55, 3.45, 0.0, 18.0),
        ],
    );
    limits(&stacked, "y", (-1.35, 28.35));
    let layer = &stacked.panels[0].layers[0];
    // Each bar is its row, and has the fill of its category.
    assert_eq!(layer.rows, [3, 4, 1, 5, 2]);
    assert_eq!(layer.fill, Paint::Scaled(vec![2.0, 1.0, 2.0, 1.0, 2.0]));
    assert_eq!(stacked.legends[0].marks, [Mark::Bars(Arrangement::Stack)]);

    let dodged = by_kind(Arrangement::Dodge);
    close(
        &edges(&dodged),
        &[
            // One bar at east has the whole width; two share it, the first at the left.
            (0.55, 1.45, 0.0, 15.0),
            (1.55, 2.0, 0.0, 7.0),
            (2.0, 2.45, 0.0, 12.0),
            (2.55, 3.0, 0.0, 9.0),
            (3.0, 3.45, 0.0, 18.0),
        ],
    );
    let centres = &dodged.panels[0].layers[0].x;
    for (ours, theirs) in centres.iter().zip([1.0, 1.775, 2.225, 2.775, 3.225]) {
        assert!((ours - theirs).abs() < 1e-12, "{ours} is not {theirs}");
    }
    limits(&dodged, "y", (-0.9, 18.9));

    let shares = by_kind(Arrangement::Share);
    close(
        &edges(&shares),
        &[
            (0.55, 1.45, 0.0, 1.0),
            (1.55, 2.45, 12.0 / 19.0, 1.0),
            (1.55, 2.45, 0.0, 12.0 / 19.0),
            (2.55, 3.45, 18.0 / 27.0, 1.0),
            (2.55, 3.45, 0.0, 18.0 / 27.0),
        ],
    );
    limits(&shares, "y", (-0.05, 1.05));
    let json = shares.json().write();
    assert!(json.contains("\"mark\": \"bars\""), "{json}");
    assert!(json.contains("\"position\": \"fill\""), "{json}");
    assert!(
        json.contains("\"span\": [0.9, 0.9, 0.9, 0.9, 0.9]"),
        "{json}"
    );
}

#[test]
fn values_below_zero_hang_down_and_shares_need_amounts() {
    // The reference is `geom_col` of ggplot2 4.0.3 on four rows of two groups.
    let d = frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 1.0, 2.0, 2.0])),
            ("y", numbers(&[2.0, -1.0, -3.0, 4.0])),
            ("g", texts(&["a", "b", "a", "b"])),
        ],
    );
    let plot = start(
        d,
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Fill, "g"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, bars(Arrangement::Stack));
    close(
        &edges(&scene),
        &[
            (0.55, 1.45, 0.0, 2.0),
            (0.55, 1.45, 0.0, -1.0),
            (1.55, 2.45, 0.0, -3.0),
            (1.55, 2.45, 0.0, 4.0),
        ],
    );
    limits(&scene, "x", (0.455, 2.545));
    limits(&scene, "y", (-3.35, 4.35));
    // As shares, a place with a value below zero has nothing to show.
    let (_, scene) = with_layer(&plot, bars(Arrangement::Share));
    assert!(scene.panels[0].layers[0].x.is_empty());
    assert_eq!(
        scene.notes,
        ["4 bars at 2 values of x left out: shares need values of 0 or more and a total above 0"]
    );
}

#[test]
fn a_bar_is_a_rectangle_on_the_page_and_its_key_a_patch() {
    let scene = by_kind(Arrangement::Dodge);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (corner, width, height, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 5);
    let (x, y) = (
        scene.scale("x").unwrap().limits,
        scene.scale("y").unwrap().limits,
    );
    let across = |v: f64| corner.0 + (v - x.0) / (x.1 - x.0) * width;
    let down = |v: f64| corner.1 + height - (v - y.0) / (y.1 - y.0) * height;
    // The second bar: `l` at north, from 1.55 to 2 and from zero to 7, in the first colour.
    match &marks[1].primitive {
        Primitive::Polygon {
            rings,
            fill: Some(fill),
            stroke: None,
            opacity,
        } => {
            let ring = &rings[0];
            assert_eq!(ring.len(), 4);
            for (corner, theirs) in ring.iter().zip([
                (across(1.55), down(0.0)),
                (across(2.0), down(0.0)),
                (across(2.0), down(7.0)),
                (across(1.55), down(7.0)),
            ]) {
                assert!(
                    (corner.0 - theirs.0).abs() < 1e-9 && (corner.1 - theirs.1).abs() < 1e-9,
                    "{corner:?} is not {theirs:?}"
                );
            }
            assert_eq!((fill.as_str(), *opacity), ("#E69F00", 1.0));
        }
        other => panic!("{other:?} is not a bar"),
    }
    let patches = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:fill"))
        .filter(|item| matches!(item.primitive, Primitive::Polygon { .. }))
        .count();
    assert_eq!(patches, 2);
    insta::assert_snapshot!("bars_side_by_side", drawing.describe());
}

#[test]
fn rows_without_a_place_or_a_height_are_counted_and_an_empty_category_keeps_its_place() {
    let d = frame(
        "d",
        vec![
            (
                "shop",
                Column::strs([Some("north"), None, Some("south"), Some("north")]),
            ),
            (
                "units",
                floats(&[Some(1.0), Some(2.0), None, Some(f64::INFINITY)]),
            ),
        ],
    );
    // Values: a row needs its shop and a finite number.
    let plot = start(
        d.clone(),
        vec![mapped(Aesthetic::X, "shop"), mapped(Aesthetic::Y, "units")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, bars(Arrangement::Stack));
    assert_eq!(scene.panels[0].layers[0].rows, [1]);
    assert_eq!(
        scene.notes,
        [
            "2 rows with NA in shop or units dropped",
            "1 row with an infinite value in shop or units dropped"
        ]
    );
    // South has no bar, and keeps its place on the axis.
    assert_eq!(scene.scale("x").unwrap().labels, ["north", "south"]);
    close(&edges(&scene), &[(0.55, 1.45, 0.0, 1.0)]);
    // Counts: a row needs its shop alone.
    let plot = start(d, vec![mapped(Aesthetic::X, "shop")], Vec::new());
    let (_, scene) = with_layer(&plot, bars(Arrangement::Stack));
    close(
        &edges(&scene),
        &[(0.55, 1.45, 0.0, 2.0), (1.55, 2.45, 0.0, 1.0)],
    );
    assert_eq!(scene.notes, ["1 row with NA in shop dropped"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn stacked_heights_sum_to_the_totals_and_counts_to_the_rows(
        table in prop::collection::vec((0u8..4, 0u8..3, 0.0..50.0f64), 0..40),
        which in 0u8..3,
        counting in any::<bool>(),
    ) {
        let how = [Arrangement::Stack, Arrangement::Dodge, Arrangement::Share][which as usize];
        // The place as a number, the group as the category `g`, the height as `y`.
        let rows: Vec<Row> = table
            .iter()
            .map(|(x, g, y)| (Some(f64::from(*x)), Some(*y), Some(*g), None))
            .collect();
        let mut mappings = vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Fill, "g")];
        if !counting {
            mappings.push(mapped(Aesthetic::Y, "y"));
        }
        let plot = start(tried(&rows), mappings, Vec::new());
        let (_, scene) = with_layer(&plot, bars(how));
        let layer = &scene.panels[0].layers[0];
        prop_assert_eq!(layer.x.len(), layer.low.len());
        prop_assert_eq!(layer.x.len(), layer.span.len());
        // The smallest gap between two places, which a bar is 0.9 wide of.
        let mut places: Vec<u8> = table.iter().map(|r| r.0).collect();
        places.sort_unstable();
        places.dedup();
        let gap = places
            .windows(2)
            .map(|pair| f64::from(pair[1] - pair[0]))
            .fold(f64::INFINITY, f64::min);
        let gap = if gap.is_finite() { gap } else { 1.0 };
        for place in 0u8..4 {
            let here: Vec<&(u8, u8, f64)> = table.iter().filter(|r| r.0 == place).collect();
            // A place without a row has no bar; the bars near it are its neighbours'.
            if here.is_empty() {
                continue;
            }
            let total: f64 = if counting {
                here.len() as f64
            } else {
                here.iter().map(|r| r.2).sum()
            };
            // The bars of this place: every bar within half a gap of it.
            let bars: Vec<usize> = (0..layer.x.len())
                .filter(|i| (layer.x[*i] - f64::from(place)).abs() < gap / 2.0)
                .collect();
            if how == Arrangement::Share && total <= 0.0 {
                prop_assert!(bars.is_empty());
                continue;
            }
            let heights: f64 = bars.iter().map(|i| layer.y[*i] - layer.low[*i]).sum();
            let top = bars.iter().map(|i| layer.y[*i]).fold(0.0, f64::max);
            let slack = 1e-9 * total.max(1.0);
            match how {
                Arrangement::Stack => {
                    prop_assert!((heights - total).abs() <= slack);
                    prop_assert!((top - total).abs() <= slack);
                }
                Arrangement::Share => {
                    prop_assert!((heights - 1.0).abs() <= 1e-9);
                    prop_assert!((top - 1.0).abs() <= 1e-9);
                }
                Arrangement::Dodge => {
                    prop_assert!((heights - total).abs() <= slack);
                    // Side by side, the groups of a place share the width of one bar, which
                    // is 0.9 of the smallest gap between two places.
                    if !bars.is_empty() {
                        let wide = BAR_WIDTH * gap;
                        let groups: std::collections::BTreeSet<u8> =
                            here.iter().map(|r| r.1).collect();
                        for i in &bars {
                            prop_assert_eq!(layer.low[*i], 0.0);
                            prop_assert!(
                                (layer.span[*i] * groups.len() as f64 - wide).abs() < 1e-12,
                                "span {} of {:?}, {} groups, wide {}, at {:?}",
                                layer.span[*i],
                                layer.span,
                                groups.len(),
                                wide,
                                layer.x
                            );
                        }
                    }
                }
            }
        }
        // Every bar starts at zero or on another, and the axis holds zero and every bar.
        if !layer.x.is_empty() {
            let (lo, hi) = scene.scale("y").unwrap().limits;
            prop_assert!(lo <= 0.0 && hi >= 0.0);
            for value in layer.y.iter().chain(&layer.low) {
                prop_assert!(lo <= *value && *value <= hi);
            }
        }
        prop_assert!(layout(&scene, Size::OneColumn, &JOURNAL).is_ok());
    }
}
