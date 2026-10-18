//! `frame(name = [values], ...)`: a frame from named lists of values (Language Reference,
//! sections 3.7 and 4.1).
//!
//! Each definition is one column. Its values are one type: Bool, Int, Float or Str, with Int and
//! Float joining as Float and `NA` fitting any of them; every list has the same length.

use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::{Diagnostic, Span, count};

pub fn frame(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let mut columns: Vec<(String, Column)> = Vec::with_capacity(args.definitions.len());
    let mut first: Option<(String, usize, Span)> = None;
    for (i, (name, value)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        // A column value (`wages["wage"]`, or an expression over columns) is a column as it is.
        if let Value::Column(column) = value {
            let n = column.len();
            match &first {
                None => first = Some((name.clone(), n, span)),
                Some((first_name, m, first_span)) if *m != n => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0217",
                            format!(
                                "`frame` needs lists of one length: `{first_name}` has {} {} and `{name}` has {}",
                                count(*m as u64),
                                if *m == 1 { "value" } else { "values" },
                                count(n as u64)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(span, format!("{} here", count(n as u64)))
                        .secondary(*first_span, format!("{} here", count(*m as u64)))
                        .help("give every column the same number of values; write `NA` for a value that is missing"),
                    ));
                }
                Some(_) => {}
            }
            columns.push((name.clone(), column.clone()));
            continue;
        }
        let Value::List(items) = value else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{name}` must be a list of values, not {}",
                        article(value.type_name())
                    ),
                )
                .in_file(ctx.script)
                .primary(span, "not a list")
                .help(format!(
                    "write the values in brackets: `{name} = [1, 2, 3]`"
                )),
            ));
        };
        let column = column_from(items, name, span, ctx.script)?;
        match &first {
            None => first = Some((name.clone(), items.len(), span)),
            Some((first_name, n, first_span)) if *n != items.len() => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0217",
                        format!(
                            "`frame` needs lists of one length: `{first_name}` has {} {} and `{name}` has {}",
                            count(*n as u64),
                            if *n == 1 { "value" } else { "values" },
                            count(items.len() as u64)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(span, format!("{} here", count(items.len() as u64)))
                    .secondary(*first_span, format!("{} here", count(*n as u64)))
                    .help("give every column the same number of values; write `NA` for a value that is missing"),
                ));
            }
            Some(_) => {}
        }
        columns.push((name.clone(), column));
    }
    let table = Table::new(columns).map_err(|e| {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script ({e})"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )
    })?;
    Ok(Value::Frame(Frame::new(table)))
}

/// One column from the values of a list.
fn column_from(items: &[Value], name: &str, span: Span, script: &str) -> Result<Column, Fail> {
    let (mut bools, mut ints, mut floats, mut strs, mut nas) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for item in items {
        match item {
            Value::Bool(_) => bools += 1,
            Value::Int(_) => ints += 1,
            Value::Float(_) => floats += 1,
            Value::Str(_) => strs += 1,
            Value::Na => nas += 1,
            other => {
                return Err(fail(
                    Diagnostic::error("Z0401", format!("`{name}` holds {}; a column holds Bool, Int, Float or Str values", article(other.type_name())))
                        .in_file(script)
                        .primary(span, "not a value a column can hold")
                        .help("a frame's columns are flat lists of values; nested lists and records arrive with the opaque column types after 1.0"),
                ));
            }
        }
    }
    if items.is_empty() {
        return Err(fail(
            Diagnostic::error(
                "Z0219",
                format!("`{name}` has no values, so its type is unknown"),
            )
            .in_file(script)
            .primary(span, "an empty list of what?")
            .help("give the column at least one value"),
        ));
    }
    if nas == items.len() {
        return Err(fail(
            Diagnostic::error(
                "Z0219",
                format!("`{name}` holds only `NA`, so its type is unknown"),
            )
            .in_file(script)
            .primary(span, "NA of what type?")
            .help("give the column at least one value beside the `NA`s"),
        ));
    }
    let kinds = usize::from(bools > 0) + usize::from(strs > 0) + usize::from(ints + floats > 0);
    if kinds > 1 {
        let present: Vec<&str> = [
            (bools, "Bool"),
            (ints, "Int"),
            (floats, "Float"),
            (strs, "Str"),
        ]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(_, t)| *t)
        .collect();
        return Err(fail(
            Diagnostic::error(
                "Z0214",
                format!(
                    "`{name}` holds values of more than one type: {}",
                    present.join(", ")
                ),
            )
            .in_file(script)
            .primary(span, "a column holds one type")
            .help("put values of one type in each column; Int and Float join as Float"),
        ));
    }
    Ok(if bools > 0 {
        Column::bools(items.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        }))
    } else if strs > 0 {
        Column::strs(items.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }))
    } else if floats > 0 {
        Column::floats(items.iter().map(|v| match v {
            Value::Float(x) => Some(*x),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }))
        .0
    } else {
        Column::ints(items.iter().map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        }))
    })
}

fn article(type_name: &str) -> String {
    // `NA` is said letter by letter, so it takes `an`.
    let vowel = type_name == "NA"
        || type_name.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']);
    format!("{} {type_name}", if vowel { "an" } else { "a" })
}
