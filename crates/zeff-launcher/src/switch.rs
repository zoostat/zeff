//! `zeff engine use <version>`, "Update engine" from the command line (Engine and Project
//! Specification, section 2.1, "Engines"; section 3.6 of the Language Reference; release
//! 0.9.0, step 20): the project is run on the engine it pins and on another, what the two
//! print and warn of is compared and said in sentences, and only `--write` moves the pin.
//!
//! A bare call that a built-in of the other engine would capture is an error there (Z0109).
//! The other engine names each such call and its full form; `--write` rewrites them first, so
//! that the script calls what it always called, and then compares.
//!
//! What "every result" is, at this release: each script of `scripts/`, what it prints, whether
//! it ends in an error, and the codes of the warnings it raises. Run records, which will carry
//! a hash for every statement, arrive with release 0.11.0.

use crate::engines::{Asked, EXIT_REFUSED, Machine, one_version, usage};
use crate::{EXIT_ENGINE, MANIFEST, Store, find_engine, installed, manifest_above, pin};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use zeff_diagnostics::{Diagnostic, Sources};

/// How one script ended on one engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// The exit code; `None` when the engine was ended by a signal.
    pub code: Option<i32>,
    pub out: String,
    pub err: String,
}

impl Ran {
    /// The codes of the warnings raised, each with how many times.
    pub fn warnings(&self) -> BTreeMap<String, usize> {
        let mut codes = BTreeMap::new();
        for line in self.err.lines() {
            if let Some(code) = line
                .strip_prefix("warning[")
                .and_then(|rest| rest.split(']').next())
            {
                *codes.entry(code.to_string()).or_insert(0) += 1;
            }
        }
        codes
    }

    /// The first error it stopped on, as its message's first line.
    pub fn first_error(&self) -> Option<&str> {
        self.err.lines().find(|line| line.starts_with("error"))
    }
}

/// A call the other engine would give to a built-in, and what it is rewritten to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Captured {
    /// The file, as the engine names it: `scripts/main.zeff`.
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    /// The full form: `helpers::mean`.
    pub replacement: String,
    /// The engine's own sentence.
    pub message: String,
}

/// The Z0109 errors of `zeff check --format json`, each with the rewrite the engine gives.
pub fn captured_names(json: &str) -> Vec<Captured> {
    let Ok(serde_json::Value::Array(records)) = serde_json::from_str::<serde_json::Value>(json)
    else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for record in &records {
        if record["code"] != "Z0109" {
            continue;
        }
        let (Some(file), Some(message)) = (record["file"].as_str(), record["message"].as_str())
        else {
            continue;
        };
        let Some(edit) = record["fixes"][0]["edits"][0].as_object() else {
            continue;
        };
        let number = |key: &str| edit.get(key).and_then(|n| n.as_u64()).map(|n| n as usize);
        let (Some(line), Some(column), Some(end_line), Some(end_column), Some(replacement)) = (
            number("line"),
            number("column"),
            number("end_line"),
            number("end_column"),
            edit.get("replacement").and_then(|text| text.as_str()),
        ) else {
            continue;
        };
        found.push(Captured {
            file: file.to_string(),
            line,
            column,
            end_line,
            end_column,
            replacement: replacement.to_string(),
            message: message.to_string(),
        });
    }
    found.sort();
    found.dedup();
    found
}

/// Where the character at `line` and `column`, both counted from one, begins in `text`.
fn offset_of(text: &str, line: usize, column: usize) -> Option<usize> {
    let mut start = 0;
    for (at, row) in text.split_inclusive('\n').enumerate() {
        if at + 1 == line {
            let within = row
                .char_indices()
                .map(|(byte, _)| byte)
                .chain(std::iter::once(row.len()))
                .nth(column.checked_sub(1)?)?;
            return Some(start + within);
        }
        start += row.len();
    }
    None
}

/// `text` with the calls of `captured`, all of one file, written in full. `None` when a place
/// is not in the text, which is then left as it was.
pub fn rewritten(text: &str, captured: &[Captured]) -> Option<String> {
    let mut edits: Vec<(usize, usize, &str)> = Vec::new();
    for call in captured {
        edits.push((
            offset_of(text, call.line, call.column)?,
            offset_of(text, call.end_line, call.end_column)?,
            &call.replacement,
        ));
    }
    // From the end, so that no edit moves a place still to be edited.
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    let mut written = text.to_string();
    for (start, end, replacement) in edits {
        if start > end || end > written.len() {
            return None;
        }
        written.replace_range(start..end, replacement);
    }
    Some(written)
}

/// The manifest with its pin moved to `version`, every other character as it was, the comment
/// on the pin's own line among them. `None` when the pin is not found as a quoted version on a
/// line `zeff = ...` under `[project]`; what is written is read back before it is given.
pub fn with_pin(manifest: &str, version: &str) -> Option<String> {
    let mut in_project = false;
    let mut written = String::with_capacity(manifest.len() + 8);
    let mut done = false;
    for line in manifest.split_inclusive('\n') {
        let text = line.trim();
        if text.starts_with('[') {
            in_project = text.split('#').next().unwrap_or_default().trim() == "[project]";
        }
        let is_pin = in_project
            && !done
            && text
                .strip_prefix("zeff")
                .is_some_and(|rest| rest.trim_start().starts_with('='));
        let quoted = is_pin
            .then(|| {
                let open = line.find('"')?;
                let close = open + 1 + line[open + 1..].find('"')?;
                Some((open, close))
            })
            .flatten();
        match quoted {
            Some((open, close)) => {
                written.push_str(&line[..=open]);
                written.push_str(version);
                written.push_str(&line[close..]);
                done = true;
            }
            None => written.push_str(line),
        }
    }
    (done && pin(&written).as_deref() == Some(version)).then_some(written)
}

/// The scripts of a project: every `.zeff` file under `scripts/`, by path, each as the engine
/// is given it, with forward slashes.
pub fn scripts_of(project: &Path) -> Vec<String> {
    fn walk(folder: &Path, under: &str, found: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = format!("{under}/{name}");
            if entry.path().is_dir() {
                walk(&entry.path(), &path, found);
            } else if name.ends_with(".zeff") {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&project.join("scripts"), "scripts", &mut found);
    found.sort();
    found
}

/// Start `engine` on the project with `args`. `trial` is the version being tried, for an
/// engine the project does not pin: it is told so and does not refuse the pin.
fn start(
    engine: &Path,
    project: &Path,
    args: &[&str],
    trial: Option<&str>,
) -> std::io::Result<Ran> {
    let mut command = std::process::Command::new(engine);
    command
        .args(args)
        .args(["--color", "never"])
        .current_dir(project)
        .env("ZEFF_LAUNCHER", crate::VERSION)
        .stdin(std::process::Stdio::null());
    match trial {
        Some(version) => command.env("ZEFF_ENGINE_USE", version),
        None => command.env_remove("ZEFF_ENGINE_USE"),
    };
    let output = command.output()?;
    Ok(Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// What is said of one script on the two engines; whether anything differs.
pub fn compared(
    pinned: (&str, &Ran),
    other: (&str, &Ran),
    out: &mut dyn Write,
) -> std::io::Result<bool> {
    let ((old, was), (new, is)) = (pinned, other);
    let mut differs = false;
    let ended = |ran: &Ran| ran.code == Some(0);
    match (ended(was), ended(is)) {
        (true, false) => {
            differs = true;
            writeln!(out, "  runs on Zeff {old} and stops on Zeff {new}:")?;
            writeln!(out, "    {}", is.first_error().unwrap_or("with no message"))?;
        }
        (false, true) => {
            differs = true;
            writeln!(
                out,
                "  stops on Zeff {old} and runs on Zeff {new}; on Zeff {old}:"
            )?;
            writeln!(
                out,
                "    {}",
                was.first_error().unwrap_or("with no message")
            )?;
        }
        (false, false) => {
            let same = was.first_error() == is.first_error();
            differs |= !same;
            writeln!(
                out,
                "  stops on both engines{}:",
                if same { ", with the same error" } else { "" }
            )?;
            writeln!(
                out,
                "    on Zeff {old}: {}",
                was.first_error().unwrap_or("no message")
            )?;
            if !same {
                writeln!(
                    out,
                    "    on Zeff {new}: {}",
                    is.first_error().unwrap_or("no message")
                )?;
            }
        }
        (true, true) => {}
    }
    let (before, after): (Vec<&str>, Vec<&str>) =
        (was.out.lines().collect(), is.out.lines().collect());
    if before == after {
        writeln!(
            out,
            "  prints the same on both engines, {}",
            plural(before.len(), "line", "lines")
        )?;
    } else {
        differs = true;
        let shared = before.len().min(after.len());
        let unlike = (0..shared).filter(|&at| before[at] != after[at]).count()
            + before.len().max(after.len())
            - shared;
        let first = (0..shared)
            .find(|&at| before[at] != after[at])
            .unwrap_or(shared);
        if before.len() == after.len() {
            writeln!(
                out,
                "  prints {} of {} differently; the first is line {}:",
                plural(unlike, "line", "lines"),
                before.len(),
                first + 1
            )?;
        } else {
            writeln!(
                out,
                "  prints {} on Zeff {old} and {} on Zeff {new}; the first that differs is line {}:",
                plural(before.len(), "line", "lines"),
                plural(after.len(), "line", "lines"),
                first + 1
            )?;
        }
        let shown = |lines: &[&str]| {
            lines
                .get(first)
                .map_or("(nothing)".to_string(), |line| (*line).to_string())
        };
        writeln!(out, "    on Zeff {old}: {}", shown(&before))?;
        writeln!(out, "    on Zeff {new}: {}", shown(&after))?;
    }
    let (warned, warns) = (was.warnings(), is.warnings());
    let listed = |codes: Vec<&String>| {
        codes
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let new_codes: Vec<&String> = warns
        .keys()
        .filter(|code| !warned.contains_key(*code))
        .collect();
    let gone: Vec<&String> = warned
        .keys()
        .filter(|code| !warns.contains_key(*code))
        .collect();
    if !new_codes.is_empty() {
        differs = true;
        writeln!(
            out,
            "  Zeff {new} raises {}, which Zeff {old} does not",
            listed(new_codes)
        )?;
    }
    if !gone.is_empty() {
        differs = true;
        writeln!(
            out,
            "  Zeff {old} raises {}, which Zeff {new} does not",
            listed(gone)
        )?;
    }
    Ok(differs)
}

fn say(err: &mut dyn Write, diagnostic: &Diagnostic) {
    let _ = write!(err, "{}", diagnostic.render(&Sources::new()));
}

/// Answer `zeff engine use <version> [--write]`; the exit code: 0 when the two engines agree,
/// or when the pin was moved; 1 when they differ and nothing was written.
pub fn use_engine(
    asked: &Asked,
    words: &[String],
    store: &Store,
    machine: &Machine<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let other = match one_version(words, "use", "0.9.1") {
        Ok(version) => version,
        Err(message) => return usage(err, &message),
    };
    let start_at = match &asked.project {
        Some(dir) if dir.is_absolute() => dir.clone(),
        Some(dir) => machine.cwd.join(dir),
        None => machine.cwd.to_path_buf(),
    };
    let Some(manifest) = manifest_above(&start_at) else {
        say(
            err,
            &Diagnostic::error("Z0709", "not inside a Zeff project").help(
                "`zeff engine use` moves a project to another engine: run it in the project's folder, or name the folder with `-p`",
            ),
        );
        return EXIT_REFUSED;
    };
    let project: PathBuf = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
    let manifest_text = std::fs::read_to_string(&manifest).unwrap_or_default();
    let Some(pinned) = pin(&manifest_text) else {
        return usage(
            err,
            &format!(
                "`{}` pins no engine that can be read; `zeff check` says what is wrong with it",
                manifest.display()
            ),
        );
    };
    let versions = installed(store, machine.platform);
    let engine_of = |version: &str, err: &mut dyn Write| {
        let found = find_engine(store, version, machine.platform);
        if found.is_none() {
            say(
                err,
                &Diagnostic::error("Z0710", format!("Zeff {version} is not installed")).help(format!(
                    "`zeff engine use` runs the project on both engines; `zeff engine install {version}` installs it; installed: {}",
                    if versions.is_empty() {
                        "none".to_string()
                    } else {
                        versions.iter().map(|v| v.text.as_str()).collect::<Vec<_>>().join(", ")
                    }
                )),
            );
        }
        found
    };
    let (Some(old_engine), Some(new_engine)) = (engine_of(&pinned, err), engine_of(other, err))
    else {
        return EXIT_ENGINE;
    };
    if pinned == other {
        let _ = writeln!(out, "the project already runs on Zeff {other}");
        return 0;
    }
    let failed = |err: &mut dyn Write, engine: &Path, e: std::io::Error| {
        say(
            err,
            &Diagnostic::error(
                "Z0710",
                format!(
                    "the engine `{}` could not be started: {e}",
                    engine.display()
                ),
            )
            .help("`zeff engine remove` and `zeff engine install` install it afresh"),
        );
        EXIT_ENGINE
    };
    let scripts = scripts_of(&project);
    let _ = writeln!(
        out,
        "Zeff {pinned} and Zeff {other} on the project in `{}`: {}",
        project.display(),
        plural(scripts.len(), "script", "scripts")
    );
    // The calls the other engine's built-ins would capture, asked of that engine.
    let mut captured = Vec::new();
    for script in &scripts {
        match start(
            &new_engine,
            &project,
            &["check", script, "--format", "json"],
            Some(other),
        ) {
            Ok(checked) => captured.extend(captured_names(&checked.out)),
            Err(e) => return failed(err, &new_engine, e),
        }
    }
    captured.sort();
    captured.dedup();
    if !captured.is_empty() {
        let _ = writeln!(
            out,
            "\n{} that a built-in function of Zeff {other} would capture (Z0109):",
            plural(captured.len(), "call", "calls")
        );
        for call in &captured {
            let _ = writeln!(
                out,
                "  {}:{}:{}: {}; in full, `{}`",
                call.file, call.line, call.column, call.message, call.replacement
            );
        }
        if !asked.write {
            let _ = writeln!(
                out,
                "\nnothing is compared until these are written in full, and nothing was changed; `zeff engine use {other} --write` rewrites them, compares, and moves the pin"
            );
            return EXIT_REFUSED;
        }
        let mut by_file: BTreeMap<&str, Vec<Captured>> = BTreeMap::new();
        for call in &captured {
            by_file.entry(&call.file).or_default().push(call.clone());
        }
        for (file, calls) in by_file {
            let path = project.join(file);
            let done = std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| rewritten(&text, &calls))
                .is_some_and(|text| std::fs::write(&path, text).is_ok());
            if !done {
                return usage(
                    err,
                    &format!(
                        "`{}` could not be rewritten; nothing else was changed",
                        path.display()
                    ),
                );
            }
        }
        let _ = writeln!(out, "rewritten in full, each to the project's own function");
    }
    // Each script on both engines. The engine whose results the project is to keep runs last,
    // so that the files a script writes are left as that engine writes them.
    let mut differing = 0;
    let mut stops_only_on_other = Vec::new();
    for script in &scripts {
        let run =
            |engine: &Path, trial: Option<&str>| start(engine, &project, &["run", script], trial);
        let (was, is) = if asked.write {
            let was = run(&old_engine, None);
            (was, run(&new_engine, Some(other)))
        } else {
            let is = run(&new_engine, Some(other));
            (run(&old_engine, None), is)
        };
        let was = match was {
            Ok(was) => was,
            Err(e) => return failed(err, &old_engine, e),
        };
        let is = match is {
            Ok(is) => is,
            Err(e) => return failed(err, &new_engine, e),
        };
        let _ = writeln!(out, "\n{script}");
        if compared((&pinned, &was), (other, &is), out).unwrap_or(true) {
            differing += 1;
        }
        if was.code == Some(0) && is.code != Some(0) {
            stops_only_on_other.push(script.clone());
        }
    }
    let _ = writeln!(
        out,
        "\n{} compared: {}",
        plural(scripts.len(), "script", "scripts"),
        match differing {
            0 => "the two engines agree on every one".to_string(),
            n => format!("{} on the two engines", plural(n, "differs", "differ")),
        }
    );
    if !asked.write {
        let _ = writeln!(
            out,
            "the pin is not moved; `zeff engine use {other} --write` moves it to Zeff {other}"
        );
        return if differing == 0 { 0 } else { EXIT_REFUSED };
    }
    if !stops_only_on_other.is_empty() {
        let _ = writeln!(
            out,
            "the pin is not moved: {} on Zeff {pinned} and {} on Zeff {other}",
            if stops_only_on_other.len() == 1 {
                "a script runs"
            } else {
                "scripts run"
            },
            if stops_only_on_other.len() == 1 {
                "stops"
            } else {
                "stop"
            },
        );
        return EXIT_REFUSED;
    }
    let moved = with_pin(&manifest_text, other).is_some_and(|text| {
        let being_written = project.join(format!(".{MANIFEST}-{}", std::process::id()));
        std::fs::write(&being_written, text)
            .and_then(|()| std::fs::rename(&being_written, &manifest))
            .inspect_err(|_| {
                let _ = std::fs::remove_file(&being_written);
            })
            .is_ok()
    });
    if !moved {
        return usage(
            err,
            &format!(
                "the pin could not be rewritten in `{}`; write `zeff = \"{other}\"` under `[project]` there by hand",
                manifest.display()
            ),
        );
    }
    let _ = writeln!(
        out,
        "the pin is moved: `{}` now names Zeff {other}",
        manifest.display()
    );
    0
}
