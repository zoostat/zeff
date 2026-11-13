//! The command line (Engine and Project Specification, section 2, "Command line").
//!
//! What a script prints goes to standard output; messages go to standard error. The exit codes are
//! those of Language Reference section 3.8: 0 success, 1 a mistake in the project, the script or
//! the command, 2 an internal error, 4 an engine, edition or lock that does not match the
//! project. A mistake in how a command was typed is a 1 as well: the user's to fix, not Zeff's.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use zeff_diagnostics::{Diagnostic, Severity, Source, Sources};
use zeff_engine::VERSION;
use zeff_engine::create::{NewError, create_project};
use zeff_engine::fs::FileSystem;
use zeff_engine::project::{MANIFEST, OpenError, Project, exit_code, not_in_project};
use zeff_engine::run::{Outcome, RunOptions, run_script_with};
use zeff_engine::session::{Opened, SESSION_FILE, Session, is_incomplete, split_statements};

const USAGE: &str = "\
Zeff, a language for statistics

Usage: zeff [options] <command>

Commands:
  new <name>        create a project folder named <name> in the current folder
  run [script]      run scripts/main.zeff, or the script named
  check [script]    check a script's names, arguments and types without running it
                    (scripts/main.zeff unless another is named)
  test              run the project's tests: every `assert` of its scripts and every example
                    of the functions in functions/
  fmt               write every script of the project in the one style of the language
                    (with --check: change nothing, and say which scripts would change)
  repl [script]     open the console: statements typed are kept in .zeff/session.zeff
                    (outside a project, in the scratch project; :help lists its commands)
  eval \"<statement>\"
                    run one statement in the scratch project and print its value
  help <name>       print a function's help page, list a package's functions, or
                    explain a code such as Z0101
  version           print the version of Zeff
  selftest          run the conformance cases shipped with the engine against their published
                    results and say whether this installation is conformant
  help              show this text

Options:
  -p, --project <dir>     the project to act on (default: the nearest folder with zeff.toml)
  --format text|json      for `check`, `test`, `version` and `selftest`: text (the default),
                          or JSON records for an editor
  --color auto|always|never
                          colour in messages; `auto` means a terminal, and NO_COLOR is honoured
  --ascii                 rules and symbols in ASCII, for fonts without them
  --width <n>             the width frames and tables wrap at (default 118, at least 40)
  --deny-warnings         for `run`, `check` and `test`: a warning that no
                          `// zeff: allow(CODE) reason` pragma allows is an error (exit code 3)
  --check                 for `fmt`: write nothing; exit code 1 if a script would change
  -q, --quiet             hide notes
  -v, --verbose           show timings and build details
";

/// The console's commands, for `:help`.
const CONSOLE_HELP: &str = "\
Type a statement to run it; it is kept in the session script and runs again with the next.
A line beginning with `.` adds a step to the statement before it. A bracket left open, or a
line ending with an operator, continues on the next line.

  :undo             drop the last statement
  :vars             list the session's bindings and their types
  :save <path>      write the session as a script (outside a project: create a project <name>)
  :load <path>      run a script's statements into the session
  :help <name>      a function's help page, a package's functions, or a code such as Z0101
  :help             this text
  :quit             leave the console (so does the end of the input)
";

/// Commands the specification names that this release does not have yet.
const LATER: [&str; 11] = [
    "lsp",
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

/// Options the specification names that this release does not have yet.
const LATER_OPTIONS: [&str; 11] = [
    "--frozen",
    "--install",
    "-y",
    "--yes",
    "--memory",
    "--threads",
    "--fresh",
    "--replay",
    "--no-external",
    "--refresh",
    "--offline",
];

/// What the command line finds around it: the Zeff folder, the terminal and the input.
pub struct Environment<'a> {
    /// `ZEFF_HOME`, or `~/.zeff`; `None` when neither is known.
    pub zeff_home: Option<PathBuf>,
    /// `NO_COLOR` is set.
    pub no_color: bool,
    /// Standard error is a terminal: colour by default.
    pub stderr_is_terminal: bool,
    /// Standard input is a terminal: the console shows prompts.
    pub stdin_is_terminal: bool,
    /// Standard input, for the console.
    pub input: &'a mut dyn BufRead,
    /// The handle by which a run is cancelled from outside: an interrupt of `zeff run` cancels
    /// it, and the run stops before its next step (Z0705).
    pub cancel: zeff_engine::control::Control,
}

/// Whether the command these words give is one an interrupt cancels between its steps, and
/// does not end at once: `zeff run`.
pub fn interruptible(args: &[String]) -> bool {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    take_options(&words).is_ok_and(|(words, _)| words.first() == Some(&"run"))
}

/// `--format text | json` (Engine and Project Specification, section 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Json,
}

/// The options of section 2.1 that this release has, as given.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Settings {
    format: Format,
    project: Option<String>,
    /// `--color`: `Some(true)` always, `Some(false)` never, `None` auto.
    color: Option<bool>,
    ascii: bool,
    width: usize,
    quiet: bool,
    verbose: bool,
    /// `--deny-warnings`: a warning that no pragma allows is an error (exit code 3).
    deny_warnings: bool,
    /// `--check`, of `zeff fmt`: nothing is written.
    check: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            format: Format::Text,
            project: None,
            color: None,
            ascii: false,
            width: RunOptions::default().width,
            quiet: false,
            verbose: false,
            deny_warnings: false,
            check: false,
        }
    }
}

/// The smallest `--width`: a frame's row numbers and a few columns need this much.
const MIN_WIDTH: usize = 40;

/// Take the options out of the words; what is left is the command and its arguments. The word
/// after `eval` is a statement and is never read as an option.
fn take_options<'a>(words: &[&'a str]) -> Result<(Vec<&'a str>, Settings), String> {
    let mut rest = Vec::with_capacity(words.len());
    let mut settings = Settings::default();
    let mut i = 0;
    let mut statement_next = false;
    while i < words.len() {
        let word = words[i];
        if statement_next || !word.starts_with('-') || word == "-" {
            rest.push(word);
            statement_next = word == "eval" && rest.len() == 1;
            i += 1;
            continue;
        }
        // `--name value` or `--name=value`.
        let (name, inline_value) = match word.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (word, None),
        };
        let mut value = |what: &str| -> Result<&'a str, String> {
            match inline_value {
                Some(value) => Ok(value),
                None => {
                    i += 1;
                    words
                        .get(i)
                        .copied()
                        .ok_or_else(|| format!("`{name}` needs a value: {what}"))
                }
            }
        };
        match name {
            "--format" => {
                settings.format = match value("`text` or `json`")? {
                    "text" => Format::Text,
                    "json" => Format::Json,
                    other => {
                        return Err(format!(
                            "`--format {other}` is not a format: write `--format text` or `--format json`"
                        ));
                    }
                }
            }
            "-p" | "--project" => settings.project = Some(value("a folder")?.to_string()),
            "--color" | "--colour" => {
                settings.color = match value("`auto`, `always` or `never`")? {
                    "auto" => None,
                    "always" => Some(true),
                    "never" => Some(false),
                    other => {
                        return Err(format!(
                            "`--color {other}` is not a choice: write `--color auto`, `--color always` or `--color never`"
                        ));
                    }
                }
            }
            "--width" => {
                let text = value("a number of characters")?;
                settings.width = match text.parse::<usize>() {
                    Ok(n) if n >= MIN_WIDTH => n,
                    _ => {
                        return Err(format!(
                            "`--width {text}` is not a width: write a whole number of at least {MIN_WIDTH}"
                        ));
                    }
                }
            }
            "--ascii" => settings.ascii = true,
            "-q" | "--quiet" => settings.quiet = true,
            "-v" | "--verbose" => settings.verbose = true,
            "--deny-warnings" => settings.deny_warnings = true,
            "--check" => settings.check = true,
            "--help" | "-h" | "--version" | "-V" => rest.push(word),
            later if LATER_OPTIONS.contains(&later) => {
                return Err(format!("`{later}` is not available in this release yet"));
            }
            unknown => return Err(format!("unknown option `{unknown}`")),
        }
        i += 1;
    }
    Ok((rest, settings))
}

/// `·`, `×`, `…`, `→` and `—` in ASCII (`--ascii`, Language Reference, section 3.8).
fn to_ascii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '·' => out.push('|'),
            '×' => out.push('x'),
            '…' => out.push_str("..."),
            '→' => out.push_str("->"),
            '—' => out.push('-'),
            c => out.push(c),
        }
    }
    out
}

/// A writer that puts the output in ASCII when asked, whole characters at a time.
struct Filtered<'a> {
    inner: &'a mut dyn Write,
    ascii: bool,
    carry: Vec<u8>,
}

impl<'a> Filtered<'a> {
    fn new(inner: &'a mut dyn Write, ascii: bool) -> Self {
        Filtered {
            inner,
            ascii,
            carry: Vec::new(),
        }
    }
}

impl Write for Filtered<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.ascii {
            return self.inner.write(buf);
        }
        self.carry.extend_from_slice(buf);
        let valid = match std::str::from_utf8(&self.carry) {
            Ok(_) => self.carry.len(),
            Err(e) => e.valid_up_to(),
        };
        let text = std::str::from_utf8(&self.carry[..valid]).unwrap_or_default();
        self.inner.write_all(to_ascii(text).as_bytes())?;
        self.carry.drain(..valid);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Drop for Filtered<'_> {
    fn drop(&mut self) {
        if !self.carry.is_empty() {
            let _ = self.inner.write_all(&self.carry);
        }
        let _ = self.inner.flush();
    }
}

/// Messages as they are written: coloured when colour is on (the word before the first colon of
/// an `error`, `warning` or `note` line), and without notes under `--quiet`.
#[derive(Debug, Clone, Copy)]
struct Messages {
    color: bool,
    quiet: bool,
}

impl Messages {
    fn text(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for line in text.split_inclusive('\n') {
            let is_note = line.starts_with("note:") || line.starts_with("note[");
            if self.quiet && is_note {
                continue;
            }
            if self.color {
                let paint = if line.starts_with("error") {
                    Some("1;31")
                } else if line.starts_with("warning") {
                    Some("1;33")
                } else if is_note {
                    Some("1;36")
                } else {
                    None
                };
                if let (Some(code), Some(colon)) = (paint, line.find(':')) {
                    out.push_str(&format!(
                        "\x1b[{code}m{}\x1b[0m{}",
                        &line[..colon],
                        &line[colon..]
                    ));
                    continue;
                }
            }
            out.push_str(line);
        }
        out
    }

    fn show(&self, err: &mut dyn Write, diagnostic: &Diagnostic, sources: &Sources) {
        if self.quiet && diagnostic.severity == Severity::Note {
            return;
        }
        let _ = err.write_all(self.text(&diagnostic.render(sources)).as_bytes());
    }
}

/// Everything a command needs besides its arguments.
struct Command<'a, 'b> {
    fs: &'a dyn FileSystem,
    cwd: PathBuf,
    env: &'a mut Environment<'b>,
    settings: Settings,
    messages: Messages,
}

impl Command<'_, '_> {
    fn run_options(&self) -> RunOptions {
        RunOptions {
            width: self.settings.width,
            deny_warnings: self.settings.deny_warnings,
            ..RunOptions::default()
        }
    }
}

/// Run one command line. `args` are the words after `zeff`; `cwd` is the current folder.
/// Returns the exit code.
pub fn run(
    args: &[String],
    cwd: &Path,
    fs: &dyn FileSystem,
    env: &mut Environment<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let (words, settings) = match take_options(&words) {
        Ok(taken) => taken,
        Err(message) => return usage_error(err, &message),
    };
    let mut out = Filtered::new(out, settings.ascii);
    let mut err = Filtered::new(err, settings.ascii);
    let color = settings
        .color
        .unwrap_or(env.stderr_is_terminal && !env.no_color);
    let messages = Messages {
        color,
        quiet: settings.quiet,
    };
    if settings.format == Format::Json
        && !matches!(
            words.first(),
            Some(&("check" | "test" | "version" | "--version" | "-V" | "selftest"))
        )
    {
        return usage_error(
            &mut err,
            "`--format json` applies to `zeff check`, `zeff test`, `zeff version` and `zeff selftest` in this release",
        );
    }
    if settings.deny_warnings && !matches!(words.first(), Some(&("run" | "check" | "test"))) {
        return usage_error(
            &mut err,
            "`--deny-warnings` applies to `zeff run`, `zeff check` and `zeff test` in this release",
        );
    }
    if settings.check && words.first() != Some(&"fmt") {
        return usage_error(&mut err, "`--check` applies to `zeff fmt`");
    }
    let cwd = match &settings.project {
        Some(dir) => {
            let dir = cwd.join(dir);
            if !fs.is_dir(&dir) {
                return usage_error(
                    &mut err,
                    &format!("`--project {}` is not a folder", dir.display()),
                );
            }
            dir
        }
        None => cwd.to_path_buf(),
    };
    let mut command = Command {
        fs,
        cwd,
        env,
        settings,
        messages,
    };
    match words.as_slice() {
        [] | ["help" | "--help" | "-h"] => {
            let _ = out.write_all(USAGE.as_bytes());
            0
        }
        ["version" | "--version" | "-V"] => version(&command, &mut out),
        ["new", name] => new_project(fs, &command.cwd, name, &mut out, &mut err),
        ["new", ..] => usage_error(
            &mut err,
            "`zeff new` takes one name, as in `zeff new schooling`",
        ),
        ["version" | "--version" | "-V", ..] => {
            usage_error(&mut err, "`zeff version` takes no arguments")
        }
        ["selftest"] => selftest_command(&command, &mut out),
        ["selftest", ..] => usage_error(&mut err, "`zeff selftest` takes no arguments"),
        ["help", name] => help_command(&command, name, &mut out, &mut err),
        ["help", ..] => usage_error(
            &mut err,
            "`zeff help` takes one name, as in `zeff help mean` or `zeff help data`",
        ),
        ["check"] => check_command(&command, None, &mut out, &mut err),
        ["check", script] => check_command(&command, Some(script), &mut out, &mut err),
        ["check", ..] => usage_error(
            &mut err,
            "`zeff check` takes at most one script, as in `zeff check scripts/clean.zeff`",
        ),
        ["run"] => run_command(&command, None, &mut out, &mut err),
        ["run", script] => run_command(&command, Some(script), &mut out, &mut err),
        ["run", ..] => usage_error(
            &mut err,
            "`zeff run` takes at most one script, as in `zeff run scripts/clean.zeff`",
        ),
        ["eval", statement] => eval_command(&command, statement, &mut out, &mut err),
        ["eval"] => usage_error(
            &mut err,
            "`zeff eval` takes one statement in quotes, as in `zeff eval \"2 + 2\"`",
        ),
        ["eval", ..] => usage_error(
            &mut err,
            "`zeff eval` takes one statement: put it in quotes, as in `zeff eval \"2 + 2\"`",
        ),
        ["repl"] => repl_command(&mut command, None, &mut out, &mut err),
        ["repl", script] => repl_command(&mut command, Some(script), &mut out, &mut err),
        ["repl", ..] => usage_error(
            &mut err,
            "`zeff repl` takes at most one script to load, as in `zeff repl scripts/clean.zeff`",
        ),
        ["fmt"] => fmt_command(&command, &mut err),
        ["fmt", ..] => usage_error(
            &mut err,
            "`zeff fmt` takes no script: it formats every script of the project",
        ),
        ["test"] => test_command(&command, &mut out, &mut err),
        ["test", ..] => usage_error(
            &mut err,
            "`zeff test` takes no script: it runs every test of the project",
        ),
        [word, ..] if LATER.contains(word) => usage_error(
            &mut err,
            &format!("`zeff {word}` is not available in this release yet"),
        ),
        [word, ..] => usage_error(&mut err, &format!("unknown command `{word}`")),
    }
}

fn usage_error(err: &mut dyn Write, message: &str) -> i32 {
    let _ = writeln!(
        err,
        "error: {message}\n\nrun `zeff help` to see the commands"
    );
    1
}

fn version(command: &Command<'_, '_>, out: &mut dyn Write) -> i32 {
    match command.settings.format {
        Format::Json => {
            let _ = writeln!(
                out,
                "{{\"version\": {}}}",
                zeff_diagnostics::json_string(VERSION)
            );
        }
        Format::Text => {
            let _ = writeln!(out, "zeff {VERSION}");
            if command.settings.verbose {
                let _ = writeln!(
                    out,
                    "target: {} {}\nbuild: source build (official, reproducible builds arrive with release 0.9.0)",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                );
            }
        }
    }
    0
}

/// `zeff selftest`: the conformance cases of this build against the published report (Engine and
/// Project Specification, section 2.1; section 8.3, point 3, "Proof"). Exit code 0 when every
/// case matches, 1 otherwise, with the differing cases named.
fn selftest_command(command: &Command<'_, '_>, out: &mut dyn Write) -> i32 {
    let cases = zeff_conformance::selftest();
    let total = cases.len();
    let differing: Vec<&zeff_conformance::SelftestCase> =
        cases.iter().filter(|c| !c.matches()).collect();
    let target = match std::env::consts::ARCH {
        "x86_64" => "x86-64",
        "aarch64" => "ARM64",
        "wasm32" => "WebAssembly",
        other => other,
    };
    match command.settings.format {
        Format::Json => {
            for case in &cases {
                let published = match case.published {
                    Some(d) => format!("\"{d:016x}\""),
                    None => "null".to_string(),
                };
                let _ = writeln!(
                    out,
                    "{{\"case\": {}, \"published\": {published}, \"actual\": \"{:016x}\", \"match\": {}}}",
                    zeff_diagnostics::json_string(&case.name),
                    case.actual,
                    case.matches()
                );
            }
        }
        Format::Text => {
            if differing.is_empty() {
                let _ = writeln!(
                    out,
                    "conformant: {total} of {total} cases match the published results of Zeff {VERSION} on this machine ({target})"
                );
            } else {
                let _ = writeln!(
                    out,
                    "not conformant: {} of {total} cases differ from the published results of Zeff {VERSION} on this machine ({target})",
                    differing.len()
                );
                for case in &differing {
                    let published = case
                        .published
                        .map_or("not in the published report".to_string(), |d| {
                            format!("published {d:016x}")
                        });
                    let _ = writeln!(
                        out,
                        "  {}: {published}, this build {:016x}",
                        case.name, case.actual
                    );
                }
                let _ = writeln!(
                    out,
                    "  this installation does not reproduce the published results; please report it with `zeff version --verbose`"
                );
            }
        }
    }
    i32::from(!differing.is_empty())
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
/// `Ok(None)` when the folder is not inside a project.
fn find_project(
    fs: &dyn FileSystem,
    cwd: &Path,
    messages: Messages,
    err: &mut dyn Write,
) -> Result<Option<Project>, i32> {
    match Project::open(fs, cwd) {
        Ok(project) => {
            if let Err(diagnostic) = project.check_pin() {
                let mut sources = Sources::new();
                sources.add(Source::new(MANIFEST, project.manifest_text.clone()));
                messages.show(err, &diagnostic, &sources);
                return Err(exit_code(&[&diagnostic]));
            }
            Ok(Some(project))
        }
        Err(OpenError::NotInProject) => Ok(None),
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
                messages.show(err, diagnostic, &sources);
            }
            Err(exit_code(&all))
        }
    }
}

/// The project a command needs; outside one, Z0709.
fn open_project(
    fs: &dyn FileSystem,
    cwd: &Path,
    messages: Messages,
    err: &mut dyn Write,
) -> Result<Project, i32> {
    match find_project(fs, cwd, messages, err)? {
        Some(project) => Ok(project),
        None => {
            messages.show(err, &not_in_project(), &Sources::new());
            Err(1)
        }
    }
}

/// A path problem as a message without a position (the path was typed on the command line).
fn path_diagnostic(problem: &zeff_engine::paths::PathProblem, written: &str) -> Diagnostic {
    let d = problem.diagnostic(written, zeff_diagnostics::Span::new(0, 0), "");
    Diagnostic {
        file: None,
        labels: Vec::new(),
        ..d
    }
}

/// The script a command works on: the project, the script's name as shown in messages, and its
/// text; or the exit code after the problem has been reported.
fn script_of(
    command: &Command<'_, '_>,
    script: Option<&str>,
    err: &mut dyn Write,
) -> Result<(Project, String, String), i32> {
    let fs = command.fs;
    let project = open_project(fs, &command.cwd, command.messages, err)?;
    let written = script.unwrap_or("scripts/main.zeff");
    let shown = written.trim_start_matches("./").to_string();
    let path = match project.resolve_read(fs, written) {
        Ok(path) => path,
        Err(problem) => {
            command
                .messages
                .show(err, &path_diagnostic(&problem, written), &Sources::new());
            return Err(1);
        }
    };
    if !fs.is_file(&path) {
        let d = Diagnostic::error("Z0714", format!("no script `{shown}` in the project")).help(
            "scripts live in `scripts/`; `zeff run` and `zeff check` alone use `scripts/main.zeff`",
        );
        command.messages.show(err, &d, &Sources::new());
        return Err(1);
    }
    match fs.read_to_string(&path) {
        Ok(text) => Ok((project, shown, text)),
        Err(error) => {
            let _ = writeln!(err, "error: cannot read {shown}: {error}");
            Err(1)
        }
    }
}

/// What a run wrote, to the two streams, with the messages filtered as the settings say.
/// `842 B`, `2.1 kB`, `3.0 MB`: decimal units, one decimal from a kilobyte, as the column
/// kernel prints sizes.
fn size_text(bytes: usize) -> String {
    if bytes < 1000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{:.1} kB", bytes as f64 / 1000.0)
    } else if bytes < 1_000_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    }
}

fn emit(command: &Command<'_, '_>, outcome: &Outcome, out: &mut dyn Write, err: &mut dyn Write) {
    let _ = out.write_all(outcome.output.stdout.as_bytes());
    let _ = err.write_all(command.messages.text(&outcome.output.stderr).as_bytes());
}

/// `zeff run [script]`: what the script prints goes to standard output, diagnostics to standard
/// error, and the exit code is that of Language Reference section 3.8.
fn run_command(
    command: &Command<'_, '_>,
    script: Option<&str>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let (project, shown, text) = match script_of(command, script, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let started = Instant::now();
    zeff_regress::reg::reset_fixed_effect_work();
    // What each statement prints and says is written as the statement ends, so that a long
    // run shows where it stands; the two streams hold what they held when written at the end.
    let outcome = {
        let streams = std::cell::RefCell::new((&mut *out, &mut *err));
        let listener = |event: &zeff_engine::control::Event| {
            if let zeff_engine::control::Event::Output { stdout, stderr } = event {
                let mut streams = streams.borrow_mut();
                let _ = streams.0.write_all(stdout.as_bytes());
                let _ = streams.0.flush();
                let _ = streams
                    .1
                    .write_all(command.messages.text(stderr).as_bytes());
                let _ = streams.1.flush();
            }
        };
        zeff_engine::run::run_script_under(
            command.fs,
            &project,
            &shown,
            &text,
            &command.run_options(),
            &command.env.cancel,
            Some(&listener),
        )
    };
    if command.settings.verbose {
        // The reused fixed-effect work (Engine and Project Specification, section 8.3, point
        // 10): the sets of group indexes built for `fe:` and the fits that reused one, with the
        // demeaned columns it carried.
        let (built, reused) = zeff_regress::reg::fixed_effect_work();
        if built + reused > 0 {
            let _ = writeln!(
                err,
                "fixed effects: {built} {} built, {reused} reused by later fits",
                if built == 1 {
                    "set of group indexes"
                } else {
                    "sets of group indexes"
                }
            );
        }
        // The memory record (Engine and Project Specification, section 8.3, point 7): what each
        // frame held when bound, its own buffers and the parent buffers it kept alive, and when
        // a binding was freed after its last use (Language Reference, section 6, "Memory").
        for note in &outcome.output.memory {
            match note {
                zeff_engine::interpreter::MemoryNote::Bound {
                    name,
                    line,
                    own,
                    kept,
                } => {
                    let _ = writeln!(
                        err,
                        "memory: `{name}` bound at line {line} holds {}{}",
                        size_text(*own),
                        if *kept > 0 {
                            format!(", keeping {} of a parent", size_text(*kept))
                        } else {
                            String::new()
                        }
                    );
                }
                zeff_engine::interpreter::MemoryNote::Freed {
                    name,
                    line,
                    gathered,
                } => {
                    let _ = writeln!(
                        err,
                        "memory: `{name}` freed after line {line}{}",
                        match gathered {
                            0 => String::new(),
                            1 => "; 1 column gathered from it".to_string(),
                            n => format!("; {n} columns gathered from it"),
                        }
                    );
                }
            }
        }
        // The warnings a pragma silenced (Language Reference, section 11.1), by statement.
        let mut silenced: Vec<(&zeff_engine::pragmas::Allowed, usize)> = Vec::new();
        for allowed in &outcome.output.allowed {
            match silenced.iter_mut().find(|(seen, _)| {
                (&seen.code, &seen.file, seen.line) == (&allowed.code, &allowed.file, allowed.line)
            }) {
                Some((_, count)) => *count += 1,
                None => silenced.push((allowed, 1)),
            }
        }
        for (allowed, count) in silenced {
            let place = if allowed.file == shown {
                format!("line {}", allowed.line)
            } else {
                format!("{}:{}", allowed.file, allowed.line)
            };
            let _ = writeln!(
                err,
                "allowed: {} at {place}, {count} {}: {}",
                allowed.code,
                if count == 1 { "warning" } else { "warnings" },
                allowed.reason
            );
        }
        let n = outcome.output.statements.len();
        let _ = writeln!(
            err,
            "ran {shown}: {n} {} in {} ms",
            if n == 1 { "statement" } else { "statements" },
            started.elapsed().as_millis()
        );
    }
    outcome.exit_code
}

/// `zeff check [script]`: parse and check a script against the registry, report every problem,
/// and exit 0 when there is none, 1 otherwise (Language Reference, section 3.8, "Exit codes").
/// With `--format json` the problems go to standard output as one JSON array of records, one per
/// line, and nothing is said on standard error; the exit code is the same. With
/// `--deny-warnings` a warning that no pragma allows is reported as an error, and the exit code
/// is 3 when such warnings are the only errors (section 11.1).
fn check_command(
    command: &Command<'_, '_>,
    script: Option<&str>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let (project, shown, text) = match script_of(command, script, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let registry = match zeff_engine::registry::registry() {
        Ok(registry) => registry,
        Err(errors) => return internal_error(err, errors),
    };
    let in_project =
        zeff_engine::run::check_in_project(command.fs, &project, &shown, &text, registry);
    let mut sources = Sources::new();
    sources.add(Source::new(shown.clone(), text));
    for module in in_project.module_sources {
        sources.add(module);
    }
    let deny = command.settings.deny_warnings;
    let (mut errors, mut warnings, mut refused) = (0usize, 0usize, 0usize);
    let diagnostics: Vec<Diagnostic> = in_project
        .checked
        .diagnostics
        .iter()
        .map(|diagnostic| {
            if deny && zeff_engine::pragmas::deniable(diagnostic) {
                refused += 1;
                zeff_engine::pragmas::denied(diagnostic)
            } else {
                diagnostic.clone()
            }
        })
        .collect();
    for diagnostic in &diagnostics {
        match diagnostic.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Note => {}
        }
    }
    // 1 for an error of the script; 3 when the only errors are warnings refused.
    let exit_code = match (errors, refused) {
        (0, _) => 0,
        (e, r) if e == r => 3,
        _ => 1,
    };
    if command.settings.format == Format::Json {
        let _ = writeln!(out, "[");
        let last = diagnostics.len().saturating_sub(1);
        for (i, diagnostic) in diagnostics.iter().enumerate() {
            let comma = if i < last { "," } else { "" };
            let _ = writeln!(out, "  {}{comma}", diagnostic.to_json(&sources));
        }
        let _ = writeln!(out, "]");
        return exit_code;
    }
    let mut first = true;
    for diagnostic in diagnostics.iter() {
        if command.settings.quiet && diagnostic.severity == Severity::Note {
            continue;
        }
        if !first {
            let _ = writeln!(err);
        }
        first = false;
        command.messages.show(err, diagnostic, &sources);
    }
    let plural = |n: usize, word: &str| {
        if n == 1 {
            format!("{n} {word}")
        } else {
            format!("{n} {word}s")
        }
    };
    if errors == 0 && warnings == 0 {
        let _ = writeln!(err, "checked {shown}: no problems");
    } else {
        if !first {
            let _ = writeln!(err);
        }
        let _ = writeln!(
            err,
            "{shown}: {}, {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        );
    }
    exit_code
}

/// `zeff fmt`: every script of the project written in the one style of the language (Language
/// Reference, section 3.5), and with `--check` nothing written and the scripts that would
/// change named. A script that does not parse is left as it is, with its errors shown. Exit
/// code 0; 1 when a script does not parse or cannot be read or written, and under `--check`
/// when a script would change; 2 when the formatter cannot vouch for what it wrote.
fn fmt_command(command: &Command<'_, '_>, err: &mut dyn Write) -> i32 {
    let fs = command.fs;
    let project = match open_project(fs, &command.cwd, command.messages, err) {
        Ok(project) => project,
        Err(code) => return code,
    };
    let check = command.settings.check;
    let scripts = match zeff_engine::project::script_files(fs, &project, &["functions", "scripts"])
    {
        Ok(scripts) => scripts,
        Err(error) => {
            let _ = writeln!(
                err,
                "error: cannot list the scripts of the project: {error}"
            );
            return 1;
        }
    };
    let count = |n: usize| {
        if n == 1 {
            "1 script".to_string()
        } else {
            format!("{n} scripts")
        }
    };
    if scripts.is_empty() {
        let _ = writeln!(
            err,
            "no scripts to format: the project has no `.zeff` file in `scripts/` or `functions/`"
        );
        return 0;
    }
    let (mut changed, mut unparsed, mut failed, mut internal) = (0usize, 0usize, 0usize, 0usize);
    for shown in &scripts {
        let path = project.root.join(shown);
        let text = match fs.read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                let _ = writeln!(err, "error: cannot read {shown}: {error}");
                failed += 1;
                continue;
            }
        };
        // `ZEFF_FMT_FORCE_INTERNAL`, naming a script, makes the formatter fail on it: a test
        // hook, so that the path of a result the formatter cannot vouch for is exercised.
        let formatted = if std::env::var("ZEFF_FMT_FORCE_INTERNAL").as_deref() == Ok(shown) {
            Err(zeff_syntax::format::Unformatted::Internal(
                "forced by ZEFF_FMT_FORCE_INTERNAL".to_string(),
            ))
        } else {
            zeff_syntax::format::format(shown, &text)
        };
        match formatted {
            Ok(written) if written == text => {}
            Ok(_) if check => {
                let _ = writeln!(err, "would format {shown}");
                changed += 1;
            }
            Ok(written) => match fs.write(&path, written.as_bytes()) {
                Ok(()) => {
                    let _ = writeln!(err, "formatted {shown}");
                    changed += 1;
                }
                Err(error) => {
                    let _ = writeln!(err, "error: cannot write {shown}: {error}");
                    failed += 1;
                }
            },
            Err(zeff_syntax::format::Unformatted::Syntax(diagnostics)) => {
                let mut sources = Sources::new();
                sources.add(Source::new(shown.clone(), text));
                for diagnostic in diagnostics.iter() {
                    command.messages.show(err, diagnostic, &sources);
                    let _ = writeln!(err);
                }
                unparsed += 1;
            }
            Err(zeff_syntax::format::Unformatted::Internal(what)) => {
                let _ = writeln!(
                    err,
                    "error[Z9001]: internal error: this is a bug in Zeff, not in your script\n  \
                     the formatter left {shown} as it is: {what}"
                );
                internal += 1;
            }
        }
    }
    // The closing line counts the scripts that could be read and parsed.
    let all = scripts.len() - unparsed - failed - internal;
    let _ = match (check, changed) {
        _ if all == 0 => Ok(()),
        (true, 0) => writeln!(err, "{}: formatted", count(all)),
        (true, n) => writeln!(
            err,
            "{n} of {} would change: `zeff fmt` formats {}",
            count(all),
            if n == 1 { "it" } else { "them" }
        ),
        (false, 0) => writeln!(err, "{}: formatted already", count(all)),
        (false, n) => writeln!(err, "formatted {n} of {}", count(all)),
    };
    if unparsed > 0 {
        let _ = writeln!(
            err,
            "{} {} not parse and {} left as {}",
            count(unparsed),
            if unparsed == 1 { "does" } else { "do" },
            if unparsed == 1 { "was" } else { "were" },
            if unparsed == 1 { "it is" } else { "they are" },
        );
    }
    if internal > 0 {
        2
    } else if unparsed > 0 || failed > 0 || (check && changed > 0) {
        1
    } else {
        0
    }
}

/// `zeff test`: the tests of the project (Engine and Project Specification, sections 2.1 and
/// 12): every `assert` of the scripts that call one, a failed one kept so that all are
/// reported, and every `/// >` example of the functions of `functions/`, compared with what its
/// page says. The report goes to standard error, or as one JSON record to standard output; the
/// exit code is that of Language Reference section 3.8.
fn test_command(command: &Command<'_, '_>, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let fs = command.fs;
    let project = match open_project(fs, &command.cwd, command.messages, err) {
        Ok(project) => project,
        Err(code) => return code,
    };
    let tests = zeff_engine::testing::test_project(fs, &project, &command.run_options());
    let (all, failed) = tests.counts();
    if command.settings.format == Format::Json {
        let text = zeff_diagnostics::json_string;
        let list = |lines: &[String]| {
            let items: Vec<String> = lines.iter().map(|line| text(line)).collect();
            format!("[{}]", items.join(", "))
        };
        let _ = writeln!(out, "{{");
        let _ = writeln!(out, "  \"tests\": {all},");
        let _ = writeln!(out, "  \"failed\": {failed},");
        let _ = writeln!(out, "  \"scripts\": [");
        let last = tests.scripts.len().saturating_sub(1);
        for (at, script) in tests.scripts.iter().enumerate() {
            let _ = writeln!(
                out,
                "    {{\"file\": {}, \"checks\": {}, \"failed\": {}, \"stopped\": {}, \"exit_code\": {}}}{}",
                text(&script.file),
                script.checks,
                list(&script.failed),
                script.stopped.as_deref().map_or("null".to_string(), text),
                script.exit_code,
                if at < last { "," } else { "" }
            );
        }
        let _ = writeln!(out, "  ],");
        let _ = writeln!(out, "  \"examples\": [");
        let last = tests.examples.len().saturating_sub(1);
        for (at, example) in tests.examples.iter().enumerate() {
            let _ = writeln!(
                out,
                "    {{\"file\": {}, \"function\": {}, \"example\": {}, \"line\": {}, \"passed\": {}, \"code\": {}, \"expected\": {}, \"found\": {}}}{}",
                text(&example.file),
                text(&example.function),
                example.example,
                example.line,
                example.passed(),
                list(&example.code),
                list(&example.expected),
                list(&example.found),
                if at < last { "," } else { "" }
            );
        }
        let _ = writeln!(out, "  ],");
        let _ = writeln!(
            out,
            "  \"without_checks\": {},",
            list(&tests.without_checks)
        );
        let _ = writeln!(out, "  \"problems\": {}", list(&tests.problems));
        let _ = writeln!(out, "}}");
        return tests.exit_code();
    }
    let count = |n: usize, one: &str, many: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {many}")
        }
    };
    let show = |err: &mut dyn Write, messages: &str| {
        let _ = err.write_all(command.messages.text(messages).as_bytes());
    };
    for problem in &tests.problems {
        let _ = writeln!(err, "error: {problem}\n");
    }
    for script in &tests.scripts {
        let checks = count(script.checks, "check", "checks");
        if script.passed() {
            if command.settings.verbose {
                let _ = writeln!(err, "passed: {}, {checks}", script.file);
            }
            continue;
        }
        if !script.failed.is_empty() {
            let _ = writeln!(
                err,
                "failed: {}, {} of {checks}",
                script.file,
                script.failed.len()
            );
            for failure in &script.failed {
                show(err, failure);
            }
            let _ = writeln!(err);
        }
        if let Some(stopped) = &script.stopped {
            let _ = writeln!(err, "stopped: {}, after {checks}", script.file);
            show(err, stopped);
            let _ = writeln!(err);
        }
    }
    for example in &tests.examples {
        let place = format!(
            "{}:{}, example {} of `{}`",
            example.file, example.line, example.example, example.function
        );
        if example.passed() {
            if command.settings.verbose {
                let _ = writeln!(err, "passed: {place}");
            }
            continue;
        }
        let _ = writeln!(
            err,
            "failed: {place} prints something else than its page says"
        );
        for line in &example.code {
            let _ = writeln!(err, "    > {line}");
        }
        let _ = writeln!(err, "  the page says:");
        for line in &example.expected {
            let _ = writeln!(err, "    {line}");
        }
        let _ = writeln!(err, "  it printed:");
        for line in &example.found {
            let _ = writeln!(err, "    {line}");
        }
        let _ = writeln!(err);
    }
    if command.settings.verbose {
        for file in &tests.without_checks {
            let _ = writeln!(err, "not run: {file} calls no `assert`");
        }
    }
    if all == 0 && tests.problems.is_empty() {
        let _ = writeln!(
            err,
            "no tests: no script of the project calls `assert`, and no function of `functions/` has an example"
        );
    } else if failed == 0 {
        let _ = writeln!(err, "{} passed", count(all, "test", "tests"));
    } else {
        let _ = writeln!(
            err,
            "{}: {} passed, {failed} failed",
            count(all, "test", "tests"),
            all - failed
        );
    }
    tests.exit_code()
}

fn internal_error(err: &mut dyn Write, errors: Vec<zeff_registry::RegistryError>) -> i32 {
    let _ = writeln!(
        err,
        "error[Z9001]: internal error: this is a bug in Zeff, not in your script"
    );
    for problem in errors {
        let _ = writeln!(err, "  {problem}");
    }
    2
}

/// `zeff help <name>`: a function's page, a package's functions, or the explained page of a code
/// such as `Z0101` (Language Reference, section 11.1), on standard output.
fn help_command(
    command: &Command<'_, '_>,
    name: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    // Inside a project, a function of its `functions/` folder has a page too (section 5.3).
    if !zeff_engine::explain::is_code(name)
        && let Ok(project) = zeff_engine::project::Project::open(command.fs, &command.cwd)
        && let Ok(registry) = zeff_engine::registry::registry()
        && let Some(page) = zeff_engine::help::project_help(command.fs, &project, registry, name)
    {
        let _ = out.write_all(page.as_bytes());
        return 0;
    }
    match help_text(name) {
        Ok(text) => {
            let _ = out.write_all(text.as_bytes());
            0
        }
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            1
        }
    }
}

/// The text `zeff help <name>` and the console's `:help <name>` print.
fn help_text(name: &str) -> Result<String, String> {
    if zeff_engine::explain::is_code(name) {
        return zeff_engine::explain::explain(name)
            .map(|page| zeff_engine::explain::render(&page))
            .map_err(|problem| problem.to_string());
    }
    let registry = zeff_engine::registry::registry().map_err(|errors| {
        format!(
            "internal error: this is a bug in Zeff, not in your script: {}",
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    zeff_engine::help::help_text(registry, name)
}

/// The scratch project, or the reason there is none.
fn scratch(command: &Command<'_, '_>, err: &mut dyn Write) -> Result<Project, i32> {
    let Some(home) = &command.env.zeff_home else {
        let _ = writeln!(
            err,
            "error: cannot find your home folder for the scratch project; set ZEFF_HOME to a folder Zeff may use"
        );
        return Err(1);
    };
    zeff_engine::scratch::open_scratch(command.fs, home).map_err(|message| {
        let _ = writeln!(err, "error: {message}");
        1
    })
}

/// `zeff eval "<statement>"`: one statement run in the scratch project, its value printed.
fn eval_command(
    command: &Command<'_, '_>,
    statement: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let project = match scratch(command, err) {
        Ok(project) => project,
        Err(code) => return code,
    };
    let text = format!("{}\n", statement.trim_end());
    let outcome = run_script_with(command.fs, &project, "eval", &text, &command.run_options());
    emit(command, &outcome, out, err);
    outcome.exit_code
}

/// What the console does after a `:` command.
enum Next {
    Continue,
    Quit,
}

/// The console: the project's session, or the scratch project's outside one.
struct Console<'a, 'b, 'c> {
    command: &'a mut Command<'b, 'c>,
    project: Project,
    /// The session runs in the scratch project (no project around the current folder).
    in_scratch: bool,
    session: Session,
    /// Files `:save` wrote in this session, which it may write again.
    saved: Vec<PathBuf>,
}

/// `zeff repl [script]`: the console (Engine and Project Specification, section 2.1).
fn repl_command(
    command: &mut Command<'_, '_>,
    script: Option<&str>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let fs = command.fs;
    let (project, in_scratch) = match find_project(fs, &command.cwd, command.messages, err) {
        Ok(Some(project)) => (project, false),
        Ok(None) => match scratch(command, err) {
            Ok(project) => (project, true),
            Err(code) => return code,
        },
        Err(code) => return code,
    };
    let (session, opened) = match Session::open(fs, &project) {
        Ok(opened) => opened,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            return 1;
        }
    };
    let place = if in_scratch {
        "the scratch project".to_string()
    } else {
        format!("the project `{}`", project.manifest.name)
    };
    let _ = writeln!(
        err,
        "Zeff {VERSION} console in {place}; :help lists the commands, :quit leaves"
    );
    match opened {
        Opened::Fresh => {}
        Opened::Resumed(n) => {
            let _ = writeln!(
                err,
                "resumed {n} {} from {SESSION_FILE}; :vars lists the bindings",
                if n == 1 { "statement" } else { "statements" }
            );
        }
        Opened::SetAside(problem) => {
            let _ = writeln!(
                err,
                "the session in {SESSION_FILE} could not be read ({problem}); it is kept as {} and the console starts afresh",
                zeff_engine::session::PREVIOUS_FILE
            );
        }
    }
    let mut console = Console {
        command,
        project,
        in_scratch,
        session,
        saved: Vec::new(),
    };
    if let Some(script) = script {
        console.load(script, out, err);
    }
    let mut pending = String::new();
    loop {
        if console.command.env.stdin_is_terminal {
            let _ = write!(
                err,
                "{}",
                if pending.is_empty() {
                    "zeff> "
                } else {
                    "  ... "
                }
            );
            let _ = err.flush();
        }
        let mut line = String::new();
        match console.command.env.input.read_line(&mut line) {
            Ok(0) => {
                if console.command.env.stdin_is_terminal {
                    let _ = writeln!(err);
                }
                break;
            }
            Ok(_) => {}
            Err(error) => {
                let _ = writeln!(err, "error: cannot read the input: {error}");
                return 1;
            }
        }
        let line = line.trim_end_matches(['\n', '\r']);
        if pending.is_empty() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(words) = trimmed.strip_prefix(':') {
                match console.command_line(words, out, err) {
                    Next::Continue => continue,
                    Next::Quit => break,
                }
            }
            if trimmed.starts_with('.') {
                match console.session.pop() {
                    Some(previous) => pending = format!("{previous}\n{line}"),
                    None => {
                        let _ = writeln!(
                            err,
                            "nothing to continue: a line beginning with `.` adds a step to the statement before it"
                        );
                        continue;
                    }
                }
            } else {
                pending = line.to_string();
            }
            if is_incomplete(&pending) {
                continue;
            }
        } else {
            pending.push('\n');
            pending.push_str(line);
            // An empty line ends an unfinished statement: its error is shown, and it is dropped.
            if !line.trim().is_empty() && is_incomplete(&pending) {
                continue;
            }
        }
        let statement = std::mem::take(&mut pending);
        console.run_new(vec![statement], out, err);
    }
    0
}

impl Console<'_, '_, '_> {
    fn messages(&self) -> Messages {
        self.command.messages
    }

    /// Run the session with `statements` added; on an error they are dropped again.
    fn run_new(&mut self, statements: Vec<String>, out: &mut dyn Write, err: &mut dyn Write) {
        let mut added = 0usize;
        for statement in statements {
            // A comment alone is nothing to run or keep.
            if let Ok(items) = split_statements(&statement)
                && items.is_empty()
            {
                continue;
            }
            self.session.push(statement);
            added += 1;
        }
        if added == 0 {
            return;
        }
        let fs = self.command.fs;
        // A session keeps every binding until it is shadowed (Language Reference, section 6,
        // "Memory"); only `zeff run` frees a binding after its last use.
        let options = RunOptions {
            free_bindings: false,
            ..self.command.run_options()
        };
        let outcome = self.session.run(fs, &self.project, &options);
        let output = &outcome.output;
        if outcome.exit_code != 0 {
            // The earlier statements' own messages were shown when they were typed.
            let earlier: String = output
                .statements
                .iter()
                .take(self.session.len() - added)
                .map(|s| s.stderr.as_str())
                .collect();
            let fresh = output
                .stderr
                .strip_prefix(earlier.as_str())
                .unwrap_or(&output.stderr);
            let _ = err.write_all(self.messages().text(fresh).as_bytes());
            for _ in 0..added {
                self.session.pop();
            }
            return;
        }
        let from = output.statements.len().saturating_sub(added);
        for statement in &output.statements[from..] {
            let _ = out.write_all(statement.stdout.as_bytes());
            let _ = err.write_all(self.messages().text(&statement.stderr).as_bytes());
        }
        if let Err(error) = self.session.save(fs, &self.project) {
            let _ = writeln!(err, "error: cannot write {SESSION_FILE}: {error}");
        }
    }

    /// A `:` command; `words` is what follows the colon.
    fn command_line(&mut self, words: &str, out: &mut dyn Write, err: &mut dyn Write) -> Next {
        let (name, argument) = match words.trim().split_once(char::is_whitespace) {
            Some((name, argument)) => (name, argument.trim()),
            None => (words.trim(), ""),
        };
        match (name, argument) {
            ("quit" | "q" | "exit", _) => return Next::Quit,
            ("help", "") => {
                let _ = out.write_all(CONSOLE_HELP.as_bytes());
            }
            ("help", name) => match help_text(name) {
                Ok(text) => {
                    let _ = out.write_all(text.as_bytes());
                }
                Err(message) => {
                    let _ = writeln!(err, "error: {message}");
                }
            },
            ("undo", _) => match self.session.pop() {
                Some(statement) => {
                    let first = statement.lines().next().unwrap_or_default();
                    let _ = writeln!(err, "undid: {first}");
                    if let Err(error) = self.session.save(self.command.fs, &self.project) {
                        let _ = writeln!(err, "error: cannot write {SESSION_FILE}: {error}");
                    }
                }
                None => {
                    let _ = writeln!(err, "nothing to undo: the session has no statements");
                }
            },
            ("vars", _) => self.vars(out, err),
            ("save", "") => {
                let _ = writeln!(
                    err,
                    "`:save` needs a path, as in `:save scripts/explore.zeff`{}",
                    if self.in_scratch {
                        "; outside a project, a name, as in `:save schooling`, creates the project"
                    } else {
                        ""
                    }
                );
            }
            ("save", path) => self.save(path, err),
            ("load", "") => {
                let _ = writeln!(
                    err,
                    "`:load` needs a script, as in `:load scripts/main.zeff`"
                );
            }
            ("load", path) => self.load(path, out, err),
            (other, _) => {
                let _ = writeln!(
                    err,
                    "unknown console command `:{other}`; :help lists the commands"
                );
            }
        }
        Next::Continue
    }

    /// `:vars`: the session's bindings with their types, from the checker.
    fn vars(&self, out: &mut dyn Write, err: &mut dyn Write) {
        let registry = match zeff_engine::registry::registry() {
            Ok(registry) => registry,
            Err(errors) => {
                internal_error(err, errors);
                return;
            }
        };
        let checked = zeff_ir::check_source(SESSION_FILE, &self.session.text(), registry);
        // A later `let` shadows an earlier one (Language Reference, section 3.1): the name is
        // listed once, with the type it has now.
        let mut bindings: Vec<&zeff_ir::Binding> = Vec::new();
        for binding in &checked.program.bindings {
            bindings.retain(|b| b.name != binding.name);
            bindings.push(binding);
        }
        if bindings.is_empty() {
            let _ = writeln!(err, "no bindings yet: `let name = ...` makes one");
            return;
        }
        let width = bindings
            .iter()
            .map(|b| b.name.chars().count())
            .max()
            .unwrap_or(0);
        for binding in bindings {
            let _ = writeln!(out, "{:width$}  {}", binding.name, binding.ty);
        }
    }

    /// `:save <path>`: the session as a script of the project; in the scratch project, a new
    /// project named `<path>` with the session as its `scripts/main.zeff`.
    fn save(&mut self, path: &str, err: &mut dyn Write) {
        let fs = self.command.fs;
        let n = self.session.len();
        let count = format!("{n} {}", if n == 1 { "statement" } else { "statements" });
        if self.in_scratch {
            match create_project(fs, &self.command.cwd, path) {
                Ok(root) => {
                    let script = root.join("scripts").join("main.zeff");
                    match fs.write(&script, self.session.text().as_bytes()) {
                        Ok(()) => {
                            let _ = writeln!(
                                err,
                                "created the project `{path}` in {} with the {count} of the session as scripts/main.zeff",
                                root.display()
                            );
                        }
                        Err(error) => {
                            let _ =
                                writeln!(err, "error: cannot write {}: {error}", script.display());
                        }
                    }
                }
                Err(NewError::BadName { name, advice }) => {
                    let _ = writeln!(
                        err,
                        "error: `{name}` is not a good project name\n  help: {advice}"
                    );
                }
                Err(NewError::Exists(existing)) => {
                    let _ = writeln!(
                        err,
                        "error: {} already exists; `:save` makes a new project and never changes an existing folder",
                        existing.display()
                    );
                }
                Err(NewError::Io(message)) => {
                    let _ = writeln!(err, "error: could not create the project: {message}");
                }
            }
            return;
        }
        let target = match self.project.resolve_write(fs, path) {
            Ok(target) => target,
            Err(problem) => {
                self.messages()
                    .show(err, &path_diagnostic(&problem, path), &Sources::new());
                return;
            }
        };
        if fs.is_file(&target) && !self.saved.contains(&target) {
            let _ = writeln!(
                err,
                "error: `{path}` already exists; `:save` writes only a new file, or one it wrote in this session"
            );
            return;
        }
        if let Some(parent) = target.parent()
            && !fs.is_dir(parent)
        {
            let _ = writeln!(
                err,
                "error: the folder of `{path}` does not exist; `:save scripts/<name>.zeff` writes beside the other scripts"
            );
            return;
        }
        match fs.write(&target, self.session.text().as_bytes()) {
            Ok(()) => {
                if !self.saved.contains(&target) {
                    self.saved.push(target);
                }
                let _ = writeln!(err, "saved the {count} of the session to {path}");
            }
            Err(error) => {
                let _ = writeln!(err, "error: cannot write {path}: {error}");
            }
        }
    }

    /// `:load <path>`: a script's statements run into the session, as if typed.
    fn load(&mut self, path: &str, out: &mut dyn Write, err: &mut dyn Write) {
        let fs = self.command.fs;
        let file = if self.in_scratch {
            self.command.cwd.join(path)
        } else {
            match self.project.resolve_read(fs, path) {
                Ok(file) => file,
                Err(problem) => {
                    self.messages()
                        .show(err, &path_diagnostic(&problem, path), &Sources::new());
                    return;
                }
            }
        };
        if !fs.is_file(&file) {
            let _ = writeln!(err, "error: no script `{path}`");
            return;
        }
        let text = match fs.read_to_string(&file) {
            Ok(text) => text,
            Err(error) => {
                let _ = writeln!(err, "error: cannot read {path}: {error}");
                return;
            }
        };
        match split_statements(&text) {
            Ok(statements) => {
                let n = statements.len();
                self.run_new(statements, out, err);
                let _ = writeln!(
                    err,
                    "loaded {n} {} from {path}",
                    if n == 1 { "statement" } else { "statements" }
                );
            }
            Err(diagnostics) => {
                let mut sources = Sources::new();
                sources.add(Source::new(path.to_string(), text));
                for diagnostic in diagnostics.iter() {
                    self.messages().show(err, diagnostic, &sources);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_taken_from_anywhere_on_the_line() {
        let words = [
            "--width",
            "60",
            "check",
            "--ascii",
            "-q",
            "x.zeff",
            "--color=never",
        ];
        let (rest, settings) = take_options(&words).unwrap();
        assert_eq!(rest, ["check", "x.zeff"]);
        assert_eq!(settings.width, 60);
        assert!(settings.ascii && settings.quiet && !settings.verbose);
        assert_eq!(settings.color, Some(false));
        let (rest, _) = take_options(&["eval", "-7"]).unwrap();
        assert_eq!(rest, ["eval", "-7"]);
        assert!(take_options(&["--width", "20"]).is_err());
        assert!(take_options(&["--width"]).is_err());
        assert!(
            take_options(&["--frozen"])
                .unwrap_err()
                .contains("not available")
        );
        assert!(
            take_options(&["--bogus"])
                .unwrap_err()
                .contains("unknown option")
        );
    }

    #[test]
    fn ascii_and_colour_filters() {
        assert_eq!(
            to_ascii("w · 2 rows × 3 variables … → —"),
            "w | 2 rows x 3 variables ... -> -"
        );
        let messages = Messages {
            color: true,
            quiet: true,
        };
        let text = "note: skipped (line 1)\nwarning: 1 row with NA in x was skipped (line 2)\nerror[Z0101]: no\n 1 | x\n";
        assert_eq!(
            messages.text(text),
            "\x1b[1;33mwarning\x1b[0m: 1 row with NA in x was skipped (line 2)\n\x1b[1;31merror[Z0101]\x1b[0m: no\n 1 | x\n"
        );
        let mut sink = Vec::new();
        {
            let mut filtered = Filtered::new(&mut sink, true);
            let bytes = "a · b".as_bytes();
            // The two-byte `·` arrives split across writes.
            filtered.write_all(&bytes[..3]).unwrap();
            filtered.write_all(&bytes[3..]).unwrap();
        }
        assert_eq!(String::from_utf8(sink).unwrap(), "a | b");
    }
}
