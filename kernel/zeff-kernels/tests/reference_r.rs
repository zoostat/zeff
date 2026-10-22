//! The kernel against R 4.6.0 (`tests/reference/r/kernels.R`): the limited-pivoting QR of `qr`
//! (LINPACK's dqrdc2, the factorisation behind `lm`), its coefficients, residuals and leverages,
//! the condition numbers and singular values, `backsolve`, `chol` and `solve`.

mod common;

use common::{
    close, equilibrated_condition, integers, load, matrix, max_abs, number, tolerance_for, vector,
    with_positive_diagonal,
};
use zeff_kernels::cholesky::{cholesky, solve_with_factor};
use zeff_kernels::product::matmul;
use zeff_kernels::qr::{least_squares, pivoted_qr, r_factor};
use zeff_kernels::svd::{condition_number, singular_values};
use zeff_kernels::triangular::{inverse_upper, solve_upper, solve_upper_transposed};
use zeff_kernels::{Matrix, norm, sum_of_squares};

#[test]
fn the_pivoted_qr_agrees_with_r() {
    let exact = load("kernels.json");
    let reference = load("r/kernels.json");
    let cases = reference["least_squares"].as_object().expect("cases");
    assert_eq!(cases.len(), 12);
    for (name, case) in cases {
        let inputs = &exact["least_squares"][name];
        let x = matrix(&inputs["x"]);
        let y = vector(&inputs["y"]);
        let k = x.cols();
        let pivoted = pivoted_qr(&r_factor(&x), k, 1e-7);
        assert_eq!(
            pivoted.rank,
            case["rank"].as_u64().unwrap() as usize,
            "{name}: rank"
        );
        // The columns kept, in order, and the set left out; the order among the columns left
        // out is that of the moves and means nothing (R pivots the n × k matrix and stops at row
        // n, the kernel pivots the k × k factor and moves every empty column to the end).
        let pivot = integers(&case["pivot"]);
        let rank = pivoted.rank;
        assert_eq!(
            &pivoted.order[..rank],
            &pivot[..rank],
            "{name}: the columns kept"
        );
        let mut ours_out = pivoted.order[rank..].to_vec();
        let mut r_out = pivot[rank..].to_vec();
        ours_out.sort_unstable();
        r_out.sort_unstable();
        assert_eq!(ours_out, r_out, "{name}: the columns left out");
        // R's qr.R: min(n, k) rows, the columns in pivot order. The same factor up to the sign of
        // each factorised row, each entry within rounding of its column's norm; the columns are
        // matched by their original index.
        let r_ref = with_positive_diagonal(&matrix(&case["r"]), rank);
        let ours = with_positive_diagonal(
            &Matrix::from_fn(r_ref.rows(), k, |i, j| pivoted.r.get(i, j)),
            rank,
        );
        for c in 0..k {
            let p_ours = pivoted.order.iter().position(|&o| o == c).unwrap();
            let p_r = pivot.iter().position(|&o| o == c).unwrap();
            let bound = 1e-12 * norm(x.column(c)).max(1e-300);
            for i in 0..r_ref.rows() {
                assert!(
                    (ours.get(i, p_ours) - r_ref.get(i, p_r)).abs() <= bound,
                    "{name}: R of column {c} at row {i} is {} against R's {}",
                    ours.get(i, p_ours),
                    r_ref.get(i, p_r)
                );
            }
        }
        let fit = least_squares(&x, &y, 1e-7, 64);
        let condition = equilibrated_condition(&x, &fit.order[..fit.rank]);
        let tolerance = tolerance_for(condition, 1e-12);
        let coefficients: Vec<f64> = case["coefficients"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        let scale = max_abs(
            &coefficients
                .iter()
                .copied()
                .filter(|v| !v.is_nan())
                .collect::<Vec<_>>(),
        );
        for (j, (got, want)) in fit.coefficients.iter().zip(&coefficients).enumerate() {
            match got {
                None => assert!(want.is_nan(), "{name}: column {j} left out, R keeps it"),
                Some(value) => assert!(
                    close(*value, *want, tolerance, tolerance * scale),
                    "{name}: coefficient {j} is {value} against R's {want} (condition {condition:.3})"
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
            "{name}: rss {} against {rss}",
            fit.residual_sum_of_squares
        );
        let leverage = vector(&case["leverage"]);
        for (i, want) in leverage.iter().enumerate() {
            let xi: Vec<f64> = fit.order[..fit.rank].iter().map(|&j| x.get(i, j)).collect();
            let h = sum_of_squares(&solve_upper_transposed(&fit.r, &xi).unwrap());
            let bound = tolerance_for(condition, 1e-11);
            assert!(
                close(h, *want, bound, bound),
                "{name}: leverage {i} is {h} against {want}"
            );
        }
        // Singular values within rounding of the largest; the condition number of the kept R
        // within what LAPACK's accuracy on the smallest singular value allows (its absolute error
        // is about ε times the largest, so the relative error of the ratio grows with the ratio).
        let values = singular_values(&x);
        let r_values = vector(&case["singular_values"]);
        assert_eq!(
            values.len(),
            r_values.len(),
            "{name}: the number of singular values"
        );
        let largest = r_values[0];
        for (got, want) in values.iter().zip(&r_values) {
            assert!(
                (got - want).abs() <= 1e-12 * largest,
                "{name}: singular value {got} against R's {want}"
            );
        }
        let kappa = condition_number(&fit.r);
        let r_kappa = number(&case["condition_number"]);
        assert!(
            close(kappa, r_kappa, 2e-13 * r_kappa.max(1.0), 0.0),
            "{name}: condition number {kappa} against R's {r_kappa}"
        );
        let r_scaled = number(&case["condition_number_scaled"]);
        assert!(
            close(condition, r_scaled, tolerance_for(condition, 1e-12), 0.0),
            "{name}: scaled condition number {condition} against R's {r_scaled}"
        );
    }
}

#[test]
fn backsolve_chol_and_solve_agree_with_r() {
    let exact = load("kernels.json");
    let reference = load("r/kernels.json");
    let r = matrix(&exact["triangular"]["r"]);
    let b = vector(&exact["triangular"]["b"]);
    let x = solve_upper(&r, &b).unwrap();
    for (got, want) in x.iter().zip(vector(&reference["triangular"]["x"])) {
        assert!(
            close(*got, want, 1e-14, 0.0),
            "backsolve: {got} against {want}"
        );
    }
    let inverse = inverse_upper(&r).unwrap();
    let want = matrix(&reference["triangular"]["inverse"]);
    assert!(inverse.max_abs_difference(&want) <= 1e-14 * max_abs(want.data()));

    let a = matrix(&exact["positive_definite"]["a"]);
    let l = cholesky(&a).unwrap();
    let want = matrix(&reference["positive_definite"]["l"]);
    assert!(
        l.max_abs_difference(&want) <= 1e-12 * max_abs(want.data()),
        "chol differs by {}",
        l.max_abs_difference(&want)
    );
    assert!(matmul(&l, &l.transpose()).max_abs_difference(&a) <= 1e-12 * max_abs(a.data()));
    let x = solve_with_factor(&l, &vector(&exact["positive_definite"]["b"])).unwrap();
    for (got, want) in x.iter().zip(vector(&reference["positive_definite"]["x"])) {
        assert!(close(*got, want, 1e-10, 0.0), "solve: {got} against {want}");
    }
}
