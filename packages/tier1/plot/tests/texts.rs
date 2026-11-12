//! `text` in the scene and on the page (Language Reference, section 7.6, "Marks"; release
//! 0.7.0, step 24d): the label of each row written at its place, in the size of the layer and
//! the colour of the row, moved by the layer's nudge in points.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer};
use zeff_plot::drawing::{Anchor, Primitive};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Paint, Sizing, TEXT_SIZE, start, with_layer};

fn text(label: &str) -> PlotLayer {
    PlotLayer {
        mark: Mark::Text,
        mappings: vec![mapped(Aesthetic::Label, label)],
        ..points(1.0)
    }
}

/// Four countries: income, life expectancy, a name (one missing), and whether rich.
fn countries() -> Frame {
    frame(
        "c",
        vec![
            ("income", numbers(&[12.0, 30.0, 45.0, 20.0])),
            ("life", numbers(&[61.0, 72.0, 80.0, 66.0])),
            (
                "name",
                Column::strs([Some("Aland"), Some("Borduria"), Some("Carpania"), None]),
            ),
            ("rich", Column::strs(["no", "yes", "yes", "no"].map(Some))),
        ],
    )
}

#[test]
fn each_row_writes_its_label_at_its_place() {
    // The limits are those of ggplot2 4.0.3 for `geom_point` and `geom_text` on the same
    // rows: 10.35 to 46.65 and 60.05 to 80.95, the labels counting as the points do.
    let plot = start(
        countries(),
        vec![mapped(Aesthetic::X, "income"), mapped(Aesthetic::Y, "life")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, text("name"));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Text);
    // The row without a name writes nothing, and a note counts it.
    assert_eq!(layer.label, ["Aland", "Borduria", "Carpania"]);
    assert_eq!(layer.rows, [1, 2, 3]);
    assert_eq!(layer.x, [12.0, 30.0, 45.0]);
    assert_eq!(layer.y, [61.0, 72.0, 80.0]);
    assert_eq!(layer.size, Sizing::Constant(TEXT_SIZE));
    assert_eq!(layer.nudge, (0.0, 0.0));
    assert_eq!(scene.notes, ["1 row with NA in name dropped"]);
    let json = scene.json().write();
    assert!(json.contains("\"mark\": \"text\""), "{json}");
    assert!(
        json.contains("\"label\": [\"Aland\", \"Borduria\", \"Carpania\"]"),
        "{json}"
    );
    assert!(json.contains("\"nudge\": [0, 0]"), "{json}");
    // With the points, the axes are those of the four rows.
    let (plot, _) = with_layer(&plot, points(1.0));
    let (_, scene) = with_layer(&plot, text("name"));
    let (x, y) = (
        scene.scale("x").unwrap().limits,
        scene.scale("y").unwrap().limits,
    );
    assert!(
        (x.0 - 10.35).abs() < 1e-12 && (x.1 - 46.65).abs() < 1e-12,
        "{x:?}"
    );
    assert!(
        (y.0 - 60.05).abs() < 1e-12 && (y.1 - 80.95).abs() < 1e-12,
        "{y:?}"
    );
    // A number is written as it prints.
    let (_, scene) = with_layer(&plot, text("life"));
    assert_eq!(scene.panels[0].layers[1].label, ["61", "72", "80", "66"]);
}

#[test]
fn a_label_is_centred_on_its_place_and_moved_by_the_nudge_in_points() {
    let plot = start(
        countries(),
        vec![
            mapped(Aesthetic::X, "income"),
            mapped(Aesthetic::Y, "life"),
            mapped(Aesthetic::Color, "rich"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let mut layer = text("name");
    layer.size = Some(10.0);
    layer.nudge = (3.0, 7.0);
    let (_, scene) = with_layer(&plot, layer);
    assert_eq!(
        scene.panels[0].layers[1].color,
        Paint::Scaled(vec![1.0, 2.0, 2.0])
    );
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    // Four points, then three labels.
    assert_eq!(marks.len(), 7);
    let place = |i: usize| match &marks[i].primitive {
        Primitive::Point { at, .. } => *at,
        other => panic!("{other:?} is not a point"),
    };
    for (i, (word, colour)) in [
        ("Aland", "#E69F00"),
        ("Borduria", "#56B4E9"),
        ("Carpania", "#56B4E9"),
    ]
    .into_iter()
    .enumerate()
    {
        match &marks[4 + i].primitive {
            Primitive::Text {
                at,
                anchor: Anchor::Middle,
                turn,
                spans,
                frame: None,
            } => {
                assert_eq!(*turn, 0.0);
                assert_eq!(spans.len(), 1);
                assert_eq!((spans[0].text.as_str(), spans[0].size), (word, 10.0));
                assert_eq!(spans[0].colour, colour);
                // Three points to the right of its point, and seven above the line on which
                // a label centred on the point would stand.
                let point = place(i);
                assert!((at.0 - point.0 - 3.0).abs() < 1e-9, "{at:?} {point:?}");
                let centred = at.1 + 7.0;
                assert!(
                    centred > point.1 && centred - point.1 < 10.0,
                    "{at:?} {point:?}"
                );
            }
            other => panic!("{other:?} is not a text"),
        }
    }
    insta::assert_snapshot!("three_labels", drawing.describe());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_row_with_a_place_and_a_label_writes_it_once(rows in rows()) {
        // The category `g` as the label, at `x` and `y`.
        let plot = start(
            tried(&rows),
            vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
            Vec::new(),
        );
        let (_, scene) = with_layer(&plot, text("g"));
        let layer = &scene.panels[0].layers[0];
        let has = |value: Option<f64>| value.is_some_and(f64::is_finite);
        let expected: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| has(r.0) && has(r.1) && r.2.is_some())
            .map(|(i, _)| i + 1)
            .collect();
        prop_assert_eq!(&layer.rows, &expected);
        prop_assert_eq!(layer.label.len(), expected.len());
        for (label, row) in layer.label.iter().zip(&expected) {
            prop_assert_eq!(label, &format!("group <{:02}> & co", rows[row - 1].2.unwrap()));
        }
        // On the page, one text for each, whatever the labels hold.
        let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
        let (_, _, _, marks) = panel_of(&drawing);
        let texts = marks
            .iter()
            .filter(|item| matches!(item.primitive, Primitive::Text { .. }))
            .count();
        prop_assert_eq!(texts, expected.len());
    }
}
