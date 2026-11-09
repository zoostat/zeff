//! `fisher_test` (release 0.7.0, step 13; Language Reference, sections 4, 6 and 10.2; open
//! decision #91): the reference (`tests/reference/r/fisher.json`, R 4.6.0's `fisher.test` on
//! 2 × 2 tables for the three alternatives and on larger tables) through the package's own
//! kernels — p-values within a relative 1e-12, the odds ratio and its bounds within 1e-9 of the
//! same quantities re-solved tightly in R and within R's own solver tolerance of `fisher.test`;
//! the properties (the conditional distribution sums to one and its tails meet; the p-value and
//! the odds ratio are unchanged when rows and columns are swapped or reordered); the edge cases
//! (an empty cell, a zero margin, the table limit); and snapshots of every form and every
//! message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_core::value::Alternative;
use zeff_stats::fisher_test::{Conditional, TABLE_LIMIT, exact_p, exact_p_limited, two_by_two};
use zeff_testkit::{root, run};

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/fisher.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(v: &Json) -> f64 {
    match v {
        Json::String(s) => match s.as_str() {
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            "NA" => f64::NAN,
            other => other.parse().unwrap(),
        },
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other:?}"),
    }
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    if got.is_infinite() || want.is_infinite() {
        return got == want;
    }
    (got - want).abs() <= relative * want.abs().max(f64::MIN_POSITIVE)
}

fn alternative(word: &str) -> Alternative {
    match word {
        "two.sided" => Alternative::TwoSided,
        "less" => Alternative::Less,
        "greater" => Alternative::Greater,
        other => panic!("{other}"),
    }
}

fn cells(case: &Json) -> [[u64; 2]; 2] {
    let t: Vec<u64> = case["table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect();
    [[t[0], t[1]], [t[2], t[3]]]
}

#[test]
fn two_by_two_tables_agree_with_fisher_test() {
    let reference = reference();
    for case in reference["two_by_two"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let got = two_by_two(
            cells(case),
            alternative(case["alternative"].as_str().unwrap()),
        );
        let p = number(&case["p_value"]);
        assert!(
            close(got.p, p, 1e-12),
            "{name}: p {} against R's {p}",
            got.p
        );
        // R solves for the estimate and the bounds with uniroot() at about 1e-4 on the root;
        // the tight values re-solve its formulas to 1e-12, and the engine must meet those.
        let tight = &case["tight"];
        let estimate = number(&tight["estimate"]);
        assert!(
            close(got.estimate, estimate, 1e-9),
            "{name}: odds ratio {} against {estimate}",
            got.estimate
        );
        let bounds = tight["ci"].as_array().unwrap();
        let (lower, upper) = (number(&bounds[0]), number(&bounds[1]));
        assert!(
            close(got.ci.0, lower, 1e-9) && close(got.ci.1, upper, 1e-9),
            "{name}: interval {:?} against ({lower}, {upper})",
            got.ci
        );
        // And R's own printed values, within its solver's tolerance.
        let r_estimate = number(&case["estimate"]);
        let r_bounds = case["ci"].as_array().unwrap();
        assert!(
            close(got.estimate, r_estimate, 1e-3),
            "{name}: odds ratio {} against R's {r_estimate}",
            got.estimate
        );
        assert!(
            close(got.ci.0, number(&r_bounds[0]), 2e-2)
                && close(got.ci.1, number(&r_bounds[1]), 2e-2),
            "{name}: interval {:?} against R's {r_bounds:?}",
            got.ci
        );
    }
}

fn larger_table(case: &Json) -> Vec<Vec<u64>> {
    let r = case["rows"].as_u64().unwrap() as usize;
    let c = case["columns"].as_u64().unwrap() as usize;
    let t: Vec<u64> = case["table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect();
    (0..r).map(|i| t[i * c..(i + 1) * c].to_vec()).collect()
}

#[test]
fn larger_tables_agree_with_fisher_test() {
    let reference = reference();
    for case in reference["larger"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let (p, tables) = exact_p(&larger_table(case)).expect(name);
        let want = number(&case["p_value"]);
        assert!(
            close(p, want, 1e-12),
            "{name}: p {p} over {tables} tables against R's {want}"
        );
    }
    // A 2 × 2 table through the general enumeration gives the two-sided p-value of the direct
    // kernel.
    let (p, tables) = exact_p(&[vec![3, 1], vec![1, 3]]).unwrap();
    assert_eq!(tables, 5);
    assert!(close(
        p,
        two_by_two([[3, 1], [1, 3]], Alternative::TwoSided).p,
        1e-14
    ));
}

#[test]
fn the_edge_cases_of_a_two_by_two_table() {
    // An empty cell at the top of the range: the odds ratio is infinite, the interval open above.
    let r = two_by_two([[5, 0], [3, 4]], Alternative::TwoSided);
    assert_eq!(r.estimate, f64::INFINITY);
    assert_eq!(r.ci.1, f64::INFINITY);
    assert!(r.ci.0 > 0.0 && r.ci.0 < 1.0);
    // At the bottom: zero, with the interval closed at zero.
    let r = two_by_two([[0, 5], [5, 0]], Alternative::TwoSided);
    assert_eq!(r.estimate, 0.0);
    assert_eq!(r.ci.0, 0.0);
    assert!(r.ci.1 > 0.0 && r.ci.1 < 1.0);
    // A balanced table: the odds ratio is 1 and the bounds are reciprocals.
    let r = two_by_two([[4, 4], [4, 4]], Alternative::TwoSided);
    assert!((r.estimate - 1.0).abs() < 1e-12);
    assert!((r.ci.0 * r.ci.1 - 1.0).abs() < 1e-9, "{:?}", r.ci);
    assert!((r.p - 1.0).abs() < 1e-12);
    // One-sided intervals are bounded by 0 and Inf on the side not estimated.
    let less = two_by_two([[3, 1], [1, 3]], Alternative::Less);
    let greater = two_by_two([[3, 1], [1, 3]], Alternative::Greater);
    assert_eq!(less.ci.0, 0.0);
    assert_eq!(greater.ci.1, f64::INFINITY);
    assert!(
        (less.p + greater.p - 1.0).abs() < 1.0,
        "the tails overlap at the observed cell"
    );
    // The conditional distribution sums to one at any odds ratio and its tails meet.
    let c = Conditional::new(7, 9, 6);
    for psi in [0.01, 0.5, 1.0, 3.0, 250.0] {
        let d = c.probabilities(psi);
        assert!((d.iter().sum::<f64>() - 1.0).abs() < 1e-14, "psi {psi}");
        for x in c.lo..=c.hi {
            let both = c.lower_tail(x, psi) + c.upper_tail(x, psi) - d[(x - c.lo) as usize];
            assert!((both - 1.0).abs() < 1e-13, "psi {psi}, x {x}");
        }
    }
}

#[test]
fn the_table_limit_refuses_what_it_cannot_enumerate() {
    // The limit is counted in tables with the given margins: a 3 × 3 table with 16 rows has
    // 1,114 of them, enumerated with any limit at or above that and refused below it.
    let t = vec![vec![2, 4, 0], vec![3, 1, 2], vec![1, 5, 3]];
    let (_, tables) = exact_p_limited(&t, TABLE_LIMIT).unwrap();
    assert_eq!(exact_p_limited(&t, tables).map(|r| r.1), Ok(tables));
    assert_eq!(exact_p_limited(&t, tables - 1), Err(tables - 1));
    // A table of twelve industries by sex over the whole of wages is beyond the limit.
    let wide: Vec<Vec<u64>> = (0..12).map(|i| vec![700 + 30 * i, 650 + 25 * i]).collect();
    assert_eq!(exact_p(&wide), None);
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 64
}

fn table_strategy() -> impl Strategy<Value = Vec<Vec<u64>>> {
    (2usize..=3, 2usize..=3)
        .prop_flat_map(|(r, c)| prop::collection::vec(prop::collection::vec(0u64..=5, c), r))
}

fn transpose(t: &[Vec<u64>]) -> Vec<Vec<u64>> {
    (0..t[0].len())
        .map(|j| t.iter().map(|row| row[j]).collect())
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    #[test]
    fn the_p_value_ignores_the_arrangement_of_rows_and_columns(t in table_strategy()) {
        let has_rows = t.iter().all(|row| row.iter().sum::<u64>() > 0);
        let has_cols = (0..t[0].len()).all(|j| t.iter().map(|row| row[j]).sum::<u64>() > 0);
        prop_assume!(has_rows && has_cols);
        let (p, _) = exact_p(&t).unwrap();
        let (swapped, _) = exact_p(&transpose(&t)).unwrap();
        prop_assert!((p - swapped).abs() <= 1e-12 * p.max(1e-300), "{t:?}: {p} against {swapped}");
        let mut reversed = t.clone();
        reversed.reverse();
        for row in &mut reversed { row.reverse(); }
        let (again, _) = exact_p(&reversed).unwrap();
        prop_assert!((p - again).abs() <= 1e-12 * p.max(1e-300), "{t:?}: {p} against {again}");
        prop_assert!((0.0..=1.0).contains(&p));
    }

    #[test]
    fn a_two_by_two_table_keeps_its_odds_ratio_when_transposed(
        n11 in 0u64..=6, n12 in 0u64..=6, n21 in 0u64..=6, n22 in 0u64..=6
    ) {
        prop_assume!(n11 + n12 > 0 && n21 + n22 > 0 && n11 + n21 > 0 && n12 + n22 > 0);
        let a = two_by_two([[n11, n12], [n21, n22]], Alternative::TwoSided);
        let b = two_by_two([[n11, n21], [n12, n22]], Alternative::TwoSided);
        prop_assert!((a.p - b.p).abs() <= 1e-12 * a.p);
        prop_assert!(a.estimate == b.estimate || (a.estimate - b.estimate).abs() <= 1e-9 * a.estimate);
        prop_assert!(a.ci.0 <= a.estimate && a.estimate <= a.ci.1, "{a:?}");
        // The one-sided p-values are the tails: each at least the point probability.
        let less = two_by_two([[n11, n12], [n21, n22]], Alternative::Less).p;
        let greater = two_by_two([[n11, n12], [n21, n22]], Alternative::Greater).p;
        prop_assert!(less + greater >= 1.0 - 1e-12, "{less} + {greater}");
        // The point sits at or below the two-sided p-value, which is at most 1.
        prop_assert!(a.p <= 1.0 + 1e-12 && a.p >= less.min(greater) - 1e-12);
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

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let wages = dataset(wages)\nwages.fisher_test(union, female)\nwages.fisher_test(union, female, alternative: greater)\nlet t = frame(a = [\"x\", \"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"y\"], b = [1, 1, 1, 2, 1, 2, 2, 2])\nt.fisher_test(a, b)\nt.fisher_test(a, b, alternative: less)\nlet z = frame(a = [\"x\", \"x\", \"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\"], b = [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2])\nz.fisher_test(a, b)\nz.fisher_test(a, b, alternative: greater)\nlet m = frame(r = [\"a\", \"a\", \"a\", \"a\", \"a\", \"a\", \"b\", \"b\", \"b\", \"b\", \"c\", \"c\", \"c\", \"c\", \"c\", \"c\"], c = [1, 1, 2, 2, 2, 2, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2])\nm.fisher_test(r, c)\nlet w = frame(r = [\"a\", \"a\", \"b\", \"b\", \"c\", \"c\"], c = [1, 2, 1, 2, 1, 2], w = [3, 1, 1, 3, 0, 0])\nw.fisher_test(r, c, weights: w, weight_type: frequency)\nlet d = frame(a = [\"x\", \"x\", NA, \"y\", \"y\", \"x\"], b = [1, 2, 1, 1, 2, 1])\nd.fisher_test(a, b)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "dataset(wages).fisher_test(industry, female, alternative: greater)\n",
        "dataset(wages).fisher_test(industry, female)\n",
        "frame(a = [1, 1, 1], b = [1, 2, 1]).fisher_test(a, b)\n",
        "frame(a = [1, 2, 1], b = [1, 1, 1]).fisher_test(a, b)\n",
        "frame(a = [1, 1, 2, 2], b = [1, 2, 1, 2], w = [1, 1, 0, 0]).fisher_test(a, b, weights: w, weight_type: frequency)\n",
        "frame(a = [1, 2, 1, 2], b = [1, 2, 2, 1], w = [1.5, 1, 1, 1]).fisher_test(a, b, weights: w, weight_type: frequency)\n",
        "frame(a = [1, 2, 1, 2], b = [1, 2, 2, 1], w = [1, 1, 1, 1]).fisher_test(a, b, weights: w)\n",
        "frame(a = [1, 2, 1, 2], b = [1, 2, 2, 1], w = [1, 1, 1, 1]).fisher_test(a, b, weights: w, weight_type: analytic)\n",
        "frame(a = [1, 2, 1, 2], b = [1, 2, 2, 1]).fisher_test(a)\n",
        "frame(a = [1, 2, 1, 2], b = [1, 2, 2, 1]).fisher_test(a, c)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
