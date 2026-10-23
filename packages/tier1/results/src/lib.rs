//! Package results (tier 1): the model matrix of section 7.2 of the Language Reference, the
//! estimator trait and the variance layer (Engine and Project Specification, section 8.3, point
//! 2), and the shared result value assembled with its printed summary (section 8.1, rule 5). It
//! registers no script function yet; the result methods arrive with step 15.

pub mod design;
pub mod estimator;
pub mod summary;
pub mod variance;

use zeff_core::runtime::Function;

/// The functions of `results` in this release: none yet; the result methods arrive with step 15.
pub fn functions() -> Vec<Function> {
    Vec::new()
}
