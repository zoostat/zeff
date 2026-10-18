//! Property tests: relations that must hold for any data, within rounding.

use proptest::prelude::*;
use zeff_numeric::{mean, sd, sum, variance};

fn data() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-1.0e6..1.0e6_f64, 2..200)
}

fn scale(values: &[f64]) -> f64 {
    values.iter().fold(1.0, |m: f64, v| m.max(v.abs()))
}

proptest! {
    #[test]
    fn shifting_every_value_shifts_the_mean(values in data(), shift in -1.0e6..1.0e6_f64) {
        let shifted: Vec<f64> = values.iter().map(|v| v + shift).collect();
        let expected = mean(&values).unwrap() + shift;
        let bound = 1e-9 * (scale(&values) + shift.abs());
        prop_assert!((mean(&shifted).unwrap() - expected).abs() <= bound);
    }

    #[test]
    fn shifting_every_value_leaves_the_sd_unchanged(values in data(), shift in -1.0e6..1.0e6_f64) {
        let shifted: Vec<f64> = values.iter().map(|v| v + shift).collect();
        let before = sd(&values).unwrap();
        let after = sd(&shifted).unwrap();
        let bound = 1e-9 * (scale(&values) + shift.abs());
        prop_assert!((after - before).abs() <= bound, "{before} against {after}");
    }

    #[test]
    fn scaling_every_value_scales_the_sd(values in data(), factor in -1.0e3..1.0e3_f64) {
        let scaled: Vec<f64> = values.iter().map(|v| v * factor).collect();
        let expected = sd(&values).unwrap() * factor.abs();
        let actual = sd(&scaled).unwrap();
        prop_assert!((actual - expected).abs() <= 1e-9 * (expected + 1e-300));
    }

    #[test]
    fn the_variance_is_never_negative(values in data()) {
        prop_assert!(variance(&values).unwrap() >= 0.0);
    }

    #[test]
    fn reversing_the_order_changes_the_sum_by_no_more_than_rounding(values in data()) {
        let mut reversed = values.clone();
        reversed.reverse();
        let bound = 1e-12 * values.iter().map(|v| v.abs()).sum::<f64>();
        prop_assert!((sum(&values) - sum(&reversed)).abs() <= bound);
    }

    #[test]
    fn the_same_input_gives_the_same_bits_twice(values in data()) {
        prop_assert_eq!(sum(&values).to_bits(), sum(&values).to_bits());
        prop_assert_eq!(sd(&values).unwrap().to_bits(), sd(&values).unwrap().to_bits());
    }
}

// ----- the maths kernel (release 0.2.0, step 5) -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases_maths(300)))]

    #[test]
    fn log_undoes_exp_and_the_circle_closes(x in -700.0f64..700.0) {
        let back = zeff_numeric::maths::log(zeff_numeric::maths::exp(x));
        prop_assert!((back - x).abs() <= 1e-13 * (1.0 + x.abs()), "log(exp({x})) = {back}");
        let (s, c) = (zeff_numeric::maths::sin(x), zeff_numeric::maths::cos(x));
        prop_assert!((s * s + c * c - 1.0).abs() <= 4e-16, "sin² + cos² at {x}: {}", s * s + c * c);
        let root = zeff_numeric::maths::sqrt(x.abs());
        // A correctly rounded root squared is within two units in the last place of x.
        prop_assert!((root * root - x.abs()).abs() <= 5e-16 * (1.0 + x.abs()));
    }

    #[test]
    fn rounding_is_idempotent_and_within_half_a_unit(x in -1e9f64..1e9, digits in -3i32..6) {
        use zeff_numeric::maths::{RoundingMode, round};
        for mode in [RoundingMode::Even, RoundingMode::Away] {
            let once = round(x, digits, mode);
            prop_assert_eq!(round(once, digits, mode).to_bits(), once.to_bits(), "idempotent");
            let unit = 10f64.powi(-digits);
            // Within half a unit of the exact value, allowing for the spacing of doubles at x.
            prop_assert!((once - x).abs() <= unit / 2.0 + 1e-15 * (1.0 + x.abs()), "round({x}, {digits}) = {once}");
            prop_assert!(once.is_finite());
        }
    }

    #[test]
    fn floored_division_and_remainder_recompose(a in any::<i64>(), b in any::<i64>()) {
        use zeff_numeric::maths::{div_int, mod_int};
        prop_assume!(b != 0);
        if a == i64::MIN && b == -1 {
            prop_assert_eq!(div_int(a, b), None);
            prop_assert_eq!(mod_int(a, b), Some(0));
        } else {
            let (q, r) = (div_int(a, b).unwrap(), mod_int(a, b).unwrap());
            prop_assert_eq!((b as i128) * (q as i128) + (r as i128), a as i128);
            prop_assert!(r == 0 || (r < 0) == (b < 0), "the remainder takes the sign of b");
            prop_assert!((r as i128).abs() < (b as i128).abs());
        }
    }
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases_maths(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
