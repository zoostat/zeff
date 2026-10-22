//! Triangular systems: back and forward substitution, and the inverse of a triangular matrix.
//! Each inner sum is a dot product in the engine's reduction shape.

use crate::matrix::Matrix;
use crate::reduce::dot;

/// A diagonal entry of zero, at which a triangular system has no solution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroPivot {
    pub index: usize,
}

fn check_square(m: &Matrix, b: &[f64]) -> usize {
    let n = m.rows();
    assert_eq!(m.cols(), n, "a triangular matrix is square");
    assert_eq!(b.len(), n, "the right-hand side needs one value per row");
    n
}

/// Solve R x = b for upper-triangular R, by back substitution.
pub fn solve_upper(r: &Matrix, b: &[f64]) -> Result<Vec<f64>, ZeroPivot> {
    let n = check_square(r, b);
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let d = r.get(i, i);
        if d == 0.0 {
            return Err(ZeroPivot { index: i });
        }
        let row: Vec<f64> = (i + 1..n).map(|j| r.get(i, j)).collect();
        x[i] = (b[i] - dot(&row, &x[i + 1..])) / d;
    }
    Ok(x)
}

/// Solve Rᵀ x = b for upper-triangular R, by forward substitution down its columns.
pub fn solve_upper_transposed(r: &Matrix, b: &[f64]) -> Result<Vec<f64>, ZeroPivot> {
    let n = check_square(r, b);
    let mut x = vec![0.0; n];
    for i in 0..n {
        let d = r.get(i, i);
        if d == 0.0 {
            return Err(ZeroPivot { index: i });
        }
        x[i] = (b[i] - dot(&r.column(i)[..i], &x[..i])) / d;
    }
    Ok(x)
}

/// Solve L x = b for lower-triangular L, by forward substitution.
pub fn solve_lower(l: &Matrix, b: &[f64]) -> Result<Vec<f64>, ZeroPivot> {
    let n = check_square(l, b);
    let mut x = vec![0.0; n];
    for i in 0..n {
        let d = l.get(i, i);
        if d == 0.0 {
            return Err(ZeroPivot { index: i });
        }
        let row: Vec<f64> = (0..i).map(|j| l.get(i, j)).collect();
        x[i] = (b[i] - dot(&row, &x[..i])) / d;
    }
    Ok(x)
}

/// Solve Lᵀ x = b for lower-triangular L, by back substitution up its columns.
pub fn solve_lower_transposed(l: &Matrix, b: &[f64]) -> Result<Vec<f64>, ZeroPivot> {
    let n = check_square(l, b);
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let d = l.get(i, i);
        if d == 0.0 {
            return Err(ZeroPivot { index: i });
        }
        x[i] = (b[i] - dot(&l.column(i)[i + 1..], &x[i + 1..])) / d;
    }
    Ok(x)
}

/// The inverse of an upper-triangular matrix, column by column.
pub fn inverse_upper(r: &Matrix) -> Result<Matrix, ZeroPivot> {
    let n = r.rows();
    assert_eq!(r.cols(), n, "a triangular matrix is square");
    let mut columns = Vec::with_capacity(n);
    for j in 0..n {
        let mut e = vec![0.0; n];
        e[j] = 1.0;
        columns.push(solve_upper(r, &e)?);
    }
    Ok(Matrix::from_columns(&columns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::matmul;

    #[test]
    fn small_systems_solve_exactly() {
        let r = Matrix::from_rows(&[vec![2.0, 1.0], vec![0.0, 4.0]]);
        assert_eq!(solve_upper(&r, &[5.0, 8.0]), Ok(vec![1.5, 2.0]));
        // Rᵀ x = b: 2 x0 = 2, x0 + 4 x1 = 9.
        assert_eq!(solve_upper_transposed(&r, &[2.0, 9.0]), Ok(vec![1.0, 2.0]));
        let l = r.transpose();
        assert_eq!(solve_lower(&l, &[2.0, 9.0]), Ok(vec![1.0, 2.0]));
        assert_eq!(solve_lower_transposed(&l, &[5.0, 8.0]), Ok(vec![1.5, 2.0]));
        let inverse = inverse_upper(&r).unwrap();
        assert_eq!(
            inverse,
            Matrix::from_rows(&[vec![0.5, -0.125], vec![0.0, 0.25]])
        );
        assert_eq!(matmul(&r, &inverse), Matrix::identity(2));
        assert_eq!(solve_upper(&Matrix::zeros(0, 0), &[]), Ok(vec![]));
    }

    #[test]
    fn a_zero_diagonal_is_reported() {
        let r = Matrix::from_rows(&[vec![1.0, 1.0], vec![0.0, 0.0]]);
        assert_eq!(solve_upper(&r, &[1.0, 1.0]), Err(ZeroPivot { index: 1 }));
        assert_eq!(
            solve_upper_transposed(&r, &[1.0, 1.0]),
            Err(ZeroPivot { index: 1 })
        );
        assert_eq!(
            solve_lower(&r.transpose(), &[1.0, 1.0]),
            Err(ZeroPivot { index: 1 })
        );
        assert_eq!(
            solve_lower_transposed(&r.transpose(), &[1.0, 1.0]),
            Err(ZeroPivot { index: 1 })
        );
        assert_eq!(inverse_upper(&r), Err(ZeroPivot { index: 1 }));
    }
}
