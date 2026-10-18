//! The example runner (Engine and Project Specification, section 9.1, "Examples"): every example
//! of every registered function is run, and what it prints is compared with what its page says,
//! so that the help can never go stale.
//!
//! The examples of one page run as one script, in order, because later examples use the bindings
//! of earlier ones; each example is compared with the output of its own statements, their
//! warnings and notes included, in the order printed. They run in a fixture project that holds the
//! small files the pages read (`data/wages.csv`); the `zeff:` data root of the shipped datasets
//! arrives with release 0.2.0.

use crate::fs::MemoryFileSystem;
use crate::project::Project;
use crate::run::run_script;
use std::fmt;
use std::path::Path;
use zeff_registry::Registry;

/// The files every example may read, as `data/<name>` in the fixture project: the small wages
/// file, and the same kind of file as a German spreadsheet writes it (`;` between fields, a
/// decimal comma, `.` between groups of digits, Windows-1252 for the accented letters).
pub const FIXTURE_FILES: [(&str, &[u8]); 2] = [
    (
        "wages.csv",
        b"id,wage,hours\n1,23.4,40\n2,19.0,35\n3,31.2,\n",
    ),
    (
        "wages_de.csv",
        b"id;name;wage;hours\n1;J\xf6rg;1.234,5;40\n2;Aim\xe9e;987,25;35\n",
    ),
];

/// The fixture project, in memory.
pub fn fixture_project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/examples/zeff.toml",
        format!(
            "[project]\nname = \"examples\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            crate::VERSION
        ),
    );
    for (name, text) in FIXTURE_FILES {
        fs.add_file(format!("/examples/data/{name}"), text);
    }
    let project = Project::open(&fs, Path::new("/examples")).expect("the fixture project opens");
    (fs, project)
}

/// An example that printed something other than its page says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleFailure {
    pub package: String,
    pub function: String,
    /// 1-based, within the page.
    pub example: usize,
    pub code: Vec<String>,
    pub expected: Vec<String>,
    pub found: Vec<String>,
}

impl fmt::Display for ExampleFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "example {} of {}::{} prints something else than its page says",
            self.example, self.package, self.function
        )?;
        for line in &self.code {
            writeln!(f, "    > {line}")?;
        }
        writeln!(f, "  the page says:")?;
        for line in &self.expected {
            writeln!(f, "    {line}")?;
        }
        writeln!(f, "  it printed:")?;
        for line in &self.found {
            writeln!(f, "    {line}")?;
        }
        Ok(())
    }
}

/// Run every example of every function in `registry`; the failures, if any.
pub fn run_examples(registry: &Registry) -> Vec<ExampleFailure> {
    let (fs, project) = fixture_project();
    let mut failures = Vec::new();
    for function in registry.functions() {
        // One script per page; remember which lines belong to which example.
        let mut script = String::new();
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        let mut line = 1u32;
        for example in &function.doc.examples {
            let start = line;
            for code in &example.code {
                script.push_str(code);
                script.push('\n');
                line += 1;
            }
            ranges.push((start, line - 1));
        }
        let outcome = run_script(&fs, &project, "examples.zeff", &script);
        // An error found before anything ran (a check error) belongs to no statement; it is shown
        // as the output of the first example that produced nothing.
        let no_statement_failed = outcome
            .output
            .statements
            .iter()
            .all(|s| s.stderr.is_empty());
        let mut unplaced_error =
            (outcome.exit_code != 0 && no_statement_failed).then(|| outcome.output.stderr.clone());
        for (i, (example, (start, end))) in function.doc.examples.iter().zip(&ranges).enumerate() {
            let mut found = String::new();
            for statement in outcome
                .output
                .statements
                .iter()
                .filter(|s| (*start..=*end).contains(&s.line))
            {
                found.push_str(&statement.stdout);
                found.push_str(&statement.stderr);
            }
            if found.is_empty()
                && let Some(error) = unplaced_error.take()
            {
                found = error;
            }
            let found_lines: Vec<String> =
                found.lines().map(|l| l.trim_end().to_string()).collect();
            let expected_lines: Vec<String> = example
                .output
                .iter()
                .map(|l| l.trim_end().to_string())
                .collect();
            if found_lines != expected_lines {
                failures.push(ExampleFailure {
                    package: function.spec.package.to_string(),
                    function: function.spec.name.to_string(),
                    example: i + 1,
                    code: example.code.clone(),
                    expected: expected_lines,
                    found: found_lines,
                });
            }
        }
        // An error that stopped the script before an example ran shows as that example's output;
        // an error in a statement the ranges do not cover (a page without examples) shows here.
        if outcome.exit_code != 0 && function.doc.examples.is_empty() {
            failures.push(ExampleFailure {
                package: function.spec.package.to_string(),
                function: function.spec.name.to_string(),
                example: 0,
                code: Vec::new(),
                expected: Vec::new(),
                found: outcome.output.stderr.lines().map(str::to_string).collect(),
            });
        }
    }
    failures
}
