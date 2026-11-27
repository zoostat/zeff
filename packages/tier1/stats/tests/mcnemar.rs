//! `mcnemar_test` (release 0.10.0, step 9; Language Reference, sections 4, 6 and 10.2; open
//! decisions #88 and #91): the reference (`tests/reference/r/mcnemar.json`, R 4.6.0's
//! `mcnemar.test` with and without its correction, `binom.test` on the pairs that differ, and
//! the difference of the paired proportions from its formula) through the package's own kernels
//! and through scripts, within a relative 1e-12; the edge cases (equal counts, one pair, no
//! pair that differs, an empty cell, all `NA`); the properties (the pairs reordered, the two
//! variables exchanged, the two values exchanged); and snapshots of every form and every
//! message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_stats::mcnemar_test::{chi2, difference, exact};
use zeff_testkit::{root, run};

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/mcnemar.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(v: &Json) -> f64 {
    match v {
        Json::String(s) => s.parse().unwrap(),
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other:?}"),
    }
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(f64::MIN_POSITIVE)
}

/// The four cells of a case, by rows: n11, n12, n21, n22.
fn cells(case: &Json) -> [u64; 4] {
    let t: Vec<u64> = case["table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect();
    [t[0], t[1], t[2], t[3]]
}

#[test]
fn thirteen_tables_agree_with_r() {
    let reference = reference();
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 13);
    let mut balanced = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let [n11, b, c, n22] = cells(case);
        let n = n11 + b + c + n22;
        // The chi-squared statistic without the correction, on one degree of freedom.
        let (statistic, p) = chi2(b, c, false).unwrap();
        let wanted = &case["uncorrected"];
        assert_eq!(wanted["df"].as_u64(), Some(1));
        assert!(
            close(statistic, number(&wanted["statistic"]), 1e-12),
            "{name}: {statistic}"
        );
        assert!(close(p, number(&wanted["p_value"]), 1e-12), "{name}: p {p}");
        // With the correction, on every table: where the two counts are equal R corrects
        // nothing and gives 0, and so does the engine.
        let wanted = &case["corrected"];
        let (statistic, p) = chi2(b, c, true).unwrap();
        assert!(
            close(statistic, number(&wanted["statistic"]), 1e-12),
            "{name}: corrected {statistic}"
        );
        assert!(
            close(p, number(&wanted["p_value"]), 1e-12),
            "{name}: corrected p {p}"
        );
        if b == c {
            assert_eq!((statistic, p), (0.0, 1.0), "{name}");
            balanced += 1;
        }
        // The exact p-value is that of R's binomial test of the two counts.
        let p = exact(b, c);
        assert!(
            close(p, number(&case["exact_p_value"]), 1e-12),
            "{name}: exact p {p}"
        );
        // The difference of the proportions, its standard error and its interval.
        let (d, se, ci) = difference(b, c, n);
        let wanted = &case["difference"];
        assert!(close(d, number(&wanted["estimate"]), 1e-12), "{name}: {d}");
        assert!(
            close(se, number(&wanted["std_error"]), 1e-12),
            "{name}: se {se}"
        );
        let bounds = wanted["ci"].as_array().unwrap();
        assert!(
            close(ci.0, number(&bounds[0]), 1e-12) && close(ci.1, number(&bounds[1]), 1e-12),
            "{name}: {ci:?}"
        );
    }
    // One table of the thirteen has equal counts.
    assert_eq!(balanced, 1);
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

/// A frame of one row for each cell of a paired table, with its count.
fn counted(cells: [u64; 4]) -> String {
    format!(
        "let t = frame(a = [\"no\", \"no\", \"yes\", \"yes\"], b = [\"no\", \"yes\", \"no\", \"yes\"], k = [{}, {}, {}, {}])\n",
        cells[0], cells[1], cells[2], cells[3]
    )
}

/// What a script prints of a number, read back.
fn printed(script: &str) -> Vec<f64> {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
        .split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect()
}

/// A number printed with 17 decimals against the reference: to a relative 1e-12, or to the
/// last decimal printed where the number is smaller than that.
fn printed_close(got: f64, want: f64) -> bool {
    (got - want).abs() <= 1e-12 * want.abs() + 1e-16
}

#[test]
fn the_function_gives_the_reference_through_a_script() {
    let reference = reference();
    for case in reference["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let table = cells(case);
        let n: u64 = table.iter().sum();
        // A cell of no pairs is a row of weight 0, left out with a note; the table keeps its
        // place for it.
        let values = |options: &str| {
            printed(&format!(
                "{}let r = t.mcnemar_test(a, b{options}, weights: k, weight_type: frequency)\nprint(format(r.p_values(), digits: 17))\nprint(r.n_obs())\n",
                counted(table)
            ))
        };
        for (options, p) in [
            (", correct: false", number(&case["uncorrected"]["p_value"])),
            (", method: exact", number(&case["exact_p_value"])),
            (
                ", method: exact, correct: false",
                number(&case["exact_p_value"]),
            ),
        ] {
            let got = values(options);
            assert!(printed_close(got[0], p), "{name}{options}: p {}", got[0]);
            assert_eq!(got[1], n as f64, "{name}");
        }
        let statistic = |options: &str| {
            printed(&format!(
                "{}let r = t.mcnemar_test(a, b{options}, weights: k, weight_type: frequency)\nprint(format(r.statistic(), digits: 17))\nprint(r.df())\n",
                counted(table)
            ))
        };
        let got = statistic(", correct: false");
        assert!(
            close(got[0], number(&case["uncorrected"]["statistic"]), 1e-12),
            "{name}"
        );
        assert_eq!(got[1], 1.0);
        if let Some(corrected) = case.get("corrected") {
            // The default is the corrected statistic, written out or not.
            for options in ["", ", correct: true", ", method: chi2"] {
                let got = values(options);
                assert!(
                    printed_close(got[0], number(&corrected["p_value"])),
                    "{name}{options}"
                );
                let got = statistic(options);
                assert!(
                    close(got[0], number(&corrected["statistic"]), 1e-12),
                    "{name}{options}"
                );
            }
        }
    }
}

#[test]
fn the_shipped_survey_gives_the_reference_from_its_rows() {
    // The rows themselves, one for each respondent, the 23 without a second answer dropped.
    let reference = reference();
    let case = &reference["cases"][0];
    assert_eq!(case["name"], "engagement");
    assert_eq!(cells(case), [465, 183, 114, 415]);
    let got = printed(
        "let r = dataset(engagement).mcnemar_test(recommend_before, recommend_after)\nprint(format(r.statistic(), digits: 17))\nprint(format(r.p_values(), digits: 17))\nprint(r.n_obs())\n",
    );
    assert!(close(
        got[0],
        number(&case["corrected"]["statistic"]),
        1e-12
    ));
    assert!(printed_close(got[1], number(&case["corrected"]["p_value"])));
    assert_eq!(got[2], 1177.0);
}

#[test]
fn the_edge_cases() {
    // Equal counts: no difference to test, a statistic of 0 and a p-value of 1, with the
    // correction or without, as R gives.
    assert_eq!(chi2(7, 7, true), Some((0.0, 1.0)));
    assert_eq!(chi2(7, 7, false), Some((0.0, 1.0)));
    assert_eq!(exact(7, 7), 1.0);
    // The correction is never larger than the difference: counts one apart give 0 too.
    assert_eq!(chi2(4, 5, true), Some((0.0, 1.0)));
    assert!(chi2(4, 5, false).unwrap().0 > 0.0);
    // One pair that differs: the exact p-value is 1, both outcomes being as probable.
    assert_eq!(exact(1, 0), 1.0);
    assert_eq!(exact(0, 1), 1.0);
    // Two, both one way: twice a quarter.
    assert_eq!(exact(2, 0), 0.5);
    // No pair that differs: no statistic, and an exact p-value of 1.
    assert_eq!(chi2(0, 0, true), None);
    assert_eq!(chi2(0, 0, false), None);
    assert_eq!(exact(0, 0), 1.0);
    // The exact p-value is small where every pair differs one way, and is never above 1.
    assert!(close(exact(0, 30), 2.0 * 0.5f64.powi(30), 1e-12));
    assert!(exact(500, 501) <= 1.0);
    // The difference: 0 when the counts are equal, its interval about 0; the whole range when
    // every pair differs one way.
    let (d, se, ci) = difference(7, 7, 54);
    assert_eq!(d, 0.0);
    assert!(se > 0.0 && ci.0 == -ci.1);
    assert_eq!(difference(0, 10, 10), (1.0, 0.0, (1.0, 1.0)));
    assert_eq!(difference(10, 0, 10), (-1.0, 0.0, (-1.0, -1.0)));
    // With no pair that differs the difference is 0 and so is its standard error.
    assert_eq!(difference(0, 0, 25), (0.0, 0.0, (0.0, 0.0)));
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 64
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// The two variables exchanged, or the two values exchanged: the same statistic and the
    /// same p-values, and the difference with its sign and its interval turned about.
    #[test]
    fn exchanging_the_variables_turns_the_difference_about_and_nothing_else(
        n11 in 0u64..40, b in 0u64..40, c in 0u64..40, n22 in 0u64..40, correct in any::<bool>(),
    ) {
        prop_assume!(n11 + b + c + n22 > 0);
        let n = n11 + b + c + n22;
        prop_assert_eq!(chi2(b, c, correct), chi2(c, b, correct));
        prop_assert_eq!(exact(b, c), exact(c, b));
        let (d, se, ci) = difference(b, c, n);
        let (other, other_se, other_ci) = difference(c, b, n);
        prop_assert_eq!(d, -other);
        prop_assert_eq!(se, other_se);
        prop_assert_eq!(ci, (-other_ci.1, -other_ci.0));
        // What the kernels promise of any table.
        prop_assert!((0.0..=1.0).contains(&exact(b, c)));
        prop_assert!(ci.0 <= d && d <= ci.1 && ci.0 >= -1.0 && ci.1 <= 1.0);
        if let Some((statistic, p)) = chi2(b, c, correct) {
            prop_assert!(statistic >= 0.0 && (0.0..=1.0).contains(&p));
            // The correction never makes the statistic larger.
            prop_assert!(chi2(b, c, true).unwrap().0 <= chi2(b, c, false).unwrap().0);
        } else {
            prop_assert_eq!(b + c, 0);
        }
    }

    /// Through a script, on rows: the rows in another order give the same result to the last
    /// character printed, and the two variables exchanged the same p-value.
    #[test]
    fn the_rows_reordered_print_the_same_result(
        pairs in prop::collection::vec((any::<bool>(), any::<bool>()), 4..30),
        turn in 0usize..30,
    ) {
        let (first, second): (Vec<bool>, Vec<bool>) = pairs.iter().copied().unzip();
        let has_both = |v: &[bool]| v.iter().any(|x| *x) && v.iter().any(|x| !*x);
        prop_assume!(has_both(&first) && has_both(&second));
        let list = |v: &[bool]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        let frame = |a: &[bool], b: &[bool]| format!("let t = frame(a = [{}], b = [{}])\n", list(a), list(b));
        let script = |a: &[bool], b: &[bool], rest: &str| format!("{}{rest}", frame(a, b));
        let shown = "t.mcnemar_test(a, b, method: exact)\n";
        let (exit, stdout, stderr) = go(&script(&first, &second, shown));
        prop_assert_eq!(exit, 0, "{}", stderr);
        let turn = turn % pairs.len();
        let mut turned = pairs.clone();
        turned.rotate_left(turn);
        turned.reverse();
        let (a, b): (Vec<bool>, Vec<bool>) = turned.iter().copied().unzip();
        let (_, again, _) = go(&script(&a, &b, shown));
        prop_assert_eq!(&stdout, &again);
        // The variables exchanged: the same p-value, written `b, a`.
        let p = "print(format(t.mcnemar_test(a, b, method: exact).p_values(), digits: 17))\n";
        let q = "print(format(t.mcnemar_test(b, a, method: exact).p_values(), digits: 17))\n";
        prop_assert_eq!(go(&script(&first, &second, p)).1, go(&script(&first, &second, q)).1);
    }
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let survey = dataset(engagement)\nsurvey.mcnemar_test(recommend_before, recommend_after)\nsurvey.mcnemar_test(recommend_before, recommend_after, correct: false)\nsurvey.mcnemar_test(recommend_before, recommend_after, method: exact)\nlet t = frame(first = [\"pos\", \"pos\", \"neg\", \"neg\"], second = [\"pos\", \"neg\", \"pos\", \"neg\"], pairs = [8, 1, 9, 12])\nt.mcnemar_test(first, second, weights: pairs, weight_type: frequency)\nt.mcnemar_test(first, second, method: exact, weights: pairs, weight_type: frequency)\nlet same = frame(a = [true, true, false, false, true, false], b = [true, false, false, true, true, false])\nsame.mcnemar_test(a, b)\nsame.mcnemar_test(a, b, method: exact)\nlet agreed = frame(a = [1, 1, 0, 0, 1], b = [1, 1, 0, 0, 1])\nagreed.mcnemar_test(a, b, method: exact)\nlet gaps = frame(a = [1, 0, NA, 1, 0, 1, 0, 0], b = [0, 0, 1, NA, 1, 1, 1, 1])\nlet r = gaps.mcnemar_test(a, b, correct: false)\nr\nr.estimates()\nr.ci()\nprint(r.h0())\nprint(r.h1())\nprint(r.alternative())\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("mcnemar_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        // No pair that differs under the chi-squared test.
        "frame(a = [1, 1, 0, 0, 1], b = [1, 1, 0, 0, 1]).mcnemar_test(a, b)\n",
        // A variable of more than two values, first and second.
        "dataset(engagement).mcnemar_test(eng1, recommend_after)\n",
        "frame(a = [1, 0, 1, 0], b = [\"x\", \"y\", \"z\", \"x\"]).mcnemar_test(a, b)\n",
        // Two variables of other values than each other's.
        "frame(a = [\"pos\", \"pos\", \"neg\", \"neg\"], b = [1, 0, 1, 0]).mcnemar_test(a, b)\n",
        "frame(a = [true, false, true, false], b = [1, 0, 1, 0]).mcnemar_test(a, b)\n",
        // A variable of one value, and of none: every row `NA`.
        "frame(a = [1, 1, 1], b = [1, 0, 1]).mcnemar_test(a, b)\n",
        "frame(a = [1, 0, NA, NA], b = [NA, NA, 1, 0]).mcnemar_test(a, b)\n",
        // The weights.
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1], w = [1.5, 1, 1, 1]).mcnemar_test(a, b, weights: w, weight_type: frequency)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1], w = [1, 1, 1, 1]).mcnemar_test(a, b, weights: w)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1], w = [1, 1, 1, 1]).mcnemar_test(a, b, weights: w, weight_type: analytic)\n",
        // The arguments.
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1]).mcnemar_test(a)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1]).mcnemar_test(a, c)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1]).mcnemar_test(a, b, method: fisher)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1]).mcnemar_test(a, b, correct: 1)\n",
        "frame(a = [1, 0, 1, 0], b = [1, 0, 0, 1]).mcnemar_test(a, b, alternative: less)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("mcnemar_messages", all);
}
