//! The command line (Engine and Project Specification, section 2, "Command line").
//!
//! What a script prints goes to standard output; messages go to standard error. The exit codes are
//! those of Language Reference section 3.8: 0 success, 1 a mistake in the project, the script or
//! the command, 2 an internal error, 4 an engine, edition or lock that does not match the
//! project. A mistake in how a command was typed is a 1 as well: the user's to fix, not Zeff's.

use std::io::Write;
use std::path::Path;
use zeff_diagnostics::{Diagnostic, Source, Sources};
use zeff_engine::VERSION;
use zeff_engine::create::{NewError, create_project};
use zeff_engine::fs::FileSystem;
use zeff_engine::project::{MANIFEST, OpenError, Project, exit_code, not_in_project};

const USAGE: &str = "\
Zeff, a language for statistics

Usage: zeff <command>

Commands:
  new <name>    create a project folder named <name> in the current folder
  version       print the version of Zeff
  help          show this text
";

/// Commands the specification names that this release does not have yet.
const LATER: [&str; 17] = [
    "run",
    "check",
    "fmt",
    "test",
    "lsp",
    "repl",
    "eval",
    "selftest",
    "engine",
    "runs",
    "export",
    "verify",
    "doctor",
    "app",
    "self",
    "completions",
    "dataset",
];

/// Run one command line. `args` are the words after `zeff`; `cwd` is the current folder.
/// Returns the exit code.
pub fn run(
    args: &[String],
    cwd: &Path,
    fs: &dyn FileSystem,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        [] | ["help" | "--help" | "-h"] => {
            let _ = out.write_all(USAGE.as_bytes());
            0
        }
        ["version" | "--version" | "-V"] => {
            let _ = writeln!(out, "zeff {VERSION}");
            0
        }
        ["new", name] => new_project(fs, cwd, name, out, err),
        ["new", ..] => usage_error(err, "`zeff new` takes one name, as in `zeff new schooling`"),
        ["version" | "--version" | "-V", ..] => {
            usage_error(err, "`zeff version` takes no arguments")
        }
        ["help", name, ..] => usage_error(
            err,
            &format!("there is no help page for `{name}` yet; `zeff help` lists the commands"),
        ),
        [command @ ("run" | "check" | "fmt" | "test"), ..] => {
            needs_a_project(fs, cwd, command, err)
        }
        [command, ..] if LATER.contains(command) => usage_error(
            err,
            &format!("`zeff {command}` is not available in this release yet"),
        ),
        [command, ..] => usage_error(err, &format!("unknown command `{command}`")),
    }
}

fn usage_error(err: &mut dyn Write, message: &str) -> i32 {
    let _ = writeln!(
        err,
        "error: {message}\n\nrun `zeff help` to see the commands"
    );
    1
}

fn show(err: &mut dyn Write, diagnostic: &Diagnostic, sources: &Sources) {
    let _ = err.write_all(diagnostic.render(sources).as_bytes());
}

fn new_project(
    fs: &dyn FileSystem,
    cwd: &Path,
    name: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    match create_project(fs, cwd, name) {
        Ok(root) => {
            let _ = writeln!(out, "created the project `{name}` in {}", root.display());
            let _ = writeln!(
                out,
                "  put your data in {name}/data/ and your analysis in {name}/scripts/main.zeff"
            );
            0
        }
        Err(NewError::BadName { name, advice }) => {
            let _ = writeln!(
                err,
                "error: `{name}` is not a good project name\n  help: {advice}"
            );
            1
        }
        Err(NewError::Exists(path)) => {
            let _ = writeln!(
                err,
                "error: {} already exists; `zeff new` does not change an existing folder",
                path.display()
            );
            1
        }
        Err(NewError::Io(message)) => {
            let _ = writeln!(err, "error: could not create the project: {message}");
            1
        }
    }
}

/// The start of every command that works on a project: find it, read it, check its pin.
fn open_project(fs: &dyn FileSystem, cwd: &Path, err: &mut dyn Write) -> Result<Project, i32> {
    let no_sources = Sources::new();
    match Project::open(fs, cwd) {
        Ok(project) => {
            if let Err(diagnostic) = project.check_pin() {
                let mut sources = Sources::new();
                sources.add(Source::new(MANIFEST, project.manifest_text.clone()));
                show(err, &diagnostic, &sources);
                return Err(exit_code(&[&diagnostic]));
            }
            Ok(project)
        }
        Err(OpenError::NotInProject) => {
            show(err, &not_in_project(), &no_sources);
            Err(1)
        }
        Err(OpenError::Unreadable { path, message }) => {
            let _ = writeln!(err, "error: cannot read {}: {message}", path.display());
            Err(1)
        }
        Err(OpenError::Invalid { text, diagnostics }) => {
            let mut sources = Sources::new();
            sources.add(Source::new(MANIFEST, text));
            let all: Vec<&Diagnostic> = diagnostics.iter().collect();
            for (i, diagnostic) in all.iter().enumerate() {
                if i > 0 {
                    let _ = writeln!(err);
                }
                show(err, diagnostic, &sources);
            }
            Err(exit_code(&all))
        }
    }
}

fn needs_a_project(fs: &dyn FileSystem, cwd: &Path, command: &str, err: &mut dyn Write) -> i32 {
    match open_project(fs, cwd, err) {
        Err(code) => code,
        Ok(_) => usage_error(
            err,
            &format!("`zeff {command}` is not available in this release yet"),
        ),
    }
}
