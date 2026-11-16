//! Property tests of the kernel: what must hold for any data, to the bit where the design
//! promises bits and within rounding where it promises a value (the lesson of the 0.3.0 close:
//! a property asks for what doubles can give).

use proptest::prelude::*;
use zeff_kernels::cholesky::cholesky;
use zeff_kernels::eigen::symmetric_eigen;
use zeff_kernels::product::{gram, mat_vec, matmul};
use zeff_kernels::qr::{TallSkinnyQr, fold, least_squares, r_factor};
use zeff_kernels::svd::singular_values;
use zeff_kernels::triangular::solve_upper;
use zeff_kernels::{LaneSum, Matrix, dot, sum, sum_of_squares};

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn matrix(
    rows: std::ops::Range<usize>,
    cols: std::ops::Range<usize>,
) -> impl Strategy<Value = Matrix> {
    (rows, cols).prop_flat_map(|(n, k)| {
        prop::collection::vec(-1.0e3..1.0e3_f64, n * k)
            .prop_map(move |data| Matrix::from_fn(n, k, |i, j| data[j * n + i]))
    })
}

fn max_abs(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |m, v| m.max(v.abs()))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    /// The lane shape is a function of the element's index: cutting a sequence into slices of
    /// any sizes gives the same bits.
    #[test]
    fn a_sum_gives_the_same_bits_however_it_is_cut(
        values in prop::collection::vec(-1.0e9..1.0e9_f64, 0..400),
        cuts in prop::collection::vec(1usize..40, 1..20),
    ) {
        let whole = sum(&values);
        let mut lanes = LaneSum::new();
        let mut start = 0;
        let mut i = 0;
        while start < values.len() {
            let end = (start + cuts[i % cuts.len()]).min(values.len());
            lanes.push_slice(&values[start..end]);
            start = end;
            i += 1;
        }
        prop_assert_eq!(lanes.value().to_bits(), whole.to_bits());
    }

    /// Integers whose partial sums fit in 53 bits are summed exactly, as are their products.
    #[test]
    fn integer_sums_and_dot_products_are_exact(
        pairs in prop::collection::vec((-30_000i64..30_000, -30_000i64..30_000), 0..600),
    ) {
        let xs: Vec<f64> = pairs.iter().map(|p| p.0 as f64).collect();
        let ys: Vec<f64> = pairs.iter().map(|p| p.1 as f64).collect();
        let exact_sum: i64 = pairs.iter().map(|p| p.0).sum();
        let exact_dot: i64 = pairs.iter().map(|p| p.0 * p.1).sum();
        prop_assert_eq!(sum(&xs), exact_sum as f64);
        prop_assert_eq!(dot(&xs, &ys), exact_dot as f64);
    }

    /// RᵀR is XᵀX within rounding.
    #[test]
    fn r_transposed_r_is_the_gram_matrix(x in matrix(1..40, 1..6)) {
        let xtx = gram(&x);
        let rtr = gram(&r_factor(&x));
        let scale = max_abs(xtx.data()).max(1e-300);
        prop_assert!(rtr.max_abs_difference(&xtx) <= 1e-12 * scale, "difference {}", rtr.max_abs_difference(&xtx));
    }

    /// The blocked QR is the block factors folded in order, whatever the block size and however
    /// the rows are handed over, to the bit (open decision #48).
    #[test]
    fn the_blocked_qr_is_the_block_factors_folded_in_order(x in matrix(1..120, 1..5), block_rows in 1usize..40, cut in 1usize..50) {
        let by_hand = {
            let mut r: Option<Matrix> = None;
            for start in (0..x.rows()).step_by(block_rows) {
                let block = r_factor(&x.rows_slice(start, (start + block_rows).min(x.rows())));
                r = Some(match r {
                    None => block,
                    Some(r) => fold(&r, &block),
                });
            }
            r.unwrap()
        };
        let mut accumulator = TallSkinnyQr::with_block_rows(x.cols(), block_rows);
        for start in (0..x.rows()).step_by(cut) {
            accumulator.push_rows(&x.rows_slice(start, (start + cut).min(x.rows())));
        }
        prop_assert_eq!(accumulator.finish(), by_hand);
    }

    /// Scaling a column by a power of two scales its coefficient by the inverse power exactly and
    /// changes no other coefficient's bits (every operation of the factorisation commutes with a
    /// power-of-two scaling, and no fused operation hides a rounding).
    #[test]
    fn a_column_scaled_by_a_power_of_two_scales_its_coefficient_exactly(
        x in matrix(6..30, 1..4),
        y_seed in prop::collection::vec(-1.0e3..1.0e3_f64, 30),
        column in 0usize..4,
        exponent in -20i32..20,
    ) {
        let column = column % x.cols();
        let y: Vec<f64> = y_seed[..x.rows()].to_vec();
        let fit = least_squares(&x, &y, 1e-7, 7);
        prop_assume!(fit.rank == x.cols());
        let factor = 2f64.powi(exponent);
        let scaled = Matrix::from_fn(x.rows(), x.cols(), |i, j| if j == column { x.get(i, j) * factor } else { x.get(i, j) });
        let scaled_fit = least_squares(&scaled, &y, 1e-7, 7);
        prop_assert_eq!(scaled_fit.rank, fit.rank);
        for j in 0..x.cols() {
            let want = if j == column { fit.coefficients[j].unwrap() / factor } else { fit.coefficients[j].unwrap() };
            prop_assert_eq!(scaled_fit.coefficients[j].unwrap().to_bits(), want.to_bits(), "coefficient {}", j);
        }
        prop_assert_eq!(scaled_fit.residual_sum_of_squares.to_bits(), fit.residual_sum_of_squares.to_bits());
    }

    /// A triangular solve inverts the product.
    #[test]
    fn a_triangular_solve_inverts_the_product(
        diagonal in prop::collection::vec(1.0..10.0_f64, 1..8),
        above in prop::collection::vec(-2.0..2.0_f64, 64),
        x in prop::collection::vec(-100.0..100.0_f64, 8),
    ) {
        let n = diagonal.len();
        let r = Matrix::from_fn(n, n, |i, j| if i == j { diagonal[i] } else if i < j { above[i * 8 + j] } else { 0.0 });
        let x = &x[..n];
        let b = mat_vec(&r, x);
        let back = solve_upper(&r, &b).unwrap();
        let bound = 1e-9 * max_abs(x).max(1.0);
        for (got, want) in back.iter().zip(x) {
            prop_assert!((got - want).abs() <= bound, "{got} against {want}");
        }
    }

    /// The Cholesky factor reproduces the matrix.
    #[test]
    fn the_cholesky_factor_reproduces_the_matrix(b in matrix(3..12, 1..6)) {
        let mut a = gram(&b);
        for i in 0..a.rows() {
            a.set(i, i, a.get(i, i) + 1.0);
        }
        let l = cholesky(&a).unwrap();
        let back = matmul(&l, &l.transpose());
        prop_assert!(back.max_abs_difference(&a) <= 1e-12 * max_abs(a.data()));
    }

    /// The singular values carry the Frobenius norm, come sorted, and are those of the transpose.
    #[test]
    fn singular_values_preserve_the_frobenius_norm(x in matrix(1..20, 1..6)) {
        let values = singular_values(&x);
        prop_assert_eq!(values.len(), x.rows().min(x.cols()));
        let total: f64 = values.iter().map(|v| v * v).sum();
        let frobenius = sum_of_squares(x.data());
        prop_assert!((total - frobenius).abs() <= 1e-11 * frobenius.max(1e-300));
        prop_assert!(values.windows(2).all(|w| w[0] >= w[1]));
        let transposed = singular_values(&x.transpose());
        let largest = values.first().copied().unwrap_or(0.0);
        for (a, b) in values.iter().zip(&transposed) {
            prop_assert!((a - b).abs() <= 1e-12 * largest.max(1e-300));
        }
    }
}

/// A symmetric matrix of order one to seven, its entries of very different sizes: each entry
/// (i, j) is a value in (−1, 1) times the scales of row i and of column j, a power of ten from
/// 1e-6 to 1e6 each, so that the matrix is graded as a covariance of variables in different
/// units is.
fn symmetric() -> impl Strategy<Value = Matrix> {
    (1usize..8).prop_flat_map(|n| {
        (
            prop::collection::vec(-1.0..1.0_f64, n * n),
            prop::collection::vec(-6i32..7, n),
        )
            .prop_map(move |(data, powers)| {
                let scale: Vec<f64> = powers.iter().map(|p| 10f64.powi(*p)).collect();
                Matrix::from_fn(n, n, |i, j| {
                    let (lo, hi) = (i.min(j), i.max(j));
                    data[lo * n + hi] * scale[i] * scale[j]
                })
            })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    /// What an eigen-decomposition is: A v = λ v for every pair, the vectors orthonormal, the
    /// values ascending and summing to the trace.
    #[test]
    fn the_eigenvectors_of_a_symmetric_matrix_are_what_they_say(a in symmetric()) {
        let n = a.rows();
        let found = symmetric_eigen(&a);
        let largest = max_abs(a.data());
        for pair in found.values.windows(2) {
            prop_assert!(pair[0] <= pair[1]);
        }
        for (c, value) in found.values.iter().enumerate() {
            let v = found.vectors.column(c);
            for i in 0..n {
                let av: f64 = (0..n).map(|j| a.get(i, j) * v[j]).sum();
                prop_assert!((av - value * v[i]).abs() <= 1e-13 * largest * n as f64, "row {} of A v − λ v: {}", i, av - value * v[i]);
            }
            for d in 0..=c {
                let product = dot(v, found.vectors.column(d));
                let want = if c == d { 1.0 } else { 0.0 };
                prop_assert!((product - want).abs() <= 1e-14 * n as f64, "vectors {} and {}: {}", c, d, product);
            }
        }
        let trace: f64 = (0..n).map(|i| a.get(i, i)).sum();
        prop_assert!((sum(&found.values) - trace).abs() <= 1e-13 * largest * n as f64);
    }

    /// The same matrix with its rows and columns in another order has the same eigenvalues, and
    /// a matrix times a power of two has its eigenvalues times that power, to the bit.
    #[test]
    fn the_eigenvalues_do_not_depend_on_the_order_or_the_unit(a in symmetric(), turn in 0usize..7) {
        let n = a.rows();
        let found = symmetric_eigen(&a);
        let largest = max_abs(a.data());
        let moved = Matrix::from_fn(n, n, |i, j| a.get((i + turn) % n, (j + turn) % n));
        for (x, y) in found.values.iter().zip(&symmetric_eigen(&moved).values) {
            prop_assert!((x - y).abs() <= 1e-13 * largest * n as f64, "{} against {}", x, y);
        }
        let scaled = Matrix::from_fn(n, n, |i, j| a.get(i, j) * 1024.0);
        for (x, y) in found.values.iter().zip(&symmetric_eigen(&scaled).values) {
            prop_assert_eq!((x * 1024.0).to_bits(), y.to_bits());
        }
    }

    /// The cross-product of a matrix has the squares of its singular values for eigenvalues.
    #[test]
    fn the_eigenvalues_of_a_cross_product_are_the_squared_singular_values(x in matrix(2..12, 1..5)) {
        // Ascending, as the eigenvalues are.
        let squares: Vec<f64> = singular_values(&x).iter().rev().map(|s| s * s).collect();
        let found = symmetric_eigen(&gram(&x));
        let largest = found.values.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        // A matrix with fewer rows than columns has more eigenvalues than singular values: the
        // extra ones are zero, the smallest.
        let extra = found.values.len() - squares.len();
        for value in &found.values[..extra] {
            prop_assert!(value.abs() <= 1e-11 * largest, "{} is not zero", value);
        }
        for (value, square) in found.values[extra..].iter().zip(&squares) {
            prop_assert!((value - square).abs() <= 1e-11 * largest, "{} against {}", value, square);
        }
    }
}
