//! The fonts of a figure (Language Reference, section 7.6, "Rendering and export"; release
//! 0.7.0, step 21a): Atkinson Hyperlegible Next and JetBrains Mono, embedded in the engine and
//! never taken from the computer, so that a text has the same width and the same shape on every
//! machine. They took the place of IBM Plex Sans and Mono at release 0.10.0 (step 4b; open
//! decision #222), so that a figure is set in the faces of the website and of Zemp Lite.
//!
//! The files are in `fonts/`, unchanged, under the SIL Open Font License (`fonts/SOURCE.md`
//! gives their source and hashes). This module reads from them, with its own code, what a
//! figure needs and nothing more:
//!
//! - which glyph draws a character (`cmap`: the Unicode map of format 4, and for the
//!   characters beyond the basic plane the map of format 12 when the font has one);
//! - how far each glyph advances (`hmtx`) and the heights of the face (`hhea`, `OS/2`);
//! - the outline of each glyph (`glyf` and `loca`), simple or made of other glyphs;
//! - how much closer or farther two neighbours are set (`GPOS`, the pair adjustments of the
//!   `kern` feature);
//! - which glyphs are marks (`GDEF`) and where a mark is put on a letter or on another mark
//!   (`GPOS`, the anchors of the `mark` and `mkmk` features; release 0.10.0, step 4c).
//!
//! A text is set by one fixed rule. It is first brought to its composed form (Unicode's NFC),
//! so that a letter and a mark written after it are the one letter Unicode has for the two,
//! `é` for `e` and U+0301; a composed letter the face lacks is drawn by its parts when the
//! face has them all. Each character is then drawn by the glyph the map gives it, in order
//! from left to right, and the advance of a glyph is corrected by the kerning with the glyph
//! after it. A mark Unicode has no composed letter for, the bar of x̄ or the hat of β̂, is put
//! on the letter before it at the point the font keeps for the two, and a second mark on the
//! first; where the font keeps no point (JetBrains Mono has none for most Greek letters) the
//! mark stays where the font draws it, over the letter before it, and is raised over a tall
//! letter by as much as the letter rises above the small ones. No ligature is formed (`fi` is two glyphs), no glyph is put in the place of another
//! (the dot of an i stays under a mark), and nothing depends on a script or a language: a
//! feature is read wherever the font lists it. A character the face lacks is drawn by the
//! glyph the face keeps for that, an empty box, and is reported.
//!
//! A line of a figure is set in two faces where one is not enough ([`set`]; release 0.10.0,
//! step 4a; open decision #234): a character the face of the line lacks is taken, letter by
//! letter, from the face that stands behind it, the mono face for the two sans faces, and only
//! a character neither has is the empty box. A letter and the marks written after it are
//! drawn by one face, since a face has points for marks on its own letters alone. A text the
//! first face has whole is set exactly as [`Face::shape`] sets it.
//!
//! Every length is a whole number of font units, of which the faces have 1000 to the em: a
//! text of `w` units set at `s` points is `w × s / 1000` points wide.

use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::{compose, decompose_canonical, is_combining_mark};

/// The faces embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceName {
    /// Atkinson Hyperlegible Next Regular: the labels.
    Sans,
    /// Atkinson Hyperlegible Next SemiBold: the titles.
    SansSemiBold,
    /// JetBrains Mono Regular: code and aligned digits, and the characters the other two lack
    /// (the Greek alphabet but for four letters, arrows, Cyrillic).
    Mono,
}

impl FaceName {
    pub const ALL: [FaceName; 3] = [FaceName::Sans, FaceName::SansSemiBold, FaceName::Mono];

    /// The file of the face, as it is embedded.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            FaceName::Sans => include_bytes!("../fonts/AtkinsonHyperlegibleNext-Regular.ttf"),
            FaceName::SansSemiBold => {
                include_bytes!("../fonts/AtkinsonHyperlegibleNext-SemiBold.ttf")
            }
            FaceName::Mono => include_bytes!("../fonts/JetBrainsMono-Regular.ttf"),
        }
    }

    /// The name of the file in `fonts/`.
    pub fn file(self) -> &'static str {
        match self {
            FaceName::Sans => "AtkinsonHyperlegibleNext-Regular.ttf",
            FaceName::SansSemiBold => "AtkinsonHyperlegibleNext-SemiBold.ttf",
            FaceName::Mono => "JetBrainsMono-Regular.ttf",
        }
    }
}

impl FaceName {
    /// The face a character is taken from when this one lacks it: the mono face stands behind
    /// the two sans faces, and nothing stands behind the mono face.
    pub fn fallback(self) -> Option<FaceName> {
        match self {
            FaceName::Sans | FaceName::SansSemiBold => Some(FaceName::Mono),
            FaceName::Mono => None,
        }
    }
}

/// A glyph of a line, with the face that draws it and the place of its origin along the
/// baseline and above it, in font units. Every glyph stands on the baseline but a mark the
/// font puts higher or lower on its letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Set {
    pub face: FaceName,
    pub glyph: u16,
    pub x: i32,
    pub y: i32,
}

/// A line of a figure, set in its face and, where that face lacks a character, in the face
/// behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub glyphs: Vec<Set>,
    /// From the origin of the first glyph to the end of the advance of the last, in font units.
    pub width: i32,
    /// The characters neither face has, each once, in the order met.
    pub missing: Vec<char>,
}

impl Line {
    /// The glyphs in runs of one face each, in order: what is kerned within itself and has
    /// its marks put on its own letters.
    pub fn runs(&self) -> impl Iterator<Item = &[Set]> {
        self.glyphs.chunk_by(|a, b| a.face == b.face)
    }

    /// The glyphs in stretches of one face at one height each, in order: what a writer that
    /// selects a font and a baseline for a stretch of text, as a PDF does, writes one stretch
    /// at a time. A mark the font raises on its letter is a stretch of its own.
    pub fn stretches(&self) -> impl Iterator<Item = &[Set]> {
        self.glyphs.chunk_by(|a, b| a.face == b.face && a.y == b.y)
    }
}

/// The glyphs `face` draws `character` by: its own glyph, or, for a composed letter it lacks,
/// the glyphs of its parts when it has them all. The parts are the fewest the face can draw
/// the letter by: of a letter with two marks, the letter the face has with the first mark, and
/// the second mark, before the bare letter and both marks.
fn glyphs_of(face: &Face, character: char) -> Option<Vec<u16>> {
    if let Some(glyph) = face.glyph(character) {
        return Some(vec![glyph]);
    }
    let mut parts: Vec<char> = Vec::new();
    decompose_canonical(character, |part| parts.push(part));
    // The first `kept` parts joined again into one letter, the rest each by its own glyph.
    (1..parts.len()).rev().find_map(|kept| {
        let mut joined = parts[0];
        for &part in &parts[1..kept] {
            joined = compose(joined, part)?;
        }
        let mut glyphs = vec![face.glyph(joined)?];
        for &part in &parts[kept..] {
            glyphs.push(face.glyph(part)?);
        }
        Some(glyphs)
    })
}

/// `text` set on one line in the face `name` and, letter by letter, in the face behind it
/// where `name` lacks a character. The text is brought to its composed form and taken a
/// letter at a time, a letter being a character with the marks written after it. The letter is
/// drawn by `name` when that face has all of it, each character whole or by its parts, and
/// has a point for each of its marks; otherwise by the fallback face when that one has all of
/// it, so that a mark stands on a letter of its own face, the only letters a face knows the
/// shapes of; otherwise by `name` after all, when it has all of it, the marks left where
/// their advances put them. A letter neither face has whole is drawn character by character,
/// each by the first face that has it, and a character neither has is the empty box of
/// `name`, and is listed.
///
/// A stretch of one face is then set as that face sets it ([`Face::place`]): kerned within
/// itself and not across a change of face, its marks put on its own letters. The faces have
/// the same number of units to the em, so that a place along the line is one count of units
/// whatever the face.
pub fn set(name: FaceName, text: &str) -> Line {
    let first = face(name);
    let behind = name.fallback().map(|other| (other, face(other)));
    let characters: Vec<char> = text.nfc().collect();
    let mut drawn: Vec<(FaceName, u16)> = Vec::with_capacity(characters.len());
    let mut missing: Vec<char> = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        let marks = characters[at + 1..]
            .iter()
            .take_while(|&&character| is_combining_mark(character))
            .count();
        let letter = &characters[at..=at + marks];
        at += letter.len();
        // The whole letter by one face, where one face has it.
        let whole = |by: &Face| -> Option<Vec<u16>> {
            let mut glyphs = Vec::new();
            for &character in letter {
                glyphs.extend(glyphs_of(by, character)?);
            }
            Some(glyphs)
        };
        let by_first = whole(first);
        // A mark that opens a text has no letter to stand on in any face.
        let lone = is_combining_mark(letter[0]);
        if let Some(glyphs) = by_first
            .as_ref()
            .filter(|glyphs| lone || first.places_marks(glyphs))
        {
            drawn.extend(glyphs.iter().map(|&glyph| (name, glyph)));
            continue;
        }
        if let Some((other, glyphs)) =
            behind.and_then(|(other, second)| Some((other, whole(second)?)))
        {
            drawn.extend(glyphs.into_iter().map(|glyph| (other, glyph)));
            continue;
        }
        if let Some(glyphs) = by_first {
            drawn.extend(glyphs.into_iter().map(|glyph| (name, glyph)));
            continue;
        }
        for &character in letter {
            if let Some(glyphs) = glyphs_of(first, character) {
                drawn.extend(glyphs.into_iter().map(|glyph| (name, glyph)));
            } else if let Some((other, glyphs)) =
                behind.and_then(|(other, second)| Some((other, glyphs_of(second, character)?)))
            {
                drawn.extend(glyphs.into_iter().map(|glyph| (other, glyph)));
            } else {
                if !missing.contains(&character) {
                    missing.push(character);
                }
                drawn.push((name, 0));
            }
        }
    }
    let mut glyphs: Vec<Set> = Vec::with_capacity(drawn.len());
    let mut along = 0_i32;
    for run in drawn.chunk_by(|a, b| a.0 == b.0) {
        let by = run[0].0;
        let of: Vec<u16> = run.iter().map(|&(_, glyph)| glyph).collect();
        let (places, width) = face(by).place(&of);
        glyphs.extend(of.iter().zip(places).map(|(&glyph, (x, y))| Set {
            face: by,
            glyph,
            x: along + x,
            y,
        }));
        along += width;
    }
    Line {
        glyphs,
        width: along,
        missing,
    }
}

/// The face, read once.
pub fn face(name: FaceName) -> &'static Face {
    static FACES: [OnceLock<Face>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let at = match name {
        FaceName::Sans => 0,
        FaceName::SansSemiBold => 1,
        FaceName::Mono => 2,
    };
    FACES[at].get_or_init(|| {
        Face::parse(name.bytes()).unwrap_or_else(|error| {
            panic!(
                "the embedded font {} cannot be read: {error:?}",
                name.file()
            )
        })
    })
}

/// Why a font file cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontError {
    /// The file ends before what a table says it holds.
    Truncated,
    /// Not a TrueType file with outlines in `glyf`.
    NotTrueType,
    /// A table this reader needs is absent.
    MissingTable(&'static str),
    /// A form of a table this reader does not read.
    Unsupported(&'static str),
}

/// Bytes read at an offset, each read checked against the end of the data.
#[derive(Clone, Copy)]
struct Bytes<'a>(&'a [u8]);

impl<'a> Bytes<'a> {
    fn u8(self, at: usize) -> Result<u8, FontError> {
        self.0.get(at).copied().ok_or(FontError::Truncated)
    }

    fn u16(self, at: usize) -> Result<u16, FontError> {
        let end = at.checked_add(2).ok_or(FontError::Truncated)?;
        let bytes = self.0.get(at..end).ok_or(FontError::Truncated)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn i16(self, at: usize) -> Result<i16, FontError> {
        Ok(self.u16(at)? as i16)
    }

    fn u32(self, at: usize) -> Result<u32, FontError> {
        let end = at.checked_add(4).ok_or(FontError::Truncated)?;
        let bytes = self.0.get(at..end).ok_or(FontError::Truncated)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// The bytes from `at` on.
    fn from(self, at: usize) -> Result<Bytes<'a>, FontError> {
        self.0.get(at..).map(Bytes).ok_or(FontError::Truncated)
    }

    /// `length` bytes from `at`.
    fn part(self, at: usize, length: usize) -> Result<Bytes<'a>, FontError> {
        let end = at.checked_add(length).ok_or(FontError::Truncated)?;
        self.0.get(at..end).map(Bytes).ok_or(FontError::Truncated)
    }
}

/// One step of an outline, in font units, the y axis upwards. A curve is quadratic, as the
/// font holds it: `Curve(control, to)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    Move(f64, f64),
    Line(f64, f64),
    Curve((f64, f64), (f64, f64)),
    Close,
}

/// A glyph of a text, with the place of its origin along the baseline and above it, in font
/// units. Every glyph stands on the baseline but a mark the font puts higher or lower on its
/// letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub glyph: u16,
    pub x: i32,
    pub y: i32,
}

/// A text set on one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shaped {
    pub glyphs: Vec<Placed>,
    /// From the origin of the first glyph to the end of the advance of the last, in font units.
    pub width: i32,
    /// The characters the face lacks, each once, in the order met.
    pub missing: Vec<char>,
}

/// The glyphs a subtable of pair adjustments starts from: ranges of glyphs, each with the
/// index of its first.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Coverage(Vec<(u16, u16, u16)>);

impl Coverage {
    fn read(table: Bytes<'_>) -> Result<Coverage, FontError> {
        let mut ranges = Vec::new();
        match table.u16(0)? {
            1 => {
                for i in 0..table.u16(2)? {
                    let glyph = table.u16(4 + 2 * usize::from(i))?;
                    ranges.push((glyph, glyph, i));
                }
            }
            2 => {
                for i in 0..usize::from(table.u16(2)?) {
                    let at = 4 + 6 * i;
                    ranges.push((table.u16(at)?, table.u16(at + 2)?, table.u16(at + 4)?));
                }
            }
            _ => return Err(FontError::Unsupported("a coverage table of another format")),
        }
        Ok(Coverage(ranges))
    }

    /// The index of `glyph` among the glyphs covered.
    fn index(&self, glyph: u16) -> Option<usize> {
        let at = self.0.partition_point(|&(_, end, _)| end < glyph);
        let &(start, _, first) = self.0.get(at)?;
        (start <= glyph).then(|| usize::from(first) + usize::from(glyph - start))
    }
}

/// The class of each glyph, 0 for a glyph in no range.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Classes(Vec<(u16, u16, u16)>);

impl Classes {
    fn read(table: Bytes<'_>) -> Result<Classes, FontError> {
        let mut ranges = Vec::new();
        match table.u16(0)? {
            1 => {
                let start = table.u16(2)?;
                for i in 0..table.u16(4)? {
                    let glyph = start.checked_add(i).ok_or(FontError::Truncated)?;
                    ranges.push((glyph, glyph, table.u16(6 + 2 * usize::from(i))?));
                }
            }
            2 => {
                for i in 0..usize::from(table.u16(2)?) {
                    let at = 4 + 6 * i;
                    ranges.push((table.u16(at)?, table.u16(at + 2)?, table.u16(at + 4)?));
                }
            }
            _ => return Err(FontError::Unsupported("a class table of another format")),
        }
        Ok(Classes(ranges))
    }

    fn of(&self, glyph: u16) -> usize {
        let at = self.0.partition_point(|&(_, end, _)| end < glyph);
        match self.0.get(at) {
            Some(&(start, _, class)) if start <= glyph => usize::from(class),
            _ => 0,
        }
    }
}

/// The classes a font gives its glyphs (`GDEF`): a letter, a ligature, a mark.
const LETTER: usize = 1;
const LIGATURE: usize = 2;
const MARK: usize = 3;

/// The glyphs a lookup passes over as if they were not in the line: the bits of its flag, and
/// the set of marks it looks at alone when it names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Passing {
    flag: u16,
    set: usize,
}

impl Passing {
    const LETTERS: u16 = 0x0002;
    const LIGATURES: u16 = 0x0004;
    const MARKS: u16 = 0x0008;
    const MARKS_OUTSIDE_A_SET: u16 = 0x0010;
    const MARKS_BY_ATTACHMENT: u16 = 0xFF00;

    /// The same lookup looking back for the mark another mark stands on: it passes over no
    /// class of glyph, and still over the marks outside its set.
    fn back(self) -> Passing {
        Passing {
            flag: self.flag & !(Passing::LETTERS | Passing::LIGATURES | Passing::MARKS),
            set: self.set,
        }
    }
}

/// One subtable that puts marks on glyphs: for each mark covered, its class and the point of
/// the mark that is brought onto the glyph; for each glyph covered, the point each class of
/// mark is brought to, where the glyph has one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Anchors {
    marks: Coverage,
    mark: Vec<(usize, (i16, i16))>,
    onto: Coverage,
    classes: usize,
    points: Vec<Option<(i16, i16)>>,
}

impl Anchors {
    fn read(table: Bytes<'static>) -> Result<Anchors, FontError> {
        if table.u16(0)? != 1 {
            return Err(FontError::Unsupported("mark placing of another format"));
        }
        let point = |from: Bytes<'static>, at: usize| -> Result<(i16, i16), FontError> {
            let anchor = from.from(at)?;
            // The three formats start alike; the rest (a point of the outline, or changes for
            // a size in dots) is for a rasteriser that fits outlines to dots, which this is not.
            match anchor.u16(0)? {
                1..=3 => Ok((anchor.i16(2)?, anchor.i16(4)?)),
                _ => Err(FontError::Unsupported("an anchor of another format")),
            }
        };
        let marks = Coverage::read(table.from(usize::from(table.u16(2)?))?)?;
        let onto = Coverage::read(table.from(usize::from(table.u16(4)?))?)?;
        let classes = usize::from(table.u16(6)?);
        let listed = table.from(usize::from(table.u16(8)?))?;
        let mut mark = Vec::new();
        for i in 0..usize::from(listed.u16(0)?) {
            let at = 2 + 4 * i;
            let class = usize::from(listed.u16(at)?);
            if class >= classes {
                return Err(FontError::Unsupported("a mark of a class the table lacks"));
            }
            mark.push((class, point(listed, usize::from(listed.u16(at + 2)?))?));
        }
        let rows = table.from(usize::from(table.u16(10)?))?;
        let mut points = Vec::new();
        for cell in 0..usize::from(rows.u16(0)?) * classes {
            points.push(match usize::from(rows.u16(2 + 2 * cell)?) {
                0 => None,
                at => Some(point(rows, at)?),
            });
        }
        Ok(Anchors {
            marks,
            mark,
            onto,
            classes,
            points,
        })
    }

    /// How far the origin of `mark` stands from the origin of `glyph` when this subtable puts
    /// the one on the other, or `None` when it says nothing of the two (it covers one of them
    /// alone, or the glyph has no point for a mark of that class) and the next is asked.
    fn offset(&self, mark: u16, glyph: u16) -> Option<(i32, i32)> {
        let &(class, from) = self.mark.get(self.marks.index(mark)?)?;
        let to = (*self
            .points
            .get(self.onto.index(glyph)? * self.classes + class)?)?;
        Some((
            i32::from(to.0) - i32::from(from.0),
            i32::from(to.1) - i32::from(from.1),
        ))
    }
}

/// A lookup that puts marks on the letters before them (the `mark` feature) or on the marks
/// before them (`mkmk`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Placing {
    on_marks: bool,
    passing: Passing,
    subtables: Vec<Anchors>,
}

/// One subtable of pair adjustments: the change to the advance of the first glyph of a pair.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pairs {
    /// Pair by pair: for each first glyph covered, its seconds in order with their values.
    Listed {
        coverage: Coverage,
        sets: Vec<Vec<(u16, i16)>>,
    },
    /// By classes of the first and of the second glyph.
    Classed {
        coverage: Coverage,
        first: Classes,
        second: Classes,
        columns: usize,
        values: Vec<i16>,
    },
}

impl Pairs {
    /// The adjustment of the pair, or `None` when the subtable says nothing of it and the
    /// next one is asked: a listed subtable speaks of the pairs it lists, a classed one of
    /// every pair whose first glyph it covers, as the shaping engines read them.
    fn adjust(&self, left: u16, right: u16) -> Option<i16> {
        match self {
            Pairs::Listed { coverage, sets } => {
                let set = sets.get(coverage.index(left)?)?;
                let at = set
                    .binary_search_by_key(&right, |&(second, _)| second)
                    .ok()?;
                Some(set[at].1)
            }
            Pairs::Classed {
                coverage,
                first,
                second,
                columns,
                values,
            } => {
                coverage.index(left)?;
                let (row, column) = (first.of(left), second.of(right));
                if column >= *columns {
                    return None;
                }
                values.get(row * columns + column).copied()
            }
        }
    }
}

/// The size in bytes of a value record of `format`, and where its change of advance is, if
/// it has one.
fn value_layout(format: u16) -> (usize, Option<usize>) {
    let size = 2 * (format & 0x00FF).count_ones() as usize;
    let advance = (format & 0x0004 != 0).then(|| 2 * (format & 0x0003).count_ones() as usize);
    (size, advance)
}

/// A font, read.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    data: &'static [u8],
    units_per_em: u16,
    ascender: i16,
    descender: i16,
    line_gap: i16,
    cap_height: i16,
    x_height: i16,
    /// The box that holds every glyph: left, bottom, right, top.
    bounds: [i16; 4],
    /// The name the font goes by in a PDF or PostScript file.
    postscript_name: String,
    /// How heavy the face is, from 1 to 1000: 400 for a regular face, 600 for a semibold one.
    weight: u16,
    /// Whether every glyph advances as far as every other.
    fixed_pitch: bool,
    /// How far the upright strokes lean, in degrees anticlockwise: 0 for an upright face.
    italic_angle: f64,
    /// What the maker of the font allows of a document that carries it (`fsType`).
    embedding: u16,
    /// The advance of each glyph.
    advances: Vec<u16>,
    /// The characters the face draws, in order, each with its glyph.
    map: Vec<(u32, u16)>,
    /// Where each glyph's outline starts in `glyf`, and after the last where it ends.
    offsets: Vec<u32>,
    glyf: (usize, usize),
    /// The lookups of the `kern` feature, each with the glyphs it passes over and its
    /// subtables.
    kerning: Vec<(Passing, Vec<Pairs>)>,
    /// The class the font gives each glyph: 1 a letter, 2 a ligature, 3 a mark.
    classes: Classes,
    /// The sets of marks a lookup may name, to look at those marks and no others.
    mark_sets: Vec<Coverage>,
    /// The lookups of the `mark` and `mkmk` features, in the order of the font.
    placing: Vec<Placing>,
}

impl Face {
    /// The font in `data`.
    pub fn parse(data: &'static [u8]) -> Result<Face, FontError> {
        let file = Bytes(data);
        if file.u32(0)? != 0x0001_0000 {
            return Err(FontError::NotTrueType);
        }
        let count = usize::from(file.u16(4)?);
        let table = |tag: &'static str| -> Result<Option<(usize, usize)>, FontError> {
            for i in 0..count {
                let at = 12 + 16 * i;
                if file.part(at, 4)?.0 == tag.as_bytes() {
                    let (offset, length) =
                        (file.u32(at + 8)? as usize, file.u32(at + 12)? as usize);
                    file.part(offset, length)?;
                    return Ok(Some((offset, length)));
                }
            }
            Ok(None)
        };
        let need = |tag: &'static str| -> Result<Bytes<'static>, FontError> {
            let (offset, length) = table(tag)?.ok_or(FontError::MissingTable(tag))?;
            file.part(offset, length)
        };
        let head = need("head")?;
        let units_per_em = head.u16(18)?;
        if units_per_em == 0 {
            return Err(FontError::Unsupported("a font with no units to the em"));
        }
        let bounds = [head.i16(36)?, head.i16(38)?, head.i16(40)?, head.i16(42)?];
        let long_offsets = match head.i16(50)? {
            0 => false,
            1 => true,
            _ => {
                return Err(FontError::Unsupported(
                    "an index of outlines of another format",
                ));
            }
        };
        let glyphs = usize::from(need("maxp")?.u16(4)?);
        let hhea = need("hhea")?;
        let (ascender, descender, line_gap) = (hhea.i16(4)?, hhea.i16(6)?, hhea.i16(8)?);
        let measured = usize::from(hhea.u16(34)?);
        if measured == 0 || measured > glyphs {
            return Err(FontError::Unsupported(
                "a table of advances that fits no glyph",
            ));
        }
        // The advances: one for each of the first glyphs, the last of them for the rest.
        let hmtx = need("hmtx")?;
        let mut advances = Vec::with_capacity(glyphs);
        for glyph in 0..glyphs {
            advances.push(hmtx.u16(4 * glyph.min(measured - 1))?);
        }
        // The heights of capitals and of small letters, from version 2 of the table on.
        let os2 = need("OS/2")?;
        let (x_height, cap_height) = if os2.u16(0)? >= 2 {
            (os2.i16(86)?, os2.i16(88)?)
        } else {
            (0, 0)
        };
        let (weight, embedding) = (os2.u16(4)?, os2.u16(8)?);
        // The lean of the face and whether it is of one width, where the font says.
        let (italic_angle, fixed_pitch) = match table("post")? {
            Some((offset, length)) => {
                let post = file.part(offset, length)?;
                (f64::from(post.u32(4)? as i32) / 65536.0, post.u32(12)? != 0)
            }
            None => (0.0, false),
        };
        let map = Face::read_map(need("cmap")?, glyphs)?;
        let postscript_name = match table("name")? {
            Some((offset, length)) => Face::read_postscript_name(file.part(offset, length)?)?,
            None => String::new(),
        };
        // Where the outlines are.
        let loca = need("loca")?;
        let glyf = table("glyf")?.ok_or(FontError::MissingTable("glyf"))?;
        let mut offsets = Vec::with_capacity(glyphs + 1);
        for glyph in 0..=glyphs {
            let offset = if long_offsets {
                loca.u32(4 * glyph)?
            } else {
                2 * u32::from(loca.u16(2 * glyph)?)
            };
            if offset as usize > glyf.1 || offsets.last().is_some_and(|&last| offset < last) {
                return Err(FontError::Truncated);
            }
            offsets.push(offset);
        }
        let (classes, mark_sets) = match table("GDEF")? {
            Some((offset, length)) => Face::read_classes(file.part(offset, length)?)?,
            None => (Classes(Vec::new()), Vec::new()),
        };
        let (kerning, placing) = match table("GPOS")? {
            Some((offset, length)) => {
                Face::read_positioning(file.part(offset, length)?, mark_sets.len())?
            }
            None => (Vec::new(), Vec::new()),
        };
        // A mark takes no room of its own: the placing below counts on it.
        if (0..glyphs).any(|glyph| classes.of(glyph as u16) == MARK && advances[glyph] != 0) {
            return Err(FontError::Unsupported("a mark that advances"));
        }
        Ok(Face {
            data,
            units_per_em,
            ascender,
            descender,
            line_gap,
            cap_height,
            x_height,
            bounds,
            postscript_name,
            weight,
            fixed_pitch,
            italic_angle,
            embedding,
            advances,
            map,
            offsets,
            glyf,
            kerning,
            classes,
            mark_sets,
            placing,
        })
    }

    /// The PostScript name of the font (name 6 of the `name` table), from the Windows record in
    /// UTF-16 or the Macintosh one in one byte to a letter; only the characters such a name may
    /// hold are kept. Empty when the table has none.
    fn read_postscript_name(name: Bytes<'static>) -> Result<String, FontError> {
        let strings = usize::from(name.u16(4)?);
        let mut found: Option<String> = None;
        for i in 0..usize::from(name.u16(2)?) {
            let at = 6 + 12 * i;
            if name.u16(at + 6)? != 6 {
                continue;
            }
            let platform = name.u16(at)?;
            let text = name.part(
                strings + usize::from(name.u16(at + 10)?),
                usize::from(name.u16(at + 8)?),
            )?;
            let read: String = match platform {
                3 | 0 => text
                    .0
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .filter_map(|pair| char::from_u32(u32::from(u16::from_be_bytes(*pair))))
                    .collect(),
                1 => text.0.iter().map(|&b| char::from(b)).collect(),
                _ => continue,
            };
            let kept: String = read
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !kept.is_empty() {
                found = Some(kept);
                if platform == 3 {
                    break;
                }
            }
        }
        Ok(found.unwrap_or_default())
    }

    /// The characters of the Unicode map of format 4, each with its glyph.
    fn read_map(cmap: Bytes<'static>, glyphs: usize) -> Result<Vec<(u32, u16)>, FontError> {
        let mut found = None;
        // The map of the whole of Unicode, of format 12, when the font has one: it is read for
        // the characters beyond the basic plane alone (release 0.10.0, step 4b: the mono face
        // has the double-struck letters there).
        let mut beyond = None;
        for i in 0..usize::from(cmap.u16(2)?) {
            let at = 4 + 8 * i;
            let (platform, encoding) = (cmap.u16(at)?, cmap.u16(at + 2)?);
            let subtable = cmap.from(cmap.u32(at + 4)? as usize)?;
            // Unicode, of the basic plane: platform 3 with encoding 1, or platform 0.
            let unicode = (platform == 3 && encoding == 1) || platform == 0;
            if unicode && subtable.u16(0)? == 4 && !matches!(found, Some((3, _))) {
                found = Some((platform, subtable));
            }
            // Unicode, in full: platform 3 with encoding 10, or platform 0.
            let full = (platform == 3 && encoding == 10) || platform == 0;
            if full && subtable.u16(0)? == 12 && !matches!(beyond, Some((3, _))) {
                beyond = Some((platform, subtable));
            }
        }
        let found = found.map(|(_, subtable)| subtable);
        let table = found.ok_or(FontError::Unsupported(
            "a font with no Unicode map of format 4",
        ))?;
        let segments = usize::from(table.u16(6)? / 2);
        let (ends, starts) = (14, 16 + 2 * segments);
        let (deltas, ranges) = (starts + 2 * segments, starts + 4 * segments);
        let mut map = Vec::new();
        for segment in 0..segments {
            let end = table.u16(ends + 2 * segment)?;
            let start = table.u16(starts + 2 * segment)?;
            let delta = table.u16(deltas + 2 * segment)?;
            let range_at = ranges + 2 * segment;
            let range = usize::from(table.u16(range_at)?);
            if start > end {
                return Err(FontError::Unsupported("a map whose segment runs backwards"));
            }
            for code in start..=end {
                if code == 0xFFFF {
                    break;
                }
                let glyph = if range == 0 {
                    code.wrapping_add(delta)
                } else {
                    let listed = table.u16(range_at + range + 2 * usize::from(code - start))?;
                    if listed == 0 {
                        0
                    } else {
                        listed.wrapping_add(delta)
                    }
                };
                if glyph != 0 && usize::from(glyph) < glyphs {
                    map.push((u32::from(code), glyph));
                }
            }
        }
        if let Some((_, table)) = beyond {
            // Groups of neighbouring characters drawn by neighbouring glyphs: first, last,
            // glyph of the first. They stand in order and apart, which also bounds the work a
            // damaged file can ask for.
            let groups = table.u32(12)? as usize;
            let mut last: Option<u32> = None;
            for group in 0..groups {
                let at = 16 + 12 * group;
                let (start, end, first) = (table.u32(at)?, table.u32(at + 4)?, table.u32(at + 8)?);
                if start > end || end > 0x10_FFFF || last.is_some_and(|before| start <= before) {
                    return Err(FontError::Unsupported(
                        "a map whose groups are not in order",
                    ));
                }
                last = Some(end);
                for code in start.max(0x1_0000)..=end {
                    // A sum a damaged file may push past what a number holds.
                    let glyph = first
                        .checked_add(code - start)
                        .ok_or(FontError::Truncated)?;
                    if glyph != 0 && (glyph as usize) < glyphs {
                        map.push((code, glyph as u16));
                    }
                }
            }
        }
        if !map.windows(2).all(|pair| pair[0].0 < pair[1].0) {
            return Err(FontError::Unsupported(
                "a map whose segments are not in order",
            ));
        }
        Ok(map)
    }

    /// The classes of the glyphs and the sets of marks (`GDEF`).
    fn read_classes(gdef: Bytes<'static>) -> Result<(Classes, Vec<Coverage>), FontError> {
        let classes = match usize::from(gdef.u16(4)?) {
            0 => Classes(Vec::new()),
            at => Classes::read(gdef.from(at)?)?,
        };
        let mut sets = Vec::new();
        // From version 1.2 of the table on, the sets of marks a lookup may name.
        if (gdef.u16(0)?, gdef.u16(2)?) >= (1, 2) {
            let at = usize::from(gdef.u16(12)?);
            if at != 0 {
                let listed = gdef.from(at)?;
                if listed.u16(0)? != 1 {
                    return Err(FontError::Unsupported("sets of marks of another format"));
                }
                for i in 0..usize::from(listed.u16(2)?) {
                    sets.push(Coverage::read(
                        listed.from(listed.u32(4 + 4 * i)? as usize)?,
                    )?);
                }
            }
        }
        Ok((classes, sets))
    }

    /// The lookups of `GPOS` this reader applies: those of the `kern` feature that adjust
    /// pairs, and those of `mark` and `mkmk` that put marks on glyphs, each kind in the order
    /// of the font. A feature is read whatever script or language the font lists it under.
    #[allow(clippy::type_complexity)]
    fn read_positioning(
        gpos: Bytes<'static>,
        sets: usize,
    ) -> Result<(Vec<(Passing, Vec<Pairs>)>, Vec<Placing>), FontError> {
        let features = gpos.from(usize::from(gpos.u16(6)?))?;
        // Each lookup named, with the features that name it: kerning, marks on letters, marks
        // on marks.
        let mut named: Vec<(u16, [bool; 3])> = Vec::new();
        for i in 0..usize::from(features.u16(0)?) {
            let at = 2 + 6 * i;
            let which = match features.part(at, 4)?.0 {
                b"kern" => 0,
                b"mark" => 1,
                b"mkmk" => 2,
                _ => continue,
            };
            let feature = features.from(usize::from(features.u16(at + 4)?))?;
            for k in 0..usize::from(feature.u16(2)?) {
                let lookup = feature.u16(4 + 2 * k)?;
                match named.iter_mut().find(|(index, _)| *index == lookup) {
                    Some((_, by)) => by[which] = true,
                    None => {
                        let mut by = [false; 3];
                        by[which] = true;
                        named.push((lookup, by));
                    }
                }
            }
        }
        named.sort_unstable_by_key(|&(index, _)| index);
        let lookups = gpos.from(usize::from(gpos.u16(8)?))?;
        let (mut kerning, mut placing) = (Vec::new(), Vec::new());
        for (index, by) in named {
            if index >= lookups.u16(0)? {
                return Err(FontError::Truncated);
            }
            let lookup = lookups.from(usize::from(lookups.u16(2 + 2 * usize::from(index))?))?;
            let (kind, flag, count) = (lookup.u16(0)?, lookup.u16(2)?, usize::from(lookup.u16(4)?));
            if flag & Passing::MARKS_BY_ATTACHMENT != 0 {
                return Err(FontError::Unsupported(
                    "a lookup that tells marks apart by their class of attachment",
                ));
            }
            let set = if flag & Passing::MARKS_OUTSIDE_A_SET != 0 {
                let set = usize::from(lookup.u16(6 + 2 * count)?);
                if set >= sets {
                    return Err(FontError::Unsupported("a set of marks the font lacks"));
                }
                set
            } else {
                0
            };
            let passing = Passing { flag, set };
            let (mut pairs, mut on_letters, mut on_marks) = (Vec::new(), Vec::new(), Vec::new());
            for k in 0..count {
                let mut subtable = lookup.from(usize::from(lookup.u16(6 + 2 * k)?))?;
                let mut of = kind;
                // An extension holds a subtable of another kind farther away.
                if of == 9 {
                    of = subtable.u16(2)?;
                    subtable = subtable.from(subtable.u32(4)? as usize)?;
                }
                match of {
                    2 if by[0] => pairs.push(Face::read_pairs(subtable)?),
                    4 if by[1] => on_letters.push(Anchors::read(subtable)?),
                    6 if by[2] => on_marks.push(Anchors::read(subtable)?),
                    _ => {}
                }
            }
            if by[0] {
                kerning.push((passing, pairs));
            }
            if !on_letters.is_empty() {
                placing.push(Placing {
                    on_marks: false,
                    passing,
                    subtables: on_letters,
                });
            }
            if !on_marks.is_empty() {
                placing.push(Placing {
                    on_marks: true,
                    passing,
                    subtables: on_marks,
                });
            }
        }
        Ok((kerning, placing))
    }

    fn read_pairs(table: Bytes<'static>) -> Result<Pairs, FontError> {
        let coverage = Coverage::read(table.from(usize::from(table.u16(2)?))?)?;
        let (first, second) = (value_layout(table.u16(4)?), value_layout(table.u16(6)?));
        let advance = |record: Bytes<'_>| -> Result<i16, FontError> {
            match first.1 {
                Some(at) => record.i16(at),
                None => Ok(0),
            }
        };
        match table.u16(0)? {
            1 => {
                let record = 2 + first.0 + second.0;
                let mut sets = Vec::new();
                for i in 0..usize::from(table.u16(8)?) {
                    let set = table.from(usize::from(table.u16(10 + 2 * i)?))?;
                    let mut pairs = Vec::new();
                    for k in 0..usize::from(set.u16(0)?) {
                        let at = 2 + record * k;
                        pairs.push((set.u16(at)?, advance(set.from(at + 2)?)?));
                    }
                    if !pairs.windows(2).all(|pair| pair[0].0 < pair[1].0) {
                        return Err(FontError::Unsupported("pairs that are not in order"));
                    }
                    sets.push(pairs);
                }
                Ok(Pairs::Listed { coverage, sets })
            }
            2 => {
                let firsts = Classes::read(table.from(usize::from(table.u16(8)?))?)?;
                let seconds = Classes::read(table.from(usize::from(table.u16(10)?))?)?;
                let (rows, columns) = (usize::from(table.u16(12)?), usize::from(table.u16(14)?));
                let record = first.0 + second.0;
                let mut values = Vec::with_capacity(rows * columns);
                for cell in 0..rows * columns {
                    values.push(advance(table.from(16 + record * cell)?)?);
                }
                Ok(Pairs::Classed {
                    coverage,
                    first: firsts,
                    second: seconds,
                    columns,
                    values,
                })
            }
            _ => Err(FontError::Unsupported("pair adjustments of another format")),
        }
    }

    /// The units of the font to the em: a text set at `s` points has `s` points to the em.
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// How far the tallest glyphs rise above the baseline, in font units.
    pub fn ascender(&self) -> i16 {
        self.ascender
    }

    /// How far the deepest glyphs fall below the baseline, in font units: negative.
    pub fn descender(&self) -> i16 {
        self.descender
    }

    /// The space the face asks for between two lines beyond the two above, in font units.
    pub fn line_gap(&self) -> i16 {
        self.line_gap
    }

    /// The height of a capital letter, in font units.
    pub fn cap_height(&self) -> i16 {
        self.cap_height
    }

    /// The height of a small letter such as x, in font units.
    pub fn x_height(&self) -> i16 {
        self.x_height
    }

    /// The box that holds every glyph, in font units: left, bottom, right, top.
    pub fn bounds(&self) -> [i16; 4] {
        self.bounds
    }

    /// The name the font goes by in a PDF file, `JetBrainsMono-Regular`; empty when the file
    /// gives none.
    pub fn postscript_name(&self) -> &str {
        &self.postscript_name
    }

    /// How heavy the face is, from 1 to 1000: 400 for a regular face, 600 for a semibold one.
    pub fn weight(&self) -> u16 {
        self.weight
    }

    /// Whether every glyph advances as far as every other.
    pub fn fixed_pitch(&self) -> bool {
        self.fixed_pitch
    }

    /// How far the upright strokes lean, in degrees anticlockwise: 0 for an upright face.
    pub fn italic_angle(&self) -> f64 {
        self.italic_angle
    }

    /// Whether the maker of the font lets a document carry it whole, for anyone to read and
    /// print: the font sets no limit on embedding.
    pub fn may_be_embedded(&self) -> bool {
        self.embedding & 0x030E == 0
    }

    /// The file the face was read from, whole.
    pub fn file_bytes(&self) -> &'static [u8] {
        self.data
    }

    /// The character a glyph draws, when one does: the first in the order of Unicode.
    pub fn character_of(&self, glyph: u16) -> Option<char> {
        self.map
            .iter()
            .find(|&&(_, g)| g == glyph)
            .and_then(|&(code, _)| char::from_u32(code))
    }

    /// The number of glyphs.
    pub fn glyphs(&self) -> u16 {
        self.advances.len() as u16
    }

    /// The characters the face draws, in order.
    pub fn characters(&self) -> impl Iterator<Item = char> + '_ {
        self.map
            .iter()
            .filter_map(|&(code, _)| char::from_u32(code))
    }

    /// The glyph that draws `character`, or `None` when the face lacks it.
    pub fn glyph(&self, character: char) -> Option<u16> {
        let code = u32::from(character);
        let at = self.map.binary_search_by_key(&code, |&(c, _)| c).ok()?;
        Some(self.map[at].1)
    }

    /// How far `glyph` advances, in font units; 0 for a glyph the face does not have.
    pub fn advance(&self, glyph: u16) -> u16 {
        self.advances.get(usize::from(glyph)).copied().unwrap_or(0)
    }

    /// The change to the advance of `left` when `right` follows it, in font units: each
    /// lookup of the `kern` feature adds what its first subtable that speaks of the pair
    /// says.
    pub fn kern(&self, left: u16, right: u16) -> i16 {
        let mut total = 0_i16;
        for (_, lookup) in &self.kerning {
            if let Some(value) = lookup.iter().find_map(|pairs| pairs.adjust(left, right)) {
                total = total.saturating_add(value);
            }
        }
        total
    }

    /// Whether `glyph` is a mark: a glyph the font classes as one, drawn on the glyph before
    /// it and taking no room of its own.
    pub fn is_mark(&self, glyph: u16) -> bool {
        self.classes.of(glyph) == MARK
    }

    /// Whether a lookup passes over `glyph`.
    fn passes(&self, passing: Passing, glyph: u16) -> bool {
        match self.classes.of(glyph) {
            LETTER => passing.flag & Passing::LETTERS != 0,
            LIGATURE => passing.flag & Passing::LIGATURES != 0,
            MARK => {
                passing.flag & Passing::MARKS != 0
                    || (passing.flag & Passing::MARKS_OUTSIDE_A_SET != 0
                        && self
                            .mark_sets
                            .get(passing.set)
                            .is_none_or(|set| set.index(glyph).is_none()))
            }
            _ => false,
        }
    }

    /// The glyphs of one stretch of a line set one after the other: where the origin of each
    /// stands, along the baseline and above it, and the width of the stretch, in font units.
    ///
    /// The advance of a glyph is corrected by the kerning with the glyph after it, a lookup
    /// that passes over marks kerning the letters on either side of them. A mark is then put
    /// on the letter before it, or on the mark before it, where the font has a point for the
    /// two: each lookup of `mark` and `mkmk` in the order of the font, the first of its
    /// subtables that speaks of the pair, a later lookup overruling an earlier one. A mark the
    /// font places nowhere stays where its advance puts it, raised where the letter before it
    /// is a tall one ([`Face::lift`]).
    pub fn place(&self, glyphs: &[u16]) -> (Vec<(i32, i32)>, i32) {
        let mut advances: Vec<i32> = glyphs
            .iter()
            .map(|&glyph| i32::from(self.advance(glyph)))
            .collect();
        for (passing, subtables) in &self.kerning {
            let mut before: Option<usize> = None;
            for (i, &glyph) in glyphs.iter().enumerate() {
                if self.passes(*passing, glyph) {
                    continue;
                }
                if let Some(left) = before
                    && let Some(value) = subtables
                        .iter()
                        .find_map(|pairs| pairs.adjust(glyphs[left], glyph))
                {
                    advances[left] += i32::from(value);
                }
                before = Some(i);
            }
        }
        let on = self.marks_on(glyphs);
        let mut places: Vec<(i32, i32)> = Vec::with_capacity(glyphs.len());
        let mut x = 0_i32;
        for (i, advance) in advances.iter().enumerate() {
            places.push(match on[i] {
                Some((j, dx, dy)) => (places[j].0 + dx, places[j].1 + dy),
                None => (x, self.lift(glyphs, i)),
            });
            x += advance;
        }
        (places, x)
    }

    /// How far above its own height a mark is drawn that the font has no point for on the
    /// letter before it. A font keeps such a mark where it clears a small letter, so over a
    /// letter that rises higher, β or θ, it would run into the letter: a mark whose outline
    /// lies above the small letters is raised by as much as the letter rises above them,
    /// which is what the fonts' own points do for a tall letter they have one for. 0 for any
    /// other glyph, for a mark with no letter before it, and for a mark drawn low.
    fn lift(&self, glyphs: &[u16], at: usize) -> i32 {
        let small = i32::from(self.x_height);
        if small <= 0 || !self.is_mark(glyphs[at]) {
            return 0;
        }
        let Some(letter) = glyphs[..at].iter().rposition(|&other| !self.is_mark(other)) else {
            return 0;
        };
        match (self.heights(glyphs[at]), self.heights(glyphs[letter])) {
            (Some((bottom, _)), Some((_, top))) if i32::from(bottom) >= small => {
                (i32::from(top) - small).max(0)
            }
            _ => 0,
        }
    }

    /// The lowest and the highest point of the outline of `glyph`, in font units, as the font
    /// records them; `None` for a glyph with no outline.
    fn heights(&self, glyph: u16) -> Option<(i16, i16)> {
        let data = self.glyph_data(glyph).ok()?;
        Some((data.i16(4).ok()?, data.i16(8).ok()?))
    }

    /// For each glyph of a stretch, the glyph before it that the font puts it on and how far
    /// from that glyph's origin its own origin then stands; `None` for a glyph that is put on
    /// nothing: a letter, or a mark the font has no point for on what stands before it.
    fn marks_on(&self, glyphs: &[u16]) -> Vec<Option<(usize, i32, i32)>> {
        let mut on: Vec<Option<(usize, i32, i32)>> = vec![None; glyphs.len()];
        for lookup in &self.placing {
            for (i, &glyph) in glyphs.iter().enumerate() {
                if self.passes(lookup.passing, glyph) {
                    continue;
                }
                let found = if lookup.on_marks {
                    // The glyph before it that the lookup looks at, which must be a mark.
                    let back = lookup.passing.back();
                    glyphs[..i]
                        .iter()
                        .rposition(|&other| !self.passes(back, other))
                        .filter(|&j| self.is_mark(glyphs[j]))
                } else {
                    // The letter before it, the marks between passed over.
                    glyphs[..i].iter().rposition(|&other| !self.is_mark(other))
                };
                let Some(j) = found else { continue };
                if let Some((dx, dy)) = lookup
                    .subtables
                    .iter()
                    .find_map(|anchors| anchors.offset(glyph, glyphs[j]))
                {
                    on[i] = Some((j, dx, dy));
                }
            }
        }
        on
    }

    /// Whether the font puts every mark of `glyphs`, a letter and the marks after it, on the
    /// letter or on a mark before it.
    pub fn places_marks(&self, glyphs: &[u16]) -> bool {
        self.marks_on(glyphs)
            .iter()
            .zip(glyphs)
            .skip(1)
            .all(|(on, &glyph)| on.is_some() || !self.is_mark(glyph))
    }

    /// `text` set on one line: brought to its composed form, each character by its glyph, in
    /// order, each advance corrected by the kerning with the glyph after it, and each mark put
    /// on its letter ([`Face::place`]). A composed letter the face lacks is drawn by its parts
    /// when the face has them all; any other character the face lacks is drawn by glyph 0, the
    /// face's empty box, and listed.
    pub fn shape(&self, text: &str) -> Shaped {
        let mut drawn: Vec<u16> = Vec::with_capacity(text.len());
        let mut missing: Vec<char> = Vec::new();
        for character in text.nfc() {
            if let Some(glyphs) = glyphs_of(self, character) {
                drawn.extend(glyphs);
            } else {
                if !missing.contains(&character) {
                    missing.push(character);
                }
                drawn.push(0);
            }
        }
        let (places, width) = self.place(&drawn);
        Shaped {
            glyphs: drawn
                .iter()
                .zip(places)
                .map(|(&glyph, (x, y))| Placed { glyph, x, y })
                .collect(),
            width,
            missing,
        }
    }

    /// The width of `text` set at `size` points, in points.
    pub fn width(&self, text: &str, size: f64) -> f64 {
        f64::from(self.shape(text).width) * size / f64::from(self.units_per_em)
    }

    /// The outline of `glyph`, in font units: closed contours of lines and quadratic curves.
    /// A glyph with no outline (a space) has none.
    pub fn outline(&self, glyph: u16) -> Result<Vec<Segment>, FontError> {
        let mut segments = Vec::new();
        self.add_outline(glyph, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], 0, &mut segments)?;
        Ok(segments)
    }

    /// The bytes of the outline of `glyph`, empty for a glyph without one.
    fn glyph_data(&self, glyph: u16) -> Result<Bytes<'static>, FontError> {
        let at = usize::from(glyph);
        let (Some(&start), Some(&end)) = (self.offsets.get(at), self.offsets.get(at + 1)) else {
            return Err(FontError::Truncated);
        };
        Bytes(self.data)
            .part(self.glyf.0, self.glyf.1)?
            .part(start as usize, (end - start) as usize)
    }

    /// The outline of `glyph` moved and scaled by `matrix` (x' = a·x + c·y + e and
    /// y' = b·x + d·y + f, for `[a, b, c, d, e, f]`), added to `out`.
    fn add_outline(
        &self,
        glyph: u16,
        matrix: [f64; 6],
        depth: u8,
        out: &mut Vec<Segment>,
    ) -> Result<(), FontError> {
        // A glyph made of glyphs made of glyphs: the faces go one deep; a file that goes
        // round in a circle is refused.
        if depth > 8 {
            return Err(FontError::Unsupported("glyphs nested too deep"));
        }
        let data = self.glyph_data(glyph)?;
        if data.0.is_empty() {
            return Ok(());
        }
        let contours = data.i16(0)?;
        if contours >= 0 {
            return Face::add_simple(data, contours as usize, matrix, out);
        }
        // A glyph made of others, each placed by an offset and perhaps scaled.
        let mut at = 10;
        loop {
            let flags = data.u16(at)?;
            let part = data.u16(at + 2)?;
            at += 4;
            if flags & 0x0002 == 0 {
                return Err(FontError::Unsupported(
                    "a part of a glyph placed by matching points",
                ));
            }
            let (dx, dy) = if flags & 0x0001 != 0 {
                at += 4;
                (f64::from(data.i16(at - 4)?), f64::from(data.i16(at - 2)?))
            } else {
                at += 2;
                (
                    f64::from(data.u8(at - 2)? as i8),
                    f64::from(data.u8(at - 1)? as i8),
                )
            };
            let fixed =
                |at: usize| -> Result<f64, FontError> { Ok(f64::from(data.i16(at)?) / 16384.0) };
            let (a, b, c, d) = if flags & 0x0008 != 0 {
                at += 2;
                let scale = fixed(at - 2)?;
                (scale, 0.0, 0.0, scale)
            } else if flags & 0x0040 != 0 {
                at += 4;
                (fixed(at - 4)?, 0.0, 0.0, fixed(at - 2)?)
            } else if flags & 0x0080 != 0 {
                at += 8;
                (
                    fixed(at - 8)?,
                    fixed(at - 6)?,
                    fixed(at - 4)?,
                    fixed(at - 2)?,
                )
            } else {
                (1.0, 0.0, 0.0, 1.0)
            };
            // The part's own matrix, then the one this glyph is drawn under.
            let [pa, pb, pc, pd, pe, pf] = matrix;
            let combined = [
                pa * a + pc * b,
                pb * a + pd * b,
                pa * c + pc * d,
                pb * c + pd * d,
                pa * dx + pc * dy + pe,
                pb * dx + pd * dy + pf,
            ];
            self.add_outline(part, combined, depth + 1, out)?;
            if flags & 0x0020 == 0 {
                return Ok(());
            }
        }
    }

    /// A glyph of `contours` contours given by its points.
    fn add_simple(
        data: Bytes<'static>,
        contours: usize,
        matrix: [f64; 6],
        out: &mut Vec<Segment>,
    ) -> Result<(), FontError> {
        let mut ends = Vec::with_capacity(contours);
        for contour in 0..contours {
            ends.push(usize::from(data.u16(10 + 2 * contour)?));
        }
        let Some(&last) = ends.last() else {
            return Ok(());
        };
        if !ends.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(FontError::Unsupported("contours that are not in order"));
        }
        let points = last + 1;
        let instructions = usize::from(data.u16(10 + 2 * contours)?);
        let mut at = 12 + 2 * contours + instructions;
        // The flags, a repeated one written once with its count.
        let mut flags: Vec<u8> = Vec::with_capacity(points);
        while flags.len() < points {
            let flag = data.u8(at)?;
            at += 1;
            let times = if flag & 0x08 != 0 {
                at += 1;
                1 + usize::from(data.u8(at - 1)?)
            } else {
                1
            };
            if flags.len() + times > points {
                return Err(FontError::Truncated);
            }
            flags.extend(std::iter::repeat_n(flag, times));
        }
        // Each coordinate is a change from the one before: one byte with its sign in the
        // flag, two bytes, or no change.
        let mut read = |short: u8, same: u8| -> Result<Vec<i32>, FontError> {
            let mut values = Vec::with_capacity(points);
            let mut value = 0_i32;
            for &flag in &flags {
                if flag & short != 0 {
                    let step = i32::from(data.u8(at)?);
                    at += 1;
                    value += if flag & same != 0 { step } else { -step };
                } else if flag & same == 0 {
                    value += i32::from(data.i16(at)?);
                    at += 2;
                }
                values.push(value);
            }
            Ok(values)
        };
        let xs = read(0x02, 0x10)?;
        let ys = read(0x04, 0x20)?;
        let [a, b, c, d, e, f] = matrix;
        let place = |x: f64, y: f64| (a * x + c * y + e, b * x + d * y + f);
        let mut start = 0;
        for &end in &ends {
            let on = |i: usize| flags[start + i] & 0x01 != 0;
            let point = |i: usize| (f64::from(xs[start + i]), f64::from(ys[start + i]));
            let count = end + 1 - start;
            let between = |p: (f64, f64), q: (f64, f64)| ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
            // The contour starts at its first point on the curve; when it has none, half
            // way between its last point and its first, which are both controls.
            let first = (0..count).find(|&i| on(i));
            let (origin, begin) = match first {
                Some(i) => (point(i), i + 1),
                None => (between(point(count - 1), point(0)), 0),
            };
            let (x, y) = place(origin.0, origin.1);
            out.push(Segment::Move(x, y));
            let mut control: Option<(f64, f64)> = None;
            let steps = if first.is_some() { count - 1 } else { count };
            for step in 0..steps {
                let i = (begin + step) % count;
                let here = point(i);
                match (on(i), control) {
                    (true, None) => {
                        let (x, y) = place(here.0, here.1);
                        out.push(Segment::Line(x, y));
                    }
                    (true, Some(held)) => {
                        out.push(Segment::Curve(place(held.0, held.1), place(here.0, here.1)));
                        control = None;
                    }
                    (false, None) => control = Some(here),
                    (false, Some(held)) => {
                        // Two controls in a row: the curve passes half way between them.
                        let middle = between(held, here);
                        out.push(Segment::Curve(
                            place(held.0, held.1),
                            place(middle.0, middle.1),
                        ));
                        control = Some(here);
                    }
                }
            }
            // Back to where the contour started.
            if let Some(held) = control {
                out.push(Segment::Curve(
                    place(held.0, held.1),
                    place(origin.0, origin.1),
                ));
            }
            out.push(Segment::Close);
            start = end + 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(words: &[u16]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    #[test]
    fn a_value_record_is_as_long_as_its_format_says() {
        // Nothing; the advance alone; a placement before the advance; devices after it.
        assert_eq!(value_layout(0x0000), (0, None));
        assert_eq!(value_layout(0x0004), (2, Some(0)));
        assert_eq!(value_layout(0x0005), (4, Some(2)));
        assert_eq!(value_layout(0x0007), (6, Some(4)));
        assert_eq!(value_layout(0x0003), (4, None));
        assert_eq!(value_layout(0x0044), (4, Some(0)));
        assert_eq!(value_layout(0x00FF), (16, Some(4)));
    }

    #[test]
    fn a_coverage_gives_each_glyph_its_index_in_both_forms() {
        // A list of glyphs.
        let listed = bytes(&[1, 3, 5, 9, 20]);
        let coverage = Coverage::read(Bytes(&listed)).unwrap();
        assert_eq!(coverage.index(5), Some(0));
        assert_eq!(coverage.index(9), Some(1));
        assert_eq!(coverage.index(20), Some(2));
        assert_eq!(coverage.index(4), None);
        assert_eq!(coverage.index(10), None);
        assert_eq!(coverage.index(21), None);
        // Ranges, each with the index of its first glyph.
        let ranged = bytes(&[2, 2, 5, 7, 0, 30, 31, 3]);
        let coverage = Coverage::read(Bytes(&ranged)).unwrap();
        assert_eq!(coverage.index(5), Some(0));
        assert_eq!(coverage.index(7), Some(2));
        assert_eq!(coverage.index(31), Some(4));
        assert_eq!(coverage.index(8), None);
        assert_eq!(coverage.index(0), None);
        // Another format, and a table cut short.
        assert_eq!(
            Coverage::read(Bytes(&bytes(&[3, 0]))),
            Err(FontError::Unsupported("a coverage table of another format"))
        );
        assert_eq!(
            Coverage::read(Bytes(&listed[..7])),
            Err(FontError::Truncated)
        );
    }

    #[test]
    fn a_glyph_in_no_class_is_of_class_zero() {
        let listed = bytes(&[1, 10, 3, 2, 0, 7]);
        let classes = Classes::read(Bytes(&listed)).unwrap();
        assert_eq!(
            [9, 10, 11, 12, 13].map(|glyph| classes.of(glyph)),
            [0, 2, 0, 7, 0]
        );
        let ranged = bytes(&[2, 2, 4, 6, 1, 20, 20, 5]);
        let classes = Classes::read(Bytes(&ranged)).unwrap();
        assert_eq!(
            [3, 4, 6, 7, 20, 21].map(|glyph| classes.of(glyph)),
            [0, 1, 1, 0, 5, 0]
        );
    }

    #[test]
    fn a_listed_subtable_speaks_of_its_pairs_and_a_classed_one_of_every_pair_it_covers() {
        let coverage = || Coverage(vec![(4, 5, 0)]);
        let listed = Pairs::Listed {
            coverage: coverage(),
            sets: vec![vec![(7, -30), (9, 12)], vec![]],
        };
        assert_eq!(listed.adjust(4, 7), Some(-30));
        assert_eq!(listed.adjust(4, 9), Some(12));
        // Covered, but the pair is not listed: the next subtable is asked.
        assert_eq!(listed.adjust(4, 8), None);
        assert_eq!(listed.adjust(5, 7), None);
        assert_eq!(listed.adjust(6, 7), None);
        let classed = Pairs::Classed {
            coverage: coverage(),
            first: Classes(vec![(5, 5, 1)]),
            second: Classes(vec![(7, 7, 1)]),
            columns: 2,
            values: vec![0, -10, 0, -50],
        };
        assert_eq!(classed.adjust(4, 7), Some(-10));
        assert_eq!(classed.adjust(5, 7), Some(-50));
        // Covered: it answers, with nothing, and no later subtable is asked.
        assert_eq!(classed.adjust(5, 8), Some(0));
        assert_eq!(classed.adjust(6, 7), None);
    }

    #[test]
    fn the_three_faces_read_and_are_three_different_fonts() {
        let [sans, bold, mono] = FaceName::ALL.map(face);
        assert_eq!(sans.glyphs(), 392);
        assert_eq!(bold.glyphs(), 392);
        assert_eq!(mono.glyphs(), 1743);
        assert_ne!(sans, bold);
        // The two families have their own heights: the sans faces share theirs.
        for (font, heights) in [
            (sans, (984, -316, 0)),
            (bold, (984, -316, 0)),
            (mono, (1020, -300, 0)),
        ] {
            assert_eq!(font.units_per_em(), 1000);
            assert_eq!(
                (font.ascender(), font.descender(), font.line_gap()),
                heights
            );
            assert!(font.x_height() > 400 && font.cap_height() > font.x_height());
            assert!(font.cap_height() < font.ascender());
        }
        assert!(mono.kerning.is_empty());
        assert_eq!(sans.kerning.len(), 2);
    }
}
