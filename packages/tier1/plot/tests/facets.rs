//! A plot parted into panels (Language Reference, section 7.6, "Facets": `facet`; release
//! 0.7.0, step 27d1): the scene of more than one panel. The rows of each layer are parted
//! among the panels by the variables of the facets; the marks of each panel are built within
//! it, on scales trained over all the panels; a layer whose frame lacks a variable of the
//! facets is drawn in every panel.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::{Column, MetadataKey, format_value_labels};
use zeff_core::value::{
    Aesthetic, Arrangement, Coord, FacetScales, Frame, LegendPlace, Mark, Plot, PlotConstant,
    PlotFacet, PlotLayer, PlotSetting, Stacking,
};
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::layout::{JOURNAL, LayoutError, SMALLEST_PANEL, Size, layout};
use zeff_plot::scene::{Scene, describe, rows_and_columns, start, with_layer};

/// A plot of `x` and `y` of a frame, parted by some of its variables.
fn parted(frame: Frame, x: &str, y: &str, by: &[&str]) -> Plot {
    let mut plot: Plot = start(
        frame,
        vec![mapped(Aesthetic::X, x), mapped(Aesthetic::Y, y)],
        Vec::new(),
    );
    if !by.is_empty() {
        plot.facet = Some(PlotFacet {
            by: by.iter().map(|name| (*name).to_string()).collect(),
            ..PlotFacet::default()
        });
    }
    plot
}

fn layer(mark: Mark) -> PlotLayer {
    PlotLayer {
        mark,
        ..points(1.0)
    }
}

fn with(plot: &Plot, mark: Mark) -> Scene {
    with_layer(plot, layer(mark)).1
}

/// Nine rows in three sites: where each was read, when, by which arm, and what.
fn sites() -> Frame {
    frame(
        "d",
        vec![
            (
                "site",
                Column::strs(
                    [
                        "north", "south", "east", "north", "south", "east", "north", "south",
                        "east",
                    ]
                    .map(Some),
                ),
            ),
            (
                "week",
                numbers(&[1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0]),
            ),
            (
                "arm",
                Column::strs(["a", "a", "a", "b", "b", "b", "a", "a", "a"].map(Some)),
            ),
            (
                "count",
                numbers(&[4.0, 7.0, 2.0, 5.0, 9.0, 3.0, 6.0, 8.0, 1.0]),
            ),
        ],
    )
}

#[test]
fn a_plot_parted_by_a_variable_has_a_panel_for_each_of_its_values() {
    let scene = with(&parted(sites(), "week", "count", &["site"]), Mark::Points);
    // A panel for each site, in the order of the categories, in one row.
    let strips: Vec<Option<&str>> = scene.panels.iter().map(|p| p.strip.as_deref()).collect();
    assert_eq!(strips, [Some("east"), Some("north"), Some("south")]);
    let places: Vec<(usize, usize)> = scene.panels.iter().map(|p| (p.row, p.column)).collect();
    assert_eq!(places, [(1, 1), (1, 2), (1, 3)]);
    assert_eq!((scene.rows, scene.columns), (1, 3));
    // Each panel holds the marks of its own rows, in their order, by the rows of the frame.
    let held: Vec<(Vec<usize>, Vec<f64>, Vec<f64>)> = scene
        .panels
        .iter()
        .map(|p| {
            (
                p.layers[0].rows.clone(),
                p.layers[0].x.clone(),
                p.layers[0].y.clone(),
            )
        })
        .collect();
    assert_eq!(
        held[0],
        (vec![3, 6, 9], vec![1.0, 2.0, 3.0], vec![2.0, 3.0, 1.0])
    );
    assert_eq!(
        held[1],
        (vec![1, 4, 7], vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0])
    );
    assert_eq!(
        held[2],
        (vec![2, 5, 8], vec![1.0, 2.0, 3.0], vec![7.0, 9.0, 8.0])
    );
    // The scales are those of the plot without facets, and every panel is placed by them.
    let whole = with(&parted(sites(), "week", "count", &[]), Mark::Points);
    assert_eq!((&scene.scales, &scene.axes), (&whole.scales, &whole.axes));
    assert!(
        scene
            .panels
            .iter()
            .all(|p| (p.x, p.y) == (Some("x"), Some("y")))
    );
    assert_eq!(whole.panels.len(), 1);
    assert_eq!(whole.panels[0].strip, None);
    // The scene says what is written over its panels, and a plot without facets nothing.
    let json = scene.json().write();
    assert!(
        json.contains("\"strips\": [\"east\", \"north\", \"south\"]"),
        "{json}"
    );
    assert!(!whole.json().write().contains("\"strips\""));
    insta::assert_snapshot!("three_sites", json);
}

#[test]
fn the_panels_are_laid_in_rows() {
    let shape = |n: usize| {
        let all = rows_and_columns(n);
        assert_eq!(all.len(), n);
        // From the top left, row by row, each row full but the last.
        for (i, pair) in all.windows(2).enumerate() {
            let ((row, column), (next_row, next_column)) = (pair[0], pair[1]);
            assert!(
                (next_row, next_column) == (row, column + 1)
                    || (next_row, next_column) == (row + 1, 1),
                "{n} panels: after {i}"
            );
        }
        (
            all.iter().map(|at| at.0).max().unwrap_or(0),
            all.iter().map(|at| at.1).max().unwrap_or(0),
        )
    };
    assert_eq!(shape(1), (1, 1));
    assert_eq!(shape(2), (1, 2));
    assert_eq!(shape(3), (1, 3));
    assert_eq!(shape(4), (2, 2));
    assert_eq!(shape(5), (2, 3));
    assert_eq!(shape(6), (2, 3));
    assert_eq!(shape(7), (3, 3));
    assert_eq!(shape(9), (3, 3));
    assert_eq!(shape(12), (3, 4));
    assert_eq!(shape(13), (4, 4));
    assert_eq!(shape(17), (4, 5));
    assert_eq!(shape(40), (6, 7));
    assert!(rows_and_columns(0).is_empty());
}

#[test]
fn bars_are_stacked_and_set_side_by_side_within_their_panel() {
    // Two bars at week 1 in the north, and one in the south: stacked, the second of the
    // north stands on the first, and nothing of the south is under it.
    let frame = frame(
        "d",
        vec![
            (
                "site",
                Column::strs(["north", "north", "south", "south"].map(Some)),
            ),
            ("week", numbers(&[1.0, 1.0, 1.0, 2.0])),
            ("arm", Column::strs(["a", "b", "a", "b"].map(Some))),
            ("count", numbers(&[4.0, 5.0, 7.0, 9.0])),
        ],
    );
    let coloured = |how: Arrangement, by: &[&str]| {
        let mut plot = parted(frame.clone(), "week", "count", by);
        plot.mappings.push(mapped(Aesthetic::Fill, "arm"));
        with(&plot, Mark::Bars(how))
    };
    let scene = coloured(Arrangement::Stack, &["site"]);
    let (north, south) = (&scene.panels[0].layers[0], &scene.panels[1].layers[0]);
    // The first arm is on top: it starts where the second ends.
    assert_eq!(
        (&north.x, &north.low, &north.y),
        (&vec![1.0, 1.0], &vec![5.0, 0.0], &vec![9.0, 5.0])
    );
    assert_eq!(
        (&south.x, &south.low, &south.y),
        (&vec![1.0, 2.0], &vec![0.0, 0.0], &vec![7.0, 9.0])
    );
    // Without facets the three bars at week 1 are one stack, 16 high.
    let whole = coloured(Arrangement::Stack, &[]);
    assert!(whole.scale("y").unwrap().limits.1 > 16.0);
    assert!(scene.scale("y").unwrap().limits.1 < 10.0);
    // Side by side: the two arms of the north share the width of a bar at week 1, and the
    // one bar of the south has it whole.
    let scene = coloured(Arrangement::Dodge, &["site"]);
    let (north, south) = (&scene.panels[0].layers[0], &scene.panels[1].layers[0]);
    assert_eq!(north.span, [0.45, 0.45]);
    assert!((north.x[0] - 0.775).abs() < 1e-12 && (north.x[1] - 1.225).abs() < 1e-12);
    assert_eq!((&south.x, &south.span), (&vec![1.0, 2.0], &vec![0.9, 0.9]));
    // Bars that count rows count those of their panel.
    let mut plot: Plot = start(
        frame.clone(),
        vec![mapped(Aesthetic::X, "week")],
        Vec::new(),
    );
    plot.facet = Some(PlotFacet {
        by: vec!["site".to_string()],
        ..PlotFacet::default()
    });
    let counted = with(&plot, Mark::Bars(Arrangement::Stack));
    assert_eq!(counted.panels[0].layers[0].y, [2.0]);
    assert_eq!(counted.panels[1].layers[0].y, [1.0, 1.0]);
}

#[test]
fn a_line_and_a_stack_of_areas_stay_in_their_panel() {
    let scene = with(&parted(sites(), "week", "count", &["site"]), Mark::Lines);
    // One line in each panel, of its three rows; no line joins two panels.
    for (panel, rows) in scene.panels.iter().zip([[3, 6, 9], [1, 4, 7], [2, 5, 8]]) {
        let line = &panel.layers[0];
        assert_eq!(line.rows, rows);
        assert_eq!(line.x, [1.0, 2.0, 3.0]);
        assert!(line.line.windows(2).all(|pair| pair[0] == pair[1]));
    }
    let ids: Vec<usize> = scene.panels.iter().map(|p| p.layers[0].line[0]).collect();
    assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);
    // Stacked areas: in each panel the areas of its arms stand on one another, and the
    // heights of another panel are not under them.
    let two = frame(
        "d",
        vec![
            (
                "site",
                Column::strs(["n", "n", "n", "n", "s", "s", "s", "s"].map(Some)),
            ),
            ("week", numbers(&[1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0])),
            (
                "arm",
                Column::strs(["a", "a", "b", "b", "a", "a", "b", "b"].map(Some)),
            ),
            (
                "count",
                numbers(&[1.0, 2.0, 10.0, 20.0, 3.0, 4.0, 30.0, 40.0]),
            ),
        ],
    );
    let mut plot = parted(two.clone(), "week", "count", &["site"]);
    plot.mappings.push(mapped(Aesthetic::Fill, "arm"));
    let scene = with(&plot, Mark::Area(Stacking::Stack));
    let tops = |panel: usize| {
        let mut all = scene.panels[panel].layers[0].y.clone();
        all.sort_by(f64::total_cmp);
        all
    };
    assert_eq!(tops(0), [10.0, 11.0, 20.0, 22.0]);
    assert_eq!(tops(1), [30.0, 33.0, 40.0, 44.0]);
    assert!(scene.scale("y").unwrap().limits.1 < 50.0);
    // The same stacks as each site drawn alone.
    for (panel, site) in ["n", "s"].iter().enumerate() {
        let keep: Vec<usize> = (0..8)
            .filter(|row| (row / 4 == 0) == (*site == "n"))
            .collect();
        let alone = frame(
            "d",
            vec![
                (
                    "week",
                    numbers(&keep.iter().map(|r| [1.0, 2.0][r % 2]).collect::<Vec<f64>>()),
                ),
                (
                    "arm",
                    Column::strs(keep.iter().map(|r| Some(["a", "b"][(r / 2) % 2]))),
                ),
                (
                    "count",
                    numbers(
                        &keep
                            .iter()
                            .map(|r| [1.0, 2.0, 10.0, 20.0, 3.0, 4.0, 30.0, 40.0][*r])
                            .collect::<Vec<f64>>(),
                    ),
                ),
            ],
        );
        let mut plot = parted(alone, "week", "count", &[]);
        plot.mappings.push(mapped(Aesthetic::Fill, "arm"));
        let one = with(&plot, Mark::Area(Stacking::Stack));
        let (here, there) = (&scene.panels[panel].layers[0], &one.panels[0].layers[0]);
        assert_eq!(
            (&here.x, &here.y, &here.low),
            (&there.x, &there.y, &there.low),
            "{site}"
        );
    }
}

#[test]
fn a_layer_whose_frame_lacks_the_variable_is_drawn_in_every_panel() {
    let plot = parted(sites(), "week", "count", &["site"]);
    let (plot, _) = with_layer(&plot, layer(Mark::Points));
    // A line across at one height, and a layer of points from a frame of its own with no
    // `site`.
    let mut level = layer(Mark::Hline);
    level.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Y,
        value: PlotSetting::Place(5.0),
    }];
    let (plot, _) = with_layer(&plot, level);
    let mut targets = layer(Mark::Points);
    targets.data = Some(frame(
        "targets",
        vec![
            ("week", numbers(&[1.0, 3.0])),
            ("count", numbers(&[10.0, 12.0])),
        ],
    ));
    let (_, scene) = with_layer(&plot, targets);
    assert_eq!(scene.panels.len(), 3);
    for panel in &scene.panels {
        assert_eq!(panel.layers.len(), 3);
        assert_eq!(panel.layers[0].x.len(), 3);
        assert_eq!(panel.layers[1].y, [5.0]);
        assert_eq!(
            (&panel.layers[2].x, &panel.layers[2].y),
            (&vec![1.0, 3.0], &vec![10.0, 12.0])
        );
    }
    // A layer whose own frame has the variable is parted by it, and a value only that frame
    // holds is a panel more, after those of the plot.
    let plot = parted(sites(), "week", "count", &["site"]);
    let (plot, _) = with_layer(&plot, layer(Mark::Points));
    let mut more = layer(Mark::Points);
    more.data = Some(frame(
        "more",
        vec![
            ("site", Column::strs(["south", "west"].map(Some))),
            ("week", numbers(&[2.0, 2.0])),
            ("count", numbers(&[11.0, 12.0])),
        ],
    ));
    let (_, scene) = with_layer(&plot, more);
    let strips: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.strip.as_deref())
        .collect();
    assert_eq!(strips, ["east", "north", "south", "west"]);
    let own: Vec<&Vec<f64>> = scene.panels.iter().map(|p| &p.layers[1].y).collect();
    assert_eq!(own, [&vec![], &vec![], &vec![11.0], &vec![12.0]]);
    assert!(scene.panels[3].layers[0].x.is_empty());
}

#[test]
fn an_empty_panel_a_row_without_a_value_and_value_labels() {
    // The west has no row that can be drawn: its panel is there, with no mark. A row with
    // no site has a panel of its own, the last, named `NA`.
    let labelled = Column::ints([Some(1), Some(1), Some(2), Some(2), Some(3)]).with_entry(
        MetadataKey::ValueLabels,
        format_value_labels(&[("1", "Northern region"), ("2", "Southern region")]),
    );
    let frame = frame(
        "d",
        vec![
            (
                "site",
                Column::strs([
                    Some("north"),
                    Some("west"),
                    None,
                    Some("north"),
                    Some("west"),
                ]),
            ),
            ("region", labelled),
            ("week", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0])),
            (
                "count",
                floats(&[Some(4.0), None, Some(6.0), Some(7.0), None]),
            ),
        ],
    );
    let scene = with(
        &parted(frame.clone(), "week", "count", &["site"]),
        Mark::Points,
    );
    let strips: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.strip.as_deref())
        .collect();
    assert_eq!(strips, ["north", "west", "NA"]);
    let marks: Vec<usize> = scene.panels.iter().map(|p| p.layers[0].x.len()).collect();
    assert_eq!(marks, [2, 0, 1]);
    // The rows that are not drawn are counted once, over all the panels.
    assert_eq!(scene.notes, ["2 rows with NA in week or count dropped"]);
    // A variable with value labels is written by them, and a value without one as it
    // prints (decision #118, F8).
    let scene = with(
        &parted(frame.clone(), "week", "count", &["region"]),
        Mark::Points,
    );
    let strips: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.strip.as_deref())
        .collect();
    assert_eq!(strips, ["Northern region", "Southern region", "3"]);
    // Two variables: a panel for each combination that is there, both values written.
    let scene = with(
        &parted(frame, "week", "count", &["region", "site"]),
        Mark::Points,
    );
    let strips: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.strip.as_deref())
        .collect();
    assert_eq!(
        strips,
        [
            "Northern region, north",
            "Northern region, west",
            "Southern region, north",
            "Southern region, NA",
            "3, west"
        ]
    );
    assert_eq!((scene.rows, scene.columns), (2, 3));
}

#[test]
fn one_panel_forty_panels_and_a_variable_the_frame_lacks() {
    // One value: one panel, with its strip.
    let one = frame(
        "d",
        vec![
            ("g", Column::strs(["all", "all"].map(Some))),
            ("x", numbers(&[1.0, 2.0])),
            ("y", numbers(&[3.0, 4.0])),
        ],
    );
    let scene = with(&parted(one.clone(), "x", "y", &["g"]), Mark::Points);
    assert_eq!(scene.panels.len(), 1);
    assert_eq!(scene.panels[0].strip.as_deref(), Some("all"));
    assert_eq!(scene.panels[0].layers[0].x, [1.0, 2.0]);
    // Forty values: forty panels in six rows of seven, the last row of five.
    let names: Vec<String> = (0..40).map(|i| format!("unit {i:02}")).collect();
    let forty = frame(
        "d",
        vec![
            (
                "g",
                Column::strs(names.iter().map(|name| Some(name.as_str()))),
            ),
            ("x", numbers(&(0..40).map(f64::from).collect::<Vec<f64>>())),
            (
                "y",
                numbers(&(0..40).map(|i| f64::from(i % 7)).collect::<Vec<f64>>()),
            ),
        ],
    );
    let scene = with(&parted(forty, "x", "y", &["g"]), Mark::Points);
    assert_eq!(scene.panels.len(), 40);
    assert_eq!((scene.rows, scene.columns), (6, 7));
    assert_eq!((scene.panels[39].row, scene.panels[39].column), (6, 5));
    for (i, panel) in scene.panels.iter().enumerate() {
        assert_eq!(panel.strip.as_deref(), Some(names[i].as_str()));
        assert_eq!(panel.layers[0].x, [i as f64]);
    }
    // A variable the plot's frame does not have parts nothing: the plot is one panel.
    let scene = with(&parted(one, "x", "y", &["nothing"]), Mark::Points);
    assert_eq!(scene.panels.len(), 1);
    assert_eq!(scene.panels[0].strip, None);
}

#[test]
fn the_printed_plot_counts_its_layers_over_the_panels_and_says_how_many_there_are() {
    let plot = parted(sites(), "week", "count", &["site"]);
    let (mut plot, _) = with_layer(&plot, layer(Mark::Lines));
    describe(&mut plot);
    let layers = plot
        .summary
        .more
        .iter()
        .find(|table| table.columns.iter().any(|(name, _)| name == "layer"))
        .expect("a table of layers");
    assert_eq!(layers.rows.len(), 1);
    assert_eq!(layers.rows[0][3], "9");
    assert!(
        layers.rows[0][4].ends_with("3 lines"),
        "{:?}",
        layers.rows[0]
    );
    let last: Vec<&String> = plot
        .summary
        .more
        .iter()
        .flat_map(|table| &table.after)
        .collect();
    assert_eq!(last, ["panels: 3 by site: east, north, south"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows: every row that the plot without facets draws is drawn in exactly
    /// one panel, the one of its category, at the same place; and the scales are the same.
    #[test]
    fn every_row_that_is_drawn_falls_in_exactly_one_panel(rows in rows()) {
        let frame = tried(&rows);
        let whole = with(&parted(frame.clone(), "x", "y", &[]), Mark::Points);
        let scene = with(&parted(frame, "x", "y", &["g"]), Mark::Points);
        prop_assert_eq!(&scene.scales, &whole.scales);
        prop_assert_eq!(&scene.notes, &whole.notes);
        let all = &whole.panels[0].layers[0];
        let mut found = vec![0usize; all.rows.len()];
        for panel in &scene.panels {
            let strip = panel.strip.clone().unwrap_or_default();
            let layer = &panel.layers[0];
            for (i, row) in layer.rows.iter().enumerate() {
                let at = all.rows.iter().position(|known| known == row);
                prop_assert!(at.is_some(), "row {} is in a panel and not in the plot", row);
                let at = at.unwrap();
                found[at] += 1;
                prop_assert_eq!((layer.x[i], layer.y[i]), (all.x[at], all.y[at]));
                // The panel is the one of the row's category.
                let wanted = match rows[row - 1].2 {
                    Some(k) => format!("group <{k:02}> & co"),
                    None => "NA".to_string(),
                };
                prop_assert_eq!(&strip, &wanted);
            }
        }
        prop_assert!(found.iter().all(|times| *times == 1), "{:?}", found);
        // The panels are in the order of the categories, each once.
        let strips: Vec<String> = scene.panels.iter().filter_map(|p| p.strip.clone()).collect();
        let mut sorted = strips.clone();
        sorted.sort_by_key(|strip| (strip == "NA", strip.clone()));
        sorted.dedup();
        prop_assert_eq!(strips, sorted);
    }
}

// ----- the layout of several panels (step 27d2) -----

/// The panels of a drawing: where each starts, with its width and its height.
fn panels_of(drawing: &Drawing) -> Vec<((f64, f64), f64, f64)> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("panel"))
        .filter_map(|item| match &item.primitive {
            Primitive::Clip {
                at, width, height, ..
            } => Some((*at, *width, *height)),
            _ => None,
        })
        .collect()
}

/// The texts of a drawing with one name, each with where it stands and how it is turned.
fn words_of(drawing: &Drawing, name: &str) -> Vec<(String, (f64, f64), f64)> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some(name))
        .filter_map(|item| match &item.primitive {
            Primitive::Text {
                at, turn, spans, ..
            } => spans.first().map(|span| (span.text.clone(), *at, *turn)),
            _ => None,
        })
        .collect()
}

/// A frame of `n` groups, three rows in each.
fn groups(n: usize) -> Frame {
    let names: Vec<String> = (0..3 * n).map(|i| format!("group {:02}", i / 3)).collect();
    frame(
        "d",
        vec![
            (
                "g",
                Column::strs(names.iter().map(|name| Some(name.as_str()))),
            ),
            (
                "x",
                numbers(&(0..3 * n).map(|i| (i % 3) as f64).collect::<Vec<f64>>()),
            ),
            (
                "y",
                numbers(&(0..3 * n).map(|i| (i % 7) as f64).collect::<Vec<f64>>()),
            ),
        ],
    )
}

fn laid(scene: &Scene, size: Size) -> Drawing {
    layout(scene, size, &JOURNAL).unwrap()
}

/// That no two of some panels overlap, and that all are of one size.
fn apart_and_alike(panels: &[((f64, f64), f64, f64)]) {
    for (i, a) in panels.iter().enumerate() {
        assert!((a.1 - panels[0].1).abs() < 1e-9 && (a.2 - panels[0].2).abs() < 1e-9);
        for b in &panels[i + 1..] {
            let beside = a.0.0 + a.1 <= b.0.0 + 1e-9 || b.0.0 + b.1 <= a.0.0 + 1e-9;
            let above = a.0.1 + a.2 <= b.0.1 + 1e-9 || b.0.1 + b.2 <= a.0.1 + 1e-9;
            assert!(beside || above, "{a:?} and {b:?} overlap");
        }
    }
}

#[test]
fn twelve_panels_are_laid_in_three_rows_with_their_strips_and_shared_axes() {
    let scene = with(&parted(groups(12), "x", "y", &["g"]), Mark::Points);
    assert!(scene.wrapped);
    assert!(scene.json().write().contains("\"wrapped\": true"));
    let drawing = laid(&scene, Size::TwoColumn);
    let panels = panels_of(&drawing);
    assert_eq!(panels.len(), 12);
    apart_and_alike(&panels);
    // Three rows of four, from the top left, the rows and the columns each in line.
    for (i, panel) in panels.iter().enumerate() {
        let (row, column) = (i / 4, i % 4);
        assert!((panel.0.0 - panels[column].0.0).abs() < 1e-9, "panel {i}");
        assert!((panel.0.1 - panels[4 * row].0.1).abs() < 1e-9, "panel {i}");
    }
    assert!(panels[1].0.0 > panels[0].0.0 + panels[0].1);
    assert!(panels[4].0.1 > panels[0].0.1 + panels[0].2);
    // Over each panel, what it holds, centred on it and above its top.
    let strips = words_of(&drawing, "strip");
    assert_eq!(strips.len(), 12);
    for (i, (words, at, turn)) in strips.iter().enumerate() {
        assert_eq!(
            (words.as_str(), *turn),
            (format!("group {i:02}").as_str(), 0.0)
        );
        assert!((at.0 - (panels[i].0.0 + panels[i].1 / 2.0)).abs() < 1e-9);
        assert!(at.1 < panels[i].0.1 && at.1 > panels[i].0.1 - 12.0);
    }
    // The labels of `y` beside the first panel of each row, and no other; those of `x`
    // under the panels of the last row, and no other; each title once, in the middle of the
    // grid.
    let (left, right) = (panels[0].0.0, panels[3].0.0 + panels[3].1);
    let (top, foot) = (panels[0].0.1, panels[8].0.1 + panels[8].2);
    let up = words_of(&drawing, "axis:y");
    let title: Vec<&(String, (f64, f64), f64)> = up.iter().filter(|w| w.0 == "y").collect();
    assert_eq!(title.len(), 1);
    assert_eq!(title[0].2, 90.0);
    // The strip over the first row is part of the grid.
    assert!((title[0].1.1 - (top + foot) / 2.0).abs() < 8.0);
    let labels: Vec<&(String, (f64, f64), f64)> = up.iter().filter(|w| w.0 != "y").collect();
    assert!(labels.iter().all(|w| w.1.0 < left));
    assert_eq!(labels.len() % 3, 0);
    assert!(labels.len() >= 9);
    let along = words_of(&drawing, "axis:x");
    let title: Vec<&(String, (f64, f64), f64)> = along.iter().filter(|w| w.0 == "x").collect();
    assert_eq!(title.len(), 1);
    assert!((title[0].1.0 - (left + right) / 2.0).abs() < 1e-9);
    let labels: Vec<&(String, (f64, f64), f64)> = along.iter().filter(|w| w.0 != "x").collect();
    assert!(labels.iter().all(|w| w.1.1 > foot));
    assert_eq!(labels.len() % 4, 0);
    // The marks of each panel are in it.
    for (i, item) in drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("panel"))
        .enumerate()
    {
        let Primitive::Clip { items, .. } = &item.primitive else {
            panic!("a panel that is not a clip");
        };
        let dots: Vec<(f64, f64)> = items
            .iter()
            .filter_map(|item| match &item.primitive {
                Primitive::Point { at, .. } => Some(*at),
                _ => None,
            })
            .collect();
        assert_eq!(dots.len(), 3);
        for dot in dots {
            assert!(dot.0 >= panels[i].0.0 && dot.0 <= panels[i].0.0 + panels[i].1);
            assert!(dot.1 >= panels[i].0.1 && dot.1 <= panels[i].0.1 + panels[i].2);
        }
    }
    insta::assert_snapshot!("twelve_panels", drawing.describe());
}

#[test]
fn a_last_row_that_is_not_full_and_a_plot_of_one_panel() {
    // Five panels: three and two. The labels of `x` are under the lowest panel of each
    // column: the two of the second row, and the third of the first.
    let scene = with(&parted(groups(5), "x", "y", &["g"]), Mark::Points);
    let drawing = laid(&scene, Size::TwoColumn);
    let panels = panels_of(&drawing);
    assert_eq!(panels.len(), 5);
    apart_and_alike(&panels);
    assert!((panels[3].0.0 - panels[0].0.0).abs() < 1e-9 && panels[3].0.1 > panels[0].0.1);
    let labels: Vec<(f64, f64)> = words_of(&drawing, "axis:x")
        .into_iter()
        .filter(|w| w.0 != "x")
        .map(|w| w.1)
        .collect();
    let under = |panel: usize| {
        labels
            .iter()
            .filter(|at| {
                at.0 >= panels[panel].0.0 - 6.0
                    && at.0 <= panels[panel].0.0 + panels[panel].1 + 6.0
                    && at.1 > panels[panel].0.1 + panels[panel].2
                    && at.1 < panels[panel].0.1 + panels[panel].2 + 16.0
            })
            .count()
    };
    assert_eq!((under(0), under(1)), (0, 0));
    assert!(under(2) >= 2 && under(3) >= 2 && under(4) >= 2);
    assert_eq!(under(2), under(3));
    // One panel: its strip over it, and a panel lower by the room of the strip than that of
    // the plot without facets.
    let one = frame(
        "d",
        vec![
            ("g", Column::strs(["all", "all"].map(Some))),
            ("x", numbers(&[1.0, 2.0])),
            ("y", numbers(&[3.0, 4.0])),
        ],
    );
    let whole = laid(
        &with(&parted(one.clone(), "x", "y", &[]), Mark::Points),
        Size::OneColumn,
    );
    let alone = laid(
        &with(&parted(one, "x", "y", &["g"]), Mark::Points),
        Size::OneColumn,
    );
    let (a, b) = (panels_of(&whole)[0], panels_of(&alone)[0]);
    assert!((a.0.0 - b.0.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9);
    assert!(b.0.1 > a.0.1 && (a.0.1 + a.2 - (b.0.1 + b.2)).abs() < 1e-9);
    assert_eq!(words_of(&alone, "strip")[0].0, "all");
    assert!(words_of(&whole, "strip").is_empty());
}

#[test]
fn forty_panels_are_drawn_on_two_columns_and_refused_on_one() {
    let scene = with(&parted(groups(40), "x", "y", &["g"]), Mark::Points);
    assert_eq!((scene.rows, scene.columns), (6, 7));
    // Six rows of seven would leave each panel too low: the panels are laid in the nearest
    // number of columns at which each has the room a panel needs, four rows of ten.
    let drawing = laid(&scene, Size::TwoColumn);
    let panels = panels_of(&drawing);
    assert_eq!(panels.len(), 40);
    apart_and_alike(&panels);
    assert!(panels[0].1 >= SMALLEST_PANEL && panels[0].2 >= SMALLEST_PANEL);
    let in_a_row = panels
        .iter()
        .filter(|panel| (panel.0.1 - panels[0].0.1).abs() < 1e-9)
        .count();
    assert_eq!(in_a_row, 10);
    assert_eq!(words_of(&drawing, "strip").len(), 40);
    // Every panel is on the page.
    for panel in &panels {
        assert!(panel.0.0 > 0.0 && panel.0.0 + panel.1 < drawing.width);
        assert!(panel.0.1 > 0.0 && panel.0.1 + panel.2 < drawing.height);
    }
    // On one column no grid gives forty panels their room.
    match layout(&scene, Size::OneColumn, &JOURNAL) {
        Err(LayoutError::TooMany { panels, panel }) => {
            assert_eq!(panels, 40);
            assert!(panel.0 < SMALLEST_PANEL || panel.1 < SMALLEST_PANEL);
        }
        other => panic!("forty panels on one column: {other:?}"),
    }
}

#[test]
fn flipped_fixed_and_polar_panels_and_the_legend_beside_them() {
    let frame = groups(4);
    let drawn_in = |coord: Coord| {
        let mut plot = parted(frame.clone(), "x", "y", &["g"]);
        plot.coord = coord;
        plot.mappings.push(mapped(Aesthetic::Color, "g"));
        plot.legend = LegendPlace::Direct;
        laid(&with(&plot, Mark::Lines), Size::TwoColumn)
    };
    for coord in [
        Coord::Cartesian,
        Coord::Flip,
        Coord::Fixed(1.0),
        Coord::Polar,
    ] {
        let drawing = drawn_in(coord);
        let panels = panels_of(&drawing);
        assert_eq!(panels.len(), 4, "{coord:?}");
        apart_and_alike(&panels);
        // The legend stands at the right of the grid: names at the ends of lines are for a
        // plot of one panel.
        let right = panels.iter().map(|p| p.0.0 + p.1).fold(0.0, f64::max);
        let keys = words_of(&drawing, "legend:color");
        assert!(keys.len() >= 4, "{coord:?}");
        assert!(keys.iter().all(|key| key.1.0 > right), "{coord:?}");
    }
    // Fixed panels stand together, a gap apart, in the middle of the room: the grid of the
    // panels is as far from the left of the room as from its right.
    let fixed = panels_of(&drawn_in(Coord::Fixed(1.0)));
    let plain = panels_of(&drawn_in(Coord::Cartesian));
    // `x` runs over 2.2 units and `y` over 6.6: a panel three times as high as it is wide.
    assert!((fixed[0].2 / fixed[0].1 - 3.0).abs() < 1e-9);
    let gap = fixed[1].0.0 - (fixed[0].0.0 + fixed[0].1);
    assert!(gap > 0.0 && gap < 30.0, "{gap}");
    let (room_left, room_right) = (plain[0].0.0, plain[1].0.0 + plain[1].1);
    let (grid_left, grid_right) = (fixed[0].0.0, fixed[1].0.0 + fixed[1].1);
    assert!(((grid_left - room_left) - (room_right - grid_right)).abs() < 1e-6);
    // Each polar panel has its own labels round its rim.
    let polar = drawn_in(Coord::Polar);
    let rims = words_of(&polar, "axis:x")
        .into_iter()
        .filter(|w| w.0 != "x")
        .count();
    assert_eq!(rims % 4, 0);
    assert!(rims >= 8);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// Whatever the rows: the panels of the drawing are as many as those of the scene, all
    /// of one size, no two overlapping, each on the page, with as many points in each as
    /// its panel of the scene holds; or the page was too small for them.
    #[test]
    fn no_two_panels_overlap_and_each_is_as_large_as_the_others(rows in rows()) {
        let scene = with(&parted(tried(&rows), "x", "y", &["g"]), Mark::Points);
        match layout(&scene, Size::TwoColumn, &JOURNAL) {
            Ok(drawing) => {
                let panels = panels_of(&drawing);
                prop_assert_eq!(panels.len(), scene.panels.len());
                for (i, a) in panels.iter().enumerate() {
                    prop_assert!((a.1 - panels[0].1).abs() < 1e-9);
                    prop_assert!((a.2 - panels[0].2).abs() < 1e-9);
                    prop_assert!(a.0.0 >= 0.0 && a.0.0 + a.1 <= drawing.width);
                    prop_assert!(a.0.1 >= 0.0 && a.0.1 + a.2 <= drawing.height);
                    for b in &panels[i + 1..] {
                        let beside = a.0.0 + a.1 <= b.0.0 + 1e-9 || b.0.0 + b.1 <= a.0.0 + 1e-9;
                        let above = a.0.1 + a.2 <= b.0.1 + 1e-9 || b.0.1 + b.2 <= a.0.1 + 1e-9;
                        prop_assert!(beside || above);
                    }
                }
                let dots: Vec<usize> = drawing
                    .items
                    .iter()
                    .filter(|item| item.name.as_deref() == Some("panel"))
                    .map(|item| match &item.primitive {
                        Primitive::Clip { items, .. } => items
                            .iter()
                            .filter(|item| matches!(item.primitive, Primitive::Point { .. }))
                            .count(),
                        _ => 0,
                    })
                    .collect();
                let held: Vec<usize> = scene.panels.iter().map(|p| p.layers[0].x.len()).collect();
                prop_assert_eq!(dots, held);
            }
            Err(LayoutError::TooSmall { .. } | LayoutError::TooMany { .. }) => {}
            Err(other) => prop_assert!(false, "{:?}", other),
        }
    }
}

// ----- a grid of panels (step 27e1) -----

/// A plot of `x` and `y` of a frame, parted into a grid.
fn gridded(frame: Frame, x: &str, y: &str, rows: Option<&str>, cols: Option<&str>) -> Plot {
    let mut plot = parted(frame, x, y, &[]);
    plot.facet = Some(PlotFacet {
        by: Vec::new(),
        rows: rows.map(str::to_string),
        cols: cols.map(str::to_string),
        ..PlotFacet::default()
    });
    plot
}

#[test]
fn a_grid_has_a_row_for_each_value_of_one_variable_and_a_column_for_each_of_another() {
    // Three sites down the page and two arms across it. No row of the east is of arm `b`
    // in week 3, but every site has both arms: six panels.
    let scene = with(
        &gridded(sites(), "week", "count", Some("site"), Some("arm")),
        Mark::Points,
    );
    assert!(!scene.wrapped);
    assert_eq!((scene.rows, scene.columns), (3, 2));
    let places: Vec<(usize, usize)> = scene.panels.iter().map(|p| (p.row, p.column)).collect();
    assert_eq!(places, [(1, 1), (1, 2), (2, 1), (2, 2), (3, 1), (3, 2)]);
    // The arms are written over the first row, and the sites beside the last column.
    let strips: Vec<Option<&str>> = scene.panels.iter().map(|p| p.strip.as_deref()).collect();
    assert_eq!(strips, [Some("a"), Some("b"), None, None, None, None]);
    let sides: Vec<Option<&str>> = scene.panels.iter().map(|p| p.side.as_deref()).collect();
    assert_eq!(
        sides,
        [None, Some("east"), None, Some("north"), None, Some("south")]
    );
    // Each panel holds the rows of its site and its arm.
    let rows: Vec<&Vec<usize>> = scene.panels.iter().map(|p| &p.layers[0].rows).collect();
    assert_eq!(
        rows,
        [
            &vec![3, 9],
            &vec![6],
            &vec![1, 7],
            &vec![4],
            &vec![2, 8],
            &vec![5]
        ]
    );
    // The scales are those of the plot without facets.
    let whole = with(&parted(sites(), "week", "count", &[]), Mark::Points);
    assert_eq!(&scene.scales, &whole.scales);
    let json = scene.json().write();
    assert!(
        json.contains("\"strips\": [\"a\", \"b\", \"\", \"\", \"\", \"\"]"),
        "{json}"
    );
    assert!(json.contains("\"sides\": [\"\", \"east\", \"\", \"north\", \"\", \"south\"]"));
    assert!(!json.contains("\"wrapped\""));
    insta::assert_snapshot!("three_sites_by_two_arms", json);
}

#[test]
fn a_cell_with_no_row_one_variable_alone_and_a_row_without_a_value() {
    // The west has arm `a` alone, and one row has no site: a grid of three rows by two
    // columns, of which two cells hold nothing.
    let frame = frame(
        "d",
        vec![
            (
                "site",
                Column::strs([Some("north"), Some("north"), Some("west"), None]),
            ),
            ("arm", Column::strs(["a", "b", "a", "b"].map(Some))),
            ("week", numbers(&[1.0, 2.0, 3.0, 4.0])),
            ("count", numbers(&[4.0, 5.0, 6.0, 7.0])),
        ],
    );
    let scene = with(
        &gridded(frame.clone(), "week", "count", Some("site"), Some("arm")),
        Mark::Points,
    );
    assert_eq!((scene.rows, scene.columns), (3, 2));
    let marks: Vec<usize> = scene.panels.iter().map(|p| p.layers[0].x.len()).collect();
    assert_eq!(marks, [1, 1, 1, 0, 0, 1]);
    let sides: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.side.as_deref())
        .collect();
    assert_eq!(sides, ["north", "west", "NA"]);
    // Rows alone: one column, the value of each row beside it and nothing over it.
    let scene = with(
        &gridded(frame.clone(), "week", "count", Some("arm"), None),
        Mark::Points,
    );
    assert_eq!((scene.rows, scene.columns), (2, 1));
    assert!(scene.panels.iter().all(|p| p.strip.is_none()));
    let sides: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.side.as_deref())
        .collect();
    assert_eq!(sides, ["a", "b"]);
    // Columns alone: one row, the value of each column over it and nothing beside it.
    let scene = with(
        &gridded(frame.clone(), "week", "count", None, Some("arm")),
        Mark::Points,
    );
    assert_eq!((scene.rows, scene.columns), (1, 2));
    assert!(scene.panels.iter().all(|p| p.side.is_none()));
    let strips: Vec<&str> = scene
        .panels
        .iter()
        .filter_map(|p| p.strip.as_deref())
        .collect();
    assert_eq!(strips, ["a", "b"]);
    // A layer that has not both variables is drawn in every panel; lines stay in their cell.
    let plot = gridded(frame.clone(), "week", "count", Some("site"), Some("arm"));
    let (plot, _) = with_layer(&plot, layer(Mark::Lines));
    let mut level = layer(Mark::Hline);
    level.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Y,
        value: PlotSetting::Place(5.0),
    }];
    let (_, scene) = with_layer(&plot, level);
    assert!(scene.panels.iter().all(|p| p.layers[1].y == [5.0]));
    assert!(scene.panels.iter().all(|p| p.layers[0].x.is_empty()));
    // A variable the frame lacks parts nothing.
    let scene = with(
        &gridded(frame, "week", "count", Some("nothing"), Some("arm")),
        Mark::Points,
    );
    assert_eq!(scene.panels.len(), 1);
    assert_eq!(
        (
            scene.panels[0].strip.as_ref(),
            scene.panels[0].side.as_ref()
        ),
        (None, None)
    );
}

#[test]
fn a_grid_is_laid_as_it_is_with_its_columns_named_above_and_its_rows_at_the_right() {
    let scene = with(
        &gridded(sites(), "week", "count", Some("site"), Some("arm")),
        Mark::Points,
    );
    let drawing = laid(&scene, Size::TwoColumn);
    let panels = panels_of(&drawing);
    assert_eq!(panels.len(), 6);
    apart_and_alike(&panels);
    // Three rows of two, as the scene has them.
    for (i, panel) in panels.iter().enumerate() {
        assert!((panel.0.0 - panels[i % 2].0.0).abs() < 1e-9);
        assert!((panel.0.1 - panels[2 * (i / 2)].0.1).abs() < 1e-9);
    }
    // The arms over the two panels of the first row, upright; the sites at the right of the
    // second column, each at the middle of its row and read downwards.
    let strips = words_of(&drawing, "strip");
    assert_eq!(strips.len(), 5);
    for (words, at, turn) in &strips[..2] {
        let panel = panels[usize::from(words == "b")];
        assert_eq!(*turn, 0.0);
        assert!((at.0 - (panel.0.0 + panel.1 / 2.0)).abs() < 1e-9 && at.1 < panel.0.1);
    }
    let right = panels[1].0.0 + panels[1].1;
    for (i, name) in ["east", "north", "south"].iter().enumerate() {
        let (words, at, turn) = strips
            .iter()
            .find(|strip| strip.0 == *name)
            .unwrap_or_else(|| panic!("`{name}` is not written"));
        let panel = panels[2 * i + 1];
        assert_eq!((words.as_str(), *turn), (*name, -90.0));
        assert!(at.0 > right && at.0 < right + 8.0);
        assert!((at.1 - (panel.0.1 + panel.2 / 2.0)).abs() < 1e-9);
    }
    // The rows are closer than those of panels that each have their strip: the columns are
    // named once.
    let gap = panels[2].0.1 - (panels[0].0.1 + panels[0].2);
    assert!((gap - 12.0).abs() < 1e-9, "{gap}");
    // A grid is never laid otherwise: on a page too small for it, it is refused.
    assert!(matches!(
        layout(&scene, Size::Millimetres(60.0, 60.0), &JOURNAL),
        Err(LayoutError::TooMany { panels: 6, .. })
    ));
    insta::assert_snapshot!("a_grid_of_six", drawing.describe());
}

// ----- panels with axes of their own (step 27e2) -----

/// A plot parted by variables, with the axes its panels share.
fn freed(frame: Frame, x: &str, y: &str, by: &[&str], scales: FacetScales) -> Plot {
    let mut plot = parted(frame, x, y, by);
    if let Some(facet) = plot.facet.as_mut() {
        facet.scales = scales;
    }
    plot
}

#[test]
fn each_panel_has_the_axes_of_its_own_marks_where_the_scales_are_free() {
    // Three sites: the counts of the east run from 1 to 3, of the north from 4 to 6 and of
    // the south from 7 to 9. Every site has the weeks 1 to 3.
    let fixed = with(
        &freed(sites(), "week", "count", &["site"], FacetScales::Fixed),
        Mark::Points,
    );
    assert!(
        fixed
            .panels
            .iter()
            .all(|p| p.own_x.is_none() && p.own_y.is_none())
    );
    assert!(!fixed.json().write().contains("own_scales"));
    let free_y = with(
        &freed(sites(), "week", "count", &["site"], FacetScales::FreeY),
        Mark::Points,
    );
    // The scales of the plot, and the marks, are those of the plot with fixed scales.
    assert_eq!(&free_y.scales, &fixed.scales);
    for (free, plain) in free_y.panels.iter().zip(&fixed.panels) {
        assert_eq!(&free.layers, &plain.layers);
        assert!(free.own_x.is_none());
    }
    // Each panel has a scale of `y` of its own: that of the plot of its rows alone.
    let limits: Vec<(f64, f64)> = free_y
        .panels
        .iter()
        .map(|p| p.own_y.as_ref().unwrap().limits)
        .collect();
    for (found, wanted) in limits.iter().zip([(1.0, 3.0), (4.0, 6.0), (7.0, 9.0)]) {
        assert!(
            (found.0 - (wanted.0 - 0.1)).abs() < 1e-9 && (found.1 - (wanted.1 + 0.1)).abs() < 1e-9
        );
    }
    for (panel, site) in free_y.panels.iter().zip(["east", "north", "south"]) {
        let keep: Vec<usize> = (0..9)
            .filter(|row| ["north", "south", "east"][row % 3] == site)
            .collect();
        let alone = frame(
            "d",
            vec![
                (
                    "week",
                    numbers(
                        &keep
                            .iter()
                            .map(|r| (r / 3 + 1) as f64)
                            .collect::<Vec<f64>>(),
                    ),
                ),
                (
                    "count",
                    numbers(
                        &keep
                            .iter()
                            .map(|r| [4.0, 7.0, 2.0, 5.0, 9.0, 3.0, 6.0, 8.0, 1.0][*r])
                            .collect::<Vec<f64>>(),
                    ),
                ),
            ],
        );
        let one = with(&parted(alone, "week", "count", &[]), Mark::Points);
        let (own, wanted) = (panel.own_y.as_ref().unwrap(), one.scale("y").unwrap());
        assert_eq!(
            (own.limits, &own.breaks, &own.labels),
            (wanted.limits, &wanted.breaks, &wanted.labels)
        );
        assert_eq!(
            (own.id, own.kind, own.title.as_str()),
            ("y", wanted.kind, "count")
        );
    }
    // `free_x` frees `x` alone, and `free` both.
    let free_x = with(
        &freed(sites(), "week", "count", &["site"], FacetScales::FreeX),
        Mark::Points,
    );
    assert!(
        free_x
            .panels
            .iter()
            .all(|p| p.own_x.is_some() && p.own_y.is_none())
    );
    let free = with(
        &freed(sites(), "week", "count", &["site"], FacetScales::Free),
        Mark::Points,
    );
    assert!(
        free.panels
            .iter()
            .all(|p| p.own_x.is_some() && p.own_y.is_some())
    );
    let json = free.json().write();
    assert!(json.contains("\"own_scales\""), "{json}");
    insta::assert_snapshot!("three_sites_free", json);
}

#[test]
fn an_axis_of_categories_bars_and_an_empty_panel_with_free_scales() {
    // An axis of categories is the plot's in every panel, with a place for every category.
    let scene = with(
        &freed(sites(), "arm", "count", &["site"], FacetScales::Free),
        Mark::Points,
    );
    assert!(
        scene
            .panels
            .iter()
            .all(|p| p.own_x.is_none() && p.own_y.is_some())
    );
    // Bars start at zero in every panel: each axis runs from under zero to over its own
    // highest bar.
    let scene = with(
        &freed(sites(), "week", "count", &["site"], FacetScales::FreeY),
        Mark::Bars(Arrangement::Stack),
    );
    for (panel, top) in scene.panels.iter().zip([3.0, 6.0, 9.0]) {
        let limits = panel.own_y.as_ref().unwrap().limits;
        assert!(limits.0 < 0.0 && limits.1 > top && limits.1 < top * 1.1);
    }
    // A panel with no mark has the axis of the whole variable.
    let frame = frame(
        "d",
        vec![
            ("g", Column::strs(["a", "a", "b"].map(Some))),
            ("x", numbers(&[1.0, 2.0, 3.0])),
            ("y", floats(&[Some(10.0), Some(20.0), None])),
        ],
    );
    let scene = with(
        &freed(frame, "x", "y", &["g"], FacetScales::FreeY),
        Mark::Points,
    );
    assert_eq!(scene.panels[1].layers[0].x.len(), 0);
    assert_eq!(
        scene.panels[1].own_y.as_ref().unwrap().limits,
        scene.scale("y").unwrap().limits
    );
}

#[test]
fn in_a_grid_each_column_has_its_x_and_each_row_its_y() {
    let mut plot = gridded(sites(), "week", "count", Some("site"), Some("arm"));
    if let Some(facet) = plot.facet.as_mut() {
        facet.scales = FacetScales::Free;
    }
    let scene = with(&plot, Mark::Points);
    // The panels of a row have one scale of `y`, and those of a column one scale of `x`.
    for row in 0..3 {
        assert_eq!(scene.panels[2 * row].own_y, scene.panels[2 * row + 1].own_y);
        assert!(scene.panels[2 * row].own_y.is_some());
    }
    for column in 0..2 {
        assert_eq!(scene.panels[column].own_x, scene.panels[column + 2].own_x);
        assert_eq!(scene.panels[column].own_x, scene.panels[column + 4].own_x);
    }
    // Arm `a` has the weeks 1 and 3, and arm `b` week 2 alone: two scales of `x`.
    assert_ne!(scene.panels[0].own_x, scene.panels[1].own_x);
    assert_ne!(scene.panels[0].own_y, scene.panels[2].own_y);
}

#[test]
fn the_labels_of_a_free_axis_are_written_wherever_it_differs_from_its_neighbours() {
    let labels_in = |scales: FacetScales| {
        let drawing = laid(
            &with(&freed(groups(6), "x", "y", &["g"], scales), Mark::Points),
            Size::TwoColumn,
        );
        let panels = panels_of(&drawing);
        let side = words_of(&drawing, "axis:y")
            .into_iter()
            .filter(|w| w.0 != "y")
            .count();
        let foot = words_of(&drawing, "axis:x")
            .into_iter()
            .filter(|w| w.0 != "x")
            .count();
        (panels, side, foot)
    };
    let (fixed, side, foot) = labels_in(FacetScales::Fixed);
    apart_and_alike(&fixed);
    // Six groups, each with the rows 0, 1 and 2 of `x` and other values of `y`: with `y`
    // free, every panel has its labels up its side, and the panels stand further apart for
    // them; the labels of `x`, which every panel shares, stay under the last row.
    let (free_y, side_free, foot_free) = labels_in(FacetScales::FreeY);
    apart_and_alike(&free_y);
    assert!(side_free > side, "{side_free} against {side}");
    assert_eq!(foot_free, foot);
    let gap = |panels: &[((f64, f64), f64, f64)]| panels[1].0.0 - (panels[0].0.0 + panels[0].1);
    assert!(gap(&free_y) > gap(&fixed) + 4.0);
    // With `x` free too, each panel would have its labels under it; here every panel has
    // the same three values of `x`, so its axis is its neighbour's and is labelled once.
    let (free, _, foot_all) = labels_in(FacetScales::Free);
    apart_and_alike(&free);
    assert_eq!(foot_all, foot);
    // Flipped, the free `y` is along the foot: labelled under every panel.
    let mut plot = freed(groups(6), "x", "y", &["g"], FacetScales::FreeY);
    plot.coord = Coord::Flip;
    let flipped = laid(&with(&plot, Mark::Points), Size::TwoColumn);
    let along = words_of(&flipped, "axis:y")
        .into_iter()
        .filter(|w| w.0 != "y")
        .count();
    assert!(along > foot, "{along}");
    insta::assert_snapshot!(
        "six_panels_free_y",
        laid(
            &with(
                &freed(groups(6), "x", "y", &["g"], FacetScales::FreeY),
                Mark::Points
            ),
            Size::TwoColumn
        )
        .describe()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// Whatever the rows: with free scales every point of a panel stands inside the limits
    /// of the scales of its panel, and is drawn inside the panel.
    #[test]
    fn a_point_stands_within_the_axes_of_its_own_panel(rows in rows()) {
        let scene = with(&freed(tried(&rows), "x", "y", &["g"], FacetScales::Free), Mark::Points);
        for panel in &scene.panels {
            let layer = &panel.layers[0];
            for (own, values) in [(&panel.own_x, &layer.x), (&panel.own_y, &layer.y)] {
                if let Some(own) = own {
                    for value in values {
                        prop_assert!(*value >= own.limits.0 && *value <= own.limits.1);
                    }
                }
            }
        }
        if let Ok(drawing) = layout(&scene, Size::TwoColumn, &JOURNAL) {
            let panels = panels_of(&drawing);
            for (i, item) in drawing
                .items
                .iter()
                .filter(|item| item.name.as_deref() == Some("panel"))
                .enumerate()
            {
                let Primitive::Clip { items, .. } = &item.primitive else {
                    continue;
                };
                for dot in items {
                    if let Primitive::Point { at, .. } = &dot.primitive {
                        prop_assert!(at.0 >= panels[i].0.0 - 1e-9 && at.0 <= panels[i].0.0 + panels[i].1 + 1e-9);
                        prop_assert!(at.1 >= panels[i].0.1 - 1e-9 && at.1 <= panels[i].0.1 + panels[i].2 + 1e-9);
                    }
                }
            }
        }
    }
}
