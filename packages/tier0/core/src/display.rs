//! The display interface (Engine and Project Specification, section 8.1, rule 5): how any value
//! prints, by the rules of Language Reference section 3.8, "Printing".
//!
//! | Value | Printed as |
//! | --- | --- |
//! | `Int`, `Float` | An Int in full; a Float to six significant digits with trailing zeros dropped, in exponent form below 1e-4 and from 1e15; `-0.0` as `0`; `NA`, `Inf`, `-Inf` as written |
//! | `Str`, `Bool` | The value itself |
//! | List, Record | One line per item or field, up to 20, then the count of the rest |
//! | `Nothing` | Nothing |
//!
//! A frame prints its `describe` header and first ten rows (section 3.1); the header alone is
//! printed until `describe` arrives (release checklist, step 19).

use crate::value::{Frame, Value};
use zeff_diagnostics::count;

/// The most items of a list or fields of a record that print; the rest is counted.
pub const LIST_LIMIT: usize = 20;

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
        Value::Var(name) | Value::Word(name) => name.clone(),
        Value::Vars(names) => names.join(", "),
        Value::Type(t) => match &t.format {
            Some(f) => format!("{}(\"{f}\")", t.name),
            None => t.name.clone(),
        },
    }
}

/// The header line of a frame: `wages · 18,412 rows × 13 variables`.
pub fn frame_header(frame: &Frame) -> String {
    let rows = frame.table.rows();
    let cols = frame.table.n_cols();
    format!(
        "{} · {} {} × {} {}",
        frame.name.as_deref().unwrap_or("frame"),
        count(rows as u64),
        if rows == 1 { "row" } else { "rows" },
        count(cols as u64),
        if cols == 1 { "variable" } else { "variables" }
    )
}

/// What a statement that is not a `let` prints for its value: the text, ending in a line break,
/// or nothing at all for `Nothing`.
pub fn show(value: &Value) -> String {
    match value {
        Value::Nothing => String::new(),
        Value::List(items) => lines(items.iter().map(inline), items.len(), "item"),
        Value::Record(fields) => lines(
            fields.iter().map(|(n, v)| format!("{n}: {}", inline(v))),
            fields.len(),
            "field",
        ),
        Value::Frame(f) => format!("{}\n", frame_header(f)),
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
    fn a_frame_prints_its_header() {
        let table = Table::new(vec![
            ("a".into(), zeff_columns::Column::ints([Some(1), Some(2)])),
            ("b".into(), zeff_columns::Column::ints([None, None])),
        ])
        .unwrap();
        let mut frame = Frame::new(table);
        assert_eq!(
            show(&Value::Frame(frame.clone())),
            "frame · 2 rows × 2 variables\n"
        );
        frame.name = Some("wages".into());
        assert_eq!(frame_header(&frame), "wages · 2 rows × 2 variables");
        let one = Frame::new(
            Table::new(vec![("a".into(), zeff_columns::Column::ints([Some(1)]))]).unwrap(),
        );
        assert_eq!(frame_header(&one), "frame · 1 row × 1 variable");
    }

    use zeff_columns::Table;
}
