//! Build tooling. `cargo run -p xtask -- check-tiers` enforces the tier rules of section 8.1.

mod tiers;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("check-tiers") => check_tiers(),
        _ => {
            eprintln!("usage: cargo run -p xtask -- check-tiers");
            ExitCode::from(2)
        }
    }
}

fn check_tiers() -> ExitCode {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            eprintln!(
                "cargo metadata failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!("could not run cargo: {error}");
            return ExitCode::from(2);
        }
    };
    let metadata = match serde_json::from_slice(&output.stdout) {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("could not read the output of cargo metadata: {error}");
            return ExitCode::from(2);
        }
    };
    match tiers::check(&metadata) {
        Ok(violations) if violations.is_empty() => {
            println!(
                "tier rules hold: core and the kernel crates are separate and every call goes down"
            );
            ExitCode::SUCCESS
        }
        Ok(violations) => {
            for violation in &violations {
                eprintln!("tier rule broken: {violation}");
            }
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
