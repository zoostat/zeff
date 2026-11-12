//! The labels of a crowded axis (Language Reference, section 7.6: "labels wrapped or thinned,
//! never overlapping"; release 0.7.0, step 26c). Along `x`, a label of an axis of categories
//! wider than its place is broken between words where every label can then be written; where
//! two would still touch, the labels stay whole and every second or third is written. No two
//! labels that are written touch, with two hundred categories as with five.

mod common;

use common::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Plot};
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::font::{FaceName, face};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Scene, start, with_layer};

/// A frame of one row for each name, with a number to plot against it.
fn named(names: &[String]) -> Frame {
    frame(
        "d",
        vec![
            (
                "group",
                Column::strs(names.iter().map(|name| Some(name.as_str()))),
            ),
            (
                "n",
                numbers(
                    &(0..names.len())
                        .map(|i| (i % 7) as f64 + 1.0)
                        .collect::<Vec<f64>>(),
                ),
            ),
        ],
    )
}

fn scene_of(names: &[&str]) -> Scene {
    let names: Vec<String> = names.iter().map(|name| (*name).to_string()).collect();
    let plot: Plot = start(
        named(&names),
        vec![mapped(Aesthetic::X, "group"), mapped(Aesthetic::Y, "n")],
        Vec::new(),
    );
    with_layer(&plot, points(1.0)).1
}

/// The width of a text at the size of the labels of the journal theme.
fn width(text: &str) -> f64 {
    let sans = face(FaceName::Sans);
    f64::from(sans.shape(text).width) * JOURNAL.label_size / f64::from(sans.units_per_em())
}

/// The lines of the labels of the x axis: the text of each, where it is centred, and its
/// baseline; the title of the axis, which is set larger, is left out.
fn x_labels(drawing: &Drawing) -> Vec<(String, f64, f64)> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("axis:x"))
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, spans, .. } => spans
                .first()
                .filter(|span| span.size == JOURNAL.label_size)
                .map(|span| (span.text.clone(), at.0, at.1)),
            _ => None,
        })
        .collect()
}

/// The ticks of the x axis: how many short lines hang under it.
fn x_ticks(drawing: &Drawing) -> usize {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("axis:x"))
        .filter(|item| match &item.primitive {
            Primitive::Segment { from, to, .. } => from.0 == to.0,
            _ => false,
        })
        .count()
}

/// No two lines of labels on one baseline touch: each ends before the next begins.
fn none_touch(labels: &[(String, f64, f64)]) {
    let mut rows: Vec<f64> = labels.iter().map(|(_, _, y)| *y).collect();
    rows.sort_by(f64::total_cmp);
    rows.dedup();
    for row in rows {
        let mut on_row: Vec<(f64, f64, &str)> = labels
            .iter()
            .filter(|(_, _, y)| *y == row)
            .map(|(text, x, _)| (x - width(text) / 2.0, x + width(text) / 2.0, text.as_str()))
            .collect();
        on_row.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in on_row.windows(2) {
            assert!(
                pair[0].1 < pair[1].0,
                "`{}` reaches {} and `{}` starts at {}",
                pair[0].2,
                pair[0].1,
                pair[1].2,
                pair[1].0
            );
        }
    }
}

#[test]
fn labels_too_wide_for_their_places_are_broken_between_words() {
    let names = [
        "New south region",
        "Old north region",
        "Far east region",
        "Mid west region",
        "The islands",
    ];
    let scene = scene_of(&names);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let labels = x_labels(&drawing);
    // Every category is written, word for word, on two lines or three.
    for name in names {
        let at = labels
            .iter()
            .find(|(text, _, _)| name.starts_with(text.as_str()))
            .map(|(_, x, _)| *x)
            .unwrap_or_else(|| panic!("`{name}` is not written: {labels:?}"));
        let mut lines: Vec<(&str, f64)> = labels
            .iter()
            .filter(|(_, x, _)| *x == at)
            .map(|(text, _, y)| (text.as_str(), *y))
            .collect();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        assert!(lines.len() >= 2 && lines.len() <= 3, "{name}: {lines:?}");
        assert_eq!(
            lines
                .iter()
                .map(|(text, _)| *text)
                .collect::<Vec<&str>>()
                .join(" "),
            name
        );
        // The lines are a line of the type apart.
        assert!(lines.windows(2).all(|pair| {
            (pair[1].1 - pair[0].1 - JOURNAL.label_size * JOURNAL.leading).abs() < 1e-9
        }));
    }
    none_touch(&labels);
    assert_eq!(x_ticks(&drawing), 5);
    // The panel gives up the room of the lines: it is lower than with one line of labels.
    let short = layout(
        &scene_of(&["a", "b", "c", "d", "e"]),
        Size::OneColumn,
        &JOURNAL,
    )
    .unwrap();
    let (_, _, height, _) = panel_of(&drawing);
    let (_, _, short_height, _) = panel_of(&short);
    assert!(height < short_height - JOURNAL.label_size);
    insta::assert_snapshot!("five_regions_on_two_or_three_lines", drawing.describe());
}

#[test]
fn labels_that_would_touch_even_so_stay_whole_and_some_are_passed_over() {
    // A word too long for its place: breaking the labels would not let all be written.
    let names = [
        "Public administration",
        "Health and social care",
        "Retail trade",
        "Arts",
    ];
    let drawing = layout(&scene_of(&names), Size::OneColumn, &JOURNAL).unwrap();
    let labels = x_labels(&drawing);
    // Each label that is written is whole, on one line, and they share one baseline.
    assert!(!labels.is_empty() && labels.len() < 4, "{labels:?}");
    assert!(
        labels
            .iter()
            .all(|(text, _, _)| names.contains(&text.as_str()))
    );
    assert!(labels.windows(2).all(|pair| pair[0].2 == pair[1].2));
    none_touch(&labels);
    // Every category keeps its tick.
    assert_eq!(x_ticks(&drawing), 4);
    // On a page of two columns there is room for all four, on one line each.
    let wide = layout(&scene_of(&names), Size::TwoColumn, &JOURNAL).unwrap();
    let labels = x_labels(&wide);
    assert_eq!(labels.len(), 4);
    none_touch(&labels);
}

#[test]
fn two_hundred_categories_never_overlap() {
    let names: Vec<String> = (0..200).map(|i| format!("category {i:03}")).collect();
    let plot: Plot = start(
        named(&names),
        vec![mapped(Aesthetic::X, "group"), mapped(Aesthetic::Y, "n")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    assert_eq!(scene.scale("x").unwrap().labels.len(), 200);
    for size in [Size::OneColumn, Size::TwoColumn, Size::Slide] {
        let drawing = layout(&scene, size, &JOURNAL).unwrap();
        let labels = x_labels(&drawing);
        // Some are written, evenly passed over, and none touches another; all have ticks.
        assert!(
            labels.len() >= 2 && labels.len() < 200,
            "{} labels",
            labels.len()
        );
        none_touch(&labels);
        assert_eq!(x_ticks(&drawing), 200);
        let places: Vec<usize> = labels
            .iter()
            .map(|(text, _, _)| {
                text.trim_start_matches("category ")
                    .parse::<usize>()
                    .unwrap()
            })
            .collect();
        let step = places[1] - places[0];
        assert!(
            places.windows(2).all(|pair| pair[1] - pair[0] == step),
            "{places:?}"
        );
        assert_eq!(places[0], 0);
    }
    // Along `y` the labels are a line high, and are passed over in the same way.
    let plot: Plot = start(
        named(&names),
        vec![mapped(Aesthetic::X, "n"), mapped(Aesthetic::Y, "group")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let mut heights: Vec<f64> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("axis:y"))
        .filter_map(|item| match &item.primitive {
            Primitive::Text {
                at, spans, turn, ..
            } if *turn == 0.0 && !spans.is_empty() => Some(at.1),
            _ => None,
        })
        .collect();
    heights.sort_by(f64::total_cmp);
    assert!(heights.len() >= 2 && heights.len() < 200);
    assert!(
        heights
            .windows(2)
            .all(|pair| pair[1] - pair[0] >= JOURNAL.label_size * 0.7)
    );
}

#[test]
fn an_axis_of_numbers_is_drawn_as_it_was() {
    // Breaking is for categories: the labels of numbers are one line each.
    let plot: Plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let labels = x_labels(&drawing);
    assert!(labels.windows(2).all(|pair| pair[0].2 == pair[1].2));
    none_touch(&labels);
}
