//! Corners of the factorial `anova` that nothing held until the weekly mutation job of release
//! 0.7.0 showed them (step 40a; Testing Handbook, section 1; Language Reference, sections 6 and
//! 10.2): the two messages of a model left without rows, the note of columns that carry no
//! degree of freedom when there are several and when no two categories meet, a covariate that
//! is zero in every row, an effect of exactly zero, and an outcome at the largest size the
//! arithmetic takes.

use zeff_testkit::run;

const TWO_BY_TWO: &str = "a = [\"p\", \"p\", \"p\", \"p\", \"q\", \"q\", \"q\", \"q\"], b = [\"u\", \"u\", \"v\", \"v\", \"u\", \"u\", \"v\", \"v\"]";

/// What a script that runs prints, and its notes.
fn printed(script: &str) -> (String, String) {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    (outcome.output.stdout.clone(), outcome.output.stderr.clone())
}

/// The message of a script that is refused, which prints nothing.
fn refused(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 1, "{script}");
    assert_eq!(outcome.output.stdout, "", "{script}");
    outcome.output.stderr.clone()
}

/// A frame of no rows is said to have none; rows that were all dropped, for an `NA` or for a
/// weight of zero, are said to have been dropped.
#[test]
fn a_model_left_without_rows_says_why() {
    let none = refused(&format!(
        "frame({TWO_BY_TWO}, y = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]).where(y > 100).anova(y ~ category(a) * category(b))\n"
    ));
    assert!(
        none.contains("error[Z0801]: no row remains for `anova`: the frame has no rows\n"),
        "{none}"
    );
    let dropped = "error[Z0801]: no row remains for `anova`: every row has NA in a model variable or a zero weight\n";
    let missing = refused(
        "frame(a = [\"p\", \"p\", \"p\", \"p\", NA, NA, NA, NA], b = [\"u\", \"u\", \"v\", \"v\", \"u\", \"u\", \"v\", \"v\"], y = [NA, NA, NA, NA, 5.0, 6.0, 7.0, 8.0]).anova(y ~ category(a) * category(b))\n",
    );
    assert!(missing.contains(dropped), "{missing}");
    let weightless = refused(&format!(
        "frame({TWO_BY_TWO}, y = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], w = [0, 0, 0, 0, 0, 0, 0, 0]).anova(y ~ category(a) * category(b), weights: w, weight_type: analytic)\n"
    ));
    assert!(weightless.contains(dropped), "{weightless}");
}

/// Two categories that repeat one another, with no interaction in the model: two of the five
/// columns carry nothing, and the note does not speak of empty cells, since no two categories
/// meet in a term. The model's own row is that of either category alone: in R,
/// `anova(lm(y ~ a))` gives F = 1.515152 on 2 and 6 degrees of freedom, p = 0.2933.
#[test]
fn categories_that_repeat_one_another_carry_no_degrees_of_freedom() {
    let (out, notes) = printed(
        "frame(a = [\"p\", \"p\", \"p\", \"q\", \"q\", \"q\", \"r\", \"r\", \"r\"], b = [\"u\", \"u\", \"u\", \"v\", \"v\", \"v\", \"w\", \"w\", \"w\"], y = [1.0, 2.0, 4.0, 3.0, 5.0, 9.0, 2.0, 8.0, 7.0]).anova(y ~ category(a) + category(b))\n",
    );
    assert!(
        out.contains("  model       2   22.222   11.111   1.52     0.293"),
        "{out}"
    );
    assert!(
        notes.contains(
            "note: 2 of the model's 5 columns are determined by the others (a term that repeats the others) and carry no degree of freedom"
        ),
        "{notes}"
    );
    assert!(
        notes.contains("note: `a` has no degrees of freedom"),
        "{notes}"
    );
    assert!(
        notes.contains("note: `b` has no degrees of freedom"),
        "{notes}"
    );
}

/// Two empty cells of a three by two table: two of the six columns carry nothing, and the note
/// names the empty cells as a cause.
#[test]
fn two_empty_cells_leave_two_columns_without_a_degree_of_freedom() {
    let (out, notes) = printed(
        "frame(a = [\"p\", \"p\", \"p\", \"p\", \"q\", \"q\", \"q\", \"q\", \"r\", \"r\", \"r\", \"r\"], b = [\"u\", \"u\", \"v\", \"v\", \"u\", \"u\", \"u\", \"u\", \"v\", \"v\", \"v\", \"v\"], y = [1.0, 2.0, 4.0, 3.0, 5.0, 9.0, 2.0, 8.0, 7.0, 6.0, 3.5, 1.5]).anova(y ~ category(a) * category(b))\n",
    );
    assert!(out.contains("  a:b         0    0.000\n"), "{out}");
    assert!(
        notes.contains(
            "note: 2 of the model's 6 columns are determined by the others (a combination of levels with no observations, or a term that repeats the others) and carry no degree of freedom"
        ),
        "{notes}"
    );
}

/// A covariate that is zero in every row is a column of zeros: it has no degree of freedom and
/// the rest of the table is that of the model without it (`anova(lm(y ~ a))` in R: F = 15.76364
/// on 1 and 6 degrees of freedom, p = 0.007364).
#[test]
fn a_covariate_of_zeros_has_no_degree_of_freedom() {
    let (out, notes) = printed(&format!(
        "frame({TWO_BY_TWO}, y = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0], x = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]).anova(y ~ category(a) + x)\n"
    ));
    assert!(
        out.contains("  a           1   36.125   36.125   15.76     0.007                 0.724\n"),
        "{out}"
    );
    assert!(out.contains("  x           0    0.000\n"), "{out}");
    assert!(
        notes.contains("note: 1 of the model's 3 columns is determined by the others"),
        "{notes}"
    );
}

/// Cells of equal means: every effect is exactly zero, which is not below zero, so nothing is
/// said of a negative effect size.
#[test]
fn an_effect_of_exactly_zero_has_no_note() {
    let (out, notes) = printed(&format!(
        "frame({TWO_BY_TWO}, y = [1.0, 3.0, 1.0, 3.0, 1.0, 3.0, 1.0, 3.0]).anova(y ~ category(a) * category(b))\n"
    ));
    assert!(
        out.contains("  model       3   0.000   0.000   0.00     1.000                 0.000\n"),
        "{out}"
    );
    assert_eq!(notes, "", "{notes}");
}

/// An outcome with a value of 1e150 is computed; only a larger one is refused (Z0803). One
/// value towers over the rest, so each term has the same sum of squares and F is 1.
#[test]
fn an_outcome_at_the_largest_size_is_computed() {
    let (out, _) = printed(&format!(
        "frame({TWO_BY_TWO}, y = [1e150, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]).anova(y ~ category(a) * category(b))\n"
    ));
    assert!(
        out.contains(
            "  a:b         1   1.250e299   1.250e299   1.00     0.374                 0.200\n"
        ),
        "{out}"
    );
    let message = refused(&format!(
        "frame({TWO_BY_TWO}, y = [1.1e150, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]).anova(y ~ category(a) * category(b))\n"
    ));
    assert!(message.contains("error[Z0803]"), "{message}");
}

/// One residual degree of freedom is enough: five rows in four cells. `car::Anova(lm(y ~ a *
/// b), type = 2)` gives F = 1.9101 (p = 0.3988) for `a`, 0.1323 (0.7779) for `b` and 0.0159
/// (0.9202) for their interaction, on a residual sum of squares of 4.5.
#[test]
fn one_residual_degree_of_freedom_is_enough() {
    let (out, _) = printed(
        "frame(a = [\"p\", \"p\", \"q\", \"q\", \"q\"], b = [\"u\", \"v\", \"u\", \"v\", \"v\"], y = [1.0, 2.0, 4.0, 3.0, 6.0]).anova(y ~ category(a) * category(b))\n",
    );
    assert!(
        out.contains("  a           1    8.595   8.595   1.91     0.399                 0.656\n"),
        "{out}"
    );
    assert!(
        out.contains("  a:b         1    0.071   0.071   0.02     0.920                 0.016\n"),
        "{out}"
    );
    assert!(out.contains("  residual    1    4.500   4.500\n"), "{out}");
}
