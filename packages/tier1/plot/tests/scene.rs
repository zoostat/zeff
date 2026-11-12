//! The scene of a plot (Language Reference, section 7.6, "Rendering and export"; Engine and
//! Project Specification, section 8.3, point 5; release 0.7.0, step 20): snapshots of the scene
//! of small plots as JSON, the JSON read back value for value, the edge cases, and the
//! properties: the same data give the same scene, reordering the rows changes nothing but the
//! order of the marks, every break and every mark lies within the limits, and jitter moves a
//! point by no more than its share of the gap. From step 20b, a position mapped from an
//! expression: its values are handed to the scene with the plot, so the scene shows them under
//! the expression as written, and an expression that is one variable gives the scene of that
//! variable. From step 20d, the scales of size and shape: a size laid out by area from 0, a
//! shape for each category, a constant beside a mapping, one legend for a variable, and the
//! properties that every size and every shape lies on its scale and travels with its mark.

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
    LARGEST_SIZE, POINT_SIZE, Paint, Range, SCENE_VERSION, ScaleKind, Scene, Shading, Shaping,
    Sizing, diameter, layer_reads, reads, scene, shading, start, with_layer,
};
use zeff_plot::shape;

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
        nudge: (0.0, 0.0),
        width: None,
        stream: None,
        computed: Vec::new(),
        mappings: Vec::new(),
        constants: Vec::new(),
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
    assert_eq!(plain.panels[0].layers[0].size, Sizing::Constant(POINT_SIZE));
    assert_eq!(plain.panels[0].layers[0].shape, Shaping::Constant("circle"));
    assert_eq!(plain.panels[0].layers[0].alpha, 1.0);
    let (_, set) = with_layer(
        &p,
        PlotLayer {
            alpha: 0.25,
            size: Some(9.0),
            ..points()
        },
    );
    assert_eq!(set.panels[0].layers[0].size, Sizing::Constant(9.0));
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
        other => panic!("`{id}` is not a scale of colours: {other:?}"),
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

// ----- size, shape, group and label (step 20d) -----

/// `size` set to one diameter for every mark, as `plot` hands it over.
fn all_sized(across: f64) -> PlotConstant {
    PlotConstant {
        aesthetic: Aesthetic::Size,
        value: PlotSetting::Size(across),
    }
}

/// `shape` set to one shape for every mark, as `plot` hands it over.
fn all_shaped(name: &str) -> PlotConstant {
    PlotConstant {
        aesthetic: Aesthetic::Shape,
        value: PlotSetting::Shape(shape::parse(name).unwrap()),
    }
}

/// A frame of `x`, `y`, the text `g` and the number `z`.
fn with_g_and_z(name: &str, g: &[Option<&str>], z: &[Option<f64>]) -> Frame {
    let mut columns = positions(g.len());
    columns.push(("g", texts(g)));
    columns.push(("z", floats(z)));
    frame(name, columns)
}

/// A plot of `frame` with `x` and `y` mapped to its variables of those names, and each of these
/// aesthetics to the variable named beside it.
fn plot_of(frame: Frame, more: &[(Aesthetic, &str)], constants: Vec<PlotConstant>) -> Plot {
    let mut mappings = vec![variable(Aesthetic::X, "x"), variable(Aesthetic::Y, "y")];
    mappings.extend(
        more.iter()
            .map(|(aesthetic, name)| variable(*aesthetic, name)),
    );
    start(frame, mappings, constants)
}

/// A plot of `d`, with `x`, `y`, a `g` of one category and `z`, sized by `z`.
fn sized_by_z(z: &[Option<f64>]) -> Plot {
    let g = vec![Some("a"); z.len()];
    plot_of(
        with_g_and_z("d", &g, z),
        &[(Aesthetic::Size, "z")],
        Vec::new(),
    )
}

/// The points of that plot.
fn by_z(z: &[Option<f64>]) -> Scene {
    with_layer(&sized_by_z(z), points()).1
}

/// A plot of `d`, with `x`, `y`, `g` and a `z` of 1 in every row, shaped by `g`.
fn shaped_by_g(g: &[Option<&str>]) -> Plot {
    let z = vec![Some(1.0); g.len()];
    plot_of(
        with_g_and_z("d", g, &z),
        &[(Aesthetic::Shape, "g")],
        Vec::new(),
    )
}

/// The value of each mark of a layer on the scale `size`.
fn sizes(drawn: &Scene, layer: usize) -> &[f64] {
    match &drawn.panels[0].layers[layer].size {
        Sizing::Scaled(values) => values,
        other => panic!("the layer is not sized from a scale: {other:?}"),
    }
}

/// The place of each mark of a layer on the scale `shape`.
fn places(drawn: &Scene, layer: usize) -> &[f64] {
    match &drawn.panels[0].layers[layer].shape {
        Shaping::Scaled(values) => values,
        other => panic!("the layer is not shaped from a scale: {other:?}"),
    }
}

/// The shapes of the scale `shape`.
fn shapes(drawn: &Scene) -> &[&'static str] {
    match &drawn.scale("shape").unwrap().range {
        Range::Shapes(names) => names,
        other => panic!("`shape` is not a scale of shapes: {other:?}"),
    }
}

/// A layer of points with a `size:` of its own.
fn points_of_size(across: f64) -> PlotLayer {
    PlotLayer {
        size: Some(across),
        ..points()
    }
}

#[test]
fn a_size_is_laid_out_by_area() {
    let range = (0.0, LARGEST_SIZE);
    let limits = (0.0, 16.0);
    // The ends of the scale are the ends of the range.
    assert_eq!(diameter(0.0, limits, range), 0.0);
    assert_eq!(diameter(16.0, limits, range), 18.0);
    // A quarter of the value is a quarter of the area, which is half the width, and four times
    // a value is twice its width.
    assert_eq!(diameter(4.0, limits, range), 9.0);
    assert_eq!(diameter(1.0, limits, range), 4.5);
    assert_eq!(
        diameter(4.0, limits, range),
        2.0 * diameter(1.0, limits, range)
    );
    // A value outside the limits takes the diameter of the nearer one.
    assert_eq!(diameter(-3.0, limits, range), 0.0);
    assert_eq!(diameter(40.0, limits, range), 18.0);
    // A range that does not begin at 0 is laid out in the same way.
    assert_eq!(diameter(4.0, limits, (2.0, 10.0)), 6.0);
    // Limits of one value, limits the wrong way round and a value that is not a number give
    // the lower end of the range.
    assert_eq!(diameter(5.0, (5.0, 5.0), (2.0, 10.0)), 2.0);
    assert_eq!(diameter(5.0, (9.0, 1.0), (2.0, 10.0)), 2.0);
    assert_eq!(diameter(f64::NAN, limits, (2.0, 10.0)), 2.0);
    // The largest and the smallest numbers have their place.
    assert_eq!(diameter(f64::MAX, (0.0, f64::MAX), range), 18.0);
    assert_eq!(diameter(5e-324, (0.0, 5e-324), range), 18.0);
    assert_eq!(diameter(-0.0, limits, range).to_bits(), 0.0_f64.to_bits());
    // The largest mark is four times the default across.
    assert_eq!(LARGEST_SIZE, 4.0 * POINT_SIZE);
}

#[test]
fn the_scene_of_points_sized_by_a_number() {
    let drawn = by_z(&[
        Some(4.0),
        Some(0.0),
        None,
        Some(16.0),
        Some(-1.0),
        Some(9.0),
    ]);
    insta::assert_snapshot!("size_number", drawn.to_json());
}

#[test]
fn the_scene_of_points_shaped_by_category() {
    let p = shaped_by_g(&[Some("b"), Some("a"), None, Some("b"), Some("a")]);
    insta::assert_snapshot!("shape_category", with_layer(&p, points()).1.to_json());
}

#[test]
fn the_scene_of_a_constant_size_and_shape_with_a_group_and_a_label() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None],
            &[Some(1.0), None, Some(3.0)],
        ),
        &[(Aesthetic::Group, "g"), (Aesthetic::Label, "z")],
        vec![all_sized(6.0), all_shaped("triangle_open")],
    );
    insta::assert_snapshot!("size_shape_constant", with_layer(&p, points()).1.to_json());
}

#[test]
fn the_scene_of_one_variable_as_colour_and_shape_beside_a_size() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None, Some("b")],
            &[Some(2.0), Some(8.0), Some(5.0), Some(0.5)],
        ),
        &[
            (Aesthetic::Color, "g"),
            (Aesthetic::Size, "z"),
            (Aesthetic::Shape, "g"),
        ],
        Vec::new(),
    );
    insta::assert_snapshot!("one_legend", with_layer(&p, points()).1.to_json());
}

#[test]
fn the_scene_of_a_sized_and_shaped_plot_before_any_layer() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None],
            &[Some(2.0), None, Some(40.0)],
        ),
        &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
        Vec::new(),
    );
    insta::assert_snapshot!("size_shape_no_layer", scene(&p).to_json());
}

#[test]
fn the_json_of_a_sized_and_shaped_scene_reads_back() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None],
            &[Some(2.0), Some(8.0), Some(5.0)],
        ),
        &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
        Vec::new(),
    );
    let text = with_layer(&p, points()).1.to_json();
    let json: Json = serde_json::from_str(&text).expect("the scene is JSON");
    assert_eq!(json["zeff_scene"].as_u64(), Some(u64::from(SCENE_VERSION)));
    let read = |value: &Json| -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    };
    // A scale of sizes: the square-root kind, and as its range the two diameters.
    let size = &json["scales"][2];
    assert_eq!(size["id"].as_str(), Some("size"));
    assert_eq!(size["kind"].as_str(), Some("sqrt"));
    assert_eq!(size["title"].as_str(), Some("z"));
    assert_eq!(bits(&list(&text, "limits", 2)), bits(&[0.0, 8.0]));
    assert_eq!(bits(&list(&text, "breaks", 2)), bits(&[2.0, 4.0, 6.0, 8.0]));
    assert_eq!(read(&size["labels"]), ["2", "4", "6", "8"]);
    assert_eq!(bits(&list(&text, "range", 0)), bits(&[0.0, 18.0]));
    // A scale of shapes: one name for each break.
    let shape = &json["scales"][3];
    assert_eq!(shape["id"].as_str(), Some("shape"));
    assert_eq!(shape["kind"].as_str(), Some("category"));
    assert_eq!(read(&shape["labels"]), ["a", "b", "NA"]);
    assert_eq!(read(&shape["range"]), ["circle", "triangle", "asterisk"]);
    // Two variables, two legends.
    let legends = json["guides"]["legends"].as_array().unwrap();
    assert_eq!(legends.len(), 2);
    assert_eq!(read(&legends[0]["scales"]), ["size"]);
    assert_eq!(read(&legends[1]["scales"]), ["shape"]);
    // Each mark has its value on each scale.
    assert_eq!(bits(&list(&text, "size", 0)), bits(&[2.0, 8.0, 5.0]));
    assert_eq!(bits(&list(&text, "shape", 0)), bits(&[2.0, 1.0, 3.0]));
    // A constant is the diameter and the name themselves.
    let p = plot_of(
        with_g_and_z("d", &[Some("a")], &[Some(1.0)]),
        &[],
        vec![all_sized(7.5), all_shaped("square")],
    );
    let constant: Json = serde_json::from_str(&with_layer(&p, points()).1.to_json()).unwrap();
    let layer = &constant["panels"][0]["layers"][0];
    assert_eq!(layer["size"].as_f64(), Some(7.5));
    assert_eq!(layer["shape"].as_str(), Some("square"));
    assert_eq!(constant["scales"].as_array().map(Vec::len), Some(2));
}

#[test]
fn the_scale_of_sizes_runs_from_0_to_the_largest_value_drawn() {
    let drawn = by_z(&[Some(2.0), Some(8.0), Some(5.0)]);
    let scale = drawn.scale("size").unwrap();
    assert_eq!(scale.kind, ScaleKind::Sqrt);
    assert_eq!(scale.kind.name(), "sqrt");
    assert_eq!(scale.title, "z");
    assert_eq!(scale.limits, (0.0, 8.0));
    // A key of no size would show nothing, so the breaks begin above 0.
    assert_eq!(scale.breaks, [2.0, 4.0, 6.0, 8.0]);
    assert_eq!(scale.labels, ["2", "4", "6", "8"]);
    assert_eq!(scale.range, Range::Sizes(0.0, LARGEST_SIZE));
    assert_eq!(sizes(&drawn, 0), [2.0, 8.0, 5.0]);
    assert!(drawn.notes.is_empty());
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].title, "z");
    assert_eq!(drawn.legends[0].scales, ["size"]);
    assert_eq!(drawn.legends[0].marks, [Mark::Points]);
    // The smallest value does not move the lower limit: a mark of 5 is drawn with five eighths
    // of the area of the largest, wherever the values begin.
    let drawn = by_z(&[Some(5.0), Some(8.0), Some(6.0)]);
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 8.0));
    // The largest value of a row that is not drawn is not on the scale. Row 2 has no `x`.
    let mut columns = vec![
        ("x", floats(&[Some(1.0), None, Some(3.0)])),
        ("y", numbers(&[1.0, 2.0, 3.0])),
    ];
    columns.push(("z", numbers(&[2.0, 800.0, 5.0])));
    let p = plot_of(frame("d", columns), &[(Aesthetic::Size, "z")], Vec::new());
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 5.0));
    assert_eq!(sizes(&drawn, 0), [2.0, 5.0]);
    assert_eq!(drawn.notes, ["1 row with NA in x, y or z dropped"]);
    // The label of the variable is the title of the scale and of its legend.
    let mut columns = positions(2);
    columns.push((
        "z",
        numbers(&[1.0, 2.0]).with_entry(MetadataKey::Label, "Hours a week"),
    ));
    let p = plot_of(frame("d", columns), &[(Aesthetic::Size, "z")], Vec::new());
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.scale("size").unwrap().title, "Hours a week");
    assert_eq!(drawn.legends[0].title, "Hours a week");
    // Whole numbers are sized as numbers.
    let mut columns = positions(3);
    columns.push(("z", Column::ints([Some(3), Some(0), Some(12)])));
    let p = plot_of(frame("d", columns), &[(Aesthetic::Size, "z")], Vec::new());
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(sizes(&drawn, 0), [3.0, 0.0, 12.0]);
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 12.0));
}

#[test]
fn a_size_of_zero_is_drawn_and_said() {
    let drawn = by_z(&[Some(0.0), Some(4.0), Some(0.0)]);
    assert_eq!(drawn.panels[0].layers[0].x.len(), 3);
    assert_eq!(sizes(&drawn, 0), [0.0, 4.0, 0.0]);
    assert_eq!(drawn.notes, ["2 rows with z of 0 are drawn with no size"]);
    assert_eq!(
        by_z(&[Some(0.0), Some(4.0)]).notes,
        ["1 row with z of 0 is drawn with no size"]
    );
    // A second layer over the same rows does not say it twice.
    let (p, _) = with_layer(&sized_by_z(&[Some(0.0), Some(4.0)]), points());
    let (_, twice) = with_layer(&p, points());
    assert_eq!(twice.notes, ["1 row with z of 0 is drawn with no size"]);
    // The zero written with a minus sign is the same zero.
    let drawn = by_z(&[Some(-0.0), Some(4.0)]);
    assert_eq!(drawn.panels[0].layers[0].x.len(), 2);
    assert_eq!(drawn.notes, ["1 row with z of 0 is drawn with no size"]);
}

#[test]
fn nothing_above_zero_to_size_gives_the_scale_from_0_to_1() {
    // Every size 0: the marks are there, and none is seen.
    let drawn = by_z(&[Some(0.0), Some(0.0)]);
    let scale = drawn.scale("size").unwrap();
    assert_eq!(scale.limits, (0.0, 1.0));
    assert_eq!(scale.breaks, [0.2, 0.4, 0.6, 0.8, 1.0]);
    assert_eq!(sizes(&drawn, 0), [0.0, 0.0]);
    assert_eq!(drawn.notes, ["2 rows with z of 0 are drawn with no size"]);
    // No size at all.
    let drawn = by_z(&[None, None]);
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 1.0));
    assert!(sizes(&drawn, 0).is_empty());
    assert_eq!(drawn.notes, ["2 rows with NA in x, y or z dropped"]);
    // No row.
    let drawn = by_z(&[]);
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 1.0));
    assert!(drawn.notes.is_empty());
    // Every size below 0.
    let drawn = by_z(&[Some(-2.0), Some(-1.0)]);
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 1.0));
    assert!(sizes(&drawn, 0).is_empty());
    // One value: the scale still begins at 0, so the one mark is the largest.
    let drawn = by_z(&[Some(3.0)]);
    let scale = drawn.scale("size").unwrap();
    assert_eq!(scale.limits, (0.0, 3.0));
    assert_eq!(
        diameter(3.0, scale.limits, (0.0, LARGEST_SIZE)),
        LARGEST_SIZE
    );
}

#[test]
fn a_row_without_a_size_or_with_one_below_0_is_not_drawn_and_is_counted() {
    let drawn = by_z(&[
        Some(4.0),
        None,
        Some(f64::INFINITY),
        Some(-1.0),
        Some(-0.5),
        Some(9.0),
        Some(f64::NEG_INFINITY),
    ]);
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 6.0]);
    assert_eq!(layer.y, [10.0, 15.0]);
    assert_eq!(sizes(&drawn, 0), [4.0, 9.0]);
    assert_eq!(
        drawn.notes,
        [
            "1 row with NA in x, y or z dropped",
            "2 rows with an infinite value in x, y or z dropped",
            "2 rows with z below 0 dropped: `size` shows amounts of 0 or more",
        ]
    );
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 9.0));
    // The axes show the marks that are drawn.
    assert_eq!(drawn.scale("x").unwrap().limits, (0.75, 6.25));
    // Before a layer reads the sizes no row is dropped, and the scale shows what the frame
    // holds.
    let before = scene(&sized_by_z(&[Some(4.0), None, Some(-1.0), Some(9.0)]));
    assert!(before.notes.is_empty());
    assert!(before.legends.is_empty());
    assert_eq!(before.scale("size").unwrap().limits, (0.0, 9.0));
}

#[test]
fn more_categories_than_shapes_repeat_them_and_say_so() {
    let names: Vec<String> = (0..10).map(|i| format!("k{i}")).collect();
    let cells: Vec<Option<&str>> = names.iter().map(|n| Some(n.as_str())).collect();
    let p = shaped_by_g(&cells);
    // Before a layer reads the shapes there is nothing to say of them.
    let before = scene(&p);
    assert_eq!(before.scale("shape").unwrap().labels.len(), 10);
    assert!(before.notes.is_empty());
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(
        shapes(&drawn),
        [
            "circle",
            "triangle",
            "square",
            "diamond",
            "plus",
            "cross",
            "triangle_down",
            "star",
            "circle",
            "triangle"
        ]
    );
    assert_eq!(
        places(&drawn, 0),
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    );
    assert_eq!(
        drawn.notes,
        [
            "`g` has 10 categories and there are 8 shapes, so the shapes repeat from the ninth category"
        ]
    );
    // A second layer does not say it twice.
    let (_, twice) = with_layer(&with_layer(&p, points()).0, points());
    assert_eq!(twice.notes, drawn.notes);
    // Eight categories and `NA` need no shape twice: `NA` has its own, which comes last.
    let mut eight: Vec<Option<&str>> = cells[..8].to_vec();
    eight.insert(2, None);
    let (_, drawn) = with_layer(&shaped_by_g(&eight), points());
    assert_eq!(
        shapes(&drawn),
        [
            "circle",
            "triangle",
            "square",
            "diamond",
            "plus",
            "cross",
            "triangle_down",
            "star",
            "asterisk"
        ]
    );
    assert_eq!(drawn.scale("shape").unwrap().labels[8], "NA");
    assert_eq!(
        places(&drawn, 0),
        [1.0, 2.0, 9.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
    );
    // No row is dropped for having no category.
    assert_eq!(drawn.panels[0].layers[0].x.len(), 9);
    assert!(drawn.notes.is_empty());
}

#[test]
fn a_shape_is_given_to_each_kind_of_category() {
    // Text in the order of its values, a Category in the order of its levels with the levels
    // that have no row left out, truth values false first, and labelled numbers by their labels.
    let shaped = |g: Column| {
        let p = plot_of(with_g(g), &[(Aesthetic::Shape, "g")], Vec::new());
        with_layer(&p, points()).1
    };
    let drawn = shaped(texts(&[Some("pear"), Some("apple"), Some("pear")]));
    let scale = drawn.scale("shape").unwrap();
    assert_eq!(scale.kind, ScaleKind::Category);
    assert_eq!(scale.labels, ["apple", "pear"]);
    assert_eq!(scale.limits, (1.0, 2.0));
    assert_eq!(scale.breaks, [1.0, 2.0]);
    assert_eq!(shapes(&drawn), ["circle", "triangle"]);
    assert_eq!(places(&drawn, 0), [2.0, 1.0, 2.0]);
    let drawn = shaped(levels(
        &["low", "middle", "high"],
        &[Some(2), Some(0), Some(2)],
    ));
    assert_eq!(drawn.scale("shape").unwrap().labels, ["low", "high"]);
    assert_eq!(shapes(&drawn), ["circle", "triangle"]);
    let drawn = shaped(Column::bools([Some(true), Some(false), None]));
    assert_eq!(
        drawn.scale("shape").unwrap().labels,
        ["false", "true", "NA"]
    );
    assert_eq!(shapes(&drawn), ["circle", "triangle", "asterisk"]);
    assert_eq!(places(&drawn, 0), [2.0, 1.0, 3.0]);
    let drawn = shaped(labelled(
        &[Some(1), Some(0), Some(1)],
        &[("0", "no"), ("1", "yes")],
    ));
    assert_eq!(drawn.scale("shape").unwrap().labels, ["no", "yes"]);
    assert_eq!(places(&drawn, 0), [2.0, 1.0, 2.0]);
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].scales, ["shape"]);
}

#[test]
fn a_constant_stands_beside_a_mapping() {
    let g = [Some("b"), Some("a"), Some("b")];
    let z = [Some(2.0), Some(8.0), Some(5.0)];
    // One size for every mark, and a shape for each category.
    let p = plot_of(
        with_g_and_z("d", &g, &z),
        &[(Aesthetic::Shape, "g")],
        vec![all_sized(6.0)],
    );
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.panels[0].layers[0].size, Sizing::Constant(6.0));
    assert_eq!(places(&drawn, 0), [2.0, 1.0, 2.0]);
    assert!(drawn.scale("size").is_none());
    assert_eq!(drawn.legends.len(), 1);
    // One shape for every mark, a size for each value, and one colour.
    let p = plot_of(
        with_g_and_z("d", &g, &z),
        &[(Aesthetic::Size, "z")],
        vec![all_shaped("diamond_open"), set(Aesthetic::Color, "tomato")],
    );
    let (_, drawn) = with_layer(&p, points());
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.shape, Shaping::Constant("diamond_open"));
    assert_eq!(layer.color, Paint::Constant("#FF6347".to_string()));
    assert_eq!(sizes(&drawn, 0), [2.0, 8.0, 5.0]);
    assert!(drawn.scale("shape").is_none());
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].scales, ["size"]);
    // Both constants: no scale but the positions, and no legend.
    let p = plot_of(
        with_g_and_z("d", &g, &z),
        &[],
        vec![all_sized(2.0), all_shaped("plus")],
    );
    let (_, drawn) = with_layer(&p, points());
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.size, Sizing::Constant(2.0));
    assert_eq!(layer.shape, Shaping::Constant("plus"));
    assert_eq!(drawn.scales.len(), 2);
    assert!(drawn.legends.is_empty());
    // Without either, a point is the default across and a circle.
    let (_, drawn) = with_layer(
        &plot_of(with_g_and_z("d", &g, &z), &[], Vec::new()),
        points(),
    );
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.size, Sizing::Constant(POINT_SIZE));
    assert_eq!(layer.shape, Shaping::Constant("circle"));
}

#[test]
fn the_size_of_a_layer_takes_the_place_of_the_size_of_the_plot() {
    // Over a constant.
    let g = [Some("a"), Some("a"), Some("a"), Some("a")];
    let z = [Some(4.0), None, Some(-1.0), Some(0.0)];
    let p = plot_of(with_g_and_z("d", &g, &z), &[], vec![all_sized(6.0)]);
    let (_, drawn) = with_layer(&p, points_of_size(3.0));
    assert_eq!(drawn.panels[0].layers[0].size, Sizing::Constant(3.0));
    // Over a mapping: the layer does not read the sizes, so no row is dropped for its size,
    // nothing is said of them and there is no legend; the scale shows what the frame holds.
    let p = sized_by_z(&z);
    let (with_own, drawn) = with_layer(&p, points_of_size(3.0));
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.size, Sizing::Constant(3.0));
    assert_eq!(layer.x.len(), 4);
    assert!(drawn.notes.is_empty());
    assert!(drawn.legends.is_empty());
    assert_eq!(drawn.scale("size").unwrap().limits, (0.0, 4.0));
    // A second layer without a size of its own reads them, and has the legend to itself.
    let (_, both) = with_layer(&with_own, points());
    assert_eq!(both.panels[0].layers[0].size, Sizing::Constant(3.0));
    assert_eq!(both.panels[0].layers[0].x.len(), 4);
    assert_eq!(sizes(&both, 1), [4.0, 0.0]);
    assert_eq!(both.legends.len(), 1);
    assert_eq!(both.legends[0].marks, [Mark::Points]);
    assert_eq!(
        both.notes,
        [
            "1 row with NA in x, y or z dropped",
            "1 row with z below 0 dropped: `size` shows amounts of 0 or more",
            "1 row with z of 0 is drawn with no size",
        ]
    );
    // What a layer reads of the plot.
    assert!(reads(Mark::Points, Aesthetic::Size));
    assert!(reads(Mark::Points, Aesthetic::Shape));
    assert!(!reads(Mark::Points, Aesthetic::Group));
    assert!(!reads(Mark::Points, Aesthetic::Label));
    assert!(layer_reads(&points(), Aesthetic::Size));
    assert!(!layer_reads(&points_of_size(3.0), Aesthetic::Size));
    assert!(layer_reads(&points_of_size(3.0), Aesthetic::Shape));
    assert!(layer_reads(&points_of_size(3.0), Aesthetic::Color));
    assert!(!layer_reads(&points(), Aesthetic::Fill));
    assert!(!layer_reads(&points(), Aesthetic::Group));
    assert!(!layer_reads(&points(), Aesthetic::Label));
}

#[test]
fn a_layer_with_its_own_frame_is_sized_and_shaped_on_the_scales_of_the_plot() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), Some("b")],
            &[Some(2.0), Some(8.0), Some(5.0)],
        ),
        &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
        Vec::new(),
    );
    let (p, _) = with_layer(&p, points());
    let other = with_g_and_z(
        "e",
        &[Some("c"), None, Some("a")],
        &[Some(100.0), Some(1.0), Some(0.0)],
    );
    let (_, drawn) = with_layer(&p, own(other));
    // One scale of sizes for both layers, to the largest value either draws.
    let size = drawn.scale("size").unwrap();
    assert_eq!(size.limits, (0.0, 100.0));
    assert_eq!(size.breaks, [20.0, 40.0, 60.0, 80.0, 100.0]);
    assert_eq!(sizes(&drawn, 0), [2.0, 8.0, 5.0]);
    assert_eq!(sizes(&drawn, 1), [100.0, 1.0, 0.0]);
    // The categories of the plot's frame first, then the one only the layer's frame has, and
    // `NA` last.
    assert_eq!(drawn.scale("shape").unwrap().labels, ["a", "b", "c", "NA"]);
    assert_eq!(shapes(&drawn), ["circle", "triangle", "square", "asterisk"]);
    assert_eq!(places(&drawn, 0), [2.0, 1.0, 2.0]);
    assert_eq!(places(&drawn, 1), [3.0, 4.0, 1.0]);
    assert_eq!(drawn.notes, ["1 row with z of 0 is drawn with no size"]);
    assert_eq!(drawn.legends.len(), 2);
}

#[test]
fn a_size_by_categories_and_a_shape_by_numbers_give_no_scale() {
    // `plot` refuses both; a plot built without it draws the marks as if nothing were said.
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), None, Some("a")],
            &[Some(2.0), Some(-8.0), None],
        ),
        &[(Aesthetic::Size, "g"), (Aesthetic::Shape, "z")],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&p, points());
    assert_eq!(drawn.scales.len(), 2);
    assert!(drawn.legends.is_empty());
    assert!(drawn.notes.is_empty());
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.x.len(), 3);
    assert_eq!(layer.size, Sizing::Constant(POINT_SIZE));
    assert_eq!(layer.shape, Shaping::Constant("circle"));
    // A date is neither an amount nor a category.
    let mut columns = positions(2);
    columns.push(("g", Column::dates([Some(0), Some(1)])));
    let p = plot_of(
        frame("d", columns),
        &[(Aesthetic::Size, "g"), (Aesthetic::Shape, "g")],
        Vec::new(),
    );
    assert_eq!(with_layer(&p, points()).1.scales.len(), 2);
}

#[test]
fn one_variable_has_one_legend() {
    let frame = || {
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None, Some("b")],
            &[Some(2.0), Some(8.0), Some(5.0), Some(0.5)],
        )
    };
    let legends = |more: &[(Aesthetic, &str)]| {
        let (_, drawn) = with_layer(&plot_of(frame(), more, Vec::new()), points());
        drawn
            .legends
            .iter()
            .map(|legend| (legend.title.clone(), legend.scales.clone()))
            .collect::<Vec<_>>()
    };
    // The colour and the shape of one variable are read from one legend, whatever the order
    // they are written in: the scales keep the order color, fill, size, shape.
    assert_eq!(
        legends(&[(Aesthetic::Shape, "g"), (Aesthetic::Color, "g")]),
        [("g".to_string(), vec!["color", "shape"])]
    );
    // And so are the colour and the size of one number.
    assert_eq!(
        legends(&[(Aesthetic::Color, "z"), (Aesthetic::Size, "z")]),
        [("z".to_string(), vec!["color", "size"])]
    );
    // Two variables, two legends, in the order of their first scale.
    assert_eq!(
        legends(&[
            (Aesthetic::Shape, "g"),
            (Aesthetic::Size, "z"),
            (Aesthetic::Color, "g"),
        ]),
        [
            ("g".to_string(), vec!["color", "shape"]),
            ("z".to_string(), vec!["size"]),
        ]
    );
    assert_eq!(
        legends(&[(Aesthetic::Color, "z"), (Aesthetic::Shape, "g")]),
        [
            ("z".to_string(), vec!["color"]),
            ("g".to_string(), vec!["shape"]),
        ]
    );
    // A scale no layer reads is in no legend: the fill of `g` is not shown beside its shape.
    assert_eq!(
        legends(&[(Aesthetic::Fill, "g"), (Aesthetic::Shape, "g")]),
        [("g".to_string(), vec!["shape"])]
    );
    // The scales are in one order whatever the order they are written in.
    let (_, drawn) = with_layer(
        &plot_of(
            frame(),
            &[
                (Aesthetic::Shape, "g"),
                (Aesthetic::Size, "z"),
                (Aesthetic::Fill, "g"),
                (Aesthetic::Color, "z"),
            ],
            Vec::new(),
        ),
        points(),
    );
    let ids: Vec<&str> = drawn.scales.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["x", "y", "color", "fill", "size", "shape"]);
    // A layer with a size of its own leaves the legend of `z` to its colour.
    let p = plot_of(
        frame(),
        &[(Aesthetic::Color, "z"), (Aesthetic::Size, "z")],
        Vec::new(),
    );
    let (_, drawn) = with_layer(&p, points_of_size(3.0));
    assert_eq!(drawn.legends.len(), 1);
    assert_eq!(drawn.legends[0].scales, ["color"]);
}

#[test]
fn group_and_label_are_kept_with_the_plot_and_no_point_reads_them() {
    let g = [Some("b"), None, Some("a")];
    let z = [Some(2.0), None, Some(f64::INFINITY)];
    let p = plot_of(
        with_g_and_z("d", &g, &z),
        &[(Aesthetic::Group, "z"), (Aesthetic::Label, "g")],
        Vec::new(),
    );
    assert_eq!(p.mapped(Aesthetic::Group), Some("z"));
    assert_eq!(p.mapped(Aesthetic::Label), Some("g"));
    let (_, drawn) = with_layer(&p, points());
    let (_, plain) = with_layer(
        &plot_of(with_g_and_z("d", &g, &z), &[], Vec::new()),
        points(),
    );
    // No scale, no legend, and no row dropped for a group or a label that is not there: the
    // scene is the one of the plot without them.
    assert_eq!(drawn, plain);
    assert_eq!(drawn.panels[0].layers[0].x.len(), 3);
    assert!(drawn.notes.is_empty());
}

#[test]
fn jitter_moves_the_points_and_leaves_their_sizes_and_shapes() {
    let p = plot_of(
        with_g_and_z(
            "d",
            &[Some("b"), Some("a"), None, Some("b")],
            &[Some(2.0), Some(8.0), Some(5.0), Some(0.5)],
        ),
        &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
        Vec::new(),
    );
    let (_, still) = with_layer(&p, points());
    let (_, moved) = with_layer(&p, jittered(0.4, 11));
    assert_ne!(moved.panels[0].layers[0].x, still.panels[0].layers[0].x);
    assert_eq!(sizes(&moved, 0), sizes(&still, 0));
    assert_eq!(places(&moved, 0), places(&still, 0));
    assert_eq!(moved.scale("size"), still.scale("size"));
    assert_eq!(moved.scale("shape"), still.scale("shape"));
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
                    !(l.starts_with("\"x\": [")
                        || l.starts_with("\"y\": [")
                        || l.starts_with("\"color\": [")
                        || l.starts_with("\"rows\": ["))
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

    /// Step 20d: a sized and shaped plot gives the same scene however often it is derived, the
    /// scene is JSON, and jitter leaves the size and the shape of a point.
    #[test]
    fn the_same_data_give_the_same_sized_and_shaped_scene(
        rows in painted_rows(), share in 0.0f64..1.0, seed in 0u64..1000
    ) {
        let build = || {
            let p = plot_of(
                painted_frame(&rows),
                &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
                Vec::new(),
            );
            let (p, _) = with_layer(&p, points());
            with_layer(&p, jittered(share, seed))
        };
        let (first_plot, first) = build();
        let (_, second) = build();
        prop_assert_eq!(first.to_json(), second.to_json());
        prop_assert_eq!(scene(&first_plot).to_json(), first.to_json());
        let read: Result<Json, _> = serde_json::from_str(&first.to_json());
        prop_assert!(read.is_ok(), "{}", first.to_json());
        prop_assert_eq!(bits(sizes(&first, 0)), bits(sizes(&first, 1)));
        prop_assert_eq!(places(&first, 0), places(&first, 1));
    }

    /// Step 20d: reordering the rows reorders the marks, each with its size and its shape, and
    /// changes nothing else: neither scale depends on which row comes first.
    #[test]
    fn reordering_the_rows_carries_each_size_and_shape_with_its_mark(
        (rows, shuffled) in painted_rows()
            .prop_flat_map(|rows| (Just(rows.clone()), Just(rows).prop_shuffle())),
    ) {
        let build = |rows: &[Painted]| {
            plot_of(
                painted_frame(rows),
                &[(Aesthetic::Size, "z"), (Aesthetic::Shape, "g")],
                Vec::new(),
            )
        };
        let a = with_layer(&build(&rows), points()).1;
        let b = with_layer(&build(&shuffled), points()).1;
        prop_assert_eq!(&a.scales, &b.scales);
        prop_assert_eq!(&a.legends, &b.legends);
        prop_assert_eq!(&a.notes, &b.notes);
        let marks = |s: &Scene| {
            let l = &s.panels[0].layers[0];
            let (size, place) = (sizes(s, 0), places(s, 0));
            assert_eq!((size.len(), place.len()), (l.x.len(), l.x.len()));
            let mut all: Vec<(u64, u64, u64, u64)> = (0..l.x.len())
                .map(|i| (l.x[i].to_bits(), l.y[i].to_bits(), size[i].to_bits(), place[i].to_bits()))
                .collect();
            all.sort_unstable();
            all
        };
        prop_assert_eq!(marks(&a), marks(&b));
        // Everything of the scene but the lists of marks is the same text.
        let without_marks = |s: &Scene| -> String {
            s.to_json()
                .lines()
                .filter(|l| {
                    let l = l.trim_start();
                    !["\"x\": [", "\"y\": [", "\"size\": [", "\"shape\": [", "\"rows\": ["]
                        .iter()
                        .any(|opening| l.starts_with(opening))
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        prop_assert_eq!(without_marks(&a), without_marks(&b));
        // And before any layer the scene is the same text.
        prop_assert_eq!(scene(&build(&rows)).to_json(), scene(&build(&shuffled)).to_json());
    }

    /// Step 20d: every mark has a size on the scale of sizes, which runs from 0 to the largest
    /// value drawn, and is given a diameter from 0 to the largest; a row is dropped for its size
    /// when that is `NA`, infinite or below 0. And every mark has a place on the scale of
    /// shapes, with the shape of its category; no row is dropped for its category.
    #[test]
    fn every_size_and_every_shape_lies_on_its_scale(rows in painted_rows()) {
        let complete = |r: &Painted| matches!((r.0, r.1), (Some(a), Some(b)) if a.is_finite() && b.is_finite());
        // Sizes.
        let p = plot_of(painted_frame(&rows), &[(Aesthetic::Size, "z")], Vec::new());
        let drawn = with_layer(&p, points()).1;
        let s = drawn.scale("size").unwrap();
        let (lo, hi) = s.limits;
        prop_assert_eq!(s.kind, ScaleKind::Sqrt);
        prop_assert_eq!(&s.range, &Range::Sizes(0.0, LARGEST_SIZE));
        prop_assert!(lo.to_bits() == 0.0_f64.to_bits() && hi.is_finite() && hi > 0.0, "{:?}", s);
        prop_assert!(!s.breaks.is_empty() && s.breaks.len() <= 50, "{:?}", s);
        prop_assert!(s.breaks.iter().all(|b| 0.0 < *b && *b <= hi), "{:?}", s);
        prop_assert!(s.breaks.windows(2).all(|w| w[0] < w[1]), "{:?}", s);
        prop_assert_eq!(s.labels.len(), s.breaks.len());
        let kept: Vec<f64> = rows
            .iter()
            .filter(|r| complete(r))
            .filter_map(|r| r.3.filter(|z| z.is_finite() && *z >= 0.0))
            .collect();
        prop_assert_eq!(bits(sizes(&drawn, 0)), bits(&kept));
        prop_assert_eq!(drawn.panels[0].layers[0].x.len(), kept.len());
        let most = kept.iter().copied().fold(0.0, f64::max);
        if most > 0.0 {
            prop_assert_eq!(hi, most);
        } else if !kept.is_empty() {
            prop_assert_eq!(hi, 1.0);
        }
        for z in &kept {
            let across = diameter(*z, s.limits, (0.0, LARGEST_SIZE));
            prop_assert!((0.0..=LARGEST_SIZE).contains(&across), "{z} is {across} across: {:?}", s);
            // A value of 0 has no size; a value so far below the largest that its share cannot
            // be told from 0 has none either.
            prop_assert!(*z != 0.0 || across == 0.0, "{z} is {across} across: {:?}", s);
            prop_assert!(across > 0.0 || z / hi < 1e-300, "{z} is {across} across: {:?}", s);
            // The area is in proportion to the value, to within the rounding of the root.
            let share = (across / LARGEST_SIZE) * (across / LARGEST_SIZE);
            prop_assert!((share - z / hi).abs() <= 1e-12, "{z} has the share {share}: {:?}", s);
        }
        if most > 0.0 {
            prop_assert_eq!(diameter(most, s.limits, (0.0, LARGEST_SIZE)), LARGEST_SIZE);
        }
        // A larger value is never drawn smaller.
        let mut sorted = kept.clone();
        sorted.sort_by(f64::total_cmp);
        let across = |z: f64| diameter(z, s.limits, (0.0, LARGEST_SIZE));
        let in_order = sorted.windows(2).all(|w| across(w[0]) <= across(w[1]));
        prop_assert!(in_order, "a larger value is drawn smaller: {:?}", s);
        // The rows counted in the notes and the marks drawn are all the rows.
        let counted = |text: &str| -> usize {
            drawn
                .notes
                .iter()
                .filter(|note| note.contains(text))
                .map(|note| note.split(' ').next().unwrap().replace(',', "").parse::<usize>().unwrap())
                .sum()
        };
        prop_assert_eq!(
            kept.len() + counted("with NA in") + counted("with an infinite value in") + counted("below 0 dropped"),
            rows.len()
        );
        prop_assert_eq!(counted("drawn with no size"), kept.iter().filter(|z| **z == 0.0).count());
        // Shapes.
        let p = plot_of(painted_frame(&rows), &[(Aesthetic::Shape, "g")], Vec::new());
        let drawn = with_layer(&p, points()).1;
        let s = drawn.scale("shape").unwrap();
        let total = s.breaks.len();
        prop_assert_eq!(s.kind, ScaleKind::Category);
        prop_assert_eq!(s.labels.len(), total);
        prop_assert_eq!(shapes(&drawn).len(), total);
        prop_assert_eq!(s.limits, (1.0, total.max(1) as f64));
        prop_assert!(s.breaks.iter().enumerate().all(|(i, b)| *b == (i + 1) as f64), "{:?}", s);
        let kept: Vec<&Painted> = rows.iter().filter(|r| complete(r)).collect();
        let place = places(&drawn, 0);
        prop_assert_eq!(place.len(), kept.len());
        prop_assert_eq!(drawn.panels[0].layers[0].x.len(), kept.len());
        for (value, row) in place.iter().zip(&kept) {
            let at = *value as usize;
            prop_assert!(at >= 1 && at <= total && at as f64 == *value, "{value} {:?}", s);
            let label = row.2.map_or("NA".to_string(), |k| format!("k{k:02}"));
            prop_assert_eq!(&s.labels[at - 1], &label);
            let expected = if row.2.is_some() { shape::PALETTE[(at - 1) % shape::PALETTE.len()] } else { shape::NA_SHAPE };
            prop_assert_eq!(shapes(&drawn)[at - 1], expected);
        }
        if !kept.is_empty() {
            for at in 1..=total {
                prop_assert!(place.contains(&(at as f64)), "no mark at {at}: {:?}", s);
            }
        }
        prop_assert!(s.labels.iter().enumerate().all(|(i, l)| !s.labels[..i].contains(l)), "{:?}", s);
        // The shapes repeat, and the scene says so, when there are more than eight categories.
        let named = total - usize::from(s.labels.last().is_some_and(|l| l == "NA"));
        prop_assert_eq!(
            drawn.notes.iter().any(|note| note.contains("shapes repeat")),
            named > shape::PALETTE.len()
        );
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
                .filter(|l| {
                    let l = l.trim_start();
                    !(l.starts_with("\"x\": [") || l.starts_with("\"y\": [") || l.starts_with("\"rows\": ["))
                })
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

// ----- where a mark came from, and what version 1 reserves (step 21a2; #118, F2 to F4) -----

#[test]
fn each_mark_knows_the_frame_and_the_row_it_came_from() {
    // Rows 2 and 5 cannot be drawn: the marks are rows 1, 3, 4 and 6 of the frame `d`.
    let x = [Some(1.0), None, Some(3.0), Some(4.0), Some(5.0), Some(6.0)];
    let y = [
        Some(2.0),
        Some(1.0),
        Some(0.5),
        Some(7.0),
        Some(f64::INFINITY),
        Some(9.0),
    ];
    let drawn = scatter(&x, &y);
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.data.as_deref(), Some("d"));
    assert_eq!(layer.rows, [1, 3, 4, 6]);
    assert_eq!(layer.x, [1.0, 3.0, 4.0, 6.0]);
    // Jitter moves a mark and not the row behind it; a layer with a frame of its own names
    // that frame and counts its rows; a frame written in place has no name to give.
    let p = plot(
        frame("d", vec![("x", floats(&x)), ("y", floats(&y))]),
        Some("x"),
        Some("y"),
    );
    let (p, _) = with_layer(&p, jittered(0.4, 7));
    let other = frame(
        "means",
        vec![
            ("x", numbers(&[9.0, f64::NAN, 2.0])),
            ("y", numbers(&[1.0, 2.0, 3.0])),
        ],
    );
    let (p, _) = with_layer(&p, own(other));
    let mut unnamed = frame("gone", vec![("x", numbers(&[5.0])), ("y", numbers(&[5.0]))]);
    unnamed.name = None;
    let (_, drawn) = with_layer(&p, own(unnamed));
    let layers = &drawn.panels[0].layers;
    assert_eq!(layers[0].rows, [1, 3, 4, 6]);
    assert_eq!(layers[0].data.as_deref(), Some("d"));
    assert_eq!(layers[1].rows, [1, 3]);
    assert_eq!(layers[1].data.as_deref(), Some("means"));
    assert_eq!(layers[2].rows, [1]);
    assert_eq!(layers[2].data, None);
    // In the JSON of the scene, which the viewer reads.
    let json = drawn.to_json();
    assert!(
        json.contains("\"data\": \"d\",\n          \"rows\": [1, 3, 4, 6]"),
        "{json}"
    );
    assert!(
        json.contains("\"data\": \"means\",\n          \"rows\": [1, 3]"),
        "{json}"
    );
    assert!(
        json.contains("\"data\": null,\n          \"rows\": [1]"),
        "{json}"
    );
    // No layer: nothing to say of rows.
    assert!(
        scene(&plot(doses(), Some("dose"), Some("response"))).panels[0]
            .layers
            .is_empty()
    );
}

#[test]
fn version_1_reserves_four_names_and_a_scene_that_holds_one_is_not_drawn() {
    let drawn = scatter(&some(&[1.0, 2.0, 3.0]), &some(&[2.0, 1.0, 4.0]));
    // A scene of this release holds none of them, in the value or in its JSON.
    assert_eq!(drawn.undrawn(), None);
    assert_eq!(drawn.frame, None);
    assert!(
        drawn
            .panels
            .iter()
            .all(|p| p.crs.is_none() && p.projection.is_none())
    );
    assert!(
        drawn.panels[0]
            .layers
            .iter()
            .all(|l| l.z.is_none() && !l.raster)
    );
    let json = drawn.to_json();
    for name in ["frame", "\"z\"", "projection", "crs", "raster"] {
        assert!(!json.contains(name), "{name} in {json}");
    }
    // Each, given a value, is written, and the scene says it cannot be drawn here.
    let mut animated = drawn.clone();
    animated.frame = Some(3);
    assert!(
        animated
            .to_json()
            .contains("\"notes\": [],\n  \"frame\": 3\n}"),
        "{}",
        animated.to_json()
    );
    assert_eq!(
        animated.undrawn().as_deref(),
        Some(
            "this scene holds `frame`, the place of a scene in an animation, which this version of Zeff does not draw; it was made by a later version, which can draw it"
        )
    );
    let mut mapped = drawn.clone();
    mapped.panels[0].crs = Some("EPSG:4326".to_string());
    assert!(mapped.to_json().contains("\"crs\": \"EPSG:4326\""));
    assert!(
        mapped
            .undrawn()
            .unwrap()
            .starts_with("this scene holds `crs`, the coordinate system of a map,")
    );
    let mut solid = drawn.clone();
    solid.panels[0].projection = Some("perspective".to_string());
    assert!(solid.to_json().contains("\"projection\": \"perspective\""));
    assert!(
        solid
            .undrawn()
            .unwrap()
            .starts_with("this scene holds `projection`,")
    );
    let mut deep = drawn.clone();
    deep.panels[0].layers[0].z = Some(vec![0.5, 1.5, 2.5]);
    assert!(
        deep.to_json()
            .contains("\"rows\": [1, 2, 3],\n          \"z\": [0.5, 1.5, 2.5]"),
        "{}",
        deep.to_json()
    );
    assert!(
        deep.undrawn()
            .unwrap()
            .starts_with("this scene holds `z`, a third coordinate,")
    );
    // The flag of a layer drawn as pixels is written when it is set, and is no refusal: a
    // writer that cannot rasterise draws the layer as it draws the others.
    let mut pixels = drawn.clone();
    pixels.panels[0].layers[0].raster = true;
    assert!(
        pixels
            .to_json()
            .contains("\"rows\": [1, 2, 3],\n          \"raster\": true"),
        "{}",
        pixels.to_json()
    );
    assert_eq!(pixels.undrawn(), None);
    // Every changed scene still reads as JSON.
    for changed in [&animated, &mapped, &solid, &deep, &pixels] {
        assert!(serde_json::from_str::<serde_json::Value>(&changed.to_json()).is_ok());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    #[test]
    fn the_rows_of_a_layer_are_those_of_its_marks_in_order(rows in rows(), share in 0.0f64..1.0, seed in 0u64..1000) {
        let (x, y) = columns(&rows);
        let p = plot(frame("d", vec![("x", floats(&x)), ("y", floats(&y))]), Some("x"), Some("y"));
        let (p, plain) = with_layer(&p, points());
        let (_, moved) = with_layer(&p, jittered(share, seed));
        let layer = &plain.panels[0].layers[0];
        // One row for each mark, rising, each a row of the frame that can be drawn; and the
        // mark is that row's values.
        prop_assert_eq!(layer.rows.len(), layer.x.len());
        prop_assert!(layer.rows.windows(2).all(|w| w[0] < w[1]));
        for (i, &row) in layer.rows.iter().enumerate() {
            prop_assert!(row >= 1 && row <= x.len());
            prop_assert_eq!(Some(layer.x[i].to_bits()), x[row - 1].map(f64::to_bits));
            prop_assert_eq!(Some(layer.y[i].to_bits()), y[row - 1].map(f64::to_bits));
        }
        // Every row that can be drawn is there.
        let drawable = (0..x.len())
            .filter(|&i| matches!((x[i], y[i]), (Some(a), Some(b)) if a.is_finite() && b.is_finite()))
            .count();
        prop_assert_eq!(layer.rows.len(), drawable);
        // A jittered layer has the same rows.
        prop_assert_eq!(&moved.panels[0].layers[1].rows, &layer.rows);
    }
}
