//! Engine kernel: compensated sums and the other fixed-order numerical routines.
//!
//! The determinism contract (Engine and Project Specification, section 8.3, point 3) asks for the
//! same bits on every target. Everything here is written so that it holds:
//!
//! * Every sum over values has the reduction shape of `zeff-kernels` (open decision #47, taken
//!   on 23 October 2026): independent compensated accumulators in a fixed order, so the bits
//!   depend on that shape alone and not on the SIMD width or the target. Every other loop (a
//!   series, a continued fraction, a running total) runs from the first element to the last.
//! * Only IEEE 754 operations that are exactly specified are used (`+`, `-`, `*`, `/`, `sqrt`,
//!   `abs`); no fused multiply-add and no platform maths library.
//! * Variances use the corrected two-pass algorithm (section 13), so the result does not depend
//!   on the size of the numbers.
//!
//! These routines work on slices of `f64` without missing values. Skipping `NA` and counting what
//! was skipped belongs to the column layer, which passes only the valid values here.

pub mod association;
pub mod maths;
pub mod order;
pub mod random;
pub mod special;

/// The sum of `values` in the reduction shape of `zeff-kernels` (compensated, in a fixed order).
/// The sum of no values is `0.0`.
///
/// If the result is not finite (an infinite input, or an overflow) the plain left-to-right IEEE
/// sum is returned instead: `Inf` for `Inf` alone, and not-a-number for `Inf` with `-Inf`. The
/// caller turns that into the language's rules for infinity and `NA`.
pub fn sum(values: &[f64]) -> f64 {
    zeff_kernels::sum(values)
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
    let deviations: Vec<f64> = values.iter().map(|&x| x - first).collect();
    Some(first + sum(&deviations) / n)
}

/// The sample variance (divisor n − 1) of `values`, or `None` for fewer than two values.
///
/// The corrected two-pass algorithm: the mean first, then the sum of squared deviations from it
/// less the correction for the mean's own rounding error. The deviations are scaled by a power of
/// two so that the largest lies in [1, 2) before squaring, which is exact and keeps the squares of
/// very small or very large deviations from underflowing or overflowing; for ordinary values the
/// result is bit for bit the unscaled one. The result is never negative. If any value is not
/// finite the result is not-a-number; the caller decides what that means.
pub fn variance(values: &[f64]) -> Option<f64> {
    scaled_variance(values).map(|(scaled, scale)| scaled / scale / scale)
}

/// The sample standard deviation (divisor n − 1), or `None` for fewer than two values.
///
/// The square root is taken of the scaled variance and the scale undone afterwards, so that a
/// standard deviation whose square underflows (values near 1e-300) is still exact to a rounding.
pub fn sd(values: &[f64]) -> Option<f64> {
    scaled_variance(values).map(|(scaled, scale)| scaled.sqrt() / scale)
}

/// The variance of the deviations multiplied by `scale`, and the scale: a power of two that puts
/// the largest absolute deviation in [1, 2), or 1 when every deviation is zero or not finite.
fn scaled_variance(values: &[f64]) -> Option<(f64, f64)> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let center = mean(values)?;
    let deviations: Vec<f64> = values.iter().map(|&x| x - center).collect();
    let largest = deviations
        .iter()
        .map(|d| d.abs())
        .filter(|d| d.is_finite())
        .fold(0.0f64, f64::max);
    let scale = if largest == 0.0 {
        1.0
    } else {
        power_of_two_scale(largest)
    };
    let scaled: Vec<f64> = deviations.iter().map(|d| d * scale).collect();
    let squares = zeff_kernels::sum_of_squares(&scaled);
    let correction = sum(&scaled);
    let result = (squares - correction * correction / n) / (n - 1.0);
    if result.is_nan() {
        Some((result, scale))
    } else {
        Some((result.max(0.0), scale))
    }
}

/// The power of two that brings a positive finite `x` into [1, 2): 2 to the minus its binary
/// exponent, read from the bits so that no library function is involved.
fn power_of_two_scale(x: f64) -> f64 {
    let exponent_field = ((x.to_bits() >> 52) & 0x7ff) as i64;
    // A subnormal has the exponent of the smallest normal number less one; its scale is capped at
    // the largest finite power of two, which still brings it into a safe range.
    let exponent = if exponent_field == 0 {
        -1022
    } else {
        exponent_field - 1023
    };
    let field = (1023 - exponent).clamp(1, 2046) as u64;
    f64::from_bits(field << 52)
}

/// `x` to the power `y`, from the pure-Rust `libm` (Engine Specification, section 8.3, point 3:
/// one implementation of the elementary functions on every target, never the platform's).
/// The result is not-a-number where the real power does not exist; the caller turns that into
/// the language's `NA` with its warning.
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The plain left-to-right sum, for what the compensated sum must improve on.
    fn plain_sum(values: &[f64]) -> f64 {
        values.iter().fold(0.0, |total, &x| total + x)
    }

    /// Each moved routine is the kernel's lane sum, to the bit (release 0.4.0, step 2b).
    #[test]
    fn the_reducers_are_the_kernels_lane_sums_to_the_bit() {
        let values: Vec<f64> = (0..1_003)
            .map(|i| ((i * 7919) % 1000) as f64 * 1e-3 + 1e8 * ((i % 3) as f64) - 5e7)
            .collect();
        assert_eq!(sum(&values).to_bits(), zeff_kernels::sum(&values).to_bits());
        let n = values.len() as f64;
        let first = zeff_kernels::sum(&values) / n;
        let deviations: Vec<f64> = values.iter().map(|&x| x - first).collect();
        let expected_mean = first + zeff_kernels::sum(&deviations) / n;
        assert_eq!(mean(&values).unwrap().to_bits(), expected_mean.to_bits());
        let centred: Vec<f64> = values.iter().map(|&x| x - expected_mean).collect();
        let largest = centred.iter().fold(0.0f64, |m, d| m.max(d.abs()));
        let scale = power_of_two_scale(largest);
        let scaled: Vec<f64> = centred.iter().map(|d| d * scale).collect();
        let correction = zeff_kernels::sum(&scaled);
        let expected_variance = (zeff_kernels::sum_of_squares(&scaled)
            - correction * correction / n)
            / (n - 1.0)
            / scale
            / scale;
        assert_eq!(
            variance(&values).unwrap().to_bits(),
            expected_variance.to_bits()
        );
        let others: Vec<f64> = values.iter().rev().map(|v| v * 0.5 + 1.0).collect();
        let mean_y = mean(&others).unwrap();
        let dy: Vec<f64> = others.iter().map(|y| y - mean_y).collect();
        let expected_covariance = zeff_kernels::dot(&centred, &dy) / (n - 1.0);
        assert_eq!(
            association::covariance(&values, &others).unwrap().to_bits(),
            expected_covariance.to_bits()
        );
        let expected_pearson = (zeff_kernels::dot(&centred, &dy)
            / zeff_kernels::sum_of_squares(&centred).sqrt()
            / zeff_kernels::sum_of_squares(&dy).sqrt())
        .clamp(-1.0, 1.0);
        assert_eq!(
            association::pearson(&values, &others).unwrap().to_bits(),
            expected_pearson.to_bits()
        );
    }

    #[test]
    fn powers_are_exact_where_the_answer_is_and_na_where_there_is_none() {
        assert_eq!(pow(2.0, 10.0), 1024.0);
        assert_eq!(pow(2.0, 0.5), 2f64.sqrt());
        assert_eq!(pow(0.0, 0.0), 1.0);
        assert_eq!(pow(-2.0, 3.0), -8.0);
        assert_eq!(pow(10.0, -2.0), 0.01);
        assert!(
            pow(-8.0, 1.0 / 3.0).is_nan(),
            "no real cube root through pow"
        );
        assert_eq!(pow(2.0, 1100.0), f64::INFINITY);
    }

    #[test]
    fn the_spread_of_very_small_and_very_large_values_survives_squaring() {
        // Squares of deviations near 1e-301 underflow and near 1e200 overflow; the scaling keeps
        // the spread, which is 1e-301 and 1e200 to a rounding in both cases.
        let tiny = [1e-301, 2e-301, 3e-301];
        let small = sd(&tiny).unwrap();
        assert!((small - 1e-301).abs() <= 1e-316, "{small:e}");
        let huge = [1e200, 2e200, 3e200];
        let large = sd(&huge).unwrap();
        assert!((large - 1e200).abs() <= 1e185, "{large:e}");
        assert_eq!(variance(&[5.0, 5.0, 5.0]), Some(0.0));
        assert_eq!(sd(&[5.0, 5.0, 5.0]), Some(0.0));
        // Ordinary values give the plain two-pass result, bit for bit.
        let plain = [23.4, 19.0, 31.2];
        let center = plain.iter().sum::<f64>() / 3.0;
        let squares: f64 = plain.iter().map(|x| (x - center) * (x - center)).sum();
        assert_eq!(variance(&plain), Some(squares / 2.0));
        // Deviations in the subnormal range (below 2.2e-308) take the capped scale; the spread of
        // 1e-310 and 3e-310 is √2 × 1e-310, not 0. The first weekly mutation run (18 October
        // 2026) found no test here.
        let subnormal = sd(&[1e-310, 3e-310]).unwrap();
        assert!(
            (subnormal - 2f64.sqrt() * 1e-310).abs() <= 1e-324 * 16.0,
            "{subnormal:e}"
        );
        assert_eq!(power_of_two_scale(1e-310), f64::from_bits(2045u64 << 52));
        assert_eq!(power_of_two_scale(1.5), 1.0);
        assert_eq!(power_of_two_scale(1e-301), 2f64.powi(1000));
        assert_eq!(power_of_two_scale(1e200), 2f64.powi(-664));
    }

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
