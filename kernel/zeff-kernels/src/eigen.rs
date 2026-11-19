//! The eigenvalues and eigenvectors of a symmetric matrix by cyclic Jacobi rotations: every
//! pair of rows and columns in a fixed order, sweep after sweep, each rotation chosen to make
//! one off-diagonal entry zero, until a sweep finds nothing left to rotate. The method is slower
//! than a tridiagonal one and is here for what the engine asks of it: matrices of a few rows
//! and columns (the concentration matrix of the instrumented regressors, the covariance of
//! first-stage coefficients), computed with the exactly specified operations alone, so that
//! the four targets give the same bits (Engine and Project Specification, section 8.3, point
//! 3; release 0.8.0, step 12a).

use crate::matrix::Matrix;

/// The largest number of sweeps over every pair; Jacobi converges quadratically once the
/// off-diagonal part is small and a few sweeps suffice, so reaching the limit is not expected.
const MAX_SWEEPS: usize = 60;

/// The eigenvalues of a symmetric matrix in ascending order, and its eigenvectors.
#[derive(Clone, Debug, PartialEq)]
pub struct SymmetricEigen {
    /// The eigenvalues, ascending.
    pub values: Vec<f64>,
    /// The eigenvectors, one per column, in the order of `values`; orthonormal.
    pub vectors: Matrix,
}

/// The eigenvalues and eigenvectors of the symmetric matrix `a`. The matrix is read as the
/// mean of itself and its transpose, so that the last bits by which a computed cross-product
/// differs from its transpose do not matter.
pub fn symmetric_eigen(a: &Matrix) -> SymmetricEigen {
    assert_eq!(a.rows(), a.cols(), "a symmetric matrix is square");
    let n = a.rows();
    let mut w = Matrix::from_fn(n, n, |i, j| 0.5 * a.get(i, j) + 0.5 * a.get(j, i));
    let mut v = Matrix::identity(n);
    // An entry far below every size in the matrix is zero for the purpose: the floor that ends
    // the sweeps when a diagonal entry is itself zero (a singular matrix).
    let largest = w.data().iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    let floor = f64::EPSILON * f64::EPSILON * largest;
    for _ in 0..MAX_SWEEPS {
        let mut rotated = false;
        for p in 0..n {
            for q in p + 1..n {
                let apq = w.get(p, q);
                let (app, aqq) = (w.get(p, p), w.get(q, q));
                if apq == 0.0
                    || apq.abs() <= floor
                    || apq.abs() <= f64::EPSILON * (app.abs() * aqq.abs()).sqrt()
                {
                    if apq != 0.0 {
                        w.set(p, q, 0.0);
                        w.set(q, p, 0.0);
                    }
                    continue;
                }
                // The tangent of the rotation angle: the smaller root of t² + 2θt − 1 = 0, so
                // that the rotation is by at most a quarter turn. θ² cannot overflow: an entry
                // rotated here is above the floor, ε² of the largest entry, and the difference
                // of two diagonal entries is at most twice the largest, so |θ| is below 1/ε²,
                // about 2e31. (An arm for |θ| above 1e150 stood here until release 0.8.0, step
                // 24a2b; the coverage report showed it never ran, and this is why.)
                let theta = (aqq - app) / (2.0 * apq);
                debug_assert!(theta.abs() < 1e40);
                let t = theta.signum() / (theta.abs() + (1.0 + theta * theta).sqrt());
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;
                w.set(p, p, app - t * apq);
                w.set(q, q, aqq + t * apq);
                w.set(p, q, 0.0);
                w.set(q, p, 0.0);
                for r in 0..n {
                    if r != p && r != q {
                        let (arp, arq) = (w.get(r, p), w.get(r, q));
                        let (new_p, new_q) = (c * arp - s * arq, s * arp + c * arq);
                        w.set(r, p, new_p);
                        w.set(p, r, new_p);
                        w.set(r, q, new_q);
                        w.set(q, r, new_q);
                    }
                    let (vrp, vrq) = (v.get(r, p), v.get(r, q));
                    v.set(r, p, c * vrp - s * vrq);
                    v.set(r, q, s * vrp + c * vrq);
                }
                rotated = true;
            }
        }
        if !rotated {
            break;
        }
    }
    // Ascending, equal values in the order the rotations left them.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| w.get(i, i).total_cmp(&w.get(j, j)).then(i.cmp(&j)));
    SymmetricEigen {
        values: order.iter().map(|&i| w.get(i, i)).collect(),
        vectors: Matrix::from_fn(n, n, |r, c| v.get(r, order[c])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-14 * a.abs().max(b.abs()).max(1.0)
    }

    #[test]
    fn known_spectra() {
        let two = Matrix::from_rows(&[vec![2.0, 1.0], vec![1.0, 2.0]]);
        assert_eq!(symmetric_eigen(&two).values, vec![1.0, 3.0]);
        // The second difference matrix of order three: 2 − √2, 2, 2 + √2.
        let three = Matrix::from_rows(&[
            vec![2.0, -1.0, 0.0],
            vec![-1.0, 2.0, -1.0],
            vec![0.0, -1.0, 2.0],
        ]);
        let root = 2.0_f64.sqrt();
        for (got, want) in symmetric_eigen(&three)
            .values
            .iter()
            .zip([2.0 - root, 2.0, 2.0 + root])
        {
            assert!(close(*got, want), "{got} against {want}");
        }
        // Not definite: a zero diagonal.
        let swap = Matrix::from_rows(&[vec![0.0, 1.0], vec![1.0, 0.0]]);
        let found = symmetric_eigen(&swap);
        assert!(close(found.values[0], -1.0) && close(found.values[1], 1.0));
    }

    #[test]
    fn a_diagonal_matrix_is_its_own_spectrum_sorted() {
        let d = Matrix::from_rows(&[
            vec![3.0, 0.0, 0.0],
            vec![0.0, -1.0, 0.0],
            vec![0.0, 0.0, 2.0],
        ]);
        let found = symmetric_eigen(&d);
        assert_eq!(found.values, vec![-1.0, 2.0, 3.0]);
        // The vectors are the axes, in the order of the values.
        assert_eq!(found.vectors.column(0), &[0.0, 1.0, 0.0]);
        assert_eq!(found.vectors.column(1), &[0.0, 0.0, 1.0]);
        assert_eq!(found.vectors.column(2), &[1.0, 0.0, 0.0]);
    }

    #[test]
    fn the_smallest_matrices() {
        let none = symmetric_eigen(&Matrix::zeros(0, 0));
        assert!(none.values.is_empty());
        let one = symmetric_eigen(&Matrix::from_rows(&[vec![-4.5]]));
        assert_eq!(one.values, vec![-4.5]);
        assert_eq!(one.vectors.get(0, 0), 1.0);
        let zero = symmetric_eigen(&Matrix::zeros(3, 3));
        assert_eq!(zero.values, vec![0.0, 0.0, 0.0]);
        assert_eq!(zero.vectors, Matrix::identity(3));
    }

    #[test]
    fn a_matrix_of_rank_one_has_one_eigenvalue_that_is_not_zero() {
        // u uᵀ with u = (1, 2, 2): the eigenvalues are 0, 0 and 9.
        let u = [1.0, 2.0, 2.0];
        let a = Matrix::from_fn(3, 3, |i, j| u[i] * u[j]);
        let found = symmetric_eigen(&a);
        assert!(found.values[0].abs() < 1e-14 && found.values[1].abs() < 1e-14);
        assert!(close(found.values[2], 9.0));
        // The vector of the largest is u over its length, up to sign.
        let sign = found.vectors.get(0, 2).signum();
        for (i, want) in u.iter().enumerate() {
            assert!(close(sign * found.vectors.get(i, 2), want / 3.0));
        }
    }

    #[test]
    fn sizes_far_apart_keep_their_small_eigenvalue() {
        // A graded matrix: the small eigenvalue is found to a relative accuracy, as Jacobi's
        // method gives it for a positive-definite matrix.
        let a = Matrix::from_rows(&[vec![1e24, 1e12], vec![1e12, 2.0]]);
        let found = symmetric_eigen(&a);
        assert!(close(found.values[0], 1.0), "{}", found.values[0]);
        assert!(close(found.values[1], 1e24));
    }

    #[test]
    fn the_transpose_half_is_read_too() {
        let lower = Matrix::from_rows(&[vec![2.0, 0.0], vec![2.0, 2.0]]);
        // Read as the mean with its transpose: [[2, 1], [1, 2]].
        assert_eq!(symmetric_eigen(&lower).values, vec![1.0, 3.0]);
    }
}
