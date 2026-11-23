//! Six printed layouts changed (release 0.9.0, step 34; open decision #171), each a row of the
//! decision's table, each with its case here:
//!
//! 1. A coefficient of `report` too small for its decimals prints by significant digits, with
//!    its sign, where it printed `0.000`; its standard error follows it.
//! 2. `labels:` of `report` applies to each part of an interaction.
//! 3. The df column of Welch comparisons has thousands separators.
//! 4. A facet of a numeric variable is titled `kids = 0`.
//! 5. Hypotheses too long for the page have their symbols on a line under the words.
//! 6. `estimates()` of a t-test gives the standard error of the difference (held to R in
//!    `packages/tier1/stats/tests/t_test.rs`).

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const WAGES: &str = "let w = dataset(wages).drop_missing([wage, educ, exper, hours]).add(big = educ * 20000, neg = -educ * 3000, huge = educ * 1e9, lsize = log(hours))\n";

// ----- 1. small coefficients -----

#[test]
fn a_coefficient_too_small_for_its_decimals_prints_by_significant_digits() {
    let (code, stdout, stderr) = go(&format!(
        "{WAGES}report([w.reg(wage ~ big + exper), w.reg(wage ~ neg), w.reg(wage ~ huge)])\nreport([w.reg(wage ~ big + exper)], digits: 5)\nreport([w.reg(wage ~ neg)], stars: false)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("small_coefficients", stdout);
    // Two significant digits with the sign, where three decimals gave `0.000`; the standard
    // error to its coefficient's decimals, or to two digits of its own where those decimals
    // would print it as zeros; below one in a million, an exponent.
    for shown in [
        "0.00010***",
        "-0.00050***",
        "(0.0000020)",
        "(0.00001)",
        "1.5e-9***",
        "(4.0e-11)",
    ] {
        assert!(stdout.contains(shown), "{shown}\n{stdout}");
    }
    assert!(!stdout.contains(" 0.000***"), "{stdout}");
    // What three decimals show is printed as it was.
    assert!(
        stdout.contains("0.596***") && stdout.contains("(0.013)"),
        "{stdout}"
    );
    // And so is its standard error, zeros or not: only a coefficient printed as zeros moves.
    let (code, stdout, stderr) =
        go("let wages = dataset(wages)\nreport([wages.reg(log(wage) ~ educ + exper + female)])\n");
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("0.022***") && stdout.contains("(0.000)"),
        "{stdout}"
    );
    // A coefficient that is exactly zero is `0.000`: there are no digits to show.
    let (code, stdout, stderr) = go(
        "let d = frame(y = [1.0, 2.0, 3.0, 4.0], x = [1.0, 2.0, 3.0, 4.0])\nreport([d.reg(y ~ x)])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("1.000"), "{stdout}");
}

// ----- 2. labels on the parts of an interaction -----

#[test]
fn labels_apply_to_each_part_of_an_interaction() {
    let (code, stdout, stderr) = go(&format!(
        "{WAGES}let m = w.reg(wage ~ lsize + exper + lsize:category(union) + category(female))\nreport([m])\nreport([m], labels: {{lsize: \"Log hours\", union: \"Union\", female: \"Sex\", exper: \"Experience\"}})\nreport([m], labels: {{`lsize:union=yes`: \"Log hours of members\", lsize: \"Log hours\"}})\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("labels_on_parts", stdout);
    for shown in [
        "  Log hours ",
        "  Log hours:Union=yes ",
        "  Sex=female ",
        "  Experience ",
        // The label of a whole term comes before those of its parts.
        "  Log hours of members ",
    ] {
        assert!(stdout.contains(shown), "{shown}\n{stdout}");
    }
    // A name that is no term, no part of one and no variable of a level is still refused.
    let (code, _, stderr) = go(&format!(
        "{WAGES}report([w.reg(wage ~ lsize:category(union))], labels: {{hours: \"Hours\"}})\n"
    ));
    assert_eq!(code, 1);
    insta::assert_snapshot!("label_of_no_term", stderr);
}

// ----- 3. the df column -----

#[test]
fn the_df_column_of_welch_comparisons_has_thousands_separators() {
    let (code, stdout, stderr) = go(&format!(
        "{WAGES}let a = w.where(industry == \"Mining\" || industry == \"Finance\" || industry == \"Retail\").anova(wage, group: industry)\na.posthoc(method: holm)\na.posthoc(method: games_howell)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("df_column", stdout);
    assert!(stdout.contains("   2,722.7   "), "{stdout}");
    assert!(!stdout.contains("2722.7"), "{stdout}");
    // Under a thousand, a number prints as it did.
    assert!(stdout.contains("     238.6   "), "{stdout}");
}

// ----- 5. long hypotheses -----

#[test]
fn hypotheses_too_long_for_the_page_put_their_symbols_under_the_words() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).drop_missing([wage]).add(member = union == 1, woman_in_the_sample = female == 1)\nw.proportion_test(member, group: woman_in_the_sample)\nw.proportion_test(member, group: female)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("long_hypotheses", stdout);
    // No line of the long block runs past the page; the short block is side by side, as it was.
    let block: Vec<&str> = stdout.lines().take(5).collect();
    assert!(
        block.iter().all(|line| line.chars().count() <= 118),
        "{stdout}"
    );
    assert_eq!(
        block[2],
        "      π(woman_in_the_sample=false) − π(woman_in_the_sample=true) = 0"
    );
    assert!(
        stdout.contains(
            "  H0: the population proportions of member are equal in male and female        π(male) − π(female) = 0\n"
        ),
        "{stdout}"
    );
}

// ----- 6. the standard error of a t-test's difference -----

#[test]
fn the_estimates_of_a_t_test_give_the_standard_error_of_the_difference() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).drop_missing([wage])\nlet t = w.t_test(wage, group: female)\nt.estimates()\nlet e = t.estimates()\nprint(abs(e[\"estimate\"][1] / e[\"std_error\"][1] - e[\"statistic\"][1]) < 1e-9)\nw.t_test(wage, mu: 20).estimates()\nw.t_test(wage, group: female, equal_var: true).estimates()\nlet s = frame(before = [5.1, 6.0, 4.4, 7.2, 5.5], after = [5.9, 6.1, 5.2, 7.9, 5.4])\ns.t_test(before, after, paired: true).estimates()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("t_test_estimates", stdout);
    assert!(stdout.contains("\ntrue\n"), "{stdout}");
    // The standard error of each difference, in the third column: Welch's, the one-sample
    // test's, the pooled one and the paired one.
    let errors: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("difference ("))
        .map(|line| {
            let after = line.split(") ").last().unwrap_or_default();
            after.split_whitespace().nth(1).unwrap_or_default()
        })
        .collect();
    assert_eq!(errors, ["0.205332", "0.103972", "0.206177", "0.191311"]);
}
