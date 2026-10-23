//! Build tooling. `cargo run -p xtask -- check-tiers` enforces the tier rules of section 8.1.

mod datasets;
mod identity;
mod messages;
mod tiers;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("check-tiers") => check_tiers(),
        Some("compare-identity") => compare_identity(),
        Some("check-messages") => check_messages(),
        Some("build-datasets") => datasets::build(),
        Some("export-samples") => export_samples(),
        _ => {
            eprintln!(
                "usage: cargo run -p xtask -- check-tiers\n       \
                 cargo run -p xtask -- check-messages\n       \
                 cargo run -p xtask -- build-datasets\n       \
                 cargo run -p xtask -- export-samples\n       \
                 cargo run -p xtask -- compare-identity <folder> <number of reports>"
            );
            ExitCode::from(2)
        }
    }
}

/// `cargo run -p xtask -- export-samples`: the sample table written as Word and LaTeX into
/// `tests/reference/exports/` (release 0.4.0, step 13). Continuous integration rebuilds the two
/// files, checks the Word file with Python's `zipfile` and the XML parser of the standard library
/// (`tests/reference/exports.py`), and compares both with the committed copies.
fn export_samples() -> ExitCode {
    let folder = std::path::Path::new("tests/reference/exports");
    let sample = zeff_results::export::sample();
    let files = [
        ("sample.docx", zeff_results::export::docx(&sample)),
        (
            "sample.tex",
            zeff_results::export::tex(&sample).into_bytes(),
        ),
    ];
    if let Err(error) = std::fs::create_dir_all(folder) {
        eprintln!("cannot create {}: {error}", folder.display());
        return ExitCode::from(1);
    }
    for (name, bytes) in files {
        let path = folder.join(name);
        if let Err(error) = std::fs::write(&path, bytes) {
            eprintln!("cannot write {}: {error}", path.display());
            return ExitCode::from(1);
        }
        println!("wrote {}", path.display());
    }
    ExitCode::SUCCESS
}

/// Every diagnostic code written in the sources has a snapshot of its wording (section 11) and an
/// explained page (section 11.1), and no page is left for a code nothing raises.
fn check_messages() -> ExitCode {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (sources, snapshots) = match messages::gather(&root) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("could not read the workspace: {error}");
            return ExitCode::from(2);
        }
    };
    let gaps = messages::missing(&sources, &snapshots);
    let total: std::collections::BTreeSet<String> = sources
        .iter()
        .flat_map(|(_, text)| messages::codes_in_source(text))
        .collect();
    let pages = match messages::pages(&root) {
        Ok(pages) => pages,
        Err(error) => {
            eprintln!("could not read the explained pages: {error}");
            return ExitCode::from(2);
        }
    };
    let (without_page, stale) = messages::page_gaps(&total, &pages);
    if gaps.is_empty() && without_page.is_empty() && stale.is_empty() {
        println!(
            "every one of the {} diagnostic codes in the sources has a snapshot ({} snapshot files) and an explained page",
            total.len(),
            snapshots.len()
        );
        ExitCode::SUCCESS
    } else {
        for (code, files) in &gaps {
            eprintln!("{code} has no snapshot; raised in {}", files.join(", "));
        }
        if !gaps.is_empty() {
            eprintln!(
                "{} of {} codes have no snapshot: add a test that renders each with insta",
                gaps.len(),
                total.len()
            );
        }
        for code in &without_page {
            eprintln!("{code} has no explained page: write crates/zeff-engine/codes/{code}.md");
        }
        for code in &stale {
            eprintln!("crates/zeff-engine/codes/{code}.md explains a code nothing raises");
        }
        ExitCode::from(1)
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

/// Compare the `*.txt` reports found under a folder: `compare-identity <folder> <how many>`.
fn compare_identity() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(2).collect();
    let [folder, count] = args.as_slice() else {
        eprintln!("usage: cargo run -p xtask -- compare-identity <folder> <number of reports>");
        return ExitCode::from(2);
    };
    let Ok(count) = count.parse::<usize>() else {
        eprintln!("`{count}` is not a number");
        return ExitCode::from(2);
    };
    let mut reports = Vec::new();
    if let Err(error) = collect(
        std::path::Path::new(folder),
        std::path::Path::new(folder),
        &mut reports,
    ) {
        eprintln!("could not read {folder}: {error}");
        return ExitCode::from(2);
    }
    reports.sort();
    let problems = identity::compare(&reports, count);
    if problems.is_empty() {
        println!("identical: {} reports agree on every case", reports.len());
        for (label, _) in &reports {
            println!("  {label}");
        }
        ExitCode::SUCCESS
    } else {
        for problem in &problems {
            eprintln!("not identical: {problem}");
        }
        ExitCode::from(1)
    }
}

fn collect(
    root: &std::path::Path,
    folder: &std::path::Path,
    reports: &mut Vec<(String, String)>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, reports)?;
        } else if path.extension().is_some_and(|e| e == "txt") {
            let label = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            reports.push((label, std::fs::read_to_string(&path)?));
        }
    }
    Ok(())
}
