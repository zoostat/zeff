//! `lines` in the scene and on the page (Language Reference, section 7.6, "Marks"; release
//! 0.7.0, step 23c): the rows of a layer of lines are parted into groups, put in the order of
//! `x` and joined, a line broken where `y` is missing; the layout draws each line as one path
//! in one colour and one type, and the key of a legend of lines is a piece of line.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Mark, PlotConstant, PlotLayer, PlotSetting};
use zeff_plot::drawing::{Head, LineType, Primitive, Step};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{LINE_WIDTH, Lining, Paint, Scene, start, with_layer};

/// A layer of lines that says nothing of its own.
fn lines() -> PlotLayer {
    PlotLayer {
        mark: Mark::Lines,
        ..points(1.0)
    }
}

/// The scene of one layer over a frame with `x`, `y`, a category `g` and a number `z`.
fn joined(rows: &[Row], layer: PlotLayer) -> Scene {
    let plot = start(
        tried(rows),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    with_layer(&plot, layer).1
}

fn row(x: f64, y: f64) -> Row {
    (Some(x), Some(y), None, None)
}

#[test]
fn rows_are_joined_in_the_order_of_x_whatever_their_order_in_the_frame() {
    let rows = [
        row(3.0, 30.0),
        row(1.0, 10.0),
        row(4.0, 40.0),
        row(2.0, 20.0),
    ];
    let scene = joined(&rows, lines());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Lines);
    assert_eq!(layer.x, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(layer.y, [10.0, 20.0, 30.0, 40.0]);
    // Each mark keeps the row of the frame it is drawn from, counted from 1.
    assert_eq!(layer.rows, [2, 4, 1, 3]);
    assert_eq!(layer.line, [1, 1, 1, 1]);
    assert_eq!(layer.linetype, Lining::Constant("solid"));
    assert_eq!(layer.width, LINE_WIDTH);
    assert_eq!(layer.color, Paint::Theme);
    assert!(scene.notes.is_empty());
    // Rows with one `x` keep the order of the frame.
    let ties = [row(2.0, 5.0), row(1.0, 1.0), row(2.0, 9.0), row(1.0, 3.0)];
    let layer = &joined(&ties, lines()).panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 1.0, 2.0, 2.0]);
    assert_eq!(layer.y, [1.0, 3.0, 5.0, 9.0]);
    assert_eq!(layer.rows, [2, 4, 1, 3]);
}

#[test]
fn a_line_is_broken_where_y_is_missing_and_a_row_left_alone_draws_nothing() {
    let rows: Vec<Row> = vec![
        row(1.0, 1.0),
        row(2.0, 2.0),
        (Some(3.0), None, None, None),
        row(4.0, 4.0),
        row(5.0, 3.0),
        (Some(6.0), None, None, None),
        row(7.0, 5.0),
        (None, Some(9.0), None, None),
    ];
    let scene = joined(&rows, lines());
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 4.0, 5.0]);
    assert_eq!(layer.line, [1, 1, 2, 2]);
    assert_eq!(layer.rows, [1, 2, 4, 5]);
    assert_eq!(
        scene.notes,
        [
            "3 rows with NA in x or y dropped",
            "2 breaks in the lines where y is NA",
            "1 row with no neighbour to be joined to draws no line",
        ]
    );
    // The axis runs over the marks that are drawn, as for any layer.
    assert_eq!(scene.scale("x").unwrap().limits, (0.8, 5.2));
}

#[test]
fn the_rows_are_parted_by_the_group_the_colour_and_the_type_of_line() {
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![
        cell(1.0, 1.0, 0),
        cell(1.0, 5.0, 1),
        cell(2.0, 2.0, 0),
        cell(2.0, 6.0, 1),
        cell(3.0, 3.0, 0),
    ];
    // By the group: the lines in the order their first row comes.
    let mut grouped = lines();
    grouped.mappings = vec![mapped(Aesthetic::Group, "g")];
    let scene = joined(&rows, grouped);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 2.0, 3.0, 1.0, 2.0]);
    assert_eq!(layer.y, [1.0, 2.0, 3.0, 5.0, 6.0]);
    assert_eq!(layer.line, [1, 1, 1, 2, 2]);
    assert!(scene.legends.is_empty());
    // By a colour: the same lines, each mark with the place of its category, and a legend.
    let mut coloured = lines();
    coloured.mappings = vec![mapped(Aesthetic::Color, "g")];
    let scene = joined(&rows, coloured);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.line, [1, 1, 1, 2, 2]);
    assert_eq!(layer.color, Paint::Scaled(vec![1.0, 1.0, 1.0, 2.0, 2.0]));
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].marks, [Mark::Lines]);
    // By a type of line: a scale of the six types in their order.
    let mut typed = lines();
    typed.mappings = vec![mapped(Aesthetic::Linetype, "g")];
    let scene = joined(&rows, typed);
    let layer = &scene.panels[0].layers[0];
    assert_eq!(
        layer.linetype,
        Lining::Scaled(vec![1.0, 1.0, 1.0, 2.0, 2.0])
    );
    let scale = scene.scale("linetype").unwrap();
    assert_eq!(scale.labels, ["group <00> & co", "group <01> & co"]);
    assert_eq!(
        scale.range,
        zeff_plot::scene::Range::LineTypes(vec!["solid", "dashed"])
    );
    // By a colour and a type of line of one variable: one legend for the two scales.
    let mut both = lines();
    both.mappings = vec![
        mapped(Aesthetic::Color, "g"),
        mapped(Aesthetic::Linetype, "g"),
    ];
    let scene = joined(&rows, both);
    assert_eq!(scene.legends.len(), 1);
    assert_eq!(scene.legends[0].scales, ["color", "linetype"]);
    // One type and one width for every line of a layer.
    let mut set = lines();
    set.constants = vec![PlotConstant {
        aesthetic: Aesthetic::Linetype,
        value: PlotSetting::Line("long_dash"),
    }];
    set.width = Some(2.5);
    let layer = &joined(&rows, set).panels[0].layers[0];
    assert_eq!(layer.linetype, Lining::Constant("long_dash"));
    assert_eq!(layer.width, 2.5);
}

#[test]
fn a_line_is_one_path_on_the_page_and_its_key_a_piece_of_line() {
    let cell = |x: f64, y: f64, g: u8| (Some(x), Some(y), Some(g), None);
    let rows: Vec<Row> = vec![
        cell(1.0, 1.0, 0),
        cell(1.0, 5.0, 1),
        cell(2.0, 2.0, 0),
        cell(2.0, 6.0, 1),
        cell(3.0, 3.0, 0),
    ];
    let mut layer = lines();
    layer.mappings = vec![
        mapped(Aesthetic::Color, "g"),
        mapped(Aesthetic::Linetype, "g"),
    ];
    layer.width = Some(1.5);
    let scene = joined(&rows, layer);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let (_, _, _, marks) = panel_of(&drawing);
    // Two lines: one of three marks and one of two, each one path, in its colour and type.
    assert_eq!(marks.len(), 2);
    let path = |i: usize| match &marks[i].primitive {
        Primitive::Path {
            steps,
            stroke: Some(stroke),
            fill: None,
            opacity,
        } => {
            assert_eq!(*opacity, 1.0);
            (steps.clone(), stroke.clone())
        }
        other => panic!("{other:?} is not a line"),
    };
    let (first, ink) = path(0);
    assert_eq!(first.len(), 3);
    assert!(matches!(first[0], Step::Move(_)) && matches!(first[1], Step::Line(_)));
    assert_eq!(
        (ink.colour.as_str(), ink.width, ink.dash),
        ("#E69F00", 1.5, LineType::Solid)
    );
    let (second, ink) = path(1);
    assert_eq!(second.len(), 2);
    assert_eq!(
        (ink.colour.as_str(), ink.width, ink.dash),
        ("#56B4E9", 1.5, LineType::Dashed)
    );
    // The line goes from left to right, and up the page as `y` grows.
    let at = |step: &Step| match step {
        Step::Move(p) | Step::Line(p) => *p,
        other => panic!("{other:?}"),
    };
    assert!(at(&first[0]).0 < at(&first[1]).0 && at(&first[1]).0 < at(&first[2]).0);
    assert!(at(&first[0]).1 > at(&first[1]).1 && at(&first[1]).1 > at(&first[2]).1);
    // The legend: for each category a piece of line, 24 points long, in its colour, its type
    // and the width of the layer, and no point on it.
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:color+linetype"))
        .collect();
    let pieces: Vec<_> = keys
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Segment {
                from,
                to,
                stroke,
                heads: (Head::Plain, Head::Plain),
            } => Some((to.0 - from.0, from.1 == to.1, stroke.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(pieces.len(), 2);
    assert!((pieces[0].0 - 24.0).abs() < 1e-9 && pieces[0].1);
    assert_eq!(pieces[0].2.dash, LineType::Solid);
    assert_eq!(pieces[1].2.dash, LineType::Dashed);
    assert_eq!(pieces[1].2.colour, "#56B4E9");
    assert_eq!(pieces[1].2.width, 1.5);
    assert!(
        !keys
            .iter()
            .any(|item| matches!(item.primitive, Primitive::Point { .. }))
    );
    insta::assert_snapshot!("two_lines", drawing.describe());

    // Points and lines of one variable: the key is a piece of line with a point on it.
    let plot = start(
        tried(&rows),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(Aesthetic::Color, "g"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let (_, scene) = with_layer(&plot, lines());
    assert_eq!(scene.legends[0].marks, [Mark::Points, Mark::Lines]);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let keys: Vec<_> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("legend:color"))
        .collect();
    let lines_in = keys
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Segment { .. }))
        .count();
    let points_in = keys
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Point { .. }))
        .count();
    assert_eq!((lines_in, points_in), (2, 2));
    // The marks of the panel: the five points, and over them the two lines.
    let (_, _, _, marks) = panel_of(&drawing);
    assert_eq!(marks.len(), 7);
    assert!(matches!(marks[5].primitive, Primitive::Path { .. }));
}

#[test]
fn the_scene_of_lines_says_which_line_each_mark_is_of() {
    let rows = [row(3.0, 30.0), row(1.0, 10.0), row(2.0, 20.0)];
    let json = joined(&rows, lines()).json().write();
    assert!(json.contains("\"mark\": \"lines\""), "{json}");
    assert!(json.contains("\"line\": [1, 1, 1]"), "{json}");
    assert!(json.contains("\"linetype\": \"solid\""), "{json}");
    assert!(json.contains("\"width\": 1"), "{json}");
    // A layer of points says none of the three.
    let json = joined(&rows, points(1.0)).json().write();
    assert!(
        !json.contains("\"line\"") && !json.contains("linetype") && !json.contains("\"width\"")
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(300)))]

    #[test]
    fn every_line_has_two_marks_or_more_in_the_order_of_x_and_no_row_is_lost_or_drawn_twice(
        rows in rows(),
        by in 0u8..4,
    ) {
        let mut layer = lines();
        layer.mappings = match by {
            1 => vec![mapped(Aesthetic::Group, "g")],
            2 => vec![mapped(Aesthetic::Color, "g")],
            3 => vec![mapped(Aesthetic::Linetype, "g"), mapped(Aesthetic::Group, "g")],
            _ => Vec::new(),
        };
        let scene = joined(&rows, layer);
        let drawn = &scene.panels[0].layers[0];
        prop_assert_eq!(drawn.x.len(), drawn.line.len());
        prop_assert_eq!(drawn.x.len(), drawn.rows.len());
        // The lines are numbered from 1 without a gap, their marks side by side.
        let mut seen = 0;
        let mut start = 0;
        while start < drawn.line.len() {
            let mut end = start + 1;
            while end < drawn.line.len() && drawn.line[end] == drawn.line[start] {
                end += 1;
            }
            seen += 1;
            prop_assert_eq!(drawn.line[start], seen);
            prop_assert!(end - start >= 2, "a line of one mark");
            for i in start + 1..end {
                prop_assert!(drawn.x[i - 1] <= drawn.x[i], "not in the order of x");
                // Rows with one x keep the order of the frame.
                if drawn.x[i - 1] == drawn.x[i] {
                    prop_assert!(drawn.rows[i - 1] < drawn.rows[i]);
                }
                // One group to a line.
                if by != 0 {
                    prop_assert_eq!(rows[drawn.rows[i - 1] - 1].2, rows[drawn.rows[i] - 1].2);
                }
            }
            start = end;
        }
        // Each mark is a row with both positions, drawn once, at its own values.
        let mut used = vec![false; rows.len()];
        for (i, &at) in drawn.rows.iter().enumerate() {
            let (x, y, _, _) = rows[at - 1];
            prop_assert_eq!(x, Some(drawn.x[i]));
            prop_assert_eq!(y, Some(drawn.y[i]));
            prop_assert!(!used[at - 1], "a row drawn twice");
            used[at - 1] = true;
        }
        // The rows not drawn are those without a position, counted in a note, and those left
        // alone, counted in another.
        let complete = rows.iter().filter(|r| r.0.is_some() && r.1.is_some()).count();
        let alone = complete - drawn.rows.len();
        let said = scene.notes.iter().any(|note| note.contains("no neighbour"));
        prop_assert_eq!(alone > 0, said, "{:?}", scene.notes);
    }

    #[test]
    fn the_lines_do_not_depend_on_the_order_of_the_rows_when_no_two_share_an_x(
        mut values in prop::collection::vec((0u8..3, -50.0..50.0f64), 2..20),
        seed in any::<u64>(),
    ) {
        // Rows with x from 1 on, so that no two share one; the same rows in another order.
        let rows: Vec<Row> = values
            .iter()
            .enumerate()
            .map(|(i, (g, y))| (Some(i as f64 + 1.0), Some(*y), Some(*g), None))
            .collect();
        let mut state = seed | 1;
        let mut shuffled = rows.clone();
        for i in (1..shuffled.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            shuffled.swap(i, (state % (i as u64 + 1)) as usize);
        }
        values.clear();
        let mut layer = lines();
        layer.mappings = vec![mapped(Aesthetic::Group, "g")];
        // Each line as its marks, the lines put in the order of their first mark.
        let as_sets = |scene: &Scene| {
            let drawn = &scene.panels[0].layers[0];
            let mut out: Vec<Vec<(u64, u64)>> = Vec::new();
            for i in 0..drawn.line.len() {
                if i == 0 || drawn.line[i] != drawn.line[i - 1] {
                    out.push(Vec::new());
                }
                out.last_mut().unwrap().push((drawn.x[i].to_bits(), drawn.y[i].to_bits()));
            }
            out.sort();
            out
        };
        prop_assert_eq!(as_sets(&joined(&rows, layer.clone())), as_sets(&joined(&shuffled, layer)));
    }
}

/// The frame of `common::tried` holds a category `g`; a column of text stands for it here.
#[test]
fn a_group_may_be_of_any_type() {
    let frame = frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 2.0, 1.0, 2.0])),
            ("y", numbers(&[1.0, 2.0, 3.0, 4.0])),
            ("who", Column::strs(["p", "p", "q", "q"].map(Some))),
            ("id", numbers(&[7.0, 7.0, 9.0, 9.0])),
        ],
    );
    for by in ["who", "id"] {
        let plot = start(
            frame.clone(),
            vec![
                mapped(Aesthetic::X, "x"),
                mapped(Aesthetic::Y, "y"),
                mapped(Aesthetic::Group, by),
            ],
            Vec::new(),
        );
        let layer = &with_layer(&plot, lines()).1.panels[0].layers[0];
        assert_eq!(layer.line, [1, 1, 2, 2], "{by}");
        assert_eq!(layer.y, [1.0, 2.0, 3.0, 4.0]);
    }
}
