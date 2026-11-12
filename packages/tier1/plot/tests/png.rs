//! The PNG of a figure (Language Reference, section 7.6; release 0.7.0, step 22b2; open
//! decision #118): a drawing in dots, and the file the dots are written as. Every primitive
//! of the closed set drawn at 300 dots to the inch, with dots that must have a given colour
//! (the page, the inside of a mark, the ink of an axis, a hole, a cell) and the picture kept
//! as its size and its hash; the sample figures of the shipped wages, each decoded by an
//! independent reader of PNG; and what holds of the reckoning and of every file: the shares
//! of the dots a shape covers sum to its area, cut by the page where it reaches beyond it;
//! two shapes side by side cover what the two together do; a region that clips leaves what
//! is outside it untouched; marks that are not opaque darken each other and the pieces of
//! one line do not; cells leave no seam; the letters have the area of their outlines; the
//! file gives back the picture dot for dot and says how many dots there are to the metre;
//! and the same drawing gives the same bytes.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::Aesthetic;
use zeff_plot::drawing::{Anchor, Drawing, Item, Place, Primitive, Span, Step, Stroke};
use zeff_plot::font::{FaceName, face};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::pdf::rgb;
use zeff_plot::png::write;
use zeff_plot::raster::{DOTS, MOST_DOTS, Picture, RasterError, coverage, dots_of, ink_of, render};

const WHITE: [u8; 3] = [255, 255, 255];

/// A file as the independent reader finds it.
struct Read {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
    /// Dots to the metre across and down, when the file says.
    to_the_metre: Option<(u32, u32)>,
    /// Whether the file says its colours are those of a screen.
    of_a_screen: bool,
}

fn read(file: &[u8]) -> Result<Read, String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|why| format!("not a PNG: {why}"))?;
    let info = reader.info();
    if info.color_type != png::ColorType::Rgb
        || info.bit_depth != png::BitDepth::Eight
        || info.interlaced
    {
        return Err("not red, green and blue of eight bits each, row after row".to_string());
    }
    let (width, height) = (info.width as usize, info.height as usize);
    let to_the_metre = match info.pixel_dims {
        Some(d) if d.unit == png::Unit::Meter => Some((d.xppu, d.yppu)),
        Some(_) => return Err("a density that is not to the metre".to_string()),
        None => None,
    };
    let of_a_screen = info.srgb.is_some();
    let mut rgb = vec![0; reader.output_buffer_size().ok_or("a picture too large")?];
    let frame = reader
        .next_frame(&mut rgb)
        .map_err(|why| format!("the dots cannot be read: {why}"))?;
    rgb.truncate(frame.buffer_size());
    reader
        .finish()
        .map_err(|why| format!("the file does not end well: {why}"))?;
    Ok(Read {
        width,
        height,
        rgb,
        to_the_metre,
        of_a_screen,
    })
}

/// The names of the chunks of a file, in order.
fn chunks(file: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut at = 8;
    while at + 12 <= file.len() {
        let length = u32::from_be_bytes([file[at], file[at + 1], file[at + 2], file[at + 3]]);
        names.push(String::from_utf8_lossy(&file[at + 4..at + 8]).into_owned());
        at += 12 + length as usize;
    }
    assert_eq!(at, file.len(), "bytes after the last chunk");
    names
}

/// What must hold of the file of a picture: said as a sentence when it does not.
fn checks(file: &[u8], picture: &Picture, dots: f64) -> Result<(), String> {
    let found = read(file)?;
    if (found.width, found.height) != (picture.width, picture.height) {
        return Err(format!("the file is {} by {}", found.width, found.height));
    }
    if found.rgb != picture.rgb {
        return Err("the dots of the file are not the dots of the picture".to_string());
    }
    let wanted = (dots / 0.0254).round() as u32;
    if found.to_the_metre != Some((wanted, wanted)) {
        return Err(format!("{:?} dots to the metre", found.to_the_metre));
    }
    if !found.of_a_screen {
        return Err("the file does not say what its colours are".to_string());
    }
    // The five chunks and no other: nothing of the day, of a program or of a machine.
    if chunks(file) != ["IHDR", "sRGB", "pHYs", "IDAT", "IEND"] {
        return Err(format!("the chunks are {:?}", chunks(file)));
    }
    Ok(())
}

/// One colour laid over another in a measure, each part to the nearest of 255.
fn mix(under: [u8; 3], over: [u8; 3], share: f64) -> [u8; 3] {
    std::array::from_fn(|i| {
        (f64::from(under[i]) + (f64::from(over[i]) - f64::from(under[i])) * share).round() as u8
    })
}

/// Whether two colours are within one step of each other in every part.
fn near(a: [u8; 3], b: [u8; 3]) -> bool {
    (0..3).all(|i| a[i].abs_diff(b[i]) <= 1)
}

/// The length and the hash of a picture and of its file, for a snapshot.
fn kept(picture: &Picture, file: &[u8]) -> String {
    format!(
        "{} by {} dots, {}; the file {} bytes, {}",
        picture.width,
        picture.height,
        zeff_hash::hash_hex(&picture.rgb),
        file.len(),
        zeff_hash::hash_hex(file)
    )
}

/// A drawing of items, on a white page of a size in points.
fn of(width: f64, height: f64, items: Vec<Item>) -> Drawing {
    let mut all = vec![unnamed(Primitive::Rectangle {
        at: (0.0, 0.0),
        width,
        height,
        radius: 0.0,
        fill: Some("#FFFFFF".to_string()),
        stroke: None,
    })];
    all.extend(items);
    Drawing {
        width,
        height,
        items: all,
        notes: Vec::new(),
    }
}

fn unnamed(primitive: Primitive) -> Item {
    Item {
        name: None,
        primitive,
    }
}

/// A rectangle of one colour and opacity, as a path.
fn slab(x: f64, y: f64, width: f64, height: f64, colour: &str, opacity: f64) -> Item {
    unnamed(Primitive::Path {
        steps: vec![
            Step::Move((x, y)),
            Step::Line((x + width, y)),
            Step::Line((x + width, y + height)),
            Step::Line((x, y + height)),
            Step::Close,
        ],
        stroke: None,
        fill: Some(colour.to_string()),
        opacity,
    })
}

// ----- every primitive -----

#[test]
fn every_primitive_of_the_closed_set_is_drawn() {
    let drawing = every_primitive();
    let picture = render(&drawing, DOTS).unwrap();
    // 200 points by 120 at 300 dots to the inch: 833 and a third, cut, by 500.
    assert_eq!((picture.width, picture.height), (833, 500));
    let file = write(&picture, DOTS).unwrap();
    checks(&file, &picture, DOTS).unwrap();
    if let Ok(folder) = std::env::var("ZEFF_FIGURES_DIR") {
        std::fs::write(format!("{folder}/primitives.png"), &file).unwrap();
    }
    // The dot at a place given in points.
    let at = |x: f64, y: f64| {
        picture.dot(
            (x * DOTS / 72.0).floor() as usize,
            (y * DOTS / 72.0).floor() as usize,
        )
    };
    let ink = rgb("#222222");
    // The page; the inside of a mark, a circle and then a square.
    assert_eq!(at(2.0, 2.0), WHITE);
    assert_eq!(at(12.0, 30.0), rgb("#E69F00"));
    assert_eq!(at(36.0, 30.0), rgb("#E69F00"));
    // A mark that is half opaque lets half of the page through.
    assert!(
        near(at(48.0, 30.0), mix(WHITE, rgb("#56B4E9"), 0.5)),
        "{:?}",
        at(48.0, 30.0)
    );
    // An outline shape: the page inside it, ink on its line.
    assert_eq!(at(120.0, 30.0), WHITE);
    assert_eq!(at(124.5, 30.0), ink);
    // A shape of lines: ink where its two lines cross, the page beside them.
    assert_eq!(at(96.0, 30.0), ink);
    assert_eq!(at(96.0, 27.0), WHITE);
    // A name that is no shape is drawn as a circle.
    assert_eq!(at(190.0, 30.0), rgb("#009E73"));
    // A path filled and not opaque; a polygon, and its hole.
    assert!(
        near(at(25.0, 67.0), mix(WHITE, rgb("#F0E442"), 0.6)),
        "{:?}",
        at(25.0, 67.0)
    );
    assert_eq!(at(93.0, 62.0), rgb("#0072B2"));
    assert_eq!(at(105.0, 70.0), WHITE);
    // A rectangle with round corners and no fill: the page inside, ink on its side, and
    // the page at its corner, which the round cuts off.
    assert_eq!(at(140.0, 70.0), WHITE);
    assert_eq!(at(130.0, 70.0), ink);
    assert_eq!(at(130.1, 60.1), WHITE);
    // The ink of an axis, half a point wide: two dots.
    assert_eq!(at(45.0, 90.0), ink);
    assert_eq!(at(45.0, 89.5), WHITE);
    // The head of an arrow at its end.
    assert_eq!(at(78.0, 90.0), ink);
    assert_eq!(at(77.0, 90.9), ink);
    assert_eq!(at(74.0, 91.6), WHITE);
    // The six cells, each at its middle.
    let cells = [
        "#440154", "#440154", "#21918C", "#FDE725", "#21918C", "#FDE725",
    ];
    for (i, cell) in cells.iter().enumerate() {
        let (column, row) = ((i % 3) as f64, (i / 3) as f64);
        assert_eq!(at(165.0 + 10.0 * column, 65.0 + 10.0 * row), rgb(cell));
    }
    // The box behind a text is filled; letters stand in it.
    assert_eq!(at(119.0, 103.0), rgb("#F7F7F7"));
    let dark = |x0: f64, y0: f64, x1: f64, y1: f64| {
        let s = DOTS / 72.0;
        let mut count = 0;
        for y in (y0 * s) as usize..(y1 * s) as usize {
            for x in (x0 * s) as usize..(x1 * s) as usize {
                if picture.dot(x, y) == ink {
                    count += 1;
                }
            }
        }
        count
    };
    // Dots that are wholly ink, in each of the three texts.
    let (first, boxed, upwards) = (
        dark(0.0, 96.0, 90.0, 107.0),
        dark(120.0, 103.0, 165.0, 115.0),
        dark(183.0, 100.0, 191.0, 110.0),
    );
    assert!(
        first > 1000 && boxed > 800 && upwards > 80,
        "{first} {boxed} {upwards}"
    );
    insta::assert_snapshot!("primitives", kept(&picture, &file));
}

// ----- the reckoning, seen in dots -----

#[test]
fn a_region_that_clips_leaves_what_is_outside_it_untouched() {
    // A black disc of radius 20 about the corner of a region from (30, 30) to (60, 60).
    let disc = unnamed(Primitive::Point {
        at: (30.0, 30.0),
        shape: "circle",
        size: 40.0,
        colour: "#000000".to_string(),
        opacity: 1.0,
    });
    let clipped = of(
        72.0,
        72.0,
        vec![unnamed(Primitive::Clip {
            at: (30.0, 30.0),
            width: 30.0,
            height: 30.0,
            items: vec![disc.clone()],
        })],
    );
    let picture = render(&clipped, 72.0).unwrap();
    let mut darkness = 0.0;
    for y in 0..72 {
        for x in 0..72 {
            let dot = picture.dot(x, y);
            if x < 30 || y < 30 {
                assert_eq!(dot, WHITE, "{x},{y}");
            }
            darkness += f64::from(255 - dot[0]) / 255.0;
        }
    }
    // A quarter of the disc is inside the region.
    let quarter = std::f64::consts::PI * 400.0 / 4.0;
    assert!(
        (darkness / quarter - 1.0).abs() < 0.005,
        "{darkness} of {quarter}"
    );
    assert_eq!(picture.dot(31, 31), [0, 0, 0]);
    // A region at half a dot: the dot it cuts is half covered.
    let half = of(
        20.0,
        20.0,
        vec![unnamed(Primitive::Clip {
            at: (5.5, 5.0),
            width: 10.0,
            height: 10.0,
            items: vec![slab(0.0, 0.0, 20.0, 20.0, "#000000", 1.0)],
        })],
    );
    let picture = render(&half, 72.0).unwrap();
    assert_eq!(picture.dot(4, 8), WHITE);
    assert_eq!(picture.dot(5, 8), [128, 128, 128]);
    assert_eq!(picture.dot(6, 8), [0, 0, 0]);
    assert_eq!(picture.dot(15, 8), [128, 128, 128]);
    assert_eq!(picture.dot(16, 8), WHITE);
    // A region inside a region shows what the two share; one outside the other, nothing.
    let nested = |inner: (f64, f64)| {
        of(
            20.0,
            20.0,
            vec![unnamed(Primitive::Clip {
                at: (2.0, 2.0),
                width: 8.0,
                height: 8.0,
                items: vec![unnamed(Primitive::Clip {
                    at: inner,
                    width: 8.0,
                    height: 8.0,
                    items: vec![slab(0.0, 0.0, 20.0, 20.0, "#000000", 1.0)],
                })],
            })],
        )
    };
    let shared = render(&nested((6.0, 6.0)), 72.0).unwrap();
    let black = shared.rgb.chunks(3).filter(|dot| dot == &[0, 0, 0]).count();
    assert_eq!(black, 16);
    assert_eq!(shared.dot(6, 6), [0, 0, 0]);
    let apart = render(&nested((12.0, 12.0)), 72.0).unwrap();
    assert!(apart.rgb.iter().all(|&part| part == 255));
}

#[test]
fn marks_that_are_not_opaque_darken_each_other_and_the_pieces_of_one_line_do_not() {
    // Two black slabs, each half opaque, that share a square.
    let two = of(
        30.0,
        30.0,
        vec![
            slab(5.0, 5.0, 12.0, 12.0, "#000000", 0.5),
            slab(11.0, 11.0, 12.0, 12.0, "#000000", 0.5),
        ],
    );
    let picture = render(&two, 72.0).unwrap();
    assert_eq!(picture.dot(7, 7), [128, 128, 128]);
    assert_eq!(picture.dot(20, 20), [128, 128, 128]);
    assert_eq!(picture.dot(14, 14), [64, 64, 64]);
    // A line of two pieces that meet at a right corner, half opaque: where the pieces lie
    // over each other, and at the point of the corner, it is as dark as anywhere along it.
    let bent = of(
        40.0,
        40.0,
        vec![unnamed(Primitive::Path {
            steps: vec![
                Step::Move((5.0, 10.0)),
                Step::Line((30.0, 10.0)),
                Step::Line((30.0, 35.0)),
            ],
            stroke: Some(Stroke {
                colour: "#000000".to_string(),
                width: 6.0,
            }),
            fill: None,
            opacity: 0.5,
        })],
    );
    let picture = render(&bent, 72.0).unwrap();
    for (x, y) in [(15, 10), (28, 11), (31, 8), (32, 7), (30, 20), (28, 12)] {
        assert_eq!(picture.dot(x, y), [128, 128, 128], "{x},{y}");
    }
    assert_eq!(picture.dot(33, 6), WHITE);
    assert_eq!(picture.dot(26, 14), WHITE);
    // Its ends are cut square: nothing before its start or after its end.
    assert_eq!(picture.dot(4, 10), WHITE);
    assert_eq!(picture.dot(30, 35), WHITE);
    assert_eq!(picture.dot(30, 34), [128, 128, 128]);
}

#[test]
fn cells_leave_no_seam_and_a_dot_over_two_takes_of_both() {
    let grid = |at: Place, width: f64, columns: usize, cells: &[&str]| {
        of(
            12.0,
            6.0,
            vec![unnamed(Primitive::Raster {
                at,
                width,
                height: 2.0,
                columns,
                rows: 1,
                cells: cells.iter().map(|c| c.to_string()).collect(),
            })],
        )
    };
    // Two cells of one colour that meet in the middle of a dot: the dot is that colour.
    let same = render(&grid((1.0, 2.0), 5.0, 2, &["#336699", "#336699"]), 72.0).unwrap();
    for x in 1..6 {
        assert_eq!(same.dot(x, 2), rgb("#336699"), "{x}");
        assert_eq!(same.dot(x, 3), rgb("#336699"), "{x}");
    }
    assert_eq!(same.dot(0, 2), WHITE);
    assert_eq!(same.dot(6, 2), WHITE);
    assert_eq!(same.dot(3, 1), WHITE);
    assert_eq!(same.dot(3, 4), WHITE);
    // Black and white that meet in the middle of a dot: the dot is grey.
    let two = render(&grid((1.0, 2.0), 5.0, 2, &["#000000", "#FFFFFF"]), 72.0).unwrap();
    assert_eq!(two.dot(2, 2), [0, 0, 0]);
    assert_eq!(two.dot(3, 2), [128, 128, 128]);
    assert_eq!(two.dot(4, 2), WHITE);
    // A grid that starts at a quarter of a dot covers three quarters of its first dot.
    let off = render(&grid((1.25, 2.0), 4.0, 1, &["#000000"]), 72.0).unwrap();
    assert_eq!(off.dot(1, 2), [64, 64, 64]);
    assert_eq!(off.dot(2, 2), [0, 0, 0]);
    assert_eq!(off.dot(5, 2), [191, 191, 191]);
    // Fewer colours than cells: the cells that have none are not drawn. No cells, nothing.
    let short = render(&grid((1.0, 2.0), 6.0, 3, &["#000000"]), 72.0).unwrap();
    assert_eq!(short.dot(2, 2), [0, 0, 0]);
    assert_eq!(short.dot(3, 2), WHITE);
    let none = render(&grid((1.0, 2.0), 6.0, 0, &[]), 72.0).unwrap();
    assert!(none.rgb.iter().all(|&part| part == 255));
    // Many cells under one dot: the dot is their mean.
    let fine = render(
        &grid(
            (2.0, 2.0),
            1.0,
            4,
            &["#000000", "#FFFFFF", "#000000", "#FFFFFF"],
        ),
        72.0,
    )
    .unwrap();
    assert_eq!(fine.dot(2, 2), [128, 128, 128]);
}

/// Twice the area a closed outline of lines and curves of one control goes round, by the
/// rule of the integral along it, kept as the independent reader of fonts walks it.
#[derive(Default)]
struct Area {
    twice: f64,
    at: (f64, f64),
    start: (f64, f64),
}

impl Area {
    fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
        a.0 * b.1 - a.1 * b.0
    }
}

impl ttf_parser::OutlineBuilder for Area {
    fn move_to(&mut self, x: f32, y: f32) {
        self.at = (f64::from(x), f64::from(y));
        self.start = self.at;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let to = (f64::from(x), f64::from(y));
        self.twice += Area::cross(self.at, to);
        self.at = to;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (control, to) = ((f64::from(x1), f64::from(y1)), (f64::from(x), f64::from(y)));
        self.twice += (2.0 * Area::cross(self.at, control)
            + 2.0 * Area::cross(control, to)
            + Area::cross(self.at, to))
            / 3.0;
        self.at = to;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        panic!("a TrueType outline has no curve of two controls");
    }
    fn close(&mut self) {
        self.twice += Area::cross(self.at, self.start);
        self.at = self.start;
    }
}

#[test]
fn the_letters_have_the_area_of_their_outlines_and_stand_where_the_font_sets_them() {
    let size = 40.0;
    let line = |words: &str, name: FaceName, turn: f64| {
        of(
            260.0,
            260.0,
            vec![unnamed(Primitive::Text {
                at: (30.0, 200.0),
                anchor: Anchor::Start,
                turn,
                spans: vec![Span {
                    text: words.to_string(),
                    face: name,
                    size,
                    colour: "#000000".to_string(),
                }],
                frame: None,
            })],
        )
    };
    for (words, name) in [
        ("Hxg", FaceName::Sans),
        ("Wage 0.47", FaceName::SansSemiBold),
        ("lm(y)", FaceName::Mono),
    ] {
        // The area of the letters as the independent reader of fonts gives their outlines.
        let font = face(name);
        let theirs = ttf_parser::Face::parse(name.bytes(), 0).unwrap();
        let mut area = Area::default();
        for placed in font.shape(words).glyphs {
            theirs.outline_glyph(ttf_parser::GlyphId(placed.glyph), &mut area);
        }
        let em = f64::from(font.units_per_em());
        let wanted = (area.twice / 2.0).abs() * (size / em) * (size / em);
        for turn in [0.0, 90.0, 30.0] {
            let picture = render(&line(words, name, turn), 72.0).unwrap();
            // The ink, in dots: black on white, each dot in the measure it is covered.
            let (mut ink, mut least, mut most) = (0.0, (usize::MAX, usize::MAX), (0, 0));
            for y in 0..picture.height {
                for x in 0..picture.width {
                    let dark = 255 - picture.dot(x, y)[0];
                    if dark > 0 {
                        ink += f64::from(dark) / 255.0;
                        least = (least.0.min(x), least.1.min(y));
                        most = (most.0.max(x), most.1.max(y));
                    }
                }
            }
            assert!(
                (ink / wanted - 1.0).abs() < 0.01,
                "{words} turned {turn}: {ink} dots of ink, {wanted} in the outlines"
            );
            // The line stands on its baseline from its start to the end of its advance.
            let long = font.width(words, size);
            let (up, down) = (
                f64::from(font.ascender()) * size / em,
                -f64::from(font.descender()) * size / em,
            );
            if turn == 0.0 {
                assert!(least.0 as f64 >= 30.0 - 3.0 && most.0 as f64 <= 30.0 + long + 3.0);
                assert!(least.1 as f64 >= 200.0 - up && most.1 as f64 <= 200.0 + down);
                // The last letter may leave some of its advance empty, never half the size
                // of the type.
                assert!(most.0 as f64 > 30.0 + long - 0.5 * size, "{words}");
            } else if turn == 90.0 {
                // Read upwards: from its place towards the top of the page.
                assert!(most.1 as f64 <= 200.0 + 3.0 && least.1 as f64 >= 200.0 - long - 3.0);
                assert!(least.0 as f64 >= 30.0 - up && most.0 as f64 <= 30.0 + down);
            }
        }
    }
}

#[test]
fn a_page_of_no_dots_and_one_of_too_many_are_refused() {
    let page = |width: f64, height: f64| Drawing {
        width,
        height,
        items: Vec::new(),
        notes: Vec::new(),
    };
    // A page with nothing on it is white.
    let empty = render(&page(10.0, 5.0), 72.0).unwrap();
    assert_eq!((empty.width, empty.height), (10, 5));
    assert!(empty.rgb.iter().all(|&part| part == 255));
    assert_eq!(render(&page(0.2, 5.0), 72.0), Err(RasterError::NotASize));
    assert_eq!(render(&page(10.0, 5.0), 0.0), Err(RasterError::NotASize));
    assert_eq!(
        render(&page(f64::NAN, 5.0), 72.0),
        Err(RasterError::NotASize)
    );
    assert_eq!(
        render(&page(10.0, 5.0), f64::INFINITY),
        Err(RasterError::NotASize)
    );
    // Forty million dots are drawn and one row more is not.
    assert_eq!(MOST_DOTS, 40_000_000);
    assert_eq!(
        render(&page(8000.0, 5001.0), 72.0),
        Err(RasterError::TooLarge {
            width: 8000.0,
            height: 5001.0
        })
    );
    // A metre at 300 dots to the inch is 11,811 dots.
    assert_eq!(dots_of(1000.0 * 72.0 / 25.4, DOTS), 11811.0);
}

// ----- figures -----

/// What holds of the picture of a figure laid out from a scene: the page is white at its
/// corner; an axis is ink at its middle; and the last mark, when it is opaque and filled, is
/// its colour at its centre.
fn figure_checks(drawing: &Drawing, picture: &Picture, dots: f64) -> Result<(), String> {
    let s = dots / 72.0;
    if (picture.width as f64, picture.height as f64)
        != (dots_of(drawing.width, dots), dots_of(drawing.height, dots))
    {
        return Err(format!("{} by {} dots", picture.width, picture.height));
    }
    if picture.dot(0, 0) != WHITE {
        return Err("the corner of the page is not white".to_string());
    }
    let dot = |at: Place| {
        let (x, y) = ((at.0 * s).floor() as usize, (at.1 * s).floor() as usize);
        (x < picture.width && y < picture.height).then(|| picture.dot(x, y))
    };
    for item in &drawing.items {
        match &item.primitive {
            // A line two dots wide or more covers the dot its middle stands in.
            Primitive::Segment {
                from, to, stroke, ..
            } if stroke.width * s >= 2.0
                && item.name.as_deref().is_some_and(|n| n.starts_with("axis")) =>
            {
                let middle = ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0);
                if dot(middle).is_some_and(|found| found != rgb(&stroke.colour)) {
                    return Err(format!("no ink at the middle of an axis, {middle:?}"));
                }
            }
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } => {
                if let Some(Primitive::Point {
                    at: centre,
                    shape,
                    colour,
                    opacity,
                    size,
                }) = items.last().map(|item| &item.primitive)
                    && *opacity == 1.0
                    && !shape.ends_with("_open")
                    && matches!(*shape, "circle" | "square" | "diamond")
                    && *size * s >= 4.0
                    && centre.0 > at.0 + 1.0
                    && centre.0 < at.0 + width - 1.0
                    && centre.1 > at.1 + 1.0
                    && centre.1 < at.1 + height - 1.0
                    && dot(*centre).is_some_and(|found| found != rgb(colour))
                {
                    return Err(format!("the last mark is not its colour at {centre:?}"));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[test]
fn a_small_figure_is_drawn_and_its_file_read_back() {
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
    let picture = render(&drawing, DOTS).unwrap();
    assert_eq!((picture.width, picture.height), (1051, 779));
    figure_checks(&drawing, &picture, DOTS).unwrap();
    let file = write(&picture, DOTS).unwrap();
    checks(&file, &picture, DOTS).unwrap();
    assert_eq!(read(&file).unwrap().to_the_metre, Some((11811, 11811)));
    // The colours of the picture: the page, the ink, the grid, the two colours of the
    // marks, and what lies between them at an edge.
    let has = |colour: &str| picture.rgb.chunks(3).any(|dot| dot == rgb(colour));
    for colour in ["#FFFFFF", "#222222", "#E3E3E3", "#E69F00", "#56B4E9"] {
        assert!(has(colour), "{colour}");
    }
    // Most of the page is white.
    let white = picture.rgb.chunks(3).filter(|dot| dot == &WHITE).count();
    assert!(white * 10 > picture.width * picture.height * 8);
    insta::assert_snapshot!("small_figure", kept(&picture, &file));
    // At a lower number of dots to the inch the same page is smaller, and says so.
    let coarse = render(&drawing, 96.0).unwrap();
    assert_eq!((coarse.width, coarse.height), (336, 249));
    let file = write(&coarse, 96.0).unwrap();
    checks(&file, &coarse, 96.0).unwrap();
    assert_eq!(read(&file).unwrap().to_the_metre, Some((3780, 3780)));
}

#[test]
fn the_sample_figures_are_drawn_and_their_files_read_back() {
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
            let picture = render(&drawing, DOTS).unwrap();
            figure_checks(&drawing, &picture, DOTS)
                .unwrap_or_else(|why| panic!("{name} at {page}: {why}"));
            let file = write(&picture, DOTS).unwrap();
            checks(&file, &picture, DOTS).unwrap_or_else(|why| panic!("{name} at {page}: {why}"));
            all.push_str(&format!("{name} at {page}: {}\n", kept(&picture, &file)));
            if let Some(folder) = &folder {
                std::fs::write(format!("{folder}/{name}_{page}.png"), &file).unwrap();
            }
        }
    }
    insta::assert_snapshot!("sample_figures", all);
}

#[test]
fn the_same_drawing_gives_the_same_dots_whatever_rows_are_behind_it() {
    let mappings = [
        (Aesthetic::X, "dose"),
        (Aesthetic::Y, "response"),
        (Aesthetic::Color, "arm"),
    ];
    let scene = drawn(small("patients_of_ward_7"), &mappings);
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let file = write(&render(&drawing, 150.0).unwrap(), 150.0).unwrap();
    let mut other = drawn(small("d"), &mappings);
    for row in &mut other.panels[0].layers[0].rows {
        *row += 40_000;
    }
    let drawing = layout(&other, Size::OneColumn, &JOURNAL).unwrap();
    assert_eq!(
        write(&render(&drawing, 150.0).unwrap(), 150.0).unwrap(),
        file
    );
    // A file of dots has no words in it: not the name of the frame, nor of a variable.
    for word in ["patients", "dose", "response", "arm"] {
        assert!(
            !file.windows(word.len()).any(|w| w == word.as_bytes()),
            "{word}"
        );
    }
}

// ----- what holds of the reckoning and of every file -----

/// What of a shape with every corner turning one way lies inside a page, corner by corner.
fn cut_to_page(shape: &[Place], width: f64, height: f64) -> Vec<Place> {
    let mut kept = shape.to_vec();
    // Each side of the page: the points with `a·x + b·y <= c` are inside it.
    for (a, b, c) in [
        (-1.0, 0.0, 0.0),
        (1.0, 0.0, width),
        (0.0, -1.0, 0.0),
        (0.0, 1.0, height),
    ] {
        let inside = |p: Place| a * p.0 + b * p.1 <= c;
        let mut next = Vec::new();
        for (i, &from) in kept.iter().enumerate() {
            let to = kept[(i + 1) % kept.len()];
            if inside(from) {
                next.push(from);
            }
            if inside(from) != inside(to) {
                let t = (c - a * from.0 - b * from.1) / (a * (to.0 - from.0) + b * (to.1 - from.1));
                next.push((from.0 + t * (to.0 - from.0), from.1 + t * (to.1 - from.1)));
            }
        }
        kept = next;
        if kept.is_empty() {
            break;
        }
    }
    kept
}

fn area_of(shape: &[Place]) -> f64 {
    let mut twice = 0.0;
    for (i, a) in shape.iter().enumerate() {
        let b = shape[(i + 1) % shape.len()];
        twice += a.0 * b.1 - b.0 * a.1;
    }
    (twice / 2.0).abs()
}

/// A shape with every corner turning one way: corners on a circle, in order round it.
fn rounded_shape() -> impl Strategy<Value = Vec<Place>> {
    (
        (-10.0..40.0f64, -10.0..40.0f64),
        0.5..35.0f64,
        prop::collection::vec(0.0..std::f64::consts::TAU, 3..9),
        any::<bool>(),
    )
        .prop_map(|(centre, radius, mut angles, back)| {
            angles.sort_by(f64::total_cmp);
            angles.dedup();
            let mut shape: Vec<Place> = angles
                .iter()
                .map(|a| (centre.0 + radius * a.cos(), centre.1 + radius * a.sin()))
                .collect();
            if back {
                shape.reverse();
            }
            shape
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(400)))]

    #[test]
    fn the_shares_of_the_dots_a_shape_covers_sum_to_its_area_inside_the_page(
        shape in rounded_shape(),
        even_odd in any::<bool>(),
    ) {
        prop_assume!(shape.len() >= 3);
        let shares = coverage(std::slice::from_ref(&shape), even_odd, 30, 30);
        let wanted = area_of(&cut_to_page(&shape, 30.0, 30.0));
        let sum: f64 = shares.iter().sum();
        prop_assert!((sum - wanted).abs() < 1e-9 * (1.0 + wanted), "{} and {}", sum, wanted);
        for share in &shares {
            prop_assert!(*share >= 0.0 && *share <= 1.0 + 1e-12);
        }
        // Each dot by itself: the area of the shape inside that dot.
        for (x, y) in [(0_usize, 0_usize), (14, 15), (29, 29), (7, 22)] {
            let moved: Vec<Place> = shape.iter().map(|p| (p.0 - x as f64, p.1 - y as f64)).collect();
            let inside = area_of(&cut_to_page(&moved, 1.0, 1.0));
            prop_assert!((shares[y * 30 + x] - inside).abs() < 1e-9, "{},{}", x, y);
        }
    }

    #[test]
    fn two_halves_of_a_shape_cover_what_the_whole_does(
        shape in rounded_shape(),
        cut in 1.0..29.0f64,
    ) {
        prop_assume!(shape.len() >= 3);
        // The shape cut by an upright line: what is left of it and what is right of it.
        let left = cut_to_page(&shape, cut, 1e6);
        let moved: Vec<Place> = shape.iter().map(|p| (p.0 - cut, p.1)).collect();
        let right: Vec<Place> = cut_to_page(&moved, 1e6, 1e6).iter().map(|p| (p.0 + cut, p.1)).collect();
        let whole = coverage(&[cut_to_page(&shape, 1e6, 1e6)], false, 30, 30);
        let mut parts = Vec::new();
        if left.len() >= 3 { parts.push(left); }
        if right.len() >= 3 { parts.push(right); }
        let both = coverage(&parts, false, 30, 30);
        for (a, b) in whole.iter().zip(&both) {
            prop_assert!((a - b).abs() < 1e-9, "{} and {}", a, b);
        }
    }

    #[test]
    fn the_ink_of_a_straight_line_is_its_length_times_its_width(
        from in (5.0..25.0f64, 5.0..25.0f64),
        to in (5.0..25.0f64, 5.0..25.0f64),
        width in 0.1..4.0f64,
    ) {
        let long = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
        prop_assume!(long > 0.01);
        let ink = ink_of(&[(vec![from, to], false)], width);
        let sum: f64 = coverage(&ink, false, 30, 30).iter().sum();
        prop_assert!((sum - long * width).abs() < 1e-9 * (1.0 + long * width));
    }

    #[test]
    fn a_picture_is_given_back_dot_for_dot_by_its_file(
        width in 1usize..40,
        height in 1usize..12,
        seed in any::<u64>(),
        kind in 0u8..4,
        dots in prop_oneof![Just(300.0), Just(72.0), 50.0..1200.0f64],
    ) {
        // Dots of four kinds: of any colour, of two colours, a slope from left to right, one
        // colour everywhere.
        let mut state = seed | 1;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        };
        let rgb: Vec<u8> = (0..width * height * 3)
            .map(|i| match kind {
                0 => next(),
                1 => if next() > 128 { 255 } else { 34 },
                2 => ((i / 3 % width) * 255 / width) as u8,
                _ => 200,
            })
            .collect();
        let picture = Picture { width, height, rgb };
        let file = write(&picture, dots).unwrap();
        if let Err(why) = checks(&file, &picture, dots) {
            prop_assert!(false, "{}", why);
        }
        prop_assert_eq!(write(&picture, dots).unwrap(), file);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(60)))]

    #[test]
    fn every_figure_is_drawn_the_same_twice_and_read_back(
        rows in rows(),
        mappings in showing(),
        size in prop_oneof![
            Just(Size::OneColumn),
            (70.0..200.0f64, 50.0..150.0f64).prop_map(|(w, h)| Size::Millimetres(w, h)),
        ],
        alpha in prop_oneof![Just(1.0), 0.05..1.0f64],
        dots in prop_oneof![Just(72.0), Just(96.0), 60.0..150.0f64],
    ) {
        let scene = drawn_with(tried(&rows), &mappings, Vec::new(), alpha);
        if let Ok(drawing) = layout(&scene, size, &JOURNAL) {
            let picture = render(&drawing, dots).unwrap();
            if let Err(why) = figure_checks(&drawing, &picture, dots) {
                prop_assert!(false, "{}", why);
            }
            prop_assert_eq!(&render(&drawing, dots).unwrap(), &picture);
            let file = write(&picture, dots).unwrap();
            if let Err(why) = checks(&file, &picture, dots) {
                prop_assert!(false, "{}", why);
            }
        }
    }
}
