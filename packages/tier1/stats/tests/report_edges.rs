//! A report of tests with failed results among them (release 0.7.0, step 40a: the weekly
//! mutation job showed that no test held the count of the failed, its wording, or the title of
//! such a report; Language Reference, sections 5.4 and 6, `report`).

use zeff_testkit::run;

const DATA: &str = "let d = frame(x = [1.0, 3.0, 2.0, 5.0, 4.0, 7.0], y = [2.0, 1.0, 4.0, 3.0, 6.0, 4.0])\nlet t1 = d.t_test(x, mu: 0)\nlet t2 = d.t_test(y, mu: 3)\nlet bad = attempt(d.where(x > 100).t_test(x, mu: 0))\nlet worse = attempt(d.where(x > 200).t_test(y, mu: 0))\n";

#[test]
fn a_report_of_tests_counts_its_failed_results_and_keeps_its_title() {
    let outcome = run(&format!(
        "{DATA}report([t1, bad, t2], title: \"Two means\")\nreport([t1, bad, worse])\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let (out, notes) = (outcome.output.stdout.clone(), outcome.output.stderr.clone());
    // The title given, and the title a report has when none is given.
    assert!(out.starts_with("Two means\n"), "{out}");
    assert!(out.contains("\nreport\n"), "{out}");
    // Two columns where one of three failed, one where two did.
    assert!(
        out.contains("  estimate         3.667      0.333\n"),
        "{out}"
    );
    assert!(out.contains("  estimate         3.667\n"), "{out}");
    assert!(
        notes.contains(
            "warning[W0801]: 1 of 3 results failed and was left out of the table (line 6)\n"
        ),
        "{notes}"
    );
    assert!(
        notes.contains(
            "warning[W0801]: 2 of 3 results failed and were left out of the table (line 7)\n"
        ),
        "{notes}"
    );
}

#[test]
fn a_report_of_tests_that_all_succeeded_says_nothing_of_failures() {
    let outcome = run(&format!("{DATA}report([t1, t2])\n"));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        !outcome.output.stderr.contains("W0801"),
        "{}",
        outcome.output.stderr
    );
    assert!(
        outcome
            .output
            .stdout
            .contains("  estimate         3.667      0.333\n"),
        "{}",
        outcome.output.stdout
    );
}
