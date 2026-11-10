//! The linear scale (Language Reference, section 7.6): the limits of an axis, its breaks and the
//! labels of the breaks.
//!
//! The limits are the extent of the values drawn, widened by 5% of its width on each side, so
//! that no mark sits on the edge of the panel. The breaks are the multiples of 1, 2 or 5 times a
//! power of ten that give about five intervals between the limits, by the rule of d3 and
//! Observable Plot, whose defaults the section follows. A break is read from its decimal text
//! (`15e-2` for 0.15), which is correctly rounded, so that 0.3 is the double nearest 0.3 and not
//! three times the double nearest 0.1; no logarithm and no power is computed. The labels are the
//! numbers as tables print them (section 3.8).

use std::f64::consts::SQRT_2;

use zeff_core::display::format_float;

use crate::json;

/// About this many intervals between the breaks of an axis.
pub const INTERVALS: f64 = 5.0;

/// The share of the extent added below and above it.
pub const MARGIN: f64 = 0.05;

/// The square roots of 10 and of 50: the geometric means of 2 and 5 and of 5 and 10, where the
/// step changes (that of 1 and 2 is the square root of 2).
const SQRT_10: f64 = 3.162_277_660_168_379_5;
const SQRT_50: f64 = 7.071_067_811_865_475_5;

/// The limits of an axis whose values run from `extent.0` to `extent.1`: the extent with 5% of
/// its width added on each side. An extent of one value `v` is widened by the larger of 0.5 and
/// 5% of |v|, and with no value at all the limits are 0 and 1.
pub fn limits(extent: Option<(f64, f64)>) -> (f64, f64) {
    let Some((lo, hi)) = extent else {
        return (0.0, 1.0);
    };
    if !(lo.is_finite() && hi.is_finite() && lo <= hi) {
        return (0.0, 1.0);
    }
    let (lo, hi) = if lo == hi {
        let half = (MARGIN * lo.abs()).max(0.5);
        (lo - half, hi + half)
    } else {
        let pad = MARGIN * hi - MARGIN * lo;
        (lo - pad, hi + pad)
    };
    (lo.clamp(-f64::MAX, f64::MAX), hi.clamp(-f64::MAX, f64::MAX))
}

/// The breaks of an axis between its limits, in increasing order: every multiple of the step
/// from the first at or above `lo` to the last at or below `hi`. Where no step can be found (the
/// limits too close for doubles to hold round numbers between them), the two limits themselves.
pub fn breaks(lo: f64, hi: f64) -> Vec<f64> {
    round_breaks(lo, hi).unwrap_or_else(|| vec![lo, hi])
}

fn round_breaks(lo: f64, hi: f64) -> Option<Vec<f64>> {
    // A fifth of the width, divided before subtracting so that the widest limits do not overflow.
    let raw = hi / INTERVALS - lo / INTERVALS;
    if !(raw.is_finite() && raw > 0.0) {
        return None;
    }
    // The exponent form gives the mantissa, from 1 to below 10, and the power of ten exactly.
    let written = format!("{raw:e}");
    let (mantissa, exponent) = written.split_once('e')?;
    let mantissa: f64 = mantissa.parse().ok()?;
    let mut exponent: i32 = exponent.parse().ok()?;
    let multiplier: i64 = if mantissa >= SQRT_50 {
        exponent += 1;
        1
    } else if mantissa >= SQRT_10 {
        5
    } else if mantissa >= SQRT_2 {
        2
    } else {
        1
    };
    let step: f64 = format!("{multiplier}e{exponent}").parse().ok()?;
    if !(step.is_finite() && step > 0.0) {
        return None;
    }
    let first = (lo / step).floor();
    let last = (hi / step).ceil();
    if !(first.is_finite() && last.is_finite()) || first.abs() > 1e15 || last.abs() > 1e15 {
        return None;
    }
    // One multiple more on each side, since the two quotients are rounded; the limits decide.
    let mut found: Vec<f64> = Vec::new();
    for k in (first as i64 - 1)..=(last as i64 + 1) {
        let value: f64 = format!("{}e{exponent}", k * multiplier).parse().ok()?;
        if value >= lo && value <= hi && found.last() != Some(&value) {
            found.push(value);
        }
    }
    if found.len() < 2 || found.len() > 50 {
        return None;
    }
    Some(found)
}

/// The labels of the breaks: each number as a table prints it, with six significant digits
/// (section 3.8). Where that would give two neighbouring breaks one label, every label is the
/// number in full.
pub fn labels(breaks: &[f64]) -> Vec<String> {
    let short: Vec<String> = breaks.iter().map(|b| format_float(*b)).collect();
    if short.windows(2).any(|pair| pair[0] == pair[1]) {
        breaks.iter().map(|b| json::number(*b)).collect()
    } else {
        short
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limits_add_five_per_cent_on_each_side() {
        assert_eq!(limits(Some((0.0, 10.0))), (-0.5, 10.5));
        assert_eq!(limits(Some((8.0, 20.0))), (7.4, 20.6));
        assert_eq!(limits(Some((-1.0, 1.0))), (-1.1, 1.1));
    }

    #[test]
    fn one_value_and_no_value_still_give_an_axis() {
        assert_eq!(limits(Some((3.0, 3.0))), (2.5, 3.5));
        assert_eq!(limits(Some((0.0, 0.0))), (-0.5, 0.5));
        assert_eq!(limits(Some((1000.0, 1000.0))), (950.0, 1050.0));
        assert_eq!(limits(Some((-1000.0, -1000.0))), (-1050.0, -950.0));
        assert_eq!(limits(None), (0.0, 1.0));
        assert_eq!(limits(Some((f64::NAN, 1.0))), (0.0, 1.0));
        assert_eq!(limits(Some((2.0, 1.0))), (0.0, 1.0));
    }

    #[test]
    fn the_widest_limits_stay_finite() {
        let (lo, hi) = limits(Some((-f64::MAX, f64::MAX)));
        assert_eq!((lo, hi), (-f64::MAX, f64::MAX));
        let (lo, hi) = limits(Some((f64::MAX, f64::MAX)));
        assert!(lo.is_finite() && hi == f64::MAX && lo < hi);
        let b = breaks(-f64::MAX, f64::MAX);
        assert!(b.len() >= 2 && b.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn the_breaks_are_round_numbers_about_five_intervals_apart() {
        assert_eq!(breaks(0.0, 10.0), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(breaks(-0.5, 10.5), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(breaks(7.4, 20.6), [8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0]);
        assert_eq!(breaks(0.0, 1.0), [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!(breaks(-1.1, 1.1), [-1.0, -0.5, 0.0, 0.5, 1.0]);
        assert_eq!(breaks(0.0, 100.0), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
        assert_eq!(breaks(0.0, 40.0), [0.0, 10.0, 20.0, 30.0, 40.0]);
        assert_eq!(breaks(0.0, 7.0), [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        assert_eq!(breaks(1.0e6, 3.0e6), [1.0e6, 1.5e6, 2.0e6, 2.5e6, 3.0e6]);
    }

    #[test]
    fn a_break_is_the_double_nearest_its_decimal() {
        // Three steps of the double nearest 0.1 would give 0.30000000000000004.
        assert_eq!(breaks(0.0, 0.5), [0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(
            breaks(0.25, 0.4),
            [0.26, 0.28, 0.3, 0.32, 0.34, 0.36, 0.38, 0.4]
        );
        let small = breaks(0.0, 3e-7);
        assert_eq!(small, [0.0, 5e-8, 1e-7, 1.5e-7, 2e-7, 2.5e-7, 3e-7]);
        // A zero among the breaks is the positive zero.
        assert!(
            breaks(-1.0, 1.0)
                .iter()
                .all(|v| v.to_bits() != (-0.0f64).to_bits())
        );
    }

    #[test]
    fn limits_too_close_for_round_numbers_are_their_own_breaks() {
        let lo = 1.0e20;
        let hi = lo + 32768.0;
        assert!(hi > lo);
        assert_eq!(breaks(lo, hi), [lo, hi]);
        assert_eq!(breaks(2.0, 2.0), [2.0, 2.0]);
        assert_eq!(breaks(3.0, 1.0), [3.0, 1.0]);
        let tiny = breaks(0.0, 5e-324);
        assert_eq!(tiny, [0.0, 5e-324]);
    }

    #[test]
    fn the_labels_are_the_numbers_as_tables_print_them() {
        assert_eq!(labels(&[0.0, 0.2, 0.4]), ["0", "0.2", "0.4"]);
        assert_eq!(labels(&[-1.0, -0.5, 0.0]), ["-1", "-0.5", "0"]);
        assert_eq!(labels(&[1.0e6, 1.5e6]), ["1000000", "1500000"]);
        assert_eq!(labels(&[0.0, 5e-8, 1e-7]), ["0", "5e-08", "1e-07"]);
        // Six digits cannot tell these apart, so they are written in full.
        assert_eq!(labels(&[1000000.2, 1000000.4]), ["1000000.2", "1000000.4"]);
    }
}
