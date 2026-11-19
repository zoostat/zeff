//! The integral of a smooth function over a short interval (`zeff_numeric::quadrature`): the
//! eight-point rule is exact for polynomials of degree fifteen, the halving comes to integrals
//! known in closed form, and a function it cannot follow within its depth, or one that is not
//! finite, is given up. A jump is the documented exception, and the test shows it.

use zeff_numeric::quadrature::{integral, panel};

fn relative(got: f64, want: f64) -> f64 {
    ((got - want) / want).abs()
}

#[test]
fn the_rule_is_exact_for_polynomials_of_degree_fifteen() {
    for degree in 0..=15 {
        let f = |x: f64| x.powi(degree);
        // The integral of x^n over [0, 1] is 1 / (n + 1), and over [-1, 2] is
        // (2^(n+1) - (-1)^(n+1)) / (n + 1).
        let unit = 1.0 / f64::from(degree + 1);
        assert!(
            relative(panel(&f, 0.0, 1.0), unit) < 2e-15,
            "degree {degree}"
        );
        let wide = (2f64.powi(degree + 1) - (-1f64).powi(degree + 1)) / f64::from(degree + 1);
        assert!(
            relative(panel(&f, -1.0, 2.0), wide) < 2e-14,
            "degree {degree}"
        );
    }
    // Degree sixteen is past the rule: one panel is not exact, and the halving is.
    let f = |x: f64| x.powi(16);
    assert!(relative(panel(&f, 0.0, 1.0), 1.0 / 17.0) > 1e-12);
    assert!(relative(integral(&f, 0.0, 1.0).unwrap(), 1.0 / 17.0) < 1e-15);
}

#[test]
fn integrals_known_in_closed_form() {
    // The exponential over [0, 1]: e - 1.
    let got = integral(&|x: f64| x.exp(), 0.0, 1.0).unwrap();
    assert!(relative(got, std::f64::consts::E - 1.0) < 4e-16);
    // 1 / x over [1, 400]: ln 400, which one panel cannot follow.
    let got = integral(&|x: f64| 1.0 / x, 1.0, 400.0).unwrap();
    assert!(relative(got, 400f64.ln()) < 1e-14);
    // x^(-1/2) from 1e-2 to 1: 2 (1 - 0.1), steep at the lower end.
    let got = integral(&|x: f64| 1.0 / x.sqrt(), 1e-2, 1.0).unwrap();
    assert!(relative(got, 1.8) < 1e-14);
    // An interval of one part in 1e300: the function's value times the width.
    let got = integral(&|x: f64| 3.0 + x, 0.0, 1e-300).unwrap();
    assert!(relative(got, 3e-300) < 4e-16);
    // An interval narrower than the spacing of the doubles around it is still its width.
    let (a, b) = (5.0, 5.0 + 4.0 * f64::EPSILON);
    let got = integral(&|x: f64| 2.0 * x, a, b).unwrap();
    assert!(relative(got, 10.0 * (b - a)) < 1e-15);
}

#[test]
fn a_function_the_rule_cannot_follow_is_given_up() {
    // x^(-1/2) from 1e-10: ten halvings do not reach the steep end.
    assert_eq!(integral(&|x: f64| 1.0 / x.sqrt(), 1e-10, 1.0), None);
    // A function that is not finite.
    assert_eq!(integral(&|_: f64| f64::NAN, 0.0, 1.0), None);
    assert_eq!(integral(&|_: f64| f64::INFINITY, 0.0, 1.0), None);
    // Not finite on a stretch that the first eight points miss and the halves find: the points
    // of [0, 1] nearest its middle are 0.4083 and 0.5917, and the last point of [0, 0.5] is
    // 0.4901 (release 0.8.0, step 24a2b, from the coverage report).
    let gap = |x: f64| {
        if (0.49..0.51).contains(&x) {
            f64::NAN
        } else {
            1.0
        }
    };
    assert_eq!(integral(&gap, 0.0, 1.0), None);
}

#[test]
fn the_same_bits_on_every_call() {
    let f = |x: f64| (-0.5 * x * x).exp();
    let first = integral(&f, 0.25, 0.75).unwrap();
    assert_eq!(first.to_bits(), integral(&f, 0.25, 0.75).unwrap().to_bits());
    // R: sqrt(2 * pi) * (pnorm(0.75) - pnorm(0.25)) = 0.43782354120112649.
    assert!(relative(first, 0.437_823_541_201_126_5) < 1e-15);
}

/// The limit the module states: a jump can sit where neither the whole nor the halves see it,
/// and the answer is then wrong with no sign of it. The caller is the guard.
#[test]
fn a_jump_can_pass_unseen() {
    let step = |x: f64| if x < 0.123_456_789 { 0.0 } else { 1.0 };
    let got = integral(&step, 0.0, 1.0).unwrap();
    assert!((got - (1.0 - 0.123_456_789)).abs() > 1e-3, "{got}");
}
