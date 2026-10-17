//! The `zeff` program.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("error: cannot tell which folder this is");
        return ExitCode::from(2);
    };
    let fs = zeff_engine::fs::StdFileSystem;
    let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr().lock());
    let code = zeff_cli::run(&args, &cwd, &fs, &mut out, &mut err);
    let _ = out.flush();
    let _ = err.flush();
    ExitCode::from(u8::try_from(code).unwrap_or(2))
}
