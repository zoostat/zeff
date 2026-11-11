//! The scene of a plot (Language Reference, section 7.6, "Rendering and export"; Engine and
//! Project Specification, section 8.3, point 5; release 0.7.0, step 20): snapshots of the scene
//! of small plots as JSON, the JSON read back value for value, the edge cases, and the
//! properties: the same data give the same scene, reordering the rows changes nothing but the
//! order of the marks, every break and every mark lies within the limits, and jitter moves a
//! point by no more than its share of the gap. From step 20b, a position mapped from an
//! expression: its values are handed to the scene with the plot, so the scene shows them under
//! the expression as written, and an expression that is one variable gives the scene of that
//! variable.

use proptest::prelude::*;
use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::{Column, MetadataKey, Table, format_value_labels};
use zeff_core::value::{
    Aesthetic, ColumnExpr, Frame, Mark, Plot, PlotComputed, PlotConstant, PlotLayer, PlotMapping,
    PlotSetting,
};
use zeff_numeric::random::Stream;
use zeff_plot::colour::{NA_COLOUR, OKABE_ITO, parse, ramp_entry};
use zeff_plot::scene::{
    POINT_SIZE, Paint, Range, SCENE_VERSION, ScaleKind, Scene, Shading, reads, scene, shading,
    start, with_layer,
};

fn floats(values: &[Option<f64>]) -> Column {
    Column::floats(values.iter().copied()).0
}

fn numbers(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

fn frame(name: &str, columns: Vec<(&str, Column)>) -> Frame {
    let table = Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap();
    Frame {
        table: Rc::new(table),
        name: Some(name.to_string()),
    }
}

/// A plot of `frame` with `x` and `y` mapped to the variables named.
fn plot(frame: Frame, x: Option<&str>, y: Option<&str>) -> Plot {
    let mut mappings = Vec::new();
    if let Some(x) = x {
        mappings.push(PlotMapping {
            aesthetic: Aesthetic::X,
            variable: x.to_string(),
            computed: None,
        });
    }
    if let Some(y) = y {
        mappings.push(PlotMapping {
            aesthetic: Aesthetic::Y,
            variable: y.to_string(),
            computed: None,
        });
    }
    start(frame, mappings, Vec::new())
}

/// A layer of points as `.points()` writes it.
fn points() -> PlotLayer {
    PlotLayer {
        mark: Mark::Points,
        data: None,
        alpha: 1.0,
        size: None,
        jitter: 0.0,
        stream: None,
        computed: Vec::new(),
    }
}

/// A layer of points with `jitter:`, drawing from the stream of a statement under a seed.
fn jittered(share: f64, seed: u64) -> PlotLayer {
    PlotLayer {
        jitter: share,
        stream: Some(Stream::new(seed, 3, 0)),
        ..points()
    }
}

/// The scatter of two float variables, `x` and `y`, of a frame called `d`.
fn scatter(x: &[Option<f64>], y: &[Option<f64>]) -> Scene {
    let p = plot(
        frame("d", vec![("x", floats(x)), ("y", floats(y))]),
        Some("x"),
        Some("y"),
    );
    with_layer(&p, points()).1
}

fn some(values: &[f64]) -> Vec<Option<f64>> {
    values.iter().map(|v| Some(*v)).collect()
}

/// The dose and response frame of the help page, with a label on the dose.
fn doses() -> Frame {
    frame(
        "d",
        vec![
            (
                "dose",
                floats(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0), None])
                    .with_entry(MetadataKey::Label, "Dose (mg)"),
            ),
            ("response", numbers(&[2.1, 3.9, 6.2, 7.8, 9.0])),
        ],
    )
}

// ----- the scene as JSON -----

#[test]
fn the_scene_of_a_small_scatter() {
    let p = plot(doses(), Some("dose"), Some("response"));
    let (_, drawn) = with_layer(&p, points());
    insta::assert_snapshot!("scatter", drawn.to_json());
}

#[test]
fn the_scene_before_any_layer() {
    let p = plot(doses(), Some("dose"), Some("response"));
    insta::assert_snapshot!("no_layer", scene(&p).to_json());
}

#[test]
fn the_scene_of_two_layers_one_jittered_and_one_with_its_own_frame() {
    let p = plot(
        frame(
            "d",
            vec![
                ("dose", numbers(&[1.0, 1.0, 2.0, 2.0, 3.0, 3.0])),
                ("response", numbers(&[2.1, 2.4, 3.9, 4.4, 6.2, 5.8])),
            ],
        ),
        Some("dose"),
        Some("response"),
    );
    let (p, _) = with_layer(&p, jittered(0.2, 20261111));
    let own = frame(
        "means",
        vec![
            ("dose", numbers(&[1.0, 2.0, 3.0, f64::INFINITY])),
            ("response", numbers(&[2.25, 4.15, 6.0, 1.0])),
        ],
    );
    let (_, drawn) = with_layer(
        &p,
        PlotLayer {
            data: Some(own),
            alpha: 0.5,
            size: Some(8.0),
            ..points()
        },
    );
    insta::assert_snapshot!("two_layers", drawn.to_json());
}

#[test]
fn the_scene_of_one_variable_and_of_none() {
    let x_alone = scene(&plot(doses(), Some("dose"), None));
    let nothing = scene(&plot(doses(), None, None));
    insta::assert_snapshot!(
        "one_variable_and_none",
        format!("{}---\n{}", x_alone.to_json(), nothing.to_json())
    );
}

/// The numbers of the `nth` list written under `key`, counted from 0, each read from its text by
/// the standard library, which rounds correctly as the readers of the scene do. (The JSON reader
/// of the tests reads a number to within one unit of its last place unless a feature is turned
/// on, so it checks the structure and the text here, and not the numbers.)
fn list(text: &str, key: &str, nth: usize) -> Vec<f64> {
    let opening = format!("\"{key}\": [");
    let line = text
        .lines()
        .map(str::trim_start)
        .filter(|l| l.starts_with(&opening))
        .nth(nth)
        .unwrap_or_else(|| panic!("no list {nth} under {key}"));
    let inside = line[opening.len()..]
        .trim_end_matches(',')
        .trim_end_matches(']');
    inside
        .split(", ")
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().unwrap_or_else(|_| panic!("not a number: {t}")))
        .collect()
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|v| v.to_bits()).collect()
}

#[test]
fn the_json_reads_back_value_for_value() {
    let x = [
        0.1 + 0.2,
        1e-7,
        123_456_789.125,
        -2.5e-12,
        1.0 / 3.0,
        6.02e23,
    ];
    let y = [1e21, -7.0, 0.000_012_345, 4.0e15, 2.0_f64.sqrt(), -1e-300];
    let drawn = scatter(&some(&x), &some(&y));
    let text = drawn.to_json();
    let json: Json = serde_json::from_str(&text).expect("the scene is JSON");
    assert_eq!(json["zeff_scene"].as_u64(), Some(u64::from(SCENE_VERSION)));
    assert_eq!(json["layout"]["rows"].as_u64(), Some(1));
    assert_eq!(json["layout"]["columns"].as_u64(), Some(1));
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected = vec![
        "zeff_scene",
        "layout",
        "scales",
        "guides",
        "panels",
        "notes",
    ];
    expected.sort_unstable();
    let mut found = keys.clone();
    found.sort_unstable();
    assert_eq!(found, expected);
    for (i, s) in drawn.scales.iter().enumerate() {
        let written = &json["scales"][i];
        assert_eq!(written["id"].as_str(), Some(s.id));
        assert_eq!(written["kind"].as_str(), Some("linear"));
        assert_eq!(written["title"].as_str(), Some(s.title.as_str()));
        assert_eq!(
            bits(&list(&text, "limits", i)),
            bits(&[s.limits.0, s.limits.1])
        );
        assert_eq!(bits(&list(&text, "breaks", i)), bits(&s.breaks));
        assert_eq!(
            written["breaks"].as_array().map(Vec::len),
            Some(s.breaks.len())
        );
        let labels: Vec<&str> = written["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect();
        assert_eq!(labels, s.labels);
    }
    assert_eq!(json["guides"]["axes"][0]["scale"].as_str(), Some("x"));
    assert_eq!(
        json["guides"]["axes"][0]["position"].as_str(),
        Some("bottom")
    );
    assert_eq!(json["guides"]["axes"][1]["scale"].as_str(), Some("y"));
    assert_eq!(json["guides"]["axes"][1]["position"].as_str(), Some("left"));
    assert_eq!(json["guides"]["legends"].as_array().map(Vec::len), Some(0));
    let panel = &json["panels"][0];
    assert_eq!(panel["row"].as_u64(), Some(1));
    assert_eq!(panel["column"].as_u64(), Some(1));
    assert_eq!(panel["x"].as_str(), Some("x"));
    assert_eq!(panel["y"].as_str(), Some("y"));
    let layer = &panel["layers"][0];
    assert_eq!(layer["mark"].as_str(), Some("points"));
    assert_eq!(bits(&list(&text, "x", 0)), bits(&x));
    assert_eq!(bits(&list(&text, "y", 0)), bits(&y));
    assert_eq!(layer["x"].as_array().map(Vec::len), Some(x.len()));
    assert_eq!(layer["y"].as_array().map(Vec::len), Some(y.len()));
    assert_eq!(layer["size"].as_f64(), Some(POINT_SIZE));
    assert_eq!(layer["alpha"].as_f64(), Some(1.0));
    assert_eq!(json["notes"].as_array().map(Vec::len), Some(0));
}

#[test]
fn a_title_or_a_note_with_quotes_and_line_breaks_is_still_json() {
    let p = plot(
        frame(
            "d",
            vec![
                (
                    "x",
                    numbers(&[1.0, 2.0])
                        .with_entry(MetadataKey::Label, "the \"dose\"\n\tin mg \\ day · μg"),
                ),
                ("y", numbers(&[1.0, 2.0])),
            ],
        ),
        Some("x"),
        Some("y"),
    );
    let (_, drawn) = with_layer(&p, points());
    let json: Json = serde_json::from_str(&drawn.to_json()).expect("the scene is JSON");
    assert_eq!(
        json["scales"][0]["title"].as_str(),
        Some("the \"dose\"\n\tin mg \\ day · μg")
    );
}

// ----- the edge cases -----

#[test]
fn one_point_has_room_on_each_side() {
    let drawn = scatter(&[Some(3.0)], &[Some(1000.0)]);
    let x = drawn.scale("x").unwrap();
    let y = drawn.scale("y").unwrap();
    assert_eq!(x.limits, (2.5, 3.5));
    assert_eq!(x.breaks, [2.6, 2.8, 3.0, 3.2, 3.4]);
    assert_eq!(x.labels, ["2.6", "2.8", "3", "3.2", "3.4"]);
    assert_eq!(y.limits, (950.0, 1050.0));
    assert_eq!(y.breaks, [960.0, 980.0, 1000.0, 1020.0, 1040.0]);
    assert_eq!(drawn.panels[0].layers[0].x, [3.0]);
    assert_eq!(drawn.panels[0].layers[0].y, [1000.0]);
    assert!(drawn.notes.is_empty());
}

#[test]
fn a_constant_variable_is_centred() {
    let drawn = scatter(&some(&[1.0, 2.0, 3.0]), &some(&[-7.0, -7.0, -7.0]));
    assert_eq!(drawn.scale("y").unwrap().limits, (-7.5, -6.5));
    assert_eq!(drawn.scale("x").unwrap().limits, (0.9, 3.1));
    let zero = scatter(&some(&[0.0, 0.0]), &some(&[0.0, 0.0]));
    assert_eq!(zero.scale("x").unwrap().limits, (-0.5, 0.5));
}

#[test]
fn a_frame_of_missing_values_draws_nothing_and_says_so() {
    let drawn = scatter(&[None, Some(2.0), None], &[Some(1.0), None, None]);
    assert!(drawn.panels[0].layers[0].x.is_empty());
    assert!(drawn.panels[0].layers[0].y.is_empty());
    assert_eq!(drawn.notes, ["3 rows with NA in x or y dropped"]);
    // No mark is drawn, so each axis shows the values its variable has in the frame.
    assert_eq!(drawn.scale("x").unwrap().limits, (1.5, 2.5));
    assert_eq!(drawn.scale("y").unwrap().limits, (0.5, 1.5));
}

#[test]
fn an_empty_frame_gives_the_unit_axes() {
    let drawn = scatter(&[], &[]);
    for id in ["x", "y"] {
        let s = drawn.scale(id).unwrap();
        assert_eq!(s.limits, (0.0, 1.0));
        assert_eq!(s.breaks, [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!(s.labels, ["0", "0.2", "0.4", "0.6", "0.8", "1"]);
    }
    assert!(drawn.notes.is_empty());
    assert_eq!(drawn.panels.len(), 1);
    assert_eq!(drawn.panels[0].layers.len(), 1);
}

#[test]
fn a_plot_without_a_layer_shows_the_axes_of_its_frame() {
    let p = plot(doses(), Some("dose"), Some("response"));
    let drawn = scene(&p);
    assert!(drawn.panels[0].layers.is_empty());
    // Before any layer nothing is dropped, so the response of the row without a dose counts.
    assert_eq!(drawn.scale("x").unwrap().limits, (0.85, 4.15));
    // 5% of the range of 2.1 to 9.0 beyond each end, to the rounding of the sums.
    let (lo, hi) = drawn.scale("y").unwrap().limits;
    assert!(
        (lo - 1.755).abs() < 1e-12 && (hi - 9.345).abs() < 1e-12,
        "{lo} {hi}"
    );
    assert_eq!(drawn.scale("x").unwrap().title, "Dose (mg)");
    assert_eq!(drawn.scale("y").unwrap().title, "response");
    assert!(drawn.notes.is_empty());
    assert_eq!(p.layers.len(), 0);
}

#[test]
fn one_variable_gives_one_axis_and_none_gives_none() {
    let x_alone = scene(&plot(doses(), Some("dose"), None));
    assert_eq!(x_alone.scales.len(), 1);
    assert_eq!(x_alone.axes.len(), 1);
    assert_eq!(x_alone.panels[0].x, Some("x"));
    assert_eq!(x_alone.panels[0].y, None);
    let y_alone = scene(&plot(doses(), None, Some("response")));
    assert_eq!(y_alone.scales[0].id, "y");
    assert_eq!(y_alone.panels[0].x, None);
    let nothing = scene(&plot(doses(), None, None));
    assert!(nothing.scales.is_empty());
    assert!(nothing.axes.is_empty());
    assert_eq!(nothing.panels.len(), 1);
}

#[test]
fn a_layer_with_its_own_frame_draws_that_frame_under_the_titles_of_the_plot() {
    let p = plot(doses(), Some("dose"), Some("response"));
    let own = frame(
        "other",
        vec![
            (
                "dose",
                numbers(&[10.0, 20.0]).with_entry(MetadataKey::Label, "another label"),
            ),
            ("response", numbers(&[-1.0, 1.0])),
        ],
    );
    let (extended, drawn) = with_layer(
        &p,
        PlotLayer {
            data: Some(own),
            ..points()
        },
    );
    assert_eq!(drawn.panels[0].layers[0].x, [10.0, 20.0]);
    assert_eq!(drawn.panels[0].layers[0].y, [-1.0, 1.0]);
    assert_eq!(drawn.scale("x").unwrap().title, "Dose (mg)");
    assert_eq!(drawn.scale("x").unwrap().limits, (9.5, 20.5));
    assert!(drawn.notes.is_empty());
    // The plot given is not changed.
    assert_eq!(p.layers.len(), 0);
    assert_eq!(extended.layers.len(), 1);
}

#[test]
fn an_infinite_value_is_not_drawn_and_is_counted_apart() {
    let drawn = scatter(
        &[Some(1.0), Some(f64::INFINITY), Some(3.0), None, Some(5.0)],
        &[
            Some(1.0),
            Some(2.0),
            Some(f64::NEG_INFINITY),
            Some(4.0),
            Some(5.0),
        ],
    );
    assert_eq!(drawn.panels[0].layers[0].x, [1.0, 5.0]);
    assert_eq!(drawn.panels[0].layers[0].y, [1.0, 5.0]);
    assert_eq!(
        drawn.notes,
        [
            "1 row with NA in x or y dropped",
            "2 rows with an infinite value in x or y dropped"
        ]
    );
    assert_eq!(drawn.scale("x").unwrap().limits, (0.8, 5.2));
}

#[test]
fn a_second_layer_over_the_same_rows_does_not_repeat_the_note() {
    let p = plot(doses(), Some("dose"), Some("response"));
    let (p, first) = with_layer(&p, points());
    let (_, second) = with_layer(
        &p,
        PlotLayer {
            size: Some(2.0),
            ..points()
        },
    );
    assert_eq!(first.notes, ["1 row with NA in dose or response dropped"]);
    assert_eq!(second.notes, first.notes);
    assert_eq!(second.panels[0].layers.len(), 2);
}

#[test]
fn one_variable_on_both_axes_is_named_once_in_the_note() {
    let p = plot(doses(), Some("dose"), Some("dose"));
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.notes, ["1 row with NA in dose dropped"]);
    assert_eq!(drawn.panels[0].layers[0].x, drawn.panels[0].layers[0].y);
}

#[test]
fn whole_numbers_and_truth_values_are_placed_as_numbers() {
    let p = plot(
        frame(
            "d",
            vec![
                ("n", Column::ints([Some(3), None, Some(-2), Some(40)])),
                (
                    "flag",
                    Column::bools([Some(true), Some(false), None, Some(true)]),
                ),
            ],
        ),
        Some("n"),
        Some("flag"),
    );
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.panels[0].layers[0].x, [3.0, 40.0]);
    assert_eq!(drawn.panels[0].layers[0].y, [1.0, 1.0]);
    assert_eq!(drawn.notes, ["2 rows with NA in n or flag dropped"]);
}

#[test]
fn the_largest_numbers_still_give_finite_axes() {
    let drawn = scatter(
        &some(&[-f64::MAX, f64::MAX, 0.0]),
        &some(&[f64::MAX, f64::MAX, f64::MAX]),
    );
    for s in &drawn.scales {
        assert!(s.limits.0.is_finite() && s.limits.1.is_finite(), "{s:?}");
        assert!(s.limits.0 < s.limits.1, "{s:?}");
        assert!(s.breaks.len() >= 2, "{s:?}");
        assert!(s.breaks.iter().all(|b| b.is_finite()), "{s:?}");
    }
    assert_eq!(drawn.scale("x").unwrap().limits, (-f64::MAX, f64::MAX));
    let _: Json = serde_json::from_str(&drawn.to_json()).expect("the scene is JSON");
}

#[test]
fn size_and_alpha_are_carried_to_the_layer() {
    let p = plot(doses(), Some("dose"), Some("response"));
    let (_, plain) = with_layer(&p, points());
    assert_eq!(plain.panels[0].layers[0].size, POINT_SIZE);
    assert_eq!(plain.panels[0].layers[0].alpha, 1.0);
    let (_, set) = with_layer(
        &p,
        PlotLayer {
            alpha: 0.25,
            size: Some(9.0),
            ..points()
        },
    );
    assert_eq!(set.panels[0].layers[0].size, 9.0);
    assert_eq!(set.panels[0].layers[0].alpha, 0.25);
}

#[test]
fn jitter_is_the_same_under_one_seed_and_differs_under_another() {
    let p = plot(
        frame(
            "d",
            vec![
                ("x", numbers(&[1.0, 1.0, 2.0, 2.0, 3.0])),
                ("y", numbers(&[5.0, 5.0, 6.0, 7.0, 7.0])),
            ],
        ),
        Some("x"),
        Some("y"),
    );
    let (extended, once) = with_layer(&p, jittered(0.4, 7));
    let (_, again) = with_layer(&p, jittered(0.4, 7));
    let (_, other) = with_layer(&p, jittered(0.4, 8));
    assert_eq!(once.to_json(), again.to_json());
    // The scene of a plot value is the same however often it is derived.
    assert_eq!(scene(&extended).to_json(), once.to_json());
    assert_ne!(once.panels[0].layers[0].x, other.panels[0].layers[0].x);
    // A share of 0 moves nothing, with or without a stream.
    let (_, still) = with_layer(&p, jittered(0.0, 7));
    assert_eq!(still.panels[0].layers[0].x, [1.0, 1.0, 2.0, 2.0, 3.0]);
}

// ----- a position mapped from an expression -----

/// The mapping of `aesthetic` from an expression written as `text`, with the values it gives on
/// the plot's frame, as `plot` hands them over.
fn computed(aesthetic: Aesthetic, text: &str, expr: ColumnExpr, values: Column) -> PlotMapping {
    PlotMapping {
        aesthetic,
        variable: text.to_string(),
        computed: Some(PlotComputed {
            expr,
            values: Rc::new(values),
        }),
    }
}

fn variable(aesthetic: Aesthetic, name: &str) -> PlotMapping {
    PlotMapping {
        aesthetic,
        variable: name.to_string(),
        computed: None,
    }
}

/// `response * 2` as `plot` keeps it.
fn doubled() -> ColumnExpr {
    ColumnExpr::Binary {
        op: zeff_core::operators::Op::Mul,
        lhs: Box::new(ColumnExpr::Var("response".to_string())),
        rhs: Box::new(ColumnExpr::Int(2)),
    }
}

#[test]
fn the_scene_of_a_mapped_expression() {
    let p = start(
        doses(),
        vec![
            variable(Aesthetic::X, "dose"),
            computed(
                Aesthetic::Y,
                "response * 2",
                doubled(),
                numbers(&[4.2, 7.8, 12.4, 15.6, 18.0]),
            ),
        ],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&p, points());
    // The fifth row has no dose; the limits are 5% of 15.6 - 4.2 beyond the values drawn.
    assert_eq!(drawn.panels[0].layers[0].y, [4.2, 7.8, 12.4, 15.6]);
    let y = drawn.scale("y").unwrap();
    assert_eq!(y.title, "response * 2");
    assert!((y.limits.0 - 3.63).abs() < 1e-12 && (y.limits.1 - 16.17).abs() < 1e-12);
    assert_eq!(
        drawn.notes,
        ["1 row with NA in dose or response * 2 dropped"]
    );
    insta::assert_snapshot!("expression", drawn.to_json());
}

#[test]
fn an_expression_is_the_title_of_its_axis_whatever_the_labels() {
    let p = start(
        doses(),
        vec![
            computed(
                Aesthetic::X,
                "dose + 0",
                ColumnExpr::Var("dose".to_string()),
                floats(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0), None]),
            ),
            variable(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    let before = scene(&p);
    assert_eq!(before.scale("x").unwrap().title, "dose + 0");
    // Before any layer the axis shows the values the expression has on the plot's frame.
    assert_eq!(before.scale("x").unwrap().limits, (0.85, 4.15));
    assert_eq!(p.summary.title, "plot · d · response against dose + 0");
}

#[test]
fn a_layer_with_its_own_frame_draws_the_expression_computed_there() {
    let p = start(
        doses(),
        vec![
            variable(Aesthetic::X, "dose"),
            computed(
                Aesthetic::Y,
                "response * 2",
                doubled(),
                numbers(&[4.2, 7.8, 12.4, 15.6, 18.0]),
            ),
        ],
        Vec::new(),
    );
    let own = frame(
        "other",
        vec![
            ("dose", numbers(&[10.0, 20.0, 30.0])),
            ("response", floats(&[Some(-1.0), None, Some(1.0)])),
        ],
    );
    let (_, drawn) = with_layer(
        &p,
        PlotLayer {
            data: Some(own.clone()),
            computed: vec![(
                Aesthetic::Y,
                Rc::new(floats(&[Some(-2.0), None, Some(2.0)])),
            )],
            ..points()
        },
    );
    assert_eq!(drawn.panels[0].layers[0].x, [10.0, 30.0]);
    assert_eq!(drawn.panels[0].layers[0].y, [-2.0, 2.0]);
    assert_eq!(
        drawn.notes,
        ["1 row with NA in dose or response * 2 dropped"]
    );
    // A layer handed no values for the expression draws nothing: the mark that adds a layer
    // computes them, or refuses the frame, before the scene is derived.
    let (_, empty) = with_layer(
        &p,
        PlotLayer {
            data: Some(own),
            ..points()
        },
    );
    assert!(empty.panels[0].layers[0].x.is_empty());
}

#[test]
fn a_constant_expression_and_one_with_no_value() {
    let x = variable(Aesthetic::X, "dose");
    let constant = start(
        doses(),
        vec![
            x.clone(),
            computed(
                Aesthetic::Y,
                "2 + 3",
                ColumnExpr::Int(5),
                Column::ints([Some(5); 5]),
            ),
        ],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&constant, points());
    assert_eq!(drawn.panels[0].layers[0].y, [5.0; 4]);
    assert_eq!(drawn.scale("y").unwrap().limits, (4.5, 5.5));
    let missing = start(
        doses(),
        vec![
            x,
            computed(Aesthetic::Y, "NA + 1", ColumnExpr::Na, floats(&[None; 5])),
        ],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&missing, points());
    assert!(drawn.panels[0].layers[0].y.is_empty());
    assert_eq!(drawn.scale("y").unwrap().limits, (0.0, 1.0));
    assert_eq!(drawn.notes, ["5 rows with NA in dose or NA + 1 dropped"]);
}

// ----- colour and fill (step 20c) -----

fn texts(values: &[Option<&str>]) -> Column {
    Column::strs(values.iter().copied())
}

/// A Category variable with these levels, each row by the place of its level counted from 0.
fn levels(levels: &[&str], codes: &[Option<i32>]) -> Column {
    Column::categories(
        levels.iter().map(|level| (*level).to_string()).collect(),
        codes.iter().copied(),
    )
    .unwrap()
}

/// A whole-number variable with value labels.
fn labelled(values: &[Option<i64>], labels: &[(&str, &str)]) -> Column {
    Column::ints(values.iter().copied())
        .with_entry(MetadataKey::ValueLabels, format_value_labels(labels))
}

/// The variables `x` and `y`, the numbers from 1 on and from 10 on, for `n` rows.
fn positions(n: usize) -> Vec<(&'static str, Column)> {
    let x: Vec<f64> = (0..n).map(|i| (i + 1) as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i + 10) as f64).collect();
    vec![("x", numbers(&x)), ("y", numbers(&y))]
}

/// A frame called `d` of `x`, `y` and one variable more, `g`.
fn with_g(g: Column) -> Frame {
    let mut columns = positions(g.len());
    columns.push(("g", g));
    frame("d", columns)
}

/// A plot of `frame` with `x` and `y` mapped to its variables of those names and `color` to
/// the variable named.
fn coloured(frame: Frame, color: &str) -> Plot {
    start(
        frame,
        vec![
            variable(Aesthetic::X, "x"),
            variable(Aesthetic::Y, "y"),
            variable(Aesthetic::Color, color),
        ],
        Vec::new(),
    )
}

/// The points of `d` with `x`, `y` and `g`, coloured by `g`.
fn by_g(g: Column) -> Scene {
    with_layer(&coloured(with_g(g), "g"), points()).1
}

/// An aesthetic set to one colour, as `plot` hands it over.
fn set(aesthetic: Aesthetic, written: &str) -> PlotConstant {
    PlotConstant {
        aesthetic,
        value: PlotSetting::Colour {
            hex: parse(written).unwrap(),
            written: written.to_string(),
        },
    }
}

/// The value of each mark of a layer on the scale `color`.
fn shades(drawn: &Scene, layer: usize) -> &[f64] {
    match &drawn.panels[0].layers[layer].color {
        Paint::Scaled(values) => values,
        other => panic!("the layer is not coloured from a scale: {other:?}"),
    }
}

/// The colours of a scale.
fn colours<'a>(drawn: &'a Scene, id: &str) -> &'a [&'static str] {
    match &drawn.scale(id).unwrap().range {
        Range::Colours(colours) => colours,
        Range::Position => panic!("`{id}` is a position"),
    }
}

fn own(data: Frame) -> PlotLayer {
    PlotLayer {
        data: Some(data),
        ..points()
    }
}

#[test]
fn the_scene_of_points_coloured_by_category() {
    let drawn = by_g(texts(&[Some("b"), Some("a"), None, Some("b"), Some("a")]));
    insta::assert_snapshot!("colour_category", drawn.to_json());
}

#[test]
fn the_scene_of_points_coloured_on_a_continuous_scale() {
    let drawn = by_g(floats(&[Some(0.5), Some(2.0), None, Some(8.0), Some(3.5)]));
    insta::assert_snapshot!("colour_ramp", drawn.to_json());
}

#[test]
fn the_scene_of_a_constant_colour_beside_a_mapped_fill() {
    let p = start(
        with_g(texts(&[Some("b"), Some("a"), None])),
        vec![
            variable(Aesthetic::X, "x"),
            variable(Aesthetic::Y, "y"),
            variable(Aesthetic::Fill, "g"),
        ],
        vec![set(Aesthetic::Color, "steelblue")],
    );
    let (_, drawn) = with_layer(&p, points());
    insta::assert_snapshot!("colour_constant", drawn.to_json());
}

#[test]
fn the_scene_of_a_coloured_plot_before_any_layer() {
    let p = coloured(
        with_g(levels(
            &["low", "middle", "high"],
            &[Some(2), Some(0), None, Some(2)],
        )),
        "g",
    );
    insta::assert_snapshot!("colour_no_layer", scene(&p).to_json());
}

#[test]
fn the_json_of_a_coloured_scene_reads_back() {
    let drawn = by_g(texts(&[Some("b"), Some("a"), None, Some("b"), Some("a")]));
    let text = drawn.to_json();
    let json: Json = serde_json::from_str(&text).expect("the scene is JSON");
    // A position has no range; a colour scale has one colour for each break.
    assert!(json["scales"][0]["range"].is_null());
    assert!(json["scales"][1]["range"].is_null());
    let scale = &json["scales"][2];
    assert_eq!(scale["id"].as_str(), Some("color"));
    assert_eq!(scale["kind"].as_str(), Some("category"));
    assert_eq!(scale["title"].as_str(), Some("g"));
    assert_eq!(bits(&list(&text, "limits", 2)), bits(&[1.0, 3.0]));
    assert_eq!(bits(&list(&text, "breaks", 2)), bits(&[1.0, 2.0, 3.0]));
    let read = |value: &Json| -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(read(&scale["labels"]), ["a", "b", "NA"]);
    assert_eq!(read(&scale["range"]), ["#E69F00", "#56B4E9", "#999999"]);
    let legends = json["guides"]["legends"].as_array().unwrap();
    assert_eq!(legends.len(), 1);
    assert_eq!(legends[0]["title"].as_str(), Some("g"));
    assert_eq!(read(&legends[0]["scales"]), ["color"]);
    assert_eq!(read(&legends[0]["marks"]), ["points"]);
    let layer = &json["panels"][0]["layers"][0];
    assert_eq!(
        bits(&list(&text, "color", 0)),
        bits(&[2.0, 1.0, 3.0, 2.0, 1.0])
    );
    assert_eq!(layer["color"].as_array().map(Vec::len), Some(5));
    // A constant is the colour itself, and a plot without a colour leaves it to the theme.
    let p = start(
        with_g(numbers(&[1.0, 2.0])),
        vec![variable(Aesthetic::X, "x"), variable(Aesthetic::Y, "y")],
        vec![set(Aesthetic::Color, "#48b")],
    );
    let constant: Json = serde_json::from_str(&with_layer(&p, points()).1.to_json()).unwrap();
    assert_eq!(
        constant["panels"][0]["layers"][0]["color"].as_str(),
        Some("#4488BB")
    );
    assert_eq!(constant["scales"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        constant["guides"]["legends"].as_array().map(Vec::len),
        Some(0)
    );
    let plain: Json =
        serde_json::from_str(&scatter(&some(&[1.0, 2.0]), &some(&[1.0, 2.0])).to_json()).unwrap();
    assert!(plain["panels"][0]["layers"][0]["color"].is_null());
}

#[test]
fn text_is_shown_as_categories_in_the_order_of_its_values() {
    let drawn = by_g(texts(&[
        Some("b"),
        Some("a"),
        Some("B"),
        Some("b"),
        Some("é"),
    ]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    // Text is ordered by code point (section 3.7).
    assert_eq!(scale.labels, ["B", "a", "b", "é"]);
    assert_eq!(scale.limits, (1.0, 4.0));
    assert_eq!(scale.breaks, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        colours(&drawn, "color"),
        ["#E69F00", "#56B4E9", "#009E73", "#F0E442"]
    );
    assert_eq!(shades(&drawn, 0), [3.0, 2.0, 1.0, 3.0, 4.0]);
    assert!(drawn.notes.is_empty());
}

#[test]
fn a_category_keeps_the_order_of_its_levels_and_a_level_with_no_row_is_left_out() {
    let drawn = by_g(levels(
        &["low", "middle", "high", "top"],
        &[Some(2), Some(0), Some(2), Some(0)],
    ));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.labels, ["low", "high"]);
    assert_eq!(scale.limits, (1.0, 2.0));
    assert_eq!(colours(&drawn, "color"), ["#E69F00", "#56B4E9"]);
    assert_eq!(shades(&drawn, 0), [2.0, 1.0, 2.0, 1.0]);
    // The levels are not sorted: the order is the variable's own.
    let drawn = by_g(levels(&["z", "m", "a"], &[Some(2), Some(1), Some(0)]));
    assert_eq!(drawn.scale("color").unwrap().labels, ["z", "m", "a"]);
    assert_eq!(shades(&drawn, 0), [3.0, 2.0, 1.0]);
}

#[test]
fn a_row_without_a_category_is_drawn_in_the_grey_of_na_which_comes_last() {
    let drawn = by_g(texts(&[None, Some("b"), Some("a"), None]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.labels, ["a", "b", "NA"]);
    assert_eq!(colours(&drawn, "color"), ["#E69F00", "#56B4E9", "#999999"]);
    // No row is dropped for it, so there is nothing to note.
    assert_eq!(drawn.panels[0].layers[0].x, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(shades(&drawn, 0), [3.0, 2.0, 1.0, 3.0]);
    assert!(drawn.notes.is_empty());
    // A variable of nothing but `NA` has that one category.
    let drawn = by_g(texts(&[None, None]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.labels, ["NA"]);
    assert_eq!(scale.limits, (1.0, 1.0));
    assert_eq!(colours(&drawn, "color"), ["#999999"]);
    assert_eq!(shades(&drawn, 0), [1.0, 1.0]);
    // The grey is not handed out while a row lacks its category but is not drawn.
    let mut columns = vec![
        ("x", floats(&[Some(1.0), None, Some(3.0)])),
        ("y", numbers(&[1.0, 2.0, 3.0])),
    ];
    columns.push(("g", texts(&[Some("a"), None, Some("b")])));
    let drawn = with_layer(&coloured(frame("d", columns), "g"), points()).1;
    assert_eq!(drawn.scale("color").unwrap().labels, ["a", "b"]);
    assert_eq!(shades(&drawn, 0), [1.0, 2.0]);
    assert_eq!(drawn.notes, ["1 row with NA in x or y dropped"]);
}

#[test]
fn truth_values_are_two_categories_false_first() {
    let drawn = by_g(Column::bools([Some(true), Some(false), None, Some(true)]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    assert_eq!(scale.labels, ["false", "true", "NA"]);
    assert_eq!(shades(&drawn, 0), [2.0, 1.0, 3.0, 2.0]);
}

#[test]
fn a_number_with_value_labels_is_shown_by_its_labels_in_the_order_of_its_values() {
    let drawn = by_g(
        labelled(
            &[Some(1), Some(0), Some(7), Some(1), None, Some(-3)],
            &[("1", "female"), ("0", "male")],
        )
        .with_entry(MetadataKey::Label, "Sex"),
    );
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    assert_eq!(scale.title, "Sex");
    // By value, not by label and not in the order the labels were written; a value without a
    // label is shown as it prints.
    assert_eq!(scale.labels, ["-3", "male", "female", "7", "NA"]);
    assert_eq!(shades(&drawn, 0), [3.0, 2.0, 4.0, 3.0, 5.0, 1.0]);
    assert_eq!(drawn.legends[0].title, "Sex");
    // Two values with one label are one category, at the place of the smaller.
    let drawn = by_g(labelled(
        &[Some(3), Some(2), Some(1), Some(2)],
        &[("1", "low"), ("2", "high"), ("3", "low")],
    ));
    assert_eq!(drawn.scale("color").unwrap().labels, ["low", "high"]);
    assert_eq!(shades(&drawn, 0), [1.0, 2.0, 1.0, 2.0]);
    // A Float with value labels too, the two zeros as one value.
    let drawn = by_g(
        floats(&[Some(0.0), Some(-0.0), Some(1.5), Some(f64::INFINITY)]).with_entry(
            MetadataKey::ValueLabels,
            format_value_labels(&[("0", "none")]),
        ),
    );
    assert_eq!(drawn.scale("color").unwrap().labels.len(), 3);
    assert_eq!(drawn.scale("color").unwrap().labels[0], "none");
    assert_eq!(shades(&drawn, 0), [1.0, 1.0, 2.0, 3.0]);
    // Labels that label nothing make no categories of numbers.
    let drawn = by_g(Column::ints([Some(1), Some(2)]).with_entry(
        MetadataKey::ValueLabels,
        format_value_labels::<&str, &str>(&[]),
    ));
    assert_eq!(drawn.scale("color").unwrap().kind, ScaleKind::Linear);
}

#[test]
fn more_categories_than_colours_repeat_the_palette_and_say_so() {
    let names: Vec<String> = (0..10).map(|i| format!("k{i}")).collect();
    let cells: Vec<Option<&str>> = names.iter().map(|n| Some(n.as_str())).collect();
    let p = coloured(with_g(texts(&cells)), "g");
    // Before a layer reads the colours there is nothing to say of them.
    let before = scene(&p);
    assert_eq!(before.scale("color").unwrap().labels.len(), 10);
    assert!(before.notes.is_empty());
    let (_, drawn) = with_layer(&p, points());
    let range = colours(&drawn, "color");
    assert_eq!(range.len(), 10);
    assert_eq!(
        range[..8],
        [
            "#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7", "#000000"
        ]
    );
    assert_eq!(range[8..], ["#E69F00", "#56B4E9"]);
    assert_eq!(
        shades(&drawn, 0),
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    );
    assert_eq!(
        drawn.notes,
        [
            "`g` has 10 categories and the palette 8 colours, so the colours repeat from the ninth category"
        ]
    );
    // A second layer does not say it twice.
    let (_, twice) = with_layer(&with_layer(&p, points()).0, points());
    assert_eq!(twice.notes, drawn.notes);
    // Eight categories and `NA` need no colour twice: `NA` has its own.
    let mut eight: Vec<Option<&str>> = cells[..8].to_vec();
    eight.push(None);
    let drawn = by_g(texts(&eight));
    assert_eq!(colours(&drawn, "color").len(), 9);
    assert_eq!(colours(&drawn, "color")[8], "#999999");
    assert!(drawn.notes.is_empty());
}

#[test]
fn the_colour_scale_shows_the_categories_drawn() {
    let p = coloured(with_g(texts(&[Some("a"), Some("b"), Some("c")])), "g");
    let other = || {
        let mut columns = positions(3);
        columns.push(("g", texts(&[Some("c"), Some("zz"), None])));
        frame("other", columns)
    };
    // The layer's own frame alone: `c` of the plot's frame, then its own `zz`, then `NA`; the
    // categories `a` and `b`, which no mark has, are not there.
    let (_, drawn) = with_layer(&p, own(other()));
    assert_eq!(drawn.scale("color").unwrap().labels, ["c", "zz", "NA"]);
    assert_eq!(colours(&drawn, "color"), ["#E69F00", "#56B4E9", "#999999"]);
    assert_eq!(shades(&drawn, 0), [1.0, 2.0, 3.0]);
    // With a layer of the plot's frame under it, the frame's categories come first, in their
    // order, and what only the other frame has follows.
    let (both, _) = with_layer(&p, points());
    let (_, drawn) = with_layer(&both, own(other()));
    assert_eq!(
        drawn.scale("color").unwrap().labels,
        ["a", "b", "c", "zz", "NA"]
    );
    assert_eq!(shades(&drawn, 0), [1.0, 2.0, 3.0]);
    assert_eq!(shades(&drawn, 1), [3.0, 4.0, 5.0]);
    // One legend, whatever the number of layers.
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].scales, ["color"]);
    assert_eq!(drawn.legends[0].marks, [Mark::Points]);
}

#[test]
fn a_layer_whose_frame_holds_the_colour_as_another_kind_is_left_to_the_theme() {
    // `points` refuses such a frame; the scene, given one, colours the other layers and leaves
    // this one alone.
    let p = coloured(with_g(texts(&[Some("a"), Some("b")])), "g");
    let (p, _) = with_layer(&p, points());
    let mut columns = positions(2);
    columns.push(("g", numbers(&[1.0, 2.0])));
    let (_, drawn) = with_layer(&p, own(frame("other", columns)));
    assert_eq!(shades(&drawn, 0), [1.0, 2.0]);
    assert_eq!(drawn.panels[0].layers[1].color, Paint::Theme);
    assert_eq!(drawn.panels[0].layers[1].x, [1.0, 2.0]);
    assert_eq!(drawn.scale("color").unwrap().labels, ["a", "b"]);
}

#[test]
fn a_number_is_shown_on_a_ramp_from_its_smallest_value_drawn_to_its_largest() {
    let drawn = by_g(Column::ints([Some(30), Some(12), Some(47), Some(20)]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.kind, ScaleKind::Linear);
    // No margin: a colour beyond the data would never be seen.
    assert_eq!(scale.limits, (12.0, 47.0));
    assert_eq!(scale.breaks, [15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0]);
    assert_eq!(scale.labels, ["15", "20", "25", "30", "35", "40", "45"]);
    let range = colours(&drawn, "color");
    assert_eq!(range.len(), 256);
    assert_eq!(range[0], "#440154");
    assert_eq!(range[255], "#FDE725");
    assert_eq!(shades(&drawn, 0), [30.0, 12.0, 47.0, 20.0]);
    assert_eq!(drawn.legends[0].scales, ["color"]);
    // The ends take the ends of the ramp, by the rule the writers follow.
    assert_eq!(ramp_entry(12.0, scale.limits.0, scale.limits.1, 256), 0);
    assert_eq!(ramp_entry(47.0, scale.limits.0, scale.limits.1, 256), 255);
}

#[test]
fn a_row_without_a_finite_value_on_the_ramp_is_not_drawn_and_is_counted() {
    let mut columns = vec![
        (
            "x",
            floats(&[Some(1.0), None, Some(3.0), Some(4.0), Some(5.0), Some(6.0)]),
        ),
        ("y", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
    ];
    columns.push((
        "g",
        floats(&[
            Some(10.0),
            Some(1000.0),
            None,
            Some(f64::NEG_INFINITY),
            Some(-2.5),
            Some(4.0),
        ]),
    ));
    let drawn = with_layer(&coloured(frame("d", columns), "g"), points()).1;
    assert_eq!(drawn.panels[0].layers[0].x, [1.0, 5.0, 6.0]);
    assert_eq!(shades(&drawn, 0), [10.0, -2.5, 4.0]);
    assert_eq!(
        drawn.notes,
        [
            "2 rows with NA in x, y or g dropped",
            "1 row with an infinite value in x, y or g dropped"
        ]
    );
    // The value of a row that is not drawn does not stretch the ramp.
    assert_eq!(drawn.scale("color").unwrap().limits, (-2.5, 10.0));
    // A variable that is a position too is named once.
    let p = coloured(with_g(numbers(&[1.0, 2.0])), "x");
    let (_, drawn) = with_layer(
        &Plot {
            frame: frame(
                "d",
                vec![
                    ("x", floats(&[Some(1.0), None, Some(3.0)])),
                    ("y", numbers(&[1.0, 2.0, 3.0])),
                ],
            ),
            ..p
        },
        points(),
    );
    assert_eq!(drawn.notes, ["1 row with NA in x or y dropped"]);
    assert_eq!(shades(&drawn, 0), drawn.panels[0].layers[0].x);
}

#[test]
fn a_ramp_of_one_value_and_of_none() {
    // One value: half a unit on each side, as a position has, and the value in the middle.
    let drawn = by_g(numbers(&[3.0, 3.0]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.limits, (2.5, 3.5));
    assert_eq!(ramp_entry(3.0, 2.5, 3.5, 256), 128);
    // No value: the unit scale, and nothing drawn.
    let drawn = by_g(floats(&[None, None]));
    assert_eq!(drawn.scale("color").unwrap().limits, (0.0, 1.0));
    assert!(drawn.panels[0].layers[0].x.is_empty());
    assert_eq!(shades(&drawn, 0), [0.0_f64; 0]);
    assert_eq!(drawn.notes, ["2 rows with NA in x, y or g dropped"]);
    // An empty frame: the unit ramp, and categories of nothing.
    let drawn = by_g(floats(&[]));
    assert_eq!(drawn.scale("color").unwrap().limits, (0.0, 1.0));
    assert!(drawn.notes.is_empty());
    let drawn = by_g(texts(&[]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    assert_eq!(scale.limits, (1.0, 1.0));
    assert!(scale.breaks.is_empty() && scale.labels.is_empty());
    assert!(colours(&drawn, "color").is_empty());
    assert_eq!(shades(&drawn, 0), [0.0_f64; 0]);
    // The two zeros are one value: the ramp is the same text whichever comes first.
    let one = by_g(numbers(&[-0.0, 0.0, 1.0]));
    let other = by_g(numbers(&[0.0, -0.0, 1.0]));
    assert_eq!(
        one.to_json()
            .lines()
            .filter(|l| l.contains("limits"))
            .collect::<Vec<_>>(),
        other
            .to_json()
            .lines()
            .filter(|l| l.contains("limits"))
            .collect::<Vec<_>>()
    );
    assert_eq!(bits(&[one.scale("color").unwrap().limits.0]), bits(&[0.0]));
    let drawn = by_g(numbers(&[-f64::MAX, f64::MAX, 0.0]));
    let scale = drawn.scale("color").unwrap();
    assert_eq!(scale.limits, (-f64::MAX, f64::MAX));
    assert!(scale.breaks.iter().all(|b| b.is_finite()));
}

#[test]
fn before_any_layer_a_colour_scale_shows_what_the_frame_holds_and_has_no_legend() {
    let p = coloured(with_g(texts(&[Some("b"), None, Some("a")])), "g");
    let before = scene(&p);
    assert_eq!(before.scale("color").unwrap().labels, ["a", "b", "NA"]);
    assert!(before.legends.is_empty());
    assert!(before.panels[0].layers.is_empty());
    let p = coloured(with_g(floats(&[Some(4.0), None, Some(-1.0)])), "g");
    let before = scene(&p);
    assert_eq!(before.scale("color").unwrap().limits, (-1.0, 4.0));
    assert!(before.legends.is_empty());
    assert!(before.notes.is_empty());
    // A layer of rows that cannot be drawn leaves the scale as the frame has it.
    let mut columns = vec![("x", floats(&[None, None])), ("y", numbers(&[1.0, 2.0]))];
    columns.push(("g", texts(&[Some("a"), Some("b")])));
    let drawn = with_layer(&coloured(frame("d", columns), "g"), points()).1;
    assert_eq!(drawn.scale("color").unwrap().labels, ["a", "b"]);
    assert_eq!(drawn.legends.len(), 1);
}

#[test]
fn fill_has_its_scale_but_no_point_reads_it() {
    let mut columns = positions(4);
    columns.push(("g", texts(&[Some("a"), Some("b"), None, Some("a")])));
    columns.push((
        "z",
        floats(&[Some(1.0), None, Some(f64::INFINITY), Some(9.0)]),
    ));
    let build = |fill: &str, constants: Vec<PlotConstant>| {
        let p = start(
            frame("d", columns.clone()),
            vec![
                variable(Aesthetic::X, "x"),
                variable(Aesthetic::Y, "y"),
                variable(Aesthetic::Fill, fill),
            ],
            constants,
        );
        with_layer(&p, points()).1
    };
    let drawn = build("g", Vec::new());
    let scale = drawn.scale("fill").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    assert_eq!(scale.labels, ["a", "b", "NA"]);
    assert!(drawn.scale("color").is_none());
    // No mark reads it: no legend, the colour of the theme, and no row dropped for it.
    assert!(drawn.legends.is_empty());
    assert_eq!(drawn.panels[0].layers[0].color, Paint::Theme);
    let drawn = build("z", vec![set(Aesthetic::Color, "tomato")]);
    assert_eq!(drawn.scale("fill").unwrap().limits, (1.0, 9.0));
    assert_eq!(drawn.panels[0].layers[0].x.len(), 4);
    assert!(drawn.notes.is_empty());
    assert!(drawn.legends.is_empty());
    assert_eq!(
        drawn.panels[0].layers[0].color,
        Paint::Constant("#FF6347".to_string())
    );
    // A constant fill is kept with the plot and changes nothing a point draws.
    let p = start(
        frame("d", columns.clone()),
        vec![variable(Aesthetic::X, "x"), variable(Aesthetic::Y, "y")],
        vec![set(Aesthetic::Fill, "tomato")],
    );
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.panels[0].layers[0].color, Paint::Theme);
    assert_eq!(drawn.scales.len(), 2);
}

#[test]
fn color_and_fill_of_one_variable_are_two_scales() {
    let mut columns = positions(3);
    columns.push(("g", texts(&[Some("a"), Some("b"), Some("a")])));
    let p = start(
        frame("d", columns),
        vec![
            variable(Aesthetic::X, "x"),
            variable(Aesthetic::Y, "y"),
            variable(Aesthetic::Color, "g"),
            variable(Aesthetic::Fill, "g"),
        ],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&p, points());
    let ids: Vec<&str> = drawn.scales.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["x", "y", "color", "fill"]);
    assert_eq!(colours(&drawn, "color"), colours(&drawn, "fill"));
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].scales, ["color"]);
}

#[test]
fn jitter_moves_the_points_and_leaves_their_colours() {
    let g = texts(&[Some("b"), Some("a"), None, Some("b")]);
    let p = coloured(with_g(g), "g");
    let (_, still) = with_layer(&p, points());
    let (_, moved) = with_layer(&p, jittered(0.4, 11));
    assert_ne!(moved.panels[0].layers[0].x, still.panels[0].layers[0].x);
    assert_eq!(shades(&moved, 0), shades(&still, 0));
    assert_eq!(moved.scale("color"), still.scale("color"));
}

#[test]
fn how_each_type_is_shown_as_a_colour() {
    assert_eq!(shading(&texts(&[Some("a")])), Some(Shading::Categories));
    assert_eq!(
        shading(&Column::bools([Some(true)])),
        Some(Shading::Categories)
    );
    assert_eq!(
        shading(&levels(&["a"], &[Some(0)])),
        Some(Shading::Categories)
    );
    assert_eq!(
        shading(&labelled(&[Some(1)], &[("1", "one")])),
        Some(Shading::Categories)
    );
    assert_eq!(shading(&Column::ints([Some(1)])), Some(Shading::Ramp));
    assert_eq!(shading(&numbers(&[1.0])), Some(Shading::Ramp));
    assert_eq!(shading(&Column::dates([Some(0)])), None);
    assert_eq!(shading(&Column::datetimes([Some(0)])), None);
    // Every point reads `color`, and none `fill`.
    assert!(reads(Mark::Points, Aesthetic::Color));
    assert!(!reads(Mark::Points, Aesthetic::Fill));
}

// ----- the properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A cell: `NA`, a small whole number, an everyday number, any double, or an awkward one.
fn cell() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        1 => Just(None),
        4 => (-5i32..6).prop_map(|i| Some(f64::from(i))),
        3 => (-1000.0f64..1000.0).prop_map(Some),
        1 => prop::num::f64::NORMAL.prop_map(Some),
        1 => prop_oneof![
            Just(f64::INFINITY),
            Just(f64::NEG_INFINITY),
            Just(0.0),
            Just(-0.0),
            Just(f64::MAX),
            Just(-f64::MAX),
            Just(f64::MIN_POSITIVE),
            Just(5e-324)
        ]
        .prop_map(Some),
    ]
}

/// An everyday cell, for the properties of jitter: no value so large that a gap overflows.
fn plain_cell() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        1 => Just(None),
        4 => (-5i32..6).prop_map(|i| Some(f64::from(i))),
        3 => (-1000.0f64..1000.0).prop_map(Some),
    ]
}

fn rows() -> impl Strategy<Value = Vec<(Option<f64>, Option<f64>)>> {
    prop::collection::vec((cell(), cell()), 0..40)
}

fn columns(rows: &[(Option<f64>, Option<f64>)]) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    (
        rows.iter().map(|r| r.0).collect(),
        rows.iter().map(|r| r.1).collect(),
    )
}

/// The marks of a layer as pairs of bit patterns, sorted: the marks whatever their order.
fn marks(drawn: &Scene, layer: usize) -> Vec<(u64, u64)> {
    let l = &drawn.panels[0].layers[layer];
    let mut pairs: Vec<(u64, u64)> =
        l.x.iter()
            .zip(&l.y)
            .map(|(x, y)| (x.to_bits(), y.to_bits()))
            .collect();
    pairs.sort_unstable();
    pairs
}

/// The smallest gap between two different values, as the page of `points` defines it.
fn smallest_gap(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|gap| *gap > 0.0)
        .fold(None, |least: Option<f64>, gap| {
            Some(least.map_or(gap, |l| l.min(gap)))
        })
        .unwrap_or(1.0)
}

/// A row of a coloured plot: the two positions, one of twelve categories or none, and a number
/// for a ramp.
type Painted = (Option<f64>, Option<f64>, Option<u8>, Option<f64>);

fn painted_rows() -> impl Strategy<Value = Vec<Painted>> {
    prop::collection::vec(
        (
            cell(),
            cell(),
            prop::option::weighted(0.85, 0u8..12),
            cell(),
        ),
        0..40,
    )
}

/// The frame `d` of such rows: `x`, `y`, the category `g` as text and the number `z`.
fn painted_frame(rows: &[Painted]) -> Frame {
    let x: Vec<Option<f64>> = rows.iter().map(|r| r.0).collect();
    let y: Vec<Option<f64>> = rows.iter().map(|r| r.1).collect();
    let g: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.2.map(|k| format!("k{k:02}")))
        .collect();
    let z: Vec<Option<f64>> = rows.iter().map(|r| r.3).collect();
    frame(
        "d",
        vec![
            ("x", floats(&x)),
            ("y", floats(&y)),
            ("g", Column::strs(g)),
            ("z", floats(&z)),
        ],
    )
}

/// The marks of a layer with their colours as bit patterns, sorted: the marks whatever their
/// order.
fn painted_marks(drawn: &Scene, layer: usize) -> Vec<(u64, u64, u64)> {
    let l = &drawn.panels[0].layers[layer];
    let shade = shades(drawn, layer);
    assert_eq!(shade.len(), l.x.len());
    let mut all: Vec<(u64, u64, u64)> = (0..l.x.len())
        .map(|i| (l.x[i].to_bits(), l.y[i].to_bits(), shade[i].to_bits()))
        .collect();
    all.sort_unstable();
    all
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    /// Step 20c: a coloured plot gives the same scene however often it is derived, and the
    /// scene is JSON.
    #[test]
    fn the_same_data_give_the_same_coloured_scene(
        rows in painted_rows(), share in 0.0f64..1.0, seed in 0u64..1000, ramp in any::<bool>()
    ) {
        let build = || {
            let p = coloured(painted_frame(&rows), if ramp { "z" } else { "g" });
            let (p, _) = with_layer(&p, points());
            with_layer(&p, jittered(share, seed))
        };
        let (first_plot, first) = build();
        let (_, second) = build();
        prop_assert_eq!(first.to_json(), second.to_json());
        prop_assert_eq!(scene(&first_plot).to_json(), first.to_json());
        let read: Result<Json, _> = serde_json::from_str(&first.to_json());
        prop_assert!(read.is_ok(), "{}", first.to_json());
        // Jitter moves a point and leaves its colour.
        prop_assert_eq!(shades(&first, 0), shades(&first, 1));
    }

    /// Step 20c: reordering the rows reorders the marks, each with its colour, and changes
    /// nothing else: the scale of colours does not depend on which row comes first.
    #[test]
    fn reordering_the_rows_carries_each_colour_with_its_mark(
        (rows, shuffled) in painted_rows()
            .prop_flat_map(|rows| (Just(rows.clone()), Just(rows).prop_shuffle())),
        ramp in any::<bool>(),
    ) {
        let by = if ramp { "z" } else { "g" };
        let a = with_layer(&coloured(painted_frame(&rows), by), points()).1;
        let b = with_layer(&coloured(painted_frame(&shuffled), by), points()).1;
        prop_assert_eq!(&a.scales, &b.scales);
        prop_assert_eq!(&a.legends, &b.legends);
        prop_assert_eq!(&a.notes, &b.notes);
        prop_assert_eq!(painted_marks(&a, 0), painted_marks(&b, 0));
        // Everything of the scene but the lists of marks is the same text.
        let without_marks = |s: &Scene| -> String {
            s.to_json()
                .lines()
                .filter(|l| {
                    let l = l.trim_start();
                    !(l.starts_with("\"x\": [") || l.starts_with("\"y\": [") || l.starts_with("\"color\": ["))
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        prop_assert_eq!(without_marks(&a), without_marks(&b));
        // And before any layer the scene is the same text.
        prop_assert_eq!(
            scene(&coloured(painted_frame(&rows), by)).to_json(),
            scene(&coloured(painted_frame(&shuffled), by)).to_json()
        );
    }

    /// Step 20c: every mark has a colour on the scale. On a scale of categories it is the place
    /// of a break, every break is the colour of some mark, a category has one place and two
    /// categories two; on a ramp it lies within the limits. No row is dropped for its category,
    /// and a row is dropped for its place on a ramp when that is `NA` or infinite.
    #[test]
    fn every_colour_lies_on_its_scale(rows in painted_rows()) {
        let complete = |r: &Painted| matches!((r.0, r.1), (Some(a), Some(b)) if a.is_finite() && b.is_finite());
        // Categories.
        let drawn = with_layer(&coloured(painted_frame(&rows), "g"), points()).1;
        let s = drawn.scale("color").unwrap();
        let total = s.breaks.len();
        prop_assert_eq!(s.kind, ScaleKind::Category);
        prop_assert_eq!(s.labels.len(), total);
        prop_assert_eq!(colours(&drawn, "color").len(), total);
        prop_assert_eq!(s.limits, (1.0, total.max(1) as f64));
        prop_assert!(s.breaks.iter().enumerate().all(|(i, b)| *b == (i + 1) as f64), "{:?}", s);
        let kept: Vec<&Painted> = rows.iter().filter(|r| complete(r)).collect();
        let shade = shades(&drawn, 0);
        prop_assert_eq!(shade.len(), kept.len());
        prop_assert_eq!(drawn.panels[0].layers[0].x.len(), kept.len());
        for (value, row) in shade.iter().zip(&kept) {
            let place = *value as usize;
            prop_assert!(place >= 1 && place <= total && place as f64 == *value, "{value} {:?}", s);
            let label = row.2.map_or("NA".to_string(), |k| format!("k{k:02}"));
            prop_assert_eq!(&s.labels[place - 1], &label);
            let expected = if row.2.is_some() { OKABE_ITO[(place - 1) % 8].0 } else { NA_COLOUR.0 };
            prop_assert_eq!(colours(&drawn, "color")[place - 1], expected);
        }
        if !kept.is_empty() {
            for place in 1..=total {
                prop_assert!(shade.contains(&(place as f64)), "no mark at {place}: {:?}", s);
            }
        }
        prop_assert!(s.labels.iter().enumerate().all(|(i, l)| !s.labels[..i].contains(l)), "{:?}", s);
        // A ramp.
        let drawn = with_layer(&coloured(painted_frame(&rows), "z"), points()).1;
        let s = drawn.scale("color").unwrap();
        let (lo, hi) = s.limits;
        prop_assert_eq!(s.kind, ScaleKind::Linear);
        prop_assert!(lo.is_finite() && hi.is_finite() && lo < hi, "{:?}", s);
        prop_assert!(s.breaks.iter().all(|b| lo <= *b && *b <= hi), "{:?}", s);
        prop_assert_eq!(s.labels.len(), s.breaks.len());
        prop_assert_eq!(colours(&drawn, "color").len(), 256);
        let kept: Vec<f64> = rows
            .iter()
            .filter(|r| complete(r))
            .filter_map(|r| r.3.filter(|z| z.is_finite()))
            .collect();
        prop_assert_eq!(bits(shades(&drawn, 0)), bits(&kept));
        prop_assert_eq!(drawn.panels[0].layers[0].x.len(), kept.len());
        prop_assert!(kept.iter().all(|z| lo <= *z && *z <= hi), "{:?}", s);
        prop_assert!(kept.iter().all(|z| ramp_entry(*z, lo, hi, 256) < 256));
        // With two values or more the ramp runs exactly from the smallest to the largest.
        let least = kept.iter().copied().fold(f64::INFINITY, f64::min);
        let most = kept.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if least < most {
            prop_assert_eq!((lo, hi), (least, most));
            prop_assert_eq!(ramp_entry(least, lo, hi, 256), 0);
            prop_assert_eq!(ramp_entry(most, lo, hi, 256), 255);
        }
    }

    #[test]
    fn the_same_data_give_the_same_scene(rows in rows(), share in 0.0f64..1.0, seed in 0u64..1000) {
        let (x, y) = columns(&rows);
        let build = || {
            let p = plot(frame("d", vec![("x", floats(&x)), ("y", floats(&y))]), Some("x"), Some("y"));
            let (p, _) = with_layer(&p, points());
            with_layer(&p, jittered(share, seed))
        };
        let (first_plot, first) = build();
        let (_, second) = build();
        prop_assert_eq!(first.to_json(), second.to_json());
        prop_assert_eq!(scene(&first_plot).to_json(), first.to_json());
        let read: Result<Json, _> = serde_json::from_str(&first.to_json());
        prop_assert!(read.is_ok(), "{}", first.to_json());
    }

    #[test]
    fn reordering_the_rows_changes_only_the_order_of_the_marks(
        (rows, shuffled) in rows().prop_flat_map(|rows| (Just(rows.clone()), Just(rows).prop_shuffle()))
    ) {
        let (x, y) = columns(&rows);
        let (sx, sy) = columns(&shuffled);
        let a = scatter(&x, &y);
        let b = scatter(&sx, &sy);
        prop_assert_eq!(&a.scales, &b.scales);
        prop_assert_eq!(&a.axes, &b.axes);
        prop_assert_eq!(&a.notes, &b.notes);
        prop_assert_eq!(marks(&a, 0), marks(&b, 0));
        // Everything of the scene but the lists of marks is the same text.
        let without_marks = |s: &Scene| -> String {
            s.to_json()
                .lines()
                .filter(|l| !(l.trim_start().starts_with("\"x\": [") || l.trim_start().starts_with("\"y\": [")))
                .collect::<Vec<_>>()
                .join("\n")
        };
        prop_assert_eq!(without_marks(&a), without_marks(&b));
    }

    #[test]
    fn every_break_and_every_mark_lies_within_the_limits(
        rows in rows(), share in 0.0f64..2.0, seed in 0u64..1000
    ) {
        let (x, y) = columns(&rows);
        let p = plot(frame("d", vec![("x", floats(&x)), ("y", floats(&y))]), Some("x"), Some("y"));
        let (p, plain) = with_layer(&p, points());
        let (_, moved) = with_layer(&p, jittered(share, seed));
        for drawn in [&plain, &moved] {
            for s in &drawn.scales {
                let (lo, hi) = s.limits;
                prop_assert!(lo.is_finite() && hi.is_finite() && lo < hi, "{:?}", s);
                prop_assert!(s.breaks.len() >= 2 && s.breaks.len() <= 50, "{:?}", s);
                prop_assert!(s.breaks.iter().all(|b| lo <= *b && *b <= hi), "{:?}", s);
                prop_assert!(s.breaks.windows(2).all(|w| w[0] < w[1]), "{:?}", s);
                prop_assert_eq!(s.labels.len(), s.breaks.len());
                prop_assert!(s.labels.windows(2).all(|w| w[0] != w[1]), "{:?}", s);
            }
            let (x_lo, x_hi) = drawn.scale("x").unwrap().limits;
            let (y_lo, y_hi) = drawn.scale("y").unwrap().limits;
            for layer in &drawn.panels[0].layers {
                prop_assert_eq!(layer.x.len(), layer.y.len());
                prop_assert!(layer.x.iter().all(|v| x_lo <= *v && *v <= x_hi), "{:?}", layer);
                prop_assert!(layer.y.iter().all(|v| y_lo <= *v && *v <= y_hi), "{:?}", layer);
            }
        }
        // The marks drawn and the rows counted in the notes are all the rows.
        let complete = rows.iter().filter(|(a, b)| matches!((a, b), (Some(a), Some(b)) if a.is_finite() && b.is_finite())).count();
        prop_assert_eq!(plain.panels[0].layers[0].x.len(), complete);
        prop_assert_eq!(moved.panels[0].layers[1].x.len(), complete);
    }

    #[test]
    fn jitter_moves_a_point_by_no_more_than_its_share_of_the_gap(
        rows in prop::collection::vec((plain_cell(), plain_cell()), 0..40),
        share in 0.0f64..1.0,
        seed in 0u64..1000,
    ) {
        let (x, y) = columns(&rows);
        let p = plot(frame("d", vec![("x", floats(&x)), ("y", floats(&y))]), Some("x"), Some("y"));
        let (_, plain) = with_layer(&p, points());
        let (_, moved) = with_layer(&p, jittered(share, seed));
        let still = &plain.panels[0].layers[0];
        let layer = &moved.panels[0].layers[0];
        prop_assert_eq!(layer.x.len(), still.x.len());
        let reach_x = share * smallest_gap(&still.x);
        let reach_y = share * smallest_gap(&still.y);
        for i in 0..still.x.len() {
            // The bound, with room for the rounding of one sum.
            prop_assert!((layer.x[i] - still.x[i]).abs() <= reach_x * (1.0 + 1e-12) + 1e-9);
            prop_assert!((layer.y[i] - still.y[i]).abs() <= reach_y * (1.0 + 1e-12) + 1e-9);
        }
    }

    /// Step 20b: an expression that is one variable gives the scene of that variable, whatever
    /// the values (missing and infinite ones among them), under jitter and in a layer that has
    /// a frame of its own.
    #[test]
    fn an_expression_that_is_one_variable_gives_the_scene_of_that_variable(
        rows in rows(),
        other in rows(),
        share in 0.0f64..1.0,
        seed in 0u64..1000,
        on_x in any::<bool>(),
        on_y in any::<bool>(),
    ) {
        let (x, y) = columns(&rows);
        let (own_x, own_y) = columns(&other);
        let own = || frame("e", vec![("x", floats(&own_x)), ("y", floats(&own_y))]);
        let layers = |p: &Plot, handed: Vec<(Aesthetic, Rc<Column>)>| {
            let (p, _) = with_layer(p, points());
            let (p, _) = with_layer(&p, jittered(share, seed));
            with_layer(&p, PlotLayer { data: Some(own()), computed: handed, ..points() }).1
        };
        let plain = plot(frame("d", vec![("x", floats(&x)), ("y", floats(&y))]), Some("x"), Some("y"));
        let mapping = |aesthetic, name: &str, values: &[Option<f64>], as_expression: bool| {
            if as_expression {
                computed(aesthetic, name, ColumnExpr::Var(name.to_string()), floats(values))
            } else {
                variable(aesthetic, name)
            }
        };
        let mapped = start(
            frame("d", vec![("x", floats(&x)), ("y", floats(&y))]),
            vec![
                mapping(Aesthetic::X, "x", &x, on_x),
                mapping(Aesthetic::Y, "y", &y, on_y),
            ],
            Vec::new(),
        );
        let mut handed = Vec::new();
        if on_x {
            handed.push((Aesthetic::X, Rc::new(floats(&own_x))));
        }
        if on_y {
            handed.push((Aesthetic::Y, Rc::new(floats(&own_y))));
        }
        prop_assert_eq!(scene(&mapped).to_json(), scene(&plain).to_json());
        prop_assert_eq!(layers(&mapped, handed).to_json(), layers(&plain, Vec::new()).to_json());
    }
}
