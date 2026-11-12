//! The PDF of a figure (Language Reference, section 7.6; release 0.7.0, step 22b1; open
//! decision #118): the bytes a drawing is written as. Every primitive of the closed set,
//! written and kept as a snapshot of the file with its streams not compressed; a small figure
//! kept whole; the sample figures of the shipped wages at the three named sizes, each read
//! back by an independent reader of PDF and kept as its size and its hash; and what holds of
//! every file: it is one page as large as the drawing, the table at its end says where each
//! object stands, each face it uses is carried whole, its words are recovered through the map
//! from glyphs to characters and stand where the font sets them, it has one mark for each
//! mark, no type is below 8 points, it says nothing of the day or of the machine, it holds
//! nothing of the rows or of the name of the frame (F2), and the same drawing gives the same
//! bytes.

mod common;

use common::*;
use lopdf::{Dictionary, Document, Object, Stream};
use proptest::prelude::*;
use std::collections::BTreeMap;
use zeff_core::value::Aesthetic;
use zeff_plot::drawing::{Anchor, Drawing, Item, Primitive, Span};
use zeff_plot::font::{FaceName, face};
use zeff_plot::layout::{JOURNAL, SMALLEST_TEXT, Size, layout};
use zeff_plot::pdf::{DOTS, SHARED, rgb, write, write_plain};

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| at + from)
}

/// Where each object of a file starts and where the table at its end does, read as the file
/// says: the last `startxref`, the table it points to, and an object under every entry.
fn table(pdf: &[u8]) -> Result<(Vec<usize>, usize), String> {
    if !pdf.starts_with(b"%PDF-1.4\n%") {
        return Err("the file does not begin as a PDF".to_string());
    }
    if !pdf.ends_with(b"\n%%EOF\n") {
        return Err("the file does not end as a PDF".to_string());
    }
    let word = b"startxref\n";
    let mut last = None;
    let mut from = 0;
    while let Some(at) = find(pdf, word, from) {
        last = Some(at);
        from = at + 1;
    }
    let last = last.ok_or("no startxref")?;
    let number = |from: usize| -> Result<(usize, usize), String> {
        let end = (from..pdf.len())
            .find(|&i| !pdf[i].is_ascii_digit())
            .unwrap_or(pdf.len());
        std::str::from_utf8(&pdf[from..end])
            .ok()
            .and_then(|digits| digits.parse().ok())
            .map(|n| (n, end))
            .ok_or(format!("no number at byte {from}"))
    };
    let (start, _) = number(last + word.len())?;
    if !pdf[start..].starts_with(b"xref\n0 ") {
        return Err("startxref does not point to the table".to_string());
    }
    let (count, end) = number(start + 7)?;
    let mut offsets = Vec::new();
    for entry in 0..count {
        let at = end + 1 + 20 * entry;
        let line = pdf.get(at..at + 20).ok_or("the table is cut short")?;
        if entry == 0 {
            if line != b"0000000000 65535 f \n" {
                return Err("the first entry of the table is not the free one".to_string());
            }
            continue;
        }
        if &line[10..] != b" 00000 n \n" {
            return Err(format!("entry {entry} of the table is not well formed"));
        }
        let (offset, _) = number(at)?;
        if !pdf[offset..].starts_with(format!("{entry} 0 obj\n").as_bytes()) {
            return Err(format!("object {entry} is not where the table says"));
        }
        offsets.push(offset);
    }
    if !pdf[end + 1 + 20 * count..].starts_with(b"trailer\n") {
        return Err("no trailer after the table".to_string());
    }
    Ok((offsets, start))
}

/// A file whose streams are not compressed, as text: a stream that is not text is replaced by
/// its length and its hash.
fn readable(plain: &[u8]) -> String {
    let (offsets, end) = table(plain).unwrap();
    let mut out = String::from("%PDF-1.4\n");
    for (i, &start) in offsets.iter().enumerate() {
        let object = &plain[start..offsets.get(i + 1).copied().unwrap_or(end)];
        match find(object, b"stream\n", 0) {
            Some(at) => {
                let head = std::str::from_utf8(&object[..at + 7]).unwrap();
                let length: usize = head
                    .split("/Length ")
                    .last()
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|digits| digits.parse().ok())
                    .unwrap();
                let data = &object[at + 7..at + 7 + length];
                out.push_str(head);
                match std::str::from_utf8(data) {
                    Ok(text) if !text.chars().any(|c| c.is_control() && c != '\n') => {
                        out.push_str(text);
                    }
                    _ => out.push_str(&format!(
                        "({length} bytes that are not text, {})",
                        &zeff_hash::hash_hex(data)[..16]
                    )),
                }
                out.push_str(std::str::from_utf8(&object[at + 7 + length..]).unwrap());
            }
            None => out.push_str(std::str::from_utf8(object).unwrap()),
        }
    }
    out.push_str(std::str::from_utf8(&plain[end..]).unwrap());
    out
}

/// The last line of a snapshot of a file: the length and the hash of the file as `export`
/// writes it, its streams compressed.
fn kept(pdf: &[u8]) -> String {
    format!(
        "--- with its streams compressed: {} bytes, {}\n",
        pdf.len(),
        zeff_hash::hash_hex(pdf)
    )
}

/// The bytes of a stream, as they were before they were compressed.
fn bytes_of(stream: &Stream) -> Vec<u8> {
    if stream.dict.has(b"Filter") {
        stream.decompressed_content().unwrap()
    } else {
        stream.content.clone()
    }
}

/// What a name of a dictionary refers to, followed to the object.
fn under<'a>(document: &'a Document, dictionary: &'a Dictionary, key: &[u8]) -> &'a Object {
    let object = dictionary
        .get(key)
        .unwrap_or_else(|_| panic!("no /{}", String::from_utf8_lossy(key)));
    document.dereference(object).unwrap().1
}

fn number(object: &Object) -> f64 {
    match object {
        Object::Integer(n) => *n as f64,
        Object::Real(x) => f64::from(*x),
        other => panic!("{other:?} is not a number"),
    }
}

/// A face of a file as the independent reader finds it: the advance of each glyph the file
/// lists, in thousandths of the size of the type, and the characters of each glyph.
struct Carried {
    face: FaceName,
    advances: BTreeMap<u16, f64>,
    characters: BTreeMap<u16, String>,
}

/// A run of glyphs of a file: its face, the size of its type, and each glyph with how far
/// along the line it stands, in thousandths of the size of the type.
struct Run {
    face: FaceName,
    glyphs: Vec<(u16, f64)>,
}

/// What the independent reader finds in a file.
struct Read {
    /// The words of each text, through the map from glyphs to characters.
    words: Vec<String>,
    runs: Vec<Run>,
    /// The sizes of type the page sets.
    sizes: Vec<f64>,
    /// What is drawn inside a region that clips: the marks.
    marks: usize,
    /// The operators of the page, as text.
    page: String,
    /// The dots of each image: how many across and down, and three bytes each.
    images: Vec<(usize, usize, Vec<u8>)>,
    /// How many shapes are written once and placed.
    forms: usize,
    faces: Vec<FaceName>,
}

/// A file read back, and what must hold of it and its drawing: said as a sentence when it
/// does not.
fn read(pdf: &[u8], drawing: &Drawing) -> Result<Read, String> {
    table(pdf)?;
    let document = Document::load_mem(pdf).map_err(|why| format!("not a PDF: {why}"))?;
    // One page, as large as the drawing.
    let pages = document.get_pages();
    if pages.len() != 1 {
        return Err(format!("{} pages", pages.len()));
    }
    let page_id = pages[&1];
    let page = document
        .get_dictionary(page_id)
        .map_err(|e| e.to_string())?;
    let media: Vec<f64> = under(&document, page, b"MediaBox")
        .as_array()
        .map_err(|e| e.to_string())?
        .iter()
        .map(number)
        .collect();
    if media.len() != 4
        || media[0] != 0.0
        || media[1] != 0.0
        || (media[2] - drawing.width).abs() > 0.0006
        || (media[3] - drawing.height).abs() > 0.0006
    {
        return Err(format!(
            "the page is {media:?}, not as large as the drawing"
        ));
    }
    // The file says what wrote it and nothing of the day; its identifier is given twice.
    let about = under(&document, &document.trailer, b"Info")
        .as_dict()
        .map_err(|e| e.to_string())?;
    let said: Vec<String> = about
        .iter()
        .map(|(key, _)| String::from_utf8_lossy(key).into_owned())
        .collect();
    if said != ["Producer"] {
        return Err(format!("the file says of itself {said:?}"));
    }
    let id = document
        .trailer
        .get(b"ID")
        .and_then(Object::as_array)
        .map_err(|e| e.to_string())?;
    if id.len() != 2 || id[0] != id[1] || id[0].as_str().map_or(0, <[u8]>::len) != 16 {
        return Err("the identifier is not sixteen bytes given twice".to_string());
    }

    // Each face is carried whole, with the advances of its glyphs and their characters.
    let mut carried: BTreeMap<Vec<u8>, Carried> = BTreeMap::new();
    for (name, font) in document
        .get_page_fonts(page_id)
        .map_err(|e| e.to_string())?
    {
        let at: usize = String::from_utf8_lossy(&name)
            .strip_prefix('F')
            .and_then(|n| n.parse().ok())
            .filter(|n| (1..=3).contains(n))
            .ok_or("a font that is not F1, F2 or F3")?;
        let which = FaceName::ALL[at - 1];
        let embedded = face(which);
        if under(&document, font, b"Subtype").as_name().ok() != Some(&b"Type0"[..]) {
            return Err("a font that is not of glyphs by number".to_string());
        }
        let inner = under(&document, font, b"DescendantFonts")
            .as_array()
            .map_err(|e| e.to_string())?;
        let inner = document
            .dereference(&inner[0])
            .and_then(|(_, o)| o.as_dict())
            .map_err(|e| e.to_string())?;
        let described = under(&document, inner, b"FontDescriptor")
            .as_dict()
            .map_err(|e| e.to_string())?;
        let based = under(&document, described, b"FontName")
            .as_name()
            .map_err(|e| e.to_string())?;
        if based != embedded.postscript_name().as_bytes() {
            return Err(format!(
                "the font is called {}",
                String::from_utf8_lossy(based)
            ));
        }
        let file = under(&document, described, b"FontFile2")
            .as_stream()
            .map_err(|e| e.to_string())?;
        if bytes_of(file) != which.bytes() {
            return Err(format!("{} is not carried whole", which.file()));
        }
        if number(under(&document, &file.dict, b"Length1")) != which.bytes().len() as f64 {
            return Err("the length of the font is not the length of its file".to_string());
        }
        // The advances: a glyph, then the advances of it and of the glyphs after it.
        let listed = under(&document, inner, b"W")
            .as_array()
            .map_err(|e| e.to_string())?;
        let mut advances = BTreeMap::new();
        for pair in listed.chunks(2) {
            let first = number(&pair[0]) as u16;
            for (i, advance) in pair[1]
                .as_array()
                .map_err(|e| e.to_string())?
                .iter()
                .enumerate()
            {
                advances.insert(first + i as u16, number(advance));
            }
        }
        let per_em = 1000.0 / f64::from(embedded.units_per_em());
        for (&glyph, &advance) in &advances {
            if (advance - f64::from(embedded.advance(glyph)) * per_em).abs() > 0.0006 {
                return Err(format!("glyph {glyph} advances {advance} in the file"));
            }
        }
        // The map from glyphs to characters: `<glyph> <units of UTF-16>` on a line.
        let map = bytes_of(
            under(&document, font, b"ToUnicode")
                .as_stream()
                .map_err(|e| e.to_string())?,
        );
        let mut characters = BTreeMap::new();
        for line in String::from_utf8_lossy(&map).lines() {
            let Some((glyph, code)) = line.strip_prefix('<').and_then(|l| l.split_once("> <"))
            else {
                continue;
            };
            let Some(code) = code.strip_suffix('>') else {
                continue;
            };
            if glyph.len() != 4 || code == "FFFF" {
                continue;
            }
            let glyph = u16::from_str_radix(glyph, 16).map_err(|e| e.to_string())?;
            let units: Vec<u16> = (0..code.len() / 4)
                .map(|i| u16::from_str_radix(&code[4 * i..4 * i + 4], 16).unwrap())
                .collect();
            characters.insert(
                glyph,
                String::from_utf16(&units).map_err(|e| e.to_string())?,
            );
        }
        carried.insert(
            name,
            Carried {
                face: which,
                advances,
                characters,
            },
        );
    }

    // The shapes written once and the images.
    let mut images = Vec::new();
    let mut forms = 0;
    if let Ok(resources) = page.get(b"Resources").and_then(Object::as_dict)
        && let Ok(objects) = resources.get(b"XObject").and_then(Object::as_dict)
    {
        for (name, object) in objects.iter() {
            let stream = document
                .dereference(object)
                .and_then(|(_, o)| o.as_stream())
                .map_err(|e| e.to_string())?;
            if name.starts_with(b"M") {
                forms += 1;
            } else {
                let (across, down) = (
                    number(under(&document, &stream.dict, b"Width")) as usize,
                    number(under(&document, &stream.dict, b"Height")) as usize,
                );
                let dots = bytes_of(stream);
                if dots.len() != across * down * 3 {
                    return Err(format!(
                        "an image of {across} by {down} with {} bytes",
                        dots.len()
                    ));
                }
                images.push((across, down, dots));
            }
        }
    }

    // What the page draws.
    let content = document.get_page_content(page_id);
    let page_text = String::from_utf8_lossy(&content).into_owned();
    for bad in ["NaN", "inf", "\u{0}", "\u{7}"] {
        if page_text.contains(bad) {
            return Err(format!("the page holds {bad:?}"));
        }
    }
    let operations = lopdf::content::Content::decode(&content)
        .map_err(|why| format!("the page cannot be read: {why}"))?
        .operations;
    let (mut depth, mut clip) = (0_usize, None);
    let mut marks = 0;
    let mut words: Vec<String> = Vec::new();
    let mut runs: Vec<Run> = Vec::new();
    let mut sizes = Vec::new();
    let mut font: Option<&Carried> = None;
    for operation in &operations {
        match operation.operator.as_str() {
            "q" => depth += 1,
            "Q" => {
                depth = depth
                    .checked_sub(1)
                    .ok_or("a state restored that was not saved")?;
                if clip.is_some_and(|at| depth < at) {
                    clip = None;
                }
            }
            "W" => clip = Some(depth),
            "Do" => {
                let name = operation.operands[0].as_name().map_err(|e| e.to_string())?;
                if clip.is_some() && name.starts_with(b"M") {
                    marks += 1;
                }
            }
            "f" | "S" | "B" | "f*" | "B*" if clip.is_some() => marks += 1,
            "BT" => words.push(String::new()),
            "Tf" => {
                let name = operation.operands[0].as_name().map_err(|e| e.to_string())?;
                font = Some(carried.get(name).ok_or("type in a font the page has not")?);
                sizes.push(number(&operation.operands[1]));
            }
            "TJ" => {
                let font = font.ok_or("glyphs before a font")?;
                let mut along = 0.0;
                let mut glyphs = Vec::new();
                for part in operation.operands[0]
                    .as_array()
                    .map_err(|e| e.to_string())?
                {
                    match part {
                        Object::String(bytes, _) => {
                            for pair in bytes.chunks(2) {
                                let glyph = u16::from_be_bytes([pair[0], pair[1]]);
                                glyphs.push((glyph, along));
                                along += font
                                    .advances
                                    .get(&glyph)
                                    .ok_or(format!("glyph {glyph} has no advance in the file"))?;
                                if let Some(text) = font.characters.get(&glyph) {
                                    words
                                        .last_mut()
                                        .ok_or("glyphs outside a text")?
                                        .push_str(text);
                                }
                            }
                        }
                        closer => along -= number(closer),
                    }
                }
                runs.push(Run {
                    face: font.face,
                    glyphs,
                });
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("a state saved and not restored".to_string());
    }

    // The words of each text of the drawing, and each glyph where the font sets it.
    let mut expected_words = Vec::new();
    let mut expected_runs = Vec::new();
    for item in drawing.primitives() {
        let Primitive::Text { spans, .. } = &item.primitive else {
            continue;
        };
        if spans.iter().all(|span| span.text.is_empty()) {
            continue;
        }
        let mut text = String::new();
        for span in spans {
            let font = face(span.face);
            let shaped = font.shape(&span.text);
            if shaped.glyphs.is_empty() {
                continue;
            }
            let per_em = 1000.0 / f64::from(font.units_per_em());
            for placed in &shaped.glyphs {
                if let Some(character) = font.character_of(placed.glyph) {
                    text.push(character);
                }
            }
            expected_runs.push((
                span.face,
                shaped
                    .glyphs
                    .iter()
                    .map(|placed| (placed.glyph, f64::from(placed.x) * per_em))
                    .collect::<Vec<_>>(),
            ));
        }
        expected_words.push(text);
    }
    if words != expected_words {
        return Err(format!("the words are {words:?}, not {expected_words:?}"));
    }
    if runs.len() != expected_runs.len() {
        return Err(format!(
            "{} runs of glyphs, not {}",
            runs.len(),
            expected_runs.len()
        ));
    }
    for (run, (name, glyphs)) in runs.iter().zip(&expected_runs) {
        if run.face != *name || run.glyphs.len() != glyphs.len() {
            return Err("a run of glyphs in another face or of another length".to_string());
        }
        for ((glyph, along), (wanted, at)) in run.glyphs.iter().zip(glyphs) {
            if glyph != wanted || (along - at).abs() > 0.002 * (1.0 + glyphs.len() as f64) {
                return Err(format!(
                    "glyph {glyph} stands at {along}, not glyph {wanted} at {at}"
                ));
            }
        }
    }
    // One mark for each mark.
    if marks != common::marks(drawing) {
        return Err(format!(
            "{marks} marks written of {}",
            common::marks(drawing)
        ));
    }
    let faces = carried.values().map(|font| font.face).collect();
    Ok(Read {
        words,
        runs,
        sizes,
        marks,
        page: page_text,
        images,
        forms,
        faces,
    })
}

/// A drawing of items, 200 by 120.
fn of(items: Vec<Item>) -> Drawing {
    Drawing {
        width: 200.0,
        height: 120.0,
        items,
        notes: Vec::new(),
    }
}

fn unnamed(primitive: Primitive) -> Item {
    Item {
        name: None,
        primitive,
    }
}

// ----- every primitive -----

#[test]
fn every_primitive_of_the_closed_set_is_written() {
    let drawing = every_primitive();
    let pdf = write(&drawing);
    let plain = write_plain(&drawing);
    let found = read(&pdf, &drawing).unwrap();
    let text = readable(&plain);
    insta::assert_snapshot!("primitives", format!("{text}{}", kept(&pdf)));
    if let Ok(folder) = std::env::var("ZEFF_FIGURES_DIR") {
        std::fs::write(format!("{folder}/primitives.pdf"), &pdf).unwrap();
        std::fs::write(format!("{folder}/primitives_plain.pdf"), &plain).unwrap();
    }
    // The file with its streams compressed draws the same page.
    assert_eq!(read(&plain, &drawing).unwrap().page, found.page);
    assert!(pdf.len() < plain.len());
    // What the snapshot shows, said. The words are found, in the order they are drawn.
    assert_eq!(found.words, ["Wage & hours <2010>", "R² = 0.47", "up"]);
    assert_eq!(
        found.faces,
        [FaceName::Sans, FaceName::SansSemiBold, FaceName::Mono]
    );
    assert_eq!(found.sizes, [9.0, 10.0, 10.0, 8.0]);
    // Sixteen marks, each of a shape met once: none is written apart.
    assert_eq!((found.marks, found.forms), (16, 0));
    // What is not opaque is painted through a state of the page, which blends it.
    assert!(text.contains("/ExtGState << /G1 << /ca 0.5 /CA 0.5 >> /G2 << /ca 0.6 /CA 0.6 >> >>"));
    assert!(text.contains("/Group << /S /Transparency /CS /DeviceRGB >>"));
    assert!(found.page.contains("/G1 gs\n0.337 0.706 0.914 rg\n"));
    // An outline shape is stroked; a hole is cut by the even-odd rule; a region clips.
    assert!(found.page.contains("0.133 0.133 0.133 RG 0.9 w\n"));
    assert!(found.page.contains("h f*\n"));
    assert!(found.page.contains("q 5 5 190 50 re W n\n"));
    // A text read upwards is turned a quarter, exactly; ending at its place, it starts its
    // own length below it.
    assert!(
        found
            .page
            .contains(" Tf 0.133 0.133 0.133 rg 0 -1 -1 0 190 109.184 Tm ")
    );
    // The grid of cells is one image, each cell 42 dots by 42.
    assert_eq!(found.images.len(), 1);
    let (across, down, dots) = &found.images[0];
    assert_eq!((*across, *down), (3 * 42, 2 * 42));
    let dot = |column: usize, row: usize| {
        let at = (row * across + column) * 3;
        [dots[at], dots[at + 1], dots[at + 2]]
    };
    let cells = [
        "#440154", "#440154", "#21918C", "#FDE725", "#21918C", "#FDE725",
    ];
    for (i, cell) in cells.iter().enumerate() {
        let (column, row) = (i % 3, i / 3);
        for (dx, dy) in [(0, 0), (21, 21), (41, 41)] {
            assert_eq!(dot(column * 42 + dx, row * 42 + dy), rgb(cell));
        }
    }
    assert!(found.page.contains("q 30 0 0 -20 160 80 cm /I1 Do Q\n"));
    // A note is nowhere in the file, nor the name of an item.
    assert!(find(&plain, b"a note is not drawn", 0).is_none());
    for name in ["background", "axis:x", "layer:1", "legend:color"] {
        assert!(find(&plain, name.as_bytes(), 0).is_none(), "{name}");
    }
}

#[test]
fn a_shape_met_often_is_written_once_and_placed() {
    let point = |x: f64, shape: &'static str, size: f64, opacity: f64| {
        unnamed(Primitive::Point {
            at: (x, 30.0),
            shape,
            size,
            colour: "#0072B2".to_string(),
            opacity,
        })
    };
    assert_eq!(SHARED, 3);
    // Two of a shape are written where they stand.
    let two = of(vec![
        point(10.0, "square", 6.0, 1.0),
        point(20.0, "square", 6.0, 1.0),
    ]);
    let plain = String::from_utf8_lossy(&write_plain(&two)).into_owned();
    assert!(
        !plain.contains("/XObject") && !plain.contains(" Do"),
        "{plain}"
    );
    assert_eq!(plain.matches(" h f\n").count(), 2);
    // Three are written once and placed three times, in the paint set once.
    let three = of(vec![
        point(10.0, "square", 6.0, 0.4),
        point(20.0, "square", 6.0, 0.4),
        point(30.5, "square", 6.0, 0.4),
    ]);
    let plain = String::from_utf8_lossy(&write_plain(&three)).into_owned();
    assert!(
        plain.contains("/XObject << /M1 9 0 R >>") || plain.contains("/XObject << /M1 5 0 R >>")
    );
    assert!(plain.contains(
        "q\n/G1 gs\n0 0.447 0.698 rg\nq 1 0 0 1 10 30 cm /M1 Do Q\nq 1 0 0 1 20 30 cm /M1 Do Q\nq 1 0 0 1 30.5 30 cm /M1 Do Q\nQ\n"
    ), "{plain}");
    // The shape, about its own centre, and the box that holds it and its line.
    assert!(plain.contains(
        "/Type /XObject /Subtype /Form /BBox [-6.259 -6.259 6.259 6.259] /Resources << >> /Length"
    ), "{plain}");
    assert!(plain.contains(
        "stream\n-2.659 -2.659 m 2.659 -2.659 l 2.659 2.659 l -2.659 2.659 l h f\n\nendstream"
    ));
    // Another size is another shape; a shape of lines is stroked.
    let sizes = of(vec![
        point(10.0, "plus", 6.0, 1.0),
        point(20.0, "plus", 6.0, 1.0),
        point(30.0, "plus", 6.0, 1.0),
        point(40.0, "plus", 7.0, 1.0),
        point(50.0, "plus", 7.0, 1.0),
        point(60.0, "plus", 7.0, 1.0),
        point(70.0, "plus", 8.0, 1.0),
    ]);
    let plain = String::from_utf8_lossy(&write_plain(&sizes)).into_owned();
    assert_eq!(plain.matches("/Subtype /Form").count(), 2);
    assert_eq!(plain.matches(" Do Q\n").count(), 6);
    assert!(plain.contains("0 0.447 0.698 RG 0.9 w\n"));
    assert!(
        plain.contains("-3 0 m 3 0 l 0 -3 m 0 3 l S\n\nendstream"),
        "{plain}"
    );
    // The seventh, of a size met once, is written where it stands.
    assert!(
        plain.contains("66 30 m 74 30 l 70 26 m 70 34 l S\n"),
        "{plain}"
    );
    let found = read(&write(&sizes), &sizes).unwrap();
    assert_eq!(found.forms, 2);
}

#[test]
fn a_grid_of_cells_is_an_image_fine_enough_to_stay_sharp() {
    let grid = |width: f64, height: f64, columns: usize, rows: usize, cells: Vec<String>| {
        of(vec![unnamed(Primitive::Raster {
            at: (10.0, 20.0),
            width,
            height,
            columns,
            rows,
            cells,
        })])
    };
    let colours = |n: usize| -> Vec<String> {
        (0..n)
            .map(|i| format!("#{:02X}{:02X}{:02X}", i % 256, (i / 256) % 256, 128))
            .collect()
    };
    assert_eq!(DOTS, 300.0);
    // A cell of 0.24 points is one dot at 300 to the inch; a little wider, two.
    let fine = grid(2.4, 2.4, 10, 10, colours(100));
    assert_eq!(read(&write(&fine), &fine).unwrap().images[0].0, 10);
    let wider = grid(2.5, 2.4, 10, 10, colours(100));
    let found = read(&write(&wider), &wider).unwrap();
    assert_eq!((found.images[0].0, found.images[0].1), (20, 10));
    // A very large cell is repeated 64 times and no more.
    let large = grid(100.0, 50.0, 2, 1, colours(2));
    let found = read(&write(&large), &large).unwrap();
    assert_eq!((found.images[0].0, found.images[0].1), (128, 64));
    assert_eq!(&found.images[0].2[..3], &[0, 0, 128]);
    assert_eq!(&found.images[0].2[64 * 3..64 * 3 + 3], &[1, 0, 128]);
    // A grid of many cells is repeated less, so that the image stays within four million dots.
    let many = grid(500.0, 500.0, 1000, 1000, colours(1_000_000));
    let plain = write_plain(&many);
    assert!(find(&plain, b"/Width 2000 /Height 2000 ", 0).is_some());
    // Fewer colours than cells: the rows that are whole, then the cells of the next.
    let short = grid(30.0, 20.0, 3, 2, colours(5));
    let found = read(&write(&short), &short).unwrap();
    assert_eq!(found.images.len(), 2);
    assert_eq!((found.images[0].0, found.images[0].1), (3 * 42, 42));
    assert_eq!((found.images[1].0, found.images[1].1), (2 * 42, 42));
    assert!(
        found
            .page
            .contains("q 30 0 0 -10 10 30 cm /I1 Do Q\nq 20 0 0 -10 10 40 cm /I2 Do Q\n")
    );
    // No cells, no image.
    let none = grid(30.0, 20.0, 0, 2, Vec::new());
    assert!(read(&write(&none), &none).unwrap().images.is_empty());
    let empty = grid(30.0, 20.0, 3, 2, Vec::new());
    assert!(read(&write(&empty), &empty).unwrap().images.is_empty());
}

#[test]
fn text_is_set_glyph_by_glyph_with_the_kerning_of_the_font() {
    let line = |words: &str, name: FaceName, anchor: Anchor, turn: f64| {
        unnamed(Primitive::Text {
            at: (100.0, 60.0),
            anchor,
            turn,
            spans: vec![Span {
                text: words.to_string(),
                face: name,
                size: 10.0,
                colour: "#222222".to_string(),
            }],
            frame: None,
        })
    };
    let sans = face(FaceName::Sans);
    let (a, v) = (sans.glyph('A').unwrap(), sans.glyph('V').unwrap());
    let closer = -sans.kern(a, v);
    assert!(closer > 0, "the font sets A and V closer");
    let drawing = of(vec![line("AVA", FaceName::Sans, Anchor::Start, 0.0)]);
    let found = read(&write(&drawing), &drawing).unwrap();
    assert_eq!(found.words, ["AVA"]);
    assert!(
        found.page.contains(&format!(
            "/F1 10 Tf 0.133 0.133 0.133 rg 1 0 0 -1 100 60 Tm [<{a:04X}> {closer} <{v:04X}> {closer} <{a:04X}>] TJ\n"
        )),
        "{}",
        found.page
    );
    // The line is as long in the file as the font makes it.
    let last = found.runs[0].glyphs[2];
    assert_eq!(
        (last.1 + f64::from(sans.advance(a))) / 100.0,
        sans.width("AVA", 10.0)
    );
    // Centred and ending at its place, the text starts half its width or its width before.
    let width = sans.width("AVA", 10.0);
    for (anchor, start) in [
        (Anchor::Middle, 100.0 - width / 2.0),
        (Anchor::End, 100.0 - width),
    ] {
        let drawing = of(vec![line("AVA", FaceName::Sans, anchor, 0.0)]);
        let page = read(&write(&drawing), &drawing).unwrap().page;
        assert!(
            page.contains(&format!(
                " 1 0 0 -1 {} 60 Tm ",
                zeff_plot::drawing::length(start)
            )),
            "{page}"
        );
    }
    // A turn that is not a quarter is written to six decimals.
    let drawing = of(vec![line("AVA", FaceName::Sans, Anchor::Start, 30.0)]);
    let page = read(&write(&drawing), &drawing).unwrap().page;
    assert!(
        page.contains(" 0.866025 -0.5 -0.5 -0.866025 100 60 Tm "),
        "{page}"
    );
    // In the face of one width nothing is set closer: one string of glyphs.
    let drawing = of(vec![line("AVA 1.5", FaceName::Mono, Anchor::Start, 0.0)]);
    let found = read(&write(&drawing), &drawing).unwrap();
    assert_eq!(found.words, ["AVA 1.5"]);
    assert_eq!(found.page.matches('<').count(), 1, "{}", found.page);
    // A letter with its mark, written apart, is the one letter; a character no face has is
    // the empty box of the face and gives no word.
    let drawing = of(vec![line(
        "re\u{301}sume\u{301} \u{4e2d} β",
        FaceName::Sans,
        Anchor::Start,
        0.0,
    )]);
    let found = read(&write(&drawing), &drawing).unwrap();
    assert_eq!(found.words, ["résumé  β"]);
    assert_eq!(found.runs[0].glyphs[7].0, 0);
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
    let pdf = write(&drawing);
    let found = read(&pdf, &drawing).unwrap();
    insta::assert_snapshot!(
        "small_figure",
        format!("{}{}", readable(&write_plain(&drawing)), kept(&pdf))
    );
    // Six marks, three of each of two shapes: each shape written once.
    assert_eq!((found.marks, found.forms), (6, 2));
    assert!(
        found.words.contains(&"dose".to_string()),
        "{:?}",
        found.words
    );
    assert!(found.words.contains(&"response".to_string()));
    assert!(found.words.contains(&"placebo".to_string()));
    assert!(found.sizes.iter().all(|&size| size >= SMALLEST_TEXT));
    // The independent reader's own reading of the page finds the words too.
    let theirs = Document::load_mem(&pdf)
        .unwrap()
        .extract_text(&[1])
        .unwrap();
    for word in ["dose", "response", "placebo", "drug", "arm"] {
        assert!(theirs.contains(word), "{word} is not in {theirs:?}");
    }
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
            let pdf = write(&drawing);
            let found =
                read(&pdf, &drawing).unwrap_or_else(|why| panic!("{name} at {page}: {why}"));
            assert!(
                found.sizes.iter().all(|&size| size >= SMALLEST_TEXT),
                "{name} at {page}: {:?}",
                found.sizes
            );
            // The frame is called `wages`; the file does not say so, nor anything of a row.
            let plain = write_plain(&drawing);
            assert!(find(&plain, b"wages", 0).is_none(), "{name} at {page}");
            assert_eq!(write(&drawing), pdf);
            all.push_str(&format!(
                "{name} at {page}: {} bytes, {} marks, {} shapes written once, {}\n",
                pdf.len(),
                found.marks,
                found.forms,
                zeff_hash::hash_hex(&pdf)
            ));
            if let Some(folder) = &folder {
                std::fs::write(format!("{folder}/{name}_{page}.pdf"), &pdf).unwrap();
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
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let pdf = write(&drawing);
    let plain = write_plain(&drawing);
    assert!(find(&plain, b"patients_of_ward_7", 0).is_none() && find(&plain, b"ward", 0).is_none());
    // The same marks from other rows of a frame of another name: the same bytes.
    let mut other = drawn(small("d"), &mappings);
    for row in &mut other.panels[0].layers[0].rows {
        *row += 40_000;
    }
    assert_eq!(
        write(&layout(&other, Size::OneColumn, &JOURNAL).unwrap()),
        pdf
    );
    // The notes of the scene, which may count rows, are not in the file either.
    let mut noted = scene.clone();
    noted
        .notes
        .push("3 rows with NA in dose dropped".to_string());
    let drawing = layout(&noted, Size::OneColumn, &JOURNAL).unwrap();
    assert_eq!(drawing.notes, ["3 rows with NA in dose dropped"]);
    assert_eq!(write(&drawing), pdf);
    assert!(find(&plain, b"dropped", 0).is_none());
}

#[test]
fn words_are_found_as_they_are_written() {
    let mut scene = drawn(
        small("d"),
        &[(Aesthetic::X, "dose"), (Aesthetic::Y, "response")],
    );
    scene.scales[0].title = "Dose (mg) <a> & \\ \"more\"".to_string();
    scene.scales[1].title = "réponse β − γ".to_string();
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let found = read(&write(&drawing), &drawing).unwrap();
    // Brackets and a backslash, which a PDF reads as its own, are words like any other.
    assert!(
        found
            .words
            .contains(&"Dose (mg) <a> & \\ \"more\"".to_string()),
        "{:?}",
        found.words
    );
    assert!(
        found.words.contains(&"réponse β − γ".to_string()),
        "{:?}",
        found.words
    );
}

// ----- what holds of every file -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(120)))]

    #[test]
    fn every_file_is_read_back_has_its_marks_and_its_words_and_is_written_the_same_twice(
        rows in rows(),
        mappings in showing(),
        size in prop_oneof![
            Just(Size::OneColumn),
            Just(Size::TwoColumn),
            (70.0..300.0f64, 50.0..250.0f64).prop_map(|(w, h)| Size::Millimetres(w, h)),
        ],
        alpha in prop_oneof![Just(1.0), 0.05..1.0f64],
    ) {
        let scene = drawn_with(tried(&rows), &mappings, Vec::new(), alpha);
        if let Ok(drawing) = layout(&scene, size, &JOURNAL) {
            let pdf = write(&drawing);
            match read(&pdf, &drawing) {
                Ok(found) => {
                    prop_assert!(found.sizes.iter().all(|&size| size >= SMALLEST_TEXT));
                    prop_assert!(found.page.is_ascii());
                }
                Err(why) => prop_assert!(false, "{}", why),
            }
            prop_assert_eq!(&write(&drawing), &pdf);
        }
    }
}
