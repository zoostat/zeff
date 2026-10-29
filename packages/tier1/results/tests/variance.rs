//! The variance layer's public pieces (Engine and Project Specification, section 8.3, point 2):
//! the effective sample size of the weights. The layer's computations are tested through `reg`
//! in the regression tests, and its symmetry check by its own unit tests.

use zeff_results::variance::effective_sample_size;

/// Kish's effective sample size, (Σw)² / Σw²: the row count under equal weights, one under a
/// single nonzero weight, and zero, not a division of zero by zero, when every weight is zero or
/// there are none.
#[test]
fn the_effective_sample_size_is_kish() {
    assert_eq!(effective_sample_size(&[2.0, 2.0, 2.0, 2.0]), 4.0);
    assert_eq!(effective_sample_size(&[0.0, 5.0, 0.0]), 1.0);
    assert_eq!(effective_sample_size(&[0.0, 0.0]), 0.0);
    assert_eq!(effective_sample_size(&[]), 0.0);
}
