//! Running a script of a project: parse, check, interpret (Engine and Project Specification,
//! section 2, `zeff run`). What the script prints goes to standard output; diagnostics go to
//! standard error; the exit code follows Language Reference section 3.8.

use crate::fs::FileSystem;
use crate::interpreter::{Interpreter, Output};
use crate::paths::PathProblem;
use crate::pragmas::{Allowed, Allows};
use crate::project::Project;
use crate::registry::{self, Bodies, ZeffBodies};
use std::path::Path;
use zeff_core::runtime::{Fail, Files, fail};
use zeff_diagnostics::{Diagnostic, Severity, Source, Sources, Span};
use zeff_registry::Registry;

/// The files of a project, as a running script may read them: by the path rules of section 2.
/// With the schema cache given, a data root's file is hashed as it is read and a change since
/// the last run is warned about (W0701) after the statement that read it.
pub struct ProjectFiles<'a> {
    pub fs: &'a dyn FileSystem,
    pub project: &'a Project,
    pub schemas: Option<&'a crate::schemas::SchemaCache<'a>>,
    pub pending: std::cell::RefCell<Vec<Diagnostic>>,
}

impl<'a> ProjectFiles<'a> {
    pub fn new(fs: &'a dyn FileSystem, project: &'a Project) -> ProjectFiles<'a> {
        ProjectFiles {
            fs,
            project,
            schemas: None,
            pending: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl Files for ProjectFiles<'_> {
    fn now(&self) -> Option<i64> {
        self.fs.now()
    }

    fn take_messages(&self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.pending.borrow_mut())
    }

    fn read(&self, written: &str, span: Span, script: &str) -> Result<Vec<u8>, Fail> {
        // The `zeff:` data root: the files of the shipped datasets (section 8.5).
        if let Some(name) = written.strip_prefix("zeff:") {
            return zeff_datasets::file(name)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| {
                    let files = zeff_datasets::files();
                    let mut d = Diagnostic::error(
                        "Z0715",
                        format!("no file `zeff:{name}` ships with Zeff"),
                    )
                    .in_file(script)
                    .primary(span, "not a shipped dataset file");
                    d = match zeff_diagnostics::did_you_mean(name, files.iter().map(String::as_str))
                    {
                        Some(near) => d.help(format!("did you mean `zeff:{near}`?")),
                        None => d.help(format!(
                            "the `zeff:` root holds: {}; `datasets()` lists the datasets",
                            files.join(", ")
                        )),
                    };
                    fail(d)
                });
        }
        let path = self
            .project
            .resolve_read(self.fs, written)
            .map_err(|p| fail(p.diagnostic(written, span, script)))?;
        if self.fs.is_dir(&path) {
            return Err(fail(
                Diagnostic::error("Z0715", format!("`{written}` is a folder, not a file"))
                    .in_file(script)
                    .primary(span, "a folder")
                    .help(format!(
                        "write the path of a file inside it, as in `{}/wages.csv`",
                        written.trim_end_matches('/')
                    )),
            ));
        }
        if !self.fs.is_file(&path) {
            return Err(fail(
                Diagnostic::error("Z0715", format!("no file `{written}` in the project"))
                    .in_file(script)
                    .primary(span, "the file is not there")
                    .help(format!("put the file in the project and write its path from the project folder, as in `data/{}`", Path::new(written).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_else(|| "wages.csv".into()))),
            ));
        }
        let bytes = self.fs.read(&path).map_err(|e| {
            fail(
                Diagnostic::error("Z0715", format!("cannot read `{written}`: {e}"))
                    .in_file(script)
                    .primary(span, "the file could not be read"),
            )
        })?;
        // A data root's file: hashed, and compared with what the last run read (section 2,
        // "Data roots"; W0701 after the statement).
        if let Some(schemas) = self.schemas
            && crate::paths::split_root(written).is_some()
        {
            let hash = zeff_hash::hash_hex(&bytes);
            let modified = self.fs.metadata(&path).ok().and_then(|m| m.modified);
            if let Some((_, previous)) = schemas.root_read(written, &hash, modified) {
                // The time to the second, as a clock on the wall shows it.
                let when = |t: Option<i64>| -> String {
                    t.map_or("an unknown time".to_string(), |t| {
                        let iso = zeff_core::display::cell(zeff_columns::Value::DateTime(t));
                        iso.split('.').next().unwrap_or(&iso).replace('T', " ")
                    })
                };
                self.pending.borrow_mut().push(
                    Diagnostic::warning(
                        "W0701",
                        format!(
                            "`{written}` has changed since the last run: it was read on {}, and the file is now from {}",
                            when(previous),
                            when(modified)
                        ),
                    )
                    .in_file(script)
                    .primary(span, "not the file the last run read")
                    .help("the results may differ from the last run's; the run records of release 0.11.0 will show how"),
                );
            }
        }
        Ok(bytes)
    }

    /// Only under the project folder (section 2, "Writes"): `output/` is created as needed, any
    /// other folder must exist, and a path outside the project, in a data root or through a link
    /// is Z0707.
    fn write(&self, written: &str, bytes: &[u8], span: Span, script: &str) -> Result<(), Fail> {
        let path = self
            .project
            .resolve_write(self.fs, written)
            .map_err(|p| fail(p.diagnostic(written, span, script)))?;
        let unwritable =
            |why: String| fail(PathProblem::Unwritable(why).diagnostic(written, span, script));
        if let Some(folder) = path.parent()
            && !self.fs.is_dir(folder)
        {
            if folder == self.project.root.join("output") {
                self.fs
                    .create_dir(folder)
                    .map_err(|e| unwritable(format!("`output/` could not be created ({e})")))?;
            } else {
                let name = written.rsplit_once('/').map_or("", |(f, _)| f);
                return Err(unwritable(format!(
                    "the folder `{name}/` does not exist; `output/` is created as needed, another folder you create first"
                )));
            }
        }
        self.fs
            .write(&path, bytes)
            .map_err(|e| unwritable(e.to_string()))
    }
}

/// A script checked with its project's modules (section 5.3, "Modules"): the lowered program
/// and every diagnostic, the modules' sources for rendering them.
pub struct CheckedInProject {
    pub checked: zeff_ir::Checked,
    pub module_sources: Vec<Source>,
    /// The warnings the pragmas of the script and the modules allow (section 11.1).
    pub allows: Allows,
    /// The warnings of the check that a pragma silenced; they are not among the diagnostics.
    pub allowed: Vec<Allowed>,
}

/// Parse and check `text` as `script`, with the modules of `functions/`; the diagnostics come
/// in the order of the files (the modules first, by name, then the script), each file's by
/// position.
pub fn check_in_project(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    registry: &Registry,
) -> CheckedInProject {
    let loaded = crate::modules::load(fs, project);
    let parsed = zeff_syntax::parse(script, text);
    // The schema pass (section 11, "Checking before running"): the schemas of the files the
    // script imports, from the cache or read now, so that variables are checked before the run.
    let schemas = crate::schemas::SchemaCache::new(fs, project);
    let mut checked = zeff_ir::check_project_with(
        script,
        &parsed.script,
        registry,
        Some(text),
        &loaded.modules,
        Some(&schemas),
    );
    let _ = schemas.save();
    let module_sources = loaded.sources();
    // The pragmas (section 11.1): a code no warning has is Z0317; a warning of the check that
    // points inside a statement whose pragma allows it is taken out and recorded.
    let mut allows = Allows::default();
    for (file, module_allows) in loaded.allows {
        allows.add(&file, module_allows);
    }
    let unknown = crate::pragmas::unknown_codes(script, &parsed.allows);
    allows.add(script, parsed.allows);
    let mut all: Vec<Diagnostic> = loaded.diagnostics;
    all.extend(parsed.diagnostics.into_vec());
    all.extend(unknown);
    all.extend(checked.diagnostics.into_vec());
    let mut allowed = Vec::new();
    if !allows.is_empty() {
        let script_source = Source::new(script, text);
        all.retain(|d| {
            if !crate::pragmas::allowable(d) {
                return true;
            }
            let file = d.file.as_deref().unwrap_or(script);
            let Some(at) = d.labels.first().map(|l| l.span.start) else {
                return true;
            };
            let Some(allow) = allows.covering(file, at, &d.code) else {
                return true;
            };
            let source = module_sources
                .iter()
                .find(|s| s.name() == file)
                .unwrap_or(&script_source);
            allowed.push(Allowed::new(allow, file, source, d));
            false
        });
    }
    let order_of = |d: &Diagnostic| -> (usize, usize) {
        let file = d.file.clone().unwrap_or_default();
        let position = loaded
            .modules
            .iter()
            .position(|m| m.file == file)
            .unwrap_or(loaded.modules.len());
        (
            position,
            d.labels.first().map_or(usize::MAX, |l| l.span.start),
        )
    };
    all.sort_by_key(order_of);
    let mut diagnostics = zeff_diagnostics::Diagnostics::new();
    for d in all {
        diagnostics.push(d);
    }
    checked.diagnostics = diagnostics;
    CheckedInProject {
        checked,
        module_sources,
        allows,
        allowed,
    }
}

/// The result of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub output: Output,
    /// 0 for success, 1 for an error in the script, 2 for an internal error, 3 for a warning
    /// that `--deny-warnings` made an error (Language Reference, section 3.8, "Exit codes").
    pub exit_code: i32,
}

/// Run the text of a script called `script` (a project-relative name) in `project`.
/// How a run is made; the defaults are what `zeff run` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOptions {
    /// The page width frames and tables wrap at (`--width`; 118 by default).
    pub width: usize,
    /// Whether a binding is freed after its last use in the script (section 6, "Memory"): so in
    /// `zeff run`; a session keeps its bindings.
    pub free_bindings: bool,
    /// A test hook: the statement at this line panics before it runs (see `Interpreter`).
    #[doc(hidden)]
    pub panic_at_line: Option<u32>,
    /// Whether column expressions inside frame methods run as fused passes (Engine and Project
    /// Specification, section 8.3, point 10): so by default; with the environment variable
    /// `ZEFF_UNFUSED` set they run through the tree-walking evaluation, the oracle the fused
    /// passes are tested against, which gives the same bits and messages.
    pub fused: bool,
    /// `--deny-warnings` (Language Reference, section 11.1): a warning that no pragma allows
    /// is an error; the run stops at the first, with exit code 3.
    pub deny_warnings: bool,
    /// `zeff test`: a failed `assert` is kept among the output's checks and the script
    /// carries on, so that every check is reported.
    pub collect_checks: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        RunOptions {
            width: zeff_core::display::PAGE_WIDTH,
            free_bindings: true,
            panic_at_line: None,
            fused: std::env::var_os("ZEFF_UNFUSED").is_none(),
            deny_warnings: false,
            collect_checks: false,
        }
    }
}

pub fn run_script(fs: &dyn FileSystem, project: &Project, script: &str, text: &str) -> Outcome {
    run_script_with(fs, project, script, text, &RunOptions::default())
}

pub fn run_script_with(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
) -> Outcome {
    let registry = match registry::registry() {
        Ok(registry) => registry,
        Err(errors) => {
            let mut output = Output::default();
            output.stderr.push_str(
                "error[Z9001]: internal error: this is a bug in Zeff, not in your script\n",
            );
            for problem in errors {
                output.stderr.push_str(&format!("  {problem}\n"));
            }
            return Outcome {
                output,
                exit_code: 2,
            };
        }
    };
    run_script_in(
        fs,
        project,
        script,
        text,
        options,
        registry,
        registry::bodies(),
        registry::zeff_bodies(),
    )
}

/// [`run_script_with`] under a handle and a listener: the run tells the listener each
/// statement, each pass of a loop and what each statement printed, and stops with Z0705 before
/// its next step once the handle has been cancelled (section 8.3, point 7).
pub fn run_script_under(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
    control: &crate::control::Control,
    listener: Option<crate::control::Listener<'_>>,
) -> Outcome {
    let registry = match registry::registry() {
        Ok(registry) => registry,
        Err(_) => return run_script_with(fs, project, script, text, options),
    };
    run_controlled(
        fs,
        project,
        script,
        text,
        options,
        registry,
        registry::bodies(),
        registry::zeff_bodies(),
        Some(control),
        listener,
    )
}

/// [`run_script_with`] against a given registry and the bodies that run its functions, Rust and
/// Zeff: what the tests use to run a script with a fixture function beside the engine's own.
#[allow(clippy::too_many_arguments)]
pub fn run_script_in(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
    registry: &Registry,
    bodies: &Bodies,
    zeff: &ZeffBodies,
) -> Outcome {
    run_controlled(
        fs, project, script, text, options, registry, bodies, zeff, None, None,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_controlled(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
    options: &RunOptions,
    registry: &Registry,
    bodies: &Bodies,
    zeff: &ZeffBodies,
    control: Option<&crate::control::Control>,
    listener: Option<crate::control::Listener<'_>>,
) -> Outcome {
    // What the check says before the run starts, and what stops it before it starts, is told
    // as it is said, so that the pieces a listener is given are the whole of the messages.
    let said = |output: &Output| {
        if let Some(listener) = listener
            && !output.stderr.is_empty()
        {
            listener(&crate::control::Event::Output {
                stdout: String::new(),
                stderr: output.stderr.clone(),
            });
        }
    };
    let mut output = Output::default();
    let checked = check_in_project(fs, project, script, text, registry);
    let source = Source::new(script, text);
    // `zeff run` stops at the first error (section 11); warnings from the check are shown.
    let mut sources = Sources::new();
    sources.add(source.clone());
    let module_sources: std::collections::HashMap<String, Source> = checked
        .module_sources
        .iter()
        .map(|s| (s.name().to_string(), s.clone()))
        .collect();
    for module in checked.module_sources.iter() {
        sources.add(module.clone());
    }
    // The pragmas of the built-ins written in Zeff join the script's and the modules', under
    // their own files, so that a warning of a body is silenced where its file says so.
    let mut allows = checked.allows;
    for body in zeff.values() {
        allows.add(body.source.name(), body.allows.clone());
    }
    output.allowed = checked.allowed;
    let checked = checked.checked;
    if let Some(first) = checked
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Error)
    {
        for warning in checked
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
        {
            if warning
                .labels
                .first()
                .is_some_and(|l| l.span.start < first.labels.first().map_or(0, |f| f.span.start))
            {
                output.stderr.push_str(&warning.render(&sources));
            }
        }
        output.stderr.push_str(&first.render(&sources));
        said(&output);
        let internal = first.code.starts_with("Z9");
        return Outcome {
            output,
            exit_code: if internal { 2 } else { 1 },
        };
    }
    // `--deny-warnings` (section 11.1): a warning of the check that no pragma allows is an
    // error, and as with any error of the check the run does not start.
    if options.deny_warnings
        && let Some(first) = checked
            .diagnostics
            .iter()
            .find(|d| crate::pragmas::deniable(d))
    {
        output
            .stderr
            .push_str(&crate::pragmas::denied(first).render(&sources));
        said(&output);
        return Outcome {
            output,
            exit_code: 3,
        };
    }
    // The check's warnings are shown before the run; its notes (the run-time mark of a file
    // that does not exist yet, section 11) are for `zeff check` and the editor, since the run
    // reports such a file itself.
    for warning in checked
        .diagnostics
        .iter()
        .filter(|d| d.severity != Severity::Note)
    {
        output.stderr.push_str(&warning.render(&sources));
    }
    said(&output);
    // The schema cache of the run: the data-root files it reads, and the columns of the
    // statements checked at run time, recorded for the next check (section 11).
    let schemas = crate::schemas::SchemaCache::new(fs, project);
    let mut files = ProjectFiles::new(fs, project);
    files.schemas = Some(&schemas);
    let interpreter = Interpreter::new(&checked.program, bodies, &files, script, &source)
        .with_zeff(zeff)
        .with_sources(&module_sources)
        .with_panic_at(options.panic_at_line)
        .with_width(options.width)
        .with_freeing(options.free_bindings)
        .with_fusion(options.fused)
        .with_allows(&allows)
        .with_denied_warnings(options.deny_warnings)
        .with_checks(options.collect_checks)
        .with_control(control)
        .with_listener(listener);
    let result = interpreter.run(&mut output);
    for (statement, columns) in std::mem::take(&mut output.produced) {
        schemas.record_run(statement, columns);
    }
    let _ = schemas.save();
    match result {
        Ok(()) => Outcome {
            output,
            exit_code: 0,
        },
        // A warning that `--deny-warnings` refused comes back as it was raised.
        Err(error) if error.severity != Severity::Error => Outcome {
            output,
            exit_code: 3,
        },
        Err(error) => Outcome {
            output,
            exit_code: if error.code.starts_with("Z9") { 2 } else { 1 },
        },
    }
}
