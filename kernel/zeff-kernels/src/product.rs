//! Matrix products. Every entry is a dot product in the engine's reduction shape, so the bits
//! depend on that shape alone; the loops below order the work for contiguous access, and any
//! blocking added for speed later can change no bit (section 8.3, point 3, "Reductions").

use crate::matrix::Matrix;
use crate::reduce::dot;

/// `a × b`.
pub fn matmul(a: &Matrix, b: &Matrix) -> Matrix {
    assert_eq!(
        a.cols(),
        b.rows(),
        "the inner dimensions of a product must agree"
    );
    let mut c = Matrix::zeros(a.rows(), b.cols());
    let mut row = vec![0.0; a.cols()];
    for i in 0..a.rows() {
        for (k, slot) in row.iter_mut().enumerate() {
            *slot = a.get(i, k);
        }
        for j in 0..b.cols() {
            c.set(i, j, dot(&row, b.column(j)));
        }
    }
    c
}

/// `a × x` for a vector `x`.
pub fn mat_vec(a: &Matrix, x: &[f64]) -> Vec<f64> {
    assert_eq!(
        a.cols(),
        x.len(),
        "the vector must have one entry per column"
    );
    (0..a.rows()).map(|i| dot(&a.row(i), x)).collect()
}

/// `aᵀ × b`: both operands read column by column.
pub fn transpose_matmul(a: &Matrix, b: &Matrix) -> Matrix {
    assert_eq!(
        a.rows(),
        b.rows(),
        "the inner dimensions of a product must agree"
    );
    Matrix::from_fn(a.cols(), b.cols(), |i, j| dot(a.column(i), b.column(j)))
}

/// `aᵀ × a`, symmetric to the bit: the entry (i, j) and the entry (j, i) are the same products in
/// the same order, since multiplication commutes exactly.
pub fn gram(a: &Matrix) -> Matrix {
    let k = a.cols();
    let mut g = Matrix::zeros(k, k);
    for i in 0..k {
        for j in i..k {
            let v = dot(a.column(i), a.column(j));
            g.set(i, j, v);
            g.set(j, i, v);
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_products_are_exact() {
        let a = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
        let b = Matrix::from_rows(&[vec![7.0, 8.0], vec![9.0, 10.0], vec![11.0, 12.0]]);
        assert_eq!(
            matmul(&a, &b),
            Matrix::from_rows(&[vec![58.0, 64.0], vec![139.0, 154.0]])
        );
        assert_eq!(matmul(&Matrix::identity(2), &a), a);
        assert_eq!(mat_vec(&a, &[1.0, 0.0, -1.0]), vec![-2.0, -2.0]);
        assert_eq!(transpose_matmul(&a, &a), gram(&a));
        assert_eq!(
            gram(&a),
            Matrix::from_rows(&[
                vec![17.0, 22.0, 27.0],
                vec![22.0, 29.0, 36.0],
                vec![27.0, 36.0, 45.0]
            ])
        );
        let empty = matmul(&Matrix::zeros(0, 2), &Matrix::zeros(2, 3));
        assert_eq!((empty.rows(), empty.cols()), (0, 3));
    }

    #[test]
    fn the_gram_matrix_is_symmetric_to_the_bit() {
        let a = Matrix::from_fn(50, 4, |i, j| {
            ((i * 31 + j * 17) % 23) as f64 * 1e-3 + 0.1 * (j as f64)
        });
        let g = gram(&a);
        for i in 0..4 {
            for j in 0..4 {
                assert_eq!(g.get(i, j).to_bits(), g.get(j, i).to_bits());
            }
        }
    }

    #[test]
    #[should_panic(expected = "inner dimensions")]
    fn a_product_of_the_wrong_shapes_is_refused() {
        matmul(&Matrix::zeros(2, 3), &Matrix::zeros(2, 3));
    }
}
