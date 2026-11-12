//! A path as it is traced (release 0.7.0, steps 22b1 and 22b2): what the writers of figures
//! that draw with lines and curves share, so that the PDF of a drawing and its picture in
//! dots have the same arcs, the same circles and the same round corners.
//!
//! A [`Trace`] takes the steps of a path and keeps them as moves, straight lines, cubic curves
//! and closings, in the units it is given. An arc of an ellipse becomes curves of at most a
//! quarter turn each, a circle four of them, and a rectangle with round corners four lines and
//! four curves. A path that does not begin with a move begins where it first goes.

use zeff_numeric::maths::{atan2, cos, sin, sqrt, tan};

use crate::drawing::{Place, Step};
use crate::shape::Geometry;
use crate::svg::turned;

/// How far the control of a quarter of a circle stands from its end, as a share of the radius.
pub(crate) const QUARTER: f64 = 0.552_284_749_830_793_6;

/// One piece of a traced path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Piece {
    Move(Place),
    Line(Place),
    /// Two controls and the end.
    Curve(Place, Place, Place),
    Close,
}

/// A path being traced: its pieces, where it stands and where its part began.
#[derive(Default)]
pub(crate) struct Trace {
    pub(crate) pieces: Vec<Piece>,
    at: Place,
    start: Place,
    pub(crate) begun: bool,
}

impl Trace {
    pub(crate) fn to(&mut self, p: Place) {
        self.pieces.push(Piece::Move(p));
        self.at = p;
        self.start = p;
        self.begun = true;
    }

    pub(crate) fn line(&mut self, p: Place) {
        // A path must begin somewhere: one that does not begins where it first goes.
        if !self.begun {
            return self.to(p);
        }
        self.pieces.push(Piece::Line(p));
        self.at = p;
    }

    pub(crate) fn curve(&mut self, a: Place, b: Place, p: Place) {
        if !self.begun {
            return self.to(p);
        }
        self.pieces.push(Piece::Curve(a, b, p));
        self.at = p;
    }

    pub(crate) fn close(&mut self) {
        if self.begun {
            self.pieces.push(Piece::Close);
            self.at = self.start;
        }
    }

    /// An arc of an ellipse from where the path stands to `to`, as an SVG path has it,
    /// drawn as curves of at most a quarter turn each.
    pub(crate) fn arc(
        &mut self,
        radii: (f64, f64),
        turn: f64,
        larger: bool,
        clockwise: bool,
        to: Place,
    ) {
        let from = self.at;
        if !self.begun {
            return self.to(to);
        }
        if from == to {
            return;
        }
        let (mut rx, mut ry) = (radii.0.abs(), radii.1.abs());
        // An ellipse of no width is the straight line between the ends.
        if !(rx > 0.0 && ry > 0.0 && rx.is_finite() && ry.is_finite()) {
            return self.line(to);
        }
        // The ends as the ellipse sees them: halved, and turned back by its turn.
        let (cosine, sine) = turned(turn);
        let (hx, hy) = ((from.0 - to.0) / 2.0, (from.1 - to.1) / 2.0);
        let (x1, y1) = (cosine * hx + sine * hy, -sine * hx + cosine * hy);
        // Radii too small to reach from one end to the other are grown until they do.
        let reach = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
        if reach > 1.0 {
            let grow = sqrt(reach);
            rx *= grow;
            ry *= grow;
        }
        // The centre, on the side the two choices name.
        let below = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let above = (rx * rx * ry * ry - below).max(0.0);
        let mut side = if below > 0.0 {
            sqrt(above / below)
        } else {
            0.0
        };
        if larger == clockwise {
            side = -side;
        }
        let (cx1, cy1) = (side * rx * y1 / ry, -side * ry * x1 / rx);
        let centre = (
            cosine * cx1 - sine * cy1 + (from.0 + to.0) / 2.0,
            sine * cx1 + cosine * cy1 + (from.1 + to.1) / 2.0,
        );
        // The angle the arc starts at and how far it sweeps.
        let first = atan2((y1 - cy1) / ry, (x1 - cx1) / rx);
        let last = atan2((-y1 - cy1) / ry, (-x1 - cx1) / rx);
        let whole = 2.0 * std::f64::consts::PI;
        let mut sweep = last - first;
        if clockwise && sweep < 0.0 {
            sweep += whole;
        } else if !clockwise && sweep > 0.0 {
            sweep -= whole;
        }
        let pieces = (sweep.abs() / (whole / 4.0) - 1e-9).ceil().max(1.0) as usize;
        let each = sweep / pieces as f64;
        let pull = 4.0 / 3.0 * tan(each / 4.0);
        // A place on the ellipse and the direction the arc leaves it in.
        let on = |angle: f64| {
            let (c, s) = (cos(angle), sin(angle));
            (
                (
                    centre.0 + rx * c * cosine - ry * s * sine,
                    centre.1 + rx * c * sine + ry * s * cosine,
                ),
                (
                    -rx * s * cosine - ry * c * sine,
                    -rx * s * sine + ry * c * cosine,
                ),
            )
        };
        let (_, mut going) = on(first);
        let mut here = from;
        for piece in 1..=pieces {
            let (mut there, coming) = on(first + each * piece as f64);
            // The arc ends where it was told to, not a rounding away from it.
            if piece == pieces {
                there = to;
            }
            self.curve(
                (here.0 + pull * going.0, here.1 + pull * going.1),
                (there.0 - pull * coming.0, there.1 - pull * coming.1),
                there,
            );
            here = there;
            going = coming;
        }
    }

    pub(crate) fn step(&mut self, step: &Step) {
        match *step {
            Step::Move(to) => self.to(to),
            Step::Line(to) => self.line(to),
            Step::Curve(a, b, to) => self.curve(a, b, to),
            Step::Arc {
                radii,
                turn,
                larger,
                clockwise,
                to,
            } => self.arc(radii, turn, larger, clockwise, to),
            Step::Close => self.close(),
        }
    }

    /// A circle, as four quarters.
    pub(crate) fn circle(&mut self, centre: Place, radius: f64) {
        let (x, y) = centre;
        let (r, k) = (radius, radius * QUARTER);
        self.to((x + r, y));
        self.curve((x + r, y + k), (x + k, y + r), (x, y + r));
        self.curve((x - k, y + r), (x - r, y + k), (x - r, y));
        self.curve((x - r, y - k), (x - k, y - r), (x, y - r));
        self.curve((x + k, y - r), (x + r, y - k), (x + r, y));
        self.close();
    }

    /// A rectangle with its corners rounded by `radius`, each of its places taken `through`
    /// a turn before it is written.
    pub(crate) fn rounded(
        &mut self,
        at: Place,
        width: f64,
        height: f64,
        radius: f64,
        through: impl Fn(Place) -> Place,
    ) {
        let (x, y) = at;
        let r = radius.min(width / 2.0).min(height / 2.0).max(0.0);
        if r.is_nan() || r == 0.0 {
            self.to(through((x, y)));
            self.line(through((x + width, y)));
            self.line(through((x + width, y + height)));
            self.line(through((x, y + height)));
            return self.close();
        }
        let k = r * (1.0 - QUARTER);
        let (right, bottom) = (x + width, y + height);
        self.to(through((x + r, y)));
        self.line(through((right - r, y)));
        self.curve(
            through((right - k, y)),
            through((right, y + k)),
            through((right, y + r)),
        );
        self.line(through((right, bottom - r)));
        self.curve(
            through((right, bottom - k)),
            through((right - k, bottom)),
            through((right - r, bottom)),
        );
        self.line(through((x + r, bottom)));
        self.curve(
            through((x + k, bottom)),
            through((x, bottom - k)),
            through((x, bottom - r)),
        );
        self.line(through((x, y + r)));
        self.curve(
            through((x, y + k)),
            through((x + k, y)),
            through((x + r, y)),
        );
        self.close();
    }

    /// The geometry of a mark, centred at a place.
    pub(crate) fn shape(&mut self, centre: Place, geometry: &Geometry) {
        let (x, y) = centre;
        match geometry {
            Geometry::Circle(radius) => self.circle(centre, *radius),
            Geometry::Polygon(corners) => {
                for (i, corner) in corners.iter().enumerate() {
                    let p = (x + corner.0, y + corner.1);
                    if i == 0 { self.to(p) } else { self.line(p) }
                }
                self.close();
            }
            Geometry::Lines(strokes) => {
                for (from, to) in strokes {
                    self.to((x + from.0, y + from.1));
                    self.line((x + to.0, y + to.1));
                }
            }
        }
    }
}
