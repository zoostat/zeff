//! The scene of a plot (Language Reference, section 7.6, "Rendering and export"; Engine and
//! Project Specification, section 8.3, point 5; release 0.7.0, step 20): snapshots of the scene
//! of small plots as JSON, the JSON read back value for value, the edge cases, and the
//! properties: the same data give the same scene, reordering the rows changes nothing but the
//! order of the marks, every break and every mark lies within the limits, and jitter moves a
//! point by no more than its share of the gap.

use proptest::prelude::*;
use serde_json::Value as Json;
use std::rc::Rc;
use zeff_columns::{Column, MetadataKey, Table};
use zeff_core::value::{Aesthetic, Frame, Mark, Plot, PlotLayer, PlotMapping};
use zeff_numeric::random::Stream;
use zeff_plot::scene::{POINT_SIZE, SCENE_VERSION, Scene, scene, start, with_layer};

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
        });
    }
    if let Some(y) = y {
        mappings.push(PlotMapping {
            aesthetic: Aesthetic::Y,
            variable: y.to_string(),
        });
    }
    start(frame, mappings)
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

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
}
