//! The SVG of a figure (Language Reference, section 7.6; release 0.7.0, step 21c; open
//! decisions #118 and #126): the bytes a drawing is written as. Every primitive of the closed
//! set, written and kept as a snapshot; a small figure kept whole; the sample figures of the
//! shipped wages at the three named sizes, each read back by an independent reader of XML and
//! kept as its size and its hash; and what holds of every file: it is well-formed, it has one
//! element for each mark, its text is outlines with the words kept beside them, no type is
//! below 8 points, it holds nothing of the rows or of the name of the frame (F2), and the same
//! drawing gives the same bytes.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::Aesthetic;
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::layout::{JOURNAL, SMALLEST_TEXT, Size, layout};
use zeff_plot::svg::{MARK_STROKE, write};

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
    let drawing = every_primitive();
    let svg = write(&drawing);
    checks(&svg, &drawing).unwrap();
    insta::assert_snapshot!("primitives", svg);
    if let Ok(folder) = std::env::var("ZEFF_FIGURES_DIR") {
        std::fs::write(format!("{folder}/primitives.svg"), &svg).unwrap();
    }
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
        let d = tried(&rows);
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
