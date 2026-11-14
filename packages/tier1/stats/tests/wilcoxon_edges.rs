//! Corners of the rank-sum form of `wilcoxon_test` that nothing held until the weekly mutation
//! job of release 0.7.0 showed them (step 40a; Testing Handbook, section 1; Language Reference,
//! section 6): the size at which the exact distribution gives way to the approximation, the
//! intervals where ranks are tied, an outcome that is a Bool, and the number of rows of the
//! result. Each figure is that of R 4.6.0's `wilcox.test(..., conf.int = TRUE)`.

use zeff_testkit::run;

/// A frame of the values `v` of two groups `a` and `b`, as a script writes it.
fn two_groups(a: &[f64], b: &[f64]) -> String {
    let values: Vec<String> = a.iter().chain(b).map(|v| format!("{v:?}")).collect();
    let groups: Vec<&str> = a
        .iter()
        .map(|_| "\"a\"")
        .chain(b.iter().map(|_| "\"b\""))
        .collect();
    format!(
        "frame(v = [{}], g = [{}])",
        values.join(", "),
        groups.join(", ")
    )
}

/// What a script that runs prints.
fn printed(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    outcome.output.stdout.clone()
}

/// Fifty values without ties, `round(((i * 37) %% 101) + i * 0.013, 3)`, and ten,
/// `round(((i * 53) %% 97) + 20 + i * 0.007, 3)`.
fn fifty_and_ten() -> (Vec<f64>, Vec<f64>) {
    let round3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let x = (1..=50)
        .map(|i| round3(f64::from((i * 37) % 101) + f64::from(i) * 0.013))
        .collect();
    let y = (1..=10)
        .map(|i| round3(f64::from((i * 53) % 97) + 20.0 + f64::from(i) * 0.007))
        .collect();
    (x, y)
}

/// The exact distribution is the default below fifty values in each group; at fifty in either
/// group the approximation takes over, as in R: W = 167, p-value = 0.1017518, interval
/// −39.747 to 3.069; with the groups exchanged, W = 333 and the interval −3.069 to 39.747.
#[test]
fn fifty_values_in_either_group_take_the_approximation() {
    let (x, y) = fifty_and_ten();
    let first = printed(&format!(
        "{}.wilcoxon_test(v, group: g)\n",
        two_groups(&x, &y)
    ));
    assert!(first.contains("W = 167.00   p = 0.102\n"), "{first}");
    assert!(first.contains("95% CI [-39.747, 3.069]\n"), "{first}");
    assert!(
        first.contains("normal approximation with continuity correction\n"),
        "{first}"
    );
    let second = printed(&format!(
        "{}.wilcoxon_test(v, group: g)\n",
        two_groups(&y, &x)
    ));
    assert!(second.contains("W = 333.00   p = 0.102\n"), "{second}");
    assert!(second.contains("95% CI [-3.069, 39.747]\n"), "{second}");
    assert!(
        second.contains("normal approximation with continuity correction\n"),
        "{second}"
    );
}

/// Tied ranks: the exact test over them gives W = 13.5, p-value = 0.03373097 and the interval
/// −4 to 0; the approximation with its correction for ties gives p-value = 0.03175896 and the
/// same interval to R's solver tolerance (−3.99995 to −0.00003).
#[test]
fn tied_ranks_have_their_intervals_by_both_methods() {
    let (x, y) = (
        [1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 5.0],
        [2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 6.0, 7.0, 7.0],
    );
    let exact = printed(&format!(
        "{}.wilcoxon_test(v, group: g)\n",
        two_groups(&x, &y)
    ));
    assert!(exact.contains("W = 13.50   p = 0.034\n"), "{exact}");
    assert!(exact.contains("95% CI [-4.000, 0.000]\n"), "{exact}");
    assert!(
        exact.contains("exact p-value, over the tied ranks\n"),
        "{exact}"
    );
    let approximate = printed(&format!(
        "{}.wilcoxon_test(v, group: g, exact: false)\n",
        two_groups(&x, &y)
    ));
    assert!(
        approximate.contains("W = 13.50   p = 0.032\n"),
        "{approximate}"
    );
    assert!(
        approximate.contains("95% CI [-4.000, 0.000]\n"),
        "{approximate}"
    );
}

/// Values to one decimal, where the differences lie close together and an interval's end
/// moves with the smallest change in a tail: ten values against twelve, and twelve against
/// eight. R gives, for the first pair, −1.4 to −0.4 by the exact test over the tied ranks,
/// −1.5 to −0.4 by the approximation, and −1.1 upward at 80 per cent for `alternative =
/// "greater"`; for the second pair by the approximation, −0.5 to −0.1.
#[test]
fn the_ends_of_an_interval_over_tied_ranks_are_those_of_r() {
    let (x, y) = (
        [0.3, 0.4, 1.4, 1.1, 0.2, 1.2, 0.2, 0.9, 0.2, 0.0],
        [1.8, 0.8, 0.8, 2.0, 1.8, 0.9, 1.9, 1.3, 1.5, 0.8, 1.9, 1.5],
    );
    let frame = two_groups(&x, &y);
    let exact = printed(&format!("{frame}.wilcoxon_test(v, group: g)\n"));
    assert!(exact.contains("95% CI [-1.400, -0.400]\n"), "{exact}");
    assert!(exact.contains("W = 16.50   p = 0.003\n"), "{exact}");
    let approximate = printed(&format!(
        "{frame}.wilcoxon_test(v, group: g, exact: false)\n"
    ));
    assert!(
        approximate.contains("95% CI [-1.500, -0.400]\n"),
        "{approximate}"
    );
    assert!(
        approximate.contains("W = 16.50   p = 0.004\n"),
        "{approximate}"
    );
    let greater = printed(&format!(
        "{frame}.wilcoxon_test(v, group: g, alternative: greater, level: 0.8)\n"
    ));
    assert!(greater.contains("80% CI [-1.100, Inf)\n"), "{greater}");
    let (x, y) = (
        [0.7, 0.7, 0.2, 0.8, 0.6, 0.7, 0.4, 0.6, 0.8, 0.6, 0.7, 0.0],
        [0.7, 0.9, 1.2, 0.7, 1.2, 1.1, 0.5, 1.0],
    );
    let second = printed(&format!(
        "{}.wilcoxon_test(v, group: g, exact: false)\n",
        two_groups(&x, &y)
    ));
    assert!(second.contains("95% CI [-0.500, -0.100]\n"), "{second}");
    assert!(second.contains("W = 17.00   p = 0.017\n"), "{second}");
}

/// A Bool outcome is counted as 0 and 1 (`wilcox.test(c(1, 0, 1), c(1, 0, 0, 1))`: W = 7,
/// p-value = 1), and the rows of the result are those of both groups.
#[test]
fn a_bool_outcome_is_zero_and_one_and_the_rows_are_those_of_both_groups() {
    let frame = "frame(v = [true, false, true, true, false, false, true], g = [\"a\", \"a\", \"a\", \"b\", \"b\", \"b\", \"b\"])";
    let out = printed(&format!(
        "let t = {frame}.wilcoxon_test(v, group: g)\nprint(t)\nprint(t.n_obs())\n"
    ));
    assert!(out.contains("W = 7.00   p = 1.000\n"), "{out}");
    assert!(out.ends_with("\n7\n"), "{out}");
}

/// The signed-rank form with a difference of zero and with tied ranks, where the statistic
/// lies on either side of its centre. `wilcox.test(x, conf.int = TRUE)` in R 4.6.0: V = 25,
/// p-value = 0.34375, interval −2 to 5; V = 9.5, p-value = 0.1289063, interval −4 to 1; V =
/// 10, p-value = 0.1796875, interval −5.5 to 1.5.
#[test]
fn the_signed_rank_test_with_a_zero_and_with_ties_on_either_side_of_its_centre() {
    for (values, v, p, ci, method) in [
        (
            "0.0, 1.0, -2.0, 3.0, 4.0, 5.0, -6.0, 7.0",
            "25.00",
            "0.344",
            "[-2.000, 5.000]",
            "exact p-value, 1 difference of 0 set aside",
        ),
        (
            "-1.0, -2.0, -2.0, -3.0, -4.0, -4.0, 5.0, -6.0, 1.0",
            "9.50",
            "0.129",
            "[-4.000, 1.000]",
            "exact p-value, over the tied ranks",
        ),
        (
            "0.0, -1.0, 2.0, -3.0, -4.0, -5.0, 6.0, -7.0, -8.0",
            "10.00",
            "0.180",
            "[-5.500, 1.500]",
            "exact p-value, 1 difference of 0 set aside",
        ),
    ] {
        let out = printed(&format!("frame(x = [{values}]).wilcoxon_test(x, mu: 0)\n"));
        assert!(out.contains(&format!("V = {v}   p = {p}\n")), "{out}");
        assert!(out.contains(&format!("95% CI {ci}\n")), "{out}");
        assert!(out.contains(&format!("{method}\n")), "{out}");
    }
}

/// The interval of the signed-rank form where its ends are hardest to hold: a level that a tail
/// of the exact distribution meets exactly, tied ranks and a difference of zero in very small
/// samples (an end that is not reached is infinite), and the approximation where the level
/// asked for cannot be reached and R doubles the error rate until it can. Thirteen cases,
/// each as `wilcox.test(x, conf.int = TRUE, ...)` gives it in R 4.6.0; under the
/// approximation R's ends are those of its solver, within 1e-4 of the ones here.
#[test]
fn the_signed_rank_interval_at_exact_levels_in_small_samples_and_past_the_level() {
    for (values, options, interval, statistic) in [
        (
            "-2.5, 2.2, -1.5, 1.1",
            ", level: 0.875, alternative: greater",
            "87.5% CI [-2.000, Inf)",
            "V = 4.00   p = 0.688",
        ),
        (
            "-2.5, 2.0, 1.5, 0.5, -2.5, 1.0",
            ", level: 0.9",
            "90% CI [-1.000, 1.500]",
            "V = 10.00   p = 0.938",
        ),
        (
            "-2.5, 2.5, 1.0, 1.0, 1.5, -1.0, 2.0, 2.0, -2.5, 3.0, 0.5, 1.0, 2.0, 1.5",
            ", exact: false, level: 0.99",
            "99% CI [-0.750, 2.000]",
            "V = 77.50   p = 0.122",
        ),
        (
            "-2.5, 2.0, 1.5, 0.5, -2.5, 1.0",
            "",
            "95% CI [-2.500, 2.000]",
            "V = 10.00   p = 0.938",
        ),
        (
            "-2.5, 2.0, 1.5, 0.5, -2.5, 1.0",
            ", level: 0.9375",
            "93.75% CI [-2.500, 1.750]",
            "V = 10.00   p = 0.938",
        ),
        (
            "-2.5, 2.0, 1.5, 0.5, -2.5, 1.0",
            ", exact: false",
            "95% CI [-1.000, 1.500]",
            "V = 10.00   p = 1.000",
        ),
        (
            "1.0, 2.5, 1.0, 3.0, -0.5, 3.0, 3.0, 1.0",
            ", exact: false",
            "95% CI [1.000, 3.000]",
            "V = 35.00   p = 0.020",
        ),
        (
            "1.0, 1.0, 2.0",
            "",
            "95% CI [1.000, Inf)",
            "V = 6.00   p = 0.250",
        ),
        (
            "1.0, -1.0, 2.0",
            ", level: 0.5",
            "50% CI [0.500, 1.500]",
            "V = 4.50   p = 0.750",
        ),
        (
            "0.0, 1.0, 2.0",
            "",
            "95% CI [0.000, Inf)",
            "V = 5.00   p = 0.500",
        ),
        (
            "1.5, 2.5",
            ", exact: false, level: 0.5",
            "50% CI [2.000, 2.000]",
            "V = 3.00   p = 0.371",
        ),
        (
            "1.5, 2.5",
            ", exact: false, level: 0.5, alternative: greater",
            "50% CI [1.500, Inf)",
            "V = 3.00   p = 0.186",
        ),
        (
            "1.5, 2.5",
            ", exact: false, level: 0.75, alternative: greater",
            "75% CI [1.500, Inf)",
            "V = 3.00   p = 0.186",
        ),
    ] {
        let out = printed(&format!(
            "frame(x = [{values}]).wilcoxon_test(x, mu: 0{options})\n"
        ));
        assert!(
            out.contains(&format!("{interval}\n")),
            "{values}{options}: {out}"
        );
        assert!(
            out.contains(&format!("{statistic}\n")),
            "{values}{options}: {out}"
        );
    }
}
