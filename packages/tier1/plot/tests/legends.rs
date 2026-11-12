//! Where the legends of a plot stand (Language Reference, section 7.6, "Guides and theme":
//! `legend`; release 0.7.0, step 25e1): at the right of the panel, above it, under it, or not
//! drawn. The scene carries the place the script says and the layout gives the legends their
//! room: keys in rows and the bar of a ramp lying down above and under the panel.

mod common;

use common::*;
use zeff_columns::{Column, MetadataKey};
use zeff_core::value::{Aesthetic, Frame, LegendPlace, Mark, Plot, PlotLayer};
use zeff_plot::drawing::{Drawing, Item, Primitive};
use zeff_plot::layout::{JOURNAL, LayoutError, Size, layout};
use zeff_plot::scene::{Scene, start, with_layer};

/// `count` rows in `groups` categories, with a level for a continuous scale.
fn rows(count: usize, groups: usize) -> Frame {
    let names: Vec<String> = (0..count)
        .map(|i| format!("group {:02}", i % groups))
        .collect();
    frame(
        "d",
        vec![
            (
                "x",
                numbers(&(0..count).map(|i| i as f64).collect::<Vec<f64>>()),
            ),
            (
                "y",
                numbers(
                    &(0..count)
                        .map(|i| ((i * 7) % 11) as f64)
                        .collect::<Vec<f64>>(),
                ),
            ),
            (
                "g",
                Column::strs(names.iter().map(|name| Some(name.as_str()))),
            ),
            (
                "z",
                numbers(&(0..count).map(|i| 10.0 * i as f64).collect::<Vec<f64>>()),
            ),
        ],
    )
}

/// The rows shown by the aesthetics named, with the legends at `place`.
fn placed(frame: Frame, shown: &[(Aesthetic, &str)], place: LegendPlace) -> Scene {
    let mut mappings = vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")];
    mappings.extend(
        shown
            .iter()
            .map(|(aesthetic, name)| mapped(*aesthetic, name)),
    );
    let mut plot: Plot = start(frame, mappings, Vec::new());
    plot.legend = place;
    with_layer(&plot, points(1.0)).1
}

/// The items of a drawing that belong to a legend.
fn of_legends(drawing: &Drawing) -> Vec<&Item> {
    drawing
        .items
        .iter()
        .filter(|item| {
            item.name
                .as_deref()
                .is_some_and(|name| name.starts_with("legend:"))
        })
        .collect()
}

/// Where an item stands: its x and y, the top left of a picture.
fn at(item: &Item) -> (f64, f64) {
    match &item.primitive {
        Primitive::Point { at, .. } | Primitive::Text { at, .. } | Primitive::Raster { at, .. } => {
            *at
        }
        Primitive::Segment { from, .. } => *from,
        other => panic!("an item of a legend that is none of the four: {other:?}"),
    }
}

fn drawn(scene: &Scene) -> Drawing {
    layout(scene, Size::OneColumn, &JOURNAL).unwrap()
}

#[test]
fn the_scene_carries_the_place_the_script_says() {
    let by_g = &[(Aesthetic::Color, "g")];
    let right = placed(rows(6, 3), by_g, LegendPlace::Right);
    assert_eq!(right.legend, LegendPlace::Right);
    // At the right is not written: a scene made before `legend` existed reads the same.
    assert!(!right.json().write().contains("\"legend\""));
    for (place, name) in [
        (LegendPlace::Top, "top"),
        (LegendPlace::Bottom, "bottom"),
        (LegendPlace::None, "none"),
    ] {
        let scene = placed(rows(6, 3), by_g, place);
        assert_eq!(scene.legend, place);
        assert!(
            scene
                .json()
                .write()
                .contains(&format!("\"legend\": \"{name}\""))
        );
        // What the legends show is as it was: the place is all that differs.
        assert_eq!(scene.legends, right.legends);
        assert_eq!(scene.scales, right.scales);
    }
}

#[test]
fn a_legend_stands_at_the_right_above_or_under_the_panel() {
    let by_g = &[(Aesthetic::Color, "g")];
    let right = drawn(&placed(rows(6, 3), by_g, LegendPlace::Right));
    let (panel_r, width_r, height_r, _) = panel_of(&right);
    let keys = of_legends(&right);
    assert!(!keys.is_empty());
    // At the right: every item beyond the right edge of the panel.
    assert!(keys.iter().all(|item| at(item).0 > panel_r.0 + width_r));

    let top = drawn(&placed(rows(6, 3), by_g, LegendPlace::Top));
    let (panel_t, width_t, height_t, _) = panel_of(&top);
    let keys = of_legends(&top);
    assert!(!keys.is_empty());
    // Above: every item over the top edge of the panel and not left of its left edge; the
    // panel is as wide as the page allows, and lower than it was.
    assert!(
        keys.iter()
            .all(|item| at(item).1 < panel_t.1 && at(item).0 >= panel_t.0 - 1e-9)
    );
    assert!(width_t > width_r + 20.0);
    assert!(height_t < height_r);
    assert!(panel_t.1 > panel_r.1);

    let bottom = drawn(&placed(rows(6, 3), by_g, LegendPlace::Bottom));
    let (panel_b, width_b, height_b, _) = panel_of(&bottom);
    let keys = of_legends(&bottom);
    assert!(!keys.is_empty());
    // Under: every item below the panel and the title of its axis.
    assert!(
        keys.iter()
            .all(|item| at(item).1 > panel_b.1 + height_b + 20.0)
    );
    assert_eq!(width_b, width_t);
    assert!((height_b - height_t).abs() < 1e-9);
    assert_eq!(panel_b.1, panel_r.1);
    // The three keys stand in one row: their points on one line.
    let points: Vec<(f64, f64)> = keys
        .iter()
        .filter(|item| matches!(item.primitive, Primitive::Point { .. }))
        .map(|item| at(item))
        .collect();
    assert_eq!(points.len(), 3);
    assert!(
        points
            .windows(2)
            .all(|pair| pair[0].1 == pair[1].1 && pair[0].0 < pair[1].0)
    );
    insta::assert_snapshot!("three_keys_above", top.describe());
    insta::assert_snapshot!("three_keys_under", bottom.describe());
}

#[test]
fn no_legend_gives_its_room_to_the_panel() {
    let by_g = &[(Aesthetic::Color, "g")];
    let none = drawn(&placed(rows(6, 3), by_g, LegendPlace::None));
    assert!(of_legends(&none).is_empty());
    let bare = drawn(&placed(rows(6, 3), &[], LegendPlace::Right));
    let (panel_n, width_n, height_n, marks) = panel_of(&none);
    let (panel_0, width_0, height_0, _) = panel_of(&bare);
    assert_eq!((panel_n, width_n, height_n), (panel_0, width_0, height_0));
    // The marks keep their colours.
    let colours: Vec<&str> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { colour, .. } => Some(colour.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(colours.len(), 6);
    assert!(colours.iter().any(|colour| *colour != colours[0]));
}

#[test]
fn the_bar_of_a_ramp_lies_down_above_the_panel() {
    let scene = placed(rows(6, 3), &[(Aesthetic::Color, "z")], LegendPlace::Top);
    let drawing = drawn(&scene);
    let rasters: Vec<(f64, f64, usize, usize)> = of_legends(&drawing)
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Raster {
                width,
                height,
                columns,
                rows,
                ..
            } => Some((*width, *height, *columns, *rows)),
            _ => None,
        })
        .collect();
    // One bar, wider than it is high, a colour to a column, the lowest value at the left.
    assert_eq!(rasters.len(), 1);
    let (width, height, columns, rows) = rasters[0];
    assert!(width > height);
    assert_eq!((columns, rows), (256, 1));
    // Standing at the right it is the other way about.
    let right = drawn(&placed(
        rows_of(),
        &[(Aesthetic::Color, "z")],
        LegendPlace::Right,
    ));
    let standing = of_legends(&right)
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::Raster { columns, rows, .. } => Some((*columns, *rows)),
            _ => None,
        })
        .unwrap();
    assert_eq!(standing, (1, 256));
    insta::assert_snapshot!("a_ramp_above", drawing.describe());
}

fn rows_of() -> Frame {
    rows(6, 3)
}

#[test]
fn many_keys_take_as_few_rows_as_the_width_holds() {
    // Forty categories on a page of two columns: more than one row, every key on the page.
    let scene = placed(
        rows(40, 40),
        &[(Aesthetic::Color, "g")],
        LegendPlace::Bottom,
    );
    let drawing = layout(&scene, Size::TwoColumn, &JOURNAL).unwrap();
    let (page_width, page_height) = Size::TwoColumn.points();
    let keys: Vec<(f64, f64)> = of_legends(&drawing)
        .iter()
        .filter(|item| matches!(item.primitive, Primitive::Point { .. }))
        .map(|item| at(item))
        .collect();
    assert_eq!(keys.len(), 40);
    assert!(
        keys.iter()
            .all(|(x, y)| *x > 0.0 && *x < page_width && *y > 0.0 && *y < page_height)
    );
    let mut lines: Vec<f64> = keys.iter().map(|(_, y)| *y).collect();
    lines.sort_by(f64::total_cmp);
    lines.dedup();
    assert!(lines.len() > 1 && lines.len() < 40, "{} rows", lines.len());
    // On a page of one column there is no room left for the panel, and the layout says so.
    let small = layout(&scene, Size::OneColumn, &JOURNAL);
    assert!(
        matches!(small, Err(LayoutError::TooSmall { .. })),
        "{small:?}"
    );
}

#[test]
fn two_legends_stand_side_by_side_and_a_long_title_is_broken() {
    // A title of many words for the colour.
    let names = [
        "group 00", "group 01", "group 02", "group 00", "group 01", "group 02",
    ];
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0])),
            ("y", numbers(&[0.0, 7.0, 3.0, 10.0, 6.0, 2.0])),
            (
                "g",
                Column::strs(names.iter().map(|name| Some(*name))).with_entry(
                    MetadataKey::Label,
                    "The group each of the units was assigned to at the start of the trial",
                ),
            ),
            ("z", numbers(&[0.0, 10.0, 20.0, 30.0, 40.0, 50.0])),
        ],
    );
    let scene = placed(
        frame,
        &[(Aesthetic::Color, "g"), (Aesthetic::Size, "z")],
        LegendPlace::Top,
    );
    assert_eq!(scene.legends.len(), 2);
    let drawing = layout(&scene, Size::TwoColumn, &JOURNAL).unwrap();
    let (panel, _, _, _) = panel_of(&drawing);
    let texts: Vec<(f64, f64)> = of_legends(&drawing)
        .iter()
        .filter(|item| matches!(item.primitive, Primitive::Text { .. }))
        .map(|item| at(item))
        .collect();
    // Both legends above the panel, and the second to the right of the first.
    assert!(texts.iter().all(|(_, y)| *y < panel.1));
    let (least, most) = texts.iter().fold((f64::MAX, f64::MIN), |(lo, hi), (x, _)| {
        (lo.min(*x), hi.max(*x))
    });
    assert!(most - least > 100.0, "{least} to {most}");
    insta::assert_snapshot!("two_legends_above", drawing.describe());
}

// ----- legend(direct): the name of each line at its end (step 25e2) -----

/// The rows as lines, shown by the aesthetics named, with the legends at `place`.
fn lined(frame: Frame, shown: &[(Aesthetic, &str)], place: LegendPlace) -> Scene {
    let mut mappings = vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")];
    mappings.extend(
        shown
            .iter()
            .map(|(aesthetic, name)| mapped(*aesthetic, name)),
    );
    let mut plot: Plot = start(frame, mappings, Vec::new());
    plot.legend = place;
    let lines = PlotLayer {
        mark: Mark::Lines,
        ..points(1.0)
    };
    with_layer(&plot, lines).1
}

/// The texts of the items of the legends of a drawing: what is written, in what colour, and
/// where.
fn written(drawing: &Drawing) -> Vec<(String, String, (f64, f64))> {
    of_legends(drawing)
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, spans, .. } => Some((
                spans
                    .iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>(),
                spans
                    .first()
                    .map_or(String::new(), |span| span.colour.clone()),
                *at,
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn the_name_of_each_line_is_written_at_its_end_in_its_colour() {
    let by_g = &[(Aesthetic::Color, "g")];
    let scene = lined(rows(6, 3), by_g, LegendPlace::Direct);
    assert_eq!(scene.legend, LegendPlace::Direct);
    assert_eq!(scene.named_at_ends(), [true]);
    assert!(scene.json().write().contains("\"legend\": \"direct\""));
    let drawing = drawn(&scene);
    let (panel, width, height, marks) = panel_of(&drawing);
    // No key, no title: three names and nothing else.
    let items = of_legends(&drawing);
    assert_eq!(items.len(), 3);
    let names = written(&drawing);
    let texts: Vec<&str> = names.iter().map(|(text, _, _)| text.as_str()).collect();
    assert_eq!(texts, ["group 00", "group 01", "group 02"]);
    // Each in the colour of its line, the three of Okabe and Ito.
    let colours: Vec<&str> = names.iter().map(|(_, colour, _)| colour.as_str()).collect();
    assert_eq!(colours, ["#E69F00", "#56B4E9", "#009E73"]);
    let strokes: Vec<String> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Path { stroke, .. } => stroke.as_ref().map(|s| s.colour.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(strokes, colours);
    // Each beside the right edge of the panel, within its height, and at the height its
    // line ends at: the lines end at y = 10, 6 and 2 of a scale from -0.5 to 10.5.
    let y = scene.scale("y").unwrap();
    for ((_, _, at), end) in names.iter().zip([10.0, 6.0, 2.0]) {
        assert!(at.0 > panel.0 + width && at.0 < panel.0 + width + 10.0);
        assert!(at.1 > panel.1 && at.1 < panel.1 + height + 5.0);
        let share = (end - y.limits.0) / (y.limits.1 - y.limits.0);
        let line_end = panel.1 + height - share * height;
        // The text stands on its baseline, half the height of a capital under the middle.
        assert!(
            (at.1 - line_end).abs() < 4.0,
            "{} is not near {line_end}",
            at.1
        );
    }
    // The room at the right is that of the names: less than a legend with its keys takes.
    let (_, width_right, _, _) = panel_of(&drawn(&lined(rows(6, 3), by_g, LegendPlace::Right)));
    assert!(width > width_right);
    insta::assert_snapshot!("three_names_at_the_ends", drawing.describe());
}

#[test]
fn names_that_would_touch_are_moved_apart_and_keep_their_order() {
    // Four lines that end within a hair of one another.
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0])),
            ("y", numbers(&[0.0, 5.00, 1.0, 5.01, 2.0, 5.02, 9.0, 5.03])),
            (
                "g",
                Column::strs(
                    ["a", "a", "b", "b", "c", "c", "d", "d"]
                        .iter()
                        .map(|name| Some(*name)),
                ),
            ),
        ],
    );
    let drawing = drawn(&lined(
        frame,
        &[(Aesthetic::Color, "g")],
        LegendPlace::Direct,
    ));
    let names = written(&drawing);
    assert_eq!(names.len(), 4);
    // Down the page the names are in the order of the ends, the highest first, and a line
    // of text apart.
    let mut by_height: Vec<(f64, &str)> =
        names.iter().map(|(t, _, at)| (at.1, t.as_str())).collect();
    by_height.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(
        by_height
            .iter()
            .map(|(_, text)| *text)
            .collect::<Vec<&str>>(),
        ["d", "c", "b", "a"]
    );
    assert!(
        by_height
            .windows(2)
            .all(|pair| pair[1].0 - pair[0].0 >= 8.0 * JOURNAL.leading - 1e-9),
        "{by_height:?}"
    );
    let (panel, _, height, _) = panel_of(&drawing);
    assert!(
        by_height
            .iter()
            .all(|(y, _)| *y > panel.1 && *y < panel.1 + height + 5.0)
    );
}

#[test]
fn a_legend_no_line_reads_stands_at_the_right() {
    // Points coloured by a category: no line to write a name at.
    let by_g = &[(Aesthetic::Color, "g")];
    let points_direct = placed(rows(6, 3), by_g, LegendPlace::Direct);
    assert_eq!(points_direct.named_at_ends(), [false]);
    let points_right = placed(rows(6, 3), by_g, LegendPlace::Right);
    assert_eq!(
        drawn(&points_direct).describe(),
        drawn(&points_right).describe()
    );
    // Lines coloured by a number: a bar of colours, which has no ends of lines either.
    let ramp = lined(rows(6, 3), &[(Aesthetic::Color, "z")], LegendPlace::Direct);
    assert_eq!(ramp.named_at_ends(), [false]);
    assert!(
        of_legends(&drawn(&ramp))
            .iter()
            .any(|item| matches!(item.primitive, Primitive::Raster { .. }))
    );
    // Lines by colour, and nothing else: the plain plot has no legend to place.
    let plain = lined(rows(6, 3), &[], LegendPlace::Direct);
    assert!(plain.named_at_ends().is_empty());
    assert!(of_legends(&drawn(&plain)).is_empty());
}

#[test]
fn a_category_drawn_as_several_lines_is_named_once() {
    // Two categories, each two lines by a group: the name stands at the line that reaches
    // furthest along x.
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[0.0, 1.0, 0.0, 3.0, 0.0, 2.0, 0.0, 1.0])),
            ("y", numbers(&[0.0, 1.0, 0.0, 9.0, 0.0, 4.0, 0.0, 6.0])),
            (
                "g",
                Column::strs(
                    ["a", "a", "a", "a", "b", "b", "b", "b"]
                        .iter()
                        .map(|name| Some(*name)),
                ),
            ),
            (
                "unit",
                Column::strs(
                    ["p", "p", "q", "q", "r", "r", "s", "s"]
                        .iter()
                        .map(|name| Some(*name)),
                ),
            ),
        ],
    );
    let scene = lined(
        frame,
        &[(Aesthetic::Color, "g"), (Aesthetic::Group, "unit")],
        LegendPlace::Direct,
    );
    let drawing = drawn(&scene);
    let names = written(&drawing);
    assert_eq!(names.len(), 2);
    let (panel, _, height, _) = panel_of(&drawing);
    let y = scene.scale("y").unwrap();
    // `a` at the end of the line that reaches x = 3, where y is 9; `b` at x = 2, y = 4.
    for ((text, _, at), end) in names.iter().zip([9.0, 4.0]) {
        let share = (end - y.limits.0) / (y.limits.1 - y.limits.0);
        let line_end = panel.1 + height - share * height;
        assert!(
            (at.1 - line_end).abs() < 4.0,
            "{text}: {} is not near {line_end}",
            at.1
        );
    }
}

#[test]
fn the_names_stand_beside_the_legends_they_do_not_replace() {
    // Lines by colour and by size of a number: the names of the lines, and the keys of the
    // sizes at the right of them.
    let scene = {
        let mut mappings = vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
            mapped(Aesthetic::Size, "z"),
        ];
        mappings.truncate(4);
        let mut plot: Plot = start(rows(6, 3), mappings, Vec::new());
        plot.legend = LegendPlace::Direct;
        let lines = PlotLayer {
            mark: Mark::Lines,
            ..points(1.0)
        };
        let (plot, _) = with_layer(&plot, lines);
        with_layer(&plot, points(1.0)).1
    };
    let named = scene.named_at_ends();
    assert_eq!(scene.legends.len(), named.len());
    assert!(
        named.iter().any(|n| *n) && named.iter().any(|n| !*n),
        "{named:?}"
    );
    let drawing = drawn(&scene);
    let names = written(&drawing);
    let texts: Vec<&str> = names.iter().map(|(text, _, _)| text.as_str()).collect();
    for name in ["group 00", "group 01", "group 02"] {
        assert_eq!(
            texts.iter().filter(|text| **text == name).count(),
            1,
            "{texts:?}"
        );
    }
    // The keys of the sizes are drawn, to the right of the names.
    let furthest_name = names
        .iter()
        .filter(|(text, _, _)| text.starts_with("group"))
        .map(|(_, _, at)| at.0)
        .fold(f64::MIN, f64::max);
    let keys: Vec<f64> = of_legends(&drawing)
        .iter()
        .filter(|item| matches!(item.primitive, Primitive::Point { .. }))
        .map(|item| at(item).0)
        .collect();
    assert!(!keys.is_empty());
    assert!(keys.iter().all(|x| *x > furthest_name));
}
