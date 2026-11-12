//! The fonts of a figure (Language Reference, section 7.6; release 0.7.0, step 21a): the three
//! embedded faces of IBM Plex as the engine's own reader gives them, against two independent
//! readers. The reference: every character, advance, height, outline and kerning pair of each
//! face against the crate `ttf-parser`, which the tests alone use; and 110 texts set in each
//! face against HarfBuzz (`tests/reference/fonts/shaping.json`), glyph by glyph and to the last
//! unit of the font. The properties: a width is the sum of its advances and kerns, grows in
//! proportion to the size, and joins as its parts do; a damaged file is refused or read, never a
//! panic. The edge cases: an empty text, a character the face lacks, a mark written after its
//! letter, a joined letter drawn by its parts. And the files themselves, which must be the ones `fonts/SOURCE.md` records.

use proptest::prelude::*;
use serde_json::Value as Json;
use ttf_parser::gpos::{PairAdjustment, PositioningSubtable};
use ttf_parser::{GlyphId, OutlineBuilder, Tag};
use zeff_plot::font::{Face, FaceName, FontError, Segment, face};
use zeff_testkit::root;

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// The independent reader of a face.
fn other(name: FaceName) -> ttf_parser::Face<'static> {
    ttf_parser::Face::parse(name.bytes(), 0).unwrap()
}

// ----- the files -----

#[test]
fn the_embedded_files_are_the_ones_recorded() {
    // The hashes of the project (BLAKE3) of the three files as they were copied from IBM's
    // archives on 12 November 2026; `fonts/SOURCE.md` gives their SHA-256 and their source.
    for (name, bytes, hash) in [
        (
            FaceName::Sans,
            200_500,
            "a99deab680943960731178f9ef7f019d26c54b3faa1a4d3b5db4de662e6de543",
        ),
        (
            FaceName::SansSemiBold,
            202_632,
            "537f94beb9b5e160c00c30f661f8033d961ff8bad4fa2ee86ad32520b15ff259",
        ),
        (
            FaceName::Mono,
            173_052,
            "51ac7ac25d0fdec160d6d810dd2a58a6b2eaa241c89e1224befcce187a99b5f4",
        ),
    ] {
        assert_eq!(name.bytes().len(), bytes, "{}", name.file());
        assert_eq!(zeff_hash::hash_hex(name.bytes()), hash, "{}", name.file());
    }
    let folder = root().join("packages/tier1/plot/fonts");
    let source = std::fs::read_to_string(folder.join("SOURCE.md")).unwrap();
    for name in FaceName::ALL {
        assert!(source.contains(name.file()), "{}", name.file());
        // The file on disk is the file embedded.
        assert_eq!(
            std::fs::read(folder.join(name.file())).unwrap(),
            name.bytes()
        );
    }
    let licence = std::fs::read_to_string(folder.join("LICENSE.txt")).unwrap();
    assert!(licence.contains("SIL OPEN FONT LICENSE Version 1.1"));
    assert!(licence.contains("Reserved Font Name \"Plex\""));
}

// ----- the reference: an independent reader of the tables -----

#[test]
fn every_character_advance_and_height_agrees_with_an_independent_reader() {
    for name in FaceName::ALL {
        let (ours, theirs) = (face(name), other(name));
        assert_eq!(ours.units_per_em(), theirs.units_per_em());
        assert_eq!(
            ours.units_per_em(),
            1000,
            "the faces have 1000 units to the em"
        );
        assert_eq!(ours.glyphs(), theirs.number_of_glyphs());
        assert_eq!(ours.x_height(), theirs.x_height().unwrap());
        assert_eq!(ours.cap_height(), theirs.capital_height().unwrap());
        // The heights of the line, as the table `hhea` has them.
        let hhea = theirs.tables().hhea;
        assert_eq!(
            (ours.ascender(), ours.descender(), ours.line_gap()),
            (hhea.ascender, hhea.descender, hhea.line_gap)
        );
        // Every character of the basic plane and beyond: the same glyph, or none in both.
        let mut drawn = 0;
        for code in 0..=0x2_FFFF_u32 {
            let Some(character) = char::from_u32(code) else {
                continue;
            };
            let glyph = ours.glyph(character);
            assert_eq!(
                glyph,
                theirs
                    .glyph_index(character)
                    .map(|g| g.0)
                    .filter(|g| *g != 0),
                "{} U+{code:04X}",
                name.file()
            );
            drawn += usize::from(glyph.is_some());
        }
        assert_eq!(drawn, ours.characters().count());
        assert!(drawn > 700, "{}: {drawn} characters", name.file());
        for glyph in 0..ours.glyphs() {
            assert_eq!(
                Some(ours.advance(glyph)),
                theirs.glyph_hor_advance(GlyphId(glyph)),
                "{} glyph {glyph}",
                name.file()
            );
        }
        // A glyph the face does not have advances nothing.
        assert_eq!(ours.advance(ours.glyphs()), 0);
        assert_eq!(ours.advance(u16::MAX), 0);
    }
    // The Mono face has one advance for every glyph that advances at all.
    let mono = face(FaceName::Mono);
    assert!(
        mono.characters()
            .all(|c| matches!(mono.advance(mono.glyph(c).unwrap()), 0 | 600))
    );
}

/// The steps of an outline as the independent reader gives them.
#[derive(Default)]
struct Steps(Vec<Segment>);

impl OutlineBuilder for Steps {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(Segment::Move(f64::from(x), f64::from(y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(Segment::Line(f64::from(x), f64::from(y)));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push(Segment::Curve(
            (f64::from(x1), f64::from(y1)),
            (f64::from(x), f64::from(y)),
        ));
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        panic!("a TrueType outline has no cubic curve");
    }
    fn close(&mut self) {
        self.0.push(Segment::Close);
    }
}

#[test]
fn every_outline_agrees_with_an_independent_reader() {
    for name in FaceName::ALL {
        let (ours, theirs) = (face(name), other(name));
        let (mut outlined, mut empty) = (0, 0);
        for glyph in 0..ours.glyphs() {
            let mine = ours.outline(glyph).unwrap();
            let mut steps = Steps::default();
            let bounds = theirs.outline_glyph(GlyphId(glyph), &mut steps);
            if bounds.is_none() {
                assert!(mine.is_empty(), "{} glyph {glyph}", name.file());
                empty += 1;
                continue;
            }
            outlined += 1;
            // The other reader draws the last side of a contour as a line back to its start
            // before closing it; here the closing is that line.
            let mut theirs_steps: Vec<Segment> = Vec::with_capacity(steps.0.len());
            let mut started = (0.0, 0.0);
            for step in steps.0 {
                match step {
                    Segment::Move(x, y) => started = (x, y),
                    Segment::Close
                        if theirs_steps.last() == Some(&Segment::Line(started.0, started.1)) =>
                    {
                        theirs_steps.pop();
                    }
                    _ => {}
                }
                theirs_steps.push(step);
            }
            let steps = Steps(theirs_steps);
            assert_eq!(mine.len(), steps.0.len(), "{} glyph {glyph}", name.file());
            // The other reader works in single precision; a part of a glyph that is scaled
            // may differ by a small share of a unit.
            let near = |a: (f64, f64), b: (f64, f64)| {
                (a.0 - b.0).abs() <= 0.01 && (a.1 - b.1).abs() <= 0.01
            };
            for (i, (a, b)) in mine.iter().zip(&steps.0).enumerate() {
                let same = match (*a, *b) {
                    (Segment::Move(x, y), Segment::Move(u, v))
                    | (Segment::Line(x, y), Segment::Line(u, v)) => near((x, y), (u, v)),
                    (Segment::Curve(c, p), Segment::Curve(d, q)) => near(c, d) && near(p, q),
                    (Segment::Close, Segment::Close) => true,
                    _ => false,
                };
                assert!(
                    same,
                    "{} glyph {glyph}, step {i}: {a:?} against {b:?}",
                    name.file()
                );
            }
            // Every contour is closed, and none is left open at the end.
            assert_eq!(
                mine.first().map(|s| matches!(s, Segment::Move(..))),
                Some(true)
            );
            assert_eq!(mine.last(), Some(&Segment::Close));
            let moves = mine
                .iter()
                .filter(|s| matches!(s, Segment::Move(..)))
                .count();
            let closes = mine.iter().filter(|s| matches!(s, Segment::Close)).count();
            assert_eq!(moves, closes);
        }
        assert!(
            outlined > 900 && empty > 0,
            "{}: {outlined}, {empty}",
            name.file()
        );
        // A space draws nothing and still advances.
        let space = ours.glyph(' ').unwrap();
        assert!(ours.outline(space).unwrap().is_empty());
        assert!(ours.advance(space) > 0);
        // A glyph the face does not have has no outline to read.
        assert_eq!(ours.outline(ours.glyphs()), Err(FontError::Truncated));
    }
}

/// The kerning of a pair as the independent reader's tables give it: over the lookups of the
/// `kern` features, the first subtable of each that speaks of the pair.
fn their_kern(theirs: &ttf_parser::Face<'_>, lookups: &[u16], left: u16, right: u16) -> i32 {
    let Some(gpos) = theirs.tables().gpos else {
        return 0;
    };
    let mut total = 0;
    for &index in lookups {
        let lookup = gpos.lookups.get(index).unwrap();
        for k in 0..lookup.subtables.len() {
            let Some(PositioningSubtable::Pair(pairs)) =
                lookup.subtables.get::<PositioningSubtable>(k)
            else {
                continue;
            };
            let value = match pairs {
                PairAdjustment::Format1 { coverage, sets } => coverage
                    .get(GlyphId(left))
                    .and_then(|at| sets.get(at))
                    .and_then(|set| set.get(GlyphId(right))),
                PairAdjustment::Format2 {
                    coverage,
                    classes,
                    matrix,
                } => coverage.get(GlyphId(left)).and_then(|_| {
                    matrix.get((classes.0.get(GlyphId(left)), classes.1.get(GlyphId(right))))
                }),
            };
            if let Some((first, second)) = value {
                // The faces move the first glyph's advance and nothing else.
                assert_eq!(
                    (first.x_placement, first.y_placement, first.y_advance),
                    (0, 0, 0)
                );
                assert_eq!((second.x_placement, second.x_advance), (0, 0));
                total += i32::from(first.x_advance);
                break;
            }
        }
    }
    total
}

/// The lookups the `kern` features of a face name, each once, in order.
fn kern_lookups(theirs: &ttf_parser::Face<'_>) -> Vec<u16> {
    let Some(gpos) = theirs.tables().gpos else {
        return Vec::new();
    };
    let mut lookups: Vec<u16> = Vec::new();
    for feature in gpos.features {
        if feature.tag == Tag::from_bytes(b"kern") {
            for index in feature.lookup_indices {
                if !lookups.contains(&index) {
                    lookups.push(index);
                }
            }
        }
    }
    lookups.sort_unstable();
    lookups
}

#[test]
fn every_kerning_pair_agrees_with_an_independent_reader() {
    for name in FaceName::ALL {
        let (ours, theirs) = (face(name), other(name));
        let lookups = kern_lookups(&theirs);
        // Every pair of the glyphs a character can ask for.
        let mut glyphs: Vec<u16> = ours.characters().map(|c| ours.glyph(c).unwrap()).collect();
        glyphs.sort_unstable();
        glyphs.dedup();
        let (mut kerned, mut pairs) = (0_u32, 0_u32);
        for &left in &glyphs {
            for &right in &glyphs {
                let mine = i32::from(ours.kern(left, right));
                assert_eq!(
                    mine,
                    their_kern(&theirs, &lookups, left, right),
                    "{} glyphs {left} and {right}",
                    name.file()
                );
                kerned += u32::from(mine != 0);
                pairs += 1;
            }
        }
        match name {
            // A face of one width kerns nothing.
            FaceName::Mono => assert_eq!((lookups.len(), kerned), (0, 0)),
            _ => {
                assert_eq!(lookups, [0, 1, 2]);
                assert!(kerned > 10_000, "{}: {kerned} of {pairs}", name.file());
            }
        }
    }
    // Figures a reader can check by eye: A before V is drawn closer, and a full stop after T.
    let sans = face(FaceName::Sans);
    let glyph = |c: char| sans.glyph(c).unwrap();
    assert_eq!(sans.advance(glyph('A')), 641);
    assert_eq!(sans.kern(glyph('A'), glyph('V')), -41);
    assert!(sans.kern(glyph('T'), glyph('.')) < 0);
    assert_eq!(sans.kern(glyph('n'), glyph('n')), 0);
}

// ----- the reference: texts set by HarfBuzz -----

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/fonts/shaping.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn a_hundred_and_ten_texts_are_set_as_harfbuzz_sets_them() {
    let reference = reference();
    let mut texts = 0;
    for name in FaceName::ALL {
        let ours = face(name);
        let cases = reference["faces"][name.file()].as_array().unwrap();
        assert_eq!(cases.len(), 110);
        for case in cases {
            let text = case["text"].as_str().unwrap();
            let shaped = ours.shape(text);
            let glyphs: Vec<u64> = shaped.glyphs.iter().map(|g| u64::from(g.glyph)).collect();
            let wanted: Vec<u64> = case["glyphs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| g.as_u64().unwrap())
                .collect();
            assert_eq!(glyphs, wanted, "{} {text:?}", name.file());
            // The advance of a glyph is from its origin to the next one's.
            let advances: Vec<i64> = shaped
                .glyphs
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    i64::from(shaped.glyphs.get(i + 1).map_or(shaped.width, |next| next.x) - g.x)
                })
                .collect();
            let wanted: Vec<i64> = case["advances"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_i64().unwrap())
                .collect();
            assert_eq!(advances, wanted, "{} {text:?}", name.file());
            assert_eq!(
                i64::from(shaped.width),
                case["width"].as_i64().unwrap(),
                "{} {text:?}",
                name.file()
            );
            // A character is reported as lacking exactly when HarfBuzz drew the empty box.
            assert_eq!(
                shaped.missing.is_empty(),
                !glyphs.contains(&0),
                "{} {text:?}",
                name.file()
            );
            texts += 1;
        }
    }
    assert_eq!(texts, 330);
    // The rule on ligatures, on record: HarfBuzz left to itself joins `fi` and `ffi` in the
    // Sans faces, and the text is then a few units narrower; the engine does not join them.
    let sans = &reference["faces"]["IBMPlexSans-Regular.ttf"];
    let office = sans
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["text"] == "office")
        .unwrap();
    assert_eq!(
        face(FaceName::Sans).shape("office").width,
        office["width"].as_i64().unwrap() as i32
    );
    assert!(office["width_with_ligatures"].as_i64() < office["width"].as_i64());
}

// ----- the edge cases -----

#[test]
fn an_empty_text_a_character_the_face_lacks_and_a_mark_after_its_letter() {
    let sans = face(FaceName::Sans);
    // Nothing to set.
    let empty = sans.shape("");
    assert!(empty.glyphs.is_empty() && empty.missing.is_empty());
    assert_eq!(empty.width, 0);
    assert_eq!(sans.width("", 9.0), 0.0);
    // A character the face lacks is the face's empty box, with the box's advance, listed once.
    let shaped = sans.shape("a中b中😀");
    assert_eq!(
        shaped.glyphs.iter().map(|g| g.glyph).collect::<Vec<u16>>(),
        [sans.glyph('a').unwrap(), 0, sans.glyph('b').unwrap(), 0, 0]
    );
    assert_eq!(shaped.missing, ['中', '😀']);
    assert!(sans.advance(0) > 0);
    assert!(
        !sans.outline(0).unwrap().is_empty(),
        "the empty box is drawn"
    );
    assert_eq!(
        shaped.width,
        i32::from(sans.advance(sans.glyph('a').unwrap()))
            + i32::from(sans.advance(sans.glyph('b').unwrap()))
            + 3 * i32::from(sans.advance(0))
    );
    assert_eq!(sans.glyph('中'), None);
    // A tab and a line break are characters no face draws: a text is one line.
    assert_eq!(sans.shape("a\tb\n").missing, ['\t', '\n']);
    // A mark written after its letter is joined to it: the two are the one letter.
    assert_eq!(sans.shape("e\u{301}"), sans.shape("é"));
    assert_eq!(sans.shape("é").glyphs.len(), 1);
    assert_eq!(sans.shape("cafe\u{301}"), sans.shape("café"));
    // A joined letter the face lacks is drawn by its parts, which the face has, and is not
    // reported as lacking; written joined or in parts, it is set the same.
    assert_eq!(sans.glyph('ǧ'), None);
    let caron = sans.glyph('\u{30C}').unwrap();
    let parts = sans.shape("ǧ");
    assert_eq!(
        parts.glyphs.iter().map(|g| g.glyph).collect::<Vec<u16>>(),
        [sans.glyph('g').unwrap(), caron]
    );
    assert!(parts.missing.is_empty());
    assert_eq!(sans.shape("g\u{30C}"), parts);
    // A mark has its own glyph, which advances nothing, so the letter keeps its width.
    assert_eq!(sans.advance(caron), 0);
    assert_eq!(parts.width, sans.shape("g").width);
    assert_eq!(sans.shape("\u{301}").width, 0);
    // The widths in points: units times the size over a thousand.
    assert_eq!(sans.shape("AV").width, 641 - 41 + 609);
    assert_eq!(sans.width("AV", 10.0), 12.09);
    assert_eq!(sans.width("AV", 8.0), 9.672);
    // The SemiBold face is wider than the Regular, and the Mono face has one width.
    let (bold, mono) = (face(FaceName::SansSemiBold), face(FaceName::Mono));
    assert!(bold.shape("Hourly wage").width > sans.shape("Hourly wage").width);
    assert_eq!(mono.shape("Hourly wage").width, 11 * 600);
    assert_eq!(mono.shape("iiiii").width, mono.shape("WWWWW").width);
}

#[test]
fn what_is_not_a_font_of_this_kind_is_refused() {
    fn held(bytes: Vec<u8>) -> &'static [u8] {
        Box::leak(bytes.into_boxed_slice())
    }
    assert_eq!(Face::parse(held(Vec::new())), Err(FontError::Truncated));
    assert_eq!(Face::parse(held(vec![0, 1, 0])), Err(FontError::Truncated));
    assert_eq!(
        Face::parse(held(b"OTTO\0\0\0\0\0\0\0\0".to_vec())),
        Err(FontError::NotTrueType)
    );
    assert_eq!(
        Face::parse(held(
            b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>".to_vec()
        )),
        Err(FontError::NotTrueType)
    );
    // A file with the right first bytes and no table.
    assert_eq!(
        Face::parse(held(vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        Err(FontError::MissingTable("head"))
    );
    // A font cut short: the tables it names lie beyond its end.
    let whole = FaceName::Sans.bytes();
    assert_eq!(
        Face::parse(held(whole[..whole.len() / 2].to_vec())),
        Err(FontError::Truncated)
    );
    // The whole file reads, and reads the same twice.
    assert_eq!(Face::parse(whole).as_ref(), Ok(face(FaceName::Sans)));
}

// ----- the properties -----

/// A text of characters the Sans faces draw, with a few they lack.
fn a_text() -> impl Strategy<Value = String> {
    let drawn: Vec<char> = face(FaceName::Sans).characters().collect();
    prop::collection::vec(
        prop_oneof![
            8 => prop::sample::select(drawn),
            1 => prop::sample::select(vec!['中', '\t', '😀', '\u{E000}']),
        ],
        0..40,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(256)))]

    #[test]
    fn a_width_is_the_sum_of_its_advances_and_kerns_and_joins_as_its_parts_do(
        first in a_text(),
        second in a_text(),
        size in 1.0..200.0f64,
    ) {
        use unicode_normalization::UnicodeNormalization;
        for name in FaceName::ALL {
            let font = face(name);
            for text in [&first, &second] {
                let shaped = font.shape(text);
                // Each glyph starts where the one before ends, moved by their kerning.
                let mut width = 0_i32;
                for (i, placed) in shaped.glyphs.iter().enumerate() {
                    let kern = if i > 0 {
                        i32::from(font.kern(shaped.glyphs[i - 1].glyph, placed.glyph))
                    } else {
                        0
                    };
                    prop_assert_eq!(placed.x, width + kern);
                    width = placed.x + i32::from(font.advance(placed.glyph));
                }
                prop_assert_eq!(shaped.width, width);
                // A text of characters the face has, none of them a mark, is one glyph for
                // each, the glyph of the font's map.
                let composed: Vec<char> = text.nfc().collect();
                if composed.iter().all(|c| font.glyph(*c).is_some()) {
                    let wanted: Vec<u16> = composed.iter().map(|c| font.glyph(*c).unwrap()).collect();
                    let got: Vec<u16> = shaped.glyphs.iter().map(|g| g.glyph).collect();
                    prop_assert_eq!(got, wanted);
                    prop_assert!(shaped.missing.is_empty());
                }
                // The empty box is drawn exactly where a character is reported as lacking.
                prop_assert_eq!(
                    shaped.missing.is_empty(),
                    !shaped.glyphs.iter().any(|g| g.glyph == 0)
                );
                prop_assert!(shaped.missing.iter().all(|c| font.glyph(*c).is_none()));
                // The same text is set the same way twice, and so is its composed form.
                prop_assert_eq!(&font.shape(text), &shaped);
                prop_assert_eq!(&font.shape(&composed.iter().collect::<String>()), &shaped);
                // In points: in proportion to the size, exactly so for a doubling.
                let points = font.width(text, size);
                prop_assert_eq!(points.to_bits(), (f64::from(width) * size / 1000.0).to_bits());
                prop_assert_eq!(font.width(text, 2.0 * size).to_bits(), (2.0 * points).to_bits());
            }
            // Two texts joined are as wide as the two and the kerning where they meet, unless
            // a mark at the start of the second joins a letter at the end of the first.
            let joined_text = format!("{first}{second}");
            let apart: Vec<char> = first.nfc().chain(second.nfc()).collect();
            if joined_text.nfc().eq(apart.iter().copied()) {
                let (a, b) = (font.shape(&first), font.shape(&second));
                let meet = match (a.glyphs.last(), b.glyphs.first()) {
                    (Some(left), Some(right)) => i32::from(font.kern(left.glyph, right.glyph)),
                    _ => 0,
                };
                prop_assert_eq!(font.shape(&joined_text).width, a.width + b.width + meet);
            }
        }
    }

    #[test]
    fn a_damaged_file_is_refused_or_read_and_never_stops_the_engine(
        which in 0usize..3,
        cuts in prop::collection::vec((0.0..1.0f64, any::<u8>()), 1..12),
        keep in 0.0..1.0f64,
        shorten in any::<bool>(),
    ) {
        let name = FaceName::ALL[which];
        let mut bytes = name.bytes().to_vec();
        for (at, value) in cuts {
            let i = ((bytes.len() - 1) as f64 * at) as usize;
            bytes[i] = value;
        }
        if shorten {
            bytes.truncate((bytes.len() as f64 * keep) as usize);
        }
        let held: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        // Whatever the damage, the reader answers; and what it reads it can be asked about.
        if let Ok(font) = Face::parse(held) {
            let shaped = font.shape("Hourly wage, AV. 10.5 é");
            prop_assert!(shaped.glyphs.len() >= 23);
            for glyph in (0..font.glyphs()).step_by(7) {
                let _ = font.outline(glyph);
                let _ = font.kern(glyph, glyph / 2);
            }
        }
    }
}

#[test]
fn what_a_pdf_asks_of_a_font_is_read_as_the_second_reader_reads_it() {
    let names = ["IBMPlexSans", "IBMPlexSans-SmBld", "IBMPlexMono"];
    let weights = [400, 600, 400];
    for (i, name) in FaceName::ALL.into_iter().enumerate() {
        let font = face(name);
        let theirs = other(name);
        // The box that holds every glyph.
        let b = theirs.global_bounding_box();
        assert_eq!(font.bounds(), [b.x_min, b.y_min, b.x_max, b.y_max]);
        // The name the font goes by in a PDF file.
        assert_eq!(font.postscript_name(), names[i]);
        let named = theirs
            .names()
            .into_iter()
            .filter(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .find_map(|n| n.to_string());
        assert_eq!(named.as_deref(), Some(names[i]));
        // Its weight, its lean, and whether it is of one width.
        assert_eq!(font.weight(), weights[i]);
        assert_eq!(font.weight(), theirs.weight().to_number());
        assert_eq!(font.fixed_pitch(), theirs.is_monospaced());
        assert_eq!(font.fixed_pitch(), name == FaceName::Mono);
        assert_eq!(font.italic_angle(), 0.0);
        assert_eq!(font.italic_angle(), f64::from(theirs.italic_angle()));
        // The fonts set no limit on being carried in a document.
        assert!(font.may_be_embedded());
        assert_eq!(
            theirs.permissions(),
            Some(ttf_parser::Permissions::Installable)
        );
        // The file is the one read, whole.
        assert_eq!(font.file_bytes(), name.bytes());
        // A glyph is given back the first character that draws it.
        assert_eq!(font.character_of(font.glyph('A').unwrap()), Some('A'));
        assert_eq!(font.character_of(font.glyph(' ').unwrap()), Some(' '));
        for character in font.characters() {
            let glyph = font.glyph(character).unwrap();
            let first = font.character_of(glyph).unwrap();
            assert!(first <= character && font.glyph(first) == Some(glyph));
        }
    }
}
