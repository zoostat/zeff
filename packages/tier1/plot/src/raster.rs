//! The picture of a drawing in dots (Language Reference, section 7.6, "Rendering and export";
//! release 0.7.0, step 22b2; open decision #118): what a PNG file holds, the same on every
//! machine.
//!
//! [`render`] turns a drawing ([`crate::drawing`]) into a grid of dots at so many dots to the
//! inch, [`DOTS`] for a figure, and is a pure function of the two. The page is white. Every
//! primitive of the closed set is drawn, with the geometry the other writers use: the shapes
//! of [`crate::shape`], the arcs, circles and round corners of the PDF writer, and the letters
//! from the outlines of the embedded fonts ([`crate::font`]), set as the SVG writer sets them.
//!
//! **How many dots.** A page of `w` points is `w` times the dots to the inch over 72 dots
//! wide, and what is left of a dot at the right and at the bottom is cut, as R cuts it: a
//! figure of one column, 89 mm by 66, is 1051 dots by 779 at 300 to the inch.
//!
//! **Edges.** A dot at the edge of a shape takes the colour of the shape in the measure in
//! which the shape covers it, and that measure is the area itself, worked out from the
//! straight edges of the shape, not counted on a finer grid. A curve is first made of
//! straight lines that stand no farther than a twentieth of a dot from it. A shape with a
//! hole, and a region that clips, are taken into the same reckoning, so that an edge cut by
//! the region is as exact as any other. What is filled by the rule of the SVG and the PDF: a
//! path wherever it winds round, a polygon with holes wherever it winds round an odd number
//! of times.
//!
//! **Lines.** A line is the shape of its ink: a rectangle along each straight piece and, at
//! each corner, the wedge that fills it, drawn out to a point where the point stays within
//! four widths, cut flat where it would not, as the other writers have it; its ends are cut
//! square. The pieces of one line are one shape, so that a line that is not opaque is no
//! darker where its pieces meet.
//!
//! **Cells.** A dot over a grid of cells takes the colours of the cells under it, each in the
//! measure of its share of the dot, so that no seam shows between two cells.
//!
//! **Colour.** Each dot keeps its red, green and blue to one part in 65,535 while the page is
//! drawn, so that many thousand marks that are not opaque, one over the other, lose nothing
//! to rounding; the picture given back has 256 steps of each. Paint is laid over what is
//! under it in the measure of its opacity, in the numbers of the colours as they are written.
//!
//! **The same on every machine.** The reckoning uses addition, subtraction, multiplication,
//! division and the square root of numbers of 64 bits, which every machine does alike, the
//! engine's own sine and cosine where a text is turned or an arc drawn, and whole numbers for
//! the colours. Nothing is drawn in an order that depends on the machine.

use zeff_numeric::maths::sqrt;

use crate::drawing::{Anchor, Drawing, Head, Item, Place, Primitive, Span, Stroke, TextBox, rgb};
use crate::font::{Segment, face};
use crate::shape::Geometry;
use crate::svg::{ABOVE, BELOW, MARK_STROKE, geometry, turned};
use crate::trace::{Piece, Trace};

/// The dots to the inch of a figure written as a picture.
pub const DOTS: f64 = 300.0;

/// The most dots a picture is made of: more is refused.
pub const MOST_DOTS: usize = 40_000_000;

/// How far a curve may stand from the straight lines it is drawn as, in dots.
const FLAT: f64 = 0.05;

/// The most straight lines one curve is made of.
const MOST_LINES: f64 = 512.0;

/// How far a corner of a line may reach beyond its place, in widths of the line, before it
/// is cut flat.
const MITRE: f64 = 4.0;

/// A picture: its dots row by row from the top left, the red, green and blue of each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

impl Picture {
    /// The red, green and blue of the dot in column `x` of row `y`, from 0.
    pub fn dot(&self, x: usize, y: usize) -> [u8; 3] {
        let at = (y * self.width + x) * 3;
        [self.rgb[at], self.rgb[at + 1], self.rgb[at + 2]]
    }
}

/// Why a drawing is not drawn in dots.
#[derive(Debug, Clone, PartialEq)]
pub enum RasterError {
    /// The page, or the number of dots to the inch, leaves no dot to draw.
    NotASize,
    /// The picture would have more than [`MOST_DOTS`]: how many across and down.
    TooLarge { width: f64, height: f64 },
}

/// How many dots a length of `points` is, at `dots` to the inch: what is left of a dot is cut.
pub fn dots_of(points: f64, dots: f64) -> f64 {
    // A length that is a whole number of dots but for the last bit of the reckoning is whole.
    (points * dots / 72.0 + 1e-6).floor()
}

/// The picture of `drawing` at `dots` to the inch.
pub fn render(drawing: &Drawing, dots: f64) -> Result<Picture, RasterError> {
    draw(
        &drawing.items,
        (0.0, 0.0),
        dots_of(drawing.width, dots),
        dots_of(drawing.height, dots),
        dots,
    )
}

/// The picture of a part of a page: what `items` draw in the rectangle that starts at `at`
/// and is `width` by `height` points, at `dots` to the inch. The first dot starts at `at`,
/// and what is left of a dot at the right and at the bottom is a dot more, so that the
/// picture covers the part.
pub fn render_part(
    items: &[Item],
    at: Place,
    width: f64,
    height: f64,
    dots: f64,
) -> Result<Picture, RasterError> {
    let covering = |points: f64| (points * dots / 72.0 - 1e-6).ceil();
    draw(items, at, covering(width), covering(height), dots)
}

/// The picture, `wide` by `high` dots, of what `items` draw from `at` on.
fn draw(
    items: &[Item],
    at: Place,
    wide: f64,
    high: f64,
    dots: f64,
) -> Result<Picture, RasterError> {
    if !(wide >= 1.0 && high >= 1.0 && wide.is_finite() && high.is_finite()) {
        return Err(RasterError::NotASize);
    }
    if wide * high > MOST_DOTS as f64 {
        return Err(RasterError::TooLarge {
            width: wide,
            height: high,
        });
    }
    let (width, height) = (wide as usize, high as usize);
    let scale = dots / 72.0;
    let mut page = Page {
        width,
        height,
        ink: vec![u16::MAX; width * height * 3],
        scale,
        shift: (at.0 * scale, at.1 * scale),
        clip: Rect {
            x0: 0.0,
            y0: 0.0,
            x1: wide,
            y1: high,
        },
        mask: Mask::default(),
    };
    page.items(items);
    // From one part in 65,535 to one in 255, to the nearest.
    let rgb = page
        .ink
        .iter()
        .map(|&part| ((u32::from(part) + 128) / 257) as u8)
        .collect();
    Ok(Picture { width, height, rgb })
}

/// The share of each dot of a page of `width` by `height` dots that closed shapes cover,
/// row by row: the shapes given by their corners, in dots. With `even_odd` what is wound
/// round an even number of times is outside. For the tests of the reckoning.
pub fn coverage(shapes: &[Vec<Place>], even_odd: bool, width: usize, height: usize) -> Vec<f64> {
    let mut out = vec![0.0; width * height];
    let mut mask = Mask::default();
    let clip = Rect {
        x0: 0.0,
        y0: 0.0,
        x1: width as f64,
        y1: height as f64,
    };
    if let Some(bounds) = bounds_of(shapes)
        && mask.begin(bounds, clip)
    {
        for shape in shapes {
            mask.shape(shape);
        }
        mask.each(even_odd, |x, y, share| out[y * width + x] = share);
    }
    out
}

/// The shapes of the ink of lines of a width: for each line its corners and whether it comes
/// back to its start. For the tests of the reckoning.
pub fn ink_of(lines: &[(Vec<Place>, bool)], width: f64) -> Vec<Vec<Place>> {
    let mut out = Vec::new();
    for (corners, closed) in lines {
        stroke(corners, *closed, width, &mut out);
    }
    out
}

/// A rectangle with its sides along the axes, in dots.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Rect {
    /// What two rectangles share; `None` when they share nothing, or one is no rectangle.
    fn both(self, other: Rect) -> Option<Rect> {
        let shared = Rect {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };
        (shared.x1 > shared.x0 && shared.y1 > shared.y0).then_some(shared)
    }
}

/// The rectangle that holds every corner of shapes; `None` for no corner, or a corner that
/// is no number.
fn bounds_of(shapes: &[Vec<Place>]) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for &(x, y) in shapes.iter().flatten() {
        if !(x.is_finite() && y.is_finite()) {
            return None;
        }
        bounds = Some(match bounds {
            None => Rect {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
            Some(b) => Rect {
                x0: b.x0.min(x),
                y0: b.y0.min(y),
                x1: b.x1.max(x),
                y1: b.y1.max(y),
            },
        });
    }
    bounds
}

/// The reckoning of how much of each dot shapes cover, over the dots a rectangle touches.
///
/// Each straight edge adds, to every dot of its rows, the area of the dot that lies to the
/// right of it, counted up for an edge that goes down the page and down for one that goes
/// up; what is added is kept as the change from one dot to the next, so that an edge touches
/// only the dots it crosses and the sum along a row gives, for each dot, how many times the
/// shapes wind round it, weighted by area.
#[derive(Default)]
struct Mask {
    /// The changes, a row after a row, each row one longer than the dots it has.
    changes: Vec<f64>,
    /// For each row the first column touched and one past the last.
    touched: Vec<(usize, usize)>,
    /// The dot of the page the first dot of the mask is.
    left: usize,
    top: usize,
    wide: usize,
    high: usize,
    /// What edges are cut to, measured from the first dot of the mask.
    cut: Option<Rect>,
}

impl Mask {
    /// Makes ready for shapes within `bounds`, drawn inside `clip`; false when nothing of
    /// them can show.
    fn begin(&mut self, bounds: Rect, clip: Rect) -> bool {
        let Some(shared) = bounds.both(clip) else {
            return false;
        };
        self.left = shared.x0.floor() as usize;
        self.top = shared.y0.floor() as usize;
        self.wide = shared.x1.ceil() as usize - self.left;
        self.high = shared.y1.ceil() as usize - self.top;
        let needed = (self.wide + 1) * self.high;
        if self.changes.len() < needed {
            self.changes.resize(needed, 0.0);
        }
        if self.touched.len() < self.high {
            self.touched.resize(self.high, (usize::MAX, 0));
        }
        let (left, top) = (self.left as f64, self.top as f64);
        self.cut = Some(Rect {
            x0: shared.x0 - left,
            y0: shared.y0 - top,
            x1: shared.x1 - left,
            y1: shared.y1 - top,
        });
        true
    }

    /// A closed shape: an edge from each corner to the next, and from the last to the first.
    fn shape(&mut self, corners: &[Place]) {
        for (i, &corner) in corners.iter().enumerate() {
            self.edge(corner, corners[(i + 1) % corners.len()]);
        }
    }

    /// An edge, cut to the rectangle: what of it lies above or below is left out, and what
    /// lies to the left or the right is moved straight onto the side, which leaves the
    /// winding of every dot inside as it was.
    fn edge(&mut self, a: Place, b: Place) {
        let Some(cut) = self.cut else {
            return;
        };
        let (left, top) = (self.left as f64, self.top as f64);
        let (mut x0, mut y0, mut x1, mut y1) = (a.0 - left, a.1 - top, b.0 - left, b.1 - top);
        if !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) || y0 == y1 {
            return;
        }
        let mut way = 1.0;
        if y0 > y1 {
            (x0, y0, x1, y1) = (x1, y1, x0, y0);
            way = -1.0;
        }
        if y1 <= cut.y0 || y0 >= cut.y1 {
            return;
        }
        let lean = (x1 - x0) / (y1 - y0);
        if y0 < cut.y0 {
            x0 += (cut.y0 - y0) * lean;
            y0 = cut.y0;
        }
        if y1 > cut.y1 {
            y1 = cut.y1;
        }
        // Where the edge crosses the two sides, in order down the page.
        let mut stops = [y0, y1, y1, y1];
        let mut count = 1;
        if lean != 0.0 {
            let mut crossings = [y0 + (cut.x0 - x0) / lean, y0 + (cut.x1 - x0) / lean];
            if crossings[0] > crossings[1] {
                crossings.swap(0, 1);
            }
            for crossing in crossings {
                if crossing > stops[count - 1] && crossing < y1 {
                    stops[count] = crossing;
                    count += 1;
                }
            }
        }
        stops[count] = y1;
        for part in 0..count {
            let (from, to) = (stops[part], stops[part + 1]);
            if to <= from {
                continue;
            }
            let (xa, xb) = (x0 + (from - y0) * lean, x0 + (to - y0) * lean);
            let middle = (xa + xb) / 2.0;
            if middle <= cut.x0 {
                self.piece(cut.x0, from, cut.x0, to, way);
            } else if middle >= cut.x1 {
                self.piece(cut.x1, from, cut.x1, to, way);
            } else {
                self.piece(
                    xa.clamp(cut.x0, cut.x1),
                    from,
                    xb.clamp(cut.x0, cut.x1),
                    to,
                    way,
                );
            }
        }
    }

    /// A piece of an edge that lies inside the rectangle, going down the page.
    fn piece(&mut self, xa: f64, ya: f64, xb: f64, yb: f64, way: f64) {
        let lean = (xb - xa) / (yb - ya);
        let (least, most) = (xa.min(xb), xa.max(xb));
        let stride = self.wide + 1;
        let first = ya.floor() as usize;
        let last = (yb.ceil() as usize).min(self.high);
        for row in first..last {
            let (above, below) = (ya.max(row as f64), yb.min(row as f64 + 1.0));
            let tall = below - above;
            if tall <= 0.0 {
                continue;
            }
            let x_above = (xa + (above - ya) * lean).clamp(least, most);
            let x_below = (xa + (below - ya) * lean).clamp(least, most);
            let (xl, xr) = (x_above.min(x_below), x_above.max(x_below));
            // The area, within this row, between the edge and the upright line at `t` to
            // its right: nothing left of the edge, a triangle while `t` is under it, and
            // beyond it as much again for every dot farther.
            let area = |t: f64| {
                if t <= xl {
                    0.0
                } else if t >= xr {
                    tall * (t - (xl + xr) / 2.0)
                } else {
                    tall * (t - xl) * (t - xl) / (2.0 * (xr - xl))
                }
            };
            let from = xl.floor() as usize;
            let to = (xr.ceil() as usize).min(self.wide);
            let base = row * stride;
            let mut before = 0.0;
            let mut left = area(from as f64);
            for column in from..=to {
                let right = area(column as f64 + 1.0);
                let covered = way * (right - left);
                self.changes[base + column] += covered - before;
                before = covered;
                left = right;
            }
            let (lo, hi) = self.touched[row];
            self.touched[row] = (lo.min(from), hi.max(to + 1));
        }
    }

    /// Gives each dot the shapes cover its place on the page and the share of it they cover,
    /// and leaves the mask empty.
    fn each(&mut self, even_odd: bool, mut dot: impl FnMut(usize, usize, f64)) {
        let stride = self.wide + 1;
        for row in 0..self.high {
            let (lo, hi) = self.touched[row];
            self.touched[row] = (usize::MAX, 0);
            if lo >= hi {
                continue;
            }
            let mut wound = 0.0;
            for column in lo..hi {
                wound += self.changes[row * stride + column];
                self.changes[row * stride + column] = 0.0;
                if column >= self.wide {
                    continue;
                }
                let turns = f64::abs(wound);
                let share = if even_odd {
                    let within = turns % 2.0;
                    if within > 1.0 { 2.0 - within } else { within }
                } else {
                    turns.min(1.0)
                };
                if share > 0.0 {
                    dot(self.left + column, self.top + row, share);
                }
            }
        }
        self.cut = None;
    }
}

/// Twice the area of a shape, above zero when its corners go one way round and below when
/// they go the other.
fn twice_area(corners: &[Place]) -> f64 {
    let mut sum = 0.0;
    for (i, a) in corners.iter().enumerate() {
        let b = corners[(i + 1) % corners.len()];
        sum += a.0 * b.1 - b.0 * a.1;
    }
    sum
}

/// A piece of the ink of a line, its corners one way round, so that pieces that lie over
/// each other add up to one shape; a piece of no area is left out.
fn piece_of_ink(mut corners: Vec<Place>, out: &mut Vec<Vec<Place>>) {
    let twice = twice_area(&corners);
    if twice.is_nan() || twice == 0.0 {
        return;
    }
    if twice < 0.0 {
        corners.reverse();
    }
    out.push(corners);
}

/// The ink of a line through `corners`, `width` wide: a rectangle along each straight piece
/// and a wedge at each corner, with the ends cut square.
fn stroke(corners: &[Place], closed: bool, width: f64, out: &mut Vec<Vec<Place>>) {
    let half = width / 2.0;
    // A width that is no number, none, or below none leaves no ink.
    if half.is_nan() || half <= 0.0 || half.is_infinite() {
        return;
    }
    // The corners, without one that stands where the one before it does.
    let mut path: Vec<Place> = Vec::with_capacity(corners.len());
    for &corner in corners {
        if path.last() != Some(&corner) {
            path.push(corner);
        }
    }
    if closed && path.len() > 1 && path.first() == path.last() {
        path.pop();
    }
    if path.len() < 2 {
        return;
    }
    let pieces = if closed { path.len() } else { path.len() - 1 };
    // Each straight piece: where it starts and ends, and its direction, one long.
    let mut straight: Vec<(Place, Place, Place)> = Vec::with_capacity(pieces);
    for i in 0..pieces {
        let (a, b) = (path[i], path[(i + 1) % path.len()]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let long = sqrt(dx * dx + dy * dy);
        if long.is_nan() || long <= 0.0 {
            continue;
        }
        straight.push((a, b, (dx / long, dy / long)));
    }
    for &(a, b, (ux, uy)) in &straight {
        let (nx, ny) = (-uy * half, ux * half);
        piece_of_ink(
            vec![
                (a.0 + nx, a.1 + ny),
                (b.0 + nx, b.1 + ny),
                (b.0 - nx, b.1 - ny),
                (a.0 - nx, a.1 - ny),
            ],
            out,
        );
    }
    // The corners: between each piece and the next, and for a closed line between the last
    // and the first.
    let corners_between = if closed {
        straight.len()
    } else {
        straight.len().saturating_sub(1)
    };
    for i in 0..corners_between {
        let (_, at, (ux, uy)) = straight[i];
        let (_, _, (vx, vy)) = straight[(i + 1) % straight.len()];
        let turn = ux * vy - uy * vx;
        // Straight on, or straight back: no corner to fill.
        if turn == 0.0 {
            continue;
        }
        // The side away from the turn.
        let side = if turn > 0.0 { -half } else { half };
        let before = (at.0 - uy * side, at.1 + ux * side);
        let after = (at.0 - vy * side, at.1 + vx * side);
        // The point is as far out as one over the cosine of half the turn; it stays within
        // the limit when one plus the cosine of the turn is at least two over its square.
        let together = 1.0 + ux * vx + uy * vy;
        if together >= 2.0 / (MITRE * MITRE) {
            let point = (
                at.0 + (-uy - vy) * side / together,
                at.1 + (ux + vx) * side / together,
            );
            piece_of_ink(vec![at, before, point, after], out);
        } else {
            piece_of_ink(vec![at, before, after], out);
        }
    }
}

/// How many straight lines a curve is made of, from how far its controls bend it, in dots:
/// `order` is 2 for a curve of one control and 3 for one of two.
fn lines_for(bend: f64, order: f64) -> usize {
    let lines = sqrt(order * (order - 1.0) / 8.0 * bend / FLAT).ceil();
    if lines.is_nan() {
        return 1;
    }
    lines.clamp(1.0, MOST_LINES) as usize
}

fn far(a: Place, b: Place, c: Place) -> f64 {
    let (x, y) = (a.0 - 2.0 * b.0 + c.0, a.1 - 2.0 * b.1 + c.1);
    sqrt(x * x + y * y)
}

/// A curve of two controls from `from`, as the ends of straight lines.
fn curve_of_two(from: Place, a: Place, b: Place, to: Place, out: &mut Vec<Place>) {
    let lines = lines_for(far(from, a, b).max(far(a, b, to)), 3.0);
    for i in 1..lines {
        let t = i as f64 / lines as f64;
        let s = 1.0 - t;
        let (k0, k1, k2, k3) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
        out.push((
            k0 * from.0 + k1 * a.0 + k2 * b.0 + k3 * to.0,
            k0 * from.1 + k1 * a.1 + k2 * b.1 + k3 * to.1,
        ));
    }
    out.push(to);
}

/// A curve of one control from `from`, as the ends of straight lines.
fn curve_of_one(from: Place, control: Place, to: Place, out: &mut Vec<Place>) {
    let lines = lines_for(far(from, control, to), 2.0);
    for i in 1..lines {
        let t = i as f64 / lines as f64;
        let s = 1.0 - t;
        let (k0, k1, k2) = (s * s, 2.0 * s * t, t * t);
        out.push((
            k0 * from.0 + k1 * control.0 + k2 * to.0,
            k0 * from.1 + k1 * control.1 + k2 * to.1,
        ));
    }
    out.push(to);
}

/// A traced path in dots: each of its parts as corners, and whether the part is closed.
fn lines_of(trace: &Trace, scale: f64) -> Vec<(Vec<Place>, bool)> {
    let dots = |p: Place| (p.0 * scale, p.1 * scale);
    let mut parts: Vec<(Vec<Place>, bool)> = Vec::new();
    for piece in &trace.pieces {
        match *piece {
            Piece::Move(p) => parts.push((vec![dots(p)], false)),
            Piece::Line(p) => {
                if let Some((corners, _)) = parts.last_mut() {
                    corners.push(dots(p));
                }
            }
            Piece::Curve(a, b, p) => {
                if let Some((corners, _)) = parts.last_mut()
                    && let Some(&from) = corners.last()
                {
                    curve_of_two(from, dots(a), dots(b), dots(p), corners);
                }
            }
            Piece::Close => {
                // What follows a closing without a move starts where the part did.
                if let Some((corners, closed)) = parts.last_mut() {
                    *closed = true;
                    let start = corners[0];
                    parts.push((vec![start], false));
                }
            }
        }
    }
    parts.retain(|(corners, _)| corners.len() > 1);
    parts
}

/// An opacity as a share of one; one for what is no number.
fn share_of(opacity: f64) -> f64 {
    if opacity.is_nan() {
        1.0
    } else {
        opacity.clamp(0.0, 1.0)
    }
}

/// The page being drawn.
struct Page {
    width: usize,
    height: usize,
    /// The red, green and blue of each dot, each of 65,535.
    ink: Vec<u16>,
    /// Dots to a point.
    scale: f64,
    /// Where the first dot of the picture starts on the page, in dots: nothing but for the
    /// picture of a part of a page.
    shift: Place,
    /// What is drawn outside this is not drawn.
    clip: Rect,
    mask: Mask,
}

impl Page {
    /// Lays a colour over a dot in a measure: a share of one.
    fn lay(ink: &mut [u16], width: usize, x: usize, y: usize, colour: [u8; 3], measure: f64) {
        let over = ((measure * 65536.0 + 0.5).floor() as u64).min(65536);
        if over == 0 {
            return;
        }
        let at = (y * width + x) * 3;
        for (part, &new) in colour.iter().enumerate() {
            let under = u64::from(ink[at + part]);
            let new = u64::from(new) * 257;
            ink[at + part] = ((under * (65536 - over) + new * over + 32768) >> 16) as u16;
        }
    }

    /// Closed shapes in one colour.
    fn fill(&mut self, shapes: &[Vec<Place>], colour: [u8; 3], opacity: f64, even_odd: bool) {
        // The picture of a part of a page: every corner counted from where the part starts.
        let moved: Vec<Vec<Place>>;
        let shapes = if self.shift == (0.0, 0.0) {
            shapes
        } else {
            let (x, y) = self.shift;
            moved = shapes
                .iter()
                .map(|shape| shape.iter().map(|p| (p.0 - x, p.1 - y)).collect())
                .collect();
            &moved
        };
        let Some(bounds) = bounds_of(shapes) else {
            return;
        };
        if !self.mask.begin(bounds, self.clip) {
            return;
        }
        for shape in shapes {
            self.mask.shape(shape);
        }
        let (ink, width) = (&mut self.ink, self.width);
        self.mask.each(even_odd, |x, y, share| {
            Page::lay(ink, width, x, y, colour, share * opacity);
        });
    }

    /// The parts of a path, filled and then drawn as a line, as the paint says.
    fn paint(
        &mut self,
        parts: &[(Vec<Place>, bool)],
        fill: &Option<String>,
        line: &Option<Stroke>,
        opacity: f64,
        even_odd: bool,
    ) {
        let opacity = share_of(opacity);
        if let Some(colour) = fill {
            let shapes: Vec<Vec<Place>> = parts.iter().map(|(c, _)| c.clone()).collect();
            self.fill(&shapes, rgb(colour), opacity, even_odd);
        }
        if let Some(line) = line {
            let mut ink = Vec::new();
            for (corners, closed) in parts {
                stroke(corners, *closed, line.width * self.scale, &mut ink);
            }
            self.fill(&ink, rgb(&line.colour), opacity, false);
        }
    }

    /// A rectangle with its sides along the axes, in points, in one colour: each dot in the
    /// measure of its width covered times its height covered.
    fn block(&mut self, at: Place, width: f64, height: f64, colour: [u8; 3]) {
        let s = self.scale;
        let (x, y) = self.shift;
        let block = Rect {
            x0: at.0 * s - x,
            y0: at.1 * s - y,
            x1: (at.0 + width) * s - x,
            y1: (at.1 + height) * s - y,
        };
        let Some(shown) = block.both(self.clip) else {
            return;
        };
        for y in shown.y0.floor() as usize..shown.y1.ceil() as usize {
            let tall = shown.y1.min(y as f64 + 1.0) - shown.y0.max(y as f64);
            for x in shown.x0.floor() as usize..shown.x1.ceil() as usize {
                let across = shown.x1.min(x as f64 + 1.0) - shown.x0.max(x as f64);
                Page::lay(&mut self.ink, self.width, x, y, colour, across * tall);
            }
        }
    }

    fn items(&mut self, items: &[Item]) {
        for item in items {
            self.primitive(&item.primitive);
        }
    }

    fn primitive(&mut self, primitive: &Primitive) {
        let s = self.scale;
        match primitive {
            Primitive::Point {
                at,
                shape,
                size,
                colour,
                opacity,
            } => {
                let outline = geometry(shape, *size);
                let lined = outline.open || matches!(outline.geometry, Geometry::Lines(_));
                let mut trace = Trace::default();
                trace.shape(*at, &outline.geometry);
                let parts = lines_of(&trace, s);
                let (fill, line) = if lined {
                    (
                        None,
                        Some(Stroke {
                            colour: colour.clone(),
                            width: MARK_STROKE,
                        }),
                    )
                } else {
                    (Some(colour.clone()), None)
                };
                self.paint(&parts, &fill, &line, *opacity, false);
            }
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
                self.paint(&lines_of(&trace, s), fill, stroke, *opacity, false);
            }
            Primitive::Polygon {
                rings,
                fill,
                stroke,
                opacity,
            } => {
                let parts: Vec<(Vec<Place>, bool)> = rings
                    .iter()
                    .filter(|ring| ring.len() > 1)
                    .map(|ring| (ring.iter().map(|p| (p.0 * s, p.1 * s)).collect(), true))
                    .collect();
                self.paint(&parts, fill, stroke, *opacity, true);
            }
            Primitive::Rectangle {
                at,
                width,
                height,
                radius,
                fill,
                stroke,
            } => {
                if *radius > 0.0 {
                    let mut trace = Trace::default();
                    trace.rounded(*at, *width, *height, *radius, |p| p);
                    self.paint(&lines_of(&trace, s), fill, stroke, 1.0, false);
                    return;
                }
                if let Some(colour) = fill {
                    self.block(*at, *width, *height, rgb(colour));
                }
                if stroke.is_some() {
                    let mut trace = Trace::default();
                    trace.rounded(*at, *width, *height, 0.0, |p| p);
                    self.paint(&lines_of(&trace, s), &None, stroke, 1.0, false);
                }
            }
            Primitive::Segment {
                from,
                to,
                stroke,
                heads,
            } => {
                let line = vec![(vec![(from.0 * s, from.1 * s), (to.0 * s, to.1 * s)], false)];
                self.paint(&line, &None, &Some(stroke.clone()), 1.0, false);
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
            } => self.cells(*at, *width, *height, *columns, *rows, cells),
            Primitive::Clip {
                at,
                width,
                height,
                items,
            } => {
                let (x, y) = self.shift;
                let region = Rect {
                    x0: at.0 * s - x,
                    y0: at.1 * s - y,
                    x1: (at.0 + width) * s - x,
                    y1: (at.1 + height) * s - y,
                };
                let outside = self.clip;
                // A region that shares nothing with what is already shown shows nothing.
                if let Some(inside) = outside.both(region) {
                    self.clip = inside;
                    self.items(items);
                    self.clip = outside;
                }
            }
        }
    }

    /// The head of an arrow at `tip`, on a line that comes from `tail`: a filled triangle
    /// six strokes long, and at least four points.
    fn arrow(&mut self, tail: Place, tip: Place, stroke: &Stroke) {
        let (dx, dy) = (tip.0 - tail.0, tip.1 - tail.1);
        let run = sqrt(dx * dx + dy * dy);
        if run.is_nan() || run <= 0.0 {
            return;
        }
        let (ux, uy) = (dx / run, dy / run);
        let long = (6.0 * stroke.width).max(4.0);
        let half = 0.35 * long;
        let base = (tip.0 - ux * long, tip.1 - uy * long);
        let s = self.scale;
        let head = vec![
            (tip.0 * s, tip.1 * s),
            ((base.0 - uy * half) * s, (base.1 + ux * half) * s),
            ((base.0 + uy * half) * s, (base.1 - ux * half) * s),
        ];
        self.fill(&[head], rgb(&stroke.colour), 1.0, false);
    }

    /// A line of text: the box behind it, and the outlines of its letters.
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
        let s = self.scale;
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
            self.paint(&lines_of(&trace, s), &frame.fill, &frame.border, 1.0, false);
        }
        let mut along = start;
        for (span, font, scale, width, shaped) in &set {
            let mut shapes: Vec<Vec<Place>> = Vec::new();
            for placed in &shaped.glyphs {
                let origin = along + f64::from(placed.x) * scale;
                let dot = |x: f64, y: f64| {
                    let (px, py) = place(origin + x * scale, y * scale);
                    (px * s, py * s)
                };
                // A glyph whose outline cannot be read is left out; the embedded faces
                // have none.
                let mut open: Vec<Place> = Vec::new();
                for step in font.outline(placed.glyph).unwrap_or_default() {
                    match step {
                        Segment::Move(x, y) => {
                            if open.len() > 2 {
                                shapes.push(std::mem::take(&mut open));
                            }
                            open.clear();
                            open.push(dot(x, y));
                        }
                        Segment::Line(x, y) => open.push(dot(x, y)),
                        Segment::Curve(control, to) => {
                            if let Some(&from) = open.last() {
                                curve_of_one(
                                    from,
                                    dot(control.0, control.1),
                                    dot(to.0, to.1),
                                    &mut open,
                                );
                            }
                        }
                        Segment::Close => {
                            if open.len() > 2 {
                                shapes.push(std::mem::take(&mut open));
                            }
                            open.clear();
                        }
                    }
                }
                if open.len() > 2 {
                    shapes.push(open);
                }
            }
            self.fill(&shapes, rgb(&span.colour), 1.0, false);
            along += width;
        }
    }

    /// A grid of cells: each dot takes the colours of the cells under it, each in the
    /// measure of its share of the dot.
    fn cells(
        &mut self,
        at: Place,
        width: f64,
        height: f64,
        columns: usize,
        rows: usize,
        cells: &[[u8; 3]],
    ) {
        if columns == 0 || rows == 0 {
            return;
        }
        let s = self.scale;
        let (left, top) = (at.0 * s - self.shift.0, at.1 * s - self.shift.1);
        let (wide, high) = (width * s / columns as f64, height * s / rows as f64);
        let grid = Rect {
            x0: left,
            y0: top,
            x1: left + wide * columns as f64,
            y1: top + high * rows as f64,
        };
        if !(wide > 0.0 && high > 0.0) {
            return;
        }
        let Some(shown) = grid.both(self.clip) else {
            return;
        };
        // The cells along one axis that lie under the dot from `from` to one more, within
        // what is shown: each with the length of the dot it lies under.
        let under = |dot: usize, first: f64, size: f64, count: usize, lo: f64, hi: f64| {
            let (from, to) = (lo.max(dot as f64), hi.min(dot as f64 + 1.0));
            let mut shares: Vec<(usize, f64)> = Vec::new();
            if to <= from {
                return shares;
            }
            let start = (((from - first) / size).floor().max(0.0) as usize).min(count - 1);
            for cell in start..count {
                let (a, b) = (first + cell as f64 * size, first + (cell + 1) as f64 * size);
                if a >= to {
                    break;
                }
                let share = b.min(to) - a.max(from);
                if share > 0.0 {
                    shares.push((cell, share));
                }
            }
            shares
        };
        let across: Vec<Vec<(usize, f64)>> = (shown.x0.floor() as usize..shown.x1.ceil() as usize)
            .map(|x| under(x, left, wide, columns, shown.x0, shown.x1))
            .collect();
        let first_column = shown.x0.floor() as usize;
        for y in shown.y0.floor() as usize..shown.y1.ceil() as usize {
            let down = under(y, top, high, rows, shown.y0, shown.y1);
            for (i, along) in across.iter().enumerate() {
                let (mut sum, mut covered) = ([0.0_f64; 3], 0.0);
                for &(row, tall) in &down {
                    for &(column, long) in along {
                        // A cell without a colour is not drawn.
                        if let Some(colour) = cells.get(row * columns + column) {
                            let area = tall * long;
                            covered += area;
                            for part in 0..3 {
                                sum[part] += area * f64::from(colour[part]);
                            }
                        }
                    }
                }
                if covered > 0.0 {
                    let mixed = [
                        (sum[0] / covered + 0.5).floor() as u8,
                        (sum[1] / covered + 0.5).floor() as u8,
                        (sum[2] / covered + 0.5).floor() as u8,
                    ];
                    Page::lay(
                        &mut self.ink,
                        self.width,
                        first_column + i,
                        y,
                        mixed,
                        covered.min(1.0),
                    );
                }
            }
        }
        debug_assert!(self.height >= shown.y1.ceil() as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, side: f64) -> Vec<Place> {
        vec![(x, y), (x + side, y), (x + side, y + side), (x, y + side)]
    }

    #[test]
    fn a_length_is_cut_to_whole_dots_as_r_cuts_it() {
        let mm = |length: f64| length * 72.0 / 25.4;
        assert_eq!(dots_of(mm(89.0), 300.0), 1051.0);
        assert_eq!(dots_of(mm(66.0), 300.0), 779.0);
        assert_eq!(dots_of(mm(183.0), 300.0), 2161.0);
        assert_eq!(dots_of(mm(110.0), 300.0), 1299.0);
        assert_eq!(dots_of(mm(254.0), 300.0), 3000.0);
        assert_eq!(dots_of(mm(143.0), 300.0), 1688.0);
        assert_eq!(dots_of(72.0, 72.0), 72.0);
        assert_eq!(dots_of(0.1, 72.0), 0.0);
    }

    #[test]
    fn a_square_on_the_dots_covers_them_whole_and_one_between_them_by_its_area() {
        let on = coverage(&[square(1.0, 1.0, 2.0)], false, 4, 4);
        assert_eq!(
            on,
            [
                0.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 1.0, 0.0, //
                0.0, 1.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 0.0,
            ]
        );
        let between = coverage(&[square(0.5, 0.5, 2.0)], false, 3, 3);
        assert_eq!(between, [0.25, 0.5, 0.25, 0.5, 1.0, 0.5, 0.25, 0.5, 0.25]);
        // The other way round, the same.
        let mut back = square(0.5, 0.5, 2.0);
        back.reverse();
        assert_eq!(coverage(&[back], false, 3, 3), between);
    }

    #[test]
    fn a_triangle_covers_each_dot_by_the_area_of_it_inside() {
        // Half of a square of 4 by 4, cut along its diagonal from the top left.
        let half = coverage(&[vec![(0.0, 0.0), (4.0, 4.0), (0.0, 4.0)]], false, 4, 4);
        for y in 0..4 {
            for x in 0..4 {
                let wanted = match x.cmp(&y) {
                    std::cmp::Ordering::Less => 1.0,
                    std::cmp::Ordering::Equal => 0.5,
                    std::cmp::Ordering::Greater => 0.0,
                };
                assert_eq!(half[y * 4 + x], wanted, "{x},{y}");
            }
        }
        // A thin triangle across many dots of one row: its area, shared out.
        let thin = coverage(&[vec![(0.0, 0.0), (8.0, 1.0), (0.0, 1.0)]], false, 8, 1);
        assert!((thin.iter().sum::<f64>() - 4.0).abs() < 1e-12);
        for (x, share) in thin.iter().enumerate() {
            assert!(
                (share - (1.0 - (x as f64 + 0.5) / 8.0)).abs() < 1e-12,
                "{x}"
            );
        }
    }

    #[test]
    fn a_hole_is_outside_by_the_rule_of_odd_and_even_whichever_way_it_goes_round() {
        let outer = square(0.0, 0.0, 6.0);
        let mut hole = square(1.5, 1.5, 3.0);
        for _ in 0..2 {
            let both = [outer.clone(), hole.clone()];
            let cut = coverage(&both, true, 6, 6);
            assert!((cut.iter().sum::<f64>() - 27.0).abs() < 1e-12);
            assert_eq!(cut[3 * 6 + 3], 0.0);
            assert_eq!(cut[0], 1.0);
            // The dot the corner of the hole stands in is a quarter inside the hole.
            assert_eq!(cut[6 + 1], 0.75);
            hole.reverse();
        }
        // By the rule of winding, a hole that goes the same way round is filled.
        let filled = coverage(&[outer.clone(), square(1.5, 1.5, 3.0)], false, 6, 6);
        assert_eq!(filled.iter().sum::<f64>(), 36.0);
        let mut back = square(1.5, 1.5, 3.0);
        back.reverse();
        let cut = coverage(&[outer, back], false, 6, 6);
        assert!((cut.iter().sum::<f64>() - 27.0).abs() < 1e-12);
    }

    #[test]
    fn what_lies_beyond_the_page_is_cut_and_what_is_inside_is_as_it_was() {
        // A square that reaches beyond every side of a page of 3 by 3.
        let over = coverage(&[square(-5.0, -5.0, 20.0)], false, 3, 3);
        assert_eq!(over, [1.0; 9]);
        // A diamond with its corners outside: the page is inside it but for half of each of
        // its corner dots.
        let diamond = vec![(1.5, -0.5), (3.5, 1.5), (1.5, 3.5), (-0.5, 1.5)];
        let cut = coverage(&[diamond], false, 3, 3);
        assert!((cut.iter().sum::<f64>() - 7.0).abs() < 1e-12, "{cut:?}");
        assert_eq!(cut[4], 1.0);
        assert!((cut[0] - 0.5).abs() < 1e-12);
        // Wholly outside: nothing.
        assert_eq!(coverage(&[square(10.0, 0.0, 2.0)], false, 3, 3), [0.0; 9]);
        assert_eq!(coverage(&[square(-10.0, 0.0, 2.0)], false, 3, 3), [0.0; 9]);
        assert_eq!(
            coverage(&[square(0.0, f64::NAN, 2.0)], false, 3, 3),
            [0.0; 9]
        );
    }

    #[test]
    fn the_ink_of_a_line_is_as_long_and_as_wide_as_the_line() {
        let area = |shapes: &[Vec<Place>]| -> f64 { coverage(shapes, false, 40, 40).iter().sum() };
        // A straight line of 20 by 3, at a slant: its ends cut square.
        let slant = ink_of(&[(vec![(5.0, 5.0), (17.0, 21.0)], false)], 3.0);
        assert_eq!(slant.len(), 1);
        assert!((area(&slant) - 60.0).abs() < 1e-9);
        // A right corner is drawn out to a point: two lines of 10 by 2, which share a square
        // of 1 inside the corner, and the square of 1 that fills it outside.
        let corner = ink_of(&[(vec![(5.0, 5.0), (15.0, 5.0), (15.0, 15.0)], false)], 2.0);
        assert_eq!(corner.len(), 3);
        assert!(
            (area(&corner) - (20.0 + 20.0 - 1.0 + 1.0)).abs() < 1e-9,
            "{}",
            area(&corner)
        );
        // A closed square of 10 with a line of 2: the ring between squares of 12 and of 8.
        let ring = ink_of(&[(square(10.0, 10.0, 10.0), true)], 2.0);
        assert_eq!(ring.len(), 8);
        assert!(
            (area(&ring) - (144.0 - 64.0)).abs() < 1e-9,
            "{}",
            area(&ring)
        );
        // A corner too sharp for its point is cut flat: less ink than the point would be.
        let sharp = ink_of(
            &[(vec![(5.0, 20.0), (30.0, 20.0), (5.0, 22.0)], false)],
            2.0,
        );
        assert_eq!(sharp.last().map(Vec::len), Some(3));
        // No width, one corner, corners that stand on each other: no ink.
        assert!(ink_of(&[(vec![(5.0, 5.0), (15.0, 5.0)], false)], 0.0).is_empty());
        assert!(ink_of(&[(vec![(5.0, 5.0)], false)], 2.0).is_empty());
        assert!(ink_of(&[(vec![(5.0, 5.0), (5.0, 5.0)], true)], 2.0).is_empty());
        // Straight on through a corner that is none: two rectangles and no wedge.
        let on = ink_of(&[(vec![(5.0, 5.0), (10.0, 5.0), (15.0, 5.0)], false)], 2.0);
        assert_eq!(on.len(), 2);
        assert!((area(&on) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn a_curve_is_made_of_lines_that_stay_a_twentieth_of_a_dot_from_it() {
        // A quarter of a circle of radius 100 dots: every corner within the tolerance of it.
        let mut trace = Trace::default();
        trace.circle((0.0, 0.0), 100.0);
        let parts = lines_of(&trace, 1.0);
        assert_eq!(parts.len(), 1);
        let (corners, closed) = &parts[0];
        assert!(*closed && corners.len() > 40, "{}", corners.len());
        for &(x, y) in corners {
            let away = (x * x + y * y).sqrt();
            assert!((away - 100.0).abs() < 0.05, "{away}");
        }
        // Halfway between two corners the straight line is no farther from the circle.
        for pair in corners.windows(2) {
            let (x, y) = ((pair[0].0 + pair[1].0) / 2.0, (pair[0].1 + pair[1].1) / 2.0);
            assert!((100.0 - (x * x + y * y).sqrt()).abs() < 2.0 * FLAT);
        }
        // The area of the circle in dots, to a part in a thousand: the straight lines lie
        // inside it.
        let disc: f64 = coverage(
            &[corners.iter().map(|p| (p.0 + 110.0, p.1 + 110.0)).collect()],
            false,
            220,
            220,
        )
        .iter()
        .sum();
        assert!(
            (disc / (std::f64::consts::PI * 10_000.0) - 1.0).abs() < 1e-3,
            "{disc}"
        );
        assert_eq!(lines_for(0.0, 3.0), 1);
        assert_eq!(lines_for(f64::NAN, 3.0), 1);
        assert_eq!(lines_for(1e30, 2.0), 512);
    }

    #[test]
    fn paint_is_laid_over_what_is_under_it_in_its_measure() {
        let mut ink = vec![u16::MAX; 3];
        Page::lay(&mut ink, 1, 0, 0, [0, 0, 0], 0.5);
        assert_eq!(ink, [32768, 32768, 32768]);
        Page::lay(&mut ink, 1, 0, 0, [255, 0, 51], 1.0);
        assert_eq!(ink, [65535, 0, 51 * 257]);
        Page::lay(&mut ink, 1, 0, 0, [0, 0, 0], 0.0);
        assert_eq!(ink, [65535, 0, 51 * 257]);
        Page::lay(&mut ink, 1, 0, 0, [0, 0, 0], 7.0);
        assert_eq!(ink, [0, 0, 0]);
        assert_eq!(share_of(f64::NAN), 1.0);
        assert_eq!(share_of(-1.0), 0.0);
    }
}
