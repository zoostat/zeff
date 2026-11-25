//! The known-issues list as data (Language Reference, section 11.1; release 0.9.0, step 22a):
//! the list the engine carries is read, says which calls on which versions each issue affects,
//! and agrees with `docs/known_issues.md` entry for entry.

use zeff_engine::known_issues::{Issue, SHIPPED, read, shipped};

fn issue(from: &str, fixed: Option<&str>, functions: &[&str], options: &[&str]) -> Issue {
    Issue {
        number: 123,
        summary: "a wrong result".to_string(),
        functions: functions.iter().map(|name| (*name).to_string()).collect(),
        options: options.iter().map(|name| (*name).to_string()).collect(),
        from: from.to_string(),
        fixed: fixed.map(str::to_string),
        affected_when: "always".to_string(),
        document: "words".to_string(),
    }
}

#[test]
fn the_list_the_engine_carries_is_read() {
    let issues = read(SHIPPED).expect("the shipped list reads");
    assert_eq!(issues, shipped());
    let numbers: Vec<u64> = issues.iter().map(|issue| issue.number).collect();
    assert_eq!(numbers, [1, 2]);
    assert_eq!(issues[0].functions, ["mad", "skewness", "kurtosis"]);
    assert_eq!(
        (issues[0].from.as_str(), issues[0].fixed.as_deref()),
        ("0.5.0", Some("0.7.0"))
    );
    assert_eq!(issues[1].functions, ["correlation"]);
    assert_eq!(
        (issues[1].from.as_str(), issues[1].fixed.as_deref()),
        ("0.3.0", Some("0.7.0"))
    );
    // This engine is past both: neither affects it.
    for found in &issues {
        assert!(
            !found.affects_version(zeff_engine::VERSION),
            "{}",
            found.number
        );
    }
}

#[test]
fn an_issue_affects_the_versions_from_its_first_to_the_one_that_corrects_it() {
    // Found in 0.9.0, corrected in the patch 0.9.2, as the example of section 11.1.
    let patched = issue("0.9.0", Some("0.9.2"), &["reg"], &["weights"]);
    for (version, affected) in [
        ("0.8.0", false),
        ("0.9.0-dev", false),
        ("0.9.0", true),
        ("0.9.1", true),
        ("0.9.2-rc.1", true),
        ("0.9.2", false),
        ("0.9.3", false),
        ("0.10.0", false),
        ("1.0.0", false),
        ("0.9", false),
        ("the newest", false),
        ("", false),
    ] {
        assert_eq!(patched.affects_version(version), affected, "{version}");
    }
    // One that no version corrects yet affects every version from its first.
    let open = issue("0.9.1", None, &["glm"], &[]);
    assert!(!open.affects_version("0.9.0"));
    assert!(open.affects_version("0.9.1") && open.affects_version("2.3.4"));
    // Numbers are compared as numbers: 0.10.0 is after 0.9.2.
    let later = issue("0.10.0", Some("0.10.1"), &["reg"], &[]);
    assert!(!later.affects_version("0.9.2") && later.affects_version("0.10.0"));
}

#[test]
fn an_issue_affects_the_calls_of_its_functions_with_its_options() {
    let written = |options: &[&str]| -> Vec<String> {
        options.iter().map(|option| (*option).to_string()).collect()
    };
    let with_weights = issue(
        "0.9.0",
        Some("0.9.2"),
        &["reg", "ivreg"],
        &["weights", "weight_type"],
    );
    assert!(with_weights.affects_call("reg", &written(&["se", "weights"])));
    assert!(with_weights.affects_call("ivreg", &written(&["weight_type"])));
    assert!(!with_weights.affects_call("reg", &written(&["se", "cluster"])));
    assert!(!with_weights.affects_call("reg", &written(&[])));
    assert!(!with_weights.affects_call("glm", &written(&["weights"])));
    // No option named: every call of the function.
    let every = issue("0.5.0", Some("0.7.0"), &["mad"], &[]);
    assert!(every.affects_call("mad", &written(&[])));
    assert!(every.affects_call("mad", &written(&["by"])));
    assert!(!every.affects_call("iqr", &written(&[])));
}

#[test]
fn a_list_that_is_not_whole_is_refused() {
    let entry = "[[issue]]\nnumber = 3\nsummary = \"s\"\nfunctions = [\"reg\"]\noptions = []\nfrom = \"0.9.0\"\nfixed = \"0.9.2\"\naffected_when = \"w\"\ndocument = \"d\"\n";
    assert_eq!(
        read(&format!("format = 1\n{entry}")).map(|issues| issues.len()),
        Ok(1)
    );
    assert_eq!(read("format = 1\n"), Ok(Vec::new()));
    // Still open: no `fixed` at all.
    let open = entry.replace("fixed = \"0.9.2\"\n", "");
    assert_eq!(
        read(&format!("format = 1\n{open}")).map(|issues| issues[0].fixed.clone()),
        Ok(None)
    );
    for (text, reason) in [
        (entry.to_string(), "the list's `format` is not 1"),
        (
            "format = 1\n[[issue]\n".to_string(),
            "the list is not valid TOML",
        ),
        (
            "format = 1\nissue = 3\n".to_string(),
            "`issue` is not a list of tables",
        ),
        (
            format!("format = 1\n{}", entry.replace("number = 3\n", "")),
            "issue 1 of the list has no `number`",
        ),
        (
            format!("format = 1\n{entry}{entry}"),
            "the list has two issues numbered 3",
        ),
        (
            format!("format = 1\n{}", entry.replace("[\"reg\"]", "[]")),
            "issue 1 of the list has no `functions`, a list of names",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("from = \"0.9.0\"", "from = \"0.9\"")
            ),
            "issue 1 of the list has no `from`, a version",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("fixed = \"0.9.2\"", "fixed = \"soon\"")
            ),
            "issue 1 of the list has no `fixed` that is a version",
        ),
        (
            format!("format = 1\n{}", entry.replace("options = []\n", "")),
            "issue 1 of the list has no `options`, a list of names",
        ),
        (
            format!("format = 1\n{}", entry.replace("document = \"d\"\n", "")),
            "issue 1 of the list has no `document`",
        ),
    ] {
        assert_eq!(read(&text), Err(reason.to_string()), "{text}");
    }
}

/// The refusals no test had asked for (release 0.9.0, step 42c3): an entry that is not a table,
/// a name that is not text, no summary and no word on when a call is affected, and versions
/// written in forms that are not versions.
#[test]
fn the_other_ways_a_list_is_not_whole() {
    let entry = "[[issue]]\nnumber = 3\nsummary = \"s\"\nfunctions = [\"reg\"]\noptions = []\nfrom = \"0.9.0\"\nfixed = \"0.9.2\"\naffected_when = \"w\"\ndocument = \"d\"\n";
    assert_eq!(
        read(&format!("format = 1\n{entry}")).map(|issues| issues.len()),
        Ok(1)
    );
    for (text, reason) in [
        (
            "format = 1\nissue = [3]\n".to_string(),
            "`issue` is not a list of tables",
        ),
        (
            format!("format = 1\n{}", entry.replace("[\"reg\"]", "[\"reg\", 3]")),
            "issue 1 of the list has no `functions`, a list of names",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("options = []", "options = [\"\"]")
            ),
            "issue 1 of the list has no `options`, a list of names",
        ),
        (
            format!("format = 1\n{}", entry.replace("summary = \"s\"\n", "")),
            "issue 1 of the list has no `summary`",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("affected_when = \"w\"\n", "")
            ),
            "issue 1 of the list has no `affected_when`",
        ),
        // A hyphen with nothing after it, a part that is no number, and four parts.
        (
            format!(
                "format = 1\n{}",
                entry.replace("from = \"0.9.0\"", "from = \"0.9.0-\"")
            ),
            "issue 1 of the list has no `from`, a version",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("from = \"0.9.0\"", "from = \"0.x.0\"")
            ),
            "issue 1 of the list has no `from`, a version",
        ),
        (
            format!(
                "format = 1\n{}",
                entry.replace("from = \"0.9.0\"", "from = \"0.9.0.1\"")
            ),
            "issue 1 of the list has no `from`, a version",
        ),
    ] {
        assert_eq!(read(&text), Err(reason.to_string()), "{text}");
    }
    // A version before its release is a version.
    assert_eq!(
        read(&format!(
            "format = 1\n{}",
            entry.replace("from = \"0.9.0\"", "from = \"0.9.0-rc.1\"")
        ))
        .map(|issues| issues[0].from.clone()),
        Ok("0.9.0-rc.1".to_string())
    );
}

/// The entries "A wrong result" of `docs/known_issues.md`, each from its bullet to the next.
fn wrong_results(document: &str) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut inside = false;
    for line in document.lines() {
        if line.starts_with("- ") || line.starts_with('#') {
            inside = line.starts_with("- **A wrong result");
            if inside {
                entries.push(String::new());
            }
        }
        if inside && let Some(entry) = entries.last_mut() {
            entry.push_str(line.trim());
            entry.push(' ');
        }
    }
    entries
}

/// The list is built from the document and held to it: every wrong result the document names
/// is an issue of the list and every issue of the list is one the document names, with the
/// same functions, the same first version and the same correcting version.
#[test]
fn the_list_and_the_document_agree_entry_for_entry() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/known_issues.md");
    let document = std::fs::read_to_string(&path).expect("docs/known_issues.md");
    let entries = wrong_results(&document);
    let issues = shipped();
    assert_eq!(
        entries.len(),
        issues.len(),
        "the document has {} entries \"A wrong result\" and the list {} issues",
        entries.len(),
        issues.len()
    );
    for found in &issues {
        let quoting: Vec<&String> = entries
            .iter()
            .filter(|entry| entry.contains(&found.document))
            .collect();
        assert_eq!(
            quoting.len(),
            1,
            "issue {}: its words are in {} entries of the document",
            found.number,
            quoting.len()
        );
        let entry = quoting[0];
        for function in &found.functions {
            assert!(
                entry.contains(&format!("`{function}`")),
                "issue {}: `{function}`",
                found.number
            );
        }
        assert!(
            entry.contains(&format!("From {} to ", found.from)),
            "issue {}: from",
            found.number
        );
        let fixed = found.fixed.as_deref().expect("both are corrected");
        assert!(
            entry.contains(&format!(" of {fixed}")),
            "issue {}: corrected at {fixed}",
            found.number
        );
    }
    // And no entry of the document is quoted by two issues.
    for entry in &entries {
        let quoted = issues
            .iter()
            .filter(|found| entry.contains(&found.document))
            .count();
        assert_eq!(quoted, 1, "{entry}");
    }
}
