//! The fonts of a figure (Language Reference, section 7.6; release 0.7.0, step 21a): the three
//! embedded faces (Atkinson Hyperlegible Next and JetBrains Mono from release 0.10.0, IBM Plex
//! before it) as the engine's own reader gives them, against two independent
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
use zeff_plot::font::{Face, FaceName, FontError, Segment, Set, face, set};
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
    // The hashes of the project (BLAKE3) of the three files as they were fetched from their
    // publishers' repositories on 27 November 2026; `fonts/SOURCE.md` gives their SHA-256 and
    // their source.
    for (name, bytes, hash) in [
        (
            FaceName::Sans,
            65_068,
            "393feb612f4db2f9b37f265208cb74717ad1846008ae138abd56b4fbe53c7244",
        ),
        (
            FaceName::SansSemiBold,
            66_680,
            "ae2ad98bc281cdc586165fc436d1a303b17da6ad6ed5758bffc00cae1d16c420",
        ),
        (
            FaceName::Mono,
            273_900,
            "9dd51647d526d83d45d0784c817ade59d27773d41a3cd11059858d462389e2ec",
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
    // Each family's licence text beside its files, as its publisher gives it.
    for (file, holder) in [
        (
            "LICENSE-AtkinsonHyperlegibleNext.txt",
            "The Atkinson Hyperlegible Next Project Authors",
        ),
        (
            "LICENSE-JetBrainsMono.txt",
            "The JetBrains Mono Project Authors",
        ),
    ] {
        let licence = std::fs::read_to_string(folder.join(file)).unwrap();
        assert!(
            licence.contains("SIL OPEN FONT LICENSE Version 1.1"),
            "{file}"
        );
        assert!(licence.contains(holder), "{file}");
        assert!(source.contains(file), "{file}");
    }
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
        // The sans faces draw 362 characters and the mono face 1,317.
        assert!(drawn > 350, "{}: {drawn} characters", name.file());
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
            outlined > 350 && empty > 0,
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
                assert_eq!(lookups, [0, 1]);
                assert!(kerned > 10_000, "{}: {kerned} of {pairs}", name.file());
            }
        }
    }
    // Figures a reader can check by eye: A before V is drawn closer, and a full stop after T.
    let sans = face(FaceName::Sans);
    let glyph = |c: char| sans.glyph(c).unwrap();
    // HarfBuzz's 586 for A before V is its advance with their kerning (`shaping.json`).
    assert_eq!(
        i32::from(sans.advance(glyph('A'))) + i32::from(sans.kern(glyph('A'), glyph('V'))),
        586
    );
    assert!(sans.kern(glyph('A'), glyph('V')) < 0);
    assert!(sans.kern(glyph('T'), glyph('.')) < 0);
    assert_eq!(sans.kern(glyph('n'), glyph('n')), 0);
}

// ----- the reference: texts set by HarfBuzz -----

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/fonts/shaping.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// The whole numbers of a list of the reference.
fn numbers(list: &Json) -> Vec<i64> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_i64().unwrap())
        .collect()
}

/// Whether the independent reader finds a point for `mark` on `letter` in the tables that put
/// marks on letters.
fn their_point(theirs: &ttf_parser::Face<'_>, letter: u16, mark: u16) -> bool {
    let Some(gpos) = theirs.tables().gpos else {
        return false;
    };
    gpos.lookups.into_iter().any(|lookup| {
        (0..lookup.subtables.len()).any(|k| {
            let Some(PositioningSubtable::MarkToBase(on)) =
                lookup.subtables.get::<PositioningSubtable>(k)
            else {
                return false;
            };
            let (Some(row), Some(at)) = (
                on.base_coverage.get(GlyphId(letter)),
                on.mark_coverage.get(GlyphId(mark)),
            ) else {
                return false;
            };
            on.marks
                .get(at)
                .is_some_and(|(class, _)| on.anchors.get(row, class).is_some())
        })
    })
}

/// The one thing the engine does that HarfBuzz does not: a mark drawn above the small letters,
/// for which the font has no point on the letter before it, is raised by as much as that
/// letter rises above the small letters. Given the glyphs of a stretch, where the engine drew
/// them and where HarfBuzz does, this holds that the two agree along the baseline everywhere
/// and above it but for that rise, worked out from the independent reader's outlines and
/// tables; it gives the number of marks raised.
fn agrees_but_for_the_rise(
    name: FaceName,
    glyphs: &[u16],
    ours: &[(i64, i64)],
    harfbuzz: &[(i64, i64)],
    text: &str,
) -> usize {
    let (font, theirs) = (face(name), other(name));
    let small = i64::from(theirs.x_height().unwrap());
    let mut raised = 0;
    for i in 0..glyphs.len() {
        assert_eq!(ours[i].0, harfbuzz[i].0, "{} {text:?}", name.file());
        if !font.is_mark(glyphs[i]) {
            assert_eq!(ours[i].1, 0, "{} {text:?}", name.file());
            assert_eq!(harfbuzz[i].1, 0, "{} {text:?}", name.file());
            continue;
        }
        // The rise of the letter the mark belongs to; no letter, no rise.
        let letter = glyphs[..i].iter().rposition(|&g| !font.is_mark(g));
        let rise = letter
            .and_then(|at| theirs.glyph_bounding_box(GlyphId(glyphs[at])))
            .map_or(0, |bounds| (i64::from(bounds.y_max) - small).max(0));
        let above = ours[i].1 - harfbuzz[i].1;
        assert!(above == 0 || above == rise, "{} {text:?}", name.file());
        // The first mark after its letter is raised exactly when the font has no point for
        // the two and draws the mark above the small letters; a later mark stands on it, or
        // follows the same rule.
        if let Some(at) = letter.filter(|&at| at + 1 == i) {
            let high = theirs
                .glyph_bounding_box(GlyphId(glyphs[i]))
                .is_some_and(|bounds| i64::from(bounds.y_min) >= small);
            let wanted = if !their_point(&theirs, glyphs[at], glyphs[i]) && high {
                rise
            } else {
                0
            };
            assert_eq!(above, wanted, "{} {text:?}", name.file());
        }
        raised += usize::from(above > 0);
    }
    raised
}

/// Where HarfBuzz draws each glyph of a stretch: the place its advances give it, moved by
/// what HarfBuzz moves it by (a mark put on its letter), along the baseline and above it.
fn harfbuzz_places(case: &Json) -> Vec<(i64, i64)> {
    let (advances, dx, dy) = (
        numbers(&case["advances"]),
        numbers(&case["dx"]),
        numbers(&case["dy"]),
    );
    let mut pen = 0;
    let mut places = Vec::new();
    for i in 0..advances.len() {
        places.push((pen + dx[i], dy[i]));
        pen += advances[i];
    }
    places
}

#[test]
fn a_hundred_and_thirty_five_texts_are_set_as_harfbuzz_sets_them() {
    let reference = reference();
    let (mut texts, mut marks, mut raised) = (0, 0, 0);
    for name in FaceName::ALL {
        let ours = face(name);
        let cases = reference["faces"][name.file()].as_array().unwrap();
        assert_eq!(cases.len(), 135);
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
            // Each glyph stands where HarfBuzz draws it: where the advances before it put
            // it, and a mark where the font puts it on its letter.
            let places: Vec<(i64, i64)> = shaped
                .glyphs
                .iter()
                .map(|g| (i64::from(g.x), i64::from(g.y)))
                .collect();
            let drawn: Vec<u16> = shaped.glyphs.iter().map(|g| g.glyph).collect();
            raised += agrees_but_for_the_rise(name, &drawn, &places, &harfbuzz_places(case), text);
            marks += places.iter().filter(|place| place.1 != 0).count();
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
    assert_eq!(texts, 405);
    // Marks the fonts raise or lower on their letters, beyond those they only move along.
    assert!(marks > 20, "{marks}");
    // Marks raised over a tall letter the font has no point for: the hat of β and of θ in the
    // mono face, and nothing in the two sans faces that the list holds.
    assert!(raised >= 2, "{raised}");
    // The rule on ligatures, on record: HarfBuzz left to itself joins `fi` and `ffi` in the
    // Sans faces, and the text is then a few units narrower; the engine does not join them.
    let sans = &reference["faces"]["AtkinsonHyperlegibleNext-Regular.ttf"];
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
    // In the faces of release 0.10.0 no text of the list is set narrower by HarfBuzz left to
    // itself: Atkinson Hyperlegible Next joins no letters, and the joined signs of JetBrains
    // Mono (an arrow for `->`) are as wide as their parts. With IBM Plex, `fi` was narrower.
    assert_eq!(
        office["width_with_ligatures"].as_i64(),
        office["width"].as_i64()
    );
}

// ----- a line in two faces (release 0.10.0, step 4a; open decision #234) -----

/// Thirty-one lines, set in each sans face with the mono face behind it, against HarfBuzz
/// asked for each stretch in the face that draws it: the stretches, the face of each, its glyphs
/// and where each stands along the line and above it, and the width of the whole.
#[test]
fn a_line_in_two_faces_is_set_as_harfbuzz_sets_its_stretches() {
    let reference = reference();
    let mut in_two = 0;
    for first in [FaceName::Sans, FaceName::SansSemiBold] {
        let cases = reference["lines"][first.file()].as_array().unwrap();
        assert_eq!(cases.len(), 31);
        for case in cases {
            let text = case["text"].as_str().unwrap();
            let line = set(first, text);
            let runs: Vec<&[Set]> = line.runs().collect();
            let theirs = case["runs"].as_array().unwrap();
            assert_eq!(runs.len(), theirs.len(), "{} {text:?}", first.file());
            let mut along = 0_i64;
            for (run, their) in runs.iter().zip(theirs) {
                let by = run[0].face;
                assert_eq!(by.file(), their["face"].as_str().unwrap(), "{text:?}");
                assert!(run.iter().all(|placed| placed.face == by));
                let glyphs: Vec<u64> = run.iter().map(|placed| u64::from(placed.glyph)).collect();
                let wanted: Vec<u64> = their["glyphs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|g| g.as_u64().unwrap())
                    .collect();
                assert_eq!(glyphs, wanted, "{} {text:?}", first.file());
                // Each glyph stands where HarfBuzz draws it in its stretch, the stretch
                // starting where the one before it ended.
                let places: Vec<(i64, i64)> = run
                    .iter()
                    .map(|placed| (i64::from(placed.x) - along, i64::from(placed.y)))
                    .collect();
                let drawn: Vec<u16> = run.iter().map(|placed| placed.glyph).collect();
                agrees_but_for_the_rise(by, &drawn, &places, &harfbuzz_places(their), text);
                along += their["width"].as_i64().unwrap();
            }
            assert_eq!(i64::from(line.width), along, "{} {text:?}", first.file());
            assert_eq!(i64::from(line.width), case["width"].as_i64().unwrap());
            // A character is reported exactly when neither face draws it.
            assert_eq!(
                line.missing.is_empty(),
                !line.glyphs.iter().any(|placed| placed.glyph == 0),
                "{} {text:?}",
                first.file()
            );
            in_two += usize::from(runs.len() > 1);
        }
    }
    // Some of the lines do need two faces, whichever faces are embedded.
    assert!(in_two >= 4, "{in_two} lines in two faces");
    // Nothing stands behind the mono face: it sets a line alone, as `shape` does.
    assert_eq!(FaceName::Mono.fallback(), None);
    assert_eq!(FaceName::Sans.fallback(), Some(FaceName::Mono));
    assert_eq!(FaceName::SansSemiBold.fallback(), Some(FaceName::Mono));
    // The faces count the same units to the em, so that a place along a line of two faces is
    // one count of units.
    for name in FaceName::ALL {
        assert_eq!(face(name).units_per_em(), 1000, "{}", name.file());
    }
}

/// The rule, case by case: a character the first face has is drawn by it; one it lacks and the
/// mono face has is drawn by the mono face, at its own advance and with no kerning across the
/// join; one neither has is the first face's empty box, and is the only kind reported.
#[test]
fn a_character_is_taken_from_the_face_behind_only_where_the_first_lacks_it() {
    let (sans, mono) = (face(FaceName::Sans), face(FaceName::Mono));
    // A character of the mono face alone, whatever faces are embedded: found, not assumed.
    let only_mono: char = mono
        .characters()
        .find(|c| {
            // One the sans face cannot draw by parts either: a character that is not a letter
            // and its mark.
            let mut parts = 0;
            unicode_normalization::char::decompose_canonical(*c, |_| parts += 1);
            sans.glyph(*c).is_none()
                && !c.is_control()
                && !unicode_normalization::char::is_combining_mark(*c)
                && parts == 1
        })
        .expect("the mono face draws a character the sans face lacks");
    let neither = '\u{4e2d}';
    assert!(sans.glyph(neither).is_none() && mono.glyph(neither).is_none());
    let text = format!("ab{only_mono}{only_mono}c{neither}d");
    let line = set(FaceName::Sans, &text);
    let faces: Vec<FaceName> = line.glyphs.iter().map(|placed| placed.face).collect();
    assert_eq!(
        faces,
        [
            FaceName::Sans,
            FaceName::Sans,
            FaceName::Mono,
            FaceName::Mono,
            FaceName::Sans,
            FaceName::Sans,
            FaceName::Sans
        ]
    );
    assert_eq!(line.missing, [neither]);
    assert_eq!(line.glyphs[5].glyph, 0, "the empty box of the first face");
    assert_eq!(line.glyphs[2].glyph, mono.glyph(only_mono).unwrap());
    // The width is the widths of the stretches, each set by its own face alone.
    let stretch = |font: &Face, words: &str| font.shape(words).width;
    let pair: String = [only_mono, only_mono].iter().collect();
    let boxed = i32::from(sans.advance(0));
    assert_eq!(
        line.width,
        stretch(sans, "ab")
            + stretch(mono, &pair)
            + stretch(sans, "c")
            + boxed
            + stretch(sans, "d")
    );
    assert_eq!(line.runs().count(), 3);
    // In the mono face the same text has no face behind it: its own glyphs, and two boxes.
    let alone = set(FaceName::Mono, &format!("a{only_mono}{neither}"));
    assert!(
        alone
            .glyphs
            .iter()
            .all(|placed| placed.face == FaceName::Mono)
    );
    assert_eq!(alone.missing, [neither]);
    // An empty text is an empty line.
    let empty = set(FaceName::Sans, "");
    assert!(empty.glyphs.is_empty() && empty.width == 0 && empty.missing.is_empty());
    assert_eq!(empty.runs().count(), 0);
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
    // HarfBuzz sets A at 586 units before V, its advance less their kerning, and V at 593.
    assert_eq!(sans.shape("AV").width, 586 + 593);
    assert_eq!(sans.width("AV", 10.0), 11.79);
    assert_eq!(sans.width("AV", 8.0), 9.432);
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

// ----- a mark on its letter (release 0.10.0, step 4c) -----

/// The rule case by case, with the numbers of the embedded faces: where a mark stands on a
/// small letter, on a capital and on another mark; kerning across a mark; and which face
/// draws a letter with a mark.
#[test]
fn a_mark_is_put_on_its_letter_and_a_letter_and_its_marks_are_of_one_face() {
    let (sans, mono) = (face(FaceName::Sans), face(FaceName::Mono));
    let places = |font: &Face, text: &str| -> Vec<(i32, i32)> {
        font.shape(text).glyphs.iter().map(|g| (g.x, g.y)).collect()
    };
    // The bar of x̄: the x advances 481 and the font brings the bar back by 418, at the
    // height it is drawn at. Over a capital the bar is brought back by 489 and raised by 172.
    assert_eq!(places(sans, "x\u{304}"), [(0, 0), (63, 0)]);
    assert_eq!(places(sans, "X\u{304}"), [(0, 0), (134, 172)]);
    // A second mark stands on the first: the hat above the bar, 172 higher.
    assert_eq!(
        places(sans, "x\u{304}\u{302}"),
        [(0, 0), (63, 0), (39, 172)]
    );
    // A mark takes no room, and what follows stands where it would without the mark.
    assert_eq!(sans.shape("x\u{304}y").width, sans.shape("xy").width);
    assert_eq!(
        sans.shape("x\u{304}y").glyphs[2].x,
        sans.shape("xy").glyphs[1].x
    );
    // Kerning passes over a mark: T and o are set closer, with a bar on the T or without.
    let (t, o) = (sans.glyph('T').unwrap(), sans.glyph('o').unwrap());
    assert!(sans.kern(t, o) < 0);
    assert_eq!(sans.shape("T\u{304}o").width, sans.shape("To").width);
    assert_eq!(
        sans.shape("T\u{304}o").glyphs[2].x,
        i32::from(sans.advance(t)) + i32::from(sans.kern(t, o))
    );
    // A mark is a mark by the font's own classes, and advances nothing.
    let (bar, hat) = (
        sans.glyph('\u{304}').unwrap(),
        sans.glyph('\u{302}').unwrap(),
    );
    assert!(sans.is_mark(bar) && !sans.is_mark(t));
    assert_eq!(sans.advance(bar), 0);
    assert!(sans.places_marks(&[t, bar]) && sans.places_marks(&[t, bar, hat]));

    // Which face draws a letter and its marks, in a line of the sans face.
    let faces = |text: &str| -> Vec<FaceName> {
        set(FaceName::Sans, text)
            .glyphs
            .iter()
            .map(|g| g.face)
            .collect()
    };
    // The sans face has the letter and a point for the mark: it draws both.
    assert_eq!(faces("x\u{304}"), [FaceName::Sans; 2]);
    // The sans face has the mark and not the letter: the mono face draws both, so that the
    // mark stands on a letter of its own face.
    assert_eq!(sans.glyph('β'), None);
    assert_eq!(faces("β\u{302}"), [FaceName::Mono; 2]);
    // The sans face has both and no point for the one on the other: the mono face draws
    // both. A statistician's estimate of a mean is such a letter.
    let mu = sans.glyph('μ').unwrap();
    assert!(!sans.places_marks(&[mu, hat]));
    assert_eq!(faces("μ\u{302}"), [FaceName::Mono; 2]);
    assert_eq!(faces("μ"), [FaceName::Sans]);
    // The sans face has the letter and not the mark (a hook above): the mono face draws both.
    assert_eq!(sans.glyph('\u{309}'), None);
    assert_eq!(faces("x\u{309}"), [FaceName::Mono; 2]);
    assert_eq!(faces("x\u{304}\u{309}"), [FaceName::Mono; 3]);
    // A mark that opens a line has no letter in any face and stays in the face of the line;
    // so does a mark after a character neither face has.
    assert_eq!(faces("\u{304}x"), [FaceName::Sans; 2]);
    assert_eq!(faces("中\u{304}"), [FaceName::Sans; 2]);
    assert_eq!(set(FaceName::Sans, "中\u{304}").missing, ['中']);
    // A line of the mono face has no face behind it.
    assert!(
        set(FaceName::Mono, "μ\u{302}")
            .glyphs
            .iter()
            .all(|g| g.face == FaceName::Mono)
    );

    // In the mono face a mark is drawn over the letter before it as the font keeps it, with
    // no point needed, at the height that clears a small letter: μ̂ and σ̂ as they are.
    let line = set(FaceName::Sans, "μ\u{302}");
    assert_eq!((line.glyphs[1].x, line.glyphs[1].y), (600, 0));
    assert_eq!(set(FaceName::Sans, "σ\u{302}").glyphs[1].y, 0);
    // Over a tall letter the font has no point for, β or θ, the mark is raised by as much as
    // the letter rises above the small letters, 740 and 730 units against 550.
    assert_eq!(mono.x_height(), 550);
    assert_eq!(set(FaceName::Sans, "β\u{302}").glyphs[1].y, 190);
    assert_eq!(set(FaceName::Sans, "θ\u{302}").glyphs[1].y, 180);
    assert_eq!(set(FaceName::Sans, "λ\u{303}").glyphs[1].y, 180);
    // The font's own point raises a hat on b by the same 180, and a second mark stands on
    // the first, raised with it.
    assert_eq!(places(mono, "b\u{302}"), [(0, 0), (600, 180)]);
    let two = set(FaceName::Sans, "β\u{302}\u{304}");
    assert_eq!(two.glyphs[1].y, 190);
    assert!(two.glyphs[2].y > 190 + 100);
    // A mark below a tall letter is not raised.
    assert_eq!(places(mono, "β\u{323}")[1].1, 0);
    // A mark adds no width in either face, and the line is as wide as its letters.
    assert_eq!(
        set(FaceName::Sans, "β\u{302} x\u{304}").width,
        set(FaceName::Sans, "β x").width
    );

    // The stretches a PDF writes: one for each face at each height.
    let line = set(FaceName::Sans, "X\u{304} = β\u{302}");
    assert_eq!(line.runs().count(), 2);
    let stretches: Vec<(FaceName, i32, usize)> = line
        .stretches()
        .map(|run| (run[0].face, run[0].y, run.len()))
        .collect();
    assert_eq!(
        stretches,
        [
            (FaceName::Sans, 0, 1),
            (FaceName::Sans, 172, 1),
            (FaceName::Sans, 0, 3),
            (FaceName::Mono, 0, 1),
            (FaceName::Mono, 190, 1),
        ]
    );
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

/// A text of characters of either face, with a few neither has.
fn a_text_of_two_faces() -> impl Strategy<Value = String> {
    let sans: Vec<char> = face(FaceName::Sans).characters().collect();
    let mono: Vec<char> = face(FaceName::Mono).characters().collect();
    prop::collection::vec(
        prop_oneof![
            6 => prop::sample::select(sans),
            3 => prop::sample::select(mono),
            1 => prop::sample::select(vec!['中', '\t', '😀', '\u{E000}']),
        ],
        0..40,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(256)))]

    /// A line in two faces: a text its face sets without an empty box is set exactly as that
    /// face alone sets it, glyph for glyph and to the unit; any line is its stretches end to
    /// end, each kerned within itself as its own face kerns it and not across a join; and no
    /// glyph is taken from the face behind where the first face could draw the character.
    #[test]
    fn a_line_is_its_stretches_and_a_text_of_one_face_is_set_as_before(
        text in a_text_of_two_faces(),
    ) {
        for name in FaceName::ALL {
            let alone = face(name).shape(&text);
            let line = set(name, &text);
            let font = face(name);
            let marked = alone.glyphs.iter().any(|g| font.is_mark(g.glyph));
            // The first face draws the whole line when it lacks nothing and the text has no
            // mark; with marks, when it also has a point for each of them.
            if alone.missing.is_empty() && !marked {
                prop_assert!(line.glyphs.iter().all(|g| g.face == name));
            }
            if line.glyphs.iter().all(|g| g.face == name) {
                let same: Vec<(u16, i32, i32)> =
                    line.glyphs.iter().map(|g| (g.glyph, g.x, g.y)).collect();
                let before: Vec<(u16, i32, i32)> =
                    alone.glyphs.iter().map(|g| (g.glyph, g.x, g.y)).collect();
                prop_assert_eq!(same, before);
                prop_assert_eq!(line.width, alone.width);
                prop_assert_eq!(&line.missing, &alone.missing);
            }
            // Never fewer characters drawn: what was a box may be a letter, not the reverse.
            prop_assert!(line.missing.len() <= alone.missing.len());
            // The letters of the line, its marks set aside, stand one after the other, each
            // kerned with the letter before it when one face draws both and no other face
            // stands between them; a mark takes no room, is drawn on what stands before it,
            // and is the only glyph off the baseline.
            let mut along = 0_i32;
            let mut before: Option<&Set> = None;
            for (i, placed) in line.glyphs.iter().enumerate() {
                let by = face(placed.face);
                prop_assert!(placed.face == name || Some(placed.face) == name.fallback());
                if i > 0 && line.glyphs[i - 1].face != placed.face {
                    before = None;
                }
                if by.is_mark(placed.glyph) {
                    prop_assert_eq!(by.advance(placed.glyph), 0);
                    continue;
                }
                prop_assert_eq!(placed.y, 0);
                let kern = before.map_or(0, |left| i32::from(by.kern(left.glyph, placed.glyph)));
                prop_assert_eq!(placed.x, along + kern);
                along = placed.x + i32::from(by.advance(placed.glyph));
                before = Some(placed);
            }
            prop_assert_eq!(line.width, along);
            // Each stretch is set as its own face sets its glyphs, from where the stretch
            // before it ended.
            let mut start = 0_i32;
            for run in line.runs() {
                let glyphs: Vec<u16> = run.iter().map(|g| g.glyph).collect();
                let (places, width) = face(run[0].face).place(&glyphs);
                let got: Vec<(i32, i32)> = run.iter().map(|g| (g.x - start, g.y)).collect();
                prop_assert_eq!(got, places);
                start += width;
            }
            prop_assert_eq!(line.width, start);
            // The stretches cover the line, in order, each of one face and no two neighbours
            // of the same.
            let runs: Vec<&[Set]> = line.runs().collect();
            prop_assert_eq!(runs.iter().map(|run| run.len()).sum::<usize>(), line.glyphs.len());
            prop_assert!(runs.windows(2).all(|pair| pair[0][0].face != pair[1][0].face));
            // The empty box is drawn exactly where a character is reported.
            prop_assert_eq!(
                line.missing.is_empty(),
                !line.glyphs.iter().any(|g| g.glyph == 0)
            );
        }
    }
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
                // Each letter starts where the one before ends, moved by their kerning,
                // which passes over the marks between them; a mark takes no room.
                let mut width = 0_i32;
                let mut before: Option<u16> = None;
                for placed in &shaped.glyphs {
                    if font.is_mark(placed.glyph) {
                        prop_assert_eq!(font.advance(placed.glyph), 0);
                        continue;
                    }
                    prop_assert_eq!(placed.y, 0);
                    let kern = before.map_or(0, |left| i32::from(font.kern(left, placed.glyph)));
                    prop_assert_eq!(placed.x, width + kern);
                    width = placed.x + i32::from(font.advance(placed.glyph));
                    before = Some(placed.glyph);
                }
                prop_assert_eq!(shaped.width, width);
                // Taking the marks out moves no letter.
                let bare: Vec<u16> = shaped
                    .glyphs
                    .iter()
                    .map(|g| g.glyph)
                    .filter(|&glyph| !font.is_mark(glyph))
                    .collect();
                let (places, bare_width) = font.place(&bare);
                let letters: Vec<(i32, i32)> = shaped
                    .glyphs
                    .iter()
                    .filter(|g| !font.is_mark(g.glyph))
                    .map(|g| (g.x, g.y))
                    .collect();
                prop_assert_eq!(letters, places);
                prop_assert_eq!(shaped.width, bare_width);
                // A text of characters the face has is one glyph for each, the glyph of the
                // font's map.
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
                // The letters that meet, the marks between them passed over.
                let letter = |g: &&zeff_plot::font::Placed| !font.is_mark(g.glyph);
                let meet = match (a.glyphs.iter().rfind(letter), b.glyphs.iter().find(letter)) {
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
    let names = [
        "AtkinsonHyperlegibleNext-Regular",
        "AtkinsonHyperlegibleNext-SemiBold",
        "JetBrainsMono-Regular",
    ];
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
