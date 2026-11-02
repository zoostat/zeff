//! `save` (catalogue row 124; Language Reference, section 6.1, "Saving"; section 3.8, "CSV
//! writing", "Exports" and "Paths"; section 4.3; Engine and Project Specification, section 2,
//! "Writes"; release 0.6.0, step 15): a frame written as CSV or TSV under the project folder,
//! the format chosen by the extension or `format:`. The writing rules of section 3.8: UTF-8
//! without a byte-order mark unless `bom: true`, LF line endings, fields quoted only when they
//! hold the delimiter, a quote or a line break, `NA` as an empty cell, `Inf` as `Inf`, Floats as
//! the shortest text that reads back to the same number (with a point or an exponent, so that
//! the column reads back as Float), dates and datetimes in ISO 8601, categories as labels unless
//! `codes: true`. Parquet, Arrow, Stata, Excel and JSON arrive with release 0.11.0 and are
//! refused by name until then. The bytes written are the same on every target: the conformance
//! report carries them.

use std::sync::Arc;
use zeff_columns::{Column, ColumnType, Table, Value as Cell};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};

use crate::verbs::the_frame;

/// The text of one cell as the file holds it, before quoting; `None` for `NA`.
fn cell_text(column: &Column, row: usize, codes: bool) -> Option<String> {
    match column.get(row)? {
        Cell::Na => None,
        Cell::Bool(b) => Some(if b { "true" } else { "false" }.to_string()),
        Cell::Int(i) => Some(i.to_string()),
        Cell::Float(x) => Some(float_text(x)),
        Cell::Str(s) => Some(s.to_string()),
        Cell::Date(d) => Some(zeff_dates::format_iso_date(d)),
        Cell::DateTime(t) => Some(zeff_dates::format_iso_datetime(t)),
        Cell::Category(label) => {
            if codes {
                // The level's position, 1-based, as `to(x, Int)` gives it.
                column
                    .as_codes()
                    .and_then(|mut it| it.nth(row))
                    .flatten()
                    .map(|code| (code + 1).to_string())
            } else {
                Some(label.to_string())
            }
        }
        Cell::Opaque(_) => None,
    }
}

/// A Float as the shortest text that reads back to the same number, always with a point or an
/// exponent so that the column reads back as Float: `120.0`, `0.1`, `1e300`; `Inf` and `-Inf`;
/// both zeros as `0.0` (section 4.1: `-0.0` equals `0.0`).
pub fn float_text(x: f64) -> String {
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_string();
    }
    if x == 0.0 {
        return "0.0".to_string();
    }
    format!("{x:?}")
}

/// A field quoted when it holds the delimiter, a quote, a line break, or leading or trailing
/// blanks, a quote inside doubled (RFC 4180).
fn quoted(text: &str, delim: u8) -> String {
    let needs = text
        .bytes()
        .any(|b| b == delim || b == b'"' || b == b'\n' || b == b'\r')
        || text.starts_with(' ')
        || text.ends_with(' ');
    if needs {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

/// The bytes of a table as delimited text under the writing rules; `Err` names an opaque
/// column, which the text formats cannot hold.
pub fn delimited_bytes(
    table: &Table,
    delim: u8,
    bom: bool,
    codes: bool,
) -> Result<Vec<u8>, String> {
    for (name, column) in table.columns() {
        if let ColumnType::Opaque(_) = column.ty() {
            return Err(name.clone());
        }
    }
    table.compact();
    let mut out = String::new();
    if bom {
        out.push('\u{feff}');
    }
    let sep = (delim as char).to_string();
    let header: Vec<String> = table.names().iter().map(|n| quoted(n, delim)).collect();
    out.push_str(&header.join(&sep));
    out.push('\n');
    let columns: Vec<&Column> = table.columns().map(|(_, c)| c).collect();
    for row in 0..table.rows() {
        let mut first = true;
        for column in &columns {
            if !first {
                out.push_str(&sep);
            }
            first = false;
            if let Some(text) = cell_text(column, row, codes) {
                out.push_str(&quoted(&text, delim));
            }
        }
        out.push('\n');
    }
    Ok(out.into_bytes())
}

/// The format a path or `format:` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Csv,
    Tsv,
}

/// A format of a later release, with the release that brings it.
fn later(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "parquet" => Some(("Parquet", "0.11.0")),
        "arrow" => Some(("Arrow", "0.11.0")),
        "dta" | "stata" => Some(("Stata", "0.11.0")),
        "xlsx" | "excel" => Some(("Excel", "0.11.0")),
        "json" => Some(("JSON", "0.11.0")),
        "sav" | "spss" => Some(("SPSS", "1.1")),
        _ => None,
    }
}

/// `data.save(path, format:, bom:, codes:, version:)`.
pub fn save(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let frame = the_frame(ctx, "save", &args)?.clone();
    let path = match args.get(1) {
        Some(Value::Str(path)) => path.clone(),
        _ => String::new(),
    };
    let span = ctx.span_of(1);
    let name = path.rsplit('/').next().unwrap_or(&path).to_string();
    let extension = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let format_option = match args.get(2) {
        Some(Value::Str(f)) => Some(f.trim().trim_start_matches('.').to_ascii_lowercase()),
        _ => None,
    };
    if let Some((stem, ext)) = name.rsplit_once('.')
        && stem.is_empty()
        && !ext.is_empty()
    {
        return Err(fail(
            Diagnostic::error(
                "Z0716",
                format!("`{path}` has no name before its extension"),
            )
            .in_file(ctx.script)
            .primary(span, "a name is needed")
            .help(format!(
                "write a name before the extension: `output/data.{ext}`"
            )),
        ));
    }
    let chosen = format_option.clone().unwrap_or_else(|| extension.clone());
    let format = match chosen.as_str() {
        "csv" => Format::Csv,
        "tsv" | "tab" => Format::Tsv,
        other => {
            let d = if let Some((what, release)) = later(other) {
                Diagnostic::error(
                    "Z0716",
                    format!("`save` writes {what} files from release {release}; this release writes CSV and TSV"),
                )
                .primary(
                    if format_option.is_some() { ctx.span_of(2) } else { span },
                    format!("{what} arrives with release {release}"),
                )
                .help("write `.csv` or `.tsv` for now; Parquet, Arrow, Stata, Excel and JSON come with the reader interface of 0.11.0")
            } else if other.is_empty() {
                Diagnostic::error(
                    "Z0716",
                    format!("`{path}` has no extension, so no format is chosen"),
                )
                .primary(span, "which format?")
                .help(format!(
                    "write `.csv` or `.tsv`, or give `format: \"csv\"`: `output/{}.csv`",
                    if name.is_empty() {
                        "data"
                    } else {
                        name.trim_end_matches('.')
                    }
                ))
            } else {
                Diagnostic::error(
                    "Z0716",
                    format!("`.{other}` is not a format `save` writes"),
                )
                .primary(
                    if format_option.is_some() { ctx.span_of(2) } else { span },
                    "not a format of `save`",
                )
                .help("this release writes `.csv` and `.tsv`; Parquet, Arrow, Stata, Excel and JSON come with release 0.11.0")
            };
            return Err(fail(d.in_file(ctx.script)));
        }
    };
    let bom = matches!(args.get(3), Some(Value::Bool(true)));
    let codes = matches!(args.get(4), Some(Value::Bool(true)));
    let delim = match format {
        Format::Csv => b',',
        Format::Tsv => b'\t',
    };
    let bytes = delimited_bytes(&frame.table, delim, bom, codes).map_err(|column| {
        fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{column}` is a column of {} that a text file cannot hold",
                    frame
                        .table
                        .column(&column)
                        .map(|c| c.ty().name().to_string())
                        .unwrap_or_default()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a frame with an opaque column")
            .help(format!("`drop({column})` before saving; Parquet and Arrow, which keep every type, arrive with release 0.11.0")),
        )
    })?;
    // `output/` by default (section 2, "Writes"): a bare name is written there; a path with a
    // folder, or one of the form `root:file`, goes to the project's rules as written.
    let written = if path.contains('/') || path.contains(':') {
        path
    } else {
        format!("output/{path}")
    };
    ctx.files.write(&written, &bytes, span, ctx.script)?;
    ctx.message(Diagnostic::plain_note(format!(
        "saved {written} · {} × {}",
        match frame.table.rows() {
            1 => "1 row".to_string(),
            n => format!("{} rows", count(n as u64)),
        },
        match frame.table.n_cols() {
            1 => "1 variable".to_string(),
            n => format!("{n} variables"),
        }
    )));
    Ok(Value::Nothing)
}

/// A table holding every type and every awkward value the writing rules name, for the sample
/// files the reference checks read back (`tests/reference/exports/sample.csv`).
pub fn sample() -> Table {
    let (floats, _) = Column::floats([
        Some(1.5),
        Some(-0.0),
        Some(0.1),
        Some(f64::INFINITY),
        Some(123_456_789.123_456_79),
        None,
        Some(1e300),
        Some(120.0),
    ]);
    let levels = vec!["low".to_string(), "mid".to_string(), "high".to_string()];
    let categories = Column::categories(
        levels,
        [
            Some(0),
            Some(2),
            Some(1),
            None,
            Some(2),
            Some(0),
            Some(1),
            Some(2),
        ],
    )
    .expect("a category with valid codes");
    let _ = Arc::new(());
    Table::new(vec![
        (
            "id".to_string(),
            Column::ints([
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(8),
            ]),
        ),
        ("x".to_string(), floats),
        (
            "flag".to_string(),
            Column::bools([
                Some(true),
                Some(false),
                None,
                Some(true),
                Some(false),
                Some(true),
                None,
                Some(false),
            ]),
        ),
        (
            "note".to_string(),
            Column::strs([
                Some("plain"),
                Some("with, comma"),
                Some("with \"quotes\""),
                Some("two\nlines"),
                Some(""),
                None,
                Some(" padded "),
                Some("naïve café"),
            ]),
        ),
        (
            "day".to_string(),
            Column::dates([
                Some(0),
                Some(19_723),
                None,
                Some(-25_567),
                Some(2_932_896),
                Some(1),
                Some(-1),
                Some(18_000),
            ]),
        ),
        (
            "at".to_string(),
            Column::datetimes([
                Some(0),
                Some(1_704_067_200_000_000),
                None,
                Some(1_704_067_200_123_456),
                Some(-1_000_000),
                Some(86_400_000_000),
                Some(1),
                Some(999_999),
            ]),
        ),
        ("level".to_string(), categories),
    ])
    .expect("the sample table builds")
}
