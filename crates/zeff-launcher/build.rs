//! What the launcher can say of itself (Engine and Project Specification, section 2.1, "Trust";
//! release 0.9.0, step 21a): the commit it was built from and the compiler that built it, read
//! once when the crate is compiled and printed by `zeff doctor`. A build from
//! sources without their history says `unknown` for the commit. Nothing here reads a clock.

use std::process::Command;

fn said(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (output.status.success() && !text.is_empty()).then_some(text)
}

fn main() {
    let commit =
        said("git", &["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let toolchain = said(&rustc, &["--version"]).unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=ZEFF_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=ZEFF_BUILD_TOOLCHAIN={toolchain}");
    // The release workflow sets this for the builds it signs; every other build is a source
    // build and says so.
    println!("cargo:rerun-if-env-changed=ZEFF_OFFICIAL_BUILD");
    // The key of a trial of the release workflow, compiled into the launcher of that trial.
    println!("cargo:rerun-if-env-changed=ZEFF_TRIAL_KEY");
    // Built again when the commit changes: the head of the repository and the branch it names.
    if let Some(folder) = said("git", &["rev-parse", "--git-dir"]) {
        let folder = std::path::PathBuf::from(folder);
        let folder = if folder.is_absolute() {
            folder
        } else {
            std::env::current_dir().unwrap_or_default().join(folder)
        };
        println!("cargo:rerun-if-changed={}", folder.join("HEAD").display());
        if let Some(branch) = said("git", &["symbolic-ref", "-q", "HEAD"]) {
            println!("cargo:rerun-if-changed={}", folder.join(branch).display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
