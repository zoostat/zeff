//! The SVG of a figure (Language Reference, section 7.6; release 0.7.0, step 21c; open
//! decisions #118 and #126): the bytes a drawing is written as. Every primitive of the closed
//! set, written and kept as a snapshot; a small figure kept whole; the sample figures of the
//! shipped wages at the three named sizes, each read back by an independent reader of XML and
//! kept as its size and its hash; and what holds of every file: it is well-formed, it has one
//! element for each mark, its text is outlines with the words kept beside them, no type is
//! below 8 points, it holds nothing of the rows or of the name of the frame (F2), and the same
//! drawing gives the same bytes.

use proptest::prelude::*;
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::value::{Aesthetic, Frame, Mark, PlotConstant, PlotLayer, PlotMapping, PlotSetting};
use zeff_plot::drawing::{Anchor, Drawing, Head, Item, Primitive, Span, Step, Stroke, TextBox};
use zeff_plot::font::FaceName;
use zeff_plot::layout::{JOURNAL, SMALLEST_TEXT, Size, layout};
use zeff_plot::scene::{Scene, start, with_layer};
use zeff_plot::svg::{MARK_STROKE, write};

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

fn drawn_with(
    frame: Frame,
    mappings: &[(Aesthetic, &str)],
    constants: Vec<PlotConstant>,
    alpha: f64,
) -> Scene {
    let plot = start(
        frame,
        mappings.iter().map(|(a, n)| mapped(*a, n)).collect(),
        constants,
    );
    with_layer(&plot, points(alpha)).1
}

fn drawn(frame: Frame, mappings: &[(Aesthetic, &str)]) -> Scene {
    drawn_with(frame, mappings, Vec::new(), 1.0)
}

fn wages() -> Frame {
    Frame {
        table: Rc::new(zeff_datasets::table("wages").expect("the shipped wages")),
        name: Some("wages".to_string()),
    }
}

fn small(name: &str) -> Frame {
    frame(
        name,
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

/// The marks of a drawing: the points inside its clip regions.
fn marks(drawing: &Drawing) -> usize {
    fn count(items: &[Item]) -> usize {
        items
            .iter()
            .map(|item| match &item.primitive {
                Primitive::Clip { items, .. } => items
                    .iter()
                    .filter(|i| matches!(i.primitive, Primitive::Point { .. }))
                    .count(),
                _ => 0,
            })
            .sum()
    }
    count(&drawing.items)
}

/// What must hold of the file a drawing is written as: said as a sentence when it does not.
fn checks(svg: &str, drawing: &Drawing) -> Result<(), String> {
    // An independent reader of XML takes it.
    let document = roxmltree::Document::parse(svg).map_err(|why| format!("not XML: {why}"))?;
    let root = document.root_element();
    if root.tag_name().name() != "svg"
        || root.tag_name().namespace() != Some("http://www.w3.org/2000/svg")
    {
        return Err("the root is not an svg of the SVG namespace".to_string());
    }
    // The page is the drawing's, in points.
    let size = |name: &str| -> Result<f64, String> {
        root.attribute(name)
            .and_then(|v| v.strip_suffix("pt"))
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or(format!("no {name} in points"))
    };
    if (size("width")? - drawing.width).abs() > 0.0006
        || (size("height")? - drawing.height).abs() > 0.0006
    {
        return Err("the page is not as large as the drawing".to_string());
    }
    // Nothing that is not a number where a number stands, and no control character.
    for bad in ["NaN", "inf", "\u{0}", "\u{7}"] {
        if svg.contains(bad) {
            return Err(format!("the file holds {bad:?}"));
        }
    }
    let mut ids: Vec<&str> = Vec::new();
    let (mut texts, mut labelled, mut in_clip) = (0, 0, 0);
    for node in document.descendants().filter(|n| n.is_element()) {
        if let Some(id) = node.attribute("id") {
            if ids.contains(&id) {
                return Err(format!("the id {id} twice"));
            }
            ids.push(id);
        }
        for attribute in node.attributes() {
            // Nothing is carried for a program to read back: no data attribute, no link.
            if attribute.name().starts_with("data") || attribute.name() == "href" {
                return Err(format!("an attribute {}", attribute.name()));
            }
        }
        match node.tag_name().name() {
            "text" => {
                texts += 1;
                // The words of a text are kept, not painted, at the size of the type.
                if node.attribute("fill") != Some("none") {
                    return Err("a text that is painted".to_string());
                }
                let type_size: f64 = node
                    .attribute("font-size")
                    .and_then(|v| v.parse().ok())
                    .ok_or("a text without a size")?;
                if type_size < SMALLEST_TEXT {
                    return Err(format!("type of {type_size} points"));
                }
                let words = node.text().unwrap_or("");
                if node.parent().and_then(|p| p.attribute("aria-label")) != Some(words) {
                    return Err(format!(
                        "the words {words:?} are not the label of their group"
                    ));
                }
            }
            "g" if node.has_attribute("aria-label") => labelled += 1,
            "circle" | "path" if node.ancestors().any(|a| a.attribute("clip-path").is_some()) => {
                in_clip += 1;
            }
            _ => {}
        }
    }
    if texts != labelled {
        return Err(format!("{texts} texts and {labelled} labelled groups"));
    }
    // One element for each mark.
    if in_clip != marks(drawing) {
        return Err(format!("{in_clip} marks written of {}", marks(drawing)));
    }
    // One text for each text of the drawing that has words.
    let written = drawing
        .primitives()
        .iter()
        .filter(|item| match &item.primitive {
            Primitive::Text { spans, .. } => spans.iter().any(|s| !s.text.is_empty()),
            _ => false,
        })
        .count();
    if texts != written {
        return Err(format!("{texts} texts written of {written}"));
    }
    Ok(())
}

// ----- every primitive -----

#[test]
fn every_primitive_of_the_closed_set_is_written() {
    let line = |width: f64| Stroke {
        colour: "#222222".to_string(),
        width,
    };
    let unnamed = |primitive: Primitive| Item {
        name: None,
        primitive,
    };
    let named = |name: &str, primitive: Primitive| Item {
        name: Some(name.to_string()),
        primitive,
    };
    let point = |x: f64, shape: &'static str, colour: &str, opacity: f64| Primitive::Point {
        at: (x, 30.0),
        shape,
        size: 9.0,
        colour: colour.to_string(),
        opacity,
    };
    let span = |words: &str, face: FaceName, size: f64| Span {
        text: words.to_string(),
        face,
        size,
        colour: "#222222".to_string(),
    };
    let drawing = Drawing {
        width: 200.0,
        height: 120.0,
        items: vec![
            named(
                "background",
                Primitive::Rectangle {
                    at: (0.0, 0.0),
                    width: 200.0,
                    height: 120.0,
                    radius: 0.0,
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                },
            ),
            // The fifteen shapes: two runs of one paint, then one mark of each kind.
            named(
                "panel",
                Primitive::Clip {
                    at: (5.0, 5.0),
                    width: 190.0,
                    height: 50.0,
                    items: vec![
                        named("layer:1", point(12.0, "circle", "#E69F00", 1.0)),
                        named("layer:1", point(24.0, "triangle", "#E69F00", 1.0)),
                        named("layer:1", point(36.0, "square", "#E69F00", 1.0)),
                        named("layer:1", point(48.0, "diamond", "#56B4E9", 0.5)),
                        named("layer:1", point(60.0, "triangle_down", "#56B4E9", 0.5)),
                        named("layer:1", point(72.0, "star", "#56B4E9", 0.5)),
                        named("layer:1", point(84.0, "plus", "#56B4E9", 0.5)),
                        named("layer:1", point(96.0, "cross", "#222222", 1.0)),
                        named("layer:1", point(108.0, "asterisk", "#222222", 1.0)),
                        named("layer:1", point(120.0, "circle_open", "#222222", 1.0)),
                        named("layer:1", point(132.0, "star_open", "#222222", 1.0)),
                        named(
                            "layer:2",
                            point(144.0, "triangle_down_open", "#009E73", 1.0),
                        ),
                        named("layer:2", point(156.0, "square_open", "#009E73", 1.0)),
                        named("layer:2", point(168.0, "diamond_open", "#009E73", 1.0)),
                        named("layer:2", point(180.0, "triangle_open", "#009E73", 1.0)),
                        // A name that is no shape is drawn as a circle.
                        named("layer:2", point(190.0, "hexagon", "#009E73", 1.0)),
                    ],
                },
            ),
            unnamed(Primitive::Path {
                steps: vec![
                    Step::Move((10.0, 70.0)),
                    Step::Line((30.0, 62.0)),
                    Step::Curve((40.0, 60.0), (50.0, 80.0), (60.0, 70.0)),
                    Step::Arc {
                        radii: (8.0, 5.0),
                        turn: 30.0,
                        larger: false,
                        clockwise: true,
                        to: (76.0, 70.0),
                    },
                    Step::Close,
                ],
                stroke: Some(line(0.75)),
                fill: Some("#F0E442".to_string()),
                opacity: 0.6,
            }),
            unnamed(Primitive::Polygon {
                rings: vec![
                    vec![(90.0, 60.0), (120.0, 60.0), (120.0, 80.0), (90.0, 80.0)],
                    vec![(98.0, 65.0), (112.0, 65.0), (112.0, 75.0), (98.0, 75.0)],
                ],
                fill: Some("#0072B2".to_string()),
                stroke: None,
                opacity: 1.0,
            }),
            unnamed(Primitive::Rectangle {
                at: (130.0, 60.0),
                width: 24.0,
                height: 20.0,
                radius: 3.0,
                fill: None,
                stroke: Some(line(0.5)),
            }),
            named(
                "axis:x",
                Primitive::Segment {
                    from: (10.0, 90.0),
                    to: (80.0, 90.0),
                    stroke: line(0.5),
                    heads: (Head::Plain, Head::Arrow),
                },
            ),
            named(
                "axis:x",
                Primitive::Segment {
                    from: (100.0, 96.0),
                    to: (150.0, 86.0),
                    stroke: line(1.0),
                    heads: (Head::Arrow, Head::Arrow),
                },
            ),
            named(
                "axis:x",
                Primitive::Text {
                    at: (45.0, 104.0),
                    anchor: Anchor::Middle,
                    turn: 0.0,
                    spans: vec![span("Wage & hours <2010>", FaceName::Sans, 9.0)],
                    frame: None,
                },
            ),
            // Two spans in two faces, in a box; and a text read upwards, in a box.
            unnamed(Primitive::Text {
                at: (120.0, 112.0),
                anchor: Anchor::Start,
                turn: 0.0,
                spans: vec![
                    span("R² = ", FaceName::SansSemiBold, 10.0),
                    span("0.47", FaceName::Mono, 10.0),
                ],
                frame: Some(TextBox {
                    padding: 2.0,
                    radius: 1.5,
                    fill: Some("#F7F7F7".to_string()),
                    border: Some(line(0.4)),
                }),
            }),
            unnamed(Primitive::Text {
                at: (190.0, 100.0),
                anchor: Anchor::End,
                turn: 90.0,
                spans: vec![span("up", FaceName::Sans, 8.0)],
                frame: Some(TextBox {
                    padding: 1.0,
                    radius: 0.0,
                    fill: None,
                    border: Some(line(0.4)),
                }),
            }),
            // A text of nothing is not written.
            unnamed(Primitive::Text {
                at: (0.0, 0.0),
                anchor: Anchor::Start,
                turn: 0.0,
                spans: vec![span("", FaceName::Sans, 8.0)],
                frame: None,
            }),
            // A grid of cells: neighbours of one colour in a row are one rectangle.
            named(
                "legend:color",
                Primitive::Raster {
                    at: (160.0, 60.0),
                    width: 30.0,
                    height: 20.0,
                    columns: 3,
                    rows: 2,
                    cells: [
                        "#440154", "#440154", "#21918C", "#FDE725", "#21918C", "#FDE725",
                    ]
                    .map(String::from)
                    .to_vec(),
                },
            ),
            // The name of the first group once more: its id is numbered.
            named(
                "background",
                Primitive::Segment {
                    from: (0.0, 119.0),
                    to: (200.0, 119.0),
                    stroke: line(0.25),
                    heads: (Head::Plain, Head::Plain),
                },
            ),
        ],
        notes: vec!["a note is not drawn".to_string()],
    };
    let svg = write(&drawing);
    checks(&svg, &drawing).unwrap();
    insta::assert_snapshot!("primitives", svg);
    // What the snapshot shows, said: the paint of a run is on its group, an outline shape is
    // stroked, a hole is cut by the even-odd rule, and a note is nowhere in the file.
    assert!(svg.contains("<g fill=\"#56B4E9\" fill-opacity=\"0.5\">"));
    // A mark with no neighbour of its paint carries its own, without a group.
    assert!(svg.contains(" r=\"4.5\" fill=\"#009E73\"/>"), "{svg}");
    assert!(svg.contains(&format!(
        "<g fill=\"none\" stroke=\"#222222\" stroke-width=\"{MARK_STROKE}\">"
    )));
    assert!(svg.contains("fill-rule=\"evenodd\""));
    assert!(svg.contains("<g id=\"axis-x\">") && svg.contains("<g id=\"background-2\">"));
    assert!(svg.contains("aria-label=\"Wage &amp; hours &lt;2010&gt;\""));
    assert!(!svg.contains("a note is not drawn"));
    assert!(svg.ends_with("</svg>\n"));
}

// ----- figures -----

#[test]
fn a_small_figure_is_written_whole() {
    let scene = drawn(
        small("d"),
        &[
            (Aesthetic::X, "dose"),
            (Aesthetic::Y, "response"),
            (Aesthetic::Color, "arm"),
            (Aesthetic::Shape, "arm"),
        ],
    );
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let svg = write(&drawing);
    checks(&svg, &drawing).unwrap();
    insta::assert_snapshot!("small_figure", svg);
}

/// The scenes of the shipped wages the release draws its sample figures from.
fn samples() -> Vec<(&'static str, Scene)> {
    let w = wages();
    let star = PlotConstant {
        aesthetic: Aesthetic::Shape,
        value: PlotSetting::Shape("star_open"),
    };
    vec![
        (
            "wage_schooling",
            drawn(w.clone(), &[(Aesthetic::X, "educ"), (Aesthetic::Y, "wage")]),
        ),
        (
            "wage_experience_translucent",
            drawn_with(
                w.clone(),
                &[(Aesthetic::X, "exper"), (Aesthetic::Y, "wage")],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "coloured_by_union",
            drawn_with(
                w.clone(),
                &[
                    (Aesthetic::X, "educ"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Color, "union"),
                ],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "coloured_by_hours",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "educ"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Color, "hours"),
                ],
            ),
        ),
        (
            "sized_coloured_shaped",
            drawn_with(
                w.clone(),
                &[
                    (Aesthetic::X, "exper"),
                    (Aesthetic::Y, "wage"),
                    (Aesthetic::Size, "hours"),
                    (Aesthetic::Color, "female"),
                    (Aesthetic::Shape, "female"),
                ],
                Vec::new(),
                0.3,
            ),
        ),
        (
            "shaped_by_industry",
            drawn(
                w.clone(),
                &[
                    (Aesthetic::X, "age"),
                    (Aesthetic::Y, "hours"),
                    (Aesthetic::Shape, "industry"),
                ],
            ),
        ),
        (
            "one_shape",
            drawn_with(
                w,
                &[(Aesthetic::X, "educ"), (Aesthetic::Y, "hours")],
                vec![star],
                1.0,
            ),
        ),
    ]
}

#[test]
fn the_sample_figures_are_written_and_read_back() {
    // Asked for with `ZEFF_FIGURES_DIR`, the figures are also written there as files, for the
    // eye; the tests do not need it.
    let folder = std::env::var("ZEFF_FIGURES_DIR").ok();
    let mut all = String::new();
    for (name, scene) in samples() {
        for (page, size) in [
            ("one_column", Size::OneColumn),
            ("two_column", Size::TwoColumn),
            ("slide", Size::Slide),
        ] {
            let drawing = layout(&scene, size, &JOURNAL).unwrap();
            let svg = write(&drawing);
            checks(&svg, &drawing).unwrap_or_else(|why| panic!("{name} at {page}: {why}"));
            // The frame is called `wages`; the file does not say so, nor anything of a row.
            assert!(!svg.contains("wages"), "{name} at {page}");
            assert_eq!(write(&drawing), svg);
            all.push_str(&format!(
                "{name} at {page}: {} bytes, {} marks, {}\n",
                svg.len(),
                marks(&drawing),
                zeff_hash::hash_hex(svg.as_bytes())
            ));
            if let Some(folder) = &folder {
                std::fs::write(format!("{folder}/{name}_{page}.svg"), &svg).unwrap();
            }
        }
    }
    insta::assert_snapshot!("sample_figures", all);
}

#[test]
fn a_file_holds_nothing_of_the_rows_or_of_the_name_of_the_frame() {
    let mappings = [
        (Aesthetic::X, "dose"),
        (Aesthetic::Y, "response"),
        (Aesthetic::Color, "arm"),
    ];
    let scene = drawn(small("patients_of_ward_7"), &mappings);
    assert_eq!(
        scene.panels[0].layers[0].data.as_deref(),
        Some("patients_of_ward_7")
    );
    assert_eq!(scene.panels[0].layers[0].rows, [1, 2, 3, 4, 5, 6]);
    let svg = write(&layout(&scene, Size::OneColumn, &JOURNAL).unwrap());
    assert!(!svg.contains("patients_of_ward_7") && !svg.contains("ward"));
    // The same marks from other rows of a frame of another name: the same bytes.
    let mut other = drawn(small("d"), &mappings);
    for row in &mut other.panels[0].layers[0].rows {
        *row += 40_000;
    }
    assert_eq!(
        write(&layout(&other, Size::OneColumn, &JOURNAL).unwrap()),
        svg
    );
    // The notes of the scene, which may count rows, are not in the file either.
    let mut noted = scene.clone();
    noted
        .notes
        .push("3 rows with NA in dose dropped".to_string());
    let drawing = layout(&noted, Size::OneColumn, &JOURNAL).unwrap();
    assert_eq!(drawing.notes, ["3 rows with NA in dose dropped"]);
    assert_eq!(write(&drawing), svg);
}

#[test]
fn words_are_kept_as_they_are_written_and_what_xml_cannot_hold_is_left_out() {
    let mut scene = drawn(
        small("d"),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
    );
    scene.scales[0].title = "Dose <mg> & \"more\"".to_string();
    scene.scales[1].title = "réponse\u{7} β\tγ".to_string();
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let svg = write(&drawing);
    checks(&svg, &drawing).unwrap();
    assert!(
        svg.contains("<g aria-label=\"Dose &lt;mg&gt; &amp; &quot;more&quot;\">"),
        "{svg}"
    );
    assert!(svg.contains(">Dose &lt;mg&gt; &amp; &quot;more&quot;</text>"));
    // A bell is left out and a tab is a space; letters beyond ASCII are written as they are.
    assert!(svg.contains("aria-label=\"réponse β γ\""), "{svg}");
    let document = roxmltree::Document::parse(&svg).unwrap();
    let labels: Vec<&str> = document
        .descendants()
        .filter_map(|n| n.attribute("aria-label"))
        .collect();
    assert!(labels.contains(&"Dose <mg> & \"more\""), "{labels:?}");
}

// ----- what holds of every file -----

fn cell() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        1 => Just(None),
        6 => (-50.0..50.0f64).prop_map(Some),
        1 => (-1e9..1e9f64).prop_map(Some),
        1 => (-1e-6..1e-6f64).prop_map(Some),
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
        0..30,
    )
}

fn showing() -> impl Strategy<Value = Vec<(Aesthetic, &'static str)>> {
    prop::sample::select(vec![
        vec![],
        vec![(Aesthetic::Color, "g")],
        vec![(Aesthetic::Color, "z")],
        vec![(Aesthetic::Shape, "g")],
        vec![(Aesthetic::Size, "z")],
        vec![
            (Aesthetic::Color, "g"),
            (Aesthetic::Shape, "g"),
            (Aesthetic::Size, "z"),
        ],
    ])
    .prop_map(|more| {
        let mut all = vec![(Aesthetic::X, "x"), (Aesthetic::Y, "y")];
        all.extend(more);
        all
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(120)))]

    #[test]
    fn every_file_is_well_formed_has_its_marks_and_its_words_and_is_written_the_same_twice(
        rows in rows(),
        mappings in showing(),
        size in prop_oneof![
            Just(Size::OneColumn),
            Just(Size::TwoColumn),
            (70.0..300.0f64, 50.0..250.0f64).prop_map(|(w, h)| Size::Millimetres(w, h)),
        ],
        alpha in prop_oneof![Just(1.0), 0.05..1.0f64],
    ) {
        let d = frame(
            "d",
            vec![
                ("x", floats(&rows.iter().map(|r| r.0).collect::<Vec<_>>())),
                ("y", floats(&rows.iter().map(|r| r.1).collect::<Vec<_>>())),
                ("g", Column::strs(rows.iter().map(|r| r.2.map(|k| format!("group <{k:02}> & co"))))),
                ("z", floats(&rows.iter().map(|r| r.3).collect::<Vec<_>>())),
            ],
        );
        let scene = drawn_with(d, &mappings, Vec::new(), alpha);
        if let Ok(drawing) = layout(&scene, size, &JOURNAL) {
            let svg = write(&drawing);
            if let Err(why) = checks(&svg, &drawing) {
                prop_assert!(false, "{}\n{}", why, svg);
            }
            prop_assert_eq!(&write(&drawing), &svg);
            prop_assert!(svg.is_ascii() || svg.chars().all(|c| !c.is_control() || c == '\n'));
        }
    }
}
