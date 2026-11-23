//! The citation file of the repository (release 0.9.0, step 37; open decision #161, option A):
//! `CITATION.cff`, in the Citation File Format 1.2.0, so that an article can cite an exact
//! version and the repository's page offers the citation.
//!
//! The file names the last version released. While the workspace is at a development
//! version, `0.9.0-dev`, that is a version before it; once the workspace is set to a
//! release, `0.9.0`, the file must name that version, so the release cannot be made without
//! bringing the citation to it.

use std::path::Path;

fn text(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(name),
    )
    .unwrap()
}

/// The value of a key at the top of the file, without its quotation marks.
fn value<'a>(file: &'a str, key: &str) -> Option<&'a str> {
    file.lines().find_map(|line| {
        let rest = line.strip_prefix(key)?.strip_prefix(':')?;
        Some(rest.trim().trim_matches('"'))
    })
}

/// `major.minor.patch` as three numbers.
fn parts(version: &str) -> Option<[u32; 3]> {
    let mut numbers = version.split('.').map(|n| n.parse::<u32>().ok());
    let found = [numbers.next()??, numbers.next()??, numbers.next()??];
    numbers.next().is_none().then_some(found)
}

#[test]
fn the_citation_file_has_what_the_format_requires() {
    let file = text("CITATION.cff");
    // The fields the format requires: its version, the message, the title and an author.
    assert_eq!(value(&file, "cff-version"), Some("1.2.0"));
    assert!(value(&file, "message").is_some_and(|m| m.len() > 20));
    assert!(value(&file, "title").is_some_and(|t| t.starts_with("Zeff")));
    assert_eq!(value(&file, "type"), Some("software"));
    assert!(file.contains("\nauthors:\n  - family-names: "), "{file}");
    assert!(file.contains("\n    given-names: "), "{file}");
    // The licence and the repository are those of the workspace.
    let workspace = text("Cargo.toml");
    let licence = value(&file, "license").unwrap();
    assert!(
        workspace.contains(&format!("license = \"{licence}\"")),
        "{licence}"
    );
    assert_eq!(
        value(&file, "repository-code"),
        Some("https://github.com/zoostat/zeff")
    );
    // No line is a key twice, and nothing is left to fill in.
    for key in ["version", "date-released", "title", "license"] {
        let times = file
            .lines()
            .filter(|line| line.starts_with(&format!("{key}:")))
            .count();
        assert_eq!(times, 1, "{key}");
    }
    assert!(!file.contains("TODO") && !file.contains("TBD"), "{file}");
}

#[test]
fn the_citation_names_the_version_released() {
    let file = text("CITATION.cff");
    let cited = value(&file, "version").unwrap();
    let cited_parts = parts(cited).unwrap_or_else(|| panic!("`{cited}` is not a version"));
    let engine = zeff_engine::VERSION;
    match engine.strip_suffix("-dev") {
        // In development: the citation is of a version released before this one.
        Some(coming) => {
            let coming = parts(coming).unwrap();
            assert!(
                cited_parts < coming,
                "the workspace is at {engine}; the citation names {cited}, which is not before it"
            );
        }
        // At a release: the citation is of this version.
        None => assert_eq!(
            cited, engine,
            "set `version` and `date-released` of CITATION.cff to this release"
        ),
    }
    // The date of that release, as year-month-day.
    let date = value(&file, "date-released").unwrap();
    let numbers: Vec<u32> = date.split('-').filter_map(|n| n.parse().ok()).collect();
    assert!(
        date.len() == 10
            && numbers.len() == 3
            && numbers[0] >= 2026
            && (1..=12).contains(&numbers[1])
            && (1..=31).contains(&numbers[2]),
        "`{date}` is not a date"
    );
}
