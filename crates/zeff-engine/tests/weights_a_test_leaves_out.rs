//! A test and a weight that is missing or zero (release 0.9.0, step 40b; a fault met while the
//! help pages of `stats` were revised).
//!
//! The Language Reference, section 4.1 ("Weights"), has it that a row with `NA` weight is
//! dropped and counted, and that a zero weight leaves its row out with a note giving how many.
//! `reg` and the reducers did both; the tests of `stats` left such rows out and said nothing,
//! so the count of their note was short: on the shipped wages of 2019,
//! `t_test(wage, group: female, weights: hours, weight_type: analytic)` left 29 rows out and
//! its note counted 26.
//!
//! Each test that takes `weights:` now counts the rows with `NA` in its weight with the others,
//! naming the weight among the variables, and says how many rows a zero weight left out. No
//! figure changes: a test under such weights prints what it prints on the frame with those rows
//! taken out by `where`.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// Twelve rows. `y` is missing in row 2; the weight `w` is missing in rows 5 and 9 and zero in
/// row 3; `v` is a weight with nothing missing and no zero.
const FRAME: &str = "let d = frame(y = [1.0, NA, 3.5, 4, 5.5, 6, 7.5, 8, 9, 10.5, 2, 6.5], z = [2.0, 1, 3, 5, 4, 7, 6.5, 9, 8, 12, 1.5, 6], g = [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2], a = [\"p\", \"q\", \"p\", \"q\", \"p\", \"q\", \"p\", \"p\", \"q\", \"q\", \"q\", \"p\"], b = [1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0], w = [1, 2, 0, 1, NA, 2, 1, 3, NA, 1, 2, 1], v = [1, 2, 1, 1, 3, 2, 1, 3, 2, 1, 2, 1])\n";

/// The same frame with the rows taken out that the weight `w` leaves out.
const KEPT: &str = "let k = d.where(!is_missing(w) && w > 0)\n";

/// Every call that takes `weights:`, with the variables its note names before the weight, and
/// how many of the twelve rows have `NA` in them (row 2, where it reads `y`).
const CALLS: [(&str, &str, usize); 11] = [
    ("t_test(y, mu: 4, WEIGHTS)", "y", 1),
    ("t_test(y, group: g, WEIGHTS)", "y, g", 1),
    ("t_test(y, z, paired: true, WEIGHTS)", "y, z", 1),
    ("wilcoxon_test(y, mu: 4, exact: false, WEIGHTS)", "y", 1),
    (
        "wilcoxon_test(y, group: g, exact: false, WEIGHTS)",
        "y, g",
        1,
    ),
    (
        "wilcoxon_test(y, z, paired: true, exact: false, WEIGHTS)",
        "y, z",
        1,
    ),
    ("anova(y, group: g, WEIGHTS)", "y, g", 1),
    ("kruskal_test(y, group: a, WEIGHTS)", "y, a", 1),
    ("chi2_test(a, g, WEIGHTS)", "a, g", 0),
    ("fisher_test(a, g, WEIGHTS)", "a, g", 0),
    ("proportion_test(b, group: g, WEIGHTS)", "b, g", 0),
];

fn listed(names: &str, weight: Option<&str>) -> String {
    let mut all: Vec<&str> = names.split(", ").collect();
    if let Some(w) = weight {
        all.push(w);
    }
    match all.len() {
        1 => all[0].to_string(),
        n => format!("{} or {}", all[..n - 1].join(", "), all[n - 1]),
    }
}

fn rows(n: usize) -> String {
    if n == 1 {
        "1 row".to_string()
    } else {
        format!("{n} rows")
    }
}

/// The lines of an output that are not its notes on rows left out.
fn figures(stdout: &str, stderr: &str) -> String {
    stdout
        .lines()
        .chain(stderr.lines())
        .filter(|line| !line.starts_with("note: "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_missing_weight_is_counted_and_a_zero_weight_is_said() {
    for (call, names, na) in CALLS {
        let weighted = call.replace("WEIGHTS", "weights: w, weight_type: frequency");
        let (code, stdout, stderr) = go(&format!("{FRAME}d.{weighted}\n"));
        assert_eq!(code, 0, "{weighted}: {stderr}");
        let all = format!("{stdout}{stderr}");
        // Rows 5 and 9 have no weight: they are counted with row 2, and the weight is named.
        let note = format!(
            "note: {} with NA in {} dropped; 1 row with zero weight left out",
            rows(na + 2),
            listed(names, Some("w"))
        );
        assert!(all.contains(&note), "{weighted}: wanted `{note}` in\n{all}");
    }
}

#[test]
fn the_figures_are_those_of_the_rows_that_count() {
    // What a test prints under the weights is what it prints on the frame without the rows the
    // weights leave out, the notes aside: the mending changed what is said, not what is found.
    for (call, _, _) in CALLS {
        let weighted = call.replace("WEIGHTS", "weights: w, weight_type: frequency");
        // Both scripts have the test on the same line, which a warning names.
        let (code_a, out_a, err_a) = go(&format!("{FRAME}{KEPT}d.{weighted}\n"));
        let (code_b, out_b, err_b) = go(&format!("{FRAME}{KEPT}k.{weighted}\n"));
        assert_eq!((code_a, code_b), (0, 0), "{weighted}: {err_a}{err_b}");
        assert_eq!(
            figures(&out_a, &err_a),
            figures(&out_b, &err_b),
            "{weighted}"
        );
    }
}

#[test]
fn a_weight_with_nothing_missing_is_not_named() {
    // With every weight present and above zero the note is the one of the unweighted test:
    // the rows with `NA` in a variable tested, and the weight not among the names.
    for (call, names, na) in CALLS {
        let weighted = call.replace("WEIGHTS", "weights: v, weight_type: frequency");
        let (code, stdout, stderr) = go(&format!("{FRAME}d.{weighted}\n"));
        assert_eq!(code, 0, "{weighted}: {stderr}");
        let all = format!("{stdout}{stderr}");
        assert!(!all.contains("zero weight"), "{weighted}:\n{all}");
        if na == 0 {
            assert!(!all.contains("note: "), "{weighted}:\n{all}");
        } else {
            let note = format!(
                "note: {} with NA in {} dropped (",
                rows(na),
                listed(names, None)
            );
            assert!(all.contains(&note), "{weighted}: wanted `{note}` in\n{all}");
        }
    }
}

#[test]
fn analytic_weights_are_counted_in_the_same_way() {
    // The tests of means take analytic weights too; the case of the shipped wages that showed
    // the fault: 26 wages and 3 hours are missing in 2019, on different rows.
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).where(year == 2019)\nw.t_test(wage, group: female, weights: hours, weight_type: analytic)\nw.anova(wage, group: female, weights: hours, weight_type: analytic)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let all = format!("{stdout}{stderr}");
    assert_eq!(
        all.matches("note: 29 rows with NA in wage, female or hours dropped (")
            .count(),
        2,
        "{all}"
    );
    // 1,518 rows less the 29: the groups' counts as before the mending.
    assert!(stdout.contains("  male     774"), "{stdout}");
    assert!(stdout.contains("  female   715"), "{stdout}");
}

#[test]
fn a_zero_weight_alone_has_its_own_note() {
    let (code, stdout, stderr) = go(
        "let d = frame(y = [1.0, 2, 4, 3, 5, 9, 7, 8], g = [1, 1, 1, 1, 2, 2, 2, 2], n = [2, 0, 1, 3, 1, 2, 0, 0])\nd.t_test(y, group: g, weights: n, weight_type: frequency)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let all = format!("{stdout}{stderr}");
    assert!(
        all.contains("note: 3 rows with zero weight left out ("),
        "{all}"
    );
    assert!(!all.contains("with NA in"), "{all}");
}
