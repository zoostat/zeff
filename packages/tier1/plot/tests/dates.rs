//! An axis of dates (Language Reference, section 7.6, "Scales"; release 0.7.0, step 25c1): a
//! Date or a DateTime variable mapped to `x` or `y` is placed in proportion to the time, as
//! the microseconds since 1970-01-01 00:00:00 UTC; the breaks fall on the calendar and are
//! written as dates; the layers of a plot share the axis; a missing date is not drawn and is
//! counted; and the layout places a mark at the share of the time between the limits.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Arrangement, AxisKind, Frame, Mark, PlotAxis, PlotLayer};
use zeff_dates::days_from_civil;
use zeff_plot::dates::{DAY, HOUR, breaks, labels, limits};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Misplaced, Range, ScaleKind, axis_fault, describe, start, with_layer};

/// A day as its count since 1970-01-01.
fn day(year: i32, month: u32, date: u32) -> i32 {
    days_from_civil(year, month, date).unwrap()
}

/// A day as its place on the scale of dates.
fn at(year: i32, month: u32, date: u32) -> f64 {
    f64::from(day(year, month, date)) * DAY as f64
}

/// Five readings: the day of each, with one missing, and the level read.
fn readings() -> Frame {
    frame(
        "r",
        vec![
            (
                "day",
                Column::dates([
                    Some(day(2024, 1, 15)),
                    Some(day(2024, 2, 29)),
                    None,
                    Some(day(2024, 4, 1)),
                    Some(day(2024, 6, 20)),
                ]),
            ),
            ("level", numbers(&[3.0, 5.0, 4.0, 8.0, 6.0])),
        ],
    )
}

fn layer(mark: Mark) -> PlotLayer {
    PlotLayer {
        mark,
        ..points(1.0)
    }
}

#[test]
fn a_date_stands_at_its_midnight_and_the_breaks_on_the_calendar() {
    let plot = start(
        readings(),
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let drawn = &scene.panels[0].layers[0];
    assert_eq!(
        drawn.x,
        [
            at(2024, 1, 15),
            at(2024, 2, 29),
            at(2024, 4, 1),
            at(2024, 6, 20)
        ]
    );
    assert_eq!(drawn.y, [3.0, 5.0, 8.0, 6.0]);
    let x = scene.scale("x").unwrap();
    assert_eq!(x.kind, ScaleKind::Date);
    assert_eq!(x.range, Range::Position);
    assert_eq!(x.title, "day");
    // 157 days from 15 January to 20 June: 5% of them, 7.85 days, beyond each end.
    let pad = 0.05 * 157.0 * DAY as f64;
    assert_eq!(x.limits, (at(2024, 1, 15) - pad, at(2024, 6, 20) + pad));
    assert_eq!(
        x.breaks,
        [
            at(2024, 2, 1),
            at(2024, 3, 1),
            at(2024, 4, 1),
            at(2024, 5, 1),
            at(2024, 6, 1)
        ]
    );
    assert_eq!(
        x.labels,
        ["2024-02", "2024-03", "2024-04", "2024-05", "2024-06"]
    );
    assert_eq!(scene.notes, ["1 row with NA in day or level dropped"]);
    // The other axis keeps its linear scale.
    assert_eq!(scene.scale("y").unwrap().kind, ScaleKind::Linear);
    // The scene says the kind by its name.
    assert!(scene.json().write().contains("\"kind\": \"date\""));
}

#[test]
fn a_date_and_time_stands_at_its_instant() {
    let noon = i64::from(day(2024, 3, 5)) * DAY + 12 * HOUR;
    let times = frame(
        "t",
        vec![
            (
                "when",
                Column::datetimes([Some(noon), Some(noon + 3 * HOUR), Some(noon + 7 * HOUR)]),
            ),
            ("level", numbers(&[1.0, 2.0, 3.0])),
        ],
    );
    let plot = start(
        times,
        vec![mapped(Aesthetic::X, "when"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, layer(Mark::Lines));
    let x = scene.scale("x").unwrap();
    assert_eq!(x.kind, ScaleKind::Date);
    assert_eq!(
        scene.panels[0].layers[0].x,
        [
            noon as f64,
            (noon + 3 * HOUR) as f64,
            (noon + 7 * HOUR) as f64
        ]
    );
    // Seven hours, marked by the hour, within one day: the time of the clock alone.
    assert_eq!(
        x.labels,
        [
            "12:00", "13:00", "14:00", "15:00", "16:00", "17:00", "18:00", "19:00"
        ]
    );
}

#[test]
fn dates_along_y_and_on_both_axes() {
    let pairs = frame(
        "p",
        vec![
            (
                "ordered",
                Column::dates([
                    Some(day(2020, 1, 1)),
                    Some(day(2021, 1, 1)),
                    Some(day(2022, 1, 1)),
                ]),
            ),
            (
                "delivered",
                Column::dates([
                    Some(day(2020, 3, 1)),
                    Some(day(2021, 6, 1)),
                    Some(day(2023, 1, 1)),
                ]),
            ),
        ],
    );
    let plot = start(
        pairs,
        vec![
            mapped(Aesthetic::X, "ordered"),
            mapped(Aesthetic::Y, "delivered"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let (x, y) = (scene.scale("x").unwrap(), scene.scale("y").unwrap());
    assert_eq!((x.kind, y.kind), (ScaleKind::Date, ScaleKind::Date));
    assert_eq!(
        x.labels,
        ["2020-01", "2020-07", "2021-01", "2021-07", "2022-01"]
    );
    assert_eq!(
        y.labels,
        [
            "2020-07", "2021-01", "2021-07", "2022-01", "2022-07", "2023-01"
        ]
    );
    assert_eq!(
        scene.panels[0].layers[0].y,
        [at(2020, 3, 1), at(2021, 6, 1), at(2023, 1, 1)]
    );
}

#[test]
fn one_day_and_no_day_still_give_an_axis() {
    let one = frame(
        "o",
        vec![
            ("day", Column::dates([Some(day(2024, 3, 5))])),
            ("level", numbers(&[2.0])),
        ],
    );
    let plot = start(
        one,
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let x = scene.scale("x").unwrap();
    assert_eq!(x.limits, (at(2024, 3, 4), at(2024, 3, 6)));
    assert_eq!(x.labels, ["2024-03-04", "2024-03-05", "2024-03-06"]);
    // One bar at one date is nine tenths of a day wide.
    let (_, scene) = with_layer(&plot, layer(Mark::Bars(Arrangement::Stack)));
    assert_eq!(scene.panels[0].layers[0].span, [0.9 * DAY as f64]);
    // No date at all: the first day of 1970, as a linear axis with no value runs from 0 to 1.
    let none = frame(
        "n",
        vec![
            ("day", Column::dates([None, None])),
            ("level", numbers(&[2.0, 3.0])),
        ],
    );
    let plot = start(
        none,
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let x = scene.scale("x").unwrap();
    assert_eq!(x.kind, ScaleKind::Date);
    assert_eq!(x.limits, (0.0, DAY as f64));
    assert_eq!(x.labels, ["1970-01-01", "1970-01-02"]);
    assert!(scene.panels[0].layers[0].x.is_empty());
    assert_eq!(scene.notes, ["2 rows with NA in day or level dropped"]);
}

#[test]
fn before_a_mark_is_drawn_the_axis_has_the_extent_of_the_frame() {
    let mut plot = start(
        readings(),
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let scene = describe(&mut plot);
    let x = scene.scale("x").unwrap();
    assert_eq!(x.kind, ScaleKind::Date);
    assert_eq!(
        x.labels,
        ["2024-02", "2024-03", "2024-04", "2024-05", "2024-06"]
    );
}

#[test]
fn a_layer_of_times_shares_an_axis_of_days() {
    // The plot's days and a layer's own instants, in another frame, stand on one axis.
    let plot = start(
        readings(),
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let noon = i64::from(day(2024, 3, 5)) * DAY + 12 * HOUR;
    let own = frame(
        "e",
        vec![
            ("when", Column::datetimes([Some(noon)])),
            ("level", numbers(&[7.0])),
        ],
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let mut over = points(1.0);
    over.data = Some(own);
    over.mappings = vec![mapped(Aesthetic::X, "when")];
    let (_, scene) = with_layer(&plot, over);
    assert_eq!(scene.panels[0].layers[1].x, [noon as f64]);
    assert_eq!(scene.scale("x").unwrap().kind, ScaleKind::Date);
}

#[test]
fn a_log_or_a_square_root_scale_has_no_place_for_dates() {
    for kind in [AxisKind::Log, AxisKind::Sqrt] {
        let mut plot = start(
            readings(),
            vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
            Vec::new(),
        );
        plot.axes.push(PlotAxis::of(Aesthetic::X, kind));
        let fault = axis_fault(&plot).unwrap();
        assert_eq!(fault.axis, "x");
        assert_eq!(fault.what, Misplaced::Dates("day".to_string()));
    }
}

#[test]
fn the_layout_places_a_date_at_the_share_of_the_time() {
    let plot = start(
        readings(),
        vec![mapped(Aesthetic::X, "day"), mapped(Aesthetic::Y, "level")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (place, width, _, items) = panel_of(&drawing);
    let x = scene.scale("x").unwrap();
    let centres: Vec<f64> = items
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { at, .. } => Some(at.0),
            _ => None,
        })
        .collect();
    assert_eq!(centres.len(), 4);
    for (centre, date) in centres.iter().zip(&scene.panels[0].layers[0].x) {
        let share = (date - x.limits.0) / (x.limits.1 - x.limits.0);
        let expected = place.0 + share * width;
        assert!(
            (centre - expected).abs() < 1e-6,
            "{centre} is not {expected}"
        );
    }
    // The labels are written under the panel, every second one, since all five would touch.
    let written = drawing.describe();
    for (i, label) in x.labels.iter().enumerate() {
        assert_eq!(
            written.contains(&format!("{label:?}")),
            i % 2 == 0,
            "{label} in {written}"
        );
    }
    insta::assert_snapshot!("readings_by_day", written);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(256)))]

    /// Whatever the limits, within the calendar: a break is never outside them, the breaks
    /// rise, each has a label of its own, and they are few enough to read.
    #[test]
    fn the_breaks_lie_within_the_limits_and_rise(
        start in -700_000_i64..2_900_000,
        minutes in 1_i64..60 * 24 * 365 * 400,
        whole_days in any::<bool>(),
    ) {
        let lo = (start * DAY) as f64;
        let hi = lo + (minutes * 60_000_000) as f64;
        let found = breaks(lo, hi, whole_days);
        prop_assert!(!found.is_empty());
        prop_assert!(found.len() <= 16, "{} breaks", found.len());
        prop_assert!(found.iter().all(|b| *b >= lo && *b <= hi));
        prop_assert!(found.windows(2).all(|pair| pair[0] < pair[1]));
        let written = labels(&found);
        prop_assert_eq!(written.len(), found.len());
        prop_assert!(written.windows(2).all(|pair| pair[0] != pair[1]), "{:?}", written);
    }

    /// The limits hold every value, with the same margin at both ends.
    #[test]
    fn the_limits_hold_the_extent(
        first in -700_000_i64..2_900_000,
        days in 1_i64..40_000,
    ) {
        let (lo, hi) = ((first * DAY) as f64, ((first + days) * DAY) as f64);
        let (low, high) = limits(Some((lo, hi)), true);
        prop_assert!(low < lo && high > hi);
        prop_assert_eq!(lo - low, high - hi);
    }
}
