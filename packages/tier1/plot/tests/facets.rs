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
    Aesthetic, Arrangement, Frame, Mark, Plot, PlotConstant, PlotFacet, PlotLayer, PlotSetting,
    Stacking,
};
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
    assert_eq!(last, ["panels: 3 by site"]);
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
