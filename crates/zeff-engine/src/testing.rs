//! The tests of a project (`zeff test`; Engine and Project Specification, sections 2.1 and
//! 9.1; Language Reference, sections 5.3 and 5.4): every `assert` of its scripts, and every
//! `/// >` example of the functions of `functions/`.
//!
//! A script is a test when it calls `assert`. It is run as `zeff run` runs it, with one
//! difference: a failed `assert` is kept and the script carries on, so that every check of the
//! script is reported and not the first that fails alone. Any other error stops the script, and
//! is its failure. What a script prints is no part of the report.
//!
//! The examples of one function run as one script, in the project, after `use functions::` of
//! its module, in the order written, because a later example uses the bindings of an earlier
//! one; each is compared with what its own statements print, their warnings and notes included
//! (the rule of the engine's own pages, `crate::examples`).

use crate::fs::FileSystem;
use crate::project::Project;
use crate::registry::{Bodies, ZeffBodies};
use crate::run::{RunOptions, run_script_in};
use zeff_registry::Registry;

/// A script of the project that calls `assert`, run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTest {
    /// The script, by its path from the project's root.
    pub file: String,
    /// How many times an `assert` was evaluated.
    pub checks: usize,
    /// The checks that failed, each as the error a run would have stopped with.
    pub failed: Vec<String>,
    /// The messages of the error that stopped the script, where one did: an error of the
    /// script, an internal error, or a warning that `--deny-warnings` refused.
    pub stopped: Option<String>,
    /// The exit code of the run: 0 also when a check failed, since the script went on.
    pub exit_code: i32,
}

impl ScriptTest {
    pub fn passed(&self) -> bool {
        self.failed.is_empty() && self.stopped.is_none()
    }
}

/// An example of a function of `functions/`, run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleTest {
    /// The module's file, by its path from the project's root.
    pub file: String,
    pub function: String,
    /// Which example of the function's page, from 1.
    pub example: usize,
    /// The line of the file on which the example begins.
    pub line: u32,
    /// Its statements, without the `> `.
    pub code: Vec<String>,
    /// What the page says it prints.
    pub expected: Vec<String>,
    /// What it printed.
    pub found: Vec<String>,
}

impl ExampleTest {
    pub fn passed(&self) -> bool {
        self.expected == self.found
    }
}

/// The tests of a project, run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTests {
    pub scripts: Vec<ScriptTest>,
    pub examples: Vec<ExampleTest>,
    /// The scripts that call no `assert`: they hold no check and were not run.
    pub without_checks: Vec<String>,
    /// What could not be read: a folder or a file of the project, with the reason.
    pub problems: Vec<String>,
}

impl ProjectTests {
    /// How many checks and examples were evaluated, and how many of them failed; a script
    /// that stopped counts as one failure more.
    pub fn counts(&self) -> (usize, usize) {
        let checks: usize = self.scripts.iter().map(|s| s.checks).sum();
        let failed_checks: usize = self.scripts.iter().map(|s| s.failed.len()).sum();
        let stopped = self.scripts.iter().filter(|s| s.stopped.is_some()).count();
        let failed_examples = self.examples.iter().filter(|e| !e.passed()).count();
        (
            checks + self.examples.len() + stopped,
            failed_checks + failed_examples + stopped,
        )
    }

    /// The exit code of `zeff test` (Language Reference, section 3.8): 2 when a script met an
    /// internal error, 1 when a check or an example failed, a script stopped on an error or a
    /// file could not be read, 3 when the only failures are warnings that `--deny-warnings`
    /// refused, and 0 otherwise.
    pub fn exit_code(&self) -> i32 {
        let internal = self.scripts.iter().any(|s| s.exit_code == 2)
            || self
                .examples
                .iter()
                .any(|e| !e.passed() && e.found.iter().any(|l| l.starts_with("error[Z9")));
        let failed = !self.problems.is_empty()
            || self.examples.iter().any(|e| !e.passed())
            || self
                .scripts
                .iter()
                .any(|s| !s.failed.is_empty() || s.exit_code == 1);
        let refused = self.scripts.iter().any(|s| s.exit_code == 3);
        if internal {
            2
        } else if failed {
            1
        } else if refused {
            3
        } else {
            0
        }
    }
}

/// Whether the text of a script calls `assert`: the name, then an opening parenthesis.
pub fn calls_assert(file: &str, text: &str) -> bool {
    use zeff_syntax::TokenKind;
    let tokens = zeff_syntax::lex(file, text).tokens;
    tokens.windows(2).any(|pair| {
        matches!(&pair[0].kind, TokenKind::Name(name) if name == "assert")
            && pair[1].kind == TokenKind::LParen
            && &text[pair[0].span.start..pair[0].span.end] == "assert"
    })
}

/// Run the tests of a project with the engine's own functions.
pub fn test_project(fs: &dyn FileSystem, project: &Project, options: &RunOptions) -> ProjectTests {
    match crate::registry::registry() {
        Ok(registry) => test_project_in(
            fs,
            project,
            options,
            registry,
            crate::registry::bodies(),
            crate::registry::zeff_bodies(),
        ),
        Err(errors) => ProjectTests {
            problems: errors.iter().map(ToString::to_string).collect(),
            ..ProjectTests::default()
        },
    }
}

/// [`test_project`] against a given registry and its bodies.
pub fn test_project_in(
    fs: &dyn FileSystem,
    project: &Project,
    options: &RunOptions,
    registry: &Registry,
    bodies: &Bodies,
    zeff: &ZeffBodies,
) -> ProjectTests {
    let mut tests = ProjectTests::default();
    // The examples of the functions of `functions/`, module by module, in the order of
    // their names; within a module in the order written.
    let loaded = crate::modules::load(fs, project);
    for module in &loaded.modules {
        let source = zeff_diagnostics::Source::new(module.file.clone(), module.text.clone());
        for item in &module.script.items {
            let zeff_syntax::ast::Item::Fn(def) = item else {
                continue;
            };
            let name = module.text[def.name.span.start..def.name.span.end].to_string();
            let doc: Vec<String> = def.doc.iter().map(|line| line.text.clone()).collect();
            let examples = zeff_registry::doc_examples(&doc);
            // A function private to its file (section 3.6) is reached by no script, and so
            // by no example.
            if examples.is_empty() || name.starts_with('_') {
                continue;
            }
            // Where each example begins in the file: its first `>` line.
            let starts: Vec<u32> = def
                .doc
                .iter()
                .filter(|line| line.text.trim_start().starts_with('>'))
                .map(|line| source.line_of(line.span.start))
                .collect();
            let mut first = 0;
            let prelude = [format!("use functions::{}", module.name)];
            let run = RunOptions {
                deny_warnings: false,
                collect_checks: false,
                ..options.clone()
            };
            let found = crate::examples::run_page(
                fs, project, &prelude, &examples, &run, registry, bodies, zeff,
            );
            for (at, (example, found)) in examples.iter().zip(found).enumerate() {
                tests.examples.push(ExampleTest {
                    file: module.file.clone(),
                    function: name.clone(),
                    example: at + 1,
                    line: starts.get(first).copied().unwrap_or(0),
                    code: example.code.clone(),
                    expected: example
                        .output
                        .iter()
                        .map(|line| line.trim_end().to_string())
                        .collect(),
                    found,
                });
                first += example.code.len();
            }
        }
    }
    // The scripts, in the order of their paths.
    let scripts = match crate::project::script_files(fs, project, &["scripts"]) {
        Ok(scripts) => scripts,
        Err(error) => {
            tests
                .problems
                .push(format!("cannot list the scripts of the project: {error}"));
            Vec::new()
        }
    };
    for file in scripts {
        let text = match fs.read_to_string(&project.root.join(&file)) {
            Ok(text) => text,
            Err(error) => {
                tests.problems.push(format!("cannot read {file}: {error}"));
                continue;
            }
        };
        if !calls_assert(&file, &text) {
            tests.without_checks.push(file);
            continue;
        }
        let run = RunOptions {
            collect_checks: true,
            ..options.clone()
        };
        let outcome = run_script_in(fs, project, &file, &text, &run, registry, bodies, zeff);
        tests.scripts.push(ScriptTest {
            file,
            checks: outcome.output.checks.len(),
            failed: outcome
                .output
                .checks
                .iter()
                .filter_map(|check| check.failure.clone())
                .collect(),
            stopped: (outcome.exit_code != 0).then(|| outcome.output.stderr.clone()),
            exit_code: outcome.exit_code,
        });
    }
    tests
}
