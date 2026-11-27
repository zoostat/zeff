//! `cargo run -p xtask -- routine`: the routine of a step, whole and in one command (testing
//! handbook, section 2.1; release 0.10.0, step 2). Format, lint, every test of the workspace,
//! the format of the sample projects' scripts, and the tier, message and grammar checks, one
//! after another, each timed. Every part runs even when an earlier one fails, every failing
//! test program is named at the end, and the tests are counted, so that a routine cut short at
//! its first failure, a check left out or a test that went missing shows in what is printed.
//!
//! The parts are the commands of the continuous-integration workflow, so that green here means
//! what green means there, on this machine's one target.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// What the test programs of a run said, read from the lines the test harness prints.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Test programs that reported a result, documentation tests among them.
    pub programs: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    /// The words Cargo gives for running a failed program again, one per failed program.
    pub failed_programs: Vec<String>,
    /// The tests that failed, by name.
    pub failed_tests: Vec<String>,
}

/// The number that stands before `word` in a result line, as the 23 of "23 passed".
fn count_before(line: &str, word: &str) -> Option<usize> {
    let at = line.find(word)?;
    line[..at].split_whitespace().next_back()?.parse().ok()
}

/// Reads the output of `cargo test` into a tally. A result line that cannot be read is counted
/// as a failed program, so that a change in what the harness prints cannot pass for green.
pub fn tally(output: &str) -> Tally {
    let mut found = Tally::default();
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("test result:") {
            found.programs += 1;
            let counts = (
                count_before(rest, " passed"),
                count_before(rest, " failed"),
                count_before(rest, " ignored"),
            );
            match counts {
                (Some(passed), Some(failed), Some(ignored)) => {
                    found.passed += passed;
                    found.failed += failed;
                    found.ignored += ignored;
                }
                _ => found
                    .failed_programs
                    .push(format!("a result line that could not be read: {line}")),
            }
        } else if let Some(rest) = line.strip_prefix("error: test failed, to rerun pass ") {
            found
                .failed_programs
                .push(rest.trim_matches('`').to_string());
        } else if let Some(rest) = line.strip_prefix("test ")
            && let Some(name) = rest.strip_suffix(" ... FAILED")
        {
            found.failed_tests.push(name.to_string());
        }
    }
    found
}

/// One part of the routine as it ended.
struct Part {
    name: &'static str,
    passed: bool,
    seconds: f64,
    said: String,
}

/// Runs one part, says how it ended and how long it took, and keeps the record.
fn timed(name: &'static str, parts: &mut Vec<Part>, body: impl FnOnce() -> (bool, String)) {
    println!("{name} ...");
    let started = Instant::now();
    let (passed, said) = body();
    let seconds = started.elapsed().as_secs_f64();
    let detail = if said.is_empty() {
        String::new()
    } else {
        format!(": {said}")
    };
    let verdict = if passed { "passed" } else { "FAILED" };
    println!("  {verdict} in {seconds:.0} s{detail}");
    parts.push(Part {
        name,
        passed,
        seconds,
        said,
    });
}

/// The root of the workspace: the folder two above this crate's.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace's folder")
}

/// Runs a command with what it prints kept in `log`, and says whether it succeeded. Both
/// streams go through one handle of the file, so that neither writes over the other: the
/// harness prints its results on one and Cargo names the failed programs on the other.
fn run(command: &mut Command, log: &Path) -> bool {
    let Ok(out) = File::create(log) else {
        eprintln!("cannot write {}", log.display());
        return false;
    };
    let Ok(err) = out.try_clone() else {
        eprintln!("cannot write {}", log.display());
        return false;
    };
    command
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .status()
        .is_ok_and(|status| status.success())
}

/// The last lines of a log, for a part that failed.
fn last_lines(log: &Path, lines: usize) -> String {
    let text = std::fs::read_to_string(log).unwrap_or_default();
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

/// How many entries the debug build folder holds: above ten thousand, every compiler start
/// spends its time reading the folder (`docs/decisions.md`, 10 November 2026).
fn build_folder_entries(root: &Path) -> usize {
    std::fs::read_dir(root.join("target/debug/deps")).map_or(0, |entries| entries.count())
}

pub fn routine() -> ExitCode {
    let root = root();
    let logs = root.join("target/routine");
    if std::fs::create_dir_all(&logs).is_err() {
        eprintln!("cannot make {}", logs.display());
        return ExitCode::FAILURE;
    }
    // A machine that sleeps stalls the run and makes its times mean nothing. On a Mac the
    // system's own tool keeps it awake for as long as this process lives; elsewhere, and if
    // the tool is missing, the routine runs as it is.
    if cfg!(target_os = "macos") {
        let _ = Command::new("caffeinate")
            .args(["-i", "-w", &std::process::id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    let entries = build_folder_entries(&root);
    println!("the debug build folder holds {entries} entries");
    if entries > 10_000 {
        println!(
            "  that is many: `cargo clean --profile dev` before the next routine keeps the compiler quick"
        );
    }

    let cargo = |arguments: &[&str]| {
        let mut command = Command::new(env!("CARGO"));
        command.current_dir(&root).args(arguments);
        command
    };
    let own = |subcommand: &str| {
        let mut command = Command::new(std::env::current_exe().expect("this program's path"));
        command.current_dir(&root).arg(subcommand);
        command
    };
    let mut parts: Vec<Part> = Vec::new();
    let mut tests = Tally::default();
    timed("format", &mut parts, || {
        let log = logs.join("format.log");
        (
            run(&mut cargo(&["fmt", "--all", "--check"]), &log),
            String::new(),
        )
    });
    timed("lint, warnings denied", &mut parts, || {
        let log = logs.join("lint.log");
        let passed = run(
            &mut cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ]),
            &log,
        );
        (passed, String::new())
    });
    timed("every test", &mut parts, || {
        let log = logs.join("test.log");
        let passed = run(
            &mut cargo(&["test", "--workspace", "--locked", "--no-fail-fast"]),
            &log,
        );
        tests = tally(&std::fs::read_to_string(&log).unwrap_or_default());
        let clean = passed && tests.failed == 0 && tests.failed_programs.is_empty();
        (
            clean && tests.programs > 0,
            format!(
                "{} tests passed, {} failed, {} ignored, in {} test programs",
                tests.passed, tests.failed, tests.ignored, tests.programs
            ),
        )
    });
    timed(
        "the sample projects' scripts are formatted",
        &mut parts,
        || {
            let log = logs.join("samples.log");
            if !run(&mut cargo(&["build", "--locked", "-p", "zeff-cli"]), &log) {
                return (false, "the command line did not build".to_string());
            }
            let Ok(entries) = std::fs::read_dir(root.join("samples")) else {
                return (false, "no folder of samples".to_string());
            };
            let mut projects: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.join("zeff.toml").is_file())
                .collect();
            projects.sort();
            let mut unformatted = Vec::new();
            for project in &projects {
                let mut command = Command::new(root.join("target/debug/zeff"));
                command
                    .current_dir(&root)
                    .args(["fmt", "--check", "--project"])
                    .arg(project);
                if !run(&mut command, &logs.join("samples-one.log")) {
                    unformatted.push(
                        project
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                    );
                }
            }
            (
                unformatted.is_empty() && !projects.is_empty(),
                if unformatted.is_empty() {
                    format!("{} projects", projects.len())
                } else {
                    format!("not formatted: {}", unformatted.join(", "))
                },
            )
        },
    );
    for (name, subcommand, file) in [
        ("the tier rules", "check-tiers", "tiers.log"),
        (
            "a snapshot for every message",
            "check-messages",
            "messages.log",
        ),
        (
            "the grammar reads every script",
            "check-grammar",
            "grammar.log",
        ),
    ] {
        timed(name, &mut parts, || {
            let log = logs.join(file);
            let passed = run(&mut own(subcommand), &log);
            (passed, last_lines(&log, 1))
        });
    }

    println!();
    let total: f64 = parts.iter().map(|part| part.seconds).sum();
    let failed: Vec<&Part> = parts.iter().filter(|part| !part.passed).collect();
    if failed.is_empty() {
        println!(
            "the routine is green: {} parts in {:.1} minutes; {} tests passed, {} ignored, in {} test programs",
            parts.len(),
            total / 60.0,
            tests.passed,
            tests.ignored,
            tests.programs
        );
        return ExitCode::SUCCESS;
    }
    println!(
        "the routine is NOT green: {} of {} parts failed, in {:.1} minutes",
        failed.len(),
        parts.len(),
        total / 60.0
    );
    for part in failed {
        if part.said.is_empty() {
            println!("  {}", part.name);
        } else {
            println!("  {}: {}", part.name, part.said);
        }
    }
    for test in &tests.failed_tests {
        println!("  failed test: {test}");
    }
    for program in &tests.failed_programs {
        println!("  failed program, run again with: cargo test {program}");
    }
    println!("  what each part printed is in {}", logs.display());
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::{Tally, tally};

    #[test]
    fn the_result_lines_of_every_program_are_added_up() {
        let output = "\
     Running tests/a.rs (target/debug/deps/a-0123456789abcdef)

running 3 tests
test one ... ok
test two ... ok
test three ... ignored

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.08s

   Doc-tests zeff_numeric

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
        assert_eq!(
            tally(output),
            Tally {
                programs: 2,
                passed: 2,
                failed: 0,
                ignored: 1,
                failed_programs: vec![],
                failed_tests: vec![],
            }
        );
    }

    #[test]
    fn a_failed_test_and_its_program_are_named() {
        let output = "\
test the_list_is_empty ... FAILED
test another ... ok

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.19s

error: test failed, to rerun pass `-p zeff-engine --test help_contract`
";
        let found = tally(output);
        assert_eq!((found.programs, found.passed, found.failed), (1, 1, 1));
        assert_eq!(found.failed_tests, ["the_list_is_empty"]);
        assert_eq!(
            found.failed_programs,
            ["-p zeff-engine --test help_contract"]
        );
    }

    /// A line the reader does not understand must not pass for a green program.
    #[test]
    fn a_result_line_in_another_form_counts_as_a_failure() {
        let found = tally("test result: ok. many passed; none failed\n");
        assert_eq!(found.programs, 1);
        assert_eq!(found.passed, 0);
        assert_eq!(found.failed_programs.len(), 1);
    }

    /// No output at all, as when the build fails before any test runs, is no test program.
    #[test]
    fn a_build_that_failed_has_no_program() {
        assert_eq!(
            tally("error: could not compile `zeff-engine`\n"),
            Tally::default()
        );
    }
}
