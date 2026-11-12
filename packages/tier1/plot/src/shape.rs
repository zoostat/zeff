//! The shapes of a mark (Language Reference, section 7.6, "Mappings and constants" and
//! "Scales"): the eight given to categories, in a fixed order, the one of `NA`, and the names a
//! constant shape may be written by. The first six date from step 20d; the seventh and eighth,
//! `triangle_down` and `star`, are the owner's ruling of 12 November 2026 on question 2 of open
//! decision #110, so that each of the eight colours of the palette has a shape of its own.
//!
//! A shape is a name, and the writers draw it: a scene never holds an outline. A mark of size
//! `s` is a circle `s` points across. Any other shape with an inside (`triangle`, `square`,
//! `diamond`, `triangle_down`, `star`, and those five and the circle `_open`, which are the
//! outline alone) is drawn with the area of that circle, so that the shapes of one size weigh
//! the same to the eye; a shape of lines alone (`plus`, `cross`, `asterisk`) is drawn `s`
//! points across. The triangle stands on its base and `triangle_down` on its point, the
//! diamond is a square on its corner, the star has five points with one upwards, `plus` is an
//! upright cross, `cross` the same turned by 45 degrees, and `asterisk` the two together.

use zeff_diagnostics::did_you_mean;

/// The shapes of categories, in the order they are given out: the four with an inside first,
/// which are the easiest to tell apart when they are small, then the two of lines, then the
/// two that are nearest to an earlier one (a triangle on its point, and a star, which is not
/// the asterisk of `NA`), so that they are met only by a seventh and an eighth category.
pub const PALETTE: [&str; 8] = [
    "circle",
    "triangle",
    "square",
    "diamond",
    "plus",
    "cross",
    "triangle_down",
    "star",
];

/// The shape of the category `NA`, which no other category is given.
pub const NA_SHAPE: &str = "asterisk";

/// The shape of a mark when nothing is said of `shape`.
pub const DEFAULT: &str = "circle";

/// The names a constant shape may be written by, in the order of their names.
pub const NAMED: [&str; 15] = [
    "asterisk",
    "circle",
    "circle_open",
    "cross",
    "diamond",
    "diamond_open",
    "plus",
    "square",
    "square_open",
    "star",
    "star_open",
    "triangle",
    "triangle_down",
    "triangle_down_open",
    "triangle_open",
];

/// The shape of the category at `position`, counted from 0: the eight in turn, and again from
/// the first for a ninth category.
pub fn category(position: usize) -> &'static str {
    PALETTE[position % PALETTE.len()]
}

/// The shape a string names, or the nearest name there is, if one is near.
pub fn parse(written: &str) -> Result<&'static str, Option<&'static str>> {
    match NAMED.binary_search(&written) {
        Ok(found) => Ok(NAMED[found]),
        Err(_) => Err(did_you_mean(written, NAMED.iter().copied())),
    }
}

/// How a shape is drawn, about its centre, in points, the y axis downwards as on a page.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    /// A circle of this radius.
    Circle(f64),
    /// A closed outline through these corners.
    Polygon(Vec<(f64, f64)>),
    /// Strokes, each from one end to the other.
    Lines(Vec<((f64, f64), (f64, f64))>),
}

/// A shape of a size: its geometry, and whether it is the outline alone (`_open`).
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    pub geometry: Geometry,
    pub open: bool,
}

// The sines and cosines of 18 and 54 degrees, written out so that a star has the same corners
// on every machine.
const SIN_18: f64 = 0.309_016_994_374_947_4;
const COS_18: f64 = 0.951_056_516_295_153_5;
const SIN_54: f64 = 0.809_016_994_374_947_4;
const COS_54: f64 = 0.587_785_252_292_473_1;

/// The inner corners of the star stand at this share of the outer ones, the share of the
/// five-pointed star drawn with five straight strokes: (3 − √5) / 2.
const STAR_WAIST: f64 = 0.381_966_011_250_105_1;

/// The shape `name` at `size` points: a circle `size` across; any other shape with an inside
/// with the area of that circle, so that the shapes of one size weigh the same to the eye
/// (question 5 of open decision #110); a shape of lines `size` across. `None` for a name that
/// is no shape.
pub fn outline(name: &str, size: f64) -> Option<Outline> {
    let (solid, open) = match name.strip_suffix("_open") {
        Some(solid) => (solid, true),
        None => (name, false),
    };
    let radius = size / 2.0;
    let area = std::f64::consts::PI * radius * radius;
    let geometry = match solid {
        "circle" => Geometry::Circle(radius),
        "square" => {
            let half = area.sqrt() / 2.0;
            Geometry::Polygon(vec![
                (-half, -half),
                (half, -half),
                (half, half),
                (-half, half),
            ])
        }
        // A square on its corner: its area is twice the square of the reach of a corner.
        "diamond" => {
            let reach = (area / 2.0).sqrt();
            Geometry::Polygon(vec![
                (0.0, -reach),
                (reach, 0.0),
                (0.0, reach),
                (-reach, 0.0),
            ])
        }
        // Three equal sides; the centre is the point the three medians meet at, a third of
        // the height above the base.
        "triangle" | "triangle_down" => {
            let side = (4.0 * area / 3.0_f64.sqrt()).sqrt();
            let height = side * 3.0_f64.sqrt() / 2.0;
            let (tip, base) = (-2.0 * height / 3.0, height / 3.0);
            let up = vec![(0.0, tip), (side / 2.0, base), (-side / 2.0, base)];
            if solid == "triangle" {
                Geometry::Polygon(up)
            } else {
                Geometry::Polygon(up.into_iter().map(|(x, y)| (x, -y)).collect())
            }
        }
        // Ten corners, outer and inner in turn, the first at the top. The area of such a
        // star is five times the outer reach, the inner reach and the sine of 36 degrees.
        "star" => {
            let outer = (area / (5.0 * STAR_WAIST * COS_54)).sqrt();
            let inner = outer * STAR_WAIST;
            Geometry::Polygon(vec![
                (0.0, -outer),
                (inner * COS_54, -inner * SIN_54),
                (outer * COS_18, -outer * SIN_18),
                (inner * COS_18, inner * SIN_18),
                (outer * COS_54, outer * SIN_54),
                (0.0, inner),
                (-outer * COS_54, outer * SIN_54),
                (-inner * COS_18, inner * SIN_18),
                (-outer * COS_18, -outer * SIN_18),
                (-inner * COS_54, -inner * SIN_54),
            ])
        }
        "plus" | "cross" | "asterisk" if !open => {
            let slant = radius / 2.0_f64.sqrt();
            let upright = [
                ((-radius, 0.0), (radius, 0.0)),
                ((0.0, -radius), (0.0, radius)),
            ];
            let turned = [
                ((-slant, -slant), (slant, slant)),
                ((-slant, slant), (slant, -slant)),
            ];
            Geometry::Lines(match solid {
                "plus" => upright.to_vec(),
                "cross" => turned.to_vec(),
                _ => upright.into_iter().chain(turned).collect(),
            })
        }
        _ => return None,
    };
    Some(Outline { geometry, open })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_in_order_and_each_is_itself() {
        assert!(NAMED.windows(2).all(|pair| pair[0] < pair[1]));
        for name in NAMED {
            assert_eq!(parse(name), Ok(name));
        }
    }

    #[test]
    fn the_shapes_of_categories_are_eight_different_names_and_na_has_its_own() {
        assert_eq!(PALETTE.len(), crate::colour::OKABE_ITO.len());
        // The six of step 20d keep their places.
        assert_eq!(
            PALETTE[..6],
            ["circle", "triangle", "square", "diamond", "plus", "cross"]
        );
        assert_eq!(PALETTE[6..], ["triangle_down", "star"]);
        for (i, shape) in PALETTE.iter().enumerate() {
            assert_eq!(parse(shape), Ok(*shape));
            assert!(!PALETTE[..i].contains(shape));
        }
        assert!(!PALETTE.contains(&NA_SHAPE));
        assert_eq!(parse(NA_SHAPE), Ok(NA_SHAPE));
        assert_eq!(PALETTE[0], DEFAULT);
    }

    #[test]
    fn the_shapes_repeat_from_the_ninth_category() {
        assert_eq!(category(0), "circle");
        assert_eq!(category(5), "cross");
        assert_eq!(category(6), "triangle_down");
        assert_eq!(category(7), "star");
        assert_eq!(category(8), "circle");
        assert_eq!(category(17), "triangle");
    }

    #[test]
    fn every_shape_with_an_inside_has_its_outline_and_no_shape_of_lines_has() {
        for solid in [
            "circle",
            "triangle",
            "square",
            "diamond",
            "triangle_down",
            "star",
        ] {
            let open = format!("{solid}_open");
            assert_eq!(
                parse(&open),
                Ok(NAMED[NAMED.binary_search(&open.as_str()).unwrap()])
            );
        }
        for lines in ["plus", "cross", "asterisk"] {
            assert!(parse(&format!("{lines}_open")).is_err());
        }
    }

    #[test]
    fn a_name_that_is_not_a_shape_is_refused_with_the_nearest() {
        assert_eq!(parse("sqaure"), Err(Some("square")));
        assert_eq!(parse("Circle"), Err(Some("circle")));
        assert_eq!(parse("circle open"), Err(Some("circle_open")));
        assert_eq!(parse("triangle down"), Err(Some("triangle_down")));
        assert_eq!(parse("stars"), Err(Some("star")));
        assert_eq!(parse(""), Err(None));
        assert_eq!(parse("steelblue"), Err(None));
    }

    /// The area inside corners, by the surveyor's rule.
    fn area(corners: &[(f64, f64)]) -> f64 {
        let n = corners.len();
        let twice: f64 = (0..n)
            .map(|i| {
                let (a, b) = (corners[i], corners[(i + 1) % n]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum();
        twice.abs() / 2.0
    }

    #[test]
    fn every_name_has_a_geometry_and_the_solid_shapes_of_one_size_have_one_area() {
        for size in [0.5, 4.5, 8.0, 18.0] {
            let circle = std::f64::consts::PI * size * size / 4.0;
            for name in NAMED {
                let drawn = outline(name, size).unwrap_or_else(|| panic!("no geometry for {name}"));
                assert_eq!(drawn.open, name.ends_with("_open"), "{name}");
                match &drawn.geometry {
                    Geometry::Circle(radius) => assert_eq!(*radius, size / 2.0),
                    Geometry::Polygon(corners) => {
                        let inside = area(corners);
                        assert!(
                            (inside - circle).abs() <= 1e-12 * circle,
                            "{name} at {size}: {inside} against {circle}"
                        );
                        // Balanced left and right.
                        let lean: f64 = corners.iter().map(|c| c.0).sum();
                        assert!(lean.abs() <= 1e-12 * size, "{name}");
                    }
                    Geometry::Lines(strokes) => {
                        // Every stroke passes through the centre and is `size` long.
                        for (from, to) in strokes {
                            assert!(
                                (from.0 + to.0).abs() <= 1e-12 && (from.1 + to.1).abs() <= 1e-12
                            );
                            let length = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
                            assert!((length - size).abs() <= 1e-12 * size, "{name}");
                        }
                    }
                }
            }
        }
        assert_eq!(outline("hexagon", 4.5), None);
        assert_eq!(outline("plus_open", 4.5), None);
        assert_eq!(outline("", 4.5), None);
    }

    #[test]
    fn a_triangle_stands_on_its_base_and_the_other_on_its_point_and_a_star_points_up() {
        let corners = |name: &str| match outline(name, 10.0).unwrap().geometry {
            Geometry::Polygon(corners) => corners,
            other => panic!("{name}: {other:?}"),
        };
        // On a page y grows downwards: the tip is above the base.
        let up = corners("triangle");
        assert_eq!(up.len(), 3);
        assert!(up[0].1 < 0.0 && up[1].1 > 0.0 && up[1].1 == up[2].1);
        let down = corners("triangle_down");
        assert!(down[0].1 > 0.0 && down[1].1 < 0.0 && down[1].1 == down[2].1);
        assert_eq!(down[0].1, -up[0].1);
        // The centre is where the medians meet: the heights of the corners sum to nothing.
        assert!((up.iter().map(|c| c.1).sum::<f64>()).abs() <= 1e-12);
        let star = corners("star");
        assert_eq!(star.len(), 10);
        assert_eq!(star[0].0, 0.0);
        assert!(
            star.iter().all(|c| c.1 >= star[0].1),
            "the first corner is the top"
        );
        // The outer corners are all as far from the centre, and the inner ones nearer.
        let reach = |c: &(f64, f64)| (c.0 * c.0 + c.1 * c.1).sqrt();
        for pair in star.chunks(2) {
            assert!((reach(&pair[0]) - reach(&star[0])).abs() <= 1e-12);
            assert!((reach(&pair[1]) - reach(&star[1])).abs() <= 1e-12);
            assert!(reach(&pair[1]) < reach(&pair[0]) / 2.0);
        }
        // A diamond is as wide as it is tall, a square has four equal sides.
        let diamond = corners("diamond");
        assert_eq!(diamond[1].0, diamond[2].1);
        let square = corners("square");
        assert_eq!(square[1].0 - square[0].0, square[2].1 - square[1].1);
        // An outline has the geometry of its solid shape.
        assert_eq!(
            outline("star_open", 6.0).unwrap().geometry,
            outline("star", 6.0).unwrap().geometry
        );
    }
}
