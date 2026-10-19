//! Package datasets (tier 1): the datasets that ship inside the engine (Engine and Project
//! Specification, section 8.5), the `zeff:` data root that serves their files, and the functions
//! `dataset(name)` and `datasets()`. Each dataset is a CSV file written by a committed script in
//! `generate/` and an Arrow IPC file built from it by `cargo run -p xtask -- build-datasets` with
//! the labels, units, value labels, categories and panel declaration; `dataset(name)` reads the
//! Arrow file, `import("zeff:name.csv")` the CSV. Parquet, the kept format of section 8.7,
//! arrives with its reader and writer at 0.11.0. The checksums of every file are in
//! `data/CHECKSUMS`.

use std::rc::Rc;
use zeff_columns::{MetadataKey, Table, read_ipc};
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::{Align, Frame, Value};
use zeff_diagnostics::{Diagnostic, count, did_you_mean};
use zeff_registry::{DatasetSpec, DiagnosticDecl, FormatRow, FunctionSpec, ParamSpec};

/// One shipped dataset.
pub struct Shipped {
    pub name: &'static str,
    /// `synthetic` or `public`.
    pub kind: &'static str,
    pub licence: &'static str,
    pub source: &'static str,
    pub csv: &'static [u8],
    pub arrow: &'static [u8],
    pub help: &'static str,
}

/// The datasets of this release, in the order `datasets()` lists them.
pub const SHIPPED: [Shipped; 2] = [
    Shipped {
        name: "sleep",
        kind: "public",
        licence: "public domain (Student 1908)",
        source: "Student (1908), Biometrika 6, 1–25; Cushny and Peebles (1905)",
        csv: include_bytes!("../data/sleep.csv"),
        arrow: include_bytes!("../data/sleep.arrow"),
        help: include_str!("../help/sleep.md"),
    },
    Shipped {
        name: "wages",
        kind: "synthetic",
        licence: "CC0-1.0",
        source: "generate/wages.py, seed 20261019",
        csv: include_bytes!("../data/wages.csv"),
        arrow: include_bytes!("../data/wages.arrow"),
        help: include_str!("../help/wages.md"),
    },
];

/// The bytes a `zeff:` path names: `zeff:wages.csv` is the CSV, `zeff:wages.arrow` the Arrow
/// file; `None` for any other name.
pub fn file(name: &str) -> Option<&'static [u8]> {
    let (stem, extension) = name.rsplit_once('.')?;
    let shipped = SHIPPED.iter().find(|d| d.name == stem)?;
    match extension {
        "csv" => Some(shipped.csv),
        "arrow" => Some(shipped.arrow),
        _ => None,
    }
}

/// The names of the files the `zeff:` root serves.
pub fn files() -> Vec<String> {
    SHIPPED
        .iter()
        .flat_map(|d| [format!("{}.csv", d.name), format!("{}.arrow", d.name)])
        .collect()
}

/// The table of a dataset, with its metadata, from the Arrow file compiled in.
pub fn table(name: &str) -> Option<Table> {
    let shipped = SHIPPED.iter().find(|d| d.name == name)?;
    read_ipc(shipped.arrow).ok()
}

/// The dataset pages for the registry, their Format part generated from the Arrow schema.
pub fn dataset_specs() -> Vec<DatasetSpec> {
    SHIPPED
        .iter()
        .map(|d| {
            let table = read_ipc(d.arrow).expect("a shipped Arrow file reads");
            let format = table
                .columns()
                .map(|(name, column)| {
                    let metadata = column.metadata();
                    let mut ty = column.ty().name().to_string();
                    if let Some(levels) = column.levels() {
                        ty = format!("Category ({} levels)", levels.len());
                    }
                    FormatRow {
                        name: name.clone(),
                        ty,
                        label: metadata.get(MetadataKey::Label).unwrap_or("").to_string(),
                        unit: metadata.get(MetadataKey::Unit).unwrap_or("").to_string(),
                        values: metadata
                            .get(MetadataKey::ValueLabels)
                            .map(value_labels_text)
                            .unwrap_or_default(),
                        missing: column.n_missing(),
                    }
                })
                .collect();
            DatasetSpec {
                name: d.name,
                kind: d.kind,
                licence: d.licence,
                rows: table.rows(),
                columns: table.n_cols(),
                panel: table.metadata().panel.clone(),
                format,
                help: d.help,
            }
        })
        .collect()
}

/// `{"0": "male", "1": "female"}` as `0 male, 1 female`.
fn value_labels_text(json: &str) -> String {
    json.trim_matches(['{', '}'])
        .split(',')
        .map(|pair| {
            let (k, v) = pair.split_once(':').unwrap_or((pair, ""));
            format!(
                "{} {}",
                k.trim().trim_matches('"'),
                v.trim().trim_matches('"')
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `dataset(name)`: the dataset as a frame with its labels, categories and panel declaration.
pub fn dataset(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let name = match args.get(0) {
        Some(Value::Var(name)) => name.clone(),
        Some(Value::Str(name)) => name.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (dataset was called without a name)")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "the call that failed"),
            ));
        }
    };
    let Some(table) = table(&name) else {
        let names: Vec<&str> = SHIPPED.iter().map(|d| d.name).collect();
        let mut d = Diagnostic::error("Z0101", format!("no dataset is called `{name}`"))
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not a shipped dataset");
        d = match did_you_mean(&name, names.iter().copied()) {
            Some(near) => d.help(format!("did you mean `{near}`?")),
            None => d.help(format!(
                "the datasets of this release: {}; `datasets()` lists them",
                names.join(", ")
            )),
        };
        return Err(fail(d));
    };
    Ok(Value::Frame(Frame {
        table: Rc::new(table),
        name: Some(name),
    }))
}

/// `datasets()`: the table of the shipped datasets.
pub fn datasets(_ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    let rows = SHIPPED
        .iter()
        .map(|d| {
            let table = read_ipc(d.arrow).ok();
            let (rows, cols) = table.as_ref().map_or((0, 0), |t| (t.rows(), t.n_cols()));
            vec![
                d.name.to_string(),
                format!("{} × {}", count(rows as u64), count(cols as u64)),
                d.kind.to_string(),
                d.licence.to_string(),
                d.source.to_string(),
                size_text(d.csv.len() + d.arrow.len()),
            ]
        })
        .collect();
    Ok(Value::Table(zeff_core::value::Table {
        title: format!("datasets · {} shipped with this release", SHIPPED.len()),
        columns: vec![
            ("name".to_string(), Align::Left),
            ("rows × columns".to_string(), Align::Right),
            ("kind".to_string(), Align::Left),
            ("licence".to_string(), Align::Left),
            ("source".to_string(), Align::Left),
            ("size".to_string(), Align::Right),
        ],
        rows,
    }))
}

fn size_text(bytes: usize) -> String {
    if bytes < 1000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{} kB", (bytes + 500) / 1000)
    } else {
        format!("{:.1} MB", bytes as f64 / 1e6)
    }
}

/// The functions of `datasets` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "dataset",
                package: "datasets",
                method: false,
                params: vec![ParamSpec::required("name", "Var")],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0101",
                    "no dataset of that name ships with this release",
                )],
                help: include_str!("../help/dataset.md"),
                accuracy: None,
            },
            body: dataset,
        },
        Function {
            spec: FunctionSpec {
                name: "datasets",
                package: "datasets",
                method: false,
                params: vec![],
                rest: None,
                returns: "Table",
                diagnostics: vec![],
                help: include_str!("../help/datasets.md"),
                accuracy: None,
            },
            body: datasets,
        },
    ]
}
