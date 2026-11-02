//! `table` (catalogue row 530; open decision #72, taken 1 November 2026; Language Reference,
//! section 3.8, "Table", and section 4, "Output"; release 0.6.0, step 16): a `Table` value built
//! from a record whose fields are lists or columns of one length each, so that a function
//! written in Zeff can shape a table of its own. Numbers are right-aligned and printed by the
//! display rules; text, Bools and dates are left-aligned; a caller formats numbers first with
//! `format` when it wants a chosen number of decimals. The table prints, exports and lists as
//! every table the engine makes.

use zeff_dates::{format_iso_date, format_iso_datetime};
use zeff_diagnostics::Diagnostic;

use crate::display::{alignment, format_float, labelled_cell};
use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::{Align, Table, Value};
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// One field of `columns:` as the cells of a column and its alignment.
struct Prepared {
    name: String,
    cells: Vec<String>,
    align: Align,
}

/// A scalar as a cell prints it; `None` for a value a cell cannot show.
fn cell_text(value: &Value) -> Option<String> {
    Some(match value {
        Value::Na => "NA".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(x) => format_float(*x),
        Value::Str(s) => s.clone(),
        Value::Date(d) => format_iso_date(*d),
        Value::DateTime(t) => format_iso_datetime(*t),
        _ => return None,
    })
}

/// `table(title:, columns:)`.
pub fn table(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let title = match args.get(0) {
        Some(Value::Str(t)) => t.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`table` needs `title:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`title:` is missing")
                    .help(
                        "usage: `table(title: \"by year\", columns: {year: years, mean: means})`",
                    ),
            ));
        }
    };
    let fields = match args.get(1) {
        Some(Value::Record(fields)) => fields.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`table` needs `columns:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`columns:` is missing")
                    .help("usage: `table(title: \"by year\", columns: {year: years, mean: means})`, each field a list or a column"),
            ));
        }
    };
    if fields.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0401", "`columns:` of `table` holds no field")
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "an empty record")
                .help("give at least one column: `columns: {year: [2010, 2011]}`"),
        ));
    }
    let mut prepared: Vec<Prepared> = Vec::with_capacity(fields.len());
    for (name, value) in &fields {
        let column = match value {
            Value::Column(c) => Prepared {
                name: name.clone(),
                cells: (0..c.len()).map(|r| labelled_cell(c, r)).collect(),
                align: alignment(c.ty()),
            },
            Value::List(items) => {
                let mut cells = Vec::with_capacity(items.len());
                let mut numeric = true;
                for item in items {
                    if let Value::Failed(failure) = item {
                        return Err(fail(
                            Diagnostic::error(
                                "Z0220",
                                format!(
                                    "a failed value cannot be a cell of `{name}`: it failed with {} at {}: {}",
                                    failure.code,
                                    failure.place(),
                                    failure.message
                                ),
                            )
                            .in_file(ctx.script)
                            .primary(ctx.span_of(1), "a failed value in a column")
                            .help("test it first with `x.is(Failed)`, or leave it out of the list"),
                        ));
                    }
                    match cell_text(item) {
                        Some(text) => {
                            if !matches!(item, Value::Int(_) | Value::Float(_) | Value::Na) {
                                numeric = false;
                            }
                            cells.push(text);
                        }
                        None => {
                            return Err(fail(
                                Diagnostic::error(
                                    "Z0401",
                                    format!(
                                        "`{name}` of `columns:` holds {}, which a cell cannot show",
                                        item.type_word()
                                    ),
                                )
                                .in_file(ctx.script)
                                .primary(ctx.span_of(1), "not a value a cell holds")
                                .help("a column of a table is a list of text, numbers, Bools or dates, or a column of a frame"),
                            ));
                        }
                    }
                }
                Prepared {
                    name: name.clone(),
                    cells,
                    align: if numeric && !items.is_empty() {
                        Align::Right
                    } else {
                        Align::Left
                    },
                }
            }
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`{name}` of `columns:` is {}, not a list or a column",
                            other.type_word()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(1), "not a column")
                    .help("each field of `columns:` is a list of values or a column of a frame, all of one length"),
                ));
            }
        };
        prepared.push(column);
    }
    let rows = prepared[0].cells.len();
    if let Some(odd) = prepared.iter().find(|p| p.cells.len() != rows) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "the columns of `table` differ in length: `{}` has {} and `{}` has {}",
                    prepared[0].name,
                    rows_text(rows),
                    odd.name,
                    rows_text(odd.cells.len())
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "columns of different lengths")
            .help("every column of a table has one value per row"),
        ));
    }
    let columns: Vec<(String, Align)> =
        prepared.iter().map(|p| (p.name.clone(), p.align)).collect();
    let rows_out: Vec<Vec<String>> = (0..rows)
        .map(|r| prepared.iter().map(|p| p.cells[r].clone()).collect())
        .collect();
    Ok(Value::Table(Table {
        title,
        before: Vec::new(),
        columns,
        rows: rows_out,
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    }))
}

fn rows_text(n: usize) -> String {
    format!("{n} {}", if n == 1 { "row" } else { "rows" })
}

pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "table",
            package: "core",
            method: false,
            params: vec![
                ParamSpec::option("title", "Str"),
                ParamSpec::option("columns", "Record"),
            ],
            rest: None,
            returns: "Table",
            diagnostics: vec![
                DiagnosticDecl::new("Z0406", "title: or columns: missing"),
                DiagnosticDecl::new(
                    "Z0401",
                    "an empty record, a field that is not a list or a column, a value a cell cannot show, or columns of different lengths",
                ),
                DiagnosticDecl::new("Z0220", "a failed value among a column's values"),
            ],
            help: include_str!("../help/table.md"),
            accuracy: None,
        },
        body: table,
    }]
}
