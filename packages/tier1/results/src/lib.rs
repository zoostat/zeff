//! Package results (tier 1): the model matrix of section 7.2 of the Language Reference (`design`
//! of the estimator trait, Engine and Project Specification, section 8.3, point 2), and, as release
//! 0.4.0 proceeds, the result and estimator interfaces every model package implements (section
//! 8.1, rule 5). It registers no script function yet.

pub mod design;

use zeff_core::runtime::Function;

/// The functions of `results` in this release: none yet; the result methods arrive with step 15.
pub fn functions() -> Vec<Function> {
    Vec::new()
}
