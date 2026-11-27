//! Section 12 of the specification's sample script, "Reliability of a survey scale", runs
//! whole (release 0.10.0, step 10): `reliability` on the six statements of the survey with the
//! fourth, worded in reverse, named in `reverse:`, and the score of each respondent with that
//! item turned the right way round (open decision #241, which found the section scoring it as
//! written).
//!
//! The section is read from `docs/spec/zeff_samples.zeff` itself, so the test follows the
//! specification's text; only the file it imports is supplied, as the shipped dataset of the
//! same name.

use zeff_testkit::run;

/// The lines of the section, from its heading to the next.
fn section() -> String {
    let script = include_str!("../../../docs/spec/zeff_samples.zeff");
    let from = script
        .find("// # 12. Reliability of a survey scale")
        .expect("the section's heading");
    let to = script.find("// # 13.").expect("the next heading");
    script[from..to].to_string()
}

#[test]
fn the_section_reverses_the_fourth_statement() {
    let text = section();
    assert!(
        text.contains(".reliability([eng1..eng6], reverse: [eng4])"),
        "{text}"
    );
    assert!(text.contains("6 - eng4"), "{text}");
    assert!(!text.contains("row([eng1..eng6]"), "{text}");
}

#[test]
fn the_section_runs_whole_on_the_shipped_survey() {
    let text = section().replace("import(\"data/engagement.csv\")", "dataset(engagement)");
    assert!(text.contains("dataset(engagement)"));
    let outcome = run(&format!(
        "{text}engage.aggregate(n = n(), mean = mean(engagement), sd = sd(engagement))\n"
    ));
    let (stdout, stderr) = (outcome.output.stdout, outcome.output.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    // Nothing worse than a note is said of the scale: with the fourth statement reversed no
    // item runs against the rest.
    assert!(!stderr.contains("error["), "{stderr}");
    assert!(!stderr.contains("W0213"), "{stderr}");
    assert!(
        stdout.contains("reliability · Cronbach's alpha · 6 items"),
        "{stdout}"
    );
    assert!(stdout.contains("N = 986 rows with every item"), "{stdout}");
    assert!(
        stdout.contains("alpha = 0.851   95% CI [0.836, 0.865] (Feldt)"),
        "{stdout}"
    );
    assert!(stdout.contains("eng4 (reversed)"), "{stdout}");
    // The score with the fourth item the right way round spreads as a scale should: its
    // standard deviation is 0.784, where the items as written give 0.552.
    assert!(stdout.contains("0.78"), "{stdout}");
    assert!(!stdout.contains("0.55"), "{stdout}");
}
