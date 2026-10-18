//! The `zeff` program.
//!
//! The command runs on a thread with a large stack: the checker and the interpreter recurse over
//! the syntax tree, whose nesting the language limits to 256 levels, and the parser alone needs
//! close to a megabyte at that depth (docs/decisions.md, 18 October 2026, nesting limits).

use std::io::Write;
use std::process::ExitCode;

/// The stack of the thread the command runs on.
const STACK_SIZE: usize = 64 << 20;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("error: cannot tell which folder this is");
        return ExitCode::from(2);
    };
    let worker = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let fs = zeff_engine::fs::StdFileSystem;
            let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr().lock());
            let code = zeff_cli::run(&args, &cwd, &fs, &mut out, &mut err);
            let _ = out.flush();
            let _ = err.flush();
            code
        });
    let code = match worker {
        Ok(handle) => handle.join().unwrap_or(2),
        Err(_) => 2,
    };
    ExitCode::from(u8::try_from(code).unwrap_or(2))
}
