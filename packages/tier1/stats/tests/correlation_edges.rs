//! Corners of the p-values of `correlation` that nothing held until the weekly mutation job of
//! release 0.7.0 showed them (step 40a; Testing Handbook, section 1; Language Reference,
//! section 6): the fewest values that have a p-value, ties in one variable alone, a negative
//! Kendall's tau by its exact distribution, and a p-value below a thousandth. Each figure is
//! that of R 4.6.0's `cor.test`, named beside it.

use zeff_testkit::run;

/// The table of p-values a script prints: the lines after the heading `p-values`.
fn p_table(frame: &str, method: &str) -> String {
    let outcome = run(&format!(
        "frame({frame}).correlation([x, y], method: {method}, p_values: true)\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let out = outcome.output.stdout.clone();
    let at = out.find("p-values\n").unwrap_or_else(|| panic!("{out}"));
    out[at..].to_string()
}

/// Pearson's r has a p-value from three values (one degree of freedom) and none on two:
/// `cor.test(c(1, 2, 4), c(2, 1, 5))` gives r = 0.8386279 and p-value = 0.3667166.
#[test]
fn pearson_has_a_p_value_from_three_values() {
    let three = p_table("x = [1.0, 2.0, 4.0], y = [2.0, 1.0, 5.0]", "pearson");
    assert!(three.contains("  x           0.367\n"), "{three}");
    let two = p_table("x = [1.0, 2.0], y = [2.0, 5.0]", "pearson");
    assert!(two.contains("  x        NA\n"), "{two}");
}

/// Spearman's rho has a p-value from two values: `cor.test(c(1, 2), c(2, 5), method =
/// "spearman")` gives p-value = 1.
#[test]
fn spearman_has_a_p_value_from_two_values() {
    let two = p_table("x = [1.0, 2.0], y = [2.0, 5.0]", "spearman");
    assert!(two.contains("  x           1.000\n"), "{two}");
}

/// Ties in one variable alone are ties: the approximation is used, as R uses it
/// (`cor.test(c(1, 2, 2, 4, 5, 6, 7), c(2, 1, 4, 3, 7, 6, 9), method = "spearman")`: rho =
/// 0.8468812, p-value = 0.01619713; the exact distribution would give another figure).
#[test]
fn ties_in_one_variable_take_spearman_to_the_approximation() {
    let tied = p_table(
        "x = [1.0, 2.0, 2.0, 4.0, 5.0, 6.0, 7.0], y = [2.0, 1.0, 4.0, 3.0, 7.0, 6.0, 9.0]",
        "spearman",
    );
    assert!(tied.contains("  x           0.016\n"), "{tied}");
}

/// A negative tau takes the lower tail of the exact distribution: `cor.test(1:8, c(8, 7, 5, 6,
/// 4, 2, 3, 1), method = "kendall")` gives tau = −0.8571429 and p-value = 0.001736111.
#[test]
fn a_negative_tau_has_the_exact_p_value_of_its_own_tail() {
    let down = p_table(
        "x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], y = [8.0, 7.0, 5.0, 6.0, 4.0, 2.0, 3.0, 1.0]",
        "kendall",
    );
    assert!(down.contains("  x           0.002\n"), "{down}");
    // Six pairs of twenty-eight in the same order, tau = −0.5714286: p-value = 0.0610119.
    let fewer = p_table(
        "x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], y = [7.0, 5.0, 8.0, 3.0, 6.0, 4.0, 1.0, 2.0]",
        "kendall",
    );
    assert!(fewer.contains("  x           0.061\n"), "{fewer}");
    // Twelve pairs, two fewer than half, tau = −0.1428571: p-value = 0.7195437.
    let just_below = p_table(
        "x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], y = [6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 8.0, 7.0]",
        "kendall",
    );
    assert!(just_below.contains("  x           0.720\n"), "{just_below}");
    // Fifteen pairs, one more than half, tau = 0.07142857: the upper tail, p-value = 0.9048611.
    let just_above = p_table(
        "x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], y = [5.0, 6.0, 3.0, 4.0, 2.0, 1.0, 7.0, 8.0]",
        "kendall",
    );
    assert!(just_above.contains("  x           0.905\n"), "{just_above}");
}

/// A p-value below a thousandth is printed as `<0.001` (`cor.test` on these ten pairs gives
/// p-value = 5.86e-11).
#[test]
fn a_p_value_below_a_thousandth_is_printed_as_such() {
    let small = p_table(
        "x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0], y = [1.1, 2.3, 2.9, 4.2, 5.1, 5.8, 7.2, 8.1, 8.8, 10.3]",
        "pearson",
    );
    assert!(small.contains("  x            <0.001\n"), "{small}");
}
