//! The `zeff` program.
//!
//! The command runs on a thread with a large stack: the checker and the interpreter recurse over
//! the syntax tree, whose nesting the language limits to 256 levels, and the parser alone needs
//! close to a megabyte at that depth (docs/decisions.md, 18 October 2026, nesting limits).

use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// The stack of the thread the command runs on.
const STACK_SIZE: usize = 64 << 20;

/// `ZEFF_HOME`, or `.zeff` in the home folder (Engine and Project Specification, section 2.1).
fn zeff_home() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("ZEFF_HOME").filter(|h| !h.is_empty()) {
        return Some(PathBuf::from(home));
    }
    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(variable)
        .filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(".zeff"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("error: cannot tell which folder this is");
        return ExitCode::from(2);
    };
    // A failure inside the engine is reported as Z9001 by the interpreter (Language Reference,
    // section 11.1); the default hook's own report, with source paths, would only add noise.
    std::panic::set_hook(Box::new(|_| {}));
    let worker = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let fs = zeff_engine::fs::StdFileSystem;
            let stdin = std::io::stdin();
            let mut input = stdin.lock();
            let mut env = zeff_cli::Environment {
                zeff_home: zeff_home(),
                no_color: std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
                stderr_is_terminal: std::io::stderr().is_terminal(),
                stdin_is_terminal: std::io::stdin().is_terminal(),
                input: &mut input,
            };
            let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr().lock());
            let code = zeff_cli::run(&args, &cwd, &fs, &mut env, &mut out, &mut err);
            let _ = out.flush();
            let _ = err.flush();
            code
        });
    let code = match worker {
        Ok(handle) => handle.join().unwrap_or_else(|_| {
            eprintln!(
                "error[Z9001]: internal error: this is a bug in Zeff, not in your script\n  = help: please report what you ran"
            );
            2
        }),
        Err(_) => 2,
    };
    ExitCode::from(u8::try_from(code).unwrap_or(2))
}
