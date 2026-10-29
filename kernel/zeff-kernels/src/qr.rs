//! Householder QR: the factorisation every least-squares fit rests on (Engine and Project
//! Specification, section 13: "OLS is solved by QR decomposition, never by inverting X′X").
//!
//! Three pieces. `r_factor` is the plain Householder QR of one matrix, LAPACK's reflector
//! convention, written with the engine's reductions. [`TallSkinnyQr`] consumes rows in blocks of
//! [`BLOCK_ROWS`] (section 8.3, point 10): each block is factorised on its own and its R folded
//! into the running R in block order, so no n × k matrix is ever held and a later parallel
//! version, computing the blocks' factors on several threads, gives the same bits (open decision
//! #48). [`pivoted_qr`] is the limited column pivoting of R's `dqrdc2` (the QR behind `lm`): a
//! column whose remaining norm falls below a tolerance of its own norm is moved to the end and
//! left out, so the collinearity rule of section 7.2 of the Language Reference can name it.

use crate::matrix::Matrix;
use crate::reduce::{dot, norm};

/// The rows of one block of the tall-skinny QR, a fixed constant of the engine: at most the
/// million of section 8.3, point 10, and small enough that a block of a hundred columns is held
/// in about 50 MB.
pub const BLOCK_ROWS: usize = 65_536;

/// A Householder reflector H = I − τ v vᵀ with v[0] = 1, mapping its column to (β, 0, …, 0).
struct Reflector {
    tau: f64,
    v: Vec<f64>,
    beta: f64,
}

/// The reflector of a column segment, as LAPACK's `dlarfg` forms it: β = −sign(x₀)‖x‖, τ = (β −
/// x₀)/β, v = x/(x₀ − β) with v₀ = 1; a segment that is already (x₀, 0, …, 0) gets τ = 0.
fn reflector(x: &[f64]) -> Reflector {
    let alpha = x[0];
    let rest_norm = norm(&x[1..]);
    if rest_norm == 0.0 {
        let mut v = vec![0.0; x.len()];
        v[0] = 1.0;
        return Reflector {
            tau: 0.0,
            v,
            beta: alpha,
        };
    }
    let beta = -(alpha * alpha + rest_norm * rest_norm)
        .sqrt()
        .copysign(alpha);
    let tau = (beta - alpha) / beta;
    let scale = 1.0 / (alpha - beta);
    let mut v = Vec::with_capacity(x.len());
    v.push(1.0);
    v.extend(x[1..].iter().map(|&xi| xi * scale));
    Reflector { tau, v, beta }
}

/// Apply a reflector to a column segment of its length: c ← c − τ v (vᵀ c).
fn apply(r: &Reflector, column: &mut [f64]) {
    if r.tau == 0.0 {
        return;
    }
    let w = r.tau * dot(&r.v, column);
    for (c, &vi) in column.iter_mut().zip(&r.v) {
        *c -= w * vi;
    }
}

/// Householder QR in place: the upper triangle of `a` becomes R, and everything under the
/// diagonal of the factorised columns is set to zero. Columns from `first_untouched` onwards are
/// transformed by every reflector but never factorised themselves.
fn factor_in_place(a: &mut Matrix, factored_cols: usize) {
    let steps = a.rows().min(factored_cols);
    for j in 0..steps {
        let segment = a.column(j)[j..].to_vec();
        let r = reflector(&segment);
        {
            let column = a.column_mut(j);
            column[j] = r.beta;
            for value in &mut column[j + 1..] {
                *value = 0.0;
            }
        }
        for c in j + 1..a.cols() {
            apply(&r, &mut a.column_mut(c)[j..]);
        }
    }
}

/// The R factor of `a` (n × k), as a k × k upper-triangular matrix; when n < k the rows beyond n
/// are zero.
pub fn r_factor(a: &Matrix) -> Matrix {
    let mut work = a.clone();
    factor_in_place(&mut work, a.cols());
    let k = a.cols();
    Matrix::from_fn(k, k, |i, j| {
        if i <= j && i < a.rows() {
            work.get(i, j)
        } else {
            0.0
        }
    })
}

/// The tall-skinny QR: rows arrive in any pieces and are re-cut into blocks of a fixed size; each
/// block is factorised on its own, and the block factors are folded into one R in block order.
pub struct TallSkinnyQr {
    cols: usize,
    block_rows: usize,
    buffer: Matrix,
    filled: usize,
    rows_seen: usize,
    r: Option<Matrix>,
}

impl TallSkinnyQr {
    /// An accumulator for matrices of `cols` columns, in blocks of [`BLOCK_ROWS`].
    pub fn new(cols: usize) -> TallSkinnyQr {
        TallSkinnyQr::with_block_rows(cols, BLOCK_ROWS)
    }

    /// The same with another block size, for tests and the conformance cases, which exercise the
    /// folding on small data.
    pub fn with_block_rows(cols: usize, block_rows: usize) -> TallSkinnyQr {
        assert!(block_rows >= 1, "a block holds at least one row");
        TallSkinnyQr {
            cols,
            block_rows,
            buffer: Matrix::zeros(block_rows, cols),
            filled: 0,
            rows_seen: 0,
            r: None,
        }
    }

    pub fn rows_seen(&self) -> usize {
        self.rows_seen
    }

    /// Add one row.
    pub fn push_row(&mut self, row: &[f64]) {
        assert_eq!(row.len(), self.cols, "a row must have one entry per column");
        for (j, &value) in row.iter().enumerate() {
            self.buffer.set(self.filled, j, value);
        }
        self.filled += 1;
        self.rows_seen += 1;
        if self.filled == self.block_rows {
            self.fold_buffer();
        }
    }

    /// Add the rows of a matrix with the same columns.
    pub fn push_rows(&mut self, rows: &Matrix) {
        assert_eq!(
            rows.cols(),
            self.cols,
            "the rows must have one entry per column"
        );
        for i in 0..rows.rows() {
            if self.filled == 0 && rows.rows() - i >= self.block_rows {
                // A whole block at once, column by column.
                for j in 0..self.cols {
                    self.buffer
                        .column_mut(j)
                        .copy_from_slice(&rows.column(j)[i..i + self.block_rows]);
                }
                self.filled = self.block_rows;
                self.rows_seen += self.block_rows;
                self.fold_buffer();
                return self.push_rows(&rows.rows_slice(i + self.block_rows, rows.rows()));
            }
            self.push_row(&rows.row(i));
        }
    }

    /// Factorise the rows in the buffer as one block and fold its R into the running R.
    fn fold_buffer(&mut self) {
        if self.filled == 0 {
            return;
        }
        let block = self.buffer.rows_slice(0, self.filled);
        self.filled = 0;
        let block_r = r_factor(&block);
        self.r = Some(match self.r.take() {
            None => block_r,
            Some(r) => fold(&r, &block_r),
        });
    }

    /// The R factor of every row seen: k × k upper triangular, zero when no row was given.
    pub fn finish(mut self) -> Matrix {
        self.fold_buffer();
        self.r
            .unwrap_or_else(|| Matrix::zeros(self.cols, self.cols))
    }
}

/// Fold a block's R factor into the running one: the R factor of the two stacked.
pub fn fold(running: &Matrix, block: &Matrix) -> Matrix {
    r_factor(&Matrix::stack(running, block))
}

/// The outcome of [`pivoted_qr`].
#[derive(Clone, Debug, PartialEq)]
pub struct PivotedQr {
    /// The transformed matrix: upper triangular in its first `rank` columns, the columns in the
    /// order of `order`, with every column transformed by the accepted reflectors (so a column
    /// given beyond the pivot candidates holds Qᵀ of its original, its tail included).
    pub r: Matrix,
    /// `order[p]` is the original index of the column now at position `p`; the accepted columns
    /// come first, in their original order, then the ones left out.
    pub order: Vec<usize>,
    /// The number of accepted columns.
    pub rank: usize,
    /// The original norms of the pivot candidates, by original index (a zero norm recorded as 1,
    /// as `dqrdc2` does, so that a zero column is left out).
    pub norms: Vec<f64>,
}

/// Householder QR with the limited column pivoting of R's `dqrdc2`: the first `pivot_cols`
/// columns are factorised in order; a column whose norm, after the accepted columns before it
/// have been projected out, is below `tolerance` times its own original norm is moved to the end
/// and left out, and the next column takes its place. The columns beyond `pivot_cols` are
/// transformed by every reflector but never factorised or moved.
pub fn pivoted_qr(a: &Matrix, pivot_cols: usize, tolerance: f64) -> PivotedQr {
    assert!(
        pivot_cols <= a.cols(),
        "there are only {} columns to pivot among",
        a.cols()
    );
    let n = a.rows();
    let mut work = a.clone();
    let mut order: Vec<usize> = (0..a.cols()).collect();
    let norms: Vec<f64> = (0..pivot_cols)
        .map(|j| {
            let value = norm(a.column(j));
            if value == 0.0 { 1.0 } else { value }
        })
        .collect();
    let mut limit = pivot_cols;
    let mut j = 0;
    while j < limit && j < n {
        let remaining = norm(&work.column(j)[j..]);
        if remaining < tolerance * norms[order[j]] {
            move_column_to_end(&mut work, j, limit);
            order[j..limit].rotate_left(1);
            limit -= 1;
            continue;
        }
        let segment = work.column(j)[j..].to_vec();
        let r = reflector(&segment);
        {
            let column = work.column_mut(j);
            column[j] = r.beta;
            for value in &mut column[j + 1..] {
                *value = 0.0;
            }
        }
        for c in j + 1..work.cols() {
            apply(&r, &mut work.column_mut(c)[j..]);
        }
        j += 1;
    }
    PivotedQr {
        r: work,
        order,
        rank: j,
        norms,
    }
}

/// Move column `from` to position `to - 1`, shifting the columns between one place left.
fn move_column_to_end(m: &mut Matrix, from: usize, to: usize) {
    let moved = m.column(from).to_vec();
    for c in from..to - 1 {
        let next = m.column(c + 1).to_vec();
        m.column_mut(c).copy_from_slice(&next);
    }
    m.column_mut(to - 1).copy_from_slice(&moved);
}

/// A least-squares solution by the pivoted QR of `[x y]`: the coefficients of the accepted
/// columns (`None` for a column left out as collinear), and the residual sum of squares.
#[derive(Clone, Debug, PartialEq)]
pub struct LeastSquares {
    pub coefficients: Vec<Option<f64>>,
    pub rank: usize,
    pub order: Vec<usize>,
    pub residual_sum_of_squares: f64,
    /// The R factor of the accepted columns, in the order of `order`: `rank` × `rank`, upper
    /// triangular; (XᵀX)⁻¹ of those columns is R⁻¹ R⁻ᵀ and a row's leverage is ‖R⁻ᵀ xᵢ‖².
    pub r: Matrix,
}

/// Solve `x β ≈ y` by the pivoted QR of the R factor of `[x y]`, with `tolerance` the collinearity
/// rule's relative tolerance. The rows of `x` go through the tall-skinny QR in blocks of
/// `block_rows`.
pub fn least_squares(x: &Matrix, y: &[f64], tolerance: f64, block_rows: usize) -> LeastSquares {
    assert_eq!(x.rows(), y.len(), "y needs one value per row of x");
    let k = x.cols();
    let mut augmented = TallSkinnyQr::with_block_rows(k + 1, block_rows);
    let mut row = vec![0.0; k + 1];
    for (i, &yi) in y.iter().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(k) {
            *slot = x.get(i, j);
        }
        row[k] = yi;
        augmented.push_row(&row);
    }
    let r = augmented.finish();
    let pivoted = pivoted_qr(&r, k, tolerance);
    let rank = pivoted.rank;
    let r11 = Matrix::from_fn(rank, rank, |i, j| pivoted.r.get(i, j));
    let qty: Vec<f64> = (0..rank).map(|i| pivoted.r.get(i, k)).collect();
    let solution =
        crate::triangular::solve_upper(&r11, &qty).expect("accepted pivots are not zero");
    let mut coefficients = vec![None; k];
    for (position, &original) in pivoted.order.iter().enumerate().take(rank) {
        coefficients[original] = Some(solution[position]);
    }
    let tail: Vec<f64> = (rank..k + 1).map(|i| pivoted.r.get(i, k)).collect();
    LeastSquares {
        coefficients,
        rank,
        order: pivoted.order,
        residual_sum_of_squares: crate::reduce::sum_of_squares(&tail),
        r: r11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::gram;

    fn close(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() <= tolerance * a.abs().max(b.abs()).max(1.0)
    }

    #[test]
    fn the_identity_and_one_column_factorise_exactly() {
        assert_eq!(r_factor(&Matrix::identity(3)), Matrix::identity(3));
        // (3, 4) has norm 5; alpha = 3 > 0 so beta = -5.
        let r = r_factor(&Matrix::from_columns(&[vec![3.0, 4.0]]));
        assert_eq!(r.get(0, 0), -5.0);
        // A segment already of the form (x, 0, 0) is left as it is.
        let r = r_factor(&Matrix::from_columns(&[
            vec![-2.0, 0.0, 0.0],
            vec![1.0, 2.0, 0.0],
        ]));
        assert_eq!(r.get(0, 0), -2.0);
        assert_eq!(r.get(0, 1), 1.0);
        // The second column's segment under the diagonal is (2, 0): already of the form the
        // reflector produces, so it keeps its sign.
        assert_eq!(r.get(1, 1), 2.0);
        assert_eq!(r.get(1, 0), 0.0);
    }

    #[test]
    fn r_transposed_r_is_the_gram_matrix() {
        let x = Matrix::from_fn(30, 4, |i, j| {
            ((i * 13 + j * 7) % 11) as f64 - 3.0 + 0.25 * (j as f64)
        });
        let r = r_factor(&x);
        let rtr = gram(&r);
        let xtx = gram(&x);
        assert!(
            rtr.max_abs_difference(&xtx) <= 1e-12 * xtx.get(0, 0).abs().max(xtx.get(3, 3).abs())
        );
        for j in 0..4 {
            for i in j + 1..4 {
                assert_eq!(r.get(i, j), 0.0);
            }
        }
    }

    #[test]
    fn fewer_rows_than_columns_leave_zero_rows() {
        let x = Matrix::from_rows(&[vec![1.0, 2.0, 3.0, 4.0], vec![0.0, 1.0, 1.0, 1.0]]);
        let r = r_factor(&x);
        assert_eq!((r.rows(), r.cols()), (4, 4));
        assert_eq!(r.row(2), vec![0.0; 4]);
        assert_eq!(r.row(3), vec![0.0; 4]);
        assert!(gram(&r).max_abs_difference(&gram(&x)) <= 1e-13 * 30.0);
        assert_eq!(r_factor(&Matrix::zeros(0, 2)), Matrix::zeros(2, 2));
    }

    #[test]
    fn the_tall_skinny_qr_folds_block_factors_in_order() {
        let x = Matrix::from_fn(100, 3, |i, j| ((i * 17 + j * 5) % 29) as f64 * 0.1 - 1.0);
        // By hand: each block's R, folded in order.
        let by_hand = {
            let mut r: Option<Matrix> = None;
            for start in (0..100).step_by(16) {
                let block_r = r_factor(&x.rows_slice(start, (start + 16).min(100)));
                r = Some(match r {
                    None => block_r,
                    Some(r) => fold(&r, &block_r),
                });
            }
            r.unwrap()
        };
        // Through the accumulator, rows given one at a time.
        let mut one_at_a_time = TallSkinnyQr::with_block_rows(3, 16);
        for i in 0..100 {
            one_at_a_time.push_row(&x.row(i));
        }
        let r1 = one_at_a_time.finish();
        assert_eq!(r1, by_hand);
        // Rows given in uneven pieces: the same blocks, the same bits.
        let mut in_pieces = TallSkinnyQr::with_block_rows(3, 16);
        for (from, to) in [(0, 7), (7, 40), (40, 41), (41, 100)] {
            in_pieces.push_rows(&x.rows_slice(from, to));
        }
        assert_eq!(in_pieces.finish(), by_hand);
        // And as whole blocks at once.
        let mut whole = TallSkinnyQr::with_block_rows(3, 16);
        whole.push_rows(&x);
        assert_eq!(whole.rows_seen(), 100);
        assert_eq!(whole.finish(), by_hand);
        // The block factors computed in reverse order fold to the same bits, which is what a
        // parallel version will do (open decision #48).
        let reversed: Vec<Matrix> = (0..100)
            .step_by(16)
            .rev()
            .map(|start| r_factor(&x.rows_slice(start, (start + 16).min(100))))
            .collect();
        let mut r = reversed[reversed.len() - 1].clone();
        for block_r in reversed[..reversed.len() - 1].iter().rev() {
            r = fold(&r, block_r);
        }
        assert_eq!(r, by_hand);
        // The folded R has the Gram matrix of the whole.
        assert!(gram(&by_hand).max_abs_difference(&gram(&x)) <= 1e-12 * gram(&x).get(0, 0));
        // No rows at all: a zero R.
        assert_eq!(TallSkinnyQr::new(2).finish(), Matrix::zeros(2, 2));
    }

    #[test]
    fn limited_pivoting_leaves_out_a_collinear_column_and_keeps_the_order() {
        // Column 2 is twice column 0 plus column 1; column 3 is independent.
        let x = Matrix::from_columns(&[
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![1.0, 0.0, 1.0, 0.0, 1.0],
            vec![3.0, 4.0, 7.0, 8.0, 11.0],
            vec![0.0, 1.0, 0.0, 0.0, 2.0],
        ]);
        let p = pivoted_qr(&x, 4, 1e-7);
        assert_eq!(p.rank, 3);
        assert_eq!(p.order, vec![0, 1, 3, 2]);
        assert_eq!(p.norms.len(), 4);
        for j in 0..3 {
            for i in j + 1..5 {
                assert_eq!(p.r.get(i, j), 0.0);
            }
        }
        // A zero column is left out as well, by the rule that its norm counts as one.
        let with_zero = Matrix::from_columns(&[vec![0.0, 0.0, 0.0], vec![1.0, 2.0, 3.0]]);
        let p = pivoted_qr(&with_zero, 2, 1e-7);
        assert_eq!((p.rank, p.order), (1, vec![1, 0]));
        // Nothing to pivot among: everything is transformed, nothing factorised.
        let p = pivoted_qr(&with_zero, 0, 1e-7);
        assert_eq!((p.rank, p.order), (0, vec![0, 1]));
        assert_eq!(p.r, with_zero);
        // More columns than rows: the rank is the row count, and the columns beyond it, never
        // examined, keep their order.
        let wide = Matrix::from_rows(&[vec![1.0, 2.0, 3.0, 4.0], vec![4.0, 5.0, 7.0, 8.0]]);
        let p = pivoted_qr(&wide, 4, 1e-7);
        assert_eq!((p.rank, p.order), (2, vec![0, 1, 2, 3]));
    }

    #[test]
    fn the_pivot_tolerance_is_relative_to_the_column_s_own_norm() {
        // Two columns at scale s that differ by a perpendicular part of relative size delta: the
        // second is left out exactly when delta is below the tolerance, whatever the scale.
        let pair = |s: f64, delta: f64| Matrix::from_columns(&[vec![s, 0.0], vec![s, s * delta]]);
        assert_eq!(pivoted_qr(&pair(1e3, 1e-8), 2, 1e-7).rank, 1);
        assert_eq!(pivoted_qr(&pair(1e3, 1e-6), 2, 1e-7).rank, 2);
        assert_eq!(pivoted_qr(&pair(1e-3, 1e-8), 2, 1e-7).rank, 1);
        assert_eq!(pivoted_qr(&pair(1e-3, 1e-6), 2, 1e-7).rank, 2);
    }

    #[test]
    fn least_squares_recovers_an_exact_fit() {
        // X'X = [[2, 1], [1, 2]], X'y = [4, 5]: beta = (1, 2) exactly, residuals zero.
        let x = Matrix::from_rows(&[vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]]);
        let fit = least_squares(&x, &[1.0, 2.0, 3.0], 1e-7, 2);
        assert_eq!(fit.rank, 2);
        assert!(close(fit.coefficients[0].unwrap(), 1.0, 1e-15));
        assert!(close(fit.coefficients[1].unwrap(), 2.0, 1e-15));
        assert!(fit.residual_sum_of_squares.abs() <= 1e-30);
        // A duplicated column is left out and the fit on the rest is the same.
        let x2 = Matrix::from_columns(&[
            x.column(0).to_vec(),
            x.column(1).to_vec(),
            x.column(0).to_vec(),
        ]);
        let fit2 = least_squares(&x2, &[1.0, 2.0, 3.0], 1e-7, 65_536);
        assert_eq!(fit2.rank, 2);
        assert_eq!(fit2.coefficients[2], None);
        assert!(close(fit2.coefficients[0].unwrap(), 1.0, 1e-15));
        // A residual that is not zero: y = (1, 1, 1) on one column (1, 1, 1): beta = 1, rss = 0;
        // on the column (1, 2, 3): beta = 6/14, rss = 3 - 36/14.
        let fit3 = least_squares(
            &Matrix::from_columns(&[vec![1.0, 2.0, 3.0]]),
            &[1.0, 1.0, 1.0],
            1e-7,
            1,
        );
        assert!(close(fit3.coefficients[0].unwrap(), 6.0 / 14.0, 1e-15));
        assert!(close(
            fit3.residual_sum_of_squares,
            3.0 - 36.0 / 14.0,
            1e-14
        ));
    }
}
