//! The kernel against the exact rational reference of `tests/reference/kernels.py`: least squares
//! with the collinearity rule, the triangular and positive-definite systems, and the compensated
//! sums, each within the tolerance its class allows (closed form, 1e-6; the kernel is held far
//! tighter, as the maths and special-function kernels are).

mod common;

use common::{
    close, equilibrated_condition, integers, load, matrix, max_abs, number, tolerance_for, vector,
};
use zeff_kernels::cholesky::{cholesky, solve_with_factor};
use zeff_kernels::product::{gram, matmul};
use zeff_kernels::qr::least_squares;
use zeff_kernels::triangular::{inverse_upper, solve_upper, solve_upper_transposed};
use zeff_kernels::{dot, sum, sum_of_squares};

#[test]
fn least_squares_matches_the_exact_solutions() {
    let reference = load("kernels.json");
    let cases = reference["least_squares"].as_object().expect("cases");
    assert_eq!(cases.len(), 12);
    for (name, case) in cases {
        let x = matrix(&case["x"]);
        let y = vector(&case["y"]);
        // Blocks of 64 rows, so that the tall case folds several block factors.
        let fit = least_squares(&x, &y, 1e-7, 64);
        assert_eq!(
            fit.rank,
            case["rank"].as_u64().unwrap() as usize,
            "{name}: rank"
        );
        assert_eq!(
            &fit.order[..fit.rank],
            &integers(&case["kept"])[..],
            "{name}: the columns kept"
        );
        let condition = equilibrated_condition(&x, &fit.order[..fit.rank]);
        let tolerance = tolerance_for(condition, 1e-12);
        let expected: Vec<f64> = case["coefficients"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        let scale = max_abs(
            &expected
                .iter()
                .copied()
                .filter(|v| !v.is_nan())
                .collect::<Vec<_>>(),
        );
        for (j, (got, want)) in fit.coefficients.iter().zip(&expected).enumerate() {
            match got {
                None => assert!(
                    want.is_nan(),
                    "{name}: column {j} left out, the reference keeps it"
                ),
                Some(value) => assert!(
                    close(*value, *want, tolerance, tolerance * scale),
                    "{name}: coefficient {j} is {value}, the exact value {want} (condition {condition:.3})"
                ),
            }
        }
        let rss = number(&case["residual_sum_of_squares"]);
        assert!(
            close(
                fit.residual_sum_of_squares,
                rss,
                1e-10,
                1e-20 * sum_of_squares(&y)
            ),
            "{name}: residual sum of squares {} against {rss}",
            fit.residual_sum_of_squares
        );
        // XᵀX by the engine's reductions, entry by entry within rounding of its own scale.
        let xtx = matrix(&case["xtx"]);
        let g = gram(&x);
        for i in 0..x.cols() {
            for j in 0..x.cols() {
                let bound = 1e-14 * (xtx.get(i, i) * xtx.get(j, j)).sqrt();
                assert!(
                    (g.get(i, j) - xtx.get(i, j)).abs() <= bound,
                    "{name}: XᵀX at ({i}, {j}) is {} against {}",
                    g.get(i, j),
                    xtx.get(i, j)
                );
            }
        }
        // (XᵀX)⁻¹ of the kept columns, R⁻¹ R⁻ᵀ, and the leverages ‖R⁻ᵀ xᵢ‖².
        let inverse = inverse_upper(&fit.r).unwrap();
        let cov = matmul(&inverse, &inverse.transpose());
        let want = matrix(&case["kept_xtx_inverse"]);
        let scale = max_abs(want.data());
        assert!(
            cov.max_abs_difference(&want) <= tolerance_for(condition, 1e-11) * scale,
            "{name}: (XᵀX)⁻¹ differs by {}",
            cov.max_abs_difference(&want)
        );
        let leverage = vector(&case["leverage"]);
        for (i, want) in leverage.iter().enumerate() {
            let xi: Vec<f64> = fit.order[..fit.rank].iter().map(|&j| x.get(i, j)).collect();
            let z = solve_upper_transposed(&fit.r, &xi).unwrap();
            let h = sum_of_squares(&z);
            let bound = tolerance_for(condition, 1e-11);
            assert!(
                close(h, *want, bound, bound),
                "{name}: leverage of row {i} is {h} against {want}"
            );
        }
        // The fitted values, from the coefficients.
        let fitted = vector(&case["fitted"]);
        let bound = tolerance_for(condition, 1e-12) * max_abs(&y);
        for (i, want) in fitted.iter().enumerate() {
            let value: f64 = (0..x.cols())
                .map(|j| fit.coefficients[j].unwrap_or(0.0) * x.get(i, j))
                .sum();
            assert!(
                (value - want).abs() <= bound,
                "{name}: fitted value {i} is {value} against {want}"
            );
        }
    }
}

#[test]
fn the_triangular_system_is_solved_to_the_last_digits() {
    let reference = load("kernels.json");
    let case = &reference["triangular"];
    let r = matrix(&case["r"]);
    let b = vector(&case["b"]);
    let x = solve_upper(&r, &b).unwrap();
    let want = vector(&case["x"]);
    for (i, (got, want)) in x.iter().zip(&want).enumerate() {
        assert!(
            close(*got, *want, 1e-14, 0.0),
            "x[{i}] is {got} against {want}"
        );
    }
    let inverse = inverse_upper(&r).unwrap();
    let want = matrix(&case["inverse"]);
    assert!(inverse.max_abs_difference(&want) <= 1e-14 * max_abs(want.data()));
}

#[test]
fn the_positive_definite_system_is_solved_through_its_factor() {
    let reference = load("kernels.json");
    let case = &reference["positive_definite"];
    let a = matrix(&case["a"]);
    let l = cholesky(&a).unwrap();
    let back = matmul(&l, &l.transpose());
    assert!(back.max_abs_difference(&a) <= 1e-12 * max_abs(a.data()));
    let x = solve_with_factor(&l, &vector(&case["b"])).unwrap();
    let want = vector(&case["x"]);
    for (i, (got, want)) in x.iter().zip(&want).enumerate() {
        assert!(
            close(*got, *want, 1e-10, 0.0),
            "x[{i}] is {got} against {want}"
        );
    }
}

#[test]
fn the_compensated_sums_match_the_exact_ones() {
    let reference = load("kernels.json");
    let case = &reference["sums"];
    let values = vector(&case["values"]);
    let others = vector(&case["others"]);
    let want = number(&case["sum"]);
    assert!(
        close(sum(&values), want, 1e-13, 0.0),
        "sum {} against {want}",
        sum(&values)
    );
    let want = number(&case["dot"]);
    assert!(
        close(dot(&values, &others), want, 1e-12, 0.0),
        "dot {} against {want}",
        dot(&values, &others)
    );
    let want = number(&case["sum_of_squares"]);
    assert!(close(sum_of_squares(&others), want, 1e-13, 0.0));
}
