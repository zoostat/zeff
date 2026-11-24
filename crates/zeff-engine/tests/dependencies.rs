//! The dependency graph of a script (release 0.8.0, step 20a): what each statement reads and
//! calls, on a script written for the test and on every sample project.

use std::path::Path;
use zeff_diagnostics::Source;
use zeff_engine::dependencies::{Graph, graph};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::check_in_project;
use zeff_ir::Program;

/// The graph of a script of the project at `/s`, with its program and its text.
fn graph_of(fs: &MemoryFileSystem, script: &str) -> (Graph, Program, Source) {
    let project = Project::open(fs, Path::new("/s")).unwrap();
    let text = String::from_utf8(fs_read(fs, script)).unwrap();
    let registry = zeff_engine::registry::registry().unwrap();
    let checked = check_in_project(fs, &project, script, &text, registry);
    let program = checked.checked.program;
    (graph(&program, script), program, Source::new(script, &text))
}

fn fs_read(fs: &MemoryFileSystem, script: &str) -> Vec<u8> {
    use zeff_engine::fs::FileSystem;
    fs.read(&Path::new("/s").join(script)).unwrap()
}

/// The graph as lines: each statement by the line it begins on, with the lines of the
/// statements it reads, the functions it calls and whether it sets the seed; then, for the
/// last statement, the lines it needs in all.
fn shown(graph: &Graph, program: &Program, source: &Source) -> String {
    let line = |index: usize| source.line_of(graph.statements[index].span.start);
    let mut out = String::new();
    for (index, node) in graph.statements.iter().enumerate() {
        let reads: Vec<String> = node.reads.iter().map(|i| line(*i).to_string()).collect();
        let calls: Vec<&str> = node
            .calls
            .iter()
            .map(|id| program.functions[*id].name.as_str())
            .collect();
        out.push_str(&format!(
            "line {:>3}: reads [{}]{}{}\n",
            line(index),
            reads.join(", "),
            if calls.is_empty() {
                String::new()
            } else {
                format!(" calls [{}]", calls.join(", "))
            },
            if node.seeds { " sets the seed" } else { "" }
        ));
    }
    if let Some(last) = graph.statements.len().checked_sub(1) {
        let needed = graph.needed_by(last);
        let lines: Vec<String> = needed
            .statements
            .iter()
            .map(|i| line(*i).to_string())
            .collect();
        let functions: Vec<&str> = needed
            .functions
            .iter()
            .map(|id| program.functions[*id].name.as_str())
            .collect();
        out.push_str(&format!(
            "the last statement needs {} of {} statements: lines [{}]{}\n",
            needed.statements.len(),
            graph.statements.len(),
            lines.join(", "),
            if functions.is_empty() {
                String::new()
            } else {
                format!("; functions [{}]", functions.join(", "))
            }
        ));
    }
    out
}

fn project() -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/s/zeff.toml",
        format!(
            "[project]\nname = \"s\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs
}

/// What a statement reads: a binding, a binding it shadows, one used inside a loop; the
/// functions it calls, a function literal it writes among them, with the functions of the
/// script and of a module those call; a seed set before it, with the binding the seed is read
/// from; and nothing else (the statements of lines 4 and 12 are not needed by the last).
#[test]
fn a_statement_needs_what_it_reads_and_what_it_calls() {
    let fs = project();
    fs.add_file(
        "/s/functions/clean.zeff",
        "/// Twice a number.\n///\n/// x: The number.\n/// returns: Twice it.\n///\n/// > twice(2)\n/// 4\nfn twice(x: Float) -> Float {\n    _two() * x\n}\n\n/// Two.\n///\n/// returns: Two.\nfn _two() -> Float {\n    2.0\n}\n",
    );
    let script = "use functions::clean
let n = 7
let wages = dataset(wages)
let unused = wages.mean(wage)
seed(n)
let wages = wages.add(lwage = log(wage))
let sizes = [10, 20]
let means = [for k in sizes { wages.sample(n: k).mean(lwage) }]
fn half(x: Float) -> Float {
    twice(x) / 4
}
print(unused)
let scaled = map(means, fn(m) half(m))
scaled
";
    fs.add_file("/s/scripts/main.zeff", script);
    let (graph, program, source) = graph_of(&fs, "scripts/main.zeff");
    let text = shown(&graph, &program, &source);
    assert_eq!(
        text,
        "line   2: reads []
line   3: reads []
line   4: reads [3]
line   5: reads [2] sets the seed
line   6: reads [3]
line   7: reads []
line   8: reads [6, 7]
line  12: reads [4]
line  13: reads [8] calls [fn(m)]
line  14: reads [13]
the last statement needs 8 of 10 statements: lines [2, 3, 5, 6, 7, 8, 13, 14]; functions [twice, _two, half, fn(m)]
",
        "{text}"
    );
    // Every statement reads earlier ones only, and what one needs holds what those need.
    for (index, node) in graph.statements.iter().enumerate() {
        assert!(node.reads.iter().all(|read| *read < index));
        let needed = graph.needed_by(index);
        assert!(needed.statements.contains(&index));
        for kept in &needed.statements {
            for read in &graph.statements[*kept].reads {
                assert!(
                    needed.statements.contains(read),
                    "{index}: {kept} reads {read}"
                );
            }
        }
    }
    // The statement at a place; none between statements' lines of a function.
    assert_eq!(
        graph.statement_at(script.find("seed(n)").unwrap() + 2),
        Some(3)
    );
    assert_eq!(
        graph.statement_at(script.find("twice(x) / 4").unwrap()),
        None
    );
    assert!(graph.needed_by(99).statements.is_empty());
}

/// The graph of every sample project's script, a stored text: the last statement of each needs
/// far fewer statements than the script has.
#[test]
fn the_graphs_of_the_sample_scripts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut all = String::new();
    for name in [
        "first_engine",
        "data_cleaning",
        "regression",
        "simulation",
        "user_functions",
        "panel",
        "plots",
        "tests",
        "instruments",
        "binary",
    ] {
        let folder = root.join(name);
        let fs = MemoryFileSystem::new();
        let copy = |relative: &str| {
            if let Ok(bytes) = std::fs::read(folder.join(relative)) {
                fs.add_file(format!("/s/{relative}"), bytes);
            }
        };
        copy("zeff.toml");
        copy("scripts/main.zeff");
        for sub in ["functions", "data"] {
            let mut files: Vec<String> = std::fs::read_dir(folder.join(sub))
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or_default();
            files.sort();
            for file in files {
                copy(&format!("{sub}/{file}"));
            }
        }
        let (graph, program, source) = graph_of(&fs, "scripts/main.zeff");
        assert!(!graph.statements.is_empty(), "{name}");
        let last = graph.statements.len() - 1;
        assert!(
            graph.needed_by(last).statements.len() < graph.statements.len(),
            "{name}: the last statement needs every statement"
        );
        all.push_str(&format!(
            "==== {name}\n{}",
            shown(&graph, &program, &source)
        ));
    }
    insta::assert_snapshot!("sample_graphs", all);
}
