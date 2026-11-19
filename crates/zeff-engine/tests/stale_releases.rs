//! No message, explained page or help page names a release that has shipped as one to come
//! (release 0.8.0, step 20h; step 17 held the same of 0.8.0 alone). The sources of the engine
//! and of the packages, the explained pages and the help pages are read for a release up to
//! the current one written after words that look forward: "arrives with", "waits for
//! release", "take from release", "ships later in release", and in a message a bare
//! "(release 0.5.0)", which marked a function as new. "Since release 0.8.0" looks back and is
//! history; so is anything in a comment.

use std::path::{Path, PathBuf};

/// The files under `dir` with one of the extensions, in order.
fn files(dir: &Path, extensions: &[&str], found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        if path.is_dir() {
            if !matches!(
                name.as_deref(),
                Some("target" | "tests" | "snapshots" | "node_modules")
            ) {
                files(&path, extensions, found);
            }
        } else if path
            .extension()
            .is_some_and(|e| extensions.iter().any(|wanted| e == *wanted))
        {
            found.push(path);
        }
    }
}

/// The releases written in a line as `0.N.0` with N up to `current`, each with the text that
/// stands before it.
fn shipped(line: &str, current: u32) -> Vec<(&str, &str)> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let mut at = 0;
    while let Some(offset) = line[at..].find("0.") {
        let start = at + offset;
        let digits: String = line[start + 2..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let end = start + 2 + digits.len();
        let whole = start == 0 || !(bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.');
        if whole
            && !digits.is_empty()
            && line[end..].starts_with(".0")
            && !line[end + 2..].starts_with(|c: char| c.is_ascii_digit())
            && digits.parse::<u32>().is_ok_and(|minor| minor <= current)
        {
            found.push((&line[..start], &line[start..end + 2]));
        }
        at = start + 2;
    }
    found
}

/// Whether the words before a release look forward to it: "arrives with release 0.5.0". In a
/// message, a bare "(release 0.5.0)" does too: it marked a function as new.
fn looks_forward(before: &str, line: &str, is_source: bool) -> bool {
    let before = before.to_lowercase();
    let forward = [
        "arrives with ",
        "arrive with ",
        "arrives with release ",
        "arrive with release ",
        "waits for release ",
        "wait for release ",
        "take from release ",
        "takes from release ",
        "ships later in release ",
        "ship later in release ",
    ];
    !before.ends_with("since release ")
        && (forward.iter().any(|words| before.ends_with(words))
            // "the `time` package of 0.6.0 brings dates and times to models"
            || (before.ends_with(" of ") && line.contains(" brings "))
            || (is_source && before.ends_with("(release ")))
}

#[test]
fn no_text_names_a_release_that_has_shipped_as_one_to_come() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let current: u32 = zeff_engine::VERSION
        .split('.')
        .nth(1)
        .and_then(|minor| minor.parse().ok())
        .expect("a version of three numbers");
    let mut sources = Vec::new();
    for part in ["packages", "kernel", "crates"] {
        files(&root.join(part), &["rs"], &mut sources);
    }
    let mut pages = Vec::new();
    files(&root.join("crates/zeff-engine/codes"), &["md"], &mut pages);
    files(&root.join("packages"), &["md"], &mut pages);
    assert!(
        sources.len() > 100 && pages.len() > 300,
        "{} sources, {} pages",
        sources.len(),
        pages.len()
    );
    let mut stale: Vec<String> = Vec::new();
    for (path, is_source) in sources
        .iter()
        .map(|p| (p, true))
        .chain(pages.iter().map(|p| (p, false)))
    {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        for (number, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            // A comment is history, not a message; so is a test's own text.
            if is_source && (trimmed.starts_with("//") || !line.contains('"')) {
                continue;
            }
            for (before, release) in shipped(line, current) {
                if looks_forward(before, line, is_source) {
                    stale.push(format!(
                        "{}:{}: {release}: {}",
                        path.strip_prefix(&root).unwrap_or(path).display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "a release up to 0.{current}.0 is named as one to come:\n{}",
        stale.join("\n")
    );
}

#[test]
fn the_search_finds_what_it_is_for() {
    let found = |line: &str| -> Vec<String> {
        shipped(line, 8)
            .into_iter()
            .map(|(_, release)| release.to_string())
            .collect()
    };
    assert_eq!(
        found("`to(NA, Float)` arrives with release 0.5.0"),
        ["0.5.0"]
    );
    assert_eq!(
        found("from 0.6.0 to 0.8.0, and 0.11.0 later"),
        ["0.6.0", "0.8.0"]
    );
    assert!(found("version 10.5.0, 0.50, 1.0.5.0 and 0.5.01").is_empty());
    assert!(found("arrive with release 0.9.0").is_empty());
    // The texts this search was written after, each as it stood, and what is history.
    let stale = |line: &str, is_source: bool| -> bool {
        shipped(line, 8)
            .into_iter()
            .any(|(before, _)| looks_forward(before, line, is_source))
    };
    for line in [
        "\"`{text}` is a {} variable, which models take from release 0.6.0\"",
        ".help(\"the `time` package of 0.6.0 brings dates and times to models\")",
        ".help(\"`x * NA`; `to(NA, Float)` arrives with release 0.5.0\")",
        ".help(\"`merge` joins frames (release 0.6.0)\")",
        ".help(\"a model for a categorical outcome arrives with `logistic` (release 0.5.0)\")",
    ] {
        assert!(stale(line, true), "{line}");
    }
    for line in [
        "`equal(a, b)` for structural equality arrives with 0.5.0.",
        "the bug-report builder arrives with 0.8.0.",
        "with `name_of` (and `join`, once the text package ships later in release 0.7.0).",
    ] {
        assert!(stale(line, false), "{line}");
    }
    for (line, is_source) in [
        (
            "\"`add_days(d, n)` is `add_time(d, n)` since release 0.8.0\"",
            true,
        ),
        (
            "and, from release 0.6.0, the firm the worker belongs to",
            false,
        ),
        ("\"causal graphs arrive with release 0.10.0\"", true),
        ("\"arrives with `logistic` (release 0.9.0)\"", true),
    ] {
        assert!(!stale(line, is_source), "{line}");
    }
}
