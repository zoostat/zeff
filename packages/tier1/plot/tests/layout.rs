//! The layout of a figure (Language Reference, section 7.6; release 0.7.0, step 21b): a scene
//! and a size become a drawing. The three named sizes; a small figure read whole as text; six
//! sample figures of the shipped wages laid out at the three sizes; the properties (no text
//! below 8 points, every mark inside the panel, every text on the page, the same scene and
//! size give the same drawing, and a drawing holds nothing of the rows of the data); and the
//! edge cases (no layer, one point, a long title, a legend of twelve categories, a page too
//! small, a size that is none, a scene of a later version, a character no font has).

use proptest::prelude::*;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::value::{Aesthetic, Frame, Mark, PlotConstant, PlotLayer, PlotMapping, PlotSetting};
use zeff_plot::drawing::{Anchor, Drawing, Item, Primitive};
use zeff_plot::font::{FaceName, face};
use zeff_plot::layout::{
    JOURNAL, LayoutError, MM, SMALLEST_PANEL, SMALLEST_TEXT, Size, layout, layout_with_notes,
};
use zeff_plot::scene::{Scene, scene, start, with_layer};

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
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

fn floats(values: &[Option<f64>]) -> Column {
    Column::floats(values.iter().copied()).0
}

fn numbers(values: &[f64]) -> Column {
    Column::floats(values.iter().map(|v| Some(*v))).0
}

fn mapped(aesthetic: Aesthetic, name: &str) -> PlotMapping {
    PlotMapping {
        aesthetic,
        variable: name.to_string(),
        computed: None,
    }
}

fn points(alpha: f64) -> PlotLayer {
    PlotLayer {
        mark: Mark::Points,
        data: None,
        alpha,
        size: None,
        jitter: 0.0,
        stream: None,
        computed: Vec::new(),
    }
}

/// A plot of `frame` with these mappings and one layer of points.
fn drawn(frame: Frame, mappings: &[(Aesthetic, &str)], constants: Vec<PlotConstant>) -> Scene {
    let plot = start(
        frame,
        mappings.iter().map(|(a, n)| mapped(*a, n)).collect(),
        constants,
    );
    with_layer(&plot, points(1.0)).1
}

fn wages() -> Frame {
    Frame {
        table: Rc::new(zeff_datasets::table("wages").expect("the shipped wages")),
        name: Some("wages".to_string()),
    }
}

/// A small frame: two positions, a category, a number.
fn small() -> Frame {
    frame(
        "d",
        vec![
            ("dose", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
            ("response", numbers(&[2.1, 3.9, 6.2, 7.8, 9.0, 11.5])),
            (
                "arm",
                Column::strs(["placebo", "drug", "placebo", "drug", "placebo", "drug"].map(Some)),
            ),
            ("weight", numbers(&[60.0, 72.5, 81.0, 55.0, 90.0, 68.0])),
        ],
    )
}

/// The width of a text as the layout measures it, and the lowest size of type in a drawing.
fn width_of(words: &str, name: FaceName, size: f64) -> f64 {
    f64::from(face(name).shape(words).width) * size / 1000.0
}

/// The box a text takes on the page: left, top, right, bottom.
fn extent(item: &Item) -> Option<(f64, f64, f64, f64)> {
    let Primitive::Text {
        at,
        anchor,
        turn,
        spans,
        ..
    } = &item.primitive
    else {
        return None;
    };
    let along: f64 = spans
        .iter()
        .map(|s| width_of(&s.text, s.face, s.size))
        .sum();
    let size = spans.iter().map(|s| s.size).fold(0.0, f64::max);
    let (before, after) = match anchor {
        Anchor::Start => (0.0, along),
        Anchor::Middle => (along / 2.0, along / 2.0),
        Anchor::End => (along, 0.0),
    };
    // Above the baseline the height of a capital, below it nothing that matters here.
    let rise = f64::from(face(FaceName::Sans).cap_height()) * size / 1000.0;
    Some(if *turn == 0.0 {
        (at.0 - before, at.1 - rise, at.0 + after, at.1)
    } else {
        // Read upwards: the text runs up the page from its start and rises to the left.
        (at.0 - rise, at.1 - after, at.0, at.1 + before)
    })
}

/// A drawing read as text with the marks of each layer counted and hashed, not listed: what
/// a snapshot of a figure of eighteen thousand points keeps.
fn outline(drawing: &Drawing) -> String {
    let whole = drawing.describe();
    let mut out = String::new();
    let mut marks = 0;
    for line in whole.lines() {
        if line.starts_with("  [layer:") {
            marks += 1;
            if marks <= 3 {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }
        if marks > 3 {
            out.push_str(&format!("  … {} more marks\n", marks - 3));
        }
        marks = 0;
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!(
        "all of it: {}\n",
        zeff_hash::hash_hex(whole.as_bytes())
    ));
    out
}

// ----- the sizes -----

#[test]
fn the_three_named_sizes_and_a_size_in_millimetres() {
    assert_eq!(Size::OneColumn.millimetres(), (89.0, 66.0));
    assert_eq!(Size::TwoColumn.millimetres(), (183.0, 110.0));
    assert_eq!(Size::Slide.millimetres(), (254.0, 143.0));
    assert_eq!(Size::Millimetres(120.0, 80.5).millimetres(), (120.0, 80.5));
    // A point is 1/72 inch and an inch 25.4 mm.
    assert_eq!(MM, 72.0 / 25.4);
    let (width, height) = Size::OneColumn.points();
    assert!(
        (width - 252.283_464_566_929_1).abs() < 1e-9
            && (height - 187.086_614_173_228_3).abs() < 1e-9
    );
    assert_eq!(Size::Slide.points().0, 720.0);
    // A drawing is as large as its page.
    let figure = layout(
        &drawn(
            small(),
            &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
            Vec::new(),
        ),
        Size::Millimetres(100.0, 50.0),
        &JOURNAL,
    )
    .unwrap();
    assert_eq!(
        (figure.width, figure.height),
        (100.0 * 72.0 / 25.4, 50.0 * 72.0 / 25.4)
    );
    assert!((figure.width - 100.0 * MM).abs() < 1e-9);
}

// ----- a small figure, whole -----

#[test]
fn a_small_figure_reads_as_text() {
    // A category shown by colour and by shape has one legend; the size has its own.
    let both = drawn(
        small(),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "arm"),
            (Aesthetic::Shape, "arm"),
            (Aesthetic::Size, "weight"),
        ],
        Vec::new(),
    );
    let figure = layout(&both, Size::OneColumn, &JOURNAL).unwrap();
    insta::assert_snapshot!("small_one_column", figure.describe());
    // The plain scatter, and the same with a number shown on a ramp of colours.
    let plain = drawn(
        small(),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
        Vec::new(),
    );
    insta::assert_snapshot!(
        "small_plain",
        layout(&plain, Size::OneColumn, &JOURNAL)
            .unwrap()
            .describe()
    );
    let ramp = drawn(
        small(),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "weight"),
        ],
        Vec::new(),
    );
    insta::assert_snapshot!(
        "small_ramp",
        layout(&ramp, Size::TwoColumn, &JOURNAL).unwrap().describe()
    );
}

// ----- the six sample figures -----

/// The six scenes of the shipped wages that the release draws its sample figures from, as
/// far as a layer of points goes; step 21d writes them as files.
fn samples() -> Vec<(&'static str, Scene)> {
    let w = wages();
    let star = PlotConstant {
        aesthetic: Aesthetic::Shape,
        value: PlotSetting::Shape("star_open"),
    };
    let translucent = |mappings: &[(Aesthetic, &str)]| {
        let plot = start(
            w.clone(),
            mappings.iter().map(|(a, n)| mapped(*a, n)).collect(),
            Vec::new(),
        );
        with_layer(&plot, points(0.3)).1
    };
    vec![
        (
            "wage against schooling",
            drawn(
                w.clone(),
                &[(Aesthetic::X, "educ"), (Aesthetic::Y, "wage")],
                Vec::new(),
            ),
        ),
        (
            "wage against experience, translucent",
            translucent(&[(Aesthetic::X, "exper"), (Aesthetic::Y, "wage")]),
        ),
        (
            "coloured by union membership",
            translucent(&[
                (Aesthetic::X, "educ"),
                (Aesthetic::Y, "wage"),
                (Aesthetic::Color, "union"),
            ]),
        ),
        (
            "coloured by hours on a ramp",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "educ"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Color, "hours"),
                ],
                Vec::new(),
            ),
        ),
        (
            "sized by hours, coloured and shaped by sex",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "exper"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Size, "hours"),
                    (Aesthetic::Color, "female"),
                    (Aesthetic::Shape, "female"),
                ],
                Vec::new(),
            ),
        ),
        (
            "shaped by industry, twelve categories",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "age"),
                    (Aesthetic::Y, "hours"),
                    (Aesthetic::Shape, "industry"),
                ],
                Vec::new(),
            ),
        ),
        (
            "one shape for every mark",
            drawn(
                w,
                &[(Aesthetic::X, "educ"), (Aesthetic::Y, "hours")],
                vec![star],
            ),
        ),
    ]
}

#[test]
fn the_sample_figures_are_laid_out_at_the_three_named_sizes() {
    let mut all = String::new();
    for (title, sample) in samples() {
        let marks: usize = sample.panels[0].layers.iter().map(|l| l.x.len()).sum();
        assert!(marks > 18_000, "{title}: {marks}");
        for (name, size) in [
            ("one_column", Size::OneColumn),
            ("two_column", Size::TwoColumn),
            ("slide", Size::Slide),
        ] {
            let figure = layout(&sample, size, &JOURNAL)
                .unwrap_or_else(|why| panic!("{title} at {name}: {why:?}"));
            checks(&figure, &sample).unwrap_or_else(|why| panic!("{title} at {name}: {why}"));
            all.push_str(&format!("=== {title}, {name}\n{}", outline(&figure)));
        }
    }
    insta::assert_snapshot!("sample_figures", all);
}

// ----- what holds of every drawing -----

/// What must hold of the drawing of a scene: said as a sentence when it does not.
fn checks(figure: &Drawing, of: &Scene) -> Result<(), String> {
    let all = figure.primitives();
    let (width, height) = (figure.width, figure.height);
    let mut panel: Option<(f64, f64, f64, f64)> = None;
    let mut inside = 0;
    for item in &all {
        match &item.primitive {
            Primitive::Text { spans, .. } => {
                // No text below 8 points, and every text on the page.
                for span in spans {
                    if span.size < SMALLEST_TEXT {
                        return Err(format!("{:?} is set at {} points", span.text, span.size));
                    }
                }
                let (left, top, right, bottom) = extent(item).unwrap();
                if left < -0.5 || top < -0.5 || right > width + 0.5 || bottom > height + 0.5 {
                    return Err(format!(
                        "text off the page: {:?} at {left:.1}, {top:.1}, {right:.1}, {bottom:.1} of {width:.1} by {height:.1}",
                        spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>()
                    ));
                }
            }
            Primitive::Clip {
                at,
                width: w,
                height: h,
                items,
            } => {
                panel = Some((at.0, at.1, at.0 + w, at.1 + h));
                if *w < SMALLEST_PANEL || *h < SMALLEST_PANEL {
                    return Err(format!("a panel of {w} by {h}"));
                }
                if at.0 < 0.0 || at.1 < 0.0 || at.0 + w > width || at.1 + h > height {
                    return Err("a panel off the page".to_string());
                }
                // Every mark is centred inside the panel.
                for mark in items {
                    let Primitive::Point {
                        at: centre,
                        size,
                        opacity,
                        ..
                    } = &mark.primitive
                    else {
                        return Err("something other than a point in the panel".to_string());
                    };
                    if !(centre.0 >= at.0 - 1e-9
                        && centre.0 <= at.0 + w + 1e-9
                        && centre.1 >= at.1 - 1e-9
                        && centre.1 <= at.1 + h + 1e-9)
                    {
                        return Err(format!("a mark at {centre:?} outside the panel"));
                    }
                    if !(size.is_finite() && *size >= 0.0 && (0.0..=1.0).contains(opacity)) {
                        return Err(format!("a mark of size {size} and opacity {opacity}"));
                    }
                    inside += 1;
                }
            }
            Primitive::Segment { from, to, .. } => {
                for end in [from, to] {
                    if !(end.0 >= -0.5
                        && end.0 <= width + 0.5
                        && end.1 >= -0.5
                        && end.1 <= height + 0.5)
                    {
                        return Err(format!("a line to {end:?}, off the page"));
                    }
                }
            }
            _ => {}
        }
    }
    if panel.is_none() {
        return Err("no panel".to_string());
    }
    // One mark drawn for each mark of the scene.
    let marks: usize = of
        .panels
        .iter()
        .flat_map(|p| &p.layers)
        .map(|l| l.x.len())
        .sum();
    if inside != marks {
        return Err(format!("{inside} marks drawn of {marks}"));
    }
    // The first item is the page itself.
    match all.first().map(|item| &item.primitive) {
        Some(Primitive::Rectangle {
            at,
            width: w,
            height: h,
            ..
        }) if *at == (0.0, 0.0) && *w == width && *h == height => {}
        other => return Err(format!("the page is not drawn first: {other:?}")),
    }
    // Every name tells what a thing is.
    for item in &all {
        let name = item.name.as_deref().unwrap_or("");
        let known = ["background", "grid", "panel", "axis:x", "axis:y", "notes"].contains(&name)
            || name.starts_with("layer:")
            || name.starts_with("legend:");
        if !known {
            return Err(format!("an item named {name:?}"));
        }
    }
    Ok(())
}

/// A cell: `NA`, an everyday number, a large or a tiny one.
fn cell() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        1 => Just(None),
        6 => (-50.0..50.0f64).prop_map(Some),
        1 => (-1e9..1e9f64).prop_map(Some),
        1 => (-1e-6..1e-6f64).prop_map(Some),
        1 => Just(Some(f64::INFINITY)),
    ]
}

type Row = (Option<f64>, Option<f64>, Option<u8>, Option<f64>);

fn rows() -> impl Strategy<Value = Vec<Row>> {
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

fn frame_of(rows: &[Row]) -> Frame {
    frame(
        "d",
        vec![
            ("x", floats(&rows.iter().map(|r| r.0).collect::<Vec<_>>())),
            ("y", floats(&rows.iter().map(|r| r.1).collect::<Vec<_>>())),
            (
                "g",
                Column::strs(rows.iter().map(|r| r.2.map(|k| format!("group {k:02}")))),
            ),
            ("z", floats(&rows.iter().map(|r| r.3).collect::<Vec<_>>())),
        ],
    )
}

/// The ways the four variables are shown.
fn showing() -> impl Strategy<Value = Vec<(Aesthetic, &'static str)>> {
    prop::sample::select(vec![
        vec![],
        vec![(Aesthetic::Color, "g")],
        vec![(Aesthetic::Color, "z")],
        vec![(Aesthetic::Shape, "g")],
        vec![(Aesthetic::Size, "z")],
        vec![(Aesthetic::Color, "g"), (Aesthetic::Shape, "g")],
        vec![(Aesthetic::Color, "g"), (Aesthetic::Size, "z")],
        vec![
            (Aesthetic::Color, "z"),
            (Aesthetic::Size, "z"),
            (Aesthetic::Shape, "g"),
        ],
    ])
    .prop_map(|more| {
        let mut all = vec![(Aesthetic::X, "x"), (Aesthetic::Y, "y")];
        all.extend(more);
        all
    })
}

fn a_size() -> impl Strategy<Value = Size> {
    prop_oneof![
        Just(Size::OneColumn),
        Just(Size::TwoColumn),
        Just(Size::Slide),
        (60.0..400.0f64, 45.0..300.0f64).prop_map(|(w, h)| Size::Millimetres(w, h)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    #[test]
    fn every_drawing_keeps_its_text_readable_its_marks_in_the_panel_and_nothing_of_the_rows(
        rows in rows(),
        mappings in showing(),
        size in a_size(),
    ) {
        let sample = drawn(frame_of(&rows), &mappings, Vec::new());
        match layout(&sample, size, &JOURNAL) {
            Ok(figure) => {
                if let Err(why) = checks(&figure, &sample) {
                    prop_assert!(false, "{}\n{}", why, figure.describe());
                }
                // The same scene at the same size is the same drawing.
                let again = layout(&sample, size, &JOURNAL).unwrap();
                prop_assert_eq!(&again, &figure);
                // A drawing holds nothing of where its marks came from: the same scene with
                // other rows behind its marks, and from a frame of another name, draws the same.
                let mut elsewhere = sample.clone();
                for layer in &mut elsewhere.panels[0].layers {
                    layer.data = Some("another".to_string());
                    for row in &mut layer.rows {
                        *row += 1000;
                    }
                }
                prop_assert_eq!(&layout(&elsewhere, size, &JOURNAL).unwrap(), &figure);
                // The notes of the scene travel with the drawing, beside it and not in it.
                prop_assert_eq!(&figure.notes[..sample.notes.len()], &sample.notes[..]);
            }
            // A page too small for its labels says so, with what was left for the panel.
            Err(LayoutError::TooSmall { panel }) => {
                prop_assert!(panel.0 < SMALLEST_PANEL || panel.1 < SMALLEST_PANEL);
            }
            Err(other) => prop_assert!(false, "{:?}", other),
        }
    }
}

// ----- the edge cases -----

#[test]
fn a_plot_with_no_layer_and_a_plot_of_one_point() {
    let plot = start(
        small(),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    // No layer yet: the page, the grid, an empty panel and the two axes.
    let empty = scene(&plot);
    let figure = layout(&empty, Size::OneColumn, &JOURNAL).unwrap();
    checks(&figure, &empty).unwrap();
    assert!(figure.describe().contains("[panel] clip"));
    assert!(!figure.describe().contains("point "));
    // One variable alone: its axis, and no other.
    let alone = scene(&start(
        small(),
        vec![mapped(Aesthetic::X, "dose")],
        Vec::new(),
    ));
    let figure = layout(&alone, Size::OneColumn, &JOURNAL).unwrap();
    let text = figure.describe();
    assert!(
        text.contains("[axis:x]") && !text.contains("[axis:y]"),
        "{text}"
    );
    // Nothing mapped: the page and an empty panel.
    let nothing = scene(&start(small(), Vec::new(), Vec::new()));
    let figure = layout(&nothing, Size::OneColumn, &JOURNAL).unwrap();
    assert_eq!(figure.items.len(), 2);
    // One point: in the middle of its panel.
    let one = drawn(
        frame("d", vec![("x", numbers(&[3.0])), ("y", numbers(&[7.0]))]),
        &[(Aesthetic::X, "x"), (Aesthetic::Y, "y")],
        Vec::new(),
    );
    let figure = layout(&one, Size::OneColumn, &JOURNAL).unwrap();
    checks(&figure, &one).unwrap();
    let all = figure.primitives();
    let (clip, mark) = all
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } => match &items[0].primitive {
                Primitive::Point {
                    at: centre,
                    size,
                    shape,
                    ..
                } => {
                    assert_eq!((*size, *shape), (4.5, "circle"));
                    Some(((at.0 + width / 2.0, at.1 + height / 2.0), *centre))
                }
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    assert!((clip.0 - mark.0).abs() < 1e-9 && (clip.1 - mark.1).abs() < 1e-9);
}

#[test]
fn a_long_title_is_broken_between_words_and_labels_that_would_touch_are_thinned() {
    let mut long = drawn(
        small(),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
        Vec::new(),
    );
    long.scales[0].title =
        "Potential labour-market experience: age less schooling less six, in years".to_string();
    long.scales[1].title =
        "Response of the patient to the dose, on the scale of the trial".to_string();
    let figure = layout(&long, Size::OneColumn, &JOURNAL).unwrap();
    checks(&figure, &long).unwrap();
    let text = figure.describe();
    assert!(
        text.contains("\"Potential labour-market experience: age less\""),
        "{text}"
    );
    assert!(text.contains("\"schooling less six, in years\""), "{text}");
    assert!(
        text.contains("turned 90 \"Response of the patient to the\""),
        "{text}"
    );
    // On a wider page the x title is one line.
    let wide = layout(&long, Size::Slide, &JOURNAL).unwrap().describe();
    assert!(
        wide.contains(
            "\"Potential labour-market experience: age less schooling less six, in years\""
        ),
        "{wide}"
    );
    // A word wider than the side stands on a line of its own and the figure is still drawn.
    long.scales[0].title =
        "Pneumonoultramicroscopicsilicovolcanoconiosis-and-then-some-more-letters-to-be-sure x"
            .to_string();
    assert!(layout(&long, Size::OneColumn, &JOURNAL).is_ok());
    // Forty breaks on a narrow axis: every tick is drawn, and not every label.
    let mut dense = drawn(
        small(),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
        Vec::new(),
    );
    dense.scales[0].limits = (0.0, 4000.0);
    dense.scales[0].breaks = (0..=40).map(|i| f64::from(i) * 100.0).collect();
    dense.scales[0].labels = dense.scales[0]
        .breaks
        .iter()
        .map(|b| format!("{b}"))
        .collect();
    for layer in &mut dense.panels[0].layers {
        for x in &mut layer.x {
            *x *= 500.0;
        }
    }
    let figure = layout(&dense, Size::OneColumn, &JOURNAL).unwrap();
    checks(&figure, &dense).unwrap();
    let described = figure.describe();
    let lines: Vec<&str> = described
        .lines()
        .filter(|l| l.starts_with("[axis:x]"))
        .collect();
    let ticks = lines.iter().filter(|l| l.contains("segment")).count();
    let labels = lines
        .iter()
        .filter(|l| l.contains("text") && !l.contains("Dose"))
        .count();
    assert_eq!(ticks, 41 + 1, "the axis and a tick at each break");
    assert!((5..41).contains(&labels), "{labels} labels");
    // The labels written do not touch.
    let mut boxes: Vec<(f64, f64)> = figure
        .primitives()
        .iter()
        .filter(|item| item.name.as_deref() == Some("axis:x"))
        .filter_map(|item| extent(item))
        .filter(|b| b.3 < figure.height - 14.0)
        .map(|b| (b.0, b.2))
        .collect();
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(
        boxes.windows(2).all(|pair| pair[0].1 < pair[1].0),
        "{boxes:?}"
    );
}

#[test]
fn a_legend_of_twelve_categories_runs_into_columns_when_the_page_is_low() {
    let names: Vec<String> = (0..12).map(|i| format!("category number {i:02}")).collect();
    let d = frame(
        "d",
        vec![
            ("x", numbers(&(0..12).map(f64::from).collect::<Vec<_>>())),
            (
                "y",
                numbers(&(0..12).map(|i| f64::from(i % 5)).collect::<Vec<_>>()),
            ),
            ("g", Column::strs(names.iter().map(|n| Some(n.as_str())))),
        ],
    );
    let sample = drawn(
        d,
        &[
            (Aesthetic::X, "x"),
            (Aesthetic::Y, "y"),
            (Aesthetic::Color, "g"),
            (Aesthetic::Shape, "g"),
        ],
        Vec::new(),
    );
    // High enough for twelve rows: one column of keys.
    let tall = layout(&sample, Size::TwoColumn, &JOURNAL).unwrap();
    checks(&tall, &sample).unwrap();
    let key_columns = |figure: &Drawing| {
        let mut lefts: Vec<String> = figure
            .describe()
            .lines()
            .filter(|l| l.starts_with("[legend:color+shape] point"))
            .map(|l| {
                l.split(' ')
                    .nth(2)
                    .unwrap()
                    .split(',')
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect();
        lefts.sort();
        lefts.dedup();
        lefts.len()
    };
    assert_eq!(key_columns(&tall), 1);
    assert_eq!(
        tall.describe()
            .matches("[legend:color+shape] point")
            .count(),
        12
    );
    // The shapes repeat from the ninth category and the scene says so, beside the drawing.
    assert!(
        tall.notes
            .iter()
            .any(|note| note.contains("repeat from the ninth category"))
    );
    // A low page: the keys run into columns, and all twelve are there.
    let low = layout(&sample, Size::Millimetres(183.0, 45.0), &JOURNAL).unwrap();
    checks(&low, &sample).unwrap();
    assert!(key_columns(&low) >= 2, "{}", low.describe());
    assert_eq!(
        low.describe().matches("[legend:color+shape] point").count(),
        12
    );
}

#[test]
fn what_cannot_be_laid_out_says_why() {
    let sample = drawn(
        small(),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    // A page that leaves no room for a panel beside its labels.
    for size in [
        Size::Millimetres(20.0, 20.0),
        Size::Millimetres(200.0, 12.0),
        Size::Millimetres(15.0, 200.0),
    ] {
        match layout(&sample, size, &JOURNAL) {
            Err(LayoutError::TooSmall { panel }) => {
                assert!(
                    panel.0 < SMALLEST_PANEL || panel.1 < SMALLEST_PANEL,
                    "{panel:?}"
                );
            }
            other => panic!("{size:?}: {other:?}"),
        }
    }
    // The smallest page that is still drawn keeps its text at 8 points.
    let least = layout(&sample, Size::Millimetres(45.0, 30.0), &JOURNAL).unwrap();
    checks(&least, &sample).unwrap();
    // What is no size.
    for size in [
        Size::Millimetres(0.0, 50.0),
        Size::Millimetres(50.0, -1.0),
        Size::Millimetres(f64::NAN, 50.0),
        Size::Millimetres(50.0, f64::INFINITY),
    ] {
        assert_eq!(
            layout(&sample, size, &JOURNAL),
            Err(LayoutError::NotASize),
            "{size:?}"
        );
    }
    // A scene that holds a name reserved for a later release.
    let mut later = sample.clone();
    later.panels[0].crs = Some("EPSG:4326".to_string());
    match layout(&later, Size::OneColumn, &JOURNAL) {
        Err(LayoutError::Undrawn(why)) => {
            assert!(why.starts_with("this scene holds `crs`"), "{why}")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_character_no_font_has_is_drawn_and_told_of() {
    let mut sample = drawn(
        small(),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    sample.scales[0].title = "剂量 (mg)".to_string();
    sample.scales[2].labels[0] = "drug\tA".to_string();
    sample
        .notes
        .push("1 row with NA in dose dropped".to_string());
    let figure = layout(&sample, Size::OneColumn, &JOURNAL).unwrap();
    assert_eq!(
        figure.notes,
        [
            "1 row with NA in dose dropped",
            "a tab, `剂` and `量` are not in the font of the figure and are drawn as an empty box"
        ]
    );
    // One character: said in the singular.
    let mut one = drawn(
        small(),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
        Vec::new(),
    );
    one.scales[1].title = "réponse → 中".to_string();
    assert_eq!(
        layout(&one, Size::OneColumn, &JOURNAL).unwrap().notes,
        ["`中` is not in the font of the figure and is drawn as an empty box"]
    );
    // A figure all of whose characters the font has says nothing.
    let plain = drawn(
        small(),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
        Vec::new(),
    );
    assert!(
        layout(&plain, Size::OneColumn, &JOURNAL)
            .unwrap()
            .notes
            .is_empty()
    );
}

// ----- the notes under a figure, and a plot as the bytes of one (step 21d) -----

#[test]
fn the_notes_are_written_under_the_figure_only_when_they_are_asked_for() {
    let mut sample = drawn(
        small(),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    sample.notes = vec![
        "2 rows with NA in dose or response dropped".to_string(),
        "`arm` has 12 categories and the palette 8 colours, so the colours repeat from the ninth category, which a reader of the figure must be told and which makes a line too long for a narrow page".to_string(),
    ];
    let plain = layout(&sample, Size::OneColumn, &JOURNAL).unwrap();
    let noted = layout_with_notes(&sample, Size::OneColumn, &JOURNAL).unwrap();
    checks(&plain, &sample).unwrap();
    checks(&noted, &sample).unwrap();
    // Beside the drawing either way; on it only when asked for.
    assert_eq!(plain.notes, sample.notes);
    assert_eq!(noted.notes, sample.notes);
    assert!(!plain.describe().contains("[notes]"));
    let lines: Vec<String> = noted
        .describe()
        .lines()
        .filter(|l| l.starts_with("[notes]"))
        .map(str::to_string)
        .collect();
    // Each note as the console prints it, the long one broken to the width of the page.
    assert!(lines.len() >= 4, "{lines:?}");
    assert!(
        lines[0].contains("\"note: 2 rows with NA in dose or response dropped\""),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains("\"note: `arm` has 12 categories"),
        "{}",
        lines[1]
    );
    assert!(
        lines
            .iter()
            .all(|l| l.contains(" start ") && l.contains(" sans 8 "))
    );
    // The notes are at the foot of the page, under one another, and on it.
    let places: Vec<f64> = noted
        .primitives()
        .iter()
        .filter(|item| item.name.as_deref() == Some("notes"))
        .filter_map(|item| extent(item))
        .map(|b| {
            assert!(
                b.0 >= 0.0 && b.2 <= noted.width + 0.5 && b.3 <= noted.height,
                "{b:?}"
            );
            b.3
        })
        .collect();
    assert!(places.windows(2).all(|pair| pair[0] < pair[1]));
    // They take their room from the panel: it is lower by what they need, and as wide.
    let panel = |figure: &Drawing| {
        figure
            .primitives()
            .iter()
            .find_map(|item| match &item.primitive {
                Primitive::Clip { width, height, .. } => Some((*width, *height)),
                _ => None,
            })
            .unwrap()
    };
    let (before, after) = (panel(&plain), panel(&noted));
    assert_eq!(before.0, after.0);
    assert!(
        (before.1 - after.1 - (lines.len() as f64 * 10.0 + 5.0)).abs() < 1e-9,
        "{before:?} {after:?}"
    );
    // A scene with nothing to tell is the same figure with or without.
    sample.notes.clear();
    assert_eq!(
        layout_with_notes(&sample, Size::OneColumn, &JOURNAL),
        layout(&sample, Size::OneColumn, &JOURNAL)
    );
}

#[test]
fn a_plot_is_written_as_the_figure_asked_for_and_what_cannot_be_drawn_is_said() {
    use zeff_core::value::{FigureFormat, FigureRequest, FigureSize};
    let plot = start(
        small(),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    let (plot, sample) = with_layer(&plot, points(1.0));
    // Every plot this package makes carries its writer.
    let writer = plot.figure.expect("the writer of the package").0;
    let request = |size: FigureSize, notes: bool| FigureRequest {
        format: FigureFormat::Svg,
        size,
        notes,
    };
    // The bytes are those of the scene laid out at that size and written as SVG.
    for (asked, laid) in [
        (FigureSize::OneColumn, Size::OneColumn),
        (FigureSize::TwoColumn, Size::TwoColumn),
        (FigureSize::Slide, Size::Slide),
        (
            FigureSize::Millimetres(120.0, 80.0),
            Size::Millimetres(120.0, 80.0),
        ),
    ] {
        let bytes = writer(&plot, &request(asked, false)).unwrap();
        let expected = zeff_plot::svg::write(&layout(&sample, laid, &JOURNAL).unwrap());
        assert_eq!(String::from_utf8(bytes).unwrap(), expected, "{asked:?}");
    }
    assert_eq!(FigureSize::DEFAULT, FigureSize::OneColumn);
    // What cannot be drawn, in the words of this package.
    let small_page =
        writer(&plot, &request(FigureSize::Millimetres(30.0, 20.0), false)).unwrap_err();
    assert!(small_page.of_size);
    assert!(
        small_page.message.starts_with("a page of 30 by 20 mm is too small for this plot: its labels, never set below 8 points, leave "),
        "{}",
        small_page.message
    );
    assert!(
        small_page
            .help
            .starts_with("a panel needs 12.7 mm each way"),
        "{}",
        small_page.help
    );
    let no_size = writer(&plot, &request(FigureSize::Millimetres(-5.0, 20.0), false)).unwrap_err();
    assert!(no_size.of_size);
    assert_eq!(
        no_size.message,
        "the size of a figure is a width and a height above 0, in millimetres"
    );
}
