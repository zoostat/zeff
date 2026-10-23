//! The variance layer (Engine and Project Specification, section 8.3, point 2): one computation
//! of every kind of standard error from an estimator's scores and bread. At this step the
//! classical variance; the sandwich kinds follow at step 11.

use crate::estimator::Fitted;
use zeff_kernels::Matrix;

/// The classical variance of the estimates, σ² (XᵀX)⁻¹ with σ² the residual sum of squares over
/// the residual degrees of freedom.
pub fn classical(fitted: &Fitted, bread: &Matrix, df_residual: usize) -> Matrix {
    let sigma2 = fitted.residual_sum_of_squares / df_residual as f64;
    Matrix::from_fn(bread.rows(), bread.cols(), |i, j| sigma2 * bread.get(i, j))
}
