//! Corners of the tests on counts that nothing held until the weekly mutation job of release
//! 0.7.0 showed them (step 40a; Testing Handbook, section 1; Language Reference, sections 6 and
//! 10.2): a table of three columns through a script, a column emptied by its weights, a
//! continuity correction smaller than a half, a chi-squared test of more than two columns and
//! a goodness of fit with frequency weights, an outcome held as 0.0 and 1.0, a share above the
//! hypothesised one, and the refusals at the ends of the range of `p:`. Each figure is R
//! 4.6.0's, named beside it.

use zeff_testkit::run;

/// What a script that runs prints.
fn printed(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    outcome.output.stdout.clone()
}

/// The message of a script that is refused, which prints nothing.
fn refused(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 1, "{script}");
    assert_eq!(outcome.output.stdout, "", "{script}");
    outcome.output.stderr.clone()
}

/// `fisher.test(matrix(c(3, 1, 2, 1, 4, 2), 2, byrow = TRUE))`: p = 0.3240093.
#[test]
fn fisher_test_takes_three_columns() {
    let out = printed(
        "let t = frame(a = [\"x\", \"x\", \"x\", \"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\"], b = [1, 1, 1, 2, 3, 3, 1, 2, 2, 2, 2, 3, 3])\nt.fisher_test(a, b)\n",
    );
    assert!(out.contains("\np = 0.324\n"), "{out}");
    assert!(
        out.contains("exact p-value over the 21 tables with these margins"),
        "{out}"
    );
}

/// A column whose weights are all zero has no rows: it is left out and said to be, and the
/// test is that of the table without it. `fisher.test(matrix(c(3, 1, 1, 4), 2, byrow = TRUE))`:
/// p = 0.2063492; the odds ratio solved tightly is 8.354, where R's own solver stops at 8.355.
#[test]
fn a_column_emptied_by_its_weights_is_left_out_of_fisher_test() {
    let out = printed(
        "let t = frame(a = [\"x\", \"x\", \"x\", \"y\", \"y\", \"y\"], b = [1, 2, 3, 1, 2, 3], w = [3, 1, 0, 1, 4, 0])\nt.fisher_test(a, b, weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("\np = 0.206\n"), "{out}");
    assert!(
        out.contains("odds ratio (conditional MLE) = 8.354"),
        "{out}"
    );
    assert!(out.contains("left out, having no rows: b = 3\n"), "{out}");
    assert!(out.contains("  a       1   2   Total\n"), "{out}");
}

/// Two rows and one column left by the weights: the refusal names the column variable.
#[test]
fn fisher_test_names_the_variable_left_with_one_value() {
    let message = refused(
        "frame(a = [\"x\", \"x\", \"y\", \"y\"], b = [1, 2, 1, 2], w = [2, 0, 3, 0]).fisher_test(a, b, weights: w, weight_type: frequency)\n",
    );
    assert!(
        message.contains(
            "error[Z0401]: `fisher_test` needs at least two values of `b` with rows, but there is one"
        ),
        "{message}"
    );
}

/// The continuity correction never exceeds a difference: on the table 5, 5, 5, 6 every count is
/// 0.238 from its expected one, so the corrected statistic is 0 and p is 1
/// (`chisq.test(matrix(c(5, 5, 5, 6), 2, byrow = TRUE))`: X-squared = 0, p-value = 1); without
/// the correction, X-squared = 0.04338843 and p = 0.8349956.
#[test]
fn the_continuity_correction_of_chi2_test_never_exceeds_a_difference() {
    let out = printed(
        "let t = frame(a = [\"x\", \"x\", \"y\", \"y\"], b = [1, 2, 1, 2], w = [5, 5, 5, 6])\nt.chi2_test(a, b, correct: true, weights: w, weight_type: frequency)\nt.chi2_test(a, b, weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("chi2(1) = 0.00   p = 1.000\n"), "{out}");
    assert!(out.contains("chi2(1) = 0.04   p = 0.835\n"), "{out}");
}

/// Three rows by three columns, four degrees of freedom:
/// `chisq.test(matrix(c(10, 5, 3, 4, 12, 6, 2, 7, 11), 3, byrow = TRUE))` gives X-squared =
/// 14.83056, df = 4, p-value = 0.005065872.
#[test]
fn chi2_test_of_three_rows_by_three_columns() {
    let out = printed(
        "let t = frame(a = [\"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"z\", \"z\", \"z\"], b = [1, 2, 3, 1, 2, 3, 1, 2, 3], w = [10, 5, 3, 4, 12, 6, 2, 7, 11])\nt.chi2_test(a, b, weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("chi2(4) = 14.83   p = 0.005\n"), "{out}");
}

/// A goodness of fit on frequency weights: `chisq.test(c(12, 7, 5), p = c(0.5, 0.3, 0.2))`
/// gives X-squared = 0.01388889, df = 2, p-value = 0.9930796. A row of weight zero adds
/// nothing and its value keeps its place.
#[test]
fn the_goodness_of_fit_counts_frequency_weights() {
    let out = printed(
        "let t = frame(a = [\"x\", \"y\", \"z\", \"z\"], w = [12, 7, 5, 0])\nt.chi2_test(a, p: [0.5, 0.3, 0.2], weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("chi2(2) = 0.01   p = 0.993\n"), "{out}");
    assert!(out.contains("Total         24       24.0     1\n"), "{out}");
}

/// Proportions that sum to 1 with one of them zero or negative are no distribution.
#[test]
fn the_goodness_of_fit_refuses_a_share_that_is_not_positive() {
    for p in ["[1.2, -0.2]", "[1.0, 0.0]"] {
        let message = refused(&format!(
            "frame(a = [\"x\", \"y\"], w = [12, 7]).chi2_test(a, p: {p}, weights: w, weight_type: frequency)\n"
        ));
        assert!(
            message.contains(
                "error[Z0401]: `p:` of `chi2_test` must be positive proportions that sum to 1, but these sum to 1"
            ),
            "{message}"
        );
    }
}

/// An outcome held as 0.0 and 1.0 is counted as one of 0 and 1, and the correction of the
/// one-sample test never exceeds the difference: `prop.test(5, 10, p = 0.52)` gives X-squared
/// = 0 and p-value = 1. A value between the two is refused.
#[test]
fn proportion_test_counts_an_outcome_of_float_zeros_and_ones() {
    let out = printed(
        "let t = frame(y = [1.0, 0.0], w = [5, 5])\nt.proportion_test(y, p: 0.52, correct: true, weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("N = 10   successes = 5\n"), "{out}");
    assert!(out.contains("z = 0.00   p = 1.000\n"), "{out}");
    let message = refused("frame(y = [1.0, 0.5, 0.0]).proportion_test(y, p: 0.5)\n");
    assert!(
        message.contains(
            "error[Z0401]: `proportion_test` counts a Bool, or a variable of 0 and 1, but `y` holds 0.5"
        ),
        "{message}"
    );
}

/// A share above the hypothesised one has a positive z: seven of ten against a half.
/// `prop.test(7, 10, p = 0.5, alternative = "greater")` gives X-squared = 0.9 (z = 0.949) and
/// p-value = 0.1713909; with `correct = FALSE`, X-squared = 1.6 (z = 1.265) and p-value =
/// 0.1029516; and `alternative = "less"`, p-value = 0.8970484.
#[test]
fn a_share_above_the_hypothesised_one_has_a_positive_z() {
    let out = printed(
        "let t = frame(y = [1.0, 0.0], w = [7, 3])\nt.proportion_test(y, p: 0.5, correct: true, alternative: greater, weights: w, weight_type: frequency)\nt.proportion_test(y, p: 0.5, alternative: greater, weights: w, weight_type: frequency)\nt.proportion_test(y, p: 0.5, alternative: less, weights: w, weight_type: frequency)\n",
    );
    assert!(out.contains("z = 0.95   p = 0.171\n"), "{out}");
    assert!(out.contains("z = 1.26   p = 0.103\n"), "{out}");
    assert!(out.contains("z = 1.26   p = 0.897\n"), "{out}");
}

/// `p:` is strictly between 0 and 1: the ends are refused, written as whole numbers or not.
#[test]
fn proportion_test_refuses_the_ends_of_the_range_of_p() {
    for (p, shown) in [("0", "0"), ("1", "1"), ("0.0", "0"), ("1.0", "1")] {
        let message = refused(&format!(
            "frame(y = [true, false]).proportion_test(y, p: {p})\n"
        ));
        assert!(
            message.contains(&format!(
                "error[Z0401]: `p:` of `proportion_test` is a proportion strictly between 0 and 1, but this is {shown}\n"
            )),
            "{message}"
        );
    }
}

/// An expected count of exactly 5 is not below 5: four cells of 5 raise no warning
/// (`chisq.test(matrix(5, 2, 2), correct = FALSE)`: X-squared = 0, p-value = 1, no warning).
#[test]
fn an_expected_count_of_exactly_five_raises_no_warning() {
    let outcome = run(
        "frame(a = [\"x\", \"x\", \"y\", \"y\"], b = [1, 2, 1, 2], w = [5, 5, 5, 5]).chi2_test(a, b, weights: w, weight_type: frequency)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .contains("chi2(1) = 0.00   p = 1.000\n"),
        "{}",
        outcome.output.stdout
    );
    assert_eq!(outcome.output.stderr, "");
}

/// Two rows and one column: the refusal names the column variable and points at it.
#[test]
fn chi2_test_names_the_variable_with_one_value() {
    let message =
        refused("frame(a = [\"x\", \"x\", \"y\", \"y\"], b = [1, 1, 1, 1]).chi2_test(a, b)\n");
    assert!(
        message.contains(
            "error[Z0401]: `chi2_test` needs at least two values of `b`, but there is one\n"
        ),
        "{message}"
    );
    assert!(
        message.contains(" --> scripts/main.zeff:1:64\n"),
        "{message}"
    );
}
