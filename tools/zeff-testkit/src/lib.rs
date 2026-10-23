//! Test support for the function tests (testing handbook, sections 1.1, 1.2 and 4): the reference
//! values of `tests/reference/functions.json`, the fixed datasets of `tests/datasets/` (the edge
//! cases among them), a project that holds every dataset, and the comparison of a printed result
//! with its reference. This crate is a development dependency only; nothing of it is linked into
//! the engine.

use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use zeff_core::display::format_float;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

/// The repository root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root")
}

/// What a reference says a value is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expected {
    /// No value: `null` in the reference (nothing to compute from).
    Nothing,
    Na,
    Inf,
    NegInf,
    Number(f64),
}

impl Expected {
    fn parse(json: &Json) -> Expected {
        match json {
            Json::Null => Expected::Nothing,
            Json::String(s) if s == "NA" => Expected::Na,
            Json::String(s) if s == "Inf" => Expected::Inf,
            Json::String(s) if s == "-Inf" => Expected::NegInf,
            Json::String(s) => Expected::Number(
                s.parse()
                    .unwrap_or_else(|_| panic!("`{s}` is not a number in the reference")),
            ),
            other => panic!("unexpected reference value {other}"),
        }
    }

    /// The text Zeff prints for this value (section 3.8); `Nothing` prints as `NA`, since a
    /// statistic of no values is `NA`.
    pub fn printed(&self) -> String {
        match self {
            Expected::Nothing | Expected::Na => "NA".to_string(),
            Expected::Inf => "Inf".to_string(),
            Expected::NegInf => "-Inf".to_string(),
            Expected::Number(x) => format_float(*x),
        }
    }
}

/// The reference statistics of one numeric column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStats {
    pub n: usize,
    pub missing: usize,
    pub mean: Expected,
    pub sd: Expected,
    pub min: Expected,
    pub max: Expected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    /// The path from the repository root.
    pub file: String,
    pub rows: usize,
    /// The import options the dataset needs before it can be read at all, as a script writes
    /// them after the path: `, types: {hours: Str}`, or nothing.
    pub import_options: String,
    /// The numeric columns, by name, in the file's order.
    pub columns: Vec<(String, ColumnStats)>,
}

impl Dataset {
    /// The file's name as a script reads it: `data/wages.csv`.
    pub fn script_path(&self) -> String {
        format!(
            "data/{}",
            Path::new(&self.file).file_name().unwrap().to_string_lossy()
        )
    }

    /// The `import(...)` call that reads the dataset, with its own options and `extra` (more
    /// options, written after a comma, or nothing).
    pub fn import_call(&self, extra: &str) -> String {
        format!(
            "import(\"{}\"{}{extra})",
            self.script_path(),
            self.import_options
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub software: String,
    pub python: String,
    pub datasets: BTreeMap<String, Dataset>,
}

/// The reference, read from `tests/reference/functions.json`.
pub fn reference() -> Reference {
    let path = root().join("tests/reference/functions.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let json: Json = serde_json::from_str(&text).expect("functions.json is JSON");
    let mut datasets = BTreeMap::new();
    for (name, dataset) in json["datasets"].as_object().expect("datasets") {
        let columns = dataset["columns"]
            .as_object()
            .expect("columns")
            .iter()
            .map(|(column, stats)| {
                (
                    column.clone(),
                    ColumnStats {
                        n: stats["n"].as_u64().unwrap() as usize,
                        missing: stats["missing"].as_u64().unwrap() as usize,
                        mean: Expected::parse(&stats["mean"]),
                        sd: Expected::parse(&stats["sd"]),
                        min: Expected::parse(&stats["min"]),
                        max: Expected::parse(&stats["max"]),
                    },
                )
            })
            .collect();
        datasets.insert(
            name.clone(),
            Dataset {
                file: dataset["file"].as_str().unwrap().to_string(),
                rows: dataset["rows"].as_u64().unwrap() as usize,
                import_options: dataset["import_options"].as_str().unwrap_or("").to_string(),
                columns,
            },
        );
    }
    Reference {
        software: json["reference"]["software"].as_str().unwrap().to_string(),
        python: json["reference"]["python"].as_str().unwrap().to_string(),
        datasets,
    }
}

/// A project in memory whose `data/` holds every dataset of `tests/datasets/`.
pub fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/t/zeff.toml",
        format!(
            "[project]\nname = \"t\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let folder = root().join("tests/datasets");
    for entry in std::fs::read_dir(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()))
    {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "csv") {
            let bytes = std::fs::read(&path).unwrap();
            fs.add_file(
                format!("/t/data/{}", path.file_name().unwrap().to_string_lossy()),
                bytes,
            );
        }
    }
    let project = Project::open(&fs, Path::new("/t")).expect("the test project opens");
    (fs, project)
}

/// Run a script in the project of every dataset.
pub fn run(script: &str) -> Outcome {
    run_with(&[], script)
}

/// Run a script in the project of every dataset with more files added, each given by its path
/// from the project root and its bytes.
pub fn run_with(files: &[(&str, &[u8])], script: &str) -> Outcome {
    let (fs, project) = project();
    for (path, bytes) in files {
        fs.add_file(format!("/t/{path}"), bytes);
    }
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// A column name as a script writes it: plain, or in backticks. A name holding a backtick cannot
/// be written at all (`names: snake` renames it); `None` for those.
pub fn in_code(name: &str) -> Option<String> {
    let plain = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "_";
    if plain {
        Some(name.to_string())
    } else if name.contains('`') || name.contains('\n') {
        None
    } else {
        Some(format!("`{name}`"))
    }
}

/// The accuracy classes of section 13 as the function tests compare printed results: a printed
/// value has six significant digits, so a closed-form result must print as its reference prints,
/// and an exact result must be identical text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accuracy {
    Exact,
    ClosedForm,
}

/// Compare what a statement printed (one line) with the reference.
pub fn assert_prints(what: &str, printed: &str, expected: &Expected, _class: Accuracy) {
    let printed = printed.trim();
    let wanted = expected.printed();
    assert_eq!(
        printed, wanted,
        "{what}: printed `{printed}`, the reference gives `{wanted}` ({expected:?})"
    );
}

/// A formula value from its text, `lwage ~ educ + exper^2`, as the checker lowers it against
/// this engine's registry and the engine evaluates it; a text that does not check is a panic
/// naming the codes.
pub fn formula(text: &str) -> zeff_core::value::Formula {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let script = format!("let f = {text}\n");
    let checked = zeff_ir::check_source("formula.zeff", &script, registry);
    let codes: Vec<String> = checked
        .diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    assert!(codes.is_empty(), "`{text}` does not check: {codes:?}");
    let zeff_ir::Statement::Let { value, .. } = &checked.program.statements[0] else {
        panic!("not a let");
    };
    let zeff_ir::ExprKind::Formula(formula) = &value.kind else {
        panic!("`{text}` is not a formula literal");
    };
    zeff_engine::formulas::formula_value(formula).unwrap_or_else(|what| panic!("{what}"))
}
