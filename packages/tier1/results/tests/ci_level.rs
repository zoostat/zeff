//! `ci()` at the fit's level (release 0.6.0, step 16; open decision #67): on a fit made at
//! `level: 0.9`, `ci()` equals `ci(level: 0.9)` bit for bit and agrees with the summary's
//! interval columns at four decimals, while `level:` written out overrides the fit's.

use zeff_testkit::run;

#[test]
fn ci_defaults_to_the_fits_level_and_level_overrides_it() {
    let script = "let m90 = dataset(wages).reg(log(wage) ~ educ + exper + female, level: 0.9)\nlet a = m90.ci(educ)\nlet b = m90.ci(educ, level: 0.9)\na\nb\nequal(a, b)\nequal(m90.ci(educ, level: 0.95), dataset(wages).reg(log(wage) ~ educ + exper + female).ci(educ))\nm90.ci()\nm90.summary()\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let stdout = outcome.output.stdout;
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], lines[2], "{stdout}");
    assert_eq!(lines[1], lines[3], "{stdout}");
    assert_eq!(lines[4], "true", "{stdout}");
    assert_eq!(lines[5], "true", "{stdout}");
    // The table of `ci()` is titled with the fit's level and its educ row agrees with the
    // summary's interval at four decimals.
    assert!(lines.contains(&"ci · 90%"), "{stdout}");
    let ci_educ = lines
        .iter()
        .find(|l| l.trim_start().starts_with("educ ") && l.split_whitespace().count() == 3)
        .expect("the educ row of ci()");
    let parts: Vec<&str> = ci_educ.split_whitespace().collect();
    let summary_educ = lines
        .iter()
        .find(|l| l.trim_start().starts_with("educ ") && l.contains('['))
        .expect("the educ row of the summary");
    let interval = summary_educ
        .split('[')
        .nth(1)
        .unwrap()
        .trim_end_matches(']')
        .replace(' ', "");
    assert_eq!(interval, format!("{},{}", parts[1], parts[2]), "{stdout}");
    assert!(stdout.contains("90% CI"), "{stdout}");
}
