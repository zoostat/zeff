//! Section 13 of the specification's sample script, "Causal graph: check the model against
//! its assumptions", runs whole (release 0.10.0, steps 15 and 22b): the graph, its adjustment
//! sets, the independencies it implies, the graph drawn and written as a PDF, the model with
//! tenure among its controls and the check of that model, which warns that tenure is a
//! mediator.
//!
//! The section is read from `docs/spec/zeff_samples.zeff` itself, so the test follows the
//! specification's text; only the file it imports is supplied, as the shipped dataset of the
//! same name, whose sites stand for the sample's firms.

use zeff_testkit::{FileSystem, project, run_script};

/// The lines of the section, from its heading to the next.
fn section() -> String {
    let script = include_str!("../../../docs/spec/zeff_samples.zeff");
    let from = script
        .find("// # 13. Causal graph: check the model against its assumptions")
        .expect("the section's heading");
    let to = script.find("// # 14.").expect("the next heading");
    script[from..to].to_string()
}

#[test]
fn the_section_runs_whole_to_its_figure_and_its_warning() {
    let text = section();
    let drawing = "g.plot().export(\"output/fig_dag.pdf\", size: one_column)\n";
    assert!(text.contains(drawing), "{text}");
    assert!(text.contains("import(\"data/staff.csv\")"), "{text}");
    let text = text.replace(
        "import(\"data/staff.csv\")",
        "dataset(staff).add(firm = site)",
    );
    let (fs, project) = project();
    let outcome = run_script(&fs, &project, "scripts/main.zeff", &text);
    // The graph is written as a figure of one column.
    let figure = FileSystem::read(&fs, std::path::Path::new("/t/output/fig_dag.pdf"))
        .expect("the section's figure is written");
    assert!(figure.starts_with(b"%PDF-"));
    let (stdout, stderr) = (outcome.output.stdout, outcome.output.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    assert!(!stderr.contains("error["), "{stderr}");
    // No observed set gives the effect of training: the list is empty and the note says why.
    assert!(stdout.starts_with("[]\n"), "{stdout}");
    assert!(
        stderr.contains("note: no adjustment set among observed variables: ability is latent and confounds training -> productivity"),
        "{stderr}"
    );
    // The graph implies three independencies among its five observed variables.
    assert!(
        stdout.contains("implied_independencies · 3 among 5 observed variables"),
        "{stdout}"
    );
    // The check of the model, as the section's comment says: tenure is a mediator.
    assert!(
        stderr.contains("warning[W0510]: tenure lies on the path training → tenure → productivity"),
        "{stderr}"
    );
    assert!(
        stderr.contains("  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect"),
        "{stderr}"
    );
    assert!(
        stderr.contains("warning[W0511]: no sufficient adjustment set exists: ability is latent and confounds training and productivity"),
        "{stderr}"
    );
    assert!(
        stdout.contains("check · mt · the effect of training on productivity")
            && stdout.contains("tenure      a mediator: removes part of the effect"),
        "{stdout}"
    );
}
