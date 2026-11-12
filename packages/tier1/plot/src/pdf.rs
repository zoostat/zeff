//! The PDF of a drawing (Language Reference, section 7.6, "Rendering and export"; release
//! 0.7.0, step 22b1; open decision #118): the bytes of a figure as a file of one page, the
//! same on every machine.
//!
//! [`write`] turns a drawing ([`crate::drawing`]) into the bytes of a PDF file and is a pure
//! function of it. The page is as large as the drawing, in points. Every primitive of the
//! closed set is drawn as lines and curves, and nothing else is written: no date, no version
//! of the program, no path of a file, no name of a machine. The identifier of the file is
//! taken from its own bytes. A length is written by the one rule of
//! [`crate::drawing::length`], to a thousandth of a point.
//!
//! **The page.** A drawing counts down from the top left and a PDF page up from the bottom
//! left, so the page begins by turning its axes over once; every place after that is written
//! as the drawing has it. Lines meet as they do in the SVG of the same drawing.
//!
//! **Text.** A text is text: the file carries each face it uses whole
//! ([`crate::font`]), and sets the words glyph by glyph at the places and with the kerning of
//! [`crate::font::Face::shape`], so that a reader can select the words, search for them and
//! copy them. Each face comes with the map from its glyphs back to characters: a glyph gives
//! the first character, in the order of Unicode, that the face draws with it. A character no
//! face has is drawn as the face's empty box and maps to nothing.
//!
//! **Marks.** A point is drawn by the geometry of its shape ([`crate::shape::outline`]). A
//! shape of one size met [`SHARED`] times or more is written once and placed where each mark
//! stands, so that a figure of many thousand marks is a short line for each; a shape met less
//! often is written where it stands. Neighbouring marks of one colour, opacity and kind share
//! their paint. A mark that is not opaque is painted by itself, so that marks that overlap
//! darken each other as they do on a screen.
//!
//! **Cells.** A grid of cells is an image of the cells, each repeated until no dot of it is
//! wider than a three-hundredth of an inch, so that a program that smooths images when it
//! shows them leaves the edges of the cells sharp.
//!
//! **Names.** The names of the items of a drawing are not carried into the file. A drawing
//! holds no row of the data, and so no file does.
//!
//! [`write_plain`] gives the same file with its streams not compressed, for reading and for
//! the tests.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use miniz_oxide::deflate::compress_to_vec_zlib;
use zeff_numeric::maths::sqrt;

use crate::drawing::{
    Anchor, Drawing, Head, Item, Place, Primitive, Span, Stroke, TextBox, length,
};
use crate::font::{FaceName, face};
use crate::shape::Geometry;
use crate::svg::{ABOVE, BELOW, MARK_STROKE, geometry, turned};
use crate::trace::{Piece, Trace};

/// How hard the streams are compressed, of 10: the same on every machine.
const LEVEL: u8 = 9;

/// How many marks of one shape and size make it worth writing the shape once and placing it.
pub const SHARED: usize = 3;

/// How fine the image of a grid of cells is, in dots to the inch, where its cells allow.
pub const DOTS: f64 = 300.0;

/// The most a cell is repeated each way, and the most dots an image is made of by repeating.
const FINEST: usize = 64;
const MOST_DOTS: usize = 4_000_000;

/// How far a corner of a line may reach beyond its place, in widths of the line, before it
/// is cut flat: what an SVG reader uses.
const MITRE: u8 = 4;

/// The PDF file of `drawing`.
pub fn write(drawing: &Drawing) -> Vec<u8> {
    assemble(drawing, true)
}

/// The PDF file of `drawing` with its streams as they are, not compressed.
pub fn write_plain(drawing: &Drawing) -> Vec<u8> {
    assemble(drawing, false)
}

/// A number with up to six decimals, for the turn of a text; `0` for what is no number.
fn fine(x: f64) -> String {
    let millionths = (x * 1_000_000.0).round();
    if millionths == 0.0 || !millionths.is_finite() {
        return "0".to_string();
    }
    let sign = if millionths < 0.0 { "-" } else { "" };
    let whole = millionths.abs() as u64;
    let (units, part) = (whole / 1_000_000, whole % 1_000_000);
    if part == 0 {
        format!("{sign}{units}")
    } else {
        let digits = format!("{part:06}");
        format!("{sign}{units}.{}", digits.trim_end_matches('0'))
    }
}

/// The red, green and blue of `#RRGGBB`, each of 255; black for what is no such colour.
pub fn rgb(colour: &str) -> [u8; 3] {
    let digits = colour.strip_prefix('#').unwrap_or("");
    if digits.len() != 6 || !digits.is_ascii() {
        return [0, 0, 0];
    }
    let part = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
    match (part(0), part(2), part(4)) {
        (Some(red), Some(green), Some(blue)) => [red, green, blue],
        _ => [0, 0, 0],
    }
}

/// A colour as three shares of one.
fn shares(colour: &str) -> String {
    let [red, green, blue] = rgb(colour);
    format!(
        "{} {} {}",
        length(f64::from(red) / 255.0),
        length(f64::from(green) / 255.0),
        length(f64::from(blue) / 255.0)
    )
}

/// An opacity as it is written: a share of one, and one for what is no number.
fn share(opacity: f64) -> String {
    if opacity.is_nan() {
        return "1".to_string();
    }
    length(opacity.clamp(0.0, 1.0))
}

impl Trace {
    /// The path as the operators of a page: every place by the one rule of lengths.
    fn ops(&self) -> String {
        let mut out = String::new();
        for piece in &self.pieces {
            match *piece {
                Piece::Move(p) => out.push_str(&format!("{} {} m ", length(p.0), length(p.1))),
                Piece::Line(p) => out.push_str(&format!("{} {} l ", length(p.0), length(p.1))),
                Piece::Curve(a, b, p) => out.push_str(&format!(
                    "{} {} {} {} {} {} c ",
                    length(a.0),
                    length(a.1),
                    length(b.0),
                    length(b.1),
                    length(p.0),
                    length(p.1)
                )),
                Piece::Close => out.push_str("h "),
            }
        }
        out
    }
}

/// The operator that paints a path: filled, stroked, both or neither; `even_odd` when what
/// lies inside a hole is outside.
fn painting(fill: bool, stroke: bool, even_odd: bool) -> &'static str {
    match (fill, stroke, even_odd) {
        (true, true, false) => "B",
        (true, true, true) => "B*",
        (true, false, false) => "f",
        (true, false, true) => "f*",
        (false, true, _) => "S",
        (false, false, _) => "n",
    }
}

/// A shape of one size, written once and placed where each of its marks stands.
struct Form {
    ops: String,
    /// How far the shape and its line reach from its centre.
    reach: f64,
}

/// An image: its dots, row by row from the top, three bytes each.
struct Image {
    columns: usize,
    rows: usize,
    dots: Vec<u8>,
}

/// A mark: where it stands, its shape and its paint.
struct Mark<'a> {
    centre: Place,
    name: &'static str,
    size: f64,
    geometry: Geometry,
    colour: &'a str,
    opacity: String,
    stroked: bool,
}

fn mark_of(primitive: &Primitive) -> Option<Mark<'_>> {
    match primitive {
        Primitive::Point {
            at,
            shape,
            size,
            colour,
            opacity,
        } => {
            let outline = geometry(shape, *size);
            Some(Mark {
                centre: *at,
                name: shape,
                size: *size,
                stroked: outline.open || matches!(outline.geometry, Geometry::Lines(_)),
                geometry: outline.geometry,
                colour,
                opacity: share(*opacity),
            })
        }
        _ => None,
    }
}

/// The index of a face among the embedded ones: its font is `F` and one more than it.
fn index_of(name: FaceName) -> usize {
    FaceName::ALL
        .iter()
        .position(|each| *each == name)
        .unwrap_or(0)
}

/// What the page draws, and what it draws with.
struct Content {
    out: String,
    /// The opacities used, as written, in the order met: the graphics state `G` and one more
    /// than its place here.
    states: Vec<String>,
    /// The glyphs used of each embedded face.
    glyphs: [BTreeSet<u16>; 3],
    /// How often each shape of each size is drawn in the whole drawing.
    uses: BTreeMap<(&'static str, String), usize>,
    /// The shapes written once, in the order met: the form `M` and one more than its place.
    placed: BTreeMap<(&'static str, String), usize>,
    forms: Vec<Form>,
    /// The images, in the order met: the image `I` and one more than its place.
    images: Vec<Image>,
}

impl Content {
    /// The name of the graphics state of an opacity.
    fn state(&mut self, opacity: &str) -> String {
        let at = match self.states.iter().position(|each| each == opacity) {
            Some(at) => at,
            None => {
                self.states.push(opacity.to_string());
                self.states.len() - 1
            }
        };
        format!("/G{}", at + 1)
    }

    /// Items from the bottom up, neighbouring marks of one paint sharing it.
    fn items(&mut self, items: &[Item]) {
        let mut at = 0;
        while at < items.len() {
            let Some(first) = mark_of(&items[at].primitive) else {
                self.primitive(&items[at].primitive);
                at += 1;
                continue;
            };
            let same = |mark: &Mark| {
                mark.colour == first.colour
                    && mark.opacity == first.opacity
                    && mark.stroked == first.stroked
            };
            let mut end = at + 1;
            while end < items.len()
                && mark_of(&items[end].primitive).is_some_and(|next| same(&next))
            {
                end += 1;
            }
            self.out.push_str("q\n");
            if first.opacity != "1" {
                let state = self.state(&first.opacity);
                self.out.push_str(&format!("{state} gs\n"));
            }
            if first.stroked {
                self.out.push_str(&format!(
                    "{} RG {} w\n",
                    shares(first.colour),
                    length(MARK_STROKE)
                ));
            } else {
                self.out.push_str(&format!("{} rg\n", shares(first.colour)));
            }
            for item in &items[at..end] {
                if let Some(mark) = mark_of(&item.primitive) {
                    self.mark(&mark);
                }
            }
            self.out.push_str("Q\n");
            at = end;
        }
    }

    /// One mark, in the paint already set: placed, when its shape is shared, or written
    /// where it stands.
    fn mark(&mut self, mark: &Mark) {
        let paint = if mark.stroked { "S" } else { "f" };
        let key = (mark.name, length(mark.size));
        if self.uses.get(&key).copied().unwrap_or(0) < SHARED {
            let mut trace = Trace::default();
            trace.shape(mark.centre, &mark.geometry);
            self.out.push_str(&format!("{}{paint}\n", trace.ops()));
            return;
        }
        let at = match self.placed.get(&key) {
            Some(at) => *at,
            None => {
                let mut trace = Trace::default();
                trace.shape((0.0, 0.0), &mark.geometry);
                let far = match &mark.geometry {
                    Geometry::Circle(radius) => radius.abs(),
                    Geometry::Polygon(corners) => corners
                        .iter()
                        .map(|c| c.0.abs().max(c.1.abs()))
                        .fold(0.0, f64::max),
                    Geometry::Lines(strokes) => strokes
                        .iter()
                        .flat_map(|(a, b)| [a.0.abs(), a.1.abs(), b.0.abs(), b.1.abs()])
                        .fold(0.0, f64::max),
                };
                self.forms.push(Form {
                    ops: format!("{}{paint}\n", trace.ops()),
                    // A corner of a line reaches beyond its place by at most this.
                    reach: far + f64::from(MITRE) * MARK_STROKE,
                });
                self.placed.insert(key, self.forms.len() - 1);
                self.forms.len() - 1
            }
        };
        self.out.push_str(&format!(
            "q 1 0 0 1 {} {} cm /M{} Do Q\n",
            length(mark.centre.0),
            length(mark.centre.1),
            at + 1
        ));
    }

    /// A path in its paint: the opacity, the colours, the path and the operator.
    fn painted(
        &mut self,
        path: &str,
        fill: &Option<String>,
        stroke: &Option<Stroke>,
        opacity: f64,
        even_odd: bool,
    ) {
        let opacity = share(opacity);
        let through = opacity != "1" && (fill.is_some() || stroke.is_some());
        if through {
            let state = self.state(&opacity);
            self.out.push_str(&format!("q {state} gs\n"));
        }
        if let Some(colour) = fill {
            self.out.push_str(&format!("{} rg\n", shares(colour)));
        }
        if let Some(line) = stroke {
            self.out.push_str(&format!(
                "{} RG {} w\n",
                shares(&line.colour),
                length(line.width)
            ));
        }
        self.out.push_str(&format!(
            "{path}{}\n",
            painting(fill.is_some(), stroke.is_some(), even_odd)
        ));
        if through {
            self.out.push_str("Q\n");
        }
    }

    fn primitive(&mut self, primitive: &Primitive) {
        match primitive {
            // A point is a run of one, drawn by `items`.
            Primitive::Point { .. } => {}
            Primitive::Path {
                steps,
                stroke,
                fill,
                opacity,
            } => {
                let mut trace = Trace::default();
                for step in steps {
                    trace.step(step);
                }
                if trace.begun {
                    self.painted(&trace.ops(), fill, stroke, *opacity, false);
                }
            }
            Primitive::Polygon {
                rings,
                fill,
                stroke,
                opacity,
            } => {
                let mut trace = Trace::default();
                for ring in rings {
                    for (i, corner) in ring.iter().enumerate() {
                        if i == 0 {
                            trace.to(*corner)
                        } else {
                            trace.line(*corner)
                        }
                    }
                    if !ring.is_empty() {
                        trace.close();
                    }
                }
                if trace.begun {
                    self.painted(&trace.ops(), fill, stroke, *opacity, true);
                }
            }
            Primitive::Rectangle {
                at,
                width,
                height,
                radius,
                fill,
                stroke,
            } => {
                let path = if *radius > 0.0 {
                    let mut trace = Trace::default();
                    trace.rounded(*at, *width, *height, *radius, |p| p);
                    trace.ops()
                } else {
                    format!(
                        "{} {} {} {} re ",
                        length(at.0),
                        length(at.1),
                        length(*width),
                        length(*height)
                    )
                };
                self.painted(&path, fill, stroke, 1.0, false);
            }
            Primitive::Segment {
                from,
                to,
                stroke,
                heads,
            } => {
                self.out.push_str(&format!(
                    "{} RG {} w\n{} {} m {} {} l S\n",
                    shares(&stroke.colour),
                    length(stroke.width),
                    length(from.0),
                    length(from.1),
                    length(to.0),
                    length(to.1)
                ));
                if heads.0 == Head::Arrow {
                    self.arrow(*to, *from, stroke);
                }
                if heads.1 == Head::Arrow {
                    self.arrow(*from, *to, stroke);
                }
            }
            Primitive::Text {
                at,
                anchor,
                turn,
                spans,
                frame,
            } => self.text(*at, *anchor, *turn, spans, frame.as_ref()),
            Primitive::Raster {
                at,
                width,
                height,
                columns,
                rows,
                cells,
            } => {
                if *columns == 0 || *rows == 0 {
                    return;
                }
                let (wide, high) = (width / *columns as f64, height / *rows as f64);
                // The rows that are whole, and then the cells of a row that is not.
                let whole = (cells.len() / columns).min(*rows);
                self.image((at.0, at.1), wide, high, *columns, whole, cells);
                if whole < *rows {
                    let rest = &cells[whole * columns..];
                    self.image(
                        (at.0, at.1 + whole as f64 * high),
                        wide,
                        high,
                        rest.len(),
                        1,
                        rest,
                    );
                }
            }
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } => {
                self.out.push_str(&format!(
                    "q {} {} {} {} re W n\n",
                    length(at.0),
                    length(at.1),
                    length(*width),
                    length(*height)
                ));
                self.items(items);
                self.out.push_str("Q\n");
            }
        }
    }

    /// Cells of one size as an image with its top left at `at`: each cell repeated until a
    /// dot is no wider than one of [`DOTS`] to the inch, where that keeps the image small
    /// enough.
    fn image(
        &mut self,
        at: Place,
        wide: f64,
        high: f64,
        columns: usize,
        rows: usize,
        cells: &[String],
    ) {
        if columns == 0 || rows == 0 {
            return;
        }
        let times = |points: f64| {
            let wanted = (points * DOTS / 72.0).ceil();
            if wanted >= 1.0 {
                (wanted as usize).min(FINEST)
            } else {
                1
            }
        };
        let (mut across, mut down) = (times(wide), times(high));
        while (across > 1 || down > 1) && (columns * across).saturating_mul(rows * down) > MOST_DOTS
        {
            if across >= down {
                across -= 1;
            } else {
                down -= 1;
            }
        }
        let mut dots = Vec::with_capacity(columns * across * rows * down * 3);
        for row in 0..rows {
            let mut line = Vec::with_capacity(columns * across * 3);
            for column in 0..columns {
                let colour = rgb(&cells[row * columns + column]);
                for _ in 0..across {
                    line.extend_from_slice(&colour);
                }
            }
            for _ in 0..down {
                dots.extend_from_slice(&line);
            }
        }
        self.images.push(Image {
            columns: columns * across,
            rows: rows * down,
            dots,
        });
        // An image fills a square of one by one, its first row at the top.
        let (width, height) = (wide * columns as f64, high * rows as f64);
        self.out.push_str(&format!(
            "q {} 0 0 {} {} {} cm /I{} Do Q\n",
            length(width),
            length(-height),
            length(at.0),
            length(at.1 + height),
            self.images.len()
        ));
    }

    /// The head of an arrow at `tip`, on a line that comes from `tail`: a filled triangle
    /// six strokes long, and at least four points.
    fn arrow(&mut self, tail: Place, tip: Place, stroke: &Stroke) {
        let (dx, dy) = (tip.0 - tail.0, tip.1 - tail.1);
        let run = sqrt(dx * dx + dy * dy);
        // A segment of no length, or to a place that is no number, has no direction.
        if run.is_nan() || run <= 0.0 {
            return;
        }
        let (ux, uy) = (dx / run, dy / run);
        let long = (6.0 * stroke.width).max(4.0);
        let half = 0.35 * long;
        let base = (tip.0 - ux * long, tip.1 - uy * long);
        let mut trace = Trace::default();
        trace.to(tip);
        trace.line((base.0 - uy * half, base.1 + ux * half));
        trace.line((base.0 + uy * half, base.1 - ux * half));
        trace.close();
        self.out.push_str(&format!(
            "{} rg\n{}f\n",
            shares(&stroke.colour),
            trace.ops()
        ));
    }

    /// A line of text: the box behind it, and its words set glyph by glyph.
    fn text(
        &mut self,
        at: Place,
        anchor: Anchor,
        turn: f64,
        spans: &[Span],
        frame: Option<&TextBox>,
    ) {
        if spans.iter().all(|span| span.text.is_empty()) {
            return;
        }
        // Each span set by its face, one after the other along the baseline.
        let set: Vec<_> = spans
            .iter()
            .map(|span| {
                let font = face(span.face);
                let shaped = font.shape(&span.text);
                let width = f64::from(shaped.width) * span.size / f64::from(font.units_per_em());
                (span, font, width, shaped)
            })
            .collect();
        let total: f64 = set.iter().map(|(_, _, width, _)| width).sum();
        let largest = spans.iter().map(|span| span.size).fold(0.0, f64::max);
        let start = match anchor {
            Anchor::Start => 0.0,
            Anchor::Middle => -total / 2.0,
            Anchor::End => -total,
        };
        // A place given along the baseline and above it, turned about the anchor.
        let (cosine, sine) = turned(turn);
        let place = |along: f64, above: f64| {
            (
                at.0 + along * cosine - above * sine,
                at.1 - along * sine - above * cosine,
            )
        };
        if let Some(frame) = frame {
            let mut trace = Trace::default();
            trace.rounded(
                (
                    at.0 + start - frame.padding,
                    at.1 - ABOVE * largest - frame.padding,
                ),
                total + 2.0 * frame.padding,
                (ABOVE + BELOW) * largest + 2.0 * frame.padding,
                frame.radius,
                |p| place(p.0 - at.0, at.1 - p.1),
            );
            self.painted(&trace.ops(), &frame.fill, &frame.border, 1.0, false);
        }
        self.out.push_str("BT\n");
        let mut along = start;
        for (span, font, width, shaped) in &set {
            if !shaped.glyphs.is_empty() {
                let index = index_of(span.face);
                let per_em = 1000.0 / f64::from(font.units_per_em());
                // The glyphs in one string, broken where a glyph stands closer to the one
                // before it, or farther from it, than its advance puts it.
                let mut shown = String::from("[<");
                for (i, placed) in shaped.glyphs.iter().enumerate() {
                    if i > 0 {
                        let before = &shaped.glyphs[i - 1];
                        let closer = before.x + i32::from(font.advance(before.glyph)) - placed.x;
                        if closer != 0 {
                            shown.push_str(&format!("> {} <", length(f64::from(closer) * per_em)));
                        }
                    }
                    shown.push_str(&format!("{:04X}", placed.glyph));
                    self.glyphs[index].insert(placed.glyph);
                }
                shown.push_str(">] TJ");
                // The letters of a font stand upright on a page that counts upwards: the
                // text is turned over once more, and then turned as it was asked to be.
                let origin = place(along, 0.0);
                self.out.push_str(&format!(
                    "/F{} {} Tf {} rg {} {} {} {} {} {} Tm {shown}\n",
                    index + 1,
                    length(span.size),
                    shares(&span.colour),
                    fine(cosine),
                    fine(-sine),
                    fine(-sine),
                    fine(-cosine),
                    length(origin.0),
                    length(origin.1)
                ));
            }
            along += width;
        }
        self.out.push_str("ET\n");
    }
}

/// A stream: what it says of itself, and its bytes, compressed when `pack` says so.
fn stream(entries: &str, data: &[u8], pack: bool) -> Vec<u8> {
    let packed;
    let (filter, body): (&str, &[u8]) = if pack {
        packed = compress_to_vec_zlib(data, LEVEL);
        (" /Filter /FlateDecode", &packed)
    } else {
        ("", data)
    };
    let mut out = format!("<< {entries}/Length {}{filter} >>\nstream\n", body.len()).into_bytes();
    out.extend_from_slice(body);
    out.extend_from_slice(b"\nendstream");
    out
}

/// The file of a face as a stream, compressed once for all the figures of a run.
fn font_file(name: FaceName, pack: bool) -> Vec<u8> {
    static PACKED: [OnceLock<Vec<u8>>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let entries = format!("/Length1 {} ", name.bytes().len());
    if !pack {
        return stream(&entries, name.bytes(), false);
    }
    PACKED[index_of(name)]
        .get_or_init(|| stream(&entries, name.bytes(), true))
        .clone()
}

/// The five objects of a face, the first of them numbered `first`: the font, the font it is
/// made of, what describes it, its file, and the map from its glyphs to characters.
fn font_objects(name: FaceName, used: &BTreeSet<u16>, first: usize, pack: bool) -> Vec<Vec<u8>> {
    let font = face(name);
    let base = if font.postscript_name().is_empty() {
        name.file().trim_end_matches(".ttf").replace('-', "")
    } else {
        font.postscript_name().to_string()
    };
    let per_em = 1000.0 / f64::from(font.units_per_em());
    let em = |units: i16| length(f64::from(units) * per_em);
    // The advances of the glyphs used, neighbours in one list.
    let mut widths = String::new();
    let mut before: Option<u16> = None;
    for &glyph in used {
        if before.is_some_and(|b| b + 1 == glyph) {
            widths.push(' ');
        } else {
            if before.is_some() {
                widths.push_str("] ");
            }
            widths.push_str(&format!("{glyph} ["));
        }
        widths.push_str(&length(f64::from(font.advance(glyph)) * per_em));
        before = Some(glyph);
    }
    if before.is_some() {
        widths.push(']');
    }
    // Of one width, 1; not the Latin of the standard encodings alone, 4; leaning, 64.
    let flags = u8::from(font.fixed_pitch()) + 4 + if font.italic_angle() == 0.0 { 0 } else { 64 };
    // The font does not say how thick its upright strokes are; a reader asks only to stand
    // in for a font that is missing, and this one is carried. A rule of thumb from its weight.
    let stem = (10.0 + 220.0 * (f64::from(font.weight()) - 50.0) / 900.0).round();
    let [left, bottom, right, top] = font.bounds();
    let capitals = if font.cap_height() > 0 {
        font.cap_height()
    } else {
        font.ascender()
    };
    // The map from glyphs to characters, a hundred to a block.
    let pairs: Vec<String> = used
        .iter()
        .filter_map(|&glyph| {
            let character = font.character_of(glyph)?;
            let mut units = [0_u16; 2];
            let code: String = character
                .encode_utf16(&mut units)
                .iter()
                .map(|unit| format!("{unit:04X}"))
                .collect();
            Some(format!("<{glyph:04X}> <{code}>\n"))
        })
        .collect();
    let mut map = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    for block in pairs.chunks(100) {
        map.push_str(&format!("{} beginbfchar\n", block.len()));
        for pair in block {
            map.push_str(pair);
        }
        map.push_str("endbfchar\n");
    }
    map.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    vec![
        format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont /{base} /Encoding /Identity-H \
             /DescendantFonts [{} 0 R] /ToUnicode {} 0 R >>",
            first + 1,
            first + 4
        )
        .into_bytes(),
        format!(
            "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{base} \
             /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
             /FontDescriptor {} 0 R /DW 1000 /W [{widths}] /CIDToGIDMap /Identity >>",
            first + 2
        )
        .into_bytes(),
        format!(
            "<< /Type /FontDescriptor /FontName /{base} /Flags {flags} \
             /FontBBox [{} {} {} {}] /ItalicAngle {} /Ascent {} /Descent {} /CapHeight {} \
             /StemV {} /FontFile2 {} 0 R >>",
            em(left),
            em(bottom),
            em(right),
            em(top),
            length(font.italic_angle()),
            em(font.ascender()),
            em(font.descender()),
            em(capitals),
            length(stem),
            first + 3
        )
        .into_bytes(),
        font_file(name, pack),
        stream("", map.as_bytes(), pack),
    ]
}

/// The file: the page and what it draws, the fonts, the shapes and the images, each an
/// object in a fixed order, then where each object stands.
fn assemble(drawing: &Drawing, pack: bool) -> Vec<u8> {
    let mut uses = BTreeMap::new();
    for item in drawing.primitives() {
        if let Primitive::Point { shape, size, .. } = &item.primitive {
            *uses.entry((*shape, length(*size))).or_insert(0) += 1;
        }
    }
    let mut content = Content {
        out: String::new(),
        states: Vec::new(),
        glyphs: [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()],
        uses,
        placed: BTreeMap::new(),
        forms: Vec::new(),
        images: Vec::new(),
    };
    let (width, height) = (length(drawing.width), length(drawing.height));
    // The axes turned over, so that a place is written as the drawing has it; and the
    // corners of lines cut as an SVG reader cuts them.
    content
        .out
        .push_str(&format!("1 0 0 -1 0 {height} cm {MITRE} M\n"));
    content.items(&drawing.items);

    // The numbers of the objects: the catalogue, the pages, the page, what it draws; five
    // for each face used; one for each shape and each image; and what the file says of itself.
    let faces: Vec<usize> = (0..3).filter(|&i| !content.glyphs[i].is_empty()).collect();
    let first_font = 5;
    let first_form = first_font + 5 * faces.len();
    let first_image = first_form + content.forms.len();
    let about = first_image + content.images.len();

    let mut resources = String::new();
    if !content.states.is_empty() {
        resources.push_str("/ExtGState <<");
        for (i, opacity) in content.states.iter().enumerate() {
            resources.push_str(&format!(" /G{} << /ca {opacity} /CA {opacity} >>", i + 1));
        }
        resources.push_str(" >> ");
    }
    if !faces.is_empty() {
        resources.push_str("/Font <<");
        for (i, face) in faces.iter().enumerate() {
            resources.push_str(&format!(" /F{} {} 0 R", face + 1, first_font + 5 * i));
        }
        resources.push_str(" >> ");
    }
    if !content.forms.is_empty() || !content.images.is_empty() {
        resources.push_str("/XObject <<");
        for i in 0..content.forms.len() {
            resources.push_str(&format!(" /M{} {} 0 R", i + 1, first_form + i));
        }
        for i in 0..content.images.len() {
            resources.push_str(&format!(" /I{} {} 0 R", i + 1, first_image + i));
        }
        resources.push_str(" >> ");
    }
    // What is not opaque is blended in the colours of a screen.
    let group = if content.states.is_empty() {
        ""
    } else {
        " /Group << /S /Transparency /CS /DeviceRGB >>"
    };

    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] \
             /Resources << {resources}>> /Contents 4 0 R{group} >>"
        )
        .into_bytes(),
        stream("", content.out.as_bytes(), pack),
    ];
    for (i, &face) in faces.iter().enumerate() {
        objects.extend(font_objects(
            FaceName::ALL[face],
            &content.glyphs[face],
            first_font + 5 * i,
            pack,
        ));
    }
    for form in &content.forms {
        let reach = length(form.reach);
        objects.push(stream(
            &format!(
                "/Type /XObject /Subtype /Form /BBox [-{reach} -{reach} {reach} {reach}] \
                 /Resources << >> "
            ),
            form.ops.as_bytes(),
            pack,
        ));
    }
    for image in &content.images {
        objects.push(stream(
            &format!(
                "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB \
                 /BitsPerComponent 8 /Interpolate false ",
                image.columns, image.rows
            ),
            &image.dots,
            pack,
        ));
    }
    objects.push(b"<< /Producer (Zeff) >>".to_vec());

    // A comment of four bytes above 127 tells a program that moves the file that it is not
    // text.
    let mut file = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(file.len());
        file.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        file.extend_from_slice(body);
        file.extend_from_slice(b"\nendobj\n");
    }
    // The identifier of the file is the start of the hash of what it holds.
    let id = zeff_hash::hash_hex(&file)[..32].to_string();
    let table = file.len();
    file.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        file.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    file.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info {about} 0 R /ID [<{id}> <{id}>] >>\n\
             startxref\n{table}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    file
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::QUARTER;

    #[test]
    fn a_colour_is_three_shares_of_one_and_what_is_no_colour_is_black() {
        assert_eq!(rgb("#E69F00"), [230, 159, 0]);
        assert_eq!(rgb("#ffffff"), [255, 255, 255]);
        assert_eq!(shares("#FFFFFF"), "1 1 1");
        assert_eq!(shares("#222222"), "0.133 0.133 0.133");
        assert_eq!(shares("#E69F00"), "0.902 0.624 0");
        for bad in [
            "", "red", "#12345", "#1234567", "#GG0000", "#é2345", "E69F00",
        ] {
            assert_eq!(rgb(bad), [0, 0, 0], "{bad}");
        }
        // Three decimals give back each of the 256 steps.
        for step in 0..=255_u8 {
            let written: f64 = length(f64::from(step) / 255.0).parse().unwrap();
            assert_eq!((written * 255.0).round() as u8, step);
        }
    }

    #[test]
    fn a_number_of_a_turn_keeps_six_decimals() {
        assert_eq!(fine(1.0), "1");
        assert_eq!(fine(-1.0), "-1");
        assert_eq!(fine(0.0), "0");
        assert_eq!(fine(-0.0), "0");
        assert_eq!(fine(0.5), "0.5");
        assert_eq!(fine(std::f64::consts::FRAC_1_SQRT_2), "0.707107");
        assert_eq!(fine(-0.000_000_4), "0");
        assert_eq!(fine(f64::NAN), "0");
        assert_eq!(fine(f64::INFINITY), "0");
    }

    #[test]
    fn an_opacity_is_a_share_of_one() {
        assert_eq!(share(1.0), "1");
        assert_eq!(share(0.3), "0.3");
        assert_eq!(share(0.0), "0");
        assert_eq!(share(-2.0), "0");
        assert_eq!(share(7.0), "1");
        assert_eq!(share(f64::NAN), "1");
    }

    /// The places a path of curves ends its pieces at, read back from its operators.
    fn ends(ops: &str) -> Vec<(f64, f64)> {
        let words: Vec<&str> = ops.split_whitespace().collect();
        let mut out = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if matches!(*word, "m" | "l" | "c") {
                out.push((words[i - 2].parse().unwrap(), words[i - 1].parse().unwrap()));
            }
        }
        out
    }

    #[test]
    fn a_circle_is_four_quarters_that_stay_on_it() {
        let mut trace = Trace::default();
        trace.circle((10.0, 20.0), 5.0);
        assert_eq!(
            ends(&trace.ops()),
            [
                (15.0, 20.0),
                (10.0, 25.0),
                (5.0, 20.0),
                (10.0, 15.0),
                (15.0, 20.0)
            ]
        );
        assert!(trace.ops().ends_with("h "));
        // Halfway along a quarter the curve is within a thousandth of the radius of the
        // circle: (1 + 3k) / (4 √2) of it from the centre along the diagonal.
        let middle = (4.0 + 3.0 * QUARTER) / 8.0;
        let away = (2.0 * middle * middle).sqrt();
        assert!((away - 1.0).abs() < 3e-4, "{away}");
    }

    #[test]
    fn an_arc_ends_where_it_is_told_and_stays_on_its_ellipse() {
        // A half circle of radius 10 from (0, 0) to (20, 0): two quarters through (10, ±10).
        for (clockwise, through) in [(true, -10.0), (false, 10.0)] {
            let mut trace = Trace::default();
            trace.to((0.0, 0.0));
            trace.arc((10.0, 10.0), 0.0, false, clockwise, (20.0, 0.0));
            assert_eq!(
                ends(&trace.ops()),
                [(0.0, 0.0), (10.0, through), (20.0, 0.0)],
                "{}",
                trace.ops()
            );
        }
        // Radii too small to reach are grown: the same half circle.
        let mut small = Trace::default();
        small.to((0.0, 0.0));
        small.arc((1.0, 1.0), 0.0, false, true, (20.0, 0.0));
        assert_eq!(ends(&small.ops()), [(0.0, 0.0), (10.0, -10.0), (20.0, 0.0)]);
        // The larger of the two arcs between ends a quarter apart is three quarters.
        let mut large = Trace::default();
        large.to((10.0, 0.0));
        large.arc((10.0, 10.0), 0.0, true, true, (0.0, 10.0));
        assert_eq!(ends(&large.ops()).len(), 4, "{}", large.ops());
        let mut little = Trace::default();
        little.to((10.0, 0.0));
        little.arc((10.0, 10.0), 0.0, false, true, (0.0, 10.0));
        assert_eq!(ends(&little.ops()).len(), 2, "{}", little.ops());
        // No radius, a straight line; no way to go, nothing; no start, a start.
        let mut flat = Trace::default();
        flat.to((0.0, 0.0));
        flat.arc((0.0, 5.0), 0.0, false, true, (4.0, 4.0));
        assert_eq!(flat.ops(), "0 0 m 4 4 l ");
        let mut still = Trace::default();
        still.to((3.0, 3.0));
        still.arc((5.0, 5.0), 0.0, false, true, (3.0, 3.0));
        assert_eq!(still.ops(), "3 3 m ");
        let mut unbegun = Trace::default();
        unbegun.arc((5.0, 5.0), 0.0, false, true, (3.0, 3.0));
        assert_eq!(unbegun.ops(), "3 3 m ");
        // A turned ellipse: every end of a piece is on it.
        let mut turned_arc = Trace::default();
        turned_arc.to((10.0, 70.0));
        turned_arc.arc((8.0, 5.0), 30.0, true, false, (16.0, 74.0));
        let all = ends(&turned_arc.ops());
        assert_eq!(all.last(), Some(&(16.0, 74.0)));
        assert!(all.len() >= 3);
    }

    #[test]
    fn a_path_that_does_not_begin_begins_where_it_first_goes() {
        let mut trace = Trace::default();
        trace.close();
        trace.line((1.0, 2.0));
        trace.line((3.0, 4.0));
        trace.close();
        assert_eq!(trace.ops(), "1 2 m 3 4 l h ");
        let mut curve = Trace::default();
        curve.curve((0.0, 0.0), (1.0, 1.0), (2.0, 2.0));
        assert_eq!(curve.ops(), "2 2 m ");
    }

    #[test]
    fn a_rectangle_is_rounded_by_no_more_than_half_its_smaller_side() {
        let mut plain = Trace::default();
        plain.rounded((1.0, 2.0), 10.0, 4.0, 0.0, |p| p);
        assert_eq!(plain.ops(), "1 2 m 11 2 l 11 6 l 1 6 l h ");
        let mut round = Trace::default();
        round.rounded((0.0, 0.0), 10.0, 4.0, 50.0, |p| p);
        // The radius is 2: the top runs from 2 to 8.
        assert!(round.ops().starts_with("2 0 m 8 0 l "), "{}", round.ops());
        let mut through = Trace::default();
        through.rounded((0.0, 0.0), 10.0, 4.0, 0.0, |p| (p.1, -p.0));
        assert_eq!(through.ops(), "0 0 m 0 -10 l 4 -10 l 4 0 l h ");
    }

    #[test]
    fn a_drawing_of_nothing_is_a_page_and_no_more() {
        let empty = Drawing {
            width: 72.0,
            height: 36.5,
            items: Vec::new(),
            notes: Vec::new(),
        };
        let plain = String::from_utf8_lossy(&write_plain(&empty)).into_owned();
        assert!(plain.starts_with("%PDF-1.4\n"));
        assert!(plain.contains("/MediaBox [0 0 72 36.5] /Resources << >> /Contents 4 0 R >>"));
        assert!(plain.contains("stream\n1 0 0 -1 0 36.5 cm 4 M\n\nendstream"));
        assert!(plain.contains("0 6\n0000000000 65535 f \n"));
        assert!(plain.ends_with("%%EOF\n"));
        assert_eq!(write(&empty), write(&empty));
        assert_ne!(write(&empty), write_plain(&empty));
    }
}
