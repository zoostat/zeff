//! Build tooling. `cargo run -p xtask -- check-tiers` enforces the tier rules of section 8.1;
//! `cargo run -p xtask -- routine` is the whole routine of a step (testing handbook, 2.1).

mod datasets;
mod grammar;
mod identity;
mod messages;
mod routine;
mod tiers;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("routine") => routine::routine(),
        Some("check-tiers") => check_tiers(),
        Some("compare-identity") => compare_identity(),
        Some("check-messages") => check_messages(),
        Some("check-grammar") => grammar::check(),
        Some("grammar-words") => grammar::write_words(),
        Some("build-datasets") => datasets::build(),
        Some("export-samples") => export_samples(),
        Some("archive") => archive(),
        _ => {
            eprintln!(
                "usage: cargo run -p xtask -- routine\n       \
                 cargo run -p xtask -- check-tiers\n       \
                 cargo run -p xtask -- check-messages\n       \
                 cargo run -p xtask -- check-grammar\n       \
                 cargo run -p xtask -- grammar-words\n       \
                 cargo run -p xtask -- build-datasets\n       \
                 cargo run -p xtask -- export-samples\n       \
                 cargo run -p xtask -- archive <engine> <version> <target> <folder> [<launcher>] [--tar]\n       \
                 cargo run -p xtask -- compare-identity <folder> <number of reports>"
            );
            ExitCode::from(2)
        }
    }
}

/// `cargo run -p xtask -- archive <engine> <version> <target> <folder> [<launcher>]`: the
/// archive of a built engine for one target, written into `<folder>` under the name the
/// launcher reads, and `SHA256SUMS` rewritten to list every archive the folder holds (release
/// 0.9.0, step 18). Signing `SHA256SUMS` is the release workflow's, with the key no source
/// file holds.
fn archive() -> ExitCode {
    use zeff_launcher::archive::{ArchiveName, Contents, Target, pack};
    let args: Vec<String> = std::env::args().skip(2).collect();
    let (Some(engine), Some(version), Some(target), Some(folder)) =
        (args.first(), args.get(1), args.get(2), args.get(3))
    else {
        eprintln!(
            "usage: cargo run -p xtask -- archive <engine> <version> <target> <folder> [<launcher>]"
        );
        return ExitCode::from(2);
    };
    let Some(version) = zeff_launcher::Version::parse(version) else {
        eprintln!("`{version}` is not a version: three numbers, as 0.9.1 or 0.10.0-rc.1");
        return ExitCode::from(2);
    };
    let Some(target) = Target::ALL.into_iter().find(|known| known.name() == target) else {
        let known: Vec<&str> = Target::ALL.iter().map(|target| target.name()).collect();
        eprintln!("`{target}` is not a target: one of {}", known.join(", "));
        return ExitCode::from(2);
    };
    let read = |path: &String| {
        std::fs::read(path).map_err(|error| eprintln!("cannot read {path}: {error}"))
    };
    let Ok(engine) = read(engine) else {
        return ExitCode::from(1);
    };
    // `--tar`, last: the tar alone, which the release workflow compresses with `zstd -19`.
    let tar_alone = args.last().is_some_and(|last| last == "--tar");
    let launcher = match args.get(4).filter(|path| *path != "--tar").map(read) {
        Some(Ok(bytes)) => Some(bytes),
        Some(Err(())) => return ExitCode::from(1),
        None => None,
    };
    let name = ArchiveName::of(&version, target);
    let folder = std::path::Path::new(folder);
    if tar_alone {
        let contents = Contents {
            engine: &engine,
            launcher: launcher.as_deref(),
            beside: &[],
        };
        let Some(tar) = zeff_launcher::archive::pack_tar(&name, &contents) else {
            eprintln!(
                "the archive for {} is a zip, which has no tar",
                target.name()
            );
            return ExitCode::from(2);
        };
        let path = folder.join(format!("{}.tar", name.top()));
        let written = tar.and_then(|bytes| {
            std::fs::create_dir_all(folder)?;
            std::fs::write(&path, bytes)
        });
        return match written {
            Ok(()) => {
                println!("wrote {}", path.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot write the tar: {error}");
                ExitCode::from(1)
            }
        };
    }
    let written = pack(
        &name,
        &Contents {
            engine: &engine,
            launcher: launcher.as_deref(),
            beside: &[],
        },
    )
    .and_then(|bytes| {
        std::fs::create_dir_all(folder)?;
        std::fs::write(folder.join(&name.file), &bytes)?;
        // Every archive of the folder, in the order of their names.
        let mut archives = Vec::new();
        for entry in std::fs::read_dir(folder)? {
            let file = entry?.file_name().to_string_lossy().into_owned();
            if ArchiveName::parse(&file).is_some() {
                archives.push((file.clone(), std::fs::read(folder.join(&file))?));
            }
        }
        archives.sort();
        let listed: Vec<(&str, &[u8])> = archives
            .iter()
            .map(|(file, bytes)| (file.as_str(), bytes.as_slice()))
            .collect();
        std::fs::write(
            folder.join(zeff_launcher::trust::SUMS),
            zeff_launcher::trust::sums_of(&listed),
        )?;
        Ok(bytes.len())
    });
    match written {
        Ok(size) => {
            println!(
                "wrote {} ({size} bytes) and {}",
                folder.join(&name.file).display(),
                folder.join(zeff_launcher::trust::SUMS).display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("cannot write the archive: {error}");
            ExitCode::from(1)
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
    let saved = zeff_data::save::sample();
    let csv = |delim, bom, codes| {
        zeff_data::save::delimited_bytes(&saved, delim, bom, codes)
            .expect("the sample has no opaque column")
    };
    let files = [
        ("sample.docx", zeff_results::export::docx(&sample)),
        (
            "sample.tex",
            zeff_results::export::tex(&sample).into_bytes(),
        ),
        ("sample.csv", csv(b',', false, false)),
        ("sample.tsv", csv(b'\t', false, false)),
        ("sample_bom_codes.csv", csv(b',', true, true)),
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
