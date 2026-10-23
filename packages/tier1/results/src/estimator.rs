//! The estimator trait (Engine and Project Specification, section 8.3, point 2): every model
//! provides its design, its fit, each observation's scores and its bread, and one variance layer
//! computes every kind of standard error from the last two. `Design::build` is the design step
//! every estimator shares.

use crate::design::{Block, Design};
use zeff_core::runtime::{CallContext, Fail};
use zeff_kernels::Matrix;

/// What a fit found: the columns kept and dropped, the estimates of the kept columns, the
/// scaled R factor the variance layer reads, and the rows' residuals, fitted values and
/// leverages.
#[derive(Debug, Clone, PartialEq)]
pub struct Fitted {
    /// The design's columns with an estimate, in the design's order.
    pub kept: Vec<usize>,
    /// The design's columns dropped as collinear with the columns before them.
    pub omitted: Vec<usize>,
    /// One estimate per kept column, in the units of the data.
    pub estimates: Vec<f64>,
    /// The R factor of the kept columns scaled to unit norm, `kept.len()` square.
    pub r: Matrix,
    /// The scale of each kept column: the reciprocal of its norm.
    pub scales: Vec<f64>,
    pub residuals: Vec<f64>,
    pub fitted: Vec<f64>,
    pub leverage: Vec<f64>,
    pub residual_sum_of_squares: f64,
    /// The condition number of the scaled R.
    pub condition_number: f64,
}

/// One model: how it is fitted, and what the variance layer needs from it.
pub trait Estimator {
    /// Estimates and what the fit found; an iterative fit adds its convergence to the notes.
    fn fit(&self, ctx: &mut CallContext<'_>, design: &Design) -> Result<Fitted, Fail>;

    /// Each observation's contribution to the estimating equations, for the rows of a block:
    /// one row per observation, one column per kept term.
    fn scores(&self, fitted: &Fitted, block: &Block, first_row: usize) -> Matrix;

    /// The Hessian or its equivalent: for least squares, (XᵀX)⁻¹ in the units of the data.
    fn bread(&self, fitted: &Fitted) -> Matrix;
}
