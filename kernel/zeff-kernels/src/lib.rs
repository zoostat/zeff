//! Engine kernel: the linear algebra of the estimators (Engine and Project Specification,
//! section 8.1, rule 8, "linear algebra (`zeff-kernels`)"), written so that the determinism
//! contract of section 8.3, point 3, holds by construction:
//!
//! * Every reduction over rows (a sum, a dot product, a norm) has one fixed shape, open decision
//!   #47: `LANES` independent accumulators, element `i` going to accumulator `i mod LANES`, each
//!   compensated, the accumulators combined in a fixed order at the end. The bits depend on that
//!   shape alone, never on whether a loop was vectorised, how wide the vectors were, or how the
//!   rows were handed over in blocks.
//! * Rows are consumed in blocks of a fixed size (`qr::BLOCK_ROWS`, open decision #48): the
//!   tall-skinny QR factorises each block on its own and folds the block's R factor into the
//!   running one in block order, so a later parallel version computing the blocks on several
//!   threads gives the same bits as this sequential one.
//! * Only exactly specified IEEE 754 operations are used (`+`, `-`, `*`, `/`, `sqrt`, `abs`,
//!   `copysign`); no fused multiply-add, no platform maths library, no `unsafe`, no intrinsics:
//!   the fixed lane shape makes the compiler's vectorisation of a loop invisible in the result.
//!
//! Matrices are dense and column-major. Inputs are finite Floats; a value whose square overflows
//! (above about 1e150) is outside what these routines promise, as the column metadata of a
//! model rules such values out before they arrive.

pub mod cholesky;
pub mod matrix;
pub mod product;
pub mod qr;
pub mod reduce;
pub mod svd;
pub mod triangular;

pub use matrix::Matrix;
pub use reduce::{LANES, LaneSum, Lanes, dot, norm, sum, sum_of_squares};
