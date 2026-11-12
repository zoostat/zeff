//! An axis of categories (Language Reference, section 7.6; open decision #128; release 0.7.0,
//! step 24a): a Str or a Category variable mapped to `x` or `y` stands its categories at the
//! places 1, 2, 3 in their order, with 0.6 of a place beyond each end; the layers of a plot
//! share the axis; a row without a category is not drawn and is counted.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Range, ScaleKind, start, with_layer};

fn texts(values: &[Option<&str>]) -> Column {
    Column::strs(values.iter().copied())
}

/// Five sales: the shop, as text, and the units sold.
fn shops() -> Frame {
    frame(
        "d",
        vec![
            (
                "shop",
                texts(&[
                    Some("north"),
                    Some("south"),
                    Some("east"),
                    Some("north"),
                    Some("south"),
                ]),
            ),
            ("units", numbers(&[12.0, 18.0, 15.0, 7.0, 9.0])),
        ],
    )
}

#[test]
fn the_categories_stand_at_one_two_three_as_ggplot2_places_them() {
    // The reference is ggplot2 4.0.3, `geom_point` on the same five rows with the shop on
    // `x`: the places 2, 3, 1, 2, 3 of the rows, the labels east, north, south, and the limits
    // 0.4 to 3.6; the units keep their linear axis, 6.45 to 18.55.
    let plot = start(
        shops(),
        vec![mapped(Aesthetic::X, "shop"), mapped(Aesthetic::Y, "units")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [2.0, 3.0, 1.0, 2.0, 3.0]);
    assert_eq!(layer.y, [12.0, 18.0, 15.0, 7.0, 9.0]);
    let x = scene.scale("x").unwrap();
    assert_eq!(x.kind, ScaleKind::Category);
    assert_eq!(x.range, Range::Position);
    assert_eq!(x.limits, (0.4, 3.6));
    assert_eq!(x.breaks, [1.0, 2.0, 3.0]);
    assert_eq!(x.labels, ["east", "north", "south"]);
    assert_eq!(x.title, "shop");
    let y = scene.scale("y").unwrap();
    assert_eq!((y.kind, y.limits), (ScaleKind::Linear, (6.45, 18.55)));
    assert!(scene.notes.is_empty());
    let json = scene.json().write();
    assert!(json.contains("\"kind\": \"category\""), "{json}");
    assert!(
        json.contains("\"labels\": [\"east\", \"north\", \"south\"]"),
        "{json}"
    );

    // Along `y` the same, with the first category at the bottom.
    let plot = start(
        shops(),
        vec![mapped(Aesthetic::X, "units"), mapped(Aesthetic::Y, "shop")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    assert_eq!(scene.panels[0].layers[0].y, [2.0, 3.0, 1.0, 2.0, 3.0]);
    assert_eq!(scene.scale("y").unwrap().limits, (0.4, 3.6));

    // Before any layer, and with one category alone, the axis has the room of its places.
    assert_eq!(
        start(shops(), vec![mapped(Aesthetic::X, "shop")], Vec::new())
            .layers
            .len(),
        0
    );
    let one = frame(
        "d",
        vec![("g", texts(&[Some("only")])), ("v", numbers(&[1.0]))],
    );
    let plot = start(
        one,
        vec![mapped(Aesthetic::X, "g"), mapped(Aesthetic::Y, "v")],
        Vec::new(),
    );
    assert_eq!(
        with_layer(&plot, points(1.0)).1.scale("x").unwrap().limits,
        (0.4, 1.6)
    );
}

#[test]
fn a_label_is_written_under_each_place() {
    let plot = start(
        shops(),
        vec![mapped(Aesthetic::X, "shop"), mapped(Aesthetic::Y, "units")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, points(1.0));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (corner, width, _, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 5);
    // The place of a category is its share of the way from 0.4 to 3.6.
    let place = |at: f64| corner.0 + (at - 0.4) / 3.2 * width;
    let labels: Vec<(f64, String)> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("axis:x"))
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, spans, .. } => Some((
                at.0,
                spans
                    .iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>(),
            )),
            _ => None,
        })
        .collect();
    for (at, label) in [(1.0, "east"), (2.0, "north"), (3.0, "south")] {
        assert!(
            labels
                .iter()
                .any(|(x, text)| text == label && (x - place(at)).abs() < 1e-9),
            "{label} is not under its place: {labels:?}"
        );
    }
    // The points of the first and the fourth row stand over "north".
    let over: Vec<f64> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { at, .. } => Some(at.0),
            _ => None,
        })
        .collect();
    assert!((over[0] - place(2.0)).abs() < 1e-9 && (over[3] - place(2.0)).abs() < 1e-9);
    insta::assert_snapshot!("points_by_shop", drawing.describe());
}

#[test]
fn the_layers_share_the_axis_and_a_row_without_a_category_is_counted() {
    // The plot's frame has north and south and a row with no shop; a layer's own frame has
    // south again and west, which takes the place after the plot's own.
    let base = frame(
        "d",
        vec![
            (
                "shop",
                texts(&[Some("south"), None, Some("north"), Some("south")]),
            ),
            ("units", numbers(&[1.0, 2.0, 3.0, 4.0])),
        ],
    );
    let more = frame(
        "more",
        vec![
            ("shop", texts(&[Some("west"), Some("south")])),
            ("units", numbers(&[5.0, 6.0])),
        ],
    );
    let plot = start(
        base,
        vec![mapped(Aesthetic::X, "shop"), mapped(Aesthetic::Y, "units")],
        Vec::new(),
    );
    let (plot, scene) = with_layer(&plot, points(1.0));
    assert_eq!(scene.panels[0].layers[0].x, [2.0, 1.0, 2.0]);
    assert_eq!(scene.panels[0].layers[0].rows, [1, 3, 4]);
    assert_eq!(scene.notes, ["1 row with NA in shop or units dropped"]);
    assert_eq!(scene.scale("x").unwrap().labels, ["north", "south"]);
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Lines,
            data: Some(more),
            ..points(1.0)
        },
    );
    let x = scene.scale("x").unwrap();
    assert_eq!(x.labels, ["north", "south", "west"]);
    assert_eq!(x.limits, (0.4, 3.6));
    // The line of the second layer goes from south to west, in the order of the axis.
    assert_eq!(scene.panels[0].layers[1].x, [2.0, 3.0]);
    assert_eq!(scene.panels[0].layers[1].y, [6.0, 5.0]);
    assert_eq!(scene.panels[0].layers[0].x, [2.0, 1.0, 2.0]);
}

#[test]
fn a_plot_that_maps_nothing_takes_its_axis_of_categories_from_a_layer() {
    let plot = start(shops(), Vec::new(), Vec::new());
    let (_, scene) = with_layer(
        &plot,
        PlotLayer {
            mappings: vec![mapped(Aesthetic::X, "shop"), mapped(Aesthetic::Y, "units")],
            ..points(1.0)
        },
    );
    let x = scene.scale("x").unwrap();
    assert_eq!((x.kind, x.title.as_str()), (ScaleKind::Category, "shop"));
    assert_eq!(scene.panels[0].layers[0].x, [2.0, 3.0, 1.0, 2.0, 3.0]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_row_with_a_category_stands_at_the_place_of_its_label(rows in rows()) {
        // The category `g` of the frame a property is tried on, along `x`.
        let plot = start(
            tried(&rows),
            vec![mapped(Aesthetic::X, "g"), mapped(Aesthetic::Y, "y")],
            Vec::new(),
        );
        let (_, scene) = with_layer(&plot, points(1.0));
        let x = scene.scale("x").unwrap();
        prop_assert_eq!(x.kind, ScaleKind::Category);
        let places = x.labels.len();
        prop_assert_eq!(x.limits, (0.4, places.max(1) as f64 + 0.6));
        prop_assert_eq!(x.breaks.len(), places);
        // The labels are the categories of the frame, each once, in the order of text.
        let mut held: Vec<String> = rows
            .iter()
            .filter_map(|r| r.2.map(|k| format!("group <{k:02}> & co")))
            .collect();
        held.sort();
        held.dedup();
        prop_assert_eq!(&x.labels, &held);
        let layer = &scene.panels[0].layers[0];
        let complete: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.2.is_some() && r.1.is_some_and(f64::is_finite))
            .map(|(i, _)| i + 1)
            .collect();
        prop_assert_eq!(&layer.rows, &complete);
        for (place, row) in layer.x.iter().zip(&layer.rows) {
            let label = format!("group <{:02}> & co", rows[row - 1].2.unwrap());
            prop_assert_eq!(&x.labels[*place as usize - 1], &label);
            prop_assert!(*place >= 1.0 && *place <= places as f64 && place.fract() == 0.0);
        }
        // The scene is laid out whatever the frame holds.
        prop_assert!(layout(&scene, Size::OneColumn, &JOURNAL).is_ok());
    }
}
