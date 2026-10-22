//! Singular values by one-sided Jacobi rotations (Hestenes), on the columns of the matrix, in a
//! fixed sweep order until no pair of columns needs a rotation. Small and exact enough for what
//! the engine asks of it: the condition number of a scaled R factor (Language Reference, section
//! 7.2, "Collinearity"), reported with W1001 above 1e8.

use crate::matrix::Matrix;
use crate::reduce::{dot, norm, sum_of_squares};

/// The largest number of sweeps over every pair of columns; Jacobi converges quadratically and a
/// few sweeps suffice, so reaching the limit is not expected.
const MAX_SWEEPS: usize = 60;

/// The singular values of `a`, in descending order.
pub fn singular_values(a: &Matrix) -> Vec<f64> {
    let mut w = if a.rows() >= a.cols() {
        a.clone()
    } else {
        a.transpose()
    };
    let k = w.cols();
    let rows = w.rows();
    for _ in 0..MAX_SWEEPS {
        let mut rotated = false;
        for p in 0..k {
            for q in p + 1..k {
                let (alpha, beta, gamma) = {
                    let cp = w.column(p);
                    let cq = w.column(q);
                    (sum_of_squares(cp), sum_of_squares(cq), dot(cp, cq))
                };
                if gamma == 0.0 || gamma.abs() <= f64::EPSILON * (alpha * beta).sqrt() {
                    continue;
                }
                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;
                for i in 0..rows {
                    let xp = w.get(i, p);
                    let xq = w.get(i, q);
                    w.set(i, p, c * xp - s * xq);
                    w.set(i, q, s * xp + c * xq);
                }
                rotated = true;
            }
        }
        if !rotated {
            break;
        }
    }
    let mut values: Vec<f64> = (0..k).map(|j| norm(w.column(j))).collect();
    values.sort_by(|x, y| y.total_cmp(x));
    values
}

/// The 2-norm condition number: the largest singular value over the smallest, `Inf` when the
/// smallest is zero, and 1 for a matrix with no columns.
pub fn condition_number(a: &Matrix) -> f64 {
    let values = singular_values(a);
    match (values.first(), values.last()) {
        (Some(&largest), Some(&smallest)) => {
            if smallest == 0.0 {
                f64::INFINITY
            } else {
                largest / smallest
            }
        }
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_singular_values() {
        let d = Matrix::from_rows(&[
            vec![3.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 2.0],
        ]);
        assert_eq!(singular_values(&d), vec![3.0, 2.0, 1.0]);
        // [[3, 0], [4, 5]]: AᵀA has eigenvalues 45 and 5.
        let a = Matrix::from_rows(&[vec![3.0, 0.0], vec![4.0, 5.0]]);
        let values = singular_values(&a);
        assert!((values[0] - 45.0_f64.sqrt()).abs() <= 1e-14);
        assert!((values[1] - 5.0_f64.sqrt()).abs() <= 1e-14);
        assert_eq!(condition_number(&Matrix::identity(4)), 1.0);
        assert!((condition_number(&a) - 3.0).abs() <= 1e-14);
        let singular = Matrix::from_rows(&[vec![1.0, 2.0], vec![2.0, 4.0]]);
        assert_eq!(condition_number(&singular), f64::INFINITY);
        assert_eq!(
            singular_values(&Matrix::from_rows(&[vec![-7.0]])),
            vec![7.0]
        );
        assert_eq!(condition_number(&Matrix::zeros(0, 0)), 1.0);
        // A wide matrix has the singular values of its transpose.
        let wide = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![0.0, 1.0, 1.0]]);
        let tall = singular_values(&wide.transpose());
        for (x, y) in singular_values(&wide).iter().zip(&tall) {
            assert!((x - y).abs() <= 1e-14);
        }
    }

    #[test]
    fn the_frobenius_norm_is_preserved() {
        let a = Matrix::from_fn(7, 4, |i, j| ((i * 5 + j * 3) % 13) as f64 - 6.0);
        let values = singular_values(&a);
        let total: f64 = values.iter().map(|v| v * v).sum();
        assert!((total - sum_of_squares(a.data())).abs() <= 1e-11 * total);
        assert!(values.windows(2).all(|w| w[0] >= w[1]));
    }
}
