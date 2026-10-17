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
