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
    // `zeff engine` is the launcher's own: it is about the store, not about a project.
    if let Some(asked) = zeff_launcher::engines::read(&readable) {
        let mut out = std::io::stdout();
        let mut err = std::io::stderr();
        let code = match asked {
            Ok(asked) => zeff_launcher::engines::run(
                &asked,
                &zeff_launcher::engines::Machine {
                    env: &|name| std::env::var(name).ok(),
                    cwd: &cwd,
                    platform: Platform::host(),
                    host: zeff_launcher::archive::Target::host(),
                    key: zeff_launcher::trust::BUILT_IN_KEY,
                    fetch: None,
                },
                &mut out,
                &mut err,
            ),
            Err(wrong) => zeff_launcher::engines::usage(&mut err, &wrong),
        };
        return ExitCode::from(code);
    }
    let choice = choose(
        &readable,
        &cwd,
        &|name| std::env::var(name).ok(),
        Platform::host(),
    );
    let args = zeff_launcher::engines::for_the_engine(&args);
    match &choice {
        Choice::Run { version, engine } => hand_over(engine, version, &args),
        Choice::PinMissing { version, .. } => {
            use std::io::IsTerminal;
            let mut err = std::io::stderr();
            let mut ask = |question: &str| {
                eprint!("{question}");
                let mut answer = String::new();
                // No answer, as at the end of the input, is no.
                match std::io::stdin().read_line(&mut answer) {
                    Ok(read) if read > 0 => answer,
                    _ => "n".to_string(),
                }
            };
            let on_a_terminal = std::io::stdin().is_terminal() && err.is_terminal();
            let installed = zeff_launcher::engines::install_pinned(
                version,
                &readable,
                &zeff_launcher::engines::Machine {
                    env: &|name| std::env::var(name).ok(),
                    cwd: &cwd,
                    platform: Platform::host(),
                    host: zeff_launcher::archive::Target::host(),
                    key: zeff_launcher::trust::BUILT_IN_KEY,
                    fetch: None,
                },
                if on_a_terminal { Some(&mut ask) } else { None },
                &mut err,
            );
            match installed {
                Ok(Some(engine)) => hand_over(&engine, version, &args),
                Ok(None) => {
                    if let Some(diagnostic) = refusal(&choice) {
                        say(&diagnostic);
                    }
                    ExitCode::from(EXIT_ENGINE)
                }
                Err(code) => ExitCode::from(code),
            }
        }
        refused => {
            if let Some(diagnostic) = refusal(refused) {
                say(&diagnostic);
            }
            ExitCode::from(EXIT_ENGINE)
        }
    }
}
