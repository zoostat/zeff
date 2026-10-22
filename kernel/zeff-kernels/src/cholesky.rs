//! The Cholesky factorisation of a symmetric positive-definite matrix, A = L Lᵀ, column by column,
//! with each inner sum in the engine's reduction shape. Only the lower triangle of the input is
//! read. A matrix that is not positive definite is reported at the first column where the
//! factorisation meets a pivot that is not positive, which is how the variance layer checks that a
//! variance matrix is positive semi-definite (Language Reference, section 11.1, "Self-checks").

use crate::matrix::Matrix;
use crate::reduce::dot;
use crate::triangular::{ZeroPivot, solve_lower, solve_lower_transposed};

/// The matrix has no Cholesky factor: the pivot at `index` is zero, negative or not a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotPositiveDefinite {
    pub index: usize,
}

/// The lower-triangular L with A = L Lᵀ.
pub fn cholesky(a: &Matrix) -> Result<Matrix, NotPositiveDefinite> {
    let n = a.rows();
    assert_eq!(a.cols(), n, "a matrix to factorise is square");
    let mut l = Matrix::zeros(n, n);
    for j in 0..n {
        let row_j: Vec<f64> = (0..j).map(|k| l.get(j, k)).collect();
        let d = a.get(j, j) - dot(&row_j, &row_j);
        if d.is_nan() || d <= 0.0 {
            return Err(NotPositiveDefinite { index: j });
        }
        let ljj = d.sqrt();
        l.set(j, j, ljj);
        for i in j + 1..n {
            let row_i: Vec<f64> = (0..j).map(|k| l.get(i, k)).collect();
            l.set(i, j, (a.get(i, j) - dot(&row_i, &row_j)) / ljj);
        }
    }
    Ok(l)
}

/// Solve A x = b from the factor L of A: two triangular solves.
pub fn solve_with_factor(l: &Matrix, b: &[f64]) -> Result<Vec<f64>, ZeroPivot> {
    let y = solve_lower(l, b)?;
    solve_lower_transposed(l, &y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::matmul;

    #[test]
    fn a_small_factor_is_exact() {
        let a = Matrix::from_rows(&[vec![4.0, 2.0], vec![2.0, 3.0]]);
        let l = cholesky(&a).unwrap();
        assert_eq!(l.get(0, 0), 2.0);
        assert_eq!(l.get(1, 0), 1.0);
        assert_eq!(l.get(1, 1), 2.0_f64.sqrt());
        assert_eq!(l.get(0, 1), 0.0);
        assert!(matmul(&l, &l.transpose()).max_abs_difference(&a) <= 1e-15);
        assert_eq!(cholesky(&Matrix::identity(3)).unwrap(), Matrix::identity(3));
        assert_eq!(cholesky(&Matrix::zeros(0, 0)).unwrap(), Matrix::zeros(0, 0));
        // A x = b with b = A (1, 2) = (8, 8).
        let x = solve_with_factor(&l, &[8.0, 8.0]).unwrap();
        assert!((x[0] - 1.0).abs() <= 1e-15 && (x[1] - 2.0).abs() <= 1e-15);
    }

    #[test]
    fn a_matrix_that_is_not_positive_definite_is_reported() {
        let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![2.0, 1.0]]);
        assert_eq!(cholesky(&a), Err(NotPositiveDefinite { index: 1 }));
        let zero = Matrix::from_rows(&[vec![0.0, 0.0], vec![0.0, 1.0]]);
        assert_eq!(cholesky(&zero), Err(NotPositiveDefinite { index: 0 }));
        let nan = Matrix::from_rows(&[vec![f64::NAN]]);
        assert_eq!(cholesky(&nan), Err(NotPositiveDefinite { index: 0 }));
    }
}
