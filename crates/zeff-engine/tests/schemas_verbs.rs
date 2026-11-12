//! The schema pass, part 2 (Language Reference, section 11, "Checking before running"; section
//! 6, every verb's columns; release 0.6.0, step 21): the columns the checker gives a frame after
//! each verb of this and the earlier releases, against the frame the interpreter produces, verb
//! by verb on the shipped `wages` and `firms`; the same for generated chains of verbs; and a
//! project function that returns a frame.

use proptest::prelude::*;
use std::path::Path;
use zeff_diagnostics::Span;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{check_in_project, run_script};
use zeff_ir::{Statement, Ty};

fn project(fs: &MemoryFileSystem) -> Project {
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    Project::open(fs, Path::new("/p")).expect("the project opens")
}

/// The checker's columns of the binding `r` of the script, as (name, type).
fn checked_columns(script: &str) -> Option<Vec<(String, Ty)>> {
    let fs = MemoryFileSystem::new();
    let project = project(&fs);
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let checked = check_in_project(&fs, &project, "scripts/main.zeff", script, registry).checked;
    let errors: Vec<String> = checked
        .diagnostics
        .iter()
        .filter(|d| d.severity == zeff_diagnostics::Severity::Error)
        .map(|d| d.message.clone())
        .collect();
    assert!(errors.is_empty(), "{script}\n{errors:?}");
    let program = &checked.program;
    let span: Span = program
        .statements
        .iter()
        .find_map(|s| match s {
            Statement::Let { binding, value, .. } if program.bindings[*binding].name == "r" => {
                Some(value.span)
            }
            _ => None,
        })
        .expect("a binding r");
    program
        .schemas
        .iter()
        .find(|(s, _)| *s == span)
        .map(|(_, schema)| schema.columns.clone())
}

/// The run's columns of `r`, from `r.describe()`: each row's variable and type, a category or a
/// labelled number as the checker does not know it.
fn run_columns(script: &str) -> Vec<(String, Ty)> {
    let fs = MemoryFileSystem::new();
    let project = project(&fs);
    let outcome = run_script(
        &fs,
        &project,
        "scripts/main.zeff",
        &format!("{script}r.describe()\n"),
    );
    assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
    let text = outcome.output.stdout;
    let mut out = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.trim_start().starts_with("variable ") {
            in_table = true;
            continue;
        }
        if !in_table || line.trim().is_empty() {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .split("  ")
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        let ty = match cells[1].split(' ').next().unwrap_or("") {
            "Int" => Ty::Int,
            "Float" => Ty::Float,
            "Str" => Ty::Str,
            "Bool" => Ty::Bool,
            "Date" => Ty::Date,
            "DateTime" => Ty::DateTime,
            _ => Ty::Unknown,
        };
        out.push((cells[0].to_string(), ty));
    }
    out
}

/// The checker's columns against the run's: the names in order, and the types where the
/// checker commits to one.
fn agree(script: &str) {
    let checked = checked_columns(script).unwrap_or_else(|| panic!("no schema for r in\n{script}"));
    let run = run_columns(script);
    let names = |c: &[(String, Ty)]| c.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
    assert_eq!(names(&checked), names(&run), "the names of\n{script}");
    for ((name, a), (_, b)) in checked.iter().zip(&run) {
        if *a != Ty::Unknown && *b != Ty::Unknown {
            assert_eq!(a, b, "the type of `{name}` in\n{script}");
        }
    }
}

#[test]
fn the_schema_after_every_verb() {
    let w = "let w = dataset(wages)\nlet f = dataset(firms)\n";
    for chain in [
        "w.head(3)",
        "w.tail(3)",
        "w.where(year > 2015)",
        "w.sort([year, worker])",
        "w.sample(n: 20, seed: 1)",
        "w.drop_missing([wage])",
        "w.distinct([worker])",
        "w.panel(unit: worker, time: year)",
        "w.label(union, [0 => \"no\", 1 => \"yes\"])",
        "w.recode(union, [0 => 10, 1 => 20])",
        "w.select([worker, year, wage])",
        "w.select(wage..hours)",
        "w.drop([hours, union])",
        "w.drop(age..female)",
        "w.rename(pay = wage, id = worker)",
        "w.add(lw = log(wage), old = age > 40, k = 1)",
        "w.add(wage = wage * 2)",
        "w.update(wage = wage * 2, where: union == 1)",
        "w.merge(f, on: [firm, industry])",
        "w.merge(f, on: firm, suffixes: [\"_w\", \"_f\"])",
        "w.merge(f, on: firm, join: inner, suffixes: [\"\", \"_f\"])",
        "w.select([worker, year, wage]).append(w.select([worker, year, wage]))",
        "w.select([worker, year, wage]).append(f.select([firm, size]))",
        "w.head(4).to_long(\"wage\", id: worker, names_to: \"what\", values_to: \"value\")",
        "w.head(4).select([worker, year, wage, hours]).to_long(\"h\", id: [worker, year], names_to: \"k\", values_to: \"v\")",
        "w.aggregate(m = mean(wage), s = sd(wage) / sqrt(n()), rows = n(), by: industry)",
        "w.aggregate(m = mean(wage))",
        "w.aggregate(m = mean(wage), by: [industry, year])",
        // The groups, and the variables kept, written with `vars(...)`: the checker knows
        // them as it knows a list (they were lost until step 23c1 of release 0.7.0, so that
        // a later use of a group of the result was refused before the run).
        "w.aggregate(m = mean(wage), by: vars(industry, year))",
        "w.aggregate(m = mean(wage), by: vars(industry))",
        "w.aggregate(m = mean(wage), by: vars(industry, year)).select(year)",
        "w.head(4).select([worker, year, wage, hours]).to_long(\"h\", id: vars(worker, year), names_to: \"k\", values_to: \"v\")",
        "w.where(wage > 10).add(lw = log(wage)).select([worker, lw]).rename(l = lw).sort([worker]).head(5)",
    ] {
        agree(&format!("{w}let r = {chain}\n"));
    }
}

#[test]
fn a_project_function_that_returns_a_frame() {
    let script = "let w = dataset(wages)\n/// Cleans.\nfn clean(data: Frame) -> Frame { data.where(wage > 0).add(lw = log(wage)) }\n/// Picks.\nfn pick(data: Frame, early: Bool) -> Frame {\n    if early { return data.select([worker, year]) }\n    data.select([worker, year, wage])\n}\nlet r = w.clean().drop([hours])\n";
    agree(script);
    let script = "let w = dataset(wages)\n/// Cleans.\nfn clean(data: Frame) -> Frame { data.where(wage > 0).add(lw = log(wage)) }\nlet r = clean(w).select([lw, worker])\n";
    agree(script);
    // Two returns with different columns: no schema, no false report.
    let script = "let w = dataset(wages)\n/// Picks.\nfn pick(data: Frame, early: Bool) -> Frame {\n    if early { return data.select([worker, year]) }\n    data.select([worker, year, wage])\n}\nlet r = w.pick(true)\nr.mean(nothing_known)\n";
    assert!(checked_columns(script).is_none());
}

/// A verb of the chains the property draws: its text, the columns it needs, and its effect on
/// a list of column names, so that every chain drawn is valid.
#[derive(Debug, Clone)]
enum Step {
    Where,
    Head,
    Sort,
    Select,
    DropHours,
    RenameWage,
    AddLog,
    DistinctWorker,
    Aggregate,
}

impl Step {
    fn text(&self) -> &'static str {
        match self {
            Step::Where => ".where(year > 2012)",
            Step::Head => ".head(7)",
            Step::Sort => ".sort([year])",
            Step::Select => ".select([worker, year, wage, industry])",
            Step::DropHours => ".drop([hours])",
            Step::RenameWage => ".rename(pay = wage)",
            Step::AddLog => ".add(lw = log(wage))",
            Step::DistinctWorker => ".distinct([worker])",
            Step::Aggregate => ".aggregate(m = mean(wage), by: [industry, year])",
        }
    }

    /// Whether the step can follow, given the columns so far, and the columns after it.
    fn apply(&self, columns: &[String]) -> Option<Vec<String>> {
        let has = |n: &str| columns.iter().any(|c| c == n);
        let mut out = columns.to_vec();
        match self {
            Step::Where | Step::Sort => has("year").then_some(out),
            Step::Head => Some(out),
            Step::Select => {
                let wanted = ["worker", "year", "wage", "industry"];
                wanted
                    .iter()
                    .all(|n| has(n))
                    .then(|| wanted.iter().map(|s| s.to_string()).collect())
            }
            Step::DropHours => has("hours").then(|| {
                out.retain(|c| c != "hours");
                out
            }),
            Step::RenameWage => has("wage").then(|| {
                for c in &mut out {
                    if c == "wage" {
                        *c = "pay".to_string();
                    }
                }
                out
            }),
            Step::AddLog => (has("wage") && !has("lw")).then(|| {
                out.push("lw".to_string());
                out
            }),
            Step::DistinctWorker => has("worker").then_some(out),
            Step::Aggregate => (has("wage") && has("industry") && has("year"))
                .then(|| vec!["industry".to_string(), "year".to_string(), "m".to_string()]),
        }
    }
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Where),
        Just(Step::Head),
        Just(Step::Sort),
        Just(Step::Select),
        Just(Step::DropHours),
        Just(Step::RenameWage),
        Just(Step::AddLog),
        Just(Step::DistinctWorker),
        Just(Step::Aggregate),
    ]
}

fn property_cases(usual: u32) -> u32 {
    if std::env::var_os("CI").is_some() {
        usual
    } else {
        usual / 4
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(24)))]

    /// For generated chains of verbs over the shipped `wages`, the checker's schema equals the
    /// run's frame's columns and types, name for name; a step that would not be valid for the
    /// columns so far is left out.
    #[test]
    fn generated_chains_agree(steps in prop::collection::vec(step(), 1..6)) {
        let wages = ["worker", "year", "wage", "educ", "exper", "age", "hours", "union", "female", "industry", "firm"];
        let mut columns: Vec<String> = wages.iter().map(|s| s.to_string()).collect();
        let mut chain = String::from("w");
        for step in &steps {
            if let Some(next) = step.apply(&columns) {
                columns = next;
                chain.push_str(step.text());
            }
        }
        let script = format!("let w = dataset(wages)\nlet r = {chain}\n");
        let checked = checked_columns(&script).expect("a schema");
        let names: Vec<String> = checked.iter().map(|(n, _)| n.clone()).collect();
        prop_assert_eq!(&names, &columns);
        let run = run_columns(&script);
        prop_assert_eq!(run.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(), columns);
        for ((name, a), (_, b)) in checked.iter().zip(&run) {
            if *a != Ty::Unknown && *b != Ty::Unknown {
                prop_assert_eq!(a, b, "the type of {}", name);
            }
        }
    }
}
