//! `reliability` (release 0.10.0, step 10; catalogue row 315; question 6 of the release; open
//! decision #241, S1 and S2): the reference (`tests/reference/r/reliability.json`, R 4.6.0: the
//! formulas in base R and `psych::alpha` beside them, on the shipped survey and on small scales
//! typed into the script) through the package's kernels within a relative 1e-10 and through
//! scripts; the edge cases (two items, an item that does not vary, items that run against one
//! another, rows with missing items); the properties (the items in another order, an item
//! reversed twice, a change of an item's units under `standardize:`); and snapshots of every
//! form and every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_measure::reliability::{
    Scale, alpha_of, average_correlation, correlations, covariances, scale,
};
use zeff_testkit::{root, run};

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/reliability.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(v: &Json) -> f64 {
    match v {
        Json::String(s) if s == "NA" => f64::NAN,
        Json::String(s) => s.parse().unwrap(),
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other:?}"),
    }
}

fn numbers(v: &Json) -> Vec<f64> {
    v.as_array().unwrap().iter().map(number).collect()
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got.is_nan() && want.is_nan()) || (got - want).abs() <= relative * want.abs().max(1e-300)
}

fn all_close(got: &[f64], want: &[f64], relative: f64) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(g, w)| close(*g, *w, relative))
}

/// The scales typed into the reference script, by the name of their case; the survey's cases
/// are read from the shipped file.
fn typed(name: &str) -> Option<Vec<(&'static str, Vec<f64>)>> {
    let five = vec![
        (
            "a",
            vec![4., 2., 5., 3., 4., 1., 5., 2., 3., 4., 2., 5., 3., 1., 4.],
        ),
        (
            "b",
            vec![5., 2., 4., 3., 4., 2., 5., 1., 3., 5., 2., 4., 2., 1., 3.],
        ),
        (
            "c",
            vec![4., 3., 5., 2., 5., 1., 4., 2., 4., 4., 1., 5., 3., 2., 4.],
        ),
        (
            "d",
            vec![3., 2., 4., 3., 3., 2., 5., 3., 2., 4., 3., 4., 3., 1., 5.],
        ),
        (
            "e",
            vec![5., 1., 4., 4., 4., 1., 3., 2., 3., 5., 2., 5., 4., 2., 3.],
        ),
    ];
    let against = vec![
        ("u", vec![1., 2., 3., 4., 5., 2., 4., 3., 5., 1.]),
        ("v", vec![5., 4., 3., 2., 1., 4., 3., 3., 1., 4.]),
        ("w", vec![2., 3., 3., 5., 4., 1., 4., 2., 5., 2.]),
    ];
    Some(match name {
        "five_items" => five,
        "two_items" => five.into_iter().take(2).collect(),
        "yes_or_no" => vec![
            (
                "q1",
                vec![
                    1., 0., 1., 1., 0., 1., 0., 1., 1., 0., 1., 1., 0., 0., 1., 1.,
                ],
            ),
            (
                "q2",
                vec![
                    1., 0., 1., 0., 0., 1., 0., 1., 1., 0., 1., 1., 0., 1., 1., 0.,
                ],
            ),
            (
                "q3",
                vec![
                    1., 1., 1., 1., 0., 1., 0., 0., 1., 0., 1., 0., 0., 0., 1., 1.,
                ],
            ),
            (
                "q4",
                vec![
                    0., 0., 1., 1., 0., 1., 1., 1., 1., 0., 0., 1., 0., 0., 1., 1.,
                ],
            ),
        ],
        "against" | "against_reversed" => against,
        "with_gaps" => {
            // Rows 2 and 9 lack `b` and row 13 lacks `e`: twelve rows are left.
            let keep = |values: &[f64]| -> Vec<f64> {
                values
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| ![1, 8, 12].contains(i))
                    .map(|(_, v)| *v)
                    .collect()
            };
            five.iter().map(|(name, v)| (*name, keep(v))).collect()
        }
        _ => return None,
    })
}

/// The six items of the shipped survey over the rows that have them all.
fn survey() -> Vec<(&'static str, Vec<f64>)> {
    let text = std::fs::read_to_string(root().join("packages/tier1/datasets/data/engagement.csv"))
        .unwrap();
    let mut items: Vec<Vec<f64>> = vec![Vec::new(); 6];
    for line in text.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        let answers: Vec<Option<f64>> = cells[1..7].iter().map(|c| c.parse().ok()).collect();
        if answers.iter().all(Option::is_some) {
            for (item, answer) in items.iter_mut().zip(answers) {
                item.push(answer.unwrap());
            }
        }
    }
    ["eng1", "eng2", "eng3", "eng4", "eng5", "eng6"]
        .into_iter()
        .zip(items)
        .collect()
}

fn check(name: &str, got: &Scale, wanted: &Json) {
    assert!(
        close(got.alpha, number(&wanted["alpha"]), 1e-10),
        "{name}: alpha {}",
        got.alpha
    );
    assert!(
        all_close(&got.item_rest, &numbers(&wanted["item_rest"]), 1e-10),
        "{name}: item–rest {:?}",
        got.item_rest
    );
    assert!(
        all_close(
            &got.alpha_if_deleted,
            &numbers(&wanted["alpha_if_deleted"]),
            1e-10
        ),
        "{name}: without an item {:?}",
        got.alpha_if_deleted
    );
    let feldt = numbers(&wanted["feldt"]);
    assert!(
        close(got.feldt.0, feldt[0], 1e-9) && close(got.feldt.1, feldt[1], 1e-9),
        "{name}: Feldt {:?} against {feldt:?}",
        got.feldt
    );
    assert!(
        close(got.std_error, number(&wanted["std_error"]), 1e-10),
        "{name}: se"
    );
    let duhachek = numbers(&wanted["duhachek"]);
    assert!(
        close(got.duhachek.0, duhachek[0], 1e-10) && close(got.duhachek.1, duhachek[1], 1e-10),
        "{name}: Duhachek {:?}",
        got.duhachek
    );
}

#[test]
fn nine_scales_agree_with_the_formulas_and_with_psych() {
    let reference = reference();
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 9);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let reversed: Vec<&str> = case["reversed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let level = number(&case["level"]);
        let items = typed(name).unwrap_or_else(survey);
        let n = items[0].1.len();
        assert_eq!(n as u64, case["n"].as_u64().unwrap(), "{name}");
        let scored: Vec<Vec<f64>> = items
            .iter()
            .map(|(item, values)| {
                let sign = if reversed.contains(item) { -1.0 } else { 1.0 };
                values.iter().map(|v| sign * v).collect()
            })
            .collect();
        let covariance = covariances(&scored);
        let correlation = correlations(&covariance).unwrap();
        let formula = &case["formula"];
        check(name, &scale(&covariance, n, level), &formula["raw"]);
        check(
            name,
            &scale(&correlation, n, level),
            &formula["standardized"],
        );
        assert!(close(
            average_correlation(&correlation),
            number(&formula["average_r"]),
            1e-10
        ));
        // The means and standard deviations of the items as scored.
        let sds: Vec<f64> = (0..scored.len()).map(|i| covariance[i][i].sqrt()).collect();
        assert!(all_close(&sds, &numbers(&formula["sd"]), 1e-10), "{name}");
        // psych's own figures: alpha, the alpha of the standardized items, the average
        // correlation, the standard error, and for each item its correlation with the rest
        // and the alpha without it; Feldt's interval where psych's level, 0.95, was asked.
        let psych = &case["psych"];
        let raw = scale(&covariance, n, level);
        assert!(
            close(raw.alpha, number(&psych["raw_alpha"]), 1e-9),
            "{name}"
        );
        assert!(
            close(alpha_of(&correlation), number(&psych["std_alpha"]), 1e-9),
            "{name}"
        );
        assert!(
            close(
                average_correlation(&correlation),
                number(&psych["average_r"]),
                1e-9
            ),
            "{name}"
        );
        assert!(close(raw.std_error, number(&psych["ase"]), 1e-9), "{name}");
        assert!(
            all_close(&raw.item_rest, &numbers(&psych["r_drop"]), 1e-9),
            "{name}"
        );
        assert!(
            all_close(&raw.alpha_if_deleted, &numbers(&psych["alpha_drop"]), 1e-9),
            "{name}"
        );
        if level == 0.95 {
            let feldt = numbers(&psych["feldt"]);
            assert!(
                close(raw.feldt.0, feldt[0], 1e-8) && close(raw.feldt.1, feldt[1], 1e-8),
                "{name}: {:?} against psych's {feldt:?}",
                raw.feldt
            );
        }
    }
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

/// What a script prints of numbers, read back.
fn printed(script: &str) -> Vec<f64> {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
        .split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect()
}

const FIGURES: &str = "print(format(r.statistic(), digits: 15), format(r.statistic(\"standardized alpha\"), digits: 15), format(r.statistic(\"average correlation\"), digits: 15), r.n_obs())\n";

#[test]
fn the_function_gives_the_reference_on_the_shipped_survey() {
    let reference = reference();
    let cases = reference["cases"].as_array().unwrap();
    for (name, options) in [
        ("engagement_reversed", ", reverse: [eng4]"),
        ("engagement_as_written", ""),
        ("engagement_level_90", ", reverse: [eng4], level: 0.9"),
    ] {
        let case = cases.iter().find(|c| c["name"] == name).unwrap();
        let formula = &case["formula"];
        let got = printed(&format!(
            "let r = dataset(engagement).reliability([eng1..eng6]{options})\n{FIGURES}"
        ));
        assert!(
            close(got[0], number(&formula["raw"]["alpha"]), 1e-12),
            "{name}"
        );
        assert!(
            close(got[1], number(&formula["standardized"]["alpha"]), 1e-12),
            "{name}"
        );
        assert!(
            close(got[2], number(&formula["average_r"]), 1e-12),
            "{name}"
        );
        assert_eq!(got[3], 986.0);
        // Under `standardize: true` the first figure is the alpha of the standardized items.
        let got = printed(&format!(
            "let r = dataset(engagement).reliability([eng1..eng6]{options}, standardize: true)\n{FIGURES}"
        ));
        assert!(
            close(got[0], number(&formula["standardized"]["alpha"]), 1e-12),
            "{name}"
        );
        assert_eq!(got[3], 986.0);
    }
    // The 214 rows without every answer are counted, and the item that runs against the
    // scale is named when it is not reversed, and only then.
    let (_, _, stderr) = go("dataset(engagement).reliability([eng1..eng6])\n");
    assert!(
        stderr.contains("note: 214 rows with NA in eng1, eng2, eng3, eng4, eng5 or eng6 dropped")
    );
    assert!(stderr.contains("warning[W0213]: `eng4` runs against the rest of the scale"));
    let (_, _, stderr) = go("dataset(engagement).reliability([eng1..eng6], reverse: [eng4])\n");
    assert!(!stderr.contains("warning["), "{stderr}");
}

#[test]
fn the_edge_cases() {
    // Two items: alpha is 2r / (1 + r) on the correlations, each item's correlation with the
    // rest is r, and there is no alpha without one of them.
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let b = vec![2.0, 1.0, 4.0, 3.0, 6.0, 5.0];
    let covariance = covariances(&[a.clone(), b.clone()]);
    let correlation = correlations(&covariance).unwrap();
    let r = correlation[0][1];
    let two = scale(&correlation, 6, 0.95);
    assert!(close(two.alpha, 2.0 * r / (1.0 + r), 1e-14));
    assert!(close(two.item_rest[0], r, 1e-14) && close(two.item_rest[1], r, 1e-14));
    assert!(two.alpha_if_deleted.iter().all(|x| x.is_nan()));
    assert!(two.feldt.0 < two.alpha && two.alpha < two.feldt.1);
    // An item that does not vary: no correlations, and so nothing standardized; alpha of the
    // items as scored stands, and the item has no correlation with the rest.
    let flat = vec![3.0; 6];
    let covariance = covariances(&[a.clone(), b.clone(), flat]);
    assert_eq!(correlations(&covariance), None);
    let with_flat = scale(&covariance, 6, 0.95);
    assert!(with_flat.alpha.is_finite());
    assert!(with_flat.item_rest[2].is_nan() && with_flat.item_rest[0].is_finite());
    // The same item twice agrees with itself perfectly: alpha is 1.
    let covariance = covariances(&[a.clone(), a.clone()]);
    assert!(close(alpha_of(&covariance), 1.0, 1e-14));
    // An item and its negative: their sum does not vary, and alpha has no value.
    let negative: Vec<f64> = a.iter().map(|x| -x).collect();
    assert!(alpha_of(&covariances(&[a.clone(), negative])).is_nan());
    // Items that run against one another: alpha below 0, and the interval below it too.
    let down: Vec<f64> = a.iter().map(|x| 7.0 - x + 0.1 * (x * 7.0 % 3.0)).collect();
    let against = scale(&covariances(&[a.clone(), down, b.clone()]), 6, 0.95);
    assert!(against.alpha < 0.0, "{}", against.alpha);
    assert!(against.item_rest.iter().any(|r| *r < 0.0));
    // A wider level gives a wider interval, by either method.
    let covariance = covariances(&[a.clone(), b.clone(), vec![1.0, 3.0, 2.0, 5.0, 4.0, 6.0]]);
    let (narrow, wide) = (scale(&covariance, 6, 0.90), scale(&covariance, 6, 0.99));
    assert!(wide.feldt.0 < narrow.feldt.0 && narrow.feldt.1 < wide.feldt.1);
    assert!(wide.duhachek.0 < narrow.duhachek.0 && narrow.duhachek.1 < wide.duhachek.1);
    assert_eq!(narrow.alpha, wide.alpha);
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 64
}

/// Three to five items of eight to twenty answers from 1 to 5.
fn answers() -> impl Strategy<Value = Vec<Vec<f64>>> {
    (3usize..=5, 8usize..=20).prop_flat_map(|(k, n)| {
        prop::collection::vec(prop::collection::vec((1u8..=5).prop_map(f64::from), n), k)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// The items in another order give the same alpha and each item its own figures; an item
    /// reversed twice is itself; the units of an item change nothing once the items are
    /// standardized, and reversing an item turns its correlation with the rest about when the
    /// others are left.
    #[test]
    fn the_order_of_the_items_and_a_double_reversal_change_nothing(
        items in answers(), turn in 1usize..5,
    ) {
        let covariance = covariances(&items);
        prop_assume!(correlations(&covariance).is_some());
        prop_assume!(covariance.iter().map(|row| row.iter().sum::<f64>()).sum::<f64>() > 1e-9);
        let n = items[0].len();
        let whole = scale(&covariance, n, 0.95);
        let k = items.len();
        let turn = turn % k;
        let mut turned = items.clone();
        turned.rotate_left(turn);
        let other = scale(&covariances(&turned), n, 0.95);
        // Equal to the rounding of sums taken in another order: a relative 1e-8, or 1e-9
        // outright for a figure near 0.
        let same = |a: f64, b: f64| close(a, b, 1e-8) || (a - b).abs() < 1e-9;
        prop_assert!(same(other.alpha, whole.alpha), "{} {}", other.alpha, whole.alpha);
        for i in 0..k {
            let was = (i + turn) % k;
            prop_assert!(same(other.item_rest[i], whole.item_rest[was]));
            prop_assert!(same(other.alpha_if_deleted[i], whole.alpha_if_deleted[was]));
        }
        // Reversed twice.
        let mut twice = items.clone();
        twice[0] = twice[0].iter().map(|x| -(-x)).collect();
        prop_assert_eq!(scale(&covariances(&twice), n, 0.95), whole.clone());
        // The first item in other units: nothing changes among the standardized items.
        let correlation = correlations(&covariance).unwrap();
        let mut scaled = items.clone();
        scaled[0] = scaled[0].iter().map(|x| 100.0 * x + 7.0).collect();
        let rescaled = correlations(&covariances(&scaled)).unwrap();
        prop_assert!(same(alpha_of(&rescaled), alpha_of(&correlation)));
        // What the figures promise: a correlation is between −1 and 1, alpha is at most 1,
        // and the interval holds it.
        prop_assert!(whole.alpha <= 1.0 + 1e-12);
        prop_assert!(whole.item_rest.iter().all(|r| r.is_nan() || r.abs() <= 1.0 + 1e-12));
        prop_assert!(whole.feldt.0 <= whole.alpha && whole.alpha <= whole.feldt.1);
        prop_assert!(whole.duhachek.0 <= whole.alpha && whole.alpha <= whole.duhachek.1);
    }
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let survey = dataset(engagement)\nsurvey.reliability([eng1..eng6], reverse: [eng4])\nsurvey.reliability([eng1..eng6])\nlet r = survey.reliability([eng1..eng6], reverse: [eng4], standardize: true, ci: duhachek, level: 0.9)\nr\nr.estimates()\nr.ci()\nprint(r.statistic(), r.statistic(\"standardized alpha\"), r.n_obs())\nsurvey.reliability([eng1, eng2, eng3], ci: duhachek)\nlet quiz = frame(q1 = [1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1], q2 = [1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0], q3 = [true, true, true, true, false, true, false, false, true, false, true, false, false, false, true, true], q4 = [0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1])\nquiz.reliability([q1, q2, q3, q4])\nquiz.reliability([q1, q2])\nlet odd = frame(u = [1, 2, 3, 4, 5, 2, 4, 3, 5, 1], v = [5, 4, 3, 2, 1, 4, 3, 3, 1, 4], w = [2, 3, 3, 5, 4, 1, 4, 2, 5, 2], same = [3, 3, 3, 3, 3, 3, 3, 3, 3, 3], gap = [1.5, NA, 2.5, 4.0, 4.5, NA, 3.5, 3.0, 5.0, 1.0])\nodd.reliability([u, v, w])\nodd.reliability([u, v, w], reverse: [v])\nodd.reliability([u, w, same])\nodd.reliability([u, w, gap])\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("reliability_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        // The items.
        "dataset(engagement).reliability([eng1])\n",
        "dataset(engagement).reliability([])\n",
        "dataset(engagement).reliability([eng1, eng2, eng1])\n",
        "dataset(engagement).reliability([eng1, eng9])\n",
        "dataset(wages).reliability([wage, industry])\n",
        "dataset(engagement).reliability([eng1, eng2], reverse: [eng4])\n",
        // Too few rows, and scales that cannot be computed.
        "frame(a = [1, NA, 3], b = [NA, 2, 4]).reliability([a, b])\n",
        "frame(a = [1, 2, 3, 4], b = [4, 3, 2, 1]).reliability([a, b])\n",
        "frame(a = [1, 2, 3, 4], b = [2, 1, 4, 3], c = [5, 5, 5, 5]).reliability([a, b, c], standardize: true)\n",
        "frame(a = [1e200, 2, 3, 4], b = [2, 1, 4, 3]).reliability([a, b])\n",
        // The arguments.
        "dataset(engagement).reliability([eng1, eng2], level: 1.5)\n",
        "dataset(engagement).reliability([eng1, eng2], ci: bootstrap)\n",
        "dataset(engagement).reliability([eng1, eng2], standardize: 1)\n",
        "dataset(engagement).reliability()\n",
        "dataset(engagement).reliability([eng1, eng2], omega: true)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("reliability_messages", all);
}
