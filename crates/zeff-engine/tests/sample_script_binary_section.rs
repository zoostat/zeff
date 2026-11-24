//! Section 10 of the specification's sample script, "Binary outcomes", runs whole (release
//! 0.9.0, step 41): the logit and the probit with clustered errors are fitted with `glm`, which
//! stands where `logistic` and `probit` stood before open decision #112, their table is
//! exported, and the likelihood-ratio and Wald tests are made.
//!
//! The section is read from `docs/spec/zeff_samples.zeff` itself, so the test follows the
//! specification's text; only the frame it works on is supplied, the shipped wages, which the
//! script's first section reads from a file of the reader's own.

use zeff_testkit::run;

/// The lines of the section, from its heading to the next.
fn section() -> String {
    let script = include_str!("../../../docs/spec/zeff_samples.zeff");
    let from = script
        .find("// # 10. Binary outcomes")
        .expect("the section's heading");
    let to = script.find("// # 11.").expect("the next heading");
    script[from..to].to_string()
}

#[test]
fn the_section_is_written_with_glm() {
    let text = section();
    assert!(text.contains(".glm("), "{text}");
    assert!(text.contains("family: binomial"), "{text}");
    assert!(text.contains("link: probit"), "{text}");
    assert!(!text.contains(".logistic("), "{text}");
    assert!(!text.contains(".probit("), "{text}");
}

#[test]
fn the_section_runs_whole_on_the_shipped_wages() {
    let outcome = run(&format!("let wages = dataset(wages)\n{}", section()));
    let (stdout, stderr) = (outcome.output.stdout, outcome.output.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    // Nothing worse than a note is said: the clusters of one observation.
    assert!(!stderr.contains("error["), "{stderr}");
    assert!(!stderr.contains("warning["), "{stderr}");
    // The likelihood-ratio test of the two nested fits, on the same rows, and the Wald test
    // on the clustered errors.
    assert!(stdout.contains("compare · lr1 against lr0"), "{stdout}");
    assert!(stdout.contains("chi2(12) = 972.33   p < 0.001"), "{stdout}");
    assert!(stdout.contains("test · lg · female == 0"), "{stdout}");
    assert!(stdout.contains("chi2(1) = 0.88   p = 0.348"), "{stdout}");
}
