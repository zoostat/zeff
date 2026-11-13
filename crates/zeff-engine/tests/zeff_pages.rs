//! The "Warnings and notes" of the built-ins written in Zeff (release 0.7.0, step 32f; open
//! decision #139). A page lists what the functions its body calls declare, less the codes the
//! body cannot raise, plus the codes of its own operators and conditions. The two lists are
//! kept by hand beside each function, and this file holds them to the truth in both
//! directions: every code on a page is raised through the function by a script here, and no
//! script here raises a code that the page of its function does not list.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::{registry, zeff_bodies};
use zeff_engine::run::run_script;

/// A function, a code of its page, and a script that raises the code through the function.
const SHOWN: [(&str, &str, &str); 49] = [
    ("assert", "Z0001", "assert(1 == 2, \"one is not two\")"),
    ("assert", "Z0203", "assert(NA, \"no condition\")"),
    (
        "fill_missing",
        "Z0214",
        "frame(x = [1.0, NA]).add(y = fill_missing(x, \"a\"))",
    ),
    ("logit", "W0204", "logit(1.0)"),
    ("logit", "W0205", "logit(2.0)"),
    (
        "center",
        "W0201",
        "frame(x = [1.0, NA, 3]).add(c = center(x))",
    ),
    (
        "center",
        "W0204",
        "frame(x = [1.0, Inf, -Inf]).add(c = center(x))",
    ),
    (
        "center",
        "Z0401",
        "frame(x = [\"a\", \"b\"]).add(c = center(x))",
    ),
    (
        "standardize",
        "W0201",
        "frame(x = [1.0, NA, 3, 5]).add(c = standardize(x))",
    ),
    (
        "standardize",
        "W0204",
        "frame(x = [1.0, Inf, -Inf]).add(c = standardize(x))",
    ),
    (
        "standardize",
        "W0205",
        "frame(x = [1.0]).add(c = standardize(x))",
    ),
    (
        "standardize",
        "Z0401",
        "frame(x = [\"a\", \"b\"]).add(c = standardize(x))",
    ),
    (
        "winsorize",
        "Z0001",
        "frame(x = [1.0, 2, 3]).add(c = winsorize(x, at: 0.9))",
    ),
    (
        "winsorize",
        "Z0203",
        "frame(x = [1.0, 2, 3]).add(c = winsorize(x, at: NA))",
    ),
    (
        "winsorize",
        "Z0401",
        "frame(x = [\"a\", \"b\"]).add(c = winsorize(x))",
    ),
    (
        "winsorize",
        "W0201",
        "frame(x = [1.0, NA, 3]).add(c = winsorize(x))",
    ),
    (
        "winsorize",
        "W0205",
        "frame(x = [-Inf, Inf]).add(c = winsorize(x, at: 0.5))",
    ),
    (
        "quantile_group",
        "Z0001",
        "frame(x = [1.0, 2, 3]).add(g = quantile_group(x, n: 0))",
    ),
    (
        "quantile_group",
        "Z0203",
        "frame(x = [1.0, 2, 3]).add(g = quantile_group(x, n: 2, type: NA))",
    ),
    (
        "quantile_group",
        "Z0401",
        "frame(x = [\"a\", \"b\"]).add(g = quantile_group(x, n: 2))",
    ),
    (
        "quantile_group",
        "Z0418",
        "frame(x = [1.0, 2, 3]).add(g = quantile_group(x, n: 20000000))",
    ),
    (
        "quantile_group",
        "W0205",
        "frame(x = [-Inf, Inf]).add(g = quantile_group(x, n: 2))",
    ),
    ("iqr", "Z0401", "frame(x = [1.0, 2, 3]).iqr(x, type: 12)"),
    ("iqr", "Z0101", "frame(x = [1.0, 2, 3]).iqr(nope)"),
    (
        "iqr",
        "Z0106",
        "frame(x = [1.0, 2, 3], var = [1.0, 2, 3]).iqr(x)",
    ),
    ("iqr", "W0201", "frame(x = [1.0, NA, 3]).iqr(x)"),
    ("iqr", "W0205", "frame(x = [-Inf, Inf]).iqr(x)"),
    ("mad", "Z0401", "frame(x = [\"a\", \"b\"]).mad(x)"),
    ("mad", "Z0101", "frame(x = [1.0, 2, 3]).mad(nope)"),
    (
        "mad",
        "Z0106",
        "frame(x = [1.0, 2, 3], var = [1.0, 2, 3]).mad(x)",
    ),
    ("mad", "W0201", "frame(x = [1.0, NA, 3]).mad(x)"),
    ("mad", "W0205", "frame(x = [-Inf, Inf]).mad(x)"),
    ("skewness", "W0205", "frame(x = [1.0, 2]).skewness(x)"),
    (
        "skewness",
        "Z0401",
        "frame(x = [\"a\", \"b\", \"c\"]).skewness(x)",
    ),
    (
        "skewness",
        "Z0203",
        "frame(x = [1.0, 2, 3, 9]).skewness(x, adjusted: NA)",
    ),
    (
        "skewness",
        "Z0106",
        "frame(x = [1.0, 2, 3], var = [1.0, 2, 3]).skewness(x)",
    ),
    (
        "skewness",
        "W0201",
        "frame(x = [1.0, 2, NA, 3, 9]).skewness(x)",
    ),
    (
        "skewness",
        "W0204",
        "frame(x = [1.0, Inf, -Inf, 2, 3]).skewness(x)",
    ),
    ("skewness", "Z0101", "frame(x = [1.0, 2, 3]).skewness(nope)"),
    ("skewness", "Z0102", "frame(x = [1.0, 2, 3]).skewness(X)"),
    ("kurtosis", "W0205", "frame(x = [1.0, 2, 3]).kurtosis(x)"),
    (
        "kurtosis",
        "Z0401",
        "frame(x = [\"a\", \"b\", \"c\"]).kurtosis(x)",
    ),
    (
        "kurtosis",
        "Z0203",
        "frame(x = [1.0, 2, 3, 9, 4]).kurtosis(x, excess: NA)",
    ),
    (
        "kurtosis",
        "Z0106",
        "frame(x = [1.0, 2, 3], var = [1.0, 2, 3]).kurtosis(x)",
    ),
    (
        "kurtosis",
        "W0201",
        "frame(x = [1.0, 2, NA, 3, 9, 4]).kurtosis(x)",
    ),
    (
        "kurtosis",
        "W0204",
        "frame(x = [1.0, Inf, -Inf, 2, 3]).kurtosis(x)",
    ),
    ("kurtosis", "Z0101", "frame(x = [1.0, 2, 3]).kurtosis(nope)"),
    ("kurtosis", "Z0102", "frame(x = [1.0, 2, 3]).kurtosis(X)"),
    (
        "kurtosis",
        "Z0203",
        "frame(x = [1.0, 2, 3, 9, 4]).kurtosis(x, adjusted: NA)",
    ),
];

/// The codes each page lists, by function: the built-ins written in Zeff, all of them.
fn pages() -> BTreeMap<String, BTreeSet<String>> {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    zeff_bodies()
        .keys()
        .map(|(package, name)| {
            let function = registry.function(package, name).expect("registered");
            let codes = function
                .spec
                .diagnostics
                .iter()
                .filter(|d| !d.code.is_empty())
                .map(|d| d.code.to_string())
                .collect();
            (name.clone(), codes)
        })
        .collect()
}

/// The codes a script raises, from its messages.
fn raised(script: &str) -> BTreeSet<String> {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    outcome
        .output
        .stderr
        .lines()
        .filter_map(|line| {
            let rest = ["warning[", "error[", "note["]
                .iter()
                .find_map(|kind| line.strip_prefix(kind))?;
            rest.split_once(']').map(|(code, _)| code.to_string())
        })
        .collect()
}

#[test]
fn every_code_on_a_page_is_shown_by_a_script_and_no_other_is() {
    let pages = pages();
    let mut shown: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (function, code, _) in SHOWN {
        shown.entry(function).or_default().insert(code);
    }
    for (function, codes) in &pages {
        let has: BTreeSet<&str> = shown.get(function.as_str()).cloned().unwrap_or_default();
        let listed: BTreeSet<&str> = codes.iter().map(String::as_str).collect();
        assert_eq!(
            has, listed,
            "the scripts for `{function}` and the codes of its page differ"
        );
    }
    for function in shown.keys() {
        assert!(
            pages.contains_key(*function),
            "`{function}` is no built-in written in Zeff"
        );
    }
}

#[test]
fn each_script_raises_its_code_and_none_that_the_page_leaves_out() {
    let pages = pages();
    for (function, code, script) in SHOWN {
        let codes = raised(script);
        assert!(
            codes.contains(code),
            "`{script}` does not raise {code}; it raises {codes:?}"
        );
        let page = &pages[function];
        let undeclared: Vec<&String> = codes.difference(page).collect();
        assert!(
            undeclared.is_empty(),
            "`{script}` raises {undeclared:?}, which the page of `{function}` does not list"
        );
    }
}

#[test]
fn the_pages_of_the_fourteen() {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let mut names: Vec<&(String, String)> = zeff_bodies().keys().collect();
    names.sort();
    assert_eq!(names.len(), 14);
    let mut all = String::new();
    for (package, name) in names {
        all.push_str(&format!("{package}::{name}\n"));
        let function = registry.function(package, name).expect("registered");
        for decl in &function.spec.diagnostics {
            let code = if decl.code.is_empty() {
                "Note"
            } else {
                decl.code
            };
            all.push_str(&format!("    {code}   {}\n", decl.when));
        }
    }
    insta::assert_snapshot!("warnings_and_notes_of_the_fourteen", all);
}
