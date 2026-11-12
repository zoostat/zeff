//! `tile` in the scene and on the page (Language Reference, section 7.6, "Marks"; release
//! 0.7.0, step 24c): a rectangle at each row's `x` and `y`, as wide and as high as the gap
//! between two places, filled by a variable on the scale of fills, of categories or of
//! numbers; the legend of a fill by a number is a bar of the ramp.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer};
use zeff_plot::drawing::Primitive;
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{Paint, Range, ScaleKind, Scene, start, with_layer};

fn tile() -> PlotLayer {
    PlotLayer {
        mark: Mark::Tile,
        ..points(1.0)
    }
}

/// Six cells: three months of two years, the year as text, with the sales of each.
fn grid() -> Frame {
    frame(
        "g",
        vec![
            ("month", numbers(&[1.0, 2.0, 3.0, 1.0, 2.0, 3.0])),
            (
                "year",
                Column::strs(["2025", "2025", "2025", "2026", "2026", "2026"].map(Some)),
            ),
            ("sales", numbers(&[4.0, 7.0, 9.0, 5.0, 8.0, 12.0])),
        ],
    )
}

/// The tiles of the first layer as ggplot2 lists them: `xmin`, `xmax`, `ymin`, `ymax`.
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

#[test]
fn a_tile_covers_its_cell_as_ggplot2_draws_it() {
    // The reference is `geom_tile` of ggplot2 4.0.3 on the same six rows: the edges of each
    // tile, and the limits 0.35 to 3.65 along `x`, numbers, and 0.4 to 2.6 along `y`,
    // categories.
    let plot = start(
        grid(),
        vec![
            mapped(Aesthetic::X, "month"),
            mapped(Aesthetic::Y, "year"),
            mapped(Aesthetic::Fill, "sales"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, tile());
    assert_eq!(
        edges(&scene),
        [
            (0.5, 1.5, 0.5, 1.5),
            (1.5, 2.5, 0.5, 1.5),
            (2.5, 3.5, 0.5, 1.5),
            (0.5, 1.5, 1.5, 2.5),
            (1.5, 2.5, 1.5, 2.5),
            (2.5, 3.5, 1.5, 2.5),
        ]
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Tile);
    assert_eq!(layer.rows, [1, 2, 3, 4, 5, 6]);
    // Each tile has the value of its row on the scale of fills, which runs exactly from the
    // smallest to the largest.
    assert_eq!(
        layer.fill,
        Paint::Scaled(vec![4.0, 7.0, 9.0, 5.0, 8.0, 12.0])
    );
    let fill = scene.scale("fill").unwrap();
    assert_eq!((fill.kind, fill.limits), (ScaleKind::Linear, (4.0, 12.0)));
    assert!(matches!(fill.range, Range::Colours(_)));
    let x = scene.scale("x").unwrap().limits;
    assert!(
        (x.0 - 0.35).abs() < 1e-12 && (x.1 - 3.65).abs() < 1e-12,
        "{x:?}"
    );
    assert_eq!(scene.scale("y").unwrap().limits, (0.4, 2.6));
    assert_eq!(scene.legends[0].marks, [Mark::Tile]);
    let json = scene.json().write();
    assert!(json.contains("\"mark\": \"tile\""), "{json}");
    assert!(json.contains("\"span\": [1, 1, 1, 1, 1, 1]"), "{json}");
    assert!(json.contains("\"fill\": [4, 7, 9, 5, 8, 12]"), "{json}");

    // Along numbers on both axes a tile is the smallest gap high: 3.05 to 12.95 in ggplot2.
    let plot = start(
        grid(),
        vec![mapped(Aesthetic::X, "month"), mapped(Aesthetic::Y, "sales")],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, tile());
    assert_eq!(edges(&scene)[0], (0.5, 1.5, 3.5, 4.5));
    assert_eq!(edges(&scene)[5], (2.5, 3.5, 11.5, 12.5));
    let y = scene.scale("y").unwrap().limits;
    assert!(
        (y.0 - 3.05).abs() < 1e-12 && (y.1 - 12.95).abs() < 1e-12,
        "{y:?}"
    );
    // With nothing to take a fill from, the fill of the theme.
    assert_eq!(scene.panels[0].layers[0].fill, Paint::Theme);
}

#[test]
fn a_tile_is_a_rectangle_in_the_colour_of_its_value_and_the_legend_a_bar_of_the_ramp() {
    let plot = start(
        grid(),
        vec![
            mapped(Aesthetic::X, "month"),
            mapped(Aesthetic::Y, "year"),
            mapped(Aesthetic::Fill, "sales"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, tile());
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 6);
    let fills: Vec<String> = marks
        .iter()
        .map(|item| match &item.primitive {
            Primitive::Polygon {
                rings,
                fill: Some(fill),
                stroke: None,
                opacity,
            } => {
                assert_eq!((rings[0].len(), *opacity), (4, 1.0));
                fill.clone()
            }
            other => panic!("{other:?} is not a tile"),
        })
        .collect();
    // The smallest value is the first colour of viridis and the largest the last.
    assert_eq!(fills[0], "#440154");
    assert_eq!(fills[5], "#FDE725");
    // The tiles of one row of the grid touch: the right edge of one is the left of the next.
    let ring = |i: usize| match &marks[i].primitive {
        Primitive::Polygon { rings, .. } => rings[0].clone(),
        _ => unreachable!(),
    };
    assert_eq!(ring(0)[1].0, ring(1)[0].0);
    assert_eq!(ring(0)[2].1, ring(3)[0].1);
    // The legend is a bar of the colours, as for a colour by a number.
    let bar = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:fill"))
        .filter(|item| matches!(item.primitive, Primitive::Raster { .. }))
        .count();
    assert_eq!(bar, 1);
    insta::assert_snapshot!("six_tiles", drawing.describe());
}

#[test]
fn a_fill_by_categories_and_the_rows_that_are_not_drawn() {
    let g = frame(
        "g",
        vec![
            ("a", Column::strs([Some("x"), Some("y"), None, Some("y")])),
            (
                "b",
                Column::strs([Some("p"), Some("p"), Some("q"), Some("q")]),
            ),
            (
                "k",
                Column::strs([Some("low"), None, Some("high"), Some("high")]),
            ),
            ("v", floats(&[Some(1.0), Some(2.0), Some(3.0), None])),
        ],
    );
    // Categories: a row without a place is not drawn; one without a category of its fill
    // is, in the grey of `NA`.
    let plot = start(
        g.clone(),
        vec![
            mapped(Aesthetic::X, "a"),
            mapped(Aesthetic::Y, "b"),
            mapped(Aesthetic::Fill, "k"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, tile());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.rows, [1, 2, 4]);
    assert_eq!(scene.notes, ["1 row with NA in a or b dropped"]);
    let fill = scene.scale("fill").unwrap();
    assert_eq!(fill.labels, ["high", "low", "NA"]);
    assert_eq!(layer.fill, Paint::Scaled(vec![2.0, 3.0, 1.0]));
    // A number: a row without it has no place on the ramp and is not drawn.
    let plot = start(
        g,
        vec![
            mapped(Aesthetic::X, "a"),
            mapped(Aesthetic::Y, "b"),
            mapped(Aesthetic::Fill, "v"),
        ],
        Vec::new(),
    );
    let (_, scene) = with_layer(&plot, tile());
    assert_eq!(scene.panels[0].layers[0].rows, [1, 2]);
    assert_eq!(scene.notes, ["2 rows with NA in a, b or v dropped"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_tile_is_centred_on_its_row_and_within_the_axes(rows in rows()) {
        let plot = start(
            tried(&rows),
            vec![
                mapped(Aesthetic::X, "x"),
                mapped(Aesthetic::Y, "y"),
                mapped(Aesthetic::Fill, "g"),
            ],
            Vec::new(),
        );
        let (_, scene) = with_layer(&plot, tile());
        let layer = &scene.panels[0].layers[0];
        prop_assert_eq!(layer.x.len(), layer.span.len());
        prop_assert_eq!(layer.x.len(), layer.low.len());
        let (x, y) = (scene.scale("x").unwrap().limits, scene.scale("y").unwrap().limits);
        for (i, row) in layer.rows.iter().enumerate() {
            let (cx, cy, ..) = rows[row - 1];
            // One size for every tile of the layer, above zero.
            prop_assert!(layer.span[i] > 0.0 && layer.span[i] == layer.span[0]);
            // A tile has a height, unless the gap between two values is too small beside
            // the values themselves for a double to hold half of it.
            let high = layer.y[i] - layer.low[i];
            prop_assert!(high >= 0.0);
            // Centred on the row, to within the rounding of numbers of the size of the tile.
            prop_assert_eq!(Some(layer.x[i]), cx);
            let centre = layer.low[i] + high / 2.0;
            let slack = 1e-9 * (cy.unwrap().abs() + high).max(1.0);
            prop_assert!((centre - cy.unwrap()).abs() <= slack);
            // The axes hold the whole of it.
            prop_assert!(x.0 <= layer.x[i] - layer.span[i] / 2.0);
            prop_assert!(layer.x[i] + layer.span[i] / 2.0 <= x.1);
            prop_assert!(y.0 <= layer.low[i] && layer.y[i] <= y.1);
        }
        prop_assert!(layout(&scene, Size::OneColumn, &JOURNAL).is_ok());
    }
}
