//! `zeff report-bug` (Language Reference, section 11.1, "Bug reports"; Engine and Project
//! Specification, section 2.1; release 0.8.0, step 20a): a bug report built on the user's own
//! machine, as one file the user reads before attaching it anywhere.
//!
//! The report holds the engine's version, edition and platform; what went wrong, as it was
//! printed; the failing statement and only the statements it depends on, found from the
//! dependency graph of the typed IR and then run alone, to see that they fail in the same way;
//! what the run said before it failed, and its seed; and the schema of the data: the names and
//! types of the variables and how many values are missing. It holds no rows of the data; what
//! a message itself names (a text that could not be converted, a level) is quoted with the
//! message, and the report says so at its head. With `--include-data` the files those
//! statements read are written beside it, as they are.
//!
//! Three kinds of report: an error of the script, an internal error (a bug in Zeff), and a
//! result believed wrong, where nothing failed: its statement is named by its line and the
//! reference it should agree with is given as a file.
//!
//! Nothing here is sent anywhere, and nothing here reads a clock: the same failure gives the
//! same file, under the same name.

use crate::control::Control;
use crate::dependencies::{Graph, graph};
use crate::fs::{FileInfo, FileSystem};
use crate::project::Project;
use crate::run::{Frames, Outcome, RunOptions, check_in_project, run_script_keeping};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use zeff_diagnostics::Source;

/// What is asked of `zeff report-bug`.
#[derive(Debug, Clone)]
pub struct Asked<'a> {
    /// The script, by its name in the project (`scripts/main.zeff`), and its text.
    pub script: &'a str,
    pub text: &'a str,
    /// A result believed wrong, where nothing failed: the line of the statement that gave it.
    pub line: Option<u32>,
    /// The reference a result believed wrong should agree with: the name of its file, and
    /// its text.
    pub reference: Option<(&'a str, &'a str)>,
    /// `--include-data`: the files the kept statements read come with the report.
    pub include_data: bool,
    /// The machine, as the report names it (`aarch64 macos`).
    pub platform: &'a str,
    pub options: RunOptions,
}

/// The kind of failure a report is of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The script stopped with an error.
    Error,
    /// Zeff stopped with an internal error (Z9xxx): a bug in Zeff.
    InternalError,
    /// Nothing failed, and a result is believed wrong.
    WrongResult,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Error => "error",
            Kind::InternalError => "internal error",
            Kind::WrongResult => "wrong result",
        }
    }
}

/// A report, not yet written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub kind: Kind,
    /// The code of the error, where the failure has one.
    pub code: Option<String>,
    /// The file's name, without its folder: `main-Z0410-1a2b3c4d.md`.
    pub name: String,
    /// The report: Markdown, written to be read as it stands.
    pub text: String,
    /// How many statements the report keeps, and how many the script has.
    pub kept: usize,
    pub of: usize,
    /// Whether the kept statements, run alone, failed in the same way; if not, every statement
    /// up to the failing one is kept.
    pub reduced: bool,
    /// The files the kept statements read, by their path in the project (or their name, for a
    /// file outside it), with their bytes when `--include-data` asked for them.
    pub files: Vec<(String, Option<Vec<u8>>)>,
}

/// Why no report was built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoReport {
    /// The script ran without an error, and no statement was named as giving a wrong result.
    NothingFailed,
    /// A statement was named as giving a wrong result, and the script did not run through.
    FailedInstead { line: u32 },
    /// No statement of the script is written at the line named.
    NoStatementAt { line: u32 },
    /// A result believed wrong needs the reference it should agree with.
    NoReference,
    /// A reference was given without the line of the statement it is for.
    NoLine,
}

impl std::fmt::Display for NoReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoReport::NothingFailed => write!(
                f,
                "the script ran without an error: there is nothing to report\n  = help: for a result you believe is wrong, name its statement and what it should give: `zeff report-bug --line <n> --reference <file>`"
            ),
            NoReport::FailedInstead { line } => write!(
                f,
                "the script stopped with an error, so the result of line {line} cannot be reported as wrong\n  = help: `zeff report-bug` without `--line` reports the error; correct it first if the wrong result is the matter"
            ),
            NoReport::NoStatementAt { line } => write!(
                f,
                "no statement of the script is written at line {line}\n  = help: `--line` takes a line of the statement whose result is wrong"
            ),
            NoReport::NoReference => write!(
                f,
                "a report of a wrong result needs the result it should agree with\n  = help: `--reference <file>` names a text file holding what R, Stata or a computation by hand gives"
            ),
            NoReport::NoLine => write!(
                f,
                "`--reference` needs the statement it is for\n  = help: `--line <n>` names a line of the statement whose result is wrong"
            ),
        }
    }
}

/// A file system that notes which files are read: what a run's statements took from the
/// project, for the report's list of data files.
struct Noting<'a> {
    inner: &'a dyn FileSystem,
    read: RefCell<Vec<PathBuf>>,
    /// The files the run wrote: one of these read back is the run's own product, not data
    /// it was given.
    written: RefCell<Vec<PathBuf>>,
}

impl FileSystem for Noting<'_> {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let bytes = self.inner.read(path)?;
        let mut read = self.read.borrow_mut();
        if !read.iter().any(|p| p == path) {
            read.push(path.to_path_buf());
        }
        Ok(bytes)
    }
    fn metadata(&self, path: &Path) -> io::Result<FileInfo> {
        self.inner.metadata(path)
    }
    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        self.inner.list(dir)
    }
    fn is_file(&self, path: &Path) -> bool {
        self.inner.is_file(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        self.inner.is_dir(path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.inner.canonicalize(path)
    }
    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.inner.create_dir(path)
    }
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.written.borrow_mut().push(path.to_path_buf());
        self.inner.write(path, bytes)
    }
    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.inner.remove_dir_all(path)
    }
    fn now(&self) -> Option<i64> {
        self.inner.now()
    }
}

/// Where the error a run stopped with stands in what the run said: from the line that begins
/// it to the end, or to the next error. A statement that fails says one error, the last thing
/// the run says. A script the check refuses has run no statement, and where the check says
/// several errors the report is of the first, as a reader meets it.
fn error_place(outcome: &Outcome) -> (usize, usize) {
    let stderr = &outcome.output.stderr;
    let begins = |line: &str| line.starts_with("error[") || line.starts_with("error:");
    let by_a_statement = outcome
        .output
        .statements
        .last()
        .is_some_and(|last| last.stderr.lines().any(begins));
    let mut starts: Vec<usize> = Vec::new();
    let mut offset = 0;
    for line in stderr.split_inclusive('\n') {
        if begins(line) {
            starts.push(offset);
        }
        offset += line.len();
    }
    match (starts.first(), starts.last()) {
        (Some(_), Some(last)) if by_a_statement => (*last, stderr.len()),
        (Some(first), _) => (*first, starts.get(1).copied().unwrap_or(stderr.len())),
        _ => (stderr.len(), stderr.len()),
    }
}

/// The error a run stopped with, as printed. Empty where the run said no error.
fn error_of(outcome: &Outcome) -> &str {
    let (start, end) = error_place(outcome);
    &outcome.output.stderr[start..end]
}

/// The code of an error as printed: `Z0410` of `error[Z0410]: ...`.
fn code_of(error: &str) -> Option<String> {
    let rest = error.strip_prefix("error[")?;
    let code = rest.split(']').next()?;
    crate::explain::is_code(code).then(|| code.to_string())
}

/// The line a failed run stopped at: the line of the last statement it began, when that
/// statement said the error; else the first place the error names in the script.
fn failing_line(outcome: &Outcome, script: &str) -> Option<u32> {
    if let Some(last) = outcome.output.statements.last()
        && (last.stderr.contains("error[") || last.stderr.contains("error:"))
    {
        return Some(last.line);
    }
    let error = error_of(outcome);
    let marker = format!(" --> {script}:");
    let at = error.find(&marker)? + marker.len();
    error[at..]
        .split(':')
        .next()
        .and_then(|line| line.trim().parse().ok())
}

/// FNV-1a over a text, for the report's name: the same report has the same name.
fn digest(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// `1,234`: a count with its thousands apart, as the engine prints counts.
fn count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `lines 1, 2 and 7`, or `line 4`.
fn lines_text(lines: &[u32]) -> String {
    let words: Vec<String> = lines.iter().map(u32::to_string).collect();
    match words.as_slice() {
        [] => "no line".to_string(),
        [one] => format!("line {one}"),
        [most @ .., last] => format!("lines {} and {last}", most.join(", ")),
    }
}

/// The script with only the lines kept, the others left empty, so that every statement kept
/// stands on the line it stood on and says what it said, place for place.
fn blanked(source: &Source, keep: &BTreeSet<u32>) -> String {
    let mut out = String::new();
    for line in 1..=source.line_count() as u32 {
        if keep.contains(&line) {
            out.push_str(source.line_text(line));
        }
        out.push('\n');
    }
    out
}

/// What the statements kept are, in lines of the script.
struct Kept {
    /// Every line kept: the statements', the `use` lines, the functions of the script they
    /// call, and the pragmas and documentation lines that stand directly above them.
    lines: BTreeSet<u32>,
    /// How many statements are kept, and the line each begins on.
    statements: Vec<u32>,
    /// The functions of the project's modules they call: file, then the lines of each.
    module_functions: Vec<(String, u32, u32)>,
}

/// The lines of a comment block that stands directly above `line`: every line above it, going
/// up, that begins with `prefix` once its indent is dropped.
fn above(source: &Source, line: u32, prefix: &str) -> Vec<u32> {
    let mut found = Vec::new();
    let mut at = line;
    while at > 1 && source.line_text(at - 1).trim_start().starts_with(prefix) {
        at -= 1;
        found.push(at);
    }
    found
}

#[allow(clippy::too_many_arguments)]
fn kept_lines(
    source: &Source,
    script: &str,
    text: &str,
    program: &zeff_ir::Program,
    graph: &Graph,
    statements: &[usize],
    functions: &[zeff_ir::FnId],
) -> Kept {
    let mut lines: BTreeSet<u32> = BTreeSet::new();
    // Every `use` line: it names a module, and holds no data.
    for item in &zeff_syntax::parse(script, text).script.items {
        if let zeff_syntax::ast::Item::Use(used) = item {
            for line in source.line_of(used.span.start)..=source.line_of(used.span.end) {
                lines.insert(line);
            }
        }
    }
    let mut begins = Vec::new();
    for index in statements {
        let span = graph.statements[*index].span;
        let first = source.line_of(span.start);
        let last = source.line_of(span.end.saturating_sub(1).max(span.start));
        begins.push(first);
        lines.extend(first..=last);
        // A pragma, `// zeff: allow(...)`, on its own line above the statement is part of it.
        lines.extend(above(source, first, "// zeff:"));
    }
    let mut module_functions = Vec::new();
    for id in functions {
        let function = &program.functions[*id];
        // A function literal is written inside its statement.
        if function.name_span == function.span {
            continue;
        }
        if function.file == script {
            let first = source.line_of(function.span.start);
            let last = source.line_of(function.span.end.saturating_sub(1).max(function.span.start));
            lines.extend(first..=last);
            lines.extend(above(source, first, "///"));
        } else {
            module_functions.push((
                function.file.clone(),
                function.span.start as u32,
                function.span.end as u32,
            ));
        }
    }
    Kept {
        lines,
        statements: begins,
        module_functions,
    }
}

/// A run of `text` under a file system that notes what is read: the outcome, the frames its
/// bindings held, the files it read that it had not written itself, and those it wrote.
fn run_noting(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
) -> (Outcome, Frames, Vec<PathBuf>, Vec<PathBuf>) {
    let noting = Noting {
        inner: fs,
        read: RefCell::default(),
        written: RefCell::default(),
    };
    let (outcome, frames) = run_script_keeping(
        &noting,
        project,
        script,
        text,
        options,
        &Control::new(),
        None,
    );
    let written = noting.written.into_inner();
    let read = noting
        .read
        .into_inner()
        .into_iter()
        .filter(|path| !written.contains(path))
        .collect();
    (outcome, frames, read, written)
}

/// What the statement that begins on `line` printed and said.
fn said_at(outcome: &Outcome, line: u32) -> Option<String> {
    outcome
        .output
        .statements
        .iter()
        .rev()
        .find(|s| s.line == line)
        .map(|s| format!("{}{}", s.stdout, s.stderr))
}

/// A text as an indented block, a line to a line; an empty text as one line that says so.
fn block(text: &str) -> String {
    let mut out = String::new();
    for line in text.trim_end_matches('\n').lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Build the report of a script's failure, or say why there is none. The script is run, once
/// whole and once with the kept statements alone; nothing is written.
pub fn build(
    fs: &dyn FileSystem,
    project: &Project,
    asked: &Asked<'_>,
) -> Result<Report, NoReport> {
    match (asked.line, asked.reference) {
        (Some(_), None) => return Err(NoReport::NoReference),
        (None, Some(_)) => return Err(NoReport::NoLine),
        _ => {}
    }
    let source = Source::new(asked.script, asked.text);
    let (whole, _, _, _) = run_noting(fs, project, asked.script, asked.text, &asked.options);
    let kind = match (whole.exit_code, asked.line) {
        (0, None) => return Err(NoReport::NothingFailed),
        (0, Some(_)) => Kind::WrongResult,
        (_, Some(line)) => return Err(NoReport::FailedInstead { line }),
        (2, None) => Kind::InternalError,
        (_, None) => Kind::Error,
    };
    // The graph, and the statement the report is about.
    let checked = crate::registry::registry()
        .ok()
        .map(|registry| check_in_project(fs, project, asked.script, asked.text, registry));
    // Without a registry nothing is checked (the run itself said so, as an internal error):
    // the graph is then of no statement, and the script is kept whole.
    let empty = zeff_ir::Program {
        statements: Vec::new(),
        bindings: Vec::new(),
        functions: Vec::new(),
        schemas: Vec::new(),
        bound_schemas: Vec::new(),
        run_time_frames: Vec::new(),
        frame_lets: Vec::new(),
    };
    let program = checked.as_ref().map_or(&empty, |c| &c.checked.program);
    let graph = graph(program, asked.script);
    let of = graph.statements.len();
    let statement_on = |line: u32| -> Option<usize> {
        graph.statements.iter().position(|node| {
            source.line_of(node.span.start) <= line
                && line <= source.line_of(node.span.end.saturating_sub(1).max(node.span.start))
        })
    };
    let about_line = match kind {
        Kind::WrongResult => asked.line,
        _ => failing_line(&whole, asked.script),
    };
    let about = about_line.and_then(statement_on);
    if kind == Kind::WrongResult && about.is_none() {
        return Err(NoReport::NoStatementAt {
            line: asked.line.unwrap_or(0),
        });
    }
    // What failed, as the whole script said it.
    let failure = |outcome: &Outcome| -> String {
        match kind {
            Kind::WrongResult => about
                .and_then(|index| {
                    said_at(outcome, source.line_of(graph.statements[index].span.start))
                })
                .unwrap_or_default(),
            _ => error_of(outcome).to_string(),
        }
    };
    let said = failure(&whole);
    // The statements it needs, run alone; and, if they do not fail in the same way, every
    // statement up to it.
    let all_until = |index: Option<usize>| -> Vec<usize> {
        match index {
            Some(index) => (0..=index).collect(),
            None => (0..of).collect(),
        }
    };
    let mut reduced = about.is_some();
    let mut attempt = match about {
        Some(index) => {
            let needed = graph.needed_by(index);
            kept_lines(
                &source,
                asked.script,
                asked.text,
                program,
                &graph,
                &needed.statements,
                &needed.functions,
            )
        }
        None => {
            let all = all_until(None);
            kept_lines(
                &source,
                asked.script,
                asked.text,
                program,
                &graph,
                &all,
                &graph.called_by(&all),
            )
        }
    };
    // A script that could not be read into statements is kept whole.
    if of == 0 {
        attempt.lines = (1..=source.line_count() as u32).collect();
        reduced = false;
    }
    let mut alone = run_noting(
        fs,
        project,
        asked.script,
        &blanked(&source, &attempt.lines),
        &asked.options,
    );
    if reduced && (alone.0.exit_code != whole.exit_code || failure(&alone.0) != said) {
        reduced = false;
        let all = all_until(about);
        attempt = kept_lines(
            &source,
            asked.script,
            asked.text,
            program,
            &graph,
            &all,
            &graph.called_by(&all),
        );
        alone = run_noting(
            fs,
            project,
            asked.script,
            &blanked(&source, &attempt.lines),
            &asked.options,
        );
    }
    let (outcome, frames, read, written_by_the_run) = alone;
    let kept = attempt;

    // ----- the text -----
    let code = code_of(&said);
    let mut text = String::new();
    let title = match kind {
        Kind::Error => match &code {
            Some(code) => format!("an error, {code}"),
            None => "an error".to_string(),
        },
        Kind::InternalError => "an internal error".to_string(),
        Kind::WrongResult => "a result believed wrong".to_string(),
    };
    text.push_str(&format!("# Zeff bug report: {title}\n\n"));
    text.push_str(
        "This file was written by `zeff report-bug` on your computer. Nothing has been sent\nanywhere. Read it before you attach it to an issue: it holds statements of your script,\nthe names and types of your variables and the names of the files they were read from.\n",
    );
    text.push_str(if asked.include_data {
        "It holds no rows of your data itself; the files named at its end were written beside it\nbecause `--include-data` asked for them, and they hold your data as it is.\n"
    } else {
        "It holds no rows of your data.\n"
    });
    text.push_str(
        "A message is quoted as it was printed, and a message may name a value where it must (a\ntext that could not be converted, the levels of a group): read those lines too.\n",
    );
    text.push_str("\n## What went wrong\n\n");
    match kind {
        Kind::Error => {
            text.push_str("The script stopped with an error:\n\n");
            text.push_str(&block(&said));
        }
        Kind::InternalError => {
            text.push_str(
                "Zeff stopped with an internal error. This is a bug in Zeff, not in the script:\n\n",
            );
            text.push_str(&block(&said));
        }
        Kind::WrongResult => {
            text.push_str(&format!(
                "The statement at line {} ran without an error, and its result is believed to be\nwrong. Zeff printed:\n\n",
                about.map_or(0, |index| source.line_of(graph.statements[index].span.start))
            ));
            text.push_str(&block(if said.is_empty() {
                "(the statement printed nothing)"
            } else {
                &said
            }));
            if let Some((name, reference)) = asked.reference {
                let name = Path::new(name)
                    .file_name()
                    .map_or(name.to_string(), |n| n.to_string_lossy().into_owned());
                text.push_str(&format!(
                    "\nThe reference it should agree with, as given in `{name}`:\n\n"
                ));
                text.push_str(&block(if reference.trim().is_empty() {
                    "(the reference file is empty)"
                } else {
                    reference
                }));
            }
        }
    }
    text.push_str("\n## Zeff\n\n");
    text.push_str(&format!(
        "- version: {}\n- edition: {}\n- platform: {}\n",
        crate::VERSION,
        project.manifest.edition.name(),
        asked.platform
    ));
    text.push_str("\n## The statements\n\n");
    let what = match kind {
        Kind::WrongResult => "The statement and the statements it depends on",
        _ => "The failing statement and the statements it depends on",
    };
    let same = match kind {
        Kind::WrongResult => "print the same result",
        _ => "fail in the same way",
    };
    if of == 0 {
        text.push_str(&format!(
            "`{}` could not be read into statements, so the script is kept whole:\n\n",
            asked.script
        ));
    } else if reduced {
        text.push_str(&format!(
            "{what}: {} of the {} statements of\n`{}` ({}). Run alone, they {same}.\n\n",
            kept.statements.len(),
            of,
            asked.script,
            lines_text(&kept.statements)
        ));
    } else if about.is_some() {
        text.push_str(&format!(
            "The statements the graph of the script gives did not {same} when run alone:\none of them depends on another through a file or the state of the run, which the graph\ndoes not see. So every statement up to it is kept: {} of the {} statements of\n`{}`.\n\n",
            kept.statements.len(),
            of,
            asked.script
        ));
    } else {
        text.push_str(&format!(
            "The error is not of one statement, so every statement of `{}` is kept: {}.\n\n",
            asked.script,
            count(kept.statements.len())
        ));
    }
    let mut statements = String::new();
    for line in &kept.lines {
        statements.push_str(source.line_text(*line));
        statements.push('\n');
    }
    text.push_str(&block(&statements));
    // The functions of the project's modules that the statements call.
    let mut files: Vec<&str> = Vec::new();
    for (file, _, _) in &kept.module_functions {
        if !files.contains(&file.as_str()) {
            files.push(file);
        }
    }
    for file in files {
        let Ok(module) = fs.read_to_string(&project.root.join(file)) else {
            continue;
        };
        let module_source = Source::new(file, &module);
        let mut lines: BTreeSet<u32> = BTreeSet::new();
        for (_, start, end) in kept.module_functions.iter().filter(|(f, _, _)| f == file) {
            let first = module_source.line_of(*start as usize);
            let last =
                module_source.line_of((*end as usize).saturating_sub(1).max(*start as usize));
            lines.extend(first..=last);
            lines.extend(above(&module_source, first, "///"));
        }
        let mut body = String::new();
        let mut before: Option<u32> = None;
        for line in &lines {
            if before.is_some_and(|b| b + 1 != *line) {
                body.push('\n');
            }
            body.push_str(module_source.line_text(*line));
            body.push('\n');
            before = Some(*line);
        }
        text.push_str(&format!(
            "\nThe functions they call in `{file}` of the project:\n\n"
        ));
        text.push_str(&block(&body));
    }
    // The run: the seed, and what was said before the failure.
    text.push_str("\n## The run\n\n");
    let seeds: Vec<u32> = graph
        .statements
        .iter()
        .filter(|node| node.seeds)
        .map(|node| source.line_of(node.span.start))
        .filter(|line| kept.lines.contains(line))
        .collect();
    if seeds.is_empty() {
        text.push_str("- seed: none is set by these statements\n");
    } else {
        for line in &seeds {
            text.push_str(&format!(
                "- seed: set at line {line}, `{}`\n",
                source.line_text(*line).trim()
            ));
        }
    }
    let before = match kind {
        Kind::WrongResult => outcome.output.stderr.clone(),
        _ => outcome.output.stderr[..error_place(&outcome).0].to_string(),
    };
    if before.trim().is_empty() {
        text.push_str(match kind {
            Kind::WrongResult => "- warnings and notes: none\n",
            _ => "- warnings and notes before the failure: none\n",
        });
    } else {
        text.push_str(match kind {
            Kind::WrongResult => "- warnings and notes of these statements:\n\n",
            _ => "- warnings and notes before the failure:\n\n",
        });
        text.push_str(&block(&before));
    }
    // The data: the schema of every frame the kept statements bound, and the files read.
    text.push_str("\n## The data\n\n");
    // Each binding the kept statements make, the last of a name where a name is bound twice.
    let mut bound: Vec<(zeff_ir::BindingId, &str)> = Vec::new();
    for node in &graph.statements {
        if !kept.lines.contains(&source.line_of(node.span.start)) {
            continue;
        }
        if let Some(id) = node.defines
            && let Some(binding) = program.bindings.get(id)
        {
            bound.retain(|(_, name)| *name != binding.name);
            bound.push((id, &binding.name));
        }
    }
    let sized = |rows: usize, variables: usize| -> String {
        format!(
            "{} {} × {} {}",
            count(rows),
            if rows == 1 { "row" } else { "rows" },
            variables,
            if variables == 1 {
                "variable"
            } else {
                "variables"
            }
        )
    };
    // One line to a variable: its name, its type, and what else is known of it.
    let variables = |rows: &[(String, String, Vec<String>)]| -> String {
        let widest = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
        let mut out = String::new();
        for (name, ty, facts) in rows {
            let mut line = format!(
                "    {name}{}  {ty}",
                " ".repeat(widest - name.chars().count())
            );
            if !facts.is_empty() {
                line.push_str(&format!(
                    "{}  {}",
                    " ".repeat(8usize.saturating_sub(ty.chars().count())),
                    facts.join(", ")
                ));
            }
            out.push_str(&line);
            out.push('\n');
        }
        out
    };
    let mut schemas = String::new();
    for (id, name) in &bound {
        // The frame the run held; or, where the statement never ran, what the check knew of it.
        if let Some((_, frame)) = frames.iter().rev().find(|(held, _)| held == name) {
            let table = &frame.table;
            let rows: Vec<(String, String, Vec<String>)> = table
                .columns()
                .map(|(variable, column)| {
                    let mut facts: Vec<String> = Vec::new();
                    if let Some(levels) = column.levels() {
                        facts.push(format!("{} levels", count(levels.len())));
                    }
                    if column.n_missing() > 0 {
                        facts.push(format!("{} missing", count(column.n_missing())));
                    }
                    (variable.clone(), column.ty().name().to_string(), facts)
                })
                .collect();
            schemas.push_str(&format!(
                "`{name}`: {}\n\n{}\n",
                sized(table.rows(), table.names().len()),
                variables(&rows)
            ));
        } else if let Some((_, schema)) = program.bound_schemas.iter().find(|(b, _)| b == id) {
            let rows: Vec<(String, String, Vec<String>)> = schema
                .columns
                .iter()
                .map(|(variable, ty)| {
                    let fact = schema.facts.iter().find(|f| f.name == *variable);
                    let kind = match fact {
                        Some(fact) if !fact.kind.is_empty() => fact.kind.clone(),
                        _ if *ty == zeff_ir::Ty::Unknown => "(known when run)".to_string(),
                        _ => ty.to_string(),
                    };
                    let facts = match fact.and_then(|f| f.missing) {
                        Some(missing) if missing > 0 => {
                            vec![format!("{} missing", count(missing as usize))]
                        }
                        _ => Vec::new(),
                    };
                    (variable.clone(), kind, facts)
                })
                .collect();
            let size = match schema.rows {
                Some(n) => sized(n as usize, schema.columns.len()),
                None => format!(
                    "{} {}",
                    schema.columns.len(),
                    if schema.columns.len() == 1 {
                        "variable"
                    } else {
                        "variables"
                    }
                ),
            };
            schemas.push_str(&format!(
                "`{name}`: {size}, as the check of the script knew it (the statement did not run)\n\n{}\n",
                variables(&rows)
            ));
        }
    }
    if schemas.is_empty() {
        text.push_str("The statements kept hold no frame.\n");
    } else {
        text.push_str(
            "The frames the statements hold: the names and types of their variables and how many\nvalues are missing. No value is written here.\n\n",
        );
        text.push_str(schemas.trim_end_matches('\n'));
        text.push('\n');
    }
    // The files read: those of the project by their path in it, others by their name alone.
    // Where the statements never ran (an error the check found), the files they import are
    // read off the statements themselves.
    let mut read = read;
    {
        let mut tell = |found: zeff_ir::walk::Found<'_>| {
            let zeff_ir::walk::Found::Expr(expr) = found else {
                return;
            };
            let zeff_ir::ExprKind::Call(call) = &expr.kind else {
                return;
            };
            if call.name != "import"
                || !kept.lines.contains(&source.line_of(call.callee_span.start))
            {
                return;
            }
            if let Some(Some(first)) = call.args.first()
                && let zeff_ir::ExprKind::Str(written) = &first.kind
                && let Ok(path) = project.resolve_read(fs, written)
                && fs.is_file(&path)
                && !read.contains(&path)
                && !written_by_the_run.contains(&path)
            {
                read.push(path);
            }
        };
        zeff_ir::walk::walk(program, asked.script, true, &mut tell);
    }
    let mut data: Vec<(String, PathBuf)> = Vec::new();
    for path in &read {
        let shown = match path.strip_prefix(&project.root) {
            Ok(inside) => {
                let inside = inside.to_string_lossy().replace('\\', "/");
                if inside == "zeff.toml"
                    || inside.starts_with("scripts/")
                    || inside.starts_with("functions/")
                    || inside.starts_with(".zeff/")
                {
                    continue;
                }
                inside
            }
            Err(_) => match path.file_name() {
                Some(name) => format!("{} (a file outside the project)", name.to_string_lossy()),
                None => continue,
            },
        };
        if !data.iter().any(|(name, _)| *name == shown) {
            data.push((shown, path.clone()));
        }
    }
    data.sort();
    let mut files: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    if data.is_empty() {
        text.push_str("\nThe statements read no file.\n");
    } else {
        text.push_str("\nThe files the statements read:\n\n");
        for (shown, path) in &data {
            let size = fs.metadata(path).map_or(0, |info| info.size);
            text.push_str(&format!("- `{shown}`, {} bytes\n", count(size as usize)));
            files.push((
                shown.clone(),
                if asked.include_data {
                    fs.read(path).ok()
                } else {
                    None
                },
            ));
        }
        text.push_str(if asked.include_data {
            "\nThey were written beside this report as they are, because `--include-data` asked for\nthem. They hold your data: read them too before you attach them.\n"
        } else {
            "\nThey are not in this report. `zeff report-bug --include-data` writes them beside it,\nfor a report whose reader needs the rows to see the failure.\n"
        });
    }
    let stem = Path::new(asked.script)
        .file_stem()
        .map_or("script".to_string(), |s| s.to_string_lossy().into_owned());
    let tag = match (kind, &code) {
        (Kind::WrongResult, _) => "wrong-result".to_string(),
        (_, Some(code)) => code.clone(),
        (Kind::InternalError, None) => "internal-error".to_string(),
        (Kind::Error, None) => "error".to_string(),
    };
    let name = format!("{stem}-{tag}-{:08x}.md", digest(&text) as u32);
    Ok(Report {
        kind,
        code,
        name,
        text,
        kept: kept.statements.len(),
        of,
        reduced,
        files,
    })
}

/// The folder of the reports, in the project: `.zeff/reports`.
pub const FOLDER: &str = ".zeff/reports";

/// Write a report into the project's `.zeff/reports/`, and with it the files it brings, in a
/// folder named after it. What comes back is every file written, by its path in the project,
/// the report first.
pub fn write(fs: &dyn FileSystem, project: &Project, report: &Report) -> io::Result<Vec<String>> {
    let make = |relative: &str| -> io::Result<()> {
        let mut at = project.root.clone();
        for part in relative.split('/') {
            at = at.join(part);
            if !fs.is_dir(&at) {
                fs.create_dir(&at)?;
            }
        }
        Ok(())
    };
    make(FOLDER)?;
    let mut written = vec![format!("{FOLDER}/{}", report.name)];
    fs.write(&project.root.join(&written[0]), report.text.as_bytes())?;
    let beside = format!("{FOLDER}/{}-data", report.name.trim_end_matches(".md"));
    for (shown, bytes) in &report.files {
        let Some(bytes) = bytes else {
            continue;
        };
        // A file outside the project goes by its name alone.
        let inside = shown
            .strip_suffix(" (a file outside the project)")
            .unwrap_or(shown);
        let relative = format!("{beside}/{inside}");
        if let Some((folder, _)) = relative.rsplit_once('/') {
            make(folder)?;
        }
        fs.write(&project.root.join(&relative), bytes)?;
        written.push(relative);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{Output, StatementOutput};

    fn outcome(stderr: &str, last: Option<(u32, &str)>) -> Outcome {
        Outcome {
            output: Output {
                stderr: stderr.to_string(),
                statements: last
                    .map(|(line, stderr)| StatementOutput {
                        line,
                        stdout: String::new(),
                        stderr: stderr.to_string(),
                    })
                    .into_iter()
                    .collect(),
                ..Output::default()
            },
            exit_code: 1,
        }
    }

    /// The error a report is of: the one the failing statement said, the last thing the run
    /// said; or, where the check refused the script and said several, the first of them.
    #[test]
    fn the_error_reported_is_the_statements_or_the_first_of_the_checks() {
        let a = "error[Z0101]: one\n --> scripts/main.zeff:4:2\n";
        let b = "error[Z0103]: two\n --> scripts/main.zeff:9:1\n";
        let warned = "warning[W0201]: 3 rows with NA in x were skipped (line 2)\n";
        // The check: nothing ran.
        let checked = outcome(&format!("{a}{b}"), None);
        assert_eq!(error_of(&checked), a);
        assert_eq!(failing_line(&checked, "scripts/main.zeff"), Some(4));
        // A statement failed after a warning.
        let ran = outcome(&format!("{warned}{b}"), Some((9, b)));
        assert_eq!(error_of(&ran), b);
        assert_eq!(error_place(&ran).0, warned.len());
        assert_eq!(failing_line(&ran, "scripts/main.zeff"), Some(9));
        // No error said.
        let fine = outcome(warned, Some((2, warned)));
        assert_eq!(error_of(&fine), "");
        assert_eq!(code_of(a).as_deref(), Some("Z0101"));
        assert_eq!(code_of("error: no code"), None);
        assert_eq!(lines_text(&[4]), "line 4");
        assert_eq!(lines_text(&[1, 2, 7]), "lines 1, 2 and 7");
        assert_eq!(count(1_234_567), "1,234,567");
    }
}
