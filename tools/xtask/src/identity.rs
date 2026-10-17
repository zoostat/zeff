//! The cross-platform identity check (Testing Handbook, section 1): the conformance report of
//! every target must be the same, line for line.
//!
//! A report is a list of `name digest` lines; lines starting with `#` are comments (the raw bits of
//! single results, kept to see where two targets part) and are not compared.

use std::collections::BTreeMap;

/// The cases of one report, by name.
fn parse(label: &str, text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut cases = BTreeMap::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(name), Some(digest), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!(
                "{label}, line {}: expected `name digest`, found `{line}`",
                number + 1
            ));
        };
        if cases.insert(name.to_string(), digest.to_string()).is_some() {
            return Err(format!("{label}: the case `{name}` appears twice"));
        }
    }
    if cases.is_empty() {
        return Err(format!("{label}: the report holds no cases"));
    }
    Ok(cases)
}

/// Compare reports (label and text). `expected_reports` is how many must be present, so that a
/// target that did not run cannot pass unseen. Returns one sentence per difference.
pub fn compare(reports: &[(String, String)], expected_reports: usize) -> Vec<String> {
    let mut problems = Vec::new();
    if reports.len() != expected_reports {
        let labels: Vec<&str> = reports.iter().map(|(label, _)| label.as_str()).collect();
        problems.push(format!(
            "{} reports were expected and {} were found ({})",
            expected_reports,
            reports.len(),
            labels.join(", ")
        ));
    }
    let mut parsed = Vec::new();
    for (label, text) in reports {
        match parse(label, text) {
            Ok(cases) => parsed.push((label.as_str(), cases)),
            Err(problem) => problems.push(problem),
        }
    }
    let Some((base_label, base)) = parsed.first() else {
        return problems;
    };
    for (label, cases) in &parsed[1..] {
        for (name, digest) in base {
            match cases.get(name) {
                None => problems.push(format!(
                    "{label} has no case `{name}`, which {base_label} has"
                )),
                Some(other) if other != digest => problems.push(format!(
                    "case `{name}` differs: {base_label} gives {digest}, {label} gives {other}"
                )),
                Some(_) => {}
            }
        }
        for name in cases.keys().filter(|name| !base.contains_key(*name)) {
            problems.push(format!(
                "{label} has a case `{name}`, which {base_label} lacks"
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(lines: &[&str]) -> String {
        lines.join("\n") + "\n"
    }

    fn reports(texts: &[&str]) -> Vec<(String, String)> {
        texts
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("target{i}"), (*t).to_string()))
            .collect()
    }

    const GOOD: &[&str] = &[
        "sum/a 0123456789abcdef",
        "# sum/a bits 3ff0000000000000",
        "sd/a 00000000000000ff",
    ];

    #[test]
    fn identical_reports_pass() {
        let a = report(GOOD);
        // Comment lines are not compared, so the bits may be absent from one report.
        let b = report(&["sum/a 0123456789abcdef", "sd/a 00000000000000ff"]);
        assert!(compare(&reports(&[&a, &b, &a]), 3).is_empty());
    }

    #[test]
    fn one_changed_digest_fails_and_names_the_case() {
        let a = report(GOOD);
        let b = report(&["sum/a 0123456789abcdee", "sd/a 00000000000000ff"]);
        let problems = compare(&reports(&[&a, &b]), 2);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("sum/a") && problems[0].contains("target1"));
    }

    #[test]
    fn a_missing_target_fails() {
        let a = report(GOOD);
        let problems = compare(&reports(&[&a, &a]), 3);
        assert!(
            problems
                .iter()
                .any(|p| p.contains("3 reports were expected")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_missing_or_extra_case_fails() {
        let a = report(GOOD);
        let fewer = report(&["sum/a 0123456789abcdef"]);
        let more = report(&[
            "sum/a 0123456789abcdef",
            "sd/a 00000000000000ff",
            "mean/a 1",
        ]);
        assert!(
            compare(&reports(&[&a, &fewer]), 2)
                .iter()
                .any(|p| p.contains("no case `sd/a`"))
        );
        assert!(
            compare(&reports(&[&a, &more]), 2)
                .iter()
                .any(|p| p.contains("lacks"))
        );
    }

    #[test]
    fn a_malformed_or_empty_report_fails() {
        let a = report(GOOD);
        assert!(!compare(&reports(&[&a, "sum/a\n"]), 2).is_empty());
        assert!(!compare(&reports(&[&a, "# only a comment\n"]), 2).is_empty());
        assert!(!compare(&reports(&[&a, "x 1\nx 2\n"]), 2).is_empty());
    }
}
