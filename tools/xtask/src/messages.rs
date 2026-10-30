//! The message-snapshot rule (Language Reference, section 11, "Message snapshots", from 0.2.0):
//! every diagnostic code the engine can raise has a stored copy of its wording. The check lists
//! the codes written in the sources (`"Z0101"`, `"W0201"`) and looks for each in the snapshot
//! files: an error or a check-time warning appears as `[Z0101]` in its rendered form; a warning
//! printed under a result has no code (open decision #45), so it is found by a phrase of its
//! wording, kept in the table below. From 0.2.0 every code also has an explained page,
//! `crates/zeff-engine/codes/<CODE>.md` (Language Reference, section 11.1), and a page for a code
//! nothing raises is stale: the check reports both.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// For the warnings printed without their code, a phrase every snapshot of them contains.
const PHRASES: &[(&str, &[&str])] = &[
    ("W0102", &["and a binding; the variable was used"]),
    ("W0201", &["with NA in", "has no values", "have NA in"]),
    (
        "W0202",
        &[
            "has a fraction, so to(Int) gave NA",
            "with a fraction became NA under to(Int)",
        ],
    ),
    (
        "W0203",
        &["could not be converted to", "none of the levels given"],
    ),
    ("W0207", &["the fit never saw"]),
    (
        "W0208",
        &[
            "matched no arm of the `case`",
            "no arm of the `case` matched",
        ],
    ),
    ("W0602", &["levels, more than"]),
    (
        "W0801",
        &["fits failed (", "and were skipped", "and was skipped"],
    ),
    ("W1001", &["near-singular regressors"]),
    ("W1002", &["few clusters"]),
    ("W1003", &["extreme weights"]),
    ("W1006", &["a coefficient: removing it moves an estimate"]),
    (
        "W0206",
        &[
            "is not a level of `",
            "is not a value of `",
            "keys are none of the levels of `",
            "keys are none of the values of `",
        ],
    ),
    (
        "W0204",
        &[
            "division by zero gave",
            "holds both Inf and -Inf",
            "of zero gave",
            "log1p of -1 gave",
        ],
    ),
    (
        "W0205",
        &[
            "gave NA",
            "needs at least two values",
            "holds an infinite value",
        ],
    ),
    ("W0601", &["Unicode normalisation form"]),
    ("W0702", &["time-zone offset"]),
];

/// Codes a source mentions without raising them: the exit-code table names the lock mismatch
/// (Z0703) that arrives with `zeff.lock`.
const MENTIONED_NOT_RAISED: &[&str] = &["Z0703"];

/// The codes written in a source text, as `"Z0101"` or `"W0201"` string literals, outside comments
/// and outside the test module at the end of a file (`#[cfg(test)]`), which quotes codes as
/// examples rather than raising them.
pub fn codes_in_source(source: &str) -> BTreeSet<String> {
    let mut kept = String::new();
    for line in source.lines() {
        if line.contains("#[cfg(test)]") {
            break;
        }
        if line.trim_start().starts_with("//") {
            continue;
        }
        kept.push_str(line);
        kept.push('\n');
    }
    let text = kept.as_str();
    let bytes = text.as_bytes();
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i + 7 <= bytes.len() {
        if bytes[i] == b'"'
            && (bytes[i + 1] == b'Z' || bytes[i + 1] == b'W')
            && bytes[i + 2..i + 6].iter().all(u8::is_ascii_digit)
            && bytes[i + 6] == b'"'
        {
            let code = &text[i + 1..i + 6];
            if !MENTIONED_NOT_RAISED.contains(&code) {
                found.insert(code.to_string());
            }
            i += 7;
        } else {
            i += 1;
        }
    }
    found
}

/// The codes a snapshot text proves: those in brackets, and those whose phrase it carries.
pub fn codes_in_snapshot(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 7 <= bytes.len() {
        if bytes[i] == b'['
            && (bytes[i + 1] == b'Z' || bytes[i + 1] == b'W')
            && bytes[i + 2..i + 6].iter().all(u8::is_ascii_digit)
            && bytes[i + 6] == b']'
        {
            found.insert(text[i + 1..i + 6].to_string());
            i += 7;
        } else {
            i += 1;
        }
    }
    for (code, phrases) in PHRASES {
        if phrases.iter().any(|p| text.contains(p)) {
            found.insert((*code).to_string());
        }
    }
    found
}

/// The codes in the sources that no snapshot proves, with where each is written.
pub fn missing(
    sources: &[(String, String)],
    snapshots: &[String],
) -> BTreeMap<String, Vec<String>> {
    let mut proven = BTreeSet::new();
    for snapshot in snapshots {
        proven.extend(codes_in_snapshot(snapshot));
    }
    let mut missing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, text) in sources {
        for code in codes_in_source(text) {
            if !proven.contains(&code) {
                missing.entry(code).or_default().push(path.clone());
            }
        }
    }
    missing
}

/// The codes without a page and the pages without a code, given the codes in the sources and the
/// codes that have a page.
pub fn page_gaps(
    raised: &BTreeSet<String>,
    pages: &BTreeSet<String>,
) -> (Vec<String>, Vec<String>) {
    let without_page = raised.difference(pages).cloned().collect();
    let stale = pages.difference(raised).cloned().collect();
    (without_page, stale)
}

/// The codes that have an explained page: the `<CODE>.md` files of the pages folder.
pub fn pages(root: &Path) -> std::io::Result<BTreeSet<String>> {
    let mut found = BTreeSet::new();
    for entry in std::fs::read_dir(root.join("crates/zeff-engine/codes"))? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if let Some(code) = name.strip_suffix(".md") {
            found.insert(code.to_string());
        }
    }
    Ok(found)
}

/// Source files as (path, text).
pub type Sources = Vec<(String, String)>;

/// Every `.rs` file under the engine's source folders and every `.snap` file in the workspace.
pub fn gather(root: &Path) -> std::io::Result<(Sources, Vec<String>)> {
    let mut sources = Vec::new();
    let mut snapshots = Vec::new();
    walk(root, root, &mut sources, &mut snapshots)?;
    sources.sort();
    Ok((sources, snapshots))
}

fn walk(
    root: &Path,
    dir: &Path,
    sources: &mut Sources,
    snapshots: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if matches!(name.as_str(), "target" | "node_modules" | ".git" | "fuzz") {
                continue;
            }
            walk(root, &path, sources, snapshots)?;
        } else if name.ends_with(".snap") {
            snapshots.push(std::fs::read_to_string(&path)?);
        } else if name.ends_with(".rs") {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            // The codes the engine raises are written in `src/`; tests and tools mention codes
            // they check, which is not raising them.
            if relative.contains("/src/") && !relative.starts_with("tools/") {
                sources.push((relative, std::fs::read_to_string(&path)?));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_found_in_sources_and_snapshots() {
        let source = "Diagnostic::error(\"Z0101\", \"x\").help(\"Z01\"); warning(\"W0201\", ..); \"ZZ999\" \"Z12345\"\n// a comment mentions \"Z0555\"\n#[cfg(test)]\nmod tests { let _ = \"Z0777\"; }\n";
        let found = codes_in_source(source);
        assert_eq!(found.into_iter().collect::<Vec<_>>(), ["W0201", "Z0101"]);
        let snapshot =
            "error[Z0101]: no variable\nwarning: 3 rows with NA in x were skipped (line 2)\n";
        let proven = codes_in_snapshot(snapshot);
        assert!(proven.contains("Z0101") && proven.contains("W0201"));
        assert!(!proven.contains("W0204"));
    }

    #[test]
    fn a_code_without_a_snapshot_is_reported_with_its_file() {
        let sources = vec![
            (
                "crates/a/src/lib.rs".to_string(),
                r#""Z0101" "Z0999""#.to_string(),
            ),
            ("crates/b/src/lib.rs".to_string(), r#""Z0999""#.to_string()),
        ];
        let snapshots = vec!["error[Z0101]: fine".to_string()];
        let gaps = missing(&sources, &snapshots);
        assert_eq!(gaps.len(), 1);
        assert_eq!(
            gaps["Z0999"],
            ["crates/a/src/lib.rs", "crates/b/src/lib.rs"]
        );
    }

    #[test]
    fn pages_are_matched_against_the_codes_raised() {
        let raised: BTreeSet<String> = ["Z0101", "Z0999"].iter().map(|s| s.to_string()).collect();
        let pages: BTreeSet<String> = ["Z0101", "Z0555"].iter().map(|s| s.to_string()).collect();
        let (without_page, stale) = page_gaps(&raised, &pages);
        assert_eq!(without_page, ["Z0999"]);
        assert_eq!(stale, ["Z0555"]);
    }
}
