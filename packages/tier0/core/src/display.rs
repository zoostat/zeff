//! The display interface (Engine and Project Specification, section 8.1, rule 5): how any value
//! prints, by the rules of Language Reference section 3.8, "Printing".
//!
//! | Value | Printed as |
//! | --- | --- |
//! | `Int`, `Float` | An Int in full; a Float to six significant digits with trailing zeros dropped, in exponent form below 1e-4 and from 1e15; `-0.0` as `0`; `NA`, `Inf`, `-Inf` as written |
//! | `Str`, `Bool` | The value itself |
//! | List, Record, Mapping | One line per item, field or `key => value` entry, up to 20, then the count of the rest |
//! | `Nothing` | Nothing |
//! | Frame | Its `describe` header, a blank line, and its first ten rows with their numbers, the columns aligned as a table's are; then the count of the rows left |
//! | Table | Its title, a blank line, and its rows under their headings: two spaces in, three between columns, numbers to the right and text to the left |
//! | `Column<T>` | Its type and length on one line, then its values as a list prints them |
//!
//! A table wider than the page (118 characters, the width of a help page) prints its columns in
//! blocks, as R does, every value shown; a frame repeats the row numbers at the start of each block.

use crate::value::{Align, Distribution, Frame, Table, Value};
use zeff_columns::{ColumnType, MetadataKey, Value as Cell};
use zeff_dates::{format_iso_date, format_iso_datetime};
use zeff_diagnostics::count;

/// The most items of a list or fields of a record that print; the rest is counted.
pub const LIST_LIMIT: usize = 20;

/// The rows of a frame that print (section 3.1).
pub const FRAME_ROWS: usize = 10;

/// The width of a page, in characters: a table wider than this prints in blocks of columns.
pub const PAGE_WIDTH: usize = 118;

/// A Float as section 3.8 prints it. Six significant digits, as R prints with its default and as
/// Stata's `%9.0g` does; a value of a million or more keeps every digit before the point and
/// rounds to a whole number, as R does, rather than losing digits before the point.
pub fn format_float(x: f64) -> String {
    if x.is_nan() {
        return "NA".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        };
    }
    if x == 0.0 {
        return "0".to_string();
    }
    // Six significant digits in scientific form give the decimal exponent exactly, rounding
    // included, so that 999999.5 is seen as 1e6.
    let scientific = format!("{x:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    if !(-4..15).contains(&exponent) {
        let mantissa = trim_zeros(mantissa);
        let sign = if exponent < 0 { "-" } else { "+" };
        return format!("{mantissa}e{sign}{:02}", exponent.abs());
    }
    let decimals = (5 - exponent).max(0) as usize;
    trim_zeros(&format!("{x:.decimals$}"))
}

/// Drop trailing zeros after a decimal point, and the point itself when nothing follows it.
fn trim_zeros(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// A value on one line, as it appears inside a list or a record.
fn inline(value: &Value) -> String {
    match value {
        Value::Nothing => String::new(),
        Value::Na => "NA".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(x) => format_float(*x),
        Value::Str(s) => s.clone(),
        Value::Frame(f) => frame_header(f),
        Value::Table(t) => t.title.clone(),
        Value::Column(c) => column_header(c),
        Value::Distribution(d) => distribution_text(d),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(inline).collect::<Vec<_>>().join(", ")
        ),
        Value::Record(fields) => {
            format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(n, v)| format!("{n}: {}", inline(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        Value::Mapping(entries) => format!(
            "[{}]",
            entries
                .iter()
                .map(|(k, v)| format!("{} => {}", inline(k), inline(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Var(name) | Value::Word(name) => name.clone(),
        Value::Vars(names) => names.join(", "),
        Value::Type(t) => match &t.format {
            Some(f) => format!("{}(\"{f}\")", t.name),
            None => t.name.clone(),
        },
    }
}

/// The first line of a printed column: `Column<Float> · 5 values` (section 3.8).
pub fn column_header(column: &zeff_columns::Column) -> String {
    format!(
        "Column<{}> · {} {}",
        column.ty().name(),
        count(column.len() as u64),
        if column.len() == 1 { "value" } else { "values" }
    )
}

/// The header line of a frame: `wages · 18,412 rows × 13 variables`.
pub fn frame_header(frame: &Frame) -> String {
    let rows = frame.table.rows();
    let cols = frame.table.n_cols();
    let panel = match &frame.table.metadata().panel {
        Some((unit, time)) => format!(" · panel {unit} × {time}"),
        None => String::new(),
    };
    format!(
        "{} · {} {} × {} {}{panel}",
        frame.name.as_deref().unwrap_or("frame"),
        count(rows as u64),
        if rows == 1 { "row" } else { "rows" },
        count(cols as u64),
        if cols == 1 { "variable" } else { "variables" }
    )
}

/// One cell of a column as it prints: the same rule as a scalar of its type, a date in ISO form,
/// a category by its label.
pub fn cell(value: Cell<'_>) -> String {
    match value {
        Cell::Na => "NA".to_string(),
        Cell::Bool(b) => b.to_string(),
        Cell::Int(i) => i.to_string(),
        Cell::Float(x) => format_float(x),
        Cell::Str(s) | Cell::Category(s) => s.to_string(),
        Cell::Date(days) => format_iso_date(days),
        Cell::DateTime(micros) => format_iso_datetime(micros),
        Cell::Opaque(summary) => summary,
    }
}

/// A cell of `column` as it prints, a labelled value by its label (section 10.2: a variable with
/// value labels, `female` with 0 male and 1 female, prints the labels).
pub fn labelled_cell(column: &zeff_columns::Column, row: usize) -> String {
    let value = column.get(row).unwrap_or(Cell::Na);
    if let Some(labels) = column.metadata().value_labels()
        && let Some(label) = value_label(&labels, &value)
    {
        return label;
    }
    cell(value)
}

/// The label of `value` among the value labels of the store (the code as text, read by the one
/// reader of the column kernel), or `None` when the value has no label.
fn value_label(labels: &[(String, String)], value: &Cell<'_>) -> Option<String> {
    let key = match value {
        Cell::Int(i) => i.to_string(),
        Cell::Float(x) => format_float(*x),
        Cell::Str(s) => (*s).to_string(),
        Cell::Bool(b) => b.to_string(),
        _ => return None,
    };
    labels
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.clone())
}

/// How a column's cells line up: numbers to the right, everything else to the left.
pub fn alignment(ty: ColumnType) -> Align {
    match ty {
        ColumnType::Int | ColumnType::Float => Align::Right,
        _ => Align::Left,
    }
}

/// The alignment of a column as printed: a labelled number prints as text.
pub fn column_alignment(column: &zeff_columns::Column) -> Align {
    if column.metadata().get(MetadataKey::ValueLabels).is_some() {
        Align::Left
    } else {
        alignment(column.ty())
    }
}

/// A frame as a statement prints it: the header, a blank line, the first ten rows numbered, and
/// the count of the rows left.
fn frame_text(frame: &Frame, width: usize) -> String {
    let table = &frame.table;
    let shown = table.rows().min(FRAME_ROWS);
    let mut columns = vec![(String::new(), Align::Right)];
    for (i, name) in table.names().iter().enumerate() {
        let align = table.column_at(i).map_or(Align::Left, column_alignment);
        columns.push((name.clone(), align));
    }
    let rows: Vec<Vec<String>> = (0..shown)
        .map(|r| {
            let mut row = vec![(r + 1).to_string()];
            for i in 0..table.n_cols() {
                row.push(match table.column_at(i) {
                    Some(column) => labelled_cell(column, r),
                    None => "NA".to_string(),
                });
            }
            row
        })
        .collect();
    let mut out = format!("{}\n\n", frame_header(frame));
    out.push_str(&grid(&columns, &rows, true, width));
    if table.rows() > shown {
        let rest = table.rows() - shown;
        out.push_str(&format!(
            "  … and {} more {}\n",
            count(rest as u64),
            if rest == 1 { "row" } else { "rows" }
        ));
    }
    out
}

/// A table as it prints: its title, a blank line, then its grid.
fn table_text(table: &Table, width: usize) -> String {
    let mut text = format!(
        "{}\n\n{}",
        table.title,
        grid(&table.columns, &table.rows, false, width)
    );
    for further in &table.more {
        text.push('\n');
        text.push_str(&table_text(further, width));
    }
    text
}

/// The width of a cell in characters.
fn width(text: &str) -> usize {
    text.chars().count()
}

/// Rows of cells under their headings: two spaces in, three between columns, each column as wide
/// as its widest cell or heading, aligned as the column says. Columns that do not fit the page
/// (`page_width` characters) go into further blocks separated by a blank line; with
/// `repeat_first`, the first column (a frame's row numbers) starts every block.
fn grid(
    columns: &[(String, Align)],
    rows: &[Vec<String>],
    repeat_first: bool,
    page_width: usize,
) -> String {
    let widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(i, (heading, _))| {
            rows.iter()
                .filter_map(|row| row.get(i))
                .map(|c| width(c))
                .chain(std::iter::once(width(heading)))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let fixed: Vec<usize> = if repeat_first && !columns.is_empty() {
        vec![0]
    } else {
        Vec::new()
    };
    let line_width = |block: &[usize]| -> usize {
        2 + block.iter().map(|&i| widths[i]).sum::<usize>() + 3 * block.len().saturating_sub(1)
    };
    let mut blocks: Vec<Vec<usize>> = Vec::new();
    let mut block = fixed.clone();
    for i in fixed.len()..columns.len() {
        let mut candidate = block.clone();
        candidate.push(i);
        if line_width(&candidate) > page_width && block.len() > fixed.len() {
            blocks.push(block);
            block = fixed.clone();
            block.push(i);
        } else {
            block = candidate;
        }
    }
    if block.len() > fixed.len() || blocks.is_empty() {
        blocks.push(block);
    }
    let mut out = String::new();
    for (b, block) in blocks.iter().enumerate() {
        if b > 0 {
            out.push('\n');
        }
        let line = |cells: &dyn Fn(usize) -> String| -> String {
            let mut text = String::from("  ");
            for (k, &i) in block.iter().enumerate() {
                if k > 0 {
                    text.push_str("   ");
                }
                let value = cells(i);
                let pad = widths[i].saturating_sub(width(&value));
                match columns[i].1 {
                    Align::Right => {
                        text.push_str(&" ".repeat(pad));
                        text.push_str(&value);
                    }
                    Align::Left => {
                        text.push_str(&value);
                        text.push_str(&" ".repeat(pad));
                    }
                }
            }
            format!("{}\n", text.trim_end())
        };
        out.push_str(&line(&|i| columns[i].0.clone()));
        for row in rows {
            out.push_str(&line(&|i| row.get(i).cloned().unwrap_or_default()));
        }
    }
    out
}

/// What a statement that is not a `let` prints for its value: the text, ending in a line break,
/// or nothing at all for `Nothing`; frames and tables wrap at the default page width.
pub fn show(value: &Value) -> String {
    show_width(value, PAGE_WIDTH)
}

/// [`show`] with the page width `width` (`--width`, Engine and Project Specification, section
/// 2.1): a frame or table wider than it prints in blocks of columns.
pub fn show_width(value: &Value, width: usize) -> String {
    match value {
        Value::Nothing => String::new(),
        Value::List(items) => lines(items.iter().map(inline), items.len(), "item"),
        Value::Record(fields) => lines(
            fields.iter().map(|(n, v)| format!("{n}: {}", inline(v))),
            fields.len(),
            "field",
        ),
        Value::Mapping(entries) => lines(
            entries
                .iter()
                .map(|(k, v)| format!("{} => {}", inline(k), inline(v))),
            entries.len(),
            "entry",
        ),
        Value::Frame(f) => frame_text(f, width),
        Value::Table(t) => table_text(t, width),
        Value::Column(c) => {
            let mut out = format!("{}\n", column_header(c));
            out.push_str(&lines(
                (0..c.len()).map(|r| cell(c.get(r).unwrap_or(Cell::Na))),
                c.len(),
                "value",
            ));
            out
        }
        other => format!("{}\n", inline(other)),
    }
}

fn lines(items: impl Iterator<Item = String>, total: usize, what: &str) -> String {
    let mut out = String::new();
    for item in items.take(LIST_LIMIT) {
        out.push_str(&item);
        out.push('\n');
    }
    if total > LIST_LIMIT {
        let rest = total - LIST_LIMIT;
        out.push_str(&format!(
            "… and {} more {}\n",
            count(rest as u64),
            if rest == 1 {
                what.to_string()
            } else {
                format!("{what}s")
            }
        ));
    }
    if total == 0 {
        out.push_str("[]\n");
    }
    out
}

/// A distribution on one line, in the form of the call that made it, the defaults filled in
/// (the owner's decision of 19 October 2026 on question 1 of 0.3.0): `t distribution · df: 24`,
/// `normal distribution · mean: 0, sd: 1`; a truncated distribution adds its bounds and a
/// mixture its components.
pub fn distribution_text(d: &Distribution) -> String {
    match d {
        Distribution::Family { name, params } => {
            let listed: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_float(*v)))
                .collect();
            if listed.is_empty() {
                format!("{name} distribution")
            } else {
                format!("{name} distribution · {}", listed.join(", "))
            }
        }
        Distribution::Truncated {
            inner,
            lower,
            upper,
        } => format!(
            "{} · truncated to [{}, {}]",
            distribution_text(inner),
            format_float(*lower),
            format_float(*upper)
        ),
        Distribution::Mixture { parts } => {
            let listed: Vec<String> = parts
                .iter()
                .map(|(d, p)| format!("{} × ({})", format_float(*p), distribution_text(d)))
                .collect();
            format!("mixture · {}", listed.join(" + "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_print_as_section_3_8_says() {
        assert_eq!(format_float(0.975002), "0.975002");
        assert_eq!(format_float(4.90671e-198), "4.90671e-198");
        assert_eq!(format_float(24.533_333_333), "24.5333");
        assert_eq!(format_float(23.4), "23.4");
        assert_eq!(format_float(19.0), "19");
        assert_eq!(format_float(-0.0), "0");
        assert_eq!(format_float(0.0), "0");
        assert_eq!(format_float(100_000.0), "100000");
        assert_eq!(format_float(0.0001), "0.0001", "1e-4 is still fixed");
        assert_eq!(
            format_float(0.00009),
            "9e-05",
            "below 1e-4 is exponent form, as R writes it"
        );
        assert_eq!(format_float(1e15), "1e+15", "from 1e15 is exponent form");
        assert_eq!(
            format_float(999_999_999_999_999.0),
            "1e+15",
            "rounds up to 1e15 at six digits, as R prints it"
        );
        assert_eq!(
            format_float(1_234_567.891),
            "1234568",
            "a million or more keeps its digits before the point"
        );
        assert_eq!(format_float(-1.23456789), "-1.23457");
        assert_eq!(format_float(2.0 / 3.0), "0.666667");
        assert_eq!(format_float(1e-5), "1e-05");
        assert_eq!(format_float(1.5e-7), "1.5e-07");
        assert_eq!(format_float(f64::INFINITY), "Inf");
        assert_eq!(format_float(f64::NEG_INFINITY), "-Inf");
        assert_eq!(format_float(f64::NAN), "NA");
        assert_eq!(
            format_float(999_999.5),
            "1000000",
            "rounds up into seven digits"
        );
    }

    #[test]
    fn scalars_and_collections() {
        assert_eq!(show(&Value::Int(42)), "42\n");
        assert_eq!(show(&Value::Int(-7)), "-7\n");
        assert_eq!(show(&Value::Str("hello".into())), "hello\n");
        assert_eq!(show(&Value::Bool(true)), "true\n");
        assert_eq!(show(&Value::Na), "NA\n");
        assert_eq!(show(&Value::Nothing), "");
        assert_eq!(
            show(&Value::List(vec![
                Value::Int(1),
                Value::Float(2.5),
                Value::Na
            ])),
            "1\n2.5\nNA\n"
        );
        assert_eq!(show(&Value::List(vec![])), "[]\n");
        assert_eq!(
            show(&Value::Record(vec![
                ("se".into(), Value::Word("robust".into())),
                ("n".into(), Value::Int(3))
            ])),
            "se: robust\nn: 3\n"
        );
        let long = Value::List((1..=25).map(Value::Int).collect());
        let text = show(&long);
        assert_eq!(text.lines().count(), 21);
        assert!(text.ends_with("… and 5 more items\n"));
        let one_more = Value::List((1..=21).map(Value::Int).collect());
        assert!(show(&one_more).ends_with("… and 1 more item\n"));
        let nested = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Record(vec![("a".into(), Value::Int(1))]),
        ]);
        assert_eq!(show(&nested), "[1, 2]\n{a: 1}\n");
        assert_eq!(
            show(&Value::Type(crate::value::TypeValue {
                name: "Date".into(),
                format: Some("%d/%m/%Y".into())
            })),
            "Date(\"%d/%m/%Y\")\n"
        );
    }

    #[test]
    fn a_frame_prints_its_header_and_its_rows() {
        let table = Table::new(vec![
            ("a".into(), zeff_columns::Column::ints([Some(1), Some(2)])),
            ("b".into(), zeff_columns::Column::ints([None, None])),
        ])
        .unwrap();
        let mut frame = Frame::new(table);
        assert_eq!(
            show(&Value::Frame(frame.clone())),
            "frame · 2 rows × 2 variables\n\n      a    b\n  1   1   NA\n  2   2   NA\n"
        );
        frame.name = Some("wages".into());
        assert_eq!(frame_header(&frame), "wages · 2 rows × 2 variables");
        let one = Frame::new(
            Table::new(vec![("a".into(), zeff_columns::Column::ints([Some(1)]))]).unwrap(),
        );
        assert_eq!(frame_header(&one), "frame · 1 row × 1 variable");
    }

    #[test]
    fn ten_rows_print_then_the_count_and_text_is_left_numbers_right() {
        let table = Table::new(vec![
            (
                "n".into(),
                zeff_columns::Column::ints((1..=12).map(|i| Some(i * 100))),
            ),
            (
                "label".into(),
                zeff_columns::Column::strs((1..=12).map(|i| Some(format!("row {i}")))),
            ),
            (
                "x".into(),
                zeff_columns::Column::floats((1..=12).map(|i| Some(f64::from(i) / 4.0))).0,
            ),
            (
                "when".into(),
                zeff_columns::Column::dates((1..=12).map(|i| Some(20_000 + i))),
            ),
        ])
        .unwrap();
        let text = show(&Value::Frame(Frame::new(table)));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "frame · 12 rows × 4 variables");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "          n   label       x   when");
        assert_eq!(lines[3], "   1    100   row 1    0.25   2024-10-05");
        assert_eq!(lines[12], "  10   1000   row 10    2.5   2024-10-14");
        assert_eq!(lines[13], "  … and 2 more rows");
        assert_eq!(lines.len(), 14);
    }

    #[test]
    fn a_wide_frame_prints_in_blocks_with_the_row_numbers_repeated() {
        let columns: Vec<(String, zeff_columns::Column)> = (1..=30)
            .map(|c| {
                (
                    format!("variable_{c:02}"),
                    zeff_columns::Column::ints([Some(c), Some(c * 2)]),
                )
            })
            .collect();
        let text = show(&Value::Frame(Frame::new(Table::new(columns).unwrap())));
        let blocks: Vec<&str> = text.split("\n\n").collect();
        assert_eq!(blocks[0], "frame · 2 rows × 30 variables");
        assert!(blocks.len() > 2, "more than one block:\n{text}");
        for block in &blocks[1..] {
            for line in block.lines() {
                assert!(width(line) <= PAGE_WIDTH, "{line}");
            }
            let rows: Vec<&str> = block.lines().collect();
            assert_eq!(
                rows.len(),
                3,
                "heading and two rows in every block:\n{block}"
            );
            assert!(
                rows[1].starts_with("  1   ") && rows[2].starts_with("  2   "),
                "{block}"
            );
        }
        let headings: Vec<&str> = text.lines().filter(|l| l.contains("variable_")).collect();
        let named: usize = headings
            .iter()
            .map(|l| l.matches("variable_").count())
            .sum();
        assert_eq!(named, 30, "every column is shown once");
    }

    #[test]
    fn a_table_prints_its_title_and_aligned_columns() {
        let table = crate::value::Table {
            title: "wages · 3 rows × 3 variables".into(),
            columns: vec![
                ("variable".into(), Align::Left),
                ("non-missing".into(), Align::Right),
                ("example".into(), Align::Left),
            ],
            rows: vec![
                vec!["id".into(), "3".into(), "1".into()],
                vec!["hours".into(), "2".into(), "40".into()],
            ],
            more: Vec::new(),
        };
        assert_eq!(
            show(&Value::Table(table.clone())),
            "wages · 3 rows × 3 variables\n\n  variable   non-missing   example\n  id                   3   1\n  hours                2   40\n"
        );
        assert_eq!(inline(&Value::Table(table)), "wages · 3 rows × 3 variables");
    }

    #[test]
    fn opaque_columns_print_their_summaries_in_a_frame() {
        let table = Table::new(vec![
            ("id".into(), zeff_columns::Column::ints([Some(1), Some(2)])),
            (
                "scan".into(),
                zeff_columns::Column::file_refs([
                    Some(("scans/0142.jpg", Some([0u8; 32]))),
                    None::<(&str, Option<[u8; 32]>)>,
                ]),
            ),
            (
                "blob".into(),
                zeff_columns::Column::bytes([Some(vec![0u8; 2100]), Some(vec![1u8; 3])]),
            ),
            (
                "shape".into(),
                zeff_columns::Column::geometries([
                    Some(vec![1u8, 3, 0, 0, 0]),
                    Some(vec![1u8, 1, 0, 0, 0]),
                ]),
            ),
            (
                "q".into(),
                zeff_columns::Column::periods(
                    zeff_columns::Frequency::Quarter,
                    [Some(199), Some(200)],
                ),
            ),
        ])
        .unwrap();
        let text = show(&Value::Frame(Frame::new(table)));
        assert_eq!(
            text,
            "frame · 2 rows × 5 variables\n\n      id   scan                    blob             shape       q\n  1    1   <file scans/0142.jpg>   <bytes 2.1 kB>   <polygon>   2019Q4\n  2    2   NA                      <bytes 3 B>      <point>     2020Q1\n"
        );
    }

    use zeff_columns::Table;
}
