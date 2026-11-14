//! The fitted values and the residuals of a weighted fit are those of the rows as they are, not
//! of the rows scaled by the square roots of their weights (release 0.7.0, step 40a: the weekly
//! mutation job showed that no test of this package held the fitted values under weights;
//! Language Reference, section 6, `fitted` and `residuals`). Held to R 4.6.0:
//! `m <- lm(y ~ x, weights = w)`, then `fitted(m)` and `resid(m)`.

use zeff_testkit::run;

#[test]
fn the_fitted_values_and_residuals_of_a_weighted_fit_are_those_of_the_rows() {
    let outcome = run(
        "let d = frame(y = [1.0, 3.0, 2.0, 5.0, 4.0, 7.0], x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0], w = [1.0, 2.0, 1.0, 3.0, 1.0, 2.0])\nlet m = d.reg(y ~ x, weights: w, weight_type: analytic)\nprint(m.fitted())\nprint(m.residuals())\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let out = outcome.output.stdout.clone();
    // fitted(m): 1.344828, 2.402299, 3.459770, 4.517241, 5.574713, 6.632184.
    assert!(
        out.contains("1.34483\n2.4023\n3.45977\n4.51724\n5.57471\n6.63218\n"),
        "{out}"
    );
    // resid(m): -0.344828, 0.597701, -1.459770, 0.482759, -1.574713, 0.367816.
    assert!(
        out.contains("-0.344828\n0.597701\n-1.45977\n0.482759\n-1.57471\n0.367816\n"),
        "{out}"
    );
}
