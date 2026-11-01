//! Package `text` (tier 1): work with text (Engine and Project Specification, section 8.2). At
//! this release `format` (release 0.6.0, step 5): a number or a date to text, to a number of
//! decimals, with a thousands separator, or by a pattern; the other twelve functions follow at
//! 0.7.0. Every function is a scalar function that applies row by row to a column (Language
//! Reference, section 3.7, "Columns and scalars").

use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_dates::{format_date_with, format_datetime_with, format_iso_date, format_iso_datetime};
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// Z9001: a body reached with an argument the checker would have refused.
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

/// The options of a `format` call, read once.
struct Options<'a> {
    digits: Option<i64>,
    big_mark: Option<&'a str>,
    pattern: Option<&'a str>,
}

fn options<'a>(ctx: &CallContext<'_>, args: &'a Args) -> Result<Options<'a>, Fail> {
    let digits = match args.get(1) {
        None | Some(Value::Na) => None,
        Some(Value::Int(n)) => Some(*n),
        Some(_) => return Err(internal(ctx, "digits: that is not an Int")),
    };
    let big_mark = match args.get(2) {
        None | Some(Value::Na) => None,
        Some(Value::Str(s)) => Some(s.as_str()),
        Some(_) => return Err(internal(ctx, "big_mark: that is not text")),
    };
    let pattern = match args.get(3) {
        None | Some(Value::Na) => None,
        Some(Value::Str(s)) => Some(s.as_str()),
        Some(_) => return Err(internal(ctx, "pattern: that is not text")),
    };
    if pattern.is_some() && (digits.is_some() || big_mark.is_some()) {
        let other = if digits.is_some() {
            "digits:"
        } else {
            "big_mark:"
        };
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                format!("`format` takes `pattern:` for a date or `{other}` for a number, not both"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(3), "`pattern:` beside an option for numbers")
            .help("a date prints by its `pattern:`; a number by its `digits:` and `big_mark:`"),
        ));
    }
    if let Some(d) = digits
        && !(0..=20).contains(&d)
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`digits:` of `format` is a count of decimal places from 0 to 20, but this is {d}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "not a count of decimal places")
            .help("`digits: 2` writes two decimals: 3.14159 becomes 3.14"),
        ));
    }
    if big_mark == Some("") {
        return Err(fail(
            Diagnostic::error("Z0401", "`big_mark:` of `format` is empty")
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "nothing to put between the groups")
                .help("write the separator, `big_mark: \",\"` or `big_mark: \" \"`"),
        ));
    }
    Ok(Options {
        digits,
        big_mark,
        pattern,
    })
}

/// Z0401: an option that does not fit the value's type.
fn wrong_option(ctx: &CallContext<'_>, option: &str, at: usize, found: &Value, help: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{option}` of `format` does not apply to {}",
                found.type_word()
            ),
        )
        .in_file(ctx.script)
        .primary(
            ctx.span_of(at),
            format!(
                "an option for {}",
                if option == "pattern:" {
                    "dates"
                } else {
                    "numbers"
                }
            ),
        )
        .help(help),
    )
}

/// The whole part of a decimal text grouped in threes from the right with `mark`; the sign and
/// the fraction kept.
fn grouped(text: &str, mark: &str) -> String {
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text),
    };
    let (whole, fraction) = match rest.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (rest, None),
    };
    if !whole.chars().all(|c| c.is_ascii_digit()) {
        return text.to_string();
    }
    let mut out = String::from(sign);
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i).is_multiple_of(3) {
            out.push_str(mark);
        }
        out.push(c);
    }
    if let Some(f) = fraction {
        out.push('.');
        out.push_str(f);
    }
    out
}

/// A Float to `decimals` places, half to even on the exact binary value as `round` rounds, in the
/// plain form whatever its size; a negative zero loses its sign.
fn fixed(x: f64, decimals: usize) -> String {
    if x.is_nan() {
        return "NA".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_string();
    }
    let text = format!("{x:.decimals$}");
    if text.starts_with('-') && text[1..].chars().all(|c| c == '0' || c == '.') {
        text[1..].to_string()
    } else {
        text
    }
}

/// `format(x, digits:, big_mark:, pattern:)`.
fn format(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let options = options(ctx, &args)?;
    let x = args.get(0).cloned().unwrap_or(Value::Na);
    let number_help = "`digits:` and `big_mark:` shape a number; a date prints by its `pattern:`";
    let date_help = "`pattern:` prints a date or a time, as `pattern: \"%d/%m/%Y\"`; a number is shaped by `digits:` and `big_mark:`";
    let text = match &x {
        Value::Na => return Ok(Value::Na),
        Value::Int(_) | Value::Float(_) => {
            if options.pattern.is_some() {
                return Err(wrong_option(ctx, "pattern:", 3, &x, date_help));
            }
            let plain = match (&x, options.digits) {
                (Value::Int(i), None) => i.to_string(),
                (Value::Int(i), Some(d)) => fixed(*i as f64, d as usize),
                (Value::Float(f), None) => format_float(*f),
                (Value::Float(f), Some(d)) => fixed(*f, d as usize),
                _ => unreachable!("a number"),
            };
            match options.big_mark {
                Some(mark) => grouped(&plain, mark),
                None => plain,
            }
        }
        Value::Date(_) | Value::DateTime(_) => {
            if options.digits.is_some() {
                return Err(wrong_option(ctx, "digits:", 1, &x, number_help));
            }
            if options.big_mark.is_some() {
                return Err(wrong_option(ctx, "big_mark:", 2, &x, number_help));
            }
            let printed = match (&x, options.pattern) {
                (Value::Date(d), None) => Ok(format_iso_date(*d)),
                (Value::DateTime(t), None) => Ok(format_iso_datetime(*t)),
                (Value::Date(d), Some(p)) => format_date_with(p, *d),
                (Value::DateTime(t), Some(p)) => format_datetime_with(p, *t),
                _ => unreachable!("a date or a time"),
            };
            match printed {
                Ok(text) => text,
                Err(e) => {
                    let pattern = options.pattern.unwrap_or_default();
                    return Err(fail(
                        Diagnostic::error("Z0409", format!("`{pattern}` is not a date pattern: {e}"))
                            .in_file(ctx.script)
                            .primary(ctx.span_of(3), "the pattern")
                            .help("a pattern is written as `strftime` writes it, with %Y, %y, %m, %b, %B, %d, %a, %A, %j, %H, %M, %S and %%: `pattern: \"%b %Y\"`"),
                    ));
                }
            }
        }
        _ => {
            return Err(internal(
                ctx,
                "a value that is not a number, a date or a time",
            ));
        }
    };
    Ok(Value::Str(text))
}

/// The functions of `text` in this release.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "format",
            package: "text",
            method: false,
            params: vec![
                ParamSpec::required("x", "Int | Float | Date | DateTime"),
                ParamSpec::option("digits", "Int"),
                ParamSpec::option("big_mark", "Str"),
                ParamSpec::option("pattern", "Str"),
            ],
            rest: None,
            returns: "Str",
            diagnostics: vec![
                DiagnosticDecl::new(
                    "Z0401",
                    "an option that does not fit the value: digits: or big_mark: on a date, pattern: on a number; digits: outside 0 to 20; an empty big_mark:",
                ),
                DiagnosticDecl::new("Z0409", "pattern: is not a date pattern"),
                DiagnosticDecl::new("Z0410", "pattern: beside digits: or big_mark:"),
            ],
            help: include_str!("../help/format.md"),
            accuracy: None,
        },
        body: format,
    }]
}
