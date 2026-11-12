//! The fonts of a figure (Language Reference, section 7.6, "Rendering and export"; release
//! 0.7.0, step 21a): IBM Plex Sans and IBM Plex Mono, embedded in the engine and never taken
//! from the computer, so that a text has the same width and the same shape on every machine.
//!
//! The files are in `fonts/`, unchanged, under the SIL Open Font License (`fonts/SOURCE.md`
//! gives their source and hashes). This module reads from them, with its own code, what a
//! figure needs and nothing more:
//!
//! - which glyph draws a character (`cmap`, the Unicode map of format 4);
//! - how far each glyph advances (`hmtx`) and the heights of the face (`hhea`, `OS/2`);
//! - the outline of each glyph (`glyf` and `loca`), simple or made of other glyphs;
//! - how much closer or farther two neighbours are set (`GPOS`, the pair adjustments of the
//!   `kern` feature).
//!
//! A text is set by one fixed rule. It is first brought to its composed form (Unicode's NFC),
//! so that a letter and a mark written after it are the one letter Unicode has for the two,
//! `é` for `e` and U+0301; a composed letter the face lacks is drawn by its parts when the
//! face has them all. Each character is then drawn by the glyph the map gives it, in order
//! from left to right, and the advance of a glyph is corrected by the kerning with the glyph
//! after it. No ligature is formed (`fi` is two glyphs), no mark is moved onto a letter, and
//! nothing depends on a language. A character the face lacks is drawn by the glyph the face
//! keeps for that, an empty box, and is reported.
//!
//! Every length is a whole number of font units, of which the faces have 1000 to the em: a
//! text of `w` units set at `s` points is `w × s / 1000` points wide.

use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::decompose_canonical;

/// The faces embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceName {
    /// IBM Plex Sans Regular: the labels.
    Sans,
    /// IBM Plex Sans SemiBold: the titles.
    SansSemiBold,
    /// IBM Plex Mono Regular: code and aligned digits.
    Mono,
}

impl FaceName {
    pub const ALL: [FaceName; 3] = [FaceName::Sans, FaceName::SansSemiBold, FaceName::Mono];

    /// The file of the face, as it is embedded.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            FaceName::Sans => include_bytes!("../fonts/IBMPlexSans-Regular.ttf"),
            FaceName::SansSemiBold => include_bytes!("../fonts/IBMPlexSans-SemiBold.ttf"),
            FaceName::Mono => include_bytes!("../fonts/IBMPlexMono-Regular.ttf"),
        }
    }

    /// The name of the file in `fonts/`.
    pub fn file(self) -> &'static str {
        match self {
            FaceName::Sans => "IBMPlexSans-Regular.ttf",
            FaceName::SansSemiBold => "IBMPlexSans-SemiBold.ttf",
            FaceName::Mono => "IBMPlexMono-Regular.ttf",
        }
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

/// A glyph of a text, with the place of its origin along the baseline, in font units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub glyph: u16,
    pub x: i32,
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
    /// The advance of each glyph.
    advances: Vec<u16>,
    /// The characters the face draws, in order, each with its glyph.
    map: Vec<(u32, u16)>,
    /// Where each glyph's outline starts in `glyf`, and after the last where it ends.
    offsets: Vec<u32>,
    glyf: (usize, usize),
    /// The lookups of the `kern` feature, each a list of subtables.
    kerning: Vec<Vec<Pairs>>,
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
        let map = Face::read_map(need("cmap")?, glyphs)?;
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
        let kerning = match table("GPOS")? {
            Some((offset, length)) => Face::read_kerning(file.part(offset, length)?)?,
            None => Vec::new(),
        };
        Ok(Face {
            data,
            units_per_em,
            ascender,
            descender,
            line_gap,
            cap_height,
            x_height,
            advances,
            map,
            offsets,
            glyf,
            kerning,
        })
    }

    /// The characters of the Unicode map of format 4, each with its glyph.
    fn read_map(cmap: Bytes<'static>, glyphs: usize) -> Result<Vec<(u32, u16)>, FontError> {
        let mut found = None;
        for i in 0..usize::from(cmap.u16(2)?) {
            let at = 4 + 8 * i;
            let (platform, encoding) = (cmap.u16(at)?, cmap.u16(at + 2)?);
            let subtable = cmap.from(cmap.u32(at + 4)? as usize)?;
            // Unicode, of the basic plane: platform 3 with encoding 1, or platform 0.
            let unicode = (platform == 3 && encoding == 1) || platform == 0;
            if unicode && subtable.u16(0)? == 4 {
                found = Some(subtable);
                if platform == 3 {
                    break;
                }
            }
        }
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
        if !map.windows(2).all(|pair| pair[0].0 < pair[1].0) {
            return Err(FontError::Unsupported(
                "a map whose segments are not in order",
            ));
        }
        Ok(map)
    }

    /// The lookups of pair adjustments that the `kern` features name, in the order of the
    /// table.
    fn read_kerning(gpos: Bytes<'static>) -> Result<Vec<Vec<Pairs>>, FontError> {
        let features = gpos.from(usize::from(gpos.u16(6)?))?;
        let mut named: Vec<u16> = Vec::new();
        for i in 0..usize::from(features.u16(0)?) {
            let at = 2 + 6 * i;
            if features.part(at, 4)?.0 != b"kern" {
                continue;
            }
            let feature = features.from(usize::from(features.u16(at + 4)?))?;
            for k in 0..usize::from(feature.u16(2)?) {
                let lookup = feature.u16(4 + 2 * k)?;
                if !named.contains(&lookup) {
                    named.push(lookup);
                }
            }
        }
        named.sort_unstable();
        let lookups = gpos.from(usize::from(gpos.u16(8)?))?;
        let mut kerning = Vec::new();
        for index in named {
            if index >= lookups.u16(0)? {
                return Err(FontError::Truncated);
            }
            let lookup = lookups.from(usize::from(lookups.u16(2 + 2 * usize::from(index))?))?;
            let kind = lookup.u16(0)?;
            let mut subtables = Vec::new();
            for k in 0..usize::from(lookup.u16(4)?) {
                let mut subtable = lookup.from(usize::from(lookup.u16(6 + 2 * k)?))?;
                let mut of = kind;
                // An extension holds a subtable of another kind farther away.
                if of == 9 {
                    of = subtable.u16(2)?;
                    subtable = subtable.from(subtable.u32(4)? as usize)?;
                }
                if of == 2 {
                    subtables.push(Face::read_pairs(subtable)?);
                }
            }
            kerning.push(subtables);
        }
        Ok(kerning)
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
        for lookup in &self.kerning {
            if let Some(value) = lookup.iter().find_map(|pairs| pairs.adjust(left, right)) {
                total = total.saturating_add(value);
            }
        }
        total
    }

    /// `text` set on one line: brought to its composed form, each character by its glyph, in
    /// order, each advance corrected by the kerning with the glyph after it. A composed
    /// letter the face lacks is drawn by its parts when the face has them all; any other
    /// character the face lacks is drawn by glyph 0, the face's empty box, and listed.
    pub fn shape(&self, text: &str) -> Shaped {
        let mut drawn: Vec<u16> = Vec::with_capacity(text.len());
        let mut missing: Vec<char> = Vec::new();
        for character in text.nfc() {
            if let Some(glyph) = self.glyph(character) {
                drawn.push(glyph);
                continue;
            }
            let mut parts: Vec<Option<u16>> = Vec::new();
            decompose_canonical(character, |part| parts.push(self.glyph(part)));
            if parts.len() > 1 && parts.iter().all(Option::is_some) {
                drawn.extend(parts.into_iter().flatten());
            } else {
                if !missing.contains(&character) {
                    missing.push(character);
                }
                drawn.push(0);
            }
        }
        let mut glyphs: Vec<Placed> = Vec::with_capacity(drawn.len());
        let mut x = 0_i32;
        for (i, &glyph) in drawn.iter().enumerate() {
            if i > 0 {
                x += i32::from(self.kern(drawn[i - 1], glyph));
            }
            glyphs.push(Placed { glyph, x });
            x += i32::from(self.advance(glyph));
        }
        Shaped {
            glyphs,
            width: x,
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
        assert_eq!(sans.glyphs(), 1019);
        assert_eq!(bold.glyphs(), 1019);
        assert_eq!(mono.glyphs(), 1207);
        assert_ne!(sans, bold);
        for font in [sans, bold, mono] {
            assert_eq!(font.units_per_em(), 1000);
            assert_eq!(
                (font.ascender(), font.descender(), font.line_gap()),
                (1025, -275, 0)
            );
            assert!(font.x_height() > 400 && font.cap_height() > font.x_height());
            assert!(font.cap_height() < font.ascender());
        }
        assert!(mono.kerning.is_empty());
        assert_eq!(sans.kerning.len(), 3);
    }
}
