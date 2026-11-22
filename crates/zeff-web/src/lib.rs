//! The engine as a worker of the browser hosts it (Engine and Project Specification, section
//! 8.3, point 7: "The engine runs in its own worker (a Web Worker in the browser, a thread
//! natively), so the interface never freezes").
//!
//! A [`Host`] is one project held in memory: the interface gives it the project's files, asks
//! it to run a script, and is told the run's steps and what it prints as it goes, each as one
//! line of JSON, then its outcome. A run is cancelled between two steps by a flag the
//! interface sets in memory it shares with the worker: a worker that is computing reads no
//! message, so the engine asks for the flag itself.
//!
//! The functions here are plain Rust and are tested natively; `browser` binds them for
//! WebAssembly, and `js/` holds the worker script that carries the messages.

use std::path::Path;
use zeff_diagnostics::json_string;
use zeff_engine::control::{Control, Event};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Frames, Plots, RunOptions, run_script_showing};

#[cfg(target_arch = "wasm32")]
mod browser;

/// Where the project lies in the worker's memory.
const ROOT: &str = "/project";

/// The largest budget a browser is given by default: 3.5 GB (section 8.3, point 7).
pub const LARGEST_DEFAULT: u64 = 3_500_000_000;

/// The memory budget of a window of the browser, where the project sets none: the smaller of
/// 3.5 GB and half the memory the device reports; 3.5 GB where it reports none.
pub fn default_budget(device_bytes: Option<u64>) -> u64 {
    device_bytes.map_or(LARGEST_DEFAULT, |bytes| (bytes / 2).min(LARGEST_DEFAULT))
}

/// One event of a run as a line of JSON: `type` is `started`, `finished`, `pass` or `output`.
pub fn event_json(event: &Event) -> String {
    match event {
        Event::Started {
            statement,
            of,
            line,
        } => format!(
            "{{\"type\": \"started\", \"statement\": {statement}, \"of\": {of}, \"line\": {line}}}"
        ),
        Event::Finished {
            statement,
            of,
            line,
            failed,
        } => format!(
            "{{\"type\": \"finished\", \"statement\": {statement}, \"of\": {of}, \"line\": {line}, \"failed\": {failed}}}"
        ),
        Event::Pass { pass, of, line } => format!(
            "{{\"type\": \"pass\", \"pass\": {pass}, \"of\": {}, \"line\": {line}}}",
            of.map_or("null".to_string(), |n| n.to_string())
        ),
        Event::Output { stdout, stderr } => format!(
            "{{\"type\": \"output\", \"stdout\": {}, \"stderr\": {}}}",
            json_string(stdout),
            json_string(stderr)
        ),
    }
}

/// The explained page of a diagnostic code (Language Reference, section 11.1), for the
/// interface's log and Help pane: a line of JSON of type `explained` whose `page` is the record
/// `zeff explain <code> --format json` prints, or of type `error` for a code no page explains.
/// The code is taken as it is written in a message, or as the link a diagnostic carries, in
/// either case of its letter. It needs no project.
pub fn explain(code: &str) -> String {
    let asked = code
        .strip_prefix("zeff://explain/")
        .unwrap_or(code)
        .to_ascii_uppercase();
    match zeff_engine::explain::explain(&asked) {
        Ok(page) => format!(
            "{{\"type\": \"explained\", \"page\": {}}}",
            zeff_engine::explain::json(&page)
        ),
        Err(_) => format!(
            "{{\"type\": \"error\", \"message\": {}}}",
            json_string(&format!("no code `{code}` has an explained page"))
        ),
    }
}

/// Every code with an explained page, by family, as a line of JSON of type `codes`: what the
/// Help pane lists before a page is asked for.
pub fn explained_codes() -> String {
    let families: Vec<String> = zeff_engine::explain::families()
        .iter()
        .map(|family| {
            let codes: Vec<String> = family
                .codes
                .iter()
                .map(|(code, title)| {
                    format!(
                        "{{\"code\": {}, \"kind\": {}, \"title\": {}}}",
                        json_string(code),
                        json_string(zeff_engine::explain::kind(code)),
                        json_string(title)
                    )
                })
                .collect();
            format!(
                "{{\"name\": {}, \"area\": {}, \"codes\": [{}]}}",
                json_string(family.name),
                json_string(family.area),
                codes.join(", ")
            )
        })
        .collect();
    format!(
        "{{\"type\": \"codes\", \"families\": [{}]}}",
        families.join(", ")
    )
}

/// A project in memory, and the runs made in it.
pub struct Host {
    fs: MemoryFileSystem,
    /// The frames the bindings of the last run held when it ended, for the interface to show.
    frames: std::cell::RefCell<Frames>,
    /// The plots the statements of the last run made, for the interface to draw at the size
    /// of its pane (open decision #197).
    plots: std::cell::RefCell<Plots>,
    /// What each statement of the last run printed, with the values it printed and their
    /// places in it, for the interface to have the print again at the width of its pane
    /// (open decision #175, C6).
    prints: std::cell::RefCell<Vec<(String, Vec<zeff_core::runtime::Shown>)>>,
}

/// The narrowest page a print is made for, as `--width` has it.
pub const NARROWEST: usize = 40;

/// A line of JSON of type `error`, for something asked of the host that it cannot give.
fn refused(message: &str) -> String {
    format!(
        "{{\"type\": \"error\", \"message\": {}}}",
        json_string(message)
    )
}

impl Host {
    /// A project called `name`, pinned to this engine, with nothing in it yet.
    pub fn new(name: &str) -> Host {
        let fs = MemoryFileSystem::new();
        fs.add_file(
            format!("{ROOT}/zeff.toml"),
            format!(
                "[project]\nname = {}\nzeff = \"{}\"\nedition = \"preview\"\n",
                json_string(name),
                zeff_engine::VERSION
            ),
        );
        Host {
            fs,
            frames: std::cell::RefCell::default(),
            plots: std::cell::RefCell::default(),
            prints: std::cell::RefCell::default(),
        }
    }

    /// A frame of the last run as the bytes of an Arrow stream, with its column types, labels
    /// and categories (section 8.3, point 5), or `None` where no binding of that name held a
    /// frame. The worker passes the buffer of these bytes to the interface, which then owns
    /// it: the bytes are not copied between the two (section 8.3, "Browser transfers").
    pub fn frame(&self, name: &str) -> Option<Vec<u8>> {
        let frames = self.frames.borrow();
        let (_, frame) = frames.iter().find(|(bound, _)| bound == name)?;
        zeff_columns::write_ipc(&frame.table).ok()
    }

    /// A file of the project read back, by its path from the project's folder: what a script
    /// wrote with `export` or `save`, or what the interface put in. `None` where there is none.
    pub fn file(&self, path: &str) -> Option<Vec<u8>> {
        use zeff_engine::fs::FileSystem;
        let at = format!("{ROOT}/{}", path.trim_start_matches('/'));
        self.fs.read(Path::new(&at)).ok()
    }

    /// A plot of the last run as an SVG, laid out on a page of `width` by `height` points
    /// without a new run (open decision #197; Zemp Specification, section 7, "Redraw to
    /// fit"): the interface gives the size of its pane and is given the plot at that size,
    /// its text at the sizes of the theme, never stretched. `index` is the plot's place in the
    /// list the run's outcome gave. `theme` is `journal`, `minimal` or `slide`; without one
    /// the plot has the theme its script gave it. The layout and the writer are those of
    /// `export`, so at the points of a page `export` writes, the text is the same byte for
    /// byte. What comes back is a line of JSON of type `plot`, or of type `error` with
    /// `of_size` saying whether the size is at fault (a pane too small for the plot's labels)
    /// or the plot.
    pub fn plot_svg(&self, index: usize, width: f64, height: f64, theme: Option<&str>) -> String {
        use zeff_core::value::ThemeName;
        let plots = self.plots.borrow();
        let Some(made) = plots.get(index) else {
            return refused(&format!(
                "the last run made {}, and none is numbered {index}",
                match plots.len() {
                    0 => "no plot".to_string(),
                    1 => "1 plot, numbered 0".to_string(),
                    n => format!("{n} plots, numbered 0 to {}", n - 1),
                }
            ));
        };
        let named = match theme {
            None => None,
            Some("journal") => Some(ThemeName::Journal),
            Some("minimal") => Some(ThemeName::Minimal),
            Some("slide") => Some(ThemeName::Slide),
            Some(other) => {
                return refused(&format!(
                    "`{other}` is no theme of a plot: `journal`, `minimal` or `slide`"
                ));
            }
        };
        match zeff_plot::figure::svg_at(&made.plot, width, height, named) {
            Ok(svg) => format!(
                "{{\"type\": \"plot\", \"index\": {index}, \"width\": {}, \"height\": {}, \"theme\": {}, \"svg\": {}}}",
                zeff_plot::json::number(width),
                zeff_plot::json::number(height),
                json_string(named.unwrap_or(made.plot.theme.name).name()),
                json_string(&svg)
            ),
            Err(problem) => format!(
                "{{\"type\": \"error\", \"message\": {}, \"of_size\": {}}}",
                json_string(&problem.message),
                problem.of_size
            ),
        }
    }

    /// What a statement of the last run printed, made again at `width` characters without a
    /// new run (open decision #175, C6; Zemp Specification, section 5, the opened result):
    /// every frame and table it printed wraps at that width, in blocks of columns, and
    /// everything else is as it was. `statement` is the statement's place from 1, as the
    /// outcome and the events of the run number it. The text is what `zeff run --width
    /// <width>` prints for that statement, character for character. A line of JSON of type
    /// `printed`, or of type `error`.
    pub fn print_at(&self, statement: usize, width: usize) -> String {
        if width < NARROWEST {
            return refused(&format!(
                "a print is at least {NARROWEST} characters wide, and {width} was asked for"
            ));
        }
        let prints = self.prints.borrow();
        let Some((stdout, shown)) = statement.checked_sub(1).and_then(|at| prints.get(at)) else {
            return refused(&format!(
                "the last run ran {}, and none is numbered {statement}",
                match prints.len() {
                    0 => "no statement".to_string(),
                    1 => "1 statement, numbered 1".to_string(),
                    n => format!("{n} statements, numbered 1 to {n}"),
                }
            ));
        };
        format!(
            "{{\"type\": \"printed\", \"statement\": {statement}, \"width\": {width}, \"stdout\": {}}}",
            json_string(&zeff_core::runtime::printed_at(stdout, shown, width))
        )
    }

    /// A plot of the last run as its scene (section 8.3, point 5; Zemp Specification, section
    /// 7): the scales with their breaks and labels, the panels, and every layer's marks in the
    /// units of the data, as the engine's own JSON of a scene, for an interface that reads the
    /// values under the pointer, shows one panel alone, or sets two plots on shared axes. A
    /// line of JSON of type `scene`, or of type `error`.
    pub fn plot_scene(&self, index: usize) -> String {
        let plots = self.plots.borrow();
        match plots.get(index) {
            Some(made) => format!(
                "{{\"type\": \"scene\", \"index\": {index}, \"scene\": {}}}",
                zeff_plot::scene::scene(&made.plot).json().write_line()
            ),
            None => refused(&format!("the last run made no plot numbered {index}")),
        }
    }

    /// Whether the project holds a file or a folder at this path from its folder.
    pub fn holds(&self, path: &str) -> bool {
        use zeff_engine::fs::FileSystem;
        let at = format!("{ROOT}/{}", path.trim_start_matches('/'));
        self.fs.is_file(Path::new(&at)) || self.fs.is_dir(Path::new(&at))
    }

    /// Put a file into the project, by its path from the project's folder (`data/wages.csv`,
    /// `functions/clean.zeff`, `zeff.toml`); a file of that path is replaced.
    pub fn add_file(&self, path: &str, bytes: &[u8]) {
        self.fs
            .add_file(format!("{ROOT}/{}", path.trim_start_matches('/')), bytes);
    }

    /// Run the text of `script` (its path in the project). `tell` is given each event as a
    /// line of JSON while the run goes; `cancelled` is asked before each step. The budget is
    /// the project's `memory`, or the browser's default for a device of `device_bytes`. What
    /// comes back is the outcome, as a line of JSON of type `done`.
    pub fn run(
        &self,
        script: &str,
        text: &str,
        device_bytes: Option<u64>,
        cancelled: fn() -> bool,
        tell: &dyn Fn(&str),
    ) -> String {
        let project = match Project::open(&self.fs, Path::new(ROOT)) {
            Ok(project) => project,
            Err(_) => {
                self.frames.borrow_mut().clear();
                self.plots.borrow_mut().clear();
                self.prints.borrow_mut().clear();
                return self.done(
                    1,
                    "",
                    "error[Z0709]: the project's `zeff.toml` could not be read\n",
                );
            }
        };
        let options = RunOptions {
            // The project's own `memory` comes first; without one the browser has a default,
            // where the command line has none.
            memory: match project.manifest.memory {
                Some(_) => None,
                None => Some(default_budget(device_bytes)),
            },
            ..RunOptions::default()
        };
        let control = Control::asking(cancelled);
        let listener = |event: &Event| tell(&event_json(event));
        let (outcome, showing) = run_script_showing(
            &self.fs,
            &project,
            script,
            text,
            &options,
            &control,
            Some(&listener),
        );
        *self.frames.borrow_mut() = showing.frames;
        *self.plots.borrow_mut() = showing.plots;
        *self.prints.borrow_mut() = outcome
            .output
            .statements
            .iter()
            .zip(showing.prints)
            .map(|(statement, shown)| (statement.stdout.clone(), shown))
            .collect();
        // Each statement with its result's type and dimensions and what it depends on (open
        // decision #175, C5), read from the script as it is checked in the project.
        let statements = match zeff_engine::registry::registry() {
            Ok(registry) => {
                let checked =
                    zeff_engine::run::check_in_project(&self.fs, &project, script, text, registry);
                zeff_engine::statements::json(&zeff_engine::statements::statement_facts(
                    &checked.checked.program,
                    script,
                    text,
                    registry,
                    &outcome.output,
                ))
            }
            Err(_) => "[]".to_string(),
        };
        self.done_with(
            outcome.exit_code,
            &outcome.output.stdout,
            &outcome.output.stderr,
            &statements,
        )
    }

    /// "Report a problem" (Language Reference, section 11.1, "Bug reports"; release 0.8.0,
    /// step 20a): the report `zeff report-bug` builds, of the text of `script` as it stands.
    /// The script is run here, once whole and once with the statements kept; nothing is
    /// written and nothing is sent: the report comes back as a line of JSON of type `report`
    /// for the interface to show, with its name, its kind (`error`, `internal error` or
    /// `wrong result`), its code, how many statements it keeps of how many, and the files the
    /// kept statements read, by name. A result believed wrong is named by `line` with its
    /// `reference` (the name of the user's file and its text). Where there is nothing to
    /// report, the line is of type `error` and says why.
    pub fn report_bug(
        &self,
        script: &str,
        text: &str,
        line: Option<u32>,
        reference: Option<(&str, &str)>,
        platform: &str,
        device_bytes: Option<u64>,
    ) -> String {
        use zeff_engine::bug_report::{Asked, build};
        let failed = |message: &str| {
            format!(
                "{{\"type\": \"error\", \"message\": {}}}",
                json_string(message)
            )
        };
        let Ok(project) = Project::open(&self.fs, Path::new(ROOT)) else {
            return failed("the project's `zeff.toml` could not be read");
        };
        let asked = Asked {
            script,
            text,
            line,
            reference,
            include_data: false,
            synthetic: false,
            trials: zeff_engine::bug_report::TRIALS,
            platform,
            options: RunOptions {
                memory: match project.manifest.memory {
                    Some(_) => None,
                    None => Some(default_budget(device_bytes)),
                },
                ..RunOptions::default()
            },
        };
        match build(&self.fs, &project, &asked) {
            Ok(report) => {
                let files: Vec<String> = report
                    .files
                    .iter()
                    .map(|(name, _)| json_string(name))
                    .collect();
                format!(
                    "{{\"type\": \"report\", \"name\": {}, \"kind\": {}, \"code\": {}, \"kept\": {}, \"of\": {}, \"reduced\": {}, \"files\": [{}], \"text\": {}}}",
                    json_string(&report.name),
                    json_string(report.kind.name()),
                    report
                        .code
                        .as_deref()
                        .map_or("null".to_string(), json_string),
                    report.kept,
                    report.of,
                    report.reduced,
                    files.join(", "),
                    json_string(&report.text)
                )
            }
            Err(none) => failed(&none.to_string()),
        }
    }

    /// The outcome of a run as a line of JSON, with the plots its statements made, each with
    /// its number, the name that binds it where one does, the line of its statement, its kind
    /// (the marks of its layers) and its theme, and the frames its bindings hold, each by
    /// name with its rows and variables: what an interface lists before it asks for one.
    fn done(&self, exit_code: i32, stdout: &str, stderr: &str) -> String {
        self.done_with(exit_code, stdout, stderr, "[]")
    }

    /// [`Host::done`] with the statements of the run, a JSON list: each with its place, its
    /// lines, the name it binds, its result's type and dimensions, what it printed and said,
    /// the statements it reads and the functions it calls, and whether it sets the seed,
    /// draws, reads a file or writes one.
    fn done_with(&self, exit_code: i32, stdout: &str, stderr: &str, statements: &str) -> String {
        let frames: Vec<String> = self
            .frames
            .borrow()
            .iter()
            .map(|(name, frame)| {
                format!(
                    "{{\"name\": {}, \"rows\": {}, \"variables\": {}}}",
                    json_string(name),
                    frame.table.rows(),
                    frame.table.names().len()
                )
            })
            .collect();
        let plots: Vec<String> = self
            .plots
            .borrow()
            .iter()
            .enumerate()
            .map(|(index, made)| {
                let mut marks: Vec<&str> = Vec::new();
                for layer in &made.plot.layers {
                    let mark = layer.mark.name();
                    if !marks.contains(&mark) {
                        marks.push(mark);
                    }
                }
                format!(
                    "{{\"index\": {index}, \"name\": {}, \"line\": {}, \"kind\": {}, \"theme\": {}}}",
                    made.name.as_deref().map_or("null".to_string(), json_string),
                    made.line,
                    json_string(&marks.join(", ")),
                    json_string(made.plot.theme.name.name())
                )
            })
            .collect();
        format!(
            "{{\"type\": \"done\", \"exit_code\": {exit_code}, \"stdout\": {}, \"stderr\": {}, \"statements\": {statements}, \"plots\": [{}], \"frames\": [{}]}}",
            json_string(stdout),
            json_string(stderr),
            plots.join(", "),
            frames.join(", ")
        )
    }
}
