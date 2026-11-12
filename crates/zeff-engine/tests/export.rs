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

// ----- a plot written as a figure (release 0.7.0, step 21d) -----

/// The wages of 2010 and a plot of them, coloured by union membership.
const PLOT: &str = "let w = dataset(wages).where(year == 2010)\nlet p = w.plot(x: educ, y: wage, color: union).points(alpha: 0.6)\n";

#[test]
fn a_plot_is_written_as_an_svg_figure_under_the_project() {
    let (fs, project) = project();
    let script = format!(
        "{PLOT}p.export(\"wage.svg\")\np.export(\"output/wide.svg\", size: two_column)\n\
         p.export(\"output/slide.svg\", size: slide)\np.export(\"output/own.svg\", size: [120, 80.5])\n\
         p.export(\"output/noted.svg\", notes: true)\np.export(\"output/again.svg\", size: one_column, notes: false)\n\
         w.plot(x: educ, y: wage).export(\"output/empty.svg\")\n"
    );
    let (code, stdout, stderr) = go(&fs, &project, &script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "", "an export prints nothing");
    // The one note is the plot's own, raised where the layer was added: an export adds none.
    assert_eq!(
        stderr,
        "note: 6 rows with NA in educ or wage dropped (line 2)\n"
    );
    let read = |name: &str| {
        fs.read_to_string(Path::new(&format!("/p/output/{name}")))
            .unwrap()
    };
    // A bare name went to `output/`; the page is one column of a journal unless a size is given.
    let one = read("wage.svg");
    assert!(one.starts_with(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"252.283pt\" height=\"187.087pt\" viewBox=\"0 0 252.283 187.087\">\n"
    ));
    assert!(one.ends_with("</svg>\n"));
    assert!(read("wide.svg").contains(" width=\"518.74pt\" height=\"311.811pt\""));
    assert!(read("slide.svg").contains(" width=\"720pt\" height=\"405.354pt\""));
    // 120 mm by 80.5 mm, in points of 1/72 inch.
    assert!(read("own.svg").contains(" width=\"340.157pt\" height=\"228.189pt\""));
    // One mark for each of the 531 rows drawn, and two for the keys of the legend.
    assert_eq!(one.matches("<circle").count(), 531 + 2);
    assert!(
        one.contains("aria-label=\"Union member\"") && one.contains("aria-label=\"Hourly wage\"")
    );
    // The note is beside the plot and not on the figure, unless it is asked for.
    assert!(!one.contains("dropped"));
    let noted = read("noted.svg");
    assert!(noted.contains("<g id=\"notes\">"));
    assert!(noted.contains("aria-label=\"note: 6 rows with NA in educ or wage dropped\""));
    assert_eq!(noted.matches("<circle").count(), 533);
    // The same plot at the same size is the same file, to the byte.
    assert_eq!(read("again.svg"), one);
    // A plot with no layer yet is written too: its axes and no mark.
    let empty = read("empty.svg");
    assert!(empty.contains("<g id=\"axis-x\">") && !empty.contains("<circle"));
    // Nothing of the frame or of its rows is in a file.
    for file in [&one, &noted, &empty] {
        assert!(!file.contains("wages") && !file.contains("\"w\"") && !file.contains("data-"));
    }
    // A file written again is replaced.
    let (code, _, _) = go(
        &fs,
        &project,
        &format!("{PLOT}p.export(\"wage.svg\", size: slide)\n"),
    );
    assert_eq!(code, 0);
    assert!(read("wage.svg").contains(" width=\"720pt\""));
}

#[test]
fn every_message_of_the_export_of_a_plot() {
    let cases: &[(&str, &str)] = &[
        ("pdf_not_yet", "p.export(\"wage.pdf\")\n"),
        (
            "png_not_yet",
            "p.export(\"output/wage.PNG\", size: slide)\n",
        ),
        ("a_table_format", "p.export(\"wage.docx\")\n"),
        ("unknown_extension", "p.export(\"wage.jpeg\")\n"),
        ("no_extension", "p.export(\"output/wage\")\n"),
        ("dot_only", "p.export(\".svg\")\n"),
        ("not_a_word", "p.export(\"wage.svg\", size: poster)\n"),
        ("a_near_word", "p.export(\"wage.svg\", size: one_colum)\n"),
        ("a_number", "p.export(\"wage.svg\", size: 89)\n"),
        ("text", "p.export(\"wage.svg\", size: \"one_column\")\n"),
        ("one_length", "p.export(\"wage.svg\", size: [89])\n"),
        (
            "three_lengths",
            "p.export(\"wage.svg\", size: [89, 66, 2])\n",
        ),
        (
            "not_numbers",
            "p.export(\"wage.svg\", size: [\"89\", \"66\"])\n",
        ),
        ("no_width", "p.export(\"wage.svg\", size: [0, 66])\n"),
        (
            "a_negative_height",
            "p.export(\"wage.svg\", size: [89, -66])\n",
        ),
        ("too_small", "p.export(\"wage.svg\", size: [30, 20])\n"),
        (
            "too_low_for_the_notes",
            "p.export(\"wage.svg\", size: [60, 26], notes: true)\n",
        ),
        (
            "notes_not_a_truth_value",
            "p.export(\"wage.svg\", notes: 1)\n",
        ),
        ("outside_the_project", "p.export(\"../wage.svg\")\n"),
        ("missing_folder", "p.export(\"figures/wage.svg\")\n"),
        (
            "size_for_a_table",
            "w.describe().export(\"w.docx\", size: slide)\n",
        ),
        (
            "notes_for_a_table",
            "w.describe().export(\"w.docx\", notes: true)\n",
        ),
        (
            "a_figure_format_for_a_table",
            "w.describe().export(\"w.svg\")\n",
        ),
        (
            "a_figure_format_for_a_result",
            "w.reg(wage ~ educ).export(\"m.pdf\")\n",
        ),
        ("a_frame", "w.export(\"w.svg\")\n"),
    ];
    let mut text = String::new();
    for (name, statement) in cases {
        let (fs, project) = project();
        let (code, stdout, stderr) = go(&fs, &project, &format!("{PLOT}{statement}"));
        assert_ne!(code, 0, "{name}: {stdout}");
        assert_eq!(stdout, "", "{name}");
        // Nothing is written when an export is refused.
        assert!(!fs.is_dir(Path::new("/p/output")), "{name}");
        text.push_str(&format!("== {name}\n{stderr}\n"));
    }
    insta::assert_snapshot!("every_message_of_a_plot", text);
}
