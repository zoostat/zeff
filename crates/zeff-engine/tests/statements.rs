//! Each statement of a run with its result and what it depends on (open decision #175, C5;
//! release 0.9.0, step 26a): its type and dimensions, the statements it reads, the functions
//! it calls, and whether it sets the seed, draws, reads a file or writes one.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::dependencies::{READERS, STOCHASTIC, WRITERS};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{check_in_project, run_script};
use zeff_engine::statements::{StatementFacts, json, statement_facts};

const HELPERS: &str = "/// A sample of the rows.\n/// data: The frame.\n/// returns: Ten rows.\n/// > dataset(wages).ten().n_rows()\n/// 10\nfn ten(data: Frame) -> Frame {\n    data.sample(n: 10)\n}\n\n/// The rows of a sample, through the function above.\n/// data: The frame.\n/// returns: Ten.\n/// > dataset(wages).counted()\n/// 10\nfn counted(data: Frame) -> Int {\n    ten(data).n_rows()\n}\n\n/// The rows, with no draw.\n/// data: The frame.\n/// returns: The rows.\n/// > dataset(wages).plain_rows()\n/// 18412\nfn plain_rows(data: Frame) -> Int {\n    data.n_rows()\n}\n";

/// The statements of `text` run in a project with a module and a data file.
fn facts_of(text: &str) -> (Vec<StatementFacts>, i32) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/helpers.zeff", HELPERS);
    fs.add_file("/p/data/pay.csv", "region,wage\nnorth,23.5\nsouth,31.25\n");
    fs.add_dir("/p/output");
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", text);
    let checked = check_in_project(&fs, &project, "scripts/main.zeff", text, registry);
    (
        statement_facts(
            &checked.checked.program,
            "scripts/main.zeff",
            text,
            registry,
            &outcome.output,
        ),
        outcome.exit_code,
    )
}

#[test]
fn each_statement_says_what_its_result_is() {
    let text = "let w = dataset(wages)\nlet fit = w.reg(log(wage) ~ educ +\n    exper)\nlet m = w.mean(wage)\nlet described = w.describe()\nlet xs = [1, 2, 3]\nlet many = seq(1, 5)\nlet t = w.t_test(wage, group: female)\nlet rec = {a: 1, b: \"two\"}\nlet p = w.scatter(educ, wage)\nprint(m)\nw.head(3)\nfor i in 1..2 { print(i) }\n";
    let (facts, exit_code) = facts_of(text);
    assert_eq!(
        exit_code,
        0,
        "{:?}",
        facts.iter().map(|f| &f.stderr).collect::<Vec<_>>()
    );
    let said: Vec<String> = facts
        .iter()
        .map(|fact| {
            let result = fact.result.as_ref();
            format!(
                "{} lines {}-{} {} {} rows {:?} columns {:?} length {:?} n {:?}",
                fact.index,
                fact.line,
                fact.end_line,
                fact.name.as_deref().unwrap_or("-"),
                result.map_or("-", |r| r.ty.as_str()),
                result.and_then(|r| r.rows),
                result.and_then(|r| r.columns),
                result.and_then(|r| r.length),
                result.and_then(|r| r.n),
            )
        })
        .collect();
    insta::assert_snapshot!(said.join("\n"));
    assert!(facts.iter().all(|fact| fact.ran));
    // The name of a `let` and the name in its result are one.
    for fact in &facts {
        if let Some(result) = &fact.result {
            assert_eq!(result.name, fact.name, "statement {}", fact.index);
        }
    }
    // What a statement printed is its own.
    assert_eq!(facts[8].stdout, "", "a `let` prints nothing");
    assert_eq!(facts[9].stdout, "27.3695\n");
    // A statement that is an expression prints its value, and says what the value is.
    assert!(
        facts[10]
            .stdout
            .starts_with("frame · 3 rows × 11 variables"),
        "{}",
        facts[10].stdout
    );
    assert_eq!(facts[11].stdout, "1\n2\n");
}

#[test]
fn each_statement_says_what_it_depends_on() {
    let text = "use functions::helpers\nseed(11)\nlet w = dataset(wages)\nlet pay = import(\"data/pay.csv\")\nlet some = w.sample(n: 50)\nlet own = w.sample(n: 50, seed: 4)\nlet opts = {seed: 9}\nlet spread = w.sample(n: 5, ..opts)\nlet through = w.ten()\nlet deeper = w.counted()\nlet m = some.mean(wage) + through.mean(wage)\npay.describe().export(\"output/pay.tex\")\npay.save(\"output/pay2.csv\")\nlet plain = pay.plain_rows() + deeper\n";
    let (facts, exit_code) = facts_of(text);
    assert_eq!(
        exit_code,
        0,
        "{:?}",
        facts.iter().map(|f| &f.stderr).collect::<Vec<_>>()
    );
    let said: Vec<String> = facts
        .iter()
        .map(|fact| {
            let mut marks = Vec::new();
            for (mark, set) in [
                ("seeds", fact.seeds),
                ("draws", fact.draws),
                ("drew", fact.drew),
                ("reads a file", fact.reads_file),
                ("writes a file", fact.writes_file),
            ] {
                if set {
                    marks.push(mark);
                }
            }
            format!(
                "{} {} reads {:?} calls {:?} {}",
                fact.index,
                fact.name.as_deref().unwrap_or("-"),
                fact.reads,
                fact.calls,
                marks.join(", ")
            )
        })
        .collect();
    insta::assert_snapshot!(said.join("\n"));
    let by_name = |name: &str| {
        facts
            .iter()
            .find(|fact| fact.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("{name}"))
    };
    // A draw from the run's stream is marked, and it drew; a call with a seed of its own,
    // written or filled by a spread, is neither.
    assert!(by_name("some").draws && by_name("some").drew);
    assert!(!by_name("own").draws && !by_name("own").drew);
    assert!(!by_name("spread").draws && !by_name("spread").drew);
    // Through a function of the project, and through a function that function calls; a
    // function that draws nothing leaves its caller unmarked.
    assert!(by_name("through").draws && by_name("through").drew);
    assert!(by_name("deeper").draws && by_name("deeper").drew);
    assert_eq!(by_name("deeper").calls, ["helpers::counted"]);
    assert!(!by_name("plain").draws && !by_name("plain").drew);
    // A file is read and written by the script's own statements alone: a function may do
    // neither (Z0110).
    assert!(by_name("pay").reads_file && !by_name("pay").writes_file);
    let writers: Vec<usize> = facts
        .iter()
        .filter(|fact| fact.writes_file)
        .map(|fact| fact.index)
        .collect();
    assert_eq!(writers, [11, 12]);
    // A statement that only reads the results of those does none of it itself.
    let m = by_name("m");
    assert!(!m.draws && !m.reads_file && !m.writes_file && !m.seeds);
    // `use` is no statement: the places are those of the statements.
    assert_eq!(m.reads, [4, 8]);
}

#[test]
fn a_run_that_stops_leaves_the_rest_not_run_and_keeps_what_they_depend_on() {
    let text = "let w = dataset(wages)\nlet m = w.mean(wage)\nassert(m < 0, \"wages are above nought\")\nlet late = w.sample(n: 5)\nprint(late.n_rows())\n";
    let (facts, exit_code) = facts_of(text);
    assert_eq!(exit_code, 1);
    let ran: Vec<bool> = facts.iter().map(|fact| fact.ran).collect();
    assert_eq!(ran, [true, true, true, false, false]);
    assert!(
        facts[2].stderr.contains("wages are above nought"),
        "{}",
        facts[2].stderr
    );
    assert!(facts[2].result.is_none());
    // What was not run still says what it would read and that it would draw.
    assert_eq!(
        (facts[3].name.as_deref(), facts[3].draws, facts[3].drew),
        (Some("late"), true, false)
    );
    assert_eq!(facts[3].reads, [1]);
    assert_eq!(facts[4].reads, [4]);
    assert!(facts[3].result.is_none() && facts[3].stdout.is_empty());
    // A script that does not pass its check runs nothing; its statements are there all the same.
    let (unchecked, exit_code) =
        facts_of("let w = dataset(wages)\nlet bad = w.mean(no_such_variable)\n");
    assert_eq!(exit_code, 1);
    assert_eq!(unchecked.len(), 2);
    assert!(
        unchecked
            .iter()
            .all(|fact| !fact.ran && fact.result.is_none())
    );
}

#[test]
fn the_record_of_a_statement_is_one_object() {
    let (facts, _) = facts_of("let w = dataset(wages).head(4)\nprint(\"a \\\"quoted\\\" line\")\n");
    assert_eq!(
        facts[0].json(),
        r#"{"index": 1, "line": 1, "end_line": 1, "ran": true, "name": "w", "type": "Frame", "rows": 4, "columns": 11, "length": null, "n": null, "stdout": "", "stderr": "", "reads": [], "calls": [], "seeds": false, "draws": false, "reads_file": false, "writes_file": false, "drew": false, "error": null}"#
    );
    assert_eq!(
        facts[1].json(),
        r#"{"index": 2, "line": 2, "end_line": 2, "ran": true, "name": null, "type": null, "rows": null, "columns": null, "length": null, "n": null, "stdout": "a \"quoted\" line\n", "stderr": "", "reads": [], "calls": [], "seeds": false, "draws": false, "reads_file": false, "writes_file": false, "drew": false, "error": null}"#
    );
    assert_eq!(
        json(&facts),
        format!("[{}, {}]", facts[0].json(), facts[1].json())
    );
    assert_eq!(json(&[]), "[]");
}

/// The lists the graph goes by are those of the catalogue: every implemented function that
/// draws is known as one, and every implemented function that takes a path reads a file or
/// writes one. A function added without its place in a list fails here.
#[test]
fn the_functions_that_draw_and_that_touch_files_are_those_of_the_catalogue() {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let mut stochastic: Vec<(String, String)> = Vec::new();
    let mut with_a_path: Vec<(String, String)> = Vec::new();
    for known in registry.functions() {
        let name = (known.spec.package.to_string(), known.spec.name.to_string());
        if known.row.accuracy == "stochastic" && !stochastic.contains(&name) {
            stochastic.push(name.clone());
        }
        if known.spec.params.iter().any(|param| param.name == "path")
            && !with_a_path.contains(&name)
        {
            with_a_path.push(name);
        }
    }
    let listed = |list: &[(&str, &str)]| -> Vec<(String, String)> {
        let mut names: Vec<(String, String)> = list
            .iter()
            .map(|(package, name)| ((*package).to_string(), (*name).to_string()))
            .collect();
        names.sort();
        names
    };
    stochastic.sort();
    with_a_path.sort();
    assert_eq!(stochastic, listed(&STOCHASTIC));
    let mut files = listed(&READERS);
    files.extend(listed(&WRITERS));
    files.sort();
    assert_eq!(with_a_path, files);
    // Every function that draws takes a `seed:` of its own, which is how a call is known to
    // have left the run's stream.
    for known in registry.functions() {
        if known.row.accuracy == "stochastic" {
            assert!(
                known.spec.params.iter().any(|param| param.name == "seed"),
                "{}",
                known.spec.name
            );
        }
    }
}

/// The specification's sample script, which this release does not run whole: what each of its
/// statements depends on is read from it all the same.
#[test]
fn the_statements_of_the_sample_script_say_what_they_depend_on() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let (facts, _) = facts_of(text);
    assert!(facts.len() > 60, "{} statements", facts.len());
    let said: Vec<String> = facts
        .iter()
        .map(|fact| {
            let mut marks = Vec::new();
            for (mark, set) in [
                ("seeds", fact.seeds),
                ("draws", fact.draws),
                ("reads a file", fact.reads_file),
                ("writes a file", fact.writes_file),
            ] {
                if set {
                    marks.push(mark);
                }
            }
            format!(
                "{:>3} lines {:>3}-{:<3} {:<14} reads {:?} {}",
                fact.index,
                fact.line,
                fact.end_line,
                fact.name.as_deref().unwrap_or("-"),
                fact.reads,
                marks.join(", ")
            )
            .trim_end()
            .to_string()
        })
        .collect();
    insta::assert_snapshot!(said.join("\n"));
}
