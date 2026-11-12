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
use zeff_plot::drawing::{Drawing, Item, LineType, Primitive};
use zeff_plot::layout::{JOURNAL, SMALLEST_TEXT, Size, layout};
use zeff_plot::svg::{MARK_STROKE, MOST_RECTANGLES, write};

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
    // Nothing that is not a number where a number stands, and no control character; the
    // letters a picture is held as may spell anything, and are left out of the search.
    let mut drawn = String::new();
    for (i, part) in svg.split("base64,").enumerate() {
        drawn.push_str(if i == 0 {
            part
        } else {
            part.split_once('"').map_or("", |(_, after)| after)
        });
    }
    for bad in ["NaN", "inf", "\u{0}", "\u{7}"] {
        if drawn.contains(bad) {
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
            // Nothing is carried for a program to read back: no data attribute, and no link
            // but a picture held inside the file itself.
            let held = node.has_tag_name("image")
                && attribute.value().starts_with("data:image/png;base64,");
            if attribute.name().starts_with("data") || (attribute.name() == "href" && !held) {
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

// ----- a layer drawn in dots (step 22b3b) -----

/// Text of 64 letters back to the bytes it holds.
fn bytes_of(letters: &str) -> Vec<u8> {
    let value = |letter: u8| match letter {
        b'A'..=b'Z' => u32::from(letter - b'A'),
        b'a'..=b'z' => u32::from(letter - b'a') + 26,
        b'0'..=b'9' => u32::from(letter - b'0') + 52,
        b'+' => 62,
        b'/' => 63,
        other => panic!("{other} is not one of the 64 letters"),
    };
    let mut out = Vec::new();
    for four in letters.as_bytes().chunks(4) {
        assert_eq!(four.len(), 4, "letters not in fours");
        let filled = four.iter().filter(|&&letter| letter == b'=').count();
        let mut word = 0;
        for &letter in &four[..4 - filled] {
            word = word << 6 | value(letter);
        }
        word <<= 6 * filled as u32;
        out.extend_from_slice(&word.to_be_bytes()[1..4 - filled]);
    }
    out
}

#[test]
fn a_layer_flagged_to_be_drawn_in_dots_is_a_picture_inside_the_file() {
    let lines = layout(&two_layers(false, false), Size::OneColumn, &JOURNAL).unwrap();
    let dots = layout(&two_layers(true, false), Size::OneColumn, &JOURNAL).unwrap();
    let pictures = pictures_in(&dots);
    assert_eq!(pictures.len(), 1);
    let (columns, rows, wanted) = &pictures[0];
    let svg = write(&dots);
    checks(&svg, &dots).unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let images: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("image"))
        .collect();
    assert_eq!(images.len(), 1);
    let image = images[0];
    // The picture is held inside the file as a PNG, written as letters: no file is named.
    let held = image
        .attribute(("http://www.w3.org/1999/xlink", "href"))
        .and_then(|link| link.strip_prefix("data:image/png;base64,"))
        .expect("a picture held inside the file");
    let file = bytes_of(held);
    let decoder = png::Decoder::new(std::io::Cursor::new(&file[..]));
    let mut reader = decoder.read_info().unwrap();
    let mut found = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut found).unwrap();
    found.truncate(frame.buffer_size());
    assert_eq!(
        (frame.width as usize, frame.height as usize),
        (*columns, *rows)
    );
    assert!(
        &found == wanted,
        "the dots of the picture are not those of the layer"
    );
    // It is stretched over its place, which starts where the panel does, inside the region
    // that clips to the panel.
    let (at, _, _, _) = panel_of(&dots);
    let number = |name: &str| {
        image
            .attribute(name)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap()
    };
    assert!((number("x") - at.0).abs() < 0.0006 && (number("y") - at.1).abs() < 0.0006);
    assert!((number("width") - *columns as f64 * 0.24).abs() < 0.0006);
    assert!((number("height") - *rows as f64 * 0.24).abs() < 0.0006);
    assert_eq!(image.attribute("preserveAspectRatio"), Some("none"));
    assert!(
        image
            .ancestors()
            .any(|a| a.attribute("clip-path").is_some())
    );
    assert_eq!(
        image.parent().and_then(|group| group.attribute("id")),
        Some("layer-1")
    );
    // The six marks of the layer over it are lines still (`checks` counted them), and the
    // words are those of the figure without the flag.
    assert_eq!(svg.matches("<circle").count(), 6);
    let plain = write(&lines);
    let labels = |file: &str| -> Vec<String> {
        roxmltree::Document::parse(file)
            .unwrap()
            .descendants()
            .filter_map(|node| node.attribute("aria-label").map(String::from))
            .collect()
    };
    assert_eq!(labels(&svg), labels(&plain));
    // A file with no picture does not speak of one.
    assert!(!plain.contains("<image") && !plain.contains("xlink"));
    assert!(svg.starts_with(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width="
    ));
    // The same drawing, the same bytes.
    assert_eq!(write(&dots), svg);
    insta::assert_snapshot!(
        "picture_layer",
        format!(
            "a picture of {columns} by {rows} dots, {}; the file {} bytes, {}",
            zeff_hash::hash_hex(wanted),
            svg.len(),
            zeff_hash::hash_hex(svg.as_bytes())
        )
    );
    if let Ok(folder) = std::env::var("ZEFF_FIGURES_DIR") {
        std::fs::write(format!("{folder}/picture_layer.svg"), &svg).unwrap();
    }
}

#[test]
fn a_grid_of_few_cells_is_rectangles_and_one_of_many_a_picture() {
    let grid = |columns: usize, rows: usize| Drawing {
        width: 100.0,
        height: 100.0,
        items: vec![Item {
            name: None,
            primitive: Primitive::Raster {
                at: (10.0, 10.0),
                width: 80.0,
                height: 80.0,
                columns,
                rows,
                cells: (0..columns * rows)
                    .map(|i| [(i % 251) as u8, (i % 7) as u8 * 30, 99])
                    .collect(),
            },
        }],
        notes: Vec::new(),
    };
    assert_eq!(MOST_RECTANGLES, 4096);
    // Sixty-four by sixty-four is the most that is drawn cell by cell.
    let few = write(&grid(64, 64));
    assert!(few.contains("<rect") && !few.contains("<image"));
    checks(&few, &grid(64, 64)).unwrap();
    let many = write(&grid(65, 64));
    assert!(
        many.contains("<image x=\"10\" y=\"10\" width=\"80\" height=\"80\" ")
            && !many.contains("<rect")
    );
    checks(&many, &grid(65, 64)).unwrap();
    assert!(many.len() < few.len() / 4);
}

// ----- lines with a type (step 23b) -----

#[test]
fn a_line_that_is_not_whole_says_its_ink_and_its_gaps() {
    let drawing = six_lines();
    let svg = write(&drawing);
    checks(&svg, &drawing).unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    // The six lines, each 1.5 wide: its ink and gaps are those of its type times its width,
    // and a whole line says nothing.
    let lines: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("line"))
        .collect();
    assert_eq!(lines.len(), 6);
    for (line, dash) in lines.iter().zip(LineType::ALL) {
        let wanted: Vec<f64> = dash.lengths(1.5);
        let found: Vec<f64> = line
            .attribute("stroke-dasharray")
            .map(|written| {
                written
                    .split(' ')
                    .map(|part| part.parse().unwrap())
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(found, wanted, "{}", dash.name());
        assert_eq!(line.attribute("stroke-width"), Some("1.5"));
    }
    assert!(lines[0].attribute("stroke-dasharray").is_none());
    assert_eq!(lines[1].attribute("stroke-dasharray"), Some("6 4.5"));
    assert_eq!(
        lines[5].attribute("stroke-dasharray"),
        Some("6 3 1.5 3 1.5 3")
    );
    // A bent path, the outline of a rectangle and the outline of a polygon say theirs too.
    assert!(svg.contains(
        "stroke=\"#0072B2\" stroke-width=\"1\" stroke-dasharray=\"4 3\" stroke-opacity=\"0.5\""
    ));
    assert!(svg.contains("stroke=\"#222222\" stroke-width=\"0.5\" stroke-dasharray=\"2 1 0.5 1\""));
    assert!(svg.contains("stroke=\"#222222\" stroke-width=\"1\" stroke-dasharray=\"1 2\""));
    // The ends of a line are cut square and its corners drawn to a point, as a reader of SVG
    // does unless told otherwise: the file tells it nothing of either.
    assert!(!svg.contains("linecap") && !svg.contains("linejoin") && !svg.contains("dashoffset"));
    assert_eq!(write(&drawing), svg);
    insta::assert_snapshot!("line_types", svg);
    if let Ok(folder) = std::env::var("ZEFF_FIGURES_DIR") {
        std::fs::write(format!("{folder}/line_types.svg"), &svg).unwrap();
    }
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
