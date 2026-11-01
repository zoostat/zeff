//! Package `time` (tier 1): dates and times (Engine and Project Specification, section 8.2;
//! Language Reference, section 3.8, "Time"). At this release the parsing functions `date` and
//! `datetime` and the date parts `year`, `month`, `day`, `weekday`, `quarter` and `week`
//! (release 0.6.0, step 3); the arithmetic and the clock follow at step 4. Every function is a
//! scalar function that applies row by row to a column (section 3.7, "Columns and scalars"),
//! and every one computes in the dates kernel, in integer arithmetic.

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_dates::{
    DateFormat, civil_from_days, days_of_micros, iso_week, micros_of_days, parse_iso_date,
    parse_iso_datetime, quarter_of, weekday_of,
};
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

/// The format of `format:`, parsed; `None` when it was not given. A text that is not a format is
/// Z0409 (Language Reference, section 11, the codes numbered while the engine was built).
fn format_of(
    ctx: &CallContext<'_>,
    args: &Args,
    function: &str,
) -> Result<Option<DateFormat>, Fail> {
    match args.get(1) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Str(pattern)) => DateFormat::new(pattern).map(Some).map_err(|e| {
            fail(
                Diagnostic::error("Z0409", format!("`{pattern}` is not a date format: {e}"))
                    .in_file(ctx.script)
                    .primary(ctx.span_of(1), "the format")
                    .help(format!(
                        "a format is written as `strftime` writes it, with %Y, %y, %m, %b, %d, %H, %M and %S: `{function}(s, format: \"%d/%m/%Y\")`"
                    )),
            )
        }),
        Some(_) => Err(internal(ctx, "a format that is not text")),
    }
}

/// W0203: the text did not read as a date or time, so the value is `NA`. Worded without the
/// text, so that the rows of a column count as one warning; `to(s, Date)` names the values.
fn unreadable(ctx: &mut CallContext<'_>, what: &str, format: Option<&DateFormat>) {
    let message = match format {
        Some(f) => format!(
            "text that does not match the format `{}` became NA",
            f.pattern()
        ),
        None => format!("text that is not {what} in ISO 8601 became NA"),
    };
    ctx.message(
        Diagnostic::warning("W0203", message)
            .help("`to(s, Date)` and `to(s, DateTime)` name the failing values; a format is written as `strftime` writes it, `format: \"%d/%m/%Y\"`"),
    );
}

/// W0702: the text carried `Z` or an offset, applied to reach UTC (section 3.8, "Time").
fn offset_applied(ctx: &mut CallContext<'_>) {
    ctx.message(
        Diagnostic::warning("W0702", "a time-zone offset was applied to reach UTC")
            .help("a DateTime has no zone in 1.0; the instant is kept, written in UTC"),
    );
}

/// `date(s, format:)`: text as a calendar day, from ISO 8601 or the format given.
fn date(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let format = format_of(ctx, &args, "date")?;
    let text = match args.get(0) {
        None | Some(Value::Na) => return Ok(Value::Na),
        Some(Value::Str(s)) => s.trim(),
        Some(_) => return Err(internal(ctx, "a value that is not text")),
    };
    let parsed = match &format {
        Some(f) => f.parse_date(text),
        None => parse_iso_date(text),
    };
    Ok(match parsed {
        Some(days) => Value::Date(days),
        None => {
            unreadable(ctx, "a date", format.as_ref());
            Value::Na
        }
    })
}

/// `datetime(s, format:)`: text as a date and time, from ISO 8601 (a date alone being midnight)
/// or the format given, `Z` or an offset applied to reach UTC.
fn datetime(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let format = format_of(ctx, &args, "datetime")?;
    let text = match args.get(0) {
        None | Some(Value::Na) => return Ok(Value::Na),
        Some(Value::Str(s)) => s.trim(),
        Some(_) => return Err(internal(ctx, "a value that is not text")),
    };
    if let Some(f) = &format {
        return Ok(match f.parse_datetime(text) {
            Some(micros) => Value::DateTime(micros),
            None => {
                unreadable(ctx, "a date and time", format.as_ref());
                Value::Na
            }
        });
    }
    if let Some(parsed) = parse_iso_datetime(text) {
        if parsed.had_offset {
            offset_applied(ctx);
        }
        return Ok(Value::DateTime(parsed.micros));
    }
    Ok(match parse_iso_date(text) {
        Some(days) => Value::DateTime(micros_of_days(days)),
        None => {
            unreadable(ctx, "a date and time", None);
            Value::Na
        }
    })
}

/// The day a date or time falls on, as days since the epoch; `None` for `NA`.
fn day_of(ctx: &CallContext<'_>, args: &Args) -> Result<Option<i32>, Fail> {
    match args.get(0) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Date(days)) => Ok(Some(*days)),
        Some(Value::DateTime(micros)) => Ok(Some(days_of_micros(*micros))),
        Some(_) => Err(internal(ctx, "a value that is not a date or a time")),
    }
}

fn part(ctx: &CallContext<'_>, args: &Args, f: fn(i32) -> i64) -> Result<Value, Fail> {
    Ok(day_of(ctx, args)?.map_or(Value::Na, |days| Value::Int(f(days))))
}

fn year(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| i64::from(civil_from_days(days).0))
}

fn month(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| i64::from(civil_from_days(days).1))
}

fn day(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| i64::from(civil_from_days(days).2))
}

fn weekday(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| i64::from(weekday_of(days)))
}

fn quarter(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| {
        i64::from(quarter_of(civil_from_days(days).1))
    })
}

fn week(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    part(ctx, &args, |days| i64::from(iso_week(days).1))
}

/// A date part: one parameter, a Date or a DateTime, and an Int.
fn date_part(name: &'static str, help: &'static str, body: zeff_core::runtime::Body) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "time",
            method: false,
            params: vec![ParamSpec::required("d", "Date | DateTime")],
            rest: None,
            returns: "Int",
            diagnostics: vec![],
            help,
            accuracy: None,
        },
        body,
    }
}

/// The functions of `time` in this release.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "date",
                package: "time",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::option("format", "Str"),
                ],
                rest: None,
                returns: "Date",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0203",
                        "the text is not a date in ISO 8601 or in the format, so the value is NA",
                    ),
                    DiagnosticDecl::new("Z0409", "format: is not a date format"),
                ],
                help: include_str!("../help/date.md"),
                accuracy: None,
            },
            body: date,
        },
        Function {
            spec: FunctionSpec {
                name: "datetime",
                package: "time",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::option("format", "Str"),
                ],
                rest: None,
                returns: "DateTime",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0203",
                        "the text is not a date and time in ISO 8601 or in the format, so the value is NA",
                    ),
                    DiagnosticDecl::new(
                        "W0702",
                        "the text carried a zone offset, which was applied to reach UTC",
                    ),
                    DiagnosticDecl::new("Z0409", "format: is not a date format"),
                ],
                help: include_str!("../help/datetime.md"),
                accuracy: None,
            },
            body: datetime,
        },
        date_part("year", include_str!("../help/year.md"), year),
        date_part("month", include_str!("../help/month.md"), month),
        date_part("day", include_str!("../help/day.md"), day),
        date_part("weekday", include_str!("../help/weekday.md"), weekday),
        date_part("quarter", include_str!("../help/quarter.md"), quarter),
        date_part("week", include_str!("../help/week.md"), week),
    ]
}
