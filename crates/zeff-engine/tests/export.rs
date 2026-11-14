//! `export` through the engine (Engine and Project Specification, section 2, "Writes"; release
//! 0.4.0, step 13): a table written as Word and LaTeX under the project folder, `output/` created
//! as needed and a bare name written there, a file replaced, and every message. A plot written
//! as a figure by its extension, SVG, PDF or PNG (release 0.7.0, steps 21d and 22b3a).

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

/// A test's result written as LaTeX (release 0.7.0, step 40a). Until this step a result
/// without a table of groups was written as a `tabular` with no column, which LaTeX refuses,
/// and the Greek letters and the `≠` of the hypotheses were written as they are, which
/// pdfLaTeX refuses and XeLaTeX drops; a mutant of the writer that no test caught led to
/// both. The lines of such a result now stand in one column between two rules, every symbol
/// is its command, and where `pdflatex` is installed the files are compiled.
#[test]
fn a_tests_result_is_written_as_latex_that_compiles() {
    let (fs, project) = project();
    let script = "let d = frame(x = [1.0, 3.0, 2.0, 5.0, 4.0, 7.0], z = [0.5, 2.0, 2.5, 3.0, 5.0, 6.0], g = [\"a\", \"a\", \"a\", \"b\", \"b\", \"b\"], u = [true, false, true, true, false, true])\n\
                  d.t_test(x, mu: 0).export(\"one.tex\")\n\
                  d.t_test(x, z, paired: true).export(\"paired.tex\")\n\
                  d.t_test(x, group: g).export(\"two.tex\")\n\
                  d.proportion_test(u, p: 0.5).export(\"share.tex\")\n\
                  d.wilcoxon_test(x, mu: 1).export(\"rank.tex\")\n\
                  d.t_test(x, mu: 0).export(\"one.docx\")\n";
    let (code, stdout, stderr) = go(&fs, &project, script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "", "an export prints nothing");
    let read = |name: &str| {
        fs.read_to_string(Path::new(&format!("/p/output/{name}.tex")))
            .unwrap()
    };
    let one = read("one");
    assert!(
        one.starts_with(
            "% requires \\usepackage{booktabs}\n% t_test · one-sample · x\n\\begin{tabular}{@{}l@{}}\n\\toprule\n  H0: the population mean of x equals 0            $\\mu$ = 0 \\\\\n  H1: the population mean of x differs from 0      $\\mu$ $\\neq$ 0 \\\\\n"
        ),
        "{one}"
    );
    assert!(
        one.contains("t = 4.16   df = 5   p = 0.009 \\\\\n"),
        "{one}"
    );
    assert!(
        one.contains("decision at $\\alpha$ = 0.05: reject H0"),
        "{one}"
    );
    assert!(one.ends_with("\\bottomrule\n\\end{tabular}\n"), "{one}");
    // A result with a table of groups keeps its grid, and its lines under it.
    let two = read("two");
    assert!(two.contains("\\begin{tabular}{lrrr}\n"), "{two}");
    assert!(two.contains("$\\mu$(a) $-$ $\\mu$(b) $\\neq$ 0"), "{two}");
    let files = [
        ("one", one),
        ("paired", read("paired")),
        ("two", two),
        ("share", read("share")),
        ("rank", read("rank")),
    ];
    // Outside the comment with the title, nothing but ASCII is left for LaTeX to stumble on.
    for (name, tex) in &files {
        for line in tex.lines().filter(|line| !line.starts_with('%')) {
            assert!(line.is_ascii(), "{name}: {line}");
        }
        assert!(!tex.contains("\\begin{tabular}{}"), "{name}: {tex}");
    }
    // As Word: the lines as paragraphs, and no table whose row has no cell.
    let docx = fs.read(Path::new("/p/output/one.docx")).unwrap();
    let inside = String::from_utf8_lossy(&docx);
    assert!(
        inside.contains("H0: the population mean of x equals 0"),
        "{inside}"
    );
    assert!(!inside.contains("<w:tbl>"), "{inside}");
    // Compiled, where there is a LaTeX to compile with.
    let installed = std::process::Command::new("pdflatex")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !installed {
        return;
    }
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("latex_of_tests");
    std::fs::create_dir_all(&folder).unwrap();
    for (name, tex) in &files {
        let document = format!(
            "\\documentclass{{article}}\n\\usepackage{{booktabs}}\n\\begin{{document}}\n{tex}\\end{{document}}\n"
        );
        std::fs::write(folder.join(format!("{name}.tex")), document).unwrap();
        let run = std::process::Command::new("pdflatex")
            .args(["-interaction=nonstopmode", "-halt-on-error"])
            .arg(format!("{name}.tex"))
            .current_dir(&folder)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{name}.tex does not compile:\n{}",
            String::from_utf8_lossy(&run.stdout)
                .lines()
                .filter(|line| line.starts_with('!'))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
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

/// The width and the height of a PNG file, in dots, and how many dots it says there are to
/// the metre: read from its first chunk and from the chunk named `pHYs`.
fn png_size(file: &[u8]) -> (u32, u32, Option<u32>) {
    assert_eq!(file[..8], [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    assert_eq!(&file[12..16], b"IHDR");
    let number =
        |at: usize| u32::from_be_bytes([file[at], file[at + 1], file[at + 2], file[at + 3]]);
    let density = file
        .windows(4)
        .position(|window| window == b"pHYs")
        .map(|at| number(at + 4));
    (number(16), number(20), density)
}

#[test]
fn a_plot_is_written_as_pdf_and_as_png_by_its_extension() {
    let (fs, project) = project();
    let script = format!(
        "{PLOT}p.export(\"wage.pdf\")\np.export(\"wage.png\")\np.export(\"wage.svg\")\n\
         p.export(\"output/wide.PDF\", size: two_column)\np.export(\"output/wide.Png\", size: two_column)\n\
         p.export(\"output/own.pdf\", size: [120, 80.5])\np.export(\"output/own.png\", size: [120, 80.5])\n\
         p.export(\"output/noted.pdf\", notes: true)\np.export(\"output/noted.png\", notes: true)\n\
         p.export(\"output/again.pdf\", size: one_column, notes: false)\n\
         p.export(\"output/again.png\", size: one_column, notes: false)\n\
         p.export(\"output/largest.pdf\", size: [5080, 100])\n\
         w.plot(x: educ, y: wage).export(\"output/empty.pdf\")\n\
         w.plot(x: educ, y: wage).export(\"output/empty.png\")\n"
    );
    let (code, stdout, stderr) = go(&fs, &project, &script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "", "an export prints nothing");
    assert_eq!(
        stderr,
        "note: 6 rows with NA in educ or wage dropped (line 2)\n"
    );
    let read = |name: &str| fs.read(Path::new(&format!("/p/output/{name}"))).unwrap();
    let holds = |file: &[u8], words: &str| file.windows(words.len()).any(|w| w == words.as_bytes());
    // A PDF of one page as large as the page asked for, in points, with its font inside it.
    let pdf = read("wage.pdf");
    assert!(pdf.starts_with(b"%PDF-1.4\n") && pdf.ends_with(b"%%EOF\n"));
    assert!(holds(&pdf, "/MediaBox [0 0 252.283 187.087]"));
    assert!(holds(&pdf, "/FontFile2") && holds(&pdf, "/BaseFont /IBMPlexSans"));
    assert!(holds(&read("wide.PDF"), "/MediaBox [0 0 518.74 311.811]"));
    assert!(holds(&read("own.pdf"), "/MediaBox [0 0 340.157 228.189]"));
    // The largest page a PDF may have is written.
    assert!(holds(&read("largest.pdf"), "/MediaBox [0 0 14400 283.465]"));
    // A PNG at 300 dots to the inch, which it says as 11,811 to the metre: 89 mm by 66 is
    // 1051 dots by 779, what is left of a dot cut.
    let png = read("wage.png");
    assert_eq!(png_size(&png), (1051, 779, Some(11811)));
    assert_eq!(png_size(&read("wide.Png")), (2161, 1299, Some(11811)));
    assert_eq!(png_size(&read("own.png")), (1417, 950, Some(11811)));
    // The same plot at the same size is the same file, to the byte, in each format.
    assert_eq!(read("again.pdf"), pdf);
    assert_eq!(read("again.png"), png);
    // With its notes a figure is another file; the note is words in the PDF, which a file
    // of dots has none of.
    assert_ne!(read("noted.pdf"), pdf);
    assert_ne!(read("noted.png"), png);
    // A plot with no layer yet is written too.
    assert!(read("empty.pdf").starts_with(b"%PDF-1.4\n"));
    assert_eq!(png_size(&read("empty.png")), (1051, 779, Some(11811)));
    // Nothing of the day, and nothing of the frame, in any of the files.
    for name in [
        "wage.pdf",
        "noted.pdf",
        "empty.pdf",
        "wage.png",
        "noted.png",
    ] {
        let file = read(name);
        for word in ["wages", "CreationDate", "ModDate", "tIME", "tEXt"] {
            assert!(!holds(&file, word), "{word} in {name}");
        }
    }
    // The three formats of one plot stand side by side.
    assert!(
        fs.read_to_string(Path::new("/p/output/wage.svg"))
            .unwrap()
            .ends_with("</svg>\n")
    );
}

#[test]
fn every_message_of_the_export_of_a_plot() {
    let cases: &[(&str, &str)] = &[
        ("a_table_format", "p.export(\"wage.docx\")\n"),
        ("a_table_format_in_capitals", "p.export(\"wage.TEX\")\n"),
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
            "too_small_for_a_pdf_too",
            "p.export(\"wage.pdf\", size: [30, 20])\n",
        ),
        (
            "too_small_for_a_png_too",
            "p.export(\"wage.png\", size: [30, 20])\n",
        ),
        (
            "too_large_for_a_pdf",
            "p.export(\"wage.pdf\", size: [5081, 100])\n",
        ),
        (
            "too_many_dots_for_a_png",
            "p.export(\"wage.png\", size: [600, 500])\n",
        ),
        (
            "no_size_for_a_png",
            "p.export(\"wage.png\", size: [0, 0])\n",
        ),
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
