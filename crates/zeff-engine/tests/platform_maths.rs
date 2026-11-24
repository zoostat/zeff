//! No elementary function of the platform on a path that bears a result (Engine and Project
//! Specification, section 8.3, point 3; release 0.9.0, step 42a).
//!
//! The engine has one pure-Rust implementation of the elementary functions, in the numeric
//! kernel (`zeff_numeric::maths`, `zeff_numeric::pow`, `zeff_numeric::powi`). A method of the
//! standard library on a float, `x.ln()`, `x.powf(y)`, `x.exp()`, calls the platform's C
//! library instead, and `powi` leaves its precision unspecified; either may give another last
//! bit on another target. At this step four `powf` were found in `derive` and `test`, two `ln`
//! in Bartlett's test and eight `powi`; this test reads the sources so that none comes back.
//!
//! What is read: the sources of the packages, of the kernel crates and of the engine's own
//! crates, outside their test modules, comments and text. The square root is not on the list:
//! IEEE 754 fixes its every bit.

use std::path::{Path, PathBuf};

/// The methods of a float that go to the platform, or whose precision is unspecified.
const METHODS: [&str; 26] = [
    "ln", "ln_1p", "log", "log2", "log10", "exp", "exp2", "exp_m1", "powf", "powi", "sin", "cos",
    "tan", "sin_cos", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh", "asinh", "acosh",
    "atanh", "cbrt", "hypot",
];

fn sources(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

/// A line of code without its text literals and its comment.
fn code(line: &str) -> String {
    let mut out = String::new();
    let mut in_text = false;
    let mut previous = ' ';
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_text {
            if c == '"' && previous != '\\' {
                in_text = false;
            }
            previous = if previous == '\\' && c == '\\' {
                ' '
            } else {
                c
            };
            continue;
        }
        if c == '"' {
            in_text = true;
            previous = c;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        out.push(c);
        previous = c;
    }
    out
}

/// The calls of a listed method in a line of code: `.name(` after something that is not a path.
fn calls(line: &str) -> Vec<&'static str> {
    METHODS
        .iter()
        .copied()
        .filter(|name| line.contains(&format!(".{name}(")))
        .collect()
}

#[test]
fn no_elementary_function_of_the_platform_bears_a_result() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut folders: Vec<PathBuf> = Vec::new();
    for group in [
        "kernel",
        "packages/tier0",
        "packages/tier1",
        "packages/tier2",
    ] {
        let mut crates: Vec<PathBuf> = std::fs::read_dir(root.join(group))
            .expect("a folder of crates")
            .flatten()
            .map(|entry| entry.path().join("src"))
            .collect();
        crates.sort();
        folders.extend(crates);
    }
    for name in [
        "zeff-engine",
        "zeff-ir",
        "zeff-syntax",
        "zeff-cli",
        "zeff-lsp",
        "zeff-web",
    ] {
        folders.push(root.join("crates").join(name).join("src"));
    }
    let mut files = Vec::new();
    for folder in &folders {
        sources(folder, &mut files);
    }
    assert!(files.len() > 200, "only {} sources were found", files.len());

    let mut found = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a source");
        let lines: Vec<&str> = text.lines().collect();
        for (number, line) in lines.iter().enumerate() {
            // A file's tests stand at its end, in a module of their own; a single item kept
            // for the tests elsewhere in the file is read like the rest.
            let module_follows = lines[number + 1..]
                .iter()
                .find(|next| !next.trim().is_empty())
                .is_some_and(|next| next.trim_start().starts_with("mod "));
            if line.contains("#[cfg(test)]") && module_follows {
                break;
            }
            for name in calls(&code(line)) {
                found.push(format!(
                    "{}:{}: `.{name}(`",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    number + 1
                ));
            }
        }
    }
    assert!(
        found.is_empty(),
        "the platform's elementary functions on a result-bearing path; use `zeff_numeric::maths`, `zeff_numeric::pow` or `zeff_numeric::powi`:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_reader_sees_a_call_and_passes_over_text_and_comments() {
    assert_eq!(calls(&code("let y = x.ln() + 1.0;")), vec!["ln"]);
    assert_eq!(calls(&code("let y = a.value.powf(b.value);")), vec!["powf"]);
    assert_eq!(calls(&code("let y = 10f64.powi(-digits);")), vec!["powi"]);
    assert!(calls(&code("let y = maths::log(x) + zeff_numeric::pow(x, 2.0);")).is_empty());
    assert!(calls(&code("// the old form was x.ln()")).is_empty());
    assert!(calls(&code("let s = \"w.log().is_missing()\";")).is_empty());
    assert!(calls(&code("let s = \"a \\\" b\"; let y = x.sqrt();")).is_empty());
    assert_eq!(
        calls(&code("let s = \"text\"; let y = x.exp();")),
        vec!["exp"]
    );
}
