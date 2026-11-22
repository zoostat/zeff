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
//! statements read are written beside it, shrunk to the lines the failure needs, and with
//! `--synthetic` with made-up values where the failure persists with them (step 20b; the
//! shrinker's parts are in `shrink.rs`).
//!
//! Three kinds of report: an error of the script, an internal error (a bug in Zeff), and a
//! result believed wrong, where nothing failed: its statement is named by its line and the
//! reference it should agree with is given as a file.
//!
//! Nothing here is sent anywhere, and nothing here reads a clock: the same failure gives the
//! same file, under the same name.

use crate::control::Control;
use crate::dependencies::{Graph, graph};
use crate::fs::FileSystem;
use crate::project::Project;
use crate::run::{Frames, Outcome, RunOptions, check_in_project, run_script_keeping};
use crate::shrink::{
    Budget, Cells, Overlay, by_halves, by_halves_keeping_the_first, lines_of, numbers_apart,
};
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
    /// `--include-data`: the files the kept statements read come with the report, shrunk to
    /// the lines the failure needs.
    pub include_data: bool,
    /// `--synthetic`: the shrunk files come with the report, their values made up where the
    /// failure persists with made-up values.
    pub synthetic: bool,
    /// How many trial runs the shrinking may make ([`TRIALS`] unless a test says otherwise).
    pub trials: usize,
    /// The machine, as the report names it (`aarch64 macos`).
    pub platform: &'a str,
    pub options: RunOptions,
}

/// How many trial runs the shrinking of one report may make: a count, not a time, so that the
/// same failure shrinks to the same reproducer on every machine. A halving of a file's lines
/// takes about two trials, so a file of a million lines with one line at fault takes about
/// forty-five; ten lines at fault spread over a file take a few hundred. The trials after the
/// first few run on little data.
pub const TRIALS: usize = 500;

/// What the shrinking did to one data file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileShrunk {
    /// The file as the report names it, and where it is.
    pub name: String,
    pub path: PathBuf,
    /// How many lines it has, and how many are kept.
    pub lines: usize,
    pub kept: usize,
    /// With `--synthetic`: the variables whose values were made up, and those whose values
    /// are the file's own, the failure not persisting without them.
    pub made_up: Vec<String>,
    pub real: Vec<String>,
}

/// What the shrinking did: to the data files, where the data is written, and to the
/// statements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shrunk {
    pub files: Vec<FileShrunk>,
    /// The lines of the statements the graph kept that were left out after all, the failure
    /// persisting without them.
    pub statements: Vec<u32>,
    /// How many trial runs were made, and whether the shrinking stopped at its limit.
    pub trials: usize,
    pub stopped: bool,
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
    /// file outside it), with their bytes when `--include-data` or `--synthetic` asked for
    /// them: shrunk, and with made-up values, as far as the failure allowed.
    pub files: Vec<(String, Option<Vec<u8>>)>,
    /// What the shrinking did.
    pub shrunk: Shrunk,
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

/// One run of a script for the report: its outcome, the frames its bindings held, the files
/// it read that it had not written itself, and those it wrote.
struct Run {
    outcome: Outcome,
    frames: Frames,
    read: Vec<PathBuf>,
    written: Vec<PathBuf>,
}

/// A run of `text` over the project's files, with the files `given` standing for the
/// project's own of their paths. What the run writes stays in memory: a report is built
/// without a file of the project being touched.
fn run_over(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
    given: &[(PathBuf, Vec<u8>)],
) -> Run {
    let overlay = Overlay::new(fs, given);
    let (outcome, frames) = run_script_keeping(
        &overlay,
        project,
        script,
        text,
        options,
        &Control::new(),
        None,
    );
    let (read, written) = overlay.into_read_and_written();
    Run {
        outcome,
        frames,
        read,
        written,
    }
}

/// The data files of a run of the lines kept: the files it read, and, where its statements
/// never ran (an error the check found), the files their `import` calls name. Each by the name
/// the report gives it, a file of the project by its path in it and any other by its name
/// alone, with its place; the project's own files (its `zeff.toml`, scripts, functions and
/// `.zeff`) and the files the run wrote itself are no data.
#[allow(clippy::too_many_arguments)]
fn data_files(
    fs: &dyn FileSystem,
    project: &Project,
    program: &zeff_ir::Program,
    source: &Source,
    script: &str,
    lines: &BTreeSet<u32>,
    run: &Run,
) -> Vec<(String, PathBuf)> {
    let mut read = run.read.clone();
    {
        let mut tell = |found: zeff_ir::walk::Found<'_>| {
            let zeff_ir::walk::Found::Expr(expr) = found else {
                return;
            };
            let zeff_ir::ExprKind::Call(call) = &expr.kind else {
                return;
            };
            if call.name != "import" || !lines.contains(&source.line_of(call.callee_span.start)) {
                return;
            }
            if let Some(Some(first)) = call.args.first()
                && let zeff_ir::ExprKind::Str(written) = &first.kind
                && let Ok(path) = project.resolve_read(fs, written)
                && fs.is_file(&path)
                && !read.contains(&path)
                && !run.written.contains(&path)
            {
                read.push(path);
            }
        };
        zeff_ir::walk::walk(program, script, true, &mut tell);
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
    data
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
    let run = |text: &str, given: &[(PathBuf, Vec<u8>)]| -> Run {
        run_over(fs, project, asked.script, text, &asked.options, given)
    };
    let whole = run(asked.text, &[]).outcome;
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
    let lines_for = |statements: &[usize]| -> Kept {
        kept_lines(
            &source,
            asked.script,
            asked.text,
            program,
            &graph,
            statements,
            &graph.called_by(statements),
        )
    };
    let mut reduced = about.is_some();
    let mut statements: Vec<usize> = match about {
        Some(index) => graph.needed_by(index).statements,
        None => all_until(None),
    };
    let mut kept = lines_for(&statements);
    // A script that could not be read into statements is kept whole.
    if of == 0 {
        kept.lines = (1..=source.line_count() as u32).collect();
        reduced = false;
    }
    // Whether the failure persists in a run of the lines given over the files given: the same
    // exit code, and the same error by its code and its message, the numbers in the message
    // set apart (a message that names a row or counts rows changes as rows are removed). A
    // result believed wrong persists only as the very same printed result.
    let persists = |lines: &BTreeSet<u32>, given: &[(PathBuf, Vec<u8>)]| -> bool {
        let trial = run(&blanked(&source, lines), given).outcome;
        if trial.exit_code != whole.exit_code {
            return false;
        }
        let now = failure(&trial);
        match kind {
            Kind::WrongResult => now == said,
            _ => code_of(&now) == code_of(&said) && numbers_apart(&now) == numbers_apart(&said),
        }
    };
    if reduced && !persists(&kept.lines, &[]) {
        reduced = false;
        statements = all_until(about);
        kept = lines_for(&statements);
    }

    // ----- shrinking (step 20b) -----
    // Rows first, where the data is to be written; then statements; then, where they are
    // asked for, made-up values. Each trial is a run of the kept lines, and a change is kept
    // only when the failure persists. A result believed wrong keeps its rows: its reference
    // was computed on them.
    let mut budget = Budget::of(asked.trials);
    let mut given: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut account = Shrunk::default();
    let with_data = (asked.include_data || asked.synthetic) && kind != Kind::WrongResult;
    if with_data {
        let first = run(&blanked(&source, &kept.lines), &[]);
        let files = data_files(
            fs,
            project,
            program,
            &source,
            asked.script,
            &kept.lines,
            &first,
        );
        for (shown, path) in files {
            let Some(text) = fs.read(&path).ok().and_then(|b| String::from_utf8(b).ok()) else {
                // Not a text: it cannot be shrunk by lines, and is kept as it is.
                continue;
            };
            let lines = lines_of(&text);
            let all: Vec<u32> = (0..lines.len() as u32).collect();
            let content = |keep: &[u32]| -> Vec<u8> {
                let mut out = String::new();
                for index in keep {
                    out.push_str(lines[*index as usize]);
                }
                out.into_bytes()
            };
            let left = by_halves_keeping_the_first(all, &mut budget, &mut |keep| {
                let mut trial = given.clone();
                trial.push((path.clone(), content(keep)));
                persists(&kept.lines, &trial)
            });
            if left.len() < lines.len() {
                given.push((path.clone(), content(&left)));
            }
            account.files.push(FileShrunk {
                name: shown,
                path,
                lines: lines.len(),
                kept: left.len(),
                made_up: Vec::new(),
                real: Vec::new(),
            });
        }
    }
    if let Some(index) = about
        && of > 0
    {
        // A statement that sets the seed is never tried: without it a draw takes a seed of
        // the system's choosing, and a trial would say one thing today and another tomorrow.
        let held: Vec<usize> = statements
            .iter()
            .copied()
            .filter(|i| *i == index || graph.statements[*i].seeds)
            .collect();
        let others: Vec<usize> = statements
            .iter()
            .copied()
            .filter(|i| !held.contains(i))
            .collect();
        let together = |others: &[usize]| -> Vec<usize> {
            let mut all = others.to_vec();
            all.extend(held.iter().copied());
            all.sort_unstable();
            all
        };
        let left = by_halves(others.clone(), &mut budget, &mut |others| {
            persists(&lines_for(&together(others)).lines, &given)
        });
        if left.len() < others.len() {
            account.statements = others
                .iter()
                .filter(|i| !left.contains(i))
                .map(|i| source.line_of(graph.statements[*i].span.start))
                .collect();
            statements = together(&left);
            kept = lines_for(&statements);
        }
    }
    if asked.synthetic && with_data {
        for file in account.files.iter_mut() {
            let now: Vec<u8> = match given.iter().find(|(path, _)| *path == file.path) {
                Some((_, bytes)) => bytes.clone(),
                None => match fs.read(&file.path) {
                    Ok(bytes) => bytes,
                    Err(_) => continue,
                },
            };
            let Some(cells) = String::from_utf8(now).ok().and_then(|t| Cells::read(&t)) else {
                continue;
            };
            let columns: Vec<usize> = (0..cells.columns()).collect();
            let others: Vec<(PathBuf, Vec<u8>)> = given
                .iter()
                .filter(|(path, _)| *path != file.path)
                .cloned()
                .collect();
            let real = by_halves(columns.clone(), &mut budget, &mut |real| {
                let mut trial = others.clone();
                trial.push((file.path.clone(), cells.written(real).into_bytes()));
                persists(&kept.lines, &trial)
            });
            if real.len() < columns.len() {
                given = others;
                given.push((file.path.clone(), cells.written(&real).into_bytes()));
            }
            file.made_up = columns
                .iter()
                .filter(|c| !real.contains(c))
                .map(|c| cells.name(*c))
                .collect();
            file.real = real.iter().map(|c| cells.name(*c)).collect();
        }
    }
    account.trials = asked.trials - budget.left;
    account.stopped = budget.spent;
    // The run the report describes: the kept lines over the files as they now are.
    let last = run(&blanked(&source, &kept.lines), &given);
    let data = data_files(
        fs,
        project,
        program,
        &source,
        asked.script,
        &kept.lines,
        &last,
    );
    let Run {
        outcome, frames, ..
    } = last;

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
    let with_files = asked.include_data || asked.synthetic;
    text.push_str(match (with_files, asked.synthetic && with_data) {
        (false, _) => "It holds no rows of your data.\n",
        (true, false) => {
            "It holds no rows of your data itself; the files named at its end were written beside it\nbecause `--include-data` asked for them, and they hold rows of your data.\n"
        }
        (true, true) => {
            "It holds no rows of your data itself; the files named at its end were written beside it\nbecause `--synthetic` asked for them, with made-up values where the failure allows; the\nsection on the data says which values are still yours.\n"
        }
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
    // "Run alone, they fail in the same way", or of one statement, "it fails".
    let alone_same = match (kind, kept.statements.len()) {
        (Kind::WrongResult, 1) => "Run alone, it prints the same result",
        (Kind::WrongResult, _) => "Run alone, they print the same result",
        (_, 1) => "Run alone, it fails in the same way",
        (_, _) => "Run alone, they fail in the same way",
    };
    if of == 0 {
        text.push_str(&format!(
            "`{}` could not be read into statements, so the script is kept whole:\n\n",
            asked.script
        ));
    } else if reduced {
        text.push_str(&format!(
            "{what}: {} of the {} statements of\n`{}` ({}). {alone_same}.\n\n",
            kept.statements.len(),
            of,
            asked.script,
            lines_text(&kept.statements)
        ));
    } else if about.is_some() {
        text.push_str(&format!(
            "The statements the graph of the script gives did not {same} when run alone:\none of them depends on another through a file or the state of the run, which the graph\ndoes not see. So every statement up to it was tried, and those the failure persists\nwithout were left out: {} of the {} statements of `{}` are kept\n({}).\n\n",
            kept.statements.len(),
            of,
            asked.script,
            lines_text(&kept.statements)
        ));
    } else {
        text.push_str(&format!(
            "The error is not of one statement, so every statement of `{}` is kept: {}.\n\n",
            asked.script,
            count(kept.statements.len())
        ));
    }
    if reduced && !account.statements.is_empty() {
        text.push_str(&match account.statements.as_slice() {
            [line] => format!(
                "One more that the graph of the script gives, at line {line}, was left out: the failure\npersists without it.\n\n"
            ),
            lines => format!(
                "{} more that the graph of the script gives, at {}, were left out: the\nfailure persists without them.\n\n",
                lines.len(),
                lines_text(lines)
            ),
        });
    }
    let mut shown_statements = String::new();
    for line in &kept.lines {
        shown_statements.push_str(source.line_text(*line));
        shown_statements.push('\n');
    }
    text.push_str(&block(&shown_statements));
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
        text.push_str(if given.is_empty() {
            "The frames the statements hold: the names and types of their variables and how many\nvalues are missing. No value is written here.\n\n"
        } else {
            "The frames the statements hold, on the files as they were shrunk: the names and types of\ntheir variables and how many values are missing. No value is written here.\n\n"
        });
        text.push_str(schemas.trim_end_matches('\n'));
        text.push('\n');
    }
    // The files read, and what the shrinking did to them.
    let mut files: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    let listed = |words: &[String]| -> String {
        let quoted: Vec<String> = words.iter().map(|w| format!("`{w}`")).collect();
        match quoted.as_slice() {
            [] => String::new(),
            [one] => one.clone(),
            [most @ .., last] => format!("{} and {last}", most.join(", ")),
        }
    };
    if data.is_empty() {
        text.push_str("\nThe statements read no file.\n");
    } else {
        text.push_str("\nThe files the statements read:\n\n");
        for (shown, path) in &data {
            let bytes = match given.iter().find(|(given, _)| given == path) {
                Some((_, bytes)) => Some(bytes.clone()),
                None => fs.read(path).ok(),
            };
            let size = bytes.as_ref().map_or(0, Vec::len);
            let shrunk = account.files.iter().find(|file| file.path == *path);
            let mut line = match shrunk {
                Some(file) if file.kept < file.lines => format!(
                    "- `{shown}`: {} of its {} lines are kept, {} bytes; the failure persists without the others",
                    count(file.kept),
                    count(file.lines),
                    count(size)
                ),
                Some(file) if with_data => format!(
                    "- `{shown}`: every one of its {} lines is kept, {} bytes; the failure does not persist without them",
                    count(file.lines),
                    count(size)
                ),
                _ => format!("- `{shown}`, {} bytes", count(size)),
            };
            if let Some(file) = shrunk
                && asked.synthetic
            {
                line.push_str(&match (file.made_up.is_empty(), file.real.is_empty()) {
                    (true, true) => String::new(),
                    (false, true) => ". Every value in it is made up".to_string(),
                    (true, false) => {
                        ". Its values are your own: the failure does not persist with made-up ones"
                            .to_string()
                    }
                    (false, false) => format!(
                        ". The values of {} are made up; those of {} are your own, since the failure does not persist without them",
                        listed(&file.made_up),
                        listed(&file.real)
                    ),
                });
            }
            line.push('\n');
            text.push_str(&line);
            files.push((shown.clone(), if with_files { bytes } else { None }));
        }
        text.push_str(match (with_files, with_data, asked.synthetic) {
            (false, _, _) if kind == Kind::WrongResult => {
                "\nThey are not in this report. `zeff report-bug --include-data` writes them beside it as they\nare, for a report whose reader needs the rows to compute the result.\n"
            }
            (false, _, _) => {
                "\nThey are not in this report. `zeff report-bug --include-data` writes them beside it, shrunk\nto the lines the failure needs, and `--synthetic` does the same with made-up values, for a\nreport whose reader needs the rows to see the failure.\n"
            }
            (true, false, _) => {
                "\nThey were written beside this report as they are: the reference of a result believed wrong\nwas computed on every row. They hold your data: read them too before you attach them.\n"
            }
            (true, true, false) => {
                "\nThey were written beside this report as said above, because `--include-data` asked for\nthem. What is kept of them is your data: read them too before you attach them.\n"
            }
            (true, true, true) => {
                "\nThey were written beside this report as said above, because `--synthetic` asked for them.\nA made-up value stands for one of yours, the same value by the same made-up one within its\nvariable: a whole number by a whole number, a number by a number, a text by a text. The\nfirst line of each file, which names its variables, and the cells that hold no value are\nas they were. Read the files too before you attach them.\n"
            }
        });
    }
    if account.stopped {
        text.push_str(&format!(
            "\nThe shrinking stopped at its limit of {} trial runs: a smaller reproducer may exist.\n",
            count(asked.trials)
        ));
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
        shrunk: account,
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
                        result: None,
                        drew: false,
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
