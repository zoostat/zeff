//! `zeff`, the launcher: chooses the engine for the command and hands the command over to it,
//! arguments, streams and exit code unchanged (Engine and Project Specification, section 2.1).

use std::ffi::OsString;
use std::process::{Command, ExitCode};
use zeff_diagnostics::{Diagnostic, Sources};
use zeff_launcher::{Choice, EXIT_ENGINE, Platform, VERSION, choose, refusal};

fn say(diagnostic: &Diagnostic) {
    eprint!("{}", diagnostic.render(&Sources::new()));
}

/// Hand the command to `engine`. On Unix the engine takes the launcher's place, so its
/// streams, its signals and its exit code are its own; elsewhere the launcher waits for it and
/// ends as it ended.
fn hand_over(engine: &std::path::Path, version: &str, args: &[OsString]) -> ExitCode {
    let mut command = Command::new(engine);
    command.args(args).env("ZEFF_LAUNCHER", VERSION);
    #[cfg(unix)]
    let failure = {
        use std::os::unix::process::CommandExt;
        command.exec()
    };
    #[cfg(not(unix))]
    let failure = match command.status() {
        // An engine ended by a signal has no code: an internal error, as the engine's own.
        Ok(status) => return ExitCode::from(status.code().unwrap_or(2).clamp(0, 255) as u8),
        Err(e) => e,
    };
    say(&Diagnostic::error(
        "Z0710",
        format!("the engine Zeff {version} could not be started: {failure}"),
    )
    .help(format!(
        "its file is `{}`; `zeff engine install {version}` installs it afresh",
        engine.display()
    )));
    ExitCode::from(EXIT_ENGINE)
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let readable: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    let choice = choose(
        &readable,
        &cwd,
        &|name| std::env::var(name).ok(),
        Platform::host(),
    );
    match &choice {
        Choice::Run { version, engine } => hand_over(engine, version, &args),
        refused => {
            if let Some(diagnostic) = refusal(refused) {
                say(&diagnostic);
            }
            ExitCode::from(EXIT_ENGINE)
        }
    }
}
