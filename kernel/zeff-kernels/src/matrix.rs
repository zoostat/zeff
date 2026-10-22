//! A dense matrix of Floats in column-major order, the shape every kernel reads and writes.
//! A column is contiguous, which is what the reductions over rows need (a design matrix is read
//! column by column), and what an Arrow column already is.

use std::fmt;

#[derive(Clone, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    /// Column-major: the value at (row, col) is `data[col * rows + row]`.
    data: Vec<f64>,
}

impl Matrix {
    pub fn zeros(rows: usize, cols: usize) -> Matrix {
        Matrix {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn identity(n: usize) -> Matrix {
        let mut m = Matrix::zeros(n, n);
        for i in 0..n {
            m.set(i, i, 1.0);
        }
        m
    }

    /// A matrix from its columns, which must all have one length.
    pub fn from_columns(columns: &[Vec<f64>]) -> Matrix {
        let rows = columns.first().map_or(0, Vec::len);
        assert!(
            columns.iter().all(|c| c.len() == rows),
            "every column of a matrix must have the same length"
        );
        let mut data = Vec::with_capacity(rows * columns.len());
        for column in columns {
            data.extend_from_slice(column);
        }
        Matrix {
            rows,
            cols: columns.len(),
            data,
        }
    }

    /// A matrix from its rows, which must all have one length.
    pub fn from_rows(rows: &[Vec<f64>]) -> Matrix {
        let cols = rows.first().map_or(0, Vec::len);
        assert!(
            rows.iter().all(|r| r.len() == cols),
            "every row of a matrix must have the same length"
        );
        let mut m = Matrix::zeros(rows.len(), cols);
        for (i, row) in rows.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                m.set(i, j, value);
            }
        }
        m
    }

    /// A matrix whose entry at (row, col) is `f(row, col)`.
    pub fn from_fn(rows: usize, cols: usize, f: impl Fn(usize, usize) -> f64) -> Matrix {
        let mut m = Matrix::zeros(rows, cols);
        for j in 0..cols {
            for i in 0..rows {
                m.set(i, j, f(i, j));
            }
        }
        m
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        assert!(
            row < self.rows && col < self.cols,
            "({row}, {col}) is outside a {} × {} matrix",
            self.rows,
            self.cols
        );
        self.data[col * self.rows + row]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        assert!(
            row < self.rows && col < self.cols,
            "({row}, {col}) is outside a {} × {} matrix",
            self.rows,
            self.cols
        );
        self.data[col * self.rows + row] = value;
    }

    /// Column `col`, contiguous.
    pub fn column(&self, col: usize) -> &[f64] {
        assert!(
            col < self.cols,
            "column {col} of a matrix with {} columns",
            self.cols
        );
        &self.data[col * self.rows..(col + 1) * self.rows]
    }

    pub fn column_mut(&mut self, col: usize) -> &mut [f64] {
        assert!(
            col < self.cols,
            "column {col} of a matrix with {} columns",
            self.cols
        );
        let rows = self.rows;
        &mut self.data[col * rows..(col + 1) * rows]
    }

    /// Row `row`, gathered into a new vector.
    pub fn row(&self, row: usize) -> Vec<f64> {
        (0..self.cols).map(|j| self.get(row, j)).collect()
    }

    /// Rows `from..to`, as a new matrix.
    pub fn rows_slice(&self, from: usize, to: usize) -> Matrix {
        assert!(
            from <= to && to <= self.rows,
            "rows {from}..{to} of a matrix with {} rows",
            self.rows
        );
        Matrix::from_fn(to - from, self.cols, |i, j| self.get(from + i, j))
    }

    pub fn transpose(&self) -> Matrix {
        Matrix::from_fn(self.cols, self.rows, |i, j| self.get(j, i))
    }

    /// `top` over `bottom`: the rows of both, in order; they must have the same number of columns.
    pub fn stack(top: &Matrix, bottom: &Matrix) -> Matrix {
        assert_eq!(
            top.cols, bottom.cols,
            "stacked matrices must have the same number of columns"
        );
        let mut data = Vec::with_capacity((top.rows + bottom.rows) * top.cols);
        for j in 0..top.cols {
            data.extend_from_slice(top.column(j));
            data.extend_from_slice(bottom.column(j));
        }
        Matrix {
            rows: top.rows + bottom.rows,
            cols: top.cols,
            data,
        }
    }

    /// The storage, column by column.
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// The largest absolute difference between two matrices of one shape.
    pub fn max_abs_difference(&self, other: &Matrix) -> f64 {
        assert!(
            self.rows == other.rows && self.cols == other.cols,
            "the matrices differ in shape"
        );
        self.data
            .iter()
            .zip(&other.data)
            .fold(0.0, |m, (a, b)| m.max((a - b).abs()))
    }
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Matrix {} × {}", self.rows, self.cols)?;
        for i in 0..self.rows {
            let row: Vec<String> = (0..self.cols)
                .map(|j| format!("{:?}", self.get(i, j)))
                .collect();
            writeln!(f, "  [{}]", row.join(", "))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_and_rows_build_the_same_matrix() {
        let by_columns = Matrix::from_columns(&[vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]]);
        let by_rows = Matrix::from_rows(&[vec![1.0, 3.0, 5.0], vec![2.0, 4.0, 6.0]]);
        assert_eq!(by_columns, by_rows);
        assert_eq!(by_columns.rows(), 2);
        assert_eq!(by_columns.cols(), 3);
        assert_eq!(by_columns.get(1, 2), 6.0);
        assert_eq!(by_columns.column(1), &[3.0, 4.0]);
        assert_eq!(by_columns.row(0), vec![1.0, 3.0, 5.0]);
        assert_eq!(by_columns.data(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn transpose_stack_and_slices() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]]);
        assert_eq!(
            m.transpose(),
            Matrix::from_rows(&[vec![1.0, 3.0, 5.0], vec![2.0, 4.0, 6.0]])
        );
        assert_eq!(m.transpose().transpose(), m);
        let top = m.rows_slice(0, 1);
        let bottom = m.rows_slice(1, 3);
        assert_eq!(Matrix::stack(&top, &bottom), m);
        assert_eq!(m.rows_slice(1, 1).rows(), 0);
        let mut i = Matrix::identity(2);
        i.set(0, 1, 7.0);
        assert_eq!(i.column_mut(1), &mut [7.0, 1.0]);
        assert_eq!(
            Matrix::from_fn(2, 2, |r, c| (r * 10 + c) as f64).row(1),
            vec![10.0, 11.0]
        );
        assert_eq!(
            m.max_abs_difference(&Matrix::from_fn(3, 2, |r, c| m.get(r, c) + 0.5)),
            0.5
        );
    }

    #[test]
    fn an_empty_matrix_is_allowed() {
        let empty = Matrix::from_columns(&[]);
        assert_eq!((empty.rows(), empty.cols()), (0, 0));
        assert_eq!(Matrix::zeros(0, 3).cols(), 3);
        assert_eq!(Matrix::zeros(3, 0).transpose().rows(), 0);
    }

    #[test]
    #[should_panic(expected = "same length")]
    fn ragged_columns_are_refused() {
        Matrix::from_columns(&[vec![1.0], vec![1.0, 2.0]]);
    }
}
