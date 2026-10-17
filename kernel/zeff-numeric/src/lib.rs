//! Engine kernel: compensated sums and the other fixed-order numerical routines.
//!
//! The determinism contract (Engine and Project Specification, section 8.3, point 3) asks for the
//! same bits on every target. Everything here is written so that it holds:
//!
//! * Every loop runs from the first element to the last, one element at a time, so the order of
//!   the floating-point operations is fixed by the code and not by the SIMD width or the target.
//! * Only IEEE 754 operations that are exactly specified are used (`+`, `-`, `*`, `/`, `sqrt`,
//!   `abs`); no fused multiply-add and no platform maths library.
//! * Sums are compensated (Neumaier) and variances use the corrected two-pass algorithm
//!   (section 13), so the result does not depend on the size of the numbers.
//!
//! These routines work on slices of `f64` without missing values. Skipping `NA` and counting what
//! was skipped belongs to the column layer, which passes only the valid values here.

/// A running compensated (Neumaier) sum.
#[derive(Debug, Clone, Copy, Default)]
struct Accumulator {
    total: f64,
    compensation: f64,
}

impl Accumulator {
    fn add(&mut self, x: f64) {
        let t = self.total + x;
        if self.total.abs() >= x.abs() {
            self.compensation += (self.total - t) + x;
        } else {
            self.compensation += (x - t) + self.total;
        }
        self.total = t;
    }

    fn value(self) -> f64 {
        self.total + self.compensation
    }
}

/// The plain left-to-right sum, used only to give an infinite or overflowing sum its IEEE value.
fn plain_sum(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |total, &x| total + x)
}

/// The sum of `values`, compensated, in order from first to last. The sum of no values is `0.0`.
///
/// If the result is not finite (an infinite input, or an overflow) the plain IEEE sum is returned
/// instead: `Inf` for `Inf` alone, and not-a-number for `Inf` with `-Inf`. The caller turns that
/// into the language's rules for infinity and `NA`.
pub fn sum(values: &[f64]) -> f64 {
    let mut accumulator = Accumulator::default();
    for &x in values {
        accumulator.add(x);
    }
    let result = accumulator.value();
    if result.is_finite() {
        result
    } else {
        plain_sum(values)
    }
}

/// The mean of `values`, or `None` for no values.
///
/// The sum is divided by the count and then corrected once by the mean of the deviations, which
/// brings the result to within a rounding of the exact mean. A non-finite sum gives the plain
/// IEEE quotient, as for [`sum`].
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let n = values.len() as f64;
    let first = sum(values) / n;
    if !first.is_finite() {
        return Some(first);
    }
    let mut deviations = Accumulator::default();
    for &x in values {
        deviations.add(x - first);
    }
    Some(first + deviations.value() / n)
}

/// The sample variance (divisor n − 1) of `values`, or `None` for fewer than two values.
///
/// The corrected two-pass algorithm: the mean first, then the sum of squared deviations from it
/// less the correction for the mean's own rounding error. The result is never negative. If any
/// value is not finite the result is not-a-number; the caller decides what that means.
pub fn variance(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let center = mean(values)?;
    let mut squares = Accumulator::default();
    let mut deviations = Accumulator::default();
    for &x in values {
        let d = x - center;
        squares.add(d * d);
        deviations.add(d);
    }
    let correction = deviations.value();
    let result = (squares.value() - correction * correction / n) / (n - 1.0);
    if result.is_nan() {
        Some(result)
    } else {
        Some(result.max(0.0))
    }
}

/// The sample standard deviation (divisor n − 1), or `None` for fewer than two values.
pub fn sd(values: &[f64]) -> Option<f64> {
    variance(values).map(f64::sqrt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sum_of_nothing_is_zero_and_the_mean_of_nothing_is_none() {
        assert_eq!(sum(&[]), 0.0);
        assert_eq!(mean(&[]), None);
        assert_eq!(variance(&[]), None);
        assert_eq!(sd(&[]), None);
    }

    #[test]
    fn ten_tenths_make_one() {
        // The plain sum of ten 0.1 gives 0.9999999999999999.
        let tenths = [0.1; 10];
        assert_eq!(plain_sum(&tenths), 0.999_999_999_999_999_9);
        assert_eq!(sum(&tenths), 1.0);
    }

    #[test]
    fn a_large_term_that_cancels_does_not_swallow_a_small_one() {
        assert_eq!(sum(&[1e100, 1.0, -1e100]), 1.0);
        assert_eq!(sum(&[1.0, 1e100, 1.0, -1e100]), 2.0);
    }

    #[test]
    fn the_order_of_the_input_does_not_change_an_exactly_representable_sum() {
        let forward = [1e16, 1.0, 1.0, -1e16, 0.5];
        let mut backward = forward;
        backward.reverse();
        assert_eq!(sum(&forward), 2.5);
        assert_eq!(sum(&backward), 2.5);
    }

    #[test]
    fn infinite_sums_follow_ieee() {
        assert_eq!(sum(&[1.0, f64::INFINITY, 2.0]), f64::INFINITY);
        assert_eq!(sum(&[1.0, f64::NEG_INFINITY]), f64::NEG_INFINITY);
        assert!(sum(&[f64::INFINITY, f64::NEG_INFINITY]).is_nan());
        assert_eq!(sum(&[1e308, 1e308]), f64::INFINITY);
        assert_eq!(mean(&[1.0, f64::INFINITY]), Some(f64::INFINITY));
    }

    #[test]
    fn the_mean_of_simple_data() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        assert_eq!(mean(&[0.1, 0.2, 0.3]), Some(0.2));
        assert_eq!(mean(&[-7.0]), Some(-7.0));
    }

    #[test]
    fn the_variance_of_simple_data() {
        assert_eq!(variance(&[1.0, 2.0, 3.0, 4.0]), Some(5.0 / 3.0));
        assert_eq!(
            sd(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]),
            Some((32.0_f64 / 7.0).sqrt())
        );
        assert_eq!(variance(&[5.0]), None);
    }

    #[test]
    fn constant_data_has_zero_variance_exactly() {
        assert_eq!(variance(&[3.25; 7]), Some(0.0));
        assert_eq!(sd(&[1e9 + 0.1; 5]), Some(0.0));
    }

    #[test]
    fn a_large_offset_does_not_spoil_the_variance() {
        // The exact sample variance of 4, 7, 13, 16 is 30; the one-pass formula loses it at 1e9.
        let data = [1e9 + 4.0, 1e9 + 7.0, 1e9 + 13.0, 1e9 + 16.0];
        assert_eq!(variance(&data), Some(30.0));
    }

    #[test]
    fn non_finite_values_give_a_non_number_variance() {
        assert!(variance(&[1.0, f64::INFINITY, 3.0]).unwrap().is_nan());
    }
}
