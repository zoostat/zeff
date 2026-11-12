//! The SVG of a drawing (Language Reference, section 7.6, "Rendering and export"; release
//! 0.7.0, step 21c; open decisions #118 and #126): the bytes of a figure, the same on every
//! machine.
//!
//! [`write`] turns a drawing ([`crate::drawing`]) into the text of an SVG file and is a pure
//! function of it. Every primitive of the closed set is drawn, and nothing else is written:
//! no date, no path of a file, no name of a machine, no identifier made at random. A length
//! is written by the one rule of [`crate::drawing::length`], to a thousandth of a point. The
//! page is as large as the drawing, in points.
//!
//! **Text** (open decision #126, decided 12 November 2026). A text is drawn as the outlines
//! of its letters, taken from the embedded fonts ([`crate::font`]) and written as paths at
//! their places on the page, so that the figure looks the same in every program whether or
//! not the font is installed there. The words are kept: each text is a group labelled with
//! them, and holds them once more as a text that is not painted, which a search finds and
//! which carries the size of the type. Every outline is written out where it stands; nothing
//! in the file refers to another part of it but a clip region.
//!
//! **Marks.** A point is drawn by the geometry of its shape ([`crate::shape::outline`]): a
//! circle, a closed outline or strokes. Neighbouring points of one colour, opacity and kind
//! share a group that carries the paint, so that a layer of many points is a short line for
//! each; a point with no such neighbour carries its own. An outline shape (`_open`) and a
//! shape of lines are stroked [`MARK_STROKE`] wide.
//!
//! **Names.** Neighbouring items of one name are a group whose `id` is the name, with the
//! characters an identifier cannot hold replaced (`axis:x` is `axis-x`); a name met again
//! later gets a number after it. A drawing holds no row of the data, and so no file does.

use zeff_numeric::maths::{cos, sin};

use crate::drawing::{Anchor, Drawing, Head, Item, Primitive, Span, Step, Stroke, TextBox, length};
use crate::font::{Segment, face};
use crate::shape::{self, Geometry};

/// The width of the stroke of an outline shape and of a shape of lines, in points.
pub const MARK_STROKE: f64 = 0.9;

/// The share of the size of the type that a line of text takes above its baseline and below
/// it, for the box behind a text.
pub(crate) const ABOVE: f64 = 0.8;
pub(crate) const BELOW: f64 = 0.2;

/// The SVG file of `drawing`, as text in UTF-8.
pub fn write(drawing: &Drawing) -> String {
    let mut writer = Writer {
        out: String::new(),
        clips: 0,
        ids: Vec::new(),
    };
    let (width, height) = (length(drawing.width), length(drawing.height));
    writer
        .out
        .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    writer.out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}pt\" height=\"{height}pt\" viewBox=\"0 0 {width} {height}\">\n"
    ));
    writer.items(&drawing.items);
    writer.out.push_str("</svg>\n");
    writer.out
}

/// Text as the content of an element or the value of an attribute.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // XML has no way to hold most control characters; a tab and a line break it
            // has, and the others are left out.
            '\t' | '\n' | '\r' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// The cosine and the sine of a turn in degrees, exact for the quarter turns.
pub(crate) fn turned(degrees: f64) -> (f64, f64) {
    let within = degrees.rem_euclid(360.0);
    if within == 0.0 {
        (1.0, 0.0)
    } else if within == 90.0 {
        (0.0, 1.0)
    } else if within == 180.0 {
        (-1.0, 0.0)
    } else if within == 270.0 {
        (0.0, -1.0)
    } else {
        let radians = within * std::f64::consts::PI / 180.0;
        (cos(radians), sin(radians))
    }
}

fn paint(fill: &Option<String>, stroke: &Option<Stroke>, opacity: f64) -> String {
    let mut out = match fill {
        Some(colour) => format!(" fill=\"{colour}\""),
        None => " fill=\"none\"".to_string(),
    };
    if let Some(line) = stroke {
        out.push_str(&format!(
            " stroke=\"{}\" stroke-width=\"{}\"",
            line.colour,
            length(line.width)
        ));
    }
    if opacity != 1.0 {
        if fill.is_some() {
            out.push_str(&format!(" fill-opacity=\"{}\"", length(opacity)));
        }
        if stroke.is_some() {
            out.push_str(&format!(" stroke-opacity=\"{}\"", length(opacity)));
        }
    }
    out
}

/// How a point is painted: filled, or stroked for an outline shape and a shape of lines.
#[derive(PartialEq)]
struct MarkPaint<'a> {
    colour: &'a str,
    opacity: f64,
    stroked: bool,
}

/// The geometry of a point of this shape and size; a name that is no shape is drawn as the
/// default shape.
pub(crate) fn geometry(name: &str, size: f64) -> shape::Outline {
    shape::outline(name, size)
        .or_else(|| shape::outline(shape::DEFAULT, size))
        .unwrap_or(shape::Outline {
            geometry: Geometry::Circle(size / 2.0),
            open: false,
        })
}

struct Writer {
    out: String,
    clips: usize,
    ids: Vec<String>,
}

impl Writer {
    /// An identifier for a name, not given before: letters, digits, `-` and `_`, starting
    /// with a letter.
    fn id(&mut self, name: &str) -> String {
        let mut id: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        if !id.starts_with(|c: char| c.is_ascii_alphabetic()) {
            id.insert_str(0, "g-");
        }
        let mut unique = id.clone();
        let mut again = 1;
        while self.ids.contains(&unique) {
            again += 1;
            unique = format!("{id}-{again}");
        }
        self.ids.push(unique.clone());
        unique
    }

    /// Items, the neighbours of one name in one named group.
    fn items(&mut self, items: &[Item]) {
        let mut start = 0;
        while start < items.len() {
            let name = &items[start].name;
            let mut end = start;
            while end < items.len() && items[end].name == *name {
                end += 1;
            }
            if let Some(name) = name {
                let id = self.id(name);
                self.out.push_str(&format!("<g id=\"{id}\">\n"));
            }
            self.run(&items[start..end]);
            if name.is_some() {
                self.out.push_str("</g>\n");
            }
            start = end;
        }
    }

    /// Items of one name: the points among them in groups that carry their paint, a point
    /// that has no neighbour of its paint carrying its own.
    fn run(&mut self, items: &[Item]) {
        // The paint of the point at `at`, with its geometry; `None` for another primitive.
        let painted = |at: usize| match items.get(at).map(|item| &item.primitive) {
            Some(Primitive::Point {
                at: centre,
                shape: name,
                size,
                colour,
                opacity,
            }) => {
                let outline = geometry(name, *size);
                let paint = MarkPaint {
                    colour,
                    opacity: *opacity,
                    stroked: outline.open || matches!(outline.geometry, Geometry::Lines(_)),
                };
                Some((paint, *centre, outline.geometry))
            }
            _ => None,
        };
        let mut at = 0;
        while at < items.len() {
            let Some((first, centre, shape)) = painted(at) else {
                self.primitive(&items[at].primitive);
                at += 1;
                continue;
            };
            let attributes = if first.stroked {
                let line = Some(Stroke {
                    colour: first.colour.to_string(),
                    width: MARK_STROKE,
                });
                paint(&None, &line, first.opacity)
            } else {
                paint(&Some(first.colour.to_string()), &None, first.opacity)
            };
            let mut end = at + 1;
            while painted(end).is_some_and(|(next, _, _)| next == first) {
                end += 1;
            }
            if end == at + 1 {
                self.mark(centre, &shape, &attributes);
            } else {
                self.out.push_str(&format!("<g{attributes}>\n"));
                for each in at..end {
                    if let Some((_, centre, shape)) = painted(each) {
                        self.mark(centre, &shape, "");
                    }
                }
                self.out.push_str("</g>\n");
            }
            at = end;
        }
    }

    /// One mark at `centre`, with `attributes` of its own when it stands outside a group.
    fn mark(&mut self, centre: (f64, f64), geometry: &Geometry, attributes: &str) {
        let (x, y) = centre;
        match geometry {
            Geometry::Circle(radius) => self.out.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"{attributes}/>\n",
                length(x),
                length(y),
                length(*radius)
            )),
            Geometry::Polygon(corners) => {
                let mut d = String::new();
                for (i, corner) in corners.iter().enumerate() {
                    d.push(if i == 0 { 'M' } else { 'L' });
                    d.push_str(&format!(
                        "{} {}",
                        length(x + corner.0),
                        length(y + corner.1)
                    ));
                }
                self.out
                    .push_str(&format!("<path d=\"{d}Z\"{attributes}/>\n"));
            }
            Geometry::Lines(strokes) => {
                let mut d = String::new();
                for (from, to) in strokes {
                    d.push_str(&format!(
                        "M{} {}L{} {}",
                        length(x + from.0),
                        length(y + from.1),
                        length(x + to.0),
                        length(y + to.1)
                    ));
                }
                self.out
                    .push_str(&format!("<path d=\"{d}\"{attributes}/>\n"));
            }
        }
    }

    fn primitive(&mut self, primitive: &Primitive) {
        match primitive {
            // A point alone is a run of one, drawn by `run`.
            Primitive::Point { .. } => {}
            Primitive::Path {
                steps,
                stroke,
                fill,
                opacity,
            } => {
                let mut d = String::new();
                for step in steps {
                    match step {
                        Step::Move(to) => {
                            d.push_str(&format!("M{} {}", length(to.0), length(to.1)))
                        }
                        Step::Line(to) => {
                            d.push_str(&format!("L{} {}", length(to.0), length(to.1)))
                        }
                        Step::Curve(a, b, to) => d.push_str(&format!(
                            "C{} {} {} {} {} {}",
                            length(a.0),
                            length(a.1),
                            length(b.0),
                            length(b.1),
                            length(to.0),
                            length(to.1)
                        )),
                        Step::Arc {
                            radii,
                            turn,
                            larger,
                            clockwise,
                            to,
                        } => d.push_str(&format!(
                            "A{} {} {} {} {} {} {}",
                            length(radii.0),
                            length(radii.1),
                            length(*turn),
                            u8::from(*larger),
                            u8::from(*clockwise),
                            length(to.0),
                            length(to.1)
                        )),
                        Step::Close => d.push('Z'),
                    }
                }
                self.out.push_str(&format!(
                    "<path d=\"{d}\"{}/>\n",
                    paint(fill, stroke, *opacity)
                ));
            }
            Primitive::Polygon {
                rings,
                fill,
                stroke,
                opacity,
            } => {
                let mut d = String::new();
                for ring in rings {
                    for (i, corner) in ring.iter().enumerate() {
                        d.push(if i == 0 { 'M' } else { 'L' });
                        d.push_str(&format!("{} {}", length(corner.0), length(corner.1)));
                    }
                    if !ring.is_empty() {
                        d.push('Z');
                    }
                }
                // What lies inside a hole is outside the polygon.
                self.out.push_str(&format!(
                    "<path d=\"{d}\" fill-rule=\"evenodd\"{}/>\n",
                    paint(fill, stroke, *opacity)
                ));
            }
            Primitive::Rectangle {
                at,
                width,
                height,
                radius,
                fill,
                stroke,
            } => {
                let round = if *radius > 0.0 {
                    format!(" rx=\"{}\"", length(*radius))
                } else {
                    String::new()
                };
                self.out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{round}{}/>\n",
                    length(at.0),
                    length(at.1),
                    length(*width),
                    length(*height),
                    paint(fill, stroke, 1.0)
                ));
            }
            Primitive::Segment {
                from,
                to,
                stroke,
                heads,
            } => {
                self.out.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>\n",
                    length(from.0),
                    length(from.1),
                    length(to.0),
                    length(to.1),
                    stroke.colour,
                    length(stroke.width)
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
                // Cells that meet are drawn without the soft edge that would show between them.
                self.out.push_str("<g shape-rendering=\"crispEdges\">\n");
                for row in 0..*rows {
                    let mut column = 0;
                    while column < *columns {
                        let Some(colour) = cells.get(row * columns + column) else {
                            break;
                        };
                        // Neighbours of one colour in a row are one rectangle.
                        let mut end = column + 1;
                        while end < *columns && cells.get(row * columns + end) == Some(colour) {
                            end += 1;
                        }
                        self.out.push_str(&format!(
                            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{colour}\"/>\n",
                            length(at.0 + column as f64 * wide),
                            length(at.1 + row as f64 * high),
                            length((end - column) as f64 * wide),
                            length(high)
                        ));
                        column = end;
                    }
                }
                self.out.push_str("</g>\n");
            }
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } => {
                self.clips += 1;
                let id = format!("clip-{}", self.clips);
                self.ids.push(id.clone());
                self.out.push_str(&format!(
                    "<clipPath id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>\n<g clip-path=\"url(#{id})\">\n",
                    length(at.0),
                    length(at.1),
                    length(*width),
                    length(*height)
                ));
                self.items(items);
                self.out.push_str("</g>\n");
            }
        }
    }

    /// The head of an arrow at `tip`, on a line that comes from `tail`: a filled triangle
    /// six strokes long, and at least four points.
    fn arrow(&mut self, tail: (f64, f64), tip: (f64, f64), stroke: &Stroke) {
        let (dx, dy) = (tip.0 - tail.0, tip.1 - tail.1);
        let run = (dx * dx + dy * dy).sqrt();
        // A segment of no length, or to a place that is no number, has no direction.
        if run.is_nan() || run <= 0.0 {
            return;
        }
        let (ux, uy) = (dx / run, dy / run);
        let long = (6.0 * stroke.width).max(4.0);
        let half = 0.35 * long;
        let base = (tip.0 - ux * long, tip.1 - uy * long);
        self.out.push_str(&format!(
            "<path d=\"M{} {}L{} {}L{} {}Z\" fill=\"{}\"/>\n",
            length(tip.0),
            length(tip.1),
            length(base.0 - uy * half),
            length(base.1 + ux * half),
            length(base.0 + uy * half),
            length(base.1 - ux * half),
            stroke.colour
        ));
    }

    /// A line of text: the box behind it, the outlines of its letters, and its words.
    fn text(
        &mut self,
        at: (f64, f64),
        anchor: Anchor,
        turn: f64,
        spans: &[Span],
        frame: Option<&TextBox>,
    ) {
        let words: String = spans.iter().map(|span| span.text.as_str()).collect();
        if words.is_empty() {
            return;
        }
        // Each span set by its face, one after the other along the baseline.
        let set: Vec<_> = spans
            .iter()
            .map(|span| {
                let font = face(span.face);
                let scale = span.size / f64::from(font.units_per_em());
                let shaped = font.shape(&span.text);
                (span, font, scale, f64::from(shaped.width) * scale, shaped)
            })
            .collect();
        let total: f64 = set.iter().map(|(_, _, _, width, _)| width).sum();
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
        let rotate = if turn == 0.0 {
            String::new()
        } else {
            format!(
                " transform=\"rotate({} {} {})\"",
                length(-turn),
                length(at.0),
                length(at.1)
            )
        };
        self.out
            .push_str(&format!("<g aria-label=\"{}\">\n", escape(&words)));
        if let Some(frame) = frame {
            let round = if frame.radius > 0.0 {
                format!(" rx=\"{}\"", length(frame.radius))
            } else {
                String::new()
            };
            self.out.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{round}{}{rotate}/>\n",
                length(at.0 + start - frame.padding),
                length(at.1 - ABOVE * largest - frame.padding),
                length(total + 2.0 * frame.padding),
                length((ABOVE + BELOW) * largest + 2.0 * frame.padding),
                paint(&frame.fill, &frame.border, 1.0)
            ));
        }
        let mut along = start;
        for (span, font, scale, width, shaped) in &set {
            let mut d = String::new();
            for placed in &shaped.glyphs {
                let origin = along + f64::from(placed.x) * scale;
                let point = |x: f64, y: f64| {
                    let (px, py) = place(origin + x * scale, y * scale);
                    format!("{} {}", length(px), length(py))
                };
                // A glyph whose outline cannot be read is left out; the embedded faces
                // have none.
                for step in font.outline(placed.glyph).unwrap_or_default() {
                    match step {
                        Segment::Move(x, y) => d.push_str(&format!("M{}", point(x, y))),
                        Segment::Line(x, y) => d.push_str(&format!("L{}", point(x, y))),
                        Segment::Curve(control, to) => d.push_str(&format!(
                            "Q{} {}",
                            point(control.0, control.1),
                            point(to.0, to.1)
                        )),
                        Segment::Close => d.push('Z'),
                    }
                }
            }
            if !d.is_empty() {
                self.out
                    .push_str(&format!("<path d=\"{d}\" fill=\"{}\"/>\n", span.colour));
            }
            along += width;
        }
        self.out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"{}\" fill=\"none\"{rotate} aria-hidden=\"true\">{}</text>\n</g>\n",
            length(at.0),
            length(at.1),
            length(largest),
            match anchor {
                Anchor::Start => "start",
                Anchor::Middle => "middle",
                Anchor::End => "end",
            },
            escape(&words)
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_xml_reads_as_markup_is_escaped_and_what_it_cannot_hold_is_left_out() {
        assert_eq!(
            escape("a < b & c > \"d\""),
            "a &lt; b &amp; c &gt; &quot;d&quot;"
        );
        assert_eq!(escape("réponse β 中"), "réponse β 中");
        assert_eq!(escape("a\tb\nc\rd"), "a b c d");
        assert_eq!(escape("bell\u{7} nul\u{0} escape\u{1b}"), "bell nul escape");
        assert_eq!(escape(""), "");
    }

    #[test]
    fn a_quarter_turn_is_exact_and_any_other_turn_is_computed() {
        assert_eq!(turned(0.0), (1.0, 0.0));
        assert_eq!(turned(90.0), (0.0, 1.0));
        assert_eq!(turned(180.0), (-1.0, 0.0));
        assert_eq!(turned(270.0), (0.0, -1.0));
        assert_eq!(turned(-90.0), (0.0, -1.0));
        assert_eq!(turned(450.0), (0.0, 1.0));
        let (cosine, sine) = turned(45.0);
        assert!(
            (cosine - sine).abs() < 1e-15 && (cosine * cosine + sine * sine - 1.0).abs() < 1e-15
        );
        let (cosine, sine) = turned(30.0);
        assert!((sine - 0.5).abs() < 1e-15 && cosine > 0.86);
    }

    #[test]
    fn a_name_becomes_an_identifier_not_given_before() {
        let mut writer = Writer {
            out: String::new(),
            clips: 0,
            ids: Vec::new(),
        };
        assert_eq!(writer.id("axis:x"), "axis-x");
        assert_eq!(writer.id("legend:color+shape"), "legend-color-shape");
        assert_eq!(writer.id("axis:x"), "axis-x-2");
        assert_eq!(writer.id("axis:x"), "axis-x-3");
        assert_eq!(writer.id("axis-x"), "axis-x-4");
        assert_eq!(writer.id("2nd layer"), "g-2nd-layer");
        assert_eq!(writer.id(""), "g-");
        assert_eq!(writer.id("node:engagé"), "node-engag-");
    }

    #[test]
    fn a_drawing_of_nothing_is_a_page_and_no_more() {
        let empty = Drawing {
            width: 72.0,
            height: 36.5,
            items: Vec::new(),
            notes: Vec::new(),
        };
        assert_eq!(
            write(&empty),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"72pt\" height=\"36.5pt\" viewBox=\"0 0 72 36.5\">\n</svg>\n"
        );
    }
}
