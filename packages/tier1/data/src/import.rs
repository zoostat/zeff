//! `import(path, ...)`: read a data file into a frame (Language Reference, section 6.1).

use crate::csv::{CsvOptions, NameRule, TypeSpec, read_csv};
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_dates::DateFormat;
use zeff_diagnostics::{Diagnostic, count};

/// The positions of `import`'s parameters.
const PATH: usize = 0;
const DELIM: usize = 1;
const HEADER: usize = 2;
const SKIP: usize = 3;
const NA_VALUES: usize = 4;
const TYPES: usize = 5;
const NAMES: usize = 6;

pub fn import(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let Some(Value::Str(path)) = args.take(PATH) else {
        return Err(internal(ctx, "import was called without a path"));
    };
    let mut options = CsvOptions::default();

    if let Some(Value::Str(delim)) = args.get(DELIM) {
        let mut chars = delim.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_ascii() && c != '"' && c != '\n' && c != '\r' => {
                options.delim = Some(c as u8)
            }
            _ => {
                return Err(fail(
                    Diagnostic::error("Z0401", format!("`delim:` takes one character, not \"{delim}\""))
                        .in_file(ctx.script)
                        .primary(ctx.span_of(DELIM), "not a delimiter")
                        .help("write the one character between the fields, as in `delim: \";\"` or `delim: \"\\t\"`"),
                ));
            }
        }
    }
    if let Some(Value::Bool(header)) = args.get(HEADER) {
        options.header = *header;
    }
    if let Some(Value::Int(skip)) = args.get(SKIP) {
        options.skip = usize::try_from(*skip).map_err(|_| {
            fail(
                Diagnostic::error("Z0401", format!("`skip:` cannot be {skip}"))
                    .in_file(ctx.script)
                    .primary(ctx.span_of(SKIP), "a count of lines")
                    .help("write the number of lines to skip at the top of the file, 0 or more"),
            )
        })?;
    }
    if let Some(Value::List(values)) = args.get(NA_VALUES) {
        for value in values {
            options.na_values.push(match value {
                Value::Int(i) => i.to_string(),
                Value::Float(x) => shortest(*x),
                Value::Str(s) => s.clone(),
                Value::Na => continue,
                other => {
                    return Err(fail(
                        Diagnostic::error("Z0401", format!("`na_values:` takes numbers and text, not {}", article(other.type_name())))
                            .in_file(ctx.script)
                            .primary(ctx.span_of(NA_VALUES), "not a missing-value code")
                            .help("write the codes as they appear in the file: `na_values: [-99, \"varies\"]`"),
                    ));
                }
            });
        }
    }
    if let Some(Value::Record(fields)) = args.get(TYPES) {
        for (name, value) in fields {
            let Value::Type(ty) = value else {
                return Err(fail(
                    Diagnostic::error("Z0401", format!("`types:` takes type names, but `{name}` is {}", article(value.type_name())))
                        .in_file(ctx.script)
                        .primary(ctx.span_of(TYPES), "not a type")
                        .help("write a type for each column, as in `types: {hours: Str, sold: Date(\"%d/%m/%Y\")}`"),
                ));
            };
            let format = ty
                .format
                .as_deref()
                .map(DateFormat::new)
                .transpose()
                .map_err(|e| internal(ctx, &format!("a checked format failed to parse: {e}")))?;
            let spec = match (ty.name.as_str(), format) {
                ("Bool", _) => TypeSpec::Bool,
                ("Int", _) => TypeSpec::Int,
                ("Float", _) => TypeSpec::Float,
                ("Str", _) => TypeSpec::Str,
                ("Date", f) => TypeSpec::Date(f),
                ("DateTime", f) => TypeSpec::DateTime(f),
                (other, _) => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!("`{other}` is not a type a column can have"),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(TYPES), "not a column type")
                        .help("a column is Bool, Int, Float, Str, Date or DateTime"),
                    ));
                }
            };
            options.types.push((name.clone(), spec));
        }
    }
    if let Some(Value::Word(word)) = args.get(NAMES) {
        options.names = if word == "snake" {
            NameRule::Snake
        } else {
            NameRule::Keep
        };
    }

    let path_span = ctx.span_of(PATH);
    let bytes = ctx.files.read(&path, path_span, ctx.script)?;
    let read = read_csv(&bytes, &options)
        .map_err(|e| fail(e.diagnostic(&path, ctx.callee_span, ctx.script)))?;
    ctx.message(Diagnostic::plain_note(format_note(
        &path,
        read.table.rows(),
        read.table.n_cols(),
    )));
    for message in read.messages {
        ctx.message(message);
    }
    // Every reader fills the `source` key (section 4.3, rule 3) with the path and the BLAKE3 hash
    // of the file's bytes, the hash the run record will carry (Engine Specification, section 8.3,
    // point 10).
    let source = format!("{path} {}", zeff_hash::labelled(&bytes));
    Ok(Value::Frame(Frame::new(read.table.with_source(&source))))
}

/// The one note every import prints (Language Reference, section 6.1): what the format carried
/// and what it could not. A CSV carries values alone.
pub fn format_note(path: &str, rows: usize, cols: usize) -> String {
    format!(
        "{path} read as CSV: {} {} × {} {}, types guessed; CSV carries no labels, value labels or categories",
        count(rows as u64),
        if rows == 1 { "row" } else { "rows" },
        count(cols as u64),
        if cols == 1 { "variable" } else { "variables" }
    )
}

/// The shortest text that reads back to the same number, for a code given as a Float.
fn shortest(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{x:.0}")
    } else {
        format_float(x)
    }
}

fn article(type_name: &str) -> String {
    // `NA` is said letter by letter, so it takes `an`.
    let vowel = type_name == "NA"
        || type_name.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']);
    format!("{} {type_name}", if vowel { "an" } else { "a" })
}

/// An argument the checker should have refused: a bug in Zeff, not in the script (Z9001).
fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}
