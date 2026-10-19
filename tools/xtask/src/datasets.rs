//! `cargo run -p xtask -- build-datasets`: the Arrow files of the shipped datasets (Engine and
//! Project Specification, section 8.5), built from the CSV files the generating scripts write,
//! with the labels, units, value labels, categories and panel declaration of each dataset, and
//! the BLAKE3 checksums of every shipped file. Continuous integration rebuilds and compares, so a
//! shipped file is always what the scripts and these declarations produce. Parquet, the kept
//! format of section 8.7, arrives with its reader and writer at 0.11.0; until then the Arrow IPC
//! form of the kernel carries the same metadata.

use std::path::Path;
use std::process::ExitCode;
use zeff_columns::{Column, FrameMetadata, MetadataKey, Table, write_ipc};
use zeff_data::csv::{CsvOptions, TypeSpec, read_csv};

/// What a dataset's variables carry beyond their values.
struct Variable {
    name: &'static str,
    ty: TypeSpec,
    label: &'static str,
    unit: &'static str,
    /// Value labels as JSON, `{"0": "male", "1": "female"}`, or empty.
    value_labels: &'static str,
    /// The levels of a category, in order; empty for the other types.
    levels: &'static [&'static str],
}

struct Dataset {
    name: &'static str,
    source: &'static str,
    panel: Option<(&'static str, &'static str)>,
    variables: Vec<Variable>,
}

fn variable(name: &'static str, ty: TypeSpec, label: &'static str) -> Variable {
    Variable {
        name,
        ty,
        label,
        unit: "",
        value_labels: "",
        levels: &[],
    }
}

const INDUSTRIES: [&str; 12] = [
    "Agriculture",
    "Mining",
    "Utilities",
    "Construction",
    "Manufacturing",
    "Retail",
    "Transport",
    "Finance",
    "Services",
    "Education",
    "Health",
    "Public administration",
];

fn declarations() -> Vec<Dataset> {
    vec![
        Dataset {
            name: "wages",
            source: "zeff:wages.csv (synthetic; generate/wages.py, seed 20261019)",
            panel: Some(("worker", "year")),
            variables: vec![
                variable("worker", TypeSpec::Int, "Worker identifier"),
                variable("year", TypeSpec::Int, "Calendar year"),
                Variable {
                    unit: "USD per hour",
                    ..variable("wage", TypeSpec::Float, "Hourly wage")
                },
                Variable {
                    unit: "years",
                    ..variable("educ", TypeSpec::Int, "Years of schooling")
                },
                Variable {
                    unit: "years",
                    ..variable(
                        "exper",
                        TypeSpec::Int,
                        "Potential labour-market experience: age less schooling less six",
                    )
                },
                Variable {
                    unit: "years",
                    ..variable("age", TypeSpec::Int, "Age")
                },
                Variable {
                    unit: "hours per week",
                    ..variable("hours", TypeSpec::Float, "Hours worked")
                },
                Variable {
                    value_labels: "{\"0\": \"no\", \"1\": \"yes\"}",
                    ..variable("union", TypeSpec::Int, "Union member")
                },
                Variable {
                    value_labels: "{\"0\": \"male\", \"1\": \"female\"}",
                    ..variable("female", TypeSpec::Int, "Sex")
                },
                Variable {
                    levels: &INDUSTRIES,
                    ..variable("industry", TypeSpec::Str, "Industry")
                },
            ],
        },
        Dataset {
            name: "sleep",
            source: "zeff:sleep.csv (Student 1908, from Cushny and Peebles 1905)",
            panel: None,
            variables: vec![
                Variable {
                    unit: "hours",
                    ..variable(
                        "extra",
                        TypeSpec::Float,
                        "Extra hours of sleep, against control",
                    )
                },
                Variable {
                    value_labels: "{\"1\": \"dextro-hyoscyamine\", \"2\": \"laevo-hyoscyamine\"}",
                    ..variable("group", TypeSpec::Int, "Drug given")
                },
                variable("id", TypeSpec::Int, "Patient"),
            ],
        },
    ]
}

fn build_one(folder: &Path, dataset: &Dataset) -> Result<(String, String), String> {
    let csv_path = folder.join(format!("{}.csv", dataset.name));
    let bytes = std::fs::read(&csv_path).map_err(|e| format!("{}: {e}", csv_path.display()))?;
    let options = CsvOptions {
        types: dataset
            .variables
            .iter()
            .map(|v| (v.name.to_string(), v.ty.clone()))
            .collect(),
        ..CsvOptions::default()
    };
    let read = read_csv(&bytes, &options).map_err(|e| format!("{}: {e:?}", dataset.name))?;
    let mut columns: Vec<(String, Column)> = Vec::new();
    for v in &dataset.variables {
        let column = read
            .table
            .column(v.name)
            .ok_or_else(|| format!("{}: no column `{}` in the CSV", dataset.name, v.name))?;
        let mut column = if v.levels.is_empty() {
            column.clone()
        } else {
            // A category: the levels in the declared order, each row its level's position.
            let labels: Vec<Option<&str>> = column
                .as_strs()
                .ok_or_else(|| format!("{}: `{}` is not text", dataset.name, v.name))?
                .collect();
            let codes = labels.iter().map(|label| {
                label.map(|l| {
                    v.levels
                        .iter()
                        .position(|lvl| *lvl == l)
                        .unwrap_or_else(|| {
                            panic!("{}: `{l}` is not a level of {}", dataset.name, v.name)
                        }) as i32
                })
            });
            Column::categories(v.levels.iter().map(|l| l.to_string()).collect(), codes)
                .map_err(|e| format!("{}: {e}", dataset.name))?
        };
        column = column.with_entry(MetadataKey::Label, v.label);
        if !v.unit.is_empty() {
            column = column.with_entry(MetadataKey::Unit, v.unit);
        }
        if !v.value_labels.is_empty() {
            column = column.with_entry(MetadataKey::ValueLabels, v.value_labels);
        }
        columns.push((v.name.to_string(), column));
    }
    let table = Table::new(columns)
        .map_err(|e| format!("{}: {e}", dataset.name))?
        .with_metadata(FrameMetadata {
            source: Some(dataset.source.to_string()),
            panel: dataset
                .panel
                .map(|(unit, time)| (unit.to_string(), time.to_string())),
        });
    let ipc = write_ipc(&table).map_err(|e| format!("{}: {e}", dataset.name))?;
    let arrow_path = folder.join(format!("{}.arrow", dataset.name));
    std::fs::write(&arrow_path, &ipc).map_err(|e| format!("{}: {e}", arrow_path.display()))?;
    println!(
        "{}: {} rows × {} variables, {} bytes of CSV, {} bytes of Arrow",
        dataset.name,
        table.rows(),
        table.n_cols(),
        bytes.len(),
        ipc.len()
    );
    Ok((
        format!("{}  {}.csv", zeff_hash::labelled(&bytes), dataset.name),
        format!("{}  {}.arrow", zeff_hash::labelled(&ipc), dataset.name),
    ))
}

pub fn build() -> ExitCode {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/tier1/datasets/data");
    let mut checksums: Vec<String> = Vec::new();
    for dataset in declarations() {
        match build_one(&folder, &dataset) {
            Ok((csv, arrow)) => {
                checksums.push(csv);
                checksums.push(arrow);
            }
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(1);
            }
        }
    }
    checksums.sort();
    let text = format!(
        "# BLAKE3 checksums of the shipped dataset files (cargo run -p xtask -- build-datasets)\n{}\n",
        checksums.join("\n")
    );
    if let Err(e) = std::fs::write(folder.join("CHECKSUMS"), text) {
        eprintln!("CHECKSUMS: {e}");
        return ExitCode::from(1);
    }
    println!("wrote CHECKSUMS");
    ExitCode::SUCCESS
}
