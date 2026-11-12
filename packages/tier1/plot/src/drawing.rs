//! A drawing (Language Reference, section 7.6, "Rendering and export"; release 0.7.0, step
//! 21b; open decision #118, F1 and F2): a figure of a fixed size as a list of primitives in
//! points of 1/72 inch, the origin at the top left and the y axis downwards, as on a page.
//!
//! The primitives are a closed set, and every writer of figures draws all of them and nothing
//! else: a point with a shape; a path of lines, cubic curves and arcs of ellipses; a polygon
//! with holes; a rectangle; a segment with arrow heads; a run of text in styled spans, with an
//! optional box behind it; a grid of coloured cells; and a region that clips what is inside
//! it. Every mark of this release and of the later ones, every diagram and every map is drawn
//! with these, so that no later release needs a writer to learn a new one. There are no
//! gradients, no images from files and no blur.
//!
//! An item may carry a name, such as `axis:x` or `layer:1`: neighbouring items of one name
//! are one named group in an exported file. A name tells what a shape is, never which rows of
//! the data it shows: a drawing holds no row and no binding (F2), only places, sizes and
//! colours.
//!
//! [`crate::layout::layout`] makes the drawing of a scene at a size.

use crate::font::FaceName;

/// A place on the page, in points.
pub type Place = (f64, f64);

/// The type of a line (decision #118, F9): whole, or ink and gaps by turns. There are six, in
/// the order a scale gives them to the categories of a variable, chosen so that each can be
/// told from the others in black on white, whatever the colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineType {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
    LongDash,
    DashDotDot,
}

impl LineType {
    /// The six, in their order.
    pub const ALL: [LineType; 6] = [
        LineType::Solid,
        LineType::Dashed,
        LineType::Dotted,
        LineType::DashDot,
        LineType::LongDash,
        LineType::DashDotDot,
    ];

    /// The type as it is written: `solid`, `dashed`, `dotted`, `dash_dot`, `long_dash`,
    /// `dash_dot_dot`.
    pub fn name(self) -> &'static str {
        match self {
            LineType::Solid => "solid",
            LineType::Dashed => "dashed",
            LineType::Dotted => "dotted",
            LineType::DashDot => "dash_dot",
            LineType::LongDash => "long_dash",
            LineType::DashDotDot => "dash_dot_dot",
        }
    }

    /// The type of a name; `None` for a name that is none of the six.
    pub fn parse(name: &str) -> Option<LineType> {
        LineType::ALL.into_iter().find(|each| each.name() == name)
    }

    /// The lengths of ink and of gap by turns, beginning with ink, in widths of the line, so
    /// that a thicker line has longer dashes; nothing for a whole line. A dot is as long as
    /// the line is wide.
    pub fn pattern(self) -> &'static [f64] {
        match self {
            LineType::Solid => &[],
            LineType::Dashed => &[4.0, 3.0],
            LineType::Dotted => &[1.0, 2.0],
            LineType::DashDot => &[4.0, 2.0, 1.0, 2.0],
            LineType::LongDash => &[8.0, 3.0],
            LineType::DashDotDot => &[4.0, 2.0, 1.0, 2.0, 1.0, 2.0],
        }
    }

    /// The lengths of ink and of gap of a line `width` wide, in the unit of the width.
    pub fn lengths(self, width: f64) -> Vec<f64> {
        self.pattern().iter().map(|part| part * width).collect()
    }
}

/// A line: its colour, `#RRGGBB`, its width in points, and its type. The ink of a line that
/// is not whole begins where each part of the path begins, and its ends are cut square.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub colour: String,
    pub width: f64,
    pub dash: LineType,
}

/// Where a text stands against its place: starting there, centred on it or ending there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

/// A stretch of text in one face, size and colour.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub face: FaceName,
    /// The size of the type, in points.
    pub size: f64,
    pub colour: String,
}

/// The box behind a text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBox {
    pub padding: f64,
    pub radius: f64,
    pub fill: Option<String>,
    pub border: Option<Stroke>,
}

/// One step of a path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Move(Place),
    Line(Place),
    /// Two controls and the end.
    Curve(Place, Place, Place),
    /// An arc of an ellipse to `to`: its radii, its turn in degrees, and which of the four
    /// arcs between the two ends, the larger or the smaller, clockwise or not.
    Arc {
        radii: (f64, f64),
        turn: f64,
        larger: bool,
        clockwise: bool,
        to: Place,
    },
    Close,
}

/// The end of a segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    Plain,
    Arrow,
}

/// What can be drawn.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// A point: a shape by its name ([`crate::shape`]) at a size, centred at a place.
    Point {
        at: Place,
        shape: &'static str,
        size: f64,
        colour: String,
        opacity: f64,
    },
    Path {
        steps: Vec<Step>,
        stroke: Option<Stroke>,
        fill: Option<String>,
        opacity: f64,
    },
    /// A polygon: its outer ring, then its holes.
    Polygon {
        rings: Vec<Vec<Place>>,
        fill: Option<String>,
        stroke: Option<Stroke>,
        opacity: f64,
    },
    Rectangle {
        at: Place,
        width: f64,
        height: f64,
        radius: f64,
        fill: Option<String>,
        stroke: Option<Stroke>,
    },
    Segment {
        from: Place,
        to: Place,
        stroke: Stroke,
        heads: (Head, Head),
    },
    /// A line of text on a baseline through `at`, turned by `turn` degrees about it
    /// (anticlockwise positive: 90 reads from the bottom of the page upwards).
    Text {
        at: Place,
        anchor: Anchor,
        turn: f64,
        spans: Vec<Span>,
        frame: Option<TextBox>,
    },
    /// A grid of cells of one size, row by row from the top left, each a colour: its red,
    /// green and blue, each of 255. A picture in dots is such a grid, a cell to a dot.
    Raster {
        at: Place,
        width: f64,
        height: f64,
        columns: usize,
        rows: usize,
        cells: Vec<[u8; 3]>,
    },
    /// A region outside which what it holds is not drawn.
    Clip {
        at: Place,
        width: f64,
        height: f64,
        items: Vec<Item>,
    },
}

/// A primitive, with the name of what it is, if it has one.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: Option<String>,
    pub primitive: Primitive,
}

/// A figure at a size.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawing {
    /// The width and height of the page, in points.
    pub width: f64,
    pub height: f64,
    /// What is drawn, from the bottom up.
    pub items: Vec<Item>,
    /// What the reader must be told and the figure does not show: the notes of the scene,
    /// and the characters no font of the figure has.
    pub notes: Vec<String>,
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

/// A colour as it is written, `#RRGGBB`.
pub fn hex(colour: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", colour[0], colour[1], colour[2])
}

/// A length as a drawing is read as text: to a thousandth of a point, without trailing
/// zeros, and `0` for either zero.
pub fn length(x: f64) -> String {
    let thousandths = (x * 1000.0).round();
    if thousandths == 0.0 {
        return "0".to_string();
    }
    let sign = if thousandths < 0.0 { "-" } else { "" };
    let whole = thousandths.abs() as u64;
    let (units, part) = (whole / 1000, whole % 1000);
    if part == 0 {
        format!("{sign}{units}")
    } else {
        let digits = format!("{part:03}");
        format!("{sign}{units}.{}", digits.trim_end_matches('0'))
    }
}

fn place(at: Place) -> String {
    format!("{},{}", length(at.0), length(at.1))
}

fn stroke(stroke: &Option<Stroke>) -> String {
    match stroke {
        Some(s) if s.dash == LineType::Solid => {
            format!(" stroke {} {}", s.colour, length(s.width))
        }
        Some(s) => format!(" stroke {} {} {}", s.colour, length(s.width), s.dash.name()),
        None => String::new(),
    }
}

fn fill(fill: &Option<String>) -> String {
    match fill {
        Some(colour) => format!(" fill {colour}"),
        None => String::new(),
    }
}

impl Drawing {
    /// The drawing read as text, one line for each primitive, what a clip holds indented
    /// under it: the form the tests keep.
    pub fn describe(&self) -> String {
        let mut out = format!(
            "drawing {} by {}\n",
            length(self.width),
            length(self.height)
        );
        describe_items(&self.items, 0, &mut out);
        for note in &self.notes {
            out.push_str(&format!("note: {note}\n"));
        }
        out
    }

    /// Every primitive, those inside a clip included, from the bottom up.
    pub fn primitives(&self) -> Vec<&Item> {
        fn gather<'a>(items: &'a [Item], out: &mut Vec<&'a Item>) {
            for item in items {
                out.push(item);
                if let Primitive::Clip { items, .. } = &item.primitive {
                    gather(items, out);
                }
            }
        }
        let mut all = Vec::new();
        gather(&self.items, &mut all);
        all
    }
}

fn describe_items(items: &[Item], depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    for item in items {
        let name = item
            .name
            .as_ref()
            .map_or(String::new(), |name| format!("[{name}] "));
        let line = match &item.primitive {
            Primitive::Point {
                at,
                shape,
                size,
                colour,
                opacity,
            } => format!(
                "point {} {shape} {} {colour} opacity {}",
                place(*at),
                length(*size),
                length(*opacity)
            ),
            Primitive::Path {
                steps,
                stroke: line,
                fill: inside,
                opacity,
            } => {
                let steps: Vec<String> = steps
                    .iter()
                    .map(|step| match step {
                        Step::Move(to) => format!("M {}", place(*to)),
                        Step::Line(to) => format!("L {}", place(*to)),
                        Step::Curve(a, b, to) => {
                            format!("C {} {} {}", place(*a), place(*b), place(*to))
                        }
                        Step::Arc {
                            radii,
                            turn,
                            larger,
                            clockwise,
                            to,
                        } => format!(
                            "A {} {} {} {} {}",
                            place(*radii),
                            length(*turn),
                            u8::from(*larger),
                            u8::from(*clockwise),
                            place(*to)
                        ),
                        Step::Close => "Z".to_string(),
                    })
                    .collect();
                format!(
                    "path {}{}{} opacity {}",
                    steps.join(" "),
                    stroke(line),
                    fill(inside),
                    length(*opacity)
                )
            }
            Primitive::Polygon {
                rings,
                fill: inside,
                stroke: line,
                opacity,
            } => {
                let rings: Vec<String> = rings
                    .iter()
                    .map(|ring| {
                        ring.iter()
                            .map(|corner| place(*corner))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect();
                format!(
                    "polygon {}{}{} opacity {}",
                    rings.join(" | "),
                    fill(inside),
                    stroke(line),
                    length(*opacity)
                )
            }
            Primitive::Rectangle {
                at,
                width,
                height,
                radius,
                fill: inside,
                stroke: line,
            } => format!(
                "rectangle {} {} by {}{}{}{}",
                place(*at),
                length(*width),
                length(*height),
                if *radius > 0.0 {
                    format!(" radius {}", length(*radius))
                } else {
                    String::new()
                },
                fill(inside),
                stroke(line)
            ),
            Primitive::Segment {
                from,
                to,
                stroke: line,
                heads,
            } => format!(
                "segment {} to {} stroke {} {}{}{}",
                place(*from),
                place(*to),
                line.colour,
                length(line.width),
                if line.dash == LineType::Solid {
                    String::new()
                } else {
                    format!(" {}", line.dash.name())
                },
                match heads {
                    (Head::Plain, Head::Plain) => "",
                    (Head::Arrow, Head::Plain) => " arrow at the start",
                    (Head::Plain, Head::Arrow) => " arrow at the end",
                    (Head::Arrow, Head::Arrow) => " arrows at both ends",
                }
            ),
            Primitive::Text {
                at,
                anchor,
                turn,
                spans,
                frame,
            } => {
                let spans: Vec<String> = spans
                    .iter()
                    .map(|span| {
                        format!(
                            "{:?} {} {} {}",
                            span.text,
                            match span.face {
                                FaceName::Sans => "sans",
                                FaceName::SansSemiBold => "semibold",
                                FaceName::Mono => "mono",
                            },
                            length(span.size),
                            span.colour
                        )
                    })
                    .collect();
                format!(
                    "text {} {}{}{} {}",
                    place(*at),
                    match anchor {
                        Anchor::Start => "start",
                        Anchor::Middle => "middle",
                        Anchor::End => "end",
                    },
                    if *turn != 0.0 {
                        format!(" turned {}", length(*turn))
                    } else {
                        String::new()
                    },
                    match frame {
                        Some(frame) => format!(
                            " boxed {} {}{}{}",
                            length(frame.padding),
                            length(frame.radius),
                            fill(&frame.fill),
                            stroke(&frame.border)
                        ),
                        None => String::new(),
                    },
                    spans.join(" + ")
                )
            }
            Primitive::Raster {
                at,
                width,
                height,
                columns,
                rows,
                cells,
            } => format!(
                "raster {} {} by {} cells {columns} by {rows} from {} to {}",
                place(*at),
                length(*width),
                length(*height),
                cells.first().map_or("none".to_string(), |c| hex(*c)),
                cells.last().map_or("none".to_string(), |c| hex(*c))
            ),
            Primitive::Clip {
                at, width, height, ..
            } => format!(
                "clip {} {} by {}",
                place(*at),
                length(*width),
                length(*height)
            ),
        };
        out.push_str(&format!("{indent}{name}{line}\n"));
        if let Primitive::Clip { items, .. } = &item.primitive {
            describe_items(items, depth + 1, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_types_of_a_line_have_names_and_begin_with_ink() {
        let names: Vec<&str> = LineType::ALL.iter().map(|each| each.name()).collect();
        assert_eq!(
            names,
            [
                "solid",
                "dashed",
                "dotted",
                "dash_dot",
                "long_dash",
                "dash_dot_dot"
            ]
        );
        for each in LineType::ALL {
            assert_eq!(LineType::parse(each.name()), Some(each));
            // Ink and gap by turns: an even number of lengths, each above zero.
            let pattern = each.pattern();
            assert_eq!(pattern.len() % 2, 0);
            assert!(pattern.iter().all(|part| *part > 0.0));
            // No two types have the same lengths.
            for other in LineType::ALL {
                assert_eq!(each.pattern() == other.pattern(), each == other);
            }
        }
        assert_eq!(LineType::default(), LineType::Solid);
        assert!(LineType::Solid.pattern().is_empty());
        assert_eq!(LineType::parse("Dashed"), None);
        assert_eq!(LineType::parse("dash"), None);
        // A thicker line has longer dashes: the lengths are in widths of the line.
        assert_eq!(LineType::Dashed.lengths(0.5), [2.0, 1.5]);
        assert_eq!(
            LineType::DashDotDot.lengths(2.0),
            [8.0, 4.0, 2.0, 4.0, 2.0, 4.0]
        );
        assert!(LineType::Solid.lengths(3.0).is_empty());
    }

    #[test]
    fn a_length_is_written_to_a_thousandth_of_a_point() {
        assert_eq!(length(0.0), "0");
        assert_eq!(length(-0.0), "0");
        assert_eq!(length(0.0004), "0");
        assert_eq!(length(-0.0004), "0");
        assert_eq!(length(12.0), "12");
        assert_eq!(length(12.5), "12.5");
        assert_eq!(length(12.3456), "12.346");
        assert_eq!(length(0.05), "0.05");
        assert_eq!(length(-3.2), "-3.2");
        assert_eq!(length(252.283_464_566_929_1), "252.283");
        assert_eq!(length(1.9996), "2");
    }

    #[test]
    fn every_primitive_reads_as_one_line_and_a_clip_holds_its_own() {
        let line = Stroke {
            colour: "#222222".to_string(),
            width: 0.5,
            dash: LineType::Solid,
        };
        let named = |name: &str, primitive: Primitive| Item {
            name: Some(name.to_string()),
            primitive,
        };
        let drawing = Drawing {
            width: 100.0,
            height: 50.0,
            items: vec![
                named(
                    "background",
                    Primitive::Rectangle {
                        at: (0.0, 0.0),
                        width: 100.0,
                        height: 50.0,
                        radius: 0.0,
                        fill: Some("#FFFFFF".to_string()),
                        stroke: None,
                    },
                ),
                named(
                    "panel",
                    Primitive::Clip {
                        at: (10.0, 5.0),
                        width: 80.0,
                        height: 40.0,
                        items: vec![
                            Item {
                                name: None,
                                primitive: Primitive::Point {
                                    at: (20.0, 15.5),
                                    shape: "star",
                                    size: 4.5,
                                    colour: "#E69F00".to_string(),
                                    opacity: 0.5,
                                },
                            },
                            Item {
                                name: None,
                                primitive: Primitive::Path {
                                    steps: vec![
                                        Step::Move((1.0, 2.0)),
                                        Step::Line((3.0, 4.0)),
                                        Step::Curve((5.0, 6.0), (7.0, 8.0), (9.0, 10.0)),
                                        Step::Arc {
                                            radii: (2.0, 3.0),
                                            turn: 0.0,
                                            larger: false,
                                            clockwise: true,
                                            to: (11.0, 12.0),
                                        },
                                        Step::Close,
                                    ],
                                    stroke: Some(line.clone()),
                                    fill: None,
                                    opacity: 1.0,
                                },
                            },
                            Item {
                                name: None,
                                primitive: Primitive::Polygon {
                                    rings: vec![
                                        vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0)],
                                        vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0)],
                                    ],
                                    fill: Some("#56B4E9".to_string()),
                                    stroke: None,
                                    opacity: 1.0,
                                },
                            },
                        ],
                    },
                ),
                named(
                    "axis:x",
                    Primitive::Segment {
                        from: (10.0, 45.0),
                        to: (90.0, 45.0),
                        stroke: line.clone(),
                        heads: (Head::Plain, Head::Arrow),
                    },
                ),
                named(
                    "axis:x",
                    Primitive::Text {
                        at: (50.0, 49.0),
                        anchor: Anchor::Middle,
                        turn: 90.0,
                        spans: vec![Span {
                            text: "Hourly \"wage\"".to_string(),
                            face: FaceName::SansSemiBold,
                            size: 9.0,
                            colour: "#222222".to_string(),
                        }],
                        frame: Some(TextBox {
                            padding: 2.0,
                            radius: 1.0,
                            fill: Some("#FFFFFF".to_string()),
                            border: Some(line),
                        }),
                    },
                ),
                Item {
                    name: None,
                    primitive: Primitive::Raster {
                        at: (92.0, 5.0),
                        width: 6.0,
                        height: 40.0,
                        columns: 1,
                        rows: 2,
                        cells: vec![rgb("#FDE725"), rgb("#440154")],
                    },
                },
            ],
            notes: vec!["2 rows with NA in x or y dropped".to_string()],
        };
        assert_eq!(
            drawing.describe(),
            "drawing 100 by 50\n\
             [background] rectangle 0,0 100 by 50 fill #FFFFFF\n\
             [panel] clip 10,5 80 by 40\n  \
               point 20,15.5 star 4.5 #E69F00 opacity 0.5\n  \
               path M 1,2 L 3,4 C 5,6 7,8 9,10 A 2,3 0 0 1 11,12 Z stroke #222222 0.5 opacity 1\n  \
               polygon 0,0 4,0 4,4 | 1,1 2,1 2,2 fill #56B4E9 opacity 1\n\
             [axis:x] segment 10,45 to 90,45 stroke #222222 0.5 arrow at the end\n\
             [axis:x] text 50,49 middle turned 90 boxed 2 1 fill #FFFFFF stroke #222222 0.5 \"Hourly \\\"wage\\\"\" semibold 9 #222222\n\
             raster 92,5 6 by 40 cells 1 by 2 from #FDE725 to #440154\n\
             note: 2 rows with NA in x or y dropped\n"
        );
        // The clip counts with what it holds.
        assert_eq!(drawing.primitives().len(), 8);
    }
}
