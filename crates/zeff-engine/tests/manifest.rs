//! `zeff.toml`: what a good file gives, and every way a file can be wrong.

use zeff_diagnostics::{Source, Sources};
use zeff_engine::manifest::{Edition, parse_manifest};

fn render(text: &str) -> String {
    let Err(found) = parse_manifest("zeff.toml", text) else {
        panic!("expected problems in {text:?}");
    };
    let mut sources = Sources::new();
    sources.add(Source::new("zeff.toml", text));
    found
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

fn codes(text: &str) -> Vec<String> {
    match parse_manifest("zeff.toml", text) {
        Ok(_) => Vec::new(),
        Err(found) => found.iter().map(|d| d.code.clone()).collect(),
    }
}

const GOOD: &str = "[project]\nname = \"schooling\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n";

#[test]
fn a_minimal_project_file() {
    let manifest = parse_manifest("zeff.toml", GOOD).unwrap();
    assert_eq!(manifest.name, "schooling");
    assert_eq!(manifest.zeff, "0.1.0");
    assert_eq!(manifest.edition, Edition::Preview);
    assert_eq!(manifest.memory, None);
    assert!(!manifest.network);
    assert!(manifest.data_roots.is_empty());
}

#[test]
fn every_setting_of_section_2() {
    let text = "[project]\nname = \"schooling\"\nzeff = \"0.1.0\"\nedition = \"preview\"   # \"2027\" from 1.0.0\nmemory = \"3.5GB\"\nnetwork = true\n\n[data]\nlfs = { path = \"D:/surveys/lfs\" }\ncensus = { path = \"../shared/census\" }\n\n[packages]\n";
    let manifest = parse_manifest("zeff.toml", text).unwrap();
    assert_eq!(manifest.memory.as_deref(), Some("3.5GB"));
    assert!(manifest.network);
    let roots: Vec<(&str, &str)> = manifest
        .data_roots
        .iter()
        .map(|r| (r.name.as_str(), r.path.as_str()))
        .collect();
    assert_eq!(
        roots,
        [("census", "../shared/census"), ("lfs", "D:/surveys/lfs")]
    );
}

#[test]
fn a_development_build_pin_is_an_exact_version() {
    let text = GOOD.replace("0.1.0", "0.1.0-dev");
    assert_eq!(
        parse_manifest("zeff.toml", &text).unwrap().zeff,
        "0.1.0-dev"
    );
}

#[test]
fn the_error_codes() {
    let table: &[(&str, &[&str])] = &[
        ("[project\nname = 1", &["Z0712"]),
        ("", &["Z0713"]),
        ("[project]\n", &["Z0713", "Z0713", "Z0713"]),
        (
            "[project]\nname = \"a\"\nzeff = \"^0.1\"\nedition = \"preview\"\n",
            &["Z0713"],
        ),
        (
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"2027\"\n",
            &["Z0706"],
        ),
        (
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"2030\"\n",
            &["Z0706"],
        ),
        (
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nmemory = \"lots\"\n",
            &["Z0713"],
        ),
        (
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nnetwork = \"yes\"\n",
            &["Z0713"],
        ),
        (
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nnetwrk = true\n",
            &["Z0713"],
        ),
        (
            "[project]\nname = 5\nzeff = \"0.1.0\"\nedition = \"preview\"\n",
            &["Z0713"],
        ),
        (
            "[project]\nname = \"\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n",
            &["Z0713"],
        ),
        (
            "[database]\nx = 1\n[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n",
            &["Z0713"],
        ),
        ("project = 3\n", &["Z0713"]),
    ];
    for (text, expected) in table {
        assert_eq!(codes(text), *expected, "{text:?}\n{}", render(text));
    }
}

#[test]
fn problems_in_the_data_section() {
    let head = GOOD.to_string();
    let table: &[(&str, &[&str])] = &[
        ("[data]\nlfs = \"D:/x\"\n", &["Z0713"]),
        ("[data]\nlfs = { }\n", &["Z0713"]),
        ("[data]\nlfs = { path = 3 }\n", &["Z0713"]),
        ("[data]\nlfs = { path = \"\" }\n", &["Z0713"]),
        (
            "[data]\nlfs = { path = \"x\", mode = \"rw\" }\n",
            &["Z0713"],
        ),
        ("[data]\nzeff = { path = \"x\" }\n", &["Z0713"]),
        ("[data]\n\"two words\" = { path = \"x\" }\n", &["Z0713"]),
        ("[packages]\nsurvival = \"^1.2\"\n", &["Z0713"]),
    ];
    for (extra, expected) in table {
        let text = format!("{head}\n{extra}");
        assert_eq!(codes(&text), *expected, "{text:?}\n{}", render(&text));
    }
}

#[test]
fn every_problem_is_reported_at_once() {
    let text = "[project]\nname = 5\nzeff = \"^0.1\"\nedition = \"2030\"\nmemory = \"lots\"\nnetwork = 1\nbogus = 2\n";
    let found = codes(text);
    assert_eq!(found.len(), 6, "{found:?}\n{}", render(text));
}

#[test]
fn messages() {
    let cases: &[(&str, &str)] = &[
        ("toml_syntax_error", "[project\nname = \"a\"\n"),
        ("no_project_table", "[data]\n"),
        ("missing_keys", "[project]\n"),
        (
            "version_range",
            "[project]\nname = \"a\"\nzeff = \"^0.1\"\nedition = \"preview\"\n",
        ),
        (
            "edition_2027",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"2027\"\n",
        ),
        (
            "unknown_edition",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"2030\"\n",
        ),
        (
            "bad_memory",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nmemory = \"lots\"\n",
        ),
        (
            "network_not_a_bool",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nnetwork = \"yes\"\n",
        ),
        (
            "misspelt_key",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\nnetwrk = true\n",
        ),
        (
            "name_not_text",
            "[project]\nname = 5\nzeff = \"0.1.0\"\nedition = \"preview\"\n",
        ),
        (
            "unknown_table",
            "[databse]\nx = 1\n[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n",
        ),
        (
            "data_root_not_a_table",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n[data]\nlfs = \"D:/x\"\n",
        ),
        (
            "data_root_without_path",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n[data]\nlfs = { }\n",
        ),
        (
            "data_root_zeff",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n[data]\nzeff = { path = \"x\" }\n",
        ),
        (
            "packages_not_yet",
            "[project]\nname = \"a\"\nzeff = \"0.1.0\"\nedition = \"preview\"\n[packages]\nsurvival = \"^1.2\"\n",
        ),
    ];
    for (name, text) in cases {
        insta::assert_snapshot!(*name, render(text));
    }
}
