//! Values too large to square (release 0.7.0, step 40a; Language Reference, section 11, Z0803).
//! A test that squares its values cannot compute with one above 1e150 in size: the square is
//! more than the arithmetic holds. Until this step `t_test` printed a standard deviation of
//! `Inf`, t = 0.00 and p = 1.000 for such a variable, as R does; the classical one-way `anova`
//! left a group whose variance overflowed out of the pooled variance and printed F = `Inf`;
//! and Pearson's `correlation` printed 0. Found by a mutant of `anova` that the tests did not
//! catch. The three now refuse the variable, as `reg` and the factorial `anova` do, and at
//! 1e150 itself they give what the same data give at their ordinary size (R 4.6.0 on the
//! data divided by 1e150: t 2.1693 for one mean and 0.225494 for two groups; F 0.736264 by the
//! classical form and 1.46824 by Welch's; r 0.329582).

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const DATA: &str = "let small = frame(y = [0.2, 0.4, 0.6, 1.0, 0.1, 0.3, 0.9, 0.5, 0.7], z = [1.0, 2, 3, 4, 5, 6, 7, 8, 10], g = [1, 1, 1, 2, 2, 2, 3, 3, 3], h = [1, 1, 1, 1, 2, 2, 2, 2, 2])\nlet at_limit = small.add(y = y * 1e150)\nlet beyond = small.add(y = y * 1e151)\n";

#[test]
fn a_variable_above_1e150_is_refused_by_the_tests_that_square_it() {
    for statement in [
        "beyond.t_test(y, mu: 3e150)",
        "beyond.t_test(y, group: h)",
        "beyond.t_test(y, group: h, equal_var: true)",
        "beyond.t_test(y, z, paired: true)",
        "beyond.t_test(z, y, paired: true)",
        "beyond.anova(y, group: g)",
        "beyond.anova(y, group: g, equal_var: true)",
        "beyond.anova(y, group: g, equal_var: true).posthoc()",
        "beyond.correlation([y, z])",
        "beyond.correlation([z, y], p_values: true)",
    ] {
        let (exit, stdout, stderr) = go(&format!("{DATA}{statement}\n"));
        assert_eq!(exit, 1, "{statement}: {stdout}");
        assert_eq!(stdout, "", "{statement}: nothing is printed");
        assert!(
            stderr.contains(
                "error[Z0803]: `y` has values too large to compute with (above 1e150 in size)"
            ),
            "{statement}: {stderr}"
        );
        assert!(
            stderr.contains("divide the variable by a power of ten first, `add(y_m = y / 1e6)`"),
            "{statement}: {stderr}"
        );
    }
}

#[test]
fn at_1e150_the_tests_give_what_the_data_give_at_their_ordinary_size() {
    // Each statistic on the large values against the same on the small ones; the units of the
    // difference change and nothing else.
    let script = [
        "abs(at_limit.t_test(y, mu: 3e149).statistic() - small.t_test(y, mu: 0.3).statistic()) < 1e-9",
        "abs(at_limit.t_test(y, group: h).statistic() - small.t_test(y, group: h).statistic()) < 1e-9",
        "abs(at_limit.anova(y, group: g, equal_var: true).statistic() - small.anova(y, group: g, equal_var: true).statistic()) < 1e-9",
        "abs(at_limit.anova(y, group: g).statistic() - small.anova(y, group: g).statistic()) < 1e-9",
        "abs(at_limit.anova(y, group: g, equal_var: true).effect_size() - small.anova(y, group: g, equal_var: true).effect_size()) < 1e-12",
        "small.t_test(y, mu: 0.3).statistic()",
        "small.t_test(y, group: h).statistic()",
        "small.anova(y, group: g, equal_var: true).statistic()",
        "small.anova(y, group: g).statistic()",
    ]
    .join("\n");
    let (exit, stdout, stderr) = go(&format!("{DATA}{script}\n"));
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(
        stdout,
        "true\ntrue\ntrue\ntrue\ntrue\n2.1693\n0.225494\n0.736264\n1.46824\n"
    );
    // Pearson's coefficient at the limit is the coefficient of the small values.
    let (exit, stdout, stderr) = go(&format!(
        "{DATA}at_limit.correlation([y, z])\nsmall.correlation([y, z])\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout.matches("0.330").count(), 4, "{stdout}");
}

#[test]
fn the_tests_that_read_ranks_or_counts_take_any_size() {
    // The rank tests and the rank correlations read the order of the values alone; Shapiro
    // and Wilk's W is computed on the values scaled.
    let script = [
        "beyond.wilcoxon_test(y, group: h).p_values() == small.wilcoxon_test(y, group: h).p_values()",
        "beyond.kruskal_test(y, group: g).p_values() == small.kruskal_test(y, group: g).p_values()",
        "abs(beyond.shapiro_test(y).statistic() - small.shapiro_test(y).statistic()) < 1e-12",
    ]
    .join("\n");
    let (exit, stdout, stderr) = go(&format!("{DATA}{script}\n"));
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "true\ntrue\ntrue\n");
    let (exit, stdout, stderr) = go(&format!(
        "{DATA}beyond.correlation([y, z], method: spearman)\nsmall.correlation([y, z], method: spearman)\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    let tables: Vec<&str> = stdout.split("correlation · ").collect();
    assert_eq!(tables.len(), 3, "{stdout}");
    assert_eq!(
        tables[1].lines().skip(1).collect::<Vec<_>>(),
        tables[2].lines().skip(1).collect::<Vec<_>>()
    );
}

#[test]
fn the_message_of_a_variable_too_large() {
    let (exit, _, stderr) = go(&format!(
        "{DATA}beyond.anova(y, group: g, equal_var: true)\n"
    ));
    assert_eq!(exit, 1);
    insta::assert_snapshot!("oversized_message", stderr);
}
