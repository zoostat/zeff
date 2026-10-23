//! `export` through the engine (Engine and Project Specification, section 2, "Writes"; release
//! 0.4.0, step 13): a table written as Word and LaTeX under the project folder, `output/` created
//! as needed and a bare name written there, a file replaced, and every message.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::{FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::run::run_script;

const WAGES: &str = "id,wage,hours\n1,23.4,40\n2,19.0,35\n3,31.2,\n";

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/wages.csv", WAGES);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn go(fs: &MemoryFileSystem, project: &Project, script: &str) -> (i32, String, String) {
    let outcome = run_script(fs, project, "scripts/main.zeff", script);
    let stderr: String = outcome
        .output
        .stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect();
    (outcome.exit_code, outcome.output.stdout, stderr)
}

const SETUP: &str = "let wages = import(\"data/wages.csv\")\n";

#[test]
fn a_table_is_written_as_word_and_latex_under_the_project() {
    let (fs, project) = project();
    assert!(!fs.is_dir(Path::new("/p/output")));
    let script = format!(
        "{SETUP}wages.describe().export(\"output/wages.docx\")\nwages.describe().export(\"wages.tex\")\n\
         wages.describe().export(\"data/summary.tex\")\nwages.describe().export(\"wages.tex\")\n"
    );
    let (code, stdout, stderr) = go(&fs, &project, &script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "", "an export prints nothing");
    assert_eq!(stderr, "");
    // `output/` was created for the first write; a bare name went there too.
    let docx = fs.read(Path::new("/p/output/wages.docx")).unwrap();
    assert!(docx.starts_with(b"PK\x03\x04"));
    let inside = String::from_utf8_lossy(&docx);
    assert!(inside.contains("word/document.xml"), "{inside}");
    assert!(inside.contains("wages · 3 rows × 3 variables"), "{inside}");
    assert!(inside.contains("<w:jc w:val=\"right\"/>"));
    let tex = fs.read_to_string(Path::new("/p/output/wages.tex")).unwrap();
    assert!(
        tex.starts_with("% requires \\usepackage{booktabs}\n% wages · 3 rows"),
        "{tex}"
    );
    assert!(
        tex.contains(
            "\\toprule\nvariable & type & non-missing & missing & example \\\\\n\\midrule\n"
        ),
        "{tex}"
    );
    // An existing folder of the project takes a file as well; the written file is the writer's.
    let summary = fs.read_to_string(Path::new("/p/data/summary.tex")).unwrap();
    assert!(summary.contains("\\begin{tabular}"), "{summary}");
    assert!(fs.is_file(Path::new("/p/data/wages.csv")));
}

#[test]
fn every_message() {
    let cases: &[(&str, &str)] = &[
        (
            "outside_the_project",
            "wages.describe().export(\"../wages.docx\")\n",
        ),
        (
            "absolute_path",
            "wages.describe().export(\"/tmp/wages.docx\")\n",
        ),
        (
            "data_root",
            "wages.describe().export(\"zeff:wages.docx\")\n",
        ),
        (
            "missing_folder",
            "wages.describe().export(\"results/wages.docx\")\n",
        ),
        (
            "missing_folder_under_output",
            "wages.describe().export(\"output/tables/wages.docx\")\n",
        ),
        (
            "unknown_extension",
            "wages.describe().export(\"wages.xlsx\")\n",
        ),
        (
            "no_extension",
            "wages.describe().export(\"output/wages\")\n",
        ),
        ("dot_only", "wages.describe().export(\".docx\")\n"),
        ("frame_receiver", "wages.export(\"wages.docx\")\n"),
        ("path_not_text", "wages.describe().export(5)\n"),
    ];
    let mut text = String::new();
    for (name, script) in cases {
        let (fs, project) = project();
        let (code, stdout, stderr) = go(&fs, &project, &format!("{SETUP}{script}"));
        assert_eq!(code, 1, "{name}: {stderr}");
        assert_eq!(stdout, "", "{name}");
        assert!(
            !fs.is_dir(Path::new("/p/output")),
            "{name}: nothing was written"
        );
        text.push_str(&format!("--- {name}\n{stderr}"));
    }
    insta::assert_snapshot!("every_message", text);
}
