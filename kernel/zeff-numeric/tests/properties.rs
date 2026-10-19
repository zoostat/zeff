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

// ----- the special functions (release 0.3.0, step 2) -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases_maths(200)))]

    /// Γ(x + 1) = x Γ(x), and ln Γ likewise, for x where neither side overflows.
    #[test]
    fn the_gamma_recurrence_holds(x in 0.05..150.0_f64) {
        use zeff_numeric::special::{gamma, log_gamma};
        let left = gamma(x + 1.0);
        let right = x * gamma(x);
        prop_assert!((left - right).abs() <= 1e-13 * right.abs(), "gamma: {left} against {right}");
        let left = log_gamma(x + 1.0);
        let right = log_gamma(x) + x.ln();
        prop_assert!((left - right).abs() <= 1e-13 * right.abs().max(1.0), "log_gamma: {left} against {right}");
    }

    /// Both tails of every incomplete function sum to one, lie in [0, 1], and the lower tail does
    /// not fall as x rises.
    #[test]
    fn the_incomplete_functions_are_probabilities(
        a in 0.05..2000.0_f64, b in 0.05..2000.0_f64, x in 0.0..1.0_f64, y in 0.0..1.0_f64
    ) {
        use zeff_numeric::special::{incomplete_beta, incomplete_gamma};
        let gx = x * 3.0 * a;
        let gy = y * 3.0 * a;
        let g1 = incomplete_gamma(a, gx.min(gy));
        let g2 = incomplete_gamma(a, gx.max(gy));
        prop_assert!((0.0..=1.0).contains(&g1.lower) && (0.0..=1.0).contains(&g1.upper));
        prop_assert!((g1.lower + g1.upper - 1.0).abs() <= 1e-14);
        prop_assert!(g1.lower <= g2.lower + 1e-15, "P not monotone: {} then {}", g1.lower, g2.lower);
        let b1 = incomplete_beta(a, b, x.min(y));
        let b2 = incomplete_beta(a, b, x.max(y));
        prop_assert!((0.0..=1.0).contains(&b1.lower) && (0.0..=1.0).contains(&b1.upper));
        prop_assert!((b1.lower + b1.upper - 1.0).abs() <= 1e-14);
        prop_assert!(b1.lower <= b2.lower + 1e-15, "I not monotone: {} then {}", b1.lower, b2.lower);
        // Symmetry: I_x(a, b) = 1 - I_{1-x}(b, a).
        let direct = incomplete_beta(a, b, x);
        // 1 − x is rounded, and a tail's sensitivity to x is about a / x, so a far tail may move
        // by a / x units in the last place: within 1e-9 here (a up to 2000, x down to 1e-3 or so).
        let mirrored = incomplete_beta(b, a, 1.0 - x);
        prop_assert!((direct.lower - mirrored.upper).abs() <= 1e-9 * direct.lower.max(mirrored.upper) + 1e-300,
            "symmetry: {} against {}", direct.lower, mirrored.upper);
    }

    /// Every inverse undoes its function: the probability comes back within the class tolerance,
    /// or, where the function is so steep that neighbouring doubles straddle p (a beta tail near
    /// one with a small shape), the returned double is the nearest one: its neighbours' tails lie
    /// on both sides of p.
    #[test]
    fn the_inverses_round_trip(
        a in 0.1..500.0_f64, b in 0.1..500.0_f64, p in 1e-12..0.999_999_f64, upper in any::<bool>()
    ) {
        use zeff_numeric::special::*;
        fn nearest(f: impl Fn(f64) -> f64, x: f64, p: f64) -> bool {
            let back = f(x);
            if (back - p).abs() <= 1e-11 * p {
                return true;
            }
            let below = f(f64::from_bits(x.to_bits() - 1));
            let above = f(f64::from_bits(x.to_bits() + 1));
            let (lo, hi) = (below.min(above), below.max(above));
            lo <= p && p <= hi
        }
        let x = incomplete_gamma_inverse(a, p, upper);
        prop_assert!(nearest(|x| incomplete_gamma(a, x).tail(upper), x, p), "gamma: p {p} -> x {x} -> {}", incomplete_gamma(a, x).tail(upper));
        let x = incomplete_beta_inverse(a, b, p, upper);
        if x == 1.0 {
            // The quantile is not representable below one: the nearest double has a larger tail.
            let inside = incomplete_beta(a, b, 1.0 - f64::EPSILON / 2.0).tail(upper);
            prop_assert!(inside >= p, "beta at one: p {p}, the tail just below one is {inside}");
        } else {
            prop_assert!(nearest(|x| incomplete_beta(a, b, x).tail(upper), x, p), "beta: p {p} -> x {x} -> {}", incomplete_beta(a, b, x).tail(upper));
        }
        let z = normal_quantile(p, upper);
        let back = normal_tails(z).tail(upper);
        prop_assert!((back - p).abs() <= 1e-13 * p, "normal: p {p} -> z {z} -> {back}");
    }
}

// ----- the random stream (release 0.3.0, step 4) -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases_maths(200)))]

    /// A stream is a pure function of (seed, statement, pass): the same triple gives the same
    /// draws, and a change to any part changes the first draw; every uniform is one of the 2^53
    /// values in [0, 1).
    #[test]
    fn the_stream_is_a_function_of_its_key_and_position(
        seed in any::<u64>(), ordinal in 1..1_000_000u32, pass in 0..1000u32, n in 1..64usize
    ) {
        use zeff_numeric::random::Stream;
        let draws: Vec<f64> = (0..n).map(|_| Stream::new(seed, ordinal, pass)).enumerate()
            .map(|(i, mut s)| { for _ in 0..i { s.uniform(); } s.uniform() })
            .collect();
        let mut straight = Stream::new(seed, ordinal, pass);
        for (i, want) in draws.iter().enumerate() {
            let got = straight.uniform();
            prop_assert_eq!(got, *want, "draw {} differs between a fresh and a running stream", i);
            prop_assert!((0.0..1.0).contains(&got));
            prop_assert_eq!((got * 9_007_199_254_740_992.0).fract(), 0.0);
        }
        let mut other = Stream::new(seed, ordinal + 1, pass);
        prop_assert_ne!(other.uniform(), draws[0]);
        let mut other = Stream::new(seed ^ 1, ordinal, pass);
        prop_assert_ne!(other.uniform(), draws[0]);
        let mut other = Stream::new(seed, ordinal, pass + 1);
        prop_assert_ne!(other.uniform(), draws[0]);
    }
}
