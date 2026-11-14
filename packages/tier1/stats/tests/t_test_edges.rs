//! Corners of `t_test` and of the arguments the tests share that nothing held until the weekly
//! mutation job of release 0.7.0 showed them (step 40a; Testing Handbook, section 1; Language
//! Reference, sections 6 and 10.2): a mean of exactly two values, alone and paired, the number
//! of rows of a two-group result, a level of 1, sampling weights named in words, and a message
//! that lists more than twenty rows.

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

/// Two values are enough for a mean: `t.test(c(1, 3))` gives t = 2 on 1 degree of freedom,
/// p-value = 0.2951672, and the interval −10.7062 to 14.7062. The paired form on two pairs
/// whose differences are 1 and 3 gives the same (`t.test(c(1, 5), c(0, 2), paired = TRUE)`).
#[test]
fn two_values_are_enough_alone_and_paired() {
    let one = printed("frame(x = [1.0, 3.0]).t_test(x, mu: 0)\n");
    assert!(one.contains("N = 2   mean = 2.000   sd = 1.414\n"), "{one}");
    assert!(one.contains("95% CI [-10.706, 14.706]\n"), "{one}");
    assert!(one.contains("t = 2.00   df = 1   p = 0.295\n"), "{one}");
    let paired = printed("frame(x = [1.0, 5.0], z = [0.0, 2.0]).t_test(x, z, paired: true)\n");
    assert!(paired.contains("N = 2 pairs"), "{paired}");
    assert!(paired.contains("95% CI [-10.706, 14.706]\n"), "{paired}");
    assert!(
        paired.contains("t = 2.00   df = 1   p = 0.295\n"),
        "{paired}"
    );
}

/// One pair, written so or left by the `NA`s, is too few.
#[test]
fn one_pair_is_too_few() {
    for script in [
        "frame(x = [1.0], z = [0.0]).t_test(x, z, paired: true)\n",
        "frame(x = [1.0, 2.0, NA], z = [NA, 2.0, 1.0]).t_test(x, z, paired: true)\n",
    ] {
        let message = refused(script);
        assert!(
            message.contains(
                "error[Z0401]: `t_test` needs at least two values of the pairs of x and z, but there is one\n"
            ),
            "{message}"
        );
    }
}

/// The rows of a two-group result are those of both groups.
#[test]
fn the_rows_of_a_two_group_result_are_those_of_both_groups() {
    let out = printed(
        "let t = frame(x = [1.0, 2.0, 4.0, 3.0, 6.0, 8.0, 9.0], g = [\"a\", \"a\", \"a\", \"b\", \"b\", \"b\", \"b\"]).t_test(x, group: g)\nprint(t.n_obs())\n",
    );
    assert_eq!(out, "7\n");
}

/// A level is strictly below 1.
#[test]
fn a_level_of_one_is_refused() {
    let message = refused("frame(x = [1.0, 3.0, 4.0]).t_test(x, mu: 0, level: 1.0)\n");
    assert!(
        message.contains(
            "error[Z0401]: `level:` of `t_test` is a confidence level between 0 and 1, but this is 1\n"
        ),
        "{message}"
    );
}

/// Sampling weights are refused when `weight_type: sampling` is written, as when it is left
/// to its default.
#[test]
fn sampling_weights_named_in_words_are_refused() {
    for kind in ["", ", weight_type: sampling"] {
        let message = refused(&format!(
            "frame(x = [1.0, 3.0, 4.0], w = [1, 2, 1]).t_test(x, mu: 0, weights: w{kind})\n"
        ));
        assert!(
            message.contains(
                "error[Z0411]: `weights:` of `t_test` are sampling weights, which a test cannot use\n"
            ),
            "{message}"
        );
    }
}

/// A message that names rows lists the first twenty and then says that there are more.
#[test]
fn more_than_twenty_rows_are_cut_with_an_ellipsis() {
    let x: Vec<String> = (0..25).map(|i| format!("{i}.0")).collect();
    let w = vec!["0.5"; 25];
    let message = refused(&format!(
        "frame(x = [{}], w = [{}]).t_test(x, mu: 0, weights: w, weight_type: frequency)\n",
        x.join(", "),
        w.join(", ")
    ));
    assert!(
        message.contains(
            "`w` has 25 fractions in rows 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, …\n"
        ),
        "{message}"
    );
    // Twenty rows or fewer are listed in full.
    let message = refused(
        "frame(x = [1.0, 2.0, 3.0], w = [0.5, 1, 1.5]).t_test(x, mu: 0, weights: w, weight_type: frequency)\n",
    );
    assert!(
        message.contains("`w` has 2 fractions in rows 1, 3\n"),
        "{message}"
    );
}
