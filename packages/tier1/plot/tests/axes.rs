//! The kinds of scale of an axis of numbers (Language Reference, section 7.6, "Scales";
//! release 0.7.0, step 25a): linear, log and square root. The limits of a log and of a
//! square-root axis have their margin in the logarithm and in the square root; the breaks of
//! a log axis are powers of ten, or 1, 2 and 5 times one; the layout places a mark by the
//! kind of its axis; and a value with no place on the scale is found and named.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::{Aesthetic, AxisKind, Mark, Plot, PlotAxis, PlotLayer, Stacking};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scale::{log_breaks, log_limits, share, sqrt_limits};
use zeff_plot::scene::{Misplaced, ScaleKind, axis_fault, describe, start, with_layer};

/// A plot of `x` and `y` of the rows, with the scale of an axis set.
fn on(rows: &[Row], aesthetic: Aesthetic, kind: AxisKind) -> Plot {
    let mut plot = start(
        tried(rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    plot.axes.push(PlotAxis::of(aesthetic, kind));
    describe(&mut plot);
    plot
}

fn row(x: f64, y: f64) -> Row {
    (Some(x), Some(y), None, None)
}

fn near(ours: (f64, f64), theirs: (f64, f64)) {
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * b.abs().max(1.0);
    assert!(
        close(ours.0, theirs.0) && close(ours.1, theirs.1),
        "{ours:?} is not {theirs:?}"
    );
}

#[test]
fn the_limits_of_a_log_and_of_a_square_root_axis_are_ggplot2s() {
    // The reference is ggplot2 4.0.3 on the shipped wages, which run from 3.53 to 152.39:
    // `scale_y_log10()` has the limits 2.92425753208 and 183.95667758324, and
    // `scale_y_sqrt()` 1.8374864437 and 165.5834864437; four values from 0.5 to 80,000 have
    // the log limits 0.274640135827 and 145645.136242, and 1 to 4 the square-root limits
    // 0.9025 and 4.2025.
    near(
        log_limits(Some((3.53, 152.39))),
        (2.92425753208, 183.95667758324),
    );
    near(
        sqrt_limits(Some((3.53, 152.39))),
        (1.8374864437, 165.5834864437),
    );
    near(
        log_limits(Some((0.5, 80000.0))),
        (0.274640135827, 145645.136242),
    );
    near(sqrt_limits(Some((1.0, 4.0))), (0.9025, 4.2025));
    // One value, and none.
    near(
        log_limits(Some((100.0, 100.0))),
        (31.6227766017, 316.227766017),
    );
    assert_eq!(log_limits(None), (1.0, 10.0));
    assert_eq!(sqrt_limits(None), (0.0, 1.0));
    // A square-root axis is never below zero.
    assert_eq!(sqrt_limits(Some((0.0, 100.0))).0, 0.0);
    // What has no place gives the limits of an empty axis, and the widest stay finite.
    assert_eq!(log_limits(Some((0.0, 5.0))), (1.0, 10.0));
    let (lo, hi) = log_limits(Some((f64::MIN_POSITIVE, f64::MAX)));
    assert!(lo > 0.0 && hi.is_finite() && lo < hi);
}

#[test]
fn the_breaks_of_a_log_axis_are_powers_of_ten_or_one_two_and_five_times_one() {
    // The wages, over less than two decades: 1, 2 and 5 times a power of ten.
    let (lo, hi) = log_limits(Some((3.53, 152.39)));
    assert_eq!(log_breaks(lo, hi), [5.0, 10.0, 20.0, 50.0, 100.0]);
    // Over three decades or more: the powers of ten.
    let (lo, hi) = log_limits(Some((0.5, 80000.0)));
    assert_eq!(
        log_breaks(lo, hi),
        [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0]
    );
    // Over very many: one power in two, or in three, so that there are seven or fewer.
    assert_eq!(log_breaks(1e-9, 2e9).len(), 7);
    assert!(log_breaks(1e-300, 1e300).len() <= 8);
    // Each is the double nearest its decimal, and a break may stand on a limit.
    assert_eq!(
        log_breaks(0.0005, 0.3),
        [0.0005, 0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2]
    );
    // Under one decade, the round numbers of a linear axis.
    assert_eq!(log_breaks(20.0, 60.0), [20.0, 30.0, 40.0, 50.0, 60.0]);
    assert_eq!(log_breaks(3.0, 4.0), zeff_plot::scale::breaks(3.0, 4.0));
}

#[test]
fn a_mark_stands_at_the_share_its_logarithm_or_its_root_has() {
    // On a log axis from 1 to 1,000, 10 is a third of the way and 100 two thirds.
    assert!((share(ScaleKind::Log, (1.0, 1000.0), 10.0) - 1.0 / 3.0).abs() < 1e-12);
    assert!((share(ScaleKind::Log, (1.0, 1000.0), 100.0) - 2.0 / 3.0).abs() < 1e-12);
    // On a square-root axis from 0 to 100, 25 is half way.
    assert_eq!(share(ScaleKind::Sqrt, (0.0, 100.0), 25.0), 0.5);
    assert_eq!(share(ScaleKind::Linear, (0.0, 100.0), 25.0), 0.25);

    // In a scene and on the page: 1, 10, 100 and 1,000 are equally far apart.
    let rows = [
        row(1.0, 1.0),
        row(2.0, 10.0),
        row(3.0, 100.0),
        row(4.0, 1000.0),
    ];
    let plot = on(&rows, Aesthetic::Y, AxisKind::Log);
    let (_, scene) = with_layer(&plot, points(1.0));
    let y = scene.scale("y").unwrap();
    assert_eq!(y.kind, ScaleKind::Log);
    near(y.limits, (0.707945784384, 1412.53754462));
    assert_eq!(y.breaks, [1.0, 10.0, 100.0, 1000.0]);
    assert_eq!(y.labels, ["1", "10", "100", "1000"]);
    assert_eq!(scene.scale("x").unwrap().kind, ScaleKind::Linear);
    assert!(scene.json().write().contains("\"kind\": \"log\""));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    let ys: Vec<f64> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { at, .. } => Some(at.1),
            _ => None,
        })
        .collect();
    assert_eq!(ys.len(), 4);
    let steps: Vec<f64> = ys.windows(2).map(|pair| pair[0] - pair[1]).collect();
    assert!(steps[0] > 0.0);
    assert!(
        (steps[0] - steps[1]).abs() < 1e-9 && (steps[1] - steps[2]).abs() < 1e-9,
        "{steps:?}"
    );
    insta::assert_snapshot!("four_decades", drawing.describe());

    // A square-root axis: 0, 25 and 100 are equally far apart, and the axis starts at zero.
    let rows = [row(1.0, 0.0), row(2.0, 25.0), row(3.0, 100.0)];
    let plot = on(&rows, Aesthetic::Y, AxisKind::Sqrt);
    let (_, scene) = with_layer(&plot, points(1.0));
    let y = scene.scale("y").unwrap();
    assert_eq!((y.kind, y.limits.0), (ScaleKind::Sqrt, 0.0));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    let ys: Vec<f64> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { at, .. } => Some(at.1),
            _ => None,
        })
        .collect();
    assert!((ys[0] - ys[1] - (ys[1] - ys[2])).abs() < 1e-9, "{ys:?}");
}

#[test]
fn what_has_no_place_on_a_scale_is_found_and_named() {
    // Rows at or below zero on a log axis: the first of them, counted from 1, and how many.
    let rows = [
        row(1.0, 5.0),
        row(2.0, 0.0),
        row(3.0, -2.0),
        (Some(4.0), None, None, None),
        row(5.0, 7.0),
    ];
    let plot = on(&rows, Aesthetic::Y, AxisKind::Log);
    assert!(
        axis_fault(&plot).is_none(),
        "a plot without a layer draws no row"
    );
    let (plot, scene) = with_layer(&plot, points(1.0));
    let fault = axis_fault(&plot).unwrap();
    assert_eq!((fault.axis, fault.kind), ("y", AxisKind::Log));
    assert_eq!(
        fault.what,
        Misplaced::Rows {
            variable: "y".to_string(),
            frame: "d".to_string(),
            rows: vec![2, 3],
            count: 2,
        }
    );
    // The scene of such a plot still stands: the rows without a place are not drawn.
    assert_eq!(scene.panels[0].layers[0].y, [5.0, 7.0]);
    // A square-root axis holds zero and not what is below it.
    let plot = with_layer(&on(&rows, Aesthetic::Y, AxisKind::Sqrt), points(1.0)).0;
    assert!(matches!(
        axis_fault(&plot).unwrap().what,
        Misplaced::Rows { ref rows, count: 1, .. } if rows == &[3]
    ));
    // The other axis, and a linear one, find nothing.
    let plot = with_layer(&on(&rows, Aesthetic::X, AxisKind::Log), points(1.0)).0;
    assert!(axis_fault(&plot).is_none());
    // A bar and an area reach down to zero, which a log axis does not hold.
    let positive = [row(1.0, 5.0), row(2.0, 7.0)];
    for mark in [
        Mark::Area(Stacking::Overlay),
        Mark::Bars(Default::default()),
    ] {
        let layer = PlotLayer {
            mark,
            ..points(1.0)
        };
        let (plot, scene) = with_layer(&on(&positive, Aesthetic::Y, AxisKind::Log), layer);
        assert_eq!(
            axis_fault(&plot).unwrap().what,
            Misplaced::Zero(mark.name())
        );
        assert!(scene.panels[0].layers[0].x.is_empty());
        // On a square-root axis both are drawn from zero.
        let layer = PlotLayer {
            mark,
            ..points(1.0)
        };
        let (plot, scene) = with_layer(&on(&positive, Aesthetic::Y, AxisKind::Sqrt), layer);
        assert!(axis_fault(&plot).is_none());
        assert_eq!(scene.panels[0].layers[0].x.len(), 2);
    }
    // An axis of categories has no other scale.
    let mut plot = start(
        tried(&[(Some(1.0), Some(1.0), Some(0), None)]),
        vec![mapped(Aesthetic::X, "g"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    plot.axes.push(PlotAxis::of(Aesthetic::X, AxisKind::Log));
    assert_eq!(
        axis_fault(&plot).unwrap().what,
        Misplaced::Categories("g".to_string())
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn a_break_is_never_outside_the_limits_and_every_mark_is_within_them(
        rows in rows(),
        which in 0u8..3,
    ) {
        let kind = [AxisKind::Linear, AxisKind::Log, AxisKind::Sqrt][which as usize];
        let plot = on(&rows, Aesthetic::Y, kind);
        let (_, scene) = with_layer(&plot, points(1.0));
        let y = scene.scale("y").unwrap();
        let (lo, hi) = y.limits;
        prop_assert!(lo.is_finite() && hi.is_finite() && lo < hi, "{:?}", y);
        if kind == AxisKind::Log {
            prop_assert!(lo > 0.0);
        }
        if kind == AxisKind::Sqrt {
            prop_assert!(lo >= 0.0);
        }
        prop_assert_eq!(y.breaks.len(), y.labels.len());
        for b in &y.breaks {
            prop_assert!(lo <= *b && *b <= hi, "{} outside {} to {}", b, lo, hi);
        }
        // The marks drawn are the rows with a place on the axis, each within its limits and
        // at a share of the way that is a number.
        let layer = &scene.panels[0].layers[0];
        for value in &layer.y {
            prop_assert!(lo <= *value && *value <= hi);
            let at = share(y.kind, y.limits, *value);
            prop_assert!((-1e-9..=1.0 + 1e-9).contains(&at), "{} at {}", value, at);
        }
        prop_assert!(layout(&scene, Size::OneColumn, &JOURNAL).is_ok());
    }
}
