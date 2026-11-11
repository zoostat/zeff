//! The shapes of a mark (Language Reference, section 7.6, "Mappings and constants" and
//! "Scales"): the six given to categories, in a fixed order, the one of `NA`, and the names a
//! constant shape may be written by.
//!
//! A shape is a name, and the writers draw it: a scene never holds an outline. A mark of size
//! `s` is a circle `s` points across. Any other shape with an inside (`triangle`, `square`,
//! `diamond`, and those three and the circle `_open`, which are the outline alone) is drawn with
//! the area of that circle, so that the shapes of one size weigh the same to the eye; a shape of
//! lines alone (`plus`, `cross`, `asterisk`) is drawn `s` points across. The triangle stands on
//! its base, the diamond is a square on its corner, `plus` is an upright cross, `cross` the same
//! turned by 45 degrees, and `asterisk` the two together.

use zeff_diagnostics::did_you_mean;

/// The shapes of categories, in the order they are given out: the four with an inside first,
/// which are the easiest to tell apart when they are small, then the two of lines.
pub const PALETTE: [&str; 6] = ["circle", "triangle", "square", "diamond", "plus", "cross"];

/// The shape of the category `NA`, which no other category is given.
pub const NA_SHAPE: &str = "asterisk";

/// The shape of a mark when nothing is said of `shape`.
pub const DEFAULT: &str = "circle";

/// The names a constant shape may be written by, in the order of their names.
pub const NAMED: [&str; 11] = [
    "asterisk",
    "circle",
    "circle_open",
    "cross",
    "diamond",
    "diamond_open",
    "plus",
    "square",
    "square_open",
    "triangle",
    "triangle_open",
];

/// The shape of the category at `position`, counted from 0: the six in turn, and again from the
/// first for a seventh category.
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
    fn the_shapes_of_categories_are_six_different_names_and_na_has_its_own() {
        for (i, shape) in PALETTE.iter().enumerate() {
            assert_eq!(parse(shape), Ok(*shape));
            assert!(!PALETTE[..i].contains(shape));
        }
        assert!(!PALETTE.contains(&NA_SHAPE));
        assert_eq!(parse(NA_SHAPE), Ok(NA_SHAPE));
        assert_eq!(PALETTE[0], DEFAULT);
    }

    #[test]
    fn the_shapes_repeat_from_the_seventh_category() {
        assert_eq!(category(0), "circle");
        assert_eq!(category(5), "cross");
        assert_eq!(category(6), "circle");
        assert_eq!(category(13), "triangle");
    }

    #[test]
    fn a_name_that_is_not_a_shape_is_refused_with_the_nearest() {
        assert_eq!(parse("sqaure"), Err(Some("square")));
        assert_eq!(parse("Circle"), Err(Some("circle")));
        assert_eq!(parse("circle open"), Err(Some("circle_open")));
        assert_eq!(parse(""), Err(None));
        assert_eq!(parse("steelblue"), Err(None));
    }
}
