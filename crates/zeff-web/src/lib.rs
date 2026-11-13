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
use zeff_engine::run::{RunOptions, run_script_under};

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

/// A project in memory, and the runs made in it.
pub struct Host {
    fs: MemoryFileSystem,
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
        Host { fs }
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
                return done(
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
        let outcome = run_script_under(
            &self.fs,
            &project,
            script,
            text,
            &options,
            &control,
            Some(&listener),
        );
        done(
            outcome.exit_code,
            &outcome.output.stdout,
            &outcome.output.stderr,
        )
    }
}

/// The outcome of a run as a line of JSON.
fn done(exit_code: i32, stdout: &str, stderr: &str) -> String {
    format!(
        "{{\"type\": \"done\", \"exit_code\": {exit_code}, \"stdout\": {}, \"stderr\": {}}}",
        json_string(stdout),
        json_string(stderr)
    )
}
