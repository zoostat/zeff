//! A record of each run (open decision #206; release 0.9.0, step 22b): what `zeff audit` needs
//! to say whether a result that was made is one a known issue affects. For every `zeff run`,
//! one small file in `.zeff/runs/` of the project: the engine and its build, the day, the
//! script, how the run ended, and the built-in functions the script and the project's
//! functions call, each with the options written at the call.
//!
//! The form is provisional and the file says so: the run record of release 0.11.0, with its
//! versioned schema, the hashes of the inputs and of every statement's result, replaces it.
//! The calls are read from the checked script, not traced as it runs: a call in a branch that
//! was not taken is among them, which errs on the side of saying so.

use crate::fs::FileSystem;
use crate::project::Project;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use toml::de::{DeTable, DeValue};
use zeff_ir::walk::{Found, walk};
use zeff_ir::{ExprKind, Program, Slot};
use zeff_registry::Registry;

/// Where the records of a project are kept, under its folder.
pub const FOLDER: &str = ".zeff/runs";

/// A built-in function a script calls, with the options written at the call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallMade {
    pub function: String,
    pub package: String,
    /// The options written, by name, in the order of their names.
    pub options: Vec<String>,
}

/// One run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    /// The version of the engine that ran it.
    pub engine: String,
    /// `official` or `source`.
    pub build: String,
    /// The day, `2026-11-22`, and the time of day, `14:03:22`, by the machine's clock; `None`
    /// for a run that had no clock.
    pub on: Option<String>,
    pub at: Option<String>,
    /// The script, as the project names it: `scripts/main.zeff`.
    pub script: String,
    /// How the run ended (Language Reference, section 3.8, "Exit codes").
    pub exit_code: i32,
    /// Every distinct call, in order.
    pub calls: Vec<CallMade>,
}

/// The calls of built-in functions in `program`: the script's statements and every function
/// of the project, each distinct pair of a function and the options written kept once.
pub fn calls_of(program: &Program, script: &str, registry: &Registry) -> Vec<CallMade> {
    let mut found: BTreeSet<CallMade> = BTreeSet::new();
    let mut tell = |what: Found<'_>| {
        let Found::Expr(expr) = what else {
            return;
        };
        let ExprKind::Call(call) = &expr.kind else {
            return;
        };
        let registered = registry.named(&call.name);
        let spec = registered
            .iter()
            .find(|known| {
                known.spec.package == call.package && known.spec.params.len() == call.args.len()
            })
            .or_else(|| {
                registered
                    .iter()
                    .find(|known| known.spec.package == call.package)
            });
        let mut options: BTreeSet<String> = BTreeSet::new();
        if let Some(known) = spec {
            let params = &known.spec.params;
            // What was written, not what the check filled in with a default.
            let written = call.order.iter().filter_map(|slot| match slot {
                Slot::Param(at) => Some(*at),
                _ => None,
            });
            // An option a spread fills is written too, for all the audit can tell.
            let spread = call
                .spreads
                .iter()
                .flat_map(|spread| spread.targets.iter().map(|(_, at)| *at));
            for at in written.chain(spread) {
                if let Some(param) = params.get(at)
                    && param.option
                {
                    options.insert(param.name.to_string());
                }
            }
        }
        found.insert(CallMade {
            function: call.name.clone(),
            package: call.package.clone(),
            options: options.into_iter().collect(),
        });
    };
    walk(program, script, true, &mut tell);
    let mut files: Vec<&str> = program
        .functions
        .iter()
        .map(|function| function.file.as_str())
        .filter(|file| *file != script)
        .collect();
    files.sort_unstable();
    files.dedup();
    for file in files {
        walk(program, file, false, &mut tell);
    }
    found.into_iter().collect()
}

/// The day and the time of day of a clock's reading, microseconds since 1970 by the local
/// wall clock (the file system's `now`): `("2026-11-22", "14:03:22")`.
pub fn day_and_time(micros: i64) -> (String, String) {
    let seconds = micros.div_euclid(1_000_000);
    let days = seconds.div_euclid(86_400);
    let of_day = seconds.rem_euclid(86_400);
    // The civil date of a count of days since 1970-01-01, by the usual reckoning in eras of
    // four hundred years.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (
        format!("{year:04}-{month:02}-{day:02}"),
        format!(
            "{:02}:{:02}:{:02}",
            of_day / 3_600,
            of_day % 3_600 / 60,
            of_day % 60
        ),
    )
}

/// A text as a TOML string.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl RunRecord {
    /// The record of a run of `script` that ended with `exit_code`: its calls read from the
    /// script as it is checked in its project, the day from the file system's clock.
    pub fn of(
        fs: &dyn FileSystem,
        project: &Project,
        script: &str,
        text: &str,
        exit_code: i32,
        build: &str,
        registry: &Registry,
    ) -> RunRecord {
        let checked = crate::run::check_in_project(fs, project, script, text, registry);
        let (on, at) = match fs.now() {
            Some(micros) => {
                let (day, time) = day_and_time(micros);
                (Some(day), Some(time))
            }
            None => (None, None),
        };
        RunRecord {
            engine: crate::VERSION.to_string(),
            build: build.to_string(),
            on,
            at,
            script: script.to_string(),
            exit_code,
            calls: calls_of(&checked.checked.program, script, registry),
        }
    }

    /// The record as its file holds it.
    pub fn text(&self) -> String {
        let mut text = String::from(
            "# A run of this project, kept for `zeff audit`. The form is provisional: the run record\n# of Zeff 0.11.0 replaces it.\nformat = 0\n",
        );
        text.push_str(&format!("engine = {}\n", quoted(&self.engine)));
        text.push_str(&format!("build = {}\n", quoted(&self.build)));
        if let (Some(on), Some(at)) = (&self.on, &self.at) {
            text.push_str(&format!("on = {}\nat = {}\n", quoted(on), quoted(at)));
        }
        text.push_str(&format!("script = {}\n", quoted(&self.script)));
        text.push_str(&format!("exit_code = {}\n", self.exit_code));
        for call in &self.calls {
            let options: Vec<String> = call.options.iter().map(|option| quoted(option)).collect();
            text.push_str(&format!(
                "\n[[call]]\nfunction = {}\npackage = {}\noptions = [{}]\n",
                quoted(&call.function),
                quoted(&call.package),
                options.join(", ")
            ));
        }
        text
    }

    /// Read a record's file; `None` for one that is not a record of this form.
    pub fn read(text: &str) -> Option<RunRecord> {
        let document = DeTable::parse(text).ok()?;
        let table = document.get_ref();
        let text_at = |table: &DeTable<'_>, key: &str| match table.get(key)?.get_ref() {
            DeValue::String(text) => Some(text.to_string()),
            _ => None,
        };
        match table.get("format")?.get_ref() {
            DeValue::Integer(format) if format.as_str() == "0" => {}
            _ => return None,
        }
        let exit_code = match table.get("exit_code")?.get_ref() {
            DeValue::Integer(code) => code.as_str().parse::<i32>().ok()?,
            _ => return None,
        };
        let mut calls = Vec::new();
        if let Some(entries) = table.get("call") {
            let DeValue::Array(entries) = entries.get_ref() else {
                return None;
            };
            for entry in entries {
                let DeValue::Table(entry) = entry.get_ref() else {
                    return None;
                };
                let options = match entry.get("options")?.get_ref() {
                    DeValue::Array(options) => options
                        .iter()
                        .map(|option| match option.get_ref() {
                            DeValue::String(option) => Some(option.to_string()),
                            _ => None,
                        })
                        .collect::<Option<Vec<String>>>()?,
                    _ => return None,
                };
                calls.push(CallMade {
                    function: text_at(entry, "function")?,
                    package: text_at(entry, "package")?,
                    options,
                });
            }
        }
        Some(RunRecord {
            engine: text_at(table, "engine")?,
            build: text_at(table, "build")?,
            on: text_at(table, "on"),
            at: text_at(table, "at"),
            script: text_at(table, "script")?,
            exit_code,
            calls,
        })
    }
}

/// Keep `record` in the project's folder of runs, under a name made of its day and time; the
/// path it was written to.
pub fn keep(fs: &dyn FileSystem, root: &Path, record: &RunRecord) -> std::io::Result<PathBuf> {
    let mut folder = root.to_path_buf();
    for part in FOLDER.split('/') {
        folder.push(part);
        if !fs.is_dir(&folder) {
            fs.create_dir(&folder)?;
        }
    }
    let stem = match (&record.on, &record.at) {
        (Some(on), Some(at)) => format!("{}-{}", on.replace('-', ""), at.replace(':', "")),
        _ => "undated".to_string(),
    };
    // Two runs within one second, or two without a clock, are numbered.
    let mut path = folder.join(format!("{stem}.toml"));
    let mut number = 1;
    while fs.is_file(&path) {
        number += 1;
        path = folder.join(format!("{stem}-{number}.toml"));
    }
    fs.write(&path, record.text().as_bytes())?;
    Ok(path)
}

/// The records kept in a project, each with its file's name, in the order of the names, which
/// is the order of the runs; a file that is not a record is passed over.
pub fn kept(fs: &dyn FileSystem, root: &Path) -> Vec<(String, RunRecord)> {
    let mut folder = root.to_path_buf();
    for part in FOLDER.split('/') {
        folder.push(part);
    }
    let mut names = fs.list(&folder).unwrap_or_default();
    names.retain(|name| name.ends_with(".toml"));
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let text = fs.read_to_string(&folder.join(&name)).ok()?;
            Some((name, RunRecord::read(&text)?))
        })
        .collect()
}
